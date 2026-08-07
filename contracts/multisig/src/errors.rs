use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MultisigError {
    AlreadyInitialized = 1,
    InvalidThreshold = 2,
    NotSigner = 3,
    ProposalNotFound = 4,
    AlreadyExecuted = 5,
    InsufficientApprovals = 6,
}
