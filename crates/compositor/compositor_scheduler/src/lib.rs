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
// public compatibility path is provided by the compositor facade.
pub(crate) mod adapter {
    pub use crate::TraceValue;
}

use std::{
    collections::BTreeMap,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeDelta(i64);

impl TimeDelta {
    pub const fn Microseconds(value: i64) -> Self {
        Self(value)
    }
    pub const fn Milliseconds(value: i64) -> Self {
        Self(value.saturating_mul(1000))
    }
    pub const fn Seconds(value: i64) -> Self {
        Self(value.saturating_mul(1_000_000))
    }
    pub const fn InMicroseconds(self) -> i64 {
        self.0
    }
    pub fn InMicrosecondsF(self) -> f64 {
        if self == Self::Max() {
            f64::INFINITY
        } else if self == Self::Min() {
            f64::NEG_INFINITY
        } else {
            self.0 as f64
        }
    }
    pub fn InSecondsF(self) -> f64 {
        self.InMicrosecondsF() / 1_000_000.0
    }
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
    pub const fn is_positive(self) -> bool {
        self.0 > 0
    }
    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }
    pub const fn Max() -> Self {
        Self(i64::MAX)
    }
    pub const fn Min() -> Self {
        Self(i64::MIN)
    }
}

impl Add for TimeDelta {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}
impl Sub for TimeDelta {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}
impl AddAssign for TimeDelta {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl SubAssign for TimeDelta {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl Neg for TimeDelta {
    type Output = Self;
    fn neg(self) -> Self {
        Self(self.0.saturating_neg())
    }
}
impl Mul<i32> for TimeDelta {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self {
        Self(self.0.saturating_mul(i64::from(rhs)))
    }
}
impl Mul<f64> for TimeDelta {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self((self.0 as f64 * rhs) as i64)
    }
}
impl Mul<f32> for TimeDelta {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        self * f64::from(rhs)
    }
}
impl Mul<TimeDelta> for f64 {
    type Output = TimeDelta;
    fn mul(self, rhs: TimeDelta) -> TimeDelta {
        rhs * self
    }
}
impl Mul<TimeDelta> for f32 {
    type Output = TimeDelta;
    fn mul(self, rhs: TimeDelta) -> TimeDelta {
        rhs * self
    }
}
impl Div<i32> for TimeDelta {
    type Output = Self;
    fn div(self, rhs: i32) -> Self {
        Self(self.0 / i64::from(rhs))
    }
}
impl Div<f64> for TimeDelta {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self::Microseconds((self.0 as f64 / rhs) as i64)
    }
}
impl Div<TimeDelta> for TimeDelta {
    type Output = f64;
    fn div(self, rhs: TimeDelta) -> f64 {
        let left_inf = self == Self::Max() || self == Self::Min();
        let right_inf = rhs == Self::Max() || rhs == Self::Min();
        assert!(!self.is_zero() || !rhs.is_zero());
        assert!(!left_inf || !right_inf);
        self.InMicrosecondsF() / rhs.InMicrosecondsF()
    }
}
impl Mul<u64> for TimeDelta {
    type Output = Self;
    fn mul(self, rhs: u64) -> Self {
        Self(
            (i128::from(self.0) * i128::from(rhs)).clamp(i128::from(i64::MIN), i128::from(i64::MAX))
                as i64,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeTicks(i64);

impl TimeTicks {
    pub const fn FromMicroseconds(value: i64) -> Self {
        Self(value)
    }
    pub const fn since_origin(self) -> TimeDelta {
        TimeDelta::Microseconds(self.0)
    }
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }
    pub const fn Min() -> Self {
        Self(i64::MIN)
    }
    pub const fn Max() -> Self {
        Self(i64::MAX)
    }
}
impl Add<TimeDelta> for TimeTicks {
    type Output = Self;
    fn add(self, rhs: TimeDelta) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}
impl Sub<TimeDelta> for TimeTicks {
    type Output = Self;
    fn sub(self, rhs: TimeDelta) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}
impl Sub for TimeTicks {
    type Output = TimeDelta;
    fn sub(self, rhs: Self) -> TimeDelta {
        TimeDelta::Microseconds(self.0.saturating_sub(rhs.0))
    }
}
impl AddAssign<TimeDelta> for TimeTicks {
    fn add_assign(&mut self, rhs: TimeDelta) {
        *self = *self + rhs;
    }
}
impl SubAssign<TimeDelta> for TimeTicks {
    fn sub_assign(&mut self, rhs: TimeDelta) {
        *self = *self - rhs;
    }
}

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
