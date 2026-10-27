// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! css_parsing_utils.cc:5384-5548 ParseBackgroundOrMask, mask branch.
#![allow(non_snake_case)]
use super::*;
const FIELDS: [CSSPropertyID; 9] = [
    CSSPropertyID::kMaskImage,
    CSSPropertyID::kWebkitMaskPositionX,
    CSSPropertyID::kWebkitMaskPositionY,
    CSSPropertyID::kMaskSize,
    CSSPropertyID::kMaskRepeat,
    CSSPropertyID::kMaskOrigin,
    CSSPropertyID::kMaskClip,
    CSSPropertyID::kMaskComposite,
    CSSPropertyID::kMaskMode,
];
fn Initial(index: usize) -> Rc<Value> {
    use CSSValueID::*;
    match index {
        0 => values::identifier(kNone),
        1 | 2 => values::numeric(0.0, UnitType::kPercentage),
        3 => values::identifier(kAuto),
        4 => Rc::new(Value::new(CSSValuePayload::kRepeatStyleClass(
            values::CSSRepeatStyleValue {
                x: values::identifier(kRepeat),
                y: values::identifier(kRepeat),
            },
        ))),
        5 | 6 => values::identifier(kBorderBox),
        7 => values::identifier(kAdd),
        8 => values::identifier(kMatchSource),
        _ => unreachable!(),
    }
}
pub(super) fn ParseShorthand<
    T: TokenStreamTokenizer,
    P: CSSParserContextPlatform<URL = crate::production_css_value::ProductionURL>,
>(
    id: CSSPropertyID,
    unresolved: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
    context: &CSSParserContext<P>,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    let mut longhands: [Vec<Rc<Value>>; 9] = std::array::from_fn(|_| Vec::new());
    loop {
        let mut parsed: [Option<Rc<Value>>; 9] = std::array::from_fn(|_| None);
        let mut found_any = false;
        loop {
            let mut found = false;
            // A slash size must immediately follow the position consumed in
            // this same scan, as in the source; it cannot follow another item.
            let mut position_in_scan = false;
            for index in 0..9 {
                if parsed[index].is_some() || index == 2 {
                    continue;
                }
                let property = FIELDS[index];
                s.EnsureLookAhead();
                let save = s.Save();
                let result = match index {
                    0 => {
                        if s.Peek().Id() == CSSValueID::kNone
                            || s.Peek().GetType() == kUrlToken
                            || is_function(s) && !IsMathFunction(s)
                        {
                            Some(list_counter_parser::ListImage(property, s, mode, context)?)
                        } else {
                            None
                        }
                    }
                    1 => match ConsumePosition(property, s, mode, false, true) {
                        Ok((x, y)) => {
                            parsed[2] = Some(y);
                            position_in_scan = true;
                            Some(x)
                        }
                        Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
                        Err(_) => None,
                    },
                    3 => {
                        if s.Peek().GetType() == kDelimiterToken
                            && s.Peek().Delimiter() == b'/' as u16
                        {
                            s.ConsumeIncludingWhitespace();
                            let size = ConsumeBackgroundSize(property, s, mode, false)?;
                            if !position_in_scan {
                                return Err(invalid(id));
                            }
                            Some(size)
                        } else {
                            None
                        }
                    }
                    4 => ConsumeRepeatStyleValue(property, s).ok(),
                    // Alias shorthand retains prefixed boxes but uses the
                    // standard mask-composite grammar (source component path).
                    _ => ConsumeLayer(property, s, mode, index != 7 && unresolved != id).ok(),
                };
                if let Some(value) = result {
                    parsed[index] = Some(value);
                    found = true;
                    found_any = true;
                } else {
                    s.EnsureLookAhead();
                    s.Restore(save);
                }
            }
            if !found || at_value_end(s) || s.Peek().GetType() == kCommaToken {
                break;
            }
        }
        if !found_any {
            return Err(invalid(id));
        }
        if parsed[6].is_none() {
            parsed[6] = parsed[5].clone();
        }
        for (index, value) in parsed.into_iter().enumerate() {
            longhands[index].push(value.unwrap_or_else(|| Initial(index)));
        }
        if s.Peek().GetType() != kCommaToken {
            break;
        }
        s.ConsumeIncludingWhitespace();
    }
    for (property, mut values) in FIELDS.into_iter().zip(longhands) {
        let value = if values.len() == 1 {
            values.pop().unwrap()
        } else {
            values::list(values, values::ListSeparator::Comma)
        };
        out.push(make_expanded(property, id, value, false));
    }
    Ok(())
}
