use crate::security::SecurityBadge;
#[cfg(feature = "contract-support")]
use crate::{constants::ARG_EVENTS_MODE, enums::EventsMode, utils::get_stored_value};
use alloc::collections::btree_map::BTreeMap;
#[cfg(feature = "contract-support")]
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_event_standard::Event;
#[cfg(feature = "contract-support")]
use casper_event_standard::{emit, Schemas};
use casper_types::{Key, URef, U256, U512};
#[cfg(feature = "contract-support")]
use core::convert::TryFrom;

#[derive(Debug)]
pub enum Event {
    ChangeSecurity(ChangeSecurity),
    SetModalities(SetModalities),
    Upgrade(Upgrade),
    CowlCep18ContractPackageUpdate(CowlCep18ContractPackageUpdate),
    UpdateTimes(UpdateTimes),
    DepositCspr(DepositCspr),
    WithdrawCowl(WithdrawCowl),
    WithdrawCspr(WithdrawCspr),
    CowlToCspr(CowlToCspr),
    CsprToCowl(CsprToCowl),
}

#[cfg(feature = "contract-support")]
pub fn record_event_dictionary(event: Event) {
    let events_mode: EventsMode =
        EventsMode::try_from(get_stored_value::<u8>(ARG_EVENTS_MODE)).unwrap_or_revert();

    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CES => ces(event),
    }
}

#[derive(Event, Debug, PartialEq, Eq, Default)]
pub struct SetModalities {}

impl SetModalities {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Event, Debug, PartialEq, Eq, Default)]
pub struct Upgrade {}

impl Upgrade {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct ChangeSecurity {
    pub admin: Key,
    pub sec_change_map: BTreeMap<Key, SecurityBadge>,
}

impl ChangeSecurity {
    pub fn new(admin: Key, sec_change_map: BTreeMap<Key, SecurityBadge>) -> Self {
        Self {
            admin,
            sec_change_map,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct CowlCep18ContractPackageUpdate {
    pub key: Key,
    pub cowl_cep18_contract_package_key: Key,
}

impl CowlCep18ContractPackageUpdate {
    pub fn new(key: Key, cowl_cep18_contract_package_key: Key) -> Self {
        Self {
            key,
            cowl_cep18_contract_package_key,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq, Default)]
pub struct UpdateTimes {
    pub new_start_time: u64,
    pub new_end_time: u64,
}

impl UpdateTimes {
    pub fn new(new_start_time: u64, new_end_time: u64) -> Self {
        Self {
            new_start_time,
            new_end_time,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq, Default)]
pub struct DepositCspr {
    pub source_purse: URef,
    pub amount: U512,
}

impl DepositCspr {
    pub fn new(source_purse: URef, amount: U512) -> Self {
        Self {
            source_purse,
            amount,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct WithdrawCowl {
    pub recipient: Key,
    pub amount: U256,
}

impl WithdrawCowl {
    pub fn new(recipient: Key, amount: U256) -> Self {
        Self { recipient, amount }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct WithdrawCspr {
    pub recipient: Key,
    pub amount: U512,
}

impl WithdrawCspr {
    pub fn new(recipient: Key, amount: U512) -> Self {
        Self { recipient, amount }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct CowlToCspr {
    pub owner: Key,
    pub recipient: Key,
    pub cowl_amount: U256,
    pub cspr_amount: U512,
    pub base_rate: U512,
    pub tax_amount: U512,
}

impl CowlToCspr {
    pub fn new(
        owner: Key,
        recipient: Key,
        cowl_amount: U256,
        cspr_amount: U512,
        base_rate: U512,
        tax_amount: U512,
    ) -> Self {
        Self {
            owner,
            recipient,
            cowl_amount,
            cspr_amount,
            base_rate,
            tax_amount,
        }
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct CsprToCowl {
    pub source_purse: URef,
    pub recipient: Key,
    pub cowl_amount: U256,
    pub cspr_amount: U512,
    pub base_rate: U512,
}

impl CsprToCowl {
    pub fn new(
        source_purse: URef,
        recipient: Key,
        cowl_amount: U256,
        cspr_amount: U512,
        base_rate: U512,
    ) -> Self {
        Self {
            source_purse,
            recipient,
            cowl_amount,
            cspr_amount,
            base_rate,
        }
    }
}

#[cfg(feature = "contract-support")]
fn ces(event: Event) {
    match event {
        Event::SetModalities(ev) => emit(ev),
        Event::Upgrade(ev) => emit(ev),
        Event::ChangeSecurity(ev) => emit(ev),
        Event::CowlCep18ContractPackageUpdate(ev) => emit(ev),
        Event::UpdateTimes(ev) => emit(ev),
        Event::DepositCspr(ev) => emit(ev),
        Event::WithdrawCowl(ev) => emit(ev),
        Event::WithdrawCspr(ev) => emit(ev),
        Event::CowlToCspr(ev) => emit(ev),
        Event::CsprToCowl(ev) => emit(ev),
    }
}

#[cfg(feature = "contract-support")]
pub fn init_events() {
    use casper_contract::contract_api::runtime::get_key;

    let events_mode =
        EventsMode::try_from(get_stored_value::<u8>(ARG_EVENTS_MODE)).unwrap_or_revert();

    if [EventsMode::CES].contains(&events_mode)
        && get_key(casper_event_standard::EVENTS_DICT).is_none()
    {
        let schemas = Schemas::new()
            .with::<SetModalities>()
            .with::<Upgrade>()
            .with::<CowlCep18ContractPackageUpdate>()
            .with::<UpdateTimes>()
            .with::<DepositCspr>()
            .with::<WithdrawCowl>()
            .with::<WithdrawCspr>()
            .with::<CowlToCspr>()
            .with::<CsprToCowl>()
            .with::<ChangeSecurity>();
        casper_event_standard::init(schemas);
    }
}
