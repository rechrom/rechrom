// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
#![allow(non_camel_case_types, non_upper_case_globals)]
// cpp: third_party/blink/public/mojom/manifest/display_mode.mojom:12-44
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    kUndefined,
    kBrowser,
    kMinimalUi,
    kStandalone,
    kFullscreen,
    kWindowControlsOverlay,
    kTabbed,
    kUnframed,
    kPictureInPicture,
}
