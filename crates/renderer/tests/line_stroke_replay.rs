//! SkStroke axis-line butt, square and round caps with source analytic AA.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::SvgStrokeLineCap;
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};
#[test]
fn axis_caps_match_skia() {
    let mut failures = 0;
    let mut first = None;
    let mut cases = 0;
    for cap in [
        SvgStrokeLineCap::kButt,
        SvgStrokeLineCap::kSquare,
        SvgStrokeLineCap::kRound,
    ] {
        for (x, y) in [(8.0, 9.0), (8.25, 9.25), (252.0, 9.0)] {
            for width in [1.6, 3.0, 5.0] {
                for vertical in [false, true] {
                    for alpha in [1.0, 0.3] {
                        for clip in [false, true] {
                            let p = |verb, x, y| PaintPathCommand {
                                verb,
                                point: Offset { x, y },
                                ..Default::default()
                            };
                            let mut items = vec![];
                            if clip {
                                items.push(DisplayItem {
                                    r#type: T::kClipRect,
                                    rect: PaintRect {
                                        x: x - 3.0,
                                        y: y - 2.0,
                                        width: 12.0,
                                        height: 12.0,
                                    },
                                    antialias: false,
                                    ..Default::default()
                                });
                            }
                            items.push(DisplayItem {
                                r#type: T::kStrokePath,
                                path: vec![
                                    p(PaintPathVerb::kMoveTo, x, y),
                                    p(
                                        PaintPathVerb::kLineTo,
                                        if vertical { x } else { x + 7.33329 },
                                        if vertical { y + 7.33329 } else { y },
                                    ),
                                ],
                                stroke_width: width,
                                svg_line_cap: cap,
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
                                &list, 512, 64,
                            );
                            let b = renderer::pure_replay::RasterizeSourceDisplayItemList(
                                &list, 512, 64,
                            );
                            let diffs: Vec<_> = a
                                .chunks_exact(4)
                                .zip(b.chunks_exact(4))
                                .enumerate()
                                .filter(|(_, (a, b))| a != b)
                                .collect();
                            if !diffs.is_empty() {
                                failures += 1;
                                eprintln!("cap={cap:?} x={x} y={y} w={width} vertical={vertical} alpha={alpha} clip={clip} diffs={} first={:?}",diffs.len(),diffs[0]);
                                first.get_or_insert_with(||format!("cap={cap:?} xy={x},{y} width={width} vertical={vertical} alpha={alpha} clip={clip}: {} pixels; first {:?}",diffs.len(),diffs[0]));
                            }
                            cases += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cases, 216);
    assert_eq!(failures, 0, "{}", first.unwrap_or_default());
}
