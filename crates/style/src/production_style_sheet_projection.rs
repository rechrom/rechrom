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
        ProductionKeyframesRule, ProductionNestedDeclarationsRule, ProductionStyleRule,
        ProductionStyleSheet,
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
    let source = foundation::String::FromUtf8(text.as_bytes());
    let parsed = crate::production_style_sheet::ParseStyleSheet(
        &source,
        crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
    );
    ProjectStyleSheet(&parsed)
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

/// Project style/font/keyframes/media/layer structure without replacing ownership.
/// `declarations` contains only accepted typed values and is not an exact author
/// declaration list; unsupported declarations remain in `declaration_text`.
/// DOM sets `owner_node_id` when attaching this projection to an owner node.
pub fn ProjectStyleSheet(sheet: &ProductionStyleSheet) -> CSSStyleSheet {
    let mut rules = sheet
        .rules
        .iter()
        .map(|rule| (rule.source_order, ProjectStyleRule(rule)))
        .chain(
            sheet
                .nested_declarations
                .iter()
                .map(|rule| (rule.source_order, ProjectNestedDeclarationsRule(rule))),
        )
        .collect::<Vec<_>>();
    rules.sort_by_key(|rule| rule.0);
    let mut output = CSSStyleSheet {
        rules: rules.into_iter().map(|rule| rule.1).collect(),
        font_faces: sheet.font_faces.iter().map(ProjectFontFaceRule).collect(),
        keyframes: sheet.keyframes.iter().map(ProjectKeyframesRule).collect(),
        scopes: sheet
            .scopes
            .iter()
            .map(crate::production_rule_effect_projection::ProjectScopeRule)
            .collect(),
        property_rules: sheet
            .property_registration_effects
            .iter()
            .filter_map(|effect| {
                crate::production_rule_effect_projection::ProjectPropertyRegistrationEffect(
                    effect,
                    &sheet.source,
                )
            })
            .collect(),
        ..CSSStyleSheet::default()
    };
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
