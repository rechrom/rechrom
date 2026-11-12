//! Shared translation from document SVG filter graphs to Skia effect surfaces.

use layoutng_assembly::internal::paint_input::{
    PaintCanvasFilter, PaintComponentTransferType, PaintConvolveEdgeMode, PaintTurbulenceType,
    SvgBlendMode, SvgColorMatrixType, SvgCompositeOperator, SvgFilterGraph, SvgFilterInput,
    SvgFilterLength, SvgFilterPrimitive, SvgFilterPrimitiveKind, SvgFilterRegion, SvgFilterUnits,
    SvgLightSource, SvgLightingType, SvgMeetOrSlice, SvgPreserveAspectRatioAlign,
};
use paint::paint_engine::PaintRect;
use paint::svg_filter_geometry::{resolve_svg_filter_geometry, SvgFilterGeometryError};
use raster_commands::filter_graph::{
    RenderSvgBlend, RenderSvgComposite, RenderSvgEdgeMode, RenderSvgFilterGraph,
    RenderSvgFilterKind, RenderSvgFilterPrimitive, RenderSvgLight, RenderSvgLightingType,
    RenderSvgMeetOrSlice, RenderSvgPreserveAspectRatioAlign, RenderSvgTransfer,
};
use std::io;
use std::sync::Arc;

pub(crate) fn translate_canvas_filter(filter: &PaintCanvasFilter) -> RenderSvgFilterKind {
    match filter {
        PaintCanvasFilter::kConvolveMatrix {
            order_x,
            order_y,
            divisor,
            bias,
            target_x,
            target_y,
            edge_mode,
            preserve_alpha,
            kernel,
        } => RenderSvgFilterKind::Convolve {
            order_x: u32::try_from(*order_x).unwrap_or(0),
            order_y: u32::try_from(*order_y).unwrap_or(0),
            kernel: kernel.iter().map(|value| *value as f32).collect(),
            divisor: *divisor as f32,
            bias: *bias as f32,
            target_x: *target_x,
            target_y: *target_y,
            edge_mode: match edge_mode {
                // FEConvolveMatrix::ToSkiaTileMode defaults unknown to clamp.
                PaintConvolveEdgeMode::kUnknown | PaintConvolveEdgeMode::kDuplicate => {
                    RenderSvgEdgeMode::Duplicate
                }
                PaintConvolveEdgeMode::kWrap => RenderSvgEdgeMode::Wrap,
                PaintConvolveEdgeMode::kNone => RenderSvgEdgeMode::None,
            },
            preserve_alpha: *preserve_alpha,
        },
        PaintCanvasFilter::kComponentTransfer { functions } => {
            RenderSvgFilterKind::ComponentTransfer(functions.clone().map(|function| {
                match function.r#type {
                    PaintComponentTransferType::kUnknown
                    | PaintComponentTransferType::kIdentity => RenderSvgTransfer::Identity,
                    PaintComponentTransferType::kTable => RenderSvgTransfer::Table(
                        function
                            .table_values
                            .into_iter()
                            .map(|value| value as f32)
                            .collect(),
                    ),
                    PaintComponentTransferType::kDiscrete => RenderSvgTransfer::Discrete(
                        function
                            .table_values
                            .into_iter()
                            .map(|value| value as f32)
                            .collect(),
                    ),
                    PaintComponentTransferType::kLinear => RenderSvgTransfer::Linear {
                        slope: function.slope as f32,
                        intercept: function.intercept as f32,
                    },
                    PaintComponentTransferType::kGamma => RenderSvgTransfer::Gamma {
                        amplitude: function.amplitude as f32,
                        exponent: function.exponent as f32,
                        offset: function.offset as f32,
                    },
                }
            }))
        }
        PaintCanvasFilter::kTurbulence {
            kind,
            base_x,
            base_y,
            octaves,
            seed,
            stitch,
        } => RenderSvgFilterKind::Turbulence {
            base_x: *base_x as f32,
            base_y: *base_y as f32,
            octaves: *octaves,
            seed: *seed as f32,
            stitch: *stitch,
            fractal_noise: matches!(kind, PaintTurbulenceType::kFractalNoise),
        },
    }
}

