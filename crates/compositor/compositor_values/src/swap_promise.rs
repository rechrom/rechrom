// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/swap_promise.h:17-77. The interface is shared: the main
// LayerTreeHost queues promises (LayerTreeHost::QueueSwapPromise ->
// CommitState::swap_promises) and the impl LayerTreeImpl keeps, swaps or
// breaks them. The impl list algorithms stay in compositor_frame.
use scroll_animation::TimeTicks;

// Source: cc/trees/swap_promise.h:22-29.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DidNotSwapReason {
    SWAP_FAILS,
    COMMIT_FAILS,
    COMMIT_NO_UPDATE,
    ACTIVATION_FAILS,
}

// Source: cc/trees/swap_promise.h:33-36.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DidNotSwapAction {
    BREAK_PROMISE,
    KEEP_ACTIVE,
}

// Source: cc/trees/swap_promise.h:38-77. `M` is viz::CompositorFrameMetadata.
// `Send`: swap promises queued on the main LayerTreeHost move to the impl
// thread with the commit (cc/trees/layer_tree_host.cc WillCommit).
pub trait SwapPromise<M: ?Sized>: Send {
    fn DidActivate(&mut self);
    fn WillSwap(&mut self, metadata: &mut M);
    fn DidSwap(&mut self);
    fn DidNotSwap(&mut self, reason: DidNotSwapReason, timestamp: TimeTicks) -> DidNotSwapAction;
    fn OnCommit(&mut self) {}
    fn GetTraceId(&self) -> i64;
}
