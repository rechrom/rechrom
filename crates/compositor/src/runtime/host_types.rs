//! Host/input configuration for the production CPU family. These implementations
//! only select the canonical owners and native values. The HostImpl bound ties
//! source escape hatches to the very same resident owner, never a projection.
//! Assembly is atomic with the Layer/Picture/Host family because these source
//! ownership bounds are mutually recursive and registered as one CPU family.
use super::{
    frame, frame_sink, input, metadata,
    types::{CpuCompositorTypes, CpuSkiaBackend},
};
use crate::adapter::{
    input_handler::{InputHandlerTypes, InputScrollNodeId},
    input_services::InputHostTypes,
    layer_tree_host_impl::{HostHitTestRegionList, HostImplTypes},
    unbounded::UnboundedTypes,
};

impl<R: CpuSkiaBackend> InputHostTypes for CpuCompositorTypes<R> {
    type InputHandler = crate::cc::input::input_handler::InputHandler<Self>;
    type CompositorCommitData = crate::cc::trees::compositor_commit_data::CompositorCommitData;
    type MutatorHost = input::MutatorHost;
    type ScrollTree = crate::cc::trees::property_tree::ScrollTree;
    type ScrollbarOrientation = crate::cc::input::scrollbar::ScrollbarOrientation;
    type ScrollNode = InputScrollNodeId;
    type LayerTreeSettings = crate::cc::trees::layer_tree_settings::LayerTreeSettings;
    type LayerTreeHostImpl = crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>;
    type LatencyInfo = input::LatencyInfo;
    type LatencyInfoSwapPromiseMonitor = dyn input::LatencyInfoSwapPromiseMonitor;
    type EventMetricsScopedMonitor = dyn input::EventMetricsScopedMonitor;
    type EventMetricsDoneCallback = input::EventMetricsDoneCallback;
    type FrameSequenceTrackerType = input::FrameSequenceTrackerType;
    type SmoothEffectDrivingThread = input::SmoothEffectDrivingThread;
    type BeginMainFrameReason = input::BeginMainFrameReason;
    fn OtherBeginMainFrameReason() -> input::BeginMainFrameReason {
        input::BeginMainFrameReason::kOther
    }
}
impl<R: CpuSkiaBackend> InputHandlerTypes for CpuCompositorTypes<R> {
    type TraceSpan = Box<dyn input::InputTraceSpan>;
    type Transform = super::geometry::Transform<R>;
    type Elasticity = Box<crate::cc::input::scroll_elasticity_helper::ScrollElasticityHelperImpl>;
}
/// This source client's address is represented by a stable owner-scoped id.
/// It does not contain a callable Host/Page pointer and cannot own their state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnboundedClientIdentity(pub u64);
pub type CpuFrame<R> = frame::CompositorFrame<
    metadata::CompositorFrameMetadata<CpuCompositorTypes<R>>,
    frame::CompositorRenderPass<CpuCompositorTypes<R>>,
    super::resources::CpuTransferableResource,
>;
impl<R: CpuSkiaBackend> UnboundedTypes for CpuCompositorTypes<R> {
    type LocalSurfaceId = super::types::LocalSurfaceId;
    type ClientIdentity = UnboundedClientIdentity;
    type Frame = CpuFrame<R>;
    type FrameSink = frame_sink::FrameSink<Self>;
    type ReturnedResource = super::resources::CpuReturnedResource;
    type HitTestRegionList = HostHitTestRegionList<Self>;
    type BeginFrameSource = std::sync::Arc<dyn crate::adapter::scheduler_host::BeginFrameSource>;
    type RepeatingClosure = crate::adapter::task_runner::RepeatingClosure;
    type FrameTimingDetails = metadata::FrameTimingDetails;
    type Transform = super::geometry::Transform<R>;
}

