// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! editing_style.cc:884–929,1476–1651,1764–1783,2121–2169 and
//! editing_style_utilities.cc:88–139. Uses original matched-rule declarations
//! and the live ComputedStyle; CSS strings are only the final markup projection.
use super::*;
use crate::resolver::production_style_builder::custom_properties::EnvironmentVariableResolver;
use crate::{
    document_style_engine::DocumentStyleEngine,
    serialization_style_query::{IsSerializationPercentage, SerializationRuleScope},
};
use cssom::editing_style::SerializationStyleOperation as SerializationOp;

fn Empty(document: &dom::Document) -> ComputedPropertySet {
    ComputedPropertySet::new(Mode(document))
}
fn Mode(document: &dom::Document) -> crate::parser::css_parser_mode::CSSParserMode {
    if document.GetCompatibilityMode() == dom::persistent_document::CompatibilityMode::kQuirks {
        crate::parser::css_parser_mode::CSSParserMode::kHTMLQuirksMode
    } else {
        crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode
    }
}
fn Node(document: &dom::Document, id: u64) -> Result<usize, Error> {
    document.FindNodeById(id).ok_or(Error::MissingStyle)
}
fn IsMailQuote(document: &dom::Document, node: usize) -> bool {
    let n = document.Node(node);
    n.Namespace() == dom::DOMNamespace::kHTML
        && n.Name() == "blockquote"
        && n.FindAttribute("type")
            .is_some_and(|a| a.value.eq_ignore_ascii_case("cite"))
}
pub(super) fn Matched(
    engine: &mut DocumentStyleEngine,
    document: &dom::Document,
    node: usize,
    scope: SerializationRuleScope,
    environment: Option<&dyn EnvironmentVariableResolver>,
) -> Result<ComputedPropertySet, Error> {
    let query = engine
        .SerializationStyleQuery(document, node, scope, environment)
        .map_err(|_| Error::UnavailableMatchedRules)?;
    let mut result = Empty(document);
    query.ForEachMatchedDeclaration(|set| result.MergeAndOverrideOnConflict(set));
    Ok(result)
}
pub(super) fn Inline(
    engine: &DocumentStyleEngine,
    document: &dom::Document,
    node: usize,
) -> Result<ComputedPropertySet, Error> {
    let mut result = Empty(document);
    if let Some(declarations) = engine
        .InlineSerializationStyle(document, node)
        .map_err(|_| Error::MissingCompiledDeclaration)?
    {
        declarations.WithPropertySet(|set| result.MergeAndOverrideOnConflict(set));
    }
    Ok(result)
}
/// MergeStyle keeps decoration unions even when ordinary values use the
/// do-not-override mode. Values remain native CSSValue owners throughout.
fn MergeEditing(
    target: &mut ComputedPropertySet,
    source: &ComputedPropertySet,
    override_values: bool,
) {
    for property in source.Properties() {
        let id = property.PropertyID();
        let mut has_value = target.HasProperty(id);
        if matches!(
            id,
            P::kTextDecorationLine | P::kWebkitTextDecorationsInEffect
        ) {
            if let CSSValuePayload::kValueListClass(to_add) = property.Value().Payload() {
                if let Some(CSSValuePayload::kValueListClass(current)) =
                    target.GetPropertyCSSValue(id).map(|v| v.Payload())
                {
                    let mut result = current.Copy();
                    for value in [V::kUnderline, V::kLineThrough] {
                        let value = values::identifier(value);
                        if to_add.HasValue(value.as_ref()) && !result.HasValue(value.as_ref()) {
                            result.values.push(value);
                        }
                    }
                    target.SetProperty(
                        id,
                        values::list(result.values, result.separator),
                        property.IsImportant(),
                    );
                    continue;
                }
                // Source MergeStyle treats text-decoration:none as absence
                // when merging an incoming list in do-not-override mode.
                has_value = false;
            }
        }
        if override_values || !has_value {
            target.SetLonghandProperty(property.clone());
        }
    }
}
fn CollapseDecorations(style: &mut ComputedPropertySet) {
    if let Some(value) = style.GetPropertyValueRef(P::kWebkitTextDecorationsInEffect) {
        if matches!(value.Payload(), CSSValuePayload::kValueListClass(_)) {
            let important = style.PropertyIsImportant(P::kTextDecorationLine);
            style.SetProperty(P::kTextDecorationLine, value, important);
        } else {
            style.RemoveProperty(P::kTextDecorationLine, None);
        }
        style.RemoveProperty(P::kWebkitTextDecorationsInEffect, None);
    }
}
fn StripUA(style: &mut ComputedPropertySet) {
    if style.GetPropertyValue(P::kFontFamily).as_str() == "\"\"" {
        style.RemoveProperty(P::kFontFamily, None);
    }
}
fn ForceInline(style: &mut ComputedPropertySet) {
    style.SetProperty(P::kDisplay, values::identifier(V::kInline), true);
}

