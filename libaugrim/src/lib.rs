// Copyright 2021 Cargill Incorporated
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

#[macro_use]
extern crate log;

#[cfg(feature = "algorithm")]
mod algorithm;
pub mod error;
#[cfg(feature = "links")]
pub mod links;
mod message;
mod process;
#[cfg(feature = "time")]
mod time;
#[cfg(feature = "algorithm-two-phase-commit")]
pub mod two_phase_commit;

#[cfg(feature = "algorithm")]
pub use algorithm::{Algorithm, IntoAlgorithm, Value};
pub use message::Message;
pub use process::Process;
#[cfg(feature = "time")]
pub use time::{SystemTimeFactory, Time, TimeSource};
