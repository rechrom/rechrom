// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/browser_controls_state.h:1-9,22-25.
// Rust module scope replaces header guards and namespace cc.

// Source: cc/input/browser_controls_state.h:10-19,21.
// A Java counterpart will be generated for this enum.
// GENERATED_JAVA_ENUM_PACKAGE: org.chromium.cc.input
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BrowserControlsState {
    // All browser controls should remain fully visible.
    kShown = 1,
    // All browser controls should remain at the minimum allowed shown ratio.
    kHidden = 2,
    // Both states are permitted. In this state, the browser controls will hide
    // when the user scrolls down, and show when the user scrolls up.
    kBoth = 3,
}

// Source: cc/input/browser_controls_state.h:20.
// Duplicate enum values are associated constants in Rust.
impl BrowserControlsState {
    pub const kMaxValue: Self = Self::kBoth;
}
