//! Resident Paint dependencies of Blink ContentLayerClientImpl. These are the
//! actual Paint/backend objects, never snapshots of display lists or invalidator
//! state. All callbacks borrow the current client only for this call.
use super::{
    geometry::{Rect, RectF, Size, Vector2dF},
    input_types::ElementId,
    main_layer::LayerContext,
    paint_geometry::MaskGeometryMapper,
    pending_layer::PendingLayerColor,
    picture_layer::PictureDisplayItemList,
};
use crate::cc::layers::layer::Layer;
use std::sync::Arc;

pub use super::pending_layer::PendingLayerTypes as ContentLayerClientTypes;
pub trait CanvasChildPaintState {
    fn reference_box_offset(&self) -> Vector2dF;
}
pub trait ContentDisplayItemList<B: ContentLayerClientTypes>: PictureDisplayItemList<B> {
    fn StartPaint(&mut self);
    fn PushNoop(&mut self);
    fn EndPaintOfUnpaired(&mut self, rect: Rect);
    fn Finalize(&mut self);
    fn has_draw_text_ops(&self) -> bool;
    fn DeepCopyAsRecord(&self) -> B::PaintRecord;
    fn RasterForLogging(
        &self,
        canvas: &mut B::LoggingCanvas,
        offsets: &super::prioritized_tile::ScrollOffsetMap,
    );
}
pub trait ContentPaintOpBuffer<B: ContentLayerClientTypes> {
    fn PushTranslate(&mut self, x: f32, y: f32);
    fn AppendDisplayListBuffer(&mut self, list: &B::ContentDisplayItemList);
    fn ReleaseAsRecord(self) -> B::PaintRecord;
}
pub trait RasterInvalidationCallback {
    fn InvalidateRect(&mut self, rect: Rect);
}
pub trait RasterInvalidator<B: ContentLayerClientTypes> {
    fn LayerBounds(&self) -> Size;
    fn Generate(
        &mut self,
        chunks: &B::PaintChunkSubset,
        offset: Vector2dF,
        bounds: Size,
        state: &B::PropertyTreeState,
        callback: &mut dyn RasterInvalidationCallback,
    );
    fn GetTracking(&self) -> Option<&B::RasterInvalidationTracking>;
    fn GetTrackingMutable(&mut self) -> Option<&mut B::RasterInvalidationTracking>;
    fn SetTracksRasterInvalidations(&mut self, value: bool);
    fn UpdateForRasterInducingScroll(&mut self, chunks: &B::PaintChunkSubset);
    fn SetOldPaintArtifact(&mut self, artifact: Arc<B::PaintArtifact>);
    fn ApproximateUnsharedMemoryUsage(&self) -> usize;
}
pub trait RasterInvalidationTracking<B: ContentLayerClientTypes> {
    fn AsJSON(&self, json: &mut B::Json, detailed: bool);
}
pub trait ContentJson<B: ContentLayerClientTypes> {
    fn SetArray(&mut self, key: &str, array: B::JsonArray);
    fn SetPaintRecordLog(&mut self, key: &str, canvas: B::LoggingCanvas);
}
pub trait ContentLoggingCanvas<B: ContentLayerClientTypes> {}

/// External Paint/backend factories, features, value operations and diagnostics.
/// Conversion/layerization decisions are called directly in canonical modules.
pub trait ContentLayerClientServices<B: ContentLayerClientTypes>:
    PendingLayerColor + MaskGeometryMapper<TransformPaintPropertyNode = B::TransformNode>
{
    fn CreateDisplayItemList(&self) -> B::ContentDisplayItemList;
    fn CreateRasterInvalidator(&self) -> B::RasterInvalidator;
    fn RasterInducingScrollEnabled(&self) -> bool;
    fn PaintUnderInvalidationCheckingEnabled(&self) -> bool;
    fn ExpensiveDChecksEnabled(&self) -> bool;
    fn NewJsonArray(&self) -> B::JsonArray;
    fn AppendChunkDebugData(
        &self,
        array: &mut B::JsonArray,
        chunks: &B::PaintChunkSubset,
        index: usize,
    );
    fn CanvasChildPaintState(
        &self,
        state: &B::PropertyTreeState,
    ) -> Option<B::CanvasChildPaintState>;
    fn ScrollContentsRect(&self, state: &B::PropertyTreeState) -> Option<Rect>;
    fn RectContains(&self, outer: RectF, inner: RectF) -> bool;
    fn CreatePaintOpBuffer(&self) -> B::PaintOpBuffer;
    fn CreateCanvasChildPaintRecord(
        &self,
        state: B::CanvasChildPaintState,
        record: B::PaintRecord,
    ) -> B::CanvasChildPaintRecord;
    fn CreateLoggingCanvas(&self) -> B::LoggingCanvas;
    fn CurrentScrollOffset(
        &self,
        layer: &Layer<B>,
        element: ElementId,
        c: &LayerContext<'_, B>,
    ) -> super::geometry::PointF;
}
