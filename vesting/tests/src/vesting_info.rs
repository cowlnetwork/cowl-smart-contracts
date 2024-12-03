use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use casper_types::Key;
use cowl_vesting::{constants::DICT_VESTING_INFO, enums::VestingType, vesting::VestingInfo};

use crate::utility::{
    installer_request_builders::{cowl_vesting_vesting_info, setup, TestContext},
    support::get_dictionary_value_from_key,
};

#[test]
fn should_get_vesting_treasury_info() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Treasury;

    let cowl_vesting_vesting_info_call = cowl_vesting_vesting_info(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
    );
    cowl_vesting_vesting_info_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();

    let vesting_info: VestingInfo = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_INFO,
        &dictionary_key,
    );

    dbg!(vesting_info);
}

#[test]
fn should_get_vesting_contributor_info() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Contributor;

    let cowl_vesting_vesting_info_call = cowl_vesting_vesting_info(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
    );
    cowl_vesting_vesting_info_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();

    let vesting_info: VestingInfo = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_INFO,
        &dictionary_key,
    );

    dbg!(vesting_info);
}

#[test]
fn should_get_vesting_development_info() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Development;

    let cowl_vesting_vesting_info_call = cowl_vesting_vesting_info(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
    );
    cowl_vesting_vesting_info_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();

    let vesting_info: VestingInfo = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_INFO,
        &dictionary_key,
    );

    dbg!(vesting_info);
}

#[test]
fn should_get_vesting_liquidity_info() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Liquidity;

    let cowl_vesting_vesting_info_call = cowl_vesting_vesting_info(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
    );
    cowl_vesting_vesting_info_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();

    let vesting_info: VestingInfo = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_INFO,
        &dictionary_key,
    );

    dbg!(vesting_info);
}

#[test]
fn should_get_vesting_community_info() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Community;

    let cowl_vesting_vesting_info_call = cowl_vesting_vesting_info(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
    );
    cowl_vesting_vesting_info_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();

    let vesting_info: VestingInfo = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_INFO,
        &dictionary_key,
    );

    dbg!(vesting_info);
}

#[test]
fn should_get_vesting_staking_info() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Staking;

    let cowl_vesting_vesting_info_call = cowl_vesting_vesting_info(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
    );
    cowl_vesting_vesting_info_call.expect_success().commit();

    let dictionary_key = vesting_type.to_string();

    let vesting_info: VestingInfo = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_INFO,
        &dictionary_key,
    );

    dbg!(vesting_info);
}
