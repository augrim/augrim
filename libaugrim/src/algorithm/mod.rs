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

//! Consensus algorithm trait and implementations.

mod value_impls;

use std::convert::TryFrom;
use std::marker::PhantomData;

use crate::error::{AlgorithmError, InternalError};

/// A value to be agreed upon between processes.
pub trait Value: Clone {}

/// A consensus algorithm.
///
/// An algorithm processes events with a given context, producing a set of actions.
///
/// Events are inputs into the algorithm. For example, if a message is received from another
/// process, it is input into the algorithm as an event (likely a message delivery event).
///
/// Actions are the outputs of the algorithm. For example, the algorithm may output an action to
/// update the context and another action to send a message to another process.
///
/// The context of the algorithm contains the state of the algorithm which must be remembered
/// between events. For example, if an algorithm must keep track of how other processes have voted,
/// it will be stored in the context. A context is passed in with an event and updated using an
/// action.
pub trait Algorithm {
    /// The event type representing all valid events for the algorithm.
    type Event;

    /// The action type representing all valid actions returned by the algorithm.
    type Action;

    /// The context type representing all algorithm-specific state which must be stored.
    type Context;

    /// Process an event with a given context, producing a list of actions.
    fn event(
        &self,
        event: Self::Event,
        context: Self::Context,
    ) -> Result<Vec<Self::Action>, AlgorithmError>;

    /// Maps the inputs and output of algorithm into alternate types.
    ///
    /// In order to easily facilitate the run-time selection of `Algorithm` this function provides
    /// a way to transform an algorithm such that it may accept inputs and produce outputs for the
    /// general system.  This could be used to handle things like run-time switching,
    /// serialization, or the like.
    ///
    /// # Example
    ///
    /// Suppose we have a very simple counting algorithm:
    ///
    /// ```no_run
    /// use augrim::Algorithm;
    ///
    /// struct ExampleEvent(Option<u32>);
    /// struct ExampleAction(Option<u32>);
    /// struct ExampleContext(u32);
    /// # #[derive(Debug, Eq, PartialEq, Clone)]
    ///
    /// struct ExampleAlgorithm;
    ///
    /// impl Algorithm for ExampleAlgorithm {
    ///     type Event = ExampleEvent;
    ///     type Action = ExampleAction;
    ///     type Context = ExampleContext;
    ///
    ///     fn event(
    ///         &self,
    ///         event: Self::Event,
    ///         context: Self::Context,
    ///     ) -> Result<Vec<Self::Action>, augrim::error::AlgorithmError> {
    ///         if let ExampleEvent(Some(i)) = event {
    ///             Ok(vec![ExampleAction(Some(i + context.0))])
    ///         } else {
    ///             Ok(vec![ExampleAction(None)])
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// We can imagine a scenario where another component operates on the values in their
    /// serialized string formats.  Adding the appropriate [`TryFrom`] implementations for the
    /// event, context, and action types allows the use of `into_algorithm`.  The results of which
    /// would be an algorithm with the following types:
    ///
    /// ```ignore
    /// impl Algorithm<Event=Option<&'_ str>, Context=&'_ str, Action=Option<String>>
    /// ```
    ///
    /// We can see it used as follows:
    ///
    /// ```
    /// use augrim::Algorithm;
    ///
    /// # include!("./doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let algorithm = ExampleAlgorithm.into_algorithm();
    ///
    /// let actions: Vec<Option<String>> = algorithm.event(Some("1"), "1")?;
    /// assert_eq!(actions, vec![Some("2".to_string())]);
    ///
    /// let actions = algorithm.event(None, "2")?;
    /// assert_eq!(actions, vec![None]);
    ///
    /// # Ok(())
    /// # }
    ///
    /// ```
    fn into_algorithm<E, A, C>(self) -> IntoAlgorithm<Self, E, A, C>
    where
        Self: Sized,
        Self::Event: TryFrom<E, Error = InternalError>,
        A: TryFrom<Self::Action, Error = InternalError>,
        Self::Context: TryFrom<C, Error = InternalError>,
    {
        IntoAlgorithm {
            inner: self,
            _event: PhantomData,
            _action: PhantomData,
            _context: PhantomData,
        }
    }
}

