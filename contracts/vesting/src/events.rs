//! Typed event emission helpers for the vesting contract.
//!
//! All event emission goes through this module.
//! Never call `env.events().publish()` directly in `lib.rs`.
//!
//! `Events::publish` is deprecated in favor of the `#[contractevent]` macro;
//! migrating would change the emitted topic/data shape, so it's deferred.
#![allow(deprecated)]

use soroban_sdk::{Address, Env, Symbol};

/// Emit a created event: topics = (created, depositor), data = (id, beneficiary, total_amount)
pub fn emit_created(
    env: &Env,
    id: u32,
    depositor: &Address,
    beneficiary: &Address,
    total_amount: i128,
) {
    let topics = (Symbol::new(env, "created"), depositor.clone());
    env.events()
        .publish(topics, (id, beneficiary.clone(), total_amount));
}

/// Emit a claimed event: topics = (claimed, beneficiary), data = (id, amount)
pub fn emit_claimed(env: &Env, id: u32, beneficiary: &Address, amount: i128) {
    let topics = (Symbol::new(env, "claimed"), beneficiary.clone());
    env.events().publish(topics, (id, amount));
}

/// Emit a revoked event: topics = (revoked, depositor), data = (id, returned_amount)
pub fn emit_revoked(env: &Env, id: u32, depositor: &Address, returned_amount: i128) {
    let topics = (Symbol::new(env, "revoked"), depositor.clone());
    env.events().publish(topics, (id, returned_amount));
}
