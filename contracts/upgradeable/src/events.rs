use soroban_sdk::{Address, BytesN, Env, Symbol};

pub fn emit_initialized(env: &Env, admin: &Address, version: u32) {
    let topics = (Symbol::new(env, "initialized"), admin.clone());
    env.events().publish(topics, version);
}

pub fn emit_upgraded(env: &Env, new_wasm_hash: &BytesN<32>) {
    let topics = (Symbol::new(env, "upgraded"),);
    env.events().publish(topics, new_wasm_hash.clone());
}

pub fn emit_migrated(env: &Env, from_version: u32, to_version: u32) {
    let topics = (Symbol::new(env, "migrated"),);
    env.events().publish(topics, (from_version, to_version));
}

pub fn emit_admin_transferred(env: &Env, from: &Address, to: &Address) {
    let topics = (Symbol::new(env, "admin_transferred"), from.clone());
    env.events().publish(topics, to.clone());
}
