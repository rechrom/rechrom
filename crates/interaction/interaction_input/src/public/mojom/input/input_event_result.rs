// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15

// Source: third_party/blink/public/mojom/input/input_event_result.mojom:15-31
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum InputEventResultState {
    kUnknown,
    kConsumed,
    kNotConsumed,
    kNoConsumerExists,
    kIgnored,
    kSetNonBlocking,
    kSetNonBlockingDueToFling,
}

// Source: third_party/blink/public/mojom/input/input_event_result.mojom:7-13.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum InputEventResultSource {
    kUnknown,
    kCompositorThread,
    kMainThread,
    kBrowser,
}
