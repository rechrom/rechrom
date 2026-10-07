#![allow(non_snake_case)]
use crate::style_resolver::{
    border_radius::{Length, SplitTopLevel},
    fonts::FontSizeOf,
    number::Number,
    selector::SourceSpace,
    shadow::SplitWhitespace,
};
use css_parser::length_percentage_parser::ParseLengthPercentage;
use layoutng_assembly::internal::layout_input::*;

fn LowerTrim(input: &str) -> String {
    input
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase()
}

// cpp: style_resolver/style_resolver.cc:1558-1588
fn ParseGridTrackBreadth(
    input: &str,
    font_size: f64,
    allow_flex: bool,
) -> Option<GridTrackBreadth> {
    let value = LowerTrim(input);
    match value.as_str() {
        "auto" => return Some(GridTrackBreadth::default()),
        "min-content" => {
            return Some(GridTrackBreadth {
                kind: GridTrackBreadthKind::kMinContent,
                ..Default::default()
            })
        }
        "max-content" => {
            return Some(GridTrackBreadth {
                kind: GridTrackBreadthKind::kMaxContent,
                ..Default::default()
            })
        }
        _ => {}
    }
    if let Some(fraction) = value.strip_suffix("fr") {
        if !allow_flex {
            return None;
        }
        return Number(fraction)
            .filter(|n| *n > 0.0)
            .map(GridTrackBreadth::Flex);
    }
    if let Some(calculated) = ParseLengthPercentage(&value, font_size) {
        return Some(GridTrackBreadth {
            kind: GridTrackBreadthKind::kCalculated,
            pixels: calculated.pixels,
            percentage: calculated.percentage,
        });
    }
    if let Some(percentage) = value
        .strip_suffix('%')
        .and_then(Number)
        .filter(|n| *n >= 0.0)
    {
        return Some(GridTrackBreadth {
            kind: GridTrackBreadthKind::kPercentage,
            percentage,
            ..Default::default()
        });
    }
    Length(&value, font_size)
        .filter(|n| *n >= 0.0)
        .map(GridTrackBreadth::Fixed)
}

// cpp: style_resolver/style_resolver.cc:1590-1620
fn ParseGridTrack(input: &str, font_size: f64) -> Option<GridTrack> {
    let value = LowerTrim(input);
    if let Some(argument) = value
        .strip_prefix("fit-content(")
        .and_then(|v| v.strip_suffix(')'))
    {
        let maximum = ParseGridTrackBreadth(argument, font_size, false)?;
        if !matches!(
            maximum.kind,
            GridTrackBreadthKind::kFixed
                | GridTrackBreadthKind::kPercentage
                | GridTrackBreadthKind::kCalculated
        ) {
            return None;
        }
        return Some(GridTrack {
            maximum,
            fit_content: true,
            ..Default::default()
        });
    }
    if let Some(arguments) = value
        .strip_prefix("minmax(")
        .and_then(|v| v.strip_suffix(')'))
    {
        let arguments = SplitTopLevel(arguments, b',');
        if arguments.len() != 2 {
            return None;
        }
        return Some(GridTrack {
            minimum: ParseGridTrackBreadth(&arguments[0], font_size, false)?,
            maximum: ParseGridTrackBreadth(&arguments[1], font_size, true)?,
            fit_content: false,
        });
    }
    let breadth = ParseGridTrackBreadth(&value, font_size, true)?;
    Some(if breadth.kind == GridTrackBreadthKind::kFlex {
        GridTrack {
            maximum: breadth,
            ..Default::default()
        }
    } else {
        GridTrack {
            minimum: breadth,
            maximum: breadth,
            fit_content: false,
        }
    })
}