impl<R: CpuSkiaBackend> HostImplTypes for CpuCompositorTypes<R> {
    fn TracePrioritizedTile(
        tile: &crate::cc::tiles::prioritized_tile::PrioritizedTile,
        trees: [Option<&crate::cc::trees::layer_tree_impl::LayerTreeImpl<Self>>; 3],
        value: &mut Self::HostTraceValue,
        trace: &dyn crate::adapter::layer_tree_host_impl::HostImplHServices<Self>,
    ) {
        crate::cc::trees::layer_tree_host_impl::TraceCpuPrioritizedTile::<R>(
            tile, trees, value, trace,
        );
    }

    fn WithPictureContextParts(
        fields: crate::adapter::host_picture::HostPictureFields<'_, Self>,
        callback: &mut dyn FnMut(
            &dyn crate::adapter::picture_layer_impl::PictureLayerReadHost<Self>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerTreeHost<Self>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplServices<Self>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<Self>,
        ),
    ) {
        super::host_picture::WithPictureContextParts::<R>(fields, callback)
    }
    fn WithLayerContextParts(
        fields: crate::adapter::host_picture::HostPictureFields<'_, Self>,
        registration: crate::adapter::host_picture::HostPictureRegistrationOwnerFields<'_, Self>,
        callback: &mut dyn FnMut(
            &dyn crate::adapter::picture_layer_impl::PictureLayerReadHost<Self>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerTreeHost<Self>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplServices<Self>,
            &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<Self>,
            &mut dyn crate::adapter::layer_tree_impl::LayerTreeScrollbarRegistrationHost<Self>,
        ),
    ) {
        super::host_picture::WithLayerContextParts::<R>(fields, registration, callback)
    }
    fn CreateResidentTileManager(
        execution: &mut Self::TileExecution,
        owner: crate::adapter::layer_tree_impl::LayerTreeHostId,
        origin: std::sync::Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>,
        worker: Self::ImageWorkerTaskRunner,
        task_limit: usize,
        running_on_renderer: bool,
        settings: crate::cc::tiles::tile_manager_settings::TileManagerSettings,
    ) -> Self::TileManager {
        execution.CreateManager(
            crate::adapter::tile::TileManagerId(owner.0),
            crate::adapter::tile_manager::TileManagerClientId(owner.0),
            origin,
            worker,
            task_limit,
            running_on_renderer,
            settings,
        )
    }
    fn ShutdownResidentTree(
        host: &mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>,
        tree: crate::adapter::layer_tree_impl::LayerTreeId,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) {
        crate::cc::trees::layer_tree_host_impl::ShutdownCpuResidentTree(host, tree, context)
    }
    fn DestroyResidentTileManager(
        host: &mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) {
        crate::cc::trees::layer_tree_host_impl::WithCpuTileManagerContext(
            host,
            context,
            |manager, runtime| manager.Destroy(runtime),
        );
    }
    fn QueueResidentImageDecode(
        manager: &mut <Self as crate::adapter::layer_tree_impl::LayerTreeTypes>::TileManager,
        image: &<Self as crate::adapter::commit_state::CommitStateTypes>::DrawImage,
        callback: crate::adapter::layer_tree_host_impl::HostImageDecodeCallback,
        speculative: bool,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) {
        let external = context.services.TileExecution();
        let mut execution = external
            .lock()
            .expect("independent native image capabilities");
        let commands = execution.commands.clone();
        manager
            .decoded_image_tracker(&mut *execution)
            .QueueImageDecode(
                image,
                Box::new(move |succeeded| {
                    commands.Push(super::execution::EngineTask::ImageDecoded {
                        callback,
                        succeeded,
                    });
                }),
                speculative,
            );
    }
    fn DisallowResidentImageCheckering(
        manager: &mut <Self as crate::adapter::layer_tree_impl::LayerTreeTypes>::TileManager,
        image: &<Self as crate::adapter::layer_tree_host_impl::HostImplCTypes>::PaintImage,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) {
        let external = context.services.TileExecution();
        let mut execution = external
            .lock()
            .expect("independent native image capabilities");
        manager
            .checker_image_tracker(&mut *execution)
            .DisallowCheckeringForImage(image);
    }
    fn ConfigureResidentTileResources(
        host: &mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) {
        crate::cc::trees::layer_tree_host_impl::ConfigureCpuTileManagerResources(host, context)
    }
    fn ResidentTileDidActivate(
        manager: &mut <Self as crate::adapter::layer_tree_impl::LayerTreeTypes>::TileManager,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) {
        let external = context.services.TileExecution();
        let mut execution = external
            .lock()
            .expect("independent native tile image capabilities");
        manager.DidActivateSyncTree(&mut *execution);
    }
    fn ClearResidentCheckerImages(
        manager: &mut <Self as crate::adapter::layer_tree_impl::LayerTreeTypes>::TileManager,
        can_clear: bool,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) {
        let external = context.services.TileExecution();
        let mut execution = external
            .lock()
            .expect("independent native tile image capabilities");
        manager.ClearCheckerImageTracking(can_clear, &mut *execution);
    }
    fn SetResidentCheckerDisabled(
        manager: &mut <Self as crate::adapter::layer_tree_impl::LayerTreeTypes>::TileManager,
        disabled: bool,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) {
        let external = context.services.TileExecution();
        let mut execution = external
            .lock()
            .expect("independent native tile image capabilities");
        manager.SetCheckerImagingForceDisabled(disabled, &mut *execution);
    }
    fn UnlockResidentTileImages(
        manager: &mut <Self as crate::adapter::layer_tree_impl::LayerTreeTypes>::TileManager,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) {
        let external = context.services.TileExecution();
        let mut execution = external
            .lock()
            .expect("independent native tile image capabilities");
        manager
            .decoded_image_tracker(&mut *execution)
            .UnlockAllImages();
    }
    fn SetResidentTileSyncFrame(
        manager: &mut <Self as crate::adapter::layer_tree_impl::LayerTreeTypes>::TileManager,
        frame: i32,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) {
        let external = context.services.TileExecution();
        let mut execution = external
            .lock()
            .expect("independent native tile image capabilities");
        manager
            .decoded_image_tracker(&mut *execution)
            .SetSyncTreeFrameNumber(frame);
    }
    fn FlushResidentTileImageTasks(
        manager: &mut <Self as crate::adapter::layer_tree_impl::LayerTreeTypes>::TileManager,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) {
        let external = context.services.TileExecution();
        let mut execution = external
            .lock()
            .expect("independent native tile image capabilities");
        manager.FlushImageControllerTasksForTesting(&mut *execution);
    }
    fn ResidentTilePrioritiesModified(
        manager: &mut <Self as crate::adapter::layer_tree_impl::LayerTreeTypes>::TileManager,
    ) {
        manager.DidModifyTilePriorities()
    }
    type TileExecution = super::tile_execution::CpuTileExecution<R>;
    fn PrepareResidentTiles(
        host: &mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) -> bool {
        crate::cc::trees::layer_tree_host_impl::WithCpuTileManagerContext(
            host,
            context,
            |manager, runtime| manager.PrepareTiles(runtime.global, runtime),
        )
    }
    fn PrepareResidentTilesToDraw(
        host: &mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) -> bool {
        crate::cc::trees::layer_tree_host_impl::WithCpuTileManagerContext(
            host,
            context,
            |manager, runtime| manager.PrepareToDraw(runtime),
        )
    }
    fn FinishResidentTileTasks(
        host: &mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>,
        context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<'_, Self>,
    ) {
        crate::cc::trees::layer_tree_host_impl::WithCpuTileManagerContext(
            host,
            context,
            |manager, runtime| manager.FinishTasksAndCleanUp(runtime),
        )
    }
    type HitTestFrameSinkId = super::types::FrameSinkId;
    type HitTestSurfaceId = super::types::SurfaceId;
    type HostTraceValue = super::host::TraceValue;
    type ActiveFrameSequenceTrackers = crate::adapter::commit_types::ActiveFrameSequenceTrackers;
    type EventMetricsList = Vec<input::EventMetrics>;
    type ImageWorkerTaskRunner =
        std::sync::Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>;
    type ImageDecodeFactory = super::host_owners::ImageDecodeFactory;
    type ResourcePool = crate::cc::resources::resource_pool::ResourcePool<Self>;
    type RasterBufferProvider = super::host_owners::RasterBufferProvider<Self>;
    type RasterQueryQueue = super::host_owners::RasterQueryQueue;
    type ContextVisibility = super::host_owners::ContextVisibility;
    type ScopedRasterContextLock = super::host_owners::RasterContextLock;
    type LayerContext = super::host_owners::LayerContext;
    type HostMutatorEvents = super::host_owners::MutatorEvents;
    type SynchronousTaskGraphRunner = std::sync::Arc<
        crate::cc::raster::synchronous_task_graph_runner::SynchronousTaskGraphRunner<
            crate::cc::tiles::tile_manager::RasterCompletion<Self>,
        >,
    >;
    type RepeatingClosure = crate::adapter::task_runner::RepeatingClosure;
    type RenderFrameMetadataObserver = super::host_owners::RenderFrameMetadataObserver<R>;
    type FrameTokenGenerator = metadata::FrameTokenGenerator;
    type ChildLocalSurfaceIdAllocator = super::host_owners::ChildLocalSurfaceIdAllocator;
    type PresentationFeedback = metadata::PresentationFeedback;
    type HostFrameTimingDetails = metadata::FrameTimingDetails;
    type SequenceChecker = super::host_owners::SequenceChecker;
    type CompositorFrameReportingController = super::host_metrics::ReportingController<Self>;
    type FrameSequenceTrackerCollection = super::host_metrics::FrameSequenceTrackers<Self>;
    type PaintWorkletLayerPainter = super::host_owners::PaintWorkletLayerPainter<R>;
    type AverageLagTrackingManager = super::host_owners::AverageLagTrackingManager;
    type EventsMetricsManager = super::host_owners::EventsMetricsManager;
    type ColorSpace = super::resources::CpuColorSpace;
    type ColorSpaceCache = super::host_owners::ColorSpaceCache;
    type ThrottleTypes = super::host_owners::ThrottleTypes<R>;
}
impl<R: CpuSkiaBackend> crate::adapter::layer_tree_host_impl::HostImplDTypes
    for CpuCompositorTypes<R>
{
    type TrackedElementFeature = metadata::TrackedElementFeature;
    type TrackedElementRect = metadata::TrackedElementRect;
    type RegionCaptureId = metadata::RegionCaptureCropId;
    type RegionCaptureBounds = metadata::RegionCaptureBounds;
    type EventMetricsSet = input::EventMetricsSet;
    type TransferableResource = super::resources::CpuTransferableResource;
    type TransitionDirective = metadata::TransitionDirective;
    type FrameDeadline = metadata::FrameDeadline;
    type DebugDrawContext<'a> = super::debug::DebugContext<'a, Self>;
    fn MakeDebugDrawContext<'a>(
        state: crate::adapter::layer_tree_host_impl::HostDebugDrawState<'a, Self>,
    ) -> Self::DebugDrawContext<'a> {
        super::debug::DebugContext::new(state)
    }
}
impl<R: CpuSkiaBackend> crate::adapter::layer_tree_host_impl::HostImplGTypes
    for CpuCompositorTypes<R>
{
    type LayerTreeMutator = super::host_owners::LayerTreeMutator;
}
impl<R: CpuSkiaBackend> metadata::NativeMetadataTypes for CpuCompositorTypes<R> {}
impl<R: CpuSkiaBackend> frame::NativeFrameTypes for CpuCompositorTypes<R> {}

