//! CSS half of HTML responsive-image selection.
//!
//! HTML owns `srcset` and source-size list traversal.  This module translates
//! the CSS/media portion of Blink's `SizesAttributeParser` with the canonical
//! tokenizer, media-query parser/evaluator, and `MediaValues` length resolver.
#![allow(non_snake_case)]

use crate::{
    css_math_expression_node::{CalculationResultCategory, ConsumeMathFunction},
    css_numeric_literal_value::IsLength,
    media_queries::{media_query_backend::MediaQueryFeatureFlags, MediaValues},
    parser::{
        css_parser_token::CSSParserTokenType::*, css_parser_token_stream::CSSParserTokenStream,
        css_tokenizer::CSSTokenizer, media_query_parser::MediaQueryParser,
    },
};
use foundation::{CSSValueID, StringView};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SizesEvaluation {
    Auto,
    Length(f64),
    Invalid,
    FatalInvalid,
}

fn CalculateLengthInPixels(
    stream: &mut CSSParserTokenStream<'_, CSSTokenizer>,
    values: &dyn MediaValues<Document = dom::Document>,
) -> Option<f64> {
    match stream.Peek().GetType() {
        kDimensionToken => {
            let token = stream.Peek().clone();
            if !IsLength(token.GetUnitType()) {
                return None;
            }
            let result = values.ComputeLength(token.NumericValue(), token.GetUnitType());
            if !result.is_finite() || result < 0.0 {
                return None;
            }
            stream.Consume();
            Some(result)
        }
        kFunctionToken => {
            if !matches!(
                stream.Peek().FunctionId(),
                Some(CSSValueID::kCalc | CSSValueID::kMin | CSSValueID::kMax | CSSValueID::kClamp)
            ) {
                return None;
            }
            let expression = ConsumeMathFunction(stream).ok()?;
            if expression.Category() != CalculationResultCategory::Length {
                return None;
            }
            expression
                .ComputeValue(
                    &mut |number, unit| Ok(values.ComputeLength(number, unit)),
                    None,
                )
                .ok()
                .filter(|result| result.is_finite() && *result >= 0.0)
        }
        kNumberToken if stream.Peek().NumericValue() == 0.0 => {
            stream.Consume();
            Some(0.0)
        }
        _ => None,
    }
}

// cpp: sizes_attribute_parser.cc Parse for one comma-delimited component.
pub fn EvaluateSizesComponent(
    component: &str,
    values: &dyn MediaValues<Document = dom::Document>,
) -> SizesEvaluation {
    let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(component), 0);
    stream.ConsumeWhitespace();
    if stream.Peek().Id() == CSSValueID::kAuto {
        stream.ConsumeIncludingWhitespace();
        return if stream.AtEnd() {
            SizesEvaluation::Auto
        } else {
            // Blink returns false for the whole attribute when `auto` is not
            // followed by a comma or the end of input.
            SizesEvaluation::FatalInvalid
        };
    }

    stream.EnsureLookAhead();
    let savepoint = stream.Save();
    let condition =
        MediaQueryParser::new(MediaQueryFeatureFlags::default()).ParseMediaCondition(&mut stream);
    let matches =
        crate::media_queries::media_query_backend::ForMediaValues(values).Eval(&condition);
    if !matches {
        stream.EnsureLookAhead();
        stream.Restore(savepoint);
    }

    let Some(length) = CalculateLengthInPixels(&mut stream, values) else {
        return SizesEvaluation::Invalid;
    };
    stream.ConsumeWhitespace();
    if stream.AtEnd() {
        SizesEvaluation::Length(length)
    } else {
        SizesEvaluation::Invalid
    }
}
