pub(crate) mod text_decoder;
#[cfg(test)]
extern crate layoutng_replaced;

use dom::persistent_document::DOMNamespace;
use dom::{ParsedDocument as Document, ScriptSource, StyleSource};
use foundation::BlinkString;
use parser::html_entity_parser::{ConsumeHTMLEntity, DecodedHTMLEntity};
use parser::segmented_string::SegmentedString;

pub mod html_parser;
pub mod html_parser_host;
pub(crate) mod html_tag_names;
pub mod parser;
pub use html_parser_host::{HTMLParserHost, ParserElementEvent, ParserElementPhase};

// The static browser adapter uses the same entity consumer as the source
// tokenizer. The extra '<' gives the closed input a non-entity terminator.
fn decode_character_references(input: &str, additional_allowed_character: u16) -> String {
    let mut output = String::with_capacity(input.len());
    let mut remaining = input;
    while let Some(ampersand) = remaining.find('&') {
        output.push_str(&remaining[..ampersand]);
        remaining = &remaining[ampersand + 1..];

        let lookahead = format!("{remaining}<");
        let mut source = SegmentedString::from_string(&BlinkString::from(lookahead.as_str()));
        let mut decoded = DecodedHTMLEntity::default();
        let mut not_enough_characters = false;
        if ConsumeHTMLEntity(
            &mut source,
            &mut decoded,
            &mut not_enough_characters,
            additional_allowed_character,
        ) {
            let consumed = source.NumberOfCharactersConsumed() as usize;
            let mut utf16_units = 0;
            let mut byte_end = 0;
            for character in remaining.chars() {
                if utf16_units >= consumed {
                    break;
                }
                utf16_units += character.len_utf16();
                byte_end += character.len_utf8();
            }
            if utf16_units == consumed {
                output.push_str(&String::from_utf16_lossy(
                    &decoded.data[..decoded.length as usize],
                ));
                remaining = &remaining[byte_end..];
                continue;
            }
        }
        output.push('&');
    }
    output.push_str(remaining);
    output
}

fn decoded_attribute(tag: &str, name: &str) -> Option<String> {
    attribute(tag, name).map(|value| decode_character_references(value, b'>' as u16))
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let bytes = tag.as_bytes();
    let mut index = tag.find(char::is_whitespace)?;
    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] == b'/' {
            break;
        }
        let start = index;
        while index < bytes.len()
            && !bytes[index].is_ascii_whitespace()
            && bytes[index] != b'='
            && bytes[index] != b'/'
        {
            index += 1;
        }
        let attribute_name = &tag[start..index];
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] != b'=' {
            if attribute_name.eq_ignore_ascii_case(name) {
                return Some("");
            }
            continue;
        }
        index += 1;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }
        let quote = bytes[index];
        let value = if quote == b'"' || quote == b'\'' {
            index += 1;
            let value_start = index;
            while index < bytes.len() && bytes[index] != quote {
                index += 1;
            }
            let value = &tag[value_start..index];
            if index < bytes.len() {
                index += 1;
            }
            value
        } else {
            let value_start = index;
            while index < bytes.len() && !bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            &tag[value_start..index]
        };
        if attribute_name.eq_ignore_ascii_case(name) {
            return Some(value);
        }
    }
    None
}

fn all_attributes(tag: &str) -> Vec<(String, String)> {
    let bytes = tag.as_bytes();
    let Some(mut index) = tag.find(char::is_whitespace) else {
        return Vec::new();
    };
    let mut attributes = Vec::new();
    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] == b'/' {
            break;
        }
        let start = index;
        while index < bytes.len()
            && !bytes[index].is_ascii_whitespace()
            && bytes[index] != b'='
            && bytes[index] != b'/'
        {
            index += 1;
        }
        if index == start {
            index += 1;
            continue;
        }
        let name = tag[start..index].to_ascii_lowercase();
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        let mut value = String::new();
        if index < bytes.len() && bytes[index] == b'=' {
            index += 1;
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            if index < bytes.len() && matches!(bytes[index], b'\'' | b'"') {
                let quote = bytes[index];
                index += 1;
                let begin = index;
                while index < bytes.len() && bytes[index] != quote {
                    index += 1;
                }
                value = tag[begin..index].to_owned();
                if index < bytes.len() {
                    index += 1;
                }
            } else {
                let begin = index;
                while index < bytes.len()
                    && !bytes[index].is_ascii_whitespace()
                    && bytes[index] != b'/'
                {
                    index += 1;
                }
                value = tag[begin..index].to_owned();
            }
        }
        attributes.push((name, decode_character_references(&value, b'>' as u16)));
    }
    attributes
}

