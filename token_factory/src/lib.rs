use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};

use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, env::storage_byte_cost, json_types::Base64VecU8, near_bindgen, AccountId, BorshStorageKey,
    Gas, PanicOnDefault, Promise, StorageUsage,
};
use near_sdk::{log, serde_json, NearToken, PromiseResult};

mod events;
pub use events::*;

pub const ZERO_TOKEN: NearToken = NearToken::from_yoctonear(0);
const FT_WASM_CODE: &[u8] = include_bytes!("../../token_factory/base_token/base_token.wasm");

const EXTRA_BYTES: usize = 10000;
const GAS: Gas = Gas::from_gas(50 * 1_000_000_000_000);
type TokenId = String;

pub fn is_valid_token_id(token_id: &TokenId) -> bool {
    for c in token_id.as_bytes() {
        match c {
            b'0'..=b'9' | b'a'..=b'z' => (),
            _ => return false,
        }
    }
    true
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Tokens,
    StorageDeposits,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct TokenFactory {
    pub tokens: UnorderedMap<TokenId, TokenArgs>,
    pub storage_deposits: LookupMap<AccountId, NearToken>,
    pub storage_balance_cost: StorageUsage, // Count the amount of storage used by a contract.
    pub default_total_supply: U128,
    pub owner_id: AccountId, // owner of the factory who is the creator of the contract
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenArgs {
    pub owner_id: AccountId,             // creator of the token
    pub total_supply: U128,              // total supply of the token defined by default
    pub metadata: FungibleTokenMetadata, // metadata of the token includes name, symbol, decimals, etc.
    pub image: String,                   // image of the token
    pub description: String,             // description of the token
    pub auction_duration: U64,
    pub min_buy_amount: NearToken,
}

#[allow(clippy::use_self)]
#[near_bindgen]
impl TokenFactory {
    #[init]
    pub fn new() -> Self {
        Self {
            tokens: UnorderedMap::new(StorageKey::Tokens),
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits),
            storage_balance_cost: 0,
            default_total_supply: U128::from(1_000_000_000),
            owner_id: env::predecessor_account_id(),
        }
    }

    pub(crate) fn measure_bytes_for_account_id(&mut self, account_id: AccountId) -> StorageUsage {
        let initial_storage_usage = env::storage_usage();
        self.storage_deposits.insert(&account_id, &ZERO_TOKEN);
        self.storage_balance_cost = env::storage_usage() - initial_storage_usage;
        self.storage_deposits.remove(&account_id);
        return self.storage_balance_cost;
    }

    fn get_min_attached_balance(&self, args: &TokenArgs) -> NearToken {
        let serialize_args = borsh::to_vec(&args).unwrap();
        storage_byte_cost().saturating_mul(
            (FT_WASM_CODE.len() + EXTRA_BYTES + serialize_args.len() * 2)
                .try_into()
                .unwrap(),
        )
    }

    #[payable]
    pub fn storage_deposit(&mut self) {
        let account_id = env::predecessor_account_id();
        let deposit = env::attached_deposit();

        let storage_balance_cost = self.measure_bytes_for_account_id(account_id.clone());

        if let Some(previous_balance) = self.storage_deposits.get(&account_id) {
            self.storage_deposits
                .insert(&account_id, &previous_balance.saturating_add(deposit));
        } else {
            assert!(
                deposit >= NearToken::from_near(storage_balance_cost.into()),
                "Deposit is too low, you need: {}",
                self.storage_balance_cost
            );

            self.storage_deposits.insert(
                &account_id,
                &deposit.saturating_sub(NearToken::from_near(self.storage_balance_cost.into())),
            );
        }
    }

    pub fn get_number_of_tokens(&self) -> u64 {
        self.tokens.len()
    }

    pub fn get_token(
        &self,
        token_id: TokenId,
    ) -> Option<(AccountId, U128, FungibleTokenMetadata, String)> {
        match self.tokens.get(&token_id) {
            Some(token) => Some((
                token.owner_id,
                token.total_supply,
                token.metadata,
                token.image,
            )),
            None => panic!("Token not found"),
        }
    }

    #[payable]
    #[allow(clippy::too_many_arguments)]
    pub fn create_token(
        &mut self,
        spec: String,
        name: String,
        symbol: String,
        icon: Option<String>,
        reference: Option<String>,
        reference_hash: Option<Base64VecU8>,
        decimals: u8,
        image: String,
        description: String,
        auction_duration: U64,
        min_buy_amount: NearToken,
    ) -> Promise {
        let owner_id = env::predecessor_account_id();

        let token_id = symbol.to_ascii_lowercase();
        assert!(is_valid_token_id(&token_id), "Invalid Symbol");

        assert!(
            !self
                .tokens
                .values()
                .any(|token| token.metadata.name == name),
            "A token with this name already exists"
        );

        //@dev Current account id is the account id of this smart contract
        let token_account_id = format!("{}.{}", token_id, env::current_account_id());
        assert!(
            env::is_valid_account_id(token_account_id.as_bytes()),
            "Token Account ID is invalid"
        );

        let args = TokenArgs {
            owner_id: owner_id.clone(),
            total_supply: self.default_total_supply,
            metadata: FungibleTokenMetadata {
                spec,
                name,
                symbol,
                icon,
                reference,
                reference_hash,
                decimals,
            },
            image,
            description,
            auction_duration,
            min_buy_amount,
        };

        if env::attached_deposit() > ZERO_TOKEN {
            self.storage_deposit();
        }
        args.metadata.assert_valid();

        let required_balance = self.get_min_attached_balance(&args);
        let user_balance = self
            .storage_deposits
            .get(&owner_id)
            .map_or(ZERO_TOKEN, |balance| balance);

        assert!(
            user_balance >= required_balance,
            "Not enough required balance,  user_balance is {} and required balance is {}",
            user_balance,
            required_balance
        );

        self.storage_deposits
            .insert(&owner_id, &user_balance.saturating_sub(required_balance));

        //@dev Current storage used by this smart contract in bytes
        let initial_storage_usage = env::storage_usage();

        assert!(
            self.tokens.insert(&token_id, &args).is_none(),
            "Token ID is already taken"
        );

        let storage_balance_used = storage_byte_cost()
            .saturating_mul((env::storage_usage() - initial_storage_usage).into());

        let transfer = required_balance.saturating_sub(storage_balance_used);

        Promise::new(token_account_id.parse().unwrap())
            .create_account()
            .transfer(transfer)
            .deploy_contract(FT_WASM_CODE.to_vec())
            .function_call(
                "new".to_string(),
                serde_json::to_vec(&args).unwrap(),
                NearToken::from_near(0),
                GAS,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS)
                    .on_create_token_callback(
                        args.owner_id,
                        args.total_supply,
                        args.metadata.spec,
                        args.metadata.name,
                        args.metadata.symbol,
                        args.metadata.icon,
                        args.metadata.reference,
                        args.metadata.reference_hash,
                        args.metadata.decimals,
                        args.image,
                        args.description,
                        args.auction_duration,
                        args.min_buy_amount,
                    ),
            )
    }

    #[private]
    pub fn on_create_token_callback(
        &mut self,
        owner_id: AccountId,
        total_supply: U128,
        spec: String,
        name: String,
        symbol: String,
        icon: Option<String>,
        reference: Option<String>,
        reference_hash: Option<Base64VecU8>,
        decimals: u8,
        image: String,
        description: String,
        auction_duration: U64,
        min_buy_amount: NearToken,
    ) {
        if let PromiseResult::Successful(_) = env::promise_result(0) {
            let register_token_log = EventLog {
                standard: "nep141".to_string(),
                version: "1.0.0".to_string(),
                event: EventLogVariant::RegisterToken(vec![RegisterTokenLog {
                    owner_id,
                    total_supply,
                    spec,
                    name,
                    symbol,
                    icon,
                    reference,
                    reference_hash,
                    decimals,
                    image,
                    description,
                    auction_duration,
                    min_buy_amount,
                }]),
            };

            log!("{}", &register_token_log.to_string());
        } else {
            panic!("Token creation failed");
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use chrono::Utc;
    use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
    use near_sdk::json_types::{U128, U64};
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, AccountId, NearToken};

    const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/icon";
    const FT_METADATA_SPEC: &str = "ft-1.0.0";

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    fn new_default_token_args(
        owner_id: AccountId,
        total_supply: U128,
        image: String,
        description: String,
    ) -> TokenArgs {
        let now = Utc::now().timestamp();
        TokenArgs {
            owner_id,
            //token_address: env::current_account_id(),
            total_supply,
            metadata: FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
            image,
            description,
            auction_duration: U64::from((now + 600) as u64 * 1000000000),
            min_buy_amount: NearToken::from_near(50),
        }
    }

    #[test]
    fn test_get_min_attached_balance() {
        let context = get_context(accounts(1));
        testing_env!(context.build());

        let contract = TokenFactory::new();

        let args = new_default_token_args(
            accounts(1),
            U128::from(100_000_000_000),
            DATA_IMAGE_SVG_NEAR_ICON.to_string(),
            "This is a test token".to_string(),
        );
        let result = contract.get_min_attached_balance(&args);
        assert_eq!(result, NearToken::from_yoctonear(2650350000000000000000000));
    }
}
