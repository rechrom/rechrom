//! Exact Paint metadata values and call-scoped dependencies for PictureLayerImpl.
//! Paint allocations remain external; layer/tiling/worklet policy stays in the
//! canonical translated owner, with no retained owner or service backpointer.
use super::{
    image_animation::PaintImageId,
    layer_tree_impl::{Color4f, LayerTreeTypes},
    paint_worklet::PaintWorkletTypes,
    picture_layer_tiling::{PictureLayerTilingTypes, TilingRasterSource},
    tile_based_layer_impl::TileBasedLayerImplTypes,
};
use std::sync::Arc;
pub trait PictureLayerImplTypes:
    TileBasedLayerImplTypes<Layer: PictureLayerImplOwner<Self>, RenderPass: PictureRenderPass<Self>>
    + PaintWorkletTypes<
        PaintImageId = PaintImageId,
        Input: PicturePaintWorkletInput<Self::PropertyKey, Self::PropertyValue>,
    > + PictureLayerTilingTypes<
        RegionBackend = <Self as LayerTreeTypes>::RegionBackend,
        OcclusionTransform = <Self as LayerTreeTypes>::Transform,
        RasterSource: PictureRasterSource<DisplayItemList: PictureDisplayItemListFor<Self>>,
        PaintWorkletRecordMap = PaintWorkletRecordMap<
            <Self as PaintWorkletTypes>::Input,
            Self::PaintRecord,
        >,
    >
{
    type PaintRecord: Clone;
    type PictureTargetColorParams: PartialEq;
    type DiscardableImageMap: PictureDiscardableImageMap<Self>;
    fn WrapPictureLayerImpl(
        layer: crate::cc::layers::picture_layer_impl::PictureLayerImpl<Self>,
    ) -> Self::Layer;
}
/// Native most-derived target projection, never a base-to-owner lookup.
pub trait PictureLayerImplOwner<B: PictureLayerImplTypes>:
    super::layer_impl::LayerImplOwner<B>
{
    fn picture_impl(&self) -> Option<&crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>>;
    fn picture_impl_mut(
        &mut self,
    ) -> Option<&mut crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>>;
}
pub trait PicturePaintWorkletInput<K, V>: super::paint_worklet::PaintWorkletInput<K> {
    fn ValueChangeShouldCauseRepaint(&self, prev: &V, next: &V) -> bool;
}
pub trait PictureDisplayItemList {
    fn has_draw_text_ops(&self) -> bool;
    fn content_color_usage(&self) -> super::layer_tree_host_impl::ContentColorUsage;
    fn raster_inducing_scrolls(
        &self,
    ) -> &std::collections::BTreeMap<super::input_types::ElementId, RasterInducingScrollInfo>;
}
pub use crate::cc::paint::display_item_list::RasterInducingScrollInfo;
pub trait PictureDiscardableImageMap<B: PictureLayerImplTypes> {
    fn empty(&self) -> bool;
    fn animated_images_metadata(
        &self,
    ) -> &std::collections::BTreeMap<PaintImageId, super::image_animation::AnimatedImageMetadata>;
    fn GetRectsForImage(&self, id: PaintImageId) -> &[super::geometry::Rect];
    fn GetDiscardableImagesInRect(
        &self,
        rect: &super::geometry::Rect,
    ) -> Vec<&<B as PictureLayerTilingTypes>::DrawImage>;
}
pub trait PictureRasterSource: TilingRasterSource {
    type DisplayItemList: PictureDisplayItemList;
    fn GetDisplayItemList(&self) -> &Option<Arc<Self::DisplayItemList>>;
    fn HasRecordings(&self) -> bool;
    fn GetSolidColor(&self) -> Color4f;
    fn directly_composited_image_info(&self) -> Option<&DirectlyCompositedImageInfo>;
    fn DidBeginTracing(&self);
    fn set_debug_name(&self, name: &str);
    fn AsValueInto(&self, value: &mut impl super::property_types::PropertyTrace);
}

// Source dependency: cc/paint/paint_image.h:219-233. External Paint values.
pub use super::layer_tree_host_impl::HostImageDecodingMode as PictureImageDecodingMode;
pub type PictureImageDecodingModeMap =
    std::collections::BTreeMap<PaintImageId, PictureImageDecodingMode>;
