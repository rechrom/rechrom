//! The cc producer facets of viz::CompositorFrameMetadata. The metadata
//! values are components/viz/common (`viz_common::compositor_frame_metadata`);
//! all source decisions remain in LayerTreeHostImpl; setters write exactly
//! one field.
use crate::adapter::{
    geometry::{PointF, Size, SizeF, Vector2dF},
    input_services::OffsetTag,
    layer_tree_host_impl::{
        ContentColorUsage, HostCompositorFrameMetadata, HostContentFrameIntervalType,
        HostImplDTypes, HostImplFrameMetadata,
    },
    layer_tree_impl::{Color4f, OverlayTransform, UnguessableToken},
    time::{TimeDelta, TimeTicks},
};
pub use viz_common::compositor_frame_metadata::{
    CompositorFrameMetadata, ContentFrameIntervalInfo, DelegatedInkMetadata, FrameDeadline,
    FrameIntervalInputs, FrameMetadataValueTypes, MetadataValues, OffsetTagConstraints,
    OffsetTagDefinition, OffsetTagValue, SharedTransitionElement, TransitionDirective,
    TransitionDirectiveType, TreesInVizTiming,
};
// The viz values a main Layer carries into the commit.
pub use viz_common::surfaces::{
    FrameToken, RegionCaptureBounds, RegionCaptureCropId, TrackedElementFeature,
    TrackedElementRect, TrackedElementRects,
};

/// Native zero-invalid token generator, incremented by the host's external
/// value service; no scheduling or submission policy is implemented here.
#[derive(Default)]
pub struct FrameTokenGenerator {
    frame_token: u32,
}
impl FrameTokenGenerator {
    pub fn Next(&mut self) -> u32 {
        assert!(
            self.frame_token < u32::MAX,
            "native FrameTokenGenerator overflow CHECK"
        );
        self.frame_token += 1;
        self.frame_token
    }
    pub fn current(&self) -> u32 {
        self.frame_token
    }
}