/// An algorithm that wraps an algorithm of another type.
///
/// This `struct` is returned by the [`Algorithm::into_algorithm`] method.
pub struct IntoAlgorithm<T, E, A, C> {
    inner: T,
    _event: PhantomData<E>,
    _action: PhantomData<A>,
    _context: PhantomData<C>,
}

impl<T, E, A, C> Algorithm for IntoAlgorithm<T, E, A, C>
where
    T: Algorithm,
    <T as Algorithm>::Event: TryFrom<E, Error = InternalError>,
    A: TryFrom<<T as Algorithm>::Action, Error = InternalError>,
    <T as Algorithm>::Context: TryFrom<C, Error = InternalError>,
{
    type Event = E;
    type Action = A;
    type Context = C;

    fn event(
        &self,
        event: Self::Event,
        context: Self::Context,
    ) -> Result<Vec<Self::Action>, AlgorithmError> {
        let inner_event = event.try_into()?;
        let inner_context = context.try_into()?;

        let inner_actions = self.inner.event(inner_event, inner_context)?;

        inner_actions
            .into_iter()
            .map(|action| {
                let res: Result<A, InternalError> = action.try_into();
                res
            })
            .collect::<Result<Vec<Self::Action>, InternalError>>()
            .map_err(AlgorithmError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that an algorithm with the appropriate TryFrom impls succeeds using the
    /// `into_algorithm` method.
    #[test]
    fn test_into_algorithm() -> Result<(), Box<dyn std::error::Error>> {
        let algorithm = TestAlgorithm.into_algorithm();

        let actions: Vec<Option<String>> = algorithm.event(Some("1"), "1")?;
        assert_eq!(actions, vec![Some("2".to_string())]);

        let actions = algorithm.event(None, "2")?;
        assert_eq!(actions, vec![None]);

        Ok(())
    }

    /// Test that the event method fails if:
    /// 1. the `TryFrom` impl for Event fails
    /// 2. the `TryFrom` impl for Context fails
    #[test]
    fn test_into_algorithm_err() -> Result<(), Box<dyn std::error::Error>> {
        let algorithm = TestAlgorithm.into_algorithm::<_, Option<String>, _>();

        assert!(algorithm.event(Some("foo"), "1").is_err());

        assert!(algorithm.event(None, "foo").is_err());

        Ok(())
    }

    struct TestEvent(Option<u32>);
    struct TestAction(Option<u32>);
    struct TestContext(u32);

    struct TestAlgorithm;

    impl Algorithm for TestAlgorithm {
        type Event = TestEvent;
        type Action = TestAction;
        type Context = TestContext;

        fn event(
            &self,
            event: Self::Event,
            context: Self::Context,
        ) -> Result<Vec<Self::Action>, AlgorithmError> {
            if let TestEvent(Some(i)) = event {
                Ok(vec![TestAction(Some(i + context.0))])
            } else {
                Ok(vec![TestAction(None)])
            }
        }
    }

    impl<'a> TryFrom<Option<&'a str>> for TestEvent {
        type Error = InternalError;

        fn try_from(val: Option<&'a str>) -> Result<Self, Self::Error> {
            val.map(|s| {
                s.parse::<u32>()
                    .map_err(|e| InternalError::from_source(Box::new(e)))
            })
            .transpose()
            .map(TestEvent)
        }
    }

    impl TryFrom<TestAction> for Option<String> {
        type Error = InternalError;

        fn try_from(val: TestAction) -> Result<Self, Self::Error> {
            Ok(val.0.map(|i| i.to_string()))
        }
    }

    impl<'a> TryFrom<&'a str> for TestContext {
        type Error = InternalError;

        fn try_from(val: &'a str) -> Result<Self, Self::Error> {
            val.parse::<u32>()
                .map_err(|e| InternalError::from_source(Box::new(e)))
                .map(TestContext)
        }
    }
}
