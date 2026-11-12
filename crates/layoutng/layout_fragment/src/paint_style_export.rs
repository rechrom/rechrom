#![allow(non_snake_case)]

#[path = "clip_path_paint_export.rs"]
pub(crate) mod clip_path_export;

use std::sync::Arc;

use foundation::{
    BlendMode, CSSPropertyID, EBackfaceVisibility, EBorderStyle, ECaretShape, EForcedColorAdjust,
    EImageRendering, EIsolation, EObjectFit, EPointerEvents, EPrintColorAdjust, EShapeRendering,
    ETransformBox, EUserSelect, EVectorEffect, EVisibility, Length, LineCap, LineJoin,
    TransformOperationType, TranslateTransformOperation, WindRule,
};
use layoutng_style::style::{
    border_image_length::BorderImageLength,
    computed_style::ComputedStyle,
    computed_style_constants::{
        BackgroundEdgeOrigin, CompositingOperator, EFillAttachment, EFillBox, EFillMaskMode,
        EFillRepeat, EFillSizeType, EPaintOrder,
    },
    fill_layer::FillLayer,
    filter_operation::{
        BasicColorMatrixFilterOperation, BasicComponentTransferFilterOperation,
        BlurFilterOperation, ColorMatrixFilterOperation, ComponentTransferFilterOperation,
        ComponentTransferType, ConvolveMatrixEdgeMode, ConvolveMatrixFilterOperation,
        DropShadowFilterOperation, OperationType as FilterOperationType, ReferenceFilterOperation,
        TurbulenceFilterOperation, TurbulenceType,
    },
    filter_operations::FilterOperations,
    generated_gradient_image::{
        GeneratedGradientData, RadialGradientExtent, RadialGradientShape, ResolvedGradient,
    },
    nine_piece_image::NinePieceImage,
    shadow_data::ShadowStyle,
    style_fetched_image::FetchedImage,
    style_transform::TransformBoxContext,
    svg_paint::{SVGPaint, SVGPaintType},
};

use crate::internal::{
    caret_rect::CaretShape,
    layout_input::{BorderLineStyle, Offset, PaintShadow, TransformMatrix},
    layout_input_types::Color,
    paint_input::{
        BackgroundAttachment, BackgroundBox, BackgroundImageLayer, BackgroundRepeat,
        BackgroundRepeatRule, BackgroundSizeMode, NinePieceImagePaint, NinePieceImageWidth,
        NinePieceImageWidths, PaintBlendMode, PaintCanvasFilter, PaintColorInterpolationSpace,
        PaintColorStop, PaintComponentTransferFunction, PaintComponentTransferType,
        PaintConvolveEdgeMode, PaintCornerRadii, PaintCornerRadius, PaintFilterOperation,
        PaintFilterType, PaintHueInterpolationMethod, PaintImageTileRule, PaintMaskComposite,
        PaintMaskImageState, PaintMaskLayer, PaintMaskMode, PaintRadialExtent, PaintRadialShape,
        PaintReflectionDirection, PaintShader, PaintShaderKind, PaintSpreadMethod, PaintStyleData,
        PaintTransform, PaintTransformOperation, PaintTransformOperationKind, PaintTransformOrigin,
        PaintTransformReferenceBox, PaintTurbulenceType, SvgPaintComponent, SvgStrokeLineCap,
        SvgStrokeLineJoin,
    },
};

use crate::internal::layout_input::{Edges, PaintImage, Size};

pub(crate) fn ColorValue(value: foundation::Color) -> Color {
    // cpp: foundation/graphics_types/graphics/color.cc Color::toSkColor4f.
    // Paint Color is explicitly normalized extended sRGB. Convert at this
    // single ownership boundary so every text/background/border/SVG/filter
    // and gradient-stop path sees the same color-space semantics.
    let (red, green, blue) = value.ToSRGB();
    Color {
        red,
        green,
        blue,
        alpha: value.Alpha(),
    }
}

pub fn StyleColor(
    style: &ComputedStyle,
    value: &layoutng_style::css::style_color::StyleColor,
) -> Color {
    ColorValue(value.Resolve(style.GetCurrentColor(None), style.UsedColorScheme(), None))
}

pub(crate) fn TextEmphasisColor(style: &ComputedStyle) -> Color {
    StyleColor(style, style.TextEmphasisColor())
}

fn LengthParts(value: &Length) -> (f64, f64) {
    if value.IsFixed() {
        return (value.Pixels() as f64, 0.0);
    }
    if value.IsPercent() {
        return (0.0, value.PercentValue() as f64);
    }
    if value.IsCalculated() {
        let calculation = value.GetCalculationValue();
        if !calculation.IsExpression() {
            let pair = calculation.GetPixelsAndPercent();
            return (
                if pair.has_explicit_pixels {
                    pair.pixels as f64
                } else {
                    0.0
                },
                if pair.has_explicit_percent {
                    pair.percent as f64
                } else {
                    0.0
                },
            );
        }
    }
    (0.0, 0.0)
}

fn LengthExpression(value: &Length) -> Option<Arc<foundation::CalculationValue>> {
    if !value.IsCalculated() {
        return None;
    }
    let calculation = value.GetCalculationValue();
    calculation.IsExpression().then_some(calculation)
}

fn Corner(value: &foundation::LengthSize) -> (PaintCornerRadius, PaintCornerRadius) {
    let x = LengthParts(value.Width());
    let y = LengthParts(value.Height());
    (
        PaintCornerRadius { x: x.0, y: y.0 },
        PaintCornerRadius { x: x.1, y: y.1 },
    )
}

fn BorderRadii(style: &ComputedStyle, output: &mut PaintStyleData) {
    let corners = [
        Corner(style.BorderTopLeftRadius()),
        Corner(style.BorderTopRightRadius()),
        Corner(style.BorderBottomRightRadius()),
        Corner(style.BorderBottomLeftRadius()),
    ];
    let fixed = PaintCornerRadii {
        top_left: corners[0].0,
        top_right: corners[1].0,
        bottom_right: corners[2].0,
        bottom_left: corners[3].0,
    };
    output.border_radii_percentages = PaintCornerRadii {
        top_left: corners[0].1,
        top_right: corners[1].1,
        bottom_right: corners[2].1,
        bottom_left: corners[3].1,
    };
    output.border_radii =
        (fixed.HasRadius() || output.border_radii_percentages.HasRadius()).then_some(fixed);
    output.border_corner_curvatures = crate::internal::paint_input::PaintCornerCurvatures([
        style.CornerTopLeftShape().Exponent(),
        style.CornerTopRightShape().Exponent(),
        style.CornerBottomRightShape().Exponent(),
        style.CornerBottomLeftShape().Exponent(),
    ]);
}

fn Shadows(
    style: &ComputedStyle,
    list: *mut layoutng_style::style::shadow_list::ShadowList,
) -> Vec<PaintShadow> {
    if list.is_null() {
        return Vec::new();
    }
    unsafe { &*list }
        .Shadows()
        .iter()
        .map(|shadow| ShadowValue(style, shadow))
        .collect()
}

pub(crate) fn ShadowValue(
    style: &ComputedStyle,
    shadow: &layoutng_style::style::shadow_data::ShadowData,
) -> PaintShadow {
    PaintShadow {
        offset: Offset {
            x: shadow.X() as f64,
            y: shadow.Y() as f64,
        },
        blur_radius: shadow.BlurRadius() as f64,
        spread: shadow.Spread() as f64,
        color: StyleColor(style, shadow.GetColor()),
        inset: shadow.Style() == ShadowStyle::kInset,
    }
}

