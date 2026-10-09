// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Generated text Apply* and StyleBuilderConverter native storage branches.
#![allow(non_snake_case)]
use super::*;
use foundation::{
    ETextAlign, ETextDecorationStyle, ETextTransform, TextDecorationLine, TextDecorationThickness,
    TextJustify,
};
use layoutng_style::style::{
    computed_style_constants::TextUnderlinePosition,
    text_indent_flags::TextIndentFlags,
    text_overflow_data::{TextOverflowData, TextOverflowType},
};

pub(super) fn IsTextProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kTextAlign
            | kTextTransform
            | kTextOverflow
            | kTextIndent
            | kTextDecorationLine
            | kTextDecorationThickness
            | kTextUnderlineOffset
            | kTextUnderlinePosition
            | kTextDecorationStyle
            | kTextDecorationColor
            | kTextJustify
    )
}

// style_builder_converter.h:490-505 ConvertFlags. None is a scalar; all
// other flags are identifier entries in a space-separated CSSValueList.
fn ConvertFlags(
    id: CSSPropertyID,
    v: &Value,
    zero: CSSValueID,
    mapping: impl Fn(CSSValueID) -> Option<u32>,
) -> std::result::Result<u32, LonghandApplicationError> {
    if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == zero) {
        return Ok(0);
    }
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != crate::production_css_value::ListSeparator::Space || list.values.is_empty()
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let mut flags = 0;
    for entry in &list.values {
        let CSSValuePayload::kIdentifierClass(keyword) = entry.Payload() else {
            return Err(LonghandApplicationError::InvalidValue(id));
        };
        flags |= mapping(keyword.0).ok_or(LonghandApplicationError::InvalidValue(id))?;
    }
    Ok(flags)
}

