// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Typed literal/raw-string branches of CSSAttrType and attribute lookup.
#![allow(non_snake_case)]
use crate::{
    css_primitive_value::{StringToUnitType, UnitType, UnitTypeToString},
    parser::{
        css_parser_token::{
            BlockType, CSSParserToken, CSSParserTokenType::*, NumericSign, NumericValueType,
        },
        css_parser_token_stream::CSSParserTokenStream,
        css_tokenizer::CSSTokenizer,
    },
    production_css_value as values,
};
use crate::{
    custom_properties::VariableResolutionError,
    production_css_value::{CSSVariableData, VariableToken},
};
use foundation::{AtomicString, String, StringView};
use std::rc::Rc;

// cpp: style_cascade.cc:2439-2442. The caller selects the ultimate originating
// element and its LowercaseIfNecessary policy; no pseudo-element guess is made.
pub trait AttributeSource {
    /// A null string means no attribute; an empty string is a present value.
    fn GetAttribute(&self, local_name: &AtomicString) -> String;
}
pub trait AttributeResolutionState {
    fn SetHasAttrFunction(&self);
    fn HasAttrFunction(&self) -> bool {
        false
    }
}
pub trait AttributeVariableResolver {
    fn CSSArgumentGrammarEnabled(&self) -> bool;
    fn SetHasAttrFunction(&self);
    fn GetAttribute(&self, local_name: &AtomicString) -> String;
    fn HasAttrFunction(&self) -> bool {
        false
    }
}
pub struct AttributeResolver<'a, S, R> {
    source: &'a S,
    state: &'a R,
    argument_grammar: bool,
}
impl<'a, S, R> AttributeResolver<'a, S, R> {
    pub fn new(source: &'a S, state: &'a R, argument_grammar: bool) -> Self {
        Self {
            source,
            state,
            argument_grammar,
        }
    }
}
impl<S: AttributeSource, R: AttributeResolutionState> AttributeVariableResolver
    for AttributeResolver<'_, S, R>
{
    fn CSSArgumentGrammarEnabled(&self) -> bool {
        self.argument_grammar
    }
    fn SetHasAttrFunction(&self) {
        self.state.SetHasAttrFunction();
    }
    fn GetAttribute(&self, local_name: &AtomicString) -> String {
        self.source.GetAttribute(local_name)
    }
    fn HasAttrFunction(&self) -> bool {
        self.state.HasAttrFunction()
    }
}
/// Uses the admitted DOM owner directly. Attribute namespaces other than null
/// are excluded, matching getAttributeNS(g_null_atom, ...) in StyleCascade.
#[derive(Clone, Copy, Debug)]
pub enum AttributeType {
    RawString,
    Unit(UnitType),
}

