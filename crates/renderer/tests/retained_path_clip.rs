#![cfg(feature = "pure_replay")]
use layoutng_assembly::internal::layout_input::{
    Offset, PaintPathCommand, PaintPathVerb, TransformMatrix,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::PaintBlendMode;
use paint::paint_engine::{
    DisplayItem, DisplayItemType as Kind, PaintArtifact, PaintChunk, PaintRect, RasterEffectOutset,
    RecordedDisplayItem, RecordedDisplayItemKind,
};
use paint::paint_property_tree::{
    ClipPaintPropertyNode, EffectPaintPropertyNode, PropertyTreeState, TransformPaintPropertyNode,
};
use std::sync::Arc;

fn point(verb: PaintPathVerb, x: f64, y: f64) -> PaintPathCommand {
    PaintPathCommand {
        verb,
        point: Offset { x, y },
        ..Default::default()
    }
}

fn assert_pixels_close(actual: &[u32], expected: &[u8], context: &str) {
    for (index, (actual, expected)) in actual.iter().zip(expected.chunks_exact(4)).enumerate() {
        let actual = actual.to_ne_bytes();
        for channel in 0..4 {
            // A clip baked into a PaintRecord is antialiased during tile
            // raster, while a property-tree clip is antialiased after
            // compositing. Chromium permits that ordering difference at the
            // one-pixel edge; solid coverage remains identical.
            let tolerance = if (actual[channel] == 0 || actual[channel] == 255)
                && (expected[channel] == 0 || expected[channel] == 255)
            {
                1
            } else {
                32
            };
            assert!(
                (actual[channel] as i16 - expected[channel] as i16).abs() <= tolerance,
                "{context} pixel={index} actual={actual:?} expected={expected:?}"
            );
        }
    }
}

#[test]
fn retained_path_clips_match_full_canvas_across_tiles_and_device_scales() {
    use PaintPathVerb::*;
    let triangle = vec![
        point(kMoveTo, 20.0, 25.0),
        point(kLineTo, 300.0, 25.0),
        point(kLineTo, 20.0, 305.0),
        point(kClose, 0.0, 0.0),
    ];
    let rectangle = vec![
        point(kMoveTo, 20.0, 25.0),
        point(kLineTo, 300.0, 25.0),
        point(kLineTo, 300.0, 305.0),
        point(kLineTo, 20.0, 305.0),
        point(kClose, 0.0, 0.0),
    ];
    let mut hole = rectangle.clone();
    hole.extend([
        point(kMoveTo, 60.0, 65.0),
        point(kLineTo, 200.0, 65.0),
        point(kLineTo, 200.0, 205.0),
        point(kLineTo, 60.0, 205.0),
        point(kClose, 0.0, 0.0),
    ]);
    for (path, even_odd) in [(rectangle, false), (triangle, false), (hole, true)] {
        for (scale, property_path) in [1.0, 1.25, 2.0]
            .into_iter()
            .flat_map(|scale| [(scale, true), (scale, false)])
        {
            let root = PropertyTreeState::default();
            let mut matrix = TransformMatrix::default();
            matrix.values[12] = 0.25;
            matrix.values[13] = 0.375;
            let transform = Arc::new(TransformPaintPropertyNode {
                id: 10,
                parent: Some(root.transform.clone()),
                matrix,
                origin: [0.0; 3],
                scroll: None,
                lifecycle: Default::default(),
                direct_compositing_reasons: vec![],
            });
            let bounds = PaintRect {
                x: 20.0,
                y: 25.0,
                width: 280.0,
                height: 280.0,
            };
            let clip = Arc::new(ClipPaintPropertyNode {
                id: 11,
                parent: Some(root.clip.clone()),
                local_transform_space: transform.clone(),
                rect: Some(bounds),
                radii: Default::default(),
                clip_path: path.clone(),
                clip_path_even_odd: even_odd,
                pixel_moving_filter: None,
                lifecycle: Default::default(),
            });
            let draw = DisplayItem {
                r#type: Kind::kDrawRect,
                rect: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 350.0,
                    height: 350.0,
                },
                color: Color {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
                ..Default::default()
            };
            let list = Arc::new(PaintArtifact {
                items: vec![
                    DisplayItem {
                        r#type: Kind::kSave,
                        ..Default::default()
                    },
                    DisplayItem {
                        r#type: Kind::kConcat,
                        transform: matrix,
                        ..Default::default()
                    },
                    DisplayItem {
                        r#type: Kind::kClipPath,
                        path: path.clone(),
                        even_odd,
                        antialias: true,
                        ..Default::default()
                    },
                    draw,
                    DisplayItem {
                        r#type: Kind::kRestore,
                        ..Default::default()
                    },
                ]
                .into(),
                display_items: vec![RecordedDisplayItem {
                    kind: RecordedDisplayItemKind::Drawing,
                    id: Default::default(),
                    visual_rect: bounds,
                    visual_rect_is_accurate: true,
                    draws_content: true,
                    raster_effect_outset: RasterEffectOutset::kHalfPixel,
                    record_begin: 3,
                    record_end: 4,
                    scroll_translation: None,
                }],
                chunks: vec![PaintChunk {
                    end_index: 1,
                    bounds,
                    drawable_bounds: bounds,
                    properties: PropertyTreeState {
                        transform,
                        clip: if property_path {
                            clip
                        } else {
                            root.clip.clone()
                        },
                        ..root
                    },
                    ..Default::default()
                }],
                ..Default::default()
            });
            let mut engine = layer_tile::LayerTileEngine::default();
            engine.SetFrameConfig(layer_tile::FrameConfig {
                viewport: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 400.0,
                    height: 400.0,
                },
                raster_scale: scale,
                activation_scroll: None,
                prepaint_scroll: None,
            });
            engine.Update(&list).unwrap();
            let size = (400.0 * scale) as u32;
            let mut actual = vec![0; size as usize * size as usize];
            let mut renderer = renderer::layer_tile_renderer::LayerTileRenderer::default();
            renderer
                .paint(
                    engine.GetFramePlan().unwrap(),
                    size,
                    size,
                    &mut actual,
                    skia::PixelFormat::Bgra8888,
                )
                .unwrap();
            let expected = renderer::surface::RenderDisplayItemListIntoTarget(
                &list,
                size,
                size,
                scale,
                skia::PixelStorage::owned(vec![0; size as usize * size as usize * 4]),
                skia::PixelFormat::Bgra8888,
            )
            .unwrap()
            .into_vec();
            assert_pixels_close(
                &actual,
                &expected,
                &format!("scale={scale} property_path={property_path} even_odd={even_odd}"),
            );
            assert!(renderer.layer_tile_stats().raster_tasks > 1);
        }
    }
}

