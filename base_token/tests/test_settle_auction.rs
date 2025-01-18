mod common;

use chrono::Utc;
use near_sdk::json_types::U128;
use near_workspaces::types::NearToken;

use common::builder::ContractBuilder;
use common::utils::{
    create_subaccount, create_subaccount_with_balance, init_contract, place_order,
    register_token_account, settle_auction, winner_list, Order,
};

#[tokio::test]
async fn test_simple_settle_auction() -> Result<(), Box<dyn std::error::Error>> {
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
        .auction_duration(((now + 600) as u64) * 1000000000)
        .min_buy_amount(NearToken::from_near(50).as_yoctonear())
        .build();
    init_contract(&contract, init_args).await?;

    let _ = register_token_account(&alice, &contract).await?;
    let _ = register_token_account(&bob, &contract).await?;

    let _ = place_order(&alice, &contract, 1, NearToken::from_near(10)).await?;
    let _ = place_order(&bob, &contract, 2, NearToken::from_near(10)).await?;

    let unsorted_orders = contract.call("get_orders").transact().await?;
    let unsorted_orders_json = unsorted_orders.json::<Vec<Order>>()?;
    assert_eq!(unsorted_orders_json.len(), 2);
    assert_eq!(unsorted_orders_json[0].bidder, "alice.test.near");
    assert_eq!(unsorted_orders_json[1].bidder, "bob.test.near");

    sandbox.fast_forward(2000).await?;

    let settle_auction = settle_auction(&contract).await?;
    assert!(settle_auction.is_success());

    let sorted_orders = contract.call("get_orders").transact().await?;
    let sorted_orders_json = sorted_orders.json::<Vec<Order>>()?;

    assert_eq!(sorted_orders_json.len(), 2);
    assert_eq!(sorted_orders_json[0].bidder, "alice.test.near");
    assert_eq!(sorted_orders_json[1].bidder, "bob.test.near");

    let winners = winner_list(&contract).await?;
    let winners_json = winners.unwrap().json::<Vec<(Order, bool)>>()?;

    assert_eq!(winners_json.len(), 2);
    assert_eq!(winners_json[0].0.bidder, "alice.test.near");
    assert_eq!(winners_json[1].0.bidder, "bob.test.near");

    let final_auction_price = contract.call("get_final_auction_price").transact().await?;
    let final_auction_price_json = final_auction_price.json::<U128>()?;
    assert_eq!(
        final_auction_price_json,
        U128::from(5_000_000_000_000_000_000_000_000)
    );

    Ok(())
}

