use crate::rate::RateTier;
use casper_types::U512;

pub const PREFIX_ACCESS_KEY_NAME: &str = "swap_contract_package_access";
pub const PREFIX_CONTRACT_NAME: &str = "swap_contract_hash";
pub const PREFIX_CONTRACT_VERSION: &str = "swap_contract_version";
pub const PREFIX_CONTRACT_PACKAGE_NAME: &str = "swap_contract_package_hash";

pub const ENTRY_POINT_ALLOWANCE: &str = "allowance";
pub const ENTRY_POINT_APPROVE: &str = "approve";
pub const ENTRY_POINT_BALANCE_COWL: &str = "balance_cowl";
pub const ENTRY_POINT_BALANCE_CSPR: &str = "balance_cspr";
pub const ENTRY_POINT_BALANCE_OF: &str = "balance_of";
pub const ENTRY_POINT_CHANGE_SECURITY: &str = "change_security";
pub const ENTRY_POINT_COWL_TO_CSPR: &str = "cowl_to_cspr";
pub const ENTRY_POINT_CSPR_TO_COWL: &str = "cspr_to_cowl";
pub const ENTRY_POINT_DEPOSIT_CSPR: &str = "deposit_cspr";
pub const ENTRY_POINT_INSTALL: &str = "install";
pub const ENTRY_POINT_SET_MODALITIES: &str = "set_modalities";
pub const ENTRY_POINT_TRANSFER: &str = "transfer";
pub const ENTRY_POINT_TRANSFER_FROM: &str = "transfer_from";
pub const ENTRY_POINT_UPDATE_TIMES: &str = "update_times";
pub const ENTRY_POINT_UPGRADE: &str = "upgrade";
pub const ENTRY_POINT_WITHDRAW_COWL: &str = "withdraw_cowl";
pub const ENTRY_POINT_WITHDRAW_CSPR: &str = "withdraw_cspr";

pub const ARG_ADDRESS: &str = "address";
pub const ARG_AMOUNT: &str = "amount";
pub const ARG_BALANCE_COWL: &str = "balance_cowl";
pub const ARG_BALANCE_CSPR: &str = "balance_cspr";
pub const ARG_CONTRACT_HASH: &str = "contract_hash";
pub const ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH: &str = "cep18_contract_package_hash";
pub const ARG_COWL_SWAP_CONTRACT_PACKAGE_HASH: &str = "swap_contract_package_hash";
pub const ARG_END_TIME: &str = "end_time";
pub const ARG_EVENTS_MODE: &str = "events_mode";
pub const ARG_INSTALLER: &str = "installer";
pub const ARG_NAME: &str = "name";
pub const ARG_OWNER: &str = "owner";
pub const ARG_PACKAGE_HASH: &str = "package_hash";
pub const ARG_PURSE: &str = "purse";
pub const ARG_RECIPIENT: &str = "recipient";
pub const ARG_SPENDER: &str = "spender";
pub const ARG_START_TIME: &str = "start_time";
pub const ARG_UPGRADE_FLAG: &str = "upgrade";

pub const DICT_SECURITY_BADGES: &str = "security_badges";

pub const ADMIN_LIST: &str = "admin_list";
pub const MINTER_LIST: &str = "minter_list";
pub const NONE_LIST: &str = "none_list";

pub const MIN_SWAP_AMOUNT: U512 = U512([10_000_000_000u64, 0, 0, 0, 0, 0, 0, 0]);

pub const RATE_TIERS: [RateTier; 4] = [
    RateTier {
        cspr_amount: U512([50_000_000_000_000u64, 0, 0, 0, 0, 0, 0, 0]),
        rate: U512([3u64, 0, 0, 0, 0, 0, 0, 0]),
    },
    RateTier {
        cspr_amount: U512([100_000_000_000_000u64, 0, 0, 0, 0, 0, 0, 0]),
        rate: U512([4u64, 0, 0, 0, 0, 0, 0, 0]),
    },
    RateTier {
        cspr_amount: U512([500_000_000_000_000u64, 0, 0, 0, 0, 0, 0, 0]),
        rate: U512([5u64, 0, 0, 0, 0, 0, 0, 0]),
    },
    RateTier {
        cspr_amount: U512([1_000_000_000_000_000u64, 0, 0, 0, 0, 0, 0, 0]),
        rate: U512([6u64, 0, 0, 0, 0, 0, 0, 0]),
    },
];

pub const TAX_RATE: U512 = U512([10, 0, 0, 0, 0, 0, 0, 0]);
