// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the Matching Engine application

#![cfg(not(target_arch = "wasm32"))]

use async_graphql::InputType;
use linera_sdk::{
    linera_base_types::{AccountOwner, Amount, ApplicationId, ApplicationPermissions},
    test::{ActiveChain, QueryOutcome, TestValidator},
};
use lst::{LstAbi, Operation, Parameters};

/// Test transferring tokens across microchains.
///
/// Creates the application on a `sender_chain`, initializing it with a single account with some
/// tokens for that chain's owner. Transfers some of those tokens to a new `receiver_chain`, and
/// checks that the balances on each microchain are correct.
#[tokio::test]
async fn test_cross_chain_transfer() {
    // // create a new validator
    // let validator = TestValidator::new().await;
    // // native token chain
    // let mut fungible_chain = validator.new_chain().await;
    // let fungible_module_id: ModuleId<FungibleTokenAbi, Parameters, InitialState> = fungible_chain.publish_bytecode_files_in("../fungible").await;

    // //crete tokens
    // let initial_amount = Amount::from_tokens(100);
    // let owner_account = AccountOwner::from(fungible_chain.public_key());
    // let initial_state = InitialStateBuilder::default().with_account(owner_account, initial_amount);
    // // create native token
    // // let params = Parameters::new("NAT");
    // // let native_token_app_id = fungible_chain.create_application(fungible_module_id, params, initial_state.build(), vec![]).await;
    // let (token_id, backers) = fungible::create_with_accounts(&validator, fungible_module_id, iter::repeat_n(initial_amount, 3)).await;

    // // // create native token
    // // let params = Parameters::new("LST");
    // // let staked_token_app_id = fungible_chain.create_application(fungible_module_id, params, initial_state.build(), vec![]).await;

    // // create a new chain
    // let mut lst_active_chain = validator.new_chain().await;
    // // publish lst module
    // let lst_module_id: ModuleId<LstAbi, ApplicationId<FungibleTokenAbi>, ()> = lst_active_chain.publish_current_module().await;
    // let initial_amount = Amount::from_tokens(100);
    // // create instance of lst
    // let application_id = lst_active_chain.create_application(lst_module_id, token_id, (), vec![token_id.forget_abi()]).await;

    // let (token_id, backers) = fungible::create_with_accounts(&validator, fungible_module_id, iter::repeat_n(initial_amount, 3)).await;
    // lst

    // let increment = 15u64;
    // chain
    //     .add_block(|block| {
    //         block.with_operation(application_id, ());
    //     })
    //     .await;
}

/// Test bouncing some tokens back to the sender.
///
/// Creates the application on a `sender_chain`, initializing it with a single account with some
/// tokens for that chain's owner. Attempts to transfer some tokens to a new `receiver_chain`,
/// but makes the `receiver_chain` reject the transfer message, causing the tokens to be
/// returned back to the sender.
// #[tokio::test]
// async fn test_bouncing_tokens() {
//     let initial_amount = Amount::from_tokens(100);
//     let target_amount = Amount::from_tokens(220);
//     let pledge_amount = Amount::from_tokens(75);

//     let (validator, module_id) = TestValidator::with_current_module::<LstAbi, Parameters, ()>().await;

//     let fungible_chain_owner = AccountSecretKey::Ed25519(Ed25519SecretKey::generate());
//     let fungible_publisher_chain = validator.new_chain_with_keypair(fungible_chain_owner).await;
//     let campaign_chain_owner = AccountSecretKey::Secp256k1(Secp256k1SecretKey::generate());
//     let mut campaign_chain = validator.new_chain_with_keypair(campaign_chain_owner).await;
//     let campaign_account = AccountOwner::from(campaign_chain.public_key());

//     let fungible_module_id = fungible_publisher_chain.publish_bytecode_files_in("../fungible").await;

//     let (token_id, backers) = fungible::create_with_accounts(&validator, fungible_module_id, iter::repeat_n(initial_amount, 3)).await;

//     // let campaign_id = campaign_chain.create_application(module_id, (), (), vec![]).await;
// }

#[tokio::test]
async fn test_create_lst() {
    let (validator, module_id) = TestValidator::with_current_module::<LstAbi, Parameters, ()>().await;

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
