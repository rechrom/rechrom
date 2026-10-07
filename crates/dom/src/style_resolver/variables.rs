#![allow(non_snake_case)]
use crate::style_resolver::selector::SourceSpace;
use std::collections::HashMap;

// cpp: style_resolver/style_resolver.cc:7020-7046
fn FindMatchingParenthesis(value: &[u8], open: usize) -> Option<usize> {
    let (mut depth, mut quote, mut cursor) = (1u32, 0u8, open + 1);
    while cursor < value.len() {
        let character = value[cursor];
        if quote != 0 {
            if character == b'\\' && cursor + 1 < value.len() {
                cursor += 1;
            } else if character == quote {
                quote = 0;
            }
        } else if matches!(character, b'\'' | b'"') {
            quote = character;
        } else if character == b'(' {
            depth += 1;
        } else if character == b')' {
            depth -= 1;
            if depth == 0 {
                return Some(cursor);
            }
        }
        cursor += 1;
    }
    None
}

// cpp: style_resolver/style_resolver.cc:7048-7069
fn FindTopLevelComma(value: &[u8]) -> Option<usize> {
    let (mut depth, mut quote, mut cursor) = (0u32, 0u8, 0usize);
    while cursor < value.len() {
        let character = value[cursor];
        if quote != 0 {
            if character == b'\\' && cursor + 1 < value.len() {
                cursor += 1;
            } else if character == quote {
                quote = 0;
            }
        } else if matches!(character, b'\'' | b'"') {
            quote = character;
        } else if character == b'(' {
            depth += 1;
        } else if character == b')' && depth != 0 {
            depth -= 1;
        } else if character == b',' && depth == 0 {
            return Some(cursor);
        }
        cursor += 1;
    }
    None
}

// cpp: style_resolver/style_resolver.cc:7071-7127
pub(crate) fn ResolveVariables(
    value: &str,
    properties: &HashMap<String, Option<String>>,
    resolving: &mut Vec<String>,
    depth: u32,
) -> Option<String> {
    if depth > 64 {
        return None;
    }
    let bytes = value.as_bytes();
    let (mut result, mut quote, mut cursor) = (Vec::<u8>::new(), 0u8, 0usize);
    while cursor < bytes.len() {
        let character = bytes[cursor];
        if quote != 0 {
            result.push(character);
            cursor += 1;
            if character == b'\\' && cursor < bytes.len() {
                result.push(bytes[cursor]);
                cursor += 1;
            } else if character == quote {
                quote = 0;
            }
            continue;
        }
        if matches!(character, b'\'' | b'"') {
            quote = character;
            result.push(character);
            cursor += 1;
            continue;
        }
        if bytes
            .get(cursor..cursor + 4)
            .is_some_and(|s| s.eq_ignore_ascii_case(b"var("))
        {
            let close = FindMatchingParenthesis(bytes, cursor + 3)?;
            let arguments = &value[cursor + 4..close];
            let comma = FindTopLevelComma(arguments.as_bytes());
            let name = arguments[..comma.unwrap_or(arguments.len())]
                .trim_matches(|c: char| c.source_space());
            if !name.starts_with("--") || name.contains([' ', '\t', '\r', '\n', '(', ')']) {
                return None;
            }
            let mut replacement = None;
            if !resolving.iter().any(|v| v == name) {
                if let Some(Some(variable)) = properties.get(name) {
                    resolving.push(name.to_owned());
                    replacement = ResolveVariables(variable, properties, resolving, depth + 1);
                    resolving.pop();
                }
            }
            if replacement.is_none() {
                if let Some(comma) = comma {
                    replacement =
                        ResolveVariables(&arguments[comma + 1..], properties, resolving, depth + 1);
                }
            }
            result.extend_from_slice(replacement?.as_bytes());
            cursor = close + 1;
            continue;
        }
        result.push(character);
        cursor += 1;
    }
    String::from_utf8(result).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_fallback_cycles_quotes_and_recursion_limit_match_cpp() {
        let mut properties: HashMap<String, Option<String>> = [
            ("--x", Some("20px")),
            ("--a", Some("var(--b)")),
            ("--b", Some("var(--a)")),
            ("--quoted", Some("'var(--missing)'")),
            ("--null", None),
            ("--unicode", Some("中文")),
            ("--function", Some("rgb(20, 40, 60)")),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.map(str::to_owned)))
        .collect();
        for index in 0..66 {
            properties.insert(
                format!("--d{index}"),
                Some(format!("var(--d{})", index + 1)),
            );
        }
        properties.insert("--d66".into(), Some("leaf".into()));
        let decode = |hex: &str| {
            String::from_utf8(
                hex.as_bytes()
                    .chunks_exact(2)
                    .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                    .collect(),
            )
            .unwrap()
        };
        for line in
            include_str!("../../../../artifacts/cpp-reference/variables-results.tsv").lines()
        {
            let fields: Vec<_> = line.split('\t').collect();
            let expected = (fields[1] != "none").then(|| decode(fields[1]));
            let mut resolving = Vec::new();
            assert_eq!(
                ResolveVariables(&decode(fields[0]), &properties, &mut resolving, 0),
                expected,
                "{line}"
            );
            assert!(
                resolving.is_empty(),
                "the source restores the cycle stack after recursion"
            );
        }
    }
}
