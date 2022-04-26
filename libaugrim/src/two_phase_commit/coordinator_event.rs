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

use super::CoordinatorMessage;
use super::TwoPhaseCommitEvent;

pub enum CoordinatorEvent<P, V>
where
    P: Process,
    V: Value,
{
    Alarm(),
    Deliver(P, CoordinatorMessage),
    Start(V),
    Vote(bool),
}

impl<P, V> TryFrom<TwoPhaseCommitEvent<P, V>> for CoordinatorEvent<P, V>
where
    P: Process,
    V: Value,
{
    type Error = InvalidStateError;

    fn try_from(event: TwoPhaseCommitEvent<P, V>) -> Result<Self, Self::Error> {
        Ok(match event {
            TwoPhaseCommitEvent::Alarm() => CoordinatorEvent::Alarm(),
            TwoPhaseCommitEvent::Deliver(p, m) => CoordinatorEvent::Deliver(p, m.try_into()?),
            TwoPhaseCommitEvent::Start(value) => CoordinatorEvent::Start(value),
            TwoPhaseCommitEvent::Vote(vote) => CoordinatorEvent::Vote(vote),
        })
    }
}
