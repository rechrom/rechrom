// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/mouse_event_init.idl:1-24

use crate::core::dom::events::event::EventTargetRef;
use crate::core::dom::events::event_modifier_init::EventModifierInit;

// Source: mouse_event_init.idl:7-24
#[derive(Clone, Default)]
pub struct MouseEventInit {
    pub event_modifier_init: EventModifierInit,
    pub screen_x: f64,
    pub screen_y: f64,
    pub client_x: f64,
    pub client_y: f64,
    pub button: i16,
    pub buttons: u16,
    pub related_target: Option<EventTargetRef>,
    pub movement_x: i32,
    pub movement_y: i32,
    pub region: Option<String>,
}
