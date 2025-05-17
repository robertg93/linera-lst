// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/*! ABI of the Lst Example Application */

use async_graphql::{scalar, Request, Response};
use fungible::FungibleTokenAbi;
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{AccountOwner, Amount, ApplicationId, ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

pub struct LstAbi;

impl ContractAbi for LstAbi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for LstAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Parameters {
    /// The token0 and token1 used for the matching engine
    pub tokens: [ApplicationId<FungibleTokenAbi>; 2],
}

scalar!(Parameters);

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    /// Pledge some tokens to the campaign (from an account on the current chain to the campaign chain).
    Stake {
        owner: AccountOwner,
        amount: Amount,
    },
    /// Collect the pledges after the campaign has reached its target (campaign chain only).
    Unstake {
        owner: AccountOwner,
        amount: Amount,
    },
    Test,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    /// The order being transmitted from the chain and received by the chain of the order book.
    Msg { order: u64 },
}
