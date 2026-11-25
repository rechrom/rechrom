//! The cc producer facets of the viz frame values. The pass, quad and frame
//! values are components/viz/common (`viz_common`); the canonical cc owners
//! (layers, render surfaces, LayerTreeHostImpl) append into them through the
//! facade traits implemented here. Throttling and frame-data facets live with
//! their traits in compositor_draw / compositor_frame.
pub use crate::adapter::layer_tree_impl::ResourceId;
pub use crate::adapter::scheduler_host::BeginFrameAck;
use crate::adapter::{
    frame_data::FrameDataTypes,
    geometry::{PointF, Rect, RectF, Size, Vector2dF},
    layer_impl::{
        LayerImplDrawQuad, LayerImplRenderPass, LayerImplSharedQuadState, LayerImplTypes,
    },
    layer_tree_impl::Color4f,
    occlusion::SkBlendMode,
    property_types::{MaskFilterInfo, SubtreeCaptureId, ViewTransitionElementResourceId},
    render_surface_impl::{RenderSurfaceRenderPass, RenderSurfaceTypes},
    solid_color_layer_impl::{SolidColorLayerImplTypes, SolidColorRenderPass},
    surface_layer_impl::{SurfaceLayerImplTypes, SurfaceRenderPass},
    tile_based_layer_impl::{
        TileBasedLayerImplTypes, TileBasedRenderPass, TileBasedSharedQuadState,
    },
};
use layerize::commit::CommitTypes;
use std::sync::Arc;
pub use viz_common::compositor_render_pass::{
    CompositorFrame, CompositorRenderPass, FrameValueTypes, FrameValues,
};
pub use viz_common::draw_quad::{
    DrawQuad, DrawQuadHandle, QuadGeometry, QuadMaterial, RenderPassId, SharedQuadState,
    SharedQuadStateHandle,
};

/// The frame value family of one compositor family.
type Values<B> = <B as FrameTypes>::FrameValues;
type DisplayList<B> = <Values<B> as FrameValueTypes>::PictureDisplayItemList;

