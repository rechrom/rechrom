// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium gap-rule consumers used by standard column-rule properties.
#![allow(non_snake_case)]
use super::*;
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kColumnRuleColor
            | CSSPropertyID::kColumnRuleStyle
            | CSSPropertyID::kColumnRuleWidth
            | CSSPropertyID::kRowRuleColor
            | CSSPropertyID::kRowRuleStyle
            | CSSPropertyID::kRowRuleWidth
    )
}
pub(super) fn IsShorthand(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kColumnRule
            | CSSPropertyID::kRowRule
            | CSSPropertyID::kRule
            | CSSPropertyID::kRuleColor
            | CSSPropertyID::kRuleStyle
            | CSSPropertyID::kRuleWidth
            | CSSPropertyID::kWebkitColumnBreakBefore
            | CSSPropertyID::kWebkitColumnBreakAfter
            | CSSPropertyID::kWebkitColumnBreakInside
    )
}
// css_parsing_utils.cc:5870-5892. Border style and color use existing consumers.
fn Entry<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    if s.Peek().FunctionId() == Some(CSSValueID::kRepeat) {
        return Err(unsupported(
            id,
            "ConsumeGapDecorationRepeatFunction / CSSRepeatValue owner",
        ));
    }
    match id {
        kColumnRuleColor | kRowRuleColor => ConsumeColor(id, s),
        kColumnRuleStyle | kRowRuleStyle => {
            if s.Peek().GetType() != kIdentToken {
                return Err(invalid(id));
            }
            ConsumeLonghand(kBorderLeftStyle, s, mode).map_err(|e| error(id, e.kind, e.operation))
        }
        // Source explicitly forbids unitless quirks, even in quirks mode.
        kColumnRuleWidth | kRowRuleWidth => {
            if s.Peek().GetType() == kFunctionToken && !IsMathFunction(s) {
                return Err(invalid(id));
            }
            ConsumeLiteral(
                id,
                s,
                mode,
                Grammar::Length {
                    percent: false,
                    nonnegative: true,
                    quirks: false,
                    keywords: &["thin", "medium", "thick"],
                },
            )
        }
        _ => Err(invalid(id)),
    }
}
// css_parsing_utils.cc:5941-5985. Source accepts a consumed trailing comma.
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let mut list = Vec::new();
    if s.AtEnd() {
        return Err(invalid(id));
    }
    loop {
        s.EnsureLookAhead();
        let save = s.Save();
        match Entry(id, s, mode) {
            Ok(v) => list.push(v),
            Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
            Err(_) => {
                s.EnsureLookAhead();
                s.Restore(save);
                break;
            }
        }
        if s.Peek().GetType() != kCommaToken {
            break;
        }
        s.ConsumeIncludingWhitespace();
    }
    if list.is_empty() {
        Err(invalid(id))
    } else {
        Ok(values::list(list, values::ListSeparator::Comma))
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
    // shorthands_custom.cc:6105-6129,6141-6165,6177-6201: consume
    // one list and retain its identity in both physical gap directions.
    if matches!(id, kRuleColor | kRuleStyle | kRuleWidth) {
        let properties = ShorthandFor(id);
        let value = Consume(properties[0], s, mode)?;
        for &property in properties {
            out.push(make_expanded(property, id, value.clone(), false));
        }
        return Ok(());
    }
    if !matches!(id, kColumnRule | kRowRule | kRule) {
        // css_parsing_utils.cc:8306-8325; legacy always becomes column.
        let value = s.ConsumeIncludingWhitespace().Id();
        let inside = id == kWebkitColumnBreakInside;
        if !matches!(value, kAuto | kAvoid) && (inside || value != kAlways) {
            return Err(invalid(id));
        }
        out.push(make_expanded(
            ShorthandFor(id)[0],
            id,
            values::identifier(if value == kAlways { kColumn } else { value }),
            false,
        ));
        return Ok(());
    }
    // css_parsing_utils.cc:7901-7972,8209-8266. Greedy width/style/color
    // per comma segment, with explicit medium/none/currentcolor defaults.
    let properties = [kColumnRuleWidth, kColumnRuleStyle, kColumnRuleColor];
    let mut lists: [Vec<Rc<Value>>; 3] = std::array::from_fn(|_| Vec::new());
    while !s.AtEnd() {
        if s.Peek().FunctionId() == Some(kRepeat) {
            return Err(unsupported(
                id,
                "ConsumeGapDecorationsShorthandRepeatFunction / CSSRepeatValue owner",
            ));
        }
        let mut segment: [Option<Rc<Value>>; 3] = std::array::from_fn(|_| None);
        while !s.AtEnd() && s.Peek().GetType() != kCommaToken {
            let mut success = false;
            for (index, &property) in properties.iter().enumerate() {
                s.EnsureLookAhead();
                let save = s.Save();
                match Entry(property, s, mode) {
                    Ok(v) => {
                        if segment[index].is_some() {
                            return Err(invalid(id));
                        }
                        segment[index] = Some(v);
                        success = true;
                    }
                    Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
                    Err(_) => {
                        s.EnsureLookAhead();
                        s.Restore(save);
                    }
                }
            }
            if !success {
                break;
            }
        }
        if segment.iter().all(Option::is_none) {
            return Err(invalid(id));
        }
        for (index, default) in [kMedium, kNone, kCurrentcolor].into_iter().enumerate() {
            lists[index].push(
                segment[index]
                    .take()
                    .unwrap_or_else(|| values::identifier(default)),
            );
        }
        if s.Peek().GetType() != kCommaToken {
            break;
        }
        s.ConsumeIncludingWhitespace();
    }
    if lists[0].is_empty() {
        return Err(invalid(id));
    }
    // RowRule:1185-1211 and Rule:6057-6085 share the same consumed
    // lists. Rule adds the column direction first and then the row direction;
    // it does not reset cap/inset/junction/visibility longhands.
    let lists = lists.map(|list| values::list(list, values::ListSeparator::Comma));
    let column = [kColumnRuleWidth, kColumnRuleStyle, kColumnRuleColor];
    let row = [kRowRuleWidth, kRowRuleStyle, kRowRuleColor];
    for direction in [column, row] {
        if (direction == column && id == kRowRule) || (direction == row && id == kColumnRule) {
            continue;
        }
        // CSSGapDecorationUtils::AddProperties assigns the directional
        // shorthand identity even when called by Rule::ParseShorthand.
        let shorthand = if direction == column {
            kColumnRule
        } else {
            kRowRule
        };
        for (property, list) in direction.into_iter().zip(&lists) {
            out.push(make_expanded(property, shorthand, list.clone(), false));
        }
    }
    Ok(())
}
