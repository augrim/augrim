// Copyright 2022 Cargill Incorporated
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

//! Contains AlgorithmError

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use super::InternalError;
use super::InvalidStateError;

/// An error which can occur while an `Algorithm` is processing `Event`s.
#[derive(Debug)]
pub enum AlgorithmError {
    /// The provided `Event` can not be run with the provided context.
    InvalidState(InvalidStateError),

    /// The algorithm could not process the event due to an unexpected internal error.
    Internal(InternalError),
}

impl Error for AlgorithmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AlgorithmError::InvalidState(e) => Some(e),
            AlgorithmError::Internal(e) => Some(e),
        }
    }
}

impl Display for AlgorithmError {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        match self {
            AlgorithmError::InvalidState(e) => write!(f, "{}", e),
            AlgorithmError::Internal(e) => write!(f, "{}", e),
        }
    }
}

impl From<InvalidStateError> for AlgorithmError {
    fn from(err: InvalidStateError) -> Self {
        AlgorithmError::InvalidState(err)
    }
}

impl From<InternalError> for AlgorithmError {
    fn from(err: InternalError) -> Self {
        AlgorithmError::Internal(err)
    }
}
