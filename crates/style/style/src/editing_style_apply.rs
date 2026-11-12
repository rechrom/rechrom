// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! EditingStyle/StyleChange operations consumed by translated ApplyStyleCommand.
use super::*;
use cssom::editing_style::{
    ApplyStyleChange, ApplyStyleConflict, ApplyStyleExtraction as Extract,
    ApplyStyleOperation as Op, ApplyStyleResult as Out,
};
fn Mode(d: &dom::Document) -> crate::parser::css_parser_mode::CSSParserMode {
    if d.GetCompatibilityMode() == dom::persistent_document::CompatibilityMode::kQuirks {
        crate::parser::css_parser_mode::CSSParserMode::kHTMLQuirksMode
    } else {
        crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode
    }
}
fn Empty(d: &dom::Document) -> ComputedPropertySet {
    ComputedPropertySet::new(Mode(d))
}
fn Node(d: &dom::Document, id: u64) -> Result<usize, Error> {
    d.FindNodeById(id).ok_or(Error::MissingStyle)
}
fn Name(name: &str) -> foundation::QualifiedName {
    foundation::QualifiedName::new(
        Default::default(),
        foundation::AtomicString::from_str(name),
        Default::default(),
    )
}
fn Identifier(set: &CSSPropertyValueSet<ProductionCSSValueDispatch>, p: P) -> Option<V> {
    match set.GetPropertyCSSValue(p)?.Payload() {
        CSSValuePayload::kIdentifierClass(v) => Some(v.0),
        _ => None,
    }
}
fn Direction(set: &CSSPropertyValueSet<ProductionCSSValueDispatch>) -> Option<TextDirection> {
    match Identifier(set, P::kDirection) {
        Some(V::kLtr) => Some(TextDirection::kLtr),
        Some(V::kRtl) => Some(TextDirection::kRtl),
        _ => None,
    }
}
const BLOCK: &[P] = &[
    P::kBreakAfter,
    P::kBreakBefore,
    P::kBreakInside,
    P::kOrphans,
    P::kOverflow,
    P::kColumnCount,
    P::kColumnGap,
    P::kColumnRuleColor,
    P::kColumnRuleStyle,
    P::kColumnRuleWidth,
    P::kRowRuleWidth,
    P::kWebkitColumnBreakBefore,
    P::kWebkitColumnBreakAfter,
    P::kWebkitColumnBreakInside,
    P::kColumnWidth,
    P::kPageBreakAfter,
    P::kPageBreakBefore,
    P::kPageBreakInside,
    P::kTextAlign,
    P::kTextAlignLast,
    P::kTextIndent,
    P::kWidows,
];
fn Equivalent(n: &str) -> Option<(P, V)> {
    Some(match n {
        "b" | "strong" => (P::kFontWeight, V::kBold),
        "i" | "em" => (P::kFontStyle, V::kItalic),
        "u" => (P::kTextDecorationLine, V::kUnderline),
        "s" | "strike" => (P::kTextDecorationLine, V::kLineThrough),
        "sub" => (P::kVerticalAlign, V::kSub),
        "sup" => (P::kVerticalAlign, V::kSuper),
        _ => return None,
    })
}
fn ImplicitValue(
    document: &dom::Document,
    node: usize,
    attribute: &str,
    property: P,
) -> Result<Option<Rc<values::Value>>, Error> {
    let Some(attribute) = document.Node(node).FindAttribute(attribute) else {
        return Ok(None);
    };
    if property == P::kFontSize {
        return Ok(
            serialization::CssValueFromFontSizeNumber(attribute.value.as_bytes())
                .map(values::identifier),
        );
    }
    if property == P::kDirection {
        return Ok(match attribute.value.to_ascii_lowercase().as_str() {
            "ltr" => Some(values::identifier(V::kLtr)),
            "rtl" => Some(values::identifier(V::kRtl)),
            _ => None,
        });
    }
    if property == P::kUnicodeBidi {
        return Ok(Some(values::identifier(
            if document.Node(node).IsHTMLElement("bdo") {
                V::kBidiOverride
            } else {
                V::kIsolate
            },
        )));
    }
    match style_parser::production_property_parser::ParseProperty(
        property,
        &foundation::String::from(attribute.value.as_str()),
        false,
        Mode(document),
    ) {
        Ok(p) => Ok(p
            .into_iter()
            .find(|p| p.PropertyID() == property)
            .map(|p| p.ValueRef())),
        Err(e)
            if e.kind
                == style_parser::production_property_parser::PropertyParseErrorKind::Invalid =>
        {
            Ok(None)
        }
        Err(_) => Err(Error::UnavailableComputedValue),
    }
}
fn Conflicts(
    document: &dom::Document,
    node: usize,
    declaration: &CSSInlineStyle,
    matching: bool,
    extract: bool,
    preserve: bool,
) -> Result<ApplyStyleConflict, Error> {
    let source = Properties(declaration)?;
    let n = document.Node(node);
    let mut extracted = Empty(document);
    let mut implicit_element = false;
    if let Some((p, v)) = Equivalent(n.Name()) {
        let actual = if p == P::kTextDecorationLine {
            source
                .GetPropertyCSSValue(P::kWebkitTextDecorationsInEffect)
                .or_else(|| source.GetPropertyCSSValue(p))
        } else {
            source.GetPropertyCSSValue(p)
        };
        let same=actual.is_some_and(|value|if p==P::kTextDecorationLine {matches!(value.Payload(),CSSValuePayload::kValueListClass(list) if list.HasValue(values::identifier(v).as_ref()))}else{value==values::identifier(v).as_ref()});
        implicit_element = actual.is_some() && (matching || !same);
        if implicit_element && extract {
            extracted.SetProperty(p, values::identifier(v), false);
        }
    }
    let mut attributes = Vec::new();
    if !implicit_element {
        for (attribute, p) in [
            ("color", P::kColor),
            ("face", P::kFontFamily),
            ("size", P::kFontSize),
            ("dir", P::kDirection),
            ("dir", P::kUnicodeBidi),
        ] {
            if attribute != "dir" && !n.IsHTMLElement("font") || attribute == "dir" && preserve {
                continue;
            }
            let Some(requested) = source.GetPropertyCSSValue(p) else {
                continue;
            };
            let Some(implicit) = ImplicitValue(document, node, attribute, p)? else {
                continue;
            };
            if matching || implicit.as_ref() != requested {
                if !attributes
                    .iter()
                    .any(|name: &foundation::QualifiedName| name.LocalName().Utf8() == attribute)
                {
                    attributes.push(Name(attribute));
                }
                if extract {
                    extracted.SetProperty(p, implicit, false);
                }
            }
        }
    }
    let inline = if n.FindAttribute("style").is_some() {
        Properties(&n.InlineStyle())?.MutableCopy()
    } else {
        Empty(document)
    };
    let mut properties = Vec::new();
    for property in source.Properties() {
        let p = property.PropertyID();
        let Some(value) = inline.GetPropertyCSSValue(p) else {
            continue;
        };
        properties.push(p);
        if extract {
            extracted.SetProperty(
                p,
                inline
                    .GetPropertyValueRef(p)
                    .ok_or(Error::MissingCompiledDeclaration)?,
                inline.PropertyIsImportant(p),
            );
        }
    }
    Ok(ApplyStyleConflict {
        implicit_element,
        attributes,
        properties,
        extracted: Retain(extracted, Mode(document)),
    })
}
fn Change(
    document: &dom::Document,
    node: usize,
    declaration: CSSInlineStyle,
    originals: &[(P, V, foundation::QualifiedName)],
    css: bool,
) -> Result<ApplyStyleChange, Error> {
    let computed = Capture(
        document,
        StyledNode(document, document.Node(node).Id()).ok_or(Error::MissingStyle)?,
        true,
        true,
    )?;
    let mut difference = replacement::PropertiesNotIn(
        document,
        node,
        Properties(&declaration)?.as_ref(),
        &computed,
    )?;
    if let Some(value) = difference.GetPropertyValueRef(P::kWebkitTextDecorationsInEffect) {
        difference.SetProperty(P::kTextDecorationLine, value, false);
        difference.RemoveProperty(P::kWebkitTextDecorationsInEffect, None);
    }
    if difference
        .GetPropertyCSSValue(P::kTextDecorationLine)
        .is_some_and(|v| !matches!(v.Payload(), CSSValuePayload::kValueListClass(_)))
    {
        difference.RemoveProperty(P::kTextDecorationLine, None);
    }
    let application =
        QueryEditingStyleApplication(document, Retain(difference, Mode(document)), originals)?;
    let mut properties = Properties(&application.declaration)?.MutableCopy();
    let mut tags = application.tags;
    if !css {
        let mut add = |name: &str| {
            tags.insert(
                0,
                foundation::QualifiedName::new(
                    Default::default(),
                    foundation::AtomicString::from_str(name),
                    foundation::AtomicString::from_str("http://www.w3.org/1999/xhtml"),
                ),
            )
        };
        if properties
            .GetPropertyCSSValue(P::kFontWeight)
            .is_some_and(|v| match v.Payload() {
                CSSValuePayload::kIdentifierClass(v) => v.0 == V::kBold,
                CSSValuePayload::kNumericLiteralClass(v) => {
                    v.GetValueIfKnown().is_some_and(|v| v >= 600.0)
                }
                _ => false,
            })
        {
            add("b");
            properties.RemoveProperty(P::kFontWeight, None);
        }
        if matches!(
            Identifier(&properties, P::kFontStyle),
            Some(V::kItalic | V::kOblique)
        ) {
            add("i");
            properties.RemoveProperty(P::kFontStyle, None);
        }
        if let Some(CSSValuePayload::kValueListClass(list)) = properties
            .GetPropertyCSSValue(P::kTextDecorationLine)
            .map(|v| v.Payload())
        {
            let mut remaining = list.Copy();
            for (value, tag) in [(V::kUnderline, "u"), (V::kLineThrough, "strike")] {
                if remaining.RemoveAll(values::identifier(value).as_ref()) {
                    add(tag);
                }
            }
            properties.RemoveProperty(P::kTextDecorationLine, None);
            if !remaining.values.is_empty() {
                properties.SetProperty(
                    P::kTextDecorationLine,
                    values::list(remaining.values, remaining.separator),
                    false,
                );
            }
        }
        match Identifier(&properties, P::kVerticalAlign) {
            Some(V::kSub) => {
                add("sub");
                properties.RemoveProperty(P::kVerticalAlign, None);
            }
            Some(V::kSuper) => {
                add("sup");
                properties.RemoveProperty(P::kVerticalAlign, None);
            }
            _ => {}
        }
    }
    // ApplyInlineStyleChange has one fixed source order regardless of whether
    // the flag came from an original equivalent tag or legacy extraction.
    tags.sort_by_key(|tag| match tag.LocalName().Utf8().as_str() {
        "sub" | "sup" => 0,
        "s" | "strike" => 1,
        "u" => 2,
        "i" | "em" => 3,
        "b" | "strong" => 4,
        _ => 5,
    });
    let mut font = Vec::new();
    if !css {
        if let Some(value) = properties.GetPropertyCSSValue(P::kColor) {
            let color = match value.Payload() {
                CSSValuePayload::kColorClass(v) => v.0,
                CSSValuePayload::kIdentifierClass(v) => {
                    values::NamedColor(v.0).unwrap_or(foundation::Color::kTransparent)
                }
                _ => return Err(Error::UnavailableComputedValue),
            };
            let text = if color.IsOpaque()
                && foundation::Color::IsLegacyColorSpace(color.GetColorSpace())
            {
                format!("#{:06x}", color.Rgb() & 0xFFFFFF)
            } else {
                values::color(color).CssText().Utf8()
            };
            font.push((
                Name("color"),
                Rc::from(text.encode_utf16().collect::<Vec<_>>()),
            ));
            properties.RemoveProperty(P::kColor, None);
        }
        if let Some(value) = properties.GetPropertyCSSValue(P::kFontFamily) {
            let text = value.CssText().Utf8().replace('"', "");
            if !text.is_empty() {
                font.push((
                    Name("face"),
                    Rc::from(text.encode_utf16().collect::<Vec<_>>()),
                ));
            }
            properties.RemoveProperty(P::kFontFamily, None);
        }
        if let Some(value) = properties.GetPropertyCSSValue(P::kFontSize) {
            let size = LegacyFontSize(document, node, value, false);
            if let Some(size) = size {
                font.push((
                    Name("size"),
                    Rc::from(size.to_string().encode_utf16().collect::<Vec<_>>()),
                ));
                properties.RemoveProperty(P::kFontSize, None);
            }
        }
    }
    Ok(ApplyStyleChange {
        declaration: Retain(properties, Mode(document)),
        tags,
        font_attributes: font,
    })
}
fn LegacyFontSize(
    document: &dom::Document,
    node: usize,
    value: &values::Value,
    always: bool,
) -> Option<i32> {
    use layoutng_style::css::font_size_functions::{
        self as size, FontSizeDocumentInputs, FontSizeSettings,
    };
    let mono = Style(document, StyledNode(document, document.Node(node).Id())?)
        .ok()?
        .GetFontDescription()
        .IsMonospace();
    let inputs = FontSizeDocumentInputs {
        settings: Some(FontSizeSettings {
            default_font_size: document.DefaultFontSize() as i32,
            default_fixed_font_size: document.DefaultFixedFontSize() as i32,
            minimum_logical_font_size: document.MinimumLogicalFontSize(),
            accessibility_scale_factor: document.AccessibilityFontScaleFactor(),
        }),
        quirks_mode: document.GetCompatibilityMode()
            == dom::persistent_document::CompatibilityMode::kQuirks,
        text_scale_meta_tag_present: document.TextScaleMetaTagPresent(),
    };
    match value.Payload() {
        CSSValuePayload::kIdentifierClass(v) => match v.0 {
            V::kXSmall => Some(1),
            V::kSmall => Some(2),
            V::kMedium => Some(3),
            V::kLarge => Some(4),
            V::kXLarge => Some(5),
            V::kXxLarge => Some(6),
            V::kXxxLarge | V::kWebkitXxxLarge => Some(7),
            _ => None,
        },
        CSSValuePayload::kNumericLiteralClass(v) if v.IsLength() => {
            use crate::css_primitive_value::{
                ConversionToCanonicalUnitsScaleFactor as Scale, UnitType as U,
            };
            if !matches!(
                v.GetType(),
                U::kPixels
                    | U::kCentimeters
                    | U::kMillimeters
                    | U::kQuarterMillimeters
                    | U::kInches
                    | U::kPoints
                    | U::kPicas
            ) {
                return None;
            }
            let pixels = (v.GetValueIfKnown()? * Scale(v.GetType())) as i32;
            let legacy = size::LegacyFontSize(inputs, pixels, mono);
            (always || size::FontSizeForKeyword(inputs, legacy as usize, mono) == pixels as f32)
                .then_some(legacy)
        }
        _ => None,
    }
}
pub fn QueryApplyEditingStyle(document: &dom::Document, operation: Op) -> Result<Out, Error> {
    Ok(match operation {
        Op::Empty => Out::Declaration(Retain(Empty(document), Mode(document))),
        Op::Create { properties } => {
            use cssom::editing_style::EditingStylePropertyValue as Value;
            let mode = crate::parser::css_parser_mode::CSSParserMode::kHTMLQuirksMode;
            let mut result = ComputedPropertySet::new(mode);
            for (property, value) in properties {
                match value {
                    Value::Identifier(value) => {
                        result.SetProperty(property, values::identifier(value), false);
                    }
                    Value::LegacyFontSize(value) => {
                        let Some(size) = serialization::CssValueFromFontSizeNumber(value.as_bytes()) else {
                            return Ok(Out::InvalidValue);
                        };
                        result.SetProperty(property, values::identifier(size), false);
                    }
                    Value::Text(value) => {
                        match style_parser::production_property_parser::ParseProperty(
                            property, &foundation::String::from(value.as_str()), false, mode,
                        ) {
                            Ok(properties) => for property in properties {
                                result.SetProperty(property.PropertyID(), property.ValueRef(), false);
                            },
                            Err(error) if error.kind == style_parser::production_property_parser::PropertyParseErrorKind::Invalid => {},
                            Err(_) => return Err(Error::UnavailableComputedValue),
                        }
                    }
                }
            }
            Out::Declaration(Retain(result, mode))
        }
        Op::CommandStart { node_id, typing } => {
            let node = StyledNode(document, node_id).ok_or(Error::MissingStyle)?;
            let mut result = Capture(document, node, true, true)?;
            let computed =
                CSSComputedStyleDeclaration::new(Some(Style(document, node)?), Default::default());
            if let Some(value) = computed
                .GetPropertyCSSValue(P::kVerticalAlign)
                .map_err(|_| Error::UnavailableComputedValue)?
            {
                result.SetProperty(P::kVerticalAlign, value, false);
            }
            if let Some(typing) = typing {
                result.MergeAndOverrideOnConflict(Properties(&typing)?.as_ref());
            }
            Out::Declaration(Retain(result, Mode(document)))
        }
        Op::CommandValue {
            node_id,
            property,
            typing,
        } => {
            let node = StyledNode(document, node_id).ok_or(Error::MissingStyle)?;
            let Out::Declaration(captured) =
                QueryApplyEditingStyle(document, Op::CommandStart { node_id, typing })?
            else {
                return Err(Error::UnavailableComputedValue);
            };
            let captured = Properties(&captured)?;
            let value = if property == P::kFontSize {
                captured
                    .GetPropertyCSSValue(property)
                    .and_then(|value| LegacyFontSize(document, node, value, true))
                    .unwrap_or(0)
                    .to_string()
            } else {
                captured.GetPropertyValue(property).as_str().to_owned()
            };
            Out::Text(value)
        }
        Op::CommandPresent {
            node_id,
            declaration,
            typing,
        } => {
            let node = StyledNode(document, node_id).ok_or(Error::MissingStyle)?;
            let Out::Declaration(captured) =
                QueryApplyEditingStyle(document, Op::CommandStart { node_id, typing })?
            else {
                return Err(Error::UnavailableComputedValue);
            };
            let mut captured = Properties(&captured)?.MutableCopy();
            let desired = Properties(&declaration)?;
            if let Some(value @ (V::kSub | V::kSuper)) = Identifier(&desired, P::kVerticalAlign) {
                let computed = CSSComputedStyleDeclaration::new(
                    Some(Style(document, node)?),
                    Default::default(),
                );
                let baseline = computed.GetPropertyCSSValue(P::kVerticalAlign)
                    .map_err(|_| Error::UnavailableComputedValue)?
                    .is_some_and(|value| matches!(value.Payload(), CSSValuePayload::kIdentifierClass(v) if v.0 == V::kBaseline));
                if baseline {
                    // EditingStyleUtilities::HasAncestorVerticalAlignStyle
                    // searches for the requested value, including nested
                    // sub/sup declarations with different values in between.
                    let mut current = Some(node);
                    while let Some(index) = current {
                        if let Ok(style) = Style(document, index) {
                            let computed =
                                CSSComputedStyleDeclaration::new(Some(style), Default::default());
                            if computed.GetPropertyCSSValue(P::kVerticalAlign)
                                .map_err(|_| Error::UnavailableComputedValue)?
                                .is_some_and(|actual| matches!(actual.Payload(), CSSValuePayload::kIdentifierClass(v) if v.0 == value)) {
                                captured.SetProperty(P::kVerticalAlign, values::identifier(value), false);
                                break;
                            }
                        }
                        current = document.Node(index).Parent();
                    }
                }
            }
            Out::Bool(
                replacement::PropertiesNotIn(document, node, desired.as_ref(), &captured)?
                    .IsEmpty(),
            )
        }
        Op::ToggleList {
            declaration,
            property,
            value,
        } => {
            let source = Properties(&declaration)?;
            let value = values::identifier(value);
            let replacement = match source.GetPropertyCSSValue(property).map(|v| v.Payload()) {
                Some(CSSValuePayload::kValueListClass(list)) => {
                    let mut list = list.Copy();
                    if !list.RemoveAll(value.as_ref()) {
                        list.Append(value);
                    }
                    if list.values.is_empty() {
                        values::identifier(V::kNone)
                    } else {
                        values::list(list.values, list.separator)
                    }
                }
                Some(CSSValuePayload::kIdentifierClass(v)) if v.0 == V::kNone => {
                    values::list(vec![value], values::ListSeparator::Space)
                }
                _ => values::identifier(V::kNone),
            };
            let mut result = Empty(document);
            let is_none = matches!(replacement.Payload(), CSSValuePayload::kIdentifierClass(v) if v.0 == V::kNone);
            result.SetProperty(property, replacement, false);
            Out::ToggledList {
                declaration: Retain(result, Mode(document)),
                is_none,
            }
        }
        Op::Count { declaration } => Out::Count(Properties(&declaration)?.PropertyCount()),
        Op::Extract { declaration, kind } => {
            let mut remaining = Properties(&declaration)?.MutableCopy();
            let mut extracted = Empty(document);
            let properties = match kind {
                Extract::Block => BLOCK,
                Extract::Direction => &[P::kUnicodeBidi, P::kDirection],
                Extract::Container => &[P::kBackground, P::kBackgroundColor],
            };
            for p in properties {
                if let Some(value) = remaining.GetPropertyValueRef(*p) {
                    extracted.SetProperty(*p, value, remaining.PropertyIsImportant(*p));
                    remaining.RemoveProperty(*p, None);
                }
            }
            if matches!(kind, Extract::Direction) {
                extracted.SetProperty(P::kUnicodeBidi, values::identifier(V::kIsolate), false);
            }
            Out::Extract {
                extracted: Retain(extracted, Mode(document)),
                remaining: Retain(remaining, Mode(document)),
            }
        }
        Op::Merge { base, overriding } => {
            let mut result = Properties(&base)?.MutableCopy();
            result.MergeAndOverrideOnConflict(Properties(&overriding)?.as_ref());
            Out::Declaration(Retain(result, Mode(document)))
        }
        Op::RemoveProperties {
            declaration,
            properties,
        } => {
            let mut result = Properties(&declaration)?.MutableCopy();
            for p in properties {
                result.RemoveProperty(p, None);
            }
            Out::Declaration(Retain(result, Mode(document)))
        }
        Op::RemovePresentProperties {
            declaration,
            present,
        } => {
            let mut result = Properties(&declaration)?.MutableCopy();
            for p in Properties(&present)?.Properties() {
                result.RemoveProperty(p.PropertyID(), None);
            }
            Out::Declaration(Retain(result, Mode(document)))
        }
        Op::Conflicts {
            node_id,
            declaration,
            matching,
            extract,
            preserve_direction,
        } => Out::Conflict(Conflicts(
            document,
            Node(document, node_id)?,
            &declaration,
            matching,
            extract,
            preserve_direction,
        )?),
        Op::Present {
            node_id,
            declaration,
        } => {
            let node = StyledNode(document, node_id).ok_or(Error::MissingStyle)?;
            let captured = Capture(document, node, true, true)?;
            Out::Bool(
                replacement::PropertiesNotIn(
                    document,
                    node,
                    Properties(&declaration)?.as_ref(),
                    &captured,
                )?
                .IsEmpty(),
            )
        }
        Op::Change {
            node_id,
            declaration,
            originals,
            css,
        } => Out::Change(Change(
            document,
            Node(document, node_id)?,
            declaration,
            &originals,
            css,
        )?),
        Op::FontDelta { declaration } => {
            let mut result = Properties(&declaration)?.MutableCopy();
            let delta = if result.HasProperty(P::kFontSize) {
                0.0
            } else {
                match result
                    .GetPropertyCSSValue(P::kInternalFontSizeDelta)
                    .map(|v| v.Payload())
                {
                    Some(CSSValuePayload::kNumericLiteralClass(v))
                        if v.UsesUnit(crate::css_primitive_value::UnitType::kPixels) =>
                    {
                        v.GetValueIfKnown().unwrap_or(0.0) as f32
                    }
                    _ => 0.0,
                }
            };
            result.RemoveProperty(P::kInternalFontSizeDelta, None);
            Out::FontDelta {
                delta,
                declaration: Retain(result, Mode(document)),
            }
        }
        Op::FontSize { node_id } => Out::Number(
            Style(
                document,
                StyledNode(document, node_id).ok_or(Error::MissingStyle)?,
            )?
            .ComputedFontSize(),
        ),
        Op::SetFontSize {
            declaration,
            pixels,
        } => {
            let mut result = Properties(&declaration)?.MutableCopy();
            result.SetProperty(
                P::kFontSize,
                values::numeric(pixels as f64, crate::css_primitive_value::UnitType::kPixels),
                false,
            );
            Out::Declaration(Retain(result, Mode(document)))
        }
        Op::Bidi { node_id } => {
            let node = StyledNode(document, node_id).ok_or(Error::MissingStyle)?;
            let style =
                CSSComputedStyleDeclaration::new(Some(Style(document, node)?), Default::default());
            let set = style
                .CopyPropertiesInSet(&[P::kUnicodeBidi, P::kDirection])
                .map_err(|_| Error::UnavailableComputedValue)?;
            Out::Bidi {
                unicode_bidi: Identifier(&set, P::kUnicodeBidi).unwrap_or(V::kInvalid),
                direction: Direction(&set),
            }
        }
        Op::StyleDirection { declaration } => Out::Direction({
            let properties = Properties(&declaration)?;
            match Identifier(&properties, P::kUnicodeBidi) {
                Some(V::kNormal) => Some(None),
                Some(V::kEmbed | V::kIsolate) => Direction(&properties).map(Some),
                _ => None,
            }
        }),
        Op::NormalBidi { declaration } => {
            let mut result = Properties(&declaration)?.MutableCopy();
            result.SetProperty(P::kUnicodeBidi, values::identifier(V::kNormal), false);
            result.RemoveProperty(P::kDirection, None);
            Out::Declaration(Retain(result, Mode(document)))
        }
    })
}
