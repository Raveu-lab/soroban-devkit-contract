//! Shared data types for the escrow contract.

use soroban_sdk::{contracttype, Address};

/// Lifecycle state of a single escrow.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    /// Funds are held, awaiting release, refund, or a dispute.
    Active,
    /// A dispute has been raised — only the arbiter can now release or refund.
    Disputed,
    /// Funds have been paid out to the recipient. Terminal.
    Released,
    /// Funds have been returned to the depositor. Terminal.
    Refunded,
}

/// A single escrow's full state.
#[contracttype]
#[derive(Clone)]
pub struct Escrow {
    pub depositor: Address,
    pub recipient: Address,
    pub arbiter: Address,
    pub token: Address,
    pub amount: i128,
    /// Unix timestamp (seconds) after which anyone may trigger release() —
    /// before this, only the depositor can release (voluntarily, early).
    pub release_time: u64,
    pub status: EscrowStatus,
}
