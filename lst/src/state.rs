use async_graphql::SimpleObject;

use linera_sdk::{
    linera_base_types::{AccountOwner, Amount, ApplicationId},
    views::{linera_views, MapView, RegisterView, RootView, SetView, ViewStorageContext},
};

#[derive(RootView, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct LstState {
    pub stake_balances: MapView<AccountOwner, Amount>,
    pub approved_lst_set: SetView<ApplicationId>,
    pub lst_with_native_stake: SetView<ApplicationId>,
}
