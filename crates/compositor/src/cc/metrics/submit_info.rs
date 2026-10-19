// Translation status: complete
// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/metrics/submit_info.h:1-14,53-56;
//         cc/metrics/submit_info.cc:1-15,47.
// Includes, export guards and namespace are represented by this Rust module.
use crate::adapter::time::TimeTicks;

// Source: cc/metrics/submit_info.h:15-51.
// E is the external EventMetricsSet owned value, supplied by the actual metrics
// owner. SubmitInfo itself remains move-only; no Clone/Copy implementation.
pub struct SubmitInfo<E: Default> {
    pub frame_token: u32,
    pub time: TimeTicks,
    pub checkerboarded_needs_raster: bool,
    pub checkerboarded_needs_record: bool,
    pub top_controls_moved: bool,
    pub events_metrics: E,
    pub drawn_with_new_layer_tree: bool,
    pub invalidate_raster_scroll: bool,
    pub normalized_invalidated_area: Option<f32>,
    pub trees_in_viz_submit_time: TimeTicks,
}
impl<E: Default> SubmitInfo<E> {
    // Source: cc/metrics/submit_info.h:17-26;
    //         cc/metrics/submit_info.cc:16-35.
    pub fn new(
        frame_token: u32,
        time: TimeTicks,
        checkerboarded_needs_raster: bool,
        checkerboarded_needs_record: bool,
        top_controls_moved: bool,
        events_metrics: E,
        drawn_with_new_layer_tree: bool,
        invalidate_raster_scroll: bool,
        normalized_invalidated_area: Option<f32>,
        trees_in_viz_submit_time: TimeTicks,
    ) -> Self {
        Self {
            frame_token,
            time,
            checkerboarded_needs_raster,
            checkerboarded_needs_record,
            top_controls_moved,
            events_metrics,
            drawn_with_new_layer_tree,
            invalidate_raster_scroll,
            normalized_invalidated_area,
            trees_in_viz_submit_time,
        }
    }
    // Source: cc/metrics/submit_info.h:28;
    //         cc/metrics/submit_info.cc:37-38.
    pub fn FromFrameToken(frame_token: u32, time: TimeTicks) -> Self {
        Self {
            frame_token,
            time,
            ..Self::default()
        }
    }
}
// Source: cc/metrics/submit_info.h:30,38-51;
//         cc/metrics/submit_info.cc:40.
impl<E: Default> Default for SubmitInfo<E> {
    fn default() -> Self {
        Self {
            frame_token: 0,
            time: TimeTicks::default(),
            checkerboarded_needs_raster: false,
            checkerboarded_needs_record: false,
            top_controls_moved: false,
            events_metrics: E::default(),
            drawn_with_new_layer_tree: true,
            invalidate_raster_scroll: false,
            normalized_invalidated_area: None,
            trees_in_viz_submit_time: TimeTicks::default(),
        }
    }
}
// Source: cc/metrics/submit_info.h:32-36;
//         cc/metrics/submit_info.cc:42-45.
// Rust's move assignment and member destruction preserve the defaulted native
// move-only operations, including destruction of the external EventMetricsSet.
