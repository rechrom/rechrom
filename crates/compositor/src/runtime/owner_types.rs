//! The production CPU family's canonical Layer/LayerTree owner binding.
//! Registered atomically when its source-backed Paint and live factories close;
//! this file contains no placeholder owners, fallback algorithms or old core.
use super::{
    geometry::Transform,
    layer::CompositorLayer,
    types::{CpuCompositorTypes, CpuSkiaBackend},
};
use crate::{
    adapter::{
        layer_impl::LayerImplTypes,
        layer_tree_impl::{LayerTreeDrawState, LayerTreeTypes, PropertyTreesFor},
        main_layer::MainLayerTypes,
    },
    cc::{
        layers::{
            layer::{LayerBorrow, LayerRef},
            layer_collections::{LayerStorage, LayerStorageRemaining, OwnedLayerImplList},
            layer_impl::LayerImpl,
            picture_layer_impl::PictureLayerImpl,
            scrollbar_layer_impl_base::ScrollbarLayerImplBase,
        },
        trees::{
            commit_state::{CommitState, ThreadUnsafeCommitState},
            layer_tree_impl::{LayerTreeContext, LayerTreeImpl},
        },
    },
};

impl<R: CpuSkiaBackend> LayerTreeTypes for CpuCompositorTypes<R> {
    type LayerTraceValue = super::host::TraceValue;
    type LayerStorage<'a> = LayerStorage<'a, CompositorLayer<Self>>;
    type LayerRemaining<'a> = LayerStorageRemaining<'a, CompositorLayer<Self>>;
    type LayerContext<'a> = super::layer_context::FullLayerContext<'a, Self>;
    type PictureLayerContext<'a> = super::layer_context::PictureContext<'a, Self>;
    type LayerSourceContext<'a> = super::layer_context::SourceContext<'a, Self>;
    fn WithLayerContext<Output>(
        tree: &mut LayerTreeImpl<Self>,
        context: &mut LayerTreeContext<'_, Self>,
        id: i32,
        callback: impl FnOnce(&mut Self::Layer, &mut Self::LayerContext<'_>) -> Output,
    ) -> Output {
        super::layer_context::WithLayerContext(tree, context, id, callback)
    }
    fn WithDetachedLayerContext<Output>(
        tree: &mut LayerTreeImpl<Self>,
        context: &mut LayerTreeContext<'_, Self>,
        layer: &mut Self::Layer,
        callback: impl FnOnce(&mut Self::Layer, &mut Self::LayerContext<'_>) -> Output,
    ) -> Output {
        super::layer_context::WithDetachedLayerContext(tree, context, layer, callback)
    }
    fn WithPictureLayersContext<Output>(
        tree: &mut LayerTreeImpl<Self>,
        host: &mut dyn crate::adapter::picture_layer_impl::PictureLayerContextHost<Self>,
        callback: impl FnOnce(&mut Self::LayerStorage<'_>, &mut Self::PictureLayerContext<'_>) -> Output,
    ) -> Output {
        super::layer_context::WithPictureLayersContext(tree, host, callback)
    }
    fn WithLayerSyncContext<Output>(
        source: &mut LayerTreeImpl<Self>,
        target: &mut LayerTreeImpl<Self>,
        context: &mut LayerTreeContext<'_, Self>,
        id: i32,
        callback: impl FnOnce(
            &mut Self::Layer,
            &Self::LayerSourceContext<'_>,
            &mut Self::Layer,
            &mut Self::LayerContext<'_>,
        ) -> Output,
    ) -> Output {
        super::layer_context::WithLayerSyncContext(source, target, context, id, callback)
    }
    type DamageContext<'a> = super::draw::DamageContext<'a, Self>;
    fn MakeDamageContext<'a>(
        tree: &'a LayerTreeImpl<Self>,
        host: &'a dyn crate::adapter::layer_tree_impl::LayerTreeHost<Self>,
    ) -> Self::DamageContext<'a> {
        super::draw::DamageContext::new(tree, host)
    }
    type DrawContext<'a> = super::draw::DrawContext<'a, Self>;
    fn MakeDrawContext<'a>(
        state: LayerTreeDrawState<'a, Self>,
        host: &'a mut dyn crate::adapter::layer_tree_impl::draw_host::LayerTreeDrawHost<Self>,
    ) -> Self::DrawContext<'a> {
        super::draw::DrawContext::new(state, host)
    }
    type OcclusionContext<'a> = super::draw::OcclusionContext<'a, Self>;
    fn MakeOcclusionContext<'a>(
        layers: &'a Self::LayerList,
        trees: &'a PropertyTreesFor<Self>,
        surfaces: &'a [i32],
    ) -> Self::OcclusionContext<'a> {
        super::draw::OcclusionContext::new(layers, trees, surfaces)
    }
    type CommitState = CommitState<Self>;
    type UnsafeCommitState = ThreadUnsafeCommitState<Self>;
    type Synchronizer = crate::cc::trees::tree_synchronizer::TreeSynchronizer;
    type Transform = Transform<R>;
    type Filters = R::Filters;
    type Path = R::Path;
    type RenderSurface = crate::cc::layers::render_surface_impl::RenderSurfaceImpl<Self>;
    type CopyOutputRequest = super::types::CopyOutputRequest;
    type RegionBackend = R;
    type Layer = CompositorLayer<Self>;
    type PictureLayer = PictureLayerImpl<Self>;
    type ScrollbarLayer = ScrollbarLayerImplBase<Self>;
    type BeginMainFrameReason = super::input::BeginMainFrameReason;
    type LayerList = OwnedLayerImplList<CompositorLayer<Self>>;
    type DisplayColorSpaces = super::resources::RasterDisplayColorSpaces;
    type LocalSurfaceId = super::types::LocalSurfaceId;
    type SurfaceRange = super::types::SurfaceRange;
    type SwapPromise = super::host_owners::SwapPromise<R>;
    type CompositorFrameMetadata = super::metadata::CompositorFrameMetadata<Self>;
    type UIResourceRequest = super::host_owners::UIResourceRequest;
    type UIResourceBitmap = super::host_resources::UIBitmap;
    type PresentationCallback =
        crate::adapter::presentation::Callback<super::metadata::PresentationFeedback>;
    type SuccessfulPresentationCallback =
        crate::adapter::presentation::SuccessfulCallbackWithDetails<
            super::metadata::FrameTimingDetails,
        >;
    type EventMetrics = super::input::EventMetrics;
    type DelegatedInkMetadata = super::metadata::DelegatedInkMetadata;
    type ViewTransitionRequest = super::host_owners::ViewTransitionRequest;
    type SelectionBound = super::metadata::SelectionBound;
    type DecodingMode = crate::adapter::picture_layer_impl::PictureImageDecodingMode;
    type Settings = crate::cc::trees::layer_tree_settings::LayerTreeSettings;
    type DebugState = crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState;
    type LayerTreeFrameSink = super::frame_sink::FrameSink<Self>;
    type RasterContextProvider = super::host_owners::RasterContextProvider;
    type ClientResourceProvider = super::host_owners::ClientResourceProvider<Self>;
    type TileManager = crate::cc::tiles::tile_manager::TileManager<Self>;
    type ImageDecodeCache = super::host_owners::ImageDecodeCache;
    type ImageAnimationController =
        crate::cc::trees::image_animation_controller::ImageAnimationController;
    type FrameSorter = super::host_owners::FrameSorter;
    type MemoryHistory = crate::cc::resources::memory_history::MemoryHistory;
    type DebugRectHistory = crate::cc::trees::debug_rect_history::DebugRectHistory;
    type GlobalTileState = crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority;
    type VideoFrameControllerClient = super::host_owners::VideoFrameControllerClient;
    type MutatorHost = super::input::MutatorHost;
    type TaskRunnerProvider = super::host_owners::TaskRunnerProvider;
    type PaintWorkletTracker =
        crate::cc::trees::animated_paint_worklet_tracker::AnimatedPaintWorkletTracker<Self>;
    type PaintImageId = crate::adapter::image_animation::PaintImageId;
    type PrioritizedTile = crate::cc::tiles::prioritized_tile::PrioritizedTile;
    type AnimatedImageDriverMap = crate::adapter::image_animation::AnimatedImageDriverMap;
    type DisplayItemList = super::paint::ContentDisplayItemList<R>;
    type ContentColorUsage = crate::adapter::layer_tree_host_impl::ContentColorUsage;
    type TargetColorParams = crate::adapter::layer_tree_host_impl::HostTargetColorParams<
        super::resources::RasterColorSpace,
    >;
}
impl<R: CpuSkiaBackend> MainLayerTypes for CpuCompositorTypes<R> {
    type NodeId = super::types::NodeId;
    type RegionCaptureBounds = super::metadata::RegionCaptureBounds;
    type TrackedElementRects = super::metadata::TrackedElementRects;
    type FilterQuality = super::paint::FilterQuality;
    type DynamicRangeLimit = super::paint::DynamicRangeLimit;
    type DidScrollCallback = super::types::DidScrollCallback;
    type NodeInfo = super::display_item_list::NodeInfo;
    type Picture = super::paint::Picture<R>;
    type LayerSelection = crate::cc::input::layer_selection_bound::LayerSelection;
    type MicroBenchmark = super::paint::MicroBenchmark<Self>;
    type TaskRunner = super::types::TaskRunner;
    type ImplContext<'a> = crate::adapter::layer_impl::LayerImplContext<'a, Self>;
    type DerivedServices = dyn super::paint::CpuPaintServices<Self>;
}
impl<R: CpuSkiaBackend> crate::adapter::scrollbar_layer_base::ScrollbarLayerTypes
    for CpuCompositorTypes<R>
{
    type ScrollbarCanvas = super::paint::ScrollbarCanvas;
    type Scrollbar = super::paint::Scrollbar;
    fn RasterizeScrollbarThumb(
        scrollbar: &Self::Scrollbar,
        logical_size: crate::adapter::geometry::Size,
        pixel_size: crate::adapter::geometry::Size,
    ) -> <Self as crate::adapter::layer_tree_impl::LayerTreeTypes>::UIResourceBitmap {
        use crate::cc::input::scrollbar::Scrollbar as _;
        let mut canvas = super::paint::ScrollbarCanvas::new(logical_size, pixel_size);
        let mut adapter = scrollbar.clone();
        adapter.PaintThumb(
            &mut canvas,
            &crate::adapter::geometry::Rect::new(0, 0, logical_size.width(), logical_size.height()),
        );
        super::host_resources::UIBitmap(std::sync::Arc::new(
            super::host_resources::RawUIBitmap::new(
                canvas.pixel_size(),
                canvas.pixels().to_vec(),
                canvas.row_bytes(),
                false,
            ),
        ))
    }
    fn PaintedScrollbarConstructor(
    ) -> Option<crate::adapter::scrollbar_layer_base::ScrollbarConstructor<Self>> {
        Some(crate::cc::layers::painted_scrollbar_layer::PaintedScrollbarLayer::<Self>::Constructor)
    }
}
impl<R: CpuSkiaBackend> crate::adapter::solid_color_scrollbar_layer::SolidColorScrollbarLayerTypes
    for CpuCompositorTypes<R>
{
}
impl<R: CpuSkiaBackend> crate::adapter::painted_scrollbar_layer::PaintedScrollbarLayerTypes
    for CpuCompositorTypes<R>
{
}
impl<R: CpuSkiaBackend>
    crate::adapter::scrollbar_layer_base::ScrollbarLayerImplTarget<CpuCompositorTypes<R>>
    for CompositorLayer<CpuCompositorTypes<R>>
{
    fn scrollbar_orientation(&self) -> crate::cc::input::scrollbar::ScrollbarOrientation {
        crate::cc::layers::scrollbar_layer_impl_base::ScrollbarLayerImplOwner::AsScrollbarBase(self)
            .expect("native scrollbar target subtype")
            .orientation()
    }

    fn scrollbar_is_left_side_vertical(&self) -> bool {
        crate::cc::layers::scrollbar_layer_impl_base::ScrollbarLayerImplOwner::AsScrollbarBase(self)
            .expect("native scrollbar target subtype")
            .is_left_side_vertical_scrollbar()
    }

    fn scrollbar_SetHasFindInPageTickmarks(
        &mut self,
        value: bool,
        tree: &mut <CpuCompositorTypes<R> as MainLayerTypes>::ImplContext<'_>,
    ) {
        crate::cc::layers::scrollbar_layer_impl_base::ScrollbarLayerImplOwner::AsScrollbarBaseMutable(
            self,
        )
        .expect("native scrollbar target subtype")
        .SetHasFindInPageTickmarks(value, tree);
    }

    fn scrollbar_SetScrollElementId(
        &mut self,
        value: crate::adapter::input_types::ElementId,
        _tree: &mut <CpuCompositorTypes<R> as MainLayerTypes>::ImplContext<'_>,
    ) {
        crate::cc::layers::scrollbar_layer_impl_base::ScrollbarLayerImplOwner::AsScrollbarBaseMutable(
            self,
        )
        .expect("native scrollbar target subtype")
        .SetScrollElementIdForPropertyPush(value);
    }
}
impl<R: CpuSkiaBackend>
    crate::adapter::painted_scrollbar_layer::PaintedScrollbarLayerImplTarget<CpuCompositorTypes<R>>
    for CompositorLayer<CpuCompositorTypes<R>>
{
    fn CreatePaintedScrollbar(
        tree: &mut LayerTreeImpl<CpuCompositorTypes<R>>,
        id: i32,
        orientation: crate::cc::input::scrollbar::ScrollbarOrientation,
        left: bool,
        context: &mut LayerTreeContext<'_, CpuCompositorTypes<R>>,
    ) -> Box<Self> {
        CpuCompositorTypes::<R>::WithLayerImplContext(tree, context, |tree_id, c| {
            crate::cc::layers::painted_scrollbar_layer_impl::PaintedScrollbarLayerImpl::<
                CpuCompositorTypes<R>,
            >::Create(tree_id, id, orientation, left, c)
        })
    }
    fn painted_scrollbar(
        &mut self,
    ) -> Option<
        &mut crate::cc::layers::painted_scrollbar_layer_impl::PaintedScrollbarLayerImpl<
            CpuCompositorTypes<R>,
        >,
    > {
        if let CompositorLayer::PaintedScrollbar(layer) = self {
            Some(layer)
        } else {
            None
        }
    }
}
impl<R: CpuSkiaBackend>
    crate::adapter::solid_color_scrollbar_layer::SolidColorScrollbarLayerImplTarget<
        CpuCompositorTypes<R>,
    > for CompositorLayer<CpuCompositorTypes<R>>
{
    fn CreateSolidColorScrollbar(
        tree: &mut LayerTreeImpl<CpuCompositorTypes<R>>,
        id: i32,
        orientation: crate::cc::input::scrollbar::ScrollbarOrientation,
        thumb_thickness: i32,
        track_start: i32,
        left: bool,
        context: &mut LayerTreeContext<'_, CpuCompositorTypes<R>>,
    ) -> Box<Self> {
        CpuCompositorTypes::<R>::WithLayerImplContext(tree, context, |tree_id, c| {
            crate::cc::layers::solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImpl::<
                CpuCompositorTypes<R>,
            >::Create(
                tree_id,
                id,
                orientation,
                thumb_thickness,
                track_start,
                left,
                c,
            )
        })
    }

    fn scrollbar_set_color(
        &mut self,
        color: crate::adapter::layer_tree_impl::Color4f,
        _tree: &mut <CpuCompositorTypes<R> as MainLayerTypes>::ImplContext<'_>,
    ) {
        let CompositorLayer::SolidColorScrollbar(layer) = self else {
            panic!("native solid-color scrollbar target subtype")
        };
        layer.set_color(color);
    }
}
impl<R: CpuSkiaBackend> crate::adapter::surface_layer::SurfaceLayerImplTarget<CpuCompositorTypes<R>>
    for CompositorLayer<CpuCompositorTypes<R>>
{
    fn CreateSurface(
        tree: &mut LayerTreeImpl<CpuCompositorTypes<R>>,
        id: i32,
        context: &mut LayerTreeContext<'_, CpuCompositorTypes<R>>,
    ) -> Box<Self> {
        CpuCompositorTypes::<R>::WithLayerImplContext(tree, context, |tree_id, c| {
            crate::cc::layers::surface_layer_impl::SurfaceLayerImpl::Create(tree_id, id, c)
        })
    }

    fn SurfaceSetRange(
        &mut self,
        range: <CpuCompositorTypes<R> as LayerTreeTypes>::SurfaceRange,
        deadline: Option<u32>,
        tree: &mut <CpuCompositorTypes<R> as MainLayerTypes>::ImplContext<'_>,
    ) {
        let CompositorLayer::Surface(layer) = self else {
            panic!("native SurfaceLayerImpl target subtype")
        };
        layer.SetRange(range, deadline, tree);
    }

    fn SurfaceSetStretchContentToFillBounds(
        &mut self,
        value: bool,
        tree: &mut <CpuCompositorTypes<R> as MainLayerTypes>::ImplContext<'_>,
    ) {
        let CompositorLayer::Surface(layer) = self else {
            panic!("native SurfaceLayerImpl target subtype")
        };
        layer.SetStretchContentToFillBounds(value, tree);
    }

    fn SurfaceSetIsReflection(
        &mut self,
        value: bool,
        tree: &mut <CpuCompositorTypes<R> as MainLayerTypes>::ImplContext<'_>,
    ) {
        let CompositorLayer::Surface(layer) = self else {
            panic!("native SurfaceLayerImpl target subtype")
        };
        layer.SetIsReflection(value, tree);
    }
}
impl<R: CpuSkiaBackend> LayerImplTypes for CpuCompositorTypes<R> {
    type SharedQuadState = super::frame::SharedQuadState<Transform<R>>;
    type DrawQuad = super::frame::DrawQuad<super::paint::ContentDisplayItemList<R>>;
    type DeletionTrace = super::types::DeletionTrace;
    fn WrapLayerImpl(layer: LayerImpl<Self>) -> Self::Layer {
        CompositorLayer::Base(layer)
    }
    fn WithLayerImplContext<Output>(
        tree: &mut LayerTreeImpl<Self>,
        context: &mut LayerTreeContext<'_, Self>,
        callback: impl FnOnce(
            crate::adapter::layer_tree_impl::LayerTreeId,
            &mut crate::adapter::layer_impl::LayerImplContext<'_, Self>,
        ) -> Output,
    ) -> Output {
        super::layer_context::WithLayerImplContext(tree, context, callback)
    }
}
impl<R: CpuSkiaBackend> crate::adapter::solid_color_layer_impl::SolidColorLayerImplTypes
    for CpuCompositorTypes<R>
{
    fn WrapSolidColorLayerImpl(
        layer: crate::cc::layers::solid_color_layer_impl::SolidColorLayerImpl<Self>,
    ) -> Self::Layer {
        CompositorLayer::SolidColor(layer)
    }
}
impl<R: CpuSkiaBackend> crate::adapter::surface_layer_impl::SurfaceLayerImplTypes
    for CpuCompositorTypes<R>
{
    fn WrapSurfaceLayerImpl(
        layer: crate::cc::layers::surface_layer_impl::SurfaceLayerImpl<Self>,
    ) -> Self::Layer {
        CompositorLayer::Surface(layer)
    }
    fn DefaultSurfaceRange() -> Self::SurfaceRange {
        Default::default()
    }
    fn SurfaceRangesEqual(a: &Self::SurfaceRange, b: &Self::SurfaceRange) -> bool {
        a == b
    }
    fn SurfaceRangeIsValid(range: &Self::SurfaceRange) -> bool {
        let valid = |id: &super::types::SurfaceId| {
            id.frame_sink_id.client_id != 0
                && id.frame_sink_id.sink_id != 0
                && id.local_surface_id.parent_sequence_number != 0
                && id.local_surface_id.child_sequence_number != 0
                && id.local_surface_id.embed_token != [0, 0]
        };
        valid(&range.end) && range.start.as_ref().is_none_or(valid)
    }
    fn SurfaceRangeEnd(range: &Self::SurfaceRange) -> Self::SurfaceId {
        range.end
    }
}
impl<R: CpuSkiaBackend>
    crate::adapter::solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes
    for CpuCompositorTypes<R>
{
    fn WrapSolidColorScrollbarLayerImpl(
        layer: crate::cc::layers::solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImpl<
            Self,
        >,
    ) -> Self::Layer {
        CompositorLayer::SolidColorScrollbar(layer)
    }
    fn WrapPaintedScrollbarLayerImpl(
        layer: crate::cc::layers::painted_scrollbar_layer_impl::PaintedScrollbarLayerImpl<Self>,
    ) -> Self::Layer {
        CompositorLayer::PaintedScrollbar(layer)
    }
}
impl<R: CpuSkiaBackend> crate::adapter::render_surface_impl::RenderSurfaceTypes
    for CpuCompositorTypes<R>
{
}
impl<R: CpuSkiaBackend> crate::adapter::tree_sync::SyncTypes for CpuCompositorTypes<R> {
    type MainLayer = LayerBorrow<Self>;
}

