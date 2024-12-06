use std::{env, time::Duration};

use once_cell::sync::Lazy;

// All those following can be overifden by .env
const DEFAULT_RPC_ADDRESS: &str = "http://127.0.0.1:7777";
const DEFAULT_EVENT_ADDRESS: &str = "http://127.0.0.1:9999/events/main";
const DEFAULT_CHAIN_NAME: &str = "casper-net-1";
const DEFAULT_TTL: &str = "30m";
pub const DEFAULT_COWL_CEP_18_INSTALL_PAYMENT_AMOUNT: &str = "300000000000";
pub const DEFAULT_COWL_CEP_18_TOKEN_NAME: &str = "test";
pub const DEFAULT_COWL_CEP_18_TOKEN_SYMBOL: &str = "COWL";
pub const DEFAULT_COWL_CEP_18_TOKEN_DECIMALS: u8 = 9;
pub const DEFAULT_PREFIX_CEP18: &str = "cowl_cep18";

pub static RPC_ADDRESS: Lazy<String> =
    Lazy::new(|| env::var("RPC_ADDRESS").unwrap_or_else(|_| DEFAULT_RPC_ADDRESS.to_string()));
pub static EVENT_ADDRESS: Lazy<String> =
    Lazy::new(|| env::var("EVENT_ADDRESS").unwrap_or_else(|_| DEFAULT_EVENT_ADDRESS.to_string()));
pub static CHAIN_NAME: Lazy<String> =
    Lazy::new(|| env::var("CHAIN_NAME").unwrap_or_else(|_| DEFAULT_CHAIN_NAME.to_string()));
pub static TTL: Lazy<String> =
    Lazy::new(|| env::var("TTL").unwrap_or_else(|_| DEFAULT_TTL.to_string()));

pub static COWL_CEP_18_INSTALL_PAYMENT_AMOUNT: Lazy<String> = Lazy::new(|| {
    env::var("COWL_CEP_18_INSTALL_PAYMENT_AMOUNT")
        .unwrap_or_else(|_| DEFAULT_COWL_CEP_18_INSTALL_PAYMENT_AMOUNT.to_string())
});
pub static COWL_CEP_18_TOKEN_NAME: Lazy<String> = Lazy::new(|| {
    env::var("COWL_CEP_18_TOKEN_NAME")
        .unwrap_or_else(|_| DEFAULT_COWL_CEP_18_TOKEN_NAME.to_string())
});
pub static COWL_CEP_18_TOKEN_SYMBOL: Lazy<String> = Lazy::new(|| {
    env::var("COWL_CEP_18_TOKEN_SYMBOL")
        .unwrap_or_else(|_| DEFAULT_COWL_CEP_18_TOKEN_SYMBOL.to_string())
});
pub static COWL_CEP_18_TOKEN_DECIMALS: Lazy<String> = Lazy::new(|| {
    env::var("COWL_CEP_18_TOKEN_DECIMALS")
        .unwrap_or_else(|_| DEFAULT_COWL_CEP_18_TOKEN_DECIMALS.to_string())
});
pub static PREFIX_CEP18: Lazy<String> =
    Lazy::new(|| env::var("PREFIX_CEP18").unwrap_or_else(|_| DEFAULT_PREFIX_CEP18.to_string()));

pub static COWL_CEP18_TOKEN_CONTRACT_HASH_NAME: Lazy<String> = Lazy::new(|| {
    format!(
        "{}_contract_hash_{}",
        PREFIX_CEP18.to_string(),
        COWL_CEP_18_TOKEN_NAME.to_string()
    )
});
pub static COWL_CEP18_TOKEN_CONTRACT_PACKAGE_HASH_NAME: Lazy<String> = Lazy::new(|| {
    format!(
        "{}_contract_package_hash_{}",
        PREFIX_CEP18.to_string(),
        COWL_CEP_18_TOKEN_NAME.to_string()
    )
});

pub const DEPLOY_TIME: Duration = Duration::from_millis(45000);
pub const WASM_PATH: &str = "../tests/wasm/";

pub const FUNDED_KEYS_URL: &str =
    "https://raw.githubusercontent.com/casper-network/casper-node-launcher-js/main/src/config.ts";
pub const FUNDED_KEYS_JSON_FILE_PATH: &str = "funded_keys.json";

pub const INSTALLER: &str = "Installer";
pub const USER_1: &str = "User_1";
pub const USER_2: &str = "User_2";
