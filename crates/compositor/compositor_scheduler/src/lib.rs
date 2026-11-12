//! BeginFrame scheduling, commit/activation/draw action selection, and time.
//!
//! Clocks and task execution are injected; this package never creates an OS
//! thread or owns a Page.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod begin_frame;
pub mod begin_frame_policy;
pub mod begin_frame_tracker;
pub mod compositor_timing_history;
pub mod headless_scheduler_state_machine;
pub mod micro_benchmark;
pub mod rolling_time_delta_history;
pub mod scheduler;
pub mod scheduler_host;
pub mod scheduler_settings;
pub mod scheduler_state_machine;
pub mod services;
pub mod settings;
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

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum TraceValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Dictionary(BTreeMap<String, TraceValue>),
}

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

pub mod commit_earlyout_reason {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CommitEarlyOutReason {
        kAbortedNotVisible,
        kAbortedDeferredMainFrameUpdate,
        kAbortedDeferredCommit,
        kFinishedNoUpdates,
        kNoEarlyOut,
    }

    impl CommitEarlyOutReason {
        pub const kMaxValue: Self = Self::kNoEarlyOut;
    }

    pub fn CommitEarlyOutReasonToString(reason: CommitEarlyOutReason) -> &'static str {
        match reason {
            CommitEarlyOutReason::kAbortedNotVisible => "CommitEarlyOutReason::kAbortedNotVisible",
            CommitEarlyOutReason::kAbortedDeferredMainFrameUpdate => {
                "CommitEarlyOutReason::kAbortedDeferredMainFrameUpdate"
            }
            CommitEarlyOutReason::kAbortedDeferredCommit => {
                "CommitEarlyOutReason::kAbortedDeferredCommit"
            }
            CommitEarlyOutReason::kFinishedNoUpdates => "CommitEarlyOutReason::kFinishedNoUpdates",
            CommitEarlyOutReason::kNoEarlyOut => "CommitEarlyOutReason::kNoEarlyOut",
        }
    }

    pub fn MainFrameAppliedDeltas(reason: CommitEarlyOutReason) -> bool {
        reason == CommitEarlyOutReason::kFinishedNoUpdates
    }
}
