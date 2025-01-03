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
            call_contract, call_versioned_contract, get_caller, get_key, get_named_arg, put_key,
            ret, revert,
        },
        storage::{
            add_contract_version, disable_contract_version, new_contract, new_dictionary, new_uref,
        },
        system::{
            create_purse, get_purse_balance, transfer_from_purse_to_account,
            transfer_from_purse_to_purse,
        },
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::NamedKeys, runtime_args, system::handle_payment::ARG_PURSE, CLValue, ContractHash,
    ContractPackageHash, Key, RuntimeArgs, URef, U256, U512,
};
use cowl_swap::{
    constants::{
        ADMIN_LIST, ARG_AMOUNT, ARG_BALANCE_COWL, ARG_BALANCE_CSPR, ARG_CONTRACT_HASH,
        ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH, ARG_END_TIME, ARG_EVENTS_MODE, ARG_INSTALLER,
        ARG_NAME, ARG_OWNER, ARG_PACKAGE_HASH, ARG_RECIPIENT, ARG_START_TIME, ARG_UPGRADE_FLAG,
        DICT_SECURITY_BADGES, ENTRY_POINT_INSTALL, ENTRY_POINT_TRANSFER, ENTRY_POINT_TRANSFER_FROM,
        ENTRY_POINT_UPGRADE, NONE_LIST, PREFIX_ACCESS_KEY_NAME, PREFIX_CONTRACT_NAME,
        PREFIX_CONTRACT_PACKAGE_NAME, PREFIX_CONTRACT_VERSION, RATE_TIERS, TAX_RATE,
    },
    entry_points::generate_entry_points,
    enums::EventsMode,
    error::SwapError,
    events::{
        init_events, record_event_dictionary, ChangeSecurity, CowlCep18ContractPackageUpdate,
        Event, SetModalities, Upgrade,
    },
    rate::{get_swap_rate, validate_amount, validate_rate, verify_swap_active},
    security::{change_sec_badge, sec_check, SecurityBadge},
    utils::{
        get_cowl_cep18_balance_for_key, get_cowl_cep18_contract_package_hash,
        get_named_arg_with_user_errors, get_optional_named_arg_with_user_errors,
        get_stored_value_with_user_errors, get_verified_caller,
    },
};

#[no_mangle]
pub extern "C" fn balance_cowl() {
    let owner = get_key(ARG_PACKAGE_HASH).unwrap_or_revert();
    let balance = get_cowl_cep18_balance_for_key(&owner);

    put_key(ARG_BALANCE_COWL, new_uref(balance).into());
    ret(CLValue::from_t(balance).unwrap_or_revert())
}

#[no_mangle]
pub extern "C" fn balance_cspr() {
    let contract_purse = get_key(ARG_PURSE).unwrap_or_revert_with(SwapError::MissingPurse);
    let balance = get_purse_balance(contract_purse.as_uref().unwrap_or_revert().into_read())
        .unwrap_or_revert();
    put_key(ARG_BALANCE_CSPR, new_uref(balance).into());
    ret(CLValue::from_t(balance).unwrap_or_revert())
}

#[no_mangle]
pub extern "C" fn cspr_to_cowl() {
    verify_swap_active().unwrap_or_revert();
    let amount: U512 = get_named_arg(ARG_AMOUNT);
    let rate = get_swap_rate(amount).unwrap_or_revert();
    validate_rate(rate).unwrap_or_revert();
    let cowl_amount = amount * rate;
    validate_amount(cowl_amount).unwrap_or_revert();
    let source_purse: URef = get_named_arg(ARG_PURSE);
    let contract_purse = get_key(ARG_PURSE).unwrap_or_revert_with(SwapError::MissingPurse);
    transfer_from_purse_to_purse(
        source_purse,
        *contract_purse
            .as_uref()
            .unwrap_or_revert_with(SwapError::MissingPurse),
        amount,
        None,
    )
    .unwrap_or_revert();

    let (caller, _) = get_verified_caller();

    call_versioned_contract::<()>(
        get_cowl_cep18_contract_package_hash(),
        None,
        ENTRY_POINT_TRANSFER,
        runtime_args! {
            ARG_RECIPIENT => caller,
            ARG_AMOUNT => cowl_amount
        },
    )
}

