use chrono::Utc;
use near_workspaces::types::NearToken;

mod common;

use common::builder::ContractBuilder;
use common::utils::{create_subaccount, init_contract, place_order, register_token_account, Order};

#[tokio::test]
async fn test_place_one_order() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let root = sandbox.root_account()?;

    let alice = create_subaccount(&root, "alice").await?;
    let contract_account = create_subaccount(&root, "contract").await?;

    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = contract_account.deploy(&contract_wasm).await?.unwrap();

    let now = Utc::now().timestamp();
    let init_args = ContractBuilder::new(root.id().to_string())
        .auction_duration(((now + 60) as u64) * 1000000000)
        .build();
    init_contract(&contract, init_args).await?;

    let _ = register_token_account(&alice, &contract).await?;

    let alice_order = place_order(&alice, &contract, 1, NearToken::from_near(1)).await?;
    assert!(alice_order.is_success());

    let orders = contract.call("get_orders").transact().await?;
    let orders_json = orders.json::<Vec<Order>>()?;

    assert_eq!(orders_json.len(), 1);
    assert_eq!(orders_json[0].bidder, "alice.test.near");
    assert_eq!(orders_json[0].buy_amount, "1");
    assert_eq!(orders_json[0].sell_amount, "1000000000000000000000000");

    Ok(())
}

#[tokio::test]
async fn test_place_multiple_orders() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let root = sandbox.root_account()?;

    let alice = create_subaccount(&root, "alice").await?;
    let bob = create_subaccount(&root, "bob").await?;
    let contract_account = create_subaccount(&root, "contract").await?;

    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = contract_account.deploy(&contract_wasm).await?.unwrap();

    let now = Utc::now().timestamp();
    let init_args = ContractBuilder::new(root.id().to_string())
        .total_supply(100)
        .auction_duration(((now + 60) as u64) * 1000000000)
        .min_buy_amount(NearToken::from_near(50).as_yoctonear())
        .build();
    init_contract(&contract, init_args).await?;

    let _ = register_token_account(&alice, &contract).await?;
    let _ = register_token_account(&bob, &contract).await?;

    let alice_order = place_order(&alice, &contract, 1, NearToken::from_near(1)).await?;
    let bob_order = place_order(&bob, &contract, 2, NearToken::from_near(5)).await?;

    assert!(alice_order.is_success());
    assert!(bob_order.is_success());

    let orders = contract.call("get_orders").transact().await?;
    let orders_json = orders.json::<Vec<Order>>()?;

    assert_eq!(orders_json.len(), 2);
    assert_eq!(orders_json[0].bidder, "alice.test.near");
    assert_eq!(orders_json[0].buy_amount, "1");
    assert_eq!(orders_json[0].sell_amount, "1000000000000000000000000");

    assert_eq!(orders_json[1].bidder, "bob.test.near");
    assert_eq!(orders_json[1].buy_amount, "2");
    assert_eq!(orders_json[1].sell_amount, "5000000000000000000000000");

    Ok(())
}
