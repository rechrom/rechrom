//! Source AtRuleDescriptorParser.cc:23-433 and css_parsing_utils.cc:6447-6700.
//! The single production font descriptor parser used by stylesheet rules and
//! imperative FontFace. Values remain the original compiled CSSValue owners.
use crate::{
    css_parser_context::{CSSParserContext, CSSParserContextPlatform, ParserModeOverridingScope},
    css_parser_mode::CSSParserMode,
    css_parser_token::CSSParserTokenType::*,
    css_parser_token_stream::{
        BlockGuard, CSSParserTokenStream, EnableUnicodeRanges, TokenStreamTokenizer,
    },
    css_value::CSSValuePayload,
    production_css_value::{self as values},
};
use foundation::{CSSPropertyID, CSSValueID, String, StringView};
use std::rc::Rc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    Invalid,
    Unsupported,
}
pub fn ParseFontFaceDescriptor<P: CSSParserContextPlatform<URL = values::ProductionURL>>(
    id: crate::at_rule_descriptors::AtRuleDescriptorID,
    text: &String,
    context: &CSSParserContext<P>,
) -> DescriptorResult {
    ParseFontFaceDescriptorWithDiagnostics(id, text, 0, context, &mut |_, _| {})
}
pub type DescriptorResult = Result<Rc<values::Value>, DiagnosticKind>;
pub fn ConsumeFamilyName<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
    descriptor: bool,
) -> Option<String> {
    if stream.Peek().GetType() == kStringToken {
        return Some(stream.ConsumeIncludingWhitespace().Value().ToString());
    }
    let first = stream.Peek().Value().ToString().Utf8().to_ascii_lowercase();
    if descriptor
        && [
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
        ]
        .contains(&first.as_str())
    {
        return None;
    }
    let mut words = Vec::new();
    while stream.Peek().GetType() == kIdentToken {
        words.push(
            stream
                .ConsumeIncludingWhitespace()
                .Value()
                .ToString()
                .Utf8(),
        );
    }
    if words.len() == 1
        && [
            "inherit",
            "initial",
            "unset",
            "revert",
            "revert-layer",
            "default",
        ]
        .contains(&first.as_str())
    {
        return None;
    }
    (!words.is_empty())
        .then(|| String::from_utf16(&words.join(" ").encode_utf16().collect::<Vec<_>>()))
}
fn ConsumeFontSource<
    T: TokenStreamTokenizer,
    P: CSSParserContextPlatform<URL = values::ProductionURL>,
