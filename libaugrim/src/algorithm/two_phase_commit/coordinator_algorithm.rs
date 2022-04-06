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
use crate::error::AlgorithmError;
use crate::process::Process;
use crate::time::TimeSource;

use super::CoordinatorAction;
use super::CoordinatorActionAlarm;
use super::CoordinatorActionNotification;
use super::CoordinatorContext;
use super::CoordinatorEvent;
use super::CoordinatorMessage;
use super::CoordinatorState;
use super::TwoPhaseCommitMessage;

const VOTE_TIMEOUT_SECONDS: u64 = 30;

pub struct CoordinatorAlgorithm<P, V, TS>
where
    P: Process,
    V: Value,
    TS: TimeSource,
{
    _process_phantom: PhantomData<P>,
    _value_phantom: PhantomData<V>,
    time_source: TS,
}

impl<P, V, TS> CoordinatorAlgorithm<P, V, TS>
where
    P: Process,
    V: Value,
    TS: TimeSource,
{
    pub fn new(time_source: TS) -> Self {
        CoordinatorAlgorithm {
            _process_phantom: PhantomData,
            _value_phantom: PhantomData,
            time_source,
        }
    }

    // Create actions for an abort decision. This set of actions is generated whenever an abort
    // occurs; an abort occurs when:
    //
    // - During a timeout when not all participants have voted
    // - When all participants have voted but at least one voted NO
    fn push_abort_actions(
        &self,
        mut context: CoordinatorContext<P, TS::Time>,
        actions: &mut Vec<CoordinatorAction<P, V, TS::Time>>,
    ) {
        // The order of actions here is important! We must update our state to `Abort` before we
        // send any messages for correctness of the algorithm.

        // Add an action to update the state to abort and unset the alarm.
        context.set_state(CoordinatorState::Abort);
        actions.push(CoordinatorAction::Update(
            context.clone(),
            CoordinatorActionAlarm::Unset,
        ));

        // Send `Abort` to all participants which have voted yes.
        for participant in context
            .participants()
            .iter()
            .filter(|p| p.vote.unwrap_or(false))
        {
            actions.push(CoordinatorAction::SendMessage(
                participant.process.clone(),
                TwoPhaseCommitMessage::Abort(*context.epoch()),
            ))
        }

        // Notify that we've aborted.
        actions.push(CoordinatorAction::Notify(
            CoordinatorActionNotification::Abort(),
        ));

        // When an abort decision is made, always advance the epoch as well.
        self.push_advance_epoch_actions(context, actions);
    }

    // Create actions for advancing to the next epoch. This set of actions is generated whenever
    // a decision has been reached, either abort or commit.
    fn push_advance_epoch_actions(
        &self,
        mut context: CoordinatorContext<P, TS::Time>,
        actions: &mut Vec<CoordinatorAction<P, V, TS::Time>>,
    ) {
        // Update the epoch and set the state to WaitingForStart. Also update the last commit epoch
        // used to answer DecisionRequest messages.
        context.set_last_commit_epoch(Some(*context.epoch()));
        context.set_epoch(context.epoch() + 1);
        context.set_state(CoordinatorState::WaitingForStart);
        actions.push(CoordinatorAction::Update(
            context,
            CoordinatorActionAlarm::Unset,
        ));

        // Notify that we need a new start value.
        actions.push(CoordinatorAction::Notify(
            CoordinatorActionNotification::RequestForStart(),
        ));
    }
}

