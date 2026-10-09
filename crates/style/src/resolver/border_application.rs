// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native border color, radius and spacing Apply* branches.
#![allow(non_snake_case)]
use super::*;
use foundation::LengthSize;

pub(super) fn IsBorderProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kBorderTopColor
            | kBorderRightColor
            | kBorderBottomColor
            | kBorderLeftColor
            | kBorderTopLeftRadius
            | kBorderTopRightRadius
            | kBorderBottomRightRadius
            | kBorderBottomLeftRadius
            | kWebkitBorderHorizontalSpacing
            | kWebkitBorderVerticalSpacing
    ) || matches!(
        id,
        kBorderTopWidth | kBorderRightWidth | kBorderBottomWidth | kBorderLeftWidth
    )
}

fn ConvertPixels(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<f64, LonghandApplicationError> {
    use crate::css_math_expression_node::CSSMathLengthResolver;
    let mut resolver = super::MathLengthResolver(
        id,
        b.GetFontDescription().ComputedSize(),
        root,
        b.EffectiveZoom(),
        media,
    );
    let pixels = match v.Payload() {
        CSSValuePayload::kNumericLiteralClass(number)
            if number.GetType() != UnitType::kPercentage =>
        {
            resolver.ComputeLength(number.DoubleValue(), number.GetType())
        }
        CSSValuePayload::kMathFunctionClass(math)
            if math.Category()
                == crate::css_math_expression_node::CalculationResultCategory::Length =>
        {
            math.ComputeValue(&mut resolver, None)
        }
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    }
    .map_err(|_| LonghandApplicationError::Unsupported(id))?;
    if pixels < 0.0 {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    Ok(pixels)
}

// Shared ConvertBorderWidth branch used by borders and gap-decoration widths.
pub(super) fn ConvertBorderWidthValue(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<i32, LonghandApplicationError> {
    let pixels = if let CSSValuePayload::kIdentifierClass(value) = v.Payload() {
        let nominal = match value.0 {
            CSSValueID::kThin => 1.0,
            CSSValueID::kMedium => 3.0,
            CSSValueID::kThick => 5.0,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        };
        nominal * b.EffectiveZoom() as f64
    } else {
        ConvertPixels(id, b, v, root, media)? as f32 as f64
    };
    Ok(crate::resolver::style_builder_converter::StyleBuilderConverter::ClampLineWidth(pixels))
}

// cpp: style_builder_converter.cc:2573-2582 ConvertRadius;
// CSSPrimitiveValue::ConvertToLength / CSSLengthResolver::ZoomedComputedPixels.
fn ConvertLength(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<Length, LonghandApplicationError> {
    let mut resolver = super::MathLengthResolver(
        id,
        b.GetFontDescription().ComputedSize(),
        root,
        b.EffectiveZoom(),
        media,
    );
    if let CSSValuePayload::kMathFunctionClass(math) = v.Payload() {
        return math
            .ConvertToLength(&mut resolver)
            .map_err(|_| LonghandApplicationError::Unsupported(id));
    }
    let CSSValuePayload::kNumericLiteralClass(number) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if number.GetType() == UnitType::kPercentage {
        return Ok(Length::Percent(number.DoubleValue()));
    }
    use crate::css_math_expression_node::CSSMathLengthResolver;
    let pixels = resolver
        .ComputeLength(number.DoubleValue(), number.GetType())
        .map_err(|_| LonghandApplicationError::Unsupported(id))?;
    if pixels < 0.0 {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    Ok(Length::Fixed(
        crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(pixels),
    ))
}

pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    use CSSPropertyID::*;
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue()
        || v.IsUnsetValue() && !CSSProperty::Get(id).IsInherited()
        || inherit && parent.is_none();
    if matches!(
        id,
        kBorderTopWidth | kBorderRightWidth | kBorderBottomWidth | kBorderLeftWidth
    ) {
        // cpp: longhands_custom.cc:1408-1426,1642-1660,1729-1747,1834-1852;
        // style_builder_converter.cc ConvertBorderWidth / ClampLineWidth.
        let width = if initial {
            let nominal = match id {
                kBorderTopWidth => ComputedStyleInitialValues::InitialBorderTopWidth(),
                kBorderRightWidth => ComputedStyleInitialValues::InitialBorderRightWidth(),
                kBorderBottomWidth => ComputedStyleInitialValues::InitialBorderBottomWidth(),
                _ => ComputedStyleInitialValues::InitialBorderLeftWidth(),
            };
            (nominal as f64 * b.EffectiveZoom() as f64) as i32
        } else if inherit {
            let p = parent.unwrap();
            if p.EffectiveZoom() != b.EffectiveZoom() {
                return Err(LonghandApplicationError::Unsupported(id));
            }
            match id {
                kBorderTopWidth => *p.SpecifiedBorderTopWidth(),
                kBorderRightWidth => *p.SpecifiedBorderRightWidth(),
                kBorderBottomWidth => *p.SpecifiedBorderBottomWidth(),
                _ => *p.SpecifiedBorderLeftWidth(),
            }
        } else {
            ConvertBorderWidthValue(id, b, v, root, media)?
        };
        ApplyConvertedBorderWidth(id, b, &width)?;
        if inherit && !initial && v.IsInheritedValue() {
            b.SetHasExplicitInheritance();
            parent.unwrap().SetChildHasExplicitInheritance();
        }
        return Ok(());
    }
    if matches!(
        id,
        kBorderTopColor | kBorderRightColor | kBorderBottomColor | kBorderLeftColor
    ) {
        // cpp: generated longhands.cc:4142-4152,4910-4920,5032-5042,5276-5286.
        if initial {
            return ApplyConvertedColor(id, b, &StyleColor::CurrentColor());
        }
        if inherit {
            if v.IsInheritedValue() {
                b.SetHasExplicitInheritance();
                parent.unwrap().SetChildHasExplicitInheritance();
            }
            return ApplyInherit(id, b, parent.unwrap());
        }
        let color = super::color_ui_application::ConvertStyleColorValue(id, v)?;
        return ApplyConvertedColor(id, b, &color);
    }
    if initial {
        return ApplyInitial(id, b);
    }
    if inherit && b.EffectiveZoom() != parent.unwrap().EffectiveZoom() {
        // ApplyParentValueIfZoomChanged needs the document's standardized zoom
        // policy and inherited conversion data; keep that service boundary.
        return Err(LonghandApplicationError::Unsupported(id));
    }
    if matches!(
        id,
        kWebkitBorderHorizontalSpacing | kWebkitBorderVerticalSpacing
    ) {
        // cpp: generated longhands.cc:18222-18235,18288-18301;
        // css_primitive_value.cc:335-339 ComputeLength<int16_t>.
        let horizontal = id == kWebkitBorderHorizontalSpacing;
        let value = if inherit {
            if horizontal {
                parent.unwrap().HorizontalBorderSpacing()
            } else {
                parent.unwrap().VerticalBorderSpacing()
            }
        } else {
            let pixels = ConvertPixels(id, b, v, root, media)?;
            let adjusted = pixels + if pixels < 0.0 { -0.01 } else { 0.01 };
            if adjusted < i16::MIN as f64 || adjusted > i16::MAX as f64 {
                0
            } else {
                adjusted as i16
            }
        };
        if horizontal {
            b.SetHorizontalBorderSpacing(value);
        } else {
            b.SetVerticalBorderSpacing(value);
        }
    } else {
        // cpp: generated longhands.cc:4188-4201,4237-4250,5322-5335,5371-5384.
        let radius = if inherit {
            match id {
                kBorderTopLeftRadius => parent.unwrap().BorderTopLeftRadius().clone(),
                kBorderTopRightRadius => parent.unwrap().BorderTopRightRadius().clone(),
                kBorderBottomRightRadius => parent.unwrap().BorderBottomRightRadius().clone(),
                kBorderBottomLeftRadius => parent.unwrap().BorderBottomLeftRadius().clone(),
                _ => unreachable!(),
            }
        } else {
            let CSSValuePayload::kValuePairClass(pair) = v.Payload() else {
                return Err(LonghandApplicationError::InvalidValue(id));
            };
            LengthSize::new(
                &ConvertLength(id, b, &pair.first, root, media)?,
                &ConvertLength(id, b, &pair.second, root, media)?,
            )
        };
        match id {
            kBorderTopLeftRadius => b.SetBorderTopLeftRadius(&radius),
            kBorderTopRightRadius => b.SetBorderTopRightRadius(&radius),
            kBorderBottomRightRadius => b.SetBorderBottomRightRadius(&radius),
            kBorderBottomLeftRadius => b.SetBorderBottomLeftRadius(&radius),
            _ => unreachable!(),
        }
    }
    if inherit && v.IsInheritedValue() && !CSSProperty::Get(id).IsInherited() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}
