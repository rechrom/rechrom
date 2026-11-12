/*
 * Copyright (C) 2009 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/composition_event.h:1-74
// Source: third_party/blink/renderer/core/events/composition_event.cc:1-85
// Rust modules replace includes/guards/namespaces and wrapper annotations.

pub use super::composition_event_init::CompositionEventInit;
use super::ui_event::{AbstractViewRef, InputDeviceCapabilitiesRef, UIEvent, UIEventInterface};
use crate::core::dom::events::event::{
    Bubbles, Cancelable, ComposedMode, Event, EventClock, EventInterface,
};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

// Source: composition_event.cc:47-48
// The view's existing capabilities owner supplies this one source query.
pub trait CompositionDeviceCapabilities {
    fn FiresTouchEvents(
        &self,
        view: &AbstractViewRef,
        fires_touch_events: bool,
    ) -> InputDeviceCapabilitiesRef;
}

// Source: composition_event.h:36-37,68-69
pub struct CompositionEvent {
    ui_event: UIEvent,
    data_: Rc<[u16]>,
}
impl CompositionEvent {
    // Source: composition_event.h:40-42,49; composition_event.cc:35
    pub fn Create(clock: &dyn EventClock) -> Self {
        Self {
            ui_event: UIEvent::Create(clock),
            data_: Rc::from([]),
        }
    }

    // Source: composition_event.h:50; composition_event.cc:37-49
    pub fn new(
        event_type: impl Into<String>,
        view: Option<AbstractViewRef>,
        data: String,
        capabilities: &dyn CompositionDeviceCapabilities,
        clock: &dyn EventClock,
    ) -> Self {
        Self::newUTF16(
            event_type,
            view,
            data.encode_utf16().collect::<Vec<_>>().into(),
            capabilities,
            clock,
        )
    }
    pub fn newUTF16(
        event_type: impl Into<String>,
        view: Option<AbstractViewRef>,
        data: Rc<[u16]>,
        capabilities: &dyn CompositionDeviceCapabilities,
        clock: &dyn EventClock,
    ) -> Self {
        let platform_time_stamp = clock.Now();
        let source_capabilities = view
            .as_ref()
            .map(|view| capabilities.FiresTouchEvents(view, false));
        Self {
            ui_event: UIEvent::new(
                event_type,
                Bubbles::kYes,
                Cancelable::kYes,
                ComposedMode::kComposed,
                platform_time_stamp,
                view,
                0,
                source_capabilities,
            ),
            data_: data,
        }
    }

    // Source: composition_event.h:44-47,51; composition_event.cc:51-56
    pub fn WithInitializer(
        event_type: impl Into<String>,
        initializer: &CompositionEventInit,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            ui_event: UIEvent::WithInitializerAndClock(
                event_type,
                &initializer.ui_event_init,
                clock,
            ),
            data_: initializer
                .data_utf16
                .clone()
                .unwrap_or_else(|| initializer.data.encode_utf16().collect::<Vec<_>>().into()),
        }
    }
    // Source: composition_event.h:52; composition_event.cc:58
    // Rust Drop supplies the source default destructor.

    // Source: composition_event.h:54-58; composition_event.cc:60-71
    pub fn initCompositionEvent(
        &mut self,
        event_type: impl Into<String>,
        bubbles: bool,
        cancelable: bool,
        view: Option<AbstractViewRef>,
        data: String,
    ) {
        if self.IsBeingDispatched() {
            return;
        }
        self.ui_event
            .initUIEvent(event_type, bubbles, cancelable, view, 0);
        self.data_ = data.encode_utf16().collect::<Vec<_>>().into();
    }
    // Source: composition_event.h:60
    pub fn data(&self) -> &[u16] {
        &self.data_
    }
    pub fn dataUTF16(&self) -> Rc<[u16]> {
        self.data_.clone()
    }
    // Source: composition_event.h:62-64; composition_event.cc:73-79
    pub fn InterfaceName(&self) -> &str {
        "CompositionEvent"
    }
    pub fn IsCompositionEvent(&self) -> bool {
        true
    }
    // Source: composition_event.h:66; composition_event.cc:81-83
    // UIEvent's retained owners and Rust Drop supply the source trace lifetime.
}
impl Deref for CompositionEvent {
    type Target = UIEvent;
    fn deref(&self) -> &UIEvent {
        &self.ui_event
    }
}
impl DerefMut for CompositionEvent {
    fn deref_mut(&mut self) -> &mut UIEvent {
        &mut self.ui_event
    }
}
impl EventInterface for CompositionEvent {
    fn AsCompositionEvent(&self) -> Option<&CompositionEvent> {
        Some(self)
    }
    fn BaseEvent(&self) -> &Event {
        self.ui_event.BaseEvent()
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        self.ui_event.BaseEventMut()
    }
    fn InterfaceName(&self) -> &str {
        CompositionEvent::InterfaceName(self)
    }
    fn IsUIEvent(&self) -> bool {
        true
    }
    fn IsCompositionEvent(&self) -> bool {
        true
    }
}
impl UIEventInterface for CompositionEvent {
    fn BaseUIEvent(&self) -> &UIEvent {
        &self.ui_event
    }
    fn BaseUIEventMut(&mut self) -> &mut UIEvent {
        &mut self.ui_event
    }
}
