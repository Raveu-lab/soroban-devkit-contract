//! A simple price feed oracle.
//!
//! The admin publishes a price per asset (a Symbol, e.g. "BTC" or "XLM").
//! Anyone can read the latest price and check whether it's stale relative
//! to a caller-supplied max age — the contract doesn't enforce staleness
//! itself, since "too old" depends entirely on what the reader is using
//! the price for.
//!
//! # Architecture
//! - `lib.rs`     — public contract interface only
//! - `storage.rs` — all ledger reads and writes
//! - `events.rs`  — all event emission
//! - `errors.rs`  — error enum
//! - `types.rs`   — shared data types

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

mod errors;
mod events;
mod storage;
mod types;

pub use errors::OracleError;
pub use types::PriceData;

#[contract]
pub struct OracleContract;

#[contractimpl]
impl OracleContract {
    /// Initialize with an admin address. Can only be called once.
    pub fn initialize(env: Env, admin: Address) {
        if storage::is_initialized(&env) {
            panic!("already initialized");
        }
        storage::set_admin(&env, &admin);
    }

    /// Publish a price for an asset. Admin only. Timestamps it with the
    /// current ledger time. Panics if `price` is not positive.
    pub fn set_price(env: Env, asset: Symbol, price: i128) {
        let admin = storage::get_admin(&env);
        admin.require_auth();

        if price <= 0 {
            panic!("price must be positive");
        }

        let data = PriceData {
            price,
            timestamp: env.ledger().timestamp(),
        };
        storage::set_price(&env, &asset, &data);
        events::emit_price_updated(&env, &asset, price, data.timestamp);
    }

    /// Return the latest published price for an asset.
    /// Panics if no price has ever been published for it.
    pub fn get_price(env: Env, asset: Symbol) -> PriceData {
        storage::get_price(&env, &asset).unwrap_or_else(|| panic!("no price for asset"))
    }

    /// Return true if the asset's latest price is older than `max_age`
    /// seconds. Panics if no price has ever been published for it.
    pub fn is_stale(env: Env, asset: Symbol, max_age: u64) -> bool {
        let data = Self::get_price(env.clone(), asset);
        env.ledger().timestamp().saturating_sub(data.timestamp) > max_age
    }

    /// Return the configured admin address.
    pub fn admin(env: Env) -> Address {
        storage::get_admin(&env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Env};

    fn setup(env: &Env) -> (Address, OracleContractClient<'_>) {
        env.mock_all_auths();
        let contract_id = env.register(OracleContract, ());
        let client = OracleContractClient::new(env, &contract_id);
        let admin = Address::generate(env);
        client.initialize(&admin);
        (admin, client)
    }

    #[test]
    fn test_set_and_get_price() {
        let env = Env::default();
        let (_admin, client) = setup(&env);
        let asset = Symbol::new(&env, "BTC");

        client.set_price(&asset, &5_000_000);

        let data = client.get_price(&asset);
        assert_eq!(data.price, 5_000_000);
    }

    #[test]
    fn test_price_updates_on_second_set() {
        let env = Env::default();
        let (_admin, client) = setup(&env);
        let asset = Symbol::new(&env, "BTC");

        client.set_price(&asset, &5_000_000);
        client.set_price(&asset, &5_100_000);

        assert_eq!(client.get_price(&asset).price, 5_100_000);
    }

    #[test]
    fn test_timestamp_reflects_ledger_time() {
        let env = Env::default();
        let (_admin, client) = setup(&env);
        let asset = Symbol::new(&env, "BTC");

        env.ledger().set_timestamp(12_345);
        client.set_price(&asset, &5_000_000);

        assert_eq!(client.get_price(&asset).timestamp, 12_345);
    }

    #[test]
    #[should_panic(expected = "no price")]
    fn test_get_price_for_unknown_asset_panics() {
        let env = Env::default();
        let (_admin, client) = setup(&env);
        client.get_price(&Symbol::new(&env, "BTC"));
    }

    #[test]
    #[should_panic(expected = "price must be positive")]
    fn test_set_price_rejects_zero() {
        let env = Env::default();
        let (_admin, client) = setup(&env);
        client.set_price(&Symbol::new(&env, "BTC"), &0);
    }

    #[test]
    #[should_panic(expected = "price must be positive")]
    fn test_set_price_rejects_negative() {
        let env = Env::default();
        let (_admin, client) = setup(&env);
        client.set_price(&Symbol::new(&env, "BTC"), &-1);
    }

    #[test]
    fn test_is_stale_false_when_fresh() {
        let env = Env::default();
        let (_admin, client) = setup(&env);
        let asset = Symbol::new(&env, "BTC");

        env.ledger().set_timestamp(1_000);
        client.set_price(&asset, &5_000_000);

        env.ledger().set_timestamp(1_050);
        assert!(!client.is_stale(&asset, &100));
    }

    #[test]
    fn test_is_stale_true_after_max_age() {
        let env = Env::default();
        let (_admin, client) = setup(&env);
        let asset = Symbol::new(&env, "BTC");

        env.ledger().set_timestamp(1_000);
        client.set_price(&asset, &5_000_000);

        env.ledger().set_timestamp(1_200);
        assert!(client.is_stale(&asset, &100));
    }

    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_double_initialize_panics() {
        let env = Env::default();
        let (admin, client) = setup(&env);
        client.initialize(&admin);
    }
}
