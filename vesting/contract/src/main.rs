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
        runtime::{
            call_contract, call_versioned_contract, get_blocktime, get_caller, get_key,
            get_named_arg, put_key, ret, revert,
        },
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::U256;
use casper_types::{
    bytesrepr::Bytes, contracts::NamedKeys, runtime_args, CLValue, ContractHash,
    ContractPackageHash, Key, RuntimeArgs,
};
use cowl_vesting::{
    constants::{
        ADMIN_LIST, ARG_AMOUNT, ARG_CONTRACT_HASH, ARG_COWL_CEP18_CONTRACT_PACKAGE, ARG_DATA,
        ARG_EVENTS_MODE, ARG_FROM, ARG_INSTALLER, ARG_NAME, ARG_OPERATOR, ARG_OWNER,
        ARG_PACKAGE_HASH, ARG_RECIPIENT, ARG_TO, ARG_TRANSFER_FILTER_CONTRACT_PACKAGE,
        ARG_TRANSFER_FILTER_METHOD, ARG_UPGRADE_FLAG, ARG_VESTING_TYPE,
        COWL_CEP_18_TOKEN_TOTAL_SUPPLY, DICT_ADDRESSES, DICT_SECURITY_BADGES, DICT_START_TIME,
        DICT_TRANSFERRED_AMOUNT, DICT_VESTING_AMOUNT, DICT_VESTING_INFO, DICT_VESTING_STATUS,
        ENTRY_POINT_CHANGE_SECURITY, ENTRY_POINT_CHECK_VESTING_TRANSFER, ENTRY_POINT_DECIMALS,
        ENTRY_POINT_INSTALL, ENTRY_POINT_MINT, ENTRY_POINT_SET_TRANSFER_FILTER,
        ENTRY_POINT_TOTAL_SUPPLY, ENTRY_POINT_TRANSFER, ENTRY_POINT_UPGRADE, MINTER_LIST,
        NONE_LIST, PREFIX_ACCESS_KEY_NAME, PREFIX_CONTRACT_NAME, PREFIX_CONTRACT_PACKAGE_NAME,
        PREFIX_CONTRACT_VERSION,
    },
    entry_points::generate_entry_points,
    enums::{EventsMode, TransferFilterContractResult, VestingType, VESTING_INFO},
    error::VestingError,
    events::{
        init_events, record_event_dictionary, ChangeSecurity, CheckTransfer,
        CowlCep18ContractPackageUpdate, Event, SetModalities, Upgrade,
    },
    security::{change_sec_badge, sec_check, SecurityBadge},
    utils::{
        get_cowl_cep18_contract_package_hash, get_named_arg_with_user_errors,
        get_optional_named_arg_with_user_errors, get_stored_value_with_user_errors,
        get_verified_caller, set_dictionary_value_for_key,
    },
    vesting::{
        calculate_vesting_allocations, get_current_balance_for_key, get_vesting_info,
        get_vesting_transfer, ret_vesting_info, ret_vesting_status, update_vesting_status,
    },
};

#[no_mangle]
pub extern "C" fn vesting_status() {
    let vesting_type: VestingType = get_named_arg_with_user_errors::<String>(
        ARG_VESTING_TYPE,
        VestingError::MissingVestingType,
        VestingError::InvalidVestingType,
    )
    .unwrap_or_revert()
    .as_str()
    .try_into()
    .unwrap_or_revert_with(VestingError::InvalidVestingType);
    ret_vesting_status(vesting_type);
}

#[no_mangle]
pub extern "C" fn vesting_info() {
    let vesting_type: VestingType = get_named_arg_with_user_errors::<String>(
        ARG_VESTING_TYPE,
        VestingError::MissingVestingType,
        VestingError::InvalidVestingType,
    )
    .unwrap_or_revert()
    .as_str()
    .try_into()
    .unwrap_or_revert_with(VestingError::InvalidVestingType);
    ret_vesting_info(vesting_type);
}

