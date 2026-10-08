//! Compare shadow-mask compositing with the source Skia backend.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]

use layoutng_assembly::internal::layout_input::Offset;
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
use paint::paint_engine::{DisplayItem, DisplayItemType, PaintArtifact, PaintRect};

fn render_pair(items: Vec<DisplayItem>, width: u32, height: u32) -> (Vec<u8>, Vec<u8>) {
    let list = PaintArtifact {
        items,
        ..Default::default()
    };
    (
        raster::source_replay::RasterizeSourceDisplayItemList(&list, width, height),
        raster::pure_replay::RasterizeSourceDisplayItemList(&list, width, height),
    )
}

#[test]
fn scaled_shadow_sigma_uses_mapped_axis_geometric_mean() {
    use layoutng_assembly::internal::layout_input::TransformMatrix;
    let radius = PaintCornerRadius { x: 8.0, y: 8.0 };
    // Native bridge implements outer shadows; its inset branch is not an
    // inset-shadow oracle. Compare uniform and nonuniform CTM mapping here.
    for (sx, sy) in [(1.0, 1.0), (2.0, 2.0), (2.0, 1.5)] {
        let mut transform = TransformMatrix::default();
        transform.values[0] = sx;
        transform.values[5] = sy;
        let items = vec![
            DisplayItem {
                r#type: DisplayItemType::kDrawRect,
                rect: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 320.0,
                    height: 240.0,
                },
                color: Color {
                    red: 0.7,
                    green: 0.8,
                    blue: 0.9,
                    alpha: 1.0,
                },
                ..Default::default()
            },
            DisplayItem {
                r#type: DisplayItemType::kConcat,
                transform,
                ..Default::default()
            },
            DisplayItem {
                r#type: DisplayItemType::kDrawBoxShadow,
                rect: PaintRect {
                    x: 18.0,
                    y: 24.0,
                    width: 74.0,
                    height: 44.0,
                },
                corner_radii: PaintCornerRadii {
                    top_left: radius,
                    top_right: radius,
                    bottom_left: radius,
                    bottom_right: radius,
                },
                shadow_offset: Offset { x: 3.0, y: -2.0 },
                color: Color {
                    red: 0.2,
                    green: 0.4,
                    blue: 0.6,
                    alpha: 0.5,
                },
                blur_radius: 8.0,
                antialias: true,
                ..Default::default()
            },
        ];
        let (native, rust) = render_pair(items, 320, 240);
        assert_eq!(native, rust, "scale={sx},{sy}");
    }
}

#[test]
fn overlapping_translucent_shadow_masks_match_skia() {
    for background in [
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
        for alpha in [0.1, 0.2, 0.5, 1.0] {
            let mut items = vec![DisplayItem {
                r#type: DisplayItemType::kDrawRect,
                rect: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 64.0,
                    height: 64.0,
                },
                color: background,
                antialias: false,
                ..Default::default()
            }];
            for offset in [8.0, 12.0] {
                items.push(DisplayItem {
                    r#type: DisplayItemType::kDrawBoxShadow,
                    rect: PaintRect {
                        x: 8.0,
                        y: 8.0,
                        width: 32.0,
                        height: 24.0,
                    },
                    shadow_offset: Offset { x: 0.0, y: offset },
                    color: Color {
                        red: 77.0 / 255.0,
                        green: 128.0 / 255.0,
                        blue: 1.0,
                        alpha,
                    },
                    antialias: false,
                    ..Default::default()
                });
            }
            let (source, rust) = render_pair(items, 64, 64);
            assert_eq!(
                source, rust,
                "background={background:?}, shadow alpha={alpha}"
            );
        }
    }
}

#[test]
fn search_box_blurred_shadow_full_rgba_matches_skia() {
    let radius = PaintCornerRadius { x: 19.0, y: 19.0 };
    let first = DisplayItem {
        r#type: DisplayItemType::kDrawBoxShadow,
        rect: PaintRect {
            x: 112.0,
            y: 185.0,
            width: 800.0,
            height: 99.0,
        },
        corner_radii: PaintCornerRadii {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        },
        color: Color {
            red: 77.0 / 255.0,
            green: 128.0 / 255.0,
            blue: 1.0,
            alpha: 26.0 / 255.0,
        },
        shadow_offset: Offset { x: 0.0, y: 10.0 },
        blur_radius: 40.0,
        spread: -8.0,
        antialias: true,
        shadow_has_opaque_background: true,
        ..Default::default()
    };
    let second = DisplayItem {
        color: Color {
            alpha: 51.0 / 255.0,
            ..first.color
        },
        shadow_offset: Offset { x: 0.0, y: 13.0 },
        blur_radius: 18.0,
        spread: -15.0,
        ..first.clone()
    };
    let fill = DisplayItem {
        r#type: DisplayItemType::kDrawRoundedRect,
        rect: first.rect,
        corner_radii: first.corner_radii,
        color: Color {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        },
        antialias: true,
        ..Default::default()
    };
    let (source, rust) = render_pair(vec![first, second, fill], 1024, 768);
    let diffs: Vec<_> = source
        .chunks_exact(4)
        .zip(rust.chunks_exact(4))
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .collect();
    assert!(
        diffs.is_empty(),
        "{} shadow pixels differ, first {:?}",
        diffs.len(),
        diffs.first()
    );
}

