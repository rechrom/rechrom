//! Chromium Paint property-node geometry queries. This module never stores a
//! second ancestry index; it borrows and retains only the actual node Arcs.
use std::sync::Arc;

use crate::paint_property_tree::{LinkedProperties, PropertyNodeResolver};

// Source: transform_paint_property_node.cc:136-149. The producer's immutable
// revisions share the actual node identity; no graph/cache is reconstructed.
pub fn TransformChanged(
    node: &crate::paint_property_tree::TransformPaintPropertyNode,
    change: crate::paint_property_tree::PaintPropertyChangeType,
    relative: &crate::paint_property_tree::TransformPaintPropertyNode,
) -> bool {
    TransformChangedIn(&LinkedProperties, node, change, relative)
}
/// TransformChanged following links through `resolver`.
pub fn TransformChangedIn<R: PropertyNodeResolver + ?Sized>(
    resolver: &R,
    node: &crate::paint_property_tree::TransformPaintPropertyNode,
    change: crate::paint_property_tree::PaintPropertyChangeType,
    relative: &crate::paint_property_tree::TransformPaintPropertyNode,
) -> bool {
    let mut current = Some(node);
    while let Some(node) = current {
        if node.lifecycle.same_node(&relative.lifecycle) {
            return false;
        }
        if node.lifecycle.node_changed() >= change {
            return true;
        }
        current = node.parent.as_ref().map(|parent| &**resolver.transform(parent));
    }
    TransformChangedIn(
        resolver,
        relative,
        change,
        &crate::paint_property_tree::PropertyTreeState::default().transform,
    )
}
// Source: clip_paint_property_node.cc:47-69. Chromium producer nodes are unaliased;
// a cache/value revision still retains exactly the same source node identity.
pub fn ClipChanged(
    node: &crate::paint_property_tree::ClipPaintPropertyNode,
    change: crate::paint_property_tree::PaintPropertyChangeType,
    relative: &crate::paint_property_tree::PropertyTreeState,
    transform_not_to_check: Option<&crate::paint_property_tree::TransformPaintPropertyNode>,
) -> bool {
    ClipChangedIn(&LinkedProperties, node, change, relative, transform_not_to_check)
}
/// ClipChanged following links through `resolver`.
pub fn ClipChangedIn<R: PropertyNodeResolver + ?Sized>(
    resolver: &R,
    node: &crate::paint_property_tree::ClipPaintPropertyNode,
    change: crate::paint_property_tree::PaintPropertyChangeType,
    relative: &crate::paint_property_tree::PropertyTreeState,
    transform_not_to_check: Option<&crate::paint_property_tree::TransformPaintPropertyNode>,
) -> bool {
    let mut current = Some(node);
    while let Some(node) = current {
        if node.lifecycle.same_node(&relative.clip.lifecycle) {
            break;
        }
        if node.lifecycle.node_changed() >= change {
            return true;
        }
        let transform = resolver.transform(&node.local_transform_space);
        if !transform_not_to_check
            .is_some_and(|other| transform.lifecycle.same_node(&other.lifecycle))
            && TransformChangedIn(resolver, transform, change, &relative.transform)
        {
            return true;
        }
        current = node.parent.as_ref().map(|parent| &**resolver.clip(parent));
    }
    false
}
// Source: effect_paint_property_node.cc:148-179.
pub fn EffectChanged(
    node: &crate::paint_property_tree::EffectPaintPropertyNode,
    change: crate::paint_property_tree::PaintPropertyChangeType,
    relative: &crate::paint_property_tree::PropertyTreeState,
    transform_not_to_check: Option<&crate::paint_property_tree::TransformPaintPropertyNode>,
) -> bool {
    EffectChangedIn(&LinkedProperties, node, change, relative, transform_not_to_check)
}
/// EffectChanged following links through `resolver`.
pub fn EffectChangedIn<R: PropertyNodeResolver + ?Sized>(
    resolver: &R,
    node: &crate::paint_property_tree::EffectPaintPropertyNode,
    change: crate::paint_property_tree::PaintPropertyChangeType,
    relative: &crate::paint_property_tree::PropertyTreeState,
    transform_not_to_check: Option<&crate::paint_property_tree::TransformPaintPropertyNode>,
) -> bool {
    let mut current = Some(node);
    while let Some(node) = current {
        if node.lifecycle.same_node(&relative.effect.lifecycle) {
            break;
        }
        if node.lifecycle.node_changed() >= change {
            return true;
        }
        let transform = resolver.transform(&node.local_transform_space);
        if node.HasFilterThatMovesPixels()
            && !transform_not_to_check
                .is_some_and(|other| transform.lifecycle.same_node(&other.lifecycle))
            && TransformChangedIn(resolver, transform, change, &relative.transform)
        {
            return true;
        }
        current = node.parent.as_ref().map(|parent| &**resolver.effect(parent));
    }
    false
}
// Source: property_tree_state.h:288-301.
pub fn PropertyStateChanged(
    node: &crate::paint_property_tree::PropertyTreeState,
    change: crate::paint_property_tree::PaintPropertyChangeType,
    relative: &crate::paint_property_tree::PropertyTreeState,
) -> bool {
    PropertyStateChangedIn(&LinkedProperties, node, change, relative)
}
/// PropertyStateChanged following links through `resolver`.
pub fn PropertyStateChangedIn<R: PropertyNodeResolver + ?Sized>(
    resolver: &R,
    node: &crate::paint_property_tree::PropertyTreeState,
    change: crate::paint_property_tree::PaintPropertyChangeType,
    relative: &crate::paint_property_tree::PropertyTreeState,
) -> bool {
    TransformChangedIn(resolver, &node.transform, change, &relative.transform)
        || ClipChangedIn(resolver, &node.clip, change, relative, Some(&node.transform))
        || EffectChangedIn(resolver, &node.effect, change, relative, Some(&node.transform))
}
pub trait PaintPropertyNodeGeometry: Sized {
    fn Parent(&self) -> Option<&Arc<Self>>;
}
impl PaintPropertyNodeGeometry for crate::paint_property_tree::TransformPaintPropertyNode {
    fn Parent(&self) -> Option<&Arc<Self>> {
        self.parent.as_ref()
    }
}
impl PaintPropertyNodeGeometry for crate::paint_property_tree::ClipPaintPropertyNode {
    fn Parent(&self) -> Option<&Arc<Self>> {
        self.parent.as_ref()
    }
}
impl PaintPropertyNodeGeometry for crate::paint_property_tree::EffectPaintPropertyNode {
    fn Parent(&self) -> Option<&Arc<Self>> {
        self.parent.as_ref()
    }
}
// Source: third_party/blink/renderer/platform/graphics/paint/paint_property_node.cc:16-26.
fn NodeDepthOrFoundAncestor<N: PaintPropertyNodeGeometry>(node: &Arc<N>, ancestor: &Arc<N>) -> i32 {
    let mut depth = 0;
    let mut n = Some(node);
    while let Some(current) = n {
        if Arc::ptr_eq(current, ancestor) {
            return -1;
        }
        depth += 1;
        n = current.Parent();
    }
    depth
}
/// LowestCommonAncestor following parents through `parent` (a resolver),
/// for links whose snapshot may be older than the node's current one.
pub fn LowestCommonAncestorBy<N>(
    node: &Arc<N>,
    other: &Arc<N>,
    parent: impl Fn(&Arc<N>) -> Option<Arc<N>>,
) -> Arc<N> {
    let depth_or_found = |from: &Arc<N>, ancestor: &Arc<N>| -> i32 {
        let mut depth = 0;
        let mut current = Some(from.clone());
        while let Some(node) = current {
            if Arc::ptr_eq(&node, ancestor) {
                return -1;
            }
            depth += 1;
            current = parent(&node);
        }
        depth
    };
    let mut depth_a = depth_or_found(node, other);
    if depth_a == -1 {
        return other.clone();
    }
    let mut depth_b = depth_or_found(other, node);
    if depth_b == -1 {
        return node.clone();
    }
    let mut a = node.clone();
    let mut b = other.clone();
    if depth_a < depth_b {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut depth_a, &mut depth_b);
    }
    while depth_a > depth_b {
        a = parent(&a).expect("source common-root tree");
        depth_a -= 1;
    }
    while !Arc::ptr_eq(&a, &b) {
        a = parent(&a).expect("source nonnull common ancestor");
        b = parent(&b).expect("source nonnull common ancestor");
    }
    a
}
// Source: third_party/blink/renderer/platform/graphics/paint/paint_property_node.h:171-183;
// Source: third_party/blink/renderer/platform/graphics/paint/paint_property_node.cc:28-65.
pub fn LowestCommonAncestor<N: PaintPropertyNodeGeometry>(node: &Arc<N>, other: &Arc<N>) -> Arc<N> {
    if Arc::ptr_eq(node, other)
        || node.Parent().is_none()
        || other.Parent().is_some_and(|p| Arc::ptr_eq(p, node))
    {
        debug_assert_eq!(NodeDepthOrFoundAncestor(other, node), -1);
        return node.clone();
    }
    if other.Parent().is_none() || node.Parent().is_some_and(|p| Arc::ptr_eq(p, other)) {
        debug_assert_eq!(NodeDepthOrFoundAncestor(node, other), -1);
        return other.clone();
    }
    let mut depth_a = NodeDepthOrFoundAncestor(node, other);
    if depth_a == -1 {
        return other.clone();
    }
    let mut depth_b = NodeDepthOrFoundAncestor(other, node);
    if depth_b == -1 {
        return node.clone();
    }
    let mut a = node;
    let mut b = other;
    if depth_a < depth_b {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut depth_a, &mut depth_b);
    }
    while depth_a > depth_b {
        a = a.Parent().expect("source common-root tree");
        depth_a -= 1;
    }
    while !Arc::ptr_eq(a, b) {
        a = a.Parent().expect("source nonnull common ancestor");
        b = b.Parent().expect("source nonnull common ancestor");
    }
    a.clone()
}
