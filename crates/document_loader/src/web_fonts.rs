#![allow(non_snake_case)]

use cssom::{CSSFontFaceRule, CSSStyleSheet};
use dom::style_resolver::ResolvedStyles;
use layoutng_assembly::internal::layout_input::{FontFace, FontUnicodeRange};
use std::{collections::HashSet, io};

fn Unquote(text: &str) -> &str {
    let text = text.trim();
    if text.len() >= 2
        && ((text.starts_with('\'') && text.ends_with('\''))
            || (text.starts_with('"') && text.ends_with('"')))
    {
        &text[1..text.len() - 1]
    } else {
        text
    }
}

// cpp: browser/browser.cc:80-105
fn ParseHex(text: &str) -> Option<u32> {
    if text.is_empty() || text.len() > 6 {
        return None;
    }
    let value = u32::from_str_radix(text, 16).ok()?;
    (value <= 0x10ffff).then_some(value)
}

// cpp: browser/browser.cc:107-155
fn ParseUnicodeRanges(text: &str) -> Vec<FontUnicodeRange> {
    let mut ranges = Vec::new();
    for part in text.split(',') {
        let part = part.trim();
        if part.len() < 3 || !part[..2].eq_ignore_ascii_case("u+") {
            continue;
        }
        let value = &part[2..];
        let parsed = if let Some(wildcard) = value.find('?') {
            if !value[wildcard..].bytes().all(|b| b == b'?') {
                continue;
            }
            let low = format!(
                "{}{}",
                &value[..wildcard],
                "0".repeat(value.len() - wildcard)
            );
            let high = format!(
                "{}{}",
                &value[..wildcard],
                "f".repeat(value.len() - wildcard)
            );
            ParseHex(&low).zip(ParseHex(&high))
        } else if let Some(dash) = value.find('-') {
            ParseHex(&value[..dash]).zip(ParseHex(&value[dash + 1..]))
        } else {
            ParseHex(value).map(|v| (v, v))
        };
        if let Some((start, end)) = parsed {
            if start <= end {
                ranges.push(FontUnicodeRange { start, end });
            }
        }
    }
    ranges
}

// cpp: browser/browser.cc:481-529
fn SplitTopLevelCommas(input: &str) -> Vec<String> {
    let mut output = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    let mut quote = 0_u8;
    let bytes = input.as_bytes();
    let mut i = 0;
    while i <= bytes.len() {
        let c = bytes.get(i).copied().unwrap_or(b',');
        if quote != 0 {
            if c == b'\\' && i + 1 < bytes.len() {
                i += 1;
            } else if c == quote {
                quote = 0;
            }
        } else if c == b'\'' || c == b'"' {
            quote = c;
        } else if c == b'(' {
            depth += 1;
        } else if c == b')' && depth > 0 {
            depth -= 1;
        } else if c == b',' && depth == 0 {
            output.push(input[start..i].trim().to_owned());
            start = i + 1;
        }
        i += 1;
    }
    output
}

fn FunctionArgument(input: &str, function: &str) -> Option<String> {
    let lower = input.to_ascii_lowercase();
    let name = lower.find(&format!("{function}("))?;
    let start = name + function.len() + 1;
    let bytes = input.as_bytes();
    let mut quote = 0_u8;
    let mut i = start;
    while i < bytes.len() {
        let c = bytes[i];
        if quote != 0 {
            if c == b'\\' && i + 1 < bytes.len() {
                i += 1;
            } else if c == quote {
                quote = 0;
            }
        } else if c == b'\'' || c == b'"' {
            quote = c;
        } else if c == b')' {
            return Some(Unquote(&input[start..i]).to_owned());
        }
        i += 1;
    }
    None
}

// cpp: browser/browser.cc:581-591
fn ParseFontSources(input: &str) -> Vec<(String, String)> {
    SplitTopLevelCommas(input)
        .into_iter()
        .filter_map(|part| {
            let url = FunctionArgument(&part, "url")?;
            if url.is_empty() {
                return None;
            }
            let format = FunctionArgument(&part, "format")
                .unwrap_or_default()
                .to_ascii_lowercase();
            Some((url, format))
        })
        .collect()
}

// cpp: browser/browser.cc:593-603
fn FontWeight(input: &str) -> Option<f64> {
    let value = input.trim().to_ascii_lowercase();
    match value.as_str() {
        "" | "normal" => Some(400.0),
        "bold" => Some(700.0),
        _ => {
            let weight = value.parse::<f64>().ok()?;
            (weight >= 1.0 && weight <= 1000.0).then_some(weight)
        }
    }
}

