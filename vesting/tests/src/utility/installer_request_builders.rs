use crate::utility::{
    constants::{
        ACCOUNT_COMMUNITY, ACCOUNT_CONTRIBUTOR, ACCOUNT_DEVELOPMENT, ACCOUNT_STACKING,
        ACCOUNT_TREASURY, ACCOUNT_USER_1, ACCOUNT_USER_2, COWL_CEP18_TEST_TOKEN_CONTRACT_NAME,
        VESTING_CONTRACT_WASM, VESTING_TEST_NAME,
    },
    support::create_funded_dummy_account,
};
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT, DEFAULT_ACCOUNT_ADDR,
    PRODUCTION_RUN_GENESIS_REQUEST,
};
use casper_types::{
    account::AccountHash, runtime_args, ContractHash, ContractPackageHash, Key, RuntimeArgs, U256,
};
use cowl_vesting::{
    constants::{
        ADMIN_LIST, ARG_COWL_CEP18_CONTRACT_PACKAGE, ARG_ENABLE_MINT_BURN, ARG_EVENTS_MODE,
        ARG_NAME, ARG_RECIPIENT, ARG_TOTAL_SUPPLY, ARG_TRANSFER_FILTER_CONTRACT_PACKAGE,
        ARG_TRANSFER_FILTER_METHOD, ARG_VESTING_TYPE, ENTRY_POINT_CHANGE_SECURITY,
        ENTRY_POINT_CHECK_VESTING_TRANSFER, ENTRY_POINT_SET_MODALITIES, ENTRY_POINT_TRANSFER,
        ENTRY_POINT_VESTING_INFO, ENTRY_POINT_VESTING_STATUS, NONE_LIST,
    },
    enums::{EventsMode, VestingType},
};
use std::collections::HashMap;

use super::constants::{
    ACCOUNT_LIQUIDITY, ARG_DECIMALS, ARG_SYMBOL, COWL_CEP18_TEST_TOKEN_CONTRACT_PACKAGE_NAME,
    COWL_CEP_18_CONTRACT_WASM, COWL_CEP_18_TOKEN_DECIMALS, COWL_CEP_18_TOKEN_NAME,
    COWL_CEP_18_TOKEN_SYMBOL, VESTING_CONTRACT_KEY_NAME, VESTING_CONTRACT_PACKAGE_HASH_KEY_NAME,
};

#[derive(Clone)]
pub struct TestContext {
    pub cowl_vesting_contract_hash: ContractHash,
    pub cowl_cep18_token_contract_hash: ContractHash,
    pub cowl_cep18_token_package_hash: ContractPackageHash,
    pub test_accounts: HashMap<[u8; 32], AccountHash>,
}

impl Drop for TestContext {
    fn drop(&mut self) {}
}

fn default_args() -> RuntimeArgs {
    runtime_args! {
        ARG_NAME => VESTING_TEST_NAME,
        ARG_EVENTS_MODE => EventsMode::CES as u8,
    }
}

pub fn setup() -> (InMemoryWasmTestBuilder, TestContext) {
    setup_with_args(default_args(), None)
}

