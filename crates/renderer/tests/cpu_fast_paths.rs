//! Device-space spans and scaled analytic rasterization against native Skia.
#![cfg(all(feature = "pure_replay", feature = "source_replay"))]

use layoutng_assembly::internal::layout_input::{Offset, TransformMatrix};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    PaintColorStop, PaintCornerRadii, PaintCornerRadius, PaintShader,
};
use paint::paint_engine::{DisplayItem, DisplayItemType as T, PaintArtifact, PaintRect};

fn check(items: Vec<DisplayItem>, scale: f64, context: &str) {
    let list = PaintArtifact {
        items,
        ..Default::default()
    };
    let mut reference = list.clone();
    let mut transform = TransformMatrix::default();
    transform.values[0] = scale;
    transform.values[5] = scale;
    std::sync::Arc::make_mut(&mut reference.items).insert(
        0,
        DisplayItem {
            r#type: T::kConcat,
            transform,
            ..Default::default()
        },
    );
    let native = raster::source_replay::RasterizeSourceDisplayItemList(&reference, 600, 400);
    let rust = raster::pure_replay::RasterizeSourceDisplayItemListWithScale(&list, 600, 400, scale);
    let mut count = 0;
    let mut first = None;
    for (i, (a, b)) in native.chunks_exact(4).zip(rust.chunks_exact(4)).enumerate() {
        if a != b {
            count += 1;
            first.get_or_insert((i % 600, i / 600, a, b));
        }
    }
    assert_eq!(count, 0, "{context}, scale={scale}, first={first:?}");
}

fn background() -> DisplayItem {
    DisplayItem {
        r#type: T::kDrawRect,
        rect: PaintRect {
            x: 0.0,
            y: 0.0,
            width: 600.0,
            height: 400.0,
        },
        color: Color {
            red: 0.3,
            green: 0.5,
            blue: 0.7,
            alpha: 1.0,
        },
        ..Default::default()
    }
}

fn radii() -> PaintCornerRadii {
    let r = PaintCornerRadius { x: 12.0, y: 9.0 };
    PaintCornerRadii {
        top_left: r,
        top_right: r,
        bottom_left: r,
        bottom_right: r,
    }
}

#[test]
fn scaled_solid_spans_preserve_translucency_and_clip_edges() {
    for scale in [1.0, 2.0] {
        for rounded in [false, true] {
            for alpha in [1.0, 0.3] {
                check(
                    vec![
                        background(),
                        DisplayItem {
                            r#type: if rounded {
                                T::kClipRoundedRect
                            } else {
                                T::kClipRect
                            },
                            rect: PaintRect {
                                x: 8.0,
                                y: 9.0,
                                width: 280.0,
                                height: 165.0,
                            },
                            corner_radii: radii(),
                            antialias: true,
                            ..Default::default()
                        },
                        DisplayItem {
                            r#type: T::kDrawRect,
                            rect: PaintRect {
                                x: 0.0,
                                y: 0.0,
                                width: 300.0,
                                height: 200.0,
                            },
                            color: Color {
                                red: 77.0 / 255.0,
                                green: 128.0 / 255.0,
                                blue: 199.0 / 255.0,
                                alpha,
                            },
                            ..Default::default()
                        },
                    ],
                    scale,
                    &format!("solid rounded_clip={rounded} alpha={alpha}"),
                );
            }
        }
    }
}

#[test]
fn scaled_rounded_paths_match_skia_across_tile_boundaries() {
    for scale in [1.5, 2.0] {
        for shift in [0.0, 0.25] {
            for alpha in [1.0, 0.3] {
                check(
                    vec![
                        background(),
                        DisplayItem {
                            r#type: T::kDrawRoundedRect,
                            rect: PaintRect {
                                x: 8.0 + shift,
                                y: 9.0 + shift,
                                width: 280.0,
                                height: 165.0,
                            },
                            corner_radii: radii(),
                            antialias: true,
                            color: Color {
                                red: 1.0,
                                green: 69.0 / 255.0,
                                blue: 91.0 / 255.0,
                                alpha,
                            },
                            ..Default::default()
                        },
                    ],
                    scale,
                    &format!("round shift={shift} alpha={alpha}"),
                );
            }
        }
    }
}

