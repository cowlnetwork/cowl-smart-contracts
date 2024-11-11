#![no_std]
#![no_main]

extern crate alloc;

use alloc::{format, vec, string::ToString};
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    CLType,
    CLTyped,
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys},
    ContractHash,
    Key,
    Parameter,
    runtime_args,
    RuntimeArgs,
    U256,
    U512,
    ApiError, URef, BlockTime
};

use core::convert::TryInto;

/// Contract name
const CONTRACT_NAME: &str = "cowl_ghost_swap";
/// Contract version
const CONTRACT_VERSION: &str = "1.0.0";
const CONTRACT_HASH_NAME: &str = "cowl_ghost_swap_contract_hash";
const CONTRACT_ACCESS_UREF_NAME: &str = "cowl_ghost_swap_access_uref";
/// Minimum swap amount in CSPR (50,000)
const MIN_SWAP_AMOUNT: U512 = U512([1_000_000_000_000, 0, 0, 0, 0, 0, 0, 0]);
/// Tax rate for COWL to CSPR swaps (10%)
const TAX_RATE: U512 = U512([10, 0, 0, 0, 0, 0, 0, 0]);
/// Maximum tax rate denominator (100%)
const TAX_DENOMINATOR: U512 = U512([100, 0, 0, 0, 0, 0, 0, 0]);


/// Named keys used in the contract
pub mod named_keys {
    pub const OWNER: &str = "owner";
    pub const START_TIME: &str = "start_time";
    pub const END_TIME: &str = "end_time";
    pub const COWL_TOKEN: &str = "cowl_token";
    pub const CONTRACT_PURSE: &str = "contract_purse";
    pub const SELF_CONTRACT_HASH: &str = "self_contract_hash"; // Removed SELF_PACKAGE_HASH (not used)

}

/// Contract error codes
#[repr(u16)]
pub enum Error {
    /// Swap operation is not yet active
    SwapNotActive = 1,
    /// Swap operation has expired
    SwapExpired = 2,
    /// Insufficient balance for operation
    InsufficientBalance = 3,
    /// Amount is below minimum requirement
    BelowMinimumSwap = 4,
    /// Caller is not authorized
    Unauthorized = 5,
    /// Invalid parameter provided
    InvalidParameter = 6,
    /// Transfer operation failed
    TransferFailed = 7,
    /// Invalid rate (zero or invalid)
    InvalidRate = 8,
    /// Zero amount provided
    ZeroAmount = 9,
    /// Invalid time window
    InvalidTimeWindow = 10,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}

/// Represents a swap rate tier
#[derive(Clone, Copy)]
struct RateTier {
    cspr_amount: U512,
    rate: U512,
}

const RATE_TIERS: [RateTier; 4] = [
    RateTier {
        cspr_amount: U512([50_000_000_000_000, 0, 0, 0, 0, 0, 0, 0]),
        rate: U512([3, 0, 0, 0, 0, 0, 0, 0]),
    },
    RateTier {
        cspr_amount: U512([100_000_000_000_000, 0, 0, 0, 0, 0, 0, 0]),
        rate: U512([4, 0, 0, 0, 0, 0, 0, 0]),
    },
    RateTier {
        cspr_amount: U512([500_000_000_000_000, 0, 0, 0, 0, 0, 0, 0]),
        rate: U512([5, 0, 0, 0, 0, 0, 0, 0]),
    },
    RateTier {
        cspr_amount: U512([1_000_000_000_000_000, 0, 0, 0, 0, 0, 0, 0]),
        rate: U512([6, 0, 0, 0, 0, 0, 0, 0]),
    },
];

