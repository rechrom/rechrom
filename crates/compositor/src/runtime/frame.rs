//! Native Viz owned values. All setters below only write the requested field or
//! append the requested material. Layerization, culling, damage, tiling, draw
//! order and scheduling remain in the canonical cc owners that call them.
pub use crate::adapter::layer_tree_impl::ResourceId;
pub use crate::adapter::scheduler_host::BeginFrameAck;
use crate::adapter::{
    frame_data::{FrameDataRenderPass, FrameDataTraceValue, FrameDataTypes},
    geometry::{PointF, Rect, RectF, Size, Vector2dF},
    layer_impl::{
        LayerImplDrawQuad, LayerImplRenderPass, LayerImplSharedQuadState, LayerImplTypes,
    },
    layer_tree_impl::{Color4f, LayerTreeTypes},
    occlusion::SkBlendMode,
    property_types::{MaskFilterInfo, SubtreeCaptureId, ViewTransitionElementResourceId},
    render_surface_impl::{RenderSurfaceRenderPass, RenderSurfaceTypes},
    solid_color_layer_impl::{SolidColorLayerImplTypes, SolidColorRenderPass},
    tile_based_layer_impl::{
        TileBasedLayerImplTypes, TileBasedRenderPass, TileBasedSharedQuadState,
    },
};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};
/// components/viz/common/quads/compositor_render_pass_id.h native 64-bit ID.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderPassId(pub u64);
impl From<u64> for RenderPassId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
/// Indices identify actual resident output entries within their producing pass.
/// Only that pass can allocate them; source EraseQuad invalidates later handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedQuadStateHandle(usize);
impl SharedQuadStateHandle {
    /// Rebind a copied Viz quad to a shared state in its destination pass.
    pub const fn FromIndex(index: usize) -> Self {
        Self(index)
    }

