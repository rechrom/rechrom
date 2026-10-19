// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tile.h:1-29,237-239; cc/tiles/tile.cc:1-20,95.
// Modules replace includes/namespaces/export declarations. Tile is not Clone,
// corresponding to the deleted source copy constructor and assignment.
use super::{
    tile_draw_info::{Mode, TileDrawInfo},
    tiling_internal::TileLike,
};
use crate::adapter::{
    geometry::{AxisTransform2d, Rect, Size, cpp_max},
    layer_tree_impl::{Color4f, ResourceId},
    property_types::PropertyTrace,
    resource_types::TileDrawResource,
    tile::{
        PictureLayerTilingId, TileId, TileIdAllocator, TileManagerContext, TileManagerId,
        TileRasterTask, TileTraceContext, TileTraceId, TileTypes,
    },
};
use crate::cc::base::math_util::MathUtil;
use std::sync::Arc;

// Source: cc/tiles/tile.h:31-44.
#[derive(Clone)]
pub struct CreateInfo<A: AxisTransform2d> {
    pub tiling: Option<PictureLayerTilingId>,
    pub tiling_i_index: i32,
    pub tiling_j_index: i32,
    pub enclosing_layer_rect: Rect,
    pub content_rect: Rect,
    pub raster_transform: A,
    pub can_use_lcd_text: bool,
}
impl<A: AxisTransform2d> Default for CreateInfo<A> {
    // Source: cc/tiles/tile.h:37-43.
    fn default() -> Self {
        Self {
            tiling: None,
            tiling_i_index: 0,
            tiling_j_index: 0,
            enclosing_layer_rect: Rect::default(),
            content_rect: Rect::default(),
            raster_transform: A::default(),
            can_use_lcd_text: false,
        }
    }
}
// Source: cc/tiles/tile.h:46.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TileRasterFlags {
    USE_PICTURE_ANALYSIS = 1 << 0,
    IS_OPAQUE = 1 << 1,
}
// Source: cc/tiles/tile.h:48.
pub type Id = u64;

