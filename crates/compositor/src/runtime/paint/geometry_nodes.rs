//! Thin resident-node reads for the canonical GeometryMapper caches.
use super::{PropertyTreeState, TransformNode};
use crate::{
    adapter::{
        geometry::{Rect, RectF, ToEnclosingRect, Vector2dF},
        paint_geometry::{GeometryMapperNode, GeometryMapperState, TransformCacheNode},
    },
    runtime::{geometry::Transform, types::CpuSkiaBackend},
};
use paint::paint_property_tree as native;
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
    type Weak = Weak<native::TransformPaintPropertyNode>;
    fn Downgrade(&self) -> Self::Weak {
        Arc::downgrade(self.as_arc())
    }
    fn Upgrade(weak: &Self::Weak) -> Self {
        Self::from_arc(weak.upgrade().expect("source live cached node"))
    }
    fn CacheSlot(&self) -> &paint::opaque_paint_cache_slot::OpaquePaintCacheSlot {
        &self.node().geometry_mapper_cache
    }
    fn IsRoot(&self) -> bool {
        self.node().parent.is_none()
    }
    fn IsIdentity(&self) -> bool {
        self.node().matrix.values == native::PropertyTreeState::default().transform.matrix.values
    }
    fn Parent(&self) -> Option<Self> {
        self.node()
            .parent
            .as_ref()
            .map(|p| Self::from_arc(p.clone()))
    }
    fn RequiresCompositingForStickyPosition(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"StickyPosition")
    }
    fn RequiresCompositingForAnchorPosition(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"AnchorPosition")
    }
    fn IsBackfaceHiddenInternal(&self, parent: bool) -> bool {
        match self.node().backface_visibility {
            native::PaintBackfaceVisibility::Inherited => parent,
            native::PaintBackfaceVisibility::Hidden => true,
            native::PaintBackfaceVisibility::Visible => false,
        }
    }
    fn HasScrollNode(&self) -> bool {
        self.node().scroll.is_some()
    }
    fn ScrollParentScrollTranslation(&self) -> Option<Self> {
        self.node()
            .scroll_parent_scroll_translation
            .as_ref()
            .map(|n| Self::from_arc(n.clone()))
    }
    fn HasDirectCompositingReasons(&self) -> bool {
        !self.node().direct_compositing_reasons.is_empty()
    }
    fn IsIdentityOr2dTranslation(&self) -> bool {
        crate::adapter::paint_chunks_to_cc_layer::ConversionTransform::IsIdentityOr2dTranslation(
            &self.Matrix(),
        )
    }
    fn HasActiveTransformAnimation(&self) -> bool {
        self.node().direct_compositing_reasons.iter().any(|r| {
            matches!(
                *r,
                "ActiveTransformAnimation"
                    | "ActiveScaleAnimation"
                    | "ActiveRotateAnimation"
                    | "ActiveTranslateAnimation"
            )
        })
    }
    fn Get2dTranslation(&self) -> Vector2dF {
        let value = self.Matrix().native().To2dTranslation();
        Vector2dF::new(value.x(), value.y())
    }
    fn MatrixWithOriginApplied(&self) -> Self::Matrix {
        let mut matrix = self.Matrix();
        let origin = self.node().origin;
        matrix.native_mut().ApplyTransformOrigin(
            origin[0] as f32,
            origin[1] as f32,
            origin[2] as f32,
        );
        matrix
    }
    fn FlattensInheritedTransform(&self) -> bool {
        self.node().flattens_inherited_transform
    }
}
impl<R: CpuSkiaBackend> GeometryMapperNode for TransformNode<R> {
    fn RawNode(&self) -> &Arc<native::TransformPaintPropertyNode> {
        self.as_arc()
    }
    fn OriginIsOrigin(&self) -> bool {
        self.node().origin == [0.; 3]
    }
    fn Matrix(&self) -> Transform<R> {
        Transform::FromColMajor(&self.node().matrix.values)
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
                .contents_rect,
        )
    }
    fn ScrollContainerRect(&self) -> Rect {
        rect(
            self.node()
                .scroll
                .as_ref()
                .expect("source ScrollNode")
                .container_rect,
        )
    }
    fn ScrollOverflowClip(&self) -> Option<Arc<native::ClipPaintPropertyNode>> {
        self.node()
            .scroll
            .as_ref()
            .expect("source ScrollNode")
            .overflow_clip
            .clone()
    }
}
impl<R: CpuSkiaBackend> GeometryMapperState for PropertyTreeState<R> {
    type Node = TransformNode<R>;
    type Effect = super::EffectNode<R>;
    fn Transform(&self) -> &Self::Node {
        &self.transform_
    }
    fn Clip(&self) -> &Arc<native::ClipPaintPropertyNode> {
        self.clip_.as_arc()
    }
    fn Effect(&self) -> &Self::Effect {
        &self.effect_
    }
    fn SetClip(&mut self, clip: Arc<native::ClipPaintPropertyNode>) {
        self.set_clip(super::ClipNode::from_arc(clip));
    }
    fn SetEffect(&mut self, effect: Self::Effect) {
        self.set_effect(effect);
    }
}
