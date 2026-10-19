// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/event_listener_properties.h:1-11,35-38.
// Rust module scope replaces header guards and namespace cc; usize is size_t.

// Source: cc/input/event_listener_properties.h:12-20,23.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EventListenerClass {
    // This includes the pointerrawupdate events which are non-rAF-aligned.
    kPointerRawUpdate = 0,
    // This value includes "touchstart", "touchmove", and "pointer" events.
    kTouchStartOrMove,
    // This value includes "wheel" and "mousewheel" events.
    kMouseWheel,
    // This value includes "touchend" and "touchcancel" events.
    kTouchEndOrCancel,
}

// Source: cc/input/event_listener_properties.h:21-22.
// Rust rejects duplicate enum discriminants; the alias is an associated constant.
impl EventListenerClass {
    pub const kLast: Self = Self::kTouchEndOrCancel;
}

// Source: cc/input/event_listener_properties.h:24-26.
pub const kEventListenerClassCount: usize = EventListenerClass::kLast as usize + 1;

// Source: cc/input/event_listener_properties.h:27-32,34.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EventListenerProperties {
    kNone,
    kPassive,
    kBlocking,
    kBlockingAndPassive,
}

// Source: cc/input/event_listener_properties.h:33.
impl EventListenerProperties {
    pub const kLast: Self = Self::kBlockingAndPassive;
}
