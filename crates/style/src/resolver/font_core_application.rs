// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! FontBuilder's scalar font inputs and text longhands, on native storage.
#![allow(non_snake_case)]
use super::*;
use font_engine::fonts::{
    font_description::{GenericFamilyType, StyleSyntax},
    font_smoothing_mode::FontSmoothingMode,
    text_rendering_mode::TextRenderingMode,
};
use font_engine::{FontFamily, FontFamilyType, FontSelectionValue};
use foundation::{ETextOrientation, EWordBreak};
use CSSPropertyID::*;
use CSSValueID::*;

pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        kFontFamily
            | kFontSize
            | kFontStyle
            | kFontWeight
            | kTextRendering
            | kWebkitFontSmoothing
            | kTextOrientation
            | kWordBreak
            | kLetterSpacing
            | kWordSpacing
    )
}
fn Keyword(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<CSSValueID, LonghandApplicationError> {
    if let CSSValuePayload::kIdentifierClass(v) = v.Payload() {
        Ok(v.0)
    } else {
        Err(LonghandApplicationError::InvalidValue(id))
    }
}
// cpp: generated longhands.cc:323,509-517,614-616,962-970,1031-1039,
// 1089-1097,9969-9982,19621-19626,19653-19666; custom:4417-4430,
// 4513-4539,10132-10146,12601-12610; converter.cc:454-566,913-959,
// 1092-1182,2784-2793. Font selector scope/system-font/metrics remain owners'
// typed dependencies. This assembly stages descriptions before layout binds data.
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    inherit: bool,
    initial: bool,
) -> Result {
    let mut d = b.GetFontDescription().clone();
    let p = parent.map(ComputedStyle::GetFontDescription);
    let inherited = inherit && !initial;
    match id {
        kFontSize => {
            let (size, keyword, absolute) = if inherited {
                let p = p.unwrap();
                (
                    p.SpecifiedSize() as f64,
                    p.KeywordSize(),
                    p.IsAbsoluteSize(),
                )
            } else if initial {
                (KeywordSize(4, media) as f64, 4, false)
            } else {
                let parent_size = p.map_or(media.em_size, |p| p.SpecifiedSize());
                let parent_absolute = p.is_some_and(|p| p.IsAbsoluteSize());
                match v.Payload() {
                    CSSValuePayload::kIdentifierClass(i) => {
                        let keyword = match i.0 {
                            kXxSmall => 1,
                            kXSmall => 2,
                            kSmall => 3,
                            kMedium => 4,
                            kLarge => 5,
                            kXLarge => 6,
                            kXxLarge => 7,
                            kXxxLarge | kWebkitXxxLarge => 8,
                            _ => 0,
                        };
                        if keyword != 0 {
                            (KeywordSize(keyword, media) as f64, keyword, false)
                        } else {
                            match i.0 {
                                kLarger => (parent_size as f64 * 1.2, 0, parent_absolute),
                                kSmaller => (parent_size as f64 / 1.2, 0, parent_absolute),
                                // OpenType MATH constants require the selected parent font.
                                kMath => return Err(LonghandApplicationError::Unsupported(id)),
                                _ => return Err(LonghandApplicationError::InvalidValue(id)),
                            }
                        }
                    }
                    CSSValuePayload::kNumericLiteralClass(n) => {
                        let u = n.GetType();
                        if u == UnitType::kPercentage {
                            (
                                n.DoubleValue() * parent_size as f64 / 100.0,
                                0,
                                parent_absolute,
                            )
                        } else {
                            (
                                Pixels(id, n.DoubleValue(), u, parent_size, root, media)?,
                                0,
                                parent_absolute
                                    || !matches!(
                                        u,
                                        UnitType::kEms
                                            | UnitType::kQuirkyEms
                                            | UnitType::kExs
                                            | UnitType::kChs
                                            | UnitType::kIcs
                                            | UnitType::kCaps
                                    ),
                            )
                        }
                    }
                    CSSValuePayload::kMathFunctionClass(math) => {
                        use crate::css_math_expression_node::CalculationResultCategory as C;
                        let value = math
                            .ComputeValue(
                                &mut MathLengthResolver(id, parent_size, root, 1.0, media),
                                (math.Category() == C::LengthFunction)
                                    .then_some(parent_size as f64),
                            )
                            .map_err(|_| LonghandApplicationError::Unsupported(id))?;
                        let size = if math.Category() == C::Percent {
                            value * parent_size as f64 / 100.0
                        } else {
                            value
                        };
                        // A typed calculation may include font-relative or
                        // percentage terms. Chromium keeps that dependency on
                        // the parent rather than classifying it as an absolute
                        // keyword-derived size.
                        (size, 0, false)
                    }
                    _ => return Err(LonghandApplicationError::Unsupported(id)),
                }
            };
            if size < 0.0 {
                return Ok(());
            }
            let size = (size as f32)
                .min(layoutng_style::style::computed_style_constants::kMaximumAllowedFontSize);
            d.SetKeywordSize(keyword);
            d.SetIsAbsoluteSize(absolute);
            d.SetSpecifiedSize(size);
            d.SetComputedSize(
                (size * b.EffectiveZoom())
                    .min(layoutng_style::style::computed_style_constants::kMaximumAllowedFontSize),
            );
        }
        kFontWeight => {
            let value = if inherited {
                p.unwrap().Weight()
            } else if initial {
                FontSelectionValue::from_int(400)
            } else {
                let weight = p.map_or(400.0, |p| p.Weight().ToFloat()) as f64;
                let number = match v.Payload() {
                    CSSValuePayload::kNumericLiteralClass(n)
                        if matches!(n.GetType(), UnitType::kNumber | UnitType::kInteger) =>
                    {
                        n.DoubleValue().clamp(1.0, 1000.0)
                    }
                    CSSValuePayload::kIdentifierClass(i) => match i.0 {
                        kNormal => 400.0,
                        kBold => 700.0,
                        kBolder => {
                            if weight < 350.0 {
                                400.0
                            } else if weight < 550.0 {
                                700.0
                            } else if weight < 900.0 {
                                900.0
                            } else {
                                weight
                            }
                        }
                        kLighter => {
                            if weight < 100.0 {
                                weight
                            } else if weight < 550.0 {
                                100.0
                            } else if weight < 750.0 {
                                400.0
                            } else {
                                700.0
                            }
                        }
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    },
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                };
                FontSelectionValue::from_double(number)
            };
            d.SetWeight(value);
        }
        kFontStyle => {
            let (slope, syntax) = if inherited {
                (p.unwrap().Style(), p.unwrap().GetStyleSyntax())
            } else if initial {
                (FontSelectionValue::from_int(0), StyleSyntax::kImplicitAngle)
            } else {
                match v.Payload() {
                    CSSValuePayload::kIdentifierClass(i) => match i.0 {
                        kNormal => (FontSelectionValue::from_int(0), StyleSyntax::kImplicitAngle),
                        kItalic => (
                            FontSelectionValue::from_int(14),
                            StyleSyntax::kItalicKeyword,
                        ),
                        kOblique => (
                            FontSelectionValue::from_int(14),
                            StyleSyntax::kImplicitAngle,
                        ),
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    },
                    CSSValuePayload::kFontStyleRangeClass(range) => {
                        let angle = range
                            .angle
                            .as_ref()
                            .ok_or(LonghandApplicationError::InvalidValue(id))?;
                        (
                            FontSelectionValue::from_double(Degrees(id, b, angle, root, media)?),
                            StyleSyntax::kExplicitAngle,
                        )
                    }
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            d.SetStyle(slope);
            d.SetStyleSyntax(syntax);
        }
        kFontFamily => {
            let (family, generic) = if inherited {
                (p.unwrap().Family().clone(), p.unwrap().GenericFamily())
            } else if initial {
                (FontFamily::default(), GenericFamilyType::kStandardFamily)
            } else {
                Family(id, v)?
            };
            d.SetFamily(&family);
            d.SetGenericFamily(generic);
        }
        kTextRendering => {
            let value = if inherited {
                p.unwrap().TextRendering()
            } else if initial {
                TextRenderingMode::kAutoTextRendering
            } else {
                match Keyword(id, v)? {
                    kAuto => TextRenderingMode::kAutoTextRendering,
                    CSSValueID::kOptimizespeed => TextRenderingMode::kOptimizeSpeed,
                    CSSValueID::kOptimizelegibility => TextRenderingMode::kOptimizeLegibility,
                    CSSValueID::kGeometricprecision => TextRenderingMode::kGeometricPrecision,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            d.SetTextRendering(value);
        }
        kWebkitFontSmoothing => {
            let value = if inherited {
                p.unwrap().FontSmoothing()
            } else if initial {
                FontSmoothingMode::kAutoSmoothing
            } else {
                match Keyword(id, v)? {
                    kAuto => FontSmoothingMode::kAutoSmoothing,
                    kNone => FontSmoothingMode::kNoSmoothing,
                    kAntialiased => FontSmoothingMode::kAntialiased,
                    kSubpixelAntialiased => FontSmoothingMode::kSubpixelAntialiased,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            d.SetFontSmoothing(value);
        }
        kTextOrientation => {
            let value = if inherited {
                parent.unwrap().GetTextOrientation()
            } else if initial {
                ComputedStyleInitialValues::InitialTextOrientation()
            } else {
                match Keyword(id, v)? {
                    kMixed => ETextOrientation::kMixed,
                    kUpright => ETextOrientation::kUpright,
                    kSideways => ETextOrientation::kSideways,
                    kSidewaysRight => ETextOrientation::kSideways,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetTextOrientation(value);
            d.SetOrientation(b.ComputeFontOrientation());
        }
        kWordBreak => {
            let value = if inherited {
                parent.unwrap().WordBreak()
            } else if initial {
                ComputedStyleInitialValues::InitialWordBreak()
            } else {
                match Keyword(id, v)? {
                    kNormal => EWordBreak::kNormal,
                    kBreakAll => EWordBreak::kBreakAll,
                    kKeepAll => EWordBreak::kKeepAll,
                    kBreakWord => EWordBreak::kBreakWord,
                    kAutoPhrase => EWordBreak::kAutoPhrase,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetWordBreak(value);
        }
        kLetterSpacing | kWordSpacing => {
            let length = if inherited {
                // Standardized zoom reapplication needs the document's flag.
                if b.EffectiveZoom() != parent.unwrap().EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                if id == kLetterSpacing {
                    p.unwrap().ComputedLetterSpacing().clone()
                } else {
                    p.unwrap().ComputedWordSpacing().clone()
                }
            } else if initial
                || matches!(v.Payload(),CSSValuePayload::kIdentifierClass(i) if i.0==kNormal)
            {
                Length::Fixed(0.0)
            } else {
                text_application::ConvertLength(id, b, v, root, media)?
            };
            if id == kLetterSpacing {
                d.SetLetterSpacing(&length)
            } else {
                d.SetWordSpacing(&length)
            }
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    StageFontDescription(b, &d);
    Ok(())
}
// cpp: font_size_functions.cc:49-94,232-253; cached em_size is the current
// proportional default. Fixed-font preference and minimum settings require the
// document settings owner; this helper does not invent those preferences.
fn KeywordSize(keyword: u32, media: &MediaValuesCachedData) -> f32 {
    const STRICT: [[u8; 8]; 8] = [
        [9, 9, 9, 9, 11, 14, 18, 27],
        [9, 9, 9, 10, 12, 15, 20, 30],
        [9, 9, 10, 11, 13, 17, 22, 33],
        [9, 9, 10, 12, 14, 18, 24, 36],
        [9, 10, 12, 13, 16, 20, 26, 39],
        [9, 10, 12, 14, 17, 21, 28, 42],
        [9, 10, 13, 15, 18, 23, 30, 45],
        [9, 10, 13, 16, 18, 24, 32, 48],
    ];
    const QUIRKS: [[u8; 8]; 8] = [
        [9, 9, 9, 9, 11, 14, 18, 28],
        [9, 9, 9, 10, 12, 15, 20, 31],
        [9, 9, 9, 11, 13, 17, 22, 34],
        [9, 9, 10, 12, 14, 18, 24, 37],
        [9, 9, 10, 13, 16, 20, 26, 40],
        [9, 9, 11, 14, 17, 21, 28, 42],
        [9, 10, 12, 15, 17, 23, 30, 45],
        [9, 10, 13, 16, 18, 24, 32, 48],
    ];
    let col = keyword as usize - 1;
    let medium = media.em_size;
    if medium.fract() == 0.0 && (9.0..=16.0).contains(&medium) {
        (if media.strict_mode { STRICT } else { QUIRKS })[medium as usize - 9][col] as f32
    } else {
        (medium * [0.60, 0.75, 0.89, 1.0, 1.2, 1.5, 2.0, 3.0][col]).max(1.0)
    }
}
fn Degrees(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<f64, LonghandApplicationError> {
    if let CSSValuePayload::kMathFunctionClass(math) = v.Payload() {
        return math
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
            .map_err(|_| LonghandApplicationError::Unsupported(id));
    }
    let CSSValuePayload::kNumericLiteralClass(n) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    Ok(n.DoubleValue()
        * match n.GetType() {
            UnitType::kDegrees => 1.0,
            UnitType::kRadians => 180.0 / std::f64::consts::PI,
            UnitType::kGradians => 0.9,
            UnitType::kTurns => 360.0,
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        })
}
fn Family(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<(FontFamily, GenericFamilyType), LonghandApplicationError> {
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    let mut generic = GenericFamilyType::kNoFamily;
    let mut result = None;
    for item in list.values.iter().rev() {
        let (name, kind, g) = match item.Payload() {
            CSSValuePayload::kFontFamilyClass(f) => (
                foundation::AtomicString::from_str(&f.0.Utf8()),
                FontFamilyType::kFamilyName,
                GenericFamilyType::kNoFamily,
            ),
            CSSValuePayload::kIdentifierClass(i) => {
                let g = match i.0 {
                    kSerif => GenericFamilyType::kSerifFamily,
                    kSansSerif => GenericFamilyType::kSansSerifFamily,
                    kMonospace => GenericFamilyType::kMonospaceFamily,
                    kCursive => GenericFamilyType::kCursiveFamily,
                    kFantasy => GenericFamilyType::kFantasyFamily,
                    kSystemUi | kMath => GenericFamilyType::kNoFamily,
                    kWebkitBody => return Err(LonghandApplicationError::Unsupported(id)),
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                };
                (
                    foundation::AtomicString::from_str(crate::css_value_keywords::GetCSSValueName(
                        i.0,
                    )),
                    FontFamilyType::kGenericFamily,
                    g,
                )
            }
            _ => return Err(LonghandApplicationError::Unsupported(id)),
        };
        if generic == GenericFamilyType::kNoFamily {
            generic = g;
        }
        let next = result.map(|mut f: FontFamily| {
            let name = f.FamilyName().clone();
            let kind = if f.FamilyIsGeneric() {
                FontFamilyType::kGenericFamily
            } else {
                FontFamilyType::kFamilyName
            };
            font_engine::SharedFontFamily::Create(name, kind, f.ReleaseNext())
        });
        result = Some(FontFamily::new(name, kind, next));
    }
    Ok((result.unwrap_or_default(), generic))
}
