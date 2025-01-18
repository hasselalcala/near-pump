use chrono::Utc;
use common::utils::new_default_token_args;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::test_utils::{accounts, VMContextBuilder};
use near_sdk::{json_types::U128, json_types::U64, testing_env, AccountId, NearToken};
use token_factory::{TokenArgs, TokenFactory};

pub mod common;

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";
const FT_METADATA_SPEC: &str = "ft-1.0.0";

fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .current_account_id(accounts(0))
        .signer_account_id(predecessor_account_id.clone())
        .predecessor_account_id(predecessor_account_id)
        .attached_deposit(NearToken::from_near(1000));
    builder
}

#[test]
fn test_new() {
    let context = get_context(accounts(1));
    testing_env!(context.build());

    let contract = TokenFactory::new();
    assert_eq!(contract.tokens.len(), 0);
    assert_eq!(contract.storage_deposits.get(&accounts(1)), None);
}

#[test]
fn test_storage_deposit() {
    let context = get_context(accounts(0));
    testing_env!(context.build());

    let mut contract = TokenFactory::new();

    let previous_balance = contract.storage_deposits.get(&accounts(0));
    assert_eq!(previous_balance, None);

    contract.storage_deposit();
    let new_balance = contract.storage_deposits.get(&accounts(0));

    assert!(new_balance > previous_balance);
}

#[test]
fn test_create_token_and_get_token() {
    let context = get_context(accounts(0));
    testing_env!(context.build());

    let mut contract = TokenFactory::new();
    let token_args = new_default_token_args(
        &accounts(0),
        U128::from(1000000000),
        DATA_IMAGE_SVG_NEAR_ICON.to_string(),
        "This is a test token".to_string(),
    );

    contract.create_token(
        token_args.metadata.spec.clone(),
        token_args.metadata.name.clone(),
        token_args.metadata.symbol.clone(),
        None,
        None,
        None,
        token_args.metadata.decimals,
        token_args.image.clone(),
        token_args.description.clone(),
        token_args.auction_duration,
        token_args.min_buy_amount,
    );

    let (owner, total_supply, metadata, image) =
        contract.get_token(token_args.metadata.symbol).unwrap();
    assert_eq!(owner, token_args.owner_id);
    assert_eq!(total_supply, token_args.total_supply);
    assert_eq!(metadata.name, token_args.metadata.name);
    assert_eq!(image, token_args.image);
}

#[test]
fn test_get_number_of_tokens() {
    let context = get_context(accounts(0));
    testing_env!(context.build());

    let mut contract = TokenFactory::new();
    let token_args = new_default_token_args(
        &accounts(0),
        U128::from(1000000000),
        DATA_IMAGE_SVG_NEAR_ICON.to_string(),
        "This is a test token".to_string(),
    );

    contract.create_token(
        token_args.metadata.spec.clone(),
        token_args.metadata.name.clone(),
        token_args.metadata.symbol.clone(),
        None,
        None,
        None,
        token_args.metadata.decimals,
        token_args.image,
        token_args.description,
        token_args.auction_duration,
        token_args.min_buy_amount,
    );

    let context = get_context(accounts(1));
    testing_env!(context.build());

    let now = Utc::now().timestamp();
    let token_args_2 = TokenArgs {
        owner_id: accounts(1),
        total_supply: U128::from(100),
        metadata: FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "Test Token 2".to_string(),
            symbol: "TT2".to_string(),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: 24,
        },
        image: DATA_IMAGE_SVG_NEAR_ICON.to_string(),
        description: "This is a test token".to_string(),
        auction_duration: U64::from((now + 600) as u64 * 1000000000),
        min_buy_amount: NearToken::from_near(50),
    };

    contract.create_token(
        token_args_2.metadata.spec.clone(),
        token_args_2.metadata.name.clone(),
        token_args_2.metadata.symbol.clone(),
        None,
        None,
        None,
        token_args_2.metadata.decimals,
        token_args_2.image,
        token_args_2.description,
        token_args.auction_duration,
        token_args.min_buy_amount,
    );

    let result = contract.get_number_of_tokens();
    assert_eq!(result, 2);
}
