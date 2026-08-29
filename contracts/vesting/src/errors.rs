//! Error codes for the vesting contract.
//!
//! All error variants are assigned a stable u32 discriminant.
//! Never reuse or reorder existing values — doing so breaks on-chain clients.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VestingError {
    /// No vesting schedule exists with the given ID
    ScheduleNotFound = 1,
    /// The caller is not authorized to perform this action on this schedule
    Unauthorized = 2,
    /// The schedule has already been revoked
    AlreadyRevoked = 3,
}
