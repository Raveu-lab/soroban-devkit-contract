//! Typed event emission helpers for the escrow contract.
//!
//! All event emission goes through this module.
//! Never call `env.events().publish()` directly in `lib.rs`.
//!
//! `Events::publish` is deprecated in favor of the `#[contractevent]` macro;
//! migrating would change the emitted topic/data shape, so it's deferred.
#![allow(deprecated)]

use soroban_sdk::{Address, Env, Symbol};

/// Emit a deposited event: topics = (deposited, depositor), data = (id, recipient, amount)
pub fn emit_deposited(env: &Env, id: u32, depositor: &Address, recipient: &Address, amount: i128) {
    let topics = (Symbol::new(env, "deposited"), depositor.clone());
    env.events()
        .publish(topics, (id, recipient.clone(), amount));
}

/// Emit a released event: topics = (released, caller), data = escrow_id
pub fn emit_released(env: &Env, id: u32, caller: &Address) {
    let topics = (Symbol::new(env, "released"), caller.clone());
    env.events().publish(topics, id);
}

/// Emit a refunded event: topics = (refunded, caller), data = escrow_id
pub fn emit_refunded(env: &Env, id: u32, caller: &Address) {
    let topics = (Symbol::new(env, "refunded"), caller.clone());
    env.events().publish(topics, id);
}

/// Emit a disputed event: topics = (disputed, caller), data = escrow_id
pub fn emit_disputed(env: &Env, id: u32, caller: &Address) {
    let topics = (Symbol::new(env, "disputed"), caller.clone());
    env.events().publish(topics, id);
}
