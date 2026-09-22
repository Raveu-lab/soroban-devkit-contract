//! Ledger storage helpers for the token contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

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
