use crate::utility::{
    constants::ACCOUNT_USER_1,
    installer_request_builders::{
        cowl_cep18_token_transfer, cowl_vesting_vesting_status, setup, TestContext,
    },
    support::{get_account_for_vesting, get_dictionary_value_from_key},
};
use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use casper_types::{Key, U256};
use cowl_vesting::{
    constants::{DICT_TRANSFERRED_AMOUNT, DICT_VESTING_STATUS, DURATION_COMMUNITY_VESTING},
    enums::VestingType,
    vesting::VestingStatus,
};

#[test]
fn should_not_allow_transfer_for_non_vesting_address_at_zero_time() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            cowl_cep18_token_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Community;

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let sender = *test_accounts
        .get(&get_account_for_vesting(vesting_type))
        .unwrap();

    let transfer_amount = U256::one();

    cowl_cep18_token_transfer(
        &mut builder,
        &cowl_cep18_token_contract_hash,
        &sender,
        transfer_amount,
        &account_user_1,
        None,
    )
    .expect_failure();

    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &vesting_type.to_string().to_owned(),
    );

    assert_eq!(vesting_status.vested_amount, U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_COMMUNITY_VESTING.unwrap()
    );
    dbg!(vesting_status);
}

#[test]
fn should_allow_transfer_for_non_vesting_address_at_time_one() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            cowl_cep18_token_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Community;

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let sender = *test_accounts
        .get(&get_account_for_vesting(vesting_type))
        .unwrap();

    let transfer_amount = U256::one();

    cowl_cep18_token_transfer(
        &mut builder,
        &cowl_cep18_token_contract_hash,
        &sender,
        transfer_amount,
        &account_user_1,
        Some(1_u64),
    )
    .expect_success()
    .commit();

    let actual_transfered_amount: U256 = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_TRANSFERRED_AMOUNT,
        &vesting_type.to_string(),
    );
    assert_eq!(actual_transfered_amount, transfer_amount);

    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &vesting_type.to_string().to_owned(),
    );

    assert!(vesting_status.vested_amount > U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_COMMUNITY_VESTING.unwrap()
    );
    dbg!(vesting_status);
}

#[test]
fn should_not_allow_transfer_for_more_than_vested_amout_at_time_one() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            cowl_cep18_token_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Community;

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let sender = *test_accounts
        .get(&get_account_for_vesting(vesting_type))
        .unwrap();

    let transfer_amount = U256::from(13);

    cowl_cep18_token_transfer(
        &mut builder,
        &cowl_cep18_token_contract_hash,
        &sender,
        transfer_amount,
        &account_user_1,
        Some(1_u64),
    )
    .expect_failure()
    .commit();

    cowl_vesting_vesting_status(
        &mut builder,
        &cowl_vesting_contract_hash,
        &DEFAULT_ACCOUNT_ADDR,
        vesting_type,
        Some(1_u64),
    )
    .expect_success()
    .commit();

    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &vesting_type.to_string().to_owned(),
    );

    assert_eq!(vesting_status.vested_amount, transfer_amount - U256::one());
    assert_eq!(vesting_status.released_amount, U256::zero());
    assert_eq!(
        vesting_status.available_for_release_amount,
        transfer_amount - U256::one()
    );
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_COMMUNITY_VESTING.unwrap()
    );
    dbg!(vesting_status);
}

#[test]
fn should_allow_full_transfer_for_non_vesting_address_at_time_one() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            cowl_cep18_token_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Community;

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let sender = *test_accounts
        .get(&get_account_for_vesting(vesting_type))
        .unwrap();

    let transfer_amount = U256::from(12);

    cowl_cep18_token_transfer(
        &mut builder,
        &cowl_cep18_token_contract_hash,
        &sender,
        transfer_amount,
        &account_user_1,
        Some(1_u64),
    )
    .expect_success()
    .commit();

    let actual_transfered_amount: U256 = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_TRANSFERRED_AMOUNT,
        &vesting_type.to_string(),
    );
    assert_eq!(actual_transfered_amount, transfer_amount);

    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &vesting_type.to_string().to_owned(),
    );

    assert!(vesting_status.vested_amount > U256::zero());
    assert_eq!(vesting_status.released_amount, transfer_amount);
    assert_eq!(vesting_status.available_for_release_amount, U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_COMMUNITY_VESTING.unwrap()
    );
    dbg!(vesting_status);
}

#[test]
fn should_allow_half_transfer_for_non_vesting_address_at_half_time() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            cowl_cep18_token_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Community;

    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &vesting_type.to_string().to_owned(),
    );

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let sender = *test_accounts
        .get(&get_account_for_vesting(vesting_type))
        .unwrap();

    let transfer_amount = vesting_status.total_amount / 2;
    let test_duration = DURATION_COMMUNITY_VESTING.map(|d| (d.whole_seconds() / 2) as u64);

    cowl_cep18_token_transfer(
        &mut builder,
        &cowl_cep18_token_contract_hash,
        &sender,
        transfer_amount,
        &account_user_1,
        test_duration,
    )
    .expect_success()
    .commit();

    let actual_transfered_amount: U256 = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_TRANSFERRED_AMOUNT,
        &vesting_type.to_string(),
    );
    assert_eq!(actual_transfered_amount, transfer_amount);

    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &vesting_type.to_string().to_owned(),
    );

    assert!(vesting_status.vested_amount > U256::zero());
    assert_eq!(vesting_status.released_amount, transfer_amount);
    assert_eq!(vesting_status.available_for_release_amount, U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_COMMUNITY_VESTING.unwrap()
    );
    dbg!(vesting_status);
}

#[test]
fn should_allow_full_transfer_for_non_vesting_address_at_full_time() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            cowl_cep18_token_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Community;

    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &vesting_type.to_string().to_owned(),
    );

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let sender = *test_accounts
        .get(&get_account_for_vesting(vesting_type))
        .unwrap();

    let transfer_amount = vesting_status.total_amount;
    let test_duration = DURATION_COMMUNITY_VESTING.map(|d| (d.whole_seconds()) as u64);

    cowl_cep18_token_transfer(
        &mut builder,
        &cowl_cep18_token_contract_hash,
        &sender,
        transfer_amount,
        &account_user_1,
        test_duration,
    )
    .expect_success()
    .commit();

    let actual_transfered_amount: U256 = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_TRANSFERRED_AMOUNT,
        &vesting_type.to_string(),
    );
    assert_eq!(actual_transfered_amount, transfer_amount);

    let vesting_status: VestingStatus = get_dictionary_value_from_key(
        &builder,
        &Key::from(cowl_vesting_contract_hash),
        DICT_VESTING_STATUS,
        &vesting_type.to_string().to_owned(),
    );

    assert!(vesting_status.vested_amount > U256::zero());
    assert_eq!(vesting_status.released_amount, transfer_amount);
    assert_eq!(vesting_status.available_for_release_amount, U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(vesting_status.total_amount, vesting_status.vested_amount);
    assert!(vesting_status.is_fully_vested);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_COMMUNITY_VESTING.unwrap()
    );
    dbg!(vesting_status);
}