/// Bind the root family to exactly the same native values as the output frame.
pub trait MetadataTypes: HostImplDTypes<TreesInVizTiming = TreesInVizTiming> {
    type MetadataValues: FrameMetadataValueTypes<
        MetadataSurfaceRange = <Self as layerize::commit::CommitTypes>::SurfaceRange,
        MetadataSurfaceId = <Self as crate::adapter::frame_data::FrameDataTypes>::SurfaceId,
        MetadataDeadline = <Self as HostImplDTypes>::FrameDeadline,
        MetadataDelegatedInk = <Self as layerize::commit::CommitTypes>::DelegatedInkMetadata,
        MetadataTransition = <Self as HostImplDTypes>::TransitionDirective,
        MetadataCaptureBounds = <Self as HostImplDTypes>::RegionCaptureBounds,
        MetadataTrackedRects = <Self as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects,
        MetadataLatency = <Self as crate::adapter::input_services::InputHostTypes>::LatencyInfo,
        MetadataAck = <Self as crate::adapter::frame_data::FrameDataTypes>::BeginFrameAck,
    >;
}
impl<V: FrameMetadataValueTypes> HostImplFrameMetadata for CompositorFrameMetadata<V> {
    fn set_content_color_usage(&mut self, usage: ContentColorUsage) {
        self.content_color_usage = usage;
    }
    fn content_color_usage(&self) -> ContentColorUsage {
        self.content_color_usage
    }
}
impl<B: MetadataTypes> HostCompositorFrameMetadata<B> for CompositorFrameMetadata<B::MetadataValues> {
    fn frame_token(&self) -> u32 {
        self.frame_token
    }
    fn top_controls_visible_height(&self) -> Option<f32> {
        self.top_controls_visible_height
    }
    fn send_frame_token_to_embedder(&self) -> bool {
        self.send_frame_token_to_embedder
    }
    fn set_frame_token(&mut self, value: u32) {
        self.frame_token = value;
    }
    fn set_device_scale_factor(&mut self, value: f32) {
        self.device_scale_factor = value;
    }
    fn set_page_scale_factor(&mut self, value: f32) {
        self.page_scale_factor = value;
    }
    fn set_scrollable_viewport_size(&mut self, value: SizeF) {
        self.scrollable_viewport_size = value;
    }
    fn set_visible_viewport_size(&mut self, value: Size) {
        self.visible_viewport_size = value;
    }
    fn set_root_background_color(&mut self, value: Color4f) {
        self.root_background_color = value;
    }
    fn set_may_throttle_if_undrawn_frames(&mut self, value: bool) {
        self.may_throttle_if_undrawn_frames = value;
    }
    fn set_is_handling_interaction(&mut self, value: bool) {
        self.is_handling_interaction = value;
    }
    fn set_is_handling_animation(&mut self, value: bool) {
        self.is_handling_animation = value;
    }
    fn set_min_page_scale_factor(&mut self, value: f32) {
        self.min_page_scale_factor = value;
    }
    fn set_root_scroll_offset(&mut self, value: PointF) {
        self.root_scroll_offset = value;
    }
    fn set_display_transform_hint(&mut self, value: OverlayTransform) {
        self.display_transform_hint = value;
    }
    fn set_is_mobile_optimized(&mut self, value: bool) {
        self.is_mobile_optimized = value;
    }
    fn set_capture_bounds(&mut self, value: B::RegionCaptureBounds) {
        self.capture_bounds = value;
    }
    fn set_tracked_element_rects(&mut self, value: B::TrackedElementRects) {
        self.tracked_element_rects = value;
    }
    fn set_is_software(&mut self, value: bool) {
        self.is_software = value;
    }
    fn set_has_shared_element_resources(&mut self, value: bool) {
        self.has_shared_element_resources = value;
    }
    fn set_deadline(&mut self, value: B::FrameDeadline) {
        self.deadline = value;
    }
    fn set_trees_in_viz_timing_details(&mut self, value: TreesInVizTiming) {
        self.trees_in_viz_timing_details = value;
    }
    fn set_activation_dependencies(&mut self, value: Vec<B::SurfaceId>) {
        self.activation_dependencies = value;
    }
    fn set_send_frame_token_to_embedder(&mut self, value: bool) {
        self.send_frame_token_to_embedder = value;
    }
    fn set_begin_frame_ack(&mut self, value: B::BeginFrameAck) {
        self.begin_frame_ack = value;
    }
    fn referenced_surfaces_mut(&mut self) -> &mut Vec<B::SurfaceRange> {
        &mut self.referenced_surfaces
    }
    fn set_top_controls_visible_height(&mut self, value: f32) {
        self.top_controls_visible_height = Some(value);
    }
    fn AppendOffsetTagValue(&mut self, tag: OffsetTag, value: Vector2dF) {
        self.offset_tag_values
            .push(OffsetTagValue { tag, offset: value });
    }
    fn delegated_ink_metadata(&self) -> Option<&B::DelegatedInkMetadata> {
        self.delegated_ink_metadata.as_deref()
    }
    fn set_delegated_ink_metadata(&mut self, value: Box<B::DelegatedInkMetadata>) {
        self.delegated_ink_metadata = Some(value);
    }
    fn TakeTrackedElementRects(&mut self) -> B::TrackedElementRects {
        std::mem::take(&mut self.tracked_element_rects)
    }
    fn set_screenshot_destination(&mut self, value: UnguessableToken) {
        self.screenshot_destination = Some(value);
    }
    fn transition_directives_mut(&mut self) -> &mut Vec<B::TransitionDirective> {
        &mut self.transition_directives
    }
    fn set_interval_frame_time(&mut self, value: TimeTicks) {
        self.frame_interval_inputs.frame_time = value;
    }
    fn set_interval_has_input(&mut self, value: bool) {
        self.frame_interval_inputs.has_input = value;
    }
    fn set_interval_has_user_input(&mut self, value: bool) {
        self.frame_interval_inputs.has_user_input = value;
    }
    fn set_major_scroll_speed(&mut self, value: f32) {
        self.frame_interval_inputs
            .major_scroll_speed_in_pixels_per_second = value;
    }
    fn AppendContentInterval(
        &mut self,
        kind: HostContentFrameIntervalType,
        interval: TimeDelta,
        duplicate_count: u32,
    ) {
        self.frame_interval_inputs
            .content_interval_info
            .push(ContentFrameIntervalInfo {
                kind,
                frame_interval: interval,
                duplicate_count,
            });
    }
    fn set_has_only_content_frame_interval_updates(&mut self, value: bool) {
        self.frame_interval_inputs
            .has_only_content_frame_interval_updates = value;
    }
    fn latency_info(&self) -> &[B::LatencyInfo] {
        &self.latency_info
    }
    fn latency_info_mut(&mut self) -> &mut Vec<B::LatencyInfo> {
        &mut self.latency_info
    }
    fn TakeLatencyInfo(&mut self) -> Vec<B::LatencyInfo> {
        std::mem::take(&mut self.latency_info)
    }
    fn begin_frame_ack(&self) -> &B::BeginFrameAck {
        &self.begin_frame_ack
    }
    fn set_submit_compositor_frame_time(&mut self, value: TimeTicks) {
        self.trees_in_viz_timing_details.submit_compositor_frame = value;
    }
}

