use crate::utils::{
    config::get_key_pair_from_vesting,
    constants::{
        CHAIN_NAME, COWL_CEP18_TOKEN_CONTRACT_HASH_NAME,
        COWL_CEP18_TOKEN_CONTRACT_PACKAGE_HASH_NAME, COWL_CEP_18_INSTALL_PAYMENT_AMOUNT,
        COWL_CEP_18_TOKEN_DECIMALS, COWL_CEP_18_TOKEN_NAME, COWL_CEP_18_TOKEN_SYMBOL,
        EVENT_ADDRESS, INSTALLER, PREFIX_CEP18, TTL, WASM_PATH,
    },
    keys::format_base64_to_pem,
    prompt_yes_no, read_wasm_file, sdk,
};
use casper_rust_wasm_sdk::{
    deploy_watcher::watcher::EventParseResult,
    rpcs::query_global_state::{KeyIdentifierInput, QueryGlobalStateParams},
    types::{
        deploy_hash::DeployHash,
        deploy_params::{deploy_str_params::DeployStrParams, session_str_params::SessionStrParams},
        public_key::PublicKey,
    },
};
use cowl_vesting::enums::EventsMode;
use once_cell::sync::Lazy;
use serde_json::{to_string, Value};
use std::{io::Error, process};

pub const ARG_NAME: &str = "name";
pub const ARG_SYMBOL: &str = "symbol";
pub const ARG_DECIMALS: &str = "decimals";
pub const ARG_TOTAL_SUPPLY: &str = "total_supply";
pub const ARG_EVENTS_MODE: &str = "events_mode";
pub const ARG_ENABLE_MINT_BURN: &str = "enable_mint_burn";

pub static ARGS_JSON: Lazy<String> = Lazy::new(|| {
    format!(
        r#"[
{{"name": "{ARG_NAME}", "type": "String", "value": "{COWL_CEP_18_TOKEN_NAME}"}},
{{"name": "{ARG_SYMBOL}", "type": "String", "value": "{COWL_CEP_18_TOKEN_SYMBOL}"}},
{{"name": "{ARG_DECIMALS}", "type": "U8", "value": {COWL_CEP_18_TOKEN_DECIMALS}}},
{{"name": "{ARG_TOTAL_SUPPLY}", "type": "U8", "value": 0}},
{{"name": "{ARG_EVENTS_MODE}", "type": "U8", "value": {events_mode}}},
{{"name": "{ARG_ENABLE_MINT_BURN}", "type": "Bool", "value": true}}
]"#,
        COWL_CEP_18_TOKEN_NAME = COWL_CEP_18_TOKEN_NAME.to_string(),
        COWL_CEP_18_TOKEN_SYMBOL = COWL_CEP_18_TOKEN_SYMBOL.to_string(),
        COWL_CEP_18_TOKEN_DECIMALS = COWL_CEP_18_TOKEN_DECIMALS.to_string(),
        events_mode = EventsMode::CES as u8,
    )
});

pub async fn deploy_all_contracts() -> Result<(), Error> {
    deploy_cep18_token().await
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
    let module_bytes =
        match read_wasm_file(&format!("{}{}.wasm", WASM_PATH, PREFIX_CEP18.to_string())) {
            Ok(module_bytes) => module_bytes,
            Err(err) => {
                eprintln!("Error reading file: {:?}", err);
                return Err(err);
            }
        };
    session_params.set_session_bytes(module_bytes.into());
    session_params.set_session_args_json(&ARGS_JSON);

    let install = sdk()
        .install(
            deploy_params,
            session_params,
            &COWL_CEP_18_INSTALL_PAYMENT_AMOUNT,
            None,
        )
        .await;
    assert!(!install
        .as_ref()
        .unwrap()
        .result
        .api_version
        .to_string()
        .is_empty());

    let deploy_hash = DeployHash::from(install.as_ref().unwrap().result.deploy_hash);
    let deploy_hash_as_string = deploy_hash.to_string();
    assert!(!deploy_hash_as_string.is_empty());
    println!("wait deploy_hash {}", deploy_hash_as_string);
    let event_parse_result: EventParseResult = sdk()
        .wait_deploy(&EVENT_ADDRESS, &deploy_hash_as_string, None)
        .await
        .unwrap();
    println!("{:?}", event_parse_result);
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

async fn get_contract_cep18_hash_keys(public_key: &PublicKey) -> Option<(String, String)> {
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

    let contract_cep18_hash = named_keys
        .iter()
        .find(|obj| obj["name"] == Value::String(COWL_CEP18_TOKEN_CONTRACT_HASH_NAME.to_string()))
        .and_then(|obj| obj["key"].as_str())
        .unwrap_or_else(|| {
            eprintln!("Contract cep18 key not found in named_keys");
            ""
        });

    // If contract_cep18_hash is None, return early
    if contract_cep18_hash.is_empty() {
        return None;
    }

    // Find the contract CEP18 package hash
    let contract_cep18_package_hash = named_keys
        .iter()
        .find(|obj| {
            obj["name"] == Value::String(COWL_CEP18_TOKEN_CONTRACT_PACKAGE_HASH_NAME.to_string())
        })
        .and_then(|obj| obj["key"].as_str())
        .unwrap_or_else(|| {
            eprintln!("Package cep18 key not found in named_keys");
            ""
        });

    // If contract_cep18_package_hash is None, return early
    if contract_cep18_package_hash.is_empty() {
        return None;
    }

    // Return the result if both keys were found
    Some((
        contract_cep18_hash.to_string(),
        contract_cep18_package_hash.to_string(),
    ))
}
