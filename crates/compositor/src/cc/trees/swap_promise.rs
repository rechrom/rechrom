// Translation status: complete
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use crate::adapter::time::TimeTicks;
// Source: cc/trees/swap_promise.h:1-84.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DidNotSwapReason {
    SWAP_FAILS,
    COMMIT_FAILS,
    COMMIT_NO_UPDATE,
    ACTIVATION_FAILS,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DidNotSwapAction {
    BREAK_PROMISE,
    KEEP_ACTIVE,
}
// M is the external Viz metadata type; no Viz storage/logic is owned by cc.
pub trait SwapPromise<M: ?Sized> {
    fn DidActivate(&mut self);
    fn WillSwap(&mut self, metadata: &mut M);
    fn DidSwap(&mut self);
    fn DidNotSwap(&mut self, reason: DidNotSwapReason, timestamp: TimeTicks) -> DidNotSwapAction;
    fn OnCommit(&mut self) {}
    fn GetTraceId(&self) -> i64;
}
