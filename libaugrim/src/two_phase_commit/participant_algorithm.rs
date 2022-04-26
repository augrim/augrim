// Copyright 2021-2022 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::marker::PhantomData;
use std::time::Duration;

use crate::algorithm::{Algorithm, Value};
use crate::error::{AlgorithmError, InvalidStateError};
use crate::process::Process;
use crate::time::TimeSource;

use super::ParticipantAction;
use super::ParticipantActionNotification;
use super::ParticipantContext;
use super::ParticipantEvent;
use super::ParticipantMessage;
use super::ParticipantState;
use super::TwoPhaseCommitContext;
use super::TwoPhaseCommitMessage;

const DECISION_TIMEOUT_SECONDS: u64 = 30;

pub struct ParticipantAlgorithm<P, V, TS>
where
    P: Process,
    V: Value,
    TS: TimeSource,
{
    _process_phantom: PhantomData<P>,
    _value_phantom: PhantomData<V>,
    time_source: TS,
}

impl<P, V, TS> ParticipantAlgorithm<P, V, TS>
where
    P: Process,
    V: Value,
    TS: TimeSource,
{
    pub fn new(time_source: TS) -> Self {
        ParticipantAlgorithm {
            _process_phantom: PhantomData,
            _value_phantom: PhantomData,
            time_source,
        }
    }
}

