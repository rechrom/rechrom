//! Chromium ComputedStyleUtils.cc:1075-1103,1226-1503,1514-1522;
//! longhands_custom.cc:4432-4487,4587-4700,4754-4830.
use crate::production_css_value::{self as values, Value};
use font_engine::fonts::{
    font_description::{
        FontSynthesisSmallCaps, FontSynthesisStyle, FontSynthesisWeight, FontVariantPosition,
        Kerning,
    },
    font_optical_sizing::OpticalSizing,
    font_size_adjust::{FontSizeAdjust, Metric},
    font_variant_east_asian::*,
    font_variant_emoji::FontVariantEmoji,
    font_variant_numeric::*,
};
use foundation::{AtomicString, CSSPropertyID as P, CSSValueID as V, String as BlinkString};
use layoutng_style::style::computed_style::ComputedStyle;
use std::rc::Rc;
use style_model::{css_primitive_value::UnitType, css_value::CSSValuePayload};
fn ident(id: V) -> Rc<Value> {
    values::identifier(id)
}
fn number(value: f64) -> Rc<Value> {
    values::numeric(value, UnitType::kNumber)
}
fn atom_string(atom: &AtomicString) -> BlinkString {
    BlinkString::from_utf16(atom.utf16_units().unwrap_or_default())
}
fn tag(tag: u32) -> AtomicString {
    AtomicString::from_utf16(&tag.to_be_bytes().map(u16::from))
}
fn list(ids: Vec<V>) -> Rc<Value> {
    values::list(
        ids.into_iter().map(ident).collect(),
        values::ListSeparator::Space,
    )
}
pub fn FontValue(property: P, style: &ComputedStyle) -> Option<Rc<Value>> {
    let font = style.GetFontDescription();
    Some(match property {
        P::kFontStretch => values::numeric(font.Stretch().ToFloat() as f64, UnitType::kPercentage),
        P::kFontKerning => ident(match font.GetKerning() {
            Kerning::kAutoKerning => V::kAuto,
            Kerning::kNormalKerning => V::kNormal,
            Kerning::kNoneKerning => V::kNone,
        }),
        P::kFontOpticalSizing => ident(match font.FontOpticalSizing() {
            OpticalSizing::kAutoOpticalSizing => V::kAuto,
            OpticalSizing::kNoneOpticalSizing => V::kNone,
        }),
        P::kFontSynthesisWeight => ident(match font.GetFontSynthesisWeight() {
            FontSynthesisWeight::kAutoFontSynthesisWeight => V::kAuto,
            FontSynthesisWeight::kNoneFontSynthesisWeight => V::kNone,
        }),
        P::kFontSynthesisStyle => ident(match font.GetFontSynthesisStyle() {
            FontSynthesisStyle::kAutoFontSynthesisStyle => V::kAuto,
            FontSynthesisStyle::kNoneFontSynthesisStyle => V::kNone,
        }),
        P::kFontSynthesisSmallCaps => ident(match font.GetFontSynthesisSmallCaps() {
            FontSynthesisSmallCaps::kAutoFontSynthesisSmallCaps => V::kAuto,
            FontSynthesisSmallCaps::kNoneFontSynthesisSmallCaps => V::kNone,
        }),
        P::kFontVariantPosition => ident(match font.VariantPosition() {
            FontVariantPosition::kNormalVariantPosition => V::kNormal,
            FontVariantPosition::kSubVariantPosition => V::kSub,
            FontVariantPosition::kSuperVariantPosition => V::kSuper,
        }),
        P::kFontVariantEmoji => ident(match font.VariantEmoji() {
            FontVariantEmoji::kNormalVariantEmoji => V::kNormal,
            FontVariantEmoji::kTextVariantEmoji => V::kText,
            FontVariantEmoji::kEmojiVariantEmoji => V::kEmoji,
            FontVariantEmoji::kUnicodeVariantEmoji => V::kUnicode,
        }),
        P::kFontLanguageOverride => {
            let language = font.FontLanguageOverride();
            if language.empty() {
                ident(V::kNormal)
            } else {
                values::string(atom_string(language))
            }
        }
        P::kFontSizeAdjust => {
            let adjust = style.FontSizeAdjust();
            if !style.HasFontSizeAdjust() || adjust.Value() == FontSizeAdjust::kFontSizeAdjustNone {
                ident(V::kNone)
            } else {
                let value = number(adjust.Value() as f64);
                match adjust.GetMetric() {
                    Metric::kExHeight => value,
                    metric => {
                        let id = match metric {
                            Metric::kExHeight => unreachable!(),
                            Metric::kCapHeight => V::kCapHeight,
                            Metric::kChWidth => V::kChWidth,
                            Metric::kIcWidth => V::kIcWidth,
                            Metric::kIcHeight => V::kIcHeight,
                        };
                        Rc::new(Value::new(CSSValuePayload::kValuePairClass(
                            values::CSSValuePair {
                                first: ident(id),
                                second: value,
                                drop_identical: false,
                            },
                        )))
                    }
                }
            }
        }
        P::kFontFeatureSettings => {
            if let Some(settings) = font
                .FeatureSettings()
                .filter(|settings| settings.size() != 0)
            {
                values::list(
                    settings
                        .iter()
                        .map(|feature| {
                            Rc::new(Value::new(CSSValuePayload::kFontFeatureClass(
                                values::CSSFontFeatureValue {
                                    tag: tag(feature.Tag()),
                                    value: number(feature.Value() as f64),
                                },
                            )))
                        })
                        .collect(),
                    values::ListSeparator::Comma,
                )
            } else {
                ident(V::kNormal)
            }
        }
        P::kFontVariationSettings => {
            if let Some(settings) = font
                .VariationSettings()
                .filter(|settings| settings.size() != 0)
            {
                values::list(
                    settings
                        .iter()
                        .map(|axis| {
                            Rc::new(Value::new(CSSValuePayload::kFontVariationClass(
                                values::CSSFontVariationValue {
                                    tag: tag(axis.Tag()),
                                    value: number(axis.Value() as f64),
                                },
                            )))
                        })
                        .collect(),
                    values::ListSeparator::Comma,
                )
            } else {
                ident(V::kNormal)
            }
        }
        P::kFontVariantNumeric => {
            let value = font.VariantNumeric();
            if value.IsAllNormal() {
                ident(V::kNormal)
            } else {
                let mut ids = Vec::new();
                if value.NumericFigureValue() != NumericFigure::kNormalFigure {
                    ids.push(
                        if value.NumericFigureValue() == NumericFigure::kLiningNums {
                            V::kLiningNums
                        } else {
                            V::kOldstyleNums
                        },
                    );
                }
                if value.NumericSpacingValue() != NumericSpacing::kNormalSpacing {
                    ids.push(
                        if value.NumericSpacingValue() == NumericSpacing::kProportionalNums {
                            V::kProportionalNums
                        } else {
                            V::kTabularNums
                        },
                    );
                }
                if value.NumericFractionValue() != NumericFraction::kNormalFraction {
                    ids.push(
                        if value.NumericFractionValue() == NumericFraction::kDiagonalFractions {
                            V::kDiagonalFractions
                        } else {
                            V::kStackedFractions
                        },
                    );
                }
                if value.OrdinalValue() == Ordinal::kOrdinalOn {
                    ids.push(V::kOrdinal);
                }
                if value.SlashedZeroValue() == SlashedZero::kSlashedZeroOn {
                    ids.push(V::kSlashedZero);
                }
                list(ids)
            }
        }
        P::kFontVariantEastAsian => {
            let value = font.VariantEastAsian();
            if value.IsAllNormal() {
                ident(V::kNormal)
            } else {
                let mut ids = Vec::new();
                match value.Form() {
                    EastAsianForm::kNormalForm => {}
                    EastAsianForm::kJis78 => ids.push(V::kJis78),
                    EastAsianForm::kJis83 => ids.push(V::kJis83),
                    EastAsianForm::kJis90 => ids.push(V::kJis90),
                    EastAsianForm::kJis04 => ids.push(V::kJis04),
                    EastAsianForm::kSimplified => ids.push(V::kSimplified),
                    EastAsianForm::kTraditional => ids.push(V::kTraditional),
                    _ => unreachable!("canonical EastAsianForm"),
                }
                match value.Width() {
                    EastAsianWidth::kNormalWidth => {}
                    EastAsianWidth::kFullWidth => ids.push(V::kFullWidth),
                    EastAsianWidth::kProportionalWidth => ids.push(V::kProportionalWidth),
                    _ => unreachable!("canonical EastAsianWidth"),
                }
                if value.Ruby() {
                    ids.push(V::kRuby);
                }
                list(ids)
            }
        }
        P::kFontVariantAlternates => {
            if let Some(alternates) = font
                .FontVariantAlternatesValue()
                .filter(|value| !value.IsNormal())
            {
                let single = |kind: V, name: &AtomicString| {
                    values::alternate(
                        kind,
                        vec![values::custom_ident(&atom_string(name), P::kInvalid)],
                    )
                };
                let mut values_out = Vec::new();
                if let Some(name) = unsafe { alternates.Stylistic().as_ref() } {
                    values_out.push(single(V::kStylistic, name));
                }
                if alternates.HistoricalForms() {
                    values_out.push(ident(V::kHistoricalForms));
                }
                for (kind, name) in [
                    (V::kSwash, alternates.Swash()),
                    (V::kOrnaments, alternates.Ornaments()),
                    (V::kAnnotation, alternates.Annotation()),
                ] {
                    if let Some(name) = unsafe { name.as_ref() } {
                        values_out.push(single(kind, name));
                    }
                }
                for (kind, names) in [
                    (V::kStyleset, alternates.Styleset()),
                    (V::kCharacterVariant, alternates.CharacterVariant()),
                ] {
                    if !names.is_empty() {
                        values_out.push(values::alternate(
                            kind,
                            names
                                .iter()
                                .map(|name| values::custom_ident(&atom_string(name), P::kInvalid))
                                .collect(),
                        ));
                    }
                }
                debug_assert!(!values_out.is_empty());
                values::list(values_out, values::ListSeparator::Space)
            } else {
                ident(V::kNormal)
            }
        }
        _ => return None,
    })
}
