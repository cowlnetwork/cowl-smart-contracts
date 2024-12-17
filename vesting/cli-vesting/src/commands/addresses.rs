use super::balance::get_balance;
use crate::utils::{
    config::CONFIG_LOCK,
    constants::{COWL_CEP_18_COOL_SYMBOL, COWL_CEP_18_TOKEN_SYMBOL},
    format_with_thousands_separator, sdk,
};
use casper_rust_wasm_sdk::{
    helpers::motes_to_cspr,
    types::{key::Key, purse_identifier::PurseIdentifier},
};
use indexmap::IndexMap;

pub async fn list_funded_addresses() -> Option<IndexMap<String, IndexMap<String, String>>> {
    // Acquire the lock and clone info
    let cloned_config = {
        let config_lock = CONFIG_LOCK.lock().await;
        config_lock.clone()
    };

    if let Some(config) = cloned_config.as_ref() {
        let mut key_info_map: IndexMap<String, IndexMap<String, String>> = IndexMap::new();
        for (vesting_type, (key_pair, _)) in config {
            let mut public_key_map = IndexMap::new();

            public_key_map.insert(
                "public_key".to_string(),
                key_pair.public_key.to_string().clone(),
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
            let maybe_balance_motes = sdk()
                .query_balance(None, None, Some(purse_identifier), None, None, None, None)
                .await;

            let balance_motes = if let Ok(balance_motes) = maybe_balance_motes {
                balance_motes.result.balance.to_string()
            } else {
                log::warn!(
                    "No CSPR balance for {}\n\
                    - Private Key: {:?}\n\
                    - Public Key: {}\n\
                    - Account Hash: {}",
                    vesting_type,
                    key_pair.private_key_base64,
                    key_pair.public_key.to_string(),
                    key_pair.public_key.to_account_hash().to_formatted_string()
                );
                "0".to_string()
            };
            let balance = motes_to_cspr(&balance_motes).unwrap();

            public_key_map.insert("balance_motes".to_string(), balance_motes);

            public_key_map.insert(
                "balance_CSPR".to_string(),
                format_with_thousands_separator(&balance),
            );

            let balance_token = get_balance(
                None,
                Some(Key::from_account(
                    key_pair.public_key.clone().to_account_hash(),
                )),
            )
            .await;

            public_key_map.insert(
                format!("balance_{}", *COWL_CEP_18_COOL_SYMBOL),
                balance_token.clone(),
            );
            public_key_map.insert(
                format!("balance_{}", *COWL_CEP_18_TOKEN_SYMBOL),
                format_with_thousands_separator(&motes_to_cspr(&balance_token).unwrap()),
            );

            key_info_map.insert(vesting_type.clone(), public_key_map);
        }

        // Sort key_info_map by its keys
        let sorted_key_info_map = sort_indexmap(key_info_map);

        Some(sorted_key_info_map)
    } else {
        None
    }
}

fn sort_indexmap<K: Ord + Clone + std::hash::Hash, V: Clone>(
    map: IndexMap<K, V>,
) -> IndexMap<K, V> {
    let mut sorted_entries: Vec<_> = map.into_iter().collect();
    sorted_entries.sort_by(|a, b| a.0.cmp(&b.0));
    sorted_entries.into_iter().collect()
}

pub async fn print_funded_addresses() {
    if let Some(key_info_map) = list_funded_addresses().await {
        let json_output = serde_json::to_string_pretty(&key_info_map).unwrap();
        log::info!("{}", json_output);
    }
}
