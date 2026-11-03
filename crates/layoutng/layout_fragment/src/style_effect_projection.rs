//! Chromium paint-grouping and transform style projection.
//!
//! Included by `boundary::input` at the reciprocal assembly boundary.
//! Filter lists and transform-operation lists scale with author input, so the
//! two entries stay out of line as fragment-qualified sampling frames.

use super::*;
use crate::internal::paint_input::{
    PaintCanvasFilter, PaintComponentTransferType, PaintConvolveEdgeMode, PaintTurbulenceType,
};
use layoutng_style::style::filter_operation::{
    ColorMatrixFilterOperation, ComponentTransferFilterOperation, ComponentTransferFunction,
    ComponentTransferType, ConvolveMatrixEdgeMode, ConvolveMatrixFilterOperation,
    TurbulenceFilterOperation, TurbulenceType,
};

// Copy actual resolved CSS paint values into the source style. The source
// LayerTypeRequired then reads the same style fields as Chromium; a sidecar
// painting value alone does not establish source stacking-context semantics.
// cpp: core/style/computed_style.h:3345-3348
// cpp: core/style/computed_style.cc:2913-2949
#[inline(never)]
pub(super) fn ApplyStylePaintGrouping(builder: &mut ComputedStyleBuilder, input: &ComputedStyle) {
    if let Some(clip) = input.css_clip {
        let side = |value: Option<f64>| {
            value
                .map(Length::Fixed)
                .unwrap_or_else(|| Length::Auto().clone())
        };
        builder.SetClip(&LengthBox::new(
            side(clip.top),
            side(clip.right),
            side(clip.bottom),
            side(clip.left),
        ));
    } else {
        builder.SetHasAutoClip();
    }
    builder.SetClipPath(
        input
            .paint
            .clip_path
            .as_ref()
            .map(clip_path_projection::ProjectClipPathOperation),
    );
    if !input.paint.opacity.is_finite() {
        Invalid("opacity");
    }
    builder.SetOpacity(input.paint.opacity);
    builder.SetIsolation(if input.paint.isolate_blending {
        EIsolation::kIsolate
    } else {
        EIsolation::kAuto
    });
    builder.SetBlendMode(match input.paint.blend_mode {
        PaintBlendMode::kNormal => BlendMode::kNormal,
        PaintBlendMode::kMultiply => BlendMode::kMultiply,
        PaintBlendMode::kScreen => BlendMode::kScreen,
        PaintBlendMode::kOverlay => BlendMode::kOverlay,
        PaintBlendMode::kDarken => BlendMode::kDarken,
        PaintBlendMode::kLighten => BlendMode::kLighten,
        PaintBlendMode::kColorDodge => BlendMode::kColorDodge,
        PaintBlendMode::kColorBurn => BlendMode::kColorBurn,
        PaintBlendMode::kHardLight => BlendMode::kHardLight,
        PaintBlendMode::kSoftLight => BlendMode::kSoftLight,
        PaintBlendMode::kDifference => BlendMode::kDifference,
        PaintBlendMode::kExclusion => BlendMode::kExclusion,
        PaintBlendMode::kHue => BlendMode::kHue,
        PaintBlendMode::kSaturation => BlendMode::kSaturation,
        PaintBlendMode::kColor => BlendMode::kColor,
        PaintBlendMode::kLuminosity => BlendMode::kLuminosity,
        PaintBlendMode::kPlusLighter => BlendMode::kPlusLighter,
    });
    // Preserve each supported CSS filter's real source class/payload.
    // cpp: core/css/resolver/filter_operation_resolver.cc:307-351
    if input.paint.backdrop_filter_has_reference {
        Invalid("backdrop reference filter graph");
    }
    for (projected_filters, is_backdrop) in [
        (&input.paint.filters, false),
        (&input.paint.backdrop_filters, true),
    ] {
        if projected_filters.is_empty() {
            continue;
        }
        let mut filters = FilterOperations::new();
        for filter in projected_filters {
            let operation: *mut FilterOperation = match filter.r#type {
                PaintFilterType::kReference => Invalid("reference filter graph"),
                PaintFilterType::kGrayscale
                | PaintFilterType::kSepia
                | PaintFilterType::kSaturate
                | PaintFilterType::kHueRotate
                | PaintFilterType::kLuminanceToAlpha => {
                    let type_ = match filter.r#type {
                        PaintFilterType::kGrayscale => OperationType::kGrayscale,
                        PaintFilterType::kSepia => OperationType::kSepia,
                        PaintFilterType::kSaturate => OperationType::kSaturate,
                        PaintFilterType::kHueRotate => OperationType::kHueRotate,
                        _ => OperationType::kLuminanceToAlpha,
                    };
                    MakeGarbageCollected(BasicColorMatrixFilterOperation::new(
                        Number(
                            filter.amount,
                            "filter amount",
                            filter.r#type != PaintFilterType::kHueRotate,
                        ),
                        type_,
                    ))
                    .cast()
                }
                PaintFilterType::kColorMatrix => {
                    MakeGarbageCollected(ColorMatrixFilterOperation::new(
                        filter
                            .color_matrix
                            .iter()
                            .map(|value| Number(*value, "filter color matrix", false) as f32)
                            .collect(),
                    ))
                    .cast()
                }
                PaintFilterType::kInvert
                | PaintFilterType::kOpacity
                | PaintFilterType::kBrightness
                | PaintFilterType::kContrast => {
                    let type_ = match filter.r#type {
                        PaintFilterType::kInvert => OperationType::kInvert,
                        PaintFilterType::kOpacity => OperationType::kOpacity,
                        PaintFilterType::kBrightness => OperationType::kBrightness,
                        _ => OperationType::kContrast,
                    };
                    MakeGarbageCollected(BasicComponentTransferFilterOperation::new(
                        Number(filter.amount, "filter amount", true),
                        type_,
                    ))
                    .cast()
                }
                PaintFilterType::kBlur => {
                    let deviation = Length::Fixed(Number(filter.amount, "filter blur", true));
                    MakeGarbageCollected(BlurFilterOperation::new(&deviation)).cast()
                }
                PaintFilterType::kDropShadow => {
                    let color = foundation::Color::FromRGBAFloat(
                        filter.color.red,
                        filter.color.green,
                        filter.color.blue,
                        filter.color.alpha,
                    );
                    let shadow = ShadowData::new(
                        gfx::Vector2dF::new(
                            Number(filter.offset.x, "filter shadow x", false) as f32,
                            Number(filter.offset.y, "filter shadow y", false) as f32,
                        ),
                        Number(filter.blur_radius, "filter shadow blur", true) as f32,
                        0.0,
                        ShadowStyle::kNormal,
                        StyleColor::from_color(color),
                        1.0,
                    );
                    MakeGarbageCollected(DropShadowFilterOperation::new(shadow)).cast()
                }
                PaintFilterType::kConvolveMatrix => {
                    let Some(PaintCanvasFilter::kConvolveMatrix {
                        order_x,
                        order_y,
                        divisor,
                        bias,
                        target_x,
                        target_y,
                        edge_mode,
                        preserve_alpha,
                        kernel,
                    }) = filter.canvas_filter.as_ref()
                    else {
                        Invalid("missing Canvas convolution payload")
                    };
                    MakeGarbageCollected(ConvolveMatrixFilterOperation::new(
                        foundation::gfx::Size::new(*order_x, *order_y),
                        Number(*divisor, "convolution divisor", false) as f32,
                        Number(*bias, "convolution bias", false) as f32,
                        foundation::gfx::Point::new(*target_x, *target_y),
                        match edge_mode {
                            PaintConvolveEdgeMode::kUnknown => ConvolveMatrixEdgeMode::kUnknown,
                            PaintConvolveEdgeMode::kDuplicate => ConvolveMatrixEdgeMode::kDuplicate,
                            PaintConvolveEdgeMode::kWrap => ConvolveMatrixEdgeMode::kWrap,
                            PaintConvolveEdgeMode::kNone => ConvolveMatrixEdgeMode::kNone,
                        },
                        *preserve_alpha,
                        kernel
                            .iter()
                            .map(|value| Number(*value, "convolution kernel", false) as f32)
                            .collect(),
                    ))
                    .cast()
                }
                PaintFilterType::kComponentTransfer => {
                    let Some(PaintCanvasFilter::kComponentTransfer { functions }) =
                        filter.canvas_filter.as_ref()
                    else {
                        Invalid("missing Canvas component-transfer payload")
                    };
                    MakeGarbageCollected(ComponentTransferFilterOperation::new(
                        functions.clone().map(|function| ComponentTransferFunction {
                            r#type: match function.r#type {
                                PaintComponentTransferType::kUnknown => {
                                    ComponentTransferType::kUnknown
                                }
                                PaintComponentTransferType::kIdentity => {
                                    ComponentTransferType::kIdentity
                                }
                                PaintComponentTransferType::kTable => ComponentTransferType::kTable,
                                PaintComponentTransferType::kDiscrete => {
                                    ComponentTransferType::kDiscrete
                                }
                                PaintComponentTransferType::kLinear => {
                                    ComponentTransferType::kLinear
                                }
                                PaintComponentTransferType::kGamma => ComponentTransferType::kGamma,
                            },
                            slope: Number(function.slope, "component-transfer slope", false) as f32,
                            intercept: Number(
                                function.intercept,
                                "component-transfer intercept",
                                false,
                            ) as f32,
                            amplitude: Number(
                                function.amplitude,
                                "component-transfer amplitude",
                                false,
                            ) as f32,
                            exponent: Number(
                                function.exponent,
                                "component-transfer exponent",
                                false,
                            ) as f32,
                            offset: Number(function.offset, "component-transfer offset", false)
                                as f32,
                            table_values: function
                                .table_values
                                .into_iter()
                                .map(|value| {
                                    Number(value, "component-transfer table", false) as f32
                                })
                                .collect(),
                        }),
                    ))
                    .cast()
                }
                PaintFilterType::kTurbulence => {
                    let Some(PaintCanvasFilter::kTurbulence {
                        kind,
                        base_x,
                        base_y,
                        octaves,
                        seed,
                        stitch,
                    }) = filter.canvas_filter.as_ref()
                    else {
                        Invalid("missing Canvas turbulence payload")
                    };
                    MakeGarbageCollected(TurbulenceFilterOperation::new(
                        match kind {
                            PaintTurbulenceType::kUnknown => TurbulenceType::kUnknown,
                            PaintTurbulenceType::kFractalNoise => TurbulenceType::kFractalNoise,
                            PaintTurbulenceType::kTurbulence => TurbulenceType::kTurbulence,
                        },
                        Number(*base_x, "turbulence base x", false) as f32,
                        Number(*base_y, "turbulence base y", false) as f32,
                        *octaves,
                        Number(*seed, "turbulence seed", false) as f32,
                        *stitch,
                    ))
                    .cast()
                }
                PaintFilterType::kBoxReflect => Invalid("box-reflect paint effect"),
            };
            filters
                .OperationsMut()
                .push_back(Member::from_ptr(operation));
        }
        if is_backdrop {
            builder.SetBackdropFilterOwned(filters);
        } else {
            builder.SetFilterOwned(filters);
        }
    }
    if let Some(index) = input.paint.z_index {
        builder.SetZIndex(index);
    }
    builder.SetSubtreeWillChangeContents(input.paint.subtree_will_change_contents);
    if input.paint.will_change_other {
        Invalid("unresolved will-change property identity");
    }
    let exact_will_change = [
        (
            input.paint.will_change_backdrop_filter,
            CSSPropertyID::kBackdropFilter,
            "backdrop-filter",
        ),
        (
            input.paint.will_change_clip_path,
            CSSPropertyID::kClipPath,
            "clip-path",
        ),
        (
            input.paint.will_change_filter,
            CSSPropertyID::kFilter,
            "filter",
        ),
        (
            input.paint.will_change_mask,
            CSSPropertyID::kMaskImage,
            "mask-image",
        ),
        (
            input.paint.will_change_mix_blend_mode,
            CSSPropertyID::kMixBlendMode,
            "mix-blend-mode",
        ),
        (
            input.paint.will_change_opacity,
            CSSPropertyID::kOpacity,
            "opacity",
        ),
        (
            input.paint.will_change_rotate,
            CSSPropertyID::kRotate,
            "rotate",
        ),
        (
            input.paint.will_change_scale,
            CSSPropertyID::kScale,
            "scale",
        ),
        (
            input.paint.will_change_translate,
            CSSPropertyID::kTranslate,
            "translate",
        ),
        (
            input.paint.will_change_transform_property,
            CSSPropertyID::kTransform,
            "transform",
        ),
        (
            input.paint.will_change_perspective,
            CSSPropertyID::kPerspective,
            "perspective",
        ),
        (
            input.paint.will_change_transform_style,
            CSSPropertyID::kTransformStyle,
            "transform-style",
        ),
    ];
    let ids: Vec<_> = exact_will_change
        .iter()
        .filter_map(|(enabled, id, _)| enabled.then_some(*id))
        .collect();
    if !ids.is_empty() || input.paint.will_change_scroll_position {
        let mut values = Vector::new();
        for (_, id, name) in exact_will_change.iter().filter(|(enabled, _, _)| *enabled) {
            let _ = id;
            values.push(AtomicString::from_utf16(
                &name.encode_utf16().collect::<Vec<_>>(),
            ));
        }
        let ids = CSSBitset::FromList(&ids);
        let has_transform_property = input.paint.will_change_transform_property
            || input.paint.will_change_perspective
            || input.paint.will_change_transform_style;
        let will_change = MakeGarbageCollected(StyleWillChangeData::new(
            values,
            ids,
            input.paint.will_change_scroll_position,
            has_transform_property,
            input.paint.will_change_transform,
        ));
        builder.SetWillChange(Member::from_ptr(will_change));
    }
}

