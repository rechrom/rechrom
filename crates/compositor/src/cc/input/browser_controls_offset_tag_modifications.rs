// Translation status: complete
// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/browser_controls_offset_tag_modifications.h:1-12,22-25.
// Rust imports/module scope replace includes, namespace and header/export macros.
use crate::cc::input::browser_controls_offset_tags::BrowserControlsOffsetTags;

// Source: cc/input/browser_controls_offset_tag_modifications.h:13-21.
// Sent from browser to renderer.
// Contains the OffsetTags for browser controls and the additional heights that
// the controls must be offset by, so visual effects that overlap the web
// contents will disappear when controls are moved completely off screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrowserControlsOffsetTagModifications {
    pub tags: BrowserControlsOffsetTags,
    pub top_controls_additional_height: i32,
    pub bottom_controls_additional_height: i32,
}
