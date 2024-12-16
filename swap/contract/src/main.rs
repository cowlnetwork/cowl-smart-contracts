#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");
extern crate alloc;

use alloc::{
    collections::btree_map::BTreeMap, format, string::String, string::ToString, vec, vec::Vec,
};
use casper_contract::{
    contract_api::{
        runtime::{self, call_contract, get_caller, get_key, get_named_arg, put_key, revert},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::NamedKeys, runtime_args, ContractHash, ContractPackageHash, Key, RuntimeArgs,
};
use cowl_swap::{
    constants::{
        ADMIN_LIST, ARG_CONTRACT_HASH, ARG_COWL_CEP18_CONTRACT_PACKAGE, ARG_EVENTS_MODE,
        ARG_INSTALLER, ARG_NAME, ARG_PACKAGE_HASH, ARG_UPGRADE_FLAG, DICT_SECURITY_BADGES,
        ENTRY_POINT_INSTALL, ENTRY_POINT_UPGRADE, NONE_LIST, PREFIX_ACCESS_KEY_NAME,
        PREFIX_CONTRACT_NAME, PREFIX_CONTRACT_PACKAGE_NAME, PREFIX_CONTRACT_VERSION,
    },
    entry_points::generate_entry_points,
    enums::EventsMode,
    error::SwapError,
    events::{
        init_events, record_event_dictionary, ChangeSecurity, CowlCep18ContractPackageUpdate,
        Event, SetModalities, Upgrade,
    },
    security::{change_sec_badge, sec_check, SecurityBadge},
    utils::{
        get_named_arg_with_user_errors, get_optional_named_arg_with_user_errors,
        get_stored_value_with_user_errors, get_verified_caller,
    },
};

#[no_mangle]
pub extern "C" fn set_cowl_cep18_contract_package() {
    sec_check(vec![SecurityBadge::Admin]);

    let (caller, _) = get_verified_caller();

    let cowl_cep18_contract_package_key: Key = get_named_arg(ARG_COWL_CEP18_CONTRACT_PACKAGE);

    let cowl_cep18_contract_package_key_hash = ContractPackageHash::from(
        cowl_cep18_contract_package_key
            .into_hash()
            .unwrap_or_revert_with(SwapError::MissingTokenContractPackage),
    );

    runtime::put_key(
        ARG_COWL_CEP18_CONTRACT_PACKAGE,
        storage::new_uref(cowl_cep18_contract_package_key_hash).into(),
    );

    record_event_dictionary(Event::CowlCep18ContractPackageUpdate(
        CowlCep18ContractPackageUpdate {
            key: caller,
            cowl_cep18_contract_package_key,
        },
    ));
}

#[no_mangle]
pub extern "C" fn set_modalities() {
    // Only the installing account can change the mutable variables.
    sec_check(vec![SecurityBadge::Admin]);

    if let Some(optional_events_mode) =
        get_optional_named_arg_with_user_errors::<u8>(ARG_EVENTS_MODE, SwapError::InvalidEventsMode)
    {
        let old_events_mode: EventsMode = get_stored_value_with_user_errors::<u8>(
            ARG_EVENTS_MODE,
            SwapError::MissingEventsMode,
            SwapError::InvalidEventsMode,
        )
        .try_into()
        .unwrap_or_revert();

        put_key(
            ARG_EVENTS_MODE,
            storage::new_uref(optional_events_mode).into(),
        );

        let new_events_mode: EventsMode = optional_events_mode
            .try_into()
            .unwrap_or_revert_with(SwapError::InvalidEventsMode);

        // Check if current_events_mode and requested_events_mode are both CES
        if old_events_mode != EventsMode::CES && new_events_mode == EventsMode::CES {
            // Initialize events structures
            init_events();
        }
    }

    record_event_dictionary(Event::SetModalities(SetModalities {}));
}

/// Beware: do not remove the last Admin because that will lock out all admin functionality.
#[no_mangle]
pub extern "C" fn change_security() {
    sec_check(vec![SecurityBadge::Admin]);

    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, SwapError::InvalidAdminList);
    let none_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(NONE_LIST, SwapError::InvalidNoneList);

    let mut badge_map: BTreeMap<Key, SecurityBadge> = BTreeMap::new();

    if let Some(admin_list) = admin_list {
        for account_key in admin_list {
            badge_map.insert(account_key, SecurityBadge::Admin);
        }
    }
    if let Some(none_list) = none_list {
        for account_key in none_list {
            badge_map.insert(account_key, SecurityBadge::None);
        }
    }

    let (caller, _) = get_verified_caller();
    badge_map.remove(&caller);

    change_sec_badge(&badge_map);
    record_event_dictionary(Event::ChangeSecurity(ChangeSecurity {
        admin: caller,
        sec_change_map: badge_map,
    }));
}

