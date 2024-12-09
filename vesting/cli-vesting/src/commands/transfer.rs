use crate::utils::{
    call_token_transfer_entry_point, constants::COWL_CEP_18_TOKEN_SYMBOL,
    get_contract_cep18_hash_keys, keys::get_private_key_base64, process_base64_or_path,
    prompt_base64_or_path, prompt_yes_no,
};
use casper_rust_wasm_sdk::{
    helpers::motes_to_cspr,
    types::{key::Key, public_key::PublicKey},
};

use super::balance::vesting_balance;

pub async fn vesting_transfer(
    from: Option<PublicKey>,
    to: Option<Key>,
    amount: String,
) -> Option<String> {
    let from = match from {
        Some(public_key) => public_key,
        None => {
            log::error!("Failed to retrieve sender account.");
            return None;
        }
    };

    let to = match to {
        Some(target_key) => target_key,
        None => {
            log::error!("Failed to retrieve target account.");
            return None;
        }
    };

    // Retrieve contract vesting hash and package hash
    let (cowl_cep18_token_contract_hash, _) = match get_contract_cep18_hash_keys().await {
        Some((hash, package_hash)) => (hash, package_hash),
        None => {
            log::error!("Failed to retrieve contract token hash and package hash.");
            return None;
        }
    };

    // Retrieve the secret key
    let secret_key = match get_private_key_base64(&from).await {
        Some(key) => Some(key),
        None => {
            let answer = prompt_base64_or_path(&format!(
                "Missing private key for {}, do you want to provide a base64 string or a .pem file path?",
                from
            ));
            let processed_key = process_base64_or_path(answer);
            if processed_key.is_empty() {
                log::warn!("No valid private key provided.");
                None
            } else {
                Some(processed_key)
            }
        }
    };

    let answer = prompt_yes_no(&format!(
        "Please confirm transfer of {} {} to {}?",
        motes_to_cspr(&amount).unwrap(),
        *COWL_CEP_18_TOKEN_SYMBOL,
        &to.to_formatted_string()
    ));

    if !answer {
        log::warn!("Transfer aborted.");
        return None;
    }

    // Call the token transfer entry point
    call_token_transfer_entry_point(
        &cowl_cep18_token_contract_hash,
        &from,
        secret_key.expect("Failed to retrieve sender private key."),
        &to,
        amount,
    )
    .await;

    let to_balance = vesting_balance(None, Some(to)).await;
    Some(to_balance)
}

pub async fn print_vesting_transfer(from: Option<PublicKey>, to: Option<Key>, amount: String) {
    if let Some(vesting_balance) = vesting_transfer(from, to.clone(), amount).await {
        log::info!("Balance for {}", to.unwrap().to_formatted_string());
        log::info!(
            "{} {}",
            motes_to_cspr(&vesting_balance).unwrap(),
            *COWL_CEP_18_TOKEN_SYMBOL
        );
    }
}
