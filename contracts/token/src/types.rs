//! Shared data types for the token contract.

use soroban_sdk::{contracttype, String};

/// Metadata stored on-chain during initialization
#[contracttype]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}
