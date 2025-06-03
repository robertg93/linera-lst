#![cfg(not(target_arch = "wasm32"))]

use linera_sdk::{
    linera_base_types::{Account, AccountOwner, Amount, ApplicationId, ApplicationPermissions, ChainId, CryptoHash},
    test::{ActiveChain, QueryOutcome, Recipient, TestValidator},
};
use lst::{LstAbi, Operation, Parameters};

#[test_log::test(tokio::test)]
async fn single_chain() {
    //create a new validator
    let validator = TestValidator::new().await;
    let mut stake_chain = validator.new_chain().await;
    let admin_account = AccountOwner::from(stake_chain.public_key());

    // create protocol lst
    let fungible_module_id_a = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    let initial_state_a = fungible::InitialStateBuilder::default().with_account(admin_account, Amount::from_tokens(100));
    let params_a = fungible::Parameters::new("PLST");
    let protocol_lst_id = stake_chain.create_application(fungible_module_id_a, params_a, initial_state_a.build(), vec![]).await;

    // create lst app
    let publisher = validator.new_chain().await;
    let module_id = publisher.publish_current_module::<LstAbi, Parameters, ()>().await;

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
    let stake_parameter = Parameters { protocol_lst: protocol_lst_id };
    let lst_id = stake_chain.create_application(module_id, stake_parameter, (), vec![]).await;

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
    // let contract_token_balance = fungible::query_account(token_id_a, &stake_chain, lst_id.application_description_hash.into()).await;
    // assert_eq!(contract_token_balance, Some(Amount::from_tokens(1)));
}

// #[test_log::test(tokio::test)]
// async fn multiple_chains() {
//     let (validator, module_id) = TestValidator::with_current_module::<LstAbi, Parameters, ()>().await;

//     let mut user_chain_a = validator.new_chain().await;
//     let owner_a = AccountOwner::from(user_chain_a.public_key());

//     let mut user_chain_b = validator.new_chain().await;
//     let owner_b = AccountOwner::from(user_chain_b.public_key());

//     let mut stake_chain = validator.new_chain().await;
//     let admin_account = AccountOwner::from(stake_chain.public_key());

//     // publish fungible module
//     let fungible_module_id_a = user_chain_a
//         .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
//         .await;
//     let fungible_module_id_b = user_chain_b
//         .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
//         .await;

//     let initial_state_a = fungible::InitialStateBuilder::default().with_account(owner_a, Amount::from_tokens(100));
//     let params_a = fungible::Parameters::new("A");
//     let token_id_a = user_chain_a.create_application(fungible_module_id_a, params_a, initial_state_a.build(), vec![]).await;
//     let initial_state_b = fungible::InitialStateBuilder::default().with_account(owner_b, Amount::from_tokens(100));
//     let params_b = fungible::Parameters::new("B");
//     let token_id_b = user_chain_b.create_application(fungible_module_id_b, params_b, initial_state_b.build(), vec![]).await;

//     // Check the initial starting amounts for chain a and chain b
//     for (owner, amount) in [(admin_account, None), (owner_a, Some(Amount::from_tokens(100))), (owner_b, None)] {
//         let value = fungible::query_account(token_id_a, &user_chain_a, owner).await;
//         assert_eq!(value, amount);
//     }
//     for (owner, amount) in [(admin_account, None), (owner_a, None), (owner_b, Some(Amount::from_tokens(100)))] {
//         let value = fungible::query_account(token_id_b, &user_chain_b, owner).await;
//         assert_eq!(value, amount);
//     }

//     // Creating the stake chain
//     let tokens = [token_id_a];
//     let matching_parameter = Parameters { tokens };
//     let lst_id = stake_chain.create_application(module_id, matching_parameter, (), vec![token_id_a.forget_abi()]).await;

//     let stake_cert = user_chain_a
//         .add_block(|block| {
//             block.with_operation(
//                 lst_id,
//                 Operation::Stake {
//                     owner: owner_a,
//                     amount: Amount::from_tokens(1),
//                 },
//             );
//         })
//         .await;

//     stake_chain
//         .add_block(|block| {
//             block.with_messages_from(&stake_cert);
//         })
//         .await;

//     let contract_token_balance = fungible::query_account(token_id_a, &stake_chain, lst_id.application_description_hash.into()).await;
//     assert_eq!(contract_token_balance, Some(Amount::from_tokens(1)));
// }