// ComputedStyle owns the canonical FilterOperation chain.  Preserve each CSS
// operation at the Layout -> Paint boundary instead of reducing `filter` to
// the layout-only HasFilter bit.  Paint's effect property tree and raster
// lowering already consume this ordered payload.
// cpp: core/style/filter_operation.h:63-81,200-225,272-299,320-401
pub(crate) fn Filters(
    style: &ComputedStyle,
    filters: &FilterOperations,
) -> Vec<PaintFilterOperation> {
    filters
        .Operations()
        .iter()
        .filter_map(|member| {
            let operation = unsafe { member.Get().as_ref()? };
            let mut output = PaintFilterOperation::default();
            output.uses_current_color = operation.UsesCurrentColor();
            output.r#type = match operation.GetType() {
                FilterOperationType::kReference => PaintFilterType::kReference,
                FilterOperationType::kGrayscale => PaintFilterType::kGrayscale,
                FilterOperationType::kSepia => PaintFilterType::kSepia,
                FilterOperationType::kSaturate => PaintFilterType::kSaturate,
                FilterOperationType::kHueRotate => PaintFilterType::kHueRotate,
                FilterOperationType::kLuminanceToAlpha => PaintFilterType::kLuminanceToAlpha,
                FilterOperationType::kColorMatrix => PaintFilterType::kColorMatrix,
                FilterOperationType::kInvert => PaintFilterType::kInvert,
                FilterOperationType::kOpacity => PaintFilterType::kOpacity,
                FilterOperationType::kBrightness => PaintFilterType::kBrightness,
                FilterOperationType::kContrast => PaintFilterType::kContrast,
                FilterOperationType::kBlur => PaintFilterType::kBlur,
                FilterOperationType::kDropShadow => PaintFilterType::kDropShadow,
                FilterOperationType::kConvolveMatrix => PaintFilterType::kConvolveMatrix,
                FilterOperationType::kComponentTransfer => PaintFilterType::kComponentTransfer,
                FilterOperationType::kTurbulence => PaintFilterType::kTurbulence,
                _ => {
                    output.r#type = PaintFilterType::kReference;
                    output.reference_error = Some(format!(
                        "unsupported source filter operation {:?}",
                        operation.GetType()
                    ));
                    return Some(output);
                }
            };
            unsafe {
                match operation.GetType() {
                    FilterOperationType::kReference => {
                        let reference =
                            &*(operation as *const _ as *const ReferenceFilterOperation);
                        let url = reference.Url().Utf8();
                        output.reference_url = Some(url.clone());
                        output.reference_error = Some(if url.starts_with('#') {
                            format!("missing local SVG filter {url}")
                        } else {
                            format!("external SVG filter IRI {url}")
                        });
                    }
                    FilterOperationType::kGrayscale
                    | FilterOperationType::kSepia
                    | FilterOperationType::kSaturate
                    | FilterOperationType::kHueRotate
                    | FilterOperationType::kLuminanceToAlpha => {
                        output.amount = (&*(operation as *const _
                            as *const BasicColorMatrixFilterOperation))
                            .Amount();
                    }
                    FilterOperationType::kColorMatrix => {
                        output.color_matrix = (&*(operation as *const _
                            as *const ColorMatrixFilterOperation))
                            .Values()
                            .iter()
                            .map(|value| f64::from(*value))
                            .collect();
                    }
                    FilterOperationType::kInvert
                    | FilterOperationType::kOpacity
                    | FilterOperationType::kBrightness
                    | FilterOperationType::kContrast => {
                        output.amount = (&*(operation as *const _
                            as *const BasicComponentTransferFilterOperation))
                            .Amount();
                    }
                    FilterOperationType::kBlur => {
                        output.amount = LengthParts(
                            (&*(operation as *const _ as *const BlurFilterOperation))
                                .StdDeviation(),
                        )
                        .0;
                    }
                    FilterOperationType::kDropShadow => {
                        let shadow = (&*(operation as *const _
                            as *const DropShadowFilterOperation))
                            .Shadow();
                        output.offset = Offset {
                            x: shadow.X() as f64,
                            y: shadow.Y() as f64,
                        };
                        output.blur_radius = shadow.BlurRadius() as f64;
                        output.color = StyleColor(style, shadow.GetColor());
                        output.color.alpha *= shadow.Opacity();
                    }
                    FilterOperationType::kConvolveMatrix => {
                        let source =
                            &*(operation as *const _ as *const ConvolveMatrixFilterOperation);
                        let size = source.KernelSize();
                        let target = source.TargetOffset();
                        output.canvas_filter = Some(PaintCanvasFilter::kConvolveMatrix {
                            order_x: size.width(),
                            order_y: size.height(),
                            divisor: f64::from(source.Divisor()),
                            bias: f64::from(source.Bias()),
                            target_x: target.x(),
                            target_y: target.y(),
                            edge_mode: match source.EdgeMode() {
                                ConvolveMatrixEdgeMode::kUnknown => PaintConvolveEdgeMode::kUnknown,
                                ConvolveMatrixEdgeMode::kDuplicate => {
                                    PaintConvolveEdgeMode::kDuplicate
                                }
                                ConvolveMatrixEdgeMode::kWrap => PaintConvolveEdgeMode::kWrap,
                                ConvolveMatrixEdgeMode::kNone => PaintConvolveEdgeMode::kNone,
                            },
                            preserve_alpha: source.PreserveAlpha(),
                            kernel: source
                                .KernelMatrix()
                                .iter()
                                .map(|value| f64::from(*value))
                                .collect(),
                        });
                    }
                    FilterOperationType::kComponentTransfer => {
                        let source =
                            &*(operation as *const _ as *const ComponentTransferFilterOperation);
                        output.canvas_filter = Some(PaintCanvasFilter::kComponentTransfer {
                            functions: source.Functions().clone().map(|function| {
                                PaintComponentTransferFunction {
                                    r#type: match function.r#type {
                                        ComponentTransferType::kUnknown => {
                                            PaintComponentTransferType::kUnknown
                                        }
                                        ComponentTransferType::kIdentity => {
                                            PaintComponentTransferType::kIdentity
                                        }
                                        ComponentTransferType::kTable => {
                                            PaintComponentTransferType::kTable
                                        }
                                        ComponentTransferType::kDiscrete => {
                                            PaintComponentTransferType::kDiscrete
                                        }
                                        ComponentTransferType::kLinear => {
                                            PaintComponentTransferType::kLinear
                                        }
                                        ComponentTransferType::kGamma => {
                                            PaintComponentTransferType::kGamma
                                        }
                                    },
                                    slope: f64::from(function.slope),
                                    intercept: f64::from(function.intercept),
                                    amplitude: f64::from(function.amplitude),
                                    exponent: f64::from(function.exponent),
                                    offset: f64::from(function.offset),
                                    table_values: function
                                        .table_values
                                        .into_iter()
                                        .map(f64::from)
                                        .collect(),
                                }
                            }),
                        });
                    }
                    FilterOperationType::kTurbulence => {
                        let source = &*(operation as *const _ as *const TurbulenceFilterOperation);
                        output.canvas_filter = Some(PaintCanvasFilter::kTurbulence {
                            kind: match source.Type() {
                                TurbulenceType::kUnknown => PaintTurbulenceType::kUnknown,
                                TurbulenceType::kFractalNoise => PaintTurbulenceType::kFractalNoise,
                                TurbulenceType::kTurbulence => PaintTurbulenceType::kTurbulence,
                            },
                            base_x: f64::from(source.BaseFrequencyX()),
                            base_y: f64::from(source.BaseFrequencyY()),
                            octaves: source.NumOctaves(),
                            seed: f64::from(source.Seed()),
                            stitch: source.StitchTiles(),
                        });
                    }
                    _ => unreachable!("unsupported filter type was rejected above"),
                }
            }
            Some(output)
        })
        .collect()
}

/// Blink appends box reflection after the authored filter chain.  Keep the
/// four CSS directions here; PrePaint resolves the final flip axis and the
/// size-dependent percentage/edge offset for the physical fragment.
pub(crate) fn AppendBoxReflection(style: &ComputedStyle, output: &mut Vec<PaintFilterOperation>) {
    let Some(reflection) = (unsafe { style.BoxReflect().as_ref() }) else {
        return;
    };
    use layoutng_style::css::css_reflection_direction::CSSReflectionDirection as D;
    let offset = reflection.Offset();
    let (pixels, percent) = LengthParts(offset);
    let calculation = if offset.IsCalculated() {
        let calculation = offset.GetCalculationValue();
        calculation.IsExpression().then_some(calculation)
    } else {
        None
    };
    output.push(PaintFilterOperation {
        r#type: PaintFilterType::kBoxReflect,
        reflection_direction: match reflection.Direction() {
            D::kReflectionAbove => PaintReflectionDirection::kAbove,
            D::kReflectionBelow => PaintReflectionDirection::kBelow,
            D::kReflectionLeft => PaintReflectionDirection::kLeft,
            D::kReflectionRight => PaintReflectionDirection::kRight,
        },
        reflection_offset: pixels,
        reflection_offset_percentage: percent,
        reflection_offset_calculation: calculation,
        reflection_has_mask: reflection.Mask().HasImage(),
        ..PaintFilterOperation::default()
    });
}

fn HasUnprojectedFilter(filters: &FilterOperations) -> bool {
    filters.Operations().iter().any(|member| {
        let Some(operation) = (unsafe { member.Get().as_ref() }) else {
            return true;
        };
        !matches!(
            operation.GetType(),
            FilterOperationType::kGrayscale
                | FilterOperationType::kSepia
                | FilterOperationType::kSaturate
                | FilterOperationType::kHueRotate
                | FilterOperationType::kInvert
                | FilterOperationType::kOpacity
                | FilterOperationType::kBrightness
                | FilterOperationType::kContrast
                | FilterOperationType::kBlur
                | FilterOperationType::kDropShadow
        )
    })
}

fn BoxValue(value: EFillBox) -> BackgroundBox {
    match value {
        EFillBox::kBorder => BackgroundBox::kBorderBox,
        EFillBox::kContent => BackgroundBox::kContentBox,
        EFillBox::kPadding => BackgroundBox::kPaddingBox,
        EFillBox::kText => BackgroundBox::kText,
        EFillBox::kBorderArea => BackgroundBox::kBorderArea,
        EFillBox::kBorderAreaText => BackgroundBox::kBorderAreaText,
        EFillBox::kFillBox => BackgroundBox::kFillBox,
        EFillBox::kStrokeBox => BackgroundBox::kStrokeBox,
        EFillBox::kViewBox => BackgroundBox::kViewBox,
        EFillBox::kNoClip => BackgroundBox::kNoClip,
    }
}

/// Background color belongs to the bottom fill layer.  Its clip therefore
/// comes from the last source FillLayer even when that layer has `image:none`
/// and is intentionally absent from the projected image vector.
pub(crate) fn BottomBackgroundClip(style: &ComputedStyle) -> BackgroundBox {
    let mut layer = style.BackgroundLayers() as *const FillLayer;
    let mut clip = style.BackgroundLayers().Clip();
    while let Some(current) = unsafe { layer.as_ref() } {
        clip = current.Clip();
        layer = current.Next();
    }
    BoxValue(clip)
}

fn RepeatValue(value: EFillRepeat) -> BackgroundRepeatRule {
    match value {
        EFillRepeat::kRepeatFill => BackgroundRepeatRule::kRepeat,
        EFillRepeat::kNoRepeatFill => BackgroundRepeatRule::kNoRepeat,
        EFillRepeat::kRoundFill => BackgroundRepeatRule::kRound,
        EFillRepeat::kSpaceFill => BackgroundRepeatRule::kSpace,
    }
}

pub(crate) fn BlendValue(value: BlendMode) -> PaintBlendMode {
    use BlendMode as N;
    use PaintBlendMode as P;
    match value {
        N::kNormal => P::kNormal,
        N::kMultiply => P::kMultiply,
        N::kScreen => P::kScreen,
        N::kOverlay => P::kOverlay,
        N::kDarken => P::kDarken,
        N::kLighten => P::kLighten,
        N::kColorDodge => P::kColorDodge,
        N::kColorBurn => P::kColorBurn,
        N::kHardLight => P::kHardLight,
        N::kSoftLight => P::kSoftLight,
        N::kDifference => P::kDifference,
        N::kExclusion => P::kExclusion,
        N::kHue => P::kHue,
        N::kSaturation => P::kSaturation,
        N::kColor => P::kColor,
        N::kLuminosity => P::kLuminosity,
        N::kPlusLighter => P::kPlusLighter,
    }
}

