//! Focused checks of the real planner/backend ownership boundary.
use layer_tile::{FrameConfig, LayerTileEngine, PendingTreeUpdate};
use layoutng_assembly::internal::layout_input_types::Color;
use paint::paint_engine::{
    DisplayItem, DisplayItemType, PaintArtifact, PaintChunk, PaintRect, RasterEffectOutset,
    RecordedDisplayItem, RecordedDisplayItemKind,
};
use renderer::{PixelFormat, RenderTarget, Renderer, SingleSurfaceResources};
use std::sync::Arc;

#[derive(Default)]
struct TestPipeline {
    raster: raster::RasterEngine,
    compositor: compositor::FrameBuilder,
    viz: viz::VizEngine,
    renderer: Renderer,
}

impl TestPipeline {
    fn paint(
        &mut self,
        engine: &mut LayerTileEngine,
        update: PendingTreeUpdate,
        width: u32,
        height: u32,
        target: &mut [u32],
        format: PixelFormat,
    ) -> std::io::Result<()> {
        let raster =
            self.raster
                .prepare(&update.frame_plan, &update.raster_batch, width, height)?;
        engine.ApplyRasterResults(&raster.completions);
        let release = engine
            .ActivatePending()
            .ok_or_else(|| std::io::Error::other("pending tree did not become ready"))?;
        let released = self.raster.release_resources(&release)?;
        engine.AcknowledgeResourceRelease(&released);
        let plan = engine.GetActiveFramePlan().expect("active test plan");
        let frame = self
            .compositor
            .BuildFrame(plan)
            .map_err(std::io::Error::other)?;
        let surface = viz::SurfaceId(1);
        self.viz
            .SubmitFrame(surface, frame)
            .map_err(std::io::Error::other)?;
        let output = compositor::DeviceRect::new(0, 0, width, height);
        let frame = self
            .viz
            .Aggregate(
                output,
                &[viz::SurfacePlacement {
                    surface_id: surface,
                    destination: output,
                }],
            )
            .map_err(std::io::Error::other)?;
        let resources = SingleSurfaceResources {
            surface_id: surface,
            resources: &self.raster,
        };
        self.renderer.Render(
            &frame,
            &resources,
            RenderTarget {
                width,
                height,
                pixels: target,
                format,
                row_stride: width as usize,
            },
        )?;
        Ok(())
    }

    fn stats(&self) -> raster::RasterStats {
        self.raster.stats()
    }
}

fn artifact(red: f32) -> Arc<PaintArtifact> {
    let rect = PaintRect {
        x: 0.0,
        y: 0.0,
        width: 64.0,
        height: 64.0,
    };
    Arc::new(PaintArtifact {
        items: vec![DisplayItem {
            r#type: DisplayItemType::kDrawRect,
            rect,
            color: Color {
                red,
                green: 0.2,
                blue: 0.3,
                alpha: 1.0,
            },
            ..Default::default()
        }]
        .into(),
        display_items: vec![RecordedDisplayItem {
            kind: RecordedDisplayItemKind::Drawing,
            id: Default::default(),
            visual_rect: rect,
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: RasterEffectOutset::kNone,
            record_begin: 0,
            record_end: 1,
            scroll_translation: None,
        }],
        chunks: vec![PaintChunk {
            is_cacheable: true,
            end_index: 1,
            bounds: rect,
            drawable_bounds: rect,
            ..Default::default()
        }],
        ..Default::default()
    })
}

fn config() -> FrameConfig {
    FrameConfig {
        viewport: PaintRect {
            x: 0.0,
            y: 0.0,
            width: 64.0,
            height: 64.0,
        },
        raster_scale: 1.0,
        activation_scroll: None,
        frame_time: None,
    }
}

