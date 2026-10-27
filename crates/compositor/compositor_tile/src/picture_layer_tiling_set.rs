// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/picture_layer_tiling_set.h:5-28
// Source: cc/tiles/picture_layer_tiling_set.cc:5-24
use compositor_tree::geometry::{Rect, RectF, ToEnclosingRect};
use std::ops::Deref;

// Source: cc/tiles/picture_layer_tiling_set.h:31-41
#[derive(Clone, Copy)]
pub enum TilingRangeType {
    HIGHER_THAN_HIGH_RES,
    HIGH_RES,
    LOWER_THAN_HIGH_RES,
}
#[derive(Clone, Copy)]
pub struct TilingRange {
    pub start: usize,
    pub end: usize,
}
impl TilingRange {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}
// Source: cc/tiles/picture_layer_tiling_set.cc:34-43
fn LargerRatio(float1: f32, float2: f32) -> f32 {
    debug_assert!(float1 > 0.0);
    debug_assert!(float2 > 0.0);
    if float1 > float2 {
        float1 / float2
    } else {
        float2 / float1
    }
}
const kSoonBorderDistanceViewportPercentage: f32 = 0.15;
const kMaxSoonBorderDistanceInScreenPixels: f32 = 312.0;

// Source: cc/tiles/picture_layer_tiling_set.h:152-160
#[derive(Clone, Copy)]
struct FrameVisibleRect {
    visible_rect_in_layer_space: Rect,
    frame_time_in_seconds: f64,
}
impl FrameVisibleRect {
    fn new(rect: Rect, time_in_seconds: f64) -> Self {
        Self {
            visible_rect_in_layer_space: rect,
            frame_time_in_seconds: time_in_seconds,
        }
    }
}
// Source: cc/tiles/picture_layer_tiling_set.h:162-182
#[derive(Default)]
struct StateSinceLastTilePriorityUpdate {
    invalidated: bool,
    added_tilings: bool,
    tiling_needs_update: bool,
}
struct AutoClear<'a> {
    state_to_clear_: &'a mut StateSinceLastTilePriorityUpdate,
}
impl<'a> AutoClear<'a> {
    fn new(state: &'a mut StateSinceLastTilePriorityUpdate) -> Self {
        Self {
            state_to_clear_: state,
        }
    }
}
impl Deref for AutoClear<'_> {
    type Target = StateSinceLastTilePriorityUpdate;
    fn deref(&self) -> &Self::Target {
        self.state_to_clear_
    }
}
impl Drop for AutoClear<'_> {
    fn drop(&mut self) {
        *self.state_to_clear_ = StateSinceLastTilePriorityUpdate::default();
    }
}

