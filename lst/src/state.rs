// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use async_graphql::SimpleObject;

use fungible::FungibleTokenAbi;
use linera_sdk::{
    linera_base_types::{AccountOwner, Amount, ApplicationId},
    views::{linera_views, MapView, RootView, SetView, ViewStorageContext},
};

#[derive(RootView, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct LstState {
    pub stake_balances: MapView<AccountOwner, Amount>,
    pub approved_lst_set: SetView<ApplicationId>,
}
