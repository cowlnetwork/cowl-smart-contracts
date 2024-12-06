use crate::utils::{
    config::{get_key_pair_from_vesting, CONFIG_LOCK},
    constants::{
        CHAIN_NAME, COWL_CEP18_TOKEN_CONTRACT_HASH_NAME,
        COWL_CEP18_TOKEN_CONTRACT_PACKAGE_HASH_NAME, COWL_CEP_18_INSTALL_PAYMENT_AMOUNT,
        COWL_CEP_18_TOKEN_DECIMALS, COWL_CEP_18_TOKEN_NAME, COWL_CEP_18_TOKEN_SYMBOL,
        EVENT_ADDRESS, INSTALLER, NAME_CEP18, NAME_VESTING, TTL, WASM_PATH,
    },
    keys::format_base64_to_pem,
    prompt_yes_no, read_wasm_file, sdk,
};
use casper_rust_wasm_sdk::{
    deploy_watcher::watcher::EventParseResult,
    helpers::motes_to_cspr,
    rpcs::query_global_state::{KeyIdentifierInput, QueryGlobalStateParams},
    types::{
        deploy_hash::DeployHash,
        deploy_params::{deploy_str_params::DeployStrParams, session_str_params::SessionStrParams},
        public_key::PublicKey,
    },
};
use cowl_vesting::{
    constants::{
        ARG_COWL_CEP18_CONTRACT_PACKAGE, PREFIX_CONTRACT_NAME, PREFIX_CONTRACT_PACKAGE_NAME,
    },
    enums::EventsMode,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{to_string, Value};
use std::{io::Error, process};

const ARG_NAME: &str = "name";
const ARG_SYMBOL: &str = "symbol";
const ARG_DECIMALS: &str = "decimals";
const ARG_TOTAL_SUPPLY: &str = "total_supply";
const ARG_EVENTS_MODE: &str = "events_mode";
const ARG_ENABLE_MINT_BURN: &str = "enable_mint_burn";

static ARGS_CEP18_JSON: Lazy<String> = Lazy::new(|| {
    format!(
        r#"[
{{"name": "{ARG_NAME}", "type": "String", "value": "{COWL_CEP_18_TOKEN_NAME}"}},
{{"name": "{ARG_SYMBOL}", "type": "String", "value": "{COWL_CEP_18_TOKEN_SYMBOL}"}},
{{"name": "{ARG_DECIMALS}", "type": "U8", "value": {COWL_CEP_18_TOKEN_DECIMALS}}},
{{"name": "{ARG_TOTAL_SUPPLY}", "type": "U8", "value": 0}},
{{"name": "{ARG_EVENTS_MODE}", "type": "U8", "value": {events_mode}}},
{{"name": "{ARG_ENABLE_MINT_BURN}", "type": "Bool", "value": true}}
]"#,
        COWL_CEP_18_TOKEN_NAME = *COWL_CEP_18_TOKEN_NAME,
        COWL_CEP_18_TOKEN_SYMBOL = *COWL_CEP_18_TOKEN_SYMBOL,
        COWL_CEP_18_TOKEN_DECIMALS = *COWL_CEP_18_TOKEN_DECIMALS,
        events_mode = EventsMode::CES as u8,
    )
});

pub async fn deploy_all_contracts() -> Result<(), Error> {
    deploy_cep18_token().await?;
    deploy_vesting_contract().await?;
    Ok(())
}

