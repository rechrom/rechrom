// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native motion path Apply branches; unavailable vector Path/resource owner
//! stays typed Unsupported. Rays use StyleRay's independent native geometry.
#![allow(non_snake_case)]
use super::*;
use crate::css_math_expression_node::CalculationResultCategory as C;
use foundation::{LengthPoint, MakeGarbageCollected, Member};
use layoutng_style::style::{
    basic_shapes::BasicShape,
    computed_style_constants::{CoordBox, OffsetRotationType},
    offset_path_operation::{
        CoordBoxOffsetPathOperation, OffsetPathOperation, ShapeOffsetPathOperation,
    },
    style_offset_rotation::StyleOffsetRotation,
    style_ray::{RaySize, StyleRay},
};
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kOffsetPath | CSSPropertyID::kOffsetRotate | CSSPropertyID::kOffsetDistance
    )
}
fn Angle(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<f32, LonghandApplicationError> {
    super::transform_application::Scalar(id, b, v, C::Angle, root, media)
        .map(|n| n.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32)
}
fn Box(id: CSSPropertyID, v: &Value) -> std::result::Result<CoordBox, LonghandApplicationError> {
    use CSSValueID::*;
    Ok(match Identifier(id, v)? {
        kBorderBox => CoordBox::kBorderBox,
        kPaddingBox => CoordBox::kPaddingBox,
        kContentBox => CoordBox::kContentBox,
        kFillBox => CoordBox::kFillBox,
        kStrokeBox => CoordBox::kStrokeBox,
        kViewBox => CoordBox::kViewBox,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
// style_builder_converter.cc:2441-2471. Parser canonicalizes keyword before
// angle; keep source per-item float additions (reverse adds 180, auto mode).
fn Rotate(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<StyleOffsetRotation, LonghandApplicationError> {
    if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kAuto) {
        return Ok(StyleOffsetRotation::new(0.0, OffsetRotationType::kAuto));
    }
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != crate::production_css_value::ListSeparator::Space
        || !(1..=2).contains(&list.values.len())
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let mut rotation = StyleOffsetRotation::new(0.0, OffsetRotationType::kFixed);
    for v in &list.values {
        match v.Payload() {
            CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kAuto => {
                rotation.r#type = OffsetRotationType::kAuto
            }
            CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kReverse => {
                rotation.r#type = OffsetRotationType::kAuto;
                rotation.angle = (rotation.angle as f64 + 180.0)
                    .clamp(-(f32::MAX as f64), f32::MAX as f64)
                    as f32;
            }
            _ => {
                rotation.angle = (rotation.angle as f64
                    + super::transform_application::Scalar(id, b, v, C::Angle, root, media)?)
                .clamp(-(f32::MAX as f64), f32::MAX as f64) as f32
            }
        }
    }
    Ok(rotation)
}
// style_builder_converter.cc:3461-3499 / basic_shape_functions.cc:567-581,
// 723-731. CoordBox and Ray preserve real operation kinds and native owners.
fn Path(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<Member<dyn OffsetPathOperation>, LonghandApplicationError> {
    if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kNone) {
        return Ok(Member::default());
    }
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != crate::production_css_value::ListSeparator::Space
        || !(1..=2).contains(&list.values.len())
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let first = &list.values[0];
    if first.IsIdentifierValue() {
        if list.values.len() != 1 {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
        return Ok(Member::from_ptr(
            MakeGarbageCollected(CoordBoxOffsetPathOperation::new(Box(id, first)?))
                as *mut dyn OffsetPathOperation,
        ));
    }
    let coord = if list.values.len() == 2 {
        Box(id, &list.values[1])?
    } else {
        CoordBox::kBorderBox
    };
    let CSSValuePayload::kRayClass(ray) = first.Payload() else {
        return Err(LonghandApplicationError::Unsupported(id));
    };
    let size = match Identifier(id, &ray.size)? {
        CSSValueID::kClosestSide => RaySize::kClosestSide,
        CSSValueID::kClosestCorner => RaySize::kClosestCorner,
        CSSValueID::kFarthestSide => RaySize::kFarthestSide,
        CSSValueID::kFarthestCorner => RaySize::kFarthestCorner,
        CSSValueID::kSides => RaySize::kSides,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    };
    let center = if let Some((x, y)) = &ray.center {
        LengthPoint::new(
            &crate::resolver::position_repeat_application::ConvertPositionLength(
                id, x, true, b, root, media,
            )?,
            &crate::resolver::position_repeat_application::ConvertPositionLength(
                id, y, false, b, root, media,
            )?,
        )
    } else {
        LengthPoint::new(&Length::Percent(50), &Length::Percent(50))
    };
    let shape = MakeGarbageCollected(StyleRay::new(
        Angle(id, b, &ray.angle, root, media)?,
        size,
        ray.contain.is_some(),
        &center,
        ray.center.is_some(),
    )) as *mut dyn BasicShape;
    Ok(Member::from_ptr(
        MakeGarbageCollected(unsafe { ShapeOffsetPathOperation::new(shape, coord) })
            as *mut dyn OffsetPathOperation,
    ))
}
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    let inherit = v.IsInheritedValue();
    let initial = v.IsInitialValue() || v.IsUnsetValue() || inherit && parent.is_none();
    if inherit
        && !initial
        && id != CSSPropertyID::kOffsetRotate
        && parent.unwrap().EffectiveZoom() != b.EffectiveZoom()
    {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    match id {
        CSSPropertyID::kOffsetPath => {
            let value = if initial {
                Member::default()
            } else if inherit {
                parent.unwrap().OffsetPath().clone()
            } else {
                Path(id, b, v, root, media)?
            };
            b.SetOffsetPath(value);
        }
        CSSPropertyID::kOffsetRotate => {
            let value = if initial {
                ComputedStyleInitialValues::InitialOffsetRotate()
            } else if inherit {
                *parent.unwrap().OffsetRotate()
            } else {
                Rotate(id, b, v, root, media)?
            };
            b.SetOffsetRotateOwned(value);
        }
        CSSPropertyID::kOffsetDistance => {
            let value = if initial {
                ComputedStyleInitialValues::InitialOffsetDistance()
            } else if inherit {
                parent.unwrap().OffsetDistance().clone()
            } else {
                crate::resolver::position_repeat_application::ConvertLength(id, v, b, root, media)?
            };
            b.SetOffsetDistanceOwned(value);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseDeclarationList, ParseProperty, PropertyParseErrorKind},
    };
    use layoutng_style::style::{basic_shapes::ShapeType, offset_path_operation::OperationType};
    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn snapshot(b: &ComputedStyleBuilder) -> &ComputedStyle {
        unsafe { &*b.CloneStyle() }
    }
    fn apply(b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, css: &str) {
        let parsed = ParseDeclarationList(
            &foundation::String::from(css),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        assert!(!parsed.properties.is_empty());
        for p in parsed.properties {
            super::super::Apply(
                p.PropertyID(),
                b,
                parent,
                p.Value(),
                16.0,
                &MediaValuesCachedData::default(),
            )
            .unwrap();
        }
    }
    fn operation(b: &ComputedStyleBuilder) -> &dyn OffsetPathOperation {
        unsafe { b.OffsetPath().GetNonNull().unwrap().as_ref() }
    }
    fn ray(b: &ComputedStyleBuilder) -> &StyleRay {
        let op = operation(b);
        assert!(op.GetType() == OperationType::kShape);
        let shape =
            unsafe { &*(op as *const dyn OffsetPathOperation as *const ShapeOffsetPathOperation) }
                .GetBasicShape();
        assert_eq!(shape.GetType(), ShapeType::kStyleRayType);
        unsafe { &*(shape as *const dyn BasicShape as *const StyleRay) }
    }
    #[test]
    fn production_motion_path_none_coord_boxes_and_source_canonicalization() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        for (css, coord) in [
            ("content-box", CoordBox::kContentBox),
            ("padding-box", CoordBox::kPaddingBox),
            ("border-box", CoordBox::kBorderBox),
            ("fill-box", CoordBox::kFillBox),
            ("stroke-box", CoordBox::kStrokeBox),
            ("view-box", CoordBox::kViewBox),
        ] {
            apply(&mut b, None, &format!("offset-path:{css}"));
            assert_eq!(operation(&b).GetType(), OperationType::kCoordBox);
            assert_eq!(operation(&b).GetCoordBox(), coord);
        }
        let p = ParseProperty(
            CSSPropertyID::kOffsetPath,
            &foundation::String::from("border-box ray(contain 90deg at right 10px bottom 20%)"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(
            p[0].Value().CssText().Utf8(),
            "ray(90deg contain at right 10px bottom 20%)"
        );
        apply(&mut b, None, "offset-path:none");
        assert!(b.OffsetPath().GetNonNull().is_none());
    }
    #[test]
    fn production_motion_ray_real_native_geometry_defaults_and_typed_math() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b,None,"offset-path:ray(calc(0.25turn + 90deg) farthest-corner contain at right 10px bottom 20%) stroke-box");
        let r = ray(&b);
        assert_eq!(r.Angle(), 180.0);
        assert_eq!(r.Size(), RaySize::kFarthestCorner);
        assert!(r.Contain() && r.HasExplicitCenter());
        assert!(r.Center().X().IsCalculated());
        assert_eq!(r.Center().Y().PercentValue(), 80.0);
        assert_eq!(operation(&b).GetCoordBox(), CoordBox::kStrokeBox);
        let p = foundation::gfx::PointF::new(25.0, 40.0);
        let size = foundation::gfx::SizeF::new(100.0, 80.0);
        let expected = 75.0_f32.hypot(40.0);
        assert!((r.CalculateRayPathLength(&p, &size) - expected).abs() < 0.0001);
        let out = r.PointAndNormalAtLength(&p, 10.0);
        assert!((out.point.x() - 25.0).abs() < 0.0001);
        assert!((out.point.y() - 50.0).abs() < 0.0001);
        assert_eq!(out.tangent_in_degrees, 90.0);
        apply(&mut b, None, "offset-path:ray(90deg)");
        let r = ray(&b);
        assert_eq!(r.Size(), RaySize::kClosestSide);
        assert!(!r.Contain() && !r.HasExplicitCenter());
        assert_eq!(r.Center().X().PercentValue(), 50.0);
        assert_eq!(r.CalculateRayPathLength(&p, &size), 25.0);
        for (keyword, length) in [
            ("farthest-side", 75.0),
            ("closest-corner", 25.0_f32.hypot(40.0)),
            ("sides", 75.0),
        ] {
            apply(&mut b, None, &format!("offset-path:ray(90deg {keyword})"));
            assert!((ray(&b).CalculateRayPathLength(&p, &size) - length).abs() < 0.0001);
        }
        assert_eq!(
            ray(&b).CalculateRayPathLength(&foundation::gfx::PointF::new(-1.0, 0.0), &size),
            0.0
        );
    }
    #[test]
    fn production_motion_rotate_keywords_angle_math_native_and_css_wide() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        for (value, angle, type_) in [
            ("auto", 0.0, OffsetRotationType::kAuto),
            ("reverse", 180.0, OffsetRotationType::kAuto),
            ("45deg reverse", 225.0, OffsetRotationType::kAuto),
            ("auto -0.5turn", -180.0, OffsetRotationType::kAuto),
            ("calc(1turn / 4)", 90.0, OffsetRotationType::kFixed),
        ] {
            apply(&mut b, None, &format!("offset-rotate:{value}"));
            assert_eq!(b.OffsetRotate().angle, angle);
            assert_eq!(b.OffsetRotate().r#type, type_);
        }
        let parent = snapshot(&b);
        let mut child = ComputedStyleBuilder::from_style(initial());
        apply(&mut child, Some(parent), "offset-rotate:inherit");
        assert_eq!(child.OffsetRotate(), parent.OffsetRotate());
        apply(&mut child, Some(parent), "offset-rotate:unset");
        assert_eq!(child.OffsetRotate().angle, 0.0);
        assert_eq!(child.OffsetRotate().r#type, OffsetRotationType::kAuto);
        apply(&mut child, None, "offset-rotate:inherit");
        assert_eq!(child.OffsetRotate().r#type, OffsetRotationType::kAuto);
    }
    #[test]
    fn production_motion_shorthand_order_resets_all_five_native_fields_and_inherits() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            None,
            "offset:10px 20px ray(30deg) reverse 25% / right bottom",
        );
        assert_eq!(b.OffsetPosition().X().Pixels(), 10.0);
        assert_eq!(b.OffsetPosition().Y().Pixels(), 20.0);
        assert_eq!(b.OffsetDistance().PercentValue(), 25.0);
        assert_eq!(b.OffsetRotate().angle, 180.0);
        assert_eq!(b.OffsetAnchor().X().PercentValue(), 100.0);
        assert_eq!(ray(&b).Angle(), 30.0);
        let parent = snapshot(&b);
        let mut child = ComputedStyleBuilder::from_style(initial());
        apply(&mut child, Some(parent), "offset:inherit");
        assert!(child.OffsetPath() == parent.OffsetPath());
        assert_eq!(child.OffsetDistance(), parent.OffsetDistance());
        assert_eq!(child.OffsetPosition(), parent.OffsetPosition());
        assert_eq!(child.OffsetAnchor(), parent.OffsetAnchor());
        apply(&mut child, None, "offset:none");
        assert!(child.OffsetPath().GetNonNull().is_none());
        assert_eq!(child.OffsetDistance().Pixels(), 0.0);
        assert_eq!(child.OffsetRotate().r#type, OffsetRotationType::kAuto);
        assert_eq!(child.OffsetRotate().angle, 0.0);
        assert!(child.OffsetAnchor().X().IsAuto());
        assert!(child.OffsetPosition().X().GetType() == foundation::LengthType::kNone);
        apply(&mut child, None, "offset:center / left top");
        assert!(child.OffsetPath().GetNonNull().is_none());
        assert_eq!(child.OffsetPosition().X().PercentValue(), 50.0);
        assert_eq!(child.OffsetAnchor().X().PercentValue(), 0.0);
        apply(&mut child, None, "offset:none calc(20% + 5px) auto 10deg");
        assert!(child.OffsetDistance().IsCalculated());
        assert_eq!(child.OffsetRotate().angle, 10.0);
        apply(&mut child, None, "offset:initial");
        assert!(child.OffsetAnchor().X().IsAuto() && child.OffsetPath().GetNonNull().is_none());
    }
    #[test]
    fn production_motion_invalid_unavailable_path_resource_and_zoom_boundaries() {
        let _heap = foundation::LayoutHeapScope::new();
        for (id, value) in [
            (CSSPropertyID::kOffsetPath, "ray(0)"),
            (CSSPropertyID::kOffsetPath, "ray(contain)"),
            (CSSPropertyID::kOffsetPath, "ray(10deg 20deg)"),
            (CSSPropertyID::kOffsetPath, "ray(10deg at center at left)"),
            (CSSPropertyID::kOffsetPath, "margin-box"),
            (CSSPropertyID::kOffsetPath, "none border-box"),
            (CSSPropertyID::kOffsetRotate, "0"),
            (CSSPropertyID::kOffsetRotate, "auto reverse"),
            (CSSPropertyID::kOffsetRotate, "10%"),
            (CSSPropertyID::kOffset, "auto 20px"),
            (CSSPropertyID::kOffset, "/ center"),
            (CSSPropertyID::kOffset, "none /"),
            (CSSPropertyID::kOffset, "none 10deg 20deg"),
        ] {
            assert!(
                ParseProperty(
                    id,
                    &foundation::String::from(value),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .is_err(),
                "{id:?}:{value}"
            );
        }
        for value in [
            "circle(20%)",
            "polygon(0 0,100% 0,50% 100%)",
            "path('M0 0 L10 10')",
            "url(#path)",
        ] {
            assert_eq!(
                ParseProperty(
                    CSSPropertyID::kOffsetPath,
                    &foundation::String::from(value),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .err()
                .unwrap()
                .kind,
                PropertyParseErrorKind::Unsupported
            );
        }
        let mut parent = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut parent,
            None,
            "offset-path:ray(10deg);offset-distance:10px",
        );
        let parent = snapshot(&parent);
        let mut child = ComputedStyleBuilder::from_style(initial());
        child.SetEffectiveZoom(2.0);
        for id in [CSSPropertyID::kOffsetPath, CSSPropertyID::kOffsetDistance] {
            let p = ParseProperty(
                id,
                &foundation::String::from("inherit"),
                false,
                CSSParserMode::kHTMLStandardMode,
            )
            .unwrap();
            assert!(
                matches!(Apply(id,&mut child,Some(parent),p[0].Value(),16.0,&MediaValuesCachedData::default()),Err(LonghandApplicationError::Unsupported(i)) if i==id)
            );
        }
        assert!(child.OffsetPath().GetNonNull().is_none());
    }
}
