// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/picture_layer_impl.cc:1-50,99-100.
// Rust imports/modules replace the C++ includes and anonymous namespace.
use crate::adapter::geometry::Rect;
// Source: cc/layers/picture_layer_impl.cc:51-53.
const kMaxScaleRatioDuringPinch: f32 = 2.0;
// Source: cc/layers/picture_layer_impl.cc:55-57.
const kSnapToExistingTilingRatio: f32 = 1.2;
// Source: cc/layers/picture_layer_impl.cc:59-65.
const kMaxIdealContentsScale: f32 = 10000.;
// Source: cc/layers/picture_layer_impl.cc:67-70.
const kMinScaleRatioForWillChangeTransform: f32 = 0.25;
// Source: cc/layers/picture_layer_impl.cc:72-79.
const kRatioToAdjustRasterScaleForTransformAnimation: f32 = 1.5;
// Source: cc/layers/picture_layer_impl.cc:81-98.
fn SafeIntersectRects(one: Rect, two: Rect) -> Rect {
    if one.IsEmpty() || two.IsEmpty() {
        return Rect::default();
    }
    let rx = one.x.max(two.x);
    let ry = one.y.max(two.y);
    let rr = (i64::from(one.x) + i64::from(one.width)).min(i64::from(two.x) + i64::from(two.width));
    let rb =
        (i64::from(one.y) + i64::from(one.height)).min(i64::from(two.y) + i64::from(two.height));
    if i64::from(rx) > rr || i64::from(ry) > rb {
        return Rect::default();
    }
    Rect::new(
        rx,
        ry,
        (rr - i64::from(rx)) as i32,
        (rb - i64::from(ry)) as i32,
    )
}
use super::{tile_based_layer_impl::TileBasedLayerImpl, tile_size_calculator::TileSizeCalculator};
use crate::{
    adapter::{
        geometry::{Size, Vector2dF},
        layer_impl::LayerImplContext,
        layer_tree_impl::{LayerTreeId, LayerTreeTypes},
        paint_worklet::PaintWorkletTypes,
        picture_layer_impl::{
            LCDTextDisallowedReason, PaintWorkletRecordMap, PictureLayerImplTypes,
            PictureRasterSource,
        },
        picture_layer_tiling::{PictureLayerTilingClientId, TilingRasterSource},
    },
    cc::{
        base::region::Region,
        tiles::{
            picture_layer_tiling::PictureLayerTiling,
            picture_layer_tiling_set::PictureLayerTilingSet, tile_index::TileIndex,
        },
    },
};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
// Native finite scale keys use numeric < ordering, including equivalent +/-0.
// NaN cannot be a valid std::map comparator key in the source scale domain.
#[derive(Clone, Copy, Debug)]
pub struct ScaleKey(f32);
impl ScaleKey {
    // Source: cc/layers/picture_layer_impl.cc:102-103.
    // Source: cc/layers/picture_layer_impl.h:227-227.
    pub fn new(value: f32) -> Self {
        assert!(!value.is_nan());
        Self(value)
    }
    pub fn value(self) -> f32 {
        self.0
    }
}
impl PartialEq for ScaleKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for ScaleKey {}
impl PartialOrd for ScaleKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ScaleKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("source ordered scale key is not NaN")
    }
}
// Source: cc/layers/picture_layer_impl.h:131-140.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageInvalidationResult {
    kNoImages,
    kNoInvalidation,
    kInvalidated,
}
// Source: cc/layers/picture_layer_impl.h:186-199.
// std::set<TileUpdateIndex> orders only its TileIndex base; update_damage is a
// mutable non-key bool. One BTreeMap entry preserves that exact key/payload
// relationship and permits setting the resident payload without key mutation.
pub type TileUpdateSet = BTreeMap<ScaleKey, BTreeMap<TileIndex, bool>>;
// Native raw twin pointer: allocation identity is validated on every current
// live sibling borrow. It never owns/retains/looks up a copied layer state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PictureLayerImplId {
    pub tree: LayerTreeId,
    pub layer: i32,
    pub stable: i32,
}
// Source: cc/layers/picture_layer_impl.h:41-44,286-398.
// All fields below belong to this resident owner. The state split lends actual
// siblings to a source TilingSet client callback; it is never cloned/projected.
pub struct PictureLayerImpl<B: PictureLayerImplTypes> {
    state_: PictureLayerImplState<B>,
    tilings_: Option<Box<PictureLayerTilingSet<B>>>,
    base_: TileBasedLayerImpl<B, PictureLayerTiling<B>>,
    // C++ pointer identity is stable independently of a Rust owner's move.
    client_identity_: PictureLayerTilingClientId,
    destroyed_: bool,
}
pub struct PictureLayerImplState<B: PictureLayerImplTypes> {
    // Source: cc/layers/picture_layer_impl.h:286-288.
    twin_layer_: Option<PictureLayerImplId>,
    // Source: cc/layers/picture_layer_impl.h:290-311.
    updated_tiles_: TileUpdateSet,
    proposed_tiling_scales_for_deletion_: BTreeSet<ScaleKey>,
    should_batch_updated_tiles_: bool,
    // Source: cc/layers/picture_layer_impl.h:315-322.
    raster_source_: Option<Arc<B::RasterSource>>,
    invalidation_: Region<<B as LayerTreeTypes>::RegionBackend>,
    discardable_image_map_: Option<Arc<B::DiscardableImageMap>>,
    pending_raster_source_: Option<Arc<B::RasterSource>>,
    pending_invalidation_: Region<<B as LayerTreeTypes>::RegionBackend>,
    // Source: cc/layers/picture_layer_impl.h:324-341.
    ideal_page_scale_: f32,
    ideal_device_scale_: f32,
    ideal_source_scale_: Vector2dF,
    ideal_contents_scale_: Vector2dF,
    raster_page_scale_: f32,
    raster_device_scale_: f32,
    raster_source_scale_: Vector2dF,
    raster_contents_scale_: Vector2dF,
    // Source: cc/layers/picture_layer_impl.h:354-369.
    was_screen_space_transform_animating_: bool,
    raster_source_size_changed_: bool,
    directly_composited_image_default_raster_scale_changed_: bool,
    has_animated_image_update_rect_: bool,
    has_non_animated_image_update_rect_: bool,
    lcd_text_disallowed_reason_: LCDTextDisallowedReason,
    // Source: cc/layers/picture_layer_impl.h:375-398.
    directly_composited_image_default_raster_scale_: f32,
    viewport_rect_for_tile_priority_in_content_space_: Rect,
    gpu_raster_max_texture_size_: Size,
    paint_worklet_records_: PaintWorkletRecordMap<<B as PaintWorkletTypes>::Input, B::PaintRecord>,
    tile_size_calculator_: TileSizeCalculator,
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    /// Pure projection of this same resident inherited base for the native
    /// most-derived layer wrapper. It never resolves an owner or copies state.
    pub(crate) fn base(&self) -> &super::layer_impl::LayerImpl<B> {
        self.base_.base()
    }
    pub(crate) fn base_mut(&mut self) -> &mut super::layer_impl::LayerImpl<B> {
        self.base_.base_mut()
    }

