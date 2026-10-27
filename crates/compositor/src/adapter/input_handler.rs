//! Live native tree/layer/viewport access used by the translated InputHandler.
//! Node IDs resolve the canonical ScrollNode for each operation; no copies
//! of ScrollNode, SnapContainerData, or layer/property trees are retained here.
use super::{
    geometry::{Point, PointF, Rect, Size, SizeF, Vector2dF},
    input_services::InputHostTypes,
    input_types::ElementId,
    scrollbar::{ScrollbarHost, ScrollbarTransform},
    time::{TimeDelta, TimeTicks},
};
use crate::cc::{
    input::{
        browser_controls_offset_tag_modifications::BrowserControlsOffsetTagModifications,
        browser_controls_state::BrowserControlsState,
        event_listener_properties::{EventListenerClass, EventListenerProperties},
        input_handler::ViewportScrollResult,
        overscroll_behavior::OverscrollBehavior,
        scroll_snap_data::TargetSnapAreaElementIds,
        scroll_state::ScrollState,
        scrollbar::ScrollbarOrientation,
        touch_action::TouchAction,
    },
    trees::{compositor_commit_data::CompositorCommitData, scroll_node::ScrollNode},
};
pub use compositor_input::input_handler_types::{
    InputFeature, ScrollInputType, ScrollTrackingType, ScrollingThread,
};
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub struct InputHandlerWeak {
    pub owner_id: u64,
    pub alive: std::sync::Weak<()>,
}
impl InputHandlerWeak {
    pub fn IsValid(&self) -> bool {
        self.alive.upgrade().is_some()
    }
}
pub use compositor_input::input_types::{InputLayerId, InputScrollNodeId};
pub trait InputSettings {
    fn enable_fluent_overlay_scrollbar(&self) -> bool;
    fn is_for_embedded_frame(&self) -> bool;
    fn scrollbar_flash_when_mouse_enter(&self) -> bool;
    fn enable_smooth_scroll(&self) -> bool;
    fn enable_hit_test_opaqueness(&self) -> bool;
    fn commit_fractional_scroll_deltas(&self) -> bool;
    fn enable_elastic_overscroll_for_subscroll(&self) -> bool;
    fn enable_elastic_overscroll_on_root(&self) -> bool;
}
// Exact source fields on the canonical immutable settings value.
impl InputSettings for crate::cc::trees::layer_tree_settings::LayerTreeSettings {
    fn enable_fluent_overlay_scrollbar(&self) -> bool {
        self.enable_fluent_overlay_scrollbar
    }
    fn is_for_embedded_frame(&self) -> bool {
        self.is_for_embedded_frame
    }
    fn scrollbar_flash_when_mouse_enter(&self) -> bool {
        self.scrollbar_flash_when_mouse_enter
    }
    fn enable_smooth_scroll(&self) -> bool {
        self.enable_smooth_scroll
    }
    fn enable_hit_test_opaqueness(&self) -> bool {
        self.enable_hit_test_opaqueness
    }
    fn commit_fractional_scroll_deltas(&self) -> bool {
        self.commit_fractional_scroll_deltas
    }
    fn enable_elastic_overscroll_for_subscroll(&self) -> bool {
        self.enable_elastic_overscroll_for_subscroll
    }
    fn enable_elastic_overscroll_on_root(&self) -> bool {
        self.enable_elastic_overscroll_on_root
    }
}

/// Input owns the canonical helper identity. Its queries/mutations require the
/// current InputHost/ElasticityContext, never a retained host reference.
pub trait InputElasticity {
    fn owner_id(&self) -> super::layer_tree_impl::LayerTreeHostId;
}
impl InputElasticity
    for Box<crate::cc::input::scroll_elasticity_helper::ScrollElasticityHelperImpl>
{
    fn owner_id(&self) -> super::layer_tree_impl::LayerTreeHostId {
        self.as_ref().owner_id()
    }
}
/// Type-level configuration shared by canonical InputHandler and HostImpl.
/// It contains no runtime owner and may be implemented alongside LayerTreeTypes.
pub trait InputHandlerTypes:
    InputHostTypes<
    InputHandler: crate::cc::input::compositor_input_interfaces::InputDelegateForCompositor<Self>,
    CompositorCommitData = CompositorCommitData,
    ScrollbarOrientation = ScrollbarOrientation,
    ScrollNode = InputScrollNodeId,
    LayerTreeSettings: InputSettings,
