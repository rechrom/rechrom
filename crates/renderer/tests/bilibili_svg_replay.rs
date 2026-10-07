//! Regression geometry for round joins, coincident scan edges, and AA clip state.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb as V};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    PaintCornerRadii, PaintCornerRadius, SvgStrokeLineCap, SvgStrokeLineJoin,
};
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};

fn geometry(name: &str) -> Vec<PaintPathCommand> {
    include_str!("fixtures/bilibili_svg_paths.tsv")
        .lines()
        .filter_map(|line| {
            let c: Vec<_> = line.split('\t').collect();
            if c[0] != name {
                return None;
            }
            let point = |i: usize| Offset {
                x: c[i].parse().unwrap(),
                y: c[i + 1].parse().unwrap(),
            };
            Some(PaintPathCommand {
                verb: match c[1] {
                    "kMoveTo" => V::kMoveTo,
                    "kLineTo" => V::kLineTo,
                    "kCubicTo" => V::kCubicTo,
                    "kClose" => V::kClose,
                    _ => panic!(),
                },
                control1: point(2),
                control2: point(4),
                point: point(6),
                ..Default::default()
            })
        })
        .collect()
}
fn compare(items: Vec<DisplayItem>, label: &str) {
    let list = PaintArtifact {
        items,
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
        "{label}"
    );
}
fn transform(x: f64, y: f64, scale: f64) -> DisplayItem {
    let mut item = DisplayItem {
        r#type: T::kConcat,
        ..Default::default()
    };
    item.transform.values[0] = scale;
    item.transform.values[5] = scale;
    item.transform.values[12] = x;
    item.transform.values[13] = y;
    item
}
#[test]
fn coincident_edges_and_closed_round_joins_match_skia() {
    compare(
        vec![
            transform(847.0, 219.0, 1.0),
            DisplayItem {
                r#type: T::kClipRect,
                rect: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 10.0,
                    height: 10.0,
                },
                antialias: false,
                ..Default::default()
            },
            transform(0.0, 0.0, 10.0 / 9.0),
            DisplayItem {
                r#type: T::kDrawPath,
                path: geometry("channel_strokes"),
                color: Color {
                    red: 97.0 / 255.0,
                    green: 102.0 / 255.0,
                    blue: 109.0 / 255.0,
                    alpha: 1.0,
                },
                antialias: true,
                ..Default::default()
            },
        ],
        "coincident edges after scaling",
    );
    for (x, y) in [(50.0, 180.0), (50.25, 180.25), (252.0, 252.0)] {
        let mut items = vec![
            DisplayItem {
                r#type: T::kDrawRect,
                rect: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 1024.0,
                    height: 768.0,
                },
                color: Color {
                    red: 1.0,
                    green: 146.0 / 255.0,
                    blue: 18.0 / 255.0,
                    alpha: 1.0,
                },
                ..Default::default()
            },
            transform(x, y, 1.0),
        ];
        for name in ["lobe0", "lobe1", "lobe2", "lobe3"] {
            items.push(DisplayItem {
                r#type: T::kStrokePath,
                path: geometry(name),
                stroke_width: 2.0,
                svg_line_cap: SvgStrokeLineCap::kRound,
                svg_line_join: SvgStrokeLineJoin::kRound,
                color: Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
                antialias: true,
                ..Default::default()
            });
        }
        compare(items, "closed cubic round joins");
    }
}
#[test]
fn contained_rect_clip_preserves_native_aa_scan_mode() {
    for (width, icon_x) in [(250.0, 682.0), (312.0, 744.0)] {
        for f16 in [false, true] {
            let r = PaintCornerRadius { x: 7.0, y: 7.0 };
            compare(
                vec![
                    DisplayItem {
                        r#type: if f16 {
                            T::kSaveLayerAlpha
                        } else {
                            T::kSaveLayer
                        },
                        rect: PaintRect {
                            x: 462.1875,
                            y: 12.0,
                            width: width + 2.0,
                            height: 40.0,
                        },
                        opacity: if f16 { 0.9 } else { 1.0 },
                        ..Default::default()
                    },
                    DisplayItem {
                        r#type: T::kClipRoundedRect,
                        rect: PaintRect {
                            x: 463.0,
                            y: 13.0,
                            width,
                            height: 38.0,
                        },
                        corner_radii: PaintCornerRadii {
                            top_left: r,
                            top_right: r,
                            bottom_left: r,
                            bottom_right: r,
                        },
                        antialias: true,
                        ..Default::default()
                    },
                    transform(icon_x, 24.0, 1.0),
                    DisplayItem {
                        r#type: T::kClipRect,
                        rect: PaintRect {
                            x: 0.0,
                            y: 0.0,
                            width: 17.0,
                            height: 17.0,
                        },
                        antialias: false,
                        ..Default::default()
                    },
                    DisplayItem {
                        r#type: T::kDrawPath,
                        path: geometry("search"),
                        even_odd: true,
                        antialias: true,
                        color: Color {
                            red: 24.0 / 255.0,
                            green: 25.0 / 255.0,
                            blue: 28.0 / 255.0,
                            alpha: 1.0,
                        },
                        ..Default::default()
                    },
                    DisplayItem {
                        r#type: T::kRestore,
                        ..Default::default()
                    },
                ],
                "AA clip followed by a contained integer rectangle",
            );
        }
    }
}

