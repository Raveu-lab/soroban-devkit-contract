//! Error codes for the escrow contract.
//!
//! All error variants are assigned a stable u32 discriminant.
//! Never reuse or reorder existing values — doing so breaks on-chain clients.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    /// No escrow exists with the given ID
    EscrowNotFound = 1,
    /// The escrow is not in a state that allows this action
    NotActive = 2,
    /// The caller is not authorized to perform this action on this escrow
    Unauthorized = 3,
    /// release() was called by a non-depositor before release_time
    TooEarly = 4,
}
