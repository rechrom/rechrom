//! Scheduler host-service boundary. Feature flags, sampling, diagnostics and
//! tracing are supplied by the embedding, without platform or clock ownership.

use super::{time::TimeDelta, TraceValue};
use std::collections::BTreeMap;
use std::sync::Arc;

// Source: cc/tiles/tile_priority.h:87-93. The scheduler consumes the same
// native enum as tile/host priority; there is no parallel representation.
pub use crate::cc::tiles::tile_priority::TreePriority;

#[derive(Clone, Copy, Debug)]
pub struct SchedulerFeatures {
    pub is_android: bool,
    pub is_desktop_android: bool,
    pub is_eligible_for_throttle_main_frame_to_60hz: bool,
    pub kThrottleMainFrameTo60HzDesktopAndroid: bool,
    pub kThrottleMainFrameTo60HzWebView: bool,
    pub kThrottleMainFrameTo60Hz: bool,
    pub kThrottleRepeatedNoDamageFrames: bool,
    pub kThrottleRepeatedNoDamageFramesThreshold1: i32,
    pub kThrottleRepeatedNoDamageFramesThreshold2: i32,
    pub kThrottleRepeatedNoDamageFramesIntervalFactor1: i32,
    pub kThrottleRepeatedNoDamageFramesIntervalFactor2: i32,
    pub kHighFramerateRequestFromClient: bool,
    pub kNoCompositorFrameAcks: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulerTraceKind {
    Instant,
    Counter,
    Duration,
    Begin,
    End,
}

#[derive(Clone, Debug)]
pub struct SchedulerTraceTrack {
    pub name: &'static str,
    pub id: u64,
}

#[derive(Clone, Debug)]
pub struct SchedulerTraceEvent {
    pub track: Option<SchedulerTraceTrack>,
    pub category: &'static str,
    pub name: &'static str,
    pub kind: SchedulerTraceKind,
    pub arguments: Vec<(&'static str, TraceValue)>,
}

pub trait SchedulerServices {
    fn trace_track_id(&self) -> u64;
    fn features(&self) -> SchedulerFeatures;
    fn SetIsEligibleForThrottleMainFrameTo60Hz(&self, eligible: bool);
    fn tracing_enabled(&self, category: &'static str) -> bool;
    fn Trace(&self, event: SchedulerTraceEvent);
    fn ShouldRecordSubsampledMetric(&self, probability: f64) -> bool;
    fn HistogramBoolean(&self, name: &'static str, value: bool);
    fn Diagnostic(&self, message: &'static str, count: u32);
}

/// Owned guard matching TRACE_EVENT's lexical lifetime without borrowing the
/// compositor's mutable state or reading a clock in the translated core.
pub struct SchedulerTraceScope {
    services: Arc<dyn SchedulerServices>,
    category: &'static str,
    name: &'static str,
    enabled: bool,
}

impl SchedulerTraceScope {
    pub fn new(
        services: Arc<dyn SchedulerServices>,
        category: &'static str,
        name: &'static str,
        arguments: impl FnOnce() -> Vec<(&'static str, TraceValue)>,
    ) -> Self {
        let enabled = services.tracing_enabled(category);
        if enabled {
            services.Trace(SchedulerTraceEvent {
                category,
                name,
                kind: SchedulerTraceKind::Begin,
                arguments: arguments(),
                track: None,
            });
        }
        Self {
            services,
            category,
            name,
            enabled,
        }
    }
}

impl Drop for SchedulerTraceScope {
    fn drop(&mut self) {
        if self.enabled {
            self.services.Trace(SchedulerTraceEvent {
                category: self.category,
                name: self.name,
                kind: SchedulerTraceKind::End,
                arguments: Vec::new(),
                track: None,
            });
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulerProtoValue {
    LAYER_TREE_FRAME_NONE,
    LAYER_TREE_FRAME_ACTIVE,
    LAYER_TREE_FRAME_CREATING,
    LAYER_TREE_FRAME_WAITING_FOR_FIRST_COMMIT,
    LAYER_TREE_FRAME_WAITING_FOR_FIRST_ACTIVATION,
    BEGIN_IMPL_FRAME_IDLE,
    BEGIN_IMPL_FRAME_INSIDE_BEGIN_FRAME,
    BEGIN_IMPL_FRAME_INSIDE_DEADLINE,
    DEADLINE_MODE_NONE,
    DEADLINE_MODE_IMMEDIATE,
    DEADLINE_MODE_WAIT_FOR_SCROLL,
    DEADLINE_MODE_REGULAR,
    DEADLINE_MODE_LATE,
    DEADLINE_MODE_BLOCKED,
    BEGIN_MAIN_FRAME_IDLE,
    BEGIN_MAIN_FRAME_SENT,
    BEGIN_MAIN_FRAME_READY_TO_COMMIT,
    FORCED_REDRAW_IDLE,
    FORCED_REDRAW_WAITING_FOR_COMMIT,
    FORCED_REDRAW_WAITING_FOR_ACTIVATION,
    FORCED_REDRAW_WAITING_FOR_DRAW,
    CC_SCHEDULER_ACTION_V2_NONE,
    CC_SCHEDULER_ACTION_V2_SEND_BEGIN_MAIN_FRAME,
    CC_SCHEDULER_ACTION_V2_COMMIT,
    CC_SCHEDULER_ACTION_V2_POST_COMMIT,
    CC_SCHEDULER_ACTION_V2_ACTIVATE_SYNC_TREE,
    CC_SCHEDULER_ACTION_V2_DRAW_IF_POSSIBLE,
    CC_SCHEDULER_ACTION_V2_DRAW_FORCED,
    CC_SCHEDULER_ACTION_V2_DRAW_ABORT,
    CC_SCHEDULER_ACTION_V2_BEGIN_LAYER_TREE_FRAME_SINK_CREATION,
    CC_SCHEDULER_ACTION_V2_PREPARE_TILES,
    CC_SCHEDULER_ACTION_V2_INVALIDATE_LAYER_TREE_FRAME_SINK,
    CC_SCHEDULER_ACTION_V2_PERFORM_IMPL_SIDE_INVALIDATION,
    TREE_PRIORITY_SAME_PRIORITY_FOR_BOTH_TREES,
    TREE_PRIORITY_SMOOTHNESS_TAKES_PRIORITY,
    TREE_PRIORITY_NEW_CONTENT_TAKES_PRIORITY,
}

#[derive(Clone, Debug, Default)]
pub struct SchedulerStateTrace {
    pub major_state: BTreeMap<&'static str, SchedulerProtoValue>,
    pub minor_state: BTreeMap<&'static str, TraceValue>,
    pub tree_priority: Option<SchedulerProtoValue>,
}

impl From<TimeDelta> for TraceValue {
    fn from(value: TimeDelta) -> Self {
        Self::Integer(value.InMicroseconds())
    }
}
