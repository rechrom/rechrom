// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native viewport/page fields and explicit FontBuilder/shape owner boundaries.
#![allow(non_snake_case)]
use super::*;
use foundation::{AtomicString, LengthBox};
use layoutng_style::style::page_size_type::PageSizeType;

pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kZoom | kClip | kSize | kPage | kWebkitLocale | kObjectViewBox
    )
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
    if inherit && parent.is_some() && v.IsInheritedValue() && !CSSProperty::Get(id).IsInherited() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    match id {
        // custom:9413-9484. Both CSS-wide methods are genuine empty methods.
        kSize => {
            if !initial && !inherit {
                ConvertSize(b, v, root, media)?;
            }
        }
        // generated:5895-5915; converter:342-350,2111-2120.
        kClip => {
            if initial || Identifier(v) == Some(CSSValueID::kAuto) {
                b.SetHasAutoClip();
            } else if inherit {
                let parent = parent.unwrap();
                if parent.HasAutoClip() {
                    b.SetHasAutoClip();
                } else if b.EffectiveZoom() != parent.EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                } else {
                    b.SetClip(parent.Clip());
                }
            } else {
                let CSSValuePayload::kQuadClass(rect) = v.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                let mut sides = Vec::new();
                for value in &rect.sides {
                    sides.push(if Identifier(value) == Some(CSSValueID::kAuto) {
                        Length::Auto().clone()
                    } else {
                        svg_application::ConvertLength(id, b, value, root, media)?
                    });
                }
                b.SetClip(&LengthBox::new(
                    sides.remove(0),
                    sides.remove(0),
                    sides.remove(0),
                    sides.remove(0),
                ));
            }
        }
        // generated:12985-12993; converter.cc:3761-3769.
        kPage => {
            let name = if initial {
                ComputedStyleInitialValues::InitialPage()
            } else if inherit {
                parent.unwrap().Page().clone()
            } else {
                match v.Payload() {
                    CSSValuePayload::kIdentifierClass(id) if id.0 == CSSValueID::kAuto => {
                        AtomicString::default()
                    }
                    CSSValuePayload::kCustomIdentClass(name) => name.name.clone(),
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetPage(&name);
        }
        // FontBuilder::SetLocale marks the locale dirty; the production font
        // description assembly installs the real Font/FontDescription owner.
        // Full FontBuilder backend font-selector/metric services and the existing
        // native per-thread-cache/ICU script collaborators stay pending.
        kWebkitLocale => {
            let mut description = b.GetFontDescription().clone();
            let locale = if initial || Identifier(v) == Some(CSSValueID::kAuto) {
                None
            } else if inherit {
                let source = parent.unwrap().GetFontDescription().Locale();
                if source.is_null() {
                    None
                } else {
                    font_engine::LayoutLocale::GetShared(unsafe { &*source }.LocaleStringValue())
                }
            } else {
                let CSSValuePayload::kStringClass(text) = v.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                font_engine::LayoutLocale::GetShared(&AtomicString::from_utf16(
                    text.0.Span16().unwrap_or_default(),
                ))
            };
            description.SetLocale(locale);
            StageFontDescription(b, &description);
        }
        // generated:11738-11751; converter.cc:3501-3509.
        kObjectViewBox => {
            if initial || Identifier(v) == Some(CSSValueID::kNone) {
                b.SetObjectViewBox(None);
            } else if inherit {
                if b.EffectiveZoom() != parent.unwrap().EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                b.SetObjectViewBox(parent.unwrap().ObjectViewBox().cloned());
            } else {
                return Err(LonghandApplicationError::Unsupported(id));
            }
        }
        // custom:12761-12773, converter:2087-2109, state.cc:325-339.
        // Zoom is a cascade-affecting property. DocumentStyleEngine applies it
        // before font properties, matching StyleCascade::ApplyCascadeAffecting,
        // so the staged font description can be refreshed here without a
        // second, approximate style pass.
        kZoom => {
            let zoom = if initial {
                ComputedStyleInitialValues::InitialZoom()
            } else if inherit {
                parent.unwrap().Zoom()
            } else {
                ConvertZoom(b, v, root, media)?
            };
            let parent_zoom = parent.map_or(ComputedStyleInitialValues::InitialZoom(), |style| {
                style.EffectiveZoom()
            });
            let effective = (parent_zoom * zoom).clamp(1e-6, 1e6);
            b.SetZoom(zoom);
            if b.SetEffectiveZoom(effective) {
                // FontBuilder::DidChangeEffectiveZoom eventually recomputes
                // ComputedSize from the unzoomed SpecifiedSize. This assembly
                // owns an equivalent immutable FontDescription boundary.
                let mut description = b.GetFontDescription().clone();
                description.SetComputedSize(
                    (description.SpecifiedSize() * effective).min(
                        layoutng_style::style::computed_style_constants::kMaximumAllowedFontSize,
                    ),
                );
                StageFontDescription(b, &description);
            }
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    Ok(())
}

fn ConvertZoom(
    builder: &ComputedStyleBuilder,
    value: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<f32, LonghandApplicationError> {
    let id = CSSPropertyID::kZoom;
    if Identifier(value) == Some(CSSValueID::kNormal) {
        return Ok(ComputedStyleInitialValues::InitialZoom());
    }
    let (number, percent) = match value.Payload() {
        CSSValuePayload::kNumericLiteralClass(number) => (
            number.DoubleValue(),
            number.GetType() == UnitType::kPercentage,
        ),
        CSSValuePayload::kMathFunctionClass(math) => {
            let number = math
                .ComputeValue(
                    &mut MathLengthResolver(
                        id,
                        builder.GetFontDescription().ComputedSize(),
                        root,
                        builder.EffectiveZoom(),
                        media,
                    ),
                    None,
                )
                .map_err(|_| LonghandApplicationError::Unsupported(id))?;
            (
                number,
                math.Category()
                    == crate::css_math_expression_node::CalculationResultCategory::Percent,
            )
        }
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    };
    let number = number.clamp(f32::MIN as f64, f32::MAX as f64) as f32;
    Ok(if number == 0. {
        1.
    } else if percent {
        number / 100.
    } else {
        number
    })
}

fn ConvertSize(
    b: &mut ComputedStyleBuilder,
    value: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    let id = CSSPropertyID::kSize;
    let CSSValuePayload::kValueListClass(list) = value.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    let mut kind = PageSizeType::kAuto;
    let size = match list.values.as_slice() {
        [first, second] => {
            kind = PageSizeType::kFixed;
            if Identifier(first).is_none() {
                foundation::gfx::SizeF::new(
                    UnzoomedLength(b, first, root, media)?,
                    UnzoomedLength(b, second, root, media)?,
                )
            } else {
                let size = NamedPageSize(Identifier(first).unwrap())?;
                if Identifier(second) == Some(CSSValueID::kLandscape) {
                    foundation::gfx::SizeF::new(size.height(), size.width())
                } else {
                    size
                }
            }
        }
        [first] => {
            if let Some(keyword) = Identifier(first) {
                match keyword {
                    CSSValueID::kAuto => foundation::gfx::SizeF::default(),
                    CSSValueID::kPortrait => {
                        kind = PageSizeType::kPortrait;
                        foundation::gfx::SizeF::default()
                    }
                    CSSValueID::kLandscape => {
                        kind = PageSizeType::kLandscape;
                        foundation::gfx::SizeF::default()
                    }
                    keyword => {
                        kind = PageSizeType::kFixed;
                        NamedPageSize(keyword)?
                    }
                }
            } else {
                kind = PageSizeType::kFixed;
                let width = UnzoomedLength(b, first, root, media)?;
                foundation::gfx::SizeF::new(width, width)
            }
        }
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    };
    b.ResetPageSizeType();
    b.SetPageSizeType(kind);
    b.SetPageSizeOwned(size);
    Ok(())
}
fn UnzoomedLength(
    b: &ComputedStyleBuilder,
    value: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<f32, LonghandApplicationError> {
    let id = CSSPropertyID::kSize;
    use crate::css_math_expression_node::CSSMathLengthResolver;
    let font = b.GetFontDescription().SpecifiedSize();
    let mut resolver = |number, unit| {
        Pixels(
            id,
            number,
            if unit == UnitType::kUserUnits {
                UnitType::kPixels
            } else {
                unit
            },
            font,
            root,
            media,
        )
        .map_err(|_| crate::css_math_expression_node::MathError::MissingLengthContext)
    };
    let number = match value.Payload() {
        CSSValuePayload::kNumericLiteralClass(number) => {
            resolver.ComputeLength(number.DoubleValue(), number.GetType())
        }
        CSSValuePayload::kMathFunctionClass(math) => math.ComputeValue(&mut resolver, None),
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    }
    .map_err(|_| LonghandApplicationError::Unsupported(id))?;
    Ok(number.clamp(f32::MIN as f64, f32::MAX as f64) as f32)
}
fn NamedPageSize(
    id: CSSValueID,
) -> std::result::Result<foundation::gfx::SizeF, LonghandApplicationError> {
    use CSSValueID::*;
    // longhands_custom.cc:9334-9366: CSS millimetres/inches, without zoom.
    let (width, height, inches) = match id {
        kA5 => (148., 210., false),
        kA4 => (210., 297., false),
        kA3 => (297., 420., false),
        kB5 => (176., 250., false),
        kB4 => (250., 353., false),
        kJisB5 => (182., 257., false),
        kJisB4 => (257., 364., false),
        kLetter => (8.5, 11., true),
        kLegal => (8.5, 14., true),
        kLedger => (11., 17., true),
        _ => return Err(LonghandApplicationError::InvalidValue(CSSPropertyID::kSize)),
    };
    // MmToPx takes float input but multiplies the genuine double CSS unit
    // constant (css_resolution_units.h:31-33) before narrowing to float.
    let factor = if inches {
        96.0f64
    } else {
        (96.0f64 / 2.54) / 10.0
    };
    Ok(foundation::gfx::SizeF::new(
        (width * factor) as f32,
        (height * factor) as f32,
    ))
}
fn Identifier(value: &Value) -> Option<CSSValueID> {
    if let CSSValuePayload::kIdentifierClass(id) = value.Payload() {
        Some(id.0)
    } else {
        None
    }
}
