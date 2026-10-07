#![allow(non_snake_case)]
use crate::persistent_document::DOMNamespace;
use crate::style_resolver::selector::SourceSpace;

// cpp: style_resolver/style_resolver.cc:7265-7368
// Read attributes directly from either DOM representation. UTF-8 scalars are
// appended as bytes equivalent to the source ConsumeCSSString/AppendUTF8.
pub(crate) fn ParseWithAttributes(
    input: &str,
    namespace: DOMNamespace,
    attribute: impl Fn(&str) -> Option<String>,
) -> Option<(bool, String)> {
    let input = input.trim_matches(|c: char| c.source_space());
    if input.eq_ignore_ascii_case("normal") || input.eq_ignore_ascii_case("none") {
        return Some((false, String::new()));
    }
    let mut text = String::new();
    let mut cursor = 0;
    let mut saw_component = false;
    let bytes = input.as_bytes();
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].source_space() {
            cursor += 1;
        }
        if cursor == bytes.len() {
            break;
        }
        if matches!(bytes[cursor], b'\'' | b'"') {
            let quote = bytes[cursor];
            cursor += 1;
            let mut closed = false;
            while cursor < bytes.len() {
                let byte = bytes[cursor];
                cursor += 1;
                if byte == quote {
                    closed = true;
                    break;
                }
                if matches!(byte, b'\n' | b'\r' | 0x0c) {
                    return None;
                }
                if byte != b'\\' {
                    let character = input[cursor - 1..].chars().next()?;
                    text.push(character);
                    cursor += character.len_utf8() - 1;
                    continue;
                }
                if cursor == bytes.len() {
                    return None;
                }
                if bytes[cursor] == b'\n' {
                    cursor += 1;
                    continue;
                }
                if bytes[cursor] == b'\r' {
                    cursor += 1;
                    if cursor < bytes.len() && bytes[cursor] == b'\n' {
                        cursor += 1;
                    }
                    continue;
                }
                if bytes[cursor].is_ascii_hexdigit() {
                    let mut codepoint = 0u32;
                    let mut digits = 0;
                    while cursor < bytes.len() && digits < 6 && bytes[cursor].is_ascii_hexdigit() {
                        codepoint = codepoint * 16 + char::from(bytes[cursor]).to_digit(16)?;
                        cursor += 1;
                        digits += 1;
                    }
                    if cursor < bytes.len() && bytes[cursor].source_space() {
                        cursor += 1;
                    }
                    text.push(
                        char::from_u32(codepoint)
                            .filter(|c| *c != '\0')
                            .unwrap_or(char::REPLACEMENT_CHARACTER),
                    );
                    continue;
                }
                let character = input[cursor..].chars().next()?;
                text.push(character);
                cursor += character.len_utf8();
            }
            if !closed {
                return None;
            }
            saw_component = true;
            continue;
        }
        if input[cursor..]
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("attr("))
        {
            let close = input[cursor + 5..].find(')')? + cursor + 5;
            let mut name = input[cursor + 5..close]
                .trim_matches(|c: char| c.source_space())
                .to_owned();
            if namespace == DOMNamespace::kHTML {
                name.make_ascii_lowercase();
            }
            if name.is_empty()
                || name
                    .chars()
                    .any(|c| matches!(c, ' ' | '\t' | '\r' | '\n' | '(' | ')'))
            {
                return None;
            }
            if let Some(value) = attribute(&name) {
                text.push_str(&value);
            }
            cursor = close + 1;
            saw_component = true;
            continue;
        }
        return None;
    }
    saw_component.then_some((true, text))
}

pub(crate) fn ParseGeneratedContent(
    input: &str,
    node: &crate::persistent_document::DOMNode,
) -> Option<(bool, String)> {
    ParseWithAttributes(input, node.Namespace(), |name| {
        node.FindAttribute(name).map(|a| a.value.clone())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn html_svg_attributes_and_css_strings_match_frozen_cpp() {
        let owner = crate::test_html::html_parser::ParseHTML("<html><body><div id='html' title='中文value' data-x='end'></div><svg id='svg' viewBox='SVGCase' title='中文value'></svg></body></html>");
        let document = owner.GetDocument();
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
            include_str!("../../../../artifacts/cpp-reference/generated-content-results.tsv")
                .lines()
        {
            let fields: Vec<_> = line.split('\t').collect();
            let node = (0..document.NodeCount())
                .map(|i| document.Node(i))
                .find(|n| n.FindAttribute("id").is_some_and(|a| a.value == fields[0]))
                .unwrap();
            let expected = if fields[2] == "none" {
                None
            } else {
                Some((fields[2] == "1", decode(fields[3])))
            };
            assert_eq!(
                ParseGeneratedContent(&decode(fields[1]), node),
                expected,
                "{line}"
            );
        }
    }
}
