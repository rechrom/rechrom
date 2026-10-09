// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium text consumers at stable runtime-feature defaults.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsTextProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kTextTransform
            | kTextOverflow
            | kTextIndent
            | kTextDecorationLine
            | kTextDecorationThickness
            | kTextUnderlineOffset
            | kTextUnderlinePosition
            | kTextJustify
    )
}

pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    use CSSValueID::*;
    match id {
        // css_parsing_utils.cc:9319-9378. FullWidth/FullSizeKana/MultiKeyword
        // are experimental; stable consumes one case keyword or math-auto.
        kTextTransform => {
            let keyword = stream.Peek().Id();
            if keyword == kNone {
                stream.ConsumeIncludingWhitespace();
                return Ok(values::identifier(keyword));
            }
            if !matches!(keyword, kCapitalize | kUppercase | kLowercase | kMathAuto) {
                return Err(invalid(id));
            }
            stream.ConsumeIncludingWhitespace();
            Ok(values::list(
                vec![values::identifier(keyword)],
                values::ListSeparator::Space,
            ))
        }
        // css_parsing_utils.cc:10121-10130. TextOverflowString is experimental.
        kTextOverflow => ConsumeLiteral(id, stream, mode, Grammar::Keywords(&["clip", "ellipsis"])),
        // longhands_custom.cc:9963-9974,10352-10361.
        kTextDecorationThickness | kTextUnderlineOffset => ConsumeLiteral(
            id,
            stream,
            mode,
            Grammar::Length {
                percent: true,
                nonnegative: false,
                quirks: false,
                keywords: if id == kTextDecorationThickness {
                    &["auto", "from-font"]
                } else {
                    &["auto"]
                },
            },
        ),
        // css_parsing_utils.cc:9221-9272, canonical order and unique flags.
        kTextDecorationLine => {
            let keyword = stream.Peek().Id();
            if keyword == kNone {
                stream.ConsumeIncludingWhitespace();
                return Ok(values::identifier(keyword));
            }
            if matches!(keyword, kSpellingError | kGrammarError) {
                stream.ConsumeIncludingWhitespace();
                return Ok(values::list(
                    vec![values::identifier(keyword)],
                    values::ListSeparator::Space,
                ));
            }
            let order = [kUnderline, kOverline, kLineThrough, kBlink];
            let mut flags = [false; 4];
            loop {
                let keyword = stream.Peek().Id();
                let Some(index) = order.iter().position(|&value| value == keyword) else {
                    break;
                };
                if flags[index] {
                    break;
                }
                flags[index] = true;
                stream.ConsumeIncludingWhitespace();
            }
            let list: Vec<_> = order
                .into_iter()
                .zip(flags)
                .filter_map(|(keyword, present)| present.then(|| values::identifier(keyword)))
                .collect();
            if list.is_empty() {
                return Err(invalid(id));
            }
            Ok(values::list(list, values::ListSeparator::Space))
        }
        // longhands_custom.cc:10000-10049. CssTextIndent is stable.
        kTextIndent => {
            let mut length = None;
            let mut hanging = false;
            let mut each_line = false;
            loop {
                if length.is_none() {
                    stream.EnsureLookAhead();
                    let savepoint = stream.Save();
                    if let Ok(value) = ConsumeLiteral(
                        id,
                        stream,
                        mode,
                        Grammar::Length {
                            percent: true,
                            nonnegative: false,
                            quirks: true,
                            keywords: &[],
                        },
                    ) {
                        length = Some(value);
                        continue;
                    }
                    stream.Restore(savepoint);
                }
                match stream.Peek().Id() {
                    kHanging if !hanging => hanging = true,
                    kEachLine if !each_line => each_line = true,
                    _ => break,
                }
                stream.ConsumeIncludingWhitespace();
            }
            let length = length.ok_or_else(|| invalid(id))?;
            if !hanging && !each_line {
                return Ok(length);
            }
            let mut list = vec![length];
            if hanging {
                list.push(values::identifier(kHanging));
            }
            if each_line {
                list.push(values::identifier(kEachLine));
            }
            Ok(values::list(list, values::ListSeparator::Space))
        }
        // longhands_custom.cc:10277-10307, canonical order independent of input.
        kTextUnderlinePosition => {
            if stream.Peek().Id() == kAuto {
                stream.ConsumeIncludingWhitespace();
                return Ok(values::identifier(kAuto));
            }
            let mut vertical = None;
            let mut horizontal = None;
            for _ in 0..2 {
                let keyword = stream.Peek().Id();
                match keyword {
                    kFromFont | kUnder if vertical.is_none() => vertical = Some(keyword),
                    CSSValueID::kLeft | CSSValueID::kRight if horizontal.is_none() => {
                        horizontal = Some(keyword)
                    }
                    _ => break,
                }
                stream.ConsumeIncludingWhitespace();
            }
            let list: Vec<_> = vertical
                .into_iter()
                .chain(horizontal)
                .map(values::identifier)
                .collect();
            if list.is_empty() {
                return Err(invalid(id));
            }
            Ok(values::list(list, values::ListSeparator::Space))
        }
        // longhands_custom.cc:10108-10123, legacy distribute normalization.
        kTextJustify => {
            let keyword = stream.Peek().Id();
            if !matches!(
                keyword,
                kAuto | kNone | kInterWord | kInterCharacter | kDistribute
            ) {
                return Err(invalid(id));
            }
            stream.ConsumeIncludingWhitespace();
            Ok(values::identifier(if keyword == kDistribute {
                kInterCharacter
            } else {
                keyword
            }))
        }
        _ => Err(unsupported(id, "text consumer")),
    }
}
