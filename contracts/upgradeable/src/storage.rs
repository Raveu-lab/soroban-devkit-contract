//! Ledger storage helpers for the upgradeable contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use soroban_sdk::{Address, Env, Symbol};

/// How close to the network's max TTL the instance is kept — same
/// self-adjusting approach used throughout this repo (oracle,
/// access-control, token, multisig): extended whenever remaining TTL drops
/// within this many ledgers of max_ttl(), back up to the max itself.
const TTL_EXTEND_BUFFER: u32 = 1_000;

/// Extends the whole contract instance's TTL — holds Admin and Version. The
/// most severe version of this bug in this repo: upgrade()/migrate() are
/// the only mechanism to ever fix a deployed contract, and both are
/// admin-gated. Losing the instance to archival would make them
/// permanently uncallable, with no way to recover.
fn extend_instance_ttl(env: &Env) {
    let max_ttl = env.storage().max_ttl();
    env.storage()
        .instance()
        .extend_ttl(max_ttl.saturating_sub(TTL_EXTEND_BUFFER), max_ttl);
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Admin"), admin);
    extend_instance_ttl(env);
}

pub fn get_admin(env: &Env) -> Address {
    let admin = env
        .storage()
        .instance()
        .get(&Symbol::new(env, "Admin"))
        .unwrap();
    extend_instance_ttl(env);
    admin
}

pub fn set_version(env: &Env, version: u32) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Version"), &version);
    extend_instance_ttl(env);
}

pub fn get_version(env: &Env) -> u32 {
    let version = env
        .storage()
        .instance()
        .get(&Symbol::new(env, "Version"))
        .unwrap_or(0);
    extend_instance_ttl(env);
    version
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&Symbol::new(env, "Admin"))
}
