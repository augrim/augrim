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

//! Contains CoordinatorMessage, a message which is delivered to the coordinator.

use std::convert::TryFrom;

use crate::algorithm::Value;
use crate::error::InvalidStateError;

use super::Epoch;
use super::TwoPhaseCommitMessage;

/// A message which is delivered to the coordinator.
///
/// This is a subset of `TwoPhaseCommitMessage`, containing only the set of messages which can be
/// delivered to a coordinator.
#[derive(Clone)]
pub enum CoordinatorMessage {
    VoteResponse(Epoch, bool),
    DecisionRequest(Epoch),
    DecisionAck(Epoch),
}

impl<V> From<CoordinatorMessage> for TwoPhaseCommitMessage<V>
where
    V: Value,
{
    fn from(message: CoordinatorMessage) -> Self {
        match message {
            CoordinatorMessage::VoteResponse(epoch, vote) => {
                TwoPhaseCommitMessage::VoteResponse(epoch, vote)
            }
            CoordinatorMessage::DecisionRequest(epoch) => {
                TwoPhaseCommitMessage::DecisionRequest(epoch)
            }
            CoordinatorMessage::DecisionAck(epoch) => TwoPhaseCommitMessage::DecisionAck(epoch),
        }
    }
}

impl<V> TryFrom<TwoPhaseCommitMessage<V>> for CoordinatorMessage
where
    V: Value,
{
    type Error = InvalidStateError;

    fn try_from(message: TwoPhaseCommitMessage<V>) -> Result<Self, Self::Error> {
        match message {
            TwoPhaseCommitMessage::VoteResponse(epoch, vote) => {
                Ok(CoordinatorMessage::VoteResponse(epoch, vote))
            }
            TwoPhaseCommitMessage::DecisionRequest(epoch) => {
                Ok(CoordinatorMessage::DecisionRequest(epoch))
            }
            TwoPhaseCommitMessage::DecisionAck(epoch) => Ok(CoordinatorMessage::DecisionAck(epoch)),
            TwoPhaseCommitMessage::VoteRequest(_, _) => Err(InvalidStateError::with_message(
                "VoteRequest message cannot be handled by a coordinator".into(),
            )),
            TwoPhaseCommitMessage::Commit(_) => Err(InvalidStateError::with_message(
                "Commit message cannot be handled by a coordinator".into(),
            )),
            TwoPhaseCommitMessage::Abort(_) => Err(InvalidStateError::with_message(
                "Abort message cannot be handled by a coordinator".into(),
            )),
        }
    }
}
