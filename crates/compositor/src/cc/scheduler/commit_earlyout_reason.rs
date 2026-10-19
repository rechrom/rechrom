// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Source: cc/scheduler/commit_earlyout_reason.h:13-23
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitEarlyOutReason {
    kAbortedNotVisible,
    kAbortedDeferredMainFrameUpdate,
    kAbortedDeferredCommit,
    kFinishedNoUpdates,
    // Not an early out case, this is only for metrics.
    kNoEarlyOut,
}

// Source: cc/scheduler/commit_earlyout_reason.h:21
impl CommitEarlyOutReason {
    pub const kMaxValue: Self = Self::kNoEarlyOut;
}

// Source: cc/scheduler/commit_earlyout_reason.h:25-40
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

// Source: cc/scheduler/commit_earlyout_reason.h:41-43
pub fn MainFrameAppliedDeltas(reason: CommitEarlyOutReason) -> bool {
    reason == CommitEarlyOutReason::kFinishedNoUpdates
}
