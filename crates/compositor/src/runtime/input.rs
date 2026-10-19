//! Native input/metrics values and call-scoped external ownership capabilities.
//! Metrics owners retain their actual allocations; compositor input only moves
//! their event values and borrows native services at the source call sites.
use crate::adapter::{
    input_types::ElementId,
    layer_tree_host_impl::{HostScrollLatency, HostScrollLatencyComponent},
    layer_tree_impl::{ElementListType, LayerTreeMutatorHost},
    property_tree::PropertyTreeMutatorHost,
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

/// The actual latency event supplies component timestamps. This trait is kept
/// with the event allocation so the host never fabricates a reduced event copy.
pub trait NativeLatencyInfo: Send {
    fn FindLatency(&self, component: HostScrollLatencyComponent, timestamp: &mut TimeTicks)
        -> bool;
    fn AddRendererSwapTimestamp(&mut self, timestamp: TimeTicks);
    fn AddRenderingScheduledImpl(&mut self) -> bool;
    fn CloneForSwapPromise(&self) -> Box<dyn NativeLatencyInfo>;
    fn Terminate(&mut self);
    fn TraceId(&self) -> i64;
    fn TraceMainCommit(&self);
}
#[derive(Clone)]
pub struct LatencyInfo(pub std::sync::Arc<std::sync::Mutex<Box<dyn NativeLatencyInfo>>>);
impl LatencyInfo {
    pub fn new(value: Box<dyn NativeLatencyInfo>) -> Self {
        Self(std::sync::Arc::new(std::sync::Mutex::new(value)))
    }
    pub(crate) fn CloneForSwapPromise(&self) -> Self {
        let value = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .CloneForSwapPromise();
        Self::new(value)
    }
    pub(crate) fn AddRenderingScheduledImpl(&mut self) -> bool {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .AddRenderingScheduledImpl()
    }
    pub(crate) fn Terminate(&mut self) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .Terminate()
    }
    pub(crate) fn TraceId(&self) -> i64 {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .TraceId()
    }
    pub(crate) fn TraceMainCommit(&self) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .TraceMainCommit()
    }
    pub(crate) fn AddRendererSwapTimestamp(&mut self, timestamp: TimeTicks) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .AddRendererSwapTimestamp(timestamp)
    }
}
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
pub trait NativeEventMetrics {}
pub type EventMetrics = Box<dyn NativeEventMetrics>;
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

/// Animation state remains owned by the actual external animation subsystem.
/// This wrapper only provides the exact read interface consumed by cc trees.
pub trait NativeMutatorHost:
    LayerTreeMutatorHost + PropertyTreeMutatorHost + std::any::Any
{
    fn AsAnyMut(&mut self) -> &mut dyn std::any::Any;
    fn SetLayerTreeMutator(&mut self, mutator: Option<Box<super::host_owners::LayerTreeMutator>>);
    fn identity(&self) -> super::types::MainMutatorHostIdentity;
    fn NextFrameHasPendingRAF(&self) -> bool;
    fn HasCanvasInvalidation(&self) -> bool;
    fn HasJSAnimation(&self) -> bool;
    fn MainThreadAnimationsCount(&self) -> usize;
    fn HasSmilAnimation(&self) -> bool;
}
pub struct MutatorHost(pub Box<dyn NativeMutatorHost>);
impl MutatorHost {
    pub(crate) fn identity(&self) -> super::types::MainMutatorHostIdentity {
        self.0.identity()
    }
    pub(crate) fn SetLayerTreeMutator(
        &mut self,
        mutator: Option<Box<super::host_owners::LayerTreeMutator>>,
    ) {
        self.0.SetLayerTreeMutator(mutator)
    }
    pub(crate) fn NextFrameHasPendingRAF(&self) -> bool {
        self.0.NextFrameHasPendingRAF()
    }
    pub(crate) fn HasCanvasInvalidation(&self) -> bool {
        self.0.HasCanvasInvalidation()
    }
    pub(crate) fn HasJSAnimation(&self) -> bool {
        self.0.HasJSAnimation()
    }
    pub(crate) fn MainThreadAnimationsCount(&self) -> usize {
        self.0.MainThreadAnimationsCount()
    }
    pub(crate) fn HasSmilAnimation(&self) -> bool {
        self.0.HasSmilAnimation()
    }
}
impl LayerTreeMutatorHost for MutatorHost {
    fn HasAnyAnimationTargetingProperty(
        &self,
        id: ElementId,
        property: crate::cc::trees::target_property::Type,
    ) -> bool {
        self.0.HasAnyAnimationTargetingProperty(id, property)
    }
    fn HasPotentiallyRunningAnimationForProperty(
        &self,
        id: ElementId,
        list: ElementListType,
        property: crate::cc::trees::target_property::Type,
    ) -> bool {
        self.0
            .HasPotentiallyRunningAnimationForProperty(id, list, property)
    }
    fn MaximumScale(&self, id: ElementId, list: ElementListType) -> f32 {
        self.0.MaximumScale(id, list)
    }
}
impl PropertyTreeMutatorHost for MutatorHost {
    fn ScrollOffsetAnimationWasInterrupted(&self, id: ElementId) -> bool {
        self.0.ScrollOffsetAnimationWasInterrupted(id)
    }
}

// Source dependency: cc/metrics/event_metrics.h:779-795. Native event lists
// preserve the original driving thread; the event objects are moved, not copied.
#[derive(Default)]
pub struct EventMetricsSet {
    pub main_event_metrics: Vec<EventMetrics>,
    pub impl_event_metrics: Vec<EventMetrics>,
    pub raster_event_metrics: Vec<EventMetrics>,
}
