//! Resident Paint inputs and leaf capabilities used by canonical invalidators.
use super::{
    geometry::{Rect, RectF},
    paint_geometry::{ChunkGeometryMapper, PaintChunkView},
};
use crate::third_party::blink::renderer::platform::graphics::paint::float_clip_rect::FloatClipRect;
use foundation::PaintInvalidationReason;
use std::sync::Arc;
pub type DisplayItemClientId = usize;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientIsOldOrNew {
    kClientIsOld,
    kClientIsNew,
}
pub trait RasterInvalidationItem {
    type Id: Eq;
    fn GetId(&self) -> Self::Id;
    fn ClientId(&self) -> DisplayItemClientId;
    fn DrawsContent(&self) -> bool;
    fn IsCacheable(&self) -> bool;
    fn IsTombstone(&self) -> bool;
    fn ScrollbarNeedsUpdateDisplay(&self) -> Option<bool>;
    fn VisualRect(&self) -> Rect;
    fn RectKnownToBeOpaque(&self) -> Rect {
        Rect::default()
    }
    fn IsBoxDecorationBackground(&self) -> bool {
        false
    }
    fn GetPaintInvalidationReason(&self) -> PaintInvalidationReason;
}
pub trait RasterInvalidationChunk<M: RasterInvalidationTypes + ?Sized>: PaintChunkView<M> {
    type Id: Clone + Eq;
    fn id(&self) -> Self::Id;
    fn client_id(&self) -> DisplayItemClientId;
    fn drawable_bounds(&self) -> Rect;
    fn effectively_invisible(&self) -> bool;
    fn is_moved_from_cached_subsequence(&self) -> bool;
    fn is_cacheable(&self) -> bool;
    fn CanMatchOldChunk(&self) -> bool;
    fn Matches(&self, old: &Self) -> bool;
    fn DrawsContent(&self) -> bool;
    fn IsForeignLayerType(&self) -> bool;
}
pub trait RasterInvalidationArtifact<M: RasterInvalidationTypes + ?Sized> {
    fn Chunk(&self, index: usize) -> &M::Chunk;
    fn DisplayItemsInChunk(&self, index: usize) -> &[M::Item];
    fn ImageContentRevisionsInChunk(&self, index: usize) -> &[Vec<(u64, u64)>];
    fn ChunkHasResourceChange(&self, index: usize) -> bool;
    fn ClientDebugName(&self, id: DisplayItemClientId) -> &str;
}
pub trait RasterInvalidationSubset<M: RasterInvalidationTypes + ?Sized> {
    fn artifact(&self) -> &Arc<M::Artifact>;
    fn len(&self) -> usize;
    fn indices(&self) -> impl Iterator<Item = usize>;
}
pub trait RasterInvalidationTrackingValue {
    fn ClearInvalidations(&mut self);
    fn AddInvalidation(
        &mut self,
        id: DisplayItemClientId,
        name: &str,
        rect: Rect,
        reason: PaintInvalidationReason,
    );
}
pub trait RasterInvalidationTypes: ChunkGeometryMapper<FloatClipRect = FloatClipRect> {
    type Chunk: RasterInvalidationChunk<Self>;
    type Item: RasterInvalidationItem;
    type Artifact: RasterInvalidationArtifact<Self>;
    type Subset: RasterInvalidationSubset<Self>;
    type Tracking: RasterInvalidationTrackingValue;
    type Region: super::region_backend::SkRegionBackend;
    fn RootState(&self) -> Self::PropertyTreeState;
    fn SameEffectNode(
        &self,
        first: &Self::PropertyTreeState,
        second: &Self::PropertyTreeState,
    ) -> bool;
    fn ShouldAlwaysTrack(&self) -> bool;
    fn NewTracking(&self) -> Self::Tracking;
    type Effect: Clone + PartialEq;
    fn Effect(&self, state: &Self::PropertyTreeState) -> Self::Effect;
    fn ParentEffect(&self, effect: &Self::Effect) -> Option<Self::Effect>;
    fn HasReferenceFilter(&self, effect: &Self::Effect) -> bool;
    fn ChunkPropertiesChangedOnlyComposited(
        &self,
        chunk: &Self::Chunk,
        layer: &Self::PropertyTreeState,
    ) -> bool;
    fn EffectChangedOnlySimple(
        &self,
        state: &Self::PropertyTreeState,
        layer: &Self::PropertyTreeState,
    ) -> bool;
    fn ClipChangedOnlySimple(
        &self,
        state: &Self::PropertyTreeState,
        layer: &Self::PropertyTreeState,
    ) -> bool;
    fn TransformApproximatelyEqual(
        &self,
        new: &Self::Transform,
        old: &Self::Transform,
        translation: f32,
        other: f32,
    ) -> bool;
    fn BackdropFilterBounds(&self, chunk: &Self::Chunk) -> Option<RectF>;
}
pub trait RasterInvalidationSink {
    fn ClipByLayerBounds(&self, rect: Rect) -> Rect;
    fn FindRoundedOpaqueBacking(&self, _rect: Rect) -> Option<RoundedOpaqueBacking> {
        None
    }
    fn AddRasterInvalidation(
        &mut self,
        rect: Rect,
        id: DisplayItemClientId,
        reason: PaintInvalidationReason,
        old_or_new: ClientIsOldOrNew,
    );
    fn AddRasterInvalidationForFullReplay(
        &mut self,
        rect: Rect,
        inner: Rect,
        replay_rect: Rect,
        id: DisplayItemClientId,
        reason: PaintInvalidationReason,
        old_or_new: ClientIsOldOrNew,
    ) {
        let _ = (inner, replay_rect);
        self.AddRasterInvalidation(rect, id, reason, old_or_new);
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundedOpaqueBacking {
    pub rect: Rect,
    pub has_overlapping_sibling: bool,
}
pub trait RasterVisualRectMapper {
    fn MapVisualRect(&self, rect: Rect) -> Rect;
    fn MapVisualRectWithoutRasterEffectOutset(&self, rect: Rect) -> Rect {
        self.MapVisualRect(rect)
    }
    fn HasRoundedClip(&self) -> bool {
        false
    }
    fn NeedsRoundedClipAAOutset(&self) -> bool {
        false
    }
    fn RoundedClipHasFractionalRightEdge(&self) -> bool {
        false
    }
}
impl<M: ChunkGeometryMapper + ?Sized> RasterVisualRectMapper
    for crate::third_party::blink::renderer::platform::graphics::compositing::chunk_to_layer_mapper::ChunkToLayerMapper<'_, M>
{
    fn MapVisualRect(&self, rect: Rect) -> Rect {
        Self::MapVisualRect(self, rect)
    }
    fn MapVisualRectWithoutRasterEffectOutset(&self, rect: Rect) -> Rect {
        Self::MapVisualRectWithoutRasterEffectOutset(self, rect)
    }
    fn HasRoundedClip(&self) -> bool {
        Self::HasRoundedClip(self)
    }
    fn NeedsRoundedClipAAOutset(&self) -> bool {
        Self::NeedsRoundedClipAAOutset(self)
    }
    fn RoundedClipHasFractionalRightEdge(&self) -> bool {
        Self::RoundedClipHasFractionalRightEdge(self)
    }
}