pub type PicturePaintWorkletInputs<B> = Vec<(Arc<<B as PaintWorkletTypes>::Input>, PaintImageId)>;
pub trait PictureDisplayItemListFor<B: PictureLayerImplTypes>: PictureDisplayItemList {
    fn GenerateDiscardableImageMap(
        &self,
        offsets: super::prioritized_tile::ScrollOffsetMap,
        decoding: Option<&mut PictureImageDecodingModeMap>,
        worklets: Option<&mut PicturePaintWorkletInputs<B>>,
    ) -> Arc<B::DiscardableImageMap>;
}
// Source dependency: cc/paint/directly_composited_image_info.h:14-19.
#[derive(Clone, Copy)]
pub struct DirectlyCompositedImageInfo {
    pub default_raster_scale: super::geometry::Vector2dF,
    pub nearest_neighbor: bool,
}
/// Source dependency: cc/paint/paint_worklet_input.h:101-106.
/// The native flat_map's key is the actual scoped_refptr allocation identity.
/// This directly owned sorted vector is the same flat storage, not a projection
/// or lookup service. Its values preserve optional<PaintRecord> ownership.
pub struct PaintWorkletRecordMap<I, R> {
    entries: Vec<(Arc<I>, (PaintImageId, Option<R>))>,
}
impl<I, R> Default for PaintWorkletRecordMap<I, R> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}
impl<I, R: Clone> Clone for PaintWorkletRecordMap<I, R> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}
impl<I, R> PaintWorkletRecordMap<I, R> {
    fn find(&self, input: &Arc<I>) -> Result<usize, usize> {
        let identity = Arc::as_ptr(input) as usize;
        self.entries
            .binary_search_by_key(&identity, |(key, _)| Arc::as_ptr(key) as usize)
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// The native flat_map::size field query, without visiting/copying entries.
    pub(crate) fn input_at(&self, index: usize) -> &Arc<I> {
        &self.entries[index].0
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn contains_key(&self, input: &Arc<I>) -> bool {
        self.find(input).is_ok()
    }
    pub fn get(&self, input: &Arc<I>) -> Option<&(PaintImageId, Option<R>)> {
        self.find(input).ok().map(|index| &self.entries[index].1)
    }
    pub fn value_mut_or_default(&mut self, input: &Arc<I>) -> &mut (PaintImageId, Option<R>) {
        let index = match self.find(input) {
            Ok(index) => index,
            Err(index) => {
                // Native operator[] value-initializes pair<ImageId, optional<Record>>.
                self.entries
                    .insert(index, (Arc::clone(input), (PaintImageId(0), None)));
                index
            }
        };
        &mut self.entries[index].1
    }
    pub fn insert(&mut self, input: Arc<I>, value: (PaintImageId, Option<R>)) {
        match self.find(&input) {
            Ok(index) => self.entries[index].1 = value,
            Err(index) => self.entries.insert(index, (input, value)),
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (&Arc<I>, &(PaintImageId, Option<R>))> {
        self.entries.iter().map(|(input, value)| (input, value))
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Arc<I>, &mut (PaintImageId, Option<R>))> {
        self.entries
            .iter_mut()
            .map(|(input, value)| (&*input, value))
    }
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
}
// Source dependency: cc/raster/lcd_text_disallowed_reason.h:16-30. This is a
// typed raster vocabulary value only; no raster implementation enters cc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LCDTextDisallowedReason {
    kNone = 0,
    kSetting = 1,
    kBackgroundColorNotOpaque = 2,
    kContentsNotOpaque = 3,
    kNonIntegralTranslation = 4,
    kNonIntegralXOffset = 5,
    kNonIntegralYOffset = 6,
    kWillChangeTransform = 7,
    kPixelOrColorEffect = 8,
    kTransformAnimation = 9,
    kNoText = 10,
}
impl LCDTextDisallowedReason {
    pub const kMaxValue: Self = Self::kNoText;
}
/// Read-only live tree classification consumed by native image animation and
/// tile-priority queries. It cannot carry copied flags or a cached decision.
pub trait PictureLayerReadContext {
    fn IsRecycleTree(&self) -> bool;
}
pub trait PictureLayerImplReadTree<B: PictureLayerImplTypes>:
    super::layer_impl::LayerImplReadTree<B> + PictureLayerReadContext
{
    fn raster_caps(&self) -> &crate::cc::trees::raster_capabilities::RasterCapabilities;
    fn GetMSAASampleCountForRaster(
        &self,
        list:&<<B as PictureLayerTilingTypes>::RasterSource as PictureRasterSource>::DisplayItemList,
    ) -> i32;
    fn GetTargetColorParams(
        &self,
        usage: super::layer_tree_host_impl::ContentColorUsage,
    ) -> B::PictureTargetColorParams;
    fn ViewportRectForTilePriority(&self) -> super::geometry::Rect;
    fn IsReadyToActivate(&self) -> bool;
    fn HasPendingTree(&self) -> bool;
    fn GetActivelyScrollingType(
        &self,
    ) -> crate::cc::input::actively_scrolling_type::ActivelyScrollingType;
    fn SmoothnessTakesPriority(&self) -> bool;
    fn CurrentBeginFrameArgs(&self) -> &super::begin_frame::BeginFrameArgs;
    fn UseGpuRasterization(&self) -> bool;
    fn RequiresHighResToDraw(&self) -> bool;
    fn global_tile_state(
        &self,
    ) -> &crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority;
    fn IsPendingTree(&self) -> bool;
    fn IsSyncTree(&self) -> bool;
    fn PinchGestureActive(&self) -> bool;
    fn device_viewport_rect_changed(&self) -> bool;
    fn GetDeviceViewport(&self) -> super::geometry::Rect;
    fn PageScaleTransformNode(
        &self,
    ) -> Option<&crate::cc::trees::transform_node::TransformNode<B::Transform>>;
    fn external_page_scale_factor(&self) -> f32;
}
pub trait PictureLayerImplTree<B: PictureLayerImplTypes>:
    super::layer_impl::LayerImplTree<B> + PictureLayerImplReadTree<B>
{
    fn NotifyLayerHasAnimatedImagesChanged(&mut self, layer_id: i32, has_images: bool);
    fn AddLayerNeedingUpdateDiscardableImageMap(&mut self, layer_id: i32, updates: &mut Vec<i32>);
    fn image_animation_controller(
        &mut self,
    ) -> &mut crate::cc::trees::image_animation_controller::ImageAnimationController;
    fn UpdateImageDecodingHints(&mut self, modes: PictureImageDecodingModeMap);
    fn BeginDiscardableImageMapUpdate(&mut self);
    fn EndDiscardableImageMapUpdate(&mut self);
    fn WithDiscardableImageMapUpdater(
        &mut self,
        action: &mut dyn DiscardableImageMapUpdateAction<B>,
    );
    fn RequestImplSideInvalidationForRerasterTiling(&mut self);
    fn NotifyLayerHasPaintWorkletsChanged(&mut self, layer_id: i32, has_worklets: bool);
    fn paint_worklet_tracker(
        &mut self,
    ) -> &mut crate::cc::trees::animated_paint_worklet_tracker::AnimatedPaintWorkletTracker<B>;
}
pub struct PictureLayerSourceContext<'a, B: PictureLayerImplTypes> {
    pub tree: &'a dyn PictureLayerImplReadTree<B>,
}
impl<B: PictureLayerImplTypes> PictureLayerSourceContext<'_, B> {
    pub fn BaseReadContext(&self) -> super::layer_impl::LayerImplReadContext<'_, B> {
        super::layer_impl::LayerImplReadContext { tree: self.tree }
    }
}
pub trait PictureLayerImplServices<B: PictureLayerImplTypes>:
    super::layer_impl::LayerImplServices<B>
{
    fn AllocateTilingSetId(
        &mut self,
    ) -> crate::cc::tiles::raster_tile_priority_queue::PictureLayerTilingSetId;
    fn ComputeRasterTranslateForExternalScale(&self) -> bool;
    fn MissedLCPCandidateImageDeadline(&self, missed: bool);
    fn SetTileIDRef(
        &self,
        tile: super::tile::TileTraceId,
        value: &mut B::LayerTraceValue,
        name: &str,
    );
    fn IsUsingRawDraw(&self) -> bool;
    fn RawDrawTileSizeFactor(&self) -> f64;
}
/// Only the current live tree siblings and exact external Paint/diagnostic
/// capabilities. It is never retained by PictureLayerImpl or one of its children.
pub struct PictureLayerImplContext<'a, B: PictureLayerImplTypes> {
    pub tree: &'a mut dyn PictureLayerImplTree<B>,
    pub services: &'a mut dyn PictureLayerImplServices<B>,
    pub tiling: &'a mut dyn PictureLayerImplTilingEnvironment<B>,
}
impl<B: PictureLayerImplTypes> PictureLayerImplContext<'_, B> {
    pub fn BaseContext(&mut self) -> super::layer_impl::LayerImplContext<'_, B> {
        super::layer_impl::LayerImplContext {
            tree: self.tree,
            services: self.services,
        }
    }
}

