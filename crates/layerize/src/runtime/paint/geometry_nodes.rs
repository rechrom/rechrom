//! Thin resident-node reads for the canonical GeometryMapper caches.
use super::{read, PropertyTreeState, TransformNode};
use crate::{
    adapter::{
        geometry::{Rect, RectF, ToEnclosingRect, Vector2dF},
        paint_geometry::{GeometryMapperNode, GeometryMapperState, TransformCacheNode},
    },
    runtime::{geometry::Transform, types::PaintSkiaBackend},
};
use paint::paint_property_tree;
use std::sync::Arc;
fn rect(value: paint::paint_engine::PaintRect) -> Rect {
    ToEnclosingRect(RectF::new(
        value.x as f32,
        value.y as f32,
        value.width as f32,
        value.height as f32,
    ))
}
impl<R: PaintSkiaBackend> TransformCacheNode for TransformNode<R> {
    type Matrix = Transform<R>;
    fn NodeArc(&self) -> Arc<paint_property_tree::TransformPaintPropertyNode> {
        self.as_arc().clone()
    }
    fn Linked(&self, node: Arc<paint_property_tree::TransformPaintPropertyNode>) -> Self {
        Self::from_arc_in(node, self.trees().clone())
    }
    fn CacheSlot(&self) -> &paint::opaque_paint_cache_slot::OpaquePaintCacheSlot {
        &self.node().geometry_mapper_cache
    }
    fn Trees(&self) -> &super::HandleTrees {
        self.trees()
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
        self.parent_handle()
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
                .map(|n| self.link_same(n))
        })
    }
    fn HasDirectCompositingReasons(&self) -> bool {
        read(|a| {
            self.node().HasDirectCompositingReasons(a)
        })
    }
    fn IsIdentityOr2dTranslation(&self) -> bool {
        crate::adapter::paint_chunks_to_cc_layer::ConversionTransform::IsIdentityOr2dTranslation(
            &self.Matrix(),
        )
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
impl<R: PaintSkiaBackend> GeometryMapperNode for TransformNode<R> {
    fn RawNode(&self) -> &Arc<paint_property_tree::TransformPaintPropertyNode> {
        self.as_arc()
    }
    fn OriginIsOrigin(&self) -> bool {
        read(|a| {
            self.node().Origin(a) == [0.; 3]
        })
    }
    fn Matrix(&self) -> Transform<R> {
        read(|a| {
            Transform::FromColMajor(&self.node().Matrix(a).values)
        })
    }
    fn LowestCommonAncestor(&self, other: &Self) -> Self {
        self.lowest_common_ancestor(other)
    }
    fn ParentScrollTranslationNode(&self) -> Self {
        let parent = self.Parent().expect("source parent scroll translation");
        crate::third_party::blink::renderer::platform::graphics::paint::geometry_mapper_transform_cache::WithTransformCache(&parent,|cache|cache.nearest_scroll_translation())
    }
    fn ScrollContentsRect(&self) -> Rect {
        rect(read(|a| self.node().ScrollNode(a).expect("source ScrollNode").ContentsRect(a)))
    }
    fn ScrollContainerRect(&self) -> Rect {
        rect(read(|a| self.node().ScrollNode(a).expect("source ScrollNode").ContainerRect(a)))
    }
    fn ScrollOverflowClip(&self) -> Option<Arc<paint_property_tree::ClipPaintPropertyNode>> {
        read(|a| {
            self.node()
                .ScrollNode(a)
                .expect("source ScrollNode")
                .OverflowClipNode(a)
                .cloned()
        })
    }
}
impl<R: PaintSkiaBackend> GeometryMapperState for PropertyTreeState<R> {
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
        let trees = self.transform_.trees().clone();
        self.set_clip(super::ClipNode::from_arc_in(clip, trees));
    }
    fn SetEffect(&mut self, effect: Self::Effect) {
        self.set_effect(effect);
    }
}