/// Freeze a generated StyleImage into the renderer-neutral shader payload.
/// The same boundary is used by fill layers and generated `content`, so the
/// source StyleImage remains the sole owner above Layout/Paint.
pub fn ExportGeneratedImageShader(
    style: &ComputedStyle,
    image: &layoutng_style::style::style_image::StyleImage,
) -> Option<Arc<PaintShader>> {
    let gradient = ResolvedGradient(image)?;
    let interpolation = |space: foundation::ColorSpace, hue: foundation::HueInterpolationMethod| {
        use foundation::ColorSpace as F;
        use PaintColorInterpolationSpace as P;
        let space = match space {
            F::kNone => P::kNone,
            F::kSRGB => P::kSRGB,
            F::kSRGBLegacy => P::kSRGBLegacy,
            F::kSRGBLinear => P::kSRGBLinear,
            F::kDisplayP3 => P::kDisplayP3,
            F::kDisplayP3Linear => P::kDisplayP3Linear,
            F::kHSL => P::kHSL,
            F::kHWB => P::kHWB,
            F::kA98RGB => P::kA98RGB,
            F::kProPhotoRGB => P::kProPhotoRGB,
            F::kRec2020 => P::kRec2020,
            F::kRec2100Linear => P::kRec2100Linear,
            F::kLab => P::kLab,
            F::kLch => P::kLch,
            F::kOklab => P::kOklab,
            F::kOklch => P::kOklch,
            F::kXYZD50 => P::kXYZD50,
            F::kXYZD65 => P::kXYZD65,
            _ => panic!("unsupported generated-gradient interpolation space {space:?}"),
        };
        let hue = match hue {
            foundation::HueInterpolationMethod::kShorter => PaintHueInterpolationMethod::kShorter,
            foundation::HueInterpolationMethod::kLonger => PaintHueInterpolationMethod::kLonger,
            foundation::HueInterpolationMethod::kIncreasing => {
                PaintHueInterpolationMethod::kIncreasing
            }
            foundation::HueInterpolationMethod::kDecreasing => {
                PaintHueInterpolationMethod::kDecreasing
            }
        };
        (space, hue)
    };
    let stops = |values: &[layoutng_style::style::generated_gradient_image::GradientStop]| {
        values
            .iter()
            .map(|stop| {
                let resolved =
                    stop.color
                        .Resolve(style.GetCurrentColor(None), style.UsedColorScheme(), None);
                let (source_color_space, _) = interpolation(
                    resolved.GetColorSpace(),
                    foundation::HueInterpolationMethod::kShorter,
                );
                PaintColorStop {
                    offset: stop.offset,
                    color: ColorValue(resolved),
                    source_color_space,
                    source_components: [
                        resolved.Param0(),
                        resolved.Param1(),
                        resolved.Param2(),
                        resolved.Alpha(),
                    ],
                    missing_components: u8::from(resolved.Param0IsNone())
                        | (u8::from(resolved.Param1IsNone()) << 1)
                        | (u8::from(resolved.Param2IsNone()) << 2)
                        | (u8::from(resolved.AlphaIsNone()) << 3),
                    offset_length: stop.offset_length,
                    is_hint: stop.is_hint,
                }
            })
            .collect()
    };
    let shader = match gradient {
        GeneratedGradientData::Linear(gradient) => {
            let (color_interpolation_space, hue_interpolation_method) = interpolation(
                gradient.color_interpolation_space,
                gradient.hue_interpolation_method,
            );
            PaintShader {
                kind: PaintShaderKind::kLinearGradient,
                spread: if gradient.repeating {
                    PaintSpreadMethod::kRepeat
                } else {
                    PaintSpreadMethod::kPad
                },
                start: Offset {
                    x: gradient.start[0],
                    y: gradient.start[1],
                },
                end: Offset {
                    x: gradient.end[0],
                    y: gradient.end[1],
                },
                linear_angle: gradient.angle_degrees,
                linear_corner: gradient.corner,
                stops: stops(&gradient.stops),
                unit_coordinates: true,
                color_interpolation_space,
                hue_interpolation_method,
                ..PaintShader::default()
            }
        }
        GeneratedGradientData::Radial(gradient) => {
            let (color_interpolation_space, hue_interpolation_method) = interpolation(
                gradient.color_interpolation_space,
                gradient.hue_interpolation_method,
            );
            PaintShader {
                kind: PaintShaderKind::kRadialGradient,
                spread: if gradient.repeating {
                    PaintSpreadMethod::kRepeat
                } else {
                    PaintSpreadMethod::kPad
                },
                center: Offset {
                    x: gradient.center_x.percent,
                    y: gradient.center_y.percent,
                },
                center_offset: Offset {
                    x: gradient.center_x.pixels,
                    y: gradient.center_y.pixels,
                },
                focal: Offset {
                    x: gradient.center_x.percent,
                    y: gradient.center_y.percent,
                },
                radius: gradient.radius_x.percent,
                radius_offset: gradient.radius_x.pixels,
                radius_y: Some(gradient.radius_y.percent),
                radius_y_offset: gradient.radius_y.pixels,
                radial_shape: match gradient.shape {
                    RadialGradientShape::Circle => PaintRadialShape::kCircle,
                    RadialGradientShape::Ellipse => PaintRadialShape::kEllipse,
                },
                radial_extent: match gradient.extent {
                    RadialGradientExtent::Explicit => PaintRadialExtent::kExplicit,
                    RadialGradientExtent::ClosestSide => PaintRadialExtent::kClosestSide,
                    RadialGradientExtent::ClosestCorner => PaintRadialExtent::kClosestCorner,
                    RadialGradientExtent::FarthestSide => PaintRadialExtent::kFarthestSide,
                    RadialGradientExtent::FarthestCorner => PaintRadialExtent::kFarthestCorner,
                },
                stops: stops(&gradient.stops),
                unit_coordinates: true,
                color_interpolation_space,
                hue_interpolation_method,
                ..PaintShader::default()
            }
        }
        GeneratedGradientData::Conic(gradient) => {
            let (color_interpolation_space, hue_interpolation_method) = interpolation(
                gradient.color_interpolation_space,
                gradient.hue_interpolation_method,
            );
            PaintShader {
                kind: PaintShaderKind::kConicGradient,
                spread: if gradient.repeating {
                    PaintSpreadMethod::kRepeat
                } else {
                    PaintSpreadMethod::kPad
                },
                center: Offset {
                    x: gradient.center_x.percent,
                    y: gradient.center_y.percent,
                },
                center_offset: Offset {
                    x: gradient.center_x.pixels,
                    y: gradient.center_y.pixels,
                },
                rotation_angle: gradient.angle_degrees,
                stops: stops(&gradient.stops),
                unit_coordinates: true,
                color_interpolation_space,
                hue_interpolation_method,
                ..PaintShader::default()
            }
        }
    };
    Some(Arc::new(shader))
}

fn BackgroundLayer(style: &ComputedStyle, layer: &FillLayer) -> Option<BackgroundImageLayer> {
    let image = unsafe { layer.GetImage().as_ref() }?;
    let image = layoutng_style::style::style_mask_source_image::MaskSourceImage(image)
        .and_then(|mask| mask.FetchedImage())
        .unwrap_or(image);
    let mut output = BackgroundImageLayer::default();
    output.can_render = image.CanRender();
    output.has_intrinsic_size = image.HasIntrinsicSize();
    output.image_occludes_next_layers = layer.ImageOccludesNextLayers(style);
    if let Some(shader) = ExportGeneratedImageShader(style, image) {
        output.shader = Some(shader);
    } else if let Some(fetched) = FetchedImage(image) {
        output.resource_id = fetched.ResourceId();
        output.source_url = fetched.SourceURL().to_owned();
        output.image_resolution_scale = fetched.ResolutionScale();
    } else {
        return None;
    }
    BackgroundLayerGeometry(layer, &mut output);
    Some(output)
}

fn BackgroundLayerGeometry(layer: &FillLayer, output: &mut BackgroundImageLayer) {
    output.origin = BoxValue(layer.Origin());
    output.clip = BoxValue(layer.Clip());
    output.attachment = match layer.Attachment() {
        EFillAttachment::kScroll => BackgroundAttachment::kScroll,
        EFillAttachment::kLocal => BackgroundAttachment::kLocal,
        EFillAttachment::kFixed => BackgroundAttachment::kFixed,
    };
    output.blend_mode = BlendValue(layer.GetBlendMode());
    output.repeat_rule_x = Some(RepeatValue(layer.Repeat().x));
    output.repeat_rule_y = Some(RepeatValue(layer.Repeat().y));
    output.repeat = match (layer.Repeat().x, layer.Repeat().y) {
        (EFillRepeat::kRepeatFill, EFillRepeat::kNoRepeatFill) => BackgroundRepeat::kRepeatX,
        (EFillRepeat::kNoRepeatFill, EFillRepeat::kRepeatFill) => BackgroundRepeat::kRepeatY,
        (EFillRepeat::kNoRepeatFill, EFillRepeat::kNoRepeatFill) => BackgroundRepeat::kNoRepeat,
        _ => BackgroundRepeat::kRepeat,
    };

    let x = LengthParts(layer.PositionX());
    let y = LengthParts(layer.PositionY());
    output.position_x_calculation = LengthExpression(layer.PositionX());
    output.position_y_calculation = LengthExpression(layer.PositionY());
    output.position_x_has_nonzero_percent =
        layer.PositionX().HasPercent() && !layer.PositionX().IsZero();
    output.position_y_has_nonzero_percent =
        layer.PositionY().HasPercent() && !layer.PositionY().IsZero();
    output.position_x_from_right = output.position_x_calculation.is_some()
        && layer.BackgroundXOrigin() == BackgroundEdgeOrigin::kRight;
    output.position_y_from_bottom = output.position_y_calculation.is_some()
        && layer.BackgroundYOrigin() == BackgroundEdgeOrigin::kBottom;
    output.origin_x_from_right = layer.BackgroundXOrigin() == BackgroundEdgeOrigin::kRight;
    output.origin_y_from_bottom = layer.BackgroundYOrigin() == BackgroundEdgeOrigin::kBottom;
    output.position.x = x.1 / 100.0;
    output.position.y = y.1 / 100.0;
    output.position_offset.x = x.0;
    output.position_offset.y = y.0;
    if layer.BackgroundXOrigin() == BackgroundEdgeOrigin::kRight {
        output.position.x = 1.0 - output.position.x;
        output.position_offset.x = -output.position_offset.x;
    }
    if layer.BackgroundYOrigin() == BackgroundEdgeOrigin::kBottom {
        output.position.y = 1.0 - output.position.y;
        output.position_offset.y = -output.position_offset.y;
    }
    output.size_mode = match layer.SizeType() {
        EFillSizeType::kContain => BackgroundSizeMode::kContain,
        EFillSizeType::kCover => BackgroundSizeMode::kCover,
        EFillSizeType::kSizeNone => BackgroundSizeMode::kAuto,
        EFillSizeType::kSizeLength => BackgroundSizeMode::kExplicit,
    };
    if layer.SizeType() == EFillSizeType::kSizeLength {
        let width = LengthParts(layer.SizeLength().Width());
        let height = LengthParts(layer.SizeLength().Height());
        output.width_calculation = LengthExpression(layer.SizeLength().Width());
        output.height_calculation = LengthExpression(layer.SizeLength().Height());
        if !layer.SizeLength().Width().IsAuto() {
            // A calculated length may carry both terms.  Keeping them
            // mutually exclusive turns `calc(50% + 10px)` into `50%` and
            // shifts every repeat/round/space decision downstream.
            output.width = (width.0 != 0.0 || width.1 == 0.0).then_some(width.0);
            output.width_percentage = (width.1 != 0.0).then_some(width.1 / 100.0);
        }
        if !layer.SizeLength().Height().IsAuto() {
            output.height = (height.0 != 0.0 || height.1 == 0.0).then_some(height.0);
            output.height_percentage = (height.1 != 0.0).then_some(height.1 / 100.0);
        }
    }
}

pub(crate) fn Backgrounds(style: &ComputedStyle) -> Vec<BackgroundImageLayer> {
    let mut output = Vec::new();
    let mut layer = style.BackgroundLayers() as *const FillLayer;
    while let Some(current) = unsafe { layer.as_ref() } {
        if let Some(mut projected) = BackgroundLayer(style, current) {
            projected.paints_background_color = current.Next().is_null();
            output.push(projected);
        }
        layer = current.Next();
    }
    output
}

pub(crate) fn BackgroundFillLayerCount(style: &ComputedStyle) -> usize {
    let mut count = 0;
    let mut layer = style.BackgroundLayers() as *const FillLayer;
    while let Some(current) = unsafe { layer.as_ref() } {
        count += 1;
        layer = current.Next();
    }
    count
}

