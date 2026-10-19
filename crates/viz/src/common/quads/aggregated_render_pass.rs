// Source: components/viz/common/quads/aggregated_render_pass.h:1-40
// Source: components/viz/common/quads/aggregated_render_pass.cc:1-33
// Includes/export macros map to Rust imports and modules. Copy-output request
// payloads remain Renderer capabilities; pass/quads and copy algorithms live here.
#![allow(non_snake_case)]

use compositor_core::adapter::{geometry::Rect, layer_tree_host_impl::ContentColorUsage};
use compositor_core::runtime::{
    frame::{DrawQuad, FrameValueTypes, QuadMaterial, RenderPassId},
    skia::SkRegion,
};
use compositor_core::NativeCompositorTypes;

pub type NativeTransform = compositor_core::runtime::geometry::Transform<SkRegion>;
pub type NativeSharedQuadState = compositor_core::runtime::frame::SharedQuadState<NativeTransform>;
pub type NativeDrawQuad =
    DrawQuad<<NativeCompositorTypes as FrameValueTypes>::PictureDisplayItemList>;
pub type NativeFilters = <NativeCompositorTypes as FrameValueTypes>::FrameFilters;
pub type NativePath = <NativeCompositorTypes as FrameValueTypes>::FramePath;

// Source: components/viz/common/quads/aggregated_render_pass.h:41-65
// Source: components/viz/common/quads/aggregated_render_pass.cc:34-86
#[derive(Clone)]
pub struct AggregatedRenderPass {
    pub id: u64,
    pub output_rect: Rect,
    pub damage_rect: Rect,
    pub transform_to_root_target: NativeTransform,
    pub content_color_usage: ContentColorUsage,
    pub has_transparent_background: bool,
    pub cache_render_pass: bool,
    pub has_damage_from_contributing_content: bool,
    pub generate_mipmap: bool,
    pub is_from_surface_root_pass: bool,
    pub video_capture_enabled: bool,
    pub filters: NativeFilters,
    pub backdrop_filters: NativeFilters,
    pub backdrop_filter_bounds: Option<NativePath>,
    pub shared_quad_state_list: Vec<NativeSharedQuadState>,
    pub quad_list: Vec<NativeDrawQuad>,
}
impl Default for AggregatedRenderPass {
    fn default() -> Self {
        Self {
            id: 0,
            output_rect: Rect::default(),
            damage_rect: Rect::default(),
            transform_to_root_target: NativeTransform::default(),
            content_color_usage: ContentColorUsage::kSRGB,
            has_transparent_background: true,
            cache_render_pass: false,
            has_damage_from_contributing_content: false,
            generate_mipmap: false,
            is_from_surface_root_pass: false,
            video_capture_enabled: false,
            filters: NativeFilters::default(),
            backdrop_filters: NativeFilters::default(),
            backdrop_filter_bounds: None,
            shared_quad_state_list: Vec::new(),
            quad_list: Vec::new(),
        }
    }
}
impl AggregatedRenderPass {
    pub fn new(shared_quad_state_size: usize, draw_quad_size: usize) -> Self {
        let mut pass = Self::default();
        pass.shared_quad_state_list.reserve(shared_quad_state_size);
        pass.quad_list.reserve(draw_quad_size);
        pass
    }
    pub fn SetNew(&mut self, id: u64, output: Rect, damage: Rect, transform: NativeTransform) {
        assert_ne!(id, 0);
        assert!(damage.IsEmpty() || output.Contains(damage));
        self.id = id;
        self.output_rect = output;
        self.damage_rect = damage;
        self.transform_to_root_target = transform;
        assert!(self.quad_list.is_empty() && self.shared_quad_state_list.is_empty());
    }
    #[allow(clippy::too_many_arguments)]
    pub fn SetAll(
        &mut self,
        id: u64,
        output: Rect,
        damage: Rect,
        transform: NativeTransform,
        color_usage: ContentColorUsage,
        transparent: bool,
        cache: bool,
        contributing_damage: bool,
        mipmap: bool,
    ) {
        assert_ne!(id, 0);
        self.id = id;
        self.output_rect = output;
        self.damage_rect = damage;
        self.transform_to_root_target = transform;
        self.content_color_usage = color_usage;
        self.has_transparent_background = transparent;
        self.cache_render_pass = cache;
        self.has_damage_from_contributing_content = contributing_damage;
        self.generate_mipmap = mipmap;
        assert!(self.quad_list.is_empty() && self.shared_quad_state_list.is_empty());
    }

