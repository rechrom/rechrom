// Copyright 2013 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: cc/input/touch_action.h:24-65.
/// Chromium cc::TouchAction bit value, intersected over the touch target chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TouchAction(pub u8);
impl TouchAction {
    pub const kNone: Self = Self(0);
    pub const kPanLeft: Self = Self(1);
    pub const kPanRight: Self = Self(2);
    pub const kPanX: Self = Self(3);
    pub const kPanUp: Self = Self(4);
    pub const kPanDown: Self = Self(8);
    pub const kPanY: Self = Self(12);
    pub const kPan: Self = Self(15);
    pub const kPinchZoom: Self = Self(16);
    pub const kManipulation: Self = Self(31);
    pub const kDoubleTapZoom: Self = Self(32);
    pub const kInternalPanXScrolls: Self = Self(64);
    pub const kInternalNotWritable: Self = Self(128);
    pub const kAuto: Self = Self(255);
    pub(crate) fn has(self, bits: Self) -> bool {
        self.0 & bits.0 != 0
    }
}
impl Default for TouchAction {
    fn default() -> Self {
        Self::kAuto
    }
}
