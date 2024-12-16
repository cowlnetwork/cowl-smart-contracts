//! Contains definition of the entry points.
use crate::constants::{
    ADMIN_LIST, ARG_CONTRACT_HASH, ARG_EVENTS_MODE, ENTRY_POINT_CHANGE_SECURITY,
    ENTRY_POINT_INSTALL, ENTRY_POINT_SET_MODALITIES, ENTRY_POINT_UPGRADE, NONE_LIST,
};
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

pub fn upgrade() -> EntryPoint {
    EntryPoint::new(
        ENTRY_POINT_UPGRADE,
        vec![Parameter::new(ARG_CONTRACT_HASH, CLType::Key)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn set_modalities() -> EntryPoint {
    EntryPoint::new(
        ENTRY_POINT_SET_MODALITIES,
        vec![Parameter::new(ARG_EVENTS_MODE, CLType::U8)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn change_security() -> EntryPoint {
    EntryPoint::new(
        ENTRY_POINT_CHANGE_SECURITY,
        vec![
            Parameter::new(ADMIN_LIST, CLType::List(Box::new(CLType::Key))),
            Parameter::new(NONE_LIST, CLType::List(Box::new(CLType::Key))),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn generate_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(init());
    entry_points.add_entry_point(upgrade());
    entry_points.add_entry_point(set_modalities());
    entry_points.add_entry_point(change_security());

    entry_points
}
