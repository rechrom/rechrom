// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Font palette ownership and two native internal booleans.
#![allow(non_snake_case)]
use super::*;
use font_engine::fonts::font_palette::{FontPalette, KeywordPaletteName as K};
// cpp: generated longhands.cc:480-488,9425-9433,9457-9465;
// style_builder_converter.cc:818-841,3745-3759.
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
) -> Result {
    use CSSPropertyID::*;
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue() || v.IsUnsetValue() && !inherit || inherit && parent.is_none();
    let inherited = parent.filter(|_| inherit && !initial);
    match id {
        kFontPalette => {
            let palette = if initial {
                None
            } else if let Some(p) = inherited {
                p.GetFontDescription().FontPaletteValue()
            } else {
                match v.Payload() {
                    CSSValuePayload::kIdentifierClass(i) => match i.0 {
                        CSSValueID::kNormal => None,
                        CSSValueID::kLight => Some(FontPalette::CreateKeyword(K::kLightPalette)),
                        CSSValueID::kDark => Some(FontPalette::CreateKeyword(K::kDarkPalette)),
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    },
                    CSSValuePayload::kCustomIdentClass(i)
                        if i.property == CSSPropertyID::kInvalid =>
                    {
                        Some(FontPalette::CreateCustom(i.name.clone()))
                    }
                    _ => return Err(LonghandApplicationError::Unsupported(id)),
                }
            };
            let mut d = b.GetFontDescription().clone();
            d.SetFontPalette(palette);
            StageFontDescription(b, &d);
        }
        kInternalAlignContentBlock => {
            let value = if initial {
                ComputedStyleInitialValues::InitialAlignContentBlockCenter()
            } else if let Some(p) = inherited {
                p.AlignContentBlockCenter()
            } else {
                matches!(v.Payload(),CSSValuePayload::kIdentifierClass(i) if i.0==CSSValueID::kCenter)
            };
            b.SetAlignContentBlockCenter(value);
        }
        kInternalEmptyLineHeight => {
            let value = if initial {
                ComputedStyleInitialValues::InitialHasLineIfEmpty()
            } else if let Some(p) = inherited {
                p.HasLineIfEmpty()
            } else {
                matches!(v.Payload(),CSSValuePayload::kIdentifierClass(i) if i.0==CSSValueID::kFabricated)
            };
            b.SetHasLineIfEmpty(value);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}
