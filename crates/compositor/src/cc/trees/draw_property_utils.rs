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
use std::collections::{BTreeMap, BTreeSet};
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
    if !layer_is_drawn {
        return false;
    }
    if !layer.draws_content() {
        return false;
    }
    if layer.bounds(p).IsEmpty()
        && !p
            .effect_tree()
            .Node(layer.effect_tree_index())
            .filters
            .HasReferenceFilter()
    {
        return false;
    }
    if layer.should_check_backface_visibility()
        && IsLayerBackFaceVisible::<C>(context, layer, p, main_thread)
    {
        return false;
    }
    true
}
// Source: cc/trees/draw_property_utils.cc:535-550.
fn MainLayerShouldBeSkipped<C: DrawContext>(
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    p: &DrawPropertyTrees<C>,
) -> bool {
    let effect = p.effect_tree().Node(layer.effect_tree_index());
    if effect.HasRenderSurface() && effect.subtree_has_copy_request {
        return false;
    }
    let node = p.transform_tree().Node(layer.transform_tree_index());
    !node.node_and_ancestors_are_animated_or_invertible || !effect.is_drawn
}
// Source: cc/trees/draw_property_utils.cc:552-560.
fn LayerDrawableContentRect<C: DrawContext>(
    layer: &dyn DrawLayer<C::Transform, Types = C::Types>,
    bounds: Rect,
    clip: Rect,
) -> Rect {
    if layer.is_clipped() {
        let mut rect = bounds;
        rect.Intersect(clip);
        rect
    } else {
        bounds
    }
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
// Source: cc/trees/draw_property_utils.cc:781-856.
fn UpdateRenderTarget<C: DrawContext>(context: &C, p: &mut DrawPropertyTrees<C>) {
    let tree = p.effect_tree_mutable();
    let mut last = kInvalidPropertyNodeId;
    let mut resources = BTreeMap::new();
    for i in kContentsRootPropertyNodeId..tree.size() as i32 {
        {
            let node = tree.MutableNode(i);
            if node.view_transition_element_resource_id.IsValid() {
                assert!(!resources.contains_key(&node.view_transition_element_resource_id));
                resources.insert(node.view_transition_element_resource_id, i);
            }
            node.view_transition_target_id = kInvalidPropertyNodeId;
        }
        let target = if i == kContentsRootPropertyNodeId {
            kContentsRootPropertyNodeId
        } else {
            let parent = tree.parent(tree.Node(i));
            if parent.HasRenderSurface() {
                tree.Node(i).parent_id
            } else {
                parent.target_id
            }
        };
        let node = tree.MutableNode(i);
        node.target_id = target;
        if node.has_potential_backdrop_filter_animation || !node.backdrop_filters.AllowsLCDText() {
            last = node.id;
        }
        node.lcd_text_disallowed_by_backdrop_filter = false;
    }
    if !resources.is_empty() {
        for id in context.layers() {
            let layer = context.layer(id);
            let resource_id = layer.ViewTransitionResourceId();
            if !resource_id.IsValid() {
                continue;
            }
            let Some(&resource_node_id) = resources.get(&resource_id) else {
                continue;
            };
            let layer_node = tree.Node(layer.effect_tree_index());
            let target = if layer_node.HasRenderSurface() {
                layer_node.id
            } else {
                layer_node.target_id
            };
            tree.MutableNode(resource_node_id).view_transition_target_id = target;
        }
    }
    if last == kInvalidPropertyNodeId {
        return;
    }
    let mut current_target = tree.Node(last).target_id;
    for i in (kContentsRootPropertyNodeId..last).rev() {
        let node = tree.MutableNode(i);
        node.lcd_text_disallowed_by_backdrop_filter = current_target <= i;
        if node.id == current_target {
            current_target = kInvalidPropertyNodeId;
        }
        if current_target == kInvalidPropertyNodeId
            && (node.has_potential_backdrop_filter_animation
                || !node.backdrop_filters.AllowsLCDText())
        {
            current_target = node.target_id;
        }
    }
}
// Source: cc/trees/draw_property_utils.cc:858-885.
pub(crate) fn ComputeClips<C: DrawContext>(p: &mut DrawPropertyTrees<C>) {
    debug_assert!(!p.transform_tree().needs_update());
    if !p.clip_tree().needs_update() {
        return;
    }
    for i in kViewportPropertyNodeId..p.clip_tree().size() as i32 {
        p.clip_tree().Node(i).cached_clip_rects.borrow_mut().clear();
        if i == kViewportPropertyNodeId {
            let clip = p.clip_tree().Node(i).clip;
            p.clip_tree_mutable()
                .MutableNode(i)
                .cached_accumulated_rect_in_screen_space = clip;
            continue;
        }
        let node = p.clip_tree().Node(i);
        let mut clip = p
            .clip_tree()
            .parent(node)
            .cached_accumulated_rect_in_screen_space;
        let success = ApplyClipNodeToAccumulatedClip::<C>(
            p,
            true,
            kContentsRootPropertyNodeId,
            kRootPropertyNodeId,
            node,
            &mut clip,
        );
        if success {
            p.clip_tree_mutable()
                .MutableNode(i)
                .cached_accumulated_rect_in_screen_space = clip;
        }
    }
    p.clip_tree_mutable().set_needs_update(false);
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
// Source: cc/trees/draw_property_utils.cc:909-1009.
fn AddSurfaceToRenderSurfaceList<C: DrawContext>(
    context: &C,
    id: i32,
    list: &mut Vec<i32>,
    p: &mut DrawPropertyTrees<C>,
    captures: &mut Vec<i32>,
) {
    let (target, surface_effect_id, vt_target_id, vt_resource, filter_moves_pixels, contributes) = {
        let surface = p.effect_tree().GetRenderSurface(id).unwrap();
        let effect = p.effect_tree().Node(id);
        (
            surface.render_target(p),
            surface.EffectTreeIndex(),
            effect.view_transition_target_id,
            effect.view_transition_element_resource_id,
            effect.filters.HasFilterThatMovesPixels(),
            p.effect_tree().ContributesToDrawnSurface(id),
        )
    };
    let is_root = surface_effect_id == kContentsRootPropertyNodeId;
    if vt_target_id != kInvalidPropertyNodeId {
        assert!(!is_root);
        assert!(vt_resource.IsValid());
        let (vt_effect_id, vt_member) = {
            let vt_target = p
                .effect_tree()
                .GetRenderSurface(vt_target_id)
                .expect("view transition target");
            (
                vt_target.EffectTreeIndex(),
                vt_target.is_render_surface_list_member(),
            )
        };
        if !vt_member {
            AddSurfaceToRenderSurfaceList::<C>(context, vt_effect_id, list, p, captures);
        }
    }
    if !is_root
        && !p
            .effect_tree()
            .GetRenderSurface(target)
            .unwrap()
            .is_render_surface_list_member()
    {
        AddSurfaceToRenderSurfaceList::<C>(context, target, list, p, captures);
    }
    {
        let surface = p
            .effect_tree_mutable()
            .GetRenderSurfaceMutable(id)
            .expect("surface");
        surface.ClearAccumulatedContentRect();
        surface.set_is_render_surface_list_member(true);
        if context.MatchesCaptureViewTransitionToken(vt_resource) {
            surface.set_has_view_transition_capture_contributions(true);
        }
        surface.set_contributes_to_drawn_surface(if is_root { false } else { contributes });
    }
    list.push(surface_effect_id);
    if context.MatchesCaptureViewTransitionToken(vt_resource) {
        captures.push(id);
    }
    ComputeSurfaceDrawProperties::<C>(p, id);
    let allow_skipping = context.AllowUndamagedNonrootRenderPassToSkip();
    let immune = p
        .effect_tree()
        .GetRenderSurface(id)
        .expect("surface")
        .CopyOfOutputRequired(p)
        || filter_moves_pixels
        || allow_skipping
        || p.effect_tree()
            .GetRenderSurface(id)
            .expect("surface")
            .IsUnbounded(p, context.enable_unbounded_element());
    let nearest = if immune {
        Some(id)
    } else if is_root {
        None
    } else {
        p.effect_tree()
            .GetRenderSurface(target)
            .unwrap()
            .nearest_occlusion_immune_ancestor()
    };
    p.effect_tree_mutable()
        .GetRenderSurfaceMutable(id)
        .expect("surface")
        .SetNearestOcclusionImmuneAncestor(nearest);
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
// Source: cc/trees/draw_property_utils.cc:1141-1262.
fn ComputeInitialRenderSurfaceList<C: DrawContext>(
    context: &mut C,
    p: &mut DrawPropertyTrees<C>,
    list: &mut Vec<i32>,
    content_layers: &mut BTreeMap<ViewTransitionElementResourceId, C::Layer>,
) {
    let effect_count = p.effect_tree().size() as i32;
    for i in kContentsRootPropertyNodeId..effect_count {
        if let Some(surface) = p.effect_tree_mutable().GetRenderSurfaceMutable(i) {
            surface.set_is_render_surface_list_member(false);
            surface.set_has_view_transition_capture_contributions(false);
            surface.reset_num_contributors();
        }
    }
    let mut captures = Vec::new();
    let root = p
        .effect_tree()
        .GetRenderSurface(kContentsRootPropertyNodeId)
        .unwrap()
        .EffectTreeIndex();
    AddSurfaceToRenderSurfaceList::<C>(context, root, list, p, &mut captures);
    for index in 0..context.layer_count() {
        let id = context.layer_id_at(index);
        context.layer(id).EnsureValidPropertyTreeIndices(p);
        {
            let layer = context.layer_mut(id);
            layer.set_contributes_to_drawn_render_surface(false);
            layer.set_raster_even_if_not_drawn(false);
        }
        let is_root = context.IsRootLayer(id);
        let (skip_draw, skip_invertibility, transform_id) = {
            let layer = context.layer(id);
            (
                LayerShouldBeSkippedForDrawPropertiesComputation::<C>(context, layer, p),
                SkipForInvertibility::<C>(layer, p),
                layer.transform_tree_index(),
            )
        };
        let skip_layer = !is_root && (skip_draw || skip_invertibility);
        let will_change = transform_id != kInvalidPropertyNodeId
            && p.transform_tree().Node(transform_id).will_change_transform;
        context
            .layer_mut(id)
            .set_raster_even_if_not_drawn((skip_invertibility && !skip_draw) || will_change);
        if skip_layer {
            continue;
        }
        let (effect_id, target_id, resource, needs_update) = {
            let layer = context.layer(id);
            let effect_id = layer.effect_tree_index();
            let drawn = p.effect_tree().Node(effect_id).is_drawn;
            (
                effect_id,
                layer.render_target(p).unwrap(),
                layer.ViewTransitionResourceId(),
                LayerNeedsUpdate::<C>(context, layer, drawn, p, false),
            )
        };
        let _ = effect_id;
        if !needs_update {
            continue;
        }
        let (target_effect_id, target_member) = {
            let target = p.effect_tree().GetRenderSurface(target_id).unwrap();
            (
                target.EffectTreeIndex(),
                target.is_render_surface_list_member(),
            )
        };
        if !target_member {
            AddSurfaceToRenderSurfaceList::<C>(context, target_effect_id, list, p, &mut captures);
        }
        AdjustLayerDrawPropertiesForPixelAlignmentOffset::<C>(context, id, p);
        context
            .layer_mut(id)
            .set_contributes_to_drawn_render_surface(true);
        context.AccumulateContentRectFromContributingLayer(p, target_id, id);
        p.effect_tree_mutable()
            .GetRenderSurfaceMutable(target_id)
            .unwrap()
            .increment_num_contributors();
        if !resource.IsValid() || context.MatchesCaptureViewTransitionToken(resource) {
            continue;
        }
        context
            .layer_mut(id)
            .AsViewTransitionContentLayer()
            .expect("source valid resource requires VT content subtype")
            .SetOriginatingSurfaceContentRect(Rect::default());
        content_layers.insert(resource, id);
    }
    for id in captures {
        assert!(p
            .effect_tree()
            .Node(id)
            .view_transition_element_resource_id
            .IsValid());
        let surface_target = {
            let surface = p.effect_tree().GetRenderSurface(id).unwrap();
            assert!(surface.has_view_transition_capture_contributions());
            surface.render_target(p)
        };
        let mut target_id = surface_target;
        loop {
            let target = p.effect_tree().GetRenderSurface(target_id).unwrap();
            if target.EffectTreeIndex() == root
                || target.has_view_transition_capture_contributions()
            {
                break;
            }
            target_id = target.render_target(p);
        }
        let target_effect = p
            .effect_tree()
            .GetRenderSurface(target_id)
            .unwrap()
            .EffectTreeIndex();
        if p.effect_tree()
            .GetRenderSurface(target_id)
            .unwrap()
            .has_view_transition_capture_contributions()
        {
            let mut intermediate_id = surface_target;
            loop {
                let (effect_id, next) = {
                    let intermediate = p.effect_tree().GetRenderSurface(intermediate_id).unwrap();
                    (
                        intermediate.EffectTreeIndex(),
                        intermediate.render_target(p),
                    )
                };
                if effect_id == target_effect {
                    break;
                }
                p.effect_tree_mutable()
                    .GetRenderSurfaceMutable(intermediate_id)
                    .unwrap()
                    .set_has_view_transition_capture_contributions(true);
                intermediate_id = next;
            }
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
    for &id in list.iter().rev() {
        let is_root = p
            .effect_tree()
            .GetRenderSurface(id)
            .unwrap()
            .EffectTreeIndex()
            == kContentsRootPropertyNodeId;
        if is_root {
            let viewport_clip = p.clip_tree().ViewportClip();
            p.effect_tree_mutable()
                .GetRenderSurfaceMutable(id)
                .unwrap()
                .SetContentRectToViewport(viewport_clip);
            continue;
        }
        context.CalculateContentRectFromAccumulatedContentRect(p, id, max_texture_size);
        let target_id = p
            .effect_tree()
            .GetRenderSurface(id)
            .unwrap()
            .render_target(p);
        debug_assert!(p
            .effect_tree()
            .GetRenderSurface(target_id)
            .unwrap()
            .is_render_surface_list_member());
        context.AccumulateContentRectFromContributingRenderSurface(p, target_id, id);
        p.effect_tree_mutable()
            .GetRenderSurfaceMutable(target_id)
            .unwrap()
            .increment_num_contributors();
        let vt = p.effect_tree().Node(id).view_transition_element_resource_id;
        if !vt.IsValid() {
            continue;
        }
        let rect = {
            let surface = p.effect_tree().GetRenderSurface(id).unwrap();
            let output = if context.MatchesCaptureViewTransitionToken(vt) {
                surface.view_transition_capture_content_rect()
            } else {
                surface.content_rect()
            };
            surface
                .SurfaceScale(p)
                .InverseOrIdentity()
                .MapRectInt(output)
        };
        content_rects.push((vt, RectF::from(rect)));
        if let Some(&layer) = content_layers.get(&vt) {
            context
                .layer_mut(layer)
                .AsViewTransitionContentLayer()
                .expect("source valid resource requires VT content subtype")
                .SetOriginatingSurfaceContentRect(rect);
        }
    }
}
// Source: cc/trees/draw_property_utils.cc:1329-1385.
fn ComputeListOfNonEmptySurfaces<C: DrawContext>(
    context: &mut C,
    p: &mut DrawPropertyTrees<C>,
    initial: &[i32],
    final_list: &mut Vec<i32>,
) {
    let mut removed = BTreeSet::new();
    for &id in initial {
        let (surface_effect, target_id, content_empty, copy_required) = {
            let surface = p.effect_tree().GetRenderSurface(id).unwrap();
            (
                surface.EffectTreeIndex(),
                surface.render_target(p),
                surface.content_rect().IsEmpty(),
                surface.CopyOfOutputRequired(p),
            )
        };
        let root = surface_effect == kContentsRootPropertyNodeId;
        let target_member = p
            .effect_tree()
            .GetRenderSurface(target_id)
            .unwrap()
            .is_render_surface_list_member();
        if !root
            && ((content_empty && !p.effect_tree().Node(id).filters.HasReferenceFilter())
                || (!target_member && !copy_required))
        {
            p.effect_tree_mutable()
                .GetRenderSurfaceMutable(id)
                .unwrap()
                .set_is_render_surface_list_member(false);
            removed.insert(id);
            continue;
        }
        let mut undelete_id = target_id;
        loop {
            let (undelete_effect, next) = {
                let undelete = p.effect_tree().GetRenderSurface(undelete_id).unwrap();
                (undelete.EffectTreeIndex(), undelete.render_target(p))
            };
            if !removed.contains(&undelete_effect) {
                break;
            }
            // The source sets `surface`, not target_to_undelete; preserve that write.
            p.effect_tree_mutable()
                .GetRenderSurfaceMutable(id)
                .unwrap()
                .set_is_render_surface_list_member(true);
            final_list.push(undelete_effect);
            removed.remove(&undelete_effect);
            undelete_id = next;
        }
        final_list.push(id);
    }
    for &id in &removed {
        let target_id = p
            .effect_tree()
            .GetRenderSurface(id)
            .unwrap()
            .render_target(p);
        p.effect_tree_mutable()
            .GetRenderSurfaceMutable(target_id)
            .unwrap()
            .decrement_num_contributors();
    }
    if !removed.is_empty() {
        for index in 0..context.layer_count() {
            let id = context.layer_id_at(index);
            let target_id = {
                let layer = context.layer(id);
                if !layer.contributes_to_drawn_render_surface() {
                    continue;
                }
                layer.render_target(p).unwrap()
            };
            if !p
                .effect_tree()
                .GetRenderSurface(target_id)
                .unwrap()
                .is_render_surface_list_member()
            {
                context
                    .layer_mut(id)
                    .set_contributes_to_drawn_render_surface(false);
                p.effect_tree_mutable()
                    .GetRenderSurfaceMutable(target_id)
                    .unwrap()
                    .decrement_num_contributors();
            }
        }
    }
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
fn ComputeDrawPropertiesOfVisibleLayers<C: DrawContext>(
    context: &mut C,
    list: &[C::Layer],
    p: &mut DrawPropertyTrees<C>,
) {
    for &id in list {
        ComputeLayerDrawTransforms::<C>(context, id, p);
        ClearRenderSurfaceCommonAncestorClip::<C>(context, id, p);
    }
    for &id in list {
        ComputeLayerDrawOpacity::<C>(context, id, p);
        UpdateRenderSurfaceCommonAncestorClip::<C>(context, id, p);
    }
    for &id in list {
        ComputeLayerClipAndVisibleRect::<C>(context, id, p);
    }
    for &id in list {
        ComputeLayerDrawableContentRect::<C>(context, id, p);
    }
}
// Source: cc/trees/draw_property_utils.cc:1469-1479.
#[cfg(debug_assertions)]
fn NodeMayContainBackdropBlurFilter<
    F: DrawFilters,
    P: crate::adapter::property_types::PropertyPath,
>(
    node: &EffectNode<F, P>,
) -> bool {
    matches!(
        node.render_surface_reason,
        RenderSurfaceReason::kMask
            | RenderSurfaceReason::kTrilinearFiltering
            | RenderSurfaceReason::kFilter
            | RenderSurfaceReason::kBackdropFilter
    )
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
    if effect.HasRenderSurface() && effect.subtree_has_copy_request {
        return false;
    }
    if effect.subtree_hidden
        && !effect.cache_render_surface
        && !effect.subtree_capture_id.is_valid()
    {
        return true;
    }
    let transform = p.transform_tree().Node(layer.transform_tree_index());
    if !transform.node_and_ancestors_are_animated_or_invertible || !effect.is_drawn {
        return true;
    }
    if context.enable_backface_visibility_interop() {
        layer.should_check_backface_visibility()
            && IsLayerBackFaceVisible::<C>(context, layer, p, false)
    } else {
        effect.hidden_by_backface_visibility
    }
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
pub fn FindMainLayersThatNeedUpdates<C: DrawContext>(
    context: &C,
    p: &DrawPropertyTrees<C>,
    list: &mut Vec<C::Layer>,
) {
    for id in context.layers() {
        let layer = context.layer(id);
        if !IsRootLayer::<C>(context, id, true) && MainLayerShouldBeSkipped::<C>(layer, p) {
            continue;
        }
        let drawn = p.effect_tree().Node(layer.effect_tree_index()).is_drawn;
        if LayerNeedsUpdate::<C>(context, layer, drawn, p, true) {
            list.push(id);
        }
    }
}
// Source: cc/trees/draw_property_utils.h:54-55; cc/trees/draw_property_utils.cc:1553-1574.
pub fn FindLayersThatNeedUpdates<C: DrawContext>(
    context: &C,
    p: &DrawPropertyTrees<C>,
    list: &mut Vec<C::Layer>,
) {
    for id in context.layers() {
        let layer = context.layer(id);
        layer.EnsureValidPropertyTreeIndices(p);
        if !IsRootLayer::<C>(context, id, false)
            && LayerShouldBeSkippedForDrawPropertiesComputation::<C>(context, layer, p)
        {
            continue;
        }
        let drawn = p.effect_tree().Node(layer.effect_tree_index()).is_drawn;
        if LayerNeedsUpdate::<C>(context, layer, drawn, p, false) {
            list.push(id);
        }
    }
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
    if !tree.needs_update() {
        return;
    }
    for i in kContentsRootPropertyNodeId..tree.size() as i32 {
        tree.UpdateEffects(i, context);
    }
    tree.set_needs_update(false);
}
// Source: cc/trees/draw_property_utils.h:48-48; cc/trees/draw_property_utils.cc:1590-1605.
pub fn UpdatePropertyTrees<C: DrawContext>(context: &C, p: &mut DrawPropertyTrees<C>) {
    if p.transform_tree().needs_update() {
        p.clip_tree_mutable().set_needs_update(true);
        p.effect_tree_mutable().set_needs_update(true);
    }
    let (tree, mut view) = p.transform_tree_with_context();
    ComputeTransforms(tree, context.viewport_property_ids(), &mut view);
    let (tree, mut view) = p.effect_tree_with_context(context);
    ComputeEffects::<C>(tree, &mut view);
    ComputeClips::<C>(p);
}
// Source: cc/trees/draw_property_utils.cc:1607-1621.
pub fn UpdatePropertyTreesAndRenderSurfaces<C: DrawContext>(
    context: &C,
    p: &mut DrawPropertyTrees<C>,
) {
    if p.transform_tree().needs_update() {
        p.clip_tree_mutable().set_needs_update(true);
        p.effect_tree_mutable().set_needs_update(true);
    }
    UpdateRenderTarget::<C>(context, p);
    let (tree, mut view) = p.transform_tree_with_context();
    ComputeTransforms(tree, context.viewport_property_ids(), &mut view);
    let (tree, mut view) = p.effect_tree_with_context(context);
    ComputeEffects::<C>(tree, &mut view);
    ComputeClips::<C>(p);
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
pub fn CalculateDrawProperties<C: DrawContext>(
    context: &mut C,
    p: &mut DrawPropertyTrees<C>,
    list: &mut Vec<i32>,
    output_updates: Option<&mut Vec<C::Layer>>,
) {
    list.clear();
    let mut visible = Vec::new();
    UpdatePageScaleFactor::<C>(
        p,
        context.PageScaleTransformNodeId(),
        context.current_page_scale_factor(),
    );
    p.clip_tree_mutable()
        .SetViewportClip(RectF::from(context.GetDeviceViewport()));
    let (tree, mut view) = p.transform_tree_with_context();
    tree.SetRootScaleAndTransform(
        context.device_scale_factor(),
        &context.DrawTransform(),
        &mut view,
    );
    tree.set_external_page_scale_factor(context.external_page_scale_factor());
    UpdatePropertyTreesAndRenderSurfaces::<C>(context, p);
    {
        context.TraceScope("draw_property_utils::FindLayersThatNeedUpdates", None);
        FindLayersThatNeedUpdates::<C>(context, p, &mut visible);
    }
    {
        context.TraceScope(
            "draw_property_utils::ComputeDrawPropertiesOfVisibleLayers",
            Some(visible.len()),
        );
        ComputeDrawPropertiesOfVisibleLayers::<C>(context, &visible, p);
    }
    {
        context.TraceScope("CalculateRenderSurfaceLayerList", None);
        let max_texture_size = context.max_texture_size();
        CalculateRenderSurfaceLayerList::<C>(context, p, list, max_texture_size);
    }
    #[cfg(debug_assertions)]
    if context.log_on_ui_double_background_blur() {
        debug_assert!(LogDoubleBackgroundBlur::<C>(context, p, list));
    }
    RecordRenderSurfaceReasonsForTracing::<C>(context, p, list);
    debug_assert!(p
        .effect_tree()
        .GetRenderSurface(kContentsRootPropertyNodeId)
        .is_some());
    if let Some(output) = output_updates {
        *output = visible;
    }
}
// Source: cc/trees/draw_property_utils.h:92-93; cc/trees/draw_property_utils.cc:1811-1820.
pub fn RasterScalesApproximatelyEqual(a: Vector2dF, b: Vector2dF) -> bool {
    const THRESHOLD: f32 = 0.001 / 10000.0;
    let difference = a - b;
    difference.x().abs() <= THRESHOLD && difference.y().abs() <= THRESHOLD
}
// Source: cc/trees/draw_property_utils.cc:1822-1855.
pub fn PixelAlignmentOffset<T: Transform>(screen: &T, target: &T) -> Option<Vector2dF> {
    if !screen.IsScaleOrTranslation() || !target.IsScaleOrTranslation() {
        return None;
    }
    if !RasterScalesApproximatelyEqual(screen.To2dScale(), target.To2dScale()) {
        return None;
    }
    let subpixel = |f: f32| f - f.floor();
    let screen_translation = screen.To2dTranslation();
    let screen_x = subpixel(screen_translation.x());
    let screen_y = subpixel(screen_translation.y());
    let draw = target.To2dTranslation();
    let draw_x = subpixel(draw.x());
    let draw_y = subpixel(draw.y());
    const THRESHOLD: f32 = 0.001;
    if (screen_x - draw_x).abs() > THRESHOLD || (screen_y - draw_y).abs() > THRESHOLD {
        return None;
    }
    Some(Vector2dF::new(screen_x, screen_y))
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
