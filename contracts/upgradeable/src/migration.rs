use soroban_sdk::Env;

/// Run state migrations for a given version upgrade.
/// Returns the new version number.
///
/// Add a new match arm for each contract version that requires a migration.
/// Migrations must be idempotent — running them twice should be safe.
pub fn run(env: &Env, current_version: u32) -> u32 {
    let _ = env;
    match current_version {
        // v1 -> v2: no state migration needed in the base scaffold
        // TODO: add migration logic for v2 here when ready
        // See: https://github.com/soroban-devkit/soroban-devkit-contracts/issues/8
        1 => 2,
        _ => current_version + 1,
    }
}
