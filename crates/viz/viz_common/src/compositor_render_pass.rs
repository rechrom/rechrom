//! viz::CompositorRenderPass and viz::CompositorFrame
//! (components/viz/common/quads/compositor_render_pass.h,
//! components/viz/common/quads/compositor_frame.h). All setters below only
//! write the requested field or append the requested material. Layerization,
//! culling, damage, tiling, draw order and scheduling remain in the cc owners
//! that call them; their facets are implemented by the cc packages that own
//! those traits.
use crate::draw_quad::{
    DrawQuad, DrawQuadHandle, QuadGeometry, QuadMaterial, RenderPassId, SharedQuadState,
    SharedQuadStateHandle,
};
use crate::ResourceId;
use compositor_tree::geometry::{Rect, RectF, Size};
use compositor_tree::property_types::{SubtreeCaptureId, ViewTransitionElementResourceId};
use compositor_values::color4f::Color4f;
use compositor_values::surface_id::SurfaceRange;
use std::marker::PhantomData;

/// The gfx/cc value types a frame carries (gfx::Transform,
/// cc::FilterOperations, SkPath, viz::CopyOutputRequest and the
/// PictureDrawQuad display list). Values only; no compositor owner.
pub trait FrameValueTypes {
    type FrameTransform: Clone + Default;
    type FrameFilters: Clone + Default;
    type FramePath: Clone;
    type FrameCopyRequest;
    type PictureDisplayItemList;
}
/// One concrete value family: the frame's value types and nothing else.
pub struct FrameValues<T, F, P, Q, L>(PhantomData<(T, F, P, Q, L)>);
impl<T: Clone + Default, F: Clone + Default, P: Clone, Q, L> FrameValueTypes
    for FrameValues<T, F, P, Q, L>
{
    type FrameTransform = T;
    type FrameFilters = F;
    type FramePath = P;
    type FrameCopyRequest = Q;
    type PictureDisplayItemList = L;
}

/// Owned native Viz pass. Copy requests are move-only; the pass intentionally
/// has no blanket Clone, so resource/result ownership cannot be duplicated.
pub struct CompositorRenderPass<V: FrameValueTypes> {
    pub id: RenderPassId,
    pub output_rect: Rect,
    pub damage_rect: Rect,
    pub transform_to_root_target: V::FrameTransform,
    pub filters: V::FrameFilters,
    pub backdrop_filters: V::FrameFilters,
    pub backdrop_filter_bounds: Option<V::FramePath>,
    pub generate_mipmap: bool,
    pub subtree_size: Size,
    pub has_damage_from_contributing_content: bool,
    pub subtree_capture_id: SubtreeCaptureId,
    pub cache_render_pass: bool,
    pub view_transition_element_resource_id: ViewTransitionElementResourceId,
    pub has_transparent_background: bool,
    pub copy_requests: Vec<Box<V::FrameCopyRequest>>,
    pub shared_quad_state_list: Vec<SharedQuadState<V::FrameTransform>>,
    pub quad_list: Vec<DrawQuad<V::PictureDisplayItemList>>,
}
impl<V: FrameValueTypes> Default for CompositorRenderPass<V> {
    fn default() -> Self {
        Self {
            id: RenderPassId::default(),
            output_rect: Rect::default(),
            damage_rect: Rect::default(),
            transform_to_root_target: V::FrameTransform::default(),
            filters: V::FrameFilters::default(),
            backdrop_filters: V::FrameFilters::default(),
            backdrop_filter_bounds: None,
            generate_mipmap: false,
            subtree_size: Size::default(),
            has_damage_from_contributing_content: false,
            subtree_capture_id: SubtreeCaptureId::default(),
            cache_render_pass: false,
            view_transition_element_resource_id: ViewTransitionElementResourceId::default(),
            has_transparent_background: true,
            copy_requests: Vec::new(),
            shared_quad_state_list: Vec::new(),
            quad_list: Vec::new(),
        }
    }
}
impl<V: FrameValueTypes> CompositorRenderPass<V> {
    pub fn new(num_contributors: usize) -> Self {
        let mut pass = Self::default();
        pass.shared_quad_state_list.reserve(num_contributors);
        pass
    }
    pub fn AppendSharedQuadState(&mut self) -> SharedQuadStateHandle {
        let id = SharedQuadStateHandle::FromIndex(self.shared_quad_state_list.len());
        self.shared_quad_state_list.push(SharedQuadState::default());
        id
    }
    pub fn SharedQuadState(
        &self,
        id: SharedQuadStateHandle,
    ) -> &SharedQuadState<V::FrameTransform> {
        &self.shared_quad_state_list[id.Index()]
    }
    /// Viz validation resolves the native handle without assuming that an
    /// untrusted submitted frame satisfies the producer-side invariant.
    pub fn TrySharedQuadState(
        &self,
        id: SharedQuadStateHandle,
    ) -> Option<&SharedQuadState<V::FrameTransform>> {
        self.shared_quad_state_list.get(id.Index())
    }
    pub fn SharedQuadStateMutable(
        &mut self,
        id: SharedQuadStateHandle,
    ) -> &mut SharedQuadState<V::FrameTransform> {
        &mut self.shared_quad_state_list[id.Index()]
    }
    /// CreateAndAppendDrawQuad: appends a default-geometry quad of the given
    /// material; the producer fills it with `SetQuad` (DrawQuad::SetNew).
    pub fn AppendMaterial(
        &mut self,
        material: QuadMaterial<V::PictureDisplayItemList>,
    ) -> DrawQuadHandle {
        let id = DrawQuadHandle::FromIndex(self.quad_list.len());
        self.quad_list.push(DrawQuad {
            geometry: QuadGeometry::default(),
            material,
        });
        id
    }
    pub fn SetQuad(
        &mut self,
        quad: DrawQuadHandle,
        shared: SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        needs_blending: bool,
        material: QuadMaterial<V::PictureDisplayItemList>,
    ) {
        assert!(
            shared.Index() < self.shared_quad_state_list.len(),
            "quad must reference its producing pass's actual shared state"
        );
        self.quad_list[quad.Index()] = DrawQuad {
            geometry: QuadGeometry {
                shared_quad_state: Some(shared),
                rect,
                visible_rect: visible,
                needs_blending,
            },
            material,
        };
    }