// cpp: style_resolver/style_resolver.cc:1622-1657
fn SplitGridComponents(input: &str) -> Option<Vec<String>> {
    let bytes = input.as_bytes();
    let (mut result, mut begin, mut parentheses, mut brackets, mut quote) =
        (Vec::new(), 0, 0u32, 0u32, 0u8);
    for index in 0..=bytes.len() {
        let character = bytes.get(index).copied().unwrap_or(b' ');
        if quote != 0 {
            if character == quote && (index == 0 || bytes[index - 1] != b'\\') {
                quote = 0;
            }
        } else if matches!(character, b'\'' | b'"') {
            quote = character;
        } else if character == b'(' {
            parentheses += 1;
        } else if character == b')' {
            if parentheses == 0 {
                return None;
            }
            parentheses -= 1;
        } else if character == b'[' {
            brackets += 1;
        } else if character == b']' {
            if brackets == 0 {
                return None;
            }
            brackets -= 1;
        }
        if parentheses == 0 && brackets == 0 && quote == 0 && character.source_space() {
            if index > begin {
                result.push(input[begin..index].to_owned());
            }
            begin = index + 1;
        }
    }
    if parentheses != 0 || brackets != 0 || quote != 0 {
        None
    } else {
        Some(result)
    }
}

// cpp: style_resolver/style_resolver.cc:1659-1675
fn ParseGridLineNames(input: &str) -> Option<Vec<String>> {
    let value = input.trim_matches(|c: char| c.source_space());
    if value.len() < 3 || !value.starts_with('[') || !value.ends_with(']') {
        return None;
    }
    let mut names = SplitWhitespace(&value[1..value.len() - 1]);
    if names.is_empty() {
        return None;
    }
    for name in &mut names {
        name.make_ascii_lowercase();
        if matches!(
            name.as_str(),
            "auto" | "span" | "initial" | "inherit" | "unset" | "revert" | "revert-layer"
        ) {
            return None;
        }
    }
    Some(names)
}

// cpp: style_resolver/style_resolver.cc:1677-1741
fn ParseGridTemplateAreas(input: &str) -> Option<GridTemplateAreasInput> {
    let bytes = input.as_bytes();
    let (mut rows, mut cursor) = (Vec::<Vec<String>>::new(), 0);
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].source_space() {
            cursor += 1;
        }
        if cursor == bytes.len() {
            break;
        }
        let quote = bytes[cursor];
        if !matches!(quote, b'\'' | b'"') {
            return None;
        }
        cursor += 1;
        let begin = cursor;
        while cursor < bytes.len() && bytes[cursor] != quote {
            if bytes[cursor] == b'\\' && cursor + 1 < bytes.len() {
                cursor += 1;
            }
            cursor += 1;
        }
        if cursor == bytes.len() {
            return None;
        }
        let cells = SplitWhitespace(&input[begin..cursor]);
        cursor += 1;
        if cells.is_empty()
            || cells.len() > 1000
            || rows.first().is_some_and(|row| row.len() != cells.len())
        {
            return None;
        }
        rows.push(cells);
        if rows.len() > 1000 {
            return None;
        }
    }
    if rows.is_empty() {
        return None;
    }
    let mut result = GridTemplateAreasInput {
        row_count: rows.len() as u32,
        column_count: rows[0].len() as u32,
        areas: Vec::new(),
    };
    for (row_index, row) in rows.iter_mut().enumerate() {
        for (column_index, cell) in row.iter_mut().enumerate() {
            if cell.bytes().all(|b| b == b'.') {
                cell.clear();
                continue;
            }
            if cell.contains('.') {
                return None;
            }
            let (row, column) = (row_index as u32, column_index as u32);
            if let Some(area) = result.areas.iter_mut().find(|a| a.name == *cell) {
                area.row_start = area.row_start.min(row);
                area.row_end = area.row_end.max(row + 1);
                area.column_start = area.column_start.min(column);
                area.column_end = area.column_end.max(column + 1);
            } else {
                result.areas.push(GridTemplateArea {
                    name: cell.clone(),
                    row_start: row,
                    row_end: row + 1,
                    column_start: column,
                    column_end: column + 1,
                });
            }
        }
    }
    for area in &result.areas {
        for row in area.row_start..area.row_end {
            for column in area.column_start..area.column_end {
                if rows[row as usize][column as usize] != area.name {
                    return None;
                }
            }
        }
    }
    Some(result)
}