#[test_log::test(tokio::test)]
async fn native_stake() {
    //create a new validator
    let validator = TestValidator::new().await;
    let mut stake_chain = validator.new_chain().await;
    let admin_account = AccountOwner::from(stake_chain.public_key());

    // chain which is able to mint native token
    let funding_chain = validator.get_chain(&ChainId::root(0));

    // create a new user chain
    let user_chain = validator.new_chain().await;
    let user_account = AccountOwner::from(user_chain.public_key());
    let recipient_user = Recipient::Account(Account::new(user_chain.id(), user_account));

    // send native token to user
    let transfer_certificate = funding_chain
        .add_block(|block| {
            block.with_native_token_transfer(AccountOwner::CHAIN, recipient_user, Amount::from_tokens(1000));
        })
        .await;

    // receive native token
    user_chain
        .add_block(|block| {
            block.with_messages_from(&transfer_certificate);
        })
        .await;

    // check if user has native token
    let recipient_balance = user_chain.owner_balance(&user_account).await;
    assert_eq!(recipient_balance, Some(Amount::from_tokens(1000)));

    // create protocol lst
    let protocol_token_module_id = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    // create protocol lst
    let initial_token_state = fungible::InitialStateBuilder::default().with_account(admin_account, Amount::from_tokens(100));
    let params_a = fungible::Parameters::new("PLST");
    let protocol_lst_id = stake_chain.create_application(protocol_token_module_id, params_a, initial_token_state.build(), vec![]).await;

    // check if admin has protocol lst
    let admin_balance = fungible::query_account(protocol_lst_id, &stake_chain, admin_account).await;
    assert_eq!(admin_balance, Some(Amount::from_tokens(100)));

    // create lst app
    let lst_module_id = stake_chain.publish_current_module::<LstAbi, Parameters, ()>().await;

    let stake_parameter = Parameters { protocol_lst: protocol_lst_id };
    let lst_id = stake_chain.create_application(lst_module_id, stake_parameter, (), vec![]).await;

    //transfer all lst to stake chain
    let stake_chain_recipient = fungible::Account {
        chain_id: stake_chain.id(),
        owner: lst_id.application_description_hash.into(),
    };

    stake_chain
        .add_block(|block| {
            block.with_operation(
                protocol_lst_id,
                fungible::Operation::Transfer {
                    owner: admin_account,
                    amount: Amount::from_tokens(100),
                    target_account: stake_chain_recipient,
                },
            );
        })
        .await;

    // stake native token and get lst by user on user chain
    let stake_cert = user_chain
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::StakeNative {
                    user: user_account,
                    amount: Amount::from_tokens(10),
                    lst_type_out: protocol_lst_id.forget_abi(),
                },
            );
        })
        .await;

    // receive msg on stake chain (send tokens)
    stake_chain
        .add_block(|block| {
            block.with_messages_from(&stake_cert);
        })
        .await;
    // check user lst balance
    let user_balance = fungible::query_account(protocol_lst_id, &stake_chain, user_account).await;
    assert_eq!(user_balance, Some(Amount::from_tokens(10)));
    // check admin protocol lst balance
    let app_native_balance = stake_chain.owner_balance(&lst_id.application_description_hash.into()).await;
    assert_eq!(app_native_balance, Some(Amount::from_tokens(10)));
}

// #[test_log::test(tokio::test)]
// async fn add_new_lst() {
//     let (validator, module_id) = TestValidator::with_current_module::<LstAbi, Parameters, ()>().await;

//     let mut stake_chain = validator.new_chain().await;
//     let admin_account = AccountOwner::from(stake_chain.public_key());

//     // publish fungible module
//     let fungible_module_id_a = stake_chain
//         .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
//         .await;

//     let initial_state = fungible::InitialStateBuilder::default().with_account(admin_account, Amount::from_tokens(100));
//     //token a
//     let params_a = fungible::Parameters::new("A");
//     let token_id_a = stake_chain.create_application(fungible_module_id_a, params_a, initial_state.build(), vec![]).await;

//     // Check the initial starting amounts for chain a and chain b
//     for (owner, amount) in [(admin_account, Some(Amount::from_tokens(100)))] {
//         let value = fungible::query_account(token_id_a, &stake_chain, owner).await;
//         assert_eq!(value, amount);
//     }

//     // Creating the lst app
//     let tokens = [token_id_a];
//     let stake_parameter = Parameters { tokens };
//     let lst_id = stake_chain.create_application(module_id, stake_parameter, (), vec![token_id_a.forget_abi()]).await;

//     let stake_cert = stake_chain
//         .add_block(|block| {
//             block.with_operation(lst_id, Operation::NewLst { token_id: token_id_a.forget_abi() });
//         })
//         .await;
//     // let contract_token_balance = fungible::query_account(token_id_a, &stake_chain, lst_id.application_description_hash.into()).await;
//     // assert_eq!(contract_token_balance, Some(Amount::from_tokens(1)));
// }
