//! The main-thread compositor notifications the Page domain receives: Blink's
//! LayerTreeView as cc::LayerTreeHostClient / LayerTreeHostSingleThreadClient
//! (third_party/blink/renderer/platform/widget/compositing/layer_tree_view.cc
//! DidCommit, DidPresentCompositorFrame, DidCompletePageScaleAnimation,
//! UpdateCompositorScrollState, ...), forwarded to its delegate. The main
//! half (`main_compositor`) builds them from the ProxyImpl -> ProxyMain tasks
//! and the main LayerTreeHost callbacks; the impl owners add the synchronous
//! input-call and presentation results. Plain cc / viz values; moved here
//! from the compositor facade, which never produced them.
use compositor_tree::geometry::{PointF, Vector2dF};
use compositor_values::begin_frame::{BeginFrameAck, BeginFrameArgs};
use compositor_values::frame_sequence_metrics::CustomTrackerResults;
use compositor_values::frame_skipped_reason::FrameSkippedReason;
use layerize::commit::{
    input_types::ElementId,
    main_effects::{CompositorScrollState, PacScrollEffect},
    paint_holding::{CommitDeferralReason, PaintHoldingReason},
    scroll_snap_data::TargetSnapAreaElementIds,
    scroll_source_type::ScrollSourceType,
};
use scroll_animation::TimeTicks;
use std::sync::Arc;
use viz_common::frame_timing_details::FrameTimingDetails;

/// A backend can acknowledge a buffer without knowing a display time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationUnavailableReason {
    BackendHasNoTimestamp,
    OffscreenTarget,
    DiscardedBeforePresentation,
}
#[derive(Clone, Debug)]
pub enum CompositorMainThreadEffect {
    /// ProxyImpl::ScheduledActionCommit records this immediately after
    /// FinishCommit, before the separate PostCommit and draw actions.
    MainFrameCommitCompleted {
        commit_finish_time: TimeTicks,
    },
    PresentationFeedbackUnavailable {
        frame_token: u32,
        surface_id: u64,
        viz_submission: u64,
        reason: PresentationUnavailableReason,
    },
    PresentationPromiseSwapped {
        request: u64,
        frame_token: u32,
        swap_time: TimeTicks,
    },
    PresentationPromiseDidNotSwap {
        request: u64,
        failure_time: TimeTicks,
    },
    FramePresented {
        frame_token: u32,
        details: FrameTimingDetails,
    },
    MainFrameUpdatesDeferralChanged {
        deferred: bool,
    },
    CommitDeferralChanged {
        deferred: bool,
        reason: PaintHoldingReason,
    },
    MainFrameCommitDeferred {
        reason: CommitDeferralReason,
    },
    ScrollAnimationCompleted(scroll_animation::compositor_mutation::ScrollAnimationCompletion),
    /// InputHandlerProxy's main-thread hit-test request
    /// (`CompositorInputEffect::WheelRequiresMainThreadHitTest`), which the
    /// main thread answers with `CompositorInput::ContinueWheelScroll`.
    WheelRequiresMainThreadHitTest {
        request: u64,
        position: PointF,
    },
    CanvasChildPaintRecords(Arc<canvas::CanvasChildRecordPublication>),
    UpdateCompositorScrollState(CompositorScrollState),
    LayerTreeFrameSinkLost,
    PageScaleAnimationCompleted,
    ViewportScaleChanged {
        scale: f32,
        delta: f32,
        pinch_active: bool,
        inner_scroll_delta: Vector2dF,
    },
    VideoBeginFrame {
        controller: u64,
        args: BeginFrameArgs,
    },
    VideoNeedsBeginFrames {
        needs: bool,
    },
    CompositorMetrics {
        results: CustomTrackerResults,
    },
    FrameNotProduced {
        ack: BeginFrameAck,
        reason: FrameSkippedReason,
    },
    /// A single-sequence embedding ran ProxyMain::BeginMainFrame; the Page
    /// schedules its lifecycle (the threaded embedding runs it inside).
    BeginMainFrame,
    SaveActiveEventMetrics,
    UpdateDeferMainFrameUpdate,
    ImageDecodeFinished {
        request_id: i32,
        speculative: bool,
        succeeded: bool,
    },
    BeginFrameSourcePaused {
        paused: bool,
    },
    ScrollOffsetChanged {
        sequence: u64,
        element: ElementId,
        offset: PointF,
        source: ScrollSourceType,
        snap: Option<TargetSnapAreaElementIds>,
    },
    ScrollbarsHiddenChanged {
        element: ElementId,
        hidden: bool,
    },
}

impl From<PacScrollEffect> for CompositorMainThreadEffect {
    fn from(effect: PacScrollEffect) -> Self {
        use PacScrollEffect as E;
        match effect {
            E::DidCompositorScroll {
                sequence,
                element,
                offset,
                source,
                snap,
            } => Self::ScrollOffsetChanged {
                sequence,
                element,
                offset,
                source,
                snap,
            },
            E::DidUpdateCompositorScrollState(state) => Self::UpdateCompositorScrollState(state),
            E::DidChangeScrollbarsHidden { element, hidden } => {
                Self::ScrollbarsHiddenChanged { element, hidden }
            }
        }
    }
}
