//! Typed event emission helpers for the upgradeable contract.
//!
//! All event emission goes through this module.
//! Never call `env.events().publish()` directly in `lib.rs`.
//!
//! `Events::publish` is deprecated in favor of the `#[contractevent]` macro;
//! migrating would change the emitted topic/data shape, so it's deferred.
#![allow(deprecated)]

use soroban_sdk::{Address, BytesN, Env, Symbol};

/// Emit an initialized event: topics = (initialized, admin), data = version
pub fn emit_initialized(env: &Env, admin: &Address, version: u32) {
    let topics = (Symbol::new(env, "initialized"), admin.clone());
    env.events().publish(topics, version);
}

/// Emit an upgraded event: topics = (upgraded,), data = new_wasm_hash
pub fn emit_upgraded(env: &Env, new_wasm_hash: &BytesN<32>) {
    let topics = (Symbol::new(env, "upgraded"),);
    env.events().publish(topics, new_wasm_hash.clone());
}

/// Emit a migrated event: topics = (migrated,), data = (from_version, to_version)
pub fn emit_migrated(env: &Env, from_version: u32, to_version: u32) {
    let topics = (Symbol::new(env, "migrated"),);
    env.events().publish(topics, (from_version, to_version));
}

/// Emit an admin_transferred event: topics = (admin_transferred, from), data = to
pub fn emit_admin_transferred(env: &Env, from: &Address, to: &Address) {
    let topics = (Symbol::new(env, "admin_transferred"), from.clone());
    env.events().publish(topics, to.clone());
}