// Source: cc/paint/render_surface_filters.cc. SVG feColorMatrix uses the
// same CSS color-matrix coefficients, but this graph translation is backend
// neutral and therefore must not reach through either Skia implementation.
fn saturate_matrix(amount: f32) -> [f32; 20] {
    let mut matrix = [0.0; 20];
    matrix[0] = 0.213 + 0.787 * amount;
    matrix[1] = 0.715 - 0.715 * amount;
    matrix[2] = 0.072 - 0.072 * amount;
    matrix[5] = 0.213 - 0.213 * amount;
    matrix[6] = 0.715 + 0.285 * amount;
    matrix[7] = 0.072 - 0.072 * amount;
    matrix[10] = 0.213 - 0.213 * amount;
    matrix[11] = 0.715 - 0.715 * amount;
    matrix[12] = 0.072 + 0.928 * amount;
    matrix[18] = 1.0;
    matrix
}

fn hue_rotate_matrix(amount: f32) -> [f32; 20] {
    let radians = amount.to_radians();
    let cos_hue = radians.cos();
    let sin_hue = radians.sin();
    let mut matrix = [0.0; 20];
    matrix[0] = 0.213 + cos_hue * 0.787 - sin_hue * 0.213;
    matrix[1] = 0.715 - cos_hue * 0.715 - sin_hue * 0.715;
    matrix[2] = 0.072 - cos_hue * 0.072 + sin_hue * 0.928;
    matrix[5] = 0.213 - cos_hue * 0.213 + sin_hue * 0.143;
    matrix[6] = 0.715 + cos_hue * 0.285 + sin_hue * 0.140;
    matrix[7] = 0.072 - cos_hue * 0.072 - sin_hue * 0.283;
    matrix[10] = 0.213 - cos_hue * 0.213 - sin_hue * 0.787;
    matrix[11] = 0.715 - cos_hue * 0.715 + sin_hue * 0.715;
    matrix[12] = 0.072 + cos_hue * 0.928 + sin_hue * 0.072;
    matrix[18] = 1.0;
    matrix
}

fn identity_color_matrix() -> [f32; 20] {
    let mut matrix = [0.0; 20];
    matrix[0] = 1.0;
    matrix[6] = 1.0;
    matrix[12] = 1.0;
    matrix[18] = 1.0;
    matrix
}

fn svg_input(input: &SvgFilterInput) -> String {
    match input {
        SvgFilterInput::kSourceGraphic => "SourceGraphic".into(),
        SvgFilterInput::kSourceAlpha => "SourceAlpha".into(),
        SvgFilterInput::kFillPaint => "FillPaint".into(),
        SvgFilterInput::kStrokePaint => "StrokePaint".into(),
        SvgFilterInput::kNamed(value) => value.clone(),
    }
}

fn image_orientation(
    value: image_resource::ImageOrientation,
) -> raster_commands::commands::ImageOrientation {
    use image_resource::ImageOrientation as Source;
    use raster_commands::commands::ImageOrientation as Target;
    match value {
        Source::kDefault => Target::kDefault,
        Source::kFlipHorizontal => Target::kFlipHorizontal,
        Source::kRotate180 => Target::kRotate180,
        Source::kFlipVertical => Target::kFlipVertical,
        Source::kTranspose => Target::kTranspose,
        Source::kRotate90 => Target::kRotate90,
        Source::kTransverse => Target::kTransverse,
        Source::kRotate270 => Target::kRotate270,
    }
}
pub fn TranslateSvgFilterGraph(
    graph: &SvgFilterGraph,
    reference: PaintRect,
    scale: f64,
) -> io::Result<RenderSvgFilterGraph> {
    TranslateSvgFilterGraphWithTransform(
        graph,
        reference,
        Some(reference),
        1.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        scale,
    )
}

