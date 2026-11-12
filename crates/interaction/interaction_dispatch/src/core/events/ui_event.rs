/*
 * Copyright (C) 2001 Peter Kelly (pmk@post.com)
 * Copyright (C) 2001 Tobias Anton (anton@stud.fbi.fh-darmstadt.de)
 * Copyright (C) 2006 Samuel Weinig (sam.weinig@gmail.com)
 * Copyright (C) 2003, 2004, 2005, 2006, 2008 Apple Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB. If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/ui_event.h:1-107
// Source: third_party/blink/renderer/core/events/ui_event.cc:1-104
// Rust modules and the embedded Event replace C++ includes, guards, namespaces,
// export/wrapper annotations, and inheritance.

use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use interaction_input::base::time::time::TimeTicks;

use crate::core::dom::events::event::{
    Bubbles, Cancelable, ComposedMode, Event, EventClock, EventInterface, EventTarget,
    EventTargetRef,
};

pub use super::ui_event_init::UIEventInit;

// Source: ui_event.h:30,37-38,56,79,95
// UIEvent only retains DOMWindow identity. More specific event classes accept
// the coordinate facts they consume separately from this shared owner.
pub trait AbstractView: EventTarget {}
pub type AbstractViewRef = Rc<dyn AbstractView>;

// The native EventTarget facade retains its Window host and allocation. IDL
// bindings validate Window kind before projecting this owner as UIEvent.view.
impl AbstractView for crate::core::dom::events::event_target::EventTarget {}

// Source: ui_event.h:35,58,81-83,97
// The resident capabilities object is retained and returned unchanged; UIEvent
// does not inspect its fields or own a device/platform service.
pub trait InputDeviceCapabilities {
    fn firesTouchEvents(&self) -> bool;
}
pub type InputDeviceCapabilitiesRef = Rc<dyn InputDeviceCapabilities>;

// Source: ui_event.h:40-42,94-98
pub struct UIEvent {
    event: Event,
    view_: Option<AbstractViewRef>,
    detail_: i32,
    source_capabilities_: Option<InputDeviceCapabilitiesRef>,
}

impl UIEvent {
    // Source: ui_event.h:44
    pub fn Create(clock: &dyn EventClock) -> Self {
        Self::Uninitialized(clock.Now())
    }

    // Source: ui_event.h:45-48
    pub fn CreateWithInitializer(
        event_type: impl Into<String>,
        initializer: &UIEventInit,
        clock: &dyn EventClock,
    ) -> Self {
        Self::WithInitializer(event_type, initializer, clock.Now())
    }

    // Source: ui_event.h:50; ui_event.cc:30
    // The implicit Event default constructor captures the same supplied clock
    // timestamp; its uninitialized flag remains false.
    pub fn Uninitialized(platform_time_stamp: TimeTicks) -> Self {
        Self {
            event: Event::Uninitialized(platform_time_stamp),
            view_: None,
            detail_: 0,
            source_capabilities_: None,
        }
    }

    // Source: ui_event.h:51-58; ui_event.cc:32-47
    pub fn new(
        event_type: impl Into<String>,
        bubbles: Bubbles,
        cancelable: Cancelable,
        composed_mode: ComposedMode,
        platform_time_stamp: TimeTicks,
        view: Option<AbstractViewRef>,
        detail: i32,
        source_capabilities: Option<InputDeviceCapabilitiesRef>,
    ) -> Self {
        Self {
            event: Event::new(
                event_type,
                bubbles,
                cancelable,
                composed_mode,
                platform_time_stamp,
            ),
            view_: view,
            detail_: detail,
            source_capabilities_: source_capabilities,
        }
    }

    // Source: ui_event.h:59-61; ui_event.cc:49-55
    pub fn WithInitializer(
        event_type: impl Into<String>,
        initializer: &UIEventInit,
        platform_time_stamp: TimeTicks,
    ) -> Self {
        Self {
            event: Event::WithInitializer(event_type, &initializer.event_init, platform_time_stamp),
            view_: initializer.view.clone(),
            detail_: initializer.detail,
            source_capabilities_: initializer.source_capabilities.clone(),
        }
    }

    // Source: ui_event.h:62-63
    pub fn WithInitializerAndClock(
        event_type: impl Into<String>,
        initializer: &UIEventInit,
        clock: &dyn EventClock,
    ) -> Self {
        Self::WithInitializer(event_type, initializer, clock.Now())
    }

    // Source: ui_event.h:64; ui_event.cc:57
    // Rust field destruction supplies the source default destructor.

    // Source: ui_event.h:66-70; ui_event.cc:59-66
    pub fn initUIEvent(
        &mut self,
        event_type: impl Into<String>,
        bubbles: bool,
        cancelable: bool,
        view: Option<AbstractViewRef>,
        detail: i32,
    ) {
        self.InitUIEventInternal(event_type, bubbles, cancelable, None, view, detail, None);
    }

    // Source: ui_event.h:71-77; ui_event.cc:68-84
    pub fn InitUIEventInternal(
        &mut self,
        event_type: impl Into<String>,
        bubbles: bool,
        cancelable: bool,
        related_target: Option<EventTargetRef>,
        view: Option<AbstractViewRef>,
        detail: i32,
        source_capabilities: Option<InputDeviceCapabilitiesRef>,
    ) {
        if self.event.IsBeingDispatched() {
            return;
        }
        self.event
            .initEventWithRelatedTarget(event_type, bubbles, cancelable, related_target);
        self.view_ = view;
        self.detail_ = detail;
        self.source_capabilities_ = source_capabilities;
    }

    // Source: ui_event.h:79
    pub fn view(&self) -> Option<AbstractViewRef> {
        self.view_.clone()
    }

    // Source: ui_event.h:80
    pub fn detail(&self) -> i32 {
        self.detail_
    }

    // Source: ui_event.h:81-83
    pub fn sourceCapabilities(&self) -> Option<InputDeviceCapabilitiesRef> {
        self.source_capabilities_.clone()
    }

    // Source: ui_event.h:85
    // EventInterface::pseudoTarget forwards to the same embedded Event owner.

    // Source: ui_event.h:87-88; ui_event.cc:86-92
    pub fn InterfaceName(&self) -> &str {
        "UIEvent"
    }

    pub fn IsUIEvent(&self) -> bool {
        true
    }

    // Source: ui_event.h:90; ui_event.cc:94-96
    pub fn which(&self) -> u32 {
        0
    }

    // Source: ui_event.h:92; ui_event.cc:98-102
    // Rc retains and traces the same view/capabilities owners; Event retains
    // its base owners. Rust Drop releases each at the enclosing event lifetime.
}

// Source: ui_event.h:40
// Derived events embed UIEvent and can borrow its original base state.
impl Deref for UIEvent {
    type Target = Event;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl DerefMut for UIEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}

impl EventInterface for UIEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }

    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }

    // Source: ui_event.h:87; ui_event.cc:90-92
    fn InterfaceName(&self) -> &str {
        UIEvent::InterfaceName(self)
    }

    // Source: ui_event.h:88,100-103; ui_event.cc:86-88
    // The source downcast guard is the virtual IsUIEvent query.
    fn IsUIEvent(&self) -> bool {
        true
    }
}

// Source: ui_event.h:90
// This virtual projection lets keyboard and mouse event classes override which
// while retaining UIEvent's common state and the same Event base owner.
pub trait UIEventInterface: EventInterface {
    fn BaseUIEvent(&self) -> &UIEvent;
    fn BaseUIEventMut(&mut self) -> &mut UIEvent;

    // Source: ui_event.cc:94-96
    fn which(&self) -> u32 {
        0
    }
}

impl UIEventInterface for UIEvent {
    fn BaseUIEvent(&self) -> &UIEvent {
        self
    }

    fn BaseUIEventMut(&mut self) -> &mut UIEvent {
        self
    }
}
