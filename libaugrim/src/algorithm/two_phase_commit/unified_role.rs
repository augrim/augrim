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

use super::ParticipantContext;
use super::TwoPhaseCommitState;
use super::{CoordinatorContext, Participant};

#[derive(Clone)]
enum InnerContext<P, T>
where
    P: Process,
    T: Time,
{
    Coordinator(CoordinatorContext<P, T>),
    Participant(ParticipantContext<P, T>),
}

#[derive(Clone)]
pub struct TwoPhaseCommitRoleContext<P, T>
where
    P: Process,
    T: Time,
{
    inner: InnerContext<P, T>,
}

impl<P, T> TwoPhaseCommitRoleContext<P, T>
where
    P: Process,
    T: Time,
{
    pub(super) fn new_coordinator(
        participants: Vec<Participant<P>>,
        state: TwoPhaseCommitState<T>,
    ) -> Result<Self, InvalidStateError> {
        Ok(Self {
            inner: InnerContext::Coordinator(CoordinatorContext {
                participants,
                state: state.try_into()?,
            }),
        })
    }

    pub(super) fn new_participant(
        participant_processes: Vec<P>,
        state: TwoPhaseCommitState<T>,
    ) -> Result<Self, InvalidStateError> {
        Ok(Self {
            inner: InnerContext::Participant(ParticipantContext {
                participant_processes,
                state: state.try_into()?,
            }),
        })
    }

    pub(super) fn participants(&self) -> Option<&Vec<Participant<P>>> {
        match &self.inner {
            InnerContext::Coordinator(c) => Some(&c.participants),
            InnerContext::Participant(_) => None,
        }
    }

    pub(super) fn participant_processes(&self) -> Option<&Vec<P>> {
        match &self.inner {
            InnerContext::Coordinator(_) => None,
            InnerContext::Participant(c) => Some(&c.participant_processes),
        }
    }

    pub fn state(&self) -> TwoPhaseCommitState<T> {
        match &self.inner {
            InnerContext::Coordinator(c) => c.state.clone().into(),
            InnerContext::Participant(c) => c.state.clone().into(),
        }
    }
}

impl<P, T> TryFrom<TwoPhaseCommitRoleContext<P, T>> for CoordinatorContext<P, T>
where
    P: Process,
    T: Time,
{
    type Error = InvalidStateError;

    fn try_from(context: TwoPhaseCommitRoleContext<P, T>) -> Result<Self, Self::Error> {
        match context.inner {
            InnerContext::Coordinator(c) => Ok(c),
            InnerContext::Participant(_) => Err(InvalidStateError::with_message(
                "unable to convert TwoPhaseCommitRoleContext to CoordinatorContext \
                because inner context type is Participant"
                    .into(),
            )),
        }
    }
}

impl<P, T> TryFrom<TwoPhaseCommitRoleContext<P, T>> for ParticipantContext<P, T>
where
    P: Process,
    T: Time,
{
    type Error = InvalidStateError;

    fn try_from(context: TwoPhaseCommitRoleContext<P, T>) -> Result<Self, Self::Error> {
        match context.inner {
            InnerContext::Participant(c) => Ok(c),
            InnerContext::Coordinator(_) => Err(InvalidStateError::with_message(
                "unable to convert TwoPhaseCommitRoleContext to ParticipantContext \
                because inner context type is Coordinator"
                    .into(),
            )),
        }
    }
}

impl<P, T> From<CoordinatorContext<P, T>> for TwoPhaseCommitRoleContext<P, T>
where
    P: Process,
    T: Time,
{
    fn from(context: CoordinatorContext<P, T>) -> Self {
        Self {
            inner: InnerContext::Coordinator(context),
        }
    }
}

impl<P, T> From<ParticipantContext<P, T>> for TwoPhaseCommitRoleContext<P, T>
where
    P: Process,
    T: Time,
{
    fn from(context: ParticipantContext<P, T>) -> Self {
        Self {
            inner: InnerContext::Participant(context),
        }
    }
}
