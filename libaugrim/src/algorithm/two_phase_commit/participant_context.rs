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

#[derive(Clone, Debug, PartialEq)]
pub enum ParticipantState<T>
where
    T: Time,
{
    Abort,
    Commit,
    Voted {
        vote: bool,
        decision_timeout_start: T,
    },
    WaitingForVoteRequest,
    WaitingForVote,
}

#[derive(Clone)]
pub struct ParticipantContext<P, T>
where
    P: Process,
    T: Time,
{
    pub(super) participant_processes: Vec<P>,
    pub(super) state: ParticipantState<T>,
}
