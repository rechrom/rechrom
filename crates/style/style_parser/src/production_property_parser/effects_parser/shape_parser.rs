//! Source css_parsing_utils.cc:8465-8860, computed shape() admission.
#![allow(non_snake_case)]
use super::*;
use crate::production_shape_value::{
    CSSShapeArc, CSSShapeCommand, CSSShapeControlPoint, CSSShapeEndPoint, CSSShapeValue,
};
use foundation::svg_path_byte_stream::SvgPathCommand as C;
fn CoordinatePair<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<[Rc<Value>; 2], PropertyParseError> {
    Ok([
        LengthValue(id, s, mode, true, false)?,
        LengthValue(id, s, mode, true, false)?,
    ])
}
fn Position<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<[Rc<Value>; 2], PropertyParseError> {
    let (x, y) = super::super::ConsumePosition(id, s, mode, false, false)?;
    Ok([x, y])
}
fn EndPoint<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<(bool, CSSShapeEndPoint), PropertyParseError> {
    let absolute = match s.ConsumeIncludingWhitespace().Id() {
        kTo => true,
        kBy => false,
        _ => return Err(invalid(id)),
    };
    Ok((
        absolute,
        CSSShapeEndPoint::Pair(if absolute {
            Position(id, s, mode)?
        } else {
            CoordinatePair(id, s, mode)?
        }),
    ))
}
fn Control<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
    absolute: bool,
) -> Result<CSSShapeControlPoint, PropertyParseError> {
    let point = if absolute {
        Position(id, s, mode)?
    } else {
        CoordinatePair(id, s, mode)?
    };
    if absolute && point.iter().any(|v| v.IsIdentifierValue()) {
        return Ok(CSSShapeControlPoint {
            origin: kOrigin,
            point,
        });
    }
    let origin = if s.Peek().Id() == kFrom {
        s.ConsumeIncludingWhitespace();
        match s.ConsumeIncludingWhitespace().Id() {
            v @ (kStart | kEnd | kOrigin) => v,
            _ => return Err(invalid(id)),
        }
    } else if absolute {
        kOrigin
    } else {
        kStart
    };
    Ok(CSSShapeControlPoint { origin, point })
}
pub(crate) fn ShapeFunction<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let mut s = RestoringBlockGuard::new(stream);
    s.ConsumeWhitespace();
    let wind_rule = match s.Peek().Id() {
        kEvenodd => {
            s.ConsumeIncludingWhitespace();
            foundation::WindRule::RULE_EVENODD
        }
        kNonzero => {
            s.ConsumeIncludingWhitespace();
            foundation::WindRule::RULE_NONZERO
        }
        _ => foundation::WindRule::RULE_NONZERO,
    };
    if s.ConsumeIncludingWhitespace().Id() != kFrom {
        return Err(invalid(id));
    }
    let origin = Position(id, &mut s, mode)?;
    if s.Peek().GetType() != kCommaToken {
        return Err(invalid(id));
    }
    s.ConsumeIncludingWhitespace();
    let mut commands = Vec::new();
    while !s.AtEnd() {
        let name = s.ConsumeIncludingWhitespace().Id();
        let command = match name {
            kClose => CSSShapeCommand::new(C::Close, None),
            kMove | kLine | kCurve | kSmooth | kArc => {
                let (absolute, point) = EndPoint(id, &mut s, mode)?;
                let kind = match name {
                    kMove => {
                        if absolute {
                            C::MoveAbs
                        } else {
                            C::MoveRel
                        }
                    }
                    kLine => {
                        if absolute {
                            C::LineAbs
                        } else {
                            C::LineRel
                        }
                    }
                    kCurve => {
                        if absolute {
                            C::QuadAbs
                        } else {
                            C::QuadRel
                        }
                    }
                    kSmooth => {
                        if absolute {
                            C::SmoothQuadAbs
                        } else {
                            C::SmoothQuadRel
                        }
                    }
                    _ => {
                        if absolute {
                            C::ArcAbs
                        } else {
                            C::ArcRel
                        }
                    }
                };
                let mut command = CSSShapeCommand::new(kind, Some(point));
                if name == kCurve || (name == kSmooth && s.Peek().Id() == kWith) {
                    if s.ConsumeIncludingWhitespace().Id() != kWith {
                        return Err(invalid(id));
                    }
                    command.controls.push(Control(id, &mut s, mode, absolute)?);
                    if name == kCurve
                        && s.Peek().GetType() == kDelimiterToken
                        && s.Peek().Delimiter() == b'/' as u16
                    {
                        s.ConsumeIncludingWhitespace();
                        command.controls.push(Control(id, &mut s, mode, absolute)?);
                        command.command = if absolute { C::CubicAbs } else { C::CubicRel };
                    } else if name == kSmooth {
                        command.command = if absolute {
                            C::SmoothCubicAbs
                        } else {
                            C::SmoothCubicRel
                        };
                    }
                }
                if name == kArc {
                    let mut radius = None;
                    let mut angle = None;
                    let mut large = None;
                    let mut sweep = None;
                    while !s.AtEnd() && s.Peek().GetType() != kCommaToken {
                        match s.ConsumeIncludingWhitespace().Id() {
                            kOf if radius.is_none() => {
                                let x = LengthValue(id, &mut s, mode, true, false)?;
                                let y = if CanStartLength(&mut s) {
                                    Some(LengthValue(id, &mut s, mode, true, false)?)
                                } else {
                                    None
                                };
                                radius = Some((y.is_none(), [x.clone(), y.unwrap_or(x)]));
                            }
                            kCw if sweep.is_none() => sweep = Some(true),
                            kCcw if sweep.is_none() => sweep = Some(false),
                            kLarge if large.is_none() => large = Some(true),
                            kSmall if large.is_none() => large = Some(false),
                            kRotate if angle.is_none() => {
                                angle = Some(
                                    super::super::transform_parser::ConsumeAngle(id, &mut s, mode)?
                                        .ok_or_else(|| invalid(id))?,
                                )
                            }
                            _ => return Err(invalid(id)),
                        }
                    }
                    let (direction_agnostic, radius) = radius.ok_or_else(|| invalid(id))?;
                    command.arc = Some(CSSShapeArc {
                        angle: angle.unwrap_or_else(|| values::numeric(0., UnitType::kDegrees)),
                        radius,
                        direction_agnostic,
                        large: large.unwrap_or(false),
                        sweep: sweep.unwrap_or(false),
                    });
                }
                command
            }
            kHline | kVline => {
                let absolute = match s.ConsumeIncludingWhitespace().Id() {
                    kTo => true,
                    kBy => false,
                    _ => return Err(invalid(id)),
                };
                let horizontal = name == kHline;
                let word = s.Peek().Id();
                let axis = if absolute
                    && (matches!(word, kCenter)
                        || (horizontal && matches!(word, kLeft | kRight | kXStart | kXEnd))
                        || (!horizontal && matches!(word, kTop | kBottom | kYStart | kYEnd)))
                {
                    values::identifier(s.ConsumeIncludingWhitespace().Id())
                } else {
                    LengthValue(id, &mut s, mode, true, false)?
                };
                CSSShapeCommand::new(
                    match (horizontal, absolute) {
                        (true, true) => C::HLineAbs,
                        (true, false) => C::HLineRel,
                        (false, true) => C::VLineAbs,
                        (false, false) => C::VLineRel,
                    },
                    Some(CSSShapeEndPoint::Axis(axis)),
                )
            }
            _ => return Err(invalid(id)),
        };
        commands.push(command);
        if !s.AtEnd() {
            if s.Peek().GetType() != kCommaToken {
                return Err(invalid(id));
            }
            s.ConsumeIncludingWhitespace();
        }
    }
    if commands.is_empty() {
        return Err(invalid(id));
    }
    s.Release();
    drop(s);
    stream.ConsumeWhitespace();
    Ok(Rc::new(Value::new(CSSValuePayload::kBasicShapeShapeClass(
        CSSShapeValue {
            wind_rule,
            origin,
            commands,
        },
    ))))
}