// Source dependencies: ui/gfx/geometry/vector2d_f.h:35,45-53,70-73;
// ui/gfx/geometry/vector2d_f.cc:25-27,46-49. Native value primitives only.
impl super::geometry::Vector2dF {
    pub fn SetToMin(&mut self, other: Self) {
        self.set_x(super::geometry::cpp_min(self.x(), other.x()));
        self.set_y(super::geometry::cpp_min(self.y(), other.y()));
    }
    pub fn SetToMax(&mut self, other: Self) {
        self.set_x(super::geometry::cpp_max(self.x(), other.x()));
        self.set_y(super::geometry::cpp_max(self.y(), other.y()));
    }
}

/// These are short borrows of other resident owners. Tile creation/destruction
/// must synchronously enter canonical TileManager; no registry or priority state
/// is retained or mirrored by this capability.
pub trait PictureLayerImplTilingEnvironment<B: PictureLayerImplTypes>:
    super::picture_layer_tiling::PictureLayerTilingQueries<B>
    + super::picture_layer_tiling::PictureLayerTileDestroyer<B>
{
    /// The source Tile destructor synchronously reenters its selected Picture.
    /// This continuation lends only the actual remaining fields for that call.
    fn DestroyTileWithManagerForSelected(
        &mut self,
        manager: super::tile::TileManagerId,
        tile: &mut crate::cc::tiles::tile::Tile<B>,
        tiling: &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
        notification: &mut dyn PictureReleasedTileClient<B>,
    );
    fn TwinLayerRead(
        &self,
        id: crate::cc::layers::picture_layer_impl::PictureLayerImplId,
    ) -> crate::cc::layers::picture_layer_impl::PictureLayerImplReadView<'_, B>;
    fn AllocateTilingId(&mut self) -> super::tile::PictureLayerTilingId;
    fn CreateTileWithManager(
        &mut self,
        info: &crate::cc::tiles::tile::CreateInfo<B::RasterTransform>,
        layer_id: i32,
        source_frame_number: i32,
        flags: i32,
    ) -> Box<crate::cc::tiles::tile::Tile<B>>;
}