async fn deploy_cep18_token() -> Result<(), Error> {
    let key_pair = get_key_pair_from_vesting(INSTALLER).unwrap();

    let (contract_cep18_hash, _) = match get_contract_cep18_hash_keys(&key_pair.public_key).await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => (String::from(""), String::from("")),
    };

    if !contract_cep18_hash.is_empty() {
        let answer = prompt_yes_no(&format!(
            "Contract already exists at {}, do you want to overwrite?",
            contract_cep18_hash
        ));

        if answer {
            println!("You chose to overwrite.");
        } else {
            println!("You chose not to overwrite.");
            process::exit(0);
        }
    }

    let deploy_params = DeployStrParams::new(
        &CHAIN_NAME,
        &key_pair.public_key_hex,
        Some(format_base64_to_pem(&key_pair.private_key_base64.clone())),
        None,
        Some(TTL.to_string()),
    );

    let session_params = SessionStrParams::default();
    let module_bytes = match read_wasm_file(&format!("{}{}.wasm", WASM_PATH, *NAME_CEP18)) {
        Ok(module_bytes) => module_bytes,
        Err(err) => {
            eprintln!("Error reading file: {:?}", err);
            return Err(err);
        }
    };
    session_params.set_session_bytes(module_bytes.into());
    session_params.set_session_args_json(&ARGS_CEP18_JSON);

    let install = sdk()
        .install(
            deploy_params,
            session_params,
            &COWL_CEP_18_INSTALL_PAYMENT_AMOUNT,
            None,
        )
        .await;
    let api_version = install.as_ref().unwrap().result.api_version.to_string();

    if api_version.is_empty() {
        eprintln!("Failed to retrieve contract API version.");
        process::exit(1)
    }

    let deploy_hash = DeployHash::from(install.as_ref().unwrap().result.deploy_hash);
    let deploy_hash_as_string = deploy_hash.to_string();

    if deploy_hash_as_string.is_empty() {
        eprintln!("Failed to retrieve deploy hash.");
        process::exit(1)
    }

    println!(
        "wait deploy_hash for token install {}",
        deploy_hash_as_string
    );

    let event_parse_result: EventParseResult = sdk()
        .wait_deploy(&EVENT_ADDRESS, &deploy_hash_as_string, None)
        .await
        .unwrap();

    let motes = event_parse_result
        .body
        .unwrap()
        .deploy_processed
        .unwrap()
        .execution_result
        .success
        .unwrap()
        .cost;

    let cost = motes_to_cspr(&motes).unwrap();

    println!("Cost {cost} CSPR");

    let finalized_approvals = true;
    let get_deploy = sdk()
        .get_deploy(deploy_hash, Some(finalized_approvals), None, None)
        .await;
    let get_deploy = get_deploy.unwrap();
    let result = DeployHash::from(get_deploy.result.deploy.hash).to_string();
    println!("processed deploy hash {result}");
    let (contract_cep18_hash, contract_cep18_package_hash) =
        match get_contract_cep18_hash_keys(&key_pair.public_key).await {
            Some((hash, package_hash)) => (hash, package_hash),
            None => {
                eprintln!("Failed to retrieve contract CEP18 keys.");
                process::exit(1)
            }
        };
    println!("contract_cep18_hash {contract_cep18_hash}");
    println!("contract_cep18_package_hash {contract_cep18_package_hash}");
    Ok(())
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
            eprintln!("Contract hash key not found in named_keys");
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
            eprintln!("Package hash key not found in named_keys");
            ""
        });

    if contract_package_hash.is_empty() {
        return None;
    }

    Some((contract_hash.to_string(), contract_package_hash.to_string()))
}

// Specific function for getting CEP18 contract hash keys
async fn get_contract_cep18_hash_keys(public_key: &PublicKey) -> Option<(String, String)> {
    get_contract_hash_keys(
        public_key,
        &COWL_CEP18_TOKEN_CONTRACT_HASH_NAME.to_string(),
        &COWL_CEP18_TOKEN_CONTRACT_PACKAGE_HASH_NAME.to_string(),
    )
    .await
}

// Specific function for getting Vesting contract hash keys
async fn get_contract_vesting_hash_keys(public_key: &PublicKey) -> Option<(String, String)> {
    get_contract_hash_keys(
        public_key,
        &format!("{PREFIX_CONTRACT_NAME}_{}", *NAME_VESTING),
        &format!("{PREFIX_CONTRACT_PACKAGE_NAME}_{}", *NAME_VESTING),
    )
    .await
}

