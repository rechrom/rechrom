//! External native allocations supplied to HostImpl. The wrappers expose only
//! the precise native capability; none owns a compositor tree, Page or policy.
use super::resources::{CpuReturnedResource, CpuSyncToken, CpuTransferableResource};
use crate::adapter::{
    layer_tree_host_impl::{
        HostImplImageDecodeCache, HostImplResourceTypes, HostRasterContextLock,
        HostSharedImageInterfaceId, HostUIResourceProvider, HostUIResourceReleaseClient,
        HostUIResourceReleaseTask, HostUIResourceTypes,
    },
    layer_tree_impl::{ResourceId, UIResourceId},
};

pub trait NativeRasterContextProvider: Send + Sync {
    fn Lock(&self) -> Box<dyn HostRasterContextLock>;
}
pub type RasterContextProvider = std::sync::Arc<dyn NativeRasterContextProvider>;
pub struct ImageDecodeCache(pub Box<dyn HostImplImageDecodeCache>);
impl HostImplImageDecodeCache for ImageDecodeCache {
    fn ImageDecodeCacheIdentity(&self) -> crate::adapter::tile_manager::ImageDecodeCacheId {
        self.0.ImageDecodeCacheIdentity()
    }
    fn ClearCache(&mut self) {
        self.0.ClearCache()
    }
    fn SetShouldAggressivelyFreeResources(&mut self, value: bool) {
        self.0.SetShouldAggressivelyFreeResources(value)
    }
}
/// Native Viz client resource provider; the actual provider retains its import
/// and export bookkeeping. Release callbacks are always passed the current
/// call's live release client and never capture the host.
/// A call-scoped callback target for a resource returned to its actual pool.
/// Providers store only the source weak identity, never this borrowed client.
pub trait NativePoolReleaseClient {
    fn OnPoolResourceReleased(
        &mut self,
        callback: crate::adapter::resource_pool::ResourcePoolReleaseCallback,
        token: &CpuSyncToken,
        lost: bool,
    );
}
pub trait NativeClientResourceProvider<
    B: HostUIResourceTypes<UITransferableResource = CpuTransferableResource>
        + HostImplResourceTypes<ResourceId = ResourceId>,
>: HostUIResourceProvider<B>
{
    fn CreateScopedBatchResourcesRelease(
        &mut self,
        enabled: bool,
    ) -> Box<dyn crate::adapter::client_layer_tree_host_impl::ClientBatchResourceRelease>;

    fn SetVisible(&mut self, visible: bool);
    fn SetEvicted(&mut self, evicted: bool);
    fn ShutdownAndReleaseAllResources(&mut self, client: &mut dyn HostUIResourceReleaseClient<B>);
    fn ReceiveReturnsFromParent(
        &mut self,
        resources: Vec<CpuReturnedResource>,
        pool: Option<&mut dyn NativePoolReleaseClient>,
        client: &mut dyn HostUIResourceReleaseClient<B>,
    );
    fn ReleaseAllExportedResources(
        &mut self,
        pool: Option<&mut dyn NativePoolReleaseClient>,
        lose: bool,
        client: &mut dyn HostUIResourceReleaseClient<B>,
    );
    fn PrepareSendToParent(
        &mut self,
        resources: &[ResourceId],
        output: &mut Vec<CpuTransferableResource>,
        interface: HostSharedImageInterfaceId,
    );
}
pub struct ClientResourceProvider<
    B: HostUIResourceTypes<UITransferableResource = CpuTransferableResource>
        + HostImplResourceTypes<ResourceId = ResourceId>,
>(pub Box<dyn NativeClientResourceProvider<B>>);
impl<
        B: HostUIResourceTypes<UITransferableResource = CpuTransferableResource>
            + HostImplResourceTypes<ResourceId = ResourceId>,
    > HostUIResourceProvider<B> for ClientResourceProvider<B>
{
    fn ImportResource(
        &mut self,
        resource: CpuTransferableResource,
        release: HostUIResourceReleaseTask,
    ) -> ResourceId {
        self.0.ImportResource(resource, release)
    }
    fn RemoveImportedResource(
        &mut self,
        id: ResourceId,
        client: &mut dyn HostUIResourceReleaseClient<B>,
    ) {
        self.0.RemoveImportedResource(id, client)
    }
}
/// The external metrics FrameSorter remains a real instance. Canonical host
/// calls Reset in its source position rather than maintaining duplicate state.
pub trait NativeFrameSorter {
    fn Reset(&mut self, reset_first_contentful_paint: bool);
    fn FirstContentfulPaintReceived(&self) -> bool;
    fn OnFirstContentfulPaintReceived(&mut self);
}
pub struct FrameSorter(pub Box<dyn NativeFrameSorter>);
impl FrameSorter {
    pub fn FirstContentfulPaintReceived(&self) -> bool {
        self.0.FirstContentfulPaintReceived()
    }
    pub fn OnFirstContentfulPaintReceived(&mut self) {
        self.0.OnFirstContentfulPaintReceived()
    }
    pub fn Reset(&mut self, reset_first_contentful_paint: bool) {
        self.0.Reset(reset_first_contentful_paint)
    }
}

