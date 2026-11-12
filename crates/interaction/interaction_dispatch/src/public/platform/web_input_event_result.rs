// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/platform/web_input_event_result.h:1-24

// Source: third_party/blink/public/platform/web_input_event_result.h:10-20
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum WebInputEventResult {
    kNotHandled = 0,
    kHandledSuppressed,
    kHandledApplication,
    kHandledSystem,
}
