// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the Matching Engine application

#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;

use async_graphql::InputType;
use fungible::{Account as FungibleAccount, FungibleTokenAbi};
use linera_sdk::{
    linera_base_types::{Account, AccountOwner, Amount, ApplicationId, ApplicationPermissions, ChainId, CryptoHash},
    test::{ActiveChain, QueryOutcome, Recipient, TestValidator},
};
use lst::{LstAbi, Operation, Parameters};

#[tokio::test(flavor = "multi_thread")]
async fn single_chain() {
    let (validator, module_id) = TestValidator::with_current_module::<LstAbi, Parameters, ()>().await;

    let mut stake_chain = validator.new_chain().await;
    let admin_account = AccountOwner::from(stake_chain.public_key());

    // publish fungible module
    let fungible_module_id_a = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;
    let fungible_module_id_b = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    let initial_state_a = fungible::InitialStateBuilder::default().with_account(admin_account, Amount::from_tokens(100));
    let params_a = fungible::Parameters::new("A");
    let token_id_a = stake_chain.create_application(fungible_module_id_a, params_a, initial_state_a.build(), vec![]).await;
    let initial_state_b = fungible::InitialStateBuilder::default().with_account(admin_account, Amount::from_tokens(100));
    let params_b = fungible::Parameters::new("B");
    let token_id_b = stake_chain.create_application(fungible_module_id_b, params_b, initial_state_b.build(), vec![]).await;

    // Check the initial starting amounts for chain a and chain b
    for (owner, amount) in [(admin_account, Some(Amount::from_tokens(100)))] {
        let value = fungible::query_account(token_id_a, &stake_chain, owner).await;
        assert_eq!(value, amount);
    }
    for (owner, amount) in [(admin_account, Some(Amount::from_tokens(100)))] {
        let value = fungible::query_account(token_id_b, &stake_chain, owner).await;
        assert_eq!(value, amount);
    }

    // Creating the stake chain
    let tokens = [token_id_a];
    let stake_parameter = Parameters { tokens };
    let lst_id = stake_chain.create_application(module_id, stake_parameter, (), vec![token_id_a.forget_abi()]).await;

    let stake_cert = stake_chain
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::Stake {
                    owner: admin_account,
                    amount: Amount::from_tokens(1),
                },
            );
        })
        .await;

    stake_chain
        .add_block(|block| {
            block.with_messages_from(&stake_cert);
        })
        .await;
    // lst_id.
    // let QueryOutcome { response, .. } = stake_chain.graphql_query(lst_id, "query { owner }").await;
    // let state_value = response["owner"].to_string();
    // println!("temp: {:?}", state_value);
    let temp = fungible::query_account(token_id_a, &stake_chain, lst_id.application_description_hash.into()).await;
    println!("temp: {:?}", temp);
    // let admin_balance = fungible::query_account(token_id_a, &stake_chain, module_id.contract_blob_hash.into()).await;
    // println!("stake_chain.owner_balances: {:?}", admin_balance);
    // assert_eq!(admin_balance, Some(Amount::from_tokens(1)));
}

#[tokio::test]
async fn multiple_chains() {
    let (validator, module_id) = TestValidator::with_current_module::<LstAbi, Parameters, ()>().await;

    let mut user_chain_a = validator.new_chain().await;
    let owner_a = AccountOwner::from(user_chain_a.public_key());

    let mut user_chain_b = validator.new_chain().await;
    let owner_b = AccountOwner::from(user_chain_b.public_key());

    let mut stake_chain = validator.new_chain().await;
    let admin_account = AccountOwner::from(stake_chain.public_key());

    // publish fungible module
    let fungible_module_id_a = user_chain_a
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;
    let fungible_module_id_b = user_chain_b
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    let initial_state_a = fungible::InitialStateBuilder::default().with_account(owner_a, Amount::from_tokens(100));
    let params_a = fungible::Parameters::new("A");
    let token_id_a = user_chain_a.create_application(fungible_module_id_a, params_a, initial_state_a.build(), vec![]).await;
    let initial_state_b = fungible::InitialStateBuilder::default().with_account(owner_b, Amount::from_tokens(100));
    let params_b = fungible::Parameters::new("B");
    let token_id_b = user_chain_b.create_application(fungible_module_id_b, params_b, initial_state_b.build(), vec![]).await;

    // Check the initial starting amounts for chain a and chain b
    for (owner, amount) in [(admin_account, None), (owner_a, Some(Amount::from_tokens(100))), (owner_b, None)] {
        let value = fungible::query_account(token_id_a, &user_chain_a, owner).await;
        assert_eq!(value, amount);
    }
    for (owner, amount) in [(admin_account, None), (owner_a, None), (owner_b, Some(Amount::from_tokens(100)))] {
        let value = fungible::query_account(token_id_b, &user_chain_b, owner).await;
        assert_eq!(value, amount);
    }

    // Creating the stake chain
    let tokens = [token_id_a];
    let matching_parameter = Parameters { tokens };
    let lst_id = stake_chain.create_application(module_id, matching_parameter, (), vec![token_id_a.forget_abi()]).await;

    let temp = fungible::query_account(token_id_a, &stake_chain, lst_id.application_description_hash.into()).await;

    let stake_cert = user_chain_a
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::Stake {
                    owner: owner_a,
                    amount: Amount::from_tokens(1),
                },
            );
        })
        .await;

    let temp = Amount::ONE;

    stake_chain
        .add_block(|block| {
            block.with_messages_from(&stake_cert);
        })
        .await;

    let temp = fungible::query_account(token_id_a, &stake_chain, lst_id.application_description_hash.into()).await;
    println!("temp: {:?}", temp);
}

