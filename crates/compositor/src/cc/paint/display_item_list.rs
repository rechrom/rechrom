// Translation status: complete
// Copyright 2014 The Chromium Authors. BSD-style license in LICENSE.
// Source: cc/paint/display_item_list.h:1-43. Includes, declarations and the
// class overview map to this module and its typed Paint/raster adapter.
use crate::{
    adapter::{
        display_item_list::{
            DirectlyCompositedImageInfo as MakeDirectImageInfo, DisplayCanvas,
            DisplayItemListTypes, DisplayNodeInfo, DisplayPaintOp, DisplayPaintOpBuffer,
            DisplayPaintOpType, DisplayPlaybackItem,
        },
        geometry::Rect,
        input_types::ElementId,
        layer_tree_host_impl::ContentColorUsage,
        layer_tree_impl::Color4f,
        picture_layer_impl::DirectlyCompositedImageInfo,
        prioritized_tile::ScrollOffsetMap,
    },
    cc::base::rtree::RTree,
};
use std::{collections::BTreeMap, mem::size_of, sync::Arc};

// Source: cc/paint/display_item_list.h:44-50,215-287.
pub struct DisplayItemList<B: DisplayItemListTypes> {
    raster_inducing_scrolls_: BTreeMap<ElementId, RasterInducingScrollInfo>,
    rtree_: RTree<usize>,
    paint_op_buffer_: B::PaintOpBuffer,
    visual_rects_: Vec<Rect>,
    offsets_: Vec<usize>,
    paired_begin_stack_: Vec<PairedBeginInfo>,
    state_: RecordingState,
}
#[derive(Clone, Copy)]
struct PairedBeginInfo {
    first_index: usize,
    count: usize,
}
// Source: cc/paint/display_item_list.h:215-220.
pub struct RasterInducingScrollInfo {
    pub visual_rect: Rect,
    pub has_discardable_images: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordingState {
    NotPainting,
    Painting(usize),
    Finalized,
}

// Source: cc/paint/display_item_list.cc:77-83.
impl<B: DisplayItemListTypes> Default for DisplayItemList<B>
where
    B::PaintOpBuffer: Default,
{
    fn default() -> Self {
        Self::new_with_buffer(B::PaintOpBuffer::default())
    }
}
impl<B: DisplayItemListTypes> DisplayItemList<B> {
    // The injected Paint value backend is independent of all compositor
    // owners. Source allocation/state initialization stays in this constructor.
    pub fn new_with_buffer(buffer: B::PaintOpBuffer) -> Self {
        Self {
            raster_inducing_scrolls_: BTreeMap::new(),
            rtree_: RTree::new(),
            paint_op_buffer_: buffer,
            visual_rects_: Vec::with_capacity(1024),
            offsets_: Vec::with_capacity(1024),
            paired_begin_stack_: Vec::with_capacity(32),
            state_: RecordingState::NotPainting,
        }
    }
}
impl<B: DisplayItemListTypes> DisplayItemList<B> {
    pub fn new() -> Self
    where
        B::PaintOpBuffer: Default,
    {
        Self::default()
    }

