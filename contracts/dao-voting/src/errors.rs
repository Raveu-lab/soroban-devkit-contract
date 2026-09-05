//! Error codes for the dao-voting contract.
//!
//! All error variants are assigned a stable u32 discriminant.
//! Never reuse or reorder existing values — doing so breaks on-chain clients.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DaoVotingError {
    /// No proposal exists with the given ID
    ProposalNotFound = 1,
    /// The caller already voted on this proposal
    AlreadyVoted = 2,
    /// The proposal is not Active (already finalized)
    NotActive = 3,
    /// Voting or finalizing was attempted at the wrong time relative to the deadline
    WrongPhase = 4,
}
