#![no_std]
#![no_main]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType,
    EntryPoints, Key, Parameter, RuntimeArgs, URef, U256,
};

// Token Configuration (Immutable)
const TOKEN_NAME: &str = "COWL Token";
const TOKEN_SYMBOL: &str = "COWL";
const TOKEN_DECIMALS: u8 = 18;
const TOTAL_SUPPLY: U256 = U256::from(5_500_000_000_000_000_000_000_000_000u128); // 5.5B with 18 decimals

// Distribution Constants (Immutable)
const TREASURY_AMOUNT: U256 = U256::from(2_750_000_000_000_000_000_000_000_000u128);
const COMMUNITY_STAKING_AMOUNT: U256 = U256::from(1_100_000_000_000_000_000_000_000_000u128);
const INVESTOR_AMOUNT: U256 = U256::from(550_000_000_000_000_000_000_000_000u128);
const TEAM_AMOUNT: U256 = U256::from(385_000_000_000_000_000_000_000_000u128);
const NETWORK_REWARDS_AMOUNT: U256 = U256::from(275_000_000_000_000_000_000_000_000u128);
const MARKETING_AMOUNT: U256 = U256::from(275_000_000_000_000_000_000_000_000u128);
const COMMUNITY_REWARDS_AMOUNT: U256 = U256::from(165_000_000_000_000_000_000_000_000u128);

// Required signatures for multi-sig operations
const REQUIRED_SIGNATURES: u32 = 2;

#[derive(CLTyped, Clone)]
struct VestingSchedule {
    total_amount: U256,
    released_amount: U256,
    start_time: u64,
    duration: u64,
    cliff_duration: u64,
}

#[derive(CLTyped, Clone)]
struct MultiSigProposal {
    proposal_id: String,
    category: String,
    proposed_address: Key,
    signatures: Vec<Key>,
    timestamp: u64,
    executed: bool,
}

// CEP-18 Event structures
#[derive(CLTyped, Clone)]
struct TransferEventData {
    from: Key,
    to: Key,
    amount: U256,
    timestamp: u64,
}

#[derive(CLTyped, Clone)]
struct VestingEventData {
    category: String,
    amount: U256,
    recipient: Key,
    timestamp: u64,
}

