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

mod coordinator_action;
mod coordinator_algorithm;
mod coordinator_context;
mod coordinator_event;
mod coordinator_message;
mod participant_action;
mod participant_algorithm;
mod participant_context;
mod participant_event;
mod participant_message;
mod unified_action;
mod unified_context;
mod unified_context_builder;
mod unified_message;
mod unified_role;
mod unified_state;

use coordinator_action::{CoordinatorAction, CoordinatorActionNotification};
pub use coordinator_algorithm::CoordinatorAlgorithm;
pub use coordinator_context::Participant;
use coordinator_context::{CoordinatorContext, CoordinatorState};
pub use coordinator_event::CoordinatorEvent;
use coordinator_message::CoordinatorMessage;
use participant_action::{ParticipantAction, ParticipantActionNotification};
pub use participant_algorithm::ParticipantAlgorithm;
use participant_context::{ParticipantContext, ParticipantState};
pub use participant_event::ParticipantEvent;
use participant_message::ParticipantMessage;
pub use unified_action::{TwoPhaseCommitAction, TwoPhaseCommitActionNotification};
pub use unified_context::TwoPhaseCommitContext;
pub use unified_context_builder::TwoPhaseCommitContextBuilder;
pub use unified_message::TwoPhaseCommitMessage;
pub use unified_role::TwoPhaseCommitRoleContext;
pub use unified_state::TwoPhaseCommitState;

pub type Epoch = u64;
