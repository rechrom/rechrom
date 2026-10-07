#![allow(non_snake_case)]
use crate::style_resolver::border_radius::Length;
use crate::style_resolver::number::Number;
use crate::style_resolver::selector::{SourceSpace, SplitSelectorList};
use crate::style_resolver::{CSSMediaType, StyleEnvironment};
fn Trim(value: &str) -> &str {
    value.trim_matches(|c: char| c.source_space())
}

// cpp: style_resolver/style_resolver.cc:1069-1103
fn SplitMediaAnd(input: &str) -> Vec<String> {
    let bytes = input.as_bytes();
    let identifier_character =
        |c: u8| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_') || c >= 0x80;
    let (mut start, mut depth, mut quote, mut index) = (0, 0u32, 0u8, 0);
    let mut parts = Vec::new();
    while index < bytes.len() {
        let character = bytes[index];
        if quote != 0 {
            if character == b'\\' && index + 1 < bytes.len() {
                index += 1;
            } else if character == quote {
                quote = 0;
            }
        } else if matches!(character, b'\'' | b'"') {
            quote = character;
        } else if character == b'(' {
            depth += 1;
        } else if character == b')' && depth > 0 {
            depth -= 1;
        } else if depth == 0
            && bytes
                .get(index..index + 3)
                .is_some_and(|s| s.eq_ignore_ascii_case(b"and"))
            && (index == 0 || !identifier_character(bytes[index - 1]))
            && (index + 3 == bytes.len() || !identifier_character(bytes[index + 3]))
        {
            parts.push(Trim(&input[start..index]).into());
            index += 2;
            start = index + 1;
        }
        index += 1;
    }
    parts.push(Trim(&input[start..]).into());
    parts
}
// cpp: style_resolver/style_resolver.cc:1105-1191
fn MediaFeatureMatches(input: &str, environment: &StyleEnvironment) -> bool {
    let feature = Trim(input).to_ascii_lowercase();
    let Some(feature) = feature
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .map(Trim)
    else {
        return false;
    };
    let Some((name, value)) = feature.split_once(':') else {
        if let Some(comparison) = feature.find(['<', '>']) {
            let range_name = Trim(&feature[..comparison]);
            let mut value_start = comparison + 1;
            let inclusive = feature.as_bytes().get(value_start) == Some(&b'=');
            if inclusive {
                value_start += 1;
            }
            let actual = match range_name {
                "width" => environment.viewport_width,
                "height" => environment.viewport_height,
                _ => return false,
            };
            let expected = Length(Trim(&feature[value_start..]), 16.0);
            let (Some(actual), Some(expected)) = (actual, expected) else {
                return false;
            };
            if expected < 0.0 {
                return false;
            }
            return if feature.as_bytes()[comparison] == b'>' {
                if inclusive {
                    actual >= expected
                } else {
                    actual > expected
                }
            } else if inclusive {
                actual <= expected
            } else {
                actual < expected
            };
        }
        return match Trim(feature) {
            "width" => environment.viewport_width.unwrap_or(0.0) > 0.0,
            "height" => environment.viewport_height.unwrap_or(0.0) > 0.0,
            _ => false,
        };
    };
    let (name, value) = (Trim(name), Trim(value));
    if name == "prefers-color-scheme" {
        return match value {
            "dark" => {
                environment.preferred_color_scheme
                    == crate::style_resolver::PreferredColorScheme::Dark
            }
            "light" => {
                environment.preferred_color_scheme
                    == crate::style_resolver::PreferredColorScheme::Light
            }
            _ => false,
        };
    }
    if name == "orientation" {
        let (Some(width), Some(height)) = (environment.viewport_width, environment.viewport_height)
        else {
            return false;
        };
        return match value {
            "landscape" => width >= height,
            "portrait" => height >= width,
            _ => false,
        };
    }
    if matches!(
        name,
        "resolution"
            | "min-resolution"
            | "max-resolution"
            | "-webkit-device-pixel-ratio"
            | "-webkit-min-device-pixel-ratio"
            | "-webkit-max-device-pixel-ratio"
    ) {
        let Some(actual) = environment.resolution_dppx else {
            return false;
        };
        let expected = if let Some(number) = value.strip_suffix("dppx") {
            Number(number)
        } else if let Some(number) = value.strip_suffix("dpi") {
            Number(number).map(|n| n / 96.0)
        } else if let Some(number) = value.strip_suffix("dpcm") {
            Number(number).map(|n| n * 2.54 / 96.0)
        } else if name.contains("device-pixel-ratio") {
            Number(value)
        } else {
            None
        };
        let Some(expected) = expected.filter(|&n| n > 0.0) else {
            return false;
        };
        return if name.contains("min-") {
            actual >= expected
        } else if name.contains("max-") {
            actual <= expected
        } else {
            (actual - expected).abs() < 0.0001
        };
    }
    let actual = match name {
        "width" | "min-width" | "max-width" => environment.viewport_width,
        "height" | "min-height" | "max-height" => environment.viewport_height,
        _ => return false,
    };
    let (Some(actual), Some(expected)) = (actual, Length(value, 16.0)) else {
        return false;
    };
    if expected < 0.0 {
        return false;
    }
    if name.starts_with("min-") {
        actual >= expected
    } else if name.starts_with("max-") {
        actual <= expected
    } else {
        (actual - expected).abs() < 0.0001
    }
}
// cpp: style_resolver/style_resolver.cc:1193-1218
fn SingleMediaQueryMatches(query: &str, environment: &StyleEnvironment) -> bool {
    let query = Trim(query).to_ascii_lowercase();
    let mut query = query.as_str();
    let mut negate = false;
    if let Some(rest) = query.strip_prefix("not ") {
        negate = true;
        query = Trim(rest);
    } else if let Some(rest) = query.strip_prefix("only ") {
        query = Trim(rest);
    }
    let parts = SplitMediaAnd(query);
    if parts.first().is_none_or(|p| p.is_empty()) {
        return false;
    }
    let mut matches = true;
    let mut feature_start = 0;
    if !parts[0].starts_with('(') {
        matches = parts[0] == "all"
            || (parts[0] == "screen" && environment.media_type == CSSMediaType::kScreen)
            || (parts[0] == "print" && environment.media_type == CSSMediaType::kPrint);
        feature_start = 1;
    }
    for part in &parts[feature_start..] {
        matches = matches && MediaFeatureMatches(part, environment);
    }
    if negate {
        !matches
    } else {
        matches
    }
}
// cpp: style_resolver/style_resolver.cc:1220-1225
pub(crate) fn MediaConditionMatches(condition: &str, environment: &StyleEnvironment) -> bool {
    SplitSelectorList(condition)
        .iter()
        .any(|query| SingleMediaQueryMatches(query, environment))
}
// cpp: style_resolver/style_resolver.cc:1227-1233
pub(crate) fn MediaRuleMatches(rule: &cssom::CSSStyleRule, environment: &StyleEnvironment) -> bool {
    rule.media_conditions
        .iter()
        .all(|c| MediaConditionMatches(c, environment))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn injected_color_scheme_defaults_to_light_and_composes_with_media_queries() {
        let mut environment = StyleEnvironment {
            viewport_width: Some(1024.0),
            viewport_height: Some(768.0),
            ..Default::default()
        };
        assert!(MediaConditionMatches(
            "(prefers-color-scheme: light)",
            &environment
        ));
        assert!(!MediaConditionMatches(
            "(prefers-color-scheme: dark)",
            &environment
        ));
        environment.preferred_color_scheme = crate::style_resolver::PreferredColorScheme::Dark;
        for (query, expected) in [
            ("(prefers-color-scheme:dark)", true),
            ("(prefers-color-scheme:light)", false),
            (
                "screen and (prefers-color-scheme: dark) and (min-width: 1024px)",
                true,
            ),
            (
                "(prefers-color-scheme: dark) and (min-width: 1025px)",
                false,
            ),
            ("not all and (prefers-color-scheme: dark)", false),
            ("(prefers-color-scheme: light), (min-width: 1000px)", true),
            ("(prefers-color-scheme: invalid)", false),
        ] {
            assert_eq!(
                MediaConditionMatches(query, &environment),
                expected,
                "{query}"
            );
        }
        assert!(MediaConditionMatches(
            "(prefers-color-scheme: light)",
            &StyleEnvironment::default()
        ));
    }

    #[test]
    fn matches_frozen_current_cpp_reference_for_viewport_resolution_and_query_lists() {
        // Generated independently by style_resolver_reference.cc using the
        // unchanged C++ source. Rust tests never call that executable.
        let cases = include_str!("../../../../artifacts/cpp-reference/media-results.tsv");
        let optional = |value: &str| {
            if value == "none" {
                None
            } else {
                Some(value.parse::<f64>().unwrap())
            }
        };
        for line in cases.lines() {
            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(fields.len(), 6);
            let environment = StyleEnvironment {
                viewport_width: optional(fields[0]),
                viewport_height: optional(fields[1]),
                resolution_dppx: optional(fields[2]),
                media_type: if fields[3] == "print" {
                    CSSMediaType::kPrint
                } else {
                    CSSMediaType::kScreen
                },
                ..Default::default()
            };
            assert_eq!(
                MediaConditionMatches(fields[4], &environment),
                fields[5] == "1",
                "{line}"
            );
        }
    }
}
