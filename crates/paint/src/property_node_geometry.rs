//! Native Paint property-node geometry queries. This module never stores a
//! second ancestry index; it borrows and retains only the actual node Arcs.
use std::sync::Arc;

// Source: transform_paint_property_node.cc:136-149. The producer's immutable
// revisions share the actual node identity; no graph/cache is reconstructed.
pub fn TransformChanged(
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
        current = node.parent.as_deref();
    }
    TransformChanged(
        relative,
        change,
        &crate::paint_property_tree::PropertyTreeState::default().transform,
    )
}
// Source: clip_paint_property_node.cc:47-69. Native producer nodes are unaliased;
// a cache/value revision still retains exactly the same source node identity.
pub fn ClipChanged(
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
        let transform = &node.local_transform_space;
        if !transform_not_to_check
            .is_some_and(|other| transform.lifecycle.same_node(&other.lifecycle))
            && TransformChanged(transform, change, &relative.transform)
        {
            return true;
        }
        current = node.parent.as_deref();
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
    let mut current = Some(node);
    while let Some(node) = current {
        if node.lifecycle.same_node(&relative.effect.lifecycle) {
            break;
        }
        if node.lifecycle.node_changed() >= change {
            return true;
        }
        let transform = &node.local_transform_space;
        if node
            .filters
            .canonical()
            .expect("validated PaintInput filters")
            .HasFilterThatMovesPixels()
            && !transform_not_to_check
                .is_some_and(|other| transform.lifecycle.same_node(&other.lifecycle))
            && TransformChanged(transform, change, &relative.transform)
        {
            return true;
        }
        current = node.parent.as_deref();
    }
    false
}
// Source: property_tree_state.h:288-301.
pub fn PropertyStateChanged(
    node: &crate::paint_property_tree::PropertyTreeState,
    change: crate::paint_property_tree::PaintPropertyChangeType,
    relative: &crate::paint_property_tree::PropertyTreeState,
) -> bool {
    TransformChanged(&node.transform, change, &relative.transform)
        || ClipChanged(&node.clip, change, relative, Some(&node.transform))
        || EffectChanged(&node.effect, change, relative, Some(&node.transform))
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
