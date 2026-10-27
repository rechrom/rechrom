// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! CSSOM storage projection of the production parser's typed stylesheet.
//!
//! This module never parses a stylesheet or declaration list. Observer-owned
//! selector/declaration source stays authoritative; CSSOM's declaration vector
//! is a view of accepted, serialized properties, including expanded longhands.
//! Font-face and keyframes records use the same observer ranges and typed
//! property stores; no projection rediscovers syntax from the author source.
#![allow(non_snake_case)]

use crate::{
    css_property_value_set::{CSSPropertyValueSet, CSSPropertyValueSetRuleHandle},
    production_css_value::ProductionCSSValueDispatch,
    production_style_sheet::{
        Backend, LayerSegment, ProductionFontFaceRule, ProductionKeyframeRule,
        ProductionKeyframesRule, ProductionNestedDeclarationsRule, ProductionProjectionRule,
        ProductionStyleRule, ProductionStyleSheet,
    },
    style_rule::{LayerNameAsString, StyleRuleBase},
};
use cssom::{
    CSSDeclaration, CSSFontFaceRule, CSSKeyframeRule, CSSKeyframesRule, CSSStyleRule, CSSStyleSheet,
};
use std::rc::Rc;

/// Parse author CSS through the translated Blink parser and project the
/// resulting typed rule ownership into the DOM's retained stylesheet storage.
/// UTF-8 decoding happens exactly once at this public boundary.
pub fn ParseCSS(text: &str) -> CSSStyleSheet {
    ParseCSSWithMode(
        text,
        crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
    )
}

/// Parse an author sheet with its response/document URL installed on the
/// production CSSParserContext. Typed URL values resolve while their token is
/// consumed and retain their author spelling separately; the source itself is
/// never rewritten.
pub fn ParseCSSWithBase(text: &str, base_url: &str) -> CSSStyleSheet {
    ParseCSSWithModeAndBase(
        text,
        crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        base_url,
    )
}

/// Parse the browser's built-in sheets with Blink's UA parser mode while
/// retaining the same typed owner used to build the CSSOM projection. Keeping
/// the mode on `ProductionStyleSheet::contents` lets StyleEngine verify that a
/// typed declaration owner is valid for its cascade origin before using it.
pub fn ParseUserAgentCSS(text: &str) -> CSSStyleSheet {
    ParseUserAgentCSSWithRole(text, cssom::CSSUserAgentStyleSheetRole::kHTML)
}

pub fn ParseUserAgentCSSWithRole(
    text: &str,
    role: cssom::CSSUserAgentStyleSheetRole,
) -> CSSStyleSheet {
    let mut sheet = ParseCSSWithMode(
        text,
        crate::parser::css_parser_mode::CSSParserMode::kUASheetMode,
    );
    sheet.SetUserAgentStyleSheetRole(role);
    sheet
}

fn ParseCSSWithMode(
    text: &str,
    mode: crate::parser::css_parser_mode::CSSParserMode,
) -> CSSStyleSheet {
    let source = foundation::String::FromUtf8(text.as_bytes());
    let parsed = Rc::new(crate::production_style_sheet::ParseStyleSheet(
        &source, mode,
    ));
    let mut projection = ProjectStyleSheet(&parsed);
    projection.AttachParsedBacking(text, parsed);
    projection
}

fn ParseCSSWithModeAndBase(
    text: &str,
    mode: crate::parser::css_parser_mode::CSSParserMode,
    base_url: &str,
) -> CSSStyleSheet {
    let source = foundation::String::FromUtf8(text.as_bytes());
    let parsed = Rc::new(crate::production_style_sheet::ParseStyleSheetWithBase(
        &source, mode, base_url,
    ));
    let mut projection = ProjectStyleSheet(&parsed);
    projection.AttachParsedBacking(text, parsed);
    projection
}

/// Parse an inline declaration list through the translated Blink property
/// parser. This is the CSSOM-facing projection of
/// `CSSParserImpl::ParseInlineStyleDeclaration`; it does not use cssom's
/// legacy declaration tokenizer.
pub fn ParseCSSDeclarationList(text: &str) -> Vec<CSSDeclaration> {
    let source = foundation::String::FromUtf8(text.as_bytes());
    let properties =
        crate::parser::css_parser_impl::CSSParserImpl::<Backend>::ParseInlineStyleDeclaration(
            &source,
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
            crate::parser::css_parser_context::SecureContextMode::kInsecureContext,
            None,
        );
    ProjectPropertySet(properties.as_ref())
}

/// Native CSSOM write result. An unchanged/failed parse still enters the
/// custom-element style mutation scope, unlike an invalid property/priority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CSSDeclarationMutation {
    Unchanged,
    Changed(String),
    /// Existing declaration with a source SupportsIncrementalStyle property.
    ChangedIndependent(String),
}

