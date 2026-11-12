// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/common/input/web_pointer_properties.h:1-127
// Header-only source. Rust modules replace its includes and namespace.

pub use super::pointer_id::PointerId;
pub use crate::public::mojom::input::input_event::Button;
pub use crate::ui::events::event_constants::EventPointerType as PointerType;
pub use foundation::gfx_geometry::PointF;

// Source: third_party/blink/public/common/input/web_pointer_properties.h:29-37
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Buttons {
    kNoButton = 0,
    kLeft = 1 << 0,
    kRight = 1 << 1,
    kMiddle = 1 << 2,
    kBack = 1 << 3,
    kForward = 1 << 4,
    kEraser = 1 << 5,
}

// Source: third_party/blink/public/common/input/web_pointer_properties.h:18-23,74-122
#[derive(Clone, Debug)]
pub struct WebPointerProperties {
    pub id: PointerId,
    /// NaN means the input device does not support pressure.
    pub force: f32,
    pub tilt_x: f64,
    pub tilt_y: f64,
    pub tangential_pressure: f32,
    pub twist: i32,
    pub button: Button,
    pub pointer_type: PointerType,
    pub movement_x: i32,
    pub movement_y: i32,
    pub is_raw_movement_event: bool,
    pub device_id: i32,
    position_in_widget_: PointF,
    position_in_screen_: PointF,
}

impl WebPointerProperties {
    // Source: third_party/blink/public/common/input/web_pointer_properties.h:39-53,76-94,108-113
    pub fn new(
        id: PointerId,
        pointer_type: PointerType,
        button: Button,
        position_in_widget: PointF,
        position_in_screen: PointF,
        movement_x: i32,
        movement_y: i32,
    ) -> Self {
        Self {
            id,
            force: f32::NAN,
            tilt_x: 0.0,
            tilt_y: 0.0,
            tangential_pressure: 0.0,
            twist: 0,
            button,
            pointer_type,
            movement_x,
            movement_y,
            is_raw_movement_event: false,
            device_id: -1,
            position_in_widget_: position_in_widget,
            position_in_screen_: position_in_screen,
        }
    }

    // Source: third_party/blink/public/common/input/web_pointer_properties.h:39-46
    // Rust expresses the C++ constructor's omitted default arguments explicitly.
    pub fn WithId(id: PointerId) -> Self {
        Self::new(
            id,
            PointerType::kUnknown,
            Button::kNoButton,
            PointF::default(),
            PointF::default(),
            0,
            0,
        )
    }

    // Source: third_party/blink/public/common/input/web_pointer_properties.h:55
    pub fn PositionInWidget(&self) -> &PointF {
        &self.position_in_widget_
    }

    // Source: third_party/blink/public/common/input/web_pointer_properties.h:56
    pub fn PositionInScreen(&self) -> &PointF {
        &self.position_in_screen_
    }

    // Source: third_party/blink/public/common/input/web_pointer_properties.h:58-60
    pub fn SetPositionInWidgetXY(&mut self, x: f32, y: f32) {
        self.position_in_widget_ = PointF::new(x, y);
    }

    // Source: third_party/blink/public/common/input/web_pointer_properties.h:62-64
    pub fn SetPositionInScreenXY(&mut self, x: f32, y: f32) {
        self.position_in_screen_ = PointF::new(x, y);
    }

    // Source: third_party/blink/public/common/input/web_pointer_properties.h:66-68
    pub fn SetPositionInWidget(&mut self, point: PointF) {
        self.position_in_widget_ = point;
    }

    // Source: third_party/blink/public/common/input/web_pointer_properties.h:70-72
    pub fn SetPositionInScreen(&mut self, point: PointF) {
        self.position_in_screen_ = point;
    }
}
