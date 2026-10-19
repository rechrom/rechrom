//! Native owned frame metadata values. This is the external Viz value boundary,
//! not a compositor algorithm or translation-coverage claim. All source
//! decisions remain in LayerTreeHostImpl; setters write exactly one field.
use crate::adapter::{
    geometry::{PointF, Size, SizeF, Vector2dF},
    input_services::OffsetTag,
    layer_tree_host_impl::{
        ContentColorUsage, HostCompositorFrameMetadata, HostContentFrameIntervalType,
        HostImplDTypes, HostImplFrameMetadata,
    },
    layer_tree_impl::{Color4f, LayerTreeTypes, OverlayTransform, UnguessableToken},
    time::{TimeDelta, TimeTicks},
};

/// External components/viz/common/quads/trees_in_viz_timing.h value. No clock
/// is read here; canonical owner calls write the injected timestamps.
#[derive(Clone, Copy, Debug, Default)]
pub struct TreesInVizTiming {
    pub start_update_display_tree: TimeTicks,
    pub start_prepare_to_draw: TimeTicks,
    pub start_draw_layers: TimeTicks,
    pub submit_compositor_frame: TimeTicks,
}
/// Native Viz field value; this does not read a clock or select a deadline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameDeadline {
    pub frame_start_time: TimeTicks,
    pub deadline_in_frames: u32,
    pub frame_interval: TimeDelta,
    pub use_default_lower_bound_deadline: bool,
}
impl Default for FrameDeadline {
    fn default() -> Self {
        Self {
            frame_start_time: TimeTicks::default(),
            deadline_in_frames: 0,
            frame_interval: crate::adapter::begin_frame::BeginFrameArgs::DefaultInterval(),
            use_default_lower_bound_deadline: true,
        }
    }
}
impl FrameDeadline {
    pub fn new(
        frame_start_time: TimeTicks,
        deadline_in_frames: u32,
        frame_interval: TimeDelta,
        use_default_lower_bound_deadline: bool,
    ) -> Self {
        Self {
            frame_start_time,
            deadline_in_frames,
            frame_interval,
            use_default_lower_bound_deadline,
        }
    }
}
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
#[derive(Clone, Copy, Debug, Default)]
pub struct OffsetTagConstraints {
    pub min_offset: Vector2dF,
    pub max_offset: Vector2dF,
}
pub struct OffsetTagDefinition<R> {
    pub tag: OffsetTag,
    pub provider: R,
    pub constraints: OffsetTagConstraints,
}
pub struct OffsetTagValue {
    pub tag: OffsetTag,
    pub offset: Vector2dF,
}

pub trait FrameMetadataValueTypes {
    type MetadataSurfaceRange;
    type MetadataSurfaceId;
    type MetadataDeadline: Default;
    type MetadataDelegatedInk;
    type MetadataTransition;
    type MetadataCaptureBounds: Default;
    type MetadataTrackedRects: Default;
    type MetadataLatency;
    type MetadataAck: Default;
}
pub struct MetadataValues<SR, SID, FD, INK, TD, CB, TR, LI, ACK>(
    std::marker::PhantomData<(SR, SID, FD, INK, TD, CB, TR, LI, ACK)>,
);
impl<SR, SID, FD: Default, INK, TD, CB: Default, TR: Default, LI, ACK: Default>
    FrameMetadataValueTypes for MetadataValues<SR, SID, FD, INK, TD, CB, TR, LI, ACK>
{
    type MetadataSurfaceRange = SR;
    type MetadataSurfaceId = SID;
    type MetadataDeadline = FD;
    type MetadataDelegatedInk = INK;
    type MetadataTransition = TD;
    type MetadataCaptureBounds = CB;
    type MetadataTrackedRects = TR;
    type MetadataLatency = LI;
    type MetadataAck = ACK;
}
/// Bind the root family to exactly the same native values as the output frame.
pub trait NativeMetadataTypes:
    HostImplDTypes<TreesInVizTiming=TreesInVizTiming> + FrameMetadataValueTypes<
        MetadataSurfaceRange=<Self as LayerTreeTypes>::SurfaceRange,
        MetadataSurfaceId=<Self as crate::adapter::frame_data::FrameDataTypes>::SurfaceId,
        MetadataDeadline=<Self as HostImplDTypes>::FrameDeadline,
        MetadataDelegatedInk=<Self as LayerTreeTypes>::DelegatedInkMetadata,
        MetadataTransition=<Self as HostImplDTypes>::TransitionDirective,
        MetadataCaptureBounds=<Self as HostImplDTypes>::RegionCaptureBounds,
        MetadataTrackedRects=<Self as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects,
        MetadataLatency=<Self as crate::adapter::input_services::InputHostTypes>::LatencyInfo,
        MetadataAck=<Self as crate::adapter::frame_data::FrameDataTypes>::BeginFrameAck,
    > {}
