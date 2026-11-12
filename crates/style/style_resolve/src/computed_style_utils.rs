// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/css/properties/computed_style_utils.cc:135-142,196-212,
// 1016-1074,1104-1224,2394-2425,4988-4998;
// core/css/zoom_adjusted_pixel_value.h:16-21.
//! Typed computed-value construction over the canonical style owner.
use crate::production_css_value::{self as values, Value};
use font_engine::fonts::{
    font_description::{FontVariantCaps, LigaturesState, StyleSyntax},
    font_selection_types::{kItalicSlopeValue, kNormalSlopeValue},
};
use foundation::{CSSValueID, Length, TextDecorationLine};
use layoutng_style::{
    css::{style_auto_color::StyleAutoColor, style_color::StyleColor},
    style::computed_style::ComputedStyle,
};
use std::rc::Rc;
use style_model::{css_primitive_value::UnitType, css_value::CSSValuePayload};
use values::ListSeparator;

pub fn ZoomAdjustedPixelValue(value: impl Into<f64>, style: &ComputedStyle) -> Rc<Value> {
    values::numeric(
        value.into() / style.EffectiveZoom() as f64,
        UnitType::kPixels,
    )
}
pub fn ZoomAdjustedPixelValueForLength(length: &Length, style: &ComputedStyle) -> Rc<Value> {
    if length.IsFixed() {
        ZoomAdjustedPixelValue(length.Pixels(), style)
    } else {
        Value::Create(length, style.EffectiveZoom())
    }
}
pub fn CurrentColorOrValidColor(style: &ComputedStyle, color: &StyleColor) -> Rc<Value> {
    values::color(color.Resolve(style.GetCurrentColor(None), style.UsedColorScheme(), None))
}
pub fn ValueForStyleAutoColor(style: &ComputedStyle, color: &StyleAutoColor) -> Rc<Value> {
    if color.IsAutoColor() {
        values::color(style.GetCurrentColor(None))
    } else {
        CurrentColorOrValidColor(style, color.ToStyleColor())
    }
}
pub fn ValueForFontFamily(style: &ComputedStyle) -> Rc<Value> {
    let mut result = Vec::new();
    let mut family = Some(style.GetFontDescription().Family());
    while let Some(current) = family {
        let name = current.FamilyName().Utf8();
        result.push(if current.FamilyIsGeneric() {
            values::identifier(match name.as_str() {
                "cursive" => CSSValueID::kCursive,
                "fantasy" => CSSValueID::kFantasy,
                "monospace" => CSSValueID::kMonospace,
                "sans-serif" => CSSValueID::kSansSerif,
                "serif" => CSSValueID::kSerif,
                "system-ui" => CSSValueID::kSystemUi,
                "math" => CSSValueID::kMath,
                _ => CSSValueID::kWebkitBody,
            })
        } else {
            Rc::new(Value::new(CSSValuePayload::kFontFamilyClass(
                values::CSSFontFamilyValue(name.into()),
            )))
        });
        family = unsafe { current.Next().as_ref() };
    }
    values::list(result, ListSeparator::Comma)
}
pub fn ValueForFontStyle(style: &ComputedStyle) -> Rc<Value> {
    let font = style.GetFontDescription();
    let slope = font.Style().ClampToObliqueRange();
    match font.GetStyleSyntax() {
        StyleSyntax::kItalicKeyword => return values::identifier(CSSValueID::kItalic),
        StyleSyntax::kImplicitAngle => {
            if slope == kNormalSlopeValue {
                return values::identifier(CSSValueID::kNormal);
            }
            if slope == kItalicSlopeValue {
                return values::identifier(CSSValueID::kOblique);
            }
        }
        StyleSyntax::kExplicitAngle => {
            if slope == kNormalSlopeValue {
                return values::identifier(CSSValueID::kNormal);
            }
        }
    }
    Rc::new(Value::new(CSSValuePayload::kFontStyleRangeClass(
        values::CSSFontStyleRangeValue {
            angle: Some(values::numeric(slope.ToFloat() as f64, UnitType::kDegrees)),
        },
    )))
}
pub fn ValueForFontVariantCaps(style: &ComputedStyle) -> Rc<Value> {
    use FontVariantCaps as F;
    values::identifier(match style.GetFontDescription().VariantCaps() {
        F::kCapsNormal => CSSValueID::kNormal,
        F::kSmallCaps => CSSValueID::kSmallCaps,
        F::kAllSmallCaps => CSSValueID::kAllSmallCaps,
        F::kPetiteCaps => CSSValueID::kPetiteCaps,
        F::kAllPetiteCaps => CSSValueID::kAllPetiteCaps,
        F::kUnicase => CSSValueID::kUnicase,
        F::kTitlingCaps => CSSValueID::kTitlingCaps,
    })
}
pub fn ValueForFontVariantLigatures(style: &ComputedStyle) -> Rc<Value> {
    use LigaturesState as L;
    let ligatures = style.GetFontDescription().GetVariantLigatures();
    let states = [
        ligatures.common,
        ligatures.discretionary,
        ligatures.historical,
        ligatures.contextual,
    ];
    if states
        .iter()
        .all(|&state| state == L::kNormalLigaturesState)
    {
        return values::identifier(CSSValueID::kNormal);
    }
    if states
        .iter()
        .all(|&state| state == L::kDisabledLigaturesState)
    {
        return values::identifier(CSSValueID::kNone);
    }
    let names = [
        (CSSValueID::kNoCommonLigatures, CSSValueID::kCommonLigatures),
        (
            CSSValueID::kNoDiscretionaryLigatures,
            CSSValueID::kDiscretionaryLigatures,
        ),
        (
            CSSValueID::kNoHistoricalLigatures,
            CSSValueID::kHistoricalLigatures,
        ),
        (CSSValueID::kNoContextual, CSSValueID::kContextual),
    ];
    values::list(
        states
            .iter()
            .zip(names)
            .filter_map(|(&state, (disabled, enabled))| {
                (state != L::kNormalLigaturesState).then(|| {
                    values::identifier(if state == L::kDisabledLigaturesState {
                        disabled
                    } else {
                        enabled
                    })
                })
            })
            .collect(),
        ListSeparator::Space,
    )
}
pub fn RenderTextDecorationFlagsToCSSValue(flags: TextDecorationLine) -> Rc<Value> {
    if flags == TextDecorationLine::kNone {
        return values::identifier(CSSValueID::kNone);
    }
    if flags == TextDecorationLine::kSpellingError {
        return values::identifier(CSSValueID::kSpellingError);
    }
    if flags == TextDecorationLine::kGrammarError {
        return values::identifier(CSSValueID::kGrammarError);
    }
    let items: Vec<_> = [
        (TextDecorationLine::kUnderline, CSSValueID::kUnderline),
        (TextDecorationLine::kOverline, CSSValueID::kOverline),
        (TextDecorationLine::kLineThrough, CSSValueID::kLineThrough),
        (TextDecorationLine::kBlink, CSSValueID::kBlink),
    ]
    .into_iter()
    .filter_map(|(flag, id)| ((flags.bits() & flag.bits()) != 0).then(|| values::identifier(id)))
    .collect();
    if items.is_empty() {
        values::identifier(CSSValueID::kNone)
    } else {
        values::list(items, ListSeparator::Space)
    }
}
