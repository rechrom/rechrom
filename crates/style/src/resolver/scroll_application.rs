// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Generated scroll Apply* plus the Chromium scrollbar/snap converters.
#![allow(non_snake_case)]
use super::*;
use foundation::{EOverscrollBehavior, MakeGarbageCollected, Member};
use layoutng_style::style::computed_style_constants::ScrollbarGutter;
use layoutng_style::style::{scroll_snap_data::cc, style_scrollbar_color::StyleScrollbarColor};

pub(super) fn IsScrollProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kScrollbarColor
            | kScrollbarGutter
            | kOverscrollBehaviorX
            | kOverscrollBehaviorY
            | kScrollSnapAlign
            | kScrollSnapType
            | kScrollMarginTop
            | kScrollMarginRight
            | kScrollMarginBottom
            | kScrollMarginLeft
            | kScrollMarginBlockStart
            | kScrollMarginBlockEnd
            | kScrollMarginInlineStart
            | kScrollMarginInlineEnd
            | kScrollPaddingTop
            | kScrollPaddingRight
            | kScrollPaddingBottom
            | kScrollPaddingLeft
            | kScrollPaddingBlockStart
            | kScrollPaddingBlockEnd
            | kScrollPaddingInlineStart
            | kScrollPaddingInlineEnd
    )
}
fn IsSide(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kScrollMarginTop
            | kScrollMarginRight
            | kScrollMarginBottom
            | kScrollMarginLeft
            | kScrollPaddingTop
            | kScrollPaddingRight
            | kScrollPaddingBottom
            | kScrollPaddingLeft
    )
}
fn Padding(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kScrollPaddingTop | kScrollPaddingRight | kScrollPaddingBottom | kScrollPaddingLeft
    )
}
// style_builder_converter.h:483-487 ConvertComputedLength<float>;
// style_builder_converter.cc:2063-2067,2111-2120 ConvertLength/ConvertLengthOrAuto.
fn ConvertLength(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    value: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<Length, LonghandApplicationError> {
    use crate::css_math_expression_node::{CSSMathLengthResolver, CalculationResultCategory as C};
    let mut resolver = super::MathLengthResolver(
        id,
        b.GetFontDescription().ComputedSize(),
        root,
        b.EffectiveZoom(),
        media,
    );
    let padding = Padding(id);
    if let CSSValuePayload::kMathFunctionClass(math) = value.Payload() {
        if !padding && math.Category() == C::Number {
            let pixels = math
                .ComputeValue(&mut resolver, None)
                .map_err(|_| LonghandApplicationError::Unsupported(id))?
                * b.EffectiveZoom() as f64;
            return Ok(Length::Fixed(
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(pixels),
            ));
        }
        if !padding && math.Category() != C::Length {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
        return math
            .ConvertToLength(&mut resolver)
            .map_err(|_| LonghandApplicationError::Unsupported(id));
    }
    match value.Payload() {
        CSSValuePayload::kIdentifierClass(k) if padding && k.0 == CSSValueID::kAuto => {
            Ok(Length::Auto().clone())
        }
        CSSValuePayload::kNumericLiteralClass(n)
            if padding && n.GetType() == UnitType::kPercentage && n.DoubleValue() >= 0.0 =>
        {
            Ok(Length::Percent(n.DoubleValue()))
        }
        CSSValuePayload::kNumericLiteralClass(n)
            if (crate::css_numeric_literal_value::IsLength(n.GetType())
                || !padding && n.GetType() == UnitType::kUserUnits)
                && (!padding || n.DoubleValue() >= 0.0) =>
        {
            let pixels = resolver
                .ComputeLength(
                    n.DoubleValue(),
                    if n.GetType() == UnitType::kUserUnits {
                        UnitType::kPixels
                    } else {
                        n.GetType()
                    },
                )
                .map_err(|_| LonghandApplicationError::Unsupported(id))?;
            Ok(Length::Fixed(
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(pixels),
            ))
        }
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}
fn Color(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<StyleColor, LonghandApplicationError> {
    match value.Payload() {
        CSSValuePayload::kColorClass(color) => Ok(StyleColor::from_color(color.0)),
        CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kCurrentcolor => {
            Ok(StyleColor::CurrentColor())
        }
        CSSValuePayload::kIdentifierClass(k) => crate::production_css_value::NamedColor(k.0)
            .map(StyleColor::from_color)
            .ok_or(LonghandApplicationError::Unsupported(id)),
        _ => Err(LonghandApplicationError::Unsupported(id)),
    }
}
fn Alignment(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<cc::SnapAlignment, LonghandApplicationError> {
    use CSSValueID::*;
    Ok(match Identifier(id, value)? {
        kNone => cc::SnapAlignment::kNone,
        kStart => cc::SnapAlignment::kStart,
        kEnd => cc::SnapAlignment::kEnd,
        kCenter => cc::SnapAlignment::kCenter,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
fn Axis(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<cc::SnapAxis, LonghandApplicationError> {
    use CSSValueID::*;
    Ok(match Identifier(id, value)? {
        kX => cc::SnapAxis::kX,
        kY => cc::SnapAxis::kY,
        kBoth => cc::SnapAxis::kBoth,
        kBlock => cc::SnapAxis::kBlock,
        kInline => cc::SnapAxis::kInline,
        kPair => return Err(LonghandApplicationError::Unsupported(id)),
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
// Generated native fields: longhands.cc ApplyInitial/Inherit/Value.
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    value: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    let id = ResolvePhysical(id, b.GetWritingDirection());
    let inherit =
        value.IsInheritedValue() || value.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial =
        value.IsInitialValue() || value.IsUnsetValue() && !inherit || inherit && parent.is_none();
    use CSSPropertyID::*;
    if IsSide(id) {
        if inherit && !initial && b.EffectiveZoom() != parent.unwrap().EffectiveZoom() {
            return Err(LonghandApplicationError::Unsupported(id));
        }
        let inherited = parent.filter(|_| inherit && !initial);
        if Padding(id) {
            let length = if initial {
                Length::Auto().clone()
            } else if let Some(p) = inherited {
                match id {
                    kScrollPaddingTop => p.ScrollPaddingTop(),
                    kScrollPaddingRight => p.ScrollPaddingRight(),
                    kScrollPaddingBottom => p.ScrollPaddingBottom(),
                    _ => p.ScrollPaddingLeft(),
                }
                .clone()
            } else {
                ConvertLength(id, b, value, root, media)?
            };
            match id {
                kScrollPaddingTop => b.SetScrollPaddingTop(&length),
                kScrollPaddingRight => b.SetScrollPaddingRight(&length),
                kScrollPaddingBottom => b.SetScrollPaddingBottom(&length),
                _ => b.SetScrollPaddingLeft(&length),
            }
        } else {
            let pixels = if initial {
                0.0
            } else if let Some(p) = inherited {
                match id {
                    kScrollMarginTop => p.ScrollMarginTop(),
                    kScrollMarginRight => p.ScrollMarginRight(),
                    kScrollMarginBottom => p.ScrollMarginBottom(),
                    _ => p.ScrollMarginLeft(),
                }
            } else {
                ConvertLength(id, b, value, root, media)?.Pixels()
            };
            match id {
                kScrollMarginTop => b.SetScrollMarginTop(pixels),
                kScrollMarginRight => b.SetScrollMarginRight(pixels),
                kScrollMarginBottom => b.SetScrollMarginBottom(pixels),
                _ => b.SetScrollMarginLeft(pixels),
            }
        }
    } else {
        match id {
            // style_builder_converter.cc:3787-3802.
            kScrollbarColor => {
                let pointer = if initial {
                    std::ptr::null_mut()
                } else if inherit {
                    parent.unwrap().ScrollbarColor()
                } else if matches!(value.Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kAuto)
                {
                    std::ptr::null_mut()
                } else {
                    let CSSValuePayload::kValueListClass(list) = value.Payload() else {
                        return Err(LonghandApplicationError::InvalidValue(id));
                    };
                    if list.separator != crate::production_css_value::ListSeparator::Space
                        || !(1..=2).contains(&list.values.len())
                    {
                        return Err(LonghandApplicationError::InvalidValue(id));
                    }
                    let thumb = Color(id, &list.values[0])?;
                    let track = Color(id, list.values.last().unwrap())?;
                    MakeGarbageCollected(StyleScrollbarColor::new(thumb, track))
                };
                b.SetScrollbarColor(Member::from_ptr(pointer));
            }
            // style_builder_converter.cc:3804-3821.
            kScrollbarGutter => {
                let flags = if initial {
                    ScrollbarGutter::kScrollbarGutterAuto.value() as u32
                } else if inherit {
                    parent.unwrap().ScrollbarGutter()
                } else {
                    let mut flags = 0;
                    let mut process = |value: &Value| -> Result {
                        flags |= match Identifier(id, value)? {
                            CSSValueID::kAuto => {
                                ScrollbarGutter::kScrollbarGutterAuto.value() as u32
                            }
                            CSSValueID::kStable => {
                                ScrollbarGutter::kScrollbarGutterStable.value() as u32
                            }
                            CSSValueID::kBothEdges => {
                                ScrollbarGutter::kScrollbarGutterBothEdges.value() as u32
                            }
                            _ => return Err(LonghandApplicationError::InvalidValue(id)),
                        };
                        Ok(())
                    };
                    if let CSSValuePayload::kValueListClass(list) = value.Payload() {
                        for value in &list.values {
                            process(value)?;
                        }
                    } else {
                        process(value)?;
                    }
                    flags
                };
                b.SetScrollbarGutter(flags);
            }
            kOverscrollBehaviorX | kOverscrollBehaviorY => {
                let behavior = if initial {
                    EOverscrollBehavior::kAuto
                } else if inherit {
                    if id == kOverscrollBehaviorX {
                        parent.unwrap().OverscrollBehaviorX()
                    } else {
                        parent.unwrap().OverscrollBehaviorY()
                    }
                } else {
                    match Identifier(id, value)? {
                        CSSValueID::kAuto => EOverscrollBehavior::kAuto,
                        CSSValueID::kChain => EOverscrollBehavior::kChain,
                        CSSValueID::kContain => EOverscrollBehavior::kContain,
                        CSSValueID::kNone => EOverscrollBehavior::kNone,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    }
                };
                if id == kOverscrollBehaviorX {
                    b.SetOverscrollBehaviorX(behavior);
                } else {
                    b.SetOverscrollBehaviorY(behavior);
                }
            }
            // style_builder_converter.cc:3334-3350.
            kScrollSnapAlign => {
                let align = if initial {
                    cc::ScrollSnapAlign::default()
                } else if inherit {
                    *parent.unwrap().GetScrollSnapAlign()
                } else if let CSSValuePayload::kValuePairClass(pair) = value.Payload() {
                    cc::ScrollSnapAlign::new(
                        Alignment(id, &pair.first)?,
                        Alignment(id, &pair.second)?,
                    )
                } else {
                    cc::ScrollSnapAlign::from_alignment(Alignment(id, value)?)
                };
                b.SetScrollSnapAlign(&align);
            }
            // style_builder_converter.cc:3310-3332.
            kScrollSnapType => {
                let snap = if initial {
                    cc::ScrollSnapType::default()
                } else if inherit {
                    *parent.unwrap().GetScrollSnapType()
                } else if matches!(value.Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kNone)
                {
                    cc::ScrollSnapType::default()
                } else if let CSSValuePayload::kValuePairClass(pair) = value.Payload() {
                    let strictness = match Identifier(id, &pair.second)? {
                        CSSValueID::kMandatory => cc::SnapStrictness::kMandatory,
                        CSSValueID::kProximity => cc::SnapStrictness::kProximity,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    };
                    cc::ScrollSnapType::new(false, Axis(id, &pair.first)?, strictness)
                } else {
                    cc::ScrollSnapType::new(false, Axis(id, value)?, cc::SnapStrictness::kProximity)
                };
                b.SetScrollSnapType(&snap);
            }
            _ => return Err(LonghandApplicationError::Unsupported(id)),
        }
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}