async fn deploy_vesting_contract() -> Result<(), Error> {
    let key_pair = get_key_pair_from_vesting(INSTALLER).unwrap();

    let (cowl_cep18_token_contract_hash, cowl_cep18_token_package_hash) =
        match get_contract_cep18_hash_keys(&key_pair.public_key).await {
            Some((hash, package_hash)) => (hash, package_hash),
            None => (String::from(""), String::from("")),
        };

    if cowl_cep18_token_contract_hash.is_empty() {
        eprintln!("Token contract does not exist in installer named keys at {cowl_cep18_token_contract_hash}");
        process::exit(1)
    }

    let deploy_params = DeployStrParams::new(
        &CHAIN_NAME,
        &key_pair.public_key_hex,
        Some(format_base64_to_pem(&key_pair.private_key_base64.clone())),
        None,
        Some(TTL.to_string()),
    );

    let session_params = SessionStrParams::default();
    let module_bytes = match read_wasm_file(&format!("{}{}.wasm", WASM_PATH, *NAME_VESTING)) {
        Ok(module_bytes) => module_bytes,
        Err(err) => {
            eprintln!("Error reading file: {:?}", err);
            return Err(err);
        }
    };
    session_params.set_session_bytes(module_bytes.into());

    let mut args_vesting_json_addresses_vec: Vec<String> = Vec::new();
    if let Some(config) = CONFIG_LOCK.lock().unwrap().as_ref() {
        for (vesting_type, (key_pair, maybe_vesting_info)) in config {
            if let Some(_vesting_info) = maybe_vesting_info {
                args_vesting_json_addresses_vec.push(format!(
                    r#"{{
                        "name": "{vesting_type}",
                        "type": "Key",
                        "value": "{funded_address_key}"
                    }}"#,
                    vesting_type = vesting_type,
                    funded_address_key =
                        key_pair.public_key.to_account_hash().to_formatted_string()
                ));
            }
        }
    }

    let args_vesting_json_addresses = args_vesting_json_addresses_vec.join(",\n");
    let args_vesting_json = format!(
        r#"[
            {{
                "name": "{ARG_NAME}",
                "type": "String",
                "value": "{NAME_VESTING}"
            }},
            {{
                "name": "{ARG_COWL_CEP18_CONTRACT_PACKAGE}",
                "type": "Key",
                "value": "{cowl_cep18_token_package_hash}"
            }},
            {args_vesting_json_addresses}
        ]"#,
        NAME_VESTING = *NAME_VESTING
    );

    let args_vesting_json = Regex::new(r"\s+")
        .unwrap()
        .replace_all(&args_vesting_json, "")
        .to_string();

    session_params.set_session_args_json(&args_vesting_json);

    let install = sdk()
        .install(
            deploy_params,
            session_params,
            &COWL_CEP_18_INSTALL_PAYMENT_AMOUNT,
            None,
        )
        .await;

    let api_version = install.as_ref().unwrap().result.api_version.to_string();

    if api_version.is_empty() {
        eprintln!("Failed to retrieve contract API version.");
        process::exit(1)
    }

    let deploy_hash = DeployHash::from(install.as_ref().unwrap().result.deploy_hash);
    let deploy_hash_as_string = deploy_hash.to_string();

    if deploy_hash_as_string.is_empty() {
        eprintln!("Failed to retrieve deploy hash.");
        process::exit(1)
    }

    println!(
        "wait deploy_hash for vesting install {}",
        deploy_hash_as_string
    );
    let event_parse_result: EventParseResult = sdk()
        .wait_deploy(&EVENT_ADDRESS, &deploy_hash_as_string, None)
        .await
        .unwrap();
    let motes = event_parse_result
        .body
        .unwrap()
        .deploy_processed
        .unwrap()
        .execution_result
        .success
        .unwrap()
        .cost;

    let cost = motes_to_cspr(&motes).unwrap();

    println!("Cost {cost} CSPR");

    let finalized_approvals = true;
    let get_deploy = sdk()
        .get_deploy(deploy_hash, Some(finalized_approvals), None, None)
        .await;
    let get_deploy = get_deploy.unwrap();
    let result = DeployHash::from(get_deploy.result.deploy.hash).to_string();
    println!("processed deploy hash {result}");
    let (contract_vesting_hash, contract_vesting_package_hash) =
        match get_contract_vesting_hash_keys(&key_pair.public_key).await {
            Some((hash, package_hash)) => (hash, package_hash),
            None => {
                eprintln!("Failed to retrieve contract vesting keys.");
                process::exit(1)
            }
        };
    println!("contract_vesting_hash {contract_vesting_hash}");
    println!("contract_vesting_package_hash {contract_vesting_package_hash}");
    Ok(())
}
