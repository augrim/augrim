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

use crate::algorithm::two_phase_commit::Epoch;
use crate::error::InvalidStateError;
use crate::process::Process;
use crate::time::Time;

use super::TwoPhaseCommitRoleContext;
use super::TwoPhaseCommitState;
use super::{CoordinatorContext, CoordinatorState, Participant};
use super::{ParticipantContext, ParticipantState};

#[derive(Clone)]
pub struct TwoPhaseCommitContext<P, T, R = TwoPhaseCommitRoleContext<P, T>>
where
    P: Process,
    T: Time,
    R: Clone,
{
    pub(super) alarm: Option<T>,
    pub(super) coordinator: P,
    pub(super) epoch: Epoch,
    pub(super) last_commit_epoch: Option<Epoch>,
    pub(super) role_context: R,
    pub(super) this_process: P,
}

impl<P, T, R> TwoPhaseCommitContext<P, T, R>
where
    P: Process,
    T: Time,
    R: Clone,
{
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

    pub fn this_process(&self) -> &P {
        &self.this_process
    }
}

impl<P, T> TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>
where
    P: Process,
    T: Time,
{
    pub fn participants(&self) -> Option<&Vec<Participant<P>>> {
        self.role_context.participants()
    }

    pub fn participant_processes(&self) -> Option<&Vec<P>> {
        self.role_context.participant_processes()
    }

    pub fn state(&self) -> TwoPhaseCommitState<T> {
        self.role_context.state()
    }
}

impl<P, T> TwoPhaseCommitContext<P, T, CoordinatorContext<P, T>>
where
    P: Process,
    T: Time,
{
    pub(super) fn participants(&self) -> &Vec<Participant<P>> {
        &self.role_context.participants
    }

    pub(super) fn participants_mut(&mut self) -> &mut Vec<Participant<P>> {
        &mut self.role_context.participants
    }

    pub(super) fn state(&self) -> &CoordinatorState<T> {
        &self.role_context.state
    }

    pub(super) fn set_state(&mut self, state: CoordinatorState<T>) {
        self.role_context.state = state;
    }
}

impl<P, T> TwoPhaseCommitContext<P, T, ParticipantContext<P, T>>
where
    P: Process,
    T: Time,
{
    pub(super) fn participant_processes(&self) -> &Vec<P> {
        &self.role_context.participant_processes
    }

    pub(super) fn state(&self) -> &ParticipantState<T> {
        &self.role_context.state
    }

    pub(super) fn set_state(&mut self, state: ParticipantState<T>) {
        self.role_context.state = state;
    }
}

#[doc(hidden)]
impl<P, T> TryFrom<TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>>
    for TwoPhaseCommitContext<P, T, CoordinatorContext<P, T>>
where
    P: Process,
    T: Time,
{
    type Error = InvalidStateError;

    fn try_from(
        context: TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            alarm: context.alarm,
            coordinator: context.coordinator,
            epoch: context.epoch,
            last_commit_epoch: context.last_commit_epoch,
            role_context: context.role_context.try_into()?,
            this_process: context.this_process,
        })
    }
}

#[doc(hidden)]
impl<P, T> TryFrom<TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>>
    for TwoPhaseCommitContext<P, T, ParticipantContext<P, T>>
where
    P: Process,
    T: Time,
{
    type Error = InvalidStateError;

    fn try_from(
        context: TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            alarm: context.alarm,
            coordinator: context.coordinator,
            epoch: context.epoch,
            last_commit_epoch: context.last_commit_epoch,
            role_context: context.role_context.try_into()?,
            this_process: context.this_process,
        })
    }
}

#[doc(hidden)]
impl<P, T> From<TwoPhaseCommitContext<P, T, CoordinatorContext<P, T>>>
    for TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>
where
    P: Process,
    T: Time,
{
    fn from(context: TwoPhaseCommitContext<P, T, CoordinatorContext<P, T>>) -> Self {
        Self {
            alarm: context.alarm,
            coordinator: context.coordinator,
            epoch: context.epoch,
            last_commit_epoch: context.last_commit_epoch,
            role_context: context.role_context.into(),
            this_process: context.this_process,
        }
    }
}

#[doc(hidden)]
impl<P, T> From<TwoPhaseCommitContext<P, T, ParticipantContext<P, T>>>
    for TwoPhaseCommitContext<P, T, TwoPhaseCommitRoleContext<P, T>>
where
    P: Process,
    T: Time,
{
    fn from(context: TwoPhaseCommitContext<P, T, ParticipantContext<P, T>>) -> Self {
        Self {
            alarm: context.alarm,
            coordinator: context.coordinator,
            epoch: context.epoch,
            last_commit_epoch: context.last_commit_epoch,
            role_context: context.role_context.into(),
            this_process: context.this_process,
        }
    }
}
