// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::ops::{Add, Sub};

use linera_sdk::{
    linera_base_types::{Amount, WithContractAbi},
    views::{RootView, View},
    Contract, ContractRuntime,
};
use lst::{LstAbi, Message, Operation, Parameters};
use state::LstState;

pub struct LstContract {
    state: LstState,
    runtime: ContractRuntime<Self>,
}
linera_sdk::contract!(LstContract);

// ANCHOR: declare_abi
impl WithContractAbi for LstContract {
    type Abi = LstAbi;
}
// ANCHOR_END: declare_abi

impl Contract for LstContract {
    type Message = Message;
    type InstantiationArgument = ();
    type Parameters = Parameters;
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = LstState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
        LstContract { state, runtime }
    }

    async fn instantiate(&mut self, _: ()) {
        // Validate that the application parameters were configured correctly.
        self.runtime.application_parameters();
    }

    async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
        match operation {
            Operation::Stake { owner, amount } => {
                // // Check if the user already has a stake
                // let current_amount = match self.state.stake_balances.get(&owner).await {
                //     Ok(Some(current)) => current,
                //     Ok(None) => 0,
                //     Err(e) => panic!("Failed to get stake balance: {}", e),
                // };

                // // Update the stake by adding the new amount to the existing one
                // let new_amount = current_amount.add(amount);
                // self.state.stake_balances.insert(&owner, new_amount).expect("Failed to insert stake balance");

                // self.runtime.application_parameters (authenticated, application, call)
            }
            Operation::Unstake { owner, amount } => {
                // // Check if the user has a stake
                // let current_amount = match self.state.stake_balances.get(&owner).await {
                //     Ok(Some(current)) => current,
                //     Ok(None) => panic!("No stake found for user"),
                //     Err(e) => panic!("Failed to get stake balance: {}", e),
                // };

                // // Ensure the user has enough stake to unstake
                // if current_amount < amount {
                //     panic!("Insufficient stake balance");
                // }

                // // Update the stake by subtracting the amount
                // let new_amount = current_amount.sub(amount);
                // self.state.stake_balances.insert(&owner, new_amount).expect("Failed to insert stake balance");
            }
        }
    }
    // ANCHOR_END: execute_operation

    async fn execute_message(&mut self, _message: Message) {
        // panic!("Lst application doesn't support any cross-chain messages");
    }

    // ANCHOR: store
    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
    // ANCHOR_END: store
}

// impl LstContract {
//     fn native_token_app_id(&mut self) -> ApplicationId<FungibleTokenAbi> {
//         self.runtime.application_parameters().0
//     }
//     fn staked_token_app_id(&mut self) -> ApplicationId<FungibleTokenAbi> {
//         self.runtime.application_parameters().1
//     }
// }

// #[cfg(test)]
// mod tests {
//     use std::str::FromStr;

//     use futures::FutureExt as _;
//     use linera_sdk::{
//         abis::fungible::{self, FungibleTokenAbi, Parameters},
//         linera_base_types::{AccountOwner, AccountSecretKey, Amount, ApplicationId, Secp256k1SecretKey},
//         util::BlockingWait,
//         views::View,
//         Contract, ContractRuntime,
//     };
//     use lst::Operation;

//     use super::{LstContract, LstState};

//     // ANCHOR: counter_test
//     #[test]
//     fn operation() {
//         //     let native_params = Parameters::new("NAT");
//         //     let staked_params = Parameters::new("LST");
//         let application_id_native = ApplicationId::default().with_abi::<FungibleTokenAbi>();
//         let application_id_staked = ApplicationId::default().with_abi::<FungibleTokenAbi>();

//         let runtime = ContractRuntime::new().with_application_parameters((application_id_native, application_id_staked));
//         let state = LstState::load(runtime.root_view_storage_context()).blocking_wait().expect("Failed to read from mock key value store");
//         let mut counter = LstContract { state, runtime };

//         let initial_value = ();
//         counter.instantiate(initial_value).now_or_never().expect("Initialization of counter state should not await anything");

//         //     let user_keypair = AccountSecretKey::Secp256k1(Secp256k1SecretKey::generate());
//         //     let user_pubkey = AccountOwner::from(user_keypair.public());
//         //     let response = counter
//         //         .execute_operation(Operation::Stake {
//         //             owner: user_pubkey,
//         //             amount: Amount::ONE,
//         //         })
//         //         .now_or_never()
//         //         .expect("Execution of counter operation should not await anything");

//         //     assert_eq!(response, ());
//     }
// ANCHOR_END: counter_test

// #[test]
// #[should_panic(expected = "Lst application doesn't support any cross-chain messages")]
// fn message() {
//     let initial_value = 72_u64;
//     let mut counter = create_and_instantiate_counter(initial_value);

//     counter.execute_message(()).now_or_never().expect("Execution of counter operation should not await anything");
// }

// #[test]
// fn cross_application_call() {
//     let initial_value = 2_845_u64;
//     let mut counter = create_and_instantiate_counter(initial_value);

//     let increment = 8_u64;

//     let response = counter.execute_operation(increment).now_or_never().expect("Execution of counter operation should not await anything");

//     let expected_value = initial_value + increment;

//     assert_eq!(response, expected_value);
//     assert_eq!(*counter.state.value.get(), expected_value);
// }

// fn create_and_instantiate_counter(initial_value: u64) -> LstContract {
//     let runtime = ContractRuntime::new().with_application_parameters(());
//     let mut contract = LstContract {
//         state: LstState::load(runtime.root_view_storage_context()).blocking_wait().expect("Failed to read from mock key value store"),
//         runtime,
//     };

//     contract.instantiate(initial_value).now_or_never().expect("Initialization of counter state should not await anything");

//     assert_eq!(*contract.state.value.get(), initial_value);

//     contract
// }
// }
