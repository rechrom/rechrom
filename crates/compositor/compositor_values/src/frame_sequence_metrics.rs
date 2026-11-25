//! The custom frame-sequence tracker results the impl side reports to the
//! main thread (LayerTreeHostClient::NotifyCompositorMetricsTrackerResults).
use scroll_animation::{TimeDelta, TimeTicks};

// Source dependency: cc/metrics/frame_sequence_metrics.h:92-111;
// cc/metrics/frame_sequence_tracker_collection.h:29-31. These are native owned
// result values, not a second tracker or a copy of compositor owner state.
#[derive(Clone, Debug)]
pub struct Jank {
    pub start_time: TimeTicks,
    pub duration: TimeDelta,
}
#[derive(Clone, Debug, Default)]
pub struct CustomReportData {
    pub frames_expected_v3: u32,
    pub frames_dropped_v3: u32,
    pub jank_count_v3: u32,
    pub janks: Vec<Jank>,
}
pub type CustomTrackerResults = std::collections::BTreeMap<i32, CustomReportData>;
