// Translation status: complete
// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/browser_controls_params.h:1-45
// Source: cc/trees/browser_controls_params.cc:1-28

// Source: cc/trees/browser_controls_params.h:12-41
#[derive(Clone, Copy, Debug, Default)]
pub struct BrowserControlsParams {
    // Source: cc/trees/browser_controls_params.h:13-15
    pub top_controls_height: f32,
    // Source: cc/trees/browser_controls_params.h:17-18
    pub top_controls_min_height: f32,
    // Source: cc/trees/browser_controls_params.h:20-21
    pub bottom_controls_height: f32,
    // Source: cc/trees/browser_controls_params.h:23-24
    pub bottom_controls_min_height: f32,
    // Source: cc/trees/browser_controls_params.h:26-28
    pub animate_browser_controls_height_changes: bool,
    // Source: cc/trees/browser_controls_params.h:30-32
    pub browser_controls_shrink_blink_size: bool,
    // Source: cc/trees/browser_controls_params.h:34-37
    pub only_expand_top_controls_at_page_top: bool,
}

impl PartialEq for BrowserControlsParams {
    // Source: cc/trees/browser_controls_params.cc:9-21
    fn eq(&self, other: &Self) -> bool {
        self.top_controls_height == other.top_controls_height
            && self.top_controls_min_height == other.top_controls_min_height
            && self.bottom_controls_height == other.bottom_controls_height
            && self.bottom_controls_min_height == other.bottom_controls_min_height
            && self.animate_browser_controls_height_changes
                == other.animate_browser_controls_height_changes
            && self.browser_controls_shrink_blink_size == other.browser_controls_shrink_blink_size
            && self.only_expand_top_controls_at_page_top
                == other.only_expand_top_controls_at_page_top
    }
    // Source: cc/trees/browser_controls_params.cc:23-26
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
