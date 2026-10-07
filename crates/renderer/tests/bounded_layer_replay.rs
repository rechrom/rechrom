//! Cropped raster devices are checked against the independent C++ Skia replay.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};
fn item(kind: T) -> DisplayItem {
    DisplayItem {
        r#type: kind,
        ..Default::default()
    }
}
fn rectangle(x: f64, y: f64, w: f64, h: f64) -> PaintRect {
    PaintRect {
        x,
        y,
        width: w,
        height: h,
    }
}
#[test]
fn bounded_nested_n32_f16_and_empty_layers_match_native_skia() {
    for (x, y) in [(13.0, 17.0), (252.0, 252.0), (-3.0, -2.0), (450.0, 450.0)] {
        for kind in [T::kSaveLayer, T::kSaveLayerAlpha] {
            for aa in [false, true] {
                for nested in [false, true] {
                    let radius = PaintCornerRadius { x: 5.0, y: 5.0 };
                    let mut items = vec![
                        DisplayItem {
                            rect: rectangle(0.0, 0.0, 400.0, 340.0),
                            color: Color {
                                red: 0.3,
                                green: 0.5,
                                blue: 0.7,
                                alpha: 1.0,
                            },
                            ..item(T::kDrawRect)
                        },
                        DisplayItem {
                            rect: rectangle(x + 0.25, y + 0.25, 31.0, 27.0),
                            antialias: aa,
                            corner_radii: if aa {
                                PaintCornerRadii {
                                    top_left: radius,
                                    top_right: radius,
                                    bottom_left: radius,
                                    bottom_right: radius,
                                }
                            } else {
                                Default::default()
                            },
                            ..item(if aa {
                                T::kClipRoundedRect
                            } else {
                                T::kClipRect
                            })
                        },
                        DisplayItem {
                            rect: rectangle(x + 2.0, y + 1.0, 25.0, 21.0),
                            opacity: if kind == T::kSaveLayerAlpha {
                                0.63
                            } else {
                                1.0
                            },
                            ..item(kind)
                        },
                    ];
                    if nested {
                        items.push(DisplayItem {
                            rect: rectangle(x + 4.0, y + 3.0, 16.0, 14.0),
                            opacity: 0.71,
                            ..item(T::kSaveLayerAlpha)
                        });
                    }
                    items.push(DisplayItem {
                        rect: rectangle(x, y, 40.0, 34.0),
                        color: Color {
                            red: 0.9,
                            green: 0.2,
                            blue: 0.4,
                            alpha: 0.4,
                        },
                        ..item(T::kDrawRect)
                    });
                    items.push(DisplayItem {
                        rect: rectangle(x + 6.0, y + 4.0, 10.0, 9.0),
                        color: Color {
                            red: 0.2,
                            green: 0.8,
                            blue: 0.3,
                            alpha: 0.7,
                        },
                        ..item(T::kDrawRect)
                    });
                    if nested {
                        items.push(item(T::kRestore));
                    }
                    items.push(item(T::kRestore));
                    // This following command also checks restoring the parent coordinates.
                    items.push(DisplayItem {
                        rect: rectangle(x + 1.0, y + 23.0, 10.0, 2.0),
                        color: Color {
                            red: 0.1,
                            green: 0.2,
                            blue: 0.3,
                            alpha: 1.0,
                        },
                        ..item(T::kDrawRect)
                    });
                    let list = PaintArtifact {
                        items,
                        ..Default::default()
                    };
                    let expected =
                        renderer::source_replay::RasterizeSourceDisplayItemList(&list, 400, 340);
                    let actual =
                        renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 400, 340);
                    let diff = actual.iter().zip(&expected).filter(|(a, b)| a != b).count();
                    assert_eq!(diff, 0, "x={x} y={y} kind={kind:?} aa={aa} nested={nested}");
                }
            }
        }
    }
}

#[test]
fn blur_layer_samples_outside_output_clip_and_keeps_padding_transparent() {
    use layoutng_assembly::internal::paint_input::{PaintFilterOperation, PaintFilterType};
    for (x, y) in [(0.0, 0.0), (31.0, 27.0), (251.0, 253.0)] {
        for sigma in [0.5, 2.0, 5.0] {
            let list = PaintArtifact {
                items: vec![
                    DisplayItem {
                        rect: rectangle(x, y, 18.0, 15.0),
                        ..item(T::kClipRect)
                    },
                    DisplayItem {
                        filters: vec![PaintFilterOperation {
                            r#type: PaintFilterType::kBlur,
                            amount: sigma,
                            ..Default::default()
                        }],
                        ..item(T::kSaveLayerFilter)
                    },
                    // Source crosses the parent clip; it must contribute to blurred output.
                    DisplayItem {
                        rect: rectangle(x - 3.0, y - 3.0, 12.0, 11.0),
                        color: Color {
                            red: 0.8,
                            green: 0.2,
                            blue: 0.4,
                            alpha: 0.6,
                        },
                        ..item(T::kDrawRect)
                    },
                    item(T::kRestore),
                ]
                .into(),
                ..Default::default()
            };
            let expected = renderer::source_replay::RasterizeSourceDisplayItemList(&list, 320, 320);
            let actual = renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 320, 320);
            let diff = actual.iter().zip(&expected).filter(|(a, b)| a != b).count();
            assert_eq!(diff, 0, "x={x} y={y} sigma={sigma}");
        }
    }
}
