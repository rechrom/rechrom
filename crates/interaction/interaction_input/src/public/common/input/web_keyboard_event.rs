// Copyright 2017 The Chromium Authors
// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/common/input/web_keyboard_event.h:1-123
// Source: third_party/blink/common/input/web_keyboard_event.cc:1-23
// Source .h:5-17,119-123 and .cc:5-7,23: Rust modules and embedded base state
// replace C++ includes, include guards, export annotations, and inheritance.

use std::any::Any;
use std::ops::{Deref, DerefMut};

use super::web_input_event::{Type, WebInputEvent, WebInputEventPayload};
use crate::base::time::time::TimeTicks;

// Source: third_party/blink/public/common/input/web_keyboard_event.h:17-88
#[derive(Clone, Debug)]
pub struct WebKeyboardEvent {
    event: WebInputEvent,

    // Source: third_party/blink/public/common/input/web_keyboard_event.h:23-31
    // Keep the Windows-equivalent non-locational key code on every platform;
    // left/right/numpad location is carried by the base event modifiers.
    pub windows_key_code: i32,

    // Source: third_party/blink/public/common/input/web_keyboard_event.h:33-36
    pub native_key_code: i32,

    // Source: third_party/blink/public/common/input/web_keyboard_event.h:38-40
    pub dom_code: i32,

    // Source: third_party/blink/public/common/input/web_keyboard_event.h:42-44
    pub dom_key: u32,

    // Source: third_party/blink/public/common/input/web_keyboard_event.h:46-51
    pub is_system_key: bool,

    // Source: third_party/blink/public/common/input/web_keyboard_event.h:53-56
    pub is_browser_shortcut: bool,

    // Source: third_party/blink/public/common/input/web_keyboard_event.h:58-79
    // BUILDFLAG(IS_ANDROID) maps to Rust's target OS condition. A true value
    // guarantees physical keyboard input; false leaves the source ambiguous.
    #[cfg(target_os = "android")]
    pub is_confirmed_physical_keyboard_input: bool,

    // Source: third_party/blink/public/common/input/web_keyboard_event.h:81-88
    // char16_t arrays retain UTF-16 code units and their fixed capacity.
    pub text: [u16; Self::kTextLengthCap],
    pub unmodified_text: [u16; Self::kTextLengthCap],
}

impl WebKeyboardEvent {
    // Source: third_party/blink/public/common/input/web_keyboard_event.h:19-21
    // Source: third_party/blink/common/input/web_keyboard_event.cc:9
    // Rust associated constants also supply the source's out-of-line constant.
    pub const kTextLengthCap: usize = 4;

    // Source: third_party/blink/public/common/input/web_keyboard_event.h:90-95
    // Source: third_party/blink/public/common/input/web_keyboard_event.h:23-88
    // Source default member initializers are applied by every constructor.
    pub fn new(event_type: Type, modifiers: i32, time_stamp: TimeTicks) -> Self {
        Self {
            event: WebInputEvent::new(
                event_type,
                Type::kKeyboardTypeFirst,
                Type::kKeyboardTypeLast,
                modifiers,
                time_stamp,
            ),
            windows_key_code: 0,
            native_key_code: 0,
            dom_code: 0,
            dom_key: 0,
            is_system_key: false,
            is_browser_shortcut: false,
            #[cfg(target_os = "android")]
            is_confirmed_physical_keyboard_input: false,
            text: [0; Self::kTextLengthCap],
            unmodified_text: [0; Self::kTextLengthCap],
        }
    }
}

// Source: third_party/blink/public/common/input/web_keyboard_event.h:97-100
impl Default for WebKeyboardEvent {
    fn default() -> Self {
        Self::new(
            Type::kUndefined,
            WebInputEvent::kNoModifiers,
            TimeTicks::new(),
        )
    }
}

impl WebKeyboardEvent {
    // Source: third_party/blink/public/common/input/web_keyboard_event.h:102-104
    // Clone, CanCoalesce, and Coalesce are defined in the .cc translation below.

    // Source: third_party/blink/public/common/input/web_keyboard_event.h:106-118
    pub fn IsCharacterKey(&self) -> bool {
        match self.windows_key_code {
            0x08 | 0x1b => false, // VK_BACK, VK_ESCAPE
            _ => true,
        }
    }
}

// Source: third_party/blink/public/common/input/web_keyboard_event.h:17
// Rust dereferencing exposes the public inherited base-event operations.
impl Deref for WebKeyboardEvent {
    type Target = WebInputEvent;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl DerefMut for WebKeyboardEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}

impl WebInputEventPayload for WebKeyboardEvent {
    // Source: third_party/blink/public/common/input/web_keyboard_event.h:17
    // Embedded-base access and checked downcasts implement C++ base identity.
    fn Event(&self) -> &WebInputEvent {
        &self.event
    }

    fn EventMut(&mut self) -> &mut WebInputEvent {
        &mut self.event
    }

    fn AsAny(&self) -> &dyn Any {
        self
    }

    fn AsAnyMut(&mut self) -> &mut dyn Any {
        self
    }

    // Source: third_party/blink/common/input/web_keyboard_event.cc:11-13
    fn Clone(&self) -> Box<dyn WebInputEventPayload> {
        Box::new(self.clone())
    }

    // Source: third_party/blink/common/input/web_keyboard_event.cc:15-17
    fn CanCoalesce(&self, _event: &dyn WebInputEventPayload) -> bool {
        false
    }

    // Source: third_party/blink/common/input/web_keyboard_event.cc:19-21
    fn Coalesce(&mut self, _event: &dyn WebInputEventPayload) {
        unreachable!("WebKeyboardEvent cannot coalesce");
    }
}
