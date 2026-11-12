// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15

// Source: third_party/blink/public/mojom/input/input_event.mojom:7-17
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Button {
    kNoButton = -1,
    kLeft,
    kMiddle,
    kRight,
    kBack,
    kForward,
    kEraser,
}

impl Button {
    // Source: third_party/blink/public/mojom/input/input_event.mojom:13
    pub const kBarrel: Self = Self::kRight;
}

// Source: third_party/blink/public/mojom/input/input_event.mojom:19-31
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum DispatchType {
    kBlocking = 0,
    kEventNonBlocking,
    kListenersNonBlockingPassive,
    kListenersForcedNonBlockingDueToFling,
}

// Source: third_party/blink/public/mojom/input/input_event.mojom:33-150
// C++ enum aliases are associated constants because Rust forbids duplicate
// discriminants. Their numeric identity and the source order are preserved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum EventType {
    kUndefined = -1,
    kMouseDown = 0,
    kMouseUp,
    kMouseMove,
    kMouseEnter,
    kMouseLeave,
    kContextMenu,
    kMouseWheel,
    kRawKeyDown,
    kKeyDown,
    kKeyUp,
    kChar,
    kGestureScrollBegin,
    kGestureScrollEnd,
    kGestureScrollUpdate,
    kGestureFlingStart,
    kGestureFlingCancel,
    kGesturePinchBegin,
    kGesturePinchEnd,
    kGesturePinchUpdate,
    kGestureBegin,
    kGestureTapDown,
    kGestureShowPress,
    kGestureTap,
    kGestureTapCancel,
    kGestureShortPress,
    kGestureLongPress,
    kGestureLongTap,
    kGestureTwoFingerTap,
    kGestureTapUnconfirmed,
    kGestureDoubleTap,
    kGestureEnd,
    kTouchStart,
    kTouchMove,
    kTouchEnd,
    kTouchCancel,
    kTouchScrollStarted,
    kPointerDown,
    kPointerUp,
    kPointerMove,
    kPointerRawUpdate,
    kPointerCancel,
    kPointerCausedUaAction,
}

impl EventType {
    // Source: third_party/blink/public/mojom/input/input_event.mojom:37,41,47,54,64,70,79,82,126,130,137,141,147,149
    pub const kTypeFirst: Self = Self::kUndefined;
    pub const kMouseTypeFirst: Self = Self::kMouseDown;
    pub const kMouseTypeLast: Self = Self::kContextMenu;
    pub const kKeyboardTypeFirst: Self = Self::kRawKeyDown;
    pub const kKeyboardTypeLast: Self = Self::kChar;
    pub const kGestureTypeFirst: Self = Self::kGestureScrollBegin;
    pub const kGesturePinchTypeFirst: Self = Self::kGesturePinchBegin;
    pub const kGesturePinchTypeLast: Self = Self::kGesturePinchUpdate;
    pub const kGestureTypeLast: Self = Self::kGestureEnd;
    pub const kTouchTypeFirst: Self = Self::kTouchStart;
    pub const kTouchTypeLast: Self = Self::kTouchScrollStarted;
    pub const kPointerTypeFirst: Self = Self::kPointerDown;
    pub const kPointerTypeLast: Self = Self::kPointerCausedUaAction;
    pub const kTypeLast: Self = Self::kPointerTypeLast;
}

// Source: third_party/blink/public/mojom/input/input_event.mojom:153-160
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum RailsMode {
    kRailsModeFree = 0,
    kRailsModeHorizontal = 1,
    kRailsModeVertical = 2,
}
