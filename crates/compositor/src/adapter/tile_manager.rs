//! Live typed raster/image capabilities for the canonical TileManager. These
//! interfaces implement no tile policy, scheduling, partitioning or accounting.
//! Owner references are lent per call; independent executor/diagnostic services
//! may be shared, but never retain Page/Host/TileManager/Layer state.
use super::{
    geometry::Rect,
    layer_tree_impl::Color4f,
    picture_layer_tiling::{PictureLayerTilingTypes, TilingRasterSource},
    resource_pool::ResourcePoolTypes,
};
use crate::cc::{
    raster::tile_task::{TileTaskImpl, TileTaskRef},
    resources::resource_pool::InUsePoolResource,
    tiles::{
        tile_manager::RasterCompletion,
        tile_priority::{PriorityBin, WhichTree},
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    hash::Hash,
};

macro_rules! identity {
    ($($name:ident),+ $(,)?) => {$(
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub(crate) struct $name(pub u64);
    )+};
}
identity!(
    ImageControllerId,
    DecodedImageTrackerId,
    CheckerImageTrackerId,
    RasterBufferProviderId,
    RasterQueryQueueId,
    TileManagerClientId,
    ImageDecodeCacheId
);

pub(crate) trait TileManagerPaintImage: Clone {
    type Id: Clone + Eq + Ord + Hash;
    fn stable_id(&self) -> Self::Id;
    fn ShouldAnimate(&self) -> bool;
    fn IsLazyGenerated(&self) -> bool;
    fn IsComplete(&self) -> bool;
    fn IsMultipart(&self) -> bool;
    type WorkletInput: ?Sized;
    fn IsPaintWorklet(&self) -> bool;
    fn GetPaintWorkletInput(&self) -> &Self::WorkletInput;
}
pub(crate) trait TileManagerTargetColorParams<C> {
    fn color_space(&self) -> &C;
    fn GetHdrHeadroom(&self) -> f32;
}
pub(crate) trait TileManagerDrawImage<B: TileManagerTypes>: Clone {
    fn paint_image(&self) -> &B::PaintImage;
    // Exact source DrawImage constructor; no partition/decode decision.
    fn WithRasterParameters(
        &self,
        contents_scale: f32,
        frame_index: usize,
        target: &B::TargetColorParams,
    ) -> Self;
    fn EstimatedDecodedBytes(&self) -> usize;
}
pub(crate) trait TileManagerRasterSource<B: TileManagerTypes>: TilingRasterSource {
    type DisplayItemList: ?Sized;
    fn content_color_usage(&self) -> B::ContentColorUsage;
    fn GetDisplayItemList(&self) -> &Self::DisplayItemList;
    fn PerformSolidColorAnalysis(&self, rect: &Rect, color: &mut Color4f, max_ops: i32) -> bool;
    type DebugName<'a>: std::ops::Deref<Target = str>
    where
        Self: 'a;
    fn debug_name(&self) -> Self::DebugName<'_>;
}
pub(crate) trait TileManagerTypes:
    ResourcePoolTypes
    + PictureLayerTilingTypes<
        Resource = InUsePoolResource<Self>,
        RasterTask = dyn TileTaskImpl<RasterCompletion<Self>>,
        RasterSource: TileManagerRasterSource<Self> + Send + Sync,
        RasterTransform: Send + Sync,
        DrawImage: Sized + TileManagerDrawImage<Self> + Send + Sync + 'static,
        PaintWorkletRecordMap: Sized + Clone,
    >
{
    type PaintImage: TileManagerPaintImage<Id = Self::PaintImageId>;
    type PaintImageId: Clone + Eq + Ord + Hash + Send + Sync + 'static;
    type TargetColorParams: TileManagerTargetColorParams<Self::ColorSpace>;
    type ContentColorUsage: Copy + Eq;
    type RasterBuffer: TileManagerRasterBuffer<Self>;
    type PlaybackImageProvider: RasterImageProvider<Self> + Send;
    type PaintWorkletImageProvider: WorkletImageProvider<Self> + Send;
    type ScopedRasterResult;
    fn TileRasterTaskResourceSource() -> Self::ResourceSource;
    fn HdrContentColorUsage() -> Self::ContentColorUsage;
    fn SrgbContentColorUsage() -> Self::ContentColorUsage;
    fn TestResourceSource() -> Self::ResourceSource;
    // Native test-only SyncToken(gpu::GPU_IO, CommandBufferId(1), 1).
    fn TestRasterSyncToken() -> Self::SyncToken;
    fn IsHDR(color_space: &Self::ColorSpace) -> bool;
}
// Value vocabulary of the external image-checkering API, not its policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DecodeType {
    kRaster,
    kPreDecode,
}
pub(crate) struct DecodeQueueItem<P> {
    pub image: P,
    pub decode_type: DecodeType,
}
pub(crate) type ImageDecodeQueue<B> = Vec<DecodeQueueItem<<B as TileManagerTypes>::PaintImage>>;
pub(crate) type AnimatedImageFrameIndexMap<B> =
    BTreeMap<<B as TileManagerTypes>::PaintImageId, usize>;
