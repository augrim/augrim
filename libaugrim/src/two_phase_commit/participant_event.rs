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

use crate::algorithm::Value;
use crate::error::InvalidStateError;
use crate::process::Process;

use super::ParticipantMessage;
use super::TwoPhaseCommitEvent;

pub enum ParticipantEvent<P, V>
where
    P: Process,
    V: Value,
{
    Alarm(),
    Deliver(P, ParticipantMessage<V>),
    Vote(bool),
}

impl<P, V> TryFrom<TwoPhaseCommitEvent<P, V>> for ParticipantEvent<P, V>
where
    P: Process,
    V: Value,
{
    type Error = InvalidStateError;

    fn try_from(event: TwoPhaseCommitEvent<P, V>) -> Result<Self, Self::Error> {
        match event {
            TwoPhaseCommitEvent::Alarm() => Ok(ParticipantEvent::Alarm()),
            TwoPhaseCommitEvent::Deliver(p, m) => Ok(ParticipantEvent::Deliver(p, m.try_into()?)),
            TwoPhaseCommitEvent::Start(_) => Err(InvalidStateError::with_message(
                "Start event can not be handled by a participant".into(),
            )),
            TwoPhaseCommitEvent::Vote(vote) => Ok(ParticipantEvent::Vote(vote)),
        }
    }
}
