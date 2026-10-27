// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/draw_property_utils.h:1-124; cc/trees/draw_property_utils.cc:1-51.
use super::{
    clip_node::{ClipNode, ConditionalClip},
    effect_node::{EffectNode, RenderSurfaceReason, RenderSurfaceReasonToString},
    property_ids::*,
    property_tree::{ClipTree, EffectTree, EffectTreeContext, TransformTree, TransformTreeContext},
    transform_node::TransformNode,
    viewport_property_ids::ViewportPropertyIds,
};
use crate::adapter::{
    draw_property::*,
    geometry::{Rect, RectF, ToEnclosingRectIgnoringError, Transform, Vector2dF},
    property_tree::{PropertyTreeFilters, PropertyTreeTransform},
    property_types::{MaskFilterInfo, ViewTransitionElementResourceId},
};
use crate::cc::base::math_util::MathUtil;
use std::collections::BTreeMap;
// Source: cc/trees/draw_property_utils.cc:52-55.
fn ToEnclosingClipRect(rect: RectF) -> Rect {
    ToEnclosingRectIgnoringError(rect, 0.00001)
}
// Source: cc/trees/draw_property_utils.cc:57-63.
fn IsRootLayer<C: DrawContext>(context: &C, layer: C::Layer, _main_thread: bool) -> bool {
    // Native overload is selected by the actual context's Layer/LayerImpl
    // family. No Impl owner is required to manufacture a parent pointer.
    context.IsRootLayer(layer)
}

