//! Shared data types for the oracle contract.

use soroban_sdk::contracttype;

/// A single published price point for one asset.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PriceData {
    /// Price, in whatever fixed-point unit the caller has agreed on off-chain
    /// (e.g. cents, or a fixed number of decimal places) — this contract
    /// doesn't interpret it, only stores and returns it.
    pub price: i128,
    /// Ledger timestamp (seconds) the price was published at.
    pub timestamp: u64,
}
