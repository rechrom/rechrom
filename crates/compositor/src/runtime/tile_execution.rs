//! Independent native image/raster/diagnostic capabilities for TileManager.
//! Canonical manager/queue/pool policy is not implemented by this object.
use super::{
    execution::{EngineTask, EngineTaskQueue, ExecutorTaskGraphRunner},
    native_images::CpuNativeImageServices,
    native_services::NativeEventSink,
    raster_resources::{CpuAllocationIds, CpuSoftwareRasterBufferProvider, EngineRasterFailure},
    types::{CpuCompositorTypes, CpuSkiaBackend},
};
use crate::{
    adapter::{
        raster_task::*,
        task_runner::{RepeatingClosure, SequencedTaskRunner},
        tile::{TileManagerId, TileTraceContext, TileTraceId},
        tile_manager::*,
    },
    cc::tiles::{
        tile_manager::{
            RasterCompletion, TileManager, TileManagerCommand, TileManagerGraphTrace,
            TileManagerWorkerDiagnostics,
        },
        tile_manager_settings::TileManagerSettings,
        tile_priority::TileResolution,
    },
    engine::{CompositorExecutor, CompositorPixelProvider},
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Weak,
};

type B<R> = CpuCompositorTypes<R>;
pub struct CpuTileExecution<R: CpuSkiaBackend> {
    pub(crate) images: CpuNativeImageServices<B<R>>,
    pub(crate) pixels: Arc<dyn CompositorPixelProvider<R>>,
    pub(crate) shared_images: crate::runtime::raster_resources::CpuPoolImages<R>,
    pub(crate) allocations: Arc<CpuAllocationIds>,
    pub(crate) failure: Arc<dyn crate::runtime::paint::CpuRasterFailureHandler>,
    pub(crate) task_graph: Arc<ExecutorTaskGraphRunner<RasterCompletion<B<R>>>>,
    pub(crate) diagnostics: Arc<CpuTileDiagnostics>,
    pub(crate) origin: Arc<dyn CompositorExecutor>,
    pub(crate) commands: EngineTaskQueue,
    pub(crate) config: CpuTileExecutionConfig,
}
impl<R: CpuSkiaBackend> CpuTileExecution<R> {
    pub(crate) fn new(
        images: CpuNativeImageServices<B<R>>,
        pixels: Arc<dyn CompositorPixelProvider<R>>,
        origin: Arc<dyn CompositorExecutor>,
        raster: Arc<dyn CompositorExecutor>,
        commands: EngineTaskQueue,
        events: Arc<dyn NativeEventSink>,
    ) -> Self {
        let diagnostics = Arc::new(CpuTileDiagnostics {
            origin: origin.clone(),
            events,
        });
        let allocations = Arc::new(CpuAllocationIds::new());
        let failure: Arc<dyn crate::runtime::paint::CpuRasterFailureHandler> =
            Arc::new(EngineRasterFailure(commands.clone()));
        Self {
            shared_images: crate::runtime::raster_resources::CpuPoolImages::new(
                pixels.clone(),
                allocations.clone(),
                failure.clone(),
            ),
            images,
            config: CpuTileExecutionConfig::default(),
            pixels,
            allocations,
            failure,
            task_graph: Arc::new(ExecutorTaskGraphRunner::new(
                raster,
                commands.clone(),
                diagnostics.clone(),
            )),
            diagnostics,
            origin,
            commands,
        }
    }
    pub(crate) fn CreateRasterBufferProvider(&self) -> CpuSoftwareRasterBufferProvider<R> {
        CpuSoftwareRasterBufferProvider::new(
            self.pixels.clone(),
            self.allocations.clone(),
            self.failure.clone(),
        )
    }
    pub(crate) fn CreateManager(
        &mut self,
        identity: TileManagerId,
        client: TileManagerClientId,
        origin: Arc<dyn SequencedTaskRunner>,
        worker: Arc<dyn SequencedTaskRunner>,
        limit: usize,
        running_on_renderer: bool,
        settings: TileManagerSettings,
    ) -> TileManager<B<R>> {
        TileManager::new(
            identity,
            client,
            origin,
            worker,
            limit,
            running_on_renderer,
            settings,
            self,
        )
    }
}
impl<R: CpuSkiaBackend> TileManagerInitialization<B<R>> for CpuTileExecution<R> {
    fn CreateImageController(
        &mut self,
        origin: Arc<dyn SequencedTaskRunner>,
        _worker: Arc<dyn SequencedTaskRunner>,
        _owner: TileManagerId,
    ) -> ImageControllerId {
        self.images.CreateImageController()
    }
    fn CreateDecodedImageTracker(
        &mut self,
        controller: ImageControllerId,
        origin: Arc<dyn SequencedTaskRunner>,
    ) -> DecodedImageTrackerId {
        self.images.CreateDecodedImageTracker(controller, origin)
    }
    fn CreateCheckerImageTracker(
        &mut self,
        controller: ImageControllerId,
        _owner: TileManagerId,
        enabled: bool,
        minimum_bytes: usize,
    ) -> CheckerImageTrackerId {
        self.images
            .CreateCheckerImageTracker(controller, enabled, minimum_bytes)
    }
    fn HasCurrentDefaultTaskRunner(&self) -> bool {
        self.origin.RunsTasksInCurrentSequence()
    }
    fn RegisterDumpProvider(&mut self, owner: TileManagerId, name: &str) {
        self.diagnostics.events.Emit(
            "compositor.tile_manager",
            format!("register_dump {name} owner={}", owner.0),
        );
    }
    fn RepeatingCommand(
        &self,
        owner: TileManagerId,
        alive: Weak<()>,
        command: TileManagerCommand,
    ) -> RepeatingClosure {
        let commands = self.commands.clone();
        Arc::new(move || {
            if alive.strong_count() != 0 {
                commands.Push(EngineTask::Tile {
                    owner,
                    alive: alive.clone(),
                    command,
                });
            }
        })
    }
}