>(
    stream: &mut CSSParserTokenStream<'_, T>,
    context: &CSSParserContext<P>,
) -> Result<values::CSSFontFaceSrcValue, DiagnosticKind> {
    use values::FontFaceResource;
    let resource = if stream.Peek().FunctionId() == Some(CSSValueID::kLocal) {
        let mut guard = BlockGuard::new(stream);
        guard.ConsumeWhitespace();
        let name = ConsumeFamilyName(&mut guard, false).ok_or(DiagnosticKind::Invalid)?;
        if !guard.AtEnd() {
            return Err(DiagnosticKind::Invalid);
        }
        FontFaceResource::Local(name)
    } else if stream.Peek().GetType() == kUrlToken {
        let url = stream.ConsumeIncludingWhitespace().Value().ToString();
        let url = crate::production_property_parser::RestrictFetchURL(url, context);
        FontFaceResource::Url(values::CSSUrlData::new(
            url.clone(),
            context.CompleteNonEmptyURL(&url),
        ))
    } else if stream.Peek().FunctionId() == Some(CSSValueID::kUrl) {
        let mut guard = BlockGuard::new(stream);
        guard.ConsumeWhitespace();
        if guard.Peek().GetType() != kStringToken {
            return Err(DiagnosticKind::Invalid);
        }
        let url = guard.ConsumeIncludingWhitespace().Value().ToString();
        let url = crate::production_property_parser::RestrictFetchURL(url, context);
        if !guard.AtEnd() {
            return Err(DiagnosticKind::Invalid);
        }
        FontFaceResource::Url(values::CSSUrlData::new(
            url.clone(),
            context.CompleteNonEmptyURL(&url),
        ))
    } else {
        return Err(DiagnosticKind::Invalid);
    };
    stream.ConsumeWhitespace();
    let mut value = values::CSSFontFaceSrcValue {
        resource,
        format: None,
        technologies: Vec::new(),
    };
    if matches!(value.resource, FontFaceResource::Url(_)) {
        if stream.Peek().FunctionId() == Some(CSSValueID::kFormat) {
            let mut guard = BlockGuard::new(stream);
            guard.ConsumeWhitespace();
            if !matches!(guard.Peek().GetType(), kIdentToken | kStringToken) {
                return Err(DiagnosticKind::Invalid);
            }
            let token = guard.ConsumeIncludingWhitespace();
            let format = token.Value().ToString();
            let keyword = format.Utf8().to_ascii_lowercase();
            let supported = ["collection", "opentype", "truetype", "woff", "woff2"]
                .contains(&keyword.as_str())
                || token.GetType() == kStringToken
                    && [
                        "woff-variations",
                        "truetype-variations",
                        "opentype-variations",
                        "woff2-variations",
                    ]
                    .contains(&keyword.as_str());
            if !supported || !guard.AtEnd() {
                return Err(DiagnosticKind::Invalid);
            }
            value.format = Some(format);
        }
        stream.ConsumeWhitespace();
        if stream.Peek().FunctionId() == Some(CSSValueID::kTech) {
            let mut guard = BlockGuard::new(stream);
            guard.ConsumeWhitespace();
            loop {
                let id = guard.Peek().Id();
                if !matches!(
                    id,
                    CSSValueID::kFeaturesOpentype
                        | CSSValueID::kFeaturesAat
                        | CSSValueID::kColorCOLRv0
                        | CSSValueID::kColorCOLRv1
                        | CSSValueID::kColorSbix
                        | CSSValueID::kColorCBDT
                        | CSSValueID::kVariations
                        | CSSValueID::kPalettes
                        | CSSValueID::kAvar2
                ) {
                    return Err(DiagnosticKind::Invalid);
                }
                guard.ConsumeIncludingWhitespace();
                if !value.technologies.contains(&id) {
                    value.technologies.push(id);
                }
                if guard.AtEnd() {
                    break;
                }
                if guard.Peek().GetType() != kCommaToken {
                    return Err(DiagnosticKind::Invalid);
                }
                guard.ConsumeIncludingWhitespace();
            }
        }
    }
    stream.ConsumeWhitespace();
    if !stream.AtEnd() && stream.Peek().GetType() != kCommaToken {
        return Err(DiagnosticKind::Invalid);
    }
    Ok(value)
}
pub fn ParseFontFaceDescriptorWithDiagnostics<
    P: CSSParserContextPlatform<URL = values::ProductionURL>,
