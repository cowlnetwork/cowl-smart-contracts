use crate::utils::{
    constants::{COWL_CEP_18_TOKEN_SYMBOL, DICT_ALLOWANCES},
    get_contract_cep18_hash_keys, get_dictionary_item_params, sdk, stored_value_to_parsed_string,
};
use casper_rust_wasm_sdk::{
    helpers::{make_dictionary_item_key, motes_to_cspr},
    types::key::Key,
};
use serde_json::to_string;

pub async fn get_allowance(owner: &Key, spender: &Key) -> String {
    // Retrieve contract token hash and package hash
    let (cowl_cep18_token_contract_hash, _) = match get_contract_cep18_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => {
            log::error!("Failed to retrieve contract token hash and package hash.");
            return 0.to_string();
        }
    };

    let dictionary_key = make_dictionary_item_key(owner.clone(), &spender);

    // Get the dictionary item parameters for the allowance
    let dictionary_item = get_dictionary_item_params(
        &cowl_cep18_token_contract_hash.to_string(),
        DICT_ALLOWANCES,
        &dictionary_key,
    );

    // Query the contract dictionary for the allowance
    let allowance_result = sdk()
        .query_contract_dict("", dictionary_item, None, None)
        .await;

    // Handle query result and extract stored value
    let stored_value = match allowance_result {
        Ok(result) => result.result.stored_value,
        Err(err) => {
            log::debug!("Failed to query allowance from the contract.{}", err);
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
pub async fn print_allowance(owner: &Key, spender: &Key) {
    let allowance = get_allowance(owner, spender).await;

    log::info!("Allowance for {}", spender.to_formatted_string());
    log::info!(
        "{} {}",
        motes_to_cspr(&allowance).unwrap(),
        *COWL_CEP_18_TOKEN_SYMBOL
    );
}
