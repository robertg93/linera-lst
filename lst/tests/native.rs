#![cfg(not(target_arch = "wasm32"))]

use linera_sdk::{
    linera_base_types::{Account, AccountOwner, Amount, ChainId, CryptoHash},
    test::{Recipient, TestValidator},
};

#[test_log::test(tokio::test)]
async fn transfer_to_owner() {
    let validator = TestValidator::new().await;

    let mut chain = validator.new_chain().await;
    let admin_account = AccountOwner::from(chain.public_key());

    let fungible_module_id_a = chain
        .publish_bytecode_files_in::<fungible::FungibleTokenAbi, fungible::Parameters, fungible::InitialState>("../fungible")
        .await;

    // let initial_state_a = fungible::InitialStateBuilder::default().with_account(admin_account, Amount::from_tokens(100));
    // let parameters = fungible::Parameters { ticker_symbol: "NAT".to_owned() };
    // chain.create_application(fungible_module_id_a, parameters, initial_state_a.build(), vec![]).await;

    let transfer_amount = Amount::from_tokens(20000);
    let funding_chain = validator.get_chain(&ChainId::root(0));
    let owner = AccountOwner::from(CryptoHash::test_hash("owner"));
    let account = Account::new(chain.id(), owner);
    let recipient = Recipient::Account(account);

    let transfer_certificate = funding_chain
        .add_block(|block| {
            block.with_native_token_transfer(AccountOwner::CHAIN, recipient, transfer_amount);
        })
        .await;

    chain
        .add_block(|block| {
            block.with_messages_from(&transfer_certificate);
        })
        .await;
    let recipient_balance = chain.owner_balance(&owner).await;
    println!("recipient_balance: {:?}", recipient_balance);
    let funding_chain_balance = funding_chain.owner_balance(&AccountOwner::CHAIN).await;
    println!("funding_chain_balance: {:?}", funding_chain_balance);
    assert_eq!(recipient_balance, Some(transfer_amount));
}
