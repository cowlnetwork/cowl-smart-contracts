use crate::utility::constants::{SWAP_CONTRACT_WASM, SWAP_TEST_NAME};
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
};
use casper_types::{
    account::AccountHash, runtime_args, ContractHash, ContractPackageHash, Key, RuntimeArgs, U256,
    U512,
};
use cowl_swap::{
    constants::{
        ADMIN_LIST, ARG_ADDRESS, ARG_AMOUNT, ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH,
        ARG_COWL_SWAP_CONTRACT_PACKAGE_HASH, ARG_END_TIME, ARG_EVENTS_MODE, ARG_NAME,
        ARG_START_TIME, ENTRY_POINT_BALANCE_COWL, ENTRY_POINT_BALANCE_CSPR,
        ENTRY_POINT_CHANGE_SECURITY, ENTRY_POINT_SET_MODALITIES, ENTRY_POINT_UPDATE_TIMES,
        ENTRY_POINT_WITHDRAW_COWL, ENTRY_POINT_WITHDRAW_CSPR, NONE_LIST,
    },
    enums::EventsMode,
};
use std::collections::HashMap;
#[cfg(test)]
use vesting_tests::setup as setup_vesting;
use vesting_tests::TestContextVesting;

use super::constants::{
    SWAP_BALANCE_COWL_SESSION_WASM, SWAP_CONTRACT_KEY_NAME, SWAP_CONTRACT_PACKAGE_HASH_KEY_NAME,
    SWAP_COWL_TO_CSPR_SESSION_WASM, SWAP_CSPR_TO_COWL_SESSION_WASM, SWAP_DEPOSIT_COWL_SESSION_WASM,
    SWAP_DEPOSIT_CSPR_SESSION_WASM,
};

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

pub fn default_args() -> RuntimeArgs {
    runtime_args! {
        ARG_NAME => SWAP_TEST_NAME,
        ARG_EVENTS_MODE => EventsMode::CES as u8,
        ARG_START_TIME => 0_u64,
        ARG_END_TIME => 86400_u64,
    }
}

pub fn setup() -> (InMemoryWasmTestBuilder, TestContext) {
    setup_with_args(default_args())
}

pub fn setup_with_args(mut install_args: RuntimeArgs) -> (InMemoryWasmTestBuilder, TestContext) {
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

    // Install vesting contract with token package as install ARG
    let _ = install_args.insert(
        ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH.to_string(),
        Key::from(cowl_cep18_token_package_hash),
    );

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

pub fn cowl_swap_update_times<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    cowl_swap: &'a ContractHash,
    new_start_time: u64,
    new_end_time: u64,
) -> &'a mut InMemoryWasmTestBuilder {
    let update_times_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        *cowl_swap,
        ENTRY_POINT_UPDATE_TIMES,
        runtime_args! {
            ARG_START_TIME => new_start_time,
            ARG_END_TIME => new_end_time,
        },
    )
    .build();
    builder.exec(update_times_request)
}

pub fn cowl_swap_deposit_cowl<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    sender_account: &'a AccountHash,
    cowl_cep18_contract_package_key: &'a ContractPackageHash,
    cowl_swap_contract_package_hash: &'a ContractPackageHash,
    amount: U256,
) -> &'a mut InMemoryWasmTestBuilder {
    let deposit_cowl_request = ExecuteRequestBuilder::standard(
        *sender_account,
        SWAP_DEPOSIT_COWL_SESSION_WASM,
        runtime_args! {
            ARG_AMOUNT => amount,
            ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH => Key::from(*cowl_cep18_contract_package_key),
            ARG_COWL_SWAP_CONTRACT_PACKAGE_HASH => Key::from(*cowl_swap_contract_package_hash)
        },
    )
    .build();
    builder.exec(deposit_cowl_request)
}

pub fn cowl_swap_deposit_cspr<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    sender_account: &'a AccountHash,
    cowl_swap_contract_package_hash: &'a ContractPackageHash,
    amount: U512,
) -> &'a mut InMemoryWasmTestBuilder {
    let swap_deposit_cspr_request = ExecuteRequestBuilder::standard(
        *sender_account,
        SWAP_DEPOSIT_CSPR_SESSION_WASM,
        runtime_args! {
            ARG_AMOUNT => amount,
            ARG_COWL_SWAP_CONTRACT_PACKAGE_HASH => Key::from(*cowl_swap_contract_package_hash)
        },
    )
    .build();

    builder.exec(swap_deposit_cspr_request)
}

