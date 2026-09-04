//! Ledger storage helpers for the oracle contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use crate::types::PriceData;
use soroban_sdk::{Address, Env, Symbol};

const ADMIN_KEY: &str = "Admin";

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&Symbol::new(env, ADMIN_KEY))
}

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

pub fn set_price(env: &Env, asset: &Symbol, data: &PriceData) {
    env.storage().persistent().set(asset, data);
}

pub fn get_price(env: &Env, asset: &Symbol) -> Option<PriceData> {
    env.storage().persistent().get(asset)
}
