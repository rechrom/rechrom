// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native color slots and currentColor inheritance. Color providers stay typed
//! conversion dependencies; no document or renderer belongs in this module.
#![allow(non_snake_case)]
use super::*;
use foundation::EInsideLink;
use layoutng_style::style::gap_data_list::GapDataList;
use CSSPropertyID::*;

pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        kColor
            | kBackgroundColor
            | kWebkitTapHighlightColor
            | kWebkitTextFillColor
            | kInternalVisitedColor
            | kInternalVisitedBackgroundColor
            | kInternalVisitedColumnRuleColor
            | kInternalVisitedTextDecorationColor
            | kInternalVisitedTextEmphasisColor
            | kInternalVisitedTextFillColor
            | kInternalVisitedTextStrokeColor
            | kInternalForcedColor
            | kInternalForcedVisitedColor
            | kInternalForcedBackgroundColor
            | kInternalForcedBorderColor
            | kInternalForcedOutlineColor
    )
}

// cpp: longhands_custom.cc:2377-2439,4854-4915. The production API applies
// ordinary tree styles. Highlight originating styles and platform color
// providers require their actual resolver-state collaborators.
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
) -> Result {
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue() || v.IsUnsetValue() && !inherit || inherit && parent.is_none();
    let p = parent.filter(|_| inherit && !initial);
    if matches!(id, kWebkitTapHighlightColor | kWebkitTextFillColor) {
        // generated longhands.cc:19105-19113,19192-19201. Tap highlight
        // uses the native platform-default provider; text fill starts currentColor.
        if initial {
            if id == kWebkitTapHighlightColor {
                b.SetTapHighlightColor(&ComputedStyleInitialValues::InitialTapHighlightColor());
            } else {
                b.SetTextFillColor(&StyleColor::CurrentColor());
            }
            return Ok(());
        }
        if let Some(p) = p {
            return ApplyInherit(id, b, p);
        }
        return ApplyConvertedColor(id, b, &color_ui_application::ConvertStyleColorValue(id, v)?);
    }
    if matches!(id, kColor | kInternalVisitedColor) {
        let visited = id == kInternalVisitedColor;
        let current = matches!(v.Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kCurrentcolor);
        let inheriting = p.is_some() || current && parent.is_some();
        let color = if initial || current && parent.is_none() {
            b.InitialColorForColorScheme()
        } else if inheriting {
            if b.ShouldPreserveParentColor() {
                // This path needs ColorIncludingFallback/VisitedDependentColor
                // and real forced/link color providers; do not lose semantics.
                return Err(LonghandApplicationError::Unsupported(id));
            }
            let parent = parent.unwrap();
            if visited && parent.InsideLink() == EInsideLink::kInsideVisitedLink {
                parent.InternalVisitedColor().clone()
            } else {
                parent.Color().clone()
            }
        } else {
            color_ui_application::ConvertStyleColorValue(id, v)?
        };
        if visited {
            b.SetInternalVisitedColor(&color);
            b.SetInternalVisitedColorIsCurrentColor(
                current || p.is_some() && parent.unwrap().InternalVisitedColorIsCurrentColor(),
            );
        } else {
            b.SetColor(&color);
            b.SetColorIsInherited(inheriting || current);
            b.SetColorIsCurrentColor(
                current || p.is_some() && parent.unwrap().ColorIsCurrentColor(),
            );
        }
    } else if id == kInternalVisitedColumnRuleColor {
        // cpp: generated longhands.cc:1714-1722; converter.cc:2615-2682.
        let colors = if initial {
            ComputedStyleInitialValues::InitialInternalVisitedColumnRuleColor()
        } else if let Some(p) = p {
            p.InternalVisitedColumnRuleColor().clone()
        } else if let CSSValuePayload::kValueListClass(list) = v.Payload() {
            if list.values.is_empty() {
                return Err(LonghandApplicationError::InvalidValue(id));
            }
            if list.values.len() > 1 && b.InsideLink() != EInsideLink::kNotInsideLink {
                GapDataList::DefaultGapColorDataList()
            } else {
                let values = list
                    .values
                    .iter()
                    .map(|v| color_ui_application::ConvertStyleColorValue(id, v))
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                let mut colors = GapDataList::default();
                for color in values {
                    colors.AddGapData(&layoutng_style::style::gap_data::GapData::from_value(color));
                }
                colors
            }
        } else {
            GapDataList::from_value(&color_ui_application::ConvertStyleColorValue(id, v)?)
        };
        b.SetInternalVisitedColumnRuleColor(&colors);
    } else {
        // cpp: generated longhands.cc:1268-1278,1866-1996,9527-9646.
        // Visited text/background slots inherit the parent's ordinary slot.
        // Forced slots retain their own forced parent slots.
        let forced_current = matches!(id, kInternalForcedColor | kInternalForcedVisitedColor)
            && matches!(v.Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kCurrentcolor);
        let inherited = p.or(parent.filter(|_| forced_current));
        let color = if let Some(p) = inherited {
            match id {
                kBackgroundColor | kInternalVisitedBackgroundColor => p.BackgroundColor().clone(),
                kInternalVisitedTextDecorationColor => p.TextDecorationColor().clone(),
                kInternalVisitedTextEmphasisColor => p.TextEmphasisColor().clone(),
                kInternalVisitedTextFillColor => p.TextFillColor().clone(),
                kInternalVisitedTextStrokeColor => p.TextStrokeColor().clone(),
                kInternalForcedColor => p.InternalForcedColor().clone(),
                kInternalForcedVisitedColor => p.InternalForcedVisitedColor().clone(),
                kInternalForcedBackgroundColor => p.InternalForcedBackgroundColor().clone(),
                kInternalForcedBorderColor => p.InternalForcedBorderColor().clone(),
                kInternalForcedOutlineColor => p.InternalForcedOutlineColor().clone(),
                _ => return Err(LonghandApplicationError::Unsupported(id)),
            }
        } else if initial || forced_current {
            match id {
                kBackgroundColor | kInternalVisitedBackgroundColor => {
                    ComputedStyleInitialValues::InitialBackgroundColor()
                }
                kInternalForcedColor => ComputedStyleInitialValues::InitialInternalForcedColor(),
                kInternalForcedVisitedColor => {
                    ComputedStyleInitialValues::InitialInternalForcedVisitedColor()
                }
                kInternalForcedBackgroundColor => {
                    ComputedStyleInitialValues::InitialInternalForcedBackgroundColor()
                }
                _ => StyleColor::CurrentColor(),
            }
        } else {
            color_ui_application::ConvertStyleColorValue(id, v)?
        };
        match id {
            kBackgroundColor => b.SetBackgroundColor(&color),
            kInternalVisitedBackgroundColor => b.SetInternalVisitedBackgroundColor(&color),
            kInternalVisitedTextDecorationColor => b.SetInternalVisitedTextDecorationColor(&color),
            kInternalVisitedTextEmphasisColor => b.SetInternalVisitedTextEmphasisColor(&color),
            kInternalVisitedTextFillColor => b.SetInternalVisitedTextFillColor(&color),
            kInternalVisitedTextStrokeColor => b.SetInternalVisitedTextStrokeColor(&color),
            kInternalForcedColor => b.SetInternalForcedColor(&color),
            kInternalForcedVisitedColor => b.SetInternalForcedVisitedColor(&color),
            kInternalForcedBackgroundColor => b.SetInternalForcedBackgroundColor(&color),
            kInternalForcedBorderColor => b.SetInternalForcedBorderColor(&color),
            kInternalForcedOutlineColor => b.SetInternalForcedOutlineColor(&color),
            _ => return Err(LonghandApplicationError::Unsupported(id)),
        }
    }
    if inherit && !initial && v.IsInheritedValue() && !CSSProperty::Get(id).IsInherited() {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn apply(
        b: &mut ComputedStyleBuilder,
        p: Option<&ComputedStyle>,
        id: CSSPropertyID,
        css: &str,
    ) -> Result {
        let values = crate::parser::production_property_parser::ParseProperty(
            id,
            &foundation::String::from(css),
            false,
            crate::parser::css_parser_mode::CSSParserMode::kUASheetMode,
        )
        .unwrap();
        super::super::Apply(
            id,
            b,
            p,
            values[0].Value(),
            16.0,
            &MediaValuesCachedData::default(),
        )
    }
    #[test]
    fn color_root_inheritance_flags_and_visited_parent_selection() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut b = ComputedStyleBuilder::from_style(initial);
        b.SetDarkColorScheme(true);
        apply(&mut b, None, kColor, "inherit").unwrap();
        assert_eq!(b.Color().GetColor(), foundation::Color::kWhite);
        assert!(!b.ColorIsCurrentColor());
        apply(&mut b, None, kColor, "red").unwrap();
        apply(&mut b, None, kInternalVisitedColor, "blue").unwrap();
        let parent = unsafe { &*b.TakeStyle() };
        let mut child = ComputedStyleBuilder::from_style(initial);
        apply(&mut child, Some(parent), kColor, "currentcolor").unwrap();
        assert!(child.Color() == parent.Color());
        assert!(child.ColorIsCurrentColor());
        apply(&mut child, Some(parent), kInternalVisitedColor, "inherit").unwrap();
        assert!(child.InternalVisitedColor() == parent.Color());
        let mut visited = ComputedStyleBuilder::from_style(parent);
        visited.SetInsideLink(EInsideLink::kInsideVisitedLink);
        let visited = unsafe { &*visited.TakeStyle() };
        apply(
            &mut child,
            Some(visited),
            kInternalVisitedColor,
            "currentcolor",
        )
        .unwrap();
        assert!(child.InternalVisitedColor() == visited.InternalVisitedColor());
        assert!(child.InternalVisitedColorIsCurrentColor());
    }
    #[test]
    fn visited_slots_inherit_ordinary_colors_and_failure_preserves_native_value() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut parent = ComputedStyleBuilder::from_style(initial);
        parent.SetTextEmphasisColor(&StyleColor::from_color(foundation::Color::kWhite));
        parent.SetInternalVisitedTextEmphasisColor(&StyleColor::from_color(
            foundation::Color::kBlack,
        ));
        let parent = unsafe { &*parent.TakeStyle() };
        let mut b = ComputedStyleBuilder::from_style(initial);
        apply(
            &mut b,
            Some(parent),
            kInternalVisitedTextEmphasisColor,
            "inherit",
        )
        .unwrap();
        let mut expected = ComputedStyleBuilder::from_style(initial);
        expected.SetInternalVisitedTextEmphasisColor(parent.TextEmphasisColor());
        if !CSSProperty::Get(kInternalVisitedTextEmphasisColor).IsInherited() {
            expected.SetHasExplicitInheritance();
        }
        let actual = unsafe { &*b.TakeStyle() };
        let expected = unsafe { &*expected.TakeStyle() };
        assert!(actual == expected);
        let mut b = ComputedStyleBuilder::from_style(initial);
        apply(
            &mut b,
            Some(parent),
            kInternalVisitedTextEmphasisColor,
            "initial",
        )
        .unwrap();
        let mut expected = ComputedStyleBuilder::from_style(initial);
        expected.SetInternalVisitedTextEmphasisColor(&StyleColor::CurrentColor());
        let actual = unsafe { &*b.TakeStyle() };
        let expected = unsafe { &*expected.TakeStyle() };
        assert!(actual == expected);
        let mut b = ComputedStyleBuilder::from_style(initial);
        apply(&mut b, None, kInternalVisitedColumnRuleColor, "red").unwrap();
        assert_eq!(
            b.InternalVisitedColumnRuleColor()
                .GetSingleValue()
                .GetColor(),
            foundation::Color::FromRGBA(255, 0, 0, 255)
        );
        apply(&mut b, None, kInternalForcedColor, "blue").unwrap();
        let before = b.InternalForcedColor().clone();
        assert_eq!(
            apply(&mut b, None, kInternalForcedColor, "canvastext"),
            Err(LonghandApplicationError::Unsupported(kInternalForcedColor))
        );
        assert!(b.InternalForcedColor() == &before);
    }
}
