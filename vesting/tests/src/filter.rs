use crate::utility::{
    constants::ACCOUNT_USER_1,
    installer_request_builders::{cowl_cep18_token_transfer, setup, TestContext},
    support::{get_account_for_vesting, get_dictionary_value_from_key},
};
use casper_types::{Key, U256};
use cowl_vesting::{constants::DICT_TRANSFERRED_AMOUNT, enums::VestingType};

#[test]
fn should_allow_transfer_for_non_vesting_address() {
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
    let liquidity_address = *test_accounts
        .get(&get_account_for_vesting(vesting_type))
        .unwrap();

    let transfer_amount = U256::one();

    cowl_cep18_token_transfer(
        &mut builder,
        &cowl_cep18_token_contract_hash,
        &liquidity_address,
        transfer_amount,
        &account_user_1,
        Some(0_u64),
    )
    .expect_failure();

    cowl_cep18_token_transfer(
        &mut builder,
        &cowl_cep18_token_contract_hash,
        &liquidity_address,
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
}
