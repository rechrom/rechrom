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

/// Concrete resident cc layer owner carried through Paint's type-erased
/// `scoped_refptr` boundary. Bounds are captured from the same layer at record
/// time, exactly when Blink's ForeignLayerDisplayItem constructor reads them.
#[derive(Clone)]
pub struct CpuForeignLayer<R: CpuSkiaBackend> {
    layer: crate::cc::layers::layer::LayerRef<CpuCompositorTypes<R>>,
    bounds: Arc<std::sync::RwLock<paint::paint_engine::ForeignLayerSize>>,
}

impl<R: CpuSkiaBackend> CpuForeignLayer<R> {
    pub fn new(
        layer: crate::cc::layers::layer::LayerRef<CpuCompositorTypes<R>>,
        c: &crate::adapter::main_layer::LayerContext<'_, CpuCompositorTypes<R>>,
    ) -> Self {
        let bounds = *layer.bounds(c);
        Self {
            layer,
            bounds: Arc::new(std::sync::RwLock::new(
                paint::paint_engine::ForeignLayerSize {
                    width: bounds.width() as f64,
                    height: bounds.height() as f64,
                },
            )),
        }
    }

    pub fn Layer(&self) -> &crate::cc::layers::layer::LayerRef<CpuCompositorTypes<R>> {
        &self.layer
    }
}

impl<R: CpuSkiaBackend> paint::paint_engine::ForeignLayer for CpuForeignLayer<R> {
    fn LayerId(&self) -> i32 {
        self.layer.id()
    }

    fn Bounds(&self) -> paint::paint_engine::ForeignLayerSize {
        *self
            .bounds
            .read()
            .unwrap_or_else(|error| error.into_inner())
    }

    fn SetBounds(&self, bounds: paint::paint_engine::ForeignLayerSize) {
        *self
            .bounds
            .write()
            .unwrap_or_else(|error| error.into_inner()) = bounds;
    }

    fn EqualsLayer(&self, other: &dyn paint::paint_engine::ForeignLayer) -> bool {
        other
            .AsAny()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.layer == other.layer)
    }

    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
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
        p: &mut RasterPlaybackParams<'_, R>,
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
    pub state: Arc<paint_property_tree::PaintCanvasChildState>,
    pub record: CpuPaintRecord<R>,
}
impl crate::adapter::content_layer_client_impl::CanvasChildPaintState
    for Arc<paint_property_tree::PaintCanvasChildState>
{
    fn reference_box_offset(&self) -> Vector2dF {
        Vector2dF::new(
            self.paint_state.reference_box_offset.x as f32,
            self.paint_state.reference_box_offset.y as f32,
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
        PropertyTreeState::from_source(&self.properties)
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
        paint::paint_engine::PaintChunk::DrawsContent(self)
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
        paint::paint_engine::PaintChunk::Matches(self, other)
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
        self.ClientDebugName(client).to_string()
    }
    fn ClientOwnerNodeId(&self, client: usize) -> crate::runtime::types::NodeId {
        crate::runtime::types::NodeId(self.ClientOwnerNodeId(client))
    }
    fn PaintChunkCount(&self) -> usize {
        self.chunks.len()
    }
}
impl<R: CpuSkiaBackend> PendingDisplayItem<CpuCompositorTypes<R>> for DisplayItem {
    fn ForeignLayerVisualRect(&self) -> RectF {
        rect(self.visual_rect)
    }
    fn ForeignLayer(&self) -> crate::cc::layers::layer::LayerRef<CpuCompositorTypes<R>> {
        self.GetForeignLayer()
            .AsAny()
            .downcast_ref::<CpuForeignLayer<R>>()
            .expect("ForeignLayerDisplayItem owner must match the active compositor backend")
            .Layer()
            .clone()
    }
    fn ForeignVideoPresentationStyle(&self) -> Option<(FilterQuality, DynamicRangeLimit)> {
        let style = self.VideoPresentationStyle()?;
        Some((
            if style.nearest_neighbor {
                FilterQuality::kNone
            } else {
                FilterQuality::kLow
            },
            DynamicRangeLimit {
                standard_mix: style.dynamic_range_limit.standard_mix,
                constrained_high_mix: style.dynamic_range_limit.constrained_high_mix,
            },
        ))
    }
    fn IsScrollbar(&self) -> bool {
        matches!(self.kind, RecordedDisplayItemKind::Scrollbar) && self.scrollbar.is_some()
    }
    fn CreateOrReuseScrollbarLayer(
        &self,
        old: Option<crate::cc::layers::layer::LayerRef<CpuCompositorTypes<R>>>,
        offset: Vector2dF,
        c: &mut crate::adapter::main_layer::LayerContext<'_, CpuCompositorTypes<R>>,
    ) -> crate::cc::layers::layer::LayerRef<CpuCompositorTypes<R>> {
        use crate::{
            adapter::{geometry::Size, input_types::ElementId},
            cc::input::scrollbar::Scrollbar,
            cc::layers::scrollbar_layer_base::ScrollbarLayerBase,
        };
        let data = self
            .scrollbar
            .as_ref()
            .expect("ScrollbarDisplayItem retains its cc scrollbar inputs");
        let scrollbar = super::Scrollbar::from_paint(data);
        let needs_display = self.NeedsUpdateDisplay();
        let layer = ScrollbarLayerBase::<CpuCompositorTypes<R>>::CreateOrReuse(
            Arc::new(scrollbar),
            old.as_ref().map(|layer| layer.borrow()),
            c,
        )
        .expect("solid-color scrollbar constructor is registered");
        layer.SetIsDrawable(true, c);
        layer.SetContentsOpaque(self.IsOpaque(), c);
        let opaqueness = match data.hit_test_opaqueness {
            paint::paint_engine::PaintHitTestOpaqueness::Transparent => {
                crate::cc::input::hit_test_opaqueness::HitTestOpaqueness::kTransparent
            }
            paint::paint_engine::PaintHitTestOpaqueness::Mixed => {
                crate::cc::input::hit_test_opaqueness::HitTestOpaqueness::kMixed
            }
            paint::paint_engine::PaintHitTestOpaqueness::Opaque => {
                crate::cc::input::hit_test_opaqueness::HitTestOpaqueness::kOpaque
            }
        };
        layer.SetHitTestOpaqueness(opaqueness, c);
        layer.SetElementId(ElementId::new(self.ElementId().GetInternalValue()), c);
        let scroll_element_id = self
            .scroll_translation
            .as_ref()
            .map(|node| TransformNode::<R>::from_arc(node.clone()).ScrollCompositorElementId())
            .unwrap_or_default();
        layer.borrow().WithSource(c, |object, c| {
            object
                .AsScrollbarLayer()
                .expect("solid scrollbar resident subtype")
                .scrollbar_base()
                .SetScrollElementId(scroll_element_id, c);
        });
        layer.SetOffsetToTransformParent(
            Vector2dF::new(self.visual_rect.x as f32, self.visual_rect.y as f32) + offset,
            c,
        );
        layer.SetBounds(
            Size::new(
                self.visual_rect.width.round() as i32,
                self.visual_rect.height.round() as i32,
            ),
            c,
        );
        if needs_display {
            layer.SetNeedsDisplay(c);
        }
        layer
    }
}
