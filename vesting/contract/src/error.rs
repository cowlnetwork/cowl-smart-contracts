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
#[derive(Clone, Copy, Debug)]
pub enum VestingError {
    InsufficientRights = 1,
    UnexpectedKeyVariant = 2,
    InvalidStorageUref = 3,
    MissingStorageUref = 4,
    InvalidKey = 5,
    MissingKey = 6,
    Phantom = 7,
    FailedToGetArgBytes = 8,
    InvalidEventsMode = 9,
    MissingEventsMode = 10,
    InvalidUpgradeFlag = 11,
    MissingVestingName = 12,
    InvalidVestingName = 13,
    InvalidContractHash = 14,
    MissingContractHash = 15,
    InvalidAdminList = 16,
    InvalidNoneList = 17,
    InvalidPackageHash = 18,
    MissingPackageHash = 19,
    ContractAlreadyInitialized = 20,
    MissingPackageHashForUpgrade = 21,
    Overflow = 22,
    MissingInstaller = 23,
    InvalidInstaller = 24,
    InvalidTokenContractPackage = 25,
    MissingTokenContractPackage = 26,
    VestingLocked = 40001,
    InvalidVestingType = 40002,
    MissingVestingType = 40003,
    InvalidInstallerTotalSupply = 40004,
    InvalidRecepientAllocation = 40005,
}

impl From<VestingError> for ApiError {
    fn from(error: VestingError) -> Self {
        ApiError::User(error as u16)
    }
}
