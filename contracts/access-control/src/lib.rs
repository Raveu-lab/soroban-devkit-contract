//! Role-based access control contract.
//!
//! Roles are open-ended Symbol values. Each role has a designated admin role
//! that controls who can grant or revoke it.
//! Emits RoleGranted and RoleRevoked events on every change.

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

mod errors;
mod events;
mod storage;

pub use errors::AccessControlError;

#[contract]
pub struct AccessControlContract;

#[contractimpl]
impl AccessControlContract {
    /// Initialize with a super admin address. Can only be called once.
    /// The super admin doesn't hold every role outright (has_role still
    /// returns false until a role is explicitly granted) — instead,
    /// require_role_admin lets them bypass the admin check entirely, so
    /// they can grant or revoke any role.
    pub fn initialize(env: Env, super_admin: Address) {
        if storage::is_initialized(&env) {
            panic!("already initialized");
        }
        storage::set_super_admin(&env, &super_admin);
    }

    /// Grant a role to an address. Caller must hold the admin role for that role.
    pub fn grant_role(env: Env, caller: Address, role: Symbol, to: Address) {
        caller.require_auth();
        storage::require_role_admin(&env, &caller, &role);
        storage::set_role(&env, &role, &to, true);
        events::emit_role_granted(&env, &role, &to, &caller);
    }

    /// Revoke a role from an address. Caller must hold the admin role for that role.
    pub fn revoke_role(env: Env, caller: Address, role: Symbol, from: Address) {
        caller.require_auth();
        storage::require_role_admin(&env, &caller, &role);
        storage::set_role(&env, &role, &from, false);
        events::emit_role_revoked(&env, &role, &from, &caller);
    }

    /// Check if an address holds a role. Returns true/false.
    pub fn has_role(env: Env, role: Symbol, addr: Address) -> bool {
        storage::get_role(&env, &role, &addr)
    }

    /// Set the admin role for a given role. Super admin only.
    pub fn set_role_admin(env: Env, caller: Address, role: Symbol, admin_role: Symbol) {
        caller.require_auth();
        let super_admin = storage::get_super_admin(&env);
        if caller != super_admin {
            panic!("only super admin can set role admins");
        }
        storage::set_role_admin(&env, &role, &admin_role);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::storage::Persistent as _, testutils::Address as _, testutils::Ledger as _, Env,
        Symbol,
    };

    #[test]
    fn test_grant_role_extends_the_persistent_entry_ttl() {
        // Same class of gap fixed in oracle: a role-membership entry that
        // never has its TTL extended eventually gets archived from
        // disuse, even though nothing about the role assignment changed.
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(AccessControlContract, ());
        let client = AccessControlContractClient::new(&env, &contract_id);

        let super_admin = Address::generate(&env);
        let user = Address::generate(&env);
        let role = Symbol::new(&env, "minter");

        client.initialize(&super_admin);
        client.grant_role(&super_admin, &role, &user);

        let (ttl, max_ttl) = env.as_contract(&contract_id, || {
            (
                env.storage()
                    .persistent()
                    .get_ttl(&(role.clone(), user.clone())),
                env.storage().max_ttl(),
            )
        });
        assert!(
            ttl > max_ttl / 2,
            "expected grant_role to extend the TTL close to max_ttl ({max_ttl}), got {ttl}"
        );
    }

    #[test]
    fn test_has_role_refreshes_the_ttl_of_an_aging_entry() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(AccessControlContract, ());
        let client = AccessControlContractClient::new(&env, &contract_id);

        let super_admin = Address::generate(&env);
        let user = Address::generate(&env);
        let role = Symbol::new(&env, "minter");

        client.initialize(&super_admin);
        client.grant_role(&super_admin, &role, &user);

        let initial_ttl = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get_ttl(&(role.clone(), user.clone()))
        });

        env.ledger()
            .set_sequence_number(env.ledger().sequence() + initial_ttl / 2);

        let ttl_before_read = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get_ttl(&(role.clone(), user.clone()))
        });
        assert!(
            ttl_before_read < initial_ttl,
            "sanity check: TTL should have visibly decreased after advancing the ledger"
        );

        client.has_role(&role, &user);

        let ttl_after_read = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get_ttl(&(role.clone(), user.clone()))
        });
        assert!(
            ttl_after_read > ttl_before_read,
            "has_role() should refresh the entry's TTL on read, not just on write"
        );
    }

    #[test]
    fn test_grant_and_check_role() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(AccessControlContract, ());
        let client = AccessControlContractClient::new(&env, &contract_id);

        let super_admin = Address::generate(&env);
        let user = Address::generate(&env);
        let role = Symbol::new(&env, "minter");

        client.initialize(&super_admin);
        assert!(!client.has_role(&role, &user));

        // The super admin can grant any role directly — require_role_admin
        // lets them bypass the admin check, no set_role_admin call needed.
        client.grant_role(&super_admin, &role, &user);
        assert!(client.has_role(&role, &user));
    }

    #[test]
    fn test_revoke_role() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(AccessControlContract, ());
        let client = AccessControlContractClient::new(&env, &contract_id);

        let super_admin = Address::generate(&env);
        let user = Address::generate(&env);
        let role = Symbol::new(&env, "minter");

        client.initialize(&super_admin);
        client.grant_role(&super_admin, &role, &user);
        assert!(client.has_role(&role, &user));

        client.revoke_role(&super_admin, &role, &user);
        assert!(!client.has_role(&role, &user));
    }

    #[test]
    fn test_delegated_admin_can_grant_role() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(AccessControlContract, ());
        let client = AccessControlContractClient::new(&env, &contract_id);

        let super_admin = Address::generate(&env);
        let delegate = Address::generate(&env);
        let user = Address::generate(&env);
        let minter_role = Symbol::new(&env, "minter");
        let minter_admin_role = Symbol::new(&env, "minter_admin");

        client.initialize(&super_admin);

        // Super admin delegates admin rights over "minter" to minter_admin_role,
        // then grants that admin role to `delegate`.
        client.set_role_admin(&super_admin, &minter_role, &minter_admin_role);
        client.grant_role(&super_admin, &minter_admin_role, &delegate);

        // delegate can now grant "minter" without being the super admin.
        client.grant_role(&delegate, &minter_role, &user);
        assert!(client.has_role(&minter_role, &user));
    }

    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_cannot_initialize_twice() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(AccessControlContract, ());
        let client = AccessControlContractClient::new(&env, &contract_id);

        let super_admin = Address::generate(&env);
        let attacker = Address::generate(&env);

        client.initialize(&super_admin);
        // A second initialize call must never succeed — otherwise anyone
        // could re-run it with their own address and hijack super admin.
        client.initialize(&attacker);
    }

    #[test]
    #[should_panic(expected = "caller does not have admin role")]
    fn test_unauthorized_caller_cannot_grant_role() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(AccessControlContract, ());
        let client = AccessControlContractClient::new(&env, &contract_id);

        let super_admin = Address::generate(&env);
        let stranger = Address::generate(&env);
        let user = Address::generate(&env);
        let minter_role = Symbol::new(&env, "minter");
        let minter_admin_role = Symbol::new(&env, "minter_admin");

        client.initialize(&super_admin);
        client.set_role_admin(&super_admin, &minter_role, &minter_admin_role);

        // stranger holds no admin role for "minter" and isn't the super admin.
        client.grant_role(&stranger, &minter_role, &user);
    }
}
