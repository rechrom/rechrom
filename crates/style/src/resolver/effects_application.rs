// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium visual-effect application against real native style owners.
#![allow(non_snake_case)]
use super::*;
use foundation::{ECursor, MakeGarbageCollected, Member};
use layoutng_style::style::{
    basic_shapes::{BasicShape, BasicShapePolygon},
    clip_path_operation::ClipPathOperation,
    computed_style_constants::{GeometryBox, ShapeBox},
    filter_operation::*,
    filter_operations::FilterOperations,
    geometry_box_clip_path_operation::GeometryBoxClipPathOperation,
    shape_clip_path_operation::ShapeClipPathOperation,
    shape_value::ShapeValue,
};
pub(super) fn IsEffectsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kFilter
            | CSSPropertyID::kBackdropFilter
            | CSSPropertyID::kClipPath
            | CSSPropertyID::kCursor
            | CSSPropertyID::kShapeOutside
            | CSSPropertyID::kShapeImageThreshold
    )
}
fn Identifier(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<CSSValueID, LonghandApplicationError> {
    if let CSSValuePayload::kIdentifierClass(i) = v.Payload() {
        Ok(i.0)
    } else {
        Err(LonghandApplicationError::InvalidValue(id))
    }
}
fn Number(
    id: CSSPropertyID,
    v: &Value,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<(f64, UnitType), LonghandApplicationError> {
    match v.Payload() {
        CSSValuePayload::kNumericLiteralClass(n) => Ok((n.DoubleValue(), n.GetType())),
        CSSValuePayload::kMathFunctionClass(m) => {
            let number = m
                .ComputeValue(
                    &mut MathLengthResolver(
                        id,
                        b.GetFontDescription().ComputedSize(),
                        root,
                        b.EffectiveZoom(),
                        media,
                    ),
                    None,
                )
                .map_err(|_| LonghandApplicationError::Unsupported(id))?;
            Ok((
                number,
                m.expression
                    .CanonicalUnit()
                    .ok_or(LonghandApplicationError::InvalidValue(id))?,
            ))
        }
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}
fn LengthValue(
    id: CSSPropertyID,
    v: &Value,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<Length, LonghandApplicationError> {
    if let CSSValuePayload::kMathFunctionClass(m) = v.Payload() {
        return m
            .ConvertToLength(&mut MathLengthResolver(
                id,
                b.GetFontDescription().ComputedSize(),
                root,
                b.EffectiveZoom(),
                media,
            ))
            .map_err(|_| LonghandApplicationError::Unsupported(id));
    }
    let (number, unit) = Number(id, v, b, root, media)?;
    if unit == UnitType::kPercentage {
        return Ok(Length::Percent(number));
    }
    if b.EffectiveZoom() != 1.0 {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    Ok(Length::Fixed(
        crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(Pixels(
            id,
            number,
            unit,
            b.GetFontDescription().ComputedSize(),
            root,
            media,
        )?) as f32,
    ))
}
// cpp: filter_operation_resolver.cc:47-73,128-170,174-190,192-273. Missing SVG URL/
// reference ownership stays a typed error. These are the genuine native classes.
fn Filter(
    id: CSSPropertyID,
    v: &Value,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<FilterOperations, LonghandApplicationError> {
    let mut operations = FilterOperations::new();
    if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(i) if i.0==CSSValueID::kNone) {
        return Ok(operations);
    }
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    for value in &list.values {
        if value.IsURIValue() {
            return Err(LonghandApplicationError::Unsupported(id));
        }
        let CSSValuePayload::kFunctionClass(function) = value.Payload() else {
            return Err(LonghandApplicationError::InvalidValue(id));
        };
        let kind = function.function_id;
        let arguments = &function.arguments.values;
        use CSSValueID::*;
        let operation_type = match kind {
            kGrayscale => OperationType::kGrayscale,
            kSepia => OperationType::kSepia,
            kSaturate => OperationType::kSaturate,
            kHueRotate => OperationType::kHueRotate,
            kInvert => OperationType::kInvert,
            kOpacity => OperationType::kOpacity,
            kBrightness => OperationType::kBrightness,
            kContrast => OperationType::kContrast,
            kBlur => OperationType::kBlur,
            kDropShadow => OperationType::kDropShadow,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        };
        let pointer = match kind {
            kBlur => {
                let length = if let Some(value) = arguments.first() {
                    LengthValue(id, value, b, root, media)?
                } else {
                    Length::Fixed(0.0)
                };
                MakeGarbageCollected(BlurFilterOperation::new(&length)).cast::<FilterOperation>()
            }
            kDropShadow => {
                if arguments.len() != 1 {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                let CSSValuePayload::kShadowClass(shadow) = arguments[0].Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                MakeGarbageCollected(DropShadowFilterOperation::new(ConvertShadow(
                    id, b, shadow, root, media,
                )?))
                .cast::<FilterOperation>()
            }
            _ => {
                let mut amount = if let Some(argument) = arguments.first() {
                    let (value, unit) = Number(id, argument, b, root, media)?;
                    if kind == kHueRotate {
                        match unit {
                            UnitType::kDegrees => value,
                            UnitType::kRadians => value.to_degrees(),
                            UnitType::kGradians => value * 0.9,
                            UnitType::kTurns => value * 360.0,
                            _ => return Err(LonghandApplicationError::InvalidValue(id)),
                        }
                    } else {
                        match unit {
                            UnitType::kNumber | UnitType::kInteger => value,
                            UnitType::kPercentage => value / 100.0,
                            _ => return Err(LonghandApplicationError::InvalidValue(id)),
                        }
                    }
                } else if kind == kHueRotate {
                    0.0
                } else {
                    1.0
                };
                if matches!(kind, kGrayscale | kSepia | kInvert | kOpacity) {
                    amount = amount.clamp(0.0, 1.0);
                }
                if matches!(kind, kGrayscale | kSepia | kSaturate | kHueRotate) {
                    MakeGarbageCollected(BasicColorMatrixFilterOperation::new(
                        amount,
                        operation_type,
                    ))
                    .cast::<FilterOperation>()
                } else {
                    MakeGarbageCollected(BasicComponentTransferFilterOperation::new(
                        amount,
                        operation_type,
                    ))
                    .cast::<FilterOperation>()
                }
            }
        };
        operations.OperationsMut().push(Member::from_ptr(pointer));
    }
    Ok(operations)
}
// cpp: basic_shape_functions.cc:636-653. No lowering to a different shape kind.
pub(super) fn Shape(
    id: CSSPropertyID,
    v: &Value,
    b: &ComputedStyleBuilder,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<*mut dyn BasicShape, LonghandApplicationError> {
    let CSSValuePayload::kBasicShapePolygonClass(value) = v.Payload() else {
        return Err(LonghandApplicationError::Unsupported(id));
    };
    let mut shape = BasicShapePolygon::default();
    shape.SetWindRule(value.wind_rule);
    if let Some(radius) = &value.rounding_radius {
        shape.SetRoundingRadius(&LengthValue(id, radius, b, root, media)?);
    }
    for point in value.coordinates.chunks_exact(2) {
        shape.AppendPoint(
            &LengthValue(id, &point[0], b, root, media)?,
            &LengthValue(id, &point[1], b, root, media)?,
        );
    }
    Ok(MakeGarbageCollected(shape))
}
pub(super) fn Geometry(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<GeometryBox, LonghandApplicationError> {
    use CSSValueID::*;
    Ok(match Identifier(id, v)? {
        kBorderBox => GeometryBox::kBorderBox,
        kPaddingBox => GeometryBox::kPaddingBox,
        kContentBox => GeometryBox::kContentBox,
        kMarginBox => GeometryBox::kMarginBox,
        kFillBox => GeometryBox::kFillBox,
        kStrokeBox => GeometryBox::kStrokeBox,
        kViewBox => GeometryBox::kViewBox,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
fn ShapeBoxValue(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<ShapeBox, LonghandApplicationError> {
    use CSSValueID::*;
    Ok(match Identifier(id, v)? {
        kBorderBox => ShapeBox::kBorderBox,
        kPaddingBox => ShapeBox::kPaddingBox,
        kContentBox => ShapeBox::kContentBox,
        kMarginBox => ShapeBox::kMarginBox,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
fn CursorKeyword(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<ECursor, LonghandApplicationError> {
    use CSSValueID::*;
    Ok(match Identifier(id, v)? {
        CSSValueID::kNone => ECursor::kNone,
        CSSValueID::kCopy => ECursor::kCopy,
        CSSValueID::kAuto => ECursor::kAuto,
        CSSValueID::kCrosshair => ECursor::kCrosshair,
        CSSValueID::kDefault => ECursor::kDefault,
        CSSValueID::kPointer => ECursor::kPointer,
        CSSValueID::kMove => ECursor::kMove,
        CSSValueID::kVerticalText => ECursor::kVerticalText,
        CSSValueID::kCell => ECursor::kCell,
        CSSValueID::kContextMenu => ECursor::kContextMenu,
        CSSValueID::kAlias => ECursor::kAlias,
        CSSValueID::kProgress => ECursor::kProgress,
        CSSValueID::kNoDrop => ECursor::kNoDrop,
        CSSValueID::kNotAllowed => ECursor::kNotAllowed,
        CSSValueID::kZoomIn | kWebkitZoomIn => ECursor::kZoomIn,
        CSSValueID::kZoomOut | kWebkitZoomOut => ECursor::kZoomOut,
        CSSValueID::kEResize => ECursor::kEResize,
        CSSValueID::kNeResize => ECursor::kNeResize,
        CSSValueID::kNwResize => ECursor::kNwResize,
        CSSValueID::kNResize => ECursor::kNResize,
        CSSValueID::kSeResize => ECursor::kSeResize,
        CSSValueID::kSwResize => ECursor::kSwResize,
        CSSValueID::kSResize => ECursor::kSResize,
        CSSValueID::kWResize => ECursor::kWResize,
        CSSValueID::kEwResize => ECursor::kEwResize,
        CSSValueID::kNsResize => ECursor::kNsResize,
        CSSValueID::kNeswResize => ECursor::kNeswResize,
        CSSValueID::kNwseResize => ECursor::kNwseResize,
        CSSValueID::kColResize => ECursor::kColResize,
        CSSValueID::kRowResize => ECursor::kRowResize,
        CSSValueID::kText => ECursor::kText,
        CSSValueID::kWait => ECursor::kWait,
        CSSValueID::kHelp => ECursor::kHelp,
        CSSValueID::kAllScroll => ECursor::kAllScroll,
        CSSValueID::kGrab | kWebkitGrab => ECursor::kGrab,
        CSSValueID::kGrabbing | kWebkitGrabbing => ECursor::kGrabbing,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
// cpp: generated longhands Filter/BackdropFilter/ClipPath/ShapeOutside/Threshold
// Apply*; longhands_custom.cc:3490-3526 Cursor Apply*; converter.cc:352-396,
// 2750-2782. Resource-bearing branches require their actual native adapter.
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue()
        || v.IsUnsetValue() && !CSSProperty::Get(id).IsInherited()
        || inherit && parent.is_none();
    let inherited = inherit && !initial;
    if inherited && v.IsInheritedValue() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    if inherited
        && matches!(
            id,
            CSSPropertyID::kFilter
                | CSSPropertyID::kBackdropFilter
                | CSSPropertyID::kClipPath
                | CSSPropertyID::kShapeOutside
        )
        && b.EffectiveZoom() != parent.unwrap().EffectiveZoom()
    {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    match id {
        CSSPropertyID::kFilter | CSSPropertyID::kBackdropFilter => {
            let operations = if inherited {
                if id == CSSPropertyID::kFilter {
                    parent.unwrap().Filter().clone()
                } else {
                    parent.unwrap().BackdropFilter().clone()
                }
            } else if initial {
                FilterOperations::new()
            } else {
                Filter(id, v, b, root, media)?
            };
            if id == CSSPropertyID::kFilter {
                b.SetFilterOwned(operations);
            } else {
                b.SetBackdropFilterOwned(operations);
            }
        }
        CSSPropertyID::kCursor => {
            if inherited {
                b.SetCursor(parent.unwrap().Cursor());
                b.SetCursorList(parent.unwrap().Cursors());
                b.SetCursorIsInherited(true);
            } else {
                let cursor = if initial {
                    ECursor::kAuto
                } else {
                    CursorKeyword(id, v)?
                };
                b.ClearCursorList();
                b.SetCursor(cursor);
                b.SetCursorIsInherited(false);
            }
        }
        CSSPropertyID::kShapeImageThreshold => {
            let value = if inherited {
                parent.unwrap().ShapeImageThreshold()
            } else if initial {
                0.0
            } else {
                let (value, unit) = Number(id, v, b, root, media)?;
                (value
                    / if unit == UnitType::kPercentage {
                        100.0
                    } else {
                        1.0
                    }) as f32
            };
            b.SetShapeImageThreshold(value);
        }
        CSSPropertyID::kClipPath => {
            let operation = if inherited {
                parent.unwrap().ClipPath()
            } else if initial
                || matches!(v.Payload(),CSSValuePayload::kIdentifierClass(i)if i.0==CSSValueID::kNone)
            {
                None
            } else {
                if v.IsURIValue() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                let CSSValuePayload::kValueListClass(list) = v.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                let Some(first) = list.values.first() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                let pointer: *mut dyn ClipPathOperation = if first.IsBasicShapeValue() {
                    let geometry = if list.values.len() == 2 {
                        Geometry(id, &list.values[1])?
                    } else {
                        GeometryBox::kBorderBox
                    };
                    MakeGarbageCollected(unsafe {
                        ShapeClipPathOperation::new(Shape(id, first, b, root, media)?, geometry)
                    })
                } else {
                    MakeGarbageCollected(GeometryBoxClipPathOperation::new(Geometry(id, first)?))
                };
                Some(pointer)
            };
            b.SetClipPath(operation);
        }
        CSSPropertyID::kShapeOutside => {
            let value = if inherited {
                parent.unwrap().ShapeOutside()
            } else if initial
                || matches!(v.Payload(),CSSValuePayload::kIdentifierClass(i)if i.0==CSSValueID::kNone)
            {
                std::ptr::null_mut()
            } else {
                if v.IsImageValue() || v.IsImageGeneratorValue() || v.IsImageSetValue() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                let CSSValuePayload::kValueListClass(list) = v.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                let mut shape = None;
                let mut shape_box = ShapeBox::kMarginBox;
                for item in &list.values {
                    if item.IsBasicShapeValue() {
                        shape = Some(Shape(id, item, b, root, media)?);
                    } else {
                        shape_box = ShapeBoxValue(id, item)?;
                    }
                }
                MakeGarbageCollected(if let Some(shape) = shape {
                    unsafe { ShapeValue::from_shape(shape, shape_box) }
                } else {
                    ShapeValue::from_box(shape_box)
                })
            };
            b.SetShapeOutside(&Member::from_ptr(value));
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    Ok(())
}