#[no_mangle]
pub extern "C" fn call() {
    // Installation parameters
    let start_time: u64 = runtime::get_named_arg("start_time");
    let end_time: u64 = runtime::get_named_arg("end_time");
    let cowl_token: ContractHash = runtime::get_named_arg("cowl_token");

    // Create contract purse
    let contract_purse = system::create_purse();

    // Set up entry points
    let mut entry_points = EntryPoints::new();

    // CSPR to COWL swap entry point
    entry_points.add_entry_point(EntryPoint::new(
        "cspr_to_cowl",
        vec![
            Parameter::new("amount", CLType::U512),
            Parameter::new("recipient", CLType::Key),
            Parameter::new("purse", CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // COWL to CSPR swap entry point
    entry_points.add_entry_point(EntryPoint::new(
        "cowl_to_cspr",
        vec![
            Parameter::new("amount", CLType::U512),
            Parameter::new("recipient", CLType::Key),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // Administrative entry points
    entry_points.add_entry_point(EntryPoint::new(
        "update_times",
        vec![
            Parameter::new("new_start_time", CLType::U256),
            Parameter::new("new_end_time", CLType::U256),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "withdraw_cspr",
        vec![Parameter::new("amount", CLType::U512)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "withdraw_cowl",
        vec![Parameter::new("amount", CLType::U512)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // Deposit COWL to contract
    entry_points.add_entry_point(EntryPoint::new(
        "deposit_cowl",
        vec![Parameter::new("amount", CLType::U512)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));


    // Set up named keys
    let mut named_keys = NamedKeys::new();
    named_keys.insert(named_keys::OWNER.to_string(), storage::new_uref(runtime::get_caller()).into());
    named_keys.insert(named_keys::START_TIME.to_string(), storage::new_uref(start_time).into());
    named_keys.insert(named_keys::END_TIME.to_string(), storage::new_uref(end_time).into());
    named_keys.insert(named_keys::COWL_TOKEN.to_string(), storage::new_uref(cowl_token).into());
    named_keys.insert(named_keys::CONTRACT_PURSE.to_string(), contract_purse.into());
    

    // Install contract
    let (contract_hash, _) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(CONTRACT_HASH_NAME.to_string()),
        Some(CONTRACT_ACCESS_UREF_NAME.to_string()),
    );

    // Now store self contract hash after creation
    runtime::put_key(named_keys::SELF_CONTRACT_HASH, storage::new_uref(contract_hash).into());

    // Store contract hash for future reference
    runtime::put_key(CONTRACT_NAME, contract_hash.into());
    runtime::put_key(
        &format!("{}_hash", CONTRACT_NAME),
        storage::new_uref(contract_hash).into(),
    );
    runtime::put_key(
        &format!("{}_version", CONTRACT_NAME),
        storage::new_uref(CONTRACT_VERSION).into(),
    );
}

/// Validate non-zero amount
fn validate_amount(amount: U512) -> Result<(), Error> {
    if amount == U512::zero() {
        return Err(Error::ZeroAmount);
    }
    Ok(())
}

/// Validate rate is non-zero
fn validate_rate(rate: U512) -> Result<(), Error> {
    if rate == U512::zero() {
        return Err(Error::InvalidRate);
    }
    Ok(())
}

/// Validate time window
fn validate_time_window(start_time: u64, end_time: u64) -> Result<(), Error> {
    if end_time <= start_time {
        return Err(Error::InvalidTimeWindow);
    }
    Ok(())
}

/// Function to get contract's COWL balance
fn get_contract_cowl_balance() -> U512 {
    let cowl_token = get_stored_value::<ContractHash>(named_keys::COWL_TOKEN).unwrap_or_revert();
    let contract_hash = get_stored_value::<Key>(named_keys::SELF_CONTRACT_HASH).unwrap_or_revert();

    // No unwrap_or_revert needed here as the return value is already a U512
    runtime::call_contract::<U512>(
        cowl_token,
        "balance_of",
        runtime_args! {
            "address" => contract_hash
        },
    )
}

/// Get swap rate based on CSPR amount
fn get_swap_rate(cspr_amount: U512) -> Result<U512, Error> {
    if cspr_amount < MIN_SWAP_AMOUNT {
        return Err(Error::BelowMinimumSwap);
    }

    // Find appropriate rate tier
    for tier in RATE_TIERS.iter().rev() {
        if cspr_amount >= tier.cspr_amount {
            let rate = tier.rate;
            validate_rate(rate)?;  // Added rate validation
            return Ok(rate);
        }
    }

    // Validate base rate
    let base_rate = RATE_TIERS[0].rate;
    validate_rate(base_rate)?;  // Added rate validation
    Ok(base_rate)
}

/// Verify caller is contract owner
fn verify_caller() -> Result<(), Error> {
    let caller = runtime::get_caller();
    let owner = get_stored_value::<AccountHash>(named_keys::OWNER)?;

    if caller != owner {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

/// Check if swap is active
fn verify_swap_active() -> Result<(), Error> {
    let current_time = runtime::get_blocktime();
    let start_time_millis = get_stored_value::<u64>(named_keys::START_TIME)?;
    let end_time_millis = get_stored_value::<u64>(named_keys::END_TIME)?;

    // Correct conversion: Directly use the milliseconds
    let start_time = BlockTime::new(start_time_millis);
    let end_time = BlockTime::new(end_time_millis);

    if current_time < start_time {
        return Err(Error::SwapNotActive);
    }
    if current_time > end_time {
        return Err(Error::SwapExpired);
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn cspr_to_cowl() {
    let amount: U512 = runtime::get_named_arg("amount");
    let recipient: Key = runtime::get_named_arg("recipient");
    let purse: URef = runtime::get_named_arg("purse");

    // Validate amount
    validate_amount(amount).unwrap_or_revert();

    // Validate recipient
    match recipient {
        Key::Account(_) | Key::Hash(_) => (), // Valid recipients
        _ => runtime::revert(Error::InvalidParameter),
    };

    // Verify swap is active
    verify_swap_active().unwrap_or_revert();

    // Get and validate rate
    let rate = get_swap_rate(amount).unwrap_or_revert();
    validate_rate(rate).unwrap_or_revert();
    
    // Calculate COWL amount and validate
    let cowl_amount = amount * rate;
    validate_amount(cowl_amount).unwrap_or_revert();

    // Check contract's COWL balance
    let contract_cowl_balance = get_contract_cowl_balance();
    if contract_cowl_balance < cowl_amount {
        runtime::revert(Error::InsufficientBalance);
    }

    // Get contract purse and verify
    let contract_purse = get_stored_value_key(named_keys::CONTRACT_PURSE)
        .into_uref()
        .unwrap_or_revert();

    // Verify purse balance before transfer
    let purse_balance = system::get_purse_balance(purse).unwrap_or_revert();
    if purse_balance < amount {
        runtime::revert(Error::InsufficientBalance);
    }

    // Transfer CSPR
    system::transfer_from_purse_to_purse(
        purse,
        contract_purse,
        amount,
        None,
    )
    .unwrap_or_revert();

    let cowl_token = get_stored_value::<ContractHash>(named_keys::COWL_TOKEN).unwrap_or_revert();

    // Convert U512 to U256 with validation
    let amount_bytes = cowl_amount.to_bytes().unwrap_or_revert();
    let cowl_amount_u256 = if amount_bytes.len() <= 32 {
        let mut bytes32 = [0u8; 32];
        bytes32[..amount_bytes.len()].copy_from_slice(&amount_bytes);
        U256::from(bytes32)
    } else {
        runtime::revert(Error::InvalidParameter)
    };

    // Transfer COWL
    runtime::call_contract::<()>(
        cowl_token,
        "transfer",
        runtime_args! {
            "recipient" => recipient,
            "amount" => cowl_amount_u256
        },
    );
}

#[no_mangle]
pub extern "C" fn deposit_cowl() {
    verify_caller().unwrap_or_revert();

    let amount: U512 = runtime::get_named_arg("amount");
    validate_amount(amount).unwrap_or_revert();

    let cowl_token = get_stored_value::<ContractHash>(named_keys::COWL_TOKEN).unwrap_or_revert();
    let contract_hash = get_stored_value::<Key>(named_keys::SELF_CONTRACT_HASH).unwrap_or_revert();

    // Convert U512 to U256
    let amount_bytes = amount.to_bytes().unwrap_or_revert();
    let amount_u256 = if amount_bytes.len() <= 32 {
        let mut bytes32 = [0u8; 32];
        bytes32[..amount_bytes.len()].copy_from_slice(&amount_bytes);
        U256::from(bytes32)
    } else {
        runtime::revert(Error::InvalidParameter)
    };

    // Transfer COWL to contract
    runtime::call_contract::<()>(
        cowl_token,
        "transfer",
        runtime_args! {
            "recipient" => contract_hash,
            "amount" => amount_u256
        },
    );
}

#[no_mangle]
pub extern "C" fn cowl_to_cspr() {
    let amount: U512 = runtime::get_named_arg("amount");
    let recipient: Key = runtime::get_named_arg("recipient");

    // Validate amount
    validate_amount(amount).unwrap_or_revert();

    // Validate recipient is account
    let recipient_account = match recipient {
        Key::Account(account_hash) => account_hash,
        _ => runtime::revert(Error::InvalidParameter),
    };

    // Verify swap is active
    verify_swap_active().unwrap_or_revert();

    // Calculate and validate CSPR amount
    let base_rate = RATE_TIERS[0].rate;
    validate_rate(base_rate).unwrap_or_revert();
    let cspr_amount = amount / base_rate;
    validate_amount(cspr_amount).unwrap_or_revert();

    // Verify minimum swap amount
    if cspr_amount < MIN_SWAP_AMOUNT {
        runtime::revert(Error::BelowMinimumSwap);
    }

    // Calculate tax and validate final amount
    let tax_amount = cspr_amount * TAX_RATE / TAX_DENOMINATOR;
    let final_amount = cspr_amount - tax_amount;
    validate_amount(final_amount).unwrap_or_revert();

    // Verify contract has sufficient CSPR
    let contract_purse = get_stored_value_key(named_keys::CONTRACT_PURSE).into_uref().unwrap_or_revert();
    let contract_balance = system::get_purse_balance(contract_purse).unwrap_or_revert();
    if contract_balance < final_amount {
        runtime::revert(Error::InsufficientBalance);
    }

    // Convert U512 to U256 for COWL token transfer
    let amount_bytes = amount.to_bytes().unwrap_or_revert();
    let amount_u256 = if amount_bytes.len() <= 32 {
        let mut bytes32 = [0u8; 32];
        bytes32[..amount_bytes.len()].copy_from_slice(&amount_bytes);
        U256::from(bytes32)
    } else {
        runtime::revert(Error::InvalidParameter)
    };

    // Get contract info for transfer
    let cowl_token = get_stored_value::<ContractHash>(named_keys::COWL_TOKEN).unwrap_or_revert();
    let contract_key = runtime::get_key(named_keys::SELF_CONTRACT_HASH)
        .unwrap_or_revert();

    // Transfer COWL to contract
    runtime::call_contract::<()>(
        cowl_token,
        "transfer_from",
        runtime_args! {
            "owner" => Key::from(runtime::get_caller()),
            "recipient" => contract_key,
            "amount" => amount_u256
        },
    );

    // Transfer CSPR to recipient
    system::transfer_from_purse_to_account(
        contract_purse,
        recipient_account,
        final_amount,
        None,
    )
    .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn update_times() {
    // Verify caller is owner
    verify_caller().unwrap_or_revert();

    let new_start_time: u64 = runtime::get_named_arg("new_start_time");
    let new_end_time: u64 = runtime::get_named_arg("new_end_time");

    // Validate time window
    validate_time_window(new_start_time, new_end_time).unwrap_or_revert();

    set_key(named_keys::START_TIME, new_start_time);
    set_key(named_keys::END_TIME, new_end_time);
}

#[no_mangle]
pub extern "C" fn withdraw_cspr() {
    // Verify caller is owner
    verify_caller().unwrap_or_revert();

    let amount: U512 = runtime::get_named_arg("amount");
    let contract_purse = get_stored_value_key(named_keys::CONTRACT_PURSE).into_uref().unwrap_or_revert();

    // Verify sufficient balance
    let balance = system::get_purse_balance(contract_purse).unwrap_or_revert();
    if balance < amount {
        runtime::revert(Error::InsufficientBalance);
    }

    // Transfer CSPR to owner
    let owner = get_stored_value::<AccountHash>(named_keys::OWNER).unwrap_or_revert();
    system::transfer_from_purse_to_account(
        contract_purse,
        owner,
        amount,
        None,
    )
    .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn withdraw_cowl() {
    // Verify caller is owner
    verify_caller().unwrap_or_revert();

    let amount: U512 = runtime::get_named_arg("amount");
    let cowl_token = get_stored_value::<ContractHash>(named_keys::COWL_TOKEN).unwrap_or_revert();
    let owner = get_stored_value::<AccountHash>(named_keys::OWNER).unwrap_or_revert();

    // Convert U512 to U256
    let amount_bytes = amount.to_bytes().unwrap_or_revert();
    let amount_u256 = if amount_bytes.len() <= 32 {
        let mut bytes32 = [0u8; 32];
        bytes32[..amount_bytes.len()].copy_from_slice(&amount_bytes);
        U256::from(bytes32)
    } else {
        runtime::revert(Error::InvalidParameter)
    };

    // Transfer COWL to owner
    runtime::call_contract::<()>(
        cowl_token,
        "transfer",
        runtime_args! {
            "recipient" => Key::from(owner),
            "amount" => amount_u256
        },
    );
}

// / Get a stored value
fn get_stored_value<T: CLTyped + FromBytes>(key: &str) -> Result<T, Error> {
    let key = runtime::get_key(key)
        .ok_or(Error::InvalidParameter)?
        .try_into()
        .map_err(|_| Error::InvalidParameter)?;

    storage::read(key)
        .map_err(|_| Error::InvalidParameter)?
        .ok_or(Error::InvalidParameter)
}

/// Get a stored key
fn get_stored_value_key(key: &str) -> Key {
    runtime::get_key(key).unwrap_or_revert()
}

/// Set a key in storage
fn set_key<T: CLTyped + ToBytes>(key: &str, value: T) {
    match runtime::get_key(key) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key_ref = storage::new_uref(value).into();
            runtime::put_key(key, key_ref);
        }
    }
}


