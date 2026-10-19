//! Resident Paint data projected into the sole canonical invalidation owner.
use super::*;
use crate::{
    adapter::{
        geometry::{Rect, RectF},
        paint_geometry::{ChunkGeometryMapper, GeometryMapperContext},
        raster_invalidator::*,
    },
    runtime::types::CpuSkiaBackend,
    third_party::blink::renderer::platform::graphics::paint::raster_invalidation_tracking::{
        RasterInvalidationBitmap, RasterInvalidationTracking, RasterInvalidationTrackingRuntime,
        RasterInvalidationTrackingTypes,
    },
};
use foundation::PaintInvalidationReason;
use std::{marker::PhantomData, sync::Arc};
/// Actual diagnostic raster pixels remain allocated by the external CPU owner.
pub trait CpuTrackingBitmapValue: Send + Sync {
    fn Row(&self, y: i32) -> &[u32];
    fn RowMut(&mut self, y: i32) -> &mut [u32];
}
pub struct CpuTrackingBitmap(pub Box<dyn CpuTrackingBitmapValue>);
impl RasterInvalidationBitmap for CpuTrackingBitmap {
    fn RowsEqual(&self, other: &Self, y: i32) -> bool {
        self.0.Row(y) == other.0.Row(y)
    }
    fn ClearRow(&mut self, y: i32) {
        self.0.RowMut(y).fill(0)
    }
    fn Pixel(&self, x: i32, y: i32) -> u32 {
        self.0.Row(y)[x as usize]
    }
    fn SetPixel(&mut self, x: i32, y: i32, color: u32) {
        self.0.RowMut(y)[x as usize] = color;
    }
}
pub trait CpuRasterTrackingServices<R: CpuSkiaBackend>: Send + Sync {
    fn PaintUnderInvalidationCheckingEnabled(&self) -> bool;
    fn IsTracingRasterInvalidations(&self) -> bool;
    fn Rasterize(&self, record: &CpuPaintRecord<R>, rect: Rect) -> Option<CpuTrackingBitmap>;
    fn BuildUnderInvalidationRecord(
        &self,
        previous: CpuPaintRecord<R>,
        overlay: CpuTrackingBitmap,
        rect: Rect,
    ) -> CpuPaintRecord<R>;
    fn LogUnderInvalidation(&self, name: &str, x: i32, y: i32, old: u32, new: u32);
    fn LogMoreUnderInvalidations(&self);
}
pub struct CpuRasterTrackingTypes<R: CpuSkiaBackend>(PhantomData<fn() -> R>);
impl<R: CpuSkiaBackend> RasterInvalidationTrackingTypes for CpuRasterTrackingTypes<R> {
    type RegionBackend = R;
    type PaintRecord = CpuPaintRecord<R>;
    type Bitmap = CpuTrackingBitmap;
    type Runtime = Arc<dyn CpuRasterTrackingServices<R>>;
}
impl<R: CpuSkiaBackend> RasterInvalidationTrackingRuntime<CpuRasterTrackingTypes<R>>
    for Arc<dyn CpuRasterTrackingServices<R>>
{
    fn PaintUnderInvalidationCheckingEnabled(&self) -> bool {
        self.as_ref().PaintUnderInvalidationCheckingEnabled()
    }
    fn IsTracingRasterInvalidations(&self) -> bool {
        self.as_ref().IsTracingRasterInvalidations()
    }
    fn Rasterize(&self, record: &CpuPaintRecord<R>, rect: Rect) -> Option<CpuTrackingBitmap> {
        self.as_ref().Rasterize(record, rect)
    }
    fn BuildUnderInvalidationRecord(
        &self,
        previous: CpuPaintRecord<R>,
        overlay: CpuTrackingBitmap,
        rect: Rect,
    ) -> CpuPaintRecord<R> {
        self.as_ref()
            .BuildUnderInvalidationRecord(previous, overlay, rect)
    }
    fn LogUnderInvalidation(&self, name: &str, x: i32, y: i32, old: u32, new: u32) {
        self.as_ref().LogUnderInvalidation(name, x, y, old, new)
    }
    fn LogMoreUnderInvalidations(&self) {
        self.as_ref().LogMoreUnderInvalidations()
    }
}
impl<R: CpuSkiaBackend> RasterInvalidationTrackingValue
    for RasterInvalidationTracking<CpuRasterTrackingTypes<R>>
{
    fn ClearInvalidations(&mut self) {
        Self::ClearInvalidations(self)
    }
    fn AddInvalidation(
        &mut self,
        id: usize,
        name: &str,
        rect: Rect,
        reason: PaintInvalidationReason,
    ) {
        Self::AddInvalidation(self, id, name, rect, reason)
    }
}
fn enclosing(r: paint::paint_engine::PaintRect) -> Rect {
    crate::adapter::geometry::ToEnclosingRect(RectF::new(
        r.x as f32,
        r.y as f32,
        r.width as f32,
        r.height as f32,
    ))
}
impl RasterInvalidationItem for DisplayItem {
    type Id = paint::display_item_id::DisplayItemId;
    fn GetId(&self) -> Self::Id {
        self.id
    }
    fn ClientId(&self) -> usize {
        self.id.client_id as usize
    }
    fn DrawsContent(&self) -> bool {
        self.draws_content
    }
    fn IsCacheable(&self) -> bool {
        self.is_cacheable
    }
    fn IsTombstone(&self) -> bool {
        self.is_tombstone
    }
    fn ScrollbarNeedsUpdateDisplay(&self) -> Option<bool> {
        if self.kind == RecordedDisplayItemKind::Scrollbar {
            Some(
                self.scrollbar_needs_update_display
                    .expect("validated scrollbar owner metadata"),
            )
        } else {
            None
        }
    }
    fn VisualRect(&self) -> Rect {
        enclosing(self.visual_rect)
    }
    fn GetPaintInvalidationReason(&self) -> PaintInvalidationReason {
        self.paint_invalidation_reason
            .expect("validated native DisplayItem reason")
    }
}
impl<R: CpuSkiaBackend> RasterInvalidationChunk<CpuGeometryMapper<R>> for PaintChunk {
    type Id = paint::display_item_id::DisplayItemId;
    fn id(&self) -> Self::Id {
        self.id
    }
    fn client_id(&self) -> usize {
        self.id.client_id as usize
    }
    fn drawable_bounds(&self) -> Rect {
        enclosing(self.drawable_bounds)
    }
    fn effectively_invisible(&self) -> bool {
        self.effectively_invisible
    }
    fn is_moved_from_cached_subsequence(&self) -> bool {
        self.is_moved_from_cached_subsequence
    }
    fn is_cacheable(&self) -> bool {
        self.is_cacheable
    }
    fn CanMatchOldChunk(&self) -> bool {
        self.can_match_old_chunk()
    }
    fn Matches(&self, old: &Self) -> bool {
        self.matches(old)
    }
    fn DrawsContent(&self) -> bool {
        !self.drawable_bounds.is_empty()
    }
    fn IsForeignLayerType(&self) -> bool {
        self.id.r#type.0 >= paint::display_item_id::DisplayItemIdType::kForeignLayerFirst.0
            && self.id.r#type.0 <= paint::display_item_id::DisplayItemIdType::kForeignLayerLast.0
    }
}
impl<R: CpuSkiaBackend> RasterInvalidationArtifact<CpuGeometryMapper<R>> for PaintArtifact {
    fn Chunk(&self, index: usize) -> &PaintChunk {
        &self.chunks[index]
    }
    fn DisplayItemsInChunk(&self, index: usize) -> &[DisplayItem] {
        let chunk = &self.chunks[index];
        &self.display_items[chunk.begin_index as usize..chunk.end_index as usize]
    }
    // Source: paint_artifact.cc:37-41; absent diagnostic name is native empty.
    fn ClientDebugName(&self, id: usize) -> &str {
        self.client_debug_names.get(&id).map_or("", |s| s.as_ref())
    }
}
impl<R: CpuSkiaBackend> RasterInvalidationSubset<CpuGeometryMapper<R>> for PaintChunkSubset {
    fn artifact(&self) -> &Arc<PaintArtifact> {
        self.artifact()
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn indices(&self) -> impl Iterator<Item = usize> {
        self.indices().iter().copied()
    }
}
impl<R: CpuSkiaBackend> RasterInvalidationTypes for CpuGeometryMapper<R> {
    type Chunk = PaintChunk;
    type Item = DisplayItem;
    type Artifact = PaintArtifact;
    type Subset = PaintChunkSubset;
    type Tracking = RasterInvalidationTracking<CpuRasterTrackingTypes<R>>;
    type Region = R;
    type Effect = EffectNode<R>;
    fn RootState(&self) -> PropertyTreeState<R> {
        GeometryMapperContext::RootState(self)
    }
    fn SameEffectNode(&self, a: &PropertyTreeState<R>, b: &PropertyTreeState<R>) -> bool {
        a.effect_ == b.effect_
    }
    fn ShouldAlwaysTrack(&self) -> bool {
        Self::Tracking::ShouldAlwaysTrack(&self.tracking)
    }
    fn NewTracking(&self) -> Self::Tracking {
        Self::Tracking::new(self.tracking.clone())
    }
    fn Effect(&self, state: &PropertyTreeState<R>) -> Self::Effect {
        state.effect_.clone()
    }
    fn ParentEffect(&self, e: &Self::Effect) -> Option<Self::Effect> {
        e.node()
            .parent
            .as_ref()
            .map(|e| EffectNode::from_arc(e.clone()))
    }
    fn HasReferenceFilter(&self, e: &Self::Effect) -> bool {
        e.node()
            .filters
            .canonical()
            .expect("validated native filters")
            .HasReferenceFilter()
    }
    fn ChunkPropertiesChangedOnlyComposited(
        &self,
        chunk: &PaintChunk,
        layer: &PropertyTreeState<R>,
    ) -> bool {
        paint::property_node_geometry::PropertyStateChanged(
            &chunk.properties,
            native::PaintPropertyChangeType::ChangedOnlyCompositedValues,
            &layer.native(),
        )
    }
    fn EffectChangedOnlySimple(
        &self,
        state: &PropertyTreeState<R>,
        layer: &PropertyTreeState<R>,
    ) -> bool {
        paint::property_node_geometry::EffectChanged(
            state.effect_.node(),
            native::PaintPropertyChangeType::ChangedOnlySimpleValues,
            &layer.native(),
            Some(state.transform_.node()),
        )
    }
    fn ClipChangedOnlySimple(
        &self,
        state: &PropertyTreeState<R>,
        layer: &PropertyTreeState<R>,
    ) -> bool {
        paint::property_node_geometry::ClipChanged(
            state.clip_.node(),
            native::PaintPropertyChangeType::ChangedOnlySimpleValues,
            &layer.native(),
            Some(state.transform_.node()),
        )
    }
    fn TransformApproximatelyEqual(
        &self,
        a: &Self::Transform,
        b: &Self::Transform,
        t: f32,
        o: f32,
    ) -> bool {
        a.native().ApproximatelyEqual(b.native(), t, o, o)
    }
    fn BackdropFilterBounds(&self, chunk: &PaintChunk) -> Option<RectF> {
        let effect = &chunk.properties.effect;
        if effect.backdrop_filters.is_empty() {
            return None;
        }
        let r = effect
            .backdrop_filter_bounds
            .as_ref()
            .expect("producer BackdropFilterInfo bounds")
            .getBounds();
        Some(RectF::new(
            r.left,
            r.top,
            r.right - r.left,
            r.bottom - r.top,
        ))
    }
}

pub struct CpuRasterInvalidator<R:CpuSkiaBackend>{
    source:crate::third_party::blink::renderer::platform::graphics::paint::raster_invalidator::RasterInvalidator<CpuGeometryMapper<R>>,
    mapper:Arc<CpuGeometryMapper<R>>,
}
impl<R: CpuSkiaBackend> CpuRasterInvalidator<R> {
    pub fn new(mapper: Arc<CpuGeometryMapper<R>>) -> Self {
        Self{source:crate::third_party::blink::renderer::platform::graphics::paint::raster_invalidator::RasterInvalidator::new(mapper.as_ref()),mapper}
    }
}
impl<R, B> crate::adapter::content_layer_client_impl::RasterInvalidator<B>
    for CpuRasterInvalidator<R>
where
    R: CpuSkiaBackend,
    B: crate::adapter::pending_layer::PendingLayerTypes<
        PaintArtifact = PaintArtifact,
        PaintChunkSubset = PaintChunkSubset,
        PropertyTreeState = PropertyTreeState<R>,
        RasterInvalidationTracking = RasterInvalidationTracking<CpuRasterTrackingTypes<R>>,
    >,
{
    fn LayerBounds(&self) -> crate::adapter::geometry::Size {
        self.source.LayerBounds()
    }
    fn Generate(
        &mut self,
        chunks: &PaintChunkSubset,
        offset: crate::adapter::geometry::Vector2dF,
        bounds: crate::adapter::geometry::Size,
        state: &PropertyTreeState<R>,
        callback: &mut dyn crate::adapter::content_layer_client_impl::RasterInvalidationCallback,
    ) {
        self.source.Generate(
            chunks,
            offset,
            bounds,
            state,
            self.mapper.as_ref(),
            callback,
        )
    }
    fn GetTracking(&self) -> Option<&RasterInvalidationTracking<CpuRasterTrackingTypes<R>>> {
        self.source.GetTracking()
    }
    fn GetTrackingMutable(
        &mut self,
    ) -> Option<&mut RasterInvalidationTracking<CpuRasterTrackingTypes<R>>> {
        self.source.GetTrackingMutable()
    }
    fn SetTracksRasterInvalidations(&mut self, value: bool) {
        self.source
            .SetTracksRasterInvalidations(value, self.mapper.as_ref())
    }
    fn UpdateForRasterInducingScroll(&mut self, chunks: &PaintChunkSubset) {
        self.source
            .UpdateForRasterInducingScroll(chunks, self.mapper.as_ref())
    }
    fn SetOldPaintArtifact(&mut self, artifact: Arc<PaintArtifact>) {
        self.source.SetOldPaintArtifact(artifact)
    }
    fn ApproximateUnsharedMemoryUsage(&self) -> usize {
        self.source.ApproximateUnsharedMemoryUsage()
    }
}
impl<R, B> crate::adapter::content_layer_client_impl::RasterInvalidationTracking<B>
    for RasterInvalidationTracking<CpuRasterTrackingTypes<R>>
where
    R: CpuSkiaBackend,
    B: crate::adapter::pending_layer::PendingLayerTypes<Json = serde_json::Value>,
{
    fn AsJSON(&self, json: &mut serde_json::Value, detailed: bool) {
        Self::AsJSON(self, json, detailed)
    }
}
impl<R, B> crate::adapter::paint_chunks_to_cc_layer::ConversionInvalidationTracking<B>
    for RasterInvalidationTracking<CpuRasterTrackingTypes<R>>
where
    R: CpuSkiaBackend,
    B: crate::adapter::paint_chunks_to_cc_layer::PaintChunksTypes<
        PaintRecord = CpuPaintRecord<R>,
        UnderInvalidationPicture = CpuPaintRecord<R>,
    >,
{
    fn CheckUnderInvalidations(&mut self, name: &str, picture: CpuPaintRecord<R>, interest: Rect) {
        Self::CheckUnderInvalidations(self, name, picture, interest)
    }
    fn UnderInvalidationRecord(&self) -> CpuPaintRecord<R> {
        Self::UnderInvalidationRecord(self)
    }
}
impl<R, B> crate::adapter::paint_artifact_compositor::ArtifactInvalidationTracking<B>
    for RasterInvalidationTracking<CpuRasterTrackingTypes<R>>
where
    R: CpuSkiaBackend,
    B: crate::adapter::paint_artifact_compositor::PaintArtifactCompositorTypes,
{
    fn AddToLayerDebugInfo(&self, info: &mut crate::cc::layers::layer::LayerDebugInfo<B::NodeId>) {
        Self::AddToLayerDebugInfo(self, info)
    }
    fn ClearInvalidations(&mut self) {
        Self::ClearInvalidations(self)
    }
}
impl crate::third_party::blink::renderer::platform::graphics::paint::raster_invalidation_tracking::RasterInvalidationJson for serde_json::Value{
 fn SetInvalidations(&mut self,detailed:bool,values:&[crate::third_party::blink::renderer::platform::graphics::paint::raster_invalidation_tracking::RasterInvalidationInfo]){
    use crate::third_party::blink::renderer::platform::graphics::paint::raster_invalidation_tracking::PaintInvalidationReasonToString;
    self["invalidations"]=serde_json::Value::Array(values.iter().map(|v|if detailed{serde_json::json!({"object":v.client_debug_name,"rect":[v.rect.x,v.rect.y,v.rect.width,v.rect.height],"reason":PaintInvalidationReasonToString(v.reason)})}else{serde_json::json!([v.rect.x,v.rect.y,v.rect.width,v.rect.height])}).collect());
 }
 fn SetUnderInvalidations(&mut self,values:&[crate::third_party::blink::renderer::platform::graphics::paint::raster_invalidation_tracking::RasterUnderInvalidation]){self["underInvalidations"]=serde_json::Value::Array(values.iter().map(|v|serde_json::json!({"x":v.x,"y":v.y,"oldPixel":sk_color_name(v.old_pixel),"newPixel":sk_color_name(v.new_pixel)})).collect());}
}

// Source: Color::NameForLayoutTreeAsText, color.cc:1113-1124; SkColor is ARGB.
fn sk_color_name(pixel: u32) -> String {
    let a = pixel >> 24;
    let r = (pixel >> 16) & 255;
    let g = (pixel >> 8) & 255;
    let b = pixel & 255;
    if a == 255 {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}