fn MaskComposite(value: CompositingOperator) -> Option<PaintMaskComposite> {
    match value {
        CompositingOperator::kAdd => Some(PaintMaskComposite::kAdd),
        CompositingOperator::kSubtract => Some(PaintMaskComposite::kSubtract),
        CompositingOperator::kIntersect => Some(PaintMaskComposite::kIntersect),
        CompositingOperator::kExclude => Some(PaintMaskComposite::kExclude),
        _ => None,
    }
}

/// Freeze the source mask FillLayer chain together with its mask-only fields.
/// Background and mask layers share image geometry in Blink, but mask mode and
/// compositing live on FillLayer and must cross the paint boundary as well.
pub(crate) fn Masks(style: &ComputedStyle) -> Vec<PaintMaskLayer> {
    // A sole `none` means no mask, but `none` inside an active layer list is a
    // non-drawing slot that preserves the original bottom-layer identity.
    if !style.MaskLayers().AnyLayerHasImage() {
        return Vec::new();
    }
    let mut output = Vec::new();
    let mut layer = style.MaskLayers() as *const FillLayer;
    while let Some(current) = unsafe { layer.as_ref() } {
        let style_image = unsafe { current.GetImage().as_ref() };
        let image_state = match style_image {
            Some(image) if !image.IsLoaded() && !image.ErrorOccurred() => {
                PaintMaskImageState::kLoading
            }
            Some(image) if image.CanRender() => PaintMaskImageState::kReady,
            _ => PaintMaskImageState::kInvalid,
        };
        let local = style_image
            .and_then(layoutng_style::style::style_mask_source_image::MaskSourceImage)
            .filter(|mask| mask.IsLocal());
        let image = local
            .map(|_| BackgroundImageLayer::default())
            .or_else(|| BackgroundLayer(style, current))
            .or_else(|| {
                style_image.is_none().then(|| {
                    let mut image = BackgroundImageLayer::default();
                    BackgroundLayerGeometry(current, &mut image);
                    image
                })
            });
        let composite = MaskComposite(current.CompositingOperator());
        if let (Some(image), Some(composite)) = (image, composite) {
            output.push(PaintMaskLayer {
                svg_reference: local.map(|mask| crate::internal::paint_input::SvgMaskReference {
                    id: mask.NormalizedFragment(),
                    scope: mask.ResourceScope().unwrap(),
                    graph: None,
                    target_id: None,
                    viewport: None,
                }),
                match_source: current.MaskMode() == EFillMaskMode::kMatchSource,
                source_error: style_image
                    .and_then(layoutng_style::style::style_mask_source_image::MaskSourceImage)
                    .filter(|mask| !mask.IsLocal() && !mask.NormalizedFragment().is_empty())
                    .and_then(|mask| mask.FetchedImage())
                    .and_then(FetchedImage)
                    .filter(|image| image.IsDocumentImage())
                    .map(|_| "external SVG mask target selection".to_owned()),
                image,
                image_state,
                mode: match current.MaskMode() {
                    EFillMaskMode::kLuminance => PaintMaskMode::kLuminance,
                    // CSS generated images and ordinary images use alpha for
                    // match-source; SVG <mask> resources take the luminance
                    // path before becoming this generated-image subtype.
                    EFillMaskMode::kAlpha | EFillMaskMode::kMatchSource => PaintMaskMode::kAlpha,
                },
                composite,
            });
        }
        layer = current.Next();
    }
    output
}

pub(crate) fn MaskReferences(
    style: &ComputedStyle,
) -> Vec<crate::internal::paint_input::SvgMaskReference> {
    if !style.MaskLayers().AnyLayerHasImage() {
        return Vec::new();
    }
    let mut output: Vec<crate::internal::paint_input::SvgMaskReference> = Vec::new();
    let mut layer = style.MaskLayers() as *const FillLayer;
    while let Some(current) = unsafe { layer.as_ref() } {
        let local = unsafe { current.GetImage().as_ref() }
            .and_then(layoutng_style::style::style_mask_source_image::MaskSourceImage)
            .filter(|mask| mask.IsLocal());
        if let Some(mask) = local {
            let reference = crate::internal::paint_input::SvgMaskReference {
                id: mask.NormalizedFragment(),
                scope: mask.ResourceScope().unwrap(),
                graph: None,
                target_id: None,
                viewport: None,
            };
            if !output
                .iter()
                .any(|existing| existing.id == reference.id && existing.scope == reference.scope)
            {
                output.push(reference);
            }
        }
        layer = current.Next();
    }
    output
}

fn BorderImageRule(
    value: layoutng_style::style::nine_piece_image::ENinePieceImageRule,
) -> PaintImageTileRule {
    use layoutng_style::style::nine_piece_image::ENinePieceImageRule as N;
    match value {
        N::kStretchImageRule => PaintImageTileRule::kStretch,
        N::kRoundImageRule => PaintImageTileRule::kRound,
        N::kSpaceImageRule => PaintImageTileRule::kSpace,
        N::kRepeatImageRule => PaintImageTileRule::kRepeat,
    }
}

fn ResolveBorderImageLength(value: &BorderImageLength, border: f64, basis: f64) -> f64 {
    if value.IsNumber() {
        return (value.Number() * border).max(0.0);
    }
    let length = value.length();
    if length.IsAuto() {
        return f64::NAN;
    }
    let (pixels, percent) = LengthParts(length);
    (pixels + basis * percent / 100.0).max(0.0)
}

fn ResolveBorderImageSlice(value: &Length, scale: f64, maximum: f64) -> f64 {
    let resolved = if value.IsFixed() {
        f64::from(value.Pixels()) * scale
    } else {
        let (_, percent) = LengthParts(value);
        maximum * percent / 100.0
    };
    foundation::LayoutUnit::FromFloatRound(resolved.clamp(0.0, maximum) as f32).ToDouble()
}

fn ExportBorderImageWidth(value: &BorderImageLength) -> NinePieceImageWidth {
    if value.IsNumber() {
        return NinePieceImageWidth::Number(value.Number());
    }
    let length = value.length();
    if length.IsAuto() {
        return NinePieceImageWidth::Auto;
    }
    let (pixels, percentage) = LengthParts(length);
    NinePieceImageWidth::Length {
        pixels,
        percentage,
        calculation: LengthExpression(length),
    }
}

pub fn FetchedNinePieceImageResourceId(
    image: &layoutng_style::style::style_image::StyleImage,
) -> Option<u64> {
    FetchedImage(image).map(|fetched| fetched.ResourceId())
}

/// Freeze the source NinePieceImage after the physical fragment size is known.
/// Width syntax remains typed so NinePieceImageGrid can resolve percentages
/// against the final pixel-snapped area. Raster slices stay in image pixel
/// space, exactly as the grid expects.
pub fn ExportNinePieceImageForFragment(
    style: &ComputedStyle,
    source: &NinePieceImage,
    fragment_size: Size,
    borders: Edges,
    image: Option<&PaintImage>,
) -> Option<NinePieceImagePaint> {
    let style_image = unsafe { source.GetImage().as_ref() }?;
    let is_loaded = style_image.IsLoaded();
    let can_render = style_image.CanRender();
    let respect_orientation = style_image.ForceOrientationIfNecessary(style.ImageOrientation())
        == foundation::RespectImageOrientationEnum::kRespectImageOrientation;
    let image_orientation = image
        .filter(|_| respect_orientation)
        .map_or(image_resource::ImageOrientation::default(), |image| {
            image.orientation
        });
    let fetched = FetchedImage(style_image);
    let resource_id = fetched.map_or(0, |fetched| fetched.ResourceId());
    let outset = source.Outset();
    let outsets = Edges {
        top: ResolveBorderImageLength(outset.Top(), borders.top, fragment_size.height),
        right: ResolveBorderImageLength(outset.Right(), borders.right, fragment_size.width),
        bottom: ResolveBorderImageLength(outset.Bottom(), borders.bottom, fragment_size.height),
        left: ResolveBorderImageLength(outset.Left(), borders.left, fragment_size.width),
    };
    debug_assert!(
        [outsets.top, outsets.right, outsets.bottom, outsets.left]
            .iter()
            .all(|value| value.is_finite()),
        "border-image-outset cannot be auto"
    );
    let area_width = fragment_size.width + outsets.left + outsets.right;
    let area_height = fragment_size.height + outsets.top + outsets.bottom;
    let source_widths = source.BorderSlices();
    let widths = NinePieceImageWidths {
        top: ExportBorderImageWidth(source_widths.Top()),
        right: ExportBorderImageWidth(source_widths.Right()),
        bottom: ExportBorderImageWidth(source_widths.Bottom()),
        left: ExportBorderImageWidth(source_widths.Left()),
    };
    let effective_zoom = f64::from(style.EffectiveZoom()).max(f64::MIN_POSITIVE);
    let shader = ExportGeneratedImageShader(style, style_image);
    let (image_size, scale) = if shader.is_some() {
        // StyleGeneratedImage::ImageSize returns the default object size. Its
        // unzoomed size is smaller by EffectiveZoom, so fixed slice numbers
        // are scaled by zoom exactly as NinePieceImagePainter does in Blink.
        (
            Size {
                width: area_width,
                height: area_height,
            },
            effective_zoom,
        )
    } else if let Some(image) = image {
        let (width, height) =
            image_orientation.oriented_size(f64::from(image.width), f64::from(image.height));
        (
            Size { width, height },
            fetched
                .map(|fetched| fetched.ResolutionScale())
                .unwrap_or(image.resolution_scale)
                .max(f64::MIN_POSITIVE),
        )
    } else {
        // Pending and unsupported StyleImages still suppress fallback borders.
        return Some(NinePieceImagePaint {
            resource_id,
            is_loaded,
            can_render,
            image_orientation,
            widths,
            border_widths: borders,
            horizontal_rule: BorderImageRule(source.HorizontalRule()),
            vertical_rule: BorderImageRule(source.VerticalRule()),
            fill: source.Fill(),
            ..NinePieceImagePaint::default()
        });
    };
    let slices = Edges {
        top: ResolveBorderImageSlice(source.ImageSlices().Top(), scale, image_size.height),
        right: ResolveBorderImageSlice(source.ImageSlices().Right(), scale, image_size.width),
        bottom: ResolveBorderImageSlice(source.ImageSlices().Bottom(), scale, image_size.height),
        left: ResolveBorderImageSlice(source.ImageSlices().Left(), scale, image_size.width),
    };
    Some(NinePieceImagePaint {
        resource_id,
        is_loaded,
        can_render,
        image_resolution_scale: scale,
        image_size,
        image_orientation,
        shader,
        slices,
        widths,
        border_widths: borders,
        outsets,
        horizontal_rule: BorderImageRule(source.HorizontalRule()),
        vertical_rule: BorderImageRule(source.VerticalRule()),
        fill: source.Fill(),
    })
}

