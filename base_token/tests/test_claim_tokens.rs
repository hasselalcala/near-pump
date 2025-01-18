use chrono::Utc;
use near_workspaces::types::NearToken;

mod common;

use common::builder::ContractBuilder;
use common::utils::{
    check_balance, claim_tokens, create_subaccount, init_contract, place_order, refund_deposit,
    register_token_account, settle_auction,
};

#[tokio::test]
async fn test_claim_and_refund() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let root = sandbox.root_account()?;

    let alice = create_subaccount(&root, "alice").await?;
    let bob = create_subaccount(&root, "bob").await?;
    let charlie = create_subaccount(&root, "charlie").await?;
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
    let _ = register_token_account(&charlie, &contract).await?;

    let _ = place_order(&alice, &contract, 60, NearToken::from_near(70)).await?;
    let _ = place_order(&bob, &contract, 50, NearToken::from_near(55)).await?;
    let _ = place_order(&charlie, &contract, 10, NearToken::from_near(10)).await?;

    sandbox.fast_forward(2000).await?;

    let _ = settle_auction(&contract).await?;

    let alice_claim = claim_tokens(&alice, &contract).await?;
    assert!(alice_claim.is_success());

    let bob_claim = claim_tokens(&bob, &contract).await?;
    assert!(bob_claim.is_success());

    // Try to claim for non-winner (should fail)
    let charlie_claim = claim_tokens(&charlie, &contract).await?;
    assert!(charlie_claim.is_failure());

    let charlie_refund = refund_deposit(&charlie, &contract).await?;
    assert!(charlie_refund.is_success());

    let alice_balance = check_balance(&alice, &contract).await?;
    assert_eq!(alice_balance.0, 60);

    let bob_balance = check_balance(&bob, &contract).await?;
    assert_eq!(bob_balance.0, 40);

    Ok(())
}
