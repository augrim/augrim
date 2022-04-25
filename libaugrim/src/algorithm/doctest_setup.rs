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


struct ExampleEvent(Option<u32>);
struct ExampleAction(Option<u32>);
struct ExampleContext(u32);
#[derive(Debug, Eq, PartialEq, Clone)]
struct ExampleProcess;

impl augrim::process::Process for ExampleProcess {}

struct ExampleAlgorithm;

impl augrim::algorithm::Algorithm<ExampleProcess> for ExampleAlgorithm {
    type Event = ExampleEvent;
    type Action = ExampleAction;
    type Context = ExampleContext;

    fn event(
        &self,
        event: Self::Event,
        context: Self::Context,
    ) -> Result<Vec<Self::Action>, augrim::error::AlgorithmError> {
        if let ExampleEvent(Some(i)) = event {
            Ok(vec![ExampleAction(Some(i + context.0))])
        } else {
            Ok(vec![ExampleAction(None)])
        }
    }
}

impl<'a> TryFrom<Option<&'a str>> for ExampleEvent {
    type Error = augrim::error::InternalError;

    fn try_from(val: Option<&'a str>) -> Result<Self, Self::Error> {
        val.map(|s| {
            s.parse::<u32>()
                .map_err(|e| augrim::error::InternalError::from_source(Box::new(e)))
        })
        .transpose()
            .map(ExampleEvent)
    }
}

impl TryFrom<ExampleAction> for Option<String> {
    type Error = augrim::error::InternalError;

    fn try_from(val: ExampleAction) -> Result<Self, Self::Error> {
        Ok(val.0.map(|i| i.to_string()))
    }
}

impl<'a> TryFrom<&'a str> for ExampleContext {
    type Error = augrim::error::InternalError;

    fn try_from(val: &'a str) -> Result<Self, Self::Error> {
        val.parse::<u32>()
            .map_err(|e| augrim::error::InternalError::from_source(Box::new(e)))
            .map(ExampleContext)
    }
}
