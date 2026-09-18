//! Ledger storage helpers for the escrow contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use crate::types::Escrow;
use soroban_sdk::{Env, Symbol};

/// How close to the network's max TTL a persistent entry is kept — same
/// approach as oracle/access-control/dao-voting: self-adjusting to the
/// network's actual max_ttl() rather than a fixed ledger-count guess tied
/// to an assumed close time.
const TTL_EXTEND_BUFFER: u32 = 1_000;

fn extend_escrow_ttl(env: &Env, id: u32) {
    let key = (Symbol::new(env, "E"), id);
    let max_ttl = env.storage().max_ttl();
    env.storage()
        .persistent()
        .extend_ttl(&key, max_ttl.saturating_sub(TTL_EXTEND_BUFFER), max_ttl);
}

pub fn set_escrow_count(env: &Env, count: u32) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Count"), &count);
}

pub fn get_escrow_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "Count"))
        .unwrap_or(0)
}

pub fn set_escrow(env: &Env, id: u32, escrow: &Escrow) {
    env.storage()
        .persistent()
        .set(&(Symbol::new(env, "E"), id), escrow);
    extend_escrow_ttl(env, id);
}

pub fn get_escrow(env: &Env, id: u32) -> Escrow {
    let escrow = env.storage().persistent().get(&(Symbol::new(env, "E"), id));
    if escrow.is_some() {
        extend_escrow_ttl(env, id);
    }
    escrow.unwrap_or_else(|| panic!("escrow not found"))
}
