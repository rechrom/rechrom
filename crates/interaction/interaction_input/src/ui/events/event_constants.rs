// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15

// Source: ui/events/event_constants.h:211-219
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EventPointerType {
    kUnknown,
    kMouse,
    kPen,
    kTouch,
    kEraser,
}

impl EventPointerType {
    // Source: ui/events/event_constants.h:218
    pub const kMaxValue: Self = Self::kEraser;
}
