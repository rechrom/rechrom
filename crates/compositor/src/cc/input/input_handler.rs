// Translation status: complete
// Source: cc/input/input_handler.h:1-942
// Source: cc/input/input_handler.cc:1-2793
// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use super::{
    actively_scrolling_type::ActivelyScrollingType,
    browser_controls_offset_tag_modifications::BrowserControlsOffsetTagModifications,
    browser_controls_state::BrowserControlsState,
    compositor_input_interfaces::InputDelegateForCompositor,
    event_listener_properties::{EventListenerClass, EventListenerProperties},
    main_thread_scrolling_reason::{
        MainThreadHitTestReason, MainThreadHitTestReasons, MainThreadRepaintReasons,
    },
    overscroll_behavior::{OverscrollBehavior, Type as OverscrollType},
    scroll_snap_data::{SnapPositionType, TargetSnapAreaElementIds},
    scroll_state::ScrollState,
    scroll_state_data::ScrollStateData,
    scroll_utils::{kMinFractionToStepWhenPaging, kPixelsPerLineStep, ScrollUtils},
    scrollbar::ScrollbarOrientation,
    scrollbar_controller::ScrollbarController,
    snap_selection_strategy::{SnapSelectionStrategy, SnapStopAlwaysFilter},
    touch_action::TouchAction,
};
use crate::adapter::{
    begin_frame::BeginFrameArgs,
    commit_types::*,
    geometry::{
        cpp_max, cpp_min, Point, PointF, QuadF, Rect, RectF, Size, SizeF, Transform, Vector2dF,
    },
    input_handler::*,
    input_types::{ElementId, ScrollGranularity},
    snap_geometry::RangeF,
    time::{TimeDelta, TimeTicks},
};
use crate::cc::{
    base::math_util::MathUtil,
    trees::{
        compositor_commit_data::CompositorCommitData, property_ids::*,
        scroll_source_type::ScrollSourceType,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
// Evaluate source call arguments before borrowing the live receiver. This is
// a Rust borrowing adaptation; evaluation order and values remain native.
macro_rules! delegate {
    ($host:ident, $method:ident($a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr, $a7:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        let a3 = $a3;
        let a4 = $a4;
        let a5 = $a5;
        let a6 = $a6;
        let a7 = $a7;
        $host.$method(a0, a1, a2, a3, a4, a5, a6, a7)
    }};
    ($host:ident, $method:ident($a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        let a3 = $a3;
        let a4 = $a4;
        let a5 = $a5;
        let a6 = $a6;
        $host.$method(a0, a1, a2, a3, a4, a5, a6)
    }};
    ($host:ident, $method:ident($a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        let a3 = $a3;
        let a4 = $a4;
        let a5 = $a5;
        $host.$method(a0, a1, a2, a3, a4, a5)
    }};
    ($host:ident, $method:ident($a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        let a3 = $a3;
        let a4 = $a4;
        $host.$method(a0, a1, a2, a3, a4)
    }};
    ($host:ident, $method:ident($a0:expr, $a1:expr, $a2:expr, $a3:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        let a3 = $a3;
        $host.$method(a0, a1, a2, a3)
    }};
    ($host:ident, $method:ident($a0:expr, $a1:expr, $a2:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        $host.$method(a0, a1, a2)
    }};
    ($host:ident, $method:ident($a0:expr, $a1:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        $host.$method(a0, a1)
    }};
    ($host:ident, $method:ident($a0:expr $(,)?)) => {{
        let a0 = $a0;
        $host.$method(a0)
    }};
    ($host:ident, $method:ident( $(,)?)) => {{
        $host.$method()
    }};
    ($host:ident, $($call:tt)*) => { $host.$($call)* };
}

// The source raw pointer is a resident identity. Resolve only for the duration
// of the source field expression, without retaining a node/tree borrow.
macro_rules! node_read {
    ($host:ident, $node:expr, $read:expr) => {{
        let node_id = $node;
        $host.ReadScrollNode(&node_id, $read)
    }};
}
macro_rules! node_write {
    ($host:ident, $node:expr, $write:expr) => {{
        let node_id = $node;
        $host.WriteScrollNode(&node_id, $write)
    }};
}
macro_rules! setting {
    ($host:ident,$field:ident) => {
        delegate!($host, GetSettings().$field())
    };
}
fn scale(v: Vector2dF, s: f32) -> Vector2dF {
    Vector2dF::new(v.x() * s, v.y() * s)
}
fn point_scale(p: PointF, s: f32) -> PointF {
    PointF::new(p.x() * s, p.y() * s)
}
fn vector(p: Point) -> PointF {
    PointF::new(p.x() as f32, p.y() as f32)
}
fn zero(v: Vector2dF) -> bool {
    v == Vector2dF::default()
}
// Source: cc/input/input_handler.h:53-88
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerResultType {
    kUnhandled = 0,
    kScrollbarScroll,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputHandlerPointerResult {
    pub type_: PointerResultType,
    pub scroll_units: ScrollGranularity,
    pub scroll_delta: Vector2dF,
    pub target_scroller: ElementId,
}
impl Default for InputHandlerPointerResult {
    fn default() -> Self {
        Self {
            type_: PointerResultType::kUnhandled,
            scroll_units: ScrollGranularity::kScrollByPrecisePixel,
            scroll_delta: Vector2dF::default(),
            target_scroller: ElementId::default(),
        }
    }
}
// Source: cc/input/input_handler.h:57-64
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
// Source: cc/input/input_handler.h:90-128
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
// Source: cc/input/input_handler.h:130-189
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollEventDispatchMode {
    kEnqueueScrollEvents,
    kDispatchScrollEventsImmediately,
    kUseScrollPredictorForEmptyQueue,
    kUseScrollPredictorForDeadline,
    kDispatchScrollEventsUntilDeadline,
}
pub trait InputHandlerClient {
    fn WillShutdown(&self);
    fn Animate(&self, time: TimeTicks);
    fn ReconcileElasticOverscrollAndRootScroll(&self);
    fn SetPrefersReducedMotion(&self, reduced: bool);
    fn UpdateRootLayerStateForSynchronousInputHandler(
        &self,
        offset: PointF,
        max: PointF,
        size: SizeF,
        scale: f32,
        min: f32,
        max_scale: f32,
    );
    fn DeliverInputForBeginFrame(&self, args: &BeginFrameArgs);
    fn DeliverInputForHighLatencyMode(&self);
    fn DeliverInputForDeadline(&self);
    fn DidFinishImplFrame(&self);
    fn HasQueuedInput(&self) -> bool;
    fn SetScrollEventDispatchMode(&self, mode: ScrollEventDispatchMode, ratio: f64);
}
// Source: cc/input/input_handler.h:214-258
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
// Source: cc/input/input_handler.h:260-291
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewportScrollResult {
    pub consumed_delta: Vector2dF,
    pub content_scrolled_delta: Vector2dF,
    pub outer_viewport_scrolled_delta: Vector2dF,
    pub inner_viewport_scrolled_delta: Vector2dF,
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
// Source: cc/input/input_handler.cc:1616-1637; cc/input/input_handler.h:818,849,874.
// Read-only output borrows the canonical input state. It owns no flags, state,
// map or host/tree reference, so selected-tree callbacks can query this source
// body with the actual receiver without aliasing their mutable InputHost.
pub struct InputHandlerView<'a> {
    pinch_gesture_active_: &'a bool,
    external_pinch_gesture_active_: &'a bool,
    last_scroll_update_state_: &'a Option<ScrollState>,
    delta_consumed_for_scroll_gesture_: &'a bool,
    snap_animation_data_map_: &'a BTreeMap<ElementId, SnapAnimationData>,
}
impl InputHandlerView<'_> {
    // Source: cc/input/input_handler.h:521-523. Borrowed resident flags.
    pub(crate) fn PinchGestureActive(&self) -> bool {
        *self.pinch_gesture_active_ || *self.external_pinch_gesture_active_
    }
    pub(crate) fn reborrow(&self) -> InputHandlerView<'_> {
        InputHandlerView {
            pinch_gesture_active_: &self.pinch_gesture_active_,
            external_pinch_gesture_active_: &self.external_pinch_gesture_active_,
            last_scroll_update_state_: self.last_scroll_update_state_,
            delta_consumed_for_scroll_gesture_: self.delta_consumed_for_scroll_gesture_,
            snap_animation_data_map_: self.snap_animation_data_map_,
        }
    }
    // Source: cc/input/input_handler.cc:1597-1614.
    pub(crate) fn IsCurrentlyScrollingForTree(
        &self,
        scroll_tree: &crate::cc::trees::property_tree::ScrollTree,
    ) -> bool {
        self.IsCurrentlyScrollingWithCurrentNode(scroll_tree.CurrentlyScrollingNode().is_some())
    }
    fn IsCurrentlyScrollingWithCurrentNode(&self, has_current: bool) -> bool {
        if has_current {
            return true;
        }
        for data in self.snap_animation_data_map_.values() {
            if data.animating_snap_target_ids_ != TargetSnapAreaElementIds::default() {
                return true;
            }
        }
        false
    }

    // Source: cc/input/input_handler.cc:2124-2146.
    pub fn ScrollSingleNode<B: InputHandlerTypes, C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        node: &B::ScrollNode,
        delta: Vector2dF,
        point: Point,
        direct: bool,
    ) -> Vector2dF {
        InputHandler::<B>::ScrollSingleNodeForNode(host, node, delta, point, direct)
    }
    // Source: cc/input/input_handler.cc:2423-2457.
    pub fn CanConsumeDelta<B: InputHandlerTypes, C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        state: &ScrollState,
        node: &B::ScrollNode,
    ) -> bool {
        InputHandler::<B>::CanConsumeDeltaForState(
            host,
            self.last_scroll_update_state_.as_ref(),
            state,
            node,
        )
    }
    // Source: cc/input/input_handler.cc:1976-2001.
    pub fn ComputeScrollDelta<B: InputHandlerTypes, C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        node: &B::ScrollNode,
        delta: Vector2dF,
        state: Option<&ScrollState>,
    ) -> Vector2dF {
        InputHandler::<B>::ComputeScrollDeltaForState(
            host,
            self.last_scroll_update_state_.as_ref(),
            node,
            delta,
            state,
        )
    }
    // Source: cc/input/input_handler.cc:1616-1637,1681-1683.
    pub fn GetActivelyScrollingTypeForTree<S: InputSettings + ?Sized>(
        &self,
        scroll_tree: &crate::cc::trees::property_tree::ScrollTree,
        settings: &S,
    ) -> ActivelyScrollingType {
        self.GetActivelyScrollingTypeWithCurrentNode(
            scroll_tree.CurrentlyScrollingNode().is_some(),
            || settings,
        )
    }
    // Source: cc/input/input_handler.cc:1616-1637.
    // Settings is evaluated only in the source ShouldAnimateScroll branch.
    fn GetActivelyScrollingTypeWithCurrentNode<'s, S: InputSettings + ?Sized + 's>(
        &self,
        currently_scrolling_node: bool,
        settings: impl FnOnce() -> &'s S,
    ) -> ActivelyScrollingType {
        if currently_scrolling_node
            && self.last_scroll_update_state_.is_some()
            && *self.delta_consumed_for_scroll_gesture_
        {
            if ShouldAnimateScrollForState(
                settings(),
                self.last_scroll_update_state_.as_ref().unwrap(),
            ) {
                return ActivelyScrollingType::kAnimated;
            }
            return ActivelyScrollingType::kPrecise;
        }
        for data in self.snap_animation_data_map_.values() {
            if data.animating_snap_target_ids_ != TargetSnapAreaElementIds::default() {
                return ActivelyScrollingType::kAnimated;
            }
        }
        ActivelyScrollingType::kNone
    }
}
// Source: cc/input/input_handler.cc:2459-2467.
fn ShouldAnimateScrollForState(
    settings: &(impl InputSettings + ?Sized),
    state: &ScrollState,
) -> bool {
    if !settings.enable_smooth_scroll() {
        return false;
    }
    let precise = state.delta_granularity() == ScrollGranularity::kScrollByPrecisePixel;
    !precise
}
// Source: cc/input/input_handler.h:334-337
#[derive(Clone, Copy, Debug)]
pub struct ScrollVector {
    pub scroll_delta: Vector2dF,
    pub granularity: ScrollGranularity,
}
// Source: cc/input/input_handler.h:636,682-686,828-833
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapReason {
    kGestureScrollEnd,
    kScrollOffsetAnimationFinished,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SnapFlingState {
    kNoFling,
    kNativeFling,
    kConstrainedNativeFling,
    kSnapFling,
}
struct ScrollHitTestResult<N> {
    scroll_node: Option<N>,
    hit_test_successful: bool,
    main_thread_hit_test_reasons: MainThreadHitTestReasons,
}
// Source: cc/input/input_handler.h:786-937
pub struct InputHandler<B: InputHandlerTypes> {
    owner_id_: u64,
    types_: std::marker::PhantomData<fn() -> B>,
    input_handler_client_: Option<Arc<dyn InputHandlerClient>>,
    scroll_elasticity_helper_: Option<B::Elasticity>,
    scrollbar_controller_: ScrollbarController,
    accumulated_root_overscroll_: Vector2dF,
    overscroll_delta_for_main_thread_: Vector2dF,
    latched_scroll_type_: Option<ScrollInputType>,
    prevent_scroll_axis_locking_: Option<bool>,
    last_scroll_begin_state_: Option<ScrollState>,
    last_scroll_update_state_: Option<ScrollState>,
    scroll_animating_snap_target_ids_: TargetSnapAreaElementIds,
    snap_fling_state_: SnapFlingState,
    fling_snap_constrain_x_: Option<RangeF>,
    fling_snap_constrain_y_: Option<RangeF>,
    updated_snapped_elements_: BTreeMap<ElementId, TargetSnapAreaElementIds>,
    snap_animation_data_map_: BTreeMap<ElementId, SnapAnimationData>,
    scroll_element_id_mouse_currently_over_: ElementId,
    scroll_element_id_mouse_currently_captured_: ElementId,
    last_latched_scroller_: ElementId,
    deferred_scroll_ends_: BTreeSet<ElementId>,
    did_scroll_x_for_scroll_gesture_: bool,
    did_scroll_y_for_scroll_gesture_: bool,
    delta_consumed_for_scroll_gesture_: bool,
    outer_viewport_consumed_delta_: bool,
    inner_viewport_consumed_delta_: bool,
    force_smooth_wheel_scrolling_for_testing_: bool,
    pinch_gesture_active_: bool,
    external_pinch_gesture_active_: bool,
    pinch_gesture_end_should_clear_scrolling_node_: bool,
    has_pinch_zoomed_: bool,
    has_scrolled_by_wheel_: bool,
    has_scrolled_by_touch_: bool,
    has_scrolled_by_precisiontouchpad_: bool,
    has_scrolled_by_scrollbar_: bool,
    prefers_reduced_motion_: bool,
    is_handling_touch_sequence_: bool,
    snap_strategy_: Option<Box<dyn SnapSelectionStrategy>>,
    pending_scrollend_containers_: BTreeSet<ElementId>,
    last_scroll_begin_time_: TimeTicks,
    last_latched_scroll_source_type_: ScrollSourceType,
    has_received_scroll_update_for_sequence_: bool,
    weak_factory_: Option<Arc<()>>,
}
// Source: cc/input/input_handler.cc:50
#[allow(dead_code)]
enum SlowScrollMetricThread {
    MAIN_THREAD,
    CC_THREAD,
}
// Source: cc/input/input_handler.cc:52-76
fn GetScrollEventDispatchMode(mode: &str) -> ScrollEventDispatchMode {
    // Source dependency constants: cc/base/features.cc:111-119.
    if mode == "DispatchScrollEventsImmediately" {
        return ScrollEventDispatchMode::kDispatchScrollEventsImmediately;
    } else if mode == "UseScrollPredictorForEmptyQueue" {
        return ScrollEventDispatchMode::kUseScrollPredictorForEmptyQueue;
    } else if mode == "UseScrollPredictorForDeadline" {
        return ScrollEventDispatchMode::kUseScrollPredictorForDeadline;
    } else if mode == "DispatchScrollEventsUntilDeadline" {
        return ScrollEventDispatchMode::kDispatchScrollEventsUntilDeadline;
    }
    ScrollEventDispatchMode::kEnqueueScrollEvents
}
// Source: cc/input/input_handler.cc:78-90
fn GetScrollInputTypeSuffix(input: ScrollInputType) -> &'static str {
    match input {
        ScrollInputType::kTouchscreen => "Touchscreen",
        ScrollInputType::kWheel => "Wheel",
        ScrollInputType::kAutoscroll => "Autoscroll",
        ScrollInputType::kScrollbar => "Scrollbar",
    }
}
// Source: cc/input/input_handler.cc:94-98
pub const kScrollEpsilon: f32 = 0.1;
impl<B: InputHandlerTypes> InputHandler<B> {
    // Source: cc/input/input_handler.cc:101-107
    pub fn Create<C: InputHost<B> + InputBindingHost<B> + ?Sized>(host: &mut C) -> InputHandlerWeak
    where
        B: crate::adapter::input_services::InputHostTypes<InputHandler = Self>,
    {
        let (input, weak) = Self::CreateUnbound(host);
        host.BindToInputHandler(input);
        weak
    }
    // Source: cc/input/input_handler.cc:101-107. Rust borrow split: release
    // the per-call InputHost facade before binding the same Box into its owner.
    pub(crate) fn CreateUnbound<C:InputHost<B>+?Sized>(host:&mut C)->(Box<Self>,InputHandlerWeak){
        let input=Box::new(Self::new(host));
        let weak=input.AsWeakPtr();
        (input,weak)
    }
    // Source: cc/input/input_handler.cc:109-112; cc/input/input_handler.h:786-937
    pub fn new<C: InputHost<B> + ?Sized>(host: &mut C) -> Self {
        let owner_id = host.input_handler_id();
        let scrollbar_controller = ScrollbarController::new_for_owner(owner_id);
        Self {
            owner_id_: owner_id,
            types_: std::marker::PhantomData,
            input_handler_client_: None,
            scroll_elasticity_helper_: None,
            scrollbar_controller_: scrollbar_controller,
            accumulated_root_overscroll_: Vector2dF::default(),
            overscroll_delta_for_main_thread_: Vector2dF::default(),
            latched_scroll_type_: None,
            prevent_scroll_axis_locking_: None,
            last_scroll_begin_state_: None,
            last_scroll_update_state_: None,
            scroll_animating_snap_target_ids_: TargetSnapAreaElementIds::default(),
            snap_fling_state_: SnapFlingState::kNoFling,
            fling_snap_constrain_x_: None,
            fling_snap_constrain_y_: None,
            updated_snapped_elements_: BTreeMap::new(),
            snap_animation_data_map_: BTreeMap::new(),
            scroll_element_id_mouse_currently_over_: ElementId::default(),
            scroll_element_id_mouse_currently_captured_: ElementId::default(),
            last_latched_scroller_: ElementId::default(),
            deferred_scroll_ends_: BTreeSet::new(),
            did_scroll_x_for_scroll_gesture_: false,
            did_scroll_y_for_scroll_gesture_: false,
            delta_consumed_for_scroll_gesture_: false,
            outer_viewport_consumed_delta_: false,
            inner_viewport_consumed_delta_: false,
            force_smooth_wheel_scrolling_for_testing_: false,
            pinch_gesture_active_: false,
            external_pinch_gesture_active_: false,
            pinch_gesture_end_should_clear_scrolling_node_: false,
            has_pinch_zoomed_: false,
            has_scrolled_by_wheel_: false,
            has_scrolled_by_touch_: false,
            has_scrolled_by_precisiontouchpad_: false,
            has_scrolled_by_scrollbar_: false,
            prefers_reduced_motion_: false,
            is_handling_touch_sequence_: false,
            snap_strategy_: None,
            pending_scrollend_containers_: BTreeSet::new(),
            last_scroll_begin_time_: TimeTicks::default(),
            last_latched_scroll_source_type_: ScrollSourceType::kNone,
            has_received_scroll_update_for_sequence_: false,
            weak_factory_: Some(Arc::new(())),
        }
    }
    // Source: cc/input/input_handler.cc:120-122
    pub fn AsWeakPtr(&self) -> InputHandlerWeak {
        InputHandlerWeak {
            owner_id: self.owner_id_,
            alive: Arc::downgrade(self.weak_factory_.as_ref().unwrap()),
        }
    }
    // Source: cc/input/input_handler.cc:124-133
    pub fn BindToClient<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        client: Arc<dyn InputHandlerClient>,
    ) {
        debug_assert!(self.input_handler_client_.is_none());
        self.input_handler_client_ = Some(client.clone());
        client.SetPrefersReducedMotion(self.prefers_reduced_motion_);
        if host.feature(InputFeature::WaitForLateScrollEvents) {
            client.SetScrollEventDispatchMode(
                GetScrollEventDispatchMode(&host.scroll_event_dispatch_mode()),
                host.late_scroll_deadline_ratio(),
            );
        }
    }
    // Source: cc/input/input_handler.cc:135-316
    pub fn ScrollBegin<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        state: &mut ScrollState,
        input: ScrollInputType,
    ) -> ScrollStatus {
        debug_assert!(state.delta_x() == 0. && state.delta_y() == 0.);
        let mut status = ScrollStatus::default();
        let _trace = host.BeginTrace("InputHandler::ScrollBegin", &[]);
        let mut had_animation = false;
        if state.delta_granularity() == ScrollGranularity::kScrollByPrecisePixel {
            if let Some(n) = self.GetAnimatingNodeForCurrentScrollingNode(host) {
                had_animation = true;
                delegate!(
                    host,
                    ScrollAnimationAbort(node_read!(host, n, |n| n.element_id))
                );
            }
            if let Some(n) = self.CurrentlyScrollingNode(host) {
                self.ClearAnimatingSnapTargetsForElement(node_read!(host, n, |n| n.element_id));
            }
        }
        if let Some(n) = self.CurrentlyScrollingNode(host) {
            if Some(input) == self.latched_scroll_type_
                && (self.GetAnimatingNodeForCurrentScrollingNode(host).is_some()
                    || had_animation
                    || node_read!(host, n, |n| n.snap_container_data.is_some()))
            {
                let id = node_read!(host, n, |n| n.element_id);
                debug_assert!(self.deferred_scroll_ends_.contains(&id));
                self.deferred_scroll_ends_.remove(&id);
                status.raster_inducing = host.ScrollTree_CanRealizeScrollsOnPendingTree(&n);
                return status;
            }
        }
        self.ClearCurrentlyScrollingNode(host);
        let target = state.target_element_id();
        let scrolling;
        if target.IsValid()
            && (state.main_thread_hit_tested_reasons().empty() || state.is_scrollbar_interaction())
        {
            host.InstantTrace("Latched scroll node provided", &[]);
            scrolling = host.ScrollTree_MutableFindNodeFromElementId(target);
        } else {
            let starting;
            if target.IsValid() {
                host.InstantTrace("Unlatched scroll node provided", &[]);
                debug_assert!(!state.main_thread_hit_tested_reasons().empty());
                starting = host.ScrollTree_MutableFindNodeFromElementId(target);
                if starting.is_none() {
                    status.thread = ScrollThread::kScrollIgnored;
                    return status;
                }
            } else {
                host.InstantTrace("Hit Testing for ScrollNode", &[]);
                let point = point_scale(
                    PointF::new(state.position_x() as f32, state.position_y() as f32),
                    host.ActiveTree_device_scale_factor(),
                );
                assert!(state.main_thread_hit_tested_reasons().empty());
                let hit = self.HitTestScrollNode(host, point);
                if !hit.hit_test_successful {
                    host.InstantTrace("Request Main Thread Hit Test", &[]);
                    status.thread = ScrollThread::kScrollOnImplThread;
                    debug_assert!(!hit.main_thread_hit_test_reasons.empty());
                    status.main_thread_hit_test_reasons = hit.main_thread_hit_test_reasons;
                    return status;
                }
                starting = hit.scroll_node;
            }
            scrolling = self.FindNodeToLatch(host, state, starting.clone(), input);
            if let Some(n) = &scrolling {
                if setting!(host, enable_fluent_overlay_scrollbar)
                    && node_read!(host, n, |n| n.element_id)
                        != node_read!(host, starting.as_ref().unwrap(), |n| n.element_id)
                {
                    delegate!(
                        host,
                        WillScrollContent(
                            node_read!(host, starting.as_ref().unwrap(), |n| n.element_id)
                        )
                    );
                }
            }
        }
        let Some(n) = scrolling else {
            if setting!(host, is_for_embedded_frame) {
                host.InstantTrace("Ignored - No ScrollNode (OOPIF or FencedFrame)", &[]);
            } else {
                host.InstantTrace("Ignored - No ScrollNode", &[]);
            }
            status.thread = ScrollThread::kScrollIgnored;
            return status;
        };
        debug_assert_eq!(status.thread, ScrollThread::kScrollOnImplThread);
        host.SetCurrentlyScrollingNode(n.clone());
        status.main_thread_repaint_reasons =
            node_read!(host, n, |n| n.main_thread_repaint_reasons.clone());
        status.raster_inducing = host.ScrollTree_CanRealizeScrollsOnPendingTree(&n);
        self.DidLatchToScroller(host, state, input);
        let current = self.CurrentlyScrollingNode(host).unwrap();
        if host.Viewport_ShouldScroll(&current) {
            self.outer_viewport_consumed_delta_ = false;
            self.inner_viewport_consumed_delta_ = false;
            if !host.Viewport_CanScroll(&self.View(), &current, state) {
                status.viewport_cannot_scroll = true;
            }
        }
        status
    }
    // Source: cc/input/input_handler.cc:318-337
    pub fn RootScrollBegin<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        state: &mut ScrollState,
        input: ScrollInputType,
    ) -> ScrollStatus {
        let _trace = host.BeginTrace("InputHandler::RootScrollBegin", &[]);
        let Some(outer) = self.OuterViewportScrollNode(host) else {
            return ScrollStatus {
                thread: ScrollThread::kScrollIgnored,
                ..ScrollStatus::default()
            };
        };
        state
            .data()
            .set_current_native_scrolling_element(node_read!(host, outer, |n| n.element_id));
        let status = self.ScrollBegin(host, state, input);
        debug_assert!(status.main_thread_hit_test_reasons.empty());
        status
    }
    // Source: cc/input/input_handler.cc:339-516
    pub fn ScrollUpdate<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        mut state: ScrollState,
        delayed_by: TimeDelta,
    ) -> InputHandlerScrollResult {
        debug_assert!(!state.data().current_native_scrolling_element().IsValid());
        let _trace = host.BeginTrace(
            "InputHandler::ScrollUpdate",
            &[
                ("dx", InputTraceValue::Float(state.delta_x())),
                ("dy", InputTraceValue::Float(state.delta_y())),
                (
                    "granularity",
                    InputTraceValue::Granularity(state.delta_granularity()),
                ),
            ],
        );
        let Some(node) = self.CurrentlyScrollingNode(host) else {
            return InputHandlerScrollResult::default();
        };
        self.last_scroll_update_state_ = Some(state.clone());
        if state.is_scrollbar_interaction()
            && state.delta_granularity() != ScrollGranularity::kScrollByPrecisePixel
        {
            self.AdjustScrollDeltaForScrollbarSnap(host, &mut state);
        }
        let unconstrained = self.prevent_scroll_axis_locking_.unwrap();
        let x = if unconstrained {
            state.delta_x_unconstrained()
        } else {
            state.delta_x()
        };
        let y = if unconstrained {
            state.delta_y_unconstrained()
        } else {
            state.delta_y()
        };
        let mut resolved = self.ResolveScrollGranularityToPixels(
            host,
            &node,
            Vector2dF::new(x as f32, y as f32),
            state.delta_granularity(),
        );
        let mut hit_constraint = false;
        let current = self.GetVisualScrollOffset(host, &node);
        if let Some(constrained) = self.ConstrainFling(current, current + resolved) {
            let applied = constrained - current;
            if applied != resolved {
                resolved = applied;
                hit_constraint = true;
            }
        }
        state.data().delta_x = resolved.x() as f64;
        state.data().delta_y = resolved.y() as f64;
        if state.delta_granularity() != ScrollGranularity::kScrollByPrecisePixel {
            state.data().delta_granularity = ScrollGranularity::kScrollByPixel;
        }
        delegate!(
            host,
            WillScrollContent(node_read!(host, node, |n| n.element_id))
        );
        let initial_controls = delegate!(host, GetBrowserControlsTopOffset());
        self.ScrollLatchedScroller(host, &mut state, delayed_by);
        self.delta_consumed_for_scroll_gesture_ |= state.delta_consumed_for_scroll_sequence();
        let did_any = (state.delta_x() - resolved.x() as f64).abs() > kScrollEpsilon as f64
            || (state.delta_y() - resolved.y() as f64).abs() > kScrollEpsilon as f64;
        if did_any {
            delegate!(host, DidScrollForMetrics());
        }
        let did_x = state.caused_scroll_x();
        let did_y = state.caused_scroll_y();
        let did_content = did_x || did_y;
        if did_content {
            let animated = self.ShouldAnimateScroll(host, &state);
            delegate!(
                host,
                DidScrollContent(
                    &self.View(),
                    node_read!(host, node, |n| n.element_id),
                    animated,
                    &resolved
                )
            );
        }
        self.SetNeedsCommit(host);
        if did_x {
            self.accumulated_root_overscroll_.set_x(0.);
        }
        if did_y {
            self.accumulated_root_overscroll_.set_y(0.);
        }
        let mut unused = Vector2dF::new(state.delta_x() as f32, state.delta_y() as f32);
        let root = host.Viewport_ShouldScroll(&node);
        if root {
            if let Some(inner) = self.InnerViewportScrollNode(host) {
                unused = self.UserScrollableDelta(host, &inner, unused);
            }
            self.accumulated_root_overscroll_ += unused;
        }
        if !host.feature(InputFeature::OverscrollEffectOnNonRootScrollers) && !root {
            unused = Vector2dF::default();
        }
        if host.feature(InputFeature::OverscrollEffectOnNonRootScrollers) && !root {
            if !node_read!(host, node, |n| n
                .overscroll_behavior
                .HasXLocalBorderEffects())
            {
                unused.set_x(0.);
            }
            if !node_read!(host, node, |n| n
                .overscroll_behavior
                .HasYLocalBorderEffects())
            {
                unused.set_y(0.);
            }
        }
        let did_controls = initial_controls != delegate!(host, GetBrowserControlsTopOffset());
        let mut result = InputHandlerScrollResult::default();
        result.did_scroll = did_content || did_controls;
        result.hit_snap_constraint = hit_constraint;
        result.did_overscroll_root = root && !zero(unused);
        result.accumulated_root_overscroll = self.accumulated_root_overscroll_;
        result.unused_scroll_delta = unused;
        result.overscroll_behavior = if state.is_scroll_chain_cut() {
            OverscrollBehavior::FromType(OverscrollType::kNone)
        } else {
            host.ActiveTree_overscroll_behavior()
        };
        if result.did_scroll {
            self.UpdateRootLayerStateForSynchronousInputHandler(host);
        }
        if !self.has_received_scroll_update_for_sequence_
            && result.did_scroll
            && host.ShouldRecordSubsampledMetric(0.01)
        {
            host.HistogramCounts("Input.FirstGestureScrollUpdate.DeltaY", resolved.y());
        }
        self.has_received_scroll_update_for_sequence_ = true;
        result.current_visual_offset = self.GetVisualScrollOffset(host, &node);
        result
            .current_visual_offset
            .Scale(host.ActiveTree_page_scale_factor_for_scroll());
        if host.ScrollTree_ShouldRealizeScrollsOnMain(&node) {
            result.needs_main_thread_repaint = true;
        }
        delegate!(host, TickScrollAnimations());
        result
    }
    // Source: cc/input/input_handler.cc:518-556
    fn AdjustScrollDeltaForScrollbarSnap<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        state: &mut ScrollState,
    ) {
        let Some(node) = self.CurrentlyScrollingNode(host) else {
            return;
        };
        if !node_read!(host, node, |n| n.snap_container_data.is_some()) {
            return;
        }
        let current = self.GetVisualScrollOffset(host, &node);
        let strategy = if state.delta_granularity() == ScrollGranularity::kScrollByPage {
            <dyn SnapSelectionStrategy>::CreateForPageScroll(
                current,
                Vector2dF::new(state.delta_x() as f32, state.delta_y() as f32),
                self.PageSize(host, &node),
                true,
                SnapStopAlwaysFilter::kIgnore,
            )
        } else {
            <dyn SnapSelectionStrategy>::CreateForDirection(
                current,
                self.ResolveScrollGranularityToPixels(
                    host,
                    &node,
                    Vector2dF::new(state.delta_x() as f32, state.delta_y() as f32),
                    state.delta_granularity(),
                ),
                true,
                SnapStopAlwaysFilter::kIgnore,
            )
        };
        let snap = node_read!(host, node, |n| {
            n.snap_container_data
                .as_ref()
                .unwrap()
                .FindSnapPosition(strategy.as_ref())
        });
        if snap.r#type == SnapPositionType::kNone {
            return;
        }
        state.data().delta_x = (snap.position.x() - current.x()) as f64;
        state.data().delta_y = (snap.position.y() - current.y()) as f64;
        state.data().delta_granularity = ScrollGranularity::kScrollByPixel;
    }
    // Source: cc/input/input_handler.cc:558-584
    fn InsertPendingScrollendContainer<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        id: ElementId,
    ) {
        let outer = self.OuterViewportScrollNode(host);
        if outer
            .as_ref()
            .is_none_or(|n| node_read!(host, n, |n| n.element_id) != id)
            || self.outer_viewport_consumed_delta_
        {
            self.pending_scrollend_containers_.insert(id);
        }
        if let Some(outer) = outer {
            if node_read!(host, outer, |n| n.element_id) == id {
                if self.outer_viewport_consumed_delta_ {
                    self.outer_viewport_consumed_delta_ = false;
                }
                if self.inner_viewport_consumed_delta_ {
                    self.pending_scrollend_containers_.insert(node_read!(
                        host,
                        self.InnerViewportScrollNode(host).unwrap(),
                        |n| n.element_id
                    ));
                    self.inner_viewport_consumed_delta_ = false;
                }
            }
        }
    }
    // Source: cc/input/input_handler.cc:586-590
    pub fn ScrollEnd<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        snap: bool,
        compensated: Option<ScrollVector>,
    ) -> InputHandlerScrollEndResult {
        self.ScrollEndForNode(host, None, snap, compensated)
    }
    // Source: cc/input/input_handler.cc:592-694
    fn ScrollEndForNode<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        node: Option<B::ScrollNode>,
        snap: bool,
        compensated: Option<ScrollVector>,
    ) -> InputHandlerScrollEndResult {
        let latched = self.CurrentlyScrollingNode(host);
        let mut result = InputHandlerScrollEndResult::default();
        if let Some(n) = node.as_ref().or(latched.as_ref()) {
            result.updates_need_main_thread_repaint = host.ScrollTree_ShouldRealizeScrollsOnMain(n);
        }
        if node.is_some() && node != latched {
            let node = node.unwrap();
            debug_assert!(!snap);
            let id = node_read!(host, node, |n| n.element_id);
            self.InsertPendingScrollendContainer(host, id);
            self.deferred_scroll_ends_.remove(&id);
            if latched.is_none() {
                self.scrollbar_controller_.ResetState();
                self.EndOfScrollCleanup(host, &node, compensated);
            }
            self.snap_animation_data_map_.remove(&id);
        } else if let Some(n) = latched {
            let id = node_read!(host, n, |n| n.element_id);
            self.scrollbar_controller_.ResetState();
            if self.GetAnimatingNodeForCurrentScrollingNode(host).is_some() {
                debug_assert!(!self.deferred_scroll_ends_.contains(&id));
                self.deferred_scroll_ends_.insert(id);
                return result;
            }
            let overscroll = self
                .scroll_elasticity_helper_
                .as_ref()
                .is_some_and(|h| !zero(host.StretchAmount(h, id)));
            if overscroll {
                self.deferred_scroll_ends_.insert(id);
                return result;
            }
            if snap && self.SnapAtScrollEnd(host, SnapReason::kGestureScrollEnd) {
                self.deferred_scroll_ends_.insert(id);
                return result;
            }
            debug_assert!(self.latched_scroll_type_.is_some());
            if self.did_scroll_x_for_scroll_gesture_ || self.did_scroll_y_for_scroll_gesture_ {
                self.InsertPendingScrollendContainer(host, id);
            }
            self.EndOfScrollCleanup(host, &n, compensated);
            self.snap_animation_data_map_.remove(&id);
            self.ClearCurrentlyScrollingNode(host);
        } else {
            self.scrollbar_controller_.ResetState();
            return result;
        }
        self.SetNeedsCommit(host);
        result
    }
    // Source: cc/input/input_handler.cc:604-619 (source's local cleanup closure).
    fn EndOfScrollCleanup<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        node: &B::ScrollNode,
        compensated: Option<ScrollVector>,
    ) {
        let mut pixels = Vector2dF::default();
        if let Some(delta) = compensated {
            pixels = self.ResolveScrollGranularityToPixels(
                host,
                node,
                delta.scroll_delta,
                delta.granularity,
            );
        }
        delegate!(host, ScrollEnd(&pixels));
        self.deferred_scroll_ends_
            .remove(&node_read!(host, node, |n| n.element_id));
        self.snap_fling_state_ = SnapFlingState::kNoFling;
        self.fling_snap_constrain_x_ = None;
        self.fling_snap_constrain_y_ = None;
        self.snap_strategy_ = None;
        self.has_received_scroll_update_for_sequence_ = false;
    }
    // Source: cc/input/input_handler.cc:696-726
    pub fn RecordScrollBegin<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        input: ScrollInputType,
        state: ScrollBeginThreadState,
    ) {
        self.last_scroll_begin_time_ = host.Now();
        let kind = self.GetTrackerTypeForScroll(input);
        debug_assert_ne!(kind, ScrollTrackingType::kMaxType);
        let thread = match state {
            ScrollBeginThreadState::kScrollingOnCompositor => ScrollingThread::kCompositor,
            ScrollBeginThreadState::kScrollingOnMain
            | ScrollBeginThreadState::kScrollingOnCompositorBlockedOnMain
            | ScrollBeginThreadState::kRasterInducingScrollBlockedOnMain => ScrollingThread::kMain,
            ScrollBeginThreadState::kRasterInducingScroll => ScrollingThread::kRaster,
        };
        let tracker = host.TrackerType(kind);
        let thread = host.DrivingThread(thread);
        delegate!(host, StartScrollSequence(tracker, thread));
    }
    // Source: cc/input/input_handler.cc:728-733
    pub fn RecordScrollEnd<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        input: ScrollInputType,
    ) {
        let tracker = host.TrackerType(self.GetTrackerTypeForScroll(input));
        delegate!(host, StopSequence(tracker));
        delegate!(
            host,
            HistogramTimes(
                &format!("Event.Scroll.{}", GetScrollInputTypeSuffix(input)),
                host.Now() - self.last_scroll_begin_time_,
            )
        );
    }
    // Source: cc/input/input_handler.cc:735-781
    pub fn MouseMoveAt<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        point: Point,
    ) -> InputHandlerPointerResult {
        let view = InputHandlerView {
            pinch_gesture_active_: &self.pinch_gesture_active_,
            external_pinch_gesture_active_: &self.external_pinch_gesture_active_,
            last_scroll_update_state_: &self.last_scroll_update_state_,
            delta_consumed_for_scroll_gesture_: &self.delta_consumed_for_scroll_gesture_,
            snap_animation_data_map_: &self.snap_animation_data_map_,
        };
        let result = host.WithScrollbarHost(
            &view,
            self.last_scroll_update_state_.as_ref(),
            |scrollbar_host| {
                self.scrollbar_controller_
                    .HandlePointerMove(scrollbar_host, vector(point))
            },
        );
        if !delegate!(host, HasAnimatedScrollbars()) {
            return result;
        }
        let device = point_scale(vector(point), host.ActiveTree_device_scale_factor());
        let hit = self.HitTestScrollNode(host, device);
        let Some(mut node) = hit.scroll_node else {
            return result;
        };
        if !hit.hit_test_successful {
            return result;
        }
        if node_read!(host, node, |n| n.scrolls_inner_viewport) {
            node = self.OuterViewportScrollNode(host).unwrap();
        }
        let id = node_read!(host, node, |n| n.element_id);
        if id != self.scroll_element_id_mouse_currently_over_ {
            delegate!(
                host,
                ScrollbarAnimationMouseLeave(self.scroll_element_id_mouse_currently_over_)
            );
            self.scroll_element_id_mouse_currently_over_ = id;
            if !node_read!(host, node, |n| n.scrolls_inner_viewport)
                && !node_read!(host, node, |n| n.scrolls_outer_viewport)
                && setting!(host, scrollbar_flash_when_mouse_enter)
            {
                delegate!(host, DidMouseEnterNonViewportScroller(id));
            }
        }
        delegate!(host, ScrollbarAnimationMouseMove(id, device));
        result
    }
    // Source: cc/input/input_handler.cc:783-787
    pub fn HitTest<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        point: PointF,
    ) -> PointerResultType {
        let view = InputHandlerView {
            pinch_gesture_active_: &self.pinch_gesture_active_,
            external_pinch_gesture_active_: &self.external_pinch_gesture_active_,
            last_scroll_update_state_: &self.last_scroll_update_state_,
            delta_consumed_for_scroll_gesture_: &self.delta_consumed_for_scroll_gesture_,
            snap_animation_data_map_: &self.snap_animation_data_map_,
        };
        if host
            .WithScrollbarHost(
                &view,
                self.last_scroll_update_state_.as_ref(),
                |scrollbar_host| self.scrollbar_controller_.HitTest(scrollbar_host, point),
            )
            .is_some()
        {
            PointerResultType::kScrollbarScroll
        } else {
            PointerResultType::kUnhandled
        }
    }
    // Source: cc/input/input_handler.cc:789-799
    pub fn MouseDown<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        point: PointF,
        shift: bool,
    ) -> InputHandlerPointerResult {
        let view = InputHandlerView {
            pinch_gesture_active_: &self.pinch_gesture_active_,
            external_pinch_gesture_active_: &self.external_pinch_gesture_active_,
            last_scroll_update_state_: &self.last_scroll_update_state_,
            delta_consumed_for_scroll_gesture_: &self.delta_consumed_for_scroll_gesture_,
            snap_animation_data_map_: &self.snap_animation_data_map_,
        };
        if delegate!(
            host,
            ScrollbarAnimationMouseDown(self.scroll_element_id_mouse_currently_over_)
        ) {
            self.scroll_element_id_mouse_currently_captured_ =
                self.scroll_element_id_mouse_currently_over_;
        }
        host.WithScrollbarHost(
            &view,
            self.last_scroll_update_state_.as_ref(),
            |scrollbar_host| {
                self.scrollbar_controller_
                    .HandlePointerDown(scrollbar_host, point, shift)
            },
        )
    }
    // Source: cc/input/input_handler.cc:801-810
    pub fn MouseUp<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        point: PointF,
    ) -> InputHandlerPointerResult {
        let view = InputHandlerView {
            pinch_gesture_active_: &self.pinch_gesture_active_,
            external_pinch_gesture_active_: &self.external_pinch_gesture_active_,
            last_scroll_update_state_: &self.last_scroll_update_state_,
            delta_consumed_for_scroll_gesture_: &self.delta_consumed_for_scroll_gesture_,
            snap_animation_data_map_: &self.snap_animation_data_map_,
        };
        if self.scroll_element_id_mouse_currently_captured_.IsValid() {
            delegate!(
                host,
                ScrollbarAnimationMouseUp(self.scroll_element_id_mouse_currently_captured_)
            );
            self.scroll_element_id_mouse_currently_captured_ = ElementId::default();
        }
        host.WithScrollbarHost(
            &view,
            self.last_scroll_update_state_.as_ref(),
            |scrollbar_host| {
                self.scrollbar_controller_
                    .HandlePointerUp(scrollbar_host, point)
            },
        )
    }
    // Source: cc/input/input_handler.cc:812-815
    pub fn MouseLeave<C: InputHost<B> + ?Sized>(&mut self, host: &mut C) {
        delegate!(host, DidMouseLeave());
        self.scroll_element_id_mouse_currently_over_ = ElementId::default();
    }
    // Source: cc/input/input_handler.cc:817-822
    pub fn FindFrameElementIdAtPoint<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        point: PointF,
    ) -> ElementId {
        delegate!(
            host,
            ActiveTree_FindFrameElementIdAtPoint(
                &self.View(),
                point_scale(point, host.ActiveTree_device_scale_factor(),)
            )
        )
    }
    // Source: cc/input/input_handler.cc:824-826
    pub fn RequestUpdateForSynchronousInputHandler<C: InputHost<B> + ?Sized>(&self, host: &mut C) {
        self.UpdateRootLayerStateForSynchronousInputHandler(host);
    }
    // Source: cc/input/input_handler.cc:828-859
    pub fn SetSynchronousInputHandlerRootScrollOffset<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        offset: PointF,
    ) {
        let _trace = host.BeginTrace(
            "InputHandler::SetSynchronousInputHandlerRootScrollOffset",
            &[
                ("offset_x", InputTraceValue::Float(offset.x() as f64)),
                ("offset_y", InputTraceValue::Float(offset.y() as f64)),
            ],
        );
        let physical = scale(
            offset - host.Viewport_TotalScrollOffset(),
            host.ActiveTree_page_scale_factor_for_scroll(),
        );
        let consumed = host
            .ViewportScrollBy(
                &self.View(),
                physical,
                Point::default(),
                false,
                false,
                true,
                false,
            )
            .consumed_delta;
        if zero(consumed) {
            return;
        }
        delegate!(
            host,
            DidScrollContent(
                &self.View(),
                node_read!(host, self.OuterViewportScrollNode(host).unwrap(), |n| n
                    .element_id),
                false,
                &consumed
            )
        );
        self.SetNeedsCommit(host);
        self.UpdateRootLayerStateForSynchronousInputHandler(host);
    }
    // Source: cc/input/input_handler.cc:861-893
    pub fn PinchGestureBegin<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        anchor: Point,
        source: ScrollInputType,
    ) {
        debug_assert!(source == ScrollInputType::kTouchscreen || source == ScrollInputType::kWheel);
        self.pinch_gesture_active_ = true;
        self.pinch_gesture_end_should_clear_scrolling_node_ =
            self.CurrentlyScrollingNode(host).is_none();
        delegate!(
            host,
            InstantTrace(
                "SetCurrentlyScrollingNode PinchGestureBegin",
                &[(
                    "isNull",
                    InputTraceValue::Bool(self.OuterViewportScrollNode(host).is_none()),
                )],
            )
        );
        if let Some(outer) = self.OuterViewportScrollNode(host) {
            host.SetCurrentlyScrollingNode(outer);
            let mut data = ScrollStateData::default();
            data.position_x = anchor.x();
            data.position_y = anchor.y();
            data.is_beginning = true;
            data.delta_granularity = ScrollGranularity::kScrollByPrecisePixel;
            data.is_direct_manipulation = source == ScrollInputType::kTouchscreen;
            self.DidLatchToScroller(host, &ScrollState::new(data), source);
        }
        delegate!(host, PinchBegin());
        delegate!(host, DidStartPinchZoom());
    }
    // Source: cc/input/input_handler.cc:895-907
    pub fn PinchGestureUpdate<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        magnify: f32,
        anchor: Point,
    ) {
        let _trace = host.BeginTrace("InputHandler::PinchGestureUpdate", &[]);
        if self.InnerViewportScrollNode(host).is_none() {
            return;
        }
        self.has_pinch_zoomed_ = true;
        host.ViewportPinchUpdate(&self.View(), magnify, anchor);
        self.SetNeedsCommit(host);
        delegate!(host, DidUpdatePinchZoom());
        self.UpdateRootLayerStateForSynchronousInputHandler(host);
    }
    // Source: cc/input/input_handler.cc:909-924
    pub fn PinchGestureEnd<C: InputHost<B> + ?Sized>(&mut self, host: &mut C, anchor: Point) {
        debug_assert!(
            self.latched_scroll_type_.is_some() || self.CurrentlyScrollingNode(host).is_none()
        );
        let snap_to_min = self.latched_scroll_type_.is_some()
            && self.latched_scroll_type_ == Some(ScrollInputType::kWheel);
        self.pinch_gesture_active_ = false;
        if self.pinch_gesture_end_should_clear_scrolling_node_ {
            self.pinch_gesture_end_should_clear_scrolling_node_ = false;
            self.ClearCurrentlyScrollingNode(host);
        }
        host.ViewportPinchEnd(&self.View(), anchor, snap_to_min);
        delegate!(host, PinchEnd());
        self.SetNeedsCommit(host);
        delegate!(host, DidEndPinchZoom());
    }
    // Source: cc/input/input_handler.cc:926-928
    pub fn SetNeedsAnimateInput<C: InputHost<B> + ?Sized>(&self, host: &mut C) {
        delegate!(host, SetNeedsAnimateInput());
    }
    // Source: cc/input/input_handler.cc:930-945
    pub fn IsCurrentlyScrollingViewport<C: InputHost<B> + ?Sized>(&self, host: &mut C) -> bool {
        if let Some(n) = self.CurrentlyScrollingNode(host) {
            if host.Viewport_ShouldScroll(&n) {
                return true;
            }
        }
        if let Some(outer) = self.OuterViewportScrollNode(host) {
            return self.IsAnimatingForSnap(node_read!(host, outer, |n| n.element_id));
        }
        false
    }
    // Source: cc/input/input_handler.cc:947-950
    pub fn GetEventListenerProperties<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        class: EventListenerClass,
    ) -> EventListenerProperties {
        host.ActiveTree_event_listener_properties(class)
    }
    // Source: cc/input/input_handler.cc:952-962
    pub fn HasBlockingWheelEventHandlerAt<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        point: Point,
    ) -> bool {
        delegate!(
            host,
            ActiveTree_FindLayerThatIsHitByPointInWheelEventHandlerRegion(
                &self.View(),
                point_scale(vector(point), host.ActiveTree_device_scale_factor(),)
            )
        )
        .is_some()
    }
    // Source: cc/input/input_handler.cc:965-1049
    pub fn EventListenerTypeForTouchStartOrMoveAt<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        rect: Rect,
        action: Option<&mut TouchAction>,
    ) -> TouchStartOrMoveEventListenerType {
        let r = RectF::from(rect);
        let dsf = host.ActiveTree_device_scale_factor();
        let device = RectF::new(r.x() * dsf, r.y() * dsf, r.width() * dsf, r.height() * dsf);
        let layer =
            host.ActiveTree_FindLayerThatIsHitByPointInTouchHandlerRegion(&self.View(), device);
        let Some(layer) = layer else {
            if let Some(action) = action {
                *action = TouchAction::kAuto;
            }
            return TouchStartOrMoveEventListenerType::kNoHandler;
        };
        if let Some(action) = action {
            let transform = host.LayerScreenSpaceTransform(&layer);
            let inverse = transform.GetCheckedInverse();
            let mut clipped = false;
            let hit =
                MathUtil::MapQuad(&inverse, QuadF::FromRect(device), &mut clipped).BoundingBox();
            let rect = Rect::new(
                hit.x().round() as i32,
                hit.y().round() as i32,
                hit.width().round() as i32,
                hit.height().round() as i32,
            );
            *action = host.LayerGetAllowedTouchAction(&layer, rect);
        }
        if !self.IsCurrentlyScrolling(host) {
            return TouchStartOrMoveEventListenerType::kHandler;
        }
        let hit_layer = host.ActiveTree_FindLayerThatIsHitByPoint(
            &self.View(),
            PointF::new(
                device.x() + device.width() / 2.,
                device.y() + device.height() / 2.,
            ),
        );
        let currently_scrolled = self
            .CurrentlyScrollingNode(host)
            .is_some_and(|node| self.IsScrolledBy(host, hit_layer.as_ref(), &node));
        if currently_scrolled {
            return TouchStartOrMoveEventListenerType::kHandlerOnScrollingLayer;
        } else if !self.snap_animation_data_map_.is_empty() {
            for (id, data) in &self.snap_animation_data_map_ {
                if data.animating_snap_target_ids_ == TargetSnapAreaElementIds::default() {
                    continue;
                }
                if let Some(node) = host.ScrollTree_MutableFindNodeFromElementId(*id) {
                    if self.IsScrolledBy(host, hit_layer.as_ref(), &node) {
                        return TouchStartOrMoveEventListenerType::kHandlerOnScrollingLayer;
                    }
                }
            }
        }
        TouchStartOrMoveEventListenerType::kHandler
    }
    // Source: cc/input/input_handler.cc:1052-1054
    pub fn CreateLatencyInfoSwapPromiseMonitor<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        latency: Option<&mut B::LatencyInfo>,
    ) -> Option<Box<B::LatencyInfoSwapPromiseMonitor>> {
        delegate!(host, CreateLatencyInfoSwapPromiseMonitor(latency))
    }
    // Source: cc/input/input_handler.cc:1057-1061
    pub fn GetScopedEventMetricsMonitor<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        callback: B::EventMetricsDoneCallback,
    ) -> Option<Box<B::EventMetricsScopedMonitor>> {
        delegate!(host, GetScopedEventMetricsMonitor(callback))
    }
    // Source: cc/input/input_handler.cc:1063-1073
    pub fn CreateScrollElasticityHelper<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
    ) -> Option<&mut B::Elasticity> {
        debug_assert!(self.scroll_elasticity_helper_.is_none());
        if setting!(host, enable_elastic_overscroll_for_subscroll)
            || setting!(host, enable_elastic_overscroll_on_root)
        {
            self.scroll_elasticity_helper_ = Some(host.CreateScrollElasticityHelper());
        }
        self.scroll_elasticity_helper_.as_mut()
    }
    // Source: cc/input/input_handler.cc:1075-1079
    pub fn DestroyScrollElasticityHelper<C: InputHost<B> + ?Sized>(&mut self, host: &mut C) {
        host.ResetStretchAmounts(self.scroll_elasticity_helper_.as_mut().unwrap());
        self.scroll_elasticity_helper_ = None;
    }
    // Source: cc/input/input_handler.cc:1081-1090
    pub fn GetScrollOffsetForLayer<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        id: ElementId,
        offset: &mut PointF,
    ) -> bool {
        if host.ScrollTree_MutableFindNodeFromElementId(id).is_none() {
            return false;
        }
        *offset = host.ScrollTree_current_scroll_offset(id);
        true
    }
    // Source: cc/input/input_handler.cc:1092-1104
    pub fn ScrollLayerTo<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        id: ElementId,
        offset: PointF,
    ) -> bool {
        let Some(node) = host.ScrollTree_MutableFindNodeFromElementId(id) else {
            return false;
        };
        let delta = offset - host.ScrollTree_current_scroll_offset(id);
        host.ScrollBy(&node, delta);
        true
    }
    // Source: cc/input/input_handler.cc:1106-1141
    fn ConstrainFling(&self, current: PointF, proposed: PointF) -> Option<PointF> {
        let mut constrained = proposed;
        if let Some(range) = self.fling_snap_constrain_x_ {
            let min = range.GetMin();
            let max = range.GetMax();
            let curr = current.x();
            let mut prop = proposed.x();
            if curr <= max {
                prop = cpp_min(prop, max);
            }
            if curr >= min {
                prop = cpp_max(prop, min);
            }
            constrained.set_x(prop);
        }
        if let Some(range) = self.fling_snap_constrain_y_ {
            let min = range.GetMin();
            let max = range.GetMax();
            let curr = current.y();
            let mut prop = proposed.y();
            if curr <= max {
                prop = cpp_min(prop, max);
            }
            if curr >= min {
                prop = cpp_max(prop, min);
            }
            constrained.set_y(prop);
        }
        if proposed == constrained {
            None
        } else {
            Some(constrained)
        }
    }
    // Source: cc/input/input_handler.cc:1143-1159
    fn PredictViewportBoundsDelta<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        distance: Vector2dF,
    ) -> f64 {
        let current = host.ScrollTree_outer_viewport_container_bounds_delta().y() as f64;
        delegate!(host, PredictViewportBoundsDelta(current, distance))
    }
    // Source: cc/input/input_handler.cc:1161-1270
    pub fn GetSnapFlingInfoAndSetAnimatingSnapTarget<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        current_delta: Vector2dF,
        natural: Vector2dF,
        initial: &mut PointF,
        target: &mut PointF,
    ) -> bool {
        let Some(node) = self.CurrentlyScrollingNode(host) else {
            return false;
        };
        if !node_read!(host, node, |n| n.snap_container_data.is_some())
            || self.snap_fling_state_ == SnapFlingState::kNativeFling
        {
            return false;
        }
        let factor = host.ActiveTree_page_scale_factor_for_scroll();
        let content_delta = scale(current_delta, 1. / factor);
        let mut displacement = scale(natural, 1. / factor);
        let current = self.GetVisualScrollOffset(host, &node);
        let new = current + content_delta;
        if self.snap_fling_state_ == SnapFlingState::kConstrainedNativeFling {
            if let Some(constrained) = self.ConstrainFling(current, new) {
                displacement = constrained - current;
            }
        }
        let mut strategy = <dyn SnapSelectionStrategy>::CreateForDisplacement(
            current,
            displacement,
            true,
            SnapStopAlwaysFilter::kIgnore,
        );
        let adjustment = if node_read!(host, node, |n| n.scrolls_outer_viewport) {
            self.PredictViewportBoundsDelta(host, displacement)
        } else {
            0.
        };
        let mut snap = node_write!(host, node, |n| {
            n.snap_container_data
                .as_mut()
                .unwrap()
                .FindSnapPositionWithViewportAdjustment(strategy.as_ref(), adjustment)
        });
        if snap.r#type == SnapPositionType::kNone {
            self.snap_fling_state_ = SnapFlingState::kNativeFling;
            return false;
        }
        if snap.is_extremity && host.feature(InputFeature::SnapFlingNearExtremes) {
            let distance_x = (snap.position.x() - current.x()).abs() as f64;
            let distance_y = (snap.position.y() - current.y()).abs() as f64;
            let displacement_x = displacement.x().abs() as f64;
            let displacement_y = displacement.y().abs() as f64;
            let mut fast = true;
            if strategy.ShouldSnapOnX() && displacement_x < distance_x {
                fast = false;
            }
            if strategy.ShouldSnapOnY() && displacement_y < distance_y {
                fast = false;
            }
            if fast {
                self.fling_snap_constrain_x_ = Some(
                    snap.covered_range_x
                        .unwrap_or(RangeF::At(snap.position.x())),
                );
                self.fling_snap_constrain_y_ = Some(
                    snap.covered_range_y
                        .unwrap_or(RangeF::At(snap.position.y())),
                );
                self.snap_fling_state_ = SnapFlingState::kConstrainedNativeFling;
                return false;
            }
        } else {
            if self.snap_fling_state_ == SnapFlingState::kConstrainedNativeFling {
                if let Some(constrained) = self.ConstrainFling(current, new) {
                    displacement = constrained - current;
                    strategy = <dyn SnapSelectionStrategy>::CreateForDisplacement(
                        current,
                        displacement,
                        true,
                        SnapStopAlwaysFilter::kIgnore,
                    );
                    snap = node_write!(host, node, |n| {
                        n.snap_container_data
                            .as_mut()
                            .unwrap()
                            .FindSnapPositionWithViewportAdjustment(strategy.as_ref(), adjustment)
                    });
                }
            }
            if self.snap_fling_state_ == SnapFlingState::kNoFling
                && snap.r#type == SnapPositionType::kCovered
            {
                if self.ConstrainFling(current, new).is_none() {
                    self.fling_snap_constrain_x_ = snap.covered_range_x;
                    self.fling_snap_constrain_y_ = snap.covered_range_y;
                    self.snap_fling_state_ = SnapFlingState::kConstrainedNativeFling;
                    return false;
                }
            }
        }
        *initial = current;
        *target = snap.position;
        target.Scale(factor);
        initial.Scale(factor);
        let id = node_read!(host, node, |n| n.element_id);
        self.EnsureSnapAnimationData(id);
        self.SetAnimatingSnapTargetsForElement(id, snap.target_element_ids);
        self.snap_fling_state_ = SnapFlingState::kSnapFling;
        true
    }
    // Source: cc/input/input_handler.cc:1272-1291
    pub fn ScrollEndForSnapFling<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        finished: bool,
    ) {
        let node = self.CurrentlyScrollingNode(host);
        if finished {
            if let Some(node) = &node {
                if node_read!(host, node, |n| n.snap_container_data.is_some()) {
                    let id = node_read!(host, node, |n| n.element_id);
                    let targets = self.GetAnimatingSnapTargetsForElement(id);
                    node_write!(host, node, |n| {
                        n.snap_container_data
                            .as_mut()
                            .unwrap()
                            .SetTargetSnapAreaElementIds(targets);
                    });
                    self.updated_snapped_elements_.insert(id, targets);
                    self.SetNeedsCommit(host);
                }
            }
        }
        if let Some(n) = node {
            self.ClearAnimatingSnapTargetsForElement(node_read!(host, n, |n| n.element_id));
        }
        self.ScrollEnd(host, true, None);
    }
    // Source: cc/input/input_handler.cc:1293-1295
    pub fn NotifyInputEvent<C: InputHost<B> + ?Sized>(&self, host: &mut C, fling: bool) {
        delegate!(host, NotifyInputEvent(fling));
    }
    // Source: cc/input/input_handler.cc:1297-1308
    fn UpdateLastLatchedScrollSourceType(&mut self) {
        if self.has_scrolled_by_wheel_
            || self.has_scrolled_by_touch_
            || self.has_scrolled_by_precisiontouchpad_
            || self.has_scrolled_by_scrollbar_
            || self.has_pinch_zoomed_
        {
            self.last_latched_scroll_source_type_ = ScrollSourceType::kRelativeScroll;
            return;
        }
        self.last_latched_scroll_source_type_ = ScrollSourceType::kNone;
    }
    // Source: cc/input/input_handler.cc:1314-1381
    pub fn ProcessCommitDeltas<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        data: &mut CompositorCommitData,
        mutator: Option<&B::MutatorHost>,
        next_bmf: bool,
    ) {
        if host.ActiveTree_LayerListIsEmpty() {
            return;
        }
        let inner = self
            .InnerViewportScrollNode(host)
            .map(|n| node_read!(host, n, |n| n.element_id))
            .unwrap_or_default();
        let snapped = std::mem::take(&mut self.updated_snapped_elements_);
        delegate!(
            host,
            CollectScrollDeltas(
                &mut *data,
                inner,
                setting!(host, commit_fractional_scroll_deltas),
                snapped,
                mutator,
                next_bmf,
            )
        );
        data.scroll_type = self.last_latched_scroll_source_type_;
        debug_assert_eq!(data.manipulation_info, 0);
        if self.has_scrolled_by_wheel_ {
            data.manipulation_info |= kManipulationInfoWheel;
        }
        if self.has_scrolled_by_touch_ {
            data.manipulation_info |= kManipulationInfoTouch;
        }
        if self.has_scrolled_by_precisiontouchpad_ {
            data.manipulation_info |= kManipulationInfoPrecisionTouchPad;
        }
        if self.has_pinch_zoomed_ {
            data.manipulation_info |= kManipulationInfoPinchZoom;
        }
        if self.has_scrolled_by_scrollbar_ {
            data.manipulation_info |= kManipulationInfoScrollbar;
        }
        self.has_scrolled_by_wheel_ = false;
        self.has_scrolled_by_touch_ = false;
        self.has_scrolled_by_precisiontouchpad_ = false;
        self.has_pinch_zoomed_ = false;
        self.has_scrolled_by_scrollbar_ = false;
        data.overscroll_delta = self.overscroll_delta_for_main_thread_;
        self.overscroll_delta_for_main_thread_ = Vector2dF::default();
        if let Some(strategy) = &self.snap_strategy_ {
            data.snap_strategy = Some(strategy.Clone());
        }
        data.scroll_latched_element_id = self.last_latched_scroller_;
        data.scroll_end_data.done_containers =
            std::mem::take(&mut self.pending_scrollend_containers_);
        if data
            .scroll_end_data
            .done_containers
            .contains(&self.last_latched_scroller_)
        {
            self.last_latched_scroller_ = ElementId::default();
            self.last_latched_scroll_source_type_ = ScrollSourceType::kNone;
        }
    }
    // Source: cc/input/input_handler.cc:1383-1390
    pub fn TickAnimations(&self, time: TimeTicks) {
        if let Some(client) = &self.input_handler_client_ {
            client.Animate(time);
        }
    }
    // Source: cc/input/input_handler.cc:1392-1399
    pub fn WillShutdown(&mut self) {
        if let Some(client) = self.input_handler_client_.take() {
            client.WillShutdown();
        }
        if self.scroll_elasticity_helper_.is_some() {
            self.scroll_elasticity_helper_ = None;
        }
    }
    // Source: cc/input/input_handler.cc:1401-1404
    pub fn WillDraw(&self) {
        if let Some(client) = &self.input_handler_client_ {
            client.ReconcileElasticOverscrollAndRootScroll();
        }
    }
    // Source: cc/input/input_handler.cc:1406-1411
    pub fn WillBeginImplFrame<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        args: &BeginFrameArgs,
    ) {
        let view = InputHandlerView {
            pinch_gesture_active_: &self.pinch_gesture_active_,
            external_pinch_gesture_active_: &self.external_pinch_gesture_active_,
            last_scroll_update_state_: &self.last_scroll_update_state_,
            delta_consumed_for_scroll_gesture_: &self.delta_consumed_for_scroll_gesture_,
            snap_animation_data_map_: &self.snap_animation_data_map_,
        };
        if let Some(client) = &self.input_handler_client_ {
            host.WithScrollbarHost(
                &view,
                self.last_scroll_update_state_.as_ref(),
                |scrollbar_host| {
                    self.scrollbar_controller_
                        .WillBeginImplFrame(scrollbar_host)
                },
            );
            client.DeliverInputForBeginFrame(args);
        }
    }
    // Source: cc/input/input_handler.cc:1413-1423
    pub fn DidCommit<C: InputHost<B> + ?Sized>(&mut self, host: &mut C) {
        if let Some(client) = &self.input_handler_client_ {
            if delegate!(host, IsInHighLatencyMode()) {
                client.DeliverInputForHighLatencyMode();
            }
        }
        if let Some(elastic) = &mut self.scroll_elasticity_helper_ {
            host.ApplyStretchAmountsToPending(elastic);
        }
    }
    // Source: cc/input/input_handler.cc:1425-1430
    pub fn DidImplSideInvalidate<C: InputHost<B> + ?Sized>(&mut self, host: &mut C) {
        if let Some(elastic) = &mut self.scroll_elasticity_helper_ {
            host.ApplyStretchAmountsToPending(elastic);
        }
    }
    // Source: cc/input/input_handler.cc:1432-1446
    pub fn DidActivatePendingTree<C: InputHost<B> + ?Sized>(&mut self, host: &mut C) {
        if self.CurrentlyScrollingNode(host).is_none() {
            self.ClearCurrentlyScrollingNode(host);
        }
        if let Some(elastic) = &mut self.scroll_elasticity_helper_ {
            host.ApplyStretchAmountsToActive(elastic);
        }
        self.UpdateRootLayerStateForSynchronousInputHandler(host);
    }
    // Source: cc/input/input_handler.cc:1448-1452
    pub fn DidFinishImplFrame(&self) {
        if let Some(client) = &self.input_handler_client_ {
            client.DidFinishImplFrame();
        }
    }
    // Source: cc/input/input_handler.cc:1454-1461
    pub fn OnBeginImplFrameDeadline<C: InputHost<B> + ?Sized>(&self, host: &mut C) {
        if !self.IsCurrentlyScrolling(host) {
            return;
        }
        if let Some(client) = &self.input_handler_client_ {
            client.DeliverInputForDeadline();
        }
    }
    // Source: cc/input/input_handler.cc:1463-1465
    pub fn RootLayerStateMayHaveChanged<C: InputHost<B> + ?Sized>(&self, host: &mut C) {
        self.UpdateRootLayerStateForSynchronousInputHandler(host);
    }
    // Source: cc/input/input_handler.cc:1467-1470
    pub fn DidRegisterScrollbar<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        id: ElementId,
        orientation: ScrollbarOrientation,
    ) {
        let view = InputHandlerView {
            pinch_gesture_active_: &self.pinch_gesture_active_,
            external_pinch_gesture_active_: &self.external_pinch_gesture_active_,
            last_scroll_update_state_: &self.last_scroll_update_state_,
            delta_consumed_for_scroll_gesture_: &self.delta_consumed_for_scroll_gesture_,
            snap_animation_data_map_: &self.snap_animation_data_map_,
        };
        host.WithScrollbarHost(
            &view,
            self.last_scroll_update_state_.as_ref(),
            |scrollbar_host| {
                self.scrollbar_controller_
                    .DidRegisterScrollbar(scrollbar_host, id, orientation)
            },
        );
    }
    // Source: cc/input/input_handler.cc:1472-1475
    pub fn DidUnregisterScrollbar<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        id: ElementId,
        orientation: ScrollbarOrientation,
    ) {
        let view = InputHandlerView {
            pinch_gesture_active_: &self.pinch_gesture_active_,
            external_pinch_gesture_active_: &self.external_pinch_gesture_active_,
            last_scroll_update_state_: &self.last_scroll_update_state_,
            delta_consumed_for_scroll_gesture_: &self.delta_consumed_for_scroll_gesture_,
            snap_animation_data_map_: &self.snap_animation_data_map_,
        };
        host.WithScrollbarHost(
            &view,
            self.last_scroll_update_state_.as_ref(),
            |scrollbar_host| {
                self.scrollbar_controller_
                    .DidUnregisterScrollbar(scrollbar_host, id, orientation)
            },
        );
    }
    // Source: cc/input/input_handler.cc:1467-1470.
    // Narrow registration path invokes the same canonical controller body and
    // never requires an unrelated viewport or a full InputHost facade.
    pub fn DidRegisterScrollbarWithHost<
        C: crate::adapter::scrollbar::ScrollbarRegistrationHost + ?Sized,
    >(
        &mut self,
        host: &mut C,
        id: ElementId,
        orientation: ScrollbarOrientation,
    ) {
        self.scrollbar_controller_
            .DidRegisterScrollbar(host, id, orientation);
    }
    // Source: cc/input/input_handler.cc:1472-1475.
    pub fn DidUnregisterScrollbarWithHost<
        C: crate::adapter::scrollbar::ScrollbarRegistrationHost + ?Sized,
    >(
        &mut self,
        host: &mut C,
        id: ElementId,
        orientation: ScrollbarOrientation,
    ) {
        self.scrollbar_controller_
            .DidUnregisterScrollbar(host, id, orientation);
    }
    // Source: cc/input/input_handler.cc:1477-1540
    pub fn ScrollOffsetAnimationFinished<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        id: ElementId,
    ) {
        let _trace = host.BeginTrace("InputHandler::ScrollOffsetAnimationFinished", &[]);
        let mut finished = host.ScrollTree_MutableFindNodeFromElementId(id);
        let inner = self.InnerViewportScrollNode(host).is_some()
            && finished == self.InnerViewportScrollNode(host);
        if inner {
            finished = self.OuterViewportScrollNode(host);
        }
        let Some(finished) = finished else {
            return;
        };
        let finished_id = node_read!(host, finished, |n| n.element_id);
        let was_snap = self.IsAnimatingForSnap(finished_id);
        let latched = self.CurrentlyScrollingNode(host);
        debug_assert!(Some(finished.clone()) == latched || was_snap);
        if was_snap {
            if node_read!(host, finished, |n| n.snap_container_data.is_some()) {
                let targets = self.GetAnimatingSnapTargetsForElement(finished_id);
                node_write!(host, finished, |n| {
                    n.snap_container_data
                        .as_mut()
                        .unwrap()
                        .SetTargetSnapAreaElementIds(targets);
                });
                self.updated_snapped_elements_.insert(finished_id, targets);
                self.SetNeedsCommit(host);
            }
            self.ClearAnimatingSnapTargetsForElement(finished_id);
            if latched != Some(finished.clone()) {
                self.ScrollEndForNode(host, Some(finished), false, None);
                return;
            }
        } else if self.SnapAtScrollEnd(host, SnapReason::kScrollOffsetAnimationFinished) {
            return;
        }
        self.ClearAnimatingSnapTargetsForElement(finished_id);
        if self.deferred_scroll_ends_.contains(&finished_id) {
            self.ScrollEnd(host, false, None);
            return;
        }
    }
    // Source: cc/input/input_handler.cc:1542-1586
    pub fn ElasticOverscrollAnimationFinished<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        finished_id: ElementId,
    ) {
        if let Some(current) = self.CurrentlyScrollingNode(host) {
            let mut targets = false;
            if node_read!(host, current, |n| n.scrolls_inner_viewport
                || n.scrolls_outer_viewport)
            {
                if self
                    .InnerViewportScrollNode(host)
                    .is_some_and(|n| node_read!(host, n, |n| n.element_id) == finished_id)
                    || self
                        .OuterViewportScrollNode(host)
                        .is_some_and(|n| node_read!(host, n, |n| n.element_id) == finished_id)
                {
                    targets = true;
                }
            } else {
                targets = node_read!(host, current, |n| n.element_id) == finished_id;
            }
            if targets {
                if !self.IsAnimatingForSnap(node_read!(host, current, |n| n.element_id)) {
                    self.ScrollEnd(host, true, None);
                }
                return;
            }
        }
        if self.IsAnimatingForSnap(finished_id) {
            return;
        }
        if self.deferred_scroll_ends_.contains(&finished_id) {
            let mut finished = host.ScrollTree_MutableFindNodeFromElementId(finished_id);
            if self.InnerViewportScrollNode(host).is_some()
                && finished == self.InnerViewportScrollNode(host)
            {
                finished = self.OuterViewportScrollNode(host);
            }
            self.ScrollEndForNode(host, finished, false, None);
        }
    }
    // Source: cc/input/input_handler.cc:1588-1595
    pub fn SetPrefersReducedMotion(&mut self, reduced: bool) {
        if self.prefers_reduced_motion_ == reduced {
            return;
        }
        self.prefers_reduced_motion_ = reduced;
        if let Some(client) = &self.input_handler_client_ {
            client.SetPrefersReducedMotion(self.prefers_reduced_motion_);
        }
    }
    // Source: cc/input/input_handler.cc:1597-1614
    pub fn IsCurrentlyScrolling<C: InputHost<B> + ?Sized>(&self, host: &mut C) -> bool {
        let has_current = self.CurrentlyScrollingNode(host).is_some();
        self.View().IsCurrentlyScrollingWithCurrentNode(has_current)
    }
    // Source: cc/input/input_handler.cc:1616-1637; cc/input/input_handler.h:818,849,874.
    // Rust borrowed output only; the algorithm remains in its single source body.
    pub fn View(&self) -> InputHandlerView<'_> {
        InputHandlerView {
            pinch_gesture_active_: &self.pinch_gesture_active_,
            external_pinch_gesture_active_: &self.external_pinch_gesture_active_,
            last_scroll_update_state_: &self.last_scroll_update_state_,
            delta_consumed_for_scroll_gesture_: &self.delta_consumed_for_scroll_gesture_,
            snap_animation_data_map_: &self.snap_animation_data_map_,
        }
    }
    /// Synchronous borrow projection of the actual controller and the five
    /// readonly source fields. It stores neither a callback nor a view.
    pub fn WithRegistrationAndView<Output>(
        &mut self,
        callback: impl FnOnce(&mut ScrollbarController, InputHandlerView<'_>) -> Output,
    ) -> Output {
        let view = InputHandlerView {
            pinch_gesture_active_: &self.pinch_gesture_active_,
            external_pinch_gesture_active_: &self.external_pinch_gesture_active_,
            last_scroll_update_state_: &self.last_scroll_update_state_,
            delta_consumed_for_scroll_gesture_: &self.delta_consumed_for_scroll_gesture_,
            snap_animation_data_map_: &self.snap_animation_data_map_,
        };
        callback(&mut self.scrollbar_controller_, view)
    }
    // Source: cc/input/input_handler.cc:1616-1637
    pub fn GetActivelyScrollingType<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
    ) -> ActivelyScrollingType {
        let currently_scrolling_node = self.CurrentlyScrollingNode(host).is_some();
        self.View()
            .GetActivelyScrollingTypeWithCurrentNode(currently_scrolling_node, || {
                host.GetSettings()
            })
    }
    // Source: cc/input/input_handler.cc:1639-1641
    pub fn IsHandlingTouchSequence(&self) -> bool {
        self.is_handling_touch_sequence_
    }
    // Source: cc/input/input_handler.cc:1643-1668
    pub fn IsCurrentScrollMainRepainted<C: InputHost<B> + ?Sized>(&self, host: &mut C) -> bool {
        if let Some(node) = self.CurrentlyScrollingNode(host) {
            if !node_read!(host, node, |n| n.main_thread_repaint_reasons.empty()) {
                return true;
            }
        }
        for (id, data) in &self.snap_animation_data_map_ {
            if data.animating_snap_target_ids_ == TargetSnapAreaElementIds::default() {
                continue;
            }
            if let Some(node) = host.ScrollTree_MutableFindNodeFromElementId(*id) {
                if !node_read!(host, node, |n| n.main_thread_repaint_reasons.empty()) {
                    return true;
                }
            }
        }
        false
    }
    // Source: cc/input/input_handler.cc:1670-1675
    pub fn HasQueuedInput(&self) -> bool {
        if let Some(client) = &self.input_handler_client_ {
            return client.HasQueuedInput();
        }
        false
    }
    // Source: cc/input/input_handler.cc:1677-1683
    fn CurrentlyScrollingNode<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
    ) -> Option<B::ScrollNode> {
        host.ScrollTree_CurrentlyScrollingNode()
    }
    // Source: cc/input/input_handler.cc:1685-1691
    fn GetScrollTree<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
    ) -> crate::adapter::layer_tree_impl::LayerTreeId {
        host.ActiveTreeId()
    }
    // Source: cc/input/input_handler.cc:1693-1695
    fn InnerViewportScrollNode<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
    ) -> Option<B::ScrollNode> {
        host.ActiveTree_InnerViewportScrollNode()
    }
    // Source: cc/input/input_handler.cc:1697-1699
    fn OuterViewportScrollNode<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
    ) -> Option<B::ScrollNode> {
        host.ActiveTree_OuterViewportScrollNode()
    }
    // Source: cc/input/input_handler.cc:1701-1703
    fn GetViewport<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
    ) -> crate::adapter::layer_tree_impl::LayerTreeId {
        host.ActiveTreeId()
    }
    // Source: cc/input/input_handler.cc:1705-1707
    fn SetNeedsCommit<C: InputHost<B> + ?Sized>(&self, host: &mut C) {
        let reason = host.ScrollBeginMainFrameReason();
        delegate!(host, SetNeedsCommit(reason));
    }
    // Source: cc/input/input_handler.cc:1709-1717
    fn ActiveTree<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
    ) -> crate::adapter::layer_tree_impl::LayerTreeId {
        host.ActiveTreeId()
    }
    // Source: cc/input/input_handler.cc:1719-1731
    fn GetTrackerTypeForScroll(&self, input: ScrollInputType) -> ScrollTrackingType {
        match input {
            ScrollInputType::kWheel => ScrollTrackingType::kWheelScroll,
            ScrollInputType::kTouchscreen => ScrollTrackingType::kTouchScroll,
            ScrollInputType::kScrollbar => ScrollTrackingType::kScrollbarScroll,
            ScrollInputType::kAutoscroll => ScrollTrackingType::kMaxType,
        }
    }
    // Source: cc/input/input_handler.cc:1733-1747
    pub fn PageSize<C: InputHost<B> + ?Sized>(&self, host: &mut C, node: &B::ScrollNode) -> Size {
        let size = node_read!(host, node, |n| n.container_bounds);
        let viewport = delegate!(host, VisualDeviceViewportSize());
        let page_scale = delegate!(host, PageScaleFactor());
        let dsf = host.ActiveTree_device_scale_factor();
        Size::new(
            cpp_min(
                size.width() as f32 * page_scale,
                viewport.width() as f32 / dsf,
            ) as i32,
            cpp_min(
                size.height() as f32 * page_scale,
                viewport.height() as f32 / dsf,
            ) as i32,
        )
    }
    // Source: cc/input/input_handler.cc:1749-1751
    pub fn LineStep<C: InputHost<B> + ?Sized>(&self, host: &mut C) -> f32 {
        Self::LineStepForHost(host)
    }
    // Source: cc/input/input_handler.cc:1749-1751.
    // Rust split-borrow entry for this same single source algorithm.
    pub fn LineStepForHost<C: InputHost<B> + ?Sized>(host: &mut C) -> f32 {
        kPixelsPerLineStep as f32 * host.ActiveTree_painted_device_scale_factor()
    }
    // Source: cc/input/input_handler.cc:1753-1780
    fn LimitDeltaToScrollerSize<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        state: &ScrollState,
        node: &B::ScrollNode,
        delta: &mut Vector2dF,
    ) {
        Self::LimitDeltaToScrollerSizeForNode(host, state, node, delta);
    }
    // Source: cc/input/input_handler.cc:1753-1780
    fn LimitDeltaToScrollerSizeForNode<C: InputHost<B> + ?Sized>(
        host: &mut C,
        state: &ScrollState,
        node: &B::ScrollNode,
        delta: &mut Vector2dF,
    ) {
        if state.is_direct_manipulation()
            || state.is_in_inertial_phase()
            || state.is_scrollbar_interaction()
            || state.delta_granularity() == ScrollGranularity::kScrollByPage
            || state.delta_granularity() == ScrollGranularity::kScrollByDocument
            || !host.feature(InputFeature::LimitScrollDeltaToScrollerSize)
        {
            return;
        }
        let size = node_read!(host, node, |n| n.container_bounds);
        let sx = if delta.x().is_sign_negative() {
            -1.
        } else {
            1.
        };
        let sy = if delta.y().is_sign_negative() {
            -1.
        } else {
            1.
        };
        let x = cpp_min(delta.x().abs(), size.width() as f32);
        let y = cpp_min(delta.y().abs(), size.height() as f32);
        delta.set_x(x.copysign(sx));
        delta.set_y(y.copysign(sy));
    }
    // Source: cc/input/input_handler.cc:1785-1813
    pub fn ResolveScrollGranularityToPixels<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        node: &B::ScrollNode,
        delta: Vector2dF,
        granularity: ScrollGranularity,
    ) -> Vector2dF {
        Self::ResolveScrollGranularityToPixelsForNode(host, node, delta, granularity)
    }
    // Source: cc/input/input_handler.cc:1785-1813.
    // Rust split-borrow entry for this same single source algorithm.
    pub fn ResolveScrollGranularityToPixelsForNode<C: InputHost<B> + ?Sized>(
        host: &mut C,
        node: &B::ScrollNode,
        delta: Vector2dF,
        granularity: ScrollGranularity,
    ) -> Vector2dF {
        let mut pixels = delta;
        if granularity == ScrollGranularity::kScrollByPage {
            let size = node_read!(host, node, |n| n.container_bounds);
            let page_scale = delegate!(host, PageScaleFactor());
            let scroller = SizeF::new(
                size.width() as f32 * page_scale,
                size.height() as f32 * page_scale,
            );
            let viewport = delegate!(host, VisualDeviceViewportSize());
            let dsf = host.ActiveTree_device_scale_factor();
            let viewport = SizeF::new(
                viewport.width() as f32 / dsf,
                viewport.height() as f32 / dsf,
            );
            pixels = scale(pixels, kMinFractionToStepWhenPaging);
            pixels = ScrollUtils::ResolveScrollPercentageToPixels(&pixels, &scroller, &viewport);
        }
        if granularity == ScrollGranularity::kScrollByLine {
            pixels = scale(pixels, Self::LineStepForHost(host));
        }
        pixels
    }
    // Source: cc/input/input_handler.cc:1815-1885
    fn HitTestScrollNode<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        point: PointF,
    ) -> ScrollHitTestResult<B::ScrollNode> {
        let mut result = ScrollHitTestResult {
            scroll_node: None,
            hit_test_successful: false,
            main_thread_hit_test_reasons: MainThreadHitTestReasons::default(),
        };
        let layers =
            host.ActiveTree_FindLayersUpToFirstScrollableOrOpaqueToHitTest(&self.View(), point);
        let mut first = None;
        if let Some(last) = layers.last() {
            if setting!(host, enable_hit_test_opaqueness) {
                if host.LayerOpaqueToHitTest(&last) {
                    first = Some(last);
                }
            } else if host.LayerIsScrollerOrScrollbar(&last) {
                first = Some(last);
            }
        }
        let mut node = None;
        for layer in &layers {
            if !self.IsInitialScrollHitTestReliable(host, layer, first, &mut node) {
                host.InstantTrace("Failed Hit Test", &[]);
                result
                    .main_thread_hit_test_reasons
                    .Put(MainThreadHitTestReason::kFailedHitTest);
                return result;
            }
            if host.ActiveTree_PointHitsMainThreadScrollHitTestRegion(point, layer) {
                result
                    .main_thread_hit_test_reasons
                    .Put(MainThreadHitTestReason::kMainThreadScrollHitTestRegion);
                return result;
            }
            let id = host.ActiveTree_PointHitsNonCompositedScroll(point, layer);
            if id.IsValid() {
                node = host.ScrollTree_MutableFindNodeFromElementId(id);
                assert!(node.is_some());
                break;
            }
        }
        if node.is_none() {
            result.hit_test_successful = true;
            if let Some(inner) = self.InnerViewportScrollNode(host) {
                result.scroll_node = self.GetNodeToScroll(host, inner);
            }
            return result;
        }
        result.scroll_node = node;
        result.hit_test_successful = true;
        result
    }
    // Source: cc/input/input_handler.cc:1887-1923
    fn GetNodeToScroll<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        node: B::ScrollNode,
    ) -> Option<B::ScrollNode> {
        if node_read!(host, node, |n| n.id) <= kSecondaryRootPropertyNodeId {
            return None;
        }
        debug_assert!(node_read!(host, node, |n| !n
            .prevent_viewport_scrolling_from_inner
            || n.scrolls_inner_viewport));
        if node_read!(host, node, |n| n.scrolls_inner_viewport
            && !n.prevent_viewport_scrolling_from_inner)
        {
            debug_assert!(self.OuterViewportScrollNode(host).is_some());
            return self.OuterViewportScrollNode(host);
        }
        Some(node)
    }
    // Source: cc/input/input_handler.cc:1925-1939
    fn GetNodeToScrollForLayer<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        layer: &InputLayerId,
    ) -> Option<B::ScrollNode> {
        if host.LayerIsScrollbarLayer(&layer) {
            let id = host.LayerScrollbarScrollElementId(layer);
            if let Some(node) = host.ScrollTree_MutableFindNodeFromElementId(id) {
                return self.GetNodeToScroll(host, node);
            }
            return None;
        }
        let index = host.LayerScrollTreeIndex(&layer);
        let node = host.ScrollTree_MutableNode(index);
        self.GetNodeToScroll(host, node)
    }
    // Source: cc/input/input_handler.cc:1941-1974
    fn IsInitialScrollHitTestReliable<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        layer: &InputLayerId,
        first: Option<&InputLayerId>,
        out: &mut Option<B::ScrollNode>,
    ) -> bool {
        let node = self.GetNodeToScrollForLayer(host, layer);
        if Some(layer) == first {
            *out = node;
            return true;
        }
        if (first.is_some() && node.is_none()) || (node.is_some() && first.is_none()) {
            return false;
        }
        if first.is_none() && node.is_none() {
            *out = None;
            return true;
        }
        if node == self.GetNodeToScrollForLayer(host, first.unwrap()) {
            *out = node;
            return true;
        }
        false
    }
    // Source: cc/input/input_handler.cc:1976-2001
    pub fn ComputeScrollDelta<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        node: &B::ScrollNode,
        delta: Vector2dF,
        state: Option<&ScrollState>,
    ) -> Vector2dF {
        Self::ComputeScrollDeltaForState(
            host,
            self.last_scroll_update_state_.as_ref(),
            node,
            delta,
            state,
        )
    }
    // Source: cc/input/input_handler.cc:1976-2001
    pub fn ComputeScrollDeltaForState<C: InputHost<B> + ?Sized>(
        host: &mut C,
        last_scroll_update_state: Option<&ScrollState>,
        node: &B::ScrollNode,
        delta: Vector2dF,
        state: Option<&ScrollState>,
    ) -> Vector2dF {
        let factor = delegate!(host, PageScaleFactor());
        let adjusted = Self::UserScrollableDeltaForNode(host, node, scale(delta, 1. / factor));
        let id = node_read!(host, node, |n| n.element_id);
        let old = host.ScrollTree_current_scroll_offset(id);
        let new = host.ScrollTree_ClampScrollOffsetToLimits(old + adjusted, node);
        let mut updated = new - old;
        let state = if let Some(state) = state {
            state
        } else if let Some(last) = last_scroll_update_state {
            last
        } else {
            return updated;
        };
        Self::LimitDeltaToScrollerSizeForNode(host, state, node, &mut updated);
        updated
    }

    // Source: cc/input/input_handler.cc:2003-2048
    fn CalculateLocalScrollDeltaAndStartPoint<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        node: &B::ScrollNode,
        point: PointF,
        delta: Vector2dF,
        out: &mut Vector2dF,
        start: Option<&mut PointF>,
    ) -> bool {
        Self::CalculateLocalScrollDeltaAndStartPointForNode(host, node, point, delta, out, start)
    }
    // Source: cc/input/input_handler.cc:2003-2048.
    // Rust split-borrow entry for this same single source algorithm.
    fn CalculateLocalScrollDeltaAndStartPointForNode<C: InputHost<B> + ?Sized>(
        host: &mut C,
        node: &B::ScrollNode,
        point: PointF,
        delta: Vector2dF,
        out: &mut Vector2dF,
        start: Option<&mut PointF>,
    ) -> bool {
        if node_read!(host, node, |n| n.transform_id) == kInvalidPropertyNodeId {
            return false;
        }
        let transform = delegate!(
            host,
            ScrollTree_ScreenSpaceTransform(node_read!(host, node, |n| n.id))
        );
        let inverse = transform.GetCheckedInverse();
        let factor = host.ActiveTree_device_scale_factor();
        let screen = point_scale(point, factor);
        let screen_delta = scale(delta, factor);
        let mut start_clipped = false;
        let mut end_clipped = false;
        let local_start = MathUtil::ProjectPoint(&inverse, screen, &mut start_clipped);
        let local_end = MathUtil::ProjectPoint(&inverse, screen + screen_delta, &mut end_clipped);
        *out = local_end - local_start;
        if let Some(start) = start {
            *start = local_start;
        }
        if start_clipped || end_clipped {
            return false;
        }
        true
    }
    // Source: cc/input/input_handler.cc:2050-2096
    fn ScrollNodeWithViewportSpaceDelta<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        node: &B::ScrollNode,
        point: PointF,
        delta: Vector2dF,
    ) -> Vector2dF {
        Self::ScrollNodeWithViewportSpaceDeltaForNode(host, node, point, delta)
    }
    // Source: cc/input/input_handler.cc:2050-2096.
    // Rust split-borrow entry for this same single source algorithm.
    fn ScrollNodeWithViewportSpaceDeltaForNode<C: InputHost<B> + ?Sized>(
        host: &mut C,
        node: &B::ScrollNode,
        point: PointF,
        delta: Vector2dF,
    ) -> Vector2dF {
        let mut local_start = PointF::default();
        let mut local_delta = Vector2dF::default();
        if !Self::CalculateLocalScrollDeltaAndStartPointForNode(
            host,
            node,
            point,
            delta,
            &mut local_delta,
            Some(&mut local_start),
        ) {
            return Vector2dF::default();
        }
        let _trace = delegate!(
            host,
            BeginTrace(
                "ScrollNodeWithViewportSpaceDelta",
                &[
                    ("delta_y", InputTraceValue::Float(local_delta.y() as f64)),
                    (
                        "is_outer",
                        InputTraceValue::Bool(node_read!(host, node, |n| n.scrolls_outer_viewport)),
                    ),
                ],
            )
        );
        let id = node_read!(host, node, |n| n.element_id);
        let previous = host.ScrollTree_current_scroll_offset(id);
        host.ScrollBy(node, local_delta);
        let scrolled = host.ScrollTree_current_scroll_offset(id) - previous;
        host.InstantTrace(
            "ConsumedDelta",
            &[("y", InputTraceValue::Float(scrolled.y() as f64))],
        );
        let actual_local_end = local_start + scrolled;
        let mut clipped = false;
        let transform = delegate!(
            host,
            ScrollTree_ScreenSpaceTransform(node_read!(host, node, |n| n.id))
        );
        let screen_end = MathUtil::MapPoint(&transform, actual_local_end, &mut clipped);
        debug_assert!(!clipped);
        if clipped {
            return Vector2dF::default();
        }
        let actual_viewport_end =
            point_scale(screen_end, 1. / host.ActiveTree_device_scale_factor());
        actual_viewport_end - point
    }
    // Source: cc/input/input_handler.cc:2098-2120
    fn ScrollNodeWithLocalDelta<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        node: &B::ScrollNode,
        delta: Vector2dF,
    ) -> Vector2dF {
        Self::ScrollNodeWithLocalDeltaForNode(host, node, delta)
    }
    // Source: cc/input/input_handler.cc:2098-2120.
    // Rust split-borrow entry for this same single source algorithm.
    fn ScrollNodeWithLocalDeltaForNode<C: InputHost<B> + ?Sized>(
        host: &mut C,
        node: &B::ScrollNode,
        delta: Vector2dF,
    ) -> Vector2dF {
        let _trace = delegate!(
            host,
            BeginTrace(
                "ScrollNodeWithLocalDelta",
                &[
                    ("delta_y", InputTraceValue::Float(delta.y() as f64)),
                    (
                        "is_outer",
                        InputTraceValue::Bool(node_read!(host, node, |n| n.scrolls_outer_viewport)),
                    ),
                ],
            )
        );
        let factor = delegate!(host, PageScaleFactor());
        let id = node_read!(host, node, |n| n.element_id);
        let previous = host.ScrollTree_current_scroll_offset(id);
        host.ScrollBy(node, scale(delta, 1. / factor));
        let scrolled = host.ScrollTree_current_scroll_offset(id) - previous;
        let consumed = scale(scrolled, factor);
        host.InstantTrace(
            "ConsumedDelta",
            &[("y", InputTraceValue::Float(consumed.y() as f64))],
        );
        consumed
    }
    // Source: cc/input/input_handler.cc:2124-2146
    pub fn ScrollSingleNode<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        node: &B::ScrollNode,
        delta: Vector2dF,
        point: Point,
        direct: bool,
    ) -> Vector2dF {
        Self::ScrollSingleNodeForNode(host, node, delta, point, direct)
    }
    // Source: cc/input/input_handler.cc:2124-2146.
    // Rust split-borrow entry for this same single source algorithm.
    pub fn ScrollSingleNodeForNode<C: InputHost<B> + ?Sized>(
        host: &mut C,
        node: &B::ScrollNode,
        delta: Vector2dF,
        point: Point,
        direct: bool,
    ) -> Vector2dF {
        let adjusted = Self::UserScrollableDeltaForNode(host, node, delta);
        if direct {
            let scaled = scale(adjusted, 1. / host.ActiveTree_external_page_scale_factor());
            return Self::ScrollNodeWithViewportSpaceDeltaForNode(
                host,
                node,
                vector(point),
                scaled,
            );
        }
        Self::ScrollNodeWithLocalDeltaForNode(host, node, adjusted)
    }
    // Source: cc/input/input_handler.cc:2148-2173
    fn GetAnimatingNodeForCurrentScrollingNode<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
    ) -> Option<B::ScrollNode> {
        let node = self.CurrentlyScrollingNode(host)?;
        if delegate!(
            host,
            ElementHasImplOnlyScrollAnimation(node_read!(host, node, |n| n.element_id))
        ) {
            return Some(node);
        }
        if let Some(inner) = self.InnerViewportScrollNode(host) {
            if node_read!(host, node, |n| n.scrolls_outer_viewport) {
                if delegate!(
                    host,
                    ElementHasImplOnlyScrollAnimation(node_read!(host, inner, |n| n.element_id))
                ) {
                    return Some(inner);
                }
            }
        }
        None
    }
    // Source: cc/input/input_handler.cc:2175-2314
    fn ScrollLatchedScroller<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        state: &mut ScrollState,
        delay: TimeDelta,
    ) {
        debug_assert!(self.CurrentlyScrollingNode(host).is_some());
        debug_assert!(self.latched_scroll_type_.is_some());
        let node = self.CurrentlyScrollingNode(host).unwrap();
        let delta = Vector2dF::new(state.delta_x() as f32, state.delta_y() as f32);
        let _trace = host.BeginTrace(
            "InputHandler::ScrollLatchedScroller",
            &[
                ("delta_x", InputTraceValue::Float(delta.x() as f64)),
                ("delta_y", InputTraceValue::Float(delta.y() as f64)),
            ],
        );
        let mut applied = Vector2dF::default();
        let mut content = Vector2dF::default();
        let strategy_offset;
        if self.ShouldAnimateScroll(host, state) {
            debug_assert!(!state.is_in_inertial_phase());
            if let Some(animating) = self.GetAnimatingNodeForCurrentScrollingNode(host) {
                host.InstantTrace("UpdateExistingAnimation", &[]);
                debug_assert!(
                    node_read!(host, animating, |n| n.id) == node_read!(host, node, |n| n.id)
                        || node_read!(host, animating, |n| n.scrolls_inner_viewport)
                );
                strategy_offset = self.ScrollAnimationUpdateTarget(host, &animating, delta, delay);
                if strategy_offset.is_some() {
                    applied = delta;
                } else {
                    host.InstantTrace("Didn't Update Animation", &[]);
                }
            } else {
                host.InstantTrace("CreateNewAnimation", &[]);
                if node_read!(host, node, |n| n.scrolls_outer_viewport) {
                    let result = host.ViewportScrollAnimated(&self.View(), delta, delay);
                    applied = result.consumed_delta;
                    self.SetViewportConsumedDelta(&result);
                } else {
                    applied = self.ComputeScrollDelta(host, &node, delta, Some(state));
                    delegate!(host, ScrollAnimationCreate(&node, &applied, delay));
                }
                let current = self.GetVisualScrollOffset(host, &node);
                strategy_offset =
                    Some(host.ScrollTree_ClampScrollOffsetToLimits(current + applied, &node));
            }
            content = delta;
        } else {
            let point = Point::new(state.position_x(), state.position_y());
            if host.Viewport_ShouldScroll(&node) {
                let result = delegate!(
                    host,
                    ViewportScrollBy(
                        &self.View(),
                        delta,
                        point,
                        state.is_direct_manipulation(),
                        self.latched_scroll_type_ != Some(ScrollInputType::kWheel),
                        node_read!(host, node, |n| n.scrolls_outer_viewport),
                        state.is_in_inertial_phase(),
                    )
                );
                applied = result.consumed_delta;
                content = result.content_scrolled_delta;
                self.SetViewportConsumedDelta(&result);
            } else {
                applied = self.ScrollSingleNode(
                    host,
                    &node,
                    delta,
                    point,
                    state.is_direct_manipulation(),
                );
            }
            strategy_offset = Some(self.GetVisualScrollOffset(host, &node));
        }
        self.overscroll_delta_for_main_thread_ += delta - applied;
        let mut scrolled = applied.x().abs() > kScrollEpsilon;
        scrolled = scrolled || applied.y().abs() > kScrollEpsilon;
        if !scrolled {
            if node_read!(host, node, |n| n.scrolls_outer_viewport) {
                state.ConsumeDelta(applied.x() as f64, applied.y() as f64);
            }
            return;
        }
        if !host.Viewport_ShouldScroll(&node) {
            if MathUtil::SmallestAngleBetweenVectors(applied, delta) < 45. {
                applied = delta;
            } else {
                applied = MathUtil::ProjectVector(delta, applied);
            }
            content = applied;
        }
        state.set_caused_scroll(
            content.x().abs() > kScrollEpsilon,
            content.y().abs() > kScrollEpsilon,
        );
        state.ConsumeDelta(applied.x() as f64, applied.y() as f64);
        self.did_scroll_x_for_scroll_gesture_ |= state.caused_scroll_x();
        self.did_scroll_y_for_scroll_gesture_ |= state.caused_scroll_y();
        if let Some(offset) = strategy_offset {
            let last = self.last_scroll_update_state_.as_ref().unwrap_or(state);
            self.snap_strategy_ = Some(self.CreateSnapStrategy(
                last,
                offset,
                SnapReason::kScrollOffsetAnimationFinished,
            ));
        }
    }
    // Source: cc/input/input_handler.cc:2316-2319
    fn CanPropagate<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        node: &B::ScrollNode,
        x: f32,
        y: f32,
    ) -> bool {
        node_read!(host, node, |n| {
            (x == 0. || n.overscroll_behavior.PropagatesXScroll())
                && (y == 0. || n.overscroll_behavior.PropagatesYScroll())
        })
    }
    // Source: cc/input/input_handler.cc:2321-2388
    fn FindNodeToLatch<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        state: &mut ScrollState,
        starting: Option<B::ScrollNode>,
        input: ScrollInputType,
    ) -> Option<B::ScrollNode> {
        let mut selected = None;
        let mut first = None;
        let mut current = starting;
        while let Some(node) = current {
            if host.Viewport_ShouldScroll(&node) {
                selected = self.GetNodeToScroll(host, node);
                break;
            }
            let chaining = node_read!(host, node, |n| {
                n.overscroll_behavior.PropagatesXScroll()
                    && n.overscroll_behavior.PropagatesYScroll()
            });
            if node_read!(host, node, |n| !n.user_scrollable_horizontal
                && !n.user_scrollable_vertical)
                && chaining
            {
                current = if host.ScrollTree_HasParent(&node) {
                    Some(host.ScrollTree_MutableParent(&node))
                } else {
                    None
                };
                continue;
            }
            if first.is_none() {
                first = Some(node.clone());
            }
            if self.CanConsumeDelta(host, state, &node) {
                selected = Some(node);
                break;
            }
            let x = if state.is_beginning() {
                state.delta_x_hint()
            } else {
                state.delta_x()
            };
            let y = if state.is_beginning() {
                state.delta_y_hint()
            } else {
                state.delta_y()
            };
            if !self.CanPropagate(host, &node, x as f32, y as f32) {
                selected = Some(node);
                state.set_is_scroll_chain_cut(true);
                break;
            }
            current = if host.ScrollTree_HasParent(&node) {
                Some(host.ScrollTree_MutableParent(&node))
            } else {
                None
            };
        }
        if input == ScrollInputType::kAutoscroll && first.is_some() {
            if !selected
                .as_ref()
                .is_some_and(|node| self.CanConsumeDelta(host, state, node))
            {
                selected = first;
            }
        }
        selected
    }
    // Source: cc/input/input_handler.cc:2390-2398
    fn UpdateRootLayerStateForSynchronousInputHandler<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
    ) {
        if let Some(client) = &self.input_handler_client_ {
            client.UpdateRootLayerStateForSynchronousInputHandler(
                host.ActiveTree_TotalScrollOffset(),
                host.ActiveTree_TotalMaxScrollOffset(),
                host.ActiveTree_ScrollableSize(),
                host.ActiveTree_current_page_scale_factor(),
                host.ActiveTree_min_page_scale_factor(),
                host.ActiveTree_max_page_scale_factor(),
            );
        }
    }
    // Source: cc/input/input_handler.cc:2400-2421
    fn DidLatchToScroller<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        state: &ScrollState,
        input: ScrollInputType,
    ) {
        debug_assert!(self.CurrentlyScrollingNode(host).is_some());
        let id = node_read!(host, self.CurrentlyScrollingNode(host).unwrap(), |n| n
            .element_id);
        self.deferred_scroll_ends_.remove(&id);
        delegate!(host, ScrollBegin());
        if let Some(animating) = self.GetAnimatingNodeForCurrentScrollingNode(host) {
            delegate!(
                host,
                ScrollAnimationAbort(node_read!(host, animating, |n| n.element_id))
            );
        }
        self.last_latched_scroller_ =
            node_read!(host, self.CurrentlyScrollingNode(host).unwrap(), |n| n
                .element_id);
        self.latched_scroll_type_ = Some(input);
        self.last_scroll_begin_state_ = Some(state.clone());
        self.prevent_scroll_axis_locking_ = Some(node_read!(
            host,
            self.CurrentlyScrollingNode(host).unwrap(),
            |n| n.prevent_scroll_axis_locking
        ));
        self.ClearAnimatingSnapTargetsForElement(self.last_latched_scroller_);
        delegate!(host, DidStartScroll());
        self.UpdateScrollSourceInfo(state, input);
        self.UpdateLastLatchedScrollSourceType();
    }
    // Source: cc/input/input_handler.cc:2423-2457
    pub fn CanConsumeDelta<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        state: &ScrollState,
        node: &B::ScrollNode,
    ) -> bool {
        Self::CanConsumeDeltaForState(host, self.last_scroll_update_state_.as_ref(), state, node)
    }
    // Source: cc/input/input_handler.cc:2423-2457.
    // Rust split-borrow entry for this same single source algorithm.
    pub fn CanConsumeDeltaForState<C: InputHost<B> + ?Sized>(
        host: &mut C,
        last_scroll_update_state: Option<&ScrollState>,
        state: &ScrollState,
        node: &B::ScrollNode,
    ) -> bool {
        let mut delta = if state.is_beginning() {
            Vector2dF::new(state.delta_x_hint() as f32, state.delta_y_hint() as f32)
        } else {
            Vector2dF::new(state.delta_x() as f32, state.delta_y() as f32)
        };
        if zero(delta) {
            return true;
        }
        if state.is_direct_manipulation() {
            let mut local = Vector2dF::default();
            if !Self::CalculateLocalScrollDeltaAndStartPointForNode(
                host,
                node,
                PointF::new(state.position_x() as f32, state.position_y() as f32),
                delta,
                &mut local,
                None,
            ) {
                return false;
            }
            delta = local;
        } else {
            delta = Self::ResolveScrollGranularityToPixelsForNode(
                host,
                node,
                delta,
                state.delta_granularity(),
            );
        }
        if !zero(Self::ComputeScrollDeltaForState(
            host,
            last_scroll_update_state,
            node,
            delta,
            Some(state),
        )) {
            return true;
        }
        false
    }
    // Source: cc/input/input_handler.cc:2459-2467
    fn ShouldAnimateScroll<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        state: &ScrollState,
    ) -> bool {
        ShouldAnimateScrollForState(host.GetSettings(), state)
    }
    // Source: cc/input/input_handler.cc:2469-2561
    fn SnapAtScrollEnd<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        reason: SnapReason,
    ) -> bool {
        let Some(node) = self.CurrentlyScrollingNode(host) else {
            return false;
        };
        if !node_read!(host, node, |n| n.snap_container_data.is_some()) {
            return false;
        }
        let current = self.GetVisualScrollOffset(host, &node);
        debug_assert!(
            self.last_scroll_update_state_.is_some() || self.last_scroll_begin_state_.is_some()
        );
        let last = self
            .last_scroll_update_state_
            .as_ref()
            .or(self.last_scroll_begin_state_.as_ref())
            .unwrap();
        if self.snap_strategy_.is_none() || last.is_in_inertial_phase() {
            self.snap_strategy_ = Some(self.CreateSnapStrategy(last, current, reason));
        }
        let adjustment = if node_read!(host, node, |n| n.scrolls_outer_viewport) {
            self.PredictViewportBoundsDelta(host, Vector2dF::default())
        } else {
            0.
        };
        let snap = node_write!(host, node, |n| {
            n.snap_container_data
                .as_mut()
                .unwrap()
                .FindSnapPositionWithViewportAdjustment(
                    self.snap_strategy_.as_ref().unwrap().as_ref(),
                    adjustment,
                )
        });
        let id = node_read!(host, node, |n| n.element_id);
        if snap.r#type == SnapPositionType::kNone {
            if reason == SnapReason::kScrollOffsetAnimationFinished {
                let current_id =
                    node_read!(host, self.CurrentlyScrollingNode(host).unwrap(), |n| n
                        .element_id);
                self.EnsureSnapAnimationData(current_id);
                self.SetAnimatingSnapTargetsForElement(current_id, snap.target_element_ids);
            } else if node_write!(host, node, |n| {
                n.snap_container_data
                    .as_mut()
                    .unwrap()
                    .SetTargetSnapAreaElementIds(snap.target_element_ids)
            }) {
                self.updated_snapped_elements_
                    .insert(id, snap.target_element_ids);
            }
            return false;
        }
        if host.Viewport_ShouldScroll(&node) {
            delegate!(host, WillScrollContent(id));
        }
        let delta = snap.position - current;
        let did_animate;
        if node_read!(host, node, |n| n.scrolls_outer_viewport) {
            let scaled = scale(delta, delegate!(host, PageScaleFactor()));
            let result = host.ViewportScrollAnimated(&self.View(), scaled, TimeDelta::default());
            did_animate = !zero(result.consumed_delta);
            self.SetViewportConsumedDelta(&result);
        } else {
            did_animate = delegate!(
                host,
                ScrollAnimationCreate(&node, &delta, TimeDelta::default())
            );
        }
        debug_assert!(!self.IsAnimatingForSnap(node_read!(
            host,
            self.CurrentlyScrollingNode(host).unwrap(),
            |n| n.element_id
        )));
        if did_animate {
            let ended =
                reason == SnapReason::kGestureScrollEnd || self.deferred_scroll_ends_.contains(&id);
            if ended {
                self.ClearCurrentlyScrollingNode(host);
            }
            self.EnsureSnapAnimationData(id);
            self.SetAnimatingSnapTargetsForElement(id, snap.target_element_ids);
        } else if node_write!(host, node, |n| {
            n.snap_container_data
                .as_mut()
                .unwrap()
                .SetTargetSnapAreaElementIds(snap.target_element_ids)
        }) {
            self.updated_snapped_elements_
                .insert(id, snap.target_element_ids);
            self.SetNeedsCommit(host);
        }
        did_animate
    }
    // Source: cc/input/input_handler.cc:2563-2566
    fn IsAnimatingForSnap(&self, id: ElementId) -> bool {
        self.GetAnimatingSnapTargetsForElement(id) != TargetSnapAreaElementIds::default()
    }
    // Source: cc/input/input_handler.cc:2568-2573
    fn GetVisualScrollOffset<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        node: &B::ScrollNode,
    ) -> PointF {
        if node_read!(host, node, |n| n.scrolls_outer_viewport) {
            return host.Viewport_TotalScrollOffset();
        }
        delegate!(
            host,
            ScrollTree_current_scroll_offset(node_read!(host, node, |n| n.element_id))
        )
    }
    // Source: cc/input/input_handler.cc:2575-2592
    fn ClearCurrentlyScrollingNode<C: InputHost<B> + ?Sized>(&mut self, host: &mut C) {
        let _trace = host.BeginTrace("InputHandler::ClearCurrentlyScrollingNode", &[]);
        self.ClearAnimatingSnapTargetsForElement(
            self.CurrentlyScrollingNode(host)
                .map(|n| node_read!(host, n, |n| n.element_id))
                .unwrap_or_default(),
        );
        host.ClearCurrentlyScrollingNode();
        self.accumulated_root_overscroll_ = Vector2dF::default();
        self.did_scroll_x_for_scroll_gesture_ = false;
        self.did_scroll_y_for_scroll_gesture_ = false;
        self.delta_consumed_for_scroll_gesture_ = false;
        self.prevent_scroll_axis_locking_ = None;
        self.latched_scroll_type_ = None;
        self.last_scroll_update_state_ = None;
        self.last_scroll_begin_state_ = None;
        delegate!(host, DidEndScroll());
    }
    // Source: cc/input/input_handler.cc:2594-2621
    fn ScrollAnimationUpdateTarget<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        node: &B::ScrollNode,
        delta: Vector2dF,
        delay: TimeDelta,
    ) -> Option<PointF> {
        let id = node_read!(host, node, |n| n.element_id);
        debug_assert!(delegate!(host, ElementHasImplOnlyScrollAnimation(id)));
        let factor = delegate!(host, PageScaleFactor());
        let adjusted = self.UserScrollableDelta(host, node, scale(delta, 1. / factor));
        let target = delegate!(
            host,
            UpdateImplAnimationScrollTargetWithDelta(
                adjusted,
                node_read!(host, node, |n| n.id),
                delay,
                id
            )
        );
        if target.is_some() {
            delegate!(host, DidUpdateScrollAnimationCurve());
            self.ClearAnimatingSnapTargetsForElement(id);
        }
        target
    }
    // Source: cc/input/input_handler.cc:2623-2636
    fn UpdateScrollSourceInfo(&mut self, state: &ScrollState, input: ScrollInputType) {
        if input == ScrollInputType::kWheel
            && state.delta_granularity() == ScrollGranularity::kScrollByPrecisePixel
        {
            self.has_scrolled_by_precisiontouchpad_ = true;
        } else if input == ScrollInputType::kWheel {
            self.has_scrolled_by_wheel_ = true;
        } else if input == ScrollInputType::kTouchscreen {
            self.has_scrolled_by_touch_ = true;
        } else if input == ScrollInputType::kScrollbar {
            self.has_scrolled_by_scrollbar_ = true;
        }
    }
    // Source: cc/input/input_handler.cc:2640-2656
    fn IsScrolledBy<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        child: Option<&InputLayerId>,
        ancestor: &B::ScrollNode,
    ) -> bool {
        debug_assert!(node_read!(host, ancestor, |n| n.user_scrollable_horizontal
            || n.user_scrollable_vertical));
        let Some(child) = child else {
            return false;
        };
        debug_assert!(host.LayerBelongsToActiveTree(&child));
        let mut current = Some(delegate!(
            host,
            ScrollTree_MutableNode(host.LayerScrollTreeIndex(&child))
        ));
        while let Some(node) = current {
            if node_read!(host, node, |n| n.id) == node_read!(host, ancestor, |n| n.id) {
                return true;
            }
            current = if host.ScrollTree_HasParent(&node) {
                Some(host.ScrollTree_MutableParent(&node))
            } else {
                None
            };
        }
        false
    }
    // Source: cc/input/input_handler.cc:2658-2668
    fn UserScrollableDelta<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        node: &B::ScrollNode,
        delta: Vector2dF,
    ) -> Vector2dF {
        Self::UserScrollableDeltaForNode(host, node, delta)
    }
    // Source: cc/input/input_handler.cc:2658-2668
    // Split-borrow form of the same source body; it owns no InputHandler state.
    fn UserScrollableDeltaForNode<C: InputHost<B> + ?Sized>(
        host: &mut C,
        node: &B::ScrollNode,
        delta: Vector2dF,
    ) -> Vector2dF {
        let mut adjusted = delta;
        if !node_read!(host, node, |n| n.user_scrollable_horizontal) {
            adjusted.set_x(0.);
        }
        if !node_read!(host, node, |n| n.user_scrollable_vertical) {
            adjusted.set_y(0.);
        }
        adjusted
    }
    // Source: cc/input/input_handler.cc:2670-2672
    pub fn ScrollbarScrollIsActive(&self) -> bool {
        self.scrollbar_controller_.ScrollbarScrollIsActive()
    }
    // Source: cc/input/input_handler.cc:2674-2676
    pub fn SetDeferBeginMainFrame<C: InputHost<B> + ?Sized>(&self, host: &mut C, defer: bool) {
        delegate!(host, SetDeferBeginMainFrame(defer));
    }
    // Source: cc/input/input_handler.cc:2678-2686
    pub fn UpdateBrowserControlsState<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
        constraints: BrowserControlsState,
        current: BrowserControlsState,
        animate: bool,
        modifications: Option<&BrowserControlsOffsetTagModifications>,
    ) {
        delegate!(
            host,
            UpdateBrowserControlsState(constraints, current, animate, modifications)
        );
    }
    // Source: cc/input/input_handler.cc:2688-2692
    pub fn SetIsHandlingTouchSequence(&mut self, handling: bool) {
        debug_assert!(!handling || !self.is_handling_touch_sequence_);
        self.is_handling_touch_sequence_ = handling;
    }
    // Source: cc/input/input_handler.cc:2694-2699
    pub fn LatchedScrollerElementId<C: InputHost<B> + ?Sized>(&self, host: &mut C) -> ElementId {
        if let Some(node) = self.CurrentlyScrollingNode(host) {
            return node_read!(host, node, |n| n.element_id);
        }
        ElementId::default()
    }
    // Source: cc/input/input_handler.cc:2701-2728
    pub fn CurrentScrollNeedsFrameAlignment<C: InputHost<B> + ?Sized>(&self, host: &mut C) -> bool {
        if let Some(node) = self.CurrentlyScrollingNode(host) {
            if delegate!(
                host,
                HasScrollLinkedAnimation(node_read!(host, node, |n| n.element_id))
            ) {
                return true;
            }
        }
        for (id, data) in &self.snap_animation_data_map_ {
            if data.animating_snap_target_ids_ == TargetSnapAreaElementIds::default() {
                continue;
            }
            if let Some(node) = host.ScrollTree_MutableFindNodeFromElementId(*id) {
                if delegate!(
                    host,
                    HasScrollLinkedAnimation(node_read!(host, node, |n| n.element_id))
                ) {
                    return true;
                }
            }
        }
        false
    }
    // Source: cc/input/input_handler.cc:2730-2753
    fn CreateSnapStrategy(
        &self,
        state: &ScrollState,
        current: PointF,
        reason: SnapReason,
    ) -> Box<dyn SnapSelectionStrategy> {
        let delta = state.DeltaOrHint();
        if self.latched_scroll_type_ == Some(ScrollInputType::kWheel)
            && state.delta_granularity() != ScrollGranularity::kScrollByPrecisePixel
            && !zero(delta)
            && reason == SnapReason::kScrollOffsetAnimationFinished
        {
            <dyn SnapSelectionStrategy>::CreateForDirection(
                current,
                delta,
                true,
                SnapStopAlwaysFilter::kIgnore,
            )
        } else {
            <dyn SnapSelectionStrategy>::CreateForEndPosition(
                &current,
                self.did_scroll_x_for_scroll_gesture_,
                self.did_scroll_y_for_scroll_gesture_,
            )
        }
    }
    // Source: cc/input/input_handler.cc:2755-2765
    pub fn SetViewportConsumedDelta(&mut self, result: &ViewportScrollResult) {
        if result.outer_viewport_scrolled_delta.x().abs() > kScrollEpsilon
            || result.outer_viewport_scrolled_delta.y().abs() > kScrollEpsilon
        {
            self.outer_viewport_consumed_delta_ = true;
        }
        if result.inner_viewport_scrolled_delta.x().abs() > kScrollEpsilon
            || result.inner_viewport_scrolled_delta.y().abs() > kScrollEpsilon
        {
            self.inner_viewport_consumed_delta_ = true;
        }
    }
    // Source: cc/input/input_handler.cc:2767-2774
    fn GetAnimatingSnapTargetsForElement(&self, id: ElementId) -> TargetSnapAreaElementIds {
        if let Some(data) = self.snap_animation_data_map_.get(&id) {
            return data.animating_snap_target_ids_;
        }
        TargetSnapAreaElementIds::default()
    }
    // Source: cc/input/input_handler.cc:2776-2783
    fn SetAnimatingSnapTargetsForElement(
        &mut self,
        id: ElementId,
        targets: TargetSnapAreaElementIds,
    ) {
        if let Some(data) = self.snap_animation_data_map_.get_mut(&id) {
            data.animating_snap_target_ids_ = targets;
        }
    }
    // Source: cc/input/input_handler.cc:2785-2787
    fn ClearAnimatingSnapTargetsForElement(&mut self, id: ElementId) {
        self.SetAnimatingSnapTargetsForElement(id, TargetSnapAreaElementIds::default());
    }
    // Source: cc/input/input_handler.cc:2789-2791
    fn EnsureSnapAnimationData(&mut self, id: ElementId) {
        self.snap_animation_data_map_.entry(id).or_default();
    }
    // Source: cc/input/input_handler.h:515-546
    pub fn set_external_pinch_gesture_active(&mut self, active: bool) {
        self.external_pinch_gesture_active_ = active;
        debug_assert!(!self.pinch_gesture_active_ || !self.external_pinch_gesture_active_);
    }
    pub fn pinch_gesture_active(&self) -> bool {
        self.pinch_gesture_active_ || self.external_pinch_gesture_active_
    }
    pub fn set_force_smooth_wheel_scrolling_for_testing(&mut self, enabled: bool) {
        self.force_smooth_wheel_scrolling_for_testing_ = enabled;
    }
    pub fn accumulated_root_overscroll_for_testing(&self) -> Vector2dF {
        self.accumulated_root_overscroll_
    }
    pub fn animating_for_snap_for_testing(&self, id: ElementId) -> bool {
        self.IsAnimatingForSnap(id)
    }
    pub fn get_snap_animation_data_map_for_testing(
        &mut self,
    ) -> &mut BTreeMap<ElementId, SnapAnimationData> {
        &mut self.snap_animation_data_map_
    }
    pub fn snap_strategy_for_testing(&self) -> &Option<Box<dyn SnapSelectionStrategy>> {
        &self.snap_strategy_
    }
    // Source: cc/input/input_handler.h:724-726
    pub fn scrollbar_controller_for_testing(&self) -> &ScrollbarController {
        &self.scrollbar_controller_
    }
    // Source: cc/input/input_handler.h:724-726. Explicit Rust mutable borrow.
    pub fn scrollbar_controller_for_testing_mut(&mut self) -> &mut ScrollbarController {
        &mut self.scrollbar_controller_
    }
}
// Source: cc/input/input_handler.cc:114 (default destructor).
// Rust normally destroys fields in declaration order. Invalidate the weak
// factory first to preserve the source's reverse C++ member destruction order.
impl<B: InputHandlerTypes> Drop for InputHandler<B> {
    fn drop(&mut self) {
        self.weak_factory_.take();
    }
}

