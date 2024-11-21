#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");
extern crate alloc;

use crate::alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{collections::btree_map::BTreeMap, format, string::String};
use casper_contract::{
    contract_api::{
        runtime::{self, get_key, put_key, revert},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::NamedKeys, runtime_args, ContractHash, ContractPackageHash, Key, RuntimeArgs,
};
use vesting::{
    constants::{
        ADMIN_LIST, ARG_CONTRACT_HASH, ARG_EVENTS_MODE, ARG_NAME, ARG_PACKAGE_HASH,
        ARG_UPGRADE_FLAG, DICT_SECURITY_BADGES, ENTRY_POINT_INSTALL, ENTRY_POINT_UPGRADE,
        NONE_LIST, PREFIX_ACCESS_KEY_NAME, PREFIX_CONTRACT_NAME, PREFIX_CONTRACT_PACKAGE_NAME,
        PREFIX_CONTRACT_VERSION,
    },
    entry_points::generate_entry_points,
    error::VestingError,
    events::{init_events, record_event_dictionary, Event, Upgrade},
    security::{change_sec_badge, SecurityBadge},
    utils::{
        get_named_arg_with_user_errors, get_optional_named_arg_with_user_errors,
        get_verified_caller,
    },
};

#[no_mangle]
pub extern "C" fn install() {
    if get_key(ARG_PACKAGE_HASH).is_some() {
        revert(VestingError::ContractAlreadyInitialized);
    }

    put_key(
        ARG_PACKAGE_HASH,
        get_named_arg_with_user_errors::<Key>(
            ARG_PACKAGE_HASH,
            VestingError::MissingPackageHash,
            VestingError::InvalidPackageHash,
        )
        .unwrap_or_revert(),
    );

    put_key(
        ARG_CONTRACT_HASH,
        get_named_arg_with_user_errors::<Key>(
            ARG_CONTRACT_HASH,
            VestingError::MissingContractHash,
            VestingError::InvalidContractHash,
        )
        .unwrap_or_revert(),
    );

    init_events();

    storage::new_dictionary(DICT_SECURITY_BADGES).unwrap_or_revert();

    let mut badge_map: BTreeMap<Key, SecurityBadge> = BTreeMap::new();

    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, VestingError::InvalidAdminList);
    let none_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(NONE_LIST, VestingError::InvalidNoneList);

    if admin_list.is_none()
        || admin_list
            .as_ref()
            .unwrap_or_revert_with(VestingError::InvalidAdminList)
            .is_empty()
    {
        badge_map.insert(get_verified_caller().0, SecurityBadge::Admin);
    } else if let Some(admin_list) = admin_list {
        for account_key in admin_list {
            badge_map.insert(account_key, SecurityBadge::Admin);
        }
    }
    if let Some(none_list) = none_list {
        for account_key in none_list {
            badge_map.insert(account_key, SecurityBadge::None);
        }
    }

    change_sec_badge(&badge_map);
}

#[no_mangle]
pub extern "C" fn upgrade() {
    put_key(
        ARG_CONTRACT_HASH,
        get_named_arg_with_user_errors::<Key>(
            ARG_CONTRACT_HASH,
            VestingError::MissingContractHash,
            VestingError::InvalidContractHash,
        )
        .unwrap_or_revert(),
    );

    record_event_dictionary(Event::Upgrade(Upgrade {}))
}

fn upgrade_contract(name: &str) {
    let entry_points = generate_entry_points();

    let contract_package_hash = runtime::get_key(&format!("{PREFIX_CONTRACT_PACKAGE_NAME}_{name}"))
        .unwrap_or_revert()
        .into_hash()
        .map(ContractPackageHash::new)
        .unwrap_or_revert_with(VestingError::MissingPackageHashForUpgrade);

    let previous_contract_hash = runtime::get_key(&format!("{PREFIX_CONTRACT_NAME}_{name}"))
        .unwrap_or_revert()
        .into_hash()
        .map(ContractHash::new)
        .unwrap_or_revert_with(VestingError::MissingPackageHashForUpgrade);

    let (contract_hash, contract_version) =
        storage::add_contract_version(contract_package_hash, entry_points, NamedKeys::new());

    storage::disable_contract_version(contract_package_hash, previous_contract_hash)
        .unwrap_or_revert();
    runtime::put_key(
        &format!("{PREFIX_CONTRACT_NAME}_{name}"),
        contract_hash.into(),
    );
    runtime::put_key(
        &format!("{PREFIX_CONTRACT_VERSION}_{name}"),
        storage::new_uref(contract_version).into(),
    );

    let contract_hash_key = Key::from(contract_hash);

    let runtime_args = runtime_args! {
        ARG_CONTRACT_HASH => contract_hash_key,
    };

    runtime::call_contract::<()>(contract_hash, ENTRY_POINT_UPGRADE, runtime_args);
}

fn install_contract(name: &str) {
    let events_mode: u8 =
        get_optional_named_arg_with_user_errors(ARG_EVENTS_MODE, VestingError::InvalidEventsMode)
            .unwrap_or_default();

    let mut named_keys = NamedKeys::new();
    named_keys.insert(ARG_NAME.to_string(), storage::new_uref(name.clone()).into());
    named_keys.insert(
        ARG_EVENTS_MODE.to_string(),
        storage::new_uref(events_mode).into(),
    );

    let entry_points = generate_entry_points();

    let package_key_name = format!("{PREFIX_CONTRACT_PACKAGE_NAME}_{name}");

    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(package_key_name.clone()),
        Some(format!("{PREFIX_ACCESS_KEY_NAME}_{name}")),
    );

    let contract_hash_key = Key::from(contract_hash);

    runtime::put_key(&format!("{PREFIX_CONTRACT_NAME}_{name}"), contract_hash_key);
    runtime::put_key(
        &format!("{PREFIX_CONTRACT_VERSION}_{name}"),
        storage::new_uref(contract_version).into(),
    );

    let package_hash_key = runtime::get_key(&package_key_name).unwrap_or_revert();

    let init_args = runtime_args! {
        ARG_CONTRACT_HASH => contract_hash_key,
        ARG_PACKAGE_HASH => package_hash_key,
    };

    runtime::call_contract::<()>(contract_hash, ENTRY_POINT_INSTALL, init_args);
}

#[no_mangle]
pub extern "C" fn call() {
    let name: String =
        get_optional_named_arg_with_user_errors(ARG_NAME, VestingError::MissingCollectionName)
            .unwrap_or_revert_with(VestingError::InvalidCollectionName);

    let upgrade_flag: Option<bool> =
        get_optional_named_arg_with_user_errors(ARG_UPGRADE_FLAG, VestingError::InvalidUpgradeFlag);

    let access_key = runtime::get_key(&format!("{PREFIX_ACCESS_KEY_NAME}_{name}"));

    if upgrade_flag.is_some() && upgrade_flag.unwrap() && access_key.is_some() {
        upgrade_contract(&name)
    } else {
        install_contract(&name)
    }
}
