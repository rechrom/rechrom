//! Direct native chunk/item projections. Missing producer capabilities are
//! rejected by PaintInput; no empty metadata or fabricated Layer reaches PAC.
use super::*;
use crate::{
    adapter::{
        geometry::{Rect, RectF, Vector2dF},
        layer_tree_impl::Color4f,
        pending_layer::*,
    },
    runtime::types::{CpuCompositorTypes, CpuSkiaBackend},
};
fn rect(r: paint::paint_engine::PaintRect) -> RectF {
    RectF::new(r.x as f32, r.y as f32, r.width as f32, r.height as f32)
}
fn metadata(chunk: &PaintChunk) -> &paint::paint_engine::PaintChunkCompositorData {
    chunk
        .compositor
        .as_deref()
        .expect("PaintInput validates native chunk derived data")
}
pub trait CpuLoggingCanvasValue<R: CpuSkiaBackend>: CpuPaintCanvas<R> {
    fn IntoLog(self: Box<Self>) -> serde_json::Value;
}
pub struct CpuLoggingCanvas<R: CpuSkiaBackend>(pub Box<dyn CpuLoggingCanvasValue<R>>);
impl<R: CpuSkiaBackend> crate::adapter::display_item_list::DisplayCanvas for CpuLoggingCanvas<R> {
    fn LocalClipBounds(&self) -> Option<Rect> {
        self.0.LocalClipBounds()
    }
}
impl<R: CpuSkiaBackend> CpuPaintCanvas<R> for CpuLoggingCanvas<R> {
    fn Playback(
        &mut self,
        b: &CpuPaintOpBuffer<R>,
        p: &mut CpuPlaybackParams<'_, R>,
        local: bool,
        offsets: Option<&[usize]>,
    ) {
        self.0.Playback(b, p, local, offsets)
    }
}
impl<R, B> crate::adapter::content_layer_client_impl::ContentLoggingCanvas<B>
    for CpuLoggingCanvas<R>