// cpp: style_resolver/style_resolver.cc:1743-1767
fn ParseRepeatedGridTracks(
    input: &str,
    font_size: f64,
    kind: GridRepeatKind,
    count: u32,
) -> Option<GridTrackRepeater> {
    let components = SplitGridComponents(input)?;
    if components.is_empty() {
        return None;
    }
    let mut result = GridTrackRepeater {
        kind,
        count,
        tracks: Vec::new(),
        line_names: vec![Vec::new()],
    };
    for component in components {
        if component.starts_with('[') {
            result
                .line_names
                .last_mut()
                .unwrap()
                .extend(ParseGridLineNames(&component)?);
        } else {
            result.tracks.push(ParseGridTrack(&component, font_size)?);
            result.line_names.push(Vec::new());
        }
    }
    if result.tracks.is_empty() {
        None
    } else {
        Some(result)
    }
}

// cpp: style_resolver/style_resolver.cc:1769-1832
fn ParseGridTrackList(
    input: &str,
    font_size: f64,
    allow_repeat: bool,
) -> Option<GridTrackListInput> {
    let mut result = GridTrackListInput::default();
    let (mut saw_auto_repeat, mut outer_line) = (false, 0u32);
    for component in SplitGridComponents(input)? {
        let value = component.to_ascii_lowercase();
        if value.starts_with('[') {
            for name in ParseGridLineNames(&value)? {
                result.line_names.push(GridNamedLineInput {
                    name,
                    position: outer_line,
                });
            }
            continue;
        }
        if let Some(arguments) = value
            .strip_prefix("repeat(")
            .and_then(|v| v.strip_suffix(')'))
        {
            if !allow_repeat {
                return None;
            }
            let arguments = SplitTopLevel(arguments, b',');
            if arguments.len() != 2 {
                return None;
            }
            let repeat_value = arguments[0].to_ascii_lowercase();
            let (kind, count) = if matches!(repeat_value.as_str(), "auto-fill" | "auto-fit") {
                if saw_auto_repeat {
                    return None;
                }
                saw_auto_repeat = true;
                (
                    if repeat_value == "auto-fill" {
                        GridRepeatKind::kAutoFill
                    } else {
                        GridRepeatKind::kAutoFit
                    },
                    1,
                )
            } else {
                (
                    GridRepeatKind::kInteger,
                    Number(&repeat_value)
                        .filter(|n| *n >= 1.0 && *n <= 1000.0 && n.floor() == *n)?
                        as u32,
                )
            };
            let repeater = ParseRepeatedGridTracks(&arguments[1], font_size, kind, count)?;
            outer_line = outer_line.wrapping_add(
                if matches!(kind, GridRepeatKind::kAutoFill | GridRepeatKind::kAutoFit) {
                    1
                } else {
                    count.wrapping_mul(repeater.tracks.len() as u32)
                },
            );
            result.repeaters.push(repeater);
            continue;
        }
        let track = ParseGridTrack(&component, font_size)?;
        if result
            .repeaters
            .last()
            .is_none_or(|r| r.kind != GridRepeatKind::kNone)
        {
            result.repeaters.push(GridTrackRepeater::default());
        }
        result.repeaters.last_mut().unwrap().tracks.push(track);
        outer_line = outer_line.wrapping_add(1);
    }
    if result.empty() {
        None
    } else {
        Some(result)
    }
}

// cpp: style_resolver/style_resolver.cc:1842-1850
fn GridInteger(input: &str) -> Option<i32> {
    Number(input)
        .filter(|n| *n != 0.0 && *n >= i32::MIN as f64 && *n <= i32::MAX as f64 && n.floor() == *n)
        .map(|n| n as i32)
}

