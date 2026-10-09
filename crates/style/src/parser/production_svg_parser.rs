// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! SVG longhands using the shared production token/color/length consumers.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsSVGProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kFill
            | kStroke
            | kInternalVisitedFill
            | kInternalVisitedStroke
            | kStrokeWidth
            | kStrokeDashoffset
            | kStrokeDasharray
            | kCx
            | kCy
            | kR
            | kRx
            | kRy
            | kX
            | kY
            | kPathLength
            | kPaintOrder
    )
}

// css_parsing_utils.cc:1459-1500,1521-1533. SVG mode permits user units and
// number-valued math; all other categories use the shared length consumer.
pub(super) fn ConsumeLength<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    percent: bool,
    nonnegative: bool,
    keywords: &'static [&'static str],
) -> Result<Rc<Value>, PropertyParseError> {
    if mode == CSSParserMode::kSVGAttributeMode && IsMathFunction(stream) {
        if !percent {
            return Err(invalid(id));
        }
        use crate::css_math_expression_node::CalculationResultCategory as C;
        use crate::css_math_function_value::ValueRange as R;
        return ConsumeMath(
            id,
            stream,
            &[C::Length, C::Percent, C::LengthFunction, C::Number],
            if nonnegative { R::NonNegative } else { R::All },
        );
    }
    ConsumeLiteral(
        id,
        stream,
        mode,
        Grammar::Length {
            percent,
            nonnegative,
            quirks: false,
            keywords,
        },
    )
}

pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    if matches!(id, kFill | kStroke | kInternalVisitedFill | kInternalVisitedStroke) {
        // css_parsing_utils.cc:9777-9805 ConsumeSVGPaint.
        if matches!(
            stream.Peek().Id(),
            CSSValueID::kNone | CSSValueID::kContextFill | CSSValueID::kContextStroke
        ) {
            return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
        }
        if let Some(url) = ConsumeUrl(id, stream)? {
            let url = values::uri(url);
            if at_value_end(stream) {
                return Ok(url);
            }
            let fallback = if stream.Peek().Id() == CSSValueID::kNone {
                values::identifier(stream.ConsumeIncludingWhitespace().Id())
            } else {
                ConsumeColor(id, stream)?
            };
            return Ok(values::list(
                vec![url, fallback],
                values::ListSeparator::Space,
            ));
        }
        return ConsumeColor(id, stream);
    }
    if id == kPaintOrder {
        // longhands_custom.cc:8167-8226. Canonical lists retain only the first
        // component and an optional non-default second component.
        if stream.Peek().Id() == CSSValueID::kNormal {
            return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
        }
        let mut components = Vec::new();
        while !at_value_end(stream) {
            let component = stream.Peek().Id();
            if !matches!(
                component,
                CSSValueID::kFill | CSSValueID::kStroke | CSSValueID::kMarkers
            ) || components.contains(&component)
            {
                return Err(invalid(id));
            }
            components.push(stream.ConsumeIncludingWhitespace().Id());
        }
        let Some(&first) = components.first() else {
            return Err(invalid(id));
        };
        let mut canonical = vec![values::identifier(first)];
        if components.get(1).is_some_and(|&second| {
            if first == CSSValueID::kMarkers {
                second == CSSValueID::kStroke
            } else {
                second == CSSValueID::kMarkers
            }
        }) {
            canonical.push(values::identifier(components[1]));
        }
        return Ok(values::list(canonical, values::ListSeparator::Space));
    }
    if id == kStrokeDasharray {
        // longhands_custom.cc:9565-9594; comma or whitespace, no trailing comma.
        if stream.Peek().Id() == CSSValueID::kNone {
            return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
        }
        let mut dashes = Vec::new();
        loop {
            dashes.push(ConsumeLength(
                id,
                stream,
                CSSParserMode::kSVGAttributeMode,
                true,
                true,
                &[],
            )?);
            if stream.Peek().GetType() == kCommaToken {
                stream.ConsumeIncludingWhitespace();
            } else if at_value_end(stream) {
                break;
            }
        }
        return Ok(values::list(dashes, values::ListSeparator::Comma));
    }
    if id == kPathLength {
        // longhands_custom.cc:3560-3570: ordinary nonnegative length or none.
        return ConsumeLength(id, stream, mode, false, true, &["none"]);
    }
    // longhands_custom.cc:3528-3551,8438-8446,8642-8672,9605-9613,
    // 9667-9676,12665-12689: geometry and stroke lengths override SVG mode.
    ConsumeLength(
        id,
        stream,
        CSSParserMode::kSVGAttributeMode,
        true,
        matches!(id, kStrokeWidth | kR | kRx | kRy),
        if matches!(id, kRx | kRy) {
            &["auto"]
        } else {
            &[]
        },
    )
}
