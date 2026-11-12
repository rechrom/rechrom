// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15

// Source: ui/events/types/event_type.h:10-75
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum EventType {
    kUnknown = 0,
    kMousePressed,
    kMouseDragged,
    kMouseReleased,
    kMouseMoved,
    kMouseEntered,
    kMouseExited,
    kKeyPressed,
    kKeyReleased,
    kMousewheel,
    kMouseCaptureChanged,
    kTouchReleased,
    kTouchPressed,
    kTouchMoved,
    kTouchCancelled,
    kDropTargetEvent,
    kGestureScrollBegin,
    kGestureScrollEnd,
    kGestureScrollUpdate,
    kGestureTap,
    kGestureTapDown,
    kGestureTapCancel,
    kGestureTapUnconfirmed,
    kGestureDoubleTap,
    kGestureBegin,
    kGestureEnd,
    kGestureTwoFingerTap,
    kGesturePinchBegin,
    kGesturePinchEnd,
    kGesturePinchUpdate,
    kGestureShortPress,
    kGestureLongPress,
    kGestureLongTap,
    kGestureSwipe,
    kGestureShowPress,
    kScroll,
    kScrollFlingStart,
    kScrollFlingCancel,
    kCancelMode,
    kUmaData,
    kLast,
}

impl EventType {
    // Source: ui/events/types/event_type.h:34,62
    pub const kGestureTypeStart: Self = Self::kGestureScrollBegin;
    pub const kGestureTypeEnd: Self = Self::kScrollFlingCancel;
}
