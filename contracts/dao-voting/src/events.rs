//! Typed event emission helpers for the dao-voting contract.
//!
//! All event emission goes through this module.
//! Never call `env.events().publish()` directly in `lib.rs`.
//!
//! `Events::publish` is deprecated in favor of the `#[contractevent]` macro;
//! migrating would change the emitted topic/data shape, so it's deferred.
#![allow(deprecated)]

use soroban_sdk::{Address, Env, Symbol};

/// Emit a proposed event: topics = (proposed, proposer), data = (id, deadline)
pub fn emit_proposed(env: &Env, id: u32, proposer: &Address, deadline: u64) {
    let topics = (Symbol::new(env, "proposed"), proposer.clone());
    env.events().publish(topics, (id, deadline));
}

/// Emit a voted event: topics = (voted, voter), data = (id, support)
pub fn emit_voted(env: &Env, id: u32, voter: &Address, support: bool) {
    let topics = (Symbol::new(env, "voted"), voter.clone());
    env.events().publish(topics, (id, support));
}

/// Emit a finalized event: topics = (finalized, caller), data = (id, passed)
pub fn emit_finalized(env: &Env, id: u32, caller: &Address, passed: bool) {
    let topics = (Symbol::new(env, "finalized"), caller.clone());
    env.events().publish(topics, (id, passed));
}
