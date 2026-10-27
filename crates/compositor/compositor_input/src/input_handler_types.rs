use crate::{
    main_thread_scrolling_reason::{MainThreadHitTestReasons, MainThreadRepaintReasons},
    overscroll_behavior::OverscrollBehavior,
    scroll_snap_data::TargetSnapAreaElementIds,
    ScrollGranularity,
};
use compositor_tree::geometry::{PointF, Vector2dF};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollInputType {
    kTouchscreen,
    kWheel,
    kAutoscroll,
    kScrollbar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollTrackingType {
    kWheelScroll,
    kTouchScroll,
    kScrollbarScroll,
    kMaxType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollingThread {
    kCompositor,
    kMain,
    kRaster,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputFeature {
    WaitForLateScrollEvents,
    OverscrollEffectOnNonRootScrollers,
    SnapFlingNearExtremes,
    LimitScrollDeltaToScrollerSize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollBeginThreadState {
    kScrollingOnCompositor = 0,
    kScrollingOnCompositorBlockedOnMain = 1,
    kScrollingOnMain = 2,
    kRasterInducingScroll = 3,
    kRasterInducingScrollBlockedOnMain = 4,
}

impl ScrollBeginThreadState {
    pub const kMaxValue: Self = Self::kRasterInducingScrollBlockedOnMain;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InputHandlerScrollResult {
    pub did_scroll: bool,
    pub did_overscroll_root: bool,
    pub accumulated_root_overscroll: Vector2dF,
    pub unused_scroll_delta: Vector2dF,
    pub overscroll_behavior: OverscrollBehavior,
    pub current_visual_offset: PointF,
    pub needs_main_thread_repaint: bool,
    pub hit_snap_constraint: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InputHandlerScrollEndResult {
    pub updates_need_main_thread_repaint: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollEventDispatchMode {
    kEnqueueScrollEvents,
    kDispatchScrollEventsImmediately,
    kUseScrollPredictorForEmptyQueue,
    kUseScrollPredictorForDeadline,
    kDispatchScrollEventsUntilDeadline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollThread {
    kScrollOnMainThread_NotUsed = 0,
    kScrollOnImplThread,
    kScrollIgnored,
    kScrollUnknown_NotUsed,
}

impl ScrollThread {
    pub const kLastScrollStatus: Self = Self::kScrollUnknown_NotUsed;
}

#[derive(Clone, Debug)]
pub struct ScrollStatus {
    pub thread: ScrollThread,
    pub main_thread_hit_test_reasons: MainThreadHitTestReasons,
    pub main_thread_repaint_reasons: MainThreadRepaintReasons,
    pub viewport_cannot_scroll: bool,
    pub raster_inducing: bool,
}

impl Default for ScrollStatus {
    fn default() -> Self {
        Self {
            thread: ScrollThread::kScrollOnImplThread,
            main_thread_hit_test_reasons: MainThreadHitTestReasons::default(),
            main_thread_repaint_reasons: MainThreadRepaintReasons::default(),
            viewport_cannot_scroll: false,
            raster_inducing: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchStartOrMoveEventListenerType {
    kNoHandler,
    kHandler,
    kHandlerOnScrollingLayer,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SnapAnimationData {
    pub animating_snap_target_ids_: TargetSnapAreaElementIds,
}

#[derive(Clone, Copy, Debug)]
pub struct ScrollVector {
    pub scroll_delta: Vector2dF,
    pub granularity: ScrollGranularity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapReason {
    kGestureScrollEnd,
    kScrollOffsetAnimationFinished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapFlingState {
    kNoFling,
    kNativeFling,
    kConstrainedNativeFling,
    kSnapFling,
}

pub const kScrollEpsilon: f32 = 0.1;

pub fn GetScrollEventDispatchMode(mode: &str) -> ScrollEventDispatchMode {
    match mode {
        "DispatchScrollEventsImmediately" => {
            ScrollEventDispatchMode::kDispatchScrollEventsImmediately
        }
        "UseScrollPredictorForEmptyQueue" => {
            ScrollEventDispatchMode::kUseScrollPredictorForEmptyQueue
        }
        "UseScrollPredictorForDeadline" => ScrollEventDispatchMode::kUseScrollPredictorForDeadline,
        "DispatchScrollEventsUntilDeadline" => {
            ScrollEventDispatchMode::kDispatchScrollEventsUntilDeadline
        }
        _ => ScrollEventDispatchMode::kEnqueueScrollEvents,
    }
}

pub const fn GetScrollInputTypeSuffix(input: ScrollInputType) -> &'static str {
    match input {
        ScrollInputType::kTouchscreen => "Touchscreen",
        ScrollInputType::kWheel => "Wheel",
        ScrollInputType::kAutoscroll => "Autoscroll",
        ScrollInputType::kScrollbar => "Scrollbar",
    }
}

pub fn ShouldAnimateScroll(enable_smooth_scroll: bool, granularity: ScrollGranularity) -> bool {
    enable_smooth_scroll && granularity != ScrollGranularity::kScrollByPrecisePixel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_mode_defaults_to_enqueue() {
        assert_eq!(
            GetScrollEventDispatchMode("unknown"),
            ScrollEventDispatchMode::kEnqueueScrollEvents
        );
        assert_eq!(
            GetScrollEventDispatchMode("UseScrollPredictorForDeadline"),
            ScrollEventDispatchMode::kUseScrollPredictorForDeadline
        );
    }

    #[test]
    fn precise_scroll_does_not_animate() {
        assert!(!ShouldAnimateScroll(
            true,
            ScrollGranularity::kScrollByPrecisePixel
        ));
        assert!(ShouldAnimateScroll(true, ScrollGranularity::kScrollByPixel));
    }
}