// Source: cc/input/input_handler.h:555-579. Source virtual interface dispatch.
impl<B: InputHandlerTypes> InputDelegateForCompositor<B> for InputHandler<B> {
    fn RunAutoscrollTask<C:InputHost<B>+?Sized>(&mut self,
        task:crate::adapter::scrollbar::AutoscrollTask,host:&mut C){
        InputHandler::RunAutoscrollTask(self,task,host)
    }

    fn ProcessCommitDeltas<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        data: &mut B::CompositorCommitData,
        mutator: Option<&B::MutatorHost>,
        next_bmf: bool,
    ) {
        InputHandler::ProcessCommitDeltas(self, host, data, mutator, next_bmf)
    }
    fn TickAnimations(&mut self, time: TimeTicks) {
        InputHandler::TickAnimations(self, time)
    }
    fn WillShutdown(&mut self) {
        InputHandler::WillShutdown(self)
    }
    fn WillDraw(&mut self) {
        InputHandler::WillDraw(self)
    }
    fn WillBeginImplFrame<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        args: &BeginFrameArgs,
    ) {
        InputHandler::WillBeginImplFrame(self, host, args)
    }
    fn DidCommit<C: InputHost<B> + ?Sized>(&mut self, host: &mut C) {
        InputHandler::DidCommit(self, host)
    }
    fn DidImplSideInvalidate<C: InputHost<B> + ?Sized>(&mut self, host: &mut C) {
        InputHandler::DidImplSideInvalidate(self, host)
    }
    fn DidActivatePendingTree<C: InputHost<B> + ?Sized>(&mut self, host: &mut C) {
        InputHandler::DidActivatePendingTree(self, host)
    }
    fn DidFinishImplFrame(&mut self) {
        InputHandler::DidFinishImplFrame(self)
    }
    fn OnBeginImplFrameDeadline<C: InputHost<B> + ?Sized>(&mut self, host: &mut C) {
        InputHandler::OnBeginImplFrameDeadline(self, host)
    }
    fn RootLayerStateMayHaveChanged<C: InputHost<B> + ?Sized>(&mut self, host: &mut C) {
        InputHandler::RootLayerStateMayHaveChanged(self, host)
    }
    fn DidRegisterScrollbar<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        id: ElementId,
        orientation: B::ScrollbarOrientation,
    ) {
        InputHandler::DidRegisterScrollbar(self, host, id, orientation)
    }
    fn DidUnregisterScrollbar<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        id: ElementId,
        orientation: B::ScrollbarOrientation,
    ) {
        InputHandler::DidUnregisterScrollbar(self, host, id, orientation)
    }
    fn DidRegisterScrollbarWithHost<
        C: crate::adapter::scrollbar::ScrollbarRegistrationHost + ?Sized,
    >(
        &mut self,
        host: &mut C,
        id: ElementId,
        orientation: B::ScrollbarOrientation,
    ) {
        InputHandler::DidRegisterScrollbarWithHost(self, host, id, orientation);
    }
    fn DidUnregisterScrollbarWithHost<
        C: crate::adapter::scrollbar::ScrollbarRegistrationHost + ?Sized,
    >(
        &mut self,
        host: &mut C,
        id: ElementId,
        orientation: B::ScrollbarOrientation,
    ) {
        InputHandler::DidUnregisterScrollbarWithHost(self, host, id, orientation);
    }
    fn ScrollOffsetAnimationFinished<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        id: ElementId,
    ) {
        InputHandler::ScrollOffsetAnimationFinished(self, host, id)
    }
    fn ElasticOverscrollAnimationFinished<C: InputHost<B> + ?Sized>(
        &mut self,
        host: &mut C,
        id: ElementId,
    ) {
        InputHandler::ElasticOverscrollAnimationFinished(self, host, id)
    }
    fn SetPrefersReducedMotion(&mut self, reduced: bool) {
        InputHandler::SetPrefersReducedMotion(self, reduced)
    }
    fn IsCurrentlyScrolling<C: InputHost<B> + ?Sized>(&self, host: &mut C) -> bool {
        InputHandler::IsCurrentlyScrolling(self, host)
    }
    fn WithRegistrationAndView<Output>(
        &mut self,
        callback: impl FnOnce(&mut ScrollbarController, InputHandlerView<'_>) -> Output,
    ) -> Output {
        InputHandler::WithRegistrationAndView(self, callback)
    }
    fn View(&self) -> InputHandlerView<'_> {
        InputHandler::View(self)
    }
    fn GetActivelyScrollingType<C: InputHost<B> + ?Sized>(
        &self,
        host: &mut C,
    ) -> ActivelyScrollingType {
        InputHandler::GetActivelyScrollingType(self, host)
    }
    fn IsHandlingTouchSequence(&self) -> bool {
        InputHandler::IsHandlingTouchSequence(self)
    }
    fn IsCurrentScrollMainRepainted<C: InputHost<B> + ?Sized>(&self, host: &mut C) -> bool {
        InputHandler::IsCurrentScrollMainRepainted(self, host)
    }
    fn HasQueuedInput(&self) -> bool {
        InputHandler::HasQueuedInput(self)
    }
}

impl<B:InputHandlerTypes> InputHandler<B>{
    /// Source: cc/input/scrollbar_controller.cc:151-156. The source weak once
    /// callback reaches the same resident controller with a live Host context.
    pub(crate) fn RunAutoscrollTask<C:crate::adapter::input_handler::InputHost<B>+?Sized>(
        &mut self,token:crate::adapter::scrollbar::AutoscrollTask,host:&mut C,
    ){
        self.WithRegistrationAndView(|controller,view|{
            host.WithScrollbarHost(&view,view.last_scroll_update_state_.as_ref(),
                |scrollbar_host|controller.RunAutoscrollTask(scrollbar_host,token))
        });
    }
}
