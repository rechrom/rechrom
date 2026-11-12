// Copyright 2023 The Chromium Authors. BSD-style license.
//! Chromium 6c1d401f: core/scroll/snap_event.cc:17-40; h:31-64.
use crate::core::dom::events::event::{
    Event, EventClock, EventInterface, EventRetargeting, EventTargetRef,
};
use std::ops::{Deref, DerefMut};
pub struct SnapEvent {
    event: Event,
    block: Option<EventTargetRef>,
    inline: Option<EventTargetRef>,
}
impl SnapEvent {
    pub fn new(
        kind: &str,
        bubbles: bool,
        block: Option<EventTargetRef>,
        inline: Option<EventTargetRef>,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            event: if bubbles {
                Event::CreateBubble(kind, clock)
            } else {
                Event::CreateWithType(kind, clock)
            },
            block,
            inline,
        }
    }
    pub fn snapTargetBlock(&self, retargeting: &dyn EventRetargeting) -> Option<EventTargetRef> {
        self.event.Retarget(self.block.clone(), retargeting)
    }
    pub fn snapTargetInline(&self, retargeting: &dyn EventRetargeting) -> Option<EventTargetRef> {
        self.event.Retarget(self.inline.clone(), retargeting)
    }
}
impl Deref for SnapEvent {
    type Target = Event;
    fn deref(&self) -> &Event {
        &self.event
    }
}
impl DerefMut for SnapEvent {
    fn deref_mut(&mut self) -> &mut Event {
        &mut self.event
    }
}
impl EventInterface for SnapEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "SnapEvent"
    }
    fn AsSnapEvent(&self) -> Option<&Self> {
        Some(self)
    }
}
