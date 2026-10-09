// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! css_parsing_utils.cc:8861-8927,8984-9028,9039-9063;
//! shorthands_custom.cc:4393-4474, ordered Offset shorthand expansion.
#![allow(non_snake_case)]
use super::*;
use CSSPropertyID as P;
use CSSValueID::*;
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(id, P::kOffsetPath | P::kOffsetRotate | P::kOffsetDistance)
}
fn CoordBox<T: TokenStreamTokenizer>(s: &mut Stream<T>) -> Option<Rc<Value>> {
    if matches!(
        s.Peek().Id(),
        kContentBox | kPaddingBox | kBorderBox | kFillBox | kStrokeBox | kViewBox
    ) {
        Some(values::identifier(s.ConsumeIncludingWhitespace().Id()))
    } else {
        None
    }
}
fn Ray<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if s.Peek().FunctionId() != Some(kRay) {
        return Ok(None);
    }
    let mut guard = RestoringBlockGuard::new(s);
    guard.ConsumeWhitespace();
    let mut angle = None;
    let mut size = None;
    let mut contain = None;
    let mut center = None;
    while !guard.AtEnd() {
        if angle.is_none() {
            angle = transform_parser::ConsumeAngle(id, &mut guard, mode)?;
            if angle.is_some() {
                continue;
            }
        }
        if size.is_none()
            && matches!(
                guard.Peek().Id(),
                kClosestSide | kClosestCorner | kFarthestSide | kFarthestCorner | kSides
            )
        {
            size = Some(values::identifier(guard.ConsumeIncludingWhitespace().Id()));
            continue;
        }
        if contain.is_none() && guard.Peek().Id() == kContain {
            contain = Some(values::identifier(guard.ConsumeIncludingWhitespace().Id()));
            continue;
        }
        if center.is_none() && guard.Peek().Id() == kAt {
            guard.ConsumeIncludingWhitespace();
            center = Some(ConsumePosition(id, &mut guard, mode, false, false)?);
            continue;
        }
        return Err(invalid(id));
    }
    let angle = angle.ok_or_else(|| invalid(id))?;
    guard.Release();
    drop(guard);
    s.ConsumeWhitespace();
    Ok(Some(Rc::new(Value::new(CSSValuePayload::kRayClass(
        crate::production_motion_value::CSSRayValue {
            angle,
            size: size.unwrap_or_else(|| values::identifier(kClosestSide)),
            contain,
            center,
        },
    )))))
}
fn Path<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if s.Peek().Id() == kNone {
        s.ConsumeIncludingWhitespace();
        return Ok(Some(values::identifier(kNone)));
    }
    let mut coord = CoordBox(s);
    let ray = Ray(id, s, mode)?;
    if ray.is_none()
        && (s.Peek().GetType() == kUrlToken
            || matches!(
                s.Peek().FunctionId(),
                Some(
                    kUrl | kPath | kCircle | kEllipse | kInset | kRect | kXywh | kPolygon | kShape
                )
            ))
    {
        return Err(unsupported(
            id,
            "OffsetPath BasicShape/Path or SVG resource owner",
        ));
    }
    if coord.is_none() {
        coord = CoordBox(s);
    }
    if ray.is_none() && coord.is_none() {
        return Ok(None);
    }
    let mut list = Vec::new();
    let has_path = ray.is_some();
    list.extend(ray);
    if !has_path
        || coord.as_ref().is_some_and(
            |c| !matches!(c.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==kBorderBox),
        )
    {
        list.extend(coord);
    }
    Ok(Some(values::list(list, values::ListSeparator::Space)))
}
fn Rotate<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    let mut angle = transform_parser::ConsumeAngle(id, s, mode)?;
    let keyword = if matches!(s.Peek().Id(), kAuto | kReverse) {
        Some(values::identifier(s.ConsumeIncludingWhitespace().Id()))
    } else {
        None
    };
    if angle.is_none() && keyword.is_none() {
        return Ok(None);
    }
    if angle.is_none() {
        angle = transform_parser::ConsumeAngle(id, s, mode)?;
    }
    Ok(Some(values::list(
        keyword.into_iter().chain(angle).collect(),
        values::ListSeparator::Space,
    )))
}
fn Distance<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if IsMathFunction(s) {
        let v = ConsumeAnimationNumericMath(
            id,
            s,
            crate::css_math_expression_node::CalculationResultCategory::Length,
            crate::css_math_function_value::ValueRange::All,
        )?;
        if v.is_some() {
            return Ok(v);
        }
        for category in [
            crate::css_math_expression_node::CalculationResultCategory::Percent,
            crate::css_math_expression_node::CalculationResultCategory::LengthFunction,
        ] {
            let v = ConsumeAnimationNumericMath(
                id,
                s,
                category,
                crate::css_math_function_value::ValueRange::All,
            )?;
            if v.is_some() {
                return Ok(v);
            }
        }
        return Ok(None);
    }
    let token = s.Peek();
    if !(matches!(token.GetType(), kDimensionToken | kPercentageToken)
        || token.GetType() == kNumberToken && token.NumericValue() == 0.0)
    {
        return Ok(None);
    }
    if token.GetType() == kDimensionToken
        && !crate::css_numeric_literal_value::IsLength(token.GetUnitType())
    {
        return Ok(None);
    }
    ConsumeLiteral(
        id,
        s,
        mode,
        Grammar::Length {
            percent: true,
            nonnegative: false,
            quirks: false,
            keywords: &[],
        },
    )
    .map(Some)
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    match id {
        P::kOffsetPath => Path(id, s, mode)?.ok_or_else(|| invalid(id)),
        P::kOffsetRotate => Rotate(id, s, mode)?.ok_or_else(|| invalid(id)),
        P::kOffsetDistance => Distance(id, s, mode)?.ok_or_else(|| invalid(id)),
        _ => Err(invalid(id)),
    }
}
fn OptionalPosition<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    s.EnsureLookAhead();
    let save = s.Save();
    match ConsumeLonghand(id, s, mode) {
        Ok(value) => Ok(Some(value)),
        Err(e) if e.kind == PropertyParseErrorKind::Unsupported => Err(e),
        Err(_) => {
            s.EnsureLookAhead();
            s.Restore(save);
            Ok(None)
        }
    }
}
pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    s: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    let position = OptionalPosition(P::kOffsetPosition, s, mode)?;
    let path = Path(P::kOffsetPath, s, mode)?;
    let mut distance = None;
    let mut rotate = None;
    if path.is_some() {
        distance = Distance(P::kOffsetDistance, s, mode)?;
        rotate = Rotate(P::kOffsetRotate, s, mode)?;
        if rotate.is_some() && distance.is_none() {
            distance = Distance(P::kOffsetDistance, s, mode)?;
        }
    }
    let anchor = if s.Peek().GetType() == kDelimiterToken && s.Peek().Delimiter() == b'/' as u16 {
        s.ConsumeIncludingWhitespace();
        Some(ConsumeLonghand(P::kOffsetAnchor, s, mode)?)
    } else {
        None
    };
    if position.is_none() && path.is_none() {
        return Err(invalid(P::kOffset));
    }
    for (id, value, initial) in [
        (P::kOffsetPosition, position, values::identifier(kNormal)),
        (P::kOffsetPath, path, values::identifier(kNone)),
        (
            P::kOffsetDistance,
            distance,
            values::numeric(0.0, UnitType::kPixels),
        ),
        (P::kOffsetRotate, rotate, values::identifier(kAuto)),
        (P::kOffsetAnchor, anchor, values::identifier(kAuto)),
    ] {
        out.push(make_expanded(
            id,
            P::kOffset,
            value.unwrap_or(initial),
            false,
        ));
    }
    Ok(())
}
