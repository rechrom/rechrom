// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: property_registration.cc:82-162 (descriptor conversion/independence).
#![allow(non_snake_case)]
use crate::{
    css_primitive_value::UnitType,
    css_syntax_definition::{CSSSyntaxDefinition, SyntaxValueError},
    css_syntax_string_parser::CSSSyntaxStringParser,
    css_value::CSSValuePayload,
    parser::{
        at_rule_descriptors::AtRuleDescriptorID as D,
        css_parser_context::CSSParserContext,
        css_parser_token::CSSParserTokenType::*,
        css_parser_token_stream::{CSSParserTokenStream, TokenStreamTokenizer},
        production_property_parser::ValidateVariableTokensWithArgumentGrammar,
    },
    production_css_value as values,
    production_style_sheet::Backend,
};
use foundation::{CSSValueID, String};
use std::rc::Rc;
pub fn ConvertSyntax(value: Option<&values::Value>) -> Option<CSSSyntaxDefinition> {
    match value?.Payload() {
        CSSValuePayload::kStringClass(value) => CSSSyntaxStringParser::new(&value.0).Parse(),
        _ => None,
    }
}
pub fn ConvertInherits(value: Option<&values::Value>) -> Option<bool> {
    match value?.Payload() {
        CSSValuePayload::kIdentifierClass(value) if value.0 == CSSValueID::kTrue => Some(true),
        CSSValuePayload::kIdentifierClass(value) if value.0 == CSSValueID::kFalse => Some(false),
        _ => None,
    }
}
pub fn ComputationallyIndependent(value: &values::Value) -> bool {
    match value.Payload() {
        CSSValuePayload::kUnparsedDeclarationClass(value) => !value.data.NeedsVariableResolution(),
        CSSValuePayload::kValueListClass(value) => {
            value.values.iter().all(|v| ComputationallyIndependent(v))
        }
        CSSValuePayload::kNumericLiteralClass(value) => {
            if !value.IsLength() {
                return true;
            }
            let unit = value.GetType();
            matches!(
                unit,
                UnitType::kPixels
                    | UnitType::kCentimeters
                    | UnitType::kMillimeters
                    | UnitType::kInches
                    | UnitType::kPoints
                    | UnitType::kPicas
                    | UnitType::kQuarterMillimeters
            ) || (UnitType::kViewportWidth as i32..=UnitType::kDynamicViewportMax as i32)
                .contains(&(unit as i32))
        }
        _ => true,
    }
}
pub fn ConvertInitial(
    value: Option<&values::Value>,
    syntax: &CSSSyntaxDefinition,
) -> Result<Option<Rc<values::Value>>, SyntaxValueError> {
    let Some(value) = value else {
        return if syntax.IsUniversal() {
            Ok(None)
        } else {
            Err(SyntaxValueError::Invalid)
        };
    };
    let CSSValuePayload::kUnparsedDeclarationClass(value) = value.Payload() else {
        return Err(SyntaxValueError::Invalid);
    };
    let initial = syntax.ParseTokens(&value.data)?;
    if !ComputationallyIndependent(&initial) {
        return Err(SyntaxValueError::Invalid);
    }
    Ok(Some(initial))
}
// Parser-only registration request. A Document registry must apply cascade/order
// and own registered interpolation/computed value semantics before consuming it.
pub struct PropertyRegistrationEffect {
    pub name: foundation::AtomicString,
    pub syntax: CSSSyntaxDefinition,
    pub syntax_text: String,
    pub initial_source: Option<Rc<values::CSSVariableData>>,
    pub inherits: bool,
    pub initial: Option<Rc<values::Value>>,
    pub source_order: u32,
    pub media: Vec<Rc<crate::production_style_sheet::QuerySet>>,
    pub layers: Vec<crate::production_style_sheet::LayerSegment>,
    pub source_range: crate::production_style_sheet::SourceRange,
}
pub fn ParseAtPropertyDescriptor<T: TokenStreamTokenizer>(
    id: D,
    s: &mut CSSParserTokenStream<'_, T>,
    context: &CSSParserContext<Backend>,
) -> Option<Rc<values::Value>> {
    s.ConsumeWhitespace();
    let result = match id {
        D::Syntax => {
            if s.Peek().GetType() != kStringToken {
                return None;
            }
            let text = s.ConsumeIncludingWhitespace().Value().ToString();
            CSSSyntaxStringParser::new(&text).Parse()?;
            values::string(text)
        }
        D::Inherits => {
            if s.Peek().GetType() != kIdentToken
                || !matches!(s.Peek().Id(), CSSValueID::kTrue | CSSValueID::kFalse)
            {
                return None;
            }
            values::identifier(s.ConsumeIncludingWhitespace().Id())
        }
        D::InitialValue => {
            let mut tokens = Vec::new();
            let mut important = None;
            crate::production_container_parser::CollectComponentTokens(
                s,
                false,
                &mut tokens,
                Some(&mut important),
            );
            if important.is_some()
                || !ValidateVariableTokensWithArgumentGrammar(&tokens, false, true, false)
            {
                return None;
            }
            let first = tokens
                .iter()
                .position(|t| !matches!(t.token.GetType(), kWhitespaceToken | kCommentToken))
                .unwrap_or(tokens.len());
            let last = tokens
                .iter()
                .rposition(|t| !matches!(t.token.GetType(), kWhitespaceToken | kCommentToken))
                .map_or(first, |i| i + 1);
            values::unparsed(
                values::CSSVariableData::FromTokens(tokens[first..last].to_vec(), false, false),
                context.Mode(),
            )
        }
        _ => return None,
    };
    s.AtEnd().then_some(result)
}
