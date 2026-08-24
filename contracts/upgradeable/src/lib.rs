//! Upgradeable contract pattern for Soroban.
//!
//! Demonstrates correct WASM hash replacement, versioned migrations,
//! and admin-gated upgrade authorization.

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol};

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
    use soroban_sdk::{testutils::Address as _, Env};

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
}
