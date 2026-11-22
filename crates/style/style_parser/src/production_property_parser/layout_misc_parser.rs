// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Layout consumers translated from Blink's property/parser path.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsLayoutProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kAspectRatio
            | kColumnCount
            | kColumnWidth
            | kColumnHeight
            | kContain
            | kContainIntrinsicWidth
            | kContainIntrinsicHeight
            | kContainIntrinsicInlineSize
            | kContainIntrinsicBlockSize
            | kOrphans
            | kWidows
    )
}

pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    match id {
        kAspectRatio => ConsumeAspectRatio(id, stream, mode),
        kContain => ConsumeContain(id, stream),
        kColumnCount => {
            if stream.Peek().Id() == CSSValueID::kAuto {
                return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
            }
            ConsumeLiteral(id, stream, mode, Grammar::Integer { minimum: 1 })
        }
        kOrphans | kWidows => ConsumeLiteral(id, stream, mode, Grammar::Integer { minimum: 1 }),
        kColumnWidth | kColumnHeight => ConsumeLiteral(
            id,
            stream,
            CSSParserMode::kHTMLStandardMode,
            Grammar::Length {
                percent: false,
                nonnegative: true,
                quirks: false,
                keywords: &["auto"],
            },
        ),
        kContainIntrinsicWidth
        | kContainIntrinsicHeight
        | kContainIntrinsicInlineSize
        | kContainIntrinsicBlockSize => {
            // cpp: css_parsing_utils.cc:3458-3482 ConsumeIntrinsicSizeLonghand.
            let has_auto = stream.Peek().Id() == CSSValueID::kAuto;
            if has_auto {
                stream.ConsumeIncludingWhitespace();
            }
            let length = ConsumeLiteral(
                id,
                stream,
                mode,
                Grammar::Length {
                    percent: false,
                    nonnegative: true,
                    quirks: false,
                    keywords: &["none"],
                },
            )?;
            Ok(if has_auto {
                values::list(
                    vec![values::identifier(CSSValueID::kAuto), length],
                    values::ListSeparator::Space,
                )
            } else {
                length
            })
        }
        _ => Err(unsupported(id, "CSSProperty::ParseSingleValue")),
    }
}

// cpp: longhands_custom.cc:863-889; css_parsing_utils.cc:1662-1689.
fn ConsumeAspectRatio<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let mut has_auto = stream.Peek().Id() == CSSValueID::kAuto;
    if has_auto {
        stream.ConsumeIncludingWhitespace();
    }
    stream.EnsureLookAhead();
    let save = stream.Save();
    let parsed_ratio: Result<Rc<Value>, PropertyParseError> = (|| {
        let first = ConsumeLiteral(id, stream, mode, Grammar::Number { nonnegative: true })?;
        let second = if stream.Peek().GetType() == kDelimiterToken
            && stream.Peek().Delimiter() == b'/' as u16
        {
            stream.ConsumeIncludingWhitespace();
            ConsumeLiteral(id, stream, mode, Grammar::Number { nonnegative: true })?
        } else {
            values::numeric(1.0, UnitType::kInteger)
        };
        Ok(values::ratio(first, second))
    })();
    let ratio = match parsed_ratio {
        Ok(ratio) => ratio,
        Err(error) => {
            stream.EnsureLookAhead();
            stream.Restore(save);
            if error.kind == PropertyParseErrorKind::Unsupported {
                return Err(error);
            }
            return if has_auto {
                Ok(values::identifier(CSSValueID::kAuto))
            } else {
                Err(error)
            };
        }
    };
    if !has_auto && stream.Peek().Id() == CSSValueID::kAuto {
        stream.ConsumeIncludingWhitespace();
        has_auto = true;
    }
    let mut items = Vec::new();
    if has_auto {
        items.push(values::identifier(CSSValueID::kAuto));
    }
    items.push(ratio);
    Ok(values::list(items, values::ListSeparator::Space))
}