// Source: cc/trees/draw_property_utils.cc:65-70.
fn PostConcatSurfaceContentsScale<C: DrawContext>(
    node: &EffectNode<C::Filters, C::Path>,
    transform: &mut C::Transform,
) {
    debug_assert!(node.HasRenderSurface());
    transform.PostScale(
        node.surface_contents_scale.x(),
        node.surface_contents_scale.y(),
    );
}
// Source: cc/trees/draw_property_utils.cc:72-104.
fn ConvertRectBetweenSurfaceSpaces<C: DrawContext>(
    p: &DrawPropertyTrees<C>,
    source_effect_id: i32,
    dest_effect_id: i32,
    clip_in_source_space: RectF,
    clip_in_dest_space: &mut RectF,
) -> bool {
    let source = p.effect_tree().Node(source_effect_id);
    let dest = p.effect_tree().Node(dest_effect_id);
    let mut transform = C::Transform::default();
    if source.transform_id > dest.transform_id {
        if p.GetToTarget(source.transform_id, dest_effect_id, &mut transform) {
            ConcatInverseSurfaceContentsScale::<C>(source, &mut transform);
            *clip_in_dest_space = MathUtil::MapClippedRect(&transform, clip_in_source_space);
        } else {
            return false;
        }
    } else if p.GetFromTarget(dest.transform_id, source_effect_id, &mut transform) {
        PostConcatSurfaceContentsScale::<C>(dest, &mut transform);
        *clip_in_dest_space = MathUtil::ProjectClippedRect(&transform, clip_in_source_space);
    } else {
        return false;
    }
    true
}
// Source: cc/trees/draw_property_utils.cc:106-125.
fn ComputeTargetRectInLocalSpace<C: DrawContext>(
    rect: RectF,
    p: &DrawPropertyTrees<C>,
    target_transform_id: i32,
    local_transform_id: i32,
    target_effect_id: i32,
) -> ConditionalClip {
    let mut transform = C::Transform::default();
    if !p.GetFromTarget(local_transform_id, target_effect_id, &mut transform) {
        return ConditionalClip::default();
    }
    ConditionalClip {
        is_clipped: true,
        clip_rect: if target_transform_id > local_transform_id {
            MathUtil::MapClippedRect(&transform, rect)
        } else {
            MathUtil::ProjectClippedRect(&transform, rect)
        },
    }
}
// Source: cc/trees/draw_property_utils.cc:127-146.
fn ComputeLocalRectInTargetSpace<C: DrawContext>(
    rect: RectF,
    p: &DrawPropertyTrees<C>,
    current_transform_id: i32,
    target_transform_id: i32,
    target_effect_id: i32,
) -> ConditionalClip {
    let mut transform = C::Transform::default();
    if !p.GetToTarget(current_transform_id, target_effect_id, &mut transform) {
        return ConditionalClip::default();
    }
    ConditionalClip {
        is_clipped: true,
        clip_rect: if current_transform_id > target_transform_id {
            MathUtil::MapClippedRect(&transform, rect)
        } else {
            MathUtil::ProjectClippedRect(&transform, rect)
        },
    }
}
// Source: cc/trees/draw_property_utils.cc:148-168.
fn ComputeCurrentClip<C: DrawContext>(
    node: &ClipNode,
    p: &DrawPropertyTrees<C>,
    target_transform_id: i32,
    target_effect_id: i32,
) -> ConditionalClip {
    if node.transform_id != target_transform_id {
        return ComputeLocalRectInTargetSpace::<C>(
            node.clip,
            p,
            node.transform_id,
            target_transform_id,
            target_effect_id,
        );
    }
    let mut clip = node.clip;
    let scale = p
        .effect_tree()
        .Node(target_effect_id)
        .surface_contents_scale;
    if scale.x() > 0.0 && scale.y() > 0.0 && node.transform_id != kRootPropertyNodeId {
        clip.Scale(scale.x(), scale.y());
    }
    ConditionalClip {
        is_clipped: true,
        clip_rect: clip,
    }
}
// Source: cc/trees/draw_property_utils.cc:170-205.
fn ExpandClipForPixelMovingFilter<C: DrawContext>(
    p: &DrawPropertyTrees<C>,
    target_id: i32,
    node: &EffectNode<C::Filters, C::Path>,
    clip: &mut RectF,
) -> bool {
    let mut mapping = RectF::default();
    if !ConvertRectBetweenSurfaceSpaces::<C>(p, target_id, node.id, *clip, &mut mapping) {
        return false;
    }
    let expanded = RectF::from(
        node.filters
            .ExpandRect(ToEnclosingClipRect(mapping), node.surface_contents_scale),
    );
    let original = *clip;
    if !ConvertRectBetweenSurfaceSpaces::<C>(p, node.id, target_id, expanded, clip) {
        return false;
    }
    clip.Union(original);
    true
}
// Source: cc/trees/draw_property_utils.cc:207-231.
fn ApplyClipNodeToAccumulatedClip<C: DrawContext>(
    p: &DrawPropertyTrees<C>,
    include_expanding_clips: bool,
    target_id: i32,
    target_transform_id: i32,
    node: &ClipNode,
    clip: &mut RectF,
) -> bool {
    if !node.AppliesLocalClip() {
        if !include_expanding_clips {
            return true;
        }
        return ExpandClipForPixelMovingFilter::<C>(
            p,
            target_id,
            p.effect_tree().Node(node.pixel_moving_filter_id),
            clip,
        );
    }
    let current = ComputeCurrentClip::<C>(node, p, target_transform_id, target_id);
    if !current.is_clipped {
        return false;
    }
    clip.Intersect(current.clip_rect);
    true
}
// Source: cc/trees/draw_property_utils.cc:233-350.
fn ComputeAccumulatedClip<C: DrawContext>(
    p: &DrawPropertyTrees<C>,
    include_expanding_clips: bool,
    local_clip_id: i32,
    target_id: i32,
) -> ConditionalClip {
    let mut cached_data = p.FetchClipRectFromCache(local_clip_id, target_id);
    if cached_data.target_id != kInvalidPropertyNodeId {
        return cached_data.clip;
    }
    cached_data.target_id = target_id;
    let clips = p.clip_tree();
    let mut node = clips.Node(local_clip_id);
    let effects = p.effect_tree();
    let mut target = effects.Node(target_id);
    let target_transform_id = target.transform_id;
    let mut cache_hit = false;
    let mut cached_clip = ConditionalClip::default();
    let unclipped = ConditionalClip::default();
    let mut parent_chain = Vec::new();
    while target.clip_id > node.id
        || effects
            .GetRenderSurface(target.id)
            .expect("render surface")
            .has_contributing_layer_that_escapes_clip()
    {
        target = effects.Node(target.target_id);
    }
    while target.clip_id < node.id {
        if !parent_chain.is_empty() {
            for data in node.cached_clip_rects.borrow().iter() {
                if data.target_id == target_id {
                    cache_hit = true;
                    cached_clip = data.clip;
                }
            }
        }
        parent_chain.push(node.id);
        node = if clips.HasParent(node) {
            clips.parent(node)
        } else {
            panic!("source null clip ancestor");
        };
    }
    if parent_chain.is_empty() {
        cached_data.clip = unclipped;
        return unclipped;
    }
    node = clips.Node(parent_chain.pop().unwrap());
    let mut accumulated;
    if cache_hit && cached_clip.is_clipped {
        accumulated = cached_clip.clip_rect;
        if !ApplyClipNodeToAccumulatedClip::<C>(
            p,
            include_expanding_clips,
            target_id,
            target_transform_id,
            node,
            &mut accumulated,
        ) {
            cached_data.clip = unclipped;
            return unclipped;
        }
    } else {
        while !node.AppliesLocalClip() && !parent_chain.is_empty() {
            node = clips.Node(parent_chain.pop().unwrap());
        }
        if !node.AppliesLocalClip() {
            cached_data.clip = unclipped;
            return unclipped;
        }
        let current = ComputeCurrentClip::<C>(node, p, target_transform_id, target_id);
        if !current.is_clipped {
            cached_data.clip = unclipped;
            return unclipped;
        }
        accumulated = current.clip_rect;
    }
    while let Some(id) = parent_chain.pop() {
        node = clips.Node(id);
        if !ApplyClipNodeToAccumulatedClip::<C>(
            p,
            include_expanding_clips,
            target_id,
            target_transform_id,
            node,
            &mut accumulated,
        ) {
            cached_data.clip = unclipped;
            return unclipped;
        }
    }
    let clip = ConditionalClip {
        is_clipped: true,
        clip_rect: if accumulated.IsEmpty() {
            RectF::default()
        } else {
            accumulated
        },
    };
    cached_data.clip = clip;
    clip
}
// Source: cc/trees/draw_property_utils.cc:352-355.
fn HasSingularTransform<T: PropertyTreeTransform>(id: i32, tree: &TransformTree<T>) -> bool {
    let node = tree.Node(id);
    !node.is_invertible || !node.ancestors_are_invertible
}
// Source: cc/trees/draw_property_utils.cc:357-370.
fn LowestCommonAncestor(a: i32, b: i32, tree: &ClipTree) -> i32 {
    let mut a = tree.Node(a);
    let mut b = tree.Node(b);
    while a.id != b.id {
        if a.id < b.id {
            b = tree.parent(b);
        } else {
            a = tree.parent(a);
        }
    }
    a.id
}
// Source: cc/trees/draw_property_utils.cc:372-390.
fn UpdateRenderSurfaceCommonAncestorClip<C: DrawContext>(
    context: &C,
    id: C::Layer,
    p: &mut DrawPropertyTrees<C>,
) {
    let mut surface_id = context.layer(id).render_target(p).expect("target");
    let mut clip_id = context.layer(id).clip_tree_index();
    loop {
        let (common, parent_id, effect_id) = {
            let surface = p
                .effect_tree()
                .GetRenderSurface(surface_id)
                .expect("surface");
            (
                surface.common_ancestor_clip_id(p),
                surface.render_target(p),
                surface.EffectTreeIndex(),
            )
        };
        if clip_id >= common {
            break;
        }
        clip_id = LowestCommonAncestor(clip_id, common, p.clip_tree());
        let own_clip_id = p.effect_tree().Node(surface_id).clip_id;
        p.effect_tree_mutable()
            .GetRenderSurfaceMutable(surface_id)
            .expect("surface")
            .set_common_ancestor_clip_id(clip_id, own_clip_id);
        let parent_effect = p
            .effect_tree()
            .GetRenderSurface(parent_id)
            .expect("parent surface")
            .EffectTreeIndex();
        if parent_effect == effect_id {
            break;
        }
        surface_id = parent_id;
    }
}
// Source: cc/trees/draw_property_utils.cc:392-403.
fn ClearRenderSurfaceCommonAncestorClip<C: DrawContext>(
    context: &C,
    id: C::Layer,
    p: &mut DrawPropertyTrees<C>,
) {
    let mut surface_id = context.layer(id).render_target(p).expect("target");
    loop {
        let (escapes, parent_id, effect_id) = {
            let surface = p
                .effect_tree()
                .GetRenderSurface(surface_id)
                .expect("surface");
            (
                surface.has_contributing_layer_that_escapes_clip(),
                surface.render_target(p),
                surface.EffectTreeIndex(),
            )
        };
        if !escapes {
            break;
        }
        let own_clip_id = p.effect_tree().Node(surface_id).clip_id;
        p.effect_tree_mutable()
            .GetRenderSurfaceMutable(surface_id)
            .expect("surface")
            .set_common_ancestor_clip_id(kInvalidPropertyNodeId, own_clip_id);
        let parent_effect = p
            .effect_tree()
            .GetRenderSurface(parent_id)
            .expect("parent surface")
            .EffectTreeIndex();
        if parent_effect == effect_id {
            break;
        }
        surface_id = parent_id;
    }
}
// Source: cc/trees/draw_property_utils.cc:406-418.
fn TransformNodeForBackfaceVisibility<'a, C: DrawContext>(
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    tree: &'a TransformTree<C::Transform>,
) -> &'a TransformNode<C::Transform> {
    let mut node = tree.Node(layer.transform_tree_index());
    while node.delegates_to_parent_for_backface {
        let parent = tree.Node(node.parent_id);
        debug_assert!(
            node.sorting_context_id == 0 || parent.sorting_context_id == node.sorting_context_id
        );
        node = parent;
    }
    node
}
// Source: cc/trees/draw_property_utils.cc:420-430.
fn IsTargetSpaceTransformBackFaceVisible<C: DrawContext>(
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    p: &DrawPropertyTrees<C>,
) -> bool {
    let node = TransformNodeForBackfaceVisibility::<C>(layer, p.transform_tree());
    let mut transform = C::Transform::default();
    p.GetToTarget(
        node.id,
        layer.render_target_effect_tree_index(p),
        &mut transform,
    );
    transform.IsBackFaceVisible()
}
// Source: cc/trees/draw_property_utils.cc:432-463.
fn IsTransformToRootOf3DRenderingContextBackFaceVisible<C: DrawContext>(
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    p: &DrawPropertyTrees<C>,
) -> bool {
    let tree = p.transform_tree();
    let node = TransformNodeForBackfaceVisibility::<C>(layer, tree);
    let mut root = node;
    let mut root_id = node.id;
    let sorting_context_id = node.sorting_context_id;
    while root_id > kRootPropertyNodeId {
        let parent_id = root.parent_id;
        let parent = tree.Node(parent_id);
        if parent.sorting_context_id != sorting_context_id {
            break;
        }
        root_id = parent_id;
        root = parent;
    }
    let mut transform = C::Transform::default();
    if node.id != root_id {
        tree.CombineTransformsBetween(node.id, root_id, &mut transform);
    }
    transform.PreConcat(&root.to_parent);
    transform.IsBackFaceVisible()
}
// Source: cc/trees/draw_property_utils.cc:465-486.
fn IsLayerBackFaceVisible<C: DrawContext>(
    context: &C,
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    p: &DrawPropertyTrees<C>,
    main_thread: bool,
) -> bool {
    if main_thread {
        return false;
    }
    if HasSingularTransform(layer.transform_tree_index(), p.transform_tree()) {
        return false;
    }
    if context.enable_backface_visibility_interop() {
        IsTransformToRootOf3DRenderingContextBackFaceVisible::<C>(layer, p)
    } else {
        IsTargetSpaceTransformBackFaceVisible::<C>(layer, p)
    }
}
// Source: cc/trees/draw_property_utils.cc:489-533.
fn LayerNeedsUpdate<C: DrawContext>(
    context: &C,
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    layer_is_drawn: bool,
    p: &DrawPropertyTrees<C>,
    main_thread: bool,
) -> bool {
    compositor_draw::draw_property_policy::LayerNeedsUpdate(
        layer_is_drawn,
        || layer.draws_content(),
        || layer.bounds(p).IsEmpty(),
        || {
            p.effect_tree()
                .Node(layer.effect_tree_index())
                .filters
                .HasReferenceFilter()
        },
        || layer.should_check_backface_visibility(),
        || IsLayerBackFaceVisible::<C>(context, layer, p, main_thread),
    )
}
// Source: cc/trees/draw_property_utils.cc:535-550.
fn MainLayerShouldBeSkipped<C: DrawContext>(
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    p: &DrawPropertyTrees<C>,
) -> bool {
    let effect = p.effect_tree().Node(layer.effect_tree_index());
    let node = p.transform_tree().Node(layer.transform_tree_index());
    compositor_draw::draw_property_policy::MainLayerShouldBeSkipped(
        effect.HasRenderSurface() && effect.subtree_has_copy_request,
        node.node_and_ancestors_are_animated_or_invertible,
        effect.is_drawn,
    )
}
// Source: cc/trees/draw_property_utils.cc:552-560.
fn LayerDrawableContentRect<C: DrawContext>(
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    bounds: Rect,
    clip: Rect,
) -> Rect {
    compositor_draw::draw_property_policy::LayerDrawableContentRect(
        layer.is_clipped(),
        bounds,
        clip,
    )
}
// Source: cc/trees/draw_property_utils.cc:562-582.
fn SetSurfaceIsClipped<C: DrawContext>(p: &mut DrawPropertyTrees<C>, surface_id: i32) {
    let surface = p
        .effect_tree()
        .GetRenderSurface(surface_id)
        .expect("surface");
    let mut clipped = false;
    if surface.EffectTreeIndex() == kContentsRootPropertyNodeId {
        clipped = true;
    } else {
        let parent_clip = p
            .effect_tree()
            .GetRenderSurface(surface.render_target(p))
            .unwrap()
            .common_ancestor_clip_id(p);
        let mut id = surface.common_ancestor_clip_id(p);
        while id != parent_clip {
            let node = p.clip_tree().Node(id);
            if node.AppliesLocalClip() {
                clipped = true;
                break;
            }
            id = p.clip_tree().parent(node).id;
        }
    }
    p.effect_tree_mutable()
        .GetRenderSurfaceMutable(surface_id)
        .expect("surface")
        .SetIsClipped(clipped);
}
// Source: cc/trees/draw_property_utils.cc:584-598.
fn SetSurfaceDrawOpacity<C: DrawContext>(p: &mut DrawPropertyTrees<C>, surface_id: i32) {
    let surface = p
        .effect_tree()
        .GetRenderSurface(surface_id)
        .expect("surface");
    let tree = p.effect_tree();
    let node = tree.Node(surface.EffectTreeIndex());
    let mut opacity = tree.EffectiveOpacity(node);
    let mut parent = if tree.HasParent(node) {
        Some(tree.parent(node))
    } else {
        None
    };
    while let Some(node) = parent {
        if node.HasRenderSurface() {
            break;
        }
        opacity *= tree.EffectiveOpacity(node);
        parent = if tree.HasParent(node) {
            Some(tree.parent(node))
        } else {
            None
        };
    }
    p.effect_tree_mutable()
        .GetRenderSurfaceMutable(surface_id)
        .expect("surface")
        .SetDrawOpacity(opacity);
}
// Source: cc/trees/draw_property_utils.cc:600-617.
fn LayerDrawOpacity<C: DrawContext>(
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    p: &DrawPropertyTrees<C>,
) -> f32 {
    let Some(target_id) = layer.render_target(p) else {
        return 0.0;
    };
    let tree = p.effect_tree();
    let target = p.effect_tree().GetRenderSurface(target_id).unwrap();
    let target_node = tree.Node(target.EffectTreeIndex());
    let mut node = tree.Node(layer.effect_tree_index());
    if node.id == target_node.id {
        return 1.0;
    }
    let mut opacity = 1.0;
    while node.id != target_node.id {
        opacity *= tree.EffectiveOpacity(node);
        node = if tree.HasParent(node) {
            tree.parent(node)
        } else {
            panic!("source null target ancestor");
        };
    }
    opacity
}
// Source: cc/trees/draw_property_utils.cc:620-628.
fn ScreenSpaceTransformInternal<C: DrawContext>(
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    tree: &TransformTree<C::Transform>,
) -> C::Transform {
    ScreenSpaceTransformForLayer(
        layer.offset_to_transform_parent(),
        layer.transform_tree_index(),
        tree,
    )
}
// Rust borrow-boundary entry for the same source cc/trees/draw_property_utils.cc:620-628.
// Both native Layer and LayerImpl forward scalar live getter results to this
// unique body; it never stores or copies a layer or property tree.
pub fn ScreenSpaceTransformForLayer<T: crate::adapter::property_tree::PropertyTreeTransform>(
    offset: Vector2dF,
    transform_tree_index: i32,
    tree: &TransformTree<T>,
) -> T {
    let mut transform = T::default();
    transform.Translate(offset.x(), offset.y());
    let screen = tree.ToScreen(transform_tree_index);
    transform.PostConcat(screen);
    transform
}
// Source: cc/trees/draw_property_utils.cc:630-655.
fn SetSurfaceClipRect<C: DrawContext>(p: &mut DrawPropertyTrees<C>, surface_id: i32) {
    let surface = p
        .effect_tree()
        .GetRenderSurface(surface_id)
        .expect("surface");
    if !surface.is_clipped() {
        p.effect_tree_mutable()
            .GetRenderSurfaceMutable(surface_id)
            .expect("surface")
            .SetClipRect(Rect::default());
        return;
    }
    let effects = p.effect_tree();
    let node = effects.Node(surface.EffectTreeIndex());
    let target = effects.Node(node.target_id);
    if surface.EffectTreeIndex() == kContentsRootPropertyNodeId {
        let rect = ToEnclosingClipRect(p.clip_tree().Node(node.clip_id).clip);
        p.effect_tree_mutable()
            .GetRenderSurfaceMutable(surface_id)
            .expect("surface")
            .SetClipRect(rect);
    } else {
        let mut clip =
            ComputeAccumulatedClip::<C>(p, false, surface.common_ancestor_clip_id(p), target.id);
        let offset = effects
            .GetRenderSurface(surface.render_target(p))
            .unwrap()
            .pixel_alignment_offset();
        clip.clip_rect.Offset(offset.x(), offset.y());
        let rect = ToEnclosingClipRect(clip.clip_rect);
        p.effect_tree_mutable()
            .GetRenderSurfaceMutable(surface_id)
            .expect("surface")
            .SetClipRect(rect);
    }
}
// Source: cc/trees/draw_property_utils.cc:657-696.
fn SetSurfaceDrawTransform<C: DrawContext>(p: &mut DrawPropertyTrees<C>, surface_id: i32) {
    let surface = p
        .effect_tree()
        .GetRenderSurface(surface_id)
        .expect("surface");
    let transforms = p.transform_tree();
    let effects = p.effect_tree();
    let node = transforms.Node(surface.TransformTreeIndex(p));
    let effect = effects.Node(surface.EffectTreeIndex());
    if surface.EffectTreeIndex() == kContentsRootPropertyNodeId {
        p.effect_tree_mutable()
            .GetRenderSurfaceMutable(surface_id)
            .expect("surface")
            .SetDrawTransform(C::Transform::default(), Vector2dF::default());
        return;
    }
    let mut transform = C::Transform::default();
    let target = effects.Node(effect.target_id);
    p.GetToTarget(node.id, target.id, &mut transform);
    ConcatInverseSurfaceContentsScale::<C>(effect, &mut transform);
    let mut offset = Vector2dF::default();
    transform.PostTranslate(
        effects
            .GetRenderSurface(surface.render_target(p))
            .unwrap()
            .pixel_alignment_offset(),
    );
    if effect.render_surface_reason
        != RenderSurfaceReason::k2DScaleTransformWithCompositedDescendants
    {
        if let Some(alignment) = PixelAlignmentOffset(&surface.screen_space_transform(), &transform)
        {
            offset = alignment;
            transform.Translate(-offset.x(), -offset.y());
        }
    }
    p.effect_tree_mutable()
        .GetRenderSurfaceMutable(surface_id)
        .expect("surface")
        .SetDrawTransform(transform, offset);
}
// Source: cc/trees/draw_property_utils.cc:699-708.
fn LayerClipRect<C: DrawContext>(
    p: &DrawPropertyTrees<C>,
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
) -> ConditionalClip {
    let effect = p.effect_tree().Node(layer.effect_tree_index());
    let target = if effect.HasRenderSurface() {
        effect
    } else {
        p.effect_tree().Node(effect.target_id)
    };
    ComputeAccumulatedClip::<C>(p, false, layer.clip_tree_index(), target.id)
}
// Source: cc/trees/draw_property_utils.cc:710-779.
fn GetMaskFilterInfoPair<C: DrawContext>(
    p: &DrawPropertyTrees<C>,
    effect_tree_index: i32,
    for_render_surface: bool,
) -> (MaskFilterInfo, bool) {
    let empty = || (MaskFilterInfo::default(), false);
    let tree = p.effect_tree();
    let effect = tree.Node(effect_tree_index);
    let target_id = effect.target_id;
    if effect.HasRenderSurface() && !for_render_surface {
        return empty();
    }
    let mut node = Some(effect);
    let mut found = false;
    while let Some(current) = node {
        found = !current.mask_filter_info.IsEmpty();
        if found {
            break;
        }
        if current.HasRenderSurface() && (!for_render_surface || effect.id != current.id) {
            break;
        }
        if current.id == target_id {
            break;
        }
        node = if tree.HasParent(current) {
            Some(tree.parent(current))
        } else {
            None
        };
    }
    let Some(node) = node else {
        return empty();
    };
    if !found {
        return empty();
    }
    let mut transform_id = node.transform_id;
    if let Some(clip_id) = node.mask_filter_info.clip_id {
        transform_id = p.clip_tree().Node(clip_id).transform_id;
    }
    let mut transform = C::Transform::default();
    if !p.GetToTarget(transform_id, target_id, &mut transform) {
        return empty();
    }
    let mut result = (node.mask_filter_info.clone(), node.is_fast_rounded_corner);
    transform.ApplyMaskFilterInfo(&mut result.0);
    if result.0.IsEmpty() {
        return empty();
    }
    result
}
struct RenderTargetUpdateAdapter<'a, C: DrawContext> {
    context: &'a C,
    p: &'a mut DrawPropertyTrees<C>,
}

