//! Error codes for the token contract.
//!
//! All error variants are assigned a stable u32 discriminant.
//! Never reuse or reorder existing values — doing so breaks on-chain clients.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenError {
    /// The caller's balance is too low for the requested transfer
    InsufficientBalance = 1,
    /// The spender's allowance is too low for the requested transfer_from
    InsufficientAllowance = 2,
    /// The caller does not hold the admin role
    NotAdmin = 3,
    /// initialize() has already been called on this contract
    AlreadyInitialized = 4,
    /// clawback was called but the clawback_enabled flag is false
    ClawbackNotEnabled = 5,
}
