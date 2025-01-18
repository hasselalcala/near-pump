use near_sdk::json_types::U128;
use near_sdk::Gas;
use near_workspaces::types::NearToken;
use serde::Deserialize;

pub const HUNDRED_NEAR: NearToken = NearToken::from_near(100);

#[allow(dead_code)]
pub const THOUSAND_NEAR: NearToken = NearToken::from_near(1000);

#[derive(Debug, Deserialize)]
pub struct Order {
    pub bidder: String,

    #[allow(dead_code)]
    pub buy_amount: String,

    #[allow(dead_code)]
    pub sell_amount: String,
}

pub async fn create_subaccount(
    root: &near_workspaces::Account,
    name: &str,
) -> Result<near_workspaces::Account, Box<dyn std::error::Error>> {
    let subaccount = root
        .create_subaccount(name)
        .initial_balance(HUNDRED_NEAR)
        .transact()
        .await?
        .unwrap();

    Ok(subaccount)
}

#[allow(dead_code)]
pub async fn create_multiple_account(
    root: &near_workspaces::Account,
    num_accounts: usize,
) -> Result<Vec<near_workspaces::Account>, Box<dyn std::error::Error>> {
    let mut accounts = Vec::new();

    for i in 1..=num_accounts {
        let account_name = format!("account{}", i);
        let account = create_subaccount(root, &account_name).await?;
        accounts.push(account);
    }

    Ok(accounts)
}

#[allow(dead_code)]
pub async fn create_subaccount_with_balance(
    root: &near_workspaces::Account,
    name: &str,
) -> Result<near_workspaces::Account, Box<dyn std::error::Error>> {
    let subaccount = root
        .create_subaccount(name)
        .initial_balance(THOUSAND_NEAR)
        .transact()
        .await?
        .unwrap();

    Ok(subaccount)
}

pub async fn init_contract(
    contract: &near_workspaces::Contract,
    init_args: serde_json::Value,
) -> Result<Gas, Box<dyn std::error::Error>> {
    let init = contract.call("new").args_json(init_args).transact().await?;
    assert!(init.is_success());
    Ok(init.total_gas_burnt)
}
pub async fn place_order(
    account: &near_workspaces::Account,
    contract: &near_workspaces::Contract,
    buy_amount: u128,
    deposit: NearToken,
) -> Result<near_workspaces::result::ExecutionFinalResult, Box<dyn std::error::Error>> {
    let order_args = serde_json::json!({
        "buy_amount": U128::from(buy_amount)
    });

    let order = account
        .call(contract.id(), "place_order")
        .args_json(order_args)
        .deposit(deposit)
        .transact()
        .await?;

    assert!(order.is_success());

    Ok(order)
}

#[allow(dead_code)]
pub async fn settle_auction(
    contract: &near_workspaces::Contract,
) -> Result<near_workspaces::result::ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = contract.call("settle_auction").max_gas().transact().await?;
    assert!(result.is_success());
    Ok(result)
}

#[allow(dead_code)]
pub async fn winner_list(
    contract: &near_workspaces::Contract,
) -> Result<near_workspaces::result::ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = contract.call("get_auction_winner").transact().await?;
    assert!(result.is_success());
    Ok(result)
}

#[allow(dead_code)]
pub async fn get_not_winners(
    vec_account: &[near_workspaces::Account],
    contract: &near_workspaces::Contract,
) -> Result<Vec<near_workspaces::Account>, Box<dyn std::error::Error>> {
    let winners = winner_list(contract).await?;
    let winners_json = winners.json::<Vec<(Order, bool)>>()?;
    let winner_accounts: Vec<near_workspaces::AccountId> = winners_json
        .into_iter()
        .map(|(order, _)| order.bidder.parse().unwrap())
        .collect();

    let not_winners: Vec<near_workspaces::Account> = vec_account
        .iter()
        .filter(|account| !winner_accounts.contains(account.id()))
        .cloned()
        .collect();

    Ok(not_winners)
}

#[allow(dead_code)]
pub async fn claim_tokens(
    account: &near_workspaces::Account,
    contract: &near_workspaces::Contract,
) -> Result<near_workspaces::result::ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = account
        .call(contract.id(), "claim_tokens")
        .transact()
        .await?;
    Ok(result)
}

#[allow(dead_code)]
pub async fn refund_deposit(
    account: &near_workspaces::Account,
    contract: &near_workspaces::Contract,
) -> Result<near_workspaces::result::ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = account
        .call(contract.id(), "refund_deposit")
        .transact()
        .await?;
    Ok(result)
}

pub async fn register_token_account(
    account: &near_workspaces::Account,
    contract: &near_workspaces::Contract,
) -> Result<near_workspaces::result::ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = account
        .call(contract.id(), "register_bidder")
        .deposit(NearToken::from_yoctonear(10u128.pow(24))) // 1 NEAR
        .transact()
        .await?;
    assert!(result.is_success());

    Ok(result)
}

#[allow(dead_code)]
pub async fn check_balance(
    account: &near_workspaces::Account,
    contract: &near_workspaces::Contract,
) -> Result<U128, Box<dyn std::error::Error>> {
    let balance: U128 = contract
        .call("ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": account.id()
        }))
        .transact()
        .await?
        .json()?;

    Ok(balance)
}
