//! BeginFrame scheduling, commit/activation/draw action selection, and time.
//!
//! Clocks and task execution are injected; this package never creates an OS
//! thread or owns a Page.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod begin_frame_policy;
pub mod begin_frame_tracker;
pub mod compositor_timing_history;
pub mod headless_scheduler_state_machine;
pub mod micro_benchmark;
pub mod rolling_time_delta_history;
pub mod scheduler;
pub mod scheduler_host;
pub mod scheduler_state_machine;
pub mod services;
pub mod slim_scheduler_state_machine;
pub mod throttle_decider;
pub mod trace_sink;
pub mod video_frame_controller;
pub mod webview_scheduler_state_machine;

pub mod draw_result {
    pub use compositor_draw::*;
}

// Keeps the mechanically translated trace-value spellings local while the
// compositor facade preserves the public Chromium source path.
pub(crate) mod adapter {
    pub use crate::TraceValue;
}

// Shared cc values named by both proxies and the main side, re-exported
// under their old paths.
pub use compositor_values::{
    begin_frame, commit_earlyout_reason, layer_tree_settings_features as settings,
    scheduler_settings, traced_value::TraceValue,
};

pub fn GetThrottlingFactor(
    consecutive_no_damage_main_frames: i32,
    features: &services::SchedulerFeatures,
) -> i32 {
    if !features.kThrottleRepeatedNoDamageFrames {
        return 0;
    }
    let threshold1 = features.kThrottleRepeatedNoDamageFramesThreshold1;
    let threshold2 = features.kThrottleRepeatedNoDamageFramesThreshold2;
    let factor1 = features.kThrottleRepeatedNoDamageFramesIntervalFactor1;
    let factor2 = features.kThrottleRepeatedNoDamageFramesIntervalFactor2;
    if consecutive_no_damage_main_frames >= threshold1 + threshold2 {
        return factor1 * factor2;
    }
    if consecutive_no_damage_main_frames >= threshold1 {
        return factor1;
    }
    0
}

pub use scroll_animation::{TimeDelta, TimeTicks};

