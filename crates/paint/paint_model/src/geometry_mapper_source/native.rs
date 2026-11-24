//! Native Paint property owners projected into the one source GeometryMapper.
//! Handles clone Arc identities only; caches stay on their original native nodes.
use super::{float_clip_rect::FloatClipRect, matrix::SharedGeometryTransform, traits::*};
use crate::{
    paint_property_tree,
    source_geometry::{Rect, RectF, ToEnclosingRect, Vector2dF},
};
use crate::property_cell::with_access as read;
use std::sync::Arc;

/// The published tree a handle resolves its links through; None follows the
/// Arc links themselves (raw PrePaint nodes, hand-built trees).
pub type NativeTrees = Option<Arc<paint_property_tree::FrozenPropertyTrees>>;

fn resolve_transform<'a>(
    trees: &'a NativeTrees,
    link: &'a Arc<paint_property_tree::TransformPaintPropertyNode>,
) -> &'a Arc<paint_property_tree::TransformPaintPropertyNode> {
    match trees {
        Some(trees) => paint_property_tree::PropertyNodeResolver::transform(&**trees, link),
        None => link,
    }
}
fn resolve_clip<'a>(
    trees: &'a NativeTrees,
    link: &'a Arc<paint_property_tree::ClipPaintPropertyNode>,
) -> &'a Arc<paint_property_tree::ClipPaintPropertyNode> {
    match trees {
        Some(trees) => paint_property_tree::PropertyNodeResolver::clip(&**trees, link),
        None => link,
    }
}
fn resolve_effect<'a>(
    trees: &'a NativeTrees,
    link: &'a Arc<paint_property_tree::EffectPaintPropertyNode>,
) -> &'a Arc<paint_property_tree::EffectPaintPropertyNode> {
    match trees {
        Some(trees) => paint_property_tree::PropertyNodeResolver::effect(&**trees, link),
        None => link,
    }
}