#[derive(Clone, Copy)]
pub(crate) struct ImageTracingInfo {
    pub prepare_tiles_id: u64,
    pub priority_bin: PriorityBin,
}

/// External image decode/cache lifetime service. All task ordering and queue
/// selection is performed by TileManager before these calls.
pub(crate) trait ImageController<B: TileManagerTypes> {
    fn SetImageDecodeCache(&mut self, cache: Option<ImageDecodeCacheId>);
    fn ExternalDependencyCompletedForTask(&mut self, dependent: TileTaskRef<RasterCompletion<B>>);
    fn SetPredecodeImages(
        &mut self,
        images: &[B::DrawImage],
        tracing: ImageTracingInfo,
    ) -> Vec<TileTaskRef<RasterCompletion<B>>>;
    fn ReduceMemoryUsage(&mut self);
    fn ConvertImagesToTasks(
        &mut self,
        images: &mut Vec<B::DrawImage>,
        tasks: &mut Vec<TileTaskRef<RasterCompletion<B>>>,
        has_at_raster_images: &mut bool,
        tracing: ImageTracingInfo,
    );
    fn UnrefImages(&mut self, images: &[B::DrawImage]);
    fn FlushDecodeTasksForTesting(&mut self);
    fn cache(&self) -> Option<ImageDecodeCacheId>;
    fn RecordCacheStats(&self);
}
pub(crate) trait DecodedImageTracker<B: TileManagerTypes> {
    fn QueueImageDecode(
        &mut self,
        image: &B::DrawImage,
        callback: Box<dyn FnOnce(bool) + Send>,
        speculative: bool,
    );
    fn UnlockAllImages(&mut self);
    fn SetSyncTreeFrameNumber(&mut self, frame: i32);
    fn OnImagesUsedInDraw(&mut self, images: &[B::DrawImage]);
}
pub(crate) trait CheckerImageTracker<B: TileManagerTypes> {
    fn ClearTracker(&mut self, can_clear_decode_policy_tracking: bool);
    fn SetNoDecodesAllowed(&mut self);
    fn ShouldCheckerImage(&mut self, image: &B::DrawImage, tree: WhichTree) -> bool;
    fn ScheduleImageDecodeQueue(&mut self, queue: ImageDecodeQueue<B>);
    fn SetMaxDecodePriorityAllowed(&mut self, decode_type: DecodeType);
    fn TakeImagesToInvalidateOnSyncTree(&mut self) -> &BTreeSet<B::PaintImageId>;
    fn DidActivateSyncTree(&mut self);
    fn DisallowCheckeringForImage(&mut self, image: &B::PaintImage);
    fn set_force_disabled(&mut self, force_disable: bool);
}

use super::{
    resource_pool::{PoolClockId, ResourcePoolId},
    task_runner::{RepeatingClosure, SequencedTaskRunner},
    tile::{TileId, TileManagerId, TileTraceContext},
    time::{TimeDelta, TimeTicks},
};
use crate::cc::{
    resources::resource_pool::ResourcePool,
    tiles::{
        eviction_tile_priority_queue::EvictionTilePriorityQueue,
        raster_tile_priority_queue::{RasterQueueContext, RasterTilePriorityQueue, Type},
        tile::Tile,
        tile_manager::TileManagerCommand,
        tile_priority::TreePriority,
    },
};
use std::sync::{Arc, Weak};