// This non-test assertion intentionally fails if any selected native owner,
// live factory or external value capability is still absent at registration.
const _: fn() = || {
    fn require<B: LayerTreeTypes + MainLayerTypes + LayerImplTypes>() {}
    require::<CpuCompositorTypes<crate::runtime::skia::SkRegion>>();
};

impl<R: CpuSkiaBackend> crate::adapter::commit_state::CommitStateTypes for CpuCompositorTypes<R> {
    type BeginMainFrameMetrics = crate::adapter::scheduler_host::BeginMainFrameMetrics;
    type DrawImage = super::paint::DrawImage;
    type MicroBenchmarkImpl = super::host_owners::MicroBenchmark<R>;
    type MainRootLayer = LayerRef<Self>;
    type MainLayerRef = LayerBorrow<Self>;
    type MutatorHostHandle = super::types::MainMutatorHostIdentity;
}

/// Assembly entry to the canonical most-derived PictureLayerImpl constructor.
/// The current resident tree is field-split by the same source-backed factory
/// used during synchronization; no parallel layer owner or plan is produced.
pub fn CreateCpuPictureLayer<R: CpuSkiaBackend>(
    tree: &mut LayerTreeImpl<CpuCompositorTypes<R>>,
    context: &mut LayerTreeContext<'_, CpuCompositorTypes<R>>,
    id: i32,
) -> Box<CompositorLayer<CpuCompositorTypes<R>>> {
    let tree_id = tree.id;
    CpuCompositorTypes::<R>::WithPictureLayersContext(tree, context.host, |_, context| {
        use crate::adapter::layer_tree_impl::LayerTreePictureContext;
        context.WithPicture(|context| {
            PictureLayerImpl::<CpuCompositorTypes<R>>::Create(tree_id, id, context)
        })
    })
}