macro_rules! native_node {
    ($name:ident, $native:ident) => {
        #[derive(Clone)]
        pub struct $name(Arc<paint_property_tree::$native>, NativeTrees);
        impl $name {
            pub fn from_arc(node: Arc<paint_property_tree::$native>) -> Self {
                Self(node, None)
            }
            /// A handle whose links resolve through `trees`.
            pub fn from_arc_in(node: Arc<paint_property_tree::$native>, trees: NativeTrees) -> Self {
                Self(node, trees)
            }
            pub fn as_arc(&self) -> &Arc<paint_property_tree::$native> {
                &self.0
            }
            fn node(&self) -> &paint_property_tree::$native {
                &self.0
            }
        }
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.0.lifecycle.same_node(&other.0.lifecycle)
            }
        }
        impl Eq for $name {}
    };
}
native_node!(NativeTransformNode, TransformPaintPropertyNode);
native_node!(NativeEffectNode, EffectPaintPropertyNode);
#[derive(Clone)]
pub struct NativeGeometryState {
    transform_: NativeTransformNode,
    clip_: Arc<paint_property_tree::ClipPaintPropertyNode>,
    effect_: NativeEffectNode,
}
impl NativeGeometryState {
    pub fn from_state(state: &paint_property_tree::PropertyTreeState) -> Self {
        Self::from_state_in(state, None)
    }
    /// A state whose links resolve through `trees`.
    pub fn from_state_in(state: &paint_property_tree::PropertyTreeState, trees: NativeTrees) -> Self {
        Self {
            transform_: NativeTransformNode::from_arc_in(state.transform.clone(), trees.clone()),
            clip_: state.clip.clone(),
            effect_: NativeEffectNode::from_arc_in(state.effect.clone(), trees),
        }
    }
}
/// Feature state supplied by the lifecycle owner, never inferred from geometry.
pub struct NativeGeometryMapperContext {
    pub singular_transform_fix: bool,
    /// The published tree the mapped states' links resolve through.
    pub trees: NativeTrees,
}
fn rect(value: crate::PaintRect) -> Rect {
    ToEnclosingRect(RectF::new(
        value.x as f32,
        value.y as f32,
        value.width as f32,
        value.height as f32,
    ))
}
impl TransformCacheNode for NativeTransformNode {
    type Matrix = SharedGeometryTransform;
    fn NodeArc(&self) -> Arc<paint_property_tree::TransformPaintPropertyNode> {
        self.as_arc().clone()
    }
    fn Linked(&self, node: Arc<paint_property_tree::TransformPaintPropertyNode>) -> Self {
        Self(node, self.1.clone())
    }
    fn CacheSlot(&self) -> &crate::opaque_paint_cache_slot::OpaquePaintCacheSlot {
        &self.node().geometry_mapper_cache
    }
    fn Trees(&self) -> &NativeTrees {
        &self.1
    }
    fn IsRoot(&self) -> bool {
        read(|a| {
            self.node().state(a).parent.is_none()
        })
    }
    fn IsIdentity(&self) -> bool {
        read(|a| {
            self.node().IsIdentity(a)
        })
    }
    fn Parent(&self) -> Option<Self> {
        read(|a| {
            self.node().state(a)
                .parent
                .as_ref()
                .map(|p| Self(resolve_transform(&self.1, p).clone(), self.1.clone()))
        })
    }
    fn RequiresCompositingForStickyPosition(&self) -> bool {
        read(|a| {
            self.node().RequiresCompositingForStickyPosition(a)
        })
    }
    fn RequiresCompositingForAnchorPosition(&self) -> bool {
        read(|a| {
            self.node().RequiresCompositingForAnchorPosition(a)
        })
    }
    fn IsBackfaceHiddenInternal(&self, parent: bool) -> bool {
        read(|a| {
            match self.node().state(a).backface_visibility {
                paint_property_tree::PaintBackfaceVisibility::Inherited => parent,
                paint_property_tree::PaintBackfaceVisibility::Hidden => true,
                paint_property_tree::PaintBackfaceVisibility::Visible => false,
            }
        })
    }
    fn HasScrollNode(&self) -> bool {
        read(|a| {
            self.node().ScrollNode(a).is_some()
        })
    }
    fn ScrollParentScrollTranslation(&self) -> Option<Self> {
        read(|a| {
            self.node().state(a)
                .scroll_parent_scroll_translation
                .as_ref()
                .map(|n| Self(resolve_transform(&self.1, n).clone(), self.1.clone()))
        })
    }
    fn HasDirectCompositingReasons(&self) -> bool {
        read(|a| {
            self.node().HasDirectCompositingReasons(a)
        })
    }
    fn IsIdentityOr2dTranslation(&self) -> bool {
        FloatClipRectTransform::IsIdentityOr2dTranslation(&self.Matrix())
    }
    fn HasActiveTransformAnimation(&self) -> bool {
        read(|a| {
            self.node().HasActiveTransformAnimation(a)
        })
    }
    fn Get2dTranslation(&self) -> Vector2dF {
        let value = self.Matrix().as_source().To2dTranslation();
        Vector2dF::new(value.x(), value.y())
    }
    fn MatrixWithOriginApplied(&self) -> Self::Matrix {
        read(|a| {
            let mut matrix = self.Matrix();
            let origin = self.node().Origin(a);
            matrix.transform_mut().ApplyTransformOrigin(
                origin[0] as f32,
                origin[1] as f32,
                origin[2] as f32,
            );
            matrix
        })
    }
    fn FlattensInheritedTransform(&self) -> bool {
        read(|a| {
            self.node().FlattensInheritedTransform(a)
        })
    }
}
impl GeometryMapperNode for NativeTransformNode {
    fn RawNode(&self) -> &Arc<paint_property_tree::TransformPaintPropertyNode> {
        self.as_arc()
    }
    fn OriginIsOrigin(&self) -> bool {
        read(|a| {
            self.node().Origin(a) == [0.; 3]
        })
    }
    fn Matrix(&self) -> SharedGeometryTransform {
        read(|a| {
            SharedGeometryTransform::FromColMajor(&self.node().Matrix(a).values)
        })
    }
    fn LowestCommonAncestor(&self, other: &Self) -> Self {
        let trees = self.1.clone().or_else(|| other.1.clone());
        let common = crate::property_node_geometry::LowestCommonAncestorBy(
            self.as_arc(),
            other.as_arc(),
            |node| {
                read(|a| node.Parent(a).map(|parent| resolve_transform(&trees, parent).clone()))
            },
        );
        Self(common, trees)
    }
    fn ParentScrollTranslationNode(&self) -> Self {
        let parent = self.Parent().expect("source parent scroll translation");
        super::geometry_mapper_transform_cache::WithTransformCache(&parent, |cache| {
            cache.nearest_scroll_translation()
        })
    }
    fn ScrollContentsRect(&self) -> Rect {
        rect(read(|access| {
            self.node().ScrollNode(access).expect("source ScrollNode").ContentsRect(access)
        }))
    }
    fn ScrollContainerRect(&self) -> Rect {
        rect(read(|access| {
            self.node().ScrollNode(access).expect("source ScrollNode").ContainerRect(access)
        }))
    }
    fn ScrollOverflowClip(&self) -> Option<Arc<paint_property_tree::ClipPaintPropertyNode>> {
        read(|access| {
            self.node()
                .ScrollNode(access)
                .expect("source ScrollNode")
                .OverflowClipNode(access)
                .cloned()
        })
    }
}
impl GeometryMapperState for NativeGeometryState {
    type Node = NativeTransformNode;
    type Effect = NativeEffectNode;
    fn Transform(&self) -> &Self::Node {
        &self.transform_
    }
    fn Clip(&self) -> &Arc<paint_property_tree::ClipPaintPropertyNode> {
        &self.clip_
    }
    fn Effect(&self) -> &Self::Effect {
        &self.effect_
    }
    fn SetClip(&mut self, clip: Arc<paint_property_tree::ClipPaintPropertyNode>) {
        self.clip_ = clip;
    }
    fn SetEffect(&mut self, effect: Self::Effect) {
        self.effect_ = effect;
    }
}

