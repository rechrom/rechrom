// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Production declaration/value slice of CSSParserImpl/CSSPropertyParser.
//! Source: css_parser_impl.cc ConsumeDeclaration / ConsumeDeclarationValue;
//! css_property_parser.cc ParseValueStart / ParseCSSWideKeyword / AddProperty.
//! Grammar tables are generated from current Chromium inputs. Unsupported
//! consumers produce an explicit typed diagnostic, never an opaque CSS value.
#![allow(non_snake_case)]
use super::css_parser_mode::CSSParserMode;
use super::css_parser_token::{BlockType, CSSParserTokenType::*, NumericValueType};
use super::css_parser_token_stream::{
    BlockGuard, CSSParserTokenStream, RestoringBlockGuard, TokenStreamTokenizer,
};
use super::css_tokenizer::CSSTokenizer;
use super::production_property_metadata::{GrammarFor, MatchingShorthands, ShorthandFor};
use crate::css_primitive_value::UnitType;
use crate::css_property_name::CSSPropertyName;
use crate::css_property_names::{FindProperty, ResolveCSSPropertyID};
use crate::css_value::CSSValuePayload;
use crate::css_value_keywords::GetCSSValueName;
use crate::production_css_value::{self as values, PropertyValue, Value};
use foundation::{CSSPropertyID, CSSValueID, Color, String, StringView};
use std::rc::Rc;

#[path = "production_corner_parser.rs"]
mod corner_parser;
#[path = "production_initial_scope_parser.rs"]
mod initial_scope_parser;

#[path = "production_anchor_parser.rs"]
mod anchor_parser;
#[path = "production_column_rule_parser.rs"]
mod column_rule_parser;
#[path = "production_layout_misc_parser.rs"]
mod layout_misc_parser;
#[path = "production_line_parser.rs"]
mod line_parser;
#[path = "production_list_counter_parser.rs"]
mod list_counter_parser;
#[path = "production_logical_border_parser.rs"]
mod logical_border_parser;
#[path = "production_palette_internal_parser.rs"]
mod palette_internal_parser;
#[path = "production_reflect_parser.rs"]
mod reflect_parser;
#[path = "production_rule_inset_parser.rs"]
mod rule_inset_parser;
#[path = "production_scroll_parser.rs"]
mod scroll_parser;
#[path = "production_stable_misc_parser.rs"]
mod stable_misc_parser;
#[path = "production_svg_parser.rs"]
mod svg_parser;
#[path = "production_svg_presentation_parser.rs"]
mod svg_presentation_parser;
#[path = "production_text_box_parser.rs"]
mod text_box_parser;
#[path = "production_text_parser.rs"]
mod text_parser;
#[path = "production_timeline_parser.rs"]
mod timeline_parser;
#[path = "production_transform_parser.rs"]
mod transform_parser;
#[path = "production_typography_parser.rs"]
mod typography_parser;
#[path = "production_viewport_parser.rs"]
mod viewport_parser;

#[path = "production_font_parser.rs"]
mod font_parser;

#[path = "production_border_image_parser.rs"]
mod border_image_parser;
#[path = "production_color_ui_parser.rs"]
mod color_ui_parser;
#[path = "production_effects_parser.rs"]
mod effects_parser;
#[path = "production_grid_lanes_parser.rs"]
mod grid_lanes_parser;
#[path = "production_grid_parser.rs"]
mod grid_parser;
#[path = "production_interaction_parser.rs"]
mod interaction_parser;
#[path = "production_mask_parser.rs"]
mod mask_parser;
#[path = "production_motion_parser.rs"]
mod motion_parser;
#[path = "production_render_delay_parser.rs"]
mod render_delay_parser;
#[path = "production_timeline_trigger_parser.rs"]
mod timeline_trigger_parser;
#[path = "production_view_transition_parser.rs"]
mod view_transition_parser;

type Stream<'a, T = CSSTokenizer> = CSSParserTokenStream<'a, T>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyParseErrorKind {
    Invalid,
    Unsupported,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyParseError {
    pub property: CSSPropertyID,
    pub name: std::string::String,
    pub offset: u32,
    pub kind: PropertyParseErrorKind,
    /// Exact untranslated consumer, useful in the production migration ledger.
    pub operation: &'static str,
    pub source_file: &'static str,
    pub source_line: u32,
}
pub struct ParsedDeclarations {
    pub properties: Vec<PropertyValue>,
    /// Each expanded longhand keeps its original declaration's UTF-16 offset.
    pub source_offsets: Vec<u32>,
    pub errors: Vec<PropertyParseError>,
}
#[derive(Clone, Copy)]
pub(super) enum Grammar {
    Keywords(&'static [&'static str]),
    Number {
        nonnegative: bool,
    },
    Integer {
        minimum: i32,
    },
    Length {
        percent: bool,
        nonnegative: bool,
        quirks: bool,
        keywords: &'static [&'static str],
    },
    Color,
    Content,
    Unsupported,
}
fn error(
    id: CSSPropertyID,
    kind: PropertyParseErrorKind,
    operation: &'static str,
) -> PropertyParseError {
    PropertyParseError {
        property: id,
        name: if id == CSSPropertyID::kInvalid {
            std::string::String::new()
        } else {
            crate::css_property_names::GetPropertyName(id).to_owned()
        },
        offset: 0,
        kind,
        operation,
        source_file: super::production_property_metadata::ParserSourceFor(id).0,
        source_line: super::production_property_metadata::ParserSourceFor(id).1,
    }
}
fn invalid(id: CSSPropertyID) -> PropertyParseError {
    error(
        id,
        PropertyParseErrorKind::Invalid,
        "CSSPropertyParser::ParseValueStart",
    )
}
fn unsupported(id: CSSPropertyID, operation: &'static str) -> PropertyParseError {
    error(id, PropertyParseErrorKind::Unsupported, operation)
}
fn matches(id: CSSValueID, words: &[&str]) -> bool {
    id != CSSValueID::kInvalid && words.contains(&GetCSSValueName(id))
}
fn is_function<T: TokenStreamTokenizer>(stream: &mut Stream<T>) -> bool {
    stream.Peek().GetType() == kFunctionToken
}

// cpp: css_parser_impl.cc ConsumeDeclaration. Semicolons inside a component
// block or string do not split declarations. The original tokenizer supplies
// decoded identifier/property names and exact UTF-16 source offsets.
pub fn ParseDeclarationList(text: &String, mode: CSSParserMode) -> ParsedDeclarations {
    let mut result = ParsedDeclarations {
        properties: Vec::new(),
        source_offsets: Vec::new(),
        errors: Vec::new(),
    };
    let mut tokenizer = CSSTokenizer::new(StringView::from(text), 0);
    let mut start = 0;
    let mut depth: u32 = 0;
    loop {
        let token = tokenizer.TokenizeSingle();
        let token_start = tokenizer.PreviousOffset();
        let ty = token.GetType();
        if ty == kEOFToken || (ty == kSemicolonToken && depth == 0) {
            ParseDeclarationRange(text, start, token_start, mode, &mut result);
            start = tokenizer.Offset();
            if ty == kEOFToken {
                break;
            }
        }
        match token.GetBlockType() {
            BlockType::kBlockStart => depth += 1,
            BlockType::kBlockEnd => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    result
}
fn ParseDeclarationRange(
    text: &String,
    start: u32,
    end: u32,
    mode: CSSParserMode,
    out: &mut ParsedDeclarations,
) {
    let view = StringView::from(text).Substring(start, end - start);
    let mut stream: Stream = Stream::new(view, 0);
    stream.ConsumeWhitespace();
    if stream.AtEnd() {
        return;
    }
    let offset = start + stream.Offset();
    let mut failure = invalid(CSSPropertyID::kInvalid);
    failure.offset = offset;
    if stream.Peek().GetType() != kIdentToken {
        out.errors.push(failure);
        return;
    }
    let name = stream.ConsumeIncludingWhitespace().Value().ToString();
    failure.name = name.Utf8();
    if stream.Peek().GetType() != kColonToken {
        out.errors.push(failure);
        return;
    }
    stream.ConsumeIncludingWhitespace();
    let property_name = name.Utf8();
    let id = FindProperty(property_name.to_ascii_lowercase().as_bytes())
        .map(|property| {
            super::production_property_metadata::UnresolvedPropertyIDFromInteger(
                property.id_and_exposed_bit
                    & !crate::css_property_names::kNotKnownExposedPropertyBit,
            )
        })
        .unwrap_or(CSSPropertyID::kInvalid);
    if id == CSSPropertyID::kInvalid
        && !(property_name.starts_with("--") && property_name.chars().count() >= 3)
    {
        out.errors.push(failure);
        return;
    }
    let value = stream.RemainingText().ToString();
    let parsed = if property_name.starts_with("--") && property_name.chars().count() >= 3 {
        ParseCustomProperty(&property_name, &value, false, mode).map(|p| vec![p])
    } else {
        ParseProperty(id, &value, false, mode)
    };
    match parsed {
        Ok(properties) => {
            out.source_offsets
                .extend(std::iter::repeat(offset).take(properties.len()));
            out.properties.extend(properties);
        }
        Err(mut failure) => {
            failure.offset = offset;
            failure.name = property_name;
            out.errors.push(failure);
        }
    }
}

// cpp: css_property_parser.cc ParseValueStart, ParseCSSWideKeyword, AddProperty.
// A failed shorthand parse contributes no longhands (transactional expansion).
pub fn ParseProperty(
    unresolved: CSSPropertyID,
    text: &String,
    important: bool,
    mode: CSSParserMode,
) -> Result<Vec<PropertyValue>, PropertyParseError> {
    let id = ResolveCSSPropertyID(unresolved);
    if id == CSSPropertyID::kInvalid || id == CSSPropertyID::kVariable {
        return Err(unsupported(
            id,
            "CSSVariableParser::ParseDeclarationIncludingCSSWide",
        ));
    }
    if !crate::production_corner_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / CSSCornersShorthand",
        ));
    }
    if !crate::production_interaction_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / interaction runtime flags",
        ));
    }
    if !crate::production_line_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / CSSLineClamp runtime flags",
        ));
    }
    if !crate::production_typography_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / typography runtime flags",
        ));
    }
    if !crate::production_render_delay_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / CSSGridLanesLayout",
        ));
    }
    if !crate::production_text_box_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / text box runtime flags",
        ));
    }
    if palette_internal_parser::IsUAOnly(id) && mode != CSSParserMode::kUASheetMode {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / kUA internal property",
        ));
    }
    let (data, parsed_important) = ParseVariableData(text, true, true).map_err(|_| invalid(id))?;
    if data.NeedsVariableResolution() {
        let value = values::unparsed(data, mode);
        let important = important || parsed_important;
        let longhands = ShorthandFor(id);
        return Ok(if longhands.is_empty() {
            vec![make_property(id, value, important)]
        } else {
            let pending = values::pending_substitution(unresolved, value);
            longhands
                .iter()
                .map(|&longhand| {
                    let mut p = make_expanded(longhand, id, pending.clone(), false);
                    if important {
                        p.SetImportant();
                    }
                    p
                })
                .collect()
        });
    }
    let mut stream: Stream = Stream::new(StringView::from(text), 0);
    ParsePropertyStream(unresolved, important, mode, true, &mut stream)
}

fn ParsePropertyStream<T: TokenStreamTokenizer>(
    unresolved: CSSPropertyID,
    important: bool,
    mode: CSSParserMode,
    allow_important: bool,
    stream: &mut Stream<T>,
) -> Result<Vec<PropertyValue>, PropertyParseError> {
    let id = ResolveCSSPropertyID(unresolved);
    if !crate::production_corner_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / CSSCornersShorthand",
        ));
    }
    if !crate::production_interaction_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / interaction runtime flags",
        ));
    }
    if !crate::production_line_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / CSSLineClamp runtime flags",
        ));
    }
    if !crate::production_typography_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / typography runtime flags",
        ));
    }
    if !crate::production_render_delay_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / CSSGridLanesLayout",
        ));
    }
    if !crate::production_text_box_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / text box runtime flags",
        ));
    }
    if palette_internal_parser::IsUAOnly(id) && mode != CSSParserMode::kUASheetMode {
        return Err(unsupported(
            id,
            "CSSProperty::Exposure / kUA internal property",
        ));
    }
    stream.ConsumeWhitespace();
    let mut parsed = Vec::new();
    let first = stream.Peek().Id();
    if let Some(value) = values::wide(first) {
        stream.ConsumeIncludingWhitespace();
        let longhands = ShorthandFor(id);
        if longhands.is_empty() {
            parsed.push(make_property(id, value, false));
        } else {
            for &longhand in longhands {
                parsed.push(make_expanded(longhand, id, value.clone(), false));
            }
        }
    } else if crate::properties::css_property::CSSProperty::Get(id).IsShorthand() {
        ParseShorthand(id, unresolved, stream, mode, &mut parsed)?;
    } else {
        let value = ConsumeLonghandWithAlias(id, stream, mode, unresolved != id)?;
        parsed.push(make_property(id, value, false));
    }
    // cpp: css_parsing_utils.cc MaybeConsumeImportant. CSS whitespace/comments
    // are handled by the source token stream, including escaped IMPORTANT.
    let mut important = important;
    if !allow_important
        && stream.Peek().GetType() == kDelimiterToken
        && stream.Peek().Delimiter() == b'!' as u16
    {
        return Err(invalid(id));
    }
    if stream.Peek().GetType() == kDelimiterToken && stream.Peek().Delimiter() == b'!' as u16 {
        stream.ConsumeIncludingWhitespace();
        if stream.Peek().GetType() != kIdentToken
            || !stream
                .Peek()
                .Value()
                .ToString()
                .Utf8()
                .eq_ignore_ascii_case("important")
        {
            return Err(invalid(id));
        }
        stream.ConsumeIncludingWhitespace();
        important = true;
    }
    if !stream.AtEnd() {
        return Err(invalid(id));
    }
    if important {
        for property in &mut parsed {
            property.SetImportant();
        }
    }
    Ok(parsed)
}
// cpp: css_variable_parser.cc:32-42,57-76,174-209,511-754.
// The grammar runs over tokenizer records, including comments. Leading/trailing
// trivia is discarded exactly once, while trivia inside a value is retained.
pub fn ParseCustomProperty(
    name: &str,
    text: &String,
    important: bool,
    mode: CSSParserMode,
) -> Result<PropertyValue, PropertyParseError> {
    ParseCustomPropertyWithArgumentGrammar(name, text, important, mode, false)
}
/// Explicit CSSArgumentGrammar context for token declarations admitted by a
/// caller enabling Chromium's test-only runtime feature. Stable entry is false.
pub fn ParseCustomPropertyWithArgumentGrammar(
    name: &str,
    text: &String,
    important: bool,
    mode: CSSParserMode,
    argument_grammar: bool,
) -> Result<PropertyValue, PropertyParseError> {
    if !name.starts_with("--") || name.chars().count() < 3 {
        return Err(invalid(CSSPropertyID::kVariable));
    }
    let (data, annotation) =
        ParseVariableDataWithArgumentGrammar(text, true, false, argument_grammar)?;
    let significant = data
        .tokens
        .iter()
        .filter(|item| !IsTrivia(&item.token))
        .collect::<Vec<_>>();
    let value = if significant.len() == 1 {
        values::wide(significant[0].token.Id()).unwrap_or_else(|| values::unparsed(data, mode))
    } else {
        values::unparsed(data, mode)
    };
    Ok(PropertyValue::new(
        &CSSPropertyName::custom(foundation::AtomicString::from_str(name)),
        value,
        important || annotation,
        false,
        0,
        false,
    ))
}
fn IsTrivia(token: &super::css_parser_token::CSSParserToken) -> bool {
    matches!(token.GetType(), kWhitespaceToken | kCommentToken)
}
/// Finds the closing token of a component block; EOF closes unfinished blocks
/// according to CSS Syntax (the tokenizer has already classified mismatches).
pub(crate) fn VariableBlockEnd(tokens: &[values::VariableToken], start: usize) -> usize {
    let mut depth = 0;
    for (index, item) in tokens.iter().enumerate().skip(start + 1) {
        match item.token.GetBlockType() {
            BlockType::kBlockStart => depth += 1,
            BlockType::kBlockEnd if depth == 0 => return index,
            BlockType::kBlockEnd => depth -= 1,
            _ => {}
        }
    }
    tokens.len()
}
pub(crate) fn VariableArguments(
    tokens: &[values::VariableToken],
) -> Option<(std::string::String, Option<&[values::VariableToken]>)> {
    let first = tokens.iter().position(|item| !IsTrivia(&item.token))?;
    let token = &tokens[first].token;
    let name = token.Value().ToString().Utf8();
    if token.GetType() != kIdentToken || !name.starts_with("--") || name.chars().count() < 3 {
        return None;
    }
    let next = (first + 1..tokens.len()).find(|&i| !IsTrivia(&tokens[i].token));
    match next {
        None => Some((name, None)),
        Some(i) if tokens[i].token.GetType() == kCommaToken => Some((name, Some(&tokens[i + 1..]))),
        _ => None,
    }
}
// cpp: css_syntax_definition.cc:26-168,284-305; css_attr_type.cc:57-86.
// Declaration validation needs only Consume's token grammar and ContainsUrlComponent,
// not CSSSyntaxDefinition::Parse or a fabricated registered syntax value.
fn ValidateAttributeSyntax(tokens: &[values::VariableToken]) -> bool {
    let mut index = 0;
    let skip = |index: &mut usize| {
        while *index < tokens.len() && IsTrivia(&tokens[*index].token) {
            *index += 1;
        }
    };
    let delimiter = |index: usize, c: u8| {
        tokens.get(index).is_some_and(|v| {
            v.token.GetType() == kDelimiterToken && v.token.Delimiter() == c as u16
        })
    };
    skip(&mut index);
    if delimiter(index, b'*') {
        index += 1;
        skip(&mut index);
        return index == tokens.len();
    }
    loop {
        let Some(token) = tokens.get(index).map(|v| &v.token) else {
            return false;
        };
        let transform_list;
        if delimiter(index, b'<') {
            index += 1;
            let Some(token) = tokens.get(index).map(|v| &v.token) else {
                return false;
            };
            if token.GetType() != kIdentToken {
                return false;
            }
            let name = token.Value().ToString().Utf8();
            if !matches!(
                name.as_str(),
                "angle"
                    | "color"
                    | "custom-ident"
                    | "image"
                    | "integer"
                    | "length"
                    | "length-percentage"
                    | "number"
                    | "percentage"
                    | "resolution"
                    | "string"
                    | "time"
                    | "transform-function"
                    | "transform-list"
            ) {
                // <url> parses as a syntax component but CSSAttrType rejects it.
                return false;
            }
            transform_list = name == "transform-list";
            index += 1;
            if !delimiter(index, b'>') {
                return false;
            }
            index += 1;
        } else {
            if token.GetType() != kIdentToken
                || values::wide(token.Id()).is_some()
                || token
                    .Value()
                    .ToString()
                    .Utf8()
                    .eq_ignore_ascii_case("default")
            {
                return false;
            }
            transform_list = false;
            index += 1;
        }
        if delimiter(index, b'#') || delimiter(index, b'+') {
            if transform_list {
                return false;
            }
            index += 1;
        }
        skip(&mut index);
        if index == tokens.len() {
            return true;
        }
        if !delimiter(index, b'|') {
            return false;
        }
        index += 1;
        skip(&mut index);
    }
}
// cpp: css_variable_parser.cc:202-293. These references have declaration-value
// fallbacks; punctuation in ordinary nested component blocks is independent.
fn ValidateEnvironmentOrAttributeReference(
    tokens: &[values::VariableToken],
    attribute: bool,
    argument_grammar: bool,
) -> bool {
    let mut index = 0;
    let skip = |index: &mut usize| {
        while *index < tokens.len() && IsTrivia(&tokens[*index].token) {
            *index += 1;
        }
    };
    skip(&mut index);
    if tokens
        .get(index)
        .is_none_or(|v| v.token.GetType() != kIdentToken)
    {
        return false;
    }
    index += 1;
    skip(&mut index);
    if attribute {
        if let Some(token) = tokens.get(index).map(|v| &v.token) {
            if token.GetType() == kIdentToken
                || token.GetType() == kDelimiterToken && token.Delimiter() == b'%' as u16
            {
                // CSSAttrType accepts any ident as a dimension unit, retaining
                // kUnknown for units that cannot subsequently parse a value.
                index += 1;
                skip(&mut index);
            } else if token.FunctionId() == Some(CSSValueID::kType) {
                let end = VariableBlockEnd(tokens, index);
                if !ValidateAttributeSyntax(&tokens[index + 1..end]) {
                    return false;
                }
                index = if end == tokens.len() { end } else { end + 1 };
                skip(&mut index);
            }
        }
    } else {
        // ViewportSegments is stable in runtime_enabled_features.json5:6687.
        while let Some(token) = tokens.get(index).map(|v| &v.token) {
            if token.GetType() != kNumberToken {
                break;
            }
            if token.GetNumericValueType() != NumericValueType::kIntegerValueType
                || token.NumericValue() < 0.0
            {
                return false;
            }
            index += 1;
            skip(&mut index);
        }
    }
    if index == tokens.len() {
        return true;
    }
    if tokens[index].token.GetType() != kCommaToken {
        return false;
    }
    ValidateVariableTokensWithArgumentGrammar(&tokens[index + 1..], false, true, argument_grammar)
}
// cpp: css_variable_parser.cc:110-159. Split only top-level comma tokens;
// punctuation in either argument is checked as a declaration value.
fn ValidateCommonArgumentGrammar(tokens: &[values::VariableToken], argument_grammar: bool) -> bool {
    let mut index = 0;
    let mut comma = tokens.len();
    while index < tokens.len() {
        let token = &tokens[index].token;
        if token.GetType() == kCommaToken {
            comma = index;
            break;
        }
        if token.GetBlockType() == BlockType::kBlockStart {
            let end = VariableBlockEnd(tokens, index);
            index = if end == tokens.len() { end } else { end + 1 };
        } else {
            index += 1;
        }
    }
    let first = &tokens[..comma];
    first.iter().any(|v| !IsTrivia(&v.token))
        && ValidateVariableTokensWithArgumentGrammar(first, false, true, argument_grammar)
        && (comma == tokens.len()
            || ValidateVariableTokensWithArgumentGrammar(
                &tokens[comma + 1..],
                false,
                true,
                argument_grammar,
            ))
}
pub(crate) fn ValidateVariableTokensWithArgumentGrammar(
    tokens: &[values::VariableToken],
    restricted: bool,
    declaration_level: bool,
    argument_grammar: bool,
) -> bool {
    let mut index = 0;
    let mut components = 0;
    let mut brace = false;
    while index < tokens.len() {
        let token = &tokens[index].token;
        if IsTrivia(token) {
            index += 1;
            continue;
        }
        components += 1;
        brace |= token.GetType() == kLeftBraceToken;
        if restricted && brace && components > 1 {
            return false;
        }
        if token.GetBlockType() == BlockType::kBlockStart {
            let end = VariableBlockEnd(tokens, index);
            let inner = &tokens[index + 1..end];
            if token.FunctionId() == Some(CSSValueID::kVar) {
                let Some((_, fallback)) = VariableArguments(inner) else {
                    return false;
                };
                if let Some(fallback) = fallback {
                    if !ValidateVariableTokensWithArgumentGrammar(
                        fallback,
                        false,
                        true,
                        argument_grammar,
                    ) {
                        return false;
                    }
                }
            } else if matches!(
                token.FunctionId(),
                Some(CSSValueID::kAttr | CSSValueID::kEnv)
            ) {
                let valid = if token.FunctionId() == Some(CSSValueID::kAttr) && argument_grammar {
                    ValidateCommonArgumentGrammar(inner, argument_grammar)
                } else {
                    ValidateEnvironmentOrAttributeReference(
                        inner,
                        token.FunctionId() == Some(CSSValueID::kAttr),
                        argument_grammar,
                    )
                };
                if !valid {
                    return false;
                }
            } else if !ValidateVariableTokensWithArgumentGrammar(
                inner,
                false,
                false,
                argument_grammar,
            ) {
                return false;
            }
            index = if end == tokens.len() { end } else { end + 1 };
        } else {
            if matches!(
                token.GetType(),
                kBadStringToken
                    | kBadUrlToken
                    | kRightParenthesisToken
                    | kRightBraceToken
                    | kRightBracketToken
            ) || declaration_level
                && (token.GetType() == kSemicolonToken
                    || token.GetType() == kDelimiterToken && token.Delimiter() == b'!' as u16)
            {
                return false;
            }
            index += 1;
        }
    }
    true
}
fn ParseVariableData(
    text: &String,
    allow_important: bool,
    restricted: bool,
) -> Result<(values::CSSVariableData, bool), PropertyParseError> {
    ParseVariableDataWithArgumentGrammar(text, allow_important, restricted, false)
}
fn ParseVariableDataWithArgumentGrammar(
    text: &String,
    allow_important: bool,
    restricted: bool,
    argument_grammar: bool,
) -> Result<(values::CSSVariableData, bool), PropertyParseError> {
    if text.length() as usize > values::CSSVariableData::MAX_VARIABLE_BYTES {
        return Err(invalid(CSSPropertyID::kVariable));
    }
    let mut tokenizer = CSSTokenizer::new(StringView::from(text), 0);
    let mut tokens = Vec::new();
    let mut depth = 0usize;
    let mut annotation = None;
    loop {
        let token = tokenizer.TokenizeSingleWithComments();
        if token.GetType() == kEOFToken {
            break;
        }
        if depth == 0 && token.GetType() == kDelimiterToken && token.Delimiter() == b'!' as u16 {
            annotation = Some(tokens.len());
        }
        match token.GetBlockType() {
            BlockType::kBlockStart => depth += 1,
            BlockType::kBlockEnd => depth = depth.saturating_sub(1),
            _ => {}
        }
        let start = tokenizer.PreviousOffset();
        let lexeme = StringView::from(text)
            .Substring(start, tokenizer.Offset() - start)
            .ToString();
        tokens.push(values::VariableToken {
            token,
            text: lexeme,
        });
    }
    let mut important = false;
    if let Some(index) = annotation {
        let suffix = tokens[index + 1..]
            .iter()
            .filter(|item| !IsTrivia(&item.token))
            .collect::<Vec<_>>();
        if !allow_important
            || suffix.len() != 1
            || suffix[0].token.GetType() != kIdentToken
            || !suffix[0]
                .token
                .Value()
                .ToString()
                .Utf8()
                .eq_ignore_ascii_case("important")
        {
            return Err(invalid(CSSPropertyID::kVariable));
        }
        important = true;
        tokens.truncate(index);
    }
    let first = tokens
        .iter()
        .position(|item| !IsTrivia(&item.token))
        .unwrap_or(tokens.len());
    let last = tokens
        .iter()
        .rposition(|item| !IsTrivia(&item.token))
        .map_or(first, |i| i + 1);
    let tokens = tokens[first..last].to_vec();
    if !ValidateVariableTokensWithArgumentGrammar(&tokens, restricted, true, argument_grammar) {
        return Err(invalid(CSSPropertyID::kVariable));
    }
    Ok((
        values::CSSVariableData::FromTokens(tokens, false, false),
        important,
    ))
}

/// Replays substituted tokenizer output through the same property consumers.
/// No joined string is re-tokenized, so adjacent number/ident tokens stay apart.
pub fn ParsePropertyTokens(
    unresolved: CSSPropertyID,
    data: &values::CSSVariableData,
    mode: CSSParserMode,
) -> Result<Vec<PropertyValue>, PropertyParseError> {
    let mut stream = CSSParserTokenStream::FromTokenizer(VariableTokenReplay::new(data));
    ParsePropertyStream(unresolved, false, mode, false, &mut stream)
}
pub(crate) struct VariableTokenReplay {
    records: Vec<(super::css_parser_token::CSSParserToken, u32)>,
    text: StringView,
    index: usize,
    offset: u32,
    previous: u32,
    count: u32,
    unicode: bool,
}
impl VariableTokenReplay {
    pub(crate) fn new(data: &values::CSSVariableData) -> Self {
        let mut offset = 0;
        let records = data
            .tokens
            .iter()
            .map(|item| {
                offset += item.text.length();
                (item.token.clone(), offset)
            })
            .collect();
        Self {
            records,
            text: StringView::from(&data.original_text),
            index: 0,
            offset: 0,
            previous: 0,
            count: 0,
            unicode: false,
        }
    }
    fn emit(&mut self) -> super::css_parser_token::CSSParserToken {
        self.previous = self.offset;
        self.count += 1;
        if let Some((token, end)) = self.records.get(self.index) {
            self.offset = *end;
            self.index += 1;
            token.clone()
        } else {
            super::css_parser_token::CSSParserToken::new(kEOFToken, BlockType::kNotBlock)
        }
    }
}
impl TokenStreamTokenizer for VariableTokenReplay {
    fn new(text: StringView, offset: u32) -> Self {
        assert!(
            text.IsEmpty() && offset == 0,
            "replay requires existing tokenizer output"
        );
        Self::new(&values::CSSVariableData::FromTokens(
            Vec::new(),
            false,
            false,
        ))
    }
    fn TokenizeSingle(&mut self) -> super::css_parser_token::CSSParserToken {
        loop {
            let token = self.emit();
            if token.GetType() != kCommentToken {
                return token;
            }
        }
    }
    fn TokenizeSingleWithComments(&mut self) -> super::css_parser_token::CSSParserToken {
        self.emit()
    }
    fn Offset(&self) -> u32 {
        self.offset
    }
    fn PreviousOffset(&self) -> u32 {
        self.previous
    }
    fn StringRangeAt(&self, start: u32, length: u32) -> StringView {
        self.text.Substring(start, length)
    }
    fn StringRangeFrom(&self, start: u32) -> StringView {
        self.text.Substring(start, self.text.length() - start)
    }
    fn SkipToEndOfBlock(&mut self, offset: u32) {
        while self.offset < offset {
            self.emit();
        }
    }
    fn Restore(
        &mut self,
        _: &super::css_parser_token::CSSParserToken,
        offset: u32,
    ) -> super::css_parser_token::CSSParserToken {
        self.index = (0..=self.records.len())
            .find(|&i| {
                if i == 0 {
                    offset == 0
                } else {
                    self.records[i - 1].1 == offset
                }
            })
            .expect("token boundary restore");
        self.offset = offset;
        self.TokenizeSingle()
    }
    fn TokenCount(&self) -> u32 {
        self.count
    }
    fn UnicodeRangesAllowed(&self) -> bool {
        self.unicode
    }
    fn SetUnicodeRangesAllowed(&mut self, value: bool) {
        self.unicode = value;
    }
    fn PopBlockStack(&mut self) {} // Replayed tokens already retain their block classification.
}

