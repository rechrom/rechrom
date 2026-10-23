//! Platform-independent host boundary for translated DirectRenderer.
use crate::components::viz::{
    common::display::renderer_settings::{DebugRendererSettings, RendererSettings},
    service::display::{
        direct_renderer::{DirectRenderer, DirectRendererBackend},
        occlusion_culler::OcclusionCuller,
    },
};
use std::io;
use viz::native::VizOutput;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderUpdate {
    pub direct_root_target: bool,
    pub passes: usize,
    pub quads: usize,
    pub damage_pixels: usize,
    pub tile_quads: usize,
    pub picture_quads: usize,
    pub solid_quads: usize,
    pub render_pass_quads: usize,
    pub tile_ns: u64,
    pub tile_image_ns: u64,
    pub tile_draw_ns: u64,
    pub tile_source_pixels: u64,
    pub tile_target_pixels: u64,
    pub tile_linear_quads: usize,
    pub tile_fractional_source_quads: usize,
    pub tile_scaled_quads: usize,
    pub tile_blended_quads: usize,
    pub tile_integer_translation_quads: usize,
    pub tile_blit_eligible_quads: usize,
    pub tile_blit_quads: usize,
    pub picture_ns: u64,
    pub solid_ns: u64,
    pub render_pass_ns: u64,
    pub render_pass_setup_ns: u64,
    pub render_pass_blit_ns: u64,
    pub render_pass_restore_ns: u64,
    pub backdrop_readback_ns: u64,
    pub backdrop_filter_ns: u64,
    pub backdrop_filter_alloc_ns: u64,
    pub backdrop_filter_build_ns: u64,
    pub backdrop_filter_snapshot_ns: u64,
    pub backdrop_filter_draw_ns: u64,
    pub backdrop_filter_restore_ns: u64,
    pub backdrop_filter_finish_ns: u64,
    pub backdrop_clip_ns: u64,
    pub backdrop_composite_ns: u64,
    pub render_pass_blit_quads: usize,
    pub render_pass_pixels: u64,
    pub transparent_render_pass_quads: usize,
    pub target_create_ns: u64,
    pub target_clear_ns: u64,
    pub target_finish_ns: u64,
}