pub fn cowl_swap_balance_cspr<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    sender_account: &'a AccountHash,
    cowl_swap_contract_package_hash: &'a ContractPackageHash,
) -> &'a mut InMemoryWasmTestBuilder {
    let withdraw_cspr_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *sender_account,
        *cowl_swap_contract_package_hash,
        None,
        ENTRY_POINT_BALANCE_CSPR,
        runtime_args! {},
    )
    .build();
    builder.exec(withdraw_cspr_request)
}

pub fn cowl_swap_balance_cowl<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    sender_account: &'a AccountHash,
    cowl_swap_contract_package_hash: &'a ContractPackageHash,
) -> &'a mut InMemoryWasmTestBuilder {
    let withdraw_cowl_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *sender_account,
        *cowl_swap_contract_package_hash,
        None,
        ENTRY_POINT_BALANCE_COWL,
        runtime_args! {},
    )
    .build();
    builder.exec(withdraw_cowl_request)
}

pub fn cowl_cep18_token_balance_cowl<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    sender_account: &'a AccountHash,
    cowl_cep18_token_package_hash: &'a ContractPackageHash,
    address: &'a Key,
) -> &'a mut InMemoryWasmTestBuilder {
    let withdraw_cowl_request = ExecuteRequestBuilder::standard(
        *sender_account,
        SWAP_BALANCE_COWL_SESSION_WASM,
        runtime_args! {
            ARG_ADDRESS => address,
            ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH => Key::from(*cowl_cep18_token_package_hash)

        },
    )
    .build();
    builder.exec(withdraw_cowl_request)
}

pub fn cowl_swap_withdraw_cowl<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    sender_account: &'a AccountHash,
    cowl_swap_contract_package_hash: &'a ContractPackageHash,
    amount: U256,
) -> &'a mut InMemoryWasmTestBuilder {
    let withdraw_cowl_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *sender_account,
        *cowl_swap_contract_package_hash,
        None,
        ENTRY_POINT_WITHDRAW_COWL,
        runtime_args! {
            ARG_AMOUNT => amount,
        },
    )
    .build();
    builder.exec(withdraw_cowl_request)
}

pub fn cowl_swap_withdraw_cspr<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    sender_account: &'a AccountHash,
    cowl_swap_contract_package_hash: &'a ContractPackageHash,
    amount: U512,
) -> &'a mut InMemoryWasmTestBuilder {
    let withdraw_cspr_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *sender_account,
        *cowl_swap_contract_package_hash,
        None,
        ENTRY_POINT_WITHDRAW_CSPR,
        runtime_args! {
            ARG_AMOUNT => amount,
        },
    )
    .build();
    builder.exec(withdraw_cspr_request)
}

pub fn cowl_swap_cspr_to_cowl<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    sender_account: &'a AccountHash,
    cowl_swap_contract_package_hash: &'a ContractPackageHash,
    amount: U512,
) -> &'a mut InMemoryWasmTestBuilder {
    let cspr_to_cowl_request = ExecuteRequestBuilder::standard(
        *sender_account,
        SWAP_CSPR_TO_COWL_SESSION_WASM,
        runtime_args! {
            ARG_AMOUNT => amount,
            ARG_COWL_SWAP_CONTRACT_PACKAGE_HASH => Key::from(*cowl_swap_contract_package_hash)

        },
    )
    .build();

    builder.exec(cspr_to_cowl_request)
}

pub fn cowl_swap_cowl_to_cspr<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    sender_account: &'a AccountHash,
    cowl_cep18_contract_package_hash: &'a ContractPackageHash,
    cowl_swap_contract_package_hash: &'a ContractPackageHash,
    amount: U256,
) -> &'a mut InMemoryWasmTestBuilder {
    let cowl_to_cspr_request = ExecuteRequestBuilder::standard(
        *sender_account,
        SWAP_COWL_TO_CSPR_SESSION_WASM,
        runtime_args! {
            ARG_AMOUNT => amount,
            ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH => Key::from(*cowl_cep18_contract_package_hash),
            ARG_COWL_SWAP_CONTRACT_PACKAGE_HASH => Key::from(*cowl_swap_contract_package_hash)

        },
    )
    .build();
    builder.exec(cowl_to_cspr_request)
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
    if merged_args.get(ARG_START_TIME).is_none() {
        if let Some(default_name_value) = default_args().get(ARG_START_TIME) {
            merged_args.insert_cl_value(ARG_START_TIME, default_name_value.clone());
        }
    }
    if merged_args.get(ARG_END_TIME).is_none() {
        if let Some(default_name_value) = default_args().get(ARG_END_TIME) {
            merged_args.insert_cl_value(ARG_END_TIME, default_name_value.clone());
        }
    }
    merged_args
}
