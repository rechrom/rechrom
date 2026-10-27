// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
#![allow(non_camel_case_types, non_upper_case_globals)]
// cpp: ui/base/mojom/window_show_state.mojom:16-26
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowShowState {
    kDefault = 0,
    kNormal = 1,
    kMinimized = 2,
    kMaximized = 3,
    kInactive = 4,
    kFullscreen = 5,
    kHidden = 6,
    kEnd = 7,
}
