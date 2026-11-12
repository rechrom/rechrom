/*
 * Copyright (C) 2013 Google Inc. All rights reserved.
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
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS''
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/focus_event.h:1-90
// Source: third_party/blink/renderer/core/events/focus_event.cc:1-79
// Rust modules replace includes/guards/namespaces and wrapper annotations.

pub use super::focus_event_init::FocusEventInit;
use super::ui_event::{AbstractViewRef, InputDeviceCapabilitiesRef, UIEvent, UIEventInterface};
use crate::core::dom::events::event::{
    Bubbles, Cancelable, ComposedMode, Event, EventClock, EventInterface, EventTargetRef,
};
use crate::core::dom::events::event_dispatch_result::DispatchEventResult;
use crate::core::dom::events::event_dispatcher::{EventDispatchHook, EventDispatcher};
use std::ops::{Deref, DerefMut};

// Source: focus_event.h:37-38,79-80
pub struct FocusEvent {
    ui_event: UIEvent,
    related_target_: Option<EventTargetRef>,
}

impl FocusEvent {
    // Source: focus_event.h:41; focus_event.cc:44
    pub fn Create(clock: &dyn EventClock) -> Self {
        Self {
            ui_event: UIEvent::Create(clock),
            related_target_: None,
        }
    }

    // Source: focus_event.h:43-51,59-64; focus_event.cc:46-60
    pub fn new(
        event_type: impl Into<String>,
        bubbles: Bubbles,
        view: Option<AbstractViewRef>,
        detail: i32,
        related_target: Option<EventTargetRef>,
        source_capabilities: Option<InputDeviceCapabilitiesRef>,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            ui_event: UIEvent::new(
                event_type,
                bubbles,
                Cancelable::kNo,
                ComposedMode::kComposed,
                clock.Now(),
                view,
                detail,
                source_capabilities,
            ),
            related_target_: related_target,
        }
    }

    // Source: focus_event.h:53-56,65; focus_event.cc:62-67
    pub fn WithInitializer(
        event_type: impl Into<String>,
        initializer: &FocusEventInit,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            ui_event: UIEvent::WithInitializerAndClock(
                event_type,
                &initializer.ui_event_init,
                clock,
            ),
            related_target_: initializer.related_target.clone(),
        }
    }

    // Source: focus_event.h:67-70
    pub fn relatedTarget(&self) -> Option<EventTargetRef> {
        self.related_target_.clone()
    }
    pub fn SetRelatedTarget(&mut self, related_target: Option<EventTargetRef>) {
        self.related_target_ = related_target;
    }

    // Source: focus_event.h:72-73; focus_event.cc:36-42
    pub fn InterfaceName(&self) -> &str {
        "FocusEvent"
    }
    pub fn IsFocusEvent(&self) -> bool {
        true
    }

    // Source: focus_event.h:75; focus_event.cc:74-76
    // The retained related target is captured before listener entry, allowing
    // the same EventRef to be borrowed synchronously by listeners.
    pub fn DispatchHook(&self) -> FocusEventDispatchHook {
        FocusEventDispatchHook {
            related_target: self.related_target_.clone(),
        }
    }

    // Source: focus_event.h:77; focus_event.cc:69-72
    // Rc retains the target; Drop releases it with the UIEvent base owner.
}

// Source: focus_event.h:75; focus_event.cc:74-77
pub struct FocusEventDispatchHook {
    related_target: Option<EventTargetRef>,
}
impl EventDispatchHook for FocusEventDispatchHook {
    fn DispatchEvent(&self, dispatcher: &mut EventDispatcher<'_>) -> DispatchEventResult {
        dispatcher
            .GetEventPath()
            .borrow_mut()
            .AdjustForRelatedTarget(&dispatcher.GetNode(), self.related_target.clone());
        dispatcher.Dispatch()
    }
}

impl Deref for FocusEvent {
    type Target = UIEvent;
    fn deref(&self) -> &UIEvent {
        &self.ui_event
    }
}
impl DerefMut for FocusEvent {
    fn deref_mut(&mut self) -> &mut UIEvent {
        &mut self.ui_event
    }
}
impl EventInterface for FocusEvent {
    fn PrepareDispatchHook<'a>(
        &self,
        _nested: &'a dyn crate::core::events::mouse_event::MouseNestedDispatch,
    ) -> Box<dyn crate::core::dom::events::event_dispatcher::EventDispatchHook + 'a> {
        Box::new(self.DispatchHook())
    }
    fn BaseEvent(&self) -> &Event {
        self.ui_event.BaseEvent()
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        self.ui_event.BaseEventMut()
    }
    fn relatedTarget(&self) -> Option<EventTargetRef> {
        FocusEvent::relatedTarget(self)
    }
    fn SetRelatedTarget(&mut self, target: Option<EventTargetRef>) {
        FocusEvent::SetRelatedTarget(self, target);
    }
    fn InterfaceName(&self) -> &str {
        FocusEvent::InterfaceName(self)
    }
    fn IsUIEvent(&self) -> bool {
        true
    }
    // Source: focus_event.h:83-86; focus_event.cc:40-42
    fn IsFocusEvent(&self) -> bool {
        true
    }
}
impl UIEventInterface for FocusEvent {
    fn BaseUIEvent(&self) -> &UIEvent {
        &self.ui_event
    }
    fn BaseUIEventMut(&mut self) -> &mut UIEvent {
        &mut self.ui_event
    }
}
