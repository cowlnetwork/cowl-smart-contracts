#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");
extern crate alloc;

use casper_contract::{
    contract_api::{
        runtime::{call_versioned_contract, get_named_arg, put_key},
        storage::new_uref,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{runtime_args, ContractPackageHash, Key, RuntimeArgs, U256};
use cowl_swap::constants::{
    ARG_ADDRESS, ARG_BALANCE_COWL, ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH, ENTRY_POINT_BALANCE_OF,
};

#[no_mangle]
pub extern "C" fn call() {
    let cowl_cep18_token_package_hash: ContractPackageHash = ContractPackageHash::new(
        get_named_arg::<Key>(ARG_COWL_CEP18_CONTRACT_PACKAGE_HASH)
            .into_hash()
            .unwrap_or_revert(),
    );
    let address: Key = get_named_arg(ARG_ADDRESS);

    let balance_args = runtime_args! {
        ARG_ADDRESS => address,
    };
    let result: U256 = call_versioned_contract(
        cowl_cep18_token_package_hash,
        None,
        ENTRY_POINT_BALANCE_OF,
        balance_args,
    );
    let new_uref = new_uref(result);
    put_key(ARG_BALANCE_COWL, new_uref.into());
}
