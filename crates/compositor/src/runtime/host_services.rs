//! Concrete injected service assembly for the canonical CPU host. Each field
//! is an independent native capability; no field owns a Page/Host/tree, and no
//! source compositor decision is implemented in this forwarding assembly.
//! The embedding supplies actual clock/task/Viz/raster/metrics capabilities.
use crate::adapter::begin_frame::BeginFrameArgs;
use crate::adapter::geometry::*;
use crate::adapter::host_picture::{
    HostPictureMutationServices, HostPictureReadEnvironment, HostPictureRegistrationServices,
    HostPictureReleaseServices, HostPictureTileServices,
};
use crate::adapter::input_handler::*;
use crate::adapter::layer_tree_host_impl::*;
use crate::adapter::layer_tree_impl::*;
use crate::adapter::picture_layer_impl::{PictureLayerImplServices, PictureLayerImplTypes};
use crate::adapter::property_types::*;
use crate::adapter::resource_types::SharedImageFormat;
use crate::adapter::scheduler::*;
use crate::adapter::time::*;
use crate::adapter::{image_animation::PaintImageId, input_types::ElementId};
use crate::cc::base::region::Region;
use crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState;
use crate::cc::trees::layer_tree_impl::LayerTreeImpl;
use crate::cc::trees::layer_tree_settings::LayerTreeSettings;
use crate::cc::{
    input::scrollbar::{ScrollbarOrientation, ScrollbarPart},
    trees::{frame_data::FrameData, presentation_time_callback_buffer::PendingCallbacks},
};
use std::{collections::BTreeMap, sync::Arc};

/// Constructors and direct platform operations at the host source callsites.
pub trait CpuHostCoreServices<B: HostImplTypes> {
    fn CreateChildLocalSurfaceIdAllocator(&mut self) -> B::ChildLocalSurfaceIdAllocator;

    fn CreateAverageLagTrackingManager(&mut self) -> B::AverageLagTrackingManager;

    fn CreateEventsMetricsManager(&mut self) -> B::EventsMetricsManager;

    fn DefaultGlobalTileState(&self) -> B::GlobalTileState;

    fn CreateFrameSorter(&mut self) -> B::FrameSorter;

    fn CreateFrameTrackers(&mut self, single_threaded: bool) -> B::FrameSequenceTrackerCollection;

    fn ResolveSequencedTaskRunner(
        &self,
        runner: HostImplTaskRunnerId,
    ) -> std::sync::Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>;

    fn RouteHostRepeatingClosure(
        &self,
        task: HostImplTask,
    ) -> crate::adapter::task_runner::RepeatingClosure;

    fn CreateColorSpaceCache(&mut self, capacity: usize) -> B::ColorSpaceCache;

    fn NewSequenceChecker(&self) -> B::SequenceChecker;

    fn InvalidFrameSinkId(
        &self,
    ) -> <B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId;

    fn CreateImageAnimationTaskRunner(
        &mut self,
        runner: HostImplTaskRunnerId,
    ) -> Box<dyn crate::adapter::image_animation::ImageAnimationTaskRunner>;

    fn CreateStubCompositorFrameReportingController(
        &mut self,
    ) -> Box<B::CompositorFrameReportingController>;

    fn CreateCompositorFrameReportingController(
        &mut self,
        should_report_histograms: bool,
        id: i32,
        is_trees_in_viz_client: bool,
    ) -> Box<B::CompositorFrameReportingController>;

    fn ShouldScrollJankV4MetricReportAndroidAppJankStats(&self) -> bool;

    fn ReportingSetScrollJankOsReporter(
        &mut self,
        controller: &mut B::CompositorFrameReportingController,
        owner: HostImplWeak,
    );

    fn CreateClientResourceProvider(
        &mut self,
        main: HostImplTaskRunnerId,
        impl_runner: HostImplTaskRunnerId,
        flush: HostImplTask,
    ) -> Box<B::ClientResourceProvider>;

    fn MutatorSetDelegate(
        &mut self,
        mutator: &mut <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
        owner: Option<crate::adapter::layer_tree_impl::LayerTreeHostId>,
    );

