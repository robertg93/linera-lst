#![cfg(not(target_arch = "wasm32"))]

use linera_sdk::{
    linera_base_types::{Account, AccountOwner, Amount, ChainId},
    test::{Recipient, TestValidator},
};
use lst::{LstAbi, Operation, Parameters};

/////////// Add new lst token to lst app ///////////
/// 1. create protocol liquid token
/// 2. create lst app with liquid token as parameter
/// 3. create new stake token "FOO"
/// 4. transfer all "FOO" lst to stake chain
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

    // create new stake token "FOO"
    let foo_token_module_id = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    // create initial state for "FOO" token
    let foo_initial_amount = Amount::from_tokens(100);
    let initial_foo_state = fungible::InitialStateBuilder::default().with_account(admin_account, foo_initial_amount);
    let foo_token_params = fungible::Parameters::new("FOO");
    let foo_token_id = stake_chain.create_application(foo_token_module_id, foo_token_params, initial_foo_state.build(), vec![]).await;

    // check if admin has "FOO" token
    let admin_foo_balance = fungible::query_account(foo_token_id, &stake_chain, admin_account).await;
    assert_eq!(admin_foo_balance, Some(foo_initial_amount));

    //transfer all "FOO" lst to stake chain
    let lst_app_vault = fungible::Account {
        chain_id: stake_chain.id(),
        owner: lst_id.application_description_hash.into(),
    };

    stake_chain
        .add_block(|block| {
            block.with_operation(
                foo_token_id,
                fungible::Operation::Transfer {
                    owner: admin_account,
                    amount: foo_initial_amount,
                    target_account: lst_app_vault,
                },
            );
        })
        .await;

    // verify transfer
    let lst_app_balance = fungible::query_account(foo_token_id, &stake_chain, AccountOwner::from(lst_id)).await;
    assert_eq!(lst_app_balance, Some(foo_initial_amount));

    stake_chain
        .add_block(|block| {
            block.with_operation(lst_id, Operation::NewLst { token_id: foo_token_id.forget_abi() });
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
/// 5. create new lst token "FOO"
/// 6. transfer all "FOO" lst to stake chain
/// 7. use Operation::NewLst to add new lst
/// 8. stake native to get "FOO" lst
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

    // create new stake token "FOO"
    let foo_token_module_id = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    // create initial state for "FOO" token
    let foo_initial_amount = Amount::from_tokens(100);
    let initial_foo_state = fungible::InitialStateBuilder::default().with_account(admin_account, foo_initial_amount);
    let foo_token_params = fungible::Parameters::new("FOO");
    let foo_token_id = stake_chain.create_application(foo_token_module_id, foo_token_params, initial_foo_state.build(), vec![]).await;

    // check if admin has "FOO" token
    let admin_foo_balance = fungible::query_account(foo_token_id, &stake_chain, admin_account).await;
    assert_eq!(admin_foo_balance, Some(foo_initial_amount));

    //transfer all "FOO" lst to stake chain
    stake_chain
        .add_block(|block| {
            block.with_operation(
                foo_token_id,
                fungible::Operation::Transfer {
                    owner: admin_account,
                    amount: foo_initial_amount,
                    target_account: lst_app_vault,
                },
            );
        })
        .await;

    // verify transfer
    let lst_app_balance = fungible::query_account(foo_token_id, &stake_chain, AccountOwner::from(lst_id)).await;
    assert_eq!(lst_app_balance, Some(foo_initial_amount));

    // add new lst
    let new_lst_cert = stake_chain
        .add_block(|block| {
            block.with_operation(lst_id, Operation::NewLst { token_id: foo_token_id.forget_abi() });
        })
        .await;

    // receive msg on stake chain not sure if needed
    stake_chain
        .add_block(|block| {
            block.with_messages_from(&new_lst_cert);
        })
        .await;

    // stake native token and get "FOO" lst by user on user chain
    let stake_cert = user_chain
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::StakeNative {
                    user: user_account,
                    amount: Amount::from_tokens(10),
                    lst_type_out: foo_token_id.forget_abi(),
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

    // check user "FOO" lst balance
    let user_balance = fungible::query_account(foo_token_id, &stake_chain, user_account).await;
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
/// 5. create new lst token "FOO"
/// 6. transfer all "FOO" lst to stake chain
/// 7. use Operation::NewLst to add new lst
/// 8. stake native to get "FOO" lst
/// 9. stake "FOO" lst to get protocol lst
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

    // create new stake token "FOO"
    let foo_token_module_id = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    // create initial state for "FOO" token
    let foo_initial_amount = Amount::from_tokens(100);
    let initial_foo_state = fungible::InitialStateBuilder::default().with_account(admin_account, foo_initial_amount);
    let foo_token_params = fungible::Parameters::new("FOO");
    let foo_token_id = stake_chain.create_application(foo_token_module_id, foo_token_params, initial_foo_state.build(), vec![]).await;

    // check if admin has "FOO" token
    let admin_foo_balance = fungible::query_account(foo_token_id, &stake_chain, admin_account).await;
    assert_eq!(admin_foo_balance, Some(foo_initial_amount));

    //transfer all "FOO" lst to stake chain
    stake_chain
        .add_block(|block| {
            block.with_operation(
                foo_token_id,
                fungible::Operation::Transfer {
                    owner: admin_account,
                    amount: foo_initial_amount,
                    target_account: lst_app_vault,
                },
            );
        })
        .await;

    // verify transfer
    let lst_app_balance = fungible::query_account(foo_token_id, &stake_chain, AccountOwner::from(lst_id)).await;
    assert_eq!(lst_app_balance, Some(foo_initial_amount));

    // add new lst
    let new_lst_cert = stake_chain
        .add_block(|block| {
            block.with_operation(lst_id, Operation::NewLst { token_id: foo_token_id.forget_abi() });
        })
        .await;

    // receive msg on stake chain not sure if needed
    stake_chain
        .add_block(|block| {
            block.with_messages_from(&new_lst_cert);
        })
        .await;

    // stake native token and get "FOO" lst by user on user chain
    let stake_cert = user_chain
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::StakeNative {
                    user: user_account,
                    amount: Amount::from_tokens(10),
                    lst_type_out: foo_token_id.forget_abi(),
                },
            );
        })
        .await;

    // receive msg on stake chain
    let stake_msg = stake_chain
        .add_block(|block| {
            block.with_messages_from(&stake_cert);
        })
        .await;

    // check balances before send msg to user chain
    let user_balance = fungible::query_account(foo_token_id, &stake_chain, user_account).await;
    // assert_eq!(user_balance, Some(Amount::from_tokens(10)));

    let user_balance = fungible::query_account(foo_token_id, &user_chain, user_account).await;
    assert_eq!(user_balance, None);

    // check app native balance
    let app_native_balance = stake_chain.owner_balance(&lst_id.application_description_hash.into()).await;
    assert_eq!(app_native_balance, Some(Amount::from_tokens(10)));

    // receive msg on user chain
    user_chain
        .add_block(|block| {
            block.with_messages_from(&stake_msg);
        })
        .await;

    // check balances after send msg to user chain
    let user_balance = fungible::query_account(foo_token_id, &stake_chain, user_account).await;
    assert_eq!(user_balance, None);

    let user_balance = fungible::query_account(foo_token_id, &user_chain, user_account).await;
    assert_eq!(user_balance, Some(Amount::from_tokens(10)));

    // check app native balance
    let app_native_balance = stake_chain.owner_balance(&lst_id.application_description_hash.into()).await;
    assert_eq!(app_native_balance, Some(Amount::from_tokens(10)));

    // stake "FOO" lst to get protocol lst
    let stake_cert = user_chain
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::StakeLst {
                    user: user_account,
                    amount: Amount::from_tokens(10),
                    lst_type_in: foo_token_id.forget_abi(),
                },
            );
        })
        .await;

    // receive msg on stake chain (send tokens)
    let stake_msg = stake_chain
        .add_block(|block| {
            block.with_messages_from(&stake_cert);
        })
        .await;
    let user_balance = fungible::query_account(protocol_lst_id, &stake_chain, user_account).await;
    assert_eq!(user_balance, None);

    user_chain
        .add_block(|block| {
            block.with_messages_from(&stake_msg);
        })
        .await;

    // check user protocol lst balance
    let user_balance = fungible::query_account(protocol_lst_id, &user_chain, user_account).await;
    assert_eq!(user_balance, Some(Amount::from_tokens(10)));

    let user_balance = fungible::query_account(protocol_lst_id, &stake_chain, user_account).await;
    assert_eq!(user_balance, None);

    // check app native balance
    let app_native_balance = stake_chain.owner_balance(&lst_id.application_description_hash.into()).await;
    assert_eq!(app_native_balance, Some(Amount::from_tokens(10)));
}

