// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the Fungible Token application.

#![cfg(not(target_arch = "wasm32"))]

use fungible::{Account, FungibleTokenAbi, InitialState, InitialStateBuilder, Operation, Parameters};
use linera_sdk::{
    linera_base_types::{AccountOwner, AccountSecretKey, Amount, Ed25519SecretKey, ModuleId, Secp256k1SecretKey},
    test::{Medium, MessageAction, TestValidator},
};
use lst::LstAbi;
use std::{borrow::Borrow, iter};

/// Test transferring tokens across microchains.
///
/// Creates the application on a `sender_chain`, initializing it with a single account with some
/// tokens for that chain's owner. Transfers some of those tokens to a new `receiver_chain`, and
/// checks that the balances on each microchain are correct.
#[tokio::test]
async fn test_cross_chain_transfer() {
    let initial_amount = Amount::from_tokens(100);
    let target_amount = Amount::from_tokens(220);
    let pledge_amount = Amount::from_tokens(75);

    let (validator, module_id) = TestValidator::with_current_module::<LstAbi, (), ()>().await;
    // let (validator, module_id) = TestValidator::with_current_module::<CrowdFundingAbi, ApplicationId<FungibleTokenAbi>, InstantiationArgument>().await;

    let fungible_chain_owner = AccountSecretKey::Ed25519(Ed25519SecretKey::generate());
    let mut fungible_publisher_chain = validator.new_chain_with_keypair(fungible_chain_owner).await;
    let campaign_chain_owner = AccountSecretKey::Secp256k1(Secp256k1SecretKey::generate());
    let mut campaign_chain = validator.new_chain_with_keypair(campaign_chain_owner).await;
    let campaign_account = AccountOwner::from(campaign_chain.public_key());

    let fungible_module_id: ModuleId<FungibleTokenAbi, Parameters, InitialState> = fungible_publisher_chain.publish_bytecode_files_in("../fungible").await;
    // let fungible_module_id_2 = fungible_publisher_chain.publish_bytecode_files_in("../fungible").await;

    let mut sender_chain = validator.new_chain().await;
    let sender_account = AccountOwner::from(sender_chain.public_key());

    let initial_state = InitialStateBuilder::default().with_account(sender_account, initial_amount);
    let params = Parameters::new("FUN");
    let application_id = fungible_publisher_chain.create_application(fungible_module_id, params, initial_state.build(), vec![]).await;

    let (token_id, backers) = fungible::create_with_accounts(&validator, fungible_module_id, iter::repeat_n(initial_amount, 3)).await;
    // let (token_id, backers) = fungible::create_with_accounts(&validator, fungible_module_id, iter::repeat_n(initial_amount, 3)).await;
}

/// Test bouncing some tokens back to the sender.
///
/// Creates the application on a `sender_chain`, initializing it with a single account with some
/// tokens for that chain's owner. Attempts to transfer some tokens to a new `receiver_chain`,
/// but makes the `receiver_chain` reject the transfer message, causing the tokens to be
/// returned back to the sender.
#[tokio::test]
async fn test_bouncing_tokens() {
    let initial_amount = Amount::from_tokens(20);
    let transfer_amount = Amount::from_tokens(15);

    let (validator, module_id) = TestValidator::with_current_module::<fungible::FungibleTokenAbi, Parameters, InitialState>().await;
    let mut sender_chain = validator.new_chain().await;
    let sender_account = AccountOwner::from(sender_chain.public_key());

    let initial_state = InitialStateBuilder::default().with_account(sender_account, initial_amount);
    let params = Parameters::new("FUN");
    let application_id = sender_chain.create_application(module_id, params, initial_state.build(), vec![]).await;
}
