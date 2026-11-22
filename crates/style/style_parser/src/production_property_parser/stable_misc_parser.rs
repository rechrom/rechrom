// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Font-synthesis and math/legacy ordinal consumers from Chromium.
#![allow(non_snake_case)]
use super::*;
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kMathDepth
            | CSSPropertyID::kWebkitBoxOrdinalGroup
            | CSSPropertyID::kFontSynthesisWeight
            | CSSPropertyID::kFontSynthesisStyle
            | CSSPropertyID::kFontSynthesisSmallCaps
            | CSSPropertyID::kScrollBehavior
    )
}
fn Integer<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    if IsMathFunction(s) {
        return ConsumeMath(
            id,
            s,
            &[crate::css_math_expression_node::CalculationResultCategory::Number],
            crate::css_math_function_value::ValueRange::Integer,
        );
    }
    let token = s.Peek();
    if token.GetType() != kNumberToken
        || token.GetNumericValueType() != NumericValueType::kIntegerValueType
    {
        return Err(invalid(id));
    }
    let n = token.NumericValue();
    s.ConsumeIncludingWhitespace();
    Ok(values::numeric(n, UnitType::kInteger))
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    if id == CSSPropertyID::kScrollBehavior {
        // css_properties.json5 generated keyword grammar, mojom enum.
        return ConsumeLiteral(id, s, mode, Grammar::Keywords(&["auto", "smooth"]));
    }
    if matches!(
        id,
        CSSPropertyID::kFontSynthesisWeight
            | CSSPropertyID::kFontSynthesisStyle
            | CSSPropertyID::kFontSynthesisSmallCaps
    ) {
        return ConsumeLiteral(
            id,
            s,
            mode,
            super::super::production_property_metadata::GrammarFor(id),
        );
    }
    if id == CSSPropertyID::kWebkitBoxOrdinalGroup {
        return ConsumeLiteral(id, s, mode, Grammar::Integer { minimum: 1 });
    }
    // css_parsing_utils.cc:6192-6223. add() retains its CSSFunctionValue.
    if s.Peek().Id() == CSSValueID::kAutoAdd {
        return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
    }
    if s.Peek().FunctionId() == Some(CSSValueID::kAdd) {
        let value = {
            let mut args = BlockGuard::new(s);
            args.ConsumeWhitespace();
            let value = Integer(id, &mut args)?;
            if !args.AtEnd() {
                return Err(invalid(id));
            }
            value
        };
        s.ConsumeWhitespace();
        return Ok(values::function(CSSValueID::kAdd, vec![value]));
    }
    Integer(id, s)
}
pub(super) fn FontSynthesis<T: TokenStreamTokenizer>(
    s: &mut Stream<T>,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    use CSSPropertyID::*;
    use CSSValueID::*;
    let id = kFontSynthesis;
    let mut enabled = [false; 3];
    if s.Peek().Id() == kNone {
        s.ConsumeIncludingWhitespace();
    } else {
        let mut any = false;
        while !s.AtEnd() {
            let slot = match s.Peek().Id() {
                kWeight => 0,
                kStyle => 1,
                kSmallCaps => 2,
                _ => break,
            };
            if enabled[slot] {
                return Err(invalid(id));
            }
            enabled[slot] = true;
            any = true;
            s.ConsumeIncludingWhitespace();
        }
        if !any {
            return Err(invalid(id));
        }
    }
    // shorthands_custom.cc:3473-3565 and generated metadata:1015-1025.
    // This checkout has exactly three longhands; position does not exist.
    for (property, enabled) in [
        kFontSynthesisWeight,
        kFontSynthesisStyle,
        kFontSynthesisSmallCaps,
    ]
    .into_iter()
    .zip(enabled)
    {
        out.push(make_expanded(
            property,
            id,
            values::identifier(if enabled { kAuto } else { kNone }),
            false,
        ));
    }
    Ok(())
}