/// This is a real canonical constructor call, compiled for the same family as
/// the output frame/input/layer owners; all external capabilities are injected.
pub fn CreateCpuHost<R: CpuSkiaBackend>(
    init: super::host_services::HostInit<CpuCompositorTypes<R>>,
    services: &mut super::host_services::CpuHostServices<CpuCompositorTypes<R>>,
) -> crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<CpuCompositorTypes<R>> {
    services.CreateHost(init)
}
// Production (non-test) integration assertion, checked at module registration.
const _: fn() = || {
    type Cpu = CpuCompositorTypes<super::skia::SkRegion>;
    fn require<
        B: HostImplTypes
            + InputHandlerTypes
            + crate::adapter::frame_data::FrameDataTypes
            + metadata::NativeMetadataTypes
            + frame::NativeFrameTypes,
    >() {
    }
    require::<Cpu>();
    let _: fn(
        super::host_services::HostInit<Cpu>,
        &mut super::host_services::CpuHostServices<Cpu>,
    ) -> crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Cpu> =
        CreateCpuHost::<super::skia::SkRegion>;
};

impl<R: CpuSkiaBackend> crate::adapter::layer_tree_host_impl::HostImplCTypes
    for CpuCompositorTypes<R>
{
    type PaintImage = super::paint::CpuPaintImage;
    type Tile = crate::cc::tiles::tile::Tile<Self>;
    type CustomTrackerResults = super::host_metrics::CustomTrackerResults;
    type EventLatencyData = super::host_metrics::EventLatencyData;
}

