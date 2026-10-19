//! Paint-owned values and platform raster operations used by the translated
//! `cc::DisplayItemList`. The list, visual-rect bookkeeping and RTree remain in
//! cc; PaintOp storage, SkCanvas playback and solid-color analysis stay in the
//! external Paint/raster implementation that owns those values.

use super::{
    geometry::{Rect, Vector2dF},
    input_types::ElementId,
    layer_tree_host_impl::ContentColorUsage,
    layer_tree_impl::Color4f,
    picture_layer_impl::DirectlyCompositedImageInfo,
    prioritized_tile::ScrollOffsetMap,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayPaintOpType {
    Save,
    Restore,
    Translate,
    Concat,
    DrawImageRect,
    DrawRecord,
    DrawTextBlob,
    Other,
}

pub trait DisplayNodeInfo: Sized {
    type NodeId: Copy + PartialEq;
    fn node_id(&self) -> Self::NodeId;
    fn visual_rect_mut(&mut self) -> &mut Rect;
    fn new(node_id: Self::NodeId, visual_rect: Rect) -> Self;
}

/// Read-only native PaintOp projection. Nested records lend their resident
/// buffer; matrices and image rectangles are queried as native scalar values.
pub trait DisplayPaintOp<B: DisplayItemListTypes> {
    fn GetType(&self) -> DisplayPaintOpType;
    fn AdditionalOpCount(&self) -> usize;
    fn HasDrawTextOps(&self) -> bool;
    fn TextNodeId(&self) -> Option<<B::NodeInfo as DisplayNodeInfo>::NodeId>;
    fn RecordBuffer(&self) -> Option<&B::PaintOpBuffer>;
    fn ConcatPreserves2DAxisAlignment(&self) -> bool;
    fn ConcatTransposesAxes(&self) -> bool;
    fn DrawImageRectGeometry(&self) -> Option<(f32, f32, f32, f32)>;
    fn DrawImageUsesNearestNeighbor(&self) -> bool;
}

pub trait DisplayCanvas {
    fn LocalClipBounds(&self) -> Option<Rect>;
}

/// Native PaintOpBuffer and platform operations. These methods expose storage
/// primitives or true external raster/Skia work; cc algorithms are not
/// delegated through this boundary.
pub trait DisplayPlaybackItem {
    fn AdditionalOpCount(&self) -> usize;
}
pub trait DisplayPaintOpBuffer<B: DisplayItemListTypes> {
    type OpRef<'a>: DisplayPaintOp<B>
    where
        Self: 'a,
        B: 'a;
    type Iter<'a>: Iterator<Item = Self::OpRef<'a>>
    where
        Self: 'a,
        B: 'a;
    type OffsetIter<'a>: Iterator<Item = Self::OpRef<'a>>
    where
        Self: 'a,
        B: 'a;
    type PlaybackItem<'a>: DisplayPlaybackItem
    where
        Self: 'a,
        B: 'a;
    type PlaybackFoldingIter<'a>: Iterator<Item = Self::PlaybackItem<'a>>
    where
        Self: 'a,
        B: 'a;

    fn iter(&self) -> Self::Iter<'_>;
    fn iter_offsets<'a>(&'a self, offsets: &'a [usize]) -> Self::OffsetIter<'a>;
    fn playback_folding_iter<'a>(&'a self, offsets: &'a [usize]) -> Self::PlaybackFoldingIter<'a>;
    fn size(&self) -> usize;
    fn next_op_offset(&self) -> usize;
    fn push(&mut self, op: B::PaintOp) -> usize;
    fn UpdateSaveLayerBounds(&mut self, id: usize, bounds: B::SaveLayerBounds);
    fn Playback(
        &self,
        canvas: &mut B::Canvas,
        params: &mut B::PlaybackParams<'_>,
        local_ctm: bool,
        offsets: Option<&[usize]>,
    );
    fn ReleaseAsRecord(&mut self) -> B::PaintRecord;
    fn ShrinkToFit(&mut self);

    fn num_slow_paths_up_to_min_for_MSAA(&self) -> i32;
    fn has_non_aa_paint(&self) -> bool;
    fn total_op_count(&self) -> usize;
    fn bytes_used(&self) -> usize;
    fn paint_ops_size(&self) -> usize;
    fn has_draw_ops(&self) -> bool;
    fn has_draw_text_ops(&self) -> bool;
    fn has_save_layer_ops(&self) -> bool;
    fn has_save_layer_alpha_ops(&self) -> bool;
    fn has_effects_preventing_lcd_text_for_save_layer_alpha(&self) -> bool;
    fn has_discardable_images(&self) -> bool;
    fn content_color_usage(&self) -> ContentColorUsage;
    fn NeedsAdditionalInvalidationForLCDText(&self, old: &Self) -> bool;

    fn GenerateDiscardableImageMap(
        &self,
        bounds: Rect,
        scrolls: &ScrollOffsetMap,
        decoding: Option<&mut B::DecodingModeMap>,
        worklets: Option<&mut B::PaintWorkletInputs>,
    ) -> B::DiscardableImageMap;
    fn DetermineIfSolidColor(
        &self,
        rect: Rect,
        max_ops_to_analyze: i32,
        offsets: Option<&[usize]>,
    ) -> Option<Color4f>;
    fn EmitTraceSnapshot(
        &self,
        bounds: Option<Rect>,
        all_bounds: &std::collections::BTreeMap<usize, Rect>,
        scrolls: &std::collections::BTreeMap<
            ElementId,
            crate::cc::paint::display_item_list::RasterInducingScrollInfo,
        >,
    );
    fn ToTraceString(
        &self,
        bounds: Option<Rect>,
        all_bounds: &std::collections::BTreeMap<usize, Rect>,
        scrolls: &std::collections::BTreeMap<
            ElementId,
            crate::cc::paint::display_item_list::RasterInducingScrollInfo,
        >,
    ) -> String;
    fn TraceRasterBegin(&self, total_op_count: usize) -> bool;
    fn TraceRasterEnd(&self, rastered_op_count: usize);
}

pub trait DisplayItemListTypes: Sized + 'static {
    type PaintOp: DisplayPaintOp<Self>;
    type PaintOpBuffer: DisplayPaintOpBuffer<Self>;
    type Canvas: DisplayCanvas + ?Sized;
    type PlaybackParams<'a>;
    type PaintRecord;
    type SaveLayerBounds;
    type NodeInfo: DisplayNodeInfo;
    type DiscardableImageMap;
    type DecodingModeMap;
    type PaintWorkletInputs;

    fn PlaybackParams<'a>(scrolls: Option<&'a ScrollOffsetMap>) -> Self::PlaybackParams<'a>;
    fn MakeDrawScrollingContentsOp(
        scroll_element_id: ElementId,
        list: std::sync::Arc<crate::cc::paint::display_item_list::DisplayItemList<Self>>,
    ) -> Self::PaintOp;
}

pub fn DirectlyCompositedImageInfo(
    src_width: f32,
    src_height: f32,
    dst_width: f32,
    dst_height: f32,
    nearest_neighbor: bool,
) -> DirectlyCompositedImageInfo {
    DirectlyCompositedImageInfo {
        default_raster_scale: Vector2dF::new(src_width / dst_width, src_height / dst_height),
        nearest_neighbor,
    }
}
