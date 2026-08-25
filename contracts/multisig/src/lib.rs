//! M-of-N multi-signature wallet contract.
//!
//! Any signer can propose a transfer. Once M signers approve,
//! any signer can execute it. Executed proposals are immutable.

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

mod errors;
mod events;
mod storage;
mod token_client;
mod types;

pub use errors::MultisigError;
pub use types::Proposal;

use token_client::TokenClient;

#[contract]
pub struct MultisigContract;

#[contractimpl]
impl MultisigContract {
    /// Initialize with a list of signers and a threshold.
    pub fn initialize(env: Env, signers: Vec<Address>, threshold: u32) {
        if storage::is_initialized(&env) {
            panic!("already initialized");
        }
        if threshold == 0 || threshold as usize > signers.len() as usize {
            panic!("invalid threshold");
        }
        storage::set_signers(&env, &signers);
        storage::set_threshold(&env, threshold);
        storage::set_proposal_count(&env, 0);
    }

    /// Propose a token transfer. Any signer can propose.
    /// Returns the proposal ID.
    pub fn propose(env: Env, proposer: Address, to: Address, amount: i128, token: Address) -> u32 {
        proposer.require_auth();
        storage::require_signer(&env, &proposer);

        let id = storage::get_proposal_count(&env);
        let proposal = Proposal {
            to: to.clone(),
            amount,
            token: token.clone(),
            executed: false,
        };
        storage::set_proposal(&env, id, &proposal);
        storage::set_proposal_count(&env, id + 1);
        events::emit_proposed(&env, id, &proposer, &to, amount);
        id
    }

    /// Approve a proposal. Each signer can approve once.
    pub fn approve(env: Env, signer: Address, proposal_id: u32) {
        signer.require_auth();
        storage::require_signer(&env, &signer);

        let proposal = storage::get_proposal(&env, proposal_id);
        if proposal.executed {
            panic!("proposal already executed");
        }
        storage::set_approval(&env, proposal_id, &signer, true);
        events::emit_approved(&env, proposal_id, &signer);
    }

    /// Execute a proposal once threshold approvals are met.
    pub fn execute(env: Env, executor: Address, proposal_id: u32) {
        executor.require_auth();
        storage::require_signer(&env, &executor);

        let mut proposal = storage::get_proposal(&env, proposal_id);
        if proposal.executed {
            panic!("already executed");
        }

        let approval_count = storage::count_approvals(&env, proposal_id);
        let threshold = storage::get_threshold(&env);
        if approval_count < threshold {
            panic!("insufficient approvals");
        }

        proposal.executed = true;
        storage::set_proposal(&env, proposal_id, &proposal);

        let token = TokenClient::new(&env, &proposal.token);
        token.transfer(
            &env.current_contract_address(),
            &proposal.to,
            &proposal.amount,
        );

        events::emit_executed(&env, proposal_id, &executor);
    }

    /// Return approval count for a proposal.
    pub fn approval_count(env: Env, proposal_id: u32) -> u32 {
        storage::count_approvals(&env, proposal_id)
    }

    /// Return the configured threshold.
    pub fn threshold(env: Env) -> u32 {
        storage::get_threshold(&env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, Vec};

    fn setup(env: &Env, n: u32, threshold: u32) -> (Vec<Address>, Address) {
        let contract_id = env.register(MultisigContract, ());
        let mut signers = Vec::new(env);
        for _ in 0..n {
            signers.push_back(Address::generate(env));
        }
        let client = MultisigContractClient::new(env, &contract_id);
        client.initialize(&signers, &threshold);
        (signers, contract_id)
    }

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        let (_, contract_id) = setup(&env, 3, 2);
        let client = MultisigContractClient::new(&env, &contract_id);
        assert_eq!(client.threshold(), 2);
    }

    #[test]
    fn test_execute_transfers_real_token_balance() {
        use soroban_sdk::String;
        use soroban_token::{TokenContract, TokenContractClient};

        let env = Env::default();
        env.mock_all_auths();

        let (signers, contract_id) = setup(&env, 3, 2);
        let client = MultisigContractClient::new(&env, &contract_id);

        let token_admin = Address::generate(&env);
        let token_id = env.register(TokenContract, ());
        let token_client = TokenContractClient::new(&env, &token_id);
        token_client.initialize(
            &token_admin,
            &String::from_str(&env, "DevKit Token"),
            &String::from_str(&env, "DKT"),
            &7,
        );
        token_client.mint(&contract_id, &1_000_000);

        let recipient = Address::generate(&env);
        let proposal_id = client.propose(&signers.get(0).unwrap(), &recipient, &400_000, &token_id);
        client.approve(&signers.get(0).unwrap(), &proposal_id);
        client.approve(&signers.get(1).unwrap(), &proposal_id);

        client.execute(&signers.get(0).unwrap(), &proposal_id);

        assert_eq!(token_client.balance(&recipient), 400_000);
        assert_eq!(token_client.balance(&contract_id), 600_000);
    }

    #[test]
    #[should_panic(expected = "already executed")]
    fn test_execute_twice_panics() {
        use soroban_sdk::String;
        use soroban_token::{TokenContract, TokenContractClient};

        let env = Env::default();
        env.mock_all_auths();

        let (signers, contract_id) = setup(&env, 3, 2);
        let client = MultisigContractClient::new(&env, &contract_id);

        let token_admin = Address::generate(&env);
        let token_id = env.register(TokenContract, ());
        let token_client = TokenContractClient::new(&env, &token_id);
        token_client.initialize(
            &token_admin,
            &String::from_str(&env, "DevKit Token"),
            &String::from_str(&env, "DKT"),
            &7,
        );
        token_client.mint(&contract_id, &1_000_000);

        let recipient = Address::generate(&env);
        let proposal_id = client.propose(&signers.get(0).unwrap(), &recipient, &400_000, &token_id);
        client.approve(&signers.get(0).unwrap(), &proposal_id);
        client.approve(&signers.get(1).unwrap(), &proposal_id);

        client.execute(&signers.get(0).unwrap(), &proposal_id);
        client.execute(&signers.get(0).unwrap(), &proposal_id);
    }

    #[test]
    fn test_propose_and_approve() {
        let env = Env::default();
        env.mock_all_auths();
        let (signers, contract_id) = setup(&env, 3, 2);
        let client = MultisigContractClient::new(&env, &contract_id);

        let token = Address::generate(&env);
        let recipient = Address::generate(&env);
        let proposal_id = client.propose(&signers.get(0).unwrap(), &recipient, &1_000_000, &token);

        client.approve(&signers.get(0).unwrap(), &proposal_id);
        assert_eq!(client.approval_count(&proposal_id), 1);

        client.approve(&signers.get(1).unwrap(), &proposal_id);
        assert_eq!(client.approval_count(&proposal_id), 2);
    }
}
