use super::constants::{
    ACCOUNT_COMMUNITY, ACCOUNT_CONTRIBUTOR, ACCOUNT_DEVELOPMENT, ACCOUNT_LIQUIDITY,
    ACCOUNT_STACKING, ACCOUNT_TREASURY,
};
use casper_engine_test_support::{
    ExecuteRequestBuilder, WasmTestBuilder, DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use casper_event_standard::EVENTS_DICT;
use casper_execution_engine::{
    core::{engine_state::Error as EngineStateError, execution},
    storage::global_state::in_memory::InMemoryGlobalState,
};
use casper_types::{
    account::AccountHash,
    bytesrepr::{Bytes, FromBytes},
    runtime_args,
    system::{
        handle_payment::{ARG_AMOUNT, ARG_TARGET},
        mint::ARG_ID,
    },
    ApiError, CLTyped, Key, PublicKey, RuntimeArgs, SecretKey,
};
use cowl_vesting::enums::VestingType;
use std::fmt::Debug;

pub fn assert_expected_error(actual_error: EngineStateError, error_code: u16, reason: &str) {
    let actual = format!("{actual_error:?}");
    let expected = format!(
        "{:?}",
        EngineStateError::Exec(execution::Error::Revert(ApiError::User(error_code)))
    );
    assert_eq!(
        actual, expected,
        "Error should match {error_code} with reason: {reason}"
    )
}

pub fn get_dictionary_value_from_key<T: CLTyped + FromBytes>(
    builder: &WasmTestBuilder<InMemoryGlobalState>,
    contract_key: &Key,
    dictionary_name: &str,
    dictionary_key: &str,
) -> T {
    let seed_uref = *builder
        .query(None, *contract_key, &[])
        .expect("must have contract")
        .as_contract()
        .expect("must convert contract")
        .named_keys()
        .get(dictionary_name)
        .expect("must have key")
        .as_uref()
        .expect("must convert to seed uref");

    builder
        .query_dictionary_item(None, seed_uref, dictionary_key)
        .expect("should have dictionary value")
        .as_cl_value()
        .expect("T should be CLValue")
        .to_owned()
        .into_t()
        .unwrap()
}

pub fn get_event<T: FromBytes + CLTyped + Debug>(
    builder: &WasmTestBuilder<InMemoryGlobalState>,
    contract_key: &Key,
    index: u32,
) -> T {
    let bytes: Bytes =
        get_dictionary_value_from_key(builder, contract_key, EVENTS_DICT, &index.to_string());
    let (event, bytes) = T::from_bytes(&bytes).unwrap();
    assert!(bytes.is_empty());
    event
}

// Creates a dummy account and transfer funds to it
pub fn create_funded_dummy_account(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    account_string: Option<[u8; 32]>,
) -> AccountHash {
    let (_, account_public_key) = create_dummy_key_pair(account_string.unwrap_or([7u8; 32]));
    let account = account_public_key.to_account_hash();
    fund_account(builder, account);
    account
}

pub fn create_dummy_key_pair(account_string: [u8; 32]) -> (SecretKey, PublicKey) {
    let secret_key =
        SecretKey::ed25519_from_bytes(account_string).expect("failed to create secret key");
    let public_key = PublicKey::from(&secret_key);
    (secret_key, public_key)
}

pub fn fund_account(builder: &mut WasmTestBuilder<InMemoryGlobalState>, account: AccountHash) {
    let transfer = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            ARG_AMOUNT => DEFAULT_ACCOUNT_INITIAL_BALANCE / 10_u64,
            ARG_TARGET => account,
            ARG_ID => Option::<u64>::None,
        },
    )
    .build();
    builder.exec(transfer).expect_success().commit();
}

pub fn get_account_for_vesting(vesting_type: VestingType) -> [u8; 32] {
    match vesting_type {
        VestingType::Liquidity => ACCOUNT_LIQUIDITY,
        VestingType::Contributor => ACCOUNT_CONTRIBUTOR,
        VestingType::Development => ACCOUNT_DEVELOPMENT,
        VestingType::Treasury => ACCOUNT_TREASURY,
        VestingType::Community => ACCOUNT_COMMUNITY,
        VestingType::Staking => ACCOUNT_STACKING,
        _ => unimplemented!(),
    }
}