>(
    id: crate::at_rule_descriptors::AtRuleDescriptorID,
    text: &String,
    source_offset: u32,
    context: &CSSParserContext<P>,
    diagnostic: &mut dyn FnMut(DiagnosticKind, u32),
) -> DescriptorResult {
    use crate::at_rule_descriptors::AtRuleDescriptorID as D;
    use crate::css_primitive_value::UnitType;
    let _mode = ParserModeOverridingScope::new(context, CSSParserMode::kCSSFontFaceRuleMode);
    let mut stream: CSSParserTokenStream<crate::css_tokenizer::CSSTokenizer> =
        CSSParserTokenStream::new(StringView::from(text), 0);
    let mut stream = EnableUnicodeRanges::new(&mut stream, id == D::UnicodeRange);
    stream.ConsumeWhitespace();
    let value = match id {
        D::FontFamily => {
            let family = ConsumeFamilyName(&mut stream, true).ok_or(DiagnosticKind::Invalid)?;
            Rc::new(values::Value::new(CSSValuePayload::kFontFamilyClass(
                values::CSSFontFamilyValue(family),
            )))
        }
        D::Src => {
            let mut sources = Vec::new();
            loop {
                let offset = source_offset + stream.LookAheadOffset();
                match ConsumeFontSource(&mut stream, context) {
                    Ok(value) => sources.push(Rc::new(values::Value::new(
                        CSSValuePayload::kFontFaceSrcClass(value),
                    ))),
                    Err(kind) => {
                        diagnostic(kind, offset);
                        stream.SkipUntilPeekedTypeIs(&[kCommaToken]);
                    }
                }
                if stream.Peek().GetType() != kCommaToken {
                    break;
                }
                stream.ConsumeIncludingWhitespace();
            }
            if sources.is_empty() {
                return Err(DiagnosticKind::Invalid);
            }
            values::list(sources, values::ListSeparator::Comma)
        }
        D::UnicodeRange => {
            let mut ranges = Vec::new();
            loop {
                if stream.Peek().GetType() != kUnicodeRangeToken {
                    return Err(DiagnosticKind::Invalid);
                }
                let token = stream.ConsumeIncludingWhitespace();
                let from = token.UnicodeRangeStart();
                let to = token.UnicodeRangeEnd();
                if from < 0 || from > to || to > 0x10ffff {
                    return Err(DiagnosticKind::Invalid);
                }
                ranges.push(Rc::new(values::Value::new(
                    CSSValuePayload::kUnicodeRangeClass(values::CSSUnicodeRangeValue {
                        from: from as u32,
                        to: to as u32,
                    }),
                )));
                if stream.Peek().GetType() != kCommaToken {
                    break;
                }
                stream.ConsumeIncludingWhitespace();
            }
            values::list(ranges, values::ListSeparator::Comma)
        }
        D::FontDisplay => {
            let keyword = stream.Peek().Id();
            if !matches!(
                keyword,
                CSSValueID::kAuto
                    | CSSValueID::kBlock
                    | CSSValueID::kSwap
                    | CSSValueID::kFallback
                    | CSSValueID::kOptional
            ) {
                return Err(DiagnosticKind::Invalid);
            }
            stream.ConsumeIncludingWhitespace();
            values::identifier(keyword)
        }
        D::FontWeight | D::FontStretch | D::FontStyle => ConsumeFontRange(id, &mut stream)?,
        D::AscentOverride | D::DescentOverride | D::LineGapOverride | D::SizeAdjust => {
            if id != D::SizeAdjust && stream.Peek().Id() == CSSValueID::kNormal {
                stream.ConsumeIncludingWhitespace();
                values::identifier(CSSValueID::kNormal)
            } else {
                ConsumeNumeric(id, &mut stream)?
            }
        }
        D::FontVariant => {
            let mut variants = Vec::new();
            loop {
                let id = stream.Peek().Id();
                if !matches!(
                    id,
                    CSSValueID::kNormal | CSSValueID::kSmallCaps | CSSValueID::kAll
                ) {
                    return Err(DiagnosticKind::Invalid);
                }
                stream.ConsumeIncludingWhitespace();
                if id == CSSValueID::kAll {
                    if !variants.is_empty() {
                        return Err(DiagnosticKind::Invalid);
                    }
                    return if stream.AtEnd() {
                        Ok(values::identifier(id))
                    } else {
                        Err(DiagnosticKind::Invalid)
                    };
                }
                variants.push(values::identifier(id));
                if stream.Peek().GetType() != kCommaToken {
                    break;
                }
                stream.ConsumeIncludingWhitespace();
            }
            values::list(variants, values::ListSeparator::Comma)
        }
        D::FontFeatureSettings | D::FontVariationSettings => {
            // FontVariationSettingsDescriptor is stable at the pinned revision
            // (runtime_enabled_features.json5:3231-3233).
            let property = if id == D::FontFeatureSettings {
                CSSPropertyID::kFontFeatureSettings
            } else {
                CSSPropertyID::kFontVariationSettings
            };
            let parsed = crate::production_property_parser::ParsePropertyWithContext(
                property, text, false, context,
            )
            .map_err(|error| match error.kind {
                crate::production_property_parser::PropertyParseErrorKind::Unsupported => {
                    DiagnosticKind::Unsupported
                }
                _ => DiagnosticKind::Invalid,
            })?;
            let value = parsed.first().ok_or(DiagnosticKind::Invalid)?;
            if value.IsImportant() || value.Value().IsCSSWideKeyword() {
                return Err(DiagnosticKind::Invalid);
            }
            stream.SkipUntilPeekedTypeIs(&[]);
            value.ValueRef()
        }
        _ => return Err(DiagnosticKind::Invalid),
    };
    if stream.AtEnd() {
        Ok(value)
    } else {
        Err(DiagnosticKind::Invalid)
    }
}

