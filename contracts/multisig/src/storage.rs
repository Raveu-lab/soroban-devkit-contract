//! Ledger storage helpers for the multisig contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use crate::types::Proposal;
use soroban_sdk::{Address, Env, IntoVal, Symbol, Val, Vec};

/// How close to the network's max TTL a persistent entry is kept — same
/// approach as oracle/access-control/dao-voting/escrow: self-adjusting to
/// the network's actual max_ttl() rather than a fixed ledger-count guess
/// tied to an assumed close time.
const TTL_EXTEND_BUFFER: u32 = 1_000;

fn extend_persistent_ttl<K: IntoVal<Env, Val>>(env: &Env, key: &K) {
    let max_ttl = env.storage().max_ttl();
    env.storage()
        .persistent()
        .extend_ttl(key, max_ttl.saturating_sub(TTL_EXTEND_BUFFER), max_ttl);
}

/// Extends the whole contract instance's TTL — holds Signers, Threshold,
/// and Count. Losing it to archival would make every function inoperable
/// (require_signer reads Signers on every proposal/approval/execution),
/// not just one proposal — same class of fix already applied in
/// oracle/access-control/token.
fn extend_instance_ttl(env: &Env) {
    let max_ttl = env.storage().max_ttl();
    env.storage()
        .instance()
        .extend_ttl(max_ttl.saturating_sub(TTL_EXTEND_BUFFER), max_ttl);
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&Symbol::new(env, "Threshold"))
}

pub fn set_signers(env: &Env, signers: &Vec<Address>) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Signers"), signers);
    extend_instance_ttl(env);
}

pub fn get_signers(env: &Env) -> Vec<Address> {
    let signers = env
        .storage()
        .instance()
        .get(&Symbol::new(env, "Signers"))
        .unwrap();
    extend_instance_ttl(env);
    signers
}

pub fn set_threshold(env: &Env, threshold: u32) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Threshold"), &threshold);
    extend_instance_ttl(env);
}

pub fn get_threshold(env: &Env) -> u32 {
    let threshold = env
        .storage()
        .instance()
        .get(&Symbol::new(env, "Threshold"))
        .unwrap();
    extend_instance_ttl(env);
    threshold
}

pub fn set_proposal_count(env: &Env, count: u32) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Count"), &count);
    extend_instance_ttl(env);
}

pub fn get_proposal_count(env: &Env) -> u32 {
    let count = env
        .storage()
        .instance()
        .get(&Symbol::new(env, "Count"))
        .unwrap_or(0);
    extend_instance_ttl(env);
    count
}

pub fn set_proposal(env: &Env, id: u32, proposal: &Proposal) {
    let key = (Symbol::new(env, "P"), id);
    env.storage().persistent().set(&key, proposal);
    extend_persistent_ttl(env, &key);
}

pub fn get_proposal(env: &Env, id: u32) -> Proposal {
    let key = (Symbol::new(env, "P"), id);
    let proposal = env.storage().persistent().get(&key);
    if proposal.is_some() {
        extend_persistent_ttl(env, &key);
    }
    proposal.unwrap_or_else(|| panic!("proposal not found"))
}

pub fn set_approval(env: &Env, proposal_id: u32, signer: &Address, value: bool) {
    let key = (Symbol::new(env, "A"), proposal_id, signer.clone());
    env.storage().persistent().set(&key, &value);
    extend_persistent_ttl(env, &key);
}

pub fn count_approvals(env: &Env, proposal_id: u32) -> u32 {
    let signers = get_signers(env);
    let mut count = 0u32;
    for signer in signers.iter() {
        let key = (Symbol::new(env, "A"), proposal_id, signer.clone());
        let approved: Option<bool> = env.storage().persistent().get(&key);
        if approved.is_some() {
            extend_persistent_ttl(env, &key);
        }
        if approved.unwrap_or(false) {
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
