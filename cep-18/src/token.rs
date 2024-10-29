use alloc::string::String;
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    CLType, EntryPoint, EntryPointAccess, EntryPointType,
    EntryPoints, Key, Parameter, U256, URef, CLValue,
};

use crate::{
    types::{TOKEN_NAME, TOKEN_SYMBOL, TOKEN_DECIMALS, TOTAL_SUPPLY},
    events::{emit_ces_event, TransferEventData, ApprovalEventData},
    error::Error,
};

pub struct TokenState {
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: U256,
    balances: URef,
    allowances: URef,
}

impl TokenState {
    pub fn new() -> Self {
        let balances = storage::new_dictionary("balances").unwrap_or_revert();
        let allowances = storage::new_dictionary("allowances").unwrap_or_revert();
        
        runtime::put_key("balances", balances.into());
        runtime::put_key("allowances", allowances.into());
        runtime::put_key("name", storage::new_uref(TOKEN_NAME).into());
        runtime::put_key("symbol", storage::new_uref(TOKEN_SYMBOL).into());
        runtime::put_key("decimals", storage::new_uref(TOKEN_DECIMALS).into());
        runtime::put_key("total_supply", storage::new_uref(TOTAL_SUPPLY).into());
        
        Self {
            name: TOKEN_NAME.to_string(),
            symbol: TOKEN_SYMBOL.to_string(),
            decimals: TOKEN_DECIMALS,
            total_supply: TOTAL_SUPPLY,
            balances,
            allowances,
        }
    }

