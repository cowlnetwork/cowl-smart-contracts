use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use casper_types::U256;
use cowl_swap::constants::ARG_BALANCE_COWL;
use vesting_tests::constants::ACCOUNT_LIQUIDITY;

use crate::utility::installer_request_builders::{
    cowl_swap_balance_cowl, cowl_swap_deposit_cowl, setup, TestContext,
};

#[test]
fn should_deposit_cowl() {
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

    let balance_cowl = cowl_swap_balance_cowl(
        &mut builder,
        &DEFAULT_ACCOUNT_ADDR,
        &cowl_swap_contract_package_hash,
    );

    balance_cowl.expect_success().commit();

    let swap_contract = builder
        .get_contract(cowl_swap_contract_hash)
        .expect("should have swap contract");

    let balance_key = swap_contract
        .named_keys()
        .get(ARG_BALANCE_COWL)
        .expect("balance uref should exist");

    let actual_balance = builder
        .query(None, *balance_key, &[])
        .expect("should be stored value.")
        .as_cl_value()
        .expect("should be cl value.")
        .clone()
        .into_t::<U256>()
        .expect("should be U256.");

    assert_eq!(actual_balance, U256::one());

    let named_keys = swap_contract.named_keys();
    dbg!(named_keys);
}
