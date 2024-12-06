use crate::security::SecurityBadge;
#[cfg(feature = "contract-support")]
use crate::{constants::ARG_EVENTS_MODE, enums::EventsMode, utils::get_stored_value};
use alloc::collections::btree_map::BTreeMap;
#[cfg(feature = "contract-support")]
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_event_standard::Event;
#[cfg(feature = "contract-support")]
use casper_event_standard::{emit, Schemas};
use casper_types::Key;
#[cfg(feature = "contract-support")]
use core::convert::TryFrom;

#[derive(Debug)]
pub enum Event {
    ChangeSecurity(ChangeSecurity),
    SetModalities(SetModalities),
    Upgrade(Upgrade),
    CheckTransfer(CheckTransfer),
    CowlCep18ContractPackageUpdate(CowlCep18ContractPackageUpdate),
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

#[derive(Event, Debug, PartialEq, Eq, Default)]
pub struct CheckTransfer {}

impl CheckTransfer {
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

#[cfg(feature = "contract-support")]
fn ces(event: Event) {
    match event {
        Event::SetModalities(ev) => emit(ev),
        Event::Upgrade(ev) => emit(ev),
        Event::ChangeSecurity(ev) => emit(ev),
        Event::CowlCep18ContractPackageUpdate(ev) => emit(ev),
        Event::CheckTransfer(ev) => emit(ev),
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
        let schemas = Schemas::new().with::<SetModalities>().with::<Upgrade>();
        casper_event_standard::init(schemas);
    }
}
