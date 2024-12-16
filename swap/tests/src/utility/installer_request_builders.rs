use crate::utility::{
    constants::{ACCOUNT_USER_1, ACCOUNT_USER_2, SWAP_CONTRACT_WASM, SWAP_TEST_NAME},
    support::create_funded_dummy_account,
};
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    PRODUCTION_RUN_GENESIS_REQUEST,
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

use super::constants::{SWAP_CONTRACT_KEY_NAME, SWAP_CONTRACT_PACKAGE_HASH_KEY_NAME};

#[derive(Clone)]
pub(crate) struct TestContext {
    pub(crate) cowl_swap_contract_hash: ContractHash,
    pub(crate) cowl_swap_contract_package_hash: ContractPackageHash,
    // pub(crate) cowl_cep18_token_contract_hash: ContractHash,
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
    setup_with_args(default_args(), None)
}

pub fn setup_with_args(
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

    // // Install cep18 token first without filter and without minter
    // let install_cowl_cep18_token_request = ExecuteRequestBuilder::standard(
    //     *DEFAULT_ACCOUNT_ADDR,
    //     COWL_CEP_18_CONTRACT_WASM,
    //     runtime_args! {
    //         ARG_NAME => COWL_CEP_18_TOKEN_NAME,
    //         ARG_SYMBOL => COWL_CEP_18_TOKEN_SYMBOL,
    //         ARG_DECIMALS => COWL_CEP_18_TOKEN_DECIMALS,
    //         ARG_TOTAL_SUPPLY => U256::zero(), // No supply before mint from SWAP contract
    //         ARG_EVENTS_MODE => EventsMode::CES as u8,
    //         ARG_ENABLE_MINT_BURN => true as u8
    //     },
    // )
    // .build();

    // builder
    //     .exec(install_cowl_cep18_token_request)
    //     .expect_success()
    //     .commit();

    // let account = builder
    //     .get_account(*DEFAULT_ACCOUNT_ADDR)
    //     .expect("should have account");

    // let cowl_cep18_token_contract_hash = account
    //     .named_keys()
    //     .get(COWL_CEP18_TEST_TOKEN_CONTRACT_NAME)
    //     .and_then(|key| key.into_hash())
    //     .map(ContractHash::new)
    //     .expect("should have contract hash");

    // let cowl_cep18_token_package_hash = account
    //     .named_keys()
    //     .get(COWL_CEP18_TEST_TOKEN_CONTRACT_PACKAGE_NAME)
    //     .and_then(|key| key.into_hash())
    //     .map(ContractPackageHash::new)
    //     .expect("should have package hash");

    // // Install SWAP contract with token package as install ARG
    // let _ = install_args.insert(
    //     ARG_COWL_CEP18_CONTRACT_PACKAGE.to_string(),
    //     Key::from(cowl_cep18_token_package_hash),
    // );

    // let accounts = vec![
    //     (SWAPType::Liquidity.to_string(), ACCOUNT_LIQUIDITY),
    //     (SWAPType::Contributor.to_string(), ACCOUNT_CONTRIBUTOR),
    //     (SWAPType::Development.to_string(), ACCOUNT_DEVELOPMENT),
    //     (SWAPType::Treasury.to_string(), ACCOUNT_TREASURY),
    //     (SWAPType::Community.to_string(), ACCOUNT_COMMUNITY),
    //     (SWAPType::Staking.to_string(), ACCOUNT_STACKING),
    // ];

    // // Iterate over the accounts and insert into install_args
    // for (address_key, account) in accounts {
    //     let account_key = create_funded_dummy_account(&mut builder, Some(account));
    //     let _ = install_args.insert(address_key.to_string(), Key::from(account_key));
    //     test_accounts.insert(account, account_key);
    // }

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

    // // Check token package has well been installed in SWAP contract
    // let actual_cowl_cep18_token_package_hash: ContractPackageHash = builder
    //     .get_value::<ContractPackageHash>(cowl_swap_contract_hash, ARG_COWL_CEP18_CONTRACT_PACKAGE);

    // assert_eq!(
    //     actual_cowl_cep18_token_package_hash,
    //     cowl_cep18_token_package_hash
    // );

    // // Check SWAP contract as filter contract has been updated in token contract
    // let actual_transfer_contract_package: ContractPackageHash = builder
    //     .get_value::<Option<ContractPackageHash>>(
    //         cowl_cep18_token_contract_hash,
    //         ARG_TRANSFER_FILTER_CONTRACT_PACKAGE,
    //     )
    //     .unwrap();

    // assert_eq!(
    //     actual_transfer_contract_package,
    //     cowl_swap_contract_package_hash
    // );

    // // Check filter method has been updated in token contract
    // let actual_transfer_method: String = builder
    //     .get_value::<Option<String>>(cowl_cep18_token_contract_hash, ARG_TRANSFER_FILTER_METHOD)
    //     .unwrap();

    // assert_eq!(actual_transfer_method, ENTRY_POINT_CHECK_SWAP_TRANSFER);

    let test_context = TestContext {
        cowl_swap_contract_hash,
        cowl_swap_contract_package_hash,
        // cowl_cep18_token_contract_hash,
        test_accounts,
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

// pub fn cowl_cep18_token_transfer<'a>(
//     builder: &'a mut InMemoryWasmTestBuilder,
//     cowl_cep18_token_contract_hash: &'a ContractHash,
//     sender: &AccountHash,
//     transfer_amount: U256,
//     recipient: &AccountHash,
//     block_time: Option<u64>,
// ) -> &'a mut InMemoryWasmTestBuilder {
//     let args = runtime_args! {
//         ARG_RECIPIENT => Key::Account(*recipient),
//         ARG_AMOUNT => transfer_amount,
//     };

//     let mut token_transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
//         *sender,
//         *cowl_cep18_token_contract_hash,
//         ENTRY_POINT_TRANSFER,
//         args,
//     );

//     if let Some(block_time) = block_time {
//         token_transfer_request = token_transfer_request.with_block_time(block_time)
//     }

//     builder.exec(token_transfer_request.build())
// }

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
