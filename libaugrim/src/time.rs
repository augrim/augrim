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

//! Time-related traits and default implementations.

use std::cmp::PartialOrd;
use std::fmt::Debug;
use std::ops::Add;
use std::time::Duration;
use std::time::SystemTime;

/// A specific instant in time.
///
/// Algorithms which use time, common for implementing timeouts, define time using this trait. The
/// user of the algorithm can implement this Time trait on the specific type desired for their
/// application.
pub trait Time: Add<Duration, Output = Self> + PartialOrd + Copy + Debug {}

/// A factory for getting the current time.
///
/// This factory is used by algorithms which need to get the current time (for example, to
/// determine if a timeout has occurred). Any implementations of [`Time`] will likely need to also
/// implement this trait as well.
pub trait TimeSource {
    type Time: Time;

    /// Return the current time.
    fn now(&self) -> Self::Time;
}

/// [`SystemTime`] can be used directly as Time. See also [`SystemTimeFactory`].
impl Time for SystemTime {}

/// An implementation of [`TimeSource`] which works with [`SystemTime`].
#[derive(Default, Clone)]
pub struct SystemTimeFactory {}

impl SystemTimeFactory {
    pub fn new() -> Self {
        SystemTimeFactory {}
    }
}

impl TimeSource for SystemTimeFactory {
    type Time = SystemTime;

    fn now(&self) -> Self::Time {
        SystemTime::now()
    }
}