#[test]
fn flat_path_clip_outside_opacity_group_is_retained() {
    use PaintPathVerb::*;
    let path = vec![
        point(kMoveTo, 20.0, 25.0),
        point(kLineTo, 300.0, 25.0),
        point(kLineTo, 20.0, 305.0),
        point(kClose, 0.0, 0.0),
    ];
    let root = PropertyTreeState::default();
    let effect = Arc::new(EffectPaintPropertyNode {
        id: 12,
        parent: Some(root.effect.clone()),
        local_transform_space: root.transform.clone(),
        output_clip: None,
        opacity: 0.5,
        blend_mode: PaintBlendMode::kNormal,
        filters: Vec::new(),
        isolates_blending: false,
        has_mask: false,
        is_mask: false,
        direct_compositing_reasons: vec![],
        lifecycle: Default::default(),
    });
    let bounds = PaintRect {
        x: 20.0,
        y: 25.0,
        width: 280.0,
        height: 280.0,
    };
    let list = Arc::new(PaintArtifact {
        items: vec![
            DisplayItem {
                r#type: Kind::kSave,
                ..Default::default()
            },
            DisplayItem {
                r#type: Kind::kClipPath,
                path,
                antialias: true,
                ..Default::default()
            },
            DisplayItem {
                r#type: Kind::kSaveLayerAlpha,
                opacity: 0.5,
                ..Default::default()
            },
            DisplayItem {
                r#type: Kind::kDrawRect,
                rect: PaintRect {
                    width: 350.0,
                    height: 350.0,
                    ..Default::default()
                },
                color: Color {
                    red: 1.0,
                    alpha: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            DisplayItem {
                r#type: Kind::kRestore,
                ..Default::default()
            },
            DisplayItem {
                r#type: Kind::kRestore,
                ..Default::default()
            },
        ]
        .into(),
        display_items: vec![RecordedDisplayItem {
            kind: RecordedDisplayItemKind::Drawing,
            id: Default::default(),
            visual_rect: bounds,
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: RasterEffectOutset::kHalfPixel,
            record_begin: 3,
            record_end: 4,
            scroll_translation: None,
        }],
        chunks: vec![PaintChunk {
            end_index: 1,
            bounds,
            drawable_bounds: bounds,
            properties: PropertyTreeState { effect, ..root },
            ..Default::default()
        }],
        ..Default::default()
    });
    let mut engine = layer_tile::LayerTileEngine::default();
    engine.SetFrameConfig(layer_tile::FrameConfig {
        viewport: PaintRect {
            width: 400.0,
            height: 400.0,
            ..Default::default()
        },
        raster_scale: 1.0,
        activation_scroll: None,
        prepaint_scroll: None,
    });
    engine.Update(&list).unwrap();
    let mut actual = vec![0; 400 * 400];
    let mut renderer = renderer::layer_tile_renderer::LayerTileRenderer::default();
    renderer
        .paint(
            engine.GetFramePlan().unwrap(),
            400,
            400,
            &mut actual,
            skia::PixelFormat::Bgra8888,
        )
        .unwrap();
    let expected = renderer::surface::RenderDisplayItemListIntoTarget(
        &list,
        400,
        400,
        1.0,
        skia::PixelStorage::owned(vec![0; 400 * 400 * 4]),
        skia::PixelFormat::Bgra8888,
    )
    .unwrap()
    .into_vec();
    assert_pixels_close(&actual, &expected, "flat clip outside opacity group");
}

#[test]
fn retained_affine_rect_clip_matches_full_canvas() {
    let root = PropertyTreeState::default();
    // x' = -y + 80, y' = x + 20. This is the same class as CSS/SVG
    // quarter-turn clips, whose diagonal terms may be tiny cos(pi/2) values.
    let mut matrix = TransformMatrix::default();
    matrix.values[0] = 0.0;
    matrix.values[1] = 1.0;
    matrix.values[4] = -1.0;
    matrix.values[5] = 0.0;
    matrix.values[12] = 80.0;
    matrix.values[13] = 20.0;
    let mut inverse = TransformMatrix::default();
    inverse.values[0] = 0.0;
    inverse.values[1] = -1.0;
    inverse.values[4] = 1.0;
    inverse.values[5] = 0.0;
    inverse.values[12] = -20.0;
    inverse.values[13] = 80.0;
    let transform = Arc::new(TransformPaintPropertyNode {
        id: 20,
        parent: Some(root.transform.clone()),
        matrix,
        origin: [0.0; 3],
        scroll: None,
        lifecycle: Default::default(),
        direct_compositing_reasons: vec![],
    });
    let local_clip = PaintRect {
        width: 40.0,
        height: 30.0,
        ..Default::default()
    };
    let clip = Arc::new(ClipPaintPropertyNode {
        id: 21,
        parent: Some(root.clip.clone()),
        local_transform_space: transform,
        rect: Some(local_clip),
        radii: Default::default(),
        clip_path: vec![],
        clip_path_even_odd: false,
        pixel_moving_filter: None,
        lifecycle: Default::default(),
    });
    let bounds = PaintRect {
        x: 50.0,
        y: 20.0,
        width: 30.0,
        height: 40.0,
    };
    let list = Arc::new(PaintArtifact {
        items: vec![
            DisplayItem {
                r#type: Kind::kSave,
                ..Default::default()
            },
            DisplayItem {
                r#type: Kind::kConcat,
                transform: matrix,
                ..Default::default()
            },
            DisplayItem {
                r#type: Kind::kClipRect,
                rect: local_clip,
                antialias: true,
                ..Default::default()
            },
            DisplayItem {
                r#type: Kind::kConcat,
                transform: inverse,
                ..Default::default()
            },
            DisplayItem {
                r#type: Kind::kDrawRect,
                rect: PaintRect {
                    width: 120.0,
                    height: 100.0,
                    ..Default::default()
                },
                color: Color {
                    red: 1.0,
                    alpha: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            DisplayItem {
                r#type: Kind::kRestore,
                ..Default::default()
            },
        ]
        .into(),
        display_items: vec![RecordedDisplayItem {
            kind: RecordedDisplayItemKind::Drawing,
            id: Default::default(),
            visual_rect: bounds,
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: RasterEffectOutset::kHalfPixel,
            record_begin: 4,
            record_end: 5,
            scroll_translation: None,
        }],
        chunks: vec![PaintChunk {
            end_index: 1,
            bounds,
            drawable_bounds: bounds,
            properties: PropertyTreeState { clip, ..root },
            ..Default::default()
        }],
        ..Default::default()
    });
    let mut engine = layer_tile::LayerTileEngine::default();
    engine.SetFrameConfig(layer_tile::FrameConfig {
        viewport: PaintRect {
            width: 120.0,
            height: 100.0,
            ..Default::default()
        },
        raster_scale: 2.0,
        activation_scroll: None,
        prepaint_scroll: None,
    });
    engine.Update(&list).unwrap();
    let mut actual = vec![0; 240 * 200];
    let mut renderer = renderer::layer_tile_renderer::LayerTileRenderer::default();
    renderer
        .paint(
            engine.GetFramePlan().unwrap(),
            240,
            200,
            &mut actual,
            skia::PixelFormat::Bgra8888,
        )
        .unwrap();
    let expected = renderer::surface::RenderDisplayItemListIntoTarget(
        &list,
        240,
        200,
        2.0,
        skia::PixelStorage::owned(vec![0; 240 * 200 * 4]),
        skia::PixelFormat::Bgra8888,
    )
    .unwrap()
    .into_vec();
    assert_pixels_close(&actual, &expected, "retained affine rect clip");
}
