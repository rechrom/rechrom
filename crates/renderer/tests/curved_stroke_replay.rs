//! Curved stroke geometry and coverage compared with native Skia.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb as V};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::SvgStrokeLineJoin;
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact};
fn geometry(name: &str, x: f64, y: f64) -> Vec<PaintPathCommand> {
    include_str!("fixtures/live_svg_strokes.tsv")
        .lines()
        .filter_map(|l| {
            let c: Vec<_> = l.split('\t').collect();
            if c[0] != name {
                return None;
            }
            let verb = match c[1] {
                "kMoveTo" => V::kMoveTo,
                "kLineTo" => V::kLineTo,
                "kCubicTo" => V::kCubicTo,
                "kClose" => V::kClose,
                _ => panic!(),
            };
            let point = |n: usize| Offset {
                x: c[n].parse::<f64>().unwrap() + x,
                y: c[n + 1].parse::<f64>().unwrap() + y,
            };
            Some(PaintPathCommand {
                verb,
                control1: point(2),
                control2: point(4),
                point: point(6),
                ..Default::default()
            })
        })
        .collect()
}
fn outline(commands: &[PaintPathCommand], width: f32, round: bool) -> Vec<PaintPathCommand> {
    use skia::raster::{PathBuilder, PathSegment};
    let mut b = PathBuilder::new();
    for c in commands {
        match c.verb {
            V::kMoveTo => b.move_to(c.point.x as f32, c.point.y as f32),
            V::kLineTo => b.line_to(c.point.x as f32, c.point.y as f32),
            V::kCubicTo => b.cubic_to(
                c.control1.x as f32,
                c.control1.y as f32,
                c.control2.x as f32,
                c.control2.y as f32,
                c.point.x as f32,
                c.point.y as f32,
            ),
            V::kClose => b.close(),
            _ => panic!(),
        }
    }
    let p = b
        .finish()
        .unwrap()
        .stroke(
            &skia::raster::Stroke {
                width,
                line_join: if round {
                    skia::raster::LineJoin::Round
                } else {
                    skia::raster::LineJoin::Miter
                },
                ..Default::default()
            },
            1.0,
        )
        .unwrap();
    let offset = |p: skia::raster::Point| Offset {
        x: p.x as f64,
        y: p.y as f64,
    };
    p.segments()
        .map(|s| {
            let mut c = PaintPathCommand::default();
            match s {
                PathSegment::MoveTo(p) => {
                    c.verb = V::kMoveTo;
                    c.point = offset(p)
                }
                PathSegment::LineTo(p) => {
                    c.verb = V::kLineTo;
                    c.point = offset(p)
                }
                PathSegment::QuadTo(a, p) => {
                    c.verb = V::kQuadraticTo;
                    c.control1 = offset(a);
                    c.point = offset(p)
                }
                PathSegment::CubicTo(a, b, p) => {
                    c.verb = V::kCubicTo;
                    c.control1 = offset(a);
                    c.control2 = offset(b);
                    c.point = offset(p)
                }
                PathSegment::Close => c.verb = V::kClose,
            };
            c
        })
        .collect()
}
#[test]
fn closed_svg_strokes_and_outlined_fills_match_source() {
    let mut failures = 0;
    let mut cases = 0;
    for name in ["tool", "circle"] {
        for (x, y) in [(8.0, 9.0), (8.25, 9.25), (252.25, 249.25)] {
            for width in [1.25, if name == "tool" { 1.5 } else { 1.6 }, 3.0] {
                for clip_kind in [0, 1, 2] {
                    for alpha in [1.0, 0.3] {
                        let round = name == "circle";
                        let item = DisplayItem {
                            r#type: T::kStrokePath,
                            path: geometry(name, x, y),
                            stroke_width: width,
                            svg_line_join: if round {
                                SvgStrokeLineJoin::kRound
                            } else {
                                SvgStrokeLineJoin::kMiter
                            },
                            antialias: true,
                            color: Color {
                                red: 0.13333334,
                                green: 0.3,
                                blue: 0.7,
                                alpha,
                            },
                            ..Default::default()
                        };
                        let fill = DisplayItem {
                            r#type: T::kDrawPath,
                            path: outline(&item.path, width as f32, round),
                            ..item.clone()
                        };
                        for (stage, draw) in [("stroke", item), ("quadratic fill", fill)] {
                            let mut items = vec![];
                            if clip_kind != 0 {
                                items.push(DisplayItem {
                                    r#type: T::kClipRect,
                                    rect: paint::paint_engine::PaintRect {
                                        x: x + 3.125,
                                        y: y + 5.25,
                                        width: 10.0,
                                        height: 10.0,
                                    },
                                    antialias: clip_kind == 2,
                                    ..Default::default()
                                });
                            }
                            items.push(draw);
                            let list = PaintArtifact {
                                items,
                                ..Default::default()
                            };
                            let native = renderer::source_replay::RasterizeSourceDisplayItemList(
                                &list, 512, 300,
                            );
                            let rust = renderer::pure_replay::RasterizeSourceDisplayItemList(
                                &list, 512, 300,
                            );
                            let diffs = native
                                .chunks_exact(4)
                                .zip(rust.chunks_exact(4))
                                .enumerate()
                                .filter(|(_, (a, b))| a != b)
                                .collect::<Vec<_>>();
                            if !diffs.is_empty() {
                                failures += 1;
                                eprintln!("{stage} {name} position={x},{y} width={width} clip={clip_kind} alpha={alpha}: {} pixels first={:?}",diffs.len(),diffs.first());
                            }
                            cases += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cases, 216);
    assert_eq!(failures, 0);
}
