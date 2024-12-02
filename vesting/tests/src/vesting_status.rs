use crate::utility::installer_request_builders::{cowl_vesting_vesting_status, setup, TestContext};
use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use cowl_vesting::enums::VestingType;

#[test]
fn should_get_vesting_status() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            cowl_cep18_token_contract_hash,
            ..
        },
    ) = setup();

    let _vesting_contract = builder
        .get_contract(cowl_vesting_contract_hash)
        .expect("should have vesting contract");
    let _cowl_cep18_token_contract = builder
        .get_contract(cowl_cep18_token_contract_hash)
        .expect("should have cowl cep18 token contract");

    let vesting_vesting_status_call = cowl_vesting_vesting_status(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        VestingType::Treasury,
    );
    vesting_vesting_status_call.expect_success().commit();
}
