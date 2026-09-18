//! Ledger storage helpers for the dao-voting contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use crate::types::Proposal;
use soroban_sdk::{Address, Env, IntoVal, Symbol, Val};

/// How close to the network's max TTL a persistent entry is kept —
/// extended whenever its remaining TTL drops within this many ledgers of
/// that max, back up to the max itself. Same approach as oracle's
/// extend_price_ttl / access-control's extend_role_ttl: self-adjusting to
/// the network's actual max_ttl() rather than a fixed ledger-count guess
/// tied to an assumed close time.
const TTL_EXTEND_BUFFER: u32 = 1_000;

fn extend_persistent_ttl<K: IntoVal<Env, Val>>(env: &Env, key: &K) {
    let max_ttl = env.storage().max_ttl();
    env.storage()
        .persistent()
        .extend_ttl(key, max_ttl.saturating_sub(TTL_EXTEND_BUFFER), max_ttl);
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

pub fn has_voted(env: &Env, id: u32, voter: &Address) -> bool {
    let key = (Symbol::new(env, "V"), id, voter.clone());
    let voted = env.storage().persistent().get(&key);
    if voted.is_some() {
        extend_persistent_ttl(env, &key);
    }
    voted.unwrap_or(false)
}

pub fn set_voted(env: &Env, id: u32, voter: &Address) {
    let key = (Symbol::new(env, "V"), id, voter.clone());
    env.storage().persistent().set(&key, &true);
    extend_persistent_ttl(env, &key);
}