pub(crate) fn BorderStyle(value: EBorderStyle) -> BorderLineStyle {
    use BorderLineStyle as P;
    use EBorderStyle as N;
    match value {
        N::kNone => P::kNone,
        N::kHidden => P::kHidden,
        N::kSolid => P::kSolid,
        N::kDashed => P::kDashed,
        N::kDotted => P::kDotted,
        N::kDouble => P::kDouble,
        N::kGroove => P::kGroove,
        N::kRidge => P::kRidge,
        N::kInset => P::kInset,
        N::kOutset => P::kOutset,
    }
}

fn PushTranslate(output: &mut PaintTransform, translate: &TranslateTransformOperation) {
    let operation_type =
        <TranslateTransformOperation as foundation::TransformOperation>::GetType(translate);
    let x = LengthParts(translate.XLength());
    let y = LengthParts(translate.YLength());
    output.operations.push(PaintTransformOperation {
        kind: if matches!(
            operation_type,
            TransformOperationType::kTranslateZ | TransformOperationType::kTranslate3D
        ) {
            PaintTransformOperationKind::kTranslate3D
        } else {
            PaintTransformOperationKind::kTranslate
        },
        pixels: Offset { x: x.0, y: y.0 },
        percentages: Offset { x: x.1, y: y.1 },
        z: translate.Z(),
        ..PaintTransformOperation::default()
    });
}

fn PushMatrix(output: &mut PaintTransform, matrix: foundation::gfx::Transform) {
    output.operations.push(PaintTransformOperation {
        kind: PaintTransformOperationKind::kMatrix,
        matrix: TransformMatrix {
            values: matrix.GetColMajor(),
        },
        ..PaintTransformOperation::default()
    });
}

pub(crate) fn Translate(style: &ComputedStyle) -> Option<PaintTransform> {
    let translate = style.Translate();
    if translate.is_null() {
        return None;
    }
    let mut output = PaintTransform::default();
    PushTranslate(&mut output, unsafe { &*translate });
    Some(output)
}

pub(crate) fn Rotate(style: &ComputedStyle) -> Option<PaintTransform> {
    let rotate = style.Rotate();
    if rotate.is_null() {
        return None;
    }
    let rotate = unsafe { &*rotate };
    let mut matrix = foundation::gfx::Transform::default();
    rotate.Apply(&mut matrix, &foundation::gfx::SizeF::new(0.0, 0.0));
    Some(PaintTransform {
        operations: vec![PaintTransformOperation {
            kind: PaintTransformOperationKind::kRotate,
            matrix: TransformMatrix {
                values: matrix.GetColMajor(),
            },
            vector: [rotate.X(), rotate.Y(), rotate.Z()],
            angle: rotate.Angle(),
            ..PaintTransformOperation::default()
        }],
    })
}

pub(crate) fn Scale(style: &ComputedStyle) -> Option<PaintTransform> {
    let scale = style.Scale();
    if scale.is_null() {
        return None;
    }
    let scale = unsafe { &*scale };
    let mut matrix = foundation::gfx::Transform::default();
    scale.Apply(&mut matrix, &foundation::gfx::SizeF::new(0.0, 0.0));
    Some(PaintTransform {
        operations: vec![PaintTransformOperation {
            kind: PaintTransformOperationKind::kScale,
            matrix: TransformMatrix {
                values: matrix.GetColMajor(),
            },
            vector: [scale.X(), scale.Y(), scale.Z()],
            ..PaintTransformOperation::default()
        }],
    })
}

pub(crate) fn Transform(style: &ComputedStyle) -> Option<PaintTransform> {
    let mut output = PaintTransform::default();

    // PaintPropertyTreeBuilder updates independent translate/rotate/scale
    // before this legacy transform node. Keep this payload to the source list.
    for operation in style.Transform().Operations().iter() {
        let operation = operation.GetNonNull().expect("transform operation is null");
        let operation = unsafe { operation.as_ref() };
        if matches!(
            operation.GetType(),
            TransformOperationType::kTranslate
                | TransformOperationType::kTranslateX
                | TransformOperationType::kTranslateY
                | TransformOperationType::kTranslateZ
                | TransformOperationType::kTranslate3D
        ) {
            let translate = unsafe {
                &*(operation as *const dyn foundation::TransformOperation
                    as *const TranslateTransformOperation)
            };
            PushTranslate(&mut output, translate);
        } else {
            let mut matrix = foundation::gfx::Transform::default();
            operation.Apply(&mut matrix, &foundation::gfx::SizeF::new(0.0, 0.0));
            PushMatrix(&mut output, matrix);
        }
    }
    (!output.operations.is_empty()).then_some(output)
}

pub(crate) fn TransformOrigin(style: &ComputedStyle) -> PaintTransformOrigin {
    let origin = style.GetTransformOrigin();
    let x = LengthParts(origin.X());
    let y = LengthParts(origin.Y());
    PaintTransformOrigin {
        pixels: Offset { x: x.0, y: y.0 },
        percentages: Offset { x: x.1, y: y.1 },
        z: origin.Z() as f64,
    }
}

pub(crate) fn PerspectiveOrigin(style: &ComputedStyle) -> PaintTransformOrigin {
    let origin = style.PerspectiveOrigin();
    let x = LengthParts(origin.X());
    let y = LengthParts(origin.Y());
    PaintTransformOrigin {
        pixels: Offset { x: x.0, y: y.0 },
        percentages: Offset { x: x.1, y: y.1 },
        z: 0.0,
    }
}

fn ObjectFitValue(value: EObjectFit) -> crate::internal::layout_input::ObjectFit {
    use crate::internal::layout_input::ObjectFit as P;
    match value {
        EObjectFit::kFill => P::kFill,
        EObjectFit::kContain => P::kContain,
        EObjectFit::kCover => P::kCover,
        EObjectFit::kNone => P::kNone,
        EObjectFit::kScaleDown => P::kScaleDown,
    }
}

fn PaintOrder(value: EPaintOrder) -> [SvgPaintComponent; 3] {
    use SvgPaintComponent::{kFill as F, kMarkers as M, kStroke as S};
    match value {
        EPaintOrder::kPaintOrderNormal | EPaintOrder::kPaintOrderFillStrokeMarkers => [F, S, M],
        EPaintOrder::kPaintOrderFillMarkersStroke => [F, M, S],
        EPaintOrder::kPaintOrderStrokeFillMarkers => [S, F, M],
        EPaintOrder::kPaintOrderStrokeMarkersFill => [S, M, F],
        EPaintOrder::kPaintOrderMarkersFillStroke => [M, F, S],
        EPaintOrder::kPaintOrderMarkersStrokeFill => [M, S, F],
    }
}

fn SvgStrokeStyle(style: &ComputedStyle, output: &mut PaintStyleData) {
    output.svg_stop_color = StyleColor(style, style.StopColor());
    output.svg_stop_current_color = style.StopColor().IsCurrentColor();
    output.svg_stop_opacity = style.StopOpacity();
    output.svg_stroke_width = LengthParts(style.StrokeWidth().length()).0;
    output.svg_stroke_dash_offset = LengthParts(style.StrokeDashOffset()).0;
    let dash_array = style.StrokeDashArray();
    if !dash_array.is_null() {
        output.svg_stroke_dash_array = unsafe { &*dash_array }
            .iter()
            .map(|length| LengthParts(length).0)
            .collect();
    }
    output.svg_stroke_line_cap = match style.CapStyle() {
        LineCap::kButtCap => SvgStrokeLineCap::kButt,
        LineCap::kRoundCap => SvgStrokeLineCap::kRound,
        LineCap::kSquareCap => SvgStrokeLineCap::kSquare,
    };
    output.svg_stroke_line_join = match style.JoinStyle() {
        LineJoin::kMiterJoin => SvgStrokeLineJoin::kMiter,
        LineJoin::kRoundJoin => SvgStrokeLineJoin::kRound,
        LineJoin::kBevelJoin => SvgStrokeLineJoin::kBevel,
    };
    output.svg_stroke_miter_limit = style.StrokeMiterLimit() as f64;
    output.svg_fill_even_odd = style.FillRule() == WindRule::RULE_EVENODD;
    output.svg_clip_even_odd = style.ClipRule() == WindRule::RULE_EVENODD;
    output.svg_non_scaling_stroke = style.VectorEffect() == EVectorEffect::kNonScalingStroke;
    // svg_shape_painter.cc:99-102 disables antialiasing for these two
    // shape-rendering values and enables it for auto/geometricPrecision.
    output.svg_shape_antialias = !matches!(
        style.ShapeRendering(),
        EShapeRendering::kCrispedges | EShapeRendering::kOptimizespeed
    );
    output.svg_paint_order = PaintOrder(style.PaintOrder());
}

pub fn SvgColor(
    style: &ComputedStyle,
    paint: &SVGPaint,
    visited_paint: &SVGPaint,
    opacity: f32,
) -> Option<Color> {
    if paint.paint_type == SVGPaintType::kNone || !paint.HasColor() {
        return None;
    }
    let mut color = ColorValue(style.VisitedDependentSvgPaint(paint, visited_paint));
    color.alpha *= opacity;
    Some(color)
}

