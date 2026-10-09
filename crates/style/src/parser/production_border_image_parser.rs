// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium border-image longhands/components/shorthand token consumers.
#![allow(non_snake_case)]
use super::*;
use crate::production_border_image_values::{quad, slice};

pub(super) fn IsBorderImageProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kBorderImageSource
            | kBorderImageSlice
            | kBorderImageWidth
            | kBorderImageOutset
            | kBorderImageRepeat
            | kWebkitBorderImage
            | kWebkitMaskBoxImageSource
            | kWebkitMaskBoxImageSlice
            | kWebkitMaskBoxImageWidth
            | kWebkitMaskBoxImageOutset
            | kWebkitMaskBoxImageRepeat
    )
}
fn Repeat<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    let valid = |k| {
        matches!(
            k,
            CSSValueID::kStretch | CSSValueID::kRepeat | CSSValueID::kRound | CSSValueID::kSpace
        )
    };
    if !valid(s.Peek().Id()) {
        return Ok(None);
    }
    let first = values::identifier(s.ConsumeIncludingWhitespace().Id());
    let second = if valid(s.Peek().Id()) {
        values::identifier(s.ConsumeIncludingWhitespace().Id())
    } else {
        first.clone()
    };
    let _ = id;
    Ok(Some(Rc::new(Value::new(CSSValuePayload::kValuePairClass(
        values::CSSValuePair {
            first,
            second,
            drop_identical: true,
        },
    )))))
}
// css_parsing_utils.cc:5663-5699 uses percent then number, with the same
// category-restoring typed math consumer used by animation numeric probes.
fn SliceSide<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    use crate::{
        css_math_expression_node::CalculationResultCategory as C,
        css_math_function_value::ValueRange as R,
    };
    for (category, token, unit) in [
        (C::Percent, kPercentageToken, UnitType::kPercentage),
        (C::Number, kNumberToken, UnitType::kNumber),
    ] {
        if let Some(v) = ConsumeAnimationNumericMath(id, s, category, R::NonNegative)? {
            return Ok(Some(v));
        }
        if s.Peek().GetType() == token && s.Peek().NumericValue() >= 0.0 {
            return Ok(Some(values::numeric(
                s.ConsumeIncludingWhitespace().NumericValue(),
                unit,
            )));
        }
    }
    Ok(None)
}
fn Sides(values: Vec<Rc<Value>>) -> [Rc<Value>; 4] {
    let top = values[0].clone();
    let right = values.get(1).cloned().unwrap_or_else(|| top.clone());
    let bottom = values.get(2).cloned().unwrap_or_else(|| top.clone());
    let left = values.get(3).cloned().unwrap_or_else(|| right.clone());
    [top, right, bottom, left]
}
fn Slice<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    default_fill: bool,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    let fill = s.Peek().Id() == CSSValueID::kFill;
    if fill {
        s.ConsumeIncludingWhitespace();
    }
    let mut parts = Vec::new();
    for _ in 0..4 {
        let Some(v) = SliceSide(id, s)? else {
            break;
        };
        parts.push(v);
    }
    if parts.is_empty() {
        return Ok(None);
    }
    let after = s.Peek().Id() == CSSValueID::kFill;
    if after {
        if fill {
            return Err(invalid(id));
        }
        s.ConsumeIncludingWhitespace();
    }
    Ok(Some(slice(
        quad(Sides(parts)),
        fill || after || default_fill,
    )))
}
// css_parsing_utils.cc:5703-5765. Number is attempted before standard-mode
// length; auto/percentage exist only in width, never outset.
fn Quad<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    width: bool,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    use crate::{
        css_math_expression_node::CalculationResultCategory as C,
        css_math_function_value::ValueRange as R,
    };
    let mut parts = Vec::new();
    for _ in 0..4 {
        let value = if let Some(v) = ConsumeAnimationNumericMath(id, s, C::Number, R::NonNegative)?
        {
            v
        } else if s.Peek().GetType() == kNumberToken && s.Peek().NumericValue() >= 0.0 {
            values::numeric(
                s.ConsumeIncludingWhitespace().NumericValue(),
                UnitType::kNumber,
            )
        } else if width && s.Peek().Id() == CSSValueID::kAuto {
            values::identifier(s.ConsumeIncludingWhitespace().Id())
        } else {
            let token = s.Peek().clone();
            if !IsMathFunction(s) && !matches!(token.GetType(), kDimensionToken | kPercentageToken)
            {
                break;
            }
            let v = ConsumeLiteral(
                id,
                s,
                CSSParserMode::kHTMLStandardMode,
                Grammar::Length {
                    percent: width,
                    nonnegative: true,
                    quirks: false,
                    keywords: &[],
                },
            )?;
            v
        };
        parts.push(value);
    }
    Ok((!parts.is_empty()).then(|| quad(Sides(parts))))
}
#[derive(Default)]
struct Components {
    source: Option<Rc<Value>>,
    slice: Option<Rc<Value>>,
    width: Option<Rc<Value>>,
    outset: Option<Rc<Value>>,
    repeat: Option<Rc<Value>>,
}
// css_parsing_utils.cc:5596-5648. Each component is consumed once; slash
// width/outset belong to the slice component and absent pieces stay absent.
fn Components<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
    fill: bool,
) -> Result<Components, PropertyParseError> {
    let mut c = Components::default();
    loop {
        if c.source.is_none() {
            if s.Peek().Id() == CSSValueID::kNone
                || s.Peek().GetType() == kUrlToken
                || s.Peek().FunctionId() == Some(CSSValueID::kUrl)
                || is_function(s) && !IsMathFunction(s)
            {
                c.source = Some(list_counter_parser::ListImage(id, s, mode)?);
                continue;
            }
        }
        if c.repeat.is_none() {
            c.repeat = Repeat(id, s)?;
            if c.repeat.is_some() {
                continue;
            }
        }
        if c.slice.is_none() {
            c.slice = Slice(id, s, fill)?;
            if c.slice.is_none() {
                break;
            }
            if s.Peek().GetType() == kDelimiterToken && s.Peek().Delimiter() == b'/' as u16 {
                s.ConsumeIncludingWhitespace();
                c.width = Quad(id, s, true)?;
                if s.Peek().GetType() == kDelimiterToken && s.Peek().Delimiter() == b'/' as u16 {
                    s.ConsumeIncludingWhitespace();
                    c.outset = Quad(id, s, false)?;
                    if c.outset.is_none() {
                        return Err(invalid(id));
                    }
                } else if c.width.is_none() {
                    return Err(invalid(id));
                }
            }
        } else {
            break;
        }
        if at_value_end(s) {
            break;
        }
    }
    if c.source.is_none() && c.slice.is_none() && c.repeat.is_none() {
        return Err(invalid(id));
    }
    Ok(c)
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    match id {
        kBorderImageSource | kWebkitMaskBoxImageSource => {
            list_counter_parser::ListImage(id, s, mode)
        }
        kBorderImageSlice | kWebkitMaskBoxImageSlice => {
            Slice(id, s, id == kWebkitMaskBoxImageSlice)?.ok_or_else(|| invalid(id))
        }
        kBorderImageWidth | kWebkitMaskBoxImageWidth => {
            Quad(id, s, true)?.ok_or_else(|| invalid(id))
        }
        kBorderImageOutset | kWebkitMaskBoxImageOutset => {
            Quad(id, s, false)?.ok_or_else(|| invalid(id))
        }
        kBorderImageRepeat | kWebkitMaskBoxImageRepeat => Repeat(id, s)?.ok_or_else(|| invalid(id)),
        // css_border_image.cc:24-59 preserves source nested slash-list order.
        kWebkitBorderImage => ConsumeWebkitBorderImage(id,s,mode),
        _ => Err(invalid(id)),
    }
}
pub(super) fn ConsumeWebkitBorderImage<T: TokenStreamTokenizer>(id:CSSPropertyID,s:&mut Stream<T>,mode:CSSParserMode)->Result<Rc<Value>,PropertyParseError> {
            let c = Components(id, s, mode, true)?;
            let mut list = Vec::new();
            if let Some(v) = c.source {
                list.push(v);
            }
            if c.width.is_some() || c.outset.is_some() {
                let slash = c.slice.into_iter().chain(c.width).chain(c.outset).collect();
                list.push(values::list(slash, values::ListSeparator::Slash));
            } else if let Some(v) = c.slice {
                list.push(v);
            }
            if let Some(v) = c.repeat {
                list.push(v);
            }
            Ok(values::list(list, values::ListSeparator::Space))
}
pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    let mask = id == CSSPropertyID::kWebkitMaskBoxImage;
    let c = Components(id, s, mode, mask)?;
    // shorthands_custom.cc:5960-6006: omitted mask-box components are
    // CSSInitialValue, with default fill supplied to the shared slice consumer.
    if mask {
        for (property, value) in [
            (CSSPropertyID::kWebkitMaskBoxImageSource, c.source),
            (CSSPropertyID::kWebkitMaskBoxImageSlice, c.slice),
            (CSSPropertyID::kWebkitMaskBoxImageWidth, c.width),
            (CSSPropertyID::kWebkitMaskBoxImageOutset, c.outset),
            (CSSPropertyID::kWebkitMaskBoxImageRepeat, c.repeat),
        ] {
            out.push(make_expanded(
                property,
                id,
                value.unwrap_or_else(|| values::wide(CSSValueID::kInitial).unwrap()),
                false,
            ));
        }
        return Ok(());
    }
    // shorthands_custom.cc:772-817; source InitialValue payloads, rather than
    // shorthand implicit flags. Omitted components reset real native fields.
    let hundred = values::numeric(100.0, UnitType::kPercentage);
    let one = values::numeric(1.0, UnitType::kNumber);
    let zero = values::numeric(0.0, UnitType::kNumber);
    let stretch = values::identifier(CSSValueID::kStretch);
    let initial_repeat = Rc::new(Value::new(CSSValuePayload::kValuePairClass(
        values::CSSValuePair {
            first: stretch.clone(),
            second: stretch,
            drop_identical: true,
        },
    )));
    for (property, value, initial) in [
        (
            CSSPropertyID::kBorderImageSource,
            c.source,
            values::identifier(CSSValueID::kNone),
        ),
        (
            CSSPropertyID::kBorderImageSlice,
            c.slice,
            slice(quad(std::array::from_fn(|_| hundred.clone())), false),
        ),
        (
            CSSPropertyID::kBorderImageWidth,
            c.width,
            quad(std::array::from_fn(|_| one.clone())),
        ),
        (
            CSSPropertyID::kBorderImageOutset,
            c.outset,
            quad(std::array::from_fn(|_| zero.clone())),
        ),
        (CSSPropertyID::kBorderImageRepeat, c.repeat, initial_repeat),
    ] {
        out.push(make_expanded(property, id, value.unwrap_or(initial), false));
    }
    Ok(())
}
