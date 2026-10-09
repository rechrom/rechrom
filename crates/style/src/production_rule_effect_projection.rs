// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Ownership projection of parsed scope boundaries and validated registrations.
#![allow(non_snake_case)]
use crate::{
    css_syntax_component::{CSSSyntaxRepeat, CSSSyntaxType},
    css_value::CSSValuePayload,
    production_container_projection::{ProjectContainerUnit, ProjectContainerVariableData},
    production_css_value as values,
    production_style_sheet::{LayerSegment, ProductionScopeRule},
    property_registration::PropertyRegistrationEffect,
    style_scope::StyleScope,
};
use ::cssom::*;
use foundation::{AtomicString, String, StringView};
pub fn ProjectStyleScope(scope: &StyleScope) -> CSSStyleScope {
    CSSStyleScope {
        root: scope
            .From()
            .map(|list| CSSScopeRoot::ExplicitSelectorList(list.SelectorsText().Utf8()))
            .unwrap_or(CSSScopeRoot::ImplicitStylesheetOwner),
        limit: scope.To().map(|list| list.SelectorsText().Utf8()),
    }
}
pub fn ProjectScopeRule(rule: &ProductionScopeRule) -> CSSScopeRule {
    CSSScopeRule {
        scope: ProjectStyleScope(&rule.scope),
        prelude_text: rule.prelude_source.Utf8(),
        source_order: rule.source_order,
    }
}
pub fn ProjectRegisteredValue(value: &values::Value) -> Option<CSSRegisteredPropertyValue> {
    use CSSRegisteredPropertyValue as V;
    Some(match value.Payload() {
        CSSValuePayload::kNumericLiteralClass(v) => V::Numeric(
            CSSContainerNumber::new(v.DoubleValue()),
            ProjectContainerUnit(v.GetType()),
        ),
        CSSValuePayload::kIdentifierClass(v) => V::Identifier(v.0),
        CSSValuePayload::kCustomIdentClass(v) => V::CustomIdent(v.name.clone()),
        CSSValuePayload::kStringClass(v) => {
            V::String(AtomicString::from_utf16(v.0.Span16().unwrap_or_default()))
        }
        CSSValuePayload::kColorClass(v) => V::Color {
            space: v.0.GetColorSpace(),
            components: [
                CSSContainerNumber::new(v.0.Param0() as f64),
                CSSContainerNumber::new(v.0.Param1() as f64),
                CSSContainerNumber::new(v.0.Param2() as f64),
            ],
            alpha: CSSContainerNumber::new(v.0.Alpha() as f64),
            none: [
                v.0.Param0IsNone(),
                v.0.Param1IsNone(),
                v.0.Param2IsNone(),
                v.0.AlphaIsNone(),
            ],
        },
        CSSValuePayload::kUnparsedDeclarationClass(v) => {
            V::Unparsed(ProjectContainerVariableData(&v.data))
        }
        CSSValuePayload::kValueListClass(v) => V::List(
            v.values
                .iter()
                .map(|v| ProjectRegisteredValue(v))
                .collect::<Option<Vec<_>>>()?,
            match v.separator {
                values::ListSeparator::Space => CSSContainerListSeparator::Space,
                values::ListSeparator::Comma => CSSContainerListSeparator::Comma,
                values::ListSeparator::Slash => CSSContainerListSeparator::Slash,
            },
        ),
        _ => return None,
    })
}
pub fn ProjectPropertyRegistrationEffect(
    effect: &PropertyRegistrationEffect,
    source: &String,
) -> Option<CSSPropertyRule> {
    let syntax = CSSPropertySyntax {
        components: effect
            .syntax
            .Components()
            .iter()
            .map(|component| CSSPropertySyntaxComponent {
                kind: match component.GetType() {
                    CSSSyntaxType::kTokenStream => CSSPropertySyntaxType::kTokenStream,
                    CSSSyntaxType::kIdent => CSSPropertySyntaxType::kIdent,
                    CSSSyntaxType::kLength => CSSPropertySyntaxType::kLength,
                    CSSSyntaxType::kNumber => CSSPropertySyntaxType::kNumber,
                    CSSSyntaxType::kPercentage => CSSPropertySyntaxType::kPercentage,
                    CSSSyntaxType::kLengthPercentage => CSSPropertySyntaxType::kLengthPercentage,
                    CSSSyntaxType::kColor => CSSPropertySyntaxType::kColor,
                    CSSSyntaxType::kImage => CSSPropertySyntaxType::kImage,
                    CSSSyntaxType::kUrl => CSSPropertySyntaxType::kUrl,
                    CSSSyntaxType::kInteger => CSSPropertySyntaxType::kInteger,
                    CSSSyntaxType::kAngle => CSSPropertySyntaxType::kAngle,
                    CSSSyntaxType::kTime => CSSPropertySyntaxType::kTime,
                    CSSSyntaxType::kResolution => CSSPropertySyntaxType::kResolution,
                    CSSSyntaxType::kTransformFunction => CSSPropertySyntaxType::kTransformFunction,
                    CSSSyntaxType::kTransformList => CSSPropertySyntaxType::kTransformList,
                    CSSSyntaxType::kCustomIdent => CSSPropertySyntaxType::kCustomIdent,
                    CSSSyntaxType::kString => CSSPropertySyntaxType::kString,
                },
                ident: AtomicString::from_utf16(component.GetString().Span16().unwrap_or_default()),
                repeat: match component.GetRepeat() {
                    CSSSyntaxRepeat::kNone => CSSPropertySyntaxRepeat::None,
                    CSSSyntaxRepeat::kSpaceSeparated => CSSPropertySyntaxRepeat::SpaceSeparated,
                    CSSSyntaxRepeat::kCommaSeparated => CSSPropertySyntaxRepeat::CommaSeparated,
                },
            })
            .collect(),
    };
    let initial = effect
        .initial
        .as_ref()
        .map(|value| ProjectRegisteredValue(value))
        .transpose_option()?;
    Some(CSSPropertyRule {
        name: effect.name.clone(),
        syntax,
        syntax_text: effect.syntax_text.Utf8(),
        inherits: effect.inherits,
        initial_value: initial,
        initial_value_data: effect
            .initial_source
            .as_ref()
            .map(|data| ProjectContainerVariableData(data)),
        initial_value_text: effect
            .initial_source
            .as_ref()
            .map(|data| data.Serialize().Utf8()),
        declaration_text: StringView::from(source)
            .Substring(
                effect.source_range.body_start,
                effect.source_range.body_end - effect.source_range.body_start,
            )
            .ToString()
            .Utf8(),
        media_conditions: effect
            .media
            .iter()
            .map(|query| query.MediaText().Utf8())
            .collect(),
        layer_name: effect
            .layers
            .iter()
            .map(|layer| match layer {
                LayerSegment::Named(names) => crate::style_rule::LayerNameAsString(names).Utf8(),
                LayerSegment::Anonymous(order) => format!("__anonymous_{order}"),
            })
            .collect::<Vec<_>>()
            .join("."),
        source_order: effect.source_order,
    })
}
trait TransposeOption<T> {
    fn transpose_option(self) -> Option<Option<T>>;
}
impl<T> TransposeOption<T> for Option<Option<T>> {
    fn transpose_option(self) -> Option<Option<T>> {
        match self {
            None => Some(None),
            Some(Some(value)) => Some(Some(value)),
            Some(None) => None,
        }
    }
}
