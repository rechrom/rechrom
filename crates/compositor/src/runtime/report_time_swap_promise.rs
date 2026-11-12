//! Chromium WebFrameWidgetImpl.cc:3922-4000. Swap promises travel through
//! CommitState and the native LayerTreeImpl list. Only an opaque callback id
//! crosses back to the widget's source owner; no Page or Window is retained.
use super::{
    host_owners::SwapPromise,
    metadata::CompositorFrameMetadata,
    resource_events::{ResourceEffect, ResourceEffects},
    skia::SkRegion,
};
use crate::{
    adapter::time::TimeTicks,
    cc::trees::swap_promise::{DidNotSwapAction, DidNotSwapReason, SwapPromise as Promise},
    CompositorTypes,
};
use std::sync::Arc;
struct ReportTimeSwapPromise {
    request: u64,
    frame_token: u32,
    effects: ResourceEffects,
    clock: Arc<dyn crate::CompositorExecutor>,
}
impl Promise<CompositorFrameMetadata<CompositorTypes>> for ReportTimeSwapPromise {
    fn DidActivate(&mut self) {}
    fn WillSwap(&mut self, metadata: &mut CompositorFrameMetadata<CompositorTypes>) {
        assert!(metadata.frame_token > 0);
        self.frame_token = metadata.frame_token;
    }
    fn DidSwap(&mut self) {
        assert!(self.frame_token > 0);
        self.effects.Push(ResourceEffect::ReportTimeSwapped {
            request: self.request,
            frame_token: self.frame_token,
            swap_time: self.clock.Now(),
        });
    }
    fn DidNotSwap(&mut self, reason: DidNotSwapReason, timestamp: TimeTicks) -> DidNotSwapAction {
        if !matches!(
            reason,
            DidNotSwapReason::SWAP_FAILS | DidNotSwapReason::COMMIT_NO_UPDATE
        ) {
            return DidNotSwapAction::KEEP_ACTIVE;
        }
        self.effects.Push(ResourceEffect::ReportTimeDidNotSwap {
            request: self.request,
            failure_time: timestamp,
        });
        DidNotSwapAction::BREAK_PROMISE
    }
    fn GetTraceId(&self) -> i64 {
        0
    }
}
pub(super) fn Create(
    request: u64,
    effects: ResourceEffects,
    clock: Arc<dyn crate::CompositorExecutor>,
) -> Box<SwapPromise<SkRegion>> {
    Box::new(SwapPromise(Box::new(ReportTimeSwapPromise {
        request,
        frame_token: 0,
        effects,
        clock,
    })))
}
