use soroban_sdk::{Address, Env, Symbol};

pub fn emit_transfer(env: &Env, from: &Address, to: &Address, amount: i128) {
    let topics = (Symbol::new(env, "transfer"), from.clone(), to.clone());
    env.events().publish(topics, amount);
}

pub fn emit_mint(env: &Env, to: &Address, amount: i128) {
    let topics = (Symbol::new(env, "mint"), to.clone());
    env.events().publish(topics, amount);
}

pub fn emit_approve(env: &Env, from: &Address, spender: &Address, amount: i128, expiry: u32) {
    let topics = (Symbol::new(env, "approve"), from.clone(), spender.clone());
    env.events().publish(topics, (amount, expiry));
}

pub fn emit_burn(env: &Env, from: &Address, amount: i128) {
    let topics = (Symbol::new(env, "burn"), from.clone());
    env.events().publish(topics, amount);
}