/// Apply a CSSOM property write through the translated Blink parser and native
/// property store. None rejects the property/priority before mutation scope;
/// Unchanged preserves its custom-element notification without style dirtiness.
/// cpp: AbstractPropertySetCSSStyleDeclaration::setProperty/SetPropertyInternal;
/// InlineCSSStyleDeclaration::DidMutate.
/// Retained form of the same mutation used by an element's CSSOM declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CSSInlineDeclarationMutation {
    Unchanged,
    Changed(cssom::CSSInlineStyle),
    ChangedIndependent(cssom::CSSInlineStyle),
}

pub(crate) struct ParsedInlineStyle {
    mode: crate::parser::css_parser_mode::CSSParserMode,
    properties: Rc<CSSPropertyValueSet<ProductionCSSValueDispatch>>,
    declarations:
        std::cell::OnceCell<Rc<Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>>>,
}

impl ParsedInlineStyle {
    pub(crate) fn Declarations(
        &self,
    ) -> Rc<Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>> {
        self.declarations
            .get_or_init(|| {
                Rc::new(CSSPropertyValueSetRuleHandle::FromImmutable(
                    self.properties.clone(),
                ))
            })
            .clone()
    }
}

fn RetainInlineProperties(
    style: &cssom::CSSInlineStyle,
    properties: Rc<CSSPropertyValueSet<ProductionCSSValueDispatch>>,
    mode: crate::parser::css_parser_mode::CSSParserMode,
) -> Rc<ParsedInlineStyle> {
    let parsed = Rc::new(ParsedInlineStyle {
        mode,
        declarations: std::cell::OnceCell::new(),
        properties,
    });
    style.AttachParsedBacking(parsed.clone());
    parsed
}

pub(crate) fn ResolveInlineStyle(
    style: &cssom::CSSInlineStyle,
    mode: crate::parser::css_parser_mode::CSSParserMode,
) -> Rc<ParsedInlineStyle> {
    let mut trace = browser_tracing::span("style_parser", "StyleResolver.ParseInlineStyle");
    if let Some(parsed) = style
        .ParsedBacking::<ParsedInlineStyle>()
        .filter(|p| p.mode == mode)
    {
        trace.set("cache_hit", 1.0);
        return parsed;
    }
    trace.set("cache_hit", 0.0);
    trace.set("source_bytes", style.SourceText().len() as f64);
    let properties = if style.SourceText().is_empty() {
        // Element::EnsureMutableInlineStyle creates an empty store directly;
        // no source needs tokenizing before the first CSSOM property write.
        trace.set("empty_init", 1.0);
        crate::css_property_value_set::ImmutableCSSPropertyValueSet::Create(&[], mode, false)
    } else {
        let text = foundation::String::FromUtf8(style.SourceText().as_bytes());
        crate::parser::css_parser_impl::CSSParserImpl::<Backend>::ParseInlineStyleDeclaration(
            &text,
            mode,
            crate::parser::css_parser_context::SecureContextMode::kInsecureContext,
            None,
        )
    };
    RetainInlineProperties(style, properties, mode)
}

fn InlineParserMode(standard_mode: bool) -> crate::parser::css_parser_mode::CSSParserMode {
    use crate::parser::css_parser_mode::CSSParserMode;
    if standard_mode {
        CSSParserMode::kHTMLStandardMode
    } else {
        CSSParserMode::kHTMLQuirksMode
    }
}

fn CSSOMPropertyID(name: &str) -> Option<foundation::CSSPropertyID> {
    use foundation::{CSSPropertyID, String as CSSString, StringView};
    if name.starts_with("--") {
        let source = CSSString::FromUtf8(name.as_bytes());
        return (crate::parser::css_parser_impl::ParseCustomPropertyName(StringView::from(
            &source,
        ))
        .Utf8()
            == name)
            .then_some(CSSPropertyID::kVariable);
    }
    let entry = crate::css_property_names::FindProperty(name.to_ascii_lowercase().as_bytes())?;
    let raw = entry.id_and_exposed_bit & !crate::css_property_names::kNotKnownExposedPropertyBit;
    if raw <= foundation::kLastCSSProperty as i32 {
        Some(foundation::ConvertToCSSPropertyID(raw))
    } else {
        crate::css_property_names::kCSSPropertyAliasList
            .get((raw - foundation::kLastCSSProperty as i32 - 1) as usize)
            .copied()
    }
}

