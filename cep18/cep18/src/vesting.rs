use alloc::{
    string::String,
    vec, 
    vec::Vec,
    format,
};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{Key, U256};

use crate::{
    error::Cep18Error,
    utils::get_uref,
};

use crate::constants::{
    TREASURY_ADDRESS, TEAM_ADDRESS, STAKING_ADDRESS, INVESTOR_ADDRESS,
    NETWORK_ADDRESS, MARKETING_ADDRESS, AIRDROP_ADDRESS,
};

use casper_types::{CLTyped, bytesrepr::{ToBytes, FromBytes}};

// Vesting durations
const MONTH_IN_SECONDS: u64 = 2_628_000; // 30.4375 days average
const YEAR_IN_SECONDS: u64 = 31_536_000;  // 365 days

const TWO_YEARS: u64 = 2 * YEAR_IN_SECONDS;
const TEN_YEARS: u64 = 10 * YEAR_IN_SECONDS;
const TWO_YEARS_MONTHS: u64 = 24;  // For monthly calculations
const TEN_YEARS_MONTHS: u64 = 120; // For monthly calculations

// Use suffixes to differentiate storage keys from runtime args
const VESTING_AMOUNT_SUFFIX: &str = "_vesting_amount";
const START_TIME_SUFFIX: &str = "_start_time";

const TREASURY_LOCK_DURATION: u64 = TWO_YEARS;  // 2 years lock
const TEAM_VESTING_DURATION: u64 = TWO_YEARS;   // 2 years linear vesting
const STAKING_VESTING_DURATION: u64 = TEN_YEARS; // 10 years linear vesting
const INVESTOR_VESTING_DURATION: u64 = TWO_YEARS;   // 2 years linear vesting
const NETWORK_VESTING_DURATION: u64 = TWO_YEARS;    // 2 years linear vesting
const MARKETING_VESTING_DURATION: u64 = TWO_YEARS;  // 2 years linear vesting
const AIRDROP_VESTING_DURATION: u64 = TWO_YEARS;    // 2 years linear vesting

// Struct to hold vesting result
pub struct VestingAllocation {
    pub address: Key,
    pub amount: U256,
}

// Structure to hold vesting initialization data
struct VestingInit {
    address_key: &'static str,
    percentage: u8,
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

pub fn calculate_vesting_allocations(
    initial_supply: U256,
    treasury_address: Key,
    team_address: Key,
    staking_address: Key,
    investor_address: Key,
    network_address: Key,
    marketing_address: Key,
    airdrop_address: Key,
) -> Vec<VestingAllocation> {
    let vestings = [
        (treasury_address, VestingInit {
            address_key: TREASURY_ADDRESS,
            percentage: 50,
        }),
        (team_address, VestingInit {
            address_key: TEAM_ADDRESS,
            percentage: 7,
        }),
        (staking_address, VestingInit {
            address_key: STAKING_ADDRESS,
            percentage: 20,
        }),
        (investor_address, VestingInit {
            address_key: INVESTOR_ADDRESS,
            percentage: 10,
        }),
        (network_address, VestingInit {
            address_key: NETWORK_ADDRESS,
            percentage: 5,
        }),
        (marketing_address, VestingInit {
            address_key: MARKETING_ADDRESS,
            percentage: 5,
        }),
        (airdrop_address, VestingInit {
            address_key: AIRDROP_ADDRESS,
            percentage: 3,
        }),
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
            }
        })
        .collect()
}

fn get_vesting_amount_key(base_key: &str) -> String {
    format!("{}{}", base_key, VESTING_AMOUNT_SUFFIX)
}

fn get_start_time_key(base_key: &str) -> String {
    format!("{}{}", base_key, START_TIME_SUFFIX)
}

// Update helper functions to use the new key construction
fn read_vesting_amount(base_key: &str) -> U256 {
    let key = get_vesting_amount_key(base_key);
    storage::read(get_uref(&key))
        .unwrap_or_revert()
        .unwrap_or_revert()
}

