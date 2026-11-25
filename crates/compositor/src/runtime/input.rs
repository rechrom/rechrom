//! Native input/metrics values and call-scoped external ownership capabilities.
//! Metrics owners retain their actual allocations; compositor input only moves
//! their event values and borrows native services at the source call sites.
use crate::adapter::{
    layer_tree_host_impl::{HostScrollLatency, HostScrollLatencyComponent},
    time::TimeTicks,
};

// Source dependency: cc/metrics/begin_main_frame_metrics.h:17-39.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum BeginMainFrameReason {
    #[default]
    kOther = 0,
    kRAFOrServiceScriptedAnimations = 1,
    kVideoFrameCallback = 2,
    kAnimation = 3,
    kStylePaintOrLayoutInvalidation = 4,
    kScroll = 5,
    kInput = 6,
    kMainThreadScroll = 7,
    kDelayedTimerFired = 8,
}
impl BeginMainFrameReason {
    pub const kRAF: Self = Self::kRAFOrServiceScriptedAnimations;
    pub const kServiceScriptedAnimations: Self = Self::kRAFOrServiceScriptedAnimations;
    pub const kCSSAnimation: Self = Self::kAnimation;
    pub const kStyleInvalidation: Self = Self::kStylePaintOrLayoutInvalidation;
    pub const kPaintInvalidation: Self = Self::kStylePaintOrLayoutInvalidation;
    pub const kLayoutInvalidation: Self = Self::kStylePaintOrLayoutInvalidation;
    pub const kMaxValue: Self = Self::kDelayedTimerFired;
}
// Source dependency: cc/metrics/frame_sequence_metrics.h:29-49.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameSequenceTrackerType {
    kCompositorAnimation = 0,
    kMainThreadAnimation = 1,
    kPinchZoom = 2,
    kRAF = 3,
    kTouchScroll = 4,
    kVideo = 6,
    kWheelScroll = 7,
    kScrollbarScroll = 8,
    kCustom = 9,
    kCanvasAnimation = 10,
    kJSAnimation = 11,
    kSETMainThreadAnimation = 12,
    kSETCompositorAnimation = 13,
    kCompositorRasterAnimation = 14,
    kCompositorNativeAnimation = 15,
    kMaxType = 16,
}
// Source dependency: cc/metrics/frame_info.h:65-71.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum SmoothEffectDrivingThread {
    kMain = 0,
    kCompositor = 1,
    kRaster = 2,
    #[default]
    kUnknown = 3,
}
impl SmoothEffectDrivingThread {
    pub const kMaxValue: Self = Self::kUnknown;
}

// ui::LatencyInfo is a viz frame value (viz_common::latency_info).
pub use viz_common::latency_info::{LatencyInfo, LatencyInfoBackend};
impl HostScrollLatency for LatencyInfo {
    fn FindLatency(
        &self,
        component: HostScrollLatencyComponent,
        timestamp: &mut TimeTicks,
    ) -> bool {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .FindLatency(component, timestamp)
    }
}
/// Source: cc/metrics/events_metrics_manager.h:38-39. Returning None has native
/// nullptr semantics; ownership of a real metrics event is moved to the caller.
/// cc/metrics/event_metrics.h is a shared cc value (the main LayerTreeHost
/// queues it in CommitState).
pub use compositor_values::event_metrics::{EventMetrics, EventMetricsBackend};
pub type EventMetricsDoneCallback = Box<dyn FnOnce(bool) -> Option<EventMetrics>>;
/// Native scope objects execute their external destruction behavior on Drop.
/// No fabricated default monitor exists and neither can expose a tree/host.
pub trait LatencyInfoSwapPromiseMonitor:
    crate::adapter::layer_tree_host_impl::HostLatencyInfoSwapPromiseMonitor
{
}
pub trait EventMetricsScopedMonitor {
    fn SetSaveMetrics(&mut self);
}
pub trait InputTraceSpan {}
/// The std::unique_ptr<MutatorHost> of LayerTreeHostImpl
/// (cc/trees/mutator_host.h), shared with the main half.
pub use layerize::commit::mutator_host::{MutatorHost, MutatorHostBackend};

// Source dependency: cc/metrics/event_metrics.h:779-795. Native event lists
// preserve the original driving thread; the event objects are moved, not copied.
#[derive(Default)]
pub struct EventMetricsSet {
    pub main_event_metrics: Vec<EventMetrics>,
    pub impl_event_metrics: Vec<EventMetrics>,
    pub raster_event_metrics: Vec<EventMetrics>,
}
