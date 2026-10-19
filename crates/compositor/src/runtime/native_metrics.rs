//! Native metric owners required by the canonical host constructor.
//!
//! Metrics may be explicitly disabled, matching Chromium's non-reporting
//! configuration. State that affects compositor behavior (active sequences,
//! event ownership and first-contentful-paint) remains real in either mode.

use super::{
    host_metrics::{
        FrameSequenceTrackers, NativeFrameSequenceTrackers, NativeReportingController,
        ReportingController,
    },
    host_owners::{
        AverageLagTrackingManager, EventsMetricsManager, FrameSorter,
        NativeAverageLagTrackingManager, NativeEventsMetricsManager, NativeFrameSorter,
    },
    input::{
        EventMetrics, EventMetricsDoneCallback, EventMetricsScopedMonitor, EventMetricsSet,
        FrameSequenceTrackerType, SmoothEffectDrivingThread,
    },
    native_services::NativeEventSink,
};
use crate::{
    adapter::{
        begin_frame::{BeginFrameArgs, BeginFrameId},
        commit_types::ActiveFrameSequenceTrackers,
        input_types::ElementId,
        layer_tree_host_impl::{HostAnimationSequence, HostDrawSequence, HostImplWeak},
        scheduler_host::{
            BeginMainFrameMetrics, CompositorFrameReportingController, FrameTimingDetails,
            SubmitInfo,
        },
        time::TimeTicks,
    },
    cc::scheduler::{commit_earlyout_reason::CommitEarlyOutReason, scheduler::FrameSkippedReason},
    NativeCompositorTypes,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Debug)]
pub struct NativeMetricsConfig {
    pub reporting_enabled: bool,
    pub frame_sequence_metrics_enabled: bool,
    pub lag_tracking_enabled: bool,
    pub event_metrics_enabled: bool,
}

/// Constructor-only capability consumed by the host core assembly. Returned
/// owners are independent allocations retained by the canonical host.
pub struct NativeMetricsServices {
    config: NativeMetricsConfig,
    sink: Arc<dyn NativeEventSink>,
}
impl NativeMetricsServices {
    pub fn new(config: NativeMetricsConfig, sink: Arc<dyn NativeEventSink>) -> Self {
        Self { config, sink }
    }
    pub fn CreateFrameSorter(&self) -> FrameSorter {
        FrameSorter(Box::new(CpuNativeFrameSorter::new()))
    }
    pub fn CreateFrameTrackers(
        &self,
        single_threaded: bool,
    ) -> FrameSequenceTrackers<NativeCompositorTypes> {
        FrameSequenceTrackers(Arc::new(CpuNativeFrameSequenceTrackers::new(
            self.config.frame_sequence_metrics_enabled,
            single_threaded,
            self.sink.clone(),
        )))
    }
    pub fn CreateReportingController(
        &self,
        explicitly_disabled: bool,
    ) -> ReportingController<NativeCompositorTypes> {
        ReportingController(Arc::new(CpuNativeReportingController::new(
            self.config.reporting_enabled && !explicitly_disabled,
            self.sink.clone(),
        )))
    }
    pub(crate) fn CreateReportingControllerForHost(
        &self,
        should_report_histograms: bool,
        host_id: i32,
        trees_in_viz_client: bool,
    ) -> ReportingController<NativeCompositorTypes> {
        ReportingController(Arc::new(CpuNativeReportingController {
            enabled: self.config.reporting_enabled && should_report_histograms,
            sink: self.sink.clone(),
            state: Mutex::new(ReportingState::default()),
            host: Some((host_id, trees_in_viz_client)),
        }))
    }
    pub fn CreateAverageLagTrackingManager(&self) -> AverageLagTrackingManager {
        AverageLagTrackingManager(Box::new(CpuNativeAverageLagTrackingManager::new(
            self.config.lag_tracking_enabled,
            self.sink.clone(),
        )))
    }
    pub fn CreateEventsMetricsManager(&self) -> EventsMetricsManager {
        EventsMetricsManager(Box::new(CpuNativeEventsMetricsManager::new(
            self.config.event_metrics_enabled,
        )))
    }
}

