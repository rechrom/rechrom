//! Draw-quad values carried by a CompositorFrame to Viz
//! (components/viz/common/quads). Producers append them; Viz and the renderer
//! read them. No layerization, tiling or draw-order logic lives here.
use crate::ResourceId;
use compositor_tree::geometry::{PointF, Rect, RectF, Size, Vector2dF};
use compositor_values::color4f::Color4f;
use compositor_values::prioritized_tile::ScrollOffsetMap;
use compositor_values::surface_id::SurfaceRange;
use compositor_tree::property_types::{
    BlendMode as SkBlendMode, MaskFilterInfo, ViewTransitionElementResourceId,
};
use image_resource::PaintImageId;
use std::sync::Arc;

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
impl DrawQuadHandle {
    /// Allocated by the producing pass for its next quad.
    pub const fn FromIndex(index: usize) -> Self {
        Self(index)
    }

    pub const fn Index(self) -> usize {
        self.0
    }
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
        image_animation_map: std::collections::BTreeMap<PaintImageId, usize>,
        display_item_list: Option<Arc<L>>,
        raster_inducing_scroll_offsets: ScrollOffsetMap,
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
    /// Source: components/viz/common/quads/texture_draw_quad.h. Modern video
    /// frames use this material with one multiplanar SharedImage resource;
    /// kYuvVideoContent was removed upstream.
    Texture {
        resource_id: ResourceId,
        /// Source: TextureDrawQuad::is_tex_coord_rect_normalized. Video uses
        /// normalized coordinates; painted scrollbar UI resources use pixels.
        tex_coord_rect: RectF,
        is_tex_coord_rect_normalized: bool,
        background_color: Color4f,
        nearest_neighbor: bool,
        secure_output_only: bool,
        is_video_frame: bool,
        force_rgbx: bool,
    },
    /// Source: components/viz/common/quads/surface_draw_quad.h:16-60.
    /// The referenced frame remains owned by Viz; this quad only carries the
    /// immutable range and embedding policy emitted by cc::SurfaceLayerImpl.
    Surface {
        surface_range: SurfaceRange,
        default_background_color: Color4f,
        stretch_content_to_fill_bounds: bool,
        is_reflection: bool,
        allow_merge: bool,
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
            Self::Texture {
                resource_id,
                tex_coord_rect,
                is_tex_coord_rect_normalized,
                background_color,
                nearest_neighbor,
                secure_output_only,
                is_video_frame,
                force_rgbx,
            } => Self::Texture {
                resource_id: *resource_id,
                tex_coord_rect: *tex_coord_rect,
                is_tex_coord_rect_normalized: *is_tex_coord_rect_normalized,
                background_color: *background_color,
                nearest_neighbor: *nearest_neighbor,
                secure_output_only: *secure_output_only,
                is_video_frame: *is_video_frame,
                force_rgbx: *force_rgbx,
            },
            Self::Surface {
                surface_range,
                default_background_color,
                stretch_content_to_fill_bounds,
                is_reflection,
                allow_merge,
            } => Self::Surface {
                surface_range: *surface_range,
                default_background_color: *default_background_color,
                stretch_content_to_fill_bounds: *stretch_content_to_fill_bounds,
                is_reflection: *is_reflection,
                allow_merge: *allow_merge,
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
impl<L> DrawQuad<L> {
    /// The single resource a quad references: the tile or texture resource, or
    /// a render-pass quad's mask resource.
    pub fn resource_id(&self) -> ResourceId {
        match self.material {
            QuadMaterial::Tile { resource_id, .. } | QuadMaterial::Texture { resource_id, .. } => {
                resource_id
            }
            QuadMaterial::CompositorRenderPass {
                mask_resource_id, ..
            } => mask_resource_id,
            _ => ResourceId::default(),
        }
    }
}
