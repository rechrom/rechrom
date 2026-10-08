#![allow(non_snake_case)]
use crate::style_resolver::border_radius::{Length, SplitTopLevel};
use crate::style_resolver::cascade::{CascadedDeclaration, ResolveDeclarationVariables};
use crate::style_resolver::css_wide::{CSSWideKeywordSource, CSSWideSource};
use crate::style_resolver::number::Number;
use crate::style_resolver::selector::SourceSpace;
use crate::style_resolver::shadow::SplitWhitespace;
use css_parser::length_percentage_parser::ParseLengthPercentage;
use layoutng_assembly::internal::layout_input::ComputedStyle;

fn Trim(value: &str) -> &str {
    value.trim_matches(|c: char| c.source_space())
}
// cpp: style_resolver/style_resolver.cc:905-909
fn Percentage(text: &str) -> Option<f64> {
    Number(Trim(text).to_ascii_lowercase().strip_suffix('%')?)
}
// cpp: style_resolver/style_resolver.cc:3679-3702
pub(crate) fn ParseFontFamilies(input: &str) -> Option<Vec<String>> {
    let mut families = Vec::new();
    for item in SplitTopLevel(input, b',') {
        let mut item = Trim(&item).to_owned();
        let bytes = item.as_bytes();
        let &first = bytes.first()?;
        if matches!(first, b'\'' | b'"') {
            if bytes.len() < 2 || bytes.last() != Some(&first) {
                return None;
            }
            let mut unquoted = Vec::new();
            let mut cursor = 1;
            while cursor + 1 < bytes.len() {
                if bytes[cursor] == b'\\' && cursor + 2 < bytes.len() {
                    cursor += 1;
                }
                unquoted.push(bytes[cursor]);
                cursor += 1;
            }
            item = String::from_utf8_lossy(&unquoted).into_owned();
        } else if item.contains(['\'', '"']) {
            return None;
        }
        if item.is_empty() {
            return None;
        }
        families.push(item);
    }
    (!families.is_empty()).then_some(families)
}

pub(crate) fn IsSoleMonospace(style: &ComputedStyle) -> bool {
    style.extended.as_ref().is_some_and(|extra| {
        extra.font_families.len() == 1 && extra.font_families[0].eq_ignore_ascii_case("monospace")
    })
}

pub(crate) fn ResolveComputedFontFamilies(
    cascade: &[CascadedDeclaration],
    properties: &crate::style_resolver::CustomProperties,
    initial: &CSSWideSource<'_>,
    inherited: &CSSWideSource<'_>,
    unset: &CSSWideSource<'_>,
    reverted: &CSSWideSource<'_>,
) -> Vec<String> {
    let families_of = |style: &ComputedStyle| {
        style
            .extended
            .as_ref()
            .map_or_else(|| vec!["serif".into()], |extra| extra.font_families.clone())
    };
    let mut result = families_of(reverted.style);
    for item in cascade {
        if item.declaration.property.starts_with("--") {
            continue;
        }
        let declaration = ResolveDeclarationVariables(&item.declaration, properties)
            .unwrap_or_else(|| cssom::CSSDeclaration {
                property: item.declaration.property.clone(),
                value: "unset".into(),
                ..Default::default()
            });
        if !matches!(
            declaration.property.as_str(),
            "font-family" | "font" | "all"
        ) {
            continue;
        }
        if let Some(source) =
            CSSWideKeywordSource(&declaration.value, initial, inherited, unset, reverted)
        {
            result = families_of(source.style);
        } else if declaration.property == "font-family" {
            if let Some(parsed) = ParseFontFamilies(&declaration.value) {
                result = parsed;
            }
        } else if declaration.property == "font" {
            if let Some(parsed) =
                ParseFontShorthand(&declaration.value, FontSizeOf(inherited.style))
            {
                result = parsed.families;
            }
        }
    }
    result
}