pub(crate) struct CpuTileDiagnostics {
    pub(crate) origin: Arc<dyn CompositorExecutor>,
    pub(crate) events: Arc<dyn NativeEventSink>,
}
pub(crate) struct CpuTraceScope {
    events: Arc<dyn NativeEventSink>,
    event: &'static str,
}
impl Drop for CpuTraceScope {
    fn drop(&mut self) {
        self.events
            .Emit("compositor.raster", format!("{} end", self.event));
    }
}
impl TileManagerTraceScope for CpuTraceScope {}
impl TaskGraphTraceScope for CpuTraceScope {}
impl TileTaskManagerTraceScope for CpuTraceScope {}
impl TaskGraphScheduleTrace for CpuTraceScope {
    fn AddFlowId(&mut self, id: u64) {
        self.events
            .Emit("compositor.raster", format!("schedule flow={id}"));
    }
}
impl SyncPrimitivesAllowance for CpuTraceScope {}
impl CpuTileDiagnostics {
    pub(crate) fn Scope(&self, event: &'static str) -> CpuTraceScope {
        self.events
            .Emit("compositor.raster", format!("{event} begin"));
        CpuTraceScope {
            events: self.events.clone(),
            event,
        }
    }
}
static NEXT_RASTER_TRACE_ID: AtomicU64 = AtomicU64::new(1);
impl TraceTaskIdProvider for CpuTileDiagnostics {
    fn GetNextGlobalTraceId(&self) -> u64 {
        NEXT_RASTER_TRACE_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("native raster trace id exhausted")
    }
}
impl TileManagerWorkerDiagnostics for CpuTileDiagnostics {
    fn TraceWorkerTask(&self, event: &'static str) -> Box<dyn TileManagerTraceScope> {
        Box::new(self.Scope(event))
    }
    fn CalledOnOriginThread(&self) -> bool {
        self.origin.RunsTasksInCurrentSequence()
    }
    fn TraceRasterWorker(&self, source_prepare_tiles_id: u64) -> Box<dyn TileManagerTraceScope> {
        self.events.Emit(
            "compositor.raster",
            format!("prepare_tiles={source_prepare_tiles_id}"),
        );
        Box::new(self.Scope("RasterWorker"))
    }
    fn ScopedRasterTask(
        &self,
        tile: TileTraceId,
        resolution: TileResolution,
        source_frame_number: i32,
        layer_id: i32,
    ) -> Box<dyn TileManagerTraceScope> {
        self.events.Emit(
            "compositor.raster",
            format!(
                "tile={} resolution={} frame={source_frame_number} layer={layer_id}",
                tile.0, resolution as u8
            ),
        );
        Box::new(self.Scope("RasterTask"))
    }
}
impl TileTaskManagerServices for CpuTileDiagnostics {
    fn TraceTileTaskManager(
        &self,
        event: TileTaskManagerTraceEvent,
    ) -> Box<dyn TileTaskManagerTraceScope + '_> {
        Box::new(self.Scope(match event {
            TileTaskManagerTraceEvent::ScheduleTasks => "ScheduleTasks",
            TileTaskManagerTraceEvent::CheckForCompletedTasks => "CheckForCompletedTasks",
            TileTaskManagerTraceEvent::Shutdown => "Shutdown",
        }))
    }
    fn AllowSyncPrimitivesOutsideBlockingScope(&self) -> Box<dyn SyncPrimitivesAllowance + '_> {
        Box::new(self.Scope("AllowSyncPrimitives"))
    }
}
impl TaskGraphTrace for CpuTileDiagnostics {
    fn TraceWorkQueueSchedule(&self) -> Box<dyn TaskGraphScheduleTrace + '_> {
        Box::new(self.Scope("WorkQueueSchedule"))
    }
    fn TraceSynchronousRunner(
        &self,
        event: SynchronousRunnerTraceEvent,
    ) -> Box<dyn TaskGraphTraceScope + '_> {
        let name = match event {
            SynchronousRunnerTraceEvent::ScheduleTasks {
                num_nodes,
                num_edges,
            } => {
                self.events.Emit(
                    "compositor.raster",
                    format!("graph nodes={num_nodes} edges={num_edges}"),
                );
                "RunnerScheduleTasks"
            }
            SynchronousRunnerTraceEvent::WaitForTasksToFinishRunning => "RunnerWait",
            SynchronousRunnerTraceEvent::CollectCompletedTasks => "RunnerCollect",
            SynchronousRunnerTraceEvent::RunTask => "RunnerRunTask",
        };
        Box::new(self.Scope(name))
    }
    fn HeapProfilerTaskExecution(&self) -> Box<dyn TaskGraphTraceScope + '_> {
        Box::new(self.Scope("HeapTaskExecution"))
    }
    fn TerminateSynchronousTaskFlow(&self, id: u64) {
        self.events
            .Emit("compositor.raster", format!("terminate flow={id}"));
    }
}
impl TileManagerGraphTrace for CpuTileDiagnostics {
    fn InsertNodeForTask(
        &self,
        category: u16,
        dependencies: usize,
    ) -> Box<dyn TileManagerTraceScope> {
        self.events.Emit(
            "compositor.raster",
            format!("node category={category} dependencies={dependencies}"),
        );
        Box::new(self.Scope("InsertNodeForTask"))
    }
}
impl<R: CpuSkiaBackend> TileTraceContext for CpuTileExecution<R> {
    fn MakeDictIntoImplicitSnapshotWithCategory<
        V: crate::adapter::property_types::PropertyTrace,
    >(
        &self,
        category: &str,
        _value: &mut V,
        name: &str,
        id: TileTraceId,
    ) {
        self.diagnostics.events.Emit(
            "compositor.tile",
            format!("category={category} name={name} id={}", id.0),
        );
    }
    fn TraceTileDeleted(&self, category: &str, event: &str, id: TileTraceId, flow: &str) {
        self.diagnostics.events.Emit(
            "compositor.tile",
            format!("category={category} event={event} id={} flow={flow}", id.0),
        );
    }
}

