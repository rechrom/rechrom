// Copyright 2013 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: components/input/touchscreen_tap_suppression_controller.cc:14-41.
// The shared source base keeps the sole suppression state. This impl is its
// existing flattened touchscreen specialization and preserves its public path.
use super::tap_suppression_controller::TapSuppressionController;
use crate::{base::time::time::TimeTicks, public::common::input::web_input_event::Type};

impl TapSuppressionController {
    pub fn FilterTouchscreenTapEvent(&mut self, event_type: Type, now: TimeTicks) -> bool {
        match event_type {
            Type::kGestureTapDown => self.ShouldSuppressTapDown(now),
            Type::kGestureShowPress
            | Type::kGestureShortPress
            | Type::kGestureLongPress
            | Type::kGestureTapUnconfirmed
            | Type::kGestureTapCancel
            | Type::kGestureTap
            | Type::kGestureDoubleTap
            | Type::kGestureLongTap
            | Type::kGestureTwoFingerTap => self.ShouldSuppressTapEnd(),
            _ => false,
        }
    }
}