// cpp: html/html_parser.cc:90-113,934-961
// The static-page adapter keeps the namespace and uses the same foreign-name
// adjustments as the source construction site.
fn append_element(
    document: &mut Document,
    parent: Option<usize>,
    tag: &str,
    name: String,
) -> usize {
    let parent_namespace = parent.map(|index| document.elements[index].namespace);
    let parent_is_html_integration_point = parent.is_some_and(|index| {
        let element = &document.elements[index];
        element.namespace == DOMNamespace::kSVG
            && matches!(element.tag.as_str(), "foreignObject" | "desc" | "title")
    });
    let namespace =
        if parent_namespace == Some(DOMNamespace::kSVG) && !parent_is_html_integration_point {
            DOMNamespace::kSVG
        } else if parent_namespace == Some(DOMNamespace::kMathML) {
            DOMNamespace::kMathML
        } else if name == "svg" {
            DOMNamespace::kSVG
        } else if name == "math" {
            DOMNamespace::kMathML
        } else {
            DOMNamespace::kHTML
        };
    let adjusted_name = parser::html_construction_site::AdjustTagName(name, namespace);
    let attributes = all_attributes(tag)
        .into_iter()
        .map(|(name, value)| {
            let adjusted = parser::html_construction_site::AdjustAttribute(name, value, namespace);
            (adjusted.local_name, adjusted.value)
        })
        .collect();
    document.append_with_namespace(
        parent,
        adjusted_name,
        namespace,
        decoded_attribute(tag, "id"),
        decoded_attribute(tag, "class")
            .map(|value| value.split_ascii_whitespace().map(str::to_owned).collect())
            .unwrap_or_default(),
        decoded_attribute(tag, "style"),
        attributes,
    )
}

fn implicitly_ends_paragraph(name: &str) -> bool {
    matches!(
        name,
        "address" | "div" | "form" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "pre" | "ul"
    )
}

fn close_open_paragraph(document: &Document, stack: &mut Vec<usize>) {
    if stack
        .last()
        .is_some_and(|&index| document.elements[index].tag == "p")
    {
        stack.pop();
    }
}