/////////// Swap scenario ///////////
/// 1. create user chain and fund it with native tokens
/// 2. create protocol liquid token
/// 3. create lst app with liquid token as parameter
/// 4. transfer protocol lst to lst app vault
/// 5. create new lst token "FOO"
/// 6. transfer all "FOO" lst to stake chain
/// 7. create new lst token "BAR"
/// 8. transfer all "BAR" lst to stake chain
/// 9. use Operation::NewLst to add new  "FOO" lst
/// 9. use Operation::NewLst to add new  "BAR" lst
/// 10. stake native to get "FOO" lst
/// 11. stake native to get "BAR" lst
/// 12. stake "FOO" for protocol lst, we need some "FOO" in the pool
/// 13. swap "BAR" for "FOO"
/// 14. check balances
#[test_log::test(tokio::test)]
async fn swap() {
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

    // create new stake token "FOO"
    let foo_token_module_id = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    // create initial state for "FOO" token
    let foo_initial_amount = Amount::from_tokens(100);
    let initial_foo_state = fungible::InitialStateBuilder::default().with_account(admin_account, foo_initial_amount);
    let foo_token_params = fungible::Parameters::new("FOO");
    let foo_token_id = stake_chain.create_application(foo_token_module_id, foo_token_params, initial_foo_state.build(), vec![]).await;

    // check if admin has "FOO" token
    let admin_foo_balance = fungible::query_account(foo_token_id, &stake_chain, admin_account).await;
    assert_eq!(admin_foo_balance, Some(foo_initial_amount));

    //transfer all "FOO" lst to stake chain
    stake_chain
        .add_block(|block| {
            block.with_operation(
                foo_token_id,
                fungible::Operation::Transfer {
                    owner: admin_account,
                    amount: foo_initial_amount,
                    target_account: lst_app_vault,
                },
            );
        })
        .await;

    // create new stake token "BAR"
    let bar_token_module_id = stake_chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    // create initial state for "BAR" token
    let bar_initial_amount = Amount::from_tokens(100);
    let initial_bar_state = fungible::InitialStateBuilder::default().with_account(admin_account, bar_initial_amount);
    let bar_token_params = fungible::Parameters::new("BAR");
    let bar_token_id = stake_chain.create_application(bar_token_module_id, bar_token_params, initial_bar_state.build(), vec![]).await;

    // check if admin has "BAR" token
    let admin_bar_balance = fungible::query_account(bar_token_id, &stake_chain, admin_account).await;
    assert_eq!(admin_bar_balance, Some(bar_initial_amount));

    //transfer all "BAR" lst to stake chain
    stake_chain
        .add_block(|block| {
            block.with_operation(
                bar_token_id,
                fungible::Operation::Transfer {
                    owner: admin_account,
                    amount: bar_initial_amount,
                    target_account: lst_app_vault,
                },
            );
        })
        .await;

    // add new FOO lst
    let new_foo_lst_cert = stake_chain
        .add_block(|block| {
            block.with_operation(lst_id, Operation::NewLst { token_id: foo_token_id.forget_abi() });
        })
        .await;

    // add new BAR lst
    let new_bar_lst_cert = stake_chain
        .add_block(|block| {
            block.with_operation(lst_id, Operation::NewLst { token_id: bar_token_id.forget_abi() });
        })
        .await;

    // receive msgs on stake chain
    stake_chain
        .add_block(|block| {
            block.with_messages_from(&new_foo_lst_cert);
            block.with_messages_from(&new_bar_lst_cert);
        })
        .await;

    // stake native token to get "FOO" lst
    let stake_foo_cert = user_chain
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::StakeNative {
                    user: user_account,
                    amount: Amount::from_tokens(10),
                    lst_type_out: foo_token_id.forget_abi(),
                },
            );
        })
        .await;

    // stake native token to get "BAR" lst
    let stake_bar_cert = user_chain
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::StakeNative {
                    user: user_account,
                    amount: Amount::from_tokens(10),
                    lst_type_out: bar_token_id.forget_abi(),
                },
            );
        })
        .await;

    // receive msgs on stake chain
    let stake_foo_msg = stake_chain
        .add_block(|block| {
            block.with_messages_from(&stake_foo_cert);
        })
        .await;

    let stake_bar_msg = stake_chain
        .add_block(|block| {
            block.with_messages_from(&stake_bar_cert);
        })
        .await;

    // receive msgs on user chain
    user_chain
        .add_block(|block| {
            block.with_messages_from(&stake_foo_msg);
            block.with_messages_from(&stake_bar_msg);
        })
        .await;

    // stake "FOO" lst to get protocol lst to create pool
    let stake_foo_for_protocol_cert = user_chain
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::StakeLst {
                    user: user_account,
                    amount: Amount::from_tokens(5),
                    lst_type_in: foo_token_id.forget_abi(),
                },
            );
        })
        .await;

    // receive msg on stake chain
    let stake_foo_for_protocol_msg = stake_chain
        .add_block(|block| {
            block.with_messages_from(&stake_foo_for_protocol_cert);
        })
        .await;

    // receive msg on user chain
    user_chain
        .add_block(|block| {
            block.with_messages_from(&stake_foo_for_protocol_msg);
        })
        .await;

    // swap "BAR" for "FOO"
    let swap_cert = user_chain
        .add_block(|block| {
            block.with_operation(
                lst_id,
                Operation::Swap {
                    user: user_account,
                    amount_in: Amount::from_tokens(5),
                    lst_type_in: bar_token_id.forget_abi(),
                    lst_type_out: foo_token_id.forget_abi(),
                },
            );
        })
        .await;

    // receive msg on stake chain
    let swap_msg = stake_chain
        .add_block(|block| {
            block.with_messages_from(&swap_cert);
        })
        .await;

    // receive msg on user chain
    user_chain
        .add_block(|block| {
            block.with_messages_from(&swap_msg);
        })
        .await;

    // check final balances
    let user_foo_balance = fungible::query_account(foo_token_id, &user_chain, user_account).await;
    let user_bar_balance = fungible::query_account(bar_token_id, &user_chain, user_account).await;
    let user_protocol_balance = fungible::query_account(protocol_lst_id, &user_chain, user_account).await;

    // User should have:
    // - 5 FOO (10 from staking - 5 staked for protocol)
    // - 5 BAR (10 from staking - 5 swapped)
    // - 5 PLST (from staking FOO)
    assert_eq!(user_foo_balance, Some(Amount::from_tokens(5)));
    assert_eq!(user_bar_balance, Some(Amount::from_tokens(5)));
    assert_eq!(user_protocol_balance, Some(Amount::from_tokens(5)));

    // check app balances
    let app_foo_balance = fungible::query_account(foo_token_id, &stake_chain, AccountOwner::from(lst_id)).await;
    let app_bar_balance = fungible::query_account(bar_token_id, &stake_chain, AccountOwner::from(lst_id)).await;
    let app_protocol_balance = fungible::query_account(protocol_lst_id, &stake_chain, AccountOwner::from(lst_id)).await;

    // App should have:
    // - 95 FOO (100 initial - 5 in user's possession)
    // - 95 BAR (100 initial - 5 in user's possession)
    // - 95 PLST (100 initial - 5 in user's possession)
    assert_eq!(app_foo_balance, Some(Amount::from_tokens(95)));
    assert_eq!(app_bar_balance, Some(Amount::from_tokens(95)));
    assert_eq!(app_protocol_balance, Some(Amount::from_tokens(95)));
}