// CSSPrimitiveValue::ConvertToLength / ConvertLengthOrAuto. Preserve mixed
// length-percentage math as native calculated Length through the shared tree.
pub(super) fn ConvertLength(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<Length, LonghandApplicationError> {
    use crate::css_math_expression_node::CSSMathLengthResolver;
    if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == CSSValueID::kAuto) {
        return Ok(Length::Auto().clone());
    }
    let mut resolver = super::MathLengthResolver(
        id,
        b.GetFontDescription().ComputedSize(),
        root,
        b.EffectiveZoom(),
        media,
    );
    if let CSSValuePayload::kMathFunctionClass(math) = v.Payload() {
        return math
            .ConvertToLength(&mut resolver)
            .map_err(|_| LonghandApplicationError::Unsupported(id));
    }
    let CSSValuePayload::kNumericLiteralClass(number) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if number.GetType() == UnitType::kPercentage {
        return Ok(Length::Percent(number.DoubleValue()));
    }
    let pixels = resolver
        .ComputeLength(number.DoubleValue(), number.GetType())
        .map_err(|_| LonghandApplicationError::Unsupported(id))?;
    Ok(Length::Fixed(
        crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(pixels),
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
    use CSSValueID::*;
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue()
        || v.IsUnsetValue() && !CSSProperty::Get(id).IsInherited()
        || inherit && parent.is_none();
    if initial {
        // generated longhands.cc:16683-16685 only resets TextIndent's length.
        // The separate inherited TextIndentFlags field retains builder state.
        if id == kTextTransform {
            b.SetTextTransform(ComputedStyleInitialValues::InitialTextTransform());
            b.SetTextTransformIsInherited(false);
            return Ok(());
        }
        if id == kTextDecorationColor {
            // generated longhands.cc:16329-16331 uses CurrentColor directly.
            b.SetTextDecorationColor(&StyleColor::CurrentColor());
            return Ok(());
        }
        return ApplyInitial(id, b);
    }
    if inherit {
        let p = parent.unwrap();
        if matches!(
            id,
            kTextIndent | kTextDecorationThickness | kTextUnderlineOffset
        ) && p.EffectiveZoom() != b.EffectiveZoom()
        {
            // Generated ApplyParentValueIfZoomChanged requires the document's
            // standardized browser zoom policy and conversion-data adapter.
            return Err(LonghandApplicationError::Unsupported(id));
        }
        match id {
            kTextAlign => b.SetTextAlign(p.GetTextAlign()),
            kTextTransform => {
                b.SetTextTransform(p.TextTransform());
                b.SetTextTransformIsInherited(true);
            }
            kTextOverflow => b.SetTextOverflow(p.TextOverflow()),
            kTextIndent => b.SetTextIndent(p.TextIndent()),
            kTextDecorationLine => b.SetTextDecorationLine(p.GetTextDecorationLine()),
            kTextDecorationThickness => {
                b.SetTextDecorationThickness(p.GetTextDecorationThickness())
            }
            kTextUnderlineOffset => b.SetTextUnderlineOffset(p.TextUnderlineOffset()),
            kTextUnderlinePosition => b.SetTextUnderlinePosition(p.GetTextUnderlinePosition()),
            kTextDecorationStyle => b.SetTextDecorationStyle(p.TextDecorationStyle()),
            kTextDecorationColor => b.SetTextDecorationColor(p.TextDecorationColor()),
            kTextJustify => b.SetTextJustify(p.GetTextJustify()),
            _ => unreachable!(),
        }
        if v.IsInheritedValue() && !CSSProperty::Get(id).IsInherited() {
            b.SetHasExplicitInheritance();
            p.SetChildHasExplicitInheritance();
        }
        return Ok(());
    }
    match id {
        // longhands_custom.cc:9741-9777; css_value_id_mappings.h:64-73.
        kTextAlign => {
            let CSSValuePayload::kIdentifierClass(keyword) = v.Payload() else {
                return Err(LonghandApplicationError::InvalidValue(id));
            };
            let align = match keyword.0 {
                kMatchParent | kWebkitMatchParent => match parent {
                    // Tree style resolution supplies no parent for the document element.
                    None => ETextAlign::kStart,
                    Some(p) => match p.GetTextAlign() {
                        ETextAlign::kStart if p.IsLeftToRightDirection() => ETextAlign::kLeft,
                        ETextAlign::kStart => ETextAlign::kRight,
                        ETextAlign::kEnd if p.IsLeftToRightDirection() => ETextAlign::kRight,
                        ETextAlign::kEnd => ETextAlign::kLeft,
                        align => align,
                    },
                },
                kInternalCenter => parent
                    .filter(|p| p.GetTextAlign() != ComputedStyleInitialValues::InitialTextAlign())
                    .map_or(ETextAlign::kCenter, |p| p.GetTextAlign()),
                kWebkitAuto | kStart => ETextAlign::kStart,
                kEnd => ETextAlign::kEnd,
                CSSValueID::kLeft => ETextAlign::kLeft,
                CSSValueID::kRight => ETextAlign::kRight,
                kCenter => ETextAlign::kCenter,
                kJustify => ETextAlign::kJustify,
                kWebkitLeft => ETextAlign::kWebkitLeft,
                kWebkitRight => ETextAlign::kWebkitRight,
                kWebkitCenter => ETextAlign::kWebkitCenter,
                _ => return Err(LonghandApplicationError::InvalidValue(id)),
            };
            b.SetTextAlign(align);
        }
        // generated longhands.cc:16806-16817; css_value_id_mappings.h.
        kTextTransform => {
            let flags = ConvertFlags(id, v, kNone, |keyword| {
                Some(match keyword {
                    kCapitalize => ETextTransform::kCapitalize.bits(),
                    kUppercase => ETextTransform::kUppercase.bits(),
                    kLowercase => ETextTransform::kLowercase.bits(),
                    kFullWidth => ETextTransform::kFullWidth.bits(),
                    kFullSizeKana => ETextTransform::kFullSizeKana.bits(),
                    kMathAuto => ETextTransform::kMathAuto.bits(),
                    _ => return None,
                })
            })?;
            b.SetTextTransform(ETextTransform::from_bits(flags));
            b.SetTextTransformIsInherited(false);
        }
        // style_builder_converter.cc:4294-4308.
        kTextOverflow => {
            let overflow = match v.Payload() {
                CSSValuePayload::kStringClass(value) => {
                    TextOverflowData::from_string(value.0.clone())
                }
                CSSValuePayload::kIdentifierClass(value) => {
                    TextOverflowData::from_type(match value.0 {
                        CSSValueID::kClip => TextOverflowType::kClip,
                        kEllipsis => TextOverflowType::kEllipsis,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    })
                }
                _ => return Err(LonghandApplicationError::InvalidValue(id)),
            };
            b.SetTextOverflowOwned(overflow);
        }
        // longhands_custom.cc:10073-10097.
        kTextIndent => {
            let mut flags = TextIndentFlags::kDefault;
            let length = if let CSSValuePayload::kValueListClass(list) = v.Payload() {
                let mut length = None;
                for entry in &list.values {
                    if let CSSValuePayload::kIdentifierClass(keyword) = entry.Payload() {
                        flags |= match keyword.0 {
                            kHanging => TextIndentFlags::kHanging,
                            kEachLine => TextIndentFlags::kEachLine,
                            _ => return Err(LonghandApplicationError::InvalidValue(id)),
                        };
                    } else {
                        length = Some(ConvertLength(id, b, entry, root, media)?);
                    }
                }
                length.ok_or(LonghandApplicationError::InvalidValue(id))?
            } else {
                ConvertLength(id, b, v, root, media)?
            };
            b.SetTextIndentOwned(length);
            b.SetTextIndentFlags(flags);
        }
        kTextDecorationLine => {
            let flags = ConvertFlags(id, v, kNone, |keyword| {
                Some(match keyword {
                    kUnderline => TextDecorationLine::kUnderline.bits(),
                    kOverline => TextDecorationLine::kOverline.bits(),
                    kLineThrough => TextDecorationLine::kLineThrough.bits(),
                    kBlink => TextDecorationLine::kBlink.bits(),
                    kSpellingError => TextDecorationLine::kSpellingError.bits(),
                    kGrammarError => TextDecorationLine::kGrammarError.bits(),
                    _ => return None,
                })
            })?;
            b.SetTextDecorationLine(TextDecorationLine::from_bits(flags));
        }
        // style_builder_converter.cc:3165-3175,3275-3279.
        kTextDecorationThickness => {
            let thickness = if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == kFromFont)
            {
                TextDecorationThickness::from_keyword(kFromFont)
            } else {
                TextDecorationThickness::new(&ConvertLength(id, b, v, root, media)?)
            };
            b.SetTextDecorationThickness(&thickness);
        }
        kTextUnderlineOffset => {
            let length = ConvertLength(id, b, v, root, media)?;
            b.SetTextUnderlineOffsetOwned(length);
        }
        // style_builder_converter.cc:3255-3272.
        kTextUnderlinePosition => {
            let flags = ConvertFlags(id, v, kAuto, |keyword| {
                Some(match keyword {
                    kFromFont => TextUnderlinePosition::kFromFont.value(),
                    kUnder => TextUnderlinePosition::kUnder.value(),
                    CSSValueID::kLeft => TextUnderlinePosition::kLeft.value(),
                    CSSValueID::kRight => TextUnderlinePosition::kRight.value(),
                    _ => return None,
                })
            })?;
            b.SetTextUnderlinePosition(TextUnderlinePosition::from_bits(flags));
        }
        kTextDecorationStyle => {
            let CSSValuePayload::kIdentifierClass(value) = v.Payload() else {
                return Err(LonghandApplicationError::InvalidValue(id));
            };
            b.SetTextDecorationStyle(match value.0 {
                kSolid => ETextDecorationStyle::kSolid,
                kDouble => ETextDecorationStyle::kDouble,
                kDotted => ETextDecorationStyle::kDotted,
                kDashed => ETextDecorationStyle::kDashed,
                kWavy => ETextDecorationStyle::kWavy,
                _ => return Err(LonghandApplicationError::InvalidValue(id)),
            });
        }
        kTextDecorationColor => {
            let color = match v.Payload() {
                CSSValuePayload::kColorClass(color) => StyleColor::from_color(color.0),
                CSSValuePayload::kIdentifierClass(color) if color.0 == kCurrentcolor => {
                    StyleColor::CurrentColor()
                }
                CSSValuePayload::kIdentifierClass(color) => StyleColor::from_color(
                    crate::production_css_value::NamedColor(color.0)
                        .ok_or(LonghandApplicationError::Unsupported(id))?,
                ),
                _ => return Err(LonghandApplicationError::Unsupported(id)),
            };
            b.SetTextDecorationColor(&color);
        }
        kTextJustify => {
            let CSSValuePayload::kIdentifierClass(value) = v.Payload() else {
                return Err(LonghandApplicationError::InvalidValue(id));
            };
            b.SetTextJustify(match value.0 {
                kAuto => TextJustify::kAuto,
                kNone => TextJustify::kNone,
                kInterWord => TextJustify::kInterWord,
                kInterCharacter => TextJustify::kInterCharacter,
                _ => return Err(LonghandApplicationError::InvalidValue(id)),
            });
        }
        _ => unreachable!(),
    }
    Ok(())
}

#[cfg(test)]
mod text_properties_production_tests {
    use super::*;
    use crate::parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseDeclarationList, ParseProperty},
    };

    fn apply(b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, css: &str) -> Result {
        let parsed = ParseDeclarationList(
            &foundation::String::from(css),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.errors.is_empty(), "{:#?}", parsed.errors);
        for property in parsed.properties {
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

    #[test]
    fn text_properties_production_values_shorthand_and_native_math() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut b = ComputedStyleBuilder::from_style(initial);
        apply(&mut b, None, "text-transform:uppercase; text-overflow:ellipsis; text-indent:each-line calc(10% + 3px) hanging; text-decoration:underline overline wavy rgb(1, 2, 3) calc(2px + 1px); text-underline-offset:-10%; text-underline-position:right under; text-justify:distribute").unwrap();
        assert_eq!(b.TextTransform(), ETextTransform::kUppercase);
        assert!(!b.TextTransformIsInherited());
        assert!(b.TextOverflow().IsEllipsis());
        assert!(b.TextIndent().IsCalculated());
        let indent = b.TextIndent().GetPixelsAndPercent();
        assert_eq!((indent.pixels, indent.percent), (3.0, 10.0));
        assert_eq!(
            b.GetTextIndentFlags(),
            TextIndentFlags::kHanging | TextIndentFlags::kEachLine
        );
        assert_eq!(
            b.GetTextDecorationLine(),
            TextDecorationLine::kUnderline | TextDecorationLine::kOverline
        );
        assert_eq!(b.TextDecorationStyle(), ETextDecorationStyle::kWavy);
        assert_eq!(
            *b.GetTextDecorationThickness().Thickness(),
            Length::Fixed(3)
        );
        assert!(
            *b.TextDecorationColor() == StyleColor::from_color(foundation::Color::FromRGB(1, 2, 3))
        );
        assert_eq!(*b.TextUnderlineOffset(), Length::Percent(-10));
        assert_eq!(
            b.GetTextUnderlinePosition(),
            TextUnderlinePosition::kUnder | TextUnderlinePosition::kRight
        );
        assert_eq!(b.GetTextJustify(), TextJustify::kInterCharacter);
        apply(
            &mut b,
            None,
            "text-decoration:line-through; text-indent:0; text-decoration-thickness:from-font",
        )
        .unwrap();
        assert_eq!(b.GetTextDecorationLine(), TextDecorationLine::kLineThrough);
        assert_eq!(b.TextDecorationStyle(), ETextDecorationStyle::kSolid);
        assert!(b.TextDecorationColor().IsCurrentColor());
        assert!(b.GetTextDecorationThickness().IsFromFont());
        assert_eq!(b.GetTextIndentFlags(), TextIndentFlags::kDefault);
    }

    #[test]
    fn text_properties_production_initial_inherit_unset_and_zoom_boundary() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut parent = ComputedStyleBuilder::from_style(initial);
        apply(&mut parent, None, "text-transform:lowercase; text-overflow:ellipsis; text-indent:5px hanging; text-decoration:overline dotted red 25%; text-underline-offset:2px; text-underline-position:from-font left; text-justify:inter-word").unwrap();
        let p = unsafe { &*parent.TakeStyle() };
        let mut b = ComputedStyleBuilder::from_style(p);
        apply(&mut b, Some(p), "text-transform:unset; text-overflow:inherit; text-indent:inherit; text-decoration:inherit; text-underline-offset:inherit; text-underline-position:inherit; text-justify:inherit").unwrap();
        assert!(b.TextTransformIsInherited());
        assert_eq!(b.TextTransform(), p.TextTransform());
        assert_eq!(b.TextOverflow(), p.TextOverflow());
        assert_eq!(b.TextIndent(), p.TextIndent());
        assert_eq!(b.GetTextIndentFlags(), p.GetTextIndentFlags());
        assert_eq!(
            b.GetTextDecorationThickness(),
            p.GetTextDecorationThickness()
        );
        assert_eq!(b.GetTextDecorationLine(), p.GetTextDecorationLine());
        assert_eq!(b.TextDecorationStyle(), p.TextDecorationStyle());
        assert!(b.TextDecorationColor() == p.TextDecorationColor());
        assert_eq!(b.TextUnderlineOffset(), p.TextUnderlineOffset());
        assert_eq!(b.GetTextUnderlinePosition(), p.GetTextUnderlinePosition());
        assert_eq!(b.GetTextJustify(), p.GetTextJustify());
        assert!(b.HasExplicitInheritance());
        apply(&mut b, Some(p), "text-transform:initial; text-overflow:unset; text-indent:initial; text-decoration:initial; text-underline-offset:initial; text-underline-position:initial; text-justify:initial").unwrap();
        assert_eq!(b.TextTransform(), ETextTransform::kNone);
        assert!(!b.TextTransformIsInherited());
        assert!(b.TextOverflow().IsClip());
        assert_eq!(*b.TextIndent(), Length::Fixed(0));
        // Match generated initial: the flags field is independently inherited.
        assert_eq!(b.GetTextIndentFlags(), TextIndentFlags::kHanging);
        assert_eq!(b.GetTextDecorationLine(), TextDecorationLine::kNone);
        assert_eq!(b.TextDecorationStyle(), ETextDecorationStyle::kSolid);
        assert!(b.TextDecorationColor().IsCurrentColor());
        assert!(b.GetTextDecorationThickness().IsAuto());
        assert!(b.TextUnderlineOffset().IsAuto());
        assert_eq!(b.GetTextUnderlinePosition(), TextUnderlinePosition::kAuto);
        assert_eq!(b.GetTextJustify(), TextJustify::kAuto);
        b.SetEffectiveZoom(2.0);
        assert_eq!(
            apply(&mut b, Some(p), "text-indent:inherit"),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kTextIndent
            ))
        );
        assert_eq!(*b.TextIndent(), Length::Fixed(0));
    }

    #[test]
    fn text_properties_production_reject_invalid_and_disabled_runtime_branches() {
        use CSSPropertyID::*;
        for (id, input) in [
            (kTextTransform, "uppercase lowercase"),
            (kTextTransform, "full-width"),
            (kTextTransform, "full-size-kana"),
            (kTextTransform, "none uppercase"),
            (kTextOverflow, "\"...\""),
            (kTextOverflow, "clip ellipsis"),
            (kTextIndent, "hanging"),
            (kTextIndent, "1px hanging hanging"),
            (kTextIndent, "2"),
            (kTextDecorationLine, "underline underline"),
            (kTextDecorationLine, "spelling-error underline"),
            (kTextDecoration, "solid dashed"),
            (kTextUnderlinePosition, "auto under"),
            (kTextUnderlinePosition, "left right"),
            (kTextDecorationThickness, "2"),
            (kTextUnderlineOffset, "none"),
            (kTextJustify, "invalid"),
        ] {
            assert!(
                ParseProperty(
                    id,
                    &foundation::String::from(input),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .is_err(),
                "{id:?}: {input}"
            );
        }
        assert!(ParseProperty(
            kTextIndent,
            &foundation::String::from("2 hanging"),
            false,
            CSSParserMode::kHTMLQuirksMode
        )
        .is_ok());
        assert!(ParseProperty(
            kTextDecorationLine,
            &foundation::String::from("grammar-error"),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .is_ok());
        assert!(ParseProperty(
            kTextTransform,
            &foundation::String::from("math-auto"),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .is_ok());
    }
}
