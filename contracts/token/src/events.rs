//! Typed event emission helpers for the token contract.
//!
//! All event emission goes through this module.
//! Never call `env.events().publish()` directly in `lib.rs`.

use soroban_sdk::{Address, Env, Symbol};

/// Emit a transfer event: topics = (transfer, from, to), data = amount
pub fn emit_transfer(env: &Env, from: &Address, to: &Address, amount: i128) {
    let topics = (Symbol::new(env, "transfer"), from.clone(), to.clone());
    env.events().publish(topics, amount);
}

/// Emit a mint event: topics = (mint, to), data = amount
pub fn emit_mint(env: &Env, to: &Address, amount: i128) {
    let topics = (Symbol::new(env, "mint"), to.clone());
    env.events().publish(topics, amount);
}

/// Emit an approve event: topics = (approve, from, spender), data = (amount, expiry)
pub fn emit_approve(env: &Env, from: &Address, spender: &Address, amount: i128, expiry: u32) {
    let topics = (Symbol::new(env, "approve"), from.clone(), spender.clone());
    env.events().publish(topics, (amount, expiry));
}

/// Emit a burn event: topics = (burn, from), data = amount
pub fn emit_burn(env: &Env, from: &Address, amount: i128) {
    let topics = (Symbol::new(env, "burn"), from.clone());
    env.events().publish(topics, amount);
}
