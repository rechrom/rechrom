// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium anchor positioning consumers, without CSS text reparsing.
#![allow(non_snake_case)]
use super::*;
use crate::production_position_area::{self as area, Kind};
pub(super) fn IsAnchorProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kAnchorName
            | kAnchorScope
            | kPositionAnchor
            | kPositionArea
            | kPositionVisibility
            | kPositionTryFallbacks
            | kPositionTryOrder
    )
}
// css_parsing_utils.cc:1757-1786. The source requires a double-dash prefix;
// tokenizer tokens own escaping and CSS-wide validation.
pub(super) fn DashedIdent<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if s.Peek().FunctionId() == Some(CSSValueID::kIdent) {
        return Err(unsupported(id, "ConsumeDashedIdent ident() runtime branch"));
    }
    if s.Peek().GetType() != kIdentToken || !s.Peek().Value().ToString().Utf8().starts_with("--") {
        return Ok(None);
    }
    Ok(Some(values::custom_ident(
        &s.ConsumeIncludingWhitespace().Value().ToString(),
        CSSPropertyID::kInvalid,
    )))
}
fn Names<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    let mut list = Vec::new();
    loop {
        list.push(DashedIdent(id, s)?.ok_or_else(|| invalid(id))?);
        if s.Peek().GetType() != kCommaToken {
            break;
        }
        s.ConsumeIncludingWhitespace();
    }
    Ok(values::list(list, values::ListSeparator::Comma))
}
// css_parsing_utils.cc:10437-10451. Shared by AnchorScope and TriggerScope.
pub(super) fn ConsumeNameScope<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    if s.Peek().Id() == CSSValueID::kNone {
        return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
    }
    if s.Peek().Id() == CSSValueID::kAll {
        s.ConsumeIncludingWhitespace();
        return Ok(values::scoped_keyword(CSSValueID::kAll));
    }
    Names(id, s)
}
fn AreaKeyword<T: TokenStreamTokenizer>(s: &mut Stream<T>) -> Option<(CSSValueID, Kind)> {
    let id = s.Peek().Id();
    let kind = area::KeywordKind(id, false)?;
    s.ConsumeIncludingWhitespace();
    Some((id, kind))
}
// css_parsing_utils.cc:10306-10349: grammar order, compatibility and canonical
// single/repeated spans. `any` belongs to the separate position-area query.
fn PositionArea<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    let mut first = AreaKeyword(s).ok_or_else(|| invalid(id))?;
    let Some(mut second) = AreaKeyword(s) else {
        return Ok(values::identifier(first.0));
    };
    if matches!(first.1, Kind::Vertical | Kind::Inline | Kind::SelfInline)
        || matches!(second.1, Kind::Horizontal | Kind::Block | Kind::SelfBlock)
    {
        std::mem::swap(&mut first, &mut second);
    }
    if !area::IsCompatible(first.1, second.1) {
        return Err(invalid(id));
    }
    if first.0 == second.0 {
        return Ok(values::identifier(first.0));
    }
    if first.0 == CSSValueID::kSpanAll && !area::IsRepeated(second.0) {
        return Ok(values::identifier(second.0));
    }
    if second.0 == CSSValueID::kSpanAll && !area::IsRepeated(first.0) {
        return Ok(values::identifier(first.0));
    }
    Ok(Pair(
        values::identifier(first.0),
        values::identifier(second.0),
        true,
    ))
}
fn IsFlip(k: CSSValueID) -> bool {
    matches!(
        k,
        CSSValueID::kFlipBlock
            | CSSValueID::kFlipInline
            | CSSValueID::kFlipStart
            | CSSValueID::kFlipX
            | CSSValueID::kFlipY
    )
}
// css_parsing_utils.cc:9958-10044. Preserve tactic order, normalize the optional
// name to the first item; a second tactic group after the name is not consumed.
fn SingleFallback<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let mut name = None;
    let mut flips = Vec::new();
    loop {
        if name.is_none() {
            name = DashedIdent(id, s)?;
            if name.is_some() {
                continue;
            }
            if mode == CSSParserMode::kUASheetMode
                && s.Peek().GetType() == kIdentToken
                && s.Peek().Value().ToString().Utf8().starts_with("-internal-")
            {
                name = Some(values::custom_ident(
                    &s.ConsumeIncludingWhitespace().Value().ToString(),
                    CSSPropertyID::kInvalid,
                ));
                continue;
            }
        }
        if flips.is_empty() {
            while IsFlip(s.Peek().Id()) && !flips.contains(&s.Peek().Id()) {
                flips.push(s.ConsumeIncludingWhitespace().Id());
            }
            if !flips.is_empty() {
                continue;
            }
        }
        break;
    }
    if name.is_none() && flips.is_empty() {
        return PositionArea(id, s);
    }
    let mut list = Vec::new();
    if let Some(name) = name {
        list.push(name);
    }
    list.extend(flips.into_iter().map(values::identifier));
    Ok(values::list(list, values::ListSeparator::Space))
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    match id {
        kAnchorScope => ConsumeNameScope(id, s),
        kAnchorName => {
            // longhands_custom.cc:326-335; css_parsing_utils.cc:10437-10452.
            if s.Peek().Id() == CSSValueID::kNone {
                return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            Names(id, s)
        }
        kPositionAnchor => {
            // longhands_custom.cc:220-230.
            if matches!(
                s.Peek().Id(),
                CSSValueID::kAuto | CSSValueID::kNone | CSSValueID::kNormal
            ) {
                return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            DashedIdent(id, s)?.ok_or_else(|| invalid(id))
        }
        kPositionArea => {
            if s.Peek().Id() == CSSValueID::kNone {
                return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            PositionArea(id, s)
        }
        kPositionVisibility => {
            // longhands_custom.cc:273-301. anchors-valid remains unsupported by
            // Chromium's grammar and therefore invalid here, not synthesized.
            if s.Peek().Id() == CSSValueID::kAlways {
                return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            let mut anchors = false;
            let mut overflow = false;
            if s.Peek().Id() == CSSValueID::kAnchorsVisible {
                anchors = true;
                s.ConsumeIncludingWhitespace();
            }
            if s.Peek().Id() == CSSValueID::kNoOverflow {
                overflow = true;
                s.ConsumeIncludingWhitespace();
            }
            if !anchors && s.Peek().Id() == CSSValueID::kAnchorsVisible {
                anchors = true;
                s.ConsumeIncludingWhitespace();
            }
            if !anchors && !overflow {
                return Err(invalid(id));
            }
            let mut list = Vec::new();
            if anchors {
                list.push(values::identifier(CSSValueID::kAnchorsVisible));
            }
            if overflow {
                list.push(values::identifier(CSSValueID::kNoOverflow));
            }
            Ok(values::list(list, values::ListSeparator::Space))
        }
        kPositionTryFallbacks => {
            // css_parsing_utils.cc:10047-10072.
            if s.Peek().Id() == CSSValueID::kNone {
                return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            let mut list = Vec::new();
            loop {
                list.push(SingleFallback(id, s, mode)?);
                if s.Peek().GetType() != kCommaToken {
                    break;
                }
                s.ConsumeIncludingWhitespace();
            }
            Ok(values::list(list, values::ListSeparator::Comma))
        }
        kPositionTryOrder => ConsumeLiteral(id, s, mode, GrammarFor(id)),
        _ => Err(unsupported(id, "anchor positioning consumer")),
    }
}
pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    // shorthands_custom.cc:4884-4911,4915-4923. The order is optional but the
    // fallback value is required. Both expanded longhands are not implicit.
    s.EnsureLookAhead();
    let save = s.Save();
    let order = match Consume(CSSPropertyID::kPositionTryOrder, s, mode) {
        Ok(value) => value,
        Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
        Err(_) => {
            s.EnsureLookAhead();
            s.Restore(save);
            values::identifier(CSSValueID::kNormal)
        }
    };
    let fallbacks = Consume(CSSPropertyID::kPositionTryFallbacks, s, mode)?;
    out.push(make_expanded(
        CSSPropertyID::kPositionTryOrder,
        id,
        order,
        false,
    ));
    out.push(make_expanded(
        CSSPropertyID::kPositionTryFallbacks,
        id,
        fallbacks,
        false,
    ));
    Ok(())
}
