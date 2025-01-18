mod common;

use common::builder::ContractBuilder;
use common::utils::{
    check_balance, claim_tokens, create_multiple_account, create_subaccount,
    create_subaccount_with_balance, get_not_winners, init_contract, place_order, refund_deposit,
    register_token_account, settle_auction,
};

use chrono::Utc;
use near_sdk::json_types::U128;
use near_workspaces::types::NearToken;

const TOTAL_SUPPLY: u128 = 1_000_000_000_000;

#[tokio::test]
async fn test_calculate_gas_with_one_bidder() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let root = sandbox.root_account()?;

    let alice = create_subaccount(&root, "alice").await?;
    let bob = create_subaccount(&root, "bob").await?;
    let contract_account = create_subaccount_with_balance(&root, "contract").await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = contract_account.deploy(&contract_wasm).await?.unwrap();

    let now = Utc::now().timestamp();
    let init_args = ContractBuilder::new(root.id().to_string())
        .total_supply(TOTAL_SUPPLY)
        .auction_duration(((now + 60) as u64) * 1000000000)
        .min_buy_amount(NearToken::from_near(50).as_yoctonear())
        .build();

    let init_gas = init_contract(&contract, init_args).await?;
    println!("Contract initialization gas with one bidder: {}", init_gas);

    let mut total_gas_burnt = init_gas;

    let register_result = register_token_account(&alice, &contract).await?;
    total_gas_burnt = total_gas_burnt.saturating_add(register_result.total_gas_burnt);
    println!(
        "Register account to token gas: {}",
        register_result.total_gas_burnt
    );

    let order_result =
        place_order(&alice, &contract, TOTAL_SUPPLY, NearToken::from_near(51)).await?;
    total_gas_burnt = total_gas_burnt.saturating_add(order_result.total_gas_burnt);
    println!("Place an order gas: {}", order_result.total_gas_burnt);

    sandbox.fast_forward(2000).await?;

    let settle_result = settle_auction(&contract).await?;
    total_gas_burnt = total_gas_burnt.saturating_add(settle_result.total_gas_burnt);
    println!("Settle auction gas: {}", settle_result.total_gas_burnt);

    let alice_claim = claim_tokens(&alice, &contract).await?.unwrap();
    total_gas_burnt = total_gas_burnt.saturating_add(alice_claim.total_gas_burnt);
    println!("Account claim gas: {}", alice_claim.total_gas_burnt);

    println!(
        "Total gas burnt for entire auction process: {}",
        total_gas_burnt
    );

    let alice_balance: U128 = check_balance(&alice, &contract).await?;
    assert_eq!(alice_balance.0, TOTAL_SUPPLY);

    let bob_balance = check_balance(&bob, &contract).await?;
    assert_eq!(bob_balance.0, 0);

    Ok(())
}

#[tokio::test]
async fn test_calculate_gas_with_ten_bidders() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let root = sandbox.root_account()?;

    let num_accounts = 10;
    let bidder_accounts = create_multiple_account(&root, num_accounts).await?;

    let contract_account = create_subaccount_with_balance(&root, "contract").await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = contract_account.deploy(&contract_wasm).await?.unwrap();

    let now = Utc::now().timestamp();
    let init_args = ContractBuilder::new(root.id().to_string())
        .total_supply(TOTAL_SUPPLY)
        .auction_duration(((now + 60) as u64) * 1000000000)
        .min_buy_amount(NearToken::from_near(50).as_yoctonear())
        .build();

    let init_gas = init_contract(&contract, init_args).await?;
    println!("Contract initialization gas: {}", init_gas);

    let mut total_gas_burnt = init_gas;

    for account in bidder_accounts.iter().take(num_accounts) {
        let register_result = register_token_account(account, &contract).await?;
        total_gas_burnt = total_gas_burnt.saturating_add(register_result.total_gas_burnt);
    }
    println!(
        "Gas after bidders register with ten bidders: {}",
        total_gas_burnt
    );

    for account in bidder_accounts.iter().take(num_accounts) {
        let order_result = place_order(
            account,
            &contract,
            TOTAL_SUPPLY / num_accounts as u128,
            NearToken::from_near(51),
        )
        .await?;
        total_gas_burnt = total_gas_burnt.saturating_add(order_result.total_gas_burnt);
    }
    println!(
        "Gas after bidders place an order with ten bidders: {}",
        total_gas_burnt
    );

    sandbox.fast_forward(2000).await?;

    let settle_result = settle_auction(&contract).await?;
    total_gas_burnt = total_gas_burnt.saturating_add(settle_result.total_gas_burnt);
    println!(
        "Settle auction gas with ten bidders: {}",
        settle_result.total_gas_burnt
    );
    println!(
        "Gas after settle auction with ten bidders: {}",
        total_gas_burnt
    );

    for account in bidder_accounts.iter().take(num_accounts) {
        let bidder_claim = claim_tokens(account, &contract).await?.unwrap();
        total_gas_burnt = total_gas_burnt.saturating_add(bidder_claim.total_gas_burnt);
    }
    println!(
        "Gas after claim tokens with ten bidders: {}",
        total_gas_burnt
    );

    let not_winners = get_not_winners(&bidder_accounts, &contract).await?;

    for not_winner in &not_winners {
        let bidders_refund = refund_deposit(not_winner, &contract).await?.unwrap();
        total_gas_burnt = total_gas_burnt.saturating_add(bidders_refund.total_gas_burnt);
    }

    println!(
        "Total gas burnt for entire auction process: {}",
        total_gas_burnt
    );

    let mut total_balance_in_accounts = 0u128;
    for account in bidder_accounts.iter().take(num_accounts) {
        let bidder_balance = check_balance(account, &contract).await?;
        total_balance_in_accounts += bidder_balance.0
    }

    assert_eq!(total_balance_in_accounts, TOTAL_SUPPLY);

    Ok(())
}

