use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AccessControlError {
    NotSuperAdmin = 1,
    NotRoleAdmin = 2,
    RoleAdminNotSet = 3,
    AlreadyInitialized = 4,
}
