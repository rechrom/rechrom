// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native corner Superellipse and StyleBorderShape owners.
#![allow(non_snake_case)]
use super::*;
use foundation::{MakeGarbageCollected, Member};
use layoutng_style::style::{basic_shapes::BasicShape, computed_style_constants::GeometryBox, style_border_shape::StyleBorderShape, superellipse::Superellipse};

pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(id, kCornerTopLeftShape | kCornerTopRightShape | kCornerBottomLeftShape | kCornerBottomRightShape
        | kCornerStartStartShape | kCornerStartEndShape | kCornerEndStartShape | kCornerEndEndShape | kBorderShape)
}
// converter.cc:2005-2045. Literal and known math values intentionally retain
// infinity/NaN; ComputeNumber's finite clamp is only used for unknown values.
fn Shape(id: CSSPropertyID, v: &Value, b: &ComputedStyleBuilder, root: f32, media: &MediaValuesCachedData) -> std::result::Result<Superellipse, LonghandApplicationError> {
    if let CSSValuePayload::kIdentifierClass(k) = v.Payload() {
        return match k.0 {
            CSSValueID::kBevel => Ok(Superellipse::Bevel()), CSSValueID::kNotch => Ok(Superellipse::Notch()),
            CSSValueID::kRound => Ok(Superellipse::Round()), CSSValueID::kScoop => Ok(Superellipse::Scoop()),
            CSSValueID::kSquircle => Ok(Superellipse::Squircle()), CSSValueID::kSquare => Ok(Superellipse::Square()),
            _ => Err(LonghandApplicationError::InvalidValue(id)),
        };
    }
    let CSSValuePayload::kSuperellipseClass(value) = v.Payload() else { return Err(LonghandApplicationError::InvalidValue(id)); };
    let n = match value.Param().Payload() {
        CSSValuePayload::kNumericLiteralClass(n) if n.IsNumber() => n.DoubleValue(),
        CSSValuePayload::kMathFunctionClass(math) if math.Category() == crate::css_math_expression_node::CalculationResultCategory::Number => {
            if let Some(known) = math.expression.GetValueIfKnown() { known } else {
                let value = math.ComputeValue(&mut MathLengthResolver(id, b.GetFontDescription().ComputedSize(), root, b.EffectiveZoom(), media), None)
                    .map_err(|_| LonghandApplicationError::Unsupported(id))?;
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampDouble(value)
            }
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    };
    Ok(Superellipse::new(n))
}

// converter.cc:297-340. Preserve half-border-box for a single shape and
// border-box/padding-box for two bare shapes, even when the shapes are equal.
fn BasicShapeAndBox(id: CSSPropertyID, v: &Value, default_box: GeometryBox, b: &ComputedStyleBuilder, root: f32, media: &MediaValuesCachedData) -> std::result::Result<(*mut dyn BasicShape, GeometryBox), LonghandApplicationError> {
    let (shape, geometry) = if let CSSValuePayload::kValuePairClass(pair) = v.Payload() {
        let geometry = if Identifier(id, &pair.second)? == CSSValueID::kHalfBorderBox { GeometryBox::kHalfBorderBox } else { effects_application::Geometry(id, &pair.second)? };
        (pair.first.as_ref(), geometry)
    } else { (v, default_box) };
    Ok((effects_application::Shape(id, shape, b, root, media)?, geometry))
}
fn BorderShape(id: CSSPropertyID, v: &Value, b: &ComputedStyleBuilder, root: f32, media: &MediaValuesCachedData) -> std::result::Result<Member<StyleBorderShape>, LonghandApplicationError> {
    if let CSSValuePayload::kIdentifierClass(k) = v.Payload() {
        return if k.0 == CSSValueID::kNone { Ok(Member::default()) } else { Err(LonghandApplicationError::InvalidValue(id)) };
    }
    let (outer, inner, outer_box, inner_box) = if let CSSValuePayload::kValueListClass(list) = v.Payload() {
        if list.separator != crate::production_css_value::ListSeparator::Space || list.values.len() != 2 { return Err(LonghandApplicationError::InvalidValue(id)); }
        let (outer, outer_box) = BasicShapeAndBox(id, &list.values[0], GeometryBox::kBorderBox, b, root, media)?;
        let (inner, inner_box) = BasicShapeAndBox(id, &list.values[1], GeometryBox::kPaddingBox, b, root, media)?;
        (outer, Some(inner), outer_box, inner_box)
    } else {
        let (outer, outer_box) = BasicShapeAndBox(id, v, GeometryBox::kHalfBorderBox, b, root, media)?;
        (outer, None, outer_box, outer_box)
    };
    // All pointers come from the genuine shared GC BasicShape converter.
    Ok(Member::from_ptr(MakeGarbageCollected(unsafe { StyleBorderShape::new(outer, inner, outer_box, inner_box) })))
}

pub(super) fn Apply(id: CSSPropertyID, b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, v: &Value, root: f32, media: &MediaValuesCachedData) -> Result {
    let id = ResolvePhysical(id, b.GetWritingDirection());
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue() || v.IsUnsetValue() && !inherit || inherit && parent.is_none();
    let inherited = parent.filter(|_| inherit && !initial);
    if id == CSSPropertyID::kBorderShape && inherited.is_some_and(|p| p.EffectiveZoom() != b.EffectiveZoom()) { return Err(LonghandApplicationError::Unsupported(id)); }
    if v.IsInheritedValue() && inherited.is_some() && !CSSProperty::Get(id).IsInherited() { b.SetHasExplicitInheritance(); inherited.unwrap().SetChildHasExplicitInheritance(); }
    if id == CSSPropertyID::kBorderShape {
        let shape = if initial { Member::default() } else if let Some(p) = inherited { p.BorderShape().clone() } else { BorderShape(id, v, b, root, media)? };
        b.SetBorderShapeOwned(shape);
        return Ok(());
    }
    let shape = if initial { Superellipse::Round() } else if let Some(p) = inherited {
        *match id { CSSPropertyID::kCornerTopLeftShape => p.CornerTopLeftShape(), CSSPropertyID::kCornerTopRightShape => p.CornerTopRightShape(), CSSPropertyID::kCornerBottomLeftShape => p.CornerBottomLeftShape(), CSSPropertyID::kCornerBottomRightShape => p.CornerBottomRightShape(), _ => return Err(LonghandApplicationError::Unsupported(id)) }
    } else { Shape(id, v, b, root, media)? };
    match id {
        CSSPropertyID::kCornerTopLeftShape => b.SetCornerTopLeftShape(&shape), CSSPropertyID::kCornerTopRightShape => b.SetCornerTopRightShape(&shape),
        CSSPropertyID::kCornerBottomLeftShape => b.SetCornerBottomLeftShape(&shape), CSSPropertyID::kCornerBottomRightShape => b.SetCornerBottomRightShape(&shape),
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    Ok(())
}
