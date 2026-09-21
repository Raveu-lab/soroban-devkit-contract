//! Ledger storage helpers for the vesting contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use crate::types::VestingSchedule;
use soroban_sdk::{Env, Symbol};

/// How close to the network's max TTL a persistent entry is kept — same
/// approach as oracle/access-control/dao-voting/escrow/multisig:
/// self-adjusting to the network's actual max_ttl() rather than a fixed
/// ledger-count guess tied to an assumed close time.
const TTL_EXTEND_BUFFER: u32 = 1_000;

fn extend_schedule_ttl(env: &Env, id: u32) {
    let key = (Symbol::new(env, "V"), id);
    let max_ttl = env.storage().max_ttl();
    env.storage()
        .persistent()
        .extend_ttl(&key, max_ttl.saturating_sub(TTL_EXTEND_BUFFER), max_ttl);
}

pub fn set_schedule_count(env: &Env, count: u32) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "Count"), &count);
}

pub fn get_schedule_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "Count"))
        .unwrap_or(0)
}

pub fn set_schedule(env: &Env, id: u32, schedule: &VestingSchedule) {
    env.storage()
        .persistent()
        .set(&(Symbol::new(env, "V"), id), schedule);
    extend_schedule_ttl(env, id);
}

pub fn get_schedule(env: &Env, id: u32) -> VestingSchedule {
    let schedule = env.storage().persistent().get(&(Symbol::new(env, "V"), id));
    if schedule.is_some() {
        extend_schedule_ttl(env, id);
    }
    schedule.unwrap_or_else(|| panic!("vesting schedule not found"))
}
