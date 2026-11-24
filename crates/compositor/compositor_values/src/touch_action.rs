// Translation status: complete
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/touch_action.h:1-13,214-217.
// Rust module scope replaces includes, guards and namespace cc.
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

// Source: cc/input/touch_action.h:14-20.
// The current touch action specifies what accelerated browser operations
// (panning and zooming) are currently permitted via touch input.
// See http://www.w3.org/TR/pointerevents/#the-touch-action-css-property.
// This is the single canonical enum definition shared by Blink/content.
pub const kTouchActionBits: usize = 8;

// Source: cc/input/touch_action.h:21-24,51-52.
// A newtype retains all C++ enum bit patterns, including operator~ results.
// A Rust enum would reject composite/unnamed bit patterns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct TouchAction(i32);
impl TouchAction {
    pub const kNone: Self = Self(0x0);
    // Source: cc/input/touch_action.h:25-34.
    pub const kPanLeft: Self = Self(0x1);
    pub const kPanRight: Self = Self(0x2);
    pub const kPanX: Self = Self(Self::kPanLeft.0 | Self::kPanRight.0);
    pub const kPanUp: Self = Self(0x4);
    pub const kPanDown: Self = Self(0x8);
    pub const kPanY: Self = Self(Self::kPanUp.0 | Self::kPanDown.0);
    pub const kPan: Self = Self(Self::kPanX.0 | Self::kPanY.0);
    pub const kPinchZoom: Self = Self(0x10);
    pub const kManipulation: Self = Self(Self::kPan.0 | Self::kPinchZoom.0);
    pub const kDoubleTapZoom: Self = Self(0x20);

    // Source: cc/input/touch_action.h:35-42.
    // Used internally by swipe-to-move-cursor. Set for elements where the
    // feature must be disabled; not set for contenteditable without a horizontal
    // scrollable ancestor (including itself). This is not web-visible.
    pub const kInternalPanXScrolls: Self = Self(0x40);

    // Source: cc/input/touch_action.h:43-47.
    // Used internally by stylus handwriting. Writing is not started with this
    // bit set; non-password edit fields with kPan do not set it.
    pub const kInternalNotWritable: Self = Self(0x80);

    // Source: cc/input/touch_action.h:48-50.
    pub const kAuto: Self = Self(
        Self::kManipulation.0
            | Self::kDoubleTapZoom.0
            | Self::kInternalPanXScrolls.0
            | Self::kInternalNotWritable.0,
    );
    pub const kMax: Self = Self((1 << kTouchActionBits) - 1);
}

