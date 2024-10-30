#![no_std]
#![no_main]

extern crate alloc;

mod allowances;
mod balances;
pub mod constants;
pub mod entry_points;
mod error;
mod events;
mod modalities;
mod utils;
mod vesting;
use vesting::{
    check_vesting_transfer,
    get_treasury_vesting_details,
    get_team_vesting_details,
    get_staking_vesting_details,
    get_investor_vesting_details,
    get_network_vesting_details,
    get_marketing_vesting_details,
    get_airdrop_vesting_details,
    calculate_vesting_allocations,
};

use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use allowances::{get_allowances_uref, read_allowance_from, write_allowance_to};
use balances::{get_balances_uref, read_balance_from, transfer_balance, write_balance_to};
use entry_points::generate_entry_points;

use casper_contract::{
    contract_api::{
        runtime::{self, get_caller, get_key, get_named_arg, put_key, revert},
        storage::{self, dictionary_put},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::ToBytes, contracts::NamedKeys, runtime_args, CLValue, ContractHash,
    ContractPackageHash, Key, RuntimeArgs, U256
};

use constants::{
    ACCESS_KEY_NAME_PREFIX, ADDRESS, ADMIN_LIST, ALLOWANCES, AMOUNT, BALANCES,
    CONTRACT_NAME_PREFIX, CONTRACT_VERSION_PREFIX, DECIMALS, ENABLE_MINT_BURN, EVENTS_MODE,
    HASH_KEY_NAME_PREFIX, INIT_ENTRY_POINT_NAME, MINTER_LIST, NAME, NONE_LIST, OWNER, PACKAGE_HASH,
    RECIPIENT, SECURITY_BADGES, SPENDER, SYMBOL, TOTAL_SUPPLY, TREASURY_ADDRESS, TEAM_ADDRESS, STAKING_ADDRESS,
    INVESTOR_ADDRESS, MARKETING_ADDRESS, NETWORK_ADDRESS, AIRDROP_ADDRESS
};
pub use error::Cep18Error;
use events::{
    init_events, Burn, ChangeSecurity, DecreaseAllowance, Event, IncreaseAllowance, Mint,
    SetAllowance, Transfer, TransferFrom,
};
use utils::{
    get_immediate_caller_address, get_total_supply_uref, read_from, read_total_supply_from,
    sec_check, write_total_supply_to, SecurityBadge,
};

#[no_mangle]
pub extern "C" fn name() {
    runtime::ret(CLValue::from_t(utils::read_from::<String>(NAME)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn symbol() {
    runtime::ret(CLValue::from_t(utils::read_from::<String>(SYMBOL)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn decimals() {
    runtime::ret(CLValue::from_t(utils::read_from::<u8>(DECIMALS)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn total_supply() {
    runtime::ret(CLValue::from_t(utils::read_from::<U256>(TOTAL_SUPPLY)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn balance_of() {
    let address: Key = runtime::get_named_arg(ADDRESS);
    let balances_uref = get_balances_uref();
    let balance = balances::read_balance_from(balances_uref, address);
    runtime::ret(CLValue::from_t(balance).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn allowance() {
    let spender: Key = runtime::get_named_arg(SPENDER);
    let owner: Key = runtime::get_named_arg(OWNER);
    let allowances_uref = get_allowances_uref();
    let val: U256 = read_allowance_from(allowances_uref, owner, spender);
    runtime::ret(CLValue::from_t(val).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn approve() {
    let owner = utils::get_immediate_caller_address().unwrap_or_revert();
    let spender: Key = runtime::get_named_arg(SPENDER);
    if spender == owner {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(AMOUNT);
    let allowances_uref = get_allowances_uref();
    write_allowance_to(allowances_uref, owner, spender, amount);
    events::record_event_dictionary(Event::SetAllowance(SetAllowance {
        owner,
        spender,
        allowance: amount,
    }))
}

#[no_mangle]
pub extern "C" fn decrease_allowance() {
    let owner = utils::get_immediate_caller_address().unwrap_or_revert();
    let spender: Key = runtime::get_named_arg(SPENDER);
    if spender == owner {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(AMOUNT);
    let allowances_uref = get_allowances_uref();
    let current_allowance = read_allowance_from(allowances_uref, owner, spender);
    let new_allowance = current_allowance.saturating_sub(amount);
    write_allowance_to(allowances_uref, owner, spender, new_allowance);
    events::record_event_dictionary(Event::DecreaseAllowance(DecreaseAllowance {
        owner,
        spender,
        decr_by: amount,
        allowance: new_allowance,
    }))
}

#[no_mangle]
pub extern "C" fn increase_allowance() {
    let owner = utils::get_immediate_caller_address().unwrap_or_revert();
    let spender: Key = runtime::get_named_arg(SPENDER);
    if spender == owner {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(AMOUNT);
    let allowances_uref = get_allowances_uref();
    let current_allowance = read_allowance_from(allowances_uref, owner, spender);
    let new_allowance = current_allowance.saturating_add(amount);
    write_allowance_to(allowances_uref, owner, spender, new_allowance);
    events::record_event_dictionary(Event::IncreaseAllowance(IncreaseAllowance {
        owner,
        spender,
        allowance: new_allowance,
        inc_by: amount,
    }))
}

#[no_mangle]
pub extern "C" fn transfer() {
    let sender = utils::get_immediate_caller_address().unwrap_or_revert();
    let recipient: Key = runtime::get_named_arg(RECIPIENT);
    if sender == recipient {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(AMOUNT);

    // Check vesting restrictions
    if !check_vesting_transfer(sender, amount) {
        revert(Cep18Error::VestingLocked);
    }

    transfer_balance(sender, recipient, amount).unwrap_or_revert();
    events::record_event_dictionary(Event::Transfer(Transfer {
        sender,
        recipient,
        amount,
    }))
}

#[no_mangle]
pub extern "C" fn transfer_from() {
    let spender = utils::get_immediate_caller_address().unwrap_or_revert();
    let recipient: Key = runtime::get_named_arg(RECIPIENT);
    let owner: Key = runtime::get_named_arg(OWNER);
    if owner == recipient {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(AMOUNT);
    if amount.is_zero() {
        return;
    }

    let allowances_uref = get_allowances_uref();
    let spender_allowance: U256 = read_allowance_from(allowances_uref, owner, spender);
    let new_spender_allowance = spender_allowance
        .checked_sub(amount)
        .ok_or(Cep18Error::InsufficientAllowance)
        .unwrap_or_revert();

    transfer_balance(owner, recipient, amount).unwrap_or_revert();
    write_allowance_to(allowances_uref, owner, spender, new_spender_allowance);
    events::record_event_dictionary(Event::TransferFrom(TransferFrom {
        spender,
        owner,
        recipient,
        amount,
    }))
}

#[no_mangle]
pub extern "C" fn mint() {
    if 0 == read_from::<u8>(ENABLE_MINT_BURN) {
        revert(Cep18Error::MintBurnDisabled);
    }

    sec_check(vec![SecurityBadge::Admin, SecurityBadge::Minter]);

    let owner: Key = runtime::get_named_arg(OWNER);
    let amount: U256 = runtime::get_named_arg(AMOUNT);

    let balances_uref = get_balances_uref();
    let total_supply_uref = get_total_supply_uref();
    let new_balance = {
        let balance = read_balance_from(balances_uref, owner);
        balance
            .checked_add(amount)
            .ok_or(Cep18Error::Overflow)
            .unwrap_or_revert()
    };
    let new_total_supply = {
        let total_supply: U256 = read_total_supply_from(total_supply_uref);
        total_supply
            .checked_add(amount)
            .ok_or(Cep18Error::Overflow)
            .unwrap_or_revert()
    };
    write_balance_to(balances_uref, owner, new_balance);
    write_total_supply_to(total_supply_uref, new_total_supply);
    events::record_event_dictionary(Event::Mint(Mint {
        recipient: owner,
        amount,
    }))
}

#[no_mangle]
pub extern "C" fn burn() {
    if 0 == read_from::<u8>(ENABLE_MINT_BURN) {
        revert(Cep18Error::MintBurnDisabled);
    }

    let owner: Key = runtime::get_named_arg(OWNER);

    if owner != get_immediate_caller_address().unwrap_or_revert() {
        revert(Cep18Error::InvalidBurnTarget);
    }

    let amount: U256 = runtime::get_named_arg(AMOUNT);
    let balances_uref = get_balances_uref();
    let total_supply_uref = get_total_supply_uref();
    let new_balance = {
        let balance = read_balance_from(balances_uref, owner);
        balance
            .checked_sub(amount)
            .ok_or(Cep18Error::InsufficientBalance)
            .unwrap_or_revert()
    };
    let new_total_supply = {
        let total_supply = read_total_supply_from(total_supply_uref);
        total_supply
            .checked_sub(amount)
            .ok_or(Cep18Error::Overflow)
            .unwrap_or_revert()
    };
    write_balance_to(balances_uref, owner, new_balance);
    write_total_supply_to(total_supply_uref, new_total_supply);
    events::record_event_dictionary(Event::Burn(Burn { owner, amount }))
}

/// Initiates the contracts states. Only used by the installer call,
/// later calls will cause it to revert.
#[no_mangle]
pub extern "C" fn init() {
    if get_key(ALLOWANCES).is_some() {
        revert(Cep18Error::AlreadyInitialized);
    }
    let package_hash = get_named_arg::<Key>(PACKAGE_HASH);
    put_key(PACKAGE_HASH, package_hash);
    storage::new_dictionary(ALLOWANCES).unwrap_or_revert();
    let balances_uref = storage::new_dictionary(BALANCES).unwrap_or_revert();
    let initial_supply:U256 = runtime::get_named_arg(TOTAL_SUPPLY);
    
    // Get all vesting addresses
    let treasury_address: Key = runtime::get_named_arg(TREASURY_ADDRESS);
    let team_address: Key = runtime::get_named_arg(TEAM_ADDRESS);
    let staking_address: Key = runtime::get_named_arg(STAKING_ADDRESS);
    let investor_address: Key = runtime::get_named_arg(INVESTOR_ADDRESS);
    let network_address: Key = runtime::get_named_arg(NETWORK_ADDRESS);
    let marketing_address: Key = runtime::get_named_arg(MARKETING_ADDRESS);
    let airdrop_address: Key = runtime::get_named_arg(AIRDROP_ADDRESS);

    let caller = get_caller();
    
    let allocations = calculate_vesting_allocations(
        initial_supply,
        treasury_address,
        team_address,
        staking_address,
        investor_address,
        network_address,
        marketing_address,
        airdrop_address,
    );

    // Write initial balances and record events
    for allocation in allocations {
        write_balance_to(balances_uref, allocation.address, allocation.amount);
        
        events::record_event_dictionary(Event::Transfer(Transfer {
            sender: Key::from(caller),
            recipient: allocation.address,
            amount: allocation.amount,
        }));
    }

    let security_badges_dict = storage::new_dictionary(SECURITY_BADGES).unwrap_or_revert();
    dictionary_put(
        security_badges_dict,
        &base64::encode(Key::from(get_caller()).to_bytes().unwrap_or_revert()),
        SecurityBadge::Admin,
    );

    let admin_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(ADMIN_LIST, Cep18Error::InvalidAdminList);
    let minter_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(MINTER_LIST, Cep18Error::InvalidMinterList);

    init_events();

    if let Some(minter_list) = minter_list {
        for minter in minter_list {
            dictionary_put(
                security_badges_dict,
                &base64::encode(minter.to_bytes().unwrap_or_revert()),
                SecurityBadge::Minter,
            );
        }
    }
    if let Some(admin_list) = admin_list {
        for admin in admin_list {
            dictionary_put(
                security_badges_dict,
                &base64::encode(admin.to_bytes().unwrap_or_revert()),
                SecurityBadge::Admin,
            );
        }
    }

}

/// Admin EntryPoint to manipulate the security access granted to users.
/// One user can only possess one access group badge.
/// Change strength: None > Admin > Minter
/// Change strength meaning by example: If user is added to both Minter and Admin they will be an
/// Admin, also if a user is added to Admin and None then they will be removed from having rights.
/// Beware: do not remove the last Admin because that will lock out all admin functionality.
#[no_mangle]
pub extern "C" fn change_security() {
    if 0 == read_from::<u8>(ENABLE_MINT_BURN) {
        revert(Cep18Error::MintBurnDisabled);
    }
    sec_check(vec![SecurityBadge::Admin]);
    let admin_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(ADMIN_LIST, Cep18Error::InvalidAdminList);
    let minter_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(MINTER_LIST, Cep18Error::InvalidMinterList);
    let none_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(NONE_LIST, Cep18Error::InvalidNoneList);

    let mut badge_map: BTreeMap<Key, SecurityBadge> = BTreeMap::new();
    if let Some(minter_list) = minter_list {
        for account_key in minter_list {
            badge_map.insert(account_key, SecurityBadge::Minter);
        }
    }
    if let Some(admin_list) = admin_list {
        for account_key in admin_list {
            badge_map.insert(account_key, SecurityBadge::Admin);
        }
    }
    if let Some(none_list) = none_list {
        for account_key in none_list {
            badge_map.insert(account_key, SecurityBadge::None);
        }
    }

    let caller = get_immediate_caller_address().unwrap_or_revert();
    badge_map.remove(&caller);

    utils::change_sec_badge(&badge_map);
    events::record_event_dictionary(Event::ChangeSecurity(ChangeSecurity {
        admin: get_immediate_caller_address().unwrap_or_revert(),
        sec_change_map: badge_map,
    }));
}

#[no_mangle]
pub extern "C" fn vesting_details() {
    let vesting_type: String = runtime::get_named_arg("vesting_type");
    
    let result = match vesting_type.as_str() {
        "treasury" => get_treasury_vesting_details(),
        "team" => get_team_vesting_details(),
        "staking" => get_staking_vesting_details(),
        "investor" => get_investor_vesting_details(),
        "network" => get_network_vesting_details(),
        "marketing" => get_marketing_vesting_details(),
        "airdrop" => get_airdrop_vesting_details(),
        _ => runtime::revert(Cep18Error::InvalidVestingType),
    };
    
    runtime::ret(CLValue::from_t(result).unwrap_or_revert());
}

// Helper function to calculate percentage of total supply
fn calculate_token_amount(initial_supply: U256, percentage: u8) -> U256 {
    if percentage == 50 {
        // Special case for 50% to avoid multiplication
        initial_supply
            .checked_div(U256::from(2))
            .unwrap_or_revert_with(Cep18Error::Overflow)
    } else {
        initial_supply
            .checked_mul(U256::from(percentage))
            .unwrap_or_revert_with(Cep18Error::Overflow)
            .checked_div(U256::from(100))
            .unwrap_or_revert_with(Cep18Error::Overflow)
    }
}

pub fn upgrade(name: &str) {
    let entry_points = generate_entry_points();

    let contract_package_hash = runtime::get_key(&format!("{HASH_KEY_NAME_PREFIX}{name}"))
        .unwrap_or_revert()
        .into_hash()
        .map(ContractPackageHash::new)
        .unwrap_or_revert_with(Cep18Error::MissingPackageHashForUpgrade);

    let previous_contract_hash = runtime::get_key(&format!("{CONTRACT_NAME_PREFIX}{name}"))
        .unwrap_or_revert()
        .into_hash()
        .map(ContractHash::new)
        .unwrap_or_revert_with(Cep18Error::MissingPackageHashForUpgrade);

    let (contract_hash, contract_version) =
        storage::add_contract_version(contract_package_hash, entry_points, NamedKeys::new());

    storage::disable_contract_version(contract_package_hash, previous_contract_hash)
        .unwrap_or_revert();
    runtime::put_key(
        &format!("{CONTRACT_NAME_PREFIX}{name}"),
        contract_hash.into(),
    );
    runtime::put_key(
        &format!("{CONTRACT_VERSION_PREFIX}{name}"),
        storage::new_uref(contract_version).into(),
    );
}

pub fn install_contract(name: &str) {
    let symbol: String = runtime::get_named_arg(SYMBOL);
    let decimals: u8 = runtime::get_named_arg(DECIMALS);
    let total_supply: U256 = runtime::get_named_arg(TOTAL_SUPPLY);
    let events_mode: u8 =
        utils::get_optional_named_arg_with_user_errors(EVENTS_MODE, Cep18Error::InvalidEventsMode)
            .unwrap_or(0u8);

    // Get the vesting addresses that will be passed to init
    let treasury_address: Key = runtime::get_named_arg(TREASURY_ADDRESS);
    let team_address: Key = runtime::get_named_arg(TEAM_ADDRESS);
    let staking_address: Key = runtime::get_named_arg(STAKING_ADDRESS);
    let investor_address: Key = runtime::get_named_arg(INVESTOR_ADDRESS);
    let network_address: Key = runtime::get_named_arg(NETWORK_ADDRESS);
    let marketing_address: Key = runtime::get_named_arg(MARKETING_ADDRESS);
    let airdrop_address: Key = runtime::get_named_arg(AIRDROP_ADDRESS);

    let admin_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(ADMIN_LIST, Cep18Error::InvalidAdminList);
    let minter_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(MINTER_LIST, Cep18Error::InvalidMinterList);

    let enable_mint_burn: u8 = utils::get_optional_named_arg_with_user_errors(
        ENABLE_MINT_BURN,
        Cep18Error::InvalidEnableMBFlag,
    )
    .unwrap_or(0);

    let mut named_keys = NamedKeys::new();

    named_keys.insert(NAME.to_string(), storage::new_uref(name).into());
    named_keys.insert(SYMBOL.to_string(), storage::new_uref(symbol).into());
    named_keys.insert(DECIMALS.to_string(), storage::new_uref(decimals).into());
    named_keys.insert(TOTAL_SUPPLY.to_string(), storage::new_uref(total_supply).into(),);
    named_keys.insert(EVENTS_MODE.to_string(),storage::new_uref(events_mode).into(),);
    named_keys.insert(ENABLE_MINT_BURN.to_string(),storage::new_uref(enable_mint_burn).into(),);
    
    let entry_points = generate_entry_points();

    let hash_key_name = format!("{HASH_KEY_NAME_PREFIX}{name}");

    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(hash_key_name.clone()),
        Some(format!("{ACCESS_KEY_NAME_PREFIX}{name}")),
    );
    let package_hash = runtime::get_key(&hash_key_name).unwrap_or_revert();

    // Store contract_hash and contract_version under the keys CONTRACT_NAME and CONTRACT_VERSION
    runtime::put_key(
        &format!("{CONTRACT_NAME_PREFIX}{name}"),
        contract_hash.into(),
    );
    runtime::put_key(
        &format!("{CONTRACT_VERSION_PREFIX}{name}"),
        storage::new_uref(contract_version).into(),
    );
    // Call contract to initialize it
    // Update init_args to include vesting addresses
    let mut init_args = runtime_args! {
        TOTAL_SUPPLY => total_supply,
        PACKAGE_HASH => package_hash,
        TREASURY_ADDRESS => treasury_address,
        TEAM_ADDRESS => team_address,
        STAKING_ADDRESS => staking_address,
        INVESTOR_ADDRESS => investor_address,
        NETWORK_ADDRESS => network_address,
        MARKETING_ADDRESS => marketing_address,
        AIRDROP_ADDRESS => airdrop_address
    };

    if let Some(admin_list) = admin_list {
        init_args.insert(ADMIN_LIST, admin_list).unwrap_or_revert();
    }
    if let Some(minter_list) = minter_list {
        init_args
            .insert(MINTER_LIST, minter_list)
            .unwrap_or_revert();
    }

    runtime::call_contract::<()>(contract_hash, INIT_ENTRY_POINT_NAME, init_args);
}

#[no_mangle]
pub extern "C" fn call() {
    let name: String = runtime::get_named_arg(NAME);
    match runtime::get_key(&format!("{ACCESS_KEY_NAME_PREFIX}{name}")) {
        Some(_) => {
            upgrade(&name);
        }
        None => {
            install_contract(&name);
        }
    }
}