impl<C: DrawContext> compositor_tree::render_target_update::RenderTargetUpdateContext
    for RenderTargetUpdateAdapter<'_, C>
{
    type Layer = C::Layer;
    type Resource = ViewTransitionElementResourceId;

    fn root_effect_id(&self) -> i32 {
        kContentsRootPropertyNodeId
    }

    fn effect_node_count(&self) -> i32 {
        self.p.effect_tree().size() as i32
    }

    fn effect_resource(&self, id: i32) -> Option<Self::Resource> {
        let resource = self
            .p
            .effect_tree()
            .Node(id)
            .view_transition_element_resource_id;
        resource.IsValid().then_some(resource)
    }

    fn clear_view_transition_target(&mut self, id: i32) {
        self.p
            .effect_tree_mutable()
            .MutableNode(id)
            .view_transition_target_id = kInvalidPropertyNodeId;
    }

    fn parent_target_state(
        &self,
        id: i32,
    ) -> compositor_tree::render_target_update::ParentTargetState {
        let tree = self.p.effect_tree();
        let node = tree.Node(id);
        let parent = tree.parent(node);
        compositor_tree::render_target_update::ParentTargetState {
            parent_id: node.parent_id,
            parent_has_render_surface: parent.HasRenderSurface(),
            parent_target_id: parent.target_id,
        }
    }

    fn set_target(&mut self, id: i32, target_id: i32) {
        self.p.effect_tree_mutable().MutableNode(id).target_id = target_id;
    }

    fn backdrop_filter_disallows_lcd_text(&self, id: i32) -> bool {
        let node = self.p.effect_tree().Node(id);
        node.has_potential_backdrop_filter_animation || !node.backdrop_filters.AllowsLCDText()
    }

    fn set_lcd_text_disallowed_by_backdrop_filter(&mut self, id: i32, value: bool) {
        self.p
            .effect_tree_mutable()
            .MutableNode(id)
            .lcd_text_disallowed_by_backdrop_filter = value;
    }

    fn target(&self, id: i32) -> i32 {
        self.p.effect_tree().Node(id).target_id
    }

    fn layer_count(&self) -> usize {
        self.context.layer_count()
    }

    fn layer_at(&self, index: usize) -> Self::Layer {
        self.context.layer_id_at(index)
    }

    fn layer_resource(&self, layer: Self::Layer) -> Option<Self::Resource> {
        let resource = self.context.layer(layer).ViewTransitionResourceId();
        resource.IsValid().then_some(resource)
    }

    fn layer_effect_id(&self, layer: Self::Layer) -> i32 {
        self.context.layer(layer).effect_tree_index()
    }

    fn effect_has_render_surface(&self, id: i32) -> bool {
        self.p.effect_tree().Node(id).HasRenderSurface()
    }

    fn set_view_transition_target(&mut self, id: i32, target_id: i32) {
        self.p
            .effect_tree_mutable()
            .MutableNode(id)
            .view_transition_target_id = target_id;
    }
}

// Source: cc/trees/draw_property_utils.cc:781-856.
fn UpdateRenderTarget<C: DrawContext>(context: &C, p: &mut DrawPropertyTrees<C>) {
    let mut adapter = RenderTargetUpdateAdapter { context, p };
    compositor_tree::render_target_update::UpdateRenderTargets(&mut adapter);
}
// Source: cc/trees/draw_property_utils.cc:858-885.
struct ClipTreeUpdateAdapter<'a, C: DrawContext> {
    p: &'a mut DrawPropertyTrees<C>,
}