impl<P, V, TS> Algorithm for ParticipantAlgorithm<P, V, TS>
where
    P: Process,
    V: Value,
    TS: TimeSource,
{
    type Event = ParticipantEvent<P, V>;
    type Action = ParticipantAction<P, V, TS::Time>;
    type Context = TwoPhaseCommitContext<P, TS::Time, ParticipantContext<P, TS::Time>>;

    fn event(
        &self,
        event: Self::Event,
        mut context: Self::Context,
    ) -> Result<Vec<Self::Action>, AlgorithmError> {
        match event {
            ParticipantEvent::Alarm() => match context.state() {
                // Receiving alarms in the Abort state is unexpected and indicates a bug in the
                // caller.
                ParticipantState::Abort => Err(AlgorithmError::InvalidState(
                    InvalidStateError::with_message("Alarm unexpected in Abort state".into()),
                )),

                // Receiving alarms in the Commit state is unexpected and indicates a bug in the
                // caller.
                ParticipantState::Commit => Err(AlgorithmError::InvalidState(
                    InvalidStateError::with_message("Alarm unexpected in Commit state".into()),
                )),

                // A vote timeout has occurred, which means we have not received a decision within
                // DECISION_TIMEOUT_SECONDS.
                ParticipantState::Voted {
                    vote,
                    decision_timeout_start,
                } => {
                    let mut actions = Vec::new();

                    // Validate that the timeout has occurred. If this is false, we shouldn't have
                    // been woken up with an alarm; however, we can just ignore it and wait for the
                    // alarm to be triggered again later.
                    if self.time_source.now()
                        > *decision_timeout_start + Duration::from_secs(DECISION_TIMEOUT_SECONDS)
                    {
                        // Send a Decision Request to all other participant processes
                        for process in context
                            .participant_processes()
                            .iter()
                            .filter(|p| *p != context.this_process())
                        {
                            actions.push(ParticipantAction::SendMessage(
                                process.clone(),
                                TwoPhaseCommitMessage::DecisionRequest(*context.epoch()),
                            ));
                        }

                        // Send a Decision Request to the coordinator
                        actions.push(ParticipantAction::SendMessage(
                            context.coordinator().clone(),
                            TwoPhaseCommitMessage::DecisionRequest(*context.epoch()),
                        ));

                        // Calculate new decision timeout start/end.
                        let new_decision_timeout_start = self.time_source.now();
                        let new_decision_timeout_end = new_decision_timeout_start
                            + Duration::from_secs(DECISION_TIMEOUT_SECONDS);

                        // Updated the Voted state with the new timeout start value.
                        let mut new_context = context.clone();
                        new_context.set_state(ParticipantState::Voted {
                            vote: *vote,
                            decision_timeout_start: new_decision_timeout_start,
                        });
                        actions.push(ParticipantAction::Update {
                            context: new_context,
                            alarm: Some(new_decision_timeout_end),
                        });
                    }

                    Ok(actions)
                }

                // An Alarm while in WaitingForVote is not allowed and indicates a bug in the
                // caller.
                ParticipantState::WaitingForVote => Err(AlgorithmError::InvalidState(
                    InvalidStateError::with_message("Alarm unexpected in WaitForVote state".into()),
                )),

                // An Alarm while in WaitingForVoteRequest is not allowed and indicates a bug in
                // the caller.
                ParticipantState::WaitingForVoteRequest => Err(AlgorithmError::InvalidState(
                    InvalidStateError::with_message(
                        "Alarm unexpected in WaitForVoteRequest state".into(),
                    ),
                )),
            },

            // If the coordinator sends a VoteRequest, generate a RequestForVote
            // notification to determine how to respond and update our state.
            ParticipantEvent::Deliver(process, ParticipantMessage::VoteRequest(epoch, value)) => {
                // A VoteRequest must come from the coordinator, drop it otherwise.
                if *context.coordinator() != process {
                    return Ok(vec![ParticipantAction::Notify(
                        ParticipantActionNotification::MessageDropped(
                            "sender process is not the coordinator".into(),
                        ),
                    )]);
                }

                // A VoteRequest must be for the current epoch to be processed, drop it otherwise.
                if *context.epoch() != epoch {
                    return Ok(vec![ParticipantAction::Notify(
                        ParticipantActionNotification::MessageDropped(
                            "epoch is not the current epoch".into(),
                        ),
                    )]);
                }

                // A VoteRequest can only be processed when we are waiting for one, drop it
                // otherwise.
                if !matches!(context.state(), ParticipantState::WaitingForVoteRequest) {
                    return Ok(vec![ParticipantAction::Notify(
                        ParticipantActionNotification::MessageDropped(
                            "context state is not WaitingForVoteRequest".into(),
                        ),
                    )]);
                }

                let mut actions = Vec::new();

                // Update the context with the new state of WaitingForVote
                context.set_state(ParticipantState::WaitingForVote);
                actions.push(ParticipantAction::Update {
                    context,
                    alarm: None,
                });

                // Send a RequestForVote notification
                actions.push(ParticipantAction::Notify(
                    ParticipantActionNotification::RequestForVote(value),
                ));

                Ok(actions)
            }
            ParticipantEvent::Deliver(_process, ParticipantMessage::Commit(epoch)) => {
                // A Commit must be for the current epoch to be processed, drop it otherwise.
                if *context.epoch() != epoch {
                    return Ok(vec![ParticipantAction::Notify(
                        ParticipantActionNotification::MessageDropped(
                            "epoch is not the current epoch".into(),
                        ),
                    )]);
                }

                if !matches!(
                    context.state(),
                    ParticipantState::Voted {
                        vote: _,
                        decision_timeout_start: _
                    }
                ) {
                    return Ok(vec![ParticipantAction::Notify(
                        ParticipantActionNotification::MessageDropped(
                            "commit received outside decision window".into(),
                        ),
                    )]);
                }

                let mut actions = Vec::new();

                // The vote was no, so record our decision to Abort.
                context.set_state(ParticipantState::Commit);
                actions.push(ParticipantAction::Update {
                    context: context.clone(),
                    alarm: None,
                });

                Ok(actions)
            }
            ParticipantEvent::Deliver(_process, ParticipantMessage::Abort(epoch)) => {
                // An Abort must be for the current epoch to be processed, drop it otherwise.
                if *context.epoch() != epoch {
                    return Ok(vec![ParticipantAction::Notify(
                        ParticipantActionNotification::MessageDropped(
                            "epoch is not the current epoch".into(),
                        ),
                    )]);
                }

                if !matches!(
                    context.state(),
                    ParticipantState::Voted {
                        vote: _,
                        decision_timeout_start: _
                    }
                ) {
                    return Ok(vec![ParticipantAction::Notify(
                        ParticipantActionNotification::MessageDropped(
                            "abort received outside decision window".into(),
                        ),
                    )]);
                }

                let mut actions = Vec::new();

                // The vote was no, so record our decision to Abort.
                context.set_state(ParticipantState::Abort);
                actions.push(ParticipantAction::Update {
                    context: context.clone(),
                    alarm: None,
                });

                Ok(actions)
            }

            ParticipantEvent::Deliver(process, ParticipantMessage::DecisionRequest(epoch)) => {
                // The sender must be a coordinator or participant.
                if !(context.participant_processes().contains(&process)
                    && *context.coordinator() != process)
                {
                    return Ok(vec![ParticipantAction::Notify(
                        ParticipantActionNotification::MessageDropped(
                            "sender process is not a coordinator or participant".into(),
                        ),
                    )]);
                }

                // We record the last commit epoch in the context; if the epoch requested was the
                // last commit epoch, send a commit message to the requesting process.
                if Some(epoch) == *context.last_commit_epoch() {
                    return Ok(vec![ParticipantAction::SendMessage(
                        process,
                        TwoPhaseCommitMessage::Commit(epoch),
                    )]);
                }

                // If the epoch is between the current epoch and the last commit epoch, we know
                // that the decision must have been Abort. Thus, we send an Abort message.
                if epoch < *context.epoch()
                    && (Some(epoch) > *context.last_commit_epoch()
                        || context.last_commit_epoch().is_none())
                {
                    return Ok(vec![ParticipantAction::SendMessage(
                        process,
                        TwoPhaseCommitMessage::Abort(epoch),
                    )]);
                }

                // A note on ignored messages:
                //
                // If the epoch is before the last commit epoch, we ignore the message as we know
                // all processes decided in the last commit epoch and no process can be in an
                // uncertainty period for an older epoch.
                //
                // If the epoch is after our current epoch, we ignore the message as we do not yet
                // know what the future holds. Similarly, we do not yet have a decision for the
                // current epoch or we would have advanced to the next epoch already.
                Ok(vec![ParticipantAction::Notify(
                    ParticipantActionNotification::MessageDropped(
                        "decision for requested epoch is unknown".into(),
                    ),
                )])
            }

            // In response to a RequestForVote, a Vote message contains either true (vote yes) or
            // false (vote no).
            ParticipantEvent::Vote(vote) => {
                // If we receive a Vote event when not in WaitingForVote, it indicates
                // a programming error by the caller of the algorithm.
                if !matches!(context.state(), ParticipantState::WaitingForVote) {
                    return Err(AlgorithmError::InvalidState(
                        InvalidStateError::with_message(
                            "Vote event when not in WaitingForVote state".into(),
                        ),
                    ));
                }

                let mut actions = Vec::new();

                if vote {
                    // A timeout will occur after DECISION_TIMEOUT_SECONDS, starting now. An alarm is
                    // set for the end of the timeout and the timeout is processed when an
                    // `ParticipantEvent::Alarm` is received.
                    let decision_timeout_start = self.time_source.now();
                    let decision_timeout_end =
                        decision_timeout_start + Duration::from_secs(DECISION_TIMEOUT_SECONDS);

                    // Record the vote and update the state to Voted.
                    context.set_state(ParticipantState::Voted {
                        vote,
                        decision_timeout_start,
                    });
                    actions.push(ParticipantAction::Update {
                        context: context.clone(),
                        alarm: Some(decision_timeout_end),
                    });
                } else {
                    // The vote was no, so record our decision to Abort.
                    context.set_state(ParticipantState::Abort);
                    actions.push(ParticipantAction::Update {
                        context: context.clone(),
                        alarm: None,
                    });
                }

                // Send the vote to the coordinator.
                actions.push(ParticipantAction::SendMessage(
                    context.coordinator().clone(),
                    TwoPhaseCommitMessage::VoteResponse(*context.epoch(), vote),
                ));

                Ok(actions)
            }
        }
    }
}
