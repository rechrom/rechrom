//! Chromium pop_state_event.cc:69-104; pop_state_event.h:70-91.
//! The serialized value remains attached to the canonical native event. The
//! binding caches `state` in its own world, as the IDL CachedAttribute requires.
//! History's Window binding supplies StateHelper when the serialization is
//! still current; retained events otherwise deserialize their original state.
use crate::{
    bindings::script_invocation::{ScriptInvocation, SerializedScriptValue, Value},
    core::dom::events::event::{Event, EventClock, EventInterface},
};
use navigation_history::UserNavigationInvolvement;
use std::rc::Rc;

pub struct PopStateEvent {
    event: Event,
    serialized_state: Option<Rc<dyn SerializedScriptValue>>,
    has_ua_visual_transition: bool,
    involvement: UserNavigationInvolvement,
}
impl PopStateEvent {
    pub fn Create(
        state: Option<Rc<dyn SerializedScriptValue>>,
        has_ua_visual_transition: bool,
        involvement: UserNavigationInvolvement,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            event: Event::CreateWithType("popstate", clock),
            serialized_state: state,
            has_ua_visual_transition,
            involvement,
        }
    }
    pub fn State(&self, invocation: &dyn ScriptInvocation) -> Result<Value, Value> {
        self.serialized_state.as_ref().map_or_else(
            || Ok(invocation.Null()),
            |state| invocation.Deserialize(state),
        )
    }
    pub fn SerializedState(&self) -> Option<Rc<dyn SerializedScriptValue>> {
        self.serialized_state.clone()
    }
    pub fn hasUAVisualTransition(&self) -> bool {
        self.has_ua_visual_transition
    }
    pub fn IsUserInitiated(&self) -> bool {
        self.involvement != UserNavigationInvolvement::None
    }
}
impl EventInterface for PopStateEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "PopStateEvent"
    }
}
