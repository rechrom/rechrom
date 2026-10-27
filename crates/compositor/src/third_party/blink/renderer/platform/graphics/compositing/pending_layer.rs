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
        geometry::{RectF, Size, Vector2dF},
        main_layer::{LayerContext, MainLayerHostId},
        pending_layer::{
            LcdTextPreference, PendingClipNode, PendingDisplayItem, PendingEffectNode,
            PendingLayerColor, PendingLayerGeometry, PendingLayerJson, PendingLayerServices,
            PendingLayerTypes, PendingMappedVisualRect, PendingPaintArtifact, PendingPaintChunk,
            PendingPaintChunkSubset, PendingPropertyTreeState, PendingTransformNode,
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
use compositor_layerize::background_color::{
    BackgroundColorOperations, BackgroundEffect, BackgroundSample,
    ComputeBackgroundColor as ComputeLayerBackgroundColor,
};
use compositor_layerize::pending_layer_policy::{
    CanUpcastPendingLayers, ComputePendingLayerMerge, ComputePendingLayerUpcast,
    MightOverlapPendingLayers, PendingLayerMergeGeometry, PendingLayerMergeInput,
    PendingLayerMergeOutput, PendingMappedVisualRect as PolicyMappedVisualRect,
    PendingUpcastResult,
};
pub use compositor_layerize::pending_layer_policy::{
    CompositingType, MergeResult, PendingLayerBounds, RequiresOwnLayer,
};
use compositor_layerize::transform_decomposition::PendingLayerDecomposition;
use std::{fmt, sync::Arc};

const K_NOT_FOUND: usize = usize::MAX;

struct BackgroundColorAdapter<'a>(&'a dyn PendingLayerColor);

impl BackgroundColorOperations<crate::adapter::layer_tree_impl::Color4f>
    for BackgroundColorAdapter<'_>
{
    fn Alpha(&self, color: crate::adapter::layer_tree_impl::Color4f) -> f32 {
        color.alpha
    }

    fn MultiplyAlpha(
        &self,
        mut color: crate::adapter::layer_tree_impl::Color4f,
        opacity: f32,
    ) -> crate::adapter::layer_tree_impl::Color4f {
        color.alpha *= opacity;
        color
    }

    fn SourceOver(
        &self,
        foreground: crate::adapter::layer_tree_impl::Color4f,
        background: crate::adapter::layer_tree_impl::Color4f,
    ) -> crate::adapter::layer_tree_impl::Color4f {
        self.0.GetResultingPaintColor(foreground, background)
    }
}

struct MergeGeometryAdapter<'a, B: PendingLayerTypes>(&'a dyn PendingLayerGeometry<B>);

