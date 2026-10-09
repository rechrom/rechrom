// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium scroll/view timeline definition consumers and aligned shorthands.
// Selected Chromium source audit: effective 482, mapped 440, omitted 19,
// remaining 23. Parser 340/306/14/20; Apply/converter 120/112/5/3;
// existing native value types 22/22/0/0. Effective lines exclude comments,
// blank/preprocessor/namespace/using/visibility/brace-only lines. Shared typed
// consumers and native classes are reused; there is no parallel timeline model.
// Nested CSSMath/primitive length conversion, native generated fields and
// runtime timeline scheduling/consumption are outside this selected scope.
// Both ledgers retain pending collaborators; partial is not complete.
//
// Parser ranges under third_party/blink/renderer/core:
// css/properties/longhands/longhands_custom.cc:
//   9178-9185,9209-9217,10849-10856,10880-10888,10912-10920,12211-12227;
// css/properties/css_parsing_utils.cc:
//   1286-1292,1294-1392,1394-1418,1452-1499,1757-1785,
//   4943-4961,4972-4999;
// css/properties/css_parsing_utils.h:962-976;
// css/properties/shorthands/shorthands_custom.cc:
//   5073-5170,5210-5219,5671-5680.
// Parser remaining: custom 12218-12220 (CSSTimelineScopeAll experimental,
// stable disabled); utils 1760-1763 (CSSIdentFunction test, stable disabled),
// 1478-1492 (shared MathFunctionParser flag/context adapter; the selected
// ConsumeLengthOrPercent property uses the default calc-size-forbid branch).
// Omitted DCHECK: utils 1399; utils.h 974; shorthand 5114-5115,5119,
//   5149-5157,5163.
//
// Apply generated out/Min/gen/.../core/css/properties/longhands.cc:
//   15226-15234,15255-15263,16959-16967,17982-17990,18011-18024,
//   18045-18053;
// css/resolver/style_builder_converter.cc:
//   188-198,2111-2120,2296-2311,3944-3997;
// css/css_identifier_value_mappings.h:1958-1972.
// Apply remaining: generated 18015-18019 (StandardizedBrowserZoom is stable
// enabled; changed-zoom parent re-conversion owner is typed Unsupported).
// Omitted DCHECK/NOTREACHED: converter 191,2300,3988,3991; mappings 1971.
// Existing native owners: animation/timeline_inset.h:14-33;
// style/style_timeline_scope.h:14-34.
// Definition-side source fields are unscoped AtomicString vectors, not
// CSSAnimationData/ScopedCSSName. Shadow TreeScope/HasTreeScopedReference
// are consumption-side collaborators outside this batch, never fabricated.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kScrollTimelineName
            | kScrollTimelineAxis
            | kViewTimelineName
            | kViewTimelineAxis
            | kViewTimelineInset
            | kTimelineScope
    )
}

// css_parsing_utils.cc:1757-1785,4949-4961. CSSIdentFunction is test-only,
// disabled at stable defaults. Preserve the decoded, case-sensitive ident.
fn Name<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    none: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    if none && s.Peek().Id() == CSSValueID::kNone {
        return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
    }
    if s.Peek().GetType() != kIdentToken || !s.Peek().Value().ToString().Utf8().starts_with("--") {
        return Err(invalid(id));
    }
    Ok(values::custom_ident(
        &s.ConsumeIncludingWhitespace().Value().ToString(),
        CSSPropertyID::kInvalid,
    ))
}

fn Axis<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    if !matches!(
        s.Peek().Id(),
        CSSValueID::kBlock | CSSValueID::kInline | CSSValueID::kX | CSSValueID::kY
    ) {
        return Err(invalid(id));
    }
    Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()))
}

fn Side<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    svg_parser::ConsumeLength(id, s, mode, true, false, &["auto"])
}

// ConsumeSingleTimelineInset returns a pair even when the sides are identical.
fn Inset<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let first = Side(id, s, mode)?;
    s.EnsureLookAhead();
    let saved = s.Save();
    let second = match Side(id, s, mode) {
        Ok(v) => v,
        Err(e) => {
            s.Restore(saved);
            if e.kind == PropertyParseErrorKind::Unsupported {
                return Err(e);
            }
            first.clone()
        }
    };
    Ok(Rc::new(Value::new(CSSValuePayload::kValuePairClass(
        values::CSSValuePair {
            first,
            second,
            drop_identical: true,
        },
    ))))
}

pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    if id == kTimelineScope && s.Peek().Id() == CSSValueID::kNone {
        return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
    }
    // CSSTimelineScopeAll is experimental: stable does not consume `all`.
    let mut items = Vec::new();
    loop {
        items.push(match id {
            kScrollTimelineName | kViewTimelineName => Name(id, s, true)?,
            kTimelineScope => Name(id, s, false)?,
            kScrollTimelineAxis | kViewTimelineAxis => Axis(id, s)?,
            kViewTimelineInset => Inset(id, s, mode)?,
            _ => return Err(invalid(id)),
        });
        if s.Peek().GetType() != kCommaToken {
            break;
        }
        s.ConsumeIncludingWhitespace();
    }
    Ok(values::list(items, values::ListSeparator::Comma))
}

// shorthands_custom.cc:5073-5170,5210-5219,5671-5680. A name is mandatory
// and first; axis and (view-only) inset are unordered and appear at most once.
pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    let view = id == CSSPropertyID::kViewTimeline;
    let mut names = Vec::new();
    let mut axes = Vec::new();
    let mut insets = Vec::new();
    loop {
        names.push(Name(id, s, true)?);
        let mut axis = None;
        let mut inset = None;
        loop {
            if axis.is_none()
                && matches!(
                    s.Peek().Id(),
                    CSSValueID::kBlock | CSSValueID::kInline | CSSValueID::kX | CSSValueID::kY
                )
            {
                axis = Some(Axis(id, s)?);
                continue;
            }
            if view && inset.is_none() {
                s.EnsureLookAhead();
                let saved = s.Save();
                match Inset(id, s, mode) {
                    Ok(v) => {
                        inset = Some(v);
                        continue;
                    }
                    Err(e) => {
                        s.Restore(saved);
                        if e.kind == PropertyParseErrorKind::Unsupported {
                            return Err(e);
                        }
                    }
                }
            }
            break;
        }
        axes.push(axis.unwrap_or_else(|| values::identifier(CSSValueID::kBlock)));
        if view {
            insets.push(inset.unwrap_or_else(|| {
                Rc::new(Value::new(CSSValuePayload::kValuePairClass(
                    values::CSSValuePair {
                        first: values::identifier(CSSValueID::kAuto),
                        second: values::identifier(CSSValueID::kAuto),
                        drop_identical: true,
                    },
                )))
            }));
        }
        if s.Peek().GetType() != kCommaToken {
            break;
        }
        s.ConsumeIncludingWhitespace();
    }
    let longhands = ShorthandFor(id);
    out.push(make_expanded(
        longhands[0],
        id,
        values::list(names, values::ListSeparator::Comma),
        false,
    ));
    out.push(make_expanded(
        longhands[1],
        id,
        values::list(axes, values::ListSeparator::Comma),
        false,
    ));
    if view {
        out.push(make_expanded(
            longhands[2],
            id,
            values::list(insets, values::ListSeparator::Comma),
            false,
        ));
    }
    Ok(())
}
