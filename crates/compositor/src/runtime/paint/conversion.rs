//! Storage-only forwarding from canonical Blink conversion commands into the
//! actual cc DisplayItemList operation buffer. All conversion decisions remain
//! in PaintChunksToCcLayer; no producer records or trees are copied here.
use super::*;
use crate::adapter::{
    content_layer_client_impl::{ContentDisplayItemList, ContentPaintOpBuffer},
    display_item_list::DisplayPaintOpBuffer,
    paint_chunks_to_cc_layer::{
        ConversionDisplayItemList, ConversionOp, ConversionPaintOpBuffer, PaintChunksTypes,
    },
    paint_geometry::ChunkGeometryMapper,
};
// Moves one conversion command into the cc op storage (CpuPaintOp is cc/paint
// commit vocabulary, so this is a free function of the conversion binding).
fn CpuPaintOpFromConversion<R: crate::runtime::types::CpuSkiaBackend, B>(
    op: ConversionOp<B>,
) -> CpuPaintOp<R>
where
    B: PaintChunksTypes<
        PaintRecord = CpuPaintRecord<R>,
        PaintFlags = CpuPaintFlags<R>,
        PaintFilter = CpuPaintFilter,
        RoundedRect = crate::adapter::geometry::RRectF,
        PaintPath = std::sync::Arc<paint::paint_clip_path::PaintClipPath>,
    >,
    B::ChunkMapper: ChunkGeometryMapper<Transform = crate::runtime::geometry::Transform<R>>,
{
    match op {
        ConversionOp::Save => CpuPaintOp::Save,
        ConversionOp::Restore => CpuPaintOp::Restore,
        ConversionOp::Translate(x, y) => CpuPaintOp::Translate(x, y),
        ConversionOp::Concat(t) => CpuPaintOp::Concat(t),
        ConversionOp::ClipRect {
            rect,
            difference,
            antialias,
        } => CpuPaintOp::ClipRect {
            rect,
            difference,
            antialias,
        },
        ConversionOp::ClipRRect(r, a) => CpuPaintOp::ClipRRect(r, a),
        ConversionOp::ClipPath {
            path,
            difference,
            antialias,
        } => CpuPaintOp::ClipPath {
            path: std::sync::Arc::new(CpuClipPath::<R>::FromPaintPath(&path)),
            difference,
            antialias,
        },
        ConversionOp::SaveLayer(flags) => CpuPaintOp::SaveLayer {
            bounds: None,
            flags,
        },
        ConversionOp::SaveLayerAlpha(alpha) => CpuPaintOp::SaveLayerAlpha {
            bounds: None,
            alpha,
        },
        ConversionOp::SaveLayerFilters {
            bounds,
            filters,
            backdrop_filter,
            flags,
        } => CpuPaintOp::SaveLayerFilters {
            bounds: Some(bounds),
            filters,
            backdrop_filter,
            flags,
        },
        ConversionOp::DrawTransparentColorSrc => CpuPaintOp::DrawTransparentColorSrc,
        ConversionOp::DrawRecord(record) => CpuPaintOp::DrawRecord(record),
    }
}
impl<R: crate::runtime::types::CpuSkiaBackend, B> ContentPaintOpBuffer<B> for CpuPaintOpBuffer<R>
where
    B: crate::adapter::content_layer_client_impl::ContentLayerClientTypes<
        PaintRecord = CpuPaintRecord<R>,
        ContentDisplayItemList = super::ContentDisplayItemList<R>,
    >,
{
    fn PushTranslate(&mut self, x: f32, y: f32) {
        self.push(CpuPaintOp::Translate(x, y));
    }
    fn AppendDisplayListBuffer(&mut self, list: &super::ContentDisplayItemList<R>) {
        self.AppendBuffer(list.paint_op_buffer());
    }
    fn ReleaseAsRecord(mut self) -> CpuPaintRecord<R> {
        DisplayPaintOpBuffer::ReleaseAsRecord(&mut self)
    }
}
impl<R: crate::runtime::types::CpuSkiaBackend, B> ContentDisplayItemList<B>
    for super::ContentDisplayItemList<R>
