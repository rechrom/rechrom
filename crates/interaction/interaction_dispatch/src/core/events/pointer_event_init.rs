// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/pointer_event_init.idl:1-25

use super::mouse_event_init::MouseEventInit;
use super::pointer_event::PointerEventRef;

// Source: pointer_event_init.idl:7-25
// Options retain generated dictionary presence; IDL defaults count as present.
#[derive(Clone)]
pub struct PointerEventInit {
    pub mouse_event_init: MouseEventInit,
    pub pointer_id: Option<i32>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub pressure: Option<f32>,
    pub tilt_x: Option<i32>,
    pub tilt_y: Option<i32>,
    pub azimuth_angle: Option<f64>,
    pub altitude_angle: Option<f64>,
    pub tangential_pressure: Option<f32>,
    pub twist: Option<i32>,
    pub pointer_type: Option<String>,
    pub is_primary: Option<bool>,
    pub persistent_device_id: Option<i32>,
    pub coalesced_events: Option<Vec<PointerEventRef>>,
    pub predicted_events: Option<Vec<PointerEventRef>>,
}

impl Default for PointerEventInit {
    fn default() -> Self {
        Self {
            mouse_event_init: MouseEventInit::default(),
            pointer_id: Some(0),
            width: Some(1.0),
            height: Some(1.0),
            pressure: Some(0.0),
            tilt_x: None,
            tilt_y: None,
            azimuth_angle: None,
            altitude_angle: None,
            tangential_pressure: Some(0.0),
            twist: Some(0),
            pointer_type: Some(String::new()),
            is_primary: Some(false),
            persistent_device_id: Some(0),
            coalesced_events: Some(Vec::new()),
            predicted_events: Some(Vec::new()),
        }
    }
}
