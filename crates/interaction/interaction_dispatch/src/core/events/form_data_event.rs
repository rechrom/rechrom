//! Chromium core/html/forms/form_data_event.cc:14-42.
use crate::core::{
    dom::events::event::{Event, EventClock, EventInit, EventInterface},
    fileapi::form_data_object::FormDataObject,
};
use std::rc::Rc;
pub struct FormDataEvent {
    event: Event,
    data: Rc<FormDataObject>,
}
impl FormDataEvent {
    pub fn new(data: Rc<FormDataObject>, clock: &dyn EventClock) -> Self {
        Self::WithType(
            "formdata",
            EventInit {
                bubbles: true,
                cancelable: false,
                composed: false,
            },
            data,
            clock,
        )
    }
    pub fn WithType(
        kind: &str,
        init: EventInit,
        data: Rc<FormDataObject>,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            event: Event::CreateWithInitializer(kind, &init, clock),
            data,
        }
    }
    pub fn formData(&self) -> Rc<FormDataObject> {
        self.data.clone()
    }
}
impl EventInterface for FormDataEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "FormDataEvent"
    }
    fn AsFormDataEvent(&self) -> Option<&Self> {
        Some(self)
    }
}
