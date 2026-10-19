//! Borrowed native node projections for PaintChunksToCcLayer. No converted
//! property graph is retained here; each handle reads its actual Paint owner.
use super::*;
use crate::{adapter::{paint_chunks_to_cc_layer::*,pending_layer::PendingClipNode,geometry::{Rect,RectF,RRectF,Vector2dF},input_types::ElementId},runtime::types::CpuSkiaBackend};
use crate::third_party::blink::renderer::platform::graphics::paint::geometry_mapper_transform_cache::WithTransformCache;
fn rect(p: paint::paint_engine::PaintRect) -> Rect {
    crate::adapter::geometry::ToEnclosingRect(RectF::new(
        p.x as f32,
        p.y as f32,
        p.width as f32,
        p.height as f32,
    ))
}
fn element(id: foundation::CompositorElementId) -> ElementId {
    if id.GetInternalValue() == 0 {
        ElementId::default()
    } else {
        ElementId::new(id.GetInternalValue())
    }
}
impl<R, B> ConversionTransformNode<B> for TransformNode<R>
where
    R: CpuSkiaBackend,
    B: PaintChunksTypes<TransformNode = Self, ScrollNode = ScrollNode<R>>,
{
    fn ScrollNode(&self) -> Option<ScrollNode<R>> {
        self.node()
            .scroll
            .as_ref()
            .map(|s| ScrollNode::from_arc(s.clone()))
    }
    fn NearestScrollTranslationNode(&self) -> Self {
        WithTransformCache(self, |c| c.nearest_scroll_translation())
    }
    fn ParentScrollTranslationNode(&self) -> Option<Self> {
        self.node().parent.as_ref().map(|p| {
            WithTransformCache(&Self::from_arc(p.clone()), |c| {
                c.nearest_scroll_translation()
            })
        })
    }
    fn IsAncestorOf(&self, other: &Self) -> bool {
        let mut n = Some(other.node());
        while let Some(node) = n {
            if self.node().lifecycle.same_node(&node.lifecycle) {
                return true;
            }
            n = node.parent.as_deref();
        }
        false
    }
    fn IsIdentity(&self) -> bool {
        crate::adapter::paint_geometry::TransformCacheNode::IsIdentity(self)
    }
    fn ParentIs(&self, other: &Self) -> bool {
        self.node()
            .parent
            .as_ref()
            .is_some_and(|p| p.lifecycle.same_node(&other.node().lifecycle))
    }
}
impl<R, B> ConversionClipNode<B> for ClipNode<R>
where
    R: CpuSkiaBackend,
    B: PaintChunksTypes<
        ClipNode = Self,
        RoundedRect = RRectF,
        PaintPath = Arc<paint::paint_clip_path::PaintClipPath>,
    >,
{
    fn UnaliasedParent(&self) -> Option<Self> {
        self.node()
            .parent
            .as_ref()
            .map(|p| Self::from_arc(p.clone()))
    }
    fn PaintClipRect(&self) -> RRectF {
        let r = self.node().radii;
        RRectF {
            rect: PendingClipNode::<TransformNode<R>, usize>::PaintClipRectRect(self),
            corner_radii: [r.top_left, r.top_right, r.bottom_right, r.bottom_left]
                .map(|p| Vector2dF::new(p.x as f32, p.y as f32)),
        }
    }
    fn ClipPath(&self) -> Option<Arc<paint::paint_clip_path::PaintClipPath>> {
        self.node().clip_path.clone()
    }
}
impl<R, B> ConversionEffectNode<B> for EffectNode<R>
where
    R: CpuSkiaBackend,
    B: PaintChunksTypes<
        EffectNode = Self,
        FilterOperations = paint::cc::paint::filter_operations::FilterOperations,
        PaintPath = Arc<paint::paint_clip_path::PaintClipPath>,
    >,
{
    fn UnaliasedParent(&self) -> Option<Self> {
        self.node()
            .parent
            .as_ref()
            .map(|p| Self::from_arc(p.clone()))
    }
    fn HasDirectCompositingReasons(&self) -> bool {
        !self.node().direct_compositing_reasons.is_empty()
    }
    fn RequiresCompositingForCanvasChild(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"CanvasChild")
    }
    fn HasRealEffects(&self) -> bool {
        self.node().opacity!=1.||crate::adapter::pending_layer::PendingEffectNode::<TransformNode<R>,usize>::BlendMode(self)!=crate::adapter::property_types::BlendMode::kSrcOver||self.node().compositor.has_filter_info||self.node().compositor.has_backdrop_filter_info
    }
    fn Filter(&self) -> Option<B::FilterOperations> {
        self.node().compositor.has_filter_info.then(|| {
            self.node()
                .filters
                .canonical()
                .expect("validated native filter")
                .clone()
        })
    }
    fn BackdropFilter(&self) -> Option<B::FilterOperations> {
        self.node().compositor.has_backdrop_filter_info.then(|| {
            self.node()
                .backdrop_filters
                .canonical()
                .expect("validated native backdrop filter")
                .clone()
        })
    }
    fn BackdropFilterBounds(&self) -> Arc<paint::paint_clip_path::PaintClipPath> {
        Arc::new(paint::paint_clip_path::PaintClipPath::from_path(
            self.node()
                .backdrop_filter_bounds
                .as_ref()
                .expect("validated backdrop bounds")
                .clone(),
        ))
    }
    fn HasReferenceFilter(&self) -> bool {
        self.node()
            .filters
            .canonical()
            .expect("validated filter")
            .HasReferenceFilter()
            || self
                .node()
                .backdrop_filters
                .canonical()
                .expect("validated backdrop filter")
                .HasReferenceFilter()
    }
    fn FilterOutputBounds(&self) -> Rect {
        rect(
            self.node()
                .compositor
                .filter_output_bounds
                .expect("PaintInput validates reference-filter output bounds"),
        )
    }
    // Source: effect_paint_property_node.cc:229-237. Preserve reference-filter
    // output bounds rather than assuming a local maximum outset.
    fn MapRect(&self, input: Rect) -> Rect {
        if !self.node().compositor.has_filter_info {
            return input;
        }
        let ops = self.node().filters.canonical().expect("validated filter");
        if ops.HasReferenceFilter() {
            return <Self as ConversionEffectNode<B>>::FilterOutputBounds(self);
        }
        let r = ops.MapRect(
            foundation::gfx::Rect::new(
                foundation::gfx::Point::new(input.x, input.y),
                foundation::gfx::Size::new(input.width, input.height),
            ),
            None,
        );
        Rect::new(r.x(), r.y(), r.width(), r.height())
    }
}
impl<R, B> ConversionScrollNode<B> for ScrollNode<R>
where
    R: CpuSkiaBackend,
    B: PaintChunksTypes<ScrollNode = Self, ClipNode = ClipNode<R>>,
{
    fn Parent(&self) -> Option<Self> {
        self.node()
            .parent
            .as_ref()
            .map(|s| Self::from_arc(s.clone()))
    }
    fn ContentsRect(&self) -> Rect {
        rect(self.node().contents_rect)
    }
    fn ContainerRect(&self) -> Rect {
        rect(self.node().container_rect)
    }
    fn UserScrollableHorizontal(&self) -> bool {
        self.node().user_scrollable_horizontal
    }
    fn PropagatesXScroll(&self) -> bool {
        self.node().compositor.overscroll_behavior[0].AllowsScrollChaining()
    }
    fn CompositedScrollingNotPreferred(&self) -> bool {
        self.node().compositor.composited_scrolling_preference
            == native::PaintCompositedScrollingPreference::NotPreferred
    }
    fn GetCompositorElementId(&self) -> ElementId {
        element(self.node().compositor.compositor_element_id)
    }
    fn OverflowClipNode(&self) -> Option<ClipNode<R>> {
        self.node()
            .overflow_clip
            .as_ref()
            .map(|p| ClipNode::from_arc(p.clone()))
    }
}
