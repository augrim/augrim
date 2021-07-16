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

//! Defines a set of communication abstractions.
//!
//! The communication abstraction is defined as a link[^note-1] between two processes.  Messages are
//! exchanged between processes over a link. These messages are unique, and each process is
//! uniquely identified.
//!
//! Given the nature of ownership in Rust, a link is provided by both a [`Sender`] and a
//! [`Receiver`][^note-2].
//!
//! [^note-1]: For a full explanation of links, see Cachin, Guerraoui, and Rodrigues, **Reliable and
//! Secure Distributed Programming**, 2nd ed., 2.4.
//!
//! [^note-2]: Note that in **Reliable and Secure Distributed Programming**, the terminology is
//! "deliver". See 2.4.1.

use crate::error::InternalError;
use crate::message::Message;
use crate::process::Process;

/// The sending half of a link
pub trait Sender<P, M>
where
    P: Process,
    M: Message,
{
    /// Send a message to a given process.
    ///
    /// # Errors
    ///
    /// An [`InternalError`] is returned if the underlying implementation encounters an error
    /// during send.
    fn send(to_process: &P, message: M) -> Result<(), InternalError>;
}

/// The receiving half of a link.
///
/// Concrete Receivers receive a message from a process, and may act on that message as they
/// see fit. In other words, a message is delivered to the receiver.
pub trait Receiver<P, M>
where
    P: Process,
    M: Message,
{
    /// Receive a message from a process.
    ///
    /// # Errors
    ///
    /// An [`InternalError`] is returned if the underlying implementation encounters an error
    /// while processing the received message.
    fn recv(from_process: &P, message: M) -> Result<(), InternalError>;
}
