//! Ledger storage helpers for the token contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use crate::types::AllowanceValue;
use soroban_sdk::{Address, Env, String, Symbol};

const ADMIN_KEY: &str = "Admin";
const CLAWBACK_KEY: &str = "Clawback";

/// How close to the network's max TTL storage is kept — same self-adjusting
/// approach used throughout this repo (oracle, access-control, etc.):
/// extended whenever remaining TTL drops within this many ledgers of
/// max_ttl(), back up to the max itself, rather than a fixed ledger-count
/// guess tied to an assumed close time.
const TTL_EXTEND_BUFFER: u32 = 1_000;

/// Extends the whole contract instance's TTL — holds Admin, Name, Symbol,
/// Decimals, and Clawback. Losing it to archival would make every function
/// on the contract inoperable, not just one balance.
fn extend_instance_ttl(env: &Env) {
    let max_ttl = env.storage().max_ttl();
    env.storage()
        .instance()
        .extend_ttl(max_ttl.saturating_sub(TTL_EXTEND_BUFFER), max_ttl);
}

fn extend_balance_ttl(env: &Env, addr: &Address) {
    let max_ttl = env.storage().max_ttl();
    env.storage()
        .persistent()
        .extend_ttl(addr, max_ttl.saturating_sub(TTL_EXTEND_BUFFER), max_ttl);
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&Symbol::new(env, ADMIN_KEY))
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, ADMIN_KEY), admin);
    extend_instance_ttl(env);
}

pub fn get_admin(env: &Env) -> Address {
    let admin = env
        .storage()
        .instance()
        .get(&Symbol::new(env, ADMIN_KEY))
        .unwrap();
    extend_instance_ttl(env);
    admin
}

pub fn set_clawback_enabled(env: &Env, enabled: bool) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, CLAWBACK_KEY), &enabled);
}

pub fn get_clawback_enabled(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&Symbol::new(env, CLAWBACK_KEY))
        .unwrap_or(false)
}

pub fn get_balance(env: &Env, addr: &Address) -> i128 {
    let balance = env.storage().persistent().get(addr);
    if balance.is_some() {
        extend_balance_ttl(env, addr);
    }
    balance.unwrap_or(0)
}

pub fn set_balance(env: &Env, addr: &Address, amount: i128) {
    env.storage().persistent().set(addr, &amount);
    extend_balance_ttl(env, addr);
}

/// Returns the remaining allowance, or 0 if it was never set or its
/// expiration ledger has passed (an allowance is valid through its
/// expiration ledger, inclusive).
pub fn get_allowance(env: &Env, from: &Address, spender: &Address) -> i128 {
    let key = (from.clone(), spender.clone());
    match env.storage().temporary().get::<_, AllowanceValue>(&key) {
        Some(v) if v.expiration_ledger >= env.ledger().sequence() => v.amount,
        _ => 0,
    }
}

pub fn set_allowance(env: &Env, from: &Address, spender: &Address, amount: i128, expiry: u32) {
    let key = (from.clone(), spender.clone());
    env.storage().temporary().set(
        &key,
        &AllowanceValue {
            amount,
            expiration_ledger: expiry,
        },
    );
    // Keep the entry alive at least until expiry, capped at the network max.
    // Outliving it is harmless — get_allowance() checks expiration_ledger.
    let ledgers = expiry
        .saturating_sub(env.ledger().sequence())
        .min(env.storage().max_ttl());
    env.storage().temporary().extend_ttl(&key, ledgers, ledgers);
}

/// Update an existing allowance's remaining amount, keeping its expiration
/// ledger and TTL. Used by `transfer_from` to decrement the allowance on spend.
pub fn set_allowance_amount(env: &Env, from: &Address, spender: &Address, amount: i128) {
    let key = (from.clone(), spender.clone());
    if let Some(mut value) = env.storage().temporary().get::<_, AllowanceValue>(&key) {
        value.amount = amount;
        env.storage().temporary().set(&key, &value);
    }
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

pub fn get_symbol(env: &Env) -> String {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "Symbol"))
        .unwrap()
}

pub fn get_decimals(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "Decimals"))
        .unwrap_or(7)
}
