use crate::utility::{
    constants::{SWAP_CONTRACT_VERSION, SWAP_CONTRACT_WASM, SWAP_TEST_NAME},
    installer_request_builders::{setup, TestContext},
};
use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{runtime_args, Key, RuntimeArgs};
use cowl_swap::{
    constants::{ARG_COWL_CEP18_CONTRACT_PACKAGE, ARG_EVENTS_MODE, ARG_NAME},
    enums::EventsMode,
};

#[test]
fn should_install_contract() {
    let (
        builder,
        TestContext {
            cowl_swap_contract_hash,
            cowl_cep18_token_contract_hash,
            cowl_vesting_contract_hash,
            ..
        },
    ) = setup();

    let cowl_cep18_token_contract = builder
        .get_contract(cowl_cep18_token_contract_hash)
        .expect("should have cowl cep18 token contract");
    let named_keys = cowl_cep18_token_contract.named_keys();
    dbg!(named_keys);
    let vesting_contract = builder
        .get_contract(cowl_vesting_contract_hash)
        .expect("should have vesting contract");
    let named_keys = vesting_contract.named_keys();
    dbg!(named_keys);
    let swap_contract = builder
        .get_contract(cowl_swap_contract_hash)
        .expect("should have swap contract");

    let named_keys = swap_contract.named_keys();
    dbg!(named_keys);
}

#[test]
fn should_prevent_reinstall_contract() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_hash,
            cowl_cep18_token_package_hash,
            ..
        },
    ) = setup();

    let version_key = *builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account")
        .named_keys()
        .get(SWAP_CONTRACT_VERSION)
        .expect("version uref should exist");

    let version = builder
        .query(None, version_key, &[])
        .expect("should be stored value.")
        .as_cl_value()
        .expect("should be cl value.")
        .clone()
        .into_t::<u32>()
        .expect("should be u32.");

    dbg!(version);

    let swap_contract = builder
        .get_contract(cowl_swap_contract_hash)
        .expect("should have swap contract");
    let named_keys = swap_contract.named_keys();
    dbg!(named_keys);

    let install_args = runtime_args!(
        ARG_NAME => SWAP_TEST_NAME,
        ARG_EVENTS_MODE => EventsMode::CES as u8,
        ARG_COWL_CEP18_CONTRACT_PACKAGE =>
        Key::from(cowl_cep18_token_package_hash),
    );

    // Install swap contract with token
    let reinstall_request_contract =
        ExecuteRequestBuilder::standard(*DEFAULT_ACCOUNT_ADDR, SWAP_CONTRACT_WASM, install_args)
            .build();

    builder
        .exec(reinstall_request_contract)
        .expect_success()
        .commit();

    let swap_contract = builder
        .get_contract(cowl_swap_contract_hash)
        .expect("should have swap contract");
    let new_named_keys = swap_contract.named_keys();
    dbg!(new_named_keys);

    assert_eq!(named_keys, new_named_keys)
}