fn float_rect(r: crate::PaintRect) -> RectF {
    RectF::new(r.x as f32, r.y as f32, r.width as f32, r.height as f32)
}
fn clip_rect(r: paint_property_tree::ClipLayoutRect) -> FloatClipRect {
    let mut value = r
        .rect
        .map(|r| FloatClipRect::FromRect(float_rect(r)))
        .unwrap_or_default();
    if r.has_radius {
        value.SetHasRadius();
    }
    value
}
impl GeometryMapperContext for NativeGeometryMapperContext {
    type State = NativeGeometryState;
    fn Trees(&self) -> &NativeTrees {
        &self.trees
    }
    fn SingularTransformFixEnabled(&self) -> bool {
        self.singular_transform_fix
    }
    fn RootState(&self) -> Self::State {
        NativeGeometryState::from_state_in(
            &paint_property_tree::PropertyTreeState::default(),
            self.trees.clone(),
        )
    }
    fn MakeState(
        &self,
        transform: NativeTransformNode,
        clip: Arc<paint_property_tree::ClipPaintPropertyNode>,
        effect: NativeEffectNode,
    ) -> Self::State {
        NativeGeometryState {
            transform_: transform,
            clip_: clip,
            effect_: effect,
        }
    }
    fn ClipLocalTransform(
        &self,
        clip: &Arc<paint_property_tree::ClipPaintPropertyNode>,
    ) -> NativeTransformNode {
        NativeTransformNode::from_arc_in(
            read(|a| resolve_transform(&self.trees, clip.LocalTransformSpace(a)).clone()),
            self.trees.clone(),
        )
    }
    fn ClipParent(
        &self,
        clip: &Arc<paint_property_tree::ClipPaintPropertyNode>,
    ) -> Option<Arc<paint_property_tree::ClipPaintPropertyNode>> {
        read(|a| clip.Parent(a).map(|parent| resolve_clip(&self.trees, parent).clone()))
    }
    fn ClipLowestCommonAncestor(
        &self,
        a: &Arc<paint_property_tree::ClipPaintPropertyNode>,
        b: &Arc<paint_property_tree::ClipPaintPropertyNode>,
    ) -> Arc<paint_property_tree::ClipPaintPropertyNode> {
        crate::property_node_geometry::LowestCommonAncestorBy(a, b, |clip| self.ClipParent(clip))
    }
    fn LayoutClipRectExcludingOverlayScrollbars(
        &self,
        c: &Arc<paint_property_tree::ClipPaintPropertyNode>,
    ) -> FloatClipRect {
        clip_rect(read(|a| c.LayoutClipRectExcludingOverlayScrollbars(a)))
    }
    fn PreciseLayoutClipRect(
        &self,
        c: &Arc<paint_property_tree::ClipPaintPropertyNode>,
    ) -> FloatClipRect {
        clip_rect(read(|a| c.PreciseLayoutClipRect(a)))
    }
    fn LayoutClipRect(&self, c: &Arc<paint_property_tree::ClipPaintPropertyNode>) -> FloatClipRect {
        clip_rect(read(|a| c.LayoutClipRect(a)))
    }
    fn PaintClipRect(&self, c: &Arc<paint_property_tree::ClipPaintPropertyNode>) -> RectF {
        read(|a| c.PaintClipRect(a))
            .map(float_rect)
            .unwrap_or_else(|| *FloatClipRect::default().Rect())
    }
    fn ClipHasPath(&self, c: &Arc<paint_property_tree::ClipPaintPropertyNode>) -> bool {
        read(|a| c.ClipPath(a).is_some())
    }
    // Source: clip_paint_property_node.h:188-217; this is an exact field query,
    // not a geometric inference about whether a CSS animation is running.
    fn IsForCompositeClipPathAnimation(
        &self,
        c: &Arc<paint_property_tree::ClipPaintPropertyNode>,
    ) -> bool {
        read(|a| c.IsForCompositeClipPathAnimation(a))
    }
    fn PixelMovingFilter(
        &self,
        c: &Arc<paint_property_tree::ClipPaintPropertyNode>,
    ) -> NativeEffectNode {
        NativeEffectNode::from_arc_in(
            read(|a| {
                resolve_effect(&self.trees, c.PixelMovingFilter(a).expect("source PixelMovingFilter"))
                    .clone()
            }),
            self.trees.clone(),
        )
    }
    fn FilterLocalTransform(&self, e: &NativeEffectNode) -> NativeTransformNode {
        NativeTransformNode::from_arc_in(
            read(|a| resolve_transform(&self.trees, e.node().LocalTransformSpace(a)).clone()),
            self.trees.clone(),
        )
    }
    fn HasActiveFilterAnimation(&self, e: &NativeEffectNode) -> bool {
        read(|a| e.node().HasActiveFilterAnimation(a))
    }
    fn FilterMapRect(&self, e: &NativeEffectNode, r: Rect) -> Rect {
        let r = read(|a| {
            e.node()
                .state(a)
                .filters
                .canonical()
                .expect("validated PaintInput native filters")
                .MapRect(
                    foundation::gfx::Rect::new(
                        foundation::gfx::Point::new(r.x, r.y),
                        foundation::gfx::Size::new(r.width, r.height),
                    ),
                    None,
                )
        });
        Rect::new(r.x(), r.y(), r.width(), r.height())
    }
    fn SelfOrAncestorParticipatesInViewTransition(&self, e: &NativeEffectNode) -> bool {
        read(|a| e.node().state(a).self_or_ancestor_participates_in_view_transition)
    }
}

