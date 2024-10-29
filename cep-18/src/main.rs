#![no_std]
#![no_main]

extern crate alloc;

mod token;
mod vesting;
mod multisig;
mod events;
mod error;
mod types;

use alloc::{string::String, vec::Vec};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType,
    EntryPoints, Key, Parameter, RuntimeArgs, URef, U256,
};

use token::TokenState;
use vesting::initialize_vesting;
use multisig::initialize_multisig;
use types::{Distribution, TOTAL_SUPPLY};
use error::Error;

#[no_mangle]
pub extern "C" fn call() {
    // Get initial configuration
    let authorized_signers: Vec<Key> = runtime::get_named_arg("authorized_signers");
    let distribution_addresses: Vec<(String, Key)> = runtime::get_named_arg("distribution_addresses");
    
    if authorized_signers.len() < 5 {
        runtime::revert(Error::InsufficientSigners);
    }

    // Initialize token state
    let token_state = TokenState::new();
    
    // Initialize vesting schedules
    let start_time = runtime::get_blocktime();
    initialize_vesting(&distribution_addresses, start_time);
    
    // Initialize multisig
    initialize_multisig(&authorized_signers);

    // Set up entry points
    let mut entry_points = EntryPoints::new();
    
    // Add token entry points
    token_state.add_entry_points(&mut entry_points);
    
    // Add vesting entry points
    vesting::add_entry_points(&mut entry_points);
    
    // Add multisig entry points
    multisig::add_entry_points(&mut entry_points);

    // Store the contract
    let contract_hash = storage::new_contract(
        entry_points,
        None,
        None,
        None,
    );

    runtime::put_key("contract_hash", contract_hash.into());
}