    // Source: components/viz/common/quads/aggregated_render_pass.h:66-90
    // Source: components/viz/common/quads/aggregated_render_pass.cc:87-153
    pub fn CopyFromAndAppendRenderPassDrawQuad(
        &mut self,
        quad: &NativeDrawQuad,
        render_pass_id: RenderPassId,
    ) -> &NativeDrawQuad {
        assert!(!self.shared_quad_state_list.is_empty());
        let mut copy = quad.clone();
        match &mut copy.material {
            QuadMaterial::CompositorRenderPass {
                render_pass_id: id, ..
            } => {
                *id = render_pass_id;
            }
            _ => panic!("expected CompositorRenderPass quad"),
        }
        copy.geometry.shared_quad_state = Some(
            compositor_core::runtime::frame::SharedQuadStateHandle::FromIndex(
                self.shared_quad_state_list.len() - 1,
            ),
        );
        self.quad_list.push(copy);
        self.quad_list.last().expect("appended quad")
    }
    pub fn CopyFromAndAppendDrawQuad(&mut self, quad: &NativeDrawQuad) -> &NativeDrawQuad {
        assert!(!self.shared_quad_state_list.is_empty());
        assert!(!matches!(quad.material, QuadMaterial::SharedElement { .. }));
        let mut copy = quad.clone();
        copy.geometry.shared_quad_state = Some(
            compositor_core::runtime::frame::SharedQuadStateHandle::FromIndex(
                self.shared_quad_state_list.len() - 1,
            ),
        );
        self.quad_list.push(copy);
        self.quad_list.last().expect("appended quad")
    }

    // Source: components/viz/common/quads/aggregated_render_pass.h:76-96
    // Source: components/viz/common/quads/aggregated_render_pass.cc:154-217
    pub fn Copy(&self, new_id: u64) -> Self {
        let mut copy = Self::new(self.shared_quad_state_list.len(), self.quad_list.len());
        copy.SetAll(
            new_id,
            self.output_rect,
            self.damage_rect,
            self.transform_to_root_target.clone(),
            self.content_color_usage,
            self.has_transparent_background,
            self.cache_render_pass,
            self.has_damage_from_contributing_content,
            self.generate_mipmap,
        );
        copy
    }
    pub fn DeepCopy(&self) -> Self {
        let mut copy = self.Copy(self.id);
        copy.shared_quad_state_list = self.shared_quad_state_list.clone();
        copy.quad_list = self.quad_list.clone();
        copy.is_from_surface_root_pass = self.is_from_surface_root_pass;
        copy.video_capture_enabled = self.video_capture_enabled;
        copy.filters = self.filters.clone();
        copy.backdrop_filters = self.backdrop_filters.clone();
        copy.backdrop_filter_bounds = self.backdrop_filter_bounds.clone();
        copy
    }
    pub fn ShouldDrawWithBlending(&self) -> bool {
        self.quad_list
            .iter()
            .any(|quad| quad.geometry.needs_blending)
    }
    pub fn HasCapture(&self) -> bool {
        self.video_capture_enabled
    }
    pub fn RenderPassId(&self) -> RenderPassId {
        RenderPassId::from(self.id)
    }
    pub fn OutputRect(&self) -> Rect {
        self.output_rect
    }
    pub fn SharedQuadStates(&self) -> &[NativeSharedQuadState] {
        &self.shared_quad_state_list
    }
    pub fn TrySharedQuadState(
        &self,
        handle: compositor_core::runtime::frame::SharedQuadStateHandle,
    ) -> Option<&NativeSharedQuadState> {
        self.shared_quad_state_list.get(handle.Index())
    }
    pub fn Quads(&self) -> &[NativeDrawQuad] {
        &self.quad_list
    }
    pub fn DamageRect(&self) -> Rect {
        self.damage_rect
    }
    pub fn Filters(&self) -> &NativeFilters {
        &self.filters
    }
    pub fn BackdropFilters(&self) -> &NativeFilters {
        &self.backdrop_filters
    }
    pub fn BackdropFilterBounds(&self) -> Option<&NativePath> {
        self.backdrop_filter_bounds.as_ref()
    }
}

// Source: components/viz/common/quads/aggregated_render_pass.h:97-136
// Source: components/viz/common/quads/aggregated_render_pass.cc:218-240
// Windows DComp flags, trace dictionaries and copy-request capture are emitted
// by platform adapters. The portable color/root/capture fields are retained.

// Source: components/viz/common/quads/aggregated_render_pass.h:137-152
// Source: components/viz/common/quads/aggregated_render_pass.cc:241
pub type AggregatedRenderPassList = Vec<AggregatedRenderPass>;
