// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/common/input/web_touch_point.h:42-64

use super::web_pointer_event::WebTouchPointView;
use super::web_pointer_properties::WebPointerProperties;
pub use crate::public::mojom::input::touch_event::TouchState;

#[derive(Clone, Debug)]
pub struct WebTouchPoint {
    pub pointer_properties: WebPointerProperties,
    pub state: TouchState,
    pub radius_x: f32,
    pub radius_y: f32,
    pub rotation_angle: f32,
}

impl Default for WebTouchPoint {
    fn default() -> Self {
        Self::new(WebPointerProperties::WithId(0))
    }
}

impl WebTouchPoint {
    pub fn new(pointer_properties: WebPointerProperties) -> Self {
        Self {
            pointer_properties,
            state: TouchState::kStateUndefined,
            radius_x: 0.0,
            radius_y: 0.0,
            rotation_angle: 0.0,
        }
    }
}

impl WebTouchPointView for WebTouchPoint {
    fn PointerProperties(&self) -> &WebPointerProperties {
        &self.pointer_properties
    }
    fn State(&self) -> TouchState {
        self.state
    }
    fn RadiusX(&self) -> f32 {
        self.radius_x
    }
    fn RadiusY(&self) -> f32 {
        self.radius_y
    }
    fn RotationAngle(&self) -> f32 {
        self.rotation_angle
    }
}