fn make_property(id: CSSPropertyID, value: Rc<Value>, important: bool) -> PropertyValue {
    PropertyValue::new(&CSSPropertyName::new(id), value, important, false, 0, false)
}
fn make_expanded(
    id: CSSPropertyID,
    shorthand: CSSPropertyID,
    value: Rc<Value>,
    implicit: bool,
) -> PropertyValue {
    let index = MatchingShorthands(id)
        .iter()
        .position(|&candidate| candidate == shorthand)
        .expect("source shorthand metadata");
    PropertyValue::new(
        &CSSPropertyName::new(id),
        value,
        false,
        true,
        index as i32,
        implicit,
    )
}
fn ConsumeLonghand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    ConsumeLonghandWithAlias(id, stream, mode, false)
}
// css_parsing_utils.cc:5764-5782. Reused by physical/logical radius and Corner.
fn ConsumeBorderRadiusCorner<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let grammar = Grammar::Length {
        percent: true,
        nonnegative: true,
        quirks: false,
        keywords: &[],
    };
    let first = ConsumeLiteral(id, stream, mode, grammar)?;
    stream.EnsureLookAhead();
    let save = stream.Save();
    let second = if matches!(
        stream.Peek().GetType(),
        kNumberToken | kPercentageToken | kDimensionToken
    ) || IsMathFunction(stream)
    {
        match ConsumeLiteral(id, stream, mode, grammar) {
            Ok(v) => v,
            Err(e) if e.kind == PropertyParseErrorKind::Invalid => {
                stream.Restore(save);
                first.clone()
            }
            Err(e) => return Err(e),
        }
    } else {
        first.clone()
    };
    Ok(Pair(first, second, true))
}
fn ConsumeLonghandWithAlias<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    alias: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    if corner_parser::IsProperty(id) {
        return corner_parser::Consume(id, stream, mode);
    }
    if initial_scope_parser::IsProperty(id) {
        return initial_scope_parser::Consume(id, stream, mode);
    }
    if id == CSSPropertyID::kGridLanesDirection {
        return grid_lanes_parser::ConsumeDirection(id, stream);
    }
    if render_delay_parser::IsProperty(id) {
        return render_delay_parser::Consume(id, stream, mode);
    }
    if timeline_trigger_parser::IsProperty(id) {
        return timeline_trigger_parser::Consume(id, stream, mode);
    }
    if view_transition_parser::IsProperty(id) {
        return view_transition_parser::Consume(id, stream);
    }
    if motion_parser::IsProperty(id) {
        return motion_parser::Consume(id, stream, mode);
    }
    if border_image_parser::IsBorderImageProperty(id) {
        return border_image_parser::Consume(id, stream, mode);
    }
    if interaction_parser::IsInteractionProperty(id) {
        return interaction_parser::Consume(id, stream, mode);
    }
    if palette_internal_parser::IsProperty(id) {
        return palette_internal_parser::Consume(id, stream, mode);
    }
    if id == CSSPropertyID::kWebkitBoxReflect {
        return reflect_parser::Consume(stream, mode);
    }
    if stable_misc_parser::IsProperty(id) {
        return stable_misc_parser::Consume(id, stream, mode);
    }
    if column_rule_parser::IsProperty(id) {
        return column_rule_parser::Consume(id, stream, mode);
    }
    if rule_inset_parser::IsProperty(id) {
        return rule_inset_parser::Consume(id, stream, mode);
    }
    if text_box_parser::IsProperty(id) {
        return text_box_parser::Consume(id, stream, mode);
    }
    if typography_parser::IsTypographyProperty(id) {
        return typography_parser::Consume(id, stream, mode);
    }
    if line_parser::IsLineProperty(id) {
        return line_parser::Consume(id, stream, mode);
    }
    if anchor_parser::IsAnchorProperty(id) {
        return anchor_parser::Consume(id, stream, mode);
    }
    if scroll_parser::IsScrollProperty(id) {
        return scroll_parser::Consume(id, stream, mode);
    }
    if list_counter_parser::IsListCounterProperty(id) {
        return list_counter_parser::Consume(id, stream, mode);
    }
    if color_ui_parser::IsColorUIProperty(id) {
        return color_ui_parser::Consume(id, stream, mode);
    }
    if effects_parser::IsEffectsProperty(id) {
        return effects_parser::Consume(id, stream, mode);
    }
    if font_parser::IsFontProperty(id) {
        return font_parser::Consume(id, stream, mode);
    }
    if layout_misc_parser::IsLayoutProperty(id) {
        return layout_misc_parser::Consume(id, stream, mode);
    }
    if text_parser::IsTextProperty(id) {
        return text_parser::Consume(id, stream, mode);
    }
    if transform_parser::IsTransformProperty(id) {
        return transform_parser::Consume(id, stream, mode, alias);
    }
    if timeline_parser::IsProperty(id) {
        return timeline_parser::Consume(id, stream, mode);
    }
    if viewport_parser::IsProperty(id) {
        return viewport_parser::Consume(id, stream, mode);
    }
    if svg_presentation_parser::IsProperty(id) {
        return svg_presentation_parser::Consume(id, stream, mode);
    }
    if svg_parser::IsSVGProperty(id) {
        return svg_parser::Consume(id, stream, mode);
    }
    if matches!(
        id,
        CSSPropertyID::kBackgroundImage | CSSPropertyID::kMaskImage
    ) {
        return ConsumeBackgroundImage(id, stream, mode);
    }
    // cpp: longhands_custom.cc:2009-2016,10171-10178.
    if matches!(id, CSSPropertyID::kBoxShadow | CSSPropertyID::kTextShadow) {
        return ConsumeShadow(id, stream, mode);
    }
    if id == CSSPropertyID::kContainerName {
        return ConsumeContainerName(id, stream);
    }
    if id == CSSPropertyID::kContainerType {
        return ConsumeContainerType(id, stream);
    }
    // cpp: longhands_custom.cc:7464-7473. The focus-ring keyword is allowed
    // here even in standard mode; its platform color binds during conversion.
    if id == CSSPropertyID::kOutlineColor {
        if stream.Peek().Id() == CSSValueID::kWebkitFocusRingColor {
            return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
        }
        return ConsumeColor(id, stream);
    }
    // cpp: css_parser_fast_paths.cc:1288-1291. Outline excludes hidden,
    // includes auto, unlike the generic EBorderStyle field's keyword metadata.
    if id == CSSPropertyID::kOutlineStyle {
        let keyword = stream.Peek().Id();
        if matches!(keyword, CSSValueID::kAuto | CSSValueID::kNone)
            || (CSSValueID::kInset as i32..=CSSValueID::kDouble as i32).contains(&(keyword as i32))
        {
            stream.ConsumeIncludingWhitespace();
            return Ok(values::identifier(keyword));
        }
        return Err(invalid(id));
    }
    if matches!(
        id,
        CSSPropertyID::kAlignContent
            | CSSPropertyID::kJustifyContent
            | CSSPropertyID::kAlignItems
            | CSSPropertyID::kJustifyItems
            | CSSPropertyID::kAlignSelf
            | CSSPropertyID::kJustifySelf
    ) {
        return ConsumeAlignment(id, stream);
    }
    // cpp: longhands_custom.cc:4300-4352, FlexWrapBalance is stable.
    if id == CSSPropertyID::kFlexWrap {
        if stream.Peek().Id() == CSSValueID::kNowrap {
            stream.ConsumeIncludingWhitespace();
            return Ok(values::identifier(CSSValueID::kNowrap));
        }
        let mut wrap = None;
        let mut balance = false;
        loop {
            if wrap.is_none() && matches(stream.Peek().Id(), &["wrap", "wrap-reverse"]) {
                wrap = Some(stream.ConsumeIncludingWhitespace().Id());
                continue;
            }
            if !balance && stream.Peek().Id() == CSSValueID::kBalance {
                balance = true;
                stream.ConsumeIncludingWhitespace();
                continue;
            }
            break;
        }
        return Ok(if balance && wrap == Some(CSSValueID::kWrapReverse) {
            values::list(
                vec![
                    values::identifier(CSSValueID::kWrapReverse),
                    values::identifier(CSSValueID::kBalance),
                ],
                values::ListSeparator::Space,
            )
        } else if balance {
            values::identifier(CSSValueID::kBalance)
        } else {
            values::identifier(wrap.ok_or_else(|| invalid(id))?)
        });
    }
    // cpp: longhands_custom.cc:7252-7259,7306-7317,7385-7405,8286-8293.
    if matches!(
        id,
        CSSPropertyID::kObjectPosition
            | CSSPropertyID::kPerspectiveOrigin
            | CSSPropertyID::kOffsetAnchor
            | CSSPropertyID::kOffsetPosition
    ) {
        let keyword = stream.Peek().Id();
        if matches!(
            id,
            CSSPropertyID::kOffsetAnchor | CSSPropertyID::kOffsetPosition
        ) && (keyword == CSSValueID::kAuto
            || id == CSSPropertyID::kOffsetPosition && keyword == CSSValueID::kNormal)
        {
            stream.ConsumeIncludingWhitespace();
            return Ok(values::identifier(keyword));
        }
        let (x, y) = ConsumePosition(id, stream, mode, false, false)?;
        return Ok(Pair(x, y, false));
    }
    if id == CSSPropertyID::kTransformOrigin {
        return ConsumeTransformOrigin(id, stream, mode);
    }
    if matches!(
        id,
        CSSPropertyID::kBackgroundRepeat | CSSPropertyID::kMaskRepeat
    ) {
        let mut items = Vec::new();
        loop {
            items.push(ConsumeRepeatStyleValue(id, stream)?);
            if stream.Peek().GetType() != kCommaToken {
                break;
            }
            stream.ConsumeIncludingWhitespace();
        }
        return Ok(values::list(items, values::ListSeparator::Comma));
    }
    if IsAnimationListConsumer(id) {
        let mut items = Vec::new();
        loop {
            items.push(ConsumeAnimationItem(id, stream, mode)?.ok_or_else(|| invalid(id))?);
            if stream.Peek().GetType() != kCommaToken {
                break;
            }
            stream.ConsumeIncludingWhitespace();
        }
        if id == CSSPropertyID::kTransitionProperty {
            ValidateTransitionPropertyList(id, &items)?;
        }
        return Ok(values::list(items, values::ListSeparator::Comma));
    }
    if super::production_property_metadata::IsLayerConsumer(id) {
        // cpp: css_parsing_utils.h:963-976 ConsumeCommaSeparatedList.
        let mut layers = Vec::new();
        loop {
            layers.push(ConsumeLayer(id, stream, mode, alias)?);
            if stream.Peek().GetType() != kCommaToken {
                break;
            }
            stream.ConsumeIncludingWhitespace();
        }
        return Ok(values::list(layers, values::ListSeparator::Comma));
    }
    if matches!(
        id,
        CSSPropertyID::kWebkitPerspectiveOriginX
            | CSSPropertyID::kWebkitPerspectiveOriginY
            | CSSPropertyID::kWebkitTransformOriginX
            | CSSPropertyID::kWebkitTransformOriginY
    ) {
        return ConsumePositionAxis(id, stream, mode, false);
    }
    if matches!(
        id,
        CSSPropertyID::kGridColumnStart
            | CSSPropertyID::kGridColumnEnd
            | CSSPropertyID::kGridRowStart
            | CSSPropertyID::kGridRowEnd
    ) {
        return ConsumeGridLine(id, stream, mode);
    }
    // cpp: longhands_custom.cc:4954-4988 GridAutoFlow::ParseSingleValue.
    if id == CSSPropertyID::kGridAutoFlow {
        let mut axis = None;
        if matches(stream.Peek().Id(), &["row", "column"]) {
            axis = Some(stream.ConsumeIncludingWhitespace().Id());
        }
        let dense = stream.Peek().Id() == CSSValueID::kDense;
        if dense {
            stream.ConsumeIncludingWhitespace();
        }
        if axis.is_none() {
            if matches(stream.Peek().Id(), &["row", "column"]) {
                axis = Some(stream.ConsumeIncludingWhitespace().Id());
            } else if !dense {
                return Err(invalid(id));
            }
        }
        let mut items = Vec::new();
        if axis == Some(CSSValueID::kColumn) || (axis == Some(CSSValueID::kRow) && !dense) {
            items.push(values::identifier(axis.unwrap()));
        }
        if dense {
            items.push(values::identifier(CSSValueID::kDense));
        }
        return Ok(values::list(items, values::ListSeparator::Space));
    }
    if id == CSSPropertyID::kOverflowClipMargin {
        return ConsumeOverflowClipMargin(id, stream, mode);
    }
    if matches!(
        id,
        CSSPropertyID::kFontFeatureSettings | CSSPropertyID::kFontVariationSettings
    ) {
        return ConsumeFontSettings(id, stream);
    }
    // cpp: css_parsing_utils.cc:6556-6584 ConsumeFontStretch.
    if id == CSSPropertyID::kFontStretch {
        let keyword = stream.Peek().Id();
        if keyword == CSSValueID::kNormal
            || (CSSValueID::kUltraCondensed as i32..=CSSValueID::kUltraExpanded as i32)
                .contains(&(keyword as i32))
            || (keyword == CSSValueID::kAuto && mode == CSSParserMode::kCSSFontFaceRuleMode)
        {
            stream.ConsumeIncludingWhitespace();
            return Ok(values::identifier(keyword));
        }
        let mut percents = Vec::new();
        for _ in 0..if mode == CSSParserMode::kCSSFontFaceRuleMode {
            2
        } else {
            1
        } {
            if is_function(stream) {
                return Err(unsupported(id, "ConsumePercent CSSMathFunctionValue"));
            }
            if stream.Peek().GetType() != kPercentageToken || stream.Peek().NumericValue() < 0.0 {
                return Err(invalid(id));
            }
            percents.push(values::numeric(
                stream.ConsumeIncludingWhitespace().NumericValue(),
                UnitType::kPercentage,
            ));
            if at_value_end(stream) {
                break;
            }
        }
        return Ok(if percents.len() == 1 {
            percents.remove(0)
        } else {
            values::list(percents, values::ListSeparator::Space)
        });
    }
    // cpp: longhands_custom.cc Display::ParseSingleValue:3788-3811.
    if id == CSSPropertyID::kDisplay {
        let keyword = stream.Peek().Id();
        let valid = (CSSValueID::kInline as i32..=CSSValueID::kBlock as i32)
            .contains(&(keyword as i32))
            || (CSSValueID::kFlowRoot as i32..CSSValueID::kGridLanes as i32)
                .contains(&(keyword as i32))
            || (CSSValueID::kTableRowGroup as i32..=CSSValueID::kRubyText as i32)
                .contains(&(keyword as i32))
            || (CSSValueID::kInlineBlock as i32..=CSSValueID::kWebkitInlineFlex as i32)
                .contains(&(keyword as i32))
            || matches!(
                keyword,
                CSSValueID::kFlow
                    | CSSValueID::kNone
                    | CSSValueID::kContents
                    | CSSValueID::kListItem
                    | CSSValueID::kMath
                    | CSSValueID::kRuby
            );
        if !valid {
            return Err(if is_function(stream) {
                unsupported(id, "Display::ParseSingleValue CSSLayoutFunctionValue")
            } else {
                invalid(id)
            });
        }
        stream.ConsumeIncludingWhitespace();
        if stream.Peek().GetType() == kIdentToken {
            return ConsumeDisplayMultiple(id, keyword, stream);
        }
        return Ok(values::identifier(if keyword == CSSValueID::kFlow {
            CSSValueID::kBlock
        } else {
            keyword
        }));
    }
    // cpp: generated longhands.cc:3950,4061,4718,4829: logical style
    // SurrogateFor uses the physical border-style generated keyword grammar.
    if matches!(
        id,
        CSSPropertyID::kBorderBlockStartStyle
            | CSSPropertyID::kBorderBlockEndStyle
            | CSSPropertyID::kBorderInlineStartStyle
            | CSSPropertyID::kBorderInlineEndStyle
    ) {
        return ConsumeLiteral(id, stream, mode, GrammarFor(CSSPropertyID::kBorderTopStyle));
    }
    // cpp: css_parsing_utils.cc:9653-9662 ConsumeBorderColorSide. The quad
    // border-color shorthand uses this consumer too; side shorthands call
    // ConsumeColor directly and therefore do not enable quirky hex colors.
    if matches!(
        id,
        CSSPropertyID::kBorderTopColor
            | CSSPropertyID::kBorderRightColor
            | CSSPropertyID::kBorderBottomColor
            | CSSPropertyID::kBorderLeftColor
    ) {
        return ConsumeBorderColorSide(id, stream, mode);
    }
    // cpp: css_parsing_utils.cc ParseBorderRadiusCorner:5765-5784.
    if matches!(
        id,
        CSSPropertyID::kBorderTopLeftRadius
            | CSSPropertyID::kBorderTopRightRadius
            | CSSPropertyID::kBorderBottomRightRadius
            | CSSPropertyID::kBorderBottomLeftRadius
            | CSSPropertyID::kBorderStartStartRadius
            | CSSPropertyID::kBorderStartEndRadius
            | CSSPropertyID::kBorderEndStartRadius
            | CSSPropertyID::kBorderEndEndRadius
    ) {
        return ConsumeBorderRadiusCorner(id, stream, mode);
    }
    // cpp: css_parsing_utils.cc ConsumeFontSize:6225-6240.
    if id == CSSPropertyID::kFontSize {
        return ConsumeLiteral(
            id,
            stream,
            mode,
            Grammar::Length {
                percent: true,
                nonnegative: true,
                quirks: true,
                keywords: &[
                    "xx-small",
                    "x-small",
                    "small",
                    "medium",
                    "large",
                    "x-large",
                    "xx-large",
                    "xxx-large",
                    "-webkit-xxx-large",
                    "larger",
                    "smaller",
                    "math",
                ],
            },
        );
    }
    // cpp: css_parsing_utils.cc ConsumeLineHeight:6242-6260.
    if id == CSSPropertyID::kLineHeight {
        if IsMathFunction(stream) {
            use crate::css_math_expression_node::CalculationResultCategory as C;
            return ConsumeMath(
                id,
                stream,
                &[C::Number, C::Length, C::Percent, C::LengthFunction],
                crate::css_math_function_value::ValueRange::NonNegative,
            );
        }
        if stream.Peek().GetType() == kNumberToken {
            return ConsumeLiteral(id, stream, mode, Grammar::Number { nonnegative: true });
        }
        return ConsumeLiteral(
            id,
            stream,
            mode,
            Grammar::Length {
                percent: true,
                nonnegative: true,
                quirks: false,
                keywords: &["normal"],
            },
        );
    }
    // cpp: css_parsing_utils.cc ConsumeFontWeight:6587-6634.
    if id == CSSPropertyID::kFontWeight {
        if matches(stream.Peek().Id(), &["normal", "bold", "bolder", "lighter"]) {
            let id = stream.ConsumeIncludingWhitespace().Id();
            return Ok(values::identifier(id));
        }
        if stream.Peek().GetType() == kNumberToken
            && !(1.0..=1000.0).contains(&stream.Peek().NumericValue())
        {
            return Err(invalid(id));
        }
        return ConsumeLiteral(id, stream, mode, Grammar::Number { nonnegative: true });
    }
    if id == CSSPropertyID::kFontFamily {
        let mut families = Vec::new();
        loop {
            families.push(ConsumeFamily(id, stream)?);
            if stream.Peek().GetType() != kCommaToken {
                break;
            }
            stream.ConsumeIncludingWhitespace();
        }
        return Ok(values::list(families, values::ListSeparator::Comma));
    }
    // cpp: css_parsing_utils.cc:6481-6523, element font-style permits one angle.
    if id == CSSPropertyID::kFontStyle
        && matches(stream.Peek().Id(), &["normal", "italic", "oblique"])
    {
        let keyword = stream.ConsumeIncludingWhitespace().Id();
        if keyword == CSSValueID::kOblique && !stream.AtEnd() {
            let angle = if IsMathFunction(stream) {
                ConsumeMath(
                    id,
                    stream,
                    &[crate::css_math_expression_node::CalculationResultCategory::Angle],
                    crate::css_math_function_value::ValueRange::All,
                )?
            } else {
                if stream.Peek().GetType() != kDimensionToken {
                    return Err(invalid(id));
                }
                let token = stream.ConsumeIncludingWhitespace();
                let unit = token.GetUnitType();
                let degrees = token.NumericValue()
                    * match unit {
                        UnitType::kDegrees => 1.0,
                        UnitType::kRadians => 180.0 / std::f64::consts::PI,
                        UnitType::kGradians => 0.9,
                        UnitType::kTurns => 360.0,
                        _ => return Err(invalid(id)),
                    };
                if !(-90.0..=90.0).contains(&degrees) {
                    return Err(invalid(id));
                }
                if degrees == 0.0 {
                    return Ok(values::identifier(CSSValueID::kNormal));
                }
                values::numeric(token.NumericValue(), unit)
            };
            return Ok(Rc::new(Value::new(CSSValuePayload::kFontStyleRangeClass(
                values::CSSFontStyleRangeValue { angle: Some(angle) },
            ))));
        }
        return Ok(values::identifier(keyword));
    }
    if id == CSSPropertyID::kGridTemplateAreas {
        return grid_parser::ConsumeAreas(id, stream);
    }
    if matches!(
        id,
        CSSPropertyID::kGridTemplateColumns
            | CSSPropertyID::kGridTemplateRows
            | CSSPropertyID::kGridAutoColumns
            | CSSPropertyID::kGridAutoRows
    ) {
        return grid_parser::ConsumeTracks(
            id,
            stream,
            mode,
            matches!(
                id,
                CSSPropertyID::kGridTemplateColumns | CSSPropertyID::kGridTemplateRows
            ),
            false,
        );
    }
    // cpp: css_parsing_utils.cc ConsumeAlphaValue/ConsumeNumberOrPercent:1420-1451.
    if matches!(
        id,
        CSSPropertyID::kOpacity
            | CSSPropertyID::kFillOpacity
            | CSSPropertyID::kFloodOpacity
            | CSSPropertyID::kStopOpacity
            | CSSPropertyID::kStrokeOpacity
    ) {
        if IsMathFunction(stream) {
            return ConsumeMath(
                id,
                stream,
                &[
                    crate::css_math_expression_node::CalculationResultCategory::Number,
                    crate::css_math_expression_node::CalculationResultCategory::Percent,
                ],
                crate::css_math_function_value::ValueRange::All,
            );
        }
        if stream.Peek().GetType() == kPercentageToken {
            let number = stream.ConsumeIncludingWhitespace().NumericValue() / 100.0;
            return Ok(values::numeric(number, UnitType::kNumber));
        }
        return ConsumeLiteral(id, stream, mode, Grammar::Number { nonnegative: false });
    }
    // cpp: longhands_custom.cc ZIndex::ParseSingleValue:12697-12707.
    if id == CSSPropertyID::kZIndex && stream.Peek().Id() == CSSValueID::kAuto {
        stream.ConsumeIncludingWhitespace();
        return Ok(values::identifier(CSSValueID::kAuto));
    }
    if id == CSSPropertyID::kZIndex {
        // ConsumeInteger admits the full finite double range. ComputeInteger
        // clamps to native int later; parser range is not the storage range.
        if is_function(stream) {
            return ConsumeMath(
                id,
                stream,
                &[crate::css_math_expression_node::CalculationResultCategory::Number],
                crate::css_math_function_value::ValueRange::Integer,
            );
        }
        let token = stream.Peek();
        if token.GetType() != kNumberToken
            || token.GetNumericValueType() != NumericValueType::kIntegerValueType
        {
            return Err(invalid(id));
        }
        let number = token.NumericValue();
        stream.ConsumeIncludingWhitespace();
        return Ok(values::numeric(number, UnitType::kInteger));
    }
    // cpp: css_parsing_utils.cc ConsumeGapLength:6110-6121.
    if matches!(id, CSSPropertyID::kRowGap | CSSPropertyID::kColumnGap) {
        return ConsumeLiteral(
            id,
            stream,
            mode,
            Grammar::Length {
                percent: true,
                nonnegative: true,
                quirks: false,
                keywords: &["normal"],
            },
        );
    }
    // cpp: longhands_custom.cc FlexBasis::ParseSingleValue, source width grammar + content.
    if id == CSSPropertyID::kFlexBasis {
        return ConsumeLiteral(
            id,
            stream,
            mode,
            Grammar::Length {
                percent: true,
                nonnegative: true,
                quirks: false,
                keywords: &[
                    "auto",
                    "content",
                    "min-content",
                    "max-content",
                    "fit-content",
                    "stretch",
                ],
            },
        );
    }
    ConsumeLiteral(id, stream, mode, GrammarFor(id))
}
// cpp: css_parsing_utils.cc:149-170,187-208,4358-4391,4499-4563;
// longhands_custom.cc:157-204,6164-6242. Stable default disables anchor-center
// for align-items/justify-items, while the self longhands still accept it.
fn ContentDistribution(
    distribution: CSSValueID,
    position: CSSValueID,
    overflow: CSSValueID,
) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kCSSContentDistributionClass(
        values::CSSContentDistributionValue {
            distribution,
            position,
            overflow,
        },
    )))
}
fn ConsumeAlignment<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    let content = matches!(
        id,
        CSSPropertyID::kAlignContent | CSSPropertyID::kJustifyContent
    );
    let justify = matches!(
        id,
        CSSPropertyID::kJustifyContent | CSSPropertyID::kJustifyItems | CSSPropertyID::kJustifySelf
    );
    let items = matches!(
        id,
        CSSPropertyID::kAlignItems | CSSPropertyID::kJustifyItems
    );
    let initial = stream.Peek().Id();
    if items && initial == CSSValueID::kAuto
        || id == CSSPropertyID::kJustifyContent && matches(initial, &["first", "last", "baseline"])
    {
        return Err(invalid(id));
    }
    if id == CSSPropertyID::kJustifyItems {
        let savepoint = stream.Save();
        let mut legacy = stream.Peek().Id() == CSSValueID::kLegacy;
        if legacy {
            stream.ConsumeIncludingWhitespace();
        }
        let position = if matches(stream.Peek().Id(), &["center", "left", "right"]) {
            Some(stream.ConsumeIncludingWhitespace().Id())
        } else {
            None
        };
        if !legacy && stream.Peek().Id() == CSSValueID::kLegacy {
            legacy = true;
            stream.ConsumeIncludingWhitespace();
        }
        if legacy {
            return Ok(if let Some(position) = position {
                Pair(
                    values::identifier(CSSValueID::kLegacy),
                    values::identifier(position),
                    true,
                )
            } else {
                values::identifier(CSSValueID::kLegacy)
            });
        }
        stream.EnsureLookAhead();
        stream.Restore(savepoint);
    }
    if content && initial == CSSValueID::kNormal {
        stream.ConsumeIncludingWhitespace();
        return Ok(ContentDistribution(
            CSSValueID::kInvalid,
            initial,
            CSSValueID::kInvalid,
        ));
    }
    if !content && matches(initial, &["auto", "normal", "stretch"]) {
        stream.ConsumeIncludingWhitespace();
        return Ok(values::identifier(initial));
    }
    // ConsumeFirstBaseline (content) / ConsumeBaseline (self). A failed optional
    // preference consumes its token, following the source consumer's contract.
    let preference = stream.Peek().Id();
    if preference == CSSValueID::kFirst || !content && preference == CSSValueID::kLast {
        stream.ConsumeIncludingWhitespace();
    }
    if stream.Peek().Id() == CSSValueID::kBaseline {
        stream.ConsumeIncludingWhitespace();
        return Ok(if content {
            ContentDistribution(
                CSSValueID::kInvalid,
                CSSValueID::kBaseline,
                CSSValueID::kInvalid,
            )
        } else if preference == CSSValueID::kLast {
            Pair(
                values::identifier(preference),
                values::identifier(CSSValueID::kBaseline),
                true,
            )
        } else {
            values::identifier(CSSValueID::kBaseline)
        });
    }
    if content
        && matches(
            initial,
            &["space-between", "space-around", "space-evenly", "stretch"],
        )
    {
        stream.ConsumeIncludingWhitespace();
        return Ok(ContentDistribution(
            initial,
            CSSValueID::kInvalid,
            CSSValueID::kInvalid,
        ));
    }
    stream.EnsureLookAhead();
    let savepoint = stream.Save();
    // Content's source tests the original token before consuming overflow.
    let overflow_keyword = if content { initial } else { stream.Peek().Id() };
    let overflow = if matches(overflow_keyword, &["safe", "unsafe"]) {
        stream.ConsumeIncludingWhitespace().Id()
    } else {
        CSSValueID::kInvalid
    };
    let position = stream.Peek().Id();
    let valid = matches(
        position,
        &["start", "end", "center", "flex-start", "flex-end"],
    ) || justify && matches(position, &["left", "right"])
        || !content && matches(position, &["self-start", "self-end"])
        || !content && !items && position == CSSValueID::kAnchorCenter;
    if !valid {
        if content {
            stream.Restore(savepoint);
        }
        return Err(invalid(id));
    }
    stream.ConsumeIncludingWhitespace();
    Ok(if content {
        ContentDistribution(CSSValueID::kInvalid, position, overflow)
    } else if overflow != CSSValueID::kInvalid {
        Pair(
            values::identifier(overflow),
            values::identifier(position),
            true,
        )
    } else {
        values::identifier(position)
    })
}
// cpp: longhands_custom.cc:361-651,10555-10665; css_parsing_utils.h:963-976.
fn IsAnimationListConsumer(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kAnimationComposition
            | CSSPropertyID::kAnimationDelay
            | CSSPropertyID::kAnimationDirection
            | CSSPropertyID::kAnimationDuration
            | CSSPropertyID::kAnimationFillMode
            | CSSPropertyID::kAnimationIterationCount
            | CSSPropertyID::kAnimationName
            | CSSPropertyID::kAnimationPlayState
            | CSSPropertyID::kAnimationTimingFunction
            | CSSPropertyID::kAnimationTimeline
            | CSSPropertyID::kAnimationTrigger
            | CSSPropertyID::kAnimationRangeStart
            | CSSPropertyID::kAnimationRangeEnd
            | CSSPropertyID::kTransitionDelay
            | CSSPropertyID::kTransitionDuration
            | CSSPropertyID::kTransitionProperty
            | CSSPropertyID::kTransitionBehavior
            | CSSPropertyID::kTransitionTimingFunction
    )
}
// cpp: css_math_expression_node.cc:4402-4443. Runtime-gated math names are
// explicit unsupported collaborators along with the unported math parser.
fn IsMathFunction<T: TokenStreamTokenizer>(stream: &mut Stream<T>) -> bool {
    matches!(
        stream.Peek().FunctionId(),
        Some(
            CSSValueID::kMin
                | CSSValueID::kMax
                | CSSValueID::kClamp
                | CSSValueID::kCalc
                | CSSValueID::kWebkitCalc
                | CSSValueID::kSin
                | CSSValueID::kCos
                | CSSValueID::kTan
                | CSSValueID::kAsin
                | CSSValueID::kAcos
                | CSSValueID::kAtan
                | CSSValueID::kAtan2
                | CSSValueID::kAnchor
                | CSSValueID::kAnchorSize
                | CSSValueID::kCalcSize
                | CSSValueID::kRound
                | CSSValueID::kMod
                | CSSValueID::kRem
                | CSSValueID::kPow
                | CSSValueID::kSqrt
                | CSSValueID::kHypot
                | CSSValueID::kLog
                | CSSValueID::kExp
                | CSSValueID::kSiblingCount
                | CSSValueID::kSiblingIndex
                | CSSValueID::kAbs
                | CSSValueID::kSign
                | CSSValueID::kProgress
                | CSSValueID::kMediaProgress
                | CSSValueID::kContainerProgress
                | CSSValueID::kRandom
        )
    )
}
// cpp: css_parsing_utils.cc:CalcParser and numeric consumers. The permitted
// range clamps at computation, so a negative calc is accepted then clamped.
fn ConsumeMath<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    categories: &[crate::css_math_expression_node::CalculationResultCategory],
    range: crate::css_math_function_value::ValueRange,
) -> Result<Rc<Value>, PropertyParseError> {
    let expression = crate::css_math_expression_node::ConsumeMathFunction(stream).map_err(|e| {
        if e == crate::css_math_expression_node::MathError::UnsupportedFunction {
            unsupported(id, "CSSMathExpressionNode extended math functions")
        } else if e == crate::css_math_expression_node::MathError::UnsupportedTypedArithmetic {
            unsupported(id, "CSSMathType typed exponent arithmetic")
        } else {
            invalid(id)
        }
    })?;
    if !categories.contains(&expression.Category()) {
        return Err(invalid(id));
    }
    Ok(values::math(expression, range))
}
// cpp: numeric CalcParser consumers restore a category mismatch. Reuse the
// shared typed math parser so a shorthand number can reach iteration-count
// after the time consumer, and linear() percentages can follow its number probe.
fn ConsumeAnimationNumericMath<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    category: crate::css_math_expression_node::CalculationResultCategory,
    range: crate::css_math_function_value::ValueRange,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    use crate::css_math_expression_node::CalculationResultCategory as C;
    if !IsMathFunction(stream) {
        return Ok(None);
    }
    let saved = stream.Save();
    let value = ConsumeMath(
        id,
        stream,
        &[
            C::Number,
            C::Length,
            C::Percent,
            C::LengthFunction,
            C::Angle,
            C::Time,
            C::Frequency,
            C::Resolution,
        ],
        range,
    )?;
    let CSSValuePayload::kMathFunctionClass(math) = value.Payload() else {
        unreachable!()
    };
    if math.Category() == category {
        return Ok(Some(value));
    }
    stream.Peek();
    stream.Restore(saved);
    Ok(None)
}
// cpp: CSSPrimitiveValue::GetValueIfKnown. Easing stop sorting and cubic
// bounds are parse-time operations, requiring a context-independent value.
fn AnimationKnownNumber(id: CSSPropertyID, value: &Value) -> Result<f64, PropertyParseError> {
    match value.Payload() {
        CSSValuePayload::kNumericLiteralClass(number) => Ok(number.DoubleValue()),
        CSSValuePayload::kMathFunctionClass(math) => math
            .ComputeValue(
                &mut |_, _| Err(crate::css_math_expression_node::MathError::MissingLengthContext),
                None,
            )
            .map(crate::css_value_clamping_utils::CSSValueClampingUtils::ClampDouble)
            .map_err(|_| invalid(id)),
        _ => Err(invalid(id)),
    }
}
fn ConsumeAnimationKnownNumeric<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    percent: bool,
) -> Result<Option<f64>, PropertyParseError> {
    use crate::css_math_expression_node::CalculationResultCategory as C;
    if let Some(value) = ConsumeAnimationNumericMath(
        id,
        stream,
        if percent { C::Percent } else { C::Number },
        crate::css_math_function_value::ValueRange::All,
    )? {
        return AnimationKnownNumber(id, &value).map(Some);
    }
    if stream.Peek().GetType()
        == if percent {
            kPercentageToken
        } else {
            kNumberToken
        }
    {
        return Ok(Some(stream.ConsumeIncludingWhitespace().NumericValue()));
    }
    Ok(None)
}
// cpp: css_parsing_utils.cc:1601-1630. Unmatched consumers never consume tokens.
fn ConsumeTime<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    nonnegative: bool,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if IsMathFunction(stream) {
        return ConsumeAnimationNumericMath(
            id,
            stream,
            crate::css_math_expression_node::CalculationResultCategory::Time,
            if nonnegative {
                crate::css_math_function_value::ValueRange::NonNegative
            } else {
                crate::css_math_function_value::ValueRange::All
            },
        );
    }
    let token = stream.Peek();
    if token.GetType() == kDimensionToken
        && matches!(
            token.GetUnitType(),
            UnitType::kSeconds | UnitType::kMilliseconds
        )
        && (!nonnegative || token.NumericValue() >= 0.0)
    {
        let unit = token.GetUnitType();
        return Ok(Some(values::numeric(
            stream.ConsumeIncludingWhitespace().NumericValue(),
            unit,
        )));
    }
    Ok(None)
}
// cpp: css_parsing_utils.cc:1757-1784,4571-4602,9613-9640.
fn ConsumeAnimationName<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    transition: bool,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    let token = stream.Peek().clone();
    if token.Id() == CSSValueID::kNone {
        stream.ConsumeIncludingWhitespace();
        return Ok(Some(values::identifier(CSSValueID::kNone)));
    }
    if !transition && token.GetType() == kStringToken {
        let name = stream.ConsumeIncludingWhitespace().Value().ToString();
        if name.length() == 0 {
            return Ok(None);
        }
        let word = name.Utf8().to_ascii_lowercase();
        let reserved = [
            "none",
            "default",
            "initial",
            "inherit",
            "unset",
            "revert",
            "revert-layer",
        ];
        return Ok(Some(if reserved.contains(&word.as_str()) {
            Rc::new(Value::new(CSSValuePayload::kStringClass(
                values::CSSStringValue(name),
            )))
        } else {
            values::custom_ident(&name, CSSPropertyID::kInvalid)
        }));
    }
    if token.FunctionId() == Some(CSSValueID::kIdent) {
        return Err(unsupported(
            id,
            "ConsumeCustomIdent CSSFunctionValue ident()",
        ));
    }
    if token.GetType() != kIdentToken
        || values::wide(token.Id()).is_some()
        || token.Id() == CSSValueID::kDefault
    {
        return Ok(None);
    }
    let name = token.Value().ToString();
    if transition {
        if let Some(property) = FindProperty(name.Utf8().to_ascii_lowercase().as_bytes()) {
            if property.id_and_exposed_bit & crate::css_property_names::kNotKnownExposedPropertyBit
                != 0
            {
                return Err(unsupported(
                    id,
                    "ConsumeTransitionProperty ExecutionContext::IsWebExposed",
                ));
            }
            let unresolved = super::production_property_metadata::UnresolvedPropertyIDFromInteger(
                property.id_and_exposed_bit,
            );
            if unresolved != CSSPropertyID::kInvalid
                && unresolved != CSSPropertyID::kVariable
                && crate::properties::css_property::CSSProperty::Get(ResolveCSSPropertyID(
                    unresolved,
                ))
                .IsProperty()
            {
                stream.ConsumeIncludingWhitespace();
                return Ok(Some(values::custom_ident(&name, unresolved)));
            }
        }
    }
    stream.ConsumeIncludingWhitespace();
    Ok(Some(values::custom_ident(&name, CSSPropertyID::kInvalid)))
}
// cpp: css_parsing_utils.cc:206-464,4725-4747. Function guards restore failures.
fn ConsumeAnimationTimingFunction<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if matches(
        stream.Peek().Id(),
        &[
            "ease",
            "linear",
            "ease-in",
            "ease-out",
            "ease-in-out",
            "step-start",
            "step-end",
        ],
    ) {
        return Ok(Some(values::identifier(
            stream.ConsumeIncludingWhitespace().Id(),
        )));
    }
    let function = stream.Peek().FunctionId();
    if !matches!(
        function,
        Some(CSSValueID::kSteps | CSSValueID::kCubicBezier | CSSValueID::kLinear)
    ) {
        return Ok(None);
    }
    let value;
    {
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        if function == Some(CSSValueID::kCubicBezier) {
            let mut args = [0.0; 4];
            for (index, arg) in args.iter_mut().enumerate() {
                let Some(number) = ConsumeAnimationKnownNumeric(id, &mut guard, false)? else {
                    return Ok(None);
                };
                *arg = number;
                if index < 3 {
                    if guard.Peek().GetType() != kCommaToken {
                        return Ok(None);
                    }
                    guard.ConsumeIncludingWhitespace();
                } else if !guard.AtEnd() {
                    return Ok(None);
                }
            }
            if !(0.0..=1.0).contains(&args[0]) || !(0.0..=1.0).contains(&args[2]) {
                return Ok(None);
            }
            value = Rc::new(Value::new(
                CSSValuePayload::kCubicBezierTimingFunctionClass(
                    values::CSSCubicBezierTimingFunctionValue(args),
                ),
            ));
        } else if function == Some(CSSValueID::kSteps) {
            let steps = if IsMathFunction(&mut guard) {
                let Some(value) = ConsumeAnimationNumericMath(
                    id,
                    &mut guard,
                    crate::css_math_expression_node::CalculationResultCategory::Number,
                    crate::css_math_function_value::ValueRange::PositiveInteger,
                )?
                else {
                    return Ok(None);
                };
                value
            } else {
                let token = guard.Peek();
                if token.GetType() != kNumberToken
                    || token.GetNumericValueType() != NumericValueType::kIntegerValueType
                    || token.NumericValue() < 1.0
                {
                    return Ok(None);
                }
                values::numeric(
                    guard.ConsumeIncludingWhitespace().NumericValue(),
                    UnitType::kInteger,
                )
            };
            let count = AnimationKnownNumber(id, &steps)?;
            let mut position = CSSValueID::kEnd;
            if guard.Peek().GetType() == kCommaToken {
                guard.ConsumeIncludingWhitespace();
                position = guard.Peek().Id();
                if !matches(
                    position,
                    &[
                        "start",
                        "end",
                        "jump-both",
                        "jump-end",
                        "jump-none",
                        "jump-start",
                    ],
                ) {
                    return Ok(None);
                }
                guard.ConsumeIncludingWhitespace();
            }
            if !guard.AtEnd() || position == CSSValueID::kJumpNone && count < 2.0 {
                return Ok(None);
            }
            value = Rc::new(Value::new(CSSValuePayload::kStepsTimingFunctionClass(
                values::CSSStepsTimingFunctionValue { steps, position },
            )));
        } else {
            // ConsumeLinearStop permits the numeric output before or after its
            // one/two percentages, but the two percentages remain contiguous.
            let mut stops = Vec::new();
            loop {
                let mut number = None;
                let mut a = None;
                let mut b = None;
                while !guard.AtEnd() && guard.Peek().GetType() != kCommaToken {
                    if number.is_none() {
                        if let Some(output) = ConsumeAnimationKnownNumeric(id, &mut guard, false)? {
                            number = Some(output);
                            continue;
                        }
                    }
                    if a.is_none() {
                        if let Some(input) = ConsumeAnimationKnownNumeric(id, &mut guard, true)? {
                            a = Some(input);
                            b = ConsumeAnimationKnownNumeric(id, &mut guard, true)?;
                            continue;
                        }
                    }
                    return Ok(None);
                }
                let Some(number) = number else {
                    return Ok(None);
                };
                stops.push((number, a, b));
                if guard.Peek().GetType() != kCommaToken {
                    break;
                }
                guard.ConsumeIncludingWhitespace();
            }
            if !guard.AtEnd() || stops.len() < 2 {
                return Ok(None);
            }
            let mut largest = f64::MIN;
            let mut points = Vec::new();
            for (index, &(output, a, b)) in stops.iter().enumerate() {
                let input = if let Some(a) = a {
                    largest = largest.max(a);
                    largest
                } else if index == 0 {
                    largest = 0.0;
                    0.0
                } else if index == stops.len() - 1 {
                    largest.max(100.0)
                } else {
                    f64::NAN
                };
                points.push(values::LinearEasingPoint { input, output });
                if let Some(b) = b {
                    largest = largest.max(b);
                    points.push(values::LinearEasingPoint {
                        input: largest,
                        output,
                    });
                }
            }
            let mut upper = 0;
            for index in 1..points.len() {
                if points[index].input.is_nan() {
                    if index > upper {
                        upper = (index + 1..points.len())
                            .find(|&i| !points[i].input.is_nan())
                            .unwrap();
                    }
                    points[index].input = points[index - 1].input
                        + (points[upper].input - points[index - 1].input)
                            / (upper - (index - 1)) as f64;
                }
            }
            value = Rc::new(Value::new(CSSValuePayload::kLinearTimingFunctionClass(
                values::CSSLinearTimingFunctionValue(points),
            )));
        }
        guard.Release();
    }
    stream.ConsumeWhitespace();
    Ok(Some(value))
}
// cpp: css_parsing_utils.cc:4814-4862, stable ScrollTimelineNamedRangeScroll.
fn ConsumeAnimationRange<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    default: f64,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    ConsumeAnimationRangeWithAuto(id, stream, mode, default, false)
}
fn ConsumeAnimationRangeWithAuto<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    default: f64,
    allow_auto: bool,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kNormal
        || allow_auto && stream.Peek().Id() == CSSValueID::kAuto
    {
        return Ok(Some(values::identifier(
            stream.ConsumeIncludingWhitespace().Id(),
        )));
    }
    let mut items = Vec::new();
    if matches(
        stream.Peek().Id(),
        &[
            "contain",
            "cover",
            "entry",
            "entry-crossing",
            "exit",
            "exit-crossing",
            "scroll",
        ],
    ) {
        items.push(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    let length = if IsMathFunction(stream) {
        use crate::css_math_expression_node::CalculationResultCategory as C;
        let saved = stream.Save();
        let value = ConsumeMath(
            id,
            stream,
            &[
                C::Number,
                C::Length,
                C::Percent,
                C::LengthFunction,
                C::Angle,
                C::Time,
                C::Frequency,
                C::Resolution,
            ],
            crate::css_math_function_value::ValueRange::All,
        )?;
        let CSSValuePayload::kMathFunctionClass(math) = value.Payload() else {
            unreachable!()
        };
        if matches!(math.Category(), C::Length | C::Percent | C::LengthFunction) {
            Some(value)
        } else {
            stream.Peek();
            stream.Restore(saved);
            None
        }
    } else {
        ConsumeLiteral(
            id,
            stream,
            mode,
            Grammar::Length {
                percent: true,
                nonnegative: false,
                quirks: false,
                keywords: &[],
            },
        )
        .ok()
    };
    if let Some(value) = length {
        let is_default = matches!(value.Payload(), CSSValuePayload::kNumericLiteralClass(n) if n.GetType() == UnitType::kPercentage && n.DoubleValue() == default);
        if items.is_empty() || !is_default {
            items.push(value);
        }
    }
    Ok(if items.is_empty() {
        None
    } else {
        Some(values::list(items, values::ListSeparator::Space))
    })
}
fn ConsumeAnimationTimeline<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if matches(stream.Peek().Id(), &["none", "auto"]) {
        return Ok(Some(values::identifier(
            stream.ConsumeIncludingWhitespace().Id(),
        )));
    }
    if matches!(
        stream.Peek().FunctionId(),
        Some(CSSValueID::kScroll | CSSValueID::kView)
    ) {
        return Err(unsupported(
            id,
            "ConsumeAnimationTimeline CSSScrollValue / CSSViewValue",
        ));
    }
    if stream.Peek().FunctionId() == Some(CSSValueID::kIdent) {
        return Err(unsupported(
            id,
            "ConsumeDashedIdent CSSFunctionValue ident()",
        ));
    }
    if stream.Peek().GetType() == kIdentToken
        && stream.Peek().Value().ToString().Utf8().starts_with("--")
    {
        return ConsumeAnimationName(id, stream, false);
    }
    Ok(None)
}
fn ConsumeAnimationItem<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    let words: &[&str] = match id {
        CSSPropertyID::kAnimationComposition => &["replace", "add", "accumulate"],
        CSSPropertyID::kAnimationDirection => {
            &["normal", "alternate", "reverse", "alternate-reverse"]
        }
        CSSPropertyID::kAnimationFillMode => &["none", "forwards", "backwards", "both"],
        CSSPropertyID::kAnimationPlayState => &["running", "paused"],
        CSSPropertyID::kTransitionBehavior => &["normal", "allow-discrete"],
        _ => &[],
    };
    if !words.is_empty() {
        return Ok(if matches(stream.Peek().Id(), words) {
            Some(values::identifier(stream.ConsumeIncludingWhitespace().Id()))
        } else {
            None
        });
    }
    match id {
        CSSPropertyID::kAnimationDelay | CSSPropertyID::kTransitionDelay => {
            ConsumeTime(id, stream, false)
        }
        CSSPropertyID::kAnimationDuration if stream.Peek().Id() == CSSValueID::kAuto => Ok(Some(
            values::identifier(stream.ConsumeIncludingWhitespace().Id()),
        )),
        CSSPropertyID::kAnimationDuration | CSSPropertyID::kTransitionDuration => {
            ConsumeTime(id, stream, true)
        }
        CSSPropertyID::kAnimationName | CSSPropertyID::kTransitionProperty => {
            ConsumeAnimationName(id, stream, id == CSSPropertyID::kTransitionProperty)
        }
        CSSPropertyID::kAnimationTimingFunction | CSSPropertyID::kTransitionTimingFunction => {
            ConsumeAnimationTimingFunction(id, stream)
        }
        CSSPropertyID::kAnimationIterationCount => {
            if stream.Peek().Id() == CSSValueID::kInfinite {
                return Ok(Some(values::identifier(
                    stream.ConsumeIncludingWhitespace().Id(),
                )));
            }
            if IsMathFunction(stream) {
                return ConsumeMath(
                    id,
                    stream,
                    &[crate::css_math_expression_node::CalculationResultCategory::Number],
                    crate::css_math_function_value::ValueRange::NonNegative,
                )
                .map(Some);
            }
            Ok(
                if stream.Peek().GetType() == kNumberToken && stream.Peek().NumericValue() >= 0.0 {
                    Some(values::numeric(
                        stream.ConsumeIncludingWhitespace().NumericValue(),
                        UnitType::kNumber,
                    ))
                } else {
                    None
                },
            )
        }
        CSSPropertyID::kAnimationRangeStart | CSSPropertyID::kAnimationRangeEnd => {
            ConsumeAnimationRange(
                id,
                stream,
                mode,
                if id == CSSPropertyID::kAnimationRangeStart {
                    0.0
                } else {
                    100.0
                },
            )
        }
        // cpp: longhands_custom.cc:121-136,651-657; css_parsing_utils.cc:4760-4799.
        CSSPropertyID::kAnimationTrigger => {
            if stream.Peek().Id() == CSSValueID::kNone {
                return Ok(Some(values::identifier(
                    stream.ConsumeIncludingWhitespace().Id(),
                )));
            }
            if stream.Peek().FunctionId() == Some(CSSValueID::kIdent) {
                return Err(unsupported(
                    id,
                    "ConsumeSingleAnimationTriggerAttachment ident()",
                ));
            }
            if stream.Peek().GetType() != kIdentToken
                || !stream.Peek().Value().ToString().Utf8().starts_with("--")
            {
                return Ok(None);
            }
            let name = ConsumeAnimationName(id, stream, false)?.unwrap();
            let behaviors = &[
                "play",
                "pause",
                "reset",
                "play-once",
                "play-alternate",
                "play-forwards",
                "play-backwards",
                "play-pause",
                "replay",
                "none",
            ];
            if !matches(stream.Peek().Id(), behaviors) {
                return Ok(None);
            }
            let enter = values::identifier(stream.ConsumeIncludingWhitespace().Id());
            let exit = if matches(stream.Peek().Id(), behaviors) {
                Some(values::identifier(stream.ConsumeIncludingWhitespace().Id()))
            } else {
                None
            };
            let mut value = Value::new(CSSValuePayload::kTriggerAttachmentClass(
                values::CSSTriggerAttachmentValue { name, enter, exit },
            ));
            value.StateMut().SetNeedsTreeScopePopulation(true);
            // Chromium currently rejects multiple attachments for one animation.
            // The enclosing declaration rejects remaining tokens after this item.
            Ok(Some(values::list(
                vec![Rc::new(value)],
                values::ListSeparator::Space,
            )))
        }
        CSSPropertyID::kAnimationTimeline => ConsumeAnimationTimeline(id, stream),
        _ => Ok(None),
    }
}
fn ValidateTransitionPropertyList(
    id: CSSPropertyID,
    items: &[Rc<Value>],
) -> Result<(), PropertyParseError> {
    if items.len() > 1 && items.iter().any(|value| matches!(value.Payload(), CSSValuePayload::kIdentifierClass(v) if v.0 == CSSValueID::kNone)) {
        return Err(invalid(id));
    }
    Ok(())
}
// cpp: css_parsing_utils.cc:4864-4943; shorthands_custom.cc:63-130,5533-5602;
// style_property_shorthand_custom.cc:27-40. Transition property parses last.
fn ParseAnimationShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    let longhands = if id == CSSPropertyID::kTransition {
        &[
            CSSPropertyID::kTransitionBehavior,
            CSSPropertyID::kTransitionDuration,
            CSSPropertyID::kTransitionTimingFunction,
            CSSPropertyID::kTransitionDelay,
            CSSPropertyID::kTransitionProperty,
        ][..]
    } else {
        ShorthandFor(id)
    };
    let reset_only = |property| {
        matches!(
            property,
            CSSPropertyID::kAnimationTimeline
                | CSSPropertyID::kAnimationRangeStart
                | CSSPropertyID::kAnimationRangeEnd
        )
    };
    let mut lists = vec![Vec::new(); longhands.len()];
    loop {
        let mut parsed = vec![false; longhands.len()];
        let mut found_any = false;
        loop {
            let mut found_property = false;
            for (index, &longhand) in longhands.iter().enumerate() {
                if parsed[index] || reset_only(longhand) {
                    continue;
                }
                if let Some(value) = ConsumeAnimationItem(longhand, stream, mode)? {
                    lists[index].push(value);
                    parsed[index] = true;
                    found_property = true;
                    found_any = true;
                    break;
                }
            }
            if !found_property || stream.AtEnd() || stream.Peek().GetType() == kCommaToken {
                break;
            }
        }
        if !found_any {
            return Err(invalid(id));
        }
        for (index, &longhand) in longhands.iter().enumerate() {
            if !parsed[index] && (!reset_only(longhand) || lists[index].is_empty()) {
                lists[index].push(AnimationInitialItem(longhand));
            }
        }
        if stream.Peek().GetType() != kCommaToken {
            break;
        }
        stream.ConsumeIncludingWhitespace();
    }
    for (&longhand, list) in longhands.iter().zip(lists) {
        if longhand == CSSPropertyID::kTransitionProperty {
            ValidateTransitionPropertyList(id, &list)?;
        }
        out.push(make_expanded(
            longhand,
            id,
            values::list(list, values::ListSeparator::Comma),
            false,
        ));
    }
    Ok(())
}
fn AnimationInitialItem(id: CSSPropertyID) -> Rc<Value> {
    match id {
        CSSPropertyID::kAnimationDuration | CSSPropertyID::kAnimationTimeline => {
            values::identifier(CSSValueID::kAuto)
        }
        CSSPropertyID::kAnimationTimingFunction | CSSPropertyID::kTransitionTimingFunction => {
            values::identifier(CSSValueID::kEase)
        }
        CSSPropertyID::kAnimationDelay
        | CSSPropertyID::kTransitionDelay
        | CSSPropertyID::kTransitionDuration => values::numeric(0.0, UnitType::kSeconds),
        CSSPropertyID::kAnimationIterationCount => values::numeric(1.0, UnitType::kNumber),
        CSSPropertyID::kAnimationDirection
        | CSSPropertyID::kAnimationRangeStart
        | CSSPropertyID::kAnimationRangeEnd
        | CSSPropertyID::kTransitionBehavior => values::identifier(CSSValueID::kNormal),
        CSSPropertyID::kAnimationFillMode | CSSPropertyID::kAnimationName => {
            values::identifier(CSSValueID::kNone)
        }
        CSSPropertyID::kAnimationPlayState => values::identifier(CSSValueID::kRunning),
        CSSPropertyID::kTransitionProperty => values::identifier(CSSValueID::kAll),
        _ => unreachable!("source animation shorthand initial item"),
    }
}
// cpp: css_parsing_utils.cc:5106-5218,5279-5319; longhands_custom.cc:
// BackgroundAttachment:949, BackgroundBlendMode:970, BackgroundClip:991,
// BackgroundOrigin:1148; MaskClip:11452, MaskComposite:11483, MaskMode:11538,
// MaskOrigin:11558. Alias parsing follows CSSParserLocalContext::UseAliasParsing.
fn ConsumeLayer<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    alias: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    if matches!(
        id,
        CSSPropertyID::kBackgroundPositionX
            | CSSPropertyID::kBackgroundPositionY
            | CSSPropertyID::kWebkitMaskPositionX
            | CSSPropertyID::kWebkitMaskPositionY
    ) {
        return ConsumePositionAxis(id, stream, mode, true);
    }
    if matches!(
        id,
        CSSPropertyID::kBackgroundSize | CSSPropertyID::kMaskSize
    ) {
        return ConsumeBackgroundSize(
            id,
            stream,
            mode,
            alias && id == CSSPropertyID::kBackgroundSize,
        );
    }
    let keyword = stream.Peek().Id();
    let valid = match id {
        CSSPropertyID::kBackgroundAttachment => matches(keyword, &["scroll", "fixed", "local"]),
        CSSPropertyID::kBackgroundBlendMode => {
            keyword == CSSValueID::kNormal
                || keyword == CSSValueID::kOverlay
                || (CSSValueID::kMultiply as i32..=CSSValueID::kLuminosity as i32)
                    .contains(&(keyword as i32))
        }
        CSSPropertyID::kBackgroundOrigin
        | CSSPropertyID::kMaskOrigin
        | CSSPropertyID::kMaskClip
            if alias =>
        {
            (CSSValueID::kBorder as i32..=CSSValueID::kPaddingBox as i32)
                .contains(&(keyword as i32))
                || (id == CSSPropertyID::kMaskClip && keyword == CSSValueID::kText)
        }
        CSSPropertyID::kBackgroundOrigin => {
            matches(keyword, &["border-box", "padding-box", "content-box"])
        }
        CSSPropertyID::kBackgroundClip => {
            matches(keyword, &["border-box", "padding-box", "content-box"])
        }
        CSSPropertyID::kMaskOrigin | CSSPropertyID::kMaskClip => {
            matches(
                keyword,
                &[
                    "content-box",
                    "padding-box",
                    "border-box",
                    "fill-box",
                    "stroke-box",
                    "view-box",
                ],
            ) || (id == CSSPropertyID::kMaskClip && keyword == CSSValueID::kNoClip)
        }
        CSSPropertyID::kMaskComposite if alias => (CSSValueID::kClear as i32
            ..=CSSValueID::kPlusLighter as i32)
            .contains(&(keyword as i32)),
        CSSPropertyID::kMaskComposite => {
            matches(keyword, &["add", "subtract", "intersect", "exclude"])
        }
        CSSPropertyID::kMaskMode => matches(keyword, &["alpha", "luminance", "match-source"]),
        _ => false,
    };
    if valid {
        stream.ConsumeIncludingWhitespace();
        return Ok(values::identifier(keyword));
    }
    // cpp: css_parsing_utils.cc:5127-5163 [ border-area || text ], stable flag.
    if id == CSSPropertyID::kBackgroundClip {
        let mut text = false;
        let mut border = false;
        if stream.Peek().Id() == CSSValueID::kText {
            text = true;
            stream.ConsumeIncludingWhitespace();
        }
        if !alias && stream.Peek().Id() == CSSValueID::kBorderArea {
            border = true;
            stream.ConsumeIncludingWhitespace();
        }
        if !text && stream.Peek().Id() == CSSValueID::kText {
            text = true;
            stream.ConsumeIncludingWhitespace();
        }
        return match (border, text) {
            (true, true) => Ok(Pair(
                values::identifier(CSSValueID::kBorderArea),
                values::identifier(CSSValueID::kText),
                true,
            )),
            (true, false) => Ok(values::identifier(CSSValueID::kBorderArea)),
            (false, true) => Ok(values::identifier(CSSValueID::kText)),
            _ => Err(invalid(id)),
        };
    }
    Err(invalid(id))
}
fn Pair(first: Rc<Value>, second: Rc<Value>, drop_identical: bool) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kValuePairClass(
        values::CSSValuePair {
            first,
            second,
            drop_identical,
        },
    )))
}
// cpp: css_parsing_utils.cc:2529-2777. Shared positional grammar keeps the
// tokenizer savepoints used to fall back from three/four values to one/two.
fn PositionKeyword(value: &Value) -> Option<CSSValueID> {
    match value.Payload() {
        CSSValuePayload::kIdentifierClass(value) => Some(value.0),
        _ => None,
    }
}
fn HorizontalPositionOnly(value: &Value) -> bool {
    matches!(
        PositionKeyword(value),
        Some(CSSValueID::kLeft | CSSValueID::kRight)
    )
}
fn VerticalPositionOnly(value: &Value) -> bool {
    matches!(
        PositionKeyword(value),
        Some(CSSValueID::kTop | CSSValueID::kBottom)
    )
}
fn PositionFromOneValue(value: Rc<Value>) -> (Rc<Value>, Rc<Value>) {
    let center = values::identifier(CSSValueID::kCenter);
    if VerticalPositionOnly(&value) {
        (center, value)
    } else {
        (value, center)
    }
}
fn PositionFromTwoValues(first: Rc<Value>, second: Rc<Value>) -> (Rc<Value>, Rc<Value>) {
    let xy = HorizontalPositionOnly(&first)
        || VerticalPositionOnly(&second)
        || !first.IsIdentifierValue()
        || !second.IsIdentifierValue();
    let yx = VerticalPositionOnly(&first) || HorizontalPositionOnly(&second);
    debug_assert!(!xy || !yx);
    if yx {
        (second, first)
    } else {
        (first, second)
    }
}
fn ConsumePositionLength<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    quirks: bool,
    percent: bool,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if IsMathFunction(stream) {
        if matches!(
            id,
            CSSPropertyID::kTransformOrigin
                | CSSPropertyID::kPerspectiveOrigin
                | CSSPropertyID::kWebkitTransformOriginX
                | CSSPropertyID::kWebkitTransformOriginY
                | CSSPropertyID::kWebkitPerspectiveOriginX
                | CSSPropertyID::kWebkitPerspectiveOriginY
        ) {
            use crate::css_math_expression_node::CalculationResultCategory as C;
            return ConsumeMath(
                id,
                stream,
                if percent {
                    &[C::Length, C::Percent, C::LengthFunction]
                } else {
                    &[C::Length]
                },
                crate::css_math_function_value::ValueRange::All,
            )
            .map(Some);
        }
        return Err(unsupported(
            id,
            "ConsumePosition CSSMathFunctionValue / negative percentage reference",
        ));
    }
    match ConsumeLiteral(
        id,
        stream,
        mode,
        Grammar::Length {
            percent,
            nonnegative: false,
            quirks,
            keywords: &[],
        },
    ) {
        Ok(value) => Ok(Some(value)),
        // Other functions do not match the source math/length consumer.
        Err(_) => Ok(None),
    }
}
fn ConsumePositionComponent<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    quirks: bool,
    horizontal: &mut bool,
    vertical: &mut bool,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if stream.Peek().GetType() != kIdentToken {
        return ConsumePositionLength(id, stream, mode, quirks, true);
    }
    let keyword = stream.Peek().Id();
    if matches!(keyword, CSSValueID::kLeft | CSSValueID::kRight) {
        if *horizontal {
            return Ok(None);
        }
        *horizontal = true;
    } else if matches!(keyword, CSSValueID::kTop | CSSValueID::kBottom) {
        if *vertical {
            return Ok(None);
        }
        *vertical = true;
    } else if keyword != CSSValueID::kCenter {
        return Ok(None);
    }
    stream.ConsumeIncludingWhitespace();
    Ok(Some(values::identifier(keyword)))
}
fn PositionFromThreeOrFourValues(items: Vec<Rc<Value>>) -> (Rc<Value>, Rc<Value>) {
    let mut x = None;
    let mut y = None;
    let mut center = None;
    let mut index = 0;
    while index < items.len() {
        let keyword = PositionKeyword(&items[index]).expect("source validated positional edge");
        if keyword == CSSValueID::kCenter {
            debug_assert!(center.is_none());
            center = Some(items[index].clone());
            index += 1;
            continue;
        }
        let value = if index + 1 < items.len() && !items[index + 1].IsIdentifierValue() {
            index += 1;
            Pair(items[index - 1].clone(), items[index].clone(), false)
        } else {
            items[index].clone()
        };
        if matches!(keyword, CSSValueID::kLeft | CSSValueID::kRight) {
            debug_assert!(x.is_none());
            x = Some(value);
        } else {
            debug_assert!(matches!(keyword, CSSValueID::kTop | CSSValueID::kBottom));
            debug_assert!(y.is_none());
            y = Some(value);
        }
        index += 1;
    }
    if let Some(center) = center {
        debug_assert_ne!(x.is_some(), y.is_some());
        if x.is_none() {
            x = Some(center);
        } else {
            y = Some(center);
        }
    }
    (
        x.expect("source validated x position"),
        y.expect("source validated y position"),
    )
}
fn ConsumePosition<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    quirks: bool,
    allow_three: bool,
) -> Result<(Rc<Value>, Rc<Value>), PropertyParseError> {
    let mut horizontal = false;
    let mut vertical = false;
    let first = ConsumePositionComponent(id, stream, mode, quirks, &mut horizontal, &mut vertical)?
        .ok_or_else(|| invalid(id))?;
    if !first.IsIdentifierValue() {
        horizontal = true;
    }
    stream.EnsureLookAhead();
    let after_first = stream.Save();
    let Some(second) =
        ConsumePositionComponent(id, stream, mode, quirks, &mut horizontal, &mut vertical)?
    else {
        return Ok(PositionFromOneValue(first));
    };
    stream.EnsureLookAhead();
    let after_second = stream.Save();
    let first_keyword = PositionKeyword(&first);
    let second_keyword = PositionKeyword(&second);
    let third = if first_keyword.is_some()
        && second_keyword.is_some() != (stream.Peek().GetType() == kIdentToken)
        && second_keyword.or(first_keyword) != Some(CSSValueID::kCenter)
    {
        ConsumePositionComponent(id, stream, mode, quirks, &mut horizontal, &mut vertical)?
    } else {
        None
    };
    let Some(third) = third else {
        if vertical && !second.IsIdentifierValue() {
            stream.EnsureLookAhead();
            stream.Restore(after_first);
            return Ok(PositionFromOneValue(first));
        }
        return Ok(PositionFromTwoValues(first, second));
    };
    let third_keyword = PositionKeyword(&third);
    let fourth = if third_keyword.is_some()
        && third_keyword != Some(CSSValueID::kCenter)
        && stream.Peek().GetType() != kIdentToken
    {
        ConsumePositionComponent(id, stream, mode, quirks, &mut horizontal, &mut vertical)?
    } else {
        None
    };
    if fourth.is_none() && !allow_three {
        stream.EnsureLookAhead();
        if vertical && !second.IsIdentifierValue() {
            stream.Restore(after_first);
            return Ok(PositionFromOneValue(first));
        }
        stream.Restore(after_second);
        return Ok(PositionFromTwoValues(first, second));
    }
    let mut items = vec![first, second, third];
    if let Some(fourth) = fourth {
        items.push(fourth);
    }
    Ok(PositionFromThreeOrFourValues(items))
}
// cpp: css_parsing_utils.cc:2746-2777; longhands_custom.cc:10487-10507.
fn ConsumeTransformOrigin<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let mut horizontal = false;
    let mut vertical = false;
    let first = ConsumePositionComponent(id, stream, mode, false, &mut horizontal, &mut vertical)?
        .ok_or_else(|| invalid(id))?;
    if !first.IsIdentifierValue() {
        horizontal = true;
    }
    if vertical && ConsumePositionLength(id, stream, mode, false, true)?.is_some() {
        return Err(invalid(id));
    }
    let second = ConsumePositionComponent(id, stream, mode, false, &mut horizontal, &mut vertical)?;
    let (x, y) = if let Some(second) = second {
        PositionFromTwoValues(first, second)
    } else {
        PositionFromOneValue(first)
    };
    let mut items = vec![x, y];
    if let Some(z) = ConsumePositionLength(id, stream, mode, false, false)? {
        items.push(z);
    }
    Ok(values::list(items, values::ListSeparator::Space))
}
// cpp: css_parsing_utils.cc:5551-5578; longhands_custom.cc:1219-1224,11639-11644.
fn ConsumeRepeatStyleValue<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    let keyword = stream.Peek().Id();
    let (x, y) = if matches!(keyword, CSSValueID::kRepeatX | CSSValueID::kRepeatY) {
        stream.ConsumeIncludingWhitespace();
        if keyword == CSSValueID::kRepeatX {
            (CSSValueID::kRepeat, CSSValueID::kNoRepeat)
        } else {
            (CSSValueID::kNoRepeat, CSSValueID::kRepeat)
        }
    } else if matches(keyword, &["repeat", "no-repeat", "round", "space"]) {
        stream.ConsumeIncludingWhitespace();
        let second = if matches(
            stream.Peek().Id(),
            &["repeat", "no-repeat", "round", "space"],
        ) {
            stream.ConsumeIncludingWhitespace().Id()
        } else {
            keyword
        };
        (keyword, second)
    } else {
        return Err(invalid(id));
    };
    Ok(Rc::new(Value::new(CSSValuePayload::kRepeatStyleClass(
        values::CSSRepeatStyleValue {
            x: values::identifier(x),
            y: values::identifier(y),
        },
    ))))
}
// cpp: css_parsing_utils.h:995-1043 ConsumePositionLonghand /
// ConsumeBackgroundPositionLonghand (SideRelativeBackgroundPosition is stable).
fn ConsumePositionAxis<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    background: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    let horizontal = matches!(
        id,
        CSSPropertyID::kBackgroundPositionX
            | CSSPropertyID::kWebkitMaskPositionX
            | CSSPropertyID::kWebkitPerspectiveOriginX
            | CSSPropertyID::kWebkitTransformOriginX
    );
    let start = if horizontal {
        CSSValueID::kLeft
    } else {
        CSSValueID::kTop
    };
    let end = if horizontal {
        CSSValueID::kRight
    } else {
        CSSValueID::kBottom
    };
    let keyword = stream.Peek().Id();
    let origin = if keyword == start || keyword == end || keyword == CSSValueID::kCenter {
        stream.ConsumeIncludingWhitespace();
        Some(keyword)
    } else if stream.Peek().GetType() == kIdentToken {
        return Err(invalid(id));
    } else {
        None
    };
    if !background {
        if let Some(origin) = origin {
            return Ok(values::numeric(
                if origin == start {
                    0.0
                } else if origin == end {
                    100.0
                } else {
                    50.0
                },
                UnitType::kPercentage,
            ));
        }
    } else if origin == Some(CSSValueID::kCenter) {
        return Ok(values::identifier(CSSValueID::kCenter));
    }
    let grammar = Grammar::Length {
        percent: true,
        nonnegative: false,
        quirks: false,
        keywords: &[],
    };
    if origin.is_some()
        && !matches!(
            stream.Peek().GetType(),
            kNumberToken | kPercentageToken | kDimensionToken | kFunctionToken
        )
    {
        return Ok(values::identifier(origin.unwrap()));
    }
    let offset = ConsumeLiteral(id, stream, mode, grammar)?;
    Ok(if let Some(origin) = origin {
        Pair(values::identifier(origin), offset, true)
    } else {
        offset
    })
}
// cpp: css_parsing_utils.cc:5180-5241 ConsumeBackgroundSize.
fn ConsumeBackgroundSize<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    legacy: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    if matches(stream.Peek().Id(), &["contain", "cover"]) {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    let grammar = Grammar::Length {
        percent: true,
        nonnegative: true,
        quirks: false,
        keywords: &["auto"],
    };
    let horizontal = ConsumeLiteral(id, stream, mode, grammar)?;
    let vertical = if stream.Peek().Id() == CSSValueID::kAuto
        || matches!(
            stream.Peek().GetType(),
            kNumberToken | kPercentageToken | kDimensionToken | kFunctionToken
        ) {
        Some(ConsumeLiteral(id, stream, mode, grammar)?)
    } else if stream.AtEnd() && legacy {
        Some(horizontal.clone())
    } else {
        None
    };
    if legacy && vertical.is_none() {
        return Ok(horizontal);
    }
    let vertical = vertical.unwrap_or_else(|| values::identifier(CSSValueID::kAuto));
    if horizontal.IsIdentifierValue()
        && horizontal.CssText().Utf8() == "auto"
        && vertical.IsIdentifierValue()
        && vertical.CssText().Utf8() == "auto"
    {
        return Ok(horizontal);
    }
    Ok(Pair(horizontal, vertical, false))
}
// cpp: longhands_custom.cc:7676-7720 OverflowClipMargin::ParseSingleValue.
fn ConsumeOverflowClipMargin<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let first = stream.Peek().GetType();
    if !matches!(first, kIdentToken | kDimensionToken) {
        return Err(invalid(id));
    }
    let mut reference = None;
    let mut length = None;
    let grammar = Grammar::Length {
        percent: false,
        nonnegative: false,
        quirks: false,
        keywords: &[],
    };
    if first == kIdentToken {
        if matches(
            stream.Peek().Id(),
            &["content-box", "padding-box", "border-box"],
        ) {
            reference = Some(stream.ConsumeIncludingWhitespace().Id());
        }
        if matches!(
            stream.Peek().GetType(),
            kNumberToken | kDimensionToken | kFunctionToken
        ) {
            length = Some(ConsumeLiteral(id, stream, mode, grammar)?);
        }
    } else {
        length = Some(ConsumeLiteral(id, stream, mode, grammar)?);
        if matches(
            stream.Peek().Id(),
            &["content-box", "padding-box", "border-box"],
        ) {
            reference = Some(stream.ConsumeIncludingWhitespace().Id());
        }
    }
    if reference.is_none() && length.is_none() {
        return Err(invalid(id));
    }
    if reference == Some(CSSValueID::kPaddingBox) {
        reference = None;
        if length.is_none() {length = Some(values::numeric(0.0, UnitType::kPixels));}
    } else if reference.is_some() && length.as_ref().is_some_and(|v| matches!(v.Payload(), CSSValuePayload::kNumericLiteralClass(n) if n.DoubleValue() == 0.0)) {
        length = None;
    }
    let mut items = Vec::new();
    if let Some(reference) = reference {
        items.push(values::identifier(reference));
    }
    if let Some(length) = length {
        items.push(length);
    }
    Ok(values::list(items, values::ListSeparator::Space))
}
// cpp: css_parsing_utils.cc:7291-7385 ConsumeGridLine, numeric/auto/span branches.
// CSSCustomIdentValue and tree-scoped names remain explicitly unavailable.
fn ConsumeGridLine<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kAuto {
        stream.ConsumeIncludingWhitespace();
        return Ok(values::identifier(CSSValueID::kAuto));
    }
    let mut span = false;
    if stream.Peek().Id() == CSSValueID::kSpan {
        span = true;
        stream.ConsumeIncludingWhitespace();
    }
    if stream.Peek().GetType() == kIdentToken {
        return Err(GridLineIdentError(id, stream.Peek().Id()));
    }
    let count = ConsumeLiteral(
        id,
        stream,
        mode,
        Grammar::Integer {
            minimum: if span { 1 } else { i32::MIN },
        },
    )?;
    if matches!(count.Payload(), CSSValuePayload::kNumericLiteralClass(n) if n.DoubleValue() == 0.0)
    {
        return Err(invalid(id));
    }
    if !span && stream.Peek().Id() == CSSValueID::kSpan {
        span = true;
        stream.ConsumeIncludingWhitespace();
    }
    if span
        && matches!(count.Payload(), CSSValuePayload::kNumericLiteralClass(n) if n.DoubleValue() < 1.0)
    {
        return Err(invalid(id));
    }
    if stream.Peek().GetType() == kIdentToken {
        return Err(GridLineIdentError(id, stream.Peek().Id()));
    }
    let mut items = Vec::new();
    if span {
        items.push(values::identifier(CSSValueID::kSpan));
    }
    items.push(count);
    Ok(values::list(items, values::ListSeparator::Space))
}
// cpp: css_parsing_utils.cc:7037-7045 ConsumeCustomIdentForGridLine;
// ConsumeCustomIdent excludes CSS-wide keywords and default before creating a value.
fn GridLineIdentError(id: CSSPropertyID, keyword: CSSValueID) -> PropertyParseError {
    if matches!(
        keyword,
        CSSValueID::kAuto | CSSValueID::kSpan | CSSValueID::kDefault
    ) || values::wide(keyword).is_some()
    {
        invalid(id)
    } else {
        unsupported(id, "ConsumeCustomIdentForGridLine CSSCustomIdentValue")
    }
}
// cpp: longhands_custom.cc ValidateDisplayKeywords / AdjustDisplayKeywords /
// ParseDisplayMultipleKeywords:3650-3780. Source adjusts compatible pairs to
// legacy identifiers; only residual compound values require CSSValueList.
fn ConsumeDisplayMultiple<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    first: CSSValueID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    let mut keywords = vec![first];
    for _ in 0..2 {
        if stream.Peek().GetType() == kIdentToken {
            keywords.push(stream.ConsumeIncludingWhitespace().Id());
        }
    }
    let mut outside = None;
    let mut inside = None;
    let mut list_item = false;
    for keyword in keywords {
        if outside.is_none() && matches!(keyword, CSSValueID::kInline | CSSValueID::kBlock) {
            outside = Some(keyword);
        } else if inside.is_none()
            && ((CSSValueID::kFlowRoot as i32..CSSValueID::kGridLanes as i32)
                .contains(&(keyword as i32))
                || matches!(keyword, CSSValueID::kMath | CSSValueID::kRuby))
        {
            inside = Some(keyword);
        } else if !list_item && keyword == CSSValueID::kListItem {
            list_item = true;
        } else {
            return Err(invalid(id));
        }
    }
    if list_item
        && inside
            .is_some_and(|keyword| !matches!(keyword, CSSValueID::kFlow | CSSValueID::kFlowRoot))
    {
        return Err(invalid(id));
    }
    let original_outside = outside;
    let original_inside = inside;
    match inside {
        Some(CSSValueID::kFlow) if outside.is_some() => inside = None,
        Some(
            CSSValueID::kFlex | CSSValueID::kFlowRoot | CSSValueID::kGrid | CSSValueID::kTable,
        ) => {
            if outside == Some(CSSValueID::kBlock) {
                outside = None;
            } else if outside == Some(CSSValueID::kInline) && !list_item {
                inside = Some(match inside.unwrap() {
                    CSSValueID::kFlex => CSSValueID::kInlineFlex,
                    CSSValueID::kFlowRoot => CSSValueID::kInlineBlock,
                    CSSValueID::kGrid => CSSValueID::kInlineGrid,
                    _ => CSSValueID::kInlineTable,
                });
                outside = None;
            }
        }
        Some(CSSValueID::kMath | CSSValueID::kRuby) if outside == Some(CSSValueID::kInline) => {
            outside = None
        }
        _ => {}
    }
    if list_item {
        if original_outside == Some(CSSValueID::kBlock) {
            outside = None;
        }
        if original_inside == Some(CSSValueID::kFlow) {
            inside = None;
        }
    }
    let mut result = Vec::new();
    if let Some(keyword) = outside {
        result.push(values::identifier(keyword));
    }
    if let Some(keyword) = inside {
        result.push(values::identifier(keyword));
    }
    if list_item {
        result.push(values::identifier(CSSValueID::kListItem));
    }
    if result.len() == 1 {
        Ok(result.remove(0))
    } else {
        Ok(values::list(result, values::ListSeparator::Space))
    }
}
// cpp: ua_counter_style_map.cc:28-467 CollectUACounterStyleRules.
// Name keys of Chromium's UA map, used by ShouldLowerCaseCounterStyleNameOnParse.
// This is shared grammar metadata; authored counter-style names stay case sensitive.
const UA_COUNTER_STYLE_NAMES: &[&str] = &[
    "decimal-leading-zero",
    "arabic-indic",
    "armenian",
    "upper-armenian",
    "lower-armenian",
    "bengali",
    "cambodian",
    "khmer",
    "cjk-decimal",
    "devanagari",
    "georgian",
    "gujarati",
    "gurmukhi",
    "hebrew",
    "kannada",
    "lao",
    "malayalam",
    "mongolian",
    "myanmar",
    "oriya",
    "persian",
    "tamil",
    "telugu",
    "thai",
    "tibetan",
    "lower-latin",
    "upper-latin",
    "lower-greek",
    "hiragana",
    "hiragana-iroha",
    "katakana",
    "katakana-iroha",
    "cjk-earthly-branch",
    "cjk-heavenly-stem",
    "japanese-informal",
    "japanese-formal",
    "korean-hangul-formal",
    "korean-hanja-informal",
    "korean-hanja-formal",
    "simp-chinese-informal",
    "simp-chinese-formal",
    "trad-chinese-informal",
    "trad-chinese-formal",
    "cjk-ideographic",
    "ethiopic-numeric",
    "ethiopic-halehame",
    "ethiopic-halehame-am",
    "ethiopic-halehame-ti-er",
    "ethiopic-halehame-ti-et",
    "hangul",
    "hangul-consonant",
    "urdu",
    "decimal",
    "lower-roman",
    "upper-roman",
    "lower-alpha",
    "upper-alpha",
    "disc",
    "circle",
    "square",
    "disclosure-open",
    "disclosure-closed",
];
// cpp: longhands_custom.cc:3000-3056 ConsumeCounterContent.
fn ConsumeCounterContent<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    counters: bool,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    let value;
    {
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        if guard.Peek().FunctionId() == Some(CSSValueID::kIdent) {
            return Err(unsupported(
                id,
                "ConsumeCustomIdent ident() TreeScope binding",
            ));
        }
        let token = guard.Peek();
        if token.GetType() != kIdentToken
            || values::wide(token.Id()).is_some()
            || token.Id() == CSSValueID::kDefault
        {
            return Ok(None);
        }
        let identifier = values::custom_ident(
            &guard.ConsumeIncludingWhitespace().Value().ToString(),
            CSSPropertyID::kInvalid,
        );
        let separator = if counters {
            if guard.Peek().GetType() != kCommaToken {
                return Ok(None);
            }
            guard.ConsumeIncludingWhitespace();
            if guard.Peek().GetType() != kStringToken {
                return Ok(None);
            }
            guard.ConsumeIncludingWhitespace().Value().ToString()
        } else {
            String::default()
        };
        let list_style = if guard.Peek().GetType() == kCommaToken {
            guard.ConsumeIncludingWhitespace();
            if guard.Peek().FunctionId() == Some(CSSValueID::kSymbols) {
                return Err(unsupported(
                    id,
                    "ConsumeCounterStyleSymbolsFunction anonymous CounterStyle",
                ));
            }
            let token = guard.Peek();
            if token.GetType() != kIdentToken
                || values::wide(token.Id()).is_some()
                || token.Id() == CSSValueID::kDefault
            {
                return Ok(None);
            }
            // cpp: css_parsing_utils.cc:9813-9839. Normalize predefined UA
            // styles only, preserving case in author-defined counter-style names.
            let is_none = token.Id() == CSSValueID::kNone;
            let name = guard.ConsumeIncludingWhitespace().Value().ToString();
            let lower = name.Utf8().to_ascii_lowercase();
            let name = if is_none || UA_COUNTER_STYLE_NAMES.contains(&lower.as_str()) {
                String::from(lower.as_str())
            } else {
                name
            };
            values::custom_ident(&name, CSSPropertyID::kInvalid)
        } else {
            values::custom_ident(&String::from("decimal"), CSSPropertyID::kInvalid)
        };
        if !guard.AtEnd() || !guard.Release() {
            return Ok(None);
        }
        value = values::counter_content(identifier, list_style, separator);
    }
    stream.ConsumeWhitespace();
    Ok(Some(value))
}
// cpp: css_parsing_utils.cc:1986-2027,3860-3870 ConsumeUrlAsToken/ConsumeImage.
fn ConsumeUrl<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Option<String>, PropertyParseError> {
    let start = stream.LookAheadOffset();
    let url = if stream.Peek().GetType() == kUrlToken {
        stream.ConsumeIncludingWhitespace().Value().ToString()
    } else if stream.Peek().FunctionId() == Some(CSSValueID::kUrl) {
        let url;
        {
            let mut guard = RestoringBlockGuard::new(stream);
            guard.ConsumeWhitespace();
            if guard.Peek().GetType() != kStringToken {
                return Ok(None);
            }
            url = guard.ConsumeIncludingWhitespace().Value().ToString();
            if !guard.AtEnd() {
                return Err(unsupported(id, "ConsumeUrlRequestModifiers"));
            }
            if !guard.Release() {
                return Ok(None);
            }
        }
        stream.ConsumeWhitespace();
        url
    } else {
        return Ok(None);
    };
    let end = stream.LookAheadOffset();
    if stream.IsAttrTainted(start, end) {
        return Err(invalid(id));
    }
    Ok(Some(url))
}
fn ConsumeContentImage<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    Ok(ConsumeUrl(id, stream)?.map(values::image))
}
// cpp: longhands_custom.cc:3058-3138 ParseContentValue.
// attr() reaches this consumer through shared typed-token cascade substitution.
fn ConsumeContent<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    if matches!(stream.Peek().Id(), CSSValueID::kNormal | CSSValueID::kNone) {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    let mut items = Vec::new();
    let mut alt_present = false;
    loop {
        let value = if stream.Peek().FunctionId() == Some(CSSValueID::kLinearGradient) {
            // cpp: css_parsing_utils.cc ConsumeImage/ConsumeGeneratedImage.
            // Reuse background/mask's typed linear-gradient consumer.
            Some(ConsumeLinearGradient(id, stream, mode)?)
        } else if let Some(image) = ConsumeContentImage(id, stream)? {
            Some(image)
        } else if matches!(
            stream.Peek().Id(),
            CSSValueID::kOpenQuote
                | CSSValueID::kCloseQuote
                | CSSValueID::kNoOpenQuote
                | CSSValueID::kNoCloseQuote
        ) {
            Some(values::identifier(stream.ConsumeIncludingWhitespace().Id()))
        } else if stream.Peek().GetType() == kStringToken {
            Some(values::string(
                stream.ConsumeIncludingWhitespace().Value().ToString(),
            ))
        } else if matches!(
            stream.Peek().FunctionId(),
            Some(CSSValueID::kCounter | CSSValueID::kCounters)
        ) {
            let counters = stream.Peek().FunctionId() == Some(CSSValueID::kCounters);
            ConsumeCounterContent(id, stream, counters)?
        } else {
            // cpp: css_parsing_utils.cc:937-964,3888-3923 image function dispatch.
            if matches!(
                stream.Peek().FunctionId(),
                Some(
                    CSSValueID::kLinearGradient
                        | CSSValueID::kRadialGradient
                        | CSSValueID::kConicGradient
                        | CSSValueID::kRepeatingLinearGradient
                        | CSSValueID::kRepeatingRadialGradient
                        | CSSValueID::kRepeatingConicGradient
                        | CSSValueID::kWebkitLinearGradient
                        | CSSValueID::kWebkitRadialGradient
                        | CSSValueID::kWebkitRepeatingLinearGradient
                        | CSSValueID::kWebkitRepeatingRadialGradient
                        | CSSValueID::kWebkitGradient
                        | CSSValueID::kWebkitCrossFade
                        | CSSValueID::kCrossFade
                        | CSSValueID::kPaint
                        | CSSValueID::kImage
                        | CSSValueID::kImageSet
                        | CSSValueID::kWebkitImageSet
                        | CSSValueID::kLightDark
                )
            ) {
                return Err(unsupported(
                    id,
                    "ConsumeImage generated/image-set/light-dark collaborators",
                ));
            }
            None
        };
        if let Some(value) = value {
            items.push(value);
        } else if stream.Peek().GetType() == kDelimiterToken
            && stream.Peek().Delimiter() == b'/' as u16
        {
            if items.is_empty() {
                return Err(invalid(id));
            }
            stream.ConsumeIncludingWhitespace();
            alt_present = true;
            break;
        } else {
            break;
        }
        if stream.AtEnd() {
            break;
        }
    }
    if items.is_empty() {
        return Err(invalid(id));
    }
    let mut outer = vec![values::list(items, values::ListSeparator::Space)];
    if alt_present {
        let mut alt = Vec::new();
        loop {
            let value = if matches!(
                stream.Peek().FunctionId(),
                Some(CSSValueID::kCounter | CSSValueID::kCounters)
            ) {
                let counters = stream.Peek().FunctionId() == Some(CSSValueID::kCounters);
                ConsumeCounterContent(id, stream, counters)?
            } else if stream.Peek().GetType() == kStringToken {
                Some(values::string(
                    stream.ConsumeIncludingWhitespace().Value().ToString(),
                ))
            } else {
                None
            };
            let Some(value) = value else {
                break;
            };
            alt.push(value);
            if stream.AtEnd() {
                break;
            }
        }
        if alt.is_empty() {
            return Err(invalid(id));
        }
        outer.push(values::list(alt, values::ListSeparator::Space));
    }
    Ok(values::list(outer, values::ListSeparator::Slash))
}

