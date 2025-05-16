// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the Matching Engine application

#![cfg(not(target_arch = "wasm32"))]

use async_graphql::InputType;
use linera_sdk::{
    linera_base_types::{AccountOwner, Amount, ApplicationId, ApplicationPermissions},
    test::{ActiveChain, QueryOutcome, TestValidator},
};
use lst_test::{LstTestAbi, Operation, Parameters};

// pub async fn get_orders(application_id: ApplicationId<MatchingEngineAbi>, chain: &ActiveChain, account_owner: AccountOwner) -> Option<Vec<OrderId>> {
//     let query = format!("query {{ accountInfo {{ entry(key: {}) {{ value {{ orders }} }} }} }}", account_owner.to_value());
//     let QueryOutcome { response, .. } = chain.graphql_query(application_id, query).await;
//     let orders = &response["accountInfo"]["entry"]["value"]["orders"];
//     let values = orders.as_array()?.iter().map(|order| order.as_u64().unwrap()).collect();
//     Some(values)
// }

/// Test creating a matching engine, pushing some orders, canceling some and
/// seeing how the transactions went.
///
/// The operation is done in exactly the same way with the same amounts
/// and quantities as the corresponding end to end test.
///
/// We have 3 chains:
/// * The chain A of User_a for tokens A
/// * The chain B of User_b for tokens B
/// * The admin chain of the matching engine.
///
/// The following operations are done:
/// * We create users and assign them their initial positions:
///   * user_a with 10 tokens A.
///   * user_b with 9 tokens B.
/// * Then we create the following orders:
///   * User_a: Offer to buy token B in exchange for token A for a price of 1 (or 2) with
///     a quantity of 3 token B.
///     User_a thus commits 3 * 1 + 3 * 2 = 9 token A to the matching engine chain and is
///     left with 1 token A on chain A
///   * User_b: Offer to sell token B in exchange for token A for a price of 2 (or 4) with
///     a quantity of 4 token B
///     User_b thus commits 4 + 4 = 8 token B on the matching engine chain and is left
///     with 1 token B.
/// * The price that is matching is 2 where a transaction can actually occur
///   * Only 3 token B can be exchanged against 6 tokens A.
///   * So, the order from user_b is only partially filled.
/// * Then the orders are cancelled and the user get back their tokens.
///   After the exchange we have
///   * User_a: It has 9 - 6 = 3 token A and the newly acquired 3 token B.
///   * User_b: It has 8 - 3 = 5 token B and the newly acquired 6 token A
#[tokio::test]
async fn single_transaction() {
    let (validator, module_id) = TestValidator::with_current_module::<LstTestAbi, Parameters, ()>().await;

    let mut user_chain_a = validator.new_chain().await;
    let owner_a = AccountOwner::from(user_chain_a.public_key());
    let mut user_chain_b = validator.new_chain().await;
    let owner_b = AccountOwner::from(user_chain_b.public_key());
    let mut matching_chain = validator.new_chain().await;
    let admin_account = AccountOwner::from(matching_chain.public_key());

    let fungible_module_id_a = user_chain_a
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;
    let fungible_module_id_b = user_chain_b
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    let initial_state_a = fungible::InitialStateBuilder::default().with_account(owner_a, Amount::from_tokens(10));
    let params_a = fungible::Parameters::new("A");
    let token_id_a = user_chain_a.create_application(fungible_module_id_a, params_a, initial_state_a.build(), vec![]).await;
    let initial_state_b = fungible::InitialStateBuilder::default().with_account(owner_b, Amount::from_tokens(9));
    let params_b = fungible::Parameters::new("B");
    let token_id_b = user_chain_b.create_application(fungible_module_id_b, params_b, initial_state_b.build(), vec![]).await;

    // Check the initial starting amounts for chain a and chain b
    for (owner, amount) in [(admin_account, None), (owner_a, Some(Amount::from_tokens(10))), (owner_b, None)] {
        let value = fungible::query_account(token_id_a, &user_chain_a, owner).await;
        assert_eq!(value, amount);
    }
    for (owner, amount) in [(admin_account, None), (owner_a, None), (owner_b, Some(Amount::from_tokens(9)))] {
        let value = fungible::query_account(token_id_b, &user_chain_b, owner).await;
        assert_eq!(value, amount);
    }

    // Creating the matching engine chain
    let tokens = [token_id_a, token_id_b];
    let matching_parameter = Parameters { tokens };
    let matching_id = matching_chain
        .create_application(module_id, matching_parameter, (), vec![token_id_a.forget_abi(), token_id_b.forget_abi()])
        .await;
}