    /// Inherited native TileBasedLayerImpl setter, projected on this same base.
    pub(crate) fn SetIsBackdropFilterMask(&mut self, value: bool, c: &mut LayerImplContext<'_, B>) {
        self.base_.SetIsBackdropFilterMask(value, c)
    }
    // Source: cc/layers/picture_layer_impl.h:90-96.
    pub fn set_gpu_raster_max_texture_size(&mut self, size: Size, c: &mut LayerImplContext<'_, B>) {
        if self.state_.gpu_raster_max_texture_size_ == size {
            return;
        }
        self.state_.gpu_raster_max_texture_size_ = size;
        self.base_.base_mut().SetNeedsPushProperties(
            super::layer_impl::LayerImpl::<B>::kChangedGeneralProperty,
            c,
        );
    }
    // Source: cc/layers/picture_layer_impl.h:98-100.
    pub fn gpu_raster_max_texture_size(&self) -> Size {
        self.state_.gpu_raster_max_texture_size_
    }
    // Source: cc/layers/picture_layer_impl.h:148-150.
    pub fn can_use_lcd_text(&self) -> bool {
        self.state_.lcd_text_disallowed_reason_ == LCDTextDisallowedReason::kNone
    }
    // Source: cc/layers/picture_layer_impl.h:151-153.
    pub fn lcd_text_disallowed_reason(&self) -> LCDTextDisallowedReason {
        self.state_.lcd_text_disallowed_reason_
    }
    // Source: cc/layers/picture_layer_impl.h:156-156.
    pub fn InvalidationForTesting(&self) -> &Region<<B as LayerTreeTypes>::RegionBackend> {
        &self.state_.invalidation_
    }
    // Source: cc/layers/picture_layer_impl.h:165-167.
    pub fn GetPaintWorkletRecordMap(
        &self,
    ) -> &PaintWorkletRecordMap<<B as PaintWorkletTypes>::Input, B::PaintRecord> {
        &self.state_.paint_worklet_records_
    }
    // Source: cc/layers/picture_layer_impl.h:176-178.
    pub fn set_has_non_animated_image_update_rect(&mut self) {
        self.state_.has_non_animated_image_update_rect_ = true;
    }
    // Source: cc/layers/picture_layer_impl.h:179-181.
    pub fn has_animated_image_update_rect(&self) -> bool {
        self.state_.has_animated_image_update_rect_
    }
    // Source: cc/layers/picture_layer_impl.h:182-184.
    pub fn has_non_animated_image_update_rect(&self) -> bool {
        self.state_.has_non_animated_image_update_rect_
    }
    // Source: cc/layers/picture_layer_impl.h:211-211.
    pub fn set_should_batch_updated_tiles(&mut self) {
        self.state_.should_batch_updated_tiles_ = true;
    }
    // Source: cc/layers/picture_layer_impl.h:213-215.
    pub fn should_batch_updated_tiles(&self) -> bool {
        self.state_.should_batch_updated_tiles_
    }
    // Source: cc/layers/picture_layer_impl.h:218-220.
    pub fn SetRasterContentsScaleForTesting(&mut self, scale: f32) {
        self.state_.raster_contents_scale_ = Vector2dF::new(scale, scale);
    }
    // Source: cc/layers/picture_layer_impl.h:222-222.
    pub fn ClearLastAppendsQuadsScalesForTesting(&mut self) {
        self.base_.ClearLastAppendQuadsScales();
    }
    // Source: cc/layers/picture_layer_impl.h:343-345.
    pub fn ideal_source_scale_key(&self) -> f32 {
        crate::adapter::geometry::cpp_max(
            self.state_.ideal_source_scale_.x(),
            self.state_.ideal_source_scale_.y(),
        )
    }
    // Source: cc/layers/picture_layer_impl.h:347-349.
    pub fn raster_source_scale_key(&self) -> f32 {
        crate::adapter::geometry::cpp_max(
            self.state_.raster_source_scale_.x(),
            self.state_.raster_source_scale_.y(),
        )
    }
    // Source: cc/layers/picture_layer_impl.h:350-352.
    pub fn raster_contents_scale_key(&self) -> f32 {
        crate::adapter::geometry::cpp_max(
            self.state_.raster_contents_scale_.x(),
            self.state_.raster_contents_scale_.y(),
        )
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.cc:111-113.
    // Source: cc/layers/picture_layer_impl.h:55-55.
    pub fn GetLayerType(&self) -> crate::adapter::layer_impl::LayerType {
        crate::adapter::layer_impl::LayerType::kPicture
    }
    // Source: cc/layers/picture_layer_impl.h:122-122.
    pub(crate) fn picture_layer_tiling_set(&self) -> &PictureLayerTilingSet<B> {
        self.tilings_.as_deref().expect("native live tiling set")
    }
    pub(crate) fn picture_layer_tiling_set_mut(&mut self) -> &mut PictureLayerTilingSet<B> {
        self.tilings_
            .as_deref_mut()
            .expect("native live tiling set")
    }
    // Source: cc/layers/picture_layer_impl.h:129-129.
    pub fn GetRasterSource(&self) -> Option<&B::RasterSource> {
        self.state_.raster_source_.as_deref()
    }
    // Source: cc/layers/picture_layer_impl.cc:522-527.
    // Source: cc/layers/picture_layer_impl.h:104-105.
    pub fn StageNewRasterSourceForCommit(
        &mut self,
        source: Arc<B::RasterSource>,
        invalidation: Region<<B as LayerTreeTypes>::RegionBackend>,
    ) {
        self.state_.pending_raster_source_ = Some(source);
        self.state_.pending_invalidation_ = invalidation;
    }
    // Source: cc/layers/picture_layer_impl.cc:818-822.
    // Source: cc/layers/picture_layer_impl.h:61-61.
    pub fn ResetChangeTracking(&mut self) {
        self.base_.ResetChangeTracking();
        self.state_.has_animated_image_update_rect_ = false;
        self.state_.has_non_animated_image_update_rect_ = false;
    }
    // Source: cc/layers/picture_layer_impl.cc:1875-1877.
    // Source: cc/layers/picture_layer_impl.h:116-116.
    pub fn GPUMemoryUsageInBytes(&self) -> usize {
        self.picture_layer_tiling_set().GPUMemoryUsageInBytes()
    }
    // Source: cc/layers/picture_layer_impl.cc:907-909.
    // Source: cc/layers/picture_layer_impl.h:208-208.
    pub fn IsDirectlyCompositedImage(&self) -> bool {
        IsDirectImageForState(&self.state_)
    }
    // Source: cc/layers/picture_layer_impl.cc:911-913.
    // Source: cc/layers/picture_layer_impl.h:209-209.
    pub fn RecordedBounds(&self) -> Rect {
        RecordedBoundsForState(&self.state_)
    }
    // Source: cc/layers/picture_layer_impl.cc:1637-1645.
    // Source: cc/layers/picture_layer_impl.h:241-241.
    pub fn MinimumContentsScale(&self) -> f32 {
        MinimumContentsScaleForState(&self.state_)
    }
    // Source: cc/layers/picture_layer_impl.cc:1669-1675.
    // Source: cc/layers/picture_layer_impl.h:62-62.
    pub fn ResetRasterScale(&mut self) {
        self.state_.raster_page_scale_ = 0.;
        self.state_.raster_device_scale_ = 0.;
        self.state_.raster_source_scale_ = Vector2dF::new(0., 0.);
        self.state_.raster_contents_scale_ = Vector2dF::new(0., 0.);
        self.state_.directly_composited_image_default_raster_scale_ = 0.;
    }
    // Source: cc/layers/picture_layer_impl.cc:1714-1717.
    // Source: cc/layers/picture_layer_impl.h:271-271.
    pub fn MaximumTilingContentsScale(&self) -> f32 {
        MaximumTilingContentsScaleForParts(&self.state_, self.picture_layer_tiling_set())
    }
    // Source: cc/layers/picture_layer_impl.cc:2030-2040.
    // Source: cc/layers/picture_layer_impl.h:200-200.
    pub fn TakeUpdatedTiles(&mut self) -> TileUpdateSet {
        let mut updates = TileUpdateSet::new();
        std::mem::swap(&mut updates, &mut self.state_.updated_tiles_);
        self.state_.should_batch_updated_tiles_ = false;
        updates
    }
    // Source: cc/layers/picture_layer_impl.cc:2042-2047.
    // Source: cc/layers/picture_layer_impl.h:202-202.
    pub fn TakeProposedTilingScalesForDeletion(&mut self) -> Vec<f32> {
        let updates = self
            .state_
            .proposed_tiling_scales_for_deletion_
            .iter()
            .map(|key| key.value())
            .collect();
        self.state_.proposed_tiling_scales_for_deletion_.clear();
        updates
    }
    // Source: cc/layers/picture_layer_impl.cc:2049-2074.
    // Source: cc/layers/picture_layer_impl.h:206-206.
    pub fn TakeAllTiles(&mut self, c: &LayerImplContext<'_, B>) -> TileUpdateSet {
        debug_assert!(c.tree.TreesInVizInClientProcess());
        debug_assert!(c.tree.IsActiveTree());
        self.state_.updated_tiles_.clear();
        let mut updates = TileUpdateSet::new();
        for tiling in self.picture_layer_tiling_set_mut().TilingsMutable() {
            let mut iter =
                crate::cc::tiles::picture_layer_tiling::TileIterator::new(tiling);
            while !iter.AtEnd() {
                let tile = iter.GetCurrent().expect("native live Tile* entry");
                updates
                    .entry(ScaleKey::new(tile.contents_scale_key()))
                    .or_default()
                    .entry(TileIndex::new(tile.tiling_i_index(), tile.tiling_j_index()))
                    .or_insert(false);
                iter.Next();
            }
        }
        self.state_.should_batch_updated_tiles_ = false;
        updates
    }
    // Source: cc/layers/picture_layer_impl.cc:2087-2096.
    // Source: cc/layers/picture_layer_impl.h:69-69.
    pub fn GetDamageReasons(
        &self,
        c: &LayerImplContext<'_, B>,
    ) -> crate::cc::trees::damage_reason::DamageReasonSet {
        use crate::cc::trees::damage_reason::DamageReason;
        let mut reasons = self.base_.base().GetDamageReasonsFromLayerPropertyChange(c);
        if self.state_.has_animated_image_update_rect_ {
            reasons.Put(DamageReason::kAnimatedImage);
        }
        if self.state_.has_non_animated_image_update_rect_ || !self.base_.GetDamageRect().IsEmpty()
        {
            reasons.Put(DamageReason::kUntracked);
        }
        reasons
    }
    // Source: cc/layers/picture_layer_impl.cc:2098-2103.
    // Source: cc/layers/picture_layer_impl.h:102-102.
    pub fn GetMaximumContentsScaleForUseInAppendQuads(&self) -> f32 {
        MaximumAppendScaleForParts(&self.state_, self.picture_layer_tiling_set())
    }
    // Source: cc/layers/picture_layer_impl.cc:2105-2115.
    // Source: cc/layers/picture_layer_impl.h:428-429.
    pub fn GetTilingResolutionForDebugBorders(
        &self,
        tiling: &PictureLayerTiling<B>,
    ) -> super::tile_based_layer_impl::TilingResolution {
        DebugTilingResolutionForState(&self.state_, tiling)
    }
    // Source: cc/layers/picture_layer_impl.cc:2117-2122.
    // Source: cc/layers/picture_layer_impl.h:409-412.
    pub fn Cover(
        &self,
        rect: Rect,
        scale: f32,
        ideal: f32,
    ) -> crate::cc::tiles::tiling_set_coverage_iterator::TilingSetCoverageIterator<
        '_,
        PictureLayerTiling<B>,
        <B as LayerTreeTypes>::RegionBackend,
    > {
        self.picture_layer_tiling_set().Cover(rect, scale, ideal)
    }
    // Source: cc/layers/picture_layer_impl.cc:2124-2126.
    // Source: cc/layers/picture_layer_impl.h:346-346.
    pub fn GetIdealContentsScaleKey(&self) -> f32 {
        IdealContentsScaleKeyForState(&self.state_)
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.h:45-48.
    pub(crate) fn Create(
        tree: LayerTreeId,
        id: i32,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> Box<B::Layer> {
        Box::new(B::WrapPictureLayerImpl(Self::new(tree, id, c)))
    }
    // Source: cc/layers/picture_layer_impl.cc:102-103.
    // Source: cc/layers/picture_layer_impl.h:227-227.
    fn new(
        tree: LayerTreeId,
        id: i32,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> Self {
        let base = TileBasedLayerImpl::new(tree, id, &mut c.BaseContext());
        // The source's client pointer denotes this native allocation. Its
        // already-global stable layer ID supplies a nonowning identity, not a
        // second owner index or a new mutable-state projection.
        let client =
            PictureLayerTilingClientId(base.base().stable_id_for_shared_quad_state() as u64);
        let updated_tiles = TileUpdateSet::new();
        let proposed_scales = BTreeSet::new();
        let tilings = Self::CreatePictureLayerTilingSetForBase(&base, client, c);
        let state = PictureLayerImplState {
            twin_layer_: None,
            updated_tiles_: updated_tiles,
            proposed_tiling_scales_for_deletion_: proposed_scales,
            should_batch_updated_tiles_: true,
            raster_source_: None,
            invalidation_: Region::default(),
            discardable_image_map_: None,
            pending_raster_source_: None,
            pending_invalidation_: Region::default(),
            ideal_page_scale_: 0.,
            ideal_device_scale_: 0.,
            ideal_source_scale_: Vector2dF::default(),
            ideal_contents_scale_: Vector2dF::default(),
            raster_page_scale_: 0.,
            raster_device_scale_: 0.,
            raster_source_scale_: Vector2dF::default(),
            raster_contents_scale_: Vector2dF::default(),
            was_screen_space_transform_animating_: false,
            raster_source_size_changed_: false,
            directly_composited_image_default_raster_scale_changed_: false,
            has_animated_image_update_rect_: false,
            has_non_animated_image_update_rect_: false,
            lcd_text_disallowed_reason_: LCDTextDisallowedReason::kNoText,
            directly_composited_image_default_raster_scale_: 0.,
            viewport_rect_for_tile_priority_in_content_space_: Rect::default(),
            gpu_raster_max_texture_size_: Size::default(),
            paint_worklet_records_: PaintWorkletRecordMap::default(),
            tile_size_calculator_: TileSizeCalculator::new(
                c.services.IsUsingRawDraw(),
                c.services.RawDrawTileSizeFactor(),
            ),
        };
        Self {
            state_: state,
            tilings_: Some(tilings),
            base_: base,
            client_identity_: client,
            destroyed_: false,
        }
    }
    // Source: cc/layers/picture_layer_impl.h:273;
    // Source: cc/layers/picture_layer_impl.cc:1719-1730.
    // Same unique body serves the constructor's native member initializer and
    // later RecreateTileResources; it borrows the actual base rather than a
    // partially constructed owner or copied settings/capabilities.
    fn CreatePictureLayerTilingSetForBase(
        base: &TileBasedLayerImpl<B, PictureLayerTiling<B>>,
        client: PictureLayerTilingClientId,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> Box<PictureLayerTilingSet<B>> {
        let identity = c.services.AllocateTilingSetId();
        let settings = c.tree.settings();
        // Borrow-only expansion of the canonical IsActive getter: validate the
        // same native tree, then read its flag while settings stay resident.
        assert_eq!(base.base().tree_id(), c.tree.id());
        let active = c.tree.IsActiveTree();
        use crate::cc::tiles::tile_priority::WhichTree;
        PictureLayerTilingSet::Create(
            identity,
            if active {
                WhichTree::ACTIVE_TREE
            } else {
                WhichTree::PENDING_TREE
            },
            client,
            settings.tiling_interest_area_padding,
            if c.tree.raster_caps().use_gpu_rasterization {
                settings.gpu_rasterization_skewport_target_time_in_seconds
            } else {
                settings.skewport_target_time_in_seconds
            },
            settings.skewport_extrapolation_limit_in_screen_pixels,
            settings.max_preraster_distance_in_screen_pixels as f32,
        )
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // The source static_cast targets the actual most-derived allocation; this
    // typed target keeps its resident base and derived fields together.
    // Source: cc/layers/picture_layer_impl.cc:120-150.
    // Source: cc/layers/picture_layer_impl.h:58-58.
    pub fn CopyPropertiesTo(
        &self,
        target: &mut Self,
        source: &crate::adapter::layer_impl::LayerImplReadContext<'_, B>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        self.base_
            .base()
            .CopyPropertiesTo(target.base_.base_mut(), source, c);
        let changed = self
            .base_
            .base()
            .GetChangeFlag(super::layer_impl::LayerImpl::<B>::kChangedGeneralProperty);
        if changed {
            target.state_.has_animated_image_update_rect_ =
                self.state_.has_animated_image_update_rect_;
            target.state_.has_non_animated_image_update_rect_ =
                self.state_.has_non_animated_image_update_rect_;
            target
                .base_
                .SetIsBackdropFilterMask(self.base_.is_backdrop_filter_mask(), c);
            debug_assert!(
                self.base_.solid_color().is_none()
                    || self.picture_layer_tiling_set().num_tilings() == 0
            );
            debug_assert!(self.picture_layer_tiling_set().num_tilings() <= 1);
            target.set_gpu_raster_max_texture_size(self.state_.gpu_raster_max_texture_size_, c);
            target.state_.raster_page_scale_ = self.state_.raster_page_scale_;
            target.state_.raster_device_scale_ = self.state_.raster_device_scale_;
            target.state_.raster_source_scale_ = self.state_.raster_source_scale_;
            target.state_.raster_contents_scale_ = self.state_.raster_contents_scale_;
            target.state_.lcd_text_disallowed_reason_ = self.state_.lcd_text_disallowed_reason_;
        }
    }
}
/// The source non-owning twin pointer borrows the actual sibling allocation for
/// this call. The root's existing tree owner resolves and validates its identity.
pub(crate) trait PictureLayerTwinAccess<B: PictureLayerImplTypes> {
    fn TwinMutable(&mut self, id: PictureLayerImplId) -> &mut PictureLayerImpl<B>;
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Owner-controlled native destructor: the root lends only real twin/manager
    // siblings, never a fabricated client or a retained parent callback. The
    // source implicit member dtors run before the canonical base dtor.
    // Source: cc/layers/picture_layer_impl.cc:105-109.
    // Source: cc/layers/picture_layer_impl.h:50-50.
    pub(crate) fn Destroy(
        &mut self,
        twins: &mut dyn PictureLayerTwinAccess<B>,
        tiles: &mut dyn crate::adapter::picture_layer_tiling::PictureLayerTileDestroyer<B>,
    ) {
        assert!(
            !self.destroyed_,
            "native PictureLayerImpl destructor runs once"
        );
        if let Some(id) = self.state_.twin_layer_ {
            let twin = twins.TwinMutable(id);
            assert_eq!(twin.identity(), id, "resident twin allocation identity");
            twin.state_.twin_layer_ = None;
            self.state_.twin_layer_ = None;
        }
        self.DestroyAfterTwinDetached(tiles);
    }
    /// Continuation of the native destructor after the owning tree has
    /// disconnected the other-tree raw twin pointer.
    pub(crate) fn DestroyAfterTwinDetached(
        &mut self,
        tiles: &mut dyn crate::adapter::picture_layer_tiling::PictureLayerTileDestroyer<B>,
    ) {
        assert!(
            self.state_.twin_layer_.is_none(),
            "native twin pointers disconnect before member destruction"
        );
        // Rust fields otherwise drop in declaration order. Explicit release
        // preserves C++ reverse member destruction for these owning fields.
        drop(std::mem::take(&mut self.state_.paint_worklet_records_));
        self.state_.pending_invalidation_.Clear();
        drop(self.state_.pending_raster_source_.take());
        drop(self.state_.discardable_image_map_.take());
        self.state_.invalidation_.Clear();
        drop(self.state_.raster_source_.take());
        self.tilings_
            .as_mut()
            .expect("native live tilings")
            .Destroy(tiles);
        drop(self.tilings_.take());
        drop(std::mem::take(
            &mut self.state_.proposed_tiling_scales_for_deletion_,
        ));
        drop(std::mem::take(&mut self.state_.updated_tiles_));
        self.destroyed_ = true;
    }
    pub(crate) fn TwinForDestruction(&self) -> Option<PictureLayerImplId> {
        self.state_.twin_layer_
    }
    pub(crate) fn ClearTwinForDestruction(&mut self) -> PictureLayerImplId {
        self.state_
            .twin_layer_
            .take()
            .expect("native symmetric PictureLayerImpl twin")
    }
    pub fn identity(&self) -> PictureLayerImplId {
        PictureLayerImplId {
            tree: self.base_.base().tree_id(),
            layer: self.base_.base().id(),
            stable: self.base_.base().stable_id_for_shared_quad_state(),
        }
    }
}
impl<B: PictureLayerImplTypes> Drop for PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.cc:105-109. The live manager action
    // belongs to Destroy above; Drop cannot call a parent or invent a context.
    fn drop(&mut self) {
        debug_assert!(
            self.destroyed_,
            "owning tree must Destroy PictureLayerImpl before releasing it"
        );
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.cc:824-826.
    // Source: cc/layers/picture_layer_impl.h:63-63.
    pub fn DidBeginTracing(&self) {
        self.state_
            .raster_source_
            .as_ref()
            .expect("source nonnull raster")
            .DidBeginTracing();
    }
    // Source: cc/layers/picture_layer_impl.cc:1001-1022.
    // Source: cc/layers/picture_layer_impl.h:261-261.
    pub fn UpdateDirectlyCompositedImageFromRasterSource(
        &mut self,
        c: &mut LayerImplContext<'_, B>,
    ) {
        let mut new_default = 0.;
        let mut new_nearest = false;
        if let Some(info) = self
            .state_
            .raster_source_
            .as_ref()
            .expect("source nonnull raster")
            .directly_composited_image_info()
        {
            new_default = super::layer_impl::LayerImpl::<B>::GetPreferredRasterScale(
                info.default_raster_scale,
            );
            new_nearest = info.nearest_neighbor;
        }
        self.state_
            .directly_composited_image_default_raster_scale_changed_ =
            new_default != self.state_.directly_composited_image_default_raster_scale_;
        if new_nearest != self.base_.GetNearestNeighbor()
            || self
                .state_
                .directly_composited_image_default_raster_scale_changed_
        {
            self.state_.directly_composited_image_default_raster_scale_ = new_default;
            self.base_.SetNearestNeighbor(new_nearest);
            self.base_.base_mut().NoteLayerPropertyChanged(c);
        }
    }
    // Source: cc/layers/picture_layer_impl.cc:1024-1046.
    // Source: cc/layers/picture_layer_impl.h:245-245.
    pub fn ShouldDirectlyCompositeImage(&self, scale: f32, c: &LayerImplContext<'_, B>) -> bool {
        if scale < 0.1 {
            return true;
        }
        let bounds = self.base_.base().bounds(c);
        let layer =
            crate::adapter::geometry::SizeF::new(bounds.width() as f32, bounds.height() as f32);
        let scaled = crate::adapter::geometry::RectF::new(
            0.,
            0.,
            layer.width() * scale,
            layer.height() * scale,
        );
        let enclosed = crate::adapter::geometry::ToEnclosingRect(scaled);
        let content = crate::adapter::geometry::RectF::new(
            enclosed.x as f32 / scale,
            enclosed.y as f32 / scale,
            enclosed.width as f32 / scale,
            enclosed.height as f32 / scale,
        );
        (layer.width() - content.width()).abs() < 1.
            && (layer.height() - content.height()).abs() < 1.
    }
    // Source: cc/layers/picture_layer_impl.cc:1048-1090.
    // Source: cc/layers/picture_layer_impl.h:252-252.
    pub fn CalculateDirectlyCompositedImageRasterScale(&self) -> f32 {
        debug_assert!(self.IsDirectlyCompositedImage());
        let mut adjusted = if self
            .state_
            .directly_composited_image_default_raster_scale_changed_
        {
            self.state_.directly_composited_image_default_raster_scale_
        } else {
            self.raster_source_scale_key()
        };
        let max = crate::adapter::geometry::cpp_max(
            self.state_.directly_composited_image_default_raster_scale_,
            self.MinimumContentsScale(),
        );
        let min = self.MinimumContentsScale();
        let ideal = self.ideal_source_scale_key().clamp(min, max);
        const kFarAwayFactor: f32 = 32.;
        if adjusted < ideal / kFarAwayFactor {
            adjusted = ideal;
        } else if adjusted > ideal * kFarAwayFactor {
            adjusted = ideal;
        } else {
            while adjusted < ideal {
                adjusted *= 2.;
            }
            while adjusted >= 2. * ideal {
                adjusted /= 2.;
            }
        }
        adjusted = adjusted.clamp(min, max);
        adjusted
    }
    // Source: cc/layers/picture_layer_impl.cc:1647-1667.
    // Source: cc/layers/picture_layer_impl.h:242-242.
    pub fn MaximumContentsScale(&self, c: &LayerImplContext<'_, B>) -> f32 {
        self.MaximumContentsScaleForTree(c.tree)
    }
    fn MaximumContentsScaleForTree(
        &self,
        tree: &dyn crate::adapter::layer_impl::LayerImplReadTree<B>,
    ) -> f32 {
        MaximumContentsScaleForBase(&self.base_, tree)
    }
    // Source: cc/layers/picture_layer_impl.cc:1677-1692.
    // Source: cc/layers/picture_layer_impl.h:120-120.
    pub fn CanHaveTilings(&self, c: &LayerImplContext<'_, B>) -> bool {
        self.CanHaveTilingsForTree(c.tree)
    }
    fn CanHaveTilingsForTree(
        &self,
        tree: &dyn crate::adapter::layer_impl::LayerImplReadTree<B>,
    ) -> bool {
        CanHaveTilingsForParts(&self.base_, &self.state_, tree)
    }
    // Source: cc/layers/picture_layer_impl.cc:1694-1712.
    // Source: cc/layers/picture_layer_impl.h:263-263.
    pub fn SanityCheckTilingState(&self, c: &LayerImplContext<'_, B>) {
        SanityCheckTilingStateForParts(
            &self.base_,
            &self.state_,
            self.picture_layer_tiling_set(),
            c,
        )
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.cc:699-704.
    // Source: cc/layers/picture_layer_impl.h:285-285.
    pub fn AffectedByWillChangeTransformHint(&self, c: &LayerImplContext<'_, B>) -> bool {
        let id = self.base_.base().transform_tree_index();
        id != crate::cc::trees::property_ids::kInvalidPropertyNodeId
            && c.tree
                .property_trees()
                .transform_tree()
                .Node(id)
                .node_or_ancestors_will_change_transform
    }
    // Source: cc/layers/picture_layer_impl.cc:1561-1582.
    // Source: cc/layers/picture_layer_impl.h:237-237.
    pub fn MinimumRasterContentsScaleForWillChangeTransform(
        &self,
        c: &LayerImplContext<'_, B>,
    ) -> f32 {
        debug_assert!(self.AffectedByWillChangeTransformHint(c));
        let native = self.state_.ideal_device_scale_ * self.state_.ideal_page_scale_;
        let ideal = self.GetIdealContentsScaleKey();
        if ideal < native * kMinScaleRatioForWillChangeTransform {
            return ideal * kMinScaleRatioForWillChangeTransform;
        }
        native
    }
    // Source: cc/layers/picture_layer_impl.cc:1234-1363.
    // Source: cc/layers/picture_layer_impl.h:233-233.
    pub(crate) fn ShouldAdjustRasterScale(
        &self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> bool {
        use crate::adapter::geometry::{cpp_max, cpp_min};
        if self.state_.raster_contents_scale_.x() == 0.
            || self.state_.raster_contents_scale_.y() == 0.
        {
            return true;
        }
        if self.state_.raster_source_size_changed_ {
            return true;
        }
        if self.IsDirectlyCompositedImage() {
            if self
                .state_
                .directly_composited_image_default_raster_scale_changed_
            {
                return true;
            }
            let max = cpp_max(
                self.state_.directly_composited_image_default_raster_scale_,
                self.MinimumContentsScale(),
            );
            if self.raster_source_scale_key() < cpp_min(self.ideal_source_scale_key(), max) {
                return true;
            }
            if self.raster_source_scale_key() > 4. * self.ideal_source_scale_key() {
                return true;
            }
            return false;
        }
        if self.state_.was_screen_space_transform_animating_
            != self
                .base_
                .base()
                .draw_properties()
                .screen_space_transform_is_animating
        {
            if self
                .base_
                .base()
                .draw_properties()
                .screen_space_transform_is_animating
            {
                let maximum = c
                    .tree
                    .property_trees_mut()
                    .MaximumAnimationToScreenScale(self.base_.base().transform_tree_index());
                if maximum != self.state_.raster_contents_scale_.x()
                    || maximum != self.state_.raster_contents_scale_.y()
                {
                    return true;
                }
            } else if !self.AffectedByWillChangeTransformHint(&c.BaseContext()) {
                return true;
            }
        }
        let pinching = c.tree.PinchGestureActive();
        if pinching && self.state_.raster_page_scale_ != 0. {
            let ratio = self.state_.ideal_page_scale_ / self.state_.raster_page_scale_;
            if self.state_.raster_page_scale_ > self.state_.ideal_page_scale_
                || ratio > kMaxScaleRatioDuringPinch
            {
                return true;
            }
        }
        if !pinching && self.state_.raster_page_scale_ != self.state_.ideal_page_scale_ {
            return true;
        }
        if self.state_.raster_device_scale_ != self.state_.ideal_device_scale_ {
            return true;
        }
        let max = self.MaximumContentsScale(&c.BaseContext());
        if self.state_.raster_contents_scale_.x() > max
            || self.state_.raster_contents_scale_.y() > max
        {
            return true;
        }
        let min = self.MinimumContentsScale();
        if self.state_.raster_contents_scale_.x() < min
            || self.state_.raster_contents_scale_.y() < min
        {
            return true;
        }
        if self
            .base_
            .base()
            .draw_properties()
            .screen_space_transform_is_animating
        {
            if c.tree.device_viewport_rect_changed() {
                return true;
            }
            if self.state_.ideal_contents_scale_.x()
                > self.state_.raster_contents_scale_.x()
                    * kRatioToAdjustRasterScaleForTransformAnimation
                || self.state_.ideal_contents_scale_.y()
                    > self.state_.raster_contents_scale_.y()
                        * kRatioToAdjustRasterScaleForTransformAnimation
            {
                let trees = c.tree.property_trees_mut();
                let id = self.base_.base().transform_tree_index();
                if trees.AnimationScaleCacheIsInvalid(id)
                    || !trees.AnimationAffectedByInvalidScale(id)
                {
                    return true;
                }
            }
            return false;
        }
        if self.state_.raster_source_scale_ == self.state_.ideal_source_scale_ {
            return false;
        }
        if self.AffectedByWillChangeTransformHint(&c.BaseContext()) {
            let min = self.MinimumRasterContentsScaleForWillChangeTransform(&c.BaseContext());
            if self.state_.raster_contents_scale_.x() >= min
                && self.state_.raster_contents_scale_.y() >= min
            {
                return false;
            }
        }
        true
    }
    // Source: cc/layers/picture_layer_impl.cc:1365-1445.
    // Source: cc/layers/picture_layer_impl.h:234-234.
    pub(crate) fn RecalculateRasterScales(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        use crate::adapter::geometry::{cpp_max, AxisTransform2d};
        if self.IsDirectlyCompositedImage() {
            let scale = self.CalculateDirectlyCompositedImageRasterScale();
            self.state_
                .directly_composited_image_default_raster_scale_changed_ = false;
            if self.ShouldDirectlyCompositeImage(scale, &c.BaseContext()) {
                self.state_.raster_source_scale_ = Vector2dF::new(scale, scale);
                self.state_.raster_page_scale_ = 1.;
                self.state_.raster_device_scale_ = 1.;
                self.state_.raster_contents_scale_ = self.state_.raster_source_scale_;
                return;
            }
            self.state_.directly_composited_image_default_raster_scale_ = 0.;
        }
        let old_contents = self.state_.raster_contents_scale_;
        let old_page = self.state_.raster_page_scale_;
        let mut preserved = old_contents;
        self.state_.raster_device_scale_ = self.state_.ideal_device_scale_;
        self.state_.raster_page_scale_ = self.state_.ideal_page_scale_;
        self.state_.raster_source_scale_ = self.state_.ideal_source_scale_;
        self.state_.raster_contents_scale_ = self.state_.ideal_contents_scale_;
        let pinching = c.tree.PinchGestureActive();
        if pinching && !old_contents.IsZero() {
            let zooming_out = old_page > self.state_.ideal_page_scale_;
            let mut desired = cpp_max(old_contents.x(), old_contents.y());
            let ideal = self.GetIdealContentsScaleKey();
            if zooming_out {
                while desired > ideal {
                    desired /= kMaxScaleRatioDuringPinch;
                }
            } else {
                while desired < ideal {
                    desired *= kMaxScaleRatioDuringPinch;
                }
            }
            if let Some(tiling) = self
                .picture_layer_tiling_set()
                .FindTilingWithNearestScaleKeyRef(desired, kSnapToExistingTilingRatio)
            {
                self.state_.raster_contents_scale_ = *tiling.raster_transform().scale();
            } else {
                self.state_.raster_contents_scale_ = old_contents;
                let factor = desired / self.raster_contents_scale_key();
                self.state_.raster_contents_scale_.Scale(factor);
            }
            preserved = self.state_.raster_contents_scale_;
            self.state_.raster_page_scale_ = cpp_max(
                self.state_.raster_contents_scale_.x() / self.state_.raster_source_scale_.x(),
                self.state_.raster_contents_scale_.y() / self.state_.raster_source_scale_.y(),
            ) / self.state_.raster_device_scale_;
        }
        if self
            .base_
            .base()
            .draw_properties()
            .screen_space_transform_is_animating
        {
            self.AdjustRasterScaleForTransformAnimation(preserved, c);
        }
        if self.AffectedByWillChangeTransformHint(&c.BaseContext()) {
            let min = self.MinimumRasterContentsScaleForWillChangeTransform(&c.BaseContext());
            self.state_
                .raster_contents_scale_
                .SetToMax(Vector2dF::new(min, min));
        }
        let min = self.MinimumContentsScale();
        let max = self.MaximumContentsScale(&c.BaseContext());
        self.state_
            .raster_contents_scale_
            .SetToMax(Vector2dF::new(min, min));
        self.state_
            .raster_contents_scale_
            .SetToMin(Vector2dF::new(max, max));
        debug_assert!(self.state_.raster_contents_scale_.x() >= min);
        debug_assert!(self.state_.raster_contents_scale_.y() >= min);
        debug_assert!(self.state_.raster_contents_scale_.x() <= max);
        debug_assert!(self.state_.raster_contents_scale_.y() <= max);
    }
    // Source: cc/layers/picture_layer_impl.cc:1447-1491.
    // Source: cc/layers/picture_layer_impl.h:235-236.
    pub(crate) fn AdjustRasterScaleForTransformAnimation(
        &mut self,
        preserved: Vector2dF,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        use crate::adapter::geometry::SizeF;
        debug_assert!(
            self.base_
                .base()
                .draw_properties()
                .screen_space_transform_is_animating
        );
        let maximum = c
            .tree
            .property_trees_mut()
            .MaximumAnimationToScreenScale(self.base_.base().transform_tree_index());
        self.state_
            .raster_contents_scale_
            .SetToMax(Vector2dF::new(maximum, maximum));
        if self.AffectedByWillChangeTransformHint(&c.BaseContext()) {
            self.state_.raster_contents_scale_.SetToMax(preserved);
        }
        let viewport = c.tree.GetDeviceViewport().size();
        const kMinViewportDimension: i32 = 500;
        let max_dimension = viewport
            .width()
            .max(viewport.height())
            .max(kMinViewportDimension) as f32;
        debug_assert!(max_dimension != 0.);
        let squared = max_dimension * max_dimension;
        let bounds = self
            .state_
            .raster_source_
            .as_ref()
            .expect("source nonnull raster")
            .recorded_bounds()
            .size();
        let visible = SizeF::new(
            crate::adapter::geometry::cpp_min(bounds.width() as f32, max_dimension),
            crate::adapter::geometry::cpp_min(bounds.height() as f32, max_dimension),
        );
        let scaled = SizeF::new(
            visible.width() * self.state_.raster_contents_scale_.x(),
            visible.height() * self.state_.raster_contents_scale_.y(),
        );
        let area = scaled.width() * scaled.height();
        if area > squared {
            self.state_
                .raster_contents_scale_
                .Scale(1. / (area / squared).sqrt());
        }
    }
    // Source: cc/layers/picture_layer_impl.cc:1732-1773.
    // Source: cc/layers/picture_layer_impl.h:270-270.
    pub(crate) fn UpdateIdealScales(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        debug_assert!(self.CanHaveTilings(&c.BaseContext()));
        let min = self.MinimumContentsScale();
        debug_assert!(min > 0.);
        self.state_.ideal_device_scale_ = c.tree.device_scale_factor();
        self.state_.ideal_page_scale_ = 1.;
        self.state_.ideal_contents_scale_ =
            self.base_.base().GetIdealContentsScale(&c.BaseContext());
        if c.tree.PageScaleTransformNode().is_some() {
            debug_assert!(c.tree.settings().is_for_scalable_page);
            self.state_.ideal_page_scale_ =
                if self.base_.base().IsAffectedByPageScale(&c.BaseContext()) {
                    c.tree.current_page_scale_factor()
                } else {
                    1.
                };
        }
        // Native constructed owner always has its nonnull tree; no service or
        // copied page-scale flag substitutes for this live getter.
        let external = c.tree.external_page_scale_factor();
        debug_assert!(
            c.tree.settings().is_for_scalable_page
                || external == 1.
                || c.tree.current_page_scale_factor() == 1.
        );
        self.state_.ideal_page_scale_ *= external;
        self.state_.ideal_contents_scale_.Scale(external);
        self.state_
            .ideal_contents_scale_
            .SetToMax(Vector2dF::new(min, min));
        self.state_.ideal_contents_scale_.SetToMin(Vector2dF::new(
            kMaxIdealContentsScale,
            kMaxIdealContentsScale,
        ));
        self.state_.ideal_source_scale_ = Vector2dF::new(
            self.state_.ideal_contents_scale_.x() / self.state_.ideal_page_scale_,
            self.state_.ideal_contents_scale_.y() / self.state_.ideal_page_scale_,
        );
    }
}

impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.cc:688-697.
    // Source: cc/layers/picture_layer_impl.h:281-281.
    pub(crate) fn UpdateCanUseLCDText(
        &mut self,
        aligns: bool,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        if !c.tree.IsSyncTree() {
            return;
        }
        self.state_.lcd_text_disallowed_reason_ =
            self.ComputeLCDTextDisallowedReason(aligns, &c.BaseContext());
    }
    // Source: cc/layers/picture_layer_impl.cc:706-758.
    // Source: cc/layers/picture_layer_impl.h:279-280.
    pub fn ComputeLCDTextDisallowedReason(
        &self,
        aligns: bool,
        c: &LayerImplContext<'_, B>,
    ) -> LCDTextDisallowedReason {
        use crate::adapter::picture_layer_impl::PictureDisplayItemList;
        let Some(source) = self.state_.raster_source_.as_ref() else {
            return LCDTextDisallowedReason::kNoText;
        };
        let Some(list) = source.GetDisplayItemList().as_ref() else {
            return LCDTextDisallowedReason::kNoText;
        };
        if !list.has_draw_text_ops() {
            return LCDTextDisallowedReason::kNoText;
        }
        if c.tree.settings().layers_always_allowed_lcd_text {
            return LCDTextDisallowedReason::kNone;
        }
        if !c.tree.settings().can_use_lcd_text {
            return LCDTextDisallowedReason::kSetting;
        }
        let node = c
            .tree
            .property_trees()
            .transform_tree()
            .Node(self.base_.base().transform_tree_index());
        if node.node_or_ancestors_will_change_transform {
            return LCDTextDisallowedReason::kWillChangeTransform;
        }
        if self.base_.base().screen_space_transform_is_animating() {
            return LCDTextDisallowedReason::kTransformAnimation;
        }
        let effect = c
            .tree
            .property_trees()
            .effect_tree()
            .Node(self.base_.base().effect_tree_index());
        if effect.lcd_text_disallowed_by_filter || effect.lcd_text_disallowed_by_backdrop_filter {
            return LCDTextDisallowedReason::kPixelOrColorEffect;
        }
        if !aligns {
            let offset = self.base_.base().offset_to_transform_parent();
            if offset.x() as i32 as f32 != offset.x() {
                return LCDTextDisallowedReason::kNonIntegralXOffset;
            }
            if offset.y() as i32 as f32 != offset.y() {
                return LCDTextDisallowedReason::kNonIntegralYOffset;
            }
            return LCDTextDisallowedReason::kNonIntegralTranslation;
        }
        if !self.base_.base().contents_opaque_for_text() {
            if self.base_.base().background_color().alpha != 1. {
                return LCDTextDisallowedReason::kBackgroundColorNotOpaque;
            }
            return LCDTextDisallowedReason::kContentsNotOpaque;
        }
        LCDTextDisallowedReason::kNone
    }
    // Source: cc/layers/picture_layer_impl.cc:767-805.
    // Source: cc/layers/picture_layer_impl.h:60-60.
    pub(crate) fn NotifyTileStateChanged(
        &mut self,
        tile: &crate::cc::tiles::tile::Tile<B>,
        update_damage: bool,
        c: &mut LayerImplContext<'_, B>,
    ) {
        self.TileNotificationParts().Notify(
            tile,
            update_damage,
            &mut PictureTilePushContext { tree: c.tree },
        );
    }
    /// Pure same-allocation split: no layer, tile or Set is moved or copied.
    pub(crate) fn TileNotificationParts(&mut self) -> PictureTileNotificationParts<'_, B> {
        let (base, damage) = self.base_.SplitTileNotification();
        PictureTileNotificationParts {
            base,
            damage,
            updated_tiles: &mut self.state_.updated_tiles_,
            should_batch: &self.state_.should_batch_updated_tiles_,
            set: self
                .tilings_
                .as_deref_mut()
                .map(PictureLayerTilingSet::SplitTileNotification),
        }
    }
    // Source: cc/layers/picture_layer_impl.cc:844-851.
    // Source: cc/layers/picture_layer_impl.h:67-67.
    pub fn GetInvalidationRegionForDebugging(
        &self,
    ) -> Region<<B as LayerTreeTypes>::RegionBackend> {
        crate::cc::base::region::IntersectRegions(
            &self.state_.invalidation_,
            self.base_.base().update_rect(),
        )
    }
    // Source: cc/layers/picture_layer_impl.cc:1966-1971.
    // Source: cc/layers/picture_layer_impl.h:159-160.
    pub fn SetPaintWorkletRecord(
        &mut self,
        input: Arc<<B as PaintWorkletTypes>::Input>,
        record: B::PaintRecord,
    ) {
        debug_assert!(self.state_.paint_worklet_records_.contains_key(&input));
        self.state_
            .paint_worklet_records_
            .value_mut_or_default(&input)
            .1 = Some(record);
    }
    // Source: cc/layers/picture_layer_impl.cc:1973-2011.
    // Source: cc/layers/picture_layer_impl.h:276-277.
    pub(crate) fn SetPaintWorkletInputs(
        &mut self,
        inputs: &[(
            Arc<<B as PaintWorkletTypes>::Input>,
            crate::adapter::image_animation::PaintImageId,
        )],
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        debug_assert!(c.tree.IsPendingTree() || inputs.is_empty());
        let had = !self.state_.paint_worklet_records_.is_empty();
        let mut new_records = PaintWorkletRecordMap::default();
        for (input, id) in inputs {
            debug_assert!(new_records.get(input).is_none_or(
                |entry: &(
                    crate::adapter::image_animation::PaintImageId,
                    Option<B::PaintRecord>
                )| entry.0 == *id
            ));
            // The native optional move retains a moved-from source, immediately
            // reset at the next source statement. take performs that exact pair.
            let moved = self
                .state_
                .paint_worklet_records_
                .value_mut_or_default(input)
                .1
                .take();
            new_records.insert(Arc::clone(input), (*id, moved));
        }
        self.state_.paint_worklet_records_.swap(&mut new_records);
        let has = !self.state_.paint_worklet_records_.is_empty();
        if has != had && c.tree.IsPendingTree() {
            c.tree
                .NotifyLayerHasPaintWorkletsChanged(self.base_.base().id(), has);
        }
        if c.tree.IsPendingTree() {
            c.tree
                .paint_worklet_tracker()
                .UpdatePaintWorkletInputProperties(inputs);
        }
        // new_records keeps the replaced native map alive through both callbacks.
    }
    // Source: cc/layers/picture_layer_impl.cc:2013-2028.
    // Source: cc/layers/picture_layer_impl.h:172-174.
    pub fn InvalidatePaintWorklets(
        &mut self,
        key: &B::PropertyKey,
        prev: &B::PropertyValue,
        next: &B::PropertyValue,
    ) {
        use crate::adapter::{
            paint_worklet::PaintWorkletInput, picture_layer_impl::PicturePaintWorkletInput,
        };
        for (input, entry) in self.state_.paint_worklet_records_.iter_mut() {
            if input.GetPropertyKeys().contains(key)
                && input.ValueChangeShouldCauseRepaint(prev, next)
            {
                entry.1 = None;
            }
        }
    }
    // Source: cc/layers/picture_layer_impl.cc:1883-1885.
    // Source: cc/layers/picture_layer_impl.h:126-126.
    pub(crate) fn IsOnActiveOrPendingTree(
        &self,
        c: &crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> bool {
        !c.tree.IsRecycleTree()
    }
    // Source: cc/layers/picture_layer_impl.cc:1887-1890.
    // Source: cc/layers/picture_layer_impl.h:80-80.
    pub(crate) fn HasValidTilePriorities(
        &self,
        c: &crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> bool {
        self.HasValidTilePrioritiesWithReadContext(c.tree)
    }
    // Same native source body with only its actual read-only tree capability.
    pub(crate) fn HasValidTilePrioritiesWithReadContext(
        &self,
        tree: &dyn crate::adapter::picture_layer_impl::PictureLayerReadContext,
    ) -> bool {
        HasValidTilePrioritiesForTree(
            tree,
            || self.base_.base().contributes_to_drawn_render_surface(),
            || self.base_.base().raster_even_if_not_drawn(),
        )
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.cc:459-462.
    // Source: cc/layers/picture_layer_impl.h:111-111.
    pub fn HasAnimatedImages(&self) -> bool {
        use crate::adapter::picture_layer_impl::PictureDiscardableImageMap;
        self.state_
            .discardable_image_map_
            .as_ref()
            .is_some_and(|map| !map.animated_images_metadata().is_empty())
    }
    // Source: cc/layers/picture_layer_impl.cc:464-476.
    // Source: cc/layers/picture_layer_impl.h:112-112.
    pub(crate) fn AnnotateAnimatedImages(
        &self,
        drivers: &mut crate::adapter::image_animation::AnimatedImageDriverMap,
        c: &crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        self.AnnotateAnimatedImagesWithReadContext(drivers, c.tree)
    }
    // Same native source body with only its actual read-only tree capability.
    pub(crate) fn AnnotateAnimatedImagesWithReadContext(
        &self,
        drivers: &mut crate::adapter::image_animation::AnimatedImageDriverMap,
        tree: &dyn crate::adapter::picture_layer_impl::PictureLayerReadContext,
    ) {
        use crate::adapter::picture_layer_impl::PictureDiscardableImageMap;
        let Some(map) = &self.state_.discardable_image_map_ else {
            return;
        };
        for (id, _) in map.animated_images_metadata() {
            let state = drivers.entry(*id).or_default();
            state.0 |= self.ShouldAnimateWithReadContext(*id, tree);
            if self.base_.base().canvas_child_id()
                != crate::adapter::input_types::ElementId::default()
            {
                state.1.push(self.base_.base().canvas_child_id());
            }
        }
    }
    // Source: cc/layers/picture_layer_impl.cc:953-995.
    // Source: cc/layers/picture_layer_impl.h:88-88.
    pub(crate) fn ShouldAnimate(
        &self,
        id: crate::adapter::image_animation::PaintImageId,
        c: &crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> bool {
        self.ShouldAnimateWithReadContext(id, c.tree)
    }
    // Same native source body with only its actual read-only tree capability.
    pub(crate) fn ShouldAnimateWithReadContext(
        &self,
        id: crate::adapter::image_animation::PaintImageId,
        tree: &dyn crate::adapter::picture_layer_impl::PictureLayerReadContext,
    ) -> bool {
        use crate::adapter::{
            image_animation::kAnimationPaused, picture_layer_impl::PictureDiscardableImageMap,
        };
        let map = self
            .state_
            .discardable_image_map_
            .as_ref()
            .expect("source CHECK discardable map");
        assert!(!map.empty());
        if !self.HasValidTilePrioritiesWithReadContext(tree)
            && self.base_.base().canvas_child_id()
                == crate::adapter::input_types::ElementId::default()
        {
            return false;
        }
        if let Some(metadata) = map.animated_images_metadata().get(&id) {
            if metadata.repetition_count == kAnimationPaused {
                return false;
            }
            if self.base_.base().canvas_child_id()
                != crate::adapter::input_types::ElementId::default()
            {
                return true;
            }
        }
        for rect in map.GetRectsForImage(id) {
            if rect.Intersects(self.base_.base().visible_layer_rect()) {
                return true;
            }
        }
        false
    }
    // Source: cc/layers/picture_layer_impl.cc:915-918.
    // Source: cc/layers/picture_layer_impl.h:83-84.
    pub fn GetDiscardableImagesInRect(
        &self,
        rect: &Rect,
    ) -> Vec<&<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::DrawImage> {
        use crate::adapter::picture_layer_impl::PictureDiscardableImageMap;
        self.state_
            .discardable_image_map_
            .as_ref()
            .expect("source nonnull map")
            .GetDiscardableImagesInRect(rect)
    }
    // Source: cc/layers/picture_layer_impl.cc:920-946.
    // Source: cc/layers/picture_layer_impl.h:85-85.
    pub fn GetRasterInducingScrollOffsets(
        &self,
        c: &LayerImplContext<'_, B>,
    ) -> crate::adapter::prioritized_tile::ScrollOffsetMap {
        RasterInducingScrollOffsetsForSource(self.state_.raster_source_.as_deref(), c.tree)
    }
    // Source: cc/layers/picture_layer_impl.cc:2076-2085.
    // Source: cc/layers/picture_layer_impl.h:68-68.
    pub fn GetContentColorUsage(&self) -> crate::adapter::layer_tree_host_impl::ContentColorUsage {
        use crate::adapter::{
            layer_tree_host_impl::ContentColorUsage, picture_layer_impl::PictureDisplayItemList,
        };
        let Some(source) = &self.state_.raster_source_ else {
            return ContentColorUsage::kSRGB;
        };
        let Some(list) = source.GetDisplayItemList() else {
            return ContentColorUsage::kSRGB;
        };
        list.content_color_usage()
    }
    // Source: cc/layers/picture_layer_impl.cc:1775-1787.
    // Source: cc/layers/picture_layer_impl.h:265-265.
    pub(crate) fn GetDebugBorderProperties(
        &self,
        c: &LayerImplContext<'_, B>,
    ) -> (crate::adapter::layer_tree_impl::Color4f, f32) {
        DebugBorderPropertiesForState(&self.state_, c)
    }
}
// The native client is the actual PictureLayerImpl. This temporary receiver
// only splits its owning siblings so Set and its selected child can execute
// synchronously; none of these references is retained past WithTilingSet.
struct PictureTilingClient<'a, B: PictureLayerImplTypes> {
    identity: PictureLayerTilingClientId,
    base: super::layer_impl::LayerImplTileClient<'a>,
    is_backdrop_filter_mask: &'a bool,
    state: PictureTilingClientState<'a, B>,
    tree: &'a mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTree<B>,
    environment:
        &'a mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<B>,
    services: &'a mut dyn crate::adapter::picture_layer_impl::PictureLayerImplServices<B>,
}
// Only the calculator is mutable during a tiling client callback. Invalidation,
// Paint records and source metadata stay borrowed at their actual owner fields,
// permitting native const references passed into Set methods to remain live.
struct PictureTilingClientState<'a, B: PictureLayerImplTypes> {
    updated_tiles_: &'a mut TileUpdateSet,
    should_batch_updated_tiles_: &'a bool,
    twin_layer_: &'a Option<PictureLayerImplId>,
    raster_source_: &'a Option<Arc<B::RasterSource>>,
    invalidation_: &'a Region<<B as LayerTreeTypes>::RegionBackend>,
    discardable_image_map_: &'a Option<Arc<B::DiscardableImageMap>>,
    paint_worklet_records_:
        &'a PaintWorkletRecordMap<<B as PaintWorkletTypes>::Input, B::PaintRecord>,
    gpu_raster_max_texture_size_: &'a Size,
    tile_size_calculator_: &'a mut TileSizeCalculator,
    viewport_rect_for_tile_priority_in_content_space_: &'a Rect,
    ideal_contents_scale_: &'a Vector2dF,
}
impl<B: PictureLayerImplTypes> PictureLayerImplState<B> {
    fn TilingClientState(&mut self) -> PictureTilingClientState<'_, B> {
        PictureTilingClientState {
            updated_tiles_: &mut self.updated_tiles_,
            should_batch_updated_tiles_: &self.should_batch_updated_tiles_,
            twin_layer_: &self.twin_layer_,
            raster_source_: &self.raster_source_,
            invalidation_: &self.invalidation_,
            discardable_image_map_: &self.discardable_image_map_,
            paint_worklet_records_: &self.paint_worklet_records_,
            gpu_raster_max_texture_size_: &self.gpu_raster_max_texture_size_,
            tile_size_calculator_: &mut self.tile_size_calculator_,
            viewport_rect_for_tile_priority_in_content_space_: &self
                .viewport_rect_for_tile_priority_in_content_space_,
            ideal_contents_scale_: &self.ideal_contents_scale_,
        }
    }
}

struct PictureTileSizeContext<'a, B: PictureLayerImplTypes> {
    is_backdrop_filter_mask: &'a bool,
    gpu_raster_max_texture_size: &'a Size,
    tree: &'a dyn crate::adapter::picture_layer_impl::PictureLayerImplReadTree<B>,
}
impl<B: PictureLayerImplTypes> super::tile_size_calculator::TileSizeCalculatorContext
    for PictureTileSizeContext<'_, B>
{
    fn IsBackdropFilterMask(&self) -> bool {
        *self.is_backdrop_filter_mask
    }
    fn MaxTextureSize(&self) -> i32 {
        self.tree.max_texture_size()
    }
    fn UseGpuRasterization(&self) -> bool {
        self.tree.UseGpuRasterization()
    }
    fn MaxGpuRasterTileSize(&self) -> Size {
        self.tree.settings().max_gpu_raster_tile_size
    }
    fn MinHeightForGpuRasterTile(&self) -> i32 {
        self.tree.settings().min_height_for_gpu_raster_tile
    }
    fn GpuRasterMaxTextureSize(&self) -> Size {
        *self.gpu_raster_max_texture_size
    }
    fn DeviceScaleFactor(&self) -> f32 {
        self.tree.device_scale_factor()
    }
    fn MaxUntiledLayerSize(&self) -> Size {
        self.tree.settings().max_untiled_layer_size
    }
    fn DefaultTileSize(&self) -> Size {
        self.tree.settings().default_tile_size
    }
}
// Source: cc/layers/picture_layer_impl.cc:920-946.
// Shared only to lend either the resident owner or its selected source client;
// the raster-inducing-scroll algorithm has this single canonical body.
// Source: cc/layers/picture_layer_impl.cc:920-946.
fn RasterInducingScrollOffsetsForSource<B: PictureLayerImplTypes>(
    source: Option<&B::RasterSource>,
    tree: &dyn crate::adapter::layer_impl::LayerImplReadTree<B>,
) -> crate::adapter::prioritized_tile::ScrollOffsetMap {
    use crate::adapter::{
        geometry::{PointF, Transform},
        picture_layer_impl::PictureDisplayItemList,
    };
    let mut map = crate::adapter::prioritized_tile::ScrollOffsetMap::new();
    if let Some(source) = source {
        let scrolls = tree.property_trees().scroll_tree();
        let transforms = tree.property_trees().transform_tree();
        for (element, _) in source
            .GetDisplayItemList()
            .as_ref()
            .expect("source display list")
            .raster_inducing_scrolls()
        {
            if let Some(node) = scrolls.FindNodeFromElementId(*element).filter(|node| {
                node.transform_id
                    != crate::cc::trees::property_ids::kInvalidPropertyNodeId
            }) {
                let translation = transforms
                    .Node(node.transform_id)
                    .to_parent
                    .To2dTranslation();
                map.insert(*element, PointF::new(-translation.x(), -translation.y()));
            } else {
                map.insert(
                    *element,
                    scrolls.current_scroll_offset(*element, tree.property_trees().state()),
                );
            }
        }
    }
    map
}
impl<B: PictureLayerImplTypes> PictureLayerImplState<B> {
    // Source: cc/layers/picture_layer_impl.cc:516-521.
    fn PendingOrActiveTwinLayer<'a>(
        &self,
        environment: &'a dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<
            B,
        >,
    ) -> Option<PictureLayerImplReadView<'a, B>> {
        PendingOrActiveTwinRead(self.twin_layer_, environment)
    }
}
impl<B: PictureLayerImplTypes>
    crate::cc::tiles::picture_layer_tiling::PictureLayerTilingClient<B>
    for PictureTilingClient<'_, B>
{
    // Source: cc/layers/picture_layer_impl.cc:997-999.
    // Source: cc/layers/picture_layer_impl.h:76-76.

    // Source: cc/layers/picture_layer_impl.cc:874-883.
    // Source: cc/layers/picture_layer_impl.h:77-77.
    fn GetPendingInvalidation(
        &self,
        queries: &dyn crate::adapter::picture_layer_tiling::PictureLayerTilingQueries<B>,
    ) -> Option<crate::adapter::picture_layer_tiling::PendingInvalidationId> {
        PendingInvalidationForIdentity(self.identity, *self.state.twin_layer_, self.tree, queries)
    }
    // Source: cc/layers/picture_layer_impl.cc:885-897.
    // Source: cc/layers/picture_layer_impl.h:78-79.
    fn GetPendingOrActiveTwinTiling(
        &self,
        tiling: &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
        queries: &dyn crate::adapter::picture_layer_tiling::PictureLayerTilingQueries<B>,
        selected: Option<
            &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
        >,
    ) -> Option<crate::adapter::tile::PictureLayerTilingId> {
        TwinTilingForIdentity(*self.state.twin_layer_, tiling, queries, selected)
    }
    // Source: cc/layers/picture_layer_impl.cc:1887-1890.
    // Source: cc/layers/picture_layer_impl.h:80-80.
    fn HasValidTilePriorities(&self) -> bool {
        HasValidTilePrioritiesForTree(
            self.tree,
            || self.base.contributes_to_drawn_render_surface(),
            || self.base.raster_even_if_not_drawn(),
        )
    }
    // Source: cc/layers/picture_layer_impl.cc:899-901.
    // Source: cc/layers/picture_layer_impl.h:81-81.
    fn RequiresHighResToDraw(&self) -> bool {
        self.tree.RequiresHighResToDraw()
    }
    // Source: cc/layers/picture_layer_impl.cc:903-905.
    // Source: cc/layers/picture_layer_impl.h:82-82.
    fn GetPaintWorkletRecords(&self) -> &B::PaintWorkletRecordMap {
        &self.state.paint_worklet_records_
    }
    // Source: cc/layers/picture_layer_impl.cc:915-918.
    // Source: cc/layers/picture_layer_impl.h:83-84.
    fn GetDiscardableImagesInRect(
        &self,
        rect: Rect,
    ) -> Vec<&<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::DrawImage> {
        use crate::adapter::picture_layer_impl::PictureDiscardableImageMap;
        self.state
            .discardable_image_map_
            .as_ref()
            .expect("source nonnull map")
            .GetDiscardableImagesInRect(&rect)
    }
    // Source: cc/layers/picture_layer_impl.cc:920-946.
    // Source: cc/layers/picture_layer_impl.h:85-85.
    fn GetRasterInducingScrollOffsets(&self) -> crate::adapter::prioritized_tile::ScrollOffsetMap {
        RasterInducingScrollOffsetsForSource::<B>(self.state.raster_source_.as_deref(), self.tree)
    }
    // Source: cc/layers/picture_layer_impl.cc:948-951.
    // Source: cc/layers/picture_layer_impl.h:86-86.
    fn global_tile_state(
        &self,
    ) -> &crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority {
        self.tree.global_tile_state()
    }
}
impl<B: PictureLayerImplTypes> crate::adapter::picture_layer_tiling::PictureLayerTilingQueries<B>
    for PictureTilingClient<'_, B>
{
    fn WithClient(
        &self,
        id: PictureLayerTilingClientId,
        callback: &mut dyn FnMut(
            &dyn crate::cc::tiles::picture_layer_tiling::PictureLayerTilingClient<B>,
        ),
    ) {
        if id == self.identity {
            callback(self)
        } else {
            self.environment.WithClient(id, callback)
        }
    }
    fn PaintWorkletRecordsForClient(
        &self,
        id: PictureLayerTilingClientId,
    ) -> &B::PaintWorkletRecordMap {
        if id == self.identity {
            self.state.paint_worklet_records_
        } else {
            self.environment.PaintWorkletRecordsForClient(id)
        }
    }
    fn TilingById(&self, id: crate::adapter::tile::PictureLayerTilingId) -> &PictureLayerTiling<B> {
        self.environment.TilingById(id)
    }
    fn IsClientOnActiveOrPendingTree(&self, id: PictureLayerTilingClientId) -> bool {
        if id == self.identity {
            !self.tree.IsRecycleTree()
        } else {
            self.environment.IsClientOnActiveOrPendingTree(id)
        }
    }
    fn TilingStateForClientAndScale(
        &self,
        id: PictureLayerTilingClientId,
        scale: f32,
    ) -> Option<&crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>>
    {
        self.environment.TilingStateForClientAndScale(id, scale)
    }
    fn PendingInvalidationById(
        &self,
        id: crate::adapter::picture_layer_tiling::PendingInvalidationId,
    ) -> &Region<<B as LayerTreeTypes>::RegionBackend> {
        if id.0 == self.identity.0 {
            &self.state.invalidation_
        } else {
            self.environment.PendingInvalidationById(id)
        }
    }
}
impl<B: PictureLayerImplTypes> crate::adapter::picture_layer_tiling::PictureLayerTileDestroyer<B>
    for PictureTilingClient<'_, B>
{
    fn RebindTileWithManager(
        &mut self,
        old: crate::adapter::tile::TileId,
        new: crate::adapter::tile::TileId,
    ) {
        self.environment.RebindTileWithManager(old, new)
    }
    fn DestroyTileWithManager(
        &mut self,
        id: crate::adapter::tile::TileManagerId,
        tile: &mut crate::cc::tiles::tile::Tile<B>,
        tiling: &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
    ) {
        let mut notification = PictureReleaseNotification {
            base: &mut self.base,
            updated_tiles: self.state.updated_tiles_,
            should_batch: self.state.should_batch_updated_tiles_,
            tree: self.tree,
        };
        self.environment
            .DestroyTileWithManagerForSelected(id, tile, tiling, &mut notification)
    }
}
impl<B: PictureLayerImplTypes>
    crate::cc::tiles::picture_layer_tiling_set::PictureLayerTilingSetContext<B>
    for PictureTilingClient<'_, B>
{
    fn CalculateTileSizeForClient(&mut self, id: PictureLayerTilingClientId, bounds: Size) -> Size {
        assert_eq!(id, self.identity, "source selected client");
        self.CalculateTileSize(bounds)
    }
    // Source: cc/layers/picture_layer_impl.h:123;
    // Source: cc/layers/picture_layer_impl.cc:853-878.
    // all_tiles_done is exactly the selected Set field supplied independently
    // of its resident tilings, preserving the native synchronous callback write.
    fn CreateTileForClient(
        &mut self,
        id: PictureLayerTilingClientId,
        info: &crate::cc::tiles::tile::CreateInfo<B::RasterTransform>,
        all_tiles_done: &mut bool,
    ) -> Option<Box<crate::cc::tiles::tile::Tile<B>>> {
        assert_eq!(id, self.identity, "source selected client");
        PictureLayerImpl::<B>::CreateTile(self, info, all_tiles_done)
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.tree.IsCCSlimmingEnabled()
    }
    fn AllocateTilingId(&mut self) -> crate::adapter::tile::PictureLayerTilingId {
        self.environment.AllocateTilingId()
    }
}

impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    fn WithTilingSet<R>(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
        callback: impl FnOnce(&mut PictureLayerTilingSet<B>, &mut PictureTilingClient<'_, B>) -> R,
    ) -> R {
        self.WithTilingSetAndDraw(c, |set, client, _draw| callback(set, client))
    }
    fn WithTilingSetAndDraw<R>(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
        callback: impl FnOnce(
            &mut PictureLayerTilingSet<B>,
            &mut PictureTilingClient<'_, B>,
            &crate::cc::layers::draw_properties::DrawProperties<B::Transform>,
        ) -> R,
    ) -> R {
        let set = self.tilings_.as_deref_mut().expect("source live tilings");
        let (base, draw, mask) = self.base_.SplitTileClient();
        let mut client = PictureTilingClient {
            identity: self.client_identity_,
            base,
            is_backdrop_filter_mask: mask,
            state: self.state_.TilingClientState(),
            tree: c.tree,
            environment: c.tiling,
            services: c.services,
        };
        callback(set, &mut client, draw)
    }
    // Source: cc/layers/picture_layer_impl.cc:828-831.
    // Source: cc/layers/picture_layer_impl.h:64-64.
    pub(crate) fn ReleaseResources(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        self.WithTilingSet(c, |set, client| set.ReleaseAllResources(client));
        self.ResetRasterScale();
    }
    // Source: cc/layers/picture_layer_impl.cc:833-836.
    // Source: cc/layers/picture_layer_impl.h:65-65.
    pub(crate) fn ReleaseTileResources(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        self.ReleaseResources(c)
    }
    // Source: cc/layers/picture_layer_impl.cc:838-842.
    // Source: cc/layers/picture_layer_impl.h:66-66.
    pub(crate) fn RecreateTileResources(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        let replacement = self.CreatePictureLayerTilingSet(c);
        // Source unique_ptr::reset first installs the new allocation, then
        // deletes its old pointee. This is that actual ownership transfer, not
        // an owner temporarily removed to bypass Rust borrowing. Reentrant tile
        // notification observes this same Picture's new resident Set.
        let mut previous = self.tilings_.replace(replacement).expect("source old set");
        self.WithTilingSet(c, |_new_set, client| previous.Destroy(client));
        drop(previous);
    }
    // Source: cc/layers/picture_layer_impl.cc:1106-1110.
    // Source: cc/layers/picture_layer_impl.h:229-229.
    pub(crate) fn RemoveAllTilings(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        self.WithTilingSet(c, |set, client| set.RemoveAllTilings(client));
        self.ResetRasterScale();
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.cc:1593-1635.
    // Source: cc/layers/picture_layer_impl.h:239-239.
    pub(crate) fn CalculateRasterTranslation(
        &self,
        translation: &mut Vector2dF,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> bool {
        use crate::adapter::{
            geometry::Transform, picture_layer_impl::PictureDisplayItemList,
            property_tree::PropertyTreeTransform,
        };
        if c.tree.settings().layers_always_allowed_lcd_text {
            return true;
        }
        if self
            .state_
            .raster_source_
            .as_ref()
            .and_then(|source| source.GetDisplayItemList().as_ref())
            .is_none_or(|list| !list.has_draw_text_ops())
        {
            return false;
        }
        let external = if c.services.ComputeRasterTranslateForExternalScale() {
            c.tree.external_page_scale_factor()
        } else {
            1.
        };
        let mut draw = self.base_.base().DrawTransform(&c.BaseContext());
        draw.PostScale(external, external);
        if !crate::cc::trees::draw_property_utils::RasterScalesApproximatelyEqual(
            draw.To2dScale(),
            self.state_.raster_contents_scale_,
        ) {
            return false;
        }
        let mut screen = self
            .base_
            .base()
            .ScreenSpaceTransform(&c.BaseContext())
            .clone();
        screen.PostScale(external, external);
        if let Some(offset) =
            crate::cc::trees::draw_property_utils::PixelAlignmentOffset(&screen, &draw)
        {
            *translation = offset;
            return true;
        }
        false
    }
    // Source: cc/layers/picture_layer_impl.cc:760-765.
    // Source: cc/layers/picture_layer_impl.h:154-154.
    pub(crate) fn ComputeLCDTextDisallowedReasonForTesting(
        &self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> LCDTextDisallowedReason {
        let mut translation = Vector2dF::default();
        let aligns = self.CalculateRasterTranslation(&mut translation, c);
        self.ComputeLCDTextDisallowedReason(aligns, &c.BaseContext())
    }
    // Source: cc/layers/picture_layer_impl.cc:478-514.
    // Source: cc/layers/picture_layer_impl.h:243-243.
    pub(crate) fn UpdateViewportRectForTilePriorityInContentSpace(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        use crate::adapter::property_tree::PropertyTreeTransform;
        let mut visible = self.base_.base().visible_layer_rect();
        let viewport = c.tree.ViewportRectForTilePriority();
        if visible.IsEmpty() || c.tree.GetDeviceViewport() != viewport {
            let mut inverse = B::Transform::default();
            if self
                .base_
                .base()
                .ScreenSpaceTransform(&c.BaseContext())
                .GetInverse(&mut inverse)
            {
                visible =
                    crate::cc::base::math_util::MathUtil::ProjectEnclosingClippedRect(
                        &inverse, viewport,
                    );
                let mut padded = Rect::new(
                    0,
                    0,
                    self.base_.base().bounds(&c.BaseContext()).width(),
                    self.base_.base().bounds(&c.BaseContext()).height(),
                );
                let padding =
                    (c.tree
                        .settings()
                        .skewport_extrapolation_limit_in_screen_pixels as f32
                        * self.MaximumTilingContentsScale()) as i32;
                padded.Inset(-padding, -padding, -padding, -padding);
                visible = SafeIntersectRects(visible, padded);
            }
        }
        self.state_
            .viewport_rect_for_tile_priority_in_content_space_ = visible;
    }
    // Source: cc/layers/picture_layer_impl.cc:1112-1158.
    // Source: cc/layers/picture_layer_impl.h:230-231.
    pub(crate) fn CanRecreateHighResTilingForLCDTextAndRasterTransform(
        &self,
        high: &PictureLayerTiling<B>,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> bool {
        use crate::adapter::geometry::AxisTransform2d;
        if high.can_use_lcd_text()
            && matches!(
                self.state_.lcd_text_disallowed_reason_,
                LCDTextDisallowedReason::kBackgroundColorNotOpaque
                    | LCDTextDisallowedReason::kContentsNotOpaque
            )
        {
            debug_assert!(!c.tree.IsSyncTree() || !c.tree.IsReadyToActivate());
            return true;
        }
        if self
            .base_
            .base()
            .draw_properties()
            .screen_space_transform_is_animating
            || self.AffectedByWillChangeTransformHint(&c.BaseContext())
        {
            return false;
        }
        if c.tree.PinchGestureActive() {
            return false;
        }
        if self.state_.lcd_text_disallowed_reason_ == LCDTextDisallowedReason::kNoText
            && *high.raster_transform().scale() == self.state_.raster_contents_scale_
        {
            return false;
        }
        if c.tree.IsSyncTree() && c.tree.IsReadyToActivate() {
            return false;
        }
        if c.tree.GetActivelyScrollingType()
            != crate::cc::input::actively_scrolling_type::ActivelyScrollingType::kNone
        {
            return false;
        }
        true
    }
    // Source: cc/layers/picture_layer_impl.cc:1092-1104.
    // Source: cc/layers/picture_layer_impl.h:228-228.
    pub(crate) fn AddTiling(
        &mut self,
        transform: &B::RasterTransform,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> crate::adapter::tile::PictureLayerTilingId {
        debug_assert!(self.CanHaveTilings(&c.BaseContext()));
        use crate::adapter::geometry::AxisTransform2d;
        debug_assert!(transform.scale().x() >= self.MinimumContentsScale());
        debug_assert!(transform.scale().y() >= self.MinimumContentsScale());
        debug_assert!(transform.scale().x() <= self.MaximumContentsScale(&c.BaseContext()));
        debug_assert!(transform.scale().y() <= self.MaximumContentsScale(&c.BaseContext()));
        debug_assert!(self.state_.raster_source_.as_ref().unwrap().HasRecordings());
        let lcd = self.state_.lcd_text_disallowed_reason_ == LCDTextDisallowedReason::kNone
            && *transform.scale() == self.state_.raster_contents_scale_;
        let source = Arc::clone(self.state_.raster_source_.as_ref().expect("source raster"));
        self.WithTilingSet(c, |set, client| {
            set.AddTiling(transform, source, lcd, client)
        })
    }
    // Source: cc/layers/picture_layer_impl.cc:1160-1232.
    // Source: cc/layers/picture_layer_impl.h:232-232.
    pub(crate) fn UpdateTilingsForRasterScaleAndTranslation(
        &mut self,
        adjusted: bool,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        use crate::adapter::geometry::AxisTransform2d;
        // The identity below is the source nonowning pointer, not tiling state.
        // All comparisons read the selected resident allocation before mutation.
        let mut high = self
            .picture_layer_tiling_set()
            .FindTilingWithScaleKeyRef(self.raster_contents_scale_key())
            .map(|tiling| tiling.identity());
        let mut translation = Vector2dF::default();
        let aligns = self.CalculateRasterTranslation(&mut translation, c);
        self.UpdateCanUseLCDText(aligns, c);
        if let Some(id) = high {
            let selected = self.picture_layer_tiling_set().TilingById(id);
            let nonideal = *selected.raster_transform().scale()
                != self.state_.raster_contents_scale_
                || *selected.raster_transform().translation() != translation;
            let lcd_changed = selected.can_use_lcd_text()
                != (self.state_.lcd_text_disallowed_reason_ == LCDTextDisallowedReason::kNone);
            let can_recreate =
                self.CanRecreateHighResTilingForLCDTextAndRasterTransform(selected, c);
            let should_recreate = (nonideal || lcd_changed) && c.tree.IsSyncTree() && can_recreate;
            let can_request = (nonideal || lcd_changed)
                && !c.tree.settings().commit_to_active_tree
                && c.tree.IsActiveTree()
                && can_recreate
                && !c.tree.HasPendingTree();
            if should_recreate {
                self.WithTilingSet(c, |set, client| set.Remove(id, client));
                high = None;
            } else if can_request {
                c.tree.RequestImplSideInvalidationForRerasterTiling();
            } else if !adjusted {
                debug_assert_eq!(
                    selected.resolution(),
                    crate::cc::tiles::tile_priority::TileResolution::HIGH_RESOLUTION
                );
                #[cfg(debug_assertions)]
                self.SanityCheckTilingState(&c.BaseContext());
                return;
            }
        }
        self.picture_layer_tiling_set_mut().MarkAllTilingsNonIdeal();
        if high.is_none() {
            high = Some(self.AddTiling(
                &B::RasterTransform::FromScaleAndTranslation(
                    self.state_.raster_contents_scale_,
                    translation,
                ),
                c,
            ));
        }
        self.picture_layer_tiling_set_mut()
            .TilingByIdMut(high.unwrap())
            .set_resolution(
                crate::cc::tiles::tile_priority::TileResolution::HIGH_RESOLUTION,
            );
        if c.tree.IsPendingTree()
            || (c.tree.settings().commit_to_active_tree && self.IsDirectlyCompositedImage())
        {
            self.WithTilingSet(c, |set, client| set.RemoveNonIdealTilings(client));
        }
        #[cfg(debug_assertions)]
        self.SanityCheckTilingState(&c.BaseContext());
    }
    // Source: cc/layers/picture_layer_impl.cc:1493-1559.
    // Source: cc/layers/picture_layer_impl.h:240-240.
    pub(crate) fn CleanUpTilingsOnActiveLayer(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        debug_assert!(c.tree.IsActiveTree());
        if self.picture_layer_tiling_set().num_tilings() == 0 {
            return;
        }
        let mut min = crate::adapter::geometry::cpp_min(
            self.raster_contents_scale_key(),
            self.GetIdealContentsScaleKey(),
        );
        let mut max = crate::adapter::geometry::cpp_max(
            self.raster_contents_scale_key(),
            self.GetIdealContentsScaleKey(),
        );
        if let Some(twin) = self.state_.PendingOrActiveTwinLayer(c.tiling) {
            // Only borrowed siblings are needed for CanHaveTilings; no target
            // tree or layer state is copied into the selected owner's context.
            if twin.CanHaveTilings() {
                min = crate::adapter::geometry::cpp_min(
                    crate::adapter::geometry::cpp_min(min, twin.raster_contents_scale_key()),
                    twin.GetIdealContentsScaleKey(),
                );
                max = crate::adapter::geometry::cpp_max(
                    crate::adapter::geometry::cpp_max(max, twin.raster_contents_scale_key()),
                    twin.GetIdealContentsScaleKey(),
                );
            }
        }
        let mut remove = Vec::new();
        let mut needs_push = false;
        for index in 0..self.picture_layer_tiling_set().num_tilings() {
            let tiling = self.picture_layer_tiling_set().tiling_at(index);
            if tiling.contents_scale_key() >= min && tiling.contents_scale_key() <= max {
                continue;
            }
            if self
                .base_
                .LastAppendQuadsScalesContains(tiling.contents_scale_key())
            {
                continue;
            }
            if c.tree.TreesInVizInClientProcess() {
                self.state_
                    .proposed_tiling_scales_for_deletion_
                    .insert(ScaleKey::new(tiling.contents_scale_key()));
                needs_push = true;
            } else {
                remove.push(tiling.identity());
            }
        }
        if c.tree.TreesInVizInClientProcess() {
            if needs_push {
                self.base_.base_mut().SetNeedsPushProperties(
                    super::layer_impl::LayerImpl::<B>::kChangedGeneralProperty,
                    &mut c.BaseContext(),
                );
            }
            return;
        }
        for id in remove {
            debug_assert_ne!(
                self.picture_layer_tiling_set().TilingById(id).resolution(),
                crate::cc::tiles::tile_priority::TileResolution::HIGH_RESOLUTION
            );
            self.WithTilingSet(c, |set, client| set.Remove(id, client));
        }
    }
    // Source: cc/layers/picture_layer_impl.cc:1584-1591.
    // Source: cc/layers/picture_layer_impl.h:372-372.
    pub(crate) fn CleanUpTilings(
        &mut self,
        scales: &[f32],
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        for scale in scales {
            if let Some(tiling) = self
                .picture_layer_tiling_set()
                .FindTilingWithScaleKeyRef(*scale)
            {
                let id = tiling.identity();
                self.WithTilingSet(c, |set, client| set.Remove(id, client));
            }
        }
    }
    // Source: cc/layers/picture_layer_impl.cc:807-816.
    // Source: cc/layers/picture_layer_impl.h:70-70.
    pub(crate) fn DidDraw(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        // Native provider is forwarded only to the empty base virtual; it is
        // omitted from this safe borrow boundary. Resource cleanup uses the
        // same actual provider already lent through the live context.
        let mut base_receiver = super::layer_impl::LayerImplBaseVirtual;
        super::layer_impl::LayerImplVirtual::DidDraw(
            &mut base_receiver,
            self.base_.base_mut(),
            &mut c.BaseContext(),
        );
        self.CleanUpTilingsOnActiveLayer(c);
        #[cfg(debug_assertions)]
        self.SanityCheckTilingState(&c.BaseContext());
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.cc:380-457.
    // Source: cc/layers/picture_layer_impl.h:110-110.
    pub(crate) fn UpdateTiles(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> bool {
        if !self.CanHaveTilings(&c.BaseContext()) {
            self.state_.ideal_page_scale_ = 0.;
            self.state_.ideal_device_scale_ = 0.;
            self.state_.ideal_contents_scale_ = Vector2dF::default();
            self.state_.ideal_source_scale_ = Vector2dF::default();
            #[cfg(debug_assertions)]
            self.SanityCheckTilingState(&c.BaseContext());
            return false;
        }
        if c.tree.IsActiveTree() {
            self.CleanUpTilingsOnActiveLayer(c);
        }
        self.UpdateIdealScales(c);
        let adjusted = self.ShouldAdjustRasterScale(c);
        if adjusted {
            self.RecalculateRasterScales(c);
        }
        self.UpdateTilingsForRasterScaleAndTranslation(adjusted, c);
        self.state_.raster_source_size_changed_ = false;
        debug_assert!(self.state_.raster_page_scale_ != 0.);
        debug_assert!(self.state_.raster_device_scale_ != 0.);
        debug_assert!(self.state_.raster_source_scale_.x() != 0.);
        debug_assert!(self.state_.raster_source_scale_.y() != 0.);
        debug_assert!(self.state_.raster_contents_scale_.x() != 0.);
        debug_assert!(self.state_.raster_contents_scale_.y() != 0.);
        self.state_.was_screen_space_transform_animating_ = self
            .base_
            .base()
            .draw_properties()
            .screen_space_transform_is_animating;
        let seconds = (c.tree.CurrentBeginFrameArgs().frame_time
            - crate::adapter::time::TimeTicks::default())
        .InSecondsF();
        self.UpdateViewportRectForTilePriorityInContentSpace(c);
        let mut can_require = false;
        if self.base_.base().contributes_to_drawn_render_surface() {
            can_require = self.base_.produced_tile_last_append_quads()
                || c.tree.RequiresHighResToDraw()
                || !c.tree.SmoothnessTakesPriority();
        }
        let updated = self.WithTilingSetAndDraw(c, |set, client, draw| {
            let empty = crate::cc::trees::occlusion::Occlusion::default();
            let occlusion = if client.tree.settings().use_occlusion_for_tile_prioritization {
                &draw.occlusion_in_content_space
            } else {
                &empty
            };
            set.UpdateTilePriorities(
                *client
                    .state
                    .viewport_rect_for_tile_priority_in_content_space_,
                crate::adapter::geometry::cpp_max(
                    client.state.ideal_contents_scale_.x(),
                    client.state.ideal_contents_scale_.y(),
                ),
                seconds,
                occlusion,
                can_require,
                client,
            )
        });
        debug_assert!(self.picture_layer_tiling_set().num_tilings() > 0);
        #[cfg(debug_assertions)]
        self.SanityCheckTilingState(&c.BaseContext());
        updated
    }
    // Source: cc/layers/picture_layer_impl.cc:307-368.
    // Source: cc/layers/picture_layer_impl.h:72-72.
    pub(crate) fn ComputeCheckerboardedNeedsRecord(
        &self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> bool {
        use crate::adapter::{
            geometry::{RectF, ToEnclosedRect},
            picture_layer_impl::PictureDisplayItemList,
        };
        if self.base_.is_backdrop_filter_mask() {
            return false;
        }
        if self.base_.solid_color().is_some() {
            return false;
        }
        let trees = c.tree.property_trees();
        let scrolls = trees.scroll_tree();
        if let Some(list) = self
            .state_
            .raster_source_
            .as_ref()
            .and_then(|source| source.GetDisplayItemList().as_ref())
        {
            for (element, info) in list.raster_inducing_scrolls() {
                if !info
                    .visual_rect
                    .Intersects(self.base_.base().visible_layer_rect())
                {
                    continue;
                }
                if let Some(cull) = scrolls.ScrollingContentsCullRect(*element) {
                    if let Some(node) = scrolls.FindNodeFromElementId(*element) {
                        if !scrolls.CanRealizeScrollsOnPendingTree(node) {
                            continue;
                        }
                        let size = scrolls.container_bounds(node.id, trees.state());
                        let mut visible = RectF::new(
                            node.container_origin.x() as f32,
                            node.container_origin.y() as f32,
                            size.width() as f32,
                            size.height() as f32,
                        );
                        let offset = scrolls.current_scroll_offset(*element, trees.state());
                        visible.Offset(offset.x(), offset.y());
                        if !cull.Contains(ToEnclosedRect(visible)) {
                            return true;
                        }
                    }
                }
            }
        }
        let Some(cull) = self.base_.CalculateCullRectInLayerSpace(&c.BaseContext()) else {
            return false;
        };
        let mut recorded = self.base_.base().visible_layer_rect();
        let bounds = self.base_.base().bounds(&c.BaseContext());
        recorded.Intersect(Rect::new(0, 0, bounds.width(), bounds.height()));
        recorded.Intersect(self.RecordedBounds());
        let unoccluded = self
            .base_
            .base()
            .draw_properties()
            .occlusion_in_content_space
            .GetUnoccludedContentRect(recorded);
        if !unoccluded.IsEmpty() && !cull.Contains(unoccluded) {
            return true;
        }
        false
    }
    // Source: cc/layers/picture_layer_impl.cc:268-271.
    // Source: cc/layers/picture_layer_impl.h:415-416.
    pub fn ShouldUpdateApproximatedVisibleContentArea(
        &self,
        resolution: crate::cc::tiles::tile_priority::TileResolution,
    ) -> bool {
        ShouldUpdateApproximatedVisibleContentAreaForResolution(resolution)
    }
    // Source: cc/layers/picture_layer_impl.cc:273-278.
    // Source: cc/layers/picture_layer_impl.h:417-419.
    pub fn ShouldReportTileAsMissing(&self, geometry: Rect, viewport: Rect) -> bool {
        ShouldReportTileAsMissingForRects(geometry, viewport)
    }
    // Source: cc/layers/picture_layer_impl.cc:298-305.
    // Source: cc/layers/picture_layer_impl.h:413-414.
    pub fn WillProcessReadyToDrawTile(
        &self,
        iter:&crate::cc::tiles::tiling_set_coverage_iterator::TilingSetCoverageIterator<'_,PictureLayerTiling<B>,<B as LayerTreeTypes>::RegionBackend>,
    ) {
        WillProcessReadyToDrawTileForIterator::<B>(iter)
    }
    // Source: cc/layers/picture_layer_impl.cc:370-372.
    // Source: cc/layers/picture_layer_impl.h:402-402.
    pub fn WillAppendQuads(&mut self) {
        WillAppendQuadsForBase(&mut self.base_)
    }
    // Source: cc/layers/picture_layer_impl.cc:374-378.
    // Source: cc/layers/picture_layer_impl.h:425-425.
    pub fn GetScaledViewportForTilePriority(&self, scale: f32) -> Rect {
        ScaledViewportForState(&self.state_, scale)
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.cc:1892-1931.
    // Source: cc/layers/picture_layer_impl.h:142-143.
    pub(crate) fn InvalidateRegionForImages(
        &mut self,
        images: &crate::adapter::image_animation::PaintImageIdFlatSet,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> ImageInvalidationResult {
        use crate::adapter::picture_layer_impl::PictureDiscardableImageMap;
        let Some(map) = self
            .state_
            .discardable_image_map_
            .as_ref()
            .filter(|map| !map.empty())
        else {
            return ImageInvalidationResult::kNoImages;
        };
        let mut all_animated = true;
        let controller = c.tree.image_animation_controller();
        let mut image_invalidation =
            crate::cc::base::invalidation_region::InvalidationRegion::new();
        for id in images {
            let rects = map.GetRectsForImage(*id);
            if rects.is_empty() {
                continue;
            }
            all_animated &= controller.IsRegistered(*id);
            for rect in rects {
                image_invalidation.Union(*rect);
            }
        }
        let mut invalidation = Region::default();
        image_invalidation.Swap(&mut invalidation);
        if invalidation.IsEmpty() {
            return ImageInvalidationResult::kNoInvalidation;
        }
        self.base_
            .base_mut()
            .UnionUpdateRect(invalidation.bounds(), &mut c.BaseContext());
        if all_animated {
            self.state_.has_animated_image_update_rect_ = true;
        } else {
            self.state_.has_non_animated_image_update_rect_ = true;
        }
        self.state_.invalidation_.Union(&invalidation);
        self.WithTilingSet(c, |set, client| set.Invalidate(&invalidation, client));
        self.base_.base_mut().SetNeedsPushProperties(
            super::layer_impl::LayerImpl::<B>::kChangedGeneralProperty,
            &mut c.BaseContext(),
        );
        ImageInvalidationResult::kInvalidated
    }
    // Source: cc/layers/picture_layer_impl.cc:1933-1964.
    // Source: cc/layers/picture_layer_impl.h:145-146.
    pub(crate) fn InvalidateRasterInducingScrolls(
        &mut self,
        scrolls: &std::collections::BTreeSet<crate::adapter::input_types::ElementId>,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        use crate::adapter::picture_layer_impl::{
            PictureDisplayItemList, PictureDisplayItemListFor,
        };
        let Some(list) = self
            .state_
            .raster_source_
            .as_ref()
            .and_then(|source| source.GetDisplayItemList().as_ref())
        else {
            return;
        };
        let inducing = list.raster_inducing_scrolls();
        let mut invalidation = Region::default();
        let mut update_map = false;
        for element in scrolls {
            if let Some(info) = inducing.get(element) {
                self.base_
                    .base_mut()
                    .UnionUpdateRect(info.visual_rect, &mut c.BaseContext());
                self.state_.has_non_animated_image_update_rect_ = true;
                invalidation.Union(&info.visual_rect);
                update_map |= info.has_discardable_images;
            }
        }
        if !invalidation.IsEmpty() {
            if update_map {
                self.state_.discardable_image_map_ = Some(list.GenerateDiscardableImageMap(
                    self.GetRasterInducingScrollOffsets(&c.BaseContext()),
                    None,
                    None,
                ));
            }
            self.state_.invalidation_.Union(&invalidation);
            self.WithTilingSet(c, |set, client| set.Invalidate(&invalidation, client));
            self.base_.base_mut().SetNeedsPushProperties(
                super::layer_impl::LayerImpl::<B>::kChangedGeneralProperty,
                &mut c.BaseContext(),
            );
        }
    }
    // Source: cc/layers/picture_layer_impl.cc:662-686.
    // Source: cc/layers/picture_layer_impl.h:109-109.
    pub(crate) fn RegenerateDiscardableImageMap(
        &mut self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        use crate::adapter::picture_layer_impl::{
            PictureDiscardableImageMap, PictureDisplayItemListFor, PictureImageDecodingModeMap,
            PicturePaintWorkletInputs,
        };
        assert!(c.tree.IsSyncTree());
        let had = self.HasAnimatedImages();
        if let Some(list) = self
            .state_
            .raster_source_
            .as_ref()
            .expect("source nonnull raster")
            .GetDisplayItemList()
            .as_ref()
        {
            let mut decoding = PictureImageDecodingModeMap::new();
            let mut worklets = PicturePaintWorkletInputs::<B>::new();
            self.state_.discardable_image_map_ = Some(list.GenerateDiscardableImageMap(
                self.GetRasterInducingScrollOffsets(&c.BaseContext()),
                Some(&mut decoding),
                Some(&mut worklets),
            ));
            let controller = c.tree.image_animation_controller();
            for (_, data) in self
                .state_
                .discardable_image_map_
                .as_ref()
                .unwrap()
                .animated_images_metadata()
            {
                controller.UpdateAnimatedImage(data);
            }
            self.SetPaintWorkletInputs(&worklets, c);
            c.tree.UpdateImageDecodingHints(decoding);
        } else {
            self.SetPaintWorkletInputs(&[], c);
            self.state_.discardable_image_map_ = None;
        }
        if had != self.HasAnimatedImages() {
            c.tree.NotifyLayerHasAnimatedImagesChanged(
                self.base_.base().id(),
                self.HasAnimatedImages(),
            );
        }
    }
}

// Source: cc/layers/picture_layer_impl.cc:1637-1645. Single native body.
// Source: cc/layers/picture_layer_impl.cc:1637-1645.
fn MinimumContentsScaleForState<B: PictureLayerImplTypes>(state: &PictureLayerImplState<B>) -> f32 {
    let recorded = state
        .raster_source_
        .as_ref()
        .expect("source nonnull raster")
        .recorded_bounds()
        .size();
    let min_dimension = recorded.width().min(recorded.height());
    if min_dimension != 0 {
        1. / min_dimension as f32
    } else {
        1.
    }
}
// Source: cc/layers/picture_layer_impl.cc:1647-1667. Single native body.
// Source: cc/layers/picture_layer_impl.cc:1647-1667.
fn MaximumContentsScaleForBase<B: PictureLayerImplTypes>(
    base: &TileBasedLayerImpl<B, PictureLayerTiling<B>>,
    tree: &dyn crate::adapter::layer_impl::LayerImplReadTree<B>,
) -> f32 {
    MaximumContentsScaleForBaseLayer(base.base(), base.is_backdrop_filter_mask(), tree)
}
// Source: cc/layers/picture_layer_impl.cc:1677-1692. Single native body.
// Source: cc/layers/picture_layer_impl.cc:1677-1692.
fn CanHaveTilingsForParts<B: PictureLayerImplTypes>(
    base: &TileBasedLayerImpl<B, PictureLayerTiling<B>>,
    state: &PictureLayerImplState<B>,
    tree: &dyn crate::adapter::layer_impl::LayerImplReadTree<B>,
) -> bool {
    CanHaveTilingsForBaseLayerParts(
        base.base(),
        state,
        tree,
        base.solid_color().is_some(),
        base.is_backdrop_filter_mask(),
    )
}
/// Only actual remainder references, never an owning/cloneable state object.
/// The owning Set can be mutably lent independently for native activation.
pub(crate) struct PictureLayerImplReadView<'a, B: PictureLayerImplTypes> {
    base: &'a TileBasedLayerImpl<B, PictureLayerTiling<B>>,
    state: &'a PictureLayerImplState<B>,
    identity: PictureLayerImplId,
    tree: &'a dyn crate::adapter::picture_layer_impl::PictureLayerImplReadTree<B>,
}
impl<B: PictureLayerImplTypes> PictureLayerImplReadView<'_, B> {
    fn identity(&self) -> PictureLayerImplId {
        self.identity
    }
    // Source: cc/layers/picture_layer_impl.cc:1677-1692.
    // Source: cc/layers/picture_layer_impl.h:120-120.
    fn CanHaveTilings(&self) -> bool {
        CanHaveTilingsForParts(self.base, self.state, self.tree)
    }
    // Source: cc/layers/picture_layer_impl.h:350-352.
    fn raster_contents_scale_key(&self) -> f32 {
        crate::adapter::geometry::cpp_max(
            self.state.raster_contents_scale_.x(),
            self.state.raster_contents_scale_.y(),
        )
    }
    // Source: cc/layers/picture_layer_impl.cc:2124-2126.
    // Source: cc/layers/picture_layer_impl.h:346-346.
    fn GetIdealContentsScaleKey(&self) -> f32 {
        crate::adapter::geometry::cpp_max(
            self.state.ideal_contents_scale_.x(),
            self.state.ideal_contents_scale_.y(),
        )
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    pub(crate) fn ReadView<'a>(
        &'a self,
        tree: &'a dyn crate::adapter::picture_layer_impl::PictureLayerImplReadTree<B>,
    ) -> PictureLayerImplReadView<'a, B> {
        assert_eq!(self.base_.base().tree_id(), tree.id());
        PictureLayerImplReadView {
            base: &self.base_,
            state: &self.state_,
            identity: self.identity(),
            tree,
        }
    }
    pub(crate) fn SplitForActivation<'a>(
        &'a mut self,
        tree: &'a dyn crate::adapter::picture_layer_impl::PictureLayerImplReadTree<B>,
    ) -> (
        PictureLayerImplReadView<'a, B>,
        &'a mut PictureLayerTilingSet<B>,
    ) {
        let identity = self.identity();
        assert_eq!(identity.tree, tree.id());
        (
            PictureLayerImplReadView {
                base: &self.base_,
                state: &self.state_,
                identity,
                tree,
            },
            self.tilings_
                .as_deref_mut()
                .expect("native selected pending Set"),
        )
    }
}
// Source: cc/layers/picture_layer_impl.cc:516-521. Single native twin check.
// Source: cc/layers/picture_layer_impl.cc:516-520.
fn PendingOrActiveTwinRead<'a, B: PictureLayerImplTypes>(
    identity: Option<PictureLayerImplId>,
    environment: &'a dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<B>,
) -> Option<PictureLayerImplReadView<'a, B>> {
    let identity = PendingOrActiveTwinId(identity, environment)?;
    let twin = environment.TwinLayerRead(identity);
    assert_eq!(twin.identity(), identity, "native twin identity");
    Some(twin)
}
// Native UpdateRasterSourceInternal has precisely two caller modes. A commit
// carries the unique source updater sink; activation carries the actual pending
// read view and tiling set and has no authority to append commit updates.
pub(crate) enum RasterSourceUpdate<'a, B: PictureLayerImplTypes> {
    Commit {
        updates: &'a mut Vec<i32>,
    },
    Activation {
        pending_view: PictureLayerImplReadView<'a, B>,
        tiling_set: &'a mut PictureLayerTilingSet<B>,
    },
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.cc:115-118.
    // Source: cc/layers/picture_layer_impl.h:56-57.
    pub(crate) fn CreateLayerImpl(
        &self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> Box<B::Layer> {
        Self::Create(c.tree.id(), self.base_.base().id(), c)
    }
    // Source: cc/layers/picture_layer_impl.cc:529-535.
    // Source: cc/layers/picture_layer_impl.h:106-106.
    pub(crate) fn CommitPendingRasterSource(
        &mut self,
        updates: &mut Vec<i32>,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        let source = self
            .state_
            .pending_raster_source_
            .take()
            .expect("source CHECK pending raster");
        let invalidation = std::mem::take(&mut self.state_.pending_invalidation_);
        self.UpdateRasterSourceInternal(
            source,
            invalidation,
            RasterSourceUpdate::Commit { updates },
            c,
        );
    }
    // Source: cc/layers/picture_layer_impl.cc:537-652.
    // Source: cc/layers/picture_layer_impl.h:254-259.
    pub(crate) fn UpdateRasterSourceInternal(
        &mut self,
        source: Arc<B::RasterSource>,
        mut invalidation: Region<<B as LayerTreeTypes>::RegionBackend>,
        mut update: RasterSourceUpdate<'_, B>,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        use crate::adapter::picture_layer_impl::PictureDisplayItemList;
        assert!(matches!(update, RasterSourceUpdate::Activation { .. }) || c.tree.IsSyncTree());
        debug_assert!(
            source.size().IsEmpty() || self.base_.base().bounds(&c.BaseContext()) == source.size()
        );
        self.base_.base_mut().SetNeedsPushProperties(
            super::layer_impl::LayerImpl::<B>::kChangedGeneralProperty,
            &mut c.BaseContext(),
        );
        if self
            .state_
            .raster_source_
            .as_ref()
            .is_none_or(|old| old.size() != source.size())
        {
            self.state_.raster_source_size_changed_ = true;
        }
        let recording_updated = self.state_.raster_source_.as_ref().is_none_or(|old| {
            !SameAllocation(old.GetDisplayItemList(), source.GetDisplayItemList())
        });
        if recording_updated {
            if let Some(old) = self.state_.raster_source_.as_ref() {
                if let (Some(current_list), Some(new_list)) = (
                    old.GetDisplayItemList().as_ref(),
                    source.GetDisplayItemList().as_ref(),
                ) {
                    let mut full = c.tree.GetMSAASampleCountForRaster(current_list)
                        != c.tree.GetMSAASampleCountForRaster(new_list);
                    full |= c
                        .tree
                        .GetTargetColorParams(current_list.content_color_usage())
                        != c.tree.GetTargetColorParams(new_list.content_color_usage());
                    if full {
                        let size = source.size();
                        invalidation.Union(&Rect::new(0, 0, size.width(), size.height()));
                    }
                }
            }
        }
        let could = self.CanHaveTilings(&c.BaseContext());
        self.state_.raster_source_ = Some(source);
        self.base_.SetSolidColor(None);
        if self.state_.raster_source_.as_ref().unwrap().IsSolidColor() {
            self.base_.SetSolidColor(Some(
                self.state_.raster_source_.as_ref().unwrap().GetSolidColor(),
            ));
        }
        self.state_
            .raster_source_
            .as_ref()
            .unwrap()
            .set_debug_name(&self.base_.base().DebugName());
        self.UpdateDirectlyCompositedImageFromRasterSource(&mut c.BaseContext());
        if let RasterSourceUpdate::Activation {
            pending_view: read, ..
        } = &update
        {
            if !SameAllocation(
                &read.state.discardable_image_map_,
                &self.state_.discardable_image_map_,
            ) {
                let had = self.HasAnimatedImages();
                self.state_.paint_worklet_records_ = read.state.paint_worklet_records_.clone();
                self.state_.discardable_image_map_ =
                    read.state.discardable_image_map_.as_ref().map(Arc::clone);
                if had != self.HasAnimatedImages() {
                    c.tree.NotifyLayerHasAnimatedImagesChanged(
                        self.base_.base().id(),
                        self.HasAnimatedImages(),
                    );
                }
            }
        } else if let RasterSourceUpdate::Commit { updates } = &mut update {
            if recording_updated {
                c.tree
                    .AddLayerNeedingUpdateDiscardableImageMap(self.base_.base().id(), updates);
            }
        }
        self.state_.invalidation_ = invalidation;
        let can = self.CanHaveTilings(&c.BaseContext());
        debug_assert!(match &update {
            RasterSourceUpdate::Activation {
                pending_view: read, ..
            } => can == read.CanHaveTilings(),
            RasterSourceUpdate::Commit { .. } => true,
        });
        if could != can {
            c.tree.set_needs_update_draw_properties();
        }
        if !can {
            self.RemoveAllTilings(c);
            return;
        }
        let min = self.MinimumContentsScale();
        let max = self.MaximumContentsScale(&c.BaseContext());
        let source = Arc::clone(self.state_.raster_source_.as_ref().unwrap());
        if let RasterSourceUpdate::Activation {
            tiling_set: set, ..
        } = update
        {
            self.WithTilingSet(c, |active, client| {
                let invalidation = client.state.invalidation_;
                active.UpdateTilingsToCurrentRasterSourceForActivation(
                    source,
                    set,
                    invalidation,
                    min,
                    max,
                    client,
                );
            });
        } else {
            self.WithTilingSet(c, |set, client| {
                let invalidation = client.state.invalidation_;
                set.UpdateTilingsToCurrentRasterSourceForCommit(
                    source,
                    invalidation,
                    min,
                    max,
                    client,
                );
            });
        }
    }
    // Source: cc/layers/picture_layer_impl.cc:654-660.
    // Source: cc/layers/picture_layer_impl.h:107-108.
    pub(crate) fn SetRasterSourceForTesting(
        &mut self,
        source: Arc<B::RasterSource>,
        invalidation: &Region<<B as LayerTreeTypes>::RegionBackend>,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        let mut action = RasterSourceTestingUpdate {
            picture: self,
            source: Some(source),
            invalidation,
            services: c.services,
            tiling: c.tiling,
        };
        c.tree.WithDiscardableImageMapUpdater(&mut action);
    }
    // Source: cc/layers/picture_layer_impl.cc:152-209.
    // Source: cc/layers/picture_layer_impl.h:59-59.
    pub(crate) fn MovePropertiesToActiveLayer(
        &mut self,
        target: &mut Self,
        source: &crate::adapter::picture_layer_impl::PictureLayerSourceContext<'_, B>,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        // Native LayerImpl::MovePropertiesToActiveLayer is solely the virtual
        // CopyPropertiesTo call. This exact most-derived target invokes the
        // canonical Picture override and the single canonical base copy body.
        self.CopyPropertiesTo(target, &source.BaseReadContext(), &mut c.BaseContext());
        debug_assert!(
            self.state_.twin_layer_.is_none() || self.state_.twin_layer_ == Some(target.identity())
        );
        debug_assert!(
            self.state_.twin_layer_.is_none() || target.state_.twin_layer_ == Some(self.identity())
        );
        self.state_.twin_layer_ = Some(target.identity());
        target.state_.twin_layer_ = Some(self.identity());
        if source.tree.TreesInVizInClientProcess() {
            let changed = self
                .base_
                .base()
                .GetChangeFlag(super::layer_impl::LayerImpl::<B>::kChangedTile);
            if changed {
                for (scale, set_source) in &mut self.state_.updated_tiles_ {
                    match target.state_.updated_tiles_.entry(*scale) {
                        std::collections::btree_map::Entry::Vacant(entry) => {
                            entry.insert(std::mem::take(set_source));
                        }
                        std::collections::btree_map::Entry::Occupied(mut entry) => {
                            // std::set::merge keeps duplicate keys in source,
                            // preserving the destination bool without OR-ing.
                            set_source.retain(|index, damage| {
                                if let std::collections::btree_map::Entry::Vacant(slot) =
                                    entry.get_mut().entry(*index)
                                {
                                    slot.insert(*damage);
                                    false
                                } else {
                                    true
                                }
                            });
                        }
                    }
                }
                self.state_.updated_tiles_.clear();
            } else {
                debug_assert!(
                    self.state_.updated_tiles_.is_empty(),
                    "kChangedTile flag should be set!"
                );
            }
            target.state_.should_batch_updated_tiles_ = true;
        }
        if self
            .base_
            .base()
            .GetChangeFlag(super::layer_impl::LayerImpl::<B>::kChangedGeneralProperty)
        {
            let raster = Arc::clone(self.state_.raster_source_.as_ref().expect("source raster"));
            let invalidation = std::mem::take(&mut self.state_.invalidation_);
            let (read, set) = self.SplitForActivation(source.tree);
            target.UpdateRasterSourceInternal(
                raster,
                invalidation,
                RasterSourceUpdate::Activation {
                    pending_view: read,
                    tiling_set: set,
                },
                c,
            );
            debug_assert!(self.state_.invalidation_.IsEmpty());
        }
        debug_assert!(
            self.base_.solid_color().is_none()
                || target.picture_layer_tiling_set().num_tilings() == 0
        );
        #[cfg(debug_assertions)]
        target.SanityCheckTilingState(&c.BaseContext());
    }
}
fn SameAllocation<T>(one: &Option<Arc<T>>, two: &Option<Arc<T>>) -> bool {
    match (one, two) {
        (Some(one), Some(two)) => Arc::ptr_eq(one, two),
        (None, None) => true,
        _ => false,
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.cc:211-266.
    // Source: cc/layers/picture_layer_impl.h:403-408.
    pub(crate) fn AppendQuadsForResourcelessSoftwareDraw(
        &self,
        _context: &super::append_quads_context::AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut super::append_quads_data::AppendQuadsData<B::SurfaceId>,
        shared: B::SharedQuadStateHandle,
        occlusion: &crate::cc::trees::occlusion::Occlusion<B::Transform>,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        AppendResourcelessForParts(
            &self.base_,
            &self.state_,
            self.picture_layer_tiling_set(),
            _context,
            pass,
            data,
            shared,
            occlusion,
            c,
        )
    }
    // Source: cc/layers/picture_layer_impl.cc:280-296.
    // Source: cc/layers/picture_layer_impl.h:420-423.
    pub(crate) fn DidAppendQuad(
        &self,
        quad: &B::DrawQuad,
        iter:&crate::cc::tiles::tiling_set_coverage_iterator::TilingSetCoverageIterator<'_,PictureLayerTiling<B>,<B as LayerTreeTypes>::RegionBackend>,
        _data: &mut super::append_quads_data::AppendQuadsData<B::SurfaceId>,
        checkerboard: bool,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        DidAppendQuadForBase(self.base_.base(), quad, iter, _data, checkerboard, c)
    }
    // Source: cc/layers/picture_layer_impl.cc:1789-1794.
    // Source: cc/layers/picture_layer_impl.h:266-267.
    pub(crate) fn GetAllPrioritizedTilesForTracing(
        &mut self,
        tiles: &mut Vec<crate::cc::tiles::prioritized_tile::PrioritizedTile>,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        if self.tilings_.is_none() {
            return;
        }
        self.WithTilingSet(c, |set, client| {
            set.GetAllPrioritizedTilesForTracing(tiles, client)
        });
    }
    // Source: cc/layers/picture_layer_impl.cc:1796-1873.
    // Source: cc/layers/picture_layer_impl.h:268-268.
    pub(crate) fn AsValueInto(
        &self,
        state: &mut B::LayerTraceValue,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        use crate::adapter::{geometry::TracedValue, property_types::PropertyTrace};
        use crate::cc::base::math_util::MathUtil;
        self.base_
            .base()
            .AsValueInto(&PictureTraceReceiver(self), state, &c.BaseContext());
        state.SetDouble(
            "ideal_contents_scale",
            self.GetIdealContentsScaleKey() as f64,
        );
        state.SetDouble(
            "geometry_contents_scale",
            self.MaximumTilingContentsScale() as f64,
        );
        state.BeginArray("tilings");
        self.picture_layer_tiling_set().AsValueInto(state);
        state.EndArray();
        MathUtil::AddToTracedValue(
            "tile_priority_rect",
            &self
                .state_
                .viewport_rect_for_tile_priority_in_content_space_,
            state,
        );
        MathUtil::AddToTracedValue(
            "visible_rect",
            &self.base_.base().visible_layer_rect(),
            state,
        );
        state.SetString(
            "lcd_text_disallowed_reason",
            self.state_.lcd_text_disallowed_reason_.as_str(),
        );
        state.BeginArray("pictures");
        self.state_
            .raster_source_
            .as_ref()
            .unwrap()
            .AsValueInto(state);
        state.EndArray();
        state.BeginArray("invalidation");
        self.state_.invalidation_.AsValueInto(state);
        state.EndArray();
        state.BeginArray("coverage_tiles");
        let bounds = self.base_.base().bounds(&c.BaseContext());
        let mut iter = self.picture_layer_tiling_set().Cover(
            Rect::new(0, 0, bounds.width(), bounds.height()),
            self.MaximumTilingContentsScale(),
            self.GetIdealContentsScaleKey(),
        );
        while iter.IsValid() {
            state.BeginDictionaryUnnamed();
            MathUtil::AddToTracedValue("geometry_rect", &iter.geometry_rect(), state);
            if let Some(tile) = iter.current_tile() {
                c.services.SetTileIDRef(
                    crate::adapter::tile::TileTraceId(std::ptr::from_ref(tile) as usize),
                    state,
                    "tile",
                );
            }
            state.EndDictionary();
            iter.Advance();
        }
        state.EndArray();
        state.BeginDictionary("can_have_tilings_state");
        state.SetBoolean("can_have_tilings", self.CanHaveTilings(&c.BaseContext()));
        state.SetBoolean(
            "raster_source_solid_color",
            self.base_.solid_color().is_some(),
        );
        state.SetBoolean("draws_content", self.base_.base().draws_content());
        state.SetBoolean(
            "raster_source_has_recordings",
            self.state_.raster_source_.as_ref().unwrap().HasRecordings(),
        );
        state.SetDouble(
            "max_contents_scale",
            self.MaximumTilingContentsScale() as f64,
        );
        state.SetDouble("min_contents_scale", self.MinimumContentsScale() as f64);
        state.EndDictionary();
        state.BeginDictionary("raster_scales");
        state.SetDouble("page_scale", self.state_.raster_page_scale_ as f64);
        state.SetDouble("device_scale", self.state_.raster_device_scale_ as f64);
        state.BeginArray("source_scale");
        state.AppendDouble(self.state_.raster_source_scale_.x() as f64);
        state.AppendDouble(self.state_.raster_source_scale_.y() as f64);
        state.EndArray();
        state.BeginArray("contents_scale");
        state.AppendDouble(self.state_.raster_contents_scale_.x() as f64);
        state.AppendDouble(self.state_.raster_contents_scale_.y() as f64);
        state.EndArray();
        state.EndDictionary();
        state.BeginDictionary("ideal_scales");
        state.SetDouble("page_scale", self.state_.ideal_page_scale_ as f64);
        state.SetDouble("device_scale", self.state_.ideal_device_scale_ as f64);
        state.BeginArray("source_scale");
        state.AppendDouble(self.state_.ideal_source_scale_.x() as f64);
        state.AppendDouble(self.state_.ideal_source_scale_.y() as f64);
        state.EndArray();
        state.BeginArray("contents_scale");
        state.AppendDouble(self.state_.ideal_contents_scale_.x() as f64);
        state.AppendDouble(self.state_.ideal_contents_scale_.y() as f64);
        state.EndArray();
        state.EndDictionary();
    }
    // Source: cc/layers/picture_layer_impl.cc:1879-1881.
    // Source: cc/layers/picture_layer_impl.h:118-118.
    pub(crate) fn RunMicroBenchmark(
        &mut self,
        benchmark: &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplBenchmark<B>,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        benchmark.RunOnPictureLayer(self, c);
    }
}
// Exact readonly virtual subset invoked by canonical LayerImpl::AsValueInto.
// It is private to the trace call and borrows the actual most-derived owner.
struct PictureTraceReceiver<'a, B: PictureLayerImplTypes>(&'a PictureLayerImpl<B>);
impl<B: PictureLayerImplTypes> super::layer_impl::LayerImplVirtual<B>
    for PictureTraceReceiver<'_, B>
{
    // Source: cc/layers/picture_layer_impl.cc:111-113.
    // Source: cc/layers/picture_layer_impl.h:55-55.
    fn GetLayerType(
        &self,
        _base: &super::layer_impl::LayerImpl<B>,
    ) -> crate::adapter::layer_impl::LayerType {
        self.0.GetLayerType()
    }
    // Source: cc/layers/picture_layer_impl.cc:1875-1877.
    // Source: cc/layers/picture_layer_impl.h:116-116.
    fn GPUMemoryUsageInBytes(&self, _base: &super::layer_impl::LayerImpl<B>) -> usize {
        self.0.GPUMemoryUsageInBytes()
    }
}

// Source: cc/layers/picture_layer_impl.cc:912-913. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:911-913.
fn RecordedBoundsForState<B: PictureLayerImplTypes>(state: &PictureLayerImplState<B>) -> Rect {
    state
        .raster_source_
        .as_ref()
        .map_or(Rect::default(), |source| source.recorded_bounds())
}
// Source: cc/layers/picture_layer_impl.cc:909-911. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:907-909.
fn IsDirectImageForState<B: PictureLayerImplTypes>(state: &PictureLayerImplState<B>) -> bool {
    state.directly_composited_image_default_raster_scale_ > 0.
}
// Source: cc/layers/picture_layer_impl.cc:1714-1717. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:1714-1717.
fn MaximumTilingContentsScaleForParts<B: PictureLayerImplTypes>(
    state: &PictureLayerImplState<B>,
    set: &PictureLayerTilingSet<B>,
) -> f32 {
    let scale = set.GetMaximumContentsScale();
    crate::adapter::geometry::cpp_max(scale, MinimumContentsScaleForState(state))
}
// Source: cc/layers/picture_layer_impl.cc:2095-2103. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:2098-2103.
fn MaximumAppendScaleForParts<B: PictureLayerImplTypes>(
    state: &PictureLayerImplState<B>,
    set: &PictureLayerTilingSet<B>,
) -> f32 {
    if set.num_tilings() != 0 {
        MaximumTilingContentsScaleForParts(state, set)
    } else {
        1.
    }
}
// Source: cc/layers/picture_layer_impl.cc:2105-2115. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:2105-2115.
fn DebugTilingResolutionForState<B: PictureLayerImplTypes>(
    state: &PictureLayerImplState<B>,
    tiling: &PictureLayerTiling<B>,
) -> super::tile_based_layer_impl::TilingResolution {
    use super::tile_based_layer_impl::TilingResolution;
    use crate::cc::tiles::tile_priority::TileResolution;
    if tiling.resolution() == TileResolution::HIGH_RESOLUTION {
        return TilingResolution::kHigh;
    }
    if tiling.contents_scale_key()
        > crate::adapter::geometry::cpp_max(
            state.raster_contents_scale_.x(),
            state.raster_contents_scale_.y(),
        )
    {
        return TilingResolution::kAboveHigh;
    }
    TilingResolution::kBelowHigh
}
// Source: cc/layers/picture_layer_impl.cc:2124-2126. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:2124-2126.
fn IdealContentsScaleKeyForState<B: PictureLayerImplTypes>(
    state: &PictureLayerImplState<B>,
) -> f32 {
    crate::adapter::geometry::cpp_max(
        state.ideal_contents_scale_.x(),
        state.ideal_contents_scale_.y(),
    )
}
// Source: cc/layers/picture_layer_impl.cc:268-270. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:268-271.
fn ShouldUpdateApproximatedVisibleContentAreaForResolution(
    resolution: crate::cc::tiles::tile_priority::TileResolution,
) -> bool {
    resolution != crate::cc::tiles::tile_priority::TileResolution::HIGH_RESOLUTION
}
// Source: cc/layers/picture_layer_impl.cc:272-278. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:273-278.
fn ShouldReportTileAsMissingForRects(geometry: Rect, viewport: Rect) -> bool {
    geometry.Intersects(viewport)
}
// Source: cc/layers/picture_layer_impl.cc:298-305. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:298-305.
fn WillProcessReadyToDrawTileForIterator<B: PictureLayerImplTypes>(
    iter: &crate::cc::tiles::tiling_set_coverage_iterator::TilingSetCoverageIterator<
        '_,
        PictureLayerTiling<B>,
        <B as LayerTreeTypes>::RegionBackend,
    >,
) {
    if let Some(tile) = iter.current_tile() {
        tile.mark_used();
    }
}
// Source: cc/layers/picture_layer_impl.cc:370-372. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:370-372.
fn WillAppendQuadsForBase<B: PictureLayerImplTypes>(
    base: &mut TileBasedLayerImpl<B, PictureLayerTiling<B>>,
) {
    base.set_produced_tile_last_append_quads(false);
}
// Source: cc/layers/picture_layer_impl.cc:374-378. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:374-378.
fn ScaledViewportForState<B: PictureLayerImplTypes>(
    state: &PictureLayerImplState<B>,
    scale: f32,
) -> Rect {
    crate::adapter::geometry::ScaleToEnclosingRect(
        state.viewport_rect_for_tile_priority_in_content_space_,
        scale,
    )
}
// Source: cc/layers/picture_layer_impl.cc:1775-1788. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:1775-1787.
fn DebugBorderPropertiesForState<B: PictureLayerImplTypes>(
    state: &PictureLayerImplState<B>,
    c: &LayerImplContext<'_, B>,
) -> (crate::adapter::layer_tree_impl::Color4f, f32) {
    use crate::cc::debug::debug_colors::DebugColors;
    let dsf = c.tree.device_scale_factor();
    if IsDirectImageForState(state) {
        (
            DebugColors::ImageLayerBorderColor(),
            DebugColors::ImageLayerBorderWidth(dsf) as f32,
        )
    } else {
        (
            DebugColors::TiledContentLayerBorderColor(),
            DebugColors::TiledContentLayerBorderWidth(dsf) as f32,
        )
    }
}
// Source: cc/layers/picture_layer_impl.cc:1694-1712. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:1694-1712.
fn SanityCheckTilingStateForParts<B: PictureLayerImplTypes>(
    base: &TileBasedLayerImpl<B, PictureLayerTiling<B>>,
    state: &PictureLayerImplState<B>,
    set: &PictureLayerTilingSet<B>,
    c: &LayerImplContext<'_, B>,
) {
    #[cfg(not(debug_assertions))]
    let _ = (base, state, set, c);
    #[cfg(debug_assertions)]
    {
        if !CanHaveTilingsForParts(base, state, c.tree) {
            debug_assert_eq!(0, set.num_tilings());
            return;
        }
        if set.num_tilings() == 0 {
            return;
        }
        if c.tree.TreesInVizInClientProcess() {
            return;
        }
        debug_assert_eq!(1, set.NumHighResTilings());
    }
}
// Source: cc/layers/picture_layer_impl.cc:211-266. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:211-266.
fn AppendResourcelessForParts<B: PictureLayerImplTypes>(
    base: &TileBasedLayerImpl<B, PictureLayerTiling<B>>,
    state: &PictureLayerImplState<B>,
    set: &PictureLayerTilingSet<B>,
    _context: &super::append_quads_context::AppendQuadsContext,
    pass: &mut B::RenderPass,
    data: &mut super::append_quads_data::AppendQuadsData<B::SurfaceId>,
    shared: B::SharedQuadStateHandle,
    occlusion: &crate::cc::trees::occlusion::Occlusion<B::Transform>,
    c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
) {
    use crate::adapter::{
        picture_layer_impl::{PictureDiscardableImageMap, PictureRenderPass},
        solid_color_layer_impl::SolidColorRenderPass,
        tile_based_layer_impl::{TileBasedRenderPass, TileBasedSharedQuadState},
    };
    use crate::cc::debug::debug_colors::DebugColors;
    debug_assert_eq!(pass.SharedQuadState(shared).quad_layer_rect().x, 0);
    debug_assert_eq!(pass.SharedQuadState(shared).quad_layer_rect().y, 0);
    let max = MaximumAppendScaleForParts(state, set);
    let dsf = c.tree.device_scale_factor();
    base.base().AppendDebugBorderQuadWithColor(
        pass,
        pass.SharedQuadState(shared).quad_layer_rect(),
        shared,
        data,
        DebugColors::DirectPictureBorderColor(),
        DebugColors::DirectPictureBorderWidth(dsf) as f32,
        &c.BaseContext(),
    );
    let mut geometry = pass.SharedQuadState(shared).visible_quad_layer_rect();
    let mut visible = occlusion.GetUnoccludedContentRect(geometry);
    let blending = !base.base().contents_opaque();
    let source = state.raster_source_.as_ref().expect("source raster");
    let recorded = crate::adapter::geometry::ScaleToEnclosingRect(source.recorded_bounds(), max);
    geometry.Intersect(recorded);
    visible.Intersect(recorded);
    if visible.IsEmpty() {
        return;
    }
    debug_assert!(source.HasRecordings());
    let content = pass.SharedQuadState(shared).visible_quad_layer_rect();
    let texture_size = content.size();
    let texture = crate::adapter::geometry::RectF::new(
        0.,
        0.,
        texture_size.width() as f32,
        texture_size.height() as f32,
    );
    let mut image_animation = std::collections::BTreeMap::new();
    let which = if c.tree.IsPendingTree() {
        crate::adapter::image_animation::WhichTree::PENDING_TREE
    } else {
        crate::adapter::image_animation::WhichTree::ACTIVE_TREE
    };
    let controller = c.tree.image_animation_controller();
    for (_, metadata) in state
        .discardable_image_map_
        .as_ref()
        .expect("source image map")
        .animated_images_metadata()
    {
        image_animation.insert(
            metadata.paint_image_id,
            controller.GetFrameIndexForImage(metadata.paint_image_id, which),
        );
    }
    let quad = pass.CreateAndAppendPictureDrawQuad();
    pass.PictureDrawQuadSetNew(
        quad,
        shared,
        geometry,
        visible,
        blending,
        texture,
        base.GetNearestNeighbor(),
        content,
        max,
        image_animation,
        source.GetDisplayItemList().as_ref().map(Arc::clone),
        RasterInducingScrollOffsetsForSource::<B>(state.raster_source_.as_deref(), c.tree),
    );
    base.base()
        .ValidateQuadResources(pass.DrawQuad(quad), &c.BaseContext());
}
// Source: cc/layers/picture_layer_impl.cc:280-296. Single canonical body.
// Source: cc/layers/picture_layer_impl.cc:280-296.
fn DidAppendQuadForBase<B: PictureLayerImplTypes>(
    base: &super::layer_impl::LayerImpl<B>,
    quad: &B::DrawQuad,
    iter: &crate::cc::tiles::tiling_set_coverage_iterator::TilingSetCoverageIterator<
        '_,
        PictureLayerTiling<B>,
        <B as LayerTreeTypes>::RegionBackend,
    >,
    _data: &mut super::append_quads_data::AppendQuadsData<B::SurfaceId>,
    checkerboard: bool,
    c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
) {
    base.ValidateQuadResources(quad, &c.BaseContext());
    if checkerboard {
        if let Some(tile) = iter.current_tile() {
            c.services
                .MissedLCPCandidateImageDeadline(tile.HasMissingLCPCandidateImages());
        }
    }
}
// Source: cc/layers/picture_layer_impl.cc:1677-1692.
fn CanHaveTilingsForBaseLayerParts<B: PictureLayerImplTypes>(
    base: &super::layer_impl::LayerImpl<B>,
    state: &PictureLayerImplState<B>,
    tree: &dyn crate::adapter::layer_impl::LayerImplReadTree<B>,
    solid_color: bool,
    backdrop: bool,
) -> bool {
    let Some(source) = state.raster_source_.as_ref() else {
        return false;
    };
    if solid_color {
        return false;
    }
    if !base.draws_content() {
        return false;
    }
    if !source.HasRecordings() {
        return false;
    }
    debug_assert!(!source.size().IsEmpty());
    if MaximumContentsScaleForBaseLayer(base, backdrop, tree) < MinimumContentsScaleForState(state)
    {
        return false;
    }
    true
}
// Source: cc/layers/picture_layer_impl.cc:1647-1667.
fn MaximumContentsScaleForBaseLayer<B: PictureLayerImplTypes>(
    base: &super::layer_impl::LayerImpl<B>,
    backdrop: bool,
    tree: &dyn crate::adapter::layer_impl::LayerImplReadTree<B>,
) -> f32 {
    if base.bounds_for_tree(tree).IsEmpty() {
        return 0.;
    }
    let max_dimension = if backdrop {
        tree.max_texture_size()
    } else {
        i32::MAX
    } as f32;
    let higher = base
        .bounds_for_tree(tree)
        .width()
        .max(base.bounds_for_tree(tree).height());
    let scale = max_dimension / higher as f32;
    // Native nextafterf(scale, 0) maps directly to the matching f32
    // neighboring-value primitive, retaining +/-zero target semantics.
    if scale == 0. {
        0.
    } else if scale > 0. {
        scale.next_down()
    } else {
        scale.next_up()
    }
}
// The actual most-derived readonly siblings beside TileBasedLayerImpl. No base
// or owner pointer is retained, and Cover returns the resident Set cursor.
struct PictureTileBasedDelegate<'a, B: PictureLayerImplTypes> {
    state: &'a PictureLayerImplState<B>,
    tilings: &'a PictureLayerTilingSet<B>,
}
impl<B: PictureLayerImplTypes>
    super::tile_based_layer_impl::TileBasedLayerDelegate<B, PictureLayerTiling<B>>
    for PictureTileBasedDelegate<'_, B>
{
    type Context<'context> =
        crate::adapter::picture_layer_impl::PictureLayerImplContext<'context, B>;
    // Source: cc/layers/picture_layer_impl.cc:1775-1787.
    // Source: cc/layers/picture_layer_impl.h:265-265.
    fn GetDebugBorderProperties(
        &self,
        _base: &super::layer_impl::LayerImpl<B>,
        c: &LayerImplContext<'_, B>,
    ) -> (crate::adapter::layer_tree_impl::Color4f, f32) {
        DebugBorderPropertiesForState(self.state, c)
    }
    // Source: cc/layers/picture_layer_impl.cc:2098-2103.
    // Source: cc/layers/picture_layer_impl.h:102-102.
    fn GetMaximumContentsScaleForUseInAppendQuads(&self) -> f32 {
        MaximumAppendScaleForParts(self.state, self.tilings)
    }
    // Source: cc/layers/picture_layer_impl.cc:911-913.
    // Source: cc/layers/picture_layer_impl.h:209-209.
    fn RecordedBounds(&self) -> Rect {
        RecordedBoundsForState(self.state)
    }
    // Source: cc/layers/picture_layer_impl.cc:907-909.
    // Source: cc/layers/picture_layer_impl.h:208-208.
    fn IsDirectlyCompositedImage(&self) -> bool {
        IsDirectImageForState(self.state)
    }
    // Source: cc/layers/picture_layer_impl.cc:2105-2115.
    // Source: cc/layers/picture_layer_impl.h:428-429.
    fn GetTilingResolutionForDebugBorders(
        &self,
        tiling: &PictureLayerTiling<B>,
    ) -> super::tile_based_layer_impl::TilingResolution {
        DebugTilingResolutionForState(self.state, tiling)
    }
    // Source: cc/layers/picture_layer_impl.cc:2117-2122.
    // Source: cc/layers/picture_layer_impl.h:409-412.
    fn Cover(
        &self,
        rect: Rect,
        scale: f32,
        ideal: f32,
    ) -> crate::cc::tiles::tiling_set_coverage_iterator::TilingSetCoverageIterator<
        '_,
        PictureLayerTiling<B>,
        <B as LayerTreeTypes>::RegionBackend,
    > {
        self.tilings.Cover(rect, scale, ideal)
    }
    // Source: cc/layers/picture_layer_impl.cc:2124-2126.
    // Source: cc/layers/picture_layer_impl.h:346-346.
    fn GetIdealContentsScaleKey(&self) -> f32 {
        IdealContentsScaleKeyForState(self.state)
    }
    // Source: cc/layers/picture_layer_impl.cc:298-305.
    // Source: cc/layers/picture_layer_impl.h:413-414.
    fn WillProcessReadyToDrawTile(
        &self,
        iter:&crate::cc::tiles::tiling_set_coverage_iterator::TilingSetCoverageIterator<'_,PictureLayerTiling<B>,<B as LayerTreeTypes>::RegionBackend>,
    ) {
        WillProcessReadyToDrawTileForIterator::<B>(iter)
    }
    // Source: cc/layers/picture_layer_impl.cc:268-271.
    // Source: cc/layers/picture_layer_impl.h:415-416.
    fn ShouldUpdateApproximatedVisibleContentArea(
        &self,
        resolution: crate::cc::tiles::tile_priority::TileResolution,
    ) -> bool {
        ShouldUpdateApproximatedVisibleContentAreaForResolution(resolution)
    }
    // Source: cc/layers/picture_layer_impl.cc:273-278.
    // Source: cc/layers/picture_layer_impl.h:417-419.
    fn ShouldReportTileAsMissing(&self, geometry: Rect, viewport: Rect) -> bool {
        ShouldReportTileAsMissingForRects(geometry, viewport)
    }
    // Source: cc/layers/picture_layer_impl.cc:374-378.
    // Source: cc/layers/picture_layer_impl.h:425-425.
    fn GetScaledViewportForTilePriority(&self, scale: f32) -> Rect {
        ScaledViewportForState(self.state, scale)
    }
    // Source: cc/layers/picture_layer_impl.cc:280-296.
    // Source: cc/layers/picture_layer_impl.h:420-423.
    fn DidAppendQuad(
        &self,
        base: &super::layer_impl::LayerImpl<B>,
        quad: &B::DrawQuad,
        iter:&crate::cc::tiles::tiling_set_coverage_iterator::TilingSetCoverageIterator<'_,PictureLayerTiling<B>,<B as LayerTreeTypes>::RegionBackend>,
        data: &mut super::append_quads_data::AppendQuadsData<B::SurfaceId>,
        checkerboard: bool,
        c: &mut Self::Context<'_>,
    ) {
        DidAppendQuadForBase(base, quad, iter, data, checkerboard, c)
    }
    // Source: cc/layers/picture_layer_impl.cc:211-266.
    // Source: cc/layers/picture_layer_impl.h:403-408.
    fn AppendQuadsForResourcelessSoftwareDraw(
        &self,
        base: &TileBasedLayerImpl<B, PictureLayerTiling<B>>,
        context: &super::append_quads_context::AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut super::append_quads_data::AppendQuadsData<B::SurfaceId>,
        shared: B::SharedQuadStateHandle,
        occlusion: &crate::cc::trees::occlusion::Occlusion<B::Transform>,
        c: &mut Self::Context<'_>,
    ) {
        AppendResourcelessForParts(
            base,
            self.state,
            self.tilings,
            context,
            pass,
            data,
            shared,
            occlusion,
            c,
        )
    }
    // Source: cc/layers/picture_layer_impl.cc:370-372.
    // Source: cc/layers/picture_layer_impl.h:402-402.
    fn WillAppendQuads(&self, base: &mut TileBasedLayerImpl<B, PictureLayerTiling<B>>) {
        WillAppendQuadsForBase(base)
    }
    // Source: cc/layers/picture_layer_impl.cc:1694-1712.
    // Source: cc/layers/picture_layer_impl.h:263-263.
    fn SanityCheckTilingState(
        &self,
        base: &TileBasedLayerImpl<B, PictureLayerTiling<B>>,
        c: &LayerImplContext<'_, B>,
    ) {
        SanityCheckTilingStateForParts(base, self.state, self.tilings, c)
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Source: cc/layers/picture_layer_impl.h:41-44. Inherited AppendQuads calls
    // the source TileBased template with actual split derived sibling dispatch.
    pub(crate) fn AppendQuads(
        &mut self,
        context: &super::append_quads_context::AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut super::append_quads_data::AppendQuadsData<B::SurfaceId>,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) {
        let delegate = PictureTileBasedDelegate {
            state: &self.state_,
            tilings: self.tilings_.as_deref().expect("native tilings"),
        };
        self.base_.AppendQuads(&delegate, context, pass, data, c);
    }
    pub(crate) fn HasMissingTiles(&self, c: &LayerImplContext<'_, B>) -> bool {
        let delegate = PictureTileBasedDelegate {
            state: &self.state_,
            tilings: self.picture_layer_tiling_set(),
        };
        self.base_.HasMissingTiles(&delegate, c)
    }
    pub(crate) fn GetContentsResourceId(
        &self,
        resource: &mut crate::adapter::layer_tree_impl::ResourceId,
        size: &mut Size,
        uv: &mut crate::adapter::geometry::SizeF,
        c: &LayerImplContext<'_, B>,
    ) {
        let delegate = PictureTileBasedDelegate {
            state: &self.state_,
            tilings: self.picture_layer_tiling_set(),
        };
        self.base_
            .GetContentsResourceId(&delegate, resource, size, uv, c);
    }
}

// Source: cc/layers/picture_layer_impl.h:1-44,49,52,224-225,238,243-245,252,254,265-267,274-275,400-401,407,413,425,430,432-434.
// Includes, forward declarations, inheritance, access labels, friend declaration,
// namespace and guards are expressed by Rust modules, generic bounds and the
// actual base/derived composition. The deleted native copy operations are
// represented by deliberately not implementing Clone/Copy for this owner.
// GetRecycledTwinLayer (h244) is declaration-only in this fixed upstream: there
// is no .cc definition or callsite. No executable body is invented for it.

impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Actual base/state/manager and selected Set bool are call-scoped field
    // borrows. This is the sole native CreateTile algorithm body.
    // Source: cc/layers/picture_layer_impl.cc:853-872.
    // Source: cc/layers/picture_layer_impl.h:75-75.
    fn CreateTile(
        client: &mut PictureTilingClient<'_, B>,
        info: &crate::cc::tiles::tile::CreateInfo<B::RasterTransform>,
        all_tiles_done: &mut bool,
    ) -> Option<Box<crate::cc::tiles::tile::Tile<B>>> {
        client.base.SetNeedsPushProperties(
            super::layer_impl::LayerImpl::<B>::kChangedGeneralProperty,
            &mut LayerImplContext {
                tree: client.tree,
                services: client.services,
            },
        );
        *all_tiles_done = false;
        let mut flags = 0;
        if !*client.is_backdrop_filter_mask {
            flags =
                crate::cc::tiles::tile::TileRasterFlags::USE_PICTURE_ANALYSIS as i32;
        }
        if client.base.contents_opaque() {
            flags |= crate::cc::tiles::tile::TileRasterFlags::IS_OPAQUE as i32;
        }
        Some(client.environment.CreateTileWithManager(
            info,
            client.base.id(),
            client.tree.source_frame_number(),
            flags,
        ))
    }
    // Source: cc/layers/picture_layer_impl.cc:516-520.
    // Source: cc/layers/picture_layer_impl.h:125-125.
    pub(crate) fn GetPendingOrActiveTwinLayer<'a>(
        &self,
        environment: &'a dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<
            B,
        >,
    ) -> Option<PictureLayerImplReadView<'a, B>> {
        self.state_.PendingOrActiveTwinLayer(environment)
    }
    // Source: cc/layers/picture_layer_impl.cc:1719-1730.
    // Source: cc/layers/picture_layer_impl.h:272-272.
    fn CreatePictureLayerTilingSet(
        &self,
        c: &mut crate::adapter::picture_layer_impl::PictureLayerImplContext<'_, B>,
    ) -> Box<PictureLayerTilingSet<B>> {
        Self::CreatePictureLayerTilingSetForBase(&self.base_, self.client_identity_, c)
    }
}
// Source: cc/layers/picture_layer_impl.h:51,53-54,71,73-74,87,89,97,101,103,113-115,117,119,121,124,127-128,130,141,144,147,155,157-158,161-164,168-171,175,185,201,203-205,207,210,212,216-217,221,223,226,246-251,253,260,262,264,269,278,282-284,399,424,426-427,431.
// Source-only documentation, inter-definition whitespace and module closure.
// Source: cc/layers/picture_layer_impl.cc:54,58,66,71,80,101,104,110,114,119,151,210,267,279,297,306,369,373,379,458,463,477,515,528,536,653,661,687,698,705,759,766,806,817,823,827,832,837,843,852,884,898,902,906,914,919,947,952,996,1000,1023,1047,1091,1105,1111,1159,1233,1364,1446,1492,1560,1583,1592,1636,1646,1668,1676,1693,1713,1718,1731,1774,1795,1874,1878,1882,1886,1891,1932,1965,1972,2012,2029,2041,2048,2075,2086,2104,2116,2123,2127-2128.
// Source-only documentation, inter-definition whitespace and module closure.

impl<B: PictureLayerImplTypes> PictureTilingClient<'_, B> {
    fn CalculateTileSize(&mut self, bounds: Size) -> Size {
        self.state.tile_size_calculator_.CalculateTileSize(
            bounds,
            &PictureTileSizeContext {
                is_backdrop_filter_mask: self.is_backdrop_filter_mask,
                gpu_raster_max_texture_size: &self.state.gpu_raster_max_texture_size_,
                tree: self.tree,
            },
        )
    }
}
// Source: cc/layers/picture_layer_impl.cc:516-520. Native pointer selection is
// just identity plus its live tree predicate. Querying a selected receiver uses
// its existing field-split client, never a second borrow of the whole owner.
fn PendingOrActiveTwinId<B: PictureLayerImplTypes>(
    twin: Option<PictureLayerImplId>,
    q: &dyn crate::adapter::picture_layer_tiling::PictureLayerTilingQueries<B>,
) -> Option<PictureLayerImplId> {
    let twin = twin?;
    if !q.IsClientOnActiveOrPendingTree(PictureLayerTilingClientId(twin.stable as u64)) {
        return None;
    }
    Some(twin)
}
// Source: cc/layers/picture_layer_impl.cc:874-883.
fn PendingInvalidationForIdentity<B: PictureLayerImplTypes>(
    identity: PictureLayerTilingClientId,
    twin: Option<PictureLayerImplId>,
    tree: &dyn crate::adapter::picture_layer_impl::PictureLayerImplReadTree<B>,
    q: &dyn crate::adapter::picture_layer_tiling::PictureLayerTilingQueries<B>,
) -> Option<crate::adapter::picture_layer_tiling::PendingInvalidationId> {
    if tree.IsPendingTree() {
        return Some(crate::adapter::picture_layer_tiling::PendingInvalidationId(
            identity.0,
        ));
    }
    if tree.IsRecycleTree() {
        return None;
    }
    debug_assert!(tree.IsActiveTree());
    PendingOrActiveTwinId(twin, q)
        .map(|twin| crate::adapter::picture_layer_tiling::PendingInvalidationId(twin.stable as u64))
}
// Source: cc/layers/picture_layer_impl.cc:885-897.
fn TwinTilingForIdentity<B: PictureLayerImplTypes>(
    twin: Option<PictureLayerImplId>,
    tiling: &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
    q: &dyn crate::adapter::picture_layer_tiling::PictureLayerTilingQueries<B>,
    selected: Option<
        &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
    >,
) -> Option<crate::adapter::tile::PictureLayerTilingId> {
    let twin = PendingOrActiveTwinId(twin, q)?;
    let client = PictureLayerTilingClientId(twin.stable as u64);
    let twin_tiling = if let Some(selected) = selected.filter(|selected| {
        selected.client() == client && selected.contents_scale_key() == tiling.contents_scale_key()
    }) {
        selected
    } else {
        q.TilingStateForClientAndScale(client, tiling.contents_scale_key())?
    };
    if twin_tiling.raster_transform() != tiling.raster_transform() {
        return None;
    }
    Some(twin_tiling.identity())
}
/// Other-tree query client: only actual readonly remainder references. Its
/// nonconst calculator/create capabilities are intentionally not part of this
/// interface and stay on the selected call-scoped mutation context.
pub(crate) struct PictureTilingReadClient<'a, B: PictureLayerImplTypes> {
    view: PictureLayerImplReadView<'a, B>,
}
impl<'a, B: PictureLayerImplTypes> PictureTilingReadClient<'a, B> {
    pub(crate) fn new(view: PictureLayerImplReadView<'a, B>) -> Self {
        Self { view }
    }
}
impl<B: PictureLayerImplTypes>
    crate::cc::tiles::picture_layer_tiling::PictureLayerTilingClient<B>
    for PictureTilingReadClient<'_, B>
{
    fn GetPendingInvalidation(
        &self,
        q: &dyn crate::adapter::picture_layer_tiling::PictureLayerTilingQueries<B>,
    ) -> Option<crate::adapter::picture_layer_tiling::PendingInvalidationId> {
        PendingInvalidationForIdentity(
            PictureLayerTilingClientId(self.view.identity.stable as u64),
            self.view.state.twin_layer_,
            self.view.tree,
            q,
        )
    }
    fn GetPendingOrActiveTwinTiling(
        &self,
        tiling: &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
        q: &dyn crate::adapter::picture_layer_tiling::PictureLayerTilingQueries<B>,
        selected: Option<
            &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
        >,
    ) -> Option<crate::adapter::tile::PictureLayerTilingId> {
        TwinTilingForIdentity(self.view.state.twin_layer_, tiling, q, selected)
    }
    fn HasValidTilePriorities(&self) -> bool {
        HasValidTilePrioritiesForTree(
            self.view.tree,
            || self.view.base.base().contributes_to_drawn_render_surface(),
            || self.view.base.base().raster_even_if_not_drawn(),
        )
    }
    fn RequiresHighResToDraw(&self) -> bool {
        self.view.tree.RequiresHighResToDraw()
    }
    fn GetPaintWorkletRecords(&self) -> &B::PaintWorkletRecordMap {
        &self.view.state.paint_worklet_records_
    }
    fn GetDiscardableImagesInRect(
        &self,
        rect: Rect,
    ) -> Vec<&<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::DrawImage> {
        use crate::adapter::picture_layer_impl::PictureDiscardableImageMap;
        self.view
            .state
            .discardable_image_map_
            .as_ref()
            .unwrap()
            .GetDiscardableImagesInRect(&rect)
    }
    fn GetRasterInducingScrollOffsets(&self) -> crate::adapter::prioritized_tile::ScrollOffsetMap {
        RasterInducingScrollOffsetsForSource::<B>(
            self.view.state.raster_source_.as_deref(),
            self.view.tree,
        )
    }
    fn global_tile_state(
        &self,
    ) -> &crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority {
        self.view.tree.global_tile_state()
    }
}

// Source: cc/layers/picture_layer_impl.cc:1883-1890. Same lazy source reads.
fn IsOnActiveOrPendingTreeForTree(
    tree: &dyn crate::adapter::picture_layer_impl::PictureLayerReadContext,
) -> bool {
    !tree.IsRecycleTree()
}
fn HasValidTilePrioritiesForTree(
    tree: &dyn crate::adapter::picture_layer_impl::PictureLayerReadContext,
    contributes: impl FnOnce() -> bool,
    raster: impl FnOnce() -> bool,
) -> bool {
    IsOnActiveOrPendingTreeForTree(tree) && (contributes() || raster())
}
impl<B: PictureLayerImplTypes> PictureTilingReadClient<'_, B> {
    pub(crate) fn IsOnActiveOrPendingTree(&self) -> bool {
        IsOnActiveOrPendingTreeForTree(self.view.tree)
    }
    pub(crate) fn invalidation(&self) -> &Region<<B as LayerTreeTypes>::RegionBackend> {
        &self.view.state.invalidation_
    }
}

impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    // Inherited source: cc/layers/tile_based_layer_impl.h:68.
    pub(crate) fn GetDamageRectForDispatch(&self) -> Rect {
        self.base_.GetDamageRect()
    }
    // Inherited source: cc/layers/tile_based_layer_impl.h:74-77.
    // GetMaximumContentsScaleForUseInAppendQuads is the actual Picture override.
    pub(crate) fn GetEnclosingVisibleRectForPropertyTrees(
        &self,
        trees: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>,
    ) -> Rect {
        self.base_
            .base()
            .GetScaledEnclosingVisibleRectInTargetSpaceForPropertyTrees(
                self.MaximumTilingContentsScale(),
                trees,
            )
    }
}

// Source: cc/layers/picture_layer_impl.cc:654-660. Borrow-only current-owner
// action used by the canonical scoped updater's source Drop regeneration loop.
struct RasterSourceTestingUpdate<'a, B: PictureLayerImplTypes> {
    picture: &'a mut PictureLayerImpl<B>,
    source: Option<Arc<B::RasterSource>>,
    invalidation: &'a Region<<B as LayerTreeTypes>::RegionBackend>,
    services: &'a mut dyn crate::adapter::picture_layer_impl::PictureLayerImplServices<B>,
    tiling: &'a mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTilingEnvironment<B>,
}
impl<B: PictureLayerImplTypes>
    crate::adapter::picture_layer_impl::DiscardableImageMapUpdateAction<B>
    for RasterSourceTestingUpdate<'_, B>
{
    fn Run(
        &mut self,
        tree: &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTree<B>,
        updates: &mut Vec<i32>,
    ) {
        self.picture.UpdateRasterSourceInternal(
            self.source
                .take()
                .expect("native current-owner update runs once"),
            self.invalidation.clone(),
            RasterSourceUpdate::Commit { updates },
            &mut crate::adapter::picture_layer_impl::PictureLayerImplContext {
                tree,
                services: self.services,
                tiling: self.tiling,
            },
        );
    }
    fn Regenerate(
        &mut self,
        id: i32,
        tree: &mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTree<B>,
    ) {
        assert_eq!(
            id,
            self.picture.base().id(),
            "this source testing scope updates its actual current PictureLayerImpl"
        );
        self.picture.RegenerateDiscardableImageMap(
            &mut crate::adapter::picture_layer_impl::PictureLayerImplContext {
                tree,
                services: self.services,
                tiling: self.tiling,
            },
        );
    }
}

