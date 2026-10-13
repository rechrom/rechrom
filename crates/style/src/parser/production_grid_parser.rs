// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Stable grid area and shorthand consumers over the existing typed tracks.
#![allow(non_snake_case)]
use super::*;
use crate::parser::css_parser_idioms::{IsCSSSpace, IsNameCodePoint};
use layoutng_style::style::grid_area::{GridArea, GridSpan, NamedGridAreaMap};

fn NoneValue() -> Rc<Value> {
    values::identifier(CSSValueID::kNone)
}
fn AutoValue() -> Rc<Value> {
    values::identifier(CSSValueID::kAuto)
}
fn Slash<T: TokenStreamTokenizer>(stream: &mut Stream<T>) -> bool {
    if stream.Peek().GetType() == kDelimiterToken && stream.Peek().Delimiter() == b'/' as u16 {
        stream.ConsumeIncludingWhitespace();
        true
    } else {
        false
    }
}
fn AtSlash<T: TokenStreamTokenizer>(stream: &mut Stream<T>) -> bool {
    stream.Peek().GetType() == kDelimiterToken && stream.Peek().Delimiter() == b'/' as u16
}
// cpp: css_parsing_utils.cc:6863-6901. Scan the decoded CSS string's name
// code points exactly; this is the source area-name tokenization operation.
pub(super) fn ColumnNames(row: &String) -> Vec<String> {
    let mut columns = Vec::new();
    let mut name = Vec::<u16>::new();
    for &c in row.Span16().unwrap_or_default() {
        if IsCSSSpace(c) {
            if !name.is_empty() {
                columns.push(String::from_utf16(&name));
                name.clear();
            }
            continue;
        }
        if c == b'.' as u16 {
            if name == [b'.' as u16] {
                continue;
            }
            if !name.is_empty() {
                columns.push(String::from_utf16(&name));
                name.clear();
            }
        } else {
            if !IsNameCodePoint(c) {
                return Vec::new();
            }
            if name == [b'.' as u16] {
                columns.push(String::from_utf16(&name));
                name.clear();
            }
        }
        name.push(c);
    }
    if !name.is_empty() {
        columns.push(String::from_utf16(&name));
    }
    columns
}
// cpp: css_parsing_utils.cc:7484-7557. Each repeated name must extend an
// adjacent row with identical start/end columns, producing one filled rectangle.
pub(super) fn ParseAreasRow(
    row: &String,
    map: &mut NamedGridAreaMap,
    rows: u32,
    columns: &mut u32,
) -> bool {
    let names = ColumnNames(row);
    if rows == 0 {
        *columns = names.len() as u32;
        if *columns == 0 {
            return false;
        }
    } else if *columns != names.len() as u32 {
        return false;
    }
    let mut column = 0;
    while column < *columns {
        let name = &names[column as usize];
        if name == &String::from(".") {
            column += 1;
            continue;
        }
        let mut end = column + 1;
        while end < *columns && names[end as usize] == *name {
            end += 1;
        }
        if let Some(area) = map.get_mut(name) {
            if rows != area.rows.EndLine()
                || column != area.columns.StartLine()
                || end != area.columns.EndLine()
            {
                return false;
            }
            area.rows = GridSpan::TranslatedDefiniteGridSpan(
                area.rows.StartLine(),
                area.rows.EndLine() + 1,
            );
        } else {
            map.insert(
                name.clone(),
                GridArea {
                    rows: GridSpan::TranslatedDefiniteGridSpan(rows, rows + 1),
                    columns: GridSpan::TranslatedDefiniteGridSpan(column, end),
                },
            );
        }
        column = end;
    }
    true
}
pub(super) fn Areas(map: NamedGridAreaMap, rows: u32, columns: u32) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kGridTemplateAreasClass(
        values::CSSGridTemplateAreasValue {
            grid_area_map: map,
            row_count: rows,
            column_count: columns,
        },
    )))
}
pub(super) fn ConsumeAreas<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kNone {
        stream.ConsumeIncludingWhitespace();
        return Ok(NoneValue());
    }
    let mut map = NamedGridAreaMap::default();
    let mut rows = 0;
    let mut columns = 0;
    while stream.Peek().GetType() == kStringToken {
        if !ParseAreasRow(
            &stream.ConsumeIncludingWhitespace().Value().ToString(),
            &mut map,
            rows,
            &mut columns,
        ) {
            return Err(invalid(id));
        }
        rows += 1;
    }
    if rows == 0 {
        return Err(invalid(id));
    }
    Ok(Areas(map, rows, columns))
}
// cpp: css_parsing_utils.cc:6997-7035. Track functions retain typed values;
// their arguments are breadths, not recursively nested track functions.
fn ConsumeTrackSize<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if stream.Peek().Id() == CSSValueID::kSubgrid {
        return Err(unsupported(id, "ConsumeGridTrackList subgrid"));
    }
    if stream.Peek().FunctionId() == Some(CSSValueID::kMinmax) {
        let result;
        {
            let mut args = RestoringBlockGuard::new(stream);
            args.ConsumeWhitespace();
            let min = ConsumeGridBreadth(id, &mut args, mode)?.ok_or_else(|| invalid(id))?;
            if matches!(min.Payload(), CSSValuePayload::kNumericLiteralClass(n) if n.GetType() == UnitType::kFlex)
                || args.Peek().GetType() != kCommaToken
            {
                return Err(invalid(id));
            }
            args.ConsumeIncludingWhitespace();
            let max = ConsumeGridBreadth(id, &mut args, mode)?.ok_or_else(|| invalid(id))?;
            if !args.AtEnd() || !args.Release() {
                return Err(invalid(id));
            }
            result = values::function(CSSValueID::kMinmax, vec![min, max]);
        }
        stream.ConsumeWhitespace();
        return Ok(Some(result));
    }
    // cpp: css_parsing_utils.cc:6926-6944,7031-7034. fit-content()
    // carries one nonnegative length-percentage, not a grid breadth. Keep
    // its typed function so the existing native track constructor retains
    // the fit-content limit independently from its intrinsic min/max sizes.
    if stream.Peek().FunctionId() == Some(CSSValueID::kFitContent) {
        let result;
        {
            let mut args = RestoringBlockGuard::new(stream);
            args.ConsumeWhitespace();
            let length = ConsumeLiteral(
                id,
                &mut args,
                mode,
                Grammar::Length {
                    percent: true,
                    nonnegative: true,
                    quirks: true,
                    keywords: &[],
                },
            )?;
            if !args.AtEnd() || !args.Release() {
                return Err(invalid(id));
            }
            result = values::function(CSSValueID::kFitContent, vec![length]);
        }
        stream.ConsumeWhitespace();
        return Ok(Some(result));
    }
    if stream.Peek().FunctionId() == Some(CSSValueID::kRepeat) {
        return Ok(None); // repeat is a track-list entry, never a track size.
    }
    ConsumeGridBreadth(id, stream, mode)
}

