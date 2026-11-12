// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/wheel_event_init.idl:1-17

use super::mouse_event_init::MouseEventInit;

// Source: wheel_event_init.idl:7-17
#[derive(Clone, Default)]
pub struct WheelEventInit {
    pub mouse_event_init: MouseEventInit,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub delta_mode: u32,
    // The binding applies RuntimeEnabled=WheelEventMomentum when projecting
    // this initializer; the event value preserves the supplied field.
    pub momentum: bool,
    pub wheel_delta_x: i32,
    pub wheel_delta_y: i32,
}