    // Source: cc/paint/display_item_list.h:51-56;
    // cc/paint/display_item_list.cc:29-35,85-128.
    pub fn Raster<'canvas>(
        &self,
        canvas: &mut B::Canvas<'canvas>,
        scrolls: Option<&ScrollOffsetMap>,
    ) {
        let mut params = B::PlaybackParams(scrolls);
        self.RasterWithParams(canvas, &mut params);
    }
    pub fn RasterWithParams<'canvas>(
        &self,
        canvas: &mut B::Canvas<'canvas>,
        params: &mut B::PlaybackParams<'_>,
    ) {
        debug_assert_eq!(self.state_, RecordingState::Finalized);
        let tracing = self.paint_op_buffer_.TraceRasterBegin(self.TotalOpCount());
        let offsets = self.OffsetsOfOpsToRaster(canvas);
        if offsets.is_empty() {
            return;
        }
        self.paint_op_buffer_
            .Playback(canvas, params, true, Some(&offsets));
        if tracing {
            let rastered = self
                .paint_op_buffer_
                .playback_folding_iter(&offsets)
                .map(|op| 1 + op.AdditionalOpCount())
                .sum();
            self.paint_op_buffer_.TraceRasterEnd(rastered);
        }
    }
    pub fn OffsetsOfOpsToRaster<'canvas>(&self, canvas: &B::Canvas<'canvas>) -> Vec<usize> {
        let mut offsets = Vec::new();
        if let Some(rect) = canvas.LocalClipBounds() {
            self.rtree_.SearchTo(&rect, &mut offsets, None);
        }
        offsets
    }

    // Source: cc/paint/display_item_list.cc:37-70,130-170.
    fn IterateTextContent(
        buffer: &B::PaintOpBuffer,
        rect: Rect,
        yield_text: &mut impl FnMut(<B::NodeInfo as DisplayNodeInfo>::NodeId, Rect),
    ) {
        if !buffer.has_draw_text_ops() {
            return;
        }
        for op in buffer.iter() {
            match op.GetType() {
                DisplayPaintOpType::DrawTextBlob => yield_text(
                    op.TextNodeId().expect("source DrawTextBlobOp node id"),
                    rect,
                ),
                DisplayPaintOpType::DrawRecord => {
                    // Rust's Empty PaintRecord has no allocation; recursively
                    // iterating the source empty buffer performs zero visits.
                    if let Some(record) = op.RecordBuffer() {
                        Self::IterateTextContent(record, rect, yield_text);
                    }
                }
                _ => {}
            }
        }
    }
    fn IterateTextContentByOffsets(
        &self,
        offsets: &[usize],
        rects: &[Rect],
        mut yield_text: impl FnMut(<B::NodeInfo as DisplayNodeInfo>::NodeId, Rect),
    ) {
        debug_assert!(self.paint_op_buffer_.has_draw_text_ops());
        debug_assert_eq!(rects.len(), offsets.len());
        for (index, op) in self.paint_op_buffer_.iter_offsets(offsets).enumerate() {
            match op.GetType() {
                DisplayPaintOpType::DrawTextBlob => yield_text(
                    op.TextNodeId().expect("source DrawTextBlobOp node id"),
                    rects[index],
                ),
                DisplayPaintOpType::DrawRecord => {
                    if let Some(record) = op.RecordBuffer() {
                        Self::IterateTextContent(record, rects[index], &mut yield_text);
                    }
                }
                _ => {}
            }
        }
    }
    pub fn CaptureContent(&self, rect: Rect, content: &mut Vec<B::NodeInfo>) {
        debug_assert_eq!(self.state_, RecordingState::Finalized);
        if !self.paint_op_buffer_.has_draw_text_ops() {
            return;
        }
        let mut offsets = Vec::new();
        let mut rects = Vec::new();
        self.rtree_.SearchTo(&rect, &mut offsets, Some(&mut rects));
        self.IterateTextContentByOffsets(&offsets, &rects, |node_id, visual_rect| {
            if content.last().is_some_and(|last| last.node_id() == node_id) {
                content
                    .last_mut()
                    .unwrap()
                    .visual_rect_mut()
                    .Union(visual_rect);
            } else {
                content.push(B::NodeInfo::new(node_id, visual_rect));
            }
        });
    }
    pub fn AreaOfDrawText(&self, rect: Rect) -> f64 {
        if !self.paint_op_buffer_.has_draw_text_ops() {
            return 0.0;
        }
        let mut offsets = Vec::new();
        let mut rects = Vec::new();
        self.rtree_.SearchTo(&rect, &mut offsets, Some(&mut rects));
        debug_assert_eq!(offsets.len(), rects.len());
        self.paint_op_buffer_
            .iter_offsets(&offsets)
            .zip(rects)
            .fold(0.0, |area, (op, rect)| {
                area + if op.HasDrawTextOps() {
                    rect.width as f64 * rect.height as f64
                } else {
                    0.0
                }
            })
    }

    // Source: cc/paint/display_item_list.h:67-122.
    pub fn StartPaint(&mut self) {
        debug_assert_eq!(self.state_, RecordingState::NotPainting);
        self.state_ = RecordingState::Painting(self.paint_op_buffer_.size());
    }
    pub fn push(&mut self, op: B::PaintOp) -> usize {
        debug_assert!(matches!(self.state_, RecordingState::Painting(_)));
        let offset = self.paint_op_buffer_.next_op_offset();
        self.offsets_.push(offset);
        let returned = self.paint_op_buffer_.push(op);
        debug_assert_eq!(offset, returned);
        offset
    }
    pub fn UpdateSaveLayerBounds(&mut self, id: usize, bounds: B::SaveLayerBounds) {
        self.paint_op_buffer_.UpdateSaveLayerBounds(id, bounds)
    }
    pub fn EndPaintOfUnpaired(&mut self, visual_rect: Rect) {
        debug_assert!(matches!(self.state_, RecordingState::Painting(_)));
        self.state_ = RecordingState::NotPainting;
        self.visual_rects_
            .resize(self.paint_op_buffer_.size(), visual_rect);
        self.GrowCurrentBeginItemVisualRect(visual_rect);
    }
    pub fn EndPaintOfPairedBegin(&mut self) {
        let RecordingState::Painting(start) = self.state_ else {
            panic!("source EndPaintOfPairedBegin outside StartPaint")
        };
        debug_assert!(start < self.paint_op_buffer_.size());
        self.state_ = RecordingState::NotPainting;
        debug_assert!(self.visual_rects_.len() < self.paint_op_buffer_.size());
        let count = self.paint_op_buffer_.size() - self.visual_rects_.len();
        self.paired_begin_stack_.push(PairedBeginInfo {
            first_index: self.visual_rects_.len(),
            count,
        });
        self.visual_rects_
            .resize(self.paint_op_buffer_.size(), Rect::default());
    }

    // Source: cc/paint/display_item_list.cc:172-198.
    pub fn EndPaintOfPairedEnd(&mut self) {
        let RecordingState::Painting(start) = self.state_ else {
            panic!("source EndPaintOfPairedEnd outside StartPaint")
        };
        debug_assert!(start < self.paint_op_buffer_.size());
        self.state_ = RecordingState::NotPainting;
        let begin = self
            .paired_begin_stack_
            .last()
            .copied()
            .expect("source paired begin stack");
        debug_assert!(begin.count > 0);
        let visual_rect = self.visual_rects_[begin.first_index];
        for index in 1..begin.count {
            self.visual_rects_[begin.first_index + index] = visual_rect;
        }
        self.paired_begin_stack_.pop();
        self.visual_rects_
            .resize(self.paint_op_buffer_.size(), visual_rect);
        self.GrowCurrentBeginItemVisualRect(visual_rect);
    }

    // Source: cc/paint/display_item_list.h:124-139;
    // cc/paint/display_item_list.cc:200-228.
    pub fn Finalize(&mut self) {
        debug_assert_eq!(self.state_, RecordingState::NotPainting);
        debug_assert!(self.paired_begin_stack_.is_empty());
        debug_assert_eq!(self.visual_rects_.len(), self.offsets_.len());
        self.state_ = RecordingState::Finalized;
        self.rtree_.Build(
            self.visual_rects_.len(),
            |i| self.visual_rects_[i],
            |i| self.offsets_[i],
        );
        self.visual_rects_.clear();
        self.visual_rects_.shrink_to_fit();
        self.offsets_.clear();
        self.offsets_.shrink_to_fit();
        self.paired_begin_stack_.shrink_to_fit();
        self.paint_op_buffer_.ShrinkToFit();
    }
    pub fn FinalizeAndReleaseAsRecordForTesting(&mut self) -> B::PaintRecord {
        self.Finalize();
        self.paint_op_buffer_.ReleaseAsRecord()
    }
    pub fn paint_op_buffer(&self) -> &B::PaintOpBuffer {
        &self.paint_op_buffer_
    }
    pub fn SearchOpsByRect(&self, query: Rect, output: &mut Vec<usize>) {
        self.rtree_.SearchTo(&query, output, None)
    }

    // Source: cc/paint/display_item_list.h:141-227.
    pub fn GetDirectlyCompositedImageInfo(&self) -> Option<DirectlyCompositedImageInfo> {
        debug_assert_eq!(self.state_, RecordingState::Finalized);
        DirectlyCompositedImageInfoForPaintOpBuffer::<B>(&self.paint_op_buffer_)
    }
    pub fn num_slow_paths_up_to_min_for_MSAA(&self) -> i32 {
        self.paint_op_buffer_.num_slow_paths_up_to_min_for_MSAA()
    }
    pub fn has_non_aa_paint(&self) -> bool {
        self.paint_op_buffer_.has_non_aa_paint()
    }
    pub fn TotalOpCount(&self) -> usize {
        self.paint_op_buffer_.total_op_count()
    }
    pub fn BytesUsed(&self) -> usize {
        size_of::<Self>() + self.paint_op_buffer_.bytes_used()
    }
    pub fn OpBytesUsed(&self) -> usize {
        self.paint_op_buffer_.paint_ops_size()
    }
    pub fn VisualRectForTesting(&self, index: i32) -> Rect {
        self.visual_rects_[index as usize]
    }
    pub fn has_draw_ops(&self) -> bool {
        self.paint_op_buffer_.has_draw_ops()
    }
    pub fn has_draw_text_ops(&self) -> bool {
        self.paint_op_buffer_.has_draw_text_ops()
    }
    pub fn has_save_layer_ops(&self) -> bool {
        self.paint_op_buffer_.has_save_layer_ops()
    }
    pub fn has_save_layer_alpha_ops(&self) -> bool {
        self.paint_op_buffer_.has_save_layer_alpha_ops()
    }
    pub fn has_effects_preventing_lcd_text_for_save_layer_alpha(&self) -> bool {
        self.paint_op_buffer_
            .has_effects_preventing_lcd_text_for_save_layer_alpha()
    }
    pub fn has_discardable_images(&self) -> bool {
        self.paint_op_buffer_.has_discardable_images()
    }
    pub fn content_color_usage(&self) -> ContentColorUsage {
        self.paint_op_buffer_.content_color_usage()
    }
    pub fn num_paint_ops(&self) -> usize {
        self.paint_op_buffer_.size()
    }
    pub fn NeedsAdditionalInvalidationForLCDText(&self, old: &Self) -> bool {
        self.paint_op_buffer_
            .NeedsAdditionalInvalidationForLCDText(&old.paint_op_buffer_)
    }
    pub fn bounds(&self) -> Option<Rect> {
        self.rtree_.bounds()
    }
    pub fn raster_inducing_scrolls(&self) -> &BTreeMap<ElementId, RasterInducingScrollInfo> {
        &self.raster_inducing_scrolls_
    }

    // Source: cc/paint/display_item_list.h:229-283.
    fn GrowCurrentBeginItemVisualRect(&mut self, visual_rect: Rect) {
        if let Some(begin) = self.paired_begin_stack_.last() {
            self.visual_rects_[begin.first_index].Union(visual_rect);
        }
    }

    // Source: cc/paint/display_item_list.cc:230-329. SkPicture serialization,
    // trace sink ownership and DiscardableImageMap generation are true external
    // Paint/platform operations; cc supplies its actual RTree/map inputs.
    pub fn EmitTraceSnapshot(&self) {
        self.paint_op_buffer_.EmitTraceSnapshot(
            self.rtree_.bounds(),
            &self.rtree_.GetAllBoundsForTracing(),
            &self.raster_inducing_scrolls_,
        )
    }
    pub fn ToString(&self) -> String {
        self.paint_op_buffer_.ToTraceString(
            self.rtree_.bounds(),
            &self.rtree_.GetAllBoundsForTracing(),
            &self.raster_inducing_scrolls_,
        )
    }
    pub fn GenerateDiscardableImageMap(
        &self,
        scrolls: &ScrollOffsetMap,
        decoding: Option<&mut B::DecodingModeMap>,
        worklets: Option<&mut B::PaintWorkletInputs>,
    ) -> B::DiscardableImageMap {
        debug_assert_eq!(self.state_, RecordingState::Finalized);
        self.paint_op_buffer_.GenerateDiscardableImageMap(
            self.bounds().unwrap_or(Rect::new(0, 0, i32::MAX, i32::MAX)),
            scrolls,
            decoding,
            worklets,
        )
    }

    // Source: cc/paint/display_item_list.cc:331-353.
    pub fn GetColorIfSolidInRect(&self, rect: Rect, color: &mut Color4f, max_ops: i32) -> bool {
        debug_assert_eq!(self.state_, RecordingState::Finalized);
        let mut offsets = Vec::new();
        let offsets = if self.rtree_.has_valid_bounds()
            && !rect.Contains(self.bounds().expect("valid RTree bounds"))
        {
            self.rtree_.SearchTo(&rect, &mut offsets, None);
            Some(offsets.as_slice())
        } else {
            None
        };
        if let Some(solid) = self
            .paint_op_buffer_
            .DetermineIfSolidColor(rect, max_ops, offsets)
        {
            *color = solid;
            true
        } else {
            false
        }
    }

    // Source: cc/paint/display_item_list.cc:452-472. The source range-for over
    // std::move(map) does not move/clear the container; it visits the same map.
    pub fn PushDrawScrollingContentsOp(
        &mut self,
        scroll_element_id: ElementId,
        display_item_list: Arc<Self>,
        visual_rect: Rect,
    ) {
        self.StartPaint();
        self.push(B::MakeDrawScrollingContentsOp(
            scroll_element_id,
            Arc::clone(&display_item_list),
        ));
        for (&nested_id, info) in &display_item_list.raster_inducing_scrolls_ {
            self.raster_inducing_scrolls_.insert(
                nested_id,
                RasterInducingScrollInfo {
                    visual_rect,
                    has_discardable_images: info.has_discardable_images,
                },
            );
        }
        self.raster_inducing_scrolls_.insert(
            scroll_element_id,
            RasterInducingScrollInfo {
                visual_rect,
                has_discardable_images: display_item_list.has_discardable_images(),
            },
        );
        self.EndPaintOfUnpaired(visual_rect);
    }
}

