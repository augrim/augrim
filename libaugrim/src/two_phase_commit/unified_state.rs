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

use crate::error::InvalidStateError;
use crate::time::Time;

use super::CoordinatorState;
use super::ParticipantState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TwoPhaseCommitState<T> {
    Abort,
    Commit,
    Voted {
        vote: bool,
        decision_timeout_start: T,
    },
    Voting {
        vote_timeout_start: T,
    },
    WaitingForStart,
    WaitingForVoteRequest,
    WaitingForVote,
    WaitingForDecisionAck {
        ack_timeout_start: T,
    },
}

impl<T> TryFrom<TwoPhaseCommitState<T>> for CoordinatorState<T>
where
    T: Time,
{
    type Error = InvalidStateError;

    fn try_from(state: TwoPhaseCommitState<T>) -> Result<Self, InvalidStateError> {
        match state {
            TwoPhaseCommitState::Abort => Ok(CoordinatorState::Abort),
            TwoPhaseCommitState::Commit => Ok(CoordinatorState::Commit),
            TwoPhaseCommitState::Voting { vote_timeout_start } => {
                Ok(CoordinatorState::Voting { vote_timeout_start })
            }
            TwoPhaseCommitState::WaitingForStart => Ok(CoordinatorState::WaitingForStart),
            TwoPhaseCommitState::WaitingForVote => Ok(CoordinatorState::WaitingForVote),
            TwoPhaseCommitState::WaitingForDecisionAck { ack_timeout_start } => {
                Ok(CoordinatorState::WaitingForDecisionAck { ack_timeout_start })
            }
            TwoPhaseCommitState::WaitingForVoteRequest | TwoPhaseCommitState::Voted { .. } => {
                Err(InvalidStateError::with_message(format!(
                    "invalid state for coordinator: {state:?}",
                )))
            }
        }
    }
}

impl<T> TryFrom<TwoPhaseCommitState<T>> for ParticipantState<T>
where
    T: Time,
{
    type Error = InvalidStateError;

    fn try_from(state: TwoPhaseCommitState<T>) -> Result<Self, InvalidStateError> {
        match state {
            TwoPhaseCommitState::Abort => Ok(ParticipantState::Abort),
            TwoPhaseCommitState::Commit => Ok(ParticipantState::Commit),
            TwoPhaseCommitState::Voted {
                vote,
                decision_timeout_start,
            } => Ok(ParticipantState::Voted {
                vote,
                decision_timeout_start,
            }),
            TwoPhaseCommitState::WaitingForVoteRequest => {
                Ok(ParticipantState::WaitingForVoteRequest)
            }
            TwoPhaseCommitState::WaitingForVote => Ok(ParticipantState::WaitingForVote),
            TwoPhaseCommitState::WaitingForStart
            | TwoPhaseCommitState::WaitingForDecisionAck { .. }
            | TwoPhaseCommitState::Voting { .. } => Err(InvalidStateError::with_message(format!(
                "invalid state for participant: {state:?}",
            ))),
        }
    }
}

impl<T> From<CoordinatorState<T>> for TwoPhaseCommitState<T>
where
    T: Time,
{
    fn from(state: CoordinatorState<T>) -> Self {
        match state {
            CoordinatorState::Abort => TwoPhaseCommitState::Abort,
            CoordinatorState::Commit => TwoPhaseCommitState::Commit,
            CoordinatorState::Voting { vote_timeout_start } => {
                TwoPhaseCommitState::Voting { vote_timeout_start }
            }
            CoordinatorState::WaitingForStart => TwoPhaseCommitState::WaitingForStart,
            CoordinatorState::WaitingForVote => TwoPhaseCommitState::WaitingForVote,
            CoordinatorState::WaitingForDecisionAck { ack_timeout_start } => {
                TwoPhaseCommitState::WaitingForDecisionAck { ack_timeout_start }
            }
        }
    }
}

impl<T> From<ParticipantState<T>> for TwoPhaseCommitState<T>
where
    T: Time,
{
    fn from(state: ParticipantState<T>) -> Self {
        match state {
            ParticipantState::Abort => TwoPhaseCommitState::Abort,
            ParticipantState::Commit => TwoPhaseCommitState::Commit,
            ParticipantState::Voted {
                vote,
                decision_timeout_start,
            } => TwoPhaseCommitState::Voted {
                vote,
                decision_timeout_start,
            },
            ParticipantState::WaitingForVoteRequest => TwoPhaseCommitState::WaitingForVoteRequest,
            ParticipantState::WaitingForVote => TwoPhaseCommitState::WaitingForVote,
        }
    }
}