impl<P, V, TS> Algorithm<P> for CoordinatorAlgorithm<P, V, TS>
where
    P: Process,
    V: Value,
    TS: TimeSource,
{
    type Event = CoordinatorEvent<P, V>;
    type Action = CoordinatorAction<P, V, TS::Time>;
    type Context = CoordinatorContext<P, TS::Time>;

    fn event(
        &self,
        event: Self::Event,
        mut context: Self::Context,
    ) -> Result<Vec<Self::Action>, AlgorithmError> {
        match event {
            // In response to a RequestForStart notification, a Start event provides the next value
            // that should be considered.
            //
            // Steps:
            //   - Send VoteRequest to all participants
            //   - Update the state to Voting
            //   - Set a timeout alarm for the maximum time to wait for votes
            CoordinatorEvent::Start(value) => {
                let mut actions = Vec::new();

                // Send a VoteRequest message to all participants
                for participant in context.participants() {
                    actions.push(CoordinatorAction::SendMessage(
                        participant.process.clone(),
                        TwoPhaseCommitMessage::VoteRequest(*context.epoch(), value.clone()),
                    ))
                }

                // A timeout will occur after VOTE_TIMEOUT_SECONDS, starting now. An alarm is set
                // for the end of the timeout and the timeout is processed when an
                // `CoordinatorEvent::Alarm` is received.
                let vote_timeout_start = self.time_source.now();
                let vote_timeout_end =
                    vote_timeout_start + Duration::from_secs(VOTE_TIMEOUT_SECONDS);

                // Add an action to update the state to Voting and set the timeout alarm.
                context.set_state(CoordinatorState::Voting { vote_timeout_start });
                actions.push(CoordinatorAction::Update(
                    context,
                    CoordinatorActionAlarm::Set(vote_timeout_end),
                ));

                Ok(actions)
            }

            // In response to a RequestForVote notification, a Vote event provides the answer to
            // whether we decide commit or abort.
            CoordinatorEvent::Vote(vote) => {
                let mut actions = Vec::new();

                // If vote is true, then we decide to commit; if vote is false, we decide to abort.
                if vote {
                    // Add an action to update the state to commit and unset the alarm.
                    context.set_state(CoordinatorState::Commit);
                    actions.push(CoordinatorAction::Update(
                        context.clone(),
                        CoordinatorActionAlarm::Unset,
                    ));

                    // Send `Commit` to all participants.
                    for participant in context.participants() {
                        actions.push(CoordinatorAction::SendMessage(
                            participant.process.clone(),
                            TwoPhaseCommitMessage::Commit(*context.epoch()),
                        ))
                    }

                    // Always advance the epoch immediately after a commit decision.
                    self.push_advance_epoch_actions(context, &mut actions);
                } else {
                    self.push_abort_actions(context, &mut actions);
                }

                Ok(actions)
            }

            // An alarm may be sent if we've previously used the `CoordinatorAction::Update` action
            // to set an alarm to `CoordinatorActionAlarm::Set`, as is the case when we enter the
            // `Coordinator::Voting` state.
            CoordinatorEvent::Alarm() => match context.state() {
                // A vote timeout has occurred, which means we have not received votes within
                // VOTE_TIMEOUT_SECONDS.
                CoordinatorState::Voting { vote_timeout_start } => {
                    let mut actions = Vec::new();

                    // Validate that the timeout has occurred. If this is false, we shouldn't have
                    // been woken up with an alarm; however, we can just ignore it and wait for the
                    // alarm to be triggered again later.
                    if self.time_source.now()
                        > *vote_timeout_start + Duration::from_secs(VOTE_TIMEOUT_SECONDS)
                    {
                        // Decide to abort. Use a function to fill in the abort actions since abort
                        // can occur in other situations as well.
                        self.push_abort_actions(context, &mut actions);
                    }

                    Ok(actions)
                }

                // If we receive an alarm an the RequestforStart state, then we re-generate
                // a RequestForStart notification.
                CoordinatorState::WaitingForStart => Ok(vec![CoordinatorAction::Notify(
                    CoordinatorActionNotification::RequestForStart(),
                )]),

                // If we receive an alarm an the RequestforStart state, then we re-generate
                // a RequestForVote notification.
                CoordinatorState::WaitingForVote => Ok(vec![CoordinatorAction::Notify(
                    CoordinatorActionNotification::RequestForVote(),
                )]),

                // Receiving alarms in the commit state is unexpected, but try and recover by
                // advancing to the next epoch.
                CoordinatorState::Commit => {
                    let mut actions = Vec::new();
                    self.push_advance_epoch_actions(context, &mut actions);
                    Ok(actions)
                }

                // Receiving alarms in the abort state is unexpected, but try and recover by
                // advancing to the next epoch.
                CoordinatorState::Abort => {
                    let mut actions = Vec::new();
                    self.push_advance_epoch_actions(context, &mut actions);
                    Ok(actions)
                }
            },

            // A participant has sent response to our request for a vote, record it and possibly
            // decide commit or abort.
            CoordinatorEvent::Deliver(process, CoordinatorMessage::VoteResponse(epoch, vote)) => {
                // Pull these out of context and copy/clone them because we borrow context to get
                // a mut participant prior to using these values for additional checks.
                let context_epoch = *context.epoch();
                let context_state = context.state().clone();

                let mut participant = match context
                    .participants_mut()
                    .iter_mut()
                    .find(|participant| participant.process == process)
                {
                    Some(inner) => inner,
                    None => {
                        return Ok(vec![CoordinatorAction::Notify(
                            CoordinatorActionNotification::MessageDropped(
                                "sender process is not a participant".into(),
                            ),
                        )]);
                    }
                };

                // Ignore the message if the vote's epoch doesn't match our context epoch; this
                // could happen under normal operation if a vote was processed after a timeout, and
                // is therefore not an error.
                if context_epoch != epoch {
                    return Ok(vec![CoordinatorAction::Notify(
                        CoordinatorActionNotification::MessageDropped(
                            "epoch is not the current epoch".into(),
                        ),
                    )]);
                }

                // Ignore the message if we are not in the voting window. This is unlikely to occur
                // in practice because we will have advanced the epoch and the epoch is checked
                // above; however, this could occur if not all Update actions were run
                // successfully.
                if matches!(
                    context_state,
                    CoordinatorState::Voting {
                        vote_timeout_start: _,
                    }
                ) {
                    return Ok(vec![CoordinatorAction::Notify(
                        CoordinatorActionNotification::MessageDropped(
                            "context state is not Voting".into(),
                        ),
                    )]);
                }

                // Ignore if this participant already voted. This should not occur in normal
                // operation.
                if participant.vote.is_some() {
                    return Ok(vec![CoordinatorAction::Notify(
                        CoordinatorActionNotification::MessageDropped(
                            "participant has already voted".into(),
                        ),
                    )]);
                }

                let mut actions = Vec::new();

                // Update the context to record the participant's vote
                participant.vote = Some(vote);
                actions.push(CoordinatorAction::Update(
                    context.clone(),
                    CoordinatorActionAlarm::Unset,
                ));

                // If all the participants have voted, then either decide to abort or change state.
                if context.participants().iter().all(|p| p.vote.is_some()) {
                    if context.participants().iter().any(|p| p.vote == Some(false)) {
                        // We got at least one NO vote, so decide to abort. Use a function to fill
                        // in the abort since abort can occur in other situations as well.
                        self.push_abort_actions(context, &mut actions)
                    } else {
                        // All participants voted yes, so we provide one last opportunity for the
                        // coordinator to vote no by waiting for the coordinators vote.
                        context.set_state(CoordinatorState::WaitingForVote);
                        actions.push(CoordinatorAction::Update(
                            context,
                            CoordinatorActionAlarm::Unset,
                        ));
                        actions.push(CoordinatorAction::Notify(
                            // Notify that we are requesting a coordinator vote.
                            CoordinatorActionNotification::RequestForVote(),
                        ));
                    }
                }

                Ok(actions)
            }

            // A node which has timed out in its uncertainty period will send a `DecisionRequest`
            // as part of its termination protocol. If we have the information, answer this request
            // with a commit or abort message.
            CoordinatorEvent::Deliver(process, CoordinatorMessage::DecisionRequest(epoch)) => {
                // The sender must be a participant.
                if context
                    .participants()
                    .iter()
                    .any(|participant| participant.process != process)
                {
                    return Ok(vec![CoordinatorAction::Notify(
                        CoordinatorActionNotification::MessageDropped(
                            "sender process is not a participant".into(),
                        ),
                    )]);
                }

                // We record the last commit epoch in the context; if the epoch requested was the
                // last commit epoch, send a commit message to the requesting process.
                if Some(epoch) == *context.last_commit_epoch() {
                    return Ok(vec![CoordinatorAction::SendMessage(
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
                    return Ok(vec![CoordinatorAction::SendMessage(
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
                Ok(vec![CoordinatorAction::Notify(
                    CoordinatorActionNotification::MessageDropped(
                        "decision for requested epoch is unknown".into(),
                    ),
                )])
            }
        }
    }
}
