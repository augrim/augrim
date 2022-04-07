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

use super::Epoch;

#[derive(Clone)]
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
    alarm: Option<T>,
    coordinator: P,
    epoch: Epoch,
    last_commit_epoch: Option<Epoch>,
    participant_processes: Vec<P>,
    state: ParticipantState<T>,
    this_process: P,
}

impl<P, T> ParticipantContext<P, T>
where
    P: Process,
    T: Time,
{
    pub fn new(this_process: P, coordinator: P, participant_processes: Vec<P>) -> Self {
        ParticipantContext {
            alarm: None,
            coordinator,
            epoch: 0,
            last_commit_epoch: None,
            participant_processes,
            state: ParticipantState::WaitingForVoteRequest,
            this_process,
        }
    }

    pub fn alarm(&self) -> &Option<T> {
        &self.alarm
    }

    pub fn set_alarm(&mut self, alarm: Option<T>) {
        self.alarm = alarm;
    }

    pub fn coordinator(&self) -> &P {
        &self.coordinator
    }

    pub fn epoch(&self) -> &Epoch {
        &self.epoch
    }

    pub fn set_epoch(&mut self, epoch: Epoch) {
        self.epoch = epoch
    }

    pub fn last_commit_epoch(&self) -> &Option<Epoch> {
        &self.last_commit_epoch
    }

    pub fn set_last_commit_epoch(&mut self, epoch: Option<Epoch>) {
        self.last_commit_epoch = epoch
    }

    pub fn participant_processes(&self) -> &Vec<P> {
        &self.participant_processes
    }

    pub fn state(&self) -> &ParticipantState<T> {
        &self.state
    }

    pub fn set_state(&mut self, state: ParticipantState<T>) {
        self.state = state;
    }

    pub fn this_process(&self) -> &P {
        &self.this_process
    }
}
