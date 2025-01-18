use chrono::Utc;
use near_sdk::json_types::{U128, U64};
use near_sdk::NearToken;
use token_factory::{EventLog, EventLogVariant, RegisterTokenLog};

#[test]
fn test_register_token() {
    let now = Utc::now().timestamp();
    let auction_duration = U64::from((now + 600) as u64 * 1000000000);

    let expected = format!(
        r#"EVENT_JSON:{{"standard":"nep141","version":"1.0.0","event":"register_token","data":[{{"owner_id":"token.near","total_supply":"100","spec":"ft-1.0.0","name":"token","symbol":"tk","icon":"data:image/icon","reference":null,"reference_hash":null,"decimals":24,"image":"data:image/icon","description":"Cool token","auction_duration":"{:?}","min_buy_amount":"50000000000000000000000000"}}]}}"#,
        auction_duration.0
    );

    let log = EventLog {
        standard: "nep141".to_string(),
        version: "1.0.0".to_string(),
        event: EventLogVariant::RegisterToken(vec![RegisterTokenLog {
            owner_id: "token.near".parse().unwrap(),
            total_supply: U128::from(100),
            spec: "ft-1.0.0".to_string(),
            name: "token".to_string(),
            symbol: "tk".to_string(),
            icon: Some("data:image/icon".to_string()),
            reference: None,
            reference_hash: None,
            decimals: 24,
            image: "data:image/icon".to_string(),
            description: "Cool token".to_string(),
            auction_duration,
            min_buy_amount: NearToken::from_near(50),
        }]),
    };
    println!("\nEXPECTED: {}", expected);
    println!("\nLOG: {}", log.to_string());

    assert_eq!(expected, log.to_string());
}
