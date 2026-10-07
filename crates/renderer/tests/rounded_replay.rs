//! Real Skia replay verifies both analytic coverage and color compositing.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};

#[test]
fn rectangular_rrect_clips_match_skia_bw_and_aa_dispatch() {
    for antialias in [false, true] {
        for x in [8.0, 8.1, 8.25, -2.25] {
            let list = PaintArtifact {
                items: vec![
                    DisplayItem {
                        r#type: T::kClipRoundedRect,
                        rect: PaintRect {
                            x,
                            y: 7.25,
                            width: 23.0,
                            height: 18.0,
                        },
                        antialias,
                        ..Default::default()
                    },
                    DisplayItem {
                        r#type: T::kClipRoundedRect,
                        rect: PaintRect {
                            x: 12.0,
                            y: 4.0,
                            width: 16.0,
                            height: 24.0,
                        },
                        antialias,
                        ..Default::default()
                    },
                    DisplayItem {
                        r#type: T::kDrawRect,
                        rect: PaintRect {
                            x: 0.0,
                            y: 0.0,
                            width: 48.0,
                            height: 40.0,
                        },
                        color: Color {
                            red: 0.2,
                            green: 0.7,
                            blue: 0.4,
                            alpha: 0.5,
                        },
                        ..Default::default()
                    },
                ]
                .into(),
                ..Default::default()
            };
            let native = renderer::source_replay::RasterizeSourceDisplayItemList(&list, 48, 40);
            let rust = renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 48, 40);
            assert_eq!(rust, native, "x={x}, antialias={antialias}");
        }
    }
}

#[test]
fn large_search_round_fill_on_colored_background_matches_skia() {
    let mut failures = 0;
    for (x, y) in [(112.0, 185.0), (112.25, 185.25), (20.0, 140.0)] {
        for alpha in [1.0, 0.3] {
            let r = PaintCornerRadius { x: 19.0, y: 19.0 };
            let list = PaintArtifact {
                items: vec![
                    DisplayItem {
                        r#type: T::kDrawRect,
                        rect: PaintRect {
                            x: 0.0,
                            y: 0.0,
                            width: 1024.0,
                            height: 768.0,
                        },
                        color: Color {
                            red: 0.3,
                            green: 0.5,
                            blue: 0.7,
                            alpha: 1.0,
                        },
                        ..Default::default()
                    },
                    DisplayItem {
                        r#type: T::kDrawRoundedRect,
                        rect: PaintRect {
                            x,
                            y,
                            width: 800.0,
                            height: 99.0,
                        },
                        corner_radii: PaintCornerRadii {
                            top_left: r,
                            top_right: r,
                            bottom_left: r,
                            bottom_right: r,
                        },
                        color: Color {
                            red: 1.0,
                            green: 1.0,
                            blue: 1.0,
                            alpha,
                        },
                        antialias: true,
                        ..Default::default()
                    },
                ]
                .into(),
                ..Default::default()
            };
            let a = renderer::source_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
            let b = renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
            let diff: Vec<_> = a
                .chunks_exact(4)
                .zip(b.chunks_exact(4))
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .collect();
            if !diff.is_empty() {
                failures += 1;
                eprintln!(
                    "large round x={x} y={y} alpha={alpha}: {} pixels first {:?}",
                    diff.len(),
                    diff[0]
                );
            }
        }
    }
    assert_eq!(failures, 0);
}

#[test]
fn rounded_fill_colors_positions_and_clips_match_skia() {
    let mut failures = 0;
    let mut first = None;
    let mut cases = 0;
    for (x, y) in [(8.0, 9.0), (8.25, 9.25), (252.0, 252.0), (393.0, 185.0)] {
        for (w, h, radius) in [(16.0, 16.0, 4.0), (52.0, 38.0, 19.0), (108.0, 44.0, 10.0)] {
            for color in [
                Color {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
                Color {
                    red: 1.0,
                    green: 69.0 / 255.0,
                    blue: 91.0 / 255.0,
                    alpha: 1.0,
                },
                Color {
                    red: 77.0 / 255.0,
                    green: 128.0 / 255.0,
                    blue: 199.0 / 255.0,
                    alpha: 0.3,
                },
            ] {
                for clip in [0, 1, 2] {
                    let rect = PaintRect {
                        x,
                        y,
                        width: w,
                        height: h,
                    };
                    let radius = PaintCornerRadius {
                        x: radius,
                        y: radius,
                    };
                    let radii = PaintCornerRadii {
                        top_left: radius,
                        top_right: radius,
                        bottom_left: radius,
                        bottom_right: radius,
                    };
                    let mut items = vec![];
                    if clip != 0 {
                        items.push(DisplayItem {
                            r#type: if clip == 2 {
                                T::kClipRoundedRect
                            } else {
                                T::kClipRect
                            },
                            rect: PaintRect {
                                x: x + 2.0,
                                y: y + 1.0,
                                width: w - 3.0,
                                height: h - 2.0,
                            },
                            antialias: clip == 2,
                            corner_radii: PaintCornerRadii {
                                top_left: radius,
                                top_right: radius,
                                bottom_left: radius,
                                bottom_right: radius,
                            },
                            ..Default::default()
                        });
                    }
                    items.push(DisplayItem {
                        r#type: T::kDrawRoundedRect,
                        rect,
                        corner_radii: radii,
                        color,
                        antialias: true,
                        ..Default::default()
                    });
                    let list = PaintArtifact {
                        items,
                        ..Default::default()
                    };
                    let a =
                        renderer::source_replay::RasterizeSourceDisplayItemList(&list, 512, 320);
                    let b = renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 512, 320);
                    let diffs: Vec<_> = a
                        .chunks_exact(4)
                        .zip(b.chunks_exact(4))
                        .enumerate()
                        .filter(|(_, (a, b))| a != b)
                        .collect();
                    if !diffs.is_empty() {
                        failures += 1;
                        eprintln!(
                            "xy={x},{y} wh={w},{h} alpha={} clip={clip} diffs={} first={:?}",
                            color.alpha,
                            diffs.len(),
                            diffs[0]
                        );
                        first.get_or_insert_with(||format!("rect={rect:?}, color={color:?}, clip={clip}: {} pixels, first {:?}",diffs.len(),diffs[0]));
                    }
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 108);
    assert_eq!(failures, 0, "{}", first.unwrap_or_default());
}

#[test]
fn repeated_gray_round_fills_across_vertical_tiles_match_skia() {
    for (x, y) in [(522.0, 483.0), (2.0, 751.0), (252.0, 252.0)] {
        let r = PaintCornerRadius { x: 6.0, y: 6.0 };
        let item = DisplayItem {
            r#type: T::kDrawRoundedRect,
            rect: PaintRect {
                x,
                y,
                width: 240.0,
                height: 135.0,
            },
            corner_radii: PaintCornerRadii {
                top_left: r,
                top_right: r,
                bottom_left: r,
                bottom_right: r,
            },
            color: Color {
                red: 241.0 / 255.0,
                green: 242.0 / 255.0,
                blue: 243.0 / 255.0,
                alpha: 1.0,
            },
            antialias: true,
            ..Default::default()
        };
        let list = PaintArtifact {
            items: vec![item.clone(), item].into(),
            ..Default::default()
        };
        let a = renderer::source_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
        let b = renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
        assert_eq!(
            a.chunks_exact(4)
                .zip(b.chunks_exact(4))
                .filter(|(a, b)| a != b)
                .count(),
            0,
            "round fill at {x},{y}"
        );
    }
}
