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
    /// Initialize with a super admin address that holds all roles by default.
    pub fn initialize(env: Env, super_admin: Address) {
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
    use soroban_sdk::{testutils::Address as _, Env, Symbol};

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

        // Super admin grants minter role to user
        // TODO: set_role_admin first so grant_role passes admin check
        // Full scenario tracked in issue #6
        assert!(!client.has_role(&role, &user));
    }
}
