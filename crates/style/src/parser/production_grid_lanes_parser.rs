// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! css_parsing_utils.cc:7673-7857; custom direction/shorthand parse functions.
#![allow(non_snake_case)]
use super::*;
use layoutng_style::style::grid_area::NamedGridAreaMap;
pub(super) fn IsShorthand(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kGridLanes | CSSPropertyID::kRuleBreak | CSSPropertyID::kRuleVisibilityItems
    )
}
pub(super) fn Direction<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    let orientation = s.Peek().Id();
    if !matches!(
        orientation,
        CSSValueID::kNormal | CSSValueID::kRow | CSSValueID::kColumn
    ) {
        return Err(invalid(id));
    }
    let first = values::identifier(s.ConsumeIncludingWhitespace().Id());
    if orientation == CSSValueID::kNormal {
        return Ok(first);
    }
    let mut items = vec![first];
    let mut previous = None;
    for _ in 0..2 {
        let reverse = s.Peek().Id();
        if !matches!(
            reverse,
            CSSValueID::kFillReverse | CSSValueID::kTrackReverse
        ) || previous == Some(reverse)
        {
            break;
        }
        previous = Some(reverse);
        items.push(values::identifier(s.ConsumeIncludingWhitespace().Id()));
    }
    Ok(values::list(items, values::ListSeparator::Space))
}
pub(super) fn ConsumeDirection<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    let v = Direction(id, s)?;
    if !at_value_end(s) {
        return Err(invalid(id));
    }
    Ok(v)
}
fn Areas(id: CSSPropertyID, text: &String, columns: bool) -> Result<Rc<Value>, PropertyParseError> {
    if text
        .Span16()
        .unwrap_or_default()
        .iter()
        .all(|&c| crate::parser::css_parser_idioms::IsCSSSpace(c))
    {
        return Ok(values::identifier(CSSValueID::kNone));
    }
    let mut map = NamedGridAreaMap::default();
    let mut row_count = 0;
    let mut column_count = 0;
    let rows = if columns {
        vec![text.clone()]
    } else {
        grid_parser::ColumnNames(text)
    };
    for row in rows {
        if !grid_parser::ParseAreasRow(&row, &mut map, row_count, &mut column_count) {
            return Err(invalid(id));
        }
        row_count += 1;
    }
    if row_count == 0 {
        return Err(invalid(id));
    }
    Ok(grid_parser::Areas(map, row_count, column_count))
}
pub(super) fn Expand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    use CSSPropertyID::*;
    if id != kGridLanes {
        let (fields, words): ([CSSPropertyID; 2], &[&str]) = if id == kRuleBreak {
            (
                [kColumnRuleBreak, kRowRuleBreak],
                &["none", "intersection", "normal"],
            )
        } else {
            (
                [kColumnRuleVisibilityItems, kRowRuleVisibilityItems],
                &["normal", "all", "around", "between"],
            )
        };
        let v = ConsumeLiteral(id, s, mode, Grammar::Keywords(words))?;
        for f in fields {
            out.push(make_expanded(f, id, v.clone(), false));
        }
        return Ok(());
    }
    let mut area_text = String::from("");
    let mut direction = None;
    let mut tracks = None;
    let mut for_columns = true;
    loop {
        if area_text.length() == 0 && s.Peek().GetType() == kStringToken {
            area_text = s.ConsumeIncludingWhitespace().Value().ToString();
            if at_value_end(s) {
                break;
            }
            continue;
        }
        if direction.is_none()
            && matches!(
                s.Peek().Id(),
                CSSValueID::kNormal | CSSValueID::kRow | CSSValueID::kColumn
            )
        {
            for_columns = s.Peek().Id() != CSSValueID::kRow;
            direction = Some(Direction(id, s)?);
            if at_value_end(s) {
                break;
            }
            continue;
        }
        if tracks.is_none() {
            s.EnsureLookAhead();
            let save = s.Save();
            match grid_parser::ConsumeTracksWithGridLanes(id, s, mode, true, false, true) {
                Ok(v) => {
                    tracks = Some(v);
                    if at_value_end(s) {
                        break;
                    }
                    continue;
                }
                Err(e) => {
                    s.Peek();
                    s.Restore(save);
                    if e.kind == PropertyParseErrorKind::Unsupported {
                        return Err(e);
                    }
                }
            }
        }
        break;
    }
    if !at_value_end(s) {
        return Err(invalid(id));
    }
    let direction = direction.unwrap_or_else(|| {
        values::list(
            vec![values::identifier(CSSValueID::kColumn)],
            values::ListSeparator::Space,
        )
    });
    let none = values::identifier(CSSValueID::kNone);
    let tracks = tracks.unwrap_or_else(|| none.clone());
    let areas = Areas(id, &area_text, for_columns)?;
    for (f, v) in [
        (kGridTemplateAreas, areas),
        (
            kGridTemplateColumns,
            if for_columns {
                tracks.clone()
            } else {
                none.clone()
            },
        ),
        (kGridTemplateRows, if for_columns { none } else { tracks }),
        (kGridLanesDirection, direction),
    ] {
        out.push(make_expanded(f, id, v, false));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media_queries::MediaValuesCachedData;
    use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn hidden(text: &str) -> Result<Vec<PropertyValue>, PropertyParseError> {
        let text = String::from(text);
        let mut s: Stream = Stream::new(StringView::from(&text), 0);
        let mut out = Vec::new();
        Expand(
            CSSPropertyID::kGridLanes,
            &mut s,
            CSSParserMode::kHTMLStandardMode,
            &mut out,
        )?;
        Ok(out)
    }
    fn native_templates(b: &mut ComputedStyleBuilder, properties: &[PropertyValue]) {
        for p in properties {
            if p.PropertyID() != CSSPropertyID::kGridLanesDirection {
                crate::resolver::production_style_builder::Apply(
                    p.PropertyID(),
                    b,
                    None,
                    p.Value(),
                    16.0,
                    &MediaValuesCachedData::default(),
                )
                .unwrap();
            }
        }
    }
    #[test]
    fn hidden_grid_lanes_typed_tracks_areas_and_transpose_use_native_grid_owners() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        for text in [
            r#"column fill-reverse 10px 1fr "a a b""#,
            r#""a a b" 10px 1fr column fill-reverse"#,
            r#"10px 1fr column fill-reverse "a a b""#,
        ] {
            let p = hidden(text).unwrap();
            assert_eq!(p.len(), 4);
            native_templates(&mut b, &p);
            let a = unsafe { &*b.GridTemplateAreas().Get() };
            assert_eq!((a.row_count, a.column_count), (1, 3));
            assert_eq!(a.named_areas[&String::from("a")].columns.EndLine(), 2);
            assert!(b.SpecifiedGridTemplateRows().Get().is_null());
            let tracks = unsafe { &*b.SpecifiedGridTemplateColumns().Get() }.GetTrackList();
            assert_eq!(tracks.TrackCountWithoutAutoRepeat(), 2);
            assert_eq!(
                tracks.RepeatTrackSize(0, 0).MinTrackBreadth().Pixels(),
                10.0
            );
            assert!(tracks.RepeatTrackSize(1, 0).MinTrackBreadth().IsFlex());
            assert_eq!(p[3].Value().CssText().Utf8(), "column fill-reverse");
        }
        let p = hidden(r#"row track-reverse "a a b" 20px 2fr"#).unwrap();
        native_templates(&mut b, &p);
        let a = unsafe { &*b.GridTemplateAreas().Get() };
        assert_eq!((a.row_count, a.column_count), (3, 1));
        assert_eq!(a.named_areas[&String::from("a")].rows.EndLine(), 2);
        assert!(b.SpecifiedGridTemplateColumns().Get().is_null());
        assert_eq!(
            unsafe { &*b.SpecifiedGridTemplateRows().Get() }
                .GetTrackList()
                .TrackCountWithoutAutoRepeat(),
            2
        );
        let p = hidden("none").unwrap();
        native_templates(&mut b, &p);
        assert!(b.GridTemplateAreas().Get().is_null());
        assert!(b.SpecifiedGridTemplateColumns().Get().is_null());
        assert!(b.SpecifiedGridTemplateRows().Get().is_null());
        assert_eq!(p[3].Value().CssText().Utf8(), "column");
    }
    #[test]
    fn hidden_grid_lanes_direction_and_invalid_tracks_match_source() {
        for text in [
            "normal",
            "row",
            "column fill-reverse track-reverse",
            "row track-reverse fill-reverse",
        ] {
            let text = String::from(text);
            let mut s: Stream = Stream::new(StringView::from(&text), 0);
            assert!(ConsumeDirection(CSSPropertyID::kGridLanesDirection, &mut s).is_ok());
        }
        for text in [
            "fill-reverse",
            "normal fill-reverse",
            "row fill-reverse fill-reverse",
            "column row",
            "row track-reverse track-reverse",
        ] {
            let text = String::from(text);
            let mut s: Stream = Stream::new(StringView::from(&text), 0);
            assert!(ConsumeDirection(CSSPropertyID::kGridLanesDirection, &mut s).is_err());
        }
        for text in [
            r#"row "a b a""#,
            r#"column "a b a""#,
            r#""a" "b""#,
            "row column",
            "10px / 20px",
            "-1fr",
            "row fill-reverse fill-reverse",
        ] {
            assert!(hidden(text).is_err(), "{text}");
        }
        for text in ["repeat(2,10px)", "[a] 10px", "subgrid"] {
            assert_eq!(
                hidden(text).err().unwrap().kind,
                PropertyParseErrorKind::Unsupported
            );
        }
    }
    #[test]
    fn production_grid_lanes_all_entries_keep_experimental_gate_hidden() {
        for id in [
            CSSPropertyID::kGridLanes,
            CSSPropertyID::kGridLanesDirection,
            CSSPropertyID::kGridLanesPack,
        ] {
            for text in ["normal", "initial", "inherit", "var(--x)"] {
                assert_eq!(
                    ParseProperty(
                        id,
                        &String::from(text),
                        false,
                        CSSParserMode::kHTMLStandardMode
                    )
                    .err()
                    .unwrap()
                    .kind,
                    PropertyParseErrorKind::Unsupported
                );
            }
        }
        let data = ParseVariableData(&String::from("normal"), true, true)
            .unwrap()
            .0;
        assert_eq!(
            ParsePropertyTokens(
                CSSPropertyID::kGridLanes,
                &data,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Unsupported
        );
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        let v = values::wide(CSSValueID::kInitial).unwrap();
        let id = CSSPropertyID::kGridLanesDirection;
        assert_eq!(
            crate::resolver::production_style_builder::Apply(
                id,
                &mut b,
                None,
                &v,
                16.0,
                &MediaValuesCachedData::default()
            ),
            Err(crate::properties::longhand_dispatch::LonghandApplicationError::Unsupported(id))
        );
        assert_eq!(
            crate::properties::longhand_dispatch::ApplyInitial(id, &mut b),
            Err(crate::properties::longhand_dispatch::LonghandApplicationError::Unsupported(id))
        );
        assert_eq!(
            crate::properties::longhand_dispatch::ApplyInherit(id, &mut b, initial()),
            Err(crate::properties::longhand_dispatch::LonghandApplicationError::Unsupported(id))
        );
    }
}
