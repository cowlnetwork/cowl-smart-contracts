use crate::enums::VestingType;
#[cfg(feature = "contract-support")]
use crate::{
    constants::{DICT_START_TIME, DICT_VESTING_AMOUNT, MONTH_IN_SECONDS},
    enums::{VESTING_INFO, VESTING_PERCENTAGES},
    error::VestingError,
    utils::get_dictionary_value_from_key,
};
#[cfg(feature = "contract-support")]
use alloc::format;
use alloc::{boxed::Box, string::String, vec::Vec};
#[cfg(feature = "contract-support")]
use casper_contract::{
    contract_api::runtime::{self, get_blocktime, ret},
    unwrap_or_revert::UnwrapOrRevert,
};
#[cfg(feature = "contract-support")]
use casper_types::CLValue;
use casper_types::{
    bytesrepr::{Bytes, Error, FromBytes, ToBytes},
    CLType, CLTyped, Key, U256,
};
use time::Duration;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct VestingInfo {
    pub vesting_type: VestingType,
    pub vesting_address: &'static str,
    pub maybe_vesting_address_key: Option<Key>,
    pub vesting_duration: Option<Duration>,
}

impl ToBytes for VestingInfo {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();

        // Ensure all components serialize correctly
        bytes.extend(self.vesting_type.to_bytes()?);
        bytes.extend(self.vesting_address.to_bytes()?);
        bytes.extend(self.maybe_vesting_address_key.to_bytes()?);

        // For optional vesting duration, use a consistent format
        match self.vesting_duration {
            Some(duration) => bytes.extend(Some(duration.whole_seconds() as u64).to_bytes()?),
            None => bytes.extend(Option::<u64>::None.to_bytes()?),
        }

        Ok(bytes)
    }

    fn serialized_length(&self) -> usize {
        self.vesting_type.serialized_length()
            + self.vesting_address.serialized_length()
            + self.maybe_vesting_address_key.serialized_length()
            + Option::<u64>::serialized_length(
                &self.vesting_duration.map(|d| d.whole_seconds() as u64),
            )
    }
}

impl FromBytes for VestingInfo {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (vesting_type, rem) = VestingType::from_bytes(bytes)?;
        let (vesting_address, rem) = String::from_bytes(rem)?;
        let (maybe_vesting_address_key, rem) = Option::<Key>::from_bytes(rem)?;
        let (vesting_duration_opt, rem) = Option::<u64>::from_bytes(rem)?;

        let vesting_duration = vesting_duration_opt.map(|seconds| Duration::new(seconds as i64, 0));

        Ok((
            VestingInfo {
                vesting_type,
                vesting_address: Box::leak(vesting_address.into_boxed_str()),
                maybe_vesting_address_key,
                vesting_duration,
            },
            rem,
        ))
    }
}

