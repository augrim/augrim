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

use crate::algorithm::{Action, Value};
use crate::process::Process;
use crate::time::Time;

use super::TwoPhaseCommitContext;
use super::TwoPhaseCommitMessage;

#[derive(Debug, PartialEq)]
pub enum TwoPhaseCommitAction<P, V, T>
where
    P: Process,
    V: Value,
    T: Time,
{
    Update {
        context: TwoPhaseCommitContext<P, T>,
        alarm: Option<T>,
    },
    SendMessage(P, TwoPhaseCommitMessage<V>),
    Notify(TwoPhaseCommitActionNotification<V>),
}

impl<P, V, T> Action for TwoPhaseCommitAction<P, V, T>
where
    P: Process,
    V: Value,
    T: Time,
{
}

#[derive(Debug, PartialEq)]
pub enum TwoPhaseCommitActionNotification<V>
where
    V: Value,
{
    Abort(),
    Commit(),
    MessageDropped(String),
    RequestForStart(),
    CoordinatorRequestForVote(),
    ParticipantRequestForVote(V),
}
