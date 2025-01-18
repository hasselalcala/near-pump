use anyhow::Ok;
use chrono::Utc;
use common::utils::new_default_token_args;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{json_types::U128, json_types::U64, AccountId, NearToken as SdkNearToken};
use near_workspaces::types::NearToken as WorkspacesNearToken;
use serde_json::json;
use token_factory::TokenArgs;

pub mod common;

const TOKEN_FACTORY_FILEPATH : &str = "/Users/hasselalcala/Documents/near_contracts/near-pump/token_factory/target/near/token_factory.wasm";
const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/icon";
const FT_METADATA_SPEC: &str = "ft-1.0.0";

#[tokio::test]
async fn test_create_one_token() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(TOKEN_FACTORY_FILEPATH)?;

    //@dev Deploy contract
    let contract = worker.dev_deploy(&wasm).await?;

    let outcome_new = contract.call("new").max_gas().transact().await?;
    assert!(outcome_new.is_success());

    //@dev Register a token
    let token_account = worker.dev_create_account().await?;
    let owner_id = token_account.id();

    //@dev Transfer funds from root to owner, to be capable to create contract
    let root_account = worker.root_account()?;
    let amount =
        WorkspacesNearToken::from_yoctonear(SdkNearToken::as_yoctonear(&SdkNearToken::from_near(500)));
    let _ = root_account.transfer_near(owner_id, amount).await?;

    let args = new_default_token_args(
        owner_id,
        U128::from(1_000_000_000),
        DATA_IMAGE_SVG_NEAR_ICON.to_string(),
        "This is a test token".to_string(),
    );

    //@dev Quantity needed to create token and pay for storage cost
    let deposit =
        WorkspacesNearToken::from_yoctonear(SdkNearToken::as_yoctonear(&SdkNearToken::from_near(300)));

    //@dev Create token
    let outcome_create_token = token_account
        .call(contract.id(), "create_token")
        .args_json(json!({
            "spec": args.metadata.spec ,
            "name": args.metadata.name ,
            "symbol":args.metadata.symbol ,
            "icon":args.metadata.icon,
            "reference": args.metadata.reference,
            "reference_hash": args.metadata.reference_hash,
            "decimals":args.metadata.decimals,
            "image": args.image,
            "description": args.description,
            "auction_duration" : args.auction_duration,
            "min_buy_amount" : args.min_buy_amount,
        }))
        .deposit(deposit)
        .max_gas()
        .transact()
        .await?;

    assert!(outcome_create_token.is_success());

    let expected = format!(
        r#"EVENT_JSON:{{"standard":"nep141","version":"1.0.0","event":"register_token","data":[{{"owner_id":"{}","total_supply":"{:?}","spec":"ft-1.0.0","name":"{}","symbol":"{}","icon":{:?},"reference":null,"reference_hash":null,"decimals":24,"image":"{}","description":"{}","auction_duration":"{}","min_buy_amount":"50000000000000000000000000"}}]}}"#,
        token_account.id(),
        U128::from(1_000_000_000).0,
        args.metadata.name,
        args.metadata.symbol,
        DATA_IMAGE_SVG_NEAR_ICON,
        args.image,
        args.description,
        args.auction_duration.0,
    );

    let logs = outcome_create_token.logs();

    let register_token_log = logs
        .iter()
        .find(|&log| log.contains("\"event\":\"register_token\""));

    if let Some(register_token_log) = register_token_log {
        println!("Register token log: {}", register_token_log);
        assert_eq!(expected, register_token_log.to_string());
    } else {
        println!("log not found");
    }

    //@dev Get number of registered tokens
    let outcome_get_number_of_tokens = token_account
        .call(contract.id(), "get_number_of_tokens")
        .transact()
        .await?;

    println!(
        "get_numer_of_tokens outcome: {:?}",
        outcome_get_number_of_tokens
    );

    //@dev deserialize result
    let result_value = outcome_get_number_of_tokens.json::<u64>()?;
    println!("Number of registered tokens: {}", result_value);
    assert_eq!(result_value, 1);

    //@dev Get token
    let token_id = args.metadata.symbol.to_ascii_lowercase();

    let outcome_get_token = token_account
        .call(contract.id(), "get_token")
        .args_json(json!({"token_id": token_id}))
        .transact()
        .await?;

    println!("get_token outcome: {:#?}", outcome_get_token);

    let (account_id, total_supply, metadata, image) =
        outcome_get_token.json::<(AccountId, U128, FungibleTokenMetadata, String)>()?;

    assert_eq!(account_id, *token_account.id());
    assert_eq!(total_supply, U128::from(1_000_000_000));
    assert_eq!(metadata.name, "Example NEAR fungible token");
    assert_eq!(image, DATA_IMAGE_SVG_NEAR_ICON);

    Ok(())
}

