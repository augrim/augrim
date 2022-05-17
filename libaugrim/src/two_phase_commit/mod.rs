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

//! An implementation of the 2PC (Two-Phase Commit) atomic commitment protocol.
//!
//! The algorithm attempts to faithfully implement 2PC as it is described in the following source:
//!
//! - Bernstein, Hadzilacos, and Goodman, Concurrency Control and Recovery in Database Systems,
//!   7.4.  This book may be downloaded for free from
//!   <https://www.microsoft.com/en-us/research/people/philbe/>.

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
mod unified_algorithm;
mod unified_context;
mod unified_context_builder;
mod unified_event;
mod unified_message;
mod unified_role;
mod unified_state;

use coordinator_action::{CoordinatorAction, CoordinatorActionNotification};
use coordinator_algorithm::CoordinatorAlgorithm;
pub use coordinator_context::Participant;
use coordinator_context::{CoordinatorContext, CoordinatorState};
use coordinator_event::CoordinatorEvent;
use coordinator_message::CoordinatorMessage;
use participant_action::{ParticipantAction, ParticipantActionNotification};
use participant_algorithm::ParticipantAlgorithm;
use participant_context::{ParticipantContext, ParticipantState};
use participant_event::ParticipantEvent;
use participant_message::ParticipantMessage;
pub use unified_action::{TwoPhaseCommitAction, TwoPhaseCommitActionNotification};
pub use unified_algorithm::TwoPhaseCommitAlgorithm;
pub use unified_context::TwoPhaseCommitContext;
pub use unified_context_builder::TwoPhaseCommitContextBuilder;
pub use unified_event::TwoPhaseCommitEvent;
pub use unified_message::TwoPhaseCommitMessage;
pub use unified_role::TwoPhaseCommitRoleContext;
pub use unified_state::TwoPhaseCommitState;

/// The scope of a single run through the algorithm.
///
/// An epoch starts with the coordinator requesting votes for a specific value and ends with
/// a decision to commit or abort.
///
/// This extension to the original algorithm allows running the algorithm continuously, to agree on
/// a sequence of values instead of a single value. In each iteration, the epoch increases by 1.
pub type Epoch = u64;
