//! Live Viz render-pass/quad and layer facets. Identities resolve retained
//! passes during one traversal; no quads or passes are copied into cc storage.
use super::{
    geometry::{Rect, RectF},
    layer_tree_impl::ResourceId,
};
pub trait ThrottleTransform {
    fn MapRect(&self, rect: RectF) -> RectF;
}
impl<T: super::property_tree::PropertyTreeTransform> ThrottleTransform for T {
    fn MapRect(&self, rect: RectF) -> RectF {
        super::property_tree::PropertyTreeTransform::MapRect(self, rect)
    }
}
pub trait SurfaceRange<F> {
    fn IsValid(&self) -> bool;
    fn end_frame_sink_id(&self) -> F;
}
pub trait RenderPassQuad<P> {
    fn render_pass_id(&self) -> P;
    fn resource_id(&self) -> ResourceId;
}
pub trait SurfaceQuad<F> {
    fn surface_range(&self) -> &dyn SurfaceRange<F>;
}
pub trait ThrottleQuad<T, P, F> {
    fn render_pass_quad(&self) -> Option<&dyn RenderPassQuad<P>>;
    fn surface_quad(&self) -> Option<&dyn SurfaceQuad<F>>;
    fn quad_to_target_transform(&self) -> &T;
    fn clip_rect(&self) -> Option<Rect>;
    fn visible_rect(&self) -> Rect;
}
pub trait ThrottlePass<T, P, F> {
    /// A stack-only view can borrow a quad and its pass-owned shared state.
    /// Native DrawQuad's non-owning state link resolves through the actual
    /// resident pass; no state copy, stored pass pointer or interior cell.
    type Quad<'a>: ThrottleQuad<T, P, F>
    where
        Self: 'a;
    type Quads<'a>: Iterator<Item = Self::Quad<'a>>
    where
        Self: 'a;
    fn id(&self) -> P;
    fn filters_have_blur(&self) -> bool;
    fn backdrop_filters_have_blur(&self) -> bool;
    fn output_rect(&self) -> Rect;
    fn backdrop_filter_bounds(&self) -> Option<RectF>;
    fn quads(&self) -> Self::Quads<'_>;
}
pub trait ThrottleLayer<F> {
    /// Native SurfaceLayerImpl subtype projection; ordinary layers have none.
    fn surface_range(&self) -> Option<&dyn SurfaceRange<F>>;
}
pub trait ThrottleTypes {
    type Transform: ThrottleTransform;
    type PassId: Ord + Copy;
    type FrameSinkId: Ord + Copy;
    type PassHandle: Copy;
    type Pass: ThrottlePass<Self::Transform, Self::PassId, Self::FrameSinkId>;
}
pub trait ThrottleContext: ThrottleTypes {
    fn pass(&self, id: Self::PassHandle) -> &Self::Pass;
}
