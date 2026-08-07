use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum UpgradeableError {
    AlreadyInitialized = 1,
    NotAdmin = 2,
    MigrationFailed = 3,
}
