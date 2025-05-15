// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use async_graphql::SimpleObject;

use linera_sdk::{
    linera_base_types::AccountOwner,
    views::{linera_views, MapView, RootView, ViewStorageContext},
};

#[derive(RootView, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct LstState {
    pub stake_balances: MapView<AccountOwner, u64>,
}
