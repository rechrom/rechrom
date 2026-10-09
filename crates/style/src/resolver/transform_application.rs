// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! TransformBuilder and independent transform converters over typed CSS values.
#![allow(non_snake_case)]
use super::*;
use crate::css_math_expression_node::{CSSMathLengthResolver, CalculationResultCategory as C};
use crate::css_value_clamping_utils::CSSValueClampingUtils as Clamp;
use crate::production_css_value::{CSSFunctionValue, ListSeparator};
use crate::resolver::position_repeat_application::ConvertLength;
use foundation::transform_operations::{
    Matrix3DTransformOperation, MatrixTransformOperation, PerspectiveTransformOperation,
    SkewTransformOperation,
};
use foundation::{
    gfx, MakeGarbageCollected, Member, RotateTransformOperation, ScaleTransformOperation,
    TransformOperation, TransformOperationType as O, TransformOperations,
    TranslateTransformOperation,
};

type Converted<T> = std::result::Result<T, LonghandApplicationError>;
pub(super) fn IsTransformProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kTransform
            | kTranslate
            | kRotate
            | kScale
            | kPerspective
            | kWebkitTransformOriginX
            | kWebkitTransformOriginY
            | kWebkitTransformOriginZ
            | kWebkitPerspectiveOriginX
            | kWebkitPerspectiveOriginY
    )
}
fn NoneValue(v: &Value) -> bool {
    matches!(v.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == CSSValueID::kNone)
}
fn List(id: CSSPropertyID, v: &Value, min: usize, max: usize) -> Converted<&[std::rc::Rc<Value>]> {
    if let CSSValuePayload::kValueListClass(list) = v.Payload() {
        if list.separator == ListSeparator::Space && (min..=max).contains(&list.values.len()) {
            return Ok(&list.values);
        }
    }
    Err(LonghandApplicationError::InvalidValue(id))
}
// cpp: CSSPrimitiveValue ComputeNumber/ComputeDegrees/ComputeLength<double>;
// css_value_clamping_utils.h:26-58. Reuse the shared actual length resolver.
pub(super) fn Scalar(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    category: C,
    root: f32,
    media: &MediaValuesCachedData,
) -> Converted<f64> {
    let mut resolver = super::MathLengthResolver(
        id,
        b.GetFontDescription().ComputedSize(),
        root,
        b.EffectiveZoom(),
        media,
    );
    let number = match v.Payload() {
        CSSValuePayload::kMathFunctionClass(math) => {
            if math.Category() != category
                && !(category == C::Number && math.Category() == C::Percent)
            {
                return Err(LonghandApplicationError::InvalidValue(id));
            }
            let n = math
                .ComputeValue(&mut resolver, None)
                .map_err(|_| LonghandApplicationError::Unsupported(id))?;
            if category == C::Number && math.Category() == C::Percent {
                n / 100.0
            } else {
                n
            }
        }
        CSSValuePayload::kNumericLiteralClass(n) => match category {
            C::Number if n.IsNumber() => n.DoubleValue(),
            C::Number if n.GetType() == UnitType::kPercentage => n.DoubleValue() / 100.0,
            C::Angle => {
                n.DoubleValue()
                    * match n.GetType() {
                        UnitType::kDegrees => 1.0,
                        UnitType::kRadians => 180.0 / std::f64::consts::PI,
                        UnitType::kGradians => 0.9,
                        UnitType::kTurns => 360.0,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    }
            }
            C::Length if n.GetType() != UnitType::kPercentage => resolver
                .ComputeLength(n.DoubleValue(), n.GetType())
                .map_err(|_| LonghandApplicationError::Unsupported(id))?,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        },
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    };
    Ok(match category {
        C::Angle => Clamp::ClampAngle(number),
        C::Length => Clamp::ClampLength(number),
        _ => Clamp::ClampDouble(number),
    })
}
fn MemberOperation<T: TransformOperation + foundation::Traceable + 'static>(
    value: T,
) -> Member<dyn TransformOperation> {
    Member::from_ptr(MakeGarbageCollected(value) as *mut dyn TransformOperation)
}
// cpp: transform_builder.cc:49-103,131-349 CreateTransformOperation.
fn CreateOperation(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    function: &CSSFunctionValue,
    root: f32,
    media: &MediaValuesCachedData,
) -> Converted<Member<dyn TransformOperation>> {
    use CSSValueID::*;
    let args = &function.arguments.values;
    let count = args.len();
    let invalid = || LonghandApplicationError::InvalidValue(id);
    let scalar = |index: usize, category| {
        Scalar(
            id,
            b,
            args.get(index).ok_or_else(invalid)?,
            category,
            root,
            media,
        )
    };
    let length =
        |index: usize| ConvertLength(id, args.get(index).ok_or_else(invalid)?, b, root, media);
    let operation = match function.function_id {
        kScale | kScaleX | kScaleY | kScaleZ | kScale3d => {
            let kind = match function.function_id {
                kScale => O::kScale,
                kScaleX => O::kScaleX,
                kScaleY => O::kScaleY,
                kScaleZ => O::kScaleZ,
                _ => O::kScale3D,
            };
            if !(if kind == O::kScale {
                1..=2
            } else if kind == O::kScale3D {
                3..=3
            } else {
                1..=1
            })
            .contains(&count)
            {
                return Err(invalid());
            }
            let first = scalar(0, C::Number)?;
            let (x, y, z) = match kind {
                O::kScaleX => (first, 1.0, 1.0),
                O::kScaleY => (1.0, first, 1.0),
                O::kScaleZ => (1.0, 1.0, first),
                O::kScale3D => (first, scalar(1, C::Number)?, scalar(2, C::Number)?),
                _ => (
                    first,
                    if count > 1 {
                        scalar(1, C::Number)?
                    } else {
                        first
                    },
                    1.0,
                ),
            };
            MemberOperation(ScaleTransformOperation::new(x, y, z, kind))
        }
        kTranslate | kTranslateX | kTranslateY | kTranslateZ | kTranslate3d => {
            let kind = match function.function_id {
                kTranslate => O::kTranslate,
                kTranslateX => O::kTranslateX,
                kTranslateY => O::kTranslateY,
                kTranslateZ => O::kTranslateZ,
                _ => O::kTranslate3D,
            };
            if !(if kind == O::kTranslate {
                1..=2
            } else if kind == O::kTranslate3D {
                3..=3
            } else {
                1..=1
            })
            .contains(&count)
            {
                return Err(invalid());
            }
            let (x, y, z) = match kind {
                O::kTranslateZ => (Length::Fixed(0), Length::Fixed(0), scalar(0, C::Length)?),
                O::kTranslateY => (Length::Fixed(0), length(0)?, 0.0),
                O::kTranslate3D => (length(0)?, length(1)?, scalar(2, C::Length)?),
                _ => (
                    length(0)?,
                    if count > 1 {
                        length(1)?
                    } else {
                        Length::Fixed(0)
                    },
                    0.0,
                ),
            };
            MemberOperation(TranslateTransformOperation::new(x, y, z, kind))
        }
        kRotate | kRotateX | kRotateY | kRotateZ | kRotate3d => {
            let kind = match function.function_id {
                kRotate => O::kRotate,
                kRotateX => O::kRotateX,
                kRotateY => O::kRotateY,
                kRotateZ => O::kRotateZ,
                _ => O::kRotate3D,
            };
            if count != if kind == O::kRotate3D { 4 } else { 1 } {
                return Err(invalid());
            }
            let (x, y, z, angle) = if kind == O::kRotate3D {
                (
                    scalar(0, C::Number)?,
                    scalar(1, C::Number)?,
                    scalar(2, C::Number)?,
                    scalar(3, C::Angle)?,
                )
            } else {
                (
                    if kind == O::kRotateX { 1.0 } else { 0.0 },
                    if kind == O::kRotateY { 1.0 } else { 0.0 },
                    if matches!(kind, O::kRotate | O::kRotateZ) {
                        1.0
                    } else {
                        0.0
                    },
                    scalar(0, C::Angle)?,
                )
            };
            MemberOperation(RotateTransformOperation::new_3d(x, y, z, angle, kind))
        }
        kSkew | kSkewX | kSkewY => {
            if !(if function.function_id == kSkew {
                1..=2
            } else {
                1..=1
            })
            .contains(&count)
            {
                return Err(invalid());
            }
            let a = scalar(0, C::Angle)?;
            let (x, y, kind) = match function.function_id {
                kSkewX => (a, 0.0, O::kSkewX),
                kSkewY => (0.0, a, O::kSkewY),
                _ => (
                    a,
                    if count > 1 { scalar(1, C::Angle)? } else { 0.0 },
                    O::kSkew,
                ),
            };
            MemberOperation(SkewTransformOperation::new(x, y, kind))
        }
        kMatrix => {
            if count != 6 {
                return Err(invalid());
            }
            MemberOperation(MatrixTransformOperation::new(
                scalar(0, C::Number)?,
                scalar(1, C::Number)?,
                scalar(2, C::Number)?,
                scalar(3, C::Number)?,
                scalar(4, C::Number)? * b.EffectiveZoom() as f64,
                scalar(5, C::Number)? * b.EffectiveZoom() as f64,
            ))
        }
        kMatrix3d => {
            if count != 16 {
                return Err(invalid());
            }
            let mut numbers = [0.0; 16];
            for (i, n) in numbers.iter_mut().enumerate() {
                *n = scalar(i, C::Number)?;
            }
            let mut matrix = gfx::Transform::ColMajor(&numbers);
            matrix.Zoom(b.EffectiveZoom());
            MemberOperation(Matrix3DTransformOperation::new(matrix))
        }
        kPerspective => {
            if count != 1 {
                return Err(invalid());
            }
            let p = if NoneValue(&args[0]) {
                None
            } else {
                Some(scalar(0, C::Length)?.max(0.0))
            };
            MemberOperation(PerspectiveTransformOperation::new(p))
        }
        _ => return Err(invalid()),
    };
    Ok(operation)
}
// cpp: transform_builder.cc:353-370 CreateTransformOperations.
fn ConvertTransform(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Converted<TransformOperations> {
    let mut operations = TransformOperations::default();
    if NoneValue(v) {
        return Ok(operations);
    }
    if let CSSValuePayload::kFunctionClass(function) = v.Payload() {
        operations
            .OperationsMut()
            .push_back(CreateOperation(id, b, function, root, media)?);
    } else {
        for value in List(id, v, 1, usize::MAX)? {
            let CSSValuePayload::kFunctionClass(function) = value.Payload() else {
                return Err(LonghandApplicationError::InvalidValue(id));
            };
            operations
                .OperationsMut()
                .push_back(CreateOperation(id, b, function, root, media)?);
        }
    }
    Ok(operations)
}
// cpp: generated longhands.cc Transform:17401-17414, Translate:17757-17770,
// Rotate:13735-13743, Scale:14269-14277, Perspective:13199-13212;
// origin axes:18992-19023,19323-19380; longhands_custom.cc axis inheritance.
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
        && !initial
        && parent.unwrap().EffectiveZoom() != b.EffectiveZoom()
        && !matches!(id, kRotate | kScale)
    {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    match id {
        kTransform => {
            let operations = if initial {
                ComputedStyleInitialValues::InitialTransform()
            } else if inherit {
                parent.unwrap().Transform().clone()
            } else {
                ConvertTransform(id, b, v, root, media)?
            };
            b.SetTransform(&operations);
        }
        kTranslate => {
            let operation = if initial || NoneValue(v) {
                Member::default()
            } else if inherit {
                Member::from_ptr(parent.unwrap().Translate())
            } else {
                let list = List(id, v, 1, 3)?;
                let x = ConvertLength(id, &list[0], b, root, media)?;
                let y = if list.len() > 1 {
                    ConvertLength(id, &list[1], b, root, media)?
                } else {
                    Length::Fixed(0)
                };
                let z = if list.len() > 2 {
                    Scalar(id, b, &list[2], C::Length, root, media)?
                } else {
                    0.0
                };
                Member::from_ptr(MakeGarbageCollected(TranslateTransformOperation::new(
                    x,
                    y,
                    z,
                    O::kTranslate3D,
                )))
            };
            b.SetTranslate(operation);
        }
        kRotate => {
            let operation = if initial || NoneValue(v) {
                Member::default()
            } else if inherit {
                Member::from_ptr(parent.unwrap().Rotate())
            } else {
                let list = List(id, v, 1, 2)?;
                let mut axis = [0.0, 0.0, 1.0];
                if list.len() == 2 {
                    let CSSValuePayload::kAxisClass(a) = list[0].Payload() else {
                        return Err(LonghandApplicationError::InvalidValue(id));
                    };
                    if a.dimensions.values.len() != 3 {
                        return Err(LonghandApplicationError::InvalidValue(id));
                    }
                    for (i, n) in axis.iter_mut().enumerate() {
                        *n = Scalar(id, b, &a.dimensions.values[i], C::Number, root, media)?;
                    }
                    // css_axis_value.cc:101-110 normalizes positive parallel axes.
                    for i in 0..3 {
                        if axis[i] > 0.0 && (0..3).all(|other| other == i || axis[other] == 0.0) {
                            axis[i] = 1.0;
                            break;
                        }
                    }
                }
                let angle = Scalar(id, b, list.last().unwrap(), C::Angle, root, media)?;
                Member::from_ptr(MakeGarbageCollected(RotateTransformOperation::new_3d(
                    axis[0],
                    axis[1],
                    axis[2],
                    angle,
                    O::kRotate3D,
                )))
            };
            b.SetRotate(operation);
        }
        kScale => {
            let operation = if initial || NoneValue(v) {
                Member::default()
            } else if inherit {
                Member::from_ptr(parent.unwrap().Scale())
            } else {
                let list = List(id, v, 1, 3)?;
                let x = Scalar(id, b, &list[0], C::Number, root, media)?;
                let y = if list.len() > 1 {
                    Scalar(id, b, &list[1], C::Number, root, media)?
                } else {
                    x
                };
                let z = if list.len() > 2 {
                    Scalar(id, b, &list[2], C::Number, root, media)?
                } else {
                    1.0
                };
                Member::from_ptr(MakeGarbageCollected(ScaleTransformOperation::new(
                    x,
                    y,
                    z,
                    O::kScale3D,
                )))
            };
            b.SetScale(operation);
        }
        kPerspective => {
            let p = if initial || NoneValue(v) {
                ComputedStyleInitialValues::InitialPerspective()
            } else if inherit {
                parent.unwrap().Perspective()
            } else {
                Scalar(id, b, v, C::Length, root, media)?
                    .max(0.0)
                    .clamp(0.0, f32::MAX as f64) as f32
            };
            b.SetPerspective(p);
        }
        kWebkitTransformOriginZ => {
            let z = if initial {
                ComputedStyleInitialValues::InitialTransformOriginZ()
            } else if inherit {
                parent.unwrap().GetTransformOrigin().Z()
            } else {
                Scalar(id, b, v, C::Length, root, media)?.clamp(-(f32::MAX as f64), f32::MAX as f64)
                    as f32
            };
            b.SetTransformOriginZ(z);
        }
        kWebkitTransformOriginX
        | kWebkitTransformOriginY
        | kWebkitPerspectiveOriginX
        | kWebkitPerspectiveOriginY => {
            let length = if initial {
                match id {
                    kWebkitTransformOriginX => {
                        ComputedStyleInitialValues::InitialTransformOriginX()
                    }
                    kWebkitTransformOriginY => {
                        ComputedStyleInitialValues::InitialTransformOriginY()
                    }
                    kWebkitPerspectiveOriginX => {
                        ComputedStyleInitialValues::InitialPerspectiveOriginX()
                    }
                    _ => ComputedStyleInitialValues::InitialPerspectiveOriginY(),
                }
            } else if inherit {
                let p = parent.unwrap();
                match id {
                    kWebkitTransformOriginX => p.GetTransformOrigin().X().clone(),
                    kWebkitTransformOriginY => p.GetTransformOrigin().Y().clone(),
                    kWebkitPerspectiveOriginX => p.PerspectiveOrigin().X().clone(),
                    _ => p.PerspectiveOrigin().Y().clone(),
                }
            } else {
                ConvertLength(id, v, b, root, media)?
            };
            match id {
                kWebkitTransformOriginX => b.SetTransformOriginX(&length),
                kWebkitTransformOriginY => b.SetTransformOriginY(&length),
                kWebkitPerspectiveOriginX => b.SetPerspectiveOriginX(&length),
                _ => b.SetPerspectiveOriginY(&length),
            }
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    if inherit && !initial && v.IsInheritedValue() {
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
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    };
    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn apply(
        b: &mut ComputedStyleBuilder,
        id: CSSPropertyID,
        css: &str,
        parent: Option<&ComputedStyle>,
    ) -> Result {
        for property in ParseProperty(
            id,
            &foundation::String::from(css),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap()
        {
            super::super::Apply(
                property.PropertyID(),
                b,
                parent,
                property.Value(),
                16.0,
                &MediaValuesCachedData::default(),
            )?;
        }
        Ok(())
    }
    fn matrix(b: &ComputedStyleBuilder) -> [f64; 16] {
        let mut matrix = gfx::Transform::default();
        b.Transform()
            .Apply(&gfx::SizeF::new(200.0, 100.0), &mut matrix);
        matrix.GetColMajor()
    }
    #[test]
    fn production_transform_functions_preserve_native_operation_types_and_math() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            CSSPropertyID::kTransform,
            "matrix(1,0,0,1,3,4) translate(calc(10% + 2px),5px) rotate(90deg) scale(2)",
            None,
        )
        .unwrap();
        assert_eq!(b.Transform().size(), 4);
        for (index, kind) in [O::kMatrix, O::kTranslate, O::kRotate, O::kScale]
            .iter()
            .enumerate()
        {
            assert_eq!(b.Transform().at(index as _).unwrap().GetType(), *kind);
        }
        let values = matrix(&b);
        assert_eq!(values[0], 0.0);
        assert_eq!(values[1], 2.0);
        assert_eq!(values[4], -2.0);
        assert_eq!(values[5], 0.0);
        assert_eq!(values[12..14], [25.0, 9.0]);
        apply(
            &mut b,
            CSSPropertyID::kTransform,
            "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,3,4,5,1)",
            None,
        )
        .unwrap();
        assert_eq!(b.Transform().at(0).unwrap().GetType(), O::kMatrix3D);
        assert_eq!(matrix(&b)[12..15], [3.0, 4.0, 5.0]);
        apply(
            &mut b,
            CSSPropertyID::kTransform,
            "skewX(45deg) perspective(0px)",
            None,
        )
        .unwrap();
        let values = matrix(&b);
        assert!((values[4] - 1.0).abs() < 1e-14);
        assert_eq!(values[11], -1.0);
        apply(&mut b, CSSPropertyID::kTransform, "perspective(none)", None).unwrap();
        assert_eq!(matrix(&b), gfx::Transform::default().GetColMajor());
        for (css, kind) in [
            ("translateX(2px)", O::kTranslateX),
            ("translateY(3%)", O::kTranslateY),
            ("translateZ(4px)", O::kTranslateZ),
            ("translate3d(2px,3%,4px)", O::kTranslate3D),
            ("scaleX(2)", O::kScaleX),
            ("scaleY(50%)", O::kScaleY),
            ("scaleZ(3)", O::kScaleZ),
            ("scale3d(2,50%,3)", O::kScale3D),
            ("rotateX(0.25turn)", O::kRotateX),
            ("rotateY(100grad)", O::kRotateY),
            ("rotateZ(0)", O::kRotateZ),
            ("rotate3d(1,0,0,calc(45deg + 45deg))", O::kRotate3D),
            ("skew(0,45deg)", O::kSkew),
            ("skewY(1rad)", O::kSkewY),
        ] {
            apply(&mut b, CSSPropertyID::kTransform, css, None).unwrap();
            assert_eq!(b.Transform().at(0).unwrap().GetType(), kind, "{css}");
        }
        apply(
            &mut b,
            CSSPropertyID::kAliasWebkitTransform,
            "perspective(calc(2 + 3))",
            None,
        )
        .unwrap();
        assert_eq!(matrix(&b)[11], -0.2);
        apply(
            &mut b,
            CSSPropertyID::kAliasWebkitPerspective,
            "calc(2 + 3)",
            None,
        )
        .unwrap();
        assert_eq!(b.Perspective(), 5.0);
    }
    #[test]
    fn production_independent_transform_and_origins_write_native_fields() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            CSSPropertyID::kTranslate,
            "calc(10% + 2px) 3px calc(1px + 4px)",
            None,
        )
        .unwrap();
        let translate = unsafe { &*b.Translate() };
        assert_eq!(translate.X(&gfx::SizeF::new(200.0, 100.0)), 22.0);
        assert_eq!(translate.Y(&gfx::SizeF::new(200.0, 100.0)), 3.0);
        assert_eq!(translate.Z(), 5.0);
        apply(
            &mut b,
            CSSPropertyID::kRotate,
            "-2 0 0 calc(45deg + 45deg)",
            None,
        )
        .unwrap();
        let rotate = unsafe { &*b.Rotate() };
        assert_eq!(*rotate.Axis(), [1.0, 0.0, 0.0]);
        assert_eq!(rotate.Angle(), -90.0);
        apply(&mut b, CSSPropertyID::kRotate, "calc(90deg) y", None).unwrap();
        assert_eq!(*unsafe { &*b.Rotate() }.Axis(), [0.0, 1.0, 0.0]);
        apply(
            &mut b,
            CSSPropertyID::kRotate,
            "calc(2) calc(0) calc(0) 30deg",
            None,
        )
        .unwrap();
        assert_eq!(*unsafe { &*b.Rotate() }.Axis(), [1.0, 0.0, 0.0]);
        apply(&mut b, CSSPropertyID::kScale, "calc(50% + 50%) 2 3", None).unwrap();
        let scale = unsafe { &*b.Scale() };
        assert_eq!((scale.X(), scale.Y(), scale.Z()), (1.0, 2.0, 3.0));
        apply(
            &mut b,
            CSSPropertyID::kTransformOrigin,
            "calc(50% + 2px) top calc(1px + 2px)",
            None,
        )
        .unwrap();
        assert!(b.GetTransformOrigin().X().IsCalculated());
        assert_eq!(b.GetTransformOrigin().Y().PercentValue(), 0.0);
        assert_eq!(b.GetTransformOrigin().Z(), 3.0);
        apply(
            &mut b,
            CSSPropertyID::kWebkitTransformOriginX,
            "right",
            None,
        )
        .unwrap();
        apply(
            &mut b,
            CSSPropertyID::kWebkitTransformOriginY,
            "calc(25% + 1px)",
            None,
        )
        .unwrap();
        apply(&mut b, CSSPropertyID::kWebkitTransformOriginZ, "-4px", None).unwrap();
        assert_eq!(b.GetTransformOrigin().X().PercentValue(), 100.0);
        assert!(b.GetTransformOrigin().Y().IsCalculated());
        assert_eq!(b.GetTransformOrigin().Z(), -4.0);
        apply(
            &mut b,
            CSSPropertyID::kPerspectiveOrigin,
            "right calc(2px + 1px) bottom 10%",
            None,
        )
        .unwrap();
        assert!(b.PerspectiveOrigin().X().IsCalculated());
        assert_eq!(b.PerspectiveOrigin().Y().PercentValue(), 90.0);
        apply(
            &mut b,
            CSSPropertyID::kWebkitPerspectiveOriginX,
            "left",
            None,
        )
        .unwrap();
        apply(
            &mut b,
            CSSPropertyID::kWebkitPerspectiveOriginY,
            "top",
            None,
        )
        .unwrap();
        assert_eq!(b.PerspectiveOrigin().X().PercentValue(), 0.0);
        assert_eq!(b.PerspectiveOrigin().Y().PercentValue(), 0.0);
        apply(&mut b, CSSPropertyID::kPerspective, "calc(2px - 3px)", None).unwrap();
        assert_eq!(b.Perspective(), 0.0);
    }
    #[test]
    fn production_transform_initial_inherit_and_zoom_are_explicit() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = ComputedStyleBuilder::from_style(initial());
        let declarations = [
            (CSSPropertyID::kTransform, "translateX(12px)"),
            (CSSPropertyID::kTranslate, "2px 3px"),
            (CSSPropertyID::kRotate, "30deg"),
            (CSSPropertyID::kScale, "2 3"),
            (CSSPropertyID::kTransformOrigin, "left top 4px"),
            (CSSPropertyID::kPerspective, "5px"),
            (CSSPropertyID::kPerspectiveOrigin, "left top"),
        ];
        for (id, css) in declarations {
            apply(&mut parent, id, css, None).unwrap();
        }
        let parent = unsafe { &*parent.TakeStyle() };
        let mut b = ComputedStyleBuilder::from_style(initial());
        for (id, _) in declarations {
            apply(&mut b, id, "inherit", Some(parent)).unwrap();
        }
        assert_eq!(matrix(&b)[12], 12.0);
        assert_eq!(b.Translate(), parent.Translate());
        assert_eq!(b.Rotate(), parent.Rotate());
        assert_eq!(b.Scale(), parent.Scale());
        assert!(b.GetTransformOrigin() == parent.GetTransformOrigin());
        assert_eq!(b.Perspective(), 5.0);
        assert!(b.PerspectiveOrigin() == parent.PerspectiveOrigin());
        for id in [
            CSSPropertyID::kWebkitTransformOriginX,
            CSSPropertyID::kWebkitTransformOriginY,
            CSSPropertyID::kWebkitTransformOriginZ,
            CSSPropertyID::kWebkitPerspectiveOriginX,
            CSSPropertyID::kWebkitPerspectiveOriginY,
        ] {
            apply(&mut b, id, "inherit", Some(parent)).unwrap();
        }
        for (id, _) in declarations {
            apply(&mut b, id, "initial", Some(parent)).unwrap();
        }
        assert_eq!(b.Transform().size(), 0);
        assert!(b.Translate().is_null() && b.Rotate().is_null() && b.Scale().is_null());
        assert_eq!(b.Perspective(), -1.0);
        assert_eq!(b.GetTransformOrigin().X().PercentValue(), 50.0);
        assert_eq!(b.GetTransformOrigin().Z(), 0.0);
        b.SetEffectiveZoom(2.0);
        assert_eq!(
            apply(&mut b, CSSPropertyID::kTransform, "inherit", Some(parent)),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kTransform
            ))
        );
        apply(
            &mut b,
            CSSPropertyID::kTransform,
            "matrix(1,0,0,1,3,4) translateX(2px)",
            None,
        )
        .unwrap();
        assert_eq!(matrix(&b)[12..14], [10.0, 8.0]);
        apply(&mut b, CSSPropertyID::kRotate, "inherit", Some(parent)).unwrap();
    }
    #[test]
    fn production_transform_invalid_grammar_and_uncovered_functions_stay_typed() {
        use CSSPropertyID::*;
        for (id, css) in [
            (kTransform, "matrix(1 0 0 1 2 3)"),
            (kTransform, "translate(2px 3px)"),
            (kTransform, "translate3d(1px,2px,3%)"),
            (kTransform, "rotate(2)"),
            (kTransform, "scale(2,)"),
            (kTransform, "matrix3d(1,0)"),
            (kTransform, "perspective(-1px)"),
            (kTranslate, "1px 2px 3%"),
            (kRotate, "0"),
            (kRotate, "x"),
            (kRotate, "1 2 30deg"),
            (kScale, "2 3 4 5"),
            (kPerspective, "-1px"),
            (kPerspective, "2"),
            (kTransformOrigin, "top 2px"),
            (kTransformOrigin, "left top 3%"),
        ] {
            let error = ParseProperty(
                id,
                &foundation::String::from(css),
                false,
                CSSParserMode::kHTMLStandardMode,
            )
            .err()
            .unwrap();
            assert_eq!(error.kind, PropertyParseErrorKind::Invalid, "{id:?}: {css}");
        }
        for (id, css) in [
            (kTransform, "scale(sin(1deg))"),
            (kTranslate, "round(2px,1px)"),
            (kRotate, "atan(1)"),
            (kScale, "pow(2,3)"),
            (kTransformOrigin, "left top sqrt(1px)"),
        ] {
            let error = ParseProperty(
                id,
                &foundation::String::from(css),
                false,
                CSSParserMode::kHTMLStandardMode,
            )
            .err()
            .unwrap();
            assert_eq!(
                error.kind,
                PropertyParseErrorKind::Unsupported,
                "{id:?}: {css}"
            );
        }
    }
}
