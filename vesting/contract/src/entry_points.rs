//! Contains definition of the entry points.
use crate::constants::{ENTRY_POINT_INSTALL, ENTRY_POINT_VESTING_DETAILS};
use alloc::{boxed::Box, string::String, vec, vec::Vec};
use casper_types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter};

/// Returns the `init` entry point.
pub fn init() -> EntryPoint {
    EntryPoint::new(
        String::from(ENTRY_POINT_INSTALL),
        Vec::new(),
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

// Single entry point for vesting details with proper boxing
pub fn vesting_details() -> EntryPoint {
    EntryPoint::new(
        String::from(ENTRY_POINT_VESTING_DETAILS),
        vec![Parameter::new("vesting_type", CLType::String)],
        CLType::Tuple3([
            Box::new(CLType::U256), // total_amount
            Box::new(CLType::U256), // vested_amount
            Box::new(CLType::Bool), // is_fully_vested
        ]),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn get_staking_status() -> EntryPoint {
    EntryPoint::new(
        String::from("get_staking_status"),
        Vec::new(),
        CLType::U256,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn debug_staking_status() -> EntryPoint {
    EntryPoint::new(
        String::from("debug_staking_status"),
        Vec::new(),
        CLType::Tuple3([
            Box::new(CLType::U256), // total_amount
            Box::new(CLType::U256), // start_time as U256
            Box::new(CLType::U256), // current_time as U256
        ]),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the default set of CEP-18 token entry points.
pub fn generate_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(init());
    entry_points.add_entry_point(vesting_details());
    entry_points.add_entry_point(get_staking_status());
    entry_points.add_entry_point(debug_staking_status());
    entry_points
}