// Only values that Paint has already resolved into fragment or resource data
// are retained here. Canonical style flags remain on ComputedStyle and are
// read through PaintStyle's shared owner.
fn ExportGeometryProperties(
    style: &ComputedStyle,
    output: &mut PaintStyleData,
    copy_canonical_flags: bool,
) {
    if copy_canonical_flags {
        output.translate = Translate(style);
        output.rotate = Rotate(style);
        output.scale = Scale(style);
        output.transform = Transform(style);
        output.transform_origin = (output.translate.is_some()
            || output.rotate.is_some()
            || output.scale.is_some()
            || output.transform.is_some()
            || style.HasOffset())
        .then(|| TransformOrigin(style));
        output.perspective_origin = PerspectiveOrigin(style);
        output.perspective = style
            .HasPerspective()
            .then(|| f64::from(style.UsedPerspective()));
        output.preserves_3d = style.Preserves3D();
        output.has_3d_transform_operation = style.Has3DTransformOperation();
        if style.Transform().HasNonPerspective3DOperation() {
            if style.Transform().HasNonTrivial3DComponent() {
                output.has_nontrivial_3d_transform = true;
            } else {
                output.has_trivial_3d_transform = true;
            }
        }
        unsafe {
            let translate = style.Translate();
            output.has_3d_translate = !translate.is_null() && (*translate).Z() != 0.0;
            let rotate = style.Rotate();
            output.has_3d_rotate =
                !rotate.is_null() && ((*rotate).X() != 0.0 || (*rotate).Y() != 0.0);
            let scale = style.Scale();
            output.has_3d_scale = !scale.is_null() && (*scale).Z() != 1.0;
        }
        output.backface_visibility_hidden =
            style.BackfaceVisibility() == EBackfaceVisibility::kHidden;
    }
    if copy_canonical_flags {
        output.transform_reference_box =
            match style.UsedTransformBox(TransformBoxContext::kLayoutBox) {
                ETransformBox::kContentBox => PaintTransformReferenceBox::kContentBox,
                ETransformBox::kBorderBox => PaintTransformReferenceBox::kBorderBox,
                ETransformBox::kFillBox | ETransformBox::kStrokeBox | ETransformBox::kViewBox => {
                    panic!("layout transform-box must be normalized to content-box or border-box")
                }
            };
        output.subtree_will_change_contents = style.SubtreeWillChangeContents();
        output.will_change_transform = style.HasWillChangeAnyTransformProperty();
        output.will_change_transform_property =
            style.HasWillChangeProperty(CSSPropertyID::kTransform);
        output.will_change_perspective = style.HasWillChangeProperty(CSSPropertyID::kPerspective);
        output.will_change_transform_style =
            style.HasWillChangeProperty(CSSPropertyID::kTransformStyle);
        output.will_change_scale = style.HasWillChangeProperty(CSSPropertyID::kScale);
        output.will_change_rotate = style.HasWillChangeProperty(CSSPropertyID::kRotate);
        output.will_change_translate = style.HasWillChangeProperty(CSSPropertyID::kTranslate);
        output.will_change_opacity = style.HasWillChangeProperty(CSSPropertyID::kOpacity);
        output.will_change_filter = style.HasWillChangeProperty(CSSPropertyID::kFilter);
        output.will_change_backdrop_filter =
            style.HasWillChangeProperty(CSSPropertyID::kBackdropFilter);
        output.will_change_clip_path = style.HasWillChangeProperty(CSSPropertyID::kClipPath);
        output.will_change_mix_blend_mode =
            style.HasWillChangeProperty(CSSPropertyID::kMixBlendMode);
        output.will_change_mask = style.HasWillChangeProperty(CSSPropertyID::kMaskImage);
        output.will_change_other = [
            CSSPropertyID::kOffsetPath,
            CSSPropertyID::kOffsetPosition,
            CSSPropertyID::kTop,
            CSSPropertyID::kLeft,
            CSSPropertyID::kBottom,
            CSSPropertyID::kRight,
        ]
        .into_iter()
        .any(|id| style.HasWillChangeProperty(id));
        output.will_change_scroll_position = style.HasWillChangeScrollPosition();
        if std::env::var_os("RECHROM_DIAG_WILL_CHANGE").is_some()
            && (output.will_change_transform
                || output.will_change_opacity
                || output.will_change_filter
                || output.will_change_backdrop_filter
                || output.will_change_clip_path
                || output.will_change_mix_blend_mode
                || output.will_change_mask
                || output.will_change_other
                || output.will_change_scroll_position)
        {
            eprintln!(
                "will-change-export style={:p} transform={} transform-property={} translate={} rotate={} scale={} opacity={} filter={}",
                style,
                output.will_change_transform,
                output.will_change_transform_property,
                output.will_change_translate,
                output.will_change_rotate,
                output.will_change_scale,
                output.will_change_opacity,
                output.will_change_filter,
            );
        }

        output.active_transform_animation = style.HasCurrentTransformAnimation();
        output.active_scale_animation = style.HasCurrentScaleAnimation();
        output.active_rotate_animation = style.HasCurrentRotateAnimation();
        output.active_translate_animation = style.HasCurrentTranslateAnimation();
        let base_data = style.BaseData();
        let important = if base_data.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*base_data }.GetBaseImportantSet()
        };
        output.active_opacity_animation = style.HasCurrentOpacityAnimation()
            && (important.is_null() || !unsafe { &*important }.Has(CSSPropertyID::kOpacity));
        output.active_filter_animation = style.HasCurrentFilterAnimation();
        output.active_backdrop_filter_animation = style.HasCurrentBackdropFilterAnimation();
    }
}

#[inline(never)]
pub(crate) fn ExportGeometryStyle(
    style: &ComputedStyle,
) -> crate::internal::layout_input::PaintStyle {
    let mut data = PaintStyleData::default();
    ExportGeometryProperties(
        style,
        &mut data,
        !foundation::IsManagedLayoutAddress(style as *const _),
    );
    crate::internal::layout_input::PaintStyle::FromSource(style, data)
}

fn ExportForPaint(
    style: &ComputedStyle,
    image_resource_id: Option<u64>,
    copy_canonical_values: bool,
) -> PaintStyleData {
    let mut output = PaintStyleData::default();
    if copy_canonical_values {
        output.color = StyleColor(style, style.Color());
    }
    // cpp: core/layout/inline/caret_rect.cc:394-410
    if copy_canonical_values {
        output.caret_shape = match style.CaretShape() {
            ECaretShape::kAuto | ECaretShape::kBar => CaretShape::kBar,
            ECaretShape::kBlock => CaretShape::kBlock,
            ECaretShape::kUnderscore => CaretShape::kUnderscore,
        };
        output.caret_color = ColorValue(style.ResolvedCaretColor());
    }
    // Use the translated ComputedStyle owner instead of reinterpreting the
    // CSS enum at the Layout/Paint boundary. This preserves Chromium's runtime
    // default-quality policy as well as the pixelated/crisp-edges cap.
    if copy_canonical_values {
        output.image_interpolation_quality = style.GetInterpolationQuality();
        output.dynamic_range_limit = *style.GetDynamicRangeLimit();
    }
    if copy_canonical_values {
        output.text_fill_color = StyleColor(style, style.TextFillColor());
        output.text_stroke_color = StyleColor(style, style.TextStrokeColor());
        output.text_stroke_width = f64::from(style.TextStrokeWidth());
        output.print_color_adjust_economy = style.PrintColorAdjust() == EPrintColorAdjust::kEconomy;
    }
    if copy_canonical_values {
        output.color_scheme_flags_is_normal = style.ColorSchemeFlagsIsNormal();
        output.used_dark_color_scheme = matches!(
            style.UsedColorScheme(),
            layoutng_style::style::color_scheme::mojom::blink::ColorScheme::kDark
        );
        output.force_dark = style.ForceDark();
    }
    if copy_canonical_values {
        output.accent_color = (!style.AccentColor().IsAutoColor())
            .then(|| StyleColor(style, style.AccentColor().ToStyleColor()));
    }
    if copy_canonical_values {
        output.background_color = StyleColor(style, style.BackgroundColor());
    }
    if copy_canonical_values {
        output.background_images = Backgrounds(style);
        output.mask_images = Masks(style);
    }
    if copy_canonical_values {
        output.filters = Filters(style, style.Filter());
        AppendBoxReflection(style, &mut output.filters);
        output.backdrop_filters = Filters(style, style.BackdropFilter());
        output.backdrop_filter_has_reference = HasUnprojectedFilter(style.BackdropFilter());
    }
    if copy_canonical_values {
        output.background_clip = BottomBackgroundClip(style);
    }
    if copy_canonical_values {
        output.box_shadows = Shadows(style, style.BoxShadow());
        output.text_shadows = Shadows(style, style.TextShadow());
    }
    BorderRadii(style, &mut output);
    if copy_canonical_values {
        output.border_shape_unsupported = style.HasBorderShape();
        output.border_colors = [
            StyleColor(style, style.BorderTopColor()),
            StyleColor(style, style.BorderRightColor()),
            StyleColor(style, style.BorderBottomColor()),
            StyleColor(style, style.BorderLeftColor()),
        ];
    }
    // The common single-value case is retained on PaintStyle for legacy
    // callers. Layout expands list/repeat values per actual gap and stores the
    // used values on each exported ColumnRule.
    if copy_canonical_values {
        if style.ColumnRuleWidth().HasSingleValue() {
            output.column_rule_width = f64::from(style.ColumnRuleWidth().GetSingleValue());
        }
        if style.ColumnRuleStyle().HasSingleValue() {
            output.column_rule_style = BorderStyle(style.ColumnRuleStyle().GetSingleValue());
        }
        if style.ColumnRuleColor().HasSingleValue() {
            output.column_rule_color = StyleColor(style, &style.ColumnRuleColor().GetSingleValue());
        }
    }
    if copy_canonical_values {
        output.outline_color = StyleColor(style, style.OutlineColor());
        output.outline_width = if style.OutlineStyle() == EBorderStyle::kNone {
            0.0
        } else {
            *style.OutlineWidth() as f64
        };
        output.outline_offset = *style.OutlineOffset() as f64;
        output.outline_style = BorderStyle(style.OutlineStyle());
    }
    if copy_canonical_values {
        output.opacity = style.Opacity();
        output.visible = style.Visibility() == EVisibility::kVisible;
        output.forced_color_adjust_auto = style.ForcedColorAdjust() == EForcedColorAdjust::kAuto;
        output.forced_background_color = StyleColor(style, style.InternalForcedBackgroundColor());
        output.pointer_events_none = style.PointerEvents() == EPointerEvents::kNone;
        output.pointer_events_bounding_box =
            style.UsedPointerEvents() == EPointerEvents::kBoundingBox;
        output.user_select_none = style.UsedUserSelect() == EUserSelect::kNone;
    }
    if copy_canonical_values {
        output.cursor = style.Cursor();
        output.z_index = (!style.HasAutoZIndex()).then(|| style.ZIndex());
        output.blend_mode = BlendValue(style.GetBlendMode());
        output.isolate_blending = style.Isolation() == EIsolation::kIsolate;
    }
    ExportGeometryProperties(style, &mut output, copy_canonical_values);
    output.clip_path = clip_path_export::ExportClipPath(style);
    output.svg_fill = SvgColor(
        style,
        style.FillPaint(),
        style.InternalVisitedFillPaint(),
        style.FillOpacity(),
    );
    output.svg_stroke = SvgColor(
        style,
        style.StrokePaint(),
        style.InternalVisitedStrokePaint(),
        style.StrokeOpacity(),
    );
    output.svg_fill_reference = style
        .FillPaint()
        .HasUrl()
        .then(|| style.FillPaint().GetUrl().Utf8());
    output.svg_stroke_reference = style
        .StrokePaint()
        .HasUrl()
        .then(|| style.StrokePaint().GetUrl().Utf8());
    output.svg_fill_current_color = style.FillPaint().HasCurrentColor();
    output.svg_stroke_current_color = style.StrokePaint().HasCurrentColor();
    SvgStrokeStyle(style, &mut output);
    if copy_canonical_values {
        output.respect_image_orientation = style.ImageOrientation()
            == foundation::RespectImageOrientationEnum::kRespectImageOrientation;
    }
    if copy_canonical_values {
        output.object_fit = ObjectFitValue(style.GetObjectFit());
        let object_position = style.ObjectPosition();
        let x = LengthParts(object_position.X());
        let y = LengthParts(object_position.Y());
        output.object_position = Offset {
            x: x.1 / 100.0,
            y: y.1 / 100.0,
        };
        output.object_position_offset = Offset { x: x.0, y: y.0 };
    }
    output.image_resource_id = image_resource_id;
    output
}

