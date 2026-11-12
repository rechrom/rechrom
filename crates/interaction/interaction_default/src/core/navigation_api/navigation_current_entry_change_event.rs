//! Chromium navigation_current_entry_change_event.cc:15-30.
use super::navigation_api::{NavigationHistoryEntry, NavigationType};
use interaction_dispatch::core::dom::events::event::{Event, EventInterface};
use std::rc::Rc;
pub struct NavigationCurrentEntryChangeEvent {
    pub event: Event,
    pub from: Option<Rc<dyn NavigationHistoryEntry>>,
    pub navigation_type: Option<NavigationType>,
}
impl EventInterface for NavigationCurrentEntryChangeEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "NavigationCurrentEntryChangeEvent"
    }
}