// cpp: browser/browser.cc:1114-1128
fn SupportsFontSource(url: &str, format: &str) -> bool {
    if !format.is_empty() {
        return format == "woff2"
            || format == "woff2-variations"
            || format.starts_with("woff2 ")
            || matches!(format, "truetype" | "opentype" | "ttf" | "otf");
    }
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase();
    ![".eot", ".woff", ".svg"]
        .iter()
        .any(|ext| path.ends_with(ext))
}

#[derive(Clone)]
pub(crate) struct PendingFontFace {
    pub family: String,
    pub urls: Vec<String>,
    pub weight: f64,
    pub italic: bool,
    pub unicode_ranges: Vec<FontUnicodeRange>,
}

// cpp: browser/browser.cc:1130-1172
pub(crate) fn QueueFontFace(rule: &CSSFontFaceRule) -> Option<PendingFontFace> {
    let mut family = String::new();
    let mut sources = String::new();
    let mut unicode_ranges = String::new();
    let mut weight = 400.0;
    let mut italic = false;
    for d in &rule.declarations {
        match d.property.as_str() {
            "font-family" => family = Unquote(&d.value).to_owned(),
            "src" => sources = d.value.clone(),
            "unicode-range" => unicode_ranges = d.value.clone(),
            "font-weight" => {
                if let Some(v) = FontWeight(&d.value) {
                    weight = v
                }
            }
            "font-style" => {
                let v = d.value.to_ascii_lowercase();
                italic = v == "italic" || v.starts_with("oblique");
            }
            _ => {}
        }
    }
    if family.is_empty() || sources.is_empty() {
        return None;
    }
    let urls = ParseFontSources(&sources)
        .into_iter()
        .filter(|(url, format)| SupportsFontSource(url, format))
        .map(|(url, _)| url)
        .collect::<Vec<_>>();
    if urls.is_empty() {
        return None;
    }
    Some(PendingFontFace {
        family,
        urls,
        weight,
        italic,
        unicode_ranges: ParseUnicodeRanges(&unicode_ranges),
    })
}

// cpp: browser/browser.cc:1174-1236
pub fn LoadUsedFontFaces(
    sheets: &[CSSStyleSheet],
    styles: &ResolvedStyles,
    mut load: impl FnMut(&str) -> io::Result<Vec<u8>>,
) -> Vec<FontFace> {
    let used: HashSet<String> = styles
        .styles
        .iter()
        .flat_map(|s| s.extended.iter())
        .flat_map(|e| e.font_families.iter())
        .map(|name| name.to_ascii_lowercase())
        .collect();
    let mut pending = Vec::<PendingFontFace>::new();
    for sheet in sheets {
        for rule in &sheet.font_faces {
            let Some(face) = QueueFontFace(rule) else {
                continue;
            };
            if !used.contains(&face.family.to_ascii_lowercase()) {
                continue;
            }
            if pending.iter().any(|existing| {
                existing.family.eq_ignore_ascii_case(&face.family)
                    && existing.weight == face.weight
                    && existing.italic == face.italic
                    && existing.urls == face.urls
            }) {
                continue;
            }
            pending.push(face);
        }
    }
    let mut loaded_urls = HashSet::new();
    let mut fonts = Vec::new();
    for face in pending {
        for url in &face.urls {
            if loaded_urls.contains(url) {
                continue;
            }
            let Ok(bytes) = load(url) else { continue };
            let Ok(bytes) = web_font::DecodeWebFont(bytes) else {
                continue;
            };
            if bytes.is_empty() {
                continue;
            }
            loaded_urls.insert(url.clone());
            fonts.push(FontFace {
                family: face.family.clone(),
                weight: face.weight,
                italic: face.italic,
                bytes: bytes.into(),
                unicode_ranges: face.unicode_ranges.clone(),
                ..Default::default()
            });
            break;
        }
    }
    fonts
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_source_format_and_unicode_ranges() {
        assert_eq!(
            ParseUnicodeRanges("U+e600-e6ff, U+4??")[0],
            FontUnicodeRange {
                start: 0xe600,
                end: 0xe6ff
            }
        );
        assert_eq!(
            ParseUnicodeRanges("U+e600-e6ff, U+4??")[1],
            FontUnicodeRange {
                start: 0x400,
                end: 0x4ff
            }
        );
        let values = ParseFontSources(
            "url('font.eot') format('embedded-opentype'), url('font.woff2') format('woff2')",
        );
        assert_eq!(values.len(), 2);
        assert!(!SupportsFontSource(&values[0].0, &values[0].1));
        assert!(SupportsFontSource(&values[1].0, &values[1].1));
    }
}
