//! Time-locked escrow contract with dispute resolution.
//!
//! A depositor locks funds for a recipient, naming an arbiter to resolve
//! disputes. The depositor can release early at any time; once
//! `release_time` has passed, anyone can trigger the release. Either party
//! can raise a dispute before release, after which only the arbiter can
//! release or refund.
//!
//! # Architecture
//! - `lib.rs`          — public contract interface only
//! - `storage.rs`       — all ledger reads and writes
//! - `events.rs`        — all event emission
//! - `errors.rs`        — error enum
//! - `types.rs`         — shared data types
//! - `token_client.rs`  — minimal cross-contract client for token transfers

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

mod errors;
mod events;
mod storage;
mod token_client;
mod types;

pub use errors::EscrowError;
pub use types::{Escrow, EscrowStatus};

use token_client::TokenClient;

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Deposit funds into a new escrow. Depositor must authorize; the
    /// contract pulls `amount` of `token` from the depositor's balance
    /// into its own custody. Returns the new escrow's ID.
    pub fn deposit(
        env: Env,
        depositor: Address,
        recipient: Address,
        arbiter: Address,
        token: Address,
        amount: i128,
        release_time: u64,
    ) -> u32 {
        depositor.require_auth();
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&depositor, &env.current_contract_address(), &amount);

        let id = storage::get_escrow_count(&env);
        let escrow = Escrow {
            depositor: depositor.clone(),
            recipient: recipient.clone(),
            arbiter,
            token,
            amount,
            release_time,
            status: EscrowStatus::Active,
        };
        storage::set_escrow(&env, id, &escrow);
        storage::set_escrow_count(&env, id + 1);
        events::emit_deposited(&env, id, &depositor, &recipient, amount);
        id
    }

    /// Release escrowed funds to the recipient.
    /// - If the escrow is Disputed, only the arbiter may call this.
    /// - Otherwise, the depositor may release at any time (voluntary, early
    ///   release); anyone may call it once release_time has passed.
    pub fn release(env: Env, caller: Address, escrow_id: u32) {
        caller.require_auth();
        let mut escrow = storage::get_escrow(&env, escrow_id);

        match escrow.status {
            EscrowStatus::Disputed => {
                if caller != escrow.arbiter {
                    panic!("only the arbiter can resolve a disputed escrow");
                }
            }
            EscrowStatus::Active => {
                let unlocked = env.ledger().timestamp() >= escrow.release_time;
                if caller != escrow.depositor && !unlocked {
                    panic!("too early — only the depositor can release before release_time");
                }
            }
            _ => panic!("escrow is not active"),
        }

        escrow.status = EscrowStatus::Released;
        storage::set_escrow(&env, escrow_id, &escrow);

        let token_client = TokenClient::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.recipient,
            &escrow.amount,
        );

        events::emit_released(&env, escrow_id, &caller);
    }

    /// Refund escrowed funds back to the depositor.
    /// - If the escrow is Disputed, only the arbiter may call this.
    /// - Otherwise, only the recipient may voluntarily refund (give back).
    pub fn refund(env: Env, caller: Address, escrow_id: u32) {
        caller.require_auth();
        let mut escrow = storage::get_escrow(&env, escrow_id);

        match escrow.status {
            EscrowStatus::Disputed => {
                if caller != escrow.arbiter {
                    panic!("only the arbiter can resolve a disputed escrow");
                }
            }
            EscrowStatus::Active => {
                if caller != escrow.recipient {
                    panic!("only the recipient can voluntarily refund an active escrow");
                }
            }
            _ => panic!("escrow is not active"),
        }

        escrow.status = EscrowStatus::Refunded;
        storage::set_escrow(&env, escrow_id, &escrow);

        let token_client = TokenClient::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.depositor,
            &escrow.amount,
        );

        events::emit_refunded(&env, escrow_id, &caller);
    }

    /// Raise a dispute on an active escrow. Only the depositor or recipient
    /// may call this. Once disputed, only the arbiter can release or refund.
    pub fn dispute(env: Env, caller: Address, escrow_id: u32) {
        caller.require_auth();
        let mut escrow = storage::get_escrow(&env, escrow_id);

        if !matches!(escrow.status, EscrowStatus::Active) {
            panic!("escrow is not active");
        }
        if caller != escrow.depositor && caller != escrow.recipient {
            panic!("only the depositor or recipient can raise a dispute");
        }

        escrow.status = EscrowStatus::Disputed;
        storage::set_escrow(&env, escrow_id, &escrow);
        events::emit_disputed(&env, escrow_id, &caller);
    }

    /// Return the full state of an escrow.
    pub fn get_escrow(env: Env, escrow_id: u32) -> Escrow {
        storage::get_escrow(&env, escrow_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::storage::Instance as _, testutils::storage::Persistent as _,
        testutils::Address as _, testutils::Ledger as _, Env, String,
    };
    use soroban_token::{TokenContract, TokenContractClient};

    struct Setup {
        contract_id: Address,
        client: EscrowContractClient<'static>,
        token_id: Address,
        token_client: TokenContractClient<'static>,
        depositor: Address,
        recipient: Address,
        arbiter: Address,
    }

    fn setup(env: &Env) -> Setup {
        env.mock_all_auths();

        let contract_id = env.register(EscrowContract, ());
        let client = EscrowContractClient::new(env, &contract_id);

        let token_admin = Address::generate(env);
        let token_id = env.register(TokenContract, ());
        let token_client = TokenContractClient::new(env, &token_id);
        token_client.initialize(
            &token_admin,
            &String::from_str(env, "DevKit Token"),
            &String::from_str(env, "DKT"),
            &7,
            &false,
        );

        let depositor = Address::generate(env);
        let recipient = Address::generate(env);
        let arbiter = Address::generate(env);
        token_client.mint(&depositor, &1_000_000);

        Setup {
            contract_id,
            client,
            token_id,
            token_client,
            depositor,
            recipient,
            arbiter,
        }
    }

    #[test]
    fn test_deposit_extends_the_instance_ttl() {
        // A gap missed by the earlier audit of this repo's instance-storage
        // contracts: Count lives in instance storage, unmanaged, same as
        // the already-fixed oracle/access-control/token/multisig/
        // upgradeable/dao-voting. Losing the instance to archival would
        // make the whole contract inoperable, not just reset the counter.
        let env = Env::default();
        let s = setup(&env);

        s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        let (ttl, max_ttl) = env.as_contract(&s.contract_id, || {
            (env.storage().instance().get_ttl(), env.storage().max_ttl())
        });
        assert!(
            ttl > max_ttl / 2,
            "expected deposit to extend the instance TTL close to max_ttl ({max_ttl}), got {ttl}"
        );
    }

    #[test]
    fn test_deposit_refreshes_the_instance_ttl_of_an_aging_contract() {
        let env = Env::default();
        let s = setup(&env);

        s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &100_000,
            &1_000,
        );

        let initial_ttl = env.as_contract(&s.contract_id, || env.storage().instance().get_ttl());

        env.ledger()
            .set_sequence_number(env.ledger().sequence() + initial_ttl / 2);

        let ttl_before_write =
            env.as_contract(&s.contract_id, || env.storage().instance().get_ttl());
        assert!(
            ttl_before_write < initial_ttl,
            "sanity check: instance TTL should have visibly decreased after advancing the ledger"
        );

        s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &100_000,
            &1_000,
        );

        let ttl_after_write =
            env.as_contract(&s.contract_id, || env.storage().instance().get_ttl());
        assert!(
            ttl_after_write > ttl_before_write,
            "deposit() (which reads/writes the escrow Count) should refresh the instance TTL too"
        );
    }

    #[test]
    fn test_deposit_extends_the_persistent_entry_ttl() {
        // Same class of gap fixed in oracle/access-control/dao-voting: an
        // escrow entry that never has its TTL extended eventually gets
        // archived from disuse, even though nothing about it changed.
        let env = Env::default();
        let s = setup(&env);

        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        let (ttl, max_ttl) = env.as_contract(&s.contract_id, || {
            (
                env.storage()
                    .persistent()
                    .get_ttl(&(soroban_sdk::Symbol::new(&env, "E"), id)),
                env.storage().max_ttl(),
            )
        });
        assert!(
            ttl > max_ttl / 2,
            "expected deposit() to extend the TTL close to max_ttl ({max_ttl}), got {ttl}"
        );
    }

    #[test]
    fn test_get_escrow_refreshes_the_ttl_of_an_aging_entry() {
        let env = Env::default();
        let s = setup(&env);
        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );
        let key = (soroban_sdk::Symbol::new(&env, "E"), id);

        let initial_ttl =
            env.as_contract(&s.contract_id, || env.storage().persistent().get_ttl(&key));

        env.ledger()
            .set_sequence_number(env.ledger().sequence() + initial_ttl / 2);

        let ttl_before_read =
            env.as_contract(&s.contract_id, || env.storage().persistent().get_ttl(&key));
        assert!(
            ttl_before_read < initial_ttl,
            "sanity check: TTL should have visibly decreased after advancing the ledger"
        );

        s.client.get_escrow(&id);

        let ttl_after_read =
            env.as_contract(&s.contract_id, || env.storage().persistent().get_ttl(&key));
        assert!(
            ttl_after_read > ttl_before_read,
            "get_escrow() should refresh the entry's TTL on read, not just on write"
        );
    }

    #[test]
    fn test_deposit_creates_active_escrow_and_moves_real_balance() {
        let env = Env::default();
        let s = setup(&env);

        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        let escrow = s.client.get_escrow(&id);
        assert_eq!(escrow.status, EscrowStatus::Active);
        assert_eq!(escrow.amount, 400_000);
        assert_eq!(escrow.depositor, s.depositor);
        assert_eq!(escrow.recipient, s.recipient);

        assert_eq!(s.token_client.balance(&s.depositor), 600_000);
        assert_eq!(s.token_client.balance(&s.contract_id), 400_000);
    }

    #[test]
    fn test_depositor_can_release_early() {
        let env = Env::default();
        let s = setup(&env);
        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        s.client.release(&s.depositor, &id);

        assert_eq!(s.client.get_escrow(&id).status, EscrowStatus::Released);
        assert_eq!(s.token_client.balance(&s.recipient), 400_000);
        assert_eq!(s.token_client.balance(&s.contract_id), 0);
    }

    #[test]
    #[should_panic(expected = "too early")]
    fn test_non_depositor_cannot_release_before_timelock() {
        let env = Env::default();
        let s = setup(&env);
        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        s.client.release(&s.recipient, &id);
    }

    #[test]
    fn test_anyone_can_release_after_timelock() {
        let env = Env::default();
        let s = setup(&env);
        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        env.ledger().set_timestamp(1_001);
        s.client.release(&s.recipient, &id);

        assert_eq!(s.client.get_escrow(&id).status, EscrowStatus::Released);
        assert_eq!(s.token_client.balance(&s.recipient), 400_000);
    }

    #[test]
    fn test_recipient_can_refund_voluntarily() {
        let env = Env::default();
        let s = setup(&env);
        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        s.client.refund(&s.recipient, &id);

        assert_eq!(s.client.get_escrow(&id).status, EscrowStatus::Refunded);
        assert_eq!(s.token_client.balance(&s.depositor), 1_000_000);
    }

    #[test]
    #[should_panic(expected = "only the recipient")]
    fn test_depositor_cannot_refund_active_escrow() {
        let env = Env::default();
        let s = setup(&env);
        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        s.client.refund(&s.depositor, &id);
    }

    #[test]
    fn test_dispute_then_arbiter_resolves_release() {
        let env = Env::default();
        let s = setup(&env);
        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        s.client.dispute(&s.depositor, &id);
        assert_eq!(s.client.get_escrow(&id).status, EscrowStatus::Disputed);

        // Before the timelock, and not the depositor/recipient path — only
        // works because the arbiter overrides the normal rules once disputed.
        s.client.release(&s.arbiter, &id);
        assert_eq!(s.client.get_escrow(&id).status, EscrowStatus::Released);
        assert_eq!(s.token_client.balance(&s.recipient), 400_000);
    }

    #[test]
    fn test_dispute_then_arbiter_resolves_refund() {
        let env = Env::default();
        let s = setup(&env);
        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        s.client.dispute(&s.recipient, &id);
        s.client.refund(&s.arbiter, &id);

        assert_eq!(s.client.get_escrow(&id).status, EscrowStatus::Refunded);
        assert_eq!(s.token_client.balance(&s.depositor), 1_000_000);
    }

    #[test]
    #[should_panic(expected = "only the arbiter")]
    fn test_non_arbiter_cannot_release_disputed_escrow() {
        let env = Env::default();
        let s = setup(&env);
        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        s.client.dispute(&s.depositor, &id);
        s.client.release(&s.depositor, &id);
    }

    #[test]
    #[should_panic(expected = "only the depositor or recipient")]
    fn test_stranger_cannot_raise_dispute() {
        let env = Env::default();
        let s = setup(&env);
        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        s.client.dispute(&s.arbiter, &id);
    }

    #[test]
    #[should_panic(expected = "escrow is not active")]
    fn test_cannot_release_twice() {
        let env = Env::default();
        let s = setup(&env);
        let id = s.client.deposit(
            &s.depositor,
            &s.recipient,
            &s.arbiter,
            &s.token_id,
            &400_000,
            &1_000,
        );

        s.client.release(&s.depositor, &id);
        s.client.release(&s.depositor, &id);
    }

    #[test]
    #[should_panic(expected = "escrow not found")]
    fn test_get_unknown_escrow_panics() {
        let env = Env::default();
        let s = setup(&env);
        s.client.get_escrow(&999);
    }
}
