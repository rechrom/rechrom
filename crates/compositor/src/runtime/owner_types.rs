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

/// The commit family this impl family's main half is (cc's one Layer /
/// CommitState type set; layerize::commit::CommitFamily). The main payloads
/// of the Proxy messages, the main-layer traits and the UI-resource
/// construction are that family's; this family adds only the LayerImpl side.
type Main<R> = layerize::commit::CommitFamily<R>;

// The impl family reads the commit values the main family carries in their
// own types (cc/trees/commit_state.h names one concrete type per value).
impl<R: CpuSkiaBackend> layerize::commit::CommitTypes for CpuCompositorTypes<R> {
    type Transform = <Main<R> as layerize::commit::CommitTypes>::Transform;
    type RegionBackend = <Main<R> as layerize::commit::CommitTypes>::RegionBackend;
    type UIResourceBitmap = <Main<R> as layerize::commit::CommitTypes>::UIResourceBitmap;
    type Filters = <Main<R> as layerize::commit::CommitTypes>::Filters;
    type Path = <Main<R> as layerize::commit::CommitTypes>::Path;
    type CopyOutputRequest = <Main<R> as layerize::commit::CommitTypes>::CopyOutputRequest;
    type DisplayColorSpaces = <Main<R> as layerize::commit::CommitTypes>::DisplayColorSpaces;
    type LocalSurfaceId = <Main<R> as layerize::commit::CommitTypes>::LocalSurfaceId;
    type SurfaceRange = <Main<R> as layerize::commit::CommitTypes>::SurfaceRange;
    type SwapPromise = <Main<R> as layerize::commit::CommitTypes>::SwapPromise;
    type CompositorFrameMetadata =
        <Main<R> as layerize::commit::CommitTypes>::CompositorFrameMetadata;
    type UIResourceRequest = <Main<R> as layerize::commit::CommitTypes>::UIResourceRequest;
    type PresentationCallback = <Main<R> as layerize::commit::CommitTypes>::PresentationCallback;
    type SuccessfulPresentationCallback =
        <Main<R> as layerize::commit::CommitTypes>::SuccessfulPresentationCallback;
    type EventMetrics = <Main<R> as layerize::commit::CommitTypes>::EventMetrics;
    type DelegatedInkMetadata = <Main<R> as layerize::commit::CommitTypes>::DelegatedInkMetadata;
    type ViewTransitionRequest = <Main<R> as layerize::commit::CommitTypes>::ViewTransitionRequest;
}
impl<R: CpuSkiaBackend> LayerTreeTypes for CpuCompositorTypes<R> {
    type Main = Main<R>;
    type Layer = CompositorLayer<Self>;
    type RenderSurface = crate::cc::layers::render_surface_impl::RenderSurfaceImpl<Self>;
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
    type CommitState = CommitState<Main<R>>;
    type UnsafeCommitState = ThreadUnsafeCommitState<Main<R>>;
    type Synchronizer = crate::cc::trees::tree_synchronizer::TreeSynchronizer;
    type PictureLayer = PictureLayerImpl<Self>;
    type ScrollbarLayer = ScrollbarLayerImplBase<Self>;
    type BeginMainFrameReason = super::input::BeginMainFrameReason;
    type LayerList = OwnedLayerImplList<CompositorLayer<Self>>;
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
impl<R: CpuSkiaBackend> layerize::commit::push_properties::PushPropertiesTypes
    for CpuCompositorTypes<R>
{
    type NodeId = <Main<R> as layerize::commit::push_properties::PushPropertiesTypes>::NodeId;
    type RegionCaptureBounds =
        <Main<R> as layerize::commit::push_properties::PushPropertiesTypes>::RegionCaptureBounds;
    type TrackedElementRects =
        <Main<R> as layerize::commit::push_properties::PushPropertiesTypes>::TrackedElementRects;
    type FilterQuality = <Main<R> as layerize::commit::push_properties::PushPropertiesTypes>::FilterQuality;
    type DynamicRangeLimit =
        <Main<R> as layerize::commit::push_properties::PushPropertiesTypes>::DynamicRangeLimit;
}
impl<R: CpuSkiaBackend>
    crate::adapter::scrollbar_layer_base::ScrollbarLayerImplTarget<Main<R>>
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
        tree: &mut crate::adapter::layer_impl::LayerImplContext<'_, CpuCompositorTypes<R>>,
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
        _tree: &mut crate::adapter::layer_impl::LayerImplContext<'_, CpuCompositorTypes<R>>,
    ) {
        crate::cc::layers::scrollbar_layer_impl_base::ScrollbarLayerImplOwner::AsScrollbarBaseMutable(
            self,
        )
        .expect("native scrollbar target subtype")
        .SetScrollElementIdForPropertyPush(value);
    }
}
impl<R: CpuSkiaBackend>
    crate::adapter::painted_scrollbar_layer::PaintedScrollbarLayerImplTarget<Main<R>>
    for CompositorLayer<CpuCompositorTypes<R>>
{
    fn painted_SetInternalContentsScaleAndBounds(
        &mut self,
        scale: f32,
        bounds: crate::adapter::geometry::Size,
    ) {
        painted_scrollbar(self).SetInternalContentsScaleAndBounds(scale, bounds);
    }
    fn painted_SetThumbUIResourceId(&mut self, id: crate::adapter::layer_tree_impl::UIResourceId) {
        painted_scrollbar(self).SetThumbUIResourceId(id);
    }
    fn painted_SetScrollbarPaintedOpacity(&mut self, opacity: f32) {
        painted_scrollbar(self).SetScrollbarPaintedOpacity(opacity);
    }
    fn painted_SetTrackRect(&mut self, rect: crate::adapter::geometry::Rect) {
        painted_scrollbar(self).SetTrackRect(rect);
    }
    fn painted_SetMinimumThumbLength(&mut self, length: i32) {
        painted_scrollbar(self).SetMinimumThumbLength(length);
    }
    fn painted_SetThumbThickness(&mut self, thickness: i32) {
        painted_scrollbar(self).SetThumbThickness(thickness);
    }
    fn painted_set_is_overlay_scrollbar(&mut self, is_overlay: bool) {
        painted_scrollbar(self)
            .scrollbar_base_mut()
            .set_is_overlay_scrollbar(is_overlay);
    }
}
/// Native static_cast to the PaintedScrollbarLayerImpl push target.
fn painted_scrollbar<R: CpuSkiaBackend>(
    layer: &mut CompositorLayer<CpuCompositorTypes<R>>,
) -> &mut crate::cc::layers::painted_scrollbar_layer_impl::PaintedScrollbarLayerImpl<
    CpuCompositorTypes<R>,
