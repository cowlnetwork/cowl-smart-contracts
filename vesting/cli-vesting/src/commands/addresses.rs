use crate::utils::{config::CONFIG_LOCK, sdk};
use casper_rust_wasm_sdk::{helpers::motes_to_cspr, types::purse_identifier::PurseIdentifier};
use std::collections::BTreeMap;

pub async fn list_funded_addresses() -> Option<BTreeMap<String, BTreeMap<String, String>>> {
    // Acquire the lock
    let config_lock = CONFIG_LOCK.lock().await;

    if let Some(config) = config_lock.as_ref() {
        let mut key_info_map: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        for (vesting_type, (key_pair, _)) in config {
            let mut public_key_map = BTreeMap::new();
            public_key_map.insert(
                "public_key_hex".to_string(),
                key_pair.public_key_hex.clone(),
            );
            public_key_map.insert(
                "account_hash".to_string(),
                key_pair
                    .public_key
                    .clone()
                    .to_account_hash()
                    .to_formatted_string(),
            );

            let purse_identifier = PurseIdentifier::from_main_purse_under_account_hash(
                key_pair.public_key.clone().to_account_hash(),
            );
            let balance_motes = sdk()
                .query_balance(None, None, Some(purse_identifier), None, None, None, None)
                .await
                .unwrap()
                .result
                .balance;
            let balance = motes_to_cspr(&balance_motes.to_string()).unwrap();

            public_key_map.insert("balance CSPR".to_string(), balance.to_string());

            key_info_map.insert(vesting_type.to_string(), public_key_map);
        }

        Some(key_info_map)
    } else {
        None
    }
}

pub async fn print_funded_addresses() {
    if let Some(key_info_map) = list_funded_addresses().await {
        let json_output = serde_json::to_string_pretty(&key_info_map).unwrap();
        log::info!("{}", json_output);
    } else {
        log::error!("Config is empty or not initialized");
    }
}
