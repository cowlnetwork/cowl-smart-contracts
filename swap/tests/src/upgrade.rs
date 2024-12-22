use crate::utility::{
    constants::{
        SWAP_CONTRACT_KEY_NAME, SWAP_CONTRACT_VERSION, SWAP_CONTRACT_WASM, SWAP_TEST_NAME,
    },
    installer_request_builders::{setup, TestContext},
};
use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs};
use cowl_swap::{
    constants::{ARG_CONTRACT_HASH, ARG_NAME, ARG_UPGRADE_FLAG},
    events::Upgrade,
};
use vesting_tests::support::get_event;

#[test]
fn should_upgrade_and_update_account_contract_contexts() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_hash,
            ..
        },
    ) = setup();

    let contract = builder
        .get_contract(cowl_swap_contract_hash)
        .expect("should have contract");

    let cowl_swap_contract_version = builder
        .query(
            None,
            Key::from(*DEFAULT_ACCOUNT_ADDR),
            &[SWAP_CONTRACT_VERSION.to_string()],
        )
        .unwrap()
        .as_cl_value()
        .unwrap()
        .to_owned()
        .into_t::<u32>()
        .unwrap();

    assert_eq!(cowl_swap_contract_version, 1_u32);

    let contract_hash_on_install: ContractHash = contract
        .named_keys()
        .get(ARG_CONTRACT_HASH)
        .expect("should have contract hash")
        .into_hash()
        .unwrap()
        .into();

    let upgrade_request_contract = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        SWAP_CONTRACT_WASM,
        runtime_args! {
            ARG_UPGRADE_FLAG => true,
            ARG_NAME => SWAP_TEST_NAME,
        },
    )
    .build();

    builder
        .exec(upgrade_request_contract)
        .expect_success()
        .commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let upgraded_cowl_swap: ContractHash = account
        .named_keys()
        .get(SWAP_CONTRACT_KEY_NAME)
        .unwrap()
        .into_hash()
        .unwrap()
        .into();

    let contract = builder
        .get_contract(upgraded_cowl_swap)
        .expect("should have contract");

    let contract_hash_after_upgrade: ContractHash = contract
        .named_keys()
        .get(ARG_CONTRACT_HASH)
        .unwrap()
        .into_hash()
        .unwrap()
        .into();

    assert_ne!(
        contract_hash_on_install.to_formatted_string(),
        contract_hash_after_upgrade.to_formatted_string()
    );

    let cowl_swap_contract_version = builder
        .query(
            None,
            Key::from(*DEFAULT_ACCOUNT_ADDR),
            &[SWAP_CONTRACT_VERSION.to_string()],
        )
        .unwrap()
        .as_cl_value()
        .unwrap()
        .to_owned()
        .into_t::<u32>()
        .unwrap();

    assert_eq!(cowl_swap_contract_version, 2_u32);
}

#[test]
fn should_emit_event_on_upgrade_with_events_mode_ces() {
    let (
        mut builder,
        TestContext {
            cowl_swap_contract_hash,
            ..
        },
    ) = setup();

    let upgrade_request_contract = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        SWAP_CONTRACT_WASM,
        runtime_args! {
            ARG_UPGRADE_FLAG => true,
            ARG_NAME => SWAP_TEST_NAME,
        },
    )
    .build();
    builder
        .exec(upgrade_request_contract)
        .expect_success()
        .commit();

    // Expect Upgrade event
    let expected_event = Upgrade::new();
    let actual_event: Upgrade = get_event(&builder, &cowl_swap_contract_hash.into(), 0);
    assert_eq!(actual_event, expected_event, "Expected Upgrade event.");
}
