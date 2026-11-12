//! EditingStyle's CSS declaration algebra stays with the compiled CSSValue
//! owners. Source editing_style.cc:80-153,579-704,867-882,1353-1420.
//! Serialized CSS is only the DOM attribute projection; the returned inline
//! owner retains the same compiled immutable property set for resolution.
#![allow(non_snake_case)]
use crate::{
    css_computed_style_declaration::{CSSComputedStyleDeclaration, ComputedPropertySet},
    production_css_value::{self as values, ProductionCSSValueDispatch},
    production_style_sheet_projection::{ParsedInlineStyle, RetainInlineProperties},
};
use cssom::{
    editing_style::{EditingStyleError as Error, EditingStyleOperation as Op},
    CSSInlineStyle,
};
use foundation::{CSSPropertyID as P, CSSValueID as V, TextDirection};
use layoutng_style::style::computed_style::ComputedStyle;
use std::rc::Rc;
use style_model::{
    css_property_value_set::{CSSPropertyValueSet, ImmutableCSSPropertyValueSet},
    css_value::CSSValuePayload,
    properties::css_property::CSSProperty,
};

#[path = "editing_style_serialization.rs"]
mod serialization;
pub use serialization::QuerySerializationEditingStyle;
#[path = "editing_style_replacement.rs"]
mod replacement;
pub use replacement::QueryReplacementEditingStyle;
#[path = "editing_style_apply.rs"]
mod apply;
pub use apply::QueryApplyEditingStyle;

