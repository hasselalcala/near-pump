use chrono::Utc;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    json_types::{U128, U64},
    AccountId, NearToken as SdkNearToken,
};
use near_workspaces::types::NearToken as WorkspacesNearToken;
use token_factory::TokenArgs;

pub const HUNDRED_NEAR: WorkspacesNearToken = WorkspacesNearToken::from_near(100);
const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/icon";
const FT_METADATA_SPEC: &str = "ft-1.0.0";

pub fn new_default_token_args(
    owner_id: &AccountId,
    total_supply: U128,
    image: String,
    description: String,
) -> TokenArgs {
    let now = Utc::now().timestamp();
    TokenArgs {
        owner_id: owner_id.clone(),
        total_supply,
        metadata: FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "Example NEAR fungible token".to_string(),
            symbol: "example".to_string(),
            icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
            reference: None,
            reference_hash: None,
            decimals: 24,
        },
        image,
        description,
        auction_duration: U64::from((now + 600) as u64 * 1000000000),
        min_buy_amount: SdkNearToken::from_yoctonear(WorkspacesNearToken::from_near(50).as_yoctonear()),
    }
}