// Source dependency: ui/gfx/selection_bound.h:24-80; selection_bound.cc:15-57.
#[derive(Clone, Debug, Default)]
pub struct SelectionBound {
    pub kind: crate::adapter::layer_tree_impl::SelectionBoundType,
    pub edge_start: PointF,
    pub edge_end: PointF,
    pub visible_edge_start: PointF,
    pub visible_edge_end: PointF,
    pub edge_start_rounded: crate::adapter::geometry::Point,
    pub edge_end_rounded: crate::adapter::geometry::Point,
    pub visible: bool,
}
impl PartialEq for SelectionBound {
    fn eq(&self, rhs: &Self) -> bool {
        self.kind == rhs.kind
            && self.visible == rhs.visible
            && self.edge_start == rhs.edge_start
            && self.edge_end == rhs.edge_end
            && self.visible_edge_start == rhs.visible_edge_start
            && self.visible_edge_end == rhs.visible_edge_end
    }
}
impl crate::adapter::layer_tree_impl::LayerTreeSelectionBound for SelectionBound {
    fn r#type(&self) -> crate::adapter::layer_tree_impl::SelectionBoundType {
        self.kind
    }
    fn visible(&self) -> bool {
        self.visible
    }
    fn edge_start(&self) -> PointF {
        self.edge_start
    }
    fn edge_end(&self) -> PointF {
        self.edge_end
    }
    fn SetEdgeStart(&mut self, point: PointF) {
        self.edge_start = point;
        self.edge_start_rounded = crate::adapter::geometry::Point::new(
            point.x().round() as i32,
            point.y().round() as i32,
        );
    }
    fn SetEdgeEnd(&mut self, point: PointF) {
        self.edge_end = point;
        self.edge_end_rounded = crate::adapter::geometry::Point::new(
            point.x().round() as i32,
            point.y().round() as i32,
        );
    }
    fn SetVisibleEdgeStart(&mut self, point: PointF) {
        self.visible_edge_start = point;
    }
    fn SetVisibleEdgeEnd(&mut self, point: PointF) {
        self.visible_edge_end = point;
    }
    fn set_type(&mut self, value: crate::adapter::layer_tree_impl::SelectionBoundType) {
        self.kind = value;
    }
    fn SetEdge(&mut self, start: PointF, end: PointF) {
        self.SetEdgeStart(start);
        self.SetEdgeEnd(end);
    }
    fn SetVisibleEdge(&mut self, start: PointF, end: PointF) {
        self.SetVisibleEdgeStart(start);
        self.SetVisibleEdgeEnd(end);
    }
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}
impl crate::adapter::layer_tree_host_impl::HostImplTrackedElementRect for TrackedElementRect {
    fn visible_bounds(&self) -> crate::adapter::geometry::Rect {
        self.visible_bounds
    }
    fn should_exclude_fixed_and_sticky_occlusions(&self) -> bool {
        self.should_exclude_fixed_and_sticky_occlusions
    }
}
impl crate::adapter::layer_tree_host_impl::HostMetadataTrackedElementRect for TrackedElementRect {
    fn should_add_to_compositor_frame_metadata(&self) -> bool {
        self.should_add_to_compositor_frame_metadata
    }
    fn set_visible_bounds(&mut self, bounds: crate::adapter::geometry::Rect) {
        self.visible_bounds = bounds;
    }
}
impl<
        B: HostImplDTypes<
            TrackedElementFeature = TrackedElementFeature,
            TrackedElementRect = TrackedElementRect,
        >,
    > crate::adapter::layer_tree_host_impl::HostTrackedElementRects<B> for TrackedElementRects
{
    type Features<'a>
        = std::iter::Map<
        std::collections::btree_map::Iter<'a, TrackedElementFeature, Vec<TrackedElementRect>>,
        fn(
            (&'a TrackedElementFeature, &'a Vec<TrackedElementRect>),
        ) -> (&'a TrackedElementFeature, &'a [TrackedElementRect]),
    >
    where
        Self: 'a,
        B: 'a;
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn iter(&self) -> Self::Features<'_> {
        self.0.iter().map(|(key, value)| (key, value.as_slice()))
    }
    fn Append(&mut self, feature: TrackedElementFeature, rect: TrackedElementRect) {
        self.0.entry(feature).or_default().push(rect);
    }
}
impl<B: HostImplDTypes<RegionCaptureId = RegionCaptureCropId>>
    crate::adapter::layer_tree_host_impl::HostRegionCaptureBounds<B> for RegionCaptureBounds
{
    type Bounds<'a>
        = std::collections::btree_map::Iter<'a, RegionCaptureCropId, crate::adapter::geometry::Rect>
    where
        Self: 'a,
        B: 'a;
    fn bounds(&self) -> Self::Bounds<'_> {
        self.0.iter()
    }
    fn Set(&mut self, id: RegionCaptureCropId, bounds: crate::adapter::geometry::Rect) {
        self.0.insert(id, bounds);
    }
}
// Source dependency: ui/gfx/presentation_feedback.h:21-109.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PresentationFeedback {
    pub timestamp: TimeTicks,
    pub interval: TimeDelta,
    pub flags: u32,
    pub available_timestamp: TimeTicks,
    pub ready_timestamp: TimeTicks,
    pub latch_timestamp: TimeTicks,
    pub writes_done_timestamp: TimeTicks,
    #[cfg(target_vendor = "apple")]
    pub ca_layer_error_code: i32,
    pub display_trace_id: Option<i64>,
}
// Source dependency: ui/gfx/swap_result.h:32-57.
#[derive(Clone, Debug, Default)]
pub struct SwapTimings {
    pub swap_start: TimeTicks,
    pub swap_end: TimeTicks,
    pub viz_scheduled_draw: TimeTicks,
    pub gpu_started_draw: TimeTicks,
    pub gpu_task_ready: TimeTicks,
    pub gpu_started_overlay: TimeTicks,
}
// Source dependency: components/viz/common/frame_timing_details.h:15-41.
#[derive(Clone, Debug, Default)]
pub struct FrameTimingDetails {
    pub received_compositor_frame_timestamp: TimeTicks,
    pub embedded_frame_timestamp: TimeTicks,
    pub draw_start_timestamp: TimeTicks,
    pub swap_timings: SwapTimings,
    pub presentation_feedback: PresentationFeedback,
    pub frame_id: crate::adapter::begin_frame::BeginFrameId,
    pub start_update_display_tree: TimeTicks,
    pub start_prepare_to_draw: TimeTicks,
    pub start_draw_layers: TimeTicks,
    pub submit_compositor_frame: TimeTicks,
}
impl crate::adapter::layer_tree_host_impl::HostFrameTimingDetails for FrameTimingDetails {
    fn PresentationTimestamp(&self) -> TimeTicks {
        self.presentation_feedback.timestamp
    }
}

impl<R: super::types::CpuSkiaBackend> crate::adapter::render_metadata::RenderFrameMetadataTypes
    for super::types::CpuCompositorTypes<R>
{
    type Selection = crate::adapter::layer_tree_impl::Selection<SelectionBound>;
    type LocalSurfaceId = super::types::LocalSurfaceId;
    type TrackedElementRects = TrackedElementRects;
}