pub struct CpuNativeFrameSorter {
    first_contentful_paint_received: bool,
}
impl CpuNativeFrameSorter {
    pub fn new() -> Self {
        Self {
            first_contentful_paint_received: false,
        }
    }
}
impl NativeFrameSorter for CpuNativeFrameSorter {
    fn Reset(&mut self, reset_first_contentful_paint: bool) {
        if reset_first_contentful_paint {
            self.first_contentful_paint_received = false;
        }
    }
    fn FirstContentfulPaintReceived(&self) -> bool {
        self.first_contentful_paint_received
    }
    fn OnFirstContentfulPaintReceived(&mut self) {
        debug_assert!(!self.first_contentful_paint_received);
        self.first_contentful_paint_received = true;
    }
}

#[derive(Default)]
struct ReportingState {
    visible: bool,
    needs_raster_properties_animated: bool,
    scroll_jank_owner: Option<u64>,
    event_latency_owner: Option<u64>,
    frame_sorter_attached: bool,
    trackers_attached: bool,
    submitted_frames: u64,
}
pub struct CpuNativeReportingController {
    enabled: bool,
    sink: Arc<dyn NativeEventSink>,
    state: Mutex<ReportingState>,
    // Source constructor facts belong to the external reporting allocation,
    // not to a compositor-side metrics registry.
    host: Option<(i32, bool)>,
}
impl CpuNativeReportingController {
    pub fn new(enabled: bool, sink: Arc<dyn NativeEventSink>) -> Self {
        Self {
            enabled,
            sink,
            state: Mutex::new(ReportingState::default()),
            host: None,
        }
    }
    fn Emit(&self, message: impl Into<String>) {
        if self.enabled {
            let message = message.into();
            self.sink.Emit(
                "compositor.frame_reporting",
                match self.host {
                    Some((id, trees_in_viz_client)) => {
                        format!("host={id} trees_in_viz_client={trees_in_viz_client} {message}")
                    }
                    None => message,
                },
            );
        }
    }
}
impl NativeReportingController<NativeCompositorTypes> for CpuNativeReportingController {
    fn ReportingSetScrollJankOsReporter(&self, owner: HostImplWeak) {
        self.state
            .lock()
            .expect("reporting state")
            .scroll_jank_owner = Some(owner.owner.0);
    }
    fn ReportingSetFrameSorter(&self, _sorter: &mut FrameSorter) {
        self.state
            .lock()
            .expect("reporting state")
            .frame_sorter_attached = true;
    }
    fn ReportingSetFrameSequenceTrackerCollection(
        &self,
        _trackers: &mut <NativeCompositorTypes as crate::adapter::layer_tree_host_impl::HostImplTypes>::FrameSequenceTrackerCollection,
    ) {
        self.state
            .lock()
            .expect("reporting state")
            .trackers_attached = true;
    }
    fn ReportingSetEventLatencyTracker(
        &self,
        owner: Option<crate::adapter::layer_tree_impl::LayerTreeHostId>,
    ) {
        self.state
            .lock()
            .expect("reporting state")
            .event_latency_owner = owner.map(|id| id.0);
    }
    fn ReportingClearFrameSequenceTrackerCollection(&self) {
        self.state
            .lock()
            .expect("reporting state")
            .trackers_attached = false;
    }
    fn ReportingSetNeedsRasterPropertiesAnimated(&self, animated: bool) {
        self.state
            .lock()
            .expect("reporting state")
            .needs_raster_properties_animated = animated;
    }
    fn ReportingSetVisible(&self, visible: bool) {
        self.state.lock().expect("reporting state").visible = visible;
    }
}
impl CompositorFrameReportingController for CpuNativeReportingController {
    fn OnFinishImplFrame(&self, frame_id: BeginFrameId, waiting_for_main: bool) {
        self.Emit(format!(
            "finish_impl_frame {frame_id:?} waiting_for_main={waiting_for_main}"
        ));
    }
    fn DidSubmitCompositorFrame(
        &self,
        submit_info: &mut SubmitInfo,
        begin_main_frame_id: BeginFrameId,
        last_activated_frame_id: BeginFrameId,
    ) {
        self.state.lock().expect("reporting state").submitted_frames += 1;
        self.Emit(format!(
            "submit token={} begin={begin_main_frame_id:?} activated={last_activated_frame_id:?}",
            submit_info.frame_token
        ));
    }
    fn NotifyReadyToCommit(&self, details: Option<BeginMainFrameMetrics>) {
        self.Emit(format!("ready_to_commit metrics={}", details.is_some()));
    }
    fn BeginMainFrameAborted(&self, frame_id: BeginFrameId, reason: CommitEarlyOutReason) {
        self.Emit(format!("begin_main_frame_aborted {frame_id:?} {reason:?}"));
    }
    fn DidPresentCompositorFrame(&self, frame_token: u32, details: &FrameTimingDetails) {
        self.Emit(format!(
            "present token={frame_token} timestamp={:?}",
            details.presentation_feedback.timestamp
        ));
    }
    fn BeginMainFrameStarted(&self, time: TimeTicks) {
        self.Emit(format!("begin_main_frame_started {time:?}"));
    }
    fn OnStoppedRequestingBeginFrames(&self) {
        self.Emit("stopped_requesting_begin_frames");
    }
    fn DidNotProduceFrame(&self, frame_id: BeginFrameId, reason: FrameSkippedReason) {
        self.Emit(format!("did_not_produce {frame_id:?} {reason:?}"));
    }
    fn WillBeginImplFrame(&self, args: &BeginFrameArgs, throttled: bool) {
        self.Emit(format!(
            "will_begin_impl_frame {:?} throttled={throttled}",
            args.frame_id
        ));
    }
    fn WillBeginMainFrame(&self, args: &BeginFrameArgs) {
        self.Emit(format!("will_begin_main_frame {:?}", args.frame_id));
    }
    fn WillCommit(&self) {
        self.Emit("will_commit");
    }
    fn DidCommit(&self) {
        self.Emit("did_commit");
    }
    fn WillActivate(&self) {
        self.Emit("will_activate");
    }
    fn DidActivate(&self) {
        self.Emit("did_activate");
    }
    fn WillInvalidateOnImplSide(&self) {
        self.Emit("will_invalidate_on_impl_side");
    }
}

