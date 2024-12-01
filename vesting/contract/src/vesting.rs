use crate::enums::VestingType;
#[cfg(feature = "contract-support")]
use crate::{
    constants::{MONTH_IN_SECONDS, SUFFIX_START_TIME, SUFFIX_VESTING_AMOUNT},
    enums::{VESTING_INFO, VESTING_PERCENTAGES},
};
#[cfg(feature = "contract-support")]
use crate::{error::VestingError, utils::get_stored_value};
#[cfg(feature = "contract-support")]
use alloc::format;
use alloc::{vec, vec::Vec};
#[cfg(feature = "contract-support")]
use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
#[cfg(feature = "contract-support")]
use casper_types::{bytesrepr::FromBytes, CLTyped};
use casper_types::{
    bytesrepr::{Error, ToBytes},
    Key, U256,
};
use time::Duration;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct VestingAddressInfo {
    pub vesting_type: VestingType,
    pub vesting_address: &'static str,
    pub maybe_vesting_address_key: Option<Key>,
    pub vesting_duration: Option<Duration>,
}

impl ToBytes for VestingAddressInfo {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();

        bytes.extend(self.vesting_type.to_bytes()?);
        bytes.extend(self.vesting_address.to_bytes()?);
        bytes.extend(self.maybe_vesting_address_key.to_bytes()?);
        if let Some(duration) = self.vesting_duration {
            bytes.extend(
                duration
                    .whole_seconds()
                    .to_bytes()
                    .unwrap_or_else(|_| vec![]),
            );
        }

        Ok(bytes)
    }

    fn serialized_length(&self) -> usize {
        self.vesting_type.serialized_length()
            + self.vesting_address.serialized_length()
            + self.maybe_vesting_address_key.serialized_length()
            + self.vesting_duration.map_or(0, |_| 8)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct VestingAllocation {
    pub vesting_address: &'static str,
    pub vesting_address_key: Key,
    pub vesting_amount: U256,
}

pub struct VestingStatus {
    pub total_amount: U256,
    pub vested_amount: U256,
    pub is_fully_vested: bool,
    pub vesting_duration: Duration,
    pub time_until_next_release: Duration,
    pub monthly_release: U256,
}

impl VestingStatus {
    pub fn new(
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

#[cfg(feature = "contract-support")]
pub fn get_vesting_info() -> Vec<VestingAddressInfo> {
    use crate::{constants::DICT_ADDRESSES, utils::get_dictionary_value_from_key};

    VESTING_INFO
        .iter()
        .map(|vesting_address_info| VestingAddressInfo {
            vesting_type: vesting_address_info.vesting_type,
            vesting_address: vesting_address_info.vesting_address,
            maybe_vesting_address_key: get_dictionary_value_from_key(
                DICT_ADDRESSES,
                vesting_address_info.vesting_address,
            ),
            vesting_duration: vesting_address_info.vesting_duration,
        })
        .collect()
}

#[cfg(feature = "contract-support")]
fn get_vesting_address_info_by_key(vesting_address_key: &Key) -> Option<VestingAddressInfo> {
    get_vesting_info()
        .into_iter()
        .find(|info| info.maybe_vesting_address_key.as_ref() == Some(vesting_address_key))
}

#[cfg(feature = "contract-support")]
pub fn get_vesting_address_info_by_type(vesting_type: VestingType) -> Option<VestingAddressInfo> {
    get_vesting_info()
        .into_iter()
        .find(|info| info.vesting_type == vesting_type)
}

#[cfg(feature = "contract-support")]
fn get_vesting_status(
    vesting_info: &VestingAddressInfo,
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
pub fn get_vesting_details(vesting_type: VestingType) -> (U256, U256, bool) {
    // Retrieve the vesting information based on the type
    let vesting_info = VESTING_INFO
        .iter()
        .find(|vesting_address_info| vesting_address_info.vesting_type == vesting_type)
        .expect("Invalid vesting type");

    // Read the required data for calculations
    let vesting_address_key = get_stored_value::<Key>(vesting_info.vesting_address);
    // TODO
    let start_time = read_stored_value_with_suffix(vesting_info.vesting_address, SUFFIX_START_TIME);
    let total_amount =
        read_stored_value_with_suffix(vesting_info.vesting_address, SUFFIX_VESTING_AMOUNT);

    let current_time: u64 = runtime::get_blocktime().into();

    // Calculate the vesting status
    let status = get_vesting_status(
        &VestingAddressInfo {
            vesting_address: vesting_info.vesting_address,
            vesting_type,
            vesting_duration: vesting_info.vesting_duration,
            maybe_vesting_address_key: Some(vesting_address_key),
        },
        start_time,
        total_amount,
        current_time,
    );

    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested,
    )
}

#[cfg(feature = "contract-support")]
pub fn check_vesting_transfer(sender: Key, amount: U256) -> bool {
    // Retrieve vesting information for the sender address
    let vesting_info = match get_vesting_address_info_by_key(&sender) {
        Some(info) => info,
        None => return true, // If sender is not a vesting address, allow transfer
    };

    // Retrieve the vesting status for the sender
    let start_time = read_stored_value_with_suffix(vesting_info.vesting_address, SUFFIX_START_TIME);
    let total_amount =
        read_stored_value_with_suffix(vesting_info.vesting_address, SUFFIX_VESTING_AMOUNT);
    let current_time: u64 = runtime::get_blocktime().into();

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
            // Find the corresponding `VestingAddressInfo` for each vesting type
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

#[cfg(feature = "contract-support")]
pub fn read_stored_value_with_suffix<T: FromBytes + CLTyped>(base_key: &str, suffix: &str) -> T {
    let key = format!("{}{}", base_key, suffix);
    get_stored_value::<T>(&key)
}
