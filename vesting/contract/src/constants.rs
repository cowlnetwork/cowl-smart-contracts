use time::Duration;

pub const PREFIX_ACCESS_KEY_NAME: &str = "vesting_contract_package_access";
pub const PREFIX_CONTRACT_NAME: &str = "vesting_contract_hash";
pub const PREFIX_CONTRACT_VERSION: &str = "vesting_contract_version";
pub const PREFIX_CONTRACT_PACKAGE_NAME: &str = "vesting_contract_package_hash";

pub const ENTRY_POINT_BALANCE_OF: &str = "balance_of";
pub const ENTRY_POINT_CHANGE_SECURITY: &str = "change_security";
pub const ENTRY_POINT_CHECK_VESTING_TRANSFER: &str = "check_vesting_transfer";
pub const ENTRY_POINT_COWL_CEP18_CONTRACT_PACKAGE: &str = "set_cowl_cep18_contract_package";
pub const ENTRY_POINT_DECIMALS: &str = "decimals";
pub const ENTRY_POINT_DECREASE_ALLOWANCE: &str = "decrease_allowance";
pub const ENTRY_POINT_INCREASE_ALLOWANCE: &str = "increase_allowance";
pub const ENTRY_POINT_INSTALL: &str = "install";
pub const ENTRY_POINT_MINT: &str = "mint";
pub const ENTRY_POINT_SET_MODALITIES: &str = "set_modalities";
pub const ENTRY_POINT_SET_TRANSFER_FILTER: &str = "set_transfer_filter";
pub const ENTRY_POINT_TOTAL_SUPPLY: &str = "total_supply";
pub const ENTRY_POINT_TRANSFER: &str = "transfer";
pub const ENTRY_POINT_TRANSFER_FROM: &str = "transfer_from";
pub const ENTRY_POINT_UPGRADE: &str = "upgrade";
pub const ENTRY_POINT_VESTING_INFO: &str = "vesting_info";
pub const ENTRY_POINT_VESTING_STATUS: &str = "vesting_status";

pub const ARG_ADDRESS: &str = "address";
pub const ARG_AMOUNT: &str = "amount";
pub const ARG_COWL_CEP18_CONTRACT_PACKAGE: &str = "cowl_cep18_contract_package";
pub const ARG_CONTRACT_HASH: &str = "contract_hash";
pub const ARG_DATA: &str = "data";
pub const ARG_ENABLE_MINT_BURN: &str = "enable_mint_burn";
pub const ARG_EVENTS_MODE: &str = "events_mode";
pub const ARG_FROM: &str = "from";
pub const ARG_INSTALLER: &str = "installer";
pub const ARG_NAME: &str = "name";
pub const ARG_OPERATOR: &str = "operator";
pub const ARG_OWNER: &str = "owner";
pub const ARG_PACKAGE_HASH: &str = "package_hash";
pub const ARG_RECIPIENT: &str = "recipient";
pub const ARG_SPENDER: &str = "spender";
pub const ARG_TOTAL_SUPPLY: &str = "total_supply";
pub const ARG_TO: &str = "to";
pub const ARG_TRANSFER_FILTER_CONTRACT_PACKAGE: &str = "transfer_filter_contract_package";
pub const ARG_TRANSFER_FILTER_METHOD: &str = "transfer_filter_method";
pub const ARG_UPGRADE_FLAG: &str = "upgrade";
pub const ARG_VESTING_TYPE: &str = "vesting_type";

pub const DICT_ADDRESSES: &str = "addresses";
pub const DICT_ALLOWANCES: &str = "allowances";
pub const DICT_BALANCES: &str = "balances";
pub const DICT_SECURITY_BADGES: &str = "security_badges";
pub const DICT_START_TIME: &str = "start_time";
pub const DICT_TRANSFERRED_AMOUNT: &str = "transfered_amount";
pub const DICT_VESTING_AMOUNT: &str = "vesting_amount";
pub const DICT_VESTING_INFO: &str = "vesting_info";
pub const DICT_VESTING_STATUS: &str = "vesting_status";

pub const ADMIN_LIST: &str = "admin_list";
pub const MINTER_LIST: &str = "minter_list";
pub const NONE_LIST: &str = "none_list";

// This is COWL Unit, not the smallest unit with decimal
pub const COWL_CEP_18_TOKEN_TOTAL_SUPPLY: u64 = 5_500_000_000;

// Durations
pub const HOUR_IN_SECONDS: u64 = 60 * 60;
pub const YEAR_IN_SECONDS: u64 = 365 * 24 * 60 * 60; // A standard year in seconds
pub const MONTH_IN_SECONDS: u64 = YEAR_IN_SECONDS / 12; // Approximation for a month

const _ONE_HOUR_IN_SECONDS: Duration = Duration::seconds(HOUR_IN_SECONDS as i64);
const ONE_MONTH_IN_SECONDS: Duration = Duration::seconds(MONTH_IN_SECONDS as i64);
const ONE_YEAR_IN_SECONDS: Duration = Duration::seconds(YEAR_IN_SECONDS as i64);
const FOUR_YEARS_IN_SECONDS: Duration = Duration::seconds(4 * YEAR_IN_SECONDS as i64);

/// Lock durations for each vesting type
pub const DURATION_LIQUIDITY_VESTING: Option<Duration> = None;
pub const DURATION_CONTRIBUTOR_VESTING: Option<Duration> = Some(ONE_YEAR_IN_SECONDS);
pub const DURATION_DEVELOPMENT_VESTING: Option<Duration> = Some(ONE_YEAR_IN_SECONDS);
pub const DURATION_TREASURY_VESTING: Option<Duration> = Some(FOUR_YEARS_IN_SECONDS);
pub const DURATION_COMMUNITY_VESTING: Option<Duration> = Some(FOUR_YEARS_IN_SECONDS);
pub const DURATION_STAKING_VESTING: Option<Duration> = None;

pub const VESTING_PERIOD_IN_SECONDS: Duration = ONE_MONTH_IN_SECONDS;
// pub const VESTING_PERIOD_IN_SECONDS: Duration = _ONE_HOUR_IN_SECONDS;