// Check that some values are sent by token contract and return a TransferFilterContractResult
#[no_mangle]
pub extern "C" fn check_vesting_transfer() {
    let operator: Key = get_named_arg(ARG_OPERATOR);
    let from: Key = get_named_arg(ARG_FROM);
    let to: Key = get_named_arg(ARG_TO);
    let amount: U256 = get_named_arg(ARG_AMOUNT);
    let data: Option<Bytes> = get_named_arg(ARG_DATA);

    let vesting_transfer = get_vesting_transfer(from, amount);

    record_event_dictionary(Event::CheckTransfer(CheckTransfer {
        operator,
        from,
        to,
        amount,
        data,
    }));

    if vesting_transfer {
        ret(CLValue::from_t(TransferFilterContractResult::ProceedTransfer).unwrap_or_revert());
    }

    ret(CLValue::from_t(TransferFilterContractResult::DenyTransfer).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn set_cowl_cep18_contract_package() {
    sec_check(vec![SecurityBadge::Admin]);

    let (caller, _) = get_verified_caller();

    let cowl_cep18_contract_package_key: Key = get_named_arg(ARG_COWL_CEP18_CONTRACT_PACKAGE);

    let cowl_cep18_contract_package_key_hash = ContractPackageHash::from(
        cowl_cep18_contract_package_key
            .into_hash()
            .unwrap_or_revert_with(VestingError::MissingTokenContractPackage),
    );

    put_key(
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

    if let Some(optional_events_mode) = get_optional_named_arg_with_user_errors::<u8>(
        ARG_EVENTS_MODE,
        VestingError::InvalidEventsMode,
    ) {
        let old_events_mode: EventsMode = get_stored_value_with_user_errors::<u8>(
            ARG_EVENTS_MODE,
            VestingError::MissingEventsMode,
            VestingError::InvalidEventsMode,
        )
        .try_into()
        .unwrap_or_revert();

        put_key(
            ARG_EVENTS_MODE,
            storage::new_uref(optional_events_mode).into(),
        );

        let new_events_mode: EventsMode = optional_events_mode
            .try_into()
            .unwrap_or_revert_with(VestingError::InvalidEventsMode);

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
        get_optional_named_arg_with_user_errors(ADMIN_LIST, VestingError::InvalidAdminList);
    let none_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(NONE_LIST, VestingError::InvalidNoneList);

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
        revert(VestingError::ContractAlreadyInitialized);
    }

    let vesting_contract_package_hash_key = get_named_arg_with_user_errors::<Key>(
        ARG_PACKAGE_HASH,
        VestingError::MissingPackageHash,
        VestingError::InvalidPackageHash,
    )
    .unwrap_or_revert();

    put_key(ARG_PACKAGE_HASH, vesting_contract_package_hash_key);

    let vesting_contract_hash_key = get_named_arg_with_user_errors::<Key>(
        ARG_CONTRACT_HASH,
        VestingError::MissingContractHash,
        VestingError::InvalidContractHash,
    )
    .unwrap_or_revert();

    put_key(ARG_CONTRACT_HASH, vesting_contract_hash_key);

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

    set_allocations(
        &vesting_contract_hash_key,
        &vesting_contract_package_hash_key,
    );

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
            VestingError::MissingContractHash,
            VestingError::InvalidContractHash,
        )
        .unwrap_or_revert(),
    );

    record_event_dictionary(Event::Upgrade(Upgrade {}))
}

fn install_contract(name: &str) {
    let events_mode: u8 =
        get_optional_named_arg_with_user_errors(ARG_EVENTS_MODE, VestingError::InvalidEventsMode)
            .unwrap_or_default();

    let cowl_cep18_contract_package_key: Key = get_named_arg(ARG_COWL_CEP18_CONTRACT_PACKAGE);

    let cowl_cep18_contract_package_hash = ContractPackageHash::from(
        cowl_cep18_contract_package_key
            .into_hash()
            .unwrap_or_revert_with(VestingError::InvalidTokenContractPackage),
    );

    let keys = vec![
        (ARG_NAME.to_string(), storage::new_uref(name).into()),
        (
            ARG_EVENTS_MODE.to_string(),
            storage::new_uref(events_mode).into(),
        ),
        (ARG_INSTALLER.to_string(), get_caller().into()),
        (
            ARG_COWL_CEP18_CONTRACT_PACKAGE.to_string(),
            storage::new_uref(cowl_cep18_contract_package_hash).into(),
        ),
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

    let address_liquidity: Key = get_named_arg(&VestingType::Liquidity.to_string());
    let address_contributor: Key = get_named_arg(&VestingType::Contributor.to_string());
    let address_development: Key = get_named_arg(&VestingType::Development.to_string());
    let address_treasury: Key = get_named_arg(&VestingType::Treasury.to_string());
    let address_community: Key = get_named_arg(&VestingType::Community.to_string());
    let address_staking: Key = get_named_arg(&VestingType::Staking.to_string());

    let mut init_args = runtime_args! {
        ARG_CONTRACT_HASH => contract_hash_key,
        ARG_PACKAGE_HASH => package_hash_key,
        &VestingType::Liquidity.to_string() => address_liquidity,
        &VestingType::Contributor.to_string() => address_contributor,
        &VestingType::Development.to_string() => address_development,
        &VestingType::Treasury.to_string() => address_treasury,
        &VestingType::Community.to_string() => address_community,
        &VestingType::Staking.to_string() => address_staking,
    };

    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, VestingError::InvalidAdminList);
    let none_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(NONE_LIST, VestingError::InvalidNoneList);

    if let Some(admin_list) = admin_list {
        init_args.insert(ADMIN_LIST, admin_list).unwrap_or_revert();
    }

    if let Some(none_list) = none_list {
        init_args.insert(NONE_LIST, none_list).unwrap_or_revert();
    }

    // Add vesting package to minter list
    call_versioned_contract::<()>(
        cowl_cep18_contract_package_hash,
        None,
        ENTRY_POINT_CHANGE_SECURITY,
        runtime_args! {
            MINTER_LIST => vec![package_hash_key],
        },
    );

    // Proceed to allocations to vesting addresses
    call_contract::<()>(contract_hash, ENTRY_POINT_INSTALL, init_args);

    // Check transfer filter package and method
    call_versioned_contract::<()>(
        cowl_cep18_contract_package_hash,
        None,
        ENTRY_POINT_SET_TRANSFER_FILTER,
        runtime_args! {
            ARG_TRANSFER_FILTER_CONTRACT_PACKAGE => Some(package_hash_key),
            ARG_TRANSFER_FILTER_METHOD => Some(ENTRY_POINT_CHECK_VESTING_TRANSFER),
        },
    );

    // Remove vesting package from minter list and add it to none list
    call_versioned_contract::<()>(
        cowl_cep18_contract_package_hash,
        None,
        ENTRY_POINT_CHANGE_SECURITY,
        runtime_args! {
            NONE_LIST => vec![package_hash_key],
        },
    );
}

fn upgrade_contract(name: &str) {
    let entry_points = generate_entry_points();

    let contract_package_hash = get_key(&format!("{PREFIX_CONTRACT_PACKAGE_NAME}_{name}"))
        .unwrap_or_revert()
        .into_hash()
        .map(ContractPackageHash::new)
        .unwrap_or_revert_with(VestingError::MissingPackageHashForUpgrade);

    let previous_contract_hash = get_key(&format!("{PREFIX_CONTRACT_NAME}_{name}"))
        .unwrap_or_revert()
        .into_hash()
        .map(ContractHash::new)
        .unwrap_or_revert_with(VestingError::MissingPackageHashForUpgrade);

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

pub fn set_allocations(vesting_contract_hash_key: &Key, vesting_contract_package_hash_key: &Key) {
    storage::new_dictionary(DICT_ADDRESSES).unwrap_or_revert();
    storage::new_dictionary(DICT_START_TIME).unwrap_or_revert();
    storage::new_dictionary(DICT_VESTING_AMOUNT).unwrap_or_revert();
    storage::new_dictionary(DICT_VESTING_INFO).unwrap_or_revert();
    storage::new_dictionary(DICT_VESTING_STATUS).unwrap_or_revert();
    storage::new_dictionary(DICT_TRANSFERRED_AMOUNT).unwrap_or_revert();

    for vesting_info in VESTING_INFO.iter() {
        set_dictionary_value_for_key(
            DICT_ADDRESSES,
            &vesting_info.vesting_type.to_string(),
            &get_named_arg::<Key>(&vesting_info.vesting_type.to_string()),
        );
    }

    let cowl_cep18_contract_package_hash = get_cowl_cep18_contract_package_hash();

    // Get token decimal
    let decimals = call_versioned_contract::<u8>(
        cowl_cep18_contract_package_hash,
        None,
        ENTRY_POINT_DECIMALS,
        runtime_args! {},
    );

    let total_supply = U256::from(COWL_CEP_18_TOKEN_TOTAL_SUPPLY * 10_u64.pow(decimals.into()));

    // Mint total supply
    call_versioned_contract::<()>(
        cowl_cep18_contract_package_hash,
        None,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_OWNER => vesting_contract_package_hash_key,
            ARG_AMOUNT =>total_supply
        },
    );

    let allocations = calculate_vesting_allocations(total_supply);

    // Write initial balances
    for allocation in allocations {
        if allocation.vesting_amount > U256::zero() {
            call_versioned_contract::<()>(
                cowl_cep18_contract_package_hash,
                None,
                ENTRY_POINT_TRANSFER,
                runtime_args! {
                    ARG_RECIPIENT => allocation.vesting_address_key,
                    ARG_AMOUNT => allocation.vesting_amount
                },
            );
        }

        let recipient_balance: U256 = get_current_balance_for_key(
            cowl_cep18_contract_package_hash,
            &allocation.vesting_address_key,
        );

        if recipient_balance != allocation.vesting_amount {
            revert(VestingError::InvalidRecepientAllocation);
        }

        set_dictionary_value_for_key(
            DICT_VESTING_AMOUNT,
            &allocation.vesting_type.to_string(),
            &recipient_balance,
        );

        let start_time_in_ms: u64 = get_blocktime().into();
        let start_time_in_seconds = start_time_in_ms.checked_div(1000).unwrap_or_default();

        set_dictionary_value_for_key(
            DICT_START_TIME,
            &allocation.vesting_type.to_string(),
            &start_time_in_seconds,
        );

        let _ = update_vesting_status(allocation.vesting_type);
    }

    for vesting_info in get_vesting_info().iter() {
        set_dictionary_value_for_key(
            DICT_VESTING_INFO,
            &vesting_info.vesting_type.to_string(),
            &vesting_info,
        );
    }

    let actual_supply: U256 = call_versioned_contract(
        cowl_cep18_contract_package_hash,
        None,
        ENTRY_POINT_TOTAL_SUPPLY,
        runtime_args! {},
    );

    if actual_supply != total_supply {
        revert(VestingError::InvalidInstallerTotalSupply);
    }

    let vesting_contract_balance: U256 =
        get_current_balance_for_key(cowl_cep18_contract_package_hash, vesting_contract_hash_key);

    // //! Vesting contract should not have remaining funds after installation
    if vesting_contract_balance != U256::zero() {
        revert(VestingError::InvalidInstallerTotalSupply);
    }
}

#[no_mangle]
pub extern "C" fn call() {
    let name: String = get_named_arg_with_user_errors(
        ARG_NAME,
        VestingError::MissingVestingName,
        VestingError::InvalidVestingName,
    )
    .unwrap_or_revert();

    let upgrade_flag: Option<bool> =
        get_optional_named_arg_with_user_errors(ARG_UPGRADE_FLAG, VestingError::InvalidUpgradeFlag);

    let access_key = get_key(&format!("{PREFIX_ACCESS_KEY_NAME}_{name}"));

    if upgrade_flag.is_some() && upgrade_flag.unwrap() && access_key.is_some() {
        upgrade_contract(&name)
    } else if access_key.is_none() {
        install_contract(&name)
    }
}
