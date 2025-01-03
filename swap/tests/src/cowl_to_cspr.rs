use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use casper_types::{Key, U256, U512};
use cowl_swap::constants::{ARG_BALANCE_COWL, RATE_TIERS};
use vesting_tests::constants::{ACCOUNT_LIQUIDITY, ACCOUNT_USER_1};

use crate::utility::installer_request_builders::{
    cowl_cep18_token_balance_cowl, cowl_swap_balance_cowl, cowl_swap_cowl_to_cspr,
    cowl_swap_cspr_to_cowl, cowl_swap_deposit_cowl, setup, TestContext,
};

#[test]
fn should_cowl_to_cspr() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_package_hash,
            cowl_cep18_token_package_hash,
            cowl_swap_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let liquidity = *test_accounts.get(&ACCOUNT_LIQUIDITY).unwrap();

    let cowl_pool_amount = U256::from_dec_str("100000000000").unwrap();
    let cspr_transfer_amount = U512::from_dec_str("10000000000").unwrap();

    let deposit_cowl = cowl_swap_deposit_cowl(
        &mut builder,
        &liquidity,
        &cowl_cep18_token_package_hash,
        &cowl_swap_contract_package_hash,
        cowl_pool_amount,
    );

    deposit_cowl.expect_success().commit();

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let account_user_1_key = Key::from(account_user_1);

    let account = builder.get_account(account_user_1).unwrap();

    let initial_balance_cspr = builder.get_purse_balance(account.main_purse());

    dbg!(initial_balance_cspr);

    let cspr_to_cowl = cowl_swap_cspr_to_cowl(
        &mut builder,
        &account_user_1,
        &cowl_swap_contract_package_hash,
        cspr_transfer_amount,
    );

    cspr_to_cowl.expect_success().commit();

    let token_balance = cowl_cep18_token_balance_cowl(
        &mut builder,
        &account_user_1,
        &cowl_cep18_token_package_hash,
        &account_user_1_key,
    );

    token_balance.expect_success().commit();

    let account = builder.get_account(account_user_1).unwrap();

    let balance_key = account
        .named_keys()
        .get(ARG_BALANCE_COWL)
        .expect("balance uref should exist");

    let actual_balance_cowl = builder
        .query(None, *balance_key, &[])
        .expect("should be stored value.")
        .as_cl_value()
        .expect("should be cl value.")
        .clone()
        .into_t::<U256>()
        .expect("should be U256.");

    let expected_balance_cowl = cspr_transfer_amount * RATE_TIERS.first().unwrap().rate;

    assert_eq!(
        actual_balance_cowl,
        U256::from_dec_str(&expected_balance_cowl.to_string()).unwrap()
    );

    let cowl_to_cspr = cowl_swap_cowl_to_cspr(
        &mut builder,
        &account_user_1,
        &cowl_cep18_token_package_hash,
        &cowl_swap_contract_package_hash,
        actual_balance_cowl,
    );

    cowl_to_cspr.expect_success().commit();

    let token_balance = cowl_cep18_token_balance_cowl(
        &mut builder,
        &account_user_1,
        &cowl_cep18_token_package_hash,
        &account_user_1_key,
    );

    token_balance.expect_success().commit();

    let account = builder.get_account(account_user_1).unwrap();

    let balance_key = account
        .named_keys()
        .get(ARG_BALANCE_COWL)
        .expect("balance uref should exist");

    let actual_balance_cowl = builder
        .query(None, *balance_key, &[])
        .expect("should be stored value.")
        .as_cl_value()
        .expect("should be cl value.")
        .clone()
        .into_t::<U256>()
        .expect("should be U256.");

    assert_eq!(actual_balance_cowl, U256::zero());

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

    let actual_balance_cowl = builder
        .query(None, *balance_key, &[])
        .expect("should be stored value.")
        .as_cl_value()
        .expect("should be cl value.")
        .clone()
        .into_t::<U256>()
        .expect("should be U256.");

    assert_eq!(actual_balance_cowl, cowl_pool_amount);

    let final_balance_cspr = builder.get_purse_balance(account.main_purse());

    assert!(initial_balance_cspr > final_balance_cspr);

    dbg!(final_balance_cspr);
}
