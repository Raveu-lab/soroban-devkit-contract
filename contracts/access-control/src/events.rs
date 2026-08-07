use soroban_sdk::{Address, Env, Symbol};

pub fn emit_role_granted(env: &Env, role: &Symbol, to: &Address, by: &Address) {
    let topics = (Symbol::new(env, "role_granted"), role.clone(), to.clone());
    env.events().publish(topics, by.clone());
}

pub fn emit_role_revoked(env: &Env, role: &Symbol, from: &Address, by: &Address) {
    let topics = (Symbol::new(env, "role_revoked"), role.clone(), from.clone());
    env.events().publish(topics, by.clone());
}