pub(crate) fn FontSizeWasSpecified(cascade: &[CascadedDeclaration]) -> bool {
    cascade.iter().any(|item| {
        matches!(
            item.declaration.property.as_str(),
            "font-size" | "font" | "all"
        )
    })
}
// cpp: style_resolver/style_resolver.cc:3707-3713
pub(crate) struct ParsedFontShorthand {
    pub computed_size: f64,
    pub weight: String,
    pub italic: bool,
    pub line_height: Option<String>,
    pub families: Vec<String>,
}
// cpp: style_resolver/style_resolver.cc:3715-3781
pub(crate) fn ParseFontShorthand(
    input: &str,
    parent_font_size: f64,
) -> Option<ParsedFontShorthand> {
    let parts = SplitWhitespace(input);
    if parts.len() < 2 {
        return None;
    }
    let mut result = ParsedFontShorthand {
        computed_size: 16.0,
        weight: "normal".into(),
        italic: false,
        line_height: None,
        families: Vec::new(),
    };
    let mut size_index = parts.len();
    let mut inline_line_height = String::new();
    for (index, part) in parts.iter().enumerate() {
        let candidate = if let Some((size, line_height)) = part.split_once('/') {
            inline_line_height = line_height.into();
            size
        } else {
            part.as_str()
        };
        if let Some(size) = ParseComputedFontSize(candidate, parent_font_size) {
            result.computed_size = size;
            size_index = index;
            break;
        }
        let lower = candidate.to_ascii_lowercase();
        if lower == "italic" || lower.starts_with("oblique") {
            result.italic = true;
        } else if lower == "normal" {
            continue;
        } else if matches!(lower.as_str(), "bold" | "bolder" | "lighter")
            || Number(&lower).is_some_and(|n| (1.0..=1000.0).contains(&n))
        {
            result.weight = lower;
        } else if matches!(
            lower.as_str(),
            "small-caps" | "condensed" | "expanded" | "semi-condensed" | "semi-expanded"
        ) {
            continue;
        } else {
            return None;
        }
    }
    if size_index == parts.len() {
        return None;
    }
    let mut family_index = size_index + 1;
    if !inline_line_height.is_empty() {
        result.line_height = Some(inline_line_height);
    } else if family_index < parts.len() {
        if parts[family_index] == "/" {
            family_index += 1;
            result.line_height = Some(parts.get(family_index)?.clone());
            family_index += 1;
        } else if let Some(line_height) = parts[family_index].strip_prefix('/') {
            if line_height.is_empty() {
                return None;
            }
            result.line_height = Some(line_height.into());
            family_index += 1;
        }
    }
    if family_index >= parts.len() {
        return None;
    }
    result.families = ParseFontFamilies(&parts[family_index..].join(" "))?;
    Some(result)
}
// cpp: style_resolver/style_resolver.cc:3783-3807
pub(crate) fn ApplyLineHeightValue(style: &mut ComputedStyle, input: &str, font_size: f64) {
    let value = Trim(input).to_ascii_lowercase();
    let extra = style.extended.get_or_insert_with(Default::default);
    if value == "normal" {
        extra.line_height = None;
        extra.line_height_percent = None;
    } else if let Some(calculated) = ParseLengthPercentage(&value, font_size)
        .filter(|p| p.pixels + p.percentage * font_size / 100.0 >= 0.0)
    {
        extra.line_height = Some(calculated.pixels + calculated.percentage * font_size / 100.0);
        extra.line_height_percent = None;
    } else if let Some(percentage) = Percentage(&value).filter(|&n| n >= 0.0) {
        extra.line_height = Some(percentage * font_size / 100.0);
        extra.line_height_percent = None;
    } else if let Some(number) = Number(&value).filter(|&n| n >= 0.0) {
        extra.line_height = None;
        extra.line_height_percent = Some(number * 100.0);
    } else if let Some(length) = Length(&value, font_size).filter(|&n| n >= 0.0) {
        extra.line_height = Some(length);
        extra.line_height_percent = None;
    }
}
// cpp: style_resolver/style_resolver.cc:7140-7142
pub(crate) fn FontSizeOf(style: &ComputedStyle) -> f64 {
    style.extended.as_ref().map_or(16.0, |e| e.font_size)
}
// cpp: style_resolver/style_resolver.cc:7144-7173
pub(crate) fn ParseComputedFontSize(input: &str, parent_font_size: f64) -> Option<f64> {
    let value = Trim(input).to_ascii_lowercase();
    if let Some(calculated) = ParseLengthPercentage(&value, parent_font_size) {
        let pixels = calculated.pixels + calculated.percentage * parent_font_size / 100.0;
        return (pixels >= 0.0 && pixels.is_finite()).then_some(pixels);
    }
    if let Some(percentage) = Percentage(&value) {
        let pixels = percentage * parent_font_size / 100.0;
        return (pixels >= 0.0 && pixels.is_finite()).then_some(pixels);
    }
    if let Some(length) = Length(&value, parent_font_size).filter(|&n| n >= 0.0) {
        return Some(length);
    }
    Some(match value.as_str() {
        "xx-small" => 9.0,
        "x-small" => 10.0,
        "small" => 13.0,
        "medium" => 16.0,
        "large" => 18.0,
        "x-large" => 24.0,
        "xx-large" => 32.0,
        "xxx-large" => 48.0,
        "smaller" => parent_font_size * 5.0 / 6.0,
        "larger" => parent_font_size * 6.0 / 5.0,
        _ => return None,
    })
}
// cpp: style_resolver/style_resolver.cc:7175-7208
pub(crate) fn ResolveComputedFontSize(
    cascade: &[CascadedDeclaration],
    properties: &crate::style_resolver::CustomProperties,
    initial: &CSSWideSource<'_>,
    inherited: &CSSWideSource<'_>,
    unset: &CSSWideSource<'_>,
    reverted: &CSSWideSource<'_>,
) -> f64 {
    let mut result = FontSizeOf(reverted.style);
    let parent_font_size = FontSizeOf(inherited.style);
    for item in cascade {
        if item.declaration.property.starts_with("--") {
            continue;
        }
        let declaration = ResolveDeclarationVariables(&item.declaration, properties)
            .unwrap_or_else(|| cssom::CSSDeclaration {
                property: item.declaration.property.clone(),
                value: "unset".into(),
                ..Default::default()
            });
        if !matches!(declaration.property.as_str(), "font-size" | "font" | "all") {
            continue;
        }
        if let Some(source) =
            CSSWideKeywordSource(&declaration.value, initial, inherited, unset, reverted)
        {
            result = FontSizeOf(source.style);
        } else if declaration.property == "font-size" {
            if let Some(parsed) = ParseComputedFontSize(&declaration.value, parent_font_size) {
                result = parsed;
            }
        } else if declaration.property == "font" {
            if let Some(parsed) = ParseFontShorthand(&declaration.value, parent_font_size) {
                result = parsed.computed_size;
            }
        }
    }
    result
}
// cpp: style_resolver/style_resolver.cc:7210-7224
pub(crate) fn ParseComputedFontWeight(input: &str, parent_weight: f64) -> Option<f64> {
    let value = Trim(input).to_ascii_lowercase();
    match value.as_str() {
        "normal" => Some(400.0),
        "bold" => Some(700.0),
        "bolder" => Some(if parent_weight < 350.0 {
            400.0
        } else if parent_weight < 550.0 {
            700.0
        } else {
            900.0
        }),
        "lighter" => Some(if parent_weight < 550.0 {
            100.0
        } else if parent_weight < 750.0 {
            400.0
        } else {
            700.0
        }),
        _ => Number(&value).filter(|&n| (1.0..=1000.0).contains(&n)),
    }
}
// cpp: style_resolver/style_resolver.cc:7226-7263
pub(crate) fn ResolveComputedFontWeight(
    cascade: &[CascadedDeclaration],
    properties: &crate::style_resolver::CustomProperties,
    initial: &CSSWideSource<'_>,
    inherited: &CSSWideSource<'_>,
    unset: &CSSWideSource<'_>,
    reverted: &CSSWideSource<'_>,
) -> f64 {
    let weight_of =
        |style: &ComputedStyle| style.extended.as_ref().map_or(400.0, |e| e.font_weight);
    let mut result = weight_of(reverted.style);
    let parent_weight = weight_of(inherited.style);
    for item in cascade {
        if item.declaration.property.starts_with("--") {
            continue;
        }
        let declaration = ResolveDeclarationVariables(&item.declaration, properties)
            .unwrap_or_else(|| cssom::CSSDeclaration {
                property: item.declaration.property.clone(),
                value: "unset".into(),
                ..Default::default()
            });
        if !matches!(
            declaration.property.as_str(),
            "font-weight" | "font" | "all"
        ) {
            continue;
        }
        if let Some(source) =
            CSSWideKeywordSource(&declaration.value, initial, inherited, unset, reverted)
        {
            result = weight_of(source.style);
        } else if declaration.property == "font-weight" {
            if let Some(parsed) = ParseComputedFontWeight(&declaration.value, parent_weight) {
                result = parsed;
            }
        } else if declaration.property == "font" {
            if let Some(shorthand) =
                ParseFontShorthand(&declaration.value, FontSizeOf(inherited.style))
            {
                if let Some(parsed) = ParseComputedFontWeight(&shorthand.weight, parent_weight) {
                    result = parsed;
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn font_parsers_match_independently_generated_cpp_results() {
        let cases = include_str!("../../../../artifacts/cpp-reference/font-results.tsv");
        let optional = |text: &str| {
            if text == "none" {
                None
            } else {
                Some(text.parse::<f64>().unwrap())
            }
        };
        for line in cases.split('\n').filter(|line| !line.is_empty()) {
            let fields: Vec<_> = line.split('\t').collect();
            let (size, weight) = (fields[0].parse().unwrap(), fields[1].parse().unwrap());
            match fields[2] {
                "size" => assert_eq!(
                    ParseComputedFontSize(fields[3], size),
                    optional(fields[4]),
                    "{line}"
                ),
                "weight" => assert_eq!(
                    ParseComputedFontWeight(fields[3], weight),
                    optional(fields[4]),
                    "{line}"
                ),
                "line" => {
                    let mut style = ComputedStyle::default();
                    ApplyLineHeightValue(&mut style, fields[3], size);
                    let e = style.extended.as_ref().unwrap();
                    assert_eq!(e.line_height, optional(fields[4]), "{line}");
                    assert_eq!(e.line_height_percent, optional(fields[5]), "{line}");
                }
                "family" => assert_eq!(
                    ParseFontFamilies(fields[3]).map(|f| f.join("|")),
                    (fields[4] != "none").then(|| fields[4].into()),
                    "{line}"
                ),
                "font" => {
                    let result = ParseFontShorthand(fields[3], size);
                    if fields[4] == "none" {
                        assert!(result.is_none(), "{line}");
                    } else {
                        let p = result.expect(line);
                        assert_eq!(p.computed_size, fields[4].parse::<f64>().unwrap(), "{line}");
                        assert_eq!(p.weight, fields[5], "{line}");
                        assert_eq!(p.italic, fields[6] == "1", "{line}");
                        assert_eq!(
                            p.line_height.as_deref(),
                            (fields[7] != "none").then_some(fields[7]),
                            "{line}"
                        );
                        assert_eq!(p.families.join("|"), fields[8], "{line}");
                    }
                }
                _ => panic!("invalid reference case"),
            }
        }
    }
}

#[cfg(test)]
mod computed_cascade_tests {
    #[test]
    fn computed_font_and_wide_cascade_matches_complete_cpp_resolver() {
        for (markup, css, expected) in [
            (
                include_str!("../../../../artifacts/cpp-reference/font-cascade.html"),
                include_str!("../../../../artifacts/cpp-reference/font-cascade.css"),
                include_str!("../../../../artifacts/cpp-reference/font-cascade-results.tsv"),
            ),
            (
                include_str!("../../../../artifacts/cpp-reference/box-cascade.html"),
                include_str!("../../../../artifacts/cpp-reference/box-cascade.css"),
                include_str!("../../../../artifacts/cpp-reference/box-cascade-results.tsv"),
            ),
        ] {
            let document = crate::test_html::Parse(markup);
            let styles = crate::style_resolver::ResolveCssom(
                &document,
                &[cssom::ParseCSS(css)],
                1024.0,
                768.0,
            );
            for line in expected.lines() {
                let fields: Vec<_> = line.split('\t').collect();
                assert_eq!(fields.len(), 26);
                let index = document
                    .elements
                    .iter()
                    .position(|e| e.id.as_deref() == Some(fields[0]))
                    .unwrap();
                let s = &styles.styles[index];
                let e = s.extended.as_ref().unwrap();
                let mut values = vec![
                    Some(e.font_size),
                    Some(e.font_weight),
                    Some(e.font_italic as u8 as f64),
                    e.line_height,
                    e.line_height_percent,
                    s.width,
                    e.width_percent,
                    Some(e.width_calculated as u8 as f64),
                ];
                values.extend(
                    [
                        s.margin.top,
                        s.margin.right,
                        s.margin.bottom,
                        s.margin.left,
                        s.padding.top,
                        s.padding.right,
                        s.padding.bottom,
                        s.padding.left,
                        s.writing_mode as i32 as f64,
                        s.direction as i32 as f64,
                    ]
                    .into_iter()
                    .map(Some),
                );
                let before = styles.before[index]
                    .as_ref()
                    .and_then(|p| p.style.extended.as_ref());
                values.extend([
                    before.map(|p| p.font_size),
                    before.map(|p| p.font_weight),
                    before.and_then(|p| p.line_height),
                    before.and_then(|p| p.line_height_percent),
                ]);
                let numbers: Vec<_> = fields[1..23]
                    .iter()
                    .map(|&n| {
                        if n == "none" {
                            None
                        } else {
                            Some(n.parse::<f64>().unwrap())
                        }
                    })
                    .collect();
                assert_eq!(values, numbers, "{}", fields[0]);
                assert_eq!(e.font_families.join("|"), fields[23], "{}", fields[0]);
                assert_eq!(
                    styles.before[index]
                        .as_ref()
                        .map_or("none", |p| p.text.as_str()),
                    fields[24],
                    "{}",
                    fields[0]
                );
                assert_eq!(e.language, fields[25], "{}", fields[0]);
            }
        }
    }
}