#[no_mangle]
pub extern "C" fn cowl_to_cspr() {
    verify_swap_active().unwrap_or_revert();
    let amount_u256: U256 = get_named_arg(ARG_AMOUNT);
    let amount_u512: U512 = U512::from_dec_str(&amount_u256.to_string())
        .unwrap_or_else(|_| revert(SwapError::InvalidAmount));

    validate_amount(amount_u512).unwrap_or_revert();

    let base_rate = RATE_TIERS.first().unwrap_or_revert().rate;
    validate_rate(base_rate).unwrap_or_revert();

    let cspr_amount: U512 = amount_u512 / base_rate;
    validate_amount(cspr_amount).unwrap_or_revert();

    let tax_amount = cspr_amount * TAX_RATE / U512::from(100);
    let cspr_amount: U512 = cspr_amount - tax_amount;

    let contract_purse = get_key(ARG_PURSE).unwrap_or_revert_with(SwapError::MissingPurse);

    let (caller, _) = get_verified_caller();

    let recipient = get_key(ARG_PACKAGE_HASH).unwrap_or_revert();

    call_versioned_contract::<()>(
        get_cowl_cep18_contract_package_hash(),
        None,
        ENTRY_POINT_TRANSFER_FROM,
        runtime_args! {
            ARG_OWNER => caller,
            ARG_RECIPIENT => recipient,
            ARG_AMOUNT => amount_u256
        },
    );

    transfer_from_purse_to_account(
        *contract_purse
            .as_uref()
            .unwrap_or_revert_with(SwapError::MissingPurse),
        caller
            .into_account()
            .unwrap_or_revert_with(SwapError::InvalidKey),
        cspr_amount,
        None,
    )
    .unwrap_or_revert_with(SwapError::InvalidPurseTransfer);
}

#[no_mangle]
pub extern "C" fn withdraw_cspr() {
    sec_check(vec![SecurityBadge::Admin]);

    let amount: U512 = get_named_arg(ARG_AMOUNT);
    validate_amount(amount).unwrap_or_revert();

    let (caller, _) = get_verified_caller();
    let contract_purse = get_key(ARG_PURSE).unwrap_or_revert_with(SwapError::MissingPurse);

    transfer_from_purse_to_account(
        *contract_purse
            .as_uref()
            .unwrap_or_revert_with(SwapError::MissingPurse),
        caller
            .into_account()
            .unwrap_or_revert_with(SwapError::InvalidKey),
        amount,
        None,
    )
    .unwrap_or_revert_with(SwapError::InvalidPurseTransfer);
}

#[no_mangle]
pub extern "C" fn withdraw_cowl() {
    sec_check(vec![SecurityBadge::Admin]);

    let amount: U256 = get_named_arg(ARG_AMOUNT);
    if amount == U256::zero() {
        revert(SwapError::InvalidAmount);
    }

    let (caller, _) = get_verified_caller();

    call_versioned_contract::<()>(
        get_cowl_cep18_contract_package_hash(),
        None,
        ENTRY_POINT_TRANSFER,
        runtime_args! {
            ARG_RECIPIENT => caller,
            ARG_AMOUNT => amount
        },
    )
}

#[no_mangle]
pub extern "C" fn deposit_cspr() {
    sec_check(vec![SecurityBadge::Admin]);

    let amount: U512 = get_named_arg(ARG_AMOUNT);
    if amount.is_zero() {
        revert(SwapError::InvalidAmount);
    }

    let source_purse: URef = get_named_arg(ARG_PURSE);
    let contract_purse = get_key(ARG_PURSE).unwrap_or_revert_with(SwapError::MissingPurse);

    transfer_from_purse_to_purse(
        source_purse,
        *contract_purse
            .as_uref()
            .unwrap_or_revert_with(SwapError::MissingPurse),
        amount,
        None,
    )
    .unwrap_or_revert_with(SwapError::InvalidPurseTransfer);
}

