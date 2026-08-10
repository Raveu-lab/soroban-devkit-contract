//! Shared data types for the multisig contract.

use soroban_sdk::{contracttype, Address};

/// A transfer proposal submitted by a signer
#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    /// Recipient address for the transfer
    pub to: Address,
    /// Token amount to transfer
    pub amount: i128,
    /// Token contract to invoke for the transfer
    pub token: Address,
    /// Whether this proposal has been executed
    pub executed: bool,
}