/// Consumes the legacy `attr()` type component from StyleCascade's retained
/// token stream.  Keeping this beside `ParseAttributeValue` makes the typed
/// attribute owner responsible for both halves of CSSAttrType; the production
/// cascade adapter only forwards the translated callback.
pub fn ConsumeAttributeType(
    stream: &mut CSSParserTokenStream<'_, CSSTokenizer>,
) -> Option<AttributeType> {
    let token = stream.Peek().clone();
    let result = if token.GetType() == kDelimiterToken && token.Delimiter() == b'%' as u16 {
        Some(AttributeType::Unit(UnitType::kPercentage))
    } else if token.GetType() == kIdentToken {
        let text = token.Value().ToString();
        match text.Utf8().as_str() {
            "raw-string" => Some(AttributeType::RawString),
            "number" => Some(AttributeType::Unit(UnitType::kNumber)),
            _ => {
                let unit = StringToUnitType(&StringView::from(&text));
                (crate::css_numeric_literal_value::IsLength(unit)
                    || matches!(
                        unit,
                        UnitType::kUserUnits
                            | UnitType::kQuirkyEms
                            | UnitType::kDegrees
                            | UnitType::kRadians
                            | UnitType::kGradians
                            | UnitType::kTurns
                            | UnitType::kMilliseconds
                            | UnitType::kSeconds
                            | UnitType::kHertz
                            | UnitType::kKilohertz
                            | UnitType::kFlex
                    ))
                .then_some(AttributeType::Unit(unit))
            }
        }
    } else {
        None
    };
    if result.is_some() {
        stream.ConsumeIncludingWhitespace();
    }
    result
}
// cpp: css_attr_type.cc:23-79 ConsumeDimensionUnitType/CSSAttrType::Consume;
// css_variable_parser.cc:255-293 ConsumeAttributeReference.
pub fn ConsumeAttributeHeader(
    tokens: &[VariableToken],
) -> Result<(AtomicString, AttributeType, bool), VariableResolutionError> {
    let significant = tokens
        .iter()
        .filter(|item| !matches!(item.token.GetType(), kWhitespaceToken | kCommentToken))
        .collect::<Vec<_>>();
    let first = significant
        .first()
        .ok_or(VariableResolutionError::Invalid)?;
    if first.token.GetType() != kIdentToken {
        return Err(VariableResolutionError::Invalid);
    }
    let name =
        AtomicString::from_utf16(first.token.Value().ToString().Span16().unwrap_or_default());
    if significant.len() == 1 {
        return Ok((name, AttributeType::RawString, true));
    }
    let token = &significant[1].token;
    if token.GetType() == kFunctionToken
        && token.Value().ToString().Utf8().eq_ignore_ascii_case("type")
    {
        return Err(VariableResolutionError::Unsupported(
            "CSSAttrType::Consume CSSSyntaxDefinition::Consume/Parse",
        ));
    }
    if significant.len() != 2 {
        return Err(VariableResolutionError::Invalid);
    }
    if token.GetType() == kDelimiterToken && token.Delimiter() == b'%' as u16 {
        return Ok((name, AttributeType::Unit(UnitType::kPercentage), false));
    }
    if token.GetType() != kIdentToken {
        return Err(VariableResolutionError::Invalid);
    }
    let text = token.Value().ToString();
    if text.Utf8() == "raw-string" {
        return Ok((name, AttributeType::RawString, false));
    }
    if text.Utf8() == "number" {
        return Ok((name, AttributeType::Unit(UnitType::kNumber), false));
    }
    let mut unit = StringToUnitType(&StringView::from(&text));
    // cpp: css_primitive_value.h:295-318,332-339,357.
    if !crate::css_numeric_literal_value::IsLength(unit)
        && !matches!(
            unit,
            UnitType::kUserUnits
                | UnitType::kQuirkyEms
                | UnitType::kDegrees
                | UnitType::kRadians
                | UnitType::kGradians
                | UnitType::kTurns
                | UnitType::kMilliseconds
                | UnitType::kSeconds
                | UnitType::kHertz
                | UnitType::kKilohertz
                | UnitType::kFlex
                | UnitType::kPercentage
        )
    {
        unit = UnitType::kUnknown;
    }
    Ok((name, AttributeType::Unit(unit), false))
}
// cpp: css_attr_type.cc:81-107 CSSAttrType::Parse. These branches construct
// tokens directly; serialized lexemes are never re-tokenized for substitution.
pub fn ParseAttributeValue(
    value: &String,
    kind: AttributeType,
) -> Result<Option<Rc<CSSVariableData>>, VariableResolutionError> {
    if value.IsNull() {
        return Ok(None);
    }
    match kind {
        AttributeType::RawString => Ok(Some(StringData(value, false))),
        AttributeType::Unit(UnitType::kUnknown) => Ok(None),
        AttributeType::Unit(unit) => {
            let mut tokenizer = CSSTokenizer::new(StringView::from(value), 0);
            // CSSAttrType::Parse does not ConsumeWhitespace before the
            // ConsumeNumber call. Comments are skipped by the tokenizer.
            let token = tokenizer.TokenizeSingle();
            // ConsumeNumber math results have CSSMathFunctionValue dynamic
            // type and fail CSSAttrType's DynamicTo<CSSNumericLiteralValue>.
            // Only an actual number token can provide this literal branch.
            if token.GetType() != kNumberToken {
                return Ok(None);
            }
            let number = token.NumericValue();
            let number = if number == 0.0 { 0.0 } else { number };
            let value = values::numeric(number, unit);
            let mut token = CSSParserToken::WithNumber(
                kNumberToken,
                number,
                if number.fract() == 0.0 {
                    NumericValueType::kIntegerValueType
                } else {
                    NumericValueType::kNumberValueType
                },
                if number < 0.0 {
                    NumericSign::kMinusSign
                } else {
                    NumericSign::kNoSign
                },
            );
            if unit == UnitType::kPercentage {
                token.ConvertToPercentage();
            } else if unit != UnitType::kNumber {
                token.ConvertToDimensionWithUnit(StringView::from(
                    UnitTypeToString(unit).expect("CSSAttrType dimension"),
                ));
            }
            Ok(Some(Rc::new(CSSVariableData::FromTokens(
                vec![VariableToken {
                    token,
                    text: value.CssText(),
                }],
                false,
                true,
            ))))
        }
    }
}
pub fn StringData(value: &String, implicit_fallback: bool) -> Rc<CSSVariableData> {
    let text = if implicit_fallback {
        String::from("''")
    } else {
        crate::parser::css_markup::SerializeString(value)
    };
    let token = CSSParserToken::WithValue(
        kStringToken,
        StringView::from(value),
        BlockType::kNotBlock,
        None,
    );
    Rc::new(CSSVariableData::FromTokens(
        vec![VariableToken { token, text }],
        false,
        true,
    ))
}
