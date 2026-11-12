//! Chromium core/css/font_face_set_load_event.h:41-73/.cc:38-55.
//! The array retains the original native FontFace wrapper owners.
use crate::{
    bindings::script_invocation::ScriptNativeObject,
    core::dom::events::event::{Event, EventClock, EventInit, EventInterface},
};
use std::rc::Rc;
pub struct FontFaceSetLoadEvent {
    event: Event,
    fontfaces: Vec<Rc<dyn ScriptNativeObject>>,
}
impl FontFaceSetLoadEvent {
    pub fn new(
        kind: &str,
        fontfaces: Vec<Rc<dyn ScriptNativeObject>>,
        clock: &dyn EventClock,
    ) -> Self {
        Self::WithType(kind, EventInit::default(), fontfaces, clock)
    }
    pub fn WithType(
        kind: &str,
        init: EventInit,
        fontfaces: Vec<Rc<dyn ScriptNativeObject>>,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            event: Event::CreateWithInitializer(kind, &init, clock),
            fontfaces,
        }
    }
    pub fn fontfaces(&self) -> &[Rc<dyn ScriptNativeObject>] {
        &self.fontfaces
    }
}
impl EventInterface for FontFaceSetLoadEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "FontFaceSetLoadEvent"
    }
    fn AsFontFaceSetLoadEvent(&self) -> Option<&Self> {
        Some(self)
    }
}
