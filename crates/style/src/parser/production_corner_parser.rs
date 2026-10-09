// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium corner shape consumers and stable/internal shorthand expansion.
#![allow(non_snake_case)]
use super::*;
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kCornerTopLeftShape
            | kCornerTopRightShape
            | kCornerBottomLeftShape
            | kCornerBottomRightShape
            | kCornerStartStartShape
            | kCornerStartEndShape
            | kCornerEndStartShape
            | kCornerEndEndShape
            | kBorderShape
    )
}
pub(super) fn IsShorthand(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kCornerShape
            | kCornerTopShape
            | kCornerBottomShape
            | kCornerLeftShape
            | kCornerRightShape
            | kCornerBlockStartShape
            | kCornerBlockEndShape
            | kCornerInlineStartShape
            | kCornerInlineEndShape
    ) || !crate::production_corner_features::IsExposed(id)
}
// css_parsing_utils.cc:5784-5821.
fn Shape<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    use CSSValueID::*;
    if matches!(
        s.Peek().Id(),
        kBevel | kNotch | kRound | kScoop | kSquircle | kSquare
    ) {
        return Ok(Some(values::identifier(
            s.ConsumeIncludingWhitespace().Id(),
        )));
    }
    if s.Peek().FunctionId() != Some(kSuperellipse) {
        return Ok(None);
    }
    let mut guard = RestoringBlockGuard::new(s);
    guard.ConsumeWhitespace();
    let param = if matches!(guard.Peek().Id(), kInfinity | kNegativeInfinity) {
        let infinity = if guard.ConsumeIncludingWhitespace().Id() == kInfinity {
            f64::INFINITY
        } else {
            f64::NEG_INFINITY
        };
        values::numeric(infinity, UnitType::kNumber)
    } else {
        ConsumeLiteral(id, &mut guard, mode, Grammar::Number { nonnegative: false })?
    };
    if !guard.AtEnd() {
        return Err(invalid(id));
    }
    guard.Release();
    drop(guard);
    s.ConsumeWhitespace();
    Ok(Some(Rc::new(Value::new(
        CSSValuePayload::kSuperellipseClass(
            crate::production_corner_value::CSSSuperellipseValue::new(param),
        ),
    ))))
}
// css_parsing_utils.cc:5823-5850. normal resets both fields, and a radius and
// shape are required for every other corner item, in either source order.
fn Corner<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<(Rc<Value>, Rc<Value>), PropertyParseError> {
    if s.Peek().Id() == CSSValueID::kNormal {
        s.ConsumeIncludingWhitespace();
        return Ok((
            Pair(
                values::numeric(0., UnitType::kPixels),
                values::numeric(0., UnitType::kPixels),
                true,
            ),
            values::identifier(CSSValueID::kRound),
        ));
    }
    let mut shape = Shape(id, s, mode)?;
    let radius = ConsumeBorderRadiusCorner(id, s, mode)?;
    if shape.is_none() {
        shape = Shape(id, s, mode)?;
    }
    Ok((radius, shape.ok_or_else(|| invalid(id))?))
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    if id == CSSPropertyID::kBorderShape {
        return BorderShape(id, s, mode);
    }
    Shape(id, s, mode)?.ok_or_else(|| invalid(id))
}
pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    let expanded = ExpandInternal(id, s, mode)?;
    if !crate::production_corner_features::IsExposed(id) {
        return Err(unsupported(
            id,
            "CSSCornersShorthand enabled metadata/Exposure owner",
        ));
    }
    for (property, value) in expanded {
        out.push(make_expanded(property, id, value, false));
    }
    Ok(())
}
// Internal source consumer preserves genuine typed IDs/values before the
// AddProperty metadata stage. The experimental feature has no enabled owner;
// its matching-shorthand/CSSOM 2-bit capacity remains partial, not truncated.
fn ExpandInternal<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Vec<(CSSPropertyID, Rc<Value>)>, PropertyParseError> {
    let longhands = ShorthandFor(id);
    let mut out = Vec::new();
    if crate::production_corner_features::IsExposed(id) {
        let mut shapes = vec![Consume(longhands[0], s, mode)?];
        while shapes.len() < longhands.len() && !at_value_end(s) {
            shapes.push(Consume(longhands[0], s, mode)?);
        }
        let indices = match shapes.len() {
            1 => [0, 0, 0, 0],
            2 => [0, 1, 0, 1],
            3 => [0, 1, 2, 1],
            _ => [0, 1, 2, 3],
        };
        for (i, &property) in longhands.iter().enumerate() {
            out.push((property, shapes[indices[i]].clone()));
        }
        return Ok(out);
    }
    // shorthands_custom.cc:2148-2185,2233-2260,2262-2309.
    // Shared source helpers require 1/2/4 corners, slash separated.
    let count = longhands.len() / 2;
    let mut corners = vec![Corner(id, s, mode)?];
    while corners.len() < count
        && s.Peek().GetType() == kDelimiterToken
        && s.Peek().Delimiter() == b'/' as u16
    {
        s.ConsumeIncludingWhitespace();
        corners.push(Corner(id, s, mode)?);
    }
    let indices = match corners.len() {
        1 => [0, 0, 0, 0],
        2 => [0, 1, 0, 1],
        3 => [0, 1, 2, 1],
        _ => [0, 1, 2, 3],
    };
    for i in 0..count {
        let (radius, shape) = &corners[indices[i]];
        out.push((longhands[i * 2], radius.clone()));
        out.push((longhands[i * 2 + 1], shape.clone()));
    }
    Ok(out)
}

// longhands_custom.cc:1870-1920. Shared polygon is the genuine typed/native
// basic shape currently available; other BasicShape/Path owners stay partial.
fn BasicShapeAndBox<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    use CSSValueID::*;
    let shape = match s.Peek().FunctionId() {
        Some(kPolygon) => effects_parser::Polygon(id, s, mode)?,
        Some(kCircle | kEllipse | kInset | kRect | kXywh | kPath | kShape) => {
            return Err(unsupported(id, "BorderShape BasicShape/Path owner"))
        }
        _ => return Ok(None),
    };
    if matches!(
        s.Peek().Id(),
        kBorderBox
            | kPaddingBox
            | kContentBox
            | kMarginBox
            | kFillBox
            | kStrokeBox
            | kViewBox
            | kHalfBorderBox
    ) {
        let box_value = values::identifier(s.ConsumeIncludingWhitespace().Id());
        return Ok(Some(Pair(shape, box_value, false)));
    }
    Ok(Some(shape))
}
fn BorderShape<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    if s.Peek().Id() == CSSValueID::kNone {
        return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
    }
    let outer = BasicShapeAndBox(id, s, mode)?.ok_or_else(|| invalid(id))?;
    let Some(inner) = BasicShapeAndBox(id, s, mode)? else {
        return Ok(outer);
    };
    if outer.IsValuePair() && inner.IsValuePair() && outer == inner {
        return Ok(outer);
    }
    Ok(values::list(
        vec![outer, inner],
        values::ListSeparator::Space,
    ))
}
