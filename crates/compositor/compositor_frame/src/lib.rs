//! Immutable compositor-frame output values passed to Viz.
//!
//! Frame values do not retain compositor tree, tile, scheduler, or raster-job
//! owners.

// Chromium identifiers keep their source spelling.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod draw_submission;
pub mod frame_data;
pub mod frame_data_boundary;
pub mod frame_generation;
pub mod metadata_build;
pub mod metadata_interval_policy;
pub mod presentation_callback_buffer;
// components/viz/common/quads values (viz_common).
pub use viz_common::draw_quad as quads;
pub mod render_frame_metadata;
pub mod render_metadata;
pub mod submit_info;
pub use compositor_values::surface_id;
pub mod swap_promise;
pub mod unbounded;
pub mod unbounded_frame_sink_handler;
pub mod view_transition_metadata;
pub mod view_transition_requests;
pub mod viz_frame;

pub struct CompositorFrameSubmission<LocalSurfaceId, Frame> {
    pub local_surface_id: LocalSurfaceId,
    pub frame: Frame,
}