#[derive(Clone, Copy, Debug)]
pub struct ContentFrameIntervalInfo {
    pub kind: HostContentFrameIntervalType,
    pub frame_interval: TimeDelta,
    pub duplicate_count: u32,
}
#[derive(Default)]
pub struct FrameIntervalInputs {
    pub frame_time: TimeTicks,
    pub has_input: bool,
    pub has_user_input: bool,
    pub major_scroll_speed_in_pixels_per_second: f32,
    pub content_interval_info: Vec<ContentFrameIntervalInfo>,
    pub has_only_content_frame_interval_updates: bool,
}
/// Move-only native payload: copy requests/delegated ink and transition values
/// are not cloned when a frame is submitted or metadata is replaced.
pub struct CompositorFrameMetadata<V: FrameMetadataValueTypes> {
    pub device_scale_factor: f32,
    pub root_scroll_offset: PointF,
    pub page_scale_factor: f32,
    pub scrollable_viewport_size: SizeF,
    pub visible_viewport_size: Size,
    pub content_color_usage: ContentColorUsage,
    pub may_contain_video: bool,
    pub may_throttle_if_undrawn_frames: bool,
    pub is_handling_interaction: bool,
    pub is_handling_animation: bool,
    pub root_background_color: Color4f,
    pub latency_info: Vec<V::MetadataLatency>,
    pub referenced_surfaces: Vec<V::MetadataSurfaceRange>,
    pub activation_dependencies: Vec<V::MetadataSurfaceId>,
    pub deadline: V::MetadataDeadline,
    pub begin_frame_ack: V::MetadataAck,
    pub frame_token: u32,
    pub send_frame_token_to_embedder: bool,
    pub min_page_scale_factor: f32,
    pub top_controls_visible_height: Option<f32>,
    pub display_transform_hint: OverlayTransform,
    pub is_mobile_optimized: bool,
    pub delegated_ink_metadata: Option<Box<V::MetadataDelegatedInk>>,
    pub transition_directives: Vec<V::MetadataTransition>,
    pub capture_bounds: V::MetadataCaptureBounds,
    pub has_shared_element_resources: bool,
    pub screenshot_destination: Option<UnguessableToken>,
    pub is_software: bool,
    pub offset_tag_definitions: Vec<OffsetTagDefinition<V::MetadataSurfaceRange>>,
    pub offset_tag_values: Vec<OffsetTagValue>,
    pub frame_interval_inputs: FrameIntervalInputs,
    pub trees_in_viz_timing_details: TreesInVizTiming,
    pub tracked_element_rects: V::MetadataTrackedRects,
}
impl<V: FrameMetadataValueTypes> Default for CompositorFrameMetadata<V> {
    fn default() -> Self {
        Self {
            device_scale_factor: 0.,
            root_scroll_offset: PointF::default(),
            page_scale_factor: 0.,
            scrollable_viewport_size: SizeF::default(),
            visible_viewport_size: Size::default(),
            content_color_usage: ContentColorUsage::kSRGB,
            may_contain_video: false,
            may_throttle_if_undrawn_frames: true,
            is_handling_interaction: false,
            is_handling_animation: false,
            root_background_color: Color4f {
                red: 1.,
                green: 1.,
                blue: 1.,
                alpha: 1.,
            },
            latency_info: Vec::new(),
            referenced_surfaces: Vec::new(),
            activation_dependencies: Vec::new(),
            deadline: V::MetadataDeadline::default(),
            begin_frame_ack: V::MetadataAck::default(),
            frame_token: 0,
            send_frame_token_to_embedder: false,
            min_page_scale_factor: 0.,
            top_controls_visible_height: None,
            display_transform_hint: OverlayTransform::None,
            is_mobile_optimized: false,
            delegated_ink_metadata: None,
            transition_directives: Vec::new(),
            capture_bounds: V::MetadataCaptureBounds::default(),
            has_shared_element_resources: false,
            screenshot_destination: None,
            is_software: false,
            offset_tag_definitions: Vec::new(),
            offset_tag_values: Vec::new(),
            frame_interval_inputs: FrameIntervalInputs::default(),
            trees_in_viz_timing_details: TreesInVizTiming::default(),
            tracked_element_rects: V::MetadataTrackedRects::default(),
        }
    }
}
impl<V: FrameMetadataValueTypes> HostImplFrameMetadata for CompositorFrameMetadata<V> {
    fn set_content_color_usage(&mut self, usage: ContentColorUsage) {
        self.content_color_usage = usage;
    }
    fn content_color_usage(&self) -> ContentColorUsage {
        self.content_color_usage
    }
}
impl<B: NativeMetadataTypes> HostCompositorFrameMetadata<B> for CompositorFrameMetadata<B> {
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
// Source dependency: components/viz/common/surfaces/tracked_element_rects.h:22-34.
// Browser-maintained feature numbers intentionally remain accepted values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrackedElementFeature(pub i32);
impl TrackedElementFeature {
    pub const kIframeTracking: Self = Self(0);
    pub const kPasswordTracking: Self = Self(1);
    pub const kTrackedElementFeatureEnd: Self = Self(2);
    pub const kTrackedElementFeatureMax: Self = Self(4);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameToken {
    Local([u64; 2]),
    Remote([u64; 2]),
}
// Source dependency: components/viz/common/surfaces/tracked_element_rects.h:37-85.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrackedElementRect {
    pub id: [u64; 2],
    pub visible_bounds: crate::adapter::geometry::Rect,
    pub should_add_to_compositor_frame_metadata: bool,
    pub should_exclude_fixed_and_sticky_occlusions: bool,
    pub frame_token: Option<FrameToken>,
    pub parent_frame_token: Option<[u64; 2]>,
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrackedElementRects(
    pub std::collections::BTreeMap<TrackedElementFeature, Vec<TrackedElementRect>>,
);
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
// Source dependency: components/viz/common/surfaces/region_capture_bounds.h:20-62.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegionCaptureCropId(pub [u64; 2]);
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RegionCaptureBounds(
    pub std::collections::BTreeMap<RegionCaptureCropId, crate::adapter::geometry::Rect>,
);
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

impl crate::adapter::main_layer::LayerValue for TrackedElementRects {
    fn IsEmpty(&self) -> bool {
        self.0.is_empty()
    }
}
impl crate::adapter::main_layer::LayerValue for RegionCaptureBounds {
    fn IsEmpty(&self) -> bool {
        self.0.is_empty()
    }
}
impl<R: super::types::CpuSkiaBackend> crate::adapter::render_metadata::RenderFrameMetadataTypes
    for super::types::CpuCompositorTypes<R>
{
    type Selection = crate::adapter::layer_tree_impl::Selection<SelectionBound>;
    type LocalSurfaceId = super::types::LocalSurfaceId;
    type TrackedElementRects = TrackedElementRects;
}

// Source dependency: ui/gfx/delegated_ink_metadata.h:31-124. This is the native
// immutable drawing/IPC value, not a copy of compositor or paint owner state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DelegatedInkMetadata {
    pub point: PointF,
    pub diameter: f64,
    pub color: u32,
    pub timestamp: TimeTicks,
    pub presentation_area: crate::adapter::geometry::RectF,
    pub frame_time: TimeTicks,
    pub is_hovering: bool,
    pub render_pass_id: u64,
}
// Source dependency: components/viz/common/quads/compositor_frame_transition_directive.h:42-43,45-67,136-154.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TransitionDirectiveType {
    #[default]
    kSave,
    kAnimateRenderer,
    kRelease,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SharedTransitionElement {
    pub render_pass_id: super::frame::RenderPassId,
    pub view_transition_element_resource_id:
        crate::adapter::property_types::ViewTransitionElementResourceId,
}
#[derive(Clone, Debug, Default)]
pub struct TransitionDirective {
    pub transition_token: crate::adapter::draw_property::ViewTransitionToken,
    pub maybe_cross_frame_sink: bool,
    pub sequence_id: u32,
    pub kind: TransitionDirectiveType,
    pub shared_elements: Vec<SharedTransitionElement>,
    pub display_color_spaces: super::resources::CpuDisplayColorSpaces,
    pub delay_layer_tree_view_deletion: bool,
}
impl<R: super::types::CpuSkiaBackend> FrameMetadataValueTypes
    for super::types::CpuCompositorTypes<R>
{
    type MetadataSurfaceRange = super::types::SurfaceRange;
    type MetadataSurfaceId = super::types::SurfaceId;
    type MetadataDeadline = FrameDeadline;
    type MetadataDelegatedInk = DelegatedInkMetadata;
    type MetadataTransition = TransitionDirective;
    type MetadataCaptureBounds = RegionCaptureBounds;
    type MetadataTrackedRects = TrackedElementRects;
    type MetadataLatency = super::input::LatencyInfo;
    type MetadataAck = super::frame::BeginFrameAck;
}