impl<B: PendingLayerTypes>
    PendingLayerMergeGeometry<B::PropertyTreeState, B::TransformNode, HitTestOpaqueness>
    for MergeGeometryAdapter<'_, B>
{
    fn SameEffectNode(&self, first: &B::PropertyTreeState, second: &B::PropertyTreeState) -> bool {
        first.SameEffectNode(second)
    }

    fn CanUpcastWith(
        &self,
        first: &B::PropertyTreeState,
        second: &B::PropertyTreeState,
        merge_fixed_layers: bool,
        merge_sticky_layers: bool,
        is_composited_scroll: &mut dyn FnMut(&B::TransformNode) -> bool,
    ) -> Option<PendingUpcastResult<B::PropertyTreeState>> {
        first.CanUpcastWith(
            second,
            merge_fixed_layers,
            merge_sticky_layers,
            is_composited_scroll,
        )
    }

    fn NearestScrollTranslationNode(&self, state: &B::PropertyTreeState) -> B::TransformNode {
        state.NearestScrollTranslationNode()
    }

    fn SameNearestDirectlyCompositedAncestor(
        &self,
        first: &B::PropertyTreeState,
        second: &B::PropertyTreeState,
    ) -> bool {
        first.SameNearestDirectlyCompositedAncestor(second)
    }

    fn MightOverlapForCompositing(
        &self,
        first_bounds: RectF,
        first_state: &B::PropertyTreeState,
        second_bounds: RectF,
        second_state: &B::PropertyTreeState,
    ) -> bool {
        self.0
            .MightOverlapForCompositing(first_bounds, first_state, second_bounds, second_state)
    }

    fn VisibilityLimit(&self, state: &B::PropertyTreeState) -> Option<RectF> {
        self.0.VisibilityLimit(state)
    }

    fn MapLocalToAncestorVisualRect(
        &self,
        source: &B::PropertyTreeState,
        destination: &B::PropertyTreeState,
        rect: RectF,
    ) -> PolicyMappedVisualRect {
        self.0
            .MapLocalToAncestorVisualRect(source, destination, rect)
    }

    fn MaximumCoveredRect(&self, first: RectF, second: RectF) -> RectF {
        self.0.MaximumCoveredRect(first, second)
    }

    fn UnionRects(&self, first: RectF, second: RectF) -> RectF {
        self.0.UnionRects(first, second)
    }

    fn ToRoundedRect(&self, rect: RectF) -> crate::adapter::geometry::Rect {
        self.0.ToRoundedRect(rect)
    }

    fn HitTestOpaquenessIsOpaque(&self, value: HitTestOpaqueness) -> bool {
        value == HitTestOpaqueness::kOpaque
    }

    fn MixedHitTestOpaqueness(&self) -> HitTestOpaqueness {
        HitTestOpaqueness::kMixed
    }

    fn UnionHitTestOpaqueness(
        &self,
        first_rect: crate::adapter::geometry::Rect,
        first: HitTestOpaqueness,
        second_rect: crate::adapter::geometry::Rect,
        second: HitTestOpaqueness,
    ) -> HitTestOpaqueness {
        UnionHitTestOpaqueness(first_rect, first, second_rect, second)
    }

    fn MergeFixedLayersEnabled(&self) -> bool {
        self.0.MergeFixedLayersEnabled()
    }

    fn MergeStickyLayersEnabled(&self) -> bool {
        self.0.MergeStickyLayersEnabled()
    }
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

impl<B: PendingLayerTypes> PendingLayerDecomposition for PendingLayer<B> {
    type Transform = B::TransformNode;
    type Clip = B::ClipNode;
    type Effect = B::EffectNode;
    type NodeIdentity = B::PropertyNodeIdentity;

    fn TransformNode(&self) -> Self::Transform {
        self.property_tree_state_.TransformNode()
    }
    fn ClipNode(&self) -> Self::Clip {
        self.property_tree_state_.ClipNode()
    }
    fn EffectNode(&self) -> Self::Effect {
        self.property_tree_state_.EffectNode()
    }
    fn TransformIsRoot(&self, node: &Self::Transform) -> bool {
        node.IsRoot()
    }
    fn TransformParentUnalias(&self, node: &Self::Transform) -> Self::Transform {
        node.ParentUnalias()
    }
    fn TransformUnaliasedParent(&self, node: &Self::Transform) -> Option<Self::Transform> {
        node.UnaliasedParent()
    }
    fn TransformIsIdentityOr2dTranslation(&self, node: &Self::Transform) -> bool {
        node.IsIdentityOr2dTranslation()
    }
    fn TransformHasScrollNode(&self, node: &Self::Transform) -> bool {
        node.HasScrollNode()
    }
    fn TransformHasDirectCompositingReasonsOtherThan3dTransform(
        &self,
        node: &Self::Transform,
    ) -> bool {
        node.HasDirectCompositingReasonsOtherThan3dTransform()
    }
    fn TransformFlattensInheritedTransformSameAsParent(&self, node: &Self::Transform) -> bool {
        node.FlattensInheritedTransformSameAsParent()
    }
    fn TransformBackfaceVisibilitySameAsParent(&self, node: &Self::Transform) -> bool {
        node.BackfaceVisibilitySameAsParent()
    }
    fn TransformHasStickyConstraint(&self, node: &Self::Transform) -> bool {
        node.HasStickyConstraint()
    }
    fn TransformHasAnchorPositionScrollData(&self, node: &Self::Transform) -> bool {
        node.HasAnchorPositionScrollData()
    }
    fn TransformIsAffectedByOuterViewportBoundsDelta(&self, node: &Self::Transform) -> bool {
        node.IsAffectedByOuterViewportBoundsDelta()
    }
    fn ClipIsRoot(&self, node: &Self::Clip) -> bool {
        node.IsRoot()
    }
    fn ClipIdentity(&self, node: &Self::Clip) -> Self::NodeIdentity {
        node.Identity()
    }
    fn ClipParentUnalias(&self, node: &Self::Clip) -> Self::Clip {
        node.ParentUnalias()
    }
    fn ClipLocalTransformSpaceUnalias(&self, node: &Self::Clip) -> Self::Transform {
        node.LocalTransformSpaceUnalias()
    }
    fn EffectIsRoot(&self, node: &Self::Effect) -> bool {
        node.IsRoot()
    }
    fn EffectIdentity(&self, node: &Self::Effect) -> Self::NodeIdentity {
        node.Identity()
    }
    fn EffectParentUnalias(&self, node: &Self::Effect) -> Self::Effect {
        node.ParentUnalias()
    }
    fn EffectLocalTransformSpaceUnalias(&self, node: &Self::Effect) -> Self::Transform {
        node.LocalTransformSpaceUnalias()
    }
    fn IsScrollHitTestLayer(&self) -> bool {
        self.compositing_type_ == CompositingType::kScrollHitTestLayer
    }
    fn ScrollTranslationForScrollHitTestLayer(&self) -> Self::Transform {
        PendingLayer::ScrollTranslationForScrollHitTestLayer(self)
    }
    fn ApplyDecompositedTransform(&mut self, node: &Self::Transform) {
        self.offset_of_decomposited_transforms_ += node.Get2dTranslation();
        self.change_of_decomposited_transforms_ = self
            .change_of_decomposited_transforms_
            .max(node.NodeChanged());
    }
    fn FinishDecompositedTransforms(&mut self, remaining_transform: Self::Transform) {
        self.property_tree_state_
            .SetTransformNode(remaining_transform);
        let offset = self.offset_of_decomposited_transforms_;
        self.bounds_.Offset(offset.x(), offset.y());
        self.rect_known_to_be_opaque_.Offset(offset.x(), offset.y());
    }
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
        compositor_layerize::transform_decomposition::DecompositeTransforms(pending_layers);
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
            item.CreateOrReuseScrollbarLayer(old_layer, self.offset_of_decomposited_transforms_, c);
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
    pub fn Upcast(
        &mut self,
        new_state: &B::PropertyTreeState,
        geometry: &dyn PendingLayerGeometry<B>,
    ) {
        debug_assert!(!self.ChunkRequiresOwnLayer());
        debug_assert!(new_state.EffectUnaliasedParentSameAs(&self.property_tree_state_));
        let upcast = ComputePendingLayerUpcast(
            &self.MergePolicyInput(),
            new_state,
            self.property_tree_state_.EffectBlendMode() != BlendMode::kSrcOver,
            &MergeGeometryAdapter::<B>(geometry),
        );
        self.rect_known_to_be_opaque_ = upcast.rect_known_to_be_opaque;
        self.bounds_ = upcast.bounds;
        self.property_tree_state_ = upcast.property_tree_state;
        self.has_decomposited_blend_mode_ = upcast.has_decomposited_blend_mode;
        self.solid_color_chunk_index_ = K_NOT_FOUND;
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:98-99.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:115-119.
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:210-227;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:444-489.
    fn MergePolicyInput(
        &self,
    ) -> PendingLayerMergeInput<
        '_,
        B::PropertyTreeState,
        B::TransformNode,
        B::DomNodeId,
        HitTestOpaqueness,
    > {
        PendingLayerMergeInput {
            property_tree_state: &self.property_tree_state_,
            non_composited_scroll_translations: &self.non_composited_scroll_translations_,
            bounds: self.bounds_,
            rect_known_to_be_opaque: self.rect_known_to_be_opaque_,
            solid_color_chunk_index: self.IsSolidColor().then_some(self.solid_color_chunk_index_),
            solid_color_alpha: self.IsSolidColor().then(|| self.GetSolidColor().alpha),
            chunk_count: self.chunks_.len(),
            compositing_type: self.compositing_type_,
            hit_test_opaqueness: self.hit_test_opaqueness_,
            canvas_child_id: self.canvas_child_id_,
            has_text: self.has_text_,
            draws_content: self.draws_content_,
            text_known_to_be_on_opaque_background: self.text_known_to_be_on_opaque_background_,
            has_decomposited_blend_mode: self.has_decomposited_blend_mode_,
        }
    }

    pub fn CanMergeWithDecompositedBlendMode(
        &self,
        guest: &Self,
        upcast_state: &B::PropertyTreeState,
        merge_fixed_layers: bool,
        merge_sticky_layers: bool,
        is_composited_scroll: &mut dyn FnMut(&B::TransformNode) -> bool,
        geometry: &dyn PendingLayerGeometry<B>,
    ) -> bool {
        CanUpcastPendingLayers(
            &self.MergePolicyInput(),
            &guest.MergePolicyInput(),
            upcast_state,
            merge_fixed_layers,
            merge_sticky_layers,
            is_composited_scroll,
            &MergeGeometryAdapter::<B>(geometry),
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
    ) -> Option<PendingLayerMergeOutput<B::PropertyTreeState, HitTestOpaqueness>> {
        debug_assert!(Arc::ptr_eq(
            self.chunks_.artifact(),
            guest.chunks_.artifact()
        ));
        ComputePendingLayerMerge(
            &self.MergePolicyInput(),
            &guest.MergePolicyInput(),
            B::InvalidDomNodeId(),
            lcd_text_preference,
            device_pixel_ratio,
            is_composited_scroll,
            &MergeGeometryAdapter::<B>(geometry),
        )
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
        self.solid_color_chunk_index_ = merged.solid_color_chunk_index.unwrap_or(K_NOT_FOUND);
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
        MightOverlapPendingLayers(
            &self.MergePolicyInput(),
            &other.MergePolicyInput(),
            &MergeGeometryAdapter::<B>(geometry),
        )
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.h:199;
    // Source: third_party/blink/renderer/platform/graphics/compositing/pending_layer.cc:875-924.
    pub fn ComputeBackgroundColor(
        &self,
        colors: &dyn PendingLayerColor,
    ) -> crate::adapter::layer_tree_impl::Color4f {
        let samples = (0..self.chunks_.len()).rev().map(|index| {
            let chunk = self.chunks_.chunk(index);
            let background = chunk.background_color();
            let chunk_state = chunk.PropertiesUnalias();
            let effect = if chunk_state.SameEffectNode(&self.property_tree_state_) {
                BackgroundEffect::SameEffect
            } else if chunk_state.EffectUnaliasedParentSameAs(&self.property_tree_state_)
                && chunk_state.EffectIsOpacityOnly()
            {
                BackgroundEffect::OpacityOnly(chunk_state.EffectOpacity())
            } else {
                BackgroundEffect::Unsupported
            };
            BackgroundSample {
                color: background.color,
                covered_area: background.area,
                effect,
            }
        });
        ComputeLayerBackgroundColor(self.bounds_, samples, &BackgroundColorAdapter(colors))
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
