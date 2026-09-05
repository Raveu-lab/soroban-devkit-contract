//! On-chain proposal and voting — one-address-one-vote governance.
//!
//! Anyone can propose. Any address can vote once per proposal, for or
//! against, before the proposal's deadline. After the deadline, anyone can
//! finalize it — simple majority: Passed if for_votes > against_votes,
//! Rejected otherwise (a tie is a Rejected, not a Passed).
//!
//! This contract only records outcomes — it doesn't execute anything itself.
//! Pairing it with `access-control` or `multisig` to actually gate an action
//! on a proposal's result is left to the composing contract.
//!
//! # Architecture
//! - `lib.rs`     — public contract interface only
//! - `storage.rs` — all ledger reads and writes
//! - `events.rs`  — all event emission
//! - `errors.rs`  — error enum
//! - `types.rs`   — shared data types

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String};

mod errors;
mod events;
mod storage;
mod types;

pub use errors::DaoVotingError;
pub use types::{Proposal, ProposalStatus};

#[contract]
pub struct DaoVotingContract;

#[contractimpl]
impl DaoVotingContract {
    /// Create a new proposal. Anyone can propose. `voting_duration` is
    /// seconds from now until the deadline. Returns the new proposal's ID.
    pub fn propose(env: Env, proposer: Address, description: String, voting_duration: u64) -> u32 {
        proposer.require_auth();

        let id = storage::get_proposal_count(&env);
        let deadline = env.ledger().timestamp() + voting_duration;
        let proposal = Proposal {
            proposer: proposer.clone(),
            description,
            deadline,
            for_votes: 0,
            against_votes: 0,
            status: ProposalStatus::Active,
        };
        storage::set_proposal(&env, id, &proposal);
        storage::set_proposal_count(&env, id + 1);
        events::emit_proposed(&env, id, &proposer, deadline);
        id
    }

    /// Cast a vote on an active proposal. Each address may vote once per
    /// proposal. Panics if the proposal isn't Active, the deadline has
    /// passed, or the caller already voted.
    pub fn vote(env: Env, voter: Address, proposal_id: u32, support: bool) {
        voter.require_auth();

        let mut proposal = storage::get_proposal(&env, proposal_id);
        if !matches!(proposal.status, ProposalStatus::Active)
            || env.ledger().timestamp() >= proposal.deadline
        {
            panic!("voting has closed");
        }
        if storage::has_voted(&env, proposal_id, &voter) {
            panic!("already voted");
        }

        if support {
            proposal.for_votes += 1;
        } else {
            proposal.against_votes += 1;
        }
        storage::set_proposal(&env, proposal_id, &proposal);
        storage::set_voted(&env, proposal_id, &voter);
        events::emit_voted(&env, proposal_id, &voter, support);
    }

    /// Finalize a proposal once its deadline has passed. Anyone may call
    /// this — it just records the already-determined outcome. Panics if
    /// the proposal isn't Active or the deadline hasn't passed yet.
    /// Returns true if the proposal passed.
    pub fn finalize(env: Env, caller: Address, proposal_id: u32) -> bool {
        caller.require_auth();

        let mut proposal = storage::get_proposal(&env, proposal_id);
        if !matches!(proposal.status, ProposalStatus::Active) {
            panic!("proposal is not active");
        }
        if env.ledger().timestamp() < proposal.deadline {
            panic!("voting is still open");
        }

        let passed = proposal.for_votes > proposal.against_votes;
        proposal.status = if passed {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        };
        storage::set_proposal(&env, proposal_id, &proposal);
        events::emit_finalized(&env, proposal_id, &caller, passed);
        passed
    }