// Source: cc/paint/display_item_list.cc:355-450.
fn DirectlyCompositedImageInfoForPaintOpBuffer<B: DisplayItemListTypes>(
    buffer: &B::PaintOpBuffer,
) -> Option<DirectlyCompositedImageInfo> {
    const K_MAX_DRAW_IMAGE_OPS: usize = 5;
    if buffer.size() > K_MAX_DRAW_IMAGE_OPS {
        return None;
    }
    let mut transpose = false;
    let mut result = None;
    for op in buffer.iter() {
        match op.GetType() {
            DisplayPaintOpType::Save
            | DisplayPaintOpType::Restore
            | DisplayPaintOpType::Translate => {}
            DisplayPaintOpType::Concat => {
                if transpose || result.is_some() || !op.ConcatPreserves2DAxisAlignment() {
                    return None;
                }
                transpose = op.ConcatTransposesAxes();
            }
            DisplayPaintOpType::DrawImageRect => {
                if result.is_some() {
                    return None;
                }
                let (src_w, src_h, dst_w, dst_h) = op.DrawImageRectGeometry()?;
                if src_w <= 0.0 || src_h <= 0.0 || dst_w <= 0.0 || dst_h <= 0.0 {
                    return None;
                }
                result = Some(MakeDirectImageInfo(
                    src_w,
                    src_h,
                    dst_w,
                    dst_h,
                    op.DrawImageUsesNearestNeighbor(),
                ));
            }
            DisplayPaintOpType::DrawRecord => {
                if result.is_some() {
                    return None;
                }
                result = DirectlyCompositedImageInfoForPaintOpBuffer::<B>(op.RecordBuffer()?);
                result.as_ref()?;
            }
            _ => return None,
        }
    }
    if let Some(info) = &mut result {
        if transpose {
            info.default_raster_scale = crate::adapter::geometry::Vector2dF::new(
                info.default_raster_scale.y(),
                info.default_raster_scale.x(),
            );
        }
    }
    result
}

// Source: cc/paint/display_item_list.h:141-213. These are direct virtual-free
// projections of the same resident list used by RecordingSource/HostImpl.
impl<B: DisplayItemListTypes> crate::adapter::picture_layer_impl::PictureDisplayItemList
    for DisplayItemList<B>
{
    fn has_draw_text_ops(&self) -> bool {
        self.has_draw_text_ops()
    }
    fn content_color_usage(&self) -> ContentColorUsage {
        self.content_color_usage()
    }
    fn raster_inducing_scrolls(&self) -> &BTreeMap<ElementId, RasterInducingScrollInfo> {
        self.raster_inducing_scrolls()
    }
}
impl<B: DisplayItemListTypes> crate::adapter::layer_tree_host_impl::HostDisplayItemList
    for DisplayItemList<B>
{
    fn num_slow_paths_up_to_min_for_MSAA(&self) -> i32 {
        self.num_slow_paths_up_to_min_for_MSAA()
    }
    fn has_non_aa_paint(&self) -> bool {
        self.has_non_aa_paint()
    }
}
