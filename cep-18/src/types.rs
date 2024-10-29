use alloc::string::String;
use casper_types::{U256, Key};

pub const TOKEN_NAME: &str = "COWL Token";
pub const TOKEN_SYMBOL: &str = "COWL";
pub const TOKEN_DECIMALS: u8 = 18;
pub const TOTAL_SUPPLY: U256 = U256::from(5_500_000_000_000_000_000_000_000_000u128); // 5.5B

// Distribution amounts
pub const TREASURY_AMOUNT: U256 = U256::from(2_750_000_000_000_000_000_000_000_000u128);
pub const COMMUNITY_STAKING_AMOUNT: U256 = U256::from(1_100_000_000_000_000_000_000_000_000u128);
pub const INVESTOR_AMOUNT: U256 = U256::from(550_000_000_000_000_000_000_000_000u128);
pub const TEAM_AMOUNT: U256 = U256::from(385_000_000_000_000_000_000_000_000u128);
pub const NETWORK_REWARDS_AMOUNT: U256 = U256::from(275_000_000_000_000_000_000_000_000u128);
pub const MARKETING_AMOUNT: U256 = U256::from(275_000_000_000_000_000_000_000_000u128);
pub const COMMUNITY_REWARDS_AMOUNT: U256 = U256::from(165_000_000_000_000_000_000_000_000u128);

// Time constants
pub const MONTH_IN_MS: u64 = 2_592_000_000;
pub const TWO_YEARS_IN_MS: u64 = MONTH_IN_MS * 24;
pub const TEN_YEARS_IN_MS: u64 = MONTH_IN_MS * 120;
pub const SIX_MONTHS_IN_MS: u64 = MONTH_IN_MS * 6;

pub const REQUIRED_SIGNATURES: u32 = 2;

#[derive(CLTyped, Clone)]
pub struct Distribution {
    pub category: String,
    pub amount: U256,
    pub address: Key,
    pub cliff_duration: u64,
    pub vesting_duration: u64,
}