#[test]
fn small_scaled_closed_round_joins_match_skia() {
    let mut items = vec![
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
                blue: 0.1,
                alpha: 1.0,
            },
            ..Default::default()
        },
        transform(913.0, 21.0, 1.0),
        transform(10.0 / 21.0, 0.0, 20.0 / 21.0),
    ];
    for name in [
        "header_lobe0",
        "header_lobe1",
        "header_lobe2",
        "header_lobe3",
    ] {
        items.push(DisplayItem {
            r#type: T::kStrokePath,
            path: geometry(name),
            stroke_width: 1.6,
            svg_line_cap: SvgStrokeLineCap::kButt,
            svg_line_join: SvgStrokeLineJoin::kRound,
            color: Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            },
            antialias: true,
            ..Default::default()
        });
    }
    compare(items, "scaled round joins of a small icon");
}

#[test]
fn bottom_clipped_rrect_aa_mask_matches_skia() {
    let r = PaintCornerRadius { x: 5.0, y: 5.0 };
    compare(
        vec![
            DisplayItem {
                r#type: T::kClipRoundedRect,
                rect: PaintRect {
                    x: 783.0,
                    y: 752.0,
                    width: 238.0,
                    height: 134.0,
                },
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
                r#type: T::kDrawRect,
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
            },
        ],
        "bottom clipped rounded mask",
    );
}

#[test]
fn partly_visible_badge_with_retained_aa_clip_matches_skia() {
    let r = PaintCornerRadius { x: 5.0, y: 5.0 };
    let mut items = vec![
        DisplayItem {
            r#type: T::kClipRoundedRect,
            rect: PaintRect {
                x: 783.0,
                y: 752.0,
                width: 238.0,
                height: 134.0,
            },
            corner_radii: PaintCornerRadii {
                top_left: r,
                top_right: r,
                bottom_left: r,
                bottom_right: r,
            },
            antialias: true,
            ..Default::default()
        },
        transform(795.0, 762.0, 1.0),
        DisplayItem {
            r#type: T::kClipRect,
            rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 20.0,
                height: 20.0,
            },
            antialias: false,
            ..Default::default()
        },
        transform(0.0, 0.0, 20.0 / 1024.0),
    ];
    for (name, color) in [
        (
            "video_dot",
            Color {
                red: 35.0 / 255.0,
                green: 173.0 / 255.0,
                blue: 229.0 / 255.0,
                alpha: 1.0,
            },
        ),
        (
            "video_badge",
            Color {
                red: 72.0 / 255.0,
                green: 207.0 / 255.0,
                blue: 229.0 / 255.0,
                alpha: 1.0,
            },
        ),
    ] {
        items.push(DisplayItem {
            r#type: T::kDrawPath,
            path: geometry(name),
            color,
            antialias: true,
            ..Default::default()
        });
    }
    compare(items, "partly visible video badge");
}