// Source: cc/resources/ui_resource_request.h:17-56. The bitmap is a shared
// immutable native allocation. The producer supplies the native empty bitmap
// for a DELETE request, preserving its source field/getter shape.
pub struct UIResourceRequest {
    pub kind: crate::adapter::layer_tree_impl::UIResourceRequestType,
    pub id: UIResourceId,
    pub bitmap: super::host_resources::UIBitmap,
}
impl<
        B: crate::adapter::layer_tree_impl::LayerTreeTypes<
            UIResourceBitmap = super::host_resources::UIBitmap,
        >,
    > crate::adapter::layer_tree_impl::LayerTreeUIResourceRequest<B> for UIResourceRequest
{
    fn GetType(&self) -> crate::adapter::layer_tree_impl::UIResourceRequestType {
        self.kind
    }
    fn GetId(&self) -> UIResourceId {
        self.id
    }
    fn GetBitmap(&self) -> &super::host_resources::UIBitmap {
        &self.bitmap
    }
}
/// No-op scopes are not provided: a native sequence checker must be supplied
/// by the Page's executor and validates the actual calling sequence.
pub struct SequenceChecker(pub Box<dyn crate::adapter::presentation::SequenceChecker>);
impl crate::adapter::presentation::SequenceChecker for SequenceChecker {
    fn Check(&self) {
        self.0.Check()
    }
}

