use alloc::{format, string::String, vec::Vec};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{Key, U256};

use crate::{error::Cep18Error, utils::get_uref};

use crate::constants::{
    AIRDROP_ADDRESS, INVESTOR_ADDRESS, LIQUIDITY_ADDRESS, MARKETING_ADDRESS, NETWORK_ADDRESS,
    STAKING_ADDRESS, TEAM_ADDRESS, TREASURY_ADDRESS,
};

// Vesting durations
const MONTH_IN_SECONDS: u64 = 2_628_000; // 30.4375 days average
const YEAR_IN_SECONDS: u64 = 31_536_000; // 365 days

pub const TWO_YEARS: u64 = 2 * YEAR_IN_SECONDS;
pub const TEN_YEARS: u64 = 10 * YEAR_IN_SECONDS;
pub const TWO_YEARS_MONTHS: u64 = 24; // For monthly calculations
pub const TEN_YEARS_MONTHS: u64 = 120; // For monthly calculations

// Use suffixes to differentiate storage keys from runtime args
pub const VESTING_AMOUNT_SUFFIX: &str = "_vesting_amount";
pub const START_TIME_SUFFIX: &str = "_start_time";

// // Allocation Address Lock Durations
// const TREASURY_LOCK_DURATION: u64 = TWO_YEARS;  // 2 years lock
// const TEAM_VESTING_DURATION: u64 = TWO_YEARS;   // 2 years linear vesting
// const STAKING_VESTING_DURATION: u64 = TEN_YEARS; // 10 years linear vesting
// const INVESTOR_VESTING_DURATION: u64 = TWO_YEARS;   // 2 years linear vesting
// const NETWORK_VESTING_DURATION: u64 = TWO_YEARS;    // 2 years linear vesting
// const MARKETING_VESTING_DURATION: u64 = TWO_YEARS;  // 2 years linear vesting
// const AIRDROP_VESTING_DURATION: u64 = TWO_YEARS;    // 2 years linear vesting

// for testing
pub const MINUTE_IN_SECONDS: u64 = 60;
pub const TEN_MINUTES: u64 = 10 * MINUTE_IN_SECONDS;
// Modified vesting durations for testing
const TEST_TWO_YEARS: u64 = 24 * TEN_MINUTES; // 24 ten-minute periods to simulate 2 years
const TEST_TEN_YEARS: u64 = 120 * TEN_MINUTES; // 120 ten-minute periods to simulate 10 years
                                               // Modified allocation durations for testing
const TREASURY_LOCK_DURATION: u64 = TEST_TWO_YEARS; // 2 years (24 periods) lock
const TEAM_VESTING_DURATION: u64 = TEST_TWO_YEARS; // 2 years (24 periods) linear vesting
const STAKING_VESTING_DURATION: u64 = TEST_TEN_YEARS; // 10 years (120 periods) linear vesting
const INVESTOR_VESTING_DURATION: u64 = TEST_TWO_YEARS; // 2 years linear vesting
const NETWORK_VESTING_DURATION: u64 = TEST_TWO_YEARS; // 2 years linear vesting
const MARKETING_VESTING_DURATION: u64 = TEST_TWO_YEARS; // 2 years linear vesting
const AIRDROP_VESTING_DURATION: u64 = TEST_TWO_YEARS; // 2 years linear vesting

// Struct to hold vesting result
pub struct VestingAllocation {
    pub address: Key,
    pub amount: U256,
    pub storage_key: &'static str,
}

// Structure to hold vesting initialization data
struct VestingInit {
    percentage: u8,
    storage_key: &'static str,
}

// Helper struct for vesting status
pub struct VestingStatus {
    pub total_amount: U256,
    pub vested_amount: U256,
    pub is_fully_vested: bool,
    pub vesting_duration: u64,
    pub time_until_next_release: u64,
    pub monthly_release: U256,
}

// Add this struct to hold address information
#[derive(Clone, Copy)]
pub struct VestingAddressInfo {
    pub address: Key,
    pub address_type: &'static str,
    pub duration: u64,
}

