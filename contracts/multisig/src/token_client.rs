//! Minimal cross-contract client for invoking a SEP-41 token's `transfer`.
//!
//! Declared as a trait + #[contractclient] rather than depending on the
//! token contract's crate directly — this contract works with any deployed
//! contract exposing the standard transfer(from, to, amount) signature,
//! not just soroban-devkit's own token contract.

use soroban_sdk::{contractclient, Address, Env};

// The trait itself is never called directly — only the #[contractclient]-
// generated TokenClient struct is — so clippy sees it as unused.
#[allow(dead_code)]
#[contractclient(name = "TokenClient")]
pub trait TokenInterface {
    fn transfer(env: Env, from: Address, to: Address, amount: i128);
}
