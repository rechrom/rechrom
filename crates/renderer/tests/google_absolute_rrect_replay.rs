//! Match the real window surface against one whole original-Skia surface.
//! The old tiled native helper changes absolute conic decomposition coordinates
//! and is deliberately not used as this oracle.
#![cfg(all(
    feature = "pure_replay",
    feature = "source_replay",
    feature = "profiling"
))]
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};

fn scene(x: f64, y: f64, width: f64, height: f64, radius: f64) -> PaintArtifact {
    let r = PaintCornerRadius {
        x: radius,
        y: radius,
    };
    PaintArtifact {
        items: vec![
            DisplayItem {
                r#type: T::kDrawRect,
                rect: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 1280.0,
                    height: 720.0,
                },
                color: Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
                ..Default::default()
            },
            DisplayItem {
                r#type: T::kDrawRoundedRect,
                rect: PaintRect {
                    x,
                    y,
                    width,
                    height,
                },
                corner_radii: PaintCornerRadii {
                    top_left: r,
                    top_right: r,
                    bottom_left: r,
                    bottom_right: r,
                },
                color: Color {
                    red: 11.0 / 255.0,
                    green: 87.0 / 255.0,
                    blue: 208.0 / 255.0,
                    alpha: 1.0,
                },
                antialias: true,
                ..Default::default()
            },
        ]
        .into(),
        ..Default::default()
    }
}
fn compare(list: &PaintArtifact, width: u32, height: u32, scale: f64) {
    let (native, _) =
        renderer::source_replay::ProfileSourceDisplayItemListWithScale(list, width, height, scale);
    let owned =
        renderer::pure_replay::RasterizeSourceDisplayItemListWithScale(list, width, height, scale);
    let mut failures = Vec::new();
    let different = owned
        .chunks_exact(4)
        .zip(native.chunks_exact(4))
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .collect::<Vec<_>>();
    if !different.is_empty() {
        failures.push(format!(
            "owned mismatch={} first={:?}",
            different.len(),
            different.first()
        ));
    }
    for format in [
        skia::PixelFormat::Rgba8888,
        skia::PixelFormat::Bgra8888,
        skia::PixelFormat::Bgrx8888,
    ] {
        let guard = 0x9d_a7_b3_c5u32;
        let mut storage = vec![guard; (width * height) as usize + 32];
        let target = &mut storage[16..16 + (width * height) as usize];
        renderer::surface::RenderDisplayItemListIntoWindowBufferWithFormat(
            list, width, height, scale, target, format,
        )
        .unwrap();
        let mut different = 0;
        let mut first = None;
        for (i, (actual, expected)) in target.iter().zip(native.chunks_exact(4)).enumerate() {
            let bytes = actual.to_le_bytes();
            let rgba = if format == skia::PixelFormat::Rgba8888 {
                bytes
            } else {
                [
                    bytes[2],
                    bytes[1],
                    bytes[0],
                    if format == skia::PixelFormat::Bgrx8888 {
                        255
                    } else {
                        bytes[3]
                    },
                ]
            };
            if rgba != expected {
                different += 1;
                if first.is_none() {
                    first = Some((i, rgba, expected.to_vec()));
                }
            }
            if format == skia::PixelFormat::Bgrx8888 {
                assert_eq!(bytes[3], 0);
            }
        }
        assert!(
            storage[..16]
                .iter()
                .chain(storage[16 + (width * height) as usize..].iter())
                .all(|&v| v == guard),
            "outside buffer write: {format:?}"
        );
        if different != 0 {
            failures.push(format!("{format:?} mismatch={different} first={first:?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "rect={:?} scale={scale} {}",
        list.items[1].rect,
        failures.join("; ")
    );
}
#[test]
fn google_sign_in_absolute_device_coordinates_match_original_whole_canvas() {
    // Actual Google top-right Sign-in control in the captured 1280x720 CSS
    // viewport. Keep identical dimensions, DPR and direct window storage.
    compare(&scene(1185.0, 9.0, 85.0, 40.0, 20.0), 2560, 1440, 2.0);
}
#[test]
fn translated_rounded_controls_keep_original_global_conic_rounding() {
    for (x, y, w, h, r) in [
        (80.0, 9.0, 85.0, 40.0, 20.0),
        (499.5, 327.5, 127.0, 35.0, 17.5),
        (879.0, 254.0, 96.0, 36.0, 18.0),
        (1185.25, 9.125, 85.0, 40.0, 20.0),
    ] {
        compare(&scene(x, y, w, h, r), 2560, 1440, 2.0);
    }
}

#[test]
fn virtual_tile_halo_crops_keep_color_pair_semantics_and_buffer_guards() {
    // Global 255/509/763/... ownership boundaries and their one-pixel halos.
    // The x80 control has an H2 coverage56 pixel, whose AntiH rounding differs.
    for (x, y) in [
        (126.5, 9.0),
        (127.0, 9.0),
        (127.5, 9.0),
        (253.5, 126.5),
        (254.0, 127.0),
        (254.5, 127.5),
        (380.5, 253.5),
        (80.0, 126.5),
    ] {
        compare(&scene(x, y, 85.0, 40.0, 20.0), 2560, 1440, 2.0);
    }
}
#[test]
fn actual_canvas_geometry_clips_on_both_sides_match_native() {
    for x in [-10.25, 1240.25] {
        compare(&scene(x, 9.125, 85.0, 40.0, 20.0), 2560, 1440, 2.0);
    }
}
#[test]
fn actual_rect_clip_keeps_original_generic_antih_semantics() {
    for left in [80.0, 85.5, 99.5] {
        let mut list = scene(80.0, 9.125, 85.0, 40.0, 20.0);
        std::sync::Arc::make_mut(&mut list.items).insert(
            1,
            DisplayItem {
                r#type: T::kClipRect,
                rect: PaintRect {
                    x: left,
                    y: 10.0,
                    width: 60.0,
                    height: 36.0,
                },
                antialias: false,
                ..Default::default()
            },
        );
        compare(&list, 2560, 1440, 2.0);
    }
}

// Preserve all 36 existing layer/filter cases with a whole-device oracle.
mod whole_canvas_f16_regressions {
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
                    let native = renderer::source_replay::ProfileSourceDisplayItemListWithScale(
                        &list, 520, 220, 1.0,
                    )
                    .0;
                    let actual = renderer::pure_replay::RasterizeSourceDisplayItemListWithScale(
                        &list, 520, 220, 1.0,
                    );
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
                            renderer::source_replay::ProfileSourceDisplayItemListWithScale(
                                &list, 520, 220, 1.0,
                            )
                            .0;
                        let actual = renderer::pure_replay::RasterizeSourceDisplayItemListWithScale(
                            &list, 520, 220, 1.0,
                        );
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
}
