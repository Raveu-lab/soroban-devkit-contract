//! Upgradeable contract pattern for Soroban.
//!
//! Demonstrates correct WASM hash replacement, versioned migrations,
//! and admin-gated upgrade authorization.

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

mod errors;
mod events;
mod migration;
mod storage;

pub use errors::UpgradeableError;

#[contract]
pub struct UpgradeableContract;

#[contractimpl]
impl UpgradeableContract {
    /// Initialize with an admin address and version 1.
    pub fn initialize(env: Env, admin: Address) {
        if storage::is_initialized(&env) {
            panic!("already initialized");
        }
        storage::set_admin(&env, &admin);
        storage::set_version(&env, 1);
        events::emit_initialized(&env, &admin, 1);
    }

    /// Upgrade the contract WASM. Admin only.
    /// After upgrade, call `migrate()` once to apply any state changes.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin = storage::get_admin(&env);
        admin.require_auth();
        env.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());
        events::emit_upgraded(&env, &new_wasm_hash);
    }

    /// Run migration logic after an upgrade. Admin only. Idempotent.
    pub fn migrate(env: Env) {
        let admin = storage::get_admin(&env);
        admin.require_auth();
        let current_version = storage::get_version(&env);
        let new_version = migration::run(&env, current_version);
        storage::set_version(&env, new_version);
        events::emit_migrated(&env, current_version, new_version);
    }

    /// Return the current contract version.
    pub fn version(env: Env) -> u32 {
        storage::get_version(&env)
    }

    /// Return the current admin address.
    pub fn admin(env: Env) -> Address {
        storage::get_admin(&env)
    }

    /// Transfer admin rights to a new address. Current admin only.
    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        current_admin.require_auth();
        let stored = storage::get_admin(&env);
        if current_admin != stored {
            panic!("not admin");
        }
        storage::set_admin(&env, &new_admin);
        events::emit_admin_transferred(&env, &current_admin, &new_admin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::storage::Instance as _, testutils::Address as _, testutils::Ledger as _, Env,
    };

    #[test]
    fn test_initialize_extends_the_instance_ttl() {
        // The most severe version of this class of bug found in this repo
        // yet: this contract exists specifically to be upgraded to fix
        // bugs. If Admin/Version get archived, upgrade()/migrate() (both
        // admin-gated, reading Admin via require_auth) become permanently
        // uncallable — there would be no way to ever upgrade the contract
        // again, since the very mechanism meant to fix problems is itself
        // the thing that broke.
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(UpgradeableContract, ());
        let client = UpgradeableContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);

        client.initialize(&admin);

        let (ttl, max_ttl) = env.as_contract(&contract_id, || {
            (env.storage().instance().get_ttl(), env.storage().max_ttl())
        });
        assert!(
            ttl > max_ttl / 2,
            "expected initialize to extend the instance TTL close to max_ttl ({max_ttl}), got {ttl}"
        );
    }

    #[test]
    fn test_migrate_refreshes_the_instance_ttl_of_an_aging_contract() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(UpgradeableContract, ());
        let client = UpgradeableContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let initial_ttl = env.as_contract(&contract_id, || env.storage().instance().get_ttl());

        env.ledger()
            .set_sequence_number(env.ledger().sequence() + initial_ttl / 2);

        let ttl_before_write = env.as_contract(&contract_id, || env.storage().instance().get_ttl());
        assert!(
            ttl_before_write < initial_ttl,
            "sanity check: instance TTL should have visibly decreased after advancing the ledger"
        );

        client.migrate();

        let ttl_after_write = env.as_contract(&contract_id, || env.storage().instance().get_ttl());
        assert!(
            ttl_after_write > ttl_before_write,
            "migrate() (which reads the admin via require_auth) should refresh the instance TTL too"
        );
    }

    #[test]
    fn test_initialize_sets_version_1() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(UpgradeableContract, ());
        let client = UpgradeableContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        assert_eq!(client.version(), 1);
        assert_eq!(client.admin(), admin);
    }

    #[test]
    fn test_migrate_is_idempotent() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(UpgradeableContract, ());
        let client = UpgradeableContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        client.migrate();
        let version_after_first = client.version();

        // Running migrate() again must produce the same result — it must
        // not keep bumping the version on every call.
        client.migrate();
        assert_eq!(client.version(), version_after_first);
    }

    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_double_initialize_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(UpgradeableContract, ());
        let client = UpgradeableContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);
        client.initialize(&admin); // should panic
    }

    #[test]
    fn test_transfer_admin_moves_admin_rights() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(UpgradeableContract, ());
        let client = UpgradeableContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);
        client.initialize(&admin);

        client.transfer_admin(&admin, &new_admin);
        assert_eq!(client.admin(), new_admin);

        // The new admin can now act; the old one no longer can (checked below).
        let another_admin = Address::generate(&env);
        client.transfer_admin(&new_admin, &another_admin);
        assert_eq!(client.admin(), another_admin);
    }

    #[test]
    #[should_panic(expected = "not admin")]
    fn test_transfer_admin_rejects_a_stale_current_admin() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(UpgradeableContract, ());
        let client = UpgradeableContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);
        let attacker = Address::generate(&env);
        client.initialize(&admin);

        // attacker passes themselves as current_admin — require_auth() would
        // need their own signature (mocked here), but they don't match the
        // stored admin, so this must still panic.
        client.transfer_admin(&attacker, &new_admin);
    }
}
