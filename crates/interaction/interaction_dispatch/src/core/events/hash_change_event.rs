//! Chromium hash_change_event.h:39-77.
use crate::core::dom::events::event::{Event, EventClock, EventInterface};
use navigation_history::UserNavigationInvolvement;

pub struct HashChangeEvent {
    event: Event,
    old_url: String,
    new_url: String,
    involvement: UserNavigationInvolvement,
}
impl HashChangeEvent {
    pub fn Create(
        old_url: String,
        new_url: String,
        involvement: UserNavigationInvolvement,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            event: Event::CreateWithType("hashchange", clock),
            old_url,
            new_url,
            involvement,
        }
    }
    pub fn oldURL(&self) -> &str {
        &self.old_url
    }
    pub fn newURL(&self) -> &str {
        &self.new_url
    }
    pub fn IsUserInitiated(&self) -> bool {
        self.involvement != UserNavigationInvolvement::None
    }
}
impl EventInterface for HashChangeEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "HashChangeEvent"
    }
}