// Source: cc/layers/picture_layer_impl.cc:783-805. Rust borrow split: normal
// notification and selected Tile destruction share this single source tail.
fn NotifyTileStateChangedAfterDamage<B: PictureLayerImplTypes>(
    updated_tiles: &mut TileUpdateSet,
    should_batch: bool,
    base: &mut super::layer_impl::LayerImplTileClient<'_>,
    tile: &crate::cc::tiles::tile::Tile<B>,
    update_damage: bool,
    tree: &mut dyn crate::adapter::layer_impl::LayerImplTree<B>,
) {
    NotifyTileStateChangedAfterDamageWithPush::<B>(
        updated_tiles,
        should_batch,
        base,
        tile,
        update_damage,
        &mut PictureTilePushContext { tree },
    );
}
// Source: cc/layers/picture_layer_impl.cc:783-805. A current tile callback
// only reads/pushes this layer's native property flags; it never uses manager,
// raster services, mutable property trees or the rest of PictureContext.
fn NotifyTileStateChangedAfterDamageWithPush<B: PictureLayerImplTypes>(
    updated_tiles: &mut TileUpdateSet,
    should_batch: bool,
    base: &mut super::layer_impl::LayerImplTileClient<'_>,
    tile: &crate::cc::tiles::tile::Tile<B>,
    update_damage: bool,
    tree: &mut dyn crate::adapter::layer_impl::LayerImplPushContext<Types = B>,
) {
    if tree.TreesInVizInClientProcess() {
        if should_batch {
            let update_in_viz = update_damage && tree.IsActiveTree();
            let entry = updated_tiles
                .entry(ScaleKey::new(tile.contents_scale_key()))
                .or_default()
                .entry(TileIndex::new(tile.tiling_i_index(), tile.tiling_j_index()))
                .or_insert(update_in_viz);
            if update_in_viz {
                *entry = true;
            }
        }
        base.SetNeedsPushProperties::<B>(super::layer_impl::LayerImpl::<B>::kChangedTile, tree);
    }
}
struct PictureTilePushContext<'a, B: PictureLayerImplTypes> {
    tree: &'a mut dyn crate::adapter::layer_impl::LayerImplTree<B>,
}
impl<B: PictureLayerImplTypes> crate::adapter::layer_impl::LayerImplPushContext
    for PictureTilePushContext<'_, B>
{
    type Types = B;
    fn id(&self) -> LayerTreeId {
        self.tree.id()
    }
    fn IsActiveTree(&self) -> bool {
        self.tree.IsActiveTree()
    }
    fn TreesInVizInClientProcess(&self) -> bool {
        self.tree.TreesInVizInClientProcess()
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.tree.IsCCSlimmingEnabled()
    }
    fn AddLayerShouldPushProperties(&mut self, id: i32) {
        self.tree.AddLayerShouldPushProperties(id)
    }
    fn set_needs_update_draw_properties(&mut self) {
        self.tree.set_needs_update_draw_properties()
    }
}

