// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! SVGLength::SetValueAsString uses CSS length token grammar and retains the
//! resulting CSSPrimitiveValue. Source: svg_length.cc:81-108,214-278.
#![allow(non_snake_case)]
use crate::{
    css_math_expression_node::{CalculationResultCategory, ConsumeMathFunction},
    css_math_function_value::ValueRange,
    css_numeric_literal_value::{CSSNumericLiteralValue, IsLength},
    css_parser_token::CSSParserTokenType::*,
    css_parser_token_stream::CSSParserTokenStream,
    css_primitive_value::UnitType,
    css_tokenizer::CSSTokenizer,
};
use crate::{
    css_parser_mode::CSSParserMode,
    css_value::CSSValuePayload,
    production_css_value::{CSSVariableData, ProductionURL, PropertyValue, Value},
    production_property_parser::{
        ParseProperty, ParsePropertyTokensWithBase, VariableArguments, VariableBlockEnd,
    },
};
use layoutng_style::css::svg_length_value::SVGMarkerLengthValues;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, OnceLock},
};

pub fn ParseSVGLength(input: &str) -> Option<CSSNumericLiteralValue> {
    let mut stream =
        CSSParserTokenStream::<CSSTokenizer>::new(foundation::StringView::from(input), 0);
    stream.ConsumeWhitespace();
    let token = stream.Peek().clone();
    let value = match token.GetType() {
        kFunctionToken => {
            let expression = ConsumeMathFunction(&mut stream).ok()?;
            if !matches!(
                expression.Category(),
                CalculationResultCategory::Number
                    | CalculationResultCategory::Length
                    | CalculationResultCategory::Percent
                    | CalculationResultCategory::LengthFunction
            ) {
                return None;
            }
            CSSNumericLiteralValue::CreateMath(expression, ValueRange::All)
        }
        kNumberToken | kPercentageToken | kDimensionToken => {
            let unit = match token.GetType() {
                kNumberToken => UnitType::kNumber,
                kPercentageToken => UnitType::kPercentage,
                _ => token.GetUnitType(),
            };
            if !(IsLength(unit) || matches!(unit, UnitType::kNumber | UnitType::kPercentage))
                || unit == UnitType::kQuirkyEms
            {
                return None;
            }
            if !token.NumericValue().is_finite() {
                return None;
            }
            stream.ConsumeIncludingWhitespace();
            CSSNumericLiteralValue::Create(token.NumericValue(), unit)
        }
        _ => return None,
    };
    stream.AtEnd().then_some(value)
}

/// SVGLength retains the same specified CSSVariableData owner as ordinary
/// declarations. Only the used numeric result crosses into native Layout.
#[derive(Debug)]
pub enum SVGLengthInput {
    Numeric(Arc<CSSNumericLiteralValue>),
    Unparsed(SVGUnparsedLength),
}
#[derive(Debug)]
pub struct SVGUnparsedLength {
    pub data: Rc<CSSVariableData>,
    resolved: RefCell<Option<ResolvedLength>>,
}
#[derive(Debug)]
struct ResolvedLength {
    data: CSSVariableData,
    mode: CSSParserMode,
    value: Arc<CSSNumericLiteralValue>,
}
fn ZeroLength() -> Arc<CSSNumericLiteralValue> {
    static ZERO: OnceLock<Arc<CSSNumericLiteralValue>> = OnceLock::new();
    Arc::clone(
        ZERO.get_or_init(|| Arc::new(CSSNumericLiteralValue::Create(0., UnitType::kUserUnits))),
    )
}
fn TakePropertyValue(properties: Vec<PropertyValue>) -> Option<Value> {
    if properties.len() != 1 || properties[0].IsImportant() {
        return None;
    }
    let value = properties[0].ValueRef();
    drop(properties);
    Rc::try_unwrap(value).ok()
}
fn NumericPayload(value: Value) -> Option<Arc<CSSNumericLiteralValue>> {
    let value = match value.IntoPayload() {
        CSSValuePayload::kNumericLiteralClass(value) => value,
        CSSValuePayload::kMathFunctionClass(value) => CSSNumericLiteralValue::FromMath(value),
        _ => return None,
    };
    let supported = match value.MathValue() {
        Some(math) => matches!(
            math.Category(),
            CalculationResultCategory::Number
                | CalculationResultCategory::Length
                | CalculationResultCategory::Percent
                | CalculationResultCategory::LengthFunction
        ),
        None => {
            (IsLength(value.GetType())
                || matches!(value.GetType(), UnitType::kNumber | UnitType::kPercentage))
                && value.GetType() != UnitType::kQuirkyEms
        }
    };
    supported.then(|| Arc::new(value))
}
impl SVGUnparsedLength {
    pub fn Unresolved(&self) -> Arc<CSSNumericLiteralValue> {
        ZeroLength()
    }

