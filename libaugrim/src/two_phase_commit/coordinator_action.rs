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
use crate::process::Process;
use crate::time::Time;

use super::CoordinatorContext;
use super::TwoPhaseCommitContext;
use super::TwoPhaseCommitMessage;
use super::{TwoPhaseCommitAction, TwoPhaseCommitActionNotification};

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

pub enum CoordinatorActionNotification {
    RequestForStart(),
    RequestForVote(),
    Commit(),
    Abort(),
    MessageDropped(String),
}

impl<P, V, T> From<CoordinatorAction<P, V, T>> for TwoPhaseCommitAction<P, V, T>
where
    P: Process,
    V: Value,
    T: Time,
{
    fn from(action: CoordinatorAction<P, V, T>) -> Self {
        match action {
            CoordinatorAction::Update { context, alarm } => TwoPhaseCommitAction::Update {
                context: context.into(),
                alarm,
            },
            CoordinatorAction::SendMessage(p, m) => TwoPhaseCommitAction::SendMessage(p, m),
            CoordinatorAction::Notify(n) => TwoPhaseCommitAction::Notify(n.into()),
        }
    }
}

impl<V> From<CoordinatorActionNotification> for TwoPhaseCommitActionNotification<V>
where
    V: Value,
{
    fn from(notification: CoordinatorActionNotification) -> Self {
        match notification {
            CoordinatorActionNotification::Abort() => TwoPhaseCommitActionNotification::Abort(),
            CoordinatorActionNotification::Commit() => TwoPhaseCommitActionNotification::Commit(),
            CoordinatorActionNotification::MessageDropped(s) => {
                TwoPhaseCommitActionNotification::MessageDropped(s)
            }
            CoordinatorActionNotification::RequestForStart() => {
                TwoPhaseCommitActionNotification::RequestForStart()
            }
            CoordinatorActionNotification::RequestForVote() => {
                TwoPhaseCommitActionNotification::CoordinatorRequestForVote()
            }
        }
    }
}