#[allow(non_snake_case)]
pub fn Parse(source: &str) -> Document {
    let mut document = Document::default();
    let mut stack: Vec<usize> = Vec::new();
    let mut position = 0;
    while position < source.len() {
        let Some(open) = source[position..].find('<').map(|x| position + x) else {
            if let Some(&parent) = stack.last() {
                document.append_text(parent, &decode_character_references(&source[position..], 0));
            } else {
                assert!(
                    source[position..].trim().is_empty(),
                    "text outside HTML root"
                );
            }
            break;
        };
        if let Some(&parent) = stack.last() {
            document.append_text(
                parent,
                &decode_character_references(&source[position..open], 0),
            );
        } else {
            assert!(
                source[position..open].trim().is_empty(),
                "text outside HTML root"
            );
        }
        let close = source[open..].find('>').expect("unclosed HTML tag") + open;
        let tag = source[open + 1..close].trim();
        position = close + 1;
        if tag.starts_with('!') {
            continue;
        }
        if let Some(name) = tag.strip_prefix('/') {
            let name = name.trim().to_ascii_lowercase();
            if name == "head" {
                continue;
            }
            if matches!(
                name.as_str(),
                "html" | "body" | "div" | "address" | "form" | "ul"
            ) {
                close_open_paragraph(&document, &mut stack);
            }
            if name == "ul"
                && stack
                    .last()
                    .is_some_and(|&index| document.elements[index].tag == "li")
            {
                stack.pop();
            }
            let current = stack.pop().expect("unmatched closing tag");
            assert!(
                document.elements[current].tag.eq_ignore_ascii_case(&name),
                "mismatched closing tag: {} vs {name}",
                document.elements[current].tag
            );
            continue;
        }
        let name = tag
            .split_ascii_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches('/')
            .to_ascii_lowercase();
        match name.as_str() {
            "head" => continue,
            "meta" => continue,
            "base" => {
                // cpp: browser/browser.cc:855-862
                if document.base_href.is_none() {
                    if let Some(href) =
                        decoded_attribute(tag, "href").filter(|href| !href.is_empty())
                    {
                        document.base_href = Some(href);
                    }
                }
            }
            "link" => {
                // cpp: browser/browser.cc:864-868
                if attribute(tag, "rel").is_some_and(|tokens| {
                    let mut alternate = false;
                    let mut stylesheet = false;
                    for token in tokens.split_ascii_whitespace() {
                        alternate |= token.eq_ignore_ascii_case("alternate");
                        stylesheet |= token.eq_ignore_ascii_case("stylesheet");
                    }
                    stylesheet && !alternate
                }) && attribute(tag, "media").is_none_or(|media| {
                    media.split(',').any(|query| {
                        let query = query.trim();
                        query.eq_ignore_ascii_case("all")
                            || query.eq_ignore_ascii_case("screen")
                            || query.to_ascii_lowercase().starts_with("screen ")
                    })
                }) {
                    if let Some(href) =
                        decoded_attribute(tag, "href").filter(|href| !href.is_empty())
                    {
                        document.style_sources.push(StyleSource::Link(href));
                    }
                }
            }
            "title" => {
                let rest = &source[position..];
                let end = rest
                    .to_ascii_lowercase()
                    .find("</title>")
                    .expect("unclosed title element");
                position += end + "</title>".len();
            }
            "style" => {
                let rest = &source[position..];
                let end = rest
                    .to_ascii_lowercase()
                    .find("</style>")
                    .expect("unclosed style element");
                document
                    .style_sources
                    .push(StyleSource::Inline(rest[..end].to_owned()));
                position += end + "</style>".len();
            }
            "script" => {
                // Preserve parser order and raw script text for the browser's
                // script scheduler. Markup inside JavaScript is not tokenized.
                let rest = &source[position..];
                let end = rest
                    .to_ascii_lowercase()
                    .find("</script>")
                    .expect("unclosed script element");
                document.script_sources.push(ScriptSource {
                    source: rest[..end].to_owned(),
                    src: decoded_attribute(tag, "src"),
                    script_type: decoded_attribute(tag, "type").unwrap_or_default(),
                    async_attribute: attribute(tag, "async").is_some(),
                    defer_attribute: attribute(tag, "defer").is_some(),
                    no_module: attribute(tag, "nomodule").is_some(),
                });
                position += end + "</script>".len();
            }
            "textarea" => {
                // RCDATA inside a textarea is text, even when it contains
                // strings that look like style or script elements.
                let index = append_element(&mut document, stack.last().copied(), tag, name);
                let rest = &source[position..];
                let end = rest
                    .to_ascii_lowercase()
                    .find("</textarea>")
                    .expect("unclosed textarea element");
                document.append_text(index, &decode_character_references(&rest[..end], 0));
                position += end + "</textarea>".len();
            }
            "img" | "br" | "input" | "hr" | "area" => {
                // HTML void elements are present in the DOM but never push a
                // matching end tag onto the open-element stack.
                append_element(&mut document, stack.last().copied(), tag, name);
            }
            "html" | "body" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "a"
            | "span" | "em" | "pre" | "form" | "ul" | "li" | "button" | "address" | "small"
            | "abbr" | "sup" | "header" | "nav" | "main" | "footer" | "strong" | "code"
            | "samp" | "blockquote" | "svg" | "section" | "noscript" | "i" | "map" => {
                if implicitly_ends_paragraph(&name) {
                    close_open_paragraph(&document, &mut stack);
                }
                if name == "li"
                    && stack
                        .last()
                        .is_some_and(|&index| document.elements[index].tag == "li")
                {
                    stack.pop();
                }
                let index = append_element(&mut document, stack.last().copied(), tag, name);
                stack.push(index);
            }
            _ => {
                // Keep unfamiliar HTML and SVG elements in the static DOM.
                // Their display semantics are resolved downstream; rejecting
                // the tag here prevents otherwise supported descendants from
                // reaching style and layout.
                if implicitly_ends_paragraph(&name) {
                    close_open_paragraph(&document, &mut stack);
                }
                let index = append_element(&mut document, stack.last().copied(), tag, name);
                if !tag.ends_with('/')
                    && !matches!(
                        document.elements[index].tag.as_str(),
                        "base" | "col" | "embed" | "param" | "source" | "track" | "wbr"
                    )
                {
                    stack.push(index);
                }
            }
        }
    }
    while let Some(&index) = stack.last() {
        if matches!(
            document.elements[index].tag.as_str(),
            "p" | "li" | "body" | "html"
        ) {
            stack.pop();
        } else {
            panic!("unclosed HTML element: {}", document.elements[index].tag);
        }
    }
    let root = document.root.expect("missing html element");
    assert_eq!(document.elements[root].tag, "html");
    document
}