#[cfg(test)]
mod published_tree_cache_tests {
    use super::*;
    use crate::geometry_mapper_source::geometry_mapper_transform_cache::{
        ClearTransformCaches, WithTransformCache,
    };
    use paint_property_tree::{
        PaintPropertyKey, PaintPropertyNodeStore, PaintPropertyOwner, PaintPropertyRole,
        PropertyTreeState, TransformPaintPropertyNode,
    };
    use std::collections::BTreeMap;

    fn raw(
        id: u64,
        parent: Arc<TransformPaintPropertyNode>,
        y: f64,
    ) -> Arc<TransformPaintPropertyNode> {
        let mut node = (*PropertyTreeState::default().transform).clone();
        node.lifecycle = Default::default();
        node.id = id;
        node.state_mut().parent = Some(parent);
        node.state_mut().matrix.values[13] = y;
        Arc::new(node)
    }
    fn key(client: u64) -> PaintPropertyKey {
        PaintPropertyKey {
            owner: PaintPropertyOwner::Fragment { client, fragment: 0 },
            role: PaintPropertyRole::Transform,
        }
    }
    fn publish(
        store: &mut PaintPropertyNodeStore,
        keys: &BTreeMap<u64, PaintPropertyKey>,
        prior: &BTreeMap<u64, PaintPropertyKey>,
        states: &mut [PropertyTreeState],
    ) {
        let effects = BTreeMap::new();
        let mut updater = store.updater(keys, prior, &effects);
        for state in states.iter_mut() {
            updater.state(state);
        }
        updater.finish();
        store.invalidate_published();
    }
    fn state(transform: Arc<TransformPaintPropertyNode>) -> PropertyTreeState {
        PropertyTreeState { transform, ..PropertyTreeState::default() }
    }

