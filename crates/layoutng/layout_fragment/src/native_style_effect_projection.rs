//! Native paint-grouping and transform style projection.
//!
//! Included by `boundary::native_input` at the reciprocal assembly boundary.
//! Filter lists and transform-operation lists scale with author input, so the
//! two entries stay out of line as fragment-qualified sampling frames.

use super::*;

// Copy actual resolved CSS paint values into the native style. The native
// LayerTypeRequired then reads the same style fields as Chromium; a sidecar
// painting value alone does not establish native stacking-context semantics.
// cpp: core/style/computed_style.h:3345-3348
// cpp: core/style/computed_style.cc:2913-2949
#[inline(never)]
pub(super) fn ApplyNativeStylePaintGrouping(
    builder: &mut ComputedStyleBuilder,
    input: &ComputedStyle,
) {
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
            .map(clip_path_native::NativeClipPath),
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
    // Preserve each supported CSS filter's real native class/payload.
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
                | PaintFilterType::kHueRotate => {
                    let type_ = match filter.r#type {
                        PaintFilterType::kGrayscale => OperationType::kGrayscale,
                        PaintFilterType::kSepia => OperationType::kSepia,
                        PaintFilterType::kSaturate => OperationType::kSaturate,
                        _ => OperationType::kHueRotate,
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
    if input.paint.will_change_transform {
        let mut values = Vector::new();
        values.push(AtomicString::from_utf16(
            &"transform".encode_utf16().collect::<Vec<_>>(),
        ));
        let ids = CSSBitset::FromList(&[CSSPropertyID::kTransform]);
        let will_change =
            MakeGarbageCollected(StyleWillChangeData::new(values, ids, false, true, true));
        builder.SetWillChange(Member::from_ptr(will_change));
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:1360-1399
#[inline(never)]
pub(super) fn ApplyNativeStyleTransforms(
    builder: &mut ComputedStyleBuilder,
    input: &ComputedStyle,
) {
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
