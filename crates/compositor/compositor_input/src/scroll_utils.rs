// Translation status: complete
// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/input/scroll_utils.h:1-19,56-59;
//         cc/input/scroll_utils.cc:1-14,74-75.
// Rust modules/imports replace guards, namespace and forward declarations.
use compositor_tree::input_geometry::{SizeF, Vector2dF};

// Source: cc/input/scroll_utils.h:20.
pub const kPixelsPerLineStep: i32 = 40;
// Source: cc/input/scroll_utils.h:21.
pub const kMinFractionToStepWhenSnapPaging: f32 = 0.4;
// Source: cc/input/scroll_utils.h:22.
pub const kMinFractionToStepWhenPaging: f32 = 0.875;
// Source: cc/input/scroll_utils.h:23-27.
#[cfg(target_os = "macos")]
pub const kMaxOverlapBetweenPages: i32 = 40;
#[cfg(not(target_os = "macos"))]
pub const kMaxOverlapBetweenPages: i32 = i32::MAX;

// Source: cc/input/scroll_utils.h:28-31,55.
// Class for scroll helper methods in cc and blink.
pub struct ScrollUtils;

// Rust language adapter for std::min/max, preserving C++ operand order even
// when a float is NaN. These are dependency operations, not a second algorithm.
fn cpp_min(a: f32, b: f32) -> f32 {
    if b < a {
        b
    } else {
        a
    }
}
fn cpp_max(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        a
    }
}

impl ScrollUtils {
    // Source: cc/input/scroll_utils.h:32-41;
    //         cc/input/scroll_utils.cc:15-38.
    // Transforms a |scroll_delta| in percent units to pixel units based on its
    // scroller_size and viewport_size. Inputs and output must be in physical
    // pixels. Used for kScrollByPage wheel scrolls handled on the compositor.
    pub fn ResolveScrollPercentageToPixels(
        delta: &Vector2dF,
        scroller: &SizeF,
        viewport: &SizeF,
    ) -> Vector2dF {
        // Work with unsigned values and keep sign information in sign_x / sign_y.
        let sign_x = if delta.x().is_sign_negative() {
            -1.0
        } else {
            1.0
        };
        let sign_y = if delta.y().is_sign_negative() {
            -1.0
        } else {
            1.0
        };
        let mut delta_x = delta.x().abs();
        let mut delta_y = delta.y().abs();
        // Resolve and clamp horizontal scroll.
        if delta_x > 0.0 {
            delta_x = cpp_max(1.0, delta_x * cpp_min(scroller.width(), viewport.width()));
        }
        // Resolve and clamp vertical scroll.
        if delta_y > 0.0 {
            delta_y = cpp_max(1.0, delta_y * cpp_min(scroller.height(), viewport.height()));
        }
        Vector2dF::new(delta_x.copysign(sign_x), delta_y.copysign(sign_y))
    }

    // Source: cc/input/scroll_utils.h:42; cc/input/scroll_utils.cc:39-44.
    pub fn CalculateMinPageSnap(length: i32) -> i32 {
        let min_page_step = (length as f32 * kMinFractionToStepWhenSnapPaging) as i32;
        min_page_step.max(1)
    }

    // Source: cc/input/scroll_utils.h:43; cc/input/scroll_utils.cc:45-49.
    pub fn CalculateMaxPageSnap(length: i32) -> i32 {
        length.max(1)
    }

    // Source: cc/input/scroll_utils.h:44-45; cc/input/scroll_utils.cc:50-57.
    pub fn CalculatePageStep(length: i32) -> i32 {
        let min_page_step = (length as f32 * kMinFractionToStepWhenPaging) as i32;
        let page_step = min_page_step.max(length - kMaxOverlapBetweenPages);
        page_step.max(1)
    }

    // Source: cc/input/scroll_utils.h:46-54; cc/input/scroll_utils.cc:58-73.
    // Calculates scrollbar thumb length from total content size, visible content
    // size, track length and the lower bound minimum_thumb_length.
    pub fn CalculateScrollbarThumbLength(
        total_size: i32,
        visible_size: i32,
        track_length: i32,
        minimum_thumb_length: i32,
    ) -> i32 {
        let mut proportion: f32 = 0.0;
        if total_size > 0 {
            proportion = visible_size as f32 / total_size as f32;
        }
        let mut length = (proportion * track_length as f32).round() as i32;
        length = length.max(minimum_thumb_length);
        if length > track_length {
            length = track_length;
        }
        length
    }
}