    /// Appends the current Chromium video quad shape. The caller owns resource
    /// import/release and the shared state, matching VideoResourceUpdater.
    pub fn AppendTextureQuad(
        &mut self,
        shared: SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        needs_blending: bool,
        resource_id: ResourceId,
        tex_coord_rect: RectF,
        is_tex_coord_rect_normalized: bool,
        background_color: Color4f,
        nearest_neighbor: bool,
        secure_output_only: bool,
        is_video_frame: bool,
        force_rgbx: bool,
    ) {
        assert_ne!(resource_id, ResourceId::default());
        let material = QuadMaterial::Texture {
            resource_id,
            tex_coord_rect,
            is_tex_coord_rect_normalized,
            background_color,
            nearest_neighbor,
            secure_output_only,
            is_video_frame,
            force_rgbx,
        };
        let quad = self.AppendMaterial(material.clone());
        self.SetQuad(quad, shared, rect, visible, needs_blending, material);
    }

    /// Source: components/viz/common/quads/surface_draw_quad.cc:23-35,37-53.
    pub fn AppendSurfaceQuad(
        &mut self,
        shared: SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        needs_blending: bool,
        surface_range: SurfaceRange,
        default_background_color: Color4f,
        stretch_content_to_fill_bounds: bool,
        is_reflection: bool,
        allow_merge: bool,
    ) {
        let material = QuadMaterial::Surface {
            surface_range,
            default_background_color,
            stretch_content_to_fill_bounds,
            is_reflection,
            allow_merge,
        };
        let quad = self.AppendMaterial(material.clone());
        self.SetQuad(quad, shared, rect, visible, needs_blending, material);
    }
    pub fn SetNew(
        &mut self,
        id: RenderPassId,
        output: Rect,
        damage: Rect,
        transform: &V::FrameTransform,
    ) {
        self.id = id;
        self.output_rect = output;
        self.damage_rect = damage;
        self.transform_to_root_target = transform.clone();
    }
}

/// Final owned output. Metadata/resources use the production family's native
/// value types; no live compositor/paint tree or raster pixel owner is copied.
pub struct CompositorFrame<M, P, R> {
    pub metadata: M,
    pub render_pass_list: Vec<Box<P>>,
    pub resource_list: Vec<R>,
}
impl<M: Default, P, R> Default for CompositorFrame<M, P, R> {
    fn default() -> Self {
        Self {
            metadata: M::default(),
            render_pass_list: Vec::new(),
            resource_list: Vec::new(),
        }
    }
}