impl<C: DrawContext> compositor_tree::clip_tree_update::ClipTreeUpdateContext
    for ClipTreeUpdateAdapter<'_, C>
{
    fn needs_update(&self) -> bool {
        self.p.clip_tree().needs_update()
    }

    fn first_node_id(&self) -> i32 {
        kViewportPropertyNodeId
    }

    fn node_count(&self) -> i32 {
        self.p.clip_tree().size() as i32
    }

    fn clear_node_cache(&mut self, id: i32) {
        self.p
            .clip_tree()
            .Node(id)
            .cached_clip_rects
            .borrow_mut()
            .clear();
    }

    fn initialize_viewport_clip(&mut self, id: i32) {
        let clip = self.p.clip_tree().Node(id).clip;
        self.p
            .clip_tree_mutable()
            .MutableNode(id)
            .cached_accumulated_rect_in_screen_space = clip;
    }

    fn update_accumulated_clip(&mut self, id: i32) -> bool {
        let node = self.p.clip_tree().Node(id);
        let mut clip = self
            .p
            .clip_tree()
            .parent(node)
            .cached_accumulated_rect_in_screen_space;
        let success = ApplyClipNodeToAccumulatedClip::<C>(
            self.p,
            true,
            kContentsRootPropertyNodeId,
            kRootPropertyNodeId,
            node,
            &mut clip,
        );
        if success {
            self.p
                .clip_tree_mutable()
                .MutableNode(id)
                .cached_accumulated_rect_in_screen_space = clip;
        }
        success
    }

    fn clear_needs_update(&mut self) {
        self.p.clip_tree_mutable().set_needs_update(false);
    }
}

pub(crate) fn ComputeClips<C: DrawContext>(p: &mut DrawPropertyTrees<C>) {
    debug_assert!(!p.transform_tree().needs_update());
    let mut adapter = ClipTreeUpdateAdapter::<C> { p };
    compositor_tree::clip_tree_update::UpdateClipTree(&mut adapter);
}
// Source: cc/trees/draw_property_utils.cc:887-907.
fn ComputeSurfaceDrawProperties<C: DrawContext>(p: &mut DrawPropertyTrees<C>, surface_id: i32) {
    SetSurfaceIsClipped::<C>(p, surface_id);
    SetSurfaceDrawOpacity::<C>(p, surface_id);
    let (transform_id, effect_id) = {
        let surface = p
            .effect_tree()
            .GetRenderSurface(surface_id)
            .expect("surface");
        (surface.TransformTreeIndex(p), surface.EffectTreeIndex())
    };
    let screen = p.ToScreenSpaceTransformWithoutSurfaceContentsScale(transform_id, effect_id);
    p.effect_tree_mutable()
        .GetRenderSurfaceMutable(surface_id)
        .expect("surface")
        .SetScreenSpaceTransform(screen);
    SetSurfaceDrawTransform::<C>(p, surface_id);
    let mut pair = GetMaskFilterInfoPair::<C>(p, effect_id, true);
    let mut transform = C::Transform::default();
    transform.Translate(
        p.effect_tree()
            .GetRenderSurface(
                p.effect_tree()
                    .GetRenderSurface(surface_id)
                    .expect("surface")
                    .render_target(p),
            )
            .unwrap()
            .pixel_alignment_offset()
            .x(),
        p.effect_tree()
            .GetRenderSurface(
                p.effect_tree()
                    .GetRenderSurface(surface_id)
                    .expect("surface")
                    .render_target(p),
            )
            .unwrap()
            .pixel_alignment_offset()
            .y(),
    );
    transform.ApplyMaskFilterInfo(&mut pair.0);
    p.effect_tree_mutable()
        .GetRenderSurfaceMutable(surface_id)
        .expect("surface")
        .SetMaskFilterInfo(pair.0, pair.1);
    SetSurfaceClipRect::<C>(p, surface_id);
}
// Source: cc/trees/draw_property_utils.cc:1011-1031.
fn SkipForInvertibility<C: DrawContext>(
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    p: &DrawPropertyTrees<C>,
) -> bool {
    let transform = p.transform_tree().Node(layer.transform_tree_index());
    let effect = p.effect_tree().Node(layer.effect_tree_index());
    let non_root = effect.closest_ancestor_with_copy_request_id > kContentsRootPropertyNodeId;
    let mut from_target = C::Transform::default();
    if non_root {
        !p.GetFromTarget(
            layer.transform_tree_index(),
            effect.closest_ancestor_with_copy_request_id,
            &mut from_target,
        )
    } else {
        !transform.ancestors_are_invertible
    }
}
// Source: cc/trees/draw_property_utils.cc:1033-1049.
fn ComputeLayerDrawTransforms<C: DrawContext>(
    context: &mut C,
    id: C::Layer,
    p: &DrawPropertyTrees<C>,
) {
    let (screen, target, animating, pair) = {
        let layer = context.layer(id);
        let node = p.transform_tree().Node(layer.transform_tree_index());
        (
            ScreenSpaceTransformInternal::<C>(layer, p.transform_tree()),
            DrawTransform::<C>(layer, p),
            node.to_screen_is_potentially_animated,
            GetMaskFilterInfoPair::<C>(p, layer.effect_tree_index(), false),
        )
    };
    let layer = context.layer_mut(id);
    layer.SetScreenSpaceTransform(screen);
    layer.SetTargetSpaceTransform(target);
    layer.SetScreenSpaceTransformIsAnimating(animating);
    layer.SetMaskFilterInfo(pair.0, pair.1);
}
// Source: cc/trees/draw_property_utils.cc:1051-1055.
fn ComputeLayerDrawOpacity<C: DrawContext>(
    context: &mut C,
    id: C::Layer,
    p: &DrawPropertyTrees<C>,
) {
    let opacity = LayerDrawOpacity::<C>(context.layer(id), p);
    context.layer_mut(id).SetDrawOpacity(opacity);
}
// Source: cc/trees/draw_property_utils.cc:1057-1066.
fn ComputeLayerClipAndVisibleRect<C: DrawContext>(
    context: &mut C,
    id: C::Layer,
    p: &DrawPropertyTrees<C>,
) {
    let clip = {
        let layer = context.layer(id);
        LayerClipRect::<C>(p, layer)
    };
    {
        // LayerVisibleRect reads both of these values. Chromium deliberately
        // installs the current clip before computing visibility; using the
        // previous frame's clip can transiently empty every visible layer.
        let layer = context.layer_mut(id);
        layer.SetIsClipped(clip.is_clipped);
        layer.SetClipRect(ToEnclosingClipRect(clip.clip_rect));
    }
    let visible = {
        let layer = context.layer(id);
        LayerVisibleRect::<C>(layer, layer.bounds(p), Some(p))
    };
    context.layer_mut(id).SetVisibleLayerRect(visible);
}
// Source: cc/trees/draw_property_utils.cc:1068-1082.
fn ComputeLayerDrawableContentRect<C: DrawContext>(
    context: &mut C,
    id: C::Layer,
    p: &DrawPropertyTrees<C>,
) {
    let layer = context.layer(id);
    let only_visible = p
        .effect_tree()
        .Node(layer.effect_tree_index())
        .only_draws_visible_content;
    let mut bounds = layer.visible_layer_rect();
    if !only_visible {
        bounds = Rect::new(0, 0, layer.bounds(p).width(), layer.bounds(p).height());
    }
    let visible = MathUtil::MapEnclosingClippedRect(&layer.target_space_transform(p), bounds);
    let visible = LayerDrawableContentRect::<C>(layer, visible, layer.clip_rect());
    context.layer_mut(id).SetVisibleDrawableContentRect(visible);
}
// Source: cc/trees/draw_property_utils.cc:1084-1139.
fn AdjustLayerDrawPropertiesForPixelAlignmentOffset<C: DrawContext>(
    context: &mut C,
    id: C::Layer,
    p: &DrawPropertyTrees<C>,
) {
    let layer = context.layer(id);
    let mut offset = p
        .effect_tree()
        .GetRenderSurface(layer.render_target(p).unwrap())
        .unwrap()
        .pixel_alignment_offset();
    if offset.IsZero() {
        return;
    }
    if !context.ViewTransitionsNewRoundingChange() {
        if layer.IsViewTransitionContentLayer() && !layer.target_space_transform(p).HasPerspective()
        {
            offset.set_x(((offset.x() * 64.0).round() / 64.0).floor());
            offset.set_y(((offset.y() * 64.0).round() / 64.0).floor());
        }
    }
    let mut transform = layer.target_space_transform(p);
    transform.PostTranslate(offset);
    let mut translation = C::Transform::default();
    translation.Translate(offset.x(), offset.y());
    let clip = if layer.is_clipped() {
        let mut clip = LayerClipRect::<C>(p, layer);
        assert!(clip.is_clipped);
        clip.clip_rect.Offset(offset.x(), offset.y());
        Some(ToEnclosingClipRect(clip.clip_rect))
    } else {
        None
    };
    let layer = context.layer_mut(id);
    layer.SetTargetSpaceTransform(transform);
    layer.ApplyMaskFilterInfoTransform(&translation);
    if let Some(clip) = clip {
        layer.SetClipRect(clip);
    }
    ComputeLayerDrawableContentRect::<C>(context, id, p);
}
struct InitialRenderSurfaceAdapter<'a, C: DrawContext> {
    context: &'a mut C,
    p: &'a mut DrawPropertyTrees<C>,
}

