//! Shared data types for the token contract.

use soroban_sdk::{contracttype, String};

/// Metadata stored on-chain during initialization
#[contracttype]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

/// An allowance and the last ledger (inclusive) it may be spent on.
/// Stored together because the ledger's own TTL mechanism can't enforce an
/// absolute expiration: `extend_ttl` only ever extends, and takes a
/// relative ledger count, so a temporary entry may outlive `expiration_ledger`.
#[contracttype]
#[derive(Clone)]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}