// cpp: css_parsing_utils.cc:6903-6923. Shared by plain tracks and both
// minmax arguments, including nonnegative math and the unitless-zero rule.
fn ConsumeGridBreadth<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if stream.Peek().GetType() == kDimensionToken && stream.Peek().GetUnitType() == UnitType::kFlex
    {
        let number = stream.Peek().NumericValue();
        if number < 0.0 {
            return Err(invalid(id));
        }
        stream.ConsumeIncludingWhitespace();
        return Ok(Some(values::numeric(number, UnitType::kFlex)));
    }
    if !(matches!(
        stream.Peek().GetType(),
        kDimensionToken | kNumberToken | kPercentageToken
    ) || matches(stream.Peek().Id(), &["auto", "min-content", "max-content"])
        || IsMathFunction(stream))
    {
        return Ok(None);
    }
    ConsumeLiteral(
        id,
        stream,
        mode,
        Grammar::Length {
            percent: true,
            nonnegative: true,
            quirks: false,
            keywords: &["auto", "min-content", "max-content"],
        },
    )
    .map(Some)
}

// cpp: css_parsing_utils.cc:6925-6948 ConsumeGridLineNames. The bracketed
// subclass is retained because its position among track sizes is the native
// grid line index.
fn ConsumeLineNames<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    let names = {
        let mut args = BlockGuard::new(stream);
        args.ConsumeWhitespace();
        let mut names = Vec::new();
        while !args.AtEnd() {
            let token = args.Peek().clone();
            if token.FunctionId() == Some(CSSValueID::kIdent) {
                return Err(unsupported(
                    id,
                    "ConsumeCustomIdent ident() TreeScope binding",
                ));
            }
            if token.GetType() != kIdentToken
                || matches!(
                    token.Id(),
                    CSSValueID::kDefault | CSSValueID::kAuto | CSSValueID::kSpan
                )
                || values::wide(token.Id()).is_some()
            {
                return Err(invalid(id));
            }
            let name = token.Value().ToString();
            args.ConsumeIncludingWhitespace();
            names.push(values::custom_ident(&name, CSSPropertyID::kInvalid));
        }
        names
    };
    stream.ConsumeWhitespace();
    Ok(values::grid_line_names(names))
}
pub(super) fn ConsumeTracks<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    template: bool,
    no_repeat: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    ConsumeTracksWithGridLanes(id, stream, mode, template, no_repeat, false)
}

