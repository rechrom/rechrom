// Translation status: complete
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.h:1-28,45-81.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1-39.
// Includes, namespace and static-only declarations map to this Rust module.
use super::chunk_to_layer_mapper::ChunkToLayerMapper;
use crate::{
    adapter::{
        content_layer_client_impl::{
            ContentDisplayItemList, ContentLayerClientTypes, ContentPaintOpBuffer,
        },
        geometry::{Point, PointF, Rect, RectF, ToEnclosingRect, Vector2dF},
        input_types::ElementId,
        layer_tree_impl::SelectionBoundType,
        main_layer::LayerContext,
        paint_chunks_to_cc_layer::*,
        paint_geometry::ChunkGeometryMapper,
        pending_layer::{
            PendingClipNode, PendingEffectNode, PendingPaintChunk, PendingPaintChunkSubset,
            PendingPropertyTreeState, PendingTransformNode,
        },
        property_types::BlendMode,
    },
    cc::{
        base::region::Region,
        input::{
            hit_test_opaqueness::HitTestOpaqueness,
            layer_selection_bound::{LayerSelection, LayerSelectionBound},
            touch_action::TouchAction,
        },
        layers::{
            layer::LayerRef, scroll_hit_test_rect::ScrollHitTestRect,
            touch_action_region::TouchActionRegion,
        },
    },
};
use std::sync::Arc;
const K_NOT_FOUND: usize = usize::MAX;

// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.h:29-43.
pub struct RasterUnderInvalidationCheckingParams<'a, B: ContentLayerClientTypes> {
    pub tracking: &'a mut B::RasterInvalidationTracking,
    pub interest_rect: Rect,
    pub debug_name: String,
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:41-51.
fn IsInCanvasChild<B: PaintChunksTypes>(starting: &B::EffectNode) -> bool {
    let mut effect = Some(starting.clone());
    while let Some(node) = effect {
        if node.HasDirectCompositingReasons() {
            return node.RequiresCompositingForCanvasChild();
        }
        effect = node.UnaliasedParent();
    }
    false
}

// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:53-67,371-374.
// Result is the C++ template concept: the two implementations below write the
// actual resident list/buffer and specialize only source-defined bookkeeping.
trait ConversionResult<B: PaintChunksTypes> {
    const DISPLAY_LIST: bool;
    fn StartPaint(&mut self);
    fn EndPaintOfUnpaired(&mut self, rect: Rect);
    fn EndPaintOfPairedBegin(&mut self);
    fn EndPaintOfPairedEnd(&mut self);
    fn push(&mut self, op: ConversionOp<B>) -> usize;
    fn UpdateSaveLayerBounds(&mut self, id: usize, bounds: RectF);
    fn PushDrawScrollingContentsOp(
        &mut self,
        element: ElementId,
        list: Arc<B::ContentDisplayItemList>,
        rect: Rect,
    );
}
struct DisplayItemListResult<'a, B: PaintChunksTypes>(&'a mut B::ContentDisplayItemList);
impl<B: PaintChunksTypes> ConversionResult<B> for DisplayItemListResult<'_, B> {
    const DISPLAY_LIST: bool = true;
    fn StartPaint(&mut self) {
        self.0.StartPaint();
    }
    fn EndPaintOfUnpaired(&mut self, r: Rect) {
        self.0.EndPaintOfUnpaired(r);
    }
    fn EndPaintOfPairedBegin(&mut self) {
        self.0.EndPaintOfPairedBegin();
    }
    fn EndPaintOfPairedEnd(&mut self) {
        self.0.EndPaintOfPairedEnd();
    }
    fn push(&mut self, op: ConversionOp<B>) -> usize {
        self.0.Push(op)
    }
    fn UpdateSaveLayerBounds(&mut self, id: usize, r: RectF) {
        self.0.UpdateSaveLayerBounds(id, r);
    }
    fn PushDrawScrollingContentsOp(
        &mut self,
        id: ElementId,
        list: Arc<B::ContentDisplayItemList>,
        r: Rect,
    ) {
        self.0.PushDrawScrollingContentsOp(id, list, r);
    }
}
struct PaintOpBufferExt<'a, B: PaintChunksTypes>(&'a mut B::PaintOpBuffer);
impl<B: PaintChunksTypes> ConversionResult<B> for PaintOpBufferExt<'_, B> {
    const DISPLAY_LIST: bool = false;
    fn StartPaint(&mut self) {}
    fn EndPaintOfUnpaired(&mut self, _: Rect) {}
    fn EndPaintOfPairedBegin(&mut self) {}
    fn EndPaintOfPairedEnd(&mut self) {}
    fn push(&mut self, op: ConversionOp<B>) -> usize {
        let offset = self.0.next_op_offset();
        self.0.Push(op);
        offset
    }
    fn UpdateSaveLayerBounds(&mut self, id: usize, bounds: RectF) {
        self.0.UpdateSaveLayerBounds(id, bounds);
    }
    fn PushDrawScrollingContentsOp(
        &mut self,
        _: ElementId,
        _: Arc<B::ContentDisplayItemList>,
        _: Rect,
    ) {
        unreachable!("source PaintOpBuffer specialization never records scrolling contents");
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:69-86.
enum ScrollTranslationAction<T> {
    None,
    Start(T),
    End,
}
impl<T> ScrollTranslationAction<T> {
    fn is_some(&self) -> bool {
        !matches!(self, Self::None)
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:88-134.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PairedType {
    Clip,
    ClipOmitted,
    Effect,
    EffectContents,
}
struct StateEntry<B: PaintChunksTypes> {
    transform: B::TransformNode,
    clip: B::ClipNode,
    effect: B::EffectNode,
    previous_transform: Option<B::TransformNode>,
    #[cfg(debug_assertions)]
    has_effect_hierarchy_issue: bool,
    type_: PairedType,
}
impl<B: PaintChunksTypes> StateEntry<B> {
    fn IsClip(&self) -> bool {
        self.type_ != PairedType::Effect && self.type_ != PairedType::EffectContents
    }
    fn IsEffect(&self) -> bool {
        !self.IsClip()
    }
    fn IsEffectContents(&self) -> bool {
        self.type_ == PairedType::EffectContents
    }
    fn NeedsRestore(&self) -> bool {
        self.type_ != PairedType::ClipOmitted
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:135-156.
struct EffectBoundsInfo<B: PaintChunksTypes> {
    save_layer_id: usize,
    transform: B::TransformNode,
    bounds: RectF,
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:158-180,235-289,322-335,344-370,376-395.
// The stored node values are native Paint node handles, not node field copies.
// Outer state stack is the current calling context's resident borrowed slice.
struct ConversionContext<'a, B: PaintChunksTypes, R: ConversionResult<B>> {
    state_stack_: Vec<StateEntry<B>>,
    effect_bounds_stack_: Vec<EffectBoundsInfo<B>>,
    chunk_to_layer_mapper_: ChunkToLayerMapper<'a, B::ChunkMapper>,
    translated_for_layer_offset_: bool,
    current_transform_: B::TransformNode,
    current_clip_: B::ClipNode,
    current_effect_: B::EffectNode,
    current_scroll_translation_: B::TransformNode,
    previous_transform_: Option<B::TransformNode>,
    result_: &'a mut R,
    outer_state_stack_: Option<&'a [StateEntry<B>]>,
    services: &'a dyn PaintChunksServices<B>,
}
impl<'a, B: PaintChunksTypes, R: ConversionResult<B>> ConversionContext<'a, B, R> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:163-176.
    fn new(
        state: &B::PropertyTreeState,
        offset: Vector2dF,
        result: &'a mut R,
        outer: Option<&'a [StateEntry<B>]>,
        services: &'a dyn PaintChunksServices<B>,
    ) -> Self {
        let transform = state.TransformNode();
        let scroll = transform.NearestScrollTranslationNode();
        Self {
            state_stack_: Vec::new(),
            effect_bounds_stack_: Vec::new(),
            chunk_to_layer_mapper_: ChunkToLayerMapper::new(services.ChunkMapper(), state, offset),
            translated_for_layer_offset_: false,
            current_transform_: transform,
            current_clip_: state.ClipNode(),
            current_effect_: state.EffectNode(),
            current_scroll_translation_: scroll,
            previous_transform_: None,
            result_: result,
            outer_state_stack_: outer,
            services,
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:182-234.
    fn Convert(&mut self, chunks: &B::PaintChunkSubset, cull: Option<&Rect>) {
        let mut index = 0;
        self.ConvertRange(chunks, &mut index, chunks.len(), cull);
        assert!(index == chunks.len());
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:291-310.
    fn ApplyTransform(&mut self, target: &B::TransformNode) {
        if target == &self.current_transform_ {
            return;
        }
        let mut projection = self.services.IdentityTransform();
        let valid = self.TargetToCurrentProjection(target, &mut projection);
        if !valid {
            self.result_.push(ConversionOp::ClipRect {
                rect: RectF::default(),
                difference: false,
                antialias: false,
            });
        } else if projection.IsIdentityOr2dTranslation() {
            let translation = projection.To2dTranslation();
            if translation != Vector2dF::default() {
                self.result_
                    .push(ConversionOp::Translate(translation.x(), translation.y()));
            }
        } else {
            self.result_.push(ConversionOp::Concat(projection));
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:311-316.
    fn TargetToCurrentProjection(
        &self,
        target: &B::TransformNode,
        projection: &mut <B::ChunkMapper as ChunkGeometryMapper>::Transform,
    ) -> bool {
        self.services
            .SourceToDestinationProjection(target, &self.current_transform_, projection)
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:318-322.
    fn AppendRestore(&mut self) {
        self.result_.StartPaint();
        self.result_.push(ConversionOp::Restore);
        self.result_.EndPaintOfPairedEnd();
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:336-344.
    fn MostRecentEffectIsChildOfBackdropFilter(&self) -> bool {
        for state in self.state_stack_.iter().rev() {
            if state.IsEffect() {
                return state.IsEffectContents();
            }
        }
        false
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:412-424.
    fn TranslateForLayerOffsetOnce(&mut self) {
        let offset = self.chunk_to_layer_mapper_.LayerOffset();
        if self.translated_for_layer_offset_ || offset == Vector2dF::default() {
            return;
        }
        self.result_.StartPaint();
        self.result_.push(ConversionOp::Save);
        self.result_
            .push(ConversionOp::Translate(-offset.x(), -offset.y()));
        self.result_.EndPaintOfPairedBegin();
        self.translated_for_layer_offset_ = true;
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:397-410.
impl<B: PaintChunksTypes, R: ConversionResult<B>> Drop for ConversionContext<'_, B, R> {
    fn drop(&mut self) {
        while !self.state_stack_.is_empty() {
            if self.state_stack_.last().unwrap().IsEffect() {
                self.EndEffect();
            } else {
                self.EndClip();
            }
        }
        self.EndTransform();
        if self.translated_for_layer_offset_ {
            self.AppendRestore();
        }
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:426-479.
fn CombineClip<B: PaintChunksTypes>(
    clip: &B::ClipNode,
    combined: &mut B::RoundedRect,
    services: &dyn PaintChunksServices<B>,
) -> bool {
    if clip.PixelMovingFilter().is_some() {
        return true;
    }
    let parent = clip.UnaliasedParent().expect("source CHECK parent");
    if parent.ClipPath().is_some() {
        return false;
    }
    let transform = clip.LocalTransformSpaceUnalias();
    let parent_transform = parent.LocalTransformSpaceUnalias();
    if transform != parent_transform {
        if !transform.ParentIs(&parent_transform) || !transform.IsIdentity() {
            return false;
        }
        if services.RasterInducingScrollEnabled() && transform.ScrollNode().is_some() {
            return false;
        }
    }
    let clip_rect = clip.PaintClipRect();
    let clip_rounded = services.RoundedRectIsRounded(&clip_rect);
    let combined_rounded = services.RoundedRectIsRounded(combined);
    if clip_rounded && combined_rounded {
        return false;
    }
    if combined_rounded {
        return services.RectContains(
            services.RoundedRectRect(&clip_rect),
            services.RoundedRectRect(combined),
        );
    }
    if clip_rounded {
        if services.RectContains(
            services.RoundedRectRect(combined),
            services.RoundedRectRect(&clip_rect),
        ) {
            *combined = clip_rect;
            return true;
        }
        return false;
    }
    debug_assert!(!combined_rounded && !clip_rounded);
    let mut intersection = services.RoundedRectRect(combined);
    intersection = services.IntersectRects(intersection, services.RoundedRectRect(&clip_rect));
    *combined = services.MakeRoundedRect(intersection);
    true
}
impl<B: PaintChunksTypes, R: ConversionResult<B>> ConversionContext<'_, B, R> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:481-569.
    fn SwitchToClip(&mut self, target: &B::ClipNode) -> ScrollTranslationAction<B::TransformNode> {
        if target == &self.current_clip_ {
            return ScrollTranslationAction::None;
        }
        {
            let mut lca = target.LowestCommonAncestor(&self.current_clip_);
            let mut clip = self.current_clip_.clone();
            while clip != lca {
                if self.state_stack_.is_empty()
                    && self
                        .outer_state_stack_
                        .is_some_and(|outer| outer.last().is_some_and(StateEntry::IsClip))
                {
                    return ScrollTranslationAction::End;
                }
                if self.state_stack_.last().is_none_or(|s| !s.IsClip()) {
                    #[cfg(debug_assertions)]
                    self.services.LogClipHierarchyIssue(target, &clip);
                    break;
                }
                clip = clip.UnaliasedParent().expect("source clip parent DCHECK");
                let previous_clip = self.state_stack_.last().unwrap().clip.clone();
                if clip == lca {
                    clip = previous_clip.clone();
                    lca = previous_clip.clone();
                }
                if clip == previous_clip {
                    self.EndClip();
                    debug_assert!(self.current_clip_ == clip);
                }
            }
        }
        if target == &self.current_clip_ {
            return ScrollTranslationAction::None;
        }
        let mut pending = Vec::new();
        let mut clip = Some(target.clone());
        while clip.as_ref() != Some(&self.current_clip_) {
            let Some(node) = clip else {
                break;
            };
            clip = node.UnaliasedParent();
            pending.push(node);
        }
        debug_assert!(!pending.is_empty());
        let mut combined = pending.last().unwrap().PaintClipRect();
        let mut lowest = pending.last().unwrap().clone();
        for sub in pending[..pending.len() - 1].iter().rev() {
            if CombineClip::<B>(sub, &mut combined, self.services) {
                lowest = sub.clone();
            } else {
                let action = self.StartClip(&combined, &lowest);
                if action.is_some() {
                    return action;
                }
                combined = sub.PaintClipRect();
                lowest = sub.clone();
            }
        }
        let action = self.StartClip(&combined, &lowest);
        if action.is_some() {
            return action;
        }
        debug_assert!(&self.current_clip_ == target);
        ScrollTranslationAction::None
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:571-609.
    fn StartClip(
        &mut self,
        combined: &B::RoundedRect,
        lowest: &B::ClipNode,
    ) -> ScrollTranslationAction<B::TransformNode> {
        if self.services.RoundedRectRect(combined) == RectF::from(self.services.InfiniteIntRect()) {
            self.PushState(PairedType::ClipOmitted);
        } else {
            let local = lowest.LocalTransformSpaceUnalias();
            if local != self.current_transform_ {
                self.EndTransform();
                let action = self.ComputeScrollTranslationAction(&local);
                if action.is_some() {
                    return action;
                }
            }
            self.result_.StartPaint();
            self.result_.push(ConversionOp::Save);
            self.ApplyTransform(&local);
            let antialias = true;
            if self.services.RoundedRectIsRounded(combined) {
                self.result_
                    .push(ConversionOp::ClipRRect(combined.clone(), antialias));
            } else {
                self.result_.push(ConversionOp::ClipRect {
                    rect: self.services.RoundedRectRect(combined),
                    difference: false,
                    antialias,
                });
            }
            if let Some(path) = lowest.ClipPath() {
                self.result_.push(ConversionOp::ClipPath {
                    path,
                    difference: false,
                    antialias,
                });
            }
            self.result_.EndPaintOfPairedBegin();
            self.PushState(PairedType::Clip);
            self.current_transform_ = local;
        }
        self.current_clip_ = lowest.clone();
        ScrollTranslationAction::None
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:620-687.
    fn SwitchToEffect(
        &mut self,
        target: &B::EffectNode,
    ) -> ScrollTranslationAction<B::TransformNode> {
        if target == &self.current_effect_ {
            return ScrollTranslationAction::None;
        }
        let lca = target.LowestCommonAncestor(&self.current_effect_);
        #[cfg(debug_assertions)]
        let mut hierarchy_issue = false;
        while self.current_effect_ != lca {
            let action = self.EndClips();
            if action.is_some() {
                return action;
            }
            if self.state_stack_.is_empty() {
                #[cfg(debug_assertions)]
                {
                    self.services
                        .LogEffectHierarchyIssue(target, &self.current_effect_);
                    hierarchy_issue = true;
                }
                if !HasRealEffects::<B>(&self.current_effect_, &lca) {
                    break;
                }
                return ScrollTranslationAction::None;
            }
            self.EndEffect();
        }
        let mut pending = Vec::new();
        let mut effect = Some(target.clone());
        while effect.as_ref() != Some(&lca) {
            let Some(node) = effect else {
                break;
            };
            effect = node.UnaliasedParent();
            pending.push(node);
        }
        for sub in pending.iter().rev() {
            #[cfg(debug_assertions)]
            if !hierarchy_issue {
                debug_assert!(Some(&self.current_effect_) == sub.UnaliasedParent().as_ref());
            }
            let action = self.StartEffect(sub);
            if action.is_some() {
                return action;
            }
            #[cfg(debug_assertions)]
            {
                self.state_stack_
                    .last_mut()
                    .unwrap()
                    .has_effect_hierarchy_issue = hierarchy_issue;
                hierarchy_issue = false;
            }
        }
        ScrollTranslationAction::None
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:689-715.
    fn EmitBackdropFilter(&mut self, effect: &B::EffectNode, flags: B::PaintFlags) -> usize {
        let filter = effect
            .BackdropFilter()
            .expect("source DCHECK backdrop filter");
        let path = effect.BackdropFilterBounds();
        let id = self.result_.push(ConversionOp::SaveLayerFilters {
            bounds: self.services.PathBounds(&path),
            filters: Vec::new(),
            backdrop_filter: self.services.BuildImageFilter(filter),
            flags,
        });
        self.result_.push(ConversionOp::Save);
        if let Some(rect) = self.services.PathIsRect(&path) {
            self.result_.push(ConversionOp::ClipRect {
                rect,
                difference: true,
                antialias: false,
            });
        } else {
            self.result_.push(ConversionOp::ClipPath {
                path,
                difference: true,
                antialias: true,
            });
        }
        self.result_.push(ConversionOp::DrawTransparentColorSrc);
        self.result_.push(ConversionOp::Restore);
        id
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:717-836.
    fn StartEffect(&mut self, effect: &B::EffectNode) -> ScrollTranslationAction<B::TransformNode> {
        if let Some(output_clip) = effect.OutputClip() {
            if !self.state_stack_.is_empty()
                && effect.BlendMode() == BlendMode::kDstIn
                && self.MostRecentEffectIsChildOfBackdropFilter()
            {
                let action = self.EndClips();
                if action.is_some() {
                    return action;
                }
                self.EndEffect();
            }
            let action = self.SwitchToClip(&output_clip);
            if action.is_some() {
                return action;
            }
            let action = self.SwitchToTransform(&effect.LocalTransformSpaceUnalias());
            if action.is_some() {
                return action;
            }
        } else {
            let action = self.EndClips();
            if action.is_some() {
                return action;
            }
        }
        let filter = effect.Filter();
        let has_filter = filter.is_some();
        let has_backdrop = effect.BackdropFilter().is_some();
        let has_opacity = effect.Opacity() != 1.0;
        let has_other = effect.BlendMode() != BlendMode::kSrcOver;
        debug_assert!(!has_filter || !(has_opacity || has_other || has_backdrop));
        let mut save_layer_id = K_NOT_FOUND;
        self.result_.StartPaint();
        if !has_filter {
            if has_other {
                let mut flags = self.services.NewPaintFlags();
                self.services.SetBlendMode(&mut flags, effect.BlendMode());
                self.services.SetAlphaf(&mut flags, effect.Opacity());
                if has_backdrop {
                    save_layer_id = self.EmitBackdropFilter(effect, flags);
                } else {
                    save_layer_id = self.result_.push(ConversionOp::SaveLayer(flags));
                }
            } else if has_backdrop {
                let mut flags = self.services.NewPaintFlags();
                self.services.SetAlphaf(&mut flags, effect.Opacity());
                save_layer_id = self.EmitBackdropFilter(effect, flags);
            } else {
                save_layer_id = self
                    .result_
                    .push(ConversionOp::SaveLayerAlpha(effect.Opacity()));
            }
        } else {
            let mut flags = self.services.NewPaintFlags();
            self.services
                .SetImageFilter(&mut flags, self.services.BuildImageFilter(filter.unwrap()));
            save_layer_id = self.result_.push(ConversionOp::SaveLayer(flags));
        }
        self.result_.EndPaintOfPairedBegin();
        debug_assert!(save_layer_id != K_NOT_FOUND);
        let input_clip = self.current_clip_.clone();
        self.PushState(PairedType::Effect);
        self.effect_bounds_stack_.push(EffectBoundsInfo {
            save_layer_id,
            transform: self.current_transform_.clone(),
            bounds: RectF::default(),
        });
        self.current_clip_ = input_clip;
        self.current_effect_ = effect.clone();
        if effect.HasReferenceFilter() && effect.Filter().is_some() {
            let bounds = effect.FilterOutputBounds();
            self.effect_bounds_stack_.last_mut().unwrap().bounds = bounds.into();
            self.result_.StartPaint();
            self.result_
                .EndPaintOfUnpaired(self.chunk_to_layer_mapper_.MapVisualRect(bounds));
        }
        if effect.BackdropFilter().is_some() {
            let bounds = ToEnclosingRect(self.services.PathBounds(&effect.BackdropFilterBounds()));
            self.effect_bounds_stack_.last_mut().unwrap().bounds = bounds.into();
            self.result_.StartPaint();
            self.result_
                .EndPaintOfUnpaired(self.chunk_to_layer_mapper_.MapVisualRect(bounds));
            self.PushState(PairedType::EffectContents);
            self.effect_bounds_stack_.push(EffectBoundsInfo {
                save_layer_id,
                transform: self.current_transform_.clone(),
                bounds: RectF::default(),
            });
            self.result_.StartPaint();
            save_layer_id = self.result_.push(ConversionOp::SaveLayerAlpha(1.0));
            self.result_.EndPaintOfPairedBegin();
        }
        let _ = save_layer_id;
        ScrollTranslationAction::None
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:838-850.
    fn UpdateEffectBounds(&mut self, bounds: RectF, transform: &B::TransformNode) {
        if self.effect_bounds_stack_.is_empty() || bounds.IsEmpty() {
            return;
        }
        let info = self.effect_bounds_stack_.last_mut().unwrap();
        let mut mapped = bounds;
        self.services
            .SourceToDestinationRect(transform, &info.transform, &mut mapped);
        info.bounds = self.services.UnionRects(info.bounds, mapped);
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:852-886.
    fn EndEffect(&mut self) {
        #[cfg(debug_assertions)]
        {
            let state = self.state_stack_.last().unwrap();
            debug_assert!(state.IsEffect());
            if !state.IsEffectContents() && !state.has_effect_hierarchy_issue {
                debug_assert!(
                    self.current_effect_.UnaliasedParent().as_ref() == Some(&state.effect)
                );
            }
            debug_assert!(self.current_clip_ == state.clip);
        }
        debug_assert!(!self.effect_bounds_stack_.is_empty());
        let info = self.effect_bounds_stack_.last().unwrap();
        let mut bounds = info.bounds;
        if self.current_effect_.Filter().is_none() {
            if !bounds.IsEmpty() {
                self.result_
                    .UpdateSaveLayerBounds(info.save_layer_id, bounds);
            }
        } else {
            self.result_
                .UpdateSaveLayerBounds(info.save_layer_id, bounds);
            bounds = self.current_effect_.MapRect(ToEnclosingRect(bounds)).into();
        }
        self.effect_bounds_stack_.pop();
        self.EndTransform();
        let transform = self.current_transform_.clone();
        self.UpdateEffectBounds(bounds, &transform);
        self.PopState();
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:888-899.
    fn EndClips(&mut self) -> ScrollTranslationAction<B::TransformNode> {
        while self.state_stack_.last().is_some_and(StateEntry::IsClip) {
            self.EndClip();
        }
        if self.state_stack_.is_empty()
            && self
                .outer_state_stack_
                .is_some_and(|stack| stack.last().is_some_and(StateEntry::IsClip))
        {
            return ScrollTranslationAction::End;
        }
        ScrollTranslationAction::None
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:901-907.
    fn EndClip(&mut self) {
        debug_assert!(self.state_stack_.last().unwrap().IsClip());
        debug_assert!(self.state_stack_.last().unwrap().effect == self.current_effect_);
        self.EndTransform();
        self.PopState();
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:909-915.
    fn PushState(&mut self, kind: PairedType) {
        self.state_stack_.push(StateEntry {
            transform: self.current_transform_.clone(),
            clip: self.current_clip_.clone(),
            effect: self.current_effect_.clone(),
            previous_transform: self.previous_transform_.clone(),
            type_: kind,
            #[cfg(debug_assertions)]
            has_effect_hierarchy_issue: false,
        });
        self.previous_transform_ = None;
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:917-929.
    fn PopState(&mut self) {
        debug_assert!(self.previous_transform_.is_none());
        if self.state_stack_.last().unwrap().NeedsRestore() {
            self.AppendRestore();
        }
        let state = self.state_stack_.pop().unwrap();
        self.current_transform_ = state.transform;
        self.previous_transform_ = state.previous_transform;
        self.current_clip_ = state.clip;
        self.current_effect_ = state.effect;
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:931-969.
    fn SwitchToTransform(
        &mut self,
        target: &B::TransformNode,
    ) -> ScrollTranslationAction<B::TransformNode> {
        if target == &self.current_transform_ {
            return ScrollTranslationAction::None;
        }
        self.EndTransform();
        if target == &self.current_transform_ {
            return ScrollTranslationAction::None;
        }
        let action = self.ComputeScrollTranslationAction(target);
        if action.is_some() {
            return action;
        }
        let mut projection = self.services.IdentityTransform();
        let valid = self.TargetToCurrentProjection(target, &mut projection);
        if valid && projection.IsIdentity() {
            return ScrollTranslationAction::None;
        }
        self.result_.StartPaint();
        self.result_.push(ConversionOp::Save);
        if !valid {
            self.result_.push(ConversionOp::ClipRect {
                rect: RectF::default(),
                difference: false,
                antialias: false,
            });
        } else if projection.IsIdentityOr2dTranslation() {
            let delta = projection.To2dTranslation();
            self.result_
                .push(ConversionOp::Translate(delta.x(), delta.y()));
        } else {
            self.result_.push(ConversionOp::Concat(projection));
        }
        self.result_.EndPaintOfPairedBegin();
        self.previous_transform_ = Some(self.current_transform_.clone());
        self.current_transform_ = target.clone();
        ScrollTranslationAction::None
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:971-981.
    fn EndTransform(&mut self) {
        let Some(previous) = self.previous_transform_.as_ref() else {
            return;
        };
        self.result_.StartPaint();
        self.result_.push(ConversionOp::Restore);
        self.result_.EndPaintOfPairedEnd();
        self.current_transform_ = previous.clone();
        self.previous_transform_ = None;
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:610-618.
fn HasRealEffects<B: PaintChunksTypes>(current: &B::EffectNode, ancestor: &B::EffectNode) -> bool {
    let mut node = current.clone();
    while &node != ancestor {
        if node.HasRealEffects() {
            return true;
        }
        node = node.UnaliasedParent().expect("source effect ancestor");
    }
    false
}
impl<B: PaintChunksTypes, R: ConversionResult<B>> ConversionContext<'_, B, R> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:982-1030.
    fn EmitDrawScrollingContentsOp(
        &mut self,
        chunks: &B::PaintChunkSubset,
        index: &mut usize,
        end: usize,
        scroll_translation: &B::TransformNode,
    ) {
        assert!(R::DISPLAY_LIST, "source unspecialized NOTREACHED");
        assert!(self.services.RasterInducingScrollEnabled());
        assert!(scroll_translation.ScrollNode().is_some());
        debug_assert!(self.previous_transform_.is_none());
        let container_transform = scroll_translation
            .UnaliasedParent()
            .expect("source scroll translation parent");
        let action = self.SwitchToTransform(&container_transform);
        assert!(!action.is_some());
        let mut scrolling_list = self.services.CreateDisplayItemList();
        let state = self.services.MakeState(
            scroll_translation.clone(),
            self.current_clip_.clone(),
            self.current_effect_.clone(),
        );
        {
            let mut result = DisplayItemListResult::<B>(&mut scrolling_list);
            ConversionContext::new(
                &state,
                Vector2dF::default(),
                &mut result,
                Some(&self.state_stack_),
                self.services,
            )
            .ConvertRange(chunks, index, end, None);
        }
        self.EndTransform();
        scrolling_list.Finalize();
        let overflow = scroll_translation.ScrollNode().unwrap().OverflowClipNode();
        let (clip, rect) = if let Some(clip) = overflow {
            let rect = self.services.RoundedRectRect(&clip.PaintClipRect());
            (clip, rect)
        } else {
            (
                self.chunk_to_layer_mapper_.ChunkState().ClipNode(),
                self.services.InfiniteIntRect().into(),
            )
        };
        let state = self.services.MakeState(
            container_transform.clone(),
            clip,
            self.chunk_to_layer_mapper_.LayerState().EffectNode(),
        );
        let visual_rect = self
            .chunk_to_layer_mapper_
            .MapVisualRectFromState(ToEnclosingRect(rect), &state);
        self.result_.PushDrawScrollingContentsOp(
            scroll_translation
                .ScrollNode()
                .unwrap()
                .GetCompositorElementId(),
            Arc::new(scrolling_list),
            visual_rect,
        );
        self.UpdateEffectBounds(rect, &container_transform);
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:282-289,1032-1082.
    fn ComputeScrollTranslationAction(
        &self,
        target: &B::TransformNode,
    ) -> ScrollTranslationAction<B::TransformNode> {
        // Native primary template returns {}, DisplayItemList specialization
        // runs the exact decision tree below. No runtime adapter policy.
        if !R::DISPLAY_LIST {
            return ScrollTranslationAction::None;
        }
        if !self.services.RasterInducingScrollEnabled() {
            return ScrollTranslationAction::None;
        }
        let target_scroll = target.NearestScrollTranslationNode();
        if target_scroll == self.current_scroll_translation_ {
            return ScrollTranslationAction::None;
        }
        if IsInCanvasChild::<B>(&self.chunk_to_layer_mapper_.ChunkState().EffectNode())
            && target_scroll
                .ScrollNode()
                .is_some_and(|scroll| scroll.CompositedScrollingNotPreferred())
        {
            return ScrollTranslationAction::None;
        }
        let chunk_scroll = self
            .chunk_to_layer_mapper_
            .ChunkState()
            .TransformNode()
            .NearestScrollTranslationNode();
        if !target_scroll.IsAncestorOf(&chunk_scroll) {
            return ScrollTranslationAction::None;
        }
        if target_scroll.ParentScrollTranslationNode().as_ref()
            == Some(&self.current_scroll_translation_)
        {
            return ScrollTranslationAction::Start(target_scroll);
        }
        debug_assert!(target_scroll.IsAncestorOf(&self.current_scroll_translation_));
        ScrollTranslationAction::End
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1084-1110.
    fn HasDrawing(
        &self,
        artifact: &Arc<B::PaintArtifact>,
        chunk: &B::PaintChunk,
        state: &B::PropertyTreeState,
    ) -> bool {
        if state.EffectNode() != self.current_effect_ {
            return true;
        }
        let mut items = artifact.DisplayItemsForChunk(chunk);
        if items.len() == 0 {
            return false;
        }
        if items.len() > 1 {
            return true;
        }
        let item = items.next().unwrap();
        if item.IsDrawing()
            && self.services.RecordIsEmpty(&item.GetPaintRecord(artifact,self.services))
            && state.EffectNode().IsRoot()
        {
            return false;
        }
        true
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1112-1208.
    fn ConvertRange(
        &mut self,
        chunks: &B::PaintChunkSubset,
        index: &mut usize,
        end: usize,
        cull: Option<&Rect>,
    ) {
        // index is the native resident subset iterator position. The Start
        // branch deliberately leaves it unchanged after nested return, exactly
        // canceling native --chunk_it/for-loop ++chunk_it without unsigned wrap.
        while *index != end {
            let chunk = chunks.chunk(*index);
            if chunk.effectively_invisible() {
                *index += 1;
                continue;
            }
            let state = chunk.PropertiesUnalias();
            if !self.HasDrawing(chunks.artifact(), chunk, &state) {
                *index += 1;
                continue;
            }
            self.TranslateForLayerOffsetOnce();
            self.chunk_to_layer_mapper_
                .SwitchToChunkWithState(chunk, &state);
            if let Some(cull) = cull {
                let visual = self
                    .chunk_to_layer_mapper_
                    .MapVisualRect(chunk.drawable_bounds());
                if !cull.Intersects(visual) {
                    *index += 1;
                    continue;
                }
            }
            let mut action = self.SwitchToEffect(&state.EffectNode());
            if !action.is_some() {
                action = self.SwitchToClip(&state.ClipNode());
            }
            if !action.is_some() {
                action = self.SwitchToTransform(&state.TransformNode());
            }
            match action {
                ScrollTranslationAction::Start(scroll) => {
                    self.EmitDrawScrollingContentsOp(chunks, index, end, &scroll);
                    continue;
                }
                ScrollTranslationAction::End if self.outer_state_stack_.is_some() => return,
                _ => {}
            }
            for item in chunks.artifact().DisplayItemsForChunk(chunk) {
                let record = if item.IsScrollbar() {
                    item.PaintScrollbar(chunks.artifact(),self.services)
                } else if item.IsDrawing() {
                    item.GetPaintRecord(chunks.artifact(),self.services)
                } else {
                    continue;
                };
                let can_ignore_record = state.EffectNode().IsRoot();
                if self.services.RecordIsEmpty(&record) && can_ignore_record {
                    continue;
                }
                let rect = self.chunk_to_layer_mapper_.MapVisualRect(item.VisualRect());
                if cull.is_some_and(|c| can_ignore_record && !c.Intersects(rect)) {
                    continue;
                }
                self.result_.StartPaint();
                if !self.services.RecordIsEmpty(&record) {
                    self.result_.push(ConversionOp::DrawRecord(record));
                }
                self.result_.EndPaintOfUnpaired(rect);
            }
            self.UpdateEffectBounds(chunk.drawable_bounds().into(), &state.TransformNode());
            *index += 1;
        }
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1210-1238.
pub fn ConvertInto<B: PaintChunksTypes>(
    chunks: &B::PaintChunkSubset,
    state: &B::PropertyTreeState,
    offset: Vector2dF,
    checking: Option<RasterUnderInvalidationCheckingParams<'_, B>>,
    list: &mut B::ContentDisplayItemList,
    services: &dyn PaintChunksServices<B>,
) {
    {
        let mut result = DisplayItemListResult::<B>(list);
        ConversionContext::new(state, offset, &mut result, None, services).Convert(chunks, None);
    }
    if let Some(params) = checking {
        let mut recorder = services.NewPaintRecorder();
        recorder.beginRecording();
        let mut buffer = services.CreatePaintOpBuffer();
        {
            let mut result = PaintOpBufferExt::<B>(&mut buffer);
            ConversionContext::new(state, offset, &mut result, None, services)
                .Convert(chunks, None);
        }
        recorder.DrawPicture(buffer.ReleaseAsRecord());
        params.tracking.CheckUnderInvalidations(
            &params.debug_name,
            recorder.finishRecordingAsPicture(),
            params.interest_rect,
        );
        let record = params.tracking.UnderInvalidationRecord();
        if !services.RecordIsEmpty(&record) {
            list.StartPaint();
            list.Push(ConversionOp::DrawRecord(record));
            list.EndPaintOfUnpaired(params.interest_rect);
        }
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1240-1247.
pub fn Convert<B: PaintChunksTypes>(
    chunks: &B::PaintChunkSubset,
    state: &B::PropertyTreeState,
    cull: Option<&Rect>,
    services: &dyn PaintChunksServices<B>,
) -> B::PaintRecord {
    let mut buffer = services.CreatePaintOpBuffer();
    {
        let mut result = PaintOpBufferExt::<B>(&mut buffer);
        ConversionContext::new(state, Vector2dF::default(), &mut result, None, services)
            .Convert(chunks, cull);
    }
    buffer.ReleaseAsRecord()
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1249-1265.
struct NonCompositedScroll<B: PaintChunksTypes> {
    scroll_translation: B::TransformNode,
    layer_hit_test_rect: Rect,
    hit_test_opaqueness: HitTestOpaqueness,
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1267-1334.
struct LayerPropertiesUpdater<'a, 'c, B: PaintChunksTypes> {
    chunk_to_layer_mapper_: ChunkToLayerMapper<'a, B::ChunkMapper>,
    layer_: &'a LayerRef<B>,
    chunks_: &'a B::PaintChunkSubset,
    layer_selection_: &'a mut LayerSelection,
    selection_only_: bool,
    layer_scroll_translation_: B::TransformNode,
    touch_action_region_: TouchActionRegion<B::RegionBackend>,
    last_disable_cursor_control_: TouchAction,
    last_disable_cursor_control_scroll_: Option<B::ScrollNode>,
    wheel_event_region_: Region<B::RegionBackend>,
    #[cfg(target_os = "android")]
    xr_hit_test_order_: Vec<ElementId>,
    main_thread_scroll_hit_test_region_: Region<B::RegionBackend>,
    capture_bounds_: B::RegionCaptureBounds,
    tracked_element_rects_: B::TrackedElementRects,
    top_non_composited_scrolls_: Vec<NonCompositedScroll<B>>,
    c: &'a mut LayerContext<'c, B>,
    services: &'a dyn PaintChunksServices<B>,
}
impl<'a, 'c, B: PaintChunksTypes> LayerPropertiesUpdater<'a, 'c, B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1271-1282.
    fn new(
        layer: &'a LayerRef<B>,
        state: &B::PropertyTreeState,
        chunks: &'a B::PaintChunkSubset,
        selection: &'a mut LayerSelection,
        selection_only: bool,
        c: &'a mut LayerContext<'c, B>,
        services: &'a dyn PaintChunksServices<B>,
    ) -> Self {
        Self {
            chunk_to_layer_mapper_: ChunkToLayerMapper::new(
                services.ChunkMapper(),
                state,
                layer.offset_to_transform_parent(c),
            ),
            layer_: layer,
            chunks_: chunks,
            layer_selection_: selection,
            selection_only_: selection_only,
            layer_scroll_translation_: state.TransformNode().NearestScrollTranslationNode(),
            touch_action_region_: TouchActionRegion::new(),
            last_disable_cursor_control_: TouchAction::kNone,
            last_disable_cursor_control_scroll_: None,
            wheel_event_region_: Region::default(),
            #[cfg(target_os = "android")]
            xr_hit_test_order_: Vec::new(),
            main_thread_scroll_hit_test_region_: Region::default(),
            capture_bounds_: Default::default(),
            tracked_element_rects_: Default::default(),
            top_non_composited_scrolls_: Vec::new(),
            c,
            services,
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1336-1364.
    fn ShouldDisableCursorControl(&mut self) -> TouchAction {
        let mut scroll = self
            .chunk_to_layer_mapper_
            .ChunkState()
            .TransformNode()
            .NearestScrollTranslationNode()
            .ScrollNode();
        if scroll == self.last_disable_cursor_control_scroll_ {
            return self.last_disable_cursor_control_;
        }
        self.last_disable_cursor_control_scroll_ = scroll.clone();
        self.last_disable_cursor_control_ = TouchAction::kNone;
        while let Some(node) = scroll {
            if node.UserScrollableHorizontal()
                && node.ContainerRect().width < node.ContentsRect().width
            {
                self.last_disable_cursor_control_ = TouchAction::kInternalPanXScrolls;
                break;
            }
            if !node.PropagatesXScroll() {
                break;
            }
            scroll = node.Parent();
        }
        self.last_disable_cursor_control_
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1366-1384.
    fn UpdateTouchActionRegion(&mut self, data: &B::HitTestData) {
        let mut rects = data.touch_action_rects().peekable();
        if rects.peek().is_none() {
            return;
        }
        for (rect, mut action) in rects {
            let rect = self.chunk_to_layer_mapper_.MapVisualRect(rect);
            if rect.IsEmpty() {
                continue;
            }
            if action & TouchAction::kPanX != TouchAction::kNone {
                action |= self.ShouldDisableCursorControl();
            }
            self.touch_action_region_.Union(action, &rect);
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1386-1392.
    fn UpdateWheelEventRegion(&mut self, data: &B::HitTestData) {
        for rect in data.wheel_event_rects() {
            self.wheel_event_region_
                .Union(&self.chunk_to_layer_mapper_.MapVisualRect(rect));
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1394-1399.
    #[cfg(target_os = "android")]
    fn UpdateXrTargetRegion(&mut self, data: &B::HitTestData) {
        self.xr_hit_test_order_.extend(data.xr_regions());
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1401-1468.
    fn UpdateScrollHitTestData(&mut self, chunk: &B::PaintChunk) {
        let data = chunk.hit_test_data().unwrap();
        if data.scroll_hit_test_rect().IsEmpty() {
            return;
        }
        if let Some(translation) = data.scroll_translation() {
            let node = translation.ScrollNode();
            if self.services.RemoveScrollNodeWorkaroundEnabled() {
                assert!(node.is_some());
            } else {
                debug_assert!(node.is_some());
                if node.is_none() {
                    return;
                }
            }
            let node = node.unwrap();
            let element = node.GetCompositorElementId();
            if !self.services.ScrollingContentsCullRectOnScrollNodeEnabled() {
                let tree = self
                    .c
                    .HostMut(self.layer_.host_id().expect("source layer host"))
                    .property_trees_mut()
                    .scroll_tree_mutable();
                if data.scrolling_contents_cull_rect() == self.services.InfiniteIntRect()
                    || data
                        .scrolling_contents_cull_rect()
                        .Contains(node.ContentsRect())
                {
                    tree.ClearScrollingContentsCullRect(element);
                } else {
                    tree.SetScrollingContentsCullRect(
                        element,
                        &data.scrolling_contents_cull_rect(),
                    );
                }
            }
            if self.layer_.element_id(self.c) == element {
                return;
            }
        }
        if self.services.RasterInducingScrollEnabled() && data.scroll_translation().is_some() {
            if !IsInCanvasChild::<B>(&chunk.PropertiesUnalias().EffectNode())
                || !data
                    .scroll_translation()
                    .unwrap()
                    .ScrollNode()
                    .expect("source scroll node")
                    .CompositedScrollingNotPreferred()
            {
                assert!(chunk.IsScrollHitTest());
                self.AddNonCompositedScroll(chunk);
                return;
            }
        }
        let rect = self
            .chunk_to_layer_mapper_
            .MapVisualRect(data.scroll_hit_test_rect());
        if rect.IsEmpty() {
            return;
        }
        self.main_thread_scroll_hit_test_region_.Union(&rect);
        if chunk.IsScrollbarHitTest() || chunk.IsResizerScrollHitTest() {
            self.touch_action_region_.Union(TouchAction::kNone, &rect);
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1470-1483.
    fn TopNonCompositedScroll(&self, translation: &B::TransformNode) -> B::TransformNode {
        let mut node = Some(translation.clone());
        while let Some(current) = node {
            let parent = current.ParentScrollTranslationNode();
            if parent.as_ref() == Some(&self.layer_scroll_translation_) {
                return current;
            }
            node = parent;
        }
        translation.clone()
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1485-1510.
    fn AddNonCompositedScroll(&mut self, chunk: &B::PaintChunk) {
        debug_assert!(self.services.RasterInducingScrollEnabled());
        let translation = chunk
            .hit_test_data()
            .unwrap()
            .scroll_translation()
            .expect("source scroll translation");
        let top = self.TopNonCompositedScroll(&translation);
        if top == translation {
            let mut opaque = chunk.hit_test_opaqueness();
            if opaque == HitTestOpaqueness::kOpaque
                && !self
                    .services
                    .ClipIsTight(self.chunk_to_layer_mapper_.ClipRect())
            {
                opaque = HitTestOpaqueness::kMixed;
            }
            self.top_non_composited_scrolls_.push(NonCompositedScroll {
                scroll_translation: translation,
                layer_hit_test_rect: self
                    .chunk_to_layer_mapper_
                    .MapVisualRect(chunk.hit_test_data().unwrap().scroll_hit_test_rect()),
                hit_test_opaqueness: opaque,
            });
        } else {
            for scroll in &mut self.top_non_composited_scrolls_ {
                if scroll.scroll_translation == top {
                    scroll.hit_test_opaqueness = HitTestOpaqueness::kMixed;
                    break;
                }
            }
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1511-1604.
    fn UpdatePreviousNonCompositedScrolls(&mut self, chunk: &B::PaintChunk) {
        if self.top_non_composited_scrolls_.is_empty() {
            return;
        }
        debug_assert!(self.services.RasterInducingScrollEnabled());
        if chunk
            .hit_test_data()
            .is_some_and(|data| data.scroll_translation().is_some())
        {
            return;
        }
        if chunk.hit_test_opaqueness() == HitTestOpaqueness::kTransparent {
            return;
        }
        let translation = chunk
            .PropertiesUnalias()
            .TransformNode()
            .NearestScrollTranslationNode();
        if translation == self.layer_scroll_translation_ {
            let rect = self
                .chunk_to_layer_mapper_
                .MapVisualRect(chunk.bounds_rect());
            for previous in self.top_non_composited_scrolls_.iter_mut().rev() {
                if previous.layer_hit_test_rect.Intersects(rect) {
                    previous.hit_test_opaqueness = HitTestOpaqueness::kMixed;
                }
                if previous.layer_hit_test_rect.Contains(rect) {
                    break;
                }
            }
            return;
        }
        let top = self.TopNonCompositedScroll(&translation);
        if top != translation {
            return;
        }
        let Some(scroll) = self
            .top_non_composited_scrolls_
            .iter_mut()
            .rev()
            .find(|p| p.scroll_translation == translation)
        else {
            return;
        };
        if scroll.hit_test_opaqueness == HitTestOpaqueness::kTransparent {
            scroll.hit_test_opaqueness = HitTestOpaqueness::kMixed;
        }
        if scroll.hit_test_opaqueness == HitTestOpaqueness::kMixed {
            return;
        }
        let rect = scroll.layer_hit_test_rect;
        for previous in self.top_non_composited_scrolls_.iter_mut().rev() {
            if previous.scroll_translation == translation {
                break;
            }
            if previous.layer_hit_test_rect.Intersects(rect) {
                previous.hit_test_opaqueness = HitTestOpaqueness::kMixed;
            }
            if previous.layer_hit_test_rect.Contains(rect) {
                break;
            }
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1624-1634.
    fn UpdateForNonCompositedScrollbar(&mut self, scrollbar: &B::DisplayItem) {
        let rect = self
            .chunk_to_layer_mapper_
            .MapVisualRect(scrollbar.VisualRect());
        if rect.IsEmpty() {
            return;
        }
        self.main_thread_scroll_hit_test_region_.Union(&rect);
        self.touch_action_region_.Union(TouchAction::kNone, &rect);
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1636-1643.
    fn UpdateRegionCaptureData(&mut self, data: &B::RegionCaptureData) {
        for (id, rect) in data.iter() {
            self.capture_bounds_
                .Set(id, self.chunk_to_layer_mapper_.MapVisualRect(rect));
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1645-1659.
    fn UpdateTrackedElementRects(&mut self, data: &B::TrackedData) {
        for (feature, rect) in data.iter() {
            let bounds = self.chunk_to_layer_mapper_.MapVisualRect(rect.bounds);
            self.tracked_element_rects_.Push(
                feature,
                TrackedRect {
                    bounds,
                    id: rect.id,
                    should_add_to_compositor_frame_metadata: rect
                        .should_add_to_compositor_frame_metadata,
                    should_exclude_fixed_and_sticky_occlusions: rect
                        .should_exclude_fixed_and_sticky_occlusions,
                    frame_token: rect.frame_token,
                    parent_frame_token: rect.parent_frame_token,
                },
            );
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1661-1665.
    fn MapSelectionBoundPoint(&self, point: Point) -> Point {
        self.services.ToRoundedPoint(
            self.chunk_to_layer_mapper_
                .Transform()
                .MapPoint(PointF::new(point.x() as f32, point.y() as f32)),
        )
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1667-1701.
    fn PaintedSelectionBoundToLayerSelectionBound(
        &self,
        bound: PaintedSelectionBound,
    ) -> LayerSelectionBound {
        let mut layer_bound = LayerSelectionBound::default();
        layer_bound.r#type = bound.r#type;
        let mut sample;
        if self.services.SelectionEdgeVisibilityUsesFullEdge() {
            sample = self.services.BoundingRect(bound.edge_start, bound.edge_end);
        } else {
            sample = Rect::new(bound.edge_end.x(), bound.edge_end.y(), 0, 0);
            let offset = |start: i32, end: i32| {
                if start < end {
                    -1
                } else if start > end {
                    1
                } else {
                    0
                }
            };
            self.services.OffsetRect(
                &mut sample,
                offset(bound.edge_start.x(), bound.edge_end.x()),
                offset(bound.edge_start.y(), bound.edge_end.y()),
            );
        }
        self.services.OutsetRect(&mut sample, 1);
        layer_bound.hidden = self.chunk_to_layer_mapper_.MapVisualRect(sample).IsEmpty();
        layer_bound.edge_start = self.MapSelectionBoundPoint(bound.edge_start);
        layer_bound.edge_end = self.MapSelectionBoundPoint(bound.edge_end);
        layer_bound
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1703-1716.
    fn UpdateLayerSelection(&mut self, data: &B::SelectionData) {
        if let Some(start) = data.start() {
            self.layer_selection_.start = self.PaintedSelectionBoundToLayerSelectionBound(start);
            self.layer_selection_.start.layer_id = self.layer_.id();
        }
        if let Some(end) = data.end() {
            self.layer_selection_.end = self.PaintedSelectionBoundToLayerSelectionBound(end);
            self.layer_selection_.end.layer_id = self.layer_.id();
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1718-1800.
    fn Update(&mut self) {
        let mut any_selection_was_painted = false;
        // Copying this borrowed subset reference splits the immutable source
        // input from this updater's mutable state; no chunk/item copy occurs.
        let chunks = self.chunks_;
        for index in 0..chunks.len() {
            let chunk = chunks.chunk(index);
            let scrollbar = NonCompositedScrollbarDisplayItem::<B>(
                chunks.artifact(),
                chunk,
                self.layer_,
                self.c,
            );
            if (!self.selection_only_
                && (chunk.hit_test_data().is_some()
                    || scrollbar.is_some()
                    || chunk.region_capture_data().is_some()
                    || chunk.tracked_element_rects().is_some()
                    || !self.top_non_composited_scrolls_.is_empty()))
                || chunk.layer_selection_data().is_some()
            {
                self.chunk_to_layer_mapper_.SwitchToChunk(chunk);
            }
            if !self.selection_only_ {
                if let Some(data) = chunk.hit_test_data() {
                    self.UpdateTouchActionRegion(data);
                    self.UpdateWheelEventRegion(data);
                    #[cfg(target_os = "android")]
                    self.UpdateXrTargetRegion(data);
                    self.UpdateScrollHitTestData(chunk);
                }
                self.UpdatePreviousNonCompositedScrolls(chunk);
                if let Some(scrollbar) = scrollbar {
                    self.UpdateForNonCompositedScrollbar(scrollbar);
                }
                if let Some(data) = chunk.region_capture_data() {
                    self.UpdateRegionCaptureData(data);
                }
                if let Some(data) = chunk.tracked_element_rects() {
                    self.UpdateTrackedElementRects(data);
                }
            }
            if let Some(data) = chunk.layer_selection_data() {
                any_selection_was_painted |= data.any_selection_was_painted();
                self.UpdateLayerSelection(data);
            }
        }
        if !self.selection_only_ {
            self.layer_
                .SetTouchActionRegion(std::mem::take(&mut self.touch_action_region_), self.c);
            self.layer_
                .SetWheelEventRegion(std::mem::take(&mut self.wheel_event_region_), self.c);
            #[cfg(target_os = "android")]
            self.layer_
                .SetXrHitTestOrder(std::mem::take(&mut self.xr_hit_test_order_), self.c);
            self.layer_
                .SetCaptureBounds(std::mem::take(&mut self.capture_bounds_), self.c);
            self.layer_
                .SetTrackedElementRects(std::mem::take(&mut self.tracked_element_rects_), self.c);
            let mut hit_rects = Vec::new();
            for scroll in &self.top_non_composited_scrolls_ {
                if scroll.hit_test_opaqueness == HitTestOpaqueness::kMixed {
                    self.main_thread_scroll_hit_test_region_
                        .Union(&scroll.layer_hit_test_rect);
                } else if scroll.hit_test_opaqueness == HitTestOpaqueness::kOpaque {
                    hit_rects.push(ScrollHitTestRect {
                        scroll_element_id: scroll
                            .scroll_translation
                            .ScrollNode()
                            .expect("source scroll node")
                            .GetCompositorElementId(),
                        hit_test_rect: scroll.layer_hit_test_rect,
                    });
                }
            }
            self.layer_.SetMainThreadScrollHitTestRegion(
                &self.main_thread_scroll_hit_test_region_,
                self.c,
            );
            self.layer_
                .SetNonCompositedScrollHitTestRects(hit_rects, self.c);
        }
        if any_selection_was_painted {
            if self.layer_selection_.start.r#type == SelectionBoundType::EMPTY {
                self.layer_selection_.start.r#type = SelectionBoundType::LEFT;
                self.layer_selection_.start.hidden = true;
            }
            if self.layer_selection_.end.r#type == SelectionBoundType::EMPTY {
                self.layer_selection_.end.r#type = SelectionBoundType::RIGHT;
                self.layer_selection_.end.hidden = true;
            }
        }
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1606-1622.
fn NonCompositedScrollbarDisplayItem<'a, B: PaintChunksTypes>(
    artifact: &'a B::PaintArtifact,
    chunk: &'a B::PaintChunk,
    layer: &LayerRef<B>,
    c: &LayerContext<'_, B>,
) -> Option<&'a B::DisplayItem> {
    if chunk.display_item_count() != 1 {
        return None;
    }
    let item = artifact.DisplayItemsForChunk(chunk).next().unwrap();
    if !item.IsScrollbar() {
        return None;
    }
    if item.ScrollbarElementId() == layer.element_id(c) {
        return None;
    }
    Some(item)
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.cc:1802-1819.
pub fn UpdateLayerProperties<B: PaintChunksTypes>(
    layer: &LayerRef<B>,
    state: &B::PropertyTreeState,
    chunks: &B::PaintChunkSubset,
    selection: &mut LayerSelection,
    selection_only: bool,
    c: &mut LayerContext<'_, B>,
    services: &dyn PaintChunksServices<B>,
) {
    LayerPropertiesUpdater::new(layer, state, chunks, selection, selection_only, c, services)
        .Update();
}
