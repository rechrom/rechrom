//! Native CPU implementation of the Host constructor/platform capability.
//! Every returned owner and every forwarded callback refers to the existing
//! native allocation. Canonical management policy stays in translated cc.
use super::{
    client_resources::CpuClientResources, host_services::CpuHostCoreServices,
    native_host_core::NativeHostCoreOwners, native_host_lifecycle::NativeHostLifecycle,
    native_host_picture::NativeHostPictureFeatures, native_services::NativeEventSink,
};
use crate::{
    adapter::{
        browser_controls::BrowserControlsFeature, image_animation::ImageAnimationTraceValue,
        layer_tree_host_impl::*, layer_tree_impl::*,
    },
    engine::CompositorExecutor,
    NativeCompositorTypes,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
type B = NativeCompositorTypes;

/// Actual process feature inputs; no fabricated feature defaults.
pub(crate) struct NativeCoreConfig {
    pub report_android_scroll_jank: bool,
    pub scroll_snap_animation: bool,
    pub height_change_cancel_animations: bool,
    pub smooth_scroll: bool,
    pub new_content_for_checkerboarded_scrolls: bool,
    pub new_content_per_scroll: bool,
    pub image_animation_trace_enabled: bool,
}

/// The external instrumentation owner, shared by all services using its id.
/// Source dependency: cc/debug/rendering_stats_instrumentation.h:32-39;
/// .cc:21-23. No parallel Host or frame accounting is stored here.
pub(crate) struct NativeRenderingStats {
    identity: HostImplRenderingStatsId,
    record: AtomicBool,
    area: std::sync::Mutex<NativeRenderingArea>,
}
// Source: cc/debug/rendering_stats.h:27-31; instrumentation.cc:21-43.
#[derive(Default)]
struct NativeRenderingArea {
    visible_content_area: i64,
    approximated_visible_content_area: i64,
}
impl NativeRenderingStats {
    pub(crate) fn new(identity: HostImplRenderingStatsId) -> Self {
        Self {
            identity,
            record: AtomicBool::new(false),
            area: std::sync::Mutex::new(NativeRenderingArea::default()),
        }
    }
    pub(crate) fn SetRecordRenderingStats(&self, id: HostImplRenderingStatsId, value: bool) {
        assert_eq!(id, self.identity);
        self.record.store(value, Ordering::Relaxed);
    }
    // Source: cc/debug/rendering_stats_instrumentation.cc:33-43.
    pub(crate) fn AddVisibleContentArea(&self, id: HostImplRenderingStatsId, area: i64) {
        assert_eq!(id, self.identity);
        if !self.record.load(Ordering::Relaxed) {
            return;
        }
        self.area
            .lock()
            .expect("rendering stats")
            .visible_content_area += area;
    }
    pub(crate) fn AddApproximatedVisibleContentArea(
        &self,
        id: HostImplRenderingStatsId,
        area: i64,
    ) {
        assert_eq!(id, self.identity);
        if !self.record.load(Ordering::Relaxed) {
            return;
        }
        self.area
            .lock()
            .expect("rendering stats")
            .approximated_visible_content_area += area;
    }
    pub(crate) fn record_rendering_stats(&self) -> bool {
        self.record.load(Ordering::Relaxed)
    }
}

/// GPU primitives remain external. A CPU endpoint reports no worker provider;
/// requesting a GPU lease without such a provider violates the source caller's
/// preceding Some-provider branch, and must never fabricate a lock/visibility.
pub(crate) trait NativeCoreGpuCapabilities {
    fn WorkerContextProvider(&self, sink: HostImplFrameSinkId)
        -> Option<HostImplContextProviderId>;
    fn ClientBecameVisible(
        &self,
        provider: HostImplContextProviderId,
    ) -> Box<super::host_owners::ContextVisibility>;
    fn ClientBecameNotVisible(
        &self,
        provider: HostImplContextProviderId,
        visibility: Option<Box<super::host_owners::ContextVisibility>>,
    );
    fn LockRasterContext(
        &self,
        provider: HostImplContextProviderId,
    ) -> super::host_owners::RasterContextLock;
    fn WorkerClientBecameVisible(
        &self,
        lock: &mut super::host_owners::RasterContextLock,
    ) -> Box<super::host_owners::ContextVisibility>;
    fn WorkerClientBecameNotVisible(
        &self,
        lock: &mut super::host_owners::RasterContextLock,
        visibility: Option<Box<super::host_owners::ContextVisibility>>,
    );
}
pub(crate) struct CpuNoGpuCapabilities {
    pub sink: HostImplFrameSinkId,
}
impl NativeCoreGpuCapabilities for CpuNoGpuCapabilities {
    fn WorkerContextProvider(
        &self,
        sink: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId> {
        assert_eq!(sink, self.sink);
        None
    }
    fn ClientBecameVisible(
        &self,
        _provider: HostImplContextProviderId,
    ) -> Box<super::host_owners::ContextVisibility> {
        panic!("CPU endpoint has no GPU context; source provider precondition violated")
    }
    fn ClientBecameNotVisible(
        &self,
        _provider: HostImplContextProviderId,
        _visibility: Option<Box<super::host_owners::ContextVisibility>>,
    ) {
        panic!("CPU endpoint has no GPU context; source provider precondition violated")
    }
    fn LockRasterContext(
        &self,
        _provider: HostImplContextProviderId,
    ) -> super::host_owners::RasterContextLock {
        panic!("CPU endpoint has no raster context to lock; source provider precondition violated")
    }
    fn WorkerClientBecameVisible(
        &self,
        _lock: &mut super::host_owners::RasterContextLock,
    ) -> Box<super::host_owners::ContextVisibility> {
        panic!("CPU endpoint has no GPU worker context; source provider precondition violated")
    }
    fn WorkerClientBecameNotVisible(
        &self,
        _lock: &mut super::host_owners::RasterContextLock,
        _visibility: Option<Box<super::host_owners::ContextVisibility>>,
    ) {
        panic!("CPU endpoint has no GPU worker context; source provider precondition violated")
    }
}

pub(crate) struct NativeCoreServices {
    owners: NativeHostCoreOwners,
    lifecycle: Arc<NativeHostLifecycle>,
    resources: CpuClientResources,
    features: Arc<NativeHostPictureFeatures>,
    config: NativeCoreConfig,
    executor: Arc<dyn CompositorExecutor>,
    events: Arc<dyn NativeEventSink>,
    stats: Arc<NativeRenderingStats>,
    gpu: Box<dyn NativeCoreGpuCapabilities>,
    provider_identity: HostImplTaskRunnerProviderId,
}
impl NativeCoreServices {
    pub(crate) fn new(
        owners: NativeHostCoreOwners,
        lifecycle: Arc<NativeHostLifecycle>,
        resources: CpuClientResources,
        features: Arc<NativeHostPictureFeatures>,
        config: NativeCoreConfig,
        executor: Arc<dyn CompositorExecutor>,
        events: Arc<dyn NativeEventSink>,
        stats: Arc<NativeRenderingStats>,
        gpu: Box<dyn NativeCoreGpuCapabilities>,
        provider_identity: HostImplTaskRunnerProviderId,
    ) -> Self {
        Self {
            owners,
            lifecycle,
            resources,
            features,
            config,
            executor,
            events,
            stats,
            gpu,
            provider_identity,
        }
    }
    fn CheckMutator(&self, mutator: &super::input::MutatorHost) {
        assert_eq!(
            mutator.0.identity(),
            self.owners.animation().identity(),
            "same native animation owner"
        );
    }
}
impl CpuHostCoreServices<B> for NativeCoreServices {
    fn CreateChildLocalSurfaceIdAllocator(
        &mut self,
    ) -> <B as HostImplTypes>::ChildLocalSurfaceIdAllocator {
        self.owners.CreateChildLocalSurfaceIdAllocator()
    }
    fn CreateAverageLagTrackingManager(
        &mut self,
    ) -> <B as HostImplTypes>::AverageLagTrackingManager {
        self.owners.CreateAverageLagTrackingManager()
    }
    fn CreateEventsMetricsManager(&mut self) -> <B as HostImplTypes>::EventsMetricsManager {
        self.owners.CreateEventsMetricsManager()
    }
    fn DefaultGlobalTileState(&self) -> <B as LayerTreeTypes>::GlobalTileState {
        Default::default()
    }
    fn CreateFrameSorter(&mut self) -> <B as LayerTreeTypes>::FrameSorter {
        self.owners.CreateFrameSorter()
    }
    fn CreateFrameTrackers(
        &mut self,
        single_threaded: bool,
    ) -> <B as HostImplTypes>::FrameSequenceTrackerCollection {
        self.owners.CreateFrameTrackers(single_threaded)
    }
    fn ResolveSequencedTaskRunner(
        &self,
        runner: HostImplTaskRunnerId,
    ) -> std::sync::Arc<dyn crate::adapter::task_runner::SequencedTaskRunner> {
        self.owners.ResolveSequencedTaskRunner(runner)
    }
    fn RouteHostRepeatingClosure(
        &self,
        task: HostImplTask,
    ) -> crate::adapter::task_runner::RepeatingClosure {
        self.owners.RouteHostRepeatingClosure(task)
    }
    fn CreateColorSpaceCache(&mut self, capacity: usize) -> <B as HostImplTypes>::ColorSpaceCache {
        self.owners.CreateColorSpaceCache(capacity)
    }
    fn NewSequenceChecker(&self) -> <B as HostImplTypes>::SequenceChecker {
        self.owners.NewSequenceChecker()
    }
    fn InvalidFrameSinkId(
        &self,
    ) -> <<B as HostImplTypes>::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId
    {
        super::types::FrameSinkId::default()
    }
    fn CreateImageAnimationTaskRunner(
        &mut self,
        runner: HostImplTaskRunnerId,
    ) -> Box<dyn crate::adapter::image_animation::ImageAnimationTaskRunner> {
        self.owners.CreateImageAnimationTaskRunner(runner)
    }
    fn CreateStubCompositorFrameReportingController(
        &mut self,
    ) -> Box<<B as HostImplTypes>::CompositorFrameReportingController> {
        Box::new(self.owners.CreateReportingController(false))
    }
    fn CreateCompositorFrameReportingController(
        &mut self,
        should_report_histograms: bool,
        id: i32,
        is_trees_in_viz_client: bool,
    ) -> Box<<B as HostImplTypes>::CompositorFrameReportingController> {
        Box::new(self.owners.CreateReportingControllerForHost(
            should_report_histograms,
            id,
            is_trees_in_viz_client,
        ))
    }
    fn ShouldScrollJankV4MetricReportAndroidAppJankStats(&self) -> bool {
        self.config.report_android_scroll_jank
    }
    fn ReportingSetScrollJankOsReporter(
        &mut self,
        controller: &mut <B as HostImplTypes>::CompositorFrameReportingController,
        owner: HostImplWeak,
    ) {
        controller.0.ReportingSetScrollJankOsReporter(owner)
    }
    fn CreateClientResourceProvider(
        &mut self,
        main: HostImplTaskRunnerId,
        impl_runner: HostImplTaskRunnerId,
        flush: HostImplTask,
    ) -> Box<<B as LayerTreeTypes>::ClientResourceProvider> {
        self.resources.ConfigureReleaseCapabilities(
            main,
            impl_runner,
            self.owners.ResolveSequencedTaskRunner(main),
            self.owners.ResolveSequencedTaskRunner(impl_runner),
            self.owners.RouteHostRepeatingClosure(flush),
        );
        Box::new(super::host_owners::ClientResourceProvider(Box::new(
            self.resources.clone(),
        )))
    }
    fn MutatorSetDelegate(
        &mut self,
        mutator: &mut <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
        owner: Option<crate::adapter::layer_tree_impl::LayerTreeHostId>,
    ) {
        self.CheckMutator(mutator);
        self.owners.MutatorSetDelegate(owner)
    }
    fn MutatorCreateEvents(
        &mut self,
        mutator: &mut <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> Box<<B as HostImplTypes>::HostMutatorEvents> {
        self.CheckMutator(mutator);
        Box::new(self.owners.MutatorCreateEvents())
    }
    fn AllocateLayerTreeIdentity(
        &mut self,
    ) -> (
        crate::adapter::layer_tree_impl::LayerTreeId,
        crate::adapter::property_tree::PropertyTreesOwnerId,
    ) {
        self.owners.AllocateLayerTreeIdentity()
    }
    fn TraceHostCreated(&self, id: i32) {
        self.events
            .Emit("cc", format!("LayerTreeHostImpl created {id}"))
    }
    fn TraceHostDeleted(&self, id: i32) {
        self.events
            .Emit("cc", format!("LayerTreeHostImpl deleted {id}"))
    }
    fn ReportingSetFrameSorter(
        &mut self,
        controller: &mut <B as HostImplTypes>::CompositorFrameReportingController,
        sorter: &mut <B as LayerTreeTypes>::FrameSorter,
    ) {
        controller.0.ReportingSetFrameSorter(sorter)
    }
    fn ReportingSetFrameSequenceTrackerCollection(
        &mut self,
        controller: &mut <B as HostImplTypes>::CompositorFrameReportingController,
        trackers: &mut <B as HostImplTypes>::FrameSequenceTrackerCollection,
    ) {
        controller
            .0
            .ReportingSetFrameSequenceTrackerCollection(trackers)
    }
    fn ReportingSetEventLatencyTracker(
        &mut self,
        controller: &mut <B as HostImplTypes>::CompositorFrameReportingController,
        owner: Option<crate::adapter::layer_tree_impl::LayerTreeHostId>,
    ) {
        controller.0.ReportingSetEventLatencyTracker(owner)
    }
    fn TrackersUpdateSmoothMainThreadHistory(
        &mut self,
        trackers: &mut <B as HostImplTypes>::FrameSequenceTrackerCollection,
        modifier: i32,
    ) {
        trackers.0.TrackersUpdateSmoothMainThreadHistory(modifier)
    }
    fn TrackersSetCustomResultsAddedCallback(
        &mut self,
        trackers: &mut <B as HostImplTypes>::FrameSequenceTrackerCollection,
        callback: HostImplWeak,
    ) {
        trackers.0.TrackersSetCustomResultsAddedCallback(callback)
    }
    fn ResourceProviderShutdownAndReleaseAllResources(
        &mut self,
        provider: &mut <B as LayerTreeTypes>::ClientResourceProvider,
        client: &mut dyn HostUIResourceReleaseClient<B>,
    ) {
        provider.0.ShutdownAndReleaseAllResources(client)
    }
    fn MutatorClearMutators(
        &mut self,
        mutator: &mut <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) {
        self.CheckMutator(mutator);
        self.owners.MutatorClearMutators()
    }
    fn ReportingClearFrameSequenceTrackerCollection(
        &mut self,
        controller: &mut <B as HostImplTypes>::CompositorFrameReportingController,
    ) {
        controller.0.ReportingClearFrameSequenceTrackerCollection()
    }
    fn MonotonicTimeNow(&self) -> crate::adapter::time::TimeTicks {
        self.executor.Now()
    }
    fn BrowserControlsFeatureEnabled(
        &self,
        feature: crate::adapter::browser_controls::BrowserControlsFeature,
    ) -> bool {
        match feature {
            BrowserControlsFeature::ScrollSnapAnimation => self.config.scroll_snap_animation,
            BrowserControlsFeature::HeightChangeCancelAnimations => {
                self.config.height_change_cancel_animations
            }
            BrowserControlsFeature::SmoothScroll => self.config.smooth_scroll,
        }
    }
    fn BindRenderFrameMetadataObserverToCurrentSequence(
        &mut self,
        observer: &mut <B as HostImplTypes>::RenderFrameMetadataObserver,
    ) {
        observer.0.BindToCurrentSequence()
    }
    fn SetRecordRenderingStats(&mut self, stats: HostImplRenderingStatsId, enabled: bool) {
        self.stats.SetRecordRenderingStats(stats, enabled)
    }
    fn ResourcePoolSetResourceUsageLimits(
        &mut self,
        pool: &mut <B as HostImplTypes>::ResourcePool,
        bytes: usize,
        resources: usize,
    ) {
        pool.SetResourceUsageLimits(bytes, resources)
    }
    fn DelegateSetNeedsPrepareTilesOnImplThread(&mut self, delegate: HostImplDelegateId) {
        self.lifecycle.SetNeedsPrepareTiles(delegate)
    }
    fn FrameSinkWorkerContextProvider(
        &self,
        sink: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId> {
        self.gpu.WorkerContextProvider(sink)
    }
    fn ClientBecameVisible(
        &mut self,
        provider: HostImplContextProviderId,
    ) -> Box<<B as HostImplTypes>::ContextVisibility> {
        self.gpu.ClientBecameVisible(provider)
    }
    fn ClientBecameNotVisible(
        &mut self,
        provider: HostImplContextProviderId,
        visibility: Option<Box<<B as HostImplTypes>::ContextVisibility>>,
    ) {
        self.gpu.ClientBecameNotVisible(provider, visibility)
    }
    fn LockRasterContext(
        &mut self,
        provider: HostImplContextProviderId,
    ) -> <B as HostImplTypes>::ScopedRasterContextLock {
        self.gpu.LockRasterContext(provider)
    }
    fn WorkerClientBecameVisible(
        &mut self,
        lock: &mut <B as HostImplTypes>::ScopedRasterContextLock,
    ) -> Box<<B as HostImplTypes>::ContextVisibility> {
        self.gpu.WorkerClientBecameVisible(lock)
    }
    fn WorkerClientBecameNotVisible(
        &mut self,
        lock: &mut <B as HostImplTypes>::ScopedRasterContextLock,
        visibility: Option<Box<<B as HostImplTypes>::ContextVisibility>>,
    ) {
        self.gpu.WorkerClientBecameNotVisible(lock, visibility)
    }
    fn DelegateSetNeedsCommitOnImplThread(
        &mut self,
        delegate: HostImplDelegateId,
        reason: <B as crate::adapter::input_services::InputHostTypes>::BeginMainFrameReason,
    ) {
        self.lifecycle.SetNeedsCommit(delegate, reason)
    }
    fn DelegateRenewTreePriority(&mut self, delegate: HostImplDelegateId) {
        self.lifecycle.RenewTreePriority(delegate)
    }
    fn TraceCanDrawReason(&self, reason: &'static str) {
        self.events.Emit("cc", format!("CanDraw: {reason}"))
    }
    fn ChildLocalSurfaceIdAllocatorCurrent<'a>(
        &self,
        allocator: &'a <B as HostImplTypes>::ChildLocalSurfaceIdAllocator,
    ) -> &'a <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::LocalSurfaceId {
        allocator.0.CurrentLocalSurfaceId()
    }
    fn FrameSequenceTrackerActiveTypes(
        &mut self,
        trackers: &mut <B as HostImplTypes>::FrameSequenceTrackerCollection,
    ) -> <B as HostImplTypes>::ActiveFrameSequenceTrackers {
        trackers.0.FrameSequenceTrackerActiveTypes()
    }
    fn TakeSavedEventsMetrics(
        &mut self,
        manager: &mut <B as HostImplTypes>::EventsMetricsManager,
    ) -> <B as HostImplTypes>::EventMetricsList {
        manager.0.TakeSavedEventsMetrics()
    }
    fn SaveActiveEventMetrics(&mut self, manager: &mut <B as HostImplTypes>::EventsMetricsManager) {
        manager.0.SaveActiveEventMetrics()
    }
    fn TrackersStartPinchZoom(
        &mut self,
        trackers: &mut <B as HostImplTypes>::FrameSequenceTrackerCollection,
    ) {
        trackers.0.TrackersStartPinchZoom()
    }
    fn TrackersStopPinchZoom(
        &mut self,
        trackers: &mut <B as HostImplTypes>::FrameSequenceTrackerCollection,
    ) {
        trackers.0.TrackersStopPinchZoom()
    }
    fn DelegateSetHasActiveThreadedScroll(&mut self, delegate: HostImplDelegateId, active: bool) {
        self.lifecycle.SetHasActiveThreadedScroll(delegate, active)
    }
    fn DelegateSetWaitingForScrollEvent(&mut self, delegate: HostImplDelegateId, waiting: bool) {
        self.lifecycle.SetWaitingForScrollEvent(delegate, waiting)
    }
    fn NewContentForCheckerboardedScrollsEnabled(&self) -> bool {
        self.config.new_content_for_checkerboarded_scrolls
    }
    fn NewContentForCheckerboardedScrollsPerScroll(&self) -> bool {
        self.config.new_content_per_scroll
    }
    fn RenderFrameMetadataObserverDidEndScroll(
        &mut self,
        observer: &mut <B as HostImplTypes>::RenderFrameMetadataObserver,
    ) {
        observer.0.DidEndScroll()
    }
    fn DelegateSetDeferBeginMainFrameFromImpl(
        &mut self,
        delegate: HostImplDelegateId,
        defer: bool,
    ) {
        self.lifecycle
            .SetDeferBeginMainFrameFromImpl(delegate, defer)
    }
    fn MutatorHasScrollLinkedAnimation(
        &self,
        mutator: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
        scroller: crate::adapter::input_types::ElementId,
    ) -> bool {
        self.CheckMutator(mutator);
        self.owners.animation().HasScrollLinkedAnimation(scroller)
    }
    fn SetDidScrollForMetrics(
        &mut self,
        manager: &mut <B as HostImplTypes>::EventsMetricsManager,
        did_scroll: bool,
    ) {
        manager.0.SetDidScroll(did_scroll)
    }
    fn DelegateSetNeedsOneBeginImplFrameOnImplThread(&mut self, delegate: HostImplDelegateId) {
        self.lifecycle.SetNeedsOneBeginImplFrame(delegate)
    }
    fn DelegateIsInsideDraw(&self, delegate: HostImplDelegateId) -> bool {
        let _ = delegate;
        self.lifecycle.IsInsideDraw()
    }
    fn DelegateSetNeedsRedrawOnImplThread(&mut self, delegate: HostImplDelegateId) {
        self.lifecycle.SetNeedsRedraw(delegate)
    }
    fn ImageAnimationTraceEnabled(&self, event: &'static str) -> bool {
        let _ = event;
        self.config.image_animation_trace_enabled
    }
    fn ImageAnimationTrace(
        &mut self,
        event: &'static str,
        value: crate::adapter::image_animation::ImageAnimationTraceValue<'_>,
    ) {
        let description = match value {
            ImageAnimationTraceValue::None => String::new(),
            ImageAnimationTraceValue::Duration(d) => format!("{}us", d.InMicroseconds()),
            ImageAnimationTraceValue::State(s) => s.to_string(),
            ImageAnimationTraceValue::Frame { index, duration } => {
                format!("frame={index} duration={}us", duration.InMicroseconds())
            }
        };
        self.events.Emit(event, description)
    }
    fn IsImplThread(&self, provider: HostImplTaskRunnerProviderId) -> bool {
        assert_eq!(provider, self.provider_identity);
        self.executor.RunsTasksInCurrentSequence()
    }
    fn SettingsFeatures(&self) -> &dyn crate::adapter::settings::LayerTreeSettingsFeatures {
        self.features.as_ref()
    }
}