#[tokio::test]
async fn test_calculate_gas_with_hundred_bidders() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let root = sandbox.root_account()?;

    let num_accounts = 100;
    let bidder_accounts = create_multiple_account(&root, num_accounts).await?;

    let contract_account = create_subaccount_with_balance(&root, "contract").await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = contract_account.deploy(&contract_wasm).await?.unwrap();

    let now = Utc::now().timestamp();
    let init_args = ContractBuilder::new(root.id().to_string())
        .total_supply(TOTAL_SUPPLY)
        .auction_duration(((now + 600) as u64) * 1000000000)
        .min_buy_amount(NearToken::from_near(50).as_yoctonear())
        .build();

    let init_gas = init_contract(&contract, init_args).await?;
    println!(
        "Contract initialization gas with a hundred bidders: {}",
        init_gas
    );

    let mut total_gas_burnt = init_gas;

    for account in bidder_accounts.iter().take(num_accounts) {
        let register_result = register_token_account(account, &contract).await?;
        total_gas_burnt = total_gas_burnt.saturating_add(register_result.total_gas_burnt);
    }
    println!("Gas after a hundred bidders register: {}", total_gas_burnt);

    for account in bidder_accounts.iter().take(num_accounts) {
        let order_result = place_order(
            account,
            &contract,
            TOTAL_SUPPLY / num_accounts as u128,
            NearToken::from_near(51),
        )
        .await?;
        total_gas_burnt = total_gas_burnt.saturating_add(order_result.total_gas_burnt);
    }
    println!(
        "Gas after a hundred bidders place an order: {}",
        total_gas_burnt
    );

    sandbox.fast_forward(20000).await?;

    let settle_result = settle_auction(&contract).await?;
    total_gas_burnt = total_gas_burnt.saturating_add(settle_result.total_gas_burnt);
    println!("Settle auction gas: {}", settle_result.total_gas_burnt);
    println!("Gas after settle auction: {}", total_gas_burnt);

    for account in bidder_accounts.iter().take(num_accounts) {
        let bidder_claim = claim_tokens(account, &contract).await?.unwrap();
        total_gas_burnt = total_gas_burnt.saturating_add(bidder_claim.total_gas_burnt);
    }
    println!(
        "Gas after a hundred bidders claim tokens: {}",
        total_gas_burnt
    );

    let not_winners = get_not_winners(&bidder_accounts, &contract).await?;

    for not_winner in &not_winners {
        let bidders_refund = refund_deposit(not_winner, &contract).await?.unwrap();
        total_gas_burnt = total_gas_burnt.saturating_add(bidders_refund.total_gas_burnt);
    }

    println!(
        "Total gas burnt for entire auction process with a hundred bidders: {}",
        total_gas_burnt
    );

    let mut total_balance_in_accounts = 0u128;

    for account in bidder_accounts.iter().take(num_accounts) {
        let bidder_balance = check_balance(account, &contract).await?;
        total_balance_in_accounts += bidder_balance.0;
    }

    assert_eq!(total_balance_in_accounts, TOTAL_SUPPLY);

    Ok(())
}
