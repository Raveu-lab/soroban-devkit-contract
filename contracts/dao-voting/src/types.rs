//! Shared data types for the dao-voting contract.

use soroban_sdk::{contracttype, Address, String};

/// Lifecycle state of a single proposal.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    /// Voting is open — before `deadline`, not yet finalized.
    Active,
    /// Finalized with more for-votes than against-votes.
    Passed,
    /// Finalized with for-votes <= against-votes.
    Rejected,
}

/// A single governance proposal.
#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub proposer: Address,
    pub description: String,
    /// Unix timestamp (seconds) after which voting closes and finalize() is allowed.
    pub deadline: u64,
    pub for_votes: u32,
    pub against_votes: u32,
    pub status: ProposalStatus,
}
