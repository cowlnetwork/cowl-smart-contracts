use alloc::string::String;
use casper_types::{CLTyped, CLValue, Key, U256};
use casper_contract::contract_api::runtime;

#[derive(CLTyped, Clone)]
pub struct TransferEventData {
    pub from: Key,
    pub to: Key,
    pub amount: U256,
    pub timestamp: u64,
}

#[derive(CLTyped, Clone)]
pub struct VestingEventData {
    pub category: String,
    pub amount: U256,
    pub recipient: Key,
    pub timestamp: u64,
}

#[derive(CLTyped, Clone)]
pub struct ApprovalEventData {
    pub owner: Key,
    pub spender: Key,
    pub amount: U256,
    pub timestamp: u64,
}

pub fn emit_ces_event<T: CLTyped + Clone>(event_name: &str, data: T) {
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
