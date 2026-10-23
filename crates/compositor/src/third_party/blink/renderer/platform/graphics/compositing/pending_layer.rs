// Translation status: complete
// Copyright 2021 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:1-31.
// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:32-38.
// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:104-109.
// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:143-145.
// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:286-293.
// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:1-42.
// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:930-931.
// Rust modules replace include guards, includes, exports and namespaces.

use crate::{
    adapter::{
        geometry::{RectF, Size, SizeF, ToEnclosingRect, Vector2dF},
        main_layer::{LayerContext, MainLayerHostId},
        pending_layer::{
            LcdTextPreference, PendingClipNode, PendingDisplayItem, PendingEffectNode,
            PendingLayerColor, PendingLayerGeometry, PendingLayerJson, PendingLayerServices,
            PendingLayerTypes, PendingMappedVisualRect, PendingPaintArtifact, PendingPaintChunk,
            PendingPaintChunkSubset, PendingPropertyTreeState, PendingTransformNode,
            PendingUpcastResult,
        },
        property_types::BlendMode,
    },
    cc::{
        input::hit_test_opaqueness::{
            HitTestOpaqueness, HitTestOpaquenessToString, UnionHitTestOpaqueness,
        },
        layers::layer::{Layer, LayerRef},
    },
};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

const K_NOT_FOUND: usize = usize::MAX;

// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:32-38.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompositingType {
    kScrollHitTestLayer,
    kForeignLayer,
    kScrollbarLayer,
    kOverlap,
    #[default]
    kOther,
}

// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:104-109.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MergeResult {
    pub merged: bool,
    pub scroll_range_dependent: bool,
}

struct PendingMergeState<S> {
    bounds: RectF,
    property_tree_state: S,
    rect_known_to_be_opaque: RectF,
    text_known_to_be_on_opaque_background: bool,
    solid_color_chunk_index: usize,
    hit_test_opaqueness: HitTestOpaqueness,
    scroll_range_dependent: bool,
}

// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:250-284.
// PaintChunkSubset retains the exact Arc<PaintArtifact> and chunk indices.
// Property-tree members are Paint-owned handles. LayerRef is the canonical
// cc::Layer strong owner. No field is a Page/tree/artifact snapshot.
pub struct PendingLayer<B: PendingLayerTypes> {
    chunks_: B::PaintChunkSubset,
    property_tree_state_: B::PropertyTreeState,
    non_composited_scroll_translations_: Vec<B::TransformNode>,
    bounds_: RectF,
    rect_known_to_be_opaque_: RectF,
    solid_color_chunk_index_: usize,
    offset_of_decomposited_transforms_: Vector2dF,
    cc_layer_: Option<LayerRef<B>>,
    content_layer_client_: Option<super::content_layer_client_impl::ContentLayerClientRef<B>>,
    change_of_decomposited_transforms_: B::PaintPropertyChange,
    compositing_type_: CompositingType,
    hit_test_opaqueness_: HitTestOpaqueness,
    merged_across_compositing_boundary_count_: i32,
    canvas_child_id_: B::DomNodeId,
    has_text_: bool,
    draws_content_: bool,
    text_known_to_be_on_opaque_background_: bool,
    has_decomposited_blend_mode_: bool,
}

// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:45;
// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:81-86.
// Rust's resident field ownership traces and retains these exact owners through
// the type graph, replacing Oilpan's explicit Visitor calls.

