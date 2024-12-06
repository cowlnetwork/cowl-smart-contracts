use std::{env, time::Duration};

use once_cell::sync::Lazy;

pub const DEFAULT_RPC_ADDRESS: &str = "http://127.0.0.1:7777";
pub const DEFAULT_EVENT_ADDRESS: &str = "http://127.0.0.1:9999/events/main";
pub const DEFAULT_CHAIN_NAME: &str = "casper-net-1";
pub const DEFAULT_TTL: &str = "30m";
pub const COWL_CEP_18_INSTALL_PAYMENT_AMOUNT: &str = "300000000000";
pub const COWL_CEP_18_TOKEN_NAME: &str = "test";
pub const COWL_CEP_18_TOKEN_SYMBOL: &str = "COWL";
pub const COWL_CEP_18_TOKEN_DECIMALS: u8 = 9;
pub const PREFIX_CEP18: &str = "cowl_cep18";

pub static COWL_CEP18_TOKEN_CONTRACT_HASH_NAME: Lazy<String> =
    Lazy::new(|| format!("{PREFIX_CEP18}_contract_hash_{COWL_CEP_18_TOKEN_NAME}"));
pub static COWL_CEP18_TOKEN_CONTRACT_PACKAGE_HASH_NAME: Lazy<String> =
    Lazy::new(|| format!("{PREFIX_CEP18}_contract_package_hash_{COWL_CEP_18_TOKEN_NAME}"));

pub static RPC_ADDRESS: Lazy<String> =
    Lazy::new(|| env::var("RPC_ADDRESS").unwrap_or_else(|_| DEFAULT_RPC_ADDRESS.to_string()));
pub static EVENT_ADDRESS: Lazy<String> =
    Lazy::new(|| env::var("EVENT_ADDRESS").unwrap_or_else(|_| DEFAULT_EVENT_ADDRESS.to_string()));
pub static CHAIN_NAME: Lazy<String> =
    Lazy::new(|| env::var("CHAIN_NAME").unwrap_or_else(|_| DEFAULT_CHAIN_NAME.to_string()));

pub const DEPLOY_TIME: Duration = Duration::from_millis(45000);
pub const WASM_PATH: &str = "../tests/wasm/";
pub const FUNDED_KEYS_URL: &str =
    "https://raw.githubusercontent.com/casper-network/casper-node-launcher-js/main/src/config.ts";
pub const FUNDED_KEYS_JSON_FILE_PATH: &str = "funded_keys.json";

pub const INSTALLER: &str = "Installer";
pub const USER_1: &str = "User_1";
pub const USER_2: &str = "User_2";
