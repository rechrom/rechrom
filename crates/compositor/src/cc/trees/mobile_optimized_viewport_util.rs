// Translation status: complete
// Copyright 2021 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/mobile_optimized_viewport_util.h:1-34
// Source: cc/trees/mobile_optimized_viewport_util.cc:1-39

use crate::adapter::input_geometry::SizeF;

// Source: cc/trees/mobile_optimized_viewport_util.cc:13-18.
const kMobileViewportWidthEpsilon: f32 = 0.15;

// Source: cc/trees/mobile_optimized_viewport_util.h:13-30;
//         cc/trees/mobile_optimized_viewport_util.cc:20-36.
pub fn IsMobileOptimized(
    min_page_scale_factor: f32,
    max_page_scale_factor: f32,
    current_page_scale_factor: f32,
    scrollable_viewport_size: SizeF,
    scrollable_size: SizeF,
    viewport_meta_mobile_optimized: bool,
) -> bool {
    let has_fixed_page_scale = min_page_scale_factor == max_page_scale_factor;

    let window_width_dip = current_page_scale_factor * scrollable_viewport_size.width();
    let content_width_css = scrollable_size.width();
    let has_mobile_viewport =
        content_width_css <= window_width_dip + kMobileViewportWidthEpsilon;

    has_mobile_viewport || has_fixed_page_scale || viewport_meta_mobile_optimized
}
