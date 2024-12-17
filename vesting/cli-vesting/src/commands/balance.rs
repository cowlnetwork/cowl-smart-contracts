use crate::utils::{
    config::get_key_pair_from_vesting,
    constants::{COWL_CEP_18_COOL_SYMBOL, COWL_CEP_18_TOKEN_SYMBOL},
    format_with_thousands_separator, get_contract_cep18_hash_keys, get_dictionary_item_params,
    keys::get_key_pair_from_key,
    sdk, stored_value_to_parsed_string,
};
use casper_rust_wasm_sdk::{
    helpers::{get_base64_key_from_account_hash, motes_to_cspr},
    types::key::Key,
};
use cowl_vesting::{constants::DICT_BALANCES, enums::VestingType};
use serde_json::to_string;

pub async fn get_balance(
    maybe_vesting_type: Option<VestingType>,
    maybe_key: Option<Key>,
) -> String {
    let dictionary_key = if let Some(vesting_type) = maybe_vesting_type {
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
    } else if let Some(key) = maybe_key.clone() {
        get_base64_key_from_account_hash(&key.clone().into_account().unwrap().to_formatted_string())
            .unwrap_or_else(|err| {
                log::error!(
                    "Failed to retrieve account_hash for {}: {:?}",
                    key.to_formatted_string(),
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
        .query_contract_dict(dictionary_item, None::<&str>, None, None)
        .await;

    // Handle query result and extract stored value
    let stored_value = match balance_result {
        Ok(result) => result.result.stored_value,
        Err(err) => {
            if let Some(vesting_type) = maybe_vesting_type {
                log::warn!(
                    "No {} balance for {}!\n- Account Hash: {}",
                    *COWL_CEP_18_TOKEN_SYMBOL,
                    vesting_type,
                    maybe_key
                        .as_ref()
                        .map(|key| key.to_formatted_string())
                        .unwrap_or_else(|| "Unknown Key".to_string())
                );
            } else if let Some(key) = maybe_key {
                let (vesting_type, key_pair) = get_key_pair_from_key(&key).await;
                if let Some(key_pair) = key_pair {
                    log::warn!(
                        "No {} balance for {}\n\
                        - Private Key: {:?}\n\
                        - Public Key: {}\n\
                        - Account Hash: {}",
                        *COWL_CEP_18_TOKEN_SYMBOL,
                        vesting_type.unwrap_or_default(),
                        key_pair.private_key_base64,
                        key_pair.public_key.to_string(),
                        key_pair.public_key.to_account_hash().to_formatted_string()
                    );
                } else {
                    log::warn!(
                        "No {} balance!\n- Account Hash: {}",
                        *COWL_CEP_18_TOKEN_SYMBOL,
                        key.to_formatted_string()
                    );
                }
            }

            log::debug!("{err}");
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
    log::info!("{} {}", &balance, *COWL_CEP_18_COOL_SYMBOL);
    log::info!(
        "{} {}",
        format_with_thousands_separator(&motes_to_cspr(&balance).unwrap()),
        *COWL_CEP_18_TOKEN_SYMBOL
    );
}
