#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selector {
    Tag(String),
    Class(String),
    Id(String),
}

#[cfg(feature = "layoutng_values")]
pub mod color_parser;
#[cfg(feature = "tokenizer")]
pub mod css_parser_idioms;
#[cfg(feature = "tokenizer")]
pub mod css_parser_token;
#[cfg(feature = "tokenizer")]
pub mod css_tokenizer;
#[cfg(feature = "tokenizer")]
pub mod css_tokenizer_input_stream;
#[cfg(feature = "tokenizer")]
pub mod length_percentage_parser;
#[cfg(feature = "layoutng_values")]
pub mod transform_parser;

#[derive(Clone, Debug)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<(String, String)>,
}

fn without_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(start) = rest.find("/*") {
        result.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after.find("*/").expect("unclosed CSS comment");
        rest = &after[end + 2..];
    }
    result.push_str(rest);
    result
}

#[allow(non_snake_case)]
pub fn ParseDeclarations(source: &str) -> Vec<(String, String)> {
    // cpp: cssom/css_style_sheet.cc:331
    without_comments(source)
        .split(';')
        .filter_map(|declaration| {
            let declaration = declaration.trim();
            if declaration.is_empty() {
                return None;
            }
            let (property, value) = declaration
                .split_once(':')
                .expect("CSS declaration lacks ':'");
            Some((
                property.trim().to_ascii_lowercase(),
                value.trim().to_owned(),
            ))
        })
        .collect()
}

#[allow(non_snake_case)]
pub fn Parse(source: &str) -> Vec<Rule> {
    let mut rules = Vec::new();
    let cleaned = without_comments(source);
    let mut remainder = cleaned.trim();
    while !remainder.is_empty() {
        let open = remainder
            .find('{')
            .expect("CSS rule lacks an opening brace");
        let close = remainder[open + 1..]
            .find('}')
            .expect("CSS rule lacks a closing brace")
            + open
            + 1;
        let selectors = remainder[..open]
            .split(',')
            .map(|selector| {
                let selector = selector.trim();
                if let Some(id) = selector.strip_prefix('#') {
                    assert!(!id.is_empty() && !id.contains(char::is_whitespace));
                    Selector::Id(id.to_owned())
                } else if let Some(class) = selector.strip_prefix('.') {
                    assert!(!class.is_empty() && !class.contains(char::is_whitespace));
                    Selector::Class(class.to_owned())
                } else {
                    assert!(
                        !selector.is_empty() && !selector.contains(char::is_whitespace),
                        "complex selectors are not installed"
                    );
                    Selector::Tag(selector.to_ascii_lowercase())
                }
            })
            .collect();
        let declarations = ParseDeclarations(&remainder[open + 1..close]);
        rules.push(Rule {
            selectors,
            declarations,
        });
        remainder = remainder[close + 1..].trim();
    }
    rules
}