>
{
    type TraceSpan;
    type Transform: ScrollbarTransform;
    /// Canonical helper state; all tree writes use the explicit live host.
    type Elasticity: InputElasticity;
}
/// The facade carries no tree/node/layer state. The IDs passed to each method
/// resolve to the current resident allocation and are validated before access.
pub type InputScrollbarHost<'a, B> =
    dyn ScrollbarHost<Transform = <B as InputHandlerTypes>::Transform> + 'a;

/// Construction-only ownership transfer. Runtime calls borrow the existing
/// input handler and cannot simultaneously borrow its owning unique_ptr slot.
/// Source: cc/input/compositor_input_interfaces.h:131-133;
///         cc/input/input_handler.cc:101-107.
pub trait InputBindingHost<B: InputHandlerTypes> {
    fn BindToInputHandler(&mut self, delegate: Box<B::InputHandler>);
}

pub trait InputHost<B: InputHandlerTypes> {
    // This is the consumed callback subset of the canonical source interface.
    // A split facade does not expose a mutable whole-tree reference; the
    // complete interface remains implemented by HostImpl itself.
    // Source callback: cc/input/compositor_input_interfaces.h:135.
    fn ScrollAnimationAbort(&mut self, element_id: ElementId);
    // Source callback: cc/input/compositor_input_interfaces.h:136.
    fn GetBrowserControlsTopOffset(&self) -> f32;
    // Source callback: cc/input/compositor_input_interfaces.h:137.
    fn ScrollBegin(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:138-139.
    fn ScrollEnd(&mut self, compensated_scroll_delta: &Vector2dF);
    // Source callback: cc/input/compositor_input_interfaces.h:140-142.
    fn StartScrollSequence(
        &mut self,
        r#type: B::FrameSequenceTrackerType,
        scrolling_thread: B::SmoothEffectDrivingThread,
    );
    // Source callback: cc/input/compositor_input_interfaces.h:143.
    fn StopSequence(&mut self, r#type: B::FrameSequenceTrackerType);
    // Source callback: cc/input/compositor_input_interfaces.h:144.
    fn ScrollbarAnimationMouseLeave(&mut self, element_id: ElementId);
    // Source callback: cc/input/compositor_input_interfaces.h:145-147.
    fn ScrollbarAnimationMouseMove(&mut self, element_id: ElementId, device_viewport_point: PointF);
    // Source callback: cc/input/compositor_input_interfaces.h:148.
    fn ScrollbarAnimationMouseDown(&mut self, element_id: ElementId) -> bool;
    // Source callback: cc/input/compositor_input_interfaces.h:149.
    fn ScrollbarAnimationMouseUp(&mut self, element_id: ElementId) -> bool;
    // Source callback: cc/input/compositor_input_interfaces.h:150.
    fn PinchBegin(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:151.
    fn PinchEnd(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:152.
    fn SetNeedsAnimateInput(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:153-155.
    fn ScrollAnimationCreate(
        &mut self,
        scroll_node: &B::ScrollNode,
        scroll_amount: &Vector2dF,
        delayed_by: TimeDelta,
    ) -> bool;
    // Source callback: cc/input/compositor_input_interfaces.h:156.
    fn TickScrollAnimations(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:157-158.
    fn CreateLatencyInfoSwapPromiseMonitor(
        &mut self,
        latency: Option<&mut B::LatencyInfo>,
    ) -> Option<Box<B::LatencyInfoSwapPromiseMonitor>>;
    // Source callback: cc/input/compositor_input_interfaces.h:159-161.
    fn GetScopedEventMetricsMonitor(
        &mut self,
        done_callback: B::EventMetricsDoneCallback,
    ) -> Option<Box<B::EventMetricsScopedMonitor>>;
    // Source callback: cc/input/compositor_input_interfaces.h:162.
    fn DidScrollForMetrics(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:163-165.
    fn PredictViewportBoundsDelta(
        &self,
        current_bounds_delta: f64,
        scroll_distance: Vector2dF,
    ) -> f64;
    // Source callback: cc/input/compositor_input_interfaces.h:166.
    fn NotifyInputEvent(&mut self, is_fling: bool);
    // Source callback: cc/input/compositor_input_interfaces.h:167-168.
    fn ElementHasImplOnlyScrollAnimation(&self, element_id: ElementId) -> bool;
    // Source callback: cc/input/compositor_input_interfaces.h:169-173.
    fn UpdateImplAnimationScrollTargetWithDelta(
        &mut self,
        adjusted_delta: Vector2dF,
        scroll_node_id: i32,
        delayed_by: TimeDelta,
        element_id: ElementId,
    ) -> Option<PointF>;
    // Source callback: cc/input/compositor_input_interfaces.h:174.
    fn HasAnimatedScrollbars(&self) -> bool;
    // Source callback: cc/input/compositor_input_interfaces.h:175-176.
    fn SetNeedsCommit(&mut self, reason: B::BeginMainFrameReason);
    // Source callback: cc/input/compositor_input_interfaces.h:177.
    fn SetDeferBeginMainFrame(&mut self, defer_begin_main_frame: bool);
    // Source callback: cc/input/compositor_input_interfaces.h:178.
    fn DidUpdateScrollAnimationCurve(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:179.
    fn DidStartPinchZoom(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:180.
    fn DidUpdatePinchZoom(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:181.
    fn DidEndPinchZoom(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:182.
    fn DidStartScroll(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:183.
    fn DidEndScroll(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:184.
    fn DidMouseEnterNonViewportScroller(&mut self, element_id: ElementId);
    // Source callback: cc/input/compositor_input_interfaces.h:185.
    fn DidMouseLeave(&mut self);
    // Source callback: cc/input/compositor_input_interfaces.h:186.
    fn IsInHighLatencyMode(&self) -> bool;
    // Source callback: cc/input/compositor_input_interfaces.h:187.
    fn WillScrollContent(&mut self, element_id: ElementId);
    // Source callback: cc/input/compositor_input_interfaces.h:188-190.
    fn DidScrollContent(
        &mut self,
        view: &crate::cc::input::input_handler::InputHandlerView<'_>,
        element_id: ElementId,
        animated: bool,
        scroll_delta: &Vector2dF,
    );
    // Source callback: cc/input/compositor_input_interfaces.h:192.
    fn PageScaleFactor(&self) -> f32;
    // Source callback: cc/input/compositor_input_interfaces.h:193.
    fn VisualDeviceViewportSize(&self) -> Size;
    // Source callback: cc/input/compositor_input_interfaces.h:194.
    fn GetSettings(&self) -> &B::LayerTreeSettings;
    // Source callback: cc/input/compositor_input_interfaces.h:195-200.
    fn UpdateBrowserControlsState(
        &mut self,
        constraints: BrowserControlsState,
        current: BrowserControlsState,
        animate: bool,
        offset_tag_modifications: Option<&BrowserControlsOffsetTagModifications>,
    );
    // Source callback: cc/input/compositor_input_interfaces.h:201-202.
    fn HasScrollLinkedAnimation(&self, for_scroller: ElementId) -> bool;

    // Source input mutations are applied to the resident canonical objects by
    // the outer owner's split facade; no whole-tree shared-mutable owner.
    fn SetCurrentlyScrollingNode(&mut self, node: B::ScrollNode);
    fn ClearCurrentlyScrollingNode(&mut self);
    fn ScrollBy(&mut self, node: &B::ScrollNode, delta: Vector2dF);
    fn CollectScrollDeltas(
        &mut self,
        data: &mut CompositorCommitData,
        inner: ElementId,
        fractional: bool,
        snapped: BTreeMap<ElementId, TargetSnapAreaElementIds>,
        mutator: Option<&B::MutatorHost>,
        next_bmf: bool,
    );
    fn ViewportScrollBy(
        &mut self,
        view: &crate::cc::input::input_handler::InputHandlerView<'_>,
        delta: Vector2dF,
        point: Point,
        direct: bool,
        controls: bool,
        outer: bool,
        inertial: bool,
    ) -> ViewportScrollResult;
    fn ViewportScrollAnimated(
        &mut self,
        view: &crate::cc::input::input_handler::InputHandlerView<'_>,
        delta: Vector2dF,
        delayed_by: TimeDelta,
    ) -> ViewportScrollResult;
    fn ViewportPinchUpdate(
        &mut self,
        view: &crate::cc::input::input_handler::InputHandlerView<'_>,
        magnify: f32,
        anchor: Point,
    );
    fn ViewportPinchEnd(
        &mut self,
        view: &crate::cc::input::input_handler::InputHandlerView<'_>,
        anchor: Point,
        snap_to_min: bool,
    );
    fn StretchAmount(&self, helper: &B::Elasticity, id: ElementId) -> Vector2dF;
    fn ResetStretchAmounts(&mut self, helper: &mut B::Elasticity);
    fn ApplyStretchAmountsToPending(&mut self, helper: &mut B::Elasticity);
    fn ApplyStretchAmountsToActive(&mut self, helper: &mut B::Elasticity);

    fn ActiveTreeId(&mut self) -> super::layer_tree_impl::LayerTreeId;
    /// Validate `node.tree == current active tree id` and bounds before calling
    /// the closure on the actual resident ScrollNode. Never copy the node.
    fn ReadScrollNode<R>(
        &mut self,
        node: &InputScrollNodeId,
        read: impl FnOnce(&ScrollNode) -> R,
    ) -> R;
    fn WriteScrollNode<R>(
        &mut self,
        node: &InputScrollNodeId,
        write: impl FnOnce(&mut ScrollNode) -> R,
    ) -> R;
    fn LayerScreenSpaceTransform(&mut self, layer: &InputLayerId) -> B::Transform;
    fn LayerGetAllowedTouchAction(&mut self, layer: &InputLayerId, rect: Rect) -> TouchAction;
    fn LayerScrollTreeIndex(&mut self, layer: &InputLayerId) -> i32;
    fn LayerOpaqueToHitTest(&mut self, layer: &InputLayerId) -> bool;
    fn LayerIsScrollerOrScrollbar(&mut self, layer: &InputLayerId) -> bool;
    fn LayerBelongsToActiveTree(&mut self, layer: &InputLayerId) -> bool;
    fn LayerIsScrollbarLayer(&mut self, layer: &InputLayerId) -> bool;
    fn LayerScrollbarScrollElementId(&mut self, layer: &InputLayerId) -> ElementId;
    fn ActiveTree_InnerViewportScrollNode(&mut self) -> Option<InputScrollNodeId>;
    fn ActiveTree_OuterViewportScrollNode(&mut self) -> Option<InputScrollNodeId>;
    fn ActiveTree_device_scale_factor(&mut self) -> f32;
    fn ActiveTree_painted_device_scale_factor(&mut self) -> f32;
    fn ActiveTree_page_scale_factor_for_scroll(&mut self) -> f32;
    fn ActiveTree_external_page_scale_factor(&mut self) -> f32;
    fn ActiveTree_current_page_scale_factor(&mut self) -> f32;
    fn ActiveTree_min_page_scale_factor(&mut self) -> f32;
    fn ActiveTree_max_page_scale_factor(&mut self) -> f32;
    fn ActiveTree_overscroll_behavior(&mut self) -> OverscrollBehavior;
    fn ActiveTree_event_listener_properties(
        &mut self,
        class: EventListenerClass,
    ) -> EventListenerProperties;
    fn ActiveTree_FindFrameElementIdAtPoint(
        &mut self,
        view: &crate::cc::input::input_handler::InputHandlerView<'_>,
        point: PointF,
    ) -> ElementId;
    fn ActiveTree_FindLayerThatIsHitByPointInWheelEventHandlerRegion(
        &mut self,
        view: &crate::cc::input::input_handler::InputHandlerView<'_>,
        point: PointF,
    ) -> Option<InputLayerId>;
    fn ActiveTree_FindLayerThatIsHitByPointInTouchHandlerRegion(
        &mut self,
        view: &crate::cc::input::input_handler::InputHandlerView<'_>,
        rect: super::geometry::RectF,
    ) -> Option<InputLayerId>;
    fn ActiveTree_FindLayerThatIsHitByPoint(
        &mut self,
        view: &crate::cc::input::input_handler::InputHandlerView<'_>,
        point: PointF,
    ) -> Option<InputLayerId>;
    fn ActiveTree_FindLayersUpToFirstScrollableOrOpaqueToHitTest(
        &mut self,
        view: &crate::cc::input::input_handler::InputHandlerView<'_>,
        point: PointF,
    ) -> Vec<InputLayerId>;
    fn ActiveTree_PointHitsMainThreadScrollHitTestRegion(
        &mut self,
        point: PointF,
        layer: &InputLayerId,
    ) -> bool;
    fn ActiveTree_PointHitsNonCompositedScroll(
        &mut self,
        point: PointF,
        layer: &InputLayerId,
    ) -> ElementId;
    fn ActiveTree_TotalScrollOffset(&mut self) -> PointF;
    fn ActiveTree_TotalMaxScrollOffset(&mut self) -> PointF;
    fn ActiveTree_ScrollableSize(&mut self) -> SizeF;
    fn ActiveTree_LayerListIsEmpty(&mut self) -> bool;
    fn ScrollTree_CurrentlyScrollingNode(&mut self) -> Option<InputScrollNodeId>;
    fn ScrollTree_MutableFindNodeFromElementId(
        &mut self,
        id: ElementId,
    ) -> Option<InputScrollNodeId>;
    fn ScrollTree_MutableNode(&mut self, id: i32) -> InputScrollNodeId;
    fn ScrollTree_HasParent(&mut self, node: &InputScrollNodeId) -> bool;
    fn ScrollTree_MutableParent(&mut self, node: &InputScrollNodeId) -> InputScrollNodeId;
    fn ScrollTree_CanRealizeScrollsOnPendingTree(&mut self, node: &InputScrollNodeId) -> bool;
    fn ScrollTree_ShouldRealizeScrollsOnMain(&mut self, node: &InputScrollNodeId) -> bool;
    fn ScrollTree_current_scroll_offset(&mut self, id: ElementId) -> PointF;
    fn ScrollTree_ClampScrollOffsetToLimits(
        &mut self,
        offset: PointF,
        node: &InputScrollNodeId,
    ) -> PointF;
    fn ScrollTree_ScreenSpaceTransform(&mut self, id: i32) -> B::Transform;
    fn ScrollTree_outer_viewport_container_bounds_delta(&mut self) -> Vector2dF;
    fn Viewport_ShouldScroll(&mut self, node: &InputScrollNodeId) -> bool;
    fn Viewport_CanScroll(
        &mut self,
        view: &crate::cc::input::input_handler::InputHandlerView<'_>,
        node: &InputScrollNodeId,
        state: &ScrollState,
    ) -> bool;
    fn Viewport_TotalScrollOffset(&mut self) -> PointF;
    /// The facade borrows last_scroll_update_state; it must synchronously call
    /// canonical InputHandler::ComputeScrollDeltaForState for clamp queries.
    fn WithScrollbarHost<R>(
        &mut self,
        view: &crate::cc::input::input_handler::InputHandlerView<'_>,
        last_scroll_update_state: Option<&ScrollState>,
        callback: impl FnOnce(&mut InputScrollbarHost<'_, B>) -> R,
    ) -> R;
    fn CreateScrollElasticityHelper(&mut self) -> B::Elasticity;
    fn input_handler_id(&self) -> u64;
    fn Now(&self) -> TimeTicks;
    fn feature(&self, feature: InputFeature) -> bool;
    fn scroll_event_dispatch_mode(&self) -> String;
    fn late_scroll_deadline_ratio(&self) -> f64;
    fn TrackerType(&self, kind: ScrollTrackingType) -> B::FrameSequenceTrackerType;
    fn DrivingThread(&self, thread: ScrollingThread) -> B::SmoothEffectDrivingThread;
    fn ScrollBeginMainFrameReason(&self) -> B::BeginMainFrameReason;
    fn ShouldRecordSubsampledMetric(&self, probability: f64) -> bool;
    fn HistogramCounts(&self, name: &str, value: f32);
    fn HistogramTimes(&self, name: &str, value: TimeDelta);
    fn BeginTrace(&self, event: &str, fields: &[InputTraceField]) -> B::TraceSpan;
    fn InstantTrace(&self, event: &str, fields: &[InputTraceField]);
}

#[derive(Clone, Copy, Debug)]
pub enum InputTraceValue {
    Float(f64),
    Bool(bool),
    Granularity(super::input_types::ScrollGranularity),
}
pub type InputTraceField = (&'static str, InputTraceValue);
