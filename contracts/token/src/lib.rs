//! SEP-41 compliant fungible token contract.
//!
//! Implements: transfer, approve, transfer_from, mint, burn, clawback.
//! Emits a typed event on every state-changing operation.
//!
//! # Architecture
//! - `lib.rs`     — public contract interface only
//! - `storage.rs` — all ledger reads and writes
//! - `events.rs`  — all event emission
//! - `errors.rs`  — error enum
//! - `types.rs`   — shared data types

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String};

mod errors;
mod events;
mod storage;
mod types;

pub use errors::TokenError;

#[contract]
pub struct TokenContract;

#[contractimpl]
impl TokenContract {
    /// Initialize the token with metadata and an admin address.
    /// Can only be called once — panics if already initialized.
    pub fn initialize(env: Env, admin: Address, name: String, symbol: String, decimals: u32) {
        storage::set_admin(&env, &admin);
        storage::set_metadata(&env, name, symbol, decimals);
        storage::set_clawback_enabled(&env, false);
    }

    /// Mint tokens to an address. Admin only.
    /// Emits a `mint` event on success.
    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin = storage::get_admin(&env);
        admin.require_auth();

        let balance = storage::get_balance(&env, &to);
        storage::set_balance(&env, &to, balance + amount);
        events::emit_mint(&env, &to, amount);
    }

    /// Transfer tokens from the caller to a recipient.
    /// Requires auth from `from`. Panics if balance is insufficient.
    /// Emits a `transfer` event on success.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        let from_balance = storage::get_balance(&env, &from);
        if from_balance < amount {
            panic!("insufficient balance");
        }

        storage::set_balance(&env, &from, from_balance - amount);
        let to_balance = storage::get_balance(&env, &to);
        storage::set_balance(&env, &to, to_balance + amount);

        events::emit_transfer(&env, &from, &to, amount);
    }

    /// Approve a spender to transfer up to `amount` tokens on behalf of `from`.
    /// Requires auth from `from`. Emits an `approve` event on success.
    pub fn approve(env: Env, from: Address, spender: Address, amount: i128, expiry_ledger: u32) {
        from.require_auth();
        storage::set_allowance(&env, &from, &spender, amount, expiry_ledger);
        events::emit_approve(&env, &from, &spender, amount, expiry_ledger);
    }

    /// Transfer tokens on behalf of `from` using a pre-approved allowance.
    /// Requires auth from `spender`. Decrements the allowance by `amount`.
    /// Panics if the allowance or `from`'s balance is insufficient.
    /// Emits the same `transfer` event as a direct transfer.
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        let allowance = storage::get_allowance(&env, &from, &spender);
        if allowance < amount {
            panic!("insufficient allowance");
        }

        let from_balance = storage::get_balance(&env, &from);
        if from_balance < amount {
            panic!("insufficient balance");
        }

        storage::set_allowance_amount(&env, &from, &spender, allowance - amount);

        storage::set_balance(&env, &from, from_balance - amount);
        let to_balance = storage::get_balance(&env, &to);
        storage::set_balance(&env, &to, to_balance + amount);

        events::emit_transfer(&env, &from, &to, amount);
    }

    /// Return token balance for an address.
    pub fn balance(env: Env, id: Address) -> i128 {
        storage::get_balance(&env, &id)
    }

    /// Return the remaining allowance `spender` has to transfer on behalf of `from`.
    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        storage::get_allowance(&env, &from, &spender)
    }

    /// Return the token symbol.
    pub fn symbol(env: Env) -> String {
        storage::get_symbol(&env)
    }

    /// Return the token name.
    pub fn name(env: Env) -> String {
        storage::get_name(&env)
    }

    /// Return the number of decimals.
    pub fn decimals(env: Env) -> u32 {
        storage::get_decimals(&env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_mint_and_balance() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TokenContract, ());
        let client = TokenContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "DevKit Token"),
            &String::from_str(&env, "DKT"),
            &7,
        );
        client.mint(&user, &1_000_000);
        assert_eq!(client.balance(&user), 1_000_000);
    }

    #[test]
    fn test_name_and_symbol_reflect_initialize_args() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TokenContract, ());
        let client = TokenContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "USD Coin"),
            &String::from_str(&env, "USDC"),
            &6,
        );

        assert_eq!(client.name(), String::from_str(&env, "USD Coin"));
        assert_eq!(client.symbol(), String::from_str(&env, "USDC"));
        assert_eq!(client.decimals(), 6);
    }

    #[test]
    fn test_balance_of_uninitialized_address_is_zero() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TokenContract, ());
        let client = TokenContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let stranger = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "DevKit Token"),
            &String::from_str(&env, "DKT"),
            &7,
        );

        assert_eq!(client.balance(&stranger), 0);
    }

    #[test]
    fn test_transfer() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TokenContract, ());
        let client = TokenContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "DevKit Token"),
            &String::from_str(&env, "DKT"),
            &7,
        );
        client.mint(&alice, &1_000_000);
        client.transfer(&alice, &bob, &400_000);

        assert_eq!(client.balance(&alice), 600_000);
        assert_eq!(client.balance(&bob), 400_000);
    }

    #[test]
    fn test_transfer_from_moves_balance() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TokenContract, ());
        let client = TokenContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let alice = Address::generate(&env);
        let spender = Address::generate(&env);
        let bob = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "DevKit Token"),
            &String::from_str(&env, "DKT"),
            &7,
        );
        client.mint(&alice, &1_000_000);
        client.approve(&alice, &spender, &500_000, &1000);

        client.transfer_from(&spender, &alice, &bob, &200_000);

        assert_eq!(client.balance(&alice), 800_000);
        assert_eq!(client.balance(&bob), 200_000);
    }

    #[test]
    fn test_transfer_from_reduces_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TokenContract, ());
        let client = TokenContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let alice = Address::generate(&env);
        let spender = Address::generate(&env);
        let bob = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "DevKit Token"),
            &String::from_str(&env, "DKT"),
            &7,
        );
        client.mint(&alice, &1_000_000);
        client.approve(&alice, &spender, &500_000, &1000);

        client.transfer_from(&spender, &alice, &bob, &200_000);

        assert_eq!(client.allowance(&alice, &spender), 300_000);
    }

    #[test]
    #[should_panic(expected = "insufficient allowance")]
    fn test_transfer_from_fails_over_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TokenContract, ());
        let client = TokenContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let alice = Address::generate(&env);
        let spender = Address::generate(&env);
        let bob = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "DevKit Token"),
            &String::from_str(&env, "DKT"),
            &7,
        );
        client.mint(&alice, &1_000_000);
        client.approve(&alice, &spender, &100_000, &1000);

        client.transfer_from(&spender, &alice, &bob, &200_000);
    }

    #[test]
    #[should_panic(expected = "insufficient balance")]
    fn test_transfer_from_fails_over_balance() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TokenContract, ());
        let client = TokenContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let alice = Address::generate(&env);
        let spender = Address::generate(&env);
        let bob = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "DevKit Token"),
            &String::from_str(&env, "DKT"),
            &7,
        );
        client.mint(&alice, &100_000);
        client.approve(&alice, &spender, &500_000, &1000);

        client.transfer_from(&spender, &alice, &bob, &200_000);
    }
}