    /// Return the full state of a proposal.
    pub fn get_proposal(env: Env, proposal_id: u32) -> Proposal {
        storage::get_proposal(&env, proposal_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Env};

    fn setup(env: &Env) -> (Address, DaoVotingContractClient<'_>) {
        env.mock_all_auths();
        let contract_id = env.register(DaoVotingContract, ());
        let client = DaoVotingContractClient::new(env, &contract_id);
        let proposer = Address::generate(env);
        (proposer, client)
    }

    #[test]
    fn test_propose_creates_active_proposal() {
        let env = Env::default();
        let (proposer, client) = setup(&env);

        let id = client.propose(
            &proposer,
            &String::from_str(&env, "Raise the fee cap"),
            &1_000,
        );

        let proposal = client.get_proposal(&id);
        assert_eq!(proposal.status, ProposalStatus::Active);
        assert_eq!(proposal.for_votes, 0);
        assert_eq!(proposal.against_votes, 0);
        assert_eq!(proposal.proposer, proposer);
    }

    #[test]
    fn test_vote_for_increments_for_votes() {
        let env = Env::default();
        let (proposer, client) = setup(&env);
        let voter = Address::generate(&env);
        let id = client.propose(&proposer, &String::from_str(&env, "Proposal"), &1_000);

        client.vote(&voter, &id, &true);

        assert_eq!(client.get_proposal(&id).for_votes, 1);
        assert_eq!(client.get_proposal(&id).against_votes, 0);
    }

    #[test]
    fn test_vote_against_increments_against_votes() {
        let env = Env::default();
        let (proposer, client) = setup(&env);
        let voter = Address::generate(&env);
        let id = client.propose(&proposer, &String::from_str(&env, "Proposal"), &1_000);

        client.vote(&voter, &id, &false);

        assert_eq!(client.get_proposal(&id).against_votes, 1);
        assert_eq!(client.get_proposal(&id).for_votes, 0);
    }

    #[test]
    #[should_panic(expected = "already voted")]
    fn test_cannot_vote_twice() {
        let env = Env::default();
        let (proposer, client) = setup(&env);
        let voter = Address::generate(&env);
        let id = client.propose(&proposer, &String::from_str(&env, "Proposal"), &1_000);

        client.vote(&voter, &id, &true);
        client.vote(&voter, &id, &false);
    }

    #[test]
    #[should_panic(expected = "voting has closed")]
    fn test_cannot_vote_after_deadline() {
        let env = Env::default();
        let (proposer, client) = setup(&env);
        let voter = Address::generate(&env);
        let id = client.propose(&proposer, &String::from_str(&env, "Proposal"), &1_000);

        env.ledger().set_timestamp(1_001);
        client.vote(&voter, &id, &true);
    }

    #[test]
    #[should_panic(expected = "voting is still open")]
    fn test_finalize_before_deadline_panics() {
        let env = Env::default();
        let (proposer, client) = setup(&env);
        let id = client.propose(&proposer, &String::from_str(&env, "Proposal"), &1_000);

        client.finalize(&proposer, &id);
    }

    #[test]
    fn test_finalize_passes_when_for_gt_against() {
        let env = Env::default();
        let (proposer, client) = setup(&env);
        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);
        let id = client.propose(&proposer, &String::from_str(&env, "Proposal"), &1_000);

        client.vote(&voter1, &id, &true);
        client.vote(&voter2, &id, &true);

        env.ledger().set_timestamp(1_001);
        let passed = client.finalize(&proposer, &id);

        assert!(passed);
        assert_eq!(client.get_proposal(&id).status, ProposalStatus::Passed);
    }

    #[test]
    fn test_finalize_rejects_on_tie() {
        let env = Env::default();
        let (proposer, client) = setup(&env);
        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);
        let id = client.propose(&proposer, &String::from_str(&env, "Proposal"), &1_000);

        client.vote(&voter1, &id, &true);
        client.vote(&voter2, &id, &false);

        env.ledger().set_timestamp(1_001);
        let passed = client.finalize(&proposer, &id);

        assert!(!passed);
        assert_eq!(client.get_proposal(&id).status, ProposalStatus::Rejected);
    }

    #[test]
    #[should_panic(expected = "voting has closed")]
    fn test_cannot_vote_after_finalize() {
        let env = Env::default();
        let (proposer, client) = setup(&env);
        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);
        let id = client.propose(&proposer, &String::from_str(&env, "Proposal"), &1_000);

        env.ledger().set_timestamp(1_001);
        client.finalize(&proposer, &id);

        client.vote(&voter1, &id, &true);
        let _ = voter2;
    }

    #[test]
    #[should_panic(expected = "not active")]
    fn test_cannot_finalize_twice() {
        let env = Env::default();
        let (proposer, client) = setup(&env);
        let id = client.propose(&proposer, &String::from_str(&env, "Proposal"), &1_000);

        env.ledger().set_timestamp(1_001);
        client.finalize(&proposer, &id);
        client.finalize(&proposer, &id);
    }

    #[test]
    #[should_panic(expected = "proposal not found")]
    fn test_get_unknown_proposal_panics() {
        let env = Env::default();
        let (_proposer, client) = setup(&env);
        client.get_proposal(&999);
    }
}