/// Read the package's existing CSSOM accepted-declaration view from its
/// retained property set. This ownership migration keeps that output contract;
/// the full CSSPropertyValueSet shorthand serializer is not yet connected.
pub fn GetInlineCSSDeclaration(
    style: &cssom::CSSInlineStyle,
    name: &str,
    standard_mode: bool,
) -> String {
    let parsed = ResolveInlineStyle(style, InlineParserMode(standard_mode));
    parsed
        .properties
        .Properties()
        .iter()
        .rev()
        .find(|property| property.Name().ToAtomicString().EqualsUtf8(name))
        .map_or_else(String::new, |property| property.Value().CssText().Utf8())
}

/// Keep the native declaration owner across CSSOM -> DOM -> Style. Parsing
/// an edited value must not reparse the current declaration or discard its
/// typed result at the attribute boundary.
pub fn SetInlineCSSDeclaration(
    style: &cssom::CSSInlineStyle,
    name: &str,
    value: &str,
    priority: &str,
    standard_mode: bool,
) -> Option<CSSInlineDeclarationMutation> {
    use crate::{
        css_property_value_set::{ImmutableCSSPropertyValueSet, SetResult},
        parser::{
            css_parser_context::{CSSParserContext, SecureContextMode},
            css_parser_impl::CSSParserImpl,
        },
    };
    use foundation::{AtomicString, CSSPropertyID, String as CSSString, StringView};
    let important = priority.eq_ignore_ascii_case("important");
    if !priority.is_empty() && !important {
        return None;
    }
    let id = CSSOMPropertyID(name)?;
    let mode = InlineParserMode(standard_mode);
    let parsed = ResolveInlineStyle(style, mode);
    let mut properties = parsed.properties.MutableCopy();
    let custom_name = AtomicString::from_str(name);
    let result = if value.is_empty() {
        let removed = if id == CSSPropertyID::kVariable {
            properties.RemoveProperty(&custom_name, None)
        } else {
            properties.RemoveProperty(crate::css_property_names::ResolveCSSPropertyID(id), None)
        };
        if removed {
            SetResult::kChangedPropertySet
        } else {
            SetResult::kUnchanged
        }
    } else {
        let context =
            CSSParserContext::<Backend>::FromMode(mode, SecureContextMode::kInsecureContext, None);
        let value = CSSString::FromUtf8(value.as_bytes());
        if id == CSSPropertyID::kVariable {
            CSSParserImpl::<Backend>::ParseVariableValue(
                &mut properties,
                &custom_name,
                StringView::from(&value),
                important,
                Some(&context),
                false,
            )
        } else {
            CSSParserImpl::<Backend>::ParseValue(
                &mut properties,
                id,
                StringView::from(&value),
                important,
                Some(&context),
            )
        }
    };
    Some(if result >= SetResult::kModifiedExisting {
        // Attribute text is still required by the current DOM mutation input.
        // It is a projection of this property set, never its replacement owner.
        let text = crate::production_style_sheet::SerializeProperties(&properties).Utf8();
        let next = cssom::CSSInlineStyle::new(&text);
        let immutable = ImmutableCSSPropertyValueSet::Create(properties.Properties(), mode, false);
        RetainInlineProperties(&next, immutable, mode);
        if result == SetResult::kModifiedExisting
            && crate::properties::css_property::CSSProperty::Get(
                crate::css_property_names::ResolveCSSPropertyID(id),
            )
            .SupportsIncrementalStyle()
        {
            CSSInlineDeclarationMutation::ChangedIndependent(next)
        } else {
            CSSInlineDeclarationMutation::Changed(next)
        }
    } else {
        CSSInlineDeclarationMutation::Unchanged
    })
}

/// Text adapter for callers that do not own a retained element declaration.
/// The CSSOM operation and its mutation classification have one implementation.
pub fn SetCSSDeclaration(
    text: &str,
    name: &str,
    value: &str,
    priority: &str,
) -> Option<CSSDeclarationMutation> {
    SetInlineCSSDeclaration(
        &cssom::CSSInlineStyle::new(text),
        name,
        value,
        priority,
        true,
    )
    .map(|result| match result {
        CSSInlineDeclarationMutation::Unchanged => CSSDeclarationMutation::Unchanged,
        CSSInlineDeclarationMutation::Changed(style) => {
            CSSDeclarationMutation::Changed(style.SourceText().into())
        }
        CSSInlineDeclarationMutation::ChangedIndependent(style) => {
            CSSDeclarationMutation::ChangedIndependent(style.SourceText().into())
        }
    })
}

