// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Token consumers for counter directives, quotes and list-style.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsListCounterProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kCounterIncrement
            | kCounterReset
            | kCounterSet
            | kQuotes
            | kListStyleType
            | kListStyleImage
            | kListStylePosition
    )
}
fn CustomIdent<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    CustomIdentWithNone(id, stream, false)
}
// Shared ConsumeCustomIdent grammar. The source excludes none only where
// the caller requests ConsumeCustomIdentExcludingNone.
pub(super) fn CustomIdentWithNone<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    allow_none: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().FunctionId() == Some(CSSValueID::kIdent) {
        return Err(unsupported(id, "ConsumeCustomIdent ident() collaborator"));
    }
    if stream.Peek().GetType() != kIdentToken
        || values::wide(stream.Peek().Id()).is_some()
        || stream.Peek().Id() == CSSValueID::kDefault
        || !allow_none && stream.Peek().Id() == CSSValueID::kNone
    {
        return Err(invalid(id));
    }
    Ok(values::custom_ident(
        &stream.ConsumeIncludingWhitespace().Value().ToString(),
        CSSPropertyID::kInvalid,
    ))
}
// cpp: css_parsing_utils.cc:6120-6158,6167-6190; reversed() is experimental at runtime defaults.
fn Counters<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kNone {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    let mut list = Vec::new();
    loop {
        if stream.Peek().FunctionId() == Some(CSSValueID::kReversed) {
            return Err(if id == CSSPropertyID::kCounterReset {
                unsupported(
                    id,
                    "ConsumeCounters CSSCounterResetReversed runtime feature",
                )
            } else {
                invalid(id)
            });
        }
        let identifier = CustomIdent(id, stream)?;
        let value = if stream.Peek().GetType() == kNumberToken {
            if stream.Peek().GetNumericValueType() != NumericValueType::kIntegerValueType {
                return Err(invalid(id));
            }
            values::numeric(
                stream.ConsumeIncludingWhitespace().NumericValue(),
                UnitType::kInteger,
            )
        } else if IsMathFunction(stream) {
            ConsumeLiteral(id, stream, mode, Grammar::Integer { minimum: i32::MIN })?
        } else {
            values::numeric(
                if id == CSSPropertyID::kCounterIncrement {
                    1.0
                } else {
                    0.0
                },
                UnitType::kInteger,
            )
        };
        list.push(values::counter(identifier, Some(value), false));
        if stream.Peek().GetType() != kIdentToken && !is_function(stream) {
            break;
        }
    }
    Ok(values::list(list, values::ListSeparator::Space))
}
// cpp: longhands_custom.cc:8388-8415.
fn Quotes<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    if matches!(stream.Peek().Id(), CSSValueID::kNone | CSSValueID::kAuto) {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    let mut strings = Vec::new();
    while stream.Peek().GetType() == kStringToken {
        strings.push(values::string(
            stream.ConsumeIncludingWhitespace().Value().ToString(),
        ));
    }
    if strings.is_empty() || strings.len() % 2 != 0 {
        return Err(invalid(id));
    }
    Ok(values::list(strings, values::ListSeparator::Space))
}
// cpp: longhands_custom.cc:6610-6630; css_parsing_utils.cc:9824-9840.
fn ListType<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kNone {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    if stream.Peek().GetType() == kStringToken {
        return Ok(values::string(
            stream.ConsumeIncludingWhitespace().Value().ToString(),
        ));
    }
    if stream.Peek().FunctionId() == Some(CSSValueID::kSymbols) {
        return Err(unsupported(
            id,
            "ConsumeCounterStyleSymbolsFunction runtime feature/native symbols collaborator",
        ));
    }
    let value = CustomIdent(id, stream)?;
    let CSSValuePayload::kCustomIdentClass(name) = value.Payload() else {
        unreachable!()
    };
    let original = String::from_utf16(name.name.utf16_units().unwrap_or_default());
    let lower = original.Utf8().to_ascii_lowercase();
    // cpp: css_parsing_utils.cc:9814-9822; UA map keys come from the shared
    // CollectUACounterStyleRules metadata. Authored names retain case.
    if mode == CSSParserMode::kUASheetMode {
        debug_assert_eq!(original.Utf8(), lower);
    }
    if mode != CSSParserMode::kUASheetMode && UA_COUNTER_STYLE_NAMES.contains(&lower.as_str()) {
        Ok(values::custom_ident(
            &String::from(lower),
            CSSPropertyID::kInvalid,
        ))
    } else {
        Ok(value)
    }
}
pub(super) fn ListImage<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kNone {
        return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
    }
    if let Some(image) = ConsumeContentImage(id, stream)? {
        return Ok(image);
    }
    if matches!(
        stream.Peek().FunctionId(),
        Some(CSSValueID::kLinearGradient | CSSValueID::kRepeatingLinearGradient)
    ) {
        return ConsumeLinearGradient(id, stream, mode);
    }
    Err(if is_function(stream) {
        unsupported(id, "ConsumeImage generated/image-set subtype")
    } else {
        invalid(id)
    })
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    match id {
        kCounterIncrement | kCounterReset | kCounterSet => Counters(id, stream, mode),
        kQuotes => Quotes(id, stream),
        kListStyleType => ListType(id, stream, mode),
        kListStyleImage => ListImage(id, stream, mode),
        kListStylePosition => {
            ConsumeLiteral(id, stream, mode, Grammar::Keywords(&["inside", "outside"]))
        }
        _ => Err(unsupported(id, "list/counter consumer")),
    }
}
// cpp: shorthands_custom.cc:4173-4263. none is consumed once before longhands;
// a second none can be the image; omitted fields reset to CSSInitialValue.
pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    let mut none = None;
    let mut position = None;
    let mut image = None;
    let mut kind = None;
    loop {
        if none.is_none() && stream.Peek().Id() == CSSValueID::kNone {
            none = Some(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
            continue;
        }
        let mut consumed = false;
        for (property, field) in [
            (CSSPropertyID::kListStylePosition, &mut position),
            (CSSPropertyID::kListStyleImage, &mut image),
            (CSSPropertyID::kListStyleType, &mut kind),
        ] {
            if field.is_some() {
                continue;
            }
            stream.EnsureLookAhead();
            let save = stream.Save();
            match Consume(property, stream, mode) {
                Ok(value) => {
                    *field = Some(value);
                    consumed = true;
                    break;
                }
                Err(failure) if failure.kind == PropertyParseErrorKind::Unsupported => {
                    // Image functions and list-name functions belong to different branches.
                    stream.Restore(save);
                    if property == CSSPropertyID::kListStyleImage
                        && matches!(
                            stream.Peek().FunctionId(),
                            Some(CSSValueID::kSymbols | CSSValueID::kIdent)
                        )
                    {
                        continue;
                    }
                    return Err(failure);
                }
                Err(_) => stream.Restore(save),
            }
        }
        if !consumed || stream.AtEnd() {
            break;
        }
    }
    if none.is_none() && position.is_none() && image.is_none() && kind.is_none() {
        return Err(invalid(id));
    }
    if let Some(none) = none {
        if kind.is_none() {
            kind = Some(none);
        } else if image.is_none() {
            image = Some(none);
        } else {
            return Err(invalid(id));
        }
    }
    for (property, value) in [
        (CSSPropertyID::kListStylePosition, position),
        (CSSPropertyID::kListStyleImage, image),
        (CSSPropertyID::kListStyleType, kind),
    ] {
        out.push(make_expanded(
            property,
            id,
            value.unwrap_or_else(|| values::wide(CSSValueID::kInitial).unwrap()),
            false,
        ));
    }
    Ok(())
}