fn ConsumeNumeric<T: TokenStreamTokenizer>(
    id: crate::at_rule_descriptors::AtRuleDescriptorID,
    stream: &mut CSSParserTokenStream<'_, T>,
) -> DescriptorResult {
    use crate::{
        at_rule_descriptors::{AtRuleDescriptorID as D, AtRuleDescriptorIDAsCSSPropertyID},
        css_math_expression_node::CalculationResultCategory as C,
        css_math_function_value::ValueRange as R,
        css_primitive_value::UnitType as U,
    };
    let (category, range) = match id {
        D::FontWeight => (C::Number, R::NonNegative),
        D::FontStyle => (C::Angle, R::All),
        _ => (C::Percent, R::NonNegative),
    };
    if stream.Peek().GetType() == kFunctionToken {
        return crate::production_property_parser::ConsumeMath(
            AtRuleDescriptorIDAsCSSPropertyID(id),
            stream,
            &[category],
            range,
        )
        .map_err(|error| match error.kind {
            crate::production_property_parser::PropertyParseErrorKind::Unsupported => {
                DiagnosticKind::Unsupported
            }
            _ => DiagnosticKind::Invalid,
        });
    }
    let token = stream.Peek().clone();
    if !matches!(
        token.GetType(),
        kNumberToken | kPercentageToken | kDimensionToken
    ) {
        return Err(DiagnosticKind::Invalid);
    }
    let unit = token.GetUnitType();
    let number = token.NumericValue();
    let valid = match id {
        D::FontWeight => token.GetType() == kNumberToken && (1.0..=1000.0).contains(&number),
        D::FontStyle => {
            token.GetType() == kDimensionToken
                && matches!(unit, U::kDegrees | U::kRadians | U::kGradians | U::kTurns)
                && (-90.0..=90.0).contains(&number)
        }
        _ => token.GetType() == kPercentageToken && number >= 0.0,
    };
    if !valid {
        return Err(DiagnosticKind::Invalid);
    }
    stream.ConsumeIncludingWhitespace();
    Ok(values::numeric(
        number,
        if id == D::FontWeight {
            U::kNumber
        } else {
            unit
        },
    ))
}
fn ConsumeFontRange<T: TokenStreamTokenizer>(
    id: crate::at_rule_descriptors::AtRuleDescriptorID,
    stream: &mut CSSParserTokenStream<'_, T>,
) -> DescriptorResult {
    use crate::at_rule_descriptors::AtRuleDescriptorID as D;
    let keyword = stream.Peek().Id();
    let keyword_only = match id {
        D::FontWeight => matches!(
            keyword,
            CSSValueID::kNormal | CSSValueID::kBold | CSSValueID::kAuto
        ),
        D::FontStretch => {
            matches!(keyword, CSSValueID::kNormal | CSSValueID::kAuto)
                || (CSSValueID::kUltraCondensed as i32..=CSSValueID::kUltraExpanded as i32)
                    .contains(&(keyword as i32))
        }
        D::FontStyle => matches!(
            keyword,
            CSSValueID::kNormal | CSSValueID::kItalic | CSSValueID::kAuto
        ),
        _ => unreachable!(),
    };
    if keyword_only {
        stream.ConsumeIncludingWhitespace();
        return Ok(values::identifier(keyword));
    }
    if id == D::FontStyle {
        if keyword != CSSValueID::kOblique {
            return Err(DiagnosticKind::Invalid);
        }
        stream.ConsumeIncludingWhitespace();
        if stream.AtEnd() {
            return Ok(values::identifier(keyword));
        }
    }
    let start = ConsumeNumeric(id, stream)?;
    let mut range = vec![start];
    if !stream.AtEnd() {
        range.push(ConsumeNumeric(id, stream)?);
    }
    if id == D::FontStyle {
        // Stable FontStyleObliqueZeroDegreeAsNormal; math is unresolved here.
        if range.iter().all(|v| matches!(v.Payload(),CSSValuePayload::kNumericLiteralClass(n) if n.DoubleValue()==0.0)) { return Ok(values::identifier(CSSValueID::kNormal)); }
        let angles = values::list(range, values::ListSeparator::Space);
        return Ok(Rc::new(values::Value::new(
            CSSValuePayload::kFontStyleRangeClass(values::CSSFontStyleRangeValue {
                angle: Some(angles),
            }),
        )));
    }
    Ok(if range.len() == 1 {
        range.remove(0)
    } else {
        values::list(range, values::ListSeparator::Space)
    })
}

/// FontFace's source SetPropertyFromStyle copies references to these exact
/// CSSValue owners; neither CSSOM projection nor loading reparses CssText.
#[derive(Clone)]
pub struct CompiledFontFaceDescriptors(
    pub  Vec<(
        crate::at_rule_descriptors::AtRuleDescriptorID,
        Rc<values::Value>,
    )>,
);
impl CompiledFontFaceDescriptors {
    pub fn Get(
        &self,
        id: crate::at_rule_descriptors::AtRuleDescriptorID,
    ) -> Option<Rc<values::Value>> {
        self.0
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value.clone())
    }
}
