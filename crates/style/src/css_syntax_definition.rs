// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: css_syntax_definition.h:26-75/.cc:26-170,244-367.
// ConsumeSingleType (.cc:173-242) uses actual typed value consumers; image/url/
// transform/CSSMath dependencies report Unsupported instead of guessing values.
#![allow(non_snake_case)]
use crate::{
    css_numeric_literal_value::IsLength,
    css_primitive_value::UnitType,
    css_syntax_component::{CSSSyntaxComponent, CSSSyntaxRepeat, CSSSyntaxType},
    css_syntax_string_parser::{SyntaxType, ValidSyntaxIdent},
    parser::{
        css_parser_token::{CSSParserTokenType::*, NumericValueType},
        css_parser_token_stream::{CSSParserTokenStream, TokenStreamTokenizer},
        production_property_parser::{ConsumeColor, VariableTokenReplay},
    },
    production_css_value as values,
};
use foundation::{CSSPropertyID, String};
use std::rc::Rc;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CSSSyntaxDefinition {
    syntax_components_: Vec<CSSSyntaxComponent>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntaxValueError {
    Invalid,
    Unsupported(CSSSyntaxType),
}
impl CSSSyntaxDefinition {
    pub fn new(components: Vec<CSSSyntaxComponent>) -> Self {
        Self {
            syntax_components_: components,
        }
    }
    pub fn Components(&self) -> &[CSSSyntaxComponent] {
        &self.syntax_components_
    }
    pub fn IsUniversal(&self) -> bool {
        self.syntax_components_.len() == 1
            && self.syntax_components_[0].GetType() == CSSSyntaxType::kTokenStream
    }
    pub fn ContainsUrlComponent(&self) -> bool {
        self.syntax_components_
            .iter()
            .any(|c| c.GetType() == CSSSyntaxType::kUrl)
    }
    pub fn CreateUniversal() -> Self {
        Self::new(vec![CSSSyntaxComponent::new(
            CSSSyntaxType::kTokenStream,
            &String::new(),
            CSSSyntaxRepeat::kNone,
        )])
    }
    pub fn CreateNumericSyntax() -> Self {
        use CSSSyntaxType::*;
        Self::new(
            [kNumber, kLength, kPercentage, kAngle, kTime, kResolution]
                .into_iter()
                .map(|ty| CSSSyntaxComponent::new(ty, &String::new(), CSSSyntaxRepeat::kNone))
                .collect(),
        )
    }
    pub fn ToString(&self) -> String {
        String::from(
            self.syntax_components_
                .iter()
                .map(|c| c.ToString().Utf8())
                .collect::<Vec<_>>()
                .join(" | ")
                .as_str(),
        )
    }
    pub fn Consume<T: TokenStreamTokenizer>(s: &mut CSSParserTokenStream<'_, T>) -> Option<Self> {
        if s.Peek().GetType() == kDelimiterToken && s.Peek().Delimiter() == 42 {
            s.ConsumeIncludingWhitespace();
            return Some(Self::CreateUniversal());
        }
        let save = s.Save();
        let result = (|| {
            let mut out = Vec::new();
            loop {
                out.push(Self::ConsumeSyntaxComponent(s)?);
                if s.Peek().GetType() != kDelimiterToken || s.Peek().Delimiter() != 124 {
                    break;
                }
                s.ConsumeIncludingWhitespace();
            }
            Some(Self::new(out))
        })();
        if result.is_none() {
            s.Restore(save);
        }
        result
    }
    pub fn ConsumeComponent<T: TokenStreamTokenizer>(
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Self> {
        Some(Self::new(vec![Self::ConsumeSyntaxComponent(s)?]))
    }
    fn ConsumeSyntaxComponent<T: TokenStreamTokenizer>(
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<CSSSyntaxComponent> {
        s.EnsureLookAhead();
        let save = s.Save();
        let result = (|| {
            let (ty, ident) = if s.Peek().GetType() == kDelimiterToken && s.Peek().Delimiter() == 60
            {
                s.Consume();
                if s.Peek().GetType() != kIdentToken {
                    return None;
                }
                let ty = SyntaxType(&s.Peek().Value())?;
                s.Consume();
                if s.Peek().GetType() != kDelimiterToken || s.Peek().Delimiter() != 62 {
                    return None;
                }
                s.Consume();
                (ty, String::new())
            } else {
                if s.Peek().GetType() != kIdentToken {
                    return None;
                }
                let ident = s.Peek().Value().ToString();
                if !ValidSyntaxIdent(&ident) {
                    return None;
                }
                s.Consume();
                (CSSSyntaxType::kIdent, ident)
            };
            let repeat = if s.Peek().GetType() == kDelimiterToken {
                match s.Peek().Delimiter() {
                    35 => {
                        s.ConsumeIncludingWhitespace();
                        CSSSyntaxRepeat::kCommaSeparated
                    }
                    43 => {
                        s.ConsumeIncludingWhitespace();
                        CSSSyntaxRepeat::kSpaceSeparated
                    }
                    _ => CSSSyntaxRepeat::kNone,
                }
            } else {
                CSSSyntaxRepeat::kNone
            };
            s.ConsumeWhitespace();
            if ty == CSSSyntaxType::kTransformList && repeat != CSSSyntaxRepeat::kNone {
                return None;
            }
            Some(CSSSyntaxComponent::new(ty, &ident, repeat))
        })();
        if result.is_none() {
            s.Restore(save);
        }
        result
    }
    pub fn ParseTokens(
        &self,
        data: &values::CSSVariableData,
    ) -> Result<Rc<values::Value>, SyntaxValueError> {
        use crate::parser::css_parser_mode::CSSParserMode;
        if self.IsUniversal() {
            let visible = data
                .tokens
                .iter()
                .filter(|t| !matches!(t.token.GetType(), kWhitespaceToken | kCommentToken))
                .collect::<Vec<_>>();
            if visible.len() == 1
                && visible[0].token.GetType() == kIdentToken
                && values::wide(visible[0].token.Id()).is_some()
            {
                return Err(SyntaxValueError::Invalid);
            }
            return Ok(values::unparsed(
                data.clone(),
                CSSParserMode::kHTMLStandardMode,
            ));
        }
        let mut unsupported = None;
        for component in &self.syntax_components_ {
            let mut s = CSSParserTokenStream::FromTokenizer(VariableTokenReplay::new(data));
            s.ConsumeWhitespace();
            match ConsumeComponentValues(component, &mut s) {
                Ok(value) if s.AtEnd() => return Ok(value),
                Err(SyntaxValueError::Unsupported(ty)) => unsupported = Some(ty),
                _ => {}
            }
        }
        Err(unsupported
            .map(SyntaxValueError::Unsupported)
            .unwrap_or(SyntaxValueError::Invalid))
    }
}
fn ConsumeComponentValues<T: TokenStreamTokenizer>(
    syntax: &CSSSyntaxComponent,
    s: &mut CSSParserTokenStream<'_, T>,
) -> Result<Rc<values::Value>, SyntaxValueError> {
    match syntax.GetRepeat() {
        CSSSyntaxRepeat::kNone => ConsumeSingleType(syntax, s),
        repeat => {
            let mut items = Vec::new();
            loop {
                items.push(ConsumeSingleType(syntax, s)?);
                if repeat == CSSSyntaxRepeat::kSpaceSeparated {
                    if s.AtEnd() {
                        break;
                    }
                } else {
                    if s.Peek().GetType() != kCommaToken {
                        break;
                    }
                    s.ConsumeIncludingWhitespace();
                }
            }
            Ok(values::list(
                items,
                if repeat == CSSSyntaxRepeat::kSpaceSeparated {
                    values::ListSeparator::Space
                } else {
                    values::ListSeparator::Comma
                },
            ))
        }
    }
}
fn ConsumeSingleType<T: TokenStreamTokenizer>(
    syntax: &CSSSyntaxComponent,
    s: &mut CSSParserTokenStream<'_, T>,
) -> Result<Rc<values::Value>, SyntaxValueError> {
    use CSSSyntaxType::*;
    use SyntaxValueError::*;
    let ty = syntax.GetType();
    let t = s.Peek().clone();
    let value = match ty {
        kIdent => {
            if t.GetType() != kIdentToken || t.Value().ToString() != *syntax.GetString() {
                return Err(Invalid);
            }
            s.ConsumeIncludingWhitespace();
            return Ok(values::custom_ident(
                syntax.GetString(),
                CSSPropertyID::kInvalid,
            ));
        }
        kCustomIdent => {
            if t.GetType() != kIdentToken || !ValidSyntaxIdent(&t.Value().ToString()) {
                return Err(Invalid);
            }
            s.ConsumeIncludingWhitespace();
            return Ok(values::custom_ident(
                &t.Value().ToString(),
                CSSPropertyID::kInvalid,
            ));
        }
        kString => {
            if t.GetType() != kStringToken {
                return Err(Invalid);
            }
            s.ConsumeIncludingWhitespace();
            return Ok(values::string(t.Value().ToString()));
        }
        kColor => {
            return ConsumeColor(CSSPropertyID::kInitialValue, s).map_err(|error| {
                match error.kind {
                crate::parser::production_property_parser::PropertyParseErrorKind::Unsupported => {
                    Unsupported(ty)
                }
                _ => Invalid,
            }
            })
        }
        kImage | kUrl | kTransformFunction | kTransformList => return Err(Unsupported(ty)),
        kTokenStream => return Err(Invalid),
        _ => {
            if t.GetType() == kFunctionToken {
                return Err(Unsupported(ty));
            }
            let unit = match t.GetType() {
                kNumberToken => UnitType::kNumber,
                kPercentageToken => UnitType::kPercentage,
                kDimensionToken => t.GetUnitType(),
                _ => return Err(Invalid),
            };
            let number = t.NumericValue();
            let output_unit = match ty {
                kNumber if unit == UnitType::kNumber => UnitType::kNumber,
                kInteger
                    if unit == UnitType::kNumber
                        && t.GetNumericValueType() == NumericValueType::kIntegerValueType =>
                {
                    UnitType::kNumber
                }
                kPercentage if unit == UnitType::kPercentage => unit,
                kLength | kLengthPercentage if unit == UnitType::kNumber && number == 0.0 => {
                    UnitType::kPixels
                }
                kLength | kLengthPercentage
                    if IsLength(unit)
                        || ty == kLengthPercentage && unit == UnitType::kPercentage =>
                {
                    unit
                }
                kAngle
                    if matches!(
                        unit,
                        UnitType::kDegrees
                            | UnitType::kRadians
                            | UnitType::kGradians
                            | UnitType::kTurns
                    ) =>
                {
                    unit
                }
                kTime if matches!(unit, UnitType::kSeconds | UnitType::kMilliseconds) => unit,
                kResolution
                    if matches!(
                        unit,
                        UnitType::kDotsPerPixel
                            | UnitType::kDotsPerInch
                            | UnitType::kDotsPerCentimeter
                            | UnitType::kX
                    ) && number >= 0.0 =>
                {
                    unit
                }
                _ => return Err(Invalid),
            };
            values::numeric(number, output_unit)
        }
    };
    s.ConsumeIncludingWhitespace();
    Ok(value)
}