/// Backend-owned target and storage keep the management core independent of
/// backend storage layouts or a concrete drawing library. The aggregate is borrowed.
pub struct Renderer<B: DirectRendererBackend> {
    direct: DirectRenderer<B>,
    occlusion_culler: OcclusionCuller,
}
impl<B: DirectRendererBackend> Renderer<B> {
    pub fn new(mut backend: B) -> Self {
        let settings = RendererSettings {
            partial_swap_enabled: true,
            ..Default::default()
        };
        backend.Configure(&settings);
        let mut direct = DirectRenderer::new(settings, DebugRendererSettings::default(), backend);
        direct.Initialize();
        direct.SetVisible(true);
        let occlusion_culler =
            OcclusionCuller::new(direct.settings.occlusion_culler_settings.clone());
        Self {
            direct,
            occlusion_culler,
        }
    }
    pub fn Render(
        &mut self,
        input: &mut VizOutput,
        target: B::Target,
    ) -> io::Result<(RenderUpdate, B::Target)> {
        let started = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        let viewport = self.direct.backend.ValidateTarget(input, &target)?;
        let validated_at = started.map(|_| std::time::Instant::now());
        let cull = self.occlusion_culler.RemoveOverdrawQuads(input.FrameMut());
        let culled_at = started.map(|_| std::time::Instant::now());
        if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
            eprintln!(
                "occlusion-profile quads_before={} quads_after={} fully_culled={} reduced={} split={} opaque_groups={}",
                cull.quads_before,
                cull.quads_after,
                cull.fully_culled,
                cull.reduced,
                cull.split,
                cull.opaque_groups,
            );
        }
        let output = {
            let _draw = browser_tracing::span("renderer", "Renderer.DrawFrame");
            self.direct.DrawFrame(input, viewport, target)?
        };
        if let Some(started) = started {
            let update = output.0;
            eprintln!(
                "renderer-profile total_ms={:.3} validate_ms={:.3} cull_ms={:.3} draw_ms={:.3} direct_root={} passes={} quads={} damage_pixels={} tiles={} tile_ms={:.3} tile_image_ms={:.3} tile_draw_ms={:.3} tile_source_pixels={} tile_target_pixels={} tile_linear={} tile_fractional_source={} tile_scaled={} tile_blended={} tile_integer_translation={} tile_blit_eligible={} tile_blit={} pictures={} picture_ms={:.3} solids={} solid_ms={:.3} render_passes={} render_pass_ms={:.3} render_pass_setup_ms={:.3} render_pass_blit_ms={:.3} render_pass_restore_ms={:.3} backdrop_readback_ms={:.3} backdrop_filter_ms={:.3} backdrop_filter_alloc_ms={:.3} backdrop_filter_build_ms={:.3} backdrop_filter_snapshot_ms={:.3} backdrop_filter_draw_ms={:.3} backdrop_filter_restore_ms={:.3} backdrop_filter_finish_ms={:.3} backdrop_clip_ms={:.3} backdrop_composite_ms={:.3} render_pass_blit={} render_pass_pixels={} transparent_render_passes={} target_create_ms={:.3} target_clear_ms={:.3} target_finish_ms={:.3}",
                started.elapsed().as_secs_f64() * 1000.,
                (validated_at.unwrap() - started).as_secs_f64() * 1000.,
                (culled_at.unwrap() - validated_at.unwrap()).as_secs_f64() * 1000.,
                culled_at.unwrap().elapsed().as_secs_f64() * 1000.,
                update.direct_root_target,
                update.passes,
                update.quads,
                update.damage_pixels,
                update.tile_quads,
                update.tile_ns as f64 / 1_000_000.,
                update.tile_image_ns as f64 / 1_000_000.,
                update.tile_draw_ns as f64 / 1_000_000.,
                update.tile_source_pixels,
                update.tile_target_pixels,
                update.tile_linear_quads,
                update.tile_fractional_source_quads,
                update.tile_scaled_quads,
                update.tile_blended_quads,
                update.tile_integer_translation_quads,
                update.tile_blit_eligible_quads,
                update.tile_blit_quads,
                update.picture_quads,
                update.picture_ns as f64 / 1_000_000.,
                update.solid_quads,
                update.solid_ns as f64 / 1_000_000.,
                update.render_pass_quads,
                update.render_pass_ns as f64 / 1_000_000.,
                update.render_pass_setup_ns as f64 / 1_000_000.,
                update.render_pass_blit_ns as f64 / 1_000_000.,
                update.render_pass_restore_ns as f64 / 1_000_000.,
                update.backdrop_readback_ns as f64 / 1_000_000.,
                update.backdrop_filter_ns as f64 / 1_000_000.,
                update.backdrop_filter_alloc_ns as f64 / 1_000_000.,
                update.backdrop_filter_build_ns as f64 / 1_000_000.,
                update.backdrop_filter_snapshot_ns as f64 / 1_000_000.,
                update.backdrop_filter_draw_ns as f64 / 1_000_000.,
                update.backdrop_filter_restore_ns as f64 / 1_000_000.,
                update.backdrop_filter_finish_ns as f64 / 1_000_000.,
                update.backdrop_clip_ns as f64 / 1_000_000.,
                update.backdrop_composite_ns as f64 / 1_000_000.,
                update.render_pass_blit_quads,
                update.render_pass_pixels,
                update.transparent_render_pass_quads,
                update.target_create_ns as f64 / 1_000_000.,
                update.target_clear_ns as f64 / 1_000_000.,
                update.target_finish_ns as f64 / 1_000_000.,
            );
        }
        Ok(output)
    }
}