pub fn Export(style: &ComputedStyle, image_resource_id: Option<u64>) -> PaintStyleData {
    ExportForPaint(style, image_resource_id, true)
}

/// Retain the canonical Chromium ComputedStyle across Layout -> Paint. The
/// accompanying data contains only values still awaiting direct-source
/// migration plus fragment/resource results.
#[inline(never)]
pub(crate) fn ExportStyle(
    style: &ComputedStyle,
    image_resource_id: Option<u64>,
) -> crate::internal::layout_input::PaintStyle {
    let copy_canonical_values = !foundation::IsManagedLayoutAddress(style as *const _);
    crate::internal::layout_input::PaintStyle::FromSource(
        style,
        ExportForPaint(style, image_resource_id, copy_canonical_values),
    )
}

pub(crate) fn ExportDetachedStyle(
    style: &ComputedStyle,
    image_resource_id: Option<u64>,
) -> crate::internal::layout_input::PaintStyle {
    crate::internal::layout_input::PaintStyle::FromData(Export(style, image_resource_id))
}

/// Preserve document-resolved SVG resources that cannot live in Layout's
/// source ComputedStyle object graph. URI identity still comes from `style`;
/// only the immutable server snapshot is carried from StyleEngine's existing
/// renderer-neutral projection.
#[inline(never)]
pub(crate) fn ExportStyleWithResourceBindings(
    style: &ComputedStyle,
    image_resource_id: Option<u64>,
    resources: &crate::style_projection::StyleResourceBindings<'_>,
    clip_resources: &crate::internal::paint_input::SvgClipResourceBinding,
) -> crate::internal::layout_input::PaintStyle {
    let mut output = ExportStyle(style, image_resource_id);
    output.svg_mask_resource = resources.svg_mask_resource.clone();
    output.svg_mask_bindings = MaskReferences(style);
    for reference in &mut output.svg_mask_bindings {
        let entry = (reference.scope.document_id == clip_resources.snapshot.version.document_id)
            .then(|| {
                clip_resources.snapshot.entries.iter().find(|entry| {
                    entry.scope_id == reference.scope.tree_scope_id && entry.id == reference.id
                })
            })
            .flatten();
        if let Some(graph) = entry.and_then(|entry| entry.mask_graph.as_ref()) {
            reference.graph = Some(Arc::clone(graph));
            reference.target_id = entry.map(|entry| entry.target_node_id);
        }
    }
    output.svg_fill_server = resources.svg_fill_server.clone();
    output.svg_stroke_server = resources.svg_stroke_server.clone();
    output.svg_markers = resources.svg_markers.to_vec();
    output.replaced_image_shader = resources.replaced_image_shader.clone();
    output.svg_filter_graphs = Arc::clone(resources.svg_filter_graphs);
    if let Some(clip) = output.clip_path.as_mut() {
        if let Some(url) = clip.reference_url.as_deref() {
            if let Some(id) = url.strip_prefix('#') {
                let scope = clip.reference_scope.unwrap_or(clip_resources.scope);
                let entry =
                    (scope.document_id == clip_resources.snapshot.version.document_id)
                        .then(|| {
                            clip_resources.snapshot.entries.iter().find(|entry| {
                                entry.scope_id == scope.tree_scope_id && entry.id == id
                            })
                        })
                        .flatten();
                if let Some(graph) = entry.and_then(|entry| entry.graph.as_ref()) {
                    clip.reference_target_id = entry.map(|entry| entry.target_node_id);
                    clip.reference_graph = Some(Arc::clone(graph));
                    clip.reference_error = graph.unsupported.clone();
                } else {
                    // cpp: ClipPathClipper::PathBasedClipInternal. A missing
                    // or wrong-kind resource supplies no clip. A present,
                    // empty graph above remains a real empty clip instead.
                    output.clip_path = None;
                }
            } else {
                // External fetch/loading ownership is not represented by this
                // local resource snapshot. Keep it explicitly unsupported.
                clip.reference_error = Some("external clip-path IRI".into());
            }
        }
    }
    let fill_paint = output.svg_fill;
    let stroke_paint = output.svg_stroke;
    let graphs = Arc::clone(resources.svg_filter_graphs);
    // Both CSS filter properties use the same document-resolved SVG resource
    // graph.  Keeping only the foreground projection turns a valid local
    // backdrop-filter url() into an unprojected marker at the Paint boundary,
    // even though Style already produced the canonical graph.
    let project_filters = |filters: &mut Vec<PaintFilterOperation>| {
        for filter in filters {
            if filter.r#type != PaintFilterType::kReference {
                continue;
            }
            let Some(id) = filter
                .reference_url
                .as_deref()
                .and_then(|url| url.strip_prefix('#'))
            else {
                continue;
            };
            if let Some(graph) = graphs.iter().find(|graph| graph.id == id) {
                let mut graph = graph.clone();
                graph.fill_paint = fill_paint;
                graph.stroke_paint = stroke_paint;
                filter.reference_error = graph.unsupported.clone();
                filter.reference_graph = Some(Arc::new(graph));
            }
        }
    };
    if output.Source().is_none() {
        project_filters(&mut output.filters);
        project_filters(&mut output.backdrop_filters);
        output.backdrop_filter_has_reference = output.backdrop_filters.iter().any(|filter| {
            filter.r#type == PaintFilterType::kReference && filter.reference_graph.is_none()
        });
    }
    output
}

#[inline(never)]
pub(crate) fn ExportStyleWithInputResources(
    style: &ComputedStyle,
    resources: &crate::internal::layout_input::StyleResourceData,
) -> crate::internal::layout_input::PaintStyle {
    let bindings = crate::style_projection::StyleResourceBindings::FromData(resources);
    ExportStyleWithResourceBindings(
        style,
        resources.image_resource_id,
        &bindings,
        &resources.svg_clip_resources,
    )
}

#[inline(never)]
pub(crate) fn ExportStyleWithProjectedSvg(
    style: &ComputedStyle,
    image_resource_id: Option<u64>,
    projected: &PaintStyleData,
    clip_resources: &crate::internal::paint_input::SvgClipResourceBinding,
) -> crate::internal::layout_input::PaintStyle {
    let resources =
        crate::style_projection::StyleResourceBindings::FromPaint(projected, clip_resources);
    ExportStyleWithResourceBindings(style, image_resource_id, &resources, clip_resources)
}

