use soroban_sdk::{Address, Env, Symbol};

pub fn set_super_admin(env: &Env, addr: &Address) {
    env.storage()
        .instance()
        .set(&Symbol::new(env, "SuperAdmin"), addr);
}

pub fn get_super_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&Symbol::new(env, "SuperAdmin"))
        .unwrap()
}

/// Storage key: (role, address) -> bool
pub fn set_role(env: &Env, role: &Symbol, addr: &Address, has_role: bool) {
    let key = (role.clone(), addr.clone());
    env.storage().persistent().set(&key, &has_role);
}

pub fn get_role(env: &Env, role: &Symbol, addr: &Address) -> bool {
    let key = (role.clone(), addr.clone());
    env.storage().persistent().get(&key).unwrap_or(false)
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
    let admin_role = get_role_admin(env, role)
        .unwrap_or_else(|| panic!("no admin role set for this role"));
    if !get_role(env, &admin_role, caller) {
        panic!("caller does not have admin role");
    }
}
