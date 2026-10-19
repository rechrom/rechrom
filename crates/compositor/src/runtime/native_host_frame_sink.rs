//! Shared native frame-sink callbacks and resource-return transport.
//!
//! Chromium correspondence:
//! - cc/trees/layer_tree_host_impl.cc:4598-4664
//! - components/viz/client/client_resource_provider.cc:275-407
//!
//! The endpoint and provider name the same Page-local allocations used by frame
//! submission. Returned resources are applied directly to that resident table.

use super::{
    client_resources::ResidentPoolRelease,
    execution::{EngineTask, EngineTaskQueue},
    host_owners::{ClientResourceProvider, NativePoolReleaseClient},
    native_frame_sink::{NativeFrameSinkEndpoint, NativeFrameSinkOperationsHandle},
    native_host_lifecycle::NativeHostLifecycle,
    native_services::{NativeEventSink, NativeTaskRunnerServices},
    pool_services::{CpuPoolConfig, CpuPoolServices},
    tile_execution::CpuTileExecution,
};
use crate::{
    adapter::{
        layer_tree_host_impl::{
            HostDrawMode, HostFrameSinkAccess, HostFrameSinkCallbackServices,
            HostImageDecodingMode, HostImplContextProviderId, HostImplDelegateId,
            HostImplFrameSinkId, HostImplGServices, HostImplPostedCompositorTask,
            HostImplResourceReturnServices, HostImplTaskRunnerId, HostImplTaskRunnerProviderId,
            HostImplTaskRunnerServices, HostImplWeak, HostSharedImageInterfaceId,
            HostTargetColorParams, HostUIResourceReleaseClient,
        },
        task_runner::TaskLocation,
        time::TimeDelta,
    },
    runtime::{
        host_metrics::FrameSequenceTrackers,
        host_owners::{
            ChildLocalSurfaceIdAllocator, LayerContext, MutatorEvents, RasterContextLock,
        },
        input::MutatorHost,
        metadata::FrameTimingDetails,
        paint::{CpuDrawImage, CpuPaintImage},
        resources::{CpuColorSpace, CpuReturnedResource},
        types::{FrameSinkId, LocalSurfaceId},
    },
    NativeCompositorTypes,
};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
    sync::{Arc, Mutex},
};

pub(crate) struct NativeHostFrameSinkCallbacks {
    endpoint: Arc<Mutex<NativeFrameSinkEndpoint>>,
    lifecycle: Arc<NativeHostLifecycle>,
    task_runners: Arc<NativeTaskRunnerServices>,
    commands: EngineTaskQueue,
}

impl NativeHostFrameSinkCallbacks {
    pub(crate) fn new(
        endpoint: Arc<Mutex<NativeFrameSinkEndpoint>>,
        lifecycle: Arc<NativeHostLifecycle>,
        task_runners: Arc<NativeTaskRunnerServices>,
        commands: EngineTaskQueue,
    ) -> Self {
        Self {
            endpoint,
            lifecycle,
            task_runners,
            commands,
        }
    }

    fn CheckSink(&self, sink: HostImplFrameSinkId) {
        assert_eq!(
            self.endpoint
                .lock()
                .expect("native frame-sink endpoint")
                .id(),
            sink,
            "HostImpl used a different frame-sink endpoint"
        );
    }
}

impl HostImplTaskRunnerServices for NativeHostFrameSinkCallbacks {
    fn MainThreadTaskRunner(&self, provider: HostImplTaskRunnerProviderId) -> HostImplTaskRunnerId {
        self.task_runners.MainThreadTaskRunner(provider)
    }
    fn ImplThreadTaskRunner(&self, provider: HostImplTaskRunnerProviderId) -> HostImplTaskRunnerId {
        self.task_runners.ImplThreadTaskRunner(provider)
    }
    fn HasImplThread(&self, provider: HostImplTaskRunnerProviderId) -> bool {
        self.task_runners.HasImplThread(provider)
    }
}