    pub fn add_entry_points(entry_points: &mut EntryPoints) {
        // Required CEP-18 entry points
        entry_points.add_entry_point(EntryPoint::new(
            "name",
            Vec::new(),
            CLType::String,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));

        entry_points.add_entry_point(EntryPoint::new(
            "symbol",
            Vec::new(),
            CLType::String,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));

        entry_points.add_entry_point(EntryPoint::new(
            "decimals",
            Vec::new(),
            CLType::U8,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));

        entry_points.add_entry_point(EntryPoint::new(
            "total_supply",
            Vec::new(),
            CLType::U256,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));

        entry_points.add_entry_point(EntryPoint::new(
            "balance_of",
            vec![Parameter::new("address", CLType::Key)],
            CLType::U256,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));

        entry_points.add_entry_point(EntryPoint::new(
            "transfer",
            vec![
                Parameter::new("recipient", CLType::Key),
                Parameter::new("amount", CLType::U256),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));

        entry_points.add_entry_point(EntryPoint::new(
            "approve",
            vec![
                Parameter::new("spender", CLType::Key),
                Parameter::new("amount", CLType::U256),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));

        entry_points.add_entry_point(EntryPoint::new(
            "allowance",
            vec![
                Parameter::new("owner", CLType::Key),
                Parameter::new("spender", CLType::Key),
            ],
            CLType::U256,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));

        entry_points.add_entry_point(EntryPoint::new(
            "transfer_from",
            vec![
                Parameter::new("owner", CLType::Key),
                Parameter::new("recipient", CLType::Key),
                Parameter::new("amount", CLType::U256),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));

        entry_points.add_entry_point(EntryPoint::new(
            "meta",
            Vec::new(),
            CLType::Any,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ));
    }
}

// Required CEP-18 functions
#[no_mangle]
pub extern "C" fn name() {
    let name: String = runtime::get_key("name")
        .unwrap_or_revert()
        .into_string()
        .unwrap_or_revert();
    runtime::ret(CLValue::from_t(name).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn symbol() {
    let symbol: String = runtime::get_key("symbol")
        .unwrap_or_revert()
        .into_string()
        .unwrap_or_revert();
    runtime::ret(CLValue::from_t(symbol).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn decimals() {
    let decimals: u8 = runtime::get_key("decimals")
        .unwrap_or_revert()
        .into_u8()
        .unwrap_or_revert();
    runtime::ret(CLValue::from_t(decimals).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn total_supply() {
    let total_supply: U256 = runtime::get_key("total_supply")
        .unwrap_or_revert()
        .into_u256()
        .unwrap_or_revert();
    runtime::ret(CLValue::from_t(total_supply).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn balance_of() {
    let address: Key = runtime::get_named_arg("address");
    let balances = runtime::get_key("balances")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();
    
    let balance: U256 = storage::dictionary_get(balances, &address.to_string())
        .unwrap_or_revert()
        .unwrap_or_default();
    
    runtime::ret(CLValue::from_t(balance).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn transfer() {
    let recipient: Key = runtime::get_named_arg("recipient");
    let amount: U256 = runtime::get_named_arg("amount");
    let sender = runtime::get_caller();

    let balances = runtime::get_key("balances")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();

    let sender_balance: U256 = storage::dictionary_get(balances, &sender.to_string())
        .unwrap_or_revert()
        .unwrap_or_default();

    if sender_balance < amount {
        runtime::revert(Error::InsufficientBalance);
    }

    let recipient_balance: U256 = storage::dictionary_get(balances, &recipient.to_string())
        .unwrap_or_revert()
        .unwrap_or_default();

    storage::dictionary_put(balances, &sender.to_string(), sender_balance - amount);
    storage::dictionary_put(balances, &recipient.to_string(), recipient_balance + amount);

    emit_ces_event(
        "transfer",
        TransferEventData {
            from: Key::from(sender),
            to: recipient,
            amount,
            timestamp: runtime::get_blocktime(),
        },
    );
}

#[no_mangle]
pub extern "C" fn approve() {
    let spender: Key = runtime::get_named_arg("spender");
    let amount: U256 = runtime::get_named_arg("amount");
    let owner = runtime::get_caller();

    let allowances = runtime::get_key("allowances")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();

    let allowance_key = format!("{}:{}", owner, spender);
    storage::dictionary_put(allowances, &allowance_key, amount);

    emit_ces_event(
        "approval",
        ApprovalEventData {
            owner: Key::from(owner),
            spender,
            amount,
            timestamp: runtime::get_blocktime(),
        },
    );
}

#[no_mangle]
pub extern "C" fn allowance() {
    let owner: Key = runtime::get_named_arg("owner");
    let spender: Key = runtime::get_named_arg("spender");

    let allowances = runtime::get_key("allowances")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();

    let allowance_key = format!("{}:{}", owner, spender);
    let amount: U256 = storage::dictionary_get(allowances, &allowance_key)
        .unwrap_or_revert()
        .unwrap_or_default();

    runtime::ret(CLValue::from_t(amount).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn transfer_from() {
    let owner: Key = runtime::get_named_arg("owner");
    let recipient: Key = runtime::get_named_arg("recipient");
    let amount: U256 = runtime::get_named_arg("amount");
    let spender = runtime::get_caller();

    // Check allowance
    let allowances = runtime::get_key("allowances")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();

    let allowance_key = format!("{}:{}", owner, spender);
    let allowance: U256 = storage::dictionary_get(allowances, &allowance_key)
        .unwrap_or_revert()
        .unwrap_or_default();

    if allowance < amount {
        runtime::revert(Error::InsufficientAllowance);
    }

    // Check balance
    let balances = runtime::get_key("balances")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();

    let owner_balance: U256 = storage::dictionary_get(balances, &owner.to_string())
        .unwrap_or_revert()
        .unwrap_or_default();

    if owner_balance < amount {
        runtime::revert(Error::InsufficientBalance);
    }

    // Update allowance
    storage::dictionary_put(allowances, &allowance_key, allowance - amount);

    // Transfer tokens
    let recipient_balance: U256 = storage::dictionary_get(balances, &recipient.to_string())
        .unwrap_or_revert()
        .unwrap_or_default();

    storage::dictionary_put(balances, &owner.to_string(), owner_balance - amount);
    storage::dictionary_put(balances, &recipient.to_string(), recipient_balance + amount);

    // Emit transfer event
    emit_ces_event(
        "transfer",
        TransferEventData {
            from: owner,
            to: recipient,
            amount,
            timestamp: runtime::get_blocktime(),
        },
    );
}

#[no_mangle]
pub extern "C" fn meta() {
    let metadata = alloc::vec![
        ("name".to_string(), TOKEN_NAME.to_string()),
        ("symbol".to_string(), TOKEN_SYMBOL.to_string()),
        ("decimals".to_string(), TOKEN_DECIMALS.to_string()),
        ("total_supply".to_string(), TOTAL_SUPPLY.to_string()),
    ];
    runtime::ret(CLValue::from_t(metadata).unwrap_or_revert());
}