// Source: cc/input/touch_action.h:53-56.
impl BitOr for TouchAction {
    type Output = Self;
    fn bitor(self, b: Self) -> Self {
        Self(self.0 | b.0)
    }
}
// Source: cc/input/touch_action.h:57-60.
impl BitOrAssign for TouchAction {
    fn bitor_assign(&mut self, b: Self) {
        *self = *self | b;
    }
}
// Source: cc/input/touch_action.h:61-64.
impl BitAnd for TouchAction {
    type Output = Self;
    fn bitand(self, b: Self) -> Self {
        Self(self.0 & b.0)
    }
}
// Source: cc/input/touch_action.h:65-68.
impl BitAndAssign for TouchAction {
    fn bitand_assign(&mut self, b: Self) {
        *self = *self & b;
    }
}
// Source: cc/input/touch_action.h:69-72.
impl Not for TouchAction {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

// Source: cc/input/touch_action.h:73-82.
pub fn TouchActionToString(mut touch_action: TouchAction) -> &'static str {
    // Skip internal pan-x-scrolls; this is not a web-exposed touch action field.
    touch_action &= !TouchAction::kInternalPanXScrolls;
    // Skip internal not-writable; this is not a web-exposed touch action field.
    touch_action &= !TouchAction::kInternalNotWritable;
    match touch_action.0 {
        // Source: cc/input/touch_action.h:83-84.
        0 => "NONE",
        // Source: cc/input/touch_action.h:85-86.
        1 => "PAN_LEFT",
        // Source: cc/input/touch_action.h:87-88.
        2 => "PAN_RIGHT",
        // Source: cc/input/touch_action.h:89-90.
        3 => "PAN_X",
        // Source: cc/input/touch_action.h:91-92.
        4 => "PAN_UP",
        // Source: cc/input/touch_action.h:93-94.
        5 => "PAN_LEFT_PAN_UP",
        // Source: cc/input/touch_action.h:95-96.
        6 => "PAN_RIGHT_PAN_UP",
        // Source: cc/input/touch_action.h:97-98.
        7 => "PAN_X_PAN_UP",
        // Source: cc/input/touch_action.h:99-100.
        8 => "PAN_DOWN",
        // Source: cc/input/touch_action.h:101-102.
        9 => "PAN_LEFT_PAN_DOWN",
        // Source: cc/input/touch_action.h:103-104.
        10 => "PAN_RIGHT_PAN_DOWN",
        // Source: cc/input/touch_action.h:105-106.
        11 => "PAN_X_PAN_DOWN",
        // Source: cc/input/touch_action.h:107-108.
        12 => "PAN_Y",
        // Source: cc/input/touch_action.h:109-110.
        13 => "PAN_LEFT_PAN_Y",
        // Source: cc/input/touch_action.h:111-112.
        14 => "PAN_RIGHT_PAN_Y",
        // Source: cc/input/touch_action.h:113-114.
        15 => "PAN_X_PAN_Y",
        // Source: cc/input/touch_action.h:115-116.
        16 => "PINCH_ZOOM",
        // Source: cc/input/touch_action.h:117-118.
        17 => "PAN_LEFT_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:119-120.
        18 => "PAN_RIGHT_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:121-122.
        19 => "PAN_X_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:123-124.
        20 => "PAN_UP_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:125-126.
        21 => "PAN_LEFT_PAN_UP_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:127-128.
        22 => "PAN_RIGHT_PAN_UP_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:129-130.
        23 => "PAN_X_PAN_UP_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:131-132.
        24 => "PAN_DOWN_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:133-134.
        25 => "PAN_LEFT_PAN_DOWN_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:135-136.
        26 => "PAN_RIGHT_PAN_DOWN_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:137-138.
        27 => "PAN_X_PAN_DOWN_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:139-140.
        28 => "PAN_Y_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:141-142.
        29 => "PAN_LEFT_PAN_Y_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:143-144.
        30 => "PAN_RIGHT_PAN_Y_PINCH_ZOOM",
        // Source: cc/input/touch_action.h:145-146.
        31 => "MANIPULATION",
        // Source: cc/input/touch_action.h:147-148.
        32 => "DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:149-150.
        33 => "PAN_LEFT_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:151-152.
        34 => "PAN_RIGHT_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:153-154.
        35 => "PAN_X_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:155-156.
        36 => "PAN_UP_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:157-158.
        37 => "PAN_LEFT_PAN_UP_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:159-160.
        38 => "PAN_RIGHT_PAN_UP_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:161-162.
        39 => "PAN_X_PAN_UP_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:163-164.
        40 => "PAN_DOWN_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:165-166.
        41 => "PAN_LEFT_PAN_DOWN_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:167-168.
        42 => "PAN_RIGHT_PAN_DOWN_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:169-170.
        43 => "PAN_X_PAN_DOWN_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:171-172.
        44 => "PAN_Y_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:173-174.
        45 => "PAN_LEFT_PAN_Y_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:175-176.
        46 => "PAN_RIGHT_PAN_Y_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:177-178.
        47 => "PAN_X_PAN_Y_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:179-180.
        48 => "PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:181-182.
        49 => "PAN_LEFT_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:183-184.
        50 => "PAN_RIGHT_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:185-186.
        51 => "PAN_X_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:187-188.
        52 => "PAN_UP_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:189-190.
        53 => "PAN_LEFT_PAN_UP_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:191-192.
        54 => "PAN_RIGHT_PAN_UP_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:193-194.
        55 => "PAN_X_PAN_UP_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:195-196.
        56 => "PAN_DOWN_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:197-198.
        57 => "PAN_LEFT_PAN_DOWN_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:199-200.
        58 => "PAN_RIGHT_PAN_DOWN_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:201-202.
        59 => "PAN_X_PAN_DOWN_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:203-204.
        60 => "PAN_Y_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:205-206.
        61 => "PAN_LEFT_PAN_Y_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:207-208.
        62 => "PAN_RIGHT_PAN_Y_PINCH_ZOOM_DOUBLE_TAP_ZOOM",
        // Source: cc/input/touch_action.h:209-210.
        63 => "AUTO",
        // Source: cc/input/touch_action.h:211-213.
        _ => unreachable!("NOTREACHED: invalid TouchAction bit pattern"),
    }
}
