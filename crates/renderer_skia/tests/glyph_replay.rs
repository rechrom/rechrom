//! Differential regression tests against the existing native Skia oracle.
//! Run with --features pure_replay,source_replay on macOS. The production
//! CoreText mask path itself does not require source_replay or C++ Skia.
#![cfg(all(
    target_os = "macos",
    feature = "pure_replay",
    feature = "source_replay"
))]

use layoutng_assembly::fragment_tree::{PaintGlyph, PaintResources};
use layoutng_assembly::internal::layout_input::{FontFace, FontSmoothing, FontVariation, Offset};
use layoutng_assembly::internal::layout_input_types::Color;
use paint::paint_engine::{DisplayItem, DisplayItemType, PaintArtifact, PaintRect};
use std::sync::Arc;

#[test]
fn coretext_glyph_masks_match_skia_replay() {
    let cases = [
        (
            "PingFang SC",
            400.0,
            false,
            vec![467, 652, 2121],
            vec![FontVariation {
                tag: u32::from_be_bytes(*b"wght"),
                value: 400.0,
            }],
        ),
        (
            "PingFang SC",
            500.0,
            false,
            vec![467, 652, 2121],
            vec![FontVariation {
                tag: u32::from_be_bytes(*b"wght"),
                value: 500.0,
            }],
        ),
        (
            "PingFang SC",
            700.0,
            false,
            vec![467, 652, 2121],
            vec![FontVariation {
                tag: u32::from_be_bytes(*b"wght"),
                value: 600.0,
            }],
        ),
        ("Arial", 400.0, false, vec![36, 74, 82], vec![]),
        ("Arial", 700.0, true, vec![36, 74, 82], vec![]),
    ];
    let colors = [
        Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        },
        Color {
            red: 34.0 / 255.0,
            green: 34.0 / 255.0,
            blue: 34.0 / 255.0,
            alpha: 1.0,
        },
        Color {
            red: 0.15,
            green: 0.55,
            blue: 0.8,
            alpha: 0.5,
        },
    ];
    for (family, weight, italic, ids, variations) in cases {
        let resources = Arc::new(PaintResources {
            fonts: vec![FontFace {
                family: family.into(),
                native_family: family.into(),
                weight,
                italic,
                variations,
                ..Default::default()
            }],
            ..Default::default()
        });
        for size in [12.0, 16.0, 24.0] {
            for smoothing in [
                FontSmoothing::kNone,
                FontSmoothing::kAntialiased,
                FontSmoothing::kAuto,
            ] {
                for color in colors {
                    for origin in [8.0, 8.125, 8.5, 8.875] {
                        let background = DisplayItem {
                            r#type: DisplayItemType::kDrawRect,
                            rect: PaintRect {
                                x: 0.0,
                                y: 0.0,
                                width: 80.0,
                                height: 48.0,
                            },
                            color: Color {
                                red: 0.9,
                                green: 0.8,
                                blue: 0.7,
                                alpha: 1.0,
                            },
                            antialias: false,
                            ..Default::default()
                        };
                        let clip = DisplayItem {
                            r#type: DisplayItemType::kClipRect,
                            rect: PaintRect {
                                x: 10.0,
                                y: 8.0,
                                width: 52.0,
                                height: 30.0,
                            },
                            antialias: false,
                            ..Default::default()
                        };
                        let text = DisplayItem {
                            r#type: DisplayItemType::kDrawGlyphRun,
                            font_face_index: 0,
                            font_size: size,
                            font_smoothing: smoothing,
                            color,
                            text_blob_origin: Offset {
                                x: origin,
                                y: 32.375,
                            },
                            glyphs: ids
                                .iter()
                                .enumerate()
                                .map(|(i, &id)| PaintGlyph {
                                    id,
                                    offset: Offset {
                                        x: i as f64 * (size + 2.0),
                                        y: 0.0,
                                    },
                                    ..Default::default()
                                })
                                .collect(),
                            ..Default::default()
                        };
                        let list = PaintArtifact {
                            items: vec![background, clip, text].into(),
                            resources: Some(resources.clone()),
                            ..Default::default()
                        };
                        let native =
                            raster::source_replay::RasterizeSourceDisplayItemList(&list, 80, 48);
                        let rust =
                            raster::pure_replay::RasterizeSourceDisplayItemList(&list, 80, 48);
                        let differences = native
                            .chunks_exact(4)
                            .zip(rust.chunks_exact(4))
                            .filter(|(a, b)| a != b)
                            .count();
                        assert_eq!(differences,0,"family={family} size={size} smoothing={smoothing:?} origin={origin} color={color:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn coretext_unit_axis_transforms_match_skia_replay() {
    use layoutng_assembly::internal::layout_input::TransformMatrix;
    let resources = Arc::new(PaintResources {
        fonts: vec![FontFace {
            family: "Arial".into(),
            native_family: "Arial".into(),
            ..Default::default()
        }],
        ..Default::default()
    });
    let transforms = [
        [1.0, 0.0, 0.0, 1.0],
        [-1.0, 0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0, -1.0],
        [-1.0, 0.0, 0.0, -1.0],
        [-1.0, 1.2246467991473532e-16, -1.2246467991473532e-16, -1.0],
        [0.0, 1.0, -1.0, 0.0],
        [0.0, -1.0, 1.0, 0.0],
        [0.0, 1.0, 1.0, 0.0],
    ];
    let mut cases = 0;
    for t in transforms {
        for size in [12.0, 18.0, 24.0] {
            for smoothing in [
                FontSmoothing::kNone,
                FontSmoothing::kAntialiased,
                FontSmoothing::kAuto,
            ] {
                for phase in [0.0, 0.125, 0.375] {
                    let mut matrix = TransformMatrix::default();
                    matrix.values[0] = t[0];
                    matrix.values[1] = t[1];
                    matrix.values[4] = t[2];
                    matrix.values[5] = t[3];
                    matrix.values[12] = 40.0 + phase;
                    matrix.values[13] = 40.0 + phase;
                    let list = PaintArtifact {
                        resources: Some(resources.clone()),
                        items: vec![
                            DisplayItem {
                                r#type: DisplayItemType::kConcat,
                                transform: matrix,
                                ..Default::default()
                            },
                            DisplayItem {
                                r#type: DisplayItemType::kDrawGlyphRun,
                                font_face_index: 0,
                                font_size: size,
                                font_smoothing: smoothing,
                                color: Color {
                                    red: 246.0 / 255.0,
                                    green: 48.0 / 255.0,
                                    blue: 81.0 / 255.0,
                                    alpha: 0.7,
                                },
                                glyphs: vec![PaintGlyph {
                                    id: 36,
                                    ..Default::default()
                                }],
                                ..Default::default()
                            },
                        ]
                        .into(),
                        ..Default::default()
                    };
                    let native =
                        raster::source_replay::RasterizeSourceDisplayItemList(&list, 80, 80);
                    let rust = raster::pure_replay::RasterizeSourceDisplayItemList(&list, 80, 80);
                    let diffs = native
                        .chunks_exact(4)
                        .zip(rust.chunks_exact(4))
                        .enumerate()
                        .filter(|(_, (a, b))| a != b)
                        .collect::<Vec<_>>();
                    assert!(diffs.is_empty(), "transform={t:?} size={size} smoothing={smoothing:?} phase={phase} count={} first={:?}",diffs.len(),diffs.first());
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 216);
}

#[test]
fn scaled_coretext_glyph_masks_match_skia_replay() {
    use layoutng_assembly::internal::layout_input::TransformMatrix;
    for family in ["Arial", "PingFang SC"] {
        let ids = if family == "Arial" {
            [36, 74, 82]
        } else {
            [467, 652, 2121]
        };
        let resources = Arc::new(PaintResources {
            fonts: vec![FontFace {
                family: family.into(),
                native_family: family.into(),
                ..Default::default()
            }],
            ..Default::default()
        });
        for scale in [1.25, 1.5, 2.0, 3.0] {
            for size in [12.0, 24.0] {
                for smoothing in [FontSmoothing::kNone, FontSmoothing::kAuto] {
                    for alpha in [1.0, 0.5] {
                        for origin in [8.0, 8.125, 8.875] {
                            let list = PaintArtifact {
                                resources: Some(resources.clone()),
                                items: vec![
                                    DisplayItem {
                                        r#type: DisplayItemType::kDrawRect,
                                        rect: PaintRect {
                                            x: 0.0,
                                            y: 0.0,
                                            width: 300.0,
                                            height: 150.0,
                                        },
                                        color: Color {
                                            red: 0.9,
                                            green: 0.8,
                                            blue: 0.7,
                                            alpha: 1.0,
                                        },
                                        ..Default::default()
                                    },
                                    DisplayItem {
                                        r#type: DisplayItemType::kDrawGlyphRun,
                                        font_size: size,
                                        font_smoothing: smoothing,
                                        color: Color {
                                            red: 34.0 / 255.0,
                                            green: 77.0 / 255.0,
                                            blue: 128.0 / 255.0,
                                            alpha,
                                        },
                                        text_blob_origin: Offset {
                                            x: origin,
                                            y: 32.375,
                                        },
                                        glyphs: (0..30)
                                            .map(|i| PaintGlyph {
                                                id: ids[i % 3],
                                                offset: Offset {
                                                    x: (i as f64 - 1.0) * (size + 2.0),
                                                    y: 0.0,
                                                },
                                                ..Default::default()
                                            })
                                            .collect(),
                                        ..Default::default()
                                    },
                                ]
                                .into(),
                                ..Default::default()
                            };
                            // Clip intersects some glyphs and rejects the rest
                            // of this long run; padded edge bounds must survive.
                            let mut list = list;
                            std::sync::Arc::make_mut(&mut list.items).insert(
                                1,
                                DisplayItem {
                                    r#type: DisplayItemType::kClipRect,
                                    rect: PaintRect {
                                        x: 12.0,
                                        y: 8.0,
                                        width: 80.0,
                                        height: 30.0,
                                    },
                                    antialias: true,
                                    ..Default::default()
                                },
                            );
                            let mut reference = list.clone();
                            let mut matrix = TransformMatrix::default();
                            matrix.values[0] = scale;
                            matrix.values[5] = scale;
                            std::sync::Arc::make_mut(&mut reference.items).insert(
                                0,
                                DisplayItem {
                                    r#type: DisplayItemType::kConcat,
                                    transform: matrix,
                                    ..Default::default()
                                },
                            );
                            let native = raster::source_replay::RasterizeSourceDisplayItemList(
                                &reference, 300, 150,
                            );
                            let rust = raster::pure_replay::RasterizeSourceDisplayItemListWithScale(
                                &list, 300, 150, scale,
                            );
                            let different = native
                                .chunks_exact(4)
                                .zip(rust.chunks_exact(4))
                                .filter(|(a, b)| a != b)
                                .count();
                            assert_eq!(different,0,"family={family} scale={scale} size={size} smoothing={smoothing:?} alpha={alpha} origin={origin}");
                        }
                    }
                }
            }
        }
    }
}