// Source: cc/tiles/picture_layer_tiling_set.h:199-200
// Source: cc/tiles/picture_layer_tiling_set.cc:366-397
// Rust borrow split: receives the actual resident history/state fields rather
// than borrowing the entire owner while AutoClear holds the state field.
fn TilingsNeedUpdate(
    num_tilings: usize,
    visible_rect_history: &[FrameVisibleRect],
    state_since_last_tile_priority_update: &StateSinceLastTilePriorityUpdate,
    visible_rect_in_layer_space: &Rect,
    current_frame_time_in_seconds: f64,
) -> bool {
    if num_tilings == 0 {
        return false;
    }
    if visible_rect_history.is_empty() {
        return true;
    }
    if state_since_last_tile_priority_update.added_tilings {
        return true;
    }
    let last_frame = visible_rect_history.last().unwrap();
    if current_frame_time_in_seconds != last_frame.frame_time_in_seconds {
        return true;
    }
    if *visible_rect_in_layer_space != last_frame.visible_rect_in_layer_space {
        return true;
    }
    if state_since_last_tile_priority_update.tiling_needs_update {
        return true;
    }
    false
}
// Borrow-only facade over the source's resident history and interest rectangles.
// Immutable configuration is borrowed too. It owns no duplicate state.
struct PriorityGeometry<'a> {
    tiling_interest_area_padding_: &'a i32,
    skewport_target_time_in_seconds_: &'a f32,
    skewport_extrapolation_limit_in_screen_pixels_: &'a i32,
    visible_rect_history_: &'a mut Vec<FrameVisibleRect>,
    visible_rect_in_layer_space_: &'a mut Rect,
    skewport_rect_in_layer_space_: &'a mut Rect,
    soon_border_rect_in_layer_space_: &'a mut Rect,
    eventually_rect_in_layer_space_: &'a mut Rect,
}
impl PriorityGeometry<'_> {
    // Source: cc/tiles/picture_layer_tiling_set.h:201-203
    // Source: cc/tiles/picture_layer_tiling_set.cc:399-449
    fn ComputeSkewport(
        &self,
        visible_rect_in_layer_space: Rect,
        current_frame_time_in_seconds: f64,
        ideal_contents_scale: f32,
    ) -> Rect {
        let mut skewport = visible_rect_in_layer_space;
        if skewport.IsEmpty() || self.visible_rect_history_.is_empty() {
            return skewport;
        }
        let historical_frame = self.visible_rect_history_.first().unwrap();
        let time_delta = current_frame_time_in_seconds - historical_frame.frame_time_in_seconds;
        if time_delta == 0.0 {
            return skewport;
        }
        let extrapolation_multiplier =
            f64::from(*self.skewport_target_time_in_seconds_) / time_delta;
        let old_x = historical_frame.visible_rect_in_layer_space.x;
        let old_y = historical_frame.visible_rect_in_layer_space.y;
        let old_right = historical_frame.visible_rect_in_layer_space.right();
        let old_bottom = historical_frame.visible_rect_in_layer_space.bottom();
        let new_x = visible_rect_in_layer_space.x;
        let new_y = visible_rect_in_layer_space.y;
        let new_right = visible_rect_in_layer_space.right();
        let new_bottom = visible_rect_in_layer_space.bottom();
        let inset_x = (f64::from(new_x - old_x) * extrapolation_multiplier) as i32;
        let inset_y = (f64::from(new_y - old_y) * extrapolation_multiplier) as i32;
        let inset_right = (f64::from(old_right - new_right) * extrapolation_multiplier) as i32;
        let inset_bottom = (f64::from(old_bottom - new_bottom) * extrapolation_multiplier) as i32;
        let skewport_extrapolation_limit_in_layer_pixels =
            (*self.skewport_extrapolation_limit_in_screen_pixels_ as f32 / ideal_contents_scale)
                as i32;
        let mut max_skewport = skewport;
        let inset = -skewport_extrapolation_limit_in_layer_pixels;
        max_skewport.Inset(inset, inset, inset, inset);
        skewport.Inset(inset_y, inset_x, inset_bottom, inset_right);
        skewport.Union(visible_rect_in_layer_space);
        skewport.Intersect(max_skewport);
        skewport.Union(visible_rect_in_layer_space);
        skewport.Intersect(*self.eventually_rect_in_layer_space_);
        skewport
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:204-205
    // Source: cc/tiles/picture_layer_tiling_set.cc:451-463
    fn ComputeSoonBorderRect(&self, visible_rect: Rect, ideal_contents_scale: f32) -> Rect {
        let max_dimension = visible_rect.width.max(visible_rect.height);
        // Native std::min<int> converts each floating operand to int first.
        let distance = ((kMaxSoonBorderDistanceInScreenPixels * ideal_contents_scale) as i32)
            .min((max_dimension as f32 * kSoonBorderDistanceViewportPercentage) as i32);
        let mut soon_border_rect = visible_rect;
        soon_border_rect.Inset(-distance, -distance, -distance, -distance);
        soon_border_rect.Intersect(*self.eventually_rect_in_layer_space_);
        soon_border_rect
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:206-208
    // Source: cc/tiles/picture_layer_tiling_set.cc:465-518
    // The lazy getter keeps RasterSource::recorded_bounds at its native branch
    // and call position; it borrows the actual immutable handle, not its tree.
    fn UpdatePriorityRects(
        &mut self,
        visible_rect_in_layer_space: Rect,
        current_frame_time_in_seconds: f64,
        ideal_contents_scale: f32,
        recorded_bounds: impl FnOnce() -> Rect,
    ) {
        let mut has_visible_rects = false;
        if !visible_rect_in_layer_space.IsEmpty() {
            let mut eventually_rectf = RectF::from(visible_rect_in_layer_space);
            let inset = -*self.tiling_interest_area_padding_ as f32 / ideal_contents_scale;
            eventually_rectf.Inset(inset, inset, inset, inset);
            if eventually_rectf.Intersects(RectF::from(recorded_bounds())) {
                *self.visible_rect_in_layer_space_ = visible_rect_in_layer_space;
                *self.eventually_rect_in_layer_space_ = ToEnclosingRect(eventually_rectf);
                has_visible_rects = true;
            }
        }
        if !has_visible_rects {
            *self.visible_rect_in_layer_space_ = Rect::default();
            *self.eventually_rect_in_layer_space_ = Rect::default();
            *self.skewport_rect_in_layer_space_ = Rect::default();
            *self.soon_border_rect_in_layer_space_ = Rect::default();
            self.visible_rect_history_.clear();
            return;
        }
        *self.skewport_rect_in_layer_space_ = self.ComputeSkewport(
            *self.visible_rect_in_layer_space_,
            current_frame_time_in_seconds,
            ideal_contents_scale,
        );
        debug_assert!(self
            .skewport_rect_in_layer_space_
            .Contains(*self.visible_rect_in_layer_space_));
        debug_assert!(self
            .eventually_rect_in_layer_space_
            .Contains(*self.skewport_rect_in_layer_space_));
        *self.soon_border_rect_in_layer_space_ =
            self.ComputeSoonBorderRect(*self.visible_rect_in_layer_space_, ideal_contents_scale);
        debug_assert!(self
            .soon_border_rect_in_layer_space_
            .Contains(*self.visible_rect_in_layer_space_));
        debug_assert!(self
            .eventually_rect_in_layer_space_
            .Contains(*self.soon_border_rect_in_layer_space_));
        let frame_visible_rect = FrameVisibleRect::new(
            *self.visible_rect_in_layer_space_,
            current_frame_time_in_seconds,
        );
        if self.visible_rect_history_.len() < 2 {
            self.visible_rect_history_
                .reserve(2 - self.visible_rect_history_.len());
            self.visible_rect_history_.push(frame_visible_rect);
        } else {
            debug_assert_eq!(self.visible_rect_history_.len(), 2);
            // Native initializer-list assignment reuses the two-element buffer.
            self.visible_rect_history_[0] = self.visible_rect_history_[1];
            self.visible_rect_history_[1] = frame_visible_rect;
        }
    }
}

use crate::{
    picture_layer_tiling::PictureLayerTiling,
    picture_layer_tiling_types::{
        PendingInvalidationId, PictureLayerTileDestroyer, PictureLayerTilingClientId,
        PictureLayerTilingContext, PictureLayerTilingQueries, PictureLayerTilingTypes,
        TilingRasterSource,
    },
    prioritized_tile::PrioritizedTile,
    tile::{CreateInfo, Tile},
    tile_priority::TileResolution,
    tile_types::{PictureLayerTilingId, TileId, TileManagerId},
    tiling_set_coverage_iterator::TilingSetCoverageIterator,
    PictureLayerTilingSetId, WhichTree,
};
use compositor_draw::occlusion::Occlusion;
#[cfg(debug_assertions)]
use compositor_tree::geometry::AxisTransform2d;
use compositor_tree::{property_types::PropertyTrace, region::Region};
use std::sync::Arc;

// Same-package identity allocation, equivalent to obtaining a new allocation's
// non-owning pointer identity. It implements no tiling/client/platform policy.
pub trait PictureLayerTilingSetContext<B: PictureLayerTilingTypes>:
    PictureLayerTilingQueries<B> + PictureLayerTileDestroyer<B>
{
    fn CalculateTileSizeForClient(
        &mut self,
        id: PictureLayerTilingClientId,
        bounds: compositor_tree::geometry::Size,
    ) -> compositor_tree::geometry::Size;
    fn CreateTileForClient(
        &mut self,
        client: PictureLayerTilingClientId,
        info: &CreateInfo<B::RasterTransform>,
        all_tiles_done: &mut bool,
    ) -> Option<Box<Tile<B>>>;
    fn IsCCSlimmingEnabled(&self) -> bool;
    fn AllocateTilingId(&mut self) -> PictureLayerTilingId;
}
// Rust borrow mapping for the synchronous source client->CreateTile reentry:
// PictureLayerImpl.cc:855-878 modifies this exact Set bool while the selected
// child tiling is executing. The facade lends that resident field independently
// of tilings_; it never retains/re-enters/locks/copies the owning Set.
struct SetTilingContext<'a, B: PictureLayerTilingTypes> {
    external: &'a mut dyn PictureLayerTilingSetContext<B>,
    all_tiles_done: &'a mut bool,
}
impl<B: PictureLayerTilingTypes> PictureLayerTilingQueries<B> for SetTilingContext<'_, B> {
    fn WithClient(
        &self,
        id: PictureLayerTilingClientId,
        callback: &mut dyn FnMut(&dyn crate::picture_layer_tiling::PictureLayerTilingClient<B>),
    ) {
        self.external.WithClient(id, callback)
    }
    fn PaintWorkletRecordsForClient(
        &self,
        id: PictureLayerTilingClientId,
    ) -> &B::PaintWorkletRecordMap {
        self.external.PaintWorkletRecordsForClient(id)
    }
    fn TilingById(&self, id: PictureLayerTilingId) -> &PictureLayerTiling<B> {
        self.external.TilingById(id)
    }
    fn IsClientOnActiveOrPendingTree(&self, id: PictureLayerTilingClientId) -> bool {
        self.external.IsClientOnActiveOrPendingTree(id)
    }
    fn TilingStateForClientAndScale(
        &self,
        id: PictureLayerTilingClientId,
        scale: f32,
    ) -> Option<&crate::picture_layer_tiling::PictureLayerTilingState<B>> {
        self.external.TilingStateForClientAndScale(id, scale)
    }
    fn PendingInvalidationById(&self, id: PendingInvalidationId) -> &Region<B::RegionBackend> {
        self.external.PendingInvalidationById(id)
    }
}
impl<B: PictureLayerTilingTypes> PictureLayerTileDestroyer<B> for SetTilingContext<'_, B> {
    fn RebindTileWithManager(&mut self, old: TileId, new: TileId) {
        self.external.RebindTileWithManager(old, new)
    }
    fn DestroyTileWithManager(
        &mut self,
        id: TileManagerId,
        tile: &mut Tile<B>,
        tiling: &crate::picture_layer_tiling::PictureLayerTilingState<B>,
    ) {
        self.external.DestroyTileWithManager(id, tile, tiling);
    }
}
impl<B: PictureLayerTilingTypes> PictureLayerTilingContext<B> for SetTilingContext<'_, B> {
    fn CalculateTileSizeForClient(
        &mut self,
        id: PictureLayerTilingClientId,
        bounds: compositor_tree::geometry::Size,
    ) -> compositor_tree::geometry::Size {
        self.external.CalculateTileSizeForClient(id, bounds)
    }
    fn CreateTileForClient(
        &mut self,
        id: PictureLayerTilingClientId,
        info: &CreateInfo<B::RasterTransform>,
    ) -> Option<Box<Tile<B>>> {
        self.external
            .CreateTileForClient(id, info, self.all_tiles_done)
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.external.IsCCSlimmingEnabled()
    }
}
// Source: cc/tiles/picture_layer_tiling_set.cc:26-32
struct LargestToSmallestScaleFunctor;
impl LargestToSmallestScaleFunctor {
    fn compare<B: PictureLayerTilingTypes>(
        left: &PictureLayerTiling<B>,
        right: &PictureLayerTiling<B>,
    ) -> bool {
        left.contents_scale_key() > right.contents_scale_key()
    }
}
// Source: cc/tiles/picture_layer_tiling_set.h:29-30,210-233
// The canonical set owns every tiling Box. IDs only name these residents; no
// external service owns the collection or copies its mutable metadata.
// Reverse member declaration preserves native member destruction order.
pub struct PictureLayerTilingSet<B: PictureLayerTilingTypes> {
    eventually_rect_in_layer_space_: Rect,
    soon_border_rect_in_layer_space_: Rect,
    skewport_rect_in_layer_space_: Rect,
    visible_rect_in_layer_space_: Rect,
    raster_source_: Option<Arc<B::RasterSource>>,
    state_since_last_tile_priority_update_: StateSinceLastTilePriorityUpdate,
    visible_rect_history_: Vec<FrameVisibleRect>,
    max_preraster_distance_: f32,
    client_: PictureLayerTilingClientId,
    tree_: WhichTree,
    skewport_extrapolation_limit_in_screen_pixels_: i32,
    skewport_target_time_in_seconds_: f32,
    tiling_interest_area_padding_: i32,
    all_tiles_done_: bool,
    tilings_: Vec<Box<PictureLayerTiling<B>>>,
    identity_: PictureLayerTilingSetId,
}
impl<B: PictureLayerTilingTypes> PictureLayerTilingSet<B> {
    // Source: cc/tiles/picture_layer_tiling_set.h:43-54
    // Source: cc/tiles/picture_layer_tiling_set.cc:45-57
    pub fn Create(
        identity: PictureLayerTilingSetId,
        tree: WhichTree,
        client: PictureLayerTilingClientId,
        tiling_interest_area_padding: i32,
        skewport_target_time_in_seconds: f32,
        skewport_extrapolation_limit_in_screen_pixels: i32,
        max_preraster_distance: f32,
    ) -> Box<Self> {
        Box::new(Self::new(
            identity,
            tree,
            client,
            tiling_interest_area_padding,
            skewport_target_time_in_seconds,
            skewport_extrapolation_limit_in_screen_pixels,
            max_preraster_distance,
        ))
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:184-190,210-230
    // Source: cc/tiles/picture_layer_tiling_set.cc:59-72
    fn new(
        identity: PictureLayerTilingSetId,
        tree: WhichTree,
        client: PictureLayerTilingClientId,
        tiling_interest_area_padding: i32,
        skewport_target_time_in_seconds: f32,
        skewport_extrapolation_limit_in_screen_pixels: i32,
        max_preraster_distance: f32,
    ) -> Self {
        Self {
            eventually_rect_in_layer_space_: Rect::default(),
            soon_border_rect_in_layer_space_: Rect::default(),
            skewport_rect_in_layer_space_: Rect::default(),
            visible_rect_in_layer_space_: Rect::default(),
            raster_source_: None,
            state_since_last_tile_priority_update_: StateSinceLastTilePriorityUpdate::default(),
            visible_rect_history_: Vec::new(),
            max_preraster_distance_: max_preraster_distance,
            client_: client,
            tree_: tree,
            skewport_extrapolation_limit_in_screen_pixels_:
                skewport_extrapolation_limit_in_screen_pixels,
            skewport_target_time_in_seconds_: skewport_target_time_in_seconds,
            tiling_interest_area_padding_: tiling_interest_area_padding,
            all_tiles_done_: true,
            tilings_: Vec::new(),
            identity_: identity,
        }
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:52
    // Source: cc/tiles/picture_layer_tiling_set.cc:74
    // Native member destruction needs a live canonical TileManager. Explicit
    // context is its Rust borrow mapping; Drop checks this release happened.
    pub fn Destroy(&mut self, context: &mut dyn PictureLayerTileDestroyer<B>) {
        self.raster_source_ = None;
        drop(std::mem::take(&mut self.visible_rect_history_));
        Self::DestroyTilings(&mut self.tilings_, context);
    }
    // Source: cc/tiles/picture_layer_tiling_set.cc:74,340-343
    // Borrow mapping of native unique_ptr member destruction/clear.
    fn DestroyTilings(
        tilings: &mut Vec<Box<PictureLayerTiling<B>>>,
        context: &mut dyn PictureLayerTileDestroyer<B>,
    ) {
        // The native vector destructor/clear releases unique_ptrs backwards.
        while let Some(mut tiling) = tilings.pop() {
            tiling.Destroy(context);
        }
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:56,81-93
    pub fn identity(&self) -> PictureLayerTilingSetId {
        self.identity_
    }
    pub fn client(&self) -> PictureLayerTilingClientId {
        self.client_
    }
    pub fn num_tilings(&self) -> usize {
        self.tilings_.len()
    }
    pub fn all_tiles_done(&self) -> bool {
        self.all_tiles_done_
    }
    pub fn set_all_tiles_done(&mut self, done: bool) {
        self.all_tiles_done_ = done;
    }
    pub fn tiling_at(&self, index: usize) -> &PictureLayerTiling<B> {
        &self.tilings_[index]
    }
    pub fn tiling_at_mut(&mut self, index: usize) -> &mut PictureLayerTiling<B> {
        &mut self.tilings_[index]
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:89-92,210
    // Call-scoped native tiling_at traversal, lending the actual Box slice.
    pub fn TilingsMutable(&mut self) -> std::slice::IterMut<'_, Box<PictureLayerTiling<B>>> {
        self.tilings_.iter_mut()
    }
    pub fn tree(&self) -> WhichTree {
        self.tree_
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:192-195
    // Source: cc/tiles/picture_layer_tiling_set.cc:76-119
    // The native const pending-set parameter still mutates its unique_ptr
    // pointees; Rust makes that actual donor ownership mutation explicit.
    fn CopyTilingsAndPropertiesFromPendingTwin(
        &mut self,
        pending_twin_set: &mut Self,
        _raster_source: Arc<B::RasterSource>,
        layer_invalidation: &Region<B::RegionBackend>,
        context: &mut dyn PictureLayerTilingSetContext<B>,
    ) {
        if pending_twin_set.tilings_.is_empty() {
            self.RemoveAllTilings(context);
            return;
        }
        let mut tiling_sort_required = false;
        for pending_twin_tiling in &mut pending_twin_set.tilings_ {
            let raster_transform = pending_twin_tiling.raster_transform().clone();
            let can_use_lcd_text = pending_twin_tiling.can_use_lcd_text();
            let mut this_tiling =
                self.FindTilingWithScaleKey(pending_twin_tiling.contents_scale_key());
            if let Some(id) = this_tiling {
                let tiling = self.TilingById(id);
                if *tiling.raster_transform() != raster_transform
                    || tiling.can_use_lcd_text() != can_use_lcd_text
                {
                    self.Remove(id, context);
                    this_tiling = None;
                }
            }
            if this_tiling.is_none() {
                let id = context.AllocateTilingId();
                let new_tiling = Box::new(PictureLayerTiling::new(
                    id,
                    self.client_,
                    self.tree_,
                    &raster_transform,
                    Arc::clone(self.raster_source_.as_ref().expect("active raster source")),
                    kMaxSoonBorderDistanceInScreenPixels,
                    self.max_preraster_distance_,
                    can_use_lcd_text,
                    &mut SetTilingContext {
                        external: context,
                        all_tiles_done: &mut self.all_tiles_done_,
                    },
                ));
                self.tilings_.push(new_tiling);
                this_tiling = Some(self.tilings_.last().unwrap().identity());
                tiling_sort_required = true;
                self.state_since_last_tile_priority_update_.added_tilings = true;
            }
            let identity = this_tiling.unwrap();
            let this_tiling = self
                .tilings_
                .iter_mut()
                .find(|tiling| tiling.identity() == identity)
                .expect("native selected resident tiling");
            this_tiling.TakeTilesAndPropertiesFrom(
                pending_twin_tiling,
                layer_invalidation,
                &mut SetTilingContext {
                    external: context,
                    all_tiles_done: &mut self.all_tiles_done_,
                },
            );
            let done = this_tiling.all_tiles_done();
            self.all_tiles_done_ &= done;
        }
        if tiling_sort_required {
            self.SortTilings();
        }
    }
    // Source: cc/tiles/picture_layer_tiling_set.cc:115-118,274
    // Share the same native sort expression; only the Box positions move.
    fn SortTilings(&mut self) {
        self.tilings_.sort_unstable_by(|left, right| {
            if LargestToSmallestScaleFunctor::compare(left, right) {
                std::cmp::Ordering::Less
            } else if LargestToSmallestScaleFunctor::compare(right, left) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
    }
    pub fn TilingById(&self, id: PictureLayerTilingId) -> &PictureLayerTiling<B> {
        self.TilingByIdIfPresent(id)
            .expect("resident tiling identity")
    }
    pub fn TilingByIdIfPresent(&self, id: PictureLayerTilingId) -> Option<&PictureLayerTiling<B>> {
        self.tilings_
            .iter()
            .find(|tiling| tiling.identity() == id)
            .map(Box::as_ref)
    }
    pub fn TilingByIdMut(&mut self, id: PictureLayerTilingId) -> &mut PictureLayerTiling<B> {
        self.tilings_
            .iter_mut()
            .find(|tiling| tiling.identity() == id)
            .expect("resident tiling identity")
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:60-66
    // Source: cc/tiles/picture_layer_tiling_set.cc:121-159
    pub fn UpdateTilingsToCurrentRasterSourceForActivation(
        &mut self,
        raster_source: Arc<B::RasterSource>,
        pending_twin_set: &mut Self,
        layer_invalidation: &Region<B::RegionBackend>,
        minimum_contents_scale: f32,
        maximum_contents_scale: f32,
        context: &mut dyn PictureLayerTilingSetContext<B>,
    ) {
        self.RemoveTilingsBelowScaleKey(minimum_contents_scale, context);
        self.RemoveTilingsAboveScaleKey(maximum_contents_scale, context);
        self.raster_source_ = Some(Arc::clone(&raster_source));
        self.CopyTilingsAndPropertiesFromPendingTwin(
            pending_twin_set,
            Arc::clone(&raster_source),
            layer_invalidation,
            context,
        );
        for tiling in &mut self.tilings_ {
            if pending_twin_set
                .FindTilingWithScaleKey(tiling.contents_scale_key())
                .is_some()
            {
                continue;
            }
            tiling.SetRasterSourceAndResize(
                Arc::clone(&raster_source),
                &mut SetTilingContext {
                    external: &mut PendingTwinSetContext {
                        external: context,
                        pending: pending_twin_set,
                    },
                    all_tiles_done: &mut self.all_tiles_done_,
                },
            );
            tiling.Invalidate(
                layer_invalidation,
                &mut SetTilingContext {
                    external: &mut PendingTwinSetContext {
                        external: context,
                        pending: pending_twin_set,
                    },
                    all_tiles_done: &mut self.all_tiles_done_,
                },
            );
            self.state_since_last_tile_priority_update_.invalidated = true;
            tiling.CreateMissingTilesInLiveTilesRect(&mut SetTilingContext {
                external: &mut PendingTwinSetContext {
                    external: context,
                    pending: pending_twin_set,
                },
                all_tiles_done: &mut self.all_tiles_done_,
            });
            tiling.set_resolution(TileResolution::NON_IDEAL_RESOLUTION);
            self.all_tiles_done_ &= tiling.all_tiles_done();
        }
        self.VerifyTilings(Some(pending_twin_set), context);
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:68-73
    // Source: cc/tiles/picture_layer_tiling_set.cc:161-199
    pub fn UpdateTilingsToCurrentRasterSourceForCommit(
        &mut self,
        raster_source: Arc<B::RasterSource>,
        layer_invalidation: &Region<B::RegionBackend>,
        minimum_contents_scale: f32,
        maximum_contents_scale: f32,
        context: &mut dyn PictureLayerTilingSetContext<B>,
    ) {
        self.RemoveTilingsBelowScaleKey(minimum_contents_scale, context);
        self.RemoveTilingsAboveScaleKey(maximum_contents_scale, context);
        self.raster_source_ = Some(Arc::clone(&raster_source));
        self.all_tiles_done_ = true;
        for tiling in &mut self.tilings_ {
            debug_assert!(self.tree_ != WhichTree::PENDING_TREE || !tiling.has_tiles());
            self.state_since_last_tile_priority_update_
                .tiling_needs_update |= tiling.SetRasterSourceAndResize(
                Arc::clone(&raster_source),
                &mut SetTilingContext {
                    external: context,
                    all_tiles_done: &mut self.all_tiles_done_,
                },
            );
            self.state_since_last_tile_priority_update_.invalidated = true;
            if self.tree_ == WhichTree::ACTIVE_TREE {
                tiling.Invalidate(
                    layer_invalidation,
                    &mut SetTilingContext {
                        external: context,
                        all_tiles_done: &mut self.all_tiles_done_,
                    },
                );
            }
            tiling.CreateMissingTilesInLiveTilesRect(&mut SetTilingContext {
                external: context,
                all_tiles_done: &mut self.all_tiles_done_,
            });
            self.all_tiles_done_ &= tiling.all_tiles_done();
        }
        self.VerifyTilings(None, context);
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:75-76
    // Source: cc/tiles/picture_layer_tiling_set.cc:201-209
    pub fn Invalidate(
        &mut self,
        layer_invalidation: &Region<B::RegionBackend>,
        context: &mut dyn PictureLayerTilingSetContext<B>,
    ) {
        self.all_tiles_done_ = true;
        for tiling in &mut self.tilings_ {
            tiling.Invalidate(
                layer_invalidation,
                &mut SetTilingContext {
                    external: context,
                    all_tiles_done: &mut self.all_tiles_done_,
                },
            );
            tiling.CreateMissingTilesInLiveTilesRect(&mut SetTilingContext {
                external: context,
                all_tiles_done: &mut self.all_tiles_done_,
            });
            self.all_tiles_done_ &= tiling.all_tiles_done();
        }
        self.state_since_last_tile_priority_update_.invalidated = true;
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:197
    // Source: cc/tiles/picture_layer_tiling_set.cc:211-237
    fn VerifyTilings(
        &self,
        pending_twin_set: Option<&Self>,
        context: &mut dyn PictureLayerTilingSetContext<B>,
    ) {
        #[cfg(not(debug_assertions))]
        let _ = (pending_twin_set, context);
        #[cfg(debug_assertions)]
        {
            for tiling in &self.tilings_ {
                let rect = tiling.tiling_rect();
                let expected = context.CalculateTileSizeForClient(
                    self.client_,
                    compositor_tree::geometry::Size::new(rect.width, rect.height),
                );
                debug_assert_eq!(
                    tiling.tile_size(),
                    expected,
                    "tile_size: {:?} tiling_size: {} CalculateTileSize: {:?}",
                    tiling.tile_size(),
                    rect.ToString(),
                    context.CalculateTileSizeForClient(
                        self.client_,
                        compositor_tree::geometry::Size::new(rect.width, rect.height)
                    )
                );
            }
            if !self.tilings_.is_empty() {
                debug_assert!(self.NumHighResTilings() <= 1);
                if let Some(pending_twin_set) = pending_twin_set {
                    debug_assert_eq!(
                        self.NumHighResTilings(),
                        1,
                        " num tilings on active: {} num tilings on pending: {} num high res on pending: {} are on active tree: {}",
                        self.tilings_.len(),
                        pending_twin_set.tilings_.len(),
                        pending_twin_set.NumHighResTilings(),
                        self.tree_ == WhichTree::ACTIVE_TREE
                    );
                }
            }
        }
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:58
    // Source: cc/tiles/picture_layer_tiling_set.cc:241-245
    pub fn RemoveNonIdealTilings(&mut self, context: &mut dyn PictureLayerTilingSetContext<B>) {
        self.EraseTilingsIf(
            |tiling| tiling.resolution() == TileResolution::NON_IDEAL_RESOLUTION,
            context,
        );
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:106
    // Source: cc/tiles/picture_layer_tiling_set.cc:247-250
    pub fn MarkAllTilingsNonIdeal(&mut self) {
        for tiling in &mut self.tilings_ {
            tiling.set_resolution(TileResolution::NON_IDEAL_RESOLUTION);
        }
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:78-80
    // Source: cc/tiles/picture_layer_tiling_set.cc:252-276
    pub fn AddTiling(
        &mut self,
        raster_transform: &B::RasterTransform,
        raster_source: Arc<B::RasterSource>,
        can_use_lcd_text: bool,
        context: &mut dyn PictureLayerTilingSetContext<B>,
    ) -> PictureLayerTilingId {
        if self.raster_source_.is_none() {
            self.raster_source_ = Some(Arc::clone(&raster_source));
        }
        #[cfg(debug_assertions)]
        {
            for tiling in &self.tilings_ {
                let scale = raster_transform.scale();
                debug_assert_ne!(tiling.contents_scale_key(), scale.x().max(scale.y()));
                debug_assert!(Arc::ptr_eq(
                    tiling
                        .raster_source()
                        .as_ref()
                        .expect("tiling raster source"),
                    &raster_source
                ));
            }
        }
        let id = context.AllocateTilingId();
        self.tilings_.push(Box::new(PictureLayerTiling::new(
            id,
            self.client_,
            self.tree_,
            raster_transform,
            raster_source,
            kMaxSoonBorderDistanceInScreenPixels,
            self.max_preraster_distance_,
            can_use_lcd_text,
            &mut SetTilingContext {
                external: context,
                all_tiles_done: &mut self.all_tiles_done_,
            },
        )));
        let appended = self.tilings_.last().unwrap().identity();
        self.state_since_last_tile_priority_update_.added_tilings = true;
        self.SortTilings();
        appended
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:88
    // Source: cc/tiles/picture_layer_tiling_set.cc:278-281
    pub fn NumHighResTilings(&self) -> i32 {
        self.tilings_
            .iter()
            .filter(|tiling| tiling.resolution() == TileResolution::HIGH_RESOLUTION)
            .count() as i32
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:95
    // Source: cc/tiles/picture_layer_tiling_set.cc:283-290
    pub fn FindTilingWithScaleKeyRef(&self, scale_key: f32) -> Option<&PictureLayerTiling<B>> {
        Self::FindTilingWithScaleKeyIn(&self.tilings_, scale_key)
    }
    /// Source: cc/tiles/picture_layer_tiling_set.cc:282-289. The one source
    /// search body also accepts this owner's disjoint borrowed tilings field.
    pub fn FindTilingWithScaleKeyIn(
        tilings: &[Box<PictureLayerTiling<B>>],
        scale_key: f32,
    ) -> Option<&PictureLayerTiling<B>> {
        for tiling in tilings {
            if tiling.contents_scale_key() == scale_key {
                return Some(tiling);
            }
        }
        None
    }
    pub fn SplitTileNotification(&mut self) -> (&[Box<PictureLayerTiling<B>>], &mut bool) {
        (&self.tilings_, &mut self.all_tiles_done_)
    }
    pub fn FindTilingWithScaleKey(&self, scale_key: f32) -> Option<PictureLayerTilingId> {
        self.FindTilingWithScaleKeyRef(scale_key)
            .map(|tiling| tiling.identity())
    }
    /// Mutable overload lends the selected native owning vector entry directly.
    pub fn FindTilingWithScaleKeyMutable(
        &mut self,
        scale_key: f32,
    ) -> Option<&mut PictureLayerTiling<B>> {
        self.tilings_
            .iter_mut()
            .find(|tiling| tiling.contents_scale_key() == scale_key)
            .map(Box::as_mut)
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:96
    // Source: cc/tiles/picture_layer_tiling_set.cc:292-299
    pub fn FindTilingWithResolution(
        &self,
        resolution: TileResolution,
    ) -> Option<PictureLayerTilingId> {
        self.tilings_
            .iter()
            .find(|tiling| tiling.resolution() == resolution)
            .map(|tiling| tiling.identity())
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:98-104
    // Source: cc/tiles/picture_layer_tiling_set.cc:301-315
    pub fn FindTilingWithNearestScaleKeyRef(
        &self,
        start_scale: f32,
        snap_to_existing_tiling_ratio: f32,
    ) -> Option<&PictureLayerTiling<B>> {
        let mut nearest_tiling = None;
        let mut nearest_ratio = snap_to_existing_tiling_ratio;
        for tiling in &self.tilings_ {
            let tiling_contents_scale = tiling.contents_scale_key();
            let ratio = LargerRatio(tiling_contents_scale, start_scale);
            if ratio <= nearest_ratio {
                nearest_tiling = Some(tiling.as_ref());
                nearest_ratio = ratio;
            }
        }
        nearest_tiling
    }
    // Safe nonowning queue-identity bridge; the unique native selection body
    // above also exposes its actual resident result directly to PictureLayerImpl.
    pub fn FindTilingWithNearestScaleKey(
        &self,
        start_scale: f32,
        ratio: f32,
    ) -> Option<PictureLayerTilingId> {
        self.FindTilingWithNearestScaleKeyRef(start_scale, ratio)
            .map(|tiling| tiling.identity())
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:116-117
    // Source: cc/tiles/picture_layer_tiling_set.cc:317-324
    pub fn RemoveTilingsBelowScaleKey(
        &mut self,
        minimum_scale_key: f32,
        context: &mut dyn PictureLayerTilingSetContext<B>,
    ) {
        self.EraseTilingsIf(
            |tiling| tiling.contents_scale_key() < minimum_scale_key,
            context,
        );
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:119-120
    // Source: cc/tiles/picture_layer_tiling_set.cc:326-333
    pub fn RemoveTilingsAboveScaleKey(
        &mut self,
        maximum_scale_key: f32,
        context: &mut dyn PictureLayerTilingSetContext<B>,
    ) {
        self.EraseTilingsIf(
            |tiling| tiling.contents_scale_key() > maximum_scale_key,
            context,
        );
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:122-123
    // Source: cc/tiles/picture_layer_tiling_set.cc:335-338
    pub fn ReleaseAllResources(&mut self, context: &mut dyn PictureLayerTilingSetContext<B>) {
        self.RemoveAllTilings(context);
        self.raster_source_ = None;
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:125-126
    // Source: cc/tiles/picture_layer_tiling_set.cc:340-343
    pub fn RemoveAllTilings(&mut self, context: &mut dyn PictureLayerTileDestroyer<B>) {
        Self::DestroyTilings(&mut self.tilings_, context);
        self.all_tiles_done_ = true;
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:113-114
    // Source: cc/tiles/picture_layer_tiling_set.cc:345-351
    pub fn Remove(
        &mut self,
        tiling: PictureLayerTilingId,
        context: &mut dyn PictureLayerTilingSetContext<B>,
    ) {
        let Some(iter) = self
            .tilings_
            .iter()
            .position(|resident| resident.identity() == tiling)
        else {
            return;
        };
        self.tilings_[iter].Destroy(context);
        self.tilings_.remove(iter);
    }
    // Rust ownership mapping of the native erase_if calls above. The predicate
    // reads each actual child; Destroy performs canonical release before the Box
    // leaves membership. No state or destructor algorithm moves into services.
    // Source: cc/tiles/picture_layer_tiling_set.cc:241-245,317-324,326-333
    fn EraseTilingsIf(
        &mut self,
        predicate: impl Fn(&PictureLayerTiling<B>) -> bool,
        context: &mut dyn PictureLayerTilingSetContext<B>,
    ) {
        self.tilings_.retain_mut(|tiling| {
            if predicate(tiling) {
                tiling.Destroy(context);
                false
            } else {
                true
            }
        });
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:128-129
    // Source: cc/tiles/picture_layer_tiling_set.cc:353-357
    pub fn RemoveAllTiles(&mut self, context: &mut dyn PictureLayerTilingSetContext<B>) {
        for tiling in &mut self.tilings_ {
            tiling.Reset(context);
        }
        self.all_tiles_done_ = true;
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:108-111
    // Source: cc/tiles/picture_layer_tiling_set.cc:359-364
    pub fn GetMaximumContentsScale(&self) -> f32 {
        if self.tilings_.is_empty() {
            return 0.0;
        }
        self.tilings_[0].contents_scale_key()
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:131-137
    // Source: cc/tiles/picture_layer_tiling_set.cc:520-548
    pub fn UpdateTilePriorities(
        &mut self,
        visible_rect_in_layer_space: Rect,
        ideal_contents_scale: f32,
        current_frame_time_in_seconds: f64,
        occlusion_in_layer_space: &Occlusion<B::OcclusionTransform>,
        can_require_tiles_for_activation: bool,
        context: &mut dyn PictureLayerTilingSetContext<B>,
    ) -> bool {
        let auto_clear_state = AutoClear::new(&mut self.state_since_last_tile_priority_update_);
        if !TilingsNeedUpdate(
            self.tilings_.len(),
            &self.visible_rect_history_,
            &auto_clear_state,
            &visible_rect_in_layer_space,
            current_frame_time_in_seconds,
        ) {
            return auto_clear_state.invalidated;
        }
        // Split only the actual fields needed by the single canonical geometry
        // body, while AutoClear retains the source state field's mutable borrow.
        PriorityGeometry {
            tiling_interest_area_padding_: &self.tiling_interest_area_padding_,
            skewport_target_time_in_seconds_: &self.skewport_target_time_in_seconds_,
            skewport_extrapolation_limit_in_screen_pixels_: &self
                .skewport_extrapolation_limit_in_screen_pixels_,
            visible_rect_history_: &mut self.visible_rect_history_,
            visible_rect_in_layer_space_: &mut self.visible_rect_in_layer_space_,
            skewport_rect_in_layer_space_: &mut self.skewport_rect_in_layer_space_,
            soon_border_rect_in_layer_space_: &mut self.soon_border_rect_in_layer_space_,
            eventually_rect_in_layer_space_: &mut self.eventually_rect_in_layer_space_,
        }
        .UpdatePriorityRects(
            visible_rect_in_layer_space,
            current_frame_time_in_seconds,
            ideal_contents_scale,
            || {
                self.raster_source_
                    .as_ref()
                    .expect("priority update raster source")
                    .recorded_bounds()
            },
        );
        self.all_tiles_done_ = true;
        for tiling in &mut self.tilings_ {
            tiling.set_can_require_tiles_for_activation(can_require_tiles_for_activation);
            tiling.ComputeTilePriorityRects(
                &self.visible_rect_in_layer_space_,
                &self.skewport_rect_in_layer_space_,
                &self.soon_border_rect_in_layer_space_,
                &self.eventually_rect_in_layer_space_,
                ideal_contents_scale,
                occlusion_in_layer_space,
                &mut SetTilingContext {
                    external: context,
                    all_tiles_done: &mut self.all_tiles_done_,
                },
            );
            self.all_tiles_done_ &= tiling.all_tiles_done();
        }
        true
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:139-140
    // Source: cc/tiles/picture_layer_tiling_set.cc:550-554
    pub fn GetAllPrioritizedTilesForTracing(
        &mut self,
        prioritized_tiles: &mut Vec<PrioritizedTile>,
        queries: &dyn PictureLayerTilingQueries<B>,
    ) {
        for tiling in &mut self.tilings_ {
            tiling.GetAllPrioritizedTilesForTracing(prioritized_tiles, queries);
        }
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:142-145
    // Source: cc/tiles/picture_layer_tiling_set.cc:556-562
    pub fn Cover(
        &self,
        coverage_rect: Rect,
        coverage_scale: f32,
        ideal_contents_scale: f32,
    ) -> TilingSetCoverageIterator<'_, PictureLayerTiling<B>, B::RegionBackend> {
        TilingSetCoverageIterator::new(
            &self.tilings_,
            coverage_rect,
            coverage_scale,
            ideal_contents_scale,
        )
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:147
    // Source: cc/tiles/picture_layer_tiling_set.cc:564-571
    pub fn AsValueInto(&self, state: &mut impl PropertyTrace) {
        for tiling in &self.tilings_ {
            state.BeginDictionaryUnnamed();
            tiling.AsValueInto(state);
            state.EndDictionary();
        }
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:148
    // Source: cc/tiles/picture_layer_tiling_set.cc:573-578
    pub fn GPUMemoryUsageInBytes(&self) -> usize {
        let mut amount = 0;
        for tiling in &self.tilings_ {
            amount += tiling.GPUMemoryUsageInBytes();
        }
        amount
    }
    // Source: cc/tiles/picture_layer_tiling_set.h:150
    // Source: cc/tiles/picture_layer_tiling_set.cc:580-608
    pub fn GetTilingRange(&self, r#type: TilingRangeType) -> TilingRange {
        let tilings_size = self.tilings_.len();
        let mut high_res_range = TilingRange::new(0, 0);
        for i in 0..tilings_size {
            let tiling = &self.tilings_[i];
            if tiling.resolution() == TileResolution::HIGH_RESOLUTION {
                high_res_range = TilingRange::new(i, i + 1);
            }
        }
        let range = match r#type {
            TilingRangeType::HIGHER_THAN_HIGH_RES => TilingRange::new(0, high_res_range.start),
            TilingRangeType::HIGH_RES => high_res_range,
            TilingRangeType::LOWER_THAN_HIGH_RES => {
                TilingRange::new(high_res_range.end, tilings_size)
            }
        };
        debug_assert!(range.start <= range.end);
        range
    }
}
impl<B: PictureLayerTilingTypes> Drop for PictureLayerTilingSet<B> {
    // Source: cc/tiles/picture_layer_tiling_set.cc:74
    fn drop(&mut self) {
        debug_assert!(
            self.tilings_.is_empty(),
            "tiling set must release canonical child tiles with a live context before destruction"
        );
    }
}
// Source: cc/tiles/picture_layer_tiling_set.h:235-237
// Source: cc/tiles/picture_layer_tiling_set.cc:610
// Namespace/guards/deleted copy operations map to Rust module and move-only Box.

// The source holds a const pending-set pointer while updating an active child.
// This is an immutable short reborrow of that same pending allocation, released
// before any subsequent pending mutation. No tiling vector is collected/copied.
struct PendingTwinSetContext<'a, B: PictureLayerTilingTypes> {
    external: &'a mut dyn PictureLayerTilingSetContext<B>,
    pending: &'a PictureLayerTilingSet<B>,
}
impl<B: PictureLayerTilingTypes> PictureLayerTilingQueries<B> for PendingTwinSetContext<'_, B> {
    fn WithClient(
        &self,
        id: PictureLayerTilingClientId,
        callback: &mut dyn FnMut(&dyn crate::picture_layer_tiling::PictureLayerTilingClient<B>),
    ) {
        self.external.WithClient(id, callback)
    }
    fn PaintWorkletRecordsForClient(
        &self,
        id: PictureLayerTilingClientId,
    ) -> &B::PaintWorkletRecordMap {
        self.external.PaintWorkletRecordsForClient(id)
    }
    fn TilingById(&self, id: PictureLayerTilingId) -> &PictureLayerTiling<B> {
        self.pending
            .TilingByIdIfPresent(id)
            .unwrap_or_else(|| self.external.TilingById(id))
    }
    fn IsClientOnActiveOrPendingTree(&self, id: PictureLayerTilingClientId) -> bool {
        self.external.IsClientOnActiveOrPendingTree(id)
    }
    fn TilingStateForClientAndScale(
        &self,
        id: PictureLayerTilingClientId,
        scale: f32,
    ) -> Option<&crate::picture_layer_tiling::PictureLayerTilingState<B>> {
        if id == self.pending.client_ {
            self.pending
                .FindTilingWithScaleKeyRef(scale)
                .map(std::ops::Deref::deref)
        } else {
            self.external.TilingStateForClientAndScale(id, scale)
        }
    }
    fn PendingInvalidationById(&self, id: PendingInvalidationId) -> &Region<B::RegionBackend> {
        self.external.PendingInvalidationById(id)
    }
}
impl<B: PictureLayerTilingTypes> PictureLayerTileDestroyer<B> for PendingTwinSetContext<'_, B> {
    fn RebindTileWithManager(&mut self, old: TileId, new: TileId) {
        self.external.RebindTileWithManager(old, new)
    }
    fn DestroyTileWithManager(
        &mut self,
        id: TileManagerId,
        tile: &mut Tile<B>,
        tiling: &crate::picture_layer_tiling::PictureLayerTilingState<B>,
    ) {
        self.external.DestroyTileWithManager(id, tile, tiling)
    }
}
impl<B: PictureLayerTilingTypes> PictureLayerTilingSetContext<B> for PendingTwinSetContext<'_, B> {
    fn CalculateTileSizeForClient(
        &mut self,
        id: PictureLayerTilingClientId,
        bounds: compositor_tree::geometry::Size,
    ) -> compositor_tree::geometry::Size {
        self.external.CalculateTileSizeForClient(id, bounds)
    }
    fn CreateTileForClient(
        &mut self,
        id: PictureLayerTilingClientId,
        info: &CreateInfo<B::RasterTransform>,
        done: &mut bool,
    ) -> Option<Box<Tile<B>>> {
        self.external.CreateTileForClient(id, info, done)
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.external.IsCCSlimmingEnabled()
    }
    fn AllocateTilingId(&mut self) -> PictureLayerTilingId {
        self.external.AllocateTilingId()
    }
}
