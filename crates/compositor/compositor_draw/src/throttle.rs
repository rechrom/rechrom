//! Live render-pass facets used by the draw throttling decision.
//!
//! Identities resolve retained passes during one traversal; no quads or
//! passes are copied into compositor storage.

use compositor_tree::{
    geometry::{Rect, RectF},
    transform::PropertyTreeTransform,
};

pub trait ThrottleTransform {
    fn MapRect(&self, rect: RectF) -> RectF;
}

impl<T: PropertyTreeTransform> ThrottleTransform for T {
    fn MapRect(&self, rect: RectF) -> RectF {
        PropertyTreeTransform::MapRect(self, rect)
    }
}

pub trait SurfaceRange<F> {
    fn IsValid(&self) -> bool;
    fn end_frame_sink_id(&self) -> F;
}

pub trait RenderPassQuad<P> {
    fn render_pass_id(&self) -> P;

    /// Whether the render-pass quad is masked by a resource. Throttling needs
    /// only this semantic fact, not the resource package's identifier type.
    fn has_mask_resource(&self) -> bool;
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

// The throttling facets of the viz frame values (viz_common): a pass, its
// quads and the cc::FilterOperations blur query the decision reads.
use viz_common::compositor_render_pass::{CompositorRenderPass, FrameValueTypes};
use viz_common::draw_quad::{DrawQuad, QuadMaterial, RenderPassId, SharedQuadState};

/// cc::FilterOperations::HasFilterOfType(kBlur) as the throttle reads it.
pub trait ThrottleFilters {
    fn HasBlurFilter(&self) -> bool;
}
impl ThrottleFilters for paint_model::cc::paint::filter_operations::FilterOperations {
    fn HasBlurFilter(&self) -> bool {
        self.HasFilterOfType(paint_model::cc::paint::filter_operation::FilterType::BLUR)
    }
}

impl<L> RenderPassQuad<RenderPassId> for DrawQuad<L> {
    fn render_pass_id(&self) -> RenderPassId {
        match self.material {
            QuadMaterial::CompositorRenderPass { render_pass_id, .. } => render_pass_id,
            _ => panic!("native render-pass quad material invariant"),
        }
    }
    fn has_mask_resource(&self) -> bool {
        self.resource_id().0 != 0
    }
}

/// Borrow-only native DrawQuad/shared-quad-state projection. A pass resolves
/// the handle at the point of iteration; neither reference is saved in a quad.
pub struct ThrottleQuadView<'a, T, L> {
    quad: &'a DrawQuad<L>,
    shared: &'a SharedQuadState<T>,
}
impl<T, F, L> ThrottleQuad<T, RenderPassId, F> for ThrottleQuadView<'_, T, L> {
    fn render_pass_quad(&self) -> Option<&dyn RenderPassQuad<RenderPassId>> {
        matches!(
            self.quad.material,
            QuadMaterial::CompositorRenderPass { .. }
        )
        .then_some(self.quad)
    }
    fn surface_quad(&self) -> Option<&dyn SurfaceQuad<F>> {
        // None of the represented native materials is a SurfaceDrawQuad.
        // Surface allocation/aggregation belongs to the external Viz boundary.
        None
    }
    fn quad_to_target_transform(&self) -> &T {
        &self.shared.quad_to_target_transform
    }
    fn clip_rect(&self) -> Option<Rect> {
        self.shared.clip_rect
    }
    fn visible_rect(&self) -> Rect {
        self.quad.geometry.visible_rect
    }
}
pub struct ThrottleQuads<'a, V: FrameValueTypes> {
    pass: &'a CompositorRenderPass<V>,
    iter: std::slice::Iter<'a, DrawQuad<V::PictureDisplayItemList>>,
}
impl<'a, V: FrameValueTypes> Iterator for ThrottleQuads<'a, V> {
    type Item = ThrottleQuadView<'a, V::FrameTransform, V::PictureDisplayItemList>;
    fn next(&mut self) -> Option<Self::Item> {
        let quad = self.iter.next()?;
        let handle = quad
            .geometry
            .shared_quad_state
            .expect("native quad has a shared quad state");
        let shared = self
            .pass
            .shared_quad_state_list
            .get(handle.Index())
            .expect("native shared state belongs to this pass");
        Some(ThrottleQuadView { quad, shared })
    }
}
impl<V, F> ThrottlePass<V::FrameTransform, RenderPassId, F> for CompositorRenderPass<V>
where
    V: FrameValueTypes,
    V::FrameFilters: ThrottleFilters,
    V::FramePath: cc_paint::skia_backend::Path,
{
    type Quad<'a>
        = ThrottleQuadView<'a, V::FrameTransform, V::PictureDisplayItemList>
    where
        Self: 'a;
    type Quads<'a>
        = ThrottleQuads<'a, V>
    where
        Self: 'a;
    fn id(&self) -> RenderPassId {
        self.id
    }
    fn filters_have_blur(&self) -> bool {
        self.filters.HasBlurFilter()
    }
    fn backdrop_filters_have_blur(&self) -> bool {
        self.backdrop_filters.HasBlurFilter()
    }
    fn output_rect(&self) -> Rect {
        self.output_rect
    }
    fn backdrop_filter_bounds(&self) -> Option<RectF> {
        self.backdrop_filter_bounds
            .as_ref()
            .map(cc_paint::skia_backend::Path::bounds)
    }
    fn quads(&self) -> Self::Quads<'_> {
        ThrottleQuads {
            pass: self,
            iter: self.quad_list.iter(),
        }
    }
}
