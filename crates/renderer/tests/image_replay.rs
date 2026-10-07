//! Full image replay compared with the source Skia backend.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::fragment_tree::PaintResources;
use layoutng_assembly::internal::layout_input::PaintImage;
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
use paint::paint_engine::{DisplayItem, DisplayItemType, PaintArtifact, PaintRect};
use std::sync::Arc;

#[test]
fn full_image_scaling_alpha_and_hard_clips_match_skia() {
    let mut cases = 0;
    let mut failures = 0;
    let mut first = None;
    for (w, h) in [(1, 19), (19, 1), (7, 9), (16, 16), (31, 27), (32, 28)] {
        for transparent in [false, true] {
            let rgba8: Vec<u8> = (0..h)
                .flat_map(|y| {
                    (0..w).flat_map(move |x| {
                        [
                            ((x * 37 + y * 17 + 11) % 256) as u8,
                            ((x * 13 + y * 53 + 71) % 256) as u8,
                            ((x * 73 + y * 29 + 19) % 256) as u8,
                            if transparent {
                                ((x * 43 + y * 61 + 29) % 256) as u8
                            } else {
                                255
                            },
                        ]
                    })
                })
                .collect();
            let resources = Arc::new(PaintResources {
                images: vec![PaintImage {
                    id: 1,
                    width: w,
                    height: h,
                    rgba8: rgba8.into(),
                    ..Default::default()
                }],
                ..Default::default()
            });
            for (dw, dh) in [(w, h), (5, 7), (13, 11), (41, 39)] {
                for (x, y) in [(8.0, 9.0), (252.0, 252.0), (-3.0, -2.0)] {
                    for clipped in [0, 1, 2] {
                        let mut items = vec![DisplayItem {
                            r#type: DisplayItemType::kDrawRect,
                            rect: PaintRect {
                                x: 0.0,
                                y: 0.0,
                                width: 320.0,
                                height: 320.0,
                            },
                            color: Color {
                                red: 77.0 / 255.0,
                                green: 128.0 / 255.0,
                                blue: 199.0 / 255.0,
                                alpha: 1.0,
                            },
                            antialias: false,
                            ..Default::default()
                        }];
                        if clipped == 1 {
                            items.push(DisplayItem {
                                r#type: DisplayItemType::kClipRect,
                                rect: PaintRect {
                                    x: x + 2.0,
                                    y: y + 1.0,
                                    width: dw.saturating_sub(3) as f64,
                                    height: dh.saturating_sub(2) as f64,
                                },
                                antialias: false,
                                ..Default::default()
                            });
                        }
                        if clipped == 2 {
                            let radius = PaintCornerRadius { x: 4.0, y: 4.0 };
                            items.push(DisplayItem {
                                r#type: DisplayItemType::kClipRoundedRect,
                                rect: PaintRect {
                                    x: x - 4.0,
                                    y: y - 4.0,
                                    width: dw as f64 + 8.0,
                                    height: dh as f64 + 8.0,
                                },
                                corner_radii: PaintCornerRadii {
                                    top_left: radius,
                                    top_right: radius,
                                    bottom_left: radius,
                                    bottom_right: radius,
                                },
                                antialias: true,
                                ..Default::default()
                            });
                        }
                        items.push(DisplayItem {
                            r#type: DisplayItemType::kDrawImageRect,
                            rect: PaintRect {
                                x,
                                y,
                                width: dw as f64,
                                height: dh as f64,
                            },
                            source_rect: PaintRect {
                                x: 0.0,
                                y: 0.0,
                                width: w as f64,
                                height: h as f64,
                            },
                            resource_id: 1,
                            ..Default::default()
                        });
                        let list = PaintArtifact {
                            items,
                            resources: Some(resources.clone()),
                            ..Default::default()
                        };
                        let source = renderer::source_replay::RasterizeSourceDisplayItemList(
                            &list, 320, 320,
                        );
                        let rust =
                            renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 320, 320);
                        let differences = source
                            .chunks_exact(4)
                            .zip(rust.chunks_exact(4))
                            .enumerate()
                            .filter(|(_, (a, b))| a != b)
                            .collect::<Vec<_>>();
                        if !differences.is_empty() {
                            failures += 1;
                            first.get_or_insert_with(|| format!("{w}x{h} -> {dw}x{dh}, origin={x},{y}, alpha={transparent}, clip={clipped}: {} pixels, first={:?}", differences.len(), differences[0]));
                        }
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 432);
    assert_eq!(failures, 0, "{}", first.unwrap_or_default());
}

#[test]
fn image_sampling_uses_clipped_layer_device_origin() {
    for (x, y, w, h) in [
        (2.0, 255.0, 500.0, 281.0),
        (522.0, 255.0, 240.0, 135.0),
        (782.0, 483.0, 240.0, 135.0),
    ] {
        let (iw, ih) = if w == 500.0 { (976, 549) } else { (672, 378) };
        let pixels: Vec<u8> = (0..ih)
            .flat_map(|sy| {
                (0..iw).flat_map(move |sx| {
                    [
                        ((sx * 37 + sy * 11) % 256) as u8,
                        ((sx * 7 + sy * 43) % 256) as u8,
                        ((sx * 19 + sy * 29) % 256) as u8,
                        255,
                    ]
                })
            })
            .collect();
        let bounds = PaintRect {
            x,
            y,
            width: w,
            height: h,
        };
        let list = PaintArtifact {
            resources: Some(Arc::new(PaintResources {
                images: vec![PaintImage {
                    id: 1,
                    width: iw,
                    height: ih,
                    rgba8: pixels.into(),
                    ..Default::default()
                }],
                ..Default::default()
            })),
            items: vec![
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kClipRect,
                    rect: bounds,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayer,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kDrawImageRect,
                    rect: bounds,
                    source_rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: iw as f64,
                        height: ih as f64,
                    },
                    resource_id: 1,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
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
        let native = renderer::source_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
        let rust = renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
        assert_eq!(
            native
                .chunks_exact(4)
                .zip(rust.chunks_exact(4))
                .filter(|(a, b)| a != b)
                .count(),
            0,
            "layer bounds {bounds:?}"
        );
    }
}

#[test]
fn rounded_image_aa_runs_and_layer_restore_match_skia() {
    let (iw, ih) = (672_u32, 378_u32);
    let rgba8: Vec<u8> = (0..ih)
        .flat_map(|y| {
            (0..iw).flat_map(move |x| {
                [
                    (x * 37 + y * 17) as u8,
                    (x * 13 + y * 53) as u8,
                    (x * 73 + y * 29) as u8,
                    255,
                ]
            })
        })
        .collect();
    let resources = Arc::new(PaintResources {
        images: vec![PaintImage {
            id: 1,
            width: iw,
            height: ih,
            rgba8: rgba8.into(),
            ..Default::default()
        }],
        ..Default::default()
    });
    for (x, y, width, height) in [
        (522.0, 483.0, 240.0, 135.0),
        (2.0, 255.0, 500.0, 281.0),
        (783.0, 752.0, 238.0, 134.0),
    ] {
        for layer in [false, true] {
            let radius = if width == 238.0 { 5.0 } else { 6.0 };
            let r = PaintCornerRadius {
                x: radius,
                y: radius,
            };
            let bounds = PaintRect {
                x,
                y,
                width,
                height,
            };
            let mut items = vec![
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 1024.0,
                        height: 768.0,
                    },
                    color: Color {
                        red: 241.0 / 255.0,
                        green: 242.0 / 255.0,
                        blue: 243.0 / 255.0,
                        alpha: 1.0,
                    },
                    ..Default::default()
                },
                DisplayItem {
                    r#type: DisplayItemType::kClipRoundedRect,
                    rect: bounds,
                    corner_radii: PaintCornerRadii {
                        top_left: r,
                        top_right: r,
                        bottom_left: r,
                        bottom_right: r,
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
            items.push(DisplayItem {
                r#type: DisplayItemType::kDrawImageRect,
                rect: bounds,
                source_rect: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: iw as f64,
                    height: ih as f64,
                },
                resource_id: 1,
                ..Default::default()
            });
            if layer {
                items.push(DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    ..Default::default()
                });
            }
            let list = PaintArtifact {
                items,
                resources: Some(resources.clone()),
                ..Default::default()
            };
            let a = renderer::source_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
            let b = renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
            let different: Vec<_> = a
                .chunks_exact(4)
                .zip(b.chunks_exact(4))
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .map(|(i, (a, b))| (i % 1024, i / 1024, a, b))
                .collect();
            assert_eq!(
                different.len(),
                0,
                "rounded image {bounds:?}, layer={layer}: {different:?}"
            );
        }
    }
}

#[test]
fn scaled_bitmap_matrix_and_cropped_layer_match_native_skia() {
    use layoutng_assembly::internal::layout_input::TransformMatrix;
    let (iw, ih) = (73, 61);
    let rgba8: Vec<u8> = (0..ih)
        .flat_map(|y| {
            (0..iw).flat_map(move |x| {
                [
                    (x * 37 + y * 17) as u8,
                    (x * 13 + y * 53) as u8,
                    (x * 73 + y * 29) as u8,
                    ((x * 43 + y * 61 + 29) % 256) as u8,
                ]
            })
        })
        .collect();
    let resources = Arc::new(PaintResources {
        images: vec![PaintImage {
            id: 1,
            width: iw,
            height: ih,
            rgba8: rgba8.into(),
            ..Default::default()
        }],
        ..Default::default()
    });
    let mut cases = 0;
    for scale in [0.8, 1.0, 1.2, 1.25, 1.5, 1.75, 2.0] {
        for layer in [false, true] {
            for (x, y, width, height) in [
                (0.0, 0.0, 40.0, 40.0),
                (200.0, 200.0, 40.0, 40.0),
                (12.2, 14.6, 40.1, 34.3),
                (168.5, 166.5, 40.0, 34.0),
            ] {
                for source in [
                    PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: iw as f64,
                        height: ih as f64,
                    },
                    PaintRect {
                        x: 4.0,
                        y: 6.0,
                        width: 60.0,
                        height: 50.0,
                    },
                ] {
                    let bounds = PaintRect {
                        x,
                        y,
                        width,
                        height,
                    };
                    let mut transform = TransformMatrix::default();
                    transform.values[0] = scale;
                    transform.values[5] = scale;
                    let mut items = vec![
                        DisplayItem {
                            r#type: DisplayItemType::kConcat,
                            transform,
                            ..Default::default()
                        },
                        DisplayItem {
                            r#type: DisplayItemType::kClipRect,
                            rect: bounds,
                            ..Default::default()
                        },
                    ];
                    if layer {
                        items.push(DisplayItem {
                            r#type: DisplayItemType::kSaveLayer,
                            ..Default::default()
                        });
                    }
                    items.push(DisplayItem {
                        r#type: DisplayItemType::kDrawImageRect,
                        rect: bounds,
                        source_rect: source,
                        resource_id: 1,
                        ..Default::default()
                    });
                    if layer {
                        items.push(DisplayItem {
                            r#type: DisplayItemType::kRestore,
                            ..Default::default()
                        });
                    }
                    let list = PaintArtifact {
                        items,
                        resources: Some(resources.clone()),
                        ..Default::default()
                    };
                    let actual =
                        renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 600, 600);
                    let expected =
                        renderer::source_replay::RasterizeSourceDisplayItemList(&list, 600, 600);
                    let diff = actual.iter().zip(&expected).filter(|(a, b)| a != b).count();

                    assert_eq!(
                        diff, 0,
                        "scale={scale} layer={layer} x={x} y={y} source={source:?}"
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 112);
}

#[test]
fn integer_translated_image_rows_and_aa_clips_match_native_skia() {
    use layoutng_assembly::internal::layout_input::TransformMatrix;
    let (iw, ih) = (37, 19);
    let rgba8: Vec<u8> = (0..ih)
        .flat_map(|y| {
            (0..iw).flat_map(move |x| {
                [
                    (x * 37 + y * 17) as u8,
                    (x * 13 + y * 53) as u8,
                    (x * 73 + y * 29) as u8,
                    ((x * 43 + y * 61 + 29) % 256) as u8,
                ]
            })
        })
        .collect();
    let resources = Arc::new(PaintResources {
        images: vec![PaintImage {
            id: 1,
            width: iw,
            height: ih,
            rgba8: rgba8.into(),
            ..Default::default()
        }],
        ..Default::default()
    });
    let mut cases = 0;
    for scale in [1.0] {
        for layer in [false, true] {
            for source in [
                PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 37.0,
                    height: 19.0,
                },
                PaintRect {
                    x: 4.0,
                    y: 6.0,
                    width: 29.0,
                    height: 11.0,
                },
            ] {
                for clip in 0..3 {
                    let bounds = PaintRect {
                        x: 128.0,
                        y: 127.0,
                        width: source.width / scale,
                        height: source.height / scale,
                    };
                    let mut transform = TransformMatrix::default();
                    transform.values[0] = scale;
                    transform.values[5] = scale;
                    let mut items = vec![
                        DisplayItem {
                            r#type: DisplayItemType::kDrawRect,
                            rect: PaintRect {
                                x: 0.0,
                                y: 0.0,
                                width: 320.0,
                                height: 300.0,
                            },
                            color: Color {
                                red: 77.0 / 255.0,
                                green: 128.0 / 255.0,
                                blue: 199.0 / 255.0,
                                alpha: 1.0,
                            },
                            ..Default::default()
                        },
                        DisplayItem {
                            r#type: DisplayItemType::kConcat,
                            transform,
                            ..Default::default()
                        },
                    ];
                    if clip != 0 {
                        let radius = PaintCornerRadius { x: 3.0, y: 3.0 };
                        items.push(DisplayItem {
                            r#type: if clip == 1 {
                                DisplayItemType::kClipRect
                            } else {
                                DisplayItemType::kClipRoundedRect
                            },
                            rect: PaintRect {
                                x: bounds.x + 0.25,
                                y: bounds.y + 0.375,
                                width: bounds.width - 0.875,
                                height: bounds.height - 0.625,
                            },
                            corner_radii: PaintCornerRadii {
                                top_left: radius,
                                top_right: radius,
                                bottom_left: radius,
                                bottom_right: radius,
                            },
                            antialias: true,
                            ..Default::default()
                        });
                    }
                    if layer {
                        items.push(DisplayItem {
                            r#type: DisplayItemType::kSaveLayer,
                            ..Default::default()
                        });
                    }
                    items.push(DisplayItem {
                        r#type: DisplayItemType::kDrawImageRect,
                        rect: bounds,
                        source_rect: source,
                        resource_id: 1,
                        ..Default::default()
                    });
                    if layer {
                        items.push(DisplayItem {
                            r#type: DisplayItemType::kRestore,
                            ..Default::default()
                        });
                    }
                    let list = PaintArtifact {
                        items,
                        resources: Some(resources.clone()),
                        ..Default::default()
                    };
                    let actual =
                        renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 320, 300);
                    let expected =
                        renderer::source_replay::RasterizeSourceDisplayItemList(&list, 320, 300);
                    let diff = actual.iter().zip(&expected).filter(|(a, b)| a != b).count();
                    assert_eq!(
                        diff, 0,
                        "scale={scale},layer={layer},source={source:?},clip={clip}"
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 12);
}
