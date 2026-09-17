//! Ledger storage helpers for the oracle contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use crate::types::PriceData;
use soroban_sdk::{Address, Env, Symbol};

const ADMIN_KEY: &str = "Admin";

/// How close to the network's max TTL a price entry is kept — extended
/// whenever its remaining TTL drops within this many ledgers of that max,
/// back up to the max itself. Self-adjusting to whatever the network's
/// actual max TTL is, rather than a fixed ledger-count guess tied to an
/// assumed close time.
const TTL_EXTEND_BUFFER: u32 = 1_000;

fn extend_price_ttl(env: &Env, asset: &Symbol) {
    let max_ttl = env.storage().max_ttl();
    env.storage().persistent().extend_ttl(
        asset,
        max_ttl.saturating_sub(TTL_EXTEND_BUFFER),
        max_ttl,
    );
}

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
    extend_price_ttl(env, asset);
}

pub fn get_price(env: &Env, asset: &Symbol) -> Option<PriceData> {
    let data = env.storage().persistent().get(asset);
    if data.is_some() {
        extend_price_ttl(env, asset);
    }
    data
}
