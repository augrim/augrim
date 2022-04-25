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

use std::convert::TryFrom;
use std::marker::PhantomData;

use crate::error::{AlgorithmError, InternalError};
use crate::process::Process;

#[cfg(feature = "algorithm-two-phase-commit")]
pub mod two_phase_commit;

pub trait Action {}
pub trait Context {}
pub trait Value: Clone {}

pub trait Algorithm<P>
where
    P: Process,
{
    type Event;
    type Action;
    type Context;

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
    /// use augrim::algorithm::Algorithm;
    ///
    /// struct ExampleEvent(Option<u32>);
    /// struct ExampleAction(Option<u32>);
    /// struct ExampleContext(u32);
    /// # #[derive(Debug, Eq, PartialEq, Clone)]
    /// # struct ExampleProcess;
    /// # impl augrim::process::Process for ExampleProcess {}
    ///
    /// struct ExampleAlgorithm;
    ///
    /// impl Algorithm<ExampleProcess> for ExampleAlgorithm {
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
    /// impl Algorithm<P, Event=Option<&'_ str>, Context=&'_ str, Action=Option<String>>
    /// ```
    ///
    /// We can see it used as follows:
    ///
    /// ```
    /// use augrim::algorithm::Algorithm;
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
    fn into_algorithm<E, A, C>(self) -> IntoAlgorithm<Self, P, E, A, C>
    where
        Self: Sized,
        Self::Event: TryFrom<E, Error = InternalError>,
        A: TryFrom<Self::Action, Error = InternalError>,
        Self::Context: TryFrom<C, Error = InternalError>,
    {
        IntoAlgorithm {
            inner: self,
            _process: PhantomData,
            _event: PhantomData,
            _action: PhantomData,
            _context: PhantomData,
        }
    }
}

pub struct IntoAlgorithm<T, P, E, A, C> {
    inner: T,
    _process: PhantomData<P>,
    _event: PhantomData<E>,
    _action: PhantomData<A>,
    _context: PhantomData<C>,
}

impl<T, P, E, A, C> Algorithm<P> for IntoAlgorithm<T, P, E, A, C>
where
    P: Process,
    T: Algorithm<P>,
    <T as Algorithm<P>>::Event: TryFrom<E, Error = InternalError>,
    A: TryFrom<<T as Algorithm<P>>::Action, Error = InternalError>,
    <T as Algorithm<P>>::Context: TryFrom<C, Error = InternalError>,
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
    #[derive(Debug, Eq, PartialEq, Clone)]
    struct TestProcess;

    impl Process for TestProcess {}

    struct TestAlgorithm;

    impl Algorithm<TestProcess> for TestAlgorithm {
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
