use casper_rust_wasm_sdk::rpcs::get_dictionary_item::DictionaryItemInput;
use casper_rust_wasm_sdk::rpcs::query_global_state::{KeyIdentifierInput, QueryGlobalStateParams};
use casper_rust_wasm_sdk::types::deploy_params::dictionary_item_str_params::DictionaryItemStrParams;
use casper_rust_wasm_sdk::types::public_key::PublicKey;
use casper_rust_wasm_sdk::{types::verbosity::Verbosity, SDK};
use config::get_key_pair_from_vesting;
use constants::{
    COWL_CEP18_TOKEN_CONTRACT_HASH_NAME, COWL_CEP18_TOKEN_CONTRACT_PACKAGE_HASH_NAME, INSTALLER,
    NAME_VESTING, RPC_ADDRESS,
};
use cowl_vesting::constants::{PREFIX_CONTRACT_NAME, PREFIX_CONTRACT_PACKAGE_NAME};
use cowl_vesting::vesting::VestingData;
use once_cell::sync::Lazy;
use serde_json::{to_string, Value};
use std::io::Write;
use std::{
    env,
    fs::File,
    io::{self, Read},
    sync::{Arc, Mutex},
};

pub mod config;
pub mod constants;
pub mod keys;

pub static SDK_INSTANCE: Lazy<Mutex<Option<Arc<SDK>>>> = Lazy::new(|| Mutex::new(None));

// Function to retrieve or create the SDK instance
pub fn sdk() -> Arc<SDK> {
    let mut instance = SDK_INSTANCE.lock().unwrap();
    if instance.is_none() {
        let new_sdk = SDK::new(Some(RPC_ADDRESS.to_string()), Some(Verbosity::High));
        *instance = Some(Arc::new(new_sdk));
    }
    instance.clone().unwrap()
}

pub fn read_wasm_file(file_path: &str) -> Result<Vec<u8>, io::Error> {
    let path_buf = env::current_dir()?;
    let mut relative_path_buf = path_buf.clone();
    relative_path_buf.push(file_path);
    let mut file = File::open(relative_path_buf)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn prompt_yes_no(question: &str) -> bool {
    loop {
        log::warn!("{} (y/n): ", question);
        io::stdout().flush().unwrap(); // Ensure the prompt is printed

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please answer with 'y' or 'n'"),
        }
    }
}

async fn get_contract_hash_keys(
    public_key: &PublicKey,
    contract_name: &str,
    contract_package_name: &str,
) -> Option<(String, String)> {
    let query_params: QueryGlobalStateParams = QueryGlobalStateParams {
        key: KeyIdentifierInput::String(public_key.to_account_hash().to_formatted_string()),
        path: None,
        maybe_global_state_identifier: None,
        state_root_hash: None,
        maybe_block_id: None,
        node_address: None,
        verbosity: None,
    };

    let query_global_state = sdk().query_global_state(query_params).await;
    let query_global_state_result = query_global_state.unwrap_or_else(|_| {
        panic!("Failed to query global state");
    });

    let json_string = to_string(&query_global_state_result.result.stored_value)
        .unwrap_or_else(|_| panic!("Failed to convert stored value to string"));

    let parsed_json: Value =
        serde_json::from_str(&json_string).unwrap_or_else(|_| panic!("Failed to parse JSON"));

    let named_keys = parsed_json["Account"]["named_keys"]
        .as_array()
        .unwrap_or_else(|| panic!("named_keys is not an array"));

    // Find the contract hash
    let contract_hash = named_keys
        .iter()
        .find(|obj| obj["name"] == Value::String(contract_name.to_string()))
        .and_then(|obj| obj["key"].as_str())
        .unwrap_or_else(|| {
            log::error!("Contract hash key not found in named_keys");
            ""
        });

    if contract_hash.is_empty() {
        return None;
    }

    // Find the contract package hash
    let contract_package_hash = named_keys
        .iter()
        .find(|obj| obj["name"] == Value::String(contract_package_name.to_string()))
        .and_then(|obj| obj["key"].as_str())
        .unwrap_or_else(|| {
            log::error!("Package hash key not found in named_keys");
            ""
        });

    if contract_package_hash.is_empty() {
        return None;
    }

    Some((contract_hash.to_string(), contract_package_hash.to_string()))
}

// Specific function for getting CEP18 contract hash keys
pub async fn get_contract_cep18_hash_keys() -> Option<(String, String)> {
    let public_key = get_key_pair_from_vesting(INSTALLER)
        .await
        .unwrap()
        .public_key;
    get_contract_hash_keys(
        &public_key,
        &COWL_CEP18_TOKEN_CONTRACT_HASH_NAME.to_string(),
        &COWL_CEP18_TOKEN_CONTRACT_PACKAGE_HASH_NAME.to_string(),
    )
    .await
}

// Specific function for getting Vesting contract hash keys
pub async fn get_contract_vesting_hash_keys() -> Option<(String, String)> {
    let public_key = get_key_pair_from_vesting(INSTALLER)
        .await
        .unwrap()
        .public_key;
    get_contract_hash_keys(
        &public_key,
        &format!("{PREFIX_CONTRACT_NAME}_{}", *NAME_VESTING),
        &format!("{PREFIX_CONTRACT_PACKAGE_NAME}_{}", *NAME_VESTING),
    )
    .await
}

pub fn get_dictionary_item_params(
    key: &str,
    dictionary_name: &str,
    dictionary_item_key: &str,
) -> DictionaryItemInput {
    let mut params = DictionaryItemStrParams::new();
    params.set_contract_named_key(key, dictionary_name, dictionary_item_key);
    DictionaryItemInput::Params(params)
}

pub fn stored_value_to_vesting_info<T>(json_string: &str) -> Option<T>
where
    T: VestingData,
{
    // Parse the JSON string
    let parsed_json: Value = match serde_json::from_str(json_string) {
        Ok(v) => v,
        Err(_) => {
            log::error!("Failed to parse JSON string.");
            return None;
        }
    };

    // Extract the "bytes" field from parsed JSON
    let cl_value_as_value = &parsed_json["CLValue"]["bytes"];

    // Check if the "bytes" field exists and is a valid string
    if let Some(hex_string) = cl_value_as_value.as_str() {
        // Decode the hex string to raw bytes
        let raw_bytes = match hex::decode(hex_string) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::error!("Failed to decode hex string: {}", e);
                return None;
            }
        };

        // Attempt to deserialize the raw bytes into T
        match T::from_bytes(&raw_bytes) {
            Ok((info, _)) => Some(info),
            Err(e) => {
                log::error!("Error parsing bytes into VestingInfo: {:?}", e);
                None
            }
        }
    } else {
        log::error!("Expected 'bytes' field to be a string in JSON.");
        None
    }
}
