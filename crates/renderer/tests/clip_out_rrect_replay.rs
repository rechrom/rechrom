//! Real, linked original Skia ClipRRect(Difference) full-frame pixel oracle.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]
use layoutng_assembly::internal::layout_input::TransformMatrix;
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};

fn rect(x: f64, y: f64, width: f64, height: f64) -> PaintRect {
    PaintRect {
        x,
        y,
        width,
        height,
    }
}
fn item(kind: T) -> DisplayItem {
    DisplayItem {
        r#type: kind,
        ..Default::default()
    }
}
fn radius(x: f64, y: f64) -> PaintCornerRadius {
    PaintCornerRadius { x, y }
}
fn uniform(x: f64, y: f64) -> PaintCornerRadii {
    let r = radius(x, y);
    PaintCornerRadii {
        top_left: r,
        top_right: r,
        bottom_left: r,
        bottom_right: r,
    }
}

#[test]
fn clip_out_rrect_matches_original_skia_all_pixels() {
    let geometry = [
        (rect(12.0, 14.0, 52.0, 38.0), uniform(8.0, 8.0)),
        (rect(12.25, 14.375, 52.125, 38.5), uniform(19.25, 13.75)),
        (
            rect(-9.25, 4.375, 49.25, 68.5),
            PaintCornerRadii {
                top_left: radius(6.25, 17.75),
                top_right: radius(22.0, 4.25),
                bottom_right: radius(13.5, 21.25),
                bottom_left: radius(0.0, 14.0),
            },
        ),
        (rect(8.25, 8.75, 66.5, 50.25), uniform(90.0, 70.0)), // Native radius normalization/oval.
        (rect(12.25, 14.375, 52.0, 38.0), uniform(0.0, 0.0)), // Native onClipRect dispatch.
        (rect(12.0625, 14.0625, 51.875, 37.875), uniform(-1.0, -1.0)), // BW near-integer gate.
        (rect(120.25, 110.75, 30.0, 20.0), uniform(6.0, 4.0)), // Entirely outside prior bounds.
        (rect(-120.0, -100.0, 500.0, 400.0), uniform(12.0, 12.0)), // Entire prior clip removed.
        (rect(30.25, 25.25, 0.0, 8.0), uniform(4.0, 4.0)),    // Empty hole is identity.
    ];
    let mut cases = 0;
    let mut failures = Vec::new();
    for (shape, radii) in geometry {
        for aa in [false, true] {
            for prior in 0..3 {
                for transform in 0..4 {
                    for layer in [false, true] {
                        let mut commands = vec![
                            DisplayItem {
                                rect: rect(0.0, 0.0, 96.0, 80.0),
                                color: Color {
                                    red: 0.17,
                                    green: 0.39,
                                    blue: 0.73,
                                    alpha: 1.0,
                                },
                                ..item(T::kDrawRect)
                            },
                            item(T::kSave),
                        ];
                        if prior != 0 {
                            commands.push(DisplayItem {
                                rect: rect(7.25, 5.375, 76.5, 64.25),
                                corner_radii: if prior == 2 {
                                    uniform(12.5, 8.75)
                                } else {
                                    uniform(0.0, 0.0)
                                },
                                antialias: prior == 2,
                                ..item(if prior == 2 {
                                    T::kClipRoundedRect
                                } else {
                                    T::kClipRect
                                })
                            });
                        }
                        if layer {
                            commands.push(DisplayItem {
                                rect: rect(5.0, 4.0, 82.0, 70.0),
                                opacity: 0.63,
                                ..item(T::kSaveLayerAlpha)
                            });
                        }
                        commands.push(item(T::kSave));
                        let mut m = TransformMatrix::default();
                        match transform {
                            1 => {
                                m.values[0] = 0.8;
                                m.values[5] = 1.1;
                                m.values[12] = 3.25;
                                m.values[13] = -2.125;
                            }
                            2 => {
                                m.values[0] = -1.0;
                                m.values[12] = 96.0;
                            }
                            3 => {
                                m.values[0] = 0.85;
                                m.values[1] = 0.125;
                                m.values[4] = 0.2;
                                m.values[5] = 0.9;
                            }
                            _ => {}
                        }
                        commands.push(DisplayItem {
                            transform: m,
                            ..item(T::kConcat)
                        });
                        commands.push(DisplayItem {
                            rect: shape,
                            corner_radii: radii,
                            antialias: aa,
                            ..item(T::kClipOutRoundedRect)
                        });
                        commands.push(DisplayItem {
                            rect: rect(42.125, 35.25, 23.25, 16.75),
                            corner_radii: uniform(4.25, 6.75),
                            antialias: !aa,
                            ..item(T::kClipOutRoundedRect)
                        }); // Compound BW/AA Difference with the first hole.
                        commands.push(DisplayItem {
                            rect: rect(-200.0, -200.0, 500.0, 500.0),
                            color: Color {
                                red: 0.91,
                                green: 0.13,
                                blue: 0.31,
                                alpha: 0.71,
                            },
                            ..item(T::kDrawRect)
                        });
                        commands.push(item(T::kRestore));
                        if layer {
                            commands.push(item(T::kRestore));
                        }
                        commands.push(item(T::kRestore));
                        commands.push(DisplayItem {
                            rect: rect(84.0, 71.0, 8.0, 6.0),
                            color: Color {
                                red: 0.1,
                                green: 0.9,
                                blue: 0.3,
                                alpha: 1.0,
                            },
                            ..item(T::kDrawRect)
                        }); // Restored clip must not inherit either hole.
                        let list = PaintArtifact {
                            items: commands.into(),
                            ..Default::default()
                        };
                        let native =
                            renderer::source_replay::RasterizeSourceDisplayItemList(&list, 96, 80);
                        let actual =
                            renderer::pure_replay::RasterizeSourceDisplayItemList(&list, 96, 80);
                        assert_eq!(native.len(), 96 * 80 * 4);
                        assert_eq!(actual.len(), native.len());
                        let different = actual.iter().zip(&native).filter(|(a, b)| a != b).count();
                        if different != 0 {
                            let first = actual
                                .chunks_exact(4)
                                .zip(native.chunks_exact(4))
                                .enumerate()
                                .find(|(_, (a, b))| a != b)
                                .unwrap();
                            failures.push(format!("case={cases} shape={shape:?} radii={radii:?} aa={aa} prior={prior} transform={transform} layer={layer} differentBytes={different} first={first:?}"));
                            if let Some(path) = std::env::var_os("CLIP_OUT_ORACLE_OUTPUT") {
                                let p = std::path::PathBuf::from(path);
                                std::fs::create_dir_all(&p).unwrap();
                                std::fs::write(
                                    p.join(format!("case-{cases}-native.rgba")),
                                    &native,
                                )
                                .unwrap();
                                std::fs::write(
                                    p.join(format!("case-{cases}-actual.rgba")),
                                    &actual,
                                )
                                .unwrap();
                            }
                        }
                        cases += 1;
                    }
                }
            }
        }
    }
    println!(
        "original-Skia-difference cases={cases} full_bytes_each={} differing_cases={}",
        96 * 80 * 4,
        failures.len()
    );
    assert_eq!(cases, 432);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
