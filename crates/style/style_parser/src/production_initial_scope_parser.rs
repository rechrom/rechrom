// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Initial-letter, scroll-marker-group and trigger-scope stable consumers.
// Audit (current Chromium checkout; effective = nonblank/noncomment lines,
// excluding brace-only lines, directives, namespaces and using declarations).
// Entry/parser: generated longhands.cc:9095-9100,17796-17801;
// custom:8776-8800; css_parsing_utils.cc:9065-9117,10437-10451.
// Exposure: generated:14693-14698,17775-17780.
// Apply: generated:9102-9110,14715-14723,17803-17811;
// style_builder_converter.cc:1892-1909,2398-2433,4329-4333.
// Entry subtotal effective148 / mapped141 / omitted2 / remaining5.
// Reused source collaborators (not new translated LOC): parsing utils
// 1155-1208,1219-1253,1262-1284,1757-1794; converter:2287-2294,2366-2390;
// css_identifier_value_mappings.h:1727-1739,1757-1769;
// css_primitive_value.cc:377-384; numeric_literal_value.cc:142-149;
// css_math_function_value.cc:99-114,190-204.
// Shared subtotal effective194 / mapped160 / omitted27 / remaining7.
// Existing native source: style_initial_letter.h:13-50/.cc:12-51,
// scroll_marker_group.h:13-46; style_trigger_scope.h:12 aliases StyleNameScope.
// Existing native subtotal effective72 / mapped72 / omitted0 / remaining0.
// Selected total effective414 / mapped373 / omitted29 / remaining12.
// Omitted: nonselected integer ranges/AllowPercent, number percentage paths,
// nonselected mathematical ranges, NOTREACHED/unreachable converter fallbacks.
// Remaining: custom:8790 alternate disabled flag; generated:14695,17777
// disabled Exposure; utils:1761-1762 test-only ident(); converter:2410
// mathematical size<1 native DCHECK and :2429 undefined out-of-int sink;
// :2290,2293,2369,2373,2375 scoped population/HasTreeScopedReference owner.
// Shared extended math expression parser, GC, CSSOM serialization and layout
// runtime bodies are outside this selected source audit; partial != complete.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kInitialLetter
            | CSSPropertyID::kScrollMarkerGroup
            | CSSPropertyID::kTriggerScope
    )
}

fn Size<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let size = ConsumeLiteral(id, s, mode, Grammar::Number { nonnegative: true })?;
    // css_parsing_utils.cc:9080-9084,9096-9100. Only literal numbers are
    // checked here; mathematical values retain their nonnegative range.
    if matches!(size.Payload(), CSSValuePayload::kNumericLiteralClass(n) if n.DoubleValue() < 1.) {
        return Err(invalid(id));
    }
    Ok(size)
}

pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSValueID::*;
    match id {
        CSSPropertyID::kTriggerScope => anchor_parser::ConsumeNameScope(id, s),
        // longhands_custom.cc:8776-8799. CSSPseudoScrollMarkers is stable
        // (runtime JSON:1858-1860), and is the flag actually checked here.
        // CSSScrollMarkerGroupModes:1923-1924 is experimental but is not used
        // by this ParseSingleValue; tabs/links are therefore stable reachable.
        CSSPropertyID::kScrollMarkerGroup => {
            let position = s.Peek().Id();
            if !matches!(position, kNone | kBefore | kAfter) {
                return Err(invalid(id));
            }
            s.ConsumeIncludingWhitespace();
            let first = values::identifier(position);
            if position == kNone || s.AtEnd() {
                return Ok(first);
            }
            let mode = s.Peek().Id();
            if !matches!(mode, kTabs | kLinks) {
                return Err(invalid(id));
            }
            s.ConsumeIncludingWhitespace();
            Ok(Pair(first, values::identifier(mode), true))
        }
        // css_parsing_utils.cc:9065-9117. Prefix drop/raise canonicalizes to
        // the same size-first list as suffix spelling; integer sink optional.
        CSSPropertyID::kInitialLetter => {
            if s.Peek().Id() == kNormal {
                s.ConsumeIncludingWhitespace();
                return Ok(values::identifier(kNormal));
            }
            if matches!(s.Peek().Id(), kDrop | kRaise) {
                let kind = values::identifier(s.ConsumeIncludingWhitespace().Id());
                let size = Size(id, s, mode)?;
                return Ok(values::list(vec![size, kind], values::ListSeparator::Space));
            }
            let size = Size(id, s, mode)?;
            let mut list = vec![size];
            if matches!(s.Peek().Id(), kDrop | kRaise) {
                list.push(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            } else if !s.AtEnd() {
                s.EnsureLookAhead();
                let saved = s.Save();
                match ConsumeLiteral(id, s, mode, Grammar::Integer { minimum: 1 }) {
                    Ok(sink) => list.push(sink),
                    Err(e) if e.kind == PropertyParseErrorKind::Invalid => s.Restore(saved),
                    Err(e) => return Err(e),
                }
            }
            Ok(values::list(list, values::ListSeparator::Space))
        }
        _ => Err(unsupported(
            id,
            "initial letter / marker group / name scope consumer",
        )),
    }
}
