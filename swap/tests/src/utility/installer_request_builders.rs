use crate::utility::constants::{SWAP_CONTRACT_WASM, SWAP_TEST_NAME};
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
};
use casper_types::{
    account::AccountHash, runtime_args, ContractHash, ContractPackageHash, Key, RuntimeArgs,
};
use cowl_swap::{
    constants::{
        ADMIN_LIST, ARG_EVENTS_MODE, ARG_NAME, ENTRY_POINT_CHANGE_SECURITY,
        ENTRY_POINT_SET_MODALITIES, NONE_LIST,
    },
    enums::EventsMode,
};
use std::collections::HashMap;
#[cfg(test)]
use vesting_tests::setup as setup_vesting;
use vesting_tests::TestContextVesting;

use super::constants::{SWAP_CONTRACT_KEY_NAME, SWAP_CONTRACT_PACKAGE_HASH_KEY_NAME};

#[derive(Clone)]
pub(crate) struct TestContext {
    pub(crate) cowl_swap_contract_hash: ContractHash,
    pub(crate) cowl_swap_contract_package_hash: ContractPackageHash,
    pub(crate) cowl_cep18_token_contract_hash: ContractHash,
    pub(crate) cowl_cep18_token_package_hash: ContractPackageHash,
    pub(crate) cowl_vesting_contract_hash: ContractHash,
    pub(crate) test_accounts: HashMap<[u8; 32], AccountHash>,
}

impl Drop for TestContext {
    fn drop(&mut self) {}
}

fn default_args() -> RuntimeArgs {
    runtime_args! {
        ARG_NAME => SWAP_TEST_NAME,
        ARG_EVENTS_MODE => EventsMode::CES as u8,
    }
}

pub fn setup() -> (InMemoryWasmTestBuilder, TestContext) {
    setup_with_args(default_args())
}

pub fn setup_with_args(install_args: RuntimeArgs) -> (InMemoryWasmTestBuilder, TestContext) {
    let (
        mut builder,
        TestContextVesting {
            cowl_vesting_contract_hash,
            cowl_cep18_token_contract_hash,
            cowl_cep18_token_package_hash,
            ref test_accounts,
            ..
        },
    ) = setup_vesting();

    // dbg!(test_accounts.clone());

    // Install SWAP contract with token
    let install_request_contract = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        SWAP_CONTRACT_WASM,
        merge_args(install_args),
    )
    .build();

    builder
        .exec(install_request_contract)
        .expect_success()
        .commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let cowl_swap_contract_hash = account
        .named_keys()
        .get(SWAP_CONTRACT_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let cowl_swap_contract_package_hash = account
        .named_keys()
        .get(SWAP_CONTRACT_PACKAGE_HASH_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have package hash");

    let test_context = TestContext {
        cowl_swap_contract_hash,
        cowl_swap_contract_package_hash,
        cowl_cep18_token_contract_hash,
        cowl_cep18_token_package_hash,
        cowl_vesting_contract_hash,
        test_accounts: test_accounts.clone(),
    };

    (builder, test_context)
}

pub fn cowl_swap_set_modalities<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    cowl_swap: &'a ContractHash,
    owner: &'a AccountHash,
    events_mode: Option<EventsMode>,
) -> &'a mut InMemoryWasmTestBuilder {
    let mut args = runtime_args! {};
    if let Some(events_mode) = events_mode {
        let _ = args.insert(ARG_EVENTS_MODE, events_mode as u8);
    };
    let set_modalities_request = ExecuteRequestBuilder::contract_call_by_hash(
        *owner,
        *cowl_swap,
        ENTRY_POINT_SET_MODALITIES,
        args,
    )
    .build();
    builder.exec(set_modalities_request)
}

pub struct SecurityLists {
    pub admin_list: Option<Vec<Key>>,
    pub none_list: Option<Vec<Key>>,
}

pub fn cowl_swap_change_security<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    cowl_swap: &'a ContractHash,
    admin_account: &'a AccountHash,
    security_lists: SecurityLists,
) -> &'a mut InMemoryWasmTestBuilder {
    let SecurityLists {
        admin_list,
        none_list,
    } = security_lists;

    let change_security_request = ExecuteRequestBuilder::contract_call_by_hash(
        *admin_account,
        *cowl_swap,
        ENTRY_POINT_CHANGE_SECURITY,
        runtime_args! {
            ADMIN_LIST => admin_list.unwrap_or_default(),
            NONE_LIST => none_list.unwrap_or_default(),
        },
    )
    .build();
    builder.exec(change_security_request)
}

fn merge_args(install_args: RuntimeArgs) -> RuntimeArgs {
    let mut merged_args = install_args;

    if merged_args.get(ARG_NAME).is_none() {
        if let Some(default_name_value) = default_args().get(ARG_NAME) {
            merged_args.insert_cl_value(ARG_NAME, default_name_value.clone());
        }
    }
    if merged_args.get(ARG_EVENTS_MODE).is_none() {
        if let Some(default_name_value) = default_args().get(ARG_EVENTS_MODE) {
            merged_args.insert_cl_value(ARG_EVENTS_MODE, default_name_value.clone());
        }
    }
    merged_args
}