#[tokio::test]
async fn test_settle_auction_when_user_buy_almost_all_tokens_but_offer_price_is_equal(
) -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let root = sandbox.root_account()?;

    let alice = create_subaccount_with_balance(&root, "alice").await?;
    let bob = create_subaccount_with_balance(&root, "bob").await?;
    let contract_account = create_subaccount_with_balance(&root, "contract").await?;

    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = contract_account.deploy(&contract_wasm).await?.unwrap();

    let now = Utc::now().timestamp();
    let init_args = ContractBuilder::new(root.id().to_string())
        .total_supply(100)
        .auction_duration(((now + 60) as u64) * 1000000000)
        .min_buy_amount(NearToken::from_near(100).as_yoctonear())
        .build();
    init_contract(&contract, init_args).await?;

    let _ = register_token_account(&alice, &contract).await?;
    let _ = register_token_account(&bob, &contract).await?;

    let _ = place_order(&alice, &contract, 90, NearToken::from_near(90)).await?;
    let _ = place_order(&bob, &contract, 20, NearToken::from_near(20)).await?;

    let unsorted_orders = contract.call("get_orders").transact().await?;
    let unsorted_orders_json = unsorted_orders.json::<Vec<Order>>()?;

    assert_eq!(unsorted_orders_json.len(), 2);
    assert_eq!(unsorted_orders_json[0].bidder, "alice.test.near");
    assert_eq!(unsorted_orders_json[1].bidder, "bob.test.near");

    sandbox.fast_forward(2000).await?;

    let _ = settle_auction(&contract).await?;

    let sorted_orders = contract.call("get_orders").transact().await?;
    let sorted_orders_json = sorted_orders.json::<Vec<Order>>()?;

    assert_eq!(sorted_orders_json.len(), 2);
    assert_eq!(sorted_orders_json[0].bidder, "alice.test.near");
    assert_eq!(sorted_orders_json[1].bidder, "bob.test.near");

    let winners = winner_list(&contract).await?;
    let winners_json = winners.unwrap().json::<Vec<(Order, bool)>>()?;

    assert_eq!(winners_json.len(), 2);
    assert_eq!(winners_json[0].0.bidder, "alice.test.near");
    assert_eq!(
        winners_json[0].0.buy_amount.parse::<u128>().unwrap(),
        90_u128
    );
    assert_eq!(
        winners_json[0].0.sell_amount.parse::<u128>().unwrap(),
        90_000_000_000_000_000_000_000_000
    );

    assert_eq!(winners_json[1].0.bidder, "bob.test.near");
    assert_eq!(
        winners_json[1].0.buy_amount.parse::<u128>().unwrap(),
        10_u128
    );
    assert_eq!(
        winners_json[1].0.sell_amount.parse::<u128>().unwrap(),
        10_000_000_000_000_000_000_000_000
    );

    let final_auction_price = contract.call("get_final_auction_price").transact().await?;
    let final_auction_price_json = final_auction_price.json::<U128>()?;
    assert_eq!(
        final_auction_price_json,
        U128::from(1_000_000_000_000_000_000_000_000)
    );

    Ok(())
}

#[tokio::test]
async fn test_settle_auction_when_user_buy_almost_all_tokens_but_offer_price_is_not_equal_and_one_bidder_win_all_tokens(
) -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let root = sandbox.root_account()?;

    let alice = create_subaccount_with_balance(&root, "alice").await?;
    let bob = create_subaccount_with_balance(&root, "bob").await?;
    let contract_account = create_subaccount_with_balance(&root, "contract").await?;

    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = contract_account.deploy(&contract_wasm).await?.unwrap();

    let now = Utc::now().timestamp();
    let init_args = ContractBuilder::new(root.id().to_string())
        .total_supply(10)
        .auction_duration(((now + 600) as u64) * 1000000000)
        .min_buy_amount(NearToken::from_near(100).as_yoctonear())
        .build();
    init_contract(&contract, init_args).await?;

    let _ = register_token_account(&alice, &contract).await?;
    let _ = register_token_account(&bob, &contract).await?;

    let _ = place_order(&alice, &contract, 1, NearToken::from_near(10)).await?;
    let _ = place_order(&bob, &contract, 10, NearToken::from_near(500)).await?;

    let unsorted_orders = contract.call("get_orders").transact().await?;
    let unsorted_orders_json = unsorted_orders.json::<Vec<Order>>()?;

    assert_eq!(unsorted_orders_json.len(), 2);
    assert_eq!(unsorted_orders_json[0].bidder, "alice.test.near");
    assert_eq!(unsorted_orders_json[1].bidder, "bob.test.near");

    sandbox.fast_forward(2000).await?;

    let settle_auction = settle_auction(&contract).await?;
    assert!(settle_auction.is_success());

    let sorted_orders = contract.call("get_orders").transact().await?;
    let sorted_orders_json = sorted_orders.json::<Vec<Order>>()?;

    assert_eq!(sorted_orders_json.len(), 2);
    assert_eq!(sorted_orders_json[0].bidder, "bob.test.near");
    assert_eq!(sorted_orders_json[1].bidder, "alice.test.near");

    let winners = winner_list(&contract).await?;
    let winners_json = winners.unwrap().json::<Vec<(Order, bool)>>()?;

    assert_eq!(winners_json.len(), 1);
    assert_eq!(winners_json[0].0.bidder, "bob.test.near");
    assert_eq!(
        winners_json[0].0.buy_amount.parse::<u128>().unwrap(),
        10_u128
    );
    assert_eq!(
        winners_json[0].0.sell_amount.parse::<u128>().unwrap(),
        500_000_000_000_000_000_000_000_000
    );

    let final_auction_price = contract.call("get_final_auction_price").transact().await?;
    let final_auction_price_json = final_auction_price.json::<U128>()?;
    assert_eq!(
        final_auction_price_json,
        U128::from(50_000_000_000_000_000_000_000_000)
    );

    Ok(())
}

