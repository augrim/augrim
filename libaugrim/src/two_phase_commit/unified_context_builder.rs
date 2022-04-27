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

use crate::error::InvalidStateError;
use crate::process::Process;
use crate::time::Time;

use super::Epoch;
use super::Participant;
use super::TwoPhaseCommitContext;
use super::TwoPhaseCommitRoleContext;
use super::TwoPhaseCommitState;

#[derive(Default)]
pub struct TwoPhaseCommitContextBuilder<P, T>
where
    P: Process,
    T: Time,
{
    alarm: Option<T>,
    coordinator: Option<P>,
    epoch: Option<Epoch>,
    last_commit_epoch: Option<Epoch>,
    participants: Option<Vec<Participant<P>>>,
    participant_processes: Option<Vec<P>>,
    state: Option<TwoPhaseCommitState<T>>,
    this_process: Option<P>,
}

impl<P, T> TwoPhaseCommitContextBuilder<P, T>
where
    P: Process,
    T: Time,
{
    pub fn new() -> Self {
        Self {
            alarm: None,
            coordinator: None,
            epoch: None,
            last_commit_epoch: None,
            participants: None,
            participant_processes: None,
            state: None,
            this_process: None,
        }
    }

    pub fn with_alarm(mut self, alarm: T) -> Self {
        self.alarm = Some(alarm);
        self
    }

    pub fn with_coordinator(mut self, coordinator: P) -> Self {
        self.coordinator = Some(coordinator);
        self
    }

    pub fn with_epoch(mut self, epoch: Epoch) -> Self {
        self.epoch = Some(epoch);
        self
    }

    pub fn with_last_commit_epoch(mut self, last_commit_epoch: Epoch) -> Self {
        self.last_commit_epoch = Some(last_commit_epoch);
        self
    }

    pub fn with_participants(mut self, participants: Vec<Participant<P>>) -> Self {
        self.participants = Some(participants);
        self
    }

    pub fn with_participant_processes(mut self, participant_processes: Vec<P>) -> Self {
        self.participant_processes = Some(participant_processes);
        self
    }

    pub fn with_state(mut self, state: TwoPhaseCommitState<T>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn with_this_process(mut self, this_process: P) -> Self {
        self.this_process = Some(this_process);
        self
    }

    pub fn build(
        self,
    ) -> Result<TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>, InvalidStateError>
    {
        let alarm = self.alarm;
        let last_commit_epoch = self.last_commit_epoch;

        let coordinator = self
            .coordinator
            .ok_or_else(|| InvalidStateError::with_message("missing coordinator field".into()))?;

        let epoch = self
            .epoch
            .ok_or_else(|| InvalidStateError::with_message("missing epoch field".into()))?;

        let state = self
            .state
            .ok_or_else(|| InvalidStateError::with_message("missing state field".into()))?;

        let this_process = self
            .this_process
            .ok_or_else(|| InvalidStateError::with_message("missing this_process field".into()))?;

        let role_context = match (self.participants, self.participant_processes) {
            (Some(participants), None) => Ok(TwoPhaseCommitRoleContext::new_coordinator(
                participants,
                state,
            )?),
            (None, Some(participant_processes)) => Ok(TwoPhaseCommitRoleContext::new_participant(
                participant_processes,
                state,
            )?),
            (Some(_), Some(_)) => Err(InvalidStateError::with_message(
                "participant and participant_processes fields are mutually exclusive".into(),
            )),
            (None, None) => Err(InvalidStateError::with_message(
                "exactly one of participant or particpant_processes fields required".into(),
            )),
        }?;

        Ok(TwoPhaseCommitContext {
            alarm,
            coordinator,
            epoch,
            last_commit_epoch,
            role_context,
            this_process,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use crate::two_phase_commit::{CoordinatorContext, CoordinatorState};

    use super::*;

    #[test]
    fn build_coordinator_context() {
        let now = SystemTime::now();

        let unified_context = TwoPhaseCommitContextBuilder::<String, SystemTime>::new()
            .with_alarm(now)
            .with_coordinator("me".into())
            .with_epoch(2)
            .with_last_commit_epoch(1)
            .with_state(TwoPhaseCommitState::WaitingForStart)
            .with_this_process("me".into())
            .with_participants(vec![
                Participant::new("me".into()),
                Participant::new("p1".into()),
                Participant::new("p2".into()),
            ])
            .build()
            .unwrap();

        let coordinator_context: TwoPhaseCommitContext<_, _, CoordinatorContext<_, _>> =
            unified_context.try_into().unwrap();

        assert_eq!(coordinator_context.alarm().unwrap(), now);
        assert_eq!(*coordinator_context.coordinator(), "me".to_string());
        assert_eq!(*coordinator_context.epoch(), 2);
        assert_eq!(coordinator_context.last_commit_epoch().unwrap(), 1);
        assert_eq!(
            *coordinator_context.state(),
            CoordinatorState::WaitingForStart
        );
        assert_eq!(coordinator_context.participants().len(), 3);

        let reunified_context: TwoPhaseCommitContext<_, _> = coordinator_context.into();

        assert_eq!(reunified_context.alarm().unwrap(), now);
        assert_eq!(*reunified_context.coordinator(), "me".to_string());
        assert_eq!(*reunified_context.epoch(), 2);
        assert_eq!(reunified_context.last_commit_epoch().unwrap(), 1);
        assert_eq!(
            reunified_context.state(),
            TwoPhaseCommitState::WaitingForStart
        );
        assert_eq!(reunified_context.participants().unwrap().len(), 3);
    }
}
