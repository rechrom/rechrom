//! Independent native Skia oracle for the F16 rounded-span optimization.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};

fn command(kind: T, rect: PaintRect) -> DisplayItem {
    DisplayItem {
        r#type: kind,
        rect,
        ..Default::default()
    }
}
fn rect(x: f64, y: f64, width: f64, height: f64) -> PaintRect {
    PaintRect {
        x,
        y,
        width,
        height,
    }
}
fn radii() -> PaintCornerRadii {
    let r = PaintCornerRadius { x: 19.25, y: 15.5 };
    PaintCornerRadii {
        top_left: r,
        top_right: r,
        bottom_left: r,
        bottom_right: r,
    }
}

#[test]
fn filtered_n32_children_restore_into_f16_parent_matches_original_skia() {
    use layoutng_assembly::internal::paint_input::{PaintFilterOperation, PaintFilterType};
    let mut failures = Vec::new();
    for opacity in [0.37, 1.0] {
        for sigma in [0.5, 2.0, 7.0] {
            for aa_clip in [false, true] {
                let list = PaintArtifact {
                    items: vec![
                        DisplayItem {
                            color: Color {
                                red: 0.19,
                                green: 0.57,
                                blue: 0.83,
                                alpha: 1.0,
                            },
                            ..command(T::kDrawRect, rect(0.0, 0.0, 520.0, 220.0))
                        },
                        DisplayItem {
                            opacity,
                            ..command(T::kSaveLayerAlpha, rect(43.0, 37.0, 382.0, 140.0))
                        },
                        DisplayItem {
                            antialias: aa_clip,
                            corner_radii: radii(),
                            ..command(
                                if aa_clip {
                                    T::kClipRoundedRect
                                } else {
                                    T::kClipRect
                                },
                                rect(64.25, 54.25, 290.0, 84.75),
                            )
                        },
                        DisplayItem {
                            filters: vec![PaintFilterOperation {
                                r#type: PaintFilterType::kBlur,
                                amount: sigma,
                                ..Default::default()
                            }],
                            ..command(T::kSaveLayerFilter, PaintRect::default())
                        },
                        DisplayItem {
                            color: Color {
                                red: 0.0,
                                green: 0.0,
                                blue: 0.0,
                                alpha: 0.63,
                            },
                            antialias: true,
                            corner_radii: radii(),
                            ..command(T::kDrawRoundedRect, rect(70.125, 59.375, 276.25, 77.75))
                        },
                        command(T::kRestore, PaintRect::default()),
                        command(T::kRestore, PaintRect::default()),
                    ]
                    .into(),
                    ..Default::default()
                };
                let native =
                    renderer::source_replay::RasterizeSourceDisplayItemList(&list, 520, 220);
                let actual = renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 520, 220);
                assert_eq!(actual.len(), native.len());
                let different = actual
                    .chunks_exact(4)
                    .zip(native.chunks_exact(4))
                    .filter(|(a, b)| a != b)
                    .count();
                if different != 0 {
                    failures.push(format!(
                        "opacity={opacity} sigma={sigma} aa_clip={aa_clip}: {different} pixels"
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn f16_rounded_spans_match_original_skia() {
    let mut failures = Vec::new();
    for x in [-3.25, 27.0, 252.125] {
        for aa_clip in [false, true] {
            for nested in [false, true] {
                for alpha in [0.37, 1.0] {
                    let mut items = vec![
                        DisplayItem {
                            color: Color {
                                red: 0.19,
                                green: 0.57,
                                blue: 0.83,
                                alpha: 1.0,
                            },
                            ..command(T::kDrawRect, rect(0.0, 0.0, 520.0, 220.0))
                        },
                        DisplayItem {
                            opacity: 0.73,
                            ..command(T::kSaveLayerAlpha, rect(0.0, 0.0, 520.0, 220.0))
                        },
                    ];
                    if nested {
                        items.push(DisplayItem {
                            opacity: 0.61,
                            ..command(T::kSaveLayerAlpha, rect(x - 1.0, 19.0, 300.0, 120.0))
                        });
                    }
                    items.push(DisplayItem {
                        antialias: aa_clip,
                        corner_radii: radii(),
                        ..command(
                            if aa_clip {
                                T::kClipRoundedRect
                            } else {
                                T::kClipRect
                            },
                            rect(x + 2.25, 24.25, 282.0, 96.75),
                        )
                    });
                    for (offset, a) in [(0.0, alpha), (9.125, 0.71)] {
                        items.push(DisplayItem {
                            color: Color {
                                red: 0.83,
                                green: 0.29,
                                blue: 0.67,
                                alpha: a,
                            },
                            antialias: true,
                            corner_radii: radii(),
                            ..command(
                                T::kDrawRoundedRect,
                                rect(x + offset, 21.375 + offset, 290.25, 100.75),
                            )
                        });
                    }
                    if nested {
                        items.push(command(T::kRestore, PaintRect::default()));
                    }
                    items.push(command(T::kRestore, PaintRect::default()));
                    let list = PaintArtifact {
                        items,
                        ..Default::default()
                    };
                    let native =
                        renderer::source_replay::RasterizeSourceDisplayItemList(&list, 520, 220);
                    let actual =
                        renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 520, 220);
                    assert_eq!(actual.len(), native.len());
                    let different = actual
                        .chunks_exact(4)
                        .zip(native.chunks_exact(4))
                        .filter(|(a, b)| a != b)
                        .count();
                    if different != 0 {
                        for (i, (a, b)) in actual
                            .chunks_exact(4)
                            .zip(native.chunks_exact(4))
                            .enumerate()
                            .filter(|(_, (a, b))| a != b)
                            .take(8)
                        {
                            eprintln!("x={x} nested={nested} alpha={alpha} at={},{} actual={a:?} native={b:?}",i%520,i/520);
                        }
                        failures.push(format!("x={x} aa_clip={aa_clip} nested={nested} alpha={alpha}: {different} pixels"));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
