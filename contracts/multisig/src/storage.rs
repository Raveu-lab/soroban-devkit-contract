//! Ledger storage helpers for the multisig contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use crate::types::Proposal;
use soroban_sdk::{Address, Env, Symbol, Vec};

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&Symbol::new(env, "Threshold"))
}

pub fn set_signers(env: &Env, signers: &Vec<Address>) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Signers"), signers);
}

pub fn get_signers(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "Signers"))
        .unwrap()
}

pub fn set_threshold(env: &Env, threshold: u32) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Threshold"), &threshold);
}

pub fn get_threshold(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "Threshold"))
        .unwrap()
}

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

pub fn set_approval(env: &Env, proposal_id: u32, signer: &Address, value: bool) {
    let key = (Symbol::new(env, "A"), proposal_id, signer.clone());
    env.storage().persistent().set(&key, &value);
}

pub fn count_approvals(env: &Env, proposal_id: u32) -> u32 {
    let signers = get_signers(env);
    let mut count = 0u32;
    for signer in signers.iter() {
        let key = (Symbol::new(env, "A"), proposal_id, signer.clone());
        let approved: bool = env.storage().persistent().get(&key).unwrap_or(false);
        if approved {
            count += 1;
        }
    }
    count
}

pub fn require_signer(env: &Env, addr: &Address) {
    let signers = get_signers(env);
    for s in signers.iter() {
        if &s == addr {
            return;
        }
    }
    panic!("not a signer");
}
