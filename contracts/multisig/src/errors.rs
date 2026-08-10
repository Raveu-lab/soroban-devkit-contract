//! Error codes for the multisig contract.
//!
//! All error variants are assigned a stable u32 discriminant.
//! Never reuse or reorder existing values — doing so breaks on-chain clients.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MultisigError {
    /// initialize() has already been called on this contract
    AlreadyInitialized = 1,
    /// threshold is zero or greater than the number of signers
    InvalidThreshold = 2,
    /// The caller is not in the signers list
    NotSigner = 3,
    /// No proposal exists with the given ID
    ProposalNotFound = 4,
    /// The proposal has already been executed
    AlreadyExecuted = 5,
    /// Not enough signers have approved the proposal yet
    InsufficientApprovals = 6,
}
