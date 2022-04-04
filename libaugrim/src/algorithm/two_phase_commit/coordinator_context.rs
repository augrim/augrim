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
pub struct Participant<P> {
    pub process: P,
    pub vote: Option<bool>,
}

impl<P> Participant<P> {
    fn new(process: P) -> Participant<P> {
        Participant {
            process,
            vote: None,
        }
    }
}

#[derive(Clone)]
pub enum CoordinatorState<T>
where
    T: Time,
{
    WaitingForStart,
    Voting { vote_timeout_start: T },
    WaitingForVote,
    Abort,
    Commit,
}

#[derive(Clone)]
pub struct CoordinatorContext<P, T>
where
    P: Process,
    T: Time,
{
    alarm: Option<T>,
    coordinator: P,
    participants: Vec<Participant<P>>,
    state: CoordinatorState<T>,
    epoch: Epoch,
    last_commit_epoch: Option<Epoch>,
}

impl<P, T> CoordinatorContext<P, T>
where
    P: Process,
    T: Time,
{
    pub fn new(coordinator: P, participant_processes: Vec<P>) -> Self {
        CoordinatorContext {
            alarm: None,
            coordinator,
            participants: participant_processes
                .into_iter()
                .map(Participant::new)
                .collect(),
            state: CoordinatorState::WaitingForStart,
            epoch: 0,
            last_commit_epoch: None,
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

    pub fn participants(&self) -> &Vec<Participant<P>> {
        &self.participants
    }

    pub fn participants_mut(&mut self) -> &mut Vec<Participant<P>> {
        &mut self.participants
    }

    pub fn state(&self) -> &CoordinatorState<T> {
        &self.state
    }

    pub fn set_state(&mut self, state: CoordinatorState<T>) {
        self.state = state;
    }
}
