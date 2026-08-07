use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub to: Address,
    pub amount: i128,
    pub token: Address,
    pub executed: bool,
}