#[test]
fn scaled_gradient_pipeline_matches_skia() {
    let rect = PaintRect {
        x: 8.0,
        y: 9.0,
        width: 280.0,
        height: 165.0,
    };
    for scale in [1.0, 2.0] {
        for (dx, dy) in [(280.0, 0.0), (0.0, 165.0), (280.0, 165.0)] {
            for alpha in [1.0, 0.3] {
                let shader = PaintShader {
                    start: Offset {
                        x: rect.x,
                        y: rect.y,
                    },
                    end: Offset {
                        x: rect.x + dx,
                        y: rect.y + dy,
                    },
                    stops: vec![
                        PaintColorStop {
                            offset: 0.0,
                            color: Color {
                                red: 51.0 / 255.0,
                                green: 119.0 / 255.0,
                                blue: 1.0,
                                alpha,
                            },
                            ..Default::default()
                        },
                        PaintColorStop {
                            offset: 1.0,
                            color: Color {
                                red: 168.0 / 255.0,
                                green: 82.0 / 255.0,
                                blue: 1.0,
                                alpha,
                            },
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                };
                check(
                    vec![
                        background(),
                        DisplayItem {
                            r#type: T::kDrawGradientRect,
                            rect,
                            tile_rect: rect,
                            paint_shader: Some(shader),
                            ..Default::default()
                        },
                    ],
                    scale,
                    &format!("gradient vector={dx},{dy} alpha={alpha}"),
                );
            }
        }
    }
}

#[test]
fn raster_clip_bounds_follow_intersections_layers_and_restore() {
    for scale in [1.0, 2.0] {
        for kind in [0, 1, 2] {
            let clip = DisplayItem {
                r#type: if kind == 1 {
                    T::kClipRoundedRect
                } else {
                    T::kClipRect
                },
                rect: PaintRect {
                    x: if kind == 2 { 1000.0 } else { 8.125 },
                    y: 9.375,
                    width: 180.0,
                    height: 140.0,
                },
                corner_radii: radii(),
                antialias: kind == 1,
                ..Default::default()
            };
            let colored = |x, y, width, height| DisplayItem {
                r#type: T::kDrawRoundedRect,
                rect: PaintRect {
                    x,
                    y,
                    width,
                    height,
                },
                corner_radii: radii(),
                antialias: true,
                color: Color {
                    red: 77.0 / 255.0,
                    green: 128.0 / 255.0,
                    blue: 199.0 / 255.0,
                    alpha: 0.5,
                },
                ..Default::default()
            };
            check(
                vec![
                    background(),
                    DisplayItem {
                        r#type: T::kSave,
                        ..Default::default()
                    },
                    clip,
                    colored(4.125, 5.25, 220.0, 155.0),
                    DisplayItem {
                        r#type: T::kSaveLayerAlpha,
                        opacity: 0.625,
                        ..Default::default()
                    },
                    DisplayItem {
                        r#type: T::kClipRect,
                        rect: PaintRect {
                            x: 24.0,
                            y: 25.0,
                            width: 112.0,
                            height: 72.0,
                        },
                        ..Default::default()
                    },
                    colored(12.5, 17.125, 144.0, 90.0),
                    DisplayItem {
                        r#type: T::kRestore,
                        ..Default::default()
                    },
                    colored(148.0, 113.0, 76.0, 48.0),
                    DisplayItem {
                        r#type: T::kRestore,
                        ..Default::default()
                    },
                    colored(240.25, 15.125, 40.0, 60.0),
                ],
                scale,
                &format!("raster clip lifecycle kind={kind}"),
            );
        }
    }
}