fn ConsumeLiteral<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    grammar: Grammar,
) -> Result<Rc<Value>, PropertyParseError> {
    if matches!(grammar, Grammar::Unsupported) {
        return Err(unsupported(
            id,
            super::production_property_metadata::ParserSourceFor(id).2,
        ));
    }
    if matches!(grammar, Grammar::Content) {
        return ConsumeContent(id, stream, mode);
    }
    if matches!(grammar, Grammar::Color) {
        return ConsumeColor(id, stream);
    }
    if IsMathFunction(stream) {
        use crate::css_math_expression_node::CalculationResultCategory as C;
        use crate::css_math_function_value::ValueRange as R;
        let (categories, range): (&[C], R) = match grammar {
            Grammar::Number { nonnegative } => (
                &[C::Number],
                if nonnegative { R::NonNegative } else { R::All },
            ),
            Grammar::Integer { minimum } => (
                &[C::Number],
                match minimum {
                    0 => R::NonNegativeInteger,
                    1 => R::PositiveInteger,
                    _ => R::Integer,
                },
            ),
            Grammar::Length {
                percent,
                nonnegative,
                ..
            } => (
                if percent {
                    &[C::Length, C::Percent, C::LengthFunction]
                } else {
                    &[C::Length]
                },
                if nonnegative { R::NonNegative } else { R::All },
            ),
            _ => return Err(invalid(id)),
        };
        return ConsumeMath(id, stream, categories, range);
    }
    if is_function(stream) {
        return Err(if matches!(grammar, Grammar::Keywords(_)) {
            invalid(id)
        } else {
            unsupported(id, "CSSMathFunctionValue / CSSPendingSubstitutionValue")
        });
    }
    let token = stream.Peek().clone();
    let number = if matches!(
        token.GetType(),
        kNumberToken | kPercentageToken | kDimensionToken
    ) {
        token.NumericValue()
    } else {
        0.0
    };
    let value = match grammar {
        Grammar::Keywords(words) if matches(token.Id(), words) => values::identifier(token.Id()),
        Grammar::Number { nonnegative }
            if token.GetType() == kNumberToken && (!nonnegative || number >= 0.0) =>
        {
            values::numeric(number, UnitType::kNumber)
        }
        Grammar::Integer { minimum }
            if token.GetType() == kNumberToken
                && token.GetNumericValueType() == NumericValueType::kIntegerValueType
                && number >= minimum as f64 =>
        {
            values::numeric(number, UnitType::kInteger)
        }
        Grammar::Length { keywords, .. } if matches(token.Id(), keywords) => {
            values::identifier(token.Id())
        }
        Grammar::Length {
            percent,
            nonnegative,
            quirks,
            ..
        } if !nonnegative || number >= 0.0 => {
            let unit = match token.GetType() {
                kDimensionToken
                    if crate::css_numeric_literal_value::IsLength(token.GetUnitType()) =>
                {
                    token.GetUnitType()
                }
                kDimensionToken
                    if token.GetUnitType() == UnitType::kQuirkyEms
                        && mode == CSSParserMode::kUASheetMode =>
                {
                    token.GetUnitType()
                }
                kPercentageToken if percent => UnitType::kPercentage,
                kNumberToken
                    if number == 0.0
                        || mode == CSSParserMode::kSVGAttributeMode
                        || (quirks && mode == CSSParserMode::kHTMLQuirksMode) =>
                {
                    if mode == CSSParserMode::kSVGAttributeMode {
                        UnitType::kUserUnits
                    } else {
                        UnitType::kPixels
                    }
                }
                _ => return Err(invalid(id)),
            };
            values::numeric(number, unit)
        }
        _ => return Err(invalid(id)),
    };
    stream.ConsumeIncludingWhitespace();
    Ok(value)
}
// cpp: longhands_custom.cc:1122-1131; css_parsing_utils.cc:3273-3349,
// 3029-3087,3860-3901. URL tokens share ConsumeContentImage's existing parser.
fn ConsumeBackgroundImage<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let mut layers = Vec::new();
    loop {
        let value = if stream.Peek().Id() == CSSValueID::kNone {
            values::identifier(stream.ConsumeIncludingWhitespace().Id())
        } else if let Some(image) = ConsumeContentImage(id, stream)? {
            image
        } else if matches!(
            stream.Peek().FunctionId(),
            Some(CSSValueID::kLinearGradient | CSSValueID::kRepeatingLinearGradient)
        ) {
            ConsumeLinearGradient(id, stream, mode)?
        } else {
            return Err(if is_function(stream) {
                unsupported(id, "ConsumeImage generated/image-set subtype")
            } else {
                invalid(id)
            });
        };
        layers.push(value);
        if stream.Peek().GetType() != kCommaToken {
            break;
        }
        stream.ConsumeIncludingWhitespace();
    }
    Ok(values::list(layers, values::ListSeparator::Comma))
}
fn ConsumeLinearGradient<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    fn optional<T: TokenStreamTokenizer>(
        id: CSSPropertyID,
        stream: &mut Stream<T>,
        mode: CSSParserMode,
        color: bool,
    ) -> Result<Option<Rc<Value>>, PropertyParseError> {
        if color && IsMathFunction(stream)
            || !color && is_function(stream) && !IsMathFunction(stream)
        {
            return Ok(None);
        }
        stream.EnsureLookAhead();
        let save = stream.Save();
        let result = if color {
            ConsumeColor(id, stream)
        } else {
            ConsumeLiteral(
                id,
                stream,
                mode,
                Grammar::Length {
                    percent: true,
                    nonnegative: false,
                    quirks: false,
                    keywords: &[],
                },
            )
        };
        match result {
            Ok(v) => Ok(Some(v)),
            Err(e) => {
                stream.Restore(save);
                if e.kind == PropertyParseErrorKind::Unsupported {
                    Err(e)
                } else {
                    Ok(None)
                }
            }
        }
    }
    let repeating = stream.Peek().FunctionId() == Some(CSSValueID::kRepeatingLinearGradient);
    let gradient;
    {
        let mut args = RestoringBlockGuard::new(stream);
        args.ConsumeWhitespace();
        if args.Peek().Id() == CSSValueID::kIn {
            return Err(unsupported(id, "ConsumeColorInterpolationSpace"));
        }
        let mut angle = None;
        let mut end_x = None;
        let mut end_y = None;
        if IsMathFunction(&mut args) {
            return Err(unsupported(id, "ConsumeAngle CSSMathFunctionValue"));
        }
        let token = args.Peek().clone();
        if token.GetType() == kDimensionToken
            && matches!(
                token.GetUnitType(),
                UnitType::kDegrees | UnitType::kRadians | UnitType::kGradians | UnitType::kTurns
            )
        {
            angle = Some(values::numeric(token.NumericValue(), token.GetUnitType()));
            args.ConsumeIncludingWhitespace();
        } else if token.GetType() == kNumberToken && token.NumericValue() == 0.0 {
            angle = Some(values::numeric(0.0, UnitType::kDegrees));
            args.ConsumeIncludingWhitespace();
        } else if token.Id() == CSSValueID::kTo {
            args.ConsumeIncludingWhitespace();
            loop {
                match args.Peek().Id() {
                    side @ (CSSValueID::kLeft | CSSValueID::kRight) if end_x.is_none() => {
                        end_x = Some(side)
                    }
                    side @ (CSSValueID::kTop | CSSValueID::kBottom) if end_y.is_none() => {
                        end_y = Some(side)
                    }
                    _ => break,
                }
                args.ConsumeIncludingWhitespace();
            }
            if end_x.is_none() && end_y.is_none() {
                return Err(invalid(id));
            }
        }
        if args.Peek().Id() == CSSValueID::kIn {
            return Err(unsupported(id, "ConsumeColorInterpolationSpace"));
        }
        if angle.is_some() || end_x.is_some() || end_y.is_some() {
            if args.Peek().GetType() != kCommaToken {
                return Err(invalid(id));
            }
            args.ConsumeIncludingWhitespace();
        }
        let mut stops = Vec::new();
        let mut previous_hint = true;
        loop {
            let color = optional(id, &mut args, mode, true)?;
            if color.is_none() && previous_hint {
                return Err(invalid(id));
            }
            previous_hint = color.is_none();
            let offset = optional(id, &mut args, mode, false)?;
            if color.is_none() && offset.is_none() {
                return Err(invalid(id));
            }
            stops.push(values::CSSGradientColorStop {
                color: color.clone(),
                offset: offset.clone(),
            });
            if color.is_some() && offset.is_some() {
                if let Some(offset) = optional(id, &mut args, mode, false)? {
                    stops.push(values::CSSGradientColorStop {
                        color,
                        offset: Some(offset),
                    });
                }
            }
            if args.Peek().GetType() != kCommaToken {
                break;
            }
            args.ConsumeIncludingWhitespace();
        }
        if previous_hint || !args.AtEnd() || !args.Release() {
            return Err(invalid(id));
        }
        gradient = values::CSSLinearGradientValue {
            angle,
            end_x,
            end_y,
            repeating,
            stops,
        };
    }
    stream.ConsumeWhitespace();
    Ok(Rc::new(Value::new(CSSValuePayload::kLinearGradientClass(
        gradient,
    ))))
}
// cpp: css_parsing_utils.cc:5987-6060 ConsumeShadow/ParseSingleShadow.
fn ConsumeShadow<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    fn color<T: TokenStreamTokenizer>(
        id: CSSPropertyID,
        stream: &mut Stream<T>,
    ) -> Result<Option<Rc<Value>>, PropertyParseError> {
        // ConsumeColor never consumes a math function. Reuse the translated
        // token-level math classifier so the length collaborator sees it.
        if IsMathFunction(stream) {
            return Ok(None);
        }
        stream.EnsureLookAhead();
        let save = stream.Save();
        match ConsumeColor(id, stream) {
            Ok(value) => Ok(Some(value)),
            Err(error) => {
                stream.Restore(save);
                if error.kind == PropertyParseErrorKind::Unsupported {
                    Err(error)
                } else {
                    Ok(None)
                }
            }
        }
    }
    fn length<T: TokenStreamTokenizer>(
        id: CSSPropertyID,
        stream: &mut Stream<T>,
        mode: CSSParserMode,
        nonnegative: bool,
    ) -> Result<Option<Rc<Value>>, PropertyParseError> {
        // Non-math functions belong to the color alternative, not ConsumeLength.
        if is_function(stream) && !IsMathFunction(stream) {
            return Ok(None);
        }
        stream.EnsureLookAhead();
        let save = stream.Save();
        match ConsumeLiteral(
            id,
            stream,
            mode,
            Grammar::Length {
                percent: false,
                nonnegative,
                quirks: false,
                keywords: &[],
            },
        ) {
            Ok(value) => Ok(Some(value)),
            Err(error) => {
                stream.Restore(save);
                if error.kind == PropertyParseErrorKind::Unsupported {
                    Err(error)
                } else {
                    Ok(None)
                }
            }
        }
    }
    if stream.Peek().Id() == CSSValueID::kNone {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    let allow = id == CSSPropertyID::kBoxShadow;
    let mut shadows = Vec::new();
    loop {
        if at_value_end(stream) {
            return Err(invalid(id));
        }
        let mut color_value = color(id, stream)?;
        let mut style = None;
        if stream.Peek().Id() == CSSValueID::kInset {
            if !allow {
                return Err(invalid(id));
            }
            style = Some(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
            if color_value.is_none() {
                color_value = color(id, stream)?;
            }
        }
        let x = length(id, stream, mode, false)?.ok_or_else(|| invalid(id))?;
        let y = length(id, stream, mode, false)?.ok_or_else(|| invalid(id))?;
        let blur = length(id, stream, mode, true)?;
        let spread = if blur.is_some() && allow {
            length(id, stream, mode, false)?
        } else {
            None
        };
        if !at_value_end(stream) {
            if color_value.is_none() {
                color_value = color(id, stream)?;
            }
            if stream.Peek().Id() == CSSValueID::kInset {
                if !allow || style.is_some() {
                    return Err(invalid(id));
                }
                style = Some(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
                if color_value.is_none() {
                    color_value = color(id, stream)?;
                }
            }
        }
        shadows.push(Rc::new(Value::new(CSSValuePayload::kShadowClass(
            values::CSSShadowValue {
                x,
                y,
                blur,
                spread,
                style,
                color: color_value,
            },
        ))));
        if stream.Peek().GetType() != kCommaToken {
            break;
        }
        stream.ConsumeIncludingWhitespace();
    }
    Ok(values::list(shadows, values::ListSeparator::Comma))
}
// cpp: css_parsing_utils.cc:9687-9722. Names retain case and order, and are
// CSSCustomIdentValue objects awaiting the document/shadow scope binding.
fn ConsumeContainerName<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kNone {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    let mut names = Vec::new();
    loop {
        let token = stream.Peek();
        if token.FunctionId() == Some(CSSValueID::kIdent) {
            return Err(unsupported(
                id,
                "ConsumeCustomIdent CSSFunctionValue ident()",
            ));
        }
        if token.GetType() != kIdentToken
            || token.Id() == CSSValueID::kNone
            || token.Id() == CSSValueID::kDefault
            || values::wide(token.Id()).is_some()
        {
            break;
        }
        let name = token.Value().ToString();
        let word = name.Utf8();
        if ["not", "and", "or"]
            .iter()
            .any(|reserved| word.eq_ignore_ascii_case(reserved))
        {
            break;
        }
        stream.ConsumeIncludingWhitespace();
        names.push(values::custom_ident(&name, CSSPropertyID::kInvalid));
    }
    if names.is_empty() {
        return Err(invalid(id));
    }
    Ok(values::list(names, values::ListSeparator::Space))
}
// cpp: css_parsing_utils.cc:9724-9775. Source accepts anchored directly;
// the resulting list has canonical size, scroll-state, anchored order.
fn ConsumeContainerType<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kNormal {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    let mut size = None;
    let mut scroll = false;
    let mut anchored = false;
    loop {
        let keyword = stream.Peek().Id();
        match keyword {
            CSSValueID::kSize | CSSValueID::kInlineSize if size.is_none() => size = Some(keyword),
            CSSValueID::kScrollState if !scroll => scroll = true,
            CSSValueID::kAnchored if !anchored => anchored = true,
            _ => break,
        }
        stream.ConsumeIncludingWhitespace();
    }
    let mut values = Vec::new();
    if let Some(size) = size {
        values.push(values::identifier(size));
    }
    if scroll {
        values.push(values::identifier(CSSValueID::kScrollState));
    }
    if anchored {
        values.push(values::identifier(CSSValueID::kAnchored));
    }
    if values.is_empty() {
        return Err(invalid(id));
    }
    Ok(values::list(values, values::ListSeparator::Space))
}
// cpp: css_parsing_utils.cc:6645-6726; longhands_custom.cc:4698-4752.
// FontVariationSettings longhand requires a numeric value; unlike the optional
// feature value it accepts neither on/off nor omission in this source path.
fn ConsumeFontSettings<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kNormal {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    let variation = id == CSSPropertyID::kFontVariationSettings;
    let mut settings = Vec::new();
    loop {
        let token = stream.Peek();
        if token.GetType() != kStringToken || token.Value().length() != 4 {
            return Err(invalid(id));
        }
        let tag = stream.ConsumeIncludingWhitespace().Value().ToString();
        if !(0..4).all(|index| (0x20..=0x7e).contains(&tag.CodeUnitAt(index))) {
            return Err(invalid(id));
        }
        let tag = foundation::AtomicString::from_utf16(tag.Span16().unwrap_or_default());
        if IsMathFunction(stream) {
            return Err(unsupported(id, "font settings CSSMathFunctionValue"));
        }
        let value = if stream.Peek().GetType() == kNumberToken {
            if !variation
                && stream.Peek().GetNumericValueType() != NumericValueType::kIntegerValueType
            {
                return Err(invalid(id));
            }
            values::numeric(
                stream.ConsumeIncludingWhitespace().NumericValue(),
                if variation {
                    UnitType::kNumber
                } else {
                    UnitType::kInteger
                },
            )
        } else if !variation && matches!(stream.Peek().Id(), CSSValueID::kOn | CSSValueID::kOff) {
            values::numeric(
                (stream.ConsumeIncludingWhitespace().Id() == CSSValueID::kOn) as u8 as f64,
                UnitType::kNumber,
            )
        } else if !variation {
            values::numeric(1.0, UnitType::kNumber)
        } else {
            return Err(invalid(id));
        };
        settings.push(Rc::new(Value::new(if variation {
            CSSValuePayload::kFontVariationClass(values::CSSFontVariationValue { tag, value })
        } else {
            CSSValuePayload::kFontFeatureClass(values::CSSFontFeatureValue { tag, value })
        })));
        if stream.Peek().GetType() != kCommaToken {
            break;
        }
        stream.ConsumeIncludingWhitespace();
    }
    Ok(values::list(settings, values::ListSeparator::Comma))
}

// cpp: css_parsing_utils.cc ConsumeFontFamily / ConsumeFamilyName.
// Each family is a real CSSFontFamilyValue or generic identifier. The calling
// ConsumeFontFamily slice places these elements in a comma-separated list.
fn ConsumeFamily<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    let token = stream.Peek().clone();
    let family = if token.GetType() == kStringToken {
        stream.ConsumeIncludingWhitespace();
        token.Value().ToString()
    } else if token.GetType() == kIdentToken {
        if matches(
            token.Id(),
            &[
                "serif",
                "sans-serif",
                "monospace",
                "cursive",
                "fantasy",
                "system-ui",
                "ui-serif",
                "ui-sans-serif",
                "ui-monospace",
                "ui-rounded",
                "math",
                "fangsong",
                "-webkit-body",
            ],
        ) {
            stream.ConsumeIncludingWhitespace();
            return Ok(values::identifier(token.Id()));
        }
        let mut name = token.Value().ToString();
        stream.ConsumeIncludingWhitespace();
        if stream.Peek().GetType() != kIdentToken
            && (values::wide(token.Id()).is_some() || name.Utf8().eq_ignore_ascii_case("default"))
        {
            return Err(invalid(id));
        }
        while stream.Peek().GetType() == kIdentToken {
            name.push_str(" ");
            name.push_str(
                &stream
                    .ConsumeIncludingWhitespace()
                    .Value()
                    .ToString()
                    .Utf8(),
            );
        }
        name
    } else {
        return Err(invalid(id));
    };
    Ok(Rc::new(Value::new(CSSValuePayload::kFontFamilyClass(
        values::CSSFontFamilyValue(family),
    ))))
}
// cpp: css_parsing_utils.cc:2368-2400,2487-2499 ParseQuirkyHexColor /
// ConsumeColorMaybeQuirky. Decode the tokenizer's typed number/unit/identifier.
fn ConsumeBorderColorSide<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    stream.EnsureLookAhead();
    let savepoint = stream.Save();
    let error = match ConsumeColor(id, stream) {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };
    stream.Restore(savepoint);
    if mode != CSSParserMode::kHTMLQuirksMode || error.kind != PropertyParseErrorKind::Invalid {
        return Err(error);
    }
    let token = stream.Peek().clone();
    let mut hex = match token.GetType() {
        kNumberToken | kDimensionToken => {
            if token.GetNumericValueType() != NumericValueType::kIntegerValueType
                || !(0.0..1_000_000.0).contains(&token.NumericValue())
            {
                return Err(invalid(id));
            }
            let mut text = format!("{}", token.NumericValue() as i32);
            if token.GetType() == kDimensionToken {
                text.push_str(&token.Value().ToString().Utf8());
            }
            while text.len() < 6 {
                text.insert(0, '0');
            }
            text
        }
        kIdentToken => token.Value().ToString().Utf8(),
        _ => return Err(invalid(id)),
    };
    if !matches!(hex.len(), 3 | 6) || !hex.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return Err(invalid(id));
    }
    if hex.len() == 3 {
        hex = hex.chars().flat_map(|c| [c, c]).collect();
    }
    let color = u32::from_str_radix(&hex, 16).map_err(|_| invalid(id))?;
    stream.ConsumeIncludingWhitespace();
    Ok(values::color(Color::FromRGBA(
        ((color >> 16) & 255) as i32,
        ((color >> 8) & 255) as i32,
        (color & 255) as i32,
        255,
    )))
}
// cpp: css_parsing_utils.cc ConsumeColor:2421-2485; ParseHexColor:2355-2366;
// platform/graphics/color.cc ParseHexColorInternal (3/4/6/8-digit forms).
pub(crate) fn ConsumeColor<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    let token = stream.Peek().clone();
    let keyword = token.Id();
    if keyword != CSSValueID::kInvalid
        && ((CSSValueID::kAqua as i32..=CSSValueID::kInternalCurrentSearchTextColor as i32)
            .contains(&(keyword as i32))
            || (CSSValueID::kAliceblue as i32..=CSSValueID::kYellowgreen as i32)
                .contains(&(keyword as i32))
            || keyword == CSSValueID::kMenu)
    {
        if matches!(
            keyword,
            CSSValueID::kAccentcolor | CSSValueID::kAccentcolortext
        ) {
            return Err(unsupported(id, "SystemAccentColorAllowed parser context"));
        }
        stream.ConsumeIncludingWhitespace();
        return Ok(values::identifier(keyword));
    }
    if token.GetType() == kHashToken {
        let hex = token.Value().ToString().Utf8();
        let bytes = hex.as_bytes();
        if !matches!(bytes.len(), 3 | 4 | 6 | 8) || !bytes.iter().all(u8::is_ascii_hexdigit) {
            return Err(invalid(id));
        }
        let digit = |byte: u8| {
            if byte <= b'9' {
                byte - b'0'
            } else {
                byte.to_ascii_lowercase() - b'a' + 10
            }
        };
        let mut rgba = [0u8, 0, 0, 255];
        if bytes.len() <= 4 {
            for (i, &byte) in bytes.iter().enumerate() {
                rgba[i] = digit(byte) * 17;
            }
        } else {
            for (i, pair) in bytes.chunks_exact(2).enumerate() {
                rgba[i] = digit(pair[0]) * 16 + digit(pair[1]);
            }
        }
        stream.ConsumeIncludingWhitespace();
        return Ok(values::color(Color::FromRGBA(
            rgba[0] as i32,
            rgba[1] as i32,
            rgba[2] as i32,
            rgba[3] as i32,
        )));
    }
    if token.GetType() == kFunctionToken {
        if !matches!(
            token.FunctionId(),
            Some(CSSValueID::kRgb | CSSValueID::kRgba)
        ) {
            return Err(unsupported(
                id,
                "ColorFunctionParser::ConsumeFunctionalSyntaxColor",
            ));
        }
        // cpp: core/css/properties/color_function_parser.cc RGB component,
        // legacy-comma and modern-space branches. Relative and missing-component
        // syntax require unresolved-color payloads and are explicit Unsupported.
        let parsed = {
            let mut args = BlockGuard::new(stream);
            args.ConsumeWhitespace();
            if args.Peek().GetType() == kIdentToken {
                return Err(unsupported(
                    id,
                    "ColorFunctionParser relative / none RGB channels",
                ));
            }
            let mut channels = [0f32; 3];
            let mut types = [kNumberToken; 3];
            let mut legacy = false;
            for i in 0..3 {
                let t = args.Peek().clone();
                types[i] = t.GetType();
                if !matches!(t.GetType(), kNumberToken | kPercentageToken) {
                    return Err(invalid(id));
                }
                channels[i] = (t.NumericValue()
                    / if t.GetType() == kPercentageToken {
                        100.0
                    } else {
                        255.0
                    })
                .clamp(0.0, 1.0) as f32;
                args.Consume();
                let had_whitespace = args.Peek().GetType() == kWhitespaceToken;
                args.ConsumeWhitespace();
                if i == 0 {
                    legacy = args.Peek().GetType() == kCommaToken;
                }
                if i < 2 && !legacy && !had_whitespace {
                    return Err(invalid(id));
                }
                if i < 2 && legacy {
                    if args.Peek().GetType() != kCommaToken {
                        return Err(invalid(id));
                    }
                    args.ConsumeIncludingWhitespace();
                }
                if i < 2 && !legacy && args.Peek().GetType() == kCommaToken {
                    return Err(invalid(id));
                }
            }
            if legacy && (types[0] != types[1] || types[0] != types[2]) {
                return Err(invalid(id));
            }
            let mut alpha = 1.0;
            if (legacy && args.Peek().GetType() == kCommaToken)
                || (!legacy
                    && args.Peek().GetType() == kDelimiterToken
                    && args.Peek().Delimiter() == b'/' as u16)
            {
                args.ConsumeIncludingWhitespace();
                let t = args.Peek().clone();
                if !matches!(t.GetType(), kNumberToken | kPercentageToken) {
                    return Err(invalid(id));
                }
                alpha = (t.NumericValue()
                    / if t.GetType() == kPercentageToken {
                        100.0
                    } else {
                        1.0
                    })
                .clamp(0.0, 1.0) as f32;
                args.ConsumeIncludingWhitespace();
            }
            if !args.AtEnd() {
                return Err(invalid(id));
            }
            Color::FromRGBAFloat(channels[0], channels[1], channels[2], alpha)
        };
        stream.ConsumeWhitespace();
        return Ok(values::color(parsed));
    }
    Err(invalid(id))
}
// cpp: css_parsing_utils.cc:4288-4352 ConsumeShorthandGreedilyViaLonghands.
// The translated longhand alternatives consume nothing when no value matches.
fn ConsumeShorthandGreedilyViaLonghands<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    let longhands = ShorthandFor(id);
    let mut parsed = vec![None; longhands.len()];
    let mut found_any = false;
    loop {
        let mut found = false;
        for (index, &longhand) in longhands.iter().enumerate() {
            if parsed[index].is_some() {
                continue;
            }
            stream.EnsureLookAhead();
            let savepoint = stream.Save();
            if let Ok(value) = ConsumeLonghand(longhand, stream, mode) {
                parsed[index] = Some(value);
                found = true;
                found_any = true;
                break;
            }
            stream.Restore(savepoint);
        }
        if !found || stream.AtEnd() {
            break;
        }
    }
    if !found_any {
        return Err(invalid(id));
    }
    for (&longhand, value) in longhands.iter().zip(parsed) {
        out.push(make_expanded(
            longhand,
            id,
            value.unwrap_or_else(|| {
                if id == CSSPropertyID::kFlexFlow {
                    values::identifier(if longhand == CSSPropertyID::kFlexDirection {
                        CSSValueID::kRow
                    } else {
                        CSSValueID::kNowrap
                    })
                } else {
                    values::wide(CSSValueID::kInitial).unwrap()
                }
            }),
            false,
        ));
    }
    Ok(())
}
fn at_value_end<T: TokenStreamTokenizer>(stream: &mut Stream<T>) -> bool {
    stream.AtEnd()
        || (stream.Peek().GetType() == kDelimiterToken && stream.Peek().Delimiter() == b'!' as u16)
}
fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    unresolved: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    if grid_lanes_parser::IsShorthand(id) {
        return grid_lanes_parser::Expand(id, stream, mode, out);
    }
    if corner_parser::IsShorthand(id) {
        return corner_parser::ParseShorthand(id, stream, mode, out);
    }
    if timeline_trigger_parser::IsShorthand(id) {
        return timeline_trigger_parser::Expand(id, stream, mode, out);
    }
    if id == CSSPropertyID::kMarker {
        return svg_presentation_parser::ParseMarker(stream, mode, out);
    }
    if logical_border_parser::IsShorthand(id) {
        return logical_border_parser::ParseShorthand(id, stream, mode, out);
    }
    // shorthands_custom.cc:6458-6466, shared greedy longhand consumer.
    if matches!(
        id,
        CSSPropertyID::kWebkitTextStroke | CSSPropertyID::kTextEmphasis
    ) {
        return ConsumeShorthandGreedilyViaLonghands(id, stream, mode, out);
    }
    if matches!(
        id,
        CSSPropertyID::kBorderImage | CSSPropertyID::kWebkitMaskBoxImage
    ) {
        return border_image_parser::ParseShorthand(id, stream, mode, out);
    }
    if matches!(
        id,
        CSSPropertyID::kScrollTimeline | CSSPropertyID::kViewTimeline
    ) {
        return timeline_parser::ParseShorthand(id, stream, mode, out);
    }
    if id == CSSPropertyID::kOffset {
        return motion_parser::ParseShorthand(stream, mode, out);
    }
    if id == CSSPropertyID::kMask {
        return mask_parser::ParseShorthand(id, unresolved, stream, mode, out);
    }
    if matches!(id, CSSPropertyID::kTextBox | CSSPropertyID::kTextSpacing) {
        return text_box_parser::ParseShorthand(id, stream, mode, out);
    }
    if id == CSSPropertyID::kFontSynthesis {
        return stable_misc_parser::FontSynthesis(stream, out);
    }
    if column_rule_parser::IsShorthand(id) {
        return column_rule_parser::ParseShorthand(id, stream, mode, out);
    }
    if rule_inset_parser::IsShorthand(id) {
        return rule_inset_parser::Expand(id, stream, mode, out);
    }
    if line_parser::IsLineShorthand(id) {
        return line_parser::ParseShorthand(id, stream, mode, out);
    }
    if id == CSSPropertyID::kPositionTry {
        return anchor_parser::ParseShorthand(id, stream, mode, out);
    }
    if matches!(id, CSSPropertyID::kGrid | CSSPropertyID::kGridTemplate) {
        return grid_parser::ParseShorthand(id, stream, mode, out);
    }
    if matches!(id, CSSPropertyID::kFont | CSSPropertyID::kFontVariant) {
        return font_parser::ParseShorthand(id, stream, mode, out);
    }
    if scroll_parser::IsScrollShorthand(id) {
        return scroll_parser::ParseShorthand(id, stream, mode, out);
    }
    if id == CSSPropertyID::kListStyle {
        return list_counter_parser::ParseShorthand(id, stream, mode, out);
    }
    if matches!(
        id,
        CSSPropertyID::kColumns | CSSPropertyID::kContainIntrinsicSize
    ) {
        return layout_misc_parser::ParseShorthand(id, stream, mode, out);
    }
    // cpp: css_parsing_utils.cc:5243-5276; shorthands_custom.cc:251-278,
    // 534-543,6034-6046. Single axes stay unwrapped; multiple layers form lists.
    if matches!(
        id,
        CSSPropertyID::kBackgroundPosition | CSSPropertyID::kMaskPosition
    ) {
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        loop {
            let (x, y) = ConsumePosition(
                id,
                stream,
                mode,
                true,
                id == CSSPropertyID::kBackgroundPosition || unresolved != id,
            )?;
            xs.push(x);
            ys.push(y);
            if stream.Peek().GetType() != kCommaToken {
                break;
            }
            stream.ConsumeIncludingWhitespace();
        }
        let longhands = ShorthandFor(id);
        let single_or_list = |mut items: Vec<Rc<Value>>| {
            if items.len() == 1 {
                items.pop().unwrap()
            } else {
                values::list(items, values::ListSeparator::Comma)
            }
        };
        out.push(make_expanded(longhands[0], id, single_or_list(xs), false));
        out.push(make_expanded(longhands[1], id, single_or_list(ys), false));
        return Ok(());
    }
    if matches!(id, CSSPropertyID::kAnimation | CSSPropertyID::kTransition) {
        return ParseAnimationShorthand(id, stream, mode, out);
    }
    // cpp: shorthands_custom.cc:307-348,444-496; css_parsing_utils.cc:1048-1061.
    if id == CSSPropertyID::kAnimationRange {
        let mut starts = Vec::new();
        let mut ends = Vec::new();
        loop {
            let start = ConsumeAnimationRange(id, stream, mode, 0.0)?.ok_or_else(|| invalid(id))?;
            let end = ConsumeAnimationRange(id, stream, mode, 100.0)?
                .or_else(|| timeline_trigger_parser::Implied(&Some(start.clone())))
                .unwrap_or_else(|| values::identifier(CSSValueID::kNormal));
            starts.push(start);
            ends.push(end);
            if stream.Peek().GetType() != kCommaToken {
                break;
            }
            stream.ConsumeIncludingWhitespace();
        }
        out.push(make_expanded(
            CSSPropertyID::kAnimationRangeStart,
            id,
            values::list(starts, values::ListSeparator::Comma),
            false,
        ));
        out.push(make_expanded(
            CSSPropertyID::kAnimationRangeEnd,
            id,
            values::list(ends, values::ListSeparator::Comma),
            false,
        ));
        return Ok(());
    }
    // cpp: shorthands_custom.cc:4701-4752,4765-4811,4824-4870.
    if matches!(
        id,
        CSSPropertyID::kPlaceContent | CSSPropertyID::kPlaceItems | CSSPropertyID::kPlaceSelf
    ) {
        let longhands = ShorthandFor(id);
        stream.EnsureLookAhead();
        let savepoint = stream.Save();
        let is_baseline = matches(stream.Peek().Id(), &["first", "last", "baseline"]);
        let first = ConsumeAlignment(longhands[0], stream)?;
        let second = match ConsumeAlignment(longhands[1], stream) {
            Ok(value) => value,
            Err(_) if id == CSSPropertyID::kPlaceContent && is_baseline => ContentDistribution(
                CSSValueID::kInvalid,
                CSSValueID::kStart,
                CSSValueID::kInvalid,
            ),
            Err(_) => {
                let end = stream.Offset();
                stream.EnsureLookAhead();
                stream.Restore(savepoint);
                let value = ConsumeAlignment(longhands[1], stream)?;
                if id != CSSPropertyID::kPlaceContent && stream.Offset() != end {
                    return Err(invalid(id));
                }
                value
            }
        };
        out.push(make_expanded(longhands[0], id, first, false));
        out.push(make_expanded(longhands[1], id, second, false));
        return Ok(());
    }
    // cpp: shorthands_custom.cc:3613-3680 GridArea; css_parsing_utils.cc:7578-7608
    // ConsumeGridItemPositionShorthand (GridColumn:3695, GridRow:3992).
    if matches!(
        id,
        CSSPropertyID::kGridArea | CSSPropertyID::kGridColumn | CSSPropertyID::kGridRow
    ) {
        let longhands = ShorthandFor(id);
        let mut items = vec![ConsumeGridLine(id, stream, mode)?];
        while items.len() < longhands.len()
            && stream.Peek().GetType() == kDelimiterToken
            && stream.Peek().Delimiter() == b'/' as u16
        {
            stream.ConsumeIncludingWhitespace();
            items.push(ConsumeGridLine(id, stream, mode)?);
        }
        while items.len() < longhands.len() {
            items.push(values::identifier(CSSValueID::kAuto));
        }
        for (&longhand, value) in longhands.iter().zip(items) {
            out.push(make_expanded(longhand, id, value, false));
        }
        return Ok(());
    }
    // cpp: css_parsing_utils.cc ConsumeQuad / ConsumeShorthandGreedily;
    // shorthands_custom.cc Margin/Padding/BorderWidth/BorderStyle/BorderColor.
    if matches!(
        id,
        CSSPropertyID::kMargin
            | CSSPropertyID::kPadding
            | CSSPropertyID::kBorderWidth
            | CSSPropertyID::kBorderStyle
            | CSSPropertyID::kBorderColor
            | CSSPropertyID::kInset
            | CSSPropertyID::kScrollMargin
            | CSSPropertyID::kScrollPadding
    ) {
        let longhands = ShorthandFor(id);
        let mut parsed = Vec::new();
        while parsed.len() < 4 && !at_value_end(stream) {
            parsed.push(ConsumeLonghand(longhands[parsed.len()], stream, mode)?);
        }
        if parsed.is_empty() {
            return Err(invalid(id));
        }
        let indexes = match parsed.len() {
            1 => [0, 0, 0, 0],
            2 => [0, 1, 0, 1],
            3 => [0, 1, 2, 1],
            _ => [0, 1, 2, 3],
        };
        for (i, &longhand) in longhands.iter().enumerate() {
            out.push(make_expanded(
                longhand,
                id,
                parsed[indexes[i]].clone(),
                i >= parsed.len(),
            ));
        }
        return Ok(());
    }
    // cpp: shorthands_custom.cc BorderRadius::ParseShorthand:970-1011;
    // css_parsing_utils.cc ConsumeRadii:9117.
    if id == CSSPropertyID::kBorderRadius {
        let grammar = Grammar::Length {
            percent: true,
            nonnegative: true,
            quirks: false,
            keywords: &[],
        };
        let mut horizontal = Vec::new();
        while horizontal.len() < 4
            && !at_value_end(stream)
            && !(stream.Peek().GetType() == kDelimiterToken
                && stream.Peek().Delimiter() == b'/' as u16)
        {
            horizontal.push(ConsumeLiteral(id, stream, mode, grammar)?);
        }
        if horizontal.is_empty() {
            return Err(invalid(id));
        }
        let mut vertical = Vec::new();
        if stream.Peek().GetType() == kDelimiterToken && stream.Peek().Delimiter() == b'/' as u16 {
            stream.ConsumeIncludingWhitespace();
            while vertical.len() < 4 && !at_value_end(stream) {
                vertical.push(ConsumeLiteral(id, stream, mode, grammar)?);
            }
            if vertical.is_empty() {
                return Err(invalid(id));
            }
        } else if unresolved == CSSPropertyID::kAliasWebkitBorderRadius && horizontal.len() == 2 {
            vertical.push(horizontal.pop().unwrap());
        } else {
            vertical = horizontal.clone();
        }
        let expand = |len| match len {
            1 => [0, 0, 0, 0],
            2 => [0, 1, 0, 1],
            3 => [0, 1, 2, 1],
            _ => [0, 1, 2, 3],
        };
        let h = expand(horizontal.len());
        let v = expand(vertical.len());
        for (index, &longhand) in ShorthandFor(id).iter().enumerate() {
            let pair = values::CSSValuePair {
                first: horizontal[h[index]].clone(),
                second: vertical[v[index]].clone(),
                drop_identical: true,
            };
            out.push(make_expanded(
                longhand,
                id,
                Rc::new(Value::new(CSSValuePayload::kValuePairClass(pair))),
                false,
            ));
        }
        return Ok(());
    }
    // cpp: shorthands_custom.cc:1043-1072 BorderSpacing::ParseShorthand.
    if id == CSSPropertyID::kBorderSpacing {
        let grammar = Grammar::Length {
            percent: false,
            nonnegative: true,
            quirks: true,
            keywords: &[],
        };
        let horizontal = ConsumeLiteral(id, stream, mode, grammar)?;
        let vertical = if at_value_end(stream) {
            horizontal.clone()
        } else {
            ConsumeLiteral(id, stream, mode, grammar)?
        };
        out.push(make_expanded(
            CSSPropertyID::kWebkitBorderHorizontalSpacing,
            id,
            horizontal,
            false,
        ));
        out.push(make_expanded(
            CSSPropertyID::kWebkitBorderVerticalSpacing,
            id,
            vertical,
            false,
        ));
        return Ok(());
    }
    // cpp: shorthands_custom.cc Gap/Overflow and logical edge shorthands;
    // BorderBlockColor/Style/Width:554,636,656; Inline:828,910,930.
    if matches!(
        id,
        CSSPropertyID::kGap
            | CSSPropertyID::kInterestDelay
            | CSSPropertyID::kOverflow
            | CSSPropertyID::kMarginBlock
            | CSSPropertyID::kMarginInline
            | CSSPropertyID::kPaddingBlock
            | CSSPropertyID::kPaddingInline
            | CSSPropertyID::kInsetBlock
            | CSSPropertyID::kInsetInline
            | CSSPropertyID::kBorderBlockColor
            | CSSPropertyID::kBorderInlineColor
            | CSSPropertyID::kBorderBlockStyle
            | CSSPropertyID::kBorderInlineStyle
            | CSSPropertyID::kBorderBlockWidth
            | CSSPropertyID::kBorderInlineWidth
    ) {
        let longhands = ShorthandFor(id);
        let first = ConsumeLonghand(longhands[0], stream, mode)?;
        let implicit = at_value_end(stream);
        let second = if implicit {
            first.clone()
        } else {
            ConsumeLonghand(longhands[1], stream, mode)?
        };
        out.push(make_expanded(longhands[0], id, first, false));
        let implicit = implicit
            && !matches!(
                id,
                CSSPropertyID::kInterestDelay
                    | CSSPropertyID::kBorderBlockColor
                    | CSSPropertyID::kBorderInlineColor
                    | CSSPropertyID::kBorderBlockStyle
                    | CSSPropertyID::kBorderInlineStyle
                    | CSSPropertyID::kBorderBlockWidth
                    | CSSPropertyID::kBorderInlineWidth
            );
        out.push(make_expanded(longhands[1], id, second, implicit));
        return Ok(());
    }
    // cpp: shorthands_custom.cc WhiteSpace::ParseShorthand:6478-6536.
    if id == CSSPropertyID::kWhiteSpace {
        stream.EnsureLookAhead();
        let savepoint = stream.Save();
        let keyword = stream.Peek().Id();
        let pair = match keyword {
            CSSValueID::kNormal => Some((CSSValueID::kCollapse, CSSValueID::kWrap)),
            CSSValueID::kNowrap => Some((CSSValueID::kCollapse, CSSValueID::kNowrap)),
            CSSValueID::kPre => Some((CSSValueID::kPreserve, CSSValueID::kNowrap)),
            CSSValueID::kPreWrap => Some((CSSValueID::kPreserve, CSSValueID::kWrap)),
            CSSValueID::kPreLine => Some((CSSValueID::kPreserveBreaks, CSSValueID::kWrap)),
            CSSValueID::kBreakSpaces => Some((CSSValueID::kBreakSpaces, CSSValueID::kWrap)),
            _ => None,
        };
        if let Some((collapse, wrap)) = pair {
            stream.ConsumeIncludingWhitespace();
            if at_value_end(stream) {
                out.push(make_expanded(
                    CSSPropertyID::kWhiteSpaceCollapse,
                    id,
                    values::identifier(collapse),
                    false,
                ));
                out.push(make_expanded(
                    CSSPropertyID::kTextWrapMode,
                    id,
                    values::identifier(wrap),
                    false,
                ));
                return Ok(());
            }
            stream.Restore(savepoint);
        }
        return ConsumeShorthandGreedilyViaLonghands(id, stream, mode, out);
    }
    // cpp: shorthands_custom.cc:5299-5308 TextWrap; generated keyword longhands.
    if id == CSSPropertyID::kTextWrap {
        return ConsumeShorthandGreedilyViaLonghands(id, stream, mode, out);
    }
    // cpp: shorthands_custom.cc:4485-4493.
    // cpp: shorthands_custom.cc:5232-5243.
    if id == CSSPropertyID::kTextDecoration {
        return ConsumeShorthandGreedilyViaLonghands(id, stream, mode, out);
    }
    if id == CSSPropertyID::kOutline {
        return ConsumeShorthandGreedilyViaLonghands(id, stream, mode, out);
    }
    // cpp: shorthands_custom.cc:2070-2101.
    if id == CSSPropertyID::kContainer {
        let name = ConsumeContainerName(CSSPropertyID::kContainerName, stream)?;
        let type_ = if stream.Peek().GetType() == kDelimiterToken
            && stream.Peek().Delimiter() == b'/' as u16
        {
            stream.ConsumeIncludingWhitespace();
            ConsumeContainerType(CSSPropertyID::kContainerType, stream)?
        } else {
            values::identifier(CSSValueID::kNormal)
        };
        out.push(make_expanded(
            CSSPropertyID::kContainerName,
            id,
            name,
            false,
        ));
        out.push(make_expanded(
            CSSPropertyID::kContainerType,
            id,
            type_,
            false,
        ));
        return Ok(());
    }
    // cpp: shorthands_custom.cc:3052-3063, greedy parse with concrete defaults.
    if id == CSSPropertyID::kFlexFlow {
        return ConsumeShorthandGreedilyViaLonghands(id, stream, mode, out);
    }
    // cpp: shorthands_custom.cc Flex::ParseShorthand:2942-3041.
    if id == CSSPropertyID::kFlex {
        if stream.Peek().Id() == CSSValueID::kNone {
            stream.ConsumeIncludingWhitespace();
            for (longhand, value) in [
                (
                    CSSPropertyID::kFlexGrow,
                    values::numeric(0.0, UnitType::kNumber),
                ),
                (
                    CSSPropertyID::kFlexShrink,
                    values::numeric(0.0, UnitType::kNumber),
                ),
                (
                    CSSPropertyID::kFlexBasis,
                    values::identifier(CSSValueID::kAuto),
                ),
            ] {
                out.push(make_expanded(longhand, id, value, false));
            }
            return Ok(());
        }
        let mut grow = None;
        let mut shrink = None;
        let mut basis = None;
        while !at_value_end(stream) {
            if stream.Peek().GetType() == kNumberToken && (grow.is_none() || shrink.is_none()) {
                let value =
                    ConsumeLiteral(id, stream, mode, Grammar::Number { nonnegative: true })?;
                if grow.is_none() {
                    grow = Some(value);
                } else {
                    shrink = Some(value);
                }
            } else if basis.is_none() {
                // A third numeric zero is accepted as the basis only after both
                // flex factors, matching Blink's unitless-basis disambiguation.
                basis = Some(ConsumeLonghand(CSSPropertyID::kFlexBasis, stream, mode)?);
                if grow.is_some() && shrink.is_none() {
                    shrink = Some(values::numeric(1.0, UnitType::kNumber));
                }
            } else {
                return Err(invalid(id));
            }
        }
        if grow.is_none() && basis.is_none() {
            return Err(invalid(id));
        }
        for (longhand, value) in [
            (
                CSSPropertyID::kFlexGrow,
                grow.unwrap_or_else(|| values::numeric(1.0, UnitType::kNumber)),
            ),
            (
                CSSPropertyID::kFlexShrink,
                shrink.unwrap_or_else(|| values::numeric(1.0, UnitType::kNumber)),
            ),
            (
                CSSPropertyID::kFlexBasis,
                basis.unwrap_or_else(|| values::numeric(0.0, UnitType::kPercentage)),
            ),
        ] {
            out.push(make_expanded(longhand, id, value, false));
        }
        return Ok(());
    }
    // cpp: shorthands_custom.cc Border::ParseShorthand:716; side border parser
    // consumes width || style || color and resets omitted members to initial.
    if matches!(
        id,
        CSSPropertyID::kBorder
            | CSSPropertyID::kBorderTop
            | CSSPropertyID::kBorderRight
            | CSSPropertyID::kBorderBottom
            | CSSPropertyID::kBorderLeft
    ) {
        let mut width = None;
        let mut style = None;
        let mut color = None;
        while !at_value_end(stream) {
            let keyword = stream.Peek().Id();
            let ty = stream.Peek().GetType();
            if width.is_none()
                && (matches!(ty, kNumberToken | kDimensionToken)
                    || matches(keyword, &["thin", "medium", "thick"]))
            {
                width = Some(ConsumeLiteral(
                    id,
                    stream,
                    mode,
                    Grammar::Length {
                        percent: false,
                        nonnegative: true,
                        quirks: false,
                        keywords: &["thin", "medium", "thick"],
                    },
                )?);
            } else if style.is_none()
                && matches(
                    keyword,
                    &[
                        "none", "hidden", "inset", "groove", "outset", "ridge", "dotted", "dashed",
                        "solid", "double",
                    ],
                )
            {
                style = Some(values::identifier(keyword));
                stream.ConsumeIncludingWhitespace();
            } else if color.is_none() {
                color = Some(ConsumeColor(id, stream)?);
            } else {
                return Err(invalid(id));
            }
        }
        if width.is_none() && style.is_none() && color.is_none() {
            return Err(invalid(id));
        }
        for &longhand in ShorthandFor(id) {
            let selected = if matches!(
                longhand,
                CSSPropertyID::kBorderTopWidth
                    | CSSPropertyID::kBorderRightWidth
                    | CSSPropertyID::kBorderBottomWidth
                    | CSSPropertyID::kBorderLeftWidth
            ) {
                width.as_ref()
            } else if matches!(
                longhand,
                CSSPropertyID::kBorderTopStyle
                    | CSSPropertyID::kBorderRightStyle
                    | CSSPropertyID::kBorderBottomStyle
                    | CSSPropertyID::kBorderLeftStyle
            ) {
                style.as_ref()
            } else if matches!(
                longhand,
                CSSPropertyID::kBorderTopColor
                    | CSSPropertyID::kBorderRightColor
                    | CSSPropertyID::kBorderBottomColor
                    | CSSPropertyID::kBorderLeftColor
            ) {
                color.as_ref()
            } else {
                None
            };
            out.push(make_expanded(
                longhand,
                id,
                selected
                    .cloned()
                    .unwrap_or_else(|| values::wide(CSSValueID::kInitial).unwrap()),
                selected.is_none(),
            ));
        }
        return Ok(());
    }
    // cpp: shorthands_custom.cc Background::ParseShorthand / ParseBackgroundOrMask.
    // This complete color-only branch resets every other background longhand.
    if id == CSSPropertyID::kBackground {
        let color = ConsumeColor(id, stream)?;
        if !at_value_end(stream) {
            return Err(unsupported(
                id,
                "ParseBackgroundOrMask layered CSSValueList / image / position",
            ));
        }
        for &longhand in ShorthandFor(id) {
            out.push(make_expanded(
                longhand,
                id,
                if longhand == CSSPropertyID::kBackgroundColor {
                    color.clone()
                } else {
                    values::wide(CSSValueID::kInitial).unwrap()
                },
                longhand != CSSPropertyID::kBackgroundColor,
            ));
        }
        return Ok(());
    }
    Err(unsupported(id, "CSSProperty::ParseShorthand"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(id: CSSPropertyID, value: &str) -> Vec<PropertyValue> {
        ParseProperty(
            id,
            &String::from(value),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap_or_else(|error| panic!("{value}: {error:?}"))
    }
    fn numeric(property: &PropertyValue) -> (f64, UnitType) {
        match property.Value().Payload() {
            CSSValuePayload::kNumericLiteralClass(value) => (value.DoubleValue(), value.GetType()),
            _ => panic!("numeric literal required"),
        }
    }
    fn rejected(id: CSSPropertyID, value: &str) -> PropertyParseError {
        ParseProperty(
            id,
            &String::from(value),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .err()
        .unwrap_or_else(|| panic!("unexpectedly accepted {value}"))
    }
    #[test]
    fn background_linear_image_properties_preserve_typed_geometry_stops_and_urls() {
        let parsed = parse(
            CSSPropertyID::kBackgroundImage,
            "url('icon.svg'), linear-gradient(to bottom right, red 10% 20%, 30%, blue), none",
        );
        let CSSValuePayload::kValueListClass(list) = parsed[0].Value().Payload() else {
            panic!()
        };
        assert_eq!(list.values.len(), 3);
        assert!(list.values[0].IsImageValue());
        let CSSValuePayload::kLinearGradientClass(g) = list.values[1].Payload() else {
            panic!()
        };
        assert_eq!(
            (g.end_x, g.end_y),
            (Some(CSSValueID::kRight), Some(CSSValueID::kBottom))
        );
        assert_eq!(g.stops.len(), 4);
        assert!(g.stops[2].color.is_none());
        assert!(g.stops[3].offset.is_none());
        assert_eq!(
            list.values[1].CssText().Utf8(),
            "linear-gradient(to right bottom, red 10%, red 20%, 30%, blue)"
        );
        assert_eq!(
            parse(
                CSSPropertyID::kBackgroundImage,
                "linear-gradient(180deg, red, blue)"
            )[0]
            .Value()
            .CssText()
            .Utf8(),
            "linear-gradient(red, blue)"
        );
        let repeating = parse(
            CSSPropertyID::kBackgroundImage,
            "repeating-linear-gradient(.25turn, red 2px, blue 50%)",
        );
        let CSSValuePayload::kValueListClass(list) = repeating[0].Value().Payload() else {
            panic!()
        };
        let CSSValuePayload::kLinearGradientClass(g) = list.values[0].Payload() else {
            panic!()
        };
        assert!(g.repeating);
        assert_eq!(g.Degrees(), Some(90.0));
        for css in [
            "linear-gradient(to, red, blue)",
            "linear-gradient(10%, red, blue)",
            "linear-gradient(red, 20%)",
            "linear-gradient(20%, red)",
            "linear-gradient(red,20%,30%,blue)",
            "linear-gradient(red,blue,)",
            "url(icon.svg),",
            "linear-gradient(to right left, red,blue)",
        ] {
            assert_eq!(
                rejected(CSSPropertyID::kBackgroundImage, css).kind,
                PropertyParseErrorKind::Invalid,
                "{css}"
            );
        }
        let math = parse(
            CSSPropertyID::kBackgroundImage,
            "linear-gradient(red calc(10% + 1px),blue)",
        );
        let CSSValuePayload::kValueListClass(list) = math[0].Value().Payload() else {
            panic!()
        };
        let CSSValuePayload::kLinearGradientClass(g) = list.values[0].Payload() else {
            panic!()
        };
        assert!(matches!(
            g.stops[0].offset.as_ref().unwrap().Payload(),
            CSSValuePayload::kMathFunctionClass(_)
        ));
        for css in [
            "linear-gradient(in oklab, red,blue)",
            "radial-gradient(red,blue)",
        ] {
            assert_eq!(
                rejected(CSSPropertyID::kBackgroundImage, css).kind,
                PropertyParseErrorKind::Unsupported,
                "{css}"
            );
        }
    }
    #[test]
    fn shadow_properties_use_shared_typed_list_and_source_component_order() {
        let canonical = parse(CSSPropertyID::kBoxShadow, "red inset 1px 2px 3px -4px");
        assert_eq!(
            canonical[0].Value().CssText().Utf8(),
            "red 1px 2px 3px -4px inset"
        );
        for input in [
            "inset red 1px 2px 3px -4px",
            "1px 2px 3px -4px red inset",
            "1px 2px 3px -4px inset red",
        ] {
            assert!(canonical[0].Value() == parse(CSSPropertyID::kBoxShadow, input)[0].Value());
        }
        let parsed = parse(
            CSSPropertyID::kBoxShadow,
            "1px -2px, rgba(10,20,30,.5) 0 0 4px 2px inset",
        );
        let CSSValuePayload::kValueListClass(list) = parsed[0].Value().Payload() else {
            panic!()
        };
        assert_eq!(list.separator, values::ListSeparator::Comma);
        assert_eq!(list.values.len(), 2);
        let CSSValuePayload::kShadowClass(first) = list.values[0].Payload() else {
            panic!()
        };
        assert!(
            first.blur.is_none()
                && first.spread.is_none()
                && first.style.is_none()
                && first.color.is_none()
        );
        let CSSValuePayload::kShadowClass(second) = list.values[1].Payload() else {
            panic!()
        };
        assert!(
            second.blur.is_some()
                && second.spread.is_some()
                && second.style.is_some()
                && second.color.as_ref().unwrap().IsColorValue()
        );
        assert!(!parsed[0].Value().HasRandomFunctions());
        let text = parse(
            CSSPropertyID::kTextShadow,
            "currentcolor 1em 2px 3px, red -1px -2px",
        );
        assert_eq!(
            text[0].Value().CssText().Utf8(),
            "currentcolor 1em 2px 3px, red -1px -2px"
        );
        for (id, input) in [
            (CSSPropertyID::kBoxShadow, "1px"),
            (CSSPropertyID::kBoxShadow, "1px 2px -3px"),
            (CSSPropertyID::kBoxShadow, "1% 2px"),
            (CSSPropertyID::kBoxShadow, "1px red 2px"),
            (CSSPropertyID::kBoxShadow, "inset 1px 2px inset"),
            (CSSPropertyID::kBoxShadow, "red 1px 2px blue"),
            (CSSPropertyID::kBoxShadow, "1px 2px,"),
            (CSSPropertyID::kBoxShadow, "none, 1px 2px"),
            (CSSPropertyID::kTextShadow, "inset 1px 2px"),
            (CSSPropertyID::kTextShadow, "1px 2px 3px 4px"),
            (CSSPropertyID::kTextShadow, "1px 2px inset"),
            (CSSPropertyID::kBoxShadow, "1 2"),
        ] {
            assert_eq!(
                rejected(id, input).kind,
                PropertyParseErrorKind::Invalid,
                "{input}"
            );
        }
        let shadows = parse(CSSPropertyID::kBoxShadow, "calc(1px + 2px) 0");
        let CSSValuePayload::kValueListClass(list) = shadows[0].Value().Payload() else {
            panic!()
        };
        let CSSValuePayload::kShadowClass(shadow) = list.values[0].Payload() else {
            panic!()
        };
        assert!(matches!(
            shadow.x.Payload(),
            CSSValuePayload::kMathFunctionClass(_)
        ));
        assert_eq!(
            rejected(CSSPropertyID::kTextShadow, "0 0 hsl(0 100% 50%)").kind,
            PropertyParseErrorKind::Unsupported
        );
    }
    #[test]
    fn production_container_names_flags_and_shorthand_are_typed() {
        let names = parse(CSSPropertyID::kContainerName, "Card card");
        let CSSValuePayload::kValueListClass(list) = names[0].Value().Payload() else {
            panic!()
        };
        assert!(list
            .values
            .iter()
            .all(|v| matches!(v.Payload(), CSSValuePayload::kCustomIdentClass(_))));
        assert_eq!(names[0].Value().CssText().Utf8(), "Card card");
        assert_eq!(
            parse(CSSPropertyID::kContainerType, "anchored scroll-state size")[0]
                .Value()
                .CssText()
                .Utf8(),
            "size scroll-state anchored"
        );
        assert_eq!(
            parse(CSSPropertyID::kContainerType, "inline-size")[0]
                .Value()
                .CssText()
                .Utf8(),
            "inline-size"
        );
        let shorthand = parse(CSSPropertyID::kContainer, "Card / scroll-state inline-size");
        assert_eq!(shorthand[0].PropertyID(), CSSPropertyID::kContainerName);
        assert_eq!(shorthand[1].PropertyID(), CSSPropertyID::kContainerType);
        assert_eq!(
            shorthand[1].Value().CssText().Utf8(),
            "inline-size scroll-state"
        );
        assert_eq!(
            parse(CSSPropertyID::kContainer, "Card")[1]
                .Value()
                .CssText()
                .Utf8(),
            "normal"
        );
        for (id, input) in [
            (CSSPropertyID::kContainerName, "NOT"),
            (CSSPropertyID::kContainerName, "card none"),
            (CSSPropertyID::kContainerName, "default"),
            (CSSPropertyID::kContainerName, "'card'"),
            (CSSPropertyID::kContainerType, "size inline-size"),
            (CSSPropertyID::kContainerType, "scroll-state scroll-state"),
            (CSSPropertyID::kContainerType, "normal size"),
            (CSSPropertyID::kContainer, "Card /"),
            (CSSPropertyID::kContainerName, "card and"),
        ] {
            assert_eq!(
                rejected(id, input).kind,
                PropertyParseErrorKind::Invalid,
                "{input}"
            );
        }
        assert_eq!(
            rejected(CSSPropertyID::kContainerName, "ident(--card)").kind,
            PropertyParseErrorKind::Unsupported
        );
    }
    #[test]
    fn overflow_outline_integer_source_grammar_and_shorthand_are_typed() {
        let outline = parse(CSSPropertyID::kOutline, "rgba(10, 20, 30, .5) auto .25px");
        assert_eq!(outline.len(), 3);
        assert!(outline
            .iter()
            .any(|p| p.PropertyID() == CSSPropertyID::kOutlineColor && p.Value().IsColorValue()));
        assert!(outline
            .iter()
            .any(|p| p.PropertyID() == CSSPropertyID::kOutlineStyle
                && p.Value().CssText().Utf8() == "auto"));
        assert!(outline
            .iter()
            .any(|p| p.PropertyID() == CSSPropertyID::kOutlineWidth
                && numeric(p) == (0.25, UnitType::kPixels)));
        let auto = parse(CSSPropertyID::kOutline, "auto");
        assert_eq!(
            auto.iter().filter(|p| p.Value().IsInitialValue()).count(),
            2
        );
        assert_eq!(
            parse(CSSPropertyID::kOutlineColor, "-webkit-focus-ring-color")[0]
                .Value()
                .CssText()
                .Utf8(),
            "-webkit-focus-ring-color"
        );
        for (id, input) in [
            (CSSPropertyID::kOutlineStyle, "hidden"),
            (CSSPropertyID::kOutlineWidth, "-1px"),
            (CSSPropertyID::kOutlineWidth, "2"),
            (CSSPropertyID::kOutlineOffset, "10%"),
            (CSSPropertyID::kOutline, "solid dashed"),
            (CSSPropertyID::kZIndex, "1.2"),
        ] {
            assert_eq!(
                rejected(id, input).kind,
                PropertyParseErrorKind::Invalid,
                "{input}"
            );
        }
        assert_eq!(
            numeric(&parse(CSSPropertyID::kZIndex, "-999999999999")[0]),
            (-999999999999.0, UnitType::kInteger)
        );
        assert!(
            matches!(parse(CSSPropertyID::kZIndex,"calc(2 + 3)")[0].Value().Payload(),CSSValuePayload::kMathFunctionClass(value)if value.range==crate::css_math_function_value::ValueRange::Integer)
        );
        let overflow = parse(CSSPropertyID::kOverflow, "overlay clip");
        assert_eq!(overflow[0].Value().CssText().Utf8(), "overlay");
        assert_eq!(overflow[1].Value().CssText().Utf8(), "clip");
    }
    #[test]
    fn opentype_font_settings_use_typed_tags_and_source_value_grammar() {
        let features = parse(
            CSSPropertyID::kFontFeatureSettings,
            "'liga', 'kern' off, 'zzzz' -2, 'abcd' on",
        );
        assert_eq!(
            features[0].Value().CssText().Utf8(),
            "\"liga\", \"kern\" 0, \"zzzz\" -2, \"abcd\""
        );
        let CSSValuePayload::kValueListClass(list) = features[0].Value().Payload() else {
            panic!()
        };
        assert_eq!(list.separator, values::ListSeparator::Comma);
        assert!(list.values.iter().all(|value| value.IsFontFeatureValue()));
        let axes = parse(
            CSSPropertyID::kFontVariationSettings,
            "'wght' 550.5, 'slnt' -12",
        );
        assert_eq!(
            axes[0].Value().CssText().Utf8(),
            "\"wght\" 550.5, \"slnt\" -12"
        );
        let CSSValuePayload::kValueListClass(list) = axes[0].Value().Payload() else {
            panic!()
        };
        assert!(list.values.iter().all(|value| value.IsFontVariationValue()));
        for id in [
            CSSPropertyID::kFontFeatureSettings,
            CSSPropertyID::kFontVariationSettings,
        ] {
            assert!(parse(id, "normal")[0].Value().IsIdentifierValue());
            assert!(parse(id, "var(--settings)")[0]
                .Value()
                .IsUnparsedDeclaration());
            for text in [
                "'abc' 1",
                "'abcde' 1",
                "'abéz' 1",
                "'abcd' 1,",
                "normal, 'abcd' 1",
            ] {
                assert_eq!(
                    rejected(id, text).kind,
                    PropertyParseErrorKind::Invalid,
                    "{id:?}: {text}"
                );
            }
            assert_eq!(
                rejected(id, "'abcd' calc(2)").kind,
                PropertyParseErrorKind::Unsupported
            );
        }
        assert_eq!(
            rejected(CSSPropertyID::kFontFeatureSettings, "'liga' 1.5").kind,
            PropertyParseErrorKind::Invalid
        );
        for text in ["'wght'", "'wght' on", "'wght' off"] {
            assert_eq!(
                rejected(CSSPropertyID::kFontVariationSettings, text).kind,
                PropertyParseErrorKind::Invalid
            );
        }
        assert_eq!(
            rejected(CSSPropertyID::kFontOpticalSizing, "normal").kind,
            PropertyParseErrorKind::Invalid
        );
        for keyword in ["auto", "normal", "none"] {
            parse(CSSPropertyID::kFontKerning, keyword);
        }
        for keyword in ["auto", "none"] {
            parse(CSSPropertyID::kFontOpticalSizing, keyword);
        }
    }

    #[test]
    fn content_uses_native_list_payloads_and_shared_attr_substitution() {
        let parsed = parse(
            CSSPropertyID::kContent,
            "'prefix' /* comment */ \"\\41\" !important",
        );
        assert!(parsed[0].IsImportant());
        let CSSValuePayload::kValueListClass(outer) = parsed[0].Value().Payload() else {
            panic!()
        };
        assert_eq!(outer.separator, values::ListSeparator::Slash);
        assert_eq!(outer.values.len(), 1);
        let CSSValuePayload::kValueListClass(items) = outer.values[0].Payload() else {
            panic!()
        };
        assert_eq!(items.separator, values::ListSeparator::Space);
        assert_eq!(items.values.len(), 2);
        let CSSValuePayload::kStringClass(decoded) = items.values[1].Payload() else {
            panic!()
        };
        assert_eq!(decoded.0.Utf8(), "A");
        for keyword in ["normal", "none"] {
            assert!(parse(CSSPropertyID::kContent, keyword)[0]
                .Value()
                .IsIdentifierValue());
        }
        assert!(parse(CSSPropertyID::kContent, "attr(data-label)")[0]
            .Value()
            .IsUnparsedDeclaration());
        for available in [
            "counter(item)",
            "url(image.png)",
            "linear-gradient(red, blue)",
            "open-quote",
            "'text' / 'alt'",
        ] {
            assert!(parse(CSSPropertyID::kContent, available)[0]
                .Value()
                .IsValueList());
        }
        for unavailable in ["radial-gradient(red, blue)", "counter(item, symbols('x'))"] {
            assert_eq!(
                rejected(CSSPropertyID::kContent, unavailable).kind,
                PropertyParseErrorKind::Unsupported
            );
        }
        for invalid_text in ["", "unknown", "normal 'extra'", "'text' none"] {
            assert_eq!(
                ParseProperty(
                    CSSPropertyID::kContent,
                    &String::from(invalid_text),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .err()
                .unwrap()
                .kind,
                PropertyParseErrorKind::Invalid
            );
        }
    }

    #[test]
    fn content_quotes_counters_alternatives_and_url_images_are_typed() {
        let parsed = parse(
            CSSPropertyID::kContent,
            "open-quote counter(Section, UPPER-ROMAN) counters(chapter, '.', CustomStyle) no-close-quote / 'Chapter ' counter(chapter)",
        );
        let value = parsed[0].Value();
        assert_eq!(
            value.CssText().Utf8(),
            "open-quote counter(Section, upper-roman) counters(chapter, \".\", CustomStyle) no-close-quote / \"Chapter \" counter(chapter)"
        );
        let CSSValuePayload::kValueListClass(outer) = value.Payload() else {
            panic!()
        };
        assert_eq!(outer.values.len(), 2);
        let CSSValuePayload::kValueListClass(items) = outer.values[0].Payload() else {
            panic!()
        };
        assert!(items.values[1].IsCounterContentValue());
        assert!(!items.values[1].IsScopedValue());
        for text in ["url(icon.svg)", "url('icon.svg')"] {
            let parsed = parse(CSSPropertyID::kContent, text);
            assert!(parsed[0].Value().MayContainUrl());
            assert_eq!(parsed[0].Value().CssText().Utf8(), "url(\"icon.svg\")");
            assert!(!parsed[0].Value().HasFailedOrCanceledSubresources());
        }
        assert_eq!(
            parse(CSSPropertyID::kContent, "counter(item, NoNe)")[0]
                .Value()
                .CssText()
                .Utf8(),
            "counter(item, none)"
        );
        for text in [
            "counter()",
            "counter(initial)",
            "counter(item,)",
            "counter(item, default)",
            "counter(item, decimal, junk)",
            "counters(item)",
            "counters(item, 3)",
            "unknown()",
            "/ 'alt'",
            "'text' /",
            "'text' / open-quote",
            "'text' / url(icon.svg)",
            "'text' / 'alt' / 'more'",
        ] {
            assert_eq!(
                rejected(CSSPropertyID::kContent, text).kind,
                PropertyParseErrorKind::Invalid,
                "{text}"
            );
        }
    }

    #[test]
    fn attr_and_env_references_validate_declaration_syntax_before_resolution() {
        let late_header = String::from("attr(var(--missing,size) px)");
        assert!(
            ParseCustomProperty("--x", &late_header, false, CSSParserMode::kHTMLStandardMode)
                .is_err()
        );
        assert!(ParseCustomPropertyWithArgumentGrammar(
            "--x",
            &late_header,
            false,
            CSSParserMode::kHTMLStandardMode,
            true
        )
        .is_ok());
        for input in [
            "attr(,)",
            "attr(var(--x,size) px,1px;2px)",
            "attr(var(--x,size) px,!)",
        ] {
            assert!(ParseCustomPropertyWithArgumentGrammar(
                "--x",
                &String::from(input),
                false,
                CSSParserMode::kHTMLStandardMode,
                true
            )
            .is_err());
        }
        for value in [
            "attr(foo px,1px;2px)",
            "attr(foo px,1px ! 2px)",
            "env(foo,1px;2px)",
            "env(foo,!)",
            "attr(123 px)",
            "attr(foo px 2px)",
            "env(foo -1)",
            "env(foo 1.0)",
            "attr(foo type(<url>))",
            "attr(foo type(<length> |))",
            "attr(foo type(<transform-list>+))",
            "attr(foo type(< length>))",
            "attr(foo type(default))",
        ] {
            assert_eq!(
                rejected(CSSPropertyID::kWidth, value).kind,
                PropertyParseErrorKind::Invalid,
                "{value}"
            );
            assert!(
                ParseCustomProperty(
                    "--x",
                    &String::from(value),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .is_err(),
                "{value}"
            );
        }
        for value in [
            "attr(foo)",
            "attr(foo px)",
            "attr(foo raw-string,)",
            "attr(foo %,var(--x, 1px))",
            "attr(foo made-up-unit, 1px)",
            "attr(foo type(<length> | foo#),1px)",
            "attr(foo type(*))",
            "env(foo 0 1,2px)",
            "env(foo -0)",
            "env(foo, fn(1px;2px!))",
            "attr(foo px,fn(1px;2px!))",
        ] {
            let parsed = parse(CSSPropertyID::kWidth, value);
            assert!(parsed[0].Value().IsUnparsedDeclaration(), "{value}");
        }
        let parsed = ParseDeclarationList(
            &String::from(
                "width:attr(foo px,1px;2px); height:env(foo,!); margin-left:attr(foo px,2px)",
            ),
            CSSParserMode::kHTMLStandardMode,
        );
        assert_eq!(parsed.errors.len(), 2);
        assert_eq!(parsed.properties.len(), 1);
        assert_eq!(
            parsed.properties[0].PropertyID(),
            CSSPropertyID::kMarginLeft
        );
    }
    #[test]
    fn repeat_style_consumers_keep_typed_axes_and_comma_layers() {
        for id in [CSSPropertyID::kBackgroundRepeat, CSSPropertyID::kMaskRepeat] {
            for (input, output, x, y) in [
                ("repeat", "repeat", CSSValueID::kRepeat, CSSValueID::kRepeat),
                (
                    "repeat-x",
                    "repeat-x",
                    CSSValueID::kRepeat,
                    CSSValueID::kNoRepeat,
                ),
                (
                    "repeat-y",
                    "repeat-y",
                    CSSValueID::kNoRepeat,
                    CSSValueID::kRepeat,
                ),
                (
                    "repeat no-repeat",
                    "repeat-x",
                    CSSValueID::kRepeat,
                    CSSValueID::kNoRepeat,
                ),
                (
                    "no-repeat repeat",
                    "repeat-y",
                    CSSValueID::kNoRepeat,
                    CSSValueID::kRepeat,
                ),
                (
                    "round space",
                    "round space",
                    CSSValueID::kRound,
                    CSSValueID::kSpace,
                ),
            ] {
                let parsed = parse(id, input);
                assert_eq!(parsed[0].Value().CssText().Utf8(), output);
                let CSSValuePayload::kValueListClass(list) = parsed[0].Value().Payload() else {
                    panic!("repeat list")
                };
                assert_eq!(list.values.len(), 1);
                let CSSValuePayload::kRepeatStyleClass(repeat) = list.values[0].Payload() else {
                    panic!("repeat style")
                };
                assert_eq!(PositionKeyword(&repeat.x), Some(x));
                assert_eq!(PositionKeyword(&repeat.y), Some(y));
                assert_eq!(
                    repeat.IsRepeat(),
                    x == CSSValueID::kRepeat && y == CSSValueID::kRepeat
                );
                assert!(!list.values[0].HasRandomFunctions());
                assert!(*parsed[0].Value() == *parse(id, output)[0].Value());
            }
            let layers = parse(id, "repeat-x, round space, no-repeat");
            let CSSValuePayload::kValueListClass(list) = layers[0].Value().Payload() else {
                panic!("repeat layers")
            };
            assert_eq!(list.values.len(), 3);
            for input in [
                "",
                "repeat-x repeat",
                "repeat repeat-x",
                "repeat repeat repeat",
                "repeat,",
                "repeat,,space",
                "auto",
            ] {
                assert_eq!(
                    rejected(id, input).kind,
                    PropertyParseErrorKind::Invalid,
                    "{input}"
                );
            }
        }
    }
    #[test]
    fn position_consumers_order_axes_and_keep_nested_edge_offsets() {
        for id in [
            CSSPropertyID::kObjectPosition,
            CSSPropertyID::kPerspectiveOrigin,
            CSSPropertyID::kOffsetAnchor,
            CSSPropertyID::kOffsetPosition,
        ] {
            for (input, output) in [
                ("center", "center center"),
                ("top", "center top"),
                ("left", "left center"),
                ("top left", "left top"),
                ("center left", "left center"),
                ("10% bottom", "10% bottom"),
                ("left 10%", "left 10%"),
                ("right -2px bottom 30%", "right -2px bottom 30%"),
                ("top 20px left 10px", "left 10px top 20px"),
            ] {
                let value = parse(id, input);
                assert_eq!(value[0].Value().CssText().Utf8(), output, "{id:?}: {input}");
                let CSSValuePayload::kValuePairClass(pair) = value[0].Value().Payload() else {
                    panic!("position pair")
                };
                assert!(!pair.drop_identical);
            }
            for input in [
                "left right",
                "top bottom",
                "top 10px",
                "left 10px top",
                "center left 10px",
                "10px left",
                "left top 10px",
            ] {
                assert_eq!(
                    rejected(id, input).kind,
                    PropertyParseErrorKind::Invalid,
                    "{id:?}: {input}"
                );
            }
            if id == CSSPropertyID::kPerspectiveOrigin {
                assert!(ParseProperty(
                    id,
                    &String::from("left calc(10% + 1px)"),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .is_ok());
            } else {
                assert_eq!(
                    rejected(id, "left calc(10% + 1px)").kind,
                    PropertyParseErrorKind::Unsupported
                );
            }
        }
        assert_eq!(
            parse(CSSPropertyID::kOffsetAnchor, "auto")[0]
                .Value()
                .CssText()
                .Utf8(),
            "auto"
        );
        assert_eq!(
            parse(CSSPropertyID::kOffsetPosition, "normal")[0]
                .Value()
                .CssText()
                .Utf8(),
            "normal"
        );
        assert_eq!(
            rejected(CSSPropertyID::kOffsetAnchor, "normal").kind,
            PropertyParseErrorKind::Invalid
        );
        assert_eq!(
            rejected(CSSPropertyID::kObjectPosition, "auto").kind,
            PropertyParseErrorKind::Invalid
        );
    }
    #[test]
    fn background_and_mask_position_shorthands_follow_alias_and_layer_rules() {
        for (input, x, y) in [
            ("top", "center", "top"),
            ("left 10px top", "left 10px", "top"),
            ("center left 10px", "left 10px", "center"),
            ("top 10px left", "left", "top 10px"),
            ("top 10px left 20px", "left 20px", "top 10px"),
        ] {
            for id in [
                CSSPropertyID::kBackgroundPosition,
                CSSPropertyID::kAliasWebkitMaskPosition,
            ] {
                let parsed = parse(id, input);
                assert_eq!(parsed.len(), 2);
                assert_eq!(parsed[0].Value().CssText().Utf8(), x);
                assert_eq!(parsed[1].Value().CssText().Utf8(), y);
                assert!(!parsed[0].Value().IsValueList());
                assert!(!parsed[0].IsImplicit());
            }
        }
        for input in ["left 10px top", "center left 10px", "top 10px left"] {
            assert_eq!(
                rejected(CSSPropertyID::kMaskPosition, input).kind,
                PropertyParseErrorKind::Invalid
            );
        }
        let mask = parse(CSSPropertyID::kMaskPosition, "right 2px bottom 3px");
        assert_eq!(mask[0].PropertyID(), CSSPropertyID::kWebkitMaskPositionX);
        assert_eq!(mask[1].PropertyID(), CSSPropertyID::kWebkitMaskPositionY);
        let layered = parse(
            CSSPropertyID::kBackgroundPosition,
            "left top, right 2px bottom 3px",
        );
        assert_eq!(layered[0].Value().CssText().Utf8(), "left, right 2px");
        assert_eq!(layered[1].Value().CssText().Utf8(), "top, bottom 3px");
        let CSSValuePayload::kValueListClass(list) = layered[0].Value().Payload() else {
            panic!("position axis layers")
        };
        assert_eq!(list.values.len(), 2);
        for input in ["top 10px", "left right", "left,", "left top top"] {
            assert_eq!(
                rejected(CSSPropertyID::kBackgroundPosition, input).kind,
                PropertyParseErrorKind::Invalid
            );
        }
        assert_eq!(
            rejected(CSSPropertyID::kBackgroundPosition, "calc(10% + 2px)").kind,
            PropertyParseErrorKind::Unsupported
        );
    }
    #[test]
    fn declaration_list_reaches_every_position_and_repeat_consumer() {
        let parsed = ParseDeclarationList(
            &String::from(
                "background-repeat:repeat-x; mask-repeat:round space; background-position:right 2px bottom 3px; mask-position:left top; object-position:top left; perspective-origin:bottom right; offset-anchor:auto; offset-position:normal; transform-origin:left top 20px",
            ),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let properties: Vec<_> = parsed
            .properties
            .iter()
            .map(PropertyValue::PropertyID)
            .collect();
        assert_eq!(
            properties,
            vec![
                CSSPropertyID::kBackgroundRepeat,
                CSSPropertyID::kMaskRepeat,
                CSSPropertyID::kBackgroundPositionX,
                CSSPropertyID::kBackgroundPositionY,
                CSSPropertyID::kWebkitMaskPositionX,
                CSSPropertyID::kWebkitMaskPositionY,
                CSSPropertyID::kObjectPosition,
                CSSPropertyID::kPerspectiveOrigin,
                CSSPropertyID::kOffsetAnchor,
                CSSPropertyID::kOffsetPosition,
                CSSPropertyID::kTransformOrigin
            ]
        );
        assert_eq!(parsed.properties[6].Value().CssText().Utf8(), "left top");
        assert_eq!(
            parsed.properties[10].Value().CssText().Utf8(),
            "left top 20px"
        );
    }
    #[test]
    fn transform_origin_and_position_quirks_match_source_domains() {
        for (input, output) in [
            ("top", "center top"),
            ("left 10px", "left 10px"),
            ("top left 20px", "left top 20px"),
            ("10% 20% -3px", "10% 20% -3px"),
        ] {
            assert_eq!(
                parse(CSSPropertyID::kTransformOrigin, input)[0]
                    .Value()
                    .CssText()
                    .Utf8(),
                output
            );
        }
        for input in [
            "top 10px",
            "left 10px top",
            "10% 20% 30%",
            "left top 20px 30px",
        ] {
            assert_eq!(
                rejected(CSSPropertyID::kTransformOrigin, input).kind,
                PropertyParseErrorKind::Invalid
            );
        }
        assert_eq!(
            parse(CSSPropertyID::kTransformOrigin, "left top calc(1px)")[0]
                .Value()
                .CssText()
                .Utf8(),
            "left top calc(1px)"
        );
        for id in [
            CSSPropertyID::kBackgroundPosition,
            CSSPropertyID::kMaskPosition,
        ] {
            assert!(ParseProperty(
                id,
                &String::from("12 34"),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .is_err());
            let quirks = ParseProperty(
                id,
                &String::from("12 34"),
                false,
                CSSParserMode::kHTMLQuirksMode,
            )
            .unwrap();
            assert_eq!(quirks[0].Value().CssText().Utf8(), "12px");
            assert_eq!(quirks[1].Value().CssText().Utf8(), "34px");
        }
        for id in [
            CSSPropertyID::kObjectPosition,
            CSSPropertyID::kPerspectiveOrigin,
            CSSPropertyID::kOffsetAnchor,
            CSSPropertyID::kOffsetPosition,
            CSSPropertyID::kTransformOrigin,
        ] {
            assert!(ParseProperty(
                id,
                &String::from("12 34"),
                false,
                CSSParserMode::kHTMLQuirksMode
            )
            .is_err());
        }
        let parsed = ParseDeclarationList(
            &String::from(
                "background-position: left 10px top !important; mask-position: left 10px top; object-position: top left",
            ),
            CSSParserMode::kHTMLStandardMode,
        );
        assert_eq!(parsed.properties.len(), 3);
        assert_eq!(parsed.errors.len(), 1);
        assert!(parsed.properties[..2]
            .iter()
            .all(PropertyValue::IsImportant));
    }
    #[test]
    fn alignment_consumers_preserve_payloads_normalization_and_property_domains() {
        for (id, input, output) in [
            (CSSPropertyID::kAlignContent, "normal", "normal"),
            (CSSPropertyID::kAlignContent, "first baseline", "baseline"),
            (CSSPropertyID::kAlignContent, "space-evenly", "space-evenly"),
            (
                CSSPropertyID::kAlignContent,
                "safe flex-end",
                "safe flex-end",
            ),
            (
                CSSPropertyID::kJustifyContent,
                "unsafe right",
                "unsafe right",
            ),
        ] {
            let property = parse(id, input);
            assert!(property[0].Value().IsContentDistributionValue());
            assert_eq!(property[0].Value().CssText().Utf8(), output);
        }
        let content = parse(CSSPropertyID::kAlignContent, "safe center");
        let CSSValuePayload::kCSSContentDistributionClass(value) = content[0].Value().Payload()
        else {
            panic!("content distribution value")
        };
        assert_eq!(value.distribution, CSSValueID::kInvalid);
        assert_eq!(value.position, CSSValueID::kCenter);
        assert_eq!(value.overflow, CSSValueID::kSafe);
        for (id, input, output) in [
            (CSSPropertyID::kAlignItems, "first baseline", "baseline"),
            (CSSPropertyID::kAlignSelf, "last baseline", "last baseline"),
            (CSSPropertyID::kJustifyItems, "right legacy", "legacy right"),
            (
                CSSPropertyID::kJustifyItems,
                "legacy center",
                "legacy center",
            ),
            (CSSPropertyID::kJustifyItems, "legacy", "legacy"),
            (CSSPropertyID::kJustifySelf, "auto", "auto"),
            (
                CSSPropertyID::kAlignSelf,
                "safe anchor-center",
                "safe anchor-center",
            ),
        ] {
            assert_eq!(parse(id, input)[0].Value().CssText().Utf8(), output);
        }
        for (id, input) in [
            (CSSPropertyID::kAlignItems, "auto"),
            (CSSPropertyID::kJustifyItems, "auto"),
            (CSSPropertyID::kAlignItems, "anchor-center"),
            (CSSPropertyID::kJustifyItems, "anchor-center"),
            (CSSPropertyID::kAlignContent, "last baseline"),
            (CSSPropertyID::kJustifyContent, "baseline"),
            (CSSPropertyID::kJustifyContent, "first baseline"),
            (CSSPropertyID::kAlignContent, "left"),
            (CSSPropertyID::kAlignSelf, "safe stretch"),
            (CSSPropertyID::kJustifyItems, "legacy flex-start"),
            (CSSPropertyID::kJustifySelf, "center safe"),
            (CSSPropertyID::kAlignContent, "safe space-between"),
        ] {
            assert_eq!(
                rejected(id, input).kind,
                PropertyParseErrorKind::Invalid,
                "{input}"
            );
        }
    }
    #[test]
    fn place_shorthands_reparse_omissions_and_use_baseline_content_default() {
        for (id, input, first, second) in [
            (
                CSSPropertyID::kPlaceContent,
                "first baseline",
                "baseline",
                "start",
            ),
            (
                CSSPropertyID::kPlaceContent,
                "safe center",
                "safe center",
                "safe center",
            ),
            (
                CSSPropertyID::kPlaceContent,
                "baseline space-around",
                "baseline",
                "space-around",
            ),
            (
                CSSPropertyID::kPlaceItems,
                "last baseline",
                "last baseline",
                "last baseline",
            ),
            (
                CSSPropertyID::kPlaceItems,
                "start legacy right",
                "start",
                "legacy right",
            ),
            (CSSPropertyID::kPlaceSelf, "auto right", "auto", "right"),
            (
                CSSPropertyID::kPlaceSelf,
                "anchor-center",
                "anchor-center",
                "anchor-center",
            ),
        ] {
            let properties = parse(id, input);
            assert_eq!(properties.len(), 2);
            assert_eq!(properties[0].Value().CssText().Utf8(), first);
            assert_eq!(properties[1].Value().CssText().Utf8(), second);
            for p in properties {
                assert!(p.IsSetFromShorthand());
                assert!(!p.IsImplicit());
            }
        }
        for (id, input) in [
            (CSSPropertyID::kPlaceItems, "auto"),
            (CSSPropertyID::kPlaceItems, "anchor-center"),
            (CSSPropertyID::kPlaceContent, "last baseline"),
            (CSSPropertyID::kPlaceContent, "baseline baseline"),
            (CSSPropertyID::kPlaceSelf, "center garbage"),
        ] {
            assert_eq!(rejected(id, input).kind, PropertyParseErrorKind::Invalid);
        }
    }
    #[test]
    fn whitespace_and_textwrap_greedy_shorthands_reset_omitted_longhands() {
        for (id, input, first, second) in [
            (CSSPropertyID::kWhiteSpace, "pre", "preserve", "nowrap"),
            (
                CSSPropertyID::kWhiteSpace,
                "break-spaces",
                "break-spaces",
                "wrap",
            ),
            (
                CSSPropertyID::kWhiteSpace,
                "nowrap break-spaces",
                "break-spaces",
                "nowrap",
            ),
            (
                CSSPropertyID::kWhiteSpace,
                "preserve-breaks wrap",
                "preserve-breaks",
                "wrap",
            ),
            (
                CSSPropertyID::kWhiteSpace,
                "preserve",
                "preserve",
                "initial",
            ),
            (CSSPropertyID::kWhiteSpace, "wrap", "initial", "wrap"),
            (
                CSSPropertyID::kTextWrap,
                "pretty nowrap",
                "nowrap",
                "pretty",
            ),
            (CSSPropertyID::kTextWrap, "balance", "initial", "balance"),
            (CSSPropertyID::kTextWrap, "wrap", "wrap", "initial"),
        ] {
            let properties = parse(id, input);
            assert_eq!(properties[0].Value().CssText().Utf8(), first);
            assert_eq!(properties[1].Value().CssText().Utf8(), second);
            for p in properties {
                assert!(!p.IsImplicit());
            }
        }
        for (id, input) in [
            (CSSPropertyID::kWhiteSpace, "normal wrap"),
            (CSSPropertyID::kWhiteSpace, "collapse preserve"),
            (CSSPropertyID::kTextWrap, "balance pretty"),
            (CSSPropertyID::kTextWrap, "wrap nowrap"),
        ] {
            assert_eq!(rejected(id, input).kind, PropertyParseErrorKind::Invalid);
        }
        let declarations = ParseDeclarationList(
            &String::from(
                "white-space: break-spaces nowrap !important; text-wrap: pretty !important",
            ),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(declarations.errors.is_empty());
        assert_eq!(declarations.properties.len(), 4);
        assert!(declarations
            .properties
            .iter()
            .all(PropertyValue::IsImportant));
    }
    #[test]
    fn flex_wrap_balance_normalizes_order_and_flex_flow_uses_concrete_initials() {
        for (input, output) in [
            ("nowrap", "nowrap"),
            ("wrap", "wrap"),
            ("balance", "balance"),
            ("wrap balance", "balance"),
            ("balance wrap", "balance"),
            ("balance wrap-reverse", "wrap-reverse balance"),
        ] {
            assert_eq!(
                parse(CSSPropertyID::kFlexWrap, input)[0]
                    .Value()
                    .CssText()
                    .Utf8(),
                output
            );
        }
        for input in ["nowrap balance", "balance balance", "wrap wrap-reverse"] {
            assert_eq!(
                rejected(CSSPropertyID::kFlexWrap, input).kind,
                PropertyParseErrorKind::Invalid
            );
        }
        for (input, direction, wrap) in [
            ("balance wrap row", "row", "balance"),
            (
                "column balance wrap-reverse",
                "column",
                "wrap-reverse balance",
            ),
            ("wrap", "row", "wrap"),
            ("column", "column", "nowrap"),
        ] {
            let properties = parse(CSSPropertyID::kFlexFlow, input);
            assert_eq!(properties[0].Value().CssText().Utf8(), direction);
            assert_eq!(properties[1].Value().CssText().Utf8(), wrap);
            assert!(properties.iter().all(|p| !p.IsImplicit()));
        }
    }
    #[test]
    fn animation_lists_use_typed_names_time_units_and_keyword_domains() {
        for (id, input, output) in [
            (
                CSSPropertyID::kAnimationDuration,
                "auto, 250ms, 2s",
                "auto, 250ms, 2s",
            ),
            (CSSPropertyID::kAnimationDelay, "-1s, 0ms", "-1s, 0ms"),
            (
                CSSPropertyID::kAnimationIterationCount,
                "infinite, 0, 2.5",
                "infinite, 0, 2.5",
            ),
            (
                CSSPropertyID::kAnimationDirection,
                "reverse, alternate-reverse",
                "reverse, alternate-reverse",
            ),
            (
                CSSPropertyID::kAnimationComposition,
                "replace, add, accumulate",
                "replace, add, accumulate",
            ),
            (
                CSSPropertyID::kAnimationPlayState,
                "running, paused",
                "running, paused",
            ),
            (
                CSSPropertyID::kAnimationFillMode,
                "none, both",
                "none, both",
            ),
            (
                CSSPropertyID::kAnimationName,
                "Slide, 'slide', 'none', 'inherit', 'a b'",
                "Slide, slide, \"none\", \"inherit\", a\\ b",
            ),
        ] {
            let properties = parse(id, input);
            assert!(properties[0].Value().IsValueList());
            assert_eq!(properties[0].Value().CssText().Utf8(), output);
        }
        let names = parse(CSSPropertyID::kAnimationName, "foo, 'none'");
        let CSSValuePayload::kValueListClass(list) = names[0].Value().Payload() else {
            panic!("list")
        };
        assert!(list.values[0].IsCustomIdentValue());
        assert!(list.values[0].State().NeedsTreeScopePopulation());
        assert!(list.values[1].IsStringValue());
        assert!(names[0].Value().State().NeedsTreeScopePopulation());
        for (id, input) in [
            (CSSPropertyID::kAnimationDuration, "-1s"),
            (CSSPropertyID::kAnimationDuration, "0"),
            (CSSPropertyID::kAnimationIterationCount, "-1"),
            (CSSPropertyID::kAnimationName, "''"),
            (CSSPropertyID::kAnimationName, "default"),
            (CSSPropertyID::kAnimationName, "foo,"),
            (CSSPropertyID::kAnimationDirection, "paused"),
        ] {
            assert_eq!(
                rejected(id, input).kind,
                PropertyParseErrorKind::Invalid,
                "{input}"
            );
        }
    }
    #[test]
    fn animation_typed_values_share_source_identifier_and_small_number_pools() {
        assert!(Rc::ptr_eq(
            &values::identifier(CSSValueID::kPlay),
            &values::identifier(CSSValueID::kPlay)
        ));
        let first = values::numeric(4.0, UnitType::kNumber);
        let second = values::numeric(4.0, UnitType::kInteger);
        assert!(Rc::ptr_eq(&first, &second));
        let CSSValuePayload::kNumericLiteralClass(number) = first.Payload() else {
            panic!("numeric")
        };
        assert_eq!(number.GetType(), UnitType::kInteger);
        assert!(!Rc::ptr_eq(
            &values::numeric(256.0, UnitType::kNumber),
            &values::numeric(256.0, UnitType::kNumber)
        ));
        assert!(!Rc::ptr_eq(
            &values::numeric(-0.0, UnitType::kPixels),
            &values::numeric(0.0, UnitType::kPixels)
        ));
        assert!(Rc::ptr_eq(
            &values::numeric(255.0, UnitType::kPixels),
            &values::numeric(255.0, UnitType::kPixels)
        ));
        assert!(!Rc::ptr_eq(
            &values::numeric(2.0, UnitType::kPixels),
            &values::numeric(2.0, UnitType::kPercentage)
        ));
        let first = parse(CSSPropertyID::kAnimationTimingFunction, "steps(4)");
        let second = parse(CSSPropertyID::kAnimationTimingFunction, "steps(4)");
        assert!(first[0].Value() == second[0].Value());
        let first = parse(CSSPropertyID::kAnimationTrigger, "--trigger play reset");
        let second = parse(CSSPropertyID::kAnimationTrigger, "--trigger play reset");
        assert!(first[0].Value() == second[0].Value());
    }
    #[test]
    fn easing_function_guards_ranges_and_normalization_follow_source() {
        for (input, output) in [
            ("steps(4, end)", "steps(4)"),
            ("steps(4, jump-end)", "steps(4)"),
            ("steps(2, jump-none)", "steps(2, jump-none)"),
            ("steps(1, jump-both)", "steps(1, jump-both)"),
            (
                "cubic-bezier(.2, -3, .8, 4)",
                "cubic-bezier(0.2, -3, 0.8, 4)",
            ),
            ("linear(0, .5, 1)", "linear(0 0%, 0.5 50%, 1 100%)"),
            ("linear(0 20% 40%, 1 30%)", "linear(0 20%, 0 40%, 1 40%)"),
            ("linear(20% 0, 80% 1)", "linear(0 20%, 1 80%)"),
            (
                "linear(0, .2, .6, 1)",
                "linear(0 0%, 0.2 33.3333%, 0.6 66.6667%, 1 100%)",
            ),
        ] {
            let properties = parse(CSSPropertyID::kAnimationTimingFunction, input);
            assert_eq!(properties[0].Value().CssText().Utf8(), output);
        }
        let functions = parse(
            CSSPropertyID::kTransitionTimingFunction,
            "linear(0, 1), steps(3), cubic-bezier(0, 1, 1, 0)",
        );
        let CSSValuePayload::kValueListClass(list) = functions[0].Value().Payload() else {
            panic!("list")
        };
        assert!(list.values[0].IsLinearTimingFunctionValue());
        assert!(list.values[1].IsStepsTimingFunctionValue());
        assert!(list.values[2].IsCubicBezierTimingFunctionValue());
        for input in [
            "steps(0)",
            "steps(1.5)",
            "steps(1, jump-none)",
            "steps(2, bad)",
            "cubic-bezier(0, 1, 2, 1)",
            "cubic-bezier(0, 1, 1)",
            "linear(1)",
            "linear(0 0% 20% 30%, 1)",
            "linear(0%, 1)",
            "steps(2) garbage",
        ] {
            assert_eq!(
                rejected(CSSPropertyID::kAnimationTimingFunction, input).kind,
                PropertyParseErrorKind::Invalid,
                "{input}"
            );
        }
        let duration = parse(CSSPropertyID::kAnimationDuration, "calc(1s + 1s)");
        let CSSValuePayload::kValueListClass(list) = duration[0].Value().Payload() else {
            panic!()
        };
        assert!(matches!(
            list.values[0].Payload(),
            CSSValuePayload::kMathFunctionClass(_)
        ));
        for (id, input) in [
            (CSSPropertyID::kAnimationTimingFunction, "steps(round(2.5))"),
            (
                CSSPropertyID::kAnimationTimingFunction,
                "cubic-bezier(sin(.2), 0, 1, 1)",
            ),
            (
                CSSPropertyID::kAnimationTimingFunction,
                "linear(sign(0), 1)",
            ),
        ] {
            assert_eq!(
                rejected(id, input).kind,
                PropertyParseErrorKind::Unsupported
            );
        }
    }
    #[test]
    fn animation_shorthand_disambiguates_and_resets_once_for_multiple_items() {
        let properties = parse(
            CSSPropertyID::kAnimation,
            "slide 2s ease-in -250ms 2 alternate both paused, 1s linear none",
        );
        let value = |id| {
            properties
                .iter()
                .find(|p| p.PropertyID() == id)
                .unwrap()
                .Value()
                .CssText()
                .Utf8()
        };
        assert_eq!(properties.len(), 11);
        assert_eq!(value(CSSPropertyID::kAnimationDuration), "2s, 1s");
        assert_eq!(
            value(CSSPropertyID::kAnimationTimingFunction),
            "ease-in, linear"
        );
        assert_eq!(value(CSSPropertyID::kAnimationDelay), "-250ms, 0s");
        assert_eq!(value(CSSPropertyID::kAnimationIterationCount), "2, 1");
        assert_eq!(
            value(CSSPropertyID::kAnimationDirection),
            "alternate, normal"
        );
        assert_eq!(value(CSSPropertyID::kAnimationName), "slide, none");
        assert_eq!(value(CSSPropertyID::kAnimationTimeline), "auto");
        assert_eq!(value(CSSPropertyID::kAnimationRangeStart), "normal");
        assert_eq!(value(CSSPropertyID::kAnimationRangeEnd), "normal");
        assert!(properties
            .iter()
            .all(|p| p.IsSetFromShorthand() && !p.IsImplicit()));
        let properties = parse(CSSPropertyID::kAnimation, "ease ease");
        assert_eq!(
            properties
                .iter()
                .find(|p| p.PropertyID() == CSSPropertyID::kAnimationName)
                .unwrap()
                .Value()
                .CssText()
                .Utf8(),
            "ease"
        );
        let properties = parse(CSSPropertyID::kAnimation, "-2s");
        assert_eq!(properties[0].Value().CssText().Utf8(), "auto");
        assert_eq!(properties[2].Value().CssText().Utf8(), "-2s");
        for input in ["foo bar", "1s 2s 3s", "foo,", ",foo", "foo scroll()"] {
            assert!(ParseProperty(
                CSSPropertyID::kAnimation,
                &String::from(input),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .is_err());
        }
    }
    #[test]
    fn transition_uses_source_parse_order_known_property_ids_and_none_validation() {
        let properties = parse(
            CSSPropertyID::kTransition,
            "opacity 1s ease 200ms allow-discrete, 2s linear",
        );
        assert_eq!(
            properties
                .iter()
                .map(PropertyValue::PropertyID)
                .collect::<Vec<_>>(),
            vec![
                CSSPropertyID::kTransitionBehavior,
                CSSPropertyID::kTransitionDuration,
                CSSPropertyID::kTransitionTimingFunction,
                CSSPropertyID::kTransitionDelay,
                CSSPropertyID::kTransitionProperty
            ]
        );
        assert_eq!(
            properties[0].Value().CssText().Utf8(),
            "allow-discrete, normal"
        );
        assert_eq!(properties[2].Value().CssText().Utf8(), "ease, linear");
        assert_eq!(properties[4].Value().CssText().Utf8(), "opacity, all");
        let known = parse(
            CSSPropertyID::kTransitionProperty,
            "WIDTH, -webkit-transform, future-property",
        );
        assert_eq!(
            known[0].Value().CssText().Utf8(),
            "width, -webkit-transform, future-property"
        );
        let CSSValuePayload::kValueListClass(list) = known[0].Value().Payload() else {
            panic!("list")
        };
        let CSSValuePayload::kCustomIdentClass(first) = list.values[0].Payload() else {
            panic!("known property")
        };
        assert_eq!(first.property, CSSPropertyID::kWidth);
        assert!(!list.values[0].State().NeedsTreeScopePopulation());
        let descriptor = parse(CSSPropertyID::kTransitionProperty, "FONT-DISPLAY");
        let CSSValuePayload::kValueListClass(items) = descriptor[0].Value().Payload() else {
            panic!("list")
        };
        let CSSValuePayload::kCustomIdentClass(descriptor) = items.values[0].Payload() else {
            panic!("custom")
        };
        assert_eq!(descriptor.property, CSSPropertyID::kInvalid);
        assert_eq!(items.values[0].CssText().Utf8(), "FONT-DISPLAY");

        assert_eq!(
            parse(CSSPropertyID::kTransitionBehavior, "normal, allow-discrete")[0]
                .Value()
                .CssText()
                .Utf8(),
            "normal, allow-discrete"
        );
        for (id, input) in [
            (CSSPropertyID::kTransitionProperty, "none, opacity"),
            (CSSPropertyID::kTransition, "none 1s, opacity 2s"),
            (CSSPropertyID::kTransitionDuration, "auto"),
            (CSSPropertyID::kTransitionDuration, "-1s"),
            (CSSPropertyID::kTransition, "1s 2s 3s"),
        ] {
            assert_eq!(
                rejected(id, input).kind,
                PropertyParseErrorKind::Invalid,
                "{input}"
            );
        }
    }
    #[test]
    fn animation_timeline_range_literals_and_implied_ends_follow_source() {
        assert_eq!(
            parse(CSSPropertyID::kAnimationTimeline, "auto, none, --scroll")[0]
                .Value()
                .CssText()
                .Utf8(),
            "auto, none, --scroll"
        );
        assert_eq!(
            parse(
                CSSPropertyID::kAnimationRangeStart,
                "entry 0%, scroll -20px, 10%"
            )[0]
            .Value()
            .CssText()
            .Utf8(),
            "entry, scroll -20px, 10%"
        );
        assert_eq!(
            parse(
                CSSPropertyID::kAnimationRangeEnd,
                "cover 100%, exit-crossing 80%"
            )[0]
            .Value()
            .CssText()
            .Utf8(),
            "cover, exit-crossing 80%"
        );
        let properties = parse(CSSPropertyID::kAnimationRange, "entry 20%, 10% 80%, normal");
        assert_eq!(
            properties[0].Value().CssText().Utf8(),
            "entry 20%, 10%, normal"
        );
        assert_eq!(properties[1].Value().CssText().Utf8(), "entry, 80%, normal");
        for input in ["scroll()", "view(inline)"] {
            assert_eq!(
                rejected(CSSPropertyID::kAnimationTimeline, input).kind,
                PropertyParseErrorKind::Unsupported
            );
        }
        assert_eq!(
            rejected(CSSPropertyID::kAnimationTimeline, "foo").kind,
            PropertyParseErrorKind::Invalid
        );
        let math_range = parse(CSSPropertyID::kAnimationRange, "entry calc(10% + 2px)");
        let CSSValuePayload::kValueListClass(comma) = math_range[0].Value().Payload() else {
            panic!("comma range list")
        };
        let CSSValuePayload::kValueListClass(range) = comma.values[0].Payload() else {
            panic!("named range")
        };
        assert!(matches!(
            range.values[1].Payload(),
            CSSValuePayload::kMathFunctionClass(_)
        ));
        let trigger = parse(
            CSSPropertyID::kAnimationTrigger,
            "none, --trigger play reset, --other play-once",
        );
        assert_eq!(
            trigger[0].Value().CssText().Utf8(),
            "none, --trigger play reset, --other play-once"
        );
        for input in [
            "--trigger",
            "--trigger bad",
            "foo play",
            "--one play --two pause",
        ] {
            assert_eq!(
                rejected(CSSPropertyID::kAnimationTrigger, input).kind,
                PropertyParseErrorKind::Invalid
            );
        }
        let declarations = ParseDeclarationList(
            &String::from(
                "animation: foo 1s, bar 2s !important; transition: opacity 1s !important",
            ),
            CSSParserMode::kHTMLStandardMode,
        );
        let prefixed = ParseDeclarationList(
            &String::from("-webkit-animation: foo 1s; -webkit-transition: opacity 1s"),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(prefixed.errors.is_empty());
        assert_eq!(prefixed.properties.len(), 16);
        assert!(declarations.errors.is_empty());
        assert_eq!(declarations.properties.len(), 16);
        assert!(declarations
            .properties
            .iter()
            .all(PropertyValue::IsImportant));
    }
    #[test]
    fn background_layers_keep_source_value_types_and_alias_rules() {
        let layers = parse(CSSPropertyID::kBackgroundAttachment, "fixed, local, scroll");
        assert_eq!(layers[0].Value().CssText().Utf8(), "fixed, local, scroll");
        let CSSValuePayload::kValueListClass(list) = layers[0].Value().Payload() else {
            panic!("comma list")
        };
        assert_eq!(list.values.len(), 3);
        assert_eq!(
            parse(CSSPropertyID::kBackgroundBlendMode, "multiply, overlay")[0]
                .Value()
                .CssText()
                .Utf8(),
            "multiply, overlay"
        );
        assert_eq!(
            rejected(CSSPropertyID::kBackgroundBlendMode, "plus-lighter").kind,
            PropertyParseErrorKind::Invalid
        );
        assert_eq!(
            parse(
                CSSPropertyID::kBackgroundClip,
                "text border-area, padding-box"
            )[0]
            .Value()
            .CssText()
            .Utf8(),
            "border-area text, padding-box"
        );
        assert_eq!(
            parse(CSSPropertyID::kAliasWebkitBackgroundClip, "text")[0]
                .Value()
                .CssText()
                .Utf8(),
            "text"
        );
        assert_eq!(
            rejected(CSSPropertyID::kAliasWebkitBackgroundClip, "border-area").kind,
            PropertyParseErrorKind::Invalid
        );
        assert_eq!(
            parse(
                CSSPropertyID::kAliasWebkitBackgroundOrigin,
                "border, content"
            )[0]
            .Value()
            .CssText()
            .Utf8(),
            "border, content"
        );
        assert_eq!(
            rejected(CSSPropertyID::kBackgroundOrigin, "border").kind,
            PropertyParseErrorKind::Invalid
        );
        assert_eq!(
            rejected(CSSPropertyID::kBackgroundAttachment, "fixed,").kind,
            PropertyParseErrorKind::Invalid
        );
    }
    #[test]
    fn background_size_axes_and_legacy_size_follow_source_serialization() {
        let sizes = parse(
            CSSPropertyID::kBackgroundSize,
            "20px, auto auto, cover, 10% 10%",
        );
        assert_eq!(
            sizes[0].Value().CssText().Utf8(),
            "20px auto, auto, cover, 10% 10%"
        );
        assert_eq!(
            parse(CSSPropertyID::kAliasWebkitBackgroundSize, "20px")[0]
                .Value()
                .CssText()
                .Utf8(),
            "20px 20px"
        );
        assert_eq!(
            parse(CSSPropertyID::kAliasWebkitBackgroundSize, "20px, 40px")[0]
                .Value()
                .CssText()
                .Utf8(),
            "20px, 40px 40px"
        );
        assert_eq!(
            parse(
                CSSPropertyID::kBackgroundPositionX,
                "right 2px, center, -20%"
            )[0]
            .Value()
            .CssText()
            .Utf8(),
            "right 2px, center, -20%"
        );
        assert_eq!(
            numeric(&parse(CSSPropertyID::kWebkitPerspectiveOriginX, "right")[0]),
            (100.0, UnitType::kPercentage)
        );
        assert_eq!(
            numeric(&parse(CSSPropertyID::kWebkitTransformOriginY, "center")[0]),
            (50.0, UnitType::kPercentage)
        );
        assert_eq!(
            rejected(CSSPropertyID::kBackgroundPositionX, "top").kind,
            PropertyParseErrorKind::Invalid
        );
        assert_eq!(
            rejected(CSSPropertyID::kBackgroundSize, "-1px").kind,
            PropertyParseErrorKind::Invalid
        );
        let math = parse(CSSPropertyID::kBackgroundSize, "calc(2px)");
        let CSSValuePayload::kValueListClass(list) = math[0].Value().Payload() else {
            panic!()
        };
        let CSSValuePayload::kValuePairClass(pair) = list.values[0].Payload() else {
            panic!()
        };
        assert!(matches!(
            pair.first.Payload(),
            CSSValuePayload::kMathFunctionClass(_)
        ));
        assert!(pair.second.IsIdentifierValue());
    }
    #[test]
    fn mask_coord_boxes_and_prefixed_composite_use_distinct_source_grammars() {
        assert_eq!(
            parse(CSSPropertyID::kMaskClip, "no-clip, fill-box")[0]
                .Value()
                .CssText()
                .Utf8(),
            "no-clip, fill-box"
        );
        assert_eq!(
            parse(CSSPropertyID::kMaskOrigin, "stroke-box, view-box")[0]
                .Value()
                .CssText()
                .Utf8(),
            "stroke-box, view-box"
        );
        assert_eq!(
            parse(CSSPropertyID::kMaskComposite, "add, exclude")[0]
                .Value()
                .CssText()
                .Utf8(),
            "add, exclude"
        );
        assert_eq!(
            parse(CSSPropertyID::kAliasWebkitMaskComposite, "source-over, xor")[0]
                .Value()
                .CssText()
                .Utf8(),
            "source-over, xor"
        );
        assert_eq!(
            parse(CSSPropertyID::kMaskMode, "alpha, match-source")[0]
                .Value()
                .CssText()
                .Utf8(),
            "alpha, match-source"
        );
        assert_eq!(
            parse(CSSPropertyID::kMaskSize, "20px")[0]
                .Value()
                .CssText()
                .Utf8(),
            "20px auto"
        );
        assert_eq!(
            parse(CSSPropertyID::kWebkitMaskPositionY, "bottom 2em")[0]
                .Value()
                .CssText()
                .Utf8(),
            "bottom 2em"
        );
        assert_eq!(
            rejected(CSSPropertyID::kMaskComposite, "xor").kind,
            PropertyParseErrorKind::Invalid
        );
        assert_eq!(
            rejected(CSSPropertyID::kAliasWebkitMaskComposite, "add").kind,
            PropertyParseErrorKind::Invalid
        );
        assert_eq!(
            rejected(CSSPropertyID::kMaskOrigin, "no-clip").kind,
            PropertyParseErrorKind::Invalid
        );
    }
    #[test]
    fn grid_numeric_lines_and_shorthands_preserve_span_and_default_auto() {
        assert_eq!(
            parse(CSSPropertyID::kGridAutoFlow, "dense row")[0]
                .Value()
                .CssText()
                .Utf8(),
            "dense"
        );
        assert_eq!(
            parse(CSSPropertyID::kGridAutoFlow, "dense column")[0]
                .Value()
                .CssText()
                .Utf8(),
            "column dense"
        );
        assert_eq!(
            parse(CSSPropertyID::kGridRowStart, "2 span")[0]
                .Value()
                .CssText()
                .Utf8(),
            "span 2"
        );
        let column = parse(CSSPropertyID::kGridColumn, "-1 / span 2");
        assert_eq!(column[0].Value().CssText().Utf8(), "-1");
        assert_eq!(column[1].Value().CssText().Utf8(), "span 2");
        let area = parse(CSSPropertyID::kGridArea, "2 / 3");
        assert_eq!(area.len(), 4);
        assert!(area
            .iter()
            .all(|p| p.ShorthandID() == CSSPropertyID::kGridArea));
        assert_eq!(area[2].Value().CssText().Utf8(), "auto");
        assert_eq!(area[3].Value().CssText().Utf8(), "auto");
        for value in [
            "0",
            "span 0",
            "span -1",
            "span",
            "span auto",
            "1 /",
            "1 / 2 / 3",
        ] {
            assert!(
                ParseProperty(
                    CSSPropertyID::kGridRow,
                    &String::from(value),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .is_err(),
                "{value}"
            );
        }
        let error = rejected(CSSPropertyID::kGridColumnStart, "sidebar");
        assert_eq!(error.kind, PropertyParseErrorKind::Unsupported);
        assert!(error.operation.contains("CSSCustomIdentValue"));
    }
    #[test]
    fn overflow_clip_margin_normalizes_box_and_zero_and_stretch_percent_ranges() {
        assert_eq!(
            parse(CSSPropertyID::kOverflowClipMargin, "padding-box")[0]
                .Value()
                .CssText()
                .Utf8(),
            "0px"
        );
        assert_eq!(
            parse(CSSPropertyID::kOverflowClipMargin, "border-box 0")[0]
                .Value()
                .CssText()
                .Utf8(),
            "border-box"
        );
        assert_eq!(
            parse(CSSPropertyID::kOverflowClipMargin, "-2px content-box")[0]
                .Value()
                .CssText()
                .Utf8(),
            "content-box -2px"
        );
        assert_eq!(
            rejected(CSSPropertyID::kOverflowClipMargin, "0").kind,
            PropertyParseErrorKind::Invalid
        );
        assert_eq!(
            rejected(CSSPropertyID::kOverflowClipMargin, "10%").kind,
            PropertyParseErrorKind::Invalid
        );
        assert_eq!(
            parse(CSSPropertyID::kFontStretch, "semi-condensed")[0]
                .Value()
                .CssText()
                .Utf8(),
            "semi-condensed"
        );
        assert_eq!(
            numeric(&parse(CSSPropertyID::kFontStretch, "75%")[0]),
            (75.0, UnitType::kPercentage)
        );
        assert_eq!(
            rejected(CSSPropertyID::kFontStretch, "75% 100%").kind,
            PropertyParseErrorKind::Invalid
        );
        assert_eq!(
            rejected(CSSPropertyID::kFontStretch, "-1%").kind,
            PropertyParseErrorKind::Invalid
        );
        let range = ParseProperty(
            CSSPropertyID::kFontStretch,
            &String::from("100% 75%"),
            false,
            CSSParserMode::kCSSFontFaceRuleMode,
        )
        .unwrap();
        assert_eq!(range[0].Value().CssText().Utf8(), "100% 75%");
    }
    #[test]
    fn declarations_preserve_order_importance_and_source_offsets() {
        let parsed = ParseDeclarationList(
            &String::from(
                r"margin: 1px 2px!important; display: flex; width: 8px; width: 12px !\69 mportant",
            ),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        assert_eq!(parsed.properties.len(), 7);
        assert_eq!(parsed.source_offsets.len(), 7);
        assert_eq!(numeric(&parsed.properties[0]), (1.0, UnitType::kPixels));
        assert_eq!(numeric(&parsed.properties[1]), (2.0, UnitType::kPixels));
        assert_eq!(numeric(&parsed.properties[2]), (1.0, UnitType::kPixels));
        assert_eq!(numeric(&parsed.properties[3]), (2.0, UnitType::kPixels));
        assert!(parsed.properties[..4]
            .iter()
            .all(PropertyValue::IsImportant));
        assert!(parsed.properties[..4]
            .iter()
            .all(|property| property.ShorthandID() == CSSPropertyID::kMargin));
        assert!(parsed.properties[6].IsImportant());
        assert_eq!(parsed.properties[4].PropertyID(), CSSPropertyID::kDisplay);
        assert!(parsed.source_offsets[4] > parsed.source_offsets[0]);
    }
    #[test]
    fn invalid_shorthands_are_transactional_and_basic_math_is_typed() {
        let parsed = ParseDeclarationList(
            &String::from(
                "margin: 2px bad; width:calc(100% - 2px); color:red; --x:4px; grid-template-columns:repeat(2,1fr); padding:-1px",
            ),
            CSSParserMode::kHTMLStandardMode,
        );
        assert_eq!(parsed.properties.len(), 3);
        assert_eq!(parsed.properties[0].PropertyID(), CSSPropertyID::kWidth);
        assert!(matches!(
            parsed.properties[0].Value().Payload(),
            CSSValuePayload::kMathFunctionClass(_)
        ));
        assert_eq!(parsed.properties[1].PropertyID(), CSSPropertyID::kColor);
        assert_eq!(parsed.properties[2].PropertyID(), CSSPropertyID::kVariable);
        assert_eq!(parsed.properties[2].CustomPropertyName().Utf8(), "--x");
        assert_eq!(parsed.errors.len(), 3);
        assert_eq!(parsed.errors[0].kind, PropertyParseErrorKind::Invalid);
        assert_eq!(parsed.errors[1].kind, PropertyParseErrorKind::Unsupported);
        assert_eq!(
            parsed.errors[1].property,
            CSSPropertyID::kGridTemplateColumns
        );
        assert_eq!(
            parsed.errors[1].source_file,
            super::super::production_property_metadata::ParserSourceFor(
                CSSPropertyID::kGridTemplateColumns
            )
            .0
        );
        assert_eq!(parsed.errors[2].kind, PropertyParseErrorKind::Invalid);
    }
    #[test]
    fn tokenizer_blocks_strings_comments_and_escaped_property_names() {
        let parsed = ParseDeclarationList(
            &String::from(
                r#"font-family: "semi;colon", serif; c\6f lor: /*comment*/ #1a2b; width: var(--size, 1px; 2px); height:4px"#,
            ),
            CSSParserMode::kHTMLStandardMode,
        );
        assert_eq!(parsed.properties.len(), 3);
        assert_eq!(parsed.errors.len(), 1);
        assert_eq!(parsed.errors[0].property, CSSPropertyID::kWidth);
        assert_eq!(parsed.errors[0].kind, PropertyParseErrorKind::Invalid);
        assert_eq!(
            parsed.properties[0].PropertyID(),
            CSSPropertyID::kFontFamily
        );
        let CSSValuePayload::kValueListClass(families) = parsed.properties[0].Value().Payload()
        else {
            panic!("family list")
        };
        assert_eq!(families.values.len(), 2);
        assert_eq!(parsed.properties[1].PropertyID(), CSSPropertyID::kColor);
    }
    #[test]
    fn literal_ranges_units_and_quirks_follow_source_consumers() {
        assert!(ParseProperty(
            CSSPropertyID::kWidth,
            &String::from("12"),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .is_err());
        let quirks = ParseProperty(
            CSSPropertyID::kWidth,
            &String::from("12"),
            false,
            CSSParserMode::kHTMLQuirksMode,
        )
        .unwrap();
        assert_eq!(numeric(&quirks[0]), (12.0, UnitType::kPixels));
        assert!(ParseProperty(
            CSSPropertyID::kPaddingTop,
            &String::from("-2px"),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .is_err());
        assert_eq!(
            numeric(&parse(CSSPropertyID::kMarginTop, "-2%")[0]),
            (-2.0, UnitType::kPercentage)
        );
        assert_eq!(
            numeric(&parse(CSSPropertyID::kOpacity, "50%")[0]),
            (0.5, UnitType::kNumber)
        );
        assert_eq!(
            numeric(&parse(CSSPropertyID::kFontWeight, "450.5")[0]),
            (450.5, UnitType::kNumber)
        );
        assert!(ParseProperty(
            CSSPropertyID::kFontWeight,
            &String::from("1001"),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .is_err());
    }
    #[test]
    fn wide_keywords_expand_and_border_resets_source_longhands() {
        let wide = parse(CSSPropertyID::kMargin, "inherit!important");
        assert_eq!(wide.len(), 4);
        assert!(wide
            .iter()
            .all(|property| property.Value().IsInheritedValue() && property.IsImportant()));
        let border = parse(CSSPropertyID::kBorder, "2px solid rgb(255 0 0 / 50%)");
        assert_eq!(border.len(), 17);
        assert_eq!(
            border
                .iter()
                .filter(|property| property.Value().IsInitialValue())
                .count(),
            5
        );
        let CSSValuePayload::kColorClass(color) = border[0].Value().Payload() else {
            panic!("color")
        };
        assert_eq!(color.0.Param0(), 255.0);
        assert_eq!(color.0.Alpha(), 0.5);
        assert_eq!(border[0].ShorthandID(), CSSPropertyID::kBorder);
    }
    #[test]
    fn display_compounds_radius_pairs_and_rgb_token_boundaries_match_source() {
        let display = parse(CSSPropertyID::kDisplay, "inline flex");
        let CSSValuePayload::kIdentifierClass(value) = display[0].Value().Payload() else {
            panic!("legacy display keyword")
        };
        assert_eq!(value.0, CSSValueID::kInlineFlex);
        let radii = parse(CSSPropertyID::kBorderRadius, "2px 4px / 6px 8px");
        assert_eq!(radii.len(), 4);
        let CSSValuePayload::kValuePairClass(pair) = radii[1].Value().Payload() else {
            panic!("radius pair")
        };
        assert_eq!(pair.first.CssText().Utf8(), "4px");
        assert_eq!(pair.second.CssText().Utf8(), "8px");
        let alias = parse(CSSPropertyID::kAliasWebkitBorderRadius, "2px 4px");
        assert_eq!(alias[0].Value().CssText().Utf8(), "2px 4px");
        assert!(ParseProperty(
            CSSPropertyID::kColor,
            &String::from("rgb(1+2+3)"),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .is_err());
        assert!(ParseProperty(
            CSSPropertyID::kColor,
            &String::from("rgb(1, 2%, 3)"),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .is_err());
        assert!(values::NamedColor(CSSValueID::kRebeccapurple).is_some());
    }
    #[test]
    fn font_flex_and_grid_are_real_typed_values() {
        let font = parse(CSSPropertyID::kFontFamily, "Arial, 'Twisty Tie', serif");
        let CSSValuePayload::kValueListClass(list) = font[0].Value().Payload() else {
            panic!("list")
        };
        assert_eq!(list.values.len(), 3);
        assert_eq!(
            font[0].Value().CssText().Utf8(),
            "Arial, \"Twisty Tie\", serif"
        );
        let flex = parse(CSSPropertyID::kFlex, "2 0 10px");
        assert_eq!(numeric(&flex[0]), (2.0, UnitType::kInteger));
        assert_eq!(numeric(&flex[1]), (0.0, UnitType::kInteger));
        assert_eq!(numeric(&flex[2]), (10.0, UnitType::kPixels));
        assert!(ParseProperty(
            CSSPropertyID::kFlex,
            &String::from("1 auto 2"),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .is_err());
        let grid = parse(CSSPropertyID::kGridTemplateColumns, "1fr min-content 20%");
        let CSSValuePayload::kValueListClass(tracks) = grid[0].Value().Payload() else {
            panic!("grid list")
        };
        assert_eq!(tracks.values.len(), 3);
        let CSSValuePayload::kNumericLiteralClass(fr) = tracks.values[0].Payload() else {
            panic!("fr")
        };
        assert_eq!(fr.GetType(), UnitType::kFlex);
    }
}
