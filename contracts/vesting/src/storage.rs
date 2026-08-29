//! Ledger storage helpers for the vesting contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use crate::types::VestingSchedule;
use soroban_sdk::{Env, Symbol};

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
}

pub fn get_schedule(env: &Env, id: u32) -> VestingSchedule {
    env.storage()
        .persistent()
        .get(&(Symbol::new(env, "V"), id))
        .unwrap_or_else(|| panic!("vesting schedule not found"))
}