const EDITING_PROPERTIES: &[P] = &[
    P::kBackgroundColor,
    P::kColor,
    P::kFontFamily,
    P::kFontSize,
    P::kFontStyle,
    P::kFontVariantLigatures,
    P::kFontVariantCaps,
    P::kFontWeight,
    P::kLetterSpacing,
    P::kOrphans,
    P::kTextAlign,
    P::kTextDecorationLine,
    P::kTextIndent,
    P::kTextTransform,
    P::kWidows,
    P::kWordSpacing,
    P::kWebkitTextDecorationsInEffect,
    P::kWebkitTextFillColor,
    P::kWebkitTextStrokeColor,
    P::kWebkitTextStrokeWidth,
    P::kCaretColor,
    P::kTextWrapMode,
    P::kWhiteSpaceCollapse,
];
/// editing_style.cc:1298-1350. Only compiled declarations participate: a
/// style string is never reparsed in Interaction's insertion path.
pub fn ElementIsStyledSpanOrHtmlEquivalent(
    document: &dom::Document,
    node_id: u64,
) -> Result<bool, Error> {
    let Some(index) = document.FindNodeById(node_id) else {
        return Ok(false);
    };
    let node = document.Node(index);
    if node.Type() != dom::DOMNodeType::kElement || node.Namespace() != dom::DOMNamespace::kHTML {
        return Ok(false);
    }
    let tag = node.Name();
    let element_equivalent = matches!(
        tag,
        "span" | "b" | "strong" | "sub" | "sup" | "i" | "em" | "u" | "s" | "strike"
    );
    let attributes = node.Attributes();
    if attributes.is_empty() {
        return Ok(element_equivalent);
    }
    let mut matched = if tag == "font" {
        ["color", "face", "size"]
            .iter()
            .filter(|name| node.FindAttribute(name).is_some())
            .count()
    } else {
        0
    };
    if !element_equivalent && matched == 0 {
        return Ok(false);
    }
    if node.FindAttribute("style").is_some() {
        let properties = Properties(&node.InlineStyle())?;
        if properties
            .Properties()
            .iter()
            .any(|property| !EDITING_PROPERTIES.contains(&property.PropertyID()))
        {
            return Ok(false);
        }
        matched += 1;
    }
    Ok(matched >= attributes.len())
}
fn StyledNode(document: &dom::Document, node_id: u64) -> Option<usize> {
    let mut node = document.FindNodeById(node_id)?;
    loop {
        if document
            .ResolvedStyleFor(node)
            .and_then(|style| style.ComputedStyle())
            .is_some()
        {
            return Some(node);
        }
        node = document.Node(node).Parent()?;
    }
}
fn Style(document: &dom::Document, node: usize) -> Result<&ComputedStyle, Error> {
    document
        .ResolvedStyleFor(node)
        .and_then(|style| style.ComputedStyle())
        .ok_or(Error::MissingStyle)
}
fn Properties(
    declaration: &CSSInlineStyle,
) -> Result<Rc<CSSPropertyValueSet<ProductionCSSValueDispatch>>, Error> {
    declaration
        .ParsedBacking::<ParsedInlineStyle>()
        .map(|parsed| parsed.Properties())
        .ok_or(Error::MissingCompiledDeclaration)
}
fn Retain(
    properties: ComputedPropertySet,
    mode: crate::parser::css_parser_mode::CSSParserMode,
) -> CSSInlineStyle {
    let source = properties.AsText();
    let result = CSSInlineStyle::FromUTF16(source.Span16().unwrap_or_default());
    let immutable = ImmutableCSSPropertyValueSet::Create(properties.Properties(), mode, false);
    RetainInlineProperties(&result, immutable, mode);
    result
}
fn IsTransparent(value: &values::Value) -> bool {
    match value.Payload() {
        CSSValuePayload::kColorClass(color) => color.0.Alpha() == 0.0,
        CSSValuePayload::kIdentifierClass(id) => id.0 == V::kTransparent,
        _ => false,
    }
}
fn BackgroundInEffect(
    document: &dom::Document,
    mut node: usize,
) -> Result<Option<Rc<values::Value>>, Error> {
    loop {
        if let Ok(style) = Style(document, node) {
            let declaration = CSSComputedStyleDeclaration::new(Some(style), Default::default());
            let value = declaration
                .GetPropertyCSSValue(P::kBackgroundColor)
                .map_err(|_| Error::UnavailableComputedValue)?;
            if let Some(value) = value.filter(|value| !IsTransparent(value)) {
                return Ok(Some(value));
            }
        }
        let Some(parent) = document.Node(node).Parent() else {
            return Ok(None);
        };
        node = parent;
    }
}
fn Capture(
    document: &dom::Document,
    node: usize,
    in_effect: bool,
    all: bool,
) -> Result<ComputedPropertySet, Error> {
    let style = Style(document, node)?;
    let declaration = CSSComputedStyleDeclaration::new(Some(style), Default::default());
    let properties: Vec<_> = EDITING_PROPERTIES
        .iter()
        .copied()
        .filter(|property| all || CSSProperty::Get(*property).IsInherited())
        .collect();
    let mut result = declaration
        .CopyPropertiesInSet(&properties)
        .map_err(|_| Error::UnavailableComputedValue)?;
    if in_effect {
        if let Some(background) = BackgroundInEffect(document, node)? {
            result.SetProperty(P::kBackgroundColor, background, false);
        }
        if let Some(decoration) = declaration
            .GetPropertyCSSValue(P::kWebkitTextDecorationsInEffect)
            .map_err(|_| Error::UnavailableComputedValue)?
        {
            result.SetProperty(P::kTextDecorationLine, decoration, false);
        }
    }
    if style.ComputedFontSize() != style.SpecifiedFontSize() {
        result.SetProperty(
            P::kFontSize,
            values::numeric(
                style.SpecifiedFontSize() as f64,
                crate::css_primitive_value::UnitType::kPixels,
            ),
            false,
        );
    }
    if let Some(keyword) = declaration.GetFontSizeCSSValuePreferringKeyword() {
        result.SetProperty(P::kFontSize, keyword, false);
    }
    if style.InForcedColorsMode() {
        result.RemoveProperty(P::kColor, None);
        result.RemoveProperty(P::kBackgroundColor, None);
        result.RemoveProperty(P::kTextDecorationColor, None);
    }
    if style.TextFillColor().IsCurrentColor() {
        result.RemoveProperty(P::kWebkitTextFillColor, None);
    }
    if style.TextStrokeColor().IsCurrentColor() {
        result.RemoveProperty(P::kWebkitTextStrokeColor, None);
    }
    if style.CaretColor().IsAutoColor() || style.CaretColor().IsCurrentColor() {
        result.RemoveProperty(P::kCaretColor, None);
    }
    Ok(result)
}
fn Align(
    set: &CSSPropertyValueSet<ProductionCSSValueDispatch>,
    direction: TextDirection,
) -> Option<V> {
    let CSSValuePayload::kIdentifierClass(id) = set.GetPropertyCSSValue(P::kTextAlign)?.Payload()
    else {
        return None;
    };
    Some(match id.0 {
        V::kWebkitLeft => V::kLeft,
        V::kWebkitRight => V::kRight,
        V::kWebkitCenter => V::kCenter,
        V::kStart => {
            if direction == TextDirection::kLtr {
                V::kLeft
            } else {
                V::kRight
            }
        }
        V::kEnd => {
            if direction == TextDirection::kLtr {
                V::kRight
            } else {
                V::kLeft
            }
        }
        value => value,
    })
}
/// StyleChange::MaybeApplyOriginalHtmlEquivalentTags/ExtractTextStyles,
/// editing_style.cc:1870-1949,1976-2036. Compare native CSSValues and retain
/// the remaining compiled declaration; never serialize and parse it again.
pub fn QueryEditingStyleApplication(
    document: &dom::Document,
    declaration: CSSInlineStyle,
    originals: &[(P, V, foundation::QualifiedName)],
) -> Result<cssom::editing_style::EditingStyleApplication, Error> {
    let mut result = Properties(&declaration)?.MutableCopy();
    let mut tags = Vec::new();
    let identifier = |set: &ComputedPropertySet, property| match set
        .GetPropertyCSSValue(property)
        .map(|value| value.Payload())
    {
        Some(CSSValuePayload::kIdentifierClass(value)) => Some(value.0),
        _ => None,
    };
    let sub = originals
        .iter()
        .find(|(p, v, _)| *p == P::kVerticalAlign && *v == V::kSub);
    let sup = originals
        .iter()
        .find(|(p, v, _)| *p == P::kVerticalAlign && *v == V::kSuper);
    let vertical = identifier(&result, P::kVerticalAlign);
    let original = match vertical {
        Some(V::kSub) => sub,
        Some(V::kSuper) => sup,
        None => {
            if sub.is_some() && sup.is_none() {
                sub
            } else if sup.is_some() && sub.is_none() {
                sup
            } else {
                None
            }
        }
        _ => None,
    };
    if let Some((_, _, tag)) = original {
        tags.push(tag.clone());
        result.RemoveProperty(P::kVerticalAlign, None);
        result.RemoveProperty(P::kFontSize, None);
    }
    // ApplyStyleCommand surrounds the CSS span in bold, italic, underline,
    // line-through, sub/sup order (cc:2043-2097). Return outside-to-inside
    // order for insertion at the caret, preserving the same final hierarchy.
    for (property_id, requested_value) in [
        (P::kTextDecorationLine, V::kLineThrough),
        (P::kTextDecorationLine, V::kUnderline),
        (P::kFontStyle, V::kItalic),
        (P::kFontWeight, V::kBold),
    ] {
        let Some((property, value_id, tag)) = originals
            .iter()
            .find(|(property, value, _)| *property == property_id && *value == requested_value)
        else {
            continue;
        };
        let apply = match (*property, *value_id) {
            (P::kFontWeight, V::kBold) => {
                result
                    .GetPropertyCSSValue(*property)
                    .is_some_and(|value| match value.Payload() {
                        CSSValuePayload::kIdentifierClass(value) => value.0 == V::kBold,
                        CSSValuePayload::kNumericLiteralClass(value) => {
                            value.GetValueIfKnown().is_some_and(|value| value >= 600.0)
                        }
                        _ => false,
                    })
            }
            (P::kFontStyle, V::kItalic) => matches!(
                identifier(&result, *property),
                Some(V::kItalic | V::kOblique)
            ),
            (P::kTextDecorationLine, V::kUnderline | V::kLineThrough) => result
                .GetPropertyCSSValue(*property)
                .is_some_and(|value| match value.Payload() {
                    CSSValuePayload::kValueListClass(list) => {
                        list.HasValue(values::identifier(*value_id).as_ref())
                    }
                    CSSValuePayload::kIdentifierClass(value) => value.0 == *value_id,
                    _ => false,
                }),
            _ => false,
        };
        if !apply {
            continue;
        }
        tags.push(tag.clone());
        if *property == P::kTextDecorationLine {
            let replacement =
                result
                    .GetPropertyCSSValue(*property)
                    .and_then(|value| match value.Payload() {
                        CSSValuePayload::kValueListClass(list) => {
                            let mut list = list.Copy();
                            list.RemoveAll(values::identifier(*value_id).as_ref());
                            (!list.values.is_empty())
                                .then(|| values::list(list.values, list.separator))
                        }
                        _ => None,
                    });
            result.RemoveProperty(*property, None);
            if let Some(value) = replacement {
                result.SetProperty(*property, value, false);
            }
        } else {
            result.RemoveProperty(*property, None);
        }
    }
    let mode = if document.GetCompatibilityMode()
        == dom::persistent_document::CompatibilityMode::kQuirks
    {
        crate::parser::css_parser_mode::CSSParserMode::kHTMLQuirksMode
    } else {
        crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode
    };
    Ok(cssom::editing_style::EditingStyleApplication {
        declaration: Retain(result, mode),
        tags,
    })
}
pub fn QueryEditingStyle(document: &dom::Document, operation: Op) -> Result<CSSInlineStyle, Error> {
    let mode = if document.GetCompatibilityMode()
        == dom::persistent_document::CompatibilityMode::kQuirks
    {
        crate::parser::css_parser_mode::CSSParserMode::kHTMLQuirksMode
    } else {
        crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode
    };
    match operation {
        Op::Capture { node_id, in_effect } => Ok(Retain(
            Capture(
                document,
                StyledNode(document, node_id).ok_or(Error::MissingStyle)?,
                in_effect,
                false,
            )?,
            mode,
        )),
        Op::Merge {
            base,
            override_style,
        } => {
            let mut result = Properties(&base)?.MutableCopy();
            let override_properties = Properties(&override_style)?;
            result.MergeAndOverrideOnConflict(override_properties.as_ref());
            Ok(Retain(result, mode))
        }
        Op::Prepare {
            node_id,
            declaration,
        } => {
            let node = StyledNode(document, node_id).ok_or(Error::MissingStyle)?;
            let base = Capture(document, node, true, false)?;
            let mut result = Properties(&declaration)?.MutableCopy();
            let bidi = result.GetPropertyValueRef(P::kUnicodeBidi);
            let direction = result.GetPropertyValueRef(P::kDirection);
            result.RemoveEquivalentProperties(&base);
            if Align(&result, Style(document, node)?.Direction())
                .zip(Align(&base, Style(document, node)?.Direction()))
                .is_some_and(|(left, right)| left == right)
            {
                result.RemoveProperty(P::kTextAlign, None);
            }
            if result
                .GetPropertyCSSValue(P::kBackgroundColor)
                .is_some_and(|value| {
                    IsTransparent(value)
                        || BackgroundInEffect(document, node)
                            .ok()
                            .flatten()
                            .as_ref()
                            .is_some_and(|background| background.as_ref() == value)
                })
            {
                result.RemoveProperty(P::kBackgroundColor, None);
            }
            if let Some(bidi) = bidi {
                result.SetProperty(P::kUnicodeBidi, bidi, false);
            }
            if let Some(direction) = direction {
                result.SetProperty(P::kDirection, direction, false);
            }
            Ok(Retain(result, mode))
        }
        Op::RemoveStyleAddedByElement {
            node_id,
            declaration,
        } => {
            let Some(node) = document.FindNodeById(node_id) else {
                return Err(Error::MissingStyle);
            };
            let Some(parent) = document.Node(node).Parent() else {
                return Ok(declaration);
            };
            let parent =
                StyledNode(document, document.Node(parent).Id()).ok_or(Error::MissingStyle)?;
            let mut element = Capture(document, node, false, true)?;
            let parent = Capture(document, parent, false, true)?;
            element.RemoveEquivalentProperties(&parent);
            let mut result = Properties(&declaration)?.MutableCopy();
            result.RemoveEquivalentProperties(&element);
            Ok(Retain(result, mode))
        }
    }
}
