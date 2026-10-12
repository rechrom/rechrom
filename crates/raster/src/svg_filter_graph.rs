//! Shared translation from document SVG filter graphs to Skia CPU effect surfaces.

use layoutng_assembly::internal::paint_input::{
    SvgBlendMode, SvgColorMatrixType, SvgCompositeOperator, SvgFilterGraph, SvgFilterInput,
    SvgFilterLength, SvgFilterPrimitiveKind, SvgFilterRegion, SvgFilterUnits, SvgLightSource,
    SvgLightingType,
};
use paint::paint_engine::PaintRect;
use skia::cpu::layer_filters::{
    ColorMatrixKind, RenderSvgBlend, RenderSvgComposite, RenderSvgEdgeMode, RenderSvgFilterGraph,
    RenderSvgFilterKind, RenderSvgFilterPrimitive, RenderSvgLight, RenderSvgLightingType,
    RenderSvgTransfer,
};
use std::io;
use std::sync::Arc;

fn svg_input(input: &SvgFilterInput) -> String {
    match input {
        SvgFilterInput::kSourceGraphic => "SourceGraphic".into(),
        SvgFilterInput::kSourceAlpha => "SourceAlpha".into(),
        SvgFilterInput::kBackgroundImage => "BackgroundImage".into(),
        SvgFilterInput::kBackgroundAlpha => "BackgroundAlpha".into(),
        SvgFilterInput::kFillPaint => "FillPaint".into(),
        SvgFilterInput::kStrokePaint => "StrokePaint".into(),
        SvgFilterInput::kNamed(value) => value.clone(),
    }
}
pub fn TranslateSvgFilterGraph(
    graph: &SvgFilterGraph,
    reference: PaintRect,
    scale: f64,
) -> io::Result<RenderSvgFilterGraph> {
    TranslateSvgFilterGraphWithTransform(graph, reference, scale, scale, 0.0, 0.0, 0.0, 0.0, 1.0)
}

