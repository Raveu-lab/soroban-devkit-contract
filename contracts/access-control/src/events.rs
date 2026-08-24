//! Typed event emission helpers for the access-control contract.
//!
//! All event emission goes through this module.
//! Never call `env.events().publish()` directly in `lib.rs`.
//!
//! `Events::publish` is deprecated in favor of the `#[contractevent]` macro;
//! migrating would change the emitted topic/data shape, so it's deferred.
#![allow(deprecated)]

use soroban_sdk::{Address, Env, Symbol};

/// Emit a role_granted event: topics = (role_granted, role, to), data = by
pub fn emit_role_granted(env: &Env, role: &Symbol, to: &Address, by: &Address) {
    let topics = (Symbol::new(env, "role_granted"), role.clone(), to.clone());
    env.events().publish(topics, by.clone());
}

/// Emit a role_revoked event: topics = (role_revoked, role, from), data = by
pub fn emit_role_revoked(env: &Env, role: &Symbol, from: &Address, by: &Address) {
    let topics = (Symbol::new(env, "role_revoked"), role.clone(), from.clone());
    env.events().publish(topics, by.clone());
}