pub trait PictureRenderPass<B: PictureLayerImplTypes>:
    super::tile_based_layer_impl::TileBasedRenderPass<B>
{
    fn CreateAndAppendPictureDrawQuad(&mut self) -> B::DrawQuadHandle;
    fn PictureDrawQuadSetNew(
        &mut self,
        quad: B::DrawQuadHandle,
        shared: B::SharedQuadStateHandle,
        geometry: super::geometry::Rect,
        visible: super::geometry::Rect,
        needs_blending: bool,
        texture: super::geometry::RectF,
        nearest: bool,
        quad_content: super::geometry::Rect,
        scale: f32,
        image_animation: std::collections::BTreeMap<PaintImageId, usize>,
        display_list: Option<Arc<<B::RasterSource as PictureRasterSource>::DisplayItemList>>,
        scroll_offsets: super::prioritized_tile::ScrollOffsetMap,
    );
}
pub(crate) trait PictureLayerImplBenchmark<B: PictureLayerImplTypes>:
    super::layer_impl::LayerImplBenchmark<B>
{
    fn RunOnPictureLayer(
        &mut self,
        layer: &mut crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        c: &mut PictureLayerImplContext<'_, B>,
    );
}
impl LCDTextDisallowedReason {
    // Source dependency: cc/raster/lcd_text_disallowed_reason.cc:12-38.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::kNone => "none",
            Self::kSetting => "setting",
            Self::kBackgroundColorNotOpaque => "background-color-not-opaque",
            Self::kContentsNotOpaque => "contents-not-opaque",
            Self::kNonIntegralTranslation => "non-integral-translation",
            Self::kNonIntegralXOffset => "non-integral-x-offset",
            Self::kNonIntegralYOffset => "non-integral-y-offset",
            Self::kWillChangeTransform => "will-change-transform",
            Self::kPixelOrColorEffect => "pixel-or-color-effect",
            Self::kTransformAnimation => "transform-animation",
            Self::kNoText => "no-text",
        }
    }
}

impl<B: PictureLayerImplTypes> super::tile_based_layer_impl::TileBasedLayerContext<B>
    for PictureLayerImplContext<'_, B>
{
    fn BaseContext(&mut self) -> super::layer_impl::LayerImplContext<'_, B> {
        PictureLayerImplContext::BaseContext(self)
    }
}

