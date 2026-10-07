// Native differential tests remain at the paint/Skia boundary.
use crate::convert::ToSkia;
use layoutng_assembly::internal::paint_input::PaintCornerRadii;
use paint::paint_engine::PaintRect;
use skia::raster::{Mask, Transform};
fn rounded_rect_mask(
    rect: PaintRect,
    radii: PaintCornerRadii,
    transform: Transform,
    width: u32,
    height: u32,
    convex: bool,
    clip: Option<&Mask>,
) -> Option<skia::cpu::analytic_aa::RoundedCoverage> {
    skia::cpu::analytic_aa::rounded_rect_mask(
        rect.to_skia(),
        radii.to_skia(),
        transform,
        width,
        height,
        convex,
        clip,
    )
}
#[cfg(all(test, feature = "source_replay"))]
mod tests {
    use super::*;
    use layoutng_assembly::internal::layout_input_types::Color;
    use layoutng_assembly::internal::paint_input::PaintCornerRadius;
    use paint::paint_engine::{DisplayItem, DisplayItemType, PaintArtifact};
    #[test]
    fn unsupported_transform_is_rejected_before_mask_allocation() {
        let bounds = PaintRect {
            x: 8.0,
            y: 9.0,
            width: 100.0,
            height: 50.0,
        };
        for transform in [
            Transform::from_row(1.0, 0.25, 0.0, 1.0, 0.0, 0.0),
            Transform::from_scale(-1.0, 1.0),
            Transform::from_scale(f32::NAN, 1.0),
        ] {
            // An allocation attempt at these dimensions would fail. Reject
            // the matrix before touching dimensions or allocating coverage.
            assert!(rounded_rect_mask(
                bounds,
                Default::default(),
                transform,
                u32::MAX,
                u32::MAX,
                false,
                None
            )
            .is_none());
        }
    }
    #[test]
    fn rounded_rect_coverage_matches_native_skia() {
        let mut failures = 0;
        let mut first = None;
        for (w, h) in [(16.0, 16.0), (52.0, 38.0), (108.0, 44.0)] {
            for radius in [2.0, 4.0, 10.0, 19.0] {
                for shift in [0.0, 0.25, 0.5, 0.75] {
                    let rect = PaintRect {
                        x: 8.0 + shift,
                        y: 9.0 + shift,
                        width: w,
                        height: h,
                    };
                    let r = PaintCornerRadius {
                        x: radius,
                        y: radius,
                    };
                    let radii = PaintCornerRadii {
                        top_left: r,
                        top_right: r,
                        bottom_left: r,
                        bottom_right: r,
                    };
                    let list = PaintArtifact {
                        items: vec![DisplayItem {
                            r#type: DisplayItemType::kDrawRoundedRect,
                            rect,
                            corner_radii: radii,
                            color: Color {
                                red: 0.0,
                                green: 0.0,
                                blue: 0.0,
                                alpha: 1.0,
                            },
                            antialias: true,
                            ..Default::default()
                        }]
                        .into(),
                        ..Default::default()
                    };
                    let source =
                        crate::source_replay::RasterizeSourceDisplayItemList(&list, 128, 64);
                    let mask =
                        rounded_rect_mask(rect, radii, Transform::identity(), 128, 64, false, None)
                            .unwrap();
                    let differences: Vec<_> = source
                        .chunks_exact(4)
                        .zip(mask.mask.data())
                        .enumerate()
                        .filter(|(_, (p, a))| 255 - p[0] != **a)
                        .collect();
                    if !differences.is_empty() {
                        failures += 1;

                        first.get_or_insert_with(|| {
                            format!(
                                "{w}x{h} radius={radius} shift={shift}: {} pixels, first {:?}",
                                differences.len(),
                                differences[0]
                            )
                        });
                    }
                }
            }
        }
        assert_eq!(failures, 0, "{}", first.unwrap_or_default());
    }
}
