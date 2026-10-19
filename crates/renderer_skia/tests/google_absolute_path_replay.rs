//! Whole original-Skia device paths: global curve chopping before fixed edges.
//! PNG's public entry point has its separate original outer real tile canvases.
#![cfg(all(
    feature = "pure_replay",
    feature = "source_replay",
    feature = "profiling"
))]
use layoutng_assembly::internal::layout_input::{
    Offset, PaintPathCommand as Cmd, PaintPathVerb as V,
};
use layoutng_assembly::internal::layout_input_types::Color;
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};

fn point(x: f64, y: f64) -> Offset {
    Offset { x, y }
}
fn lens(kind: u8, x: f64, y: f64) -> Vec<Cmd> {
    let p = |a, b| point(x + a, y + b);
    let curve = |bottom: bool| {
        let h = if bottom { 55.0 } else { -15.0 };
        Cmd {
            verb: match kind {
                0 => V::kQuadraticTo,
                1 => V::kConicTo,
                _ => V::kCubicTo,
            },
            control1: if kind == 2 {
                p(if bottom { 54.875 } else { 12.1774 }, h)
            } else {
                p(32.625, if bottom { 60.0 } else { -20.0 })
            },
            control2: p(if bottom { 12.1774 } else { 54.875 }, h),
            point: if bottom { p(0.0, 20.0) } else { p(65.25, 20.0) },
            conic_weight: std::f64::consts::FRAC_1_SQRT_2,
            ..Default::default()
        }
    };
    vec![
        Cmd {
            verb: V::kMoveTo,
            point: p(0.0, 20.0),
            ..Default::default()
        },
        curve(false),
        curve(true),
        Cmd {
            verb: V::kClose,
            ..Default::default()
        },
    ]
}
fn crossing(x: f64, y: f64) -> Vec<Cmd> {
    let p = |a, b| point(x + a, y + b);
    vec![
        Cmd {
            verb: V::kMoveTo,
            point: p(0.0, 0.0),
            ..Default::default()
        },
        Cmd {
            verb: V::kCubicTo,
            control1: p(65.25, 80.5),
            control2: p(0.0, 80.5),
            point: p(65.25, 0.0),
            ..Default::default()
        },
        Cmd {
            verb: V::kLineTo,
            point: p(0.0, 65.25),
            ..Default::default()
        },
        Cmd {
            verb: V::kLineTo,
            point: p(65.25, 65.25),
            ..Default::default()
        },
        Cmd {
            verb: V::kClose,
            ..Default::default()
        },
    ]
}
fn circle(x: f64, y: f64, w: f64, reverse: bool) -> Vec<Cmd> {
    let r = w / 2.0;
    let cx = x + r;
    let cy = y + r;
    let k = 0.5522847498307936;
    let ends = [(cx + r, cy), (cx, cy + r), (cx - r, cy), (cx, cy - r)];
    let controls = [
        ((cx + r, cy + k * r), (cx + k * r, cy + r)),
        ((cx - k * r, cy + r), (cx - r, cy + k * r)),
        ((cx - r, cy - k * r), (cx - k * r, cy - r)),
        ((cx + k * r, cy - r), (cx + r, cy - k * r)),
    ];
    let p = |(a, b)| point(a, b);
    let mut out = vec![Cmd {
        verb: V::kMoveTo,
        point: p(ends[0]),
        ..Default::default()
    }];
    for j in 0..4 {
        let i = if reverse { 3 - j } else { j };
        let (a, b) = controls[i];
        out.push(Cmd {
            verb: V::kCubicTo,
            control1: p(if reverse { b } else { a }),
            control2: p(if reverse { a } else { b }),
            point: p(ends[if reverse { i } else { (i + 1) % 4 }]),
            ..Default::default()
        });
    }
    out.push(Cmd {
        verb: V::kClose,
        ..Default::default()
    });
    out
}
fn scene(
    path: Vec<Cmd>,
    translate: Option<(f64, f64)>,
    clip: Option<(PaintRect, bool)>,
    layer: bool,
    even_odd: bool,
) -> PaintArtifact {
    let mut list = PaintArtifact {
        items: vec![DisplayItem {
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
        }]
        .into(),
        ..Default::default()
    };
    if layer {
        std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
            r#type: T::kSaveLayerAlpha,
            opacity: 0.73,
            rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 1280.0,
                height: 720.0,
            },
            ..Default::default()
        });
    }
    if let Some((rect, aa)) = clip {
        std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
            r#type: T::kClipRect,
            rect,
            antialias: aa,
            ..Default::default()
        });
    }
    if let Some((x, y)) = translate {
        let mut transform = DisplayItem {
            r#type: T::kConcat,
            ..Default::default()
        };
        transform.transform.values[12] = x;
        transform.transform.values[13] = y;
        std::sync::Arc::make_mut(&mut list.items).push(transform);
    }
    std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
        r#type: T::kDrawPath,
        path,
        even_odd,
        antialias: true,
        color: Color {
            red: 11.0 / 255.0,
            green: 87.0 / 255.0,
            blue: 208.0 / 255.0,
            alpha: if layer { 0.37 } else { 1.0 },
        },
        ..Default::default()
    });
    if layer {
        std::sync::Arc::make_mut(&mut list.items).push(DisplayItem {
            r#type: T::kRestore,
            ..Default::default()
        });
    }
    list
}
fn compare(list: &PaintArtifact, label: &str, all_failures: &mut Vec<String>) {
    let (width, height, scale) = (2560, 1440, 2.0);
    let (native, _) =
        raster::source_replay::ProfileSourceDisplayItemListWithScale(list, width, height, scale);
    let owned =
        raster::pure_replay::RasterizeSourceDisplayItemListWithScale(list, width, height, scale);
    let differences = owned
        .chunks_exact(4)
        .zip(native.chunks_exact(4))
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .collect::<Vec<_>>();
    let mut failures = Vec::new();
    if !differences.is_empty() {
        failures.push(format!(
            "owned mismatch={} first={:?}",
            differences.len(),
            differences.first()
        ));
    }
    for format in [
        skia::PixelFormat::Rgba8888,
        skia::PixelFormat::Bgra8888,
        skia::PixelFormat::Bgrx8888,
    ] {
        let guard = 0x9da7b3c5;
        let n = (width * height) as usize;
        let mut storage = vec![guard; n + 32];
        let target = &mut storage[16..16 + n];
        raster::surface::RenderDisplayItemListIntoWindowBufferWithFormat(
            list, width, height, scale, target, format,
        )
        .unwrap();
        let mut count = 0;
        let mut first = None;
        for (i, (&pixel, expected)) in target.iter().zip(native.chunks_exact(4)).enumerate() {
            let b = pixel.to_le_bytes();
            let actual = if format == skia::PixelFormat::Rgba8888 {
                b
            } else {
                [
                    b[2],
                    b[1],
                    b[0],
                    if format == skia::PixelFormat::Bgrx8888 {
                        255
                    } else {
                        b[3]
                    },
                ]
            };
            if actual != expected {
                count += 1;
                if first.is_none() {
                    first = Some((i, actual, expected.to_vec()));
                }
            }
            if format == skia::PixelFormat::Bgrx8888 {
                assert_eq!(b[3], 0);
            }
        }
        assert!(
            storage[..16]
                .iter()
                .chain(storage[16 + n..].iter())
                .all(|&v| v == guard),
            "{label} {format:?} out-of-buffer write"
        );
        if count != 0 {
            failures.push(format!("{format:?} mismatch={count} first={first:?}"));
        }
    }
    if !failures.is_empty() {
        let detail = format!("{label}: {}", failures.join("; "));
        eprintln!("{detail}");
        all_failures.push(detail);
    }
}
#[test]
fn large_absolute_quad_conic_cubic_extrema_match_original_whole_device() {
    let mut failures = Vec::new();
    for kind in 0..3 {
        for (x, y) in [(529.5, 135.0), (1180.25, 330.125)] {
            for translated in [false, true] {
                let (path, tr) = if translated {
                    (lens(kind, 0.0, 0.0), Some((x, y)))
                } else {
                    (lens(kind, x, y), None)
                };
                compare(
                    &scene(path, tr, None, false, false),
                    &format!("kind={kind} origin={x},{y} translate={translated}"),
                    &mut failures,
                );
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn curve_extrema_cross_virtual_256_tiles_and_halo_without_chopping() {
    let mut failures = Vec::new();
    for kind in 0..3 {
        for (x, y) in [(126.5, 126.5), (253.5, 253.5), (380.5, 380.5)] {
            compare(
                &scene(lens(kind, x, y), None, None, false, false),
                &format!("kind={kind} halo={x},{y}"),
                &mut failures,
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn actual_geometry_canvas_and_aa_rect_clips_keep_original_rounding() {
    let mut failures = Vec::new();
    for kind in 0..3 {
        for x in [-10.0, 1260.0] {
            compare(
                &scene(lens(kind, x, 9.125), None, None, false, false),
                &format!("kind={kind} canvas-side={x}"),
                &mut failures,
            );
        }
        for aa in [false, true] {
            let clip = PaintRect {
                x: 537.5,
                y: 137.0,
                width: 45.0,
                height: 36.0,
            };
            compare(
                &scene(
                    lens(kind, 529.5, 135.0),
                    None,
                    Some((clip, aa)),
                    false,
                    false,
                ),
                &format!("kind={kind} actual-two-sided-clip aa={aa}"),
                &mut failures,
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn f16_layers_keep_global_curve_coefficients_across_tile_edges() {
    let mut failures = Vec::new();
    for kind in 0..3 {
        for (x, y) in [(529.5, 135.0), (126.5, 126.5)] {
            compare(
                &scene(lens(kind, x, y), None, None, true, false),
                &format!("kind={kind} f16={x},{y}"),
                &mut failures,
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn winding_even_odd_rings_and_crossing_cubics_match_original() {
    let mut failures = Vec::new();
    for (x, y) in [(529.5, 135.0), (126.5, 126.5)] {
        for even_odd in [false, true] {
            let mut path = circle(x, y, 65.25, false);
            path.extend(circle(x + 10.125, y + 10.125, 45.0, !even_odd));
            compare(
                &scene(path, None, None, false, even_odd),
                &format!("ring={x},{y} even_odd={even_odd}"),
                &mut failures,
            );
        }
        compare(
            &scene(crossing(x, y), None, None, false, false),
            &format!("crossing={x},{y}"),
            &mut failures,
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
