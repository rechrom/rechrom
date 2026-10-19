// Translation status: complete
// Copyright 2021 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/viewport_property_ids.h:1-23
use super::property_ids::kInvalidPropertyNodeId;

// Source: cc/trees/viewport_property_ids.h:13-19
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewportPropertyIds {
    pub overscroll_elasticity_transform: i32,
    pub page_scale_transform: i32,
    pub inner_scroll: i32,
    pub outer_clip: i32,
    pub outer_scroll: i32,
}

impl Default for ViewportPropertyIds {
    // Source: cc/trees/viewport_property_ids.h:14-18
    fn default() -> Self {
        Self {
            overscroll_elasticity_transform: kInvalidPropertyNodeId,
            page_scale_transform: kInvalidPropertyNodeId,
            inner_scroll: kInvalidPropertyNodeId,
            outer_clip: kInvalidPropertyNodeId,
            outer_scroll: kInvalidPropertyNodeId,
        }
    }
}
