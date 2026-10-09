// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Typography consumers translated from Chromium's longhands and parsing utils.
#![allow(non_snake_case)]
use super::*;
pub(super) fn IsTypographyProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kTextEmphasisStyle
            | kTextEmphasisPosition
            | kHyphenateCharacter
            | kHyphenateLimitChars
            | kRubyPosition
            | kRubyOverhang
            | kTextSizeAdjust
            | kTextDecorationSkipSpaces
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hidden_typography_skip_spaces_consumer_preserves_canonical_list() {
        for (text, expected) in [
            ("none", "none"),
            ("all", "all"),
            ("end start", "start end"),
            ("start", "start"),
            ("end", "end"),
        ] {
            let text = String::from(text);
            let mut s: Stream = Stream::new(StringView::from(&text), 0);
            let value = Consume(
                CSSPropertyID::kTextDecorationSkipSpaces,
                &mut s,
                CSSParserMode::kHTMLStandardMode,
            )
            .unwrap();
            assert!(s.AtEnd());
            assert_eq!(value.CssText().Utf8(), expected);
        }
        for text in ["start start", "all end", "auto"] {
            let text = String::from(text);
            let mut s: Stream = Stream::new(StringView::from(&text), 0);
            assert!(
                Consume(
                    CSSPropertyID::kTextDecorationSkipSpaces,
                    &mut s,
                    CSSParserMode::kHTMLStandardMode
                )
                .is_err()
                    || !s.AtEnd()
            );
        }
    }
}
fn Ident<T: TokenStreamTokenizer>(s: &mut Stream<T>, allowed: &[CSSValueID]) -> Option<Rc<Value>> {
    allowed
        .contains(&s.Peek().Id())
        .then(|| values::identifier(s.ConsumeIncludingWhitespace().Id()))
}
fn StringValue<T: TokenStreamTokenizer>(s: &mut Stream<T>) -> Option<Rc<Value>> {
    (s.Peek().GetType() == kStringToken)
        .then(|| values::string(s.ConsumeIncludingWhitespace().Value().ToString()))
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    use CSSValueID::*;
    use CSSValueID::{kAll, kLeft, kRight};
    match id {
        // longhands_custom.cc:11990-12028. Canonical fill precedes shape.
        kTextEmphasisStyle => {
            if let Some(v) = Ident(s, &[kNone]) {
                return Ok(v);
            }
            if let Some(v) = StringValue(s) {
                return Ok(v);
            }
            let mut fill = Ident(s, &[kFilled, kOpen]);
            let shape = Ident(s, &[kDot, kCircle, kDoubleCircle, kTriangle, kSesame]);
            if fill.is_none() {
                fill = Ident(s, &[kFilled, kOpen]);
            }
            match (fill, shape) {
                (Some(f), Some(m)) => Ok(values::list(vec![f, m], values::ListSeparator::Space)),
                (Some(v), None) | (None, Some(v)) => Ok(v),
                _ => Err(invalid(id)),
            }
        }
        // longhands_custom.cc:11901-11958, auto is experimental default off.
        kTextEmphasisPosition => {
            if foundation::RuntimeEnabledFeatures::TextEmphasisPositionAutoEnabled() {
                if let Some(v) = Ident(s, &[kAuto]) {
                    return Ok(values::list(vec![v], values::ListSeparator::Space));
                }
            }
            let (mut vertical, mut horizontal) = (None, None);
            for _ in 0..2 {
                match s.Peek().Id() {
                    kOver | kUnder if vertical.is_none() => {
                        vertical = Some(values::identifier(s.ConsumeIncludingWhitespace().Id()))
                    }
                    kLeft | kRight if horizontal.is_none() => {
                        horizontal = Some(values::identifier(s.ConsumeIncludingWhitespace().Id()))
                    }
                    _ => break,
                }
            }
            let Some(v) = vertical else {
                return Err(invalid(id));
            };
            let mut list = vec![v];
            if let Some(h) = horizontal {
                list.push(h);
            }
            Ok(values::list(list, values::ListSeparator::Space))
        }
        kHyphenateCharacter => Ident(s, &[kAuto])
            .or_else(|| StringValue(s))
            .ok_or_else(|| invalid(id)),
        // css_parsing_utils.cc:8268-8290. Missing entries stay auto in converter.
        kHyphenateLimitChars => {
            let mut list = Vec::new();
            while list.len() < 3 && !s.AtEnd() {
                s.EnsureLookAhead();
                let save = s.Save();
                match ConsumeLiteral(id, s, mode, Grammar::Integer { minimum: 1 }) {
                    Ok(v) => {
                        list.push(v);
                        continue;
                    }
                    Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
                    Err(_) => {
                        s.EnsureLookAhead();
                        s.Restore(save);
                    }
                }
                if let Some(v) = Ident(s, &[kAuto]) {
                    list.push(v);
                } else {
                    break;
                }
            }
            if list.is_empty() {
                Err(invalid(id))
            } else {
                Ok(values::list(list, values::ListSeparator::Space))
            }
        }
        kRubyPosition => Ident(s, &[kOver, kUnder]).ok_or_else(|| invalid(id)),
        kRubyOverhang => {
            if Ident(s, &[kNone, kSpaces]).is_some() {
                Ok(values::identifier(kSpaces))
            } else {
                Ident(s, &[kAuto]).ok_or_else(|| invalid(id))
            }
        }
        kTextSizeAdjust => {
            if let Some(v) = Ident(s, &[kAuto, kNone]) {
                return Ok(v);
            }
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
        // css_parsing_utils.cc:9276-9314. Internal consumer; Exposure is gated.
        kTextDecorationSkipSpaces => {
            if let Some(v) = Ident(s, &[kNone]) {
                return Ok(v);
            }
            if let Some(v) = Ident(s, &[kAll]) {
                return Ok(values::list(vec![v], values::ListSeparator::Space));
            }
            let (mut start, mut end) = (None, None);
            loop {
                match s.Peek().Id() {
                    kStart if start.is_none() => start = Ident(s, &[kStart]),
                    kEnd if end.is_none() => end = Ident(s, &[kEnd]),
                    _ => break,
                }
            }
            let list = start.into_iter().chain(end).collect::<Vec<_>>();
            if list.is_empty() {
                Err(invalid(id))
            } else {
                Ok(values::list(list, values::ListSeparator::Space))
            }
        }
        _ => Err(unsupported(id, "typography consumer")),
    }
}
