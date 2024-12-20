use crate::{
    constants::VESTING_PERIOD_IN_SECONDS, enums::VestingType,
    utils::display_human_readable_duration,
};
#[cfg(feature = "contract-support")]
use crate::{
    constants::{
        ARG_ADDRESS, DICT_ADDRESSES, DICT_START_TIME, DICT_TRANSFERRED_AMOUNT, DICT_VESTING_AMOUNT,
        DICT_VESTING_INFO, DICT_VESTING_STATUS, ENTRY_POINT_BALANCE_OF,
    },
    enums::{VESTING_INFO, VESTING_PERCENTAGES},
    error::VestingError,
    utils::{get_dictionary_value_from_key, set_dictionary_value_for_key},
};
#[cfg(feature = "contract-support")]
use alloc::string::ToString;
use alloc::{fmt, string::String, vec::Vec};
#[cfg(feature = "contract-support")]
use casper_contract::{
    contract_api::runtime::{call_versioned_contract, get_blocktime, ret},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::{Bytes, Error, FromBytes, ToBytes},
    CLType, CLTyped, Key, U256,
};
#[cfg(feature = "contract-support")]
use casper_types::{runtime_args, CLValue, ContractPackageHash, RuntimeArgs};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::Duration;

pub trait VestingData: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error>;
}

impl VestingData for U256 {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        // Using fully qualified syntax to avoid conflicts
        <U256 as FromBytes>::from_bytes(bytes)
    }
}

impl VestingData for String {
    fn from_bytes(value: &[u8]) -> Result<(Self, &[u8]), Error> {
        // Using fully qualified syntax to avoid conflicts
        <String as FromBytes>::from_bytes(value)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct VestingInfo {
    pub vesting_type: VestingType,
    pub maybe_vesting_address_key: Option<Key>,
    pub vesting_duration: Option<Duration>,
}

impl VestingInfo {
    // Helper function for shared formatting logic
    fn fmt_inner(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VestingInfo {{ vesting_type: {:?}, vesting_address_key: {:?}, vesting_duration: {:?}, vesting_period: {:?} }}",
            self.vesting_type,
            self.maybe_vesting_address_key,
            self.vesting_duration.map(|d| d.whole_seconds() as u64),
            VESTING_PERIOD_IN_SECONDS.whole_seconds() as u64,
        )
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (vesting_type, rem) = VestingType::from_bytes(bytes)?;
        let (maybe_vesting_address_key, rem) = Option::<Key>::from_bytes(rem)?;
        let (vesting_duration_opt, rem) = Option::<u64>::from_bytes(rem)?;

        let vesting_duration = vesting_duration_opt.map(|seconds| Duration::new(seconds as i64, 0));

        Ok((
            VestingInfo {
                vesting_type,
                maybe_vesting_address_key,
                vesting_duration,
            },
            rem,
        ))
    }
}

impl VestingData for VestingInfo {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        VestingInfo::from_bytes(bytes)
    }
}

impl FromBytes for VestingInfo {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        VestingInfo::from_bytes(bytes)
    }
}

impl fmt::Debug for VestingInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_inner(f)
    }
}

impl fmt::Display for VestingInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_inner(f)
    }
}

impl ToBytes for VestingInfo {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();

        bytes.extend(self.vesting_type.to_bytes()?);
        bytes.extend(self.maybe_vesting_address_key.to_bytes()?);

        match self.vesting_duration {
            Some(duration) => bytes.extend(Some(duration.whole_seconds() as u64).to_bytes()?),
            None => bytes.extend(Option::<u64>::None.to_bytes()?),
        }

        Ok(bytes)
    }

    fn serialized_length(&self) -> usize {
        self.vesting_type.serialized_length()
            + self.maybe_vesting_address_key.serialized_length()
            + Option::<u64>::serialized_length(
                &self.vesting_duration.map(|d| d.whole_seconds() as u64),
            )
    }
}