#[cfg(test)]
mod tests {
    use crate::{css_parser_mode::CSSParserMode, production_property_parser::ParseProperty};
    use foundation::{CSSPropertyID, String};
    #[test]
    fn authored_shape_commands_and_motion_ray_admission_follow_source_grammar() {
        let shapes = [
            "shape(from left top, line to right bottom, hline by -10px, vline to y-start, close)",
            "shape(evenodd from calc(10px + 5%) 20%, move by 5px 2px, curve by 10% 20% with 5px 0 from end / 0 5px from origin, smooth to right bottom with 20% 40%, smooth by 1px 2px, close)",
            "shape(from 0 0, arc to 100% 100% of 10% 20px cw large rotate 45deg, arc by 10px -2px small ccw of 20%, close)",
        ];
        for id in [CSSPropertyID::kClipPath, CSSPropertyID::kBorderShape] {
            for shape in shapes {
                let parsed = ParseProperty(
                    id,
                    &String::from(shape),
                    false,
                    CSSParserMode::kHTMLStandardMode,
                );
                assert!(parsed.is_ok(), "{id:?}: {shape}: {:?}", parsed.err());
            }
            for invalid in [
                "ray(45deg)",
                "shape(from 0 0)",
                "shape(from 0 0, curve by 1px 2px with 3px)",
                "shape(from 0 0, arc to 1px 2px cw)",
                "shape(from 0 0, arc to 1px 2px of 5px cw ccw)",
            ] {
                assert!(
                    ParseProperty(
                        id,
                        &String::from(invalid),
                        false,
                        CSSParserMode::kHTMLStandardMode
                    )
                    .is_err(),
                    "{invalid}"
                );
            }
        }
    }
}
