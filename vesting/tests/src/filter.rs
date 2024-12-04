use crate::utility::{
    constants::ACCOUNT_USER_1,
    installer_request_builders::{cowl_cep18_token_transfer, setup, TestContext},
    support::{get_account_for_vesting, get_dictionary_value_from_key},
};
use casper_types::{Key, U256};
use cowl_vesting::{
    constants::{DICT_TRANSFERRED_AMOUNT, DICT_VESTING_STATUS, DURATION_CONTRIBUTOR_VESTING},
    enums::VestingType,
    vesting::VestingStatus,
};

#[test]
fn should_allow_transfer_for_non_vesting_address_at_zero_time() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            cowl_cep18_token_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let vesting_type = VestingType::Contributor;

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

    assert_eq!(vesting_status.vested_amount, U256::zero());
    assert_eq!(vesting_status.vesting_type, vesting_type);
    assert_eq!(
        vesting_status.vesting_duration,
        DURATION_CONTRIBUTOR_VESTING.unwrap()
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

    let vesting_type = VestingType::Contributor;

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
        DURATION_CONTRIBUTOR_VESTING.unwrap()
    );
    dbg!(vesting_status);
}
