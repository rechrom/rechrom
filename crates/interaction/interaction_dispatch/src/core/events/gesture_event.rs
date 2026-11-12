/*
 * Copyright (C) 2012 Google Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *  * Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 *  * Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/gesture_event.{h,cc}
use super::{
    ui_event::{AbstractViewRef, UIEvent, UIEventInterface},
    ui_event_with_key_state::{UIEventWithKeyState, UIEventWithKeyStateInterface},
};
use crate::core::dom::events::event::{Bubbles, Cancelable, Event, EventInterface};
use interaction_input::public::common::input::{
    web_gesture_event::WebGestureEvent, web_input_event::Type,
};
use std::ops::{Deref, DerefMut};
pub struct GestureEvent {
    ui_event: UIEventWithKeyState,
    native_event_: WebGestureEvent,
}
impl GestureEvent {
    pub fn Create(view: Option<AbstractViewRef>, event: &WebGestureEvent) -> Option<Self> {
        let kind = match event.GetType() {
            Type::kGestureTap => "gesturetap",
            Type::kGestureTapUnconfirmed => "gesturetapunconfirmed",
            Type::kGestureTapDown => "gesturetapdown",
            Type::kGestureShowPress => "gestureshowpress",
            Type::kGestureLongPress => "gesturelongpress",
            Type::kGestureFlingStart => "gestureflingstart",
            _ => return None,
        };
        Some(Self::new(kind, view, event))
    }
    pub fn new(kind: &str, view: Option<AbstractViewRef>, event: &WebGestureEvent) -> Self {
        Self {
            ui_event: UIEventWithKeyState::new(
                kind,
                Bubbles::kYes,
                Cancelable::kYes,
                view,
                0,
                event.GetModifiers(),
                event.TimeStamp(),
                None,
            ),
            native_event_: event.clone(),
        }
    }
    pub fn NativeEvent(&self) -> &WebGestureEvent {
        &self.native_event_
    }
}
impl Deref for GestureEvent {
    type Target = UIEventWithKeyState;
    fn deref(&self) -> &Self::Target {
        &self.ui_event
    }
}
impl DerefMut for GestureEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ui_event
    }
}
impl EventInterface for GestureEvent {
    fn BaseEvent(&self) -> &Event {
        self.ui_event.BaseEvent()
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        self.ui_event.BaseEventMut()
    }
    fn InterfaceName(&self) -> &str {
        self.ui_event.InterfaceName()
    }
    fn IsUIEvent(&self) -> bool {
        true
    }
    fn IsGestureEvent(&self) -> bool {
        true
    }
}
impl UIEventInterface for GestureEvent {
    fn BaseUIEvent(&self) -> &UIEvent {
        self.ui_event.BaseUIEvent()
    }
    fn BaseUIEventMut(&mut self) -> &mut UIEvent {
        self.ui_event.BaseUIEventMut()
    }
}
impl UIEventWithKeyStateInterface for GestureEvent {
    fn BaseUIEventWithKeyState(&self) -> &UIEventWithKeyState {
        &self.ui_event
    }
    fn BaseUIEventWithKeyStateMut(&mut self) -> &mut UIEventWithKeyState {
        &mut self.ui_event
    }
}