// cpp: style_resolver/style_resolver.cc:1852-1888
fn ParseGridLine(input: &str) -> Option<GridLine> {
    let parts = SplitWhitespace(&input.to_ascii_lowercase());
    if parts.len() == 1 && parts[0] == "auto" {
        return Some(GridLine::default());
    }
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }
    if parts[0] == "span" {
        let (mut number, mut saw_number, mut name) = (1, false, String::new());
        for part in &parts[1..] {
            if let Some(parsed) = GridInteger(part) {
                if parsed < 1 || saw_number {
                    return None;
                }
                number = parsed;
                saw_number = true;
            } else if name.is_empty() && part != "auto" && part != "span" {
                name = part.clone();
            } else {
                return None;
            }
        }
        return Some(GridLine {
            kind: GridLineKind::kSpan,
            number,
            name,
        });
    }
    if let Some(number) = GridInteger(&parts[0]) {
        if parts.len() > 2 || (parts.len() == 2 && matches!(parts[1].as_str(), "auto" | "span")) {
            return None;
        }
        return Some(GridLine {
            kind: GridLineKind::kExplicit,
            number,
            name: parts.get(1).cloned().unwrap_or_default(),
        });
    }
    if parts.len() == 1 && parts[0] != "span" {
        Some(GridLine {
            kind: GridLineKind::kNamedArea,
            name: parts[0].clone(),
            ..Default::default()
        })
    } else {
        None
    }
}

