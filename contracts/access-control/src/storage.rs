//! Ledger storage helpers for the access-control contract.
//!
//! All reads and writes to contract storage go through this module.
//! Never call `env.storage()` directly in `lib.rs`.

use soroban_sdk::{Address, Env, Symbol};

/// How close to the network's max TTL a role-membership entry is kept —
/// extended whenever its remaining TTL drops within this many ledgers of
/// that max, back up to the max itself. Same approach as oracle's
/// extend_price_ttl: self-adjusting to the network's actual max_ttl()
/// rather than a fixed ledger-count guess tied to an assumed close time.
const TTL_EXTEND_BUFFER: u32 = 1_000;

fn extend_role_ttl(env: &Env, key: &(Symbol, Address)) {
    let max_ttl = env.storage().max_ttl();
    env.storage()
        .persistent()
        .extend_ttl(key, max_ttl.saturating_sub(TTL_EXTEND_BUFFER), max_ttl);
}

/// Extends the whole contract instance's TTL — same class of fix as
/// extend_role_ttl, but more severe: instance storage holds SuperAdmin
/// itself, so losing it to archival would make every function that checks
/// admin rights inoperable, not just one role entry.
fn extend_instance_ttl(env: &Env) {
    let max_ttl = env.storage().max_ttl();
    env.storage()
        .instance()
        .extend_ttl(max_ttl.saturating_sub(TTL_EXTEND_BUFFER), max_ttl);
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .instance()
        .has(&Symbol::new(env, "SuperAdmin"))
}

pub fn set_super_admin(env: &Env, addr: &Address) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "SuperAdmin"), addr);
    extend_instance_ttl(env);
}

pub fn get_super_admin(env: &Env) -> Address {
    let super_admin = env
        .storage()
        .instance()
        .get(&Symbol::new(env, "SuperAdmin"))
        .unwrap();
    extend_instance_ttl(env);
    super_admin
}

/// Storage key: (role, address) -> bool
pub fn set_role(env: &Env, role: &Symbol, addr: &Address, has_role: bool) {
    let key = (role.clone(), addr.clone());
    env.storage().persistent().set(&key, &has_role);
    extend_role_ttl(env, &key);
}

pub fn get_role(env: &Env, role: &Symbol, addr: &Address) -> bool {
    let key = (role.clone(), addr.clone());
    let value = env.storage().persistent().get(&key);
    if value.is_some() {
        extend_role_ttl(env, &key);
    }
    value.unwrap_or(false)
}

/// Storage key: role -> admin_role
pub fn set_role_admin(env: &Env, role: &Symbol, admin_role: &Symbol) {
    let key = (Symbol::new(env, "RoleAdmin"), role.clone());
    env.storage().instance().set(&key, admin_role);
}

pub fn get_role_admin(env: &Env, role: &Symbol) -> Option<Symbol> {
    let key = (Symbol::new(env, "RoleAdmin"), role.clone());
    env.storage().instance().get(&key)
}

/// Panics if caller does not hold the admin role for `role`.
pub fn require_role_admin(env: &Env, caller: &Address, role: &Symbol) {
    let super_admin = get_super_admin(env);
    if caller == &super_admin {
        return;
    }
    let admin_role =
        get_role_admin(env, role).unwrap_or_else(|| panic!("no admin role set for this role"));
    if !get_role(env, &admin_role, caller) {
        panic!("caller does not have admin role");
    }
}
