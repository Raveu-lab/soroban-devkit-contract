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
    use soroban_sdk::{
        testutils::storage::Instance as _, testutils::storage::Persistent as _,
        testutils::Address as _, testutils::Ledger as _, Env,
    };

    #[test]
    fn test_propose_extends_the_instance_ttl() {
        // A gap missed by the earlier audit of this repo's instance-storage
        // contracts: Count lives in instance storage, unmanaged, same as
        // the already-fixed oracle/access-control/token/multisig/
        // upgradeable. Losing the instance to archival would make the
        // whole contract inoperable (every function needs the instance
        // live to execute at all), not just reset the proposal counter.
        let env = Env::default();
        let (proposer, client) = setup(&env);

        client.propose(
            &proposer,
            &String::from_str(&env, "Raise the fee cap"),
            &1_000,
        );

        let contract_id = client.address.clone();
        let (ttl, max_ttl) = env.as_contract(&contract_id, || {
            (env.storage().instance().get_ttl(), env.storage().max_ttl())
        });
        assert!(
            ttl > max_ttl / 2,
            "expected propose to extend the instance TTL close to max_ttl ({max_ttl}), got {ttl}"
        );
    }

    #[test]
    fn test_propose_refreshes_the_instance_ttl_of_an_aging_contract() {
        let env = Env::default();
        let (proposer, client) = setup(&env);
        let contract_id = client.address.clone();

        client.propose(&proposer, &String::from_str(&env, "First proposal"), &1_000);

        let initial_ttl = env.as_contract(&contract_id, || env.storage().instance().get_ttl());

        env.ledger()
            .set_sequence_number(env.ledger().sequence() + initial_ttl / 2);

        let ttl_before_write = env.as_contract(&contract_id, || env.storage().instance().get_ttl());
        assert!(
            ttl_before_write < initial_ttl,
            "sanity check: instance TTL should have visibly decreased after advancing the ledger"
        );

        client.propose(
            &proposer,
            &String::from_str(&env, "Second proposal"),
            &1_000,
        );

        let ttl_after_write = env.as_contract(&contract_id, || env.storage().instance().get_ttl());
        assert!(
            ttl_after_write > ttl_before_write,
            "propose() (which reads/writes the proposal Count) should refresh the instance TTL too"
        );
    }

    fn setup(env: &Env) -> (Address, DaoVotingContractClient<'_>) {
        env.mock_all_auths();
        let contract_id = env.register(DaoVotingContract, ());
        let client = DaoVotingContractClient::new(env, &contract_id);
        let proposer = Address::generate(env);
        (proposer, client)
    }

    #[test]
    fn test_propose_extends_the_persistent_entry_ttl() {
        // Same class of gap fixed in oracle/access-control: a proposal
        // entry that never has its TTL extended eventually gets archived
        // from disuse, even though the proposal itself hasn't changed.
        let env = Env::default();
        let (proposer, client) = setup(&env);

        let id = client.propose(
            &proposer,
            &String::from_str(&env, "Raise the fee cap"),
            &1_000,
        );

        let contract_id = client.address.clone();
        let (ttl, max_ttl) = env.as_contract(&contract_id, || {
            (
                env.storage()
                    .persistent()
                    .get_ttl(&(soroban_sdk::Symbol::new(&env, "P"), id)),
                env.storage().max_ttl(),
            )
        });
        assert!(
            ttl > max_ttl / 2,
            "expected propose() to extend the TTL close to max_ttl ({max_ttl}), got {ttl}"
        );
    }

    #[test]
    fn test_get_proposal_refreshes_the_ttl_of_an_aging_entry() {
        let env = Env::default();
        let (proposer, client) = setup(&env);
        let id = client.propose(
            &proposer,
            &String::from_str(&env, "Raise the fee cap"),
            &1_000,
        );
        let contract_id = client.address.clone();
        let key = (soroban_sdk::Symbol::new(&env, "P"), id);

        let initial_ttl =
            env.as_contract(&contract_id, || env.storage().persistent().get_ttl(&key));

        env.ledger()
            .set_sequence_number(env.ledger().sequence() + initial_ttl / 2);

        let ttl_before_read =
            env.as_contract(&contract_id, || env.storage().persistent().get_ttl(&key));
        assert!(
            ttl_before_read < initial_ttl,
            "sanity check: TTL should have visibly decreased after advancing the ledger"
        );

        client.get_proposal(&id);

        let ttl_after_read =
            env.as_contract(&contract_id, || env.storage().persistent().get_ttl(&key));
        assert!(
            ttl_after_read > ttl_before_read,
            "get_proposal() should refresh the entry's TTL on read, not just on write"
        );
    }

    #[test]
    fn test_vote_extends_the_persistent_vote_record_ttl() {
        let env = Env::default();
        let (proposer, client) = setup(&env);
        let voter = Address::generate(&env);
        let id = client.propose(
            &proposer,
            &String::from_str(&env, "Raise the fee cap"),
            &1_000,
        );

        client.vote(&voter, &id, &true);

        let contract_id = client.address.clone();
        let key = (soroban_sdk::Symbol::new(&env, "V"), id, voter);
        let (ttl, max_ttl) = env.as_contract(&contract_id, || {
            (
                env.storage().persistent().get_ttl(&key),
                env.storage().max_ttl(),
            )
        });
        assert!(
            ttl > max_ttl / 2,
            "expected vote() to extend the vote record's TTL close to max_ttl ({max_ttl}), got {ttl}"
        );
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