/// Source: cc/layers/picture_layer_impl.cc:783-805. The destructor enters this
/// stack continuation while the selected Set/Tiling is executing; each reference
/// is a disjoint field of the very same Picture allocation, never copied state.
struct PictureReleaseNotification<'a, 'b, B: PictureLayerImplTypes> {
    base: &'a mut super::layer_impl::LayerImplTileClient<'b>,
    updated_tiles: &'a mut TileUpdateSet,
    should_batch: &'a bool,
    tree: &'a mut dyn crate::adapter::picture_layer_impl::PictureLayerImplTree<B>,
}
impl<B: PictureLayerImplTypes> crate::adapter::picture_layer_impl::PictureReleasedTileClient<B>
    for PictureReleaseNotification<'_, '_, B>
{
    fn NotifyReleasedTile(&mut self, tile: &crate::cc::tiles::tile::Tile<B>) {
        NotifyTileStateChangedAfterDamage::<B>(
            self.updated_tiles,
            *self.should_batch,
            self.base,
            tile,
            false,
            self.tree,
        )
    }
    fn layer(&self) -> &dyn crate::adapter::layer_tree_host_impl::PictureLayerDisplayRead {
        self.base
    }
    fn should_batch_updated_tiles(&self) -> bool {
        *self.should_batch
    }
}

