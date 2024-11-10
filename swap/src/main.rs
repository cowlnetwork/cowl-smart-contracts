#![no_std]
#![no_main]

//! COWL Ghost Swap Contract
//! 
//! This contract manages the swapping of CSPR to COWL tokens and vice versa.
//! It implements a tiered rate system and time-bound operations.
//! 
//! # Features
//! - Time-bound swap operations with upgradeable duration
//! - Tiered swap rates based on CSPR amount
//! - 10% tax on COWL to CSPR swaps
//! - Minimum swap amount enforcement
//! - Owner-only administrative functions
//! - Secure token management
//!
//! # Rate Tiers
//! - 50,000 CSPR  -> 1:3 ratio
//! - 100,000 CSPR -> 1:4 ratio
//! - 500,000 CSPR -> 1:5 ratio
//! - 1,000,000 CSPR -> 1:6 ratio

extern crate alloc;

// // Add the global allocator
// #[cfg(target_arch = "wasm32")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloc::{string::String,vec, vec::Vec, string::ToString, format};
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::AccountHash,
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys},
    CLType, CLTyped, bytesrepr::{FromBytes, ToBytes}, CLValue, ContractHash, ContractPackageHash, Key, Parameter, RuntimeArgs, U256, U512, ApiError, runtime_args
};
use core::convert::TryInto;

/// Contract name
const CONTRACT_NAME: &str = "cowl_ghost_swap";
/// Contract version
const CONTRACT_VERSION: &str = "1.0.0";
const CONTRACT_HASH_NAME: &str = "cowl_ghost_swap_contract_hash";
const CONTRACT_ACCESS_UREF_NAME: &str = "cowl_ghost_swap_access_uref";
/// Minimum swap amount in CSPR (50,000)
const MIN_SWAP_AMOUNT: U512 = U512([50_000_000_000_000, 0, 0, 0, 0, 0, 0, 0]);
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
    pub const SELF_PACKAGE_HASH: &str = "self_package_hash";
    pub const SELF_CONTRACT_HASH: &str = "self_contract_hash";
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
        rate: U512([3, 0, 0, 0, 0, 0, 0, 0])
    },
    RateTier { 
        cspr_amount: U512([100_000_000_000_000, 0, 0, 0, 0, 0, 0, 0]), 
        rate: U512([4, 0, 0, 0, 0, 0, 0, 0])
    },
    RateTier { 
        cspr_amount: U512([500_000_000_000_000, 0, 0, 0, 0, 0, 0, 0]), 
        rate: U512([5, 0, 0, 0, 0, 0, 0, 0])
    },
    RateTier { 
        cspr_amount: U512([1_000_000_000_000_000, 0, 0, 0, 0, 0, 0, 0]), 
        rate: U512([6, 0, 0, 0, 0, 0, 0, 0])
    },
];

