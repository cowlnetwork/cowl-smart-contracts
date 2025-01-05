use casper_types::{runtime_args, Key, RuntimeArgs};
use cowl_vesting::{
    constants::ADMIN_LIST, enums::EventsMode, error::VestingError, events::ChangeSecurity,
    security::SecurityBadge,
};

use std::collections::{BTreeMap, HashMap};

use crate::{
    support::get_event,
    utility::{
        constants::{ACCOUNT_USER_1, ACCOUNT_USER_2},
        installer_request_builders::{
            cowl_vesting_change_security, cowl_vesting_set_modalities, setup, setup_with_args,
            SecurityLists, TestContext,
        },
        support::{assert_expected_error, create_dummy_key_pair, fund_account},
    },
};

#[test]
fn should_test_security_no_rights() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let owner = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let set_modalities_call = cowl_vesting_set_modalities(
        &mut builder,
        &cowl_vesting_contract_hash,
        &owner,
        Some(EventsMode::NoEvents),
    );
    set_modalities_call.expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(
        error,
        VestingError::InsufficientRights as u16,
        "should not allow to set_modalities for non default admin account",
    );
}

#[test]
fn should_test_security_rights() {
    let (_, public_key_account_user_1) = create_dummy_key_pair(ACCOUNT_USER_1);
    let account_user_1 = public_key_account_user_1.to_account_hash();
    let mut test_accounts = HashMap::new();
    test_accounts.insert(ACCOUNT_USER_1, account_user_1);

    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup_with_args(
        runtime_args! {
            ADMIN_LIST => vec![Key::from(account_user_1)]
        },
        Some(test_accounts),
    );

    // account_user_1 was created before genesis and is not yet funded so fund it
    fund_account(&mut builder, account_user_1);

    let owner = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let set_modalities_call = cowl_vesting_set_modalities(
        &mut builder,
        &cowl_vesting_contract_hash,
        &owner,
        Some(EventsMode::NoEvents),
    );
    set_modalities_call.expect_success().commit();

    // account_user_2 is not in admin list, request should fail
    let owner = *test_accounts.get(&ACCOUNT_USER_2).unwrap();
    let set_modalities_call = cowl_vesting_set_modalities(
        &mut builder,
        &cowl_vesting_contract_hash,
        &owner,
        Some(EventsMode::NoEvents),
    );

    set_modalities_call.expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(
        error,
        VestingError::InsufficientRights as u16,
        "should not allow to set_modalities for non default admin account",
    );
}

#[test]
fn should_test_change_security() {
    let (_, public_key_account_user_1) = create_dummy_key_pair(ACCOUNT_USER_1);
    let account_user_1 = public_key_account_user_1.to_account_hash();
    let mut test_accounts = HashMap::new();
    test_accounts.insert(ACCOUNT_USER_1, account_user_1);

    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup_with_args(
        runtime_args! {
            ADMIN_LIST => vec![Key::from(account_user_1)],
        },
        None,
    );

    let account_user_2 = *test_accounts.get(&ACCOUNT_USER_2).unwrap();

    let security_lists = SecurityLists {
        admin_list: Some(vec![Key::Account(account_user_2)]),
        none_list: None,
    };

    let change_security = cowl_vesting_change_security(
        &mut builder,
        &cowl_vesting_contract_hash,
        &account_user_1,
        security_lists,
    );

    change_security.expect_success().commit();

    let expected_event = ChangeSecurity {
        admin: Key::Account(account_user_1),
        sec_change_map: {
            let mut map = BTreeMap::new();
            map.insert(Key::Account(account_user_2), SecurityBadge::Admin);
            map
        },
    };
    let actual_event: ChangeSecurity = get_event(&builder, &cowl_vesting_contract_hash.into(), 0);

    assert_eq!(
        actual_event, expected_event,
        "Expected ChangeSecurity event."
    );

    let owner = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let set_modalities_call = cowl_vesting_set_modalities(
        &mut builder,
        &cowl_vesting_contract_hash,
        &owner,
        Some(EventsMode::NoEvents),
    );
    set_modalities_call.expect_success().commit();

    let owner = *test_accounts.get(&ACCOUNT_USER_2).unwrap();
    let set_modalities_call = cowl_vesting_set_modalities(
        &mut builder,
        &cowl_vesting_contract_hash,
        &owner,
        Some(EventsMode::CES),
    );
    set_modalities_call.expect_success().commit();

    let security_lists = SecurityLists {
        admin_list: None,
        none_list: Some(vec![Key::Account(account_user_2)]),
    };

    let change_security = cowl_vesting_change_security(
        &mut builder,
        &cowl_vesting_contract_hash,
        &account_user_1,
        security_lists,
    );

    change_security.expect_success().commit();

    let owner = *test_accounts.get(&ACCOUNT_USER_2).unwrap();
    let set_modalities_call = cowl_vesting_set_modalities(
        &mut builder,
        &cowl_vesting_contract_hash,
        &owner,
        Some(EventsMode::CES),
    );
    set_modalities_call.expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(
        error,
        VestingError::InsufficientRights as u16,
        "should not allow to mint for non default admin account",
    );
}
