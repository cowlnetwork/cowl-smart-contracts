use crate::utils::config::CONFIG_LOCK;
use std::collections::BTreeMap;

pub fn list_funded_addresses() {
    let config_lock = CONFIG_LOCK.lock().unwrap();

    if let Some(config) = config_lock.as_ref() {
        // Prepare a BTreeMap to hold the key information, using vesting_type as the key
        let mut key_info_map: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

        // Loop through the config and populate the key_info_map
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

            key_info_map.insert(vesting_type.to_string(), public_key_map);
        }

        // Serialize the BTreeMap to JSON with pretty formatting
        let json_output = serde_json::to_string_pretty(&key_info_map).unwrap();
        println!("{}", json_output);
    } else {
        println!("Config is empty or not initialized.");
    }
}
