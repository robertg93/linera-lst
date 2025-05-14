// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/*! ABI of the Lst Example Application */

use async_graphql::{Request, Response};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{AccountOwner, Amount, ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

// ANCHOR: contract_abi
pub struct LstAbi;

impl ContractAbi for LstAbi {
    type Operation = Operation;
    type Response = ();
}
// ANCHOR_END: contract_abi

// ANCHOR: service_abi
impl ServiceAbi for LstAbi {
    type Query = Request;
    type QueryResponse = Response;
}
// ANCHOR_END: service_abi

/// Operations that can be executed by the application.
#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    /// Pledge some tokens to the campaign (from an account on the current chain to the campaign chain).
    Stake { owner: AccountOwner, amount: Amount },
    /// Collect the pledges after the campaign has reached its target (campaign chain only).
    Unstake { owner: AccountOwner, amount: Amount },
}
