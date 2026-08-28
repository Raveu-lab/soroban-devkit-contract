//! Ledger storage helpers for the escrow contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use crate::types::Escrow;
use soroban_sdk::{Env, Symbol};

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
}

pub fn get_escrow(env: &Env, id: u32) -> Escrow {
    env.storage()
        .persistent()
        .get(&(Symbol::new(env, "E"), id))
        .unwrap_or_else(|| panic!("escrow not found"))
}