/// Host facts and live owners consumed by PictureLayerImpl; selected-tree
/// property/viewport/lifecycle fields remain in the separate resident borrow.
/// Methods are native source queries/callbacks, never cached decisions.
pub trait PictureLayerReadHost<B: PictureLayerImplTypes> {
    fn GetMSAASampleCountForRaster(
        &self,
        list:&<<B as PictureLayerTilingTypes>::RasterSource as PictureRasterSource>::DisplayItemList,
        selected: super::layer_tree_impl::LayerTreeId,
        device_scale: &f32,
    ) -> i32;
    fn GetTargetColorParams(
        &self,
        usage: super::layer_tree_host_impl::ContentColorUsage,
        selected: super::layer_tree_impl::LayerTreeId,
        display_colors: &B::DisplayColorSpaces,
    ) -> B::PictureTargetColorParams;
    fn IsSyncTree(&self, id: super::layer_tree_impl::LayerTreeId) -> bool;
    fn TreeAnimationsInVizInVizProcess(&self) -> bool;
    fn IsActiveTree(&self, id: super::layer_tree_impl::LayerTreeId) -> bool;
    fn IsPendingTree(&self, id: super::layer_tree_impl::LayerTreeId) -> bool;
    fn IsRecycleTree(&self, id: super::layer_tree_impl::LayerTreeId) -> bool;
    fn HasPendingTree(&self) -> bool;
    fn settings(&self) -> &crate::cc::trees::layer_tree_settings::LayerTreeSettings;
    fn debug_state(&self) -> &crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState;
    fn raster_caps(&self) -> &crate::cc::trees::raster_capabilities::RasterCapabilities;
    fn IsCCSlimmingEnabled(&self) -> bool;
    fn TreesInVizInClientProcess(&self) -> bool;
    fn IsReadyToActivate(&self) -> bool;
    fn GetActivelyScrollingType(
        &self,
        active_scroll_tree: &crate::cc::trees::property_tree::ScrollTree,
    ) -> crate::cc::input::actively_scrolling_type::ActivelyScrollingType;
    fn ViewportRectForTilePriority(&self) -> super::geometry::Rect;
    fn SmoothnessTakesPriority(&self) -> bool;
    fn CurrentBeginFrameArgs(&self) -> &super::begin_frame::BeginFrameArgs;
    fn UseGpuRasterization(&self) -> bool;
    fn RequiresHighResToDraw(&self) -> bool;
    fn global_tile_state(
        &self,
    ) -> &crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority;
    fn IsPinchGestureActive(&self) -> bool;
    fn external_viewport(&self) -> super::geometry::Rect;
}
pub trait PictureLayerTreeHost<B: PictureLayerImplTypes> {
    fn RequestImplSideInvalidationForRerasterTiling(&mut self);
    fn image_animation_controller(
        &mut self,
    ) -> &mut crate::cc::trees::image_animation_controller::ImageAnimationController;
    fn paint_worklet_tracker(
        &mut self,
    ) -> &mut crate::cc::trees::animated_paint_worklet_tracker::AnimatedPaintWorkletTracker<B>;
    fn UpdateImageDecodingHints(&mut self, modes: PictureImageDecodingModeMap);
}
/// Pure short factory boundary implemented by full and input draw host facets.
/// It cannot retain the callback or any selected/other tree reference.
pub trait PictureLayerContextHost<B: LayerTreeTypes> {
    fn WithPictureContextParts(
        &mut self,
        selected: super::layer_tree_impl::LayerTreeId,
        callback: &mut dyn FnMut(
            &dyn PictureLayerReadHost<B>,
            &mut dyn PictureLayerTreeHost<B>,
            &mut dyn PictureLayerImplServices<B>,
            &mut dyn PictureLayerImplTilingEnvironment<B>,
        ),
    ) where
        B: PictureLayerImplTypes;
}

/// One current native owner action and its post-update live resolution. The
/// same object owns both phases, so Rust never aliases two closures capturing
/// the current PictureLayerImpl or captures an owner in a stored callback.
pub trait DiscardableImageMapUpdateAction<B: PictureLayerImplTypes> {
    fn Run(&mut self, tree: &mut dyn PictureLayerImplTree<B>, updates: &mut Vec<i32>);
    fn Regenerate(&mut self, id: i32, tree: &mut dyn PictureLayerImplTree<B>);
}

/// Source PictureLayerImpl::NotifyTileStateChanged(update_damage=false) during
/// Tile destruction, followed by the exact native Viz read and redraw phase.
pub trait PictureReleasedTileClient<B: PictureLayerImplTypes> {
    fn NotifyReleasedTile(&mut self, tile: &crate::cc::tiles::tile::Tile<B>);
    fn layer(&self) -> &dyn super::layer_tree_host_impl::PictureLayerDisplayRead;
    fn should_batch_updated_tiles(&self) -> bool;
}
