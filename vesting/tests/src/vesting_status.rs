use std::time::Duration;

use crate::utility::{
    installer_request_builders::{cowl_vesting_vesting_status, setup, TestContext},
    support::get_dictionary_value_from_key,
};
use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use casper_types::Key;
use cowl_vesting::{
    constants::{DICT_VESTING_STATUS, YEAR_IN_SECONDS},
    enums::VestingType,
    vesting::VestingStatus,
};

#[test]
fn should_get_vesting_treasury_status() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Treasury;

    let vesting_vesting_status_call = cowl_vesting_vesting_status(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
        None,
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_contributor_status() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Contributor;

    let vesting_vesting_status_call = cowl_vesting_vesting_status(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
        None,
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_development_status() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Development;

    let vesting_vesting_status_call = cowl_vesting_vesting_status(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
        None,
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_liquidity_status() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Liquidity;

    let vesting_vesting_status_call = cowl_vesting_vesting_status(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
        None,
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );

    assert_eq!(vesting_status.vesting_duration, Duration::ZERO);
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_community_status() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Community;

    let vesting_vesting_status_call = cowl_vesting_vesting_status(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
        None,
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_staking_status() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Staking;

    let vesting_vesting_status_call = cowl_vesting_vesting_status(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
        None,
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );
    dbg!(vesting_status);
}
