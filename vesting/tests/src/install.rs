use crate::utility::{
    installer_request_builders::{setup, TestContext},
    support::{get_account_for_vesting, get_dictionary_value_from_key},
};
use casper_types::{Key, U256};
use cowl_vesting::{
    constants::{
        ARG_CONTRACT_HASH, ARG_COWL_CEP18_CONTRACT_PACKAGE, ARG_EVENTS_MODE, ARG_INSTALLER,
        ARG_NAME, ARG_PACKAGE_HASH, ARG_TRANSFER_FILTER_CONTRACT_PACKAGE,
        ARG_TRANSFER_FILTER_METHOD, COWL_CEP_18_TOKEN_TOTAL_SUPPLY, DICT_ADDRESSES,
        DICT_SECURITY_BADGES, DICT_START_TIME, DICT_VESTING_AMOUNT, DICT_VESTING_STATUS,
    },
    enums::VESTING_INFO,
    vesting::VestingStatus,
};

#[test]
fn should_install_contract() {
    let (
        builder,
        TestContext {
            cowl_vesting_contract_hash,
            cowl_cep18_token_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup();
    let vesting_contract = builder
        .get_contract(cowl_vesting_contract_hash)
        .expect("should have vesting contract");
    let cowl_cep18_token_contract = builder
        .get_contract(cowl_cep18_token_contract_hash)
        .expect("should have cowl cep18 token contract");

    let named_keys = cowl_cep18_token_contract.named_keys();
    // dbg!(named_keys);

    assert!(
        named_keys.contains_key(ARG_TRANSFER_FILTER_METHOD),
        "{:?}",
        named_keys
    );

    assert!(
        named_keys.contains_key(ARG_TRANSFER_FILTER_CONTRACT_PACKAGE),
        "{:?}",
        named_keys
    );

    let named_keys = vesting_contract.named_keys();
    // dbg!(named_keys);

    assert!(
        named_keys.contains_key(ARG_CONTRACT_HASH),
        "{:?}",
        named_keys
    );
    assert!(
        named_keys.contains_key(ARG_PACKAGE_HASH),
        "{:?}",
        named_keys
    );
    assert!(
        named_keys.contains_key(DICT_SECURITY_BADGES),
        "{:?}",
        named_keys
    );
    assert!(named_keys.contains_key(ARG_NAME), "{:?}", named_keys);
    assert!(named_keys.contains_key(ARG_INSTALLER), "{:?}", named_keys);
    assert!(named_keys.contains_key(ARG_EVENTS_MODE), "{:?}", named_keys);
    assert!(
        named_keys.contains_key(ARG_COWL_CEP18_CONTRACT_PACKAGE),
        "{:?}",
        named_keys
    );

    // Check all vesting addresses in dictionary
    for vesting_info in VESTING_INFO.iter() {
        let actual_address = *get_dictionary_value_from_key::<Key>(
            &builder,
            &Key::from(cowl_vesting_contract_hash),
            DICT_ADDRESSES,
            &vesting_info.vesting_type.to_string(),
        )
        .as_account()
        .unwrap();

        let expected_account = get_account_for_vesting(vesting_info.vesting_type);

        // Retrieve the expected address from the test_accounts map
        let expected_address = *test_accounts.get(&expected_account).unwrap();

        // Assert that the actual address matches the expected address
        assert_eq!(
            actual_address, expected_address,
            "Mismatch for {:?}",
            vesting_info.vesting_type
        );

        let vesting_status: VestingStatus = get_dictionary_value_from_key(
            &builder,
            &Key::from(cowl_vesting_contract_hash),
            DICT_VESTING_STATUS,
            &vesting_info.vesting_type.to_string().to_owned(),
        );
        assert_eq!(vesting_status.vesting_type, vesting_info.vesting_type);

        dbg!(vesting_status);
    }

    let total_vested_amount: U256 = VESTING_INFO
        .iter()
        .map(|vesting_info| {
            let actual_amount: U256 = get_dictionary_value_from_key::<U256>(
                &builder,
                &Key::from(cowl_vesting_contract_hash),
                DICT_VESTING_AMOUNT,
                &vesting_info.vesting_type.to_string(),
            );

            // Perform the check for the start time as well
            let actual_start_time: u64 = get_dictionary_value_from_key::<u64>(
                &builder,
                &Key::from(cowl_vesting_contract_hash),
                DICT_START_TIME,
                &vesting_info.vesting_type.to_string(),
            );

            // Assert the start time for the current vesting info
            assert_eq!(
                actual_start_time, 0_u64,
                "Mismatch for {:?}",
                vesting_info.vesting_type
            );

            actual_amount
        })
        .sum();

    assert_eq!(
        total_vested_amount,
        COWL_CEP_18_TOKEN_TOTAL_SUPPLY.into(),
        "The total vested amount does not match the token total supply!"
    );
}