/// Source non-owning task-runner provider reads the injected executors. They
/// are independent resources; this constructor creates no thread or clock.
pub struct TaskRunnerProvider {
    pub main_thread: std::sync::Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>,
    pub impl_thread: Option<std::sync::Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>>,
}
impl TaskRunnerProvider {
    pub fn new(
        main_thread: std::sync::Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>,
        impl_thread: Option<std::sync::Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>>,
    ) -> Self {
        Self {
            main_thread,
            impl_thread,
        }
    }
}
pub struct SwapPromise<R: super::types::CpuSkiaBackend>(
    pub  Box<
        dyn crate::cc::trees::swap_promise::SwapPromise<
            super::metadata::CompositorFrameMetadata<super::types::CpuCompositorTypes<R>>,
        >,
    >,
);
impl<R: super::types::CpuSkiaBackend> crate::adapter::commit_types::SwapPromise for SwapPromise<R> {}
impl<R: super::types::CpuSkiaBackend>
    crate::cc::trees::swap_promise::SwapPromise<
        super::metadata::CompositorFrameMetadata<super::types::CpuCompositorTypes<R>>,
    > for SwapPromise<R>
{
    fn DidActivate(&mut self) {
        self.0.DidActivate()
    }
    fn WillSwap(
        &mut self,
        metadata: &mut super::metadata::CompositorFrameMetadata<
            super::types::CpuCompositorTypes<R>,
        >,
    ) {
        self.0.WillSwap(metadata)
    }
    fn DidSwap(&mut self) {
        self.0.DidSwap()
    }
    fn DidNotSwap(
        &mut self,
        reason: crate::cc::trees::swap_promise::DidNotSwapReason,
        time: crate::adapter::time::TimeTicks,
    ) -> crate::cc::trees::swap_promise::DidNotSwapAction {
        self.0.DidNotSwap(reason, time)
    }
    fn OnCommit(&mut self) {
        self.0.OnCommit()
    }
    fn GetTraceId(&self) -> i64 {
        self.0.GetTraceId()
    }
}
/// The actual native request owns its callback and capture parameters. Calls
/// only expose the exact request API consumed by the canonical compositor.
pub trait NativeViewTransitionRequest:
    crate::adapter::layer_tree_impl::LayerTreeViewTransitionRequest
    + crate::adapter::layer_tree_host_impl::HostMetadataTransitionRequest
{
    // Source: cc/view_transition/view_transition_request.h:35-36,70-72,89-91.
    fn TakeFinishedCallback(&mut self) -> Option<ViewTransitionCaptureCallback>;
    fn maybe_cross_frame_sink(&self) -> bool;
    fn SetContentRect(
        &self,
        id: crate::adapter::property_types::ViewTransitionElementResourceId,
        rect: crate::adapter::geometry::RectF,
    );
    fn ConstructDirective(
        &self,
        elements: &std::collections::BTreeMap<
            crate::adapter::property_types::ViewTransitionElementResourceId,
            super::frame::RenderPassId,
        >,
        colors: &super::resources::CpuDisplayColorSpaces,
        delay: bool,
    ) -> super::metadata::TransitionDirective;
}
pub struct ViewTransitionRequest(pub Box<dyn NativeViewTransitionRequest + Send + Sync>);
impl ViewTransitionRequest {
    pub(crate) fn ConstructDirective(
        &self,
        elements: &std::collections::BTreeMap<
            crate::adapter::property_types::ViewTransitionElementResourceId,
            super::frame::RenderPassId,
        >,
        colors: &super::resources::CpuDisplayColorSpaces,
        delay: bool,
    ) -> super::metadata::TransitionDirective {
        self.0.ConstructDirective(elements, colors, delay)
    }
}
pub type ViewTransitionCaptureCallback = Box<dyn FnOnce(&crate::adapter::layer_tree_host_impl::host_impl_chunk_c::HostViewTransitionElementResourceRects) + Send + Sync>;
impl crate::adapter::layer_tree_impl::LayerTreeViewTransitionRequest for ViewTransitionRequest {
    fn IsSave(&self) -> bool {
        self.0.IsSave()
    }
    fn token(&self) -> crate::adapter::draw_property::ViewTransitionToken {
        self.0.token()
    }
    fn sequence_id(&self) -> u32 {
        self.0.sequence_id()
    }
}
impl crate::adapter::layer_tree_host_impl::HostMetadataTransitionRequest for ViewTransitionRequest {
    fn delay_layer_tree_view_deletion(&self) -> bool {
        self.0.delay_layer_tree_view_deletion()
    }
    fn IsAnimateRenderer(&self) -> bool {
        self.0.IsAnimateRenderer()
    }
}
pub struct ColorSpaceCache(
    pub  Box<
        dyn crate::adapter::layer_tree_host_impl::HostColorSpaceCache<
            super::resources::CpuColorSpace,
        >,
    >,
);
impl crate::adapter::layer_tree_host_impl::HostColorSpaceCache<super::resources::CpuColorSpace>
    for ColorSpaceCache
{
    fn Get(&mut self, key: &super::resources::CpuColorSpace) -> Option<&bool> {
        self.0.Get(key)
    }
    fn Put(&mut self, key: super::resources::CpuColorSpace, value: bool) {
        self.0.Put(key, value)
    }
}

/// Source VideoFrameControllerClient is the resident HostImpl interface. This
/// non-owning result is its checked owner identity, not a callable object that
/// could retain HostImpl. Add/remove are issued with the live HostImpl context
/// by the external video integration; the current CPU layer family has no video
/// subtype or fabricated controller instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VideoFrameControllerClient {
    pub owner: crate::adapter::layer_tree_impl::LayerTreeHostId,
}