impl CLTyped for VestingInfo {
    fn cl_type() -> CLType {
        Bytes::cl_type()
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct VestingAllocation {
    pub vesting_address: &'static str,
    pub vesting_address_key: Key,
    pub vesting_amount: U256,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct VestingStatus {
    pub total_amount: U256,
    pub vested_amount: U256,
    pub is_fully_vested: bool,
    pub vesting_duration: Duration,
    pub time_until_next_release: Duration,
    pub monthly_release: U256,
}

impl VestingStatus {
    fn new(
        total_amount: U256,
        vested_amount: U256,
        is_fully_vested: bool,
        vesting_duration: Duration,
        time_until_next: Duration,
        monthly_release: U256,
    ) -> Self {
        Self {
            total_amount,
            vested_amount,
            is_fully_vested,
            vesting_duration,
            time_until_next_release: time_until_next,
            monthly_release,
        }
    }
}

impl CLTyped for VestingStatus {
    fn cl_type() -> CLType {
        Bytes::cl_type()
    }
}

impl ToBytes for VestingStatus {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();

        // Serialize each field in the VestingStatus struct
        bytes.extend(self.total_amount.to_bytes()?);
        bytes.extend(self.vested_amount.to_bytes()?);
        bytes.extend(self.is_fully_vested.to_bytes()?);
        bytes.extend((self.vesting_duration.whole_seconds() as u64).to_bytes()?);
        bytes.extend((self.time_until_next_release.whole_seconds() as u64).to_bytes()?);
        bytes.extend(self.monthly_release.to_bytes()?);

        Ok(bytes)
    }

    fn serialized_length(&self) -> usize {
        self.total_amount.serialized_length()
            + self.vested_amount.serialized_length()
            + self.is_fully_vested.serialized_length()
            + (self.vesting_duration.whole_seconds() as u64).serialized_length()
            + (self.time_until_next_release.whole_seconds() as u64).serialized_length()
            + self.monthly_release.serialized_length()
    }
}

impl FromBytes for VestingStatus {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (total_amount, bytes) = U256::from_bytes(bytes)?;
        let (vested_amount, bytes) = U256::from_bytes(bytes)?;
        let (is_fully_vested, bytes) = bool::from_bytes(bytes)?;
        let (vesting_duration_ms, bytes) = u64::from_bytes(bytes)?; // Convert back from milliseconds
        let (time_until_next_release_ms, bytes) = u64::from_bytes(bytes)?; // Convert back from milliseconds
        let (monthly_release, bytes) = U256::from_bytes(bytes)?;

        let vesting_duration = Duration::new(vesting_duration_ms as i64, 0);
        let time_until_next_release = Duration::new(time_until_next_release_ms as i64, 0);

        Ok((
            VestingStatus::new(
                total_amount,
                vested_amount,
                is_fully_vested,
                vesting_duration,
                time_until_next_release,
                monthly_release,
            ),
            bytes,
        ))
    }
}

#[cfg(feature = "contract-support")]
pub fn ret_vesting_status(vesting_type: VestingType) {
    let vesting_status = get_vesting_status_by_type(vesting_type);
    runtime::print(&format!("{:?}", vesting_status));
    let result = CLValue::from_t(vesting_status).unwrap_or_revert();

    ret(result);
}

#[cfg(feature = "contract-support")]
pub fn ret_vesting_info(vesting_type: VestingType) {
    let vesting_info = get_vesting_info_by_type(vesting_type)
        .unwrap_or_revert_with(VestingError::InvalidVestingType);
    runtime::print(&format!("{:?}", vesting_info));
    let result = CLValue::from_t(vesting_info).unwrap_or_revert();
    ret(result);
}

#[cfg(feature = "contract-support")]
fn get_vesting_info() -> Vec<VestingInfo> {
    use crate::{constants::DICT_ADDRESSES, utils::get_dictionary_value_from_key};

    VESTING_INFO
        .iter()
        .map(|vesting_info| VestingInfo {
            vesting_type: vesting_info.vesting_type,
            vesting_address: vesting_info.vesting_address,
            maybe_vesting_address_key: get_dictionary_value_from_key(
                DICT_ADDRESSES,
                vesting_info.vesting_address,
            ),
            vesting_duration: vesting_info.vesting_duration,
        })
        .collect()
}

#[cfg(feature = "contract-support")]
fn get_vesting_info_by_key(vesting_address_key: &Key) -> Option<VestingInfo> {
    get_vesting_info()
        .into_iter()
        .find(|info| info.maybe_vesting_address_key.as_ref() == Some(vesting_address_key))
}

#[cfg(feature = "contract-support")]
fn get_vesting_info_by_type(vesting_type: VestingType) -> Option<VestingInfo> {
    get_vesting_info()
        .into_iter()
        .find(|info| info.vesting_type == vesting_type)
}

#[cfg(feature = "contract-support")]
fn get_vesting_status(
    vesting_info: &VestingInfo,
    start_time: u64,
    total_amount: U256,
    current_time: u64,
) -> VestingStatus {
    let elapsed_time = current_time.saturating_sub(start_time);

    // Handle linear vesting or immediate full vesting based on type
    let vested_amount = match vesting_info.vesting_type {
        VestingType::Treasury => {
            // Treasury has an immediate unlock at the end of the lock duration
            if let Some(duration) = vesting_info.vesting_duration {
                if elapsed_time >= duration.whole_seconds() as u64 {
                    total_amount
                } else {
                    U256::zero()
                }
            } else {
                U256::zero() // If duration is None, default to no vesting
            }
        }
        _ => {
            // Linear vesting for other types
            if let Some(duration) = vesting_info.vesting_duration {
                calculate_linear_vesting(start_time, duration, total_amount, current_time)
            } else {
                U256::zero() // Default to no vesting if duration is None
            }
        }
    };

    let is_fully_vested = vested_amount == total_amount;
    let time_until_next_release = if is_fully_vested {
        Duration::ZERO
    } else if let Some(duration) = vesting_info.vesting_duration {
        calculate_time_until_next_release(start_time, duration, current_time)
    } else {
        Duration::ZERO
    };

    let monthly_release = if vesting_info.vesting_type == VestingType::Treasury {
        U256::zero() // Treasury does not have monthly releases
    } else if let Some(duration) = vesting_info.vesting_duration {
        calculate_monthly_release(total_amount, duration)
    } else {
        U256::zero()
    };

    VestingStatus::new(
        total_amount,
        vested_amount,
        is_fully_vested,
        vesting_info.vesting_duration.unwrap_or(Duration::ZERO),
        time_until_next_release,
        monthly_release,
    )
}

#[cfg(feature = "contract-support")]
fn get_vesting_status_by_type(vesting_type: VestingType) -> VestingStatus {
    // Retrieve the vesting information based on the type
    use crate::{
        constants::{DICT_ADDRESSES, DICT_START_TIME, DICT_VESTING_AMOUNT},
        utils::get_dictionary_value_from_key,
    };
    let vesting_info = VESTING_INFO
        .iter()
        .find(|vesting_info| vesting_info.vesting_type == vesting_type)
        .expect("Invalid vesting type");

    let maybe_vesting_address_key =
        get_dictionary_value_from_key(DICT_ADDRESSES, vesting_info.vesting_address);
    let start_time = get_dictionary_value_from_key(DICT_START_TIME, vesting_info.vesting_address)
        .unwrap_or_default();
    let total_amount =
        get_dictionary_value_from_key(DICT_VESTING_AMOUNT, vesting_info.vesting_address)
            .unwrap_or_default();

    let current_time: u64 = get_blocktime().into();

    // Calculate the vesting status
    get_vesting_status(
        &VestingInfo {
            vesting_address: vesting_info.vesting_address,
            vesting_type,
            vesting_duration: vesting_info.vesting_duration,
            maybe_vesting_address_key,
        },
        start_time,
        total_amount,
        current_time,
    )
}

#[cfg(feature = "contract-support")]
pub fn check_vesting_transfer(sender: Key, amount: U256) -> bool {
    // Retrieve vesting information for the sender address

    let vesting_info = match get_vesting_info_by_key(&sender) {
        Some(info) => info,
        None => return true, // If sender is not a vesting address, allow transfer
    };

    let start_time = get_dictionary_value_from_key(DICT_START_TIME, vesting_info.vesting_address)
        .unwrap_or_default();
    let total_amount =
        get_dictionary_value_from_key(DICT_VESTING_AMOUNT, vesting_info.vesting_address)
            .unwrap_or_default();
    let current_time: u64 = get_blocktime().into();
    // Retrieve the vesting status for the sender
    let status = get_vesting_status(&vesting_info, start_time, total_amount, current_time);

    // Allow transfer if the amount is within the vested amount
    amount <= status.vested_amount
}

#[cfg(feature = "contract-support")]
fn calculate_time_until_next_release(
    start_time: u64,
    duration: Duration,
    current_time: u64,
) -> Duration {
    let elapsed_time = current_time.saturating_sub(start_time);
    let total_duration_secs = duration.whole_seconds() as u64; // Convert `Duration` to seconds

    if elapsed_time >= total_duration_secs {
        Duration::ZERO
    } else {
        // Calculate the next release point (start of the next month in seconds)
        let next_release_point = ((elapsed_time / MONTH_IN_SECONDS) + 1) * MONTH_IN_SECONDS;
        let remaining_time_secs = next_release_point.saturating_sub(elapsed_time);

        // Convert remaining time back to `Duration`
        Duration::seconds(remaining_time_secs as i64)
    }
}

#[cfg(feature = "contract-support")]
fn calculate_monthly_release(total_amount: U256, duration: Duration) -> U256 {
    let months = duration.whole_seconds() as u64 / MONTH_IN_SECONDS; // Approximate months in duration
    if months == 0 {
        U256::zero()
    } else {
        total_amount / U256::from(months)
    }
}

#[cfg(feature = "contract-support")]
fn calculate_linear_vesting(
    start_time: u64,
    duration: Duration,
    total_amount: U256,
    current_time: u64,
) -> U256 {
    let elapsed_time = current_time.saturating_sub(start_time);
    let duration_as_secs = duration.whole_seconds() as u64;
    if elapsed_time >= duration_as_secs {
        total_amount
    } else {
        let total_duration_secs = U256::from(duration_as_secs);
        let elapsed_secs = U256::from(elapsed_time);
        (total_amount * elapsed_secs) / total_duration_secs
    }
}

#[cfg(feature = "contract-support")]
pub fn calculate_vesting_allocations(initial_supply: U256) -> Vec<VestingAllocation> {
    VESTING_PERCENTAGES
        .iter()
        .filter_map(|(vesting_type, percentage)| {
            // Find the corresponding `VestingInfo` for each vesting type
            get_vesting_info()
                .iter()
                .find(|info| info.vesting_type == *vesting_type)
                .map(|info| {
                    let vesting_address = info.vesting_address;
                    let vesting_address_key = info
                        .maybe_vesting_address_key
                        .unwrap_or_revert_with(VestingError::MissingKey);
                    (vesting_address, vesting_address_key, percentage)
                })
        })
        .map(|(vesting_address, vesting_address_key, percentage)| {
            let vesting_amount = initial_supply
                .checked_mul(U256::from(*percentage))
                .unwrap_or_revert_with(VestingError::Overflow)
                .checked_div(U256::from(100))
                .unwrap_or_revert_with(VestingError::Overflow);

            // Create the VestingAllocation with the required fields
            VestingAllocation {
                vesting_address,
                vesting_address_key,
                vesting_amount,
            }
        })
        .collect()
}
