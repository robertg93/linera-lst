// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::Arc;

use async_graphql::{EmptySubscription, Object, Request, Response, Schema};
use linera_sdk::{graphql::GraphQLMutationRoot, linera_base_types::WithServiceAbi, views::View, Service, ServiceRuntime};
use lst::{Operation, Parameters};

use crate::state::LstState;

// ANCHOR: service_struct
linera_sdk::service!(LstService);

pub struct LstService {
    state: Arc<LstState>,
    runtime: Arc<ServiceRuntime<Self>>,
}
// ANCHOR_END: service_struct

// ANCHOR: declare_abi
impl WithServiceAbi for LstService {
    type Abi = lst::LstAbi;
}
// ANCHOR_END: declare_abi

impl Service for LstService {
    type Parameters = Parameters;

    // ANCHOR: new
    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = LstState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
        LstService {
            state: Arc::new(state),
            runtime: Arc::new(runtime),
        }
    }
    // ANCHOR_END: new

    // ANCHOR: handle_query
    async fn handle_query(&self, request: Request) -> Response {
        let schema = Schema::build(self.state.clone(), Operation::mutation_root(self.runtime.clone()), EmptySubscription).finish();
        schema.execute(request).await
    }
    // ANCHOR_END: handle_query
}

// ANCHOR: mutation
struct MutationRoot {
    runtime: Arc<ServiceRuntime<LstService>>,
}

#[Object]
impl MutationRoot {
    async fn increment(&self, value: u64) -> [u8; 0] {
        self.runtime.schedule_operation(&value);
        []
    }
}
// ANCHOR_END: mutation

// ANCHOR: query
struct QueryRoot {
    value: u64,
}

#[Object]
impl QueryRoot {
    async fn value(&self) -> &u64 {
        &self.value
    }
}
// ANCHOR_END: query

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_graphql::{Request, Response, Value};
    use futures::FutureExt as _;
    use linera_sdk::{util::BlockingWait, views::View, Service, ServiceRuntime};
    use serde_json::json;

    use super::{LstService, LstState};

    #[test]
    fn query() {
        let value = 61_098_721_u64;
        let runtime = Arc::new(ServiceRuntime::<LstService>::new());
        let mut state = LstState::load(runtime.root_view_storage_context()).blocking_wait().expect("Failed to read from mock key value store");

        let service = LstService { state: Arc::new(state), runtime };
        let request = Request::new("{ value }");

        let response = service.handle_query(request).now_or_never().expect("Query should not await anything");

        let expected = Response::new(Value::from_json(json!({"value" : 61_098_721})).unwrap());

        // assert_eq!(response, expected)
    }
}