#[inline(never)]
pub(super) fn ApplyStyleTransforms(builder: &mut ComputedStyleBuilder, input: &ComputedStyle) {
    if let Some(translate) = &input.paint.translate {
        let operation = translate.operations.as_slice();
        if operation.len() != 1
            || !matches!(
                operation[0].kind,
                PaintTransformOperationKind::kTranslate | PaintTransformOperationKind::kTranslate3D
            )
        {
            Invalid("independent translate payload");
        }
        let operation = operation[0];
        let x = LengthPercentage(operation.pixels.x, operation.percentages.x, "translate.x");
        let y = LengthPercentage(operation.pixels.y, operation.percentages.y, "translate.y");
        let kind = if operation.kind == PaintTransformOperationKind::kTranslate3D {
            TransformOperationType::kTranslate3D
        } else {
            TransformOperationType::kTranslate
        };
        builder.SetTranslate(Member::from_ptr(MakeGarbageCollected(
            TranslateTransformOperation::new(x, y, Number(operation.z, "translate.z", false), kind),
        )));
    }
    if let Some(rotate) = &input.paint.rotate {
        let operation = rotate.operations.as_slice();
        if operation.len() != 1 || operation[0].kind != PaintTransformOperationKind::kRotate {
            Invalid("independent rotate payload");
        }
        let operation = operation[0];
        let [x, y, z] = operation.vector;
        let kind = if x == 0.0 && y == 0.0 && z == 1.0 {
            TransformOperationType::kRotate
        } else {
            TransformOperationType::kRotate3D
        };
        builder.SetRotate(Member::from_ptr(MakeGarbageCollected(
            RotateTransformOperation::new_3d(
                Number(x, "rotate.x", false),
                Number(y, "rotate.y", false),
                Number(z, "rotate.z", false),
                Number(operation.angle, "rotate.angle", false),
                kind,
            ),
        )));
    }
    if let Some(scale) = &input.paint.scale {
        let operation = scale.operations.as_slice();
        if operation.len() != 1 || operation[0].kind != PaintTransformOperationKind::kScale {
            Invalid("independent scale payload");
        }
        let operation = operation[0];
        let [x, y, z] = operation.vector;
        let kind = if z == 1.0 {
            TransformOperationType::kScale
        } else {
            TransformOperationType::kScale3D
        };
        builder.SetScale(Member::from_ptr(MakeGarbageCollected(
            ScaleTransformOperation::new(
                Number(x, "scale.x", false),
                Number(y, "scale.y", false),
                Number(z, "scale.z", false),
                kind,
            ),
        )));
    }
    if input.paint.offset_transform.is_some() {
        Invalid("motion-path projection without source ComputedStyle");
    }
    if let Some(transform) = &input.paint.transform {
        let mut operations = TransformOperations::default();
        for operation in &transform.operations {
            match operation.kind {
                PaintTransformOperationKind::kMatrix => {
                    for value in operation.matrix.values {
                        if !value.is_finite() {
                            Invalid("transform matrix");
                        }
                    }
                    operations.PushMatrix3D(gfx::Transform::ColMajor(&operation.matrix.values));
                }
                PaintTransformOperationKind::kTranslate => {
                    let x = LengthPercentage(
                        operation.pixels.x,
                        operation.percentages.x,
                        "transform translate x",
                    );
                    let y = LengthPercentage(
                        operation.pixels.y,
                        operation.percentages.y,
                        "transform translate y",
                    );
                    operations.PushTranslate(x, y);
                }
                PaintTransformOperationKind::kTranslate3D => {
                    let x = LengthPercentage(
                        operation.pixels.x,
                        operation.percentages.x,
                        "transform translate3d x",
                    );
                    let y = LengthPercentage(
                        operation.pixels.y,
                        operation.percentages.y,
                        "transform translate3d y",
                    );
                    operations.PushTranslate3D(
                        x,
                        y,
                        Number(operation.z, "transform translate3d z", false),
                    );
                }
                PaintTransformOperationKind::kRotate | PaintTransformOperationKind::kScale => {
                    Invalid("independent operation in legacy transform list")
                }
            }
        }
        builder.SetTransformOwned(operations);
    }
    if let Some(origin) = &input.paint.transform_origin {
        builder.SetTransformOriginX(&LengthPercentage(
            origin.pixels.x,
            origin.percentages.x,
            "transform_origin.x",
        ));
        builder.SetTransformOriginY(&LengthPercentage(
            origin.pixels.y,
            origin.percentages.y,
            "transform_origin.y",
        ));
        builder.SetTransformOriginZ(Number(origin.z, "transform_origin.z", false) as f32);
    }
}
