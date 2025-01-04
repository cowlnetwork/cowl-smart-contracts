use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use casper_types::U256;
use cowl_swap::error::SwapError;
use vesting_tests::{constants::ACCOUNT_LIQUIDITY, support::assert_expected_error};

use crate::utility::installer_request_builders::{
    cowl_swap_deposit_cowl, cowl_swap_withdraw_cowl, setup, TestContext,
};

#[test]
fn should_fail_withdraw_cowl_when_non_installer() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_package_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let liquidity = *test_accounts.get(&ACCOUNT_LIQUIDITY).unwrap();

    let withdraw_cowl = cowl_swap_withdraw_cowl(
        &mut builder,
        &liquidity,
        &cowl_swap_contract_package_hash,
        U256::one(),
    );

    withdraw_cowl.expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(
        error,
        SwapError::InsufficientRights as u16,
        "withdraw_cowl is only installer entrypoint",
    );
}

#[test]
fn should_withdraw_cowl() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_hash,
            cowl_swap_contract_package_hash,
            cowl_cep18_token_package_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let liquidity = *test_accounts.get(&ACCOUNT_LIQUIDITY).unwrap();

    let deposit_cowl = cowl_swap_deposit_cowl(
        &mut builder,
        &liquidity,
        &cowl_cep18_token_package_hash,
        &cowl_swap_contract_package_hash,
        U256::one(),
    );

    deposit_cowl.expect_success().commit();

    let withdraw_cowl = cowl_swap_withdraw_cowl(
        &mut builder,
        &DEFAULT_ACCOUNT_ADDR,
        &cowl_swap_contract_package_hash,
        U256::one(),
    );

    withdraw_cowl.expect_success().commit();

    let swap_contract = builder
        .get_contract(cowl_swap_contract_hash)
        .expect("should have swap contract");

    let named_keys = swap_contract.named_keys();
    dbg!(named_keys);
}

#[test]
fn should_fail_withdraw_cowl_more_than_contract_balance() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_package_hash,
            cowl_cep18_token_package_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let liquidity = *test_accounts.get(&ACCOUNT_LIQUIDITY).unwrap();

    let deposit_cowl = cowl_swap_deposit_cowl(
        &mut builder,
        &liquidity,
        &cowl_cep18_token_package_hash,
        &cowl_swap_contract_package_hash,
        U256::one(),
    );

    deposit_cowl.expect_success().commit();

    let withdraw_cowl = cowl_swap_withdraw_cowl(
        &mut builder,
        &DEFAULT_ACCOUNT_ADDR,
        &cowl_swap_contract_package_hash,
        U256::from("2"),
    );

    withdraw_cowl.expect_failure();
    let error = builder.get_error().expect("must have error");

    assert_expected_error(error, 60001, "can not withdraw more than contract balance");
}
