// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/*! ABI of the Matching Engine Example Application */

use async_graphql::{scalar, InputObject, Request, Response, SimpleObject};
use fungible::FungibleTokenAbi;
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{ApplicationId, ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

pub struct MatchingEngineAbi;

impl ContractAbi for MatchingEngineAbi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for MatchingEngineAbi {
    type Query = Request;
    type QueryResponse = Response;
}

/// When the matching engine is created we need to create to
/// trade between two tokens 0 and 1. Those two tokens
/// are put as parameters in the creation of the matching engine
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Parameters {
    /// The token0 and token1 used for the matching engine
    pub tokens: [ApplicationId<FungibleTokenAbi>; 2],
}

scalar!(Parameters);

/// Operations that can be sent to the application.
#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    /// The order that is going to be executed on the chain of the order book.
    ExecuteOrder,
    /// Close this chain, and cancel all orders.
    /// Requires that this application is authorized to close the chain.
    CloseChain,
}

/// Messages that can be processed by the application.
#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    /// The order being transmitted from the chain and received by the chain of the order book.
    ExecuteOrder { order: u64 },
}
