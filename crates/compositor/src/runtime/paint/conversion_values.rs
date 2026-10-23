//! Native source metadata and record leases used by the conversion owner.
use super::*;
use crate::adapter::display_item_list::DisplayPaintOpBuffer;
use crate::{
    adapter::{
        geometry::{Point, Rect, RectF},
        input_types::ElementId,
        paint_chunks_to_cc_layer::*,
    },
    runtime::types::CpuSkiaBackend,
};
fn rect(p: paint::paint_engine::PaintRect) -> Rect {
    crate::adapter::geometry::ToEnclosingRect(RectF::new(
        p.x as f32,
        p.y as f32,
        p.width as f32,
        p.height as f32,
    ))
}
fn metadata(c: &PaintChunk) -> &paint::paint_engine::PaintChunkCompositorData {
    c.compositor
        .as_deref()
        .expect("validated native derived data")
}
impl<B> ConversionPaintChunk<B> for PaintChunk
where
    B: PaintChunksTypes<
        HitTestData = paint::paint_engine::PaintHitTestData,
        RegionCaptureData = Vec<([u64; 2], paint::paint_engine::PaintRect)>,
        TrackedData = Vec<paint::paint_engine::PaintTrackedElementRect>,
        SelectionData = paint::paint_engine::PaintLayerSelectionData,
    >,
{
    fn effectively_invisible(&self) -> bool {
        self.effectively_invisible
    }
    fn drawable_bounds(&self) -> Rect {
        rect(self.drawable_bounds)
    }
    fn bounds_rect(&self) -> Rect {
        rect(self.bounds)
    }
    fn IsScrollHitTest(&self) -> bool {
        self.id.r#type == paint::display_item_id::DisplayItemIdType::kScrollHitTest
    }
    fn IsScrollbarHitTest(&self) -> bool {
        self.id.r#type == paint::display_item_id::DisplayItemIdType::kScrollbarHitTest
    }
    fn IsResizerScrollHitTest(&self) -> bool {
        self.id.r#type == paint::display_item_id::DisplayItemIdType::kResizerScrollHitTest
    }
    fn hit_test_data(&self) -> Option<&B::HitTestData> {
        metadata(self).hit_test_data.as_deref()
    }
    fn region_capture_data(&self) -> Option<&B::RegionCaptureData> {
        metadata(self).region_capture_data.as_deref()
    }
    fn tracked_element_rects(&self) -> Option<&B::TrackedData> {
        metadata(self).tracked_element_rects.as_deref()
    }
    fn layer_selection_data(&self) -> Option<&B::SelectionData> {
        metadata(self).layer_selection_data.as_deref()
    }
}
impl<B: PaintChunksTypes<PaintArtifact = PaintArtifact, DisplayItem = DisplayItem>>
    ConversionDisplayItem<B> for DisplayItem
{
    fn VisualRect(&self) -> Rect {
        rect(self.visual_rect)
    }
    fn IsDrawing(&self) -> bool {
        matches!(
            self.kind,
            RecordedDisplayItemKind::Drawing | RecordedDisplayItemKind::Scrollbar
        )
    }
    fn IsScrollbar(&self) -> bool {
        matches!(self.kind, RecordedDisplayItemKind::Scrollbar) && self.scrollbar.is_some()
    }
    fn GetPaintRecord(
        &self,
        a: &Arc<PaintArtifact>,
        s: &dyn PaintChunksServices<B>,
    ) -> B::PaintRecord {
        s.NativePaintRecord(a, self)
    }
    fn PaintScrollbar(
        &self,
        a: &Arc<PaintArtifact>,
        s: &dyn PaintChunksServices<B>,
    ) -> B::PaintRecord {
        s.NativePaintRecord(a, self)
    }
    fn ScrollbarElementId(&self) -> ElementId {
        ElementId::new(
            self.scrollbar
                .as_ref()
                .expect("ScrollbarDisplayItem retains its compositor identity")
                .element_id
                .GetInternalValue(),
        )
    }
}
impl<R, B> ConversionHitTestData<B> for paint::paint_engine::PaintHitTestData
where
    R: CpuSkiaBackend,
    B: PaintChunksTypes<TransformNode = TransformNode<R>>,
{
    fn touch_action_rects(
        &self,
    ) -> impl Iterator<Item = (Rect, crate::cc::input::touch_action::TouchAction)> {
        self.touch_action_rects.iter().map(|(r, bits)| {
            use crate::cc::input::touch_action::TouchAction as T;
            let flags = [
                T::kPanLeft,
                T::kPanRight,
                T::kPanUp,
                T::kPanDown,
                T::kPinchZoom,
                T::kDoubleTapZoom,
                T::kInternalPanXScrolls,
                T::kInternalNotWritable,
            ];
            let mut value = T::kNone;
            for (i, flag) in flags.into_iter().enumerate() {
                if bits & (1 << i) != 0 {
                    value |= flag;
                }
            }
            (rect(*r), value)
        })
    }
    fn wheel_event_rects(&self) -> impl Iterator<Item = Rect> {
        self.wheel_event_rects.iter().copied().map(rect)
    }
    fn xr_regions(&self) -> impl Iterator<Item = ElementId> {
        self.xr_regions
            .iter()
            .map(|id| ElementId::new(id.GetInternalValue()))
    }
    fn scroll_hit_test_rect(&self) -> Rect {
        rect(self.scroll_hit_test_rect)
    }
    fn scroll_translation(&self) -> Option<TransformNode<R>> {
        self.scroll_translation
            .as_ref()
            .map(|t| TransformNode::from_arc(t.clone()))
    }
    fn scrolling_contents_cull_rect(&self) -> Rect {
        rect(self.scrolling_contents_cull_rect)
    }
}
impl<B: PaintChunksTypes<CaptureId = crate::runtime::metadata::RegionCaptureCropId>>
    ConversionRegionCaptureData<B> for Vec<([u64; 2], paint::paint_engine::PaintRect)>
{
    fn iter(&self) -> impl Iterator<Item = (B::CaptureId, Rect)> {
        self.as_slice()
            .iter()
            .map(|(id, r)| (crate::runtime::metadata::RegionCaptureCropId(*id), rect(*r)))
    }
}
impl ConversionCaptureBounds<crate::runtime::metadata::RegionCaptureCropId>
    for crate::runtime::metadata::RegionCaptureBounds
{
    fn Set(&mut self, id: crate::runtime::metadata::RegionCaptureCropId, r: Rect) {
        self.0.insert(id, r);
    }
}
fn selection(s: paint::paint_engine::PaintSelectionBound) -> PaintedSelectionBound {
    use crate::adapter::layer_tree_impl::SelectionBoundType as T;
    use paint::paint_engine::PaintSelectionBoundType::*;
    PaintedSelectionBound {
        r#type: match s.bound_type {
            Left => T::LEFT,
            Right => T::RIGHT,
            Center => T::CENTER,
            Empty => T::EMPTY,
            Hidden => T::HIDDEN,
        },
        edge_start: Point::new(s.edge_start[0], s.edge_start[1]),
        edge_end: Point::new(s.edge_end[0], s.edge_end[1]),
    }
}
impl ConversionSelectionData for paint::paint_engine::PaintLayerSelectionData {
    fn start(&self) -> Option<PaintedSelectionBound> {
        self.start.map(selection)
    }
    fn end(&self) -> Option<PaintedSelectionBound> {
        self.end.map(selection)
    }
    fn any_selection_was_painted(&self) -> bool {
        self.any_selection_was_painted
    }
}
fn token(t: paint::paint_engine::PaintFrameToken) -> crate::runtime::metadata::FrameToken {
    match t {
        paint::paint_engine::PaintFrameToken::Local(v) => {
            crate::runtime::metadata::FrameToken::Local(v)
        }
        paint::paint_engine::PaintFrameToken::Remote(v) => {
            crate::runtime::metadata::FrameToken::Remote(v)
        }
    }
}
impl<B> ConversionTrackedData<B> for Vec<paint::paint_engine::PaintTrackedElementRect>
where
    B: PaintChunksTypes<
        TrackingFeature = crate::runtime::metadata::TrackedElementFeature,
        TrackedId = [u64; 2],
        FrameToken = Option<crate::runtime::metadata::FrameToken>,
    >,
{
    fn iter(
        &self,
    ) -> impl Iterator<Item = (B::TrackingFeature, TrackedRect<B::TrackedId, B::FrameToken>)> {
        self.as_slice().iter().map(|r| {
            (
                crate::runtime::metadata::TrackedElementFeature(r.feature),
                TrackedRect {
                    bounds: rect(r.rect),
                    id: r.id,
                    should_add_to_compositor_frame_metadata: r
                        .should_add_to_compositor_frame_metadata,
                    should_exclude_fixed_and_sticky_occlusions: r
                        .should_exclude_fixed_and_sticky_occlusions,
                    frame_token: r.frame_token.map(token),
                    parent_frame_token: r.parent_frame_token.map(token),
                },
            )
        })
    }
}
impl<B> ConversionTrackedElementRects<B> for crate::runtime::metadata::TrackedElementRects
where
    B: PaintChunksTypes<
        TrackingFeature = crate::runtime::metadata::TrackedElementFeature,
        TrackedId = [u64; 2],
        FrameToken = Option<crate::runtime::metadata::FrameToken>,
    >,
{
    fn Push(&mut self, f: B::TrackingFeature, r: TrackedRect<B::TrackedId, B::FrameToken>) {
        self.0
            .entry(f)
            .or_default()
            .push(crate::runtime::metadata::TrackedElementRect {
                id: r.id,
                visible_bounds: r.bounds,
                should_add_to_compositor_frame_metadata: r.should_add_to_compositor_frame_metadata,
                should_exclude_fixed_and_sticky_occlusions: r
                    .should_exclude_fixed_and_sticky_occlusions,
                frame_token: r.frame_token,
                parent_frame_token: r.parent_frame_token.map(|t| match t {
                    crate::runtime::metadata::FrameToken::Local(t) => t,
                    crate::runtime::metadata::FrameToken::Remote(_) => {
                        unreachable!("PaintInput rejects remote parent frame tokens")
                    }
                }),
            });
    }
}
pub struct CpuConversionRecorder<R: CpuSkiaBackend> {
    pub buffer: CpuPaintOpBuffer<R>,
}
impl<R, B> ConversionPaintRecorder<B> for CpuConversionRecorder<R>
where
    R: CpuSkiaBackend,
    B: PaintChunksTypes<
        PaintRecord = CpuPaintRecord<R>,
        UnderInvalidationPicture = CpuPaintRecord<R>,
    >,
{
    fn beginRecording(&mut self) {
        assert!(self.buffer.size() == 0);
    }
    fn DrawPicture(&mut self, r: CpuPaintRecord<R>) {
        self.buffer.push(CpuPaintOp::DrawRecord(r));
    }
    fn finishRecordingAsPicture(mut self) -> CpuPaintRecord<R> {
        crate::adapter::display_item_list::DisplayPaintOpBuffer::ReleaseAsRecord(&mut self.buffer)
    }
}
