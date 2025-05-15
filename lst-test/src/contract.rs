// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};
use lst_test::{LstTestAbi, Message, Operation, Parameters};
use state::LstTestState;

pub struct LstTestContract {
    state: LstTestState,
    runtime: ContractRuntime<Self>,
}
linera_sdk::contract!(LstTestContract);

impl WithContractAbi for LstTestContract {
    type Abi = LstTestAbi;
}

impl Contract for LstTestContract {
    type Message = Message;
    type InstantiationArgument = ();
    type Parameters = Parameters;
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = LstTestState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
        LstTestContract { state, runtime }
    }

    async fn instantiate(&mut self, _argument: ()) {
        // Validate that the application parameters were configured correctly.
        let _ = self.runtime.application_parameters();
    }

    async fn execute_operation(&mut self, operation: Operation) -> Self::Response {
        match operation {
            Operation::ExecuteOrder => {
                // let owner = Self::get_owner(&order);
                // let chain_id = self.runtime.chain_id();
                // self.runtime.check_account_permission(owner).expect("Permission for ExecuteOrder operation");
                // if chain_id == self.runtime.application_creator_chain_id() {
                //     self.execute_order_local(order, chain_id).await;
                // } else {
                //     self.execute_order_remote(order);
                // }
            }
            Operation::CloseChain => {
                // let order_ids = self.state.orders.indices().await.expect("Failed to read existing order IDs");
                // for order_id in order_ids {
                //     match self.modify_order(order_id, ModifyAmount::All).await {
                //         Some(transfer) => self.send_to(transfer),
                //         // Orders with amount zero may have been cleared in an earlier iteration.
                //         None => continue,
                //     }
                // }
                // self.runtime.close_chain().expect("The application does not have permissions to close the chain.");
            }
        }
    }

    /// Execution of the order on the creation chain
    async fn execute_message(&mut self, message: Message) {
        assert_eq!(
            self.runtime.chain_id(),
            self.runtime.application_creator_chain_id(),
            "Action can only be executed on the chain that created the matching engine"
        );
        // match message {
        //     Message::ExecuteOrder { order } => {
        //         let owner = Self::get_owner(&order);
        //         let message_id = self.runtime.message_id().expect("Incoming message ID has to be available when executing a message");
        //         self.runtime.check_account_permission(owner).expect("Permission for ExecuteOrder message");
        //         self.execute_order_local(order, message_id.chain_id).await;
        //     }
        // }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}
