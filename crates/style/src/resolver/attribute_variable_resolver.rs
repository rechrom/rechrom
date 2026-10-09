// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Typed literal/raw-string branches of CSSAttrType and attribute lookup.
#![allow(non_snake_case)]
use super::{CSSVariableData, VariableResolutionError, VariableToken};
use crate::{
    css_primitive_value::{StringToUnitType, UnitType, UnitTypeToString},
    parser::{
        css_parser_token::{
            BlockType, CSSParserToken, CSSParserTokenType::*, NumericSign, NumericValueType,
        },
        css_tokenizer::CSSTokenizer,
    },
    production_css_value as values,
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
}
pub trait AttributeVariableResolver {
    fn CSSArgumentGrammarEnabled(&self) -> bool;
    fn SetHasAttrFunction(&self);
    fn GetAttribute(&self, local_name: &AtomicString) -> String;
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
}
/// Uses the admitted DOM owner directly. Attribute namespaces other than null
/// are excluded, matching getAttributeNS(g_null_atom, ...) in StyleCascade.
pub struct PersistentAttributeSource<'a> {
    document: &'a dom::Document,
    ultimate_originating_element: usize,
    lowercase_names: bool,
}
impl<'a> PersistentAttributeSource<'a> {
    pub fn new(
        document: &'a dom::Document,
        ultimate_originating_element: usize,
        lowercase_names: bool,
    ) -> Self {
        assert_eq!(
            document.Node(ultimate_originating_element).Type(),
            dom::persistent_document::DOMNodeType::kElement
        );
        Self {
            document,
            ultimate_originating_element,
            lowercase_names,
        }
    }
}
impl AttributeSource for PersistentAttributeSource<'_> {
    fn GetAttribute(&self, local_name: &AtomicString) -> String {
        let name = if self.lowercase_names {
            local_name.Utf8().to_ascii_lowercase()
        } else {
            local_name.Utf8()
        };
        self.document
            .Node(self.ultimate_originating_element)
            .Attributes()
            .iter()
            .find(|attribute| attribute.namespace_uri.is_empty() && attribute.local_name == name)
            .map_or_else(String::default, |attribute| {
                String::from(attribute.value.as_str())
            })
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum AttributeType {
    RawString,
    Unit(UnitType),
}
// cpp: css_attr_type.cc:23-79 ConsumeDimensionUnitType/CSSAttrType::Consume;
// css_variable_parser.cc:255-293 ConsumeAttributeReference.
pub(crate) fn ConsumeAttributeHeader(
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
pub(crate) fn ParseAttributeValue(
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
pub(crate) fn StringData(value: &String, implicit_fallback: bool) -> Rc<CSSVariableData> {
    let text = if implicit_fallback {
        String::from("''")
    } else {
        crate::css_markup::SerializeString(value)
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

#[cfg(test)]
mod tests {
    use super::super::{CustomProperties, ResolvePropertyValueWithContext, SubstitutionContext};
    use super::*;
    use crate::{
        css_value::CSSValuePayload,
        parser::{
            css_parser_mode::CSSParserMode,
            production_property_parser::{
                ParseCustomProperty, ParseDeclarationList, ParseProperty,
            },
        },
    };
    use foundation::CSSPropertyID;
    use std::{cell::Cell, collections::HashMap};
    #[derive(Default)]
    struct Source(HashMap<AtomicString, String>);
    impl Source {
        fn new(entries: &[(&str, &str)]) -> Self {
            Self(
                entries
                    .iter()
                    .map(|(name, value)| (AtomicString::from_str(name), String::from(*value)))
                    .collect(),
            )
        }
    }
    impl AttributeSource for Source {
        fn GetAttribute(&self, name: &AtomicString) -> String {
            self.0.get(name).cloned().unwrap_or_default()
        }
    }
    #[derive(Default)]
    struct State(Cell<usize>);
    impl AttributeResolutionState for State {
        fn SetHasAttrFunction(&self) {
            self.0.set(self.0.get() + 1);
        }
    }
    fn data(css: &str) -> Rc<CSSVariableData> {
        let property = ParseCustomProperty(
            "--fixture",
            &String::from(css),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        let CSSValuePayload::kUnparsedDeclarationClass(declaration) = property.Value().Payload()
        else {
            panic!("token declaration")
        };
        declaration.data.clone()
    }
    fn substitute(
        css: &str,
        source: &Source,
        grammar: bool,
    ) -> Result<CSSVariableData, VariableResolutionError> {
        let state = State::default();
        let attributes = AttributeResolver::new(source, &state, grammar);
        let result = CustomProperties::default().SubstituteWithContext(
            &data(css),
            SubstitutionContext {
                attributes: Some(&attributes),
                environment: None,
            },
        );
        assert!(state.0.get() > 0);
        result
    }
    fn resolve(id: CSSPropertyID, css: &str, source: &Source) -> Rc<values::Value> {
        let state = State::default();
        let attributes = AttributeResolver::new(source, &state, false);
        let parsed = ParseProperty(
            id,
            &String::from(css),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        ResolvePropertyValueWithContext(
            id,
            &parsed[0].ValueRef(),
            &CustomProperties::default(),
            SubstitutionContext {
                attributes: Some(&attributes),
                environment: None,
            },
        )
        .unwrap()
    }
    #[test]
    fn literal_units_replay_into_native_property_consumers() {
        let source = Source::new(&[("size", "12"), ("fraction", ".25"), ("percent", "50")]);
        for (id, css, expected) in [
            (CSSPropertyID::kWidth, "attr(size px)", "12px"),
            (CSSPropertyID::kOpacity, "attr(fraction number)", "0.25"),
            (CSSPropertyID::kWidth, "attr(percent %)", "50%"),
        ] {
            assert_eq!(resolve(id, css, &source).CssText().Utf8(), expected);
        }
        assert_eq!(
            substitute("attr(size fr)", &source, false).unwrap().tokens[0]
                .token
                .GetUnitType(),
            UnitType::kFlex
        );
        assert_eq!(
            substitute("attr(size __qem)", &source, false)
                .unwrap()
                .tokens[0]
                .token
                .GetUnitType(),
            UnitType::kEms
        );
    }
    #[test]
    fn raw_string_preserves_utf16_and_does_not_resolve_embedded_functions() {
        let raw = String::from_utf16(&[
            0x76, 0x61, 0x72, 0x28, 0x2d, 0x2d, 0x78, 0x29, 0xd800, 0x22, 0x5c,
        ]);
        let source = Source(
            [(AtomicString::from_str("label"), raw.clone())]
                .into_iter()
                .collect(),
        );
        let result = substitute("attr(label)", &source, false).unwrap();
        assert_eq!(result.tokens.len(), 1);
        assert_eq!(result.tokens[0].token.GetType(), kStringToken);
        assert_eq!(result.tokens[0].token.Value().ToString(), raw);
        assert!(!result.NeedsVariableResolution());
        assert!(result.is_attr_tainted);
    }
    #[test]
    fn null_empty_and_explicit_type_have_distinct_fallbacks() {
        let source = Source::new(&[("empty", "")]);
        assert_eq!(
            substitute("attr(missing)", &source, false)
                .unwrap()
                .original_text
                .Utf8(),
            "''"
        );
        let present = substitute("attr(empty raw-string,wrong)", &source, false).unwrap();
        assert_eq!(present.tokens[0].token.Value().ToString(), String::from(""));
        assert!(matches!(
            substitute("attr(missing raw-string)", &source, false),
            Err(VariableResolutionError::Invalid)
        ));
        let empty = substitute("attr(missing px,)", &source, false).unwrap();
        assert!(empty.tokens.is_empty());
        assert!(empty.is_attr_tainted);
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "attr(missing px,7px)", &source)
                .CssText()
                .Utf8(),
            "7px"
        );
    }
    #[test]
    fn numeric_attribute_uses_literal_dynamic_type_and_first_number() {
        let source = Source::new(&[
            ("dimension", "12px"),
            ("math", "calc(5)"),
            ("trailing", "12 trailing"),
            ("comment", "/*x*/-0"),
            ("leading", " 12"),
        ]);
        for css in [
            "attr(dimension px,7px)",
            "attr(math px,7px)",
            "attr(trailing unknown,7px)",
            "attr(leading px,7px)",
        ] {
            assert_eq!(
                resolve(CSSPropertyID::kWidth, css, &source)
                    .CssText()
                    .Utf8(),
                "7px"
            );
        }
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "attr(trailing px)", &source)
                .CssText()
                .Utf8(),
            "12px"
        );
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "attr(comment px)", &source)
                .CssText()
                .Utf8(),
            "0px"
        );
    }
    #[test]
    fn substitution_keeps_token_boundaries_and_taints_fallback() {
        let source = Source::new(&[("size", "12")]);
        assert_eq!(
            substitute("attr(size number)px", &source, false)
                .unwrap()
                .original_text
                .Utf8(),
            "12/**/px"
        );
        assert!(resolve(CSSPropertyID::kWidth, "attr(size number)px", &source).IsUnsetValue());
        let fallback = substitute("attr(missing px,8px)", &source, false).unwrap();
        assert!(fallback.is_attr_tainted);
    }
    #[test]
    fn argument_grammar_runtime_gate_substitutes_typed_header() {
        use crate::parser::production_property_parser::ParseCustomPropertyWithArgumentGrammar;
        let source = Source::new(&[("size", "23")]);
        let css = String::from("attr(var(--missing,size) px)");
        assert!(
            ParseCustomProperty("--fixture", &css, false, CSSParserMode::kHTMLStandardMode)
                .is_err()
        );
        let parsed = ParseCustomPropertyWithArgumentGrammar(
            "--fixture",
            &css,
            false,
            CSSParserMode::kHTMLStandardMode,
            true,
        )
        .unwrap();
        let CSSValuePayload::kUnparsedDeclarationClass(declaration) = parsed.Value().Payload()
        else {
            panic!("argument grammar declaration");
        };
        let substitute = |enabled| {
            let state = State::default();
            let attributes = AttributeResolver::new(&source, &state, enabled);
            CustomProperties::default().SubstituteWithContext(
                &declaration.data,
                SubstitutionContext {
                    attributes: Some(&attributes),
                    environment: None,
                },
            )
        };
        assert!(matches!(
            substitute(false),
            Err(VariableResolutionError::Invalid)
        ));
        assert_eq!(substitute(true).unwrap().original_text.Utf8(), "23px");
    }
    #[test]
    fn attribute_cycles_in_fallback_invalidate_custom_cycle_members() {
        let source = Source::default();
        assert!(matches!(
            substitute("attr(missing px,attr(missing px,7px))", &source, false),
            Err(VariableResolutionError::Cyclic)
        ));
        let parsed = ParseDeclarationList(
            &String::from(
                "--a:attr(missing px,var(--b));--b:attr(missing px,7px);--c:var(--a,9px);",
            ),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.errors.is_empty());
        let state = State::default();
        let attributes = AttributeResolver::new(&source, &state, false);
        let custom = CustomProperties::ComputeWithContext(
            None,
            parsed.properties.iter(),
            SubstitutionContext {
                attributes: Some(&attributes),
                environment: None,
            },
        );
        assert!(custom.Get("--a").is_none());
        assert!(custom.Get("--b").is_none());
        assert_eq!(custom.Get("--c").unwrap().original_text.Utf8(), "9px");
    }
    #[test]
    fn missing_owners_and_syntax_definition_stay_typed_unsupported() {
        assert!(matches!(
            CustomProperties::default().Substitute(&data("attr(size px,7px)")),
            Err(VariableResolutionError::Unsupported(
                "StyleCascade::ResolveAttrInto attribute owner/state"
            ))
        ));
        assert!(matches!(
            substitute(
                "attr(size type(<length>),7px)",
                &Source::new(&[("size", "12px")]),
                false
            ),
            Err(VariableResolutionError::Unsupported(
                "CSSAttrType::Consume CSSSyntaxDefinition::Consume/Parse"
            ))
        ));
    }
    #[test]
    fn persistent_dom_lookup_preserves_namespace_and_case_policy() {
        use dom::persistent_document::{DOMAttribute, DOMNamespace};
        let mut owner = dom::DOM::new();
        let d = owner.GetDocumentMut();
        let index = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        d.SetAttribute(
            index,
            DOMAttribute {
                local_name: "data-size".into(),
                value: "17".into(),
                ..Default::default()
            },
        );
        d.SetAttribute(
            index,
            DOMAttribute {
                local_name: "foreign".into(),
                namespace_uri: "urn:test".into(),
                value: "wrong".into(),
                ..Default::default()
            },
        );
        let html = PersistentAttributeSource::new(d, index, true);
        assert_eq!(
            html.GetAttribute(&AtomicString::from_str("DATA-SIZE")),
            String::from("17")
        );
        assert!(html
            .GetAttribute(&AtomicString::from_str("foreign"))
            .IsNull());
        let exact = PersistentAttributeSource::new(d, index, false);
        assert!(exact
            .GetAttribute(&AtomicString::from_str("DATA-SIZE"))
            .IsNull());
    }
}