#[tokio::test]
async fn test_settle_auction_when_user_buy_almost_all_tokens_but_offer_price_is_not_equal_and_both_bidders_win_tokens(
) -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let root = sandbox.root_account()?;

    let alice = create_subaccount_with_balance(&root, "alice").await?;
    let bob = create_subaccount_with_balance(&root, "bob").await?;
    let contract_account = create_subaccount_with_balance(&root, "contract").await?;

    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = contract_account.deploy(&contract_wasm).await?.unwrap();

    let now = Utc::now().timestamp();
    let init_args = ContractBuilder::new(root.id().to_string())
        .total_supply(10)
        .auction_duration(((now + 600) as u64) * 1000000000)
        .min_buy_amount(NearToken::from_near(100).as_yoctonear())
        .build();
    init_contract(&contract, init_args).await?;

    let _ = register_token_account(&alice, &contract).await?;
    let _ = register_token_account(&bob, &contract).await?;

    let _ = place_order(&alice, &contract, 1, NearToken::from_near(10)).await?;
    let _ = place_order(&bob, &contract, 8, NearToken::from_near(400)).await?;

    let unsorted_orders = contract.call("get_orders").transact().await?;
    let unsorted_orders_json = unsorted_orders.json::<Vec<Order>>()?;

    assert_eq!(unsorted_orders_json.len(), 2);
    assert_eq!(unsorted_orders_json[0].bidder, "alice.test.near");
    assert_eq!(unsorted_orders_json[1].bidder, "bob.test.near");

    sandbox.fast_forward(2000).await?;

    let settle_auction = settle_auction(&contract).await?;
    assert!(settle_auction.is_success());

    let sorted_orders = contract.call("get_orders").transact().await?;
    let sorted_orders_json = sorted_orders.json::<Vec<Order>>()?;

    assert_eq!(sorted_orders_json.len(), 2);
    assert_eq!(sorted_orders_json[0].bidder, "bob.test.near");
    assert_eq!(sorted_orders_json[1].bidder, "alice.test.near");

    let winners = winner_list(&contract).await?;
    let winners_json = winners.unwrap().json::<Vec<(Order, bool)>>()?;

    assert_eq!(winners_json.len(), 2);
    assert_eq!(winners_json[0].0.bidder, "bob.test.near");
    assert_eq!(
        winners_json[0].0.buy_amount.parse::<u128>().unwrap(),
        8_u128
    );
    assert_eq!(
        winners_json[0].0.sell_amount.parse::<u128>().unwrap(),
        400_000_000_000_000_000_000_000_000
    );

    assert_eq!(winners_json[1].0.bidder, "alice.test.near");
    assert_eq!(
        winners_json[1].0.buy_amount.parse::<u128>().unwrap(),
        1_u128
    );
    assert_eq!(
        winners_json[1].0.sell_amount.parse::<u128>().unwrap(),
        10_000_000_000_000_000_000_000_000
    );

    let final_auction_price = contract.call("get_final_auction_price").transact().await?;
    let final_auction_price_json = final_auction_price.json::<U128>()?;
    assert_eq!(
        final_auction_price_json,
        U128::from(10_000_000_000_000_000_000_000_000)
    );

    Ok(())
}