fn MergeRulesForSerialization(
    engine: &mut DocumentStyleEngine,
    document: &dom::Document,
    node: usize,
    inline: &ComputedPropertySet,
    environment: Option<&dyn EnvironmentVariableResolver>,
) -> Result<ComputedPropertySet, Error> {
    let mut result = Matched(
        engine,
        document,
        node,
        SerializationRuleScope::Author,
        environment,
    )?;
    result.MergeAndOverrideOnConflict(inline);
    if let Ok(style) = Style(document, node) {
        let computed = CSSComputedStyleDeclaration::new(Some(style), Default::default());
        let replacements = result
            .Properties()
            .iter()
            .filter(|p| IsSerializationPercentage(p.Value()))
            .map(|p| {
                computed
                    .GetPropertyCSSValue(p.PropertyID())
                    .map(|value| (p.PropertyID(), value, p.IsImportant()))
                    .map_err(|_| Error::UnavailableComputedValue)
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (id, value, important) in replacements {
            if let Some(value) = value {
                result.SetProperty(id, value, important);
            }
        }
        if style.InForcedColorsMode() {
            for id in [P::kBackgroundColor, P::kColor, P::kTextDecorationColor] {
                result.SetProperty(id, values::identifier(V::kInitial), false);
            }
        }
        let mut resolved = Empty(document);
        for property in result.Properties() {
            let value = if matches!(
                property.Value().Payload(),
                CSSValuePayload::kUnparsedDeclarationClass(_)
                    | CSSValuePayload::kPendingSubstitutionValueClass(_)
            ) {
                engine
                    .ResolveSerializationValue(
                        document,
                        node,
                        &property.Name(),
                        property.ValueRef(),
                        environment,
                    )
                    .map_err(|_| Error::UnavailableSerializationValue)?
                    .unwrap_or_else(|| property.ValueRef())
            } else {
                property.ValueRef()
            };
            resolved.SetPropertyName(&property.Name(), value, property.IsImportant());
        }
        result = resolved;
    }
    Ok(result)
}
fn RemoveConflicting(
    document: &dom::Document,
    node: usize,
    style: &mut ComputedPropertySet,
) -> Result<(), Error> {
    if let Some(parent) = document
        .Node(node)
        .Parent()
        .filter(|p| document.Node(*p).Type() == dom::DOMNodeType::kElement)
    {
        let mut element = Capture(document, node, false, true)?;
        let parent = Capture(document, parent, false, true)?;
        element.RemoveEquivalentProperties(&parent);
        for property in element.Properties() {
            style.RemoveProperty(property.PropertyID(), None);
        }
    }
    Ok(())
}
fn AddImplicit(
    document: &dom::Document,
    node: usize,
    style: &mut ComputedPropertySet,
    inline: &ComputedPropertySet,
) -> Result<(), Error> {
    let n = document.Node(node);
    if n.Namespace() != dom::DOMNamespace::kHTML {
        return Ok(());
    }
    let equivalent = match n.Name() {
        "b" | "strong" => Some((P::kFontWeight, V::kBold)),
        "i" | "em" => Some((P::kFontStyle, V::kItalic)),
        "u" => Some((P::kTextDecorationLine, V::kUnderline)),
        "s" | "strike" => Some((P::kTextDecorationLine, V::kLineThrough)),
        "sub" => Some((P::kVerticalAlign, V::kSub)),
        "sup" => Some((P::kVerticalAlign, V::kSuper)),
        _ => None,
    };
    if let Some((property, value)) = equivalent {
        let exists = |set: &ComputedPropertySet| {
            set.HasProperty(property)
                || property == P::kTextDecorationLine
                    && set.HasProperty(P::kWebkitTextDecorationsInEffect)
        };
        if !exists(style) && !exists(inline) {
            let value = values::identifier(value);
            let value = if property == P::kTextDecorationLine {
                values::list(vec![value], values::ListSeparator::Space)
            } else {
                value
            };
            style.SetProperty(property, value, false);
        }
    }
    // HtmlAttributeEquivalent::AttributeValueAsCSSValue intentionally parses
    // the *attribute* in quirks mode. This is not a reparse of a compiled style.
    // HtmlFontSizeEquivalent uses HTMLFontElement's signed legacy size grammar.
    if n.Name() == "font" {
        for (attribute, property) in [
            ("color", P::kColor),
            ("face", P::kFontFamily),
            ("size", P::kFontSize),
        ] {
            if style.HasProperty(property) || inline.HasProperty(property) {
                continue;
            }
            let Some(attribute) = n.FindAttribute(attribute) else {
                continue;
            };
            if property == P::kFontSize {
                if let Some(value) = CssValueFromFontSizeNumber(attribute.value.as_bytes()) {
                    style.SetProperty(property, values::identifier(value), false);
                }
            } else {
                match style_parser::production_property_parser::ParseProperty(property,&foundation::String::from(attribute.value.as_str()),false,crate::parser::css_parser_mode::CSSParserMode::kHTMLQuirksMode) {
                    Ok(properties) => { style.AddParsedProperties(&properties); }
                    Err(error) if error.kind == style_parser::production_property_parser::PropertyParseErrorKind::Unsupported => return Err(Error::UnavailableSerializationValue),
                    Err(_) => {},
                }
            }
        }
    }
    Ok(())
}
/// html_font_element.cc:45–158. Preserve the signed relative-size and trailing
/// junk behavior of the HTML algorithm, including checked integer overflow.
pub(super) fn CssValueFromFontSizeNumber(input: &[u8]) -> Option<V> {
    let mut position = 0;
    while input
        .get(position)
        .is_some_and(|c| matches!(c, 9 | 10 | 12 | 13 | 32))
    {
        position += 1;
    }
    let sign = match input.get(position) {
        Some(b'+') => {
            position += 1;
            1
        }
        Some(b'-') => {
            position += 1;
            -1
        }
        _ => 0,
    };
    let start = position;
    while input.get(position).is_some_and(u8::is_ascii_digit) {
        position += 1;
    }
    if start == position {
        return None;
    }
    let mut value = std::str::from_utf8(&input[start..position])
        .ok()?
        .parse::<i32>()
        .unwrap_or(0);
    if sign == 1 {
        value = value.checked_add(3).unwrap_or(value);
    } else if sign == -1 {
        value = 3i32.checked_sub(value).unwrap_or(value);
    }
    Some(match value.clamp(1, 7) {
        1 => V::kXSmall,
        2 => V::kSmall,
        3 => V::kMedium,
        4 => V::kLarge,
        5 => V::kXLarge,
        6 => V::kXxLarge,
        _ => V::kXxxLarge,
    })
}
fn Wrapping(
    engine: &mut DocumentStyleEngine,
    document: &dom::Document,
    node: usize,
    annotate: bool,
    environment: Option<&dyn EnvironmentVariableResolver>,
) -> Result<ComputedPropertySet, Error> {
    if annotate {
        let mut result = Capture(document, node, true, true)?;
        let mut quote = Some(node);
        while let Some(n) = quote {
            if IsMailQuote(document, n) {
                if let Some(parent) = document.Node(n).Parent() {
                    let mut quote_style = Capture(document, n, false, true)?;
                    let parent_style = Capture(document, parent, false, true)?;
                    quote_style.RemoveEquivalentProperties(&parent_style);
                    result.RemoveEquivalentProperties(&quote_style);
                }
                break;
            }
            quote = document.Node(n).Parent();
        }
        CollapseDecorations(&mut result);
        return Ok(result);
    }
    let mut result = Empty(document);
    let mut current = Some(node);
    while let Some(n) = current {
        if document.Node(n).Type() == dom::DOMNodeType::kDocument {
            break;
        }
        if document.Node(n).Type() == dom::DOMNodeType::kElement && !IsMailQuote(document, n) {
            let inline = Inline(engine, document, n)?;
            let rules = MergeRulesForSerialization(engine, document, n, &inline, environment)?;
            let rules = rules.CopyPropertiesInSet(EDITING_PROPERTIES);
            MergeEditing(&mut result, &rules, false);
            AddImplicit(document, n, &mut result, &inline)?;
        }
        current = document.Node(n).Parent();
    }
    Ok(result)
}

pub fn QuerySerializationEditingStyle(
    engine: &mut DocumentStyleEngine,
    document: &dom::Document,
    operation: SerializationOp,
    environment: Option<&dyn EnvironmentVariableResolver>,
) -> Result<CSSInlineStyle, Error> {
    let result = match operation {
        SerializationOp::Wrapping { node_id, annotate } => Wrapping(
            engine,
            document,
            Node(document, node_id)?,
            annotate,
            environment,
        )?,
        SerializationOp::Inline {
            node_id,
            wrapping,
            annotate,
            sanitize,
            force_inline,
            remove_float,
        } => {
            let node = Node(document, node_id)?;
            let mut result = if let Some(wrapping) = wrapping {
                let mut result = Properties(&wrapping)?.MutableCopy();
                let defaults = Matched(
                    engine,
                    document,
                    node,
                    SerializationRuleScope::UserAgentAndUser,
                    environment,
                )?;
                for property in defaults.Properties() {
                    result.RemoveProperty(property.PropertyID(), None);
                }
                RemoveConflicting(document, node, &mut result)?;
                result
            } else {
                Empty(document)
            };
            let inline = Inline(engine, document, node)?;
            result.MergeAndOverrideOnConflict(&inline);
            if annotate && document.Node(node).Namespace() == dom::DOMNamespace::kHTML {
                result = MergeRulesForSerialization(engine, document, node, &result, environment)?;
            }
            if sanitize {
                StripUA(&mut result);
            }
            if force_inline {
                ForceInline(&mut result);
            }
            if remove_float {
                result.RemoveProperty(P::kFloat, None);
            }
            result
        }
        SerializationOp::Text { wrapping, sanitize } => {
            let mut result = Properties(&wrapping)?.MutableCopy();
            ForceInline(&mut result);
            result.SetProperty(P::kFloat, values::identifier(V::kNone), false);
            if sanitize {
                StripUA(&mut result);
            }
            result
        }
        SerializationOp::FullySelectedRoot { node_id } => {
            let node = Node(document, node_id)?;
            let inline = Inline(engine, document, node)?;
            let mut result = Matched(
                engine,
                document,
                node,
                SerializationRuleScope::Author,
                environment,
            )?;
            result.MergeAndOverrideOnConflict(&inline);
            if !result.HasProperty(P::kBackgroundImage)
                && document.Node(node).FindAttribute("background").is_some()
            {
                if let Some(image) = document
                    .BackgroundAttributeImage(node)
                    .map_err(|_| Error::UnavailableSerializationValue)?
                {
                    result.SetProperty(P::kBackgroundImage, image, false);
                }
            }
            for id in [P::kTextDecoration, P::kWebkitTextDecorationsInEffect] {
                if result.HasProperty(id) {
                    result.SetProperty(id, values::identifier(V::kNone), false);
                }
            }
            result
        }
    };
    Ok(Retain(result, Mode(document)))
}