fn read_start_time(base_key: &str) -> u64 {
    let key = get_start_time_key(base_key);
    storage::read(get_uref(&key))
        .unwrap_or_revert()
        .unwrap_or_revert()
}

// Helper function to calculate time until next release
fn calculate_time_until_next_release(start_time: u64) -> u64 {
    let current_time: u64 = runtime::get_blocktime().into();
    if current_time <= start_time {
        return MONTH_IN_SECONDS;
    }
    
    let time_elapsed = current_time - start_time;
    let months_elapsed = time_elapsed / MONTH_IN_SECONDS;
    let next_release = (months_elapsed + 1) * MONTH_IN_SECONDS + start_time;
    
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

pub fn init_vesting(
    total_supply: U256,
    treasury_address: Key,
    team_address: Key,
    staking_address: Key,
    investor_address: Key,
    network_address: Key,
    marketing_address: Key,
    airdrop_address: Key,
) {
    let start_time: u64 = runtime::get_blocktime().into();

    // Define all vestings
    let vestings = [
        (treasury_address, VestingInit {
            address_key: TREASURY_ADDRESS,
            percentage: 50,
        }),
        (team_address, VestingInit {
            address_key: TEAM_ADDRESS,
            percentage: 7,
        }),
        (staking_address, VestingInit {
            address_key: STAKING_ADDRESS,
            percentage: 20,
        }),
        (investor_address, VestingInit {
            address_key: INVESTOR_ADDRESS,
            percentage: 10,
        }),
        (network_address, VestingInit {
            address_key: NETWORK_ADDRESS,
            percentage: 5,
        }),
        (marketing_address, VestingInit {
            address_key: MARKETING_ADDRESS,
            percentage: 5,
        }),
        (airdrop_address, VestingInit {
            address_key: AIRDROP_ADDRESS,
            percentage: 3,
        }),
    ];

    // Initialize each vesting
    for (address, init) in vestings.iter() {
        let amount = total_supply
            .checked_mul(U256::from(init.percentage))
            .unwrap_or_revert_with(Cep18Error::Overflow)
            .checked_div(U256::from(100))
            .unwrap_or_revert_with(Cep18Error::Overflow);

        runtime::put_key(init.address_key, storage::new_uref(*address).into());
        runtime::put_key(
            &get_vesting_amount_key(init.address_key),
            storage::new_uref(amount).into()
        );
        runtime::put_key(
            &get_start_time_key(init.address_key),
            storage::new_uref(start_time).into()
        );
    }
}

// Helper function for linear vesting calculation
fn calculate_linear_vesting(
    start_time: u64,
    vesting_duration: u64,
    total_amount: U256,
) -> U256 {
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

fn is_treasury_address(address: &Key) -> bool {
    let treasury_address: Key = storage::read(get_uref(TREASURY_ADDRESS))
        .unwrap_or_revert()
        .unwrap_or_revert();
    *address == treasury_address
}

fn is_team_address(address: &Key) -> bool {
    let team_address: Key = storage::read(get_uref(TEAM_ADDRESS))
        .unwrap_or_revert()
        .unwrap_or_revert();
    *address == team_address
}

// Implementation of status checks for each vesting type
pub fn get_treasury_status() -> VestingStatus {
    let start_time = read_start_time(TREASURY_ADDRESS);
    let total_amount = read_vesting_amount(TREASURY_ADDRESS);
    
    let current_time: u64 = runtime::get_blocktime().into();
    let is_fully_vested = current_time - start_time >= TREASURY_LOCK_DURATION;
    let vested_amount = if is_fully_vested { total_amount } else { U256::zero() };
    
    VestingStatus::new(
        total_amount,
        vested_amount,
        is_fully_vested,
        TREASURY_LOCK_DURATION,
        if is_fully_vested { 0 } else { start_time + TREASURY_LOCK_DURATION - current_time },
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
    
    let vested_amount = calculate_linear_vesting(start_time, STAKING_VESTING_DURATION, total_amount);
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
    
    let vested_amount = calculate_linear_vesting(start_time, INVESTOR_VESTING_DURATION, total_amount);
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
    
    let vested_amount = calculate_linear_vesting(start_time, NETWORK_VESTING_DURATION, total_amount);
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
    
    let vested_amount = calculate_linear_vesting(start_time, MARKETING_VESTING_DURATION, total_amount);
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
    
    let vested_amount = calculate_linear_vesting(start_time, AIRDROP_VESTING_DURATION, total_amount);
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

// Helper type for vesting check functions
type VestingCheckFn = fn(U256) -> bool;

fn check_team_transfer(amount: U256) -> bool {
    let status = get_team_status();
    amount <= status.vested_amount
}

// For consistency, let's add similar helper functions for other vesting types
fn check_investor_transfer(amount: U256) -> bool {
    let status = get_investor_status();
    amount <= status.vested_amount
}

fn check_staking_transfer(amount: U256) -> bool {
    let status = get_staking_status();
    amount <= status.vested_amount
}

fn check_network_transfer(amount: U256) -> bool {
    let status = get_network_status();
    amount <= status.vested_amount
}

fn check_marketing_transfer(amount: U256) -> bool {
    let status = get_marketing_status();
    amount <= status.vested_amount
}

fn check_airdrop_transfer(amount: U256) -> bool {
    let status = get_airdrop_status();
    amount <= status.vested_amount
}

// Update check_vesting_transfer to use these helper functions
pub fn check_vesting_transfer(sender: Key, amount: U256) -> bool {
    // Check Treasury first (special case - no linear vesting)
    if is_treasury_address(&sender) {
        let status = get_treasury_status();
        return status.is_fully_vested;
    }

    // Check all linear vesting schedules
    if is_team_address(&sender) {
        return check_team_transfer(amount);
    }

    // Check other vesting types using their specific addresses and checks
    let vesting_configs: [(&str, VestingCheckFn); 5] = [
        (STAKING_ADDRESS, check_staking_transfer as VestingCheckFn),
        (INVESTOR_ADDRESS, check_investor_transfer as VestingCheckFn),
        (NETWORK_ADDRESS, check_network_transfer as VestingCheckFn),
        (MARKETING_ADDRESS, check_marketing_transfer as VestingCheckFn),
        (AIRDROP_ADDRESS, check_airdrop_transfer as VestingCheckFn),
    ];

    for (address_key, check_fn) in vesting_configs.iter() {
        let vesting_address: Key = storage::read(get_uref(address_key))
            .unwrap_or_revert()
            .unwrap_or_revert();
        
        if sender == vesting_address {
            return check_fn(amount);
        }
    }

    // If not a vesting address, allow transfer
    true
}

// Public getters for addresses
pub fn get_treasury_address() -> Key {
    storage::read(get_uref(TREASURY_ADDRESS))
        .unwrap_or_revert()
        .unwrap_or_revert()
}

pub fn get_team_address() -> Key {
    storage::read(get_uref(TEAM_ADDRESS))
        .unwrap_or_revert()
        .unwrap_or_revert()
}

pub fn get_treasury_vesting_details() -> (U256, U256, bool) {
    let status = get_treasury_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested
    )
}

pub fn get_team_vesting_details() -> (U256, U256, bool) {
    let status = get_team_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested
    )
}

pub fn get_staking_vesting_details() -> (U256, U256, bool) {
    let status = get_staking_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested
    )
}

pub fn get_investor_vesting_details() -> (U256, U256, bool) {
    let status = get_investor_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested
    )
}

pub fn get_network_vesting_details() -> (U256, U256, bool) {
    let status = get_network_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested
    )
}

pub fn get_marketing_vesting_details() -> (U256, U256, bool) {
    let status = get_marketing_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested
    )
}

pub fn get_airdrop_vesting_details() -> (U256, U256, bool) {
    let status = get_airdrop_status();
    (
        status.total_amount,
        status.vested_amount,
        status.is_fully_vested
    )
}