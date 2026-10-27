//! External live services consumed by LayerTreeHostImpl. Native owner state and
//! all cc decisions stay in translated sources; geometry/raster/Viz/platform
//! operations are supplied through typed borrowed objects and host services.
use super::{
    geometry::{Rect, Size, SizeF},
    property_tree::PropertyTreeTransform,
    region_backend::SkRegionBackend,
    resource_types::SharedImageFormat,
};
use crate::cc::base::region::Region;
// Source dependency: ui/gfx/display_color_spaces.h:36-45.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ContentColorUsage {
    #[default]
    kSRGB = 0,
    kWideColorGamut = 1,
    kHDR = 2,
}
impl ContentColorUsage {
    pub const kMaxValue: Self = Self::kHDR;
}
/// CheckedNumeric<uint32_t> retains invalid state through additional additions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckedU32(Option<u32>);
impl Default for CheckedU32 {
    fn default() -> Self {
        Self(Some(0))
    }
}
impl CheckedU32 {
    pub fn FromValue(value: u32) -> Self {
        Self(Some(value))
    }
    pub fn IsValid(self) -> bool {
        self.0.is_some()
    }
    pub fn ValueOrDie(self) -> u32 {
        self.0.expect("CheckedNumeric<uint32_t> invalid")
    }
    pub fn AddCheckedRectArea(&mut self, rect: Rect) {
        let area = rect
            .width
            .checked_mul(rect.height)
            .and_then(|a| u32::try_from(a).ok());
        self.0 = self
            .0
            .zip(area)
            .and_then(|(total, area)| total.checked_add(area));
    }
}
pub trait HostImplDebugServices {
    fn VlogIsOn(&self, level: i32) -> bool;
    fn GetClientNameForMetrics(&self) -> Option<&str>;
    fn DumpCompositorFrameSwitch(&self) -> Option<String>;
    fn WriteFileInCurrentDirectory(&self, filename: &str, data: &str) -> bool;
    fn BeginVisibilityTrace(&self, owner_id: u64);
    fn EndVisibilityTrace(&self, owner_id: u64);
}
pub trait HostImplLayer {
    type Types: super::layer_tree_impl::LayerTreeTypes<
        Transform = Self::Transform,
        RegionBackend = Self::RegionBackend,
    >;
    type Transform: PropertyTreeTransform;
    type RegionBackend: SkRegionBackend;
    fn GetInvalidationRegionForDebugging(&self) -> Region<Self::RegionBackend>;
    fn draws_content(&self) -> bool;
    fn ScreenSpaceTransform(
        &self,
        trees: &super::layer_tree_impl::PropertyTreesFor<Self::Types>,
    ) -> Self::Transform;
    fn VisibleLayerRect(
        &self,
        trees: &super::layer_tree_impl::PropertyTreesFor<Self::Types>,
    ) -> Rect;
    fn transform_tree_index(&self) -> i32;
    fn GetContentColorUsage(&self) -> ContentColorUsage;
}
/// Only immediate borrowed queries into the canonical resident LayerTreeImpl.
/// Implementations must not materialize a second layer/tree state.
pub trait HostImplTreeView {
    type Layer: HostImplLayer;
    fn LayerById(&self, id: i32) -> Option<&Self::Layer>;
    fn min_page_scale_factor(&self) -> f32;
    fn max_page_scale_factor(&self) -> f32;
    fn current_page_scale_factor(&self) -> f32;
    fn ScrollableViewportSize(&self) -> SizeF;
    fn ScrollableSize(&self) -> SizeF;
    fn viewport_mobile_optimized(&self) -> bool;
}
pub trait HostImplFrameLayerList {
    fn will_draw_layers(&self) -> &[i32];
}
pub trait HostImplFrameMetadata {
    fn set_content_color_usage(&mut self, usage: ContentColorUsage);
    fn content_color_usage(&self) -> ContentColorUsage;
}
pub trait HostImplTrackedElementRect {
    fn visible_bounds(&self) -> Rect;
    fn should_exclude_fixed_and_sticky_occlusions(&self) -> bool;
}
pub trait HostImplResourceTypes {
    type SharedMemoryMapping: Default;
    type SharedImage;
    type ResourceId: Default;
}
pub trait HostImplImageDecodeCache {
    /// Safe value identity for the source non-owning ImageDecodeCache*.
    fn ImageDecodeCacheIdentity(&self) -> crate::adapter::tile_manager::ImageDecodeCacheId;
    fn ClearCache(&mut self);
    fn SetShouldAggressivelyFreeResources(&mut self, aggressive: bool);
}
pub trait HostImplImageDecodeFactory {
    type ImageDecodeCache: HostImplImageDecodeCache;
    type RasterContextProvider;
    type DarkModeFilter: ?Sized;
    type ColorType;
    fn ToClosestSkColorType(&self, format: SharedImageFormat) -> Self::ColorType;
    fn CreateGpuImageDecodeCache(
        &self,
        provider: Self::RasterContextProvider,
        color: Self::ColorType,
        budget: usize,
        max_texture_size: i32,
        filter: Option<&Self::DarkModeFilter>,
    ) -> Box<Self::ImageDecodeCache>;
    fn CreateSoftwareImageDecodeCache(
        &self,
        color: Self::ColorType,
        budget: usize,
    ) -> Box<Self::ImageDecodeCache>;
}

/// Stable identities replace source non-owning service pointers. Resolving a
/// service always uses the current call's HostImplContext, never an owner copy.
macro_rules! service_identity {
    ($($name:ident),+ $(,)?) => {$ (
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(pub u64);
    )+};
}
service_identity!(
    HostImplDelegateId,
    HostImplSchedulingDelegateId,
    HostImplTaskRunnerProviderId,
    HostImplTaskGraphRunnerId,
    HostImplRenderingStatsId,
    HostImplDarkModeFilterId,
    HostImplFrameSinkId,
    HostImplVideoFrameControllerId,
    HostImplLatencyMonitorId,
    HostImplTaskRunnerId,
    HostImplContextProviderId
);

pub trait HostLatencyInfoSwapPromiseMonitor {
    fn identity(&self) -> HostImplLatencyMonitorId;
}

