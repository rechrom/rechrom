// Translation status: complete
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:1-37.
// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:59-62.
// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:38-58.
// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:63-227.
// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:364-422.
// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:423.
// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:1-27.
// Rust modules replace include guards, includes, exports and namespaces.

use super::paint_artifact_compositor::LayerListBuilder;
use crate::adapter::property_tree_manager::{
    PropertyTreeManagerClient, PropertyTreeManagerScrollNode, PropertyTreeManagerTypes,
};
use crate::adapter::{
    geometry::{PointF, RRectF, Rect, RectF, Transform as _},
    input_types::ElementId,
    layer_tree_impl::LayerTreeTypes,
    main_layer::{LayerContext, MainLayerHost, MainLayerHostId, MainLayerHostRead},
    pending_layer::{PendingClipNode, PendingEffectNode, PendingTransformNode},
    property_types::{BlendMode, FilterOperations, MaskFilterInfo},
};
use crate::cc::{
    input::main_thread_scrolling_reason::{MainThreadRepaintReason, MainThreadRepaintReasons},
    layers::layer::LayerRef,
    trees::{
        clip_node::ClipNode,
        damage_reason::DamageReason,
        effect_node::{EffectNode, RenderSurfaceReason},
        property_ids::{kInvalidPropertyNodeId, kRootPropertyNodeId, kSecondaryRootPropertyNodeId},
        scroll_node::ScrollNode,
        transform_node::TransformNode,
    },
};
use std::collections::HashSet;

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:38-40.
pub type StackTransformPaintPropertyNodeVector<B> =
    Vec<<B as crate::adapter::pending_layer::PendingLayerTypes>::TransformNode>;

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:42-57.
// PropertyTreeManagerClient is represented by the exact typed call boundary
// in adapter::property_tree_manager; implementations remain owned by the
// PaintArtifactCompositor which receives these callbacks in Chromium.

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:228-245.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CcEffectType(u8);
impl CcEffectType {
    pub const kEffect: Self = Self(0);
    pub const kSyntheticForNonTrivialClip: Self = Self(1 << 0);
    pub const kSyntheticFor2dAxisAlignment: Self = Self(1 << 1);

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}
impl std::ops::BitOr for CcEffectType {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl std::ops::BitOrAssign for CcEffectType {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:247-318.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Alignment {
    #[default]
    kAligned,
    kMisaligned,
    kUnknown,
}

pub struct EffectState<B: PropertyTreeManagerTypes> {
    pub effect: Option<B::EffectNode>,
    pub clip: Option<B::ClipNode>,
    pub transform: Option<B::TransformNode>,
    pub effect_id: i32,
    pub effect_type: CcEffectType,
    pub may_be_2d_axis_misaligned_to_render_surface: Alignment,
    pub contained_by_non_render_surface_synthetic_rounded_clip: bool,
}
impl<B: PropertyTreeManagerTypes> Clone for EffectState<B> {
    fn clone(&self) -> Self {
        Self {
            effect: self.effect.clone(),
            clip: self.clip.clone(),
            transform: self.transform.clone(),
            effect_id: self.effect_id,
            effect_type: self.effect_type,
            may_be_2d_axis_misaligned_to_render_surface: self
                .may_be_2d_axis_misaligned_to_render_surface,
            contained_by_non_render_surface_synthetic_rounded_clip: self
                .contained_by_non_render_surface_synthetic_rounded_clip,
        }
    }
}
impl<B: PropertyTreeManagerTypes> Default for EffectState<B> {
    fn default() -> Self {
        Self {
            effect: None,
            clip: None,
            transform: None,
            effect_id: 0,
            effect_type: CcEffectType::kEffect,
            may_be_2d_axis_misaligned_to_render_surface: Alignment::kAligned,
            contained_by_non_render_surface_synthetic_rounded_clip: false,
        }
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:320-338.
pub struct CurrentEffectState<B: PropertyTreeManagerTypes> {
    pub effect: Option<B::EffectNode>,
    pub clip: Option<B::ClipNode>,
    pub transform: Option<B::TransformNode>,
    pub effect_id: i32,
    pub effect_type: CcEffectType,
    pub may_be_2d_axis_misaligned_to_render_surface: Alignment,
    pub contained_by_non_render_surface_synthetic_rounded_clip: bool,
}
impl<B: PropertyTreeManagerTypes> Default for CurrentEffectState<B> {
    fn default() -> Self {
        Self {
            effect: None,
            clip: None,
            transform: None,
            effect_id: 0,
            effect_type: CcEffectType::kEffect,
            may_be_2d_axis_misaligned_to_render_surface: Alignment::kAligned,
            contained_by_non_render_surface_synthetic_rounded_clip: false,
        }
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:28-36.
impl<B: PropertyTreeManagerTypes> From<&CurrentEffectState<B>> for EffectState<B> {
    fn from(current: &CurrentEffectState<B>) -> Self {
        Self {
            effect: current.effect.clone(),
            clip: current.clip.clone(),
            transform: current.transform.clone(),
            effect_id: current.effect_id,
            effect_type: CcEffectType::kEffect,
            may_be_2d_axis_misaligned_to_render_surface: current
                .may_be_2d_axis_misaligned_to_render_surface,
            contained_by_non_render_surface_synthetic_rounded_clip: current
                .contained_by_non_render_surface_synthetic_rounded_clip,
        }
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:38-47.
impl<B: PropertyTreeManagerTypes> From<&EffectState<B>> for CurrentEffectState<B> {
    fn from(state: &EffectState<B>) -> Self {
        Self {
            effect: state.effect.clone(),
            clip: state.clip.clone(),
            transform: state.transform.clone(),
            effect_id: state.effect_id,
            effect_type: CcEffectType::kEffect,
            may_be_2d_axis_misaligned_to_render_surface: state
                .may_be_2d_axis_misaligned_to_render_surface,
            contained_by_non_render_surface_synthetic_rounded_clip: state
                .contained_by_non_render_surface_synthetic_rounded_clip,
        }
    }
}

// Keep the client type in this module's public signature exactly as the source
// nested owner contract; no client or PropertyTrees backpointer is global.
pub type ManagerClient<'a, B> = &'a mut dyn PropertyTreeManagerClient<B>;

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:63-71.
// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:340-422.
// Like the source STACK_ALLOCATED owner, this value only borrows the live Paint
// nodes, cc trees, root layer, builder, and call-scoped Layer context. It owns
// no snapshot and leaves no backpointer in any resident object.
pub struct PropertyTreeManager<
    'manager,
    'context,
    B: PropertyTreeManagerTypes,
    C: PropertyTreeManagerClient<B> + ?Sized = dyn PropertyTreeManagerClient<B> + 'manager,
> {
    current_: EffectState<B>,
    client_: &'manager mut C,
    // Source raw property_trees_ points at this host's one resident tree. The
    // identity plus current LayerContext permits short field borrows without
    // aliasing that tree through a second mutable host/context reference.
    host_id_: MainLayerHostId,
    root_layer_: &'manager LayerRef<B>,
    layer_list_builder_: &'manager mut LayerListBuilder<B>,
    layer_context_: &'manager mut LayerContext<'context, B>,
    effect_stack_: Vec<EffectState<B>>,
    pending_synthetic_mask_layers_: HashSet<i32>,
    pixel_moving_filter_clip_expanders_: Vec<B::ClipNode>,
    anchor_position_adjustment_container_ids_: HashSet<ElementId>,
    new_sequence_number_: i32,
}

impl<'manager, 'context, B: PropertyTreeManagerTypes, C: PropertyTreeManagerClient<B> + ?Sized>
    PropertyTreeManager<'manager, 'context, B, C>
{
    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:49-66.
    pub fn new(
        client: &'manager mut C,
        host_id: MainLayerHostId,
        root_layer: &'manager LayerRef<B>,
        layer_list_builder: &'manager mut LayerListBuilder<B>,
        layer_context: &'manager mut LayerContext<'context, B>,
        new_sequence_number: i32,
        device_scale_factor: f32,
        device_viewport_rect: Rect,
        root_effect_element_id: ElementId,
    ) -> Self {
        let mut result = Self {
            current_: EffectState::default(),
            client_: client,
            host_id_: host_id,
            root_layer_: root_layer,
            layer_list_builder_: layer_list_builder,
            layer_context_: layer_context,
            effect_stack_: Vec::new(),
            pending_synthetic_mask_layers_: HashSet::new(),
            pixel_moving_filter_clip_expanders_: Vec::new(),
            anchor_position_adjustment_container_ids_: HashSet::new(),
            new_sequence_number_: new_sequence_number,
        };
        result.SetupRootTransformNode(device_scale_factor);
        result.SetupRootClipNode(device_viewport_rect);
        result.SetupRootEffectNode(root_effect_element_id);
        result.SetupRootScrollNode();
        result
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:120-136.
    // Rust borrow split of the source stack object's already-borrowed fields.
    // No context, builder, tree or owner is copied or allowed to outlive self.
    pub(crate) fn LayerContextMutable(&mut self) -> &mut LayerContext<'context, B> {
        self.layer_context_
    }
    pub(crate) fn ClientAndLayerContext(&mut self) -> (&mut C, &mut LayerContext<'context, B>) {
        (self.client_, self.layer_context_)
    }
    pub(crate) fn ClientMutable(&mut self) -> &mut C {
        self.client_
    }
    pub(crate) fn LayerListBuilderMutable(&mut self) -> &mut LayerListBuilder<B> {
        self.layer_list_builder_
    }

    pub fn DirectlyUpdateCompositedOpacityValue(
        host: &mut dyn MainLayerHost<B>,
        effect: &B::EffectNode,
    ) -> bool {
        let cc_id = effect.CcNodeId(host.property_trees().sequence_number());
        if cc_id == kInvalidPropertyNodeId {
            return false;
        }
        let effect_tree = host.property_trees_mut().effect_tree_mutable();
        let cc_effect = effect_tree.MutableNode(cc_id);
        cc_effect.opacity = effect.Opacity();
        cc_effect.effect_changed = true;
        effect_tree.set_needs_update(true);
        host.SetNeedsCommit();
        true
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:138-177.
    pub fn DirectlyUpdateScrollOffsetTransform(
        host_id: MainLayerHostId,
        transform: &B::TransformNode,
        layer_context: &mut LayerContext<'_, B>,
    ) -> bool {
        let Some(scroll_node) = B::ScrollNodeForTransform(transform) else {
            return false;
        };
        let cc_scroll_node_id = scroll_node.CcNodeId(
            layer_context
                .Host(host_id)
                .property_trees()
                .sequence_number(),
        );
        if cc_scroll_node_id == kInvalidPropertyNodeId
            || layer_context
                .Host(host_id)
                .property_trees()
                .scroll_tree()
                .ShouldRealizeScrollsOnMain(
                    layer_context
                        .Host(host_id)
                        .property_trees()
                        .scroll_tree()
                        .Node(cc_scroll_node_id),
                )
        {
            return false;
        }
        let cc_transform_id = transform.CcNodeId(
            layer_context
                .Host(host_id)
                .property_trees()
                .sequence_number(),
        );
        if cc_transform_id == kInvalidPropertyNodeId {
            return false;
        }
        debug_assert!(
            !layer_context
                .Host(host_id)
                .property_trees()
                .transform_tree()
                .Node(cc_transform_id)
                .is_currently_animating
        );
        let translation = transform.Get2dTranslation();
        let scroll_offset = PointF::new(-translation.x(), -translation.y());
        Self::DirectlySetScrollOffset(
            host_id,
            scroll_node.GetCompositorElementId(),
            &scroll_offset,
            layer_context,
        );
        if layer_context
            .Host(host_id)
            .property_trees()
            .transform_tree()
            .Node(cc_transform_id)
            .scroll_offset()
            != &scroll_offset
        {
            let (transform_tree, mut context) = layer_context
                .HostMut(host_id)
                .property_trees_mut()
                .transform_tree_with_context();
            let cc_transform = transform_tree.MutableNode(cc_transform_id);
            UpdateCcTransformLocalMatrix::<B>(cc_transform, transform);
            cc_transform.SetTransformChanged(DamageReason::kUntracked);
            transform_tree.set_needs_update(true, &mut context);
            layer_context.HostMut(host_id).SetNeedsCommit();
        }
        true
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:253-266.
    pub fn DirectlySetScrollOffset(
        host_id: MainLayerHostId,
        element_id: ElementId,
        scroll_offset: &PointF,
        layer_context: &mut LayerContext<'_, B>,
    ) {
        // Rust borrow split of the same resident host. End each host borrow
        // before the layer callback; preserve source push-properties→commit.
        layer_context
            .Host(host_id)
            .WaitForProtectedSequenceCompletion();
        let changed = {
            let host = layer_context.HostMut(host_id);
            let (scroll_tree, _, state) = host.property_trees_mut().scroll_tree_with_context();
            scroll_tree.SetScrollOffset(element_id, scroll_offset, state)
        };
        if changed {
            let layer = layer_context.Host(host_id).LayerByElementId(element_id);
            if let Some(layer) = layer {
                layer.WithSource(layer_context, |object, context| {
                    object.base().SetNeedsPushProperties(
                        crate::cc::layers::layer::Layer::<B>::kChangedGeneralProperty,
                        context,
                    );
                });
            }
            layer_context.HostMut(host_id).SetNeedsCommit();
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:267-273.
    pub fn DropCompositorScrollDeltaNextCommit(
        host: &mut dyn MainLayerHost<B>,
        element_id: ElementId,
    ) {
        host.DropActiveScrollDeltaNextCommit(element_id);
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:179-206.
    pub fn DirectlyUpdateTransform(
        host: &mut dyn MainLayerHost<B>,
        transform: &B::TransformNode,
    ) -> bool {
        debug_assert!(!transform.HasScrollNode());
        let cc_transform_id = transform.CcNodeId(host.property_trees().sequence_number());
        if cc_transform_id == kInvalidPropertyNodeId {
            return false;
        }
        {
            let (transform_tree, mut context) =
                host.property_trees_mut().transform_tree_with_context();
            let cc_transform = transform_tree.MutableNode(cc_transform_id);
            UpdateCcTransformLocalMatrix::<B>(cc_transform, transform);
            cc_transform.is_currently_animating = false;
            cc_transform.SetTransformChanged(DamageReason::kUntracked);
            transform_tree.set_needs_update(true, &mut context);
        }
        host.SetNeedsCommit();
        true
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:111-118.
    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:208-228.
    pub fn DirectlyUpdatePageScaleTransform(
        host: &mut dyn MainLayerHost<B>,
        transform: &B::TransformNode,
    ) -> bool {
        debug_assert!(!transform.HasScrollNode());
        let cc_transform_id = transform.CcNodeId(host.property_trees().sequence_number());
        if cc_transform_id == kInvalidPropertyNodeId {
            return false;
        }
        let (transform_tree, mut context) = host.property_trees_mut().transform_tree_with_context();
        let page_scale = {
            let cc_transform = transform_tree.MutableNode(cc_transform_id);
            UpdateCcTransformLocalMatrix::<B>(cc_transform, transform);
            debug_assert!(cc_transform.local.IsScale2d());
            let page_scale = cc_transform.local.To2dScale();
            debug_assert_eq!(page_scale.x(), page_scale.y());
            cc_transform.SetTransformChanged(DamageReason::kUntracked);
            page_scale.x()
        };
        transform_tree.set_page_scale_factor(page_scale);
        transform_tree.set_needs_update(true, &mut context);
        true
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:288-299.
    pub fn GetMainThreadRepaintReasons(
        host: &dyn MainLayerHostRead<B>,
        scroll: &B::ScrollNode,
    ) -> MainThreadRepaintReasons {
        let property_trees = host.property_trees();
        let cc_node_id = scroll.CcNodeId(property_trees.sequence_number());
        if cc_node_id != kInvalidPropertyNodeId {
            return property_trees
                .scroll_tree()
                .Node(cc_node_id)
                .main_thread_repaint_reasons;
        }
        let mut reasons = MainThreadRepaintReasons::default();
        reasons.Put(MainThreadRepaintReason::kPreferNonCompositedScrolling);
        reasons
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:301-309.
    pub fn UsesCompositedScrolling(
        host: &dyn MainLayerHostRead<B>,
        scroll: &B::ScrollNode,
    ) -> bool {
        let property_trees = host.property_trees();
        let cc_node_id = scroll.CcNodeId(property_trees.sequence_number());
        cc_node_id != kInvalidPropertyNodeId
            && property_trees.scroll_tree().Node(cc_node_id).is_composited
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:311-319.
    pub fn UsesRasterInducingScroll(
        host: &dyn MainLayerHostRead<B>,
        scroll: &B::ScrollNode,
    ) -> bool {
        let property_trees = host.property_trees();
        let cc_node_id = scroll.CcNodeId(property_trees.sequence_number());
        cc_node_id != kInvalidPropertyNodeId
            && property_trees
                .scroll_tree()
                .CanRealizeScrollsOnPendingTree(property_trees.scroll_tree().Node(cc_node_id))
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:246-251.
    pub fn DirectlyUpdateScrollingContentsCullRect(
        host: &mut dyn MainLayerHost<B>,
        scroll: &B::ScrollNode,
        scrolling_contents_cull_rect_enabled: bool,
    ) {
        UpdateCcScrollingContentsCullRect::<B>(
            host.property_trees_mut().scroll_tree_mutable(),
            scroll,
            scrolling_contents_cull_rect_enabled,
        );
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:274-286.
    fn NonCompositedMainThreadRepaintReasons(
        &self,
        scroll_translation: &B::TransformNode,
    ) -> MainThreadRepaintReasons {
        let scroll = B::ScrollNodeForTransform(scroll_translation)
            .expect("scroll translation must retain its Paint scroll node");
        let mut reasons = MainThreadRepaintReasons::default();
        if scroll.CompositedScrollingNotPreferred() {
            reasons.Put(MainThreadRepaintReason::kPreferNonCompositedScrolling);
            return reasons;
        }
        if self.layer_context_.Services().RasterInducingScrollEnabled()
            && !self
                .client_
                .ShouldForceMainThreadRepaint(scroll_translation)
        {
            return reasons;
        }
        reasons.Put(MainThreadRepaintReason::kNotOpaqueForTextAndLCDText);
        reasons
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:321-345.
    fn SetupRootTransformNode(&mut self, device_scale_factor: f32) {
        let id = {
            let (transform_tree, mut context) = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .transform_tree_with_context();
            transform_tree.clear();
            let id = transform_tree.Insert(
                &TransformNode::<<B as LayerTreeTypes>::Transform>::default(),
                kRootPropertyNodeId,
            );
            debug_assert_eq!(id, kSecondaryRootPropertyNodeId);

            transform_tree.set_device_scale_factor(device_scale_factor);
            let to_screen = B::UniformScaleTransform(device_scale_factor);
            transform_tree.SetToScreen(kRootPropertyNodeId, &to_screen);
            let from_screen = to_screen.GetCheckedInverse();
            transform_tree.SetFromScreen(kRootPropertyNodeId, &from_screen);
            transform_tree.set_needs_update(true, &mut context);
            id
        };
        B::RootTransformNode().SetCcNodeId(self.new_sequence_number_, id);
        self.root_layer_
            .SetTransformTreeIndex(id, self.layer_context_);
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:347-364.
    fn SetupRootClipNode(&mut self, device_viewport_rect: Rect) {
        let id = {
            let clip_tree = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .clip_tree_mutable();
            clip_tree.clear();
            let id = clip_tree.Insert(&ClipNode::default(), kRootPropertyNodeId);
            debug_assert_eq!(id, kSecondaryRootPropertyNodeId);
            let clip_node = clip_tree.MutableNode(id);
            clip_node.clip = RectF::from(device_viewport_rect);
            clip_node.transform_id = kRootPropertyNodeId;
            id
        };
        B::RootClipNode().SetCcNodeId(self.new_sequence_number_, id);
        self.root_layer_.SetClipTreeIndex(id, self.layer_context_);
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:366-386.
    fn SetupRootEffectNode(&mut self, root_effect_element_id: ElementId) {
        let id = {
            let effect_tree = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .effect_tree_mutable();
            effect_tree.clear();
            let id = effect_tree.Insert(
                &EffectNode::<<B as LayerTreeTypes>::Filters, <B as LayerTreeTypes>::Path>::default(
                ),
                kInvalidPropertyNodeId,
            );
            debug_assert_eq!(id, kSecondaryRootPropertyNodeId);
            let effect_node = effect_tree.MutableNode(id);
            effect_node.element_id = root_effect_element_id;
            effect_node.transform_id = kRootPropertyNodeId;
            effect_node.clip_id = kSecondaryRootPropertyNodeId;
            effect_node.render_surface_reason = RenderSurfaceReason::kRoot;
            id
        };
        self.root_layer_.SetEffectTreeIndex(id, self.layer_context_);
        let effect = B::RootEffectNode();
        let clip = B::RootClipNode();
        let transform = B::RootTransformNode();
        effect.SetCcNodeId(self.new_sequence_number_, id);
        self.SetCurrentEffectState(
            id,
            RenderSurfaceReason::kRoot,
            CcEffectType::kEffect,
            effect,
            clip,
            transform,
        );
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:388-398.
    fn SetupRootScrollNode(&mut self) {
        let id = {
            let (scroll_tree, context) = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .scroll_tree_mutation_context();
            scroll_tree.clear(context.state);
            let id = scroll_tree.Insert(&ScrollNode::default(), kRootPropertyNodeId);
            debug_assert_eq!(id, kSecondaryRootPropertyNodeId);
            scroll_tree.MutableNode(id).transform_id = kSecondaryRootPropertyNodeId;
            id
        };
        B::RootScrollNode().SetCcNodeId(self.new_sequence_number_, id);
        self.root_layer_.SetScrollTreeIndex(id, self.layer_context_);
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:470-639.
    pub fn EnsureCompositorTransformNode(&mut self, transform_node: &B::TransformNode) -> i32 {
        let id = transform_node.CcNodeId(self.new_sequence_number_);
        if id != kInvalidPropertyNodeId {
            return id;
        }

        let parent = transform_node
            .UnaliasedParent()
            .expect("non-root Paint transform node must retain its parent");
        let parent_id = self.EnsureCompositorTransformNode(&parent);
        let id = self
            .layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .transform_tree_mutable()
            .Insert(&TransformNode::default(), parent_id);

        if let Some(scroll_parent) = transform_node.ScrollParentScrollTranslation() {
            self.EnsureCompositorTransformNode(&scroll_parent);
        }

        {
            let compositor_node = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .transform_tree_mutable()
                .MutableNode(id);
            UpdateCcTransformLocalMatrix::<B>(compositor_node, transform_node);
            compositor_node.should_undo_overscroll =
                transform_node.RequiresCompositingForFixedToViewport();
            if transform_node.NodeChanged() != B::UnchangedPaintPropertyChange() {
                compositor_node.SetTransformChanged(DamageReason::kUntracked);
            } else {
                compositor_node.ClearTransformChanged();
            }
            compositor_node.flattens_inherited_transform =
                transform_node.FlattensInheritedTransform();
            compositor_node.sorting_context_id = transform_node.RenderingContextId();
            compositor_node.delegates_to_parent_for_backface =
                transform_node.DelegatesToParentForBackface();
        }

        if transform_node.IsAffectedByOuterViewportBoundsDelta() {
            let transform_tree = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .transform_tree_mutable();
            transform_tree
                .MutableNode(id)
                .moved_by_outer_viewport_bounds_delta_y = true;
            transform_tree.AddNodeAffectedByOuterViewportBoundsDelta(id);
        }
        if transform_node.IsAffectedBySafeAreaBottom() {
            let transform_tree = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .transform_tree_mutable();
            transform_tree.MutableNode(id).moved_by_safe_area_bottom = true;
            transform_tree.AddNodeAffectedBySafeAreaInsetBottom(id);
        }

        {
            let compositor_node = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .transform_tree_mutable()
                .MutableNode(id);
            compositor_node.in_subtree_of_page_scale_layer =
                transform_node.IsInSubtreeOfPageScale();
            compositor_node.will_change_transform = transform_node
                .RequiresCompositingForWillChangeTransform()
                && !transform_node.IsForSVGChild();
        }

        if let Some(constraints) = transform_node.GetStickyConstraint() {
            let x_scroll_ancestor = self
                .layer_context_
                .Host(self.host_id_)
                .property_trees()
                .scroll_tree()
                .FindNodeFromElementId(constraints.x_scroll_ancestor_element_id)
                .map_or(kInvalidPropertyNodeId, |node| node.id);
            let y_scroll_ancestor = self
                .layer_context_
                .Host(self.host_id_)
                .property_trees()
                .scroll_tree()
                .FindNodeFromElementId(constraints.y_scroll_ancestor_element_id)
                .map_or(kInvalidPropertyNodeId, |node| node.id);

            let sticky_translation_node_id = |element_id: ElementId, manager: &Self| -> i32 {
                if !element_id.IsValid() {
                    return kInvalidPropertyNodeId;
                }
                manager
                    .layer_context_
                    .Host(manager.host_id_)
                    .property_trees()
                    .transform_tree()
                    .FindNodeFromElementId(element_id)
                    .filter(|node| node.sticky_position_constraint_id != kInvalidPropertyNodeId)
                    .map_or(kInvalidPropertyNodeId, |node| node.id)
            };
            let nearest_node_shifting_sticky_box =
                sticky_translation_node_id(constraints.nearest_element_shifting_sticky_box, self);
            let nearest_node_shifting_containing_block = sticky_translation_node_id(
                constraints.nearest_element_shifting_containing_block,
                self,
            );

            {
                let sticky_data = self
                    .layer_context_
                    .HostMut(self.host_id_)
                    .property_trees_mut()
                    .transform_tree_mutable()
                    .EnsureStickyPositionData(id);
                sticky_data.constraints = constraints;
                sticky_data.x_scroll_ancestor = x_scroll_ancestor;
                sticky_data.y_scroll_ancestor = y_scroll_ancestor;
                sticky_data.nearest_node_shifting_sticky_box = nearest_node_shifting_sticky_box;
                sticky_data.nearest_node_shifting_containing_block =
                    nearest_node_shifting_containing_block;
            }

            let scrolls_outer_viewport = |scroll_ancestor_id: i32, manager: &Self| {
                scroll_ancestor_id != kInvalidPropertyNodeId
                    && manager
                        .layer_context_
                        .Host(manager.host_id_)
                        .property_trees()
                        .scroll_tree()
                        .Node(scroll_ancestor_id)
                        .scrolls_outer_viewport
            };
            if scrolls_outer_viewport(x_scroll_ancestor, self)
                || scrolls_outer_viewport(y_scroll_ancestor, self)
            {
                self.layer_context_
                    .HostMut(self.host_id_)
                    .property_trees_mut()
                    .transform_tree_mutable()
                    .AddNodeAffectedByOuterViewportBoundsDelta(id);
            }
        }

        if let Some(data) = transform_node.GetAnchorPositionScrollData() {
            *self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .transform_tree_mutable()
                .EnsureAnchorPositionScrollData(id) = data.clone();
            self.anchor_position_adjustment_container_ids_
                .extend(data.adjustment_container_ids);
        }

        let compositor_element_id = transform_node.GetCompositorElementId();
        if compositor_element_id.IsValid() {
            let transform_tree = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .transform_tree_mutable();
            transform_tree.SetElementIdForNodeId(id, compositor_element_id);
            transform_tree.MutableNode(id).element_id = compositor_element_id;
        }
        transform_node.SetCcNodeId(self.new_sequence_number_, id);

        if transform_node.HasScrollNode() {
            {
                let compositor_node = self
                    .layer_context_
                    .HostMut(self.host_id_)
                    .property_trees_mut()
                    .transform_tree_mutable()
                    .MutableNode(id);
                compositor_node.scrolls = true;
                compositor_node.should_be_snapped = true;
            }
            let scroll_id = self.EnsureCompositorScrollNode(transform_node);
            let is_composited = self.client_.NeedsCompositedScrolling(transform_node);
            let repaint_reasons = if is_composited {
                None
            } else {
                Some(self.NonCompositedMainThreadRepaintReasons(transform_node))
            };
            let compositor_scroll_node = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .scroll_tree_mutable()
                .MutableNode(scroll_id);
            compositor_scroll_node.transform_id = id;
            compositor_scroll_node.is_composited = is_composited;
            if let Some(reasons) = repaint_reasons {
                compositor_scroll_node
                    .main_thread_repaint_reasons
                    .PutAll(&reasons);
            }
            if self
                .layer_context_
                .Services()
                .ScrollingContentsCullRectOnScrollNodeEnabled()
            {
                let scroll_node = B::ScrollNodeForTransform(transform_node)
                    .expect("scroll translation must retain its Paint scroll node");
                UpdateCcScrollingContentsCullRect::<B>(
                    self.layer_context_
                        .HostMut(self.host_id_)
                        .property_trees_mut()
                        .scroll_tree_mutable(),
                    &scroll_node,
                    true, // The source feature was checked immediately above.
                );
            }
        }

        let parent_frame_id = if parent.IsFramePaintOffsetTranslation() {
            parent_id
        } else {
            self.layer_context_
                .Host(self.host_id_)
                .property_trees()
                .transform_tree()
                .Node(parent_id)
                .parent_frame_id
        };
        self.layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .transform_tree_mutable()
            .MutableNode(id)
            .visible_frame_element_id = transform_node.GetVisibleFrameElementId();
        self.layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .transform_tree_mutable()
            .MutableNode(id)
            .parent_frame_id = parent_frame_id;
        let (transform_tree, mut context) = self
            .layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .transform_tree_with_context();
        transform_tree.set_needs_update(true, &mut context);
        id
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:641-649.
    pub fn EnsureCompositorPageScaleTransformNode(&mut self, node: &B::TransformNode) -> i32 {
        debug_assert!(!node.IsInSubtreeOfPageScale());
        let id = self.EnsureCompositorTransformNode(node);
        let page_scale = self
            .layer_context_
            .Host(self.host_id_)
            .property_trees()
            .transform_tree()
            .Node(id)
            .local
            .To2dScale()
            .x();
        let (transform_tree, mut context) = self
            .layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .transform_tree_with_context();
        transform_tree.set_page_scale_factor(page_scale);
        transform_tree.set_needs_update(true, &mut context);
        id
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:651-676.
    pub fn EnsureCompositorClipNode(&mut self, clip_node: &B::ClipNode) -> i32 {
        let id = clip_node.CcNodeId(self.new_sequence_number_);
        if id != kInvalidPropertyNodeId {
            return id;
        }

        debug_assert!(!clip_node.IsRoot());
        let parent = clip_node.ParentUnalias();
        let parent_id = self.EnsureCompositorClipNode(&parent);
        let id = self
            .layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .clip_tree_mutable()
            .Insert(&ClipNode::default(), parent_id);
        let transform = clip_node.LocalTransformSpaceUnalias();
        let transform_id = self.EnsureCompositorTransformNode(&transform);
        {
            let compositor_node = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .clip_tree_mutable()
                .MutableNode(id);
            compositor_node.clip = clip_node.PaintClipRectRect();
            compositor_node.transform_id = transform_id;
        }
        if clip_node.PixelMovingFilter().is_some() {
            self.pixel_moving_filter_clip_expanders_
                .push(clip_node.clone());
        }
        clip_node.SetCcNodeId(self.new_sequence_number_, id);
        self.layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .clip_tree_mutable()
            .set_needs_update(true);
        id
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:678-687.
    pub fn EnsureCompositorScrollNode(&mut self, scroll_translation: &B::TransformNode) -> i32 {
        let scroll_node = B::ScrollNodeForTransform(scroll_translation)
            .expect("scroll translation must retain its Paint scroll node");
        let scroll_id = self.EnsureCompositorScrollNodeInternal(&scroll_node);
        let translation = scroll_translation.Get2dTranslation();
        let scroll_offset = PointF::new(-translation.x(), -translation.y());
        let (scroll_tree, _, state) = self
            .layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .scroll_tree_with_context();
        scroll_tree.SetScrollOffset(scroll_node.GetCompositorElementId(), &scroll_offset, state);
        scroll_id
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:689-733.
    fn EnsureCompositorScrollNodeInternal(&mut self, scroll_node: &B::ScrollNode) -> i32 {
        let id = scroll_node.CcNodeId(self.new_sequence_number_);
        if id != kInvalidPropertyNodeId {
            return id;
        }

        let parent = scroll_node
            .Parent()
            .expect("non-root Paint scroll node must retain its parent");
        let parent_id = self.EnsureCompositorScrollNodeInternal(&parent);
        let (scroll_tree, _) = self
            .layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .scroll_tree_mutation_context();
        let id = scroll_tree.Insert(&ScrollNode::default(), parent_id);

        let container_rect = scroll_node.ContainerRect();
        let compositor_node = scroll_tree.MutableNode(id);
        compositor_node.container_origin = container_rect.origin();
        compositor_node.container_bounds = container_rect.size();
        compositor_node.bounds = scroll_node.ContentsRect().size();
        compositor_node.user_scrollable_horizontal = scroll_node.UserScrollableHorizontal();
        compositor_node.user_scrollable_vertical = scroll_node.UserScrollableVertical();
        compositor_node.prevent_viewport_scrolling_from_inner =
            scroll_node.PreventViewportScrollingFromInner();
        compositor_node.max_scroll_offset_affected_by_page_scale =
            scroll_node.MaxScrollOffsetAffectedByPageScale();
        compositor_node.overscroll_behavior = scroll_node.OverscrollBehavior();
        compositor_node.snap_container_data = scroll_node.GetSnapContainerData();
        compositor_node.prevent_scroll_axis_locking = scroll_node.PreventScrollAxisLocking();

        let compositor_element_id = scroll_node.GetCompositorElementId();
        if compositor_element_id.IsValid() {
            compositor_node.element_id = compositor_element_id;
            scroll_tree.SetElementIdForNodeId(id, compositor_element_id);
        }

        let compositor_node = scroll_tree.MutableNode(id);
        compositor_node.transform_id = kInvalidPropertyNodeId;
        compositor_node.is_composited = false;
        compositor_node.main_thread_repaint_reasons = scroll_node.GetMainThreadRepaintReasons();

        scroll_node.SetCcNodeId(self.new_sequence_number_, id);
        id
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:735-754.
    pub fn EnsureCompositorScrollAndTransformNode(
        &mut self,
        scroll_translation: &B::TransformNode,
    ) -> i32 {
        self.EnsureCompositorTransformNode(scroll_translation);
        let id = B::ScrollNodeForTransform(scroll_translation)
            .expect("scroll translation must retain its Paint scroll node")
            .CcNodeId(self.new_sequence_number_);
        id
    }

    pub fn EnsureCompositorInnerScrollAndTransformNode(
        &mut self,
        scroll_translation: &B::TransformNode,
    ) -> i32 {
        let node_id = self.EnsureCompositorScrollAndTransformNode(scroll_translation);
        self.layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .scroll_tree_mutable()
            .MutableNode(node_id)
            .scrolls_inner_viewport = true;
        node_id
    }

    pub fn EnsureCompositorOuterScrollAndTransformNode(
        &mut self,
        scroll_translation: &B::TransformNode,
    ) -> i32 {
        let node_id = self.EnsureCompositorScrollAndTransformNode(scroll_translation);
        self.layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .scroll_tree_mutable()
            .MutableNode(node_id)
            .scrolls_outer_viewport = true;
        node_id
    }

    fn IsCurrentCcEffectSynthetic(&self) -> bool {
        self.current_.effect_type != CcEffectType::kEffect
    }

    fn IsCurrentCcEffectSyntheticForNonTrivialClip(&self) -> bool {
        self.current_
            .effect_type
            .contains(CcEffectType::kSyntheticForNonTrivialClip)
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:756-806.
    fn EmitClipMaskLayer(&mut self) {
        let current_effect_id = self.current_.effect_id;
        let needs_layer = {
            let mask_isolation = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .effect_tree_mutable()
                .MutableNode(current_effect_id);
            !self
                .pending_synthetic_mask_layers_
                .contains(&mask_isolation.id)
                && mask_isolation.mask_filter_info.IsEmpty()
        };

        let clip_node = self
            .current_
            .clip
            .clone()
            .expect("current effect must retain its clip");
        let transform = self
            .current_
            .transform
            .clone()
            .expect("current effect must retain its transform");
        let mut mask_isolation_id = ElementId::default();
        let mut mask_effect_id = ElementId::default();
        let synthesized_clip = self.client_.CreateOrReuseSynthesizedClipLayer(
            &clip_node,
            &transform,
            needs_layer,
            &mut mask_isolation_id,
            &mut mask_effect_id,
            self.layer_context_,
        );

        self.layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .effect_tree_mutable()
            .MutableNode(current_effect_id)
            .element_id = mask_isolation_id;
        if !needs_layer {
            return;
        }
        let mask_layer = synthesized_clip
            .LayerRef()
            .expect("new synthesized clip must retain its PictureLayer");

        let mask_effect_tree_id = self
            .layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .effect_tree_mutable()
            .Insert(&EffectNode::default(), current_effect_id);
        let (mask_clip_id, mask_effect_node_id) = {
            let effect_tree = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .effect_tree_mutable();
            let mask_clip_id = effect_tree.MutableNode(current_effect_id).clip_id;
            let mask_effect = effect_tree.MutableNode(mask_effect_tree_id);
            mask_effect.element_id = mask_effect_id;
            mask_effect.clip_id = mask_clip_id;
            mask_effect.blend_mode = BlendMode::kDstIn;
            (mask_clip_id, mask_effect.id)
        };

        self.layer_list_builder_.Add(mask_layer.clone());
        mask_layer.set_property_tree_sequence_number(
            self.root_layer_
                .property_tree_sequence_number(self.layer_context_),
            self.layer_context_,
        );
        let transform_id = self.EnsureCompositorTransformNode(&transform);
        mask_layer.SetTransformTreeIndex(transform_id, self.layer_context_);
        let scroll_translation = transform.PropertyTreeNearestScrollTranslationNode();
        let scroll_id = self.EnsureCompositorScrollAndTransformNode(&scroll_translation);
        mask_layer.SetScrollTreeIndex(scroll_id, self.layer_context_);
        mask_layer.SetClipTreeIndex(mask_clip_id, self.layer_context_);
        mask_layer.SetEffectTreeIndex(mask_effect_node_id, self.layer_context_);

        let has_backdrop_filters = !self
            .layer_context_
            .Host(self.host_id_)
            .property_trees()
            .effect_tree()
            .Node(current_effect_id)
            .backdrop_filters
            .IsEmpty();
        if has_backdrop_filters {
            match mask_layer.clone().IntoPicture() {
                Ok(picture) => picture.WithPictureLayer(self.layer_context_, |layer, context| {
                    layer.SetIsBackdropFilterMask(true, context)
                }),
                Err(_) => panic!("synthesized clip layer must be a PictureLayer"),
            }
            let element_id = B::EffectMaskElementId(mask_effect_id);
            mask_layer.SetElementId(element_id, self.layer_context_);
            self.layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .effect_tree_mutable()
                .MutableNode(current_effect_id)
                .backdrop_mask_element_id = element_id;
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:808-843.
    fn CloseCcEffect(&mut self) {
        debug_assert!(!self.effect_stack_.is_empty());
        let clear_synthetic_effects = !self.IsCurrentCcEffectSynthetic()
            && self
                .current_
                .effect
                .as_ref()
                .expect("current state must retain its Paint effect")
                .MayHaveBackdropEffect();

        if self.IsCurrentCcEffectSyntheticForNonTrivialClip() {
            self.EmitClipMaskLayer();
        }
        if self.IsCurrentCcEffectSynthetic() {
            self.pending_synthetic_mask_layers_
                .remove(&self.current_.effect_id);
        }

        self.current_ = self
            .effect_stack_
            .pop()
            .expect("closing effect requires a previous effect state");
        if clear_synthetic_effects {
            while self.IsCurrentCcEffectSynthetic() {
                self.CloseCcEffect();
            }
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:922-964.
    fn EffectStateMayBe2dAxisMisalignedToRenderSurface(&mut self, state_index: usize) -> bool {
        fn resolve<B: PropertyTreeManagerTypes>(
            states: &mut [EffectState<B>],
            index: usize,
            current_transform: &B::TransformNode,
        ) -> bool {
            if states[index].may_be_2d_axis_misaligned_to_render_surface == Alignment::kUnknown {
                debug_assert_ne!(0, index);
                let (before, state_and_after) = states.split_at_mut(index);
                let previous_misaligned = resolve::<B>(before, index - 1, current_transform);
                let state = &mut state_and_after[0];
                state.may_be_2d_axis_misaligned_to_render_surface = if previous_misaligned
                    || TransformsMayBe2dAxisMisaligned::<B>(
                        before[index - 1]
                            .transform
                            .as_ref()
                            .expect("effect state must retain its transform"),
                        current_transform,
                    ) {
                    Alignment::kMisaligned
                } else {
                    Alignment::kAligned
                };
            }
            states[index].may_be_2d_axis_misaligned_to_render_surface == Alignment::kMisaligned
        }

        let current_transform = self
            .current_
            .transform
            .clone()
            .expect("current effect must retain its transform");
        if state_index < self.effect_stack_.len() {
            resolve::<B>(&mut self.effect_stack_, state_index, &current_transform)
        } else {
            debug_assert_eq!(state_index, self.effect_stack_.len());
            if self.current_.may_be_2d_axis_misaligned_to_render_surface == Alignment::kUnknown {
                debug_assert!(!self.effect_stack_.is_empty());
                let previous_index = self.effect_stack_.len() - 1;
                let previous_misaligned =
                    resolve::<B>(&mut self.effect_stack_, previous_index, &current_transform);
                self.current_.may_be_2d_axis_misaligned_to_render_surface = if previous_misaligned
                    || TransformsMayBe2dAxisMisaligned::<B>(
                        self.effect_stack_[previous_index]
                            .transform
                            .as_ref()
                            .expect("effect state must retain its transform"),
                        &current_transform,
                    ) {
                    Alignment::kMisaligned
                } else {
                    Alignment::kAligned
                };
            }
            self.current_.may_be_2d_axis_misaligned_to_render_surface == Alignment::kMisaligned
        }
    }

    fn CurrentEffectMayBe2dAxisMisalignedToRenderSurface(&mut self) -> bool {
        self.EffectStateMayBe2dAxisMisalignedToRenderSurface(self.effect_stack_.len())
    }

    fn SyntheticEffectType(&mut self, clip: &B::ClipNode) -> CcEffectType {
        let mut effect_type = CcEffectType::kEffect;
        if clip.PaintClipRectIsRounded() || clip.HasClipPath() {
            effect_type |= CcEffectType::kSyntheticForNonTrivialClip;
        }
        let clip_transform = clip.LocalTransformSpaceUnalias();
        let current_transform = self
            .current_
            .transform
            .clone()
            .expect("current effect must retain its transform");
        if self.CurrentEffectMayBe2dAxisMisalignedToRenderSurface()
            || TransformsMayBe2dAxisMisaligned::<B>(&clip_transform, &current_transform)
        {
            effect_type |= CcEffectType::kSyntheticFor2dAxisAlignment;
        }
        effect_type
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:984-1036.
    fn ShaderBasedRRect(
        clip: &B::ClipNode,
        effect_type: CcEffectType,
        transform: &B::TransformNode,
        _next_effect: Option<&B::EffectNode>,
    ) -> Option<RRectF> {
        if effect_type.contains(CcEffectType::kSyntheticFor2dAxisAlignment) || clip.HasClipPath() {
            return None;
        }
        let mut rrect = clip.PaintClipRRect()?;
        if rrect
            .corner_radii
            .iter()
            .any(|radius| radius.x() != radius.y())
        {
            return None;
        }
        #[cfg(target_os = "macos")]
        if rrect.corner_radii[0] != rrect.corner_radii[1]
            || rrect.corner_radii[0] != rrect.corner_radii[2]
            || rrect.corner_radii[0] != rrect.corner_radii[3]
        {
            return None;
        }

        let local_transform = clip.LocalTransformSpaceUnalias();
        if &local_transform != transform {
            let projection = B::SourceToDestinationProjection(&local_transform, transform);
            if !projection.IsIdentityOrTranslation() {
                return None;
            }
            let translation = projection.To2dTranslation();
            rrect.rect = RectF::new(
                rrect.rect.x() + translation.x(),
                rrect.rect.y() + translation.y(),
                rrect.rect.width(),
                rrect.rect.height(),
            );
        }
        Some(rrect)
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:1038-1213.
    fn SynthesizeCcEffectsForClipsIfNeeded(
        &mut self,
        target_clip: &B::ClipNode,
        next_effect: Option<&B::EffectNode>,
    ) -> i32 {
        let mut backdrop_effect_clip_id = kInvalidPropertyNodeId;
        let mut should_realize_backdrop_effect = false;
        if next_effect.is_some_and(|effect| effect.MayHaveBackdropEffect()) {
            while self.IsCurrentCcEffectSynthetic() {
                self.CloseCcEffect();
            }
            self.layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .effect_tree_mutable()
                .MutableNode(self.current_.effect_id)
                .render_surface_reason = RenderSurfaceReason::kBackdropScope;
            should_realize_backdrop_effect = true;
            backdrop_effect_clip_id = self.EnsureCompositorClipNode(target_clip);
        } else {
            let current_clip = self
                .current_
                .clip
                .clone()
                .expect("current effect must retain its clip");
            let lca = current_clip.LowestCommonAncestor(target_clip);
            while self.current_.clip.as_ref() != Some(&lca) {
                if !self.IsCurrentCcEffectSynthetic() {
                    return kInvalidPropertyNodeId;
                }
                let pre_exit_clip = self
                    .current_
                    .clip
                    .clone()
                    .expect("synthetic effect must retain its clip");
                self.CloseCcEffect();
                let enclosing_clip = self
                    .current_
                    .clip
                    .as_ref()
                    .expect("enclosing effect must retain its clip");
                if IsNodeOnAncestorChain::<B>(&lca, &pre_exit_clip, enclosing_clip) {
                    break;
                }
            }
        }

        // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:974-982.
        let current_clip = self
            .current_
            .clip
            .clone()
            .expect("current effect must retain its clip");
        let mut pending_clips = Vec::new();
        let mut clip_node = target_clip.clone();
        while clip_node != current_clip {
            let effect_type = self.SyntheticEffectType(&clip_node);
            if effect_type != CcEffectType::kEffect {
                pending_clips.push((clip_node.clone(), effect_type));
            }
            if clip_node.IsRoot() {
                return kInvalidPropertyNodeId;
            }
            clip_node = clip_node.ParentUnalias();
        }
        if pending_clips.is_empty() {
            return kInvalidPropertyNodeId;
        }

        let mut cc_effect_id_for_backdrop_effect = kInvalidPropertyNodeId;
        for (pending_clip, mut effect_type) in pending_clips.into_iter().rev() {
            let mut clip_id = backdrop_effect_clip_id;
            let synthetic_effect_id = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .effect_tree_mutable()
                .Insert(&EffectNode::default(), self.current_.effect_id);
            let clip_transform = pending_clip.LocalTransformSpaceUnalias();
            let transform = if should_realize_backdrop_effect {
                next_effect
                    .expect("backdrop realization requires its Paint effect")
                    .LocalTransformSpaceUnalias()
            } else {
                clip_transform.clone()
            };

            if effect_type.contains(CcEffectType::kSyntheticFor2dAxisAlignment) {
                if should_realize_backdrop_effect {
                    effect_type |= CcEffectType::kSyntheticForNonTrivialClip;
                } else {
                    let parent_clip = pending_clip.ParentUnalias();
                    clip_id = self.EnsureCompositorClipNode(&parent_clip);
                    let synthetic_effect = self
                        .layer_context_
                        .HostMut(self.host_id_)
                        .property_trees_mut()
                        .effect_tree_mutable()
                        .MutableNode(synthetic_effect_id);
                    synthetic_effect.element_id = B::NewUniqueElementId();
                    synthetic_effect.render_surface_reason =
                        RenderSurfaceReason::kClipAxisAlignment;
                }
            }

            if effect_type.contains(CcEffectType::kSyntheticForNonTrivialClip) {
                if clip_id == kInvalidPropertyNodeId {
                    let mut output_clip = pending_clip.clone();
                    let current_effect = self
                        .current_
                        .effect
                        .as_ref()
                        .expect("current state must retain its effect");
                    if !current_effect.ViewTransitionElementResourceId().IsValid() {
                        output_clip = output_clip.ParentUnalias();
                    }
                    clip_id = self.EnsureCompositorClipNode(&output_clip);
                }

                if let Some(rrect) =
                    Self::ShaderBasedRRect(&pending_clip, effect_type, &clip_transform, next_effect)
                {
                    let mask_clip_id = self.EnsureCompositorClipNode(&pending_clip);
                    let synthetic_effect = self
                        .layer_context_
                        .HostMut(self.host_id_)
                        .property_trees_mut()
                        .effect_tree_mutable()
                        .MutableNode(synthetic_effect_id);
                    synthetic_effect.mask_filter_info = MaskFilterInfo {
                        rounded_corner_bounds: rrect,
                        clip_id: Some(mask_clip_id),
                        ..Default::default()
                    };
                    synthetic_effect.is_fast_rounded_corner = true;

                    if self
                        .current_
                        .contained_by_non_render_surface_synthetic_rounded_clip
                    {
                        self.ForceRenderSurfaceIfSyntheticRoundedCornerClip(
                            self.current_.effect_id,
                            self.current_.effect_type,
                        );
                        let enclosing_states: Vec<_> = self
                            .effect_stack_
                            .iter()
                            .rev()
                            .map(|state| (state.effect_id, state.effect_type))
                            .collect();
                        for (effect_id, effect_type) in enclosing_states {
                            let node = self
                                .layer_context_
                                .Host(self.host_id_)
                                .property_trees()
                                .effect_tree()
                                .Node(effect_id);
                            if node.HasRenderSurface()
                                && !IsConditionalRenderSurfaceReason(node.render_surface_reason)
                            {
                                break;
                            }
                            self.ForceRenderSurfaceIfSyntheticRoundedCornerClip(
                                effect_id,
                                effect_type,
                            );
                        }
                    }
                } else {
                    self.layer_context_
                        .HostMut(self.host_id_)
                        .property_trees_mut()
                        .effect_tree_mutable()
                        .MutableNode(synthetic_effect_id)
                        .render_surface_reason = if pending_clip.PaintClipRectIsRounded() {
                        RenderSurfaceReason::kRoundedCorner
                    } else {
                        RenderSurfaceReason::kClipPath
                    };
                }
                self.pending_synthetic_mask_layers_
                    .insert(synthetic_effect_id);
            }

            if should_realize_backdrop_effect {
                debug_assert_eq!(cc_effect_id_for_backdrop_effect, kInvalidPropertyNodeId);
                self.PopulateCcEffectNode(
                    synthetic_effect_id,
                    next_effect.expect("backdrop realization requires its Paint effect"),
                    clip_id,
                );
                cc_effect_id_for_backdrop_effect = synthetic_effect_id;
                should_realize_backdrop_effect = false;
            } else {
                self.layer_context_
                    .HostMut(self.host_id_)
                    .property_trees_mut()
                    .effect_tree_mutable()
                    .MutableNode(synthetic_effect_id)
                    .clip_id = clip_id;
            }

            let transform_id = self.EnsureCompositorTransformNode(&transform);
            {
                let synthetic_effect = self
                    .layer_context_
                    .HostMut(self.host_id_)
                    .property_trees_mut()
                    .effect_tree_mutable()
                    .MutableNode(synthetic_effect_id);
                synthetic_effect.transform_id = transform_id;
                synthetic_effect.double_sided = !transform.IsBackfaceHidden();
            }
            self.effect_stack_.push(self.current_.clone());
            let effect = self
                .current_
                .effect
                .clone()
                .expect("current state must retain its Paint effect");
            self.SetCurrentEffectState(
                synthetic_effect_id,
                self.layer_context_
                    .Host(self.host_id_)
                    .property_trees()
                    .effect_tree()
                    .Node(synthetic_effect_id)
                    .render_surface_reason,
                effect_type,
                effect,
                pending_clip,
                transform,
            );
        }
        cc_effect_id_for_backdrop_effect
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:1215-1310.
    fn BuildEffectNodesRecursively(&mut self, next_effect: &B::EffectNode) {
        if self.current_.effect.as_ref() == Some(next_effect) {
            return;
        }
        let parent = next_effect.ParentUnalias();
        self.BuildEffectNodesRecursively(&parent);
        debug_assert!(self.current_.effect.as_ref() == Some(&parent));

        let has_multiple_groups =
            next_effect.CcNodeId(self.new_sequence_number_) != kInvalidPropertyNodeId;
        let (mut real_effect_node_id, output_clip, output_clip_id) = if let Some(output_clip) =
            next_effect.OutputClip()
        {
            let real_id = self.SynthesizeCcEffectsForClipsIfNeeded(&output_clip, Some(next_effect));
            let clip_id = self.EnsureCompositorClipNode(&output_clip);
            (real_id, output_clip, clip_id)
        } else {
            while self.IsCurrentCcEffectSynthetic() {
                self.CloseCcEffect();
            }
            let output_clip = self
                .current_
                .clip
                .clone()
                .expect("non-synthetic effect must retain its output clip");
            let clip_id = self
                .layer_context_
                .Host(self.host_id_)
                .property_trees()
                .effect_tree()
                .Node(self.current_.effect_id)
                .clip_id;
            debug_assert_eq!(clip_id, self.EnsureCompositorClipNode(&output_clip));
            (kInvalidPropertyNodeId, output_clip, clip_id)
        };

        let transform = next_effect.LocalTransformSpaceUnalias();
        let effect_node_id = self
            .layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .effect_tree_mutable()
            .Insert(&EffectNode::default(), self.current_.effect_id);
        if real_effect_node_id == kInvalidPropertyNodeId {
            real_effect_node_id = effect_node_id;
            debug_assert!(
                (!next_effect.ViewTransitionElementResourceId().IsValid()
                    && !next_effect.SubtreeCaptureId().is_valid())
                    || !has_multiple_groups
            );
            self.PopulateCcEffectNode(effect_node_id, next_effect, output_clip_id);
        } else {
            let transform_id = self.EnsureCompositorTransformNode(&transform);
            let effect_node = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .effect_tree_mutable()
                .MutableNode(effect_node_id);
            effect_node.clip_id = output_clip_id;
            effect_node.transform_id = transform_id;
            effect_node.element_id = next_effect.GetCompositorElementId();
        }

        if has_multiple_groups {
            let effect_node = self
                .layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .effect_tree_mutable()
                .MutableNode(effect_node_id);
            if effect_node.element_id.IsValid() {
                effect_node.element_id = B::NewUniqueElementId();
            }
        } else {
            next_effect.SetCcNodeId(self.new_sequence_number_, real_effect_node_id);
        }

        let compositor_element_id = next_effect.GetCompositorElementId();
        if compositor_element_id.IsValid() && !has_multiple_groups {
            debug_assert!(
                self.layer_context_
                    .Host(self.host_id_)
                    .property_trees()
                    .effect_tree()
                    .FindNodeFromElementId(compositor_element_id)
                    .is_none()
            );
            self.layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .effect_tree_mutable()
                .SetElementIdForNodeId(real_effect_node_id, compositor_element_id);
        }

        let render_surface_reason = self
            .layer_context_
            .Host(self.host_id_)
            .property_trees()
            .effect_tree()
            .Node(effect_node_id)
            .render_surface_reason;
        self.effect_stack_.push(self.current_.clone());
        self.SetCurrentEffectState(
            effect_node_id,
            render_surface_reason,
            CcEffectType::kEffect,
            next_effect.clone(),
            output_clip,
            transform,
        );
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:845-903.
    pub fn SwitchToEffectNodeWithSynthesizedClip(
        &mut self,
        next_effect: &B::EffectNode,
        next_clip: &B::ClipNode,
        layer_draws_content: bool,
    ) -> i32 {
        let current_effect = self
            .current_
            .effect
            .clone()
            .expect("current state must retain its Paint effect");
        let ancestor = current_effect.LowestCommonAncestor(next_effect);
        while self.current_.effect.as_ref() != Some(&ancestor) {
            self.CloseCcEffect();
        }
        self.BuildEffectNodesRecursively(next_effect);
        self.SynthesizeCcEffectsForClipsIfNeeded(next_clip, None);
        if layer_draws_content {
            self.pending_synthetic_mask_layers_.clear();
        }
        self.current_.effect_id
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.h:145-149.
    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:73-80.
    pub fn Finalize(&mut self) {
        while !self.effect_stack_.is_empty() {
            self.CloseCcEffect();
        }
        self.UpdatePixelMovingFilterClipExpanders();
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:429-468.
    fn SetCurrentEffectState(
        &mut self,
        effect_id: i32,
        render_surface_reason: RenderSurfaceReason,
        effect_type: CcEffectType,
        effect: B::EffectNode,
        clip: B::ClipNode,
        transform: B::TransformNode,
    ) {
        let previous_transform = if effect.IsRoot() {
            None
        } else {
            self.current_.transform.clone()
        };
        self.current_.effect_id = effect_id;
        self.current_.effect_type = effect_type;
        self.current_.effect = Some(effect);
        self.current_.clip = Some(clip);
        self.current_.transform = Some(transform.clone());

        if render_surface_reason != RenderSurfaceReason::kNone
            && !IsConditionalRenderSurfaceReason(render_surface_reason)
        {
            self.current_.may_be_2d_axis_misaligned_to_render_surface = Alignment::kAligned;
            self.current_
                .contained_by_non_render_surface_synthetic_rounded_clip = false;
        } else {
            if self.current_.may_be_2d_axis_misaligned_to_render_surface == Alignment::kAligned
                && previous_transform.as_ref() != Some(&transform)
            {
                self.current_.may_be_2d_axis_misaligned_to_render_surface = Alignment::kUnknown;
            }
            self.current_
                .contained_by_non_render_surface_synthetic_rounded_clip |=
                effect_type.contains(CcEffectType::kSyntheticForNonTrivialClip);
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:966-972.
    fn ForceRenderSurfaceIfSyntheticRoundedCornerClip(
        &mut self,
        effect_id: i32,
        effect_type: CcEffectType,
    ) {
        if effect_type.contains(CcEffectType::kSyntheticForNonTrivialClip) {
            self.layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .effect_tree_mutable()
                .MutableNode(effect_id)
                .render_surface_reason = RenderSurfaceReason::kRoundedCorner;
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:1403-1471.
    pub fn UpdateConditionalRenderSurfaceReasons(
        &mut self,
        layers: &[LayerRef<B>],
        layers_having_text: &HashSet<i32>,
        layers_having_video: &HashSet<i32>,
    ) {
        let tree_size = self
            .layer_context_
            .Host(self.host_id_)
            .property_trees()
            .effect_tree()
            .size();
        let mut effect_layer_counts = vec![0_i32; tree_size];
        let mut has_text = vec![false; tree_size];
        let mut has_video = vec![false; tree_size];
        let mut has_child_surface = vec![false; tree_size];

        for layer in layers {
            if layer.draws_content(self.layer_context_) {
                let effect_id = layer.effect_tree_index(self.layer_context_) as usize;
                effect_layer_counts[effect_id] += 1;
                has_text[effect_id] |= layers_having_text.contains(&layer.id());
                has_video[effect_id] |= layers_having_video.contains(&layer.id());
            }
        }

        let effect_tree = self
            .layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .effect_tree_mutable();
        for id in ((kSecondaryRootPropertyNodeId + 1) as usize..tree_size).rev() {
            let effect = effect_tree.MutableNode(id as i32);
            if effect.render_surface_reason == RenderSurfaceReason::kNone
                && effect.needs_effect_for_2d_scale_transform
                && effect_layer_counts[id] >= 2
                && !has_text[id]
                && !has_video[id]
            {
                effect.render_surface_reason =
                    RenderSurfaceReason::k2DScaleTransformWithCompositedDescendants;
            }

            if effect_layer_counts[id] < 2
                && IsConditionalRenderSurfaceReason(effect.render_surface_reason)
                && (effect.render_surface_reason != RenderSurfaceReason::kBlendModeDstIn
                    || !(has_child_surface[id] || effect.is_fast_rounded_corner))
            {
                effect.render_surface_reason = RenderSurfaceReason::kNone;
            }

            let parent_id = effect.parent_id as usize;
            debug_assert_ne!(-1, effect_layer_counts[parent_id]);
            if effect.HasRenderSurface() {
                effect_layer_counts[parent_id] += 1;
                has_child_surface[parent_id] = true;
            } else {
                effect_layer_counts[parent_id] += effect_layer_counts[id];
                has_child_surface[parent_id] |= has_child_surface[id];
            }
            has_text[parent_id] |= has_text[id];

            #[cfg(debug_assertions)]
            {
                effect_layer_counts[id] = -1;
            }
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:1490-1508.
    pub fn EnsureCompositorNodesForAnchorPositionAdjustmentContainers(
        &mut self,
        transforms: &StackTransformPaintPropertyNodeVector<B>,
    ) {
        if self.anchor_position_adjustment_container_ids_.is_empty() {
            return;
        }
        for transform in transforms {
            if let Some(scroll) = B::ScrollNodeForTransform(transform) {
                if self
                    .anchor_position_adjustment_container_ids_
                    .contains(&scroll.GetCompositorElementId())
                {
                    self.EnsureCompositorScrollAndTransformNode(transform);
                }
            } else if self
                .anchor_position_adjustment_container_ids_
                .contains(&transform.GetCompositorElementId())
            {
                self.EnsureCompositorTransformNode(transform);
            }
        }
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:1367-1401.
    fn PopulateCcEffectNode(
        &mut self,
        effect_node_id: i32,
        effect: &B::EffectNode,
        output_clip_id: i32,
    ) {
        let transform = effect.LocalTransformSpaceUnalias();
        let transform_id = self.EnsureCompositorTransformNode(&transform);
        let may_have_backdrop_effect = effect.MayHaveBackdropEffect();
        let filter = effect.FilterOperations();
        let backdrop_filter = effect.BackdropFilterOperations();
        if may_have_backdrop_effect {
            debug_assert!(filter.is_none());
        }
        let effect_node = self
            .layer_context_
            .HostMut(self.host_id_)
            .property_trees_mut()
            .effect_tree_mutable()
            .MutableNode(effect_node_id);
        effect_node.element_id = effect.GetCompositorElementId();
        effect_node.clip_id = output_clip_id;
        effect_node.render_surface_reason = RenderSurfaceReasonForEffect::<B>(effect);
        effect_node.opacity = effect.Opacity();
        effect_node.transform_id = transform_id;
        effect_node.needs_effect_for_2d_scale_transform = effect.NeedsEffectFor2DScaleTransform();
        if may_have_backdrop_effect {
            effect_node.may_have_backdrop_effect = true;
            if let Some(backdrop_filter) = backdrop_filter {
                effect_node.backdrop_filters = backdrop_filter;
                effect_node.backdrop_filter_bounds = effect.PropertyTreeBackdropFilterBounds();
                effect_node.backdrop_mask_element_id = effect.BackdropMaskElementId();
            }
            effect_node.blend_mode = effect.BlendMode();
        } else if let Some(filter) = filter {
            effect_node.filters = filter;
        }
        effect_node.double_sided = !transform.IsBackfaceHidden();
        effect_node.effect_changed = effect.NodeChanged() != B::UnchangedPaintPropertyChange();
        effect_node.view_transition_element_resource_id = effect.ViewTransitionElementResourceId();
        effect_node.subtree_capture_id = effect.SubtreeCaptureId();
    }

    // Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:1473-1489.
    fn UpdatePixelMovingFilterClipExpanders(&mut self) {
        for clip in &self.pixel_moving_filter_clip_expanders_ {
            let pixel_moving_filter = clip
                .PixelMovingFilter()
                .expect("collected clip must retain its pixel-moving filter");
            let clip_id = clip.CcNodeId(self.new_sequence_number_);
            if clip_id == kInvalidPropertyNodeId {
                continue;
            }
            self.layer_context_
                .HostMut(self.host_id_)
                .property_trees_mut()
                .clip_tree_mutable()
                .MutableNode(clip_id)
                .pixel_moving_filter_id = pixel_moving_filter.CcNodeId(self.new_sequence_number_);
        }
        self.pixel_moving_filter_clip_expanders_.clear();
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:82-109.
fn UpdateCcTransformLocalMatrix<B: PropertyTreeManagerTypes>(
    compositor_node: &mut TransformNode<<B as LayerTreeTypes>::Transform>,
    transform_node: &B::TransformNode,
) {
    if transform_node.HasStickyConstraint() || transform_node.HasAnchorPositionScrollData() {
        debug_assert!(compositor_node.local == <B as LayerTreeTypes>::Transform::default());
        debug_assert!(compositor_node.origin == Default::default());
    } else if transform_node.HasScrollNode() {
        debug_assert!(transform_node.IsIdentityOr2dTranslation());
        let translation = transform_node.Get2dTranslation();
        compositor_node.SetScrollOffset(
            &PointF::new(-translation.x(), -translation.y()),
            DamageReason::kUntracked,
        );
        debug_assert!(compositor_node.local == <B as LayerTreeTypes>::Transform::default());
        debug_assert!(compositor_node.origin == Default::default());
    } else {
        compositor_node.local = transform_node.Matrix();
        compositor_node.origin = transform_node.Origin();
    }
    compositor_node.needs_local_transform_update = true;
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:230-244.
fn UpdateCcScrollingContentsCullRect<B: PropertyTreeManagerTypes>(
    scroll_tree: &mut crate::cc::trees::property_tree::ScrollTree,
    scroll_node: &B::ScrollNode,
    enabled: bool,
) {
    assert!(enabled);
    let cull_rect = scroll_node.ScrollingContentsCullRect();
    if cull_rect.Contains(scroll_node.ContentsRect()) {
        scroll_tree.ClearScrollingContentsCullRect(scroll_node.GetCompositorElementId());
    } else {
        scroll_tree.SetScrollingContentsCullRect(scroll_node.GetCompositorElementId(), &cull_rect);
    }
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:900-920.
fn IsNodeOnAncestorChain<B: PropertyTreeManagerTypes>(
    find: &B::ClipNode,
    current: &B::ClipNode,
    ancestor: &B::ClipNode,
) -> bool {
    debug_assert!(ancestor.IsAncestorOf(current));
    let mut node = current.clone();
    while &node != ancestor {
        if &node == find {
            return true;
        }
        node = node.ParentUnalias();
    }
    false
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:400-412.
fn TransformsToAncestorHaveNonAxisAlignedActiveAnimation<B: PropertyTreeManagerTypes>(
    descendant: &B::TransformNode,
    ancestor: &B::TransformNode,
) -> bool {
    if descendant == ancestor {
        return false;
    }
    let mut node = descendant.clone();
    while &node != ancestor {
        if node.HasActiveTransformAnimation() && !node.TransformAnimationIsAxisAligned() {
            return true;
        }
        node = node
            .UnaliasedParent()
            .expect("ancestor must be on the unaliased transform chain");
    }
    false
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:414-427.
fn TransformsMayBe2dAxisMisaligned<B: PropertyTreeManagerTypes>(
    a: &B::TransformNode,
    b: &B::TransformNode,
) -> bool {
    if a == b {
        return false;
    }
    if !B::SourceToDestinationProjection(a, b).Preserves2dAxisAlignment() {
        return true;
    }
    let lca = a.PropertyTreeLowestCommonAncestor(b);
    TransformsToAncestorHaveNonAxisAlignedActiveAnimation::<B>(a, &lca)
        || TransformsToAncestorHaveNonAxisAlignedActiveAnimation::<B>(b, &lca)
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:429-437.
fn IsConditionalRenderSurfaceReason(reason: RenderSurfaceReason) -> bool {
    matches!(
        reason,
        RenderSurfaceReason::kBlendModeDstIn
            | RenderSurfaceReason::kOpacity
            | RenderSurfaceReason::kOpacityAnimation
    )
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:1312-1324.
fn ConditionalRenderSurfaceReasonForEffect<B: PropertyTreeManagerTypes>(
    effect: &B::EffectNode,
) -> RenderSurfaceReason {
    if effect.BlendMode() == BlendMode::kDstIn {
        return RenderSurfaceReason::kBlendModeDstIn;
    }
    if effect.Opacity() != 1.0 {
        return RenderSurfaceReason::kOpacity;
    }
    if effect.HasActiveOpacityAnimation() {
        return RenderSurfaceReason::kOpacityAnimation;
    }
    RenderSurfaceReason::kNone
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:1326-1365.
fn RenderSurfaceReasonForEffect<B: PropertyTreeManagerTypes>(
    effect: &B::EffectNode,
) -> RenderSurfaceReason {
    if effect.HasFilter() || effect.RequiresCompositingForWillChangeFilter() {
        return RenderSurfaceReason::kFilter;
    }
    if effect.HasActiveFilterAnimation() {
        return RenderSurfaceReason::kFilterAnimation;
    }
    if effect.HasBackdropFilter() || effect.RequiresCompositingForWillChangeBackdropFilter() {
        return RenderSurfaceReason::kBackdropFilter;
    }
    if effect.HasActiveBackdropFilterAnimation() {
        return RenderSurfaceReason::kBackdropFilterAnimation;
    }
    if !matches!(effect.BlendMode(), BlendMode::kSrcOver | BlendMode::kDstIn) {
        return RenderSurfaceReason::kBlendMode;
    }
    if effect.HasValidViewTransitionElementResourceId() {
        return RenderSurfaceReason::kViewTransitionParticipant;
    }
    if effect.RequiresCompositingForUnboundedElement() {
        return RenderSurfaceReason::kUnboundedElement;
    }
    if effect.FlattensAtLeafOf3DScene() {
        return RenderSurfaceReason::k3dTransformFlattening;
    }
    if effect.HasElementCaptureId() {
        return RenderSurfaceReason::kSubtreeIsBeingCaptured;
    }
    let conditional_reason = ConditionalRenderSurfaceReasonForEffect::<B>(effect);
    debug_assert!(
        conditional_reason == RenderSurfaceReason::kNone
            || IsConditionalRenderSurfaceReason(conditional_reason)
    );
    conditional_reason
}

// Source: third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.cc:68-71.
impl<B: PropertyTreeManagerTypes, C: PropertyTreeManagerClient<B> + ?Sized> Drop
    for PropertyTreeManager<'_, '_, B, C>
{
    fn drop(&mut self) {
        debug_assert!(
            self.effect_stack_.is_empty(),
            "PropertyTreeManager::Finalize() must be called at the end of tree conversion"
        );
    }
}
