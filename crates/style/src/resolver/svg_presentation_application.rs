// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native SVG style slots; resource/path owners remain explicit boundaries.
#![allow(non_snake_case)]
use super::*;
use foundation::{LineCap, LineJoin, Member};
use layoutng_style::style::computed_style_constants::EBaselineShiftType;

pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kD | kMarkerStart
            | kMarkerMid
            | kMarkerEnd
            | kBaselineShift
            | kStrokeLinecap
            | kStrokeLinejoin
            | kStrokeMiterlimit
            | kWebkitTextStrokeWidth
            | kWebkitTextStrokeColor
    )
}

pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    use CSSPropertyID::*;
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue()
        || v.IsUnsetValue() && !CSSProperty::Get(id).IsInherited()
        || inherit && parent.is_none();
    // StandardizedBrowserZoom is stable. Its parent re-conversion owner is
    // absent; do not inherit already-zoomed lengths under different zoom.
    if inherit
        && parent.is_some()
        && matches!(id, kBaselineShift | kWebkitTextStrokeWidth)
        && b.EffectiveZoom() != parent.unwrap().EffectiveZoom()
    {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    if inherit && parent.is_some() && v.IsInheritedValue() && !CSSProperty::Get(id).IsInherited() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    match id {
        // generated longhands.cc:7621-7629; converter.cc:3450-3458.
        kD => {
            let path = if initial {
                Member::default()
            } else if inherit {
                Member::from_ptr(parent.unwrap().D())
            } else if Identifier(v) == Some(CSSValueID::kNone) {
                Member::default()
            } else {
                return Err(LonghandApplicationError::Unsupported(id));
            };
            b.SetD(path);
        }
        // generated:10613-10618,10639-10644,10665-10670;
        // custom:6968-6973,6994-6999,7020-7025; converter:270-283.
        kMarkerStart | kMarkerMid | kMarkerEnd => {
            let resource = if initial {
                Member::default()
            } else if inherit {
                Member::from_ptr(match id {
                    kMarkerStart => parent.unwrap().MarkerStartResource(),
                    kMarkerMid => parent.unwrap().MarkerMidResource(),
                    _ => parent.unwrap().MarkerEndResource(),
                })
            } else if Identifier(v) == Some(CSSValueID::kNone) {
                Member::default()
            } else {
                return Err(LonghandApplicationError::Unsupported(id));
            };
            match id {
                kMarkerStart => b.SetMarkerStartResource(resource),
                kMarkerMid => b.SetMarkerMidResource(resource),
                _ => b.SetMarkerEndResource(resource),
            }
        }
        // custom:1273-1317, generated:3777-3779. Initial intentionally
        // resets only the length; the source does not reset BaselineShiftType.
        kBaselineShift => {
            if initial {
                b.SetBaselineShift(&ComputedStyleInitialValues::InitialBaselineShift());
            } else if inherit {
                b.SetBaselineShiftType(parent.unwrap().BaselineShiftType());
                b.SetBaselineShift(parent.unwrap().BaselineShift());
            } else {
                let (kind, length) = match Identifier(v) {
                    Some(CSSValueID::kBaseline) => (EBaselineShiftType::kLength, Length::Fixed(0)),
                    Some(CSSValueID::kSub) => (EBaselineShiftType::kSub, Length::Fixed(0)),
                    Some(CSSValueID::kSuper) => (EBaselineShiftType::kSuper, Length::Fixed(0)),
                    Some(_) => return Err(LonghandApplicationError::InvalidValue(id)),
                    None => (
                        EBaselineShiftType::kLength,
                        svg_application::ConvertLength(id, b, v, root, media)?,
                    ),
                };
                b.SetBaselineShiftType(kind);
                b.SetBaselineShiftOwned(length);
            }
        }
        // generated:15824-15832,15853-15861; identifier mappings:1094-1108,1126-1140.
        kStrokeLinecap => {
            let cap = if initial {
                ComputedStyleInitialValues::InitialCapStyle()
            } else if inherit {
                parent.unwrap().CapStyle()
            } else {
                match Identifier(v) {
                    Some(CSSValueID::kButt) => LineCap::kButtCap,
                    Some(CSSValueID::kRound) => LineCap::kRoundCap,
                    Some(CSSValueID::kSquare) => LineCap::kSquareCap,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetCapStyle(cap);
        }
        kStrokeLinejoin => {
            let join = if initial {
                ComputedStyleInitialValues::InitialJoinStyle()
            } else if inherit {
                parent.unwrap().JoinStyle()
            } else {
                match Identifier(v) {
                    Some(CSSValueID::kMiter) => LineJoin::kMiterJoin,
                    Some(CSSValueID::kRound) => LineJoin::kRoundJoin,
                    Some(CSSValueID::kBevel) => LineJoin::kBevelJoin,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetJoinStyle(join);
        }
        // generated:15882-15890, CSSPrimitiveValue::ConvertTo<float>.
        kStrokeMiterlimit => {
            let number = if initial {
                ComputedStyleInitialValues::InitialStrokeMiterLimit()
            } else if inherit {
                parent.unwrap().StrokeMiterLimit()
            } else {
                Number(id, v)? as f32
            };
            b.SetStrokeMiterLimit(number);
        }
        // generated:19258-19268, converter:2898-3010 shared native StyleColor.
        kWebkitTextStrokeColor => {
            let color = if initial {
                StyleColor::CurrentColor()
            } else if inherit {
                parent.unwrap().TextStrokeColor().clone()
            } else {
                color_ui_application::ConvertStyleColorValue(id, v)?
            };
            b.SetTextStrokeColor(&color);
        }
        // generated:19289-19302; converter.cc:3224-3235 and .h:504-540.
        kWebkitTextStrokeWidth => {
            let width = if initial {
                ComputedStyleInitialValues::InitialTextStrokeWidth()
            } else if inherit {
                parent.unwrap().TextStrokeWidth()
            } else {
                let zoom = b.EffectiveZoom() as f64;
                let font = b.GetFontDescription().SpecifiedSize();
                let pixels = if let Some(keyword) = Identifier(v) {
                    let magnitude = match keyword {
                        CSSValueID::kThin => 1.,
                        CSSValueID::kMedium => 3.,
                        CSSValueID::kThick => 5.,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    };
                    let line_width = magnitude * zoom;
                    let multiplier = if line_width * zoom > 0. && line_width * zoom < 1. {
                        1.
                    } else {
                        line_width
                    };
                    // ConvertLineWidth<float> and multiplier/48 are both float
                    // operations before CSSNumericLiteralValue's double storage.
                    let ems =
                        (multiplier.clamp(f32::MIN as f64, f32::MAX as f64) as f32 / 48.) as f64;
                    super::Pixels(id, ems, UnitType::kEms, font, root, media)? * zoom
                } else {
                    let length = svg_application::ConvertLength(id, b, v, root, media)?;
                    if !length.IsFixed() {
                        return Err(LonghandApplicationError::Unsupported(id));
                    }
                    length.Pixels() as f64
                };
                pixels.clamp(f32::MIN as f64, f32::MAX as f64) as f32
            };
            b.SetTextStrokeWidth(width);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    Ok(())
}

fn Number(id: CSSPropertyID, value: &Value) -> std::result::Result<f64, LonghandApplicationError> {
    match value.Payload() {
        CSSValuePayload::kNumericLiteralClass(number)
            if matches!(number.GetType(), UnitType::kNumber | UnitType::kInteger) =>
        {
            Ok(number.DoubleValue().clamp(f32::MIN as f64, f32::MAX as f64))
        }
        CSSValuePayload::kMathFunctionClass(math) => math
            .ComputeValue(
                &mut |_, _| Err(crate::css_math_expression_node::MathError::MissingLengthContext),
                None,
            )
            .map(|number| number.clamp(f32::MIN as f64, f32::MAX as f64))
            .map_err(|_| LonghandApplicationError::Unsupported(id)),
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}

fn Identifier(value: &Value) -> Option<CSSValueID> {
    if let CSSValuePayload::kIdentifierClass(id) = value.Payload() {
        Some(id.0)
    } else {
        None
    }
}
