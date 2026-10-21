//! Platform-independent host boundary for translated DirectRenderer.
use crate::components::viz::{
    common::display::renderer_settings::{DebugRendererSettings, RendererSettings},
    service::display::direct_renderer::{DirectRenderer, DirectRendererBackend},
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
    pub picture_ns: u64,
    pub solid_ns: u64,
}

/// Backend-owned target and storage keep the management core independent of
/// backend storage layouts or a concrete drawing library. The aggregate is borrowed.
pub struct Renderer<B: DirectRendererBackend> {
    direct: DirectRenderer<B>,
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
        Self { direct }
    }
    pub fn Render(&mut self, input: &VizOutput, target: B::Target<'_>) -> io::Result<RenderUpdate> {
        let started = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        let viewport = self.direct.backend.ValidateTarget(input, &target)?;
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
            eprintln!("renderer-profile passes_ms={:.3} root_ms={:.3} total_ms={:.3} passes={} quads={} damage_pixels={} tiles={} tile_ms={:.3} pictures={} picture_ms={:.3} solids={} solid_ms={:.3}",
                (passes-started).as_secs_f64()*1000., passes.elapsed().as_secs_f64()*1000., started.elapsed().as_secs_f64()*1000.,
                update.passes, update.quads, update.damage_pixels, update.tile_quads, update.tile_ns as f64/1_000_000.,
                update.picture_quads, update.picture_ns as f64/1_000_000., update.solid_quads, update.solid_ns as f64/1_000_000.);
        }
        Ok(update)
    }
}