// Native queue layer pointers are const range borrows. This projection contains
// only references into the same PictureLayer allocation, including its Set.
pub(crate) struct PictureQueueLayerRead<'a, B: PictureLayerImplTypes> {
    picture: &'a PictureLayerImpl<B>,
    client: PictureTilingReadClient<'a, B>,
}
impl<'a, B: PictureLayerImplTypes> PictureQueueLayerRead<'a, B> {
    pub(crate) fn new(
        picture: &'a PictureLayerImpl<B>,
        tree: &'a dyn crate::adapter::picture_layer_impl::PictureLayerImplReadTree<B>,
    ) -> Self {
        Self {
            picture,
            client: PictureTilingReadClient::new(picture.ReadView(tree)),
        }
    }
}
impl<B: PictureLayerImplTypes>
    crate::cc::tiles::raster_tile_priority_queue::RasterQueueLayer<B>
    for PictureQueueLayerRead<'_, B>
{
    fn HasValidTilePriorities(&self) -> bool {
        <PictureTilingReadClient<'_,B> as crate::cc::tiles::picture_layer_tiling::PictureLayerTilingClient<B>>::HasValidTilePriorities(&self.client)
    }
    fn contributes_to_drawn_render_surface(&self) -> bool {
        self.picture
            .base_
            .base()
            .contributes_to_drawn_render_surface()
    }
    fn picture_layer_tiling_set(&self) -> &PictureLayerTilingSet<B> {
        self.picture.picture_layer_tiling_set()
    }
}

