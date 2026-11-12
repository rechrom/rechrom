// Copyright 2013 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: components/input/touchpad_tap_suppression_controller.cc:10-22.
// Both source delegates operate on the same shared suppression state.
use super::tap_suppression_controller::TapSuppressionController;
use crate::base::time::time::TimeTicks;

impl TapSuppressionController {
    pub fn ShouldSuppressMouseDown(&mut self, now: TimeTicks) -> bool {
        self.ShouldSuppressTapDown(now)
    }
    pub fn ShouldSuppressMouseUp(&self) -> bool {
        self.ShouldSuppressTapEnd()
    }
}
