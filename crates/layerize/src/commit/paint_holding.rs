//! Commit-deferral values shared by ProxyMain and ProxyImpl.
//! Source: cc/trees/paint_holding_reason.h; cc/trees/proxy.h
//! (StartDeferringCommits, SetDeferMainFrameUpdate). Cut from
//! compositor/src/runtime/main_frame_lifecycle.rs; the deferral state
//! itself is main-half (layerize runtime/main_frame_lifecycle.rs).
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
