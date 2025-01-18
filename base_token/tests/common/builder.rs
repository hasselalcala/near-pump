use near_sdk::json_types::{U128, U64};
use near_workspaces::types::NearToken;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ContractBuilder {
    owner_id: String,
    total_supply: u128,
    metadata: ContractMetadata,
    image: String,
    auction_duration: u64,
    min_buy_amount: u128,
}

#[derive(Debug, Deserialize)]
pub struct ContractMetadata {
    spec: String,
    name: String,
    symbol: String,
    decimals: u8,
}

impl ContractBuilder {
    pub fn new(owner_id: String) -> Self {
        Self {
            owner_id,
            total_supply: 1_000_000_000_000_000,
            metadata: ContractMetadata {
                spec: "ft-1.0.0".to_string(),
                name: "Example Token".to_string(),
                symbol: "EXTKN".to_string(),
                decimals: 18,
            },
            image: "https://example.com/token-image.png".to_string(),
            auction_duration: 600, // 10 minutes
            min_buy_amount: NearToken::from_near(1).as_yoctonear(),
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn total_supply(mut self, total_supply: u128) -> Self {
        self.total_supply = total_supply;
        self
    }

    #[allow(dead_code)]
    pub fn metadata(mut self, metadata: ContractMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    #[allow(dead_code)]
    pub fn image(mut self, image: String) -> Self {
        self.image = image;
        self
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn auction_duration(mut self, auction_duration: u64) -> Self {
        self.auction_duration = auction_duration;
        self
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn min_buy_amount(mut self, min_buy_amount: u128) -> Self {
        self.min_buy_amount = min_buy_amount;
        self
    }

    pub fn build(self) -> serde_json::Value {
        serde_json::json!({
            "owner_id": self.owner_id,
            "total_supply": U128::from(self.total_supply),
            "metadata": {
                "spec": self.metadata.spec,
                "name": self.metadata.name,
                "symbol": self.metadata.symbol,
                "decimals": self.metadata.decimals
            },
            "image": self.image,
            "auction_duration": U64::from(self.auction_duration),
            "min_buy_amount": U128::from(self.min_buy_amount)
        })
    }
}
