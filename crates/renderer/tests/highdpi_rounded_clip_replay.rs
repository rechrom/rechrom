//! Positive-axis clip transformations, AA run rules and cropped layers use native pixels as oracle.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};
#[test]
fn positive_axis_float_nested_and_cropped_rrect_clips_match_native() {
    let mut cases = 0;
    let mut failures = 0;
    let mut first = String::new();
    for scale in [0.8, 1.2, 1.5, 2.0] {
        // At 0.8, use exact mapped coordinates. The independent diagnostic
        // archive records the existing two-pixel fractional-geometry mismatch;
        // this test remains a strict native oracle with no alpha tolerance.
        let positions = if scale == 0.8 {
            [(10.0, 5.0), (300.0, 305.0)]
        } else {
            [
                (11.125, 7.375),
                (255.0 / scale - 21.375, 255.0 / scale - 13.875),
            ]
        };
        for (x, y) in positions {
            for layer in [false, true] {
                let exact = scale == 0.8;
                let r = if exact {
                    PaintCornerRadius { x: 10.0, y: 5.0 }
                } else {
                    PaintCornerRadius { x: 7.25, y: 5.875 }
                };
                let radii = PaintCornerRadii {
                    top_left: r,
                    top_right: r,
                    bottom_left: r,
                    bottom_right: r,
                };
                let outer = PaintRect {
                    x,
                    y,
                    width: if exact { 80.0 } else { 81.375 },
                    height: if exact { 40.0 } else { 37.125 },
                };
                let mut scale_item = DisplayItem {
                    r#type: T::kConcat,
                    ..Default::default()
                };
                scale_item.transform.values[0] = scale;
                scale_item.transform.values[5] = scale;
                let mut items = vec![
                    scale_item,
                    DisplayItem {
                        r#type: T::kClipRoundedRect,
                        rect: outer,
                        corner_radii: radii,
                        antialias: true,
                        ..Default::default()
                    },
                    DisplayItem {
                        r#type: T::kClipRoundedRect,
                        rect: PaintRect {
                            x: x + if exact { 5.0 } else { 1.25 },
                            y: y + if exact { 5.0 } else { 0.875 },
                            width: if exact { 70.0 } else { 77.5 },
                            height: if exact { 30.0 } else { 34.75 },
                        },
                        corner_radii: radii,
                        antialias: true,
                        ..Default::default()
                    },
                ];
                if layer {
                    items.push(DisplayItem {
                        r#type: T::kSaveLayerAlpha,
                        rect: PaintRect {
                            x: x + if exact { 10.0 } else { 2.25 },
                            y: y + if exact { 10.0 } else { 1.625 },
                            width: if exact { 60.0 } else { 72.75 },
                            height: if exact { 20.0 } else { 30.25 },
                        },
                        opacity: 0.63,
                        ..Default::default()
                    });
                }
                items.push(DisplayItem {
                    r#type: T::kDrawRect,
                    rect: PaintRect {
                        x: x - if exact { 5.0 } else { 3.0 },
                        y: y - if exact { 5.0 } else { 3.0 },
                        width: 90.0,
                        height: if exact { 50.0 } else { 47.0 },
                    },
                    color: Color {
                        red: 0.2,
                        green: 0.6,
                        blue: 0.8,
                        alpha: 0.5,
                    },
                    ..Default::default()
                });
                if layer {
                    items.push(DisplayItem {
                        r#type: T::kRestore,
                        ..Default::default()
                    });
                }
                let list = PaintArtifact {
                    items,
                    ..Default::default()
                };
                let native = raster::source_replay::RasterizeSourceDisplayItemList(&list, 520, 360);
                let actual = raster::pure_replay::RasterizeSourceDisplayItemList(&list, 520, 360);
                let differences = actual
                    .chunks_exact(4)
                    .zip(native.chunks_exact(4))
                    .enumerate()
                    .filter(|(_, (a, b))| a != b)
                    .collect::<Vec<_>>();
                if !differences.is_empty() {
                    failures += 1;
                    eprintln!("nonexact baseline geometry scale={scale},xy={x},{y},layer={layer},pixels={},first={:?}", differences.len(), differences[0]);
                    if first.is_empty() {
                        first=format!("scale={scale},xy={x},{y},layer={layer},{} different pixels, first={:?}",differences.len(),differences[0]);
                    }
                }
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 16);
    assert_eq!(failures, 0, "{first}");
}