impl<C: DrawContext> compositor_draw::initial_render_surface_list::InitialRenderSurfaceContext
    for InitialRenderSurfaceAdapter<'_, C>
{
    type Layer = C::Layer;
    type Resource = ViewTransitionElementResourceId;

    fn effect_node_count(&self) -> i32 {
        self.p.effect_tree().size() as i32
    }

    fn reset_surface_state(&mut self, id: i32) {
        if let Some(surface) = self.p.effect_tree_mutable().GetRenderSurfaceMutable(id) {
            surface.set_is_render_surface_list_member(false);
            surface.set_has_view_transition_capture_contributions(false);
            surface.reset_num_contributors();
        }
    }

    fn root_surface_id(&self) -> i32 {
        self.p
            .effect_tree()
            .GetRenderSurface(kContentsRootPropertyNodeId)
            .unwrap()
            .EffectTreeIndex()
    }

    fn surface_descriptor(
        &self,
        id: i32,
    ) -> compositor_draw::initial_render_surface_list::SurfaceDescriptor {
        let surface = self.p.effect_tree().GetRenderSurface(id).unwrap();
        let effect = self.p.effect_tree().Node(id);
        let resource = effect.view_transition_element_resource_id;
        compositor_draw::initial_render_surface_list::SurfaceDescriptor {
            effect_id: surface.EffectTreeIndex(),
            target_id: surface.render_target(self.p),
            view_transition_target_id: (effect.view_transition_target_id != kInvalidPropertyNodeId)
                .then_some(effect.view_transition_target_id),
            has_valid_view_transition_resource: resource.IsValid(),
            capture_token_matches: self.context.MatchesCaptureViewTransitionToken(resource),
            filter_moves_pixels: effect.filters.HasFilterThatMovesPixels(),
            contributes_to_drawn_surface: self.p.effect_tree().ContributesToDrawnSurface(id),
        }
    }

    fn surface_identity(
        &self,
        id: i32,
    ) -> compositor_draw::initial_render_surface_list::SurfaceIdentity {
        let surface = self.p.effect_tree().GetRenderSurface(id).unwrap();
        compositor_draw::initial_render_surface_list::SurfaceIdentity {
            effect_id: surface.EffectTreeIndex(),
            is_list_member: surface.is_render_surface_list_member(),
        }
    }

    fn initialize_surface(
        &mut self,
        id: i32,
        is_root: bool,
        descriptor: compositor_draw::initial_render_surface_list::SurfaceDescriptor,
    ) {
        let surface = self
            .p
            .effect_tree_mutable()
            .GetRenderSurfaceMutable(id)
            .expect("surface");
        surface.ClearAccumulatedContentRect();
        surface.set_is_render_surface_list_member(true);
        if descriptor.capture_token_matches {
            surface.set_has_view_transition_capture_contributions(true);
        }
        surface.set_contributes_to_drawn_surface(if is_root {
            false
        } else {
            descriptor.contributes_to_drawn_surface
        });
    }

    fn compute_surface_draw_properties(&mut self, id: i32) {
        ComputeSurfaceDrawProperties::<C>(self.p, id);
    }

    fn surface_copy_output_required(&self, id: i32) -> bool {
        self.p
            .effect_tree()
            .GetRenderSurface(id)
            .expect("surface")
            .CopyOfOutputRequired(self.p)
    }

    fn allow_undamaged_nonroot_render_pass_to_skip(&self) -> bool {
        self.context.AllowUndamagedNonrootRenderPassToSkip()
    }

    fn surface_is_unbounded(&self, id: i32) -> bool {
        self.p
            .effect_tree()
            .GetRenderSurface(id)
            .expect("surface")
            .IsUnbounded(self.p, self.context.enable_unbounded_element())
    }

    fn nearest_occlusion_immune_ancestor(&self, id: i32) -> Option<i32> {
        self.p
            .effect_tree()
            .GetRenderSurface(id)
            .unwrap()
            .nearest_occlusion_immune_ancestor()
    }

    fn set_nearest_occlusion_immune_ancestor(&mut self, id: i32, ancestor: Option<i32>) {
        self.p
            .effect_tree_mutable()
            .GetRenderSurfaceMutable(id)
            .expect("surface")
            .SetNearestOcclusionImmuneAncestor(ancestor);
    }

    fn layer_count(&self) -> usize {
        self.context.layer_count()
    }

    fn layer_at(&self, index: usize) -> Self::Layer {
        self.context.layer_id_at(index)
    }

    fn prepare_layer(&mut self, layer: Self::Layer) {
        self.context
            .layer(layer)
            .EnsureValidPropertyTreeIndices(self.p);
        let layer = self.context.layer_mut(layer);
        layer.set_contributes_to_drawn_render_surface(false);
        layer.set_raster_even_if_not_drawn(false);
    }

    fn layer_eligibility(
        &self,
        id: Self::Layer,
    ) -> compositor_draw::initial_render_surface_list::LayerEligibility {
        let layer = self.context.layer(id);
        let transform_id = layer.transform_tree_index();
        compositor_draw::initial_render_surface_list::LayerEligibility {
            is_root: self.context.IsRootLayer(id),
            skip_draw: LayerShouldBeSkippedForDrawPropertiesComputation::<C>(
                self.context,
                layer,
                self.p,
            ),
            skip_invertibility: SkipForInvertibility::<C>(layer, self.p),
            will_change_transform: transform_id != kInvalidPropertyNodeId
                && self
                    .p
                    .transform_tree()
                    .Node(transform_id)
                    .will_change_transform,
        }
    }

    fn set_layer_raster_even_if_not_drawn(&mut self, layer: Self::Layer, value: bool) {
        self.context
            .layer_mut(layer)
            .set_raster_even_if_not_drawn(value);
    }

    fn layer_draw_candidate(
        &self,
        id: Self::Layer,
    ) -> compositor_draw::initial_render_surface_list::LayerDrawCandidate<Self::Resource> {
        let layer = self.context.layer(id);
        let effect_id = layer.effect_tree_index();
        let resource = layer.ViewTransitionResourceId();
        compositor_draw::initial_render_surface_list::LayerDrawCandidate {
            target_id: layer.render_target(self.p).unwrap(),
            view_transition_resource: resource.IsValid().then_some(resource),
            needs_update: LayerNeedsUpdate::<C>(
                self.context,
                layer,
                self.p.effect_tree().Node(effect_id).is_drawn,
                self.p,
                false,
            ),
        }
    }

    fn adjust_layer_for_pixel_alignment(&mut self, layer: Self::Layer) {
        AdjustLayerDrawPropertiesForPixelAlignmentOffset::<C>(self.context, layer, self.p);
    }

    fn set_layer_contributes_to_drawn_surface(&mut self, layer: Self::Layer, value: bool) {
        self.context
            .layer_mut(layer)
            .set_contributes_to_drawn_render_surface(value);
    }

    fn accumulate_layer_into_surface(&mut self, surface_id: i32, layer: Self::Layer) {
        self.context
            .AccumulateContentRectFromContributingLayer(self.p, surface_id, layer);
    }

    fn increment_surface_contributors(&mut self, surface_id: i32) {
        self.p
            .effect_tree_mutable()
            .GetRenderSurfaceMutable(surface_id)
            .unwrap()
            .increment_num_contributors();
    }

    fn resource_matches_capture_token(&self, resource: Self::Resource) -> bool {
        self.context.MatchesCaptureViewTransitionToken(resource)
    }

    fn clear_originating_surface_content_rect(&mut self, layer: Self::Layer) {
        self.context
            .layer_mut(layer)
            .AsViewTransitionContentLayer()
            .expect("source valid resource requires VT content subtype")
            .SetOriginatingSurfaceContentRect(Rect::default());
    }

    fn capture_surface_descriptor(
        &self,
        id: i32,
    ) -> compositor_draw::initial_render_surface_list::CaptureSurfaceDescriptor {
        let surface = self.p.effect_tree().GetRenderSurface(id).unwrap();
        compositor_draw::initial_render_surface_list::CaptureSurfaceDescriptor {
            target_id: surface.render_target(self.p),
            has_valid_resource: self
                .p
                .effect_tree()
                .Node(id)
                .view_transition_element_resource_id
                .IsValid(),
            has_capture_contributions: surface.has_view_transition_capture_contributions(),
        }
    }

    fn capture_chain_surface(
        &self,
        id: i32,
    ) -> compositor_draw::initial_render_surface_list::CaptureChainSurface {
        let surface = self.p.effect_tree().GetRenderSurface(id).unwrap();
        compositor_draw::initial_render_surface_list::CaptureChainSurface {
            effect_id: surface.EffectTreeIndex(),
            target_id: surface.render_target(self.p),
            has_capture_contributions: surface.has_view_transition_capture_contributions(),
        }
    }

    fn mark_surface_has_capture_contributions(&mut self, id: i32) {
        self.p
            .effect_tree_mutable()
            .GetRenderSurfaceMutable(id)
            .unwrap()
            .set_has_view_transition_capture_contributions(true);
    }
}

// Source: cc/trees/draw_property_utils.cc:909-1009,1141-1262.
fn ComputeInitialRenderSurfaceList<C: DrawContext>(
    context: &mut C,
    p: &mut DrawPropertyTrees<C>,
    list: &mut Vec<i32>,
    content_layers: &mut BTreeMap<ViewTransitionElementResourceId, C::Layer>,
) {
    let mut adapter = InitialRenderSurfaceAdapter { context, p };
    let result =
        compositor_draw::initial_render_surface_list::ComputeInitialRenderSurfaceList(&mut adapter);
    list.extend_from_slice(&result.surface_list);
    content_layers.extend(result.content_layers);
}
struct SurfaceContentRectAdapter<'a, C: DrawContext> {
    context: &'a mut C,
    p: &'a mut DrawPropertyTrees<C>,
    max_texture_size: i32,
    content_layers: &'a BTreeMap<ViewTransitionElementResourceId, C::Layer>,
    content_rects: &'a mut Vec<(ViewTransitionElementResourceId, RectF)>,
}

