// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! longhands_custom.cc:3992-4064,5248-5259,5275-5286;
//! css_parsing_utils.cc:7885-7894. Production gates live at the parser entry.
//! Selected source audit: effective 300, mapped 291, omitted 3, remaining 6.
//! Parser 108/105/1/2; Apply/converter 93/88/1/4; typed owner 44/44/0/0;
//! reused native types 55/54/1/0. Effective excludes comments, blank lines,
//! preprocessor/namespace/visibility/brace-only lines. Omitted are DCHECK only.
//! Remaining selected lines: custom 4006-4007 FunctionLocalContext adapter;
//! generated 8272-8274 changed-zoom reconversion; converter 1843 shared length
//! context. Nested shared CSSMath/time/length consumers remain partial in the
//! ledgers and are outside this selected-line count. Flow is stable hidden.
//! Selected source ranges, under Blink core unless specified:
//! custom longhands:3992-4064,5248-5259,5275-5286;
//! custom shorthands:4154-4162; utils:4188-4225 (excluding Overflow-only
//! use counter 4212-4214),7885-7894;
//! generated out/Min/gen/.../core/css/properties/longhands.cc:
//! 7703-7708,7725-7733,8240-8245,8261-8266,8268-8281,9364-9372,9393-9401;
//! style_builder_converter.cc:228-268,1835-1844,1942-1951,3891-3895;
//! css_dynamic_range_limit_mix_value.h:18-25,27,31-36,39-40;
//! .cc:14-27,29-42,44-56. The production Rc<Value> owner retains child values;
//! Blink GC Trace methods are outside the selected owner-method count.
//! reused native flow_tolerance.h:20-31,33-50; style_interest_delay.h:17-25,28-34;
//! cc/paint/paint_flags.h:95-100,104-123,130-131 (runtime HDR methods excluded).
//! Runtime defaults: platform/runtime_enabled_features.json5:1615-1616 stable,
//! 1641-1643 experimental. InterestDelay has no runtime flag in css_properties.json5.
#![allow(non_snake_case)]
use super::*;
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kDynamicRangeLimit
            | CSSPropertyID::kFlowTolerance
            | CSSPropertyID::kInterestDelayStart
            | CSSPropertyID::kInterestDelayEnd
    )
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    match id {
        kDynamicRangeLimit => Limit(id, s),
        kFlowTolerance => ConsumeLiteral(
            id,
            s,
            mode,
            Grammar::Length {
                percent: true,
                nonnegative: true,
                quirks: false,
                keywords: &["normal", "infinite"],
            },
        ),
        kInterestDelayStart | kInterestDelayEnd => {
            if s.Peek().Id() == CSSValueID::kNormal {
                return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            ConsumeTime(id, s, true)?.ok_or_else(|| invalid(id))
        }
        _ => Err(invalid(id)),
    }
}
fn Limit<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    if matches!(
        s.Peek().Id(),
        CSSValueID::kStandard | CSSValueID::kNoLimit | CSSValueID::kConstrained
    ) {
        return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
    }
    if s.Peek().FunctionId() != Some(CSSValueID::kDynamicRangeLimitMix) {
        return Err(invalid(id));
    }
    let mut guard = RestoringBlockGuard::new(s);
    let mut limits = Vec::new();
    let mut percentages = Vec::new();
    let mut all_zero = true;
    loop {
        guard.ConsumeWhitespace();
        limits.push(Limit(id, &mut guard)?);
        guard.ConsumeWhitespace();
        let percentage = if IsMathFunction(&mut guard) {
            ConsumeMath(
                id,
                &mut guard,
                &[crate::css_math_expression_node::CalculationResultCategory::Percent],
                crate::css_math_function_value::ValueRange::NonNegative,
            )?
        } else {
            let t = guard.Peek();
            if t.GetType() != kPercentageToken || t.NumericValue() < 0.0 || t.NumericValue() > 100.0
            {
                return Err(invalid(id));
            }
            values::numeric(
                guard.ConsumeIncludingWhitespace().NumericValue(),
                UnitType::kPercentage,
            )
        };
        if let CSSValuePayload::kNumericLiteralClass(n) = percentage.Payload() {
            all_zero &= n.DoubleValue() == 0.0;
        } else {
            all_zero = false
        }
        percentages.push(percentage);
        guard.ConsumeWhitespace();
        if guard.Peek().GetType() != kCommaToken {
            if !guard.AtEnd() {
                return Err(invalid(id));
            }
            break;
        }
        guard.ConsumeIncludingWhitespace();
    }
    if all_zero {
        return Err(invalid(id));
    }
    guard.Release();
    drop(guard);
    s.ConsumeWhitespace();
    Ok(Rc::new(Value::new(
        CSSValuePayload::kDynamicRangeLimitMixClass(
            crate::production_dynamic_range_value::CSSDynamicRangeLimitMixValue::new(
                limits,
                percentages,
            ),
        ),
    )))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hidden_flow_tolerance_typed_consumer_matches_native_grammar() {
        for text in ["normal", "infinite", "20px", "5%", "calc(10px + 5%)"] {
            let text = String::from(text);
            let mut s: Stream = Stream::new(StringView::from(&text), 0);
            assert!(Consume(
                CSSPropertyID::kFlowTolerance,
                &mut s,
                CSSParserMode::kHTMLStandardMode
            )
            .is_ok());
            assert!(s.AtEnd());
        }
        for text in ["-1px", "-2%", "none", "1s"] {
            let text = String::from(text);
            let mut s: Stream = Stream::new(StringView::from(&text), 0);
            assert!(Consume(
                CSSPropertyID::kFlowTolerance,
                &mut s,
                CSSParserMode::kHTMLStandardMode
            )
            .is_err());
        }
    }
}
