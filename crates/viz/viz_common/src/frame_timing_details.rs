//! The presentation timing values Viz returns for a submitted frame
//! (DidPresentCompositorFrame): gfx::PresentationFeedback, gfx::SwapTimings
//! and viz::FrameTimingDetails. Values only; the cc impl side fills them and
//! the main side's presentation callbacks read them.
use compositor_values::begin_frame::BeginFrameId;
use scroll_animation::{TimeDelta, TimeTicks};

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
    pub frame_id: BeginFrameId,
    pub start_update_display_tree: TimeTicks,
    pub start_prepare_to_draw: TimeTicks,
    pub start_draw_layers: TimeTicks,
    pub submit_compositor_frame: TimeTicks,
}
