// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;
use fungible::{Account as FungibleAccount, FungibleTokenAbi};
use linera_sdk::{
    linera_base_types::{Account, AccountOwner, Amount, ApplicationId, WithContractAbi},
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

impl WithContractAbi for LstContract {
    type Abi = LstAbi;
}

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
            Operation::StakeNative { owner, amount } => {
                let chain_id = self.runtime.chain_id();
                self.runtime.transfer(owner, Account { chain_id, owner }, amount);
            }
            Operation::Stake { owner, amount } => {
                // Check if the user already has a stake
                let current_amount = match self.state.stake_balances.get(&owner).await {
                    Ok(Some(current)) => current,
                    Ok(None) => Amount::ZERO,
                    Err(e) => panic!("Failed to get stake balance: {}", e),
                };

                // Update the stake by adding the new amount to the existing one
                let new_amount = current_amount.try_add(amount).expect("Failed to add stake balance");
                self.state.stake_balances.insert(&owner, new_amount).expect("Failed to insert stake balance");

                if self.runtime.chain_id() == self.runtime.application_creator_chain_id() {
                    self.stake_from_local_account(owner, amount).await;
                } else {
                    self.stake_from_remote_account(owner, amount);
                }
                // // Transfer the native token to the contract
                // let native_token_id = self.native_token_app_id();
                // self.receive_from_account(owner, amount, native_token_id);

                // // Transfer the staked token to the user
                // let staked_token_id = self.staked_token_app_id();
                // // self.send_to(amount, owner, staked_token_id);
            }
            Operation::Unstake { owner, amount } => {
                // Check if the user has a stake
                let current_amount = match self.state.stake_balances.get(&owner).await {
                    Ok(Some(current)) => current,
                    Ok(None) => panic!("No stake found for user"),
                    Err(e) => panic!("Failed to get stake balance: {}", e),
                };

                // Ensure the user has enough stake to unstake
                if current_amount < amount {
                    panic!("Insufficient stake balance");
                }

                // Update the stake by subtracting the amount
                let new_amount = current_amount.try_sub(amount).expect("Failed to subtract stake balance");
                self.state.stake_balances.insert(&owner, new_amount).expect("Failed to insert stake balance");
            }
            Operation::Swap { owner, amount } => {
                println!("Swap operation");
            }
            Operation::Test => {
                println!("Test operation");
            }
        }
    }
    // ANCHOR_END: execute_operation

    async fn execute_message(&mut self, message: Message) {
        match message {
            Message::StakeLocalAccount { owner, amount } => {
                assert_eq!(
                    self.runtime.chain_id(),
                    self.runtime.application_creator_chain_id(),
                    "Action can only be executed on the chain that created the crowd-funding \
                    campaign"
                );
                self.stake_from_local_account(owner, amount).await;
            }
        }
    }

    // ANCHOR: store
    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
    // ANCHOR_END: store
}

impl LstContract {
    fn native_token_app_id(&mut self) -> ApplicationId<FungibleTokenAbi> {
        self.runtime.application_parameters().tokens[0]
    }
    // fn staked_token_app_id(&mut self) -> ApplicationId<FungibleTokenAbi> {
    //     self.runtime.application_parameters().tokens[1]
    // }
    /// Adds a pledge from a local account to the remote campaign chain.
    fn stake_from_remote_account(&mut self, owner: AccountOwner, amount: Amount) {
        assert!(amount > Amount::ZERO, "Stake is empty");
        // The stake chain.
        let chain_id = self.runtime.application_creator_chain_id();
        // First, move the funds to the campaign chain (under the same owner).
        // TODO(#589): Simplify this when the messaging system guarantees atomic delivery
        // of all messages created in the same operation/message.
        let target_account = FungibleAccount { chain_id, owner };
        let call = fungible::Operation::Transfer { owner, amount, target_account };
        let fungible_id = self.native_token_app_id();
        self.runtime.call_application(/* authenticated by owner */ true, fungible_id, &call);
        // Second, schedule the attribution of the funds to the (remote) campaign.
        self.runtime.prepare_message(Message::StakeLocalAccount { owner, amount }).with_authentication().send_to(chain_id);
    }

