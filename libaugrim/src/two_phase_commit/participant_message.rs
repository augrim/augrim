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

//! Contains ParticipantMessage, a message which is delivered to the participant.

use std::convert::TryFrom;

use crate::algorithm::Value;
use crate::error::InvalidStateError;

use super::Epoch;
use super::TwoPhaseCommitMessage;

/// A message which is delivered to the participant.
///
/// This is a subset of `TwoPhaseCommitMessage`, containing only the set of messages which can be
/// delivered to a participant.
#[derive(Clone)]
pub enum ParticipantMessage<V>
where
    V: Value,
{
    VoteRequest(Epoch, V),
    Commit(Epoch),
    Abort(Epoch),
    DecisionRequest(Epoch),
}

impl<V> From<ParticipantMessage<V>> for TwoPhaseCommitMessage<V>
where
    V: Value,
{
    fn from(message: ParticipantMessage<V>) -> Self {
        match message {
            ParticipantMessage::VoteRequest(epoch, value) => {
                TwoPhaseCommitMessage::VoteRequest(epoch, value)
            }
            ParticipantMessage::Commit(epoch) => TwoPhaseCommitMessage::Commit(epoch),
            ParticipantMessage::Abort(epoch) => TwoPhaseCommitMessage::Abort(epoch),
            ParticipantMessage::DecisionRequest(epoch) => {
                TwoPhaseCommitMessage::DecisionRequest(epoch)
            }
        }
    }
}

impl<V> TryFrom<TwoPhaseCommitMessage<V>> for ParticipantMessage<V>
where
    V: Value,
{
    type Error = InvalidStateError;

    fn try_from(message: TwoPhaseCommitMessage<V>) -> Result<Self, Self::Error> {
        match message {
            TwoPhaseCommitMessage::VoteRequest(epoch, value) => {
                Ok(ParticipantMessage::VoteRequest(epoch, value))
            }
            TwoPhaseCommitMessage::Commit(epoch) => Ok(ParticipantMessage::Commit(epoch)),
            TwoPhaseCommitMessage::Abort(epoch) => Ok(ParticipantMessage::Abort(epoch)),
            TwoPhaseCommitMessage::DecisionRequest(epoch) => {
                Ok(ParticipantMessage::DecisionRequest(epoch))
            }
            TwoPhaseCommitMessage::VoteResponse(_, _) => Err(InvalidStateError::with_message(
                "VoteResponse message cannot be handled by a participant".into(),
            )),
        }
    }
}