impl HostImplResourceReturnServices<NativeCompositorTypes> for NativeHostFrameSinkCallbacks {
    fn FrameSinkContextProvider(
        &self,
        sink: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId> {
        self.CheckSink(sink);
        None
    }
    fn PostDelayedCompositorTask(
        &mut self,
        runner: HostImplTaskRunnerId,
        _location: TaskLocation,
        task: HostImplPostedCompositorTask<FrameTimingDetails>,
        delay: TimeDelta,
    ) {
        let executor = self
            .task_runners
            .Resolve(runner)
            .unwrap_or_else(|| panic!("unknown compositor task runner {}", runner.0))
            .clone();
        let commands = self.commands.clone();
        executor.ExecuteAfter(
            delay,
            Box::new(move || commands.Push(EngineTask::PostedHost(task))),
        )
    }
    fn ReceiveReturnsFromParent(
        &mut self,
        provider: &mut ClientResourceProvider<NativeCompositorTypes>,
        resources: Vec<CpuReturnedResource>,
        pool: Option<&mut crate::cc::resources::resource_pool::ResourcePool<NativeCompositorTypes>>,
        client: &mut dyn HostUIResourceReleaseClient<NativeCompositorTypes>,
    ) {
        let mut pool_release = pool.map(ResidentPoolRelease);
        provider.0.ReceiveReturnsFromParent(
            resources,
            pool_release
                .as_mut()
                .map(|release| release as &mut dyn super::host_owners::NativePoolReleaseClient),
            client,
        )
    }
    fn ResourcePoolReduceResourceUsage(
        &mut self,
        pool: &mut crate::cc::resources::resource_pool::ResourcePool<NativeCompositorTypes>,
    ) {
        pool.ReduceResourceUsage()
    }
    fn ContextHasSupport(&self, _provider: HostImplContextProviderId) -> bool {
        false
    }
    fn ContextSupportFlushPendingWork(&mut self, _provider: HostImplContextProviderId) {
        panic!("CPU compositor has no GPU context support")
    }
}

impl HostFrameSinkCallbackServices<NativeCompositorTypes> for NativeHostFrameSinkCallbacks {
    fn DelegateSetBeginFrameSource(
        &mut self,
        delegate: crate::adapter::layer_tree_host_impl::HostImplDelegateId,
        source: Option<&Arc<dyn crate::adapter::scheduler_host::BeginFrameSource>>,
    ) {
        self.lifecycle.SetBeginFrameSource(delegate, source)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct NativeHostFrameSinkConfig {
    pub send_explicit_decode_requests_immediately: bool,
    pub new_content_per_frame: bool,
    pub pool: CpuPoolConfig,
}

/// The complete HostImpl G facade. Frame-sink and resource operations are real
/// resident owners. Animation entry points that require an actual animation
/// model are forwarded to the same native mutator host and never synthesize a
/// compositor-side animation state.
pub(crate) struct NativeHostFrameSinkServices {
    operations: NativeFrameSinkOperationsHandle,
    callbacks: NativeHostFrameSinkCallbacks,
    lifecycle: Arc<NativeHostLifecycle>,
    events: Arc<dyn NativeEventSink>,
    resources: super::client_resources::CpuClientResources,
    tile_execution: Arc<Mutex<CpuTileExecution<super::skia::SkRegion>>>,
    commands: EngineTaskQueue,
    config: NativeHostFrameSinkConfig,
    timing: Arc<super::native_scheduler::NativeTimingHistory>,
    animations: super::native_animation::NativeAnimationOwner,
}

impl NativeHostFrameSinkServices {
    pub(crate) fn new(
        endpoint: Arc<Mutex<NativeFrameSinkEndpoint>>,
        lifecycle: Arc<NativeHostLifecycle>,
        events: Arc<dyn NativeEventSink>,
        resources: super::client_resources::CpuClientResources,
        tile_execution: Arc<Mutex<CpuTileExecution<super::skia::SkRegion>>>,
        task_runners: Arc<NativeTaskRunnerServices>,
        commands: EngineTaskQueue,
        config: NativeHostFrameSinkConfig,
        timing: Arc<super::native_scheduler::NativeTimingHistory>,
        animations: super::native_animation::NativeAnimationOwner,
    ) -> Self {
        Self {
            operations: NativeFrameSinkOperationsHandle::new(endpoint.clone()),
            callbacks: NativeHostFrameSinkCallbacks::new(
                endpoint,
                lifecycle.clone(),
                task_runners,
                commands.clone(),
            ),
            lifecycle,
            events,
            resources,
            tile_execution,
            commands,
            config,
            timing,
            animations,
        }
    }
}

impl HostImplTaskRunnerServices for NativeHostFrameSinkServices {
    fn MainThreadTaskRunner(&self, provider: HostImplTaskRunnerProviderId) -> HostImplTaskRunnerId {
        self.callbacks.MainThreadTaskRunner(provider)
    }
    fn ImplThreadTaskRunner(&self, provider: HostImplTaskRunnerProviderId) -> HostImplTaskRunnerId {
        self.callbacks.ImplThreadTaskRunner(provider)
    }
    fn HasImplThread(&self, provider: HostImplTaskRunnerProviderId) -> bool {
        self.callbacks.HasImplThread(provider)
    }
}
impl HostImplResourceReturnServices<NativeCompositorTypes> for NativeHostFrameSinkServices {
    fn FrameSinkContextProvider(
        &self,
        sink: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId> {
        self.callbacks.FrameSinkContextProvider(sink)
    }
    fn PostDelayedCompositorTask(
        &mut self,
        runner: HostImplTaskRunnerId,
        location: TaskLocation,
        task: HostImplPostedCompositorTask<FrameTimingDetails>,
        delay: TimeDelta,
    ) {
        self.callbacks
            .PostDelayedCompositorTask(runner, location, task, delay)
    }
    fn ReceiveReturnsFromParent(
        &mut self,
        provider: &mut ClientResourceProvider<NativeCompositorTypes>,
        resources: Vec<CpuReturnedResource>,
        pool: Option<&mut crate::cc::resources::resource_pool::ResourcePool<NativeCompositorTypes>>,
        client: &mut dyn HostUIResourceReleaseClient<NativeCompositorTypes>,
    ) {
        self.callbacks
            .ReceiveReturnsFromParent(provider, resources, pool, client)
    }
    fn ResourcePoolReduceResourceUsage(
        &mut self,
        pool: &mut crate::cc::resources::resource_pool::ResourcePool<NativeCompositorTypes>,
    ) {
        self.callbacks.ResourcePoolReduceResourceUsage(pool)
    }
    fn ContextHasSupport(&self, provider: HostImplContextProviderId) -> bool {
        self.callbacks.ContextHasSupport(provider)
    }
    fn ContextSupportFlushPendingWork(&mut self, provider: HostImplContextProviderId) {
        self.callbacks.ContextSupportFlushPendingWork(provider)
    }
}
impl HostFrameSinkCallbackServices<NativeCompositorTypes> for NativeHostFrameSinkServices {
    fn DelegateSetBeginFrameSource(
        &mut self,
        delegate: HostImplDelegateId,
        source: Option<&Arc<dyn crate::adapter::scheduler_host::BeginFrameSource>>,
    ) {
        self.callbacks.DelegateSetBeginFrameSource(delegate, source)
    }
}

static NEXT_POOL_ID: AtomicU64 = AtomicU64::new(1);
impl HostImplGServices<NativeCompositorTypes> for NativeHostFrameSinkServices {
    fn MutatorSetLayerTreeMutator(
        &mut self,
        host: &mut MutatorHost,
        mutator: Option<Box<super::host_owners::LayerTreeMutator>>,
    ) {
        host.SetLayerTreeMutator(mutator)
    }
    fn TraceHostImplGEvent(&mut self, category: &'static str, name: &'static str) {
        self.events.Emit(category, name.into())
    }
    fn TraceQueueImageDecode(&mut self, image: &CpuPaintImage, frame_index: usize) {
        use crate::adapter::layer_tree_host_impl::HostPaintImage;
        self.events.Emit(
            "compositor.image_decode",
            format!("image={} frame={frame_index}", image.stable_id().0),
        )
    }
    fn CreateDrawImageForDecode(
        &mut self,
        image: &CpuDrawImage,
        scale_adjustment: f32,
        frame_index: usize,
        color: HostTargetColorParams<CpuColorSpace>,
    ) -> CpuDrawImage {
        let mut result = image.clone();
        result.scale = crate::adapter::geometry::Vector2dF::new(
            image.scale.x() * scale_adjustment,
            image.scale.y() * scale_adjustment,
        );
        result.frame_index = frame_index;
        result.target = Some(color);
        result
    }
    fn SendExplicitDecodeRequestsImmediately(&self) -> bool {
        self.config.send_explicit_decode_requests_immediately
    }
    fn DelegateNotifyImageDecodeRequestFinished(
        &mut self,
        delegate: HostImplDelegateId,
        request_id: i32,
        speculative: bool,
        succeeded: bool,
    ) {
        self.lifecycle
            .ImageDecodeRequestFinished(delegate, request_id, speculative, succeeded)
    }
    fn MutatorTakeTimeUpdatedEvents(
        &mut self,
        host: &mut MutatorHost,
        events: Option<&mut MutatorEvents>,
    ) {
        self.CheckAnimationOwner(host);
        self.animations.TakeTimeUpdatedEvents(events)
    }
    fn DelegateClearHistory(&mut self, delegate: HostImplDelegateId) {
        self.lifecycle.ClearHistory(delegate)
    }
    fn DelegateCommitDurationSampleCountForTesting(&self, _delegate: HostImplDelegateId) -> usize {
        // Source: cc/trees/proxy_impl.cc:1078-1080. The scheduler and this
        // endpoint share the very same native timing-history allocation.
        crate::adapter::scheduler_host::CompositorTimingHistory::CommitDurationSampleCountForTesting(
            self.timing.as_ref(),
        )
    }
    fn ResourcePoolInvalidateResources(
        &mut self,
        pool: &mut crate::cc::resources::resource_pool::ResourcePool<NativeCompositorTypes>,
    ) {
        pool.InvalidateResources()
    }
    fn RasterContextOrderingBarrier(&mut self, _lock: &mut RasterContextLock) {
        panic!("CPU compositor has no raster context ordering barrier")
    }
    fn FrameTrackersClearAll(
        &mut self,
        trackers: &mut FrameSequenceTrackers<NativeCompositorTypes>,
    ) {
        trackers.0.FrameTrackersClearAll()
    }
    fn FrameSinkAccess(
        &mut self,
        sink: HostImplFrameSinkId,
    ) -> HostFrameSinkAccess<'_, NativeCompositorTypes> {
        self.callbacks.CheckSink(sink);
        HostFrameSinkAccess {
            sink: &mut self.operations,
            callbacks: &mut self.callbacks,
        }
    }
    fn ResourceProviderReleaseAllExportedResources(
        &mut self,
        provider: &mut ClientResourceProvider<NativeCompositorTypes>,
        pool: Option<&mut crate::cc::resources::resource_pool::ResourcePool<NativeCompositorTypes>>,
        lose: bool,
        client: &mut dyn HostUIResourceReleaseClient<NativeCompositorTypes>,
    ) {
        let mut pool_release = pool.map(ResidentPoolRelease);
        provider.0.ReleaseAllExportedResources(
            pool_release
                .as_mut()
                .map(|release| release as &mut dyn NativePoolReleaseClient),
            lose,
            client,
        )
    }
    fn FrameSinkCreateLayerContext(
        &mut self,
        _sink: HostImplFrameSinkId,
        _target: HostImplWeak,
        _draw_mode: HostDrawMode,
        _settings: &crate::cc::trees::layer_tree_settings::LayerTreeSettings,
    ) -> Box<LayerContext> {
        panic!("TreesInViz layer context is not configured for the CPU compositor")
    }
    fn FrameSinkIdIsValid(&self, id: &FrameSinkId) -> bool {
        id.client_id != 0 && id.sink_id != 0
    }
    fn CreateResourcePool(
        &mut self,
        _provider: &mut ClientResourceProvider<NativeCompositorTypes>,
        context: Option<HostImplContextProviderId>,
        runner: HostImplTaskRunnerId,
        expiration: TimeDelta,
        disallow_non_exact_reuse: bool,
    ) -> Box<crate::cc::resources::resource_pool::ResourcePool<NativeCompositorTypes>> {
        let id = crate::adapter::resource_pool::ResourcePoolId(
            NEXT_POOL_ID.fetch_add(1, Ordering::Relaxed),
        );
        assert_ne!(id.0, 0, "resource-pool identity exhausted");
        let executor = self
            .callbacks
            .task_runners
            .Resolve(runner)
            .expect("resource-pool runner identity")
            .clone();
        let services = Arc::new(CpuPoolServices::new(
            id,
            self.resources.clone(),
            executor,
            self.commands.clone(),
            self.config.pool,
            self.events.clone(),
        ));
        Box::new(crate::cc::resources::resource_pool::ResourcePool::new(
            id,
            services,
            context.is_some(),
            expiration,
            disallow_non_exact_reuse,
        ))
    }
    fn DelegateDidObserveFirstScrollDelay(
        &mut self,
        delegate: HostImplDelegateId,
        frame: i32,
        delay: TimeDelta,
        timestamp: crate::adapter::time::TimeTicks,
    ) {
        self.lifecycle
            .FirstScrollDelay(delegate, frame, delay, timestamp)
    }
    fn MutatorImplOnlyAutoScrollAnimationCreate(
        &mut self,
        host: &mut MutatorHost,
        id: crate::adapter::input_types::ElementId,
        target: &crate::adapter::geometry::PointF,
        current: crate::adapter::geometry::PointF,
        velocity: f32,
        start_offset: TimeDelta,
    ) {
        self.CheckAnimationOwner(host);
        self.animations
            .CreateAutoScrollAnimation(id, *target, current, velocity, start_offset)
    }
    fn MutatorImplOnlyScrollAnimationCreate(
        &mut self,
        host: &mut MutatorHost,
        id: crate::adapter::input_types::ElementId,
        target: crate::adapter::geometry::PointF,
        current: crate::adapter::geometry::PointF,
        delayed_by: TimeDelta,
        start_offset: TimeDelta,
    ) {
        self.CheckAnimationOwner(host);
        self.animations.CreateMouseWheelScrollAnimation(
            id,
            target,
            current,
            delayed_by,
            start_offset,
        )
    }
    fn UpdateCheckerImageDecodingHints(
        &mut self,
        manager: &mut crate::cc::tiles::tile_manager::TileManager<NativeCompositorTypes>,
        map: BTreeMap<crate::adapter::image_animation::PaintImageId, HostImageDecodingMode>,
    ) {
        let id = manager.checker_image_tracker_id();
        self.tile_execution
            .lock()
            .expect("native tile execution")
            .images
            .UpdateCheckerImageDecodingHints(id, map)
    }
    fn RecordAppliedScrollObservation(
        &mut self,
        manager: &mut super::host_owners::EventsMetricsManager,
        id: crate::adapter::input_types::ElementId,
    ) {
        manager.0.DidApplyScroll(id)
    }
    fn NewContentForCheckerboardedScrollsPerFrame(&self) -> bool {
        self.config.new_content_per_frame
    }
    fn MutatorScrollAnimationAbort(
        &mut self,
        host: &mut MutatorHost,
        id: crate::adapter::input_types::ElementId,
    ) {
        self.CheckAnimationOwner(host);
        self.animations.AbortScrollAnimation(id, false)
    }
    fn FrameTrackersStartScrollSequence(
        &mut self,
        trackers: &mut FrameSequenceTrackers<NativeCompositorTypes>,
        sequence: super::input::FrameSequenceTrackerType,
        thread: super::input::SmoothEffectDrivingThread,
    ) {
        trackers
            .0
            .FrameTrackersStartScrollSequence(sequence, thread)
    }
    fn FrameTrackersStopSequence(
        &mut self,
        trackers: &mut FrameSequenceTrackers<NativeCompositorTypes>,
        sequence: super::input::FrameSequenceTrackerType,
    ) {
        trackers.0.FrameTrackersStopSequence(sequence)
    }
    fn MutatorTickScrollAnimations(
        &mut self,
        host: &mut MutatorHost,
        time: crate::adapter::time::TimeTicks,
        trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<NativeCompositorTypes>,
    ) {
        self.CheckAnimationOwner(host);
        self.animations.TickMutator(time, trees, true)
    }
    fn MutatorElementHasImplOnlyScrollAnimation(
        &self,
        host: &MutatorHost,
        id: crate::adapter::input_types::ElementId,
    ) -> bool {
        self.CheckAnimationOwner(host);
        self.animations.ElementHasImplOnlyScrollAnimation(id)
    }
    fn MutatorImplOnlyScrollAnimationUpdateTarget(
        &mut self,
        host: &mut MutatorHost,
        delta: crate::adapter::geometry::Vector2dF,
        max_offset: crate::adapter::geometry::PointF,
        time: crate::adapter::time::TimeTicks,
        delayed_by: TimeDelta,
        id: crate::adapter::input_types::ElementId,
    ) -> Option<crate::adapter::geometry::PointF> {
        self.CheckAnimationOwner(host);
        self.animations
            .UpdateScrollAnimationTarget(delta, max_offset, time, delayed_by, id)
    }
    fn ChildLocalSurfaceIdAllocatorUpdateFromParent(
        &mut self,
        allocator: &mut ChildLocalSurfaceIdAllocator,
        id: &LocalSurfaceId,
    ) {
        allocator.0.UpdateFromParent(id)
    }
    fn DelegateReturnResource(
        &mut self,
        delegate: HostImplDelegateId,
        resource: CpuReturnedResource,
    ) {
        self.lifecycle.ReturnResource(delegate, resource)
    }
    fn FrameSinkNotifyNewLocalSurfaceIdExpectedWhilePaused(&mut self, sink: HostImplFrameSinkId) {
        self.lifecycle.NewLocalSurfaceIdExpectedWhilePaused(sink)
    }
}

impl NativeHostFrameSinkServices {
    fn CheckAnimationOwner(&self, host: &MutatorHost) {
        assert_eq!(
            host.0.identity(),
            self.animations.identity(),
            "G services and MutatorHost must name the same animation allocation"
        );
    }
}
