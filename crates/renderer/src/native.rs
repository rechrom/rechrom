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
    pub passes: usize,
    pub quads: usize,
    pub damage_pixels: usize,
    pub tile_quads: usize,
    pub picture_quads: usize,
    pub solid_quads: usize,
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
        let occlusion_culler = OcclusionCuller::new(
            direct.settings.occlusion_culler_settings.clone(),
        );
        Self {
            direct,
            occlusion_culler,
        }
    }
    pub fn Render(
        &mut self,
        input: &mut VizOutput,
        target: B::Target<'_>,
    ) -> io::Result<RenderUpdate> {
        let started = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        let viewport = self.direct.backend.ValidateTarget(input, &target)?;
        let cull = self.occlusion_culler.RemoveOverdrawQuads(input.FrameMut());
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
        {
            let _draw = browser_tracing::span("renderer", "Renderer.DrawFrame");
            self.direct.DrawFrame(input, viewport)?;
        }
        let passes = started.map(|_| std::time::Instant::now());
        let update = {
            let _publish = browser_tracing::span("renderer", "Renderer.PublishFrame");
            self.direct.backend.PublishFrame(input, target)?
        };
        if let (Some(started), Some(passes)) = (started, passes) {
            eprintln!("renderer-profile passes_ms={:.3} root_ms={:.3} total_ms={:.3} passes={} quads={} damage_pixels={} tiles={} tile_ms={:.3} tile_image_ms={:.3} tile_draw_ms={:.3} tile_source_pixels={} tile_target_pixels={} tile_linear={} tile_fractional_source={} tile_scaled={} tile_blended={} tile_integer_translation={} tile_blit_eligible={} tile_blit={} pictures={} picture_ms={:.3} solids={} solid_ms={:.3}",
                (passes-started).as_secs_f64()*1000., passes.elapsed().as_secs_f64()*1000., started.elapsed().as_secs_f64()*1000.,
                update.passes, update.quads, update.damage_pixels, update.tile_quads, update.tile_ns as f64/1_000_000.,
                update.tile_image_ns as f64/1_000_000., update.tile_draw_ns as f64/1_000_000.,
                update.tile_source_pixels, update.tile_target_pixels, update.tile_linear_quads,
                update.tile_fractional_source_quads, update.tile_scaled_quads, update.tile_blended_quads,
                update.tile_integer_translation_quads, update.tile_blit_eligible_quads, update.tile_blit_quads,
                update.picture_quads, update.picture_ns as f64/1_000_000., update.solid_quads, update.solid_ns as f64/1_000_000.);
        }
        Ok(update)
    }
}
