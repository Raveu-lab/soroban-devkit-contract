//! Shared data types for the vesting contract.

use soroban_sdk::{contracttype, Address};

/// A single linear vesting schedule with a cliff.
#[contracttype]
#[derive(Clone)]
pub struct VestingSchedule {
    pub depositor: Address,
    pub beneficiary: Address,
    pub token: Address,
    /// Total amount to be vested over the full schedule.
    pub total_amount: i128,
    /// Amount already transferred to the beneficiary (via claim or revoke).
    pub claimed_amount: i128,
    /// Unix timestamp (seconds) the schedule starts accruing from.
    pub start_time: u64,
    /// Seconds after start_time before anything is claimable, even though
    /// vesting has been accruing linearly since start_time the whole time.
    pub cliff_duration: u64,
    /// Seconds from start_time until the schedule is 100% vested.
    pub vesting_duration: u64,
    /// Set once the depositor revokes the schedule — no further claims accrue.
    pub revoked: bool,
}
