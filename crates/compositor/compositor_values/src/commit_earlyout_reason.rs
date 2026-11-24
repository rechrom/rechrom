//! Source: cc/scheduler/commit_earlyout_reason.h. Named by both proxies
//! (ProxyMain::BeginMainFrame aborts, ProxyImpl::BeginMainFrameAbortedOnImpl).
//! Moved from compositor_scheduler, which re-exports it.

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