/// Actual fields used by cc/layers/picture_layer_impl.cc:767-805. In particular
/// Set's all_tiles_done flag is distinct from its tiling vector, so a tile may
/// stay borrowed while its owner's damage/property flags are updated.
pub(crate) struct PictureTileNotificationParts<'a, B: PictureLayerImplTypes> {
    base: super::layer_impl::LayerImplTileClient<'a>,
    damage: &'a mut Rect,
    updated_tiles: &'a mut TileUpdateSet,
    should_batch: &'a bool,
    set: Option<(&'a [Box<PictureLayerTiling<B>>], &'a mut bool)>,
}
impl<B: PictureLayerImplTypes> PictureTileNotificationParts<'_, B> {
    pub(crate) fn layer(&self)->&dyn crate::adapter::layer_tree_host_impl::PictureLayerDisplayRead {&self.base}
    pub(crate) fn should_batch_updated_tiles(&self)->bool {*self.should_batch}
    pub(crate) fn WithTileTilingState<Output>(
        &mut self,id:crate::adapter::tile::PictureLayerTilingId,
        body:impl FnOnce(&crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,&mut Self)->Output,
    )->Option<Output> {
        let tilings=self.set.as_ref()?.0;
        let tiling=tilings.iter().find(|tiling|tiling.identity()==id)?;
        Some(body(tiling,self))
    }
    pub(crate) fn WithResidentTile<Output>(
        &mut self,id:crate::adapter::tile::TileId,
        body:impl FnOnce(
            &crate::cc::tiles::tile::Tile<B>,
            &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
            &mut Self,
        )->Output,
    )->Output {
        let tilings=self.set.as_ref().expect("source resident tile Set").0;
        let tiling=tilings.iter().find(|tiling|tiling.identity()==id.tiling).expect("source resident tiling");
        body(tiling.TileById(id),tiling,self)
    }
    // Source: cc/layers/picture_layer_impl.cc:767-805.
    pub(crate) fn Notify(
        &mut self,
        tile: &crate::cc::tiles::tile::Tile<B>,
        update_damage: bool,
        tree: &mut dyn crate::adapter::layer_impl::LayerImplPushContext<Types = B>,
    ) {
        NotifyTileStateChangedWithParts::<B>(
            &mut self.base,
            self.damage,
            self.updated_tiles,
            self.should_batch,
            self.set
                .as_mut()
                .map(|(tilings, done)| (*tilings, &mut **done)),
            tile,
            update_damage,
            tree,
        );
    }
    pub(crate) fn NotifyResident(
        &mut self,
        id: crate::adapter::tile::TileId,
        update_damage: bool,
        tree: &mut dyn crate::adapter::layer_impl::LayerImplPushContext<Types = B>,
    ) {
        let (tilings, done) = self.set.as_mut().expect("source resident tile Set");
        let tiling = tilings
            .iter()
            .find(|tiling| tiling.identity() == id.tiling)
            .expect("source resident tile tiling");
        let tile = tiling.TileById(id);
        NotifyTileStateChangedWithParts::<B>(
            &mut self.base,
            self.damage,
            self.updated_tiles,
            self.should_batch,
            Some((*tilings, &mut **done)),
            tile,
            update_damage,
            tree,
        );
    }
}
// Source: cc/layers/picture_layer_impl.cc:767-805. Single canonical body,
// including damage, NeedsRaster short-circuit and tiling lookup ordering.
fn NotifyTileStateChangedWithParts<B: PictureLayerImplTypes>(
    base: &mut super::layer_impl::LayerImplTileClient<'_>,
    damage: &mut Rect,
    updated_tiles: &mut TileUpdateSet,
    should_batch: &bool,
    mut set: Option<(&[Box<PictureLayerTiling<B>>], &mut bool)>,
    tile: &crate::cc::tiles::tile::Tile<B>,
    update_damage: bool,
    tree: &mut dyn crate::adapter::layer_impl::LayerImplPushContext<Types = B>,
) {
    if update_damage {
        if tree.IsActiveTree() {
            damage.Union(*tile.enclosing_layer_rect());
        }
        if tile.draw_info().NeedsRaster() {
            let (tilings, done) = set.as_mut().expect("source picture_layer_tiling_set()");
            if let Some(tiling) =
                PictureLayerTilingSet::FindTilingWithScaleKeyIn(tilings, tile.contents_scale_key())
            {
                tiling.set_all_tiles_done(false);
                **done = false;
            }
        }
    }
    NotifyTileStateChangedAfterDamageWithPush::<B>(
        updated_tiles,
        *should_batch,
        base,
        tile,
        update_damage,
        tree,
    );
}