/// Translate a graph from its owning effect space into an axis-aligned device
/// render target.  Filter and primitive regions are positions, rather than
/// mere extents: retaining the transformed reference-box origin is required
/// when the isolated surface is not rooted at (0, 0).
pub fn TranslateSvgFilterGraphWithTransform(
    graph: &SvgFilterGraph,
    reference: PaintRect,
    scale_x: f64,
    scale_y: f64,
    translate_x: f64,
    translate_y: f64,
    user_origin_x: f64,
    user_origin_y: f64,
    user_scale: f64,
) -> io::Result<RenderSvgFilterGraph> {
    if let Some(error) = &graph.unsupported {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("svg-filter: {error}"),
        ));
    }
    let (rw, rh) = (
        (reference.width * scale_x).max(0.0),
        (reference.height * scale_y).max(0.0),
    );
    let (rx, ry) = (
        reference.x * scale_x + translate_x,
        reference.y * scale_y + translate_y,
    );
    let (user_scale_x, user_scale_y) = (scale_x * user_scale, scale_y * user_scale);
    let (user_tx, user_ty) = (
        user_origin_x * scale_x + translate_x,
        user_origin_y * scale_y + translate_y,
    );
    let resolve_position = |value: Option<SvgFilterLength>,
                            reference_origin: f64,
                            axis: f64,
                            user_scale: f64,
                            user_translation: f64,
                            units: SvgFilterUnits,
                            default: f64|
     -> f32 {
        value.map_or(default, |v| {
            if v.percentage {
                reference_origin + axis * v.value / 100.0
            } else if units == SvgFilterUnits::kObjectBoundingBox {
                reference_origin + axis * v.value
            } else {
                v.value * user_scale + user_translation
            }
        }) as f32
    };
    let resolve_length = |value: Option<SvgFilterLength>,
                          axis: f64,
                          user_scale: f64,
                          units: SvgFilterUnits,
                          default: f64|
     -> f32 {
        value.map_or(default, |v| {
            if v.percentage {
                axis * v.value / 100.0
            } else if units == SvgFilterUnits::kObjectBoundingBox {
                axis * v.value
            } else {
                v.value * user_scale
            }
        }) as f32
    };
    let resolve_region =
        |region: &SvgFilterRegion, units: SvgFilterUnits, defaults: [f64; 4]| -> [f32; 4] {
            [
                resolve_position(region.x, rx, rw, user_scale_x, user_tx, units, defaults[0]),
                resolve_position(region.y, ry, rh, user_scale_y, user_ty, units, defaults[1]),
                resolve_length(region.width, rw, user_scale_x, units, defaults[2]),
                resolve_length(region.height, rh, user_scale_y, units, defaults[3]),
            ]
        };
    let filter_region = resolve_region(
        &graph.region,
        graph.filter_units,
        [rx - rw * 0.1, ry - rh * 0.1, rw * 1.2, rh * 1.2],
    );
    let mut primitives = Vec::with_capacity(graph.primitives.len());
    for primitive in &graph.primitives {
        let region = resolve_region(&primitive.region, graph.primitive_units, [rx, ry, rw, rh]);
        let kind = match &primitive.kind {
            SvgFilterPrimitiveKind::kGaussianBlur {
                sigma_x,
                sigma_y,
                edge_mode,
            } => RenderSvgFilterKind::GaussianBlur {
                x: (sigma_x
                    * if graph.primitive_units == SvgFilterUnits::kObjectBoundingBox {
                        rw
                    } else {
                        user_scale_x
                    }) as f32,
                y: (sigma_y
                    * if graph.primitive_units == SvgFilterUnits::kObjectBoundingBox {
                        rh
                    } else {
                        user_scale_y
                    }) as f32,
                edge_mode: match edge_mode {
                    layoutng_assembly::internal::paint_input::SvgFilterEdgeMode::kNone => {
                        RenderSvgEdgeMode::None
                    }
                    layoutng_assembly::internal::paint_input::SvgFilterEdgeMode::kDuplicate => {
                        RenderSvgEdgeMode::Duplicate
                    }
                    layoutng_assembly::internal::paint_input::SvgFilterEdgeMode::kWrap => {
                        RenderSvgEdgeMode::Wrap
                    }
                },
            },
            SvgFilterPrimitiveKind::kOffset { dx, dy } => RenderSvgFilterKind::Offset {
                x: (dx
                    * if graph.primitive_units == SvgFilterUnits::kObjectBoundingBox {
                        rw
                    } else {
                        user_scale_x
                    }) as f32,
                y: (dy
                    * if graph.primitive_units == SvgFilterUnits::kObjectBoundingBox {
                        rh
                    } else {
                        user_scale_y
                    }) as f32,
            },
            SvgFilterPrimitiveKind::kFlood { color } => {
                RenderSvgFilterKind::Flood([color.red, color.green, color.blue, color.alpha])
            }
            SvgFilterPrimitiveKind::kColorMatrix { kind, values } => {
                let matrix = match kind {
                    SvgColorMatrixType::kMatrix => {
                        let mut m = [0.0f32; 20];
                        if values.len() == 20 {
                            for (i, v) in values.iter().enumerate() {
                                m[i] = *v as f32
                            }
                        } else {
                            m[0] = 1.0;
                            m[6] = 1.0;
                            m[12] = 1.0;
                            m[18] = 1.0
                        }
                        m
                    }
                    SvgColorMatrixType::kSaturate => skia::cpu::layer_filters::css_color_matrix(
                        ColorMatrixKind::Saturate,
                        values.first().copied().unwrap_or(1.0) as f32,
                    ),
                    SvgColorMatrixType::kHueRotate => skia::cpu::layer_filters::css_color_matrix(
                        ColorMatrixKind::HueRotate,
                        values.first().copied().unwrap_or(0.0) as f32,
                    ),
                    SvgColorMatrixType::kLuminanceToAlpha => {
                        let mut m = [0.0; 20];
                        m[15] = 0.2125;
                        m[16] = 0.7154;
                        m[17] = 0.0721;
                        m
                    }
                };
                RenderSvgFilterKind::ColorMatrix(matrix)
            }
            SvgFilterPrimitiveKind::kBlend { mode, input2 } => RenderSvgFilterKind::Blend {
                input2: svg_input(input2),
                mode: match mode {
                    SvgBlendMode::kNormal => RenderSvgBlend::Normal,
                    SvgBlendMode::kMultiply => RenderSvgBlend::Multiply,
                    SvgBlendMode::kScreen => RenderSvgBlend::Screen,
                    SvgBlendMode::kDarken => RenderSvgBlend::Darken,
                    SvgBlendMode::kLighten => RenderSvgBlend::Lighten,
                },
            },
            SvgFilterPrimitiveKind::kComposite {
                operator,
                input2,
                k,
            } => RenderSvgFilterKind::Composite {
                input2: svg_input(input2),
                operator: match operator {
                    SvgCompositeOperator::kOver => RenderSvgComposite::Over,
                    SvgCompositeOperator::kIn => RenderSvgComposite::In,
                    SvgCompositeOperator::kOut => RenderSvgComposite::Out,
                    SvgCompositeOperator::kAtop => RenderSvgComposite::Atop,
                    SvgCompositeOperator::kXor => RenderSvgComposite::Xor,
                    SvgCompositeOperator::kArithmetic => RenderSvgComposite::Arithmetic,
                },
                k: k.map(|v| v as f32),
            },
            SvgFilterPrimitiveKind::kMerge { inputs } => {
                RenderSvgFilterKind::Merge(inputs.iter().map(svg_input).collect())
            }
            SvgFilterPrimitiveKind::kComponentTransfer { functions } => {
                RenderSvgFilterKind::ComponentTransfer(functions.clone().map(|f| match f {
                    layoutng_assembly::internal::paint_input::SvgTransferFunction::kIdentity => {
                        RenderSvgTransfer::Identity
                    }
                    layoutng_assembly::internal::paint_input::SvgTransferFunction::kTable(v) => {
                        RenderSvgTransfer::Table(v.into_iter().map(|x| x as f32).collect())
                    }
                    layoutng_assembly::internal::paint_input::SvgTransferFunction::kDiscrete(v) => {
                        RenderSvgTransfer::Discrete(v.into_iter().map(|x| x as f32).collect())
                    }
                    layoutng_assembly::internal::paint_input::SvgTransferFunction::kLinear {
                        slope,
                        intercept,
                    } => RenderSvgTransfer::Linear {
                        slope: slope as f32,
                        intercept: intercept as f32,
                    },
                    layoutng_assembly::internal::paint_input::SvgTransferFunction::kGamma {
                        amplitude,
                        exponent,
                        offset,
                    } => RenderSvgTransfer::Gamma {
                        amplitude: amplitude as f32,
                        exponent: exponent as f32,
                        offset: offset as f32,
                    },
                }))
            }
            SvgFilterPrimitiveKind::kMorphology {
                operator,
                radius_x,
                radius_y,
            } => RenderSvgFilterKind::Morphology {
                dilate: matches!(
                    operator,
                    layoutng_assembly::internal::paint_input::SvgMorphologyOperator::kDilate
                ),
                x: (radius_x
                    * if graph.primitive_units == SvgFilterUnits::kObjectBoundingBox {
                        rw
                    } else {
                        user_scale_x
                    })
                .round()
                .max(0.0) as u32,
                y: (radius_y
                    * if graph.primitive_units == SvgFilterUnits::kObjectBoundingBox {
                        rh
                    } else {
                        user_scale_y
                    })
                .round()
                .max(0.0) as u32,
            },
            SvgFilterPrimitiveKind::kDisplacementMap {
                input2,
                scale: amount,
                x_channel,
                y_channel,
            } => RenderSvgFilterKind::DisplacementMap {
                input2: svg_input(input2),
                scale: (amount * user_scale_x.max(user_scale_y)) as f32,
                x_channel: *x_channel as usize,
                y_channel: *y_channel as usize,
            },
            SvgFilterPrimitiveKind::kTurbulence {
                base_x,
                base_y,
                octaves,
                seed,
                stitch,
                kind,
            } => RenderSvgFilterKind::Turbulence {
                base_x: *base_x as f32,
                base_y: *base_y as f32,
                octaves: *octaves,
                seed: *seed as f32,
                stitch: *stitch,
                fractal_noise: matches!(
                    kind,
                    layoutng_assembly::internal::paint_input::SvgTurbulenceType::kFractalNoise
                ),
            },
            SvgFilterPrimitiveKind::kTile => RenderSvgFilterKind::Tile,
            SvgFilterPrimitiveKind::kImage {
                href,
                local_fragment,
                preserve_none,
                width,
                height,
                pixels,
                ..
            } => {
                if !*local_fragment && pixels.is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "feImage bitmap resource is not present in effect payload",
                    ));
                }
                RenderSvgFilterKind::Image {
                    href: href.clone(),
                    local_fragment: *local_fragment,
                    preserve_none: *preserve_none,
                    width: *width,
                    height: *height,
                    pixels: Arc::clone(pixels),
                }
            }
            SvgFilterPrimitiveKind::kConvolveMatrix {
                order_x,
                order_y,
                kernel,
                divisor,
                bias,
                target_x,
                target_y,
                edge_mode,
                kernel_unit_x,
                kernel_unit_y,
                preserve_alpha,
            } => RenderSvgFilterKind::Convolve {
                order_x: *order_x,
                order_y: *order_y,
                kernel: kernel.iter().map(|v| *v as f32).collect(),
                divisor: *divisor as f32,
                bias: *bias as f32,
                target_x: *target_x,
                target_y: *target_y,
                edge_mode: match edge_mode {
                    layoutng_assembly::internal::paint_input::SvgFilterEdgeMode::kNone => {
                        RenderSvgEdgeMode::None
                    }
                    layoutng_assembly::internal::paint_input::SvgFilterEdgeMode::kDuplicate => {
                        RenderSvgEdgeMode::Duplicate
                    }
                    layoutng_assembly::internal::paint_input::SvgFilterEdgeMode::kWrap => {
                        RenderSvgEdgeMode::Wrap
                    }
                },
                unit_x: (kernel_unit_x
                    * if graph.primitive_units == SvgFilterUnits::kObjectBoundingBox {
                        rw
                    } else {
                        user_scale_x
                    }) as f32,
                unit_y: (kernel_unit_y
                    * if graph.primitive_units == SvgFilterUnits::kObjectBoundingBox {
                        rh
                    } else {
                        user_scale_y
                    }) as f32,
                preserve_alpha: *preserve_alpha,
            },
            SvgFilterPrimitiveKind::kLighting {
                lighting_type,
                light,
                color,
                surface_scale,
                constant,
                specular_exponent,
                ..
            } => {
                let object_units = graph.primitive_units == SvgFilterUnits::kObjectBoundingBox;
                let (sx, sy, sz) = if object_units {
                    (rw, rh, (rw.mul_add(rw, rh * rh) * 0.5).sqrt())
                } else {
                    (
                        user_scale_x,
                        user_scale_y,
                        user_scale_x.max(user_scale_y),
                    )
                };
                let point =
                    |x: f64, y: f64, z: f64| [(x * sx) as f32, (y * sy) as f32, (z * sz) as f32];
                let light = light.as_ref().map(|light| match light {
                    SvgLightSource::kDistant { azimuth, elevation } => {
                        let azimuth = azimuth.to_radians();
                        let elevation = elevation.to_radians();
                        RenderSvgLight::Distant {
                            direction: [
                                (azimuth.cos() * elevation.cos()) as f32,
                                (azimuth.sin() * elevation.cos()) as f32,
                                elevation.sin() as f32,
                            ],
                        }
                    }
                    SvgLightSource::kPoint { x, y, z } => RenderSvgLight::Point {
                        position: point(*x, *y, *z),
                    },
                    SvgLightSource::kSpot {
                        x,
                        y,
                        z,
                        points_at_x,
                        points_at_y,
                        points_at_z,
                        specular_exponent,
                        limiting_cone_angle,
                    } => RenderSvgLight::Spot {
                        position: point(*x, *y, *z),
                        target: point(*points_at_x, *points_at_y, *points_at_z),
                        falloff_exponent: *specular_exponent as f32,
                        limiting_cone_angle: *limiting_cone_angle as f32,
                    },
                });
                RenderSvgFilterKind::Lighting {
                    lighting_type: match lighting_type {
                        SvgLightingType::kDiffuse => RenderSvgLightingType::Diffuse,
                        SvgLightingType::kSpecular => RenderSvgLightingType::Specular,
                    },
                    light,
                    color: [color.red, color.green, color.blue],
                    surface_scale: (surface_scale * user_scale_x.max(user_scale_y)) as f32,
                    constant: *constant as f32,
                    shininess: *specular_exponent as f32,
                }
            }
        };
        primitives.push(RenderSvgFilterPrimitive {
            input: svg_input(&primitive.input),
            result: primitive.result.clone(),
            region,
            linear_rgb: primitive.linear_rgb,
            kind,
        });
    }
    let color = |value: Option<layoutng_assembly::internal::layout_input_types::Color>| {
        value.map(|c| [c.red, c.green, c.blue, c.alpha])
    };
    Ok(RenderSvgFilterGraph {
        primitives,
        region: filter_region,
        fill_paint: color(graph.fill_paint),
        stroke_paint: color(graph.stroke_paint),
    })
}
