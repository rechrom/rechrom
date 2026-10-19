//! Skia differential checks for thin transformed strokes and empty fills.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb as V};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{SvgStrokeLineCap, SvgStrokeLineJoin};
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};

fn compare(items: Vec<DisplayItem>, label: &str) {
    let list = PaintArtifact {
        items,
        ..Default::default()
    };
    let native = raster::source_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
    let rust = raster::pure_replay::RasterizeSourceDisplayItemList(&list, 1024, 768);
    let differences: Vec<_> = native
        .chunks_exact(4)
        .zip(rust.chunks_exact(4))
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .collect();
    assert!(
        differences.is_empty(),
        "{label}: {} pixels; first {:?}",
        differences.len(),
        differences.first()
    );
}

fn transform(x: f64, y: f64, scale: f64, angle: f64) -> DisplayItem {
    let mut item = DisplayItem {
        r#type: T::kConcat,
        ..Default::default()
    };
    let (sin, cos) = angle.to_radians().sin_cos();
    item.transform.values[0] = scale * cos;
    item.transform.values[1] = scale * sin;
    item.transform.values[4] = -scale * sin;
    item.transform.values[5] = scale * cos;
    item.transform.values[12] = x;
    item.transform.values[13] = y;
    item
}

#[test]
fn transformed_thin_chevron_caps_match_skia() {
    for (x, y) in [(32.0, 40.0), (252.125, 253.375), (968.0, 428.0)] {
        for (scale, width) in [(2.0 / 3.0, 1.2), (0.5, 0.75)] {
            for angle in [0.0, -90.0, 45.0, 90.0] {
                for cap in [
                    SvgStrokeLineCap::kButt,
                    SvgStrokeLineCap::kRound,
                    SvgStrokeLineCap::kSquare,
                ] {
                    for alpha in [1.0, 0.3] {
                        let path = [
                            (V::kMoveTo, 19.0, 9.0),
                            (V::kLineTo, 12.0, 16.0),
                            (V::kLineTo, 5.0, 9.0),
                        ]
                        .into_iter()
                        .map(|(verb, x, y)| PaintPathCommand {
                            verb,
                            point: Offset { x, y },
                            ..Default::default()
                        })
                        .collect();
                        compare(vec![transform(x, y, scale, angle), DisplayItem {
                            r#type: T::kStrokePath, path, stroke_width: width,
                            svg_line_cap: cap, svg_line_join: SvgStrokeLineJoin::kRound,
                            color: Color { red: 0.2, green: 0.4, blue: 0.7, alpha },
                            antialias: true, ..Default::default()
                        }], &format!("xy={x},{y} scale={scale} width={width} angle={angle} cap={cap:?} alpha={alpha}"));
                    }
                }
            }
        }
    }
}

#[test]
fn empty_filled_rectangles_match_skia_before_rounding() {
    for (width, height) in [(0.0, 2.0), (2.0, 0.0), (0.0, 0.0), (-1.0, 2.0), (2.0, -1.0)] {
        for antialias in [false, true] {
            for transformed in [false, true] {
                for layer in [false, true] {
                    let mut items = vec![];
                    if layer {
                        items.push(DisplayItem {
                            r#type: T::kSaveLayer,
                            ..Default::default()
                        });
                    }
                    if transformed {
                        items.push(transform(10.25, 20.5, 2.0 / 3.0, 45.0));
                    }
                    items.push(DisplayItem {
                        r#type: T::kDrawRect,
                        rect: PaintRect {
                            x: 31.25,
                            y: 38.25,
                            width,
                            height,
                        },
                        color: Color {
                            red: 0.2,
                            green: 0.4,
                            blue: 0.7,
                            alpha: 1.0,
                        },
                        antialias,
                        ..Default::default()
                    });
                    if layer {
                        items.push(DisplayItem {
                            r#type: T::kRestore,
                            ..Default::default()
                        });
                    }
                    compare(
                        items,
                        &format!(
                            "{width}x{height} aa={antialias} transform={transformed} layer={layer}"
                        ),
                    );
                }
            }
        }
    }
}
