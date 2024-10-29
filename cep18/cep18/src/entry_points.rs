//! Contains definition of the entry points.
use alloc::{string::String, vec, vec::Vec, boxed::Box};

use casper_types::{
    CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key, Parameter,
    U256,
};

use crate::constants::{
    ADDRESS, ALLOWANCE_ENTRY_POINT_NAME, AMOUNT, APPROVE_ENTRY_POINT_NAME,
    BALANCE_OF_ENTRY_POINT_NAME, BURN_ENTRY_POINT_NAME, CHANGE_SECURITY_ENTRY_POINT_NAME,
    DECIMALS_ENTRY_POINT_NAME, DECREASE_ALLOWANCE_ENTRY_POINT_NAME,
    INCREASE_ALLOWANCE_ENTRY_POINT_NAME, INIT_ENTRY_POINT_NAME, MINT_ENTRY_POINT_NAME,
    NAME_ENTRY_POINT_NAME, OWNER, RECIPIENT, SPENDER, SYMBOL_ENTRY_POINT_NAME,
    TOTAL_SUPPLY_ENTRY_POINT_NAME, TRANSFER_ENTRY_POINT_NAME, TRANSFER_FROM_ENTRY_POINT_NAME,
    TREASURY_STATUS_ENTRY_POINT_NAME, TEAM_STATUS_ENTRY_POINT_NAME, STAKING_STATUS_ENTRY_POINT_NAME,
    INVESTOR_STATUS_ENTRY_POINT_NAME, NETWORK_STATUS_ENTRY_POINT_NAME, MARKETING_STATUS_ENTRY_POINT_NAME,
    AIRDROP_STATUS_ENTRY_POINT_NAME, VESTING_DETAILS_ENTRY_POINT
};