/// Public bridge constructor used by Blink ForeignLayer producers such as
/// VideoPainter. The returned resident cc::SurfaceLayer is updated through
/// LayerRef::SetSurfaceRange and never owns the child frame.
pub fn CreateCpuSurfaceLayer<R: CpuSkiaBackend>(
    services: &dyn crate::adapter::main_layer::MainLayerServices,
) -> LayerRef<CpuCompositorTypes<R>> {
    crate::cc::layers::surface_layer::SurfaceLayer::<CpuCompositorTypes<R>>::Create(services)
}

/// Concrete native LayerImpl scroll entry with the same scoped current/rest
/// projection as property synchronization. Assembly lends the host callbacks
/// and diagnostics from its actual independent resident fields.
pub(crate) fn ScrollCpuLayer<R: CpuSkiaBackend>(
    tree: &mut LayerTreeImpl<CpuCompositorTypes<R>>,
    host: &mut dyn crate::adapter::layer_tree_impl::mutation::LayerTreeScrollHost<
        CpuCompositorTypes<R>,
    >,
    debug: &crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState,
    caps: &crate::cc::trees::raster_capabilities::RasterCapabilities,
    id: i32,
    delta: crate::adapter::geometry::Vector2dF,
) -> crate::adapter::geometry::Vector2dF {
    super::layer_context::WithLayerScrollContext::<CpuCompositorTypes<R>, _>(
        tree,
        host,
        debug,
        caps,
        id,
        |layer, c| layer.ScrollBy(delta, c),
    )
}
pub(crate) fn SetCpuLayerScrollOffset<R: CpuSkiaBackend>(
    tree: &mut LayerTreeImpl<CpuCompositorTypes<R>>,
    host: &mut dyn crate::adapter::layer_tree_impl::mutation::LayerTreeScrollHost<
        CpuCompositorTypes<R>,
    >,
    debug: &crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState,
    caps: &crate::cc::trees::raster_capabilities::RasterCapabilities,
    id: i32,
    offset: crate::adapter::geometry::PointF,
) {
    super::layer_context::WithLayerScrollContext::<CpuCompositorTypes<R>, _>(
        tree,
        host,
        debug,
        caps,
        id,
        |layer, c| layer.SetCurrentScrollOffset(offset, c),
    );
}
