//! Contains definition of the entry points.
use crate::constants::{
    ADMIN_LIST, ARG_AMOUNT, ARG_CONTRACT_HASH, ARG_END_TIME, ARG_EVENTS_MODE, ARG_PURSE,
    ARG_RECIPIENT, ARG_START_TIME, ENTRY_POINT_BALANCE_COWL, ENTRY_POINT_BALANCE_CSPR,
    ENTRY_POINT_CHANGE_SECURITY, ENTRY_POINT_COWL_TO_CSPR, ENTRY_POINT_CSPR_TO_COWL,
    ENTRY_POINT_DEPOSIT_CSPR, ENTRY_POINT_INSTALL, ENTRY_POINT_SET_MODALITIES,
    ENTRY_POINT_UPDATE_TIMES, ENTRY_POINT_UPGRADE, ENTRY_POINT_WITHDRAW_COWL,
    ENTRY_POINT_WITHDRAW_CSPR, NONE_LIST,
};
use alloc::{boxed::Box, string::String, vec, vec::Vec};
use casper_types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter};

/// Returns the `init` entry point.
pub fn install() -> EntryPoint {
    EntryPoint::new(
        String::from(ENTRY_POINT_INSTALL),
        Vec::new(),
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn cspr_to_cowl() -> EntryPoint {
    EntryPoint::new(
        ENTRY_POINT_CSPR_TO_COWL,
        vec![
            Parameter::new(ARG_AMOUNT, CLType::U512),
            Parameter::new(ARG_RECIPIENT, CLType::Key),
            Parameter::new(ARG_PURSE, CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn cowl_to_cspr() -> EntryPoint {
    EntryPoint::new(
        ENTRY_POINT_COWL_TO_CSPR,
        vec![
            Parameter::new(ARG_AMOUNT, CLType::U512),
            Parameter::new(ARG_RECIPIENT, CLType::Key),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn update_times() -> EntryPoint {
    EntryPoint::new(
        ENTRY_POINT_UPDATE_TIMES,
        vec![
            Parameter::new(ARG_START_TIME, CLType::U64),
            Parameter::new(ARG_END_TIME, CLType::U64),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn withdraw_cspr() -> EntryPoint {
    EntryPoint::new(
        ENTRY_POINT_WITHDRAW_CSPR,
        vec![Parameter::new(ARG_AMOUNT, CLType::U512)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn withdraw_cowl() -> EntryPoint {
    EntryPoint::new(
        ENTRY_POINT_WITHDRAW_COWL,
        vec![Parameter::new(ARG_AMOUNT, CLType::U512)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn deposit_cspr() -> EntryPoint {
    EntryPoint::new(
        ENTRY_POINT_DEPOSIT_CSPR,
        vec![Parameter::new(ARG_AMOUNT, CLType::U512)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn balance_cowl() -> EntryPoint {
    EntryPoint::new(
        ENTRY_POINT_BALANCE_COWL,
        vec![],
        CLType::U512,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn balance_cspr() -> EntryPoint {
    EntryPoint::new(
        ENTRY_POINT_BALANCE_CSPR,
        vec![],
        CLType::U512,
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
    entry_points.add_entry_point(install());
    entry_points.add_entry_point(upgrade());
    entry_points.add_entry_point(set_modalities());
    entry_points.add_entry_point(change_security());

    entry_points.add_entry_point(update_times());

    entry_points.add_entry_point(withdraw_cowl());
    entry_points.add_entry_point(balance_cowl());
    entry_points.add_entry_point(cspr_to_cowl());

    entry_points.add_entry_point(withdraw_cspr());
    entry_points.add_entry_point(deposit_cspr());
    entry_points.add_entry_point(balance_cspr());
    entry_points.add_entry_point(cowl_to_cspr());

    entry_points
}
