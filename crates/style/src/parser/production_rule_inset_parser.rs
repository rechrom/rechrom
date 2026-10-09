// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! css_parsing_utils.cc:8071-8205; shorthands_custom.cc:1261-1921.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kColumnRuleInsetCapStart
            | kColumnRuleInsetCapEnd
            | kColumnRuleInsetJunctionStart
            | kColumnRuleInsetJunctionEnd
            | kRowRuleInsetCapStart
            | kRowRuleInsetCapEnd
            | kRowRuleInsetJunctionStart
            | kRowRuleInsetJunctionEnd
    )
}
pub(super) fn IsShorthand(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kColumnRuleInset
            | kRowRuleInset
            | kRuleInset
            | kColumnRuleInsetCap
            | kRowRuleInsetCap
            | kRuleInsetCap
            | kColumnRuleInsetJunction
            | kRowRuleInsetJunction
            | kRuleInsetJunction
            | kColumnRuleInsetStart
            | kRowRuleInsetStart
            | kRuleInsetStart
            | kColumnRuleInsetEnd
            | kRowRuleInsetEnd
            | kRuleInsetEnd
    )
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    ConsumeLiteral(
        id,
        s,
        mode,
        Grammar::Length {
            percent: true,
            nonnegative: false,
            quirks: false,
            keywords: &["overlap-join"],
        },
    )
}
fn PairValues<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<[Rc<Value>; 2], PropertyParseError> {
    let first = Consume(id, s, mode)?;
    let second = if at_value_end(s) || AtSlash(s) {
        first.clone()
    } else {
        Consume(id, s, mode)?
    };
    Ok([first, second])
}
fn AtSlash<T: TokenStreamTokenizer>(s: &mut Stream<T>) -> bool {
    s.Peek().GetType() == kDelimiterToken && s.Peek().Delimiter() == b'/' as u16
}
pub(super) fn Expand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    use CSSPropertyID::*;
    let full = matches!(id, kColumnRuleInset | kRowRuleInset | kRuleInset);
    let start_end = matches!(
        id,
        kColumnRuleInsetStart
            | kRowRuleInsetStart
            | kRuleInsetStart
            | kColumnRuleInsetEnd
            | kRowRuleInsetEnd
            | kRuleInsetEnd
    );
    let cap = if start_end {
        let first = Consume(id, s, mode)?;
        [first.clone(), first]
    } else {
        PairValues(id, s, mode)?
    };
    let junction = if full && AtSlash(s) {
        s.ConsumeIncludingWhitespace();
        PairValues(id, s, mode)?
    } else {
        cap.clone()
    };
    if !at_value_end(s) {
        return Err(invalid(id));
    }
    // RuleInset emits column then row; RuleInsetCap/Junction use the respective
    // axis shorthands as source provenance. Start/End retain the original ID.
    let fields = if id == kRuleInset {
        vec![
            kColumnRuleInsetCapStart,
            kColumnRuleInsetCapEnd,
            kColumnRuleInsetJunctionStart,
            kColumnRuleInsetJunctionEnd,
            kRowRuleInsetCapStart,
            kRowRuleInsetCapEnd,
            kRowRuleInsetJunctionStart,
            kRowRuleInsetJunctionEnd,
        ]
    } else {
        ShorthandFor(id).to_vec()
    };
    for field in fields {
        let (v, axis) = match field {
            kColumnRuleInsetCapStart => (&cap[0], kColumnRuleInsetCap),
            kColumnRuleInsetCapEnd => (&cap[1], kColumnRuleInsetCap),
            kColumnRuleInsetJunctionStart => (&junction[0], kColumnRuleInsetJunction),
            kColumnRuleInsetJunctionEnd => (&junction[1], kColumnRuleInsetJunction),
            kRowRuleInsetCapStart => (&cap[0], kRowRuleInsetCap),
            kRowRuleInsetCapEnd => (&cap[1], kRowRuleInsetCap),
            kRowRuleInsetJunctionStart => (&junction[0], kRowRuleInsetJunction),
            kRowRuleInsetJunctionEnd => (&junction[1], kRowRuleInsetJunction),
            _ => unreachable!(),
        };
        let source = if matches!(id, kRuleInsetCap | kRuleInsetJunction) {
            axis
        } else {
            id
        };
        out.push(make_expanded(field, source, v.clone(), false));
    }
    Ok(())
}