where
    R: CpuSkiaBackend,
    B: PendingLayerTypes<LoggingCanvas = Self>,
{
}
impl<R, B> crate::adapter::content_layer_client_impl::ContentJson<B> for serde_json::Value
where
    R: CpuSkiaBackend,
    B: PendingLayerTypes<JsonArray = Vec<serde_json::Value>, LoggingCanvas = CpuLoggingCanvas<R>>,
{
    fn SetArray(&mut self, key: &str, array: Vec<Self>) {
        self[key] = Self::Array(array)
    }
    fn SetPaintRecordLog(&mut self, key: &str, canvas: CpuLoggingCanvas<R>) {
        self[key] = canvas.0.IntoLog()
    }
}
#[derive(Clone)]
pub struct CpuCanvasChildPaintRecord<R: CpuSkiaBackend> {
    pub state: Arc<native::PaintCanvasChildState>,
    pub record: CpuPaintRecord<R>,
}
impl crate::adapter::content_layer_client_impl::CanvasChildPaintState
    for Arc<native::PaintCanvasChildState>
{
    fn reference_box_offset(&self) -> Vector2dF {
        Vector2dF::new(
            self.reference_box_offset.x as f32,
            self.reference_box_offset.y as f32,
        )
    }
}
impl<R, B> PendingPaintChunk<B> for PaintChunk
where
    R: CpuSkiaBackend,
    B: PendingLayerTypes<
        PropertyTreeState = PropertyTreeState<R>,
        TransformNode = TransformNode<R>,
        ClientId = usize,
        PaintArtifact = PaintArtifact,
        DisplayItem = DisplayItem,
    >,
{
    fn HasHitTestData(&self) -> bool {
        metadata(self).hit_test_data.is_some()
    }
    fn PropertiesUnalias(&self) -> PropertyTreeState<R> {
        PropertyTreeState::from_native(&self.properties)
    }
    fn client_id(&self) -> usize {
        self.id.client_id as usize
    }
    fn bounds(&self) -> RectF {
        rect(self.bounds)
    }
    fn rect_known_to_be_opaque(&self) -> RectF {
        rect(self.rect_known_to_be_opaque)
    }
    fn background_color(&self) -> PendingChunkBackgroundColor {
        let m = metadata(self);
        PendingChunkBackgroundColor {
            color: Color4f {
                red: m.background_color.red,
                green: m.background_color.green,
                blue: m.background_color.blue,
                alpha: m.background_color.alpha,
            },
            area: m.background_area,
            is_solid_color: m.background_is_solid_color,
        }
    }
    fn hit_test_opaqueness(&self) -> crate::cc::input::hit_test_opaqueness::HitTestOpaqueness {
        use crate::cc::input::hit_test_opaqueness::HitTestOpaqueness as O;
        use paint::paint_engine::PaintHitTestOpaqueness::*;
        match metadata(self).hit_test_opaqueness {
            Transparent => O::kTransparent,
            Mixed => O::kMixed,
            Opaque => O::kOpaque,
        }
    }
    fn has_text(&self) -> bool {
        metadata(self).has_text
    }
    fn text_known_to_be_on_opaque_background(&self) -> bool {
        metadata(self).text_known_to_be_on_opaque_background
    }
    fn DrawsContent(&self) -> bool {
        !self.effectively_invisible && !self.drawable_bounds.is_empty()
    }
    fn display_item_count(&self) -> usize {
        self.size() as usize
    }
    fn non_composited_scroll_translation(&self) -> Option<TransformNode<R>> {
        metadata(self)
            .hit_test_data
            .as_ref()
            .and_then(|d| d.scroll_translation.as_ref())
            .map(|n| TransformNode::from_arc(n.clone()))
    }
    fn Matches(&self, other: &Self) -> bool {
        self.matches(other)
    }
    fn first_display_item_is_foreign_layer_video(&self, a: &PaintArtifact) -> bool {
        a.display_items[self.begin_index as usize].id.r#type
            == paint::display_item_id::DisplayItemIdType::kForeignLayerVideo
    }
    fn FirstDisplayItem<'a>(&self, a: &'a PaintArtifact) -> &'a DisplayItem {
        &a.display_items[self.begin_index as usize]
    }
    fn is_moved_from_cached_subsequence(&self) -> bool {
        self.is_moved_from_cached_subsequence
    }
}
impl<B> PendingPaintArtifact<B> for PaintArtifact
where
    B: PendingLayerTypes<
        PaintChunk = PaintChunk,
        ClientId = usize,
        DomNodeId = crate::runtime::types::NodeId,
    >,
{
    fn PaintChunk(&self, index: usize) -> Option<&PaintChunk> {
        self.chunks.get(index)
    }
    fn ClientDebugName(&self, client: usize) -> String {
        self.client_debug_names
            .get(&client)
            .map(|v| v.to_string())
            .unwrap_or_default()
    }
    fn ClientOwnerNodeId(&self, client: usize) -> crate::runtime::types::NodeId {
        crate::runtime::types::NodeId(
            self.client_owner_node_ids
                .get(&client)
                .copied()
                .unwrap_or(0),
        )
    }
    fn PaintChunkCount(&self) -> usize {
        self.chunks.len()
    }
}
impl<B: PendingLayerTypes> PendingDisplayItem<B> for DisplayItem {
    fn ForeignLayerVisualRect(&self) -> RectF {
        rect(self.visual_rect)
    }
    fn ForeignLayer(&self) -> crate::cc::layers::layer::LayerRef<B> {
        unreachable!("PaintInput rejects missing foreign-layer native owner")
    }
    fn IsScrollbar(&self) -> bool {
        false
    }
    fn CreateOrReuseScrollbarLayer(
        &self,
        _old: Option<crate::cc::layers::layer::LayerRef<B>>,
        _offset: Vector2dF,
    ) -> crate::cc::layers::layer::LayerRef<B> {
        unreachable!("PaintInput rejects missing scrollbar native layer owner")
    }
}
