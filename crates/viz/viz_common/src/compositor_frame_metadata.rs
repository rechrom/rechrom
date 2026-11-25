//! viz::CompositorFrameMetadata and the values it carries
//! (components/viz/common/quads/compositor_frame_metadata.h). This is the
//! external Viz value boundary; all source decisions remain in
//! LayerTreeHostImpl, whose metadata facets the compositor implements.
use crate::draw_quad::RenderPassId;
use crate::offset_tag::OffsetTag;
use crate::ViewTransitionToken;
use compositor_tree::geometry::{PointF, RectF, Size, SizeF, Vector2dF};
use compositor_tree::property_types::ViewTransitionElementResourceId;
use compositor_values::begin_frame::BeginFrameArgs;
use compositor_values::color4f::Color4f;
use compositor_values::display_color_spaces::RasterDisplayColorSpaces;
use compositor_values::overlay_transform::OverlayTransform;
use compositor_values::surface_id::ContentColorUsage;
use compositor_values::unguessable_token::UnguessableToken;
use scroll_animation::{TimeDelta, TimeTicks};

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
// Source: components/viz/common/quads/frame_deadline.h.
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
            frame_interval: BeginFrameArgs::DefaultInterval(),
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
// Source: components/viz/common/quads/offset_tag.h.
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

/// The value types one metadata instance carries.
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
// Source: components/viz/common/quads/frame_interval_inputs.h:31-36.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentFrameIntervalType {
    kVideo,
    kAnimatingImage,
    kScrollBarFadeOutAnimation,
    kCompositorScroll,
}
#[derive(Clone, Copy, Debug)]
pub struct ContentFrameIntervalInfo {
    pub kind: ContentFrameIntervalType,
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

// Source dependency: ui/gfx/delegated_ink_metadata.h:31-124. This is the native
// immutable drawing/IPC value, not a copy of compositor or paint owner state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DelegatedInkMetadata {
    pub point: PointF,
    pub diameter: f64,
    pub color: u32,
    pub timestamp: TimeTicks,
    pub presentation_area: RectF,
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
    pub render_pass_id: RenderPassId,
    pub view_transition_element_resource_id: ViewTransitionElementResourceId,
}
#[derive(Clone, Debug, Default)]
pub struct TransitionDirective {
    pub transition_token: ViewTransitionToken,
    pub maybe_cross_frame_sink: bool,
    pub sequence_id: u32,
    pub kind: TransitionDirectiveType,
    pub shared_elements: Vec<SharedTransitionElement>,
    pub display_color_spaces: RasterDisplayColorSpaces,
    pub delay_layer_tree_view_deletion: bool,
}
