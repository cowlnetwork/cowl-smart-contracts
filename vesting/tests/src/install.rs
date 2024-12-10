use crate::utility::{
    constants::{
        ACCOUNT_COMMUNITY, ACCOUNT_CONTRIBUTOR, ACCOUNT_DEVELOPMENT, ACCOUNT_LIQUIDITY,
        ACCOUNT_STACKING, ACCOUNT_TREASURY, VESTING_CONTRACT_VERSION, VESTING_CONTRACT_WASM,
        VESTING_TEST_NAME,
    },
    installer_request_builders::{setup, TestContext},
    support::{get_account_for_vesting, get_dictionary_value_from_key},
};
use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{runtime_args, ContractPackageHash, Key, RuntimeArgs, U256};
use cowl_vesting::{
    constants::{
        ARG_CONTRACT_HASH, ARG_COWL_CEP18_CONTRACT_PACKAGE, ARG_EVENTS_MODE, ARG_INSTALLER,
        ARG_NAME, ARG_PACKAGE_HASH, ARG_TOTAL_SUPPLY, ARG_TRANSFER_FILTER_CONTRACT_PACKAGE,
        ARG_TRANSFER_FILTER_METHOD, DICT_ADDRESSES, DICT_SECURITY_BADGES, DICT_START_TIME,
        DICT_VESTING_AMOUNT, DICT_VESTING_STATUS,
    },
    enums::{EventsMode, VestingType, VESTING_INFO},
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

    assert!(
        named_keys.contains_key(ARG_TOTAL_SUPPLY),
        "{:?}",
        named_keys
    );
    // dbg!(named_keys);

    let total_supply = builder
        .query(
            None,
            cowl_cep18_token_contract_hash.into(),
            &[ARG_TOTAL_SUPPLY.to_string()],
        )
        .unwrap()
        .as_cl_value()
        .unwrap()
        .to_owned()
        .into_t::<U256>()
        .unwrap_or_default();

    dbg!(total_supply);

    assert!(total_supply > U256::zero());

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
        total_vested_amount, total_supply,
        "The total vested amount does not match the token total supply!"
    );
}

#[test]
fn should_prevent_reinstall_contract() {
    let (
        mut builder,
        TestContext {
            cowl_vesting_contract_hash,
            ref test_accounts,
            ..
        },
    ) = setup();

    let version_key = *builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account")
        .named_keys()
        .get(VESTING_CONTRACT_VERSION)
        .expect("version uref should exist");

    let version = builder
        .query(None, version_key, &[])
        .expect("should be stored value.")
        .as_cl_value()
        .expect("should be cl value.")
        .clone()
        .into_t::<u32>()
        .expect("should be u32.");

    dbg!(version);

    let vesting_contract = builder
        .get_contract(cowl_vesting_contract_hash)
        .expect("should have vesting contract");
    let named_keys = vesting_contract.named_keys();
    dbg!(named_keys);

    let cowl_cep18_token_package_hash: ContractPackageHash = builder
        .get_value::<ContractPackageHash>(
            cowl_vesting_contract_hash,
            ARG_COWL_CEP18_CONTRACT_PACKAGE,
        );

    let mut install_args = runtime_args!(
        ARG_NAME => VESTING_TEST_NAME,
        ARG_EVENTS_MODE => EventsMode::CES as u8,
        ARG_COWL_CEP18_CONTRACT_PACKAGE =>
        Key::from(cowl_cep18_token_package_hash),
    );

    let accounts = vec![
        (
            VestingType::Liquidity.to_string(),
            Key::from(*test_accounts.get(&ACCOUNT_LIQUIDITY).unwrap()),
        ),
        (
            VestingType::Contributor.to_string(),
            Key::from(*test_accounts.get(&ACCOUNT_CONTRIBUTOR).unwrap()),
        ),
        (
            VestingType::Development.to_string(),
            Key::from(*test_accounts.get(&ACCOUNT_DEVELOPMENT).unwrap()),
        ),
        (
            VestingType::Treasury.to_string(),
            Key::from(*test_accounts.get(&ACCOUNT_TREASURY).unwrap()),
        ),
        (
            VestingType::Community.to_string(),
            Key::from(*test_accounts.get(&ACCOUNT_COMMUNITY).unwrap()),
        ),
        (
            VestingType::Staking.to_string(),
            Key::from(*test_accounts.get(&ACCOUNT_STACKING).unwrap()),
        ),
    ];

    for (address_key, account_key) in accounts {
        let _ = install_args.insert(address_key.to_string(), account_key);
    }

    // Install vesting contract with token
    let reinstall_request_contract =
        ExecuteRequestBuilder::standard(*DEFAULT_ACCOUNT_ADDR, VESTING_CONTRACT_WASM, install_args)
            .build();

    builder
        .exec(reinstall_request_contract)
        .expect_success()
        .commit();

    let vesting_contract = builder
        .get_contract(cowl_vesting_contract_hash)
        .expect("should have vesting contract");
    let new_named_keys = vesting_contract.named_keys();
    dbg!(new_named_keys);

    assert_eq!(named_keys, new_named_keys)
}