    // svg_length_context.cc:55-94. Replay the already substituted token owner
    // through the document parser mode, never serialize/re-tokenize in Layout.
    pub fn ResolveTokens(
        &self,
        data: CSSVariableData,
        mode: CSSParserMode,
    ) -> Arc<CSSNumericLiteralValue> {
        let mut retained = self.resolved.borrow_mut();
        if let Some(old) = retained.as_ref().filter(|old| {
            old.mode == mode
                && old.data.is_animation_tainted == data.is_animation_tainted
                && old.data.is_attr_tainted == data.is_attr_tainted
                && old.data.tokens.len() == data.tokens.len()
                && old
                    .data
                    .tokens
                    .iter()
                    .zip(&data.tokens)
                    .all(|(a, b)| a.token == b.token && a.text == b.text)
        }) {
            return Arc::clone(&old.value);
        }
        let value = ParsePropertyTokensWithBase(
            foundation::CSSPropertyID::kX,
            &data,
            mode,
            ProductionURL::Null,
        )
        .ok()
        .and_then(TakePropertyValue)
        .and_then(NumericPayload)
        .unwrap_or_else(ZeroLength);
        *retained = Some(ResolvedLength {
            data,
            mode,
            value: Arc::clone(&value),
        });
        value
    }
}
impl SVGLengthInput {
    fn Parse(source: Option<String>, initial: f64, nonnegative: bool) -> Self {
        let source = source.as_deref();
        let numeric = source.and_then(ParseSVGLength);
        if let Some(value) = numeric {
            if !nonnegative || value.MathValue().is_some() || value.DoubleValue() >= 0. {
                return Self::Numeric(Arc::new(value));
            }
        } else if let Some(source) = source {
            let value = ParseProperty(
                foundation::CSSPropertyID::kX,
                &foundation::String::from(source),
                false,
                CSSParserMode::kSVGAttributeMode,
            )
            .ok()
            .and_then(TakePropertyValue);
            if let Some(CSSValuePayload::kUnparsedDeclarationClass(value)) =
                value.map(Value::IntoPayload)
            {
                if value.data.NeedsVariableResolution() {
                    return Self::Unparsed(SVGUnparsedLength {
                        data: value.data,
                        resolved: RefCell::new(None),
                    });
                }
            }
        }
        Self::Numeric(Arc::new(CSSNumericLiteralValue::Create(
            initial,
            UnitType::kUserUnits,
        )))
    }
    fn Fallback(&self) -> Arc<CSSNumericLiteralValue> {
        match self {
            Self::Numeric(value) => Arc::clone(value),
            Self::Unparsed(_) => ZeroLength(),
        }
    }
}
#[derive(Debug)]
pub struct SVGMarkerLengthInputs {
    pub width: SVGLengthInput,
    pub height: SVGLengthInput,
    pub ref_x: SVGLengthInput,
    pub ref_y: SVGLengthInput,
    fallback: Arc<SVGMarkerLengthValues>,
}
impl SVGMarkerLengthInputs {
    pub fn Parse(mut attribute: impl FnMut(&str) -> Option<String>) -> Self {
        let width = SVGLengthInput::Parse(attribute("markerWidth"), 3., true);
        let height = SVGLengthInput::Parse(attribute("markerHeight"), 3., true);
        let ref_x = SVGLengthInput::Parse(attribute("refX"), 0., false);
        let ref_y = SVGLengthInput::Parse(attribute("refY"), 0., false);
        let fallback = Arc::new(SVGMarkerLengthValues {
            width: width.Fallback(),
            height: height.Fallback(),
            ref_x: ref_x.Fallback(),
            ref_y: ref_y.Fallback(),
        });
        Self {
            width,
            height,
            ref_x,
            ref_y,
            fallback,
        }
    }
    pub fn Fallback(&self) -> Arc<SVGMarkerLengthValues> {
        Arc::clone(&self.fallback)
    }
    pub fn HasUnparsed(&self) -> bool {
        self.Values()
            .into_iter()
            .any(|value| matches!(value, SVGLengthInput::Unparsed(_)))
    }
    pub fn Values(&self) -> [&SVGLengthInput; 4] {
        [&self.width, &self.height, &self.ref_x, &self.ref_y]
    }
    /// Source StyleCascade::ResolveTokensInto marks this dependency at attr().
    /// Retain it on the authored token owner so element recalc can attach the
    /// canonical HasAttrFunction bit before publishing its ComputedStyle.
    pub fn HasAttributeReference(&self) -> bool {
        self.Values().into_iter().any(|value| match value {
            SVGLengthInput::Numeric(_) => false,
            SVGLengthInput::Unparsed(value) => value.data.tokens.iter().any(|item| {
                item.token.GetType() == kFunctionToken
                    && item.token.FunctionId() == Some(foundation::CSSValueID::kAttr)
            }),
        })
    }
    pub fn VisitReferences(
        &self,
        mut variable: impl FnMut(String),
        mut environment: impl FnMut(String),
    ) {
        for value in self.Values() {
            let SVGLengthInput::Unparsed(value) = value else {
                continue;
            };
            let tokens = &value.data.tokens;
            for (index, item) in tokens.iter().enumerate() {
                if item.token.GetType() != kFunctionToken {
                    continue;
                }
                let end = VariableBlockEnd(tokens, index);
                if item.token.FunctionId() == Some(foundation::CSSValueID::kVar) {
                    if let Some((name, _)) = VariableArguments(&tokens[index + 1..end]) {
                        variable(name);
                    }
                } else if item.token.FunctionId() == Some(foundation::CSSValueID::kEnv) {
                    if let Some(name) = tokens[index + 1..end].iter().find(|item| {
                        !matches!(item.token.GetType(), kWhitespaceToken | kCommentToken)
                    }) {
                        environment(name.token.Value().ToString().Utf8());
                    }
                }
            }
        }
    }
}
