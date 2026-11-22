// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium text box, fitting and spacing token consumers.
#![allow(non_snake_case)]
use super::*;
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kTextBoxEdge
            | kTextBoxTrim
            | kTextFit
            | kTextAutospace
            | kTextSpacingTrim
            | kTextDecorationInset
            | kWebkitTextDecorationsInEffect
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hidden_text_box_spacing_and_inset_consumers_preserve_typed_forms() {
        for (text, auto, trim) in [
            ("none", "no-autospace", "space-all"),
            ("normal", "normal", "normal"),
            ("normal normal", "normal", "normal"),
            ("space-first no-autospace", "no-autospace", "space-first"),
            ("trim-start normal", "normal", "trim-start"),
            ("no-autospace", "no-autospace", "normal"),
        ] {
            let text = String::from(text);
            let mut s: Stream = Stream::new(StringView::from(&text), 0);
            let mut out = Vec::new();
            ParseShorthand(
                CSSPropertyID::kTextSpacing,
                &mut s,
                CSSParserMode::kHTMLStandardMode,
                &mut out,
            )
            .unwrap();
            assert!(s.AtEnd());
            assert_eq!(out.len(), 2);
            assert_eq!(out[0].Value().CssText().Utf8(), auto);
            assert_eq!(out[1].Value().CssText().Utf8(), trim);
        }
        for text in [
            "space-all space-first",
            "none normal",
            "normal normal normal",
            "auto",
        ] {
            let text = String::from(text);
            let mut s: Stream = Stream::new(StringView::from(&text), 0);
            let mut out = Vec::new();
            assert!(
                ParseShorthand(
                    CSSPropertyID::kTextSpacing,
                    &mut s,
                    CSSParserMode::kHTMLStandardMode,
                    &mut out
                )
                .is_err()
                    || !s.AtEnd()
            );
        }
        for (text, expected) in [
            ("auto", "auto"),
            ("2px", "2px"),
            ("2px 2px", "2px"),
            ("-3px 20%", "-3px 20%"),
            ("calc(2px + 20%)", "calc(2px + 20%)"),
        ] {
            let text = String::from(text);
            let mut s: Stream = Stream::new(StringView::from(&text), 0);
            let v = Consume(
                CSSPropertyID::kTextDecorationInset,
                &mut s,
                CSSParserMode::kHTMLStandardMode,
            )
            .unwrap();
            assert!(s.AtEnd());
            assert_eq!(v.CssText().Utf8(), expected);
        }
        for text in ["auto 1px", "1px 2px 3px", "none", "1"] {
            let text = String::from(text);
            let mut s: Stream = Stream::new(StringView::from(&text), 0);
            assert!(
                Consume(
                    CSSPropertyID::kTextDecorationInset,
                    &mut s,
                    CSSParserMode::kHTMLStandardMode
                )
                .is_err()
                    || !s.AtEnd()
            );
        }
    }
}
fn Ident<T: TokenStreamTokenizer>(s: &mut Stream<T>, ids: &[CSSValueID]) -> Option<Rc<Value>> {
    ids.contains(&s.Peek().Id())
        .then(|| values::identifier(s.ConsumeIncludingWhitespace().Id()))
}
// css_parsing_utils.cc:9380-9424. Restore when cap/ex needs an under edge.
fn Edge<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSValueID::*;
    if let Some(v) = Ident(s, &[kAuto]) {
        return Ok(v);
    }
    s.EnsureLookAhead();
    let save = s.Save();
    let over = Ident(s, &[kText, kCap, kEx]).ok_or_else(|| invalid(id))?;
    if let Some(under) = Ident(s, &[kText, kAlphabetic]) {
        if matches!(over.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==kText)
            && matches!(under.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==kText)
        {
            return Ok(over);
        }
        return Ok(values::list(
            vec![over, under],
            values::ListSeparator::Space,
        ));
    }
    if matches!(over.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==kText) {
        return Ok(over);
    }
    s.EnsureLookAhead();
    s.Restore(save);
    Err(invalid(id))
}
fn Percent<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    if IsMathFunction(s) {
        return ConsumeMath(
            id,
            s,
            &[crate::css_math_expression_node::CalculationResultCategory::Percent],
            crate::css_math_function_value::ValueRange::NonNegative,
        );
    }
    if s.Peek().GetType() == kPercentageToken && s.Peek().NumericValue() >= 0.0 {
        return Ok(values::numeric(
            s.ConsumeIncludingWhitespace().NumericValue(),
            UnitType::kPercentage,
        ));
    }
    Err(invalid(id))
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    use CSSValueID::*;
    match id {
        kTextBoxEdge => Edge(id, s),
        kTextBoxTrim => {
            Ident(s, &[kNone, kTrimStart, kTrimEnd, kTrimBoth]).ok_or_else(|| invalid(id))
        }
        kTextAutospace => Ident(s, &[kNormal, kNoAutospace]).ok_or_else(|| invalid(id)),
        kTextSpacingTrim => {
            Ident(s, &[kNormal, kSpaceAll, kSpaceFirst, kTrimStart]).ok_or_else(|| invalid(id))
        }
        kTextFit => {
            let first = Ident(s, &[kNone, kGrow, kShrink]).ok_or_else(|| invalid(id))?;
            let mut list = vec![first];
            if let Some(target) = Ident(s, &[kConsistent, kPerLine, kPerLineAll]) {
                list.push(target);
            }
            s.EnsureLookAhead();
            let save = s.Save();
            match Percent(id, s) {
                Ok(v) => list.push(v),
                Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
                Err(_) => {
                    s.EnsureLookAhead();
                    s.Restore(save);
                }
            }
            Ok(values::list(list, values::ListSeparator::Space))
        }
        kTextDecorationInset => {
            if let Some(v) = Ident(s, &[kAuto]) {
                return Ok(v);
            }
            let grammar = Grammar::Length {
                percent: true,
                nonnegative: false,
                quirks: false,
                keywords: &[],
            };
            let first = ConsumeLiteral(id, s, mode, grammar)?;
            s.EnsureLookAhead();
            let save = s.Save();
            let second = match ConsumeLiteral(id, s, mode, grammar) {
                Ok(v) => v,
                Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
                Err(_) => {
                    s.EnsureLookAhead();
                    s.Restore(save);
                    first.clone()
                }
            };
            Ok(Pair(first, second, true))
        }
        // longhands_custom.cc:11854-11859 uses the SAME decoration-line helper.
        kWebkitTextDecorationsInEffect => text_parser::Consume(kTextDecorationLine, s, mode)
            .map_err(|e| error(id, e.kind, e.operation)),
        _ => Err(unsupported(id, "text box consumer")),
    }
}
pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    _mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    use CSSPropertyID::*;
    use CSSValueID::*;
    match id {
        // shorthands_custom.cc:6251-6299. Omitted trim is trim-both, not initial.
        kTextBox => {
            let (mut trim, mut edge) = (None, None);
            if Ident(s, &[kNormal]).is_some() {
                trim = Some(values::identifier(kNone));
                edge = Some(values::identifier(kAuto));
            } else {
                while !s.AtEnd() && (trim.is_none() || edge.is_none()) {
                    if trim.is_none() {
                        if let Some(v) = Ident(s, &[kNone, kTrimStart, kTrimEnd, kTrimBoth]) {
                            trim = Some(v);
                            continue;
                        }
                    }
                    if edge.is_none() {
                        if let Ok(v) = Edge(kTextBoxEdge, s) {
                            edge = Some(v);
                            continue;
                        }
                    }
                    break;
                }
                if trim.is_none() && edge.is_none() {
                    return Err(invalid(id));
                }
            }
            out.push(make_expanded(
                kTextBoxTrim,
                id,
                trim.unwrap_or_else(|| values::identifier(kTrimBoth)),
                false,
            ));
            out.push(make_expanded(
                kTextBoxEdge,
                id,
                edge.unwrap_or_else(|| values::identifier(kAuto)),
                false,
            ));
            Ok(())
        }
        // shorthands_custom.cc:6358-6418. Internal; CSSTextSpacing is test-only.
        kTextSpacing => {
            let (mut autospace, mut trim) = (None, None);
            if Ident(s, &[kNone]).is_some() {
                autospace = Some(values::identifier(kNoAutospace));
                trim = Some(values::identifier(kSpaceAll));
            } else {
                let mut count = 0;
                while !s.AtEnd() {
                    count += 1;
                    if count > 2 {
                        break;
                    }
                    if Ident(s, &[kNormal]).is_some() {
                        continue;
                    }
                    if autospace.is_none() {
                        if let Some(v) = Ident(s, &[kNoAutospace]) {
                            autospace = Some(v);
                            continue;
                        }
                    }
                    if trim.is_none() {
                        if let Some(v) = Ident(s, &[kTrimStart, kSpaceAll, kSpaceFirst]) {
                            trim = Some(v);
                            continue;
                        }
                    }
                    break;
                }
                if count == 0 {
                    return Err(invalid(id));
                }
            }
            out.push(make_expanded(
                kTextAutospace,
                id,
                autospace.unwrap_or_else(|| values::identifier(kNormal)),
                false,
            ));
            out.push(make_expanded(
                kTextSpacingTrim,
                id,
                trim.unwrap_or_else(|| values::identifier(kNormal)),
                false,
            ));
            Ok(())
        }
        _ => Err(unsupported(id, "text box shorthand")),
    }
}
