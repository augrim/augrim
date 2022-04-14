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

use std::cmp::PartialOrd;
use std::fmt::Debug;
use std::ops::Add;
use std::time::Duration;
use std::time::SystemTime;

pub trait Time: Add<Duration, Output = Self> + PartialOrd + Copy + Debug {}

pub trait TimeSource {
    type Time: Time;

    fn now(&self) -> Self::Time;
}

impl Time for SystemTime {}

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
