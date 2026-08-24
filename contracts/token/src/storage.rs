//! Ledger storage helpers for the token contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use soroban_sdk::{Address, Env, String, Symbol};

const ADMIN_KEY: &str = "Admin";
const CLAWBACK_KEY: &str = "Clawback";

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, ADMIN_KEY), admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&Symbol::new(env, ADMIN_KEY))
        .unwrap()
}

pub fn set_clawback_enabled(env: &Env, enabled: bool) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, CLAWBACK_KEY), &enabled);
}

pub fn get_balance(env: &Env, addr: &Address) -> i128 {
    env.storage().persistent().get(addr).unwrap_or(0)
}

pub fn set_balance(env: &Env, addr: &Address, amount: i128) {
    env.storage().persistent().set(addr, &amount);
}

pub fn get_allowance(env: &Env, from: &Address, spender: &Address) -> i128 {
    let key = (from.clone(), spender.clone());
    env.storage().temporary().get(&key).unwrap_or(0)
}

pub fn set_allowance(env: &Env, from: &Address, spender: &Address, amount: i128, expiry: u32) {
    let key = (from.clone(), spender.clone());
    env.storage().temporary().set(&key, &amount);
    env.storage().temporary().extend_ttl(&key, expiry, expiry);
}

/// Update an existing allowance's remaining amount without touching its TTL.
/// Used by `transfer_from` to decrement the allowance on spend.
pub fn set_allowance_amount(env: &Env, from: &Address, spender: &Address, amount: i128) {
    let key = (from.clone(), spender.clone());
    env.storage().temporary().set(&key, &amount);
}

pub fn set_metadata(env: &Env, name: String, symbol: String, decimals: u32) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Name"), &name);
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Symbol"), &symbol);
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Decimals"), &decimals);
}

pub fn get_name(env: &Env) -> String {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "Name"))
        .unwrap()
}

pub fn get_symbol(env: &Env) -> Symbol {
    Symbol::new(env, "DKT") // TODO: read from storage dynamically
}

pub fn get_decimals(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "Decimals"))
        .unwrap_or(7)
}