#[tokio::test(flavor = "multi_thread")]
async fn native_token_transfer() {
    let parameters = fungible::Parameters { ticker_symbol: "NAT".to_owned() };
    let initial_state = fungible::InitialStateBuilder::default().build();
    let (validator, _application_id, recipient_chain) = TestValidator::with_current_application::<FungibleTokenAbi, _, _>(parameters, initial_state).await;
    // let (validator, module_id) = TestValidator::with_current_module::<LstAbi, Parameters, ()>().await;

    // let mut stake_chain = validator.new_chain().await;
    // let admin_account = AccountOwner::from(stake_chain.public_key());

    // // create native token
    // let parameters = fungible::Parameters { ticker_symbol: "NAT".to_owned() };
    // let initial_state = fungible::InitialStateBuilder::default().build();
    // let (validator_native, _application_id, recipient_chain) = TestValidator::with_current_application::<FungibleTokenAbi, _, _>(parameters, initial_state).await;

    let transfer_amount = Amount::from_tokens(2);
    let funding_chain = validator.get_chain(&ChainId::root(0));
    let owner = AccountOwner::from(CryptoHash::test_hash("owner"));
    let account = Account::new(recipient_chain.id(), owner);
    let recipient = Recipient::Account(account);

    let transfer_certificate = funding_chain
        .add_block(|block| {
            block.with_native_token_transfer(AccountOwner::CHAIN, recipient, transfer_amount);
        })
        .await;

    // recipient_chain
    //     .add_block(|block| {
    //         block.with_messages_from(&transfer_certificate);
    //     })
    //     .await;

    // // Creating the stake chain
    // let tokens = [token_id_a];
    // let stake_parameter = Parameters { tokens };
    // let lst_id = stake_chain.create_application(module_id, stake_parameter, (), vec![token_id_a.forget_abi()]).await;

    // let stake_cert = stake_chain
    //     .add_block(|block| {
    //         block.with_operation(
    //             lst_id,
    //             Operation::StakeNative {
    //                 owner: admin_account,
    //                 amount: Amount::from_tokens(1),
    //             },
    //         );
    //     })
    //     .await;

    // stake_chain
    //     .add_block(|block| {
    //         block.with_messages_from(&stake_cert);
    //     })
    //     .await;
    // // lst_id.
    // // let QueryOutcome { response, .. } = stake_chain.graphql_query(lst_id, "query { owner }").await;
    // // let state_value = response["owner"].to_string();
    // // println!("temp: {:?}", state_value);
    // let temp = fungible::query_account(token_id_a, &stake_chain, lst_id.application_description_hash.into()).await;
    // println!("temp: {:?}", temp);
    // let admin_balance = fungible::query_account(token_id_a, &stake_chain, module_id.contract_blob_hash.into()).await;
    // println!("stake_chain.owner_balances: {:?}", admin_balance);
    // assert_eq!(admin_balance, Some(Amount::from_tokens(1)));
}
#[tokio::test()]
async fn transfer_to_owner() {
    let parameters = fungible::Parameters { ticker_symbol: "NAT".to_owned() };
    let initial_state = fungible::InitialStateBuilder::default().build();
    let (validator, _application_id, recipient_chain) = TestValidator::with_current_application::<FungibleTokenAbi, _, _>(parameters, initial_state).await;

    // let transfer_amount = Amount::ONE;
    // let funding_chain = validator.get_chain(&ChainId::root(0));
    // let recipient = Recipient::chain(recipient_chain.id());

    // let transfer_certificate = funding_chain
    //     .add_block(|block| {
    //         block.with_native_token_transfer(AccountOwner::CHAIN, recipient, transfer_amount);
    //     })
    //     .await;

    // recipient_chain
    //     .add_block(|block| {
    //         block.with_messages_from(&transfer_certificate);
    //     })
    //     .await;
}