// Source: cc/tiles/tile.h:189-235.
// Rust fields drop in declaration order; C++ members drop in reverse order.
// The observable owned-task -> draw-resource ordering is retained explicitly.
// Manager/tiling fields are non-owning identities, with no retained owner route.
pub struct Tile<B: TileTypes> {
    // Source: cc/tiles/tile.h:232.
    pub(crate) raster_task_: Option<Arc<B::RasterTask>>,
    // Source: cc/tiles/tile.h:201.
    pub(crate) draw_info_: TileDrawInfo<B::Resource>,
    // Source: cc/tiles/tile.h:199.
    raster_transform_: B::RasterTransform,
    // Source: cc/tiles/tile.h:194-198.
    tile_manager_: TileManagerId,
    tiling_: Option<PictureLayerTilingId>,
    content_rect_: Rect,
    enclosing_layer_rect_: Rect,
    // Source: cc/tiles/tile.h:203-207.
    layer_id_: i32,
    source_frame_number_: i32,
    flags_: i32,
    tiling_i_index_: i32,
    tiling_j_index_: i32,
    // Source: cc/tiles/tile.h:209-224.
    invalidated_id_: Id,
    pub(crate) scheduled_priority_: u32,
    // Source MakePrioritizedTile/UpdateRequiredStatesOnTile are const tiling
    // calls (picture_layer_tiling.cc:694-700,708-746). These leaf flags are
    // compositor-sequence logical mutation of this same resident Tile.
    required_for_activation_: std::cell::Cell<bool>,
    required_for_draw_: std::cell::Cell<bool>,
    is_solid_color_analysis_performed_: bool,
    can_use_lcd_text_: bool,
    raster_task_scheduled_with_checker_images_: bool,
    deleted_: bool,
    // Source: cc/tiles/tile.h:226-234.
    id_: Id,
    invalidated_content_rect_: Rect,
    // C++ const coverage cursors still expose a mutable Tile*. This one
    // sequence-local logical leaf stays in the same resident Tile allocation.
    used_: std::cell::Cell<bool>,
}
impl<B: TileTypes> Tile<B> {
    // Source: cc/tiles/tile.h:177-187; cc/tiles/tile.cc:22-38.
    // The actual manager is lent only for source identity allocation.
    pub(crate) fn new(
        manager: &mut impl TileIdAllocator,
        info: &CreateInfo<B::RasterTransform>,
        layer_id: i32,
        source_frame_number: i32,
        flags: i32,
    ) -> Self {
        Self {
            tile_manager_: manager.identity(),
            tiling_: info.tiling,
            content_rect_: info.content_rect,
            enclosing_layer_rect_: info.enclosing_layer_rect,
            raster_transform_: info.raster_transform.clone(),
            draw_info_: TileDrawInfo::new(),
            layer_id_: layer_id,
            source_frame_number_: source_frame_number,
            flags_: flags,
            tiling_i_index_: info.tiling_i_index,
            tiling_j_index_: info.tiling_j_index,
            invalidated_id_: 0,
            scheduled_priority_: 0,
            required_for_activation_: std::cell::Cell::new(false),
            required_for_draw_: std::cell::Cell::new(false),
            is_solid_color_analysis_performed_: false,
            can_use_lcd_text_: info.can_use_lcd_text,
            raster_task_scheduled_with_checker_images_: false,
            deleted_: false,
            id_: manager.GetUniqueTileId(),
            invalidated_content_rect_: Rect::default(),
            raster_task_: None,
            used_: std::cell::Cell::new(false),
        }
    }
    // Source: cc/tiles/tile.h:51; cc/tiles/tile.cc:40-45.
    // C++ destruction calls back into its owner. Rust's owning manager invokes
    // this explicit source destructor before removing/dropping the tile; Drop
    // cannot borrow that manager and only validates completed teardown below.
    pub(crate) fn Destroy(&mut self, manager: &mut impl TileManagerContext<B>) {
        assert_eq!(
            self.tile_manager_,
            manager.identity(),
            "source TileManager identity"
        );
        debug_assert!(!self.deleted_, "source destructor runs once");
        manager.TraceTileDeleted(
            "disabled-by-default-cc.debug",
            "cc::Tile:deleted",
            TileTraceId(std::ptr::from_ref(self) as usize),
            "Tile",
        );
        self.deleted_ = true;
        manager.Release(self);
    }
    // Source: cc/tiles/tile.h:55-57.
    // Rust non-owning pointer bridge: both components identify this resident Tile.
    pub(crate) fn identity(&self) -> TileId {
        TileId {
            manager: self.tile_manager_,
            id: self.id_,
            tiling: self
                .tiling_
                .expect("detached tile has no resident identity"),
            index: super::tile_index::TileIndex::new(self.tiling_i_index_, self.tiling_j_index_),
        }
    }
    pub fn id(&self) -> Id {
        self.id_
    }
    // Source: cc/tiles/tile.h:59-62.
    pub fn draw_mode(&self) -> Mode {
        assert!(self.IsReadyToDraw());
        self.draw_info().mode()
    }
    // Source: cc/tiles/tile.h:64.
    pub fn IsReadyToDraw(&self) -> bool {
        self.draw_info().IsReadyToDraw()
    }
    // Source: cc/tiles/tile.h:66-71.
    pub fn GetResourceId(&self) -> Option<ResourceId> {
        if self.IsReadyToDraw() && self.draw_info().mode() == Mode::RESOURCE_MODE {
            return Some(self.draw_info().resource_id_for_export());
        }
        None
    }
    // Source: cc/tiles/tile.h:73-78.
    pub fn GetResourceSize(&self) -> Option<Size> {
        if self.IsReadyToDraw() && self.draw_info().mode() == Mode::RESOURCE_MODE {
            return Some(*self.draw_info().resource_size());
        }
        None
    }
    // Source: cc/tiles/tile.h:80-85.
    pub fn GetSolidColor(&self) -> Option<Color4f> {
        if self.draw_info().mode() == Mode::SOLID_COLOR_MODE {
            return Some(self.draw_info().solid_color());
        }
        None
    }
    // Source: cc/tiles/tile.h:87.
    pub fn IsOOM(&self) -> bool {
        self.draw_info().mode() == Mode::OOM_MODE
    }
    // Source: cc/tiles/tile.h:89-97.
    pub fn required_for_activation(&self) -> bool {
        self.required_for_activation_.get()
    }
    pub fn set_required_for_activation(&self, is_required: bool) {
        self.required_for_activation_.set(is_required)
    }
    pub fn required_for_draw(&self) -> bool {
        self.required_for_draw_.get()
    }
    pub fn set_required_for_draw(&self, is_required: bool) {
        self.required_for_draw_.set(is_required)
    }
    // Source: cc/tiles/tile.h:99-101.
    pub fn is_prepaint(&self) -> bool {
        !self.required_for_activation() && !self.required_for_draw()
    }
    // Source: cc/tiles/tile.h:103-107.
    pub fn use_picture_analysis(&self) -> bool {
        (self.flags_ & TileRasterFlags::USE_PICTURE_ANALYSIS as i32) != 0
    }
    pub fn is_opaque(&self) -> bool {
        (self.flags_ & TileRasterFlags::IS_OPAQUE as i32) != 0
    }
    // Source: cc/tiles/tile.h:109; cc/tiles/tile.cc:47-79.
    pub fn AsValueInto(&self, value: &mut impl PropertyTrace, trace: &impl TileTraceContext) {
        trace.MakeDictIntoImplicitSnapshotWithCategory(
            "disabled-by-default-cc.debug",
            value,
            "cc::Tile",
            TileTraceId(std::ptr::from_ref(self) as usize),
        );
        value.SetDouble("contents_scale", self.contents_scale_key() as f64);
        value.BeginDictionary("raster_transform");
        value.BeginArray("scale");
        value.AppendDouble(self.raster_transform_.scale().x() as f64);
        value.AppendDouble(self.raster_transform_.scale().y() as f64);
        value.EndArray();
        value.BeginArray("translation");
        value.AppendDouble(self.raster_transform_.translation().x() as f64);
        value.AppendDouble(self.raster_transform_.translation().y() as f64);
        value.EndArray();
        value.EndDictionary();
        MathUtil::AddToTracedValue("content_rect", &self.content_rect_, value);
        value.SetInteger("layer_id", self.layer_id_);
        value.BeginDictionary("draw_info");
        self.draw_info_.AsValueInto(value);
        value.EndDictionary();
        value.SetBoolean("has_resource", self.draw_info().has_resource());
        value.SetBoolean(
            "is_using_gpu_memory",
            self.draw_info().has_resource() || self.HasRasterTask(),
        );
        value.SetInteger("scheduled_priority", self.scheduled_priority_ as i32);
        value.SetBoolean("use_picture_analysis", self.use_picture_analysis());
        value.SetInteger(
            "gpu_memory_usage",
            self.GPUMemoryUsageInBytes().min(i32::MAX as usize) as i32,
        );
    }
    // Source: cc/tiles/tile.h:111-112. Rust names the mutable accessor explicitly.
    pub fn draw_info(&self) -> &TileDrawInfo<B::Resource> {
        &self.draw_info_
    }
    pub fn draw_info_mut(&mut self) -> &mut TileDrawInfo<B::Resource> {
        &mut self.draw_info_
    }
    // Source: cc/tiles/tile.h:114-117.
    pub fn contents_scale_key(&self) -> f32 {
        let scale = self.raster_transform_.scale();
        cpp_max(scale.x(), scale.y())
    }
    // Source: cc/tiles/tile.h:118-124.
    pub fn raster_transform(&self) -> &B::RasterTransform {
        &self.raster_transform_
    }
    pub fn content_rect(&self) -> &Rect {
        &self.content_rect_
    }
    pub fn enclosing_layer_rect(&self) -> &Rect {
        &self.enclosing_layer_rect_
    }
    // Source: cc/tiles/tile.h:126-128.
    pub fn layer_id(&self) -> i32 {
        self.layer_id_
    }
    pub fn source_frame_number(&self) -> i32 {
        self.source_frame_number_
    }
    // Source: cc/tiles/tile.h:130; cc/tiles/tile.cc:85-93.
    pub fn GPUMemoryUsageInBytes(&self) -> usize {
        if self.draw_info_.has_resource() {
            let format = self.draw_info_.resource_shared_image_format();
            let size = self.draw_info_.resource_size();
            return self
                .draw_info_
                .GetResource()
                .EstimatedSizeInBytes(format, size);
        }
        0
    }
    // Source: cc/tiles/tile.h:132.
    // Source content_rect_ is immutable; its leaf Size value is projected from
    // the shared Rect boundary's integer dimensions without a retained cache.
    pub fn desired_texture_size(&self) -> Size {
        Size::new(self.content_rect_.width, self.content_rect_.height)
    }
    // Source: cc/tiles/tile.h:134-135.
    pub fn tiling_i_index(&self) -> i32 {
        self.tiling_i_index_
    }
    pub fn tiling_j_index(&self) -> i32 {
        self.tiling_j_index_
    }
    // Source: cc/tiles/tile.h:137-146.
    pub fn SetInvalidated(&mut self, invalid_content_rect: &Rect, previous_tile_id: Id) {
        self.invalidated_content_rect_ = *invalid_content_rect;
        self.invalidated_id_ = previous_tile_id
    }
    pub fn invalidated_id(&self) -> Id {
        self.invalidated_id_
    }
    pub fn invalidated_content_rect(&self) -> &Rect {
        &self.invalidated_content_rect_
    }
    // Source: cc/tiles/tile.h:148.
    pub fn HasRasterTask(&self) -> bool {
        self.raster_task_.is_some()
    }
    // Source: cc/tiles/tile.h:150; cc/tiles/tile.cc:81-83.
    pub fn HasMissingLCPCandidateImages(&self) -> bool {
        self.HasRasterTask()
            && self
                .raster_task_
                .as_ref()
                .unwrap()
                .TaskContainsLCPCandidateImages()
    }
    // Source: cc/tiles/tile.h:152-158.
    pub fn set_solid_color_analysis_performed(&mut self, performed: bool) {
        self.is_solid_color_analysis_performed_ = performed
    }
    pub fn is_solid_color_analysis_performed(&self) -> bool {
        self.is_solid_color_analysis_performed_
    }
    pub fn can_use_lcd_text(&self) -> bool {
        self.can_use_lcd_text_
    }
    // Source: cc/tiles/tile.h:160-167.
    pub fn set_raster_task_scheduled_with_checker_images(
        &mut self,
        has_checker_images: bool,
    ) -> bool {
        let previous_value = self.raster_task_scheduled_with_checker_images_;
        self.raster_task_scheduled_with_checker_images_ = has_checker_images;
        previous_value
    }
    pub fn raster_task_scheduled_with_checker_images(&self) -> bool {
        self.raster_task_scheduled_with_checker_images_
    }
    // Source: cc/tiles/tile.h:169-170.
    pub fn tiling(&self) -> Option<PictureLayerTilingId> {
        self.tiling_
    }
    pub fn set_tiling(&mut self, tiling: Option<PictureLayerTilingId>) {
        self.tiling_ = tiling
    }
    // Source: cc/tiles/tile.h:172-175.
    pub fn mark_used(&self) {
        self.used_.set(true)
    }
    pub fn clear_used(&self) {
        self.used_.set(false)
    }
    pub fn used(&self) -> bool {
        self.used_.get()
    }
    pub fn deleted(&self) -> bool {
        self.deleted_
    }
}
impl<B: TileTypes> TileLike for Tile<B> {
    // Source: cc/tiles/tile.h:64; cc/tiles/tiling_internal.h:17-22.
    fn IsReadyToDraw(&self) -> bool {
        Tile::IsReadyToDraw(self)
    }
}
impl<B: TileTypes> Drop for Tile<B> {
    // Source: cc/tiles/tile.cc:40-45. The owning manager must execute Destroy
    // while its resident context is available. Drop never calls an owner.
    fn drop(&mut self) {
        debug_assert!(
            self.deleted_,
            "TileManager must call Tile::Destroy before drop"
        );
    }
}