impl<B: PendingLayerTypes> PendingLayer<B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:40-43;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:44-79.
    pub fn new(
        artifact: Arc<B::PaintArtifact>,
        first_chunk_index: usize,
        canvas_child_id: B::DomNodeId,
        compositing_type: CompositingType,
        geometry: &dyn PendingLayerGeometry<B>,
    ) -> Self {
        let first_chunk = artifact
            .PaintChunk(first_chunk_index)
            .expect("source first PaintChunk belongs to PaintArtifact");
        debug_assert!(!RequiresOwnLayer(compositing_type) || first_chunk.display_item_count() <= 1);
        let property_tree_state = first_chunk.PropertiesUnalias();
        let mut bounds = first_chunk.bounds();
        let mut rect_known_to_be_opaque = first_chunk.rect_known_to_be_opaque();
        let background = first_chunk.background_color();
        let has_text = first_chunk.has_text();
        let text_known_to_be_on_opaque_background =
            first_chunk.text_known_to_be_on_opaque_background();
        debug_assert!(has_text || text_known_to_be_on_opaque_background);
        let mut draws_content = first_chunk.DrawsContent();
        if let Some(visibility_limit) = geometry.VisibilityLimit(&property_tree_state) {
            intersect_rectf(&mut bounds, visibility_limit);
            if bounds.IsEmpty() {
                draws_content = false;
            }
        }
        intersect_rectf(&mut rect_known_to_be_opaque, bounds);
        let non_composited_scroll_translations = if compositing_type == CompositingType::kOther {
            first_chunk
                .non_composited_scroll_translation()
                .into_iter()
                .collect()
        } else {
            Vec::new()
        };
        let hit_test_opaqueness = first_chunk.hit_test_opaqueness();
        let chunks = B::PaintChunkSubset::FromFirst(artifact, first_chunk_index);
        Self {
            chunks_: chunks,
            property_tree_state_: property_tree_state,
            non_composited_scroll_translations_: non_composited_scroll_translations,
            bounds_: bounds,
            rect_known_to_be_opaque_: rect_known_to_be_opaque,
            solid_color_chunk_index_: if background.is_solid_color {
                0
            } else {
                K_NOT_FOUND
            },
            offset_of_decomposited_transforms_: Vector2dF::default(),
            cc_layer_: None,
            content_layer_client_: None,
            change_of_decomposited_transforms_: B::UnchangedPaintPropertyChange(),
            compositing_type_: compositing_type,
            hit_test_opaqueness_: hit_test_opaqueness,
            merged_across_compositing_boundary_count_: 0,
            canvas_child_id_: canvas_child_id,
            has_text_: has_text,
            draws_content_: draws_content,
            text_known_to_be_on_opaque_background_: text_known_to_be_on_opaque_background,
            has_decomposited_blend_mode_: false,
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:47-72.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:140-145.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:201-208.
    pub fn Bounds(&self) -> (Vector2dF, Size) {
        PendingLayerBounds(
            self.bounds_,
            self.rect_known_to_be_opaque_,
            self.IsSolidColor(),
        )
    }
    pub fn BoundsForTesting(&self) -> &RectF {
        &self.bounds_
    }
    pub fn RectKnownToBeOpaque(&self) -> &RectF {
        &self.rect_known_to_be_opaque_
    }
    pub fn TextKnownToBeOnOpaqueBackground(&self) -> bool {
        self.text_known_to_be_on_opaque_background_
    }
    pub fn Chunks(&self) -> &B::PaintChunkSubset {
        &self.chunks_
    }
    pub fn GetPropertyTreeState(&self) -> B::PropertyTreeState {
        self.property_tree_state_.clone()
    }
    pub fn OffsetOfDecompositedTransforms(&self) -> &Vector2dF {
        &self.offset_of_decomposited_transforms_
    }
    pub fn ChangeOfDecompositedTransforms(&self) -> B::PaintPropertyChange {
        self.change_of_decomposited_transforms_
    }
    pub fn GetCompositingType(&self) -> CompositingType {
        self.compositing_type_
    }
    pub fn GetHitTestOpaqueness(&self) -> HitTestOpaqueness {
        self.hit_test_opaqueness_
    }
    pub fn HasText(&self) -> bool {
        self.has_text_
    }
    pub fn ForceDrawsContent(&mut self) {
        self.draws_content_ = true;
    }
    pub fn DrawsContent(&self) -> bool {
        self.draws_content_
    }
    pub fn IsSolidColor(&self) -> bool {
        self.solid_color_chunk_index_ != K_NOT_FOUND
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:247.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:740-761.
    pub fn UsesSolidColorLayer(&self) -> bool {
        if !self.IsSolidColor() {
            return false;
        }
        if self
            .property_tree_state_
            .EffectRequiresCompositingForBackdropFilterMask()
        {
            return false;
        }
        if self
            .property_tree_state_
            .EffectRequiresCompositingForCanvasChild()
        {
            return false;
        }
        #[cfg(target_os = "macos")]
        if self.GetSolidColor().alpha != 1.0 {
            return false;
        }
        true
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:248.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:763-767.
    pub fn GetSolidColor(&self) -> crate::adapter::layer_tree_impl::Color4f {
        assert_ne!(self.solid_color_chunk_index_, K_NOT_FOUND);
        let background = self
            .chunks_
            .chunk(self.solid_color_chunk_index_)
            .background_color();
        debug_assert!(background.is_solid_color);
        background.color
    }
    pub fn MergedAcrossCompositingBoundaryCount(&self) -> i32 {
        self.merged_across_compositing_boundary_count_
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:96.
    pub fn CanvasChildId(&self) -> B::DomNodeId {
        self.canvas_child_id_
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:76-79.
    pub fn SetCompositingTypeToOverlap(&mut self) {
        debug_assert_eq!(self.compositing_type_, CompositingType::kOther);
        self.compositing_type_ = CompositingType::kOverlap;
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:81-83.
    pub fn SetPaintArtifact(&mut self, artifact: Arc<B::PaintArtifact>) {
        self.chunks_.SetPaintArtifact(artifact);
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:85-94.
    pub fn GetCanvasChildPaintRecord(
        &self,
        c: &LayerContext<'_, B>,
        services: &dyn crate::adapter::content_layer_client_impl::ContentLayerClientServices<B>,
    ) -> Option<B::CanvasChildPaintRecord>
    where
        B: crate::adapter::content_layer_client_impl::ContentLayerClientTypes,
    {
        self.content_layer_client_
            .as_ref()
            .and_then(|client| client.GetCanvasChildPaintRecord(c, services))
    }
    pub fn canvas_child_paint_state(
        &self,
    ) -> Option<super::content_layer_client_impl::CanvasStateRead<'_, B>> {
        self.content_layer_client_.as_ref().and_then(
            super::content_layer_client_impl::ContentLayerClientRef::canvas_child_paint_state,
        )
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:128;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:187-189.
    pub fn FirstPaintChunk(&self) -> &B::PaintChunk {
        self.chunks_.chunk(0)
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:129;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:191-193.
    pub fn FirstDisplayItem(&self) -> &B::DisplayItem {
        self.FirstPaintChunk()
            .FirstDisplayItem(self.chunks_.artifact())
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:137-138;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:153-161.
    pub fn DebugName(&self) -> String {
        self.chunks_
            .artifact()
            .ClientDebugName(self.FirstPaintChunk().client_id())
    }
    pub fn OwnerNodeId(&self) -> B::DomNodeId {
        self.chunks_
            .artifact()
            .ClientOwnerNodeId(self.FirstPaintChunk().client_id())
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:136;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:134-150.
    pub fn ToJSON(&self) -> PendingLayerJson {
        PendingLayerJson {
            debug_name: self.DebugName(),
            bounds: self.bounds_,
            rect_known_to_be_opaque: self.rect_known_to_be_opaque_,
            text_known_to_be_on_opaque_background: self.text_known_to_be_on_opaque_background_,
            property_tree_state: self.property_tree_state_.ToDebugString(),
            offset_of_decomposited_transforms: self.offset_of_decomposited_transforms_,
            paint_chunks: self.chunks_.ToDebugJson(),
            draws_content: self.DrawsContent(),
            is_solid_color: self.IsSolidColor(),
            hit_test_opaqueness: HitTestOpaquenessToString(self.hit_test_opaqueness_).into(),
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:131;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:195-206.
    pub fn Matches(&self, old_pending_layer: &Self) -> bool {
        if self.ChunkRequiresOwnLayer() != old_pending_layer.ChunkRequiresOwnLayer() {
            return false;
        }
        if self.ChunkRequiresOwnLayer()
            && self.compositing_type_ != old_pending_layer.compositing_type_
        {
            return false;
        }
        self.FirstPaintChunk()
            .Matches(old_pending_layer.FirstPaintChunk())
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:147-159.
    pub fn ChunkRequiresOwnLayer(&self) -> bool {
        let result = RequiresOwnLayer(self.compositing_type_);
        #[cfg(debug_assertions)]
        if result {
            debug_assert!(self.content_layer_client_.is_none());
            debug_assert_eq!(self.chunks_.len(), 1);
        } else {
            debug_assert!(self.cc_layer_.is_none() || self.UsesSolidColorLayer());
            debug_assert!(self.chunks_.len() >= 1);
        }
        result
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:74;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:926-929.
    pub fn HasVideo(&self) -> bool {
        self.chunks_.len() == 1
            && self.FirstPaintChunk().display_item_count() == 1
            && self
                .FirstPaintChunk()
                .first_display_item_is_foreign_layer_video(self.chunks_.artifact())
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:133-134.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:491-500.
    pub fn ScrollTranslationForScrollHitTestLayer(&self) -> B::TransformNode {
        debug_assert_eq!(self.compositing_type_, CompositingType::kScrollHitTestLayer);
        debug_assert_eq!(self.chunks_.len(), 1);
        let scroll_translation = self
            .FirstPaintChunk()
            .non_composited_scroll_translation()
            .expect("scroll hit-test PaintChunk must retain its scroll translation");
        debug_assert!(scroll_translation.HasScrollNode());
        scroll_translation
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:162-163.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:523-625.
    pub fn DecompositeTransforms(pending_layers: &mut [Self]) {
        let mut can_be_decomposited: HashMap<B::TransformNode, bool> = HashMap::new();
        let mut clips_and_effects_seen: HashSet<B::PropertyNodeIdentity> = HashSet::new();
        for pending_layer in pending_layers.iter() {
            let property_state = &pending_layer.property_tree_state_;
            let mut node = property_state.TransformNode();
            while !node.IsRoot() && !can_be_decomposited.contains_key(&node) {
                if !node.IsIdentityOr2dTranslation()
                    || node.HasScrollNode()
                    || node.HasDirectCompositingReasonsOtherThan3dTransform()
                    || !node.FlattensInheritedTransformSameAsParent()
                    || !node.BackfaceVisibilitySameAsParent()
                {
                    mark_not_decompositable(&mut can_be_decomposited, node.clone());
                    break;
                }
                debug_assert!(!node.HasStickyConstraint());
                debug_assert!(!node.HasAnchorPositionScrollData());
                debug_assert!(!node.IsAffectedByOuterViewportBoundsDelta());
                can_be_decomposited.insert(node.clone(), true);
                node = node.ParentUnalias();
            }

            let mut clip = property_state.ClipNode();
            while !clip.IsRoot() && !clips_and_effects_seen.contains(&clip.Identity()) {
                clips_and_effects_seen.insert(clip.Identity());
                mark_not_decompositable(
                    &mut can_be_decomposited,
                    clip.LocalTransformSpaceUnalias(),
                );
                clip = clip.ParentUnalias();
            }
            let mut effect = property_state.EffectNode();
            while !effect.IsRoot() && !clips_and_effects_seen.contains(&effect.Identity()) {
                clips_and_effects_seen.insert(effect.Identity());
                mark_not_decompositable(
                    &mut can_be_decomposited,
                    effect.LocalTransformSpaceUnalias(),
                );
                effect = effect.ParentUnalias();
            }
            if pending_layer.compositing_type_ == CompositingType::kScrollHitTestLayer {
                mark_not_decompositable(
                    &mut can_be_decomposited,
                    pending_layer.ScrollTranslationForScrollHitTestLayer(),
                );
            }
        }

        for pending_layer in pending_layers.iter_mut() {
            let mut transform = pending_layer.property_tree_state_.TransformNode();
            while !transform.IsRoot()
                && *can_be_decomposited
                    .get(&transform)
                    .expect("every visited transform has a decomposition decision")
            {
                pending_layer.offset_of_decomposited_transforms_ += transform.Get2dTranslation();
                pending_layer.change_of_decomposited_transforms_ = pending_layer
                    .change_of_decomposited_transforms_
                    .max(transform.NodeChanged());
                transform = transform.ParentUnalias();
            }
            pending_layer
                .property_tree_state_
                .SetTransformNode(transform);
            let offset = pending_layer.offset_of_decomposited_transforms_;
            pending_layer.bounds_.Offset(offset.x(), offset.y());
            pending_layer
                .rect_known_to_be_opaque_
                .Offset(offset.x(), offset.y());
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:165-176.
    pub fn CcLayer(&self) -> LayerRef<B> {
        if let Some(client) = self.content_layer_client_.as_ref() {
            return client.LayerRef();
        }
        self.cc_layer_
            .as_ref()
            .expect("layerization must establish a canonical cc::Layer")
            .clone()
    }

    pub fn GetContentLayerClient(
        &self,
    ) -> Option<&super::content_layer_client_impl::ContentLayerClientRef<B>> {
        self.content_layer_client_.as_ref()
    }
    /// Rust finalization of the source Member<ContentLayerClientImpl>. Consumes
    /// this owner slot and runs ClearClient at final release with live context.
    pub(crate) fn ReleaseContentLayerClient(&mut self, c: &mut LayerContext<'_, B>) {
        if let Some(client) = self.content_layer_client_.take() {
            client.Release(c);
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:235-243.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:627-739.
    fn UpdateForeignLayer(&mut self, c: &mut LayerContext<'_, B>) {
        debug_assert_eq!(self.compositing_type_, CompositingType::kForeignLayer);
        debug_assert!(!self.FirstPaintChunk().HasHitTestData());
        let item = self.FirstDisplayItem();
        let visual_rect = item.ForeignLayerVisualRect();
        let layer_offset = Vector2dF::new(visual_rect.x(), visual_rect.y());
        self.cc_layer_ = Some(item.ForeignLayer());
        self.cc_layer_
            .as_ref()
            .expect("foreign display item supplies its resident layer")
            .SetOffsetToTransformParent(layer_offset + self.offset_of_decomposited_transforms_, c);
    }

    fn UpdateScrollHitTestLayer(
        &mut self,
        old_pending_layer: Option<&mut Self>,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert_eq!(self.compositing_type_, CompositingType::kScrollHitTestLayer);
        debug_assert_eq!(
            self.offset_of_decomposited_transforms_,
            Vector2dF::default()
        );
        let scroll_node = self.ScrollTranslationForScrollHitTestLayer();
        debug_assert!(self.cc_layer_.is_none());
        if let Some(old) = old_pending_layer {
            self.cc_layer_ = old.cc_layer_.take();
        }
        let element = scroll_node.ScrollCompositorElementId();
        if let Some(layer) = self.cc_layer_.as_ref() {
            debug_assert_eq!(layer.element_id(c), element);
        } else {
            let layer = Layer::<B>::Create(c.Services());
            layer.SetElementId(element, c);
            self.cc_layer_ = Some(layer);
        }
        let layer = self.cc_layer_.as_ref().unwrap();
        layer.SetHitTestOpaqueness(self.hit_test_opaqueness_, c);
        let container = scroll_node.ScrollContainerRect();
        layer.SetOffsetToTransformParent(Vector2dF::new(container.x as f32, container.y as f32), c);
        layer.SetBounds(container.size(), c);
        if scroll_node.NodeChanged() != B::UnchangedPaintPropertyChange() {
            layer.SetNeedsPushProperties(Layer::<B>::kChangedAllProperties, c);
            layer.SetNeedsCommit(c);
        }
    }

    fn UpdateScrollbarLayer(
        &mut self,
        old_pending_layer: Option<&mut Self>,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert_eq!(self.compositing_type_, CompositingType::kScrollbarLayer);
        let item = self.FirstDisplayItem();
        debug_assert!(item.IsScrollbar());
        let old_layer = old_pending_layer.and_then(|old| old.cc_layer_.take());
        let layer =
            item.CreateOrReuseScrollbarLayer(
                old_layer,
                self.offset_of_decomposited_transforms_,
                c,
            );
        debug_assert!(self.cc_layer_.is_none());
        self.cc_layer_ = Some(layer);
    }

    fn UpdateContentLayer(
        &mut self,
        old_pending_layer: Option<&mut Self>,
        property_state_for_paint: B::PropertyTreeState,
        tracks_raster_invalidations: bool,
        c: &mut LayerContext<'_, B>,
        paint: &dyn crate::adapter::paint_chunks_to_cc_layer::PaintChunksServices<B>,
    ) where
        B: crate::adapter::paint_chunks_to_cc_layer::PaintChunksTypes,
    {
        debug_assert!(!self.ChunkRequiresOwnLayer());
        debug_assert!(self.cc_layer_.is_none());
        debug_assert!(self.content_layer_client_.is_none());
        debug_assert!(!self.UsesSolidColorLayer());
        if let Some(old) = old_pending_layer {
            self.content_layer_client_ = old.content_layer_client_.take();
        }
        let mut client = self.content_layer_client_.take().unwrap_or_else(|| {
            let mut client =
                super::content_layer_client_impl::ContentLayerClientRef::<B>::new(c, paint);
            client.SetTracksRasterInvalidations(tracks_raster_invalidations);
            client
        });
        client.UpdateCcPictureLayer(self, property_state_for_paint, c, paint);
        self.content_layer_client_ = Some(client);
    }

    fn UpdateSolidColorLayer(
        &mut self,
        old_pending_layer: Option<&mut Self>,
        services: &mut dyn PendingLayerServices<B>,
        c: &mut LayerContext<'_, B>,
    ) {
        debug_assert!(!self.ChunkRequiresOwnLayer());
        debug_assert!(self.cc_layer_.is_none());
        debug_assert!(self.content_layer_client_.is_none());
        debug_assert!(self.UsesSolidColorLayer());
        if let Some(old) = old_pending_layer {
            self.cc_layer_ = old.cc_layer_.take();
        }
        if self.cc_layer_.is_none() {
            self.cc_layer_ = Some(services.CreateSolidColorLayer(c.Services()));
        }
        let (layer_offset, layer_bounds) = self.Bounds();
        let layer = self.cc_layer_.as_ref().unwrap();
        layer.SetOffsetToTransformParent(layer_offset, c);
        layer.SetBounds(layer_bounds, c);
        layer.SetHitTestOpaqueness(self.hit_test_opaqueness_, c);
        layer.SetBackgroundColor(self.GetSolidColor(), c);
        layer.SetIsDrawable(self.draws_content_, c);
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:178-193.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:769-857.
    pub fn UpdateCompositedLayer(
        &mut self,
        mut old_pending_layer: Option<&mut Self>,
        property_state_for_paint: B::PropertyTreeState,
        layer_selection: &mut B::LayerSelection,
        tracks_raster_invalidations: bool,
        layer_tree_host: Option<MainLayerHostId>,
        services: &mut dyn PendingLayerServices<B>,
        c: &mut LayerContext<'_, B>,
        paint: &dyn crate::adapter::paint_chunks_to_cc_layer::PaintChunksServices<B>,
    ) where
        B: crate::adapter::paint_chunks_to_cc_layer::PaintChunksTypes,
    {
        self.non_composited_scroll_translations_.clear();
        match self.compositing_type_ {
            CompositingType::kForeignLayer => self.UpdateForeignLayer(c),
            CompositingType::kScrollHitTestLayer => {
                self.UpdateScrollHitTestLayer(old_pending_layer.as_deref_mut(), c)
            }
            CompositingType::kScrollbarLayer => {
                self.UpdateScrollbarLayer(old_pending_layer.as_deref_mut(), c)
            }
            _ => {
                debug_assert!(!self.ChunkRequiresOwnLayer());
                if self.UsesSolidColorLayer() {
                    self.UpdateSolidColorLayer(old_pending_layer.as_deref_mut(), services, c);
                } else {
                    self.UpdateContentLayer(
                        old_pending_layer.as_deref_mut(),
                        property_state_for_paint,
                        tracks_raster_invalidations,
                        c,
                        paint,
                    );
                }
            }
        }
        let layer = self.CcLayer();
        layer.SetLayerTreeHost(layer_tree_host, c);
        self.UpdateLayerProperties(layer_selection, false, c, paint);
        if !layer.subtree_property_changed(c)
            && self.PropertyTreeStateChanged(old_pending_layer.as_deref())
        {
            layer.SetSubtreePropertyChanged(c);
        }
    }

    pub fn UpdateCompositedLayerForRepaint(
        &mut self,
        repainted_artifact: Arc<B::PaintArtifact>,
        property_state_for_paint: B::PropertyTreeState,
        layer_selection: &mut B::LayerSelection,
        c: &mut LayerContext<'_, B>,
        paint: &dyn crate::adapter::paint_chunks_to_cc_layer::PaintChunksServices<B>,
    ) where
        B: crate::adapter::paint_chunks_to_cc_layer::PaintChunksTypes,
    {
        debug_assert_eq!(
            self.chunks_.artifact().PaintChunkCount(),
            repainted_artifact.PaintChunkCount()
        );
        self.SetPaintArtifact(repainted_artifact);
        let chunks_unchanged = (0..self.chunks_.len())
            .all(|index| self.chunks_.chunk(index).is_moved_from_cached_subsequence());
        if !self.ChunkRequiresOwnLayer() {
            if self.UsesSolidColorLayer() {
                let layer = self
                    .cc_layer_
                    .as_ref()
                    .expect("solid-color repaint requires its resident layer");
                if !chunks_unchanged {
                    layer.SetBackgroundColor(self.GetSolidColor(), c);
                }
            } else {
                let mut client = self
                    .content_layer_client_
                    .take()
                    .expect("content repaint requires its resident client");
                if chunks_unchanged {
                    client.SetOldPaintArtifact(Arc::clone(self.chunks_.artifact()));
                } else {
                    client.UpdateCcPictureLayer(self, property_state_for_paint, c, paint);
                }
                self.content_layer_client_ = Some(client);
            }
        }
        self.UpdateLayerProperties(layer_selection, chunks_unchanged, c, paint);
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:245.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:866-874.
    fn UpdateLayerProperties(
        &self,
        layer_selection: &mut B::LayerSelection,
        selection_only: bool,
        c: &mut LayerContext<'_, B>,
        paint: &dyn crate::adapter::paint_chunks_to_cc_layer::PaintChunksServices<B>,
    ) where
        B: crate::adapter::paint_chunks_to_cc_layer::PaintChunksTypes,
    {
        if self.compositing_type_ == CompositingType::kForeignLayer {
            return;
        }
        let layer = self.CcLayer();
        super::paint_chunks_to_cc_layer::UpdateLayerProperties::<B>(
            &layer,
            &self.property_tree_state_,
            &self.chunks_,
            layer_selection,
            selection_only,
            c,
            paint,
        );
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:121-126.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:229-231.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:114-132.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:167-185.
    fn MapRectKnownToBeOpaque(
        &self,
        new_state: &B::PropertyTreeState,
        mapped_layer_bounds: PendingMappedVisualRect,
        geometry: &dyn PendingLayerGeometry<B>,
    ) -> RectF {
        if !mapped_layer_bounds.is_tight || self.rect_known_to_be_opaque_.IsEmpty() {
            return RectF::default();
        }
        if self.rect_known_to_be_opaque_ == self.bounds_ {
            return mapped_layer_bounds.rect;
        }
        let mut mapped_opaque = geometry.MapLocalToAncestorVisualRect(
            &self.property_tree_state_,
            new_state,
            self.rect_known_to_be_opaque_,
        );
        intersect_rectf(&mut mapped_opaque.rect, mapped_layer_bounds.rect);
        debug_assert!(mapped_opaque.is_tight);
        mapped_opaque.rect
    }

    pub fn Upcast(
        &mut self,
        new_state: &B::PropertyTreeState,
        geometry: &dyn PendingLayerGeometry<B>,
    ) {
        debug_assert!(!self.ChunkRequiresOwnLayer());
        debug_assert!(new_state.EffectUnaliasedParentSameAs(&self.property_tree_state_));
        if self.property_tree_state_.EffectBlendMode() != BlendMode::kSrcOver {
            self.has_decomposited_blend_mode_ = true;
        }
        let mapped_bounds = geometry.MapLocalToAncestorVisualRect(
            &self.property_tree_state_,
            new_state,
            self.bounds_,
        );
        self.rect_known_to_be_opaque_ =
            self.MapRectKnownToBeOpaque(new_state, mapped_bounds, geometry);
        self.bounds_ = mapped_bounds.rect;
        self.property_tree_state_ = new_state.clone();
        self.solid_color_chunk_index_ = K_NOT_FOUND;
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:98-99.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:115-119.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:210-227;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:444-489.
    fn CanUpcastWith(
        &self,
        guest: &Self,
        guest_state: &B::PropertyTreeState,
        merge_fixed_layers: bool,
        merge_sticky_layers: bool,
        is_composited_scroll: &mut dyn FnMut(&B::TransformNode) -> bool,
    ) -> Option<PendingUpcastResult<B::PropertyTreeState>> {
        debug_assert!(Arc::ptr_eq(
            self.chunks_.artifact(),
            guest.chunks_.artifact()
        ));
        if self.ChunkRequiresOwnLayer() || guest.ChunkRequiresOwnLayer() {
            return None;
        }
        if !self.property_tree_state_.SameEffectNode(guest_state) {
            return None;
        }
        let result = self.property_tree_state_.CanUpcastWith(
            guest_state,
            merge_fixed_layers,
            merge_sticky_layers,
            is_composited_scroll,
        )?;
        let home_scroll_translation = self.property_tree_state_.NearestScrollTranslationNode();
        let guest_scroll_translation = guest_state.NearestScrollTranslationNode();
        if home_scroll_translation == guest_scroll_translation {
            return Some(result);
        }
        let lca_scroll_translation = result.upcasted_state.NearestScrollTranslationNode();
        if (guest_scroll_translation == lca_scroll_translation
            || self
                .non_composited_scroll_translations_
                .contains(&guest_scroll_translation))
            && (home_scroll_translation == lca_scroll_translation
                || guest
                    .non_composited_scroll_translations_
                    .contains(&home_scroll_translation))
        {
            return Some(result);
        }
        None
    }

    pub fn CanMergeWithDecompositedBlendMode(
        &self,
        guest: &Self,
        upcast_state: &B::PropertyTreeState,
        merge_fixed_layers: bool,
        merge_sticky_layers: bool,
        is_composited_scroll: &mut dyn FnMut(&B::TransformNode) -> bool,
    ) -> bool {
        self.CanUpcastWith(
            guest,
            upcast_state,
            merge_fixed_layers,
            merge_sticky_layers,
            is_composited_scroll,
        )
        .is_some()
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:101-119;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:217-227;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:204-442.
    fn CanMerge(
        &self,
        guest: &Self,
        lcd_text_preference: LcdTextPreference,
        device_pixel_ratio: f32,
        is_composited_scroll: &mut dyn FnMut(&B::TransformNode) -> bool,
        geometry: &dyn PendingLayerGeometry<B>,
    ) -> Option<PendingMergeState<B::PropertyTreeState>> {
        const K_MERGE_SPARSITY_AREA_TOLERANCE: f32 = 10_000.0;
        if self.canvas_child_id_ != guest.canvas_child_id_ {
            return None;
        }
        let mut force_merge = self.canvas_child_id_ == guest.canvas_child_id_
            && self.canvas_child_id_ != B::InvalidDomNodeId();
        let upcast_result = self.CanUpcastWith(
            guest,
            &guest.property_tree_state_,
            geometry.MergeFixedLayersEnabled(),
            geometry.MergeStickyLayersEnabled(),
            is_composited_scroll,
        );
        let PendingUpcastResult {
            upcasted_state: merged_state,
            scroll_range_dependent,
        } = match upcast_result {
            Some(result) => result,
            None if force_merge => PendingUpcastResult {
                upcasted_state: self.property_tree_state_.clone(),
                scroll_range_dependent: false,
            },
            None => return None,
        };
        let merged_visibility_limit = geometry.VisibilityLimit(&merged_state);
        if !guest.has_decomposited_blend_mode_
            && merged_visibility_limit == Some(self.bounds_)
            && merged_state == self.property_tree_state_
            && self.rect_known_to_be_opaque_ == self.bounds_
        {
            return Some(PendingMergeState {
                bounds: self.bounds_,
                property_tree_state: merged_state,
                rect_known_to_be_opaque: self.bounds_,
                text_known_to_be_on_opaque_background: true,
                solid_color_chunk_index: if guest.draws_content_ {
                    K_NOT_FOUND
                } else {
                    self.solid_color_chunk_index_
                },
                hit_test_opaqueness: UnionHitTestOpaqueness(
                    geometry.ToRoundedRect(self.bounds_),
                    self.hit_test_opaqueness_,
                    geometry.ToRoundedRect(guest.bounds_),
                    guest.hit_test_opaqueness_,
                ),
                scroll_range_dependent,
            });
        }

        let mut new_home_bounds = geometry.MapLocalToAncestorVisualRect(
            &self.property_tree_state_,
            &merged_state,
            self.bounds_,
        );
        let mut new_guest_bounds = geometry.MapLocalToAncestorVisualRect(
            &guest.property_tree_state_,
            &merged_state,
            guest.bounds_,
        );
        if let Some(limit) = merged_visibility_limit {
            intersect_rectf(&mut new_home_bounds.rect, limit);
            intersect_rectf(&mut new_guest_bounds.rect, limit);
        }
        let mut merged_bounds = geometry.UnionRects(new_home_bounds.rect, new_guest_bounds.rect);
        if guest.has_decomposited_blend_mode_ {
            force_merge = true;
        }

        let mut merged_rect_known_to_be_opaque = RectF::default();
        let mut merged_text_known_to_be_on_opaque_background = false;
        let mut merged_solid_color_chunk_index = K_NOT_FOUND;
        if !force_merge {
            let sum_area = new_home_bounds.rect.width() * new_home_bounds.rect.height()
                + new_guest_bounds.rect.width() * new_guest_bounds.rect.height();
            let tolerance =
                K_MERGE_SPARSITY_AREA_TOLERANCE * device_pixel_ratio * device_pixel_ratio;
            if merged_bounds.width() * merged_bounds.height() - sum_area > tolerance {
                return None;
            }
            let home_opaque = self.MapRectKnownToBeOpaque(&merged_state, new_home_bounds, geometry);
            let guest_opaque =
                guest.MapRectKnownToBeOpaque(&merged_state, new_guest_bounds, geometry);
            merged_rect_known_to_be_opaque = geometry.MaximumCoveredRect(home_opaque, guest_opaque);
            merged_text_known_to_be_on_opaque_background =
                self.text_known_to_be_on_opaque_background_;
            if self.text_known_to_be_on_opaque_background_
                != guest.text_known_to_be_on_opaque_background_
            {
                if !self.text_known_to_be_on_opaque_background_ {
                    if merged_rect_known_to_be_opaque.Contains(new_home_bounds.rect) {
                        merged_text_known_to_be_on_opaque_background = true;
                    }
                } else if !guest.text_known_to_be_on_opaque_background_
                    && !merged_rect_known_to_be_opaque.Contains(new_guest_bounds.rect)
                {
                    merged_text_known_to_be_on_opaque_background = false;
                }
            }
            if lcd_text_preference == LcdTextPreference::kStronglyPreferred
                && !merged_text_known_to_be_on_opaque_background
            {
                if self.has_text_ && self.text_known_to_be_on_opaque_background_ {
                    return None;
                }
                if guest.has_text_ && guest.text_known_to_be_on_opaque_background_ {
                    return None;
                }
            }
            if self.IsSolidColor()
                && new_home_bounds.is_tight
                && !guest.draws_content_
                && new_home_bounds.rect == merged_bounds
            {
                merged_solid_color_chunk_index = self.solid_color_chunk_index_;
            } else if guest.IsSolidColor()
                && new_guest_bounds.is_tight
                && new_guest_bounds.rect == merged_bounds
                && (!self.draws_content_ || guest.GetSolidColor().alpha == 1.0)
            {
                merged_solid_color_chunk_index =
                    self.chunks_.len() + guest.solid_color_chunk_index_;
            }
            if !self
                .property_tree_state_
                .SameNearestDirectlyCompositedAncestor(&guest.property_tree_state_)
            {
                assert!(geometry.MergeFixedLayersEnabled() || geometry.MergeStickyLayersEnabled());
                if (self.IsSolidColor() || guest.IsSolidColor())
                    && merged_solid_color_chunk_index == K_NOT_FOUND
                {
                    return None;
                }
            }
        }

        let mut home_hit_test_opaqueness = self.hit_test_opaqueness_;
        if home_hit_test_opaqueness == HitTestOpaqueness::kOpaque && !new_home_bounds.is_tight {
            home_hit_test_opaqueness = HitTestOpaqueness::kMixed;
        }
        let mut guest_hit_test_opaqueness = guest.hit_test_opaqueness_;
        if guest_hit_test_opaqueness == HitTestOpaqueness::kOpaque && !new_guest_bounds.is_tight {
            guest_hit_test_opaqueness = HitTestOpaqueness::kMixed;
        }
        let merged_hit_test_opaqueness = UnionHitTestOpaqueness(
            geometry.ToRoundedRect(new_home_bounds.rect),
            home_hit_test_opaqueness,
            geometry.ToRoundedRect(new_guest_bounds.rect),
            guest_hit_test_opaqueness,
        );
        PreserveNearIntegralBounds(&mut merged_bounds);
        PreserveNearIntegralBounds(&mut merged_rect_known_to_be_opaque);
        Some(PendingMergeState {
            bounds: merged_bounds,
            property_tree_state: merged_state,
            rect_known_to_be_opaque: merged_rect_known_to_be_opaque,
            text_known_to_be_on_opaque_background: merged_text_known_to_be_on_opaque_background,
            solid_color_chunk_index: merged_solid_color_chunk_index,
            hit_test_opaqueness: merged_hit_test_opaqueness,
            scroll_range_dependent,
        })
    }

    pub fn Merge(
        &mut self,
        guest: &Self,
        lcd_text_preference: LcdTextPreference,
        device_pixel_ratio: f32,
        is_composited_scroll: &mut dyn FnMut(&B::TransformNode) -> bool,
        geometry: &dyn PendingLayerGeometry<B>,
    ) -> MergeResult {
        let Some(merged) = self.CanMerge(
            guest,
            lcd_text_preference,
            device_pixel_ratio,
            is_composited_scroll,
            geometry,
        ) else {
            return MergeResult::default();
        };
        self.chunks_.Merge(&guest.chunks_);
        self.bounds_ = merged.bounds;
        if self.property_tree_state_ != merged.property_tree_state {
            self.property_tree_state_ = merged.property_tree_state;
        }
        self.draws_content_ |= guest.draws_content_;
        self.rect_known_to_be_opaque_ = merged.rect_known_to_be_opaque;
        self.text_known_to_be_on_opaque_background_ = merged.text_known_to_be_on_opaque_background;
        self.has_text_ |= guest.has_text_;
        self.solid_color_chunk_index_ = merged.solid_color_chunk_index;
        self.change_of_decomposited_transforms_ = self
            .change_of_decomposited_transforms_
            .max(guest.change_of_decomposited_transforms_);
        self.hit_test_opaqueness_ = merged.hit_test_opaqueness;
        self.non_composited_scroll_translations_
            .extend(guest.non_composited_scroll_translations_.iter().cloned());
        self.merged_across_compositing_boundary_count_ +=
            guest.merged_across_compositing_boundary_count_;
        if !self
            .property_tree_state_
            .SameNearestDirectlyCompositedAncestor(&guest.property_tree_state_)
        {
            assert!(geometry.MergeFixedLayersEnabled() || geometry.MergeStickyLayersEnabled());
            self.merged_across_compositing_boundary_count_ += 1;
        }
        MergeResult {
            merged: true,
            scroll_range_dependent: merged.scroll_range_dependent,
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:233.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:502-513.
    pub fn PropertyTreeStateChanged(&self, old_pending_layer: Option<&Self>) -> bool {
        if old_pending_layer
            .map(|old| old.property_tree_state_ != self.property_tree_state_)
            .unwrap_or(true)
        {
            return true;
        }
        let change = B::ChangedOnlySimpleValues();
        if self.change_of_decomposited_transforms_ >= change {
            return true;
        }
        self.property_tree_state_.ChangedToRoot(change)
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:161;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:515-522.
    pub fn MightOverlap(&self, other: &Self, geometry: &dyn PendingLayerGeometry<B>) -> bool {
        if self.canvas_child_id_ != other.canvas_child_id_ {
            return false;
        }
        geometry.MightOverlapForCompositing(
            self.bounds_,
            &self.property_tree_state_,
            other.bounds_,
            &other.property_tree_state_,
        )
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:199;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:875-924.
    pub fn ComputeBackgroundColor(
        &self,
        colors: &dyn PendingLayerColor,
    ) -> crate::adapter::layer_tree_impl::Color4f {
        const K_MIN_BACKGROUND_COLOR_COVERAGE_RATIO: f32 = 0.5;
        let mut background_colors = Vec::with_capacity(4);
        let min_background_area =
            K_MIN_BACKGROUND_COLOR_COVERAGE_RATIO * self.bounds_.width() * self.bounds_.height();
        for index in (0..self.chunks_.len()).rev() {
            let chunk = self.chunks_.chunk(index);
            let background = chunk.background_color();
            if background.color.alpha == 0.0 || background.area < min_background_area {
                continue;
            }
            let mut chunk_background_color = background.color;
            let chunk_state = chunk.PropertiesUnalias();
            if !chunk_state.SameEffectNode(&self.property_tree_state_) {
                if !chunk_state.EffectUnaliasedParentSameAs(&self.property_tree_state_)
                    || !chunk_state.EffectIsOpacityOnly()
                {
                    continue;
                }
                chunk_background_color.alpha *= chunk_state.EffectOpacity();
            }
            background_colors.push(chunk_background_color);
            if chunk_background_color.alpha == 1.0 {
                break;
            }
        }

        let Some(mut background_color) = background_colors.pop() else {
            return crate::adapter::layer_tree_impl::Color4f::default();
        };
        for color in background_colors.into_iter().rev() {
            background_color = colors.GetResultingPaintColor(color, background_color);
        }
        background_color
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:194-197;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:858-864.
    pub fn UpdateForRasterInducingScroll(&mut self) {
        if let Some(client) = self.content_layer_client_.as_mut() {
            if client.HasRasterInducingScroll() {
                client.UpdateForRasterInducingScroll(&self.chunks_);
            }
        }
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:163-165.
impl<B: PendingLayerTypes> fmt::Display for PendingLayer<B> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = self.ToJSON();
        write!(
            output,
            concat!(
                "{{\n",
                "  \"debug_name\": {},\n",
                "  \"bounds\": [{}, {}, {}, {}],\n",
                "  \"rect_known_to_be_opaque\": [{}, {}, {}, {}],\n",
                "  \"text_known_to_be_on_opaque_background\": {},\n",
                "  \"property_tree_state\": {},\n",
                "  \"offset_of_decomposited_transforms\": [{}, {}],\n",
                "  \"paint_chunks\": {},\n",
                "  \"draws_content\": {},\n",
                "  \"is_solid_color\": {},\n",
                "  \"hit_test_opaqueness\": {}\n",
                "}}"
            ),
            json_string(&json.debug_name),
            json.bounds.x(),
            json.bounds.y(),
            json.bounds.width(),
            json.bounds.height(),
            json.rect_known_to_be_opaque.x(),
            json.rect_known_to_be_opaque.y(),
            json.rect_known_to_be_opaque.width(),
            json.rect_known_to_be_opaque.height(),
            json.text_known_to_be_on_opaque_background,
            json_string(&json.property_tree_state),
            json.offset_of_decomposited_transforms.x(),
            json.offset_of_decomposited_transforms.y(),
            json.paint_chunks,
            json.draws_content,
            json.is_solid_color,
            json_string(&json.hit_test_opaqueness),
        )
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:143-145.
pub const fn RequiresOwnLayer(compositing_type: CompositingType) -> bool {
    !matches!(
        compositing_type,
        CompositingType::kOverlap | CompositingType::kOther
    )
}

// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:27-42.
// Snap bounds only when all four edges are already within the source tolerance
// of integral values. Reconstructing RectF is the translated ToRoundedRect path.
pub fn PreserveNearIntegralBounds(bounds: &mut RectF) {
    const K_TOLERANCE: f32 = 1e-3;
    let left = bounds.x();
    let top = bounds.y();
    let right = bounds.right();
    let bottom = bounds.bottom();
    if (left.round() - left).abs() <= K_TOLERANCE
        && (top.round() - top).abs() <= K_TOLERANCE
        && (right.round() - right).abs() <= K_TOLERANCE
        && (bottom.round() - bottom).abs() <= K_TOLERANCE
    {
        let rounded_left = left.round();
        let rounded_top = top.round();
        let rounded_right = right.round();
        let rounded_bottom = bottom.round();
        *bounds = RectF::new(
            rounded_left,
            rounded_top,
            rounded_right - rounded_left,
            rounded_bottom - rounded_top,
        );
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:47-48;
// Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:88-112.
// This is the exact body used by PendingLayer::Bounds. Keeping it independent
// of owner storage lets the eventual resident object call it without copying
// PaintArtifact, chunks, property nodes, or layer state.
pub fn PendingLayerBounds(
    bounds: RectF,
    rect_known_to_be_opaque: RectF,
    is_solid_color: bool,
) -> (Vector2dF, Size) {
    let ceiled_size = Size::new(bounds.width().ceil() as i32, bounds.height().ceil() as i32);
    if is_solid_color {
        return (Vector2dF::new(bounds.x(), bounds.y()), ceiled_size);
    }

    let bounds_with_ceiled_size = RectF::FromOriginSize(
        bounds.origin(),
        SizeF::new(ceiled_size.width() as f32, ceiled_size.height() as f32),
    );
    let enclosing_bounds = ToEnclosingRect(bounds);
    if rect_known_to_be_opaque.Contains(bounds_with_ceiled_size)
        && !rect_known_to_be_opaque.Contains(RectF::from(enclosing_bounds))
    {
        return (Vector2dF::new(bounds.x(), bounds.y()), ceiled_size);
    }

    (
        Vector2dF::new(enclosing_bounds.x as f32, enclosing_bounds.y as f32),
        enclosing_bounds.size(),
    )
}

fn intersect_rectf(rect: &mut RectF, other: RectF) {
    let left = rect.x().max(other.x());
    let top = rect.y().max(other.y());
    let right = rect.right().min(other.right());
    let bottom = rect.bottom().min(other.bottom());
    *rect = if left >= right || top >= bottom {
        RectF::default()
    } else {
        RectF::new(left, top, right - left, bottom - top)
    };
}

fn mark_not_decompositable<T, C>(decisions: &mut HashMap<T, bool>, mut node: T)
where
    T: PendingTransformNode<C>,
    C: Copy,
{
    loop {
        if node.IsRoot() {
            break;
        }
        match decisions.insert(node.clone(), false) {
            Some(false) => break,
            Some(true) | None => {}
        }
        let Some(parent) = node.UnaliasedParent() else {
            break;
        };
        node = parent;
    }
}

fn json_string(value: &str) -> String {
    let mut result = String::with_capacity(value.len() + 2);
    result.push('"');
    for character in value.chars() {
        match character {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\u{08}' => result.push_str("\\b"),
            '\u{0c}' => result.push_str("\\f"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c <= '\u{1f}' => {
                use fmt::Write;
                write!(&mut result, "\\u{:04x}", c as u32).unwrap();
            }
            c => result.push(c),
        }
    }
    result.push('"');
    result
}
