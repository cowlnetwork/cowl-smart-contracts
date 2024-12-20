use crate::utility::{
    installer_request_builders::{cowl_vesting_vesting_status, setup, TestContext},
    support::get_dictionary_value_from_key,
};
use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use casper_types::{Key, U256};
use cowl_vesting::{
    constants::{
        DICT_VESTING_STATUS, DURATION_COMMUNITY_VESTING, DURATION_CONTRIBUTOR_VESTING,
        DURATION_DEVELOPMENT_VESTING, DURATION_TREASURY_VESTING,
    },
    enums::VestingType,
    vesting::VestingStatus,
};
use std::time::Duration;

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
    assert!(!vesting_status.is_fully_vested);
    assert_eq!(vesting_status.vested_amount, U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_TREASURY_VESTING.unwrap()
    );
    assert!(vesting_status.time_until_next_release > Duration::ZERO);
    assert!(vesting_status.release_amount_per_period > U256::zero());
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
    assert!(!vesting_status.is_fully_vested);
    assert_eq!(vesting_status.vested_amount, U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_CONTRIBUTOR_VESTING.unwrap()
    );
    assert!(vesting_status.time_until_next_release > Duration::ZERO);
    assert!(vesting_status.release_amount_per_period > U256::zero());
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
    assert!(!vesting_status.is_fully_vested);
    assert_eq!(vesting_status.vested_amount, U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_DEVELOPMENT_VESTING.unwrap()
    );
    assert!(vesting_status.time_until_next_release > Duration::ZERO);
    assert!(vesting_status.release_amount_per_period > U256::zero());
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

    assert!(vesting_status.is_fully_vested);
    assert_eq!(vesting_status.vested_amount, U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(vesting_status.vesting_duration, Duration::ZERO);
    assert_eq!(vesting_status.time_until_next_release, Duration::ZERO);
    assert_eq!(vesting_status.release_amount_per_period, U256::zero());
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

    assert!(!vesting_status.is_fully_vested);
    assert_eq!(vesting_status.vested_amount, U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_COMMUNITY_VESTING.unwrap()
    );
    assert!(vesting_status.time_until_next_release > Duration::ZERO);
    assert!(vesting_status.release_amount_per_period > U256::zero());
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
    assert!(vesting_status.is_fully_vested);
    assert_eq!(vesting_status.vested_amount, U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(vesting_status.vesting_duration, Duration::ZERO);
    assert_eq!(vesting_status.time_until_next_release, Duration::ZERO);
    assert_eq!(vesting_status.release_amount_per_period, U256::zero());
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_contributor_status_half_year() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Contributor;

    let test_duration = DURATION_CONTRIBUTOR_VESTING.map(|d| (d.whole_seconds() / 2) as u64);

    let vesting_vesting_status_call = cowl_vesting_vesting_status(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
        test_duration,
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );

    assert!(!vesting_status.is_fully_vested);
    assert!(vesting_status.vested_amount > U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_CONTRIBUTOR_VESTING.unwrap()
    );
    assert!(vesting_status.time_until_next_release > Duration::ZERO);
    assert!(vesting_status.release_amount_per_period > U256::zero());
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_contributor_status_one_and_half_year() {
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
        DURATION_CONTRIBUTOR_VESTING.map(|d| (d.whole_seconds() * 15 / 10) as u64),
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );

    assert!(vesting_status.is_fully_vested);
    assert!(vesting_status.vested_amount > U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_CONTRIBUTOR_VESTING.unwrap()
    );
    assert_eq!(vesting_status.time_until_next_release, Duration::ZERO);
    assert!(vesting_status.release_amount_per_period > U256::zero());
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_development_status_half_year() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Development;

    let test_duration = DURATION_DEVELOPMENT_VESTING.map(|d| (d.whole_seconds() / 2) as u64);

    let vesting_vesting_status_call = cowl_vesting_vesting_status(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
        test_duration,
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );

    assert!(!vesting_status.is_fully_vested);
    assert!(vesting_status.vested_amount > U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_DEVELOPMENT_VESTING.unwrap()
    );
    assert!(vesting_status.time_until_next_release > Duration::ZERO);
    assert!(vesting_status.release_amount_per_period > U256::zero());
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_development_status_one_and_half_year() {
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
        DURATION_DEVELOPMENT_VESTING.map(|d| (d.whole_seconds() * 15 / 10) as u64),
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );

    assert!(vesting_status.is_fully_vested);
    assert!(vesting_status.vested_amount > U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_DEVELOPMENT_VESTING.unwrap()
    );
    assert_eq!(vesting_status.time_until_next_release, Duration::ZERO);
    assert!(vesting_status.release_amount_per_period > U256::zero());
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_treasury_status_two_year() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Treasury;

    let test_duration = DURATION_TREASURY_VESTING.map(|d| (d.whole_seconds() / 2) as u64);

    let vesting_vesting_status_call = cowl_vesting_vesting_status(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
        test_duration,
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );

    assert!(!vesting_status.is_fully_vested);
    assert!(vesting_status.vested_amount > U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_TREASURY_VESTING.unwrap()
    );
    assert!(vesting_status.time_until_next_release > Duration::ZERO);
    assert!(vesting_status.release_amount_per_period > U256::zero());
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_treasury_status_six_year() {
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
        DURATION_TREASURY_VESTING.map(|d| (d.whole_seconds() * 15 / 10) as u64),
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );

    assert!(vesting_status.is_fully_vested);
    assert!(vesting_status.vested_amount > U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_TREASURY_VESTING.unwrap()
    );
    assert_eq!(vesting_status.time_until_next_release, Duration::ZERO);
    assert!(vesting_status.release_amount_per_period > U256::zero());
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_community_status_two_year() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Community;

    let test_duration = DURATION_COMMUNITY_VESTING.map(|d| (d.whole_seconds() / 2) as u64);

    let vesting_vesting_status_call = cowl_vesting_vesting_status(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
        test_duration,
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );

    assert!(!vesting_status.is_fully_vested);
    assert!(vesting_status.vested_amount > U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_COMMUNITY_VESTING.unwrap()
    );
    assert!(vesting_status.time_until_next_release > Duration::ZERO);
    assert!(vesting_status.release_amount_per_period > U256::zero());
    dbg!(vesting_status);
}

#[test]
fn should_get_vesting_community_status_six_year() {
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
        DURATION_COMMUNITY_VESTING.map(|d| (d.whole_seconds() * 15 / 10) as u64),
    );
    vesting_vesting_status_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();
    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &dictionary_key.to_owned(),
    );

    assert!(vesting_status.is_fully_vested);
    assert!(vesting_status.vested_amount > U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_COMMUNITY_VESTING.unwrap()
    );
    assert_eq!(vesting_status.time_until_next_release, Duration::ZERO);
    //  assert_eq!(vesting_status.release_amount_per_period, U256::zero());
    dbg!(vesting_status);
}
