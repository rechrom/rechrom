// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native auto/caret/visited colors and color-scheme application.
#![allow(non_snake_case)]
use super::*;
use foundation::{AtomicString, Vector};
use layoutng_style::css::{
    color_scheme_flags::{ColorSchemeFlag as F, ColorSchemeFlags},
    style_auto_color::StyleAutoColor,
    style_caret_color::StyleCaretColor,
};
use layoutng_style::style::color_scheme::mojom::blink::PreferredColorScheme;
use CSSPropertyID::*;
/// Real page/style-engine settings. The defaults are Chromium's initial page
/// flags and force-dark setting; media supplies the preferred scheme separately.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorSchemeSettings {
    pub page_color_schemes: ColorSchemeFlags,
    pub force_dark: bool,
}
pub(super) fn IsColorUIProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        kAccentColor
            | kCaretColor
            | kColorScheme
            | kInternalVisitedCaretColor
            | kInternalVisitedOutlineColor
            | kInternalVisitedBorderTopColor
            | kInternalVisitedBorderRightColor
            | kInternalVisitedBorderBottomColor
            | kInternalVisitedBorderLeftColor
    )
}
// cpp: style_builder_converter.cc:3054-3070 ConvertStyleColor;
// ResolveColorValueImpl literal/current/named branches. Context-dependent
// link/system/provider/advanced colors (including visited link colors) cannot
// pass this boundary without their real source collaborators.
pub(super) fn ConvertStyleColorValue(
    id: CSSPropertyID,
    value: &Value,
) -> std::result::Result<StyleColor, LonghandApplicationError> {
    match value.Payload() {
        CSSValuePayload::kColorClass(color) => Ok(StyleColor::from_color(color.0)),
        CSSValuePayload::kIdentifierClass(i) if i.0 == CSSValueID::kCurrentcolor => {
            Ok(StyleColor::CurrentColor())
        }
        CSSValuePayload::kIdentifierClass(i) => crate::production_css_value::NamedColor(i.0)
            .map(StyleColor::from_color)
            .ok_or(LonghandApplicationError::Unsupported(id)),
        _ => Err(LonghandApplicationError::Unsupported(id)),
    }
}
// cpp: style_builder_converter.cc:3072-3082.
fn AutoColor(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<StyleAutoColor, LonghandApplicationError> {
    if matches!(v.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == CSSValueID::kAuto) {
        Ok(StyleAutoColor::AutoColor())
    } else {
        Ok(StyleAutoColor::new(ConvertStyleColorValue(id, v)?))
    }
}
// cpp: style_builder_converter.cc:3084-3096. The stable single value sets the
// caret fill; its separate text color stays AutoColor.
fn CaretColor(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<StyleCaretColor, LonghandApplicationError> {
    if let CSSValuePayload::kValueListClass(list) = v.Payload() {
        if list.values.len() != 2 {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
        return Ok(StyleCaretColor::new(
            AutoColor(id, &list.values[0])?,
            AutoColor(id, &list.values[1])?,
        ));
    }
    Ok(StyleCaretColor::new(
        AutoColor(id, v)?,
        StyleAutoColor::AutoColor(),
    ))
}
// cpp: style_builder_converter.cc:3856-3889 ExtractColorSchemes.
fn ExtractColorSchemes(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<(ColorSchemeFlags, Vector<AtomicString>), LonghandApplicationError> {
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    let mut flags = F::kNormal as u8;
    let mut schemes = Vector::new();
    for item in &list.values {
        match item.Payload() {
            CSSValuePayload::kCustomIdentClass(value) => schemes.push(value.name.clone()),
            CSSValuePayload::kIdentifierClass(value) => {
                schemes.push(AtomicString::from_str(
                    crate::css_value_keywords::GetCSSValueName(value.0),
                ));
                flags |= match value.0 {
                    CSSValueID::kDark => F::kDark as u8,
                    CSSValueID::kLight => F::kLight as u8,
                    CSSValueID::kOnly => F::kOnly as u8,
                    _ => 0,
                };
            }
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        }
    }
    Ok((flags, schemes))
}
// cpp: generated longhands.cc Apply*; custom ColorScheme:2526-2575.
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    media: &MediaValuesCachedData,
    settings: ColorSchemeSettings,
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
    match id {
        kAccentColor => {
            let color = if initial {
                StyleAutoColor::AutoColor()
            } else if inherited {
                parent.unwrap().AccentColor().clone()
            } else {
                AutoColor(id, v)?
            };
            b.SetAccentColorOwned(color);
        }
        kCaretColor | kInternalVisitedCaretColor => {
            let color = if initial {
                StyleCaretColor::default()
            } else if inherited {
                parent.unwrap().CaretColor().clone()
            } else {
                CaretColor(id, v)?
            };
            if id == kCaretColor {
                b.SetCaretColorOwned(color);
            } else {
                b.SetInternalVisitedCaretColorOwned(color);
            }
        }
        kColorScheme => {
            if inherited {
                let p = parent.unwrap();
                b.SetColorScheme(p.ColorScheme());
                b.SetDarkColorScheme(p.DarkColorScheme());
                b.SetColorSchemeForced(p.ColorSchemeForced());
            } else {
                let normal = initial
                    || matches!(v.Payload(), CSSValuePayload::kIdentifierClass(i) if i.0 == CSSValueID::kNormal);
                let (flags, schemes) = if normal {
                    (settings.page_color_schemes, Vector::new())
                } else {
                    ExtractColorSchemes(id, v)?
                };
                b.SetColorSchemeOwned(schemes);
                let preferred = match media.preferred_color_scheme {
                    crate::media_queries::PreferredColorScheme::kDark => {
                        PreferredColorScheme::kDark
                    }
                    crate::media_queries::PreferredColorScheme::kLight => {
                        PreferredColorScheme::kLight
                    }
                };
                b.SetUsedColorScheme(flags, preferred, settings.force_dark);
            }
        }
        kInternalVisitedOutlineColor
        | kInternalVisitedBorderTopColor
        | kInternalVisitedBorderRightColor
        | kInternalVisitedBorderBottomColor
        | kInternalVisitedBorderLeftColor => {
            let color = if initial {
                StyleColor::CurrentColor()
            } else if inherited {
                let p = parent.unwrap();
                match id {
                    kInternalVisitedOutlineColor => p.OutlineColor(),
                    kInternalVisitedBorderTopColor => p.BorderTopColor(),
                    kInternalVisitedBorderRightColor => p.BorderRightColor(),
                    kInternalVisitedBorderBottomColor => p.BorderBottomColor(),
                    _ => p.BorderLeftColor(),
                }
                .clone()
            } else {
                ConvertStyleColorValue(id, v)?
            };
            match id {
                kInternalVisitedOutlineColor => b.SetInternalVisitedOutlineColorOwned(color),
                kInternalVisitedBorderTopColor => b.SetInternalVisitedBorderTopColorOwned(color),
                kInternalVisitedBorderRightColor => {
                    b.SetInternalVisitedBorderRightColorOwned(color)
                }
                kInternalVisitedBorderBottomColor => {
                    b.SetInternalVisitedBorderBottomColorOwned(color)
                }
                _ => b.SetInternalVisitedBorderLeftColorOwned(color),
            }
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    Ok(())
}