impl<C: DrawContext> compositor_draw::render_surface_content::RenderSurfaceContentContext
    for SurfaceContentRectAdapter<'_, C>
{
    type Resource = ViewTransitionElementResourceId;
    type ContentRect = Rect;

    fn is_root_surface(&self, id: i32) -> bool {
        self.p
            .effect_tree()
            .GetRenderSurface(id)
            .unwrap()
            .EffectTreeIndex()
            == kContentsRootPropertyNodeId
    }

    fn set_root_content_rect_to_viewport(&mut self, id: i32) {
        let viewport_clip = self.p.clip_tree().ViewportClip();
        self.p
            .effect_tree_mutable()
            .GetRenderSurfaceMutable(id)
            .unwrap()
            .SetContentRectToViewport(viewport_clip);
    }

    fn calculate_surface_content_rect(&mut self, id: i32) {
        self.context.CalculateContentRectFromAccumulatedContentRect(
            self.p,
            id,
            self.max_texture_size,
        );
    }

    fn render_target(&self, id: i32) -> i32 {
        self.p
            .effect_tree()
            .GetRenderSurface(id)
            .unwrap()
            .render_target(self.p)
    }

    fn target_is_surface_list_member(&self, target_id: i32) -> bool {
        self.p
            .effect_tree()
            .GetRenderSurface(target_id)
            .unwrap()
            .is_render_surface_list_member()
    }

    fn accumulate_surface_into_target(&mut self, target_id: i32, contributor_id: i32) {
        self.context
            .AccumulateContentRectFromContributingRenderSurface(self.p, target_id, contributor_id);
    }

    fn increment_target_contributors(&mut self, target_id: i32) {
        self.p
            .effect_tree_mutable()
            .GetRenderSurfaceMutable(target_id)
            .unwrap()
            .increment_num_contributors();
    }

    fn view_transition_resource(&self, id: i32) -> Option<Self::Resource> {
        let resource = self
            .p
            .effect_tree()
            .Node(id)
            .view_transition_element_resource_id;
        resource.IsValid().then_some(resource)
    }

    fn view_transition_content_rect(&self, id: i32, resource: Self::Resource) -> Self::ContentRect {
        let surface = self.p.effect_tree().GetRenderSurface(id).unwrap();
        let output = if self.context.MatchesCaptureViewTransitionToken(resource) {
            surface.view_transition_capture_content_rect()
        } else {
            surface.content_rect()
        };
        surface
            .SurfaceScale(self.p)
            .InverseOrIdentity()
            .MapRectInt(output)
    }

    fn record_view_transition_content_rect(
        &mut self,
        resource: Self::Resource,
        rect: Self::ContentRect,
    ) {
        self.content_rects.push((resource, RectF::from(rect)));
    }

    fn update_originating_content_layer(
        &mut self,
        resource: Self::Resource,
        rect: Self::ContentRect,
    ) {
        if let Some(&layer) = self.content_layers.get(&resource) {
            self.context
                .layer_mut(layer)
                .AsViewTransitionContentLayer()
                .expect("source valid resource requires VT content subtype")
                .SetOriginatingSurfaceContentRect(rect);
        }
    }
}

// Source: cc/trees/draw_property_utils.cc:1264-1327.
fn ComputeSurfaceContentRects<C: DrawContext>(
    context: &mut C,
    p: &mut DrawPropertyTrees<C>,
    list: &[i32],
    max_texture_size: i32,
    content_layers: &BTreeMap<ViewTransitionElementResourceId, C::Layer>,
    content_rects: &mut Vec<(ViewTransitionElementResourceId, RectF)>,
) {
    let mut adapter = SurfaceContentRectAdapter {
        context,
        p,
        max_texture_size,
        content_layers,
        content_rects,
    };
    compositor_draw::render_surface_content::ComputeSurfaceContentRects(&mut adapter, list);
}
// Source: cc/trees/draw_property_utils.cc:1329-1385.
fn ComputeListOfNonEmptySurfaces<C: DrawContext>(
    context: &mut C,
    p: &mut DrawPropertyTrees<C>,
    initial: &[i32],
    final_list: &mut Vec<i32>,
) {
    use compositor_draw::render_surface_pruning::{
        LayerContributorCandidate, PlanLayerContributorPruning, PlanRenderSurfacePruning,
        RenderSurfacePruneCandidate,
    };

    let candidates = initial
        .iter()
        .map(|&id| {
            let surface = p.effect_tree().GetRenderSurface(id).unwrap();
            let surface_effect = surface.EffectTreeIndex();
            let target_id = surface.render_target(p);
            let content_is_empty = surface.content_rect().IsEmpty();
            let is_root = surface_effect == kContentsRootPropertyNodeId;
            RenderSurfacePruneCandidate {
                id: surface_effect,
                target_id,
                is_root,
                content_is_empty,
                // Preserve the source's short circuit: reference filters are
                // queried only for an empty, non-root surface.
                has_reference_filter: (!is_root && content_is_empty)
                    .then(|| p.effect_tree().Node(id).filters.HasReferenceFilter()),
                target_is_surface_list_member: p
                    .effect_tree()
                    .GetRenderSurface(target_id)
                    .unwrap()
                    .is_render_surface_list_member(),
                copy_output_required: surface.CopyOfOutputRequired(p),
            }
        })
        .collect::<Vec<_>>();
    let plan = PlanRenderSurfacePruning(&candidates);

    for &id in &plan.nonmember_surfaces {
        p.effect_tree_mutable()
            .GetRenderSurfaceMutable(id)
            .unwrap()
            .set_is_render_surface_list_member(false);
    }
    for &target_id in &plan.surface_contributor_decrement_targets {
        p.effect_tree_mutable()
            .GetRenderSurfaceMutable(target_id)
            .unwrap()
            .decrement_num_contributors();
    }

    if plan.requires_layer_cleanup() {
        let layer_candidates = (0..context.layer_count())
            .filter_map(|layer_index| {
                let id = context.layer_id_at(layer_index);
                let layer = context.layer(id);
                layer
                    .contributes_to_drawn_render_surface()
                    .then(|| LayerContributorCandidate {
                        layer_index,
                        target_id: layer.render_target(p).unwrap(),
                    })
            })
            .collect::<Vec<_>>();
        let layer_plan = PlanLayerContributorPruning(&plan.nonmember_surfaces, &layer_candidates);
        for (&layer_index, &target_id) in layer_plan
            .dropped_layer_indices
            .iter()
            .zip(&layer_plan.contributor_decrement_targets)
        {
            let id = context.layer_id_at(layer_index);
            context
                .layer_mut(id)
                .set_contributes_to_drawn_render_surface(false);
            p.effect_tree_mutable()
                .GetRenderSurfaceMutable(target_id)
                .unwrap()
                .decrement_num_contributors();
        }
    }
    final_list.extend_from_slice(&plan.final_surfaces);
}
// Source: cc/trees/draw_property_utils.cc:1387-1417.
fn CalculateRenderSurfaceLayerList<C: DrawContext>(
    context: &mut C,
    p: &mut DrawPropertyTrees<C>,
    list: &mut Vec<i32>,
    max_texture_size: i32,
) {
    let mut initial = Vec::new();
    let mut content_rects = Vec::new();
    let mut content_layers = BTreeMap::new();
    ComputeInitialRenderSurfaceList::<C>(context, p, &mut initial, &mut content_layers);
    ComputeSurfaceContentRects::<C>(
        context,
        p,
        &initial,
        max_texture_size,
        &content_layers,
        &mut content_rects,
    );
    ComputeListOfNonEmptySurfaces::<C>(context, p, &initial, list);
    for (id, rect) in content_rects {
        context.SetViewTransitionContentRect(id, rect);
    }
}
// Source: cc/trees/draw_property_utils.cc:1419-1447.
fn RecordRenderSurfaceReasonsForTracing<C: DrawContext>(
    context: &C,
    p: &DrawPropertyTrees<C>,
    list: &[i32],
) {
    if !context.TraceCategoryEnabled() || list.len() <= 1 {
        return;
    }
    context.TraceRenderSurfaceReasonCount("total", list.len());
    const N: usize = RenderSurfaceReason::kTest as usize;
    let mut counts = [0usize; N];
    for &id in list {
        let reason = p.effect_tree().Node(id).render_surface_reason;
        counts[reason as usize] += 1;
    }
    for i in 0..N {
        if counts[i] == 0 {
            continue;
        }
        let reason = ALL_RENDER_SURFACE_REASONS[i];
        context.TraceRenderSurfaceReasonCount(RenderSurfaceReasonToString(reason), counts[i]);
    }
}
// Source dependency: cc/trees/effect_node.h:35-68 (enum declaration order).
const ALL_RENDER_SURFACE_REASONS: [RenderSurfaceReason; RenderSurfaceReason::kTest as usize] = [
    RenderSurfaceReason::kNone,
    RenderSurfaceReason::kRoot,
    RenderSurfaceReason::k3dTransformFlattening,
    RenderSurfaceReason::kBackdropScope,
    RenderSurfaceReason::kBlendMode,
    RenderSurfaceReason::kBlendModeDstIn,
    RenderSurfaceReason::kOpacity,
    RenderSurfaceReason::kOpacityAnimation,
    RenderSurfaceReason::kFilter,
    RenderSurfaceReason::kFilterAnimation,
    RenderSurfaceReason::kBackdropFilter,
    RenderSurfaceReason::kBackdropFilterAnimation,
    RenderSurfaceReason::kRoundedCorner,
    RenderSurfaceReason::kClipPath,
    RenderSurfaceReason::kClipAxisAlignment,
    RenderSurfaceReason::kMask,
    RenderSurfaceReason::kTrilinearFiltering,
    RenderSurfaceReason::kCache,
    RenderSurfaceReason::kCopyRequest,
    RenderSurfaceReason::kMirrored,
    RenderSurfaceReason::kSubtreeIsBeingCaptured,
    RenderSurfaceReason::kViewTransitionParticipant,
    RenderSurfaceReason::kGradientMask,
    RenderSurfaceReason::k2DScaleTransformWithCompositedDescendants,
    RenderSurfaceReason::kUnboundedElement,
];
// Source: cc/trees/draw_property_utils.cc:1449-1465.
struct VisibleLayerDrawAdapter<'a, C: DrawContext> {
    context: &'a mut C,
    p: &'a mut DrawPropertyTrees<C>,
}

impl<C: DrawContext> compositor_draw::draw_property_walk::VisibleLayerDrawContext
    for VisibleLayerDrawAdapter<'_, C>
{
    type Layer = C::Layer;

    fn compute_draw_transforms(&mut self, layer: Self::Layer) {
        ComputeLayerDrawTransforms::<C>(self.context, layer, self.p);
    }

    fn clear_render_surface_common_ancestor_clip(&mut self, layer: Self::Layer) {
        ClearRenderSurfaceCommonAncestorClip::<C>(self.context, layer, self.p);
    }

    fn compute_draw_opacity(&mut self, layer: Self::Layer) {
        ComputeLayerDrawOpacity::<C>(self.context, layer, self.p);
    }

    fn update_render_surface_common_ancestor_clip(&mut self, layer: Self::Layer) {
        UpdateRenderSurfaceCommonAncestorClip::<C>(self.context, layer, self.p);
    }

    fn compute_clip_and_visible_rect(&mut self, layer: Self::Layer) {
        ComputeLayerClipAndVisibleRect::<C>(self.context, layer, self.p);
    }

    fn compute_drawable_content_rect(&mut self, layer: Self::Layer) {
        ComputeLayerDrawableContentRect::<C>(self.context, layer, self.p);
    }
}