impl CLTyped for VestingInfo {
    fn cl_type() -> CLType {
        Bytes::cl_type()
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct VestingAllocation {
    pub vesting_type: VestingType,
    pub vesting_address_key: Key,
    pub vesting_amount: U256,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Copy)]
pub struct VestingStatus {
    pub vesting_type: VestingType,
    pub total_amount: U256,
    pub vested_amount: U256,
    pub is_fully_vested: bool,
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub vesting_duration: Duration,
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub start_time: Duration,
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub time_until_next_release: Duration,
    pub release_amount_per_period: U256,
    pub released_amount: U256,
    pub elapsed_periods: U256,
    pub available_for_release_amount: U256,
    pub total_to_release_amount: U256,
}

impl VestingStatus {
    fn fmt_inner(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VestingStatus {{ vesting_type: {:?}, total_amount: {:?}, vested_amount: {:?}, is_fully_vested: {:?}, vesting_duration: {:?}, start_time: {:?}, time_until_next_release: {:?}, until_next_release: {:?}, release_amount_per_period: {:?}, released_amount: {:?}, elapsed_periods: {:?}, available_for_release_amount: {:?} , total_to_release_amount: {:?} }}",
            self.vesting_type,
            self.total_amount,
            self.vested_amount,
            self.is_fully_vested,
            self.vesting_duration.whole_seconds(),  // Displaying seconds for duration
            self.start_time.whole_seconds(),  // Displaying seconds for duration
            self.time_until_next_release.whole_seconds(),  // Displaying seconds for duration
            display_human_readable_duration(self.time_until_next_release),
            self.release_amount_per_period,
            self.released_amount,
            self.elapsed_periods,
            self.available_for_release_amount,
            self.total_to_release_amount
        )
    }
}

impl fmt::Debug for VestingStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_inner(f)
    }
}

impl fmt::Display for VestingStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_inner(f)
    }
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_seconds_f64() as u64)
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds = u64::deserialize(deserializer)?;
    Ok(Duration::seconds(seconds as i64))
}

impl VestingStatus {
    #[allow(clippy::too_many_arguments)]
    fn new(
        vesting_type: VestingType,
        total_amount: U256,
        vested_amount: U256,
        is_fully_vested: bool,
        vesting_duration: Duration,
        start_time: Duration,
        time_until_next_release: Duration,
        release_amount_per_period: U256,
        released_amount: U256,
        elapsed_periods: U256,
        available_for_release_amount: U256,
        total_to_release_amount: U256,
    ) -> Self {
        Self {
            vesting_type,
            total_amount,
            vested_amount,
            is_fully_vested,
            vesting_duration,
            start_time,
            time_until_next_release,
            release_amount_per_period,
            released_amount,
            elapsed_periods,
            available_for_release_amount,
            total_to_release_amount,
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (vesting_type, bytes) = VestingType::from_bytes(bytes)?;
        let (total_amount, bytes) = <casper_types::U256 as FromBytes>::from_bytes(bytes)?;
        let (vested_amount, bytes) = <casper_types::U256 as FromBytes>::from_bytes(bytes)?;
        let (is_fully_vested, bytes) = bool::from_bytes(bytes)?;
        let (vesting_duration, bytes) = u64::from_bytes(bytes)?;
        let (start_time, bytes) = u64::from_bytes(bytes)?;
        let (time_until_next_release, bytes) = u64::from_bytes(bytes)?;
        let (release_amount_per_period, bytes) =
            <casper_types::U256 as FromBytes>::from_bytes(bytes)?;
        let (released_amount, bytes) = <casper_types::U256 as FromBytes>::from_bytes(bytes)?;
        let (elapsed_periods, bytes) = <casper_types::U256 as FromBytes>::from_bytes(bytes)?;
        let (available_for_release_amount, bytes) =
            <casper_types::U256 as FromBytes>::from_bytes(bytes)?;
        let (total_to_release_amount, bytes) =
            <casper_types::U256 as FromBytes>::from_bytes(bytes)?;

        let vesting_duration = Duration::new(vesting_duration as i64, 0);
        let start_time = Duration::new(start_time as i64, 0);
        let time_until_next_release = Duration::new(time_until_next_release as i64, 0);

        Ok((
            VestingStatus::new(
                vesting_type,
                total_amount,
                vested_amount,
                is_fully_vested,
                vesting_duration,
                start_time,
                time_until_next_release,
                release_amount_per_period,
                released_amount,
                elapsed_periods,
                available_for_release_amount,
                total_to_release_amount,
            ),
            bytes,
        ))
    }
}

impl CLTyped for VestingStatus {
    fn cl_type() -> CLType {
        Bytes::cl_type()
    }
}

impl VestingData for VestingStatus {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        VestingStatus::from_bytes(bytes)
    }
}

