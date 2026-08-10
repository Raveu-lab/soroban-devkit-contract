//! Typed event emission helpers for the multisig contract.
//!
//! All event emission goes through this module.
//! Never call `env.events().publish()` directly in `lib.rs`.

use soroban_sdk::{Address, Env, Symbol};

/// Emit a proposed event: topics = (proposed, proposer), data = (id, to, amount)
pub fn emit_proposed(env: &Env, id: u32, proposer: &Address, to: &Address, amount: i128) {
    let topics = (Symbol::new(env, "proposed"), proposer.clone());
    env.events().publish(topics, (id, to.clone(), amount));
}

/// Emit an approved event: topics = (approved, signer), data = proposal_id
pub fn emit_approved(env: &Env, proposal_id: u32, signer: &Address) {
    let topics = (Symbol::new(env, "approved"), signer.clone());
    env.events().publish(topics, proposal_id);
}

/// Emit an executed event: topics = (executed, executor), data = proposal_id
pub fn emit_executed(env: &Env, proposal_id: u32, executor: &Address) {
    let topics = (Symbol::new(env, "executed"), executor.clone());
    env.events().publish(topics, proposal_id);
}