    fn MutatorCreateEvents(
        &mut self,
        mutator: &mut <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> Box<B::HostMutatorEvents>;

    fn AllocateLayerTreeIdentity(
        &mut self,
    ) -> (
        crate::adapter::layer_tree_impl::LayerTreeId,
        crate::adapter::property_tree::PropertyTreesOwnerId,
    );

    fn TraceHostCreated(&self, id: i32);

    fn TraceHostDeleted(&self, id: i32);

    fn ReportingSetFrameSorter(
        &mut self,
        controller: &mut B::CompositorFrameReportingController,
        sorter: &mut B::FrameSorter,
    );

    fn ReportingSetFrameSequenceTrackerCollection(
        &mut self,
        controller: &mut B::CompositorFrameReportingController,
        trackers: &mut B::FrameSequenceTrackerCollection,
    );

    fn ReportingSetEventLatencyTracker(
        &mut self,
        controller: &mut B::CompositorFrameReportingController,
        owner: Option<crate::adapter::layer_tree_impl::LayerTreeHostId>,
    );

    fn TrackersUpdateSmoothMainThreadHistory(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        modifier: i32,
    );

    fn TrackersSetCustomResultsAddedCallback(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        callback: HostImplWeak,
    );

    fn ResourceProviderShutdownAndReleaseAllResources(
        &mut self,
        provider: &mut B::ClientResourceProvider,
        client: &mut dyn HostUIResourceReleaseClient<B>,
    );

    fn MutatorClearMutators(
        &mut self,
        mutator: &mut <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    );

    fn ReportingClearFrameSequenceTrackerCollection(
        &mut self,
        controller: &mut B::CompositorFrameReportingController,
    );

    fn MonotonicTimeNow(&self) -> crate::adapter::time::TimeTicks;

    fn BrowserControlsFeatureEnabled(
        &self,
        feature: crate::adapter::browser_controls::BrowserControlsFeature,
    ) -> bool;

    fn BindRenderFrameMetadataObserverToCurrentSequence(
        &mut self,
        observer: &mut B::RenderFrameMetadataObserver,
    );

    fn SetRecordRenderingStats(&mut self, stats: HostImplRenderingStatsId, enabled: bool);

    fn ResourcePoolSetResourceUsageLimits(
        &mut self,
        pool: &mut B::ResourcePool,
        bytes: usize,
        resources: usize,
    );

    fn DelegateSetNeedsPrepareTilesOnImplThread(&mut self, delegate: HostImplDelegateId);

    fn FrameSinkWorkerContextProvider(
        &self,
        sink: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId>;

    fn ClientBecameVisible(
        &mut self,
        provider: HostImplContextProviderId,
    ) -> Box<B::ContextVisibility>;

    fn ClientBecameNotVisible(
        &mut self,
        provider: HostImplContextProviderId,
        visibility: Option<Box<B::ContextVisibility>>,
    );

    fn LockRasterContext(
        &mut self,
        provider: HostImplContextProviderId,
    ) -> B::ScopedRasterContextLock;

    fn WorkerClientBecameVisible(
        &mut self,
        lock: &mut B::ScopedRasterContextLock,
    ) -> Box<B::ContextVisibility>;

    fn WorkerClientBecameNotVisible(
        &mut self,
        lock: &mut B::ScopedRasterContextLock,
        visibility: Option<Box<B::ContextVisibility>>,
    );

    fn DelegateSetNeedsCommitOnImplThread(
        &mut self,
        delegate: HostImplDelegateId,
        reason: <B as crate::adapter::input_services::InputHostTypes>::BeginMainFrameReason,
    );

    fn DelegateRenewTreePriority(&mut self, delegate: HostImplDelegateId);

    fn TraceCanDrawReason(&self, reason: &'static str);

    fn ChildLocalSurfaceIdAllocatorCurrent<'a>(
        &self,
        allocator: &'a B::ChildLocalSurfaceIdAllocator,
    ) -> &'a <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::LocalSurfaceId;

    fn FrameSequenceTrackerActiveTypes(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
    ) -> B::ActiveFrameSequenceTrackers;

    fn TakeSavedEventsMetrics(
        &mut self,
        manager: &mut B::EventsMetricsManager,
    ) -> B::EventMetricsList;

    fn SaveActiveEventMetrics(&mut self, manager: &mut B::EventsMetricsManager);

    fn TrackersStartPinchZoom(&mut self, trackers: &mut B::FrameSequenceTrackerCollection);

    fn TrackersStopPinchZoom(&mut self, trackers: &mut B::FrameSequenceTrackerCollection);

    fn DelegateSetHasActiveThreadedScroll(&mut self, delegate: HostImplDelegateId, active: bool);

    fn DelegateSetWaitingForScrollEvent(&mut self, delegate: HostImplDelegateId, waiting: bool);

    fn NewContentForCheckerboardedScrollsEnabled(&self) -> bool;

    fn NewContentForCheckerboardedScrollsPerScroll(&self) -> bool;

    fn RenderFrameMetadataObserverDidEndScroll(
        &mut self,
        observer: &mut B::RenderFrameMetadataObserver,
    );

    fn DelegateSetDeferBeginMainFrameFromImpl(&mut self, delegate: HostImplDelegateId, defer: bool);

    fn MutatorHasScrollLinkedAnimation(
        &self,
        mutator: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
        scroller: crate::adapter::input_types::ElementId,
    ) -> bool;

    fn SetDidScrollForMetrics(&mut self, manager: &mut B::EventsMetricsManager, did_scroll: bool);

    fn DelegateSetNeedsOneBeginImplFrameOnImplThread(&mut self, delegate: HostImplDelegateId);

    fn DelegateIsInsideDraw(&self, delegate: HostImplDelegateId) -> bool;

    fn DelegateSetNeedsRedrawOnImplThread(&mut self, delegate: HostImplDelegateId);

    fn ImageAnimationTraceEnabled(&self, event: &'static str) -> bool;

    fn ImageAnimationTrace(
        &mut self,
        event: &'static str,
        value: crate::adapter::image_animation::ImageAnimationTraceValue<'_>,
    );

    fn IsImplThread(&self, provider: HostImplTaskRunnerProviderId) -> bool;

    fn SettingsFeatures(&self) -> &dyn crate::adapter::settings::LayerTreeSettingsFeatures;
}
pub struct CpuHostServices<B: HostImplTypes + PictureLayerImplTypes> {
    pub tile_execution: std::sync::Arc<std::sync::Mutex<B::TileExecution>>,
    pub client:
        Option<Box<dyn crate::adapter::client_layer_tree_host_impl::ClientHostCapabilities<B>>>,
    pub client_worklet:
        Box<dyn crate::adapter::client_layer_tree_host_impl::ClientWorkletCapabilities<B>>,

    pub picture_environment: Box<dyn HostPictureReadEnvironment>,
    pub picture_services: Box<dyn PictureLayerImplServices<B>>,
    pub picture_tile_services: Box<dyn HostPictureTileServices>,
    pub picture_mutation_services: Box<dyn HostPictureMutationServices>,
    pub picture_registration_services: Box<dyn HostPictureRegistrationServices<B>>,
    pub picture_release_services: Box<dyn HostPictureReleaseServices<B>>,
    pub core: Box<dyn CpuHostCoreServices<B>>,
    pub debug: Box<dyn HostImplDebugServices>,
    pub ui: Box<dyn HostUIResourceServices<B>>,
    pub animation: Option<Box<dyn HostImplHServices<B>>>,
    pub scrollbar_layer: Box<dyn HostImplLayerServices<B>>,
    pub worklet: Box<dyn HostImplJServices<B>>,
    pub begin_frame: Box<dyn HostImplEServices<B>>,
    pub activation: Box<dyn HostImplFServices<B>>,
    pub lifecycle: Box<dyn HostImplAServices<B>>,
    pub input_environment: Box<dyn HostImplInputEnvironmentServices<B>>,
    pub scrollbar_registration: Box<dyn HostImplScrollbarRegistrationServices<B>>,
    pub browser_controls: Box<dyn HostImplBrowserControlsServices>,
    pub frame_build: Box<dyn HostImplBServices<B>>,
    pub resources: Box<dyn HostImplCServices<B>>,
    pub frame_sink: Box<dyn HostImplGServices<B>>,
    pub tree_environment: Box<dyn HostImplTreeHostServices<B>>,
    pub metadata: Box<dyn HostImplDServices<B>>,
    pub task_runners: Box<dyn HostImplTaskRunnerServices>,
    pub resource_returns: Box<dyn HostImplResourceReturnServices<B>>,
    pub frame_sink_callbacks: Box<dyn HostFrameSinkCallbackServices<B>>,
    pub draw: Box<dyn crate::adapter::layer_tree_impl::draw_host::LayerTreeDrawServices>,
    pub surface: Box<dyn crate::adapter::render_surface_impl::RenderSurfaceServices<B>>,
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplServices<B> for CpuHostServices<B> {
    fn WithClientFacet(
        &mut self,
        callback: &mut dyn FnMut(
            &mut dyn crate::adapter::client_layer_tree_host_impl::ClientHostCapabilities<B>,
            &mut dyn HostImplServices<B>,
        ),
    ) {
        let mut facet = self
            .client
            .take()
            .expect("native client facet already lent");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            callback(facet.as_mut(), self)
        }));
        assert!(
            self.client.is_none(),
            "source client facet replaced during callback"
        );
        self.client = Some(facet);
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }
    fn WithAnimationFacet(
        &mut self,
        callback: &mut dyn FnMut(&mut dyn HostImplHServices<B>, &mut dyn HostImplServices<B>),
    ) {
        let mut facet = self
            .animation
            .take()
            .expect("native animation facet already lent");
        // Restore the same allocation even if an external callback unwinds.
        // No default object/second animation state is installed in the slot.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            callback(facet.as_mut(), self);
        }));
        assert!(
            self.animation.is_none(),
            "animation facet replaced during its live call"
        );
        self.animation = Some(facet);
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }

    fn TileExecution(&self) -> std::sync::Arc<std::sync::Mutex<B::TileExecution>> {
        self.tile_execution.clone()
    }
    fn PictureReadEnvironment(&self) -> &dyn HostPictureReadEnvironment {
        self.picture_environment.as_ref()
    }
    fn WithPictureExternalParts(
        &mut self,
        callback: &mut dyn FnMut(
            &dyn HostPictureReadEnvironment,
            &mut dyn PictureLayerImplServices<B>,
            &mut dyn HostPictureTileServices,
            &mut dyn HostPictureMutationServices,
            &mut dyn HostPictureRegistrationServices<B>,
            &mut dyn HostPictureReleaseServices<B>,
        ),
    ) {
        callback(
            self.picture_environment.as_ref(),
            self.picture_services.as_mut(),
            self.picture_tile_services.as_mut(),
            self.picture_mutation_services.as_mut(),
            self.picture_registration_services.as_mut(),
            self.picture_release_services.as_mut(),
        );
    }

    fn CreateChildLocalSurfaceIdAllocator(&mut self) -> B::ChildLocalSurfaceIdAllocator {
        self.core.CreateChildLocalSurfaceIdAllocator()
    }

    fn CreateAverageLagTrackingManager(&mut self) -> B::AverageLagTrackingManager {
        self.core.CreateAverageLagTrackingManager()
    }

    fn CreateEventsMetricsManager(&mut self) -> B::EventsMetricsManager {
        self.core.CreateEventsMetricsManager()
    }

    fn CreateTileManager(
        &mut self,
        owner: crate::adapter::layer_tree_impl::LayerTreeHostId,
        runner: HostImplTaskRunnerId,
        image_worker: B::ImageWorkerTaskRunner,
        task_limit: usize,
        running_on_renderer: bool,
        settings: crate::cc::tiles::tile_manager_settings::TileManagerSettings,
    ) -> B::TileManager {
        let origin = self.core.ResolveSequencedTaskRunner(runner);
        let mut execution = self
            .tile_execution
            .lock()
            .expect("independent native tile capabilities");
        B::CreateResidentTileManager(
            &mut execution,
            owner,
            origin,
            image_worker,
            task_limit,
            running_on_renderer,
            settings,
        )
    }

    fn DefaultGlobalTileState(&self) -> B::GlobalTileState {
        self.core.DefaultGlobalTileState()
    }

    fn CreateFrameSorter(&mut self) -> B::FrameSorter {
        self.core.CreateFrameSorter()
    }

    fn CreateFrameTrackers(&mut self, single_threaded: bool) -> B::FrameSequenceTrackerCollection {
        self.core.CreateFrameTrackers(single_threaded)
    }

    fn ResolveSequencedTaskRunner(
        &self,
        runner: HostImplTaskRunnerId,
    ) -> std::sync::Arc<dyn crate::adapter::task_runner::SequencedTaskRunner> {
        self.core.ResolveSequencedTaskRunner(runner)
    }

    fn RouteHostRepeatingClosure(
        &self,
        task: HostImplTask,
    ) -> crate::adapter::task_runner::RepeatingClosure {
        self.core.RouteHostRepeatingClosure(task)
    }

    fn CreateColorSpaceCache(&mut self, capacity: usize) -> B::ColorSpaceCache {
        self.core.CreateColorSpaceCache(capacity)
    }

    fn NewSequenceChecker(&self) -> B::SequenceChecker {
        self.core.NewSequenceChecker()
    }

    fn InvalidFrameSinkId(
        &self,
    ) -> <B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId {
        self.core.InvalidFrameSinkId()
    }

    fn CreateImageAnimationTaskRunner(
        &mut self,
        runner: HostImplTaskRunnerId,
    ) -> Box<dyn crate::adapter::image_animation::ImageAnimationTaskRunner> {
        self.core.CreateImageAnimationTaskRunner(runner)
    }

    fn CreateStubCompositorFrameReportingController(
        &mut self,
    ) -> Box<B::CompositorFrameReportingController> {
        self.core.CreateStubCompositorFrameReportingController()
    }

    fn CreateCompositorFrameReportingController(
        &mut self,
        should_report_histograms: bool,
        id: i32,
        is_trees_in_viz_client: bool,
    ) -> Box<B::CompositorFrameReportingController> {
        self.core.CreateCompositorFrameReportingController(
            should_report_histograms,
            id,
            is_trees_in_viz_client,
        )
    }

    fn ShouldScrollJankV4MetricReportAndroidAppJankStats(&self) -> bool {
        self.core
            .ShouldScrollJankV4MetricReportAndroidAppJankStats()
    }

    fn ReportingSetScrollJankOsReporter(
        &mut self,
        controller: &mut B::CompositorFrameReportingController,
        owner: HostImplWeak,
    ) {
        self.core
            .ReportingSetScrollJankOsReporter(controller, owner)
    }

    fn CreateClientResourceProvider(
        &mut self,
        main: HostImplTaskRunnerId,
        impl_runner: HostImplTaskRunnerId,
        flush: HostImplTask,
    ) -> Box<B::ClientResourceProvider> {
        self.core
            .CreateClientResourceProvider(main, impl_runner, flush)
    }

    fn MutatorSetDelegate(
        &mut self,
        mutator: &mut <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
        owner: Option<crate::adapter::layer_tree_impl::LayerTreeHostId>,
    ) {
        self.core.MutatorSetDelegate(mutator, owner)
    }

    fn MutatorCreateEvents(
        &mut self,
        mutator: &mut <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> Box<B::HostMutatorEvents> {
        self.core.MutatorCreateEvents(mutator)
    }

    fn AllocateLayerTreeIdentity(
        &mut self,
    ) -> (
        crate::adapter::layer_tree_impl::LayerTreeId,
        crate::adapter::property_tree::PropertyTreesOwnerId,
    ) {
        self.core.AllocateLayerTreeIdentity()
    }

    fn TraceHostCreated(&self, id: i32) {
        self.core.TraceHostCreated(id)
    }

    fn TraceHostDeleted(&self, id: i32) {
        self.core.TraceHostDeleted(id)
    }

    fn ReportingSetFrameSorter(
        &mut self,
        controller: &mut B::CompositorFrameReportingController,
        sorter: &mut B::FrameSorter,
    ) {
        self.core.ReportingSetFrameSorter(controller, sorter)
    }

    fn ReportingSetFrameSequenceTrackerCollection(
        &mut self,
        controller: &mut B::CompositorFrameReportingController,
        trackers: &mut B::FrameSequenceTrackerCollection,
    ) {
        self.core
            .ReportingSetFrameSequenceTrackerCollection(controller, trackers)
    }

    fn ReportingSetEventLatencyTracker(
        &mut self,
        controller: &mut B::CompositorFrameReportingController,
        owner: Option<crate::adapter::layer_tree_impl::LayerTreeHostId>,
    ) {
        self.core.ReportingSetEventLatencyTracker(controller, owner)
    }

    fn TrackersUpdateSmoothMainThreadHistory(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        modifier: i32,
    ) {
        self.core
            .TrackersUpdateSmoothMainThreadHistory(trackers, modifier)
    }

    fn TrackersSetCustomResultsAddedCallback(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        callback: HostImplWeak,
    ) {
        self.core
            .TrackersSetCustomResultsAddedCallback(trackers, callback)
    }

    fn ResourceProviderShutdownAndReleaseAllResources(
        &mut self,
        provider: &mut B::ClientResourceProvider,
        client: &mut dyn HostUIResourceReleaseClient<B>,
    ) {
        self.core
            .ResourceProviderShutdownAndReleaseAllResources(provider, client)
    }

    fn MutatorClearMutators(
        &mut self,
        mutator: &mut <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) {
        self.core.MutatorClearMutators(mutator)
    }

    fn ReportingClearFrameSequenceTrackerCollection(
        &mut self,
        controller: &mut B::CompositorFrameReportingController,
    ) {
        self.core
            .ReportingClearFrameSequenceTrackerCollection(controller)
    }

    fn MonotonicTimeNow(&self) -> crate::adapter::time::TimeTicks {
        self.core.MonotonicTimeNow()
    }

    fn BrowserControlsFeatureEnabled(
        &self,
        feature: crate::adapter::browser_controls::BrowserControlsFeature,
    ) -> bool {
        self.core.BrowserControlsFeatureEnabled(feature)
    }

    fn BindRenderFrameMetadataObserverToCurrentSequence(
        &mut self,
        observer: &mut B::RenderFrameMetadataObserver,
    ) {
        self.core
            .BindRenderFrameMetadataObserverToCurrentSequence(observer)
    }

    fn SetRecordRenderingStats(&mut self, stats: HostImplRenderingStatsId, enabled: bool) {
        self.core.SetRecordRenderingStats(stats, enabled)
    }

    fn ResourcePoolSetResourceUsageLimits(
        &mut self,
        pool: &mut B::ResourcePool,
        bytes: usize,
        resources: usize,
    ) {
        self.core
            .ResourcePoolSetResourceUsageLimits(pool, bytes, resources)
    }

    fn DelegateSetNeedsPrepareTilesOnImplThread(&mut self, delegate: HostImplDelegateId) {
        self.core.DelegateSetNeedsPrepareTilesOnImplThread(delegate)
    }

    fn FrameSinkWorkerContextProvider(
        &self,
        sink: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId> {
        self.core.FrameSinkWorkerContextProvider(sink)
    }

    fn ClientBecameVisible(
        &mut self,
        provider: HostImplContextProviderId,
    ) -> Box<B::ContextVisibility> {
        self.core.ClientBecameVisible(provider)
    }

    fn ClientBecameNotVisible(
        &mut self,
        provider: HostImplContextProviderId,
        visibility: Option<Box<B::ContextVisibility>>,
    ) {
        self.core.ClientBecameNotVisible(provider, visibility)
    }

    fn LockRasterContext(
        &mut self,
        provider: HostImplContextProviderId,
    ) -> B::ScopedRasterContextLock {
        self.core.LockRasterContext(provider)
    }

    fn WorkerClientBecameVisible(
        &mut self,
        lock: &mut B::ScopedRasterContextLock,
    ) -> Box<B::ContextVisibility> {
        self.core.WorkerClientBecameVisible(lock)
    }

    fn WorkerClientBecameNotVisible(
        &mut self,
        lock: &mut B::ScopedRasterContextLock,
        visibility: Option<Box<B::ContextVisibility>>,
    ) {
        self.core.WorkerClientBecameNotVisible(lock, visibility)
    }

    fn DelegateSetNeedsCommitOnImplThread(
        &mut self,
        delegate: HostImplDelegateId,
        reason: <B as crate::adapter::input_services::InputHostTypes>::BeginMainFrameReason,
    ) {
        self.core
            .DelegateSetNeedsCommitOnImplThread(delegate, reason)
    }

    fn DelegateRenewTreePriority(&mut self, delegate: HostImplDelegateId) {
        self.core.DelegateRenewTreePriority(delegate)
    }

    fn TraceCanDrawReason(&self, reason: &'static str) {
        self.core.TraceCanDrawReason(reason)
    }

    fn ChildLocalSurfaceIdAllocatorCurrent<'a>(
        &self,
        allocator: &'a B::ChildLocalSurfaceIdAllocator,
    ) -> &'a <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::LocalSurfaceId {
        self.core.ChildLocalSurfaceIdAllocatorCurrent(allocator)
    }

    fn FrameSequenceTrackerActiveTypes(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
    ) -> B::ActiveFrameSequenceTrackers {
        self.core.FrameSequenceTrackerActiveTypes(trackers)
    }

    fn TakeSavedEventsMetrics(
        &mut self,
        manager: &mut B::EventsMetricsManager,
    ) -> B::EventMetricsList {
        self.core.TakeSavedEventsMetrics(manager)
    }

    fn SaveActiveEventMetrics(&mut self, manager: &mut B::EventsMetricsManager) {
        self.core.SaveActiveEventMetrics(manager)
    }

    fn TrackersStartPinchZoom(&mut self, trackers: &mut B::FrameSequenceTrackerCollection) {
        self.core.TrackersStartPinchZoom(trackers)
    }

    fn TrackersStopPinchZoom(&mut self, trackers: &mut B::FrameSequenceTrackerCollection) {
        self.core.TrackersStopPinchZoom(trackers)
    }

    fn DelegateSetHasActiveThreadedScroll(&mut self, delegate: HostImplDelegateId, active: bool) {
        self.core
            .DelegateSetHasActiveThreadedScroll(delegate, active)
    }

    fn DelegateSetWaitingForScrollEvent(&mut self, delegate: HostImplDelegateId, waiting: bool) {
        self.core
            .DelegateSetWaitingForScrollEvent(delegate, waiting)
    }

    fn NewContentForCheckerboardedScrollsEnabled(&self) -> bool {
        self.core.NewContentForCheckerboardedScrollsEnabled()
    }

    fn NewContentForCheckerboardedScrollsPerScroll(&self) -> bool {
        self.core.NewContentForCheckerboardedScrollsPerScroll()
    }

    fn RenderFrameMetadataObserverDidEndScroll(
        &mut self,
        observer: &mut B::RenderFrameMetadataObserver,
    ) {
        self.core.RenderFrameMetadataObserverDidEndScroll(observer)
    }

    fn DelegateSetDeferBeginMainFrameFromImpl(
        &mut self,
        delegate: HostImplDelegateId,
        defer: bool,
    ) {
        self.core
            .DelegateSetDeferBeginMainFrameFromImpl(delegate, defer)
    }

    fn MutatorHasScrollLinkedAnimation(
        &self,
        mutator: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
        scroller: crate::adapter::input_types::ElementId,
    ) -> bool {
        self.core.MutatorHasScrollLinkedAnimation(mutator, scroller)
    }

    fn SetDidScrollForMetrics(&mut self, manager: &mut B::EventsMetricsManager, did_scroll: bool) {
        self.core.SetDidScrollForMetrics(manager, did_scroll)
    }

    fn DelegateSetNeedsOneBeginImplFrameOnImplThread(&mut self, delegate: HostImplDelegateId) {
        self.core
            .DelegateSetNeedsOneBeginImplFrameOnImplThread(delegate)
    }

    fn DelegateIsInsideDraw(&self, delegate: HostImplDelegateId) -> bool {
        self.core.DelegateIsInsideDraw(delegate)
    }

    fn DelegateSetNeedsRedrawOnImplThread(&mut self, delegate: HostImplDelegateId) {
        self.core.DelegateSetNeedsRedrawOnImplThread(delegate)
    }

    fn ImageAnimationTraceEnabled(&self, event: &'static str) -> bool {
        self.core.ImageAnimationTraceEnabled(event)
    }

    fn ImageAnimationTrace(
        &mut self,
        event: &'static str,
        value: crate::adapter::image_animation::ImageAnimationTraceValue<'_>,
    ) {
        self.core.ImageAnimationTrace(event, value)
    }

    fn IsImplThread(&self, provider: HostImplTaskRunnerProviderId) -> bool {
        self.core.IsImplThread(provider)
    }

    fn SettingsFeatures(&self) -> &dyn crate::adapter::settings::LayerTreeSettingsFeatures {
        self.core.SettingsFeatures()
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplDebugServices for CpuHostServices<B> {
    fn VlogIsOn(&self, level: i32) -> bool {
        self.debug.VlogIsOn(level)
    }

    fn GetClientNameForMetrics(&self) -> Option<&str> {
        self.debug.GetClientNameForMetrics()
    }

    fn DumpCompositorFrameSwitch(&self) -> Option<String> {
        self.debug.DumpCompositorFrameSwitch()
    }

    fn WriteFileInCurrentDirectory(&self, filename: &str, data: &str) -> bool {
        self.debug.WriteFileInCurrentDirectory(filename, data)
    }

    fn BeginVisibilityTrace(&self, owner_id: u64) {
        self.debug.BeginVisibilityTrace(owner_id)
    }

    fn EndVisibilityTrace(&self, owner_id: u64) {
        self.debug.EndVisibilityTrace(owner_id)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostUIResourceServices<B> for CpuHostServices<B> {
    fn TraceUIResourceEvent(&mut self, name: &'static str) {
        self.ui.TraceUIResourceEvent(name)
    }

    fn UIResourceContextProviderExists(&self, sink: HostImplFrameSinkId) -> bool {
        self.ui.UIResourceContextProviderExists(sink)
    }

    fn UIResourceSharedImageInterfaceExists(&self, sink: HostImplFrameSinkId) -> bool {
        self.ui.UIResourceSharedImageInterfaceExists(sink)
    }

    fn UIResourceSupportsScanoutSharedImages(&self, sink: HostImplFrameSinkId) -> bool {
        self.ui.UIResourceSupportsScanoutSharedImages(sink)
    }

    fn CanCreateNativeBufferForFormat(&self, format: SharedImageFormat) -> bool {
        self.ui.CanCreateNativeBufferForFormat(format)
    }

    fn ScaleToCeiledSize(&self, size: Size, scale: f32) -> Size {
        self.ui.ScaleToCeiledSize(size, scale)
    }

    fn SRGBColorSpace(&self) -> B::UIColorSpace {
        self.ui.SRGBColorSpace()
    }

    fn CreateUIResourceSharedImageForSoftwareCompositor(
        &mut self,
        sink: HostImplFrameSinkId,
        format: SharedImageFormat,
        size: Size,
        color_space: &B::UIColorSpace,
        usage: HostUISharedImageUsage,
        debug_label: &'static str,
    ) -> Option<B::SharedImage> {
        self.ui.CreateUIResourceSharedImageForSoftwareCompositor(
            sink,
            format,
            size,
            color_space,
            usage,
            debug_label,
        )
    }

    fn MapUIResourceSharedImage(&mut self, image: &B::SharedImage) -> B::UIScopedMapping {
        self.ui.MapUIResourceSharedImage(image)
    }

    fn CreateUIResourceSharedImage(
        &mut self,
        sink: HostImplFrameSinkId,
        format: SharedImageFormat,
        size: Size,
        color_space: &B::UIColorSpace,
        usage: HostUISharedImageUsage,
        debug_label: &'static str,
        pixels: &[u8],
    ) -> Option<B::SharedImage> {
        self.ui.CreateUIResourceSharedImage(
            sink,
            format,
            size,
            color_space,
            usage,
            debug_label,
            pixels,
        )
    }

    fn MakeN32Premul(&self, size: Size) -> B::UIImageInfo {
        self.ui.MakeN32Premul(size)
    }

    fn NewUIResourceSourceBitmap<'a>(&mut self) -> B::UISourceBitmap<'a> {
        self.ui.NewUIResourceSourceBitmap()
    }

    fn WrapUIResourcePixels<'a>(
        &mut self,
        info: B::UIImageInfo,
        pixels: &'a mut [u8],
        row_bytes: usize,
    ) -> Option<B::UISurface<'a>> {
        self.ui.WrapUIResourcePixels(info, pixels, row_bytes)
    }

    fn RasterUIResourceSurface<'a>(&mut self, info: B::UIImageInfo) -> Option<B::UISurface<'a>> {
        self.ui.RasterUIResourceSurface(info)
    }

    fn MakeUITransferableResource(
        &mut self,
        image: B::SharedImage,
        sync_token: B::UISyncToken,
    ) -> B::UITransferableResource {
        self.ui.MakeUITransferableResource(image, sync_token)
    }

    fn OnCanDrawStateChanged(&mut self, delegate: HostImplDelegateId, can_draw: bool) {
        self.ui.OnCanDrawStateChanged(delegate, can_draw)
    }

    fn NotifyLatencyInfoSwapPromiseMonitor(
        &mut self,
        monitor: HostImplLatencyMonitorId,
    ) -> Option<Box<B::SwapPromise>> {
        self.ui.NotifyLatencyInfoSwapPromiseMonitor(monitor)
    }

    fn OptionalImplThreadTaskRunner(
        &self,
        provider: HostImplTaskRunnerProviderId,
    ) -> Option<HostImplTaskRunnerId> {
        self.ui.OptionalImplThreadTaskRunner(provider)
    }

    fn RunsTasksInCurrentSequence(&self, runner: HostImplTaskRunnerId) -> bool {
        self.ui.RunsTasksInCurrentSequence(runner)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplHServices<B> for CpuHostServices<B> {
    fn MutatorHasImplOnlyScrollAnimatingElement(
        &self,
        host: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> bool {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .MutatorHasImplOnlyScrollAnimatingElement(host)
    }

    fn MutatorHasImplOnlyAutoScrollAnimatingElement(
        &self,
        host: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> bool {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .MutatorHasImplOnlyAutoScrollAnimatingElement(host)
    }

    fn MutatorTickAnimations(
        &mut self,
        owner: crate::adapter::layer_tree_impl::LayerTreeHostId,
        time: crate::adapter::time::TimeTicks,
        is_active: bool,
        events: Option<&mut B::HostMutatorEvents>,
        target: &mut dyn crate::adapter::animation_host::AnimationMutationTarget,
    ) -> HostAnimationTickResult {
        self.animation
            .as_mut()
            .expect("native animation facet is currently lent")
            .MutatorTickAnimations(owner, time, is_active, events, target)
    }

    fn MutatorHasInvalidationAnimation(
        &self,
        host: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> bool {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .MutatorHasInvalidationAnimation(host)
    }

    fn MutatorHasNativePropertyAnimation(
        &self,
        host: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> bool {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .MutatorHasNativePropertyAnimation(host)
    }

    fn MutatorHasViewTransition(
        &self,
        host: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> bool {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .MutatorHasViewTransition(host)
    }

    fn MutatorUpdateAnimationState(
        &mut self,
        owner: crate::adapter::layer_tree_impl::LayerTreeHostId,
        start_ready: bool,
        events: Option<&mut B::HostMutatorEvents>,
        target: &mut dyn crate::adapter::animation_host::AnimationMutationTarget,
    ) -> bool {
        self.animation
            .as_mut()
            .expect("native animation facet currently lent")
            .MutatorUpdateAnimationState(owner, start_ready, events, target)
    }
    fn MutatorActivateAnimations(
        &mut self,
        owner: crate::adapter::layer_tree_impl::LayerTreeHostId,
        events: Option<&mut B::HostMutatorEvents>,
        target: &mut dyn crate::adapter::animation_host::AnimationMutationTarget,
    ) -> bool {
        self.animation
            .as_mut()
            .expect("native animation facet currently lent")
            .MutatorActivateAnimations(owner, events, target)
    }
    fn MutatorEventsIsEmpty(&self, events: &B::HostMutatorEvents) -> bool {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .MutatorEventsIsEmpty(events)
    }

    fn TrackersStartSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        kind: HostAnimationSequence,
    ) {
        self.animation
            .as_mut()
            .expect("native animation facet is currently lent")
            .TrackersStartSequence(trackers, kind)
    }

    fn TrackersStopSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        kind: HostAnimationSequence,
    ) {
        self.animation
            .as_mut()
            .expect("native animation facet is currently lent")
            .TrackersStopSequence(trackers, kind)
    }

    fn DidCompletePageScaleAnimationOnImplThread(&mut self, delegate: HostImplDelegateId) {
        self.animation
            .as_mut()
            .expect("native animation facet is currently lent")
            .DidCompletePageScaleAnimationOnImplThread(delegate)
    }

    fn VideoFrameControllerOnBeginFrame(
        &mut self,
        controller: HostImplVideoFrameControllerId,
        args: &crate::adapter::begin_frame::BeginFrameArgs,
    ) {
        self.animation
            .as_mut()
            .expect("native animation facet is currently lent")
            .VideoFrameControllerOnBeginFrame(controller, args)
    }

    fn SetVideoNeedsBeginFrames(&mut self, delegate: HostImplDelegateId, needs: bool) {
        self.animation
            .as_mut()
            .expect("native animation facet is currently lent")
            .SetVideoNeedsBeginFrames(delegate, needs)
    }

    fn GlobalTileStateTreePriority(
        &self,
        global: &B::GlobalTileState,
    ) -> crate::adapter::scheduler::TreePriority {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .GlobalTileStateTreePriority(global)
    }

    fn NewHostTraceValue(&self) -> Box<B::HostTraceValue> {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .NewHostTraceValue()
    }

    fn NewHostTraceValueJSON(&self) -> B::HostTraceValue {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .NewHostTraceValueJSON()
    }

    fn HostTraceCategoryEnabled(&self, category: &str) -> bool {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .HostTraceCategoryEnabled(category)
    }

    fn TraceTileDeletion(
        &self,
        category: &str,
        event: &str,
        id: crate::adapter::tile::TileTraceId,
        flow: &str,
    ) {
        self.animation
            .as_ref()
            .expect("native animation facet currently lent")
            .TraceTileDeletion(category, event, id, flow);
    }
    fn TraceTileSnapshot(
        &self,
        category: &str,
        value: &mut dyn crate::adapter::property_types::PropertyTrace,
        name: &str,
        id: crate::adapter::tile::TileTraceId,
    ) {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .TraceTileSnapshot(category, value, name, id);
    }
    fn TraceRasterSourceIdRef(
        &self,
        id: crate::adapter::prioritized_tile::RasterSourceTraceId,
        value: &mut dyn crate::adapter::property_types::PropertyTrace,
        name: &str,
    ) {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .TraceRasterSourceIdRef(id, value, name);
    }

    fn TileManagerBasicStateAsValueInto(
        &self,
        manager: &B::TileManager,
        value: &mut B::HostTraceValue,
    ) {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .TileManagerBasicStateAsValueInto(manager, value)
    }

    fn TraceSetHostIdRef(
        &self,
        id: crate::adapter::layer_tree_impl::LayerTreeHostId,
        value: &mut B::HostTraceValue,
        name: &str,
    ) {
        self.animation
            .as_ref()
            .expect("native animation facet is currently lent")
            .TraceSetHostIdRef(id, value, name)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplLayerServices<B> for CpuHostServices<B> {
    fn Layer_Scrollbar_orientation(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> ScrollbarOrientation {
        self.scrollbar_layer
            .Layer_Scrollbar_orientation(tree, layer)
    }

    fn Layer_Scrollbar_scroll_element_id(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> ElementId {
        self.scrollbar_layer
            .Layer_Scrollbar_scroll_element_id(tree, layer)
    }

    fn Layer_Scrollbar_OverlayScrollbarOpacity(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32 {
        self.scrollbar_layer
            .Layer_Scrollbar_OverlayScrollbarOpacity(tree, layer)
    }

    fn Layer_Scrollbar_UsesAuraOverlayAnimator(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool {
        self.scrollbar_layer
            .Layer_Scrollbar_UsesAuraOverlayAnimator(tree, layer)
    }

    fn Layer_Scrollbar_has_find_in_page_tickmarks(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool {
        self.scrollbar_layer
            .Layer_Scrollbar_has_find_in_page_tickmarks(tree, layer)
    }

    fn Layer_Scrollbar_JumpOnTrackClick(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool {
        self.scrollbar_layer
            .Layer_Scrollbar_JumpOnTrackClick(tree, layer)
    }

    fn Layer_Scrollbar_current_pos(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32 {
        self.scrollbar_layer
            .Layer_Scrollbar_current_pos(tree, layer)
    }

    fn Layer_Scrollbar_scroll_layer_length(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32 {
        self.scrollbar_layer
            .Layer_Scrollbar_scroll_layer_length(tree, layer)
    }

    fn Layer_Scrollbar_ComputeThumbQuadRect(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect {
        self.scrollbar_layer
            .Layer_Scrollbar_ComputeThumbQuadRect(tree, layer)
    }

    fn Layer_Scrollbar_SupportsDragSnapBack(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool {
        self.scrollbar_layer
            .Layer_Scrollbar_SupportsDragSnapBack(tree, layer)
    }

    fn Layer_Scrollbar_ForwardTrackRect(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect {
        self.scrollbar_layer
            .Layer_Scrollbar_ForwardTrackRect(tree, layer)
    }

    fn Layer_Scrollbar_BackTrackRect(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect {
        self.scrollbar_layer
            .Layer_Scrollbar_BackTrackRect(tree, layer)
    }

    fn Layer_Scrollbar_BackButtonRect(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect {
        self.scrollbar_layer
            .Layer_Scrollbar_BackButtonRect(tree, layer)
    }

    fn Layer_Scrollbar_ForwardButtonRect(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect {
        self.scrollbar_layer
            .Layer_Scrollbar_ForwardButtonRect(tree, layer)
    }

    fn Layer_Scrollbar_ThumbThickness(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> i32 {
        self.scrollbar_layer
            .Layer_Scrollbar_ThumbThickness(tree, layer)
    }

    fn Layer_Scrollbar_ThumbLength(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> i32 {
        self.scrollbar_layer
            .Layer_Scrollbar_ThumbLength(tree, layer)
    }

    fn Layer_Scrollbar_TrackLength(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32 {
        self.scrollbar_layer
            .Layer_Scrollbar_TrackLength(tree, layer)
    }

    fn Layer_Scrollbar_ScreenSpaceTransform(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::Transform {
        self.scrollbar_layer
            .Layer_Scrollbar_ScreenSpaceTransform(tree, layer)
    }

    fn Layer_Scrollbar_IdentifyScrollbarPart(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
        position: PointF,
    ) -> ScrollbarPart {
        self.scrollbar_layer
            .Layer_Scrollbar_IdentifyScrollbarPart(tree, layer, position)
    }

    fn Layer_Scrollbar_is_overlay_scrollbar(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool {
        self.scrollbar_layer
            .Layer_Scrollbar_is_overlay_scrollbar(tree, layer)
    }

    fn Layer_Scrollbar_bounds(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> crate::adapter::geometry::Size {
        self.scrollbar_layer.Layer_Scrollbar_bounds(tree, layer)
    }

    fn Layer_Scrollbar_device_scale_factor(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32 {
        self.scrollbar_layer
            .Layer_Scrollbar_device_scale_factor(tree, layer)
    }

    fn Layer_Scrollbar_ComputeHitTestableExpandedThumbQuadRect(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect {
        self.scrollbar_layer
            .Layer_Scrollbar_ComputeHitTestableExpandedThumbQuadRect(tree, layer)
    }

    fn Layer_Scrollbar_thumb_thickness_scale_factor(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32 {
        self.scrollbar_layer
            .Layer_Scrollbar_thumb_thickness_scale_factor(tree, layer)
    }

    fn Layer_Scrollbar_SetThumbThicknessScaleFactor(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
        scale: f32,
    ) {
        self.scrollbar_layer
            .Layer_Scrollbar_SetThumbThicknessScaleFactor(tree, layer, scale)
    }

    fn Layer_Scrollbar_CanScrollOrientation(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool {
        self.scrollbar_layer
            .Layer_Scrollbar_CanScrollOrientation(tree, layer)
    }

    fn Layer_Scrollbar_SetOverlayScrollbarLayerOpacityAnimated(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
        opacity: f32,
        fade_out_animation: bool,
    ) {
        self.scrollbar_layer
            .Layer_Scrollbar_SetOverlayScrollbarLayerOpacityAnimated(
                tree,
                layer,
                opacity,
                fade_out_animation,
            )
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplJServices<B> for CpuHostServices<B> {
    fn NotifyAnimationWorkletStateChange(
        &mut self,
        delegate: HostImplDelegateId,
        state: AnimationWorkletMutationState,
        tree_type: ElementListType,
    ) {
        self.worklet
            .NotifyAnimationWorkletStateChange(delegate, state, tree_type)
    }

    fn SetNeedsImplSideInvalidation(
        &mut self,
        delegate: HostImplDelegateId,
        needs_first_draw_on_activation: bool,
    ) {
        self.worklet
            .SetNeedsImplSideInvalidation(delegate, needs_first_draw_on_activation)
    }

    fn IsInSynchronousComposite(&self, delegate: HostImplDelegateId) -> bool {
        self.worklet.IsInSynchronousComposite(delegate)
    }

    fn IsReadyToActivate(&self, delegate: HostImplDelegateId) -> bool {
        self.worklet.IsReadyToActivate(delegate)
    }

    fn RouteFirstScrollCallback(
        &mut self,
        target: HostImplWeak,
        payload: FirstScrollTrackingPayload,
    ) -> crate::adapter::presentation::SuccessfulCallback {
        self.worklet.RouteFirstScrollCallback(target, payload)
    }

    fn UnboundedClientIdentity(
        &mut self,
        owner: LayerTreeHostId,
    ) -> <B as crate::adapter::unbounded::UnboundedTypes>::ClientIdentity {
        self.worklet.UnboundedClientIdentity(owner)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplEServices<B> for CpuHostServices<B> {
    fn ScaleToEnclosingRect(&self, rect: Rect, scale: f32) -> Rect {
        self.begin_frame.ScaleToEnclosingRect(rect, scale)
    }

    fn IsLowEndDevice(&self) -> bool {
        self.begin_frame.IsLowEndDevice()
    }

    fn FrameTokenValue(&self, token: &B::FrameTokenGenerator) -> u32 {
        self.begin_frame.FrameTokenValue(token)
    }

    fn BeginFrameTrace(&mut self) -> &mut dyn crate::adapter::trace_sink::TraceSink {
        self.begin_frame.BeginFrameTrace()
    }

    fn TrackersNotifyBeginImplFrame(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        args: &crate::adapter::begin_frame::BeginFrameArgs,
    ) {
        self.begin_frame
            .TrackersNotifyBeginImplFrame(trackers, args)
    }

    fn ReportingSetNeedsRasterPropertiesAnimated(
        &mut self,
        reporter: &mut B::CompositorFrameReportingController,
        animated: bool,
    ) {
        self.begin_frame
            .ReportingSetNeedsRasterPropertiesAnimated(reporter, animated)
    }

    fn DevtoolsDidBeginFrame(
        &mut self,
        id: i32,
        time: crate::adapter::time::TimeTicks,
        sequence: u64,
    ) {
        self.begin_frame.DevtoolsDidBeginFrame(id, time, sequence)
    }

    fn LocalSurfaceIdIsNewerThan(
        &self,
        target: &<B as LayerTreeTypes>::LocalSurfaceId,
        upcoming: &<B as LayerTreeTypes>::LocalSurfaceId,
    ) -> bool {
        self.begin_frame.LocalSurfaceIdIsNewerThan(target, upcoming)
    }

    fn TrackersNotifyFrameEnd(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        current: &crate::adapter::begin_frame::BeginFrameArgs,
        args: &crate::adapter::begin_frame::BeginFrameArgs,
    ) {
        self.begin_frame
            .TrackersNotifyFrameEnd(trackers, current, args)
    }

    fn TraceDidNotProduceFrame(
        &self,
        ack: &<B as crate::adapter::frame_data::FrameDataTypes>::BeginFrameAck,
        reason_name: &str,
    ) {
        self.begin_frame.TraceDidNotProduceFrame(ack, reason_name)
    }

    fn FrameSinkDidNotProduceFrame(
        &mut self,
        sink: HostImplFrameSinkId,
        ack: &<B as crate::adapter::frame_data::FrameDataTypes>::BeginFrameAck,
        reason: crate::cc::scheduler::scheduler::FrameSkippedReason,
    ) {
        self.begin_frame
            .FrameSinkDidNotProduceFrame(sink, ack, reason)
    }

    fn DropSavedEventMetricsForNoFrameUpdate(&mut self, manager: &mut B::EventsMetricsManager) {
        self.begin_frame
            .DropSavedEventMetricsForNoFrameUpdate(manager)
    }

    fn DidChangeBeginFrameSourcePaused(&mut self, delegate: HostImplDelegateId, paused: bool) {
        self.begin_frame
            .DidChangeBeginFrameSourcePaused(delegate, paused)
    }

    fn SynchronousTaskGraphRunnerRunUntilIdle(
        &mut self,
        runner: &mut B::SynchronousTaskGraphRunner,
    ) {
        self.begin_frame
            .SynchronousTaskGraphRunnerRunUntilIdle(runner)
    }

    fn TraceBuildHitTestData(&self) {
        self.begin_frame.TraceBuildHitTestData()
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplFServices<B> for CpuHostServices<B> {
    fn DelegateDidLoseLayerTreeFrameSink(&mut self, delegate: HostImplDelegateId) {
        self.activation.DelegateDidLoseLayerTreeFrameSink(delegate)
    }

    fn LagTrackingClear(&mut self, lag: &mut B::AverageLagTrackingManager) {
        self.activation.LagTrackingClear(lag)
    }

    fn FrameSorterReset(&mut self, sorter: &mut B::FrameSorter, reset_fcp: bool) {
        self.activation.FrameSorterReset(sorter, reset_fcp)
    }

    fn TraceActivateSyncTree(
        &self,
        trace: &mut dyn FnMut() -> crate::adapter::commit_types::BeginMainFrameTraceId,
    ) {
        self.activation.TraceActivateSyncTree(trace)
    }

    fn TracePendingTreeActivated(
        &self,
        tree: LayerTreeId,
        local: &<B as LayerTreeTypes>::LocalSurfaceId,
    ) {
        self.activation.TracePendingTreeActivated(tree, local)
    }

    fn TraceTreeSynchronization(&self, name: &str, count: Option<usize>) {
        self.activation.TraceTreeSynchronization(name, count)
    }

    fn PromoteScrollTimelinesPendingToActive(
        &mut self,
        mutator: &mut <B as LayerTreeTypes>::MutatorHost,
    ) {
        self.activation
            .PromoteScrollTimelinesPendingToActive(mutator)
    }

    fn InvalidPreviousSurfaceScreenshot(&self) {
        self.activation.InvalidPreviousSurfaceScreenshot()
    }

    fn DelegateDidActivateSyncTree(&mut self, delegate: HostImplDelegateId) {
        self.activation.DelegateDidActivateSyncTree(delegate)
    }

    fn RunTreeActivationCallback(&mut self, callback: &<B as HostImplTypes>::RepeatingClosure) {
        self.activation.RunTreeActivationCallback(callback)
    }

    fn VerboseLogActivatedTree(&self, tree: &LayerTreeImpl<B>) {
        self.activation.VerboseLogActivatedTree(tree)
    }

    fn ReportingSetVisible(
        &mut self,
        reporting: &mut B::CompositorFrameReportingController,
        visible: bool,
    ) {
        self.activation.ReportingSetVisible(reporting, visible)
    }

    fn LayerContextSetVisible(
        &mut self,
        layer_context: &mut <B as HostImplTypes>::LayerContext,
        visible: bool,
    ) {
        self.activation
            .LayerContextSetVisible(layer_context, visible)
    }

    fn ResourceProviderSetVisible(
        &mut self,
        provider: &mut B::ClientResourceProvider,
        visible: bool,
    ) {
        self.activation
            .ResourceProviderSetVisible(provider, visible)
    }

    fn ImageDecodeFactory(&self) -> &B::ImageDecodeFactory {
        self.activation.ImageDecodeFactory()
    }

    fn ImageDecodeContextProvider(
        &self,
        provider: Option<HostImplContextProviderId>,
    ) -> <B::ImageDecodeFactory as HostImplImageDecodeFactory>::RasterContextProvider {
        self.activation.ImageDecodeContextProvider(provider)
    }

    fn ImageDecodeDarkModeFilter(
        &self,
        id: Option<HostImplDarkModeFilterId>,
    ) -> Option<&<B::ImageDecodeFactory as HostImplImageDecodeFactory>::DarkModeFilter> {
        self.activation.ImageDecodeDarkModeFilter(id)
    }

    fn CreateRasterQueryQueue(
        &mut self,
        worker: HostImplContextProviderId,
    ) -> Box<B::RasterQueryQueue> {
        self.activation.CreateRasterQueryQueue(worker)
    }

    fn CreateSynchronousTaskGraphRunner(&mut self) -> Box<B::SynchronousTaskGraphRunner> {
        self.activation.CreateSynchronousTaskGraphRunner()
    }

    fn FrameSinkSharedImageInterface(
        &self,
        sink: HostImplFrameSinkId,
    ) -> HostSharedImageInterfaceId {
        self.activation.FrameSinkSharedImageInterface(sink)
    }

    fn ContextProviderSharedImageInterface(
        &self,
        provider: HostImplContextProviderId,
    ) -> HostSharedImageInterfaceId {
        self.activation
            .ContextProviderSharedImageInterface(provider)
    }

    fn CreateZeroCopyRasterBufferProvider(
        &mut self,
        interface: HostSharedImageInterfaceId,
        is_software: bool,
    ) -> Box<B::RasterBufferProvider> {
        self.activation
            .CreateZeroCopyRasterBufferProvider(interface, is_software)
    }

    fn CreateGpuRasterBufferProvider(
        &mut self,
        interface: HostSharedImageInterfaceId,
        compositor: HostImplContextProviderId,
        worker: HostImplContextProviderId,
        overlay_candidate: bool,
        max_tile_size: Size,
        queries: Option<&B::RasterQueryQueue>,
    ) -> Box<B::RasterBufferProvider> {
        self.activation.CreateGpuRasterBufferProvider(
            interface,
            compositor,
            worker,
            overlay_candidate,
            max_tile_size,
            queries,
        )
    }

    fn MissingWorkerContextForcingZeroCopy(&self) {
        self.activation.MissingWorkerContextForcingZeroCopy()
    }

    fn CreateOneCopyRasterBufferProvider(
        &mut self,
        interface: HostSharedImageInterfaceId,
        runner: HostImplTaskRunnerId,
        compositor: HostImplContextProviderId,
        worker: HostImplContextProviderId,
        use_partial_raster: bool,
        max_staging_bytes: i32,
        overlay_candidate: bool,
    ) -> Box<B::RasterBufferProvider> {
        self.activation.CreateOneCopyRasterBufferProvider(
            interface,
            runner,
            compositor,
            worker,
            use_partial_raster,
            max_staging_bytes,
            overlay_candidate,
        )
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplAServices<B> for CpuHostServices<B> {
    fn TraceHostImplAEvent(&self, name: &'static str) {
        self.lifecycle.TraceHostImplAEvent(name)
    }

    fn DelegateDidPrepareTiles(&mut self, delegate: HostImplDelegateId) {
        self.lifecycle.DelegateDidPrepareTiles(delegate)
    }

    fn CreateLatencyInfoSwapPromiseMonitor(
        &mut self,
        latency: Option<&mut B::LatencyInfo>,
        owner: HostImplWeak,
    ) -> Box<B::LatencyInfoSwapPromiseMonitor> {
        self.lifecycle
            .CreateLatencyInfoSwapPromiseMonitor(latency, owner)
    }

    fn GetScopedEventMetricsMonitor(
        &mut self,
        manager: &mut B::EventsMetricsManager,
        callback: B::EventMetricsDoneCallback,
    ) -> Box<B::EventMetricsScopedMonitor> {
        self.lifecycle
            .GetScopedEventMetricsMonitor(manager, callback)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplInputEnvironmentServices<B>
    for CpuHostServices<B>
{
    fn InputFeatureEnabled(&self, feature: crate::adapter::input_handler::InputFeature) -> bool {
        self.input_environment.InputFeatureEnabled(feature)
    }

    fn InputScrollEventDispatchMode(&self) -> String {
        self.input_environment.InputScrollEventDispatchMode()
    }

    fn InputLateScrollDeadlineRatio(&self) -> f64 {
        self.input_environment.InputLateScrollDeadlineRatio()
    }

    fn InputTrackerType(
        &self,
        kind: crate::adapter::input_handler::ScrollTrackingType,
    ) -> <B as crate::adapter::input_services::InputHostTypes>::FrameSequenceTrackerType {
        self.input_environment.InputTrackerType(kind)
    }

    fn InputDrivingThread(
        &self,
        thread: crate::adapter::input_handler::ScrollingThread,
    ) -> <B as crate::adapter::input_services::InputHostTypes>::SmoothEffectDrivingThread {
        self.input_environment.InputDrivingThread(thread)
    }

    fn InputHistogramCounts(&self, name: &str, value: f32) {
        self.input_environment.InputHistogramCounts(name, value)
    }

    fn InputHistogramTimes(&self, name: &str, value: crate::adapter::time::TimeDelta) {
        self.input_environment.InputHistogramTimes(name, value)
    }

    fn InputBeginTrace(
        &self,
        event: &str,
        fields: &[crate::adapter::input_handler::InputTraceField],
    ) -> <B as crate::adapter::input_handler::InputHandlerTypes>::TraceSpan {
        self.input_environment.InputBeginTrace(event, fields)
    }

    fn InputInstantTrace(
        &self,
        event: &str,
        fields: &[crate::adapter::input_handler::InputTraceField],
    ) {
        self.input_environment.InputInstantTrace(event, fields)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplScrollbarRegistrationServices<B>
    for CpuHostServices<B>
{
    fn PostDelayedAnimationTaskOnImplThread(
        &mut self,
        delegate: HostImplDelegateId,
        task: crate::adapter::scrollbar_animation::ScrollbarAnimationTask,
        delay: crate::adapter::time::TimeDelta,
    ) {
        self.scrollbar_registration
            .PostDelayedAnimationTaskOnImplThread(delegate, task, delay)
    }

    fn TraceScrollbarNeedsAnimate(&self) {
        self.scrollbar_registration.TraceScrollbarNeedsAnimate()
    }

    fn PostDelayedAutoscrollTask(
        &mut self,
        owner: HostImplWeak,
        task: crate::adapter::scrollbar::AutoscrollTask,
        delay: crate::adapter::time::TimeDelta,
    ) {
        self.scrollbar_registration
            .PostDelayedAutoscrollTask(owner, task, delay)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplBrowserControlsServices
    for CpuHostServices<B>
{
    fn EmitBrowserControlsDiagnostic(
        &mut self,
        event: crate::adapter::browser_controls::BrowserControlsDiagnostic,
    ) {
        self.browser_controls.EmitBrowserControlsDiagnostic(event)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplBServices<B> for CpuHostServices<B> {
    fn TraceEmptyDamageRect(&self) {
        self.frame_build.TraceEmptyDamageRect()
    }

    fn TraceCalculateRenderPassesBegin(&self, size: u64, requires_high_res: bool) {
        self.frame_build
            .TraceCalculateRenderPassesBegin(size, requires_high_res)
    }

    fn TraceCalculateRenderPassesEnd(
        &self,
        result: crate::cc::scheduler::draw_result::DrawResult,
        missing_tiles: Option<i32>,
    ) {
        self.frame_build
            .TraceCalculateRenderPassesEnd(result, missing_tiles)
    }

    fn ShouldRecordSubsampledMetric(&self, probability: f64) -> bool {
        self.frame_build.ShouldRecordSubsampledMetric(probability)
    }

    fn CrashKey(&mut self, name: &str, value: HostCrashKeyValue) -> Box<dyn HostCrashKey> {
        self.frame_build.CrashKey(name, value)
    }

    fn DumpWillBeCheck(&self, condition: bool, message: &str) {
        self.frame_build.DumpWillBeCheck(condition, message)
    }

    fn AddVisibleContentArea(&mut self, stats: HostImplRenderingStatsId, area: i64) {
        self.frame_build.AddVisibleContentArea(stats, area)
    }

    fn AddApproximatedVisibleContentArea(&mut self, stats: HostImplRenderingStatsId, area: i64) {
        self.frame_build
            .AddApproximatedVisibleContentArea(stats, area)
    }

    fn TickWorkletAnimations(&mut self, host: &mut <B as LayerTreeTypes>::MutatorHost) {
        self.frame_build.TickWorkletAnimations(host)
    }

    fn TracePrepareToDraw(&self, source_frame_number: i32) {
        self.frame_build.TracePrepareToDraw(source_frame_number)
    }

    fn VerboseLogFrame(&self, frame: &FrameData<B>) {
        self.frame_build.VerboseLogFrame(frame)
    }

    fn BeginMainFrameReasonScroll(&self) -> <B as LayerTreeTypes>::BeginMainFrameReason {
        self.frame_build.BeginMainFrameReasonScroll()
    }

    fn RectOutset(&self, rect: &mut Rect, amount: i32) {
        self.frame_build.RectOutset(rect, amount)
    }

    fn RectInsetsString(&self, outer: Rect, inner: Rect) -> String {
        self.frame_build.RectInsetsString(outer, inner)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplCServices<B> for CpuHostServices<B> {
    fn TraceHostImplCEvent(&mut self, category: &'static str, name: &'static str) {
        self.resources.TraceHostImplCEvent(category, name)
    }

    fn LayerContextSetTargetLocalSurfaceId(
        &mut self,
        context: &mut <B as HostImplTypes>::LayerContext,
        id: &<B as LayerTreeTypes>::LocalSurfaceId,
    ) {
        self.resources
            .LayerContextSetTargetLocalSurfaceId(context, id)
    }

    fn DelegateNotifyReadyToActivate(&mut self, delegate: HostImplDelegateId) {
        self.resources.DelegateNotifyReadyToActivate(delegate)
    }

    fn DelegateNotifyReadyToDraw(&mut self, delegate: HostImplDelegateId) {
        self.resources.DelegateNotifyReadyToDraw(delegate)
    }

    fn UpdateDisplayTile(
        &mut self,
        context: &mut <B as HostImplTypes>::LayerContext,
        layer: &dyn PictureLayerDisplayRead,
        tiling: &dyn HostDisplayTileTiling,
        tile: &B::Tile,
        provider: &mut B::ClientResourceProvider,
        sink: HostImplFrameSinkId,
        update_damage: bool,
    ) {
        self.resources.UpdateDisplayTile(
            context,
            layer,
            tiling,
            tile,
            provider,
            sink,
            update_damage,
        )
    }

    fn DelegateDidReceiveCompositorFrameAckOnImplThread(&mut self, delegate: HostImplDelegateId) {
        self.resources
            .DelegateDidReceiveCompositorFrameAckOnImplThread(delegate)
    }

    fn PresentationFeedbackFailed(&self, details: &B::HostFrameTimingDetails) -> bool {
        self.resources.PresentationFeedbackFailed(details)
    }

    fn DelegateDidPresentCompositorFrameOnImplThread(
        &mut self,
        delegate: HostImplDelegateId,
        frame_token: u32,
        callbacks: PendingCallbacks<B::PresentationFeedback, B::HostFrameTimingDetails>,
        details: &B::HostFrameTimingDetails,
    ) {
        self.resources
            .DelegateDidPresentCompositorFrameOnImplThread(
                delegate,
                frame_token,
                callbacks,
                details,
            )
    }

    fn PostCompositorTask(
        &mut self,
        runner: HostImplTaskRunnerId,
        location: crate::adapter::task_runner::TaskLocation,
        task: HostImplPostedCompositorTask<B::HostFrameTimingDetails>,
    ) {
        self.resources.PostCompositorTask(runner, location, task)
    }

    fn LagTrackingDidPresentCompositorFrame(
        &mut self,
        manager: &mut B::AverageLagTrackingManager,
        frame_token: u32,
        details: &B::HostFrameTimingDetails,
    ) {
        self.resources
            .LagTrackingDidPresentCompositorFrame(manager, frame_token, details)
    }

    fn DelegateNotifyCompositorMetricsTrackerResults(
        &mut self,
        delegate: HostImplDelegateId,
        results: &B::CustomTrackerResults,
    ) {
        self.resources
            .DelegateNotifyCompositorMetricsTrackerResults(delegate, results)
    }

    fn FrameTrackersNotifyPauseFrameProduction(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
    ) {
        self.resources
            .FrameTrackersNotifyPauseFrameProduction(trackers)
    }

    fn DelegateOnDrawForLayerTreeFrameSink(
        &mut self,
        delegate: HostImplDelegateId,
        resourceless: bool,
        skip: bool,
    ) {
        self.resources
            .DelegateOnDrawForLayerTreeFrameSink(delegate, resourceless, skip)
    }

    fn DelegateNotifyTransitionRequestFinished(
        &mut self,
        delegate: HostImplDelegateId,
        sequence_id: u32,
        rects: HostViewTransitionElementResourceRects,
    ) {
        self.resources
            .DelegateNotifyTransitionRequestFinished(delegate, sequence_id, rects)
    }

    fn LocalSurfaceIdIsNewerThanOrEmbeddingChanged(
        &self,
        current: &<B as LayerTreeTypes>::LocalSurfaceId,
        other: &<B as LayerTreeTypes>::LocalSurfaceId,
    ) -> bool {
        self.resources
            .LocalSurfaceIdIsNewerThanOrEmbeddingChanged(current, other)
    }

    fn ResourceProviderSetEvicted(
        &mut self,
        provider: &mut B::ClientResourceProvider,
        evicted: bool,
    ) {
        self.resources.ResourceProviderSetEvicted(provider, evicted)
    }

    fn HasCustomMetricRecorder(&self) -> bool {
        self.resources.HasCustomMetricRecorder()
    }

    fn CustomMetricRecorderReportEventLatency(
        &mut self,
        args: &BeginFrameArgs,
        latencies: Vec<B::EventLatencyData>,
    ) {
        self.resources
            .CustomMetricRecorderReportEventLatency(args, latencies)
    }
    #[cfg(target_os = "android")]
    fn RenderFrameMetadataObserverReportScrollJankStats(
        &mut self,
        observer: &mut B::RenderFrameMetadataObserver,
        total: u32,
        janky: u32,
    ) {
        self.resources
            .RenderFrameMetadataObserverReportScrollJankStats(observer, total, janky)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplGServices<B> for CpuHostServices<B> {
    fn MutatorSetLayerTreeMutator(
        &mut self,
        host: &mut <B as LayerTreeTypes>::MutatorHost,
        mutator: Option<Box<B::LayerTreeMutator>>,
    ) {
        self.frame_sink.MutatorSetLayerTreeMutator(host, mutator)
    }

    fn TraceHostImplGEvent(&mut self, category: &'static str, name: &'static str) {
        self.frame_sink.TraceHostImplGEvent(category, name)
    }

    fn TraceQueueImageDecode(&mut self, image: &B::PaintImage, frame_index: usize) {
        self.frame_sink.TraceQueueImageDecode(image, frame_index)
    }

    fn CreateDrawImageForDecode(
        &mut self,
        image: &<B as crate::adapter::commit_state::CommitStateTypes>::DrawImage,
        scale_adjustment: f32,
        frame_index: usize,
        color: HostTargetColorParams<B::ColorSpace>,
    ) -> <B as crate::adapter::commit_state::CommitStateTypes>::DrawImage {
        self.frame_sink
            .CreateDrawImageForDecode(image, scale_adjustment, frame_index, color)
    }

    fn SendExplicitDecodeRequestsImmediately(&self) -> bool {
        self.frame_sink.SendExplicitDecodeRequestsImmediately()
    }

    fn DelegateNotifyImageDecodeRequestFinished(
        &mut self,
        delegate: HostImplDelegateId,
        request_id: i32,
        speculative: bool,
        succeeded: bool,
    ) {
        self.frame_sink.DelegateNotifyImageDecodeRequestFinished(
            delegate,
            request_id,
            speculative,
            succeeded,
        )
    }

    fn MutatorTakeTimeUpdatedEvents(
        &mut self,
        host: &mut <B as LayerTreeTypes>::MutatorHost,
        events: Option<&mut B::HostMutatorEvents>,
    ) {
        self.frame_sink.MutatorTakeTimeUpdatedEvents(host, events)
    }

    fn DelegateClearHistory(&mut self, delegate: HostImplDelegateId) {
        self.frame_sink.DelegateClearHistory(delegate)
    }

    fn DelegateCommitDurationSampleCountForTesting(&self, delegate: HostImplDelegateId) -> usize {
        self.frame_sink
            .DelegateCommitDurationSampleCountForTesting(delegate)
    }

    fn ResourcePoolInvalidateResources(&mut self, pool: &mut B::ResourcePool) {
        self.frame_sink.ResourcePoolInvalidateResources(pool)
    }

    fn RasterContextOrderingBarrier(&mut self, lock: &mut B::ScopedRasterContextLock) {
        self.frame_sink.RasterContextOrderingBarrier(lock)
    }

    fn FrameTrackersClearAll(&mut self, trackers: &mut B::FrameSequenceTrackerCollection) {
        self.frame_sink.FrameTrackersClearAll(trackers)
    }

    fn FrameSinkAccess(&mut self, sink: HostImplFrameSinkId) -> HostFrameSinkAccess<'_, B> {
        self.frame_sink.FrameSinkAccess(sink)
    }

    fn ResourceProviderReleaseAllExportedResources(
        &mut self,
        provider: &mut B::ClientResourceProvider,
        pool: Option<&mut B::ResourcePool>,
        lose: bool,
        client: &mut dyn HostUIResourceReleaseClient<B>,
    ) {
        self.frame_sink
            .ResourceProviderReleaseAllExportedResources(provider, pool, lose, client)
    }

    fn FrameSinkCreateLayerContext(
        &mut self,
        sink: HostImplFrameSinkId,
        target: HostImplWeak,
        draw_mode: HostDrawMode,
        settings: &crate::cc::trees::layer_tree_settings::LayerTreeSettings,
    ) -> Box<<B as HostImplTypes>::LayerContext> {
        self.frame_sink
            .FrameSinkCreateLayerContext(sink, target, draw_mode, settings)
    }

    fn FrameSinkIdIsValid(
        &self,
        id: &<B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId,
    ) -> bool {
        self.frame_sink.FrameSinkIdIsValid(id)
    }

    fn CreateResourcePool(
        &mut self,
        provider: &mut B::ClientResourceProvider,
        context: Option<HostImplContextProviderId>,
        runner: HostImplTaskRunnerId,
        expiration: TimeDelta,
        disallow_non_exact_reuse: bool,
    ) -> Box<B::ResourcePool> {
        self.frame_sink.CreateResourcePool(
            provider,
            context,
            runner,
            expiration,
            disallow_non_exact_reuse,
        )
    }

    fn DelegateDidObserveFirstScrollDelay(
        &mut self,
        delegate: HostImplDelegateId,
        frame: i32,
        delay: TimeDelta,
        timestamp: TimeTicks,
    ) {
        self.frame_sink
            .DelegateDidObserveFirstScrollDelay(delegate, frame, delay, timestamp)
    }

    fn MutatorImplOnlyAutoScrollAnimationCreate(
        &mut self,
        host: &mut <B as LayerTreeTypes>::MutatorHost,
        id: ElementId,
        target: &PointF,
        current: PointF,
        velocity: f32,
        start_offset: TimeDelta,
    ) {
        self.frame_sink.MutatorImplOnlyAutoScrollAnimationCreate(
            host,
            id,
            target,
            current,
            velocity,
            start_offset,
        )
    }

    fn MutatorImplOnlyScrollAnimationCreate(
        &mut self,
        host: &mut <B as LayerTreeTypes>::MutatorHost,
        id: ElementId,
        target: PointF,
        current: PointF,
        delayed_by: TimeDelta,
        start_offset: TimeDelta,
    ) {
        self.frame_sink.MutatorImplOnlyScrollAnimationCreate(
            host,
            id,
            target,
            current,
            delayed_by,
            start_offset,
        )
    }

    fn UpdateCheckerImageDecodingHints(
        &mut self,
        manager: &mut B::TileManager,
        map: std::collections::BTreeMap<PaintImageId, HostImageDecodingMode>,
    ) {
        self.frame_sink
            .UpdateCheckerImageDecodingHints(manager, map)
    }

    fn RecordAppliedScrollObservation(
        &mut self,
        manager: &mut B::EventsMetricsManager,
        id: ElementId,
    ) {
        self.frame_sink.RecordAppliedScrollObservation(manager, id)
    }

    fn NewContentForCheckerboardedScrollsPerFrame(&self) -> bool {
        self.frame_sink.NewContentForCheckerboardedScrollsPerFrame()
    }

    fn MutatorScrollAnimationAbort(
        &mut self,
        host: &mut <B as LayerTreeTypes>::MutatorHost,
        id: ElementId,
    ) {
        self.frame_sink.MutatorScrollAnimationAbort(host, id)
    }

    fn FrameTrackersStartScrollSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        sequence: B::FrameSequenceTrackerType,
        thread: B::SmoothEffectDrivingThread,
    ) {
        self.frame_sink
            .FrameTrackersStartScrollSequence(trackers, sequence, thread)
    }

    fn FrameTrackersStopSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        sequence: B::FrameSequenceTrackerType,
    ) {
        self.frame_sink
            .FrameTrackersStopSequence(trackers, sequence)
    }

    fn MutatorTickScrollAnimations(
        &mut self,
        host: &mut <B as LayerTreeTypes>::MutatorHost,
        time: TimeTicks,
        trees: &PropertyTreesFor<B>,
    ) {
        self.frame_sink
            .MutatorTickScrollAnimations(host, time, trees)
    }

    fn MutatorElementHasImplOnlyScrollAnimation(
        &self,
        host: &<B as LayerTreeTypes>::MutatorHost,
        id: ElementId,
    ) -> bool {
        self.frame_sink
            .MutatorElementHasImplOnlyScrollAnimation(host, id)
    }

    fn MutatorImplOnlyScrollAnimationUpdateTarget(
        &mut self,
        host: &mut <B as LayerTreeTypes>::MutatorHost,
        delta: Vector2dF,
        max_offset: PointF,
        time: TimeTicks,
        delayed_by: TimeDelta,
        id: ElementId,
    ) -> Option<PointF> {
        self.frame_sink.MutatorImplOnlyScrollAnimationUpdateTarget(
            host, delta, max_offset, time, delayed_by, id,
        )
    }

    fn ChildLocalSurfaceIdAllocatorUpdateFromParent(
        &mut self,
        allocator: &mut B::ChildLocalSurfaceIdAllocator,
        id: &<B as LayerTreeTypes>::LocalSurfaceId,
    ) {
        self.frame_sink
            .ChildLocalSurfaceIdAllocatorUpdateFromParent(allocator, id)
    }

    fn DelegateReturnResource(
        &mut self,
        delegate: HostImplDelegateId,
        resource: <B as crate::adapter::unbounded::UnboundedTypes>::ReturnedResource,
    ) {
        self.frame_sink.DelegateReturnResource(delegate, resource)
    }

    fn FrameSinkNotifyNewLocalSurfaceIdExpectedWhilePaused(&mut self, sink: HostImplFrameSinkId) {
        self.frame_sink
            .FrameSinkNotifyNewLocalSurfaceIdExpectedWhilePaused(sink)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplTreeHostServices<B> for CpuHostServices<B> {
    fn SelectionEdgeVisibilityUsesFullEdge(&self) -> bool {
        self.tree_environment.SelectionEdgeVisibilityUsesFullEdge()
    }

    fn ViewTransitionCaptureAndDisplay(&self) -> bool {
        self.tree_environment.ViewTransitionCaptureAndDisplay()
    }

    fn IsCCSlimmingEnabled(&self) -> bool {
        self.tree_environment.IsCCSlimmingEnabled()
    }

    fn VideoFrameControllerClient(&self, owner: LayerTreeHostId) -> &B::VideoFrameControllerClient {
        self.tree_environment.VideoFrameControllerClient(owner)
    }

    fn TaskRunnerProvider(&self, id: HostImplTaskRunnerProviderId) -> &B::TaskRunnerProvider {
        self.tree_environment.TaskRunnerProvider(id)
    }

    fn WithLayerTreeFrameSink(
        &self,
        id: HostImplFrameSinkId,
        callback: &mut dyn FnMut(&B::LayerTreeFrameSink),
    ) {
        self.tree_environment.WithLayerTreeFrameSink(id, callback)
    }
    fn FrameSinkRasterContext(&self, id: HostImplFrameSinkId) -> Option<&B::RasterContextProvider> {
        self.tree_environment.FrameSinkRasterContext(id)
    }

    fn TraceDidActivateLayerTree(&self, owner: LayerTreeHostId, frame: i32) {
        self.tree_environment
            .TraceDidActivateLayerTree(owner, frame)
    }

    fn MissingPropertyNodeForElementId(
        &self,
        mutation: crate::cc::trees::layer_tree_impl::PropertyMutation,
    ) {
        self.tree_environment
            .MissingPropertyNodeForElementId(mutation)
    }

    fn TraceDelegatedInkMetadataPush(&self, metadata: &B::DelegatedInkMetadata) {
        self.tree_environment
            .TraceDelegatedInkMetadataPush(metadata)
    }

    fn TraceLayersUpdatedCount(&self, count: usize) {
        self.tree_environment.TraceLayersUpdatedCount(count)
    }

    fn TraceImageInvalidationCounts(
        &self,
        no_images: usize,
        no_invalidation: usize,
        invalidated: usize,
    ) {
        self.tree_environment
            .TraceImageInvalidationCounts(no_images, no_invalidation, invalidated)
    }

    fn PushMutatorPropertiesTo(
        &mut self,
        state: &B::UnsafeCommitState,
        target: &mut <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
        trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
    ) {
        self.tree_environment
            .PushMutatorPropertiesTo(state, target, trees)
    }

    fn CreateRenderSurface(
        &mut self,
        tree: crate::adapter::layer_tree_impl::LayerTreeId,
        id: crate::adapter::input_types::ElementId,
    ) -> Box<B::RenderSurface> {
        self.tree_environment.CreateRenderSurface(tree, id)
    }

    fn ReportUnexpectedOopifPageScaleOnce(&mut self, scale: f32) {
        self.tree_environment
            .ReportUnexpectedOopifPageScaleOnce(scale)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplDServices<B> for CpuHostServices<B> {
    fn MakeEventMetricsSet(
        &self,
        main: Vec<B::EventMetrics>,
        saved: B::EventMetricsList,
        raster: Vec<B::EventMetrics>,
    ) -> B::EventMetricsSet {
        self.metadata.MakeEventMetricsSet(main, saved, raster)
    }

    fn CollectScrollEventsFromFrame(
        &mut self,
        manager: &mut B::AverageLagTrackingManager,
        token: u32,
        events: &B::EventMetricsSet,
    ) {
        self.metadata
            .CollectScrollEventsFromFrame(manager, token, events)
    }

    fn FrameSinkSetSourceFrameNumber(&mut self, sink: HostImplFrameSinkId, number: i32) {
        self.metadata.FrameSinkSetSourceFrameNumber(sink, number)
    }

    fn PostCompositorFrameDump(&mut self, frame: &B::Frame, client: bool, viz: bool, token: u32) {
        self.metadata
            .PostCompositorFrameDump(frame, client, viz, token)
    }

    fn VerboseLogCompositorFrame(&self, frame: &B::Frame) {
        self.metadata.VerboseLogCompositorFrame(frame)
    }

    fn FrameSinkExportFrameTiming(&mut self, sink: HostImplFrameSinkId) {
        self.metadata.FrameSinkExportFrameTiming(sink)
    }

    fn TraceSubmitCompositorFrame(&self, trace_id: i64, latency: &[B::LatencyInfo]) {
        self.metadata.TraceSubmitCompositorFrame(trace_id, latency)
    }

    fn FrameSinkSubmitCompositorFrame(
        &mut self,
        sink: HostImplFrameSinkId,
        frame: B::Frame,
        hit_test_data_changed: bool,
    ) {
        self.metadata
            .FrameSinkSubmitCompositorFrame(sink, frame, hit_test_data_changed)
    }

    fn MutatorNextFrameHasPendingRAF(&self, host: &<B as LayerTreeTypes>::MutatorHost) -> bool {
        self.metadata.MutatorNextFrameHasPendingRAF(host)
    }

    fn MutatorHasCanvasInvalidation(&self, host: &<B as LayerTreeTypes>::MutatorHost) -> bool {
        self.metadata.MutatorHasCanvasInvalidation(host)
    }

    fn MutatorHasJSAnimation(&self, host: &<B as LayerTreeTypes>::MutatorHost) -> bool {
        self.metadata.MutatorHasJSAnimation(host)
    }

    fn MutatorMainThreadAnimationsCount(&self, host: &<B as LayerTreeTypes>::MutatorHost) -> usize {
        self.metadata.MutatorMainThreadAnimationsCount(host)
    }

    fn MutatorHasSmilAnimation(&self, host: &<B as LayerTreeTypes>::MutatorHost) -> bool {
        self.metadata.MutatorHasSmilAnimation(host)
    }

    fn StopDrawSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        kind: HostDrawSequence,
    ) {
        self.metadata.StopDrawSequence(trackers, kind)
    }

    fn DidDrawDevtoolsFrame(&self, id: i32, sequence: u64) {
        self.metadata.DidDrawDevtoolsFrame(id, sequence)
    }

    fn IssueImplThreadRenderingStatsEvent(&mut self, stats: HostImplRenderingStatsId) {
        self.metadata.IssueImplThreadRenderingStatsEvent(stats)
    }

    fn FrameSinksToThrottleUpdated(
        &mut self,
        delegate: HostImplDelegateId,
        ids: &std::collections::BTreeSet<
            <B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId,
        >,
    ) {
        self.metadata.FrameSinksToThrottleUpdated(delegate, ids)
    }

    fn NewContentForCheckerboardedScrollsMode(&self) -> HostCheckerboardMode {
        self.metadata.NewContentForCheckerboardedScrollsMode()
    }

    fn TileManagerMemoryStats<'a>(
        &self,
        manager: &'a B::TileManager,
    ) -> &'a crate::cc::resources::memory_history::Entry {
        self.metadata.TileManagerMemoryStats(manager)
    }

    fn TraceScrollDeltaThisFrame(&self, delta: Vector2dF) {
        self.metadata.TraceScrollDeltaThisFrame(delta)
    }

    fn IsNewTrace(&self) -> bool {
        self.metadata.IsNewTrace()
    }

    fn TraceFrameViewerSnapshot(&self, id: i32, value: &B::HostTraceValue) {
        self.metadata.TraceFrameViewerSnapshot(id, value)
    }

    fn UpdateHudTexture(
        &mut self,
        layer: &mut B::Layer,
        mode: crate::cc::layers::draw_mode::DrawMode,
        sink: HostImplFrameSinkId,
        provider: Option<&mut B::ClientResourceProvider>,
        caps: &crate::cc::trees::raster_capabilities::RasterCapabilities,
        passes: &mut Vec<Box<B::RenderPass>>,
    ) {
        self.metadata
            .UpdateHudTexture(layer, mode, sink, provider, caps, passes)
    }

    fn ConstructTransitionDirective(
        &self,
        request: &B::ViewTransitionRequest,
        elements: &BTreeMap<ViewTransitionElementResourceId, B::RenderPassId>,
        colors: &B::DisplayColorSpaces,
        delay: bool,
    ) -> B::TransitionDirective {
        self.metadata
            .ConstructTransitionDirective(request, elements, colors, delay)
    }

    fn MakeFrameDeadline(
        &self,
        time: TimeTicks,
        frames: u32,
        interval: TimeDelta,
        lower_bound: bool,
    ) -> B::FrameDeadline {
        self.metadata
            .MakeFrameDeadline(time, frames, interval, lower_bound)
    }

    fn DelegatedInkIsHovering(&self, metadata: &B::DelegatedInkMetadata) -> bool {
        self.metadata.DelegatedInkIsHovering(metadata)
    }

    fn ObserveRenderFrameSubmission(
        &mut self,
        observer: &mut B::RenderFrameMetadataObserver,
        render_metadata: &crate::cc::trees::render_frame_metadata::RenderFrameMetadata<B>,
        metadata: &mut B::CompositorFrameMetadata,
        force: bool,
    ) {
        self.metadata
            .ObserveRenderFrameSubmission(observer, render_metadata, metadata, force)
    }

    fn AddRendererSwapLatencyTimestamp(&self, latency: &mut B::LatencyInfo, time: TimeTicks) {
        self.metadata.AddRendererSwapLatencyTimestamp(latency, time)
    }

    fn PrepareSendResourcesToParent(
        &mut self,
        provider: &mut B::ClientResourceProvider,
        resources: &[crate::adapter::layer_tree_impl::ResourceId],
        output: &mut Vec<B::TransferableResource>,
        interface: HostSharedImageInterfaceId,
    ) {
        self.metadata
            .PrepareSendResourcesToParent(provider, resources, output, interface)
    }

    fn RecordCompositorFrameQuadCount(&self, client: &str, count: usize) {
        self.metadata.RecordCompositorFrameQuadCount(client, count)
    }

    fn TraceNoDamageEarlyOut(&self) {
        self.metadata.TraceNoDamageEarlyOut()
    }

    fn FrameSinkSetCurrentLocalSurfaceId(
        &mut self,
        sink: HostImplFrameSinkId,
        id: &<B as LayerTreeTypes>::LocalSurfaceId,
    ) {
        self.metadata.FrameSinkSetCurrentLocalSurfaceId(sink, id)
    }

    fn IncrementFrameToken(&self, token: &mut B::FrameTokenGenerator) -> u32 {
        self.metadata.IncrementFrameToken(token)
    }

    fn HasMainThreadAnimation(&self, types: &B::ActiveFrameSequenceTrackers) -> bool {
        self.metadata.HasMainThreadAnimation(types)
    }

    fn HasCompositorThreadAnimation(&self, types: &B::ActiveFrameSequenceTrackers) -> bool {
        self.metadata.HasCompositorThreadAnimation(types)
    }

    fn CloneDelegatedInkMetadata(
        &self,
        metadata: &B::DelegatedInkMetadata,
    ) -> Box<B::DelegatedInkMetadata> {
        self.metadata.CloneDelegatedInkMetadata(metadata)
    }

    fn SetDelegatedInkFrameTime(&self, metadata: &mut B::DelegatedInkMetadata, time: TimeTicks) {
        self.metadata.SetDelegatedInkFrameTime(metadata, time)
    }

    fn TraceDelegatedInkOnCompositorFrame(&self, metadata: &B::DelegatedInkMetadata) {
        self.metadata.TraceDelegatedInkOnCompositorFrame(metadata)
    }

    fn VideoFrameControllerDidDrawFrame(&mut self, controller: HostImplVideoFrameControllerId) {
        self.metadata.VideoFrameControllerDidDrawFrame(controller)
    }

    fn LayerContextUpdateDisplayTreeFrom(
        &mut self,
        layer_context: &mut <B as HostImplTypes>::LayerContext,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        resource_provider: &mut B::ClientResourceProvider,
        shared_image_interface: HostSharedImageInterfaceId,
        damage: Rect,
        has_damage: bool,
        is_flush: bool,
        latency_info: Vec<B::LatencyInfo>,
        tracked: <B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects,
    ) -> TimeTicks {
        self.metadata.LayerContextUpdateDisplayTreeFrom(
            layer_context,
            tree,
            resource_provider,
            shared_image_interface,
            damage,
            has_damage,
            is_flush,
            latency_info,
            tracked,
        )
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplTaskRunnerServices for CpuHostServices<B> {
    fn MainThreadTaskRunner(
        &self,
        provider: crate::adapter::layer_tree_host_impl::HostImplTaskRunnerProviderId,
    ) -> HostImplTaskRunnerId {
        self.task_runners.MainThreadTaskRunner(provider)
    }

    fn ImplThreadTaskRunner(
        &self,
        provider: crate::adapter::layer_tree_host_impl::HostImplTaskRunnerProviderId,
    ) -> HostImplTaskRunnerId {
        self.task_runners.ImplThreadTaskRunner(provider)
    }

    fn HasImplThread(
        &self,
        provider: crate::adapter::layer_tree_host_impl::HostImplTaskRunnerProviderId,
    ) -> bool {
        self.task_runners.HasImplThread(provider)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostImplResourceReturnServices<B>
    for CpuHostServices<B>
{
    fn FrameSinkContextProvider(
        &self,
        sink: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId> {
        self.resource_returns.FrameSinkContextProvider(sink)
    }

    fn PostDelayedCompositorTask(
        &mut self,
        runner: HostImplTaskRunnerId,
        location: crate::adapter::task_runner::TaskLocation,
        task: HostImplPostedCompositorTask<B::HostFrameTimingDetails>,
        delay: TimeDelta,
    ) {
        self.resource_returns
            .PostDelayedCompositorTask(runner, location, task, delay)
    }

    fn ReceiveReturnsFromParent(
        &mut self,
        provider: &mut B::ClientResourceProvider,
        resources: Vec<<B as crate::adapter::unbounded::UnboundedTypes>::ReturnedResource>,
        pool: Option<&mut B::ResourcePool>,
        client: &mut dyn HostUIResourceReleaseClient<B>,
    ) {
        self.resource_returns
            .ReceiveReturnsFromParent(provider, resources, pool, client)
    }

    fn ResourcePoolReduceResourceUsage(&mut self, pool: &mut B::ResourcePool) {
        self.resource_returns.ResourcePoolReduceResourceUsage(pool)
    }

    fn ContextHasSupport(&self, provider: HostImplContextProviderId) -> bool {
        self.resource_returns.ContextHasSupport(provider)
    }

    fn ContextSupportFlushPendingWork(&mut self, provider: HostImplContextProviderId) {
        self.resource_returns
            .ContextSupportFlushPendingWork(provider)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> HostFrameSinkCallbackServices<B>
    for CpuHostServices<B>
{
    fn DelegateSetBeginFrameSource(
        &mut self,
        delegate: HostImplDelegateId,
        source: Option<&<B as crate::adapter::unbounded::UnboundedTypes>::BeginFrameSource>,
    ) {
        self.frame_sink_callbacks
            .DelegateSetBeginFrameSource(delegate, source)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes>
    crate::adapter::layer_tree_impl::draw_host::LayerTreeDrawServices for CpuHostServices<B>
{
    fn SizeOopifEffectSurfacesAtExternalScale(&self) -> bool {
        self.draw.SizeOopifEffectSurfacesAtExternalScale()
    }

    fn AllowUndamagedNonrootRenderPassToSkip(&self) -> bool {
        self.draw.AllowUndamagedNonrootRenderPassToSkip()
    }

    fn ViewTransitionsNewRoundingChange(&self) -> bool {
        self.draw.ViewTransitionsNewRoundingChange()
    }

    fn LogDoubleBlur(&self, first: &str, second: &str) {
        self.draw.LogDoubleBlur(first, second)
    }

    fn LogMaxTextureExceeded(&self, maximum: i32, width: i32, height: i32) {
        self.draw.LogMaxTextureExceeded(maximum, width, height)
    }

    fn TraceCategoryEnabled(&self) -> bool {
        self.draw.TraceCategoryEnabled()
    }

    fn TraceRenderSurfaceReasonCount(&self, name: &str, count: usize) {
        self.draw.TraceRenderSurfaceReasonCount(name, count)
    }

    fn TraceScope(&self, name: &str, count: Option<usize>) {
        self.draw.TraceScope(name, count)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes>
    crate::adapter::render_surface_impl::RenderSurfaceServices<B> for CpuHostServices<B>
{
    fn NextStableIdSequenceValue(&self) -> i32 {
        self.surface.NextStableIdSequenceValue()
    }

    fn TraceMaskLayerGpuMemoryUsage(&self, bytes: usize) {
        self.surface.TraceMaskLayerGpuMemoryUsage(bytes)
    }

    fn LogMaxTextureExceeded(&self, maximum: i32, width: i32, height: i32) {
        self.surface.LogMaxTextureExceeded(maximum, width, height)
    }

    fn CreateRenderPass(&mut self, num_contributors: i32) -> Box<B::RenderPass> {
        self.surface.CreateRenderPass(num_contributors)
    }
}
/// Ownership transfers and source non-owning identities at HostImpl's native
/// constructor. Services are lent only for this call; they do not become host
/// state or retain a callable owner pointer.
pub struct HostInit<B: HostImplTypes> {
    pub owner: LayerTreeHostId,
    pub settings: LayerTreeSettings,
    pub delegate: HostImplDelegateId,
    pub task_runner_provider: HostImplTaskRunnerProviderId,
    pub rendering_stats: HostImplRenderingStatsId,
    pub task_graph_runner: HostImplTaskGraphRunnerId,
    pub mutator_host: Box<<B as LayerTreeTypes>::MutatorHost>,
    pub dark_mode_filter: Option<HostImplDarkModeFilterId>,
    pub id: i32,
    pub image_worker_task_runner: B::ImageWorkerTaskRunner,
    pub scheduling_delegate: HostImplSchedulingDelegateId,
}
impl<B: HostImplTypes + PictureLayerImplTypes> CpuHostServices<B> {
    pub fn CreateHost(
        &mut self,
        init: HostInit<B>,
    ) -> crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<B> {
        crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl::new(
            init.owner,
            &init.settings,
            init.delegate,
            init.task_runner_provider,
            init.rendering_stats,
            init.task_graph_runner,
            init.mutator_host,
            init.dark_mode_filter,
            init.id,
            init.image_worker_task_runner,
            init.scheduling_delegate,
            &mut HostImplContext { services: self },
        )
    }
}

impl<B: HostImplTypes + PictureLayerImplTypes>
    crate::adapter::client_layer_tree_host_impl::ClientHostCapabilities<B> for CpuHostServices<B>
{
    fn TracePendingTreeWaiting(
        &self,
        pending: LayerTreeId,
        active_lsid: &<B as LayerTreeTypes>::LocalSurfaceId,
    ) {
        self.client
            .as_ref()
            .expect("native client facet lent")
            .TracePendingTreeWaiting(pending, active_lsid)
    }
    fn ClientInvalidateFrameSink(&mut self, sink: HostImplFrameSinkId, needs_redraw: bool) {
        self.client
            .as_mut()
            .expect("native client facet lent")
            .ClientInvalidateFrameSink(sink, needs_redraw)
    }
    fn ClientHistogramBoolean(&self, name: &'static str, value: bool) {
        self.client
            .as_ref()
            .expect("native client facet lent")
            .ClientHistogramBoolean(name, value)
    }
    fn ClientFirstContentfulPaintReceived(&self, sorter: &B::FrameSorter) -> bool {
        self.client
            .as_ref()
            .expect("native client facet lent")
            .ClientFirstContentfulPaintReceived(sorter)
    }
    fn ClientOnFirstContentfulPaintReceived(&mut self, sorter: &mut B::FrameSorter) {
        self.client
            .as_mut()
            .expect("native client facet lent")
            .ClientOnFirstContentfulPaintReceived(sorter)
    }
    fn ClientScopedBatchResourceRelease(
        &mut self,
        provider: &mut B::ClientResourceProvider,
    ) -> Box<dyn crate::adapter::client_layer_tree_host_impl::ClientBatchResourceRelease> {
        self.client
            .as_mut()
            .expect("native client facet lent")
            .ClientScopedBatchResourceRelease(provider)
    }
    fn ClientHandleRemovedScrollAnimatingElements(
        &mut self,
        owner: crate::adapter::layer_tree_impl::LayerTreeHostId,
        active: bool,
        target: &mut dyn crate::adapter::animation_host::AnimationMutationTarget,
    ) {
        self.client
            .as_mut()
            .expect("native client facet lent")
            .ClientHandleRemovedScrollAnimatingElements(owner, active, target)
    }
    fn ClientVerboseLog(&self, message: &str) {
        self.client
            .as_ref()
            .expect("native client facet lent")
            .ClientVerboseLog(message)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes>
    crate::adapter::client_layer_tree_host_impl::ClientWorkletCapabilities<B>
    for CpuHostServices<B>
{
    fn ClientTraceCommitComplete(
        &self,
        trace: crate::adapter::commit_types::BeginMainFrameTraceId,
    ) {
        self.client_worklet.ClientTraceCommitComplete(trace)
    }
    fn ClientScrollingThreadIsRaster(&self, trackers: &B::FrameSequenceTrackerCollection) -> bool {
        self.client_worklet.ClientScrollingThreadIsRaster(trackers)
    }
    fn ClientCurrentFrameHadRAF(&self, mutator: &<B as LayerTreeTypes>::MutatorHost) -> bool {
        self.client_worklet.ClientCurrentFrameHadRAF(mutator)
    }
    fn ClientMutatorHasViewTransition(&self, mutator: &<B as LayerTreeTypes>::MutatorHost) -> bool {
        self.client_worklet.ClientMutatorHasViewTransition(mutator)
    }
    fn ClientTakePendingTrackerInfos(
        &mut self,
        mutator: &mut <B as LayerTreeTypes>::MutatorHost,
    ) -> Vec<crate::adapter::client_layer_tree_host_impl::TrackedAnimationSequenceInfo> {
        self.client_worklet.ClientTakePendingTrackerInfos(mutator)
    }
    fn ClientStartSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        sequence: crate::adapter::client_layer_tree_host_impl::ClientSequence,
    ) {
        self.client_worklet.ClientStartSequence(trackers, sequence)
    }
    fn ClientStartCustomSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        id: usize,
    ) {
        self.client_worklet.ClientStartCustomSequence(trackers, id)
    }
    fn ClientStopCustomSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        id: usize,
    ) {
        self.client_worklet.ClientStopCustomSequence(trackers, id)
    }
    fn ClientTakeImagesToInvalidate(
        &mut self,
        tracker: crate::adapter::tile_manager::CheckerImageTrackerId,
    ) -> crate::adapter::image_animation::PaintImageIdFlatSet {
        self.client_worklet.ClientTakeImagesToInvalidate(tracker)
    }
    fn ClientHasOngoingWorkletDispatch(&self, painter: &B::PaintWorkletLayerPainter) -> bool {
        self.client_worklet.ClientHasOngoingWorkletDispatch(painter)
    }
    fn ClientNotifyPaintWorkletStateChange(
        &mut self,
        delegate: HostImplDelegateId,
        state: crate::cc::scheduler::scheduler_state_machine::PaintWorkletState,
    ) {
        self.client_worklet
            .ClientNotifyPaintWorkletStateChange(delegate, state)
    }
    fn ClientDispatchWorklets(
        &mut self,
        painter: &mut B::PaintWorkletLayerPainter,
        jobs: crate::adapter::client_layer_tree_host_impl::PaintWorkletJobMap<B>,
        owner: HostImplWeak,
    ) -> Option<crate::adapter::client_layer_tree_host_impl::PaintWorkletJobMap<B>> {
        self.client_worklet
            .ClientDispatchWorklets(painter, jobs, owner)
    }
}
impl<B: HostImplTypes + PictureLayerImplTypes> CpuHostServices<B> {
    pub fn CreateClientHost(
        &mut self,
        init: HostInit<B>,
    ) -> crate::cc::trees::client_layer_tree_host_impl::ClientLayerTreeHostImpl<B> {
        crate::cc::trees::client_layer_tree_host_impl::ClientLayerTreeHostImpl::Create(
            init.owner,
            &init.settings,
            init.delegate,
            init.task_runner_provider,
            init.rendering_stats,
            init.task_graph_runner,
            init.mutator_host,
            init.dark_mode_filter,
            init.id,
            init.image_worker_task_runner,
            init.scheduling_delegate,
            &mut HostImplContext { services: self },
        )
    }
}
