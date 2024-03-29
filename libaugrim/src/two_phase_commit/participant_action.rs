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

use super::ParticipantContext;
use super::TwoPhaseCommitContext;
use super::TwoPhaseCommitMessage;
use super::{TwoPhaseCommitAction, TwoPhaseCommitActionNotification};

pub enum ParticipantAction<P, V, T>
where
    P: Process,
    V: Value,
    T: Time,
{
    Notify(ParticipantActionNotification<V>),
    SendMessage(P, TwoPhaseCommitMessage<V>),
    Update {
        context: TwoPhaseCommitContext<P, T, ParticipantContext<P, T>>,
        alarm: Option<T>,
    },
}

pub enum ParticipantActionNotification<V> {
    Abort(),
    Commit(),
    MessageDropped(String),
    RequestForVote(V),
}

impl<P, V, T> From<ParticipantAction<P, V, T>> for TwoPhaseCommitAction<P, V, T>
where
    P: Process,
    V: Value,
    T: Time,
{
    fn from(action: ParticipantAction<P, V, T>) -> Self {
        match action {
            ParticipantAction::Update { context, alarm } => TwoPhaseCommitAction::Update {
                context: context.into(),
                alarm,
            },
            ParticipantAction::SendMessage(p, m) => TwoPhaseCommitAction::SendMessage(p, m),
            ParticipantAction::Notify(n) => TwoPhaseCommitAction::Notify(n.into()),
        }
    }
}

impl<V> From<ParticipantActionNotification<V>> for TwoPhaseCommitActionNotification<V>
where
    V: Value,
{
    fn from(notification: ParticipantActionNotification<V>) -> Self {
        match notification {
            ParticipantActionNotification::Abort() => TwoPhaseCommitActionNotification::Abort(),
            ParticipantActionNotification::Commit() => TwoPhaseCommitActionNotification::Commit(),
            ParticipantActionNotification::MessageDropped(s) => {
                TwoPhaseCommitActionNotification::MessageDropped(s)
            }
            ParticipantActionNotification::RequestForVote(v) => {
                TwoPhaseCommitActionNotification::ParticipantRequestForVote(v)
            }
        }
    }
}