#[no_mangle]
pub extern "C" fn update_times() {
    sec_check(vec![SecurityBadge::Admin]);

    let new_start_time: u64 = get_named_arg(ARG_START_TIME);
    let new_end_time: u64 = get_named_arg(ARG_END_TIME);

    if new_end_time <= new_start_time {
        revert(SwapError::InvalidTimeWindow)
    }
    put_key(ARG_START_TIME, new_uref(new_start_time).into());
    put_key(ARG_END_TIME, new_uref(new_end_time).into());
}

#[no_mangle]
pub extern "C" fn set_cowl_cep18_contract_package() {
    sec_check(vec![SecurityBadge::Admin]);

    let (caller, _) = get_verified_caller();

    let cowl_cep18_contract_package_key: Key = get_named_arg(ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH);

    let cowl_cep18_contract_package_key_hash = ContractPackageHash::from(
        cowl_cep18_contract_package_key
            .into_hash()
            .unwrap_or_revert_with(SwapError::MissingTokenContractPackage),
    );

    put_key(
        ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH,
        new_uref(cowl_cep18_contract_package_key_hash).into(),
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

        put_key(ARG_EVENTS_MODE, new_uref(optional_events_mode).into());

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

    new_dictionary(DICT_SECURITY_BADGES).unwrap_or_revert();

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

    let contract_purse = create_purse();
    put_key(ARG_PURSE, contract_purse.into());
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

    record_event_dictionary(Event::Upgrade(Upgrade {}));
}

fn install_contract(name: &str) {
    let events_mode: u8 =
        get_optional_named_arg_with_user_errors(ARG_EVENTS_MODE, SwapError::InvalidEventsMode)
            .unwrap_or_default();

    let cowl_cep18_contract_package_key: Key = get_named_arg(ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH);

    let cowl_cep18_contract_package_hash = ContractPackageHash::from(
        cowl_cep18_contract_package_key
            .into_hash()
            .unwrap_or_revert_with(SwapError::InvalidTokenContractPackage),
    );

    let start_time: u64 = get_named_arg(ARG_START_TIME);
    let end_time: u64 = get_named_arg(ARG_END_TIME);

    let keys = vec![
        (ARG_NAME.to_string(), new_uref(name).into()),
        (ARG_EVENTS_MODE.to_string(), new_uref(events_mode).into()),
        (ARG_INSTALLER.to_string(), get_caller().into()),
        (ARG_START_TIME.to_string(), new_uref(start_time).into()),
        (ARG_END_TIME.to_string(), new_uref(end_time).into()),
        (
            ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH.to_string(),
            new_uref(cowl_cep18_contract_package_hash).into(),
        ),
    ];

    let mut named_keys = NamedKeys::new();
    for (key, value) in keys {
        named_keys.insert(key, value);
    }

    let entry_points = generate_entry_points();

    let package_key_name = format!("{PREFIX_CONTRACT_PACKAGE_NAME}_{name}");

    let (contract_hash, contract_version) = new_contract(
        entry_points,
        Some(named_keys),
        Some(package_key_name.clone()),
        Some(format!("{PREFIX_ACCESS_KEY_NAME}_{name}")),
    );

    let contract_hash_key = Key::from(contract_hash);

    put_key(&format!("{PREFIX_CONTRACT_NAME}_{name}"), contract_hash_key);
    put_key(
        &format!("{PREFIX_CONTRACT_VERSION}_{name}"),
        new_uref(contract_version).into(),
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
        add_contract_version(contract_package_hash, entry_points, NamedKeys::new());

    disable_contract_version(contract_package_hash, previous_contract_hash).unwrap_or_revert();
    put_key(
        &format!("{PREFIX_CONTRACT_NAME}_{name}"),
        contract_hash.into(),
    );
    put_key(
        &format!("{PREFIX_CONTRACT_VERSION}_{name}"),
        new_uref(contract_version).into(),
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
