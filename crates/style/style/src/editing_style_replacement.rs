// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! editing_style.cc:239-405,1180-1286,1667-1762,2064-2242. Replacement
//! declaration algebra runs against the same native CSSValue and RuleSet owners.
use super::serialization::{CssValueFromFontSizeNumber, Inline, Matched};
use super::*;
use crate::{
    document_style_engine::DocumentStyleEngine,
    resolver::production_style_builder::custom_properties::EnvironmentVariableResolver,
    serialization_style_query::SerializationRuleScope,
};
use cssom::editing_style::{
    ReplacementStyleOperation as Op, ReplacementStyleResult as Out, ReplacementWhiteSpaceFacts,
};
use layoutng_style::style::style_image::StyleImage;
use layoutng_style::style::visual_invalidation_diff::{
    LayoutWorkletDefinition, PaintInvalidationProperties, VisualInvalidationDiffContext,
};

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
fn IsStyleSpan(document: &dom::Document, node: usize) -> bool {
    let n = document.Node(node);
    n.IsHTMLElement("span")
        && (n.Attributes().is_empty()
            || n.Attributes().len() == 1 && n.FindAttribute("style").is_some())
}
fn Identifier(set: &CSSPropertyValueSet<ProductionCSSValueDispatch>, id: P) -> Option<V> {
    match set.GetPropertyCSSValue(id)?.Payload() {
        CSSValuePayload::kIdentifierClass(id) => Some(id.0),
        _ => None,
    }
}
fn FontWeight(value: &values::Value) -> Option<bool> {
    match value.Payload() {
        CSSValuePayload::kIdentifierClass(id) if matches!(id.0, V::kLighter | V::kBolder) => None,
        CSSValuePayload::kIdentifierClass(id) => Some(id.0 == V::kBold),
        CSSValuePayload::kNumericLiteralClass(v) => {
            Some(v.GetValueIfKnown().is_some_and(|v| v >= 600.0))
        }
        CSSValuePayload::kMathFunctionClass(v) => {
            Some(v.GetValueIfKnown().is_some_and(|v| v >= 600.0))
        }
        _ => None,
    }
}
fn Italic(value: &values::Value) -> bool {
    matches!(value.Payload(),CSSValuePayload::kIdentifierClass(id) if matches!(id.0,V::kItalic|V::kOblique))
        || value.IsFontStyleRangeValue()
}
fn Color(value: Option<&values::Value>) -> foundation::Color {
    match value.map(|v| v.Payload()) {
        Some(CSSValuePayload::kColorClass(color)) => color.0,
        Some(CSSValuePayload::kIdentifierClass(id)) => {
            values::NamedColor(id.0).unwrap_or(foundation::Color::kTransparent)
        }
        _ => foundation::Color::kTransparent,
    }
}
fn DiffDecorations(result: &mut ComputedPropertySet, property: P, base: Option<&values::Value>) {
    let Some(CSSValuePayload::kValueListClass(reference)) = base.map(|v| v.Payload()) else {
        return;
    };
    let Some(CSSValuePayload::kValueListClass(current)) =
        result.GetPropertyCSSValue(property).map(|v| v.Payload())
    else {
        return;
    };
    let important = result.PropertyIsImportant(property);
    let mut difference = current.Copy();
    for value in &reference.values {
        difference.RemoveAll(value.as_ref());
    }
    if difference.values.is_empty() {
        result.RemoveProperty(property, None);
    } else {
        result.SetProperty(
            property,
            values::list(difference.values, difference.separator),
            important,
        );
    }
}
/// Source GetPropertiesNotIn preserves author shorthand grouping before the
/// editing-specific equivalences for decoration, weight, italics, color/align.
pub(super) fn PropertiesNotIn(
    document: &dom::Document,
    node: usize,
    source: &CSSPropertyValueSet<ProductionCSSValueDispatch>,
    base: &CSSPropertyValueSet<ProductionCSSValueDispatch>,
) -> Result<ComputedPropertySet, Error> {
    let mut result = source.MutableCopy();
    result.RemoveEquivalentPropertiesPreservingShorthandsFromSet(base);
    for p in [P::kTextDecorationLine, P::kWebkitTextDecorationsInEffect] {
        DiffDecorations(
            &mut result,
            p,
            base.GetPropertyCSSValue(P::kWebkitTextDecorationsInEffect),
        );
    }
    if result
        .GetPropertyCSSValue(P::kFontWeight)
        .and_then(FontWeight)
        .zip(
            base.GetPropertyCSSValue(P::kFontWeight)
                .and_then(FontWeight),
        )
        .is_some_and(|(a, b)| a == b)
    {
        result.RemoveProperty(P::kFontWeight, None);
    }
    if result
        .GetPropertyCSSValue(P::kFontStyle)
        .is_some_and(Italic)
        && base.GetPropertyCSSValue(P::kFontStyle).is_some_and(Italic)
    {
        result.RemoveProperty(P::kFontStyle, None);
    }
    for p in [P::kColor, P::kBackgroundColor] {
        if base.HasProperty(p)
            && Color(result.GetPropertyCSSValue(p)) == Color(base.GetPropertyCSSValue(p))
        {
            result.RemoveProperty(p, None);
        }
    }
    if let Ok(style) = Style(document, node) {
        if Align(&result, style.Direction())
            .zip(Align(base, style.Direction()))
            .is_some_and(|(a, b)| a == b)
        {
            result.RemoveProperty(P::kTextAlign, None);
        }
    }
    Ok(result)
}
fn RemoveContext(
    engine: &mut DocumentStyleEngine,
    document: &dom::Document,
    node: usize,
    context: Option<usize>,
    declaration: &CSSInlineStyle,
    rules: bool,
    environment: Option<&dyn EnvironmentVariableResolver>,
) -> Result<CSSInlineStyle, Error> {
    if !rules && context.is_none() {
        return Ok(declaration.clone());
    }
    let mut result = Properties(declaration)?.MutableCopy();
    let matched = if rules {
        Matched(
            engine,
            document,
            node,
            SerializationRuleScope::All,
            environment,
        )?
    } else {
        ComputedPropertySet::new(Mode(document))
    };
    if rules && !matched.IsEmpty() {
        result = PropertiesNotIn(document, node, &result, &matched)?;
    }
    if let Some(context) = context {
        let mut base = Capture(document, context, true, true)?;
        if !base.HasProperty(P::kBackgroundColor) {
            base.SetProperty(
                P::kBackgroundColor,
                values::identifier(V::kTransparent),
                false,
            );
        }
        if rules {
            for property in matched.Properties() {
                base.RemoveProperty(property.PropertyID(), None);
            }
        }
        result = PropertiesNotIn(document, node, &result, &base)?;
    }
    if IsStyleSpan(document, node) {
        for (property, value) in [(P::kDisplay, V::kInline), (P::kFloat, V::kNone)] {
            if (!rules || !matched.HasProperty(property))
                && Identifier(&result, property) == Some(value)
            {
                result.RemoveProperty(property, None);
            }
        }
    }
    Ok(Retain(result, Mode(document)))
}
fn ImplicitConflicts(
    document: &dom::Document,
    node: usize,
    declaration: &CSSInlineStyle,
) -> Result<Out, Error> {
    let n = document.Node(node);
    let set = Properties(declaration)?;
    let equivalent = match n.Name() {
        "b" | "strong" => Some((P::kFontWeight, V::kBold)),
        "i" | "em" => Some((P::kFontStyle, V::kItalic)),
        "u" => Some((P::kTextDecorationLine, V::kUnderline)),
        "s" | "strike" => Some((P::kTextDecorationLine, V::kLineThrough)),
        "sub" => Some((P::kVerticalAlign, V::kSub)),
        "sup" => Some((P::kVerticalAlign, V::kSuper)),
        _ => None,
    };
    let replace_with_span=equivalent.is_some_and(|(p,v)| {
        let property=if p==P::kTextDecorationLine {set.GetPropertyCSSValue(P::kWebkitTextDecorationsInEffect).or_else(||set.GetPropertyCSSValue(p))}else{set.GetPropertyCSSValue(p)};
        property.is_some_and(|value|if p==P::kTextDecorationLine{!matches!(value.Payload(),CSSValuePayload::kValueListClass(list) if list.HasValue(values::identifier(v).as_ref()))}else{!matches!(value.Payload(),CSSValuePayload::kIdentifierClass(id) if id.0==v)})
    });
    let mut attributes = Vec::new();
    // Source kPreserveWritingDirection skips the two dir equivalents.
    if !replace_with_span && n.IsHTMLElement("font") {
        for (attribute, property) in [
            ("color", P::kColor),
            ("face", P::kFontFamily),
            ("size", P::kFontSize),
        ] {
            let Some(current) = set.GetPropertyCSSValue(property) else {
                continue;
            };
            let Some(attribute_value) = n.FindAttribute(attribute) else {
                continue;
            };
            let implicit = if property == P::kFontSize {
                CssValueFromFontSizeNumber(attribute_value.value.as_bytes()).map(values::identifier)
            } else {
                match style_parser::production_property_parser::ParseProperty(property,&foundation::String::from(attribute_value.value.as_str()),false,crate::parser::css_parser_mode::CSSParserMode::kHTMLQuirksMode){Ok(parsed)=>parsed.into_iter().find(|p|p.PropertyID()==property).map(|p|p.ValueRef()),Err(e) if e.kind==style_parser::production_property_parser::PropertyParseErrorKind::Invalid=>None,Err(_)=>return Err(Error::UnavailableComputedValue)}
            };
            if implicit.as_ref().is_none_or(|v| v.as_ref() != current) {
                attributes.push(foundation::QualifiedName::new(
                    Default::default(),
                    foundation::AtomicString::from_str(attribute),
                    Default::default(),
                ));
            }
        }
    }
    Ok(Out::ImplicitConflicts {
        replace_with_span,
        attributes,
    })
}
struct InvalidationContext;
impl VisualInvalidationDiffContext for InvalidationContext {
    fn LayoutWorkletDefinition(&self, _: &foundation::AtomicString) -> LayoutWorkletDefinition<'_> {
        LayoutWorkletDefinition::FeatureDisabled
    }
    fn PaintInvalidationProperties(&self, _: &StyleImage) -> PaintInvalidationProperties<'_> {
        // No CSSPaintImageGenerator is installed in this product. Source
        // DiffNeedsNormalPaintInvalidation returns true for an unpainted image.
        PaintInvalidationProperties::Unpainted
    }
    fn NativePropertiesEqual(
        &self,
        properties: &[P],
        a: &ComputedStyle,
        b: &ComputedStyle,
    ) -> bool {
        let a = CSSComputedStyleDeclaration::new(Some(a), Default::default());
        let b = CSSComputedStyleDeclaration::new(Some(b), Default::default());
        properties.iter().all(|&p| {
            a.GetPropertyCSSValue(p)
                .ok()
                .zip(b.GetPropertyCSSValue(p).ok())
                .is_some_and(|(a, b)| a == b)
        })
    }
    fn CustomPropertiesEqual(
        &self,
        properties: &[foundation::AtomicString],
        a: &ComputedStyle,
        b: &ComputedStyle,
    ) -> bool {
        a.CustomPropertiesEqual(&properties.to_vec(), b)
    }
    fn Potential3DCompositingReasons(
        &self,
        style: &ComputedStyle,
    ) -> foundation::CompositingReasons {
        style.Potential3DCompositingReasons()
    }
}
pub fn QueryReplacementEditingStyle(
    engine: &mut DocumentStyleEngine,
    document: &dom::Document,
    operation: Op,
    environment: Option<&dyn EnvironmentVariableResolver>,
) -> Result<Out, Error> {
    match operation {
        Op::Inline { node_id } => {
            let node = Node(document, node_id)?;
            Ok(Out::Inline(
                document
                    .Node(node)
                    .FindAttribute("style")
                    .map(|_| document.Node(node).InlineStyle()),
            ))
        }
        Op::PropertyCount { declaration } => {
            Ok(Out::Count(Properties(&declaration)?.PropertyCount()))
        }
        Op::ImplicitConflicts {
            node_id,
            declaration,
        } => ImplicitConflicts(document, Node(document, node_id)?, &declaration),
        Op::RemoveContext {
            node_id,
            context_id,
            declaration,
            include_rules,
        } => Ok(Out::Declaration(RemoveContext(
            engine,
            document,
            Node(document, node_id)?,
            context_id.map(|id| Node(document, id)).transpose()?,
            &declaration,
            include_rules,
            environment,
        )?)),
        Op::WhiteSpaceFacts {
            node_id,
            include_author_rules,
        } => {
            let node = Node(document, node_id)?;
            let inline = Inline(engine, document, node)?;
            let author = if include_author_rules {
                Some(Matched(
                    engine,
                    document,
                    node,
                    SerializationRuleScope::Author,
                    environment,
                )?)
            } else {
                None
            };
            let whitespace = |set: &CSSPropertyValueSet<ProductionCSSValueDispatch>| {
                set.HasProperty(P::kWhiteSpaceCollapse) || set.HasProperty(P::kTextWrapMode)
            };
            Ok(Out::WhiteSpaceFacts(ReplacementWhiteSpaceFacts {
                has_inline_white_space: whitespace(&inline),
                has_author_white_space: whitespace(&inline)
                    || author.as_ref().is_some_and(|set| whitespace(set)),
                collapse_breaks_and_wrap: Style(document, node)
                    .ok()
                    .map(|s| (s.ShouldCollapseBreaks(), s.ShouldWrapLine())),
            }))
        }
        Op::WhiteSpaceNormal => {
            let mut set = ComputedPropertySet::new(Mode(document));
            set.SetProperty(
                P::kWhiteSpaceCollapse,
                values::identifier(V::kCollapse),
                false,
            );
            set.SetProperty(P::kTextWrapMode, values::identifier(V::kWrap), false);
            Ok(Out::Declaration(Retain(set, Mode(document))))
        }
        Op::InsertionStyleMatchesAttribute { node_id, span_id } => {
            let node = StyledNode(document, node_id).ok_or(Error::MissingStyle)?;
            let style = Capture(document, node, false, false)?;
            let span = document.Node(Node(document, span_id)?);
            Ok(Out::Bool(
                style.AsText() == *span.InlineStyle().SourceTextValue(),
            ))
        }
        Op::VisualDifference { node_id, other_id } => {
            let node = Node(document, node_id)?;
            let other = Node(document, other_id)?;
            Ok(Out::VisualDifference(
                Style(document, node)
                    .ok()
                    .zip(Style(document, other).ok())
                    .map(|(a, b)| {
                        a.VisualInvalidationDiff(b, &InvalidationContext)
                            .HasDifference()
                    }),
            ))
        }
    }
}