#[no_mangle]
pub extern "C" fn call() {
    // Installation parameters
    let start_time: U256 = runtime::get_named_arg("start_time");
    let end_time: U256 = runtime::get_named_arg("end_time");
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

/// Function to get contract's COWL balance
fn get_contract_cowl_balance() -> U512 {
    let cowl_token = get_stored_value::<ContractHash>(named_keys::COWL_TOKEN).unwrap_or_revert();
    let contract_hash = get_stored_value::<Key>(named_keys::SELF_CONTRACT_HASH).unwrap_or_revert();
    
    runtime::call_contract(
        cowl_token,
        "balance_of",
        runtime_args! {
            "address" => contract_hash
        },
    )
}

/// Function to get contract's CSPR balance
fn get_contract_cspr_balance() -> U512 {
    let cspr_purse = get_stored_value_key(named_keys::CONTRACT_PURSE)
        .into_uref()
        .unwrap_or_revert();
    system::get_purse_balance(cspr_purse).unwrap_or_revert()
}

/// Get the contract's package hash, which is used as the token holder address
fn get_contract_package_hash() -> ContractPackageHash {
    runtime::get_key(CONTRACT_NAME)
        .unwrap_or_revert()
        .into_hash()
        .unwrap_or_revert()
        .into()
}

/// Get swap rate based on CSPR amount
fn get_swap_rate(cspr_amount: U512) -> Result<U512, Error> {
    if cspr_amount < MIN_SWAP_AMOUNT {
        return Err(Error::BelowMinimumSwap);
    }

    // Find appropriate rate tier
    for tier in RATE_TIERS.iter().rev() {
        if cspr_amount >= tier.cspr_amount {
            return Ok(tier.rate);
        }
    }

    // Should never reach here due to minimum amount check
    Ok(RATE_TIERS[0].rate)
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
    let current_time = u64::from(runtime::get_blocktime());
    let start_time = get_stored_value::<u64>(named_keys::START_TIME)?;
    let end_time = get_stored_value::<u64>(named_keys::END_TIME)?;

    if current_time < start_time {
        return Err(Error::SwapNotActive);
    }
    if current_time > end_time {
        return Err(Error::SwapExpired);
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn cspr_to_cowl<T>() {
    let amount: U512 = runtime::get_named_arg("amount");
    let recipient: Key = runtime::get_named_arg("recipient");

    // Verify swap is active
    verify_swap_active().unwrap_or_revert();

    // Get swap rate and calculate COWL amount
    let rate = get_swap_rate(amount).unwrap_or_revert();
    let cowl_amount = amount * rate;

    // Check contract's COWL balance
    let contract_cowl_balance = get_contract_cowl_balance();
    if contract_cowl_balance < cowl_amount {
        runtime::revert(Error::InsufficientBalance);
    }


    // First, accept CSPR into contract's purse
    let contract_purse = get_stored_value_key(named_keys::CONTRACT_PURSE)
        .into_uref()
        .unwrap_or_revert();
    
    system::transfer_from_purse_to_purse(
        runtime::get_named_arg("purse"),
        contract_purse,
        amount,
        None
    ).unwrap_or_revert();

    // Then transfer COWL tokens from contract to recipient
    let cowl_token = get_stored_value::<ContractHash>(named_keys::COWL_TOKEN).unwrap_or_revert();
    runtime::call_contract::<T>(
        cowl_token,
        "transfer",
        runtime_args! {
            "recipient" => recipient,
            "amount" => cowl_amount
        },
    );
}

// Function for owner to deposit COWL tokens to contract
#[no_mangle]
pub extern "C" fn deposit_cowl() {
    verify_caller().unwrap_or_revert();  // Only owner can deposit

    let amount: U512 = runtime::get_named_arg("amount");
    let cowl_token = get_stored_value::<ContractHash>(named_keys::COWL_TOKEN).unwrap_or_revert();
    let contract_hash = get_stored_value::<Key>(named_keys::SELF_CONTRACT_HASH).unwrap_or_revert();
    
    // Transfer COWL tokens from owner to contract
    runtime::call_contract(
        cowl_token,
        "transfer",
        runtime_args! {
            "recipient" => contract_hash,
            "amount" => amount
        },
    );
}

#[no_mangle]
pub extern "C" fn cowl_to_cspr() {
    let amount: U512 = runtime::get_named_arg("amount");
    let recipient: Key = runtime::get_named_arg("recipient");

    // Verify swap is active
    verify_swap_active().unwrap_or_revert();

    // Calculate CSPR amount
    let base_rate = RATE_TIERS[0].rate;
    let cspr_amount = amount / base_rate;

    // Verify minimum swap amount
    if cspr_amount < MIN_SWAP_AMOUNT {
        runtime::revert(Error::BelowMinimumSwap);
    }

    // Calculate tax
    let tax_amount = cspr_amount * TAX_RATE / TAX_DENOMINATOR;
    let final_amount = cspr_amount - tax_amount;

    // Verify contract has sufficient CSPR
    let contract_purse = get_stored_value_key(named_keys::CONTRACT_PURSE).into_uref().unwrap_or_revert();
    let contract_balance = system::get_purse_balance(contract_purse).unwrap_or_revert();
    if contract_balance < final_amount {
        runtime::revert(Error::InsufficientBalance);
    }

    // Transfer COWL tokens to contract
    let cowl_token = get_stored_value::<ContractHash>(named_keys::COWL_TOKEN).unwrap_or_revert();
    runtime::call_contract(
        cowl_token,
        "transfer_from",
        runtime_args! {
            "owner" => Key::from(runtime::get_caller()),
            "recipient" => Key::Contract(runtime::get_blocktime()),
            "amount" => amount
        },
    );

    // Transfer CSPR to recipient
    let recipient_purse = match recipient {
        Key::Account(account_hash) => account_hash,
        _ => runtime::revert(Error::InvalidParameter),
    };
    system::transfer_from_purse_to_account(
        contract_purse,
        recipient_purse,
        final_amount,
        None
    ).unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn update_times() {
    // Verify caller is owner
    verify_caller().unwrap_or_revert();

    let new_start_time: U256 = runtime::get_named_arg("new_start_time");
    let new_end_time: U256 = runtime::get_named_arg("new_end_time");

    // Update times
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
        None
    ).unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn withdraw_cowl() {
    // Verify caller is owner
    verify_caller().unwrap_or_revert();

    let amount: U512 = runtime::get_named_arg("amount");
    let cowl_token = get_stored_value::<ContractHash>(named_keys::COWL_TOKEN).unwrap_or_revert();
    let owner = get_stored_value::<AccountHash>(named_keys::OWNER).unwrap_or_revert();

    // Transfer COWL to owner
    runtime::call_contract(
        cowl_token,
        "transfer",
        runtime_args! {
            "recipient" => Key::from(owner),
            "amount" => amount
        },
    );
}

/// Get a stored value
fn get_stored_value<T: CLTyped + FromBytes>(key: &str) -> Result<T, Error> {
    let uref = runtime::get_key(key)
        .ok_or(Error::InvalidParameter)?
        .into_uref()
        .map_err(|_| Error::InvalidParameter)?;

    storage::read(uref)
        .map_err(|_| Error::InvalidParameter)?
        .ok_or(Error::InvalidParameter)
}

/// Get a stored key
fn get_stored_value_key(key: &str) -> Key {
    runtime::get_key(key).unwrap_or_revert()
}

/// Set a key in storage
fn set_key<T: CLTyped + ToBytes>(key: &str, value: T) {
    runtime::put_key(key, storage::new_uref(value).into());
}

/// Helper function to validate timestamps
fn validate_timestamps(start_time: U256, end_time: U256) -> Result<(), Error> {
    if end_time <= start_time {
        return Err(Error::InvalidParameter);
    }
    Ok(())
}

/// Helper function to check if a purse exists
fn check_purse_exists(purse_key: &str) -> Result<(), Error> {
    runtime::get_key(purse_key)
        .ok_or(Error::InvalidParameter)?
        .into_uref()
        .map_err(|_| Error::InvalidParameter)?;
    Ok(())
}

/// Helper function to verify COWL token contract exists
fn verify_cowl_token() -> Result<ContractHash, Error> {
    get_stored_value::<ContractHash>(named_keys::COWL_TOKEN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use casper_engine_test_support::{Code, SessionBuilder, TestContext, TestContextBuilder};
    use casper_types::{account::AccountHash, runtime_args, RuntimeArgs, U512};

    const CONTRACT_WASM: &str = "contract.wasm";
    const TOKEN_WASM: &str = "erc20_token.wasm";
    
    fn deploy_contract(
        context: &mut TestContext,
        sender: AccountHash,
        start_time: U256,
        end_time: U256,
        cowl_token: ContractHash,
    ) -> ContractHash {
        let session_code = Code::from(CONTRACT_WASM);
        let session_args = runtime_args! {
            "start_time" => start_time,
            "end_time" => end_time,
            "cowl_token" => cowl_token
        };

        let session = SessionBuilder::new(session_code, session_args)
            .with_address(sender)
            .with_authorization_keys(&[sender])
            .build();

        context.run(session);
        let contract_hash = context
            .query(None, &[CONTRACT_NAME.to_string()])
            .unwrap()
            .into_t()
            .unwrap();

        contract_hash
    }

    #[test]
    fn test_swap_initialization() {
        let mut context = TestContextBuilder::new()
            .with_public_key("test_account", U512::from(500_000_000_000_000u64))
            .build();

        // Deploy COWL token first
        let token_code = Code::from(TOKEN_WASM);
        let token_args = runtime_args! {
            "name" => "COWL Token",
            "symbol" => "COWL",
            "decimals" => 9u8,
            "total_supply" => U256::from(1_000_000_000u64)
        };

        let session = SessionBuilder::new(token_code, token_args)
            .with_address(context.get_account("test_account").unwrap())
            .with_authorization_keys(&[context.get_account("test_account").unwrap()])
            .build();

        context.run(session);
        let token_hash: ContractHash = context
            .query(None, &["erc20_token_contract".to_string()])
            .unwrap()
            .into_t()
            .unwrap();

        // Deploy swap contract
        let start_time = U256::from(runtime::get_blocktime());
        let end_time = start_time + U256::from(86400); // 24 hours from now

        let contract_hash = deploy_contract(
            &mut context,
            context.get_account("test_account").unwrap(),
            start_time,
            end_time,
            token_hash,
        );

        // Verify contract installation
        assert!(contract_hash.as_bytes().len() > 0);

        // Verify named keys
        let owner: AccountHash = context
            .query(None, &[CONTRACT_NAME.to_string(), named_keys::OWNER.to_string()])
            .unwrap()
            .into_t()
            .unwrap();
        assert_eq!(owner, context.get_account("test_account").unwrap());

        let stored_start_time: U256 = context
            .query(None, &[CONTRACT_NAME.to_string(), named_keys::START_TIME.to_string()])
            .unwrap()
            .into_t()
            .unwrap();
        assert_eq!(stored_start_time, start_time);

        let stored_end_time: U256 = context
            .query(None, &[CONTRACT_NAME.to_string(), named_keys::END_TIME.to_string()])
            .unwrap()
            .into_t()
            .unwrap();
        assert_eq!(stored_end_time, end_time);
    }

    #[test]
    fn test_cspr_to_cowl_swap() {
        // Test implementation for CSPR to COWL swap
    }

    #[test]
    fn test_cowl_to_cspr_swap() {
        // Test implementation for COWL to CSPR swap
    }

    #[test]
    fn test_update_times() {
        // Test implementation for updating times
    }

    #[test]
    fn test_withdraw_functions() {
        // Test implementation for withdraw functions
    }
}