#[derive(Default)]
struct TrackerState {
    active: BTreeMap<u8, SmoothEffectDrivingThread>,
    smooth_main_thread_count: i32,
    custom_results_callback: Option<HostImplWeak>,
    begin_frames: u64,
    paused: bool,
    scrolling_thread: SmoothEffectDrivingThread,
    custom_active: std::collections::BTreeSet<usize>,
    custom_terminating: std::collections::BTreeSet<usize>,
}
pub struct CpuNativeFrameSequenceTrackers {
    enabled: bool,
    single_threaded: bool,
    sink: Arc<dyn NativeEventSink>,
    state: Mutex<TrackerState>,
}
impl CpuNativeFrameSequenceTrackers {
    pub fn new(enabled: bool, single_threaded: bool, sink: Arc<dyn NativeEventSink>) -> Self {
        Self {
            enabled,
            single_threaded,
            sink,
            state: Mutex::new(TrackerState::default()),
        }
    }
    fn Start(&self, kind: u8, thread: SmoothEffectDrivingThread) {
        if self.single_threaded {
            return;
        }
        let mut state = self.state.lock().expect("tracker state");
        if matches!(kind, 4 | 7 | 8) {
            state.scrolling_thread = thread;
        }
        state.active.insert(kind, thread);
        drop(state);
        if self.enabled {
            self.sink.Emit(
                "compositor.frame_sequence",
                format!("start type={kind} thread={thread:?}"),
            );
        }
    }
    fn Stop(&self, kind: u8) {
        let mut state = self.state.lock().expect("tracker state");
        if matches!(kind, 4 | 7 | 8) {
            state.scrolling_thread = SmoothEffectDrivingThread::kUnknown;
        }
        state.active.remove(&kind);
        drop(state);
        if self.enabled {
            self.sink
                .Emit("compositor.frame_sequence", format!("stop type={kind}"));
        }
    }
}
impl NativeFrameSequenceTrackers<NativeCompositorTypes> for CpuNativeFrameSequenceTrackers {
    fn GetScrollingThread(&self) -> SmoothEffectDrivingThread {
        self.state.lock().expect("tracker state").scrolling_thread
    }
    fn StartMainSequence(
        &self,
        sequence: crate::adapter::client_layer_tree_host_impl::ClientSequence,
    ) {
        use crate::adapter::client_layer_tree_host_impl::ClientSequence;
        let kind = match sequence {
            ClientSequence::RAF => FrameSequenceTrackerType::kRAF,
            ClientSequence::CanvasAnimation => FrameSequenceTrackerType::kCanvasAnimation,
            ClientSequence::JSAnimation => FrameSequenceTrackerType::kJSAnimation,
            ClientSequence::MainThreadAnimation => FrameSequenceTrackerType::kMainThreadAnimation,
            ClientSequence::SETMainThreadAnimation => {
                FrameSequenceTrackerType::kSETMainThreadAnimation
            }
        };
        self.Start(kind as u8, SmoothEffectDrivingThread::kMain);
    }
    fn StartCustomSequence(&self, id: usize) {
        // Source external tracker lifecycle: frame_sequence_tracker_collection.cc:245-256.
        let mut state = self.state.lock().expect("tracker state");
        assert!(
            state.custom_active.insert(id),
            "custom sequence already active"
        );
    }
    fn StopCustomSequence(&self, id: usize) {
        // Source: frame_sequence_tracker_collection.cc:258-271. No fabricated
        // metric results are emitted; the native tracker retains termination.
        let mut state = self.state.lock().expect("tracker state");
        if state.custom_active.remove(&id) {
            state.custom_terminating.insert(id);
        }
    }

