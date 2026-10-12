use compositor::FrameBuilder;
use layer_tile::{FrameConfig, LayerTileEngine};
use layoutng_assembly::internal::layout_input::Offset;
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
use paint::paint_engine::{
    DisplayItem, DisplayItemType, PaintArtifact, PaintChunk, PaintRect, RasterEffectOutset,
    RecordedDisplayItem, RecordedDisplayItemKind,
};
use renderer::{PixelFormat, RenderTarget, Renderer, SingleSurfaceResources};
use std::sync::Arc;

#[test]
fn tile_shadow_equals_whole_canvas() {
    let rect = PaintRect {
        x: 20.1484375,
        y: 12.0,
        width: 108.8515625,
        height: 32.0,
    };
    let radius = PaintCornerRadius { x: 6.0, y: 6.0 };
    let radii = PaintCornerRadii {
        top_left: radius,
        top_right: radius,
        bottom_left: radius,
        bottom_right: radius,
    };
    let mut items = vec![DisplayItem {
        r#type: DisplayItemType::kDrawRect,
        rect: PaintRect {
            x: 0.0,
            y: 0.0,
            width: 160.0,
            height: 60.0,
        },
        color: Color {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        },
        ..Default::default()
    }];
    for (blur_radius, alpha) in [(2.0, 0.03), (1.0, 0.04)] {
        items.push(DisplayItem {
            r#type: DisplayItemType::kDrawBoxShadow,
            rect,
            corner_radii: radii,
            color: Color {
                red: 31.0 / 255.0,
                green: 35.0 / 255.0,
                blue: 40.0 / 255.0,
                alpha,
            },
            shadow_offset: Offset { x: 0.0, y: 1.0 },
            blur_radius,
            antialias: true,
            shadow_has_opaque_background: true,
            ..Default::default()
        });
    }
    items.push(DisplayItem {
        r#type: DisplayItemType::kDrawRoundedRect,
        rect,
        corner_radii: radii,
        color: Color {
            red: 31.0 / 255.0,
            green: 136.0 / 255.0,
            blue: 61.0 / 255.0,
            alpha: 1.0,
        },
        antialias: true,
        ..Default::default()
    });
    let artifact = Arc::new(PaintArtifact {
        items: items.into(),
        display_items: vec![RecordedDisplayItem {
            kind: RecordedDisplayItemKind::Drawing,
            id: Default::default(),
            visual_rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 160.0,
                height: 60.0,
            },
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: RasterEffectOutset::kWholePixel,
            record_begin: 0,
            record_end: 4,
            scroll_translation: None,
        }],
        chunks: vec![PaintChunk {
            is_cacheable: true,
            end_index: 1,
            bounds: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 160.0,
                height: 60.0,
            },
            drawable_bounds: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 160.0,
                height: 60.0,
            },
            ..Default::default()
        }],
        ..Default::default()
    });
    let config = FrameConfig {
        viewport: PaintRect {
            x: 0.0,
            y: 0.0,
            width: 160.0,
            height: 60.0,
        },
        raster_scale: 2.0,
        activation_scroll: None,
        frame_time: None,
    };
    let mut tiles = LayerTileEngine::default();
    let update = tiles.UpdatePending(&artifact, config).unwrap();
    let mut raster = raster::RasterEngine::default();
    let result = raster
        .prepare(&update.frame_plan, &update.raster_batch, 320, 120)
        .unwrap();
    tiles.ApplyRasterResults(&result.completions);
    let release = tiles.ActivatePending().unwrap();
    let released = raster.release_resources(&release).unwrap();
    tiles.AcknowledgeResourceRelease(&released);
    let frame = FrameBuilder::default()
        .BuildFrame(tiles.GetActiveFramePlan().unwrap())
        .unwrap();
    let surface = viz::SurfaceId(1);
    let mut viz = viz::VizEngine::default();
    viz.SubmitFrame(surface, frame).unwrap();
    let output = compositor::DeviceRect::new(0, 0, 320, 120);
    let frame = viz
        .Aggregate(
            output,
            &[viz::SurfacePlacement {
                surface_id: surface,
                destination: output,
            }],
        )
        .unwrap();
    let resources = SingleSurfaceResources {
        surface_id: surface,
        resources: &raster,
    };
    let mut renderer = Renderer::default();
    let mut words = vec![0u32; 320 * 120];
    renderer
        .Render(
            &frame,
            &resources,
            RenderTarget {
                width: 320,
                height: 120,
                pixels: &mut words,
                format: PixelFormat::Rgba8888,
                row_stride: 320,
            },
        )
        .unwrap();
    let actual: Vec<u8> = words.into_iter().flat_map(u32::to_ne_bytes).collect();
    let expected =
        raster::pure_replay::RasterizeSourceDisplayItemListWithScale(&artifact, 320, 120, 2.0);
    let count = actual
        .chunks_exact(4)
        .zip(expected.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(count, 0);
}