/// Disjoint fields read/written by the native notification tail during an
/// already-selected Set/Tile traversal. This is a borrowed view of the same
/// allocation; it cannot find or retain an owner, tiling or resource.
pub(crate) struct PictureResourceTileNotificationParts<'a> {
    twin_layer: &'a Option<PictureLayerImplId>,
    base: super::layer_impl::LayerImplTileClient<'a>,
    updated_tiles: &'a mut TileUpdateSet,
    should_batch: &'a bool,
}
impl PictureResourceTileNotificationParts<'_> {
    // Source: cc/layers/picture_layer_impl.cc:516-520,903-918;
    // cc/tiles/picture_layer_tiling.h:371-398. Only the same native twin
    // pointer field is shared with this selected resource traversal.
    pub(crate) fn IsTileOccluded<B:PictureLayerImplTypes>(
        &self, tile:&crate::cc::tiles::tile::Tile<B>,
        state:&crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
        queries:&dyn crate::adapter::picture_layer_tiling::PictureLayerTilingQueries<B>,
    )->bool {
        state.IsTileOccludedWithResolvedTwin(tile, || {
            let twin=TwinTilingForIdentity(*self.twin_layer,state,queries,None)?;
            Some(queries.TilingById(twin))
        })
    }
    // Source: cc/layers/picture_layer_impl.cc:783-805. Occluded-tile release
    // calls NotifyTileStateChanged(update_damage=false), so this same canonical
    // tail needs no mutable Set or tile-map borrow.
    pub(crate) fn NotifyReleasedTile<B: PictureLayerImplTypes>(
        &mut self,
        tile: &crate::cc::tiles::tile::Tile<B>,
        tree: &mut dyn crate::adapter::layer_impl::LayerImplPushContext<Types=B>,
    ) {
        NotifyTileStateChangedAfterDamageWithPush::<B>(
            self.updated_tiles, *self.should_batch, &mut self.base, tile, false, tree,
        );
    }
    pub(crate) fn layer(&self)->&dyn crate::adapter::layer_tree_host_impl::PictureLayerDisplayRead {
        &self.base
    }
    pub(crate) fn should_batch_updated_tiles(&self)->bool { *self.should_batch }
}
impl<B: PictureLayerImplTypes> PictureLayerImpl<B> {
    /// Source: cc/tiles/tiles_with_resource_iterator.cc:84-103 and
    /// cc/layers/picture_layer_impl.cc:783-805. Borrow-only split, not a second
    /// iterator or notification algorithm. The caller keeps the selected Tile
    /// live while source notification updates its owner's disjoint flags.
    pub(crate) fn SplitResourceTraversal(&mut self)->(
        &mut PictureLayerTilingSet<B>, PictureResourceTileNotificationParts<'_>
    ) {
        let (base, _damage) = self.base_.SplitTileNotification();
        (self.tilings_.as_deref_mut().expect("source live tilings"),
         PictureResourceTileNotificationParts {
             base, twin_layer:&self.state_.twin_layer_, updated_tiles:&mut self.state_.updated_tiles_,
             should_batch:&self.state_.should_batch_updated_tiles_,
         })
    }
}
