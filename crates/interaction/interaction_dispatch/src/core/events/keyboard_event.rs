/*
 * Copyright (C) 2001 Peter Kelly (pmk@post.com)
 * Copyright (C) 2001 Tobias Anton (anton@stud.fbi.fh-darmstadt.de)
 * Copyright (C) 2006 Samuel Weinig (sam.weinig@gmail.com)
 * Copyright (C) 2003, 2004, 2005, 2006, 2007, 2008 Apple Inc. All rights
 * reserved.
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
// Source: third_party/blink/renderer/core/events/keyboard_event.h:1-125
// Source: third_party/blink/renderer/core/events/keyboard_event.cc:1-234
// Rust modules, owned WebKeyboardEvent and an embedded key-state base replace
// includes, namespaces, unique_ptr, inheritance and wrapper annotations.

use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

/// The source GC KeyboardEvent owner shared by its manager, dispatcher and
/// retained script wrappers. A canonical EventRef projects this same Rc.
pub type KeyboardEventRef = Rc<RefCell<KeyboardEvent>>;

use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::web_input_event::{Type, WebInputEvent};
use interaction_input::public::common::input::web_keyboard_event::WebKeyboardEvent;

pub use super::keyboard_event_init::KeyboardEventInit;
use super::ui_event::{AbstractViewRef, InputDeviceCapabilitiesRef, UIEvent, UIEventInterface};
use super::ui_event_with_key_state::{UIEventWithKeyState, UIEventWithKeyStateInterface};
use crate::core::dom::events::event::{Bubbles, Cancelable, Event, EventClock, EventInterface};

// Source: keyboard_event.h:42-47
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum KeyLocationCode {
    kDomKeyLocationStandard = 0x00,
    kDomKeyLocationLeft = 0x01,
    kDomKeyLocationRight = 0x02,
    kDomKeyLocationNumpad = 0x03,
}

// Source: keyboard_event.cc:72-79,107,111-114
// These are the window's existing shared view/capabilities owners and the
// source frame's composition fact. None is retained as a broad window model.
pub trait KeyboardEventWindow {
    fn View(&self) -> AbstractViewRef;
    fn FrameHasComposition(&self) -> Option<bool>;
    fn FiresTouchEvents(&self, fires_touch_events: bool) -> InputDeviceCapabilitiesRef;
}

// Source: keyboard_event.cc:117-120
// The embedding projects Chromium KeycodeConverter. KeyboardEvent supplies the
// encoded DomCode/DomKey untouched; it does not maintain a second key mapping.
pub trait KeyboardEventKeycodeConverter {
    fn DomCodeToCodeString(&self, dom_code: i32) -> String;
    fn DomKeyToKeyString(&self, dom_key: u32) -> String;
}

// Source: keyboard_event.cc:87-95,175-177
pub trait KeyboardEventScriptState {
    fn IsIsolatedWorld(&self) -> bool;
}

// Source: keyboard_event.h:38-39,109-115
pub struct KeyboardEvent {
    ui_event_with_key_state: UIEventWithKeyState,
    key_event_: Option<Box<WebKeyboardEvent>>,
    code_: String,
    key_: String,
    location_: u32,
    is_composing_: bool,
    char_code_: u32,
    key_code_: u32,
}

impl KeyboardEvent {
    // Source: keyboard_event.h:49-51
    pub fn Create(clock: &dyn EventClock) -> Self {
        Self::Uninitialized(clock.Now())
    }

    // Source: keyboard_event.h:53-58
    pub fn CreateFromWebEvent(
        web_event: &WebKeyboardEvent,
        dom_window: Option<&dyn KeyboardEventWindow>,
        cancellable: bool,
        keycodes: &dyn KeyboardEventKeycodeConverter,
    ) -> Self {
        Self::FromWebEvent(web_event, dom_window, cancellable, keycodes)
    }

    // Source: keyboard_event.h:60-62; keyboard_event.cc:87-96
    pub fn CreateWithInitializer(
        script_state: &dyn KeyboardEventScriptState,
        event_type: impl Into<String>,
        initializer: &KeyboardEventInit,
        clock: &dyn EventClock,
    ) -> Self {
        if script_state.IsIsolatedWorld() {
            let modifiers = &initializer.event_modifier_init;
            // The call retains the source ctrl/alt/shift/meta argument order;
            // the helper consumes only the platform's new-tab modifier.
            UIEventWithKeyState::DidCreateEventInIsolatedWorld(
                modifiers.ctrl_key,
                modifiers.alt_key,
                modifiers.shift_key,
                modifiers.meta_key,
            );
        }
        Self::WithInitializer(event_type, initializer, clock.Now())
    }

    // Source: keyboard_event.h:64; keyboard_event.cc:98
    pub fn Uninitialized(platform_time_stamp: TimeTicks) -> Self {
        let mut event = Self {
            ui_event_with_key_state: UIEventWithKeyState::Uninitialized(platform_time_stamp),
            key_event_: None,
            code_: String::new(),
            key_: String::new(),
            location_: KeyLocationCode::kDomKeyLocationStandard as u32,
            is_composing_: false,
            char_code_: 0,
            key_code_: 0,
        };
        event
            .BaseEventMut()
            .SetEventPathClass(crate::core::dom::events::event::EventPathClass::Keyboard);
        event
    }

    // Source: keyboard_event.h:65-67; keyboard_event.cc:100-142
    pub fn FromWebEvent(
        key: &WebKeyboardEvent,
        dom_window: Option<&dyn KeyboardEventWindow>,
        cancellable: bool,
        keycodes: &dyn KeyboardEventKeycodeConverter,
    ) -> Self {
        let mut event = Self {
            ui_event_with_key_state: UIEventWithKeyState::new(
                EventTypeForKeyboardEventType(key.GetType()),
                Bubbles::kYes,
                if cancellable {
                    Cancelable::kYes
                } else {
                    Cancelable::kNo
                },
                dom_window.map(KeyboardEventWindow::View),
                0,
                key.GetModifiers(),
                key.TimeStamp(),
                dom_window.map(|window| window.FiresTouchEvents(false)),
            ),
            key_event_: Some(Box::new(key.clone())),
            code_: keycodes.DomCodeToCodeString(key.dom_code),
            key_: keycodes.DomKeyToKeyString(key.dom_key),
            location_: GetKeyLocationCode(key) as u32,
            is_composing_: HasCurrentComposition(dom_window),
            char_code_: 0,
            key_code_: 0,
        };
        event.InitLocationModifiers(event.location_);
        if event.r#type() == "keypress" {
            event.char_code_ = key.text[0] as u32;
        }
        if event.r#type() == "keydown" || event.r#type() == "keyup" {
            event.key_code_ = key.windows_key_code as u32;
        } else {
            event.key_code_ = event.char_code_;
        }
        #[cfg(target_os = "android")]
        if key.native_key_code == 0xe5 {
            event.key_code_ = 0xe5;
        }
        event
            .BaseEventMut()
            .SetEventPathClass(crate::core::dom::events::event::EventPathClass::Keyboard);
        event
    }

    // Source: keyboard_event.h:68-70; keyboard_event.cc:144-157
    pub fn WithInitializer(
        event_type: impl Into<String>,
        initializer: &KeyboardEventInit,
        platform_time_stamp: TimeTicks,
    ) -> Self {
        let mut event = Self {
            ui_event_with_key_state: UIEventWithKeyState::WithInitializer(
                event_type,
                &initializer.event_modifier_init,
                platform_time_stamp,
            ),
            key_event_: None,
            code_: initializer.code.clone(),
            key_: initializer.key.clone(),
            location_: initializer.location,
            is_composing_: initializer.is_composing,
            char_code_: initializer.char_code,
            key_code_: initializer.key_code,
        };
        if initializer.repeat {
            event.ui_event_with_key_state.modifiers_ |= WebInputEvent::kIsAutoRepeat;
        }
        event.InitLocationModifiers(initializer.location);
        event
            .BaseEventMut()
            .SetEventPathClass(crate::core::dom::events::event::EventPathClass::Keyboard);
        event
    }

    // Source: keyboard_event.h:71; keyboard_event.cc:159
    // Rust field destruction releases the same raw event and inherited owners.

    // Source: keyboard_event.h:73-83; keyboard_event.cc:161-184
    pub fn initKeyboardEvent(
        &mut self,
        script_state: &dyn KeyboardEventScriptState,
        event_type: impl Into<String>,
        bubbles: bool,
        cancelable: bool,
        view: Option<AbstractViewRef>,
        _key_identifier: &str,
        location: u32,
        ctrl_key: bool,
        alt_key: bool,
        shift_key: bool,
        meta_key: bool,
    ) {
        if self.IsBeingDispatched() {
            return;
        }
        if script_state.IsIsolatedWorld() {
            UIEventWithKeyState::DidCreateEventInIsolatedWorld(
                ctrl_key, alt_key, shift_key, meta_key,
            );
        }
        self.initUIEvent(event_type, bubbles, cancelable, view, 0);
        self.location_ = location;
        self.InitModifiers(ctrl_key, alt_key, shift_key, meta_key);
        self.InitLocationModifiers(location);
    }

    // Source: keyboard_event.h:85-90
    pub fn code(&self) -> &str {
        &self.code_
    }
    pub fn key(&self) -> &str {
        &self.key_
    }
    pub fn location(&self) -> u32 {
        self.location_
    }
    pub fn KeyEvent(&self) -> Option<&WebKeyboardEvent> {
        self.key_event_.as_deref()
    }

    // Source: keyboard_event.h:92-94; keyboard_event.cc:186-192
    pub fn keyCode(&self) -> i32 {
        self.key_code_ as i32
    }
    pub fn charCode(&self) -> i32 {
        self.char_code_ as i32
    }
    // Source: keyboard_event.h:95
    pub fn repeat(&self) -> bool {
        self.GetModifiers() & WebInputEvent::kIsAutoRepeat != 0
    }

    // Source: keyboard_event.h:97-100; keyboard_event.cc:194-207
    pub fn InterfaceName(&self) -> &str {
        "KeyboardEvent"
    }
    pub fn IsKeyboardEvent(&self) -> bool {
        true
    }
    pub fn which(&self) -> u32 {
        self.keyCode() as u32
    }
    pub fn isComposing(&self) -> bool {
        self.is_composing_
    }

    // Source: keyboard_event.h:102,118-121; keyboard_event.cc:209-214
    // The caller supplies the source DynamicTo<KeyboardEvent> projection; None
    // represents any other Event interface and retains the false guard result.
    pub fn IsEnterKeyKeydownEvent(event: Option<&Self>) -> bool {
        event.is_some_and(|event| {
            event.r#type() == "keydown" && event.key() == "Enter" && !event.repeat()
        })
    }

    // Source: keyboard_event.h:107; keyboard_event.cc:216-228
    fn InitLocationModifiers(&mut self, location: u32) {
        match location {
            value if value == KeyLocationCode::kDomKeyLocationNumpad as u32 => {
                self.ui_event_with_key_state.modifiers_ |= WebInputEvent::kIsKeyPad
            }
            value if value == KeyLocationCode::kDomKeyLocationLeft as u32 => {
                self.ui_event_with_key_state.modifiers_ |= WebInputEvent::kIsLeft
            }
            value if value == KeyLocationCode::kDomKeyLocationRight as u32 => {
                self.ui_event_with_key_state.modifiers_ |= WebInputEvent::kIsRight
            }
            _ => {}
        }
    }

    // Source: keyboard_event.h:104; keyboard_event.cc:230-232
    // Embedded base owners are retained and released by Rust; no extra traced
    // owner is introduced by this leaf event.
}

// Source: keyboard_event.cc:44-60
fn EventTypeForKeyboardEventType(event_type: Type) -> &'static str {
    match event_type {
        Type::kKeyUp => "keyup",
        Type::kRawKeyDown => "keydown",
        Type::kChar => "keypress",
        _ => unreachable!("combined keydown must be disambiguated before DOM construction"),
    }
}

// Source: keyboard_event.cc:62-70
fn GetKeyLocationCode(key: &WebInputEvent) -> KeyLocationCode {
    if key.GetModifiers() & WebInputEvent::kIsKeyPad != 0 {
        return KeyLocationCode::kDomKeyLocationNumpad;
    }
    if key.GetModifiers() & WebInputEvent::kIsLeft != 0 {
        return KeyLocationCode::kDomKeyLocationLeft;
    }
    if key.GetModifiers() & WebInputEvent::kIsRight != 0 {
        return KeyLocationCode::kDomKeyLocationRight;
    }
    KeyLocationCode::kDomKeyLocationStandard
}

// Source: keyboard_event.cc:72-79
fn HasCurrentComposition(dom_window: Option<&dyn KeyboardEventWindow>) -> bool {
    dom_window
        .and_then(KeyboardEventWindow::FrameHasComposition)
        .unwrap_or(false)
}
// Source: keyboard_event.cc:81-83
// KeycodeConverter's typed port already returns a decoded Rust String, carrying
// exactly the source UTF-8 conversion result at the package boundary.

impl Deref for KeyboardEvent {
    type Target = UIEventWithKeyState;
    fn deref(&self) -> &Self::Target {
        &self.ui_event_with_key_state
    }
}
impl DerefMut for KeyboardEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ui_event_with_key_state
    }
}
impl EventInterface for KeyboardEvent {
    fn AsKeyboardEvent(&self) -> Option<&KeyboardEvent> {
        Some(self)
    }
    fn AsUIEventWithKeyState(&self) -> Option<&UIEventWithKeyState> {
        Some(&self.ui_event_with_key_state)
    }
    fn BaseEvent(&self) -> &Event {
        self.ui_event_with_key_state.BaseEvent()
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        self.ui_event_with_key_state.BaseEventMut()
    }
    fn InterfaceName(&self) -> &str {
        KeyboardEvent::InterfaceName(self)
    }
    fn IsUIEvent(&self) -> bool {
        true
    }
    fn IsKeyboardEvent(&self) -> bool {
        true
    }
    // Source has no KeyboardEvent override of CanBeDispatchedInWorld; the
    // inherited Event default is retained.
}
impl UIEventInterface for KeyboardEvent {
    fn BaseUIEvent(&self) -> &UIEvent {
        self.ui_event_with_key_state.BaseUIEvent()
    }
    fn BaseUIEventMut(&mut self) -> &mut UIEvent {
        self.ui_event_with_key_state.BaseUIEventMut()
    }
    fn which(&self) -> u32 {
        KeyboardEvent::which(self)
    }
}
impl UIEventWithKeyStateInterface for KeyboardEvent {
    fn BaseUIEventWithKeyState(&self) -> &UIEventWithKeyState {
        &self.ui_event_with_key_state
    }
    fn BaseUIEventWithKeyStateMut(&mut self) -> &mut UIEventWithKeyState {
        &mut self.ui_event_with_key_state
    }
}
