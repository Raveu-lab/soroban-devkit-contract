//! Linear token vesting with a cliff.
//!
//! A depositor locks tokens for a beneficiary, vesting linearly from
//! `start_time` over `vesting_duration` seconds. Nothing is claimable until
//! `cliff_duration` has passed, even though vesting has been accruing the
//! whole time — once the cliff passes, whatever has accrued becomes
//! claimable at once. The depositor may revoke at any time before full
//! vesting: the beneficiary is paid whatever has vested so far, and the
//! unvested remainder is refunded to the depositor.
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

pub use errors::VestingError;
pub use types::VestingSchedule;

use token_client::TokenClient;

#[contract]
pub struct VestingContract;

#[contractimpl]
impl VestingContract {
    /// Create a new vesting schedule. Depositor must authorize; the
    /// contract pulls `total_amount` of `token` from the depositor's
    /// balance into its own custody. Returns the new schedule's ID.
    #[allow(clippy::too_many_arguments)]
    pub fn create_vesting(
        env: Env,
        depositor: Address,
        beneficiary: Address,
        token: Address,
        total_amount: i128,
        start_time: u64,
        cliff_duration: u64,
        vesting_duration: u64,
    ) -> u32 {
        depositor.require_auth();
        if total_amount <= 0 {
            panic!("total_amount must be positive");
        }
        if vesting_duration == 0 {
            panic!("vesting_duration must be positive");
        }

        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&depositor, &env.current_contract_address(), &total_amount);

