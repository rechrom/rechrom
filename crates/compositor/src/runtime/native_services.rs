//! Platform-independent native services for the CPU compositor assembly.
//!
//! These values contain explicit configuration, injected clocks/executors and
//! diagnostic sinks only. They never retain a Page, Host, tree or layer.

use crate::{
    adapter::{
        browser_controls::BrowserControlsDiagnostic,
        input_handler::{InputFeature, InputTraceField, ScrollTrackingType, ScrollingThread},
        layer_tree_host_impl::{
            HostImplBrowserControlsServices, HostImplDebugServices,
            HostImplInputEnvironmentServices, HostImplTaskRunnerId, HostImplTaskRunnerProviderId,
            HostImplTaskRunnerServices,
        },
        time::TimeDelta,
    },
    engine::CompositorExecutor,
    runtime::input::{FrameSequenceTrackerType, InputTraceSpan, SmoothEffectDrivingThread},
    NativeCompositorTypes,
};
use std::{path::Path, sync::Arc};

/// Optional diagnostics and metrics are observable through one narrow sink.
/// `NoopNativeEventSink` is an explicit production choice for embeddings that
/// do not collect optional Chromium trace/histogram events.
pub trait NativeEventSink: Send + Sync {
    fn Emit(&self, category: &'static str, message: String);
    fn HistogramCount(&self, name: &str, value: f32);
    fn HistogramTime(&self, name: &str, value: TimeDelta);
    fn HistogramCustomTimeBuckets(&self, name: &str, value: TimeDelta, buckets: &[i32]);
}

pub struct NoopNativeEventSink;
impl NativeEventSink for NoopNativeEventSink {
    fn Emit(&self, _category: &'static str, _message: String) {}
    fn HistogramCount(&self, _name: &str, _value: f32) {}
    fn HistogramTime(&self, _name: &str, _value: TimeDelta) {}
    fn HistogramCustomTimeBuckets(&self, _name: &str, _value: TimeDelta, _buckets: &[i32]) {}
}

#[derive(Clone, Debug)]
pub struct NativeDiagnosticConfig {
    pub maximum_vlog_level: Option<i32>,
    pub client_name_for_metrics: Option<String>,
    pub dump_compositor_frame_switch: Option<String>,
    pub allow_debug_file_output: bool,
}

pub struct NativeDiagnosticServices {
    config: NativeDiagnosticConfig,
    sink: Arc<dyn NativeEventSink>,
}
impl NativeDiagnosticServices {
    pub fn new(config: NativeDiagnosticConfig, sink: Arc<dyn NativeEventSink>) -> Self {
        Self { config, sink }
    }
}
impl HostImplDebugServices for NativeDiagnosticServices {
    fn VlogIsOn(&self, level: i32) -> bool {
        self.config
            .maximum_vlog_level
            .is_some_and(|maximum| level <= maximum)
    }
    fn GetClientNameForMetrics(&self) -> Option<&str> {
        self.config.client_name_for_metrics.as_deref()
    }
    fn DumpCompositorFrameSwitch(&self) -> Option<String> {
        self.config.dump_compositor_frame_switch.clone()
    }
    fn WriteFileInCurrentDirectory(&self, filename: &str, data: &str) -> bool {
        if !self.config.allow_debug_file_output {
            return false;
        }
        let path = Path::new(filename);
        if path.is_absolute() || path.components().count() != 1 {
            self.sink.Emit(
                "compositor.debug",
                format!("rejected non-local debug output path: {filename}"),
            );
            return false;
        }
        match std::fs::write(path, data) {
            Ok(()) => true,
            Err(error) => {
                self.sink.Emit(
                    "compositor.debug",
                    format!("failed writing {filename}: {error}"),
                );
                false
            }
        }
    }
    fn BeginVisibilityTrace(&self, owner_id: u64) {
        self.sink
            .Emit("compositor.visibility", format!("begin owner={owner_id}"))
    }
    fn EndVisibilityTrace(&self, owner_id: u64) {
        self.sink
            .Emit("compositor.visibility", format!("end owner={owner_id}"))
    }
}

/// IDs are derived from the actual injected executor allocations. When Page
/// selects synchronous execution it supplies the same Arc for both sequences;
/// when it selects a compositor thread it supplies distinct executors.
pub struct NativeTaskRunnerServices {
    compositor: Arc<dyn CompositorExecutor>,
    main: Arc<dyn CompositorExecutor>,
    compositor_id: HostImplTaskRunnerId,
    main_id: HostImplTaskRunnerId,
    has_impl_thread: bool,
}
impl NativeTaskRunnerServices {
    pub fn new(compositor: Arc<dyn CompositorExecutor>, main: Arc<dyn CompositorExecutor>) -> Self {
        fn identity(executor: &Arc<dyn CompositorExecutor>) -> HostImplTaskRunnerId {
            HostImplTaskRunnerId(Arc::as_ptr(executor) as *const () as usize as u64)
        }
        let compositor_id = identity(&compositor);
        let main_id = identity(&main);
        Self {
            has_impl_thread: !Arc::ptr_eq(&compositor, &main),
            compositor,
            main,
            compositor_id,
            main_id,
        }
    }

