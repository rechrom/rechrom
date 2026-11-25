//! components/viz/common: the values a CompositorFrame carries from cc to
//! Viz and the display renderer. The producer (cc impl side) fills them; Viz
//! and the renderer read them after the frame is moved. No layerization,
//! tiling, aggregation or drawing logic lives here; the cc packages implement
//! their own facets of these values next to their traits.

// Chromium identifiers keep their source spelling.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod compositor_frame_metadata;
pub mod compositor_render_pass;
pub mod copy_output_request;
pub mod draw_quad;
pub mod frame_timing_details;
pub mod latency_info;
pub mod offset_tag;
pub mod raster_resource_lease;
pub mod returned_resource;
pub mod shared_image;
pub mod surfaces;
pub mod transferable_resource;

/// components/viz/common/frame_sinks/begin_frame_args.h (BeginFrameArgs,
/// BeginFrameAck), shared with the cc scheduler.
pub use compositor_values::begin_frame;

// Source: components/viz/common/resources/resource_id.h.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

// Source dependency: third_party/blink/public/common/tokens/tokens.h
// ViewTransitionToken (base::UnguessableToken halves), carried by
// CompositorFrameTransitionDirective.
pub type ViewTransitionToken = [u64; 2];

/// The production software frame family: gfx::Transform over the SkRegion
/// value backend, cc::FilterOperations, SkPath, viz::CopyOutputRequest and
/// the cc::DisplayItemList PictureDrawQuad holds.
pub type FrameTransform =
    compositor_values::transform::Transform<cc_paint::skia_region::SkRegion>;
pub type SoftwareFrameValues = compositor_render_pass::FrameValues<
    FrameTransform,
    paint_model::cc::paint::filter_operations::FilterOperations,
    cc_paint::skia_region::SkPath,
    copy_output_request::CopyOutputRequest,
    cc_paint::ContentDisplayItemList<cc_paint::skia_region::SkRegion>,
>;
pub type SoftwareMetadataValues = compositor_frame_metadata::MetadataValues<
    surfaces::SurfaceRange,
    surfaces::SurfaceId,
    compositor_frame_metadata::FrameDeadline,
    compositor_frame_metadata::DelegatedInkMetadata,
    compositor_frame_metadata::TransitionDirective,
    surfaces::RegionCaptureBounds,
    surfaces::TrackedElementRects,
    latency_info::LatencyInfo,
    begin_frame::BeginFrameAck,
>;
pub type CompositorRenderPass = compositor_render_pass::CompositorRenderPass<SoftwareFrameValues>;
pub type CompositorFrameMetadata =
    compositor_frame_metadata::CompositorFrameMetadata<SoftwareMetadataValues>;
pub type TransferableResource = transferable_resource::RasterTransferableResource;
pub type CompositorFrame = compositor_render_pass::CompositorFrame<
    CompositorFrameMetadata,
    CompositorRenderPass,
    TransferableResource,
>;