    fn TrackersUpdateSmoothMainThreadHistory(&self, modifier: i32) {
        self.state
            .lock()
            .expect("tracker state")
            .smooth_main_thread_count += modifier;
    }
    fn TrackersSetCustomResultsAddedCallback(&self, callback: HostImplWeak) {
        self.state
            .lock()
            .expect("tracker state")
            .custom_results_callback = Some(callback);
    }
    fn FrameSequenceTrackerActiveTypes(&self) -> ActiveFrameSequenceTrackers {
        self.state
            .lock()
            .expect("tracker state")
            .active
            .keys()
            .fold(0u16, |bits, kind| bits | (1u16 << u32::from(*kind)))
    }
    fn TrackersStartPinchZoom(&self) {
        self.Start(
            FrameSequenceTrackerType::kPinchZoom as u8,
            SmoothEffectDrivingThread::kCompositor,
        );
    }
    fn TrackersStopPinchZoom(&self) {
        self.Stop(FrameSequenceTrackerType::kPinchZoom as u8);
    }
    fn TrackersStartSequence(&self, kind: HostAnimationSequence) {
        let (kind, thread) = match kind {
            HostAnimationSequence::kCompositorAnimation => (
                FrameSequenceTrackerType::kCompositorAnimation,
                SmoothEffectDrivingThread::kCompositor,
            ),
            HostAnimationSequence::kCompositorRasterAnimation => (
                FrameSequenceTrackerType::kCompositorRasterAnimation,
                SmoothEffectDrivingThread::kRaster,
            ),
            HostAnimationSequence::kCompositorNativeAnimation => (
                FrameSequenceTrackerType::kCompositorNativeAnimation,
                SmoothEffectDrivingThread::kCompositor,
            ),
            HostAnimationSequence::kSETCompositorAnimation => (
                FrameSequenceTrackerType::kSETCompositorAnimation,
                SmoothEffectDrivingThread::kCompositor,
            ),
        };
        self.Start(kind as u8, thread);
    }
    fn TrackersStopSequence(&self, kind: HostAnimationSequence) {
        self.Stop(kind as u8);
    }
    fn TrackersNotifyBeginImplFrame(&self, _args: &BeginFrameArgs) {
        self.state.lock().expect("tracker state").begin_frames += 1;
    }
    fn TrackersNotifyFrameEnd(&self, _current: &BeginFrameArgs, _args: &BeginFrameArgs) {}
    fn FrameTrackersNotifyPauseFrameProduction(&self) {
        self.state.lock().expect("tracker state").paused = true;
    }
    fn FrameTrackersClearAll(&self) {
        let mut state = self.state.lock().expect("tracker state");
        state.active.clear();
        state.custom_active.clear();
        state.custom_terminating.clear();
    }
    fn FrameTrackersStartScrollSequence(
        &self,
        sequence: FrameSequenceTrackerType,
        thread: SmoothEffectDrivingThread,
    ) {
        self.Start(sequence as u8, thread);
    }
    fn FrameTrackersStopSequence(&self, sequence: FrameSequenceTrackerType) {
        self.Stop(sequence as u8);
    }
    fn StopDrawSequence(&self, kind: HostDrawSequence) {
        self.Stop(kind as u8);
    }
}