        let id = storage::get_schedule_count(&env);
        let schedule = VestingSchedule {
            depositor: depositor.clone(),
            beneficiary: beneficiary.clone(),
            token,
            total_amount,
            claimed_amount: 0,
            start_time,
            cliff_duration,
            vesting_duration,
            revoked: false,
        };
        storage::set_schedule(&env, id, &schedule);
        storage::set_schedule_count(&env, id + 1);
        events::emit_created(&env, id, &depositor, &beneficiary, total_amount);
        id
    }

    /// Claim whatever has vested and not yet been claimed. Only the
    /// beneficiary may call this. Returns the amount transferred.
    pub fn claim(env: Env, caller: Address, id: u32) -> i128 {
        caller.require_auth();
        let mut schedule = storage::get_schedule(&env, id);

        if caller != schedule.beneficiary {
            panic!("only the beneficiary can claim");
        }
        if schedule.revoked {
            panic!("schedule has been revoked");
        }

        let vested = Self::vested_at(&schedule, env.ledger().timestamp());
        let claimable = vested - schedule.claimed_amount;
        if claimable <= 0 {
            panic!("nothing to claim yet");
        }

        schedule.claimed_amount += claimable;
        storage::set_schedule(&env, id, &schedule);

        let token_client = TokenClient::new(&env, &schedule.token);
        token_client.transfer(
            &env.current_contract_address(),
            &schedule.beneficiary,
            &claimable,
        );

        events::emit_claimed(&env, id, &caller, claimable);
        claimable
    }

    /// Revoke a schedule. Only the depositor may call this. Pays the
    /// beneficiary whatever has vested so far, and refunds the unvested
    /// remainder to the depositor.
    pub fn revoke(env: Env, caller: Address, id: u32) {
        caller.require_auth();
        let mut schedule = storage::get_schedule(&env, id);

        if caller != schedule.depositor {
            panic!("only the depositor can revoke");
        }
        if schedule.revoked {
            panic!("schedule already revoked");
        }

        let vested = Self::vested_at(&schedule, env.ledger().timestamp());
        let claimable = vested - schedule.claimed_amount;
        let remainder = schedule.total_amount - vested;

        schedule.revoked = true;
        schedule.claimed_amount = schedule.total_amount;
        storage::set_schedule(&env, id, &schedule);

        let token_client = TokenClient::new(&env, &schedule.token);
        if claimable > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &schedule.beneficiary,
                &claimable,
            );
        }
        if remainder > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &schedule.depositor,
                &remainder,
            );
        }

        events::emit_revoked(&env, id, &caller, remainder);
    }

    /// Return how much of the schedule has vested as of now, regardless of
    /// how much has already been claimed.
    pub fn vested_amount(env: Env, id: u32) -> i128 {
        let schedule = storage::get_schedule(&env, id);
        Self::vested_at(&schedule, env.ledger().timestamp())
    }

    /// Return the full state of a vesting schedule.
    pub fn get_schedule(env: Env, id: u32) -> VestingSchedule {
        storage::get_schedule(&env, id)
    }

    /// Pure vesting-curve calculation: linear from start_time over
    /// vesting_duration, but 0 until cliff_duration has passed.
    fn vested_at(schedule: &VestingSchedule, now: u64) -> i128 {
        if now < schedule.start_time {
            return 0;
        }
        let elapsed = now - schedule.start_time;
        if elapsed < schedule.cliff_duration {
            return 0;
        }
        if elapsed >= schedule.vesting_duration {
            return schedule.total_amount;
        }
        (schedule.total_amount * elapsed as i128) / schedule.vesting_duration as i128
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Env, String};
    use soroban_token::{TokenContract, TokenContractClient};

    struct Setup {
        contract_id: Address,
        client: VestingContractClient<'static>,
        token_id: Address,
        token_client: TokenContractClient<'static>,
        depositor: Address,
        beneficiary: Address,
    }

    fn setup(env: &Env) -> Setup {
        env.mock_all_auths();

        let contract_id = env.register(VestingContract, ());
        let client = VestingContractClient::new(env, &contract_id);

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
        let beneficiary = Address::generate(env);
        token_client.mint(&depositor, &1_000_000);

        Setup {
            contract_id,
            client,
            token_id,
            token_client,
            depositor,
            beneficiary,
        }
    }

    /// start_time=0, cliff_duration=100, vesting_duration=1000, total=1_000_000
    fn create_default(s: &Setup) -> u32 {
        s.client.create_vesting(
            &s.depositor,
            &s.beneficiary,
            &s.token_id,
            &1_000_000,
            &0,
            &100,
            &1_000,
        )
    }

    #[test]
    fn test_create_vesting_moves_real_balance() {
        let env = Env::default();
        let s = setup(&env);

        let id = create_default(&s);

        let schedule = s.client.get_schedule(&id);
        assert_eq!(schedule.total_amount, 1_000_000);
        assert_eq!(schedule.claimed_amount, 0);
        assert!(!schedule.revoked);

        assert_eq!(s.token_client.balance(&s.depositor), 0);
        assert_eq!(s.token_client.balance(&s.contract_id), 1_000_000);
    }

    #[test]
    #[should_panic(expected = "nothing to claim yet")]
    fn test_nothing_claimable_before_cliff() {
        let env = Env::default();
        let s = setup(&env);
        let id = create_default(&s);

        env.ledger().set_timestamp(50); // before cliff_duration (100)
        s.client.claim(&s.beneficiary, &id);
    }

    #[test]
    fn test_partial_claim_after_cliff_before_full_vest() {
        let env = Env::default();
        let s = setup(&env);
        let id = create_default(&s);

        env.ledger().set_timestamp(500); // 50% of vesting_duration (1000)
        let claimed = s.client.claim(&s.beneficiary, &id);

        assert_eq!(claimed, 500_000);
        assert_eq!(s.token_client.balance(&s.beneficiary), 500_000);
        assert_eq!(s.token_client.balance(&s.contract_id), 500_000);
    }

    #[test]
    fn test_full_claim_after_vesting_duration() {
        let env = Env::default();
        let s = setup(&env);
        let id = create_default(&s);

        env.ledger().set_timestamp(2_000); // well past vesting_duration
        let claimed = s.client.claim(&s.beneficiary, &id);

        assert_eq!(claimed, 1_000_000);
        assert_eq!(s.token_client.balance(&s.beneficiary), 1_000_000);
        assert_eq!(s.token_client.balance(&s.contract_id), 0);
    }

    #[test]
    fn test_second_claim_only_pays_the_delta() {
        let env = Env::default();
        let s = setup(&env);
        let id = create_default(&s);

        env.ledger().set_timestamp(500);
        s.client.claim(&s.beneficiary, &id);

        env.ledger().set_timestamp(1_000);
        let second_claim = s.client.claim(&s.beneficiary, &id);

        assert_eq!(second_claim, 500_000);
        assert_eq!(s.token_client.balance(&s.beneficiary), 1_000_000);
    }

    #[test]
    #[should_panic(expected = "only the beneficiary")]
    fn test_non_beneficiary_cannot_claim() {
        let env = Env::default();
        let s = setup(&env);
        let id = create_default(&s);

        env.ledger().set_timestamp(500);
        s.client.claim(&s.depositor, &id);
    }

    #[test]
    fn test_revoke_pays_vested_and_refunds_remainder() {
        let env = Env::default();
        let s = setup(&env);
        let id = create_default(&s);

        env.ledger().set_timestamp(500); // 50% vested
        s.client.revoke(&s.depositor, &id);

        assert_eq!(s.token_client.balance(&s.beneficiary), 500_000);
        assert_eq!(s.token_client.balance(&s.depositor), 500_000);
        assert_eq!(s.token_client.balance(&s.contract_id), 0);
        assert!(s.client.get_schedule(&id).revoked);
    }

    #[test]
    #[should_panic(expected = "schedule has been revoked")]
    fn test_cannot_claim_after_revoke() {
        let env = Env::default();
        let s = setup(&env);
        let id = create_default(&s);

        env.ledger().set_timestamp(500);
        s.client.revoke(&s.depositor, &id);
        s.client.claim(&s.beneficiary, &id);
    }

    #[test]
    #[should_panic(expected = "only the depositor")]
    fn test_non_depositor_cannot_revoke() {
        let env = Env::default();
        let s = setup(&env);
        let id = create_default(&s);

        s.client.revoke(&s.beneficiary, &id);
    }

    #[test]
    #[should_panic(expected = "already revoked")]
    fn test_cannot_revoke_twice() {
        let env = Env::default();
        let s = setup(&env);
        let id = create_default(&s);

        s.client.revoke(&s.depositor, &id);
        s.client.revoke(&s.depositor, &id);
    }

    #[test]
    #[should_panic(expected = "vesting schedule not found")]
    fn test_get_unknown_schedule_panics() {
        let env = Env::default();
        let s = setup(&env);
        s.client.get_schedule(&999);
    }
}