where
    B: crate::adapter::content_layer_client_impl::ContentLayerClientTypes<
            PaintRecord = CpuPaintRecord<R>,
        > + crate::adapter::picture_layer::PictureLayerTypes<
            Recording = CpuRecordingTypes<R>,
            NodeInfo = NodeInfo,
            PictureRecorder = CpuPictureRecorder<R>,
        >,
    B::LoggingCanvas: CpuPaintCanvas<R> + 'static,
    CpuRecordingTypes<R>:crate::adapter::recording_source::RecordingSourceTypes<DirectlyCompositedImageInfo=crate::adapter::picture_layer_impl::DirectlyCompositedImageInfo>,
{
    fn StartPaint(&mut self) {
        Self::StartPaint(self)
    }
    fn PushNoop(&mut self) {
        self.push(CpuPaintOp::Noop);
    }
    fn EndPaintOfUnpaired(&mut self, rect: crate::adapter::geometry::Rect) {
        Self::EndPaintOfUnpaired(self, rect)
    }
    fn Finalize(&mut self) {
        Self::Finalize(self)
    }
    fn has_draw_text_ops(&self) -> bool {
        Self::has_draw_text_ops(self)
    }
    fn DeepCopyAsRecord(&self) -> CpuPaintRecord<R> {
        self.paint_op_buffer().DeepCopyAsRecord()
    }
    fn RasterForLogging(
        &self,
        canvas: &mut B::LoggingCanvas,
        offsets: &crate::adapter::prioritized_tile::ScrollOffsetMap,
    ) {
        Self::Raster(self, canvas, Some(offsets))
    }
}
impl<R: crate::runtime::types::CpuSkiaBackend, B> ConversionPaintOpBuffer<B> for CpuPaintOpBuffer<R>
where
    B: PaintChunksTypes<
        PaintRecord = CpuPaintRecord<R>,
        PaintFlags = CpuPaintFlags<R>,
        PaintFilter = CpuPaintFilter,
        RoundedRect = crate::adapter::geometry::RRectF,
        PaintPath = std::sync::Arc<paint::paint_clip_path::PaintClipPath>,
    >,
    B::ChunkMapper: ChunkGeometryMapper<Transform = crate::runtime::geometry::Transform<R>>,
{
    fn next_op_offset(&self) -> usize {
        DisplayPaintOpBuffer::next_op_offset(self)
    }
    fn Push(&mut self, op: ConversionOp<B>) {
        self.push(CpuPaintOpFromConversion(op));
    }
    fn UpdateSaveLayerBounds(&mut self, id: usize, rect: crate::adapter::geometry::RectF) {
        DisplayPaintOpBuffer::UpdateSaveLayerBounds(self, id, rect)
    }
}
impl<R: crate::runtime::types::CpuSkiaBackend, B> ConversionDisplayItemList<B>
    for super::ContentDisplayItemList<R>
where
    B: PaintChunksTypes<
            PaintRecord = CpuPaintRecord<R>,
            ContentDisplayItemList = Self,
            PaintFlags = CpuPaintFlags<R>,
            PaintFilter = CpuPaintFilter,
            RoundedRect = crate::adapter::geometry::RRectF,
            PaintPath = std::sync::Arc<paint::paint_clip_path::PaintClipPath>,
        > + crate::adapter::picture_layer::PictureLayerTypes<
            Recording = CpuRecordingTypes<R>,
            NodeInfo = NodeInfo,
            PictureRecorder = CpuPictureRecorder<R>,
        >,
    B::ChunkMapper: ChunkGeometryMapper<Transform = crate::runtime::geometry::Transform<R>>,
    B::LoggingCanvas: CpuPaintCanvas<R> + 'static,
    CpuRecordingTypes<R>:crate::adapter::recording_source::RecordingSourceTypes<DirectlyCompositedImageInfo=crate::adapter::picture_layer_impl::DirectlyCompositedImageInfo>,
{
    fn Push(&mut self, op: ConversionOp<B>) -> usize {
        self.push(CpuPaintOpFromConversion(op))
    }
    fn EndPaintOfPairedBegin(&mut self) {
        Self::EndPaintOfPairedBegin(self)
    }
    fn EndPaintOfPairedEnd(&mut self) {
        Self::EndPaintOfPairedEnd(self)
    }
    fn UpdateSaveLayerBounds(&mut self, id: usize, rect: crate::adapter::geometry::RectF) {
        Self::UpdateSaveLayerBounds(self, id, rect)
    }
    fn PushDrawScrollingContentsOp(
        &mut self,
        id: crate::adapter::input_types::ElementId,
        list: std::sync::Arc<Self>,
        rect: crate::adapter::geometry::Rect,
    ) {
        Self::PushDrawScrollingContentsOp(self, id, list, rect)
    }
}
