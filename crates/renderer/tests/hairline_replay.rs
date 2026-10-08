//! Full RGBA differential checks for Skia's AA hairline scan profile.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb as V};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};
fn radii(r: f64) -> PaintCornerRadii {
    let r = PaintCornerRadius { x: r, y: r };
    PaintCornerRadii {
        top_left: r,
        top_right: r,
        bottom_left: r,
        bottom_right: r,
    }
}
fn compare(item: DisplayItem, clip: Option<DisplayItem>, label: &str) -> usize {
    let mut items = Vec::new();
    if let Some(c) = clip {
        items.push(c);
    }
    items.push(item);
    let list = PaintArtifact {
        items,
        ..Default::default()
    };
    let a = raster::source_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
    let b = raster::pure_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
    let diff: Vec<_> = a
        .chunks_exact(4)
        .zip(b.chunks_exact(4))
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .collect();
    if !diff.is_empty() {
        eprintln!("{label}: {} pixels; first {:?}", diff.len(), diff[0]);
        1
    } else {
        0
    }
}
#[test]
fn rounded_uniform_hairline_borders_match_skia() {
    let mut failures = 0;
    let mut cases = 0;
    for (x, y) in [(956.0, 586.0), (252.25, 249.25), (8.125, 9.375)] {
        for w in [1.0, 0.625] {
            for r in [9.0, 13.0] {
                for (alpha, black) in [(1.0, false), (0.3, false), (1.0, true)] {
                    for clipped in [false, true] {
                        let item = DisplayItem {
                            r#type: T::kDrawDoubleRoundedRect,
                            rect: PaintRect {
                                x,
                                y,
                                width: 44.0,
                                height: 138.0,
                            },
                            inner_rect: PaintRect {
                                x: x + w,
                                y: y + w,
                                width: 44.0 - 2.0 * w,
                                height: 138.0 - 2.0 * w,
                            },
                            corner_radii: radii(r),
                            inner_corner_radii: radii(r - w),
                            color: Color {
                                red: if black { 0.0 } else { 219.0 / 255.0 },
                                green: if black { 0.0 } else { 220.0 / 255.0 },
                                blue: if black { 0.0 } else { 224.0 / 255.0 },
                                alpha,
                            },
                            antialias: true,
                            ..Default::default()
                        };
                        let clip = clipped.then_some(DisplayItem {
                            r#type: T::kClipRect,
                            rect: PaintRect {
                                x: x + 2.0,
                                y: y + 1.0,
                                width: 38.0,
                                height: 136.0,
                            },
                            ..Default::default()
                        });
                        failures+=compare(item,clip,&format!("border xy={x},{y} w={w} r={r} alpha={alpha} black={black} clip={clipped}"));
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 72);
    assert_eq!(failures, 0);
}
fn command(v: V, x: f64, y: f64) -> PaintPathCommand {
    PaintPathCommand {
        verb: v,
        point: Offset { x, y },
        ..Default::default()
    }
}
#[test]
fn butt_hairline_lines_quads_conics_cubics_match_skia() {
    let mut failures = 0;
    let mut cases = 0;
    for (x, y) in [(12.0, 13.0), (12.25, 13.25), (252.25, 249.25)] {
        for w in [0.0, 0.5, 1.0] {
            for alpha in [1.0, 0.3] {
                for kind in 0..4 {
                    let mut p = command(V::kLineTo, x + 18.0, y + 23.0);
                    match kind {
                        1 => {
                            p.verb = V::kQuadraticTo;
                            p.control1 = Offset {
                                x: x + 27.0,
                                y: y + 4.0,
                            };
                        }
                        2 => {
                            p.verb = V::kConicTo;
                            p.control1 = Offset {
                                x: x + 27.0,
                                y: y + 4.0,
                            };
                            p.conic_weight = 0.70710677;
                        }
                        3 => {
                            p.verb = V::kCubicTo;
                            p.control1 = Offset {
                                x: x + 12.0,
                                y: y + 1.0,
                            };
                            p.control2 = Offset {
                                x: x + 4.0,
                                y: y + 22.0,
                            };
                        }
                        _ => {}
                    }
                    let item = DisplayItem {
                        r#type: T::kStrokePath,
                        path: vec![command(V::kMoveTo, x, y), p],
                        stroke_width: w,
                        color: Color {
                            red: 0.2,
                            green: 0.4,
                            blue: 0.7,
                            alpha,
                        },
                        antialias: true,
                        ..Default::default()
                    };
                    failures += compare(
                        item,
                        None,
                        &format!("path xy={x},{y} w={w} alpha={alpha} kind={kind}"),
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 72);
    assert_eq!(failures, 0);
}
