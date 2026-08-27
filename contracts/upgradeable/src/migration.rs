//! State migration logic for the upgradeable contract.
//!
//! Add a new match arm for each contract version that requires a state migration.
//! Migrations must be idempotent — running them twice must produce the same result.

use soroban_sdk::Env;

/// The newest version this WASM build knows how to migrate to.
/// Bump this whenever a new match arm is added below.
pub const LATEST_VERSION: u32 = 2;

/// Run state migrations for the given current version.
/// Returns the new version number after migration. Idempotent: calling this
/// again once `current_version >= LATEST_VERSION` is a no-op that returns
/// `current_version` unchanged, rather than incrementing forever.
pub fn run(env: &Env, current_version: u32) -> u32 {
    let _ = env;
    if current_version >= LATEST_VERSION {
        return current_version;
    }
    match current_version {
        // v1 -> v2: no state migration needed in the base scaffold
        // TODO: add migration logic for v2 here when ready
        // See: https://github.com/soroban-devkit/soroban-devkit-contracts/issues/8
        1 => 2,
        other => other + 1,
    }
}
