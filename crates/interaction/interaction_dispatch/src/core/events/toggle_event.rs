// Copyright 2022 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/events/toggle_event.cc:25-105; .h:23-52.
use crate::core::dom::events::{
    event::{Event, EventClock, EventInterface, EventTargetRef},
    event_dispatch_result::DispatchEventResult,
    event_dispatcher::{EventDispatchHook, EventDispatcher},
    event_init::EventInit,
};
use std::ops::{Deref, DerefMut};
pub struct ToggleEvent {
    event: Event,
    old_state_: String,
    new_state_: String,
    source_: Option<EventTargetRef>,
    related_target_: Option<EventTargetRef>,
    reference_target_enabled: bool,
}
impl ToggleEvent {
    // HTMLDetailsElement always supplies a null source (cc:181-185).
    pub fn ForDetails(old_state: &str, new_state: &str, clock: &dyn EventClock) -> Self {
        debug_assert!(matches!(old_state, "closed" | "open"));
        debug_assert!(matches!(new_state, "closed" | "open"));
        Self::WithInitializer(
            "toggle",
            &EventInit::default(),
            old_state,
            new_state,
            None,
            true,
            clock,
        )
    }
    // toggle_event.cc:42-56. Unlike the native details creation path, WebIDL
    // initializer strings are not restricted to closed/open keywords.
    pub fn WithInitializer(
        event_type: &str,
        init: &EventInit,
        old_state: &str,
        new_state: &str,
        source: Option<EventTargetRef>,
        reference_target_enabled: bool,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            event: Event::CreateWithInitializer(event_type, init, clock),
            old_state_: old_state.into(),
            new_state_: new_state.into(),
            related_target_: source.clone(),
            source_: source,
            reference_target_enabled,
        }
    }
    pub fn oldState(&self) -> &str {
        &self.old_state_
    }
    pub fn newState(&self) -> &str {
        &self.new_state_
    }
    pub fn OriginalSource(&self) -> Option<EventTargetRef> {
        self.source_.clone()
    }
    pub fn ReferenceTargetEnabled(&self) -> bool {
        self.reference_target_enabled
    }
    pub fn RelatedSource(&self) -> Option<EventTargetRef> {
        self.related_target_.clone()
    }
    pub fn DispatchHook(&self) -> ToggleEventDispatchHook {
        ToggleEventDispatchHook {
            source: self.related_target_.clone(),
            reference_target_enabled: self.reference_target_enabled,
        }
    }
}
pub struct ToggleEventDispatchHook {
    source: Option<EventTargetRef>,
    reference_target_enabled: bool,
}
impl EventDispatchHook for ToggleEventDispatchHook {
    fn DispatchEvent(&self, dispatcher: &mut EventDispatcher<'_>) -> DispatchEventResult {
        if self.reference_target_enabled {
            dispatcher
                .GetEventPath()
                .borrow_mut()
                .AdjustForRelatedTarget(&dispatcher.GetNode(), self.source.clone());
        }
        dispatcher.Dispatch()
    }
}
impl Deref for ToggleEvent {
    type Target = Event;
    fn deref(&self) -> &Event {
        &self.event
    }
}
impl DerefMut for ToggleEvent {
    fn deref_mut(&mut self) -> &mut Event {
        &mut self.event
    }
}
impl EventInterface for ToggleEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "ToggleEvent"
    }
    fn AsToggleEvent(&self) -> Option<&ToggleEvent> {
        Some(self)
    }
    fn relatedTarget(&self) -> Option<EventTargetRef> {
        self.related_target_.clone()
    }
    fn SetRelatedTarget(&mut self, target: Option<EventTargetRef>) {
        self.related_target_ = target;
    }
    fn PrepareDispatchHook<'a>(
        &self,
        _nested: &'a dyn crate::core::events::mouse_event::MouseNestedDispatch,
    ) -> Box<dyn EventDispatchHook + 'a> {
        Box::new(self.DispatchHook())
    }
}
