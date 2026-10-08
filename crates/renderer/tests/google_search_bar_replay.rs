#![cfg(all(
    feature = "pure_replay",
    feature = "source_replay",
    feature = "profiling"
))]
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};

// Exact frozen Google commands 258..261, independently separable.
fn radii(r: f64) -> PaintCornerRadii {
    let r = PaintCornerRadius { x: r, y: r };
    PaintCornerRadii {
        top_left: r,
        top_right: r,
        bottom_left: r,
        bottom_right: r,
    }
}
fn background() -> DisplayItem {
    DisplayItem {
        r#type: T::kDrawRect,
        rect: PaintRect {
            x: 0.,
            y: 0.,
            width: 1280.,
            height: 720.,
        },
        color: Color {
            red: 1.,
            green: 1.,
            blue: 1.,
            alpha: 1.,
        },
        ..Default::default()
    }
}
fn command(index: usize) -> DisplayItem {
    match index {
        258 => DisplayItem {
            r#type: T::kDrawBoxShadow,
            rect: PaintRect {
                x: 296.0,
                y: 246.0,
                width: 688.0,
                height: 52.0,
            },
            inner_rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            corner_radii: radii(26.0),
            inner_corner_radii: radii(0.0),
            color: Color {
                red: 0.12156862765550613,
                green: 0.12156862765550613,
                blue: 0.12156862765550613,
                alpha: 0.0784313753247261,
            },
            antialias: true,
            shadow_offset: layoutng_assembly::internal::layout_input::Offset { x: 0.0, y: 3.0 },
            blur_radius: 10.0,
            shadow_has_opaque_background: true,
            ..Default::default()
        },
        259 => DisplayItem {
            r#type: T::kDrawRoundedRect,
            rect: PaintRect {
                x: 296.5,
                y: 246.5,
                width: 687.0,
                height: 51.0,
            },
            inner_rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            corner_radii: radii(25.5),
            inner_corner_radii: radii(0.0),
            color: Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            },
            antialias: true,
            shadow_offset: layoutng_assembly::internal::layout_input::Offset { x: 0.0, y: 0.0 },
            blur_radius: 0.0,
            shadow_has_opaque_background: false,
            ..Default::default()
        },
        260 => DisplayItem {
            r#type: T::kDrawDoubleRoundedRect,
            line_style: layoutng_assembly::internal::layout_input::BorderLineStyle::kSolid,
            rect: PaintRect {
                x: 296.0,
                y: 246.0,
                width: 688.0,
                height: 52.0,
            },
            inner_rect: PaintRect {
                x: 297.0,
                y: 247.0,
                width: 686.0,
                height: 50.0,
            },
            corner_radii: radii(26.0),
            inner_corner_radii: radii(25.0),
            color: Color {
                red: 0.8549019694328308,
                green: 0.8627451062202454,
                blue: 0.8784313797950745,
                alpha: 1.0,
            },
            antialias: true,
            shadow_offset: layoutng_assembly::internal::layout_input::Offset { x: 0.0, y: 0.0 },
            blur_radius: 0.0,
            shadow_has_opaque_background: false,
            ..Default::default()
        },
        261 => DisplayItem {
            r#type: T::kDrawRoundedRect,
            rect: PaintRect {
                x: 297.0,
                y: 247.0,
                width: 686.0,
                height: 50.0,
            },
            inner_rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            corner_radii: radii(25.0),
            inner_corner_radii: radii(0.0),
            color: Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            },
            antialias: true,
            shadow_offset: layoutng_assembly::internal::layout_input::Offset { x: 0.0, y: 0.0 },
            blur_radius: 0.0,
            shadow_has_opaque_background: false,
            ..Default::default()
        },
        _ => panic!("unsupported frozen command"),
    }
}
fn scene(indices: &[usize], layer: bool) -> PaintArtifact {
    let mut items = vec![background()];
    if layer {
        items.push(DisplayItem {
            r#type: T::kSaveLayer,
            ..Default::default()
        });
    }
    for &i in indices {
        items.push(command(i));
    }
    if layer {
        items.push(DisplayItem {
            r#type: T::kRestore,
            ..Default::default()
        });
    }
    PaintArtifact {
        items,
        ..Default::default()
    }
}
fn compare(list: &PaintArtifact, width: u32, height: u32, scale: f64) {
    let (native, _) =
        raster::source_replay::ProfileSourceDisplayItemListWithScale(list, width, height, scale);
    let owned =
        raster::pure_replay::RasterizeSourceDisplayItemListWithScale(list, width, height, scale);
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
        raster::surface::RenderDisplayItemListIntoWindowBufferWithFormat(
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
        list.items.last().unwrap().rect,
        failures.join("; ")
    );
}
#[test]
fn frozen_google_bar_shadow_matches_whole_native() {
    compare(&scene(&[258], false), 2560, 1440, 2.0);
}
#[test]
fn frozen_google_bar_white_under_border_matches_whole_native() {
    compare(&scene(&[259], false), 2560, 1440, 2.0);
}
#[test]
fn frozen_google_bar_colored_border_matches_whole_native() {
    compare(&scene(&[260], false), 2560, 1440, 2.0);
}
#[test]
fn frozen_google_bar_white_inner_matches_whole_native() {
    compare(&scene(&[261], false), 2560, 1440, 2.0);
}
#[test]
fn frozen_google_bar_border_with_inner_matches_whole_native() {
    compare(&scene(&[260, 261], false), 2560, 1440, 2.0);
}
#[test]
fn frozen_google_bar_all_four_matches_whole_native() {
    compare(&scene(&[258, 259, 260, 261], false), 2560, 1440, 2.0);
}
#[test]
fn frozen_google_bar_full_f16_matches_whole_native() {
    compare(&scene(&[258, 259, 260, 261], true), 2560, 1440, 2.0);
}
