//! Error handling on the Casper platform.
use casper_types::ApiError;

/// Errors that the contract can return.
///
/// When an `Error` is returned from a smart contract, it is converted to an [`ApiError::User`].
///
/// While the code consuming this contract needs to define further error variants, it can
/// return those via the [`Error::User`] variant or equivalently via the [`ApiError::User`]
/// variant.
#[repr(u16)]
#[derive(Clone, Copy)]
pub enum VestingError {
    InsufficientRights = 1,
    UnexpectedKeyVariant = 2,
    InvalidStorageUref = 3,
    MissingStorageUref = 4,
    InvalidKey = 5,
    Phantom = 6,
    FailedToGetArgBytes = 7,
    InvalidEventsMode = 8,
    InvalidUpgradeFlag = 9,
    MissingCollectionName = 10,
    InvalidCollectionName = 11,
    InvalidContractHash = 12,
    MissingContractHash = 13,
    InvalidAdminList = 14,
    InvalidNoneList = 15,
    InvalidPackageHash = 16,
    MissingPackageHash = 17,
    ContractAlreadyInitialized = 18,
    MissingPackageHashForUpgrade = 19,
    VestingLocked = 60001,
    InvalidVestingType = 60002,
}

impl From<VestingError> for ApiError {
    fn from(error: VestingError) -> Self {
        ApiError::User(error as u16)
    }
}