impl ToBytes for VestingStatus {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();

        // Serialize each field in the VestingStatus struct
        bytes.extend(self.vesting_type.to_bytes()?);
        bytes.extend(self.total_amount.to_bytes()?);
        bytes.extend(self.vested_amount.to_bytes()?);
        bytes.extend(self.is_fully_vested.to_bytes()?);
        bytes.extend((self.vesting_duration.whole_seconds() as u64).to_bytes()?);
        bytes.extend((self.start_time.whole_seconds() as u64).to_bytes()?);
        bytes.extend((self.time_until_next_release.whole_seconds() as u64).to_bytes()?);
        bytes.extend(self.release_amount_per_period.to_bytes()?);
        bytes.extend(self.released_amount.to_bytes()?);
        bytes.extend(self.elapsed_periods.to_bytes()?);
        bytes.extend(self.available_for_release_amount.to_bytes()?);
        bytes.extend(self.total_to_release_amount.to_bytes()?);
        Ok(bytes)
    }

    fn serialized_length(&self) -> usize {
        self.vesting_type.serialized_length()
            + self.total_amount.serialized_length()
            + self.vested_amount.serialized_length()
            + self.is_fully_vested.serialized_length()
            + (self.vesting_duration.whole_seconds() as u64).serialized_length()
            + (self.start_time.whole_seconds() as u64).serialized_length()
            + (self.time_until_next_release.whole_seconds() as u64).serialized_length()
            + self.release_amount_per_period.serialized_length()
            + self.released_amount.serialized_length()
            + self.elapsed_periods.serialized_length()
            + self.available_for_release_amount.serialized_length()
            + self.total_to_release_amount.serialized_length()
    }
}

impl FromBytes for VestingStatus {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        VestingStatus::from_bytes(bytes)
    }
}

#[cfg(feature = "contract-support")]
pub fn ret_vesting_status(vesting_type: VestingType) {
    let vesting_status = update_vesting_status(vesting_type);
    let result = CLValue::from_t(vesting_status).unwrap_or_revert();
    ret(result);
}

#[cfg(feature = "contract-support")]
pub fn update_vesting_status(vesting_type: VestingType) -> VestingStatus {
    let vesting_status = get_vesting_status_by_type(vesting_type);

    set_dictionary_value_for_key(
        DICT_VESTING_STATUS,
        &vesting_status.vesting_type.to_string(),
        &vesting_status,
    );
    vesting_status
}

#[cfg(feature = "contract-support")]
pub fn ret_vesting_info(vesting_type: VestingType) {
    let vesting_info = get_vesting_info_by_type(&vesting_type)
        .unwrap_or_revert_with(VestingError::InvalidVestingType);

    set_dictionary_value_for_key(DICT_VESTING_INFO, &vesting_type.to_string(), &vesting_info);

    let result = CLValue::from_t(vesting_info).unwrap_or_revert();
    ret(result);
}

