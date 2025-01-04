use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use casper_types::U512;
use cowl_swap::error::SwapError;
use vesting_tests::{constants::ACCOUNT_LIQUIDITY, support::assert_expected_error};

use crate::utility::{
    constants::MINIMUM_TRANSFER_AMOUNT,
    installer_request_builders::{
        cowl_swap_deposit_cspr, cowl_swap_withdraw_cspr, setup, TestContext,
    },
};

#[test]
fn should_fail_withdraw_cspr_when_non_installer() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_package_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let deposit_cspr = cowl_swap_deposit_cspr(
        &mut builder,
        &DEFAULT_ACCOUNT_ADDR,
        &cowl_swap_contract_package_hash,
        U512::from_dec_str(MINIMUM_TRANSFER_AMOUNT).unwrap(),
    );

    deposit_cspr.expect_success().commit();

    let liquidity = *test_accounts.get(&ACCOUNT_LIQUIDITY).unwrap();

    let withdraw_cspr = cowl_swap_withdraw_cspr(
        &mut builder,
        &liquidity,
        &cowl_swap_contract_package_hash,
        U512::one(),
    );

    withdraw_cspr.expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(
        error,
        SwapError::InsufficientRights as u16,
        "withdraw_cspr is only installer entrypoint",
    );
}

#[test]
fn should_withdraw_cspr() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_hash,
            cowl_swap_contract_package_hash,
            ..
        },
    ) = setup();

    let deposit_cspr = cowl_swap_deposit_cspr(
        &mut builder,
        &DEFAULT_ACCOUNT_ADDR,
        &cowl_swap_contract_package_hash,
        U512::from_dec_str(MINIMUM_TRANSFER_AMOUNT).unwrap(),
    );

    deposit_cspr.expect_success().commit();

    let withdraw_cspr = cowl_swap_withdraw_cspr(
        &mut builder,
        &DEFAULT_ACCOUNT_ADDR,
        &cowl_swap_contract_package_hash,
        U512::from_dec_str(MINIMUM_TRANSFER_AMOUNT).unwrap(),
    );

    withdraw_cspr.expect_success().commit();

    let swap_contract = builder
        .get_contract(cowl_swap_contract_hash)
        .expect("should have swap contract");
    let named_keys = swap_contract.named_keys();
    dbg!(named_keys);
}

#[test]
fn should_fail_withdraw_cspr_more_than_contract_balance() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_package_hash,
            ..
        },
    ) = setup();

    let deposit_cspr = cowl_swap_deposit_cspr(
        &mut builder,
        &DEFAULT_ACCOUNT_ADDR,
        &cowl_swap_contract_package_hash,
        U512::from_dec_str(MINIMUM_TRANSFER_AMOUNT).unwrap(),
    );

    deposit_cspr.expect_success().commit();

    let withdraw_cspr = cowl_swap_withdraw_cspr(
        &mut builder,
        &DEFAULT_ACCOUNT_ADDR,
        &cowl_swap_contract_package_hash,
        U512::from_dec_str(MINIMUM_TRANSFER_AMOUNT)
            .unwrap()
            .checked_add(U512::one())
            .unwrap(),
    );

    withdraw_cspr.expect_failure();
    let error = builder.get_error().expect("must have error");

    assert_expected_error(
        error,
        SwapError::InvalidPurseTransfer as u16,
        "can not withdraw more than contract balance",
    );
}
