// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native AspectRatio, columns, containment and fragmentation Apply* branches.
#![allow(non_snake_case)]
use super::*;
use foundation::{EAspectRatioType, StyleAspectRatio};
use layoutng_style::style::{
    computed_style_constants::Containment,
    style_intrinsic_length::{StyleIntrinsicLength, StyleIntrinsicLengthOptions},
};

pub(super) fn IsLayoutProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kAspectRatio
            | kColumnCount
            | kColumnWidth
            | kColumnHeight
            | kContain
            | kContainIntrinsicWidth
            | kContainIntrinsicHeight
            | kOrphans
            | kWidows
            | kColumnGap
            | kRowGap
            | kOverflowClipMargin
    )
}

// CSSPrimitiveValue::ComputeNumber / ConvertTo<T> share the existing CSSMath
// expression resolver. Missing relative-unit context remains typed Unsupported.
fn Number(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<f64, LonghandApplicationError> {
    match v.Payload() {
        CSSValuePayload::kNumericLiteralClass(n)
            if matches!(n.GetType(), UnitType::kNumber | UnitType::kInteger) =>
        {
            Ok(n.DoubleValue())
        }
        CSSValuePayload::kMathFunctionClass(math)
            if math.Category()
                == crate::css_math_expression_node::CalculationResultCategory::Number =>
        {
            math.ComputeValue(
                &mut super::MathLengthResolver(
                    id,
                    b.GetFontDescription().ComputedSize(),
                    root,
                    b.EffectiveZoom(),
                    media,
                ),
                None,
            )
            .map_err(|_| LonghandApplicationError::Unsupported(id))
        }
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}

fn Pixels(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<f64, LonghandApplicationError> {
    use crate::css_math_expression_node::{CSSMathLengthResolver, CalculationResultCategory};
    let mut resolver = super::MathLengthResolver(
        id,
        b.GetFontDescription().ComputedSize(),
        root,
        b.EffectiveZoom(),
        media,
    );
    let pixels = match v.Payload() {
        CSSValuePayload::kNumericLiteralClass(n)
            if crate::css_numeric_literal_value::IsLength(n.GetType()) =>
        {
            resolver.ComputeLength(n.DoubleValue(), n.GetType())
        }
        CSSValuePayload::kMathFunctionClass(math)
            if math.Category() == CalculationResultCategory::Length =>
        {
            math.ComputeValue(&mut resolver, None)
        }
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    }
    .map_err(|_| LonghandApplicationError::Unsupported(id))?;
    if pixels < 0.0 {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    Ok(crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(pixels))
}

fn Intrinsic(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<StyleIntrinsicLength, LonghandApplicationError> {
    // cpp: style_builder_converter.cc:3839-3856 ConvertIntrinsicDimension;
    // ConvertLengthOrNone uses ConvertLength for the non-identifier alternative.
    let (has_auto, value) = if let CSSValuePayload::kValueListClass(list) = v.Payload() {
        if list.values.len() != 2 || Identifier(id, &list.values[0])? != CSSValueID::kAuto {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
        (true, list.values[1].as_ref())
    } else {
        (false, v)
    };
    let length = if let CSSValuePayload::kIdentifierClass(ident) = value.Payload() {
        if ident.0 != CSSValueID::kNone {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
        None
    } else {
        Some(Length::Fixed(
            // cpp: css_primitive_value.cc:43-66,328-332 ClampToCSSLengthRange.
            Pixels(id, b, value, root, media)?.clamp(
                (foundation::LayoutUnit::Min().ToInt() + 2) as f64,
                (foundation::LayoutUnit::Max().ToInt() - 2) as f64,
            ),
        ))
    };
    Ok(StyleIntrinsicLength::new(
        &length,
        StyleIntrinsicLengthOptions { has_auto },
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
    use CSSPropertyID::*;
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue()
        || v.IsUnsetValue() && !CSSProperty::Get(id).IsInherited()
        || inherit && parent.is_none();
    match id {
        // generated longhands.cc:6163-6175,13770-13782;
        // converter.cc:2051-2061 ConvertGapLength.
        kColumnGap | kRowGap => {
            let row = id == kRowGap;
            let gap = if initial {
                None
            } else if inherit {
                let p = parent.unwrap();
                if p.EffectiveZoom() != b.EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                if row {
                    p.RowGap().clone()
                } else {
                    p.ColumnGap().clone()
                }
            } else if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kNormal)
            {
                None
            } else {
                Some(text_application::ConvertLength(id, b, v, root, media)?)
            };
            if row {
                b.SetRowGap(&gap);
            } else {
                b.SetColumnGap(&gap);
            }
        }
        // generated longhands.cc:12261-12273; converter.cc:3898-3943.
        kOverflowClipMargin => {
            use layoutng_style::style::style_overflow_clip_margin::{
                ReferenceBox, StyleOverflowClipMargin,
            };
            let margin = if initial {
                ComputedStyleInitialValues::InitialOverflowClipMargin()
            } else if inherit {
                let p = parent.unwrap();
                if p.EffectiveZoom() != b.EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                *p.OverflowClipMargin()
            } else {
                let CSSValuePayload::kValueListClass(list) = v.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                if !(1..=2).contains(&list.values.len()) {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                let mut reference = ReferenceBox::kPaddingBox;
                let mut length = None;
                for v in &list.values {
                    if let CSSValuePayload::kIdentifierClass(k) = v.Payload() {
                        reference = match k.0 {
                            CSSValueID::kContentBox => ReferenceBox::kContentBox,
                            CSSValueID::kPaddingBox => ReferenceBox::kPaddingBox,
                            CSSValueID::kBorderBox => ReferenceBox::kBorderBox,
                            _ => return Err(LonghandApplicationError::InvalidValue(id)),
                        };
                    } else if length.is_none() {
                        let l = text_application::ConvertLength(id, b, v, root, media)?;
                        if !l.IsFixed() {
                            return Err(LonghandApplicationError::InvalidValue(id));
                        }
                        length = Some(l.Pixels());
                    } else {
                        return Err(LonghandApplicationError::InvalidValue(id));
                    }
                }
                Some(StyleOverflowClipMargin::new(
                    reference,
                    foundation::LayoutUnit::from_f64(length.unwrap_or(0.) as f64),
                ))
            };
            b.SetOverflowClipMargin(&margin);
        }
        // cpp: generated longhands.cc:2967-2975; converter.cc:3697-3743.
        kAspectRatio => {
            let ratio = if initial {
                ComputedStyleInitialValues::InitialAspectRatio()
            } else if inherit {
                *parent.unwrap().AspectRatio()
            } else if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == CSSValueID::kAuto)
            {
                StyleAspectRatio::new(EAspectRatioType::kAuto, foundation::gfx::SizeF::default())
            } else {
                let CSSValuePayload::kValueListClass(list) = v.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                if !(1..=2).contains(&list.values.len()) {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                let mut ratio = None;
                let mut auto = false;
                for item in &list.values {
                    match item.Payload() {
                        CSSValuePayload::kRatioClass(value) if ratio.is_none() => {
                            ratio = Some(foundation::gfx::SizeF::new(
                                Number(id, b, &value.first, root, media)? as f32,
                                Number(id, b, &value.second, root, media)? as f32,
                            ));
                        }
                        CSSValuePayload::kIdentifierClass(i)
                            if i.0 == CSSValueID::kAuto && !auto =>
                        {
                            auto = true
                        }
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    }
                }
                StyleAspectRatio::new(
                    if auto {
                        EAspectRatioType::kAutoAndRatio
                    } else {
                        EAspectRatioType::kRatio
                    },
                    ratio.ok_or(LonghandApplicationError::InvalidValue(id))?,
                )
            };
            b.SetAspectRatioOwned(ratio);
        }
        // cpp: generated longhands.cc:6092-6107; CSSPrimitiveValue::ConvertTo<unsigned short>.
        kColumnCount => {
            let auto = initial
                || inherit && parent.unwrap().HasAutoColumnCount()
                || !inherit
                    && matches!(v.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == CSSValueID::kAuto);
            if auto {
                b.SetHasAutoColumnCount();
            } else {
                let number = if inherit {
                    parent.unwrap().ColumnCount()
                } else {
                    Number(id, b, v, root, media)?.clamp(0.0, u16::MAX as f64) as u16
                };
                b.SetColumnCount(number);
            }
        }
        // cpp: generated longhands.cc:6203-6224,6554-6574.
        kColumnWidth | kColumnHeight => {
            if inherit && !initial && parent.unwrap().EffectiveZoom() != b.EffectiveZoom() {
                return Err(LonghandApplicationError::Unsupported(id));
            }
            let width = id == kColumnWidth;
            let auto = initial
                || inherit
                    && if width {
                        parent.unwrap().HasAutoColumnWidth()
                    } else {
                        parent.unwrap().HasAutoColumnHeight()
                    }
                || !inherit
                    && matches!(v.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == CSSValueID::kAuto);
            if auto {
                if width {
                    b.SetHasAutoColumnWidth();
                } else {
                    b.SetHasAutoColumnHeight();
                }
            } else {
                let pixels = if inherit {
                    if width {
                        parent.unwrap().ColumnWidth()
                    } else {
                        parent.unwrap().ColumnHeight()
                    }
                } else {
                    Pixels(id, b, v, root, media)?.clamp(0.0, f32::MAX as f64) as f32
                };
                if width {
                    b.SetColumnWidth(pixels);
                } else {
                    b.SetColumnHeight(pixels);
                }
            }
        }
        // cpp: generated longhands.cc:6624-6632; converter.h:489-502 ConvertFlags.
        kContain => {
            let flags = if initial {
                ComputedStyleInitialValues::InitialContain()
            } else if inherit {
                parent.unwrap().Contain()
            } else if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == CSSValueID::kNone)
            {
                0
            } else {
                let CSSValuePayload::kValueListClass(list) = v.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                let mut flags = 0;
                for item in &list.values {
                    let flag = match Identifier(id, item)? {
                        CSSValueID::kStrict => Containment::kContainsStrict,
                        CSSValueID::kContent => Containment::kContainsContent,
                        CSSValueID::kSize => Containment::kContainsSize,
                        CSSValueID::kInlineSize => Containment::kContainsInlineSize,
                        CSSValueID::kLayout => Containment::kContainsLayout,
                        CSSValueID::kStyle => Containment::kContainsStyle,
                        CSSValueID::kPaint => Containment::kContainsPaint,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    };
                    flags |= flag.value() as u32;
                }
                flags
            };
            b.SetContain(flags);
        }
        // cpp: generated longhands.cc:6700-6713,6781-6794.
        kContainIntrinsicWidth | kContainIntrinsicHeight => {
            let width = id == kContainIntrinsicWidth;
            let length = if initial {
                if width {
                    ComputedStyleInitialValues::InitialContainIntrinsicWidth()
                } else {
                    ComputedStyleInitialValues::InitialContainIntrinsicHeight()
                }
            } else if inherit {
                let p = parent.unwrap();
                if p.EffectiveZoom() != b.EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                if width {
                    p.ContainIntrinsicWidth().clone()
                } else {
                    p.ContainIntrinsicHeight().clone()
                }
            } else {
                Intrinsic(id, b, v, root, media)?
            };
            if width {
                b.SetContainIntrinsicWidthOwned(length);
            } else {
                b.SetContainIntrinsicHeightOwned(length);
            }
        }
        // cpp: generated longhands.cc:12060-12068,19494-19502; ConvertTo<short>.
        kOrphans | kWidows => {
            let number = if initial {
                if id == kOrphans {
                    ComputedStyleInitialValues::InitialOrphans()
                } else {
                    ComputedStyleInitialValues::InitialWidows()
                }
            } else if inherit {
                if id == kOrphans {
                    parent.unwrap().Orphans()
                } else {
                    parent.unwrap().Widows()
                }
            } else {
                Number(id, b, v, root, media)?.clamp(i16::MIN as f64, i16::MAX as f64) as i16
            };
            if id == kOrphans {
                b.SetOrphans(number);
            } else {
                b.SetWidows(number);
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
