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
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
pub(crate) enum NativeResourceEffect {
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
pub(crate) struct NativeResourceEffects(Rc<RefCell<VecDeque<NativeResourceEffect>>>);
impl NativeResourceEffects {
    pub(crate) fn Push(&self, effect: NativeResourceEffect) {
        self.0.borrow_mut().push_back(effect);
    }
    pub(crate) fn Pop(&self) -> Option<NativeResourceEffect> {
        self.0.borrow_mut().pop_front()
    }
    pub(crate) fn IsEmpty(&self) -> bool {
        self.0.borrow().is_empty()
    }
}
