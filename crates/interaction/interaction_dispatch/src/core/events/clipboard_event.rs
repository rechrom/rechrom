// Chromium core/events/clipboard_event.cc:31-51, clipboard_event.h:35-63.
use crate::core::{
    clipboard::data_transfer::DataTransfer,
    dom::events::event::{Bubbles, Cancelable, ComposedMode, Event, EventClock, EventInterface},
};
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};
pub struct ClipboardEvent {
    event: Event,
    clipboard_data: Option<Rc<DataTransfer>>,
}
impl ClipboardEvent {
    pub fn new(kind: &str, data: Option<Rc<DataTransfer>>, clock: &dyn EventClock) -> Self {
        Self {
            event: Event::new(
                kind,
                Bubbles::kYes,
                Cancelable::kYes,
                ComposedMode::kComposed,
                clock.Now(),
            ),
            clipboard_data: data,
        }
    }
    pub fn WithInitializer(
        kind: &str,
        data: Option<Rc<DataTransfer>>,
        init: &crate::core::dom::events::event_init::EventInit,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            event: Event::new(
                kind,
                if init.bubbles {
                    Bubbles::kYes
                } else {
                    Bubbles::kNo
                },
                if init.cancelable {
                    Cancelable::kYes
                } else {
                    Cancelable::kNo
                },
                if init.composed {
                    ComposedMode::kComposed
                } else {
                    ComposedMode::kScoped
                },
                clock.Now(),
            ),
            clipboard_data: data,
        }
    }
    pub fn clipboardData(&self) -> Option<Rc<DataTransfer>> {
        self.clipboard_data.clone()
    }
}
impl Deref for ClipboardEvent {
    type Target = Event;
    fn deref(&self) -> &Event {
        &self.event
    }
}
impl DerefMut for ClipboardEvent {
    fn deref_mut(&mut self) -> &mut Event {
        &mut self.event
    }
}
impl EventInterface for ClipboardEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "ClipboardEvent"
    }
    fn IsClipboardEvent(&self) -> bool {
        true
    }
    fn AsClipboardEvent(&self) -> Option<&ClipboardEvent> {
        Some(self)
    }
}