#[no_mangle]
pub extern "C" fn install() {
    if get_key(ARG_PACKAGE_HASH).is_some() {
        revert(SwapError::ContractAlreadyInitialized);
    }

    let swap_contract_package_hash_key = get_named_arg_with_user_errors::<Key>(
        ARG_PACKAGE_HASH,
        SwapError::MissingPackageHash,
        SwapError::InvalidPackageHash,
    )
    .unwrap_or_revert();

    put_key(ARG_PACKAGE_HASH, swap_contract_package_hash_key);

    let swap_contract_hash_key = get_named_arg_with_user_errors::<Key>(
        ARG_CONTRACT_HASH,
        SwapError::MissingContractHash,
        SwapError::InvalidContractHash,
    )
    .unwrap_or_revert();

    put_key(ARG_CONTRACT_HASH, swap_contract_hash_key);

    init_events();

    storage::new_dictionary(DICT_SECURITY_BADGES).unwrap_or_revert();

    let mut badge_map: BTreeMap<Key, SecurityBadge> = BTreeMap::new();

    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, SwapError::InvalidAdminList);
    let none_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(NONE_LIST, SwapError::InvalidNoneList);

    if admin_list.is_none()
        || admin_list
            .as_ref()
            .unwrap_or_revert_with(SwapError::InvalidAdminList)
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
    // Only the admin can upgrade
    sec_check(vec![SecurityBadge::Admin]);

    put_key(
        ARG_CONTRACT_HASH,
        get_named_arg_with_user_errors::<Key>(
            ARG_CONTRACT_HASH,
            SwapError::MissingContractHash,
            SwapError::InvalidContractHash,
        )
        .unwrap_or_revert(),
    );

    record_event_dictionary(Event::Upgrade(Upgrade {}))
}

fn install_contract(name: &str) {
    let events_mode: u8 =
        get_optional_named_arg_with_user_errors(ARG_EVENTS_MODE, SwapError::InvalidEventsMode)
            .unwrap_or_default();

    // let cowl_cep18_contract_package_key: Key = get_named_arg(ARG_COWL_CEP18_CONTRACT_PACKAGE);

    // let cowl_cep18_contract_package_hash = ContractPackageHash::from(
    //     cowl_cep18_contract_package_key
    //         .into_hash()
    //         .unwrap_or_revert_with(SwapError::InvalidTokenContractPackage),
    // );

    let keys = vec![
        (ARG_NAME.to_string(), storage::new_uref(name).into()),
        (
            ARG_EVENTS_MODE.to_string(),
            storage::new_uref(events_mode).into(),
        ),
        (ARG_INSTALLER.to_string(), get_caller().into()),
        // (
        //     ARG_COWL_CEP18_CONTRACT_PACKAGE.to_string(),
        //     storage::new_uref(cowl_cep18_contract_package_hash).into(),
        // ),
    ];

    let mut named_keys = NamedKeys::new();
    for (key, value) in keys {
        named_keys.insert(key, value);
    }

    let entry_points = generate_entry_points();

    let package_key_name = format!("{PREFIX_CONTRACT_PACKAGE_NAME}_{name}");

    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(package_key_name.clone()),
        Some(format!("{PREFIX_ACCESS_KEY_NAME}_{name}")),
    );

    let contract_hash_key = Key::from(contract_hash);

    put_key(&format!("{PREFIX_CONTRACT_NAME}_{name}"), contract_hash_key);
    put_key(
        &format!("{PREFIX_CONTRACT_VERSION}_{name}"),
        storage::new_uref(contract_version).into(),
    );

    let package_hash_key = get_key(&package_key_name).unwrap_or_revert();

    let mut init_args = runtime_args! {
        ARG_CONTRACT_HASH => contract_hash_key,
        ARG_PACKAGE_HASH => package_hash_key,
    };

    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, SwapError::InvalidAdminList);
    let none_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(NONE_LIST, SwapError::InvalidNoneList);

    if let Some(admin_list) = admin_list {
        init_args.insert(ADMIN_LIST, admin_list).unwrap_or_revert();
    }

    if let Some(none_list) = none_list {
        init_args.insert(NONE_LIST, none_list).unwrap_or_revert();
    }

    call_contract::<()>(contract_hash, ENTRY_POINT_INSTALL, init_args);
}

fn upgrade_contract(name: &str) {
    let entry_points = generate_entry_points();

    let contract_package_hash = get_key(&format!("{PREFIX_CONTRACT_PACKAGE_NAME}_{name}"))
        .unwrap_or_revert()
        .into_hash()
        .map(ContractPackageHash::new)
        .unwrap_or_revert_with(SwapError::MissingPackageHashForUpgrade);

    let previous_contract_hash = get_key(&format!("{PREFIX_CONTRACT_NAME}_{name}"))
        .unwrap_or_revert()
        .into_hash()
        .map(ContractHash::new)
        .unwrap_or_revert_with(SwapError::MissingPackageHashForUpgrade);

    let (contract_hash, contract_version) =
        storage::add_contract_version(contract_package_hash, entry_points, NamedKeys::new());

    storage::disable_contract_version(contract_package_hash, previous_contract_hash)
        .unwrap_or_revert();
    put_key(
        &format!("{PREFIX_CONTRACT_NAME}_{name}"),
        contract_hash.into(),
    );
    put_key(
        &format!("{PREFIX_CONTRACT_VERSION}_{name}"),
        storage::new_uref(contract_version).into(),
    );

    let contract_hash_key = Key::from(contract_hash);

    let runtime_args = runtime_args! {
        ARG_CONTRACT_HASH => contract_hash_key,
    };

    call_contract::<()>(contract_hash, ENTRY_POINT_UPGRADE, runtime_args);
}

#[no_mangle]
pub extern "C" fn call() {
    let name: String = get_named_arg_with_user_errors(
        ARG_NAME,
        SwapError::MissingSwapName,
        SwapError::InvalidSwapName,
    )
    .unwrap_or_revert();

    let upgrade_flag: Option<bool> =
        get_optional_named_arg_with_user_errors(ARG_UPGRADE_FLAG, SwapError::InvalidUpgradeFlag);

    let access_key = get_key(&format!("{PREFIX_ACCESS_KEY_NAME}_{name}"));

    if upgrade_flag.is_some() && upgrade_flag.unwrap() && access_key.is_some() {
        upgrade_contract(&name)
    } else if access_key.is_none() {
        install_contract(&name)
    }
}
