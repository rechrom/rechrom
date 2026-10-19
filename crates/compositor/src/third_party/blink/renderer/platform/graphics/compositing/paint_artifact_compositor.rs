// Translation status: complete
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:1-43.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1-52.
// Includes, namespaces and export/allocation decorations map to Rust modules.
use super::adjust_mask_layer_geometry::AdjustMaskLayerGeometry;
use crate::{
    adapter::{
        content_layer_client_impl::{ContentDisplayItemList, RasterInvalidator},
        geometry::{Rect, ToEnclosingRect, Vector2dF},
        input_types::ElementId,
        layer_tree_impl::LayerTreeTypes,
        main_layer::LayerContext,
        paint_artifact_compositor::{
            SynthesizedClipPaintServices, SynthesizedClipServices, SynthesizedClipTypes,
        },
        paint_chunks_to_cc_layer::{
            ConversionClipNode, ConversionDisplayItemList, ConversionOp, ConversionTransform,
        },
        pending_layer::PendingLayerTypes,
    },
    cc::layers::{
        content_layer_client::ContentLayerClient,
        layer::{LayerRef, PictureLayerRef},
        picture_layer::{PictureLayer, PictureLayerClientWeak},
    },
};
use std::{
    collections::HashSet,
    sync::{Arc, RwLock, Weak},
};

// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:44-56.
pub struct LayerListBuilder<B: PendingLayerTypes> {
    list_valid_: bool,
    list_: Vec<LayerRef<B>>,
    layer_ids_: HashSet<i32>,
}
impl<B: PendingLayerTypes> Default for LayerListBuilder<B> {
    fn default() -> Self {
        Self {
            list_valid_: true,
            list_: Vec::new(),
            layer_ids_: HashSet::new(),
        }
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:57-103.
// Only source fields are stored; native client identity is the same allocation's
// weak vtable. No LayerContext, host reference, or cached layer bounds is held.
pub struct SynthesizedClip<B: SynthesizedClipTypes> {
    rrect_is_local_: bool,
    path_: Option<B::PaintPath>,
    mask_effect_id_: ElementId,
    mask_isolation_id_: ElementId,
    layer_: Option<PictureLayerRef<B>>,
    rrect_: B::SkRoundedRect,
    projection_: <B as LayerTreeTypes>::Transform,
    identity_: PictureLayerClientWeak<B>,
    destroyed_: bool,
}
impl<B: SynthesizedClipTypes> SynthesizedClip<B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:71-76.
    pub fn new(services: &dyn SynthesizedClipServices<B>) -> Arc<RwLock<Self>> {
        Arc::new_cyclic(|weak: &Weak<RwLock<Self>>| {
            let client: PictureLayerClientWeak<B> = weak.clone();
            let isolation = services.NewUniqueObjectElementId();
            let effect = services.NewUniqueObjectElementId();
            RwLock::new(Self {
                projection_: services.IdentityTransform(),
                rrect_: Default::default(),
                layer_: None,
                mask_isolation_id_: isolation,
                mask_effect_id_: effect,
                path_: None,
                rrect_is_local_: false,
                identity_: client,
                destroyed_: false,
            })
        })
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:77-80.
    pub fn Destroy(&mut self, c: &mut LayerContext<'_, B>) {
        assert!(!self.destroyed_);
        if let Some(layer) = &self.layer_ {
            layer.WithPictureLayer(c, |layer, c| layer.ClearClient(c));
        }
        self.destroyed_ = true;
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:85-87.
    pub fn Layer(&self) -> Option<&PictureLayerRef<B>> {
        self.layer_.as_ref()
    }
    pub fn GetMaskIsolationId(&self) -> ElementId {
        self.mask_isolation_id_
    }
    pub fn GetMaskEffectId(&self) -> ElementId {
        self.mask_effect_id_
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:979-1026.
    pub fn UpdateLayer(
        &mut self,
        clip: &B::ClipNode,
        transform: &B::TransformNode,
        c: &mut LayerContext<'_, B>,
        services: &dyn SynthesizedClipServices<B>,
    ) {
        if self.layer_.is_none() {
            let layer = PictureLayer::<B>::Create(Some(self.identity_.clone()), c);
            layer.LayerRef().SetIsDrawable(true, c);
            layer.LayerRef().SetHitTestable(true, c);
            self.layer_ = Some(layer);
        }
        let layer = self.layer_.as_ref().unwrap();
        layer.WithPictureLayer(c, |layer, _| {
            assert!(layer
                .client()
                .is_some_and(|client| client.ptr_eq(&self.identity_)))
        });
        let path = clip.ClipPath();
        let mut new_rrect = services.ToSkRoundedRect(&clip.PaintClipRect());
        let mut layer_rect = ToEnclosingRect(services.RoundedRectRect(&clip.PaintClipRect()));
        let mut new_projection = services.Projection(
            &crate::adapter::pending_layer::PendingClipNode::LocalTransformSpaceUnalias(clip),
            transform,
        );
        layer_rect = services.MapRect(&new_projection, layer_rect);
        let mut offset = Vector2dF::new(layer_rect.x as f32, layer_rect.y as f32);
        let mut bounds = layer_rect.size();
        AdjustMaskLayerGeometry(services, transform, &mut offset, &mut bounds);
        services.PostTranslate(
            &mut new_projection,
            Vector2dF::new(-offset.x(), -offset.y()),
        );
        let needs_display;
        if path.is_none() && new_projection.IsIdentityOr2dTranslation() {
            services.OffsetSkRoundedRect(&mut new_rrect, new_projection.To2dTranslation());
            needs_display = !self.rrect_is_local_ || new_rrect != self.rrect_;
            services.MakeIdentity(&mut self.projection_);
            self.rrect_is_local_ = true;
        } else {
            needs_display = self.rrect_is_local_
                || new_rrect != self.rrect_
                || new_projection != self.projection_
                || !services.ClipPathEquals(clip, &self.path_);
            self.projection_ = new_projection;
            self.rrect_is_local_ = false;
        }
        if needs_display {
            layer.LayerRef().SetNeedsDisplay(c);
        }
        layer.LayerRef().SetOffsetToTransformParent(offset, c);
        layer.LayerRef().SetBounds(bounds, c);
        self.rrect_ = new_rrect;
        self.path_ = path;
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:89-92;
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1027-1054.
impl<'c, B: SynthesizedClipTypes> ContentLayerClient<B::ContentDisplayItemList, LayerContext<'c, B>>
    for SynthesizedClip<B>
{
    fn PaintContentsToDisplayList(
        &mut self,
        c: &mut LayerContext<'c, B>,
    ) -> Option<Arc<B::ContentDisplayItemList>> {
        let mut list = c.DerivedServices().CreateDisplayItemList();
        let mut flags = c.DerivedServices().NewPaintFlags();
        c.DerivedServices().SetAntiAlias(&mut flags, true);
        list.StartPaint();
        if self.rrect_is_local_ {
            c.DerivedServices()
                .PushDrawRRect(&mut list, &self.rrect_, flags);
        } else {
            list.Push(ConversionOp::Save);
            if self.projection_.IsIdentityOr2dTranslation() {
                let t = self.projection_.To2dTranslation();
                list.Push(ConversionOp::Translate(t.x(), t.y()));
            } else {
                list.Push(ConversionOp::Concat(self.projection_.clone()));
            }
            if let Some(path) = &self.path_ {
                list.Push(ConversionOp::ClipPath {
                    path: path.clone(),
                    difference: false,
                    antialias: true,
                });
            }
            c.DerivedServices()
                .PushDrawRRect(&mut list, &self.rrect_, flags);
            list.Push(ConversionOp::Restore);
        }
        let bounds = *self
            .layer_
            .as_ref()
            .expect("source layer")
            .LayerRef()
            .bounds(c);
        list.EndPaintOfUnpaired(Rect::new(0, 0, bounds.width(), bounds.height()));
        list.Finalize();
        Some(Arc::new(list))
    }
    fn FillsBoundsCompletely(&self) -> bool {
        false
    }
}
impl<B: SynthesizedClipTypes> Drop for SynthesizedClip<B> {
    fn drop(&mut self) {
        assert!(
            self.destroyed_,
            "native synthesized client ClearClient requires live finalization"
        );
    }
}
/// Native unique_ptr<SynthesizedClip> points at this one resident allocation;
/// PictureLayer receives only its nonowning dyn Weak interface.
pub struct SynthesizedClipRef<B: SynthesizedClipTypes>(Arc<RwLock<SynthesizedClip<B>>>);
impl<B: SynthesizedClipTypes> SynthesizedClipRef<B> {
    pub fn new(services: &dyn SynthesizedClipServices<B>) -> Self {
        Self(SynthesizedClip::new(services))
    }
    pub fn LayerRef(&self) -> Option<LayerRef<B>> {
        self.0
            .read()
            .unwrap()
            .Layer()
            .map(|layer| layer.LayerRef().clone())
    }
    pub fn GetMaskIsolationId(&self) -> ElementId {
        self.0.read().unwrap().GetMaskIsolationId()
    }
    pub fn GetMaskEffectId(&self) -> ElementId {
        self.0.read().unwrap().GetMaskEffectId()
    }
    pub fn UpdateLayer(
        &mut self,
        clip: &B::ClipNode,
        transform: &B::TransformNode,
        c: &mut LayerContext<'_, B>,
        services: &dyn SynthesizedClipServices<B>,
    ) {
        self.0
            .write()
            .unwrap()
            .UpdateLayer(clip, transform, c, services);
    }
    pub fn Release(self, c: &mut LayerContext<'_, B>) {
        self.0.write().unwrap().Destroy(c);
        drop(self);
    }
}
impl<B: PendingLayerTypes> LayerListBuilder<B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1803-1811.
    pub fn Add(&mut self, layer: LayerRef<B>) {
        debug_assert!(self.list_valid_);
        if self.layer_ids_.insert(layer.id()) {
            self.list_.push(layer);
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1812-1817.
    pub fn Finalize(&mut self) -> Vec<LayerRef<B>> {
        debug_assert!(self.list_valid_);
        self.list_valid_ = false;
        std::mem::take(&mut self.list_)
    }
}

use super::{content_layer_client_impl::CanvasStateRead, pending_layer::PendingLayer};
use crate::adapter::{
    paint_artifact_compositor::{
        ArtifactPaintChunk, PaintArtifactCompositorContext, PaintArtifactCompositorServices,
        PaintArtifactCompositorTypes,
    },
    pending_layer::{
        LcdTextPreference, PendingClipNode, PendingPaintArtifact, PendingPaintChunk,
        PendingPaintChunkSubset,
    },
};
use crate::cc::{layers::layer::Layer, trees::property_tree::ScrollCallbacks};
use std::collections::HashMap;

// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:120-130.
pub struct ViewportProperties<B: PendingLayerTypes> {
    pub overscroll_elasticity_transform: Option<B::TransformNode>,
    pub page_scale: Option<B::TransformNode>,
    pub inner_scroll_translation: Option<B::TransformNode>,
    pub outer_clip: Option<B::ClipNode>,
    pub outer_scroll_translation: Option<B::TransformNode>,
}
impl<B: PendingLayerTypes> Default for ViewportProperties<B> {
    fn default() -> Self {
        Self {
            overscroll_elasticity_transform: None,
            page_scale: None,
            inner_scroll_translation: None,
            outer_clip: None,
            outer_scroll_translation: None,
        }
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:131-158.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpdateType {
    kNone,
    kRasterInducingScroll,
    kRepaint,
    kFull,
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:276-293.
pub struct SynthesizedClipEntry<B: SynthesizedClipTypes> {
    pub clip_key: B::ClipNode,
    pub synthesized_clip: SynthesizedClipRef<B>,
    pub in_use: bool,
    pub transform_key: B::TransformNode,
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:368-381.
#[derive(Default)]
struct ScrollTranslationInfo {
    is_composited: bool,
    force_main_thread_repaint: bool,
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:104-119.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:122-127.
// The native GC Trace visits these actual four resident collections. Rust
// fields/retained Paint handles own those same roots directly; no visitor,
// copied owner graph, or second runtime collection is created.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:288-291.
// SynthesizedClipEntry's GC edges are its two retained native node handles.
// Reverse native owning-member order. All layer/client/PendingLayer instances
// are canonical resident owners, while node keys are native Paint handles.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:344-367.
pub struct PaintArtifactCompositor<B: PaintArtifactCompositorTypes> {
    range_dependent_scrolls_: HashSet<B::ScrollNode>,
    painted_scroll_translations_: HashMap<B::TransformNode, ScrollTranslationInfo>,
    get_canvas_snapshot_callback_: Option<B::CanvasSnapshotCallback>,
    canvas_child_layer_map_: HashMap<B::DomNodeId, usize>,
    pending_layers_: Vec<PendingLayer<B>>,
    synthesized_clip_cache_: Vec<SynthesizedClipEntry<B>>,
    root_layer_: Option<LayerRef<B>>,
    device_pixel_ratio_: f32,
    lcd_text_preference_: LcdTextPreference,
    previous_update_for_testing_: UpdateType,
    needs_update_: UpdateType,
    should_always_update_on_scroll_: bool,
    layer_debug_info_enabled_: bool,
    tracks_raster_invalidations_: bool,
    scroll_callbacks_: std::rc::Weak<dyn ScrollCallbacks>,
}
impl<B: PaintArtifactCompositorTypes> PaintArtifactCompositor<B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:113-128.
    pub fn new(
        scroll_callbacks: std::rc::Weak<dyn ScrollCallbacks>,
        c: &mut LayerContext<'_, B>,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) -> Self {
        let tracks = services.VlogIsOn(3);
        let root = Layer::<B>::Create(c.Services());
        Self {
            scroll_callbacks_: scroll_callbacks,
            tracks_raster_invalidations_: tracks,
            layer_debug_info_enabled_: false,
            should_always_update_on_scroll_: false,
            needs_update_: UpdateType::kFull,
            previous_update_for_testing_: UpdateType::kNone,
            lcd_text_preference_: LcdTextPreference::kIgnored,
            device_pixel_ratio_: 1.0,
            root_layer_: Some(root),
            synthesized_clip_cache_: Vec::new(),
            pending_layers_: Vec::new(),
            canvas_child_layer_map_: HashMap::new(),
            get_canvas_snapshot_callback_: None,
            painted_scroll_translations_: HashMap::new(),
            range_dependent_scrolls_: HashSet::new(),
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:159-179.
    pub fn SetNeedsUpdate(&mut self) {
        self.SetNeedsUpdateInternal(UpdateType::kFull);
    }
    pub fn SetNeedsUpdateForRasterInducingScroll(&mut self) {
        self.SetNeedsUpdateInternal(UpdateType::kRasterInducingScroll);
    }
    pub fn NeedsUpdate(&self) -> UpdateType {
        self.needs_update_
    }
    pub fn ClearNeedsUpdateForTesting(&mut self) {
        self.needs_update_ = UpdateType::kNone;
    }
    pub fn PreviousUpdateForTesting(&self) -> UpdateType {
        self.previous_update_for_testing_
    }
    pub fn ClearPreviousUpdateForTesting(&mut self) {
        self.previous_update_for_testing_ = UpdateType::kNone;
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:219-221.
    pub fn RootLayer(&self) -> Option<&LayerRef<B>> {
        self.root_layer_.as_ref()
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:224-229.
    pub fn SetGetCanvasSnapshotCallback(&mut self, callback: Option<B::CanvasSnapshotCallback>) {
        self.get_canvas_snapshot_callback_ = callback;
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:272-275.
    pub fn ShouldAlwaysUpdateOnScroll(&self) -> bool {
        self.should_always_update_on_scroll_
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:129-136.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:222-223.
    pub fn SetTracksRasterInvalidations(
        &mut self,
        track: bool,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) {
        self.tracks_raster_invalidations_ = track || services.VlogIsOn(3);
        for pending in &self.pending_layers_ {
            if let Some(client) = pending.GetContentLayerClient() {
                client
                    .GetRasterInvalidator()
                    .SetTracksRasterInvalidations(track);
            }
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:137-141.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:230-230.
    pub fn HasCanvasChildPaintRecord(&self, child: B::DomNodeId) -> bool {
        self.canvas_child_layer_map_.contains_key(&child)
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:165-174.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:233-235.
    pub fn GetCanvasChildPaintState(&self, child: B::DomNodeId) -> Option<CanvasStateRead<'_, B>> {
        let index = self.canvas_child_layer_map_.get(&child)?;
        self.pending_layers_[*index].canvas_child_paint_state()
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:175-178.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:236-239.
    pub fn WillBeRemovedFromFrame(&self, c: &mut LayerContext<'_, B>) {
        self.root_layer_
            .as_ref()
            .expect("source root layer")
            .RemoveAllChildren(c);
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:179-187.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:251-251.
    pub fn SetLCDTextPreference(&mut self, preference: LcdTextPreference) {
        if self.lcd_text_preference_ == preference {
            return;
        }
        self.SetNeedsUpdate();
        self.lcd_text_preference_ = preference;
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:188-195.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:252-253.
    pub fn SetDevicePixelRatio(&mut self, ratio: f32) {
        if self.device_pixel_ratio_ == ratio {
            return;
        }
        self.SetNeedsUpdate();
        self.device_pixel_ratio_ = ratio;
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:633-645.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:294-296.
    fn SetNeedsUpdateInternal(&mut self, update: UpdateType) {
        if (update == UpdateType::kRasterInducingScroll
            && self.needs_update_ == UpdateType::kRepaint)
            || (update == UpdateType::kRepaint
                && self.needs_update_ == UpdateType::kRasterInducingScroll)
        {
            self.needs_update_ = UpdateType::kFull;
        } else {
            self.needs_update_ = self.needs_update_.max(update);
        }
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:82-112.
struct OldPendingLayerMatcher<B: PendingLayerTypes> {
    next_index_: usize,
    pending_layers_: Vec<PendingLayer<B>>,
}
impl<B: PaintArtifactCompositorTypes> OldPendingLayerMatcher<B> {
    fn new(pending_layers: Vec<PendingLayer<B>>) -> Self {
        Self {
            next_index_: 0,
            pending_layers_: pending_layers,
        }
    }
    fn Find(&mut self, new_layer: &PendingLayer<B>) -> Option<&mut PendingLayer<B>> {
        if self.pending_layers_.is_empty() {
            return None;
        }
        if !new_layer.FirstPaintChunk().CanMatchOldChunk() {
            return None;
        }
        let mut index = self.next_index_;
        loop {
            let next = (index + 1) % self.pending_layers_.len();
            if new_layer.Matches(&self.pending_layers_[index]) {
                self.next_index_ = next;
                return Some(&mut self.pending_layers_[index]);
            }
            index = next;
            if index == self.next_index_ {
                break;
            }
        }
        None
    }
    fn ReleaseClients(&mut self, c: &mut LayerContext<'_, B>) {
        for pending in &mut self.pending_layers_ {
            pending.ReleaseContentLayerClient(c);
        }
    }
}

use super::pending_layer::CompositingType;
use crate::adapter::{
    paint_artifact_compositor::{
        ArtifactDisplayItem, ArtifactEffectNode, ArtifactPropertyState, ArtifactScrollNode,
        ArtifactTransformNode,
    },
    paint_chunks_to_cc_layer::{
        ConversionDisplayItem, ConversionHitTestData, ConversionPaintArtifact,
        ConversionPaintChunk, ConversionScrollNode, ConversionTransformNode,
    },
    pending_layer::{PendingEffectNode, PendingPropertyTreeState, PendingTransformNode},
};

// Rust borrow-only view used by the owner and its stack-scoped manager client.
// Every reference points to the same source field; no state or map is copied.
struct CompositingScrollState<'a, B: PaintArtifactCompositorTypes> {
    needs_update: &'a UpdateType,
    lcd: &'a LcdTextPreference,
    painted: &'a HashMap<B::TransformNode, ScrollTranslationInfo>,
    services: &'a dyn PaintArtifactCompositorServices<B>,
}
impl<B: PaintArtifactCompositorTypes> CompositingScrollState<'_, B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:251-282.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:315-316.
    fn NeedsCompositedScrolling(&self, translation: &B::TransformNode) -> bool {
        debug_assert!(*self.needs_update == UpdateType::kFull);
        debug_assert!(translation.ScrollNode().is_some());
        if translation.HasDirectCompositingReasons() {
            return true;
        }
        if let Some(info) = self.painted.get(translation) {
            return info.is_composited;
        }
        if self.services.CanvasDrawElementEnabledByRuntimeFlag()
            && ConversionScrollNode::<B>::CompositedScrollingNotPreferred(
                &translation.ScrollNode().unwrap(),
            )
        {
            return false;
        }
        *self.lcd != LcdTextPreference::kStronglyPreferred
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:283-290.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:317-319.
    fn ShouldForceMainThreadRepaint(&self, translation: &B::TransformNode) -> bool {
        debug_assert!(!self.NeedsCompositedScrolling(translation));
        self.painted
            .get(translation)
            .is_some_and(|info| info.force_main_thread_repaint)
    }
}
impl<B: PaintArtifactCompositorTypes> PaintArtifactCompositor<B> {
    fn ScrollState<'a>(
        &'a self,
        services: &'a dyn PaintArtifactCompositorServices<B>,
    ) -> CompositingScrollState<'a, B> {
        CompositingScrollState {
            needs_update: &self.needs_update_,
            lcd: &self.lcd_text_preference_,
            painted: &self.painted_scroll_translations_,
            services,
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:239-250.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:301-303.
    fn ScrollTranslationStateForLayer(pending: &PendingLayer<B>) -> B::TransformNode {
        if pending.GetCompositingType() == CompositingType::kScrollHitTestLayer {
            return pending.ScrollTranslationForScrollHitTestLayer();
        }
        pending
            .GetPropertyTreeState()
            .TransformNode()
            .ScrollTranslationState()
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:291-341.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:320-322.
    fn ComputeNeedsCompositedScrolling(&self, artifact: &B::PaintArtifact, cursor: usize) -> bool {
        let chunk = artifact
            .PaintChunk(cursor)
            .expect("source current PaintChunk");
        debug_assert!(chunk.hit_test_data().is_some());
        debug_assert!(chunk
            .hit_test_data()
            .unwrap()
            .scroll_translation()
            .is_some());
        debug_assert!(chunk.display_item_count() == 0);
        let translation = chunk.hit_test_data().unwrap().scroll_translation().unwrap();
        debug_assert!(translation.ScrollNode().is_some());
        if translation.HasDirectCompositingReasons() {
            return true;
        }
        let scroll = translation.ScrollNode().unwrap();
        if !scroll.UserScrollable() {
            return false;
        }
        if ConversionScrollNode::<B>::CompositedScrollingNotPreferred(&scroll) {
            return false;
        }
        if scroll.CompositedScrollingPreferred() {
            return true;
        }
        if self.lcd_text_preference_ != LcdTextPreference::kStronglyPreferred {
            return true;
        }
        for index in cursor + 1..artifact.PaintChunkCount() {
            let next = artifact.PaintChunk(index).unwrap();
            if next.SameRawTransform(chunk) {
                continue;
            }
            return next.UnaliasedTransform() == translation
                && Some(next.UnaliasedClip())
                    == ConversionScrollNode::<B>::OverflowClipNode(&scroll)
                && next.UnaliasedEffect() == chunk.UnaliasedEffect()
                && next
                    .opaque_rect()
                    .Contains(ConversionScrollNode::<B>::ContentsRect(&scroll));
        }
        true
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:342-414.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:323-325.
    fn UpdatePaintedScrollTranslationsBeforeLayerization(
        &mut self,
        artifact: &B::PaintArtifact,
        cursor: usize,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) {
        let chunk = artifact.PaintChunk(cursor).unwrap();
        let hit = chunk.hit_test_data();
        if let Some(translation) = hit.and_then(|hit| hit.scroll_translation()) {
            let composited = self.ComputeNeedsCompositedScrolling(artifact, cursor);
            match self.painted_scroll_translations_.entry(translation) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(ScrollTranslationInfo {
                        is_composited: composited,
                        force_main_thread_repaint: false,
                    });
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    if composited {
                        entry.get_mut().is_composited = true;
                        entry.get_mut().force_main_thread_repaint = false;
                    } else {
                        assert!(!entry.get().is_composited);
                    }
                }
            }
        }
        if services.RasterInducingScrollEnabled()
            && (hit.is_some_and(|hit| {
                hit.touch_action_rects().next().is_some()
                    || hit.wheel_event_rects().next().is_some()
                    || chunk.IsScrollbarHitTest()
                    || chunk.IsResizerScrollHitTest()
            }) || chunk.region_capture_data().is_some()
                || chunk.tracked_element_rects().is_some()
                || chunk.layer_selection_data().is_some())
        {
            let transform = chunk.PropertiesUnalias().TransformNode();
            let composited_ancestor = transform.NearestDirectlyCompositedAncestor();
            let mut translation = Some(transform.NearestScrollTranslationNode());
            while let Some(node) = translation {
                if node.NearestDirectlyCompositedAncestor() != composited_ancestor {
                    break;
                }
                match self.painted_scroll_translations_.entry(node.clone()) {
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(ScrollTranslationInfo {
                            is_composited: false,
                            force_main_thread_repaint: true,
                        });
                    }
                    std::collections::hash_map::Entry::Occupied(mut entry) => {
                        if entry.get().is_composited || entry.get().force_main_thread_repaint {
                            break;
                        }
                        entry.get_mut().force_main_thread_repaint = true;
                    }
                }
                translation = node.ParentScrollTranslationNode();
            }
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:415-441.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:326-328.
    fn ChunkCompositingType(
        &self,
        artifact: &B::PaintArtifact,
        chunk: &B::PaintChunk,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) -> CompositingType {
        if let Some(translation) = chunk
            .hit_test_data()
            .and_then(|data| data.scroll_translation())
        {
            if self
                .ScrollState(services)
                .NeedsCompositedScrolling(&translation)
            {
                return CompositingType::kScrollHitTestLayer;
            }
        }
        if chunk.display_item_count() == 1 {
            let item = artifact.DisplayItemsForChunk(chunk).next().unwrap();
            if item.IsForeignLayer() {
                return CompositingType::kForeignLayer;
            }
            if ConversionDisplayItem::<B>::IsScrollbar(item) {
                if let Some(translation) = item.ScrollTranslation() {
                    if services.RasterInducingScrollEnabled()
                        || self
                            .ScrollState(services)
                            .NeedsCompositedScrolling(&translation)
                    {
                        assert!(!chunk
                            .PropertiesUnalias()
                            .EffectNode()
                            .IsInDrawableCanvasSubtree());
                        return CompositingType::kScrollbarLayer;
                    }
                }
            }
        }
        CompositingType::kOther
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:442-447.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:329-330.
    fn AddRangeDependentScroll(&mut self, state: &B::PropertyTreeState) {
        self.range_dependent_scrolls_.insert(
            state
                .TransformNode()
                .NearestScrollTranslationNode()
                .ScrollNode()
                .expect("source scroll node"),
        );
    }
}

use crate::adapter::{
    paint_artifact_compositor::ArtifactDebugNode, paint_chunks_to_cc_layer::ConversionEffectNode,
};
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:448-459.
fn ForeignLayer<B: PaintArtifactCompositorTypes>(
    chunk: &B::PaintChunk,
    artifact: &B::PaintArtifact,
) -> Option<crate::cc::layers::layer::LayerBorrow<B>> {
    if chunk.display_item_count() != 1 {
        return None;
    }
    let item = artifact.DisplayItemsForChunk(chunk).next().unwrap();
    if item.IsForeignLayer() {
        Some(item.ForeignLayerBorrow())
    } else {
        None
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:460-467.
fn DescribePaintPropertyNode<B: PaintArtifactCompositorTypes, N: ArtifactDebugNode>(
    node: &N,
    parent: Option<&N>,
    services: &dyn PaintArtifactCompositorServices<B>,
) -> String {
    let parent_pointer = parent.map_or_else(
        || services.NullPointerForDebug(),
        ArtifactDebugNode::DebugPointer,
    );
    let parent_json = parent.map_or_else(String::new, ArtifactDebugNode::DebugJSONString);
    format!(
        "Parent {} : ({}) \n Self {} : ({})",
        parent_pointer,
        parent_json,
        node.DebugPointer(),
        node.DebugJSONString()
    )
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:468-515.
fn DumpWithDifferingPaintPropertiesIncluded<B: PaintArtifactCompositorTypes>(
    previous: &B::PaintChunk,
    repainted: &B::PaintChunk,
    services: &dyn PaintArtifactCompositorServices<B>,
) {
    let node = previous.RawTransformForDebug();
    let _previous_transform = services.CrashKey(
        "PrevTransform",
        "prev_tf_property_state",
        DescribePaintPropertyNode::<B, _>(&node, node.UnaliasedParent().as_ref(), services),
    );
    let node = previous.RawClipForDebug();
    let _previous_clip = services.CrashKey(
        "PrevClip",
        "json",
        DescribePaintPropertyNode::<B, _>(&node, node.UnaliasedParent().as_ref(), services),
    );
    let node = previous.RawEffectForDebug();
    let _previous_effect = services.CrashKey(
        "PrevEffect",
        "json",
        DescribePaintPropertyNode::<B, _>(&node, node.UnaliasedParent().as_ref(), services),
    );
    let node = repainted.RawTransformForDebug();
    let _repainted_transform = services.CrashKey(
        "RepaintTransform",
        "json",
        DescribePaintPropertyNode::<B, _>(&node, node.UnaliasedParent().as_ref(), services),
    );
    let node = repainted.RawClipForDebug();
    let _repainted_clip = services.CrashKey(
        "RepaintClip",
        "json",
        DescribePaintPropertyNode::<B, _>(&node, node.UnaliasedParent().as_ref(), services),
    );
    let node = repainted.RawEffectForDebug();
    let _repainted_effect = services.CrashKey(
        "RepaintEffect",
        "json",
        DescribePaintPropertyNode::<B, _>(&node, node.UnaliasedParent().as_ref(), services),
    );
    let _chunk_id = services.CrashKey("ChunkId", "id", previous.IdToString());
    services.DumpWithoutCrashing();
    debug_assert!(
        false,
        "Paint Property Tree State differs for Paint Chunk: Missing call to PaintArtifactCompositor::SetNeedsUpdate?"
    );
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:516-616.
fn NeedsFullUpdateAfterPaintingChunk<B: PaintArtifactCompositorTypes>(
    previous: &B::PaintChunk,
    previous_artifact: &B::PaintArtifact,
    repainted: &B::PaintChunk,
    repainted_artifact: &B::PaintArtifact,
    services: &dyn PaintArtifactCompositorServices<B>,
) -> bool {
    if !repainted.Matches(previous) {
        return true;
    }
    if repainted.is_moved_from_cached_subsequence() {
        debug_assert_eq!(previous.bounds_rect(), repainted.bounds_rect());
        debug_assert_eq!(previous.DrawsContent(), repainted.DrawsContent());
        debug_assert_eq!(previous.opaque_rect(), repainted.opaque_rect());
        debug_assert_eq!(
            previous.text_known_to_be_on_opaque_background(),
            repainted.text_known_to_be_on_opaque_background()
        );
        debug_assert_eq!(previous.has_text(), repainted.has_text());
        if !previous.SameRawProperties(repainted) {
            DumpWithDifferingPaintPropertiesIncluded::<B>(previous, repainted, services);
            return true;
        }
        return false;
    }
    if previous.bounds_rect() != repainted.bounds_rect() {
        return true;
    }
    if ForeignLayer::<B>(previous, previous_artifact)
        != ForeignLayer::<B>(repainted, repainted_artifact)
    {
        return true;
    }
    if previous.opaque_rect() != repainted.opaque_rect() {
        return true;
    }
    if previous.text_known_to_be_on_opaque_background()
        != repainted.text_known_to_be_on_opaque_background()
    {
        return true;
    }
    let transparent = crate::adapter::layer_tree_impl::Color4f::default();
    if (previous.background_color().color == transparent)
        != (repainted.background_color().color == transparent)
    {
        return true;
    }
    if previous.has_text() != repainted.has_text() {
        return true;
    }
    if previous.DrawsContent() != repainted.DrawsContent() {
        return true;
    }
    if previous.background_color().is_solid_color != repainted.background_color().is_solid_color {
        return true;
    }
    if previous.hit_test_opaqueness() != repainted.hit_test_opaqueness() {
        return true;
    }
    if !previous.SameRawProperties(repainted) {
        DumpWithDifferingPaintPropertiesIncluded::<B>(previous, repainted, services);
        return true;
    }
    false
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:617-629.
fn GetPropertyTreeStateForPaint<B: PaintArtifactCompositorTypes>(
    state: &B::PropertyTreeState,
) -> B::PropertyTreeState {
    let mut result = state.clone();
    let effect = state.EffectNode();
    if effect.HasCanvasChildState() {
        result.SetClipNode(effect.CanvasChildContentClip());
        result.SetEffectNode(effect.CanvasChildContentEffect());
        result.SetTransformNode(effect.CanvasChildContentTransform());
    }
    result
}
impl<B: PaintArtifactCompositorTypes> PaintArtifactCompositor<B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:646-671.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:164-165.
    pub fn SetNeedsUpdateAfterRepaint(
        &mut self,
        previous: &B::PaintArtifact,
        repainted: &B::PaintArtifact,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) {
        if self.needs_update_ == UpdateType::kFull {
            return;
        }
        if previous.PaintChunkCount() != repainted.PaintChunkCount() {
            self.SetNeedsUpdate();
            return;
        }
        for index in 0..previous.PaintChunkCount() {
            if NeedsFullUpdateAfterPaintingChunk::<B>(
                previous.PaintChunk(index).unwrap(),
                previous,
                repainted.PaintChunk(index).unwrap(),
                repainted,
                services,
            ) {
                self.SetNeedsUpdate();
                return;
            }
        }
        self.SetNeedsUpdateInternal(UpdateType::kRepaint);
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:672-682.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:254-257.
    pub fn HasComposited(&self, element: ElementId, c: &LayerContext<'_, B>) -> bool {
        debug_assert!(
            self.needs_update_ == UpdateType::kNone,
            "only after an update"
        );
        let host = self
            .root_layer_
            .as_ref()
            .expect("source root layer")
            .host_id()
            .expect("source attached host");
        c.Host(host).property_trees().HasElement(element)
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:683-745.
// The source stack object borrows the resident compositor and immutable artifact.
// Arc only retains that same artifact allocation for PendingLayer's native owner.
struct Layerizer<'a, B: PaintArtifactCompositorTypes> {
    layer_merge_distance_limit_: usize,
    compositor_: &'a mut PaintArtifactCompositor<B>,
    artifact_: Arc<B::PaintArtifact>,
    chunk_cursor_: usize,
    pending_layers_: Vec<PendingLayer<B>>,
    merged_sticky_transforms_: Vec<B::TransformNode>,
    directly_composited_transforms_: HashSet<B::TransformNode>,
    services: &'a dyn PaintArtifactCompositorServices<B>,
}
impl<'a, B: PaintArtifactCompositorTypes> Layerizer<'a, B> {
    fn new(
        compositor: &'a mut PaintArtifactCompositor<B>,
        artifact: Arc<B::PaintArtifact>,
        reserve_capacity: usize,
        services: &'a dyn PaintArtifactCompositorServices<B>,
    ) -> Self {
        let limit = services.LayerMergeDistanceLimit();
        Self {
            layer_merge_distance_limit_: usize::try_from(limit).unwrap_or(0),
            compositor_: compositor,
            artifact_: artifact,
            chunk_cursor_: 0,
            pending_layers_: Vec::with_capacity(reserve_capacity),
            merged_sticky_transforms_: Vec::new(),
            directly_composited_transforms_: HashSet::new(),
            services,
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:759-835.
    fn DecompositeEffect(
        &mut self,
        parent_effect: &B::EffectNode,
        first_layer_in_parent_group_index: usize,
        effect: &B::EffectNode,
        layer_index: usize,
    ) -> bool {
        debug_assert_eq!(layer_index, self.pending_layers_.len() - 1);
        let layer = &self.pending_layers_[layer_index];
        if layer.GetPropertyTreeState().EffectNode() != *effect {
            return false;
        }
        if layer.ChunkRequiresOwnLayer() {
            return false;
        }
        if effect.HasDirectCompositingReasons() {
            return false;
        }
        let group_state = self.services.MakeState(
            effect.LocalTransformSpaceUnalias(),
            effect
                .OutputClip()
                .unwrap_or_else(|| layer.GetPropertyTreeState().ClipNode()),
            effect.clone(),
        );
        let scroll = self.compositor_.ScrollState(self.services);
        let mut is_composited_scroll = |t: &B::TransformNode| scroll.NeedsCompositedScrolling(t);
        let Some(mut upcast_result) = group_state.CanUpcastWith(
            &layer.GetPropertyTreeState(),
            self.services.MergeFixedLayersEnabled(),
            self.services.MergeStickyLayersEnabled(),
            &mut is_composited_scroll,
        ) else {
            return false;
        };
        if upcast_result.scroll_range_dependent {
            self.compositor_
                .AddRangeDependentScroll(&upcast_result.upcasted_state);
        }
        upcast_result
            .upcasted_state
            .SetEffectNode(parent_effect.clone());
        if effect.BlendMode() != crate::adapter::property_types::BlendMode::kSrcOver {
            let num_previous_siblings = layer_index - first_layer_in_parent_group_index;
            if num_previous_siblings != 0 {
                if num_previous_siblings > 2 {
                    return false;
                }
                if num_previous_siblings == 2
                    && self.pending_layers_[first_layer_in_parent_group_index].DrawsContent()
                {
                    return false;
                }
                let previous_sibling = &self.pending_layers_[layer_index - 1];
                let layer = &self.pending_layers_[layer_index];
                let scroll = self.compositor_.ScrollState(self.services);
                let mut is_composited_scroll =
                    |t: &B::TransformNode| scroll.NeedsCompositedScrolling(t);
                if previous_sibling.DrawsContent()
                    && !previous_sibling.CanMergeWithDecompositedBlendMode(
                        layer,
                        &upcast_result.upcasted_state,
                        self.services.MergeFixedLayersEnabled(),
                        self.services.MergeStickyLayersEnabled(),
                        &mut is_composited_scroll,
                    )
                {
                    return false;
                }
            }
        }
        self.pending_layers_[layer_index].Upcast(&upcast_result.upcasted_state, self.services);
        true
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:836-965.
    fn LayerizeGroup(
        &mut self,
        current_group: &B::EffectNode,
        mut canvas_child_id: B::DomNodeId,
        force_draws_content: bool,
    ) {
        if current_group.CanvasChildId() != B::InvalidDomNodeId() {
            canvas_child_id = current_group.CanvasChildId();
        }
        let first_layer_in_current_group = self.pending_layers_.len();
        while self.chunk_cursor_ != self.artifact_.PaintChunkCount() {
            let chunk_effect = self
                .artifact_
                .PaintChunk(self.chunk_cursor_)
                .unwrap()
                .UnaliasedEffect();
            if chunk_effect == *current_group {
                self.compositor_
                    .UpdatePaintedScrollTranslationsBeforeLayerization(
                        &self.artifact_,
                        self.chunk_cursor_,
                        self.services,
                    );
                self.pending_layers_.push(PendingLayer::new(
                    self.artifact_.clone(),
                    self.chunk_cursor_,
                    canvas_child_id,
                    self.compositor_.ChunkCompositingType(
                        &self.artifact_,
                        self.artifact_.PaintChunk(self.chunk_cursor_).unwrap(),
                        self.services,
                    ),
                    self.services,
                ));
                self.chunk_cursor_ += 1;
                if self.pending_layers_.last().unwrap().ChunkRequiresOwnLayer() {
                    continue;
                }
            } else {
                let Some(subgroup) =
                    StrictUnaliasedChildOfAlongPath::<B>(current_group, &chunk_effect)
                else {
                    break;
                };
                let first_layer_in_subgroup = self.pending_layers_.len();
                self.LayerizeGroup(
                    &subgroup,
                    canvas_child_id,
                    force_draws_content || subgroup.DrawsContent(),
                );
                if first_layer_in_subgroup != self.pending_layers_.len().wrapping_sub(1) {
                    continue;
                }
                if !self.DecompositeEffect(
                    current_group,
                    first_layer_in_current_group,
                    &subgroup,
                    first_layer_in_subgroup,
                ) {
                    continue;
                }
            }
            let new_layer = self.pending_layers_.last_mut().unwrap();
            debug_assert!(!new_layer.ChunkRequiresOwnLayer());
            debug_assert!(new_layer.GetPropertyTreeState().EffectNode() == *current_group);
            if force_draws_content {
                new_layer.ForceDrawsContent();
            }
            let composited_transform = new_layer
                .GetPropertyTreeState()
                .TransformNode()
                .NearestDirectlyCompositedAncestor();
            if let Some(transform) = &composited_transform {
                if (!self.services.MergeFixedLayersEnabled()
                    || !transform.RequiresCompositingForFixedPositionOnly())
                    && (!self.services.MergeStickyLayersEnabled()
                        || !transform.RequiresCompositingForStickyPositionOnly())
                    && self
                        .directly_composited_transforms_
                        .insert(transform.clone())
                {
                    continue;
                }
            }
            let scroll = self.compositor_.ScrollState(self.services);
            let mut is_composited_scroll =
                |t: &B::TransformNode| scroll.NeedsCompositedScrolling(t);
            let mut candidate_index = self.pending_layers_.len() - 1;
            while candidate_index > first_layer_in_current_group
                && self.pending_layers_.len() - candidate_index <= self.layer_merge_distance_limit_
            {
                candidate_index -= 1;
                let new_index = self.pending_layers_.len() - 1;
                let (previous, last) = self.pending_layers_.split_at_mut(new_index);
                let candidate_layer = &mut previous[candidate_index];
                let new_layer = &mut last[0];
                let merge_result = candidate_layer.Merge(
                    new_layer,
                    self.compositor_.lcd_text_preference_,
                    self.compositor_.device_pixel_ratio_,
                    &mut is_composited_scroll,
                    self.services,
                );
                if merge_result.merged {
                    if let Some(transform) = &composited_transform {
                        if transform.RequiresCompositingForStickyPositionOnly()
                            && candidate_layer
                                .GetPropertyTreeState()
                                .TransformNode()
                                .NearestDirectlyCompositedAncestor()
                                .as_ref()
                                != Some(transform)
                        {
                            self.merged_sticky_transforms_.push(transform.clone());
                        }
                    }
                    let range_state = merge_result
                        .scroll_range_dependent
                        .then(|| candidate_layer.GetPropertyTreeState());
                    self.pending_layers_.pop();
                    if let Some(state) = range_state {
                        self.compositor_.AddRangeDependentScroll(&state);
                    }
                    break;
                }
                if new_layer.MightOverlap(candidate_layer, self.services) {
                    new_layer.SetCompositingTypeToOverlap();
                    break;
                }
            }
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:966-978.
    fn Layerize(mut self) -> (Vec<PendingLayer<B>>, Vec<B::TransformNode>) {
        self.LayerizeGroup(&B::RootEffectNode(), B::InvalidDomNodeId(), false);
        debug_assert_eq!(self.chunk_cursor_, self.artifact_.PaintChunkCount());
        // Source dependency: third_party/blink/renderer/platform/wtf/vector.h:1678-1684.
        if self.pending_layers_.len() * 2 < self.pending_layers_.capacity() {
            self.pending_layers_
                .shrink_to(self.pending_layers_.len() + self.pending_layers_.len() / 4 + 1);
        }
        (self.pending_layers_, self.merged_sticky_transforms_)
    }
}
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:746-758.
fn StrictUnaliasedChildOfAlongPath<B: PaintArtifactCompositorTypes>(
    ancestor: &B::EffectNode,
    node: &B::EffectNode,
) -> Option<B::EffectNode> {
    let mut n = Some(node.clone());
    while let Some(current) = n {
        let parent = current.UnaliasedParent();
        if parent.as_ref() == Some(ancestor) {
            return Some(current);
        }
        n = parent;
    }
    None
}

use super::property_tree_manager::PropertyTreeManager;
use crate::adapter::property_tree_manager::PropertyTreeManagerClient;
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1055-1104.
// This one source body is shared by the owner entry point and the manager's
// call-scoped client, borrowing only the native fields needed at this call.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:304-314.
fn CreateOrReuseSynthesizedClipLayer<'a, B: PaintArtifactCompositorTypes>(
    cache: &'a mut Vec<SynthesizedClipEntry<B>>,
    root: &LayerRef<B>,
    layer_debug_info_enabled: bool,
    should_always_update_on_scroll: &mut bool,
    clip: &B::ClipNode,
    transform: &B::TransformNode,
    needs_layer: bool,
    mask_isolation_id: &mut ElementId,
    mask_effect_id: &mut ElementId,
    c: &mut LayerContext<'_, B>,
    services: &dyn PaintArtifactCompositorServices<B>,
) -> &'a mut SynthesizedClipRef<B> {
    let entry_index = cache.iter().position(|entry| {
        entry.clip_key == *clip && !entry.in_use && entry.transform_key == *transform
    });
    let index = entry_index.unwrap_or_else(|| {
        cache.push(SynthesizedClipEntry {
            clip_key: clip.clone(),
            synthesized_clip: SynthesizedClipRef::new(services),
            in_use: false,
            transform_key: transform.clone(),
        });
        cache.len() - 1
    });
    let entry = &mut cache[index];
    entry.in_use = true;
    let synthesized_clip = &mut entry.synthesized_clip;
    if needs_layer {
        synthesized_clip.UpdateLayer(clip, transform, c, services);
        let layer = synthesized_clip
            .LayerRef()
            .expect("source materialized mask layer");
        layer.SetLayerTreeHost(root.host_id(), c);
        if layer_debug_info_enabled && layer.debug_info(c).is_none() {
            layer.SetDebugName("Synthesized Clip".into(), c);
        }
    }
    if !*should_always_update_on_scroll {
        let clip_transform = clip.LocalTransformSpaceUnalias();
        if clip_transform != *transform
            && clip_transform.PropertyTreeNearestScrollTranslationNode()
                != transform.PropertyTreeNearestScrollTranslationNode()
        {
            *should_always_update_on_scroll = true;
        }
    }
    *mask_isolation_id = synthesized_clip.GetMaskIsolationId();
    *mask_effect_id = synthesized_clip.GetMaskEffectId();
    synthesized_clip
}
// Stack-scoped split of native PropertyTreeManagerClient's PAC receiver. It
// stores field borrows, never another compositor owner or LayerContext.
struct PropertyTreeManagerClientFacade<'a, B: PaintArtifactCompositorTypes> {
    cache: &'a mut Vec<SynthesizedClipEntry<B>>,
    root: &'a LayerRef<B>,
    layer_debug_info_enabled: bool,
    should_always_update_on_scroll: &'a mut bool,
    needs_update: &'a UpdateType,
    lcd: &'a LcdTextPreference,
    painted_mut: Option<&'a mut HashMap<B::TransformNode, ScrollTranslationInfo>>,
    painted_shared: Option<&'a HashMap<B::TransformNode, ScrollTranslationInfo>>,
    services: &'a dyn PaintArtifactCompositorServices<B>,
}
impl<'a, B: PaintArtifactCompositorTypes> PropertyTreeManagerClientFacade<'a, B> {
    fn Painted(&self) -> &HashMap<B::TransformNode, ScrollTranslationInfo> {
        if let Some(value) = self.painted_shared {
            value
        } else {
            self.painted_mut
                .as_deref()
                .expect("source viewport write phase")
        }
    }
    // Native writes finish with viewport registration. Narrow the unique
    // borrow of this same resident map to shared references for the source
    // iteration and synchronous client queries; no map/value is copied.
    fn FinishViewportWrites(&mut self) -> &'a HashMap<B::TransformNode, ScrollTranslationInfo> {
        let value: &'a HashMap<B::TransformNode, ScrollTranslationInfo> = self
            .painted_mut
            .take()
            .expect("finish native viewport writes once");
        self.painted_shared = Some(value);
        value
    }
}
impl<B: PaintArtifactCompositorTypes> PropertyTreeManagerClient<B>
    for PropertyTreeManagerClientFacade<'_, B>
{
    fn CreateOrReuseSynthesizedClipLayer(
        &mut self,
        clip: &B::ClipNode,
        transform: &B::TransformNode,
        needs_layer: bool,
        mask_isolation_id: &mut ElementId,
        mask_effect_id: &mut ElementId,
        c: &mut LayerContext<'_, B>,
    ) -> &mut SynthesizedClipRef<B> {
        CreateOrReuseSynthesizedClipLayer(
            self.cache,
            self.root,
            self.layer_debug_info_enabled,
            self.should_always_update_on_scroll,
            clip,
            transform,
            needs_layer,
            mask_isolation_id,
            mask_effect_id,
            c,
            self.services,
        )
    }
    fn NeedsCompositedScrolling(&self, node: &B::TransformNode) -> bool {
        CompositingScrollState {
            needs_update: self.needs_update,
            lcd: self.lcd,
            painted: self.Painted(),
            services: self.services,
        }
        .NeedsCompositedScrolling(node)
    }
    fn ShouldForceMainThreadRepaint(&self, node: &B::TransformNode) -> bool {
        CompositingScrollState {
            needs_update: self.needs_update,
            lcd: self.lcd,
            painted: self.Painted(),
            services: self.services,
        }
        .ShouldForceMainThreadRepaint(node)
    }
}
impl<B: PaintArtifactCompositorTypes> PaintArtifactCompositor<B> {
    pub fn CreateOrReuseSynthesizedClipLayer(
        &mut self,
        clip: &B::ClipNode,
        transform: &B::TransformNode,
        needs_layer: bool,
        mask_isolation_id: &mut ElementId,
        mask_effect_id: &mut ElementId,
        c: &mut LayerContext<'_, B>,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) -> &mut SynthesizedClipRef<B> {
        CreateOrReuseSynthesizedClipLayer(
            &mut self.synthesized_clip_cache_,
            self.root_layer_.as_ref().expect("source root"),
            self.layer_debug_info_enabled_,
            &mut self.should_always_update_on_scroll_,
            clip,
            transform,
            needs_layer,
            mask_isolation_id,
            mask_effect_id,
            c,
            services,
        )
    }
}

impl<B: PaintArtifactCompositorTypes> PaintArtifactCompositor<B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1447-1457.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:336-337.
    pub fn CanDirectlyUpdateProperties(&self, c: &LayerContext<'_, B>) -> bool {
        if self.needs_update_ == UpdateType::kFull {
            return false;
        }
        self.root_layer_
            .as_ref()
            .and_then(|root| root.host_id())
            .is_some_and(|host| !c.Host(host).in_will_commit())
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1458-1469.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:197-197.
    pub fn DirectlyUpdateCompositedOpacityValue(
        &self,
        effect: &B::EffectNode,
        c: &mut LayerContext<'_, B>,
    ) -> bool {
        debug_assert!(effect.HasDirectCompositingReasons());
        if self.CanDirectlyUpdateProperties(c) {
            return PropertyTreeManager::<B>::DirectlyUpdateCompositedOpacityValue(
                c.HostMut(self.root_layer_.as_ref().unwrap().host_id().unwrap()),
                effect,
            );
        }
        false
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1470-1478.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:198-198.
    pub fn DirectlyUpdateScrollOffsetTransform(
        &self,
        transform: &B::TransformNode,
        c: &mut LayerContext<'_, B>,
    ) -> bool {
        if self.CanDirectlyUpdateProperties(c) {
            return PropertyTreeManager::<B>::DirectlyUpdateScrollOffsetTransform(
                self.root_layer_.as_ref().unwrap().host_id().unwrap(),
                transform,
                c,
            );
        }
        false
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1479-1495.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:199-199.
    pub fn DirectlyUpdateTransform(
        &self,
        transform: &B::TransformNode,
        c: &mut LayerContext<'_, B>,
    ) -> bool {
        debug_assert!(transform.HasDirectCompositingReasons());
        debug_assert!(transform.HasActiveTransformAnimation());
        if self.CanDirectlyUpdateProperties(c) {
            return PropertyTreeManager::<B>::DirectlyUpdateTransform(
                c.HostMut(self.root_layer_.as_ref().unwrap().host_id().unwrap()),
                transform,
            );
        }
        false
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1496-1507.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:200-200.
    pub fn DirectlyUpdatePageScaleTransform(
        &self,
        transform: &B::TransformNode,
        c: &mut LayerContext<'_, B>,
    ) -> bool {
        debug_assert!(transform.HasDirectCompositingReasons());
        if self.CanDirectlyUpdateProperties(c) {
            return PropertyTreeManager::<B>::DirectlyUpdatePageScaleTransform(
                c.HostMut(self.root_layer_.as_ref().unwrap().host_id().unwrap()),
                transform,
            );
        }
        false
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1508-1519.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:201-202.
    pub fn DirectlyUpdateScrollingContentsCullRect(
        &self,
        scroll: &B::ScrollNode,
        c: &mut LayerContext<'_, B>,
    ) -> bool {
        let scrolling_contents_cull_rect_enabled =
            c.Services().ScrollingContentsCullRectOnScrollNodeEnabled();
        assert!(scrolling_contents_cull_rect_enabled);
        if self.CanDirectlyUpdateProperties(c) && !self.range_dependent_scrolls_.contains(scroll) {
            PropertyTreeManager::<B>::DirectlyUpdateScrollingContentsCullRect(
                c.HostMut(self.root_layer_.as_ref().unwrap().host_id().unwrap()),
                scroll,
                scrolling_contents_cull_rect_enabled,
            );
            return true;
        }
        false
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1520-1534.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:203-208.
    pub fn DirectlySetScrollOffset(
        &self,
        element_id: ElementId,
        offset: &crate::adapter::geometry::PointF,
        c: &mut LayerContext<'_, B>,
    ) -> bool {
        let Some(host) = self.root_layer_.as_ref().and_then(|root| root.host_id()) else {
            return false;
        };
        if c.Host(host).in_will_commit() {
            return false;
        }
        if c.Host(host)
            .property_trees()
            .scroll_tree()
            .FindNodeFromElementId(element_id)
            .is_none()
        {
            return false;
        }
        PropertyTreeManager::<B>::DirectlySetScrollOffset(host, element_id, offset, c);
        true
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1535-1547.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:209-210.
    pub fn DropCompositorScrollDeltaNextCommit(
        &self,
        element_id: ElementId,
        c: &mut LayerContext<'_, B>,
    ) {
        let Some(host) = self.root_layer_.as_ref().and_then(|root| root.host_id()) else {
            return;
        };
        if c.Host(host)
            .property_trees()
            .scroll_tree()
            .FindNodeFromElementId(element_id)
            .is_none()
        {
            return;
        }
        PropertyTreeManager::<B>::DropCompositorScrollDeltaNextCommit(c.HostMut(host), element_id);
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1548-1558.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:211-215.
    pub fn GetMainThreadRepaintReasons(
        &self,
        scroll: &B::ScrollNode,
        c: &LayerContext<'_, B>,
    ) -> crate::cc::input::main_thread_scrolling_reason::MainThreadRepaintReasons {
        let root = self.root_layer_.as_ref().expect("source root CHECK");
        let Some(host) = root.host_id() else {
            return Default::default();
        };
        PropertyTreeManager::<B>::GetMainThreadRepaintReasons(c.Host(host), scroll)
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1559-1568.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:216-216.
    pub fn UsesCompositedScrolling(&self, scroll: &B::ScrollNode, c: &LayerContext<'_, B>) -> bool {
        let root = self.root_layer_.as_ref().expect("source root CHECK");
        let Some(host) = root.host_id() else {
            return false;
        };
        PropertyTreeManager::<B>::UsesCompositedScrolling(c.Host(host), scroll)
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1569-1578.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:217-218.
    pub fn UsesRasterInducingScroll(
        &self,
        scroll: &B::ScrollNode,
        c: &LayerContext<'_, B>,
    ) -> bool {
        let root = self.root_layer_.as_ref().expect("source root CHECK");
        let Some(host) = root.host_id() else {
            return false;
        };
        PropertyTreeManager::<B>::UsesRasterInducingScroll(c.Host(host), scroll)
    }
}

use crate::adapter::paint_artifact_compositor::{
    ArtifactCanvasChildPaintRecord, ArtifactChunkSubset, ArtifactDisplayItemType,
    ArtifactInvalidationTracking, ArtifactLayersAsJson, ArtifactPaintRecord, ArtifactRecordOp,
    CompositingReason, CompositingReasons,
};
use std::collections::BTreeMap;
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:62-79.
fn FindCustomDataPlaceholders<B: PaintArtifactCompositorTypes>(
    record: &B::PaintRecord,
    callback: &B::CanvasSnapshotCallback,
    replacements: &mut BTreeMap<u32, B::PaintRecord>,
    services: &dyn PaintArtifactCompositorServices<B>,
) {
    for op in record.ops() {
        match op {
            ArtifactRecordOp::CustomData(id) => {
                if let Some(snapshot) =
                    services.CanvasSnapshot(callback, services.DomNodeIdFromCustomData(id))
                {
                    replacements.insert(id, snapshot);
                }
            }
            ArtifactRecordOp::DrawRecord(record) => {
                FindCustomDataPlaceholders::<B>(record, callback, replacements, services)
            }
            ArtifactRecordOp::Other => {}
        }
    }
}
impl<B: PaintArtifactCompositorTypes> PaintArtifactCompositor<B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:142-164.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:231-232.
    pub fn GetCanvasChildPaintRecord(
        &self,
        child: B::DomNodeId,
        c: &mut LayerContext<'_, B>,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) -> Option<B::CanvasChildPaintRecord> {
        let index = *self.canvas_child_layer_map_.get(&child)?;
        let mut record = self.pending_layers_[index].GetCanvasChildPaintRecord(c, services)?;
        if let Some(callback) = self.get_canvas_snapshot_callback_.as_ref() {
            let mut replacements = BTreeMap::new();
            FindCustomDataPlaceholders::<B>(record.record(), callback, &mut replacements, services);
            if !replacements.is_empty() {
                *record.record_mut() = record.record().ReplaceCustomData(&replacements);
            }
        }
        Some(record)
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:196-204.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:240-241.
    pub fn GetPendingLayersAsJSON(
        &self,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) -> B::JsonArray {
        let mut result = services.NewJsonArray();
        for layer in &self.pending_layers_ {
            services.AppendPendingLayerJson(&mut result, layer.ToJSON());
        }
        result
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:205-238.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:242-243.
    pub fn GetLayersAsJSON(
        &self,
        mut flags: super::content_layer_client_impl::LayerTreeFlags,
        c: &LayerContext<'_, B>,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) -> B::Json {
        if !self.tracks_raster_invalidations_ {
            flags.0 &= !((1 << 1) | (1 << 2));
        }
        let mut json = services.NewLayersAsJson(flags);
        let root = self.root_layer_.as_ref().expect("source root");
        for layer in root.children(c).iter() {
            let mut client = None;
            let mut transform = None;
            for pending in &self.pending_layers_ {
                if *layer == pending.CcLayer() {
                    client = pending.GetContentLayerClient();
                    transform = Some(pending.GetPropertyTreeState().TransformNode());
                    break;
                }
            }
            if transform.is_none() {
                for pending in &self.pending_layers_ {
                    let node = pending.GetPropertyTreeState().TransformNode();
                    if node.CcNodeId(layer.property_tree_sequence_number(c))
                        == layer.transform_tree_index(c)
                    {
                        transform = Some(node);
                        break;
                    }
                }
            }
            debug_assert!(transform.is_some());
            json.AddLayer(
                layer,
                &transform.expect("native DCHECK transform"),
                client,
                c,
            );
        }
        json.Finalize()
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1746-1754.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:260-261.
    pub fn SynthesizedClipLayersForTesting(&self) -> Vec<Option<LayerRef<B>>> {
        let mut layers = Vec::new();
        for entry in &self.synthesized_clip_cache_ {
            layers.push(entry.synthesized_clip.LayerRef());
        }
        layers
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1755-1769.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:262-263.
    pub fn ApproximateUnsharedMemoryUsage(&self) -> usize {
        let mut result = std::mem::size_of::<Self>()
            + self.synthesized_clip_cache_.capacity()
                * std::mem::size_of::<SynthesizedClipEntry<B>>()
            + self.pending_layers_.capacity() * std::mem::size_of::<PendingLayer<B>>();
        for layer in &self.pending_layers_ {
            if let Some(client) = layer.GetContentLayerClient() {
                result += client.ApproximateUnsharedMemoryUsage();
            }
            let chunks_size = layer.Chunks().ApproximateUnsharedMemoryUsage();
            debug_assert!(chunks_size >= std::mem::size_of::<B::PaintChunkSubset>());
            result += chunks_size - std::mem::size_of::<B::PaintChunkSubset>();
        }
        result
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1818-1826.
    #[cfg(debug_assertions)]
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:244-247.
    pub fn ShowDebugData(
        &self,
        c: &LayerContext<'_, B>,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) {
        services.DebugShowLayers(self.GetLayersAsJSON(
            super::content_layer_client_impl::LayerTreeFlags((1 << 0) | (1 << 2)),
            c,
            services,
        ));
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1827-1837.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:248-250.
    pub fn ContentLayerClientForTesting(
        &self,
        mut index: usize,
    ) -> Option<&super::content_layer_client_impl::ContentLayerClientRef<B>> {
        for layer in &self.pending_layers_ {
            if let Some(client) = layer.GetContentLayerClient() {
                if index == 0 {
                    return Some(client);
                }
                index -= 1;
            }
        }
        None
    }
}

impl<B: PaintArtifactCompositorTypes> PaintArtifactCompositor<B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1579-1594.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:258-259.
    pub fn SetLayerDebugInfoEnabled(&mut self, enabled: bool, c: &mut LayerContext<'_, B>) {
        if enabled == self.layer_debug_info_enabled_ {
            return;
        }
        debug_assert!(self.needs_update_ == UpdateType::kFull);
        self.layer_debug_info_enabled_ = enabled;
        let root = self.root_layer_.as_ref().expect("source root");
        if enabled {
            root.SetDebugName("root".into(), c);
        } else {
            root.ClearDebugInfo(c);
            for layer in root.children(c).iter() {
                layer.ClearDebugInfo(c);
            }
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1595-1628.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:342-343.
    pub fn UpdateDebugInfo(
        &self,
        c: &mut LayerContext<'_, B>,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) {
        if !self.layer_debug_info_enabled_ {
            return;
        }
        let mut previous_layer_state = services.MakeState(
            B::RootTransformNode(),
            B::RootClipNode(),
            B::RootEffectNode(),
        );
        for pending in &self.pending_layers_ {
            let layer = pending.CcLayer();
            let mut invalidator = pending
                .GetContentLayerClient()
                .map(|client| client.GetRasterInvalidator());
            let tracking = invalidator
                .as_mut()
                .and_then(|value| value.GetTrackingMutable());
            // Both guards project resident leaf objects. EnsureDebugInfo cannot
            // reenter this ContentLayerClient; no guard crosses a paint callback.
            let mut info = layer.EnsureDebugInfo(c);
            info.name = pending.DebugName();
            if self.needs_update_ == UpdateType::kFull {
                let reasons = self.GetCompositingReasons(pending, &previous_layer_state, services);
                info.compositing_reasons = services.Descriptions(&reasons);
                info.compositing_reason_ids = services.ShortNames(&reasons);
            }
            info.owner_node_id = pending.OwnerNodeId();
            if services.IsTracingRasterInvalidations() {
                if let Some(tracking) = tracking {
                    tracking.AddToLayerDebugInfo(&mut info);
                    tracking.ClearInvalidations();
                }
            }
            previous_layer_state = pending.GetPropertyTreeState();
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1629-1745.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:338-341.
    pub fn GetCompositingReasons(
        &self,
        layer: &PendingLayer<B>,
        previous: &B::PropertyTreeState,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) -> B::CompositingReasons {
        debug_assert!(self.layer_debug_info_enabled_);
        debug_assert!(self.needs_update_ == UpdateType::kFull);
        let singleton = |reason| {
            let mut result = B::CompositingReasons::default();
            result.Put(reason);
            result
        };
        if layer.GetCompositingType() == CompositingType::kScrollHitTestLayer {
            return singleton(CompositingReason::OverflowScrolling);
        }
        if layer.Chunks().len() == 1 && layer.FirstPaintChunk().display_item_count() == 1 {
            let item = layer
                .Chunks()
                .artifact()
                .DisplayItemsForChunk(layer.FirstPaintChunk())
                .next()
                .unwrap();
            let reason = match item.GetType() {
                ArtifactDisplayItemType::FixedAttachmentBackground => {
                    Some(CompositingReason::FixedAttachmentBackground)
                }
                ArtifactDisplayItemType::Caret => Some(CompositingReason::Caret),
                ArtifactDisplayItemType::ScrollbarHorizontal
                | ArtifactDisplayItemType::ScrollbarVertical => Some(CompositingReason::Scrollbar),
                ArtifactDisplayItemType::ForeignLayerCanvas => Some(CompositingReason::Canvas),
                ArtifactDisplayItemType::ForeignLayerDevToolsOverlay => {
                    Some(CompositingReason::DevToolsOverlay)
                }
                ArtifactDisplayItemType::ForeignLayerPlugin => Some(CompositingReason::Plugin),
                ArtifactDisplayItemType::ForeignLayerVideo => Some(CompositingReason::Video),
                ArtifactDisplayItemType::ForeignLayerRemoteFrame => Some(CompositingReason::IFrame),
                ArtifactDisplayItemType::ForeignLayerLinkHighlight => {
                    Some(CompositingReason::LinkHighlight)
                }
                ArtifactDisplayItemType::ForeignLayerViewportScroll => {
                    Some(CompositingReason::Viewport)
                }
                ArtifactDisplayItemType::ForeignLayerViewportScrollbar => {
                    Some(CompositingReason::Scrollbar)
                }
                ArtifactDisplayItemType::ForeignLayerViewTransitionContent => {
                    Some(CompositingReason::ViewTransitionContent)
                }
                _ => None,
            };
            if let Some(reason) = reason {
                return singleton(reason);
            }
        }
        let mut reasons = B::CompositingReasons::default();
        let transform = layer.GetPropertyTreeState().TransformNode();
        if transform.IsBackfaceHidden() && !previous.TransformNode().IsBackfaceHidden() {
            reasons = singleton(CompositingReason::BackfaceVisibilityHidden);
        }
        if layer.GetCompositingType() == CompositingType::kOverlap {
            if reasons.empty() {
                return singleton(CompositingReason::Overlap);
            }
            return reasons;
        }
        let composited_ancestor = |transform: &B::TransformNode| {
            let ancestor = transform.NearestDirectlyCompositedAncestor();
            let scroll = transform.PropertyTreeNearestScrollTranslationNode();
            if self.ScrollState(services).NeedsCompositedScrolling(&scroll)
                && ancestor
                    .as_ref()
                    .is_none_or(|value| value.IsAncestorOf(&scroll))
            {
                return Some(scroll);
            }
            ancestor
        };
        let transform_reasons = |transform: &B::TransformNode, previous: &B::TransformNode| {
            let mut reasons = B::CompositingReasons::default();
            if let Some(ancestor) = composited_ancestor(transform) {
                if Some(ancestor.clone()) != composited_ancestor(previous) {
                    reasons = ancestor.DirectCompositingReasonsForDebugging();
                    if ancestor.ScrollNode().is_some() {
                        reasons.Put(CompositingReason::OverflowScrolling);
                    }
                }
            }
            reasons
        };
        let clip_reasons = |clip: &B::ClipNode, previous: &B::ClipNode| {
            transform_reasons(
                &clip.LocalTransformSpaceUnalias(),
                &previous.LocalTransformSpaceUnalias(),
            )
        };
        reasons.PutAll(transform_reasons(&transform, &previous.TransformNode()));
        let effect = layer.GetPropertyTreeState().EffectNode();
        if effect != previous.EffectNode() {
            reasons.PutAll(effect.DirectCompositingReasonsForDebugging());
            if reasons.empty() {
                reasons = transform_reasons(
                    &effect.LocalTransformSpaceUnalias(),
                    &previous.EffectNode().LocalTransformSpaceUnalias(),
                );
                if reasons.empty() {
                    if let (Some(clip), Some(previous_clip)) =
                        (effect.OutputClip(), previous.EffectNode().OutputClip())
                    {
                        reasons = clip_reasons(&clip, &previous_clip);
                    }
                }
            }
        }
        if reasons.empty() {
            reasons = clip_reasons(
                &layer.GetPropertyTreeState().ClipNode(),
                &previous.ClipNode(),
            );
        }
        reasons
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1770-1783.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:264-267.
    pub fn SetScrollbarNeedsDisplay(
        &self,
        element: ElementId,
        c: &mut LayerContext<'_, B>,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) -> bool {
        debug_assert!(self.root_layer_.is_some());
        assert!(services.IsScrollbarElementId(element));
        if let Some(host) = self.root_layer_.as_ref().expect("source root").host_id() {
            if let Some(layer) = c.Host(host).LayerByElementId(element) {
                layer.WithSource(c, |object, context| {
                    let bounds = object.base().bounds(context);
                    let rect = Rect::new(0, 0, bounds.width(), bounds.height());
                    drop(bounds);
                    object.SetNeedsDisplayRect(&rect, context);
                });
                return true;
            }
        }
        false
    }
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1784-1802.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:268-271.
    pub fn SetScrollbarSolidColor(
        &self,
        element: ElementId,
        color: crate::adapter::layer_tree_impl::Color4f,
        c: &mut LayerContext<'_, B>,
        services: &dyn PaintArtifactCompositorServices<B>,
    ) -> bool {
        debug_assert!(self.root_layer_.is_some());
        assert!(services.IsScrollbarElementId(element));
        if let Some(host) = self.root_layer_.as_ref().expect("source root").host_id() {
            if let Some(layer) = c.Host(host).LayerByElementId(element) {
                return layer.WithSource(c, |object, context| {
                    let scrollbar = object.AsScrollbarLayer().expect("source scrollbar static_cast precondition");
                    if scrollbar.GetScrollbarLayerType() == crate::cc::layers::scrollbar_layer_base::ScrollbarLayerType::kSolidColor {
                        scrollbar.AsSolidColorScrollbarLayer().expect("source solid scrollbar static_cast").SetColor(color, context);
                        return true;
                    }
                    false
                });
            }
        }
        false
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1105-1152.
// The native PAC callback receiver is the manager's actual borrowed client.
// Its typed generic projection permits short mutations of the same map.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:297-300.
fn UpdateCompositorViewportProperties<B: PaintArtifactCompositorTypes>(
    properties: &ViewportProperties<B>,
    manager: &mut PropertyTreeManager<'_, '_, B, PropertyTreeManagerClientFacade<'_, B>>,
    host: crate::adapter::main_layer::MainLayerHostId,
) {
    assert_eq!(
        properties.outer_scroll_translation.is_some(),
        properties.inner_scroll_translation.is_some()
    );
    assert!(properties.outer_clip.is_none() || properties.inner_scroll_translation.is_some());
    let mut ids =
        crate::cc::trees::viewport_property_ids::ViewportPropertyIds::default();
    if let Some(node) = &properties.overscroll_elasticity_transform {
        ids.overscroll_elasticity_transform = manager.EnsureCompositorTransformNode(node);
    }
    if let Some(node) = &properties.page_scale {
        ids.page_scale_transform = manager.EnsureCompositorPageScaleTransformNode(node);
    }
    if let Some(inner) = &properties.inner_scroll_translation {
        ids.inner_scroll = manager.EnsureCompositorInnerScrollAndTransformNode(inner);
        if let Some(clip) = &properties.outer_clip {
            ids.outer_clip = manager.EnsureCompositorClipNode(clip);
        }
        let outer = properties
            .outer_scroll_translation
            .as_ref()
            .expect("source outer viewport CHECK");
        ids.outer_scroll = manager.EnsureCompositorOuterScrollAndTransformNode(outer);
        assert!(manager.ClientMutable().NeedsCompositedScrolling(inner));
        assert!(manager.ClientMutable().NeedsCompositedScrolling(outer));
        manager
            .ClientMutable()
            .painted_mut
            .as_deref_mut()
            .expect("source viewport write phase")
            .entry(inner.clone())
            .or_insert(ScrollTranslationInfo {
                is_composited: true,
                ..Default::default()
            });
        manager
            .ClientMutable()
            .painted_mut
            .as_deref_mut()
            .expect("source viewport write phase")
            .entry(outer.clone())
            .or_insert(ScrollTranslationInfo {
                is_composited: true,
                ..Default::default()
            });
    }
    manager
        .LayerContextMutable()
        .HostMut(host)
        .RegisterViewportPropertyIds(ids);
}
impl<B: PaintArtifactCompositorTypes> PaintArtifactCompositor<B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1397-1446.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:180-183.
    pub fn TryFastPathUpdate(
        &mut self,
        artifact: Arc<B::PaintArtifact>,
        context: &mut PaintArtifactCompositorContext<'_, '_, B>,
    ) -> bool {
        #[cfg(debug_assertions)]
        if self.needs_update_ != UpdateType::kFull {
            for index in 0..artifact.PaintChunkCount() {
                debug_assert!(!artifact
                    .PaintChunk(index)
                    .unwrap()
                    .PropertiesUnalias()
                    .ChangedToRoot(B::ChangedOnlySimpleValues()));
            }
        }
        match self.needs_update_ {
            UpdateType::kNone => return true,
            UpdateType::kRasterInducingScroll => {
                for layer in &mut self.pending_layers_ {
                    layer.UpdateForRasterInducingScroll();
                }
            }
            UpdateType::kRepaint => {
                let mut selection =
                    crate::cc::input::layer_selection_bound::LayerSelection::default();
                for layer in &mut self.pending_layers_ {
                    let state = GetPropertyTreeStateForPaint::<B>(&layer.GetPropertyTreeState());
                    layer.UpdateCompositedLayerForRepaint(
                        artifact.clone(),
                        state,
                        &mut selection,
                        context.layer,
                        context.services,
                    );
                }
                let host = self
                    .root_layer_
                    .as_ref()
                    .expect("source root")
                    .host_id()
                    .expect("source attached host");
                context.layer.HostMut(host).RegisterSelection(selection);
                self.UpdateDebugInfo(context.layer, context.services);
            }
            UpdateType::kFull => return false,
        }
        self.previous_update_for_testing_ = self.needs_update_;
        self.needs_update_ = UpdateType::kNone;
        if context.services.VlogIsOn(3) {
            context.services.DebugVlogLayers(
                "PaintArtifactCompositor::TryFastPathUpdate() done",
                self.GetLayersAsJSON(
                    super::content_layer_client_impl::LayerTreeFlags(
                        if context.services.VlogIsOn(3) {
                            u32::MAX
                        } else {
                            0
                        },
                    ),
                    context.layer,
                    context.services,
                ),
            );
        }
        true
    }
}

impl<B: PaintArtifactCompositorTypes> PaintArtifactCompositor<B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1153-1396.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:184-196.
    pub fn Update(
        &mut self,
        artifact: Arc<B::PaintArtifact>,
        viewport: &ViewportProperties<B>,
        scroll_translation_nodes: Vec<B::TransformNode>,
        transition_requests: Vec<Box<B::ViewTransitionRequest>>,
        context: &mut PaintArtifactCompositorContext<'_, '_, B>,
    ) {
        debug_assert!(self.needs_update_ == UpdateType::kFull);
        debug_assert!(self.root_layer_.is_some());
        let _trace = context.services.NewUpdateTrace();
        let Some(host) = self.root_layer_.as_ref().expect("source root").host_id() else {
            return;
        };
        for request in transition_requests {
            context
                .layer
                .HostMut(host)
                .AddViewTransitionRequest(request);
        }
        {
            let (scroll_tree, _, state) = context
                .layer
                .HostMut(host)
                .property_trees_mut()
                .scroll_tree_with_context();
            scroll_tree.SetScrollCallbacks(self.scroll_callbacks_.clone(), state);
        }
        self.root_layer_
            .as_ref()
            .unwrap()
            .set_property_tree_sequence_number(
                *context.property_tree_sequence_number,
                context.layer,
            );
        let old_size = self.pending_layers_.len();
        let mut old_matcher =
            OldPendingLayerMatcher::new(std::mem::take(&mut self.pending_layers_));
        self.canvas_child_layer_map_.clear();
        assert!(self.painted_scroll_translations_.is_empty());
        self.range_dependent_scrolls_.clear();
        self.should_always_update_on_scroll_ = false;
        let (pending, merged_sticky_transforms) =
            Layerizer::new(self, artifact.clone(), old_size, context.services).Layerize();
        self.pending_layers_ = pending;
        PendingLayer::DecompositeTransforms(&mut self.pending_layers_);
        let mut builder = LayerListBuilder::default();
        let root = self.root_layer_.as_ref().unwrap();
        let mut client = PropertyTreeManagerClientFacade {
            cache: &mut self.synthesized_clip_cache_,
            root,
            layer_debug_info_enabled: self.layer_debug_info_enabled_,
            should_always_update_on_scroll: &mut self.should_always_update_on_scroll_,
            needs_update: &self.needs_update_,
            lcd: &self.lcd_text_preference_,
            painted_mut: Some(&mut self.painted_scroll_translations_),
            painted_shared: None,
            services: context.services,
        };
        let scale = context.layer.Host(host).device_scale_factor();
        let viewport_rect = context.layer.Host(host).device_viewport_rect();
        let mut manager = PropertyTreeManager::<B, _>::new(
            &mut client,
            host,
            root,
            &mut builder,
            context.layer,
            *context.property_tree_sequence_number,
            scale,
            viewport_rect,
            context.services.RootEffectElementId(),
        );
        UpdateCompositorViewportProperties(viewport, &mut manager, host);
        for entry in manager.ClientMutable().cache.iter_mut() {
            entry.in_use = false;
        }
        for node in &scroll_translation_nodes {
            manager.EnsureCompositorScrollNode(node);
        }
        let painted = manager.ClientMutable().FinishViewportWrites();
        for node in painted.keys() {
            manager.EnsureCompositorScrollAndTransformNode(node);
        }
        let report_metrics = context.services.ShouldRecordSubsampledMetric(0.01);
        let mut fixed_count = 0;
        let mut merged_fixed_count = 0;
        let mut sticky_count = 0;
        let mut merged_sticky_count = 0;
        let mut selection =
            crate::cc::input::layer_selection_bound::LayerSelection::default();
        let mut layers_having_text = HashSet::new();
        let mut layers_having_video = HashSet::new();
        for (index, pending) in self.pending_layers_.iter_mut().enumerate() {
            let state = pending.GetPropertyTreeState();
            let state_for_paint = GetPropertyTreeStateForPaint::<B>(&state);
            let old = old_matcher.Find(pending);
            pending.UpdateCompositedLayer(
                old,
                state_for_paint,
                &mut selection,
                self.tracks_raster_invalidations_,
                Some(host),
                context.pending,
                manager.LayerContextMutable(),
                context.services,
            );
            let layer = pending.CcLayer();
            let clip = state.ClipNode();
            let effect = state.EffectNode();
            let transform = if effect.CanvasChildId() != B::InvalidDomNodeId() {
                effect.CanvasChildContentTransform()
            } else {
                state.TransformNode()
            };
            let transform_id = manager.EnsureCompositorTransformNode(&transform);
            let draws_content = layer.draws_content(manager.LayerContextMutable());
            let effect_id =
                manager.SwitchToEffectNodeWithSynthesizedClip(&effect, &clip, draws_content);
            let clip_id = manager.EnsureCompositorClipNode(&clip);
            if state.EffectRequiresCompositingForBackdropFilterMask()
                && effect.CcNodeId(*context.property_tree_sequence_number) == effect_id
            {
                assert!(pending.GetContentLayerClient().is_some());
                let picture = layer
                    .clone()
                    .IntoPicture()
                    .ok()
                    .expect("source mask PictureLayer static_cast");
                picture.WithPictureLayer(manager.LayerContextMutable(), |picture, c| {
                    picture.SetIsBackdropFilterMask(true, c)
                });
                layer.SetElementId(
                    effect.GetCompositorElementId(),
                    manager.LayerContextMutable(),
                );
                let effect_tree = manager
                    .LayerContextMutable()
                    .HostMut(host)
                    .property_trees_mut()
                    .effect_tree_mutable();
                let parent = effect_tree.Node(effect_id).parent_id;
                if parent != crate::cc::trees::property_ids::kInvalidPropertyNodeId {
                    let node = effect_tree.MutableNode(parent);
                    if !crate::adapter::property_types::FilterOperations::IsEmpty(
                        &node.backdrop_filters,
                    ) {
                        node.backdrop_mask_element_id = effect.GetCompositorElementId();
                    }
                }
            } else if pending.GetContentLayerClient().is_some()
                && !state.EffectRequiresCompositingForBackdropFilterMask()
            {
                let picture = layer
                    .clone()
                    .IntoPicture()
                    .ok()
                    .expect("source PictureLayer static_cast");
                picture.WithPictureLayer(manager.LayerContextMutable(), |picture, c| {
                    if picture.is_backdrop_filter_mask() {
                        picture.SetIsBackdropFilterMask(false, c);
                    }
                });
            }
            let scroll_id = manager.EnsureCompositorScrollAndTransformNode(
                &Self::ScrollTranslationStateForLayer(pending),
            );
            manager.LayerListBuilderMutable().Add(layer.clone());
            if pending.HasText() {
                layers_having_text.insert(layer.id());
            }
            if pending.HasVideo() {
                layers_having_video.insert(layer.id());
            }
            let sequence = root.property_tree_sequence_number(manager.LayerContextMutable());
            layer.set_property_tree_sequence_number(sequence, manager.LayerContextMutable());
            layer.SetTransformTreeIndex(transform_id, manager.LayerContextMutable());
            layer.SetScrollTreeIndex(scroll_id, manager.LayerContextMutable());
            layer.SetClipTreeIndex(clip_id, manager.LayerContextMutable());
            layer.SetEffectTreeIndex(effect_id, manager.LayerContextMutable());
            layer.SetShouldCheckBackfaceVisibility(
                transform.IsBackfaceHidden(),
                manager.LayerContextMutable(),
            );
            if effect.CanvasChildId() != B::InvalidDomNodeId() {
                self.canvas_child_layer_map_
                    .insert(effect.CanvasChildId(), index);
                layer.SetCanvasChildId(
                    context
                        .services
                        .CompositorElementIdFromDomNodeId(effect.CanvasChildId()),
                    manager.LayerContextMutable(),
                );
            } else {
                assert!(!effect.IsInDrawableCanvasSubtree());
            }
            if layer.subtree_property_changed(manager.LayerContextMutable()) {
                root.SetNeedsCommit(manager.LayerContextMutable());
            }
            if report_metrics {
                if transform.RequiresCompositingForFixedPosition() {
                    fixed_count += 1;
                    merged_fixed_count += pending.MergedAcrossCompositingBoundaryCount();
                }
                if transform.RequiresCompositingForStickyPosition() {
                    sticky_count += 1;
                    merged_sticky_count += pending.MergedAcrossCompositingBoundaryCount();
                }
            }
        }
        if report_metrics {
            context
                .services
                .HistogramCounts100("Blink.Compositor.FixedLayerCount", fixed_count);
            context
                .services
                .HistogramCounts100("Blink.Compositor.MergedFixedLayerCount", merged_fixed_count);
            context
                .services
                .HistogramCounts100("Blink.Compositor.StickyLayerCount", sticky_count);
            context.services.HistogramCounts100(
                "Blink.Compositor.MergedStickyLayerCount",
                merged_sticky_count,
            );
        }
        manager
            .LayerContextMutable()
            .HostMut(host)
            .RegisterSelection(selection);
        manager.Finalize();
        {
            let (client, c) = manager.ClientAndLayerContext();
            // std::remove_if's retained order is unchanged. Native erased unique
            // clients finalize against this same current LayerContext.
            let mut index = 0;
            while index < client.cache.len() {
                if !client.cache[index].in_use {
                    client.cache.remove(index).synthesized_clip.Release(c);
                } else {
                    index += 1;
                }
            }
        }
        manager
            .LayerContextMutable()
            .HostMut(host)
            .property_trees_mut()
            .set_sequence_number(*context.property_tree_sequence_number);
        let layers = manager.LayerListBuilderMutable().Finalize();
        manager.UpdateConditionalRenderSurfaceReasons(
            &layers,
            &layers_having_text,
            &layers_having_video,
        );
        root.SetChildLayerList(layers, manager.LayerContextMutable());
        manager
            .EnsureCompositorNodesForAnchorPositionAdjustmentContainers(&scroll_translation_nodes);
        manager
            .EnsureCompositorNodesForAnchorPositionAdjustmentContainers(&merged_sticky_transforms);
        manager
            .LayerContextMutable()
            .HostMut(host)
            .property_trees_mut()
            .set_needs_rebuild(false);
        manager
            .LayerContextMutable()
            .HostMut(host)
            .property_trees_mut()
            .ResetCachedData();
        drop(manager);
        drop(client);
        self.previous_update_for_testing_ = self.needs_update_;
        self.UpdateDebugInfo(context.layer, context.services);
        self.painted_scroll_translations_.clear();
        self.needs_update_ = UpdateType::kNone;
        *context.property_tree_sequence_number += 1;
        for index in 0..artifact.PaintChunkCount() {
            let chunk = artifact.PaintChunk(index).unwrap();
            chunk
                .PropertiesUnalias()
                .ClearChangedToRoot(*context.property_tree_sequence_number);
            if let Some(translation) = chunk
                .hit_test_data()
                .and_then(|hit| hit.scroll_translation())
            {
                translation.ClearChangedToRoot(*context.property_tree_sequence_number);
            }
        }
        if context.services.VlogIsOn(2) {
            context.services.DebugVlogLayers(
                "PaintArtifactCompositor::Update() done",
                self.GetLayersAsJSON(
                    super::content_layer_client_impl::LayerTreeFlags(
                        if context.services.VlogIsOn(3) {
                            u32::MAX
                        } else {
                            0
                        },
                    ),
                    context.layer,
                    context.services,
                ),
            );
        }
        old_matcher.ReleaseClients(context.layer);
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:331-335.
// UpdateRenderSurfaceForEffects is declared but has no definition or call in
// this pinned Chromium revision. No speculative Rust implementation is added.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:383-392.
// Test friendship, namespace close and WTF allocation decorations are Rust
// module visibility/type allocation rules; they add no runtime control flow.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:53-61.
// The source static sequence starts at 1. Its same mutable scalar is supplied
// through PaintArtifactCompositorContext, incremented only at the source site.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:80-81.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:630-632.
// Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:1838-1839.
// Native anonymous/Blink namespace boundaries are the Rust module boundary.

impl<B: PaintArtifactCompositorTypes> PaintArtifactCompositor<B> {
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.cc:120-120.
    // Source: third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.h:344-382.
    // Native destructor body is empty; members are released in reverse order.
    // Rust exposes current-call finalization because child ClearClient virtual
    // calls require the actual live LayerContext, never a stored host pointer.
    pub fn Destroy(mut self, c: &mut LayerContext<'_, B>) {
        self.range_dependent_scrolls_.clear();
        self.painted_scroll_translations_.clear();
        self.get_canvas_snapshot_callback_ = None;
        self.canvas_child_layer_map_.clear();
        for pending in &mut self.pending_layers_ {
            pending.ReleaseContentLayerClient(c);
        }
        self.pending_layers_.clear();
        for entry in self.synthesized_clip_cache_.drain(..) {
            entry.synthesized_clip.Release(c);
        }
        self.root_layer_.take();
    }
}
impl<B: PaintArtifactCompositorTypes> Drop for PaintArtifactCompositor<B> {
    fn drop(&mut self) {
        assert!(
            self.pending_layers_.is_empty() && self.synthesized_clip_cache_.is_empty(),
            "native child client finalization requires Destroy with current LayerContext"
        );
    }
}