#[test]
fn individual_search_shadows_colors_and_positions_match_skia() {
    let mut cases = 0;
    let mut failures = 0;
    let radius = PaintCornerRadius { x: 19.0, y: 19.0 };
    for (x, y) in [(112.0, 185.0), (112.25, 185.25), (20.0, 140.0)] {
        for (spread, blur, offset) in [(-8.0, 40.0, 10.0), (-15.0, 18.0, 13.0)] {
            for alpha in [26.0 / 255.0, 51.0 / 255.0, 0.5, 1.0] {
                let shadow = DisplayItem {
                    r#type: DisplayItemType::kDrawBoxShadow,
                    rect: PaintRect {
                        x,
                        y,
                        width: 800.0,
                        height: 99.0,
                    },
                    corner_radii: PaintCornerRadii {
                        top_left: radius,
                        top_right: radius,
                        bottom_left: radius,
                        bottom_right: radius,
                    },
                    color: Color {
                        red: 77.0 / 255.0,
                        green: 128.0 / 255.0,
                        blue: 1.0,
                        alpha,
                    },
                    shadow_offset: Offset { x: 0.0, y: offset },
                    blur_radius: blur,
                    spread,
                    antialias: true,
                    shadow_has_opaque_background: true,
                    ..Default::default()
                };
                let (a, b) = render_pair(vec![shadow], 1024, 768);
                let diffs: Vec<_> = a
                    .chunks_exact(4)
                    .zip(b.chunks_exact(4))
                    .enumerate()
                    .filter(|(_, (a, b))| a != b)
                    .collect();
                if !diffs.is_empty() {
                    failures += 1;
                    eprintln!("shadow x={x} y={y} spread={spread} blur={blur} alpha={alpha}: {} pixels, first {:?}",diffs.len(),diffs[0]);
                }
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 24);
    assert_eq!(failures, 0);
}

#[test]
fn search_shadow_source_and_hole_masks_match_skia() {
    let mut failures = 0;
    for (x, y) in [(112.0, 185.0), (112.25, 185.25)] {
        for hole in [false, true] {
            let radius = PaintCornerRadius {
                x: if hole { 18.0 } else { 19.0 },
                y: if hole { 18.0 } else { 19.0 },
            };
            let r = if hole {
                PaintRect {
                    x: x + 1.0,
                    y: y + 1.0,
                    width: 798.0,
                    height: 97.0,
                }
            } else {
                PaintRect {
                    x: x + 8.0,
                    y: y + 18.0,
                    width: 784.0,
                    height: 83.0,
                }
            };
            let round = DisplayItem {
                r#type: if hole {
                    DisplayItemType::kClipRoundedRect
                } else {
                    DisplayItemType::kDrawRoundedRect
                },
                rect: r,
                corner_radii: PaintCornerRadii {
                    top_left: radius,
                    top_right: radius,
                    bottom_left: radius,
                    bottom_right: radius,
                },
                color: Color {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
                antialias: true,
                ..Default::default()
            };
            let mut items = vec![round];
            if hole {
                items.push(DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 1024.0,
                        height: 768.0,
                    },
                    color: Color {
                        red: 0.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                    ..Default::default()
                });
            }
            let (a, b) = render_pair(items, 1024, 768);
            let diffs: Vec<_> = a
                .chunks_exact(4)
                .zip(b.chunks_exact(4))
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .collect();
            if !diffs.is_empty() {
                failures += 1;
                eprintln!(
                    "mask x={x} y={y} hole={hole}: {} pixels first {:?}",
                    diffs.len(),
                    diffs[0]
                );
            }
        }
    }
    assert_eq!(failures, 0);
}

#[test]
fn shadows_extending_beyond_the_device_match_skia() {
    for (x, y, width, height, radius, blur, alpha) in [
        (782.0, 751.0, 240.0, 224.0, 6.0, 40.0, 8.0),
        (811.0, 76.0, 361.0, 237.0, 8.0, 30.0, 26.0),
        (-20.0, -30.0, 240.0, 224.0, 6.0, 40.0, 26.0),
    ] {
        let r = PaintCornerRadius {
            x: radius,
            y: radius,
        };
        let items = vec![
            DisplayItem {
                r#type: DisplayItemType::kDrawRect,
                rect: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 1024.0,
                    height: 768.0,
                },
                color: Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
                antialias: false,
                ..Default::default()
            },
            DisplayItem {
                r#type: DisplayItemType::kDrawBoxShadow,
                rect: PaintRect {
                    x,
                    y,
                    width,
                    height,
                },
                corner_radii: PaintCornerRadii {
                    top_left: r,
                    top_right: r,
                    bottom_left: r,
                    bottom_right: r,
                },
                blur_radius: blur,
                color: Color {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: alpha / 255.0,
                },
                shadow_has_opaque_background: true,
                antialias: true,
                ..Default::default()
            },
        ];
        let (native, rust) = render_pair(items, 1024, 768);
        let differences = native
            .chunks_exact(4)
            .zip(rust.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(differences, 0, "shadow at {x},{y}");
    }
}