#[cfg(feature = "contract-support")]
pub fn get_vesting_info() -> Vec<VestingInfo> {
    VESTING_INFO
        .iter()
        .map(|vesting_info| VestingInfo {
            vesting_type: vesting_info.vesting_type,
            maybe_vesting_address_key: get_dictionary_value_from_key(
                DICT_ADDRESSES,
                &vesting_info.vesting_type.to_string(),
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
fn get_vesting_info_by_type(vesting_type: &VestingType) -> Option<VestingInfo> {
    get_vesting_info()
        .into_iter()
        .find(|info| info.vesting_type == *vesting_type)
}

#[cfg(feature = "contract-support")]
fn get_vesting_status(
    vesting_info: &VestingInfo,
    start_time: u64,
    total_amount: U256,
) -> VestingStatus {
    let start_time_in_ms: u64 = get_blocktime().into();
    let current_time = start_time_in_ms.checked_div(1000).unwrap_or_default();

    #[allow(clippy::match_single_binding)]
    let vested_amount = match vesting_info.vesting_type {
        _ => {
            if let Some(duration) = vesting_info.vesting_duration {
                calculate_linear_vesting(start_time, duration, total_amount, current_time)
            } else {
                U256::zero() // Default to no vesting if duration is None
            }
        }
    };

    let is_fully_vested = vesting_info.vesting_duration.is_none() || vested_amount == total_amount;
    let time_until_next_release = if is_fully_vested {
        Duration::ZERO
    } else {
        vesting_info
            .vesting_duration
            .map_or(Duration::ZERO, |duration| {
                calculate_time_until_next_release(start_time, duration, current_time)
            })
    };

    let release_amount_per_period = if let Some(duration) = vesting_info.vesting_duration {
        calculate_release_per_period(total_amount, duration)
    } else {
        U256::zero()
    };

    let released_amount = get_dictionary_value_from_key(
        DICT_TRANSFERRED_AMOUNT,
        &vesting_info.vesting_type.to_string(),
    )
    .unwrap_or_default();

    let elapsed_time = current_time.saturating_sub(start_time);
    let elapsed_time = Duration::seconds(elapsed_time as i64);

    let elapsed_periods = if elapsed_time >= VESTING_PERIOD_IN_SECONDS {
        U256::from(
            elapsed_time
                .whole_seconds()
                .checked_div(VESTING_PERIOD_IN_SECONDS.whole_seconds())
                .unwrap_or_default(),
        )
    } else {
        U256::zero()
    };

    let expected_released_amount = if is_fully_vested {
        // if elapsed_periods is above full vesting time we don't want to calculate expected_released_amount based on periods
        total_amount
    } else {
        release_amount_per_period * elapsed_periods
    };

    let available_for_release_amount = if is_fully_vested {
        expected_released_amount
            .saturating_sub(released_amount)
            .min(total_amount)
    } else if expected_released_amount >= released_amount {
        expected_released_amount.saturating_sub(released_amount)
    } else {
        U256::zero()
    };

    let total_to_release_amount = total_amount.saturating_sub(released_amount);

    VestingStatus::new(
        vesting_info.vesting_type,
        total_amount,
        vested_amount,
        is_fully_vested,
        vesting_info.vesting_duration.unwrap_or(Duration::ZERO),
        Duration::new(start_time as i64, 0),
        time_until_next_release,
        release_amount_per_period,
        released_amount,
        elapsed_periods,
        available_for_release_amount,
        total_to_release_amount,
    )
}

#[cfg(feature = "contract-support")]
fn get_vesting_status_by_type(vesting_type: VestingType) -> VestingStatus {
    let vesting_info = VESTING_INFO
        .iter()
        .find(|vesting_info| vesting_info.vesting_type == vesting_type)
        .expect("Invalid vesting type");

    let maybe_vesting_address_key =
        get_dictionary_value_from_key(DICT_ADDRESSES, &vesting_info.vesting_type.to_string());
    let start_time =
        get_dictionary_value_from_key(DICT_START_TIME, &vesting_info.vesting_type.to_string())
            .unwrap_or_default();
    let total_amount =
        get_dictionary_value_from_key(DICT_VESTING_AMOUNT, &vesting_info.vesting_type.to_string())
            .unwrap_or_default();

    // Calculate the vesting status
    get_vesting_status(
        &VestingInfo {
            vesting_type,
            vesting_duration: vesting_info.vesting_duration,
            maybe_vesting_address_key,
        },
        start_time,
        total_amount,
    )
}

#[cfg(feature = "contract-support")]
pub fn get_vesting_transfer(owner: Key, requested_amount: U256) -> bool {
    let vesting_info = match get_vesting_info_by_key(&owner) {
        Some(info) => info,
        None => return true, // If owner is not a vesting address, allow transfer
    };
    let start_time =
        get_dictionary_value_from_key(DICT_START_TIME, &vesting_info.vesting_type.to_string())
            .unwrap_or_default();
    let total_amount =
        get_dictionary_value_from_key(DICT_VESTING_AMOUNT, &vesting_info.vesting_type.to_string())
            .unwrap_or_default();

    let status = get_vesting_status(&vesting_info, start_time, total_amount);

    if requested_amount <= status.available_for_release_amount {
        // Update transferred amount if all checks pass
        let cumulative_transferred: U256 = get_dictionary_value_from_key(
            DICT_TRANSFERRED_AMOUNT,
            &vesting_info.vesting_type.to_string(),
        )
        .unwrap_or_default();
        let new_transferred_amount = cumulative_transferred + requested_amount;
        set_dictionary_value_for_key(
            DICT_TRANSFERRED_AMOUNT,
            &vesting_info.vesting_type.to_string(),
            &new_transferred_amount,
        );
        update_vesting_status(vesting_info.vesting_type);
        return true;
    }
    status.is_fully_vested
}

#[cfg(feature = "contract-support")]
pub fn get_current_balance_for_key(
    contract_package_hash: ContractPackageHash,
    owner: &Key,
) -> U256 {
    call_versioned_contract(
        contract_package_hash,
        None,
        ENTRY_POINT_BALANCE_OF,
        runtime_args! {ARG_ADDRESS => owner },
    )
}

#[cfg(feature = "contract-support")]
fn calculate_time_until_next_release(
    start_time: u64,
    duration: Duration,
    current_time: u64,
) -> Duration {
    let elapsed = current_time.saturating_sub(start_time);
    let total_duration = duration.whole_seconds() as u64;
    let vesting_period = VESTING_PERIOD_IN_SECONDS.whole_seconds() as u64;

    if elapsed >= total_duration {
        return Duration::ZERO;
    }

    // Perform modulo on raw seconds
    let time_in_period = elapsed % vesting_period;
    let remaining_time_secs = vesting_period.saturating_sub(time_in_period);

    Duration::seconds(remaining_time_secs as i64)
}

#[cfg(feature = "contract-support")]
fn calculate_release_per_period(total_amount: U256, duration: Duration) -> U256 {
    let periods = (duration.whole_seconds() as u64)
        .checked_div(VESTING_PERIOD_IN_SECONDS.whole_seconds() as u64)
        .unwrap_or(0);

    if periods == 0 || total_amount.is_zero() {
        return U256::zero();
    }

    total_amount
        .checked_div(U256::from(periods))
        .unwrap_or(U256::zero())
}

#[cfg(feature = "contract-support")]
fn calculate_linear_vesting(
    start_time: u64,
    duration: Duration,
    total_amount: U256,
    current_time: u64,
) -> U256 {
    let elapsed_time = current_time.saturating_sub(start_time);
    let total_duration = duration.whole_seconds() as u64;
    let vesting_period = VESTING_PERIOD_IN_SECONDS.whole_seconds() as u64;

    if elapsed_time == 0_u64 {
        return U256::zero();
    }

    if elapsed_time >= total_duration {
        return total_amount;
    }

    let total_periods = total_duration
        .checked_div(vesting_period)
        .unwrap_or_default();
    let elapsed_periods = elapsed_time.checked_div(vesting_period).unwrap_or_default();

    let amount_per_period = total_amount
        .checked_div(U256::from(total_periods))
        .unwrap_or_default();

    amount_per_period * U256::from(elapsed_periods)
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
                    let vesting_address_key = info
                        .maybe_vesting_address_key
                        .unwrap_or_revert_with(VestingError::MissingKey);
                    (vesting_type, vesting_address_key, percentage)
                })
        })
        .map(|(vesting_type, vesting_address_key, percentage)| {
            let vesting_amount = initial_supply
                .checked_mul(U256::from(*percentage))
                .unwrap_or_revert_with(VestingError::Overflow)
                .checked_div(U256::from(100))
                .unwrap_or_revert_with(VestingError::Overflow);

            // Create the VestingAllocation with the required fields
            VestingAllocation {
                vesting_type: *vesting_type,
                vesting_address_key,
                vesting_amount,
            }
        })
        .collect()
}
