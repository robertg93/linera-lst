#![cfg(not(target_arch = "wasm32"))]

use linera_sdk::{
    linera_base_types::{Account, AccountOwner, Amount, ChainId},
    test::{Recipient, TestValidator},
};
use lst::{LstAbi, Operation, Parameters};

/////////// Add new lst token to lst app ///////////
/// 1. create protocol liquid token
/// 2. create lst app with liquid token as parameter
/// 3. create new stake token "FUN"
/// 4. transfer all FUN lst to stake chain
/// 5. use Operation::NewLst to add new lst
/// 6. check balances
#[test_log::test(tokio::test)]
async fn add_new_lst() {
    //create a new validator
    let validator = TestValidator::new().await;
    let mut stake_chain = validator.new_chain().await;
    let admin_account = AccountOwner::from(stake_chain.public_key());

    // create protocol lst
    let protocol_token_module_id = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    // create protocol lst
    let initial_token_state = fungible::InitialStateBuilder::default().with_account(admin_account, Amount::from_tokens(100));
    let protocol_token_params = fungible::Parameters::new("PLST");
    let protocol_lst_id = stake_chain
        .create_application(protocol_token_module_id, protocol_token_params, initial_token_state.build(), vec![])
        .await;

    // check if admin has protocol lst
    let admin_balance = fungible::query_account(protocol_lst_id, &stake_chain, admin_account).await;
    assert_eq!(admin_balance, Some(Amount::from_tokens(100)));

    // create lst app
    let lst_module_id = stake_chain.publish_current_module::<LstAbi, Parameters, ()>().await;

    let stake_parameter = Parameters { protocol_lst: protocol_lst_id };
    let lst_id = stake_chain.create_application(lst_module_id, stake_parameter, (), vec![]).await;

    // create new stake token "FUN"
    let fun_token_module_id = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    // create initial state for FUN token
    let fun_initial_amount = Amount::from_tokens(100);
    let initial_fun_state = fungible::InitialStateBuilder::default().with_account(admin_account, fun_initial_amount);
    let fun_token_params = fungible::Parameters::new("FUN");
    let fun_token_id = stake_chain.create_application(fun_token_module_id, fun_token_params, initial_fun_state.build(), vec![]).await;

    // check if admin has FUN token
    let admin_fun_balance = fungible::query_account(fun_token_id, &stake_chain, admin_account).await;
    assert_eq!(admin_fun_balance, Some(fun_initial_amount));

    //transfer all FUN lst to stake chain
    let lst_app_vault = fungible::Account {
        chain_id: stake_chain.id(),
        owner: lst_id.application_description_hash.into(),
    };

    stake_chain
        .add_block(|block| {
            block.with_operation(
                fun_token_id,
                fungible::Operation::Transfer {
                    owner: admin_account,
                    amount: fun_initial_amount,
                    target_account: lst_app_vault,
                },
            );
        })
        .await;

    // verify transfer
    let lst_app_balance = fungible::query_account(fun_token_id, &stake_chain, AccountOwner::from(lst_id)).await;
    assert_eq!(lst_app_balance, Some(fun_initial_amount));

    stake_chain
        .add_block(|block| {
            block.with_operation(lst_id, Operation::NewLst { token_id: fun_token_id.forget_abi() });
        })
        .await;
}