/// Project style/font/keyframes/media/layer structure without replacing ownership.
/// `declarations` contains only accepted typed values and is not an exact author
/// declaration list; unsupported declarations remain in `declaration_text`.
/// DOM sets `owner_node_id` when attaching this projection to an owner node.
pub fn ProjectStyleSheet(sheet: &ProductionStyleSheet) -> CSSStyleSheet {
    let mut output = CSSStyleSheet::default();
    output.rules = sheet
        .ProjectionRulesInSourceOrder()
        .into_iter()
        .map(|rule| match rule {
            ProductionProjectionRule::Style(rule) => ProjectStyleRule(rule),
            ProductionProjectionRule::NestedDeclarations(rule) => {
                ProjectNestedDeclarationsRule(rule)
            }
        })
        .collect();
    output.font_faces = sheet.font_faces.iter().map(ProjectFontFaceRule).collect();
    output.keyframes = sheet.keyframes.iter().map(ProjectKeyframesRule).collect();
    output.scopes = sheet
        .scopes
        .iter()
        .map(crate::production_rule_effect_projection::ProjectScopeRule)
        .collect();
    output.property_rules = sheet
        .property_registration_effects
        .iter()
        .filter_map(|effect| {
            crate::production_rule_effect_projection::ProjectPropertyRegistrationEffect(
                effect,
                &sheet.source,
            )
        })
        .collect();
    let mut source_order = 0;
    for index in 0..sheet.contents.RuleCount() {
        AppendLayerOrder(
            &sheet.contents.RuleAt(index),
            "",
            &mut source_order,
            &mut output.layer_order,
        );
    }
    output
}

fn ProjectStyleRule(rule: &ProductionStyleRule) -> CSSStyleRule {
    CSSStyleRule {
        selector_text: if rule.parent_rule_for_nesting.is_some() {
            rule.rule.Selectors().SelectorsTextInternal(true, 0).Utf8()
        } else {
            rule.selector_source.Utf8()
        },
        declaration_text: rule.declaration_source.Utf8(),
        declarations: ProjectDeclarations(rule.rule.Properties().as_ref()),
        media_conditions: rule
            .media
            .iter()
            .map(|query| query.MediaText().Utf8())
            .collect(),
        layer_name: ProjectLayerName(&rule.layers),
        scope_conditions: rule
            .scope_conditions
            .iter()
            .map(|scope| crate::production_rule_effect_projection::ProjectStyleScope(scope))
            .collect(),
        container_conditions: rule
            .container_conditions
            .iter()
            .map(|set| crate::production_container_projection::ProjectContainerQuerySet(set))
            .collect(),
    }
}

fn ProjectNestedDeclarationsRule(rule: &ProductionNestedDeclarationsRule) -> CSSStyleRule {
    CSSStyleRule {
        selector_text: rule
            .rule
            .InnerStyleRule()
            .Selectors()
            .SelectorsTextInternal(true, 0)
            .Utf8(),
        declaration_text: rule.declaration_source.Utf8(),
        declarations: ProjectDeclarations(rule.rule.Properties().as_ref()),
        media_conditions: rule
            .media
            .iter()
            .map(|query| query.MediaText().Utf8())
            .collect(),
        layer_name: ProjectLayerName(&rule.layers),
        scope_conditions: rule
            .scope_conditions
            .iter()
            .map(|scope| crate::production_rule_effect_projection::ProjectStyleScope(scope))
            .collect(),
        container_conditions: rule
            .container_conditions
            .iter()
            .map(|set| crate::production_container_projection::ProjectContainerQuerySet(set))
            .collect(),
    }
}
fn ProjectFontFaceRule(rule: &ProductionFontFaceRule) -> CSSFontFaceRule {
    CSSFontFaceRule {
        declaration_text: rule.declaration_source.Utf8(),
        declarations: ProjectDeclarations(rule.rule.Properties()),
        media_conditions: rule
            .media
            .iter()
            .map(|query| query.MediaText().Utf8())
            .collect(),
        layer_name: ProjectLayerName(&rule.layers),
    }
}

fn ProjectKeyframesRule(rule: &ProductionKeyframesRule) -> CSSKeyframesRule {
    CSSKeyframesRule {
        name: rule.rule.name.Utf8(),
        keyframes: rule.keyframes.iter().map(ProjectKeyframeRule).collect(),
        media_conditions: rule
            .media
            .iter()
            .map(|query| query.MediaText().Utf8())
            .collect(),
        layer_name: ProjectLayerName(&rule.layers),
    }
}

fn ProjectKeyframeRule(rule: &ProductionKeyframeRule) -> CSSKeyframeRule {
    CSSKeyframeRule {
        key_text: rule.key_source.Utf8(),
        declaration_text: rule.declaration_source.Utf8(),
        declarations: ProjectDeclarations(rule.rule.Properties().as_ref()),
    }
}

