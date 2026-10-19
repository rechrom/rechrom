// Translation status: complete
// Copyright 2015 The Chromium Authors. BSD-style license in LICENSE.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:1-74;
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:1-24.
use crate::adapter::{geometry::{Rect,RectF,Size,Vector2dF,ToEnclosingRect},paint_geometry::PaintChunkView,raster_invalidator::*,region_backend::{SkRegionBackend,SkRegionOp,SkIRect},content_layer_client_impl::RasterInvalidationCallback};
use super::{float_clip_rect::FloatClipRect,display_item_raster_invalidator::DisplayItemRasterInvalidator};
use crate::third_party::blink::renderer::platform::graphics::compositing::chunk_to_layer_mapper::ChunkToLayerMapper;
use foundation::{PaintInvalidationReason as Reason,IsFullPaintInvalidationReason};
use std::sync::Arc;
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:75-123.
struct PaintChunkInfo<M: RasterInvalidationTypes> {
    index_in_paint_artifact: usize,
    #[cfg(debug_assertions)]
    id: <M::Chunk as RasterInvalidationChunk<M>>::Id,
    bounds_in_layer: Rect,
    chunk_to_layer_clip: FloatClipRect,
    chunk_to_layer_transform: M::Transform,
}
impl<M: RasterInvalidationTypes> PaintChunkInfo<M> {
    fn new(
        bounds: Size,
        mapper: &ChunkToLayerMapper<'_, M>,
        artifact: &M::Artifact,
        index: usize,
        m: &M,
    ) -> Self {
        let chunk = artifact.Chunk(index);
        let mut result = Self {
            index_in_paint_artifact: index,
            #[cfg(debug_assertions)]
            id: chunk.id(),
            bounds_in_layer: clip_rect(mapper.MapVisualRect(chunk.drawable_bounds()), bounds),
            chunk_to_layer_clip: *mapper.ClipRect(),
            chunk_to_layer_transform: mapper.Transform().clone(),
        };
        if let Some(backdrop) = m.BackdropFilterBounds(chunk) {
            result.bounds_in_layer.Union(clip_rect(
                mapper.MapVisualRect(ToEnclosingRect(backdrop)),
                bounds,
            ));
        }
        result
    }
    fn FromOld(old: &Self, _artifact: &M::Artifact, index: usize) -> Self {
        #[cfg(debug_assertions)]
        debug_assert!(_artifact.Chunk(index).id() == old.id);
        Self {
            index_in_paint_artifact: index,
            #[cfg(debug_assertions)]
            id: _artifact.Chunk(index).id(),
            bounds_in_layer: old.bounds_in_layer,
            chunk_to_layer_clip: old.chunk_to_layer_clip,
            chunk_to_layer_transform: old.chunk_to_layer_transform.clone(),
        }
    }
}
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:26-72,174-187.
// The C++ callback raw_ref becomes an explicit call-scoped callback. No child
// retains its ContentLayerClient owner; artifact Arcs are native record leases.
pub struct RasterInvalidator<M: RasterInvalidationTypes> {
    layer_offset_: Vector2dF,
    layer_bounds_: Size,
    layer_state_: M::PropertyTreeState,
    old_paint_chunks_info_: Vec<PaintChunkInfo<M>>,
    current_paint_artifact_: Option<Arc<M::Artifact>>,
    old_paint_artifact_: Option<Arc<M::Artifact>>,
    tracking_: Option<M::Tracking>,
}
fn clip_rect(mut rect: Rect, size: Size) -> Rect {
    rect.Intersect(Rect::new(0, 0, size.width(), size.height()));
    rect
}
fn to_float(rect: Rect) -> RectF {
    RectF::new(
        rect.x as f32,
        rect.y as f32,
        rect.width as f32,
        rect.height as f32,
    )
}
impl<M: RasterInvalidationTypes> RasterInvalidator<M> {
    pub fn new(m: &M) -> Self {
        Self {
            layer_offset_: Vector2dF::default(),
            layer_bounds_: Size::default(),
            layer_state_: m.RootState(),
            old_paint_chunks_info_: Vec::new(),
            current_paint_artifact_: None,
            old_paint_artifact_: None,
            tracking_: None,
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:26-37;
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:39-42.
    pub fn SetTracksRasterInvalidations(&mut self, should_track: bool, m: &M) {
        if should_track {
            if self.tracking_.is_none() {
                self.tracking_ = Some(m.NewTracking());
            }
            self.tracking_.as_mut().unwrap().ClearInvalidations();
        } else if !m.ShouldAlwaysTrack() {
            self.tracking_ = None;
        } else if let Some(t) = &mut self.tracking_ {
            t.ClearInvalidations();
        }
    }
    pub fn GetTracking(&self) -> Option<&M::Tracking> {
        self.tracking_.as_ref()
    }
    pub fn GetTrackingMutable(&mut self) -> Option<&mut M::Tracking> {
        self.tracking_.as_mut()
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:39-62.
    fn GetOldChunk(&self, index: usize) -> &M::Chunk {
        let info = &self.old_paint_chunks_info_[index];
        let chunk = self
            .old_paint_artifact_
            .as_ref()
            .expect("source old artifact")
            .Chunk(info.index_in_paint_artifact);
        #[cfg(debug_assertions)]
        debug_assert!(chunk.id() == info.id);
        chunk
    }
    fn MatchNewChunkToOldChunk(&self, new: &M::Chunk, index: usize) -> Option<usize> {
        if !new.CanMatchOldChunk() {
            return None;
        }
        for i in index..self.old_paint_chunks_info_.len() {
            if new.Matches(self.GetOldChunk(i)) {
                return Some(i);
            }
        }
        None
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:64-143;
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:154-163.
    fn ChunkPropertiesChanged(
        &self,
        new: &M::Chunk,
        old: &M::Chunk,
        new_info: &PaintChunkInfo<M>,
        old_info: &PaintChunkInfo<M>,
        layer: &M::PropertyTreeState,
        translation: f32,
        other: f32,
        m: &M,
    ) -> Reason {
        if new.effectively_invisible() != old.effectively_invisible() {
            return Reason::kPaintProperty;
        }
        if !m.TransformApproximatelyEqual(
            &new_info.chunk_to_layer_transform,
            &old_info.chunk_to_layer_transform,
            translation,
            other,
        ) {
            return Reason::kPaintProperty;
        }
        let state = new.properties_unalias();
        let old_state = old.properties_unalias();
        let mut clip_different = false;
        let mut effect_different = false;
        if !new.is_moved_from_cached_subsequence() {
            clip_different = !m.SameClipNode(&state, &old_state);
            effect_different = !m.SameEffectNode(&state, &old_state);
        }
        if effect_different || m.EffectChangedOnlySimple(&state, layer) {
            return Reason::kPaintProperty;
        }
        if new_info.chunk_to_layer_clip.IsTight() && old_info.chunk_to_layer_clip.IsTight() {
            let new_rect = *new_info.chunk_to_layer_clip.Rect();
            let old_rect = *old_info.chunk_to_layer_clip.Rect();
            if new_rect == old_rect {
                return Reason::kNone;
            }
            let mut new_clip = new_rect;
            new_clip.Intersect(to_float(Rect::new(
                0,
                0,
                self.layer_bounds_.width(),
                self.layer_bounds_.height(),
            )));
            let mut old_clip = old_rect;
            old_clip.Intersect(to_float(Rect::new(
                0,
                0,
                self.layer_bounds_.width(),
                self.layer_bounds_.height(),
            )));
            if new_clip == old_clip {
                return Reason::kNone;
            }
            if !new_clip.Contains(to_float(new_info.bounds_in_layer))
                || !old_clip.Contains(to_float(old_info.bounds_in_layer))
            {
                return Reason::kPaintProperty;
            }
            return Reason::kIncremental;
        }
        if clip_different || m.ClipChangedOnlySimple(&state, layer) {
            return Reason::kPaintProperty;
        }
        Reason::kNone
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:145-172.
    fn ShouldSkipForRasterInvalidation(
        chunk: &M::Chunk,
        layer: &M::PropertyTreeState,
        m: &M,
    ) -> bool {
        if !chunk.DrawsContent() {
            let mut has_reference_filter = false;
            let parent_effect = m.Effect(layer);
            let mut node = m.Effect(&chunk.properties_unalias());
            while node != parent_effect {
                if m.HasReferenceFilter(&node) {
                    has_reference_filter = true;
                    break;
                }
                node = m.ParentEffect(&node).expect("source effect ancestor");
            }
            if !has_reference_filter {
                return true;
            }
        }
        chunk.IsForeignLayerType()
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:174-184.
    fn ScrollbarNeedsUpdateDisplay(items: &[M::Item]) -> bool {
        if items.len() != 1 {
            return false;
        }
        if let Some(needs_update) = items[0].ScrollbarNeedsUpdateDisplay() {
            return needs_update;
        }
        false
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:186-320;
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:124-134.
    fn GenerateRasterInvalidations(
        &mut self,
        chunks: &M::Subset,
        offset_changed: bool,
        effect_changed: bool,
        new_infos: &mut Vec<PaintChunkInfo<M>>,
        m: &M,
        callback: &mut dyn RasterInvalidationCallback,
    ) {
        let mut mapper = ChunkToLayerMapper::new(m, &self.layer_state_, self.layer_offset_);
        let mut matched = vec![false; self.old_paint_chunks_info_.len()];
        let mut old_index = 0;
        let translation = 1e-2f32;
        let other = 1e-4f32;
        for index in chunks.indices() {
            let new = chunks.artifact().Chunk(index);
            if Self::ShouldSkipForRasterInvalidation(new, &self.layer_state_, m) {
                continue;
            }
            let Some(matched_index) = self.MatchNewChunkToOldChunk(new, old_index) else {
                mapper.SwitchToChunk(new);
                new_infos.push(PaintChunkInfo::new(
                    self.layer_bounds_,
                    &mapper,
                    chunks.artifact(),
                    index,
                    m,
                ));
                let bounds = new_infos.last().unwrap().bounds_in_layer;
                self.AddRasterInvalidation(
                    bounds,
                    new.client_id(),
                    if new.is_cacheable() {
                        Reason::kChunkAppeared
                    } else {
                        Reason::kChunkUncacheable
                    },
                    ClientIsOldOrNew::kClientIsNew,
                    callback,
                );
                continue;
            };
            debug_assert!(!matched[matched_index]);
            matched[matched_index] = true;
            let clipped = clip_rect(
                self.old_paint_chunks_info_[matched_index].bounds_in_layer,
                self.layer_bounds_,
            );
            self.old_paint_chunks_info_[matched_index].bounds_in_layer = clipped;
            let mut reason = if Self::ScrollbarNeedsUpdateDisplay(
                chunks.artifact().DisplayItemsInChunk(index),
            ) {
                Reason::kScrollControl
            } else {
                Reason::kNone
            };
            if !offset_changed
                && reason == Reason::kNone
                && new.is_moved_from_cached_subsequence()
                && !m.ChunkPropertiesChangedOnlyComposited(new, &self.layer_state_)
            {
                new_infos.push(PaintChunkInfo::FromOld(
                    &self.old_paint_chunks_info_[matched_index],
                    chunks.artifact(),
                    index,
                ));
            } else {
                mapper.SwitchToChunk(new);
                new_infos.push(PaintChunkInfo::new(
                    self.layer_bounds_,
                    &mapper,
                    chunks.artifact(),
                    index,
                    m,
                ));
                let info = new_infos.last_mut().unwrap();
                if reason == Reason::kNone {
                    reason = if effect_changed {
                        Reason::kPaintProperty
                    } else {
                        self.ChunkPropertiesChanged(
                            new,
                            self.GetOldChunk(matched_index),
                            info,
                            &self.old_paint_chunks_info_[matched_index],
                            &self.layer_state_,
                            translation,
                            other,
                            m,
                        )
                    };
                }
                if IsFullPaintInvalidationReason(reason) {
                    let old_bounds = self.old_paint_chunks_info_[matched_index].bounds_in_layer;
                    self.AddRasterInvalidation(
                        old_bounds,
                        new.client_id(),
                        reason,
                        ClientIsOldOrNew::kClientIsNew,
                        callback,
                    );
                    if old_bounds != info.bounds_in_layer {
                        self.AddRasterInvalidation(
                            info.bounds_in_layer,
                            new.client_id(),
                            reason,
                            ClientIsOldOrNew::kClientIsNew,
                            callback,
                        );
                    }
                } else {
                    info.chunk_to_layer_transform = self.old_paint_chunks_info_[matched_index]
                        .chunk_to_layer_transform
                        .clone();
                    if reason == Reason::kIncremental {
                        self.IncrementallyInvalidateChunk(
                            self.old_paint_chunks_info_[matched_index].bounds_in_layer,
                            info.bounds_in_layer,
                            new.client_id(),
                            callback,
                        );
                    }
                    let old_artifact = self
                        .old_paint_artifact_
                        .as_ref()
                        .expect("source old artifact")
                        .clone();
                    if !Arc::ptr_eq(chunks.artifact(), &old_artifact)
                        && !new.is_moved_from_cached_subsequence()
                    {
                        let old_chunk_index =
                            self.old_paint_chunks_info_[matched_index].index_in_paint_artifact;
                        let mut sink = RasterInvalidatorSink {
                            owner: self,
                            callback,
                        };
                        DisplayItemRasterInvalidator::new(
                            &mut sink,
                            old_artifact.DisplayItemsInChunk(old_chunk_index),
                            chunks.artifact().DisplayItemsInChunk(index),
                            &mapper,
                        )
                        .Generate();
                    }
                }
            }
            old_index = matched_index + 1;
        }
        for i in 0..self.old_paint_chunks_info_.len() {
            if matched[i] {
                continue;
            }
            let old = self.GetOldChunk(i);
            let reason = if old.is_cacheable() {
                Reason::kChunkDisappeared
            } else {
                Reason::kChunkUncacheable
            };
            let id = old.client_id();
            self.AddRasterInvalidation(
                self.old_paint_chunks_info_[i].bounds_in_layer,
                id,
                reason,
                ClientIsOldOrNew::kClientIsOld,
                callback,
            );
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:322-333;
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:135-139.
    fn IncrementallyInvalidateChunk(
        &mut self,
        old: Rect,
        new: Rect,
        id: DisplayItemClientId,
        callback: &mut dyn RasterInvalidationCallback,
    ) {
        let mut diff = M::Region::from_rect(SkIRect::FromRect(old));
        diff.op_rect(SkIRect::FromRect(new), SkRegionOp::XOR);
        for rect in diff.rects() {
            self.AddRasterInvalidation(
                rect.ToRect(),
                id,
                Reason::kIncremental,
                ClientIsOldOrNew::kClientIsNew,
                callback,
            );
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:141-152;
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:335-347.
    fn AddRasterInvalidation(
        &mut self,
        rect: Rect,
        id: DisplayItemClientId,
        reason: Reason,
        old: ClientIsOldOrNew,
        callback: &mut dyn RasterInvalidationCallback,
    ) {
        callback.InvalidateRect(rect);
        if self.tracking_.is_some() {
            self.TrackRasterInvalidation(rect, id, reason, old);
        }
    }
    fn TrackRasterInvalidation(
        &mut self,
        rect: Rect,
        id: DisplayItemClientId,
        reason: Reason,
        old: ClientIsOldOrNew,
    ) {
        if rect.IsEmpty() {
            return;
        }
        let artifact = if old == ClientIsOldOrNew::kClientIsOld {
            self.old_paint_artifact_.as_ref()
        } else {
            self.current_paint_artifact_.as_ref()
        }
        .expect("source tracking artifact");
        self.tracking_
            .as_mut()
            .expect("source tracking")
            .AddInvalidation(id, artifact.ClientDebugName(id), rect, reason);
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:349-354.
    pub fn EnsureTracking(&mut self, m: &M) -> &mut M::Tracking {
        if self.tracking_.is_none() {
            self.tracking_ = Some(m.NewTracking());
        }
        self.tracking_.as_mut().unwrap()
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:356-394;
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:44-51.
    pub fn Generate(
        &mut self,
        chunks: &M::Subset,
        offset: Vector2dF,
        bounds: Size,
        state: &M::PropertyTreeState,
        m: &M,
        callback: &mut dyn RasterInvalidationCallback,
    ) {
        if m.ShouldAlwaysTrack() {
            self.EnsureTracking(m);
        }
        let offset_changed = self.layer_offset_ != offset || self.layer_state_ != *state;
        let effect_changed = !m.SameEffectNode(&self.layer_state_, state);
        let was_empty = self.layer_bounds_.IsEmpty();
        self.layer_offset_ = offset;
        self.layer_bounds_ = bounds;
        self.layer_state_ = state.clone();
        self.current_paint_artifact_ = Some(chunks.artifact().clone());
        let mut new_infos = Vec::with_capacity(chunks.len());
        if was_empty || self.layer_bounds_.IsEmpty() {
            Self::PopulatePaintChunksInfo(
                self.layer_bounds_,
                chunks,
                offset,
                state,
                &mut new_infos,
                m,
            );
            if !bounds.IsEmpty() && chunks.len() != 0 {
                let first = chunks.indices().next().expect("source nonempty chunks");
                self.AddRasterInvalidation(
                    Rect::new(0, 0, bounds.width(), bounds.height()),
                    chunks.artifact().Chunk(first).client_id(),
                    Reason::kFullLayer,
                    ClientIsOldOrNew::kClientIsNew,
                    callback,
                );
            }
        } else {
            self.GenerateRasterInvalidations(
                chunks,
                offset_changed,
                effect_changed,
                &mut new_infos,
                m,
                callback,
            );
        }
        self.old_paint_chunks_info_ = new_infos;
        self.old_paint_artifact_ = Some(chunks.artifact().clone());
        self.current_paint_artifact_ = None;
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:396-418;
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:53-55,169-172.
    pub fn UpdateForRasterInducingScroll(&mut self, chunks: &M::Subset, m: &M) {
        self.old_paint_chunks_info_.clear();
        Self::PopulatePaintChunksInfo(
            self.layer_bounds_,
            chunks,
            self.layer_offset_,
            &self.layer_state_,
            &mut self.old_paint_chunks_info_,
            m,
        );
    }
    fn PopulatePaintChunksInfo(
        bounds: Size,
        chunks: &M::Subset,
        offset: Vector2dF,
        state: &M::PropertyTreeState,
        infos: &mut Vec<PaintChunkInfo<M>>,
        m: &M,
    ) {
        debug_assert!(infos.is_empty());
        let mut mapper = ChunkToLayerMapper::new(m, state, offset);
        for index in chunks.indices() {
            let chunk = chunks.artifact().Chunk(index);
            if Self::ShouldSkipForRasterInvalidation(chunk, state, m) {
                continue;
            }
            mapper.SwitchToChunk(chunk);
            infos.push(PaintChunkInfo::new(
                bounds,
                &mapper,
                chunks.artifact(),
                index,
                m,
            ));
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:420-437;
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:57-65,165-167.
    pub fn SetOldPaintArtifact(&mut self, artifact: Arc<M::Artifact>) {
        self.old_paint_artifact_ = Some(artifact);
    }
    pub fn LayerBounds(&self) -> Size {
        self.layer_bounds_
    }
    pub fn ApproximateUnsharedMemoryUsage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.old_paint_chunks_info_.capacity() * std::mem::size_of::<PaintChunkInfo<M>>()
    }
    pub fn ClearOldStates(&mut self) {
        self.old_paint_artifact_ = None;
        self.old_paint_chunks_info_.clear();
        self.layer_offset_ = Vector2dF::default();
        self.layer_bounds_ = Size::default();
    }
}
struct RasterInvalidatorSink<'a, M: RasterInvalidationTypes> {
    owner: &'a mut RasterInvalidator<M>,
    callback: &'a mut dyn RasterInvalidationCallback,
}
impl<M: RasterInvalidationTypes> RasterInvalidationSink for RasterInvalidatorSink<'_, M> {
    fn ClipByLayerBounds(&self, rect: Rect) -> Rect {
        clip_rect(rect, self.owner.layer_bounds_)
    }
    fn AddRasterInvalidation(
        &mut self,
        rect: Rect,
        id: DisplayItemClientId,
        reason: Reason,
        old: ClientIsOldOrNew,
    ) {
        self.owner
            .AddRasterInvalidation(rect, id, reason, old, self.callback)
    }
}

// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:140.
// ClientIsOldOrNew retains the two source variants in the shared typed boundary.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:153-173.
// ClipByLayerBounds has integer and float specializations; PopulatePaintChunksInfo
// is shared by Generate and the resident metadata refresh entry above.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:174-187.
// Source fields are owned in this resident invalidator; GC tracing is represented
// by the artifact leases. Namespace/include-guard closures map to module scope.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:173.
// Closes the source ShouldSkipForRasterInvalidation function above.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:25.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:38.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:63.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:144.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:185.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:321.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:334.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:348.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:355.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:395.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:419.
// Definition separators have no runtime instructions.