#[test]
fn frame_plan_reuses_pixels_and_preserves_retirement_across_skipped_frames() {
    let mut engine = LayerTileEngine::default();
    let source = artifact(0.4);
    let update = engine.UpdatePending(&source, config()).unwrap();
    assert!(std::ptr::eq(
        update.frame_plan.GetPaintArtifact(),
        source.as_ref()
    ));
    let mut renderer = TestPipeline::default();
    let mut pixels = vec![0; 64 * 64];
    renderer
        .paint(
            &mut engine,
            update,
            64,
            64,
            &mut pixels,
            PixelFormat::Bgra8888,
        )
        .unwrap();
    let cold = pixels.clone();
    let old_id = engine.GetActiveFramePlan().unwrap().layers[0].tiles[0].tile_id;
    assert_eq!(engine.GetStats().ready_tile_count, 1);

    let update = engine.UpdatePending(&source, config()).unwrap();
    assert!(update.raster_batch.tasks.is_empty());
    renderer
        .paint(
            &mut engine,
            update,
            64,
            64,
            &mut pixels,
            PixelFormat::Bgra8888,
        )
        .unwrap();
    assert_eq!(pixels, cold);
    assert_eq!(renderer.stats().reused_tiles, 1);

    let mut replacement = (*source).clone();
    std::sync::Arc::make_mut(&mut replacement.items)[0]
        .color
        .red = 0.8;
    let replacement = Arc::new(replacement);
    let _skipped = engine.UpdatePending(&replacement, config()).unwrap();
    // Skip presentation of the invalidating plan entirely.
    let update = engine.UpdatePending(&replacement, config()).unwrap();
    renderer
        .paint(
            &mut engine,
            update,
            64,
            64,
            &mut pixels,
            PixelFormat::Bgra8888,
        )
        .unwrap();
    let mut expected = vec![0; pixels.len()];
    raster::surface::RenderDisplayItemListIntoWindowBufferWithFormat(
        &replacement,
        64,
        64,
        1.0,
        &mut expected,
        PixelFormat::Bgra8888,
    )
    .unwrap();
    assert_eq!(pixels, expected);
    assert_eq!(renderer.stats().resident_bytes, 256 * 256 * 4);
    assert!(raster::RasterResourceProvider::resource(&renderer.raster, old_id).is_none());
    let update = engine.UpdatePending(&replacement, config()).unwrap();
    assert!(update.raster_batch.tasks.is_empty());
    let release = engine.ActivatePending().expect("unchanged tree activates");
    assert!(release.tile_ids.is_empty());
}

#[test]
fn frame_plan_rejects_stale_completion_and_recovers_lost_backend_resources() {
    let mut engine = LayerTileEngine::default();
    let source = artifact(0.4);
    let stale = engine.UpdatePending(&source, config()).unwrap();
    let replacement = artifact(0.8);
    let current = engine.UpdatePending(&replacement, config()).unwrap();
    assert_eq!(engine.GetStats().ready_tile_count, 0);
    let id = current.frame_plan.layers[0].tiles[0].tile_id;
    let mut pixels = vec![0; 64 * 64];
    let mut renderer = TestPipeline::default();
    let stale_raster = renderer
        .raster
        .prepare(&stale.frame_plan, &stale.raster_batch, 64, 64)
        .unwrap();
    assert_eq!(engine.ApplyRasterResults(&stale_raster.completions), 0);
    renderer
        .paint(
            &mut engine,
            current,
            64,
            64,
            &mut pixels,
            PixelFormat::Bgra8888,
        )
        .unwrap();
    assert_eq!(engine.GetStats().ready_tile_count, 1);
    renderer.raster.invalidate();
    engine.InvalidateResources();
    assert_eq!(engine.GetStats().ready_tile_count, 0);
    assert_eq!(engine.GetStats().tile_count, 1);
    let update = engine.UpdatePending(&replacement, config()).unwrap();
    assert_eq!(update.frame_plan.layers[0].tiles[0].tile_id, id);
    assert_eq!(update.raster_batch.tasks.len(), 1);
    renderer
        .paint(
            &mut engine,
            update,
            64,
            64,
            &mut pixels,
            PixelFormat::Bgra8888,
        )
        .unwrap();
}