/// Shared by style, descriptor and keyframe projections. Serialize the
/// individual values, rather than asking the unfinished shorthand serializer
/// to reconstruct author source. The exact source remains in declaration_text.
pub(crate) fn ProjectDeclarations(
    properties: &CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>,
) -> Vec<CSSDeclaration> {
    match properties {
        CSSPropertyValueSetRuleHandle::Immutable(set) => ProjectPropertySet(set),
        CSSPropertyValueSetRuleHandle::Mutable(set) => ProjectPropertySet(&set.borrow()),
    }
}

fn ProjectPropertySet(
    properties: &CSSPropertyValueSet<ProductionCSSValueDispatch>,
) -> Vec<CSSDeclaration> {
    properties
        .Properties()
        .iter()
        .map(|property| CSSDeclaration {
            property: property.Name().ToAtomicString().Utf8(),
            value: property.Value().CssText().Utf8(),
            important: property.IsImportant(),
        })
        .collect()
}

fn AnonymousLayerName(source_order: u32) -> std::string::String {
    format!("__anonymous_{source_order}")
}

fn QualifiedLayerName(outer: &str, inner: &str) -> std::string::String {
    if outer.is_empty() {
        inner.to_owned()
    } else {
        format!("{outer}.{inner}")
    }
}

fn ProjectLayerName(layers: &[LayerSegment]) -> std::string::String {
    layers
        .iter()
        .map(|layer| match layer {
            LayerSegment::Named(name) => LayerNameAsString(name).Utf8(),
            LayerSegment::Anonymous(order) => AnonymousLayerName(*order),
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn AppendUnique(names: &mut Vec<std::string::String>, name: std::string::String) {
    if !names.contains(&name) {
        names.push(name);
    }
}

// Traverse already parsed rule ownership, including empty layer blocks. The
// pre-order position agrees with ProductionStyleSheet's LayerSegment identity;
// CSS text, braces and tokens are never consulted to rediscover rule structure.
fn AppendLayerOrder(
    rule: &Rc<StyleRuleBase<Backend>>,
    parent: &str,
    source_order: &mut u32,
    layers: &mut Vec<std::string::String>,
) {
    let position = *source_order;
    *source_order += 1;
    match rule.as_ref() {
        StyleRuleBase::LayerBlock(rule) => {
            let name = if rule.GetName().len() == 1 && rule.GetName()[0].Utf8().is_empty() {
                AnonymousLayerName(position)
            } else {
                rule.GetNameAsString().Utf8()
            };
            let name = QualifiedLayerName(parent, &name);
            AppendUnique(layers, name.clone());
            for child in rule.ChildRules() {
                AppendLayerOrder(child, &name, source_order, layers);
            }
        }
        StyleRuleBase::LayerStatement(rule) => {
            for name in rule.GetNames() {
                AppendUnique(
                    layers,
                    QualifiedLayerName(parent, &LayerNameAsString(name).Utf8()),
                );
            }
        }
        StyleRuleBase::Scope(rule) => {
            for child in rule.ChildRules() {
                AppendLayerOrder(child, parent, source_order, layers);
            }
        }
        StyleRuleBase::Container(rule) => {
            for child in rule.ChildRules() {
                AppendLayerOrder(child, parent, source_order, layers);
            }
        }
        StyleRuleBase::Media(rule) => {
            for child in rule.ChildRules() {
                AppendLayerOrder(child, parent, source_order, layers);
            }
        }
        StyleRuleBase::Supports(rule) => {
            // Match the production traversal's source positions for anonymous
            // layers even when a supports condition disables its descendants.
            for child in rule.ChildRules() {
                let mut ignored = Vec::new();
                AppendLayerOrder(
                    child,
                    parent,
                    source_order,
                    if rule.ConditionIsSupported() {
                        layers
                    } else {
                        &mut ignored
                    },
                );
            }
        }
        StyleRuleBase::Style(rule) => {
            if let Some(children) = rule.ChildRules() {
                for child in children.iter() {
                    AppendLayerOrder(child, parent, source_order, layers);
                }
            }
        }
        // Leaf rule kinds, including keyframe blocks, don't establish layer order.
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser::css_parser_mode::CSSParserMode, production_style_sheet::ParseStyleSheet};
    use foundation::String;

    fn project(text: &str) -> CSSStyleSheet {
        ProjectStyleSheet(&ParseStyleSheet(
            &String::FromUtf8(text.as_bytes()),
            CSSParserMode::kHTMLStandardMode,
        ))
    }

    #[test]
    fn public_parse_retains_the_typed_owner_and_source_provenance() {
        let css = "/* exact source */ .item { color: red }";
        let sheet = ParseCSS(css);
        assert_ne!(sheet.Identity(), 0);
        assert_eq!(sheet.Revision(), 1);
        assert_eq!(sheet.SourceText(), css);
        let parsed = sheet
            .ParsedBacking::<ProductionStyleSheet>()
            .expect("translated parser result must survive its CSSOM projection");
        assert_eq!(parsed.source.Utf8(), css);
        assert_eq!(parsed.rules.len(), 1);
        let cloned = sheet.clone();
        assert!(Rc::ptr_eq(
            &parsed,
            &cloned
                .ParsedBacking::<ProductionStyleSheet>()
                .expect("clones retain the same typed owner")
        ));
        assert!(!format!("{sheet:?}").contains("parsed_backing"));
    }

    #[test]
    fn parse_with_base_rebases_urls_in_the_retained_typed_owner() {
        let css = "/* url(fake.png) */ .item{--raw:'url(fake.png)';background-image:url(../images/a.svg);mask-image:url(mask.svg);filter:url(#fx)}.item::before{content:url('flag')}@font-face{font-family:x;src:url('../fonts/x.woff2') format('woff2')}";
        let sheet = ParseCSSWithBase(css, "https://example.test/css/main/site.css");
        assert_eq!(sheet.SourceText(), css);
        let declarations = sheet
            .rules
            .iter()
            .flat_map(|rule| rule.declarations.iter())
            .map(|declaration| declaration.value.as_str())
            .collect::<Vec<_>>();
        assert!(declarations
            .iter()
            .any(|value| { value.contains("https://example.test/css/images/a.svg") }));
        assert!(declarations
            .iter()
            .any(|value| value.contains("https://example.test/css/main/mask.svg")));
        assert!(declarations
            .iter()
            .any(|value| value.contains("https://example.test/css/main/flag")));
        assert!(declarations
            .iter()
            .any(|value| value.contains("url(\"#fx\")")));
        assert!(sheet.font_faces[0].declarations.iter().any(|declaration| {
            declaration
                .value
                .contains("https://example.test/css/fonts/x.woff2")
        }));
        let parsed = sheet
            .ParsedBacking::<ProductionStyleSheet>()
            .expect("based sheet keeps its translated typed owner");
        assert_eq!(parsed.source.Utf8(), css);
        assert_eq!(
            parsed.contents.ParserContext().BaseURL(),
            &crate::production_css_value::ProductionURL::FromString(
                "https://example.test/css/main/site.css"
            )
        );
    }

    #[test]
    fn user_agent_parse_retains_ua_mode_on_the_typed_owner() {
        let sheet = ParseUserAgentCSS("html { display: block }");
        let parsed = sheet
            .ParsedBacking::<ProductionStyleSheet>()
            .expect("UA projection retains its translated owner");
        assert_eq!(
            parsed.contents.ParserContext().Mode(),
            CSSParserMode::kUASheetMode
        );
    }

    #[test]
    fn exact_observer_source_and_only_accepted_properties_reach_cssom() {
        let css = "/*head*/ é😀[data-v='a}b']/**/ { width: 12px; unknown-property: 8; color:red!important; color:blue; --Payload: [a;{b:'}'}]; }";
        let sheet = project(css);
        assert_eq!(sheet.rules.len(), 1);
        let rule = &sheet.rules[0];
        assert_eq!(rule.selector_text, "é😀[data-v='a}b']/**/ ");
        assert_eq!(rule.declaration_text, " width: 12px; unknown-property: 8; color:red!important; color:blue; --Payload: [a;{b:'}'}]; ");
        assert!(!rule
            .declarations
            .iter()
            .any(|property| property.property == "unknown-property"));
        let color = rule
            .declarations
            .iter()
            .find(|property| property.property == "color")
            .unwrap();
        assert_eq!(color.value, "red");
        assert!(color.important);
        assert!(rule
            .declarations
            .iter()
            .any(|property| property.property == "--Payload"));
        assert_eq!(sheet.owner_node_id, 0);
    }

    #[test]
    fn inline_declarations_use_the_translated_property_parser() {
        let declarations = ParseCSSDeclarationList(
            "width:12px; color:red!important; unknown-property:1; --token: calc(1px + 2px)",
        );
        assert!(declarations
            .iter()
            .any(|value| value.property == "width" && value.value == "12px"));
        assert!(declarations
            .iter()
            .any(|value| { value.property == "color" && value.value == "red" && value.important }));
        assert!(declarations.iter().any(|value| value.property == "--token"));
        assert!(!declarations
            .iter()
            .any(|value| value.property == "unknown-property"));

        let retained = cssom::CSSInlineStyle::new("margin:1px;width:12px");
        assert_eq!(GetInlineCSSDeclaration(&retained, "width", true), "12px");
        assert_eq!(
            GetInlineCSSDeclaration(&retained, "margin-top", true),
            "1px"
        );
        // Google reads absent shorthand properties during bootstrap. The
        // established CSSOM view must not enter an unavailable serializer.
        assert_eq!(
            GetInlineCSSDeclaration(&cssom::CSSInlineStyle::new(""), "margin", true),
            ""
        );

        // CSSOM writes consume the native SetResult, retaining source text on
        // no change and rejecting values before a DOM mutation is synthesized.
        let source = "width:200px;color:red";
        assert_eq!(
            SetCSSDeclaration(source, "width", "200px", ""),
            Some(CSSDeclarationMutation::Unchanged)
        );
        assert_eq!(
            SetCSSDeclaration(source, "width", "invalid", ""),
            Some(CSSDeclarationMutation::Unchanged)
        );
        assert_eq!(
            SetCSSDeclaration(source, "width", "10px;display:none", ""),
            Some(CSSDeclarationMutation::Unchanged)
        );
        assert_eq!(SetCSSDeclaration(source, "width", "201px", "bad"), None);
        let changed = SetCSSDeclaration(source, "WIDTH", "201px", "IMPORTANT").unwrap();
        assert_eq!(
            changed,
            CSSDeclarationMutation::ChangedIndependent(
                "width: 201px !important; color: red;".into()
            )
        );
        let shorthand = SetCSSDeclaration("margin-left:9px;color:red", "margin", "", "").unwrap();
        assert_eq!(
            shorthand,
            CSSDeclarationMutation::Changed("color: red;".into())
        );
        let custom = SetCSSDeclaration("--X:one;--x:two", "--X", "", "").unwrap();
        assert_eq!(custom, CSSDeclarationMutation::Changed("--x: two;".into()));
        let logical = SetCSSDeclaration(
            "margin-left:1px;margin-inline-start:2px",
            "margin-left",
            "1px",
            "",
        )
        .unwrap();
        assert_eq!(
            logical,
            CSSDeclarationMutation::Changed("margin-inline-start: 2px; margin-left: 1px;".into())
        );
        assert_eq!(SetCSSDeclaration("height:1px", "height", "", "bad"), None);
        assert_eq!(SetCSSDeclaration("", "not-a-property", "1px", ""), None);
        assert_eq!(
            SetCSSDeclaration("", "word-wrap", "break-word", ""),
            Some(CSSDeclarationMutation::Changed(
                "overflow-wrap: break-word;".into()
            ))
        );
    }

    #[test]
    fn layers_include_empty_blocks_statements_nested_names_and_stable_anonymous_identity() {
        let sheet = project("@layer reset, theme; @layer empty {} @media screen { @layer theme { @layer base, widgets; a{width:12px} @media(min-width:700px){b{height:23px}} } } @layer { c{opacity:.5} } d{width:5px}");
        assert_eq!(
            sheet.layer_order,
            [
                "reset",
                "theme",
                "empty",
                "theme.base",
                "theme.widgets",
                "__anonymous_8"
            ]
        );
        assert_eq!(sheet.rules[0].layer_name, "theme");
        assert_eq!(sheet.rules[1].layer_name, "theme");
        assert_eq!(sheet.rules[2].layer_name, "__anonymous_8");
        assert_eq!(sheet.rules[3].layer_name, "");
        assert_eq!(sheet.rules[0].media_conditions, ["screen"]);
        assert_eq!(
            sheet.rules[1].media_conditions,
            ["screen", "(min-width: 700px)"]
        );
    }

    #[test]
    fn escaped_layer_names_and_expanded_shorthands_come_from_typed_values() {
        let sheet = project(r"@layer outer { @layer a\+b.c { a{margin:1px 2px; --Accent:RED} } }");
        assert_eq!(sheet.layer_order, ["outer", r"outer.a\+b.c"]);
        assert_eq!(sheet.rules[0].layer_name, r"outer.a\+b.c");
        assert!(sheet.rules[0]
            .declarations
            .iter()
            .any(|property| property.property == "margin-left" && property.value == "2px"));
        assert!(sheet.rules[0]
            .declarations
            .iter()
            .any(|property| property.property == "--Accent" && property.value == "RED"));
    }

    #[test]
    fn font_faces_preserve_observer_body_and_project_only_accepted_descriptors() {
        let body = " /* }😀 */ font-family: 'Demo😀'; src: local(Demo Face), bogus(x), url('a}b.woff2') format(woff2); unicode-range: U+0-7F, U+4??; font-weight: 900 100; font-display:swap!important; unknown-descriptor: [a;{b:'}'}]; ";
        let sheet = project(&format!(
            "@layer outer{{@media screen{{@layer fonts{{@media(min-width:700px){{@font-face{{{body}}}}}}}}}}}"
        ));
        assert_eq!(sheet.font_faces.len(), 1);
        let face = &sheet.font_faces[0];
        assert_eq!(face.declaration_text, body);
        assert_eq!(face.media_conditions, ["screen", "(min-width: 700px)"]);
        assert_eq!(face.layer_name, "outer.fonts");
        assert_eq!(sheet.layer_order, ["outer", "outer.fonts"]);
        assert!(face.declarations.iter().any(|declaration| {
            declaration.property == "font-family" && declaration.value == "Demo😀"
        }));
        assert!(face.declarations.iter().any(|declaration| {
            declaration.property == "src"
                && declaration.value == "local(\"Demo Face\"), url(\"a}b.woff2\") format(\"woff2\")"
        }));
        assert!(face.declarations.iter().any(|declaration| {
            declaration.property == "unicode-range" && declaration.value == "U+0-7F, U+400-4FF"
        }));
        assert!(face.declarations.iter().any(|declaration| {
            declaration.property == "font-weight" && declaration.value == "900 100"
        }));
        assert!(!face.declarations.iter().any(|declaration| {
            declaration.property == "unknown-descriptor" || declaration.property == "font-display"
        }));
        assert!(face
            .declarations
            .iter()
            .all(|declaration| !declaration.important));
    }

    #[test]
    fn keyframes_preserve_exact_keys_bodies_and_accepted_typed_properties() {
        let body = "opacity:0;--Payload:[a;{b:'}😀'}];margin:1px 2px;width:8px!important;unknown-property:7;";
        let sheet = project(&format!(
            "@media screen{{@layer motion{{@media(min-width:700px){{@-webkit-keyframes f\\61 de {{ from, 50%/**/ {{{body}}} 120%{{opacity:.9}} unknown{{width:0}} to {{opacity:1}} }} @keyframes 'Quoted😀'{{entry -20%{{height:4px}}}}}}}}}}"
        ));
        assert_eq!(sheet.keyframes.len(), 2);
        let animation = &sheet.keyframes[0];
        assert_eq!(animation.name, "fade");
        assert_eq!(animation.media_conditions, ["screen", "(min-width: 700px)"]);
        assert_eq!(animation.layer_name, "motion");
        assert_eq!(animation.keyframes.len(), 2);
        let first = &animation.keyframes[0];
        assert_eq!(first.key_text, "from, 50%/**/ ");
        assert_eq!(first.declaration_text, body);
        assert!(first
            .declarations
            .iter()
            .any(|declaration| { declaration.property == "opacity" && declaration.value == "0" }));
        assert!(first.declarations.iter().any(|declaration| {
            declaration.property == "margin-left" && declaration.value == "2px"
        }));
        assert!(first.declarations.iter().any(|declaration| {
            declaration.property == "--Payload" && declaration.value == "[a;{b:'}😀'}]"
        }));
        assert!(!first.declarations.iter().any(|declaration| {
            declaration.property == "width" || declaration.property == "unknown-property"
        }));
        assert!(first
            .declarations
            .iter()
            .all(|declaration| !declaration.important));
        assert_eq!(animation.keyframes[1].key_text, "to ");
        assert_eq!(animation.keyframes[1].declaration_text, "opacity:1");
        assert_eq!(sheet.keyframes[1].name, "Quoted😀");
        assert_eq!(sheet.keyframes[1].keyframes[0].key_text, "entry -20%");
        assert_eq!(sheet.keyframes[1].layer_name, "motion");
    }

    #[test]
    fn font_and_keyframe_leaves_keep_anonymous_layer_identity_and_eof_source() {
        let sheet = project("@font-face{font-family:First;src:url(first)} @keyframes first{from{opacity:0}to{opacity:1}} @layer{@font-face{font-family:Second;src:url(second)} @keyframes second{from{opacity:0}to{opacity:1}} a{width:1px}} @keyframes 'Tail😀'{from{opacity:0!important}to{opacity:1");
        assert_eq!(sheet.layer_order, ["__anonymous_2"]);
        assert_eq!(sheet.rules[0].layer_name, "__anonymous_2");
        assert_eq!(sheet.font_faces[0].layer_name, "");
        assert_eq!(sheet.font_faces[1].layer_name, "__anonymous_2");
        assert_eq!(sheet.keyframes[0].name, "first");
        assert_eq!(sheet.keyframes[1].name, "second");
        assert_eq!(sheet.keyframes[1].layer_name, "__anonymous_2");
        let tail = &sheet.keyframes[2];
        assert_eq!(tail.name, "Tail😀");
        assert_eq!(tail.layer_name, "");
        assert_eq!(tail.keyframes.len(), 2);
        assert_eq!(tail.keyframes[0].declaration_text, "opacity:0!important");
        assert!(tail.keyframes[0].declarations.is_empty());
        assert_eq!(tail.keyframes[1].declaration_text, "opacity:1");
    }
}
