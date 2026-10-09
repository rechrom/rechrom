// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! SVG ApplyInitial/Inherit/Value and StyleBuilderConverter native branches.
#![allow(non_snake_case)]
use super::*;
use foundation::{MakeGarbageCollected, Member};
use layoutng_style::style::{
    computed_style_constants::EPaintOrder,
    svg_dash_array::SVGDashArray,
    svg_paint::{SVGPaint, SVGPaintType},
    unzoomed_length::UnzoomedLength,
};

pub(super) fn IsSVGProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kFill
            | kFillOpacity
            | kFloodOpacity
            | kStopOpacity
            | kStrokeOpacity
            | kFillRule
            | kClipRule
            | kStroke
            | kInternalVisitedFill
            | kInternalVisitedStroke
            | kStrokeWidth
            | kStrokeDashoffset
            | kStrokeDasharray
            | kCx
            | kCy
            | kR
            | kRx
            | kRy
            | kX
            | kY
            | kPathLength
            | kPaintOrder
    )
}

// style_builder_converter.cc:2063-2067,2080-2085. SVG user units resolve as
// pixels with conversion-data zoom; stroke-width uses unzoomed conversion.
pub(super) fn ConvertLength(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<Length, LonghandApplicationError> {
    use crate::css_math_expression_node::{CSSMathLengthResolver, CalculationResultCategory as C};
    let unzoomed = id == CSSPropertyID::kStrokeWidth;
    let zoom = if unzoomed { 1.0 } else { b.EffectiveZoom() };
    let font = b.GetFontDescription().SpecifiedSize();
    let mut resolver = |value, unit| {
        let pixels = super::Pixels(
            id,
            value,
            if unit == UnitType::kUserUnits {
                UnitType::kPixels
            } else {
                unit
            },
            font,
            root,
            media,
        )
        .map_err(|_| crate::css_math_expression_node::MathError::MissingLengthContext)?;
        Ok(pixels * zoom as f64)
    };
    if let CSSValuePayload::kMathFunctionClass(math) = v.Payload() {
        // ConsumeLengthOrPercent accepts number-valued calculations in SVG mode.
        if math.Category() == C::Number {
            let number = math
                .ComputeValue(&mut resolver, None)
                .map_err(|_| LonghandApplicationError::Unsupported(id))?;
            return Ok(Length::Fixed(
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(
                    number * zoom as f64,
                ),
            ));
        }
        return math
            .ConvertToLength(&mut resolver)
            .map_err(|_| LonghandApplicationError::Unsupported(id));
    }
    if let CSSValuePayload::kIdentifierClass(keyword) = v.Payload() {
        return match (id, keyword.0) {
            (CSSPropertyID::kRx | CSSPropertyID::kRy, CSSValueID::kAuto) => {
                Ok(Length::Auto().clone())
            }
            (CSSPropertyID::kPathLength, CSSValueID::kNone) => Ok(Length::None()),
            _ => Err(LonghandApplicationError::InvalidValue(id)),
        };
    }
    let CSSValuePayload::kNumericLiteralClass(number) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if number.GetType() == UnitType::kPercentage {
        return Ok(Length::Percent(number.DoubleValue()));
    }
    let pixels = resolver
        .ComputeLength(
            number.DoubleValue(),
            if number.GetType() == UnitType::kUserUnits {
                UnitType::kPixels
            } else {
                number.GetType()
            },
        )
        .map_err(|_| LonghandApplicationError::Unsupported(id))?;
    Ok(Length::Fixed(
        crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(pixels),
    ))
}

// style_builder_converter.cc:3098-3147. Resource references retain the URI and
// fallback in CSSValue; a real document StyleSVGResource adapter must bind them.
fn ConvertPaint(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<SVGPaint, LonghandApplicationError> {
    let mut paint = SVGPaint::default();
    match v.Payload() {
        CSSValuePayload::kURIClass(_) | CSSValuePayload::kValueListClass(_) => {
            return Err(LonghandApplicationError::Unsupported(id))
        }
        CSSValuePayload::kIdentifierClass(keyword) => match keyword.0 {
            CSSValueID::kNone => paint.paint_type = SVGPaintType::kNone,
            CSSValueID::kContextFill => paint.paint_type = SVGPaintType::kContextFill,
            CSSValueID::kContextStroke => paint.paint_type = SVGPaintType::kContextStroke,
            CSSValueID::kCurrentcolor => {
                paint.color = StyleColor::CurrentColor();
                paint.paint_type = SVGPaintType::kColor;
            }
            color => {
                paint.color = StyleColor::from_color(
                    crate::production_css_value::NamedColor(color)
                        .ok_or(LonghandApplicationError::Unsupported(id))?,
                );
                paint.paint_type = SVGPaintType::kColor;
            }
        },
        CSSValuePayload::kColorClass(color) => paint = SVGPaint::from_color(color.0),
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    }
    Ok(paint)
}

fn ConvertPaintOrder(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<EPaintOrder, LonghandApplicationError> {
    // style_builder_converter.cc:2523-2545, after parser canonicalization.
    use EPaintOrder::*;
    if let CSSValuePayload::kIdentifierClass(keyword) = v.Payload() {
        return if keyword.0 == CSSValueID::kNormal {
            Ok(kPaintOrderNormal)
        } else {
            Err(LonghandApplicationError::InvalidValue(id))
        };
    }
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != crate::production_css_value::ListSeparator::Space
        || !(1..=2).contains(&list.values.len())
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let CSSValuePayload::kIdentifierClass(first) = list.values[0].Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    let second = list.values.get(1).map(|v| {
        if let CSSValuePayload::kIdentifierClass(k) = v.Payload() {
            k.0
        } else {
            CSSValueID::kInvalid
        }
    });
    Ok(match (first.0, second) {
        (CSSValueID::kFill, None) => kPaintOrderFillStrokeMarkers,
        (CSSValueID::kFill, Some(CSSValueID::kMarkers)) => kPaintOrderFillMarkersStroke,
        (CSSValueID::kStroke, None) => kPaintOrderStrokeFillMarkers,
        (CSSValueID::kStroke, Some(CSSValueID::kMarkers)) => kPaintOrderStrokeMarkersFill,
        (CSSValueID::kMarkers, None) => kPaintOrderMarkersFillStroke,
        (CSSValueID::kMarkers, Some(CSSValueID::kStroke)) => kPaintOrderMarkersStrokeFill,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
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
    if inherit
        && parent.is_some()
        && !matches!(
            id,
            kFill
                | kStroke
                | kInternalVisitedFill
                | kInternalVisitedStroke
                | kPaintOrder
                | kFillOpacity
                | kFloodOpacity
                | kStopOpacity
                | kStrokeOpacity
                | kFillRule
                | kClipRule
        )
        && b.EffectiveZoom() != parent.unwrap().EffectiveZoom()
    {
        // Generated ApplyParentValueIfZoomChanged requires the document's
        // standardized-browser-zoom policy and parent conversion-data service.
        return Err(LonghandApplicationError::Unsupported(id));
    }
    if inherit && parent.is_some() && v.IsInheritedValue() && !CSSProperty::Get(id).IsInherited() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    if matches!(
        id,
        kFillOpacity | kFloodOpacity | kStopOpacity | kStrokeOpacity | kFillRule | kClipRule
    ) {
        if initial {
            return ApplyInitial(id, b);
        }
        if inherit {
            return ApplyInherit(id, b, parent.unwrap());
        }
        if matches!(id, kFillRule | kClipRule) {
            // generated longhands.cc:5976,7910, css_value_id_mappings_generated.h.
            let rule = match Identifier(id, v)? {
                CSSValueID::kNonzero => foundation::WindRule::RULE_NONZERO,
                CSSValueID::kEvenodd => foundation::WindRule::RULE_EVENODD,
                _ => return Err(LonghandApplicationError::InvalidValue(id)),
            };
            if id == kFillRule {
                b.SetFillRule(rule);
            } else {
                b.SetClipRule(rule);
            }
        } else {
            // ConvertAlpha:2260-2263 uses number/percentage then clamps to [0,1].
            let percent = matches!(v.Payload(), CSSValuePayload::kNumericLiteralClass(n) if n.GetType() == UnitType::kPercentage)
                || matches!(v.Payload(), CSSValuePayload::kMathFunctionClass(m) if m.Category() == crate::css_math_expression_node::CalculationResultCategory::Percent);
            let alpha = typography_application::Scalar(id, b, v, root, media, percent)?
                / if percent { 100. } else { 1. };
            let alpha = alpha.max(0.).min(1.) as f32;
            match id {
                kFillOpacity => b.SetFillOpacity(alpha),
                kFloodOpacity => b.SetFloodOpacity(alpha),
                kStopOpacity => b.SetStopOpacity(alpha),
                kStrokeOpacity => b.SetStrokeOpacity(alpha),
                _ => unreachable!(),
            }
        }
    } else if matches!(
        id,
        kFill | kStroke | kInternalVisitedFill | kInternalVisitedStroke
    ) {
        // generated longhands.cc:1752-1762,1826-1836 visited Apply* inherits
        // parent ordinary paint. ConvertSVGPaint ignores for_visited_link,
        // so both slots share this converter without color reinterpretation.
        let paint = if initial {
            if matches!(id, kFill | kInternalVisitedFill) {
                ComputedStyleInitialValues::InitialFillPaint()
            } else {
                ComputedStyleInitialValues::InitialStrokePaint()
            }
        } else if inherit {
            if matches!(id, kFill | kInternalVisitedFill) {
                parent.unwrap().FillPaint().clone()
            } else {
                parent.unwrap().StrokePaint().clone()
            }
        } else {
            ConvertPaint(id, v)?
        };
        match id {
            kFill => b.SetFillPaintOwned(paint),
            kStroke => b.SetStrokePaintOwned(paint),
            kInternalVisitedFill => b.SetInternalVisitedFillPaintOwned(paint),
            kInternalVisitedStroke => b.SetInternalVisitedStrokePaintOwned(paint),
            _ => unreachable!(),
        }
    } else if id == kPaintOrder {
        let order = if initial {
            ComputedStyleInitialValues::InitialPaintOrder()
        } else if inherit {
            parent.unwrap().PaintOrder()
        } else {
            ConvertPaintOrder(id, v)?
        };
        b.SetPaintOrder(order);
    } else if id == kStrokeDasharray {
        // generated:15756-15769; converter:2794-2807. Convert all entries before
        // changing style, then allocate the existing traced native SVGDashArray.
        let array = if initial {
            Member::default()
        } else if inherit {
            Member::from_ptr(parent.unwrap().StrokeDashArray())
        } else if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kNone)
        {
            Member::default()
        } else {
            let CSSValuePayload::kValueListClass(list) = v.Payload() else {
                return Err(LonghandApplicationError::InvalidValue(id));
            };
            if list.separator != crate::production_css_value::ListSeparator::Comma
                || list.values.is_empty()
            {
                return Err(LonghandApplicationError::InvalidValue(id));
            }
            let lengths = list
                .values
                .iter()
                .map(|value| ConvertLength(id, b, value, root, media))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let array: SVGDashArray = lengths.into_iter().collect();
            Member::from_ptr(MakeGarbageCollected(array))
        };
        b.SetStrokeDashArray(array);
    } else if id == kStrokeWidth {
        let width = if initial {
            ComputedStyleInitialValues::InitialStrokeWidth()
        } else if inherit {
            parent.unwrap().StrokeWidth().clone()
        } else {
            UnzoomedLength::new(&ConvertLength(id, b, v, root, media)?)
        };
        b.SetStrokeWidthOwned(width);
    } else {
        macro_rules! length {
            ($get:ident, $set:ident, $initial:ident) => {{
                let value = if initial {
                    ComputedStyleInitialValues::$initial()
                } else if inherit {
                    parent.unwrap().$get().clone()
                } else {
                    ConvertLength(id, b, v, root, media)?
                };
                b.$set(&value);
            }};
        }
        // generated longhands.cc:7553-7599,13113-13125,13519-13531,
        // 14201-14247,15790-15802,19687-19733.
        match id {
            kCx => length!(Cx, SetCx, InitialCx),
            kCy => length!(Cy, SetCy, InitialCy),
            kR => length!(R, SetR, InitialR),
            kRx => length!(Rx, SetRx, InitialRx),
            kRy => length!(Ry, SetRy, InitialRy),
            kX => length!(X, SetX, InitialX),
            kY => length!(Y, SetY, InitialY),
            kPathLength => length!(PathLength, SetPathLength, InitialPathLength),
            kStrokeDashoffset => length!(
                StrokeDashOffset,
                SetStrokeDashOffset,
                InitialStrokeDashOffset
            ),
            _ => return Err(LonghandApplicationError::Unsupported(id)),
        }
    }
    Ok(())
}
