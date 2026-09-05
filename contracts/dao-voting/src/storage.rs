//! Ledger storage helpers for the dao-voting contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use crate::types::Proposal;
use soroban_sdk::{Address, Env, Symbol};

pub fn set_proposal_count(env: &Env, count: u32) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Count"), &count);
}

pub fn get_proposal_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "Count"))
        .unwrap_or(0)
}

pub fn set_proposal(env: &Env, id: u32, proposal: &Proposal) {
    env.storage()
        .persistent()
        .set(&(Symbol::new(env, "P"), id), proposal);
}

pub fn get_proposal(env: &Env, id: u32) -> Proposal {
    env.storage()
        .persistent()
        .get(&(Symbol::new(env, "P"), id))
        .unwrap_or_else(|| panic!("proposal not found"))
}

pub fn has_voted(env: &Env, id: u32, voter: &Address) -> bool {
    let key = (Symbol::new(env, "V"), id, voter.clone());
    env.storage().persistent().get(&key).unwrap_or(false)
}

pub fn set_voted(env: &Env, id: u32, voter: &Address) {
    let key = (Symbol::new(env, "V"), id, voter.clone());
    env.storage().persistent().set(&key, &true);
}
