//! ProxyMain.cc:282-307,328-337,382-426,793-817,848-887 and
//! SingleThreadProxy.cc:436-473. State belongs to cc's main-frame proxy;
//! Page never filters submitted CompositorFrames to implement paint holding.
use crate::adapter::time::{TimeDelta, TimeTicks};
use std::time::Duration;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintHoldingReason {
    FirstContentfulPaint,
}
#[derive(Clone, Copy, Debug)]
pub enum FrameLifecycleMutation {
    SetDeferMainFrameUpdates(bool),
    StartDeferringCommits {
        timeout: Duration,
        reason: PaintHoldingReason,
    },
    StopDeferringCommits,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitDeferralReason {
    MainFrameUpdates,
    PaintHolding(PaintHoldingReason),
}
#[derive(Default)]
pub(crate) struct MainFrameLifecycle {
    defer_main_frame_updates: bool,
    defer_from_impl: bool,
    holding: Option<(PaintHoldingReason, TimeTicks)>,
}
impl MainFrameLifecycle {
    pub fn DeferBeginMainFrame(&self) -> bool {
        self.defer_main_frame_updates || self.defer_from_impl
    }
    pub fn SetDeferFromImpl(&mut self, value: bool) {
        self.defer_from_impl = value;
    }
    pub fn DeferMainFrameUpdates(&self) -> bool {
        self.defer_main_frame_updates
    }
    pub fn SetDeferMainFrameUpdates(&mut self, value: bool) -> bool {
        if self.defer_main_frame_updates == value {
            return false;
        }
        self.defer_main_frame_updates = value;
        true
    }
    pub fn IsDeferringCommits(&self) -> bool {
        self.holding.is_some()
    }
    pub fn StartDeferringCommits(
        &mut self,
        now: TimeTicks,
        timeout: Duration,
        reason: PaintHoldingReason,
    ) -> bool {
        if self.holding.is_some() {
            return false;
        }
        self.holding = Some((
            reason,
            now + TimeDelta::Microseconds(
                timeout
                    .as_micros()
                    .try_into()
                    .expect("source commit timeout"),
            ),
        ));
        true
    }
    pub fn StopDeferringCommits(&mut self) -> Option<PaintHoldingReason> {
        self.holding.take().map(|(reason, _)| reason)
    }
    pub fn Expire(&mut self, now: TimeTicks) -> Option<PaintHoldingReason> {
        if self.holding.is_some_and(|(_, deadline)| now > deadline) {
            self.StopDeferringCommits()
        } else {
            None
        }
    }
    pub fn DeferralReason(&self) -> Option<CommitDeferralReason> {
        if self.defer_main_frame_updates {
            Some(CommitDeferralReason::MainFrameUpdates)
        } else {
            self.holding
                .map(|(reason, _)| CommitDeferralReason::PaintHolding(reason))
        }
    }
}
