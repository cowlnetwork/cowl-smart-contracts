use crate::utility::{
    constants::MINIMUM_TRANSFER_AMOUNT,
    installer_request_builders::{
        cowl_swap_balance_cspr, cowl_swap_deposit_cspr, setup, TestContext,
    },
};
use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use casper_types::U512;
use cowl_swap::{constants::ARG_BALANCE_CSPR, error::SwapError};
use vesting_tests::{constants::ACCOUNT_LIQUIDITY, support::assert_expected_error};

#[test]
fn should_fail_deposit_cspr_when_non_installer() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_package_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let liquidity = *test_accounts.get(&ACCOUNT_LIQUIDITY).unwrap();

    let deposit_cspr = cowl_swap_deposit_cspr(
        &mut builder,
        &liquidity,
        &cowl_swap_contract_package_hash,
        U512::one(),
    );

    deposit_cspr.expect_failure();
    let error = builder.get_error().expect("must have error");

    assert_expected_error(
        error,
        SwapError::InsufficientRights as u16,
        "withdraw_cspr is only installer entrypoint",
    );
}

#[test]
fn should_deposit_cspr_when_installer() {
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

    let balance_cspr = cowl_swap_balance_cspr(
        &mut builder,
        &DEFAULT_ACCOUNT_ADDR,
        &cowl_swap_contract_package_hash,
    );

    balance_cspr.expect_success().commit();

    let swap_contract = builder
        .get_contract(cowl_swap_contract_hash)
        .expect("should have swap contract");

    let balance_key = swap_contract
        .named_keys()
        .get(ARG_BALANCE_CSPR)
        .expect("balance uref should exist");

    let actual_balance = builder
        .query(None, *balance_key, &[])
        .expect("should be stored value.")
        .as_cl_value()
        .expect("should be cl value.")
        .clone()
        .into_t::<U512>()
        .expect("should be U512.");

    assert_eq!(
        actual_balance,
        U512::from_dec_str(MINIMUM_TRANSFER_AMOUNT).unwrap()
    );

    let named_keys = swap_contract.named_keys();
    dbg!(named_keys);
}
