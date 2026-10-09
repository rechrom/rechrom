// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native stable grid values; track construction is shared with the converter.
#![allow(non_snake_case)]
use super::*;
use crate::production_css_value::ListSeparator;
use crate::resolver::position_repeat_application::ConvertLength;
use crate::resolver::style_builder_converter::StyleBuilderConverter;
use foundation::{MakeGarbageCollected, Member};
use layoutng_style::style::{
    computed_grid_template_areas::ComputedGridTemplateAreas,
    computed_style_constants::GridAutoFlow, grid_track_size::GridTrackSize,
};

pub(super) fn IsGridProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kGridTemplateAreas
            | CSSPropertyID::kGridTemplateRows
            | CSSPropertyID::kGridTemplateColumns
            | CSSPropertyID::kGridAutoRows
            | CSSPropertyID::kGridAutoColumns
            | CSSPropertyID::kGridAutoFlow
    )
}
fn List(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<&[std::rc::Rc<Value>], LonghandApplicationError> {
    match v.Payload() {
        CSSValuePayload::kValueListClass(list)
            if list.separator == ListSeparator::Space && !list.values.is_empty() =>
        {
            Ok(&list.values)
        }
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}
// cpp: style_builder_converter.cc ConvertGridTrackBreadth:123-143.
fn Track(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> std::result::Result<GridTrackSize, LonghandApplicationError> {
    let breadth = match v.Payload() {
        CSSValuePayload::kIdentifierClass(k) => match k.0 {
            CSSValueID::kAuto => Length::Auto().clone(),
            CSSValueID::kMinContent => Length::MinContent().clone(),
            CSSValueID::kMaxContent => Length::MaxContent().clone(),
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        },
        CSSValuePayload::kNumericLiteralClass(n) if n.GetType() == UnitType::kFlex => {
            if n.DoubleValue() < 0.0 {
                return Err(LonghandApplicationError::InvalidValue(id));
            }
            Length::Flex(
                crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(n.DoubleValue())
                    as f32,
            )
        }
        CSSValuePayload::kNumericLiteralClass(_) | CSSValuePayload::kMathFunctionClass(_) => {
            ConvertLength(id, v, b, root, media)?
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    };
    Ok(GridTrackSize::from_length(&breadth))
}
// cpp: style_builder_converter.cc:1490-1532. Typed first/second identifiers
// retain the canonical dense-only row spelling used by the source parser.
fn Flow(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<GridAutoFlow, LonghandApplicationError> {
    let items = List(id, v)?;
    if items.len() > 2 {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let ident = |v: &Value| match v.Payload() {
        CSSValuePayload::kIdentifierClass(k) => Ok(k.0),
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    };
    let first = ident(&items[0])?;
    let second = items.get(1).map(|v| ident(v)).transpose()?;
    Ok(match (first, second) {
        (CSSValueID::kRow, None) => GridAutoFlow::kAutoFlowRow,
        (CSSValueID::kColumn, None) => GridAutoFlow::kAutoFlowColumn,
        (CSSValueID::kRow, Some(CSSValueID::kDense)) | (CSSValueID::kDense, None) => {
            GridAutoFlow::kAutoFlowRowDense
        }
        (CSSValueID::kColumn, Some(CSSValueID::kDense)) => GridAutoFlow::kAutoFlowColumnDense,
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
    inherit: bool,
    initial: bool,
) -> Result {
    use CSSPropertyID::*;
    if initial {
        return ApplyInitial(id, b);
    }
    if inherit {
        let p = parent.unwrap();
        if v.IsInheritedValue() {
            b.SetHasExplicitInheritance();
            p.SetChildHasExplicitInheritance();
        }
        if matches!(
            id,
            kGridTemplateRows | kGridTemplateColumns | kGridAutoRows | kGridAutoColumns
        ) {
            if b.EffectiveZoom() != p.EffectiveZoom() {
                return Err(LonghandApplicationError::Unsupported(id));
            }
            match id {
                kGridTemplateRows => b.SetGridTemplateRows(p.SpecifiedGridTemplateRows()),
                kGridTemplateColumns => b.SetGridTemplateColumns(p.SpecifiedGridTemplateColumns()),
                kGridAutoRows => b.SetGridAutoRows(p.GridAutoRows()),
                kGridAutoColumns => b.SetGridAutoColumns(p.GridAutoColumns()),
                _ => unreachable!(),
            }
            return Ok(());
        }
        return ApplyInherit(id, b, p);
    }
    match id {
        // cpp: style_builder_converter.cc:1602-1616; generated longhands.cc:8670.
        kGridTemplateAreas => {
            let areas = match v.Payload() {
                CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kNone => {
                    Member::default()
                }
                CSSValuePayload::kGridTemplateAreasClass(areas) => {
                    Member::from_ptr(MakeGarbageCollected(ComputedGridTemplateAreas::new(
                        &areas.grid_area_map,
                        areas.row_count,
                        areas.column_count,
                    )))
                }
                _ => return Err(LonghandApplicationError::InvalidValue(id)),
            };
            b.SetGridTemplateAreasOwned(areas);
            Ok(())
        }
        kGridAutoFlow => {
            b.SetGridAutoFlow(Flow(id, v)?);
            Ok(())
        }
        kGridTemplateRows | kGridTemplateColumns => {
            let value = if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kNone)
            {
                Member::default()
            } else {
                let sizes = List(id, v)?
                    .iter()
                    .map(|v| Track(id, b, v, root, media))
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Member::from_ptr(MakeGarbageCollected(
                    StyleBuilderConverter::ConvertPlainGridTrackListFromSizes(&sizes),
                ))
            };
            if id == kGridTemplateRows {
                b.SetGridTemplateRowsOwned(value);
            } else {
                b.SetGridTemplateColumnsOwned(value);
            }
            Ok(())
        }
        kGridAutoRows | kGridAutoColumns => {
            let sizes = if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kAuto)
            {
                vec![GridTrackSize::from_length(Length::Auto())]
            } else {
                List(id, v)?
                    .iter()
                    .map(|v| Track(id, b, v, root, media))
                    .collect::<std::result::Result<Vec<_>, _>>()?
            };
            let value = StyleBuilderConverter::ConvertGridTrackSizeListFromSizes(&sizes);
            if id == kGridAutoRows {
                b.SetGridAutoRowsOwned(value);
            } else {
                b.SetGridAutoColumnsOwned(value);
            }
            Ok(())
        }
        _ => Err(LonghandApplicationError::Unsupported(id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseDeclarationList, ParseProperty, PropertyParseErrorKind},
    };
    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn apply(b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, css: &str) {
        let parsed = ParseDeclarationList(
            &foundation::String::FromUtf8(css.as_bytes()),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        assert!(!parsed.properties.is_empty());
        for p in parsed.properties {
            super::super::Apply(
                p.PropertyID(),
                b,
                parent,
                p.Value(),
                16.0,
                &MediaValuesCachedData::default(),
            )
            .unwrap();
        }
    }
    fn flow(b: &ComputedStyleBuilder) -> GridAutoFlow {
        unsafe { &*b.CloneStyle() }.GetGridAutoFlow()
    }
    fn rows(b: &ComputedStyleBuilder) -> &layoutng_style::style::grid_track_list::GridTrackList {
        unsafe { &*b.SpecifiedGridTemplateRows().Get() }.GetTrackList()
    }
    fn columns(b: &ComputedStyleBuilder) -> &layoutng_style::style::grid_track_list::GridTrackList {
        unsafe { &*b.SpecifiedGridTemplateColumns().Get() }.GetTrackList()
    }
    #[test]
    fn production_grid_areas_rectangles_native_spans_and_implicit_lines() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            None,
            r#"grid-template-areas:"head head head" "side main main" "side main main""#,
        );
        let areas = unsafe { &*b.GridTemplateAreas().Get() };
        assert_eq!((areas.row_count, areas.column_count), (3, 3));
        let main = &areas.named_areas[&foundation::String::from("main")];
        assert_eq!(
            (
                main.rows.StartLine(),
                main.rows.EndLine(),
                main.columns.StartLine(),
                main.columns.EndLine()
            ),
            (1, 3, 1, 3)
        );
        assert_eq!(
            areas.implicit_named_grid_row_lines[&foundation::String::from("side-start")],
            vec![1]
        );
        assert_eq!(
            areas.implicit_named_grid_row_lines[&foundation::String::from("side-end")],
            vec![3]
        );
        assert_eq!(
            areas.implicit_named_grid_column_lines[&foundation::String::from("main-start")],
            vec![1]
        );
        // Adjacent name/dots form distinct cells; repeated dots form one null cell.
        apply(&mut b, None, r#"grid-template-areas:"λ...123" "λ...123""#);
        let areas = unsafe { &*b.GridTemplateAreas().Get() };
        assert_eq!((areas.row_count, areas.column_count), (2, 3));
        assert_eq!(areas.named_areas.len(), 2);
        let parsed = ParseProperty(
            CSSPropertyID::kGridTemplateAreas,
            &foundation::String::FromUtf8(r#""λ...123" "λ...123""#.as_bytes()),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(parsed[0].Value().CssText().Utf8(), r#""λ . 123" "λ . 123""#);
        apply(&mut b, None, "grid-template-areas:none");
        assert!(b.GridTemplateAreas().Get().is_null());
    }
    #[test]
    fn production_grid_shorthands_build_tracks_and_reset_explicit_implicit_fields() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            None,
            "grid-auto-flow:column dense;grid-auto-rows:17px;grid-auto-columns:2fr",
        );
        apply(
            &mut b,
            None,
            r#"grid-template:"a a" calc(10px + 10%) "b c" / 1fr 25%"#,
        );
        assert_eq!(flow(&b), GridAutoFlow::kAutoFlowColumnDense);
        assert_eq!(
            b.GridAutoRows()
                .RepeatTrackSize(0, 0)
                .MinTrackBreadth()
                .Pixels(),
            17.0
        );
        assert_eq!(rows(&b).TrackCountWithoutAutoRepeat(), 2);
        assert!(rows(&b)
            .RepeatTrackSize(0, 0)
            .MinTrackBreadth()
            .IsCalculated());
        assert!(rows(&b).RepeatTrackSize(1, 0).MinTrackBreadth().IsAuto());
        assert_eq!(
            columns(&b)
                .RepeatTrackSize(0, 0)
                .MaxTrackBreadth()
                .FlexValue(),
            1.0
        );
        assert_eq!(
            columns(&b)
                .RepeatTrackSize(1, 0)
                .MinTrackBreadth()
                .PercentValue(),
            25.0
        );
        apply(&mut b, None, "grid:20px 30px / 1fr 2fr");
        assert!(b.GridTemplateAreas().Get().is_null());
        assert_eq!(flow(&b), GridAutoFlow::kAutoFlowRow);
        assert!(b
            .GridAutoRows()
            .RepeatTrackSize(0, 0)
            .MinTrackBreadth()
            .IsAuto());
        assert!(b
            .GridAutoColumns()
            .RepeatTrackSize(0, 0)
            .MinTrackBreadth()
            .IsAuto());
        apply(&mut b, None, "grid:dense auto-flow 8px 12px / 1fr 40px");
        assert!(b.SpecifiedGridTemplateRows().Get().is_null());
        assert_eq!(columns(&b).TrackCountWithoutAutoRepeat(), 2);
        assert_eq!(flow(&b), GridAutoFlow::kAutoFlowRowDense);
        assert_eq!(b.GridAutoRows().RepeatSize(0), 2);
        assert_eq!(
            b.GridAutoRows()
                .RepeatTrackSize(0, 1)
                .MinTrackBreadth()
                .Pixels(),
            12.0
        );
        apply(&mut b, None, "grid:none / auto-flow dense 2fr");
        assert!(
            b.SpecifiedGridTemplateRows().Get().is_null()
                && b.SpecifiedGridTemplateColumns().Get().is_null()
        );
        assert_eq!(flow(&b), GridAutoFlow::kAutoFlowColumnDense);
        assert_eq!(
            b.GridAutoColumns()
                .RepeatTrackSize(0, 0)
                .MaxTrackBreadth()
                .FlexValue(),
            2.0
        );
        apply(&mut b, None, "grid:auto-flow / none");
        assert_eq!(flow(&b), GridAutoFlow::kAutoFlowRow);
        assert!(b
            .GridAutoRows()
            .RepeatTrackSize(0, 0)
            .MinTrackBreadth()
            .IsAuto());
        let expanded = ParseProperty(
            CSSPropertyID::kGrid,
            &foundation::String::from("none"),
            true,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(expanded.len(), 6);
        assert!(expanded.iter().all(|p| p.IsImportant()));
        assert!(expanded
            .iter()
            .filter(|p| matches!(
                p.PropertyID(),
                CSSPropertyID::kGridAutoRows
                    | CSSPropertyID::kGridAutoColumns
                    | CSSPropertyID::kGridAutoFlow
            ))
            .all(|p| p.IsImplicit()));
    }
    #[test]
    fn production_grid_css_wide_uses_native_inherit_and_initial_reset() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut p,
            None,
            r#"grid:"a a" 30px / 1fr 2fr;grid-auto-flow:column dense;grid-auto-rows:4px;grid-auto-columns:5px"#,
        );
        let p = unsafe { &*p.TakeStyle() };
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b, Some(p), "grid:inherit");
        assert!(b.HasExplicitInheritance());
        assert_eq!(flow(&b), GridAutoFlow::kAutoFlowColumnDense);
        assert_eq!(b.GridTemplateAreas().Get(), p.GridTemplateAreas().Get());
        assert_eq!(
            b.SpecifiedGridTemplateColumns().Get(),
            p.SpecifiedGridTemplateColumns().Get()
        );
        assert_eq!(
            b.GridAutoRows()
                .RepeatTrackSize(0, 0)
                .MinTrackBreadth()
                .Pixels(),
            4.0
        );
        apply(&mut b, Some(p), "grid-template:unset");
        assert!(
            b.GridTemplateAreas().Get().is_null()
                && b.SpecifiedGridTemplateRows().Get().is_null()
                && b.SpecifiedGridTemplateColumns().Get().is_null()
        );
        assert_eq!(flow(&b), GridAutoFlow::kAutoFlowColumnDense);
        apply(&mut b, Some(p), "grid:initial");
        assert_eq!(flow(&b), GridAutoFlow::kAutoFlowRow);
        assert!(b
            .GridAutoRows()
            .RepeatTrackSize(0, 0)
            .MinTrackBreadth()
            .IsAuto());
        assert!(!p.GridTemplateAreas().Get().is_null());
        b.SetEffectiveZoom(2.0);
        let inherited = ParseProperty(
            CSSPropertyID::kGridTemplateRows,
            &foundation::String::from("inherit"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(
            super::super::Apply(
                CSSPropertyID::kGridTemplateRows,
                &mut b,
                Some(p),
                inherited[0].Value(),
                16.0,
                &MediaValuesCachedData::default()
            ),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kGridTemplateRows
            ))
        );
    }
    #[test]
    fn production_grid_invalid_area_and_shorthand_grammars_remain_rejected() {
        for (id, css, kind) in [
            (
                CSSPropertyID::kGridTemplateAreas,
                r#""""#,
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kGridTemplateAreas,
                r#""a a" "a .""#,
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kGridTemplateAreas,
                r#""a . a""#,
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kGridTemplateAreas,
                r#""a" "." "a""#,
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kGridTemplateAreas,
                r#""a b" "c""#,
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kGridTemplateAreas,
                r#""a!""#,
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kGridTemplate,
                "10px",
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kGrid,
                "auto-flow dense / auto-flow",
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kGridTemplate,
                r#""a" / repeat(2,1fr)"#,
                PropertyParseErrorKind::Invalid,
            ),
            (
                CSSPropertyID::kGridTemplate,
                "[line] 1fr / 1fr",
                PropertyParseErrorKind::Unsupported,
            ),
            (
                CSSPropertyID::kGridTemplate,
                "repeat(2,1fr) / 1fr",
                PropertyParseErrorKind::Unsupported,
            ),
            (
                CSSPropertyID::kGridTemplate,
                "subgrid / 1fr",
                PropertyParseErrorKind::Unsupported,
            ),
            (
                CSSPropertyID::kGridTemplate,
                "minmax(0,1fr) / 1fr",
                PropertyParseErrorKind::Unsupported,
            ),
        ] {
            let error = ParseProperty(
                id,
                &foundation::String::from(css),
                false,
                CSSParserMode::kHTMLStandardMode,
            )
            .err()
            .unwrap_or_else(|| panic!("accepted {css}"));
            assert_eq!(error.kind, kind, "{css}");
        }
    }
}