    /// Adds a pledge from a local account to the campaign chain.
    async fn stake_from_local_account(&mut self, owner: AccountOwner, amount: Amount) {
        assert!(amount > Amount::ZERO, "Pledge is empty");
        let fungible_id = self.native_token_app_id();
        self.receive_from_account(owner, amount, fungible_id);
    }
    /// Transfers `amount` tokens from the funds in custody to the `owner`'s account.
    fn send_to(&mut self, amount: Amount, owner: AccountOwner, fungible_id: ApplicationId<FungibleTokenAbi>) {
        let target_account = FungibleAccount {
            chain_id: self.runtime.chain_id(),
            owner,
        };
        let transfer = fungible::Operation::Transfer {
            owner: self.runtime.application_id().into(),
            amount,
            target_account,
        };

        self.runtime.call_application(true, fungible_id, &transfer);
    }

    /// Calls into the Fungible Token application to receive tokens from the given account.
    fn receive_from_account(&mut self, owner: AccountOwner, amount: Amount, fungible_id: ApplicationId<FungibleTokenAbi>) {
        let app_owner = self.runtime.application_id().into();

        let target_account = FungibleAccount {
            chain_id: self.runtime.chain_id(),
            owner: app_owner,
        };
        let transfer = fungible::Operation::Transfer { owner, amount, target_account };
        self.runtime.call_application(false, fungible_id, &transfer);
    }

    // /// Calls into the Fungible Token application to receive tokens from the given account.
    // fn receive_from_account(&mut self, owner: &AccountOwner, amount: &Amount, nature: &OrderNature, price: &Price) {
    //     let destination = Account {
    //         chain_id: self.runtime.chain_id(),
    //         owner: self.runtime.application_id().into(),
    //     };
    //     let (amount, token_idx) = Self::get_amount_idx(nature, price, amount);
    //     self.transfer(*owner, amount, destination, token_idx)
    // }

    // /// Transfers `amount` tokens from the funds in custody to the `destination`.
    // fn send_to(&mut self, transfer: Transfer) {
    //     let destination = transfer.account;
    //     let owner_app = self.runtime.application_id().into();
    //     self.transfer(owner_app, transfer.amount, destination, transfer.token_idx);
    // }

    // /// Transfers tokens from the owner to the destination
    // fn transfer(&mut self, owner: AccountOwner, amount: Amount, target_account: Account, token_idx: u32) {
    //     let transfer = fungible::Operation::Transfer { owner, amount, target_account };
    //     let token = self.fungible_id(token_idx);
    //     self.runtime.call_application(true, token, &transfer);
    // }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use futures::FutureExt as _;
    use linera_sdk::{
        abis::fungible::{self, FungibleTokenAbi, Parameters},
        linera_base_types::{AccountOwner, AccountSecretKey, Amount, ApplicationId, ChainId, Secp256k1SecretKey},
        util::BlockingWait,
        views::View,
        Contract, ContractRuntime,
    };
    use lst::{LstAbi, Operation};

    use super::{LstContract, LstState};

    #[test]
    fn operation() {
        //     let native_params = Parameters::new("NAT");
        //     let staked_params = Parameters::new("LST");
        let application_id_native = ApplicationId::default().with_abi::<FungibleTokenAbi>();
        let application_id_staked = ApplicationId::default().with_abi::<FungibleTokenAbi>();
        let lst_id_staked = ApplicationId::default().with_abi::<LstAbi>();
        let params = lst::Parameters { tokens: [application_id_native] };

        let mut runtime = ContractRuntime::new().with_application_parameters(params);
        runtime.set_chain_id(ChainId::default());
        runtime.set_application_id(lst_id_staked);
        let state = LstState::load(runtime.root_view_storage_context()).blocking_wait().expect("Failed to read from mock key value store");
        let mut lst: LstContract = LstContract { state, runtime };

        let user_keypair = AccountSecretKey::Secp256k1(Secp256k1SecretKey::generate());
        let user_pubkey = AccountOwner::from(user_keypair.public());
        let response = lst
            .execute_operation(Operation::Stake {
                owner: user_pubkey,
                amount: Amount::ONE,
            })
            .blocking_wait();

        //     assert_eq!(response, ());
    }
}
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