fn ComputeDrawPropertiesOfVisibleLayers<C: DrawContext>(
    context: &mut C,
    list: &[C::Layer],
    p: &mut DrawPropertyTrees<C>,
) {
    let mut adapter = VisibleLayerDrawAdapter { context, p };
    compositor_draw::draw_property_walk::ComputeDrawPropertiesOfVisibleLayers(&mut adapter, list);
}
// Source: cc/trees/draw_property_utils.cc:1469-1479.
#[cfg(debug_assertions)]
fn NodeMayContainBackdropBlurFilter<
    F: DrawFilters,
    P: crate::adapter::property_types::PropertyPath,
>(
    node: &EffectNode<F, P>,
) -> bool {
    compositor_draw::draw_property_policy::MayContainBackdropBlurFilter(node.render_surface_reason)
}
// Source: cc/trees/draw_property_utils.h:82-84; cc/trees/draw_property_utils.cc:1484-1518.
pub fn LayerShouldBeSkippedForDrawPropertiesComputation<C: DrawContext>(
    context: &C,
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    p: &DrawPropertyTrees<C>,
) -> bool {
    let tree = p.effect_tree();
    let id = layer.effect_tree_index();
    assert_ne!(id, kInvalidPropertyNodeId);
    let effect = tree.Node(id);
    let transform = p.transform_tree().Node(layer.transform_tree_index());
    compositor_draw::draw_property_policy::ClassifyLayerSkip(
        effect.HasRenderSurface() && effect.subtree_has_copy_request,
        effect.subtree_hidden,
        effect.cache_render_surface,
        effect.subtree_capture_id.is_valid(),
        transform.node_and_ancestors_are_animated_or_invertible,
        effect.is_drawn,
        context.enable_backface_visibility_interop(),
        layer.should_check_backface_visibility(),
        effect.hidden_by_backface_visibility,
        || IsLayerBackFaceVisible::<C>(context, layer, p, false),
    )
    .is_some()
}
// Source: cc/trees/draw_property_utils.cc:1520-1523.
pub fn IsLayerBackFaceVisibleForTesting<C: DrawContext>(
    context: &C,
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    p: &DrawPropertyTrees<C>,
) -> bool {
    IsLayerBackFaceVisible::<C>(context, layer, p, false)
}
// Source: cc/trees/draw_property_utils.h:36-37; cc/trees/draw_property_utils.cc:1525-1533.
pub fn ConcatInverseSurfaceContentsScale<C: DrawContext>(
    node: &EffectNode<C::Filters, C::Path>,
    transform: &mut C::Transform,
) {
    debug_assert!(node.HasRenderSurface());
    if node.surface_contents_scale.x() != 0.0 && node.surface_contents_scale.y() != 0.0 {
        transform.Scale(
            (1.0_f64 / node.surface_contents_scale.x() as f64) as f32,
            (1.0_f64 / node.surface_contents_scale.y() as f64) as f32,
        );
    }
}
// Source: cc/trees/draw_property_utils.h:54-55; cc/trees/draw_property_utils.cc:1535-1551.
struct LayerUpdateListAdapter<'a, C: DrawContext> {
    context: &'a C,
    p: &'a DrawPropertyTrees<C>,
}

impl<C: DrawContext> compositor_draw::draw_property_walk::LayerUpdateListContext
    for LayerUpdateListAdapter<'_, C>
{
    type Layer = C::Layer;

    fn layer_count(&self) -> usize {
        self.context.layer_count()
    }

    fn layer_at(&self, index: usize) -> Self::Layer {
        self.context.layer_id_at(index)
    }

    fn ensure_valid_property_tree_indices(&self, layer: Self::Layer) {
        self.context
            .layer(layer)
            .EnsureValidPropertyTreeIndices(self.p);
    }

    fn is_root_layer(&self, layer: Self::Layer, main_thread: bool) -> bool {
        IsRootLayer::<C>(self.context, layer, main_thread)
    }

    fn should_skip_layer(&self, layer: Self::Layer, main_thread: bool) -> bool {
        let layer = self.context.layer(layer);
        if main_thread {
            MainLayerShouldBeSkipped::<C>(layer, self.p)
        } else {
            LayerShouldBeSkippedForDrawPropertiesComputation::<C>(self.context, layer, self.p)
        }
    }

    fn layer_needs_update(&self, layer: Self::Layer, main_thread: bool) -> bool {
        let layer = self.context.layer(layer);
        let drawn = self
            .p
            .effect_tree()
            .Node(layer.effect_tree_index())
            .is_drawn;
        LayerNeedsUpdate::<C>(self.context, layer, drawn, self.p, main_thread)
    }
}

pub fn FindMainLayersThatNeedUpdates<C: DrawContext>(
    context: &C,
    p: &DrawPropertyTrees<C>,
    list: &mut Vec<C::Layer>,
) {
    let adapter = LayerUpdateListAdapter { context, p };
    let (updates, _) =
        compositor_draw::draw_property_walk::FindLayersThatNeedUpdates(&adapter, true);
    list.extend(updates);
}
// Source: cc/trees/draw_property_utils.h:54-55; cc/trees/draw_property_utils.cc:1553-1574.
pub fn FindLayersThatNeedUpdates<C: DrawContext>(
    context: &C,
    p: &DrawPropertyTrees<C>,
    list: &mut Vec<C::Layer>,
) {
    let adapter = LayerUpdateListAdapter { context, p };
    let (updates, _) =
        compositor_draw::draw_property_walk::FindLayersThatNeedUpdates(&adapter, false);
    list.extend(updates);
}
// Source: cc/trees/draw_property_utils.cc:1576-1579.
pub fn ComputeTransforms<T: PropertyTreeTransform>(
    tree: &mut TransformTree<T>,
    ids: &ViewportPropertyIds,
    context: &mut TransformTreeContext<'_, T>,
) {
    tree.UpdateAllTransforms(ids, context);
}
// Source: cc/trees/draw_property_utils.h:46-46; cc/trees/draw_property_utils.cc:1581-1588.
pub fn ComputeEffects<C: DrawContext>(
    tree: &mut EffectTree<C::Filters, C::Path, C::Surface, C::CopyRequest>,
    context: &mut EffectTreeContext<'_, C::Transform>,
) {
    let needs_update = tree.needs_update();
    let end = tree.size() as i32;
    let stats = compositor_tree::effect_tree_update::UpdateEffectTree(
        needs_update,
        kContentsRootPropertyNodeId,
        end,
        |id| tree.UpdateEffects(id, context),
    );
    if stats.updated {
        tree.set_needs_update(false);
    }
}
// Source: cc/trees/draw_property_utils.h:48-48; cc/trees/draw_property_utils.cc:1590-1605.
struct PropertyTreeUpdateAdapter<'a, C: DrawContext> {
    context: &'a C,
    p: &'a mut DrawPropertyTrees<C>,
}

impl<C: DrawContext> compositor_tree::property_tree_update::PropertyTreeUpdateContext
    for PropertyTreeUpdateAdapter<'_, C>
{
    fn transform_tree_needs_update(&self) -> bool {
        self.p.transform_tree().needs_update()
    }

    fn mark_clip_tree_needs_update(&mut self) {
        self.p.clip_tree_mutable().set_needs_update(true);
    }

    fn mark_effect_tree_needs_update(&mut self) {
        self.p.effect_tree_mutable().set_needs_update(true);
    }

    fn update_render_targets(&mut self) {
        UpdateRenderTarget::<C>(self.context, self.p);
    }

    fn update_transforms(&mut self) {
        let (tree, mut view) = self.p.transform_tree_with_context();
        ComputeTransforms(tree, self.context.viewport_property_ids(), &mut view);
    }

    fn update_effects(&mut self) {
        let (tree, mut view) = self.p.effect_tree_with_context(self.context);
        ComputeEffects::<C>(tree, &mut view);
    }

    fn update_clips(&mut self) {
        ComputeClips::<C>(self.p);
    }
}