    /// Resolves an immutable handle in an owning Viz render pass.
    pub const fn Index(self) -> usize {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawQuadHandle(usize);
/// External geometry/Paint values selected once by the production family.
/// These are value/backend types, never compositor owner implementations.
pub trait FrameValueTypes {
    type FrameTransform: Clone + Default;
    type FrameFilters: Clone + Default;
    type FramePath: Clone;
    type FrameCopyRequest;
    type PictureDisplayItemList;
}
/// Shared native field storage for any concrete injected geometry/Paint backend.
/// A full owner family implements this same value vocabulary; no owner is held.
pub struct FrameValues<T, F, P, Q, S, Timing, L>(PhantomData<(T, F, P, Q, S, Timing, L)>);
impl<T: Clone + Default, F: Clone + Default, P: Clone, Q, S, Timing, L> FrameValueTypes
    for FrameValues<T, F, P, Q, S, Timing, L>
{
    type FrameTransform = T;
    type FrameFilters = F;
    type FramePath = P;
    type FrameCopyRequest = Q;
    type PictureDisplayItemList = L;
}
impl<T: Clone + Default, F: Clone + Default, P: Clone, Q, S, Timing: Clone, L> FrameDataTypes
    for FrameValues<T, F, P, Q, S, Timing, L>
{
    type SharedQuadStateHandle = SharedQuadStateHandle;
    type DrawQuadHandle = DrawQuadHandle;
    type RenderPassId = RenderPassId;
    type SurfaceId = S;
    type ResourceId = ResourceId;
    type RenderPass = CompositorRenderPass<Self>;
    type BeginFrameAck = BeginFrameAck;
    type TreesInVizTiming = Timing;
}
/// One root-family constraint makes all canonical material adapters use this
/// exact owned pass/state/quad storage and the existing unique native IDs.
pub trait NativeFrameTypes:
    FrameValueTypes<
        FrameTransform = <Self as LayerTreeTypes>::Transform,
        FrameFilters = <Self as LayerTreeTypes>::Filters,
        FramePath = <Self as LayerTreeTypes>::Path,
        FrameCopyRequest = <Self as LayerTreeTypes>::CopyOutputRequest,
    > + LayerImplTypes<
        SharedQuadState = SharedQuadState<<Self as LayerTreeTypes>::Transform>,
        DrawQuad = DrawQuad<<Self as FrameValueTypes>::PictureDisplayItemList>,
    > + FrameDataTypes<
        RenderPassId = RenderPassId,
        SharedQuadStateHandle = SharedQuadStateHandle,
        DrawQuadHandle = DrawQuadHandle,
        RenderPass = CompositorRenderPass<Self>,
    >
{
}
#[derive(Clone, Debug)]
pub struct SharedQuadState<T> {
    pub quad_to_target_transform: T,
    pub quad_layer_rect: Rect,
    pub visible_quad_layer_rect: Rect,
    pub mask_filter_info: MaskFilterInfo,
    pub clip_rect: Option<Rect>,
    pub are_contents_opaque: bool,
    pub opacity: f32,
    pub blend_mode: SkBlendMode,
    pub sorting_context_id: i32,
    pub layer_id: u32,
    pub is_fast_rounded_corner: bool,
}
impl<T: Default> Default for SharedQuadState<T> {
    fn default() -> Self {
        Self {
            quad_to_target_transform: T::default(),
            quad_layer_rect: Rect::default(),
            visible_quad_layer_rect: Rect::default(),
            mask_filter_info: MaskFilterInfo::default(),
            clip_rect: None,
            are_contents_opaque: false,
            opacity: 1.,
            blend_mode: SkBlendMode::kSrcOver,
            sorting_context_id: 0,
            layer_id: 0,
            is_fast_rounded_corner: false,
        }
    }
}
impl<T: Clone> LayerImplSharedQuadState<T> for SharedQuadState<T> {
    fn opacity(&self) -> f32 {
        self.opacity
    }
    fn SetAll(
        &mut self,
        transform: &T,
        rect: Rect,
        visible: Rect,
        mask: &MaskFilterInfo,
        clip: Option<Rect>,
        opaque: bool,
        opacity: f32,
        blend: SkBlendMode,
        sorting: i32,
        layer: u32,
        fast: bool,
    ) {
        self.quad_to_target_transform = transform.clone();
        self.quad_layer_rect = rect;
        self.visible_quad_layer_rect = visible;
        self.mask_filter_info = mask.clone();
        self.clip_rect = clip;
        self.are_contents_opaque = opaque;
        self.opacity = opacity;
        self.blend_mode = blend;
        self.sorting_context_id = sorting;
        self.layer_id = layer;
        self.is_fast_rounded_corner = fast;
    }
}
impl<T: Clone> TileBasedSharedQuadState<T> for SharedQuadState<T> {
    fn quad_to_target_transform(&self) -> &T {
        &self.quad_to_target_transform
    }
    fn quad_to_target_transform_mut(&mut self) -> &mut T {
        &mut self.quad_to_target_transform
    }
    fn quad_layer_rect(&self) -> Rect {
        self.quad_layer_rect
    }
    fn set_quad_layer_rect(&mut self, value: Rect) {
        self.quad_layer_rect = value;
    }
    fn visible_quad_layer_rect(&self) -> Rect {
        self.visible_quad_layer_rect
    }
    fn set_visible_quad_layer_rect(&mut self, value: Rect) {
        self.visible_quad_layer_rect = value;
    }
    fn clip_rect(&self) -> Option<Rect> {
        self.clip_rect
    }
    fn set_clip_rect(&mut self, value: Option<Rect>) {
        self.clip_rect = value;
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct QuadGeometry {
    pub shared_quad_state: Option<SharedQuadStateHandle>,
    pub rect: Rect,
    pub visible_rect: Rect,
    pub needs_blending: bool,
}
#[derive(Debug)]
pub enum QuadMaterial<L> {
    /// Source components/viz/common/quads/picture_draw_quad.h:29-50.
    Picture {
        tex_coord_rect: RectF,
        nearest_neighbor: bool,
        content_rect: Rect,
        contents_scale: f32,
        image_animation_map:
            std::collections::BTreeMap<crate::adapter::image_animation::PaintImageId, usize>,
        display_item_list: Option<Arc<L>>,
        raster_inducing_scroll_offsets: crate::adapter::prioritized_tile::ScrollOffsetMap,
    },
    /// Native SharedElementDrawQuad value, resolved to resources by external Viz.
    SharedElement {
        element_resource_id: ViewTransitionElementResourceId,
    },
    SolidColor {
        color: Color4f,
        force_anti_aliasing_off: bool,
    },
    DebugBorder {
        color: Color4f,
        width: i32,
    },
    Tile {
        resource_id: ResourceId,
        tex_coord_rect: RectF,
        nearest_neighbor: bool,
        force_anti_aliasing_off: bool,
    },
    CompositorRenderPass {
        render_pass_id: RenderPassId,
        mask_resource_id: ResourceId,
        mask_uv_rect: RectF,
        mask_texture_size: Size,
        filters_scale: Vector2dF,
        filters_origin: PointF,
        force_anti_aliasing_off: bool,
        backdrop_filter_quality: f32,
        intersects_damage_under: bool,
    },
}
impl<L> Clone for QuadMaterial<L> {
    fn clone(&self) -> Self {
        match self {
            Self::Picture {
                tex_coord_rect,
                nearest_neighbor,
                content_rect,
                contents_scale,
                image_animation_map,
                display_item_list,
                raster_inducing_scroll_offsets,
            } => Self::Picture {
                tex_coord_rect: *tex_coord_rect,
                nearest_neighbor: *nearest_neighbor,
                content_rect: *content_rect,
                contents_scale: *contents_scale,
                image_animation_map: image_animation_map.clone(),
                display_item_list: display_item_list.clone(),
                raster_inducing_scroll_offsets: raster_inducing_scroll_offsets.clone(),
            },
            Self::SharedElement {
                element_resource_id,
            } => Self::SharedElement {
                element_resource_id: *element_resource_id,
            },
            Self::SolidColor {
                color,
                force_anti_aliasing_off,
            } => Self::SolidColor {
                color: *color,
                force_anti_aliasing_off: *force_anti_aliasing_off,
            },
            Self::DebugBorder { color, width } => Self::DebugBorder {
                color: *color,
                width: *width,
            },
            Self::Tile {
                resource_id,
                tex_coord_rect,
                nearest_neighbor,
                force_anti_aliasing_off,
            } => Self::Tile {
                resource_id: *resource_id,
                tex_coord_rect: *tex_coord_rect,
                nearest_neighbor: *nearest_neighbor,
                force_anti_aliasing_off: *force_anti_aliasing_off,
            },
            Self::CompositorRenderPass {
                render_pass_id,
                mask_resource_id,
                mask_uv_rect,
                mask_texture_size,
                filters_scale,
                filters_origin,
                force_anti_aliasing_off,
                backdrop_filter_quality,
                intersects_damage_under,
            } => Self::CompositorRenderPass {
                render_pass_id: *render_pass_id,
                mask_resource_id: *mask_resource_id,
                mask_uv_rect: *mask_uv_rect,
                mask_texture_size: *mask_texture_size,
                filters_scale: *filters_scale,
                filters_origin: *filters_origin,
                force_anti_aliasing_off: *force_anti_aliasing_off,
                backdrop_filter_quality: *backdrop_filter_quality,
                intersects_damage_under: *intersects_damage_under,
            },
        }
    }
}
#[derive(Debug)]
pub struct DrawQuad<L> {
    pub geometry: QuadGeometry,
    pub material: QuadMaterial<L>,
}
impl<L> Clone for DrawQuad<L> {
    fn clone(&self) -> Self {
        Self {
            geometry: self.geometry,
            material: self.material.clone(),
        }
    }
}
impl<L> LayerImplDrawQuad for DrawQuad<L> {
    fn resource_id(&self) -> ResourceId {
        match self.material {
            QuadMaterial::Tile { resource_id, .. } => resource_id,
            QuadMaterial::CompositorRenderPass {
                mask_resource_id, ..
            } => mask_resource_id,
            _ => ResourceId::default(),
        }
    }
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
        let id = SharedQuadStateHandle(self.shared_quad_state_list.len());
        self.shared_quad_state_list.push(SharedQuadState::default());
        id
    }
    pub fn SharedQuadState(
        &self,
        id: SharedQuadStateHandle,
    ) -> &SharedQuadState<V::FrameTransform> {
        &self.shared_quad_state_list[id.0]
    }
    /// Viz validation resolves the native handle without assuming that an
    /// untrusted submitted frame satisfies the producer-side invariant.
    pub fn TrySharedQuadState(
        &self,
        id: SharedQuadStateHandle,
    ) -> Option<&SharedQuadState<V::FrameTransform>> {
        self.shared_quad_state_list.get(id.0)
    }
    pub fn SharedQuadStateMutable(
        &mut self,
        id: SharedQuadStateHandle,
    ) -> &mut SharedQuadState<V::FrameTransform> {
        &mut self.shared_quad_state_list[id.0]
    }
    fn AppendMaterial(
        &mut self,
        material: QuadMaterial<V::PictureDisplayItemList>,
    ) -> DrawQuadHandle {
        let id = DrawQuadHandle(self.quad_list.len());
        self.quad_list.push(DrawQuad {
            geometry: QuadGeometry::default(),
            material,
        });
        id
    }
    fn SetQuad(
        &mut self,
        quad: DrawQuadHandle,
        shared: SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        needs_blending: bool,
        material: QuadMaterial<V::PictureDisplayItemList>,
    ) {
        assert!(
            shared.0 < self.shared_quad_state_list.len(),
            "quad must reference its producing pass's actual shared state"
        );
        self.quad_list[quad.0] = DrawQuad {
            geometry: QuadGeometry {
                shared_quad_state: Some(shared),
                rect,
                visible_rect: visible,
                needs_blending,
            },
            material,
        };
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
impl<V: FrameValueTypes> FrameDataRenderPass for CompositorRenderPass<V> {
    type ResourceId = ResourceId;
    fn AsValueInto<T: FrameDataTraceValue>(
        &self,
        value: &mut T,
        resources: &mut HashMap<ResourceId, usize>,
    ) {
        value.BeginDictionary();
        value.SetBoolean(
            "has_transparent_background",
            self.has_transparent_background,
        );
        value.SetBoolean(
            "has_damage_from_contributing_content",
            self.has_damage_from_contributing_content,
        );
        value.BeginArray("quad_list");
        for quad in &self.quad_list {
            value.BeginDictionary();
            let id = quad.resource_id();
            if id != ResourceId::default() {
                let next = resources.len();
                resources.entry(id).or_insert(next);
            }
            value.SetBoolean("needs_blending", quad.geometry.needs_blending);
            value.EndDictionary();
        }
        value.EndArray();
    }
}
impl<B: NativeFrameTypes> LayerImplRenderPass<B> for CompositorRenderPass<B> {
    fn AppendDebugBorderQuad(
        &mut self,
        state: SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        color: Color4f,
        width: f32,
    ) {
        let quad = self.AppendMaterial(QuadMaterial::DebugBorder {
            color,
            width: width as i32,
        });
        self.SetQuad(
            quad,
            state,
            rect,
            visible,
            color.alpha < 1.,
            QuadMaterial::DebugBorder {
                color,
                width: width as i32,
            },
        );
    }
}
impl<B: NativeFrameTypes + SolidColorLayerImplTypes> SolidColorRenderPass<B>
    for CompositorRenderPass<B>
{
    fn CreateAndAppendSharedQuadState(&mut self) -> SharedQuadStateHandle {
        self.AppendSharedQuadState()
    }
    fn SharedQuadState(&self, id: SharedQuadStateHandle) -> &SharedQuadState<B::FrameTransform> {
        self.SharedQuadState(id)
    }
    fn SharedQuadStateMutable(
        &mut self,
        id: SharedQuadStateHandle,
    ) -> &mut SharedQuadState<B::FrameTransform> {
        self.SharedQuadStateMutable(id)
    }
    fn CreateAndAppendSolidColorDrawQuad(&mut self) -> DrawQuadHandle {
        self.AppendMaterial(QuadMaterial::SolidColor {
            color: Color4f::default(),
            force_anti_aliasing_off: false,
        })
    }
    fn SolidColorDrawQuadSetNew(
        &mut self,
        quad: DrawQuadHandle,
        state: SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        mut color: Color4f,
        force_anti_aliasing_off: bool,
    ) {
        color.alpha = crate::adapter::geometry::cpp_clamp(color.alpha, 0., 1.);
        self.SetQuad(
            quad,
            state,
            rect,
            visible,
            color.alpha != 1.,
            QuadMaterial::SolidColor {
                color,
                force_anti_aliasing_off,
            },
        );
    }
}
impl<B: NativeFrameTypes + TileBasedLayerImplTypes> TileBasedRenderPass<B>
    for CompositorRenderPass<B>
{
    fn DrawQuad(&self, id: DrawQuadHandle) -> &DrawQuad<B::PictureDisplayItemList> {
        &self.quad_list[id.0]
    }
    fn CreateAndAppendDebugBorderDrawQuad(&mut self) -> DrawQuadHandle {
        self.AppendMaterial(QuadMaterial::DebugBorder {
            color: Color4f::default(),
            width: 0,
        })
    }
    fn DebugBorderDrawQuadSetNew(
        &mut self,
        quad: DrawQuadHandle,
        state: SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        color: Color4f,
        width: f32,
    ) {
        self.SetQuad(
            quad,
            state,
            rect,
            visible,
            color.alpha < 1.,
            QuadMaterial::DebugBorder {
                color,
                width: width as i32,
            },
        );
    }
    fn CreateAndAppendTileDrawQuad(&mut self) -> DrawQuadHandle {
        self.AppendMaterial(QuadMaterial::Tile {
            resource_id: ResourceId::default(),
            tex_coord_rect: RectF::default(),
            nearest_neighbor: false,
            force_anti_aliasing_off: false,
        })
    }
    fn TileDrawQuadSetNew(
        &mut self,
        quad: DrawQuadHandle,
        state: SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        needs_blending: bool,
        resource_id: ResourceId,
        tex_coord_rect: RectF,
        nearest_neighbor: bool,
        force_anti_aliasing_off: bool,
    ) {
        assert_ne!(
            resource_id,
            ResourceId::default(),
            "native TileDrawQuad resource must be valid"
        );
        self.SetQuad(
            quad,
            state,
            rect,
            visible,
            needs_blending,
            QuadMaterial::Tile {
                resource_id,
                tex_coord_rect,
                nearest_neighbor,
                force_anti_aliasing_off,
            },
        );
    }
}
impl<B: NativeFrameTypes + RenderSurfaceTypes> RenderSurfaceRenderPass<B>
    for CompositorRenderPass<B>
{
    fn SetNew(
        &mut self,
        id: RenderPassId,
        output: Rect,
        damage: Rect,
        transform: &B::FrameTransform,
    ) {
        self.SetNew(id, output, damage, transform);
    }
    fn SetFilters(&mut self, value: &B::FrameFilters) {
        self.filters = value.clone();
    }
    fn SetBackdropFilters(&mut self, value: &B::FrameFilters) {
        self.backdrop_filters = value.clone();
    }
    fn SetBackdropFilterBounds(&mut self, value: Option<B::FramePath>) {
        self.backdrop_filter_bounds = value;
    }
    fn set_generate_mipmap(&mut self, value: bool) {
        self.generate_mipmap = value;
    }
    fn set_subtree_size(&mut self, value: Size) {
        self.subtree_size = value;
    }
    fn subtree_size_mut(&mut self) -> &mut Size {
        &mut self.subtree_size
    }
    fn set_has_damage_from_contributing_content(&mut self, value: bool) {
        self.has_damage_from_contributing_content = value;
    }
    fn set_subtree_capture_id(&mut self, value: SubtreeCaptureId) {
        self.subtree_capture_id = value;
    }
    fn set_cache_render_pass(&mut self, value: bool) {
        self.cache_render_pass = value;
    }
    fn set_view_transition_element_resource_id(&mut self, value: ViewTransitionElementResourceId) {
        self.view_transition_element_resource_id = value;
    }
    fn CreateAndAppendRenderPassDrawQuad(&mut self) -> DrawQuadHandle {
        self.AppendMaterial(QuadMaterial::CompositorRenderPass {
            render_pass_id: RenderPassId::default(),
            mask_resource_id: ResourceId::default(),
            mask_uv_rect: RectF::default(),
            mask_texture_size: Size::default(),
            filters_scale: Vector2dF::new(1., 1.),
            filters_origin: PointF::default(),
            force_anti_aliasing_off: false,
            backdrop_filter_quality: 1.,
            intersects_damage_under: true,
        })
    }
    fn RenderPassDrawQuadSetAll(
        &mut self,
        quad: DrawQuadHandle,
        state: SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        needs_blending: bool,
        render_pass_id: RenderPassId,
        mask_resource_id: ResourceId,
        mask_uv_rect: RectF,
        mask_texture_size: Size,
        filters_scale: Vector2dF,
        filters_origin: PointF,
        force_anti_aliasing_off: bool,
        backdrop_filter_quality: f32,
        intersects_damage_under: bool,
    ) {
        self.SetQuad(
            quad,
            state,
            rect,
            visible,
            needs_blending,
            QuadMaterial::CompositorRenderPass {
                render_pass_id,
                mask_resource_id,
                mask_uv_rect,
                mask_texture_size,
                filters_scale,
                filters_origin,
                force_anti_aliasing_off,
                backdrop_filter_quality,
                intersects_damage_under,
            },
        );
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

/// Borrow-only native DrawQuad/shared-quad-state projection. A pass resolves
/// the handle at the point of iteration; neither reference is saved in a quad.
pub struct ThrottleQuadView<'a, T, L> {
    quad: &'a DrawQuad<L>,
    shared: &'a SharedQuadState<T>,
}
impl<L> crate::adapter::throttle::RenderPassQuad<RenderPassId> for DrawQuad<L> {
    fn render_pass_id(&self) -> RenderPassId {
        match self.material {
            QuadMaterial::CompositorRenderPass { render_pass_id, .. } => render_pass_id,
            _ => panic!("native render-pass quad material invariant"),
        }
    }
    fn resource_id(&self) -> ResourceId {
        LayerImplDrawQuad::resource_id(self)
    }
}
impl<T, F, L> crate::adapter::throttle::ThrottleQuad<T, RenderPassId, F>
    for ThrottleQuadView<'_, T, L>
{
    fn render_pass_quad(
        &self,
    ) -> Option<&dyn crate::adapter::throttle::RenderPassQuad<RenderPassId>> {
        matches!(
            self.quad.material,
            QuadMaterial::CompositorRenderPass { .. }
        )
        .then_some(self.quad)
    }
    fn surface_quad(&self) -> Option<&dyn crate::adapter::throttle::SurfaceQuad<F>> {
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
            .get(handle.0)
            .expect("native shared state belongs to this pass");
        Some(ThrottleQuadView { quad, shared })
    }
}
impl<V, F> crate::adapter::throttle::ThrottlePass<V::FrameTransform, RenderPassId, F>
    for CompositorRenderPass<V>
where
    V: FrameValueTypes,
    V::FrameFilters: crate::adapter::draw_property::DrawFilters,
    V::FramePath: crate::adapter::geometry::Path,
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
        crate::adapter::draw_property::DrawFilters::HasBlurFilter(&self.filters)
    }
    fn backdrop_filters_have_blur(&self) -> bool {
        crate::adapter::draw_property::DrawFilters::HasBlurFilter(&self.backdrop_filters)
    }
    fn output_rect(&self) -> Rect {
        self.output_rect
    }
    fn backdrop_filter_bounds(&self) -> Option<RectF> {
        self.backdrop_filter_bounds
            .as_ref()
            .map(crate::adapter::geometry::Path::bounds)
    }
    fn quads(&self) -> Self::Quads<'_> {
        ThrottleQuads {
            pass: self,
            iter: self.quad_list.iter(),
        }
    }
}
impl<V: FrameValueTypes> crate::adapter::layer_tree_host_impl::HostMetadataPass
    for CompositorRenderPass<V>
{
    fn has_transparent_background(&self) -> bool {
        self.has_transparent_background
    }
    fn quad_resource_id(&self, index: usize) -> ResourceId {
        LayerImplDrawQuad::resource_id(&self.quad_list[index])
    }
}
impl<B> crate::adapter::layer_tree_host_impl::HostDrawPass<B> for CompositorRenderPass<B>
where
    B: NativeFrameTypes + crate::adapter::layer_tree_host_impl::HostImplTypes,
{
    fn Create() -> Box<Self> {
        Box::new(Self::default())
    }
    fn SetNew(
        &mut self,
        id: RenderPassId,
        output: Rect,
        damage: Rect,
        transform: B::FrameTransform,
    ) {
        Self::SetNew(self, id, output, damage, &transform);
    }
    fn set_damage_rect(&mut self, rect: Rect) {
        self.damage_rect = rect;
    }
    fn set_has_transparent_background(&mut self, value: bool) {
        self.has_transparent_background = value;
    }
    fn copy_requests(&self) -> &Vec<Box<B::FrameCopyRequest>> {
        &self.copy_requests
    }
    fn copy_requests_mut(&mut self) -> &mut Vec<Box<B::FrameCopyRequest>> {
        &mut self.copy_requests
    }
    fn quad_count(&self) -> usize {
        self.quad_list.len()
    }
    fn quad_material(
        &self,
        index: usize,
    ) -> crate::adapter::layer_tree_host_impl::HostQuadMaterial {
        use crate::adapter::layer_tree_host_impl::HostQuadMaterial::*;
        match self.quad_list[index].material {
            QuadMaterial::SharedElement { .. } => kSharedElement,
            QuadMaterial::CompositorRenderPass { .. } => kCompositorRenderPass,
            _ => Other,
        }
    }
    fn quad_shared_element_resource_id(&self, index: usize) -> ViewTransitionElementResourceId {
        match self.quad_list[index].material {
            QuadMaterial::SharedElement {
                element_resource_id,
            } => element_resource_id,
            _ => panic!("native shared element quad material invariant"),
        }
    }
    fn quad_render_pass_id(&self, index: usize) -> Option<RenderPassId> {
        match self.quad_list[index].material {
            QuadMaterial::CompositorRenderPass { render_pass_id, .. } => Some(render_pass_id),
            _ => None,
        }
    }
    fn EraseQuadAndInvalidateAllPointers(&mut self, index: usize) {
        self.quad_list.remove(index);
    }
    fn quad_has_shared_quad_state(&self, index: usize) -> bool {
        self.quad_list[index].geometry.shared_quad_state.is_some()
    }
    fn subtree_capture_id_is_valid(&self) -> bool {
        self.subtree_capture_id.is_valid()
    }
    fn filters_is_empty(&self) -> bool {
        crate::adapter::property_types::FilterOperations::IsEmpty(&self.filters)
    }
    fn backdrop_filters_is_empty(&self) -> bool {
        crate::adapter::property_types::FilterOperations::IsEmpty(&self.backdrop_filters)
    }
    fn view_transition_element_resource_id(&self) -> ViewTransitionElementResourceId {
        self.view_transition_element_resource_id
    }
    fn CreateAndAppendSharedQuadState(&mut self) -> SharedQuadStateHandle {
        self.AppendSharedQuadState()
    }
    fn SetSharedQuadStateAll(
        &mut self,
        state: SharedQuadStateHandle,
        transform: B::FrameTransform,
        rect: Rect,
        visible: Rect,
        mask: MaskFilterInfo,
        clip: Option<Rect>,
        opaque: bool,
        opacity: f32,
        blend: SkBlendMode,
        sorting_context: i32,
        layer_id: u32,
        fast: bool,
    ) {
        self.shared_quad_state_list[state.0].SetAll(
            &transform,
            rect,
            visible,
            &mask,
            clip,
            opaque,
            opacity,
            blend,
            sorting_context,
            layer_id,
            fast,
        );
    }
    fn CreateAndAppendSolidColorDrawQuad(&mut self) -> DrawQuadHandle {
        self.AppendMaterial(QuadMaterial::SolidColor {
            color: Color4f::default(),
            force_anti_aliasing_off: false,
        })
    }
    fn SolidColorDrawQuadSetNew(
        &mut self,
        quad: DrawQuadHandle,
        state: SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        mut color: Color4f,
        force_anti_aliasing_off: bool,
    ) {
        color.alpha = crate::adapter::geometry::cpp_clamp(color.alpha, 0., 1.);
        self.SetQuad(
            quad,
            state,
            rect,
            visible,
            color.alpha != 1.,
            QuadMaterial::SolidColor {
                color,
                force_anti_aliasing_off,
            },
        );
    }
}
impl<B> crate::adapter::layer_tree_host_impl::HostCompositorFrame<B>
    for CompositorFrame<
        B::CompositorFrameMetadata,
        CompositorRenderPass<B>,
        B::TransferableResource,
    >
where
    B: NativeFrameTypes + crate::adapter::layer_tree_host_impl::HostImplDTypes,
{
    fn metadata(&self) -> &B::CompositorFrameMetadata {
        &self.metadata
    }
    fn metadata_mut(&mut self) -> &mut B::CompositorFrameMetadata {
        &mut self.metadata
    }
    fn set_metadata(&mut self, value: B::CompositorFrameMetadata) {
        self.metadata = value;
    }
    fn render_passes(&self) -> &[Box<CompositorRenderPass<B>>] {
        &self.render_pass_list
    }
    fn set_render_passes(&mut self, value: Vec<Box<CompositorRenderPass<B>>>) {
        self.render_pass_list = value;
    }
    fn resource_list_mut(&mut self) -> &mut Vec<B::TransferableResource> {
        &mut self.resource_list
    }
}
impl crate::adapter::layer_tree_host_impl::HostMetadataBeginFrameAck for BeginFrameAck {
    fn frame_id(&self) -> crate::adapter::begin_frame::BeginFrameId {
        self.frame_id
    }
    fn IsSequenceValid(&self) -> bool {
        self.frame_id.sequence_number
            >= crate::adapter::begin_frame::BeginFrameArgs::kStartingFrameNumber
    }
}

/// Narrow external Viz/gfx value function. This is independent of Skia and
/// carries no compositor owner, tree, frame lifecycle or layerization policy.
pub trait ViewTransitionGeometry: FrameValueTypes {
    fn GetViewTransitionTransform(shared_element_quad: Rect, output: Rect) -> Self::FrameTransform;
}

impl<B> crate::adapter::picture_layer_impl::PictureRenderPass<B> for CompositorRenderPass<B>
where B:NativeFrameTypes+crate::adapter::picture_layer_impl::PictureLayerImplTypes+FrameValueTypes<PictureDisplayItemList=<<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList>
{
 fn CreateAndAppendPictureDrawQuad(&mut self)->DrawQuadHandle {
  // Native default construction before SetNew. No fabricated display content.
  self.AppendMaterial(QuadMaterial::Picture{tex_coord_rect:RectF::default(),nearest_neighbor:false,content_rect:Rect::default(),contents_scale:0.0,image_animation_map:Default::default(),display_item_list:None,raster_inducing_scroll_offsets:Default::default()})
 }
 fn PictureDrawQuadSetNew(&mut self,quad:DrawQuadHandle,shared:SharedQuadStateHandle,geometry:Rect,visible:Rect,needs_blending:bool,texture:RectF,nearest:bool,quad_content:Rect,scale:f32,image_animation:std::collections::BTreeMap<crate::adapter::image_animation::PaintImageId,usize>,display_list:Option<Arc<<<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList>>,scroll_offsets:crate::adapter::prioritized_tile::ScrollOffsetMap){
  self.SetQuad(quad,shared,geometry,visible,needs_blending,QuadMaterial::Picture{tex_coord_rect:texture,nearest_neighbor:nearest,content_rect:quad_content,contents_scale:scale,image_animation_map:image_animation,display_item_list:display_list,raster_inducing_scroll_offsets:scroll_offsets});
 }
}