    /// The native engine serializes both Proxy sequences through one injected
    /// owner executor, while Page keeps the document sequence outside that
    /// engine. Preserve the two logical runner identities so cc selects the
    /// same threaded ProxyImpl lifecycle as Chromium production widgets.
    pub(crate) fn new_threaded_on_single_executor(executor: Arc<dyn CompositorExecutor>) -> Self {
        let compositor_id =
            HostImplTaskRunnerId(Arc::as_ptr(&executor) as *const () as usize as u64);
        // Trait-object allocations are pointer-aligned, so the low bit is free
        // for the colocated main-sequence identity and remains stable across
        // independently assembled service facets.
        let main_id = HostImplTaskRunnerId(compositor_id.0 ^ 1);
        Self {
            compositor: executor.clone(),
            main: executor,
            compositor_id,
            main_id,
            has_impl_thread: true,
        }
    }
    pub fn Resolve(&self, id: HostImplTaskRunnerId) -> Option<&Arc<dyn CompositorExecutor>> {
        if id == self.compositor_id {
            Some(&self.compositor)
        } else if id == self.main_id {
            Some(&self.main)
        } else {
            None
        }
    }

    pub(crate) fn CompositorRunnerId(&self) -> HostImplTaskRunnerId {
        self.compositor_id
    }

    pub(crate) fn MainRunnerId(&self) -> HostImplTaskRunnerId {
        self.main_id
    }
}
impl HostImplTaskRunnerServices for NativeTaskRunnerServices {
    fn MainThreadTaskRunner(
        &self,
        _provider: HostImplTaskRunnerProviderId,
    ) -> HostImplTaskRunnerId {
        self.main_id
    }
    fn ImplThreadTaskRunner(
        &self,
        _provider: HostImplTaskRunnerProviderId,
    ) -> HostImplTaskRunnerId {
        self.compositor_id
    }
    fn HasImplThread(&self, _provider: HostImplTaskRunnerProviderId) -> bool {
        self.has_impl_thread
    }
}

pub struct NativeBrowserControlsServices {
    sink: Arc<dyn NativeEventSink>,
}
impl NativeBrowserControlsServices {
    pub fn new(sink: Arc<dyn NativeEventSink>) -> Self {
        Self { sink }
    }
}
impl HostImplBrowserControlsServices for NativeBrowserControlsServices {
    fn EmitBrowserControlsDiagnostic(&mut self, event: BrowserControlsDiagnostic) {
        self.sink
            .Emit("compositor.browser_controls", format!("{event:?}"))
    }
}

#[derive(Clone, Debug)]
pub struct NativeInputConfig {
    pub wait_for_late_scroll_events: bool,
    pub overscroll_effect_on_non_root_scrollers: bool,
    pub snap_fling_near_extremes: bool,
    pub limit_scroll_delta_to_scroller_size: bool,
    pub scroll_event_dispatch_mode: String,
    pub late_scroll_deadline_ratio: f64,
}

struct NativeInputTraceSpan;
impl InputTraceSpan for NativeInputTraceSpan {}

pub struct NativeInputEnvironmentServices {
    config: NativeInputConfig,
    sink: Arc<dyn NativeEventSink>,
}
impl NativeInputEnvironmentServices {
    pub fn new(config: NativeInputConfig, sink: Arc<dyn NativeEventSink>) -> Self {
        Self { config, sink }
    }
    fn FormatTrace(event: &str, fields: &[InputTraceField]) -> String {
        format!("{event} {fields:?}")
    }
}
impl HostImplInputEnvironmentServices<NativeCompositorTypes> for NativeInputEnvironmentServices {
    fn InputFeatureEnabled(&self, feature: InputFeature) -> bool {
        match feature {
            InputFeature::WaitForLateScrollEvents => self.config.wait_for_late_scroll_events,
            InputFeature::OverscrollEffectOnNonRootScrollers => {
                self.config.overscroll_effect_on_non_root_scrollers
            }
            InputFeature::SnapFlingNearExtremes => self.config.snap_fling_near_extremes,
            InputFeature::LimitScrollDeltaToScrollerSize => {
                self.config.limit_scroll_delta_to_scroller_size
            }
        }
    }
    fn InputScrollEventDispatchMode(&self) -> String {
        self.config.scroll_event_dispatch_mode.clone()
    }
    fn InputLateScrollDeadlineRatio(&self) -> f64 {
        self.config.late_scroll_deadline_ratio
    }
    fn InputTrackerType(&self, kind: ScrollTrackingType) -> FrameSequenceTrackerType {
        match kind {
            ScrollTrackingType::kWheelScroll => FrameSequenceTrackerType::kWheelScroll,
            ScrollTrackingType::kTouchScroll => FrameSequenceTrackerType::kTouchScroll,
            ScrollTrackingType::kScrollbarScroll => FrameSequenceTrackerType::kScrollbarScroll,
            ScrollTrackingType::kMaxType => FrameSequenceTrackerType::kMaxType,
        }
    }
    fn InputDrivingThread(&self, thread: ScrollingThread) -> SmoothEffectDrivingThread {
        match thread {
            ScrollingThread::kCompositor => SmoothEffectDrivingThread::kCompositor,
            ScrollingThread::kMain => SmoothEffectDrivingThread::kMain,
            ScrollingThread::kRaster => SmoothEffectDrivingThread::kRaster,
        }
    }
    fn InputHistogramCounts(&self, name: &str, value: f32) {
        self.sink.HistogramCount(name, value)
    }
    fn InputHistogramTimes(&self, name: &str, value: TimeDelta) {
        self.sink.HistogramTime(name, value)
    }
    fn InputBeginTrace(&self, event: &str, fields: &[InputTraceField]) -> Box<dyn InputTraceSpan> {
        self.sink
            .Emit("compositor.input", Self::FormatTrace(event, fields));
        Box::new(NativeInputTraceSpan)
    }
    fn InputInstantTrace(&self, event: &str, fields: &[InputTraceField]) {
        self.sink
            .Emit("compositor.input", Self::FormatTrace(event, fields))
    }
}
