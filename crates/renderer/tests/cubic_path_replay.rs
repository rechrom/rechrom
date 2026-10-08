//! Convex cubics and winding/even-odd rings compare the analytic edge pipeline with Skia.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb as V};
use layoutng_assembly::internal::layout_input_types::Color;
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact};
fn circle(x: f64, y: f64, w: f64, h: f64, reverse: bool) -> Vec<PaintPathCommand> {
    let rx = w * 0.5;
    let ry = h * 0.5;
    let cx = x + rx;
    let cy = y + ry;
    let k = 0.5522847498307936;
    let points = [(cx + rx, cy), (cx, cy + ry), (cx - rx, cy), (cx, cy - ry)];
    let controls = [
        ((cx + rx, cy + k * ry), (cx + k * rx, cy + ry)),
        ((cx - k * rx, cy + ry), (cx - rx, cy + k * ry)),
        ((cx - rx, cy - k * ry), (cx - k * rx, cy - ry)),
        ((cx + k * rx, cy - ry), (cx + rx, cy - k * ry)),
    ];
    let p = |(x, y)| Offset { x, y };
    let mut out = vec![PaintPathCommand {
        verb: V::kMoveTo,
        point: p(points[0]),
        ..Default::default()
    }];
    for j in 0..4 {
        let i = if reverse { 3 - j } else { j };
        let (a, b) = controls[i];
        out.push(PaintPathCommand {
            verb: V::kCubicTo,
            control1: p(if reverse { b } else { a }),
            control2: p(if reverse { a } else { b }),
            point: p(points[if reverse { i } else { (i + 1) % 4 }]),
            ..Default::default()
        });
    }
    out.push(PaintPathCommand {
        verb: V::kClose,
        ..Default::default()
    });
    out
}
#[test]
fn convex_cubic_contours_match_native_skia() {
    let mut failures = 0;
    let mut first = None;
    let mut cases = 0;
    for (x, y) in [(8.0, 9.0), (8.25, 9.25), (393.0, 9.0)] {
        for w in [2.3, 6.666, 17.0, 38.0] {
            for aspect in [0.5, 1.0] {
                for reverse in [false, true] {
                    for alpha in [1.0, 0.3] {
                        let list = PaintArtifact {
                            items: vec![DisplayItem {
                                r#type: T::kDrawPath,
                                path: circle(x, y, w, w * aspect, reverse),
                                color: Color {
                                    red: 0.13333334,
                                    green: 0.3,
                                    blue: 0.7,
                                    alpha,
                                },
                                antialias: true,
                                ..Default::default()
                            }]
                            .into(),
                            ..Default::default()
                        };
                        let a =
                            raster::source_replay::RasterizeSourceDisplayItemList(&list, 512, 64);
                        let b = raster::pure_replay::RasterizeSourceDisplayItemList(&list, 512, 64);
                        let diffs: Vec<_> = a
                            .chunks_exact(4)
                            .zip(b.chunks_exact(4))
                            .enumerate()
                            .filter(|(_, (a, b))| a != b)
                            .collect();
                        if !diffs.is_empty() {
                            failures += 1;
                            eprintln!("x={x} y={y} w={w} aspect={aspect} reverse={reverse} alpha={alpha} diffs={} first={:?}",diffs.len(),diffs[0]);
                            first.get_or_insert_with(||format!("xy={x},{y} w={w} aspect={aspect} reverse={reverse} alpha={alpha}: {} pixels; first {:?}",diffs.len(),diffs[0]));
                        }
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 96);
    assert_eq!(failures, 0, "{}", first.unwrap_or_default());
}

#[test]
fn cubic_ring_winding_and_even_odd_match_native_skia() {
    let mut failures = 0;
    let mut cases = 0;
    for (x, y) in [(8.0, 9.0), (8.25, 9.25), (393.0, 9.0)] {
        for w in [6.666, 17.0, 38.0] {
            for inset in [0.75, 1.5] {
                for even_odd in [false, true] {
                    for alpha in [1.0, 0.3] {
                        let mut path = circle(x, y, w, w, false);
                        path.extend(circle(
                            x + inset,
                            y + inset,
                            w - 2.0 * inset,
                            w - 2.0 * inset,
                            !even_odd,
                        ));
                        let list = PaintArtifact {
                            items: vec![DisplayItem {
                                r#type: T::kDrawPath,
                                path,
                                even_odd,
                                antialias: true,
                                color: Color {
                                    red: 0.13333334,
                                    green: 0.3,
                                    blue: 0.7,
                                    alpha,
                                },
                                ..Default::default()
                            }]
                            .into(),
                            ..Default::default()
                        };
                        let a =
                            raster::source_replay::RasterizeSourceDisplayItemList(&list, 512, 64);
                        let b = raster::pure_replay::RasterizeSourceDisplayItemList(&list, 512, 64);
                        let diffs: Vec<_> = a
                            .chunks_exact(4)
                            .zip(b.chunks_exact(4))
                            .enumerate()
                            .filter(|(_, (a, b))| a != b)
                            .collect();
                        if !diffs.is_empty() {
                            failures += 1;
                            eprintln!("ring x={x} y={y} w={w} inset={inset} even_odd={even_odd} alpha={alpha}: {} pixels; first {:?}",diffs.len(),diffs[0]);
                        }
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 72);
    assert_eq!(failures, 0);
}

#[test]
fn live_svg_ring_contours_match_native_skia() {
    let mut cases = 0;
    let mut failures = 0;
    for name in ["hexagon", "circle"] {
        for (x, y) in [(8.0, 9.0), (8.25, 9.25), (393.0, 9.0)] {
            for even_odd in [false, true] {
                for alpha in [1.0, 0.3] {
                    let path = svg_ring(name, x, y);
                    assert!(!path.is_empty());
                    let list = PaintArtifact {
                        items: vec![DisplayItem {
                            r#type: T::kDrawPath,
                            path,
                            even_odd,
                            antialias: true,
                            color: Color {
                                red: 0.13333334,
                                green: 0.3,
                                blue: 0.7,
                                alpha,
                            },
                            ..Default::default()
                        }]
                        .into(),
                        ..Default::default()
                    };
                    let a = raster::source_replay::RasterizeSourceDisplayItemList(&list, 512, 64);
                    let b = raster::pure_replay::RasterizeSourceDisplayItemList(&list, 512, 64);
                    let diffs: Vec<_> = a
                        .chunks_exact(4)
                        .zip(b.chunks_exact(4))
                        .enumerate()
                        .filter(|(_, (a, b))| a != b)
                        .collect();
                    if !diffs.is_empty() {
                        failures += 1;
                        eprintln!("svg {name} x={x} y={y} even_odd={even_odd} alpha={alpha}: {} pixels; first {:?}",diffs.len(),diffs[0]);
                    }
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 24);
    assert_eq!(failures, 0);
}

fn svg_ring(name: &str, x: f64, y: f64) -> Vec<PaintPathCommand> {
    include_str!("fixtures/live_svg_rings.tsv")
        .lines()
        .filter_map(|line| {
            let f: Vec<_> = line.split('\t').collect();
            if f[0] != name {
                return None;
            }
            let n = |i: usize| f[i].parse::<f64>().unwrap();
            let p = |i| Offset {
                x: n(i) + x,
                y: n(i + 1) + y,
            };
            Some(PaintPathCommand {
                verb: match f[1] {
                    "kMoveTo" => V::kMoveTo,
                    "kLineTo" => V::kLineTo,
                    "kCubicTo" => V::kCubicTo,
                    "kClose" => V::kClose,
                    _ => panic!("unknown fixture verb"),
                },
                control1: p(2),
                control2: p(4),
                point: p(6),
                conic_weight: n(8),
            })
        })
        .collect::<Vec<_>>()
}

#[test]
fn clipped_svg_rings_match_native_skia() {
    use paint::paint_engine::PaintRect;
    let mut cases = 0;
    let mut failures = 0;
    for name in ["hexagon", "circle"] {
        for (x, y) in [(8.0, 9.0), (8.25, 9.25), (252.0, 9.0)] {
            for alpha in [1.0, 0.3] {
                for profile in 0..4 {
                    let mut items = vec![];
                    if profile < 3 {
                        items.push(DisplayItem {
                            r#type: T::kClipRect,
                            rect: if profile == 2 {
                                PaintRect {
                                    x: x + 1.0,
                                    y: y + 1.0,
                                    width: 12.0,
                                    height: 14.0,
                                }
                            } else {
                                PaintRect {
                                    x,
                                    y,
                                    width: 16.236,
                                    height: 17.832,
                                }
                            },
                            antialias: profile == 1,
                            ..Default::default()
                        });
                    }
                    items.push(DisplayItem {
                        r#type: T::kDrawPath,
                        path: svg_ring(name, x, y),
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
                    let a = raster::source_replay::RasterizeSourceDisplayItemList(&list, 512, 64);
                    let b = raster::pure_replay::RasterizeSourceDisplayItemList(&list, 512, 64);
                    let diffs: Vec<_> = a
                        .chunks_exact(4)
                        .zip(b.chunks_exact(4))
                        .enumerate()
                        .filter(|(_, (a, b))| a != b)
                        .collect();
                    if !diffs.is_empty() {
                        failures += 1;
                        eprintln!("clipped svg {name} x={x} y={y} profile={profile} alpha={alpha}: {} pixels; first {:?}",diffs.len(),diffs[0]);
                    }
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 48);
    assert_eq!(failures, 0);
}
