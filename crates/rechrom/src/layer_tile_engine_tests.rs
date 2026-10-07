//! Focused checks of the real planner/backend ownership boundary.
use layer_tile::{FrameConfig, LayerTileEngine};
use layoutng_assembly::internal::layout_input_types::Color;
use paint::paint_engine::{
    DisplayItem, DisplayItemType, PaintArtifact, PaintChunk, PaintRect, RasterEffectOutset,
    RecordedDisplayItem, RecordedDisplayItemKind,
};
use renderer::{layer_tile_renderer::LayerTileRenderer, surface::PixelFormat};
use std::sync::Arc;

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

fn engine() -> LayerTileEngine {
    let mut engine = LayerTileEngine::default();
    engine.SetFrameConfig(FrameConfig {
        viewport: PaintRect {
            x: 0.0,
            y: 0.0,
            width: 64.0,
            height: 64.0,
        },
        raster_scale: 1.0,
        activation_scroll: None,
        prepaint_scroll: None,
    });
    engine
}

#[test]
fn frame_plan_reuses_pixels_and_preserves_retirement_across_skipped_frames() {
    let mut engine = engine();
    let source = artifact(0.4);
    engine.Update(&source).unwrap();
    assert!(std::ptr::eq(
        engine.GetFramePlan().unwrap().GetPaintArtifact(),
        source.as_ref()
    ));
    let mut renderer = LayerTileRenderer::default();
    let mut pixels = vec![0; 64 * 64];
    renderer
        .paint(
            engine.GetFramePlan().unwrap(),
            64,
            64,
            &mut pixels,
            PixelFormat::Bgra8888,
        )
        .unwrap();
    let cold = pixels.clone();
    let old_id = engine.GetFramePlan().unwrap().layers[0].tiles[0].tile_id;
    assert_eq!(engine.GetStats().ready_tile_count, 1);

    engine.Update(&source).unwrap();
    assert!(engine.GetFramePlan().unwrap().tasks.is_empty());
    renderer
        .paint(
            engine.GetFramePlan().unwrap(),
            64,
            64,
            &mut pixels,
            PixelFormat::Bgra8888,
        )
        .unwrap();
    assert_eq!(pixels, cold);
    assert_eq!(renderer.layer_tile_stats().reused_tiles, 1);

    let mut replacement = (*source).clone();
    std::sync::Arc::make_mut(&mut replacement.items)[0]
        .color
        .red = 0.8;
    let replacement = Arc::new(replacement);
    engine.Update(&replacement).unwrap();
    assert!(engine
        .GetFramePlan()
        .unwrap()
        .retired_tiles
        .contains(&old_id));
    // Skip presentation of the invalidating plan entirely.
    engine.Update(&replacement).unwrap();
    assert!(engine
        .GetFramePlan()
        .unwrap()
        .retired_tiles
        .contains(&old_id));
    renderer
        .paint(
            engine.GetFramePlan().unwrap(),
            64,
            64,
            &mut pixels,
            PixelFormat::Bgra8888,
        )
        .unwrap();
    let mut expected = vec![0; pixels.len()];
    renderer::surface::RenderDisplayItemListIntoWindowBufferWithFormat(
        &replacement,
        64,
        64,
        1.0,
        &mut expected,
        PixelFormat::Bgra8888,
    )
    .unwrap();
    assert_eq!(pixels, expected);
    assert_eq!(renderer.layer_tile_stats().resident_bytes, 256 * 256 * 4);
    engine.Update(&replacement).unwrap();
    assert!(engine.GetFramePlan().unwrap().retired_tiles.is_empty());
    assert!(engine.GetFramePlan().unwrap().tasks.is_empty());
}

#[test]
fn frame_plan_rejects_stale_completion_and_recovers_lost_backend_resources() {
    let mut engine = engine();
    let source = artifact(0.4);
    engine.Update(&source).unwrap();
    let stale = engine.GetFramePlan().unwrap().clone();
    engine.Update(&source).unwrap();
    assert!(!stale.DidRasterize());
    assert_eq!(engine.GetStats().ready_tile_count, 0);
    let id = engine.GetFramePlan().unwrap().layers[0].tiles[0].tile_id;
    let mut pixels = vec![0; 64 * 64];
    {
        let mut renderer = LayerTileRenderer::default();
        assert!(renderer
            .paint(&stale, 64, 64, &mut pixels, PixelFormat::Bgra8888)
            .is_err());
        renderer
            .paint(
                engine.GetFramePlan().unwrap(),
                64,
                64,
                &mut pixels,
                PixelFormat::Bgra8888,
            )
            .unwrap();
        assert_eq!(engine.GetStats().ready_tile_count, 1);
    }
    assert_eq!(engine.GetStats().ready_tile_count, 0);
    assert_eq!(engine.GetStats().tile_count, 1);
    engine.Update(&source).unwrap();
    assert_eq!(
        engine.GetFramePlan().unwrap().layers[0].tiles[0].tile_id,
        id
    );
    assert_eq!(engine.GetFramePlan().unwrap().tasks.len(), 1);
    LayerTileRenderer::default()
        .paint(
            engine.GetFramePlan().unwrap(),
            64,
            64,
            &mut pixels,
            PixelFormat::Bgra8888,
        )
        .unwrap();
}
