//! Contains definition of the entry points.
use crate::{
    constants::{
        ADMIN_LIST, ARG_AMOUNT, ARG_CONTRACT_HASH, ARG_COWL_CEP18_CONTRACT_PACKAGE, ARG_DATA,
        ARG_EVENTS_MODE, ARG_FROM, ARG_OPERATOR, ARG_TO, ARG_VESTING_TYPE,
        ENTRY_POINT_CHANGE_SECURITY, ENTRY_POINT_CHECK_VESTING_TRANSFER,
        ENTRY_POINT_COWL_CEP18_CONTRACT_PACKAGE, ENTRY_POINT_INSTALL, ENTRY_POINT_SET_MODALITIES,
        ENTRY_POINT_UPGRADE, ENTRY_POINT_VESTING_INFO, ENTRY_POINT_VESTING_STATUS, NONE_LIST,
    },
    enums::TransferFilterContractResult,
};
use alloc::{boxed::Box, string::String, vec, vec::Vec};
use casper_types::{
    bytesrepr::Bytes, CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints,
    Parameter,
};

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

pub fn vesting_status() -> EntryPoint {
    EntryPoint::new(
        String::from(ENTRY_POINT_VESTING_STATUS),
        vec![Parameter::new(ARG_VESTING_TYPE, CLType::String)],
        Bytes::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn vesting_info() -> EntryPoint {
    EntryPoint::new(
        String::from(ENTRY_POINT_VESTING_INFO),
        vec![Parameter::new(ARG_VESTING_TYPE, CLType::String)],
        Bytes::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn check_vesting_transfer() -> EntryPoint {
    EntryPoint::new(
        String::from(ENTRY_POINT_CHECK_VESTING_TRANSFER),
        vec![
            Parameter::new(ARG_OPERATOR, CLType::Key),
            Parameter::new(ARG_FROM, CLType::Key),
            Parameter::new(ARG_TO, CLType::Key),
            Parameter::new(ARG_AMOUNT, CLType::U256),
            Parameter::new(ARG_DATA, CLType::Option(Box::new(Bytes::cl_type()))),
        ],
        TransferFilterContractResult::cl_type(),
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

pub fn set_cowl_cep18_contract_package() -> EntryPoint {
    EntryPoint::new(
        String::from(ENTRY_POINT_COWL_CEP18_CONTRACT_PACKAGE),
        vec![Parameter::new(
            ARG_COWL_CEP18_CONTRACT_PACKAGE,
            CLType::Option(Box::new(CLType::Key)),
        )],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn generate_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(init());
    entry_points.add_entry_point(upgrade());
    entry_points.add_entry_point(vesting_status());
    entry_points.add_entry_point(vesting_info());
    entry_points.add_entry_point(check_vesting_transfer());
    entry_points.add_entry_point(set_modalities());
    entry_points.add_entry_point(change_security());
    entry_points.add_entry_point(set_cowl_cep18_contract_package());

    entry_points
}
