//! Native Paint property owners projected into the one source GeometryMapper.
//! Handles clone Arc identities only; caches stay on their original native nodes.
use super::{float_clip_rect::FloatClipRect, matrix::SharedGeometryTransform, traits::*};
use crate::{
    paint_property_tree,
    source_geometry::{Rect, RectF, ToEnclosingRect, Vector2dF},
};
use std::sync::{Arc, Weak};

macro_rules! native_node {
    ($name:ident, $native:ident) => {
        #[derive(Clone)]
        pub struct $name(Arc<paint_property_tree::$native>);
        impl $name {
            pub fn from_arc(node: Arc<paint_property_tree::$native>) -> Self {
                Self(node)
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
        Self {
            transform_: NativeTransformNode::from_arc(state.transform.clone()),
            clip_: state.clip.clone(),
            effect_: NativeEffectNode::from_arc(state.effect.clone()),
        }
    }
}
/// Feature state supplied by the lifecycle owner, never inferred from geometry.
pub struct NativeGeometryMapperContext {
    pub singular_transform_fix: bool,
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
    fn Downgrade(&self) -> Weak<paint_property_tree::TransformPaintPropertyNode> {
        Arc::downgrade(self.as_arc())
    }
    fn Upgrade(weak: &Weak<paint_property_tree::TransformPaintPropertyNode>) -> Self {
        Self::from_arc(weak.upgrade().expect("source live cached node"))
    }
    fn CacheSlot(&self) -> &crate::opaque_paint_cache_slot::OpaquePaintCacheSlot {
        &self.node().geometry_mapper_cache
    }
    fn IsRoot(&self) -> bool {
        self.node().parent.is_none()
    }
    fn IsIdentity(&self) -> bool {
        self.node().IsIdentity()
    }
    fn Parent(&self) -> Option<Self> {
        self.node()
            .parent
            .as_ref()
            .map(|p| Self::from_arc(p.clone()))
    }
    fn RequiresCompositingForStickyPosition(&self) -> bool {
        self.node().RequiresCompositingForStickyPosition()
    }
    fn RequiresCompositingForAnchorPosition(&self) -> bool {
        self.node().RequiresCompositingForAnchorPosition()
    }
    fn IsBackfaceHiddenInternal(&self, parent: bool) -> bool {
        match self.node().backface_visibility {
            paint_property_tree::PaintBackfaceVisibility::Inherited => parent,
            paint_property_tree::PaintBackfaceVisibility::Hidden => true,
            paint_property_tree::PaintBackfaceVisibility::Visible => false,
        }
    }
    fn HasScrollNode(&self) -> bool {
        self.node().ScrollNode().is_some()
    }
    fn ScrollParentScrollTranslation(&self) -> Option<Self> {
        self.node()
            .scroll_parent_scroll_translation
            .as_ref()
            .map(|n| Self::from_arc(n.clone()))
    }
    fn HasDirectCompositingReasons(&self) -> bool {
        self.node().HasDirectCompositingReasons()
    }
    fn IsIdentityOr2dTranslation(&self) -> bool {
        FloatClipRectTransform::IsIdentityOr2dTranslation(&self.Matrix())
    }
    fn HasActiveTransformAnimation(&self) -> bool {
        self.node().HasActiveTransformAnimation()
    }
    fn Get2dTranslation(&self) -> Vector2dF {
        let value = self.Matrix().as_source().To2dTranslation();
        Vector2dF::new(value.x(), value.y())
    }
    fn MatrixWithOriginApplied(&self) -> Self::Matrix {
        let mut matrix = self.Matrix();
        let origin = self.node().Origin();
        matrix.transform_mut().ApplyTransformOrigin(
            origin[0] as f32,
            origin[1] as f32,
            origin[2] as f32,
        );
        matrix
    }
    fn FlattensInheritedTransform(&self) -> bool {
        self.node().FlattensInheritedTransform()
    }
}
impl GeometryMapperNode for NativeTransformNode {
    fn RawNode(&self) -> &Arc<paint_property_tree::TransformPaintPropertyNode> {
        self.as_arc()
    }
    fn OriginIsOrigin(&self) -> bool {
        self.node().Origin() == [0.; 3]
    }
    fn Matrix(&self) -> SharedGeometryTransform {
        SharedGeometryTransform::FromColMajor(&self.node().Matrix().values)
    }
    fn LowestCommonAncestor(&self, other: &Self) -> Self {
        Self::from_arc(crate::property_node_geometry::LowestCommonAncestor(
            self.as_arc(),
            other.as_arc(),
        ))
    }
    fn ParentScrollTranslationNode(&self) -> Self {
        let parent = self.Parent().expect("source parent scroll translation");
        super::geometry_mapper_transform_cache::WithTransformCache(&parent, |cache| {
            cache.nearest_scroll_translation()
        })
    }
    fn ScrollContentsRect(&self) -> Rect {
        rect(
            self.node()
                .scroll
                .as_ref()
                .expect("source ScrollNode")
                .ContentsRect(),
        )
    }
    fn ScrollContainerRect(&self) -> Rect {
        rect(
            self.node()
                .scroll
                .as_ref()
                .expect("source ScrollNode")
                .ContainerRect(),
        )
    }
    fn ScrollOverflowClip(&self) -> Option<Arc<paint_property_tree::ClipPaintPropertyNode>> {
        self.node()
            .scroll
            .as_ref()
            .expect("source ScrollNode")
            .OverflowClipNode()
            .cloned()
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
    fn SingularTransformFixEnabled(&self) -> bool {
        self.singular_transform_fix
    }
    fn RootState(&self) -> Self::State {
        NativeGeometryState::from_state(&paint_property_tree::PropertyTreeState::default())
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
        NativeTransformNode::from_arc(clip.LocalTransformSpace().clone())
    }
    fn ClipLowestCommonAncestor(
        &self,
        a: &Arc<paint_property_tree::ClipPaintPropertyNode>,
        b: &Arc<paint_property_tree::ClipPaintPropertyNode>,
    ) -> Arc<paint_property_tree::ClipPaintPropertyNode> {
        crate::property_node_geometry::LowestCommonAncestor(a, b)
    }
    fn LayoutClipRectExcludingOverlayScrollbars(
        &self,
        c: &Arc<paint_property_tree::ClipPaintPropertyNode>,
    ) -> FloatClipRect {
        clip_rect(c.LayoutClipRectExcludingOverlayScrollbars())
    }
    fn PreciseLayoutClipRect(
        &self,
        c: &Arc<paint_property_tree::ClipPaintPropertyNode>,
    ) -> FloatClipRect {
        clip_rect(c.PreciseLayoutClipRect())
    }
    fn LayoutClipRect(&self, c: &Arc<paint_property_tree::ClipPaintPropertyNode>) -> FloatClipRect {
        clip_rect(c.LayoutClipRect())
    }
    fn PaintClipRect(&self, c: &Arc<paint_property_tree::ClipPaintPropertyNode>) -> RectF {
        c.PaintClipRect()
            .map(float_rect)
            .unwrap_or_else(|| *FloatClipRect::default().Rect())
    }
    fn ClipHasPath(&self, c: &Arc<paint_property_tree::ClipPaintPropertyNode>) -> bool {
        c.ClipPath().is_some()
    }
    // Source: clip_paint_property_node.h:188-217; this is an exact field query,
    // not a geometric inference about whether a CSS animation is running.
    fn IsForCompositeClipPathAnimation(
        &self,
        c: &Arc<paint_property_tree::ClipPaintPropertyNode>,
    ) -> bool {
        c.IsForCompositeClipPathAnimation()
    }
    fn PixelMovingFilter(
        &self,
        c: &Arc<paint_property_tree::ClipPaintPropertyNode>,
    ) -> NativeEffectNode {
        NativeEffectNode::from_arc(
            c.PixelMovingFilter()
                .expect("source PixelMovingFilter")
                .clone(),
        )
    }
    fn FilterLocalTransform(&self, e: &NativeEffectNode) -> NativeTransformNode {
        NativeTransformNode::from_arc(e.node().local_transform_space.clone())
    }
    fn HasActiveFilterAnimation(&self, e: &NativeEffectNode) -> bool {
        e.node()
            .direct_compositing_reasons
            .Has(foundation::CompositingReason::kActiveFilterAnimation)
    }
    fn FilterMapRect(&self, e: &NativeEffectNode, r: Rect) -> Rect {
        let r = e
            .node()
            .filters
            .canonical()
            .expect("validated PaintInput native filters")
            .MapRect(
                foundation::gfx::Rect::new(
                    foundation::gfx::Point::new(r.x, r.y),
                    foundation::gfx::Size::new(r.width, r.height),
                ),
                None,
            );
        Rect::new(r.x(), r.y(), r.width(), r.height())
    }
    fn SelfOrAncestorParticipatesInViewTransition(&self, e: &NativeEffectNode) -> bool {
        e.node().self_or_ancestor_participates_in_view_transition
    }
}