pub trait NativeChildLocalSurfaceIdAllocator:
    crate::adapter::layer_tree_host_impl::HostChildLocalSurfaceIdAllocator
{
    fn CurrentLocalSurfaceId(&self) -> &super::types::LocalSurfaceId;
    fn GetCurrentLocalSurfaceId(&self) -> super::types::LocalSurfaceId;
    fn UpdateFromParent(&mut self, id: &super::types::LocalSurfaceId);
}
pub struct ChildLocalSurfaceIdAllocator(pub Box<dyn NativeChildLocalSurfaceIdAllocator>);
impl crate::adapter::layer_tree_host_impl::HostChildLocalSurfaceIdAllocator
    for ChildLocalSurfaceIdAllocator
{
    fn GenerateId(&mut self) {
        self.0.GenerateId()
    }
}
pub trait NativeAverageLagTrackingManager {
    fn Clear(&mut self);
    fn DidPresentCompositorFrame(
        &mut self,
        token: u32,
        details: &super::metadata::FrameTimingDetails,
    );
    fn CollectScrollEventsFromFrame(&mut self, token: u32, events: &super::input::EventMetricsSet);
}
pub struct AverageLagTrackingManager(pub Box<dyn NativeAverageLagTrackingManager>);
impl AverageLagTrackingManager {
    pub(crate) fn CollectScrollEventsFromFrame(
        &mut self,
        token: u32,
        events: &super::input::EventMetricsSet,
    ) {
        self.0.CollectScrollEventsFromFrame(token, events)
    }
}
pub trait NativeEventsMetricsManager {
    fn TakeSavedEventsMetrics(&mut self) -> Vec<super::input::EventMetrics>;
    fn SaveActiveEventMetrics(&mut self);
    fn SetDidScroll(&mut self, scrolled: bool);
    fn DropSavedEventMetricsForNoFrameUpdate(&mut self);
    fn DidApplyScroll(&mut self, id: crate::adapter::input_types::ElementId);
    fn GetScopedMonitor(
        &mut self,
        callback: super::input::EventMetricsDoneCallback,
    ) -> Box<dyn super::input::EventMetricsScopedMonitor>;
}
pub struct EventsMetricsManager(pub Box<dyn NativeEventsMetricsManager>);
/// Type-only throttle configuration. A native frame is borrowed separately by
/// BorrowedFrameThrottle; this marker has no `pass()` method or fake pass owner.
pub struct ThrottleTypes<R: super::types::CpuSkiaBackend>(std::marker::PhantomData<fn() -> R>);
impl<R: super::types::CpuSkiaBackend> crate::adapter::throttle::ThrottleTypes for ThrottleTypes<R> {
    type Transform = super::geometry::Transform<R>;
    type PassId = super::frame::RenderPassId;
    type FrameSinkId = super::types::FrameSinkId;
    type PassHandle = crate::adapter::layer_tree_host_impl::HostRenderPassLocation;
    type Pass = super::frame::CompositorRenderPass<super::types::CpuCompositorTypes<R>>;
}

/// Native SkColorType enum value returned by the external format conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkColorType(pub u32);
pub trait NativeDarkModeFilter: Send + Sync {
    fn ApplyToColor(
        &self,
        color: crate::adapter::layer_tree_impl::Color4f,
    ) -> crate::adapter::layer_tree_impl::Color4f;
}
pub trait NativeImageDecodeFactory {
    fn ToClosestSkColorType(
        &self,
        format: crate::adapter::resource_types::SharedImageFormat,
    ) -> SkColorType;
    fn CreateGpuImageDecodeCache(
        &self,
        provider: Option<RasterContextProvider>,
        color: SkColorType,
        budget: usize,
        max_texture_size: i32,
        filter: Option<&dyn NativeDarkModeFilter>,
    ) -> Box<ImageDecodeCache>;
    fn CreateSoftwareImageDecodeCache(
        &self,
        color: SkColorType,
        budget: usize,
    ) -> Box<ImageDecodeCache>;
}
pub struct ImageDecodeFactory(pub Box<dyn NativeImageDecodeFactory>);
impl crate::adapter::layer_tree_host_impl::HostImplImageDecodeFactory for ImageDecodeFactory {
    type ImageDecodeCache = ImageDecodeCache;
    // Source: cc/trees/layer_tree_host_impl.cc:426-440. The worker
    // provider is nullable and remains unused in the software cache branch.
    type RasterContextProvider = Option<RasterContextProvider>;
    type DarkModeFilter = dyn NativeDarkModeFilter;
    type ColorType = SkColorType;
    fn ToClosestSkColorType(
        &self,
        format: crate::adapter::resource_types::SharedImageFormat,
    ) -> SkColorType {
        self.0.ToClosestSkColorType(format)
    }
    fn CreateGpuImageDecodeCache(
        &self,
        provider: Option<RasterContextProvider>,
        color: SkColorType,
        budget: usize,
        max_texture_size: i32,
        filter: Option<&dyn NativeDarkModeFilter>,
    ) -> Box<ImageDecodeCache> {
        self.0
            .CreateGpuImageDecodeCache(provider, color, budget, max_texture_size, filter)
    }
    fn CreateSoftwareImageDecodeCache(
        &self,
        color: SkColorType,
        budget: usize,
    ) -> Box<ImageDecodeCache> {
        self.0.CreateSoftwareImageDecodeCache(color, budget)
    }
}
pub struct RasterContextLock(pub Box<dyn HostRasterContextLock>);
impl HostRasterContextLock for RasterContextLock {
    fn max_texture_size(&self) -> i32 {
        self.0.max_texture_size()
    }
    fn BestSupportedTextureFormat(&self) -> crate::adapter::resource_types::SharedImageFormat {
        self.0.BestSupportedTextureFormat()
    }
    fn supports_scanout_shared_images(&self) -> bool {
        self.0.supports_scanout_shared_images()
    }
    fn FeatureStatus(
        &self,
        feature: crate::adapter::layer_tree_host_impl::HostGpuFeatureType,
    ) -> crate::adapter::layer_tree_host_impl::HostGpuFeatureStatus {
        self.0.FeatureStatus(feature)
    }
    fn msaa_is_slow(&self) -> bool {
        self.0.msaa_is_slow()
    }
    fn avoid_stencil_buffers(&self) -> bool {
        self.0.avoid_stencil_buffers()
    }
    fn BestSupportedRenderbufferFormat(&self) -> crate::adapter::resource_types::SharedImageFormat {
        self.0.BestSupportedRenderbufferFormat()
    }
}
pub trait NativeRenderFrameMetadataObserver<R: super::types::CpuSkiaBackend> {
    fn BindToCurrentSequence(&mut self);
    fn DidEndScroll(&mut self);
    fn ReportScrollJankStats(&mut self, total: u32, janky: u32);
    fn ObserveRenderFrameSubmission(
        &mut self,
        metadata: &crate::cc::trees::render_frame_metadata::RenderFrameMetadata<
            super::types::CpuCompositorTypes<R>,
        >,
        frame: &mut super::metadata::CompositorFrameMetadata<super::types::CpuCompositorTypes<R>>,
        force: bool,
    );
}
pub struct RenderFrameMetadataObserver<R: super::types::CpuSkiaBackend>(
    pub Box<dyn NativeRenderFrameMetadataObserver<R>>,
);
impl<R: super::types::CpuSkiaBackend> RenderFrameMetadataObserver<R> {
    pub(crate) fn ObserveRenderFrameSubmission(
        &mut self,
        metadata: &crate::cc::trees::render_frame_metadata::RenderFrameMetadata<
            super::types::CpuCompositorTypes<R>,
        >,
        frame: &mut super::metadata::CompositorFrameMetadata<super::types::CpuCompositorTypes<R>>,
        force: bool,
    ) {
        self.0.ObserveRenderFrameSubmission(metadata, frame, force)
    }
}

