use crate::{
    constants::{
        DURATION_COMMUNITY_VESTING, DURATION_CONTRIBUTOR_VESTING, DURATION_DEVELOPMENT_VESTING,
        DURATION_LIQUIDITY_VESTING, DURATION_STAKING_VESTING, DURATION_TREASURY_VESTING,
    },
    error::VestingError,
    vesting::VestingInfo,
};
use alloc::{fmt, vec, vec::Vec};
use casper_types::{
    bytesrepr::{Error, FromBytes, ToBytes, U8_SERIALIZED_LENGTH},
    CLType, CLTyped,
};
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use time::Duration;

#[repr(u8)]
#[derive(PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum EventsMode {
    NoEvents = 0,
    CES = 1,
}

impl TryFrom<u8> for EventsMode {
    type Error = VestingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EventsMode::NoEvents),
            1 => Ok(EventsMode::CES),
            _ => Err(VestingError::InvalidEventsMode),
        }
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug, Copy, EnumIter)]
pub enum VestingType {
    Treasury = 0,
    Contributor = 1,
    Development = 2,
    Liquidity = 3,
    Community = 4,
    Staking = 128,
}

impl ToBytes for VestingType {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let bytes = vec![*self as u8];
        Ok(bytes)
    }

    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
    }
}

impl FromBytes for VestingType {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        if bytes.is_empty() {
            return Err(Error::EarlyEndOfStream);
        }

        // Extract the first byte
        let (value, rem) = bytes.split_at(1);
        let vesting_type = VestingType::try_from(value[0]).map_err(|_| Error::Formatting)?;
        Ok((vesting_type, rem))
    }
}

impl TryFrom<u8> for VestingType {
    type Error = VestingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(VestingType::Treasury),
            1 => Ok(VestingType::Contributor),
            2 => Ok(VestingType::Development),
            3 => Ok(VestingType::Liquidity),
            4 => Ok(VestingType::Community),
            128 => Ok(VestingType::Staking),
            _ => Err(VestingError::InvalidVestingType),
        }
    }
}

impl From<VestingType> for u8 {
    fn from(vesting_type: VestingType) -> Self {
        vesting_type as u8
    }
}

impl TryFrom<&str> for VestingType {
    type Error = VestingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Treasury" => Ok(VestingType::Treasury),
            "Contributor" => Ok(VestingType::Contributor),
            "Development" => Ok(VestingType::Development),
            "Liquidity" => Ok(VestingType::Liquidity),
            "Community" => Ok(VestingType::Community),
            "Staking" => Ok(VestingType::Staking),
            _ => Err(VestingError::InvalidVestingType),
        }
    }
}

impl fmt::Display for VestingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VestingType::Treasury => write!(f, "Treasury"),
            VestingType::Contributor => write!(f, "Contributor"),
            VestingType::Development => write!(f, "Development"),
            VestingType::Liquidity => write!(f, "Liquidity"),
            VestingType::Community => write!(f, "Community"),
            VestingType::Staking => write!(f, "Staking"),
        }
    }
}

pub const VESTING_INFO: &[VestingInfo] = &[
    VestingInfo {
        vesting_type: VestingType::Treasury,
        maybe_vesting_address_key: None,
        vesting_duration: DURATION_TREASURY_VESTING,
    },
    VestingInfo {
        vesting_type: VestingType::Contributor,
        maybe_vesting_address_key: None,
        vesting_duration: DURATION_CONTRIBUTOR_VESTING,
    },
    VestingInfo {
        vesting_type: VestingType::Development,
        maybe_vesting_address_key: None,
        vesting_duration: DURATION_DEVELOPMENT_VESTING,
    },
    VestingInfo {
        vesting_type: VestingType::Liquidity,
        maybe_vesting_address_key: None,
        vesting_duration: DURATION_LIQUIDITY_VESTING,
    },
    VestingInfo {
        vesting_type: VestingType::Community,
        maybe_vesting_address_key: None,
        vesting_duration: DURATION_COMMUNITY_VESTING,
    },
    VestingInfo {
        vesting_type: VestingType::Staking,
        maybe_vesting_address_key: None,
        vesting_duration: DURATION_STAKING_VESTING,
    },
];

/// Function to get the vesting duration for a specific vesting type
pub fn get_vesting_duration(vesting_type: VestingType) -> Option<Duration> {
    VESTING_INFO
        .iter()
        .find(|vesting_info| vesting_info.vesting_type == vesting_type)
        .and_then(|vesting_info| vesting_info.vesting_duration)
}

pub const VESTING_PERCENTAGES: &[(VestingType, u8)] = &[
    (VestingType::Liquidity, 20),
    (VestingType::Contributor, 10),
    (VestingType::Development, 12),
    (VestingType::Treasury, 30),
    (VestingType::Community, 28),
    (VestingType::Staking, 0),
];

#[repr(u8)]
#[non_exhaustive]
#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
pub enum TransferFilterContractResult {
    #[default]
    DenyTransfer = 0,
    ProceedTransfer,
}

impl From<u8> for TransferFilterContractResult {
    fn from(value: u8) -> Self {
        match value {
            0 => TransferFilterContractResult::DenyTransfer,
            _ => TransferFilterContractResult::ProceedTransfer,
        }
    }
}

impl CLTyped for TransferFilterContractResult {
    fn cl_type() -> casper_types::CLType {
        CLType::U8
    }
}

impl FromBytes for TransferFilterContractResult {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        match bytes.split_first() {
            None => Err(casper_types::bytesrepr::Error::EarlyEndOfStream),
            Some((byte, rem)) => Ok((TransferFilterContractResult::from(*byte), rem)),
        }
    }
}

impl ToBytes for TransferFilterContractResult {
    fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, casper_types::bytesrepr::Error> {
        Ok(vec![*self as u8])
    }

    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
    }
}
