//! The input queue: requests the document thread moves to the compositor
//! thread's input handler.
//!
//! Chromium's WidgetInputHandlerManager owns this sequenced ingress: the main
//! thread posts its completed scroll hit test
//! (WidgetInputHandlerManager::FindScrollTargetReply ->
//! InputHandlerProxy::ContinueScrollBeginAfterMainThreadHitTest,
//! third_party/blink/renderer/platform/widget/input/
//! widget_input_handler_manager.cc), the blocking-wheel DOM
//! acknowledgement, and injected scrollbar gestures
//! (WebFrameWidgetImpl::InjectScrollbarGestureScroll ->
//! WidgetInputHandlerManager::DispatchScrollGestureEvent, core/frame/
//! web_frame_widget_impl.cc) to the compositor task runner, which runs them
//! in posting order. The values are moved; they hold no document state.
//! The impl owner converts each to its InputHandlerProxy call.

use crate::ScrollGranularity;
use compositor_tree::geometry::{Point, Vector2dF};
use compositor_tree::ElementId;
use scroll_animation::{TimeDelta, TimeTicks};

/// cc gesture-scroll phase (begin / update / end).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollPhase {
    Begin,
    Update,
    End,
}

#[derive(Clone, Copy, Debug)]
pub enum InputToImpl {
    /// WebFrameWidgetImpl's generated kScrollbar gesture. Values carry the
    /// original gesture signs; InputHandlerProxy performs cc state
    /// conversion.
    ScrollbarGesture {
        scroll_element_id: ElementId,
        delta: Vector2dF,
        granularity: ScrollGranularity,
        event_timestamp: TimeTicks,
        phase: ScrollPhase,
    },
    /// The DOM dispatch result of a blocking wheel the impl side kept
    /// (InputHandlerProxy's wheel event disposition).
    WheelDOMAcknowledgement { request: u64, default_prevented: bool },
    /// InputHandlerProxy::ContinueScrollBeginAfterMainThreadHitTest. The id
    /// is the impl side's hit-test request, never selected by the Page.
    ContinueWheelScroll {
        request: u64,
        scroll_element_id: Option<ElementId>,
    },
    /// A double-tap zoom's page scale animation. Chromium carries it on the
    /// next commit (LayerTreeHost::StartPageScaleAnimation); Rechrom keeps
    /// its existing delivery to the impl input handler in queue order.
    StartPageScaleAnimation {
        target: Point,
        use_anchor: bool,
        scale: f32,
        duration: TimeDelta,
    },
}