#[cfg(test)]
mod tests {
    use super::*;
    use dom::Child;

    #[test]
    fn preserves_text_order_inside_common_page_elements() {
        let page = Parse("<html><body><h1>Hello <span>world</span>!</h1><p>Next <a href=/x>link</a>.</p></body></html>");
        let body = &page.elements[1];
        assert_eq!(body.children, vec![Child::Element(2), Child::Element(4)]);
        assert_eq!(
            page.elements[2].children,
            vec![Child::Text(0), Child::Element(3), Child::Text(2)]
        );
        assert_eq!(
            page.elements[4].children,
            vec![Child::Text(3), Child::Element(5), Child::Text(5)]
        );
        assert_eq!(page.texts, ["Hello ", "world", "!", "Next ", "link", "."]);
    }

    #[test]
    fn records_parser_scripts_in_order_without_tokenizing_script_markup() {
        let page = Parse("<html><body><script>const markup = '<div>x</div>';</script><script src='app.js?a=1&amp;b=2' defer nomodule></script><p>After</p></body></html>");
        assert_eq!(page.script_sources.len(), 2);
        assert_eq!(
            page.script_sources[0].source,
            "const markup = '<div>x</div>';"
        );
        assert_eq!(
            page.script_sources[1].src.as_deref(),
            Some("app.js?a=1&b=2")
        );
        assert!(page.script_sources[1].defer_attribute);
        assert!(page.script_sources[1].no_module);
        assert_eq!(page.texts, ["After"]);
    }

    #[test]
    fn decodes_character_references_in_page_text() {
        let page = Parse("<html><body><button>Menu &#9660;</button><p>A&nbsp;B &amp; C &#x1F600; &bogus;</p></body></html>");
        assert_eq!(page.texts, ["Menu ▼", "A\u{a0}B & C 😀 &bogus;"]);
    }

    #[test]
    fn decodes_character_references_in_resource_urls_and_attributes() {
        let page = Parse("<html><head><link rel=stylesheet href='site.css?a=1&amp;b=2'></head><body><a href='/?x=1&amp;y=2'>Link</a></body></html>");
        assert!(
            matches!(&page.style_sources[0], StyleSource::Link(href) if href == "site.css?a=1&b=2")
        );
        assert!(page.elements.iter().any(|element| element
            .attributes
            .iter()
            .any(|(name, value)| name == "href" && value == "/?x=1&y=2")));
    }

    #[test]
    fn static_adapter_preserves_svg_namespace_and_foreign_case() {
        let page = Parse("<html><body><svg viewbox='0 0 20 20'><clippath id='clip'><rect width='20' height='20'/></clippath><path d='M0 0 L20 20' stroke='currentColor'/></svg></body></html>");
        let svg = &page.elements[2];
        assert_eq!(svg.namespace, DOMNamespace::kSVG);
        assert_eq!(
            svg.attributes
                .iter()
                .find(|(name, _)| name == "viewBox")
                .unwrap()
                .1,
            "0 0 20 20"
        );
        assert_eq!(page.elements[3].tag, "clipPath");
        assert_eq!(page.elements[3].namespace, DOMNamespace::kSVG);
        assert_eq!(page.elements[5].tag, "path");
        assert_eq!(page.elements[5].namespace, DOMNamespace::kSVG);
    }
}