pub(crate) trait TileManagerTraceScope {}
/// Source image classes remain external image services. Construction preserves
/// source order and arguments, and binds only a command/effect endpoint named
/// by manager identity; the service cannot dereference/call a compositor owner.
/// Only capabilities actually called by cc/tiles/tile_manager.cc:368-413.
/// Construction has no resident-tree access. Runtime mutation/query methods
/// remain on TileManagerRuntime and are never fabricated to satisfy a ctor.
pub(crate) trait TileManagerInitialization<B: TileManagerTypes> {
    fn CreateImageController(
        &mut self,
        origin: Arc<dyn SequencedTaskRunner>,
        worker: Arc<dyn SequencedTaskRunner>,
        owner: TileManagerId,
    ) -> ImageControllerId;
    fn CreateDecodedImageTracker(
        &mut self,
        controller: ImageControllerId,
        origin: Arc<dyn SequencedTaskRunner>,
    ) -> DecodedImageTrackerId;
    fn CreateCheckerImageTracker(
        &mut self,
        controller: ImageControllerId,
        owner: TileManagerId,
        enable_checker_imaging: bool,
        min_image_bytes_to_checker: usize,
    ) -> CheckerImageTrackerId;
    fn HasCurrentDefaultTaskRunner(&self) -> bool;
    fn RegisterDumpProvider(&mut self, owner: TileManagerId, name: &str);
    fn RepeatingCommand(
        &self,
        owner: TileManagerId,
        lifetime: Weak<()>,
        command: TileManagerCommand,
    ) -> RepeatingClosure;
}

/// Only the native image owners used by the manager's image-only source
/// accessors. Construction/activation/cache clearing need no tree/pool facade.
pub(crate) trait TileManagerImages<B: TileManagerTypes> {
    fn ImageController(&mut self, id: ImageControllerId) -> &mut dyn ImageController<B>;
    fn DecodedImageTracker(&mut self, id: DecodedImageTrackerId)
        -> &mut dyn DecodedImageTracker<B>;
    fn CheckerImageTracker(&mut self, id: CheckerImageTrackerId)
        -> &mut dyn CheckerImageTracker<B>;
}