pub fn UpdatePropertyTrees<C: DrawContext>(context: &C, p: &mut DrawPropertyTrees<C>) {
    let mut adapter = PropertyTreeUpdateAdapter::<C> { context, p };
    compositor_tree::property_tree_update::UpdatePropertyTrees(&mut adapter, false);
}
// Source: cc/trees/draw_property_utils.cc:1607-1621.
pub fn UpdatePropertyTreesAndRenderSurfaces<C: DrawContext>(
    context: &C,
    p: &mut DrawPropertyTrees<C>,
) {
    let mut adapter = PropertyTreeUpdateAdapter::<C> { context, p };
    compositor_tree::property_tree_update::UpdatePropertyTrees(&mut adapter, true);
}
// Source: cc/trees/draw_property_utils.h:61-63; cc/trees/draw_property_utils.cc:1623-1636.
pub fn DrawTransform<C: DrawContext>(
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    p: &DrawPropertyTrees<C>,
) -> C::Transform {
    DrawTransformForLayer::<C>(
        layer.offset_to_transform_parent(),
        layer.transform_tree_index(),
        layer.render_target_effect_tree_index(p),
        p,
    )
}
// Scoped native Layer/LayerImpl getter results enter the unique source body.
// Source: cc/trees/draw_property_utils.cc:1623-1636.
pub fn DrawTransformForLayer<C: DrawContext>(
    offset: Vector2dF,
    transform_tree_index: i32,
    render_target_effect_tree_index: i32,
    p: &DrawPropertyTrees<C>,
) -> C::Transform {
    let mut transform = C::Transform::default();
    p.GetToTarget(
        transform_tree_index,
        render_target_effect_tree_index,
        &mut transform,
    );
    transform.Translate(offset.x(), offset.y());
    transform
}
// Source: cc/trees/draw_property_utils.h:65-68; cc/trees/draw_property_utils.cc:1638-1646.
// The C++ Layer and LayerImpl overloads share this generic live-layer entry.
pub fn ScreenSpaceTransform<C: DrawContext>(
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    tree: &TransformTree<C::Transform>,
) -> C::Transform {
    ScreenSpaceTransformInternal::<C>(layer, tree)
}
// Source: cc/trees/draw_property_utils.h:70-71; cc/trees/draw_property_utils.cc:1648-1725.
pub fn LayerVisibleRect<C: DrawContext>(
    layer: &(impl crate::adapter::draw_property::LayerVisibleRectLayer<C::Transform, Types = C::Types>
          + ?Sized),
    content_bounds: crate::adapter::geometry::Size,
    p: Option<&DrawPropertyTrees<C>>,
) -> Rect {
    let content = Rect::new(0, 0, content_bounds.width(), content_bounds.height());
    let Some(p) = p else {
        return content;
    };
    if layer.effect_tree_index() == kInvalidPropertyNodeId
        || layer.clip_tree_index() == kInvalidPropertyNodeId
        || layer.transform_tree_index() == kInvalidPropertyNodeId
    {
        return content;
    }
    let effects = p.effect_tree();
    let mut unbounded = kInvalidPropertyNodeId;
    let mut i = layer.effect_tree_index();
    while i != kInvalidPropertyNodeId {
        let node = effects.Node(i);
        if node.id == kContentsRootPropertyNodeId {
            break;
        }
        if node.render_surface_reason == RenderSurfaceReason::kUnboundedElement {
            unbounded = node.id;
            break;
        }
        i = node.parent_id;
    }
    let effect = effects.Node(layer.effect_tree_index());
    let mut lower = effect.closest_ancestor_with_cached_render_surface_id;
    lower = lower.max(effect.closest_ancestor_with_copy_request_id);
    lower = lower.max(effect.closest_ancestor_being_captured_id);
    lower = lower.max(effect.closest_ancestor_with_shared_element_id);
    if unbounded != kInvalidPropertyNodeId {
        lower = lower.max(unbounded);
    }
    let non_root = lower > kContentsRootPropertyNodeId;
    let clip_root;
    if non_root {
        let clip = ComputeAccumulatedClip::<C>(p, true, layer.clip_tree_index(), lower);
        if !clip.is_clipped {
            return content;
        }
        clip_root = clip.clip_rect;
    } else {
        clip_root = p
            .clip_tree()
            .Node(layer.clip_tree_index())
            .cached_accumulated_rect_in_screen_space;
    }
    let root = if non_root {
        effects.Node(lower)
    } else {
        effects.Node(kContentsRootPropertyNodeId)
    };
    let clip_layer = ComputeTargetRectInLocalSpace::<C>(
        clip_root,
        p,
        root.transform_id,
        layer.transform_tree_index(),
        root.id,
    );
    if !clip_layer.is_clipped {
        return content;
    }
    let mut clip = clip_layer.clip_rect;
    let offset = layer.offset_to_transform_parent();
    clip.Offset(-offset.x(), -offset.y());
    let mut visible = ToEnclosingClipRect(clip);
    visible.Intersect(content);
    visible
}
// Source: cc/trees/draw_property_utils.h:73-75; cc/trees/draw_property_utils.cc:1727-1746.
// A node identity replaces the aliased C++ pointer; the mutation still borrows
// the resident transform node directly and never constructs a node projection.
pub fn UpdatePageScaleFactor<C: DrawContext>(
    p: &mut DrawPropertyTrees<C>,
    node_id: Option<i32>,
    factor: f32,
) {
    if p.transform_tree().page_scale_factor() == factor || node_id.is_none() {
        return;
    }
    p.transform_tree_mutable().set_page_scale_factor(factor);
    let node = p.transform_tree_mutable().MutableNode(node_id.unwrap());
    node.local = C::Transform::default();
    node.local.Scale(factor, factor);
    node.needs_local_transform_update = true;
    let (tree, mut context) = p.transform_tree_with_context();
    tree.set_needs_update(true, &mut context);
}
// Source: cc/trees/draw_property_utils.h:77-80; cc/trees/draw_property_utils.cc:1748-1809.
struct DrawPropertyPipelineAdapter<'a, C: DrawContext> {
    context: &'a mut C,
    p: &'a mut DrawPropertyTrees<C>,
    list: &'a mut Vec<i32>,
}

impl<C: DrawContext> compositor_draw::draw_property_pipeline::DrawPropertyPipelineContext
    for DrawPropertyPipelineAdapter<'_, C>
{
    type Layer = C::Layer;

    fn input_layer_count(&self) -> usize {
        self.context.layer_count()
    }

    fn transform_node_count(&self) -> usize {
        self.p.transform_tree().size()
    }

    fn effect_node_count(&self) -> usize {
        self.p.effect_tree().size()
    }

    fn clip_node_count(&self) -> usize {
        self.p.clip_tree().size()
    }

    fn clear_render_surface_list(&mut self) {
        self.list.clear();
    }

    fn update_page_scale(&mut self) {
        UpdatePageScaleFactor::<C>(
            self.p,
            self.context.PageScaleTransformNodeId(),
            self.context.current_page_scale_factor(),
        );
    }

    fn update_viewport_clip(&mut self) {
        self.p
            .clip_tree_mutable()
            .SetViewportClip(RectF::from(self.context.GetDeviceViewport()));
    }

    fn update_root_transform_and_scale(&mut self) {
        let (tree, mut view) = self.p.transform_tree_with_context();
        tree.SetRootScaleAndTransform(
            self.context.device_scale_factor(),
            &self.context.DrawTransform(),
            &mut view,
        );
        tree.set_external_page_scale_factor(self.context.external_page_scale_factor());
    }

    fn update_property_trees_and_render_surfaces(&mut self) {
        UpdatePropertyTreesAndRenderSurfaces::<C>(self.context, self.p);
    }

    fn find_visible_layers(&mut self) -> Vec<Self::Layer> {
        self.context
            .TraceScope("draw_property_utils::FindLayersThatNeedUpdates", None);
        let mut visible = Vec::new();
        FindLayersThatNeedUpdates::<C>(self.context, self.p, &mut visible);
        visible
    }

    fn compute_visible_layer_properties(&mut self, visible: &[Self::Layer]) {
        self.context.TraceScope(
            "draw_property_utils::ComputeDrawPropertiesOfVisibleLayers",
            Some(visible.len()),
        );
        ComputeDrawPropertiesOfVisibleLayers::<C>(self.context, visible, self.p);
    }

    fn calculate_render_surface_list(&mut self) {
        self.context
            .TraceScope("CalculateRenderSurfaceLayerList", None);
        let max_texture_size = self.context.max_texture_size();
        CalculateRenderSurfaceLayerList::<C>(self.context, self.p, self.list, max_texture_size);
    }

    fn validate_no_double_background_blur(&self) -> bool {
        #[cfg(debug_assertions)]
        if self.context.log_on_ui_double_background_blur() {
            return LogDoubleBackgroundBlur::<C>(self.context, self.p, self.list);
        }
        true
    }

    fn record_render_surface_reasons(&self) {
        RecordRenderSurfaceReasonsForTracing::<C>(self.context, self.p, self.list);
    }

    fn render_surface_count(&self) -> usize {
        self.list.len()
    }

    fn root_render_surface_exists(&self) -> bool {
        self.p
            .effect_tree()
            .GetRenderSurface(kContentsRootPropertyNodeId)
            .is_some()
    }
}

pub fn CalculateDrawProperties<C: DrawContext>(
    context: &mut C,
    p: &mut DrawPropertyTrees<C>,
    list: &mut Vec<i32>,
    output_updates: Option<&mut Vec<C::Layer>>,
) {
    let mut adapter = DrawPropertyPipelineAdapter::<C> {
        context,
        p,
        list,
    };
    let (visible, _) =
        compositor_draw::draw_property_pipeline::CalculateDrawProperties(&mut adapter);
    if let Some(output) = output_updates {
        *output = visible;
    }
}
// Source: cc/trees/draw_property_utils.h:92-93; cc/trees/draw_property_utils.cc:1811-1820.
pub fn RasterScalesApproximatelyEqual(a: Vector2dF, b: Vector2dF) -> bool {
    compositor_draw::draw_property_policy::RasterScalesApproximatelyEqual(a, b)
}
// Source: cc/trees/draw_property_utils.cc:1822-1855.
pub fn PixelAlignmentOffset<T: Transform>(screen: &T, target: &T) -> Option<Vector2dF> {
    compositor_draw::draw_property_policy::PixelAlignmentOffset(screen, target)
}
// Source: cc/trees/draw_property_utils.cc:1858-1895.
#[cfg(debug_assertions)]
pub fn LogDoubleBackgroundBlur<C: DrawContext>(
    context: &C,
    p: &DrawPropertyTrees<C>,
    list: &[i32],
) -> bool {
    let mut rects: Vec<(Option<C::Layer>, Rect)> = Vec::with_capacity(list.len());
    for &id in list {
        let surface = p.effect_tree().GetRenderSurface(id).unwrap();
        let node = p.effect_tree().Node(id);
        if NodeMayContainBackdropBlurFilter(node) {
            if node.backdrop_filters.HasBlurFilter() {
                if !surface.content_rect().IsEmpty() {
                    let layer = context.LayerByElementId(node.element_id);
                    let screen = MathUtil::MapEnclosingClippedRect(
                        &surface.screen_space_transform(),
                        surface.content_rect(),
                    );
                    if let Some((first, _)) = rects.iter().find(|(_, r)| r.Intersects(screen)) {
                        context.LogDoubleBlur(
                            &context.layer(first.expect("first blur layer")).DebugName(),
                            &context.layer(layer.expect("blur layer")).DebugName(),
                        );
                        return false;
                    } else {
                        rects.push((layer, screen));
                    }
                }
            }
        }
    }
    true
}
