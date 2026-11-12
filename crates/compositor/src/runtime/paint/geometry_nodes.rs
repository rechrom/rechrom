//! Thin resident-node reads for the canonical GeometryMapper caches.
use super::{PropertyTreeState, TransformNode};
use crate::{
    adapter::{
        geometry::{Rect, RectF, ToEnclosingRect, Vector2dF},
        paint_geometry::{GeometryMapperNode, GeometryMapperState, TransformCacheNode},
    },
    runtime::{geometry::Transform, types::CpuSkiaBackend},
};
use paint::paint_property_tree;
use std::sync::{Arc, Weak};
fn rect(value: paint::paint_engine::PaintRect) -> Rect {
    ToEnclosingRect(RectF::new(
        value.x as f32,
        value.y as f32,
        value.width as f32,
        value.height as f32,
    ))
}
impl<R: CpuSkiaBackend> TransformCacheNode for TransformNode<R> {
    type Matrix = Transform<R>;
    fn Downgrade(&self) -> Weak<paint_property_tree::TransformPaintPropertyNode> {
        Arc::downgrade(self.as_arc())
    }
    fn Upgrade(weak: &Weak<paint_property_tree::TransformPaintPropertyNode>) -> Self {
        Self::from_arc(weak.upgrade().expect("source live cached node"))
    }
    fn CacheSlot(&self) -> &paint::opaque_paint_cache_slot::OpaquePaintCacheSlot {
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
        crate::adapter::paint_chunks_to_cc_layer::ConversionTransform::IsIdentityOr2dTranslation(
            &self.Matrix(),
        )
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
impl<R: CpuSkiaBackend> GeometryMapperNode for TransformNode<R> {
    fn RawNode(&self) -> &Arc<paint_property_tree::TransformPaintPropertyNode> {
        self.as_arc()
    }
    fn OriginIsOrigin(&self) -> bool {
        self.node().Origin() == [0.; 3]
    }
    fn Matrix(&self) -> Transform<R> {
        Transform::FromColMajor(&self.node().Matrix().values)
    }
    fn LowestCommonAncestor(&self, other: &Self) -> Self {
        Self::from_arc(paint::property_node_geometry::LowestCommonAncestor(
            self.as_arc(),
            other.as_arc(),
        ))
    }
    fn ParentScrollTranslationNode(&self) -> Self {
        let parent = self.Parent().expect("source parent scroll translation");
        crate::third_party::blink::renderer::platform::graphics::paint::geometry_mapper_transform_cache::WithTransformCache(&parent,|cache|cache.nearest_scroll_translation())
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
impl<R: CpuSkiaBackend> GeometryMapperState for PropertyTreeState<R> {
    type Node = TransformNode<R>;
    type Effect = super::EffectNode<R>;
    fn Transform(&self) -> &Self::Node {
        &self.transform_
    }
    fn Clip(&self) -> &Arc<paint_property_tree::ClipPaintPropertyNode> {
        self.clip_.as_arc()
    }
    fn Effect(&self) -> &Self::Effect {
        &self.effect_
    }
    fn SetClip(&mut self, clip: Arc<paint_property_tree::ClipPaintPropertyNode>) {
        self.set_clip(super::ClipNode::from_arc(clip));
    }
    fn SetEffect(&mut self, effect: Self::Effect) {
        self.set_effect(effect);
    }
}
