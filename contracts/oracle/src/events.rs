//! Typed event emission helpers for the oracle contract.
//!
//! All event emission goes through this module.
//! Never call `env.events().publish()` directly in `lib.rs`.
//!
//! `Events::publish` is deprecated in favor of the `#[contractevent]` macro;
//! migrating would change the emitted topic/data shape, so it's deferred.
#![allow(deprecated)]

use soroban_sdk::{Env, Symbol};

/// Emit a price_updated event: topics = (price_updated, asset), data = (price, timestamp)
pub fn emit_price_updated(env: &Env, asset: &Symbol, price: i128, timestamp: u64) {
    let topics = (Symbol::new(env, "price_updated"), asset.clone());
    env.events().publish(topics, (price, timestamp));
}