    /// A scroll republishes a moved ancestor while an unchanged descendant
    /// keeps its snapshot, so consecutive published trees share that
    /// descendant and its one cache slot. A query through the previous tree
    /// (the compositor still maps it) must not leave values computed against
    /// its ancestors for a query through the current tree, nor a link that
    /// dies with the previous tree.
    #[test]
    fn shared_node_cache_is_scoped_to_its_published_tree() {
        let mut store = PaintPropertyNodeStore::default();
        let first_keys = BTreeMap::from([(101, key(1)), (102, key(2))]);
        let root = PropertyTreeState::default().transform;
        let mut first = [state(raw(102, raw(101, root.clone(), 10.0), 0.0))];
        publish(&mut store, &first_keys, &BTreeMap::new(), &mut first);
        let first_child = first[0].transform.clone();
        // The compositor commit of the first tree clears its change markers.
        crate::property_cell::with_access(|a| first_child.ClearChangedToRoot(a, 1));
        let first_trees = Some(store.published_trees());

        // Paint rebuilds the moved parent; the clean child subtree is retained
        // as recorded, still linking the parent's previous snapshot.
        let second_keys = BTreeMap::from([(201, key(1))]);
        let mut second = [state(raw(201, root, 50.0)), first[0].clone()];
        publish(&mut store, &second_keys, &first_keys, &mut second);
        let second_child = second[1].transform.clone();
        let second_trees = Some(store.published_trees());
        assert!(
            Arc::ptr_eq(&first_child, &second_child),
            "the unchanged child keeps one snapshot across both trees"
        );

        ClearTransformCaches();
        let to_root = |trees: &NativeTrees| {
            let node = NativeTransformNode::from_arc_in(second_child.clone(), trees.clone());
            WithTransformCache(&node, |cache| cache.to_2d_translation_root().y())
        };
        assert_eq!(to_root(&first_trees), 10.0);
        drop(first_trees);
        drop(first_child);
        assert_eq!(to_root(&second_trees), 50.0);
    }
}
