// Copyright 2015 The Chromium Authors. BSD-style license in LICENSE.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:1-74;
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:1-24.
use super::{
    display_item_raster_invalidator::DisplayItemRasterInvalidator,
    float_clip_rect::FloatClipRect,
};
use crate::adapter::{
    content_layer_client_impl::RasterInvalidationCallback,
    geometry::{Rect, RectF, Size, ToEnclosingRect, Vector2dF},
    paint_geometry::PaintChunkView,
    raster_invalidator::*,
    region_backend::{SkIRect, SkRegionBackend, SkRegionOp},
};
use crate::third_party::blink::renderer::platform::graphics::compositing::chunk_to_layer_mapper::ChunkToLayerMapper;
use foundation::{PaintInvalidationReason as Reason,IsFullPaintInvalidationReason};
use std::sync::Arc;
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:75-123.
struct PaintChunkInfo<M: RasterInvalidationTypes> {
    index_in_paint_artifact: usize,
    #[cfg(debug_assertions)]
    id: <M::Chunk as RasterInvalidationChunk<M>>::Id,
    bounds_in_layer: Rect,
    // Rechrom partial-raster metadata (see GenerateRasterInvalidations). A
    // moved cached chunk carries the same immutable rects forward, so they are
    // shared rather than reallocated per frame; `None` when there are none.
    opaque_backing_rects_in_layer: Option<Arc<[Rect]>>,
    chunk_to_layer_clip: FloatClipRect,
    chunk_to_layer_transform: M::Transform,
}
impl<M: RasterInvalidationTypes> PaintChunkInfo<M> {
    fn new(
        bounds: Size,
        mapper: &ChunkToLayerMapper<'_, M>,
        access: &ArtifactAccess,
        artifact: &M::Artifact,
        index: usize,
        m: &M,
    ) -> Self {
        let chunk = artifact.Chunk(access, index);
        let mut opaque_backing_rects_in_layer = Vec::new();
        for item in artifact.DisplayItemsInChunk(access, index) {
            if !item.DrawsContent() || item.RectKnownToBeOpaque().IsEmpty() {
                continue;
            }
            let visual = clip_rect(mapper.MapVisualRect(item.VisualRect()), bounds);
            if visual.IsEmpty() {
                continue;
            }
            opaque_backing_rects_in_layer.push(visual);
        }
        let mut result = Self {
            index_in_paint_artifact: index,
            #[cfg(debug_assertions)]
            id: chunk.id(),
            bounds_in_layer: clip_rect(mapper.MapVisualRect(chunk.drawable_bounds()), bounds),
            opaque_backing_rects_in_layer: (!opaque_backing_rects_in_layer.is_empty())
                .then(|| Arc::from(opaque_backing_rects_in_layer)),
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
    fn FromOld(
        old: &Self,
        _access: &ArtifactAccess,
        _artifact: &M::Artifact,
        index: usize,
    ) -> Self {
        #[cfg(debug_assertions)]
        debug_assert!(_artifact.Chunk(_access, index).id() == old.id);
        Self {
            index_in_paint_artifact: index,
            #[cfg(debug_assertions)]
            id: _artifact.Chunk(_access, index).id(),
            bounds_in_layer: old.bounds_in_layer,
            opaque_backing_rects_in_layer: old.opaque_backing_rects_in_layer.clone(),
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
    // Rust allocation reuse for the per-Generate locals of
    // raster_invalidator.cc:193,356-394 (`matched_old_chunks` and
    // `new_chunks_info`, which the source swaps into old_paint_chunks_info_):
    // the previous frame's buffers are kept empty here, so steady-state
    // frames do not allocate. They never carry values between calls.
    spare_paint_chunks_info_: Vec<PaintChunkInfo<M>>,
    matched_old_chunks_: Vec<bool>,
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
fn find_smallest_local_opaque_backing(
    backings: &[Rect],
    rect: Rect,
    layer_bounds: Size,
) -> Option<RoundedOpaqueBacking> {
    let full_layer = Rect::new(0, 0, layer_bounds.width(), layer_bounds.height());
    let outer = backings
        .iter()
        .copied()
        // A layer-filling background is not the local rounded backing whose
        // antialias coverage is shared with the changed descendant.
        .filter(|backing| *backing != full_layer && backing.Contains(rect))
        .min_by_key(|backing| i64::from(backing.width) * i64::from(backing.height))?;
    let outer_area = i64::from(outer.width) * i64::from(outer.height);
    let rect_area = i64::from(rect.width) * i64::from(rect.height);
    let has_overlapping_sibling = backings.iter().copied().any(|candidate| {
        if candidate == outer || candidate == full_layer {
            return false;
        }
        let candidate_area = i64::from(candidate.width) * i64::from(candidate.height);
        if candidate_area * 2 < outer_area || candidate_area > outer_area * 2 {
            return false;
        }
        let mut intersection = candidate;
        intersection.Intersect(rect);
        !intersection.IsEmpty()
            && i64::from(intersection.width) * i64::from(intersection.height) * 5
                >= rect_area * 4
    });
    Some(RoundedOpaqueBacking {
        rect: outer,
        has_overlapping_sibling,
    })
}
impl<M: RasterInvalidationTypes> RasterInvalidator<M> {
    pub fn new(m: &M) -> Self {
        Self {
            layer_offset_: Vector2dF::default(),
            layer_bounds_: Size::default(),
            layer_state_: m.RootState(),
            old_paint_chunks_info_: Vec::new(),
            spare_paint_chunks_info_: Vec::new(),
            matched_old_chunks_: Vec::new(),
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
    #[inline(always)]
    fn GetOldChunk<'a>(&'a self, access: &'a ArtifactAccess, index: usize) -> &'a M::Chunk {
        let info = &self.old_paint_chunks_info_[index];
        let chunk = self
            .old_paint_artifact_
            .as_ref()
            .expect("source old artifact")
            .Chunk(access, info.index_in_paint_artifact);
        #[cfg(debug_assertions)]
        debug_assert!(chunk.id() == info.id);
        chunk
    }
    #[inline(always)]
    fn MatchNewChunkToOldChunk(
        &self,
        access: &ArtifactAccess,
        new: &M::Chunk,
        index: usize,
    ) -> Option<usize> {
        if !new.CanMatchOldChunk() {
            return None;
        }
        for i in index..self.old_paint_chunks_info_.len() {
            if new.Matches(self.GetOldChunk(access, i)) {
                return Some(i);
            }
        }
        None
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:64-143;
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:154-163.
    #[inline(always)]
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
        // One proof for the whole pass over the old and new artifacts.
        let access = ::paint::artifact_access();
        let access = &*access;
        let mut mapper = ChunkToLayerMapper::new(m, &self.layer_state_, self.layer_offset_);
        // Partial raster starts from the previous frame's pixels. Preserve
        // the old opaque backing metadata across cached/moved chunks so a
        // changed descendant in a separate rounded chunk can replay the
        // smallest enclosing backing with identical antialias coverage. Only
        // a rounded-clip display-item invalidation consults it, so it is
        // collected on first use; the old infos' opaque rects are not
        // modified by this loop, so the lazily collected value is the same.
        let old_opaque_backings = OldOpaqueBackings {
            offset_changed,
            rects: std::cell::OnceCell::new(),
        };
        let mut matched = std::mem::take(&mut self.matched_old_chunks_);
        matched.clear();
        matched.resize(self.old_paint_chunks_info_.len(), false);
        let mut old_index = 0;
        let translation = 1e-2f32;
        let other = 1e-4f32;
        for index in chunks.indices() {
            let new = chunks.artifact().Chunk(access, index);
            if Self::ShouldSkipForRasterInvalidation(new, &self.layer_state_, m) {
                continue;
            }
            let Some(matched_index) = self.MatchNewChunkToOldChunk(access, new, old_index) else {
                mapper.SwitchToChunk(new.properties_unalias(), new.raster_effect_outset());
                new_infos.push(PaintChunkInfo::new(
                    self.layer_bounds_,
                    &mapper,
                    access, chunks.artifact(),
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
                chunks.artifact().DisplayItemsInChunk(access, index),
            ) {
                Reason::kScrollControl
            } else {
                Reason::kNone
            };
            // Chromium's PaintRecord keeps ImageResourceContent identity live,
            // so a moved cached subsequence observes a completed image without
            // replacing its record. Rechrom publishes immutable resource
            // snapshots across the Paint -> Compositor boundary. Compare the
            // exact per-record revision proof here before taking Chromium's
            // moved-subsequence shortcut; otherwise the old (often
            // provisional/empty) cc display list survives after the image
            // record becomes available.
            let old_chunk_index =
                self.old_paint_chunks_info_[matched_index].index_in_paint_artifact;
            let resource_changed = chunks.artifact().ChunkHasResourceChange(index)
                || self
                    .old_paint_artifact_
                    .as_ref()
                    .expect("source old artifact")
                    .ImageContentRevisionsInChunk(old_chunk_index)
                    != chunks
                        .artifact()
                        .ImageContentRevisionsInChunk(index);
            if !offset_changed
                && reason == Reason::kNone
                && new.is_moved_from_cached_subsequence()
                && !resource_changed
                && !m.ChunkPropertiesChangedOnlyComposited(new, &self.layer_state_)
            {
                new_infos.push(PaintChunkInfo::FromOld(
                    &self.old_paint_chunks_info_[matched_index],
                    access, chunks.artifact(),
                    index,
                ));
            } else {
                mapper.SwitchToChunk(new.properties_unalias(), new.raster_effect_outset());
                new_infos.push(PaintChunkInfo::new(
                    self.layer_bounds_,
                    &mapper,
                    access, chunks.artifact(),
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
                            self.GetOldChunk(access, matched_index),
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
                    // Source: raster_invalidator.cc:293-301 runs the display
                    // item comparison only for a different PaintArtifact
                    // object: an artifact Paint did not re-record stays the
                    // same object, so the invalidation reasons recorded with
                    // its last repaint are never replayed. Paint keeps that
                    // same Arc for a frame without repaint; it versions an
                    // unchanged recording under a new Arc only for a value
                    // outside the recording (a PrePaint property-tree update
                    // generation, a retained scroll step's chunk flags),
                    // keeping the recording identity, which is therefore the
                    // fallback same-artifact proof here. A resource change
                    // still compares the display items, as before.
                    let same_artifact = Arc::ptr_eq(chunks.artifact(), &old_artifact);
                    let recording_revision = chunks.artifact().RecordingRevision();
                    let same_recording = same_artifact
                        || (recording_revision != 0
                            && recording_revision == old_artifact.RecordingRevision());
                    if !same_artifact
                        && (resource_changed
                            || (!same_recording && !new.is_moved_from_cached_subsequence()))
                    {
                        let old_chunk_index =
                            self.old_paint_chunks_info_[matched_index].index_in_paint_artifact;
                        let mut sink = RasterInvalidatorSink {
                            owner: self,
                            callback,
                            old_opaque_backings: &old_opaque_backings,
                        };
                        DisplayItemRasterInvalidator::new(
                            &mut sink,
                            old_artifact.DisplayItemsInChunk(access, old_chunk_index),
                            chunks.artifact().DisplayItemsInChunk(access, index),
                            &mapper,
                        )
                        .WithImageContentRevisions(
                            old_artifact.ImageContentRevisionsInChunk(old_chunk_index),
                            chunks
                                .artifact()
                                .ImageContentRevisionsInChunk(index),
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
            let old = self.GetOldChunk(access, i);
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
        self.matched_old_chunks_ = matched;
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.cc:322-333;
    // Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidator.h:135-139.
    #[inline(always)]
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
        let mut new_infos = std::mem::take(&mut self.spare_paint_chunks_info_);
        debug_assert!(new_infos.is_empty());
        new_infos.reserve(chunks.len());
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
                let client_id = chunks
                    .artifact()
                    .Chunk(&::paint::artifact_access(), first)
                    .client_id();
                self.AddRasterInvalidation(
                    Rect::new(0, 0, bounds.width(), bounds.height()),
                    client_id,
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
        let mut previous = std::mem::replace(&mut self.old_paint_chunks_info_, new_infos);
        previous.clear();
        self.spare_paint_chunks_info_ = previous;
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
        let access = ::paint::artifact_access();
        let access = &*access;
        let mut mapper = ChunkToLayerMapper::new(m, state, offset);
        for index in chunks.indices() {
            let chunk = chunks.artifact().Chunk(access, index);
            if Self::ShouldSkipForRasterInvalidation(chunk, state, m) {
                continue;
            }
            mapper.SwitchToChunk(chunk.properties_unalias(), chunk.raster_effect_outset());
            infos.push(PaintChunkInfo::new(
                bounds,
                &mapper,
                access, chunks.artifact(),
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
            + (self.old_paint_chunks_info_.capacity() + self.spare_paint_chunks_info_.capacity())
                * std::mem::size_of::<PaintChunkInfo<M>>()
            + self.matched_old_chunks_.capacity()
    }
    pub fn ClearOldStates(&mut self) {
        self.old_paint_artifact_ = None;
        self.old_paint_chunks_info_.clear();
        self.layer_offset_ = Vector2dF::default();
        self.layer_bounds_ = Size::default();
    }
}
// The previous frame's opaque backing rects in layer space, collected from
// `old_paint_chunks_info_` on first use within one GenerateRasterInvalidations.
struct OldOpaqueBackings {
    offset_changed: bool,
    rects: std::cell::OnceCell<Vec<Rect>>,
}
impl OldOpaqueBackings {
    fn Get<M: RasterInvalidationTypes>(&self, owner: &RasterInvalidator<M>) -> &[Rect] {
        self.rects.get_or_init(|| {
            if self.offset_changed {
                return Vec::new();
            }
            owner
                .old_paint_chunks_info_
                .iter()
                .flat_map(|info| info.opaque_backing_rects_in_layer.iter())
                .flat_map(|rects| rects.iter().copied())
                .map(|rect| clip_rect(rect, owner.layer_bounds_))
                .filter(|rect| !rect.IsEmpty())
                .collect()
        })
    }
}
struct RasterInvalidatorSink<'a, M: RasterInvalidationTypes> {
    owner: &'a mut RasterInvalidator<M>,
    callback: &'a mut dyn RasterInvalidationCallback,
    old_opaque_backings: &'a OldOpaqueBackings,
}
impl<M: RasterInvalidationTypes> RasterInvalidationSink for RasterInvalidatorSink<'_, M> {
    fn ClipByLayerBounds(&self, rect: Rect) -> Rect {
        clip_rect(rect, self.owner.layer_bounds_)
    }
    fn FindRoundedOpaqueBacking(&self, rect: Rect) -> Option<RoundedOpaqueBacking> {
        find_smallest_local_opaque_backing(
            self.old_opaque_backings.Get(&*self.owner),
            rect,
            self.owner.layer_bounds_,
        )
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
    fn AddRasterInvalidationForFullReplay(
        &mut self,
        rect: Rect,
        inner: Rect,
        replay_rect: Rect,
        id: DisplayItemClientId,
        reason: Reason,
        old: ClientIsOldOrNew,
    ) {
        self.callback
            .InvalidateRectForFullReplay(rect, inner, replay_rect);
        if self.owner.tracking_.is_some() {
            self.owner.TrackRasterInvalidation(rect, id, reason, old);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::commit::skia_region::SkRegion;
    use crate::runtime::{
        paint::{
            CpuGeometryMapper, CpuPaintRecord, CpuRasterTrackingServices, CpuTrackingBitmap,
            PaintArtifact, PaintChunk, PaintChunkSubset, PropertyTreeState,
        },
    };
    use paint::{
        display_item_id::{DisplayItemId, DisplayItemIdType},
        paint_engine::PaintRect,
    };

    #[derive(Default)]
    struct TrackingRuntime;

    impl CpuRasterTrackingServices<SkRegion> for TrackingRuntime {
        fn PaintUnderInvalidationCheckingEnabled(&self) -> bool {
            false
        }

        fn IsTracingRasterInvalidations(&self) -> bool {
            false
        }

        fn Rasterize(
            &self,
            _: &CpuPaintRecord<SkRegion>,
            _: Rect,
        ) -> Option<CpuTrackingBitmap> {
            unreachable!("under-invalidation checking is disabled")
        }

        fn BuildUnderInvalidationRecord(
            &self,
            _: CpuPaintRecord<SkRegion>,
            _: CpuTrackingBitmap,
            _: Rect,
        ) -> CpuPaintRecord<SkRegion> {
            unreachable!("under-invalidation checking is disabled")
        }

        fn LogUnderInvalidation(&self, _: &str, _: i32, _: i32, _: u32, _: u32) {}

        fn LogMoreUnderInvalidations(&self) {}
    }

    #[derive(Default)]
    struct Callback(Vec<Rect>);

    impl RasterInvalidationCallback for Callback {
        fn InvalidateRect(&mut self, rect: Rect) {
            self.0.push(rect);
        }
    }

    fn mapper() -> CpuGeometryMapper<SkRegion> {
        CpuGeometryMapper::new(false, Arc::new(TrackingRuntime))
    }

    fn chunk(client_id: u64, bounds: PaintRect) -> PaintChunk {
        PaintChunk {
            id: DisplayItemId {
                client_id,
                r#type: DisplayItemIdType::kLayerChunk,
                fragment: 0,
            },
            is_cacheable: true,
            bounds,
            drawable_bounds: bounds,
            ..PaintChunk::default()
        }
    }

    fn artifact(chunks: Vec<PaintChunk>) -> Arc<PaintArtifact> {
        let mut artifact = PaintArtifact::default();
        // Debug names are recorded only with paint debug info enabled
        // (paint_controller.cc:609-611), which DCHECK builds imply.
        artifact.debug_info_enabled.0 = true;
        for chunk in &chunks {
            artifact.RecordDebugInfo(
                chunk.id.client_id as usize,
                Arc::<str>::from(format!("client-{}", chunk.id.client_id)),
                0,
            );
        }
        artifact.chunks = chunks.into();
        Arc::new(artifact)
    }

    fn rect(x: f64, y: f64, width: f64, height: f64) -> PaintRect {
        PaintRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn rounded_backing_search_ignores_layer_filling_background() {
        let bounds = Size::new(100, 100);
        let changed = Rect::new(10, 20, 30, 40);
        let full_layer = Rect::new(0, 0, 100, 100);
        assert_eq!(
            find_smallest_local_opaque_backing(&[full_layer], changed, bounds),
            None
        );
        let local = Rect::new(9, 19, 32, 42);
        assert_eq!(
            find_smallest_local_opaque_backing(&[full_layer, local], changed, bounds),
            Some(RoundedOpaqueBacking {
                rect: local,
                has_overlapping_sibling: false,
            })
        );
        let sibling = Rect::new(7, 20, 32, 40);
        assert_eq!(
            find_smallest_local_opaque_backing(
                &[full_layer, local, sibling],
                changed,
                bounds,
            ),
            Some(RoundedOpaqueBacking {
                rect: local,
                has_overlapping_sibling: true,
            })
        );
    }

    fn run(
        invalidator: &mut RasterInvalidator<CpuGeometryMapper<SkRegion>>,
        artifact: Arc<PaintArtifact>,
        mapper: &CpuGeometryMapper<SkRegion>,
        callback: &mut Callback,
    ) {
        invalidator.Generate(
            &PaintChunkSubset::from_artifact(artifact),
            Vector2dF::default(),
            Size::new(100, 100),
            &PropertyTreeState::default(),
            mapper,
            callback,
        );
    }

    #[test]
    fn first_nonempty_layer_invalidates_the_full_layer() {
        let mapper = mapper();
        let mut invalidator = RasterInvalidator::new(&mapper);
        invalidator.SetTracksRasterInvalidations(true, &mapper);
        let mut callback = Callback::default();

        run(
            &mut invalidator,
            artifact(vec![chunk(1, rect(10., 20., 30., 40.))]),
            &mapper,
            &mut callback,
        );

        assert_eq!(callback.0, [Rect::new(0, 0, 100, 100)]);
        let tracked = invalidator.GetTracking().unwrap().Invalidations();
        assert_eq!(tracked.len(), 1);
        assert_eq!(tracked[0].reason, Reason::kFullLayer);
        assert_eq!(tracked[0].client_debug_name, "client-1");
    }

    #[test]
    fn reordered_chunks_follow_source_appeared_then_disappeared_order() {
        let mapper = mapper();
        let mut invalidator = RasterInvalidator::new(&mapper);
        let mut callback = Callback::default();
        run(
            &mut invalidator,
            artifact(vec![
                chunk(1, rect(0., 0., 10., 10.)),
                chunk(2, rect(10., 0., 10., 10.)),
                chunk(3, rect(20., 0., 10., 10.)),
            ]),
            &mapper,
            &mut callback,
        );

        invalidator.SetTracksRasterInvalidations(true, &mapper);
        callback.0.clear();
        run(
            &mut invalidator,
            artifact(vec![
                chunk(1, rect(0., 0., 10., 10.)),
                chunk(3, rect(20., 0., 10., 10.)),
                chunk(2, rect(10., 0., 10., 10.)),
            ]),
            &mapper,
            &mut callback,
        );

        let tracked = invalidator.GetTracking().unwrap().Invalidations();
        assert_eq!(tracked.len(), 2);
        assert_eq!(tracked[0].client_id, 2);
        assert_eq!(tracked[0].reason, Reason::kChunkAppeared);
        assert_eq!(tracked[1].client_id, 2);
        assert_eq!(tracked[1].reason, Reason::kChunkDisappeared);
    }

    #[test]
    fn effectively_invisible_chunk_is_reported_as_disappeared() {
        let mapper = mapper();
        let mut invalidator = RasterInvalidator::new(&mapper);
        let mut callback = Callback::default();
        run(
            &mut invalidator,
            artifact(vec![chunk(1, rect(10., 20., 30., 40.))]),
            &mapper,
            &mut callback,
        );

        invalidator.SetTracksRasterInvalidations(true, &mapper);
        callback.0.clear();
        let mut changed = chunk(1, rect(20., 30., 30., 40.));
        changed.effectively_invisible = true;
        run(
            &mut invalidator,
            artifact(vec![changed]),
            &mapper,
            &mut callback,
        );

        assert_eq!(callback.0, [Rect::new(10, 20, 30, 40)]);
        let tracked = invalidator.GetTracking().unwrap().Invalidations();
        assert_eq!(tracked.len(), 1);
        assert_eq!(tracked[0].reason, Reason::kChunkDisappeared);
    }

    #[test]
    fn foreign_layer_chunks_own_their_raster_invalidation() {
        let mapper = mapper();
        let mut invalidator = RasterInvalidator::new(&mapper);
        let mut callback = Callback::default();
        let mut foreign = chunk(1, rect(10., 20., 30., 40.));
        foreign.id.r#type = DisplayItemIdType::kForeignLayerVideo;

        run(
            &mut invalidator,
            artifact(vec![foreign.clone()]),
            &mapper,
            &mut callback,
        );

        invalidator.SetTracksRasterInvalidations(true, &mapper);
        callback.0.clear();
        foreign.drawable_bounds = rect(20., 30., 40., 50.);
        foreign.bounds = foreign.drawable_bounds;
        run(
            &mut invalidator,
            artifact(vec![foreign]),
            &mapper,
            &mut callback,
        );

        assert!(callback.0.is_empty());
        assert!(invalidator
            .GetTracking()
            .unwrap()
            .Invalidations()
            .is_empty());
    }

    #[test]
    fn moved_cached_subsequence_reuses_old_chunk_info() {
        let mapper = mapper();
        let mut invalidator = RasterInvalidator::new(&mapper);
        let mut callback = Callback::default();
        run(
            &mut invalidator,
            artifact(vec![chunk(1, rect(10., 20., 30., 40.))]),
            &mapper,
            &mut callback,
        );

        invalidator.SetTracksRasterInvalidations(true, &mapper);
        callback.0.clear();
        let mut moved = chunk(1, rect(10., 20., 30., 40.));
        moved.is_moved_from_cached_subsequence = true;
        run(
            &mut invalidator,
            artifact(vec![moved]),
            &mapper,
            &mut callback,
        );

        assert!(callback.0.is_empty());
        assert!(invalidator
            .GetTracking()
            .unwrap()
            .Invalidations()
            .is_empty());
    }
}
