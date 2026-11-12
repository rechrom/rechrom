//! Chromium page_transition_event.h:40-85, .cc:34-73.
use crate::core::dom::events::event::{
    Bubbles, Cancelable, ComposedMode, Event, EventClock, EventInit, EventInterface,
};
use interaction_input::base::time::time::TimeTicks;
use std::ops::{Deref, DerefMut};
pub struct PageTransitionEvent {
    event: Event,
    persisted: bool,
}
impl PageTransitionEvent {
    pub fn Create(event_type: &str, persisted: bool, clock: &dyn EventClock) -> Self {
        debug_assert!(!(persisted && event_type == "pageshow"));
        Self {
            event: Event::new(
                event_type,
                Bubbles::kYes,
                Cancelable::kYes,
                ComposedMode::kScoped,
                clock.Now(),
            ),
            persisted,
        }
    }
    pub fn WithInitializer(
        event_type: &str,
        init: &EventInit,
        persisted: bool,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            event: Event::CreateWithInitializer(event_type, init, clock),
            persisted,
        }
    }
    pub fn CreatePersistedPageshow(navigation_start: TimeTicks) -> Self {
        Self {
            event: Event::new(
                "pageshow",
                Bubbles::kYes,
                Cancelable::kYes,
                ComposedMode::kScoped,
                navigation_start,
            ),
            persisted: true,
        }
    }
    pub fn persisted(&self) -> bool {
        self.persisted
    }
}
impl Deref for PageTransitionEvent {
    type Target = Event;
    fn deref(&self) -> &Event {
        &self.event
    }
}
impl DerefMut for PageTransitionEvent {
    fn deref_mut(&mut self) -> &mut Event {
        &mut self.event
    }
}
impl EventInterface for PageTransitionEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "PageTransitionEvent"
    }
    fn AsPageTransitionEvent(&self) -> Option<&PageTransitionEvent> {
        Some(self)
    }
}
