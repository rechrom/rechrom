// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/keyboard_event_init.idl:1-17

use crate::core::dom::events::event_modifier_init::EventModifierInit;

// Source: keyboard_event_init.idl:7-17
#[derive(Clone, Default)]
pub struct KeyboardEventInit {
    pub event_modifier_init: EventModifierInit,
    pub key: String,
    pub code: String,
    pub location: u32,
    pub repeat: bool,
    pub is_composing: bool,
    pub char_code: u32,
    pub key_code: u32,
}
