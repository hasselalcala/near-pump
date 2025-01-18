use base_token::Contract;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_contract_standards::fungible_token::Balance;
use near_contract_standards::fungible_token::FungibleTokenCore;
use near_contract_standards::storage_management::StorageManagement;
use near_sdk::test_utils::{accounts, VMContextBuilder};
use near_sdk::testing_env;
use near_sdk::{env, AccountId, NearToken, json_types::U64};
use chrono::Utc;

const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;
const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";
const FT_METADATA_SPEC: &str = "ft-1.0.0";

fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .current_account_id(accounts(0))
        .signer_account_id(predecessor_account_id.clone())
        .predecessor_account_id(predecessor_account_id);
    builder
}
fn new_default_meta() -> FungibleTokenMetadata {
    FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "Example NEAR fungible token".to_string(),
        symbol: "EXAMPLE".to_string(),
        icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
        reference: None,
        reference_hash: None,
        decimals: 24,
    }
}

#[test]
fn test_new() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let now = Utc::now().timestamp();
    let contract = Contract::new(
        env::current_account_id(),
        TOTAL_SUPPLY.into(),
        new_default_meta(),
        DATA_IMAGE_SVG_NEAR_ICON.to_string(),
        "New cool token to be aucted".to_string(),
        U64::from((now + 600) as u64 * 1000000000),
        NearToken::from_near(50),
    );

    testing_env!(context.is_view(true).build());
    assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
    assert_eq!(
        contract.ft_balance_of(env::current_account_id()).0,
        TOTAL_SUPPLY
    );
}

#[test]
#[should_panic(expected = "The contract is not initialized")]
fn test_default() {
    let context = get_context(accounts(1));
    testing_env!(context.build());
    let _contract = Contract::default();
}

#[test]
fn test_transfer() {
    let mut context = get_context(accounts(2));
    testing_env!(context.build());
    let now = Utc::now().timestamp();
    let mut contract = Contract::new(
        env::current_account_id(),
        TOTAL_SUPPLY.into(),
        new_default_meta(),
        DATA_IMAGE_SVG_NEAR_ICON.to_string(),
        "New cool token to be aucted".to_string(),
        U64::from((now + 600) as u64 * 1000000000),
        NearToken::from_near(50),
    );
    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(contract.storage_balance_bounds().min)
        .predecessor_account_id(accounts(1))
        .build());

    // Paying for account registration, aka storage deposit
    contract.storage_deposit(None, None);

    testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(NearToken::from_yoctonear(1))
        .predecessor_account_id(env::current_account_id())
        .build());
    let transfer_amount = TOTAL_SUPPLY / 3;
    contract.ft_transfer(accounts(1), transfer_amount.into(), None);

    testing_env!(context
        .storage_usage(env::storage_usage())
        .account_balance(env::account_balance())
        .is_view(true)
        .attached_deposit(NearToken::from_near(0))
        .build());
    assert_eq!(
        contract.ft_balance_of(env::current_account_id()).0,
        (TOTAL_SUPPLY - transfer_amount)
    );
    assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
}