// cpp: css_parsing_utils.cc:7105-7203 ConsumeGridTrackRepeatFunction,
// standalone integer-repeat branch. Auto-repeat and subgrid remain explicit
// unsupported collaborators rather than being expanded into plain tracks.
fn ConsumeIntegerRepeat<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let result;
    {
        let mut args = RestoringBlockGuard::new(stream);
        args.ConsumeWhitespace();
        if matches!(
            args.Peek().Id(),
            CSSValueID::kAutoFill | CSSValueID::kAutoFit
        ) {
            return Err(unsupported(id, "CSSGridAutoRepeatValue"));
        }
        let repetitions = ConsumeLiteral(id, &mut args, mode, Grammar::Integer { minimum: 1 })?;
        if args.Peek().GetType() != kCommaToken {
            return Err(invalid(id));
        }
        args.ConsumeIncludingWhitespace();
        let mut entries = Vec::new();
        if args.Peek().GetType() == kLeftBracketToken {
            entries.push(ConsumeLineNames(id, &mut args)?);
        }
        let mut tracks = 0u32;
        while !args.AtEnd() {
            entries.push(ConsumeTrackSize(id, &mut args, mode)?.ok_or_else(|| invalid(id))?);
            tracks += 1;
            if args.Peek().GetType() == kLeftBracketToken {
                entries.push(ConsumeLineNames(id, &mut args)?);
            }
        }
        if tracks == 0 || !args.Release() {
            return Err(invalid(id));
        }
        result = Rc::new(Value::new(CSSValuePayload::kGridIntegerRepeatClass(
            values::CSSGridIntegerRepeatValue {
                repetitions,
                extra_clamp: layoutng_style::style::grid_area::K_GRID_MAX_TRACKS as u32 / tracks,
                list: values::CSSValueList::new(entries, values::ListSeparator::Space),
            },
        )));
    }
    stream.ConsumeWhitespace();
    Ok(result)
}
// cpp: css_parsing_utils.cc:7420-7425,7472-7479. The same track-list consumer
// stops at direction/string components only for the GridLanes shorthand.
pub(super) fn ConsumeTracksWithGridLanes<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    template: bool,
    no_repeat: bool,
    is_grid_lanes: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    if template && stream.Peek().Id() == CSSValueID::kNone {
        stream.ConsumeIncludingWhitespace();
        return Ok(NoneValue());
    }
    let mut tracks = Vec::new();
    while !at_value_end(stream) && !AtSlash(stream) {
        if is_grid_lanes
            && (matches!(
                stream.Peek().Id(),
                CSSValueID::kNormal | CSSValueID::kRow | CSSValueID::kColumn
            ) || stream.Peek().GetType() == kStringToken)
        {
            break;
        }
        if stream.Peek().FunctionId() == Some(CSSValueID::kRepeat) {
            if no_repeat || !template {
                return Err(invalid(id));
            }
            tracks.push(ConsumeIntegerRepeat(id, stream, mode)?);
            continue;
        }
        if stream.Peek().GetType() == kLeftBracketToken {
            if !template {
                return Err(invalid(id));
            }
            tracks.push(ConsumeLineNames(id, stream)?);
            continue;
        }
        tracks.push(ConsumeTrackSize(id, stream, mode)?.ok_or_else(|| invalid(id))?);
    }
    if tracks.is_empty() {
        return Err(invalid(id));
    }
    Ok(values::list(tracks, values::ListSeparator::Space))
}
struct Template {
    rows: Rc<Value>,
    columns: Rc<Value>,
    areas: Rc<Value>,
}
// cpp: css_parsing_utils.cc:7207-7284,7607-7671. Failed explicit-track
// attempts restore before the area/none alternatives or grid auto-flow fallback.
fn ConsumeTemplate<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Template>, PropertyParseError> {
    stream.Peek();
    let saved = stream.Save();
    if stream.Peek().GetType() != kStringToken {
        let rows = ConsumeTracks(id, stream, mode, true, false);
        match rows {
            Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
            Ok(rows) if Slash(stream) => match ConsumeTracks(id, stream, mode, true, false) {
                Ok(columns) => {
                    return Ok(Some(Template {
                        rows,
                        columns,
                        areas: NoneValue(),
                    }))
                }
                Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
                _ => {}
            },
            _ => {}
        }
        stream.Peek();
        stream.Restore(saved);
    }
    if stream.Peek().GetType() == kStringToken {
        let mut map = NamedGridAreaMap::default();
        let mut rows = 0;
        let mut columns = 0;
        let mut tracks = Vec::new();
        loop {
            if stream.Peek().GetType() != kStringToken {
                return Err(invalid(id));
            }
            if !ParseAreasRow(
                &stream.ConsumeIncludingWhitespace().Value().ToString(),
                &mut map,
                rows,
                &mut columns,
            ) {
                return Err(invalid(id));
            }
            rows += 1;
            tracks.push(ConsumeTrackSize(id, stream, mode)?.unwrap_or_else(AutoValue));
            if stream.Peek().GetType() == kLeftBracketToken {
                return Err(unsupported(
                    id,
                    "ConsumeGridLineNames CSSBracketedValueList",
                ));
            }
            if at_value_end(stream) || AtSlash(stream) {
                break;
            }
        }
        let template_columns = if Slash(stream) {
            ConsumeTracks(id, stream, mode, false, true)?
        } else {
            NoneValue()
        };
        return Ok(Some(Template {
            rows: values::list(tracks, values::ListSeparator::Space),
            columns: template_columns,
            areas: Areas(map, rows, columns),
        }));
    }
    if stream.Peek().Id() == CSSValueID::kNone {
        stream.ConsumeIncludingWhitespace();
        return Ok(Some(Template {
            rows: NoneValue(),
            columns: NoneValue(),
            areas: NoneValue(),
        }));
    }
    Ok(None)
}
// cpp: shorthands_custom.cc:3736-3762.
fn ConsumeImplicitFlow<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    column: bool,
) -> Result<Rc<Value>, PropertyParseError> {
    let dense = if stream.Peek().Id() == CSSValueID::kAutoFlow {
        stream.ConsumeIncludingWhitespace();
        let dense = stream.Peek().Id() == CSSValueID::kDense;
        if dense {
            stream.ConsumeIncludingWhitespace();
        }
        dense
    } else {
        if stream.Peek().Id() != CSSValueID::kDense {
            return Err(invalid(id));
        }
        stream.ConsumeIncludingWhitespace();
        if stream.Peek().Id() != CSSValueID::kAutoFlow {
            return Err(invalid(id));
        }
        stream.ConsumeIncludingWhitespace();
        true
    };
    let mut items = Vec::new();
    if column || !dense {
        items.push(values::identifier(if column {
            CSSValueID::kColumn
        } else {
            CSSValueID::kRow
        }));
    }
    if dense {
        items.push(values::identifier(CSSValueID::kDense));
    }
    Ok(values::list(items, values::ListSeparator::Space))
}
pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    use CSSPropertyID::*;
    stream.Peek();
    let saved = stream.Save();
    if let Some(t) = ConsumeTemplate(id, stream, mode)? {
        if id == kGridTemplate || at_value_end(stream) {
            out.push(make_expanded(kGridTemplateRows, id, t.rows, false));
            out.push(make_expanded(kGridTemplateColumns, id, t.columns, false));
            out.push(make_expanded(kGridTemplateAreas, id, t.areas, false));
            if id == kGrid {
                out.push(make_expanded(
                    kGridAutoFlow,
                    id,
                    values::list(
                        vec![values::identifier(CSSValueID::kRow)],
                        values::ListSeparator::Space,
                    ),
                    true,
                ));
                out.push(make_expanded(kGridAutoColumns, id, AutoValue(), true));
                out.push(make_expanded(kGridAutoRows, id, AutoValue(), true));
            }
            return Ok(());
        }
    }
    if id == kGridTemplate {
        return Err(invalid(id));
    }
    stream.Peek();
    stream.Restore(saved);
    let (rows, columns, flow, auto_rows, auto_columns) = if matches!(
        stream.Peek().Id(),
        CSSValueID::kAutoFlow | CSSValueID::kDense
    ) {
        let flow = ConsumeImplicitFlow(id, stream, false)?;
        let auto_rows = if Slash(stream) {
            AutoValue()
        } else {
            let v = ConsumeTracks(id, stream, mode, false, false)?;
            if !Slash(stream) {
                return Err(invalid(id));
            }
            v
        };
        let columns = ConsumeTracks(id, stream, mode, true, false)?;
        (NoneValue(), columns, flow, auto_rows, AutoValue())
    } else {
        let rows = ConsumeTracks(id, stream, mode, true, false)?;
        if !Slash(stream) {
            return Err(invalid(id));
        }
        let flow = ConsumeImplicitFlow(id, stream, true)?;
        let auto_columns = if at_value_end(stream) {
            AutoValue()
        } else {
            ConsumeTracks(id, stream, mode, false, false)?
        };
        (rows, NoneValue(), flow, AutoValue(), auto_columns)
    };
    out.push(make_expanded(kGridTemplateColumns, id, columns, false));
    out.push(make_expanded(kGridTemplateRows, id, rows, false));
    out.push(make_expanded(kGridTemplateAreas, id, NoneValue(), false));
    out.push(make_expanded(kGridAutoFlow, id, flow, true));
    out.push(make_expanded(kGridAutoColumns, id, auto_columns, true));
    out.push(make_expanded(kGridAutoRows, id, auto_rows, true));
    Ok(())
}
