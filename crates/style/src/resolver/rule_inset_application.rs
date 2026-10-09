// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! style_builder_converter.cc:2069-2077; generated inset Apply* in longhands.cc.
#![allow(non_snake_case)]
use super::*;
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kColumnRuleInsetCapStart
            | CSSPropertyID::kColumnRuleInsetCapEnd
            | CSSPropertyID::kColumnRuleInsetJunctionStart
            | CSSPropertyID::kColumnRuleInsetJunctionEnd
            | CSSPropertyID::kRowRuleInsetCapStart
            | CSSPropertyID::kRowRuleInsetCapEnd
            | CSSPropertyID::kRowRuleInsetJunctionStart
            | CSSPropertyID::kRowRuleInsetJunctionEnd
    )
}
fn Convert(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<Length, LonghandApplicationError> {
    if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kOverlapJoin)
    {
        return Ok(Length::new(0.0, LengthType::kOverlapJoin));
    }
    let mut resolver = MathLengthResolver(
        id,
        b.GetFontDescription().ComputedSize(),
        root,
        b.EffectiveZoom(),
        media,
    );
    if let CSSValuePayload::kMathFunctionClass(m) = v.Payload() {
        return m
            .ConvertToLength(&mut resolver)
            .map_err(|_| LonghandApplicationError::Unsupported(id));
    }
    let CSSValuePayload::kNumericLiteralClass(n) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if n.GetType() == UnitType::kPercentage {
        return Ok(Length::Percent(n.DoubleValue()));
    }
    use crate::css_math_expression_node::CSSMathLengthResolver;
    let pixels = resolver
        .ComputeLength(n.DoubleValue(), n.GetType())
        .map_err(|_| LonghandApplicationError::Unsupported(id))?;
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
    let inherit = v.IsInheritedValue();
    let initial = v.IsInitialValue() || v.IsUnsetValue() || inherit && parent.is_none();
    if inherit && !initial && b.EffectiveZoom() != parent.unwrap().EffectiveZoom() {
        // Chromium ApplyParentValueIfZoomChanged requires the document zoom
        // owner and parent computed CSS value; preserve this typed boundary.
        return Err(LonghandApplicationError::Unsupported(id));
    }
    let length = if initial {
        Length::Fixed(0.0)
    } else if inherit {
        match id {
            CSSPropertyID::kColumnRuleInsetCapStart => {
                parent.unwrap().ColumnRuleInsetCapStart().clone()
            }
            CSSPropertyID::kColumnRuleInsetCapEnd => {
                parent.unwrap().ColumnRuleInsetCapEnd().clone()
            }
            CSSPropertyID::kColumnRuleInsetJunctionStart => {
                parent.unwrap().ColumnRuleInsetJunctionStart().clone()
            }
            CSSPropertyID::kColumnRuleInsetJunctionEnd => {
                parent.unwrap().ColumnRuleInsetJunctionEnd().clone()
            }
            CSSPropertyID::kRowRuleInsetCapStart => parent.unwrap().RowRuleInsetCapStart().clone(),
            CSSPropertyID::kRowRuleInsetCapEnd => parent.unwrap().RowRuleInsetCapEnd().clone(),
            CSSPropertyID::kRowRuleInsetJunctionStart => {
                parent.unwrap().RowRuleInsetJunctionStart().clone()
            }
            CSSPropertyID::kRowRuleInsetJunctionEnd => {
                parent.unwrap().RowRuleInsetJunctionEnd().clone()
            }
            _ => return Err(LonghandApplicationError::Unsupported(id)),
        }
    } else {
        Convert(id, b, v, root, media)?
    };
    ApplyConvertedLength(id, b, &length)?;
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}