/// Explicit source feature defaults, not owner/cache/query stand-ins.
/// Source: cc/base/features.cc:56-74.
pub(crate) struct CpuTileExecutionConfig {
    pub reclaim_prepaint_when_idle: bool,
    pub reclaim_old_prepaint: bool,
    pub tile_oom_freeze_mitigation: bool,
    pub reclaim_delay_seconds: i64,
    pub trace_enabled: bool,
}
impl Default for CpuTileExecutionConfig {
    fn default() -> Self {
        Self {
            reclaim_prepaint_when_idle: false,
            reclaim_old_prepaint: false,
            tile_oom_freeze_mitigation: true,
            reclaim_delay_seconds: 30,
            trace_enabled: false,
        }
    }
}

impl<R: CpuSkiaBackend> TileManagerImages<B<R>> for CpuTileExecution<R> {
    fn ImageController(&mut self, id: ImageControllerId) -> &mut dyn ImageController<B<R>> {
        self.images.ImageController(id)
    }
    fn DecodedImageTracker(
        &mut self,
        id: DecodedImageTrackerId,
    ) -> &mut dyn DecodedImageTracker<B<R>> {
        self.images.DecodedImageTracker(id)
    }
    fn CheckerImageTracker(
        &mut self,
        id: CheckerImageTrackerId,
    ) -> &mut dyn CheckerImageTracker<B<R>> {
        self.images.CheckerImageTracker(id)
    }
}
