// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium text box, fitting, spacing and intentionally empty legacy Apply*.
#![allow(non_snake_case)]
use super::*;
use font_engine::fonts::shaping::text_spacing_trim::TextSpacingTrim;
use foundation::{ETextAutospace, ETextBoxTrim};
use layoutng_style::style::{
    text_box_edge::{TextBoxEdge, TextBoxEdgeType as Edge},
    text_decoration_inset::TextDecorationInset,
    text_fit::{TextFit, TextFitTarget, TextFitType},
};
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kTextBoxEdge
            | kTextBoxTrim
            | kTextFit
            | kTextAutospace
            | kTextSpacingTrim
            | kTextDecorationInset
            | kWebkitTextDecorationsInEffect
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::production_css_value as values;
    #[test]
    fn hidden_text_box_inset_and_spacing_apply_to_real_native_fields() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut b = ComputedStyleBuilder::from_style(initial);
        let media = MediaValuesCachedData::default();
        let pair = values::Value::new(CSSValuePayload::kValuePairClass(values::CSSValuePair {
            first: values::numeric(-3.0, UnitType::kPixels),
            second: values::numeric(20.0, UnitType::kPercentage),
            drop_identical: true,
        }));
        ApplyInternal(
            CSSPropertyID::kTextDecorationInset,
            &mut b,
            None,
            &pair,
            16.0,
            &media,
        )
        .unwrap();
        assert_eq!(b.GetTextDecorationInset().GetStart().Pixels(), -3.0);
        assert_eq!(b.GetTextDecorationInset().GetEnd().PercentValue(), 20.0);
        let parent = unsafe { &*b.TakeStyle() };
        let mut child = ComputedStyleBuilder::from_style(initial);
        ApplyInternal(
            CSSPropertyID::kTextDecorationInset,
            &mut child,
            Some(parent),
            &values::wide(CSSValueID::kInherit).unwrap(),
            16.0,
            &media,
        )
        .unwrap();
        assert!(child.GetTextDecorationInset() == parent.GetTextDecorationInset());
        child.SetEffectiveZoom(2.0);
        assert!(
            ApplyInternal(
                CSSPropertyID::kTextDecorationInset,
                &mut child,
                Some(parent),
                &values::wide(CSSValueID::kInherit).unwrap(),
                16.0,
                &media
            )
            .is_err()
        );
        child.SetEffectiveZoom(1.0);
        ApplyInternal(
            CSSPropertyID::kTextDecorationInset,
            &mut child,
            None,
            &values::identifier(CSSValueID::kAuto),
            16.0,
            &media,
        )
        .unwrap();
        assert!(child.GetTextDecorationInset().GetStart().IsAuto());
        assert!(child.GetTextDecorationInset().GetEnd().IsAuto());
        ApplyInternal(
            CSSPropertyID::kTextDecorationInset,
            &mut child,
            None,
            &values::wide(CSSValueID::kInitial).unwrap(),
            16.0,
            &media,
        )
        .unwrap();
        assert_eq!(child.GetTextDecorationInset().GetStart().Pixels(), 0.0);
        assert_eq!(child.GetTextDecorationInset().GetEnd().Pixels(), 0.0);
        // The hidden shorthand's expansion consists of public real longhands.
        ApplyInternal(
            CSSPropertyID::kTextAutospace,
            &mut child,
            None,
            &values::identifier(CSSValueID::kNoAutospace),
            16.0,
            &media,
        )
        .unwrap();
        ApplyInternal(
            CSSPropertyID::kTextSpacingTrim,
            &mut child,
            None,
            &values::identifier(CSSValueID::kSpaceAll),
            16.0,
            &media,
        )
        .unwrap();
        assert_eq!(child.TextAutospace(), ETextAutospace::kNoAutospace);
        assert_eq!(
            child.GetFontDescription().GetTextSpacingTrim(),
            TextSpacingTrim::kSpaceAll
        );
        let bad = values::list(
            vec![
                values::identifier(CSSValueID::kCap),
                values::identifier(CSSValueID::kAuto),
            ],
            values::ListSeparator::Space,
        );
        assert!(
            ApplyInternal(
                CSSPropertyID::kTextBoxEdge,
                &mut child,
                None,
                &bad,
                16.0,
                &media
            )
            .is_err()
        );
        assert!(child.GetTextBoxEdge().IsAuto());
        let bad = values::list(
            vec![
                values::identifier(CSSValueID::kGrow),
                values::numeric(-1.0, UnitType::kPercentage),
            ],
            values::ListSeparator::Space,
        );
        assert!(
            ApplyInternal(
                CSSPropertyID::kTextFit,
                &mut child,
                None,
                &bad,
                16.0,
                &media
            )
            .is_err()
        );
        assert_eq!(child.GetTextFit().Type(), TextFitType::kNone);
    }
}
fn EdgeKeyword(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<Edge, LonghandApplicationError> {
    use CSSValueID::*;
    Ok(match Identifier(id, v)? {
        kAuto => Edge::kAuto,
        kText => Edge::kText,
        kCap => Edge::kCap,
        kEx => Edge::kEx,
        kAlphabetic => Edge::kAlphabetic,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    if !crate::production_text_box_features::IsExposed(id) {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    ApplyInternal(id, b, parent, v, root, media)
}
pub(super) fn ApplyInternal(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    use CSSPropertyID::*;
    use CSSValueID::*;
    // generated longhands.cc:19157-19165. A genuine intentionally empty apply:
    // this legacy property reports effective decorations but cannot modify them.
    if id == kWebkitTextDecorationsInEffect {
        return Ok(());
    }
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue() || v.IsUnsetValue() && !inherit || inherit && parent.is_none();
    let inherited = parent.filter(|_| inherit && !initial);
    match id {
        kTextBoxEdge => {
            let edge = if initial {
                TextBoxEdge::default()
            } else if let Some(p) = inherited {
                p.GetTextBoxEdge()
            } else {
                match v.Payload() {
                    CSSValuePayload::kIdentifierClass(_) => {
                        let over = EdgeKeyword(id, v)?;
                        if over != Edge::kAuto && over != Edge::kText {
                            return Err(LonghandApplicationError::InvalidValue(id));
                        }
                        TextBoxEdge::from_over(over)
                    }
                    CSSValuePayload::kValueListClass(l)
                        if l.separator == crate::production_css_value::ListSeparator::Space
                            && l.values.len() == 2 =>
                    {
                        let over = EdgeKeyword(id, &l.values[0])?;
                        let under = EdgeKeyword(id, &l.values[1])?;
                        if ![Edge::kText, Edge::kCap, Edge::kEx].contains(&over)
                            || ![Edge::kText, Edge::kAlphabetic].contains(&under)
                        {
                            return Err(LonghandApplicationError::InvalidValue(id));
                        }
                        TextBoxEdge::new(over, under)
                    }
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetTextBoxEdge(edge);
        }
        kTextBoxTrim => {
            let trim = if initial {
                ETextBoxTrim::kNone
            } else if let Some(p) = inherited {
                p.TextBoxTrim()
            } else {
                match Identifier(id, v)? {
                    kNone => ETextBoxTrim::kNone,
                    kTrimBoth => ETextBoxTrim::kTrimBoth,
                    kTrimStart => ETextBoxTrim::kTrimStart,
                    kTrimEnd => ETextBoxTrim::kTrimEnd,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetTextBoxTrim(trim);
        }
        kTextAutospace => {
            let val = if initial {
                ETextAutospace::kNoAutospace
            } else if let Some(p) = inherited {
                p.TextAutospace()
            } else {
                match Identifier(id, v)? {
                    kNormal => ETextAutospace::kNormal,
                    kNoAutospace => ETextAutospace::kNoAutospace,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetTextAutospace(val);
        }
        // generated longhands.cc:1060-1068 uses FontBuilder's description field.
        // StageFontDescription preserves all other native font attributes.
        kTextSpacingTrim => {
            let trim = if initial {
                TextSpacingTrim::kInitial
            } else if let Some(p) = inherited {
                p.GetFontDescription().GetTextSpacingTrim()
            } else {
                match Identifier(id, v)? {
                    kNormal => TextSpacingTrim::kNormal,
                    kSpaceAll => TextSpacingTrim::kSpaceAll,
                    kSpaceFirst => TextSpacingTrim::kSpaceFirst,
                    kTrimStart => TextSpacingTrim::kTrimStart,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            let mut d = b.GetFontDescription().clone();
            d.SetTextSpacingTrim(trim);
            StageFontDescription(b, &d);
        }
        // style_builder_converter.cc:4244-4292. List order is type/target/limit.
        kTextFit => {
            let fit = if initial {
                TextFit::default()
            } else if let Some(p) = inherited {
                p.GetTextFit().clone()
            } else {
                let CSSValuePayload::kValueListClass(l) = v.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                if l.separator != crate::production_css_value::ListSeparator::Space
                    || !(1..=3).contains(&l.values.len())
                {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                let kind = match Identifier(id, &l.values[0])? {
                    kNone => TextFitType::kNone,
                    kGrow => TextFitType::kGrow,
                    kShrink => TextFitType::kShrink,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                };
                let (mut index, mut target) = (1, TextFitTarget::kConsistent);
                if index < l.values.len()
                    && matches!(
                        l.values[index].Payload(),
                        CSSValuePayload::kIdentifierClass(_)
                    )
                {
                    target = match Identifier(id, &l.values[index])? {
                        kConsistent => TextFitTarget::kConsistent,
                        kPerLine => TextFitTarget::kPerLine,
                        kPerLineAll => TextFitTarget::kPerLineAll,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    };
                    index += 1;
                }
                let limit = if index < l.values.len() {
                    let percent =
                        typography_application::Scalar(id, b, &l.values[index], root, media, true)?;
                    if percent.is_nan() || percent < 0.0 {
                        return Err(LonghandApplicationError::InvalidValue(id));
                    }
                    index += 1;
                    Some((percent.min(f32::MAX as f64) as f32) / 100.0)
                } else {
                    None
                };
                if index != l.values.len() {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                TextFit::new(kind, target, limit)
            };
            b.SetTextFit(&fit);
        }
        // This hidden property's same-zoom native branch is translated, with
        // the source's changed standardized-browser-zoom owner left Unsupported.
        kTextDecorationInset => {
            let inset = if initial {
                TextDecorationInset::new(&Length::Fixed(0), &Length::Fixed(0))
            } else if let Some(p) = inherited {
                if p.EffectiveZoom() != b.EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                p.GetTextDecorationInset().clone()
            } else if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==kAuto) {
                TextDecorationInset::new(Length::Auto(), Length::Auto())
            } else {
                let CSSValuePayload::kValuePairClass(pair) = v.Payload() else {
                    return Err(LonghandApplicationError::InvalidValue(id));
                };
                let start = text_application::ConvertLength(id, b, &pair.first, root, media)?;
                let end = text_application::ConvertLength(id, b, &pair.second, root, media)?;
                TextDecorationInset::new(&start, &end)
            };
            b.SetTextDecorationInset(&inset);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}