#[tokio::test]
async fn test_create_multiple_tokens() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(TOKEN_FACTORY_FILEPATH)?;

    //@dev Deploy contract
    let contract = worker.dev_deploy(&wasm).await?;

    let outcome_new = contract.call("new").max_gas().transact().await?;

    println!("new_contract outcome: {:#?}", outcome_new);
    assert!(outcome_new.is_success());

    //@dev Register a token
    let token_account = worker.dev_create_account().await?;
    let owner_id = token_account.id();

    //@dev Transfer funds from root to owner, to be capable to create contract
    let root_account = worker.root_account()?;
    let amount =
        WorkspacesNearToken::from_yoctonear(SdkNearToken::as_yoctonear(&SdkNearToken::from_near(500)));
    let _ = root_account.transfer_near(owner_id, amount).await?;

    let args = new_default_token_args(
        owner_id,
        U128::from(1_000_000_000),
        DATA_IMAGE_SVG_NEAR_ICON.to_string(),
        "This is a test token".to_string(),
    );

    //@dev Quantity needed to create token and pay for storage cost
    let deposit =
        WorkspacesNearToken::from_yoctonear(SdkNearToken::as_yoctonear(&SdkNearToken::from_near(300)));

    //@dev Create token
    let outcome_create_token = token_account
        .call(contract.id(), "create_token")
        .args_json(json!({"owner_id": args.owner_id,
            "total_supply":args.total_supply,
            "spec": args.metadata.spec ,
            "name": args.metadata.name ,
            "symbol":args.metadata.symbol ,
            "icon":args.metadata.icon,
            "reference": args.metadata.reference,
            "reference_hash": args.metadata.reference_hash,
            "decimals":args.metadata.decimals,
            "image": args.image,
            "description": args.description,
            "auction_duration" : args.auction_duration,
            "min_buy_amount" : args.min_buy_amount,
        }))
        .deposit(deposit)
        .max_gas()
        .transact()
        .await?;

    println!("create_token outcome: {:#?}", outcome_create_token);

    let expected = format!(
        r#"EVENT_JSON:{{"standard":"nep141","version":"1.0.0","event":"register_token","data":[{{"owner_id":"{}","total_supply":"{:?}","spec":"ft-1.0.0","name":"{}","symbol":"{}","icon":{:?},"reference":null,"reference_hash":null,"decimals":24,"image":"{}","description":"{}","auction_duration":"{}","min_buy_amount":"50000000000000000000000000"}}]}}"#,
        token_account.id(),
        U128::from(1_000_000_000).0,
        args.metadata.name,
        args.metadata.symbol,
        DATA_IMAGE_SVG_NEAR_ICON,
        args.image,
        args.description,
        args.auction_duration.0,
    );

    let logs = outcome_create_token.logs();

    // Buscar el log que contiene el evento "register_token"
    let register_token_log = logs
        .iter()
        .find(|&log| log.contains("\"event\":\"register_token\""));

    if let Some(register_token_log) = register_token_log {
        println!("Register token log: {}", register_token_log);
        assert_eq!(expected, register_token_log.to_string());
    } else {
        println!("log not found");
    }

    //@dev Get number of registered tokens
    let outcome_get_number_of_tokens = token_account
        .call(contract.id(), "get_number_of_tokens")
        .transact()
        .await?;

    println!(
        "get_numer_of_tokens outcome: {:?}",
        outcome_get_number_of_tokens
    );

    //@dev deserialize result
    let result_value = outcome_get_number_of_tokens.json::<u64>()?;
    println!("Number of registered tokens: {}", result_value);
    assert_eq!(result_value, 1);

    //@dev Get token
    let token_id = args.metadata.symbol.to_ascii_lowercase();

    let outcome_get_token = token_account
        .call(contract.id(), "get_token")
        .args_json(json!({"token_id": token_id}))
        .transact()
        .await?;

    println!("get_token outcome: {:#?}", outcome_get_token);

    let (account_id, total_supply, metadata, image) =
        outcome_get_token.json::<(AccountId, U128, FungibleTokenMetadata, String)>()?;

    assert_eq!(account_id, *token_account.id());
    assert_eq!(total_supply, U128::from(1_000_000_000));
    assert_eq!(metadata.name, "Example NEAR fungible token");
    assert_eq!(image, DATA_IMAGE_SVG_NEAR_ICON);

    //@dev Create a second token
    let token_account_2 = worker.dev_create_account().await?;
    let owner_id_2 = token_account_2.id();

    //@ Transfer funds
    let _ = root_account.transfer_near(owner_id_2, amount).await?;

    //@dev arguments to create a token
    let now = Utc::now().timestamp();
    let args = TokenArgs {
        owner_id: owner_id_2.clone(),
        total_supply,
        metadata: FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "MemeCoin".to_string(),
            symbol: "MC".to_string(),
            icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
            reference: None,
            reference_hash: None,
            decimals: 24,
        },
        image: DATA_IMAGE_SVG_NEAR_ICON.to_string(),
        description: "This is a test token".to_string(),
        auction_duration: U64::from((now + 600) as u64 * 1000000000),
        min_buy_amount: SdkNearToken::from_near(50),
    };

    let outcome_create_token_2 = token_account_2
        .call(contract.id(), "create_token")
        .args_json(json!({"owner_id": args.owner_id,
            "total_supply":args.total_supply,
            "spec": args.metadata.spec ,
            "name": args.metadata.name ,
            "symbol":args.metadata.symbol ,
            "icon":args.metadata.icon,
            "reference": args.metadata.reference,
            "reference_hash": args.metadata.reference_hash,
            "decimals":args.metadata.decimals,
            "image": args.image,
            "description": args.description,
            "auction_duration": args.auction_duration,
            "min_buy_amount": args.min_buy_amount
        }))
        .deposit(deposit)
        .max_gas()
        .transact()
        .await?;

    println!("create_token outcome 2: {:#?}", outcome_create_token_2);
    let expected = format!(
        r#"EVENT_JSON:{{"standard":"nep141","version":"1.0.0","event":"register_token","data":[{{"owner_id":"{}","total_supply":"{:?}","spec":"ft-1.0.0","name":"{}","symbol":"{}","icon":{:?},"reference":null,"reference_hash":null,"decimals":24,"image":"{}","description":"{}","auction_duration":"{}","min_buy_amount":"50000000000000000000000000"}}]}}"#,
        token_account_2.id(),
        U128::from(1_000_000_000).0,
        args.metadata.name,
        args.metadata.symbol,
        DATA_IMAGE_SVG_NEAR_ICON,
        args.image,
        args.description,
        args.auction_duration.0,
    );

    let logs = outcome_create_token_2.logs();

    // Buscar el log que contiene el evento "register_token"
    let register_token_log = logs
        .iter()
        .find(|&log| log.contains("\"event\":\"register_token\""));

    if let Some(register_token_log) = register_token_log {
        println!("Register token log: {}", register_token_log);
        assert_eq!(expected, register_token_log.to_string());
    } else {
        println!("Log not found");
    }

    //@dev Get number of registered tokens
    let outcome_get_number_of_tokens = token_account_2
        .call(contract.id(), "get_number_of_tokens")
        .transact()
        .await?;

    println!(
        "get_numer_of_tokens outcome: {:?}",
        outcome_get_number_of_tokens
    );

    //@dev deserialize result
    let result_value = outcome_get_number_of_tokens.json::<u64>()?;
    println!("Number of registered tokens: {}", result_value);
    assert_eq!(result_value, 2);

    //@dev Get token
    let token_id = args.metadata.symbol.to_ascii_lowercase();

    let outcome_get_token = token_account
        .call(contract.id(), "get_token")
        .args_json(json!({"token_id": token_id}))
        .transact()
        .await?;

    println!("get_token outcome: {:#?}", outcome_get_token);

    let (account_id, total_supply, metadata, image) =
        outcome_get_token.json::<(AccountId, U128, FungibleTokenMetadata, String)>()?;

    assert_eq!(account_id, *token_account_2.id());
    assert_eq!(total_supply, U128::from(1_000_000_000));
    assert_eq!(metadata.name, "MemeCoin");
    assert_eq!(image, DATA_IMAGE_SVG_NEAR_ICON);

    Ok(())
}
