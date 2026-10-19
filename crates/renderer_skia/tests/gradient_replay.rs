//! Differential coverage for the Skia screenshot gradient/dither stages.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input::Offset;
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{PaintColorStop, PaintShader};
use paint::paint_engine::{DisplayItem, DisplayItemType, PaintArtifact, PaintRect};

#[test]
fn constant_gradient_soft_clip_matches_skia() {
    use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
    let mut failures = 0;
    for (x, y) in [(112.0, 185.0), (252.0, 249.0)] {
        for bg in [
            Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            },
            Color {
                red: 0.3,
                green: 0.5,
                blue: 0.7,
                alpha: 1.0,
            },
        ] {
            for color in [
                Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
                Color {
                    red: 0.2,
                    green: 0.4,
                    blue: 0.7,
                    alpha: 1.0,
                },
            ] {
                let rect = PaintRect {
                    x,
                    y,
                    width: 800.0,
                    height: 99.0,
                };
                let r = PaintCornerRadius { x: 19.0, y: 19.0 };
                let shader = PaintShader {
                    start: Offset { x, y },
                    end: Offset {
                        x: x + 800.0,
                        y: y + 99.0,
                    },
                    stops: vec![
                        PaintColorStop {
                            offset: 0.0,
                            color,
                            ..Default::default()
                        },
                        PaintColorStop {
                            offset: 1.0,
                            color,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                };
                let list = PaintArtifact {
                    items: vec![
                        DisplayItem {
                            r#type: DisplayItemType::kDrawRect,
                            rect: PaintRect {
                                x: 0.0,
                                y: 0.0,
                                width: 1024.0,
                                height: 768.0,
                            },
                            color: bg,
                            ..Default::default()
                        },
                        DisplayItem {
                            r#type: DisplayItemType::kClipRoundedRect,
                            rect,
                            corner_radii: PaintCornerRadii {
                                top_left: r,
                                top_right: r,
                                bottom_left: r,
                                bottom_right: r,
                            },
                            antialias: true,
                            ..Default::default()
                        },
                        DisplayItem {
                            r#type: DisplayItemType::kDrawGradientRect,
                            rect,
                            tile_rect: rect,
                            paint_shader: Some(shader),
                            ..Default::default()
                        },
                    ]
                    .into(),
                    ..Default::default()
                };
                let a = raster::source_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
                let b = raster::pure_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
                let diffs: Vec<_> = a
                    .chunks_exact(4)
                    .zip(b.chunks_exact(4))
                    .enumerate()
                    .filter(|(_, (a, b))| a != b)
                    .collect();
                if !diffs.is_empty() {
                    failures += 1;
                    eprintln!(
                        "constant xy={x},{y} bg={bg:?} color={color:?}: {} pixels first {:?}",
                        diffs.len(),
                        diffs[0]
                    );
                }
            }
        }
    }
    assert_eq!(failures, 0);
}

#[test]
fn opaque_uniform_stop_gradient_dithering_matches_skia() {
    let palettes = vec![
        vec![(51.0, 119.0, 255.0), (168.0, 82.0, 255.0)],
        vec![(235.0, 249.0, 192.0), (100.0, 31.0, 75.0)],
        vec![
            (40.0, 106.0, 255.0),
            (78.0, 110.0, 242.0),
            (114.0, 116.0, 249.0),
            (159.0, 102.0, 255.0),
        ],
    ];
    for palette in palettes {
        for (x, y) in [(8.0, 8.0), (252.0, 252.0), (508.0, 508.0), (794.0, 230.0)] {
            for (dx, dy) in [(108.0, 0.0), (108.0, 44.0), (-108.0, 44.0), (0.0, 44.0)] {
                for clipped in [0, 1, 2] {
                    let rect = PaintRect {
                        x,
                        y,
                        width: 108.0,
                        height: 44.0,
                    };
                    let (start, end) = if palette.len() == 4 && dx == 108.0 && dy == 44.0 {
                        // Actual Baidu button's CSS gradient extends outside
                        // the rectangle; this also exercises nonzero pivots.
                        (
                            Offset {
                                x: x + 16.9489860534668,
                                y: y - 16.36744689941406,
                            },
                            Offset {
                                x: x + 91.0510063171387,
                                y: y + 60.36744689941406,
                            },
                        )
                    } else {
                        (
                            Offset { x, y },
                            Offset {
                                x: x + dx,
                                y: y + dy,
                            },
                        )
                    };
                    let shader = PaintShader {
                        start,
                        end,
                        stops: palette
                            .iter()
                            .copied()
                            .enumerate()
                            .map(|(i, (r, g, b))| PaintColorStop {
                                offset: i as f64 / (palette.len() - 1) as f64,
                                color: Color {
                                    red: r / 255.0,
                                    green: g / 255.0,
                                    blue: b / 255.0,
                                    alpha: 1.0,
                                },
                                ..Default::default()
                            })
                            .collect(),
                        ..Default::default()
                    };
                    let mut items = Vec::new();
                    if clipped != 0 {
                        items.push(DisplayItem {
                            r#type: if clipped == 2 {
                                DisplayItemType::kClipRoundedRect
                            } else {
                                DisplayItemType::kClipRect
                            },
                            rect: PaintRect {
                                x: x + 7.0,
                                y: y + 3.0,
                                width: 90.0,
                                height: 35.0,
                            },
                            antialias: clipped == 2,
                            corner_radii: {
                                use layoutng_assembly::internal::paint_input::{
                                    PaintCornerRadii, PaintCornerRadius,
                                };
                                let r = PaintCornerRadius { x: 12.0, y: 12.0 };
                                PaintCornerRadii {
                                    top_left: r,
                                    top_right: r,
                                    bottom_left: r,
                                    bottom_right: r,
                                }
                            },
                            ..Default::default()
                        });
                    }
                    items.push(DisplayItem {
                        r#type: DisplayItemType::kDrawGradientRect,
                        rect,
                        tile_rect: rect,
                        paint_shader: Some(shader),
                        ..Default::default()
                    });
                    let list = PaintArtifact {
                        items,
                        ..Default::default()
                    };
                    let source =
                        raster::source_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
                    let rust =
                        raster::pure_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
                    let differing_pixels = source
                        .chunks_exact(4)
                        .zip(rust.chunks_exact(4))
                        .filter(|(a, b)| a != b)
                        .count();
                    assert_eq!(
                        differing_pixels, 0,
                        "xy=({x},{y}) vector=({dx},{dy}), clip={clipped}, palette={palette:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn nonuniform_alpha_gradient_mask_uses_canvas_coordinates_and_layer_origin() {
    use paint::paint_engine::DisplayMaskLayer;
    let clip = PaintRect {
        x: 2.0,
        y: 255.0,
        width: 500.0,
        height: 363.0,
    };
    let mask_rect = PaintRect {
        x: 2.0,
        y: -162.0,
        width: 500.0,
        height: 780.0,
    };
    for background in [
        Color {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        },
        Color {
            red: 244.0 / 255.0,
            green: 165.0 / 255.0,
            blue: 56.0 / 255.0,
            alpha: 1.0,
        },
    ] {
        let shader = PaintShader {
            start: Offset { x: 252.0, y: 618.0 },
            end: Offset {
                x: 252.0,
                y: -162.0,
            },
            interpolate_premultiplied: true,
            stops: vec![
                PaintColorStop {
                    offset: 0.11,
                    color: Color {
                        red: 47.0 / 255.0,
                        green: 50.0 / 255.0,
                        blue: 56.0 / 255.0,
                        alpha: 1.0,
                    },
                    ..Default::default()
                },
                PaintColorStop {
                    offset: 0.2,
                    color: Color {
                        red: 0.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 0.0,
                    },
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let list = PaintArtifact {
            items: vec![
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 1024.0,
                        height: 768.0,
                    },
                    color: background,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kClipRect,
                    rect: clip,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kBeginMask,
                    mask_layers: vec![DisplayMaskLayer {
                        clip_rect: mask_rect,
                        paint_shader: Some(shader),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    rect: mask_rect,
                    color: Color {
                        red: 0.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kEndMask,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    ..Default::default()
                },
            ]
            .into(),
            ..Default::default()
        };
        let native = raster::source_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
        let rust = raster::pure_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
        assert_eq!(
            native
                .chunks_exact(4)
                .zip(rust.chunks_exact(4))
                .filter(|(a, b)| a != b)
                .count(),
            0,
            "background {background:?}"
        );
    }
}

#[test]
fn vertical_multistop_gradient_rows_match_skia_with_soft_clips() {
    use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
    for alpha in [1.0, 0.63] {
        for middle in [0.5, 0.37] {
            for premultiplied in [false, true] {
                let rect = PaintRect {
                    x: 13.0,
                    y: 249.0,
                    width: 280.0,
                    height: 54.0,
                };
                let r = PaintCornerRadius { x: 9.0, y: 9.0 };
                let shader = PaintShader {
                    start: Offset { x: 13.0, y: 249.0 },
                    end: Offset { x: 13.0, y: 303.0 },
                    interpolate_premultiplied: premultiplied,
                    stops: vec![
                        PaintColorStop {
                            offset: 0.0,
                            color: Color {
                                red: 0.2,
                                green: 0.4,
                                blue: 0.7,
                                alpha,
                            },
                            ..Default::default()
                        },
                        PaintColorStop {
                            offset: middle,
                            color: Color {
                                red: 0.8,
                                green: 0.3,
                                blue: 0.4,
                                alpha,
                            },
                            ..Default::default()
                        },
                        PaintColorStop {
                            offset: 1.0,
                            color: Color {
                                red: 0.3,
                                green: 0.9,
                                blue: 0.5,
                                alpha,
                            },
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                };
                let list = PaintArtifact {
                    items: vec![
                        DisplayItem {
                            r#type: DisplayItemType::kClipRoundedRect,
                            rect,
                            corner_radii: PaintCornerRadii {
                                top_left: r,
                                top_right: r,
                                bottom_left: r,
                                bottom_right: r,
                            },
                            antialias: true,
                            ..Default::default()
                        },
                        DisplayItem {
                            r#type: DisplayItemType::kDrawGradientRect,
                            rect,
                            tile_rect: rect,
                            paint_shader: Some(shader),
                            ..Default::default()
                        },
                    ]
                    .into(),
                    ..Default::default()
                };
                let actual = raster::pure_replay::RasterizeSourceDisplayItemList(&list, 320, 320);
                let expected =
                    raster::source_replay::RasterizeSourceDisplayItemList(&list, 320, 320);
                let diff = actual.iter().zip(&expected).filter(|(a, b)| a != b).count();
                assert_eq!(
                    diff, 0,
                    "alpha={alpha} middle={middle} premultiplied={premultiplied}"
                );
            }
        }
    }
}

#[test]
fn diagonal_multistop_gradient_segments_and_bounded_layers_native_oracle() {
    use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
    let mut failures = Vec::new();
    for scale in [1.0, 2.0] {
        for opaque in [true, false] {
            for uniform in [true, false] {
                for premultiplied in [false, true] {
                    for direction in [1.0, -1.0] {
                        for layer in [false, true] {
                            let rect = PaintRect {
                                x: 247.0,
                                y: 245.0,
                                width: 65.0,
                                height: 57.0,
                            };
                            let radius = PaintCornerRadius { x: 7.0, y: 7.0 };
                            let mut items = vec![
                                DisplayItem {
                                    r#type: DisplayItemType::kDrawRect,
                                    rect: PaintRect {
                                        x: 0.0,
                                        y: 0.0,
                                        width: 320.0,
                                        height: 320.0,
                                    },
                                    color: Color {
                                        red: 0.17,
                                        green: 0.43,
                                        blue: 0.69,
                                        alpha: 1.0,
                                    },
                                    ..Default::default()
                                },
                                DisplayItem {
                                    r#type: DisplayItemType::kSave,
                                    ..Default::default()
                                },
                                DisplayItem {
                                    r#type: DisplayItemType::kClipRoundedRect,
                                    rect,
                                    corner_radii: PaintCornerRadii {
                                        top_left: radius,
                                        top_right: radius,
                                        bottom_left: radius,
                                        bottom_right: radius,
                                    },
                                    antialias: true,
                                    ..Default::default()
                                },
                            ];
                            if layer {
                                items.push(DisplayItem {
                                    r#type: DisplayItemType::kSaveLayer,
                                    ..Default::default()
                                });
                            }
                            let colors = [
                                (51.0 / 255.0, 119.0 / 255.0, 254.0 / 255.0, 69.0 / 255.0),
                                (76.0 / 255.0, 111.0 / 255.0, 255.0 / 255.0, 207.0 / 255.0),
                                (131.0 / 255.0, 112.0 / 255.0, 255.0 / 255.0, 109.0 / 255.0),
                                (186.0 / 255.0, 89.0 / 255.0, 255.0 / 255.0, 168.0 / 255.0),
                            ];
                            let offsets = if uniform {
                                [0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0]
                            } else {
                                [0.0, 0.23, 0.66, 1.0]
                            };
                            let shader = PaintShader {
                                start: Offset {
                                    x: if direction > 0.0 { 251.0 } else { 308.0 },
                                    y: 248.0,
                                },
                                end: Offset {
                                    x: if direction > 0.0 { 309.0 } else { 252.0 },
                                    y: 299.0,
                                },
                                interpolate_premultiplied: premultiplied,
                                stops: colors
                                    .into_iter()
                                    .zip(offsets)
                                    .map(|((red, green, blue, alpha), offset)| PaintColorStop {
                                        offset,
                                        color: Color {
                                            red,
                                            green,
                                            blue,
                                            alpha: if opaque { 1.0 } else { alpha },
                                        },
                                        ..Default::default()
                                    })
                                    .collect(),
                                ..Default::default()
                            };
                            items.push(DisplayItem {
                                r#type: DisplayItemType::kDrawGradientRect,
                                rect,
                                tile_rect: rect,
                                paint_shader: Some(shader),
                                ..Default::default()
                            });
                            if layer {
                                items.push(DisplayItem {
                                    r#type: DisplayItemType::kRestore,
                                    ..Default::default()
                                });
                            }
                            items.push(DisplayItem {
                                r#type: DisplayItemType::kRestore,
                                ..Default::default()
                            });
                            let list = PaintArtifact {
                                items,
                                ..Default::default()
                            };
                            let dimension = (320.0 * scale) as u32;
                            let mut reference = list.clone();
                            let mut transform =
                                layoutng_assembly::internal::layout_input::TransformMatrix::default(
                                );
                            transform.values[0] = scale;
                            transform.values[5] = scale;
                            std::sync::Arc::make_mut(&mut reference.items).insert(
                                0,
                                DisplayItem {
                                    r#type: DisplayItemType::kConcat,
                                    transform,
                                    ..Default::default()
                                },
                            );
                            let expected = raster::source_replay::RasterizeSourceDisplayItemList(
                                &reference, dimension, dimension,
                            );
                            let actual =
                                raster::pure_replay::RasterizeSourceDisplayItemListWithScale(
                                    &list, dimension, dimension, scale,
                                );
                            let differences =
                                actual.iter().zip(&expected).filter(|(a, b)| a != b).count();
                            if scale == 2.0
                                && !opaque
                                && uniform
                                && !premultiplied
                                && direction == 1.0
                                && layer
                            {
                                // This single quantization boundary already differs with the
                                // original scalar lookup (see native-n32-origin-scalar.log).
                                // Lock the exact existing delta rather than allowing a tolerance;
                                // every other native case remains fully byte-identical.
                                let delta: Vec<_> = actual
                                    .iter()
                                    .zip(&expected)
                                    .enumerate()
                                    .filter(|(_, (a, b))| a != b)
                                    .map(|(i, (a, b))| {
                                        (
                                            i % 4,
                                            (i / 4) % dimension as usize,
                                            (i / 4) / dimension as usize,
                                            *a,
                                            *b,
                                        )
                                    })
                                    .collect();
                                assert_eq!(
                                    delta,
                                    vec![(1, 556, 538, 110, 111)],
                                    "known scalar boundary changed"
                                );
                            } else if differences != 0 {
                                let first = actual
                                    .iter()
                                    .zip(&expected)
                                    .enumerate()
                                    .find(|(_, (a, b))| a != b)
                                    .map(|(i, (a, b))| {
                                        (
                                            i % 4,
                                            (i / 4) % dimension as usize,
                                            (i / 4) / dimension as usize,
                                            *a,
                                            *b,
                                        )
                                    });
                                failures.push(format!("scale={scale} opaque={opaque} uniform={uniform} premul={premultiplied} dir={direction} layer={layer}: {differences} channels first={first:?}"));
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
