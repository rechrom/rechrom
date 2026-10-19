// Translation status: complete
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::adapter::TraceValue;
use std::collections::BTreeMap;

// Source: cc/scheduler/scheduler_settings.h:20-73
// Source: cc/scheduler/scheduler_settings.cc:15-17
#[derive(Clone, Debug)]
pub struct SchedulerSettings {
    // Source: cc/scheduler/scheduler_settings.h:26-29
    // Whether a BeginMainFrame should be issued while there is a pending-tree
    // still waiting to be activated. This is disabled by default for the UI
    // compositor, and enabled for renderers (unless there are too few cores).
    pub main_frame_before_activation_enabled: bool,

    // Source: cc/scheduler/scheduler_settings.h:31-33
    // Whether a BeginMainFrame should be issued while there is a previous commit
    // still waiting to be processed.
    pub main_frame_before_commit_enabled: bool,

    // Source: cc/scheduler/scheduler_settings.h:35-38
    // Whether commits should happen directly to the active tree, skipping the
    // pending tree. This is turned on only for the UI compositor (and in some
    // tests).
    pub commit_to_active_tree: bool,

    // Source: cc/scheduler/scheduler_settings.h:40-41
    // This is enabled for android-webview.
    pub using_synchronous_renderer_compositor: bool,

    // Source: cc/scheduler/scheduler_settings.h:43-47
    // Turning this on effectively disables pipelining of compositor frame
    // production stages by waiting for each stage to complete before producing
    // the frame. This is enabled for headless-mode and some tests, and disabled
    // elsewhere by default.
    pub wait_for_all_pipeline_stages_before_draw: bool,

    // Source: cc/scheduler/scheduler_settings.h:49
    pub maximum_number_of_failed_draws_before_draw_is_forced: i32,

    // Source: cc/scheduler/scheduler_settings.h:51-54
    // Whether to disable the limit by which frames are drawn. If this is set,
    // draws are not throttled when pending frames exceed the allowed maximum, as
    // they would be under the default settings.
    pub disable_frame_rate_limit: bool,

    // Source: cc/scheduler/scheduler_settings.h:56-60
    // When true BeginImplFrameDeadlineMode::SCROLL will be enabled. This deadline
    // is used to wait for `scroll_deadline_ratio` of `BeginFramrArgs::interval`
    // for input to arrive, before attempting to draw.
    pub scroll_deadline_mode_enabled: bool,
    pub scroll_deadline_ratio: f64,

    // Source: cc/scheduler/scheduler_settings.h:62-64
    // The number of frames to allow slow main commits to delay impl invalidation
    // frames by.
    pub delay_impl_invalidation_frames: i32,

    // Source: cc/scheduler/scheduler_settings.h:66-70
    // When true SingleThreadedProxy is in use, which typically combines
    // `commit_to_active_tree` and disables `main_frame_before_commit_enabled`.
    // However those two settings are also changed by Blink WebTests. The
    // SlimScheduler currently does not support SingleThreadedProxy.
    pub single_threaded_proxy: bool,
}

// Source: cc/scheduler/scheduler_settings.cc:13
// Source: cc/scheduler/scheduler_settings.h:29-70
impl Default for SchedulerSettings {
    fn default() -> Self {
        Self {
            main_frame_before_activation_enabled: false,
            main_frame_before_commit_enabled: false,
            commit_to_active_tree: false,
            using_synchronous_renderer_compositor: false,
            wait_for_all_pipeline_stages_before_draw: false,
            maximum_number_of_failed_draws_before_draw_is_forced: 3,
            disable_frame_rate_limit: false,
            scroll_deadline_mode_enabled: false,
            scroll_deadline_ratio: 0.333,
            delay_impl_invalidation_frames: 0,
            single_threaded_proxy: false,
        }
    }
}

impl SchedulerSettings {
    // Source: cc/scheduler/scheduler_settings.h:22
    // Source: cc/scheduler/scheduler_settings.cc:13
    pub fn new() -> Self {
        Self::default()
    }

    // Source: cc/scheduler/scheduler_settings.h:72
    // Source: cc/scheduler/scheduler_settings.cc:19-34
    pub fn AsValue(&self) -> TraceValue {
        let mut state = BTreeMap::new();
        state.insert(
            "main_frame_before_activation_enabled".to_owned(),
            TraceValue::Bool(self.main_frame_before_activation_enabled),
        );
        state.insert(
            "commit_to_active_tree".to_owned(),
            TraceValue::Bool(self.commit_to_active_tree),
        );
        state.insert(
            "maximum_number_of_failed_draws_before_draw_is_forced".to_owned(),
            TraceValue::Integer(i64::from(
                self.maximum_number_of_failed_draws_before_draw_is_forced,
            )),
        );
        state.insert(
            "using_synchronous_renderer_compositor".to_owned(),
            TraceValue::Bool(self.using_synchronous_renderer_compositor),
        );
        state.insert(
            "wait_for_all_pipeline_stages_before_draw".to_owned(),
            TraceValue::Bool(self.wait_for_all_pipeline_stages_before_draw),
        );
        state.insert(
            "disable_frame_rate_limit".to_owned(),
            TraceValue::Bool(self.disable_frame_rate_limit),
        );
        TraceValue::Dictionary(state)
    }
}
