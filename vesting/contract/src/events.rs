use crate::security::SecurityBadge;
#[cfg(feature = "contract-support")]
use crate::{constants::ARG_EVENTS_MODE, modalities::EventsMode, utils::get_stored_value};
use alloc::collections::BTreeMap;
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

#[cfg(feature = "contract-support")]
fn ces(event: Event) {
    match event {
        Event::ChangeSecurity(ev) => emit(ev),
        Event::SetModalities(ev) => emit(ev),
        Event::Upgrade(ev) => emit(ev),
    }
}

#[cfg(feature = "contract-support")]
pub fn init_events() {
    let events_mode =
        EventsMode::try_from(get_stored_value::<u8>(ARG_EVENTS_MODE)).unwrap_or_revert();

    if events_mode == EventsMode::CES {
        let schemas = Schemas::new()
            .with::<ChangeSecurity>()
            .with::<SetModalities>()
            .with::<Upgrade>();
        casper_event_standard::init(schemas);
    }
}
