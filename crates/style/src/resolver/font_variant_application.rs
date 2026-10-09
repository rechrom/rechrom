// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! FontBuilder converters applied to the existing native FontDescription.
#![allow(non_snake_case)]
use super::*;
use font_engine::fonts::{
    font_description::*, font_size_adjust::*, font_variant_alternates::FontVariantAlternates,
    font_variant_east_asian::*, font_variant_numeric::*,
};
use CSSPropertyID::*;
use CSSValueID::*;

pub(super) fn IsFontProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        kFontVariantCaps
            | kFontVariantLigatures
            | kFontVariantNumeric
            | kFontVariantEastAsian
            | kFontVariantAlternates
            | kFontVariantPosition
            | kFontVariantEmoji
            | kFontLanguageOverride
            | kFontSizeAdjust
    )
}
fn Identifier(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<CSSValueID, LonghandApplicationError> {
    if let CSSValuePayload::kIdentifierClass(v) = value.Payload() {
        Ok(v.0)
    } else {
        Err(LonghandApplicationError::InvalidValue(id))
    }
}
fn Identifiers(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<Vec<CSSValueID>, LonghandApplicationError> {
    if let CSSValuePayload::kValueListClass(list) = value.Payload() {
        list.values.iter().map(|v| Identifier(id, v)).collect()
    } else if Identifier(id, value)? == kNormal {
        Ok(Vec::new())
    } else {
        Err(LonghandApplicationError::InvalidValue(id))
    }
}
// cpp: style_builder_converter.cc:981-1028,1185-1432,745-758,606-644;
// generated longhands.cc Apply* exact ranges: 416-424,544-552,724-732,
// 753-761,782-790,811-819,840-848,869-877,898-906; each covers 6 effective
// lines, all mapped. LineHeight:10125-10138 covers 9 effective lines, mapped=6,
// remaining=3 (10129-10131 standardized-zoom reapplication).
// font_builder.cc UpdateFontDescription uses the same setters/initial values.
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    inherit: bool,
    initial: bool,
) -> Result {
    let mut d = b.GetFontDescription().clone();
    let p = parent.map(ComputedStyle::GetFontDescription);
    let inherited = inherit && !initial;
    match id {
        kFontVariantCaps => {
            let value = if inherited {
                p.unwrap().VariantCaps()
            } else if initial {
                FontVariantCaps::kCapsNormal
            } else {
                match Identifier(id, v)? {
                    kNormal => FontVariantCaps::kCapsNormal,
                    kSmallCaps => FontVariantCaps::kSmallCaps,
                    kAllSmallCaps => FontVariantCaps::kAllSmallCaps,
                    kPetiteCaps => FontVariantCaps::kPetiteCaps,
                    kAllPetiteCaps => FontVariantCaps::kAllPetiteCaps,
                    kUnicase => FontVariantCaps::kUnicase,
                    kTitlingCaps => FontVariantCaps::kTitlingCaps,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            d.SetVariantCaps(value);
        }
        kFontVariantLigatures => {
            let mut value = if inherited {
                p.unwrap().GetVariantLigatures()
            } else {
                VariantLigatures::default()
            };
            if !inherited && !initial {
                if let CSSValuePayload::kIdentifierClass(i) = v.Payload() {
                    value = match i.0 {
                        kNone => VariantLigatures::new(LigaturesState::kDisabledLigaturesState),
                        kNormal => value,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    };
                } else {
                    use LigaturesState::*;
                    for item in Identifiers(id, v)? {
                        match item {
                            kCommonLigatures => value.common = kEnabledLigaturesState,
                            kNoCommonLigatures => value.common = kDisabledLigaturesState,
                            kDiscretionaryLigatures => value.discretionary = kEnabledLigaturesState,
                            kNoDiscretionaryLigatures => {
                                value.discretionary = kDisabledLigaturesState
                            }
                            kHistoricalLigatures => value.historical = kEnabledLigaturesState,
                            kNoHistoricalLigatures => value.historical = kDisabledLigaturesState,
                            kContextual => value.contextual = kEnabledLigaturesState,
                            kNoContextual => value.contextual = kDisabledLigaturesState,
                            _ => return Err(LonghandApplicationError::InvalidValue(id)),
                        }
                    }
                }
            }
            d.SetVariantLigatures(&value);
        }
        kFontVariantNumeric => {
            let mut value = if inherited {
                p.unwrap().VariantNumeric()
            } else {
                FontVariantNumeric::default()
            };
            if !inherited && !initial {
                for item in Identifiers(id, v)? {
                    match item {
                        kLiningNums => value.SetNumericFigure(NumericFigure::kLiningNums),
                        kOldstyleNums => value.SetNumericFigure(NumericFigure::kOldstyleNums),
                        kProportionalNums => {
                            value.SetNumericSpacing(NumericSpacing::kProportionalNums)
                        }
                        kTabularNums => value.SetNumericSpacing(NumericSpacing::kTabularNums),
                        kDiagonalFractions => {
                            value.SetNumericFraction(NumericFraction::kDiagonalFractions)
                        }
                        kStackedFractions => {
                            value.SetNumericFraction(NumericFraction::kStackedFractions)
                        }
                        kOrdinal => value.SetOrdinal(Ordinal::kOrdinalOn),
                        kSlashedZero => value.SetSlashedZero(SlashedZero::kSlashedZeroOn),
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    }
                }
            }
            d.SetVariantNumeric(&value);
        }
        kFontVariantEastAsian => {
            let mut value = if inherited {
                p.unwrap().VariantEastAsian()
            } else {
                FontVariantEastAsian::default()
            };
            if !inherited && !initial {
                for item in Identifiers(id, v)? {
                    match item {
                        kJis78 => value.SetForm(EastAsianForm::kJis78),
                        kJis83 => value.SetForm(EastAsianForm::kJis83),
                        kJis90 => value.SetForm(EastAsianForm::kJis90),
                        kJis04 => value.SetForm(EastAsianForm::kJis04),
                        kSimplified => value.SetForm(EastAsianForm::kSimplified),
                        kTraditional => value.SetForm(EastAsianForm::kTraditional),
                        kFullWidth => value.SetWidth(EastAsianWidth::kFullWidth),
                        kProportionalWidth => value.SetWidth(EastAsianWidth::kProportionalWidth),
                        kRuby => value.SetRuby(true),
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    }
                }
            }
            d.SetVariantEastAsian(value);
        }
        kFontVariantAlternates => {
            let value = if inherited {
                p.unwrap().FontVariantAlternatesValue().cloned()
            } else if initial
                || matches!(v.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == kNormal)
            {
                None
            } else {
                let CSSValuePayload::kValueListClass(list) = v.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                let mut result = FontVariantAlternates::Create();
                for item in &list.values {
                    match item.Payload() {
                        CSSValuePayload::kIdentifierClass(i) if i.0 == kHistoricalForms => {
                            result.SetHistoricalForms()
                        }
                        CSSValuePayload::kAlternateClass(alternate) => {
                            let names = alternate
                                .aliases
                                .values
                                .iter()
                                .map(|v| {
                                    if let CSSValuePayload::kCustomIdentClass(i) = v.Payload() {
                                        Ok(i.name.clone())
                                    } else {
                                        Err(LonghandApplicationError::InvalidValue(id))
                                    }
                                })
                                .collect::<std::result::Result<Vec<_>, _>>()?;
                            if names.is_empty() {
                                return Err(LonghandApplicationError::InvalidValue(id));
                            }
                            match alternate.function.function_id {
                                kStylistic => result.SetStylistic(names[0].clone()),
                                kSwash => result.SetSwash(names[0].clone()),
                                kOrnaments => result.SetOrnaments(names[0].clone()),
                                kAnnotation => result.SetAnnotation(names[0].clone()),
                                kStyleset => result.SetStyleset(names),
                                kCharacterVariant => result.SetCharacterVariant(names),
                                _ => return Err(LonghandApplicationError::InvalidValue(id)),
                            }
                        }
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    }
                }
                (!result.IsNormal()).then(|| std::sync::Arc::new(result))
            };
            d.SetFontVariantAlternates(value);
        }
        kFontVariantPosition => {
            let value = if inherited {
                p.unwrap().VariantPosition()
            } else if initial {
                FontVariantPosition::kNormalVariantPosition
            } else {
                match Identifier(id, v)? {
                    kNormal => FontVariantPosition::kNormalVariantPosition,
                    kSub => FontVariantPosition::kSubVariantPosition,
                    kSuper => FontVariantPosition::kSuperVariantPosition,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            d.SetVariantPosition(value);
        }
        kFontVariantEmoji => {
            use font_engine::fonts::font_variant_emoji::FontVariantEmoji::*;
            let value = if inherited {
                p.unwrap().VariantEmoji()
            } else if initial {
                kNormalVariantEmoji
            } else {
                match Identifier(id, v)? {
                    kNormal => kNormalVariantEmoji,
                    kText => kTextVariantEmoji,
                    kEmoji => kEmojiVariantEmoji,
                    kUnicode => kUnicodeVariantEmoji,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            d.SetVariantEmoji(value);
        }
        kFontLanguageOverride => {
            let value = if inherited {
                p.unwrap().FontLanguageOverride().clone()
            } else if initial {
                foundation::AtomicString::default()
            } else {
                match v.Payload() {
                    CSSValuePayload::kIdentifierClass(i) if i.0 == kNormal => {
                        foundation::AtomicString::default()
                    }
                    CSSValuePayload::kStringClass(s) => {
                        foundation::AtomicString::from_str(&s.0.Utf8())
                    }
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            d.SetFontLanguageOverride(value);
        }
        kFontSizeAdjust => {
            let value = if inherited {
                p.unwrap().SizeAdjust()
            } else if initial {
                FontSizeAdjust::default()
            } else {
                let (metric, v) = if let CSSValuePayload::kValuePairClass(pair) = v.Payload() {
                    (
                        match Identifier(id, &pair.first)? {
                            kExHeight => Metric::kExHeight,
                            kCapHeight => Metric::kCapHeight,
                            kChWidth => Metric::kChWidth,
                            kIcWidth => Metric::kIcWidth,
                            kIcHeight => Metric::kIcHeight,
                            _ => return Err(LonghandApplicationError::InvalidValue(id)),
                        },
                        pair.second.as_ref(),
                    )
                } else {
                    (Metric::kExHeight, v)
                };
                match v.Payload() {
                    CSSValuePayload::kIdentifierClass(i) if i.0 == kNone => {
                        FontSizeAdjust::default()
                    }
                    CSSValuePayload::kIdentifierClass(i) if i.0 == kFromFont => {
                        FontSizeAdjust::with_metric_and_type(
                            FontSizeAdjust::kFontSizeAdjustNone,
                            metric,
                            ValueType::kFromFont,
                        )
                    }
                    CSSValuePayload::kNumericLiteralClass(n)
                        if n.GetType() == UnitType::kNumber =>
                    {
                        FontSizeAdjust::with_metric(n.DoubleValue() as f32, metric)
                    }
                    CSSValuePayload::kMathFunctionClass(m) => {
                        if m.Category()
                            != crate::css_math_expression_node::CalculationResultCategory::Number
                        {
                            return Err(LonghandApplicationError::InvalidValue(id));
                        }
                        let value = m.ComputeValue(&mut |_, _| Err(crate::css_math_expression_node::MathError::MissingLengthContext), None)
                            .map_err(|_| LonghandApplicationError::Unsupported(id))?;
                        FontSizeAdjust::with_metric(value as f32, metric)
                    }
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            d.SetSizeAdjust(&value);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    StageFontDescription(b, &d);
    Ok(())
}

// cpp: style_builder_converter.cc:2200-2239; generated LineHeight Apply*.
// Stable scalar and math branches. Real font/container length metrics and text
// zoom remain collaborators of MathLengthResolver/Pixels, never approximated.
pub(super) fn ApplyLineHeight(
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    let id = kLineHeight;
    let inherit = v.IsInheritedValue() || v.IsUnsetValue();
    let initial = v.IsInitialValue() || inherit && parent.is_none();
    if inherit && !initial && v.IsInheritedValue() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    if inherit && !initial && b.EffectiveZoom() != parent.unwrap().EffectiveZoom() {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    let length = if initial {
        ComputedStyleInitialValues::InitialLineHeight()
    } else if inherit {
        parent.unwrap().LineHeight().clone()
    } else {
        let font = b.GetFontDescription().ComputedSize();
        let scalar = |number: f64, unit| -> std::result::Result<Length, LonghandApplicationError> {
            Ok(match unit {
                UnitType::kNumber | UnitType::kInteger => Length::Percent(
                    (number * 100.0).clamp(-(f32::MAX as f64), f32::MAX as f64) as f32,
                ),
                UnitType::kPercentage => Length::Fixed(
                    font * (number.clamp(i32::MIN as f64, i32::MAX as f64) as i32) as f32 / 100.0,
                ),
                _ => {
                    if b.EffectiveZoom() != 1.0 {
                        return Err(LonghandApplicationError::Unsupported(id));
                    }
                    Length::Fixed(
                        crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(Pixels(
                            id, number, unit, font, root, media,
                        )?) as f32,
                    )
                }
            })
        };
        match v.Payload() {
            CSSValuePayload::kIdentifierClass(i) if i.0 == kNormal => {
                ComputedStyleInitialValues::InitialLineHeight()
            }
            CSSValuePayload::kNumericLiteralClass(n) => scalar(n.DoubleValue(), n.GetType())?,
            CSSValuePayload::kMathFunctionClass(m) => {
                use crate::css_math_expression_node::CalculationResultCategory as C;
                let number = m
                    .ComputeValue(
                        &mut MathLengthResolver(id, font, root, b.EffectiveZoom(), media),
                        Some(font as f64),
                    )
                    .map_err(|_| LonghandApplicationError::Unsupported(id))?;
                if m.Category() == C::LengthFunction {
                    Length::Fixed(number as f32)
                } else {
                    scalar(
                        number,
                        m.expression
                            .CanonicalUnit()
                            .ok_or(LonghandApplicationError::InvalidValue(id))?,
                    )?
                }
            }
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        }
    };
    b.SetLineHeight(&length);
    Ok(())
}
