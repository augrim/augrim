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

use super::CoordinatorContext;
use super::TwoPhaseCommitContext;
use super::TwoPhaseCommitMessage;

pub enum CoordinatorAction<P, V, T>
where
    P: Process,
    V: Value,
    T: Time,
{
    Update {
        context: TwoPhaseCommitContext<P, T, CoordinatorContext<P, T>>,
        alarm: Option<T>,
    },
    SendMessage(P, TwoPhaseCommitMessage<V>),
    Notify(CoordinatorActionNotification),
}

impl<P, V, T> Action for CoordinatorAction<P, V, T>
where
    P: Process,
    V: Value,
    T: Time,
{
}

pub enum CoordinatorActionNotification {
    RequestForStart(),
    RequestForVote(),
    Commit(),
    Abort(),
    MessageDropped(String),
}