// cpp: style_resolver/style_resolver.cc:5249-5282,5303-5360
pub(crate) fn ApplyGridProperty(style: &mut ComputedStyle, property: &str, raw: &str) -> bool {
    let value = LowerTrim(raw);
    let font_size = FontSizeOf(style);
    match property {
        "grid-template-columns" | "grid-template-rows" => {
            if value == "subgrid" {
                let e = style.extended.get_or_insert_with(Default::default);
                if property == "grid-template-columns" {
                    style.grid_template_columns.clear();
                    e.grid_columns.clear();
                    e.subgrid_columns = true;
                } else {
                    e.grid_rows.clear();
                    e.subgrid_rows = true;
                }
            } else if let Some(tracks) = ParseGridTrackList(&value, font_size, true) {
                let e = style.extended.get_or_insert_with(Default::default);
                if property == "grid-template-columns" {
                    style.grid_template_columns.clear();
                    e.grid_columns = tracks;
                    e.subgrid_columns = false;
                } else {
                    e.grid_rows = tracks;
                    e.subgrid_rows = false;
                }
            }
        }
        "grid-template-areas" => {
            if value == "none" {
                style
                    .extended
                    .get_or_insert_with(Default::default)
                    .grid_template_areas = None;
            } else if let Some(areas) = ParseGridTemplateAreas(&value) {
                style
                    .extended
                    .get_or_insert_with(Default::default)
                    .grid_template_areas = Some(areas);
            }
        }
        "grid-auto-columns" | "grid-auto-rows" => {
            if let Some(tracks) = ParseGridTrackList(&value, font_size, false) {
                let e = style.extended.get_or_insert_with(Default::default);
                if property == "grid-auto-columns" {
                    e.grid_auto_columns = tracks;
                } else {
                    e.grid_auto_rows = tracks;
                }
            }
        }
        "grid-area" => {
            let parts = SplitTopLevel(&value, b'/');
            if !parts.is_empty() && parts.len() <= 4 {
                let parsed: Option<Vec<_>> = parts.iter().map(|p| ParseGridLine(p)).collect();
                if let Some(parsed) = parsed {
                    let mut lines: [GridLine; 4] = std::array::from_fn(|_| GridLine::default());
                    for (index, line) in parsed.into_iter().enumerate() {
                        lines[index] = line;
                    }
                    let named_or_auto = |line: &GridLine| {
                        if line.kind == GridLineKind::kNamedArea {
                            line.clone()
                        } else {
                            GridLine::default()
                        }
                    };
                    if parts.len() < 2 {
                        lines[1] = named_or_auto(&lines[0]);
                    }
                    if parts.len() < 3 {
                        lines[2] = named_or_auto(&lines[0]);
                    }
                    if parts.len() < 4 {
                        lines[3] = named_or_auto(&lines[1]);
                    }
                    let [row_start, column_start, row_end, column_end] = lines;
                    let e = style.extended.get_or_insert_with(Default::default);
                    e.grid_row_start = row_start;
                    e.grid_column_start = column_start;
                    e.grid_row_end = row_end;
                    e.grid_column_end = column_end;
                }
            }
        }
        "grid-column" | "grid-row" => {
            let parts = SplitTopLevel(&value, b'/');
            if !parts.is_empty() && parts.len() <= 2 {
                if let Some(start) = ParseGridLine(&parts[0]) {
                    let end = if parts.len() == 2 {
                        ParseGridLine(&parts[1])
                    } else {
                        Some(if start.kind == GridLineKind::kNamedArea {
                            start.clone()
                        } else {
                            GridLine::default()
                        })
                    };
                    if let Some(end) = end {
                        let e = style.extended.get_or_insert_with(Default::default);
                        if property == "grid-column" {
                            e.grid_column_start = start;
                            e.grid_column_end = end;
                        } else {
                            e.grid_row_start = start;
                            e.grid_row_end = end;
                        }
                    }
                }
            }
        }
        "grid-column-start" | "grid-column-end" | "grid-row-start" | "grid-row-end" => {
            if let Some(line) = ParseGridLine(&value) {
                let e = style.extended.get_or_insert_with(Default::default);
                match property {
                    "grid-column-start" => e.grid_column_start = line,
                    "grid-column-end" => e.grid_column_end = line,
                    "grid-row-start" => e.grid_row_start = line,
                    _ => e.grid_row_end = line,
                }
            }
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug, PartialEq)]
    enum Token {
        Number(f64),
        Text(String),
    }
    #[derive(Default)]
    struct Dump(Vec<Token>);
    impl Dump {
        fn number(&mut self, value: f64) {
            self.0.push(Token::Number(value));
        }
        fn text(&mut self, value: &str) {
            self.0.push(Token::Text(value.into()));
        }
        fn breadth(&mut self, value: &GridTrackBreadth) {
            self.number(value.kind as i32 as f64);
            self.number(value.pixels);
            self.number(value.percentage);
        }
        fn track(&mut self, value: &GridTrack) {
            self.breadth(&value.minimum);
            self.breadth(&value.maximum);
            self.number(value.fit_content as u8 as f64);
        }
        fn list(&mut self, value: &GridTrackListInput) {
            self.number(value.repeaters.len() as f64);
            for r in &value.repeaters {
                self.number(r.kind as i32 as f64);
                self.number(r.count as f64);
                self.number(r.tracks.len() as f64);
                for t in &r.tracks {
                    self.track(t);
                }
                self.number(r.line_names.len() as f64);
                for names in &r.line_names {
                    self.number(names.len() as f64);
                    for name in names {
                        self.text(name);
                    }
                }
            }
            self.number(value.line_names.len() as f64);
            for name in &value.line_names {
                self.text(&name.name);
                self.number(name.position as f64);
            }
        }
        fn areas(&mut self, value: &GridTemplateAreasInput) {
            self.number(value.row_count as f64);
            self.number(value.column_count as f64);
            self.number(value.areas.len() as f64);
            for a in &value.areas {
                self.text(&a.name);
                for v in [a.row_start, a.row_end, a.column_start, a.column_end] {
                    self.number(v as f64);
                }
            }
        }
        fn line(&mut self, value: &GridLine) {
            self.number(value.kind as i32 as f64);
            self.number(value.number as f64);
            self.text(&value.name);
        }
    }
    fn decode(hex: &str) -> String {
        String::from_utf8(
            hex.as_bytes()
                .chunks_exact(2)
                .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                .collect(),
        )
        .unwrap()
    }
    fn expected(value: &str) -> Option<Vec<Token>> {
        if value == "none" {
            return None;
        }
        Some(
            value
                .split(',')
                .map(|s| {
                    if let Some(v) = s.strip_prefix("n:") {
                        Token::Number(v.parse().unwrap())
                    } else {
                        Token::Text(decode(s.strip_prefix("s:").unwrap()))
                    }
                })
                .collect(),
        )
    }
    #[test]
    fn complete_grid_parsers_match_frozen_cpp() {
        for line in
            include_str!("../../../../artifacts/cpp-reference/grid-parser-results.tsv").lines()
        {
            let fields: Vec<_> = line.split('\t').collect();
            let input = decode(fields[1]);
            let mut dump = Dump::default();
            let valid = match fields[0] {
                "breadth0" | "breadth1" => {
                    ParseGridTrackBreadth(&input, 24.0, fields[0] == "breadth1")
                        .map(|v| dump.breadth(&v))
                        .is_some()
                }
                "track" => ParseGridTrack(&input, 24.0)
                    .map(|v| dump.track(&v))
                    .is_some(),
                "list0" | "list1" => ParseGridTrackList(&input, 24.0, fields[0] == "list1")
                    .map(|v| dump.list(&v))
                    .is_some(),
                "names" => ParseGridLineNames(&input)
                    .map(|v| {
                        dump.number(v.len() as f64);
                        for name in v {
                            dump.text(&name);
                        }
                    })
                    .is_some(),
                "components" => SplitGridComponents(&input)
                    .map(|v| {
                        dump.number(v.len() as f64);
                        for component in v {
                            dump.text(&component);
                        }
                    })
                    .is_some(),
                "areas" => ParseGridTemplateAreas(&input)
                    .map(|v| dump.areas(&v))
                    .is_some(),
                "line" => ParseGridLine(&input).map(|v| dump.line(&v)).is_some(),
                _ => panic!("unknown reference parser"),
            };
            assert_eq!(
                valid.then_some(dump.0),
                expected(fields[2]),
                "{} {:?}",
                fields[0],
                input
            );
        }
    }
    #[test]
    fn sequential_grid_declarations_match_cpp_including_subgrid_clear_behavior() {
        let mut style = ComputedStyle::default();
        style
            .extended
            .get_or_insert_with(Default::default)
            .font_size = 24.0;
        style.grid_template_columns = vec![42.0];
        for line in
            include_str!("../../../../artifacts/cpp-reference/grid-declaration-results.tsv").lines()
        {
            let fields: Vec<_> = line.split('\t').collect();
            let input = decode(fields[1]);
            crate::style_resolver::apply(&mut style, fields[0], &input, (1024.0, 768.0));
            let e = style.extended.as_ref().unwrap();
            let mut dump = Dump::default();
            dump.number(style.grid_template_columns.len() as f64);
            for &v in &style.grid_template_columns {
                dump.number(v);
            }
            for list in [
                &e.grid_columns,
                &e.grid_rows,
                &e.grid_auto_columns,
                &e.grid_auto_rows,
            ] {
                dump.list(list);
            }
            dump.number(e.subgrid_columns as u8 as f64);
            dump.number(e.subgrid_rows as u8 as f64);
            dump.number(e.grid_template_areas.is_some() as u8 as f64);
            if let Some(areas) = &e.grid_template_areas {
                dump.areas(areas);
            }
            for line in [
                &e.grid_row_start,
                &e.grid_column_start,
                &e.grid_row_end,
                &e.grid_column_end,
            ] {
                dump.line(line);
            }
            assert_eq!(
                Some(dump.0),
                expected(fields[2]),
                "{} {:?}",
                fields[0],
                input
            );
        }
    }
}