// Initialize the contract
#[no_mangle]
pub extern "C" fn call() {
    // Get initial configuration
    let authorized_signers: Vec<Key> = runtime::get_named_arg("authorized_signers");
    let distribution_addresses: Vec<(String, Key)> = runtime::get_named_arg("distribution_addresses");
    
    if authorized_signers.len() < 5 {
        runtime::revert(ApiError::InsufficientSigners);
    }

    // Create storage
    let balances = storage::new_dictionary("balances").unwrap_or_revert();
    let vesting_schedules = storage::new_dictionary("vesting_schedules").unwrap_or_revert();
    let signers = storage::new_dictionary("authorized_signers").unwrap_or_revert();
    let proposals = storage::new_dictionary("multisig_proposals").unwrap_or_revert();

    // Store authorized signers
    for signer in authorized_signers {
        storage::dictionary_put(signers, &signer.to_string(), true);
    }

    // Initialize vesting schedules and distribution
    let start_time = runtime::get_blocktime();
    initialize_distribution(
        distribution_addresses,
        vesting_schedules,
        balances,
        start_time
    );

    // Define entry points
    let mut entry_points = EntryPoints::new();

    // CEP-18 standard entry points
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

    // Multi-sig management entry points
    entry_points.add_entry_point(EntryPoint::new(
        "propose_address_change",
        vec![
            Parameter::new("category", CLType::String),
            Parameter::new("new_address", CLType::Key),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "sign_proposal",
        vec![Parameter::new("proposal_id", CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // Store the contract
    let contract_hash = storage::new_contract(
        entry_points,
        None,
        None,
        None,
    );
}

// Emit CES compatible events
fn emit_ces_event<T: CLTyped + Clone>(event_name: &str, data: T) {
    let event = {
        let mut event_items = Vec::new();
        event_items.push((
            "ces_event_type".to_string(),
            CLValue::from_t("cowl_token_event").unwrap_or_revert()
        ));
        event_items.push((
            "event_name".to_string(),
            CLValue::from_t(event_name).unwrap_or_revert()
        ));
        event_items.push((
            "event_data".to_string(),
            CLValue::from_t(data).unwrap_or_revert()
        ));
        event_items
    };
    runtime::emit(&event);
}

// CEP-18 transfer implementation
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
        runtime::revert(ApiError::InsufficientBalance);
    }

    let recipient_balance: U256 = storage::dictionary_get(balances, &recipient.to_string())
        .unwrap_or_revert()
        .unwrap_or_default();

    // Update balances
    storage::dictionary_put(balances, &sender.to_string(), sender_balance - amount);
    storage::dictionary_put(balances, &recipient.to_string(), recipient_balance + amount);

    // Emit transfer event
    emit_ces_event("transfer", TransferEventData {
        from: Key::from(sender),
        to: recipient,
        amount,
        timestamp: runtime::get_blocktime(),
    });
}

// Multi-sig address change proposal
#[no_mangle]
pub extern "C" fn propose_address_change() {
    let category: String = runtime::get_named_arg("category");
    let new_address: Key = runtime::get_named_arg("new_address");
    let caller = runtime::get_caller();

    // Verify caller is authorized signer
    assert_authorized_signer(&caller);

    let proposal_id = format!("{}_{}", category, runtime::get_blocktime());
    let proposal = MultiSigProposal {
        proposal_id: proposal_id.clone(),
        category,
        proposed_address: new_address,
        signatures: vec![Key::from(caller)],
        timestamp: runtime::get_blocktime(),
        executed: false,
    };

    let proposals = runtime::get_key("multisig_proposals")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();

    storage::dictionary_put(proposals, &proposal_id, proposal);
}

// Multi-sig proposal signing
#[no_mangle]
pub extern "C" fn sign_proposal() {
    let proposal_id: String = runtime::get_named_arg("proposal_id");
    let caller = runtime::get_caller();

    // Verify caller is authorized signer
    assert_authorized_signer(&caller);

    let proposals = runtime::get_key("multisig_proposals")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();

    let mut proposal: MultiSigProposal = storage::dictionary_get(proposals, &proposal_id)
        .unwrap_or_revert()
        .unwrap_or_revert();

    // Check if proposal is still valid (24 hour window)
    if runtime::get_blocktime() > proposal.timestamp + 24 * 60 * 60 * 1000 {
        runtime::revert(ApiError::ProposalExpired);
    }

    // Check if already signed
    if proposal.signatures.contains(&Key::from(caller)) {
        runtime::revert(ApiError::AlreadySigned);
    }

    // Add signature
    proposal.signatures.push(Key::from(caller));

    // Check if enough signatures to execute
    if proposal.signatures.len() >= REQUIRED_SIGNATURES as usize && !proposal.executed {
        execute_address_change(&proposal);
        proposal.executed = true;
    }

    // Update proposal
    storage::dictionary_put(proposals, &proposal_id, proposal);
}

// Execute address change after sufficient signatures
fn execute_address_change(proposal: &MultiSigProposal) {
    let vesting_schedules = runtime::get_key("vesting_schedules")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();

    let mut schedule: VestingSchedule = storage::dictionary_get(vesting_schedules, &proposal.category)
        .unwrap_or_revert()
        .unwrap_or_revert();

    // Update schedule with new address
    storage::dictionary_put(vesting_schedules, &proposal.category, schedule);

    // Emit address change event
    emit_ces_event("address_change", AddressChangeEventData {
        category: proposal.category.clone(),
        new_address: proposal.proposed_address,
        timestamp: runtime::get_blocktime(),
    });
}

// Release vested tokens
#[no_mangle]
pub extern "C" fn release_vested() {
    let category: String = runtime::get_named_arg("category");

    let vesting_schedules = runtime::get_key("vesting_schedules")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();

    let mut schedule: VestingSchedule = storage::dictionary_get(vesting_schedules, &category)
        .unwrap_or_revert()
        .unwrap_or_revert();

    let current_time = runtime::get_blocktime();

    // Check cliff period
    if current_time < schedule.start_time + schedule.cliff_duration {
        runtime::revert(ApiError::CliffPeriodNotEnded);
    }

    // Calculate vested amount
    let vested_amount = calculate_vested_amount(&schedule, current_time);
    let releasable = vested_amount - schedule.released_amount;

    if releasable == U256::zero() {
        runtime::revert(ApiError::NoTokensToRelease);
    }

    // Update released amount
    schedule.released_amount += releasable;
    storage::dictionary_put(vesting_schedules, &category, schedule);

    // Transfer tokens
    let recipient = runtime::get_caller();
    let balances = runtime::get_key("balances")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();

    let recipient_balance: U256 = storage::dictionary_get(balances, &recipient.to_string())
        .unwrap_or_revert()
        .unwrap_or_default();

    storage::dictionary_put(
        balances,
        &recipient.to_string(),
        recipient_balance + releasable
    );

    // Emit vesting release event
    emit_ces_event("vesting_release", VestingReleaseEventData {
        category,
        amount: releasable,
        recipient: Key::from(recipient),
        timestamp: current_time,
    });
}

// Calculate vested amount
fn calculate_vested_amount(schedule: &VestingSchedule, current_time: u64) -> U256 {
    if current_time < schedule.start_time + schedule.cliff_duration {
        return U256::zero();
    }

    if current_time >= schedule.start_time + schedule.duration {
        return schedule.total_amount;
    }

    let time_from_start = current_time - schedule.start_time;
    let vesting_duration = schedule.duration - schedule.cliff_duration;
    let time_after_cliff = time_from_start - schedule.cliff_duration;

    schedule.total_amount * U256::from(time_after_cliff) / U256::from(vesting_duration)
}

// Query functions
#[no_mangle]
pub extern "C" fn get_vesting_schedule() {
    let category: String = runtime::get_named_arg("category");

    let vesting_schedules = runtime::get_key("vesting_schedules")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();

    let schedule: VestingSchedule = storage::dictionary_get(vesting_schedules, &category)
        .unwrap_or_revert()
        .unwrap_or_revert();

    runtime::ret(CLValue::from_t(schedule).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn get_balance() {
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

// Helper functions
fn assert_authorized_signer(caller: &Key) {
    let signers = runtime::get_key("authorized_signers")
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();

    let is_authorized: bool = storage::dictionary_get(signers, &caller.to_string())
        .unwrap_or_revert()
        .unwrap_or_default();

    if !is_authorized {
        runtime::revert(ApiError::UnauthorizedSigner);
    }
}

// Event data structures
#[derive(CLTyped, Clone)]
struct AddressChangeEventData {
    category: String,
    new_address: Key,
    timestamp: u64,
}

#[derive(CLTyped, Clone)]
struct VestingReleaseEventData {
    category: String,
    amount: U256,
    recipient: Key,
    timestamp: u64,
}

// Error handling
#[repr(u16)]
enum ApiError {
    InsufficientBalance = 1,
    CliffPeriodNotEnded = 2,
    UnauthorizedSigner = 3,
    ProposalExpired = 4,
    AlreadySigned = 5,
    NoTokensToRelease = 6,
    InsufficientSigners = 7,
}

// Initialize distribution and vesting schedules
fn initialize_distribution(
    distribution_addresses: Vec<(String, Key)>,
    vesting_schedules: URef,
    balances: URef,
    start_time: u64,
) {
    for (category, address) in distribution_addresses {
        let (amount, cliff_duration, duration) = match category.as_str() {
            "treasury" => (TREASURY_AMOUNT, TWO_YEARS_IN_MS, TWO_YEARS_IN_MS),
            "community_staking" => (COMMUNITY_STAKING_AMOUNT, 0, TEN_YEARS_IN_MS),
            "investor" => (INVESTOR_AMOUNT, 0, TWO_YEARS_IN_MS),
            "team" => (TEAM_AMOUNT, SIX_MONTHS_IN_MS, TWO_YEARS_IN_MS),
            "network_rewards" => (NETWORK_REWARDS_AMOUNT, 0, TWO_YEARS_IN_MS),
            "marketing" => (MARKETING_AMOUNT, 0, TWO_YEARS_IN_MS),
            "community_rewards" => (COMMUNITY_REWARDS_AMOUNT, 0, TWO_YEARS_IN_MS),
            _ => runtime::revert(ApiError::InvalidCategory),
        };

        let schedule = VestingSchedule {
            total_amount: amount,
            released_amount: U256::zero(),
            start_time,
            duration,
            cliff_duration,
        };

        storage::dictionary_put(vesting_schedules, &category, schedule);
        storage::dictionary_put(balances, &address.to_string(), U256::zero());
    }
}
