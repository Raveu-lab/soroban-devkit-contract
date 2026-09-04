//! Error codes for the oracle contract.
//!
//! All error variants are assigned a stable u32 discriminant.
//! Never reuse or reorder existing values — doing so breaks on-chain clients.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum OracleError {
    /// No price has ever been published for the requested asset
    NoPriceData = 1,
    /// set_price was called with a zero or negative price
    InvalidPrice = 2,
}
