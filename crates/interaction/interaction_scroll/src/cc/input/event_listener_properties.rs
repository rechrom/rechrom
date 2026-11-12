// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: cc/input/event_listener_properties.h:12-34

// Source: cc/input/event_listener_properties.h:12-23
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EventListenerClass {
    kPointerRawUpdate = 0,
    kTouchStartOrMove,
    kMouseWheel,
    kTouchEndOrCancel,
}

impl EventListenerClass {
    pub const kLast: Self = Self::kTouchEndOrCancel;
}

// Source: cc/input/event_listener_properties.h:25-26
pub const kEventListenerClassCount: usize = EventListenerClass::kLast as usize + 1;

// Source: cc/input/event_listener_properties.h:28-34
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EventListenerProperties {
    kNone,
    kPassive,
    kBlocking,
    kBlockingAndPassive,
}

impl EventListenerProperties {
    pub const kLast: Self = Self::kBlockingAndPassive;
}