// Non-test proof of the actual source queue/iterator constructors on the same
// CPU family. These paths call canonical owners; no embedding factory can
// substitute their algorithms or return a fake queue/iterator implementation.
#[allow(dead_code)]
fn AssertCpuCanonicalQueueConstruction(
    host: &mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<
        CpuCompositorTypes<super::skia::SkRegion>,
    >,
    context: &mut crate::adapter::layer_tree_host_impl::HostImplContext<
        '_,
        CpuCompositorTypes<super::skia::SkRegion>,
    >,
) {
    let _raster = host.BuildRasterQueue(
        crate::cc::tiles::tile_priority::TreePriority::SAME_PRIORITY_FOR_BOTH_TREES,
        crate::cc::tiles::raster_tile_priority_queue::Type::ALL,
        context,
    );
    let _eviction = host.BuildEvictionQueue(context);
    let resource_tiles = host.CreateTilesWithResourceIterator();
    fn require<
        T: crate::adapter::tile_manager::ResourceTileTraversal<
            CpuCompositorTypes<super::skia::SkRegion>,
        >,
    >(
        _: &T,
    ) {
    }
    require(&resource_tiles);
}

/// Actual client-only owner factory for the default pending-tree pipeline.
pub fn CreateCpuClientHost<R: CpuSkiaBackend>(
    init: super::host_services::HostInit<CpuCompositorTypes<R>>,
    services: &mut super::host_services::CpuHostServices<CpuCompositorTypes<R>>,
) -> crate::cc::trees::client_layer_tree_host_impl::ClientLayerTreeHostImpl<CpuCompositorTypes<R>> {
    services.CreateClientHost(init)
}
// A production proof of the normal client commit chain, including its genuine
// main context and benchmark owner. No new test or simplified commit path.
#[allow(dead_code)]
fn AssertCpuClientCommit(
    host: &mut crate::cc::trees::client_layer_tree_host_impl::ClientLayerTreeHostImpl<
        crate::NativeCompositorTypes,
    >,
    state: &mut crate::cc::trees::commit_state::CommitState<crate::NativeCompositorTypes>,
    unsafe_state: &crate::cc::trees::commit_state::ThreadUnsafeCommitState<
        crate::NativeCompositorTypes,
    >,
    main: &mut super::sync::MainSynchronizationContext<'_, '_, crate::NativeCompositorTypes>,
    services: &mut super::host_services::CpuHostServices<crate::NativeCompositorTypes>,
) {
    let mut context = crate::adapter::client_layer_tree_host_impl::ClientHostContext { services };
    host.BeginCommit(state.source_frame_number, state.trace_id, &mut context);
    host.FinishCommit(state, unsafe_state, main, &mut context);
    host.CommitComplete(&mut context);
}
