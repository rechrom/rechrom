//! Actual Paint node projections into the canonical GeometryMapper algorithms.
use super::{ClipNode, EffectNode, PropertyTreeState, TransformNode};
use crate::{
    adapter::{
        geometry::{Rect, RectF, Vector2dF},
        paint_geometry::{
            ChunkGeometryMapper, FloatClipRectTransform, GeometryMapperContext, VisualRectFlags,
        },
        property_tree::PropertyTreeTransform,
    },
    runtime::{geometry::Transform, types::CpuSkiaBackend},
    third_party::blink::renderer::platform::graphics::paint::{
        float_clip_rect::{FloatClipRect, InfiniteLooseFloatClipRect},
        geometry_mapper::GeometryMapper,
        geometry_mapper_clip_cache::{OverlayScrollbarClipBehavior, WithClipCache},
    },
};
use paint::paint_property_tree as native;
use std::{marker::PhantomData, sync::Arc};
pub struct CpuGeometryMapper<R: CpuSkiaBackend> {
    pub(super) tracking: Arc<dyn super::CpuRasterTrackingServices<R>>,
    singular_transform_fix: bool,
    marker: PhantomData<fn() -> R>,
}
impl<R: CpuSkiaBackend> CpuGeometryMapper<R> {
    pub fn IsTracingRasterInvalidations(&self) -> bool {
        self.tracking.IsTracingRasterInvalidations()
    }
    pub fn new(
        singular_transform_fix: bool,
        tracking: Arc<dyn super::CpuRasterTrackingServices<R>>,
    ) -> Self {
        Self {
            tracking,
            singular_transform_fix,
            marker: PhantomData,
        }
    }
}
fn rect(r: paint::paint_engine::PaintRect) -> RectF {
    RectF::new(r.x as f32, r.y as f32, r.width as f32, r.height as f32)
}
fn clip_rect(r: native::ClipLayoutRect) -> FloatClipRect {
    let mut value = r
        .rect
        .map(|r| FloatClipRect::FromRect(rect(r)))
        .unwrap_or_default();
    if r.has_radius {
        value.SetHasRadius();
    }
    value
}
impl<R: CpuSkiaBackend> GeometryMapperContext for CpuGeometryMapper<R> {
    type State = PropertyTreeState<R>;
    fn SingularTransformFixEnabled(&self) -> bool {
        self.singular_transform_fix
    }
    fn RootState(&self) -> Self::State {
        Self::State::default()
    }
    fn MakeState(
        &self,
        transform: TransformNode<R>,
        clip: Arc<native::ClipPaintPropertyNode>,
        effect: EffectNode<R>,
    ) -> Self::State {
        PropertyTreeState {
            transform_: transform,
            clip_: ClipNode::from_arc(clip),
            effect_: effect,
        }
    }
    fn ClipLocalTransform(&self, clip: &Arc<native::ClipPaintPropertyNode>) -> TransformNode<R> {
        TransformNode::from_arc(clip.local_transform_space.clone())
    }
    fn ClipLowestCommonAncestor(
        &self,
        a: &Arc<native::ClipPaintPropertyNode>,
        b: &Arc<native::ClipPaintPropertyNode>,
    ) -> Arc<native::ClipPaintPropertyNode> {
        paint::property_node_geometry::LowestCommonAncestor(a, b)
    }
    fn LayoutClipRectExcludingOverlayScrollbars(
        &self,
        c: &Arc<native::ClipPaintPropertyNode>,
    ) -> FloatClipRect {
        clip_rect(
            c.layout_clip_rects
                .excluding_overlay
                .unwrap_or(c.layout_clip_rects.expanded),
        )
    }
    fn PreciseLayoutClipRect(&self, c: &Arc<native::ClipPaintPropertyNode>) -> FloatClipRect {
        clip_rect(c.layout_clip_rects.precise)
    }
    fn LayoutClipRect(&self, c: &Arc<native::ClipPaintPropertyNode>) -> FloatClipRect {
        clip_rect(c.layout_clip_rects.expanded)
    }
    fn PaintClipRect(&self, c: &Arc<native::ClipPaintPropertyNode>) -> RectF {
        c.rect
            .map(rect)
            .unwrap_or_else(|| *FloatClipRect::default().Rect())
    }
    fn ClipHasPath(&self, c: &Arc<native::ClipPaintPropertyNode>) -> bool {
        c.clip_path.is_some()
    }
    // Source: clip_paint_property_node.h:188-217; this is an exact field query,
    // not a geometric inference about whether a CSS animation is running.
    fn IsForCompositeClipPathAnimation(&self, c: &Arc<native::ClipPaintPropertyNode>) -> bool {
        c.layout_clip_rects.expanded != c.layout_clip_rects.precise
    }
    fn PixelMovingFilter(&self, c: &Arc<native::ClipPaintPropertyNode>) -> EffectNode<R> {
        EffectNode::from_arc(
            c.pixel_moving_filter
                .as_ref()
                .expect("source PixelMovingFilter")
                .clone(),
        )
    }
    fn FilterLocalTransform(&self, e: &EffectNode<R>) -> TransformNode<R> {
        TransformNode::from_arc(e.node().local_transform_space.clone())
    }
    fn HasActiveFilterAnimation(&self, e: &EffectNode<R>) -> bool {
        e.node()
            .direct_compositing_reasons
            .contains(&"ActiveFilterAnimation")
    }
    fn FilterMapRect(&self, e: &EffectNode<R>, r: Rect) -> Rect {
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
    fn SelfOrAncestorParticipatesInViewTransition(&self, e: &EffectNode<R>) -> bool {
        e.node().self_or_ancestor_participates_in_view_transition
    }
}
impl<R: CpuSkiaBackend> ChunkGeometryMapper for CpuGeometryMapper<R> {
    type PropertyTreeState = PropertyTreeState<R>;
    type Transform = Transform<R>;
    type FloatClipRect = FloatClipRect;
    fn MakeTranslation(&self, v: Vector2dF) -> Transform<R> {
        let mut m = Transform::default();
        m.Translate(v.x(), v.y());
        m
    }
    fn MakeDefaultClipRect(&self) -> FloatClipRect {
        FloatClipRect::default()
    }
    fn MakeFloatClipRect(&self, r: RectF) -> FloatClipRect {
        FloatClipRect::FromRect(r)
    }
    fn InfiniteLooseFloatClipRect(&self) -> FloatClipRect {
        InfiniteLooseFloatClipRect()
    }
    fn SameTransformNode(&self, a: &Self::PropertyTreeState, b: &Self::PropertyTreeState) -> bool {
        a.transform_ == b.transform_
    }
    fn SameClipNode(&self, a: &Self::PropertyTreeState, b: &Self::PropertyTreeState) -> bool {
        a.clip_ == b.clip_
    }
    fn SameNearestPixelMovingFilterClip(
        &self,
        a: &Self::PropertyTreeState,
        b: &Self::PropertyTreeState,
    ) -> bool {
        let a = WithClipCache(a.clip_.as_arc(), |c| c.NearestPixelMovingFilterClip());
        let b = WithClipCache(b.clip_.as_arc(), |c| c.NearestPixelMovingFilterClip());
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(&a, &b),
            _ => false,
        }
    }
    fn SourceToDestinationProjection(
        &self,
        a: &Self::PropertyTreeState,
        b: &Self::PropertyTreeState,
        out: &mut Transform<R>,
    ) -> bool {
        GeometryMapper::<Self>::SourceToDestinationProjectionTo(
            self,
            &a.transform_,
            &b.transform_,
            out,
        )
    }
    fn PostTranslate(&self, m: &mut Transform<R>, v: Vector2dF) {
        m.PostTranslate(v)
    }
    fn MakeIdentity(&self, m: &mut Transform<R>) {
        *m = Transform::default()
    }
    fn ScaleToZero(&self, m: &mut Transform<R>) {
        m.Scale(0., 0.)
    }
    fn MapRect(&self, m: &Transform<R>, r: RectF) -> RectF {
        FloatClipRectTransform::MapRect(m, r)
    }
    fn LocalToAncestorClipRect(
        &self,
        a: &Self::PropertyTreeState,
        b: &Self::PropertyTreeState,
    ) -> FloatClipRect {
        GeometryMapper::<Self>::LocalToAncestorClipRect(
            self,
            a,
            b,
            OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
        )
    }
    fn LocalToAncestorVisualRect(
        &self,
        a: &Self::PropertyTreeState,
        b: &Self::PropertyTreeState,
        r: &mut FloatClipRect,
    ) {
        GeometryMapper::<Self>::LocalToAncestorVisualRect(
            self,
            a,
            b,
            r,
            OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
            VisualRectFlags::default(),
        );
    }
    fn ClipIsInfinite(&self, r: &FloatClipRect) -> bool {
        r.IsInfinite()
    }
    fn ClipRect(&self, r: &FloatClipRect) -> RectF {
        *r.Rect()
    }
    fn MoveClip(&self, r: &mut FloatClipRect, v: Vector2dF) {
        r.Move(v)
    }
}
