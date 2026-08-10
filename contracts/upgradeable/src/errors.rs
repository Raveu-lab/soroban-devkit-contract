//! Error codes for the upgradeable contract.
//!
//! All error variants are assigned a stable u32 discriminant.
//! Never reuse or reorder existing values — doing so breaks on-chain clients.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum UpgradeableError {
    /// initialize() has already been called on this contract
    AlreadyInitialized = 1,
    /// The caller does not hold the admin role
    NotAdmin = 2,
    /// The migration function failed to complete
    MigrationFailed = 3,
}