/////////// Native stake for protocol lst scenario ///////////
/// 1. create protocol liquid token
/// 2. create user chain and fund it with native tokens
/// 3. create lst app with liquid token as parameter
/// 4. transfer protocol lst to lst app vault
/// 5. user stakes native tokens to get protocol lst
/// 6. verify user received protocol lst and app received native tokens
#[test_log::test(tokio::test)]
async fn native_stake_protocol_lst() {
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
    let user_deposit_amount = Amount::from_tokens(1000);
    // send native token to user
    let transfer_certificate = funding_chain
        .add_block(|block| {
            block.with_native_token_transfer(AccountOwner::CHAIN, recipient_user, user_deposit_amount);
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
    assert_eq!(recipient_balance, Some(user_deposit_amount));

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
    let lst_app_vault = fungible::Account {
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
                    target_account: lst_app_vault,
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

/////////// Native stake for other lst type scenario ///////////
/// 1. create user chain and fund it with native tokens
/// 2. create protocol liquid token
/// 3. create lst app with liquid token as parameter
/// 4. transfer protocol lst to lst app vault
/// 5. create new lst token "FUN"
/// 6. transfer all FUN lst to stake chain
/// 7. use Operation::NewLst to add new lst
/// 8. stake native to get FUN lst
#[test_log::test(tokio::test)]
async fn native_stake_other_lst_type() {
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
    let user_deposit_amount = Amount::from_tokens(1000);
    // send native token to user
    let transfer_certificate = funding_chain
        .add_block(|block| {
            block.with_native_token_transfer(AccountOwner::CHAIN, recipient_user, user_deposit_amount);
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
    assert_eq!(recipient_balance, Some(user_deposit_amount));

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
    let lst_app_vault = fungible::Account {
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
                    target_account: lst_app_vault,
                },
            );
        })
        .await;

    // create new stake token "FUN"
    let fun_token_module_id = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    // create initial state for FUN token
    let fun_initial_amount = Amount::from_tokens(100);
    let initial_fun_state = fungible::InitialStateBuilder::default().with_account(admin_account, fun_initial_amount);
    let fun_token_params = fungible::Parameters::new("FUN");
    let fun_token_id = stake_chain.create_application(fun_token_module_id, fun_token_params, initial_fun_state.build(), vec![]).await;

    // check if admin has FUN token
    let admin_fun_balance = fungible::query_account(fun_token_id, &stake_chain, admin_account).await;
    assert_eq!(admin_fun_balance, Some(fun_initial_amount));

    //transfer all FUN lst to stake chain
    stake_chain
        .add_block(|block| {
            block.with_operation(
                fun_token_id,
                fungible::Operation::Transfer {
                    owner: admin_account,
                    amount: fun_initial_amount,
                    target_account: lst_app_vault,
                },
            );
        })
        .await;

    // verify transfer
    let lst_app_balance = fungible::query_account(fun_token_id, &stake_chain, AccountOwner::from(lst_id)).await;
    assert_eq!(lst_app_balance, Some(fun_initial_amount));

    // add new lst
    let new_lst_cert = stake_chain
        .add_block(|block| {
            block.with_operation(lst_id, Operation::NewLst { token_id: fun_token_id.forget_abi() });
        })
        .await;

    // receive msg on stake chain not sure if needed
    stake_chain
        .add_block(|block| {
            block.with_messages_from(&new_lst_cert);
        })
        .await;

    // stake native token and get FUN lst by user on user chain
    let stake_cert = user_chain
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::StakeNative {
                    user: user_account,
                    amount: Amount::from_tokens(10),
                    lst_type_out: fun_token_id.forget_abi(),
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

    // check user FUN lst balance
    let user_balance = fungible::query_account(fun_token_id, &stake_chain, user_account).await;
    assert_eq!(user_balance, Some(Amount::from_tokens(10)));

    // check app native balance
    let app_native_balance = stake_chain.owner_balance(&lst_id.application_description_hash.into()).await;
    assert_eq!(app_native_balance, Some(Amount::from_tokens(10)));
}

/////////// Stake lst for protocol lst scenario ///////////
/// 1. create user chain and fund it with native tokens
/// 2. create protocol liquid token
/// 3. create lst app with liquid token as parameter
/// 4. transfer protocol lst to lst app vault
/// 5. create new lst token "FUN"
/// 6. transfer all FUN lst to stake chain
/// 7. use Operation::NewLst to add new lst
/// 8. stake native to get FUN lst
/// 9. stake "FUN" lst to get protocol lst
#[test_log::test(tokio::test)]
async fn stake_lst() {
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
    let user_deposit_amount = Amount::from_tokens(1000);
    // send native token to user
    let transfer_certificate = funding_chain
        .add_block(|block| {
            block.with_native_token_transfer(AccountOwner::CHAIN, recipient_user, user_deposit_amount);
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
    assert_eq!(recipient_balance, Some(user_deposit_amount));

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
    let lst_app_vault = fungible::Account {
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
                    target_account: lst_app_vault,
                },
            );
        })
        .await;

    // create new stake token "FUN"
    let fun_token_module_id = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    // create initial state for FUN token
    let fun_initial_amount = Amount::from_tokens(100);
    let initial_fun_state = fungible::InitialStateBuilder::default().with_account(admin_account, fun_initial_amount);
    let fun_token_params = fungible::Parameters::new("FUN");
    let fun_token_id = stake_chain.create_application(fun_token_module_id, fun_token_params, initial_fun_state.build(), vec![]).await;

    // check if admin has FUN token
    let admin_fun_balance = fungible::query_account(fun_token_id, &stake_chain, admin_account).await;
    assert_eq!(admin_fun_balance, Some(fun_initial_amount));

    //transfer all FUN lst to stake chain
    stake_chain
        .add_block(|block| {
            block.with_operation(
                fun_token_id,
                fungible::Operation::Transfer {
                    owner: admin_account,
                    amount: fun_initial_amount,
                    target_account: lst_app_vault,
                },
            );
        })
        .await;

    // verify transfer
    let lst_app_balance = fungible::query_account(fun_token_id, &stake_chain, AccountOwner::from(lst_id)).await;
    assert_eq!(lst_app_balance, Some(fun_initial_amount));

    // add new lst
    let new_lst_cert = stake_chain
        .add_block(|block| {
            block.with_operation(lst_id, Operation::NewLst { token_id: fun_token_id.forget_abi() });
        })
        .await;

    // receive msg on stake chain not sure if needed
    stake_chain
        .add_block(|block| {
            block.with_messages_from(&new_lst_cert);
        })
        .await;

    // stake native token and get FUN lst by user on user chain
    let stake_cert = user_chain
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::StakeNative {
                    user: user_account,
                    amount: Amount::from_tokens(10),
                    lst_type_out: fun_token_id.forget_abi(),
                },
            );
        })
        .await;

    // receive msg on stake chain (send tokens)
    let temp = stake_chain
        .add_block(|block| {
            block.with_messages_from(&stake_cert);
        })
        .await;

    user_chain
        .add_block(|block| {
            block.with_messages_from(&temp);
        })
        .await;

    // check user FUN lst balance
    let user_balance = fungible::query_account(fun_token_id, &stake_chain, user_account).await;
    assert_eq!(user_balance, None);

    let user_balance = fungible::query_account(fun_token_id, &user_chain, user_account).await;
    println!("user_balance: {:?}", user_balance);

    // check app native balance
    let app_native_balance = stake_chain.owner_balance(&lst_id.application_description_hash.into()).await;
    assert_eq!(app_native_balance, Some(Amount::from_tokens(10)));

    // let user_stake_chain_account = fungible::Account {
    //     chain_id: stake_chain.id(),
    //     owner: user_account,
    // };

    // let user_user_chain_account = fungible::Account {
    //     chain_id: user_chain.id(),
    //     owner: user_account,
    // };
    println!("1");

    // //transfer all fun lst token to user chain
    // let claim_cert = user_chain
    //     .add_block(|block| {
    //         block.with_operation(
    //             fun_token_id,
    //             fungible::Operation::Claim {
    //                 source_account: user_stake_chain_account,
    //                 amount: Amount::from_tokens(10),
    //                 target_account: user_user_chain_account,
    //             },
    //         );
    //     })
    //     .await;
    // println!("2");
    // // receive msg on stake chain (send tokens)
    // stake_chain
    //     .add_block(|block| {
    //         block.with_messages_from(&claim_cert);
    //     })
    //     .await;

    // check user FUN lst balance
    let user_balance = fungible::query_account(fun_token_id, &stake_chain, user_account).await;
    assert_eq!(user_balance, None);
    println!("2,5");
    // check user FUN lst balance
    let user_balance = fungible::query_account(fun_token_id, &user_chain, user_account).await;
    println!("user_balance: {:?}", user_balance);
    // assert_eq!(user_balance, Some(Amount::from_tokens(10)));
    println!("3");
    // stake "FUN" lst to get protocol lst
    let stake_cert = user_chain
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::StakeLst {
                    user: user_account,
                    amount: Amount::from_tokens(10),
                    lst_type_in: fun_token_id.forget_abi(),
                },
            );
        })
        .await;
    println!("4");
    // receive msg on stake chain (send tokens)
    let temp = stake_chain
        .add_block(|block| {
            block.with_messages_from(&stake_cert);
        })
        .await;
    let user_balance = fungible::query_account(protocol_lst_id, &stake_chain, user_account).await;
    println!("user_balance: {:?}", user_balance);

    user_chain
        .add_block(|block| {
            block.with_messages_from(&temp);
        })
        .await;

    println!("5");
    // check user protocol lst balance
    let user_balance = fungible::query_account(protocol_lst_id, &user_chain, user_account).await;
    // assert_eq!(user_balance, Some(Amount::from_tokens(20)));
    println!("user_balance: {:?}", user_balance);

    let user_balance = fungible::query_account(protocol_lst_id, &stake_chain, user_account).await;
    // assert_eq!(user_balance, Some(Amount::from_tokens(20)));
    println!("user_balance: {:?}", user_balance);

    // check app native balance
    let app_native_balance = stake_chain.owner_balance(&lst_id.application_description_hash.into()).await;
    assert_eq!(app_native_balance, Some(Amount::from_tokens(20)));
}