/// External scoped GPU visibility owner. Its native destructor restores the
/// visibility state; the compositor neither caches that state nor emulates it.
pub trait NativeContextVisibility {}
pub struct ContextVisibility(pub Box<dyn NativeContextVisibility>);
/// Animation events stay in their native animation subsystem allocation.
/// The host only tests emptiness and transfers the same owner to the mutator.
pub trait NativeMutatorEvents {
    fn IsEmpty(&self) -> bool;
    fn WithScrollAnimationEvents(
        &mut self,
        callback: &mut dyn FnMut(
            &mut crate::cc::animation::animation_events::AnimationEvents<
                crate::cc::animation::scroll_offset_animation_curve::ScrollOffsetAnimationCurve,
            >,
        ),
    );
}
pub struct MutatorEvents(pub Box<dyn NativeMutatorEvents>);
impl MutatorEvents {
    pub(crate) fn WithScrollAnimationEvents(
        &mut self,
        callback: &mut dyn FnMut(
            &mut crate::cc::animation::animation_events::AnimationEvents<
                crate::cc::animation::scroll_offset_animation_curve::ScrollOffsetAnimationCurve,
            >,
        ),
    ) {
        self.0.WithScrollAnimationEvents(callback)
    }
    pub(crate) fn IsEmpty(&self) -> bool {
        self.0.IsEmpty()
    }
}
/// Native animation-worklet mutator. HostImpl transfers this owner to the
/// animation subsystem; no animation state or mutation algorithm is copied.
pub trait NativeLayerTreeMutator {
    fn HasMutators(&mut self) -> bool;
}
pub struct LayerTreeMutator(pub Box<dyn NativeLayerTreeMutator>);
/// HostImpl itself only owns/transfers the painter. Dispatch is performed by
/// the external PaintWorklet subsystem, on this same native allocation.
pub trait NativePaintWorkletLayerPainter<R: super::types::CpuSkiaBackend = super::skia::SkRegion> {
    fn DispatchWorklets(
        &mut self,
        jobs: crate::adapter::client_layer_tree_host_impl::PaintWorkletJobMap<
            super::types::CpuCompositorTypes<R>,
        >,
        owner: crate::adapter::layer_tree_host_impl::HostImplWeak,
    ) -> Option<
        crate::adapter::client_layer_tree_host_impl::PaintWorkletJobMap<
            super::types::CpuCompositorTypes<R>,
        >,
    >;
    fn HasOngoingDispatch(&self) -> bool;
}
pub struct PaintWorkletLayerPainter<R: super::types::CpuSkiaBackend = super::skia::SkRegion>(
    pub Box<dyn NativePaintWorkletLayerPainter<R>>,
);
/// Native Viz transport, separate from the source-owned unbounded handler.
/// Every operation is forwarded on the injected transport owner.
pub trait NativeLayerContext {
    fn SetUnboundedFrameSinkId(
        &mut self,
        sink: &super::types::FrameSinkId,
        local: &super::types::LocalSurfaceId,
    );
    fn DismissUnboundedFrameSink(&mut self);
    fn SetUnboundedLocalSurfaceId(&mut self, local: &super::types::LocalSurfaceId);
    fn SetVisible(&mut self, visible: bool);
    fn Flush(&mut self);
    fn SetTargetLocalSurfaceId(&mut self, local: &super::types::LocalSurfaceId);
}
pub struct LayerContext(pub Box<dyn NativeLayerContext>);
impl<B> crate::adapter::layer_tree_host_impl::HostUnboundedLayerContext<B> for LayerContext
where
    B: crate::adapter::layer_tree_host_impl::HostImplTypes<
            ThrottleTypes: crate::adapter::throttle::ThrottleTypes<
                FrameSinkId = super::types::FrameSinkId,
            >,
        > + crate::adapter::layer_tree_impl::LayerTreeTypes<
            LocalSurfaceId = super::types::LocalSurfaceId,
        >,
{
    fn SetUnboundedFrameSinkId(
        &mut self,
        sink: &super::types::FrameSinkId,
        local: &super::types::LocalSurfaceId,
    ) {
        self.0.SetUnboundedFrameSinkId(sink, local)
    }
    fn DismissUnboundedFrameSink(&mut self) {
        self.0.DismissUnboundedFrameSink()
    }
    fn SetUnboundedLocalSurfaceId(&mut self, local: &super::types::LocalSurfaceId) {
        self.0.SetUnboundedLocalSurfaceId(local)
    }
}
/// The native raster provider is owned here, while all acquisition/selection
/// decisions remain in canonical TileManager. Its private capability is only
/// lent by actual embedding services during the current call.
pub struct RasterBufferProvider<B: crate::adapter::tile_manager::TileManagerTypes>(
    pub(crate) Box<dyn crate::adapter::tile_manager::RasterBufferProvider<B>>,
);
use crate::cc::tiles::tile_manager::TileManagerRasterQueryQueue;
pub struct RasterQueryQueue(pub(crate) std::sync::Arc<dyn TileManagerRasterQueryQueue>);
/// Native benchmark implementation. Completion receives the actual owner and
/// services for this call, never a retained host identity masquerading as host.
pub trait NativeMicroBenchmark<R: super::types::CpuSkiaBackend>
where
    super::types::CpuCompositorTypes<R>: crate::adapter::layer_tree_host_impl::HostImplTypes,
{
    fn IsDone(&self) -> bool;
    fn DidCompleteCommit(
        &mut self,
        context: &mut crate::adapter::client_layer_tree_host_impl::BenchmarkContext<
            '_,
            super::types::CpuCompositorTypes<R>,
        >,
    );
}
pub struct MicroBenchmark<R: super::types::CpuSkiaBackend>(pub Box<dyn NativeMicroBenchmark<R>>)
where
    super::types::CpuCompositorTypes<R>: crate::adapter::layer_tree_host_impl::HostImplTypes;

impl<'a, R: super::types::CpuSkiaBackend>
    crate::adapter::micro_benchmark::MicroBenchmarkImpl<
        crate::adapter::client_layer_tree_host_impl::BenchmarkContext<
            'a,
            super::types::CpuCompositorTypes<R>,
        >,
    > for MicroBenchmark<R>
where
    super::types::CpuCompositorTypes<R>: crate::adapter::layer_tree_host_impl::HostImplTypes,
{
    fn IsDone(&self) -> bool {
        self.0.IsDone()
    }
    fn DidCompleteCommit(
        &mut self,
        context: &mut crate::adapter::client_layer_tree_host_impl::BenchmarkContext<
            'a,
            super::types::CpuCompositorTypes<R>,
        >,
    ) {
        self.0.DidCompleteCommit(context)
    }
}
