use soroban_sdk::{Address, Env, Symbol};

pub fn emit_proposed(env: &Env, id: u32, proposer: &Address, to: &Address, amount: i128) {
    let topics = (Symbol::new(env, "proposed"), proposer.clone());
    env.events().publish(topics, (id, to.clone(), amount));
}

pub fn emit_approved(env: &Env, proposal_id: u32, signer: &Address) {
    let topics = (Symbol::new(env, "approved"), signer.clone());
    env.events().publish(topics, proposal_id);
}

pub fn emit_executed(env: &Env, proposal_id: u32, executor: &Address) {
    let topics = (Symbol::new(env, "executed"), executor.clone());
    env.events().publish(topics, proposal_id);
}