pub(crate) fn setup_with_args(
    mut install_args: RuntimeArgs,
    test_accounts: Option<HashMap<[u8; 32], AccountHash>>,
) -> (InMemoryWasmTestBuilder, TestContext) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST);

    let mut test_accounts = test_accounts.unwrap_or_default();

    test_accounts
        .entry(ACCOUNT_USER_1)
        .or_insert_with(|| create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1)));
    test_accounts
        .entry(ACCOUNT_USER_2)
        .or_insert_with(|| create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_2)));

    // Install cep18 token first without filter and without minter
    let install_cowl_cep18_token_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        COWL_CEP_18_CONTRACT_WASM,
        runtime_args! {
            ARG_NAME => COWL_CEP_18_TOKEN_NAME,
            ARG_SYMBOL => COWL_CEP_18_TOKEN_SYMBOL,
            ARG_DECIMALS => COWL_CEP_18_TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::zero(), // No supply before mint from vesting contract
            ARG_EVENTS_MODE => EventsMode::CES as u8,
            ARG_ENABLE_MINT_BURN => true as u8
        },
    )
    .build();

    builder
        .exec(install_cowl_cep18_token_request)
        .expect_success()
        .commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let cowl_cep18_token_contract_hash = account
        .named_keys()
        .get(COWL_CEP18_TEST_TOKEN_CONTRACT_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let cowl_cep18_token_package_hash = account
        .named_keys()
        .get(COWL_CEP18_TEST_TOKEN_CONTRACT_PACKAGE_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have package hash");

    // Install vesting contract with token package as install ARG
    let _ = install_args.insert(
        ARG_COWL_CEP18_CONTRACT_PACKAGE.to_string(),
        Key::from(cowl_cep18_token_package_hash),
    );

    let accounts = vec![
        (VestingType::Liquidity.to_string(), ACCOUNT_LIQUIDITY),
        (VestingType::Contributor.to_string(), ACCOUNT_CONTRIBUTOR),
        (VestingType::Development.to_string(), ACCOUNT_DEVELOPMENT),
        (VestingType::Treasury.to_string(), ACCOUNT_TREASURY),
        (VestingType::Community.to_string(), ACCOUNT_COMMUNITY),
        (VestingType::Staking.to_string(), ACCOUNT_STACKING),
    ];

    // Iterate over the accounts and insert into install_args
    for (address_key, account) in accounts {
        let account_key = create_funded_dummy_account(&mut builder, Some(account));
        let _ = install_args.insert(address_key.to_string(), Key::from(account_key));
        test_accounts.insert(account, account_key);
    }

    // Install vesting contract with token
    let install_request_contract = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        VESTING_CONTRACT_WASM,
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

    let cowl_vesting_contract_hash = account
        .named_keys()
        .get(VESTING_CONTRACT_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let cowl_vesting_contract_package_hash = account
        .named_keys()
        .get(VESTING_CONTRACT_PACKAGE_HASH_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have package hash");

    // Check token package has well been installed in vesting contract
    let actual_cowl_cep18_token_package_hash: ContractPackageHash = builder
        .get_value::<ContractPackageHash>(
            cowl_vesting_contract_hash,
            ARG_COWL_CEP18_CONTRACT_PACKAGE,
        );

    assert_eq!(
        actual_cowl_cep18_token_package_hash,
        cowl_cep18_token_package_hash
    );

    // Check vesting contract as filter contract has been updated in token contract
    let actual_transfer_contract_package: ContractPackageHash = builder
        .get_value::<Option<ContractPackageHash>>(
            cowl_cep18_token_contract_hash,
            ARG_TRANSFER_FILTER_CONTRACT_PACKAGE,
        )
        .unwrap();

    assert_eq!(
        actual_transfer_contract_package,
        cowl_vesting_contract_package_hash
    );

    // Check filter method has been updated in token contract
    let actual_transfer_method: String = builder
        .get_value::<Option<String>>(cowl_cep18_token_contract_hash, ARG_TRANSFER_FILTER_METHOD)
        .unwrap();

    assert_eq!(actual_transfer_method, ENTRY_POINT_CHECK_VESTING_TRANSFER);

    let test_context = TestContext {
        cowl_vesting_contract_hash,
        cowl_cep18_token_contract_hash,
        cowl_cep18_token_package_hash,
        test_accounts,
    };

    (builder, test_context)
}

pub fn cowl_vesting_set_modalities<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    cowl_vesting: &'a ContractHash,
    owner: &'a AccountHash,
    events_mode: Option<EventsMode>,
) -> &'a mut InMemoryWasmTestBuilder {
    let mut args = runtime_args! {};
    if let Some(events_mode) = events_mode {
        let _ = args.insert(ARG_EVENTS_MODE, events_mode as u8);
    };
    let set_modalities_request = ExecuteRequestBuilder::contract_call_by_hash(
        *owner,
        *cowl_vesting,
        ENTRY_POINT_SET_MODALITIES,
        args,
    )
    .build();
    builder.exec(set_modalities_request)
}

pub fn cowl_vesting_vesting_status<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    cowl_vesting: &'a ContractHash,
    sender: &AccountHash,
    vesting_type: VestingType,
    block_time: Option<u64>,
) -> &'a mut InMemoryWasmTestBuilder {
    let args = runtime_args! {
        ARG_VESTING_TYPE => vesting_type.to_string()
    };

    let mut vesting_status_request = ExecuteRequestBuilder::contract_call_by_hash(
        *sender,
        *cowl_vesting,
        ENTRY_POINT_VESTING_STATUS,
        args,
    );

    if let Some(block_time) = block_time {
        vesting_status_request = vesting_status_request.with_block_time(block_time * 1000)
    }

    builder.exec(vesting_status_request.build())
}

pub fn cowl_vesting_vesting_info<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    cowl_vesting: &'a ContractHash,
    sender: &AccountHash,
    vesting_type: VestingType,
) -> &'a mut InMemoryWasmTestBuilder {
    let args = runtime_args! {
        ARG_VESTING_TYPE => vesting_type.to_string()
    };

    let vesting_info_request = ExecuteRequestBuilder::contract_call_by_hash(
        *sender,
        *cowl_vesting,
        ENTRY_POINT_VESTING_INFO,
        args,
    )
    .build();
    builder.exec(vesting_info_request)
}

pub fn cowl_cep18_token_transfer<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    cowl_cep18_token_contract_hash: &'a ContractHash,
    sender: &AccountHash,
    transfer_amount: U256,
    recipient: &AccountHash,
    block_time: Option<u64>,
) -> &'a mut InMemoryWasmTestBuilder {
    let args = runtime_args! {
        ARG_RECIPIENT => Key::Account(*recipient),
        ARG_AMOUNT => transfer_amount,
    };

    let mut token_transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        *sender,
        *cowl_cep18_token_contract_hash,
        ENTRY_POINT_TRANSFER,
        args,
    );

    if let Some(block_time) = block_time {
        token_transfer_request = token_transfer_request.with_block_time(block_time * 1000)
    }

    builder.exec(token_transfer_request.build())
}

pub struct SecurityLists {
    pub admin_list: Option<Vec<Key>>,
    pub none_list: Option<Vec<Key>>,
}

pub fn cowl_vesting_change_security<'a>(
    builder: &'a mut InMemoryWasmTestBuilder,
    cowl_vesting: &'a ContractHash,
    admin_account: &'a AccountHash,
    security_lists: SecurityLists,
) -> &'a mut InMemoryWasmTestBuilder {
    let SecurityLists {
        admin_list,
        none_list,
    } = security_lists;

    let change_security_request = ExecuteRequestBuilder::contract_call_by_hash(
        *admin_account,
        *cowl_vesting,
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
