// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! style_builder_converter.cc:228-268,1835-1844,1942-1951; generated Apply*.
#![allow(non_snake_case)]
use super::*;
use foundation::{DynamicRangeLimit, DynamicRangeLimitKind};
use layoutng_style::style::{
    flow_tolerance::FlowTolerance, style_interest_delay::StyleInterestDelay,
};
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kDynamicRangeLimit
            | CSSPropertyID::kFlowTolerance
            | CSSPropertyID::kInterestDelayStart
            | CSSPropertyID::kInterestDelayEnd
    )
}
fn Limit(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<DynamicRangeLimit, LonghandApplicationError> {
    if let CSSValuePayload::kIdentifierClass(k) = v.Payload() {
        return Ok(DynamicRangeLimit::new(match k.0 {
            CSSValueID::kStandard => DynamicRangeLimitKind::kStandard,
            CSSValueID::kNoLimit => DynamicRangeLimitKind::kHigh,
            CSSValueID::kConstrained => DynamicRangeLimitKind::kConstrainedHigh,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        }));
    }
    let CSSValuePayload::kDynamicRangeLimitMixClass(m) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if m.limits.is_empty() || m.limits.len() != m.percentages.len() {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let mut standard = 0f32;
    let mut constrained = 0f32;
    let mut total = 0f32;
    for (l, p) in m.limits.iter().zip(&m.percentages) {
        let limit = Limit(id, l)?;
        let resolved = ResolveNumericMathList(id, p)?;
        let p = resolved.as_deref().unwrap_or(p);
        let CSSValuePayload::kNumericLiteralClass(p) = p.Payload() else {
            return Err(LonghandApplicationError::InvalidValue(id));
        };
        if p.GetType() != UnitType::kPercentage {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
        let fraction = 0.01f32 * (p.DoubleValue() as f32).clamp(0.0, 100.0);
        total += fraction;
        standard += fraction * limit.standard_mix;
        constrained += fraction * limit.constrained_high_mix;
    }
    Ok(if total == 0.0 {
        DynamicRangeLimit::new(DynamicRangeLimitKind::kHigh)
    } else {
        DynamicRangeLimit::from_mix(standard / total, constrained / total)
    })
}
pub(super) fn ApplyInternal(
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
    if inherit && !initial && v.IsInheritedValue() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    match id {
        kDynamicRangeLimit => {
            let limit = if initial {
                ComputedStyleInitialValues::InitialDynamicRangeLimit()
            } else if inherit {
                *parent.unwrap().GetDynamicRangeLimit()
            } else {
                Limit(id, v)?
            };
            b.SetDynamicRangeLimit(&limit);
            Ok(())
        }
        kFlowTolerance => {
            let flow = if initial {
                ComputedStyleInitialValues::InitialFlowTolerance()
            } else if inherit {
                if b.EffectiveZoom() != parent.unwrap().EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                parent.unwrap().GetFlowTolerance().clone()
            } else if let CSSValuePayload::kIdentifierClass(k) = v.Payload() {
                if !matches!(k.0, CSSValueID::kNormal | CSSValueID::kInfinite) {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                FlowTolerance::from_keyword(k.0)
            } else {
                FlowTolerance::from_length(
                    &crate::resolver::position_repeat_application::ConvertLength(
                        id, v, b, root, media,
                    )?,
                )
            };
            b.SetFlowTolerance(&flow);
            Ok(())
        }
        kInterestDelayStart | kInterestDelayEnd => {
            let delay = if initial {
                StyleInterestDelay::default()
            } else if inherit {
                if id == kInterestDelayStart {
                    *parent.unwrap().InterestDelayStart()
                } else {
                    *parent.unwrap().InterestDelayEnd()
                }
            } else if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kNormal)
            {
                StyleInterestDelay::default()
            } else {
                let resolved = ResolveNumericMathList(id, v)?;
                StyleInterestDelay::new(Seconds(id, resolved.as_deref().unwrap_or(v))?)
            };
            if id == kInterestDelayStart {
                b.SetInterestDelayStart(&delay)
            } else {
                b.SetInterestDelayEnd(&delay)
            }
            Ok(())
        }
        _ => Err(LonghandApplicationError::Unsupported(id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{
            ParseDeclarationList, ParseProperty, ParsePropertyTokens, PropertyParseErrorKind,
        },
    };
    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn apply(b: &mut ComputedStyleBuilder, p: Option<&ComputedStyle>, css: &str) {
        let d = ParseDeclarationList(
            &foundation::String::from(css),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(d.errors.is_empty(), "{:?}", d.errors);
        for v in d.properties {
            super::super::Apply(
                v.PropertyID(),
                b,
                p,
                v.Value(),
                16.0,
                &MediaValuesCachedData::default(),
            )
            .unwrap();
        }
    }
    #[test]
    fn production_render_delay_nested_mix_reaches_native_weights() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b,None,"dynamic-range-limit:dynamic-range-limit-mix(standard 25%,dynamic-range-limit-mix(constrained 50%,no-limit 50%) 75%)");
        assert_eq!(b.GetDynamicRangeLimit().standard_mix, 0.25);
        assert_eq!(b.GetDynamicRangeLimit().constrained_high_mix, 0.375);
        apply(&mut b,None,"dynamic-range-limit:dynamic-range-limit-mix(standard calc(200%),constrained calc(-5%))");
        assert_eq!(
            b.GetDynamicRangeLimit(),
            &DynamicRangeLimit::new(DynamicRangeLimitKind::kStandard)
        );
        apply(
            &mut b,
            None,
            "dynamic-range-limit:dynamic-range-limit-mix(standard calc(0%),constrained calc(0%))",
        );
        assert_eq!(
            b.GetDynamicRangeLimit(),
            &DynamicRangeLimit::new(DynamicRangeLimitKind::kHigh)
        );
        apply(&mut b, None, "dynamic-range-limit:constrained");
        assert_eq!(
            b.GetDynamicRangeLimit(),
            &DynamicRangeLimit::new(DynamicRangeLimitKind::kConstrainedHigh)
        );
    }
    #[test]
    fn production_render_delay_interest_time_math_and_two_longhand_repetition() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b, None, "interest-delay:calc(1s + 250ms) 500ms");
        assert_eq!(b.InterestDelayStart().DelaySeconds(), 1.25);
        assert_eq!(b.InterestDelayEnd().DelaySeconds(), 0.5);
        apply(&mut b, None, "interest-delay:250ms");
        assert_eq!(b.InterestDelayStart().DelaySeconds(), 0.25);
        assert_eq!(b.InterestDelayEnd().DelaySeconds(), 0.25);
        apply(&mut b, None, "interest-delay:normal calc(-2s)");
        assert!(b.InterestDelayStart().IsNormal());
        assert_eq!(b.InterestDelayEnd().DelaySeconds(), 0.0);
        let parsed = ParseProperty(
            CSSPropertyID::kInterestDelay,
            &foundation::String::from("normal"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(parsed[0].Value() == parsed[1].Value());
    }
    #[test]
    fn production_render_delay_css_wide_inherit_native_and_cow() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut p,
            None,
            "dynamic-range-limit:standard;interest-delay:1s 2s",
        );
        let p = unsafe { &*p.TakeStyle() };
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            Some(p),
            "dynamic-range-limit:unset;interest-delay:inherit",
        );
        assert_eq!(b.GetDynamicRangeLimit(), p.GetDynamicRangeLimit());
        assert_eq!(b.InterestDelayEnd(), p.InterestDelayEnd());
        assert!(b.HasExplicitInheritance());
        apply(
            &mut b,
            Some(p),
            "dynamic-range-limit:initial;interest-delay:unset",
        );
        assert_eq!(
            b.GetDynamicRangeLimit(),
            &DynamicRangeLimit::new(DynamicRangeLimitKind::kHigh)
        );
        assert!(b.InterestDelayStart().IsNormal());
        assert!(b.InterestDelayEnd().IsNormal());
        assert_eq!(p.InterestDelayEnd().DelaySeconds(), 2.0);
    }
    #[test]
    fn production_render_delay_invalids_and_all_flow_entry_gates() {
        for text in [
            "dynamic-range-limit-mix(standard 0%,no-limit 0%)",
            "dynamic-range-limit-mix(standard -1%)",
            "dynamic-range-limit-mix(standard 101%)",
            "dynamic-range-limit-mix(standard 1)",
            "dynamic-range-limit-mix(standard 20%,)",
            "auto",
        ] {
            assert!(
                ParseProperty(
                    CSSPropertyID::kDynamicRangeLimit,
                    &foundation::String::from(text),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .is_err(),
                "{text}"
            );
        }
        for text in ["-1s", "0", "1%", "1s 2s 3s", "1s,2s"] {
            assert!(
                ParseProperty(
                    CSSPropertyID::kInterestDelay,
                    &foundation::String::from(text),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .is_err(),
                "{text}"
            );
        }
        for text in ["normal", "20px", "initial", "inherit", "var(--x)"] {
            assert_eq!(
                ParseProperty(
                    CSSPropertyID::kFlowTolerance,
                    &foundation::String::from(text),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .err()
                .unwrap()
                .kind,
                PropertyParseErrorKind::Unsupported
            );
        }
        let v = ParseProperty(
            CSSPropertyID::kDynamicRangeLimit,
            &foundation::String::from("var(--x)"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        let CSSValuePayload::kUnparsedDeclarationClass(value) = v[0].Value().Payload() else {
            panic!("variable data")
        };
        assert_eq!(
            ParsePropertyTokens(
                CSSPropertyID::kFlowTolerance,
                &value.data,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Unsupported
        );
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        let value = crate::production_css_value::wide(CSSValueID::kInitial).unwrap();
        assert_eq!(
            super::super::Apply(
                CSSPropertyID::kFlowTolerance,
                &mut b,
                None,
                &value,
                16.0,
                &MediaValuesCachedData::default()
            ),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kFlowTolerance
            ))
        );
        assert!(b.GetFlowTolerance().IsNormal());
        assert_eq!(
            ApplyInitial(CSSPropertyID::kFlowTolerance, &mut b),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kFlowTolerance
            ))
        );
        assert_eq!(
            ApplyInherit(CSSPropertyID::kFlowTolerance, &mut b, initial()),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kFlowTolerance
            ))
        );
    }
    #[test]
    fn hidden_flow_tolerance_native_application_css_wide_and_zoom() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        let id = CSSPropertyID::kFlowTolerance;
        let media = MediaValuesCachedData::default();
        ApplyInternal(
            id,
            &mut b,
            None,
            &crate::production_css_value::numeric(20.0, UnitType::kPixels),
            16.0,
            &media,
        )
        .unwrap();
        assert_eq!(b.GetFlowTolerance().GetLength(), &Length::Fixed(20));
        ApplyInternal(
            id,
            &mut b,
            None,
            &crate::production_css_value::identifier(CSSValueID::kInfinite),
            16.0,
            &media,
        )
        .unwrap();
        assert!(b.GetFlowTolerance().IsInfinite());
        let p = unsafe { &*b.TakeStyle() };
        let mut child = ComputedStyleBuilder::from_style(initial());
        ApplyInternal(
            id,
            &mut child,
            Some(p),
            &crate::production_css_value::wide(CSSValueID::kInherit).unwrap(),
            16.0,
            &media,
        )
        .unwrap();
        assert!(child.GetFlowTolerance().IsInfinite());
        child.SetEffectiveZoom(2.0);
        assert_eq!(
            ApplyInternal(
                id,
                &mut child,
                Some(p),
                &crate::production_css_value::wide(CSSValueID::kInherit).unwrap(),
                16.0,
                &media
            ),
            Err(LonghandApplicationError::Unsupported(id))
        );
        ApplyInternal(
            id,
            &mut child,
            Some(p),
            &crate::production_css_value::wide(CSSValueID::kUnset).unwrap(),
            16.0,
            &media,
        )
        .unwrap();
        assert!(child.GetFlowTolerance().IsNormal());
    }
}
