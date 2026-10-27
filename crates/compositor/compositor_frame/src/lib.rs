//! Immutable compositor-frame output values passed to Viz.
//!
//! Frame values do not retain compositor tree, tile, scheduler, or raster-job
//! owners.

pub mod draw_submission;
pub mod frame_data;
pub mod frame_data_boundary;
pub mod frame_generation;
pub mod metadata_build;
pub mod metadata_interval_policy;
pub mod presentation_callback_buffer;
pub mod render_frame_metadata;
pub mod render_metadata;
pub mod submit_info;
pub mod swap_promise;
pub mod unbounded;
pub mod unbounded_frame_sink_handler;
pub mod view_transition_metadata;
pub mod view_transition_requests;

pub struct CompositorFrameSubmission<LocalSurfaceId, Frame> {
    pub local_surface_id: LocalSurfaceId,
    pub frame: Frame,
}
