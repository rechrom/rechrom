#![allow(non_snake_case)]

use cssom::CSSFontFaceRule;
use font_engine::FontFeatureValue;
use layoutng_assembly::internal::layout_input::FontUnicodeRange;
use std::collections::BTreeMap;

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

// cpp: core/css/font_face.cc:1120-1127
// cpp: core/css/resolver/style_builder_converter.cc:676-705
fn ParseFontFeatureSettings(input: &str) -> Option<Vec<FontFeatureValue>> {
    let input = input.trim();
    if input.eq_ignore_ascii_case("normal") {
        return Some(Vec::new());
    }
    let mut features = BTreeMap::new();
    for item in SplitTopLevelCommas(input) {
        let item = item.trim();
        let quote = item.as_bytes().first().copied()?;
        if quote != b'\'' && quote != b'"' {
            return None;
        }
        let end = item.as_bytes()[1..]
            .iter()
            .position(|&byte| byte == quote)?
            + 1;
        let tag = &item.as_bytes()[1..end];
        if tag.len() != 4 || !tag.iter().all(|byte| (0x20..=0x7e).contains(byte)) {
            return None;
        }
        let value = match item[end + 1..].trim().to_ascii_lowercase().as_str() {
            "" | "on" => 1,
            "off" => 0,
            value => value.parse::<i32>().ok()?,
        };
        features.insert(u32::from_be_bytes(tag.try_into().ok()?), value as u32);
    }
    (!features.is_empty()).then(|| {
        features
            .into_iter()
            .map(|(tag, value)| FontFeatureValue { tag, value })
            .collect()
    })
}

// cpp: browser/browser.cc:1114-1128
fn SupportsFontSource(url: &str, format: &str) -> bool {
    if !format.is_empty() {
        return format == "woff"
            || format == "woff-variations"
            || format.starts_with("woff ")
            || format == "woff2"
            || format == "woff2-variations"
            || format.starts_with("woff2 ")
            || matches!(format, "truetype" | "opentype" | "ttf" | "otf");
    }
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase();
    ![".eot", ".svg"].iter().any(|ext| path.ends_with(ext))
}

#[derive(Clone)]
pub(crate) struct PendingFontFace {
    pub family: String,
    pub urls: Vec<String>,
    pub weight: f64,
    pub italic: bool,
    pub font_feature_settings: Vec<FontFeatureValue>,
    pub unicode_ranges: Vec<FontUnicodeRange>,
}

// cpp: browser/browser.cc:1130-1172
pub(crate) fn QueueFontFace(rule: &CSSFontFaceRule) -> Option<PendingFontFace> {
    let mut family = String::new();
    let mut sources = String::new();
    let mut unicode_ranges = String::new();
    let mut font_feature_settings = Vec::new();
    let mut weight = 400.0;
    let mut italic = false;
    for d in &rule.declarations {
        match d.property.as_str() {
            "font-family" => family = Unquote(&d.value).to_owned(),
            "src" => sources = d.value.clone(),
            "unicode-range" => unicode_ranges = d.value.clone(),
            "font-feature-settings" | "-webkit-font-feature-settings" => {
                if let Some(settings) = ParseFontFeatureSettings(&d.value) {
                    font_feature_settings = settings;
                }
            }
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
        font_feature_settings,
        unicode_ranges: ParseUnicodeRanges(&unicode_ranges),
    })
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
        assert!(SupportsFontSource("font.woff", "woff"));
        assert!(SupportsFontSource("font.woff", ""));
    }

    #[test]
    fn parses_font_face_feature_settings_with_last_value_winning() {
        assert_eq!(
            ParseFontFeatureSettings("\"liga\" 0, 'kern' on, \"liga\" 3"),
            Some(vec![
                FontFeatureValue {
                    tag: u32::from_be_bytes(*b"kern"),
                    value: 1,
                },
                FontFeatureValue {
                    tag: u32::from_be_bytes(*b"liga"),
                    value: 3,
                },
            ])
        );
        assert_eq!(ParseFontFeatureSettings("normal"), Some(Vec::new()));
        assert_eq!(ParseFontFeatureSettings("\"bad\" 1"), None);
    }
}
