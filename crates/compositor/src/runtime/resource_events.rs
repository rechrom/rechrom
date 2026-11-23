//! Same-sequence Proxy transport for presentation callbacks. This queue contains
//! only the original once callbacks and native values, never a Host/tree borrow.
//! Source: cc/trees/proxy_impl.cc:711-737,759-766.
use super::{
    host_metrics::CustomTrackerResults,
    metadata::{FrameTimingDetails, PresentationFeedback},
};
use crate::{
    adapter::layer_tree_host_impl::{HostImplDelegateId, HostViewTransitionElementResourceRects},
    cc::trees::presentation_time_callback_buffer::PendingCallbacks,
};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
pub(crate) enum ResourceEffect {
    ReportTimeSwapped {
        request: u64,
        frame_token: u32,
        swap_time: crate::adapter::time::TimeTicks,
    },
    ReportTimeDidNotSwap {
        request: u64,
        failure_time: crate::adapter::time::TimeTicks,
    },
    DidLoseLayerTreeFrameSink {
        delegate: HostImplDelegateId,
    },
    CanDrawStateChanged {
        delegate: HostImplDelegateId,
        can_draw: bool,
    },
    PageScaleAnimationCompleted {
        delegate: HostImplDelegateId,
    },
    VideoBeginFrame {
        controller: crate::adapter::layer_tree_host_impl::HostImplVideoFrameControllerId,
        args: crate::adapter::begin_frame::BeginFrameArgs,
    },
    VideoNeedsBeginFrames {
        delegate: HostImplDelegateId,
        needs: bool,
    },
    Presented {
        delegate: HostImplDelegateId,
        token: u32,
        callbacks: PendingCallbacks<PresentationFeedback, FrameTimingDetails>,
        details: FrameTimingDetails,
    },
    TransitionFinished {
        delegate: HostImplDelegateId,
        sequence: u32,
        rects: HostViewTransitionElementResourceRects,
    },
    CustomMetrics {
        delegate: HostImplDelegateId,
        results: CustomTrackerResults,
    },
}
#[derive(Clone, Default)]
// Arc<Mutex>: swap promises that push into this queue are queued on the main
// LayerTreeHost and move to the impl thread with the commit (ReportTimeSwapPromise).
pub(crate) struct ResourceEffects(Arc<Mutex<VecDeque<ResourceEffect>>>);
impl ResourceEffects {
    pub(crate) fn Push(&self, effect: ResourceEffect) {
        self.0.lock().unwrap().push_back(effect);
    }
    pub(crate) fn Pop(&self) -> Option<ResourceEffect> {
        self.0.lock().unwrap().pop_front()
    }
    pub(crate) fn IsEmpty(&self) -> bool {
        self.0.lock().unwrap().is_empty()
    }
}