/// Binds the root family to the viz frame values it produces: a pass is
/// `CompositorRenderPass<Self::FrameValues>`, whose value types are exactly
/// the family's commit values (as cc::LayerTreeHostImpl fills a
/// viz::CompositorRenderPass with its gfx::Transform and FilterOperations).
pub trait FrameTypes:
    LayerImplTypes<
        SharedQuadState = SharedQuadState<<Self as CommitTypes>::Transform>,
        DrawQuad = DrawQuad<
            <<Self as FrameTypes>::FrameValues as FrameValueTypes>::PictureDisplayItemList,
        >,
    > + FrameDataTypes<
        RenderPassId = RenderPassId,
        SharedQuadStateHandle = SharedQuadStateHandle,
        DrawQuadHandle = DrawQuadHandle,
        RenderPass = CompositorRenderPass<<Self as FrameTypes>::FrameValues>,
    >
{
    type FrameValues: FrameValueTypes<
        FrameTransform = <Self as CommitTypes>::Transform,
        FrameFilters = <Self as CommitTypes>::Filters,
        FramePath = <Self as CommitTypes>::Path,
        FrameCopyRequest = <Self as CommitTypes>::CopyOutputRequest,
    >;
}
impl<T: Clone> LayerImplSharedQuadState<T> for SharedQuadState<T> {
    fn opacity(&self) -> f32 {
        self.opacity
    }
    fn MultiplyOpacity(&mut self, value: f32) {
        self.opacity *= value;
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
impl<L> LayerImplDrawQuad for DrawQuad<L> {
    fn resource_id(&self) -> ResourceId {
        DrawQuad::resource_id(self)
    }
}
impl<B: FrameTypes> LayerImplRenderPass<B> for CompositorRenderPass<Values<B>> {
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
impl<B: FrameTypes + SolidColorLayerImplTypes> SolidColorRenderPass<B>
    for CompositorRenderPass<Values<B>>
{
    fn CreateAndAppendSharedQuadState(&mut self) -> SharedQuadStateHandle {
        self.AppendSharedQuadState()
    }
    fn SharedQuadState(
        &self,
        id: SharedQuadStateHandle,
    ) -> &SharedQuadState<<B as CommitTypes>::Transform> {
        CompositorRenderPass::SharedQuadState(self, id)
    }
    fn SharedQuadStateMutable(
        &mut self,
        id: SharedQuadStateHandle,
    ) -> &mut SharedQuadState<<B as CommitTypes>::Transform> {
        CompositorRenderPass::SharedQuadStateMutable(self, id)
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
    fn AppendScrollbarTextureQuad(
        &mut self,
        state: SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        resource: ResourceId,
        tex_coord_rect: RectF,
        background: Color4f,
    ) {
        self.AppendTextureQuad(
            state,
            rect,
            visible,
            true,
            resource,
            tex_coord_rect,
            false,
            background,
            false,
            false,
            false,
            false,
        );
    }
}
impl<B> SurfaceRenderPass<B> for CompositorRenderPass<Values<B>>
where
    B: FrameTypes
        + SurfaceLayerImplTypes<
            SurfaceRange = crate::runtime::types::SurfaceRange,
            SurfaceId = crate::runtime::types::SurfaceId,
        >,
{
    fn AppendSurfaceDrawQuad(
        &mut self,
        state: B::SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        range: &B::SurfaceRange,
        background_color: Color4f,
        stretch_content_to_fill_bounds: bool,
        is_reflection: bool,
        allow_merge: bool,
    ) {
        self.AppendSurfaceQuad(
            state,
            rect,
            visible,
            true,
            *range,
            background_color,
            stretch_content_to_fill_bounds,
            is_reflection,
            allow_merge,
        );
    }
}
impl<B: FrameTypes + TileBasedLayerImplTypes> TileBasedRenderPass<B>
    for CompositorRenderPass<Values<B>>
{
    fn DrawQuad(&self, id: DrawQuadHandle) -> &DrawQuad<DisplayList<B>> {
        &self.quad_list[id.Index()]
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
impl<B: FrameTypes + RenderSurfaceTypes> RenderSurfaceRenderPass<B>
    for CompositorRenderPass<Values<B>>
{
    fn SetNew(
        &mut self,
        id: RenderPassId,
        output: Rect,
        damage: Rect,
        transform: &<B as CommitTypes>::Transform,
    ) {
        CompositorRenderPass::SetNew(self, id, output, damage, transform);
    }
    fn SetFilters(&mut self, value: &<B as CommitTypes>::Filters) {
        self.filters = value.clone();
    }
    fn SetBackdropFilters(&mut self, value: &<B as CommitTypes>::Filters) {
        self.backdrop_filters = value.clone();
    }
    fn SetBackdropFilterBounds(&mut self, value: Option<<B as CommitTypes>::Path>) {
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
impl<B> crate::adapter::layer_tree_host_impl::HostDrawPass<B> for CompositorRenderPass<Values<B>>
where
    B: FrameTypes + crate::adapter::layer_tree_host_impl::HostImplTypes,
    <B as CommitTypes>::Filters: crate::adapter::throttle::ThrottleFilters,
{
    fn Create() -> Box<Self> {
        Box::new(Self::default())
    }
    fn SetNew(
        &mut self,
        id: RenderPassId,
        output: Rect,
        damage: Rect,
        transform: <B as CommitTypes>::Transform,
    ) {
        CompositorRenderPass::SetNew(self, id, output, damage, &transform);
    }
    fn set_damage_rect(&mut self, rect: Rect) {
        self.damage_rect = rect;
    }
    fn set_has_transparent_background(&mut self, value: bool) {
        self.has_transparent_background = value;
    }
    fn copy_requests(&self) -> &Vec<Box<<B as CommitTypes>::CopyOutputRequest>> {
        &self.copy_requests
    }
    fn copy_requests_mut(&mut self) -> &mut Vec<Box<<B as CommitTypes>::CopyOutputRequest>> {
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
        transform: <B as CommitTypes>::Transform,
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
        self.shared_quad_state_list[state.Index()].SetAll(
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
        CompositorRenderPass<Values<B>>,
        B::TransferableResource,
    >
where
    B: FrameTypes + crate::adapter::layer_tree_host_impl::HostImplDTypes,
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
    fn render_passes(&self) -> &[Box<CompositorRenderPass<Values<B>>>] {
        &self.render_pass_list
    }
    fn set_render_passes(&mut self, value: Vec<Box<CompositorRenderPass<Values<B>>>>) {
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
pub trait ViewTransitionGeometry {
    type ViewTransitionTransform;
    fn GetViewTransitionTransform(
        shared_element_quad: Rect,
        output: Rect,
    ) -> Self::ViewTransitionTransform;
}

impl<B> crate::adapter::picture_layer_impl::PictureRenderPass<B> for CompositorRenderPass<Values<B>>
where B:FrameTypes+crate::adapter::picture_layer_impl::PictureLayerImplTypes,
      Values<B>: FrameValueTypes<PictureDisplayItemList=<<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList>
{
 fn CreateAndAppendPictureDrawQuad(&mut self)->DrawQuadHandle {
  // Native default construction before SetNew. No fabricated display content.
  self.AppendMaterial(QuadMaterial::Picture{tex_coord_rect:RectF::default(),nearest_neighbor:false,content_rect:Rect::default(),contents_scale:0.0,image_animation_map:Default::default(),display_item_list:None,raster_inducing_scroll_offsets:Default::default()})
 }
 fn PictureDrawQuadSetNew(&mut self,quad:DrawQuadHandle,shared:SharedQuadStateHandle,geometry:Rect,visible:Rect,needs_blending:bool,texture:RectF,nearest:bool,quad_content:Rect,scale:f32,image_animation:std::collections::BTreeMap<crate::adapter::image_animation::PaintImageId,usize>,display_list:Option<Arc<<<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as crate::adapter::picture_layer_impl::PictureRasterSource>::DisplayItemList>>,scroll_offsets:crate::adapter::prioritized_tile::ScrollOffsetMap){
  self.SetQuad(quad,shared,geometry,visible,needs_blending,QuadMaterial::Picture{tex_coord_rect:texture,nearest_neighbor:nearest,content_rect:quad_content,contents_scale:scale,image_animation_map:image_animation,display_item_list:display_list,raster_inducing_scroll_offsets:scroll_offsets});
 }
}
