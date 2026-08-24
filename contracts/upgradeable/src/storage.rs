//! Ledger storage helpers for the upgradeable contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use soroban_sdk::{Address, Env, Symbol};

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Admin"), admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "Admin"))
        .unwrap()
}

pub fn set_version(env: &Env, version: u32) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Version"), &version);
}

pub fn get_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "Version"))
        .unwrap_or(0)
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&Symbol::new(env, "Admin"))
}
