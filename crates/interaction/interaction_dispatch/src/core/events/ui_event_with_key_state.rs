/*
 * Copyright (C) 2001 Peter Kelly (pmk@post.com)
 * Copyright (C) 2001 Tobias Anton (anton@stud.fbi.fh-darmstadt.de)
 * Copyright (C) 2006 Samuel Weinig (sam.weinig@gmail.com)
 * Copyright (C) 2003, 2004, 2005, 2006, 2008 Apple Inc. All rights reserved.
 * Copyright (C) 2006 Apple Computer, Inc.
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
// Source: third_party/blink/renderer/core/events/ui_event_with_key_state.h:1-99
// Source: third_party/blink/renderer/core/events/ui_event_with_key_state.cc:1-166
// Embedded state and Rust modules replace inheritance/includes/namespaces.

use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::web_input_event::WebInputEvent;

use super::ui_event::{AbstractViewRef, InputDeviceCapabilitiesRef, UIEvent, UIEventInterface};
use crate::core::dom::events::event::{
    Bubbles, Cancelable, ComposedMode, Event, EventInterface, EventRef,
};
pub use crate::core::dom::events::event_modifier_init::EventModifierInit;

// Source: ui_event_with_key_state.h:35-36,89
pub struct UIEventWithKeyState {
    ui_event: UIEvent,
    pub(crate) modifiers_: i32,
}

impl UIEventWithKeyState {
    // Source: ui_event_with_key_state.h:37-40
    pub fn ctrlKey(&self) -> bool {
        self.modifiers_ & WebInputEvent::kControlKey != 0
    }
    pub fn shiftKey(&self) -> bool {
        self.modifiers_ & WebInputEvent::kShiftKey != 0
    }
    pub fn altKey(&self) -> bool {
        self.modifiers_ & WebInputEvent::kAltKey != 0
    }
    pub fn metaKey(&self) -> bool {
        self.modifiers_ & WebInputEvent::kMetaKey != 0
    }

    // Source: ui_event_with_key_state.h:42-52
    pub fn NewTabModifierSetFromIsolatedWorld() -> bool {
        NEW_TAB_MODIFIER_SET_FROM_ISOLATED_WORLD.load(Ordering::Relaxed)
    }
    pub fn ClearNewTabModifierSetFromIsolatedWorld() {
        NEW_TAB_MODIFIER_SET_FROM_ISOLATED_WORLD.store(false, Ordering::Relaxed);
    }

    // Source: ui_event_with_key_state.h:63-65
    pub fn GetModifiers(&self) -> i32 {
        self.modifiers_
    }

    // Source: ui_event_with_key_state.h:68
    pub fn Uninitialized(platform_time_stamp: TimeTicks) -> Self {
        Self {
            ui_event: UIEvent::Uninitialized(platform_time_stamp),
            modifiers_: 0,
        }
    }

    // Source: ui_event_with_key_state.h:70-77; ui_event_with_key_state.cc:28-45
    pub fn new(
        event_type: impl Into<String>,
        bubbles: Bubbles,
        cancelable: Cancelable,
        view: Option<AbstractViewRef>,
        detail: i32,
        modifiers: i32,
        platform_time_stamp: TimeTicks,
        source_capabilities: Option<InputDeviceCapabilitiesRef>,
    ) -> Self {
        Self {
            ui_event: UIEvent::new(
                event_type,
                bubbles,
                cancelable,
                ComposedMode::kComposed,
                platform_time_stamp,
                view,
                detail,
                source_capabilities,
            ),
            modifiers_: modifiers,
        }
    }

    // Source: ui_event_with_key_state.h:78-83; ui_event_with_key_state.cc:47-71
    pub fn WithInitializer(
        event_type: impl Into<String>,
        initializer: &EventModifierInit,
        platform_time_stamp: TimeTicks,
    ) -> Self {
        let mut result = Self {
            ui_event: UIEvent::WithInitializer(
                event_type,
                &initializer.ui_event_init,
                platform_time_stamp,
            ),
            modifiers_: 0,
        };
        if initializer.ctrl_key {
            result.modifiers_ |= WebInputEvent::kControlKey;
        }
        if initializer.shift_key {
            result.modifiers_ |= WebInputEvent::kShiftKey;
        }
        if initializer.alt_key {
            result.modifiers_ |= WebInputEvent::kAltKey;
        }
        if initializer.meta_key {
            result.modifiers_ |= WebInputEvent::kMetaKey;
        }
        if initializer.modifier_alt_graph {
            result.modifiers_ |= WebInputEvent::kAltGrKey;
        }
        if initializer.modifier_fn {
            result.modifiers_ |= WebInputEvent::kFnKey;
        }
        if initializer.modifier_caps_lock {
            result.modifiers_ |= WebInputEvent::kCapsLockOn;
        }
        if initializer.modifier_scroll_lock {
            result.modifiers_ |= WebInputEvent::kScrollLockOn;
        }
        if initializer.modifier_num_lock {
            result.modifiers_ |= WebInputEvent::kNumLockOn;
        }
        if initializer.modifier_symbol {
            result.modifiers_ |= WebInputEvent::kSymbolKey;
        }
        result
    }

    // Source: ui_event_with_key_state.h:53-56; ui_event_with_key_state.cc:75-85
    pub fn DidCreateEventInIsolatedWorld(
        ctrl_key: bool,
        _shift_key: bool,
        _alt_key: bool,
        meta_key: bool,
    ) {
        #[cfg(target_os = "macos")]
        let new_tab_modifier_set = meta_key;
        #[cfg(not(target_os = "macos"))]
        let new_tab_modifier_set = ctrl_key;
        let _ = (ctrl_key, meta_key);
        NEW_TAB_MODIFIER_SET_FROM_ISOLATED_WORLD.fetch_or(new_tab_modifier_set, Ordering::Relaxed);
    }

    // Source: ui_event_with_key_state.h:58-59; ui_event_with_key_state.cc:87-110
    pub fn SetFromWebInputEventModifiers(initializer: &mut EventModifierInit, modifiers: i32) {
        if modifiers & WebInputEvent::kControlKey != 0 {
            initializer.ctrl_key = true;
        }
        if modifiers & WebInputEvent::kShiftKey != 0 {
            initializer.shift_key = true;
        }
        if modifiers & WebInputEvent::kAltKey != 0 {
            initializer.alt_key = true;
        }
        if modifiers & WebInputEvent::kMetaKey != 0 {
            initializer.meta_key = true;
        }
        if modifiers & WebInputEvent::kAltGrKey != 0 {
            initializer.modifier_alt_graph = true;
        }
        if modifiers & WebInputEvent::kFnKey != 0 {
            initializer.modifier_fn = true;
        }
        if modifiers & WebInputEvent::kCapsLockOn != 0 {
            initializer.modifier_caps_lock = true;
        }
        if modifiers & WebInputEvent::kScrollLockOn != 0 {
            initializer.modifier_scroll_lock = true;
        }
        if modifiers & WebInputEvent::kNumLockOn != 0 {
            initializer.modifier_num_lock = true;
        }
        if modifiers & WebInputEvent::kSymbolKey != 0 {
            initializer.modifier_symbol = true;
        }
    }

    // Source: ui_event_with_key_state.h:61; ui_event_with_key_state.cc:112-141
    pub fn getModifierState(&self, key_identifier: &str) -> bool {
        #[cfg(target_os = "macos")]
        const ACCEL: i32 = WebInputEvent::kMetaKey;
        #[cfg(not(target_os = "macos"))]
        const ACCEL: i32 = WebInputEvent::kControlKey;
        const IDENTIFIERS: &[(&str, i32)] = &[
            ("Shift", WebInputEvent::kShiftKey),
            ("Control", WebInputEvent::kControlKey),
            ("Alt", WebInputEvent::kAltKey),
            ("Meta", WebInputEvent::kMetaKey),
            ("AltGraph", WebInputEvent::kAltGrKey),
            ("Accel", ACCEL),
            ("Fn", WebInputEvent::kFnKey),
            ("CapsLock", WebInputEvent::kCapsLockOn),
            ("ScrollLock", WebInputEvent::kScrollLockOn),
            ("NumLock", WebInputEvent::kNumLockOn),
            ("Symbol", WebInputEvent::kSymbolKey),
        ];
        for &(identifier, mask) in IDENTIFIERS {
            if key_identifier == identifier {
                return self.modifiers_ & mask != 0;
            }
        }
        false
    }

    // Source: ui_event_with_key_state.h:84-87; ui_event_with_key_state.cc:143-156
    pub fn InitModifiers(
        &mut self,
        ctrl_key: bool,
        alt_key: bool,
        shift_key: bool,
        meta_key: bool,
    ) {
        self.modifiers_ = 0;
        if ctrl_key {
            self.modifiers_ |= WebInputEvent::kControlKey;
        }
        if alt_key {
            self.modifiers_ |= WebInputEvent::kAltKey;
        }
        if shift_key {
            self.modifiers_ |= WebInputEvent::kShiftKey;
        }
        if meta_key {
            self.modifiers_ |= WebInputEvent::kMetaKey;
        }
    }
}

// Source: ui_event_with_key_state.h:92; ui_event_with_key_state.cc:73
// This source process-wide flag is a single atomic value; no event owner or
// additional scheduling state is introduced by the Rust synchronization.
static NEW_TAB_MODIFIER_SET_FROM_ISOLATED_WORLD: AtomicBool = AtomicBool::new(false);

// Source: ui_event_with_key_state.h:95; ui_event_with_key_state.cc:158-164
// Returning the original EventRef retains the same derived owner. Its key-state
// base can then be projected by the concrete event binding without copying it.
pub fn FindEventWithKeyState(mut event: Option<EventRef>) -> Option<EventRef> {
    while let Some(current) = event {
        let (is_key_state, underlying) = {
            let borrowed = current.borrow();
            (
                borrowed.IsKeyboardEvent() || borrowed.IsMouseEvent() || borrowed.IsPointerEvent(),
                borrowed.BaseEvent().UnderlyingEvent(),
            )
        };
        if is_key_state {
            return Some(current);
        }
        event = underlying;
    }
    None
}

impl Deref for UIEventWithKeyState {
    type Target = UIEvent;
    fn deref(&self) -> &Self::Target {
        &self.ui_event
    }
}
impl DerefMut for UIEventWithKeyState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ui_event
    }
}
impl EventInterface for UIEventWithKeyState {
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
}
impl UIEventInterface for UIEventWithKeyState {
    fn BaseUIEvent(&self) -> &UIEvent {
        &self.ui_event
    }
    fn BaseUIEventMut(&mut self) -> &mut UIEvent {
        &mut self.ui_event
    }
}

// Source: ui_event_with_key_state.h:35,89
// Concrete event classes retain and expose their existing key-state base.
pub trait UIEventWithKeyStateInterface: UIEventInterface {
    fn BaseUIEventWithKeyState(&self) -> &UIEventWithKeyState;
    fn BaseUIEventWithKeyStateMut(&mut self) -> &mut UIEventWithKeyState;
}
impl UIEventWithKeyStateInterface for UIEventWithKeyState {
    fn BaseUIEventWithKeyState(&self) -> &UIEventWithKeyState {
        self
    }
    fn BaseUIEventWithKeyStateMut(&mut self) -> &mut UIEventWithKeyState {
        self
    }
}
