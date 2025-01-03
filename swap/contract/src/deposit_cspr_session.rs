#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");
extern crate alloc;

use casper_contract::{
    contract_api::{
        account::get_main_purse,
        runtime::{call_versioned_contract, get_named_arg},
        system::{create_purse, transfer_from_purse_to_purse},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{runtime_args, ContractPackageHash, Key, RuntimeArgs, U512};
use cowl_swap::{
    constants::{
        ARG_AMOUNT, ARG_COWL_SWAP_CONTRACT_PACKAGE_HASH, ARG_PURSE, ENTRY_POINT_DEPOSIT_CSPR,
    },
    error::SwapError,
};

#[no_mangle]
pub extern "C" fn call() {
    let amount: U512 = get_named_arg(ARG_AMOUNT);

    let cowl_swap_contract_package_hash_key: Key =
        get_named_arg(ARG_COWL_SWAP_CONTRACT_PACKAGE_HASH);

    let cowl_swap_contract_package_hash_key_hash = ContractPackageHash::from(
        cowl_swap_contract_package_hash_key
            .into_hash()
            .unwrap_or_revert_with(SwapError::MissingTokenContractPackage),
    );

    let local_purse = create_purse();
    let source_purse = get_main_purse();

    transfer_from_purse_to_purse(source_purse, local_purse, amount, None).unwrap_or_revert();

    call_versioned_contract(
        cowl_swap_contract_package_hash_key_hash,
        None,
        ENTRY_POINT_DEPOSIT_CSPR,
        runtime_args! {
            ARG_AMOUNT => amount,
            ARG_PURSE => local_purse
        },
    )
}