impl VestingStatus {
    pub fn new(
        total_amount: U256,
        vested_amount: U256,
        is_fully_vested: bool,
        vesting_duration: u64,
        time_until_next: u64,
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

pub fn get_all_vesting_addresses() -> Vec<VestingAddressInfo> {
    [
        VestingAddressInfo {
            address: storage::read(get_uref(TREASURY_ADDRESS))
                .unwrap_or_revert()
                .unwrap_or_revert(),
            address_type: TREASURY_ADDRESS,
            duration: TREASURY_LOCK_DURATION,
        },
        VestingAddressInfo {
            address: storage::read(get_uref(TEAM_ADDRESS))
                .unwrap_or_revert()
                .unwrap_or_revert(),
            address_type: TEAM_ADDRESS,
            duration: TEAM_VESTING_DURATION,
        },
        VestingAddressInfo {
            address: storage::read(get_uref(STAKING_ADDRESS))
                .unwrap_or_revert()
                .unwrap_or_revert(),
            address_type: STAKING_ADDRESS,
            duration: STAKING_VESTING_DURATION,
        },
        VestingAddressInfo {
            address: storage::read(get_uref(INVESTOR_ADDRESS))
                .unwrap_or_revert()
                .unwrap_or_revert(),
            address_type: INVESTOR_ADDRESS,
            duration: INVESTOR_VESTING_DURATION,
        },
        VestingAddressInfo {
            address: storage::read(get_uref(NETWORK_ADDRESS))
                .unwrap_or_revert()
                .unwrap_or_revert(),
            address_type: NETWORK_ADDRESS,
            duration: NETWORK_VESTING_DURATION,
        },
        VestingAddressInfo {
            address: storage::read(get_uref(MARKETING_ADDRESS))
                .unwrap_or_revert()
                .unwrap_or_revert(),
            address_type: MARKETING_ADDRESS,
            duration: MARKETING_VESTING_DURATION,
        },
        VestingAddressInfo {
            address: storage::read(get_uref(AIRDROP_ADDRESS))
                .unwrap_or_revert()
                .unwrap_or_revert(),
            address_type: AIRDROP_ADDRESS,
            duration: AIRDROP_VESTING_DURATION,
        },
    ]
    .to_vec()
}

// Helper function to get vesting info for a specific address
pub fn get_vesting_info(address: &Key) -> Option<VestingAddressInfo> {
    get_all_vesting_addresses()
        .into_iter()
        .find(|info| &info.address == address)
}

pub fn calculate_vesting_allocations(
    initial_supply: U256,
    treasury_address: Key,
    team_address: Key,
    staking_address: Key,
    investor_address: Key,
    network_address: Key,
    marketing_address: Key,
    airdrop_address: Key,
    liquidity_address: Key,
) -> Vec<VestingAllocation> {
    let vestings = [
        (
            treasury_address,
            VestingInit {
                percentage: 30,
                storage_key: TREASURY_ADDRESS,
            },
        ),
        (
            team_address,
            VestingInit {
                percentage: 7,
                storage_key: TEAM_ADDRESS,
            },
        ),
        (
            staking_address,
            VestingInit {
                percentage: 20,
                storage_key: STAKING_ADDRESS,
            },
        ),
        (
            investor_address,
            VestingInit {
                percentage: 10,
                storage_key: INVESTOR_ADDRESS,
            },
        ),
        (
            network_address,
            VestingInit {
                percentage: 5,
                storage_key: NETWORK_ADDRESS,
            },
        ),
        (
            marketing_address,
            VestingInit {
                percentage: 5,
                storage_key: MARKETING_ADDRESS,
            },
        ),
        (
            airdrop_address,
            VestingInit {
                percentage: 3,
                storage_key: AIRDROP_ADDRESS,
            },
        ),
        (
            liquidity_address,
            VestingInit {
                percentage: 20,
                storage_key: LIQUIDITY_ADDRESS,
            },
        ),
    ];

    vestings
        .iter()
        .map(|(address, init)| {
            let amount = if init.percentage == 50 {
                initial_supply
                    .checked_div(U256::from(2))
                    .unwrap_or_revert_with(Cep18Error::Overflow)
            } else {
                initial_supply
                    .checked_mul(U256::from(init.percentage))
                    .unwrap_or_revert_with(Cep18Error::Overflow)
                    .checked_div(U256::from(100))
                    .unwrap_or_revert_with(Cep18Error::Overflow)
            };

            VestingAllocation {
                address: *address,
                amount,
                storage_key: init.storage_key,
            }
        })
        .collect()
}

pub fn get_vesting_amount_key(base_key: &str) -> String {
    format!("{}{}", base_key, VESTING_AMOUNT_SUFFIX)
}

pub fn get_start_time_key(base_key: &str) -> String {
    format!("{}{}", base_key, START_TIME_SUFFIX)
}

// Update helper functions to use the new key construction
fn read_vesting_amount(base_key: &str) -> U256 {
    let key = get_vesting_amount_key(base_key);
    storage::read(get_uref(&key))
        .unwrap_or_revert()
        .unwrap_or_revert()
}

pub fn read_start_time(base_key: &str) -> u64 {
    let key = get_start_time_key(base_key);
    storage::read(get_uref(&key))
        .unwrap_or_revert()
        .unwrap_or_revert()
}

// Helper function to calculate time until next release
fn calculate_time_until_next_release(start_time: u64) -> u64 {
    let current_time: u64 = runtime::get_blocktime().into();
    if current_time <= start_time {
        // return MONTH_IN_SECONDS;
        return TEN_MINUTES;
    }

    let time_elapsed = current_time - start_time;
    // let months_elapsed = time_elapsed / MONTH_IN_SECONDS;
    // let next_release = (months_elapsed + 1) * MONTH_IN_SECONDS + start_time;
    let periods_elapsed = time_elapsed / TEN_MINUTES;
    let next_release = (periods_elapsed + 1) * TEN_MINUTES + start_time;

    if next_release <= current_time {
        0
    } else {
        next_release - current_time
    }
}

// Helper function to calculate monthly release amount
fn calculate_monthly_release(total_amount: U256, duration_months: u64) -> U256 {
    total_amount
        .checked_div(U256::from(duration_months))
        .unwrap_or_revert_with(Cep18Error::Overflow)
}

// Helper function for linear vesting calculation
fn calculate_linear_vesting(start_time: u64, vesting_duration: u64, total_amount: U256) -> U256 {
    let current_time: u64 = runtime::get_blocktime().into();

    if current_time <= start_time {
        return U256::zero();
    }

    let time_elapsed = current_time - start_time;
    if time_elapsed >= vesting_duration {
        return total_amount;
    }

    total_amount
        .checked_mul(U256::from(time_elapsed))
        .unwrap_or_revert()
        .checked_div(U256::from(vesting_duration))
        .unwrap_or_revert()
}

// Implementation of status checks for each vesting type
pub fn get_treasury_status() -> VestingStatus {
    let start_time = read_start_time(TREASURY_ADDRESS);
    let total_amount = read_vesting_amount(TREASURY_ADDRESS);

    let current_time: u64 = runtime::get_blocktime().into();
    let is_fully_vested = current_time - start_time >= TREASURY_LOCK_DURATION;
    let vested_amount = if is_fully_vested {
        total_amount
    } else {
        U256::zero()
    };

    VestingStatus::new(
        total_amount,
        vested_amount,
        is_fully_vested,
        TREASURY_LOCK_DURATION,
        if is_fully_vested {
            0
        } else {
            start_time + TREASURY_LOCK_DURATION - current_time
        },
        U256::zero(), // Treasury has no monthly release
    )
}

pub fn get_team_status() -> VestingStatus {
    let start_time = read_start_time(TEAM_ADDRESS);
    let total_amount = read_vesting_amount(TEAM_ADDRESS);

    let vested_amount = calculate_linear_vesting(start_time, TEAM_VESTING_DURATION, total_amount);
    let is_fully_vested = vested_amount == total_amount;
    let monthly_release = calculate_monthly_release(total_amount, TWO_YEARS_MONTHS);

    VestingStatus::new(
        total_amount,
        vested_amount,
        is_fully_vested,
        TEAM_VESTING_DURATION,
        calculate_time_until_next_release(start_time),
        monthly_release,
    )
}

pub fn get_staking_status() -> VestingStatus {
    let start_time = read_start_time(STAKING_ADDRESS);
    let total_amount = read_vesting_amount(STAKING_ADDRESS);

    let vested_amount =
        calculate_linear_vesting(start_time, STAKING_VESTING_DURATION, total_amount);
    let is_fully_vested = vested_amount == total_amount;
    let monthly_release = calculate_monthly_release(total_amount, TEN_YEARS_MONTHS);

    VestingStatus::new(
        total_amount,
        vested_amount,
        is_fully_vested,
        STAKING_VESTING_DURATION,
        calculate_time_until_next_release(start_time),
        monthly_release,
    )
}

pub fn get_investor_status() -> VestingStatus {
    let start_time = read_start_time(INVESTOR_ADDRESS);
    let total_amount = read_vesting_amount(INVESTOR_ADDRESS);

    let vested_amount =
        calculate_linear_vesting(start_time, INVESTOR_VESTING_DURATION, total_amount);
    let is_fully_vested = vested_amount == total_amount;
    let monthly_release = calculate_monthly_release(total_amount, TWO_YEARS_MONTHS);

    VestingStatus::new(
        total_amount,
        vested_amount,
        is_fully_vested,
        INVESTOR_VESTING_DURATION,
        calculate_time_until_next_release(start_time),
        monthly_release,
    )
}

pub fn get_network_status() -> VestingStatus {
    let start_time = read_start_time(NETWORK_ADDRESS);
    let total_amount = read_vesting_amount(NETWORK_ADDRESS);

    let vested_amount =
        calculate_linear_vesting(start_time, NETWORK_VESTING_DURATION, total_amount);
    let is_fully_vested = vested_amount == total_amount;
    let monthly_release = calculate_monthly_release(total_amount, TWO_YEARS_MONTHS);

    VestingStatus::new(
        total_amount,
        vested_amount,
        is_fully_vested,
        NETWORK_VESTING_DURATION,
        calculate_time_until_next_release(start_time),
        monthly_release,
    )
}

pub fn get_marketing_status() -> VestingStatus {
    let start_time = read_start_time(MARKETING_ADDRESS);
    let total_amount = read_vesting_amount(MARKETING_ADDRESS);

    let vested_amount =
        calculate_linear_vesting(start_time, MARKETING_VESTING_DURATION, total_amount);
    let is_fully_vested = vested_amount == total_amount;
    let monthly_release = calculate_monthly_release(total_amount, TWO_YEARS_MONTHS);

    VestingStatus::new(
        total_amount,
        vested_amount,
        is_fully_vested,
        MARKETING_VESTING_DURATION,
        calculate_time_until_next_release(start_time),
        monthly_release,
    )
}

pub fn get_airdrop_status() -> VestingStatus {
    let start_time = read_start_time(AIRDROP_ADDRESS);
    let total_amount = read_vesting_amount(AIRDROP_ADDRESS);

    let vested_amount =
        calculate_linear_vesting(start_time, AIRDROP_VESTING_DURATION, total_amount);
    let is_fully_vested = vested_amount == total_amount;
    let monthly_release = calculate_monthly_release(total_amount, TWO_YEARS_MONTHS);

    VestingStatus::new(
        total_amount,
        vested_amount,
        is_fully_vested,
        AIRDROP_VESTING_DURATION,
        calculate_time_until_next_release(start_time),
        monthly_release,
    )
}

// This could replace the current check_vesting_transfer function:
pub fn check_vesting_transfer(sender: Key, amount: U256) -> bool {
    let vesting_info = match get_vesting_info(&sender) {
        Some(info) => info,
        None => return true, // Not a vesting address
    };

    // Special case for treasury
    if vesting_info.address_type == TREASURY_ADDRESS {
        let status = get_treasury_status();
        return status.is_fully_vested;
    }

    // Get the appropriate status based on address type
    let status = match vesting_info.address_type {
        TEAM_ADDRESS => get_team_status(),
        STAKING_ADDRESS => get_staking_status(),
        INVESTOR_ADDRESS => get_investor_status(),
        NETWORK_ADDRESS => get_network_status(),
        MARKETING_ADDRESS => get_marketing_status(),
        AIRDROP_ADDRESS => get_airdrop_status(),
        _ => return true, // Shouldn't happen, but being safe
    };

    amount <= status.vested_amount
}

pub fn get_treasury_vesting_details() -> (U256, U256, bool) {
    let status = get_treasury_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested,
    )
}

pub fn get_team_vesting_details() -> (U256, U256, bool) {
    let status = get_team_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested,
    )
}

pub fn get_staking_vesting_details() -> (U256, U256, bool) {
    let status = get_staking_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested,
    )
}

pub fn get_investor_vesting_details() -> (U256, U256, bool) {
    let status = get_investor_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested,
    )
}

pub fn get_network_vesting_details() -> (U256, U256, bool) {
    let status = get_network_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested,
    )
}

pub fn get_marketing_vesting_details() -> (U256, U256, bool) {
    let status = get_marketing_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested,
    )
}

pub fn get_airdrop_vesting_details() -> (U256, U256, bool) {
    let status = get_airdrop_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested,
    )
}
