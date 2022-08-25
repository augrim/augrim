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

use crate::process::Process;
use crate::time::Time;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Participant<P> {
    pub process: P,
    pub vote: Option<bool>,
    pub decision_ack: bool,
}

impl<P> Participant<P> {
    pub fn new(process: P) -> Participant<P> {
        Participant {
            process,
            vote: None,
            decision_ack: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoordinatorState<T>
where
    T: Time,
{
    Abort,
    Commit,
    Voting { vote_timeout_start: T },
    WaitingForDecisionAck { ack_timeout_start: T },
    WaitingForStart,
    WaitingForVote,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoordinatorContext<P, T>
where
    P: Process,
    T: Time,
{
    pub(super) participants: Vec<Participant<P>>,
    pub(super) state: CoordinatorState<T>,
}
