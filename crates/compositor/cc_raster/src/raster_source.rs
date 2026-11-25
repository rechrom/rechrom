//! cc::RasterSource (cc/raster/raster_source.h). RecordingSource (main)
//! creates it (RecordingSource::CreateRasterSource, in layerize) and the impl
//! tilings and the raster service read it. Impl-only facets
//! (PictureRasterSource, TileManagerRasterSource) are implemented by the
//! compositor, which owns those traits.
use cc_paint::display_item_list_types::DirectlyCompositedImageInfo;
use cc_paint::skia_backend::RecordSkiaBackend;
use cc_paint::ContentDisplayItemList;
use compositor_tree::geometry::{Rect, Size};
use compositor_values::color4f::Color4f;
use compositor_values::raster_source::TilingRasterSource;
use std::{
    ops::Deref,
    sync::{Arc, RwLock, RwLockReadGuard},
};

/// One recorded full-replay region for partial raster
/// (cc/raster/raster_source.h, RecordingSource invalidation bookkeeping).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartialRasterFullReplay {
    pub outer: Rect,
    pub inner: Rect,
    pub replay_rect: Rect,
}

/// Source: cc/raster/raster_source.cc:26-40. These are precisely the immutable
/// metadata values owned by RasterSource; its recording lease remains shared.
pub struct RasterSourceStorage<R: RecordSkiaBackend> {
    pub display_list: Option<Arc<ContentDisplayItemList<R>>>,
    pub background_color: Color4f,
    pub requires_clear: bool,
    pub is_solid_color: bool,
    pub solid_color: Color4f,
    pub recorded_bounds: Rect,
    pub size: Size,
    pub slow_down_raster_scale_factor_for_debug: i32,
    pub recording_scale_factor: f32,
    pub directly_composited_image_info: Option<DirectlyCompositedImageInfo>,
    pub full_replays_for_partial_raster: Vec<PartialRasterFullReplay>,
    pub debug_name: RwLock<String>,
}
impl<R: RecordSkiaBackend> RasterSourceStorage<R> {
    pub fn display_list(&self) -> &Option<Arc<ContentDisplayItemList<R>>> {
        &self.display_list
    }
    // Source: cc/raster/raster_source.h GetDisplayItemList.
    pub fn GetDisplayItemList(&self) -> &Option<Arc<ContentDisplayItemList<R>>> {
        &self.display_list
    }
    pub fn FullReplaysForPartialRaster(&self) -> &[PartialRasterFullReplay] {
        &self.full_replays_for_partial_raster
    }
    pub fn DebugName(&self) -> RasterDebugName<'_> {
        RasterDebugName(
            self.debug_name
                .read()
                .expect("native source debug name poisoned"),
        )
    }
}
pub struct RasterDebugName<'a>(RwLockReadGuard<'a, String>);
impl Deref for RasterDebugName<'_> {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.as_str()
    }
}
impl<R: RecordSkiaBackend> TilingRasterSource for RasterSourceStorage<R> {
    fn IsSolidColor(&self) -> bool {
        self.is_solid_color
    }
    fn size(&self) -> Size {
        self.size
    }
    // Source: cc/raster/raster_source.h:118-122. Direct image rasterization uses
    // full size; the stored recorded rectangle itself is never rewritten.
    fn recorded_bounds(&self) -> Rect {
        if self.directly_composited_image_info.is_some() {
            Rect::new(0, 0, self.size.width(), self.size.height())
        } else {
            self.recorded_bounds
        }
    }
    fn IntersectsRect(&self, rect: Rect) -> bool {
        self.recorded_bounds().Intersects(rect)
    }
}