> {
    let CompositorLayer::PaintedScrollbar(layer) = layer else {
        panic!("native PaintedScrollbarLayerImpl target subtype")
    };
    layer
}
impl<R: CpuSkiaBackend>
    crate::adapter::solid_color_scrollbar_layer::SolidColorScrollbarLayerImplTarget<Main<R>>
    for CompositorLayer<CpuCompositorTypes<R>>
{
    fn scrollbar_set_color(
        &mut self,
        color: crate::adapter::layer_tree_impl::Color4f,
        _tree: &mut crate::adapter::layer_impl::LayerImplContext<'_, CpuCompositorTypes<R>>,
    ) {
        let CompositorLayer::SolidColorScrollbar(layer) = self else {
            panic!("native solid-color scrollbar target subtype")
        };
        layer.set_color(color);
    }
}
impl<R: CpuSkiaBackend> crate::adapter::surface_layer::SurfaceLayerImplTarget<Main<R>>
    for CompositorLayer<CpuCompositorTypes<R>>
{
    fn SurfaceSetRange(
        &mut self,
        range: <CpuCompositorTypes<R> as layerize::commit::CommitTypes>::SurfaceRange,
        deadline: Option<u32>,
        tree: &mut crate::adapter::layer_impl::LayerImplContext<'_, CpuCompositorTypes<R>>,
    ) {
        let CompositorLayer::Surface(layer) = self else {
            panic!("native SurfaceLayerImpl target subtype")
        };
        layer.SetRange(range, deadline, tree);
    }

    fn SurfaceSetStretchContentToFillBounds(
        &mut self,
        value: bool,
        tree: &mut crate::adapter::layer_impl::LayerImplContext<'_, CpuCompositorTypes<R>>,
    ) {
        let CompositorLayer::Surface(layer) = self else {
            panic!("native SurfaceLayerImpl target subtype")
        };
        layer.SetStretchContentToFillBounds(value, tree);
    }

    fn SurfaceSetIsReflection(
        &mut self,
        value: bool,
        tree: &mut crate::adapter::layer_impl::LayerImplContext<'_, CpuCompositorTypes<R>>,
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
    fn SurfaceRangeEnd(range: &Self::SurfaceRange) -> Self::SurfaceId {
        range.end
    }
}
// viz::SurfaceRange's operations are the main family's (one definition,
// components/viz/common/surfaces/surface_range.cc).
impl<R: CpuSkiaBackend> layerize::commit::surface_range::SurfaceRangeTypes
    for CpuCompositorTypes<R>
{
    fn DefaultSurfaceRange() -> Self::SurfaceRange {
        <Main<R> as layerize::commit::surface_range::SurfaceRangeTypes>::DefaultSurfaceRange()
    }
    fn SurfaceRangesEqual(a: &Self::SurfaceRange, b: &Self::SurfaceRange) -> bool {
        <Main<R> as layerize::commit::surface_range::SurfaceRangeTypes>::SurfaceRangesEqual(a, b)
    }
    fn SurfaceRangeIsValid(range: &Self::SurfaceRange) -> bool {
        <Main<R> as layerize::commit::surface_range::SurfaceRangeTypes>::SurfaceRangeIsValid(range)
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
    type MainLayer = LayerBorrow<Main<R>>;
    fn WithLayerImplFactory<Output>(
        tree: &mut LayerTreeImpl<Self>,
        context: &mut LayerTreeContext<'_, Self>,
        callback: impl FnOnce(
            &mut dyn crate::adapter::main_layer::LayerImplFactory<CompositorLayer<Self>>,
        ) -> Output,
    ) -> Output {
        callback(&mut CpuLayerImplFactory { tree, context })
    }
}

/// The pending LayerTreeImpl lent to main Layer::CreateLayerImpl. Each method
/// runs the canonical most-derived LayerImpl::Create inside the tree's scoped
/// impl context, exactly as the source overrides call it.
struct CpuLayerImplFactory<'a, 'c, R: CpuSkiaBackend> {
    tree: &'a mut LayerTreeImpl<CpuCompositorTypes<R>>,
    context: &'a mut LayerTreeContext<'c, CpuCompositorTypes<R>>,
}
impl<R: CpuSkiaBackend> crate::adapter::main_layer::LayerImplFactory<CompositorLayer<CpuCompositorTypes<R>>>
    for CpuLayerImplFactory<'_, '_, R>
{
    // Source: cc/layers/layer.cc:1619-1622.
    fn CreateLayer(&mut self, id: i32) -> Box<CompositorLayer<CpuCompositorTypes<R>>> {
        CpuCompositorTypes::<R>::WithLayerImplContext(self.tree, self.context, |tree_id, c| {
            LayerImpl::<CpuCompositorTypes<R>>::Create(tree_id, id, c)
        })
    }
    // Source: cc/layers/picture_layer.cc:38-41.
    fn CreatePictureLayer(&mut self, id: i32) -> Box<CompositorLayer<CpuCompositorTypes<R>>> {
        CreateCpuPictureLayer(self.tree, self.context, id)
    }
    // Source: cc/layers/solid_color_layer.cc:14-17.
    fn CreateSolidColorLayer(&mut self, id: i32) -> Box<CompositorLayer<CpuCompositorTypes<R>>> {
        CpuCompositorTypes::<R>::WithLayerImplContext(self.tree, self.context, |tree_id, c| {
            crate::cc::layers::solid_color_layer_impl::SolidColorLayerImpl::Create(tree_id, id, c)
        })
    }
    // Source: cc/layers/surface_layer.cc SurfaceLayer::CreateLayerImpl.
    fn CreateSurfaceLayer(&mut self, id: i32) -> Box<CompositorLayer<CpuCompositorTypes<R>>> {
        CpuCompositorTypes::<R>::WithLayerImplContext(self.tree, self.context, |tree_id, c| {
            crate::cc::layers::surface_layer_impl::SurfaceLayerImpl::Create(tree_id, id, c)
        })
    }
    // Source: cc/layers/painted_scrollbar_layer.cc
    //         PaintedScrollbarLayer::CreateLayerImpl.
    fn CreatePaintedScrollbarLayer(
        &mut self,
        id: i32,
        orientation: crate::cc::input::scrollbar::ScrollbarOrientation,
        left: bool,
    ) -> Box<CompositorLayer<CpuCompositorTypes<R>>> {
        CpuCompositorTypes::<R>::WithLayerImplContext(self.tree, self.context, |tree_id, c| {
            crate::cc::layers::painted_scrollbar_layer_impl::PaintedScrollbarLayerImpl::<
                CpuCompositorTypes<R>,
            >::Create(tree_id, id, orientation, left, c)
        })
    }
    // Source: cc/layers/solid_color_scrollbar_layer.cc:16-21.
    fn CreateSolidColorScrollbarLayer(
        &mut self,
        id: i32,
        orientation: crate::cc::input::scrollbar::ScrollbarOrientation,
        thumb_thickness: i32,
        track_start: i32,
        left: bool,
    ) -> Box<CompositorLayer<CpuCompositorTypes<R>>> {
        CpuCompositorTypes::<R>::WithLayerImplContext(self.tree, self.context, |tree_id, c| {
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
}

// This non-test assertion intentionally fails if any selected native owner,
// live factory or external value capability is still absent at registration.
const _: fn() = || {
    fn require<B: LayerTreeTypes<Main: MainLayerTypes> + LayerImplTypes>() {}
    require::<CpuCompositorTypes<crate::runtime::skia::SkRegion>>();
};

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
) -> LayerRef<Main<R>> {
    crate::cc::layers::surface_layer::SurfaceLayer::<Main<R>>::Create(services)
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
