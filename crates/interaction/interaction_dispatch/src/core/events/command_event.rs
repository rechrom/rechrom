// Copyright 2022 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/events/command_event.cc:17-61; command_event.h:22-61.
use crate::core::dom::events::{
    event::{Event, EventClock, EventInterface, EventRetargeting, EventTargetRef},
    event_dispatch_result::DispatchEventResult,
    event_dispatcher::{EventDispatchHook, EventDispatcher},
    event_init::EventInit,
};
use std::ops::{Deref, DerefMut};

pub struct CommandEvent {
    event: Event,
    command_: String,
    source_: Option<EventTargetRef>,
    related_target_: Option<EventTargetRef>,
    reference_target_enabled: bool,
}
impl CommandEvent {
    pub fn new(
        command: &str,
        source: EventTargetRef,
        reference_target_enabled: bool,
        clock: &dyn EventClock,
    ) -> Self {
        Self::WithInitializer(
            "command",
            &EventInit {
                bubbles: false,
                cancelable: true,
                composed: false,
            },
            command,
            Some(source),
            reference_target_enabled,
            clock,
        )
    }
    pub fn WithInitializer(
        event_type: &str,
        init: &EventInit,
        command: &str,
        source: Option<EventTargetRef>,
        reference_target_enabled: bool,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            event: Event::CreateWithInitializer(event_type, init, clock),
            command_: command.into(),
            related_target_: source.clone(),
            source_: source,
            reference_target_enabled,
        }
    }
    pub fn command(&self) -> &str {
        &self.command_
    }
    pub fn source(&self, retargeting: &dyn EventRetargeting) -> Option<EventTargetRef> {
        if self.source_.is_none() {
            debug_assert!(self.related_target_.is_none());
            return None;
        }
        if self.reference_target_enabled {
            self.related_target_.clone()
        } else {
            self.event.Retarget(self.source_.clone(), retargeting)
        }
    }
    pub fn OriginalSource(&self) -> Option<EventTargetRef> {
        self.source_.clone()
    }
    pub fn DispatchHook(&self) -> CommandEventDispatchHook {
        CommandEventDispatchHook {
            source: self.related_target_.clone(),
            reference_target_enabled: self.reference_target_enabled,
        }
    }
}
impl Deref for CommandEvent {
    type Target = Event;
    fn deref(&self) -> &Event {
        &self.event
    }
}
impl DerefMut for CommandEvent {
    fn deref_mut(&mut self) -> &mut Event {
        &mut self.event
    }
}
pub struct CommandEventDispatchHook {
    source: Option<EventTargetRef>,
    reference_target_enabled: bool,
}
impl EventDispatchHook for CommandEventDispatchHook {
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
impl EventInterface for CommandEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "CommandEvent"
    }
    fn AsCommandEvent(&self) -> Option<&CommandEvent> {
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
        Box::new(CommandEventDispatchHook {
            source: self.related_target_.clone(),
            reference_target_enabled: self.reference_target_enabled,
        })
    }
}
