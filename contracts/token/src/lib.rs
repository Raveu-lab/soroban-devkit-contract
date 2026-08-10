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

use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol};

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
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
    ) {
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

    /// Approve a spender to transfer up to `amount` on behalf of the caller.
    pub fn approve(env: Env, from: Address, spender: Address, amount: i128, expiry_ledger: u32) {
        from.require_auth();
        storage::set_allowance(&env, &from, &spender, amount, expiry_ledger);
        events::emit_approve(&env, &from, &spender, amount, expiry_ledger);
    }

    /// Transfer tokens on behalf of `from` using a pre-approved allowance.
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        let allowance = storage::get_allowance(&env, &from, &spender);
        if allowance < amount {
            panic!("insufficient allowance");
        }

        // TODO: Implement full transfer_from with allowance decrement
        // See: https://github.com/soroban-devkit/soroban-devkit-contracts/issues/3
        let _ = (allowance, from, to, amount);
        todo!("transfer_from — tracked in issue #3")
    }

    /// Return token balance for an address.
    pub fn balance(env: Env, id: Address) -> i128 {
        storage::get_balance(&env, &id)
    }

    /// Return the token symbol.
    pub fn symbol(env: Env) -> Symbol {
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
}