/// Translate a graph from SVG user space into Skia image-filter parameter
/// space. Chromium applies only `Filter::Scale()` while building the graph;
/// Skia applies the Canvas CTM later when it maps parameter space to layer
/// space. Keeping those stages separate prevents transforms from being applied
/// twice and preserves non-axis-aligned CTMs.
pub fn TranslateSvgFilterGraphWithTransform(
    graph: &SvgFilterGraph,
    reference: PaintRect,
    reference_viewport: Option<PaintRect>,
    _scale_x: f64,
    _scale_y: f64,
    _translate_x: f64,
    _translate_y: f64,
    _user_origin_x: f64,
    _user_origin_y: f64,
    user_scale: f64,
) -> io::Result<RenderSvgFilterGraph> {
    if let Some(error) = &graph.unsupported {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("svg-filter: {error}"),
        ));
    }
    let geometry = resolve_svg_filter_geometry(graph, reference, reference_viewport, user_scale)
        .map_err(|error| match error {
            SvgFilterGeometryError::MissingViewport => io::Error::new(
                io::ErrorKind::Unsupported,
                "SVG userSpaceOnUse filter needs its filter viewport",
            ),
        })?;
    // Paint owns filter/primitive rectangle resolution. Raster only projects
    // the resolved immutable geometry into its command vocabulary.
    let rect = |value: PaintRect| {
        [
            value.x as f32,
            value.y as f32,
            value.width as f32,
            value.height as f32,
        ]
    };
    let filter_region = rect(geometry.filter_region);
    let mut primitives = Vec::with_capacity(graph.primitives.len());
    for resolved in &geometry.primitives {
        let primitive = &graph.primitives[resolved.source_index];
        let region = rect(resolved.region);
        let tile_source_region = resolved.tile_source_region.map(rect);
        let kind = match &primitive.kind {
            SvgFilterPrimitiveKind::kGaussianBlur { sigma_x, sigma_y } => {
                RenderSvgFilterKind::GaussianBlur {
                    x: geometry.scale_primitive_x(*sigma_x),
                    y: geometry.scale_primitive_y(*sigma_y),
                }
            }
            SvgFilterPrimitiveKind::kOffset { dx, dy } => RenderSvgFilterKind::Offset {
                x: geometry.scale_primitive_x(*dx),
                y: geometry.scale_primitive_y(*dy),
            },
            SvgFilterPrimitiveKind::kDropShadow {
                sigma_x,
                sigma_y,
                dx,
                dy,
                color,
            } => RenderSvgFilterKind::DropShadow {
                sigma_x: geometry.scale_primitive_x(*sigma_x),
                sigma_y: geometry.scale_primitive_y(*sigma_y),
                dx: geometry.scale_primitive_x(*dx),
                dy: geometry.scale_primitive_y(*dy),
                color: [color.red, color.green, color.blue, color.alpha],
            },
            SvgFilterPrimitiveKind::kFlood { color } => {
                RenderSvgFilterKind::Flood([color.red, color.green, color.blue, color.alpha])
            }
            SvgFilterPrimitiveKind::kColorMatrix { kind, values } => {
                let matrix = match kind {
                    SvgColorMatrixType::kMatrix => {
                        let mut m = identity_color_matrix();
                        if values.len() == 20 {
                            for (i, v) in values.iter().enumerate() {
                                m[i] = *v as f32
                            }
                        }
                        m
                    }
                    SvgColorMatrixType::kSaturate if values.len() == 1 => {
                        saturate_matrix(values[0] as f32)
                    }
                    SvgColorMatrixType::kHueRotate if values.len() == 1 => {
                        hue_rotate_matrix(values[0] as f32)
                    }
                    SvgColorMatrixType::kSaturate | SvgColorMatrixType::kHueRotate => {
                        identity_color_matrix()
                    }
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
                    SvgBlendMode::kOverlay => RenderSvgBlend::Overlay,
                    SvgBlendMode::kColorDodge => RenderSvgBlend::ColorDodge,
                    SvgBlendMode::kColorBurn => RenderSvgBlend::ColorBurn,
                    SvgBlendMode::kHardLight => RenderSvgBlend::HardLight,
                    SvgBlendMode::kSoftLight => RenderSvgBlend::SoftLight,
                    SvgBlendMode::kDifference => RenderSvgBlend::Difference,
                    SvgBlendMode::kExclusion => RenderSvgBlend::Exclusion,
                    SvgBlendMode::kHue => RenderSvgBlend::Hue,
                    SvgBlendMode::kSaturation => RenderSvgBlend::Saturation,
                    SvgBlendMode::kColor => RenderSvgBlend::Color,
                    SvgBlendMode::kLuminosity => RenderSvgBlend::Luminosity,
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
                    SvgCompositeOperator::kLighter => RenderSvgComposite::Lighter,
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
                x: geometry.scale_primitive_x(*radius_x).max(0.0),
                y: geometry.scale_primitive_y(*radius_y).max(0.0),
            },
            SvgFilterPrimitiveKind::kDisplacementMap {
                input2,
                scale: amount,
                x_channel,
                y_channel,
            } => RenderSvgFilterKind::DisplacementMap {
                input2: svg_input(input2),
                // Source: fe_displacement_map.cc:121-127. Chromium applies
                // only the horizontal Filter scale (including object-bbox
                // units); the vertical transform does not participate.
                scale: geometry.scale_primitive_x(*amount),
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
            SvgFilterPrimitiveKind::kTile => RenderSvgFilterKind::Tile {
                source_region: tile_source_region.expect("feTile source region"),
            },
            SvgFilterPrimitiveKind::kImage {
                href,
                local_subtree,
                preserve_aspect_ratio,
                width,
                height,
                orientation,
                pixels,
                ..
            } => RenderSvgFilterKind::Image {
                href: href.clone(),
                local_subtree: *local_subtree,
                align: match preserve_aspect_ratio.align {
                    SvgPreserveAspectRatioAlign::kNone => RenderSvgPreserveAspectRatioAlign::None,
                    SvgPreserveAspectRatioAlign::kXMinYMin => {
                        RenderSvgPreserveAspectRatioAlign::XMinYMin
                    }
                    SvgPreserveAspectRatioAlign::kXMidYMin => {
                        RenderSvgPreserveAspectRatioAlign::XMidYMin
                    }
                    SvgPreserveAspectRatioAlign::kXMaxYMin => {
                        RenderSvgPreserveAspectRatioAlign::XMaxYMin
                    }
                    SvgPreserveAspectRatioAlign::kXMinYMid => {
                        RenderSvgPreserveAspectRatioAlign::XMinYMid
                    }
                    SvgPreserveAspectRatioAlign::kXMidYMid => {
                        RenderSvgPreserveAspectRatioAlign::XMidYMid
                    }
                    SvgPreserveAspectRatioAlign::kXMaxYMid => {
                        RenderSvgPreserveAspectRatioAlign::XMaxYMid
                    }
                    SvgPreserveAspectRatioAlign::kXMinYMax => {
                        RenderSvgPreserveAspectRatioAlign::XMinYMax
                    }
                    SvgPreserveAspectRatioAlign::kXMidYMax => {
                        RenderSvgPreserveAspectRatioAlign::XMidYMax
                    }
                    SvgPreserveAspectRatioAlign::kXMaxYMax => {
                        RenderSvgPreserveAspectRatioAlign::XMaxYMax
                    }
                },
                meet_or_slice: match preserve_aspect_ratio.meet_or_slice {
                    SvgMeetOrSlice::kMeet => RenderSvgMeetOrSlice::Meet,
                    SvgMeetOrSlice::kSlice => RenderSvgMeetOrSlice::Slice,
                },
                width: *width,
                height: *height,
                orientation: image_orientation(*orientation),
                pixels: Arc::clone(pixels),
            },
            SvgFilterPrimitiveKind::kConvolveMatrix {
                order_x,
                order_y,
                kernel,
                divisor,
                bias,
                target_x,
                target_y,
                edge_mode,
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
                // Paint owns Filter::Resolve3dPoint's exact object-box and
                // float scaling order together with all other SVG geometry.
                let point = |x: f64, y: f64, z: f64| geometry.resolve_3d_point(x, y, z);
                let light = light.as_ref().map(|light| match light {
                    SvgLightSource::kDistant { azimuth, elevation } => {
                        let azimuth = (*azimuth as f32).to_radians();
                        let elevation = (*elevation as f32).to_radians();
                        RenderSvgLight::Distant {
                            direction: [
                                azimuth.cos() * elevation.cos(),
                                azimuth.sin() * elevation.cos(),
                                elevation.sin(),
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
                    // Source: fe_lighting.cc:96-132. SurfaceScale is passed to
                    // Skia verbatim; Skia maps this parameter-space Z value
                    // through the CTM when evaluating the image filter.
                    surface_scale: *surface_scale as f32,
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
            origin_tainted: primitive.origin_tainted,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn user_space_graph() -> SvgFilterGraph {
        SvgFilterGraph {
            resource_node_id: 0,
            id: "viewport-filter".into(),
            filter_units: SvgFilterUnits::kUserSpaceOnUse,
            primitive_units: SvgFilterUnits::kUserSpaceOnUse,
            region: SvgFilterRegion::default(),
            primitives: Vec::new(),
            fill_paint: None,
            stroke_paint: None,
            unsupported: None,
        }
    }

    #[test]
    fn default_user_space_region_uses_filter_viewport() {
        let graph = TranslateSvgFilterGraphWithTransform(
            &user_space_graph(),
            PaintRect {
                x: 10.0,
                y: 20.0,
                width: 40.0,
                height: 60.0,
            },
            Some(PaintRect {
                x: 3.0,
                y: 4.0,
                width: 200.0,
                height: 100.0,
            }),
            1.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
        .unwrap();
        assert_eq!(graph.region, [-20.0, -10.0, 240.0, 120.0]);
    }

    #[test]
    fn missing_user_space_viewport_does_not_fall_back_to_reference_box() {
        let error = TranslateSvgFilterGraphWithTransform(
            &user_space_graph(),
            PaintRect {
                x: 10.0,
                y: 20.0,
                width: 40.0,
                height: 60.0,
            },
            None,
            1.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
        .unwrap_err();
        assert!(error.to_string().contains("filter viewport"));
    }

    #[test]
    fn simple_translation_argument_is_filter_zoom() {
        let mut graph = user_space_graph();
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: String::new(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted: false,
            kind: SvgFilterPrimitiveKind::kOffset { dx: 3.0, dy: 4.0 },
        });
        let translated = TranslateSvgFilterGraph(
            &graph,
            PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            },
            2.0,
        )
        .expect("translated graph");
        assert!(matches!(
            translated.primitives[0].kind,
            RenderSvgFilterKind::Offset { x: 6.0, y: 8.0 }
        ));
    }

    #[test]
    fn fe_image_carries_full_preserve_aspect_ratio_across_package_boundary() {
        let mut graph = user_space_graph();
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: String::new(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted: false,
            kind: SvgFilterPrimitiveKind::kImage {
                href: "memory:image".into(),
                local_subtree: false,
                preserve_aspect_ratio:
                    layoutng_assembly::internal::paint_input::SvgPreserveAspectRatio {
                        align: SvgPreserveAspectRatioAlign::kXMaxYMin,
                        meet_or_slice: SvgMeetOrSlice::kSlice,
                    },
                resource_id: Some(1),
                width: 2,
                height: 1,
                orientation: image_resource::ImageOrientation::kRotate90,
                pixels: Arc::new(vec![0; 8]),
            },
        });
        let translated = TranslateSvgFilterGraph(
            &graph,
            PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            },
            1.0,
        )
        .expect("translated graph");
        assert!(matches!(
            translated.primitives[0].kind,
            RenderSvgFilterKind::Image {
                align: RenderSvgPreserveAspectRatioAlign::XMaxYMin,
                meet_or_slice: RenderSvgMeetOrSlice::Slice,
                orientation: raster_commands::commands::ImageOrientation::kRotate90,
                ..
            }
        ));
    }

    #[test]
    fn missing_fe_image_resource_reaches_transparent_black_backend_case() {
        let mut graph = user_space_graph();
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: String::new(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted: true,
            kind: SvgFilterPrimitiveKind::kImage {
                href: "missing.png".into(),
                local_subtree: false,
                preserve_aspect_ratio: Default::default(),
                resource_id: None,
                width: 0,
                height: 0,
                orientation: Default::default(),
                pixels: Arc::new(Vec::new()),
            },
        });
        let translated = TranslateSvgFilterGraph(
            &graph,
            PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            },
            1.0,
        )
        .expect("missing feImage is a transparent image, not unsupported");
        assert!(matches!(
            translated.primitives[0].kind,
            RenderSvgFilterKind::Image {
                local_subtree: false,
                width: 0,
                height: 0,
                ..
            }
        ));
    }

    #[test]
    fn result_that_collides_with_builtin_does_not_replace_last_effect() {
        let mut graph = user_space_graph();
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: "SourceGraphic".into(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted: true,
            kind: SvgFilterPrimitiveKind::kOffset { dx: 5.0, dy: 0.0 },
        });
        let translated = TranslateSvgFilterGraphWithTransform(
            &graph,
            PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            },
            Some(PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            }),
            1.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
        .expect("translated graph");
        assert!(translated.primitives.is_empty());
    }

    #[test]
    fn color_matrix_uses_identity_for_wrong_saturate_and_hue_rotate_arity() {
        let mut graph = user_space_graph();
        for (kind, values) in [
            (SvgColorMatrixType::kSaturate, vec![0.0, 0.5]),
            (SvgColorMatrixType::kHueRotate, vec![30.0, 60.0]),
        ] {
            graph.primitives.push(SvgFilterPrimitive {
                input: SvgFilterInput::kSourceGraphic,
                result: String::new(),
                region: SvgFilterRegion::default(),
                linear_rgb: false,
                origin_tainted: false,
                kind: SvgFilterPrimitiveKind::kColorMatrix { kind, values },
            });
        }
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: String::new(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted: false,
            kind: SvgFilterPrimitiveKind::kColorMatrix {
                kind: SvgColorMatrixType::kSaturate,
                values: vec![0.0],
            },
        });

        let translated = TranslateSvgFilterGraphWithTransform(
            &graph,
            PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            },
            Some(PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            }),
            1.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
        .expect("translated graph");

        let matrices: Vec<[f32; 20]> = translated
            .primitives
            .iter()
            .map(|primitive| match &primitive.kind {
                RenderSvgFilterKind::ColorMatrix(matrix) => *matrix,
                kind => panic!("unexpected filter kind: {kind:?}"),
            })
            .collect();
        assert_eq!(matrices[0], identity_color_matrix());
        assert_eq!(matrices[1], identity_color_matrix());
        assert_eq!(matrices[2], saturate_matrix(0.0));
        assert_ne!(matrices[2], identity_color_matrix());
    }

    #[test]
    fn lighter_composite_reaches_the_render_command() {
        let mut graph = user_space_graph();
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: String::new(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted: false,
            kind: SvgFilterPrimitiveKind::kComposite {
                operator: SvgCompositeOperator::kLighter,
                input2: SvgFilterInput::kSourceAlpha,
                k: [0.0; 4],
            },
        });
        let translated = TranslateSvgFilterGraphWithTransform(
            &graph,
            PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            },
            Some(PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            }),
            1.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
        .expect("translated graph");
        assert!(matches!(
            translated.primitives[0].kind,
            RenderSvgFilterKind::Composite {
                operator: RenderSvgComposite::Lighter,
                ..
            }
        ));
    }

    #[test]
    fn morphology_preserves_fractional_scaled_radii() {
        let mut graph = user_space_graph();
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: String::new(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted: false,
            kind: SvgFilterPrimitiveKind::kMorphology {
                operator: layoutng_assembly::internal::paint_input::SvgMorphologyOperator::kErode,
                radius_x: 0.4,
                radius_y: 0.6,
            },
        });
        let translated = TranslateSvgFilterGraphWithTransform(
            &graph,
            PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            },
            Some(PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            }),
            1.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            2.0,
        )
        .expect("translated graph");
        assert!(matches!(
            translated.primitives[0].kind,
            RenderSvgFilterKind::Morphology {
                dilate: false,
                x,
                y
            } if x == 0.8 && y == 1.2
        ));
    }

    #[test]
    fn displacement_uses_filter_zoom_but_not_the_canvas_ctm() {
        let mut graph = user_space_graph();
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: String::new(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted: false,
            kind: SvgFilterPrimitiveKind::kDisplacementMap {
                input2: SvgFilterInput::kSourceGraphic,
                scale: 3.0,
                x_channel: layoutng_assembly::internal::paint_input::SvgFilterChannel::kA,
                y_channel: layoutng_assembly::internal::paint_input::SvgFilterChannel::kA,
            },
        });
        let translated = TranslateSvgFilterGraphWithTransform(
            &graph,
            PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            },
            Some(PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            }),
            2.0,
            5.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
        .expect("translated graph");
        assert!(matches!(
            translated.primitives[0].kind,
            RenderSvgFilterKind::DisplacementMap { scale, .. } if scale == 3.0
        ));
    }

    #[test]
    fn drop_shadow_leaves_canvas_ctm_to_skia_and_preserves_color() {
        let mut graph = user_space_graph();
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: String::new(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted: false,
            kind: SvgFilterPrimitiveKind::kDropShadow {
                sigma_x: 2.0,
                sigma_y: 3.0,
                dx: 4.0,
                dy: 5.0,
                color: layoutng_assembly::internal::layout_input_types::Color {
                    red: 0.1,
                    green: 0.2,
                    blue: 0.3,
                    alpha: 0.4,
                },
            },
        });
        let translated = TranslateSvgFilterGraphWithTransform(
            &graph,
            PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            },
            Some(PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            }),
            2.0,
            5.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
        .expect("translated graph");
        assert!(matches!(
            translated.primitives[0].kind,
            RenderSvgFilterKind::DropShadow {
                sigma_x: 2.0,
                sigma_y: 3.0,
                dx: 4.0,
                dy: 5.0,
                color: [0.1, 0.2, 0.3, 0.4]
            }
        ));
    }

    #[test]
    fn object_bbox_lighting_point_includes_the_reference_box_origin() {
        let mut graph = user_space_graph();
        graph.primitive_units = SvgFilterUnits::kObjectBoundingBox;
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: String::new(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted: false,
            kind: SvgFilterPrimitiveKind::kLighting {
                lighting_type: SvgLightingType::kDiffuse,
                light: Some(SvgLightSource::kPoint {
                    x: 0.5,
                    y: 0.25,
                    z: 1.0,
                }),
                color: layoutng_assembly::internal::layout_input_types::Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
                surface_scale: 1.0,
                constant: 1.0,
                specular_exponent: 1.0,
            },
        });
        let translated = TranslateSvgFilterGraphWithTransform(
            &graph,
            PaintRect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            },
            Some(PaintRect {
                width: 100.0,
                height: 100.0,
                ..Default::default()
            }),
            2.0,
            3.0,
            5.0,
            7.0,
            0.0,
            0.0,
            1.0,
        )
        .expect("translated graph");
        match &translated.primitives[0].kind {
            RenderSvgFilterKind::Lighting {
                light: Some(RenderSvgLight::Point { position }),
                ..
            } => {
                assert_eq!(position[0], 25.0);
                assert_eq!(position[1], 30.0);
                assert!((position[2] - 35.355_34).abs() < 0.0001);
            }
            kind => panic!("expected point lighting, got {kind:?}"),
        }
    }

    #[test]
    fn user_space_lighting_uses_filter_zoom_without_ctm_or_viewport_translation() {
        let mut graph = user_space_graph();
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: String::new(),
            region: SvgFilterRegion::default(),
            linear_rgb: false,
            origin_tainted: false,
            kind: SvgFilterPrimitiveKind::kLighting {
                lighting_type: SvgLightingType::kDiffuse,
                light: Some(SvgLightSource::kPoint {
                    x: 4.0,
                    y: 5.0,
                    z: 6.0,
                }),
                color: layoutng_assembly::internal::layout_input_types::Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
                surface_scale: 3.0,
                constant: 1.0,
                specular_exponent: 1.0,
            },
        });
        let translated = TranslateSvgFilterGraphWithTransform(
            &graph,
            PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            },
            Some(PaintRect {
                x: 100.0,
                y: 200.0,
                width: 20.0,
                height: 30.0,
            }),
            7.0,
            11.0,
            13.0,
            17.0,
            19.0,
            23.0,
            2.0,
        )
        .expect("translated graph");
        assert!(matches!(
            translated.primitives[0].kind,
            RenderSvgFilterKind::Lighting {
                light: Some(RenderSvgLight::Point {
                    position: [8.0, 10.0, 12.0]
                }),
                surface_scale: 3.0,
                ..
            }
        ));
    }

    #[test]
    fn tile_repeats_the_input_subregion_into_its_own_subregion() {
        let length = |value| {
            Some(SvgFilterLength {
                value,
                percentage: false,
            })
        };
        let mut graph = user_space_graph();
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kSourceGraphic,
            result: "tile-source".into(),
            region: SvgFilterRegion {
                x: length(1.0),
                y: length(2.0),
                width: length(3.0),
                height: length(4.0),
            },
            linear_rgb: false,
            origin_tainted: false,
            kind: SvgFilterPrimitiveKind::kOffset { dx: 0.0, dy: 0.0 },
        });
        graph.primitives.push(SvgFilterPrimitive {
            input: SvgFilterInput::kNamed("tile-source".into()),
            result: String::new(),
            region: SvgFilterRegion {
                x: length(5.0),
                y: length(6.0),
                width: length(7.0),
                height: length(8.0),
            },
            linear_rgb: false,
            origin_tainted: false,
            kind: SvgFilterPrimitiveKind::kTile,
        });
        let translated = TranslateSvgFilterGraphWithTransform(
            &graph,
            PaintRect {
                width: 20.0,
                height: 20.0,
                ..Default::default()
            },
            Some(PaintRect {
                width: 20.0,
                height: 20.0,
                ..Default::default()
            }),
            1.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
        .expect("translated graph");
        assert_eq!(translated.primitives[1].region, [5.0, 6.0, 7.0, 8.0]);
        assert!(matches!(
            translated.primitives[1].kind,
            RenderSvgFilterKind::Tile {
                source_region: [1.0, 2.0, 3.0, 4.0]
            }
        ));
    }
}
