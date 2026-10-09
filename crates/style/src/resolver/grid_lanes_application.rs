// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! style_builder_converter.cc:1846-1895; genuine native GridLanesDirection.
//! Public production entry is hidden by the CSSGridLanesLayout stable default.
#![allow(non_snake_case)]
use super::*;
use layoutng_style::style::grid_lanes_direction::GridLanesDirection;
fn Direction(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<GridLanesDirection, LonghandApplicationError> {
    if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kNormal) {
        return Ok(GridLanesDirection::default());
    }
    let CSSValuePayload::kValueListClass(l) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if l.separator != crate::production_css_value::ListSeparator::Space
        || l.values.is_empty()
        || l.values.len() > 3
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let key = |v: &Value| {
        if let CSSValuePayload::kIdentifierClass(k) = v.Payload() {
            Ok(k.0)
        } else {
            Err(LonghandApplicationError::InvalidValue(id))
        }
    };
    let identifiers = l
        .values
        .iter()
        .map(|v| key(v))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    crate::resolver::style_builder_converter::ConvertGridLanesDirectionIdentifiers(&identifiers)
        .ok_or(LonghandApplicationError::InvalidValue(id))
}
pub(super) fn ApplyInternal(
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
) -> Result {
    let id = CSSPropertyID::kGridLanesDirection;
    let inherit = v.IsInheritedValue();
    let initial = v.IsInitialValue() || v.IsUnsetValue() || inherit && parent.is_none();
    let direction = if initial {
        ComputedStyleInitialValues::InitialGridLanesDirection()
    } else if inherit {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
        *parent.unwrap().GetGridLanesDirection()
    } else {
        Direction(id, v)?
    };
    b.SetGridLanesDirection(&direction);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseDeclarationList, ParseProperty},
    };
    use foundation::{RuleBreak, RuleVisibilityItems};
    use layoutng_style::style::grid_lanes_direction::GridLanesOrientation;
    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn apply(b: &mut ComputedStyleBuilder, p: Option<&ComputedStyle>, css: &str) {
        let d = ParseDeclarationList(
            &foundation::String::from(css),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(d.errors.is_empty(), "{:?}", d.errors);
        for v in d.properties {
            super::super::Apply(
                v.PropertyID(),
                b,
                p,
                v.Value(),
                16.0,
                &MediaValuesCachedData::default(),
            )
            .unwrap();
        }
    }
    #[test]
    fn hidden_grid_lanes_direction_native_flags_and_css_wide_use_real_owner() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        let v = crate::production_css_value::list(
            vec![
                CSSValueID::kRow,
                CSSValueID::kTrackReverse,
                CSSValueID::kFillReverse,
            ]
            .into_iter()
            .map(crate::production_css_value::identifier)
            .collect(),
            crate::production_css_value::ListSeparator::Space,
        );
        ApplyInternal(&mut b, None, &v).unwrap();
        let d = b.GetGridLanesDirection();
        assert_eq!(d.orientation, GridLanesOrientation::kRow);
        assert!(d.is_track_reverse && d.is_fill_reverse);
        let p = unsafe { &*b.TakeStyle() };
        let mut b = ComputedStyleBuilder::from_style(initial());
        ApplyInternal(
            &mut b,
            Some(p),
            &crate::production_css_value::wide(CSSValueID::kInherit).unwrap(),
        )
        .unwrap();
        assert_eq!(b.GetGridLanesDirection(), p.GetGridLanesDirection());
        assert!(b.HasExplicitInheritance());
        ApplyInternal(
            &mut b,
            Some(p),
            &crate::production_css_value::wide(CSSValueID::kUnset).unwrap(),
        )
        .unwrap();
        assert_eq!(b.GetGridLanesDirection(), &GridLanesDirection::default());
        assert!(p.GetGridLanesDirection().is_fill_reverse);
    }
    #[test]
    fn production_grid_lanes_related_stable_rule_keywords_shorthands_and_native_bits() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            None,
            "rule-break:intersection;rule-visibility-items:between",
        );
        assert_eq!(b.ColumnRuleBreak(), RuleBreak::kIntersection);
        assert_eq!(b.RowRuleBreak(), RuleBreak::kIntersection);
        assert_eq!(b.ColumnRuleVisibilityItems(), RuleVisibilityItems::kBetween);
        assert_eq!(b.RowRuleVisibilityItems(), RuleVisibilityItems::kBetween);
        apply(&mut b,None,"column-rule-break:none;row-rule-break:normal;column-rule-visibility-items:all;row-rule-visibility-items:around");
        assert_eq!(b.ColumnRuleBreak(), RuleBreak::kNone);
        assert_eq!(b.RowRuleBreak(), RuleBreak::kNormal);
        assert_eq!(b.ColumnRuleVisibilityItems(), RuleVisibilityItems::kAll);
        assert_eq!(b.RowRuleVisibilityItems(), RuleVisibilityItems::kAround);
        let p = unsafe { &*b.TakeStyle() };
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            Some(p),
            "rule-break:inherit;rule-visibility-items:inherit",
        );
        assert_eq!(b.ColumnRuleBreak(), p.ColumnRuleBreak());
        assert_eq!(b.RowRuleVisibilityItems(), p.RowRuleVisibilityItems());
        apply(
            &mut b,
            Some(p),
            "rule-break:unset;rule-visibility-items:initial",
        );
        assert_eq!(b.ColumnRuleBreak(), RuleBreak::kNormal);
        assert_eq!(b.RowRuleBreak(), RuleBreak::kNormal);
        assert_eq!(b.ColumnRuleVisibilityItems(), RuleVisibilityItems::kNormal);
        assert_eq!(b.RowRuleVisibilityItems(), RuleVisibilityItems::kNormal);
        for (id, text) in [
            (CSSPropertyID::kRuleBreak, "none intersection"),
            (CSSPropertyID::kRuleBreak, "all"),
            (CSSPropertyID::kRuleVisibilityItems, "none"),
            (CSSPropertyID::kRuleVisibilityItems, "around between"),
        ] {
            assert!(ParseProperty(
                id,
                &foundation::String::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .is_err());
        }
    }
}