/// Returns the `name` entry point.
pub fn name() -> EntryPoint {
    EntryPoint::new(
        String::from(NAME_ENTRY_POINT_NAME),
        Vec::new(),
        String::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `symbol` entry point.
pub fn symbol() -> EntryPoint {
    EntryPoint::new(
        String::from(SYMBOL_ENTRY_POINT_NAME),
        Vec::new(),
        String::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `transfer_from` entry point.
pub fn transfer_from() -> EntryPoint {
    EntryPoint::new(
        String::from(TRANSFER_FROM_ENTRY_POINT_NAME),
        vec![
            Parameter::new(OWNER, Key::cl_type()),
            Parameter::new(RECIPIENT, Key::cl_type()),
            Parameter::new(AMOUNT, U256::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `allowance` entry point.
pub fn allowance() -> EntryPoint {
    EntryPoint::new(
        String::from(ALLOWANCE_ENTRY_POINT_NAME),
        vec![
            Parameter::new(OWNER, Key::cl_type()),
            Parameter::new(SPENDER, Key::cl_type()),
        ],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `approve` entry point.
pub fn approve() -> EntryPoint {
    EntryPoint::new(
        String::from(APPROVE_ENTRY_POINT_NAME),
        vec![
            Parameter::new(SPENDER, Key::cl_type()),
            Parameter::new(AMOUNT, U256::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `increase_allowance` entry point.
pub fn increase_allowance() -> EntryPoint {
    EntryPoint::new(
        String::from(INCREASE_ALLOWANCE_ENTRY_POINT_NAME),
        vec![
            Parameter::new(SPENDER, Key::cl_type()),
            Parameter::new(AMOUNT, U256::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `decrease_allowance` entry point.
pub fn decrease_allowance() -> EntryPoint {
    EntryPoint::new(
        String::from(DECREASE_ALLOWANCE_ENTRY_POINT_NAME),
        vec![
            Parameter::new(SPENDER, Key::cl_type()),
            Parameter::new(AMOUNT, U256::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `transfer` entry point.
pub fn transfer() -> EntryPoint {
    EntryPoint::new(
        String::from(TRANSFER_ENTRY_POINT_NAME),
        vec![
            Parameter::new(RECIPIENT, Key::cl_type()),
            Parameter::new(AMOUNT, U256::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `balance_of` entry point.
pub fn balance_of() -> EntryPoint {
    EntryPoint::new(
        String::from(BALANCE_OF_ENTRY_POINT_NAME),
        vec![Parameter::new(ADDRESS, Key::cl_type())],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `total_supply` entry point.
pub fn total_supply() -> EntryPoint {
    EntryPoint::new(
        String::from(TOTAL_SUPPLY_ENTRY_POINT_NAME),
        Vec::new(),
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `decimals` entry point.
pub fn decimals() -> EntryPoint {
    EntryPoint::new(
        String::from(DECIMALS_ENTRY_POINT_NAME),
        Vec::new(),
        u8::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `burn` entry point.
pub fn burn() -> EntryPoint {
    EntryPoint::new(
        String::from(BURN_ENTRY_POINT_NAME),
        vec![
            Parameter::new(OWNER, Key::cl_type()),
            Parameter::new(AMOUNT, U256::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `mint` entry point.
pub fn mint() -> EntryPoint {
    EntryPoint::new(
        String::from(MINT_ENTRY_POINT_NAME),
        vec![
            Parameter::new(OWNER, Key::cl_type()),
            Parameter::new(AMOUNT, U256::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `change_security` entry point.
pub fn change_security() -> EntryPoint {
    EntryPoint::new(
        String::from(CHANGE_SECURITY_ENTRY_POINT_NAME),
        vec![
            // Optional Arguments (can be added or omitted when calling):
            /*
            - "admin_list" : Vec<Key>
            - "mint_and_burn_list" : Vec<Key>
            - "minter_list" : Vec<Key>
            - "burner_list" : Vec<Key>
            - "none_list" : Vec<Key>
            */
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `init` entry point.
pub fn init() -> EntryPoint {
    EntryPoint::new(
        String::from(INIT_ENTRY_POINT_NAME),
        Vec::new(),
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

// Single entry point for vesting details with proper boxing
pub fn vesting_details() -> EntryPoint {
    EntryPoint::new(
        String::from(VESTING_DETAILS_ENTRY_POINT),
        vec![Parameter::new("vesting_type", CLType::String)],
        CLType::Tuple3([
            Box::new(CLType::U256),  // total_amount
            Box::new(CLType::U256),  // vested_amount
            Box::new(CLType::Bool),  // is_fully_vested
        ]),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

// Add entry points for each vesting status query
pub fn treasury_status() -> EntryPoint {
    EntryPoint::new(
        String::from(TREASURY_STATUS_ENTRY_POINT_NAME),
        Vec::new(),
        CLType::Any, // VestingStatus will be serialized into CLValue
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn team_status() -> EntryPoint {
    EntryPoint::new(
        String::from(TEAM_STATUS_ENTRY_POINT_NAME),
        Vec::new(),
        CLType::Any,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn staking_status() -> EntryPoint {
    EntryPoint::new(
        String::from(STAKING_STATUS_ENTRY_POINT_NAME),
        Vec::new(),
        CLType::Any,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn investor_status() -> EntryPoint {
    EntryPoint::new(
        String::from(INVESTOR_STATUS_ENTRY_POINT_NAME),
        Vec::new(),
        CLType::Any,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn network_status() -> EntryPoint {
    EntryPoint::new(
        String::from(NETWORK_STATUS_ENTRY_POINT_NAME),
        Vec::new(),
        CLType::Any,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn marketing_status() -> EntryPoint {
    EntryPoint::new(
        String::from(MARKETING_STATUS_ENTRY_POINT_NAME),
        Vec::new(),
        CLType::Any,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn airdrop_status() -> EntryPoint {
    EntryPoint::new(
        String::from(AIRDROP_STATUS_ENTRY_POINT_NAME),
        Vec::new(),
        CLType::Any,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the default set of CEP-18 token entry points.
pub fn generate_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(init());
    entry_points.add_entry_point(name());
    entry_points.add_entry_point(symbol());
    entry_points.add_entry_point(decimals());
    entry_points.add_entry_point(total_supply());
    entry_points.add_entry_point(balance_of());
    entry_points.add_entry_point(transfer());
    entry_points.add_entry_point(approve());
    entry_points.add_entry_point(allowance());
    entry_points.add_entry_point(decrease_allowance());
    entry_points.add_entry_point(increase_allowance());
    entry_points.add_entry_point(transfer_from());
    entry_points.add_entry_point(change_security());
    entry_points.add_entry_point(burn());
    entry_points.add_entry_point(mint());

    // Add vesting status entry points
    entry_points.add_entry_point(vesting_details());

    entry_points
}
