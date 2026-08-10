//! Error codes for the access-control contract.
//!
//! All error variants are assigned a stable u32 discriminant.
//! Never reuse or reorder existing values — doing so breaks on-chain clients.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AccessControlError {
    /// The caller does not hold the super admin role
    NotSuperAdmin = 1,
    /// The caller does not hold the admin role for the target role
    NotRoleAdmin = 2,
    /// No admin role has been set for the target role
    RoleAdminNotSet = 3,
    /// initialize() has already been called on this contract
    AlreadyInitialized = 4,
}
