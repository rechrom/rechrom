// Copyright 2024 The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/events/interest_event.h:17-62; cc:18-84.
use crate::core::dom::events::{
    event::{Event, EventClock, EventInterface, EventTargetRef},
    event_dispatch_result::DispatchEventResult,
    event_dispatcher::{EventDispatchHook, EventDispatcher},
    event_init::EventInit,
};
use std::ops::{Deref, DerefMut};
pub struct InterestEvent {
    event: Event,
    source_: Option<EventTargetRef>,
    related_target_: Option<EventTargetRef>,
    reference_target_enabled: bool,
}
impl InterestEvent {
    pub fn new(
        kind: &str,
        source: EventTargetRef,
        cancelable: bool,
        non_composed: bool,
        reference_target_enabled: bool,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            event: Event::CreateWithInitializer(
                kind,
                &EventInit {
                    cancelable,
                    composed: !non_composed,
                    ..Default::default()
                },
                clock,
            ),
            source_: Some(source.clone()),
            related_target_: Some(source),
            reference_target_enabled,
        }
    }
    pub fn WithInitializer(
        kind: &str,
        init: &EventInit,
        source: Option<EventTargetRef>,
        reference_target_enabled: bool,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            event: Event::CreateWithInitializer(kind, init, clock),
            source_: source.clone(),
            related_target_: source,
            reference_target_enabled,
        }
    }
    pub fn OriginalSource(&self) -> Option<EventTargetRef> {
        self.source_.clone()
    }
    pub fn RelatedSource(&self) -> Option<EventTargetRef> {
        self.related_target_.clone()
    }
    pub fn ReferenceTargetEnabled(&self) -> bool {
        self.reference_target_enabled
    }
}
impl Deref for InterestEvent {
    type Target = Event;
    fn deref(&self) -> &Event {
        &self.event
    }
}
impl DerefMut for InterestEvent {
    fn deref_mut(&mut self) -> &mut Event {
        &mut self.event
    }
}
struct InterestHook {
    source: Option<EventTargetRef>,
    reference_target_enabled: bool,
}
impl EventDispatchHook for InterestHook {
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
impl EventInterface for InterestEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "InterestEvent"
    }
    fn AsInterestEvent(&self) -> Option<&InterestEvent> {
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
        Box::new(InterestHook {
            source: self.related_target_.clone(),
            reference_target_enabled: self.reference_target_enabled,
        })
    }
}
