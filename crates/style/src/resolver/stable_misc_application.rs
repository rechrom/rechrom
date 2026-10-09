// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native FontDescription synthesis and math/legacy ordinal application.
#![allow(non_snake_case)]
use super::*;
use font_engine::fonts::font_description::{
    FontSynthesisSmallCaps, FontSynthesisStyle, FontSynthesisWeight,
};
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kMathDepth
            | kWebkitBoxOrdinalGroup
            | kFontSynthesisWeight
            | kFontSynthesisStyle
            | kFontSynthesisSmallCaps
            | kScrollBehavior
            | kResize
            | kUnicodeBidi
    )
}
fn Scalar(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<f64, LonghandApplicationError> {
    typography_application::Scalar(id, b, v, root, media, false)
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
    let initial = v.IsInitialValue() || v.IsUnsetValue() && !inherit || inherit && parent.is_none();
    let inherited = parent.filter(|_| inherit && !initial);
    match id {
        kScrollBehavior | kResize | kUnicodeBidi => {
            if initial {
                return ApplyInitial(id, b);
            }
            if let Some(p) = inherited {
                ApplyInherit(id, b, p)?;
            } else {
                // generated longhands.cc:14339,17858 and custom:8480-8499.
                let keyword = Identifier(id, v)?;
                match id {
                    kScrollBehavior => {
                        use layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior as S;
                        b.SetScrollBehavior(match keyword {
                            CSSValueID::kAuto => S::kAuto,
                            CSSValueID::kSmooth => S::kSmooth,
                            _ => return Err(LonghandApplicationError::InvalidValue(id)),
                        });
                    }
                    kResize => {
                        use foundation::EResize as R;
                        b.SetResize(match keyword {
                            CSSValueID::kNone => R::kNone,
                            CSSValueID::kBoth => R::kBoth,
                            CSSValueID::kHorizontal => R::kHorizontal,
                            CSSValueID::kVertical => R::kVertical,
                            CSSValueID::kBlock => R::kBlock,
                            CSSValueID::kInline => R::kInline,
                            CSSValueID::kAuto | CSSValueID::kInternalTextareaAuto => {
                                return Err(LonghandApplicationError::Unsupported(id))
                            }
                            _ => return Err(LonghandApplicationError::InvalidValue(id)),
                        });
                    }
                    kUnicodeBidi => {
                        use foundation::UnicodeBidi as U;
                        b.SetUnicodeBidi(match keyword {
                            CSSValueID::kNormal => U::kNormal,
                            CSSValueID::kEmbed => U::kEmbed,
                            CSSValueID::kBidiOverride => U::kBidiOverride,
                            CSSValueID::kIsolate => U::kIsolate,
                            CSSValueID::kPlaintext | CSSValueID::kWebkitPlaintext => U::kPlaintext,
                            CSSValueID::kIsolateOverride | CSSValueID::kWebkitIsolateOverride => {
                                U::kIsolateOverride
                            }
                            CSSValueID::kWebkitIsolate => U::kIsolate,
                            _ => return Err(LonghandApplicationError::InvalidValue(id)),
                        });
                    }
                    _ => unreachable!(),
                }
            }
        }
        kMathDepth => {
            let depth = if initial {
                ComputedStyleInitialValues::InitialMathDepth()
            } else if let Some(p) = inherited {
                p.MathDepth()
            } else {
                // longhands_custom.cc:7070-7093; int ComputeInteger precedes
                // ClampTo<int16_t> / ClampAdd. Missing root parent is initial style.
                let p = parent
                    .unwrap_or_else(|| unsafe { &*ComputedStyle::GetInitialStyleSingleton() });
                match v.Payload() {
                    CSSValuePayload::kIdentifierClass(i) if i.0 == CSSValueID::kAutoAdd => p
                        .MathDepth()
                        .saturating_add(if p.MathStyle() == foundation::EMathStyle::kCompact {
                            1
                        } else {
                            0
                        }),
                    CSSValuePayload::kFunctionClass(f)
                        if f.function_id == CSSValueID::kAdd && f.arguments.values.len() == 1 =>
                    {
                        let n = Scalar(id, b, &f.arguments.values[0], root, media)? as i32;
                        (p.MathDepth() as i64 + n as i64).clamp(i16::MIN as i64, i16::MAX as i64)
                            as i16
                    }
                    CSSValuePayload::kNumericLiteralClass(_)
                    | CSSValuePayload::kMathFunctionClass(_) => {
                        (Scalar(id, b, v, root, media)? as i32)
                            .clamp(i16::MIN as i32, i16::MAX as i32) as i16
                    }
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetMathDepth(depth);
        }
        kWebkitBoxOrdinalGroup => {
            let ordinal = if initial {
                ComputedStyleInitialValues::InitialBoxOrdinalGroup()
            } else if let Some(p) = inherited {
                p.BoxOrdinalGroup()
            } else {
                let n = Scalar(id, b, v, root, media)?;
                if n.is_nan() || n < 1.0 || n.is_finite() && n.fract() != 0.0 {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                n as u32
            };
            // ComputedStyle::SetBoxOrdinalGroup clamps to UINT_MAX - 1.
            b.SetBoxOrdinalGroup(ordinal);
        }
        kFontSynthesisWeight | kFontSynthesisStyle | kFontSynthesisSmallCaps => {
            let mut d = b.GetFontDescription().clone();
            let none = if initial {
                false
            } else if inherited.is_none() {
                match Identifier(id, v)? {
                    CSSValueID::kNone => true,
                    CSSValueID::kAuto => false,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            } else {
                false
            };
            let p = inherited.map(ComputedStyle::GetFontDescription);
            match id {
                kFontSynthesisWeight => d.SetFontSynthesisWeight(
                    p.map(|p| p.GetFontSynthesisWeight()).unwrap_or(if none {
                        FontSynthesisWeight::kNoneFontSynthesisWeight
                    } else {
                        FontSynthesisWeight::kAutoFontSynthesisWeight
                    }),
                ),
                kFontSynthesisStyle => d.SetFontSynthesisStyle(
                    p.map(|p| p.GetFontSynthesisStyle()).unwrap_or(if none {
                        FontSynthesisStyle::kNoneFontSynthesisStyle
                    } else {
                        FontSynthesisStyle::kAutoFontSynthesisStyle
                    }),
                ),
                kFontSynthesisSmallCaps => d.SetFontSynthesisSmallCaps(
                    p.map(|p| p.GetFontSynthesisSmallCaps()).unwrap_or(if none {
                        FontSynthesisSmallCaps::kNoneFontSynthesisSmallCaps
                    } else {
                        FontSynthesisSmallCaps::kAutoFontSynthesisSmallCaps
                    }),
                ),
                _ => unreachable!(),
            }
            StageFontDescription(b, &d);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}
