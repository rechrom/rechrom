//! Real post-JS SVG geometry, compared with unchanged native Skia.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input::{
    Offset, PaintPathCommand, PaintPathVerb as V, TransformMatrix,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{SvgStrokeLineCap, SvgStrokeLineJoin};
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};
fn geometry(name: &str, x: f64, y: f64) -> Vec<PaintPathCommand> {
    let fixture = if name == "tool-dot" {
        include_str!("fixtures/live_svg_tool_dot.tsv")
    } else {
        include_str!("fixtures/live_svg_conics.tsv")
    };
    fixture
        .lines()
        .filter_map(|l| {
            let fields: Vec<_> = l.split('\t').collect();
            if fields[0] != name {
                return None;
            }
            let point = |n: usize| Offset {
                x: fields[n].parse::<f64>().unwrap() + x,
                y: fields[n + 1].parse::<f64>().unwrap() + y,
            };
            Some(PaintPathCommand {
                verb: match fields[1] {
                    "kMoveTo" => V::kMoveTo,
                    "kLineTo" => V::kLineTo,
                    "kConicTo" => V::kConicTo,
                    "kCubicTo" => V::kCubicTo,
                    "kClose" => V::kClose,
                    _ => panic!(),
                },
                control1: point(2),
                control2: point(4),
                point: point(6),
                conic_weight: fields[8].parse().unwrap(),
            })
        })
        .collect()
}
#[test]
fn transformed_cubic_dot_resolves_device_space_convexity() {
    let mut failures = 0;
    let mut cases = 0;
    for (x, y) in [
        (8., 9.),
        (8.25, 9.25),
        (252.25, 249.25),
        (970., 604.),
        (916., 604.),
        (760., 235.),
    ] {
        for clip_kind in [0, 1, 2] {
            for alpha in [1.0, 0.3] {
                let mut transform = TransformMatrix::default();
                transform.values[12] = x;
                transform.values[13] = y;
                let mut items = vec![DisplayItem {
                    r#type: T::kConcat,
                    transform,
                    ..Default::default()
                }];
                if clip_kind != 0 {
                    items.push(DisplayItem {
                        r#type: T::kClipRect,
                        rect: PaintRect {
                            x: 7.125,
                            y: 15.25,
                            width: 1.5,
                            height: 2.,
                        },
                        antialias: clip_kind == 2,
                        ..Default::default()
                    });
                }
                items.push(DisplayItem {
                    r#type: T::kDrawPath,
                    path: geometry("tool-dot", 0., 0.),
                    antialias: true,
                    color: Color {
                        red: 0.2,
                        green: 0.3,
                        blue: 0.7,
                        alpha,
                    },
                    ..Default::default()
                });
                let list = PaintArtifact {
                    items,
                    ..Default::default()
                };
                let a = renderer::source_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
                let b = renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
                let diffs: Vec<_> = a
                    .chunks_exact(4)
                    .zip(b.chunks_exact(4))
                    .enumerate()
                    .filter(|(_, (a, b))| a != b)
                    .collect();
                if !diffs.is_empty() {
                    failures += 1;
                    eprintln!("dot transform={x},{y} clip={clip_kind} alpha={alpha}: {} pixels first={:?}",diffs.len(),diffs.first());
                }
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 36);
    assert_eq!(failures, 0);
}
#[test]
fn conic_fills_strokes_and_open_round_caps_match_source() {
    let mut failures = 0;
    let mut cases = 0;
    for (name, kind) in [
        ("conic-mic", T::kDrawPath),
        ("conic-mic", T::kStrokePath),
        ("open-mic", T::kStrokePath),
    ] {
        for (x, y) in [(8., 9.), (8.25, 9.25), (252.25, 249.25)] {
            for width in [1.25, 1.6, 3.0] {
                for clip_kind in [0, 1, 2] {
                    for alpha in [1.0, 0.3] {
                        let mut items = vec![];
                        if clip_kind != 0 {
                            items.push(DisplayItem {
                                r#type: T::kClipRect,
                                rect: PaintRect {
                                    x: x + 3.125,
                                    y: y + 5.25,
                                    width: 10.,
                                    height: 10.,
                                },
                                antialias: clip_kind == 2,
                                ..Default::default()
                            });
                        }
                        items.push(DisplayItem {
                            r#type: kind,
                            path: geometry(name, x, y),
                            stroke_width: width,
                            svg_line_cap: SvgStrokeLineCap::kRound,
                            svg_line_join: SvgStrokeLineJoin::kRound,
                            antialias: true,
                            color: Color {
                                red: 0.13333334,
                                green: 0.3,
                                blue: 0.7,
                                alpha,
                            },
                            ..Default::default()
                        });
                        let list = PaintArtifact {
                            items,
                            ..Default::default()
                        };
                        let a = renderer::source_replay::RasterizeSourceDisplayItemList(
                            &list, 512, 300,
                        );
                        let b =
                            renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 512, 300);
                        let diffs: Vec<_> = a
                            .chunks_exact(4)
                            .zip(b.chunks_exact(4))
                            .enumerate()
                            .filter(|(_, (a, b))| a != b)
                            .collect();
                        if !diffs.is_empty() {
                            failures += 1;
                            eprintln!("{name} {kind:?} at={x},{y} width={width} clip={clip_kind} alpha={alpha}: {} pixels first={:?}",diffs.len(),diffs.first());
                        }
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 162);
    assert_eq!(failures, 0);
}
