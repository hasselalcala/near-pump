use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::{
    FungibleToken, FungibleTokenCore, FungibleTokenResolver,
};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::borsh::BorshSerialize;
use near_sdk::collections::{LazyOption, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::{
    env, log, near, require, AccountId, BorshStorageKey, NearToken, PanicOnDefault, Promise,
    PromiseOrValue,
};

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    image: String,
    description: String,
    auction: Auction,
    orders: Vector<Order>,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Auction {
    auctioner: AccountId,
    auction_duration: U64,
    auctioned_sell_amount: U128, //total amount of tokens to sell in the auction
    min_buy_amount: NearToken, // near amount to pay for all tokens
    is_settled: bool,
    winning_orders: Vec<(Order, bool)>,
    final_auction_price: NearToken, // price for the last token bought that applies for all the tokens bought
    refunded_orders: Vec<Order>,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Order {
    bidder: AccountId,
    buy_amount: U128,       // amount of tokens to buy
    sell_amount: NearToken, // near amount to pay for the buy_amount
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    FungibleToken,
    Metadata,
    Orders,
}

#[near]
impl Contract {
    #[init]
    #[allow(clippy::use_self)]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
        image: String,
        description: String,
        auction_duration: U64,
        min_buy_amount: NearToken,
    ) -> Self {
        require!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();

        let mut this = Self {
            token: FungibleToken::new(StorageKey::FungibleToken),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            image,
            description,
            auction: Auction {
                auctioner: owner_id,
                auction_duration,
                auctioned_sell_amount: total_supply,
                min_buy_amount,
                is_settled: false,
                winning_orders: Vec::new(),
                final_auction_price: NearToken::from_yoctonear(0),
                refunded_orders: Vec::new(),
            },
            orders: Vector::new(StorageKey::Orders),
        };

        // Assign the tokens to the contract itself 
        this.token
            .internal_register_account(&env::current_account_id());
        this.token
            .internal_deposit(&env::current_account_id(), total_supply.into());

        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &env::current_account_id(),
            amount: total_supply,
            memo: Some("New tokens are minted and ready to be auctioned"),
        }
        .emit();

        this
    }

    #[payable]
    pub fn register_bidder(&mut self) {
        let account_id = env::predecessor_account_id();
        let deposit = env::attached_deposit();

        let storage_balance_bounds = self.storage_balance_bounds();
        let minimum_balance = storage_balance_bounds.min;

        assert!(
            deposit >= minimum_balance,
            "The attached deposit is too small. Minimum required: {}",
            minimum_balance
        );

        // Register account in the token
        let _storage_balance = self.storage_deposit(Some(account_id.clone()), Some(true));

        // Refund the exceed deposit
        if deposit > minimum_balance {
            let refund = deposit.saturating_sub(minimum_balance);
            Promise::new(account_id.clone()).transfer(refund);
        }

        log!("Account @{} registered as a bidder", account_id);
    }

    #[payable]
    pub fn place_order(&mut self, buy_amount: U128) -> bool {
        let sell_amount = env::attached_deposit();

        assert!(
            self.token
                .accounts
                .contains_key(&env::predecessor_account_id()),
            "Account is not registered in token"
        );

        assert!(
            self.auction.auction_duration.0 > env::block_timestamp(),
            "Auction has ended"
        );

        assert!(
            sell_amount > NearToken::from_yoctonear(0),
            "Sell amount must be greater than 0, tokens are not free"
        );

        assert!(
            buy_amount <= self.auction.auctioned_sell_amount,
            "Buy amount is greater than tokens to sell"
        );
        assert!(
            buy_amount > near_sdk::json_types::U128(0),
            "Buy amount must be greater than 0"
        );

        let auctioner_price = self.auction.min_buy_amount.as_yoctonear() as f64
            / self.auction.auctioned_sell_amount.0 as f64;
        let offer_price = sell_amount.as_yoctonear() as f64 / buy_amount.0 as f64;
        assert!(
            offer_price >= auctioner_price,
            "Offer price is less than minimum price that auctioner is willing to accept"
        );

        let order = Order {
            bidder: env::predecessor_account_id(),
            buy_amount,
            sell_amount,
        };

        self.orders.push(&order);

        true
    }

    pub fn settle_auction(&mut self) {
        log!("Auction duration: {}", self.auction.auction_duration.0);
        log!("Block timestamp: {}", env::block_timestamp());

        assert!(
            self.auction.auction_duration.0 < env::block_timestamp(),
            "Auction has not ended yet, cannot calculate winning orders"
        );
        assert!(!self.auction.is_settled, "Auction already settled");

        self.sort_orders();
        self.calculate_winning_orders();
        self.calculate_final_auction_price();

        self.auction.is_settled = true;
    }

    // Orders are sorted by the price of the tokens (sell_amount / buy_amount) = (NearToken to pay/ amount of tokens to buy)
    // So the order with the highest price per token is the first
    fn sort_orders(&mut self) {
        let mut orders = self.orders.to_vec();
        orders.sort_by(|a, b| {
            let price_a = (a.sell_amount.as_yoctonear() as f64) / (a.buy_amount.0 as f64);
            let price_b = (b.sell_amount.as_yoctonear() as f64) / (b.buy_amount.0 as f64);
            price_b
                .partial_cmp(&price_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.orders.clear();
        for order in orders {
            self.orders.push(&order);
        }
    }

    fn calculate_winning_orders(&mut self) {
        let mut sum_sell_tokens = U128(0);

        for order in self.orders.to_vec() {
            let new_sum = Self::add(sum_sell_tokens, order.buy_amount);

            if new_sum.0 <= self.auction.auctioned_sell_amount.0 {
                sum_sell_tokens = new_sum;
                self.auction.winning_orders.push((order.clone(), false));
            } else {
                let remaining_tokens = self
                    .auction
                    .auctioned_sell_amount
                    .0
                    .saturating_sub(sum_sell_tokens.0);
                if remaining_tokens > 0 {
                    let final_order = Order {
                        bidder: order.bidder,
                        buy_amount: U128(remaining_tokens),
                        sell_amount: NearToken::from_yoctonear(
                            (order.sell_amount.as_yoctonear() * remaining_tokens)
                                / order.buy_amount.0,
                        ),
                    };
                    self.auction.winning_orders.push((final_order, false));
                }
                break;
            }
        }
    }

    fn add(a: U128, b: U128) -> U128 {
        U128(a.0.checked_add(b.0).expect("Math overflow"))
    }

    fn calculate_final_auction_price(&mut self) {
        let last_order = self.auction.winning_orders.last().unwrap().0.clone();

        let sell_amount_yocto = last_order.sell_amount.as_yoctonear();

        let buy_amount_u128 = last_order.buy_amount.0;

        let price_per_token_yocto = sell_amount_yocto / buy_amount_u128;

        self.auction.final_auction_price = NearToken::from_yoctonear(price_per_token_yocto);
    }

    pub fn claim_tokens(&mut self) {
        let claimer = env::predecessor_account_id();

        assert!(self.auction.is_settled, "Auction not settled yet");

        let (order_index, order) = self
            .find_unclaimed_winning_order(&claimer)
            .expect("You are not allowed to claim or have already claimed");

        self.auction.winning_orders[order_index].1 = true;

        self.token.internal_transfer(
            &env::current_account_id(),
            &claimer,
            order.buy_amount.into(),
            None,
        );

        let refund = self.calculate_near_to_return(&order);

        if refund > NearToken::from_yoctonear(0) {
            Promise::new(claimer.clone()).transfer(refund);
        }
    }

    fn find_unclaimed_winning_order(&self, claimer: &AccountId) -> Option<(usize, Order)> {
        self.auction
            .winning_orders
            .iter()
            .enumerate()
            .find(|(_, (order, claimed))| order.bidder == *claimer && !claimed)
            .map(|(index, (order, _))| (index, order.clone()))
    }

    #[allow(clippy::missing_const_for_fn)]
    fn calculate_near_to_return(&self, order: &Order) -> NearToken {
        let final_price = self.auction.final_auction_price;
        let tokens_to_buy = order.buy_amount.0;
        let total_cost = final_price.saturating_mul(tokens_to_buy);
        order.sell_amount.saturating_sub(total_cost)
    }

    pub fn refund_deposit(&mut self) -> Promise {
        assert!(self.auction.is_settled, "Auction not settled yet");
        let claimer = env::predecessor_account_id();

        assert!(
            self.orders.iter().any(|order| order.bidder == claimer),
            "No order found for this account"
        );

        assert!(
            !self
                .auction
                .refunded_orders
                .iter()
                .any(|order| order.bidder == claimer),
            "Refund has already been claimed"
        );

        assert!(
            !self
                .auction
                .winning_orders
                .iter()
                .any(|(order, _)| order.bidder == claimer),
            "Winning orders cannot claim refund"
        );

        let order = self
            .orders
            .iter()
            .find(|order| order.bidder == claimer)
            .expect("No order found for this account");

        self.auction.refunded_orders.push(order.clone());

        Promise::new(claimer).transfer(order.sell_amount)
    }

    //Get info about the auction

    pub fn get_orders(&self) -> Vec<Order> {
        self.orders.to_vec()
    }

    pub fn get_auction_info(&self) -> Auction {
        self.auction.clone()
    }

    pub fn get_auction_winner(&self) -> Vec<(Order, bool)> {
        self.auction.winning_orders.clone()
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn get_final_auction_price(&self) -> NearToken {
        self.auction.final_auction_price
    }
}

#[near]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}

#[near]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, burned_amount) =
            self.token
                .internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            log!("Account @{} burned {}", sender_id, burned_amount);
        }
        used_amount.into()
    }
}

#[near]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.token.storage_deposit(account_id, registration_only)
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<NearToken>) -> StorageBalance {
        self.token.storage_withdraw(amount)
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        #[allow(unused_variables)]
        if let Some((account_id, balance)) = self.token.internal_storage_unregister(force) {
            log!("Closed @{} with {}", account_id, balance);
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.token.storage_balance_bounds()
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.token.storage_balance_of(account_id)
    }
}

#[near]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
