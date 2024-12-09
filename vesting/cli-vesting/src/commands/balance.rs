use crate::utils::{
    config::get_key_pair_from_vesting, constants::COWL_CEP_18_TOKEN_SYMBOL,
    get_contract_cep18_hash_keys, get_dictionary_item_params, sdk, stored_value_to_parsed_string,
};
use casper_rust_wasm_sdk::{
    helpers::{get_base64_key_from_account_hash, motes_to_cspr},
    types::key::Key,
};
use cowl_vesting::{constants::DICT_BALANCES, enums::VestingType};
use serde_json::to_string;

pub async fn get_balance(vesting_type: Option<VestingType>, vesting_key: Option<Key>) -> String {
    let dictionary_key = if let Some(vesting_type) = vesting_type {
        let key_pair = get_key_pair_from_vesting(&vesting_type.to_string())
            .await
            .unwrap();
        get_base64_key_from_account_hash(
            &key_pair.public_key.to_account_hash().to_formatted_string(),
        )
        .unwrap_or_else(|err| {
            log::error!(
                "Failed to retrieve account_hash for {}: {:?}",
                vesting_type,
                err
            );
            0.to_string()
        })
    } else if let Some(vesting_key) = vesting_key {
        get_base64_key_from_account_hash(
            &vesting_key
                .clone()
                .into_account()
                .unwrap()
                .to_formatted_string(),
        )
        .unwrap_or_else(|err| {
            log::error!(
                "Failed to retrieve account_hash for {}: {:?}",
                vesting_key.to_formatted_string(),
                err
            );
            0.to_string()
        })
    } else {
        log::error!("Both vesting_type and vesting_key are missing.");
        return 0.to_string();
    };

    // Retrieve contract token hash and package hash
    let (cowl_cep18_token_contract_hash, _) = match get_contract_cep18_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => {
            log::error!("Failed to retrieve contract token hash and package hash.");
            return 0.to_string();
        }
    };

    // Get the dictionary item parameters for the balance
    let dictionary_item = get_dictionary_item_params(
        &cowl_cep18_token_contract_hash.to_string(),
        DICT_BALANCES,
        &dictionary_key,
    );

    // Query the contract dictionary for the balance
    let balance_result = sdk()
        .query_contract_dict("", dictionary_item, None, None)
        .await;

    // Handle query result and extract stored value
    let stored_value = match balance_result {
        Ok(result) => result.result.stored_value,
        Err(err) => {
            log::debug!("Failed to query balance from the contract.{}", err);
            return 0.to_string();
        }
    };

    let json_string = match to_string(&stored_value) {
        Ok(s) => s,
        Err(_) => {
            log::error!("Failed to serialize stored value into JSON.");
            return 0.to_string();
        }
    };
    stored_value_to_parsed_string(&json_string).unwrap_or_default()
}

pub async fn print_balance(vesting_type: Option<VestingType>, vesting_key: Option<Key>) {
    let balance = get_balance(vesting_type, vesting_key.clone()).await;

    let identifier = match vesting_type {
        Some(vesting_type) => vesting_type.to_string(),
        None => vesting_key
            .map(|key| key.to_formatted_string())
            .unwrap_or_else(|| "Failed to retrieve account hash".to_string()),
    };

    log::info!("Balance for {}", identifier);
    log::info!(
        "{} {}",
        motes_to_cspr(&balance).unwrap(),
        *COWL_CEP_18_TOKEN_SYMBOL
    );
}