pub(crate) trait TileManagerRuntime<B: TileManagerTypes>:
    TileManagerInitialization<B> + TileManagerImages<B> + RasterQueueContext<B> + TileTraceContext
{
    fn UnregisterDumpProvider(&mut self, owner: TileManagerId);
    fn OnceCommand(
        &self,
        owner: TileManagerId,
        lifetime: Weak<()>,
        command: TileManagerCommand,
    ) -> super::task_runner::OnceClosure;
    fn CheckRasterFinishedQueries(&mut self, id: RasterQueryQueueId) -> bool;
    fn IsResourceReadyToDrawForTile(
        &mut self,
        provider: RasterBufferProviderId,
        tile: TileId,
    ) -> bool;
    // Borrow-only mapping of the source's vector<const InUsePoolResource*>:
    // every identity resolves the current resident tile lease and the actual
    // raster provider receives those references synchronously. No lease or
    // backing is cloned and no readiness/priority algorithm is implemented.
    fn SetReadyToDrawCallbackForTiles(
        &mut self,
        provider: RasterBufferProviderId,
        resources: &[TileId],
        callback: super::task_runner::OnceClosure,
        previous_callback_id: u64,
    ) -> u64;
    fn TaskManagerServices(&self) -> Arc<dyn super::raster_task::TileTaskManagerServices>;
    fn DestroyCheckerImageTracker(&mut self, id: CheckerImageTrackerId);
    fn DestroyDecodedImageTracker(&mut self, id: DecodedImageTrackerId);
    fn DestroyImageController(&mut self, id: ImageControllerId);
    // These two builders only split the currently resident owner fields and
    // forward the exact native query. They run the supplied canonical body
    // exactly once and never retain a borrow or implement iteration/policy.
    fn ImagePartitionParts(
        &mut self,
        source_tiling: super::tile::PictureLayerTilingId,
        tile: TileId,
        enclosing_rect: Rect,
        client: TileManagerClientId,
        checker: CheckerImageTrackerId,
    ) -> ImagePartitionParts<'_, B>;
    fn WithTilesWithResourceIterator<R>(
        &mut self,
        client: TileManagerClientId,
        pool: ResourcePoolId,
        body: impl for<'s> FnOnce(ResourceIterationParts<'s, B>) -> R,
    ) -> R;
    fn GetTargetColorParams(
        &self,
        client: TileManagerClientId,
        usage: B::ContentColorUsage,
    ) -> B::TargetColorParams;
    fn GetTileFormat(
        &self,
        client: TileManagerClientId,
    ) -> super::resource_types::SharedImageFormat;
    fn GraphTrace(&self) -> &dyn crate::cc::tiles::tile_manager::TileManagerGraphTrace;
    fn ShouldRecordSubsampledMetric(&self, sampling_rate: f32) -> bool;
    fn TraceBeginDefault(&self, event: &'static str);
    fn TraceEndWithState(&self, state: impl FnOnce() -> super::property_types::TracedValueJSON);
    fn LogError(&self, message: &'static str);
    fn HistogramBoolean(&self, name: &'static str, value: bool);
    fn HistogramMemoryMB(&self, name: &'static str, value: i64);
    fn TraceBeginOnTrack(&self, event: &'static str, track: TileManagerTraceTrack);
    fn TraceEndOnTrack(&self, track: TileManagerTraceTrack);
    fn TraceInstantOnTrack(
        &self,
        event: &'static str,
        track: TileManagerTraceTrack,
        state: impl FnOnce() -> Option<super::property_types::TracedValueJSON>,
    );
    fn TestSharedImageInterface(&self) -> &dyn super::resource_pool::PoolSharedImageInterface<B>;
    fn TraceScopeWithArgs(
        &self,
        event: &'static str,
        arguments: impl FnOnce() -> Vec<TileManagerTraceArgument>,
    ) -> Box<dyn TileManagerTraceScope>;
    fn AllTilesDoneCommand(
        &self,
        owner: TileManagerId,
        lifetime: Weak<()>,
        start_time: TimeTicks,
    ) -> Box<dyn FnOnce(bool) + Send>;
    fn RasterQueryCapability(
        &self,
        id: RasterQueryQueueId,
    ) -> Arc<dyn crate::cc::tiles::tile_manager::TileManagerRasterQueryQueue>;
    fn RasterFlushCapability(
        &self,
        id: RasterBufferProviderId,
    ) -> Arc<dyn crate::cc::tiles::tile_manager::TileManagerRasterFlush>;
    fn FeatureEnabled(&self, feature: TileManagerFeature) -> bool;
    fn ReclaimDelayInSeconds(&self) -> i64;
    fn Now(&self) -> TimeTicks;
    fn NowWithClock(&self, clock: PoolClockId) -> TimeTicks;
    fn RasterBufferProvider(
        &mut self,
        id: RasterBufferProviderId,
    ) -> &mut dyn RasterBufferProvider<B>;
    /// Borrow-only split of the independent provider and its actual resident
    /// pool. Source acquisition may install a backing on the supplied lease;
    /// the pool receives its original OnBackingAllocated notification.
    fn AcquireRasterBufferForResource(
        &mut self,
        provider: RasterBufferProviderId,
        pool: ResourcePoolId,
        resource: &InUsePoolResource<B>,
        resource_content_id: u64,
        previous_content_id: u64,
    ) -> Option<B::RasterBuffer>;

    fn TileRasterSourceAndPool(
        &mut self,
        tile: TileId,
        source_tiling: super::tile::PictureLayerTilingId,
        pool: ResourcePoolId,
    ) -> (&Tile<B>, &B::RasterSource, &mut ResourcePool<B>);
    fn GetMSAASampleCountForRaster(
        &self,
        client: TileManagerClientId,
        display_list: &<B::RasterSource as TileManagerRasterSource<B>>::DisplayItemList,
    ) -> i32;
    fn WorkerDiagnostics(
        &self,
    ) -> Arc<dyn crate::cc::tiles::tile_manager::TileManagerWorkerDiagnostics>;
    fn CreatePlaybackImageProvider(
        &mut self,
        cache: Option<ImageDecodeCacheId>,
        target: &B::TargetColorParams,
        settings: PlaybackImageProviderSettings<B>,
    ) -> B::PlaybackImageProvider;
    fn CreatePaintWorkletImageProvider(
        &mut self,
        records: B::PaintWorkletRecordMap,
    ) -> B::PaintWorkletImageProvider;
    fn TileAndPool(
        &mut self,
        tile: TileId,
        pool: ResourcePoolId,
    ) -> (&mut Tile<B>, &mut ResourcePool<B>);
    // Source client methods are forwarded synchronously. ById variants only
    // resolve a current resident pointer, retaining source call/read ordering.
    fn NotifyTileStateChanged(
        &mut self,
        client: TileManagerClientId,
        tile: &Tile<B>,
        update_damage: bool,
        set_needs_redraw: bool,
    );
    fn NotifyTileStateChangedById(
        &mut self,
        client: TileManagerClientId,
        tile: TileId,
        update_damage: bool,
        set_needs_redraw: bool,
    );
    fn NotifyReadyToActivate(&mut self, client: TileManagerClientId);
    fn NotifyReadyToDraw(&mut self, client: TileManagerClientId);
    fn NotifyAllTileTasksCompleted(&mut self, client: TileManagerClientId);
    fn HasPendingTree(&mut self, client: TileManagerClientId) -> bool;
    fn SetNeedsRedraw(
        &mut self,
        client: TileManagerClientId,
        animation_only: bool,
        skip_if_inside_draw: bool,
    );
    fn SetIsLikelyToRequireADraw(&mut self, client: TileManagerClientId, value: bool);
    fn RequestImplSideInvalidationForCheckerImagedTiles(&mut self, client: TileManagerClientId);
    fn ResourcePool(&self, id: ResourcePoolId) -> &ResourcePool<B>;
    fn ResourcePoolMutable(&mut self, id: ResourcePoolId) -> &mut ResourcePool<B>;
    fn TileByIdMutable(&mut self, id: TileId) -> &mut Tile<B>;
    // Exact canonical client calls, forwarded synchronously by the live owner.
    fn BuildRasterQueue(
        &mut self,
        client: TileManagerClientId,
        tree_priority: TreePriority,
        kind: Type,
    ) -> Box<dyn RasterTilePriorityQueue<B>>;
    fn BuildEvictionQueue(
        &mut self,
        client: TileManagerClientId,
    ) -> Box<EvictionTilePriorityQueue<B>>;
    fn TraceScope(&self, event: &'static str) -> Box<dyn TileManagerTraceScope>;
    fn TraceInstant(
        &self,
        event: &'static str,
        state: impl FnOnce() -> Option<super::property_types::TracedValueJSON>,
    );
    fn HistogramDuration(
        &self,
        name: &'static str,
        value: TimeDelta,
        min: TimeDelta,
        max: TimeDelta,
        buckets: usize,
    );
}

// Raster execution/resource status only: compositor performs all decisions
// around these primitives, including required flags and signal accounting.
pub(crate) trait RasterBufferProvider<B: TileManagerTypes> {
    fn Shutdown(&mut self);
    fn NotifyWorkSubmitted(&mut self);
    fn IsResourceReadyToDraw(&self, resource: &InUsePoolResource<B>) -> bool;
    fn CanPartialRasterIntoProvidedResource(&self) -> bool;
    fn AcquireBufferForRaster(
        &mut self,
        resource: &InUsePoolResource<B>,
        pool: &mut ResourcePool<B>,
        resource_content_id: u64,
        previous_content_id: u64,
    ) -> Option<B::RasterBuffer>;
}

#[derive(Clone, Copy)]
pub(crate) enum TileManagerFeature {
    ReclaimPrepaintTilesWhenIdle,
    ReclaimOldPrepaintTiles,
    TileOOMFreezeMitigation,
}

pub(crate) trait TileManagerImageFrameClient<B: TileManagerTypes> {
    fn GetFrameIndexForImage(&self, image: &B::PaintImage, tree: WhichTree) -> usize;
}
pub(crate) struct ImagePartitionParts<'a, B: TileManagerTypes> {
    pub tile: &'a Tile<B>,
    pub tile_tiling: &'a crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
    pub images: Vec<&'a B::DrawImage>,
    pub client: &'a dyn TileManagerImageFrameClient<B>,
    pub checker: &'a mut dyn CheckerImageTracker<B>,
}
pub trait ResourceTileIterator<B: TileManagerTypes> {
    fn AtEnd(&self) -> bool;
    fn GetCurrent(&mut self) -> Option<&mut Tile<B>>;
    fn Next(&mut self);
    fn IsCurrentTileOccluded(
        &mut self,
        queries: &dyn super::picture_layer_tiling::PictureLayerTilingQueries<B>,
    ) -> bool;
}
/// A synchronous traversal of actual resident tiles. The leaf and its tiling
/// state are disjoint borrows; no cursor or owner state is copied or retained.
pub trait ResourceTileTraversal<B: TileManagerTypes> {
    fn ForEachResourceTile(
        &mut self,
        callback: &mut dyn FnMut(
            &mut Tile<B>,
            &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
        ) -> std::ops::ControlFlow<()>,
    ) -> std::ops::ControlFlow<()>;
}
pub(crate) trait TileStateNotification<B: TileManagerTypes> {
    fn NotifyTileStateChanged(
        &mut self,
        tile: &Tile<B>,
        update_damage: bool,
        set_needs_redraw: bool,
    );
}
/// One source tile, its live occlusion result, and the call-scoped owner
/// continuation travel together. A separate client reference would alias the
/// Picture/Set already lent to the traversal; no owner is recovered by id.
pub(crate) trait ResourceTileReleaseTraversal<B: TileManagerTypes> {
    fn ForEachResourceTile(
        &mut self,
        callback: &mut dyn FnMut(
            &mut Tile<B>,
            bool,
            &mut dyn TileStateNotification<B>,
        ) -> std::ops::ControlFlow<()>,
    ) -> std::ops::ControlFlow<()>;
}
pub(crate) struct ResourceIterationParts<'a, B: TileManagerTypes> {
    pub traversal: Box<dyn ResourceTileReleaseTraversal<B> + 'a>,
    pub pool: &'a mut ResourcePool<B>,
}

pub(crate) trait RasterImageProvider<B: TileManagerTypes> {
    fn GetRasterContent(&mut self, image: &B::DrawImage) -> B::ScopedRasterResult;
}
pub(crate) trait WorkletImageProvider<B: TileManagerTypes> {
    fn GetPaintRecordResult(
        &mut self,
        input: &<B::PaintImage as TileManagerPaintImage>::WorkletInput,
    ) -> B::ScopedRasterResult;
}
#[derive(Clone, Copy, Default)]
pub(crate) struct RasterPlaybackSettings {
    pub use_lcd_text: bool,
    pub msaa_sample_count: i32,
    pub visible: bool,
    pub hdr_headroom: f32,
}
pub(crate) struct RasterPlaybackSettingsView<'a, B: TileManagerTypes> {
    pub values: &'a RasterPlaybackSettings,
    pub image_provider: &'a mut dyn RasterImageProvider<B>,
    pub raster_inducing_scroll_offsets: &'a super::prioritized_tile::ScrollOffsetMap,
}
pub(crate) struct RasterPlaybackRequest<'a, B: TileManagerTypes> {
    pub source: &'a B::RasterSource,
    pub content_rect: &'a Rect,
    pub invalid_content_rect: &'a Rect,
    pub new_content_id: u64,
    pub raster_transform: &'a B::RasterTransform,
    pub settings: RasterPlaybackSettingsView<'a, B>,
    pub url: &'a str,
}
pub(crate) trait TileManagerRasterBuffer<B: TileManagerTypes>: Send {
    fn SupportsBackgroundThreadPriority(&self) -> bool;
    fn Playback(&mut self, request: RasterPlaybackRequest<'_, B>);
}
pub(crate) struct PlaybackImageProviderSettings<B: TileManagerTypes> {
    pub images_to_skip: BTreeSet<B::PaintImageId>,
    pub image_to_current_frame_index: Arc<AnimatedImageFrameIndexMap<B>>,
    pub use_gpu_raster_mode: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TileManagerTraceTrack {
    pub name: &'static str,
    pub owner: TileManagerId,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MemoryDumpLevel {
    Background,
    Light,
    Detailed,
}
pub(crate) trait TileManagerMemoryDump<N> {
    fn CreateAllocatorDump(&mut self, path: &str) -> N;
    fn AddString(&mut self, node: &N, name: &str, units: &str, value: &str);
    fn AddScalar(&mut self, node: &N, name: &str, units: &str, value: u64);
}

// Native TRACE_EVENT evaluates its data only when the category is enabled.
// Every lazy trace factory above must invoke its FnOnce at most once, only for
// an enabled category, synchronously within the current call. It cannot retain
// closures or owner borrows; trace-disabled paths create no trace dictionaries.
pub(crate) enum TileManagerTraceArgument {
    Unsigned(&'static str, u64),
    String(&'static str, &'static str),
}

/// Source TileManager::Release only needs these resident/resource callbacks.
/// This call-scoped subset deliberately does not require a raster-queue context
/// or retain any host, tree, tile or pool owner.
pub(crate) trait TileReleaseRuntime<B: TileManagerTypes>: TileTraceContext {
    fn ResourcePoolMutable(&mut self, id: ResourcePoolId) -> &mut ResourcePool<B>;
    fn NotifyTileStateChanged(
        &mut self,
        client: TileManagerClientId,
        tile: &Tile<B>,
        update_damage: bool,
        set_needs_redraw: bool,
    );
}
