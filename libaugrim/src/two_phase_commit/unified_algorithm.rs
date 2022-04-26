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

use crate::algorithm::{Algorithm, Value};
use crate::error::AlgorithmError;
use crate::process::Process;
use crate::time::TimeSource;

use super::CoordinatorAlgorithm;
use super::ParticipantAlgorithm;
use super::TwoPhaseCommitAction;
use super::TwoPhaseCommitContext;
use super::TwoPhaseCommitEvent;

pub struct TwoPhaseCommitAlgorithm<P, V, TS>
where
    P: Process,
    V: Value,
    TS: TimeSource,
{
    coordinator: CoordinatorAlgorithm<P, V, TS>,
    participant: ParticipantAlgorithm<P, V, TS>,
}

impl<P, V, TS> TwoPhaseCommitAlgorithm<P, V, TS>
where
    P: Process,
    V: Value,
    TS: TimeSource + Clone,
{
    pub fn new(time_source: TS) -> Self {
        Self {
            coordinator: CoordinatorAlgorithm::new(time_source.clone()),
            participant: ParticipantAlgorithm::new(time_source),
        }
    }
}

impl<P, V, TS> Algorithm for TwoPhaseCommitAlgorithm<P, V, TS>
where
    P: Process,
    V: Value,
    TS: TimeSource,
{
    type Event = TwoPhaseCommitEvent<P, V>;
    type Action = TwoPhaseCommitAction<P, V, TS::Time>;
    type Context = TwoPhaseCommitContext<P, TS::Time>;

    fn event(
        &self,
        event: Self::Event,
        context: Self::Context,
    ) -> Result<Vec<Self::Action>, AlgorithmError> {
        if context.coordinator() == context.this_process() {
            self.coordinator
                .event(event.try_into()?, context.try_into()?)
                .map(|v| v.into_iter().map(|a| a.into()).collect())
        } else {
            self.participant
                .event(event.try_into()?, context.try_into()?)
                .map(|v| v.into_iter().map(|a| a.into()).collect())
        }
    }
}