// cpp: longhands_custom.cc:2834-2886. Canonical list order is source-defined.
fn ConsumeContain<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSValueID::*;
    let first = stream.Peek().Id();
    if first == kNone {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    if matches!(first, kStrict | kContent) {
        return Ok(values::list(
            vec![values::identifier(stream.ConsumeIncludingWhitespace().Id())],
            values::ListSeparator::Space,
        ));
    }
    let mut slots = [None; 4];
    loop {
        let word = stream.Peek().Id();
        let slot = match word {
            kSize | kInlineSize => 0,
            kLayout => 1,
            kStyle => 2,
            kPaint => 3,
            _ => break,
        };
        if slots[slot].is_some() {
            break;
        }
        slots[slot] = Some(word);
        stream.ConsumeIncludingWhitespace();
    }
    let items: Vec<_> = slots
        .into_iter()
        .flatten()
        .map(values::identifier)
        .collect();
    if items.is_empty() {
        return Err(invalid(id));
    }
    Ok(values::list(items, values::ListSeparator::Space))
}

// cpp: shorthands_custom.cc:1934-1989; css_parsing_utils.cc:6089-6108.
fn ConsumeColumnWidthOrCount<T: TokenStreamTokenizer>(
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    width: &mut Option<Rc<Value>>,
    count: &mut Option<Rc<Value>>,
) -> Result<bool, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kAuto {
        stream.ConsumeIncludingWhitespace();
        return Ok(true);
    }
    if width.is_none() {
        stream.EnsureLookAhead();
        let save = stream.Save();
        match Consume(CSSPropertyID::kColumnWidth, stream, mode) {
            Ok(value) => {
                *width = Some(value);
                return Ok(true);
            }
            Err(error) if error.kind == PropertyParseErrorKind::Unsupported => return Err(error),
            Err(_) => {
                stream.EnsureLookAhead();
                stream.Restore(save);
            }
        }
    }
    if count.is_none() {
        stream.EnsureLookAhead();
        let save = stream.Save();
        match Consume(CSSPropertyID::kColumnCount, stream, mode) {
            Ok(value) => {
                *count = Some(value);
                return Ok(true);
            }
            Err(error) if error.kind == PropertyParseErrorKind::Unsupported => return Err(error),
            Err(_) => {
                stream.EnsureLookAhead();
                stream.Restore(save);
            }
        }
    }
    Ok(false)
}

pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    use CSSPropertyID::*;
    if id == kContainIntrinsicSize {
        // cpp: shorthands_custom.cc:2044-2052; ConsumeShorthandVia2Longhands.
        let width = Consume(kContainIntrinsicWidth, stream, mode)?;
        let height = if at_value_end(stream) {
            width.clone()
        } else {
            Consume(kContainIntrinsicHeight, stream, mode)?
        };
        out.push(make_expanded(kContainIntrinsicWidth, id, width, false));
        out.push(make_expanded(kContainIntrinsicHeight, id, height, false));
        return Ok(());
    }
    let (mut width, mut count) = (None, None);
    if !ConsumeColumnWidthOrCount(stream, mode, &mut width, &mut count)? {
        return Err(invalid(id));
    }
    ConsumeColumnWidthOrCount(stream, mode, &mut width, &mut count)?;
    let height =
        if stream.Peek().GetType() == kDelimiterToken && stream.Peek().Delimiter() == b'/' as u16 {
            stream.ConsumeIncludingWhitespace();
            // The shorthand uses current mode for the height alternative.
            ConsumeLiteral(
                kColumnHeight,
                stream,
                mode,
                Grammar::Length {
                    percent: false,
                    nonnegative: true,
                    quirks: false,
                    keywords: &["auto"],
                },
            )?
        } else {
            values::identifier(CSSValueID::kAuto)
        };
    for (longhand, value) in [
        (
            kColumnWidth,
            width.unwrap_or_else(|| values::identifier(CSSValueID::kAuto)),
        ),
        (
            kColumnCount,
            count.unwrap_or_else(|| values::identifier(CSSValueID::kAuto)),
        ),
        (kColumnHeight, height),
        (kColumnWrap, values::identifier(CSSValueID::kAuto)),
    ] {
        out.push(make_expanded(longhand, id, value, false));
    }
    Ok(())
}
