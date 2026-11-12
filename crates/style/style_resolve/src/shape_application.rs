//! Source basic_shape_functions.cc:92-242,736-745; shape_functions.h:13-41.
#![allow(non_snake_case)]
use super::effects_application::{LengthValue, Number};
use super::*;
use foundation::{svg_path_byte_stream::SvgPathCommand as C, Length, LengthPoint, LengthSize};
use layoutng_style::style::style_shape::{
    ArcSegment, ControlPoint, ControlPointOrigin, CurveSegment, Segment, StyleShape,
};
use std::rc::Rc;
use style_parser::production_shape_value::{
    CSSShapeCommand, CSSShapeControlPoint, CSSShapeEndPoint, CSSShapeValue,
};
type Result<T> = std::result::Result<T, LonghandApplicationError>;
fn Axis(
    id: CSSPropertyID,
    v: &Value,
    horizontal: bool,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result<Length> {
    if let CSSValuePayload::kIdentifierClass(word) = v.Payload() {
        return match word.0 {
            CSSValueID::kCenter => Ok(Length::Percent(50)),
            CSSValueID::kLeft | CSSValueID::kXStart if horizontal => Ok(Length::Percent(0)),
            CSSValueID::kRight | CSSValueID::kXEnd if horizontal => Ok(Length::Percent(100)),
            CSSValueID::kTop | CSSValueID::kYStart if !horizontal => Ok(Length::Percent(0)),
            CSSValueID::kBottom | CSSValueID::kYEnd if !horizontal => Ok(Length::Percent(100)),
            _ => Err(LonghandApplicationError::InvalidValue(id)),
        };
    }
    if let CSSValuePayload::kValuePairClass(pair) = v.Payload() {
        let length = LengthValue(id, &pair.second, b, root, media)?;
        return match pair.first.Payload() {
            CSSValuePayload::kIdentifierClass(word)
                if (horizontal && word.0 == CSSValueID::kLeft)
                    || (!horizontal && word.0 == CSSValueID::kTop) =>
            {
                Ok(length)
            }
            CSSValuePayload::kIdentifierClass(word)
                if (horizontal && word.0 == CSSValueID::kRight)
                    || (!horizontal && word.0 == CSSValueID::kBottom) =>
            {
                Ok(length.SubtractFromOneHundredPercent())
            }
            _ => Err(LonghandApplicationError::InvalidValue(id)),
        };
    }
    LengthValue(id, v, b, root, media)
}
fn Point(
    id: CSSPropertyID,
    p: &[Rc<Value>; 2],
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result<LengthPoint> {
    Ok(LengthPoint::new(
        &Axis(id, &p[0], true, b, root, media)?,
        &Axis(id, &p[1], false, b, root, media)?,
    ))
}
fn EndPoint(
    id: CSSPropertyID,
    command: &CSSShapeCommand,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result<LengthPoint> {
    match &command.end_point {
        Some(CSSShapeEndPoint::Pair(p)) => Point(id, p, b, root, media),
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}
fn Control(
    id: CSSPropertyID,
    p: &CSSShapeControlPoint,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result<ControlPoint> {
    let origin = match p.origin {
        CSSValueID::kOrigin => ControlPointOrigin::ReferenceBox,
        CSSValueID::kStart => ControlPointOrigin::SegmentStart,
        CSSValueID::kEnd => ControlPointOrigin::SegmentEnd,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    };
    Ok(ControlPoint {
        origin,
        point: Point(id, &p.point, b, root, media)?,
    })
}
fn Curve<const N: usize>(
    id: CSSPropertyID,
    command: &CSSShapeCommand,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result<CurveSegment<N>> {
    let points = command
        .controls
        .iter()
        .map(|p| Control(id, p, b, root, media))
        .collect::<Result<Vec<_>>>()?;
    Ok(CurveSegment {
        target_point: EndPoint(id, command, b, root, media)?,
        control_points: points
            .try_into()
            .map_err(|_| LonghandApplicationError::InvalidValue(id))?,
    })
}
pub(super) fn Shape(
    id: CSSPropertyID,
    value: &CSSShapeValue,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result<StyleShape> {
    let mut segments = Vec::with_capacity(value.commands.len());
    for command in &value.commands {
        let segment = match command.command {
            C::Close => Segment::Close,
            C::MoveAbs => Segment::MoveTo(EndPoint(id, command, b, root, media)?),
            C::MoveRel => Segment::MoveBy(EndPoint(id, command, b, root, media)?),
            C::LineAbs => Segment::LineTo(EndPoint(id, command, b, root, media)?),
            C::LineRel => Segment::LineBy(EndPoint(id, command, b, root, media)?),
            C::SmoothQuadAbs => {
                Segment::SmoothQuadraticCurveTo(EndPoint(id, command, b, root, media)?)
            }
            C::SmoothQuadRel => {
                Segment::SmoothQuadraticCurveBy(EndPoint(id, command, b, root, media)?)
            }
            C::HLineAbs | C::HLineRel | C::VLineAbs | C::VLineRel => {
                let Some(CSSShapeEndPoint::Axis(v)) = &command.end_point else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                let horizontal = matches!(command.command, C::HLineAbs | C::HLineRel);
                let length = Axis(id, v, horizontal, b, root, media)?;
                match command.command {
                    C::HLineAbs => Segment::HLineTo(length),
                    C::HLineRel => Segment::HLineBy(length),
                    C::VLineAbs => Segment::VLineTo(length),
                    _ => Segment::VLineBy(length),
                }
            }
            C::CubicAbs => Segment::CubicCurveTo(Curve(id, command, b, root, media)?),
            C::CubicRel => Segment::CubicCurveBy(Curve(id, command, b, root, media)?),
            C::QuadAbs => Segment::QuadraticCurveTo(Curve(id, command, b, root, media)?),
            C::QuadRel => Segment::QuadraticCurveBy(Curve(id, command, b, root, media)?),
            C::SmoothCubicAbs => Segment::SmoothCubicCurveTo(Curve(id, command, b, root, media)?),
            C::SmoothCubicRel => Segment::SmoothCubicCurveBy(Curve(id, command, b, root, media)?),
            C::ArcAbs | C::ArcRel => {
                let arc = command
                    .arc
                    .as_ref()
                    .ok_or(LonghandApplicationError::InvalidValue(id))?;
                let (angle, unit) = Number(id, &arc.angle, b, root, media)?;
                let angle = match unit {
                    UnitType::kDegrees => angle,
                    UnitType::kRadians => angle.to_degrees(),
                    UnitType::kGradians => angle * 0.9,
                    UnitType::kTurns => angle * 360.,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                };
                let x = LengthValue(id, &arc.radius[0], b, root, media)?;
                let y = LengthValue(id, &arc.radius[1], b, root, media)?;
                let zero = Length::Fixed(0);
                let data = ArcSegment {
                    target_point: EndPoint(id, command, b, root, media)?,
                    angle,
                    radius: if arc.direction_agnostic {
                        LengthSize::new(&zero, &zero)
                    } else {
                        LengthSize::new(&x, &y)
                    },
                    direction_agnostic_radius: if arc.direction_agnostic { x } else { zero },
                    large: arc.large,
                    sweep: arc.sweep,
                };
                if command.command == C::ArcAbs {
                    Segment::ArcTo(data)
                } else {
                    Segment::ArcBy(data)
                }
            }
        };
        segments.push(segment);
    }
    Ok(StyleShape::new(
        value.wind_rule,
        Point(id, &value.origin, b, root, media)?,
        segments,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use style_parser::{
        css_parser_mode::CSSParserMode, production_property_parser::ParseDeclarationList,
    };
    #[test]
    fn authored_shape_installs_the_canonical_computed_owner() {
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        let css = foundation::String::from("clip-path:shape(evenodd from left top, curve by 100% 100% with 10px 20% from end / 20% 10px from origin, arc to 0 0 of 25% cw large rotate 45deg, close);border-shape:shape(from 0 0,line to 100% 0,line to 50% 100%,close) border-box");
        let parsed = ParseDeclarationList(&css, CSSParserMode::kHTMLStandardMode);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        for property in parsed.properties {
            super::super::Apply(
                property.PropertyID(),
                &mut builder,
                None,
                property.Value(),
                16.,
                &MediaValuesCachedData::default(),
            )
            .unwrap();
        }
        let operation = unsafe { builder.ClipPath().unwrap().as_ref().unwrap() };
        let operation = unsafe {
            &*(operation as *const dyn layoutng_style::style::clip_path_operation::ClipPathOperation
                as *const layoutng_style::style::shape_clip_path_operation::ShapeClipPathOperation)
        };
        assert_eq!(
            operation.GetBasicShape().GetType(),
            layoutng_style::style::basic_shapes::ShapeType::kStyleShapeType
        );
        let shape = unsafe {
            &*(operation.GetBasicShape()
                as *const dyn layoutng_style::style::basic_shapes::BasicShape
                as *const StyleShape)
        };
        assert_eq!(shape.GetWindRule(), foundation::WindRule::RULE_EVENODD);
        let Segment::CubicCurveBy(curve) = &shape.Segments()[0] else {
            panic!("computed cubic")
        };
        assert!(curve.target_point.X().IsPercent());
        assert_eq!(
            curve.control_points[0].origin,
            ControlPointOrigin::SegmentEnd
        );
        let Segment::ArcTo(arc) = &shape.Segments()[1] else {
            panic!("computed arc")
        };
        assert!(arc.direction_agnostic_radius.IsPercent());
        assert_eq!(arc.angle, 45.);
        assert!(arc.sweep && arc.large);
        let border = unsafe { builder.BorderShape().Get().as_ref().unwrap() };
        assert_eq!(
            border.OuterShape().GetType(),
            layoutng_style::style::basic_shapes::ShapeType::kStyleShapeType
        );
    }
}
