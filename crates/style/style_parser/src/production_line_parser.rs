// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Line/pagination consumers from Chromium's custom longhands/shorthands.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsLineProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kVerticalAlign
            | kTabSize
            | kWebkitLineClamp
            | kLineClamp
            | kMaxLines
            | kAlternativeWebkitLineClampLonghand
            | kContinue
            | kBlockEllipsis
            | kBreakBefore
            | kBreakAfter
            | kBreakInside
    )
}
pub(super) fn IsLineShorthand(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kPageBreakBefore
            | kPageBreakAfter
            | kPageBreakInside
            | kAlternativeLineClampShorthand
            | kAlternativeWebkitLineClampShorthand
    )
}
fn PositiveInteger<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    ConsumeLiteral(id, s, mode, Grammar::Integer { minimum: 1 })
}
#[derive(Default)]
struct ClampParts {
    number: Option<Rc<Value>>,
    auto: bool,
    ellipsis: Option<CSSValueID>,
    legacy: bool,
}
// longhands_custom.cc:6404-6446; shorthands_custom.cc:5775-5827.
fn ConsumeClampParts<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<ClampParts, PropertyParseError> {
    use CSSValueID::*;
    let mut p = ClampParts::default();
    loop {
        match s.Peek().Id() {
            kWebkitLegacy => {
                p.legacy = true;
                s.ConsumeIncludingWhitespace();
                break;
            }
            kAuto if !p.auto => {
                p.auto = true;
                s.ConsumeIncludingWhitespace();
                continue;
            }
            kEllipsis | kNoEllipsis if p.ellipsis.is_none() => {
                p.ellipsis = Some(s.ConsumeIncludingWhitespace().Id());
                continue;
            }
            _ => {}
        }
        if p.number.is_none() {
            s.EnsureLookAhead();
            let save = s.Save();
            match PositiveInteger(id, s, mode) {
                Ok(value) => {
                    p.number = Some(value);
                    continue;
                }
                Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
                Err(_) => {
                    s.EnsureLookAhead();
                    s.Restore(save);
                }
            }
        }
        break;
    }
    if p.number.is_none() && !p.auto && p.ellipsis.is_none() {
        return Err(invalid(id));
    }
    Ok(p)
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    use CSSValueID::*;
    match id {
        // longhands_custom.cc:10776-10788. Allow the source's unitless quirk.
        kVerticalAlign => {
            if (kBaseline as i32..=kWebkitBaselineMiddle as i32).contains(&(s.Peek().Id() as i32)) {
                return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            if mode == CSSParserMode::kSVGAttributeMode && IsMathFunction(s) {
                use crate::css_math_expression_node::CalculationResultCategory as C;
                return ConsumeMath(
                    id,
                    s,
                    &[C::Length, C::Percent, C::LengthFunction, C::Number],
                    crate::css_math_function_value::ValueRange::All,
                );
            }
            ConsumeLiteral(
                id,
                s,
                mode,
                Grammar::Length {
                    percent: true,
                    nonnegative: false,
                    quirks: true,
                    keywords: &[],
                },
            )
        }
        // longhands_custom.cc:9696-9709. Try number before length.
        kTabSize => {
            s.EnsureLookAhead();
            let save = s.Save();
            match ConsumeLiteral(id, s, mode, Grammar::Number { nonnegative: true }) {
                Ok(value) => return Ok(value),
                Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
                Err(_) => {
                    s.EnsureLookAhead();
                    s.Restore(save);
                }
            }
            ConsumeLiteral(
                id,
                s,
                mode,
                Grammar::Length {
                    percent: false,
                    nonnegative: true,
                    quirks: false,
                    keywords: &[],
                },
            )
        }
        kWebkitLineClamp | kAlternativeWebkitLineClampLonghand => {
            // longhands_custom.cc:11253-11288.
            if s.Peek().Id() == kNone {
                return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            PositiveInteger(id, s, mode)
        }
        // longhands_custom.cc:6341-6373; normalize to number/auto pair.
        kMaxLines => {
            let mut number = None;
            let mut auto = false;
            loop {
                if !auto && s.Peek().Id() == kAuto {
                    auto = true;
                    s.ConsumeIncludingWhitespace();
                    continue;
                }
                if number.is_none() {
                    s.EnsureLookAhead();
                    let save = s.Save();
                    match PositiveInteger(id, s, mode) {
                        Ok(value) => {
                            number = Some(value);
                            continue;
                        }
                        Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
                        Err(_) => {
                            s.EnsureLookAhead();
                            s.Restore(save);
                        }
                    }
                }
                break;
            }
            match (number, auto) {
                (Some(number), true) => Ok(Pair(number, values::identifier(kAuto), false)),
                (Some(number), false) => Ok(number),
                (None, true) => Ok(values::identifier(kAuto)),
                _ => Err(invalid(id)),
            }
        }
        // longhands_custom.cc:6404-6475; ellipsis is the omitted default.
        kLineClamp => {
            if s.Peek().Id() == kNone {
                return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            let mut p = ConsumeClampParts(id, s, mode)?;
            if p.ellipsis == Some(kEllipsis) {
                p.ellipsis = None;
            }
            let mut list = Vec::new();
            let has_number = p.number.is_some();
            if let Some(number) = p.number {
                list.push(number);
            }
            if p.auto {
                if has_number || p.ellipsis.is_none() {
                    list.push(values::identifier(kAuto));
                }
            } else if !has_number && p.ellipsis.is_none() {
                list.push(values::identifier(kAuto));
            }
            if let Some(ellipsis) = p.ellipsis {
                list.push(values::identifier(ellipsis));
            }
            if p.legacy {
                list.push(values::identifier(kWebkitLegacy));
            }
            Ok(values::list(list, values::ListSeparator::Space))
        }
        kContinue => ConsumeLiteral(
            id,
            s,
            mode,
            Grammar::Keywords(&["normal", "collapse", "-webkit-legacy"]),
        ),
        kBlockEllipsis | kBreakBefore | kBreakAfter | kBreakInside => {
            ConsumeLiteral(id, s, mode, GrammarFor(id))
        }
        _ => Err(unsupported(id, "line/pagination consumer")),
    }
}
pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    use CSSPropertyID::*;
    use CSSValueID::*;
    if matches!(id, kPageBreakBefore | kPageBreakAfter | kPageBreakInside) {
        // css_parsing_utils.cc:8292-8304,8319-8325; shorthands_custom.cc:4621-4691.
        let value = s.ConsumeIncludingWhitespace().Id();
        let inside = id == kPageBreakInside;
        if !matches!(value, kAuto | kAvoid)
            && (inside || !matches!(value, kAlways | CSSValueID::kLeft | CSSValueID::kRight))
        {
            return Err(invalid(id));
        }
        let value = if value == kAlways {
            CSSValueID::kPage
        } else {
            value
        };
        out.push(make_expanded(
            ShorthandFor(id)[0],
            id,
            values::identifier(value),
            false,
        ));
        return Ok(());
    }
    if !crate::production_line_features::IsExposed(id) {
        return Err(unsupported(id, "CSSLineClampAsShorthand runtime flag"));
    }
    for (longhand, value) in ClampShorthandValues(id, s, mode)? {
        out.push(make_expanded(longhand, id, value, false));
    }
    Ok(())
}
fn ClampShorthandValues<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Vec<(CSSPropertyID, Rc<Value>)>, PropertyParseError> {
    use CSSPropertyID::*;
    use CSSValueID::*;
    // shorthands_custom.cc:5775-5869,5898-5940; gated alternatives retain
    // the real MaxLines / BlockEllipsis / Continue fields and reset semantics.
    let (max, ellipsis, continuation) = if s.Peek().Id() == kNone {
        s.ConsumeIncludingWhitespace();
        (values::identifier(kAuto), kNoEllipsis, kNormal)
    } else if id == kAlternativeWebkitLineClampShorthand {
        (PositiveInteger(id, s, mode)?, kEllipsis, kWebkitLegacy)
    } else {
        let p = ConsumeClampParts(id, s, mode)?;
        let max = match (p.number, p.auto) {
            (Some(number), true) => Pair(number, values::identifier(kAuto), false),
            (Some(number), false) => number,
            _ => values::identifier(kAuto),
        };
        (
            max,
            p.ellipsis.unwrap_or(kEllipsis),
            if p.legacy { kWebkitLegacy } else { kCollapse },
        )
    };
    Ok(vec![
        (kMaxLines, max),
        (kBlockEllipsis, values::identifier(ellipsis)),
        (kContinue, values::identifier(continuation)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    fn internal(id: CSSPropertyID, text: &str) -> Rc<Value> {
        let text = String::from(text);
        let mut s: Stream = Stream::new(StringView::from(&text), 0);
        let value = Consume(id, &mut s, CSSParserMode::kHTMLStandardMode).unwrap();
        assert!(s.AtEnd());
        value
    }
    #[test]
    fn hidden_line_consumers_keep_chromium_canonical_values_and_shorthand_resets() {
        use CSSPropertyID::*;
        for (text, expected) in [
            ("3 auto ellipsis -webkit-legacy", "3 auto -webkit-legacy"),
            ("ellipsis", "auto"),
            ("auto no-ellipsis", "no-ellipsis"),
            ("none", "none"),
        ] {
            assert_eq!(internal(kLineClamp, text).CssText().Utf8(), expected);
        }
        assert_eq!(internal(kMaxLines, "auto 3").CssText().Utf8(), "3 auto");
        for (text, expected) in [
            ("none", ["auto", "no-ellipsis", "normal"]),
            ("3", ["3", "ellipsis", "-webkit-legacy"]),
        ] {
            let text = String::from(text);
            let mut s: Stream = Stream::new(StringView::from(&text), 0);
            let out = ClampShorthandValues(
                kAlternativeWebkitLineClampShorthand,
                &mut s,
                CSSParserMode::kHTMLStandardMode,
            )
            .unwrap();
            assert!(s.AtEnd());
            assert_eq!(out.len(), 3);
            for ((v, expected), id) in
                out.iter()
                    .zip(expected)
                    .zip([kMaxLines, kBlockEllipsis, kContinue])
            {
                assert_eq!(v.0, id);
                assert_eq!(v.1.CssText().Utf8(), expected);
            }
        }
    }
}