pub struct CpuNativeAverageLagTrackingManager {
    enabled: bool,
    sink: Arc<dyn NativeEventSink>,
    presented_frames: u64,
    collected_scroll_frames: u64,
}
impl CpuNativeAverageLagTrackingManager {
    pub fn new(enabled: bool, sink: Arc<dyn NativeEventSink>) -> Self {
        Self {
            enabled,
            sink,
            presented_frames: 0,
            collected_scroll_frames: 0,
        }
    }
}
impl NativeAverageLagTrackingManager for CpuNativeAverageLagTrackingManager {
    fn Clear(&mut self) {
        self.presented_frames = 0;
        self.collected_scroll_frames = 0;
    }
    fn DidPresentCompositorFrame(
        &mut self,
        token: u32,
        _details: &super::metadata::FrameTimingDetails,
    ) {
        self.presented_frames += 1;
        if self.enabled {
            self.sink
                .Emit("compositor.average_lag", format!("present token={token}"));
        }
    }
    fn CollectScrollEventsFromFrame(&mut self, token: u32, _events: &EventMetricsSet) {
        self.collected_scroll_frames += 1;
        if self.enabled {
            self.sink
                .Emit("compositor.average_lag", format!("scroll token={token}"));
        }
    }
}

#[derive(Default)]
struct EventMetricsState {
    active: Vec<EventMetrics>,
    saved: Vec<EventMetrics>,
    did_scroll: bool,
    applied_scrolls: Vec<ElementId>,
}
struct CpuEventMetricsMonitor {
    enabled: bool,
    save_metrics: bool,
    callback: Option<EventMetricsDoneCallback>,
    state: Arc<Mutex<EventMetricsState>>,
}
impl EventMetricsScopedMonitor for CpuEventMetricsMonitor {
    fn SetSaveMetrics(&mut self) {
        self.save_metrics = true;
    }
}
impl Drop for CpuEventMetricsMonitor {
    fn drop(&mut self) {
        let Some(callback) = self.callback.take() else {
            return;
        };
        if let Some(event) = callback(self.enabled && self.save_metrics) {
            if self.enabled && self.save_metrics {
                self.state
                    .lock()
                    .expect("event metrics state")
                    .active
                    .push(event);
            }
        }
    }
}
pub struct CpuNativeEventsMetricsManager {
    enabled: bool,
    state: Arc<Mutex<EventMetricsState>>,
}
impl CpuNativeEventsMetricsManager {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            state: Arc::new(Mutex::new(EventMetricsState::default())),
        }
    }
}
impl NativeEventsMetricsManager for CpuNativeEventsMetricsManager {
    fn TakeSavedEventsMetrics(&mut self) -> Vec<EventMetrics> {
        std::mem::take(&mut self.state.lock().expect("event metrics state").saved)
    }
    fn SaveActiveEventMetrics(&mut self) {
        let mut state = self.state.lock().expect("event metrics state");
        let active = std::mem::take(&mut state.active);
        state.saved.extend(active);
    }
    fn SetDidScroll(&mut self, scrolled: bool) {
        self.state.lock().expect("event metrics state").did_scroll = scrolled;
    }
    fn DropSavedEventMetricsForNoFrameUpdate(&mut self) {
        self.state
            .lock()
            .expect("event metrics state")
            .saved
            .clear();
    }
    fn DidApplyScroll(&mut self, id: ElementId) {
        self.state
            .lock()
            .expect("event metrics state")
            .applied_scrolls
            .push(id);
    }
    fn GetScopedMonitor(
        &mut self,
        callback: EventMetricsDoneCallback,
    ) -> Box<dyn EventMetricsScopedMonitor> {
        Box::new(CpuEventMetricsMonitor {
            enabled: self.enabled,
            save_metrics: false,
            callback: Some(callback),
            state: self.state.clone(),
        })
    }
}
