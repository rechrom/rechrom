// Translation status: complete
// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/browser_controls_offset_tags.h:1-12,21-24.
// Rust imports/module scope replace includes, namespace and header/export macros.
use crate::adapter::input_services::OffsetTag;

// Source: cc/input/browser_controls_offset_tags.h:13-20.
// Contains all OffsetTags for browser controls. Tagging browser controls with
// OffsetTags allow the renderer and viz to move them during scrolls/animations
// without needing browser compositor frames.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrowserControlsOffsetTags {
    pub top_controls_offset_tag: OffsetTag,
    pub content_offset_tag: OffsetTag,
    pub bottom_controls_offset_tag: OffsetTag,
}