/// Pure type configuration shared by the canonical host, resident trees and
/// input state. Every associated owner here belongs to an external subsystem;
/// cc management state is held by concrete translated classes in the host.
pub trait HostImplTypes:
    super::layer_tree_impl::LayerTreeTypes<
        PrioritizedTile = crate::cc::tiles::prioritized_tile::PrioritizedTile,
        GlobalTileState: HostImplGlobalTileState,
        LayerTraceValue = Self::HostTraceValue,
        Settings = crate::cc::trees::layer_tree_settings::LayerTreeSettings,
        BeginMainFrameReason = <Self as super::input_services::InputHostTypes>::BeginMainFrameReason,
        Layer: super::render_surface_impl::RenderSurfaceLayer<Self> + HostMetadataLayer<Self> + HostImplLayer<Types=Self,Transform=<Self as super::layer_tree_impl::LayerTreeTypes>::Transform,RegionBackend=<Self as super::layer_tree_impl::LayerTreeTypes>::RegionBackend> + HostImplHitTestLayer<Self> + super::tree_sync::SyncImplLayer<Self> + HostDrawLayer<Self> + HostTileLayer<Self>,
        LayerList: super::tree_sync::SyncLayerList<Self>,
        PictureLayer: super::tree_sync::SyncPictureLayer + HostTilePictureLayer,
        RenderSurface: HostDrawSurface<Self> + HostScrollRenderSurface<Self>,
        ClientResourceProvider: HostUIResourceProvider<Self>,
        PresentationCallback=crate::adapter::presentation::Callback<Self::PresentationFeedback>,
        SuccessfulPresentationCallback=crate::adapter::presentation::SuccessfulCallbackWithDetails<Self::HostFrameTimingDetails>,
        SwapPromise: super::commit_types::SwapPromise + 'static,
        AnimatedImageDriverMap = super::image_animation::AnimatedImageDriverMap,
        PaintImageId = super::image_animation::PaintImageId,
        ImageAnimationController = crate::cc::trees::image_animation_controller::ImageAnimationController,
        MemoryHistory = crate::cc::resources::memory_history::MemoryHistory,
        DebugRectHistory = crate::cc::trees::debug_rect_history::DebugRectHistory,
        DebugState = crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState,
        PaintWorkletTracker = crate::cc::trees::animated_paint_worklet_tracker::AnimatedPaintWorkletTracker<Self>,
        LocalSurfaceId: super::unbounded::LocalSurfaceId,
        DisplayColorSpaces: HostDisplayColorSpaces<Self::ColorSpace>,
        ImageDecodeCache: HostImplImageDecodeCache,
        UIResourceBitmap = <Self as HostUIResourceTypes>::UIBitmap,
        DecodingMode = HostImageDecodingMode,
        DisplayItemList: HostDisplayItemList,
        ContentColorUsage = ContentColorUsage,
        TargetColorParams = HostTargetColorParams<Self::ColorSpace>,
    > + super::render_surface_impl::RenderSurfaceTypes
    + super::input_handler::InputHandlerTypes<
        Elasticity=Box<crate::cc::input::scroll_elasticity_helper::ScrollElasticityHelperImpl>,
        Transform=<Self as super::layer_tree_impl::LayerTreeTypes>::Transform,
        LayerTreeSettings = crate::cc::trees::layer_tree_settings::LayerTreeSettings,
        InputHandler:HostImplInputHandler,
    > + super::paint_worklet::PaintWorkletTypes
    + super::tree_sync::SyncTypes
    + super::render_metadata::RenderFrameMetadataTypes
    + super::unbounded::UnboundedTypes<LocalSurfaceId=<Self as super::layer_tree_impl::LayerTreeTypes>::LocalSurfaceId,Transform=<Self as super::layer_tree_impl::LayerTreeTypes>::Transform>
    + super::frame_data::FrameDataTypes<
        ResourceId=<Self as HostImplResourceTypes>::ResourceId,
        SurfaceId: Clone,
        RenderPass: HostDrawPass<Self>,
    >
    + super::input_services::InputHostTypes<
        MutatorHost: super::property_tree::PropertyTreeMutatorHost + Sized,
        LatencyInfoSwapPromiseMonitor: HostLatencyInfoSwapPromiseMonitor,
    >
    + HostImplDTypes
    + HostImplGTypes
    + HostUIResourceTypes
    + HostImplResourceTypes<ResourceId=super::layer_tree_impl::ResourceId, SharedImage:Clone+HostUISharedImage<<Self as HostUIResourceTypes>::UISyncToken>>
{
    /// Pure family constructor: split these actual native fields into short
    /// host/services/tiling capabilities. No source algorithm belongs here.
    fn WithPictureContextParts(fields:super::host_picture::HostPictureFields<'_,Self>,callback:&mut dyn FnMut(&dyn super::picture_layer_impl::PictureLayerReadHost<Self>,&mut dyn super::picture_layer_impl::PictureLayerTreeHost<Self>,&mut dyn super::picture_layer_impl::PictureLayerImplServices<Self>,&mut dyn super::picture_layer_impl::PictureLayerImplTilingEnvironment<Self>)) where Self:super::picture_layer_impl::PictureLayerImplTypes;
    /// Full-layer constructor also lends the independent scrollbar registration
    /// source fields; Picture-only callers never request that mutation ability.
    fn WithLayerContextParts(fields:super::host_picture::HostPictureFields<'_,Self>,registration:super::host_picture::HostPictureRegistrationOwnerFields<'_,Self>,callback:&mut dyn FnMut(
        &dyn super::picture_layer_impl::PictureLayerReadHost<Self>,
        &mut dyn super::picture_layer_impl::PictureLayerTreeHost<Self>,
        &mut dyn super::picture_layer_impl::PictureLayerImplServices<Self>,
        &mut dyn super::picture_layer_impl::PictureLayerImplTilingEnvironment<Self>,
        &mut dyn super::layer_tree_impl::LayerTreeScrollbarRegistrationHost<Self>,
    )) where Self:super::picture_layer_impl::PictureLayerImplTypes;
    type HitTestFrameSinkId:Default;
    type HitTestSurfaceId:HostHitTestSurfaceId<FrameSinkId=Self::HitTestFrameSinkId>;
    type HostTraceValue:HostTraceValue + super::layer_impl::LayerImplTraceValue;
    type ActiveFrameSequenceTrackers;
    type EventMetricsList;
    /// Independent image/task/pixel capabilities; no cc owner lives here.
    type TileExecution;
    fn CreateResidentTileManager(
        execution:&mut Self::TileExecution,owner:super::layer_tree_impl::LayerTreeHostId,
        origin:std::sync::Arc<dyn super::task_runner::SequencedTaskRunner>,worker:Self::ImageWorkerTaskRunner,
        task_limit:usize,running_on_renderer:bool,
        settings:crate::cc::tiles::tile_manager_settings::TileManagerSettings,
    )->Self::TileManager;
    fn QueueResidentImageDecode(manager:&mut Self::TileManager,image:&Self::DrawImage,callback:HostImageDecodeCallback,speculative:bool,context:&mut HostImplContext<'_,Self>);
    fn DisallowResidentImageCheckering(manager:&mut Self::TileManager,image:&Self::PaintImage,context:&mut HostImplContext<'_,Self>);
    fn ConfigureResidentTileResources(host:&mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>,context:&mut HostImplContext<'_,Self>);
    fn ResidentTileDidActivate(manager:&mut Self::TileManager,context:&mut HostImplContext<'_,Self>);
    fn ClearResidentCheckerImages(manager:&mut Self::TileManager,can_clear:bool,context:&mut HostImplContext<'_,Self>);
    fn SetResidentCheckerDisabled(manager:&mut Self::TileManager,disabled:bool,context:&mut HostImplContext<'_,Self>);
    fn UnlockResidentTileImages(manager:&mut Self::TileManager,context:&mut HostImplContext<'_,Self>);
    fn SetResidentTileSyncFrame(manager:&mut Self::TileManager,frame:i32,context:&mut HostImplContext<'_,Self>);
    fn ResidentTilePrioritiesModified(manager:&mut Self::TileManager);
    fn FlushResidentTileImageTasks(manager:&mut Self::TileManager,context:&mut HostImplContext<'_,Self>);
    fn PrepareResidentTiles(host:&mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>,context:&mut HostImplContext<'_,Self>)->bool;
    fn PrepareResidentTilesToDraw(host:&mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>,context:&mut HostImplContext<'_,Self>)->bool;
    fn FinishResidentTileTasks(host:&mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>,context:&mut HostImplContext<'_,Self>);
    /// Source member destruction needs the same live twin/manager siblings.
    fn ShutdownResidentTree(host:&mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>,tree:super::layer_tree_impl::LayerTreeId,context:&mut HostImplContext<'_,Self>);
    fn DestroyResidentTileManager(host:&mut crate::cc::trees::layer_tree_host_impl::LayerTreeHostImpl<Self>,context:&mut HostImplContext<'_,Self>);
    type ImageWorkerTaskRunner;
    type ImageDecodeFactory: HostImplImageDecodeFactory<ImageDecodeCache=<Self as super::layer_tree_impl::LayerTreeTypes>::ImageDecodeCache>;
    /// Same-package lifetime/identity binding only; the source trace algorithm
    /// stays in canonical PrioritizedTile and borrows its actual resident owner.
    fn TracePrioritizedTile(
        tile:&crate::cc::tiles::prioritized_tile::PrioritizedTile,
        trees:[Option<&crate::cc::trees::layer_tree_impl::LayerTreeImpl<Self>>;3],
        value:&mut Self::HostTraceValue,
        trace:&dyn HostImplHServices<Self>,
    );
    type ResourcePool;
    type RasterBufferProvider;
    type RasterQueryQueue;
    type ContextVisibility;
    type ScopedRasterContextLock:HostRasterContextLock;
    type LayerContext:HostUnboundedLayerContext<Self>;
    type HostMutatorEvents;
    type SynchronousTaskGraphRunner;
    type RepeatingClosure;
    type RenderFrameMetadataObserver;
    type FrameTokenGenerator: Default;
    type ChildLocalSurfaceIdAllocator: HostChildLocalSurfaceIdAllocator;
    type PresentationFeedback;
    type HostFrameTimingDetails:HostFrameTimingDetails + Clone;
    type SequenceChecker: super::presentation::SequenceChecker;
    type CompositorFrameReportingController;
    type FrameSequenceTrackerCollection;
    type PaintWorkletLayerPainter;
    type AverageLagTrackingManager;
    type EventsMetricsManager;
    type ColorSpace: HostColorSpace;
    type ColorSpaceCache: HostColorSpaceCache<Self::ColorSpace>;
    type ThrottleTypes: super::throttle::ThrottleTypes<
        Transform=<Self as super::layer_tree_impl::LayerTreeTypes>::Transform,
        PassId=Self::RenderPassId,
        PassHandle=HostRenderPassLocation,
        Pass=<Self as super::frame_data::FrameDataTypes>::RenderPass,
    >;
}

/// Only external service borrows are retained during a host call. Native tree,
/// input, animation and commit state remains resident inside LayerTreeHostImpl.
pub struct HostImplContext<'a, B: HostImplTypes> {
    pub services: &'a mut dyn HostImplServices<B>,
}

/// The initial facet is deliberately platform-only. Source management methods
/// are translated on LayerTreeHostImpl and extend this facet with synchronous
/// external operations as their concrete call sites are translated.
pub trait HostImplServices<B: HostImplTypes>:
    HostImplDebugServices
    + HostUIResourceServices<B>
    + HostImplHServices<B>
    + HostImplLayerServices<B>
    + HostImplJServices<B>
    + HostImplEServices<B>
    + HostImplFServices<B>
    + HostImplAServices<B>
    + HostImplInputEnvironmentServices<B>
    + HostImplScrollbarRegistrationServices<B>
    + HostImplBrowserControlsServices
    + HostImplBServices<B>
    + HostImplCServices<B>
    + HostImplGServices<B>
    + HostImplTreeHostServices<B>
    + crate::adapter::layer_tree_impl::draw_host::LayerTreeDrawServices
    + HostImplDServices<B>
    + super::render_surface_impl::RenderSurfaceServices<B>
{
    /// Source AnimationHost callbacks synchronously mutate the current Host.
    /// Lend the one resident external H facet separately from the remaining
    /// services, so a live mutation target never aliases that facet's owner.
    fn WithClientFacet(
        &mut self,
        callback: &mut dyn FnMut(
            &mut dyn super::client_layer_tree_host_impl::ClientHostCapabilities<B>,
            &mut dyn HostImplServices<B>,
        ),
    );
    fn WithAnimationFacet(
        &mut self,
        callback: &mut dyn FnMut(&mut dyn HostImplHServices<B>, &mut dyn HostImplServices<B>),
    );
    fn TileExecution(&self) -> std::sync::Arc<std::sync::Mutex<B::TileExecution>>;
    fn PictureReadEnvironment(&self) -> &dyn super::host_picture::HostPictureReadEnvironment;

    /// Pure split of independent external capability fields during this call.
    /// Source host/tree/manager state is not lent to or retained by any facet.
    fn WithPictureExternalParts(
        &mut self,
        callback: &mut dyn FnMut(
            &dyn super::host_picture::HostPictureReadEnvironment,
            &mut dyn super::picture_layer_impl::PictureLayerImplServices<B>,
            &mut dyn super::host_picture::HostPictureTileServices,
            &mut dyn super::host_picture::HostPictureMutationServices,
            &mut dyn super::host_picture::HostPictureRegistrationServices<B>,
            &mut dyn super::host_picture::HostPictureReleaseServices<B>,
        ),
    ) where
        B: super::picture_layer_impl::PictureLayerImplTypes;

    // Exact external native constructors at source HostImpl initializer points.
    fn CreateChildLocalSurfaceIdAllocator(&mut self) -> B::ChildLocalSurfaceIdAllocator;
    fn CreateAverageLagTrackingManager(&mut self) -> B::AverageLagTrackingManager;
    fn CreateEventsMetricsManager(&mut self) -> B::EventsMetricsManager;
    fn CreateTileManager(
        &mut self,
        owner: super::layer_tree_impl::LayerTreeHostId,
        runner: HostImplTaskRunnerId,
        image_worker: B::ImageWorkerTaskRunner,
        task_limit: usize,
        running_on_renderer: bool,
        settings: crate::cc::tiles::tile_manager_settings::TileManagerSettings,
    ) -> B::TileManager;
    fn DefaultGlobalTileState(&self) -> B::GlobalTileState;
    fn CreateFrameSorter(&mut self) -> B::FrameSorter;
    fn CreateFrameTrackers(&mut self, single_threaded: bool) -> B::FrameSequenceTrackerCollection;
    fn ResolveSequencedTaskRunner(
        &self,
        runner: HostImplTaskRunnerId,
    ) -> std::sync::Arc<dyn super::task_runner::SequencedTaskRunner>;
    fn RouteHostRepeatingClosure(&self, task: HostImplTask)
        -> super::task_runner::RepeatingClosure;
    fn CreateColorSpaceCache(&mut self, capacity: usize) -> B::ColorSpaceCache;
    fn NewSequenceChecker(&self) -> B::SequenceChecker;
    fn InvalidFrameSinkId(
        &self,
    ) -> <B::ThrottleTypes as super::throttle::ThrottleTypes>::FrameSinkId;
    fn CreateImageAnimationTaskRunner(
        &mut self,
        runner: HostImplTaskRunnerId,
    ) -> Box<dyn super::image_animation::ImageAnimationTaskRunner>;
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
        mutator: &mut <B as super::layer_tree_impl::LayerTreeTypes>::MutatorHost,
        owner: Option<super::layer_tree_impl::LayerTreeHostId>,
    );
    fn MutatorCreateEvents(
        &mut self,
        mutator: &mut <B as super::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> Box<B::HostMutatorEvents>;
    fn AllocateLayerTreeIdentity(
        &mut self,
    ) -> (
        super::layer_tree_impl::LayerTreeId,
        super::property_tree::PropertyTreesOwnerId,
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
        owner: Option<super::layer_tree_impl::LayerTreeHostId>,
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
        mutator: &mut <B as super::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    );
    fn ReportingClearFrameSequenceTrackerCollection(
        &mut self,
        controller: &mut B::CompositorFrameReportingController,
    );
    fn MonotonicTimeNow(&self) -> super::time::TimeTicks;
    fn BrowserControlsFeatureEnabled(
        &self,
        feature: super::browser_controls::BrowserControlsFeature,
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
        reason: <B as super::input_services::InputHostTypes>::BeginMainFrameReason,
    );
    fn DelegateRenewTreePriority(&mut self, delegate: HostImplDelegateId);
    fn TraceCanDrawReason(&self, reason: &'static str);
    fn ChildLocalSurfaceIdAllocatorCurrent<'a>(
        &self,
        allocator: &'a B::ChildLocalSurfaceIdAllocator,
    ) -> &'a <B as super::layer_tree_impl::LayerTreeTypes>::LocalSurfaceId;
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
        mutator: &<B as super::layer_tree_impl::LayerTreeTypes>::MutatorHost,
        scroller: super::input_types::ElementId,
    ) -> bool;
    fn SetDidScrollForMetrics(&mut self, manager: &mut B::EventsMetricsManager, did_scroll: bool);
    fn DelegateSetNeedsOneBeginImplFrameOnImplThread(&mut self, delegate: HostImplDelegateId);
    fn DelegateIsInsideDraw(&self, delegate: HostImplDelegateId) -> bool;
    fn DelegateSetNeedsRedrawOnImplThread(&mut self, delegate: HostImplDelegateId);
    fn ImageAnimationTraceEnabled(&self, event: &'static str) -> bool;
    fn ImageAnimationTrace(
        &mut self,
        event: &'static str,
        value: super::image_animation::ImageAnimationTraceValue<'_>,
    );
    fn IsImplThread(&self, provider: HostImplTaskRunnerProviderId) -> bool;
    fn SettingsFeatures(&self) -> &dyn super::settings::LayerTreeSettingsFeatures;
}

/// Typed weak routing token; callbacks cannot own or recover a retained host.
#[derive(Clone, Debug)]
pub struct HostImplWeak {
    pub owner: super::layer_tree_impl::LayerTreeHostId,
    pub alive: std::sync::Weak<()>,
}
impl HostImplWeak {
    pub fn IsValid(&self) -> bool {
        self.alive.upgrade().is_some()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostImplTaskKind {
    ResetHasInputForFrameInterval,
    MaybeFlushPendingWork,
}
#[derive(Clone, Debug)]
pub struct HostImplTask {
    pub target: HostImplWeak,
    pub kind: HostImplTaskKind,
}

/// Immediate fields of the resident external tiles value; no copied policy.
pub trait HostImplGlobalTileState {
    fn set_tree_priority(&mut self, priority: crate::cc::tiles::tile_priority::TreePriority);
    fn set_viewport_size(&mut self, size: Size);
    fn hard_memory_limit_in_bytes(&self) -> usize;
    fn soft_memory_limit_in_bytes(&self) -> usize;
    fn num_resources_limit(&self) -> usize;
    fn set_hard_memory_limit_in_bytes(&mut self, value: usize);
    fn set_soft_memory_limit_in_bytes(&mut self, value: usize);
    fn set_num_resources_limit(&mut self, value: usize);
    fn set_memory_limit_policy(&mut self, value: super::resource_types::TileMemoryLimitPolicy);
}

// HostImpl chunk I adapter proposal. Merge these type/capability declarations
// into adapter/layer_tree_host_impl.rs; add HostUIResourceTypes as a supertrait
// of HostImplTypes. No UI-resource decision or management algorithm lives here.

use super::layer_tree_impl::LayerTreeHostId;
use std::sync::{Arc, Weak};

// Source dependencies: cc/resources/ui_resource_bitmap.h; native gfx/Skia/GPU
// owners remain external to cc. The enum is the source UIResourceBitmap format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostUIBitmapFormat {
    RGBA8,
    ETC1,
}
pub trait HostUIResourceBitmap {
    fn GetFormat(&self) -> HostUIBitmapFormat;
    fn GetSize(&self) -> Size;
    fn GetOpaque(&self) -> bool;
    fn GetPixels(&self) -> &[u8];
    fn row_bytes(&self) -> usize;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostUISharedImageUsage(u32);
// Source dependency: gpu/command_buffer/common/shared_image_usage.h:25,29,50.
impl HostUISharedImageUsage {
    pub const DISPLAY_READ: Self = Self(1 << 2);
    pub const CPU_WRITE_ONLY: Self = Self(1 << 11);
    pub const SCANOUT: Self = Self(1 << 4);
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
impl std::ops::BitOrAssign for HostUISharedImageUsage {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
pub trait HostUISharedImage<T> {
    fn creation_sync_token(&self) -> T;
    fn UpdateDestructionSyncToken(&mut self, token: &T);
}
pub trait HostUISharedMapping {
    fn GetMemoryForPlane(&mut self, plane: usize) -> &mut [u8];
}
pub trait HostUIResourceImageInfo {
    fn minRowBytes(&self) -> usize;
}
pub trait HostUISourceBitmap<'a, Info, Image> {
    fn setInfo(&mut self, info: Info, row_bytes: usize);
    fn setPixels(&mut self, pixels: &'a [u8]);
    fn asImage(&self) -> Image;
}
pub trait HostUICanvas<Info, Image> {
    // Source dependency: third_party/skia/include/core/SkCanvas.h:384-396.
    fn peekPixels(&self) -> HostUIPixmap<'_>;
    fn writePixels(&mut self, info: &Info, pixels: &[u8], row_bytes: usize, x: i32, y: i32);
    fn scale(&mut self, x: f32, y: f32);
    fn clearTransparent(&mut self);
    fn drawImage(&mut self, image: &Image, x: f32, y: f32);
}
/// Borrowed native SkPixmap-to-span result. The surface owns its pixels.
pub struct HostUIPixmap<'a> {
    pub pixels: &'a [u8],
}
pub trait HostUISurface<Info, Image> {
    type Canvas: HostUICanvas<Info, Image>;
    fn getCanvas(&mut self) -> &mut Self::Canvas;
    fn peekPixels(&self) -> HostUIPixmap<'_>;
}
pub trait HostUIResourceTypes: HostImplResourceTypes {
    type UIBitmap: HostUIResourceBitmap;
    type UISyncToken: Default;
    type UIColorSpace;
    type UIImageInfo: HostUIResourceImageInfo;
    type UIImage;
    type UISourceBitmap<'a>: HostUISourceBitmap<'a, Self::UIImageInfo, Self::UIImage>;
    type UIScopedMapping: HostUISharedMapping;
    type UISurface<'a>: HostUISurface<Self::UIImageInfo, Self::UIImage>;
    type UITransferableResource;
}
/// Scope-owned one-shot equivalent of BindOnce(OnUIResourceReleased, AsWeakPtr,
/// uid). It stores only a stable identity and lifetime token, never a host ref.
pub struct HostUIResourceReleaseTask {
    owner: LayerTreeHostId,
    uid: super::layer_tree_impl::UIResourceId,
    alive: Weak<()>,
}
impl HostUIResourceReleaseTask {
    pub fn new(
        owner: LayerTreeHostId,
        uid: super::layer_tree_impl::UIResourceId,
        alive: &Arc<()>,
    ) -> Self {
        Self {
            owner,
            uid,
            alive: Arc::downgrade(alive),
        }
    }
    pub fn Run<B: HostUIResourceTypes>(
        self,
        sync_token: &B::UISyncToken,
        lost: bool,
        client: &mut dyn HostUIResourceReleaseClient<B>,
    ) {
        let Some(_alive) = self.alive.upgrade() else {
            return;
        };
        assert_eq!(
            self.owner,
            client.OwnerId(),
            "resource callback routed to another owner"
        );
        client.OnUIResourceReleased(self.uid, sync_token, lost);
    }
}
pub trait HostUIResourceReleaseClient<B: HostUIResourceTypes> {
    fn OwnerId(&self) -> LayerTreeHostId;
    fn OnUIResourceReleased(
        &mut self,
        uid: super::layer_tree_impl::UIResourceId,
        sync_token: &B::UISyncToken,
        lost: bool,
    );
}
pub trait HostUIResourceProvider<B: HostUIResourceTypes> {
    fn ImportResource(
        &mut self,
        resource: B::UITransferableResource,
        release: HostUIResourceReleaseTask,
    ) -> <B as HostImplResourceTypes>::ResourceId;
    /// May synchronously invoke stored once callbacks with this live client.
    /// The caller has already moved the UI data into the deleted map.
    fn RemoveImportedResource(
        &mut self,
        id: <B as HostImplResourceTypes>::ResourceId,
        client: &mut dyn HostUIResourceReleaseClient<B>,
    );
}

// Add these signatures to HostImplServices<B>. HostImplTypes must also imply
// B::SharedImage: Clone + HostUISharedImage<B::UISyncToken>, <B as HostImplResourceTypes>::ResourceId:
// Copy+Eq, and B::ClientResourceProvider: HostUIResourceProvider<B>.
// Every method is a direct native GPU/Skia/platform operation, not a cc branch.
pub trait HostUIResourceServices<B: HostImplTypes + HostUIResourceTypes> {
    fn TraceUIResourceEvent(&mut self, name: &'static str);
    fn UIResourceContextProviderExists(&self, sink: HostImplFrameSinkId) -> bool;
    fn UIResourceSharedImageInterfaceExists(&self, sink: HostImplFrameSinkId) -> bool;
    fn UIResourceSupportsScanoutSharedImages(&self, sink: HostImplFrameSinkId) -> bool;
    fn CanCreateNativeBufferForFormat(&self, format: SharedImageFormat) -> bool;
    fn ScaleToCeiledSize(&self, size: Size, scale: f32) -> Size;
    fn SRGBColorSpace(&self) -> B::UIColorSpace;
    fn CreateUIResourceSharedImageForSoftwareCompositor(
        &mut self,
        sink: HostImplFrameSinkId,
        format: SharedImageFormat,
        size: Size,
        color_space: &B::UIColorSpace,
        usage: HostUISharedImageUsage,
        debug_label: &'static str,
    ) -> Option<B::SharedImage>;
    fn MapUIResourceSharedImage(&mut self, image: &B::SharedImage) -> B::UIScopedMapping;
    fn CreateUIResourceSharedImage(
        &mut self,
        sink: HostImplFrameSinkId,
        format: SharedImageFormat,
        size: Size,
        color_space: &B::UIColorSpace,
        usage: HostUISharedImageUsage,
        debug_label: &'static str,
        pixels: &[u8],
    ) -> Option<B::SharedImage>;
    fn MakeN32Premul(&self, size: Size) -> B::UIImageInfo;
    fn NewUIResourceSourceBitmap<'a>(&mut self) -> B::UISourceBitmap<'a>;
    fn WrapUIResourcePixels<'a>(
        &mut self,
        info: B::UIImageInfo,
        pixels: &'a mut [u8],
        row_bytes: usize,
    ) -> Option<B::UISurface<'a>>;
    fn RasterUIResourceSurface<'a>(&mut self, info: B::UIImageInfo) -> Option<B::UISurface<'a>>;
    fn MakeUITransferableResource(
        &mut self,
        image: B::SharedImage,
        sync_token: B::UISyncToken,
    ) -> B::UITransferableResource;
    fn OnCanDrawStateChanged(&mut self, delegate: HostImplDelegateId, can_draw: bool);
    fn NotifyLatencyInfoSwapPromiseMonitor(
        &mut self,
        monitor: HostImplLatencyMonitorId,
    ) -> Option<Box<B::SwapPromise>>;
    fn OptionalImplThreadTaskRunner(
        &self,
        provider: HostImplTaskRunnerProviderId,
    ) -> Option<HostImplTaskRunnerId>;
    fn RunsTasksInCurrentSequence(&self, runner: HostImplTaskRunnerId) -> bool;
}

/// Extra concrete-InputHandler primitive corresponding to the source static_cast.
pub trait HostImplInputHandler {
    fn set_external_pinch_gesture_active(&mut self, active: bool);
    fn pinch_gesture_active(&self) -> bool;
    fn set_force_smooth_wheel_scrolling_for_testing(&mut self, enabled: bool);
}
impl<B: super::input_handler::InputHandlerTypes> HostImplInputHandler
    for crate::cc::input::input_handler::InputHandler<B>
{
    fn set_external_pinch_gesture_active(&mut self, active: bool) {
        crate::cc::input::input_handler::InputHandler::set_external_pinch_gesture_active(
            self, active,
        );
    }

    fn pinch_gesture_active(&self) -> bool {
        crate::cc::input::input_handler::InputHandler::pinch_gesture_active(self)
    }

    fn set_force_smooth_wheel_scrolling_for_testing(&mut self, enabled: bool) {
        crate::cc::input::input_handler::InputHandler::set_force_smooth_wheel_scrolling_for_testing(
            self, enabled,
        );
    }
}

// Compatibility name for callers that predate the canonical source module.
// The value itself is the translated cc/layers/draw_mode.h type.
pub use crate::cc::layers::draw_mode::DrawMode as HostDrawMode;
/// perfetto NamedTrack::FromPointer uses the exact resident identity; no tree copy.
pub struct HostImplNamedTrack {
    pub name: &'static str,
    pub tree: Option<super::layer_tree_impl::LayerTreeId>,
}

// Chunk H adapter additions. Merge the indicated associated type/bounds/methods
// into existing HostImplTypes/HostImplServices; this file owns no cc algorithm.
// HostImplTypes additionally extends adapter::frame_data::FrameDataTypes.
// LayerTreeTypes::SwapPromise additionally: adapter::commit_types::SwapPromise + 'static.
// HostImplTypes adds: type HostTraceValue: HostTraceValue;

// Source dependency: cc/trees/mutator_host.h:29-38.
#[derive(Clone, Copy, Debug, Default)]
pub struct HostAnimationTickResult {
    pub animated: bool,
    pub needs_next_frame: bool,
}
// Source dependency: cc/metrics/frame_sequence_metrics.h:29-49.
// Named projection into external cc/metrics/FrameSequenceTrackerType. Not a
// substitute tracker or a second metric state. Backend maps these names exactly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum HostAnimationSequence {
    kCompositorAnimation = 0,
    kCompositorRasterAnimation = 14,
    kCompositorNativeAnimation = 15,
    kSETCompositorAnimation = 13,
}
pub trait HostTraceValue: crate::adapter::property_types::PropertyTrace {
    fn ToFormattedJSON(&self) -> String;
}

// Merge these method declarations into HostImplServices<B>.
pub trait HostImplHServices<B: HostImplTypes> {
    fn MutatorHasImplOnlyScrollAnimatingElement(
        &self,
        host: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> bool;
    fn MutatorHasImplOnlyAutoScrollAnimatingElement(
        &self,
        host: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> bool;
    // Source: cc/trees/layer_tree_host_impl.cc:5315-5325.
    // The actual animation owner is already held by this facet. Pass only the
    // invoking Host identity; callbacks borrow that same Host synchronously.
    fn MutatorTickAnimations(
        &mut self,
        owner: crate::adapter::layer_tree_impl::LayerTreeHostId,
        time: crate::adapter::time::TimeTicks,
        is_active: bool,
        events: Option<&mut B::HostMutatorEvents>,
        target: &mut dyn crate::adapter::animation_host::AnimationMutationTarget,
    ) -> HostAnimationTickResult;
    fn MutatorHasInvalidationAnimation(
        &self,
        host: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> bool;
    fn MutatorHasNativePropertyAnimation(
        &self,
        host: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> bool;
    fn MutatorHasViewTransition(
        &self,
        host: &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
    ) -> bool;
    fn MutatorUpdateAnimationState(
        &mut self,
        owner: crate::adapter::layer_tree_impl::LayerTreeHostId,
        start_ready: bool,
        events: Option<&mut B::HostMutatorEvents>,
        target: &mut dyn crate::adapter::animation_host::AnimationMutationTarget,
    ) -> bool;
    fn MutatorActivateAnimations(
        &mut self,
        owner: crate::adapter::layer_tree_impl::LayerTreeHostId,
        events: Option<&mut B::HostMutatorEvents>,
        target: &mut dyn crate::adapter::animation_host::AnimationMutationTarget,
    ) -> bool;
    fn MutatorEventsIsEmpty(&self, events: &B::HostMutatorEvents) -> bool;
    fn TrackersStartSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        kind: HostAnimationSequence,
    );
    fn TrackersStopSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        kind: HostAnimationSequence,
    );
    fn DidCompletePageScaleAnimationOnImplThread(&mut self, delegate: HostImplDelegateId);
    fn VideoFrameControllerOnBeginFrame(
        &mut self,
        controller: HostImplVideoFrameControllerId,
        args: &crate::adapter::begin_frame::BeginFrameArgs,
    );
    fn SetVideoNeedsBeginFrames(&mut self, delegate: HostImplDelegateId, needs: bool);
    fn GlobalTileStateTreePriority(
        &self,
        global: &B::GlobalTileState,
    ) -> crate::adapter::scheduler::TreePriority;
    fn NewHostTraceValue(&self) -> Box<B::HostTraceValue>;
    fn NewHostTraceValueJSON(&self) -> B::HostTraceValue;
    fn HostTraceCategoryEnabled(&self, category: &str) -> bool;
    fn TraceTileDeletion(
        &self,
        category: &str,
        event: &str,
        id: crate::adapter::tile::TileTraceId,
        flow: &str,
    );
    fn TraceTileSnapshot(
        &self,
        category: &str,
        value: &mut dyn crate::adapter::property_types::PropertyTrace,
        name: &str,
        id: crate::adapter::tile::TileTraceId,
    );
    fn TraceRasterSourceIdRef(
        &self,
        id: crate::adapter::prioritized_tile::RasterSourceTraceId,
        value: &mut dyn crate::adapter::property_types::PropertyTrace,
        name: &str,
    );
    fn TileManagerBasicStateAsValueInto(
        &self,
        manager: &B::TileManager,
        value: &mut B::HostTraceValue,
    );
    // viz::TracedValue::SetIDRef/Id external trace-format operation only.
    fn TraceSetHostIdRef(
        &self,
        id: crate::adapter::layer_tree_impl::LayerTreeHostId,
        value: &mut B::HostTraceValue,
        name: &str,
    );
}

// Trace interface name projections only: no tree copy, geometry/state decisions.
pub struct HostFrameTraceValue<'a, V: HostTraceValue>(pub &'a mut V);
impl<V: HostTraceValue> crate::adapter::frame_data::FrameDataTraceValue
    for HostFrameTraceValue<'_, V>
{
    fn SetBoolean(&mut self, n: &str, v: bool) {
        self.0.SetBoolean(n, v);
    }
    fn BeginArray(&mut self, n: &str) {
        self.0.BeginArray(n);
    }
    fn BeginDictionary(&mut self) {
        self.0.BeginDictionaryUnnamed();
    }
    fn EndDictionary(&mut self) {
        self.0.EndDictionary();
    }
    fn EndArray(&mut self) {
        self.0.EndArray();
    }
}
pub struct HostFrameTraceJson<V: HostTraceValue>(pub V);
impl<V: HostTraceValue> crate::adapter::frame_data::FrameDataTraceValue for HostFrameTraceJson<V> {
    fn SetBoolean(&mut self, n: &str, v: bool) {
        self.0.SetBoolean(n, v);
    }
    fn BeginArray(&mut self, n: &str) {
        self.0.BeginArray(n);
    }
    fn BeginDictionary(&mut self) {
        self.0.BeginDictionaryUnnamed();
    }
    fn EndDictionary(&mut self) {
        self.0.EndDictionary();
    }
    fn EndArray(&mut self) {
        self.0.EndArray();
    }
}
impl<V: HostTraceValue> crate::adapter::frame_data::FrameDataTraceJson for HostFrameTraceJson<V> {
    fn ToFormattedJSON(&self) -> String {
        self.0.ToFormattedJSON()
    }
}
pub struct HostFrameTraceServices<'a, B: HostImplTypes>(pub &'a dyn HostImplServices<B>);
impl<B: HostImplTypes> crate::adapter::frame_data::FrameDataTraceServices
    for HostFrameTraceServices<'_, B>
{
    type TracedValueJSON = HostFrameTraceJson<B::HostTraceValue>;
    fn GetClientNameForMetrics(&self) -> Option<&str> {
        self.0.GetClientNameForMetrics()
    }
    fn TraceCategoryEnabled(&self, category: &str) -> bool {
        self.0.HostTraceCategoryEnabled(category)
    }
    fn NewTracedValueJSON(&self) -> Self::TracedValueJSON {
        HostFrameTraceJson(self.0.NewHostTraceValueJSON())
    }
}

use super::{geometry::PointF, input_handler::InputLayerId, input_types::ElementId};
use crate::cc::input::scrollbar::{ScrollbarOrientation, ScrollbarPart};
/// Synchronous calls into the external LayerImpl/ScrollbarLayerImplBase owner.
/// The caller validates a resident ID and lends the actual canonical tree for
/// this one call. Implementations forward to real layer methods; they retain
/// no tree borrow and contain no compositor/input policy or copied layer data.
pub trait HostImplLayerServices<B: HostImplTypes> {
    fn Layer_Scrollbar_orientation(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> ScrollbarOrientation;
    fn Layer_Scrollbar_scroll_element_id(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> ElementId;
    fn Layer_Scrollbar_OverlayScrollbarOpacity(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32;
    fn Layer_Scrollbar_UsesAuraOverlayAnimator(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool;
    fn Layer_Scrollbar_has_find_in_page_tickmarks(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool;
    fn Layer_Scrollbar_JumpOnTrackClick(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool;
    fn Layer_Scrollbar_current_pos(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32;
    fn Layer_Scrollbar_scroll_layer_length(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32;
    fn Layer_Scrollbar_ComputeThumbQuadRect(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect;
    fn Layer_Scrollbar_SupportsDragSnapBack(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool;
    fn Layer_Scrollbar_ForwardTrackRect(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect;
    fn Layer_Scrollbar_BackTrackRect(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect;
    fn Layer_Scrollbar_BackButtonRect(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect;
    fn Layer_Scrollbar_ForwardButtonRect(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect;
    fn Layer_Scrollbar_ThumbThickness(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> i32;
    fn Layer_Scrollbar_ThumbLength(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> i32;
    fn Layer_Scrollbar_TrackLength(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32;
    fn Layer_Scrollbar_ScreenSpaceTransform(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> <B as super::layer_tree_impl::LayerTreeTypes>::Transform;
    fn Layer_Scrollbar_IdentifyScrollbarPart(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
        position: PointF,
    ) -> ScrollbarPart;
    fn Layer_Scrollbar_is_overlay_scrollbar(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool;
    fn Layer_Scrollbar_bounds(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> super::geometry::Size;
    fn Layer_Scrollbar_device_scale_factor(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32;
    fn Layer_Scrollbar_ComputeHitTestableExpandedThumbQuadRect(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect;
    fn Layer_Scrollbar_thumb_thickness_scale_factor(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32;
    fn Layer_Scrollbar_SetThumbThicknessScaleFactor(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
        scale: f32,
    );
    fn Layer_Scrollbar_CanScrollOrientation(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool;
    fn Layer_Scrollbar_SetOverlayScrollbarLayerOpacityAnimated(
        &mut self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        layer: &InputLayerId,
        opacity: f32,
        fade_out_animation: bool,
    );
}

use super::layer_tree_impl::{ElementListType, LayerTreeTypes};
use super::time::TimeTicks;
// Source dependency: cc/trees/mutator_host_delegate.h:27-32.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AnimationWorkletMutationState {
    STARTED = 0,
    COMPLETED_WITH_UPDATE = 1,
    COMPLETED_NO_UPDATE = 2,
    CANCELED = 3,
}
// Source: cc/trees/layer_tree_host_impl.cc:173-179.

// Source dependency: components/viz/common/surfaces/child_local_surface_id_allocator.h:39.
pub trait HostChildLocalSurfaceIdAllocator {
    fn GenerateId(&mut self);
}
// Source dependency: components/viz/common/frame_timing_details.h:30.
pub trait HostFrameTimingDetails {
    fn PresentationTimestamp(&self) -> TimeTicks;
}
// Source dependency: ui/latency/latency_info.h:64-67,163.
// These named variants select upstream components, not replacement numeric IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostScrollLatencyComponent {
    FirstScrollUpdateOriginal,
    ScrollUpdateOriginal,
}
pub trait HostScrollLatency {
    fn FindLatency(&self, component: HostScrollLatencyComponent, timestamp: &mut TimeTicks)
        -> bool;
}
// Source: cc/trees/layer_tree_host_impl.cc:6279-6288.
#[derive(Clone, Copy, Debug)]
pub struct FirstScrollTrackingPayload {
    pub source_frame_number: i32,
    pub event_creation: TimeTicks,
}
// Source dependency: cc/trees/layer_context.h:69-74.
pub trait HostUnboundedLayerContext<B: HostImplTypes> {
    fn SetUnboundedFrameSinkId(
        &mut self,
        frame_sink: &<B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId,
        local: &<B as LayerTreeTypes>::LocalSurfaceId,
    );
    fn DismissUnboundedFrameSink(&mut self);
    fn SetUnboundedLocalSurfaceId(&mut self, local: &<B as LayerTreeTypes>::LocalSurfaceId);
}
// Direct delegate/platform forwarding only; all branching, native flash state,
// scroll mutation and callback arithmetic are in translated chunk J.
pub trait HostImplJServices<B: HostImplTypes> {
    // Source: cc/trees/layer_tree_host_impl.cc:6143.
    fn NotifyAnimationWorkletStateChange(
        &mut self,
        delegate: HostImplDelegateId,
        state: AnimationWorkletMutationState,
        tree_type: ElementListType,
    );
    // Source: cc/trees/layer_tree_host_impl.cc:6234,6247,6253-6254.
    fn SetNeedsImplSideInvalidation(
        &mut self,
        delegate: HostImplDelegateId,
        needs_first_draw_on_activation: bool,
    );
    // Source: cc/trees/layer_tree_host_impl.cc:6238,6242.
    fn IsInSynchronousComposite(&self, delegate: HostImplDelegateId) -> bool;
    fn IsReadyToActivate(&self, delegate: HostImplDelegateId) -> bool;
    // Source: cc/trees/layer_tree_host_impl.cc:6279-6288.
    // The root wraps a typed weak identity and payload in the successful callback.
    // Invocation resolves the current live HostImpl/context and synchronously
    // calls canonical OnFirstScrollPresented on the same compositor sequence.
    // Invalid weak targets do nothing; no second task/async queue is permitted.
    fn RouteFirstScrollCallback(
        &mut self,
        target: HostImplWeak,
        payload: FirstScrollTrackingPayload,
    ) -> crate::adapter::presentation::SuccessfulCallback;
    // Source: cc/trees/layer_tree_host_impl.cc:6447; cc/trees/unbounded_frame_sink_handler.cc:20-21.
    fn UnboundedClientIdentity(
        &mut self,
        owner: LayerTreeHostId,
    ) -> <B as crate::adapter::unbounded::UnboundedTypes>::ClientIdentity;
}
// Native borrow-only bridges are owned by the translated HostImpl writer:
// WithActiveTree/WithPendingTree/WithRecycleTree(context, FnOnce(tree,live_context)).
// WithInputHandler(context, FnOnce(Option<&mut B::InputHandler>,live_input_facade)).
// WithScrollbarAnimationController(id,context,FnOnce(Option<&mut controller>,client)).
// WithScrollbarFlashControllers(context,FnOnce(&mut resident controller map,
//     &mut flashed_scrollbars_, &mut previously_visible_scrollable_elements_,client)).
// Scrollbar map values are Option<Box<controller>> to retain source factory-null slots.
// The approved pub(crate) LayerTreeImpl::OnScrollOffsetAnimated(id,index,offset,
//     &mut LayerTreeContext) bridge uses only its resident sibling field split,
// validates owner identity, and calls canonical ScrollTree::OnScrollOffsetAnimated.
// Source: cc/trees/layer_tree_host_impl.cc:5996-6012; cc/trees/property_tree.cc:1867-1885.

use super::geometry::RRectF;
// gfx and Viz fields stay owned by the surrounding backend. Values below are
// exact public Viz wire outputs, not a resident tree/render-state copy.
// Source dependency: components/viz/common/hit_test/hit_test_region_list.h:20-64.
pub mod HostHitTestFlags {
    pub const kHitTestMine: u32 = 0x01;
    pub const kHitTestIgnore: u32 = 0x02;
    pub const kHitTestChildSurface: u32 = 0x04;
    pub const kHitTestAsk: u32 = 0x08;
    pub const kHitTestMouse: u32 = 0x10;
    pub const kHitTestTouch: u32 = 0x20;
    pub const kHitTestNotActive: u32 = 0x40;
    pub const kNotAsyncHitTest: u32 = 0;
    pub const kOverlappedRegion: u32 = 1;
    pub const kIrregularClip: u32 = 2;
    pub const kRegionNotActive: u32 = 4;
    pub const kAsyncHitTestReasonCount: u32 = 3;
}
// Source dependency: components/viz/common/hit_test/hit_test_region_list.h:66-88.
pub struct HostHitTestRegion<B: HostImplTypes> {
    pub flags: u32,
    pub async_hit_test_reasons: u32,
    pub frame_sink_id: B::HitTestFrameSinkId,
    pub rect: RRectF,
    pub transform: <B as LayerTreeTypes>::Transform,
}
impl<B: HostImplTypes> Default for HostHitTestRegion<B> {
    fn default() -> Self {
        Self {
            flags: 0,
            async_hit_test_reasons: 0,
            frame_sink_id: Default::default(),
            rect: Default::default(),
            transform: Default::default(),
        }
    }
}
// Source dependency: components/viz/common/hit_test/hit_test_region_list.h:90-123.
pub struct HostHitTestRegionList<B: HostImplTypes> {
    pub flags: u32,
    pub async_hit_test_reasons: u32,
    pub bounds: Rect,
    pub transform: <B as LayerTreeTypes>::Transform,
    pub regions: Vec<HostHitTestRegion<B>>,
}
impl<B: HostImplTypes> Default for HostHitTestRegionList<B> {
    fn default() -> Self {
        Self {
            flags: 0,
            async_hit_test_reasons: 0,
            bounds: Default::default(),
            transform: Default::default(),
            regions: Vec::new(),
        }
    }
}
// Source consumers: cc/trees/layer_tree_host_impl.cc:3780-3930. Each query reads
// the current native layer; this facade does not cache/copy a SurfaceLayer.
pub trait HostImplHitTestLayer<B: LayerTreeTypes> {
    fn draws_content(&self) -> bool;
    /// Source IsSurfaceLayer()/static_cast<SurfaceLayerImpl*> share one actual
    /// subtype projection. Non-surface layers do not invent surface fields.
    fn AsHitTestSurfaceLayer(&self) -> Option<&dyn HostHitTestSurfaceLayer<B>>;
    fn is_clipped(&self) -> bool;
}
pub trait HostHitTestSurfaceLayer<B: LayerTreeTypes> {
    fn has_pointer_events_none(&self) -> bool;
    fn surface_hit_testable(&self) -> bool;
    fn surface_range_is_valid(&self) -> bool;
    fn surface_range_end(&self) -> &<B as HostImplTypes>::HitTestSurfaceId
    where
        B: HostImplTypes;
}
// Source dependency: components/viz/common/surfaces/surface_id.h frame_sink_id().
pub trait HostHitTestSurfaceId {
    type FrameSinkId;
    fn frame_sink_id(&self) -> Self::FrameSinkId;
}
// Source dependency: gpu/config/gpu_feature_info.h:27-34.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum HostGpuFeatureStatus {
    kGpuFeatureStatusEnabled = 0,
    kGpuFeatureStatusBlocklisted = 1,
    kGpuFeatureStatusDisabled = 2,
    kGpuFeatureStatusSoftware = 3,
    kGpuFeatureStatusUndefined = 4,
    kGpuFeatureStatusMax = 5,
}
// Source dependency: gpu/config/gpu_feature_type.h:13-28.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum HostGpuFeatureType {
    GPU_FEATURE_TYPE_ACCELERATED_2D_CANVAS = 0,
    GPU_FEATURE_TYPE_ACCELERATED_WEBGL = 1,
    GPU_FEATURE_TYPE_ACCELERATED_VIDEO_DECODE = 2,
    GPU_FEATURE_TYPE_ACCELERATED_VIDEO_ENCODE = 3,
    GPU_FEATURE_TYPE_GPU_TILE_RASTERIZATION = 4,
    GPU_FEATURE_TYPE_ACCELERATED_WEBGL2 = 5,
    GPU_FEATURE_TYPE_ACCELERATED_GL = 6,
    GPU_FEATURE_TYPE_VULKAN = 7,
    GPU_FEATURE_TYPE_ACCELERATED_WEBGPU = 8,
    GPU_FEATURE_TYPE_SKIA_GRAPHITE = 9,
    GPU_FEATURE_TYPE_WEBNN = 10,
    GPU_FEATURE_TYPE_DIRECT_RENDERING_DISPLAY_COMPOSITOR = 11,
    GPU_FEATURE_TYPE_WEBGPU_ON_VK_VIA_GL_INTEROP = 12,
    NUMBER_OF_GPU_FEATURE_TYPES = 13,
}
// Live calls on the existing RAII lock; owner supplies context/GPU interfaces.
// Source consumer: cc/trees/layer_tree_host_impl.cc:3537-3579.
pub trait HostRasterContextLock {
    fn max_texture_size(&self) -> i32;
    fn BestSupportedTextureFormat(&self) -> crate::adapter::resource_types::SharedImageFormat;
    fn supports_scanout_shared_images(&self) -> bool;
    fn FeatureStatus(&self, feature: HostGpuFeatureType) -> HostGpuFeatureStatus;
    fn msaa_is_slow(&self) -> bool;
    fn avoid_stencil_buffers(&self) -> bool;
    fn BestSupportedRenderbufferFormat(&self) -> crate::adapter::resource_types::SharedImageFormat;
}
// Extend HostImplServices with this live facet. No callback below makes cc state
// decisions; canonical frame/damage/input/animation code owns all sequencing.
pub trait HostImplEServices<B: HostImplTypes> {
    fn ScaleToEnclosingRect(&self, rect: Rect, scale: f32) -> Rect;
    fn IsLowEndDevice(&self) -> bool;
    fn FrameTokenValue(&self, token: &B::FrameTokenGenerator) -> u32;
    fn BeginFrameTrace(&mut self) -> &mut dyn crate::adapter::trace_sink::TraceSink;
    fn TrackersNotifyBeginImplFrame(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        args: &crate::adapter::begin_frame::BeginFrameArgs,
    );
    fn ReportingSetNeedsRasterPropertiesAnimated(
        &mut self,
        reporter: &mut B::CompositorFrameReportingController,
        animated: bool,
    );
    fn DevtoolsDidBeginFrame(
        &mut self,
        id: i32,
        time: crate::adapter::time::TimeTicks,
        sequence: u64,
    );
    fn LocalSurfaceIdIsNewerThan(
        &self,
        target: &<B as LayerTreeTypes>::LocalSurfaceId,
        upcoming: &<B as LayerTreeTypes>::LocalSurfaceId,
    ) -> bool;
    fn TrackersNotifyFrameEnd(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        current: &crate::adapter::begin_frame::BeginFrameArgs,
        args: &crate::adapter::begin_frame::BeginFrameArgs,
    );
    fn TraceDidNotProduceFrame(
        &self,
        ack: &<B as crate::adapter::frame_data::FrameDataTypes>::BeginFrameAck,
        reason_name: &str,
    );
    fn FrameSinkDidNotProduceFrame(
        &mut self,
        sink: HostImplFrameSinkId,
        ack: &<B as crate::adapter::frame_data::FrameDataTypes>::BeginFrameAck,
        reason: crate::cc::scheduler::scheduler::FrameSkippedReason,
    );
    fn DropSavedEventMetricsForNoFrameUpdate(&mut self, manager: &mut B::EventsMetricsManager);
    fn DidChangeBeginFrameSourcePaused(&mut self, delegate: HostImplDelegateId, paused: bool);
    fn SynchronousTaskGraphRunnerRunUntilIdle(
        &mut self,
        runner: &mut B::SynchronousTaskGraphRunner,
    );
    fn TraceBuildHitTestData(&self);
}

// Canonical PropertyTrees pure live sibling bridge required by source
// cc/trees/layer_tree_host_impl.cc:3890-3892. Add to existing canonical impl:
// pub fn ClippedHitTestRegionIsRectangle(&self,id:i32)->bool {
//   self.effect_tree_.ClippedHitTestRegionIsRectangle(
//     id,&self.transform_tree_,&self.cached_data_)
// }

pub enum HostRasterTaskGraphRunner<'a, B: HostImplTypes> {
    Injected(HostImplTaskGraphRunnerId),
    Synchronous(&'a B::SynchronousTaskGraphRunner),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostSharedImageInterfaceId(pub u64);
pub trait HostImplFServices<B: HostImplTypes> {
    fn DelegateDidLoseLayerTreeFrameSink(&mut self, delegate: HostImplDelegateId);
    fn LagTrackingClear(&mut self, lag: &mut B::AverageLagTrackingManager);
    fn FrameSorterReset(&mut self, sorter: &mut B::FrameSorter, reset_fcp: bool);
    // Source lambda is evaluated at most once, and only when trace event is enabled.
    fn TraceActivateSyncTree(
        &self,
        trace: &mut dyn FnMut() -> crate::adapter::commit_types::BeginMainFrameTraceId,
    );
    fn TracePendingTreeActivated(
        &self,
        tree: LayerTreeId,
        local: &<B as LayerTreeTypes>::LocalSurfaceId,
    );
    fn TraceTreeSynchronization(&self, name: &str, count: Option<usize>);
    fn PromoteScrollTimelinesPendingToActive(
        &mut self,
        mutator: &mut <B as LayerTreeTypes>::MutatorHost,
    );
    fn InvalidPreviousSurfaceScreenshot(&self);
    fn DelegateDidActivateSyncTree(&mut self, delegate: HostImplDelegateId);
    fn RunTreeActivationCallback(&mut self, callback: &<B as HostImplTypes>::RepeatingClosure);
    fn VerboseLogActivatedTree(&self, tree: &LayerTreeImpl<B>);
    fn ReportingSetVisible(
        &mut self,
        reporting: &mut B::CompositorFrameReportingController,
        visible: bool,
    );
    fn LayerContextSetVisible(
        &mut self,
        layer_context: &mut <B as HostImplTypes>::LayerContext,
        visible: bool,
    );
    fn ResourceProviderSetVisible(
        &mut self,
        provider: &mut B::ClientResourceProvider,
        visible: bool,
    );
    fn ImageDecodeFactory(&self) -> &B::ImageDecodeFactory;
    fn ImageDecodeContextProvider(
        &self,
        provider: Option<HostImplContextProviderId>,
    ) -> <B::ImageDecodeFactory as HostImplImageDecodeFactory>::RasterContextProvider;
    fn ImageDecodeDarkModeFilter(
        &self,
        id: Option<HostImplDarkModeFilterId>,
    ) -> Option<&<B::ImageDecodeFactory as HostImplImageDecodeFactory>::DarkModeFilter>;
    fn CreateRasterQueryQueue(
        &mut self,
        worker: HostImplContextProviderId,
    ) -> Box<B::RasterQueryQueue>;
    fn CreateSynchronousTaskGraphRunner(&mut self) -> Box<B::SynchronousTaskGraphRunner>;
    fn FrameSinkSharedImageInterface(
        &self,
        sink: HostImplFrameSinkId,
    ) -> HostSharedImageInterfaceId;
    fn ContextProviderSharedImageInterface(
        &self,
        provider: HostImplContextProviderId,
    ) -> HostSharedImageInterfaceId;
    fn CreateZeroCopyRasterBufferProvider(
        &mut self,
        interface: HostSharedImageInterfaceId,
        is_software: bool,
    ) -> Box<B::RasterBufferProvider>;
    fn CreateGpuRasterBufferProvider(
        &mut self,
        interface: HostSharedImageInterfaceId,
        compositor: HostImplContextProviderId,
        worker: HostImplContextProviderId,
        overlay_candidate: bool,
        max_tile_size: Size,
        queries: Option<&B::RasterQueryQueue>,
    ) -> Box<B::RasterBufferProvider>;
    fn MissingWorkerContextForcingZeroCopy(&self);
    fn CreateOneCopyRasterBufferProvider(
        &mut self,
        interface: HostSharedImageInterfaceId,
        runner: HostImplTaskRunnerId,
        compositor: HostImplContextProviderId,
        worker: HostImplContextProviderId,
        use_partial_raster: bool,
        max_staging_bytes: i32,
        overlay_candidate: bool,
    ) -> Box<B::RasterBufferProvider>;
}
// This trace-only live adapter has no main-thread layer owner or fallback path.
pub struct HostPendingSynchronizationTrace<'a, B: HostImplTypes>(pub &'a dyn HostImplServices<B>);
impl<B: HostImplTypes> crate::adapter::tree_sync::SynchronizationTrace
    for HostPendingSynchronizationTrace<'_, B>
{
    fn TraceScope(&self, name: &str, count: Option<usize>) {
        self.0.TraceTreeSynchronization(name, count);
    }
}

use super::layer_tree_impl::LayerTreeId;
use crate::cc::trees::layer_tree_impl::LayerTreeImpl;

pub trait HostImplAServices<B: HostImplTypes> {
    fn TraceHostImplAEvent(&self, name: &'static str);
    fn DelegateDidPrepareTiles(&mut self, delegate: HostImplDelegateId);
    fn CreateLatencyInfoSwapPromiseMonitor(
        &mut self,
        latency: Option<&mut B::LatencyInfo>,
        owner: HostImplWeak,
    ) -> Box<B::LatencyInfoSwapPromiseMonitor>;
    fn GetScopedEventMetricsMonitor(
        &mut self,
        manager: &mut B::EventsMetricsManager,
        callback: B::EventMetricsDoneCallback,
    ) -> Box<B::EventMetricsScopedMonitor>;
}

pub trait HostImplBrowserControlsServices {
    fn EmitBrowserControlsDiagnostic(
        &mut self,
        event: super::browser_controls::BrowserControlsDiagnostic,
    );
}

// Integrated HostImpl source chunks B/C/G typed boundaries.
pub mod host_impl_chunk_b {
    use super::*;
    // B external typed boundary proposals. No cc management algorithm lives here.
    // Stronger external payload constraints belong only to HostImplTypes:
    // + LayerTreeTypes<Layer:HostDrawLayer<Self>+HostImplLayer<
    //     Transform=<Self as LayerTreeTypes>::Transform,
    //     RegionBackend=<Self as LayerTreeTypes>::RegionBackend>,
    //     RenderSurface:HostDrawSurface<Self>>
    // + FrameDataTypes<RenderPass:HostDrawPass<Self>,SurfaceId:Clone>.
    // HostImplTypes adds type RenderPassId:Copy+Ord+From<u64>;
    // type SharedQuadStateHandle:Copy; type DrawQuadHandle:Copy;
    // Existing ThrottleContext adds exact associated equality constraints:
    // Transform=<Self as LayerTreeTypes>::Transform, PassId=Self::RenderPassId,
    // PassHandle=HostRenderPassLocation, Pass=Self::RenderPass.
    // HostImplServices extends HostImplBServices<B>.
    use crate::adapter::{
        draw_property::ViewTransitionToken,
        geometry::Rect,
        layer_tree_impl::{Color4f, LayerTreeId, LayerTreeTypes, PropertyTreesFor},
        property_types::ViewTransitionElementResourceId,
    };
    use crate::cc::{
        base::region::Region,
        layers::{
            append_quads_context::AppendQuadsContext, append_quads_data::AppendQuadsData,
            draw_mode::DrawMode, effect_tree_layer_list_iterator::IteratorContext,
        },
        trees::frame_data::FrameData,
    };
    use std::collections::BTreeSet;
    // Pure split of original LTI fields. Constructor belongs to LTI, whose private
    // fields are borrowed here; it performs no rendering/layout decision.
    pub struct RenderPassState<'a, B: LayerTreeTypes> {
        pub tree_id: LayerTreeId,
        pub layer_list: &'a mut B::LayerList,
        pub property_trees: &'a mut PropertyTreesFor<B>,
        pub render_surface_list: &'a [i32],
        pub unoccluded_screen_space_region: &'a Region<B::RegionBackend>,
    }
    impl<B: LayerTreeTypes> IteratorContext for RenderPassState<'_, B> {
        type Types = B;
        fn id(&self) -> LayerTreeId {
            self.tree_id
        }
        fn num_layers(&self) -> usize {
            use crate::adapter::layer_tree_impl::LayerTreeLayerList;
            self.layer_list.len()
        }
        fn LayerAtReverseIndex(&self, index: usize) -> Option<&B::Layer> {
            use crate::adapter::layer_tree_impl::LayerTreeLayerList;
            let len = self.layer_list.len();
            if index < len {
                Some(self.layer_list.LayerAt(len - 1 - index))
            } else {
                None
            }
        }
        fn property_trees(&self) -> &PropertyTreesFor<B> {
            self.property_trees
        }
    }
    impl<B: LayerTreeTypes> crate::adapter::frame_data::FrameDataSurfaceList
        for RenderPassState<'_, B>
    {
        fn ActiveTreeId(&self) -> LayerTreeId {
            self.tree_id
        }
        fn GetActiveRenderSurfaceList(&self) -> &[i32] {
            self.render_surface_list
        }
    }
    // Native Viz materials needed for source casts; other variants are left opaque.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum HostQuadMaterial {
        kSharedElement,
        kCompositorRenderPass,
        Other,
    }
    pub trait HostDrawPass<B: HostImplTypes>:
        crate::adapter::throttle::ThrottlePass<
        <B as LayerTreeTypes>::Transform,
        B::RenderPassId,
        <B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId,
    >
    {
        fn Create() -> Box<Self>;
        fn SetNew(
            &mut self,
            id: B::RenderPassId,
            output: Rect,
            damage: Rect,
            transform: <B as LayerTreeTypes>::Transform,
        );
        fn set_damage_rect(&mut self, rect: Rect);
        fn set_has_transparent_background(&mut self, value: bool);
        fn copy_requests(&self) -> &Vec<Box<B::CopyOutputRequest>>;
        fn copy_requests_mut(&mut self) -> &mut Vec<Box<B::CopyOutputRequest>>;
        fn quad_count(&self) -> usize;
        fn quad_material(&self, index: usize) -> HostQuadMaterial;
        fn quad_shared_element_resource_id(&self, index: usize) -> ViewTransitionElementResourceId;
        fn quad_render_pass_id(&self, index: usize) -> Option<B::RenderPassId>;
        fn EraseQuadAndInvalidateAllPointers(&mut self, index: usize);
        fn quad_has_shared_quad_state(&self, index: usize) -> bool;
        fn subtree_capture_id_is_valid(&self) -> bool;
        fn filters_is_empty(&self) -> bool;
        fn backdrop_filters_is_empty(&self) -> bool;
        fn view_transition_element_resource_id(&self) -> ViewTransitionElementResourceId;
        fn CreateAndAppendSharedQuadState(&mut self) -> B::SharedQuadStateHandle;
        fn SetSharedQuadStateAll(
            &mut self,
            state: B::SharedQuadStateHandle,
            transform: <B as LayerTreeTypes>::Transform,
            quad_layer_rect: Rect,
            visible_quad_layer_rect: Rect,
            mask_filter_info: crate::adapter::property_types::MaskFilterInfo,
            clip_rect: Option<Rect>,
            are_contents_opaque: bool,
            opacity: f32,
            blend_mode: crate::adapter::occlusion::SkBlendMode,
            sorting_context: i32,
            layer_id: u32,
            fast_rounded_corner: bool,
        );
        fn CreateAndAppendSolidColorDrawQuad(&mut self) -> B::DrawQuadHandle;
        fn SolidColorDrawQuadSetNew(
            &mut self,
            quad: B::DrawQuadHandle,
            state: B::SharedQuadStateHandle,
            rect: Rect,
            visible: Rect,
            color: Color4f,
            force_anti_aliasing_off: bool,
        );
    }
    pub trait HostDrawSurface<B: HostImplTypes> {
        fn GetDamageRect(&self) -> Rect;
        fn contributes_to_drawn_surface(&self) -> bool;
        fn ViewTransitionElementResourceId(
            &self,
            trees: &PropertyTreesFor<B>,
        ) -> ViewTransitionElementResourceId;
        fn CreateViewTransitionCaptureRenderPass(
            &self,
            tokens: &BTreeSet<ViewTransitionToken>,
            c: &mut dyn crate::adapter::render_surface_impl::RenderSurfaceContext<B>,
        ) -> Box<B::RenderPass>;
        fn CreateRenderPass(
            &self,
            tokens: &BTreeSet<ViewTransitionToken>,
            c: &mut dyn crate::adapter::render_surface_impl::RenderSurfaceContext<B>,
        ) -> Box<B::RenderPass>;
        fn render_pass_id(&self) -> B::RenderPassId;
        fn view_transition_capture_render_pass_id(&self) -> B::RenderPassId;
        fn HasCopyRequest(&self, trees: &PropertyTreesFor<B>) -> bool;
        fn AppendQuads(
            &self,
            c: &AppendQuadsContext,
            target: &mut B::RenderPass,
            data: &mut AppendQuadsData<B::SurfaceId>,
            surface_context: &dyn crate::adapter::render_surface_impl::RenderSurfaceContext<B>,
        );
        fn draw_transform(&self) -> &<B as LayerTreeTypes>::Transform;
    }
    pub trait HostDrawLayer<B: HostImplTypes>:
        crate::adapter::throttle::ThrottleLayer<
        <B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId,
    >
    {
        // Tracked-element lists stay native; caller iterates live borrowed flags.
        type TrackedRectLists<'a>: Iterator<Item = Self::TrackedRectList<'a>>
        where
            Self: 'a;
        type TrackedRectList<'a>: Iterator<Item = bool>
        where
            Self: 'a;
        fn tracked_element_rects_empty(&self) -> Option<bool>;
        fn tracked_element_rects(&self) -> Self::TrackedRectLists<'_>;
        // All selected resident subtypes inherit LayerImpl::WillDraw, whose
        // source provider argument is unused (cc/layers/layer_impl.cc:279-301).
        // Actual resource use is obtained inside the live Picture context.
        fn WillDraw(&mut self, mode: DrawMode, context: &mut B::PictureLayerContext<'_>) -> bool;
        fn layer_tree_impl_id(&self) -> LayerTreeId;
        fn NotifyKnownResourceIdsBeforeAppendQuads(
            &mut self,
            ids: &BTreeSet<ViewTransitionElementResourceId>,
        );
        fn AppendQuads(
            &mut self,
            c: &AppendQuadsContext,
            target: &mut B::RenderPass,
            data: &mut AppendQuadsData<B::SurfaceId>,
            context: &mut B::PictureLayerContext<'_>,
        );
        fn HasMissingTiles(&self, context: &mut B::PictureLayerContext<'_>) -> bool;
        fn screen_space_transform_is_animating(&self) -> bool;
        fn ComputeCheckerboardedNeedsRecord(
            &mut self,
            context: &mut B::PictureLayerContext<'_>,
        ) -> bool;
    }
    pub trait HostImplBServices<B: HostImplTypes> {
        fn TraceEmptyDamageRect(&self);
        fn TraceCalculateRenderPassesBegin(&self, size: u64, requires_high_res: bool);
        fn TraceCalculateRenderPassesEnd(
            &self,
            result: crate::cc::scheduler::draw_result::DrawResult,
            missing_tiles: Option<i32>,
        );
        fn ShouldRecordSubsampledMetric(&self, probability: f64) -> bool;
        // RAII crash annotations outlive only their enclosing source scope.
        fn CrashKey(&mut self, name: &str, value: HostCrashKeyValue) -> Box<dyn HostCrashKey>;
        fn DumpWillBeCheck(&self, condition: bool, message: &str);
        fn AddVisibleContentArea(&mut self, stats: HostImplRenderingStatsId, area: i64);
        fn AddApproximatedVisibleContentArea(&mut self, stats: HostImplRenderingStatsId, area: i64);
        fn TickWorkletAnimations(&mut self, host: &mut <B as LayerTreeTypes>::MutatorHost);
        fn TracePrepareToDraw(&self, source_frame_number: i32);
        fn VerboseLogFrame(&self, frame: &FrameData<B>);
        fn BeginMainFrameReasonScroll(&self) -> <B as LayerTreeTypes>::BeginMainFrameReason;
        // Geometry formatting operations, never cc culling/damage algorithms.
        fn RectOutset(&self, rect: &mut Rect, amount: i32);
        fn RectInsetsString(&self, outer: Rect, inner: Rect) -> String;
    }
    pub trait HostCrashKey {}
    // Exact base crash-key kind/size preserves source annotation truncation/types.
    pub enum HostCrashKeyValue {
        String64(String),
        String32(String),
        String256(String),
        Boolean(bool),
        Number(u64),
    }
    // LTI pure split accessor proposal: fn RenderPassState(&mut self)->RenderPassState<'_,B>.
    // LTI shared scoped_refptr getters proposal: SharedPageScaleFactor() and
    // SharedTopControlsShownRatio()/SharedBottomControlsShownRatio() return Rc clones
    // of those three source scoped_refptr values only, never trees/siblings.
    // Native pointer identity during the frame pass walk. No pass state is copied.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum HostRenderPassLocation {
        Bounded(usize),
        Unbounded(usize),
    }
    impl HostRenderPassLocation {
        pub fn get<'a, B: HostImplTypes>(self, frame: &'a FrameData<B>) -> &'a B::RenderPass {
            match self {
                Self::Bounded(i) => &frame.render_passes[i],
                Self::Unbounded(i) => &frame.unbounded_render_passes[i],
            }
        }
        pub fn get_mut<'a, B: HostImplTypes>(
            self,
            frame: &'a mut FrameData<B>,
        ) -> &'a mut B::RenderPass {
            match self {
                Self::Bounded(i) => &mut frame.render_passes[i],
                Self::Unbounded(i) => &mut frame.unbounded_render_passes[i],
            }
        }
    }
    pub struct BorrowedFrameThrottle<'a, B: HostImplTypes> {
        frame: &'a FrameData<B>,
    }
    impl<'a, B: HostImplTypes> BorrowedFrameThrottle<'a, B> {
        pub fn new(frame: &'a FrameData<B>) -> Self {
            Self { frame }
        }
    }
    impl<B: HostImplTypes> crate::adapter::throttle::ThrottleTypes for BorrowedFrameThrottle<'_, B> {
        type Transform = <B as LayerTreeTypes>::Transform;
        type PassId = B::RenderPassId;
        type FrameSinkId =
            <B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId;
        type PassHandle = HostRenderPassLocation;
        type Pass = B::RenderPass;
    }
    impl<B: HostImplTypes> crate::adapter::throttle::ThrottleContext for BorrowedFrameThrottle<'_, B> {
        fn pass(&self, id: Self::PassHandle) -> &Self::Pass {
            id.get::<B>(self.frame)
        }
    }
    // Exact LTI pure split accessor body proposal:
    // pub fn RenderPassState(&mut self)->RenderPassState<'_,B> {
    //   debug_assert!(!self.needs_update_draw_properties_);
    //   RenderPassState { tree_id:self.id,layer_list:&mut self.layer_list_,
    //     property_trees:&mut self.property_trees_,render_surface_list:&self.render_surface_list_,
    //     unoccluded_screen_space_region:&self.unoccluded_screen_space_region_ }
    // }
    // SharedPageScaleFactor, SharedTopControlsShownRatio, SharedBottomControlsShownRatio
    // return Rc::clone of corresponding three source scoped_refptr members.

    // Canonical FrameData non-owning surface-list pointer resolution, live only.
    impl<B: LayerTreeTypes> crate::adapter::frame_data::FrameDataSurfaceList
        for crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>
    {
        fn ActiveTreeId(&self) -> LayerTreeId {
            self.id
        }
        fn GetActiveRenderSurfaceList(&self) -> &[i32] {
            self.GetRenderSurfaceList()
        }
    }
}
pub use host_impl_chunk_b::*;
pub mod host_impl_chunk_c {
    use super::*;
    // Typed external owner operations for HostImpl source chunk C. Integrate the
    // types/bounds/services into the shared HostImpl adapter; no native cc state
    // machine, resource policy, callback arithmetic, or tree copy belongs here.
    use crate::adapter::{
        begin_frame::BeginFrameArgs,
        geometry::{Rect, RectF},
        image_animation::{PaintImageId, WhichTree},
        layer_tree_host_impl::{
            ContentColorUsage, HostImplContextProviderId, HostImplDelegateId, HostImplFrameSinkId,
            HostImplTaskRunnerId, HostImplTypes, HostImplWeak, HostUIResourceReleaseClient,
        },
        layer_tree_impl::{LayerTreeLayerList, LayerTreeTypes},
        property_types::ViewTransitionElementResourceId,
        time::TimeDelta,
    };
    use crate::cc::trees::presentation_time_callback_buffer::PendingCallbacks;
    use std::collections::HashMap;
    // Source dependency: cc/tiles/raster_tile_priority_queue.h:23.
    pub use crate::cc::tiles::raster_tile_priority_queue::Type as HostRasterQueueType;
    // Source dependency: cc/paint/paint_op_buffer.h:111.
    pub const kMinNumberOfSlowPathsForMSAA: i32 = 6;
    // Source dependency: cc/paint/target_color_params.h:18-35. This typed paint
    // boundary retains both native fields and the source default sRGB/no headroom.
    #[derive(Clone, Debug, PartialEq)]
    pub struct HostTargetColorParams<C> {
        pub color_space: C,
        pub hdr_headroom: Option<f32>,
    }
    impl<C: HostColorSpace> Default for HostTargetColorParams<C> {
        fn default() -> Self {
            Self {
                color_space: C::CreateSRGB(),
                hdr_headroom: None,
            }
        }
    }
    impl<C> HostTargetColorParams<C> {
        pub fn GetHdrHeadroom(&self) -> f32 {
            self.hdr_headroom.unwrap_or(0.)
        }
    }
    // Color math is supplied by the existing geometry/color owner.
    // Source dependency: ui/gfx/color_space.h:65-92.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum HostColorSpacePrimaryID {
        INVALID = 0,
        BT709,
        BT470M,
        BT470BG,
        SMPTE170M,
        SMPTE240M,
        FILM,
        BT2020,
        SMPTEST428_1,
        SMPTEST431_2,
        P3,
        XYZ_D50,
        ADOBE_RGB,
        APPLE_GENERIC_RGB,
        WIDE_GAMUT_COLOR_SPIN,
        CUSTOM,
        EBU_3213_E,
    }
    impl HostColorSpacePrimaryID {
        pub const kMaxValue: Self = Self::EBU_3213_E;
    }
    pub trait HostColorSpace: Clone + Ord {
        fn CreateSRGB() -> Self;
        fn IsValid(&self) -> bool;
        fn IsHDR(&self) -> bool;
        fn GetPrimaryID(&self) -> HostColorSpacePrimaryID;
        /// Exact fixed argument used by HostImpl: native Contains(CreateSRGB()).
        fn ContainsSRGB(&self) -> bool;
    }
    pub trait HostDisplayColorSpaces<C> {
        fn GetRasterAndCompositeColorSpace(&self, usage: ContentColorUsage) -> C;
        fn GetHdrHeadroom(&self) -> f32;
    }
    // Source dependency: base/containers/lru_cache.h. Get updates recency, and Put
    // applies the capacity set by CreateColorSpaceCache; no extra compositor cache.
    pub trait HostColorSpaceCache<C> {
        fn Get(&mut self, key: &C) -> Option<&bool>;
        fn Put(&mut self, key: C, value: bool);
    }
    // Source dependency: cc/paint/paint_image.h:179,385,489.
    pub trait HostPaintImage {
        fn ShouldAnimate(&self) -> bool;
        fn stable_id(&self) -> PaintImageId;
    }
    pub trait HostDisplayItemList {
        fn num_slow_paths_up_to_min_for_MSAA(&self) -> i32;
        fn has_non_aa_paint(&self) -> bool;
    }
    // A resident paint-owned tile, not a copied tiling/tree record.
    // Source dependency: cc/tiles/tile.h:94,126,169; cc/tiles/picture_layer_tiling.h:435.
    pub trait HostTile {
        fn tiling_id(&self) -> Option<crate::adapter::tile::PictureLayerTilingId>;
        fn layer_id(&self) -> i32;
        fn required_for_draw(&self) -> bool;
    }
    pub trait HostTileLayer<B: HostImplTypes> {
        fn NotifyTileStateChanged(
            &mut self,
            tile: &B::Tile,
            update_damage: bool,
            context: &mut B::PictureLayerContext<'_>,
        );
    }
    pub trait PictureLayerDisplayRead {
        fn id(&self) -> i32;
    }
    /// Vtable on the actual resident PictureLayerTilingState. The external Viz
    /// serializer only reads these native geometry getters; no fields are copied.
    pub trait HostDisplayTileTiling {
        fn raster_translation(&self) -> &crate::adapter::geometry::Vector2dF;
        fn raster_scale(&self) -> &crate::adapter::geometry::Vector2dF;
        fn tile_size(&self) -> crate::adapter::geometry::Size;
        fn tiling_rect(&self) -> &crate::adapter::geometry::Rect;
    }
    pub trait HostTilePictureLayer: PictureLayerDisplayRead {
        fn ResidentDisplayTiling(
            &self,
            id: crate::adapter::tile::PictureLayerTilingId,
        ) -> Option<&dyn HostDisplayTileTiling>;
        fn should_batch_updated_tiles(&self) -> bool;
        /// Pure live query on this PictureLayerImpl's own resident tiling set.
        fn ResidentTilingTree(
            &self,
            id: crate::adapter::tile::PictureLayerTilingId,
        ) -> Option<WhichTree>;
    }
    // Source dependency: components/viz/common/view_transition_element_resource_id.h.
    pub type HostViewTransitionElementResourceRects =
        HashMap<ViewTransitionElementResourceId, RectF>;
    // Required HostImplTypes supertrait. Shared original associated types additionally
    // require ColorSpace:HostColorSpace, ColorSpaceCache:HostColorSpaceCache<ColorSpace>,
    // DisplayColorSpaces:HostDisplayColorSpaces<ColorSpace>, Layer:HostTileLayer<Self>,
    // DisplayItemList:HostDisplayItemList,
    // PictureLayer:HostTilePictureLayer, HostFrameTimingDetails:Clone, and
    // Host field uses the concrete HostViewTransitionElementResourceRects alias;
    // remove the obsolete ViewTransitionElementResourceRects associated type.
    pub trait HostImplCTypes {
        type PaintImage: HostDecodePaintImage;
        type Tile: HostTile;
        type CustomTrackerResults;
        type EventLatencyData;
    }
    // Source: cc/trees/layer_tree_host_impl.cc:2208-2215,2257-2262.
    // The platform queues this once payload. On execution the root validates weak
    // identity and dispatches RunPostedCompositorTask synchronously on that sequence.
    // Copying details copies only the native Viz feedback value, never a tree/host.
    pub struct HostImplPostedCompositorTask<D> {
        pub target: HostImplWeak,
        pub kind: HostImplPostedCompositorTaskKind<D>,
    }
    pub enum HostImplPostedCompositorTaskKind<D> {
        LogAverageLagEvents { frame_token: u32, details: D },
        MaybeFlushPendingWork,
    }
    // Only direct native service calls. Queue arguments are live iterator borrows;
    // any required tile identity retention belongs to the existing tile owner.
    pub trait HostImplCServices<B: HostImplTypes>: HostImplResourceReturnServices<B> {
        fn TraceHostImplCEvent(&mut self, category: &'static str, name: &'static str);
        fn LayerContextSetTargetLocalSurfaceId(
            &mut self,
            context: &mut <B as HostImplTypes>::LayerContext,
            id: &<B as LayerTreeTypes>::LocalSurfaceId,
        );
        fn DelegateNotifyReadyToActivate(&mut self, delegate: HostImplDelegateId);
        fn DelegateNotifyReadyToDraw(&mut self, delegate: HostImplDelegateId);
        fn UpdateDisplayTile(
            &mut self,
            context: &mut <B as HostImplTypes>::LayerContext,
            layer: &dyn PictureLayerDisplayRead,
            tiling: &dyn HostDisplayTileTiling,
            tile: &B::Tile,
            provider: &mut B::ClientResourceProvider,
            sink: HostImplFrameSinkId,
            update_damage: bool,
        );
        fn DelegateDidReceiveCompositorFrameAckOnImplThread(
            &mut self,
            delegate: HostImplDelegateId,
        );
        fn PresentationFeedbackFailed(&self, details: &B::HostFrameTimingDetails) -> bool;
        fn DelegateDidPresentCompositorFrameOnImplThread(
            &mut self,
            delegate: HostImplDelegateId,
            frame_token: u32,
            callbacks: PendingCallbacks<B::PresentationFeedback, B::HostFrameTimingDetails>,
            details: &B::HostFrameTimingDetails,
        );
        fn PostCompositorTask(
            &mut self,
            runner: HostImplTaskRunnerId,
            location: crate::adapter::task_runner::TaskLocation,
            task: HostImplPostedCompositorTask<B::HostFrameTimingDetails>,
        );

        fn LagTrackingDidPresentCompositorFrame(
            &mut self,
            manager: &mut B::AverageLagTrackingManager,
            frame_token: u32,
            details: &B::HostFrameTimingDetails,
        );
        fn DelegateNotifyCompositorMetricsTrackerResults(
            &mut self,
            delegate: HostImplDelegateId,
            results: &B::CustomTrackerResults,
        );
        fn FrameTrackersNotifyPauseFrameProduction(
            &mut self,
            trackers: &mut B::FrameSequenceTrackerCollection,
        );
        // May run stored release callbacks synchronously against the resident UI map.

        fn DelegateOnDrawForLayerTreeFrameSink(
            &mut self,
            delegate: HostImplDelegateId,
            resourceless: bool,
            skip: bool,
        );
        fn DelegateNotifyTransitionRequestFinished(
            &mut self,
            delegate: HostImplDelegateId,
            sequence_id: u32,
            rects: HostViewTransitionElementResourceRects,
        );
        fn LocalSurfaceIdIsNewerThanOrEmbeddingChanged(
            &self,
            current: &<B as LayerTreeTypes>::LocalSurfaceId,
            other: &<B as LayerTreeTypes>::LocalSurfaceId,
        ) -> bool;
        fn ResourceProviderSetEvicted(
            &mut self,
            provider: &mut B::ClientResourceProvider,
            evicted: bool,
        );
        fn HasCustomMetricRecorder(&self) -> bool;
        fn CustomMetricRecorderReportEventLatency(
            &mut self,
            args: &BeginFrameArgs,
            latencies: Vec<B::EventLatencyData>,
        );
        #[cfg(target_os = "android")]
        fn RenderFrameMetadataObserverReportScrollJankStats(
            &mut self,
            observer: &mut B::RenderFrameMetadataObserver,
            total: u32,
            janky: u32,
        );
    }

    // Move the existing MainThreadTaskRunner/ImplThreadTaskRunner/HasImplThread
    // declarations from HostImplServices into this shared narrow base facet. The
    // full service contract inherits it; do not duplicate same-named methods.
    pub trait HostImplTaskRunnerServices {
        fn MainThreadTaskRunner(
            &self,
            provider: crate::adapter::layer_tree_host_impl::HostImplTaskRunnerProviderId,
        ) -> HostImplTaskRunnerId;
        fn ImplThreadTaskRunner(
            &self,
            provider: crate::adapter::layer_tree_host_impl::HostImplTaskRunnerProviderId,
        ) -> HostImplTaskRunnerId;
        fn HasImplThread(
            &self,
            provider: crate::adapter::layer_tree_host_impl::HostImplTaskRunnerProviderId,
        ) -> bool;
    }
    // HostImplCServices extends this narrow facet. Move FrameSinkContextProvider
    // from HostImplServices here as well. Native binding splits this capability
    // from the sink operation; it never manufactures a second full services owner.
    pub trait HostImplResourceReturnServices<B: HostImplTypes>: HostImplTaskRunnerServices {
        fn FrameSinkContextProvider(
            &self,
            sink: HostImplFrameSinkId,
        ) -> Option<HostImplContextProviderId>;
        fn PostDelayedCompositorTask(
            &mut self,
            runner: HostImplTaskRunnerId,
            location: crate::adapter::task_runner::TaskLocation,
            task: HostImplPostedCompositorTask<B::HostFrameTimingDetails>,
            delay: TimeDelta,
        );
        fn ReceiveReturnsFromParent(
            &mut self,
            provider: &mut B::ClientResourceProvider,
            resources: Vec<<B as crate::adapter::unbounded::UnboundedTypes>::ReturnedResource>,
            pool: Option<&mut B::ResourcePool>,
            client: &mut dyn HostUIResourceReleaseClient<B>,
        );
        fn ResourcePoolReduceResourceUsage(&mut self, pool: &mut B::ResourcePool);
        fn ContextHasSupport(&self, provider: HostImplContextProviderId) -> bool;
        fn ContextSupportFlushPendingWork(&mut self, provider: HostImplContextProviderId);
    }
}
pub use host_impl_chunk_c::*;
pub mod host_impl_chunk_g {
    use super::*;
    // Typed external additions for HostImpl G. Native algorithms, queueing
    // conditions, ownership, weak checks and synchronous client forwarding remain
    // in the same translated layer_tree_host_impl.rs; only external owners live here.
    use crate::adapter::{
        geometry::{PointF, RectF, Vector2dF},
        image_animation::PaintImageId,
        input_types::ElementId,
        layer_tree_host_impl::*,
        layer_tree_impl::{LayerTreeTypes, PropertyTreesFor},
        time::{TimeDelta, TimeTicks},
    };
    // HostImplTypes adds HostImplGTypes; its existing PaintImage must additionally
    // implement HostDecodePaintImage and RenderSurface must implement HostScrollRenderSurface.
    // CompositorCommitData binds to canonical compositor_commit_data::CompositorCommitData.
    // Existing LayerTreeTypes::ImageDecodeCache additionally requires HostImplImageDecodeCache;
    // DecodingMode=HostImageDecodingMode. Reuse CommitStateTypes::DrawImage, no second type.
    pub trait HostImplGTypes:
        HostImplCTypes
        + crate::adapter::commit_state::CommitStateTypes<
            DrawImage: HostDrawImage<PaintImage = <Self as HostImplCTypes>::PaintImage>,
        > + Sized
    {
        type LayerTreeMutator;
    }
    pub trait HostDrawImage {
        type PaintImage;
        fn paint_image(&self) -> &Self::PaintImage;
    }
    // Bound required: DrawImage:HostDrawImage<PaintImage=Self::PaintImage>.
    pub trait HostDecodePaintImage: HostPaintImage {
        fn GetContentColorUsage(&self) -> ContentColorUsage;
    }
    pub trait HostScrollRenderSurface<B: LayerTreeTypes> {
        fn DrawableContentRect(&self, trees: &PropertyTreesFor<B>) -> RectF;
    }
    // Source dependency: cc/paint/paint_image.h:219-232.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum HostImageDecodingMode {
        kUnspecified = 0,
        kSync = 1,
        kAsync = 2,
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4459-4461.
    // Native decode tracker owns/consumes this once endpoint and passes its actual
    // boolean result to canonical RunImageDecodeCallback on the compositor sequence.
    pub struct HostImageDecodeCallback {
        pub target: HostImplWeak,
        pub request_id: i32,
        pub speculative: bool,
    }
    // Source: cc/trees/layer_tree_frame_sink_client.h:33-45.
    // Bind/detach get only this lexical client facade; they may not retain it. Native
    // AsyncLayerTreeFrameSink synchronously calls SetBeginFrameSource while binding
    // and detaching (cc/mojo_embedder/async_layer_tree_frame_sink.cc:128-134,162).
    pub trait HostFrameSinkClientFacade<B: HostImplTypes> {
        fn SetBeginFrameSource(
            &mut self,
            source: Option<&<B as crate::adapter::unbounded::UnboundedTypes>::BeginFrameSource>,
        );
        fn ReclaimResources(
            &mut self,
            resources: Vec<<B as crate::adapter::unbounded::UnboundedTypes>::ReturnedResource>,
        );
    }
    // The native owner stores only the typed weak registration identity for future
    // callbacks, never this facade or a HostImpl/Page pointer/reference.
    pub trait HostFrameSinkOperations<B: HostImplTypes> {
        fn BindToClient(
            &mut self,
            target: HostImplWeak,
            client: &mut dyn HostFrameSinkClientFacade<B>,
        ) -> bool;
        fn DetachFromClient(&mut self, client: &mut dyn HostFrameSinkClientFacade<B>);
    }
    pub trait HostFrameSinkCallbackServices<B: HostImplTypes>:
        HostImplResourceReturnServices<B>
    {
        fn DelegateSetBeginFrameSource(
            &mut self,
            delegate: HostImplDelegateId,
            source: Option<&<B as crate::adapter::unbounded::UnboundedTypes>::BeginFrameSource>,
        );
    }
    // Split actual native sink operation state from its independent callback
    // services. The callback facet exposes no full HostImplServices interface, no
    // native owner copy, and no duplicate resource policy/state implementation.
    pub struct HostFrameSinkAccess<'a, B: HostImplTypes> {
        pub sink: &'a mut dyn HostFrameSinkOperations<B>,
        pub callbacks: &'a mut dyn HostFrameSinkCallbackServices<B>,
    }
    // Add HostImplGServices<B> to full services; it inherits the narrow callback facet.
    pub trait HostImplGServices<B: HostImplTypes>: HostFrameSinkCallbackServices<B> {
        fn MutatorSetLayerTreeMutator(
            &mut self,
            host: &mut <B as LayerTreeTypes>::MutatorHost,
            mutator: Option<Box<B::LayerTreeMutator>>,
        );
        fn TraceHostImplGEvent(&mut self, category: &'static str, name: &'static str);
        // Source: cc:4447-4450. External paint owns frame-key construction/stringification.
        fn TraceQueueImageDecode(&mut self, image: &B::PaintImage, frame_index: usize);
        fn CreateDrawImageForDecode(
            &mut self,
            image: &B::DrawImage,
            scale_adjustment: f32,
            frame_index: usize,
            color: HostTargetColorParams<B::ColorSpace>,
        ) -> B::DrawImage;
        fn SendExplicitDecodeRequestsImmediately(&self) -> bool;
        fn DelegateNotifyImageDecodeRequestFinished(
            &mut self,
            delegate: HostImplDelegateId,
            request_id: i32,
            speculative: bool,
            succeeded: bool,
        );
        fn MutatorTakeTimeUpdatedEvents(
            &mut self,
            host: &mut <B as LayerTreeTypes>::MutatorHost,
            events: Option<&mut B::HostMutatorEvents>,
        );
        fn DelegateClearHistory(&mut self, delegate: HostImplDelegateId);
        fn DelegateCommitDurationSampleCountForTesting(
            &self,
            delegate: HostImplDelegateId,
        ) -> usize;
        fn ResourcePoolInvalidateResources(&mut self, pool: &mut B::ResourcePool);
        fn RasterContextOrderingBarrier(&mut self, lock: &mut B::ScopedRasterContextLock);
        fn FrameTrackersClearAll(&mut self, trackers: &mut B::FrameSequenceTrackerCollection);
        fn FrameSinkAccess(&mut self, sink: HostImplFrameSinkId) -> HostFrameSinkAccess<'_, B>;
        fn ResourceProviderReleaseAllExportedResources(
            &mut self,
            provider: &mut B::ClientResourceProvider,
            pool: Option<&mut B::ResourcePool>,
            lose: bool,
            client: &mut dyn HostUIResourceReleaseClient<B>,
        );
        // Source factory reads existing settings and draw mode, then serializes the
        // external Viz transport settings; it must not retain a HostImpl reference.
        fn FrameSinkCreateLayerContext(
            &mut self,
            sink: HostImplFrameSinkId,
            target: HostImplWeak,
            draw_mode: HostDrawMode,
            settings: &crate::cc::trees::layer_tree_settings::LayerTreeSettings,
        ) -> Box<<B as HostImplTypes>::LayerContext>;
        fn FrameSinkIdIsValid(
            &self,
            id: &<B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId,
        ) -> bool;
        // Source: cc/resources/resource_pool.h:65 (expiration is 5 seconds).
        fn CreateResourcePool(
            &mut self,
            provider: &mut B::ClientResourceProvider,
            context: Option<HostImplContextProviderId>,
            runner: HostImplTaskRunnerId,
            expiration: TimeDelta,
            disallow_non_exact_reuse: bool,
        ) -> Box<B::ResourcePool>;
        fn DelegateDidObserveFirstScrollDelay(
            &mut self,
            delegate: HostImplDelegateId,
            frame: i32,
            delay: TimeDelta,
            timestamp: TimeTicks,
        );
        fn MutatorImplOnlyAutoScrollAnimationCreate(
            &mut self,
            host: &mut <B as LayerTreeTypes>::MutatorHost,
            id: ElementId,
            target: &PointF,
            current: PointF,
            velocity: f32,
            start_offset: TimeDelta,
        );
        fn MutatorImplOnlyScrollAnimationCreate(
            &mut self,
            host: &mut <B as LayerTreeTypes>::MutatorHost,
            id: ElementId,
            target: PointF,
            current: PointF,
            delayed_by: TimeDelta,
            start_offset: TimeDelta,
        );
        fn UpdateCheckerImageDecodingHints(
            &mut self,
            manager: &mut B::TileManager,
            map: std::collections::BTreeMap<PaintImageId, HostImageDecodingMode>,
        );
        fn RecordAppliedScrollObservation(
            &mut self,
            manager: &mut B::EventsMetricsManager,
            id: ElementId,
        );
        fn NewContentForCheckerboardedScrollsPerFrame(&self) -> bool;
        fn MutatorScrollAnimationAbort(
            &mut self,
            host: &mut <B as LayerTreeTypes>::MutatorHost,
            id: ElementId,
        );
        fn FrameTrackersStartScrollSequence(
            &mut self,
            trackers: &mut B::FrameSequenceTrackerCollection,
            sequence: B::FrameSequenceTrackerType,
            thread: B::SmoothEffectDrivingThread,
        );
        fn FrameTrackersStopSequence(
            &mut self,
            trackers: &mut B::FrameSequenceTrackerCollection,
            sequence: B::FrameSequenceTrackerType,
        );
        fn MutatorTickScrollAnimations(
            &mut self,
            host: &mut <B as LayerTreeTypes>::MutatorHost,
            time: TimeTicks,
            trees: &PropertyTreesFor<B>,
        );
        fn MutatorElementHasImplOnlyScrollAnimation(
            &self,
            host: &<B as LayerTreeTypes>::MutatorHost,
            id: ElementId,
        ) -> bool;
        fn MutatorImplOnlyScrollAnimationUpdateTarget(
            &mut self,
            host: &mut <B as LayerTreeTypes>::MutatorHost,
            delta: Vector2dF,
            max_offset: PointF,
            time: TimeTicks,
            delayed_by: TimeDelta,
            id: ElementId,
        ) -> Option<PointF>;
        fn ChildLocalSurfaceIdAllocatorUpdateFromParent(
            &mut self,
            allocator: &mut B::ChildLocalSurfaceIdAllocator,
            id: &<B as LayerTreeTypes>::LocalSurfaceId,
        );
        fn DelegateReturnResource(
            &mut self,
            delegate: HostImplDelegateId,
            resource: <B as crate::adapter::unbounded::UnboundedTypes>::ReturnedResource,
        );
        fn FrameSinkNotifyNewLocalSurfaceIdExpectedWhilePaused(
            &mut self,
            sink: HostImplFrameSinkId,
        );
    }
}
pub use host_impl_chunk_g::*;

pub trait HostImplScrollbarRegistrationServices<B: HostImplTypes> {
    fn PostDelayedAnimationTaskOnImplThread(
        &mut self,
        delegate: HostImplDelegateId,
        task: crate::adapter::scrollbar_animation::ScrollbarAnimationTask,
        delay: crate::adapter::time::TimeDelta,
    );
    fn TraceScrollbarNeedsAnimate(&self);

    fn PostDelayedAutoscrollTask(
        &mut self,
        owner: HostImplWeak,
        task: super::scrollbar::AutoscrollTask,
        delay: super::time::TimeDelta,
    );
}

mod host_impl_tree_host {
    use super::*;
    // Extra external capabilities required by canonical LayerTreeImpl TreeHost.
    // No source algorithm/state decision is defined here. Merge as a supertrait
    // of HostImplServices<B> after review; all references are live service outputs.
    pub trait HostImplTreeHostServices<B: HostImplTypes> {
        fn SelectionEdgeVisibilityUsesFullEdge(&self) -> bool;
        fn ViewTransitionCaptureAndDisplay(&self) -> bool;
        fn IsCCSlimmingEnabled(&self) -> bool;
        fn VideoFrameControllerClient(
            &self,
            owner: LayerTreeHostId,
        ) -> &B::VideoFrameControllerClient;
        fn TaskRunnerProvider(&self, id: HostImplTaskRunnerProviderId) -> &B::TaskRunnerProvider;
        fn WithLayerTreeFrameSink(
            &self,
            id: HostImplFrameSinkId,
            callback: &mut dyn FnMut(&B::LayerTreeFrameSink),
        );
        fn FrameSinkRasterContext(
            &self,
            id: HostImplFrameSinkId,
        ) -> Option<&B::RasterContextProvider>;
        fn TraceDidActivateLayerTree(&self, owner: LayerTreeHostId, frame: i32);
        fn MissingPropertyNodeForElementId(
            &self,
            mutation: crate::cc::trees::layer_tree_impl::PropertyMutation,
        );
        fn TraceDelegatedInkMetadataPush(&self, metadata: &B::DelegatedInkMetadata);
        fn TraceLayersUpdatedCount(&self, count: usize);
        fn TraceImageInvalidationCounts(
            &self,
            no_images: usize,
            no_invalidation: usize,
            invalidated: usize,
        );

        fn PushMutatorPropertiesTo(
            &mut self,
            state: &B::UnsafeCommitState,
            target: &mut <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::MutatorHost,
            trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
        );
        fn CreateRenderSurface(
            &mut self,
            tree: crate::adapter::layer_tree_impl::LayerTreeId,
            id: crate::adapter::input_types::ElementId,
        ) -> Box<B::RenderSurface>;
        fn ReportUnexpectedOopifPageScaleOnce(&mut self, scale: f32);
    }
}
pub use host_impl_tree_host::*;

/// External environment reads and metric/trace sinks used by canonical
/// cc/input/input_handler.cc. This facet owns no input state, task runner,
/// clock or feature implementation. Each call reads the current environment.
/// Now uses HostImplServices::MonotonicTimeNow; scroll reasons and metric
/// sampling reuse HostImplBServices instead of adding duplicate capabilities.
pub trait HostImplInputEnvironmentServices<B: HostImplTypes> {
    fn InputFeatureEnabled(&self, feature: super::input_handler::InputFeature) -> bool;
    fn InputScrollEventDispatchMode(&self) -> String;
    fn InputLateScrollDeadlineRatio(&self) -> f64;
    fn InputTrackerType(
        &self,
        kind: super::input_handler::ScrollTrackingType,
    ) -> <B as super::input_services::InputHostTypes>::FrameSequenceTrackerType;
    fn InputDrivingThread(
        &self,
        thread: super::input_handler::ScrollingThread,
    ) -> <B as super::input_services::InputHostTypes>::SmoothEffectDrivingThread;
    fn InputHistogramCounts(&self, name: &str, value: f32);
    fn InputHistogramTimes(&self, name: &str, value: super::time::TimeDelta);
    fn InputBeginTrace(
        &self,
        event: &str,
        fields: &[super::input_handler::InputTraceField],
    ) -> <B as super::input_handler::InputHandlerTypes>::TraceSpan;
    fn InputInstantTrace(&self, event: &str, fields: &[super::input_handler::InputTraceField]);
}

pub mod host_impl_chunk_d {
    use super::*;
    // Typed external Viz/metrics values consumed by source HostImpl chunk D.
    // HostImplTypes adds HostImplDTypes; Layer adds HostMetadataLayer<Self>.
    // These traits contain only native field/container/owned-value operations.
    use crate::adapter::{
        geometry::{PointF, Rect, Size, SizeF, Vector2dF},
        layer_tree_impl::{Color4f, LayerTreeTypes},
        property_types::ViewTransitionElementResourceId,
        time::{TimeDelta, TimeTicks},
    };
    use std::collections::BTreeMap;
    pub trait HostImplDTypes:
        crate::adapter::input_services::InputHostTypes<LatencyInfo: HostScrollLatency>
        + LayerTreeTypes<
            CompositorFrameMetadata: HostCompositorFrameMetadata<Self>,
            SelectionBound: PartialEq,
            ViewTransitionRequest: HostMetadataTransitionRequest,
        > + crate::adapter::frame_data::FrameDataTypes<
            RenderPass: HostMetadataPass,
            BeginFrameAck: HostMetadataBeginFrameAck + Clone,
        > + crate::adapter::unbounded::UnboundedTypes<Frame: HostCompositorFrame<Self>>
        + crate::adapter::render_metadata::RenderFrameMetadataTypes<
            TrackedElementRects: HostTrackedElementRects<Self>,
            Selection = crate::adapter::layer_tree_impl::Selection<
                <Self as LayerTreeTypes>::SelectionBound,
            >,
            LocalSurfaceId = <Self as LayerTreeTypes>::LocalSurfaceId,
        >
    {
        type TrackedElementFeature: Copy;
        type TrackedElementRect: HostMetadataTrackedElementRect;
        type RegionCaptureId: Copy;
        type RegionCaptureBounds: HostRegionCaptureBounds<Self>;
        type EventMetricsSet: Default;
        type TransferableResource;
        type TransitionDirective;
        type FrameDeadline;
        type DebugDrawContext<'a>: crate::adapter::debug_rect::DebugRectContext<
            Types = Self,
            Transform = <Self as LayerTreeTypes>::Transform,
            Filters = Self::Filters,
            Path = Self::Path,
            Surface = Self::RenderSurface,
            CopyRequest = Self::CopyOutputRequest,
            Layer = i32,
            RegionBackend = Self::RegionBackend,
        >
        where
            Self: 'a;
        fn MakeDebugDrawContext<'a>(
            state: HostDebugDrawState<'a, Self>,
        ) -> Self::DebugDrawContext<'a>;
    }
    pub trait HostMetadataTrackedElementRect:
        crate::adapter::layer_tree_host_impl::HostImplTrackedElementRect + Clone
    {
        fn should_add_to_compositor_frame_metadata(&self) -> bool;
        fn set_visible_bounds(&mut self, bounds: Rect);
    }
    pub trait HostTrackedElementRects<B: HostImplDTypes>: Default {
        type Features<'a>: Iterator<
            Item = (&'a B::TrackedElementFeature, &'a [B::TrackedElementRect]),
        >
        where
            Self: 'a,
            B: 'a;
        fn is_empty(&self) -> bool;
        fn iter(&self) -> Self::Features<'_>;
        fn Append(&mut self, feature: B::TrackedElementFeature, rect: B::TrackedElementRect);
    }
    pub trait HostRegionCaptureBounds<B: HostImplDTypes>: Default {
        type Bounds<'a>: Iterator<Item = (&'a B::RegionCaptureId, &'a Rect)>
        where
            Self: 'a,
            B: 'a;
        fn bounds(&self) -> Self::Bounds<'_>;
        fn Set(&mut self, id: B::RegionCaptureId, bounds: Rect);
    }
    pub trait HostMetadataLayer<B: HostImplDTypes> {
        fn tracked_element_rects_value(
            &self,
        ) -> Option<
            &<B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects,
        >;
        fn capture_bounds(
            &self,
        ) -> Option<&<B as crate::adapter::layer_tree_host_impl::HostImplDTypes>::RegionCaptureBounds>;
        fn draw_opacity(&self) -> f32;
        fn DidBeginTracing(&mut self);
        // Native selected-subtype chain forwards provider only to the empty
        // LayerImpl base (h152); real cleanup borrows the current PictureContext.
        fn DidDraw(&mut self, context: &mut B::PictureLayerContext<'_>);
    }

    // Source dependency: components/viz/common/quads/compositor_frame_metadata.h.
    // Every operation corresponds to one native member access; default values and
    // payload storage belong to the existing Viz metadata implementation.
    pub trait HostCompositorFrameMetadata<B: HostImplDTypes>:
        Default + HostImplFrameMetadata
    {
        fn frame_token(&self) -> u32;
        fn set_frame_token(&mut self, value: u32);
        fn set_device_scale_factor(&mut self, value: f32);
        fn set_page_scale_factor(&mut self, value: f32);
        fn set_scrollable_viewport_size(&mut self, value: SizeF);
        fn set_visible_viewport_size(&mut self, value: Size);
        fn set_root_background_color(&mut self, value: Color4f);
        fn set_may_throttle_if_undrawn_frames(&mut self, value: bool);
        fn set_is_handling_interaction(&mut self, value: bool);
        fn set_is_handling_animation(&mut self, value: bool);
        fn referenced_surfaces_mut(&mut self) -> &mut Vec<B::SurfaceRange>;
        fn set_min_page_scale_factor(&mut self, value: f32);
        fn set_top_controls_visible_height(&mut self, value: f32);
        fn top_controls_visible_height(&self) -> Option<f32>;
        fn AppendOffsetTagValue(
            &mut self,
            tag: crate::adapter::input_services::OffsetTag,
            value: Vector2dF,
        );
        fn set_root_scroll_offset(&mut self, value: PointF);
        fn set_display_transform_hint(
            &mut self,
            value: crate::adapter::layer_tree_impl::OverlayTransform,
        );
        fn set_is_mobile_optimized(&mut self, value: bool);
        fn delegated_ink_metadata(&self) -> Option<&B::DelegatedInkMetadata>;
        fn set_delegated_ink_metadata(&mut self, value: Box<B::DelegatedInkMetadata>);
        fn set_capture_bounds(
            &mut self,
            value: <B as crate::adapter::layer_tree_host_impl::HostImplDTypes>::RegionCaptureBounds,
        );
        fn set_tracked_element_rects(
            &mut self,
            value: <B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects,
        );
        fn TakeTrackedElementRects(
            &mut self,
        ) -> <B as crate::adapter::render_metadata::RenderFrameMetadataTypes>::TrackedElementRects;
        fn set_screenshot_destination(
            &mut self,
            value: crate::adapter::layer_tree_impl::UnguessableToken,
        );
        fn set_is_software(&mut self, value: bool);
        fn set_has_shared_element_resources(&mut self, value: bool);
        fn transition_directives_mut(&mut self) -> &mut Vec<B::TransitionDirective>;
        fn set_deadline(&mut self, value: B::FrameDeadline);
        fn set_interval_frame_time(&mut self, value: TimeTicks);
        fn set_interval_has_input(&mut self, value: bool);
        fn set_interval_has_user_input(&mut self, value: bool);
        fn set_major_scroll_speed(&mut self, value: f32);
        fn AppendContentInterval(
            &mut self,
            kind: HostContentFrameIntervalType,
            interval: TimeDelta,
            duplicate_count: u32,
        );
        fn set_trees_in_viz_timing_details(&mut self, value: B::TreesInVizTiming);
        fn set_has_only_content_frame_interval_updates(&mut self, value: bool);
        fn set_activation_dependencies(&mut self, value: Vec<B::SurfaceId>);
        fn set_send_frame_token_to_embedder(&mut self, value: bool);
        fn send_frame_token_to_embedder(&self) -> bool;
        fn latency_info(&self) -> &[B::LatencyInfo];
        fn latency_info_mut(&mut self) -> &mut Vec<B::LatencyInfo>;
        fn TakeLatencyInfo(&mut self) -> Vec<B::LatencyInfo>;
        fn set_begin_frame_ack(&mut self, value: B::BeginFrameAck);
        fn begin_frame_ack(&self) -> &B::BeginFrameAck;
        fn set_submit_compositor_frame_time(&mut self, value: TimeTicks);
    }
    // Add these bounds to HostImplDTypes's LayerTreeTypes supertrait:
    // CompositorFrameMetadata:HostCompositorFrameMetadata<Self>,
    // SelectionBound:PartialEq; RenderFrameMetadataTypes::Selection must equal
    // adapter::layer_tree_impl::Selection<Self::SelectionBound>.
    // Add to HostImplTypes::LayerTreeTypes:
    // PresentationCallback=adapter::presentation::Callback<Self::PresentationFeedback>,
    // SuccessfulPresentationCallback=adapter::presentation::SuccessfulCallbackWithDetails<Self::HostFrameTimingDetails>.
    pub trait HostImplDServices<B: HostImplTypes> {
        fn MakeEventMetricsSet(
            &self,
            main: Vec<B::EventMetrics>,
            saved: B::EventMetricsList,
            raster: Vec<B::EventMetrics>,
        ) -> B::EventMetricsSet;
        fn CollectScrollEventsFromFrame(
            &mut self,
            manager: &mut B::AverageLagTrackingManager,
            token: u32,
            events: &B::EventMetricsSet,
        );
        fn FrameSinkSetSourceFrameNumber(&mut self, sink: HostImplFrameSinkId, number: i32);
        fn PostCompositorFrameDump(
            &mut self,
            frame: &B::Frame,
            client: bool,
            viz: bool,
            token: u32,
        );
        fn VerboseLogCompositorFrame(&self, frame: &B::Frame);
        fn FrameSinkExportFrameTiming(&mut self, sink: HostImplFrameSinkId);
        fn TraceSubmitCompositorFrame(&self, trace_id: i64, latency: &[B::LatencyInfo]);
        fn FrameSinkSubmitCompositorFrame(
            &mut self,
            sink: HostImplFrameSinkId,
            frame: B::Frame,
            hit_test_data_changed: bool,
        );
        fn MutatorNextFrameHasPendingRAF(&self, host: &<B as LayerTreeTypes>::MutatorHost) -> bool;
        fn MutatorHasCanvasInvalidation(&self, host: &<B as LayerTreeTypes>::MutatorHost) -> bool;
        fn MutatorHasJSAnimation(&self, host: &<B as LayerTreeTypes>::MutatorHost) -> bool;
        fn MutatorMainThreadAnimationsCount(
            &self,
            host: &<B as LayerTreeTypes>::MutatorHost,
        ) -> usize;
        fn MutatorHasSmilAnimation(&self, host: &<B as LayerTreeTypes>::MutatorHost) -> bool;
        fn StopDrawSequence(
            &mut self,
            trackers: &mut B::FrameSequenceTrackerCollection,
            kind: HostDrawSequence,
        );
        fn DidDrawDevtoolsFrame(&self, id: i32, sequence: u64);
        fn IssueImplThreadRenderingStatsEvent(&mut self, stats: HostImplRenderingStatsId);
        fn FrameSinksToThrottleUpdated(
            &mut self,
            delegate: HostImplDelegateId,
            ids: &std::collections::BTreeSet<
                <B::ThrottleTypes as crate::adapter::throttle::ThrottleTypes>::FrameSinkId,
            >,
        );
        fn NewContentForCheckerboardedScrollsMode(&self) -> HostCheckerboardMode;
        fn TileManagerMemoryStats<'a>(
            &self,
            manager: &'a B::TileManager,
        ) -> &'a crate::cc::resources::memory_history::Entry;
        fn TraceScrollDeltaThisFrame(&self, delta: Vector2dF);
        fn IsNewTrace(&self) -> bool;
        fn TraceFrameViewerSnapshot(&self, id: i32, value: &B::HostTraceValue);
        fn UpdateHudTexture(
            &mut self,
            layer: &mut B::Layer,
            mode: crate::cc::layers::draw_mode::DrawMode,
            sink: HostImplFrameSinkId,
            provider: Option<&mut B::ClientResourceProvider>,
            caps: &crate::cc::trees::raster_capabilities::RasterCapabilities,
            passes: &mut Vec<Box<B::RenderPass>>,
        );
        fn ConstructTransitionDirective(
            &self,
            request: &B::ViewTransitionRequest,
            elements: &BTreeMap<ViewTransitionElementResourceId, B::RenderPassId>,
            colors: &B::DisplayColorSpaces,
            delay: bool,
        ) -> B::TransitionDirective;
        fn MakeFrameDeadline(
            &self,
            time: TimeTicks,
            frames: u32,
            interval: TimeDelta,
            lower_bound: bool,
        ) -> B::FrameDeadline;
        fn DelegatedInkIsHovering(&self, metadata: &B::DelegatedInkMetadata) -> bool;
        fn ObserveRenderFrameSubmission(
            &mut self,
            observer: &mut B::RenderFrameMetadataObserver,
            render_metadata: &crate::cc::trees::render_frame_metadata::RenderFrameMetadata<B>,
            metadata: &mut B::CompositorFrameMetadata,
            force: bool,
        );
        fn AddRendererSwapLatencyTimestamp(&self, latency: &mut B::LatencyInfo, time: TimeTicks);
        fn PrepareSendResourcesToParent(
            &mut self,
            provider: &mut B::ClientResourceProvider,
            resources: &[crate::adapter::layer_tree_impl::ResourceId],
            output: &mut Vec<B::TransferableResource>,
            interface: HostSharedImageInterfaceId,
        );
        fn RecordCompositorFrameQuadCount(&self, client: &str, count: usize);
        fn TraceNoDamageEarlyOut(&self);
        fn FrameSinkSetCurrentLocalSurfaceId(
            &mut self,
            sink: HostImplFrameSinkId,
            id: &<B as LayerTreeTypes>::LocalSurfaceId,
        );
        fn IncrementFrameToken(&self, token: &mut B::FrameTokenGenerator) -> u32;
        fn HasMainThreadAnimation(&self, types: &B::ActiveFrameSequenceTrackers) -> bool;
        fn HasCompositorThreadAnimation(&self, types: &B::ActiveFrameSequenceTrackers) -> bool;
        // Source copies this native external metadata value before setting time.
        fn CloneDelegatedInkMetadata(
            &self,
            metadata: &B::DelegatedInkMetadata,
        ) -> Box<B::DelegatedInkMetadata>;
        fn SetDelegatedInkFrameTime(&self, metadata: &mut B::DelegatedInkMetadata, time: TimeTicks);
        fn TraceDelegatedInkOnCompositorFrame(&self, metadata: &B::DelegatedInkMetadata);
        fn VideoFrameControllerDidDrawFrame(&mut self, controller: HostImplVideoFrameControllerId);
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
        ) -> TimeTicks;
    }

    pub trait HostMetadataPass {
        fn has_transparent_background(&self) -> bool;
        fn quad_resource_id(&self, index: usize) -> crate::adapter::layer_tree_impl::ResourceId;
    }

    // External Viz enum identity: components/viz/common/quads/frame_interval_inputs.h:31-36.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum HostContentFrameIntervalType {
        kVideo,
        kAnimatingImage,
        kScrollBarFadeOutAnimation,
        kCompositorScroll,
    }
    pub trait HostMetadataBeginFrameAck {
        fn frame_id(&self) -> crate::adapter::begin_frame::BeginFrameId;
        fn IsSequenceValid(&self) -> bool;
    }
    pub trait HostCompositorFrame<B: HostImplDTypes>: Default {
        fn metadata(&self) -> &B::CompositorFrameMetadata;
        fn metadata_mut(&mut self) -> &mut B::CompositorFrameMetadata;
        fn set_metadata(&mut self, value: B::CompositorFrameMetadata);
        fn render_passes(&self) -> &[Box<B::RenderPass>];
        fn set_render_passes(&mut self, value: Vec<Box<B::RenderPass>>);
        fn resource_list_mut(&mut self) -> &mut Vec<B::TransferableResource>;
    }
    // Live layer-table borrow only. PropertyTrees is passed separately to the
    // canonical history algorithm; there is no owner/backpointer or capture state.
    pub struct HostDebugDrawState<'a, B: LayerTreeTypes> {
        pub layers: &'a mut B::LayerList,
        pub have_scroll_event_handlers: bool,
    }

    #[derive(Clone, Copy)]
    #[repr(i32)]
    pub enum HostDrawSequence {
        kRAF = 3,
        kCanvasAnimation = 10,
        kJSAnimation = 11,
        kMainThreadAnimation = 1,
        kSETMainThreadAnimation = 12,
    }
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum HostCheckerboardMode {
        kOther,
        kPerScroll,
        kPerFrame,
    }

    pub trait HostMetadataTransitionRequest {
        fn delay_layer_tree_view_deletion(&self) -> bool;
        fn IsAnimateRenderer(&self) -> bool;
    }
}
pub use host_impl_chunk_d::*;

/// The native frame-sink context-provider query, split from mutable transport.
/// No compositor state is implemented or retained by this capability.
pub trait HostColorContextServices {
    fn FrameSinkContextProvider(
        &self,
        sink: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId>;
}
impl<B: HostImplTypes> HostColorContextServices for dyn HostImplServices<B> + '_ {
    fn FrameSinkContextProvider(
        &self,
        sink: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId> {
        HostImplResourceReturnServices::FrameSinkContextProvider(self, sink)
    }
}