/// Motion-path is resolved against the actual containing LayoutBox, so its
/// final matrix cannot be reconstructed from ComputedStyle alone at the
/// immutable Layout -> Paint boundary. Preserve only UpdateOffset's matrix;
/// independent and legacy transform properties keep their own slots.
#[inline(never)]
pub(crate) fn SetResolvedOffsetTransform(
    output: &mut crate::internal::layout_input::PaintStyle,
    matrix: foundation::gfx::Transform,
) {
    let mut transform = PaintTransform::default();
    PushMatrix(&mut transform, matrix);
    output.offset_transform = Some(transform);
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{color::ColorSpace, gfx, Member};
    use layoutng_style::style::computed_style::ComputedStyleBuilder;
    use layoutng_style::style::filter_operation::{
        BasicColorMatrixFilterOperation, BlurFilterOperation, DropShadowFilterOperation,
        FilterOperation, OperationType as FilterOperationType,
    };
    use layoutng_style::style::filter_operations::FilterOperations;
    use layoutng_style::style::generated_gradient_image::{
        CSSImageGeneratorValue, CSSLinearGradientValue, GradientStop, LinearGradientData,
        StyleGeneratedImage,
    };
    use layoutng_style::style::style_image::StyleImage;

    fn near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.005,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn caret_shape_crosses_the_style_paint_boundary_without_reinterpretation() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        for (source, expected) in [
            (ECaretShape::kAuto, CaretShape::kBar),
            (ECaretShape::kBar, CaretShape::kBar),
            (ECaretShape::kBlock, CaretShape::kBlock),
            (ECaretShape::kUnderscore, CaretShape::kUnderscore),
        ] {
            let mut builder = ComputedStyleBuilder::from_style(initial);
            builder.SetCaretShape(source);
            let style = unsafe { &*builder.TakeStyle() };
            assert_eq!(Export(style, None).caret_shape, expected);
        }
    }

    #[test]
    fn wide_gamut_colors_convert_to_extended_srgb_at_paint_boundary() {
        let p3_red = ColorValue(foundation::Color::FromColorSpace(
            ColorSpace::kDisplayP3,
            Some(1.0),
            Some(0.0),
            Some(0.0),
            Some(0.4),
        ));
        // Chromium's conversion deliberately preserves extended-sRGB values;
        // the destination backend performs its normal gamut clipping.
        assert!(p3_red.red > 1.0);
        assert!(p3_red.green < 0.0);
        assert!(p3_red.blue < 0.0);
        near(p3_red.alpha, 0.4);

        let lab_white = ColorValue(foundation::Color::FromColorSpace(
            ColorSpace::kLab,
            Some(100.0),
            Some(0.0),
            Some(0.0),
            Some(1.0),
        ));
        near(lab_white.red, 1.0);
        near(lab_white.green, 1.0);
        near(lab_white.blue, 1.0);

        let oklab_gray = ColorValue(foundation::Color::FromColorSpace(
            ColorSpace::kOklab,
            Some(0.5),
            Some(0.0),
            Some(0.0),
            Some(1.0),
        ));
        near(oklab_gray.red, 0.3886);
        near(oklab_gray.green, 0.3886);
        near(oklab_gray.blue, 0.3886);
    }

    #[test]
    fn percent_radii_and_zoomed_shadows_cross_only_at_paint_export() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        let radius = foundation::LengthSize::new(&Length::Percent(50.0), &Length::Percent(50.0));
        builder.SetBorderTopLeftRadius(&radius);
        builder.SetBorderTopRightRadius(&radius);
        builder.SetBorderBottomRightRadius(&radius);
        builder.SetBorderBottomLeftRadius(&radius);
        let style = unsafe { &*builder.TakeStyle() };
        let paint = Export(style, None);
        assert_eq!(paint.border_radii_percentages.top_left.x, 50.0);
        assert_eq!(paint.border_radii_percentages.bottom_right.y, 50.0);
        assert!(paint.border_radii.is_some());
    }

    #[test]
    fn independent_transforms_keep_distinct_paint_property_payloads() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        builder.SetTranslate(Member::from_ptr(foundation::MakeGarbageCollected(
            foundation::TranslateTransformOperation::new(
                Length::Percent(25.0),
                Length::Fixed(8.0),
                0.0,
                TransformOperationType::kTranslate,
            ),
        )));
        builder.SetRotate(Member::from_ptr(foundation::MakeGarbageCollected(
            foundation::RotateTransformOperation::new(30.0, TransformOperationType::kRotate),
        )));
        builder.SetScale(Member::from_ptr(foundation::MakeGarbageCollected(
            foundation::ScaleTransformOperation::new(2.0, 3.0, 1.0, TransformOperationType::kScale),
        )));
        let mut legacy = foundation::TransformOperations::default();
        legacy.PushTranslate(Length::Fixed(4.0), Length::Fixed(5.0));
        builder.SetTransform(&legacy);
        let style = unsafe { &*builder.TakeStyle() };

        let paint = Export(style, None);
        assert_eq!(paint.translate.as_ref().unwrap().operations.len(), 1);
        assert_eq!(
            paint.rotate.as_ref().unwrap().operations[0].kind,
            PaintTransformOperationKind::kRotate
        );
        assert_eq!(
            paint.scale.as_ref().unwrap().operations[0].kind,
            PaintTransformOperationKind::kScale
        );
        assert_eq!(paint.transform.as_ref().unwrap().operations.len(), 1);
        assert_eq!(
            paint.transform.as_ref().unwrap().operations[0].pixels.x,
            4.0
        );
    }

    #[test]
    fn filter_chain_crosses_paint_boundary_with_ordered_payloads() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        let mut filters = FilterOperations::new();
        filters.OperationsMut().push(Member::from_ptr(
            foundation::MakeGarbageCollected(BasicColorMatrixFilterOperation::new(
                0.35,
                FilterOperationType::kGrayscale,
            ))
            .cast::<FilterOperation>(),
        ));
        filters.OperationsMut().push(Member::from_ptr(
            foundation::MakeGarbageCollected(BlurFilterOperation::new(&Length::Fixed(4.5)))
                .cast::<FilterOperation>(),
        ));
        filters.OperationsMut().push(Member::from_ptr(
            foundation::MakeGarbageCollected(DropShadowFilterOperation::new(
                layoutng_style::style::shadow_data::ShadowData::new(
                    gfx::Vector2dF::new(2.0, -3.0),
                    8.0,
                    0.0,
                    ShadowStyle::kNormal,
                    layoutng_style::css::style_color::StyleColor::from_color(
                        foundation::Color::FromRGBA(10, 20, 30, 128),
                    ),
                    1.0,
                ),
            ))
            .cast::<FilterOperation>(),
        ));
        filters.OperationsMut().push(Member::from_ptr(
            foundation::MakeGarbageCollected(DropShadowFilterOperation::new(
                layoutng_style::style::shadow_data::ShadowData::new(
                    gfx::Vector2dF::new(1.0, 1.0),
                    2.0,
                    0.0,
                    ShadowStyle::kNormal,
                    layoutng_style::css::style_color::StyleColor::CurrentColor(),
                    1.0,
                ),
            ))
            .cast::<FilterOperation>(),
        ));
        builder.SetFilterOwned(filters);
        let style = unsafe { &*builder.TakeStyle() };

        let paint = Export(style, None);
        assert_eq!(paint.filters.len(), 4);
        assert_eq!(paint.filters[0].r#type, PaintFilterType::kGrayscale);
        assert_eq!(paint.filters[0].amount, 0.35);
        assert_eq!(paint.filters[1].r#type, PaintFilterType::kBlur);
        assert_eq!(paint.filters[1].amount, 4.5);
        assert_eq!(paint.filters[2].r#type, PaintFilterType::kDropShadow);
        assert_eq!(paint.filters[2].offset, Offset { x: 2.0, y: -3.0 });
        assert_eq!(paint.filters[2].blur_radius, 8.0);
        assert!((paint.filters[2].color.alpha - 128.0 / 255.0).abs() < 1e-6);
        assert!(!paint.filters[2].uses_current_color);
        assert!(paint.filters[3].uses_current_color);
    }

    #[test]
    fn generated_background_crosses_only_at_paint_export() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        let data = LinearGradientData {
            start: [0.0, 0.0],
            end: [1.0, 1.0],
            angle_degrees: None,
            corner: None,
            repeating: false,
            color_interpolation_space: foundation::ColorSpace::kNone,
            hue_interpolation_method: foundation::HueInterpolationMethod::kShorter,
            stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: layoutng_style::css::style_color::StyleColor::from_color(
                        foundation::Color::FromRGBA(0, 180, 255, 255),
                    ),
                    offset_length: 0.0,
                    is_hint: false,
                },
                GradientStop {
                    offset: 1.0,
                    color: layoutng_style::css::style_color::StyleColor::from_color(
                        foundation::Color::FromRGBA(255, 220, 0, 255),
                    ),
                    offset_length: 0.0,
                    is_hint: false,
                },
            ],
        };
        let generator =
            foundation::MakeGarbageCollected(CSSLinearGradientValue::FromResolved(data));
        let image = foundation::MakeGarbageCollected(StyleGeneratedImage::new(
            generator.cast::<CSSImageGeneratorValue>(),
            layoutng_style::css::css_to_length_conversion_data::ContainerSizes::default(),
        ))
        .cast::<StyleImage>();
        builder.AccessBackgroundLayers().SetImage(image);
        let style = unsafe { &*builder.TakeStyle() };

        let paint = Export(style, None);
        assert_eq!(paint.background_images.len(), 1);
        let shader = paint.background_images[0]
            .shader
            .as_ref()
            .expect("source generated image must export a shader");
        assert_eq!(shader.kind, PaintShaderKind::kLinearGradient);
        assert_eq!(shader.stops.len(), 2);
    }

    #[test]
    fn generated_mask_crosses_paint_boundary_with_mask_properties() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        let data = LinearGradientData {
            start: [0.0, 0.0],
            end: [0.0, 1.0],
            angle_degrees: Some(75.0),
            corner: None,
            repeating: false,
            color_interpolation_space: foundation::ColorSpace::kNone,
            hue_interpolation_method: foundation::HueInterpolationMethod::kShorter,
            stops: vec![
                GradientStop {
                    offset: 0.3,
                    color: layoutng_style::css::style_color::StyleColor::from_color(
                        foundation::Color::FromRGBA(0, 0, 0, 255),
                    ),
                    offset_length: 0.0,
                    is_hint: false,
                },
                GradientStop {
                    offset: 0.8,
                    color: layoutng_style::css::style_color::StyleColor::from_color(
                        foundation::Color::FromRGBA(0, 0, 0, 166),
                    ),
                    offset_length: 0.0,
                    is_hint: false,
                },
            ],
        };
        let generator =
            foundation::MakeGarbageCollected(CSSLinearGradientValue::FromResolved(data));
        let image = foundation::MakeGarbageCollected(StyleGeneratedImage::new(
            generator.cast::<CSSImageGeneratorValue>(),
            layoutng_style::css::css_to_length_conversion_data::ContainerSizes::default(),
        ))
        .cast::<StyleImage>();
        let mask = builder.AccessMaskLayers();
        mask.SetImage(image);
        mask.SetMaskMode(EFillMaskMode::kLuminance);
        mask.SetCompositingOperator(CompositingOperator::kIntersect);
        let style = unsafe { &*builder.TakeStyle() };

        let paint = Export(style, None);
        assert_eq!(paint.mask_images.len(), 1);
        assert_eq!(paint.mask_images[0].mode, PaintMaskMode::kLuminance);
        assert_eq!(
            paint.mask_images[0].composite,
            PaintMaskComposite::kIntersect
        );
        let shader = paint.mask_images[0]
            .image
            .shader
            .as_ref()
            .expect("source generated mask must export a shader");
        assert_eq!(shader.kind, PaintShaderKind::kLinearGradient);
        assert_eq!(shader.linear_angle, Some(75.0));
        assert_eq!(shader.stops.len(), 2);
    }

    #[test]
    fn svg_visited_fill_and_stroke_replace_rgb_but_keep_unvisited_alpha() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        builder.SetInsideLink(foundation::EInsideLink::kInsideVisitedLink);
        builder.SetFillPaintOwned(SVGPaint::from_color(foundation::Color::FromRGBA(
            255, 0, 0, 64,
        )));
        builder.SetInternalVisitedFillPaintOwned(SVGPaint::from_color(
            foundation::Color::FromRGBA(0, 255, 0, 230),
        ));
        builder.SetStrokePaintOwned(SVGPaint::from_color(foundation::Color::FromRGBA(
            0, 0, 255, 128,
        )));
        builder.SetInternalVisitedStrokePaintOwned(SVGPaint::from_color(
            foundation::Color::FromRGBA(255, 255, 0, 26),
        ));
        builder.SetFillOpacity(0.5);
        builder.SetStrokeOpacity(0.25);
        let style = unsafe { &*builder.TakeStyle() };

        let paint = Export(style, None);
        let fill = paint.svg_fill.expect("visited fill");
        near(fill.red, 0.0);
        near(fill.green, 1.0);
        near(fill.blue, 0.0);
        near(fill.alpha, 64.0 / 255.0 * 0.5);
        let stroke = paint.svg_stroke.expect("visited stroke");
        near(stroke.red, 1.0);
        near(stroke.green, 1.0);
        near(stroke.blue, 0.0);
        near(stroke.alpha, 128.0 / 255.0 * 0.25);
    }

    #[test]
    fn svg_stroke_and_geometry_properties_cross_paint_boundary() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        builder.SetStrokeWidthOwned(layoutng_style::style::unzoomed_length::UnzoomedLength::new(
            &Length::Fixed(1.5),
        ));
        builder.SetStrokeDashOffset(&Length::Fixed(2.0));
        builder.SetStrokeMiterLimit(7.0);
        builder.SetCapStyle(LineCap::kRoundCap);
        builder.SetJoinStyle(LineJoin::kBevelJoin);
        builder.SetFillRule(WindRule::RULE_EVENODD);
        builder.SetClipRule(WindRule::RULE_EVENODD);
        builder.SetVectorEffect(EVectorEffect::kNonScalingStroke);
        builder.SetShapeRendering(EShapeRendering::kCrispedges);
        builder.SetPaintOrder(EPaintOrder::kPaintOrderStrokeMarkersFill);
        let style = unsafe { &*builder.TakeStyle() };

        let paint = Export(style, None);
        assert_eq!(paint.svg_stroke_width, 1.5);
        assert_eq!(paint.svg_stroke_dash_offset, 2.0);
        assert_eq!(paint.svg_stroke_miter_limit, 7.0);
        assert_eq!(paint.svg_stroke_line_cap, SvgStrokeLineCap::kRound);
        assert_eq!(paint.svg_stroke_line_join, SvgStrokeLineJoin::kBevel);
        assert!(paint.svg_fill_even_odd);
        assert!(paint.svg_clip_even_odd);
        assert!(paint.svg_non_scaling_stroke);
        assert!(!paint.svg_shape_antialias);
        assert_eq!(
            paint.svg_paint_order,
            [
                SvgPaintComponent::kStroke,
                SvgPaintComponent::kMarkers,
                SvgPaintComponent::kFill,
            ]
        );
    }

    #[test]
    fn dynamic_range_limit_crosses_the_layout_paint_boundary() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        let limit = foundation::DynamicRangeLimit::from_mix(0.25, 0.5);
        builder.SetDynamicRangeLimit(&limit);
        let style = unsafe { &*builder.TakeStyle() };

        let paint = Export(style, None);
        assert_eq!(paint.dynamic_range_limit, limit);
    }
}
