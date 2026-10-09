// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium native line and pagination Apply* branches and converters.
#![allow(non_snake_case)]
use super::*;
use foundation::{
    EBlockEllipsis, EBreakBetween, EBreakInside, EContinue, TabSize, TabSizeValueType,
};
use layoutng_style::style::{
    computed_style_constants::EVerticalAlign, max_lines_data::MaxLinesData,
};

pub(super) fn IsLineProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kVerticalAlign
            | kTabSize
            | kWebkitLineClamp
            | kLineClamp
            | kMaxLines
            | kAlternativeWebkitLineClampLonghand
            | kContinue
            | kBlockEllipsis
            | kBreakBefore
            | kBreakAfter
            | kBreakInside
    )
}
fn Number(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<f64, LonghandApplicationError> {
    match v.Payload() {
        CSSValuePayload::kNumericLiteralClass(n)
            if matches!(n.GetType(), UnitType::kNumber | UnitType::kInteger) =>
        {
            Ok(n.DoubleValue())
        }
        CSSValuePayload::kMathFunctionClass(m)
            if m.Category()
                == crate::css_math_expression_node::CalculationResultCategory::Number =>
        {
            m.ComputeValue(
                &mut MathLengthResolver(
                    id,
                    b.GetFontDescription().ComputedSize(),
                    root,
                    b.EffectiveZoom(),
                    media,
                ),
                None,
            )
            .map_err(|_| LonghandApplicationError::Unsupported(id))
        }
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}
fn PositiveInteger(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<f64, LonghandApplicationError> {
    let n = Number(id, b, v, root, media)?;
    if n.is_nan() || n < 1.0 || n.is_finite() && n.fract() != 0.0 {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    Ok(n)
}
fn LengthValue(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    percent: bool,
) -> std::result::Result<Length, LonghandApplicationError> {
    use crate::css_math_expression_node::{CSSMathLengthResolver, CalculationResultCategory as C};
    let mut r = MathLengthResolver(
        id,
        b.GetFontDescription().ComputedSize(),
        root,
        b.EffectiveZoom(),
        media,
    );
    match v.Payload() {
        CSSValuePayload::kNumericLiteralClass(n)
            if percent && n.GetType() == UnitType::kPercentage =>
        {
            Ok(Length::Percent(n.DoubleValue()))
        }
        CSSValuePayload::kNumericLiteralClass(n)
            if crate::css_numeric_literal_value::IsLength(n.GetType())
                || n.GetType() == UnitType::kUserUnits =>
        {
            let px = r
                .ComputeLength(
                    n.DoubleValue(),
                    if n.GetType() == UnitType::kUserUnits {
                        UnitType::kPixels
                    } else {
                        n.GetType()
                    },
                )
                .map_err(|_| LonghandApplicationError::Unsupported(id))?;
            Ok(Length::Fixed(
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(px),
            ))
        }
        CSSValuePayload::kMathFunctionClass(m) if percent || m.Category() == C::Length => {
            if m.Category() == C::Number && percent {
                let px = m
                    .ComputeValue(&mut r, None)
                    .map_err(|_| LonghandApplicationError::Unsupported(id))?
                    * b.EffectiveZoom() as f64;
                return Ok(Length::Fixed(
                    crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(px),
                ));
            }
            m.ConvertToLength(&mut r)
                .map_err(|_| LonghandApplicationError::Unsupported(id))
        }
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}
fn Between(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<EBreakBetween, LonghandApplicationError> {
    use CSSValueID::*;
    Ok(match Identifier(id, v)? {
        kAuto => EBreakBetween::kAuto,
        kAvoid => EBreakBetween::kAvoid,
        kAvoidColumn => EBreakBetween::kAvoidColumn,
        kAvoidPage => EBreakBetween::kAvoidPage,
        kColumn => EBreakBetween::kColumn,
        kPage => EBreakBetween::kPage,
        kLeft => EBreakBetween::kLeft,
        kRight => EBreakBetween::kRight,
        kRecto => EBreakBetween::kRecto,
        kVerso => EBreakBetween::kVerso,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
fn Alignment(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<EVerticalAlign, LonghandApplicationError> {
    use CSSValueID::*;
    Ok(match Identifier(id, v)? {
        kBaseline => EVerticalAlign::kBaseline,
        kMiddle => EVerticalAlign::kMiddle,
        kSub => EVerticalAlign::kSub,
        kSuper => EVerticalAlign::kSuper,
        kTextTop => EVerticalAlign::kTextTop,
        kTextBottom => EVerticalAlign::kTextBottom,
        kTop => EVerticalAlign::kTop,
        kBottom => EVerticalAlign::kBottom,
        kWebkitBaselineMiddle => EVerticalAlign::kBaselineMiddle,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
// style_builder_converter.cc:4308-4330; CSSPrimitiveValue::ConvertTo<uint16_t>
// uses ClampTo, so conversion saturates the native field rather than wrapping.
fn MaxLines(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<MaxLinesData, LonghandApplicationError> {
    if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kAuto) {
        return Ok(MaxLinesData::new(0, true));
    }
    let (number, auto) = if let CSSValuePayload::kValuePairClass(pair) = v.Payload() {
        if Identifier(id, &pair.second)? != CSSValueID::kAuto {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
        (&*pair.first, true)
    } else {
        (v, false)
    };
    Ok(MaxLinesData::new(
        PositiveInteger(id, b, number, root, media)? as u16,
        auto,
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
    if !crate::production_line_features::IsExposed(id) {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    ApplyInternal(id, b, parent, v, root, media)
}
fn ApplyInternal(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    use CSSPropertyID::*;
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue() || v.IsUnsetValue() && !inherit || inherit && parent.is_none();
    let inherited = parent.filter(|_| inherit && !initial);
    match id {
        // longhands_custom.cc:10821-10849. Do not invoke a nonexistent document
        // standardized-browser-zoom collaborator when inheriting changed zoom.
        kVerticalAlign => {
            if initial {
                b.SetVerticalAlign(EVerticalAlign::kBaseline);
            } else if let Some(p) = inherited {
                if b.EffectiveZoom() != p.EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                b.SetVerticalAlign(p.VerticalAlign());
                if p.VerticalAlign() == EVerticalAlign::kLength {
                    b.SetVerticalAlignLength(p.GetVerticalAlignLength());
                }
            } else if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(_)) {
                b.SetVerticalAlign(Alignment(id, v)?);
            } else {
                b.SetVerticalAlignLength(&LengthValue(id, b, v, root, media, true)?);
            }
        }
        // generated longhands.cc:16060-16073; converter.cc:2170-2184.
        kTabSize => {
            let tab = if initial {
                TabSize::spaces(8.0)
            } else if let Some(p) = inherited {
                if b.EffectiveZoom() != p.EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                *p.GetTabSize()
            } else if matches!(v.Payload(),CSSValuePayload::kNumericLiteralClass(n) if matches!(n.GetType(),UnitType::kNumber | UnitType::kInteger))
                || matches!(v.Payload(),CSSValuePayload::kMathFunctionClass(m) if m.Category()==crate::css_math_expression_node::CalculationResultCategory::Number)
            {
                let n = Number(id, b, v, root, media)?;
                if n.is_nan() || n < 0.0 {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                TabSize::spaces(n as f32)
            } else {
                let length = LengthValue(id, b, v, root, media, false)?;
                if length.Pixels() < 0.0 {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                TabSize::new(length.Pixels(), TabSizeValueType::kLength)
            };
            b.SetTabSize(&tab);
        }
        kBreakBefore | kBreakAfter => {
            let val = if initial {
                EBreakBetween::kAuto
            } else if let Some(p) = inherited {
                if id == kBreakBefore {
                    p.BreakBefore()
                } else {
                    p.BreakAfter()
                }
            } else {
                Between(id, v)?
            };
            if id == kBreakBefore {
                b.SetBreakBefore(val);
            } else {
                b.SetBreakAfter(val);
            }
        }
        kBreakInside => {
            let val = if initial {
                EBreakInside::kAuto
            } else if let Some(p) = inherited {
                p.BreakInside()
            } else {
                match Identifier(id, v)? {
                    CSSValueID::kAuto => EBreakInside::kAuto,
                    CSSValueID::kAvoid => EBreakInside::kAvoid,
                    CSSValueID::kAvoidPage => EBreakInside::kAvoidPage,
                    CSSValueID::kAvoidColumn => EBreakInside::kAvoidColumn,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetBreakInside(val);
        }
        // generated longhands.cc:18593-18601; converter.h:583-594.
        kWebkitLineClamp => {
            let lines = if initial {
                0
            } else if let Some(p) = inherited {
                p.WebkitLineClamp()
            } else if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kNone)
            {
                0
            } else {
                PositiveInteger(id, b, v, root, media)? as i32
            };
            b.SetWebkitLineClamp(lines);
        }
        // longhands_custom.cc:6515-6565,11312-11347. These fields belong to
        // enabled CSSLineClamp, and remain behind the runtime Exposure gate.
        kLineClamp | kAlternativeWebkitLineClampLonghand => {
            let (continuation, max, ellipsis) = if initial {
                (
                    EContinue::kNormal,
                    MaxLinesData::new(0, true),
                    EBlockEllipsis::kNoEllipsis,
                )
            } else if let Some(p) = inherited {
                (
                    p.Continue(),
                    *p.MaxLines(),
                    p.LineClampInternalBlockEllipsis(),
                )
            } else if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kNone)
            {
                (
                    EContinue::kNormal,
                    MaxLinesData::new(0, true),
                    EBlockEllipsis::kNoEllipsis,
                )
            } else if id == kAlternativeWebkitLineClampLonghand {
                (
                    EContinue::kWebkitLegacy,
                    MaxLinesData::new(PositiveInteger(id, b, v, root, media)? as u16, false),
                    EBlockEllipsis::kEllipsis,
                )
            } else {
                let CSSValuePayload::kValueListClass(list) = v.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                if list.separator != crate::production_css_value::ListSeparator::Space
                    || list.values.is_empty()
                {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                let mut lines = 0;
                let mut auto = false;
                let mut legacy = false;
                let mut ellipsis = EBlockEllipsis::kEllipsis;
                for item in &list.values {
                    if matches!(
                        item.Payload(),
                        CSSValuePayload::kNumericLiteralClass(_)
                            | CSSValuePayload::kMathFunctionClass(_)
                    ) {
                        lines = PositiveInteger(id, b, item, root, media)? as u16;
                        continue;
                    }
                    match Identifier(id, item)? {
                        CSSValueID::kAuto => auto = true,
                        CSSValueID::kWebkitLegacy => legacy = true,
                        CSSValueID::kEllipsis => ellipsis = EBlockEllipsis::kEllipsis,
                        CSSValueID::kNoEllipsis => ellipsis = EBlockEllipsis::kNoEllipsis,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    }
                }
                (
                    if legacy {
                        EContinue::kWebkitLegacy
                    } else {
                        EContinue::kCollapse
                    },
                    MaxLinesData::new(lines, auto || lines == 0),
                    ellipsis,
                )
            };
            b.SetContinue(continuation);
            b.SetMaxLines(&max);
            b.SetLineClampInternalBlockEllipsis(ellipsis);
        }
        kMaxLines => {
            let max = if initial {
                MaxLinesData::new(0, true)
            } else if let Some(p) = inherited {
                *p.MaxLines()
            } else {
                MaxLines(id, b, v, root, media)?
            };
            b.SetMaxLines(&max);
        }
        kContinue => {
            let val = if initial {
                EContinue::kNormal
            } else if let Some(p) = inherited {
                p.Continue()
            } else {
                match Identifier(id, v)? {
                    CSSValueID::kNormal => EContinue::kNormal,
                    CSSValueID::kCollapse => EContinue::kCollapse,
                    CSSValueID::kWebkitLegacy => EContinue::kWebkitLegacy,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetContinue(val);
        }
        kBlockEllipsis => {
            let val = if initial {
                EBlockEllipsis::kNoEllipsis
            } else if let Some(p) = inherited {
                p.BlockEllipsis()
            } else {
                match Identifier(id, v)? {
                    CSSValueID::kEllipsis => EBlockEllipsis::kEllipsis,
                    CSSValueID::kNoEllipsis => EBlockEllipsis::kNoEllipsis,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetBlockEllipsis(val);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::production_css_value as values;
    #[test]
    fn hidden_line_native_branches_preserve_separate_legacy_and_extra_fields() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut b = ComputedStyleBuilder::from_style(initial);
        let media = MediaValuesCachedData::default();
        b.SetWebkitLineClamp(7);
        let value = values::list(
            vec![
                values::numeric(3.0, UnitType::kInteger),
                values::identifier(CSSValueID::kAuto),
                values::identifier(CSSValueID::kNoEllipsis),
                values::identifier(CSSValueID::kWebkitLegacy),
            ],
            values::ListSeparator::Space,
        );
        ApplyInternal(
            CSSPropertyID::kLineClamp,
            &mut b,
            None,
            &value,
            16.0,
            &media,
        )
        .unwrap();
        assert_eq!(b.Continue(), EContinue::kWebkitLegacy);
        assert_eq!(b.MaxLines().Lines(), 3);
        assert!(b.MaxLines().HasAutoKeyword());
        assert_eq!(
            b.LineClampInternalBlockEllipsis(),
            EBlockEllipsis::kNoEllipsis
        );
        assert_eq!(b.WebkitLineClamp(), 7);
        let huge = values::numeric(9999999.0, UnitType::kInteger);
        ApplyInternal(
            CSSPropertyID::kAlternativeWebkitLineClampLonghand,
            &mut b,
            None,
            &huge,
            16.0,
            &media,
        )
        .unwrap();
        assert_eq!(b.MaxLines().Lines(), u16::MAX as u32);
        assert!(!b.MaxLines().HasAutoKeyword());
        assert_eq!(
            b.LineClampInternalBlockEllipsis(),
            EBlockEllipsis::kEllipsis
        );
        let none = values::identifier(CSSValueID::kNone);
        ApplyInternal(CSSPropertyID::kLineClamp, &mut b, None, &none, 16.0, &media).unwrap();
        assert_eq!(b.Continue(), EContinue::kNormal);
        assert!(b.MaxLines().IsAutoValue());
        assert_eq!(
            b.LineClampInternalBlockEllipsis(),
            EBlockEllipsis::kNoEllipsis
        );
        assert_eq!(b.WebkitLineClamp(), 7);
        let max = values::Value::new(CSSValuePayload::kValuePairClass(values::CSSValuePair {
            first: values::numeric(4.0, UnitType::kInteger),
            second: values::identifier(CSSValueID::kAuto),
            drop_identical: false,
        }));
        ApplyInternal(CSSPropertyID::kMaxLines, &mut b, None, &max, 16.0, &media).unwrap();
        assert_eq!(b.MaxLines().Lines(), 4);
        assert!(b.MaxLines().HasAutoKeyword());
    }
}
