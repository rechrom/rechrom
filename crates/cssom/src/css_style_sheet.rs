#![allow(non_snake_case)]

use css_parser::css_parser_token::{BlockType, CSSParserTokenType};
use css_parser::css_tokenizer::CSSTokenizer;
use foundation::{InitStringStatics, String as BlinkString};

// cpp: cssom/css_style_sheet.h:11-15
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CSSDeclaration {
    pub property: String,
    pub value: String,
    pub important: bool,
}

// cpp: cssom/css_style_sheet.h:17-25
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CSSStyleRule {
    pub selector_text: String,
    pub declarations: Vec<CSSDeclaration>,
    pub media_conditions: Vec<String>,
    pub layer_name: String,
}

// cpp: cssom/css_style_sheet.h:27-31
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CSSFontFaceRule {
    pub declarations: Vec<CSSDeclaration>,
}

/// One selector block inside an author `@keyframes` rule.  Keep the selector
/// text (rather than prematurely converting it to a single offset): CSS permits
/// comma-separated offsets and the `from`/`to` aliases.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CSSKeyframeRule {
    pub key_text: String,
    pub declarations: Vec<CSSDeclaration>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CSSKeyframesRule {
    pub name: String,
    pub keyframes: Vec<CSSKeyframeRule>,
    pub media_conditions: Vec<String>,
}

// cpp: cssom/css_style_sheet.h:33-41
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CSSStyleSheet {
    pub rules: Vec<CSSStyleRule>,
    pub font_faces: Vec<CSSFontFaceRule>,
    pub keyframes: Vec<CSSKeyframesRule>,
    pub layer_order: Vec<String>,
    pub owner_node_id: u64,
}

// cpp: cssom/css_style_sheet.cc:16-37
fn Trim(text: &str) -> String {
    text.trim_matches(|character: char| character.is_ascii_whitespace())
        .to_owned()
}

fn LowerASCII(text: &str) -> String {
    text.to_ascii_lowercase()
}

fn AppendUnique(values: &mut Vec<String>, value: String) {
    if !value.is_empty() && !values.contains(&value) {
        values.push(value);
    }
}

// cpp: cssom/css_style_sheet.cc:39-57
fn SplitLayerNames(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(Trim)
        .filter(|name| !name.is_empty())
        .collect()
}

fn QualifiedLayerName(outer: &str, inner: &str) -> String {
    match (outer.is_empty(), inner.is_empty()) {
        (true, _) => inner.to_owned(),
        (_, true) => outer.to_owned(),
        _ => format!("{outer}.{inner}"),
    }
}

// cpp: cssom/css_style_sheet.cc:59-78
fn AppendNestedSheet(
    destination: &mut CSSStyleSheet,
    nested: CSSStyleSheet,
    outer_layer: &str,
    media_condition: Option<&str>,
) {
    if !outer_layer.is_empty() {
        AppendUnique(&mut destination.layer_order, outer_layer.to_owned());
    }
    for layer in nested.layer_order {
        AppendUnique(
            &mut destination.layer_order,
            QualifiedLayerName(outer_layer, &layer),
        );
    }
    for mut rule in nested.rules {
        rule.layer_name = QualifiedLayerName(outer_layer, &rule.layer_name);
        if let Some(condition) = media_condition {
            rule.media_conditions.insert(0, condition.to_owned());
        }
        destination.rules.push(rule);
    }
    destination.font_faces.extend(nested.font_faces);
    for mut keyframes in nested.keyframes {
        if let Some(condition) = media_condition {
            keyframes.media_conditions.insert(0, condition.to_owned());
        }
        destination.keyframes.push(keyframes);
    }
}

fn ParseKeyframes(name: String, body: &str) -> Option<CSSKeyframesRule> {
    let source = BlinkString::FromUtf8(body.as_bytes());
    let mut tokenizer = CSSTokenizer::new(&source, 0);
    let mut keyframes = Vec::new();
    loop {
        let mut token = tokenizer.TokenizeSingleWithComments();
        while matches!(
            token.GetType(),
            CSSParserTokenType::kWhitespaceToken | CSSParserTokenType::kSemicolonToken
        ) {
            token = tokenizer.TokenizeSingleWithComments();
        }
        if token.IsEOF() {
            break;
        }
        let selector_start = tokenizer.PreviousOffset();
        let mut selector_end = selector_start;
        while !token.IsEOF() {
            if token.GetType() == CSSParserTokenType::kLeftBraceToken {
                selector_end = tokenizer.PreviousOffset();
                break;
            }
            token = tokenizer.TokenizeSingleWithComments();
        }
        if token.IsEOF() {
            break;
        }
        let body_start = tokenizer.Offset();
        let mut body_end = body_start;
        let mut depth = 1;
        while depth > 0 {
            let token = tokenizer.TokenizeSingleWithComments();
            if token.IsEOF() {
                body_end = tokenizer.PreviousOffset();
                break;
            }
            match token.GetBlockType() {
                BlockType::kBlockStart => depth += 1,
                BlockType::kBlockEnd => {
                    depth -= 1;
                    if depth == 0 {
                        body_end = tokenizer.PreviousOffset();
                        break;
                    }
                }
                _ => {}
            }
        }
        let key_text = Trim(&Range(&tokenizer, selector_start, selector_end));
        let declarations = ParseDeclarations(&Range(&tokenizer, body_start, body_end));
        if !key_text.is_empty() && !declarations.is_empty() {
            keyframes.push(CSSKeyframeRule {
                key_text,
                declarations,
            });
        }
    }
    (!name.is_empty() && !keyframes.is_empty()).then_some(CSSKeyframesRule {
        name,
        keyframes,
        media_conditions: Vec::new(),
    })
}

// cpp: cssom/css_style_sheet.cc:80-113
fn RemoveComments(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut quote = 0;
    let mut index = 0;
    while index < bytes.len() {
        let character = bytes[index];
        if quote != 0 {
            output.push(character);
            index += 1;
            if character == b'\\' && index < bytes.len() {
                output.push(bytes[index]);
                index += 1;
            } else if character == quote {
                quote = 0;
            }
            continue;
        }
        if character == b'\'' || character == b'"' {
            quote = character;
            output.push(character);
            index += 1;
            continue;
        }
        if character == b'/' && bytes.get(index + 1) == Some(&b'*') {
            index += 2;
            while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
            continue;
        }
        output.push(character);
        index += 1;
    }
    String::from_utf8(output).expect("removing ASCII CSS comments preserves UTF-8")
}

// cpp: cssom/css_style_sheet.cc:115-119
fn Range(tokenizer: &CSSTokenizer, start: u32, end: u32) -> String {
    tokenizer
        .StringRangeAt(start, end - start)
        .ToString()
        .Utf8()
}

// cpp: cssom/css_style_sheet.cc:121-193
fn ParseDeclarations(input: &str) -> Vec<CSSDeclaration> {
    let source = BlinkString::FromUtf8(input.as_bytes());
    let mut tokenizer = CSSTokenizer::new(&source, 0);
    let mut declarations = Vec::new();
    loop {
        let mut token = tokenizer.TokenizeSingleWithComments();
        if token.IsEOF() {
            break;
        }
        if matches!(
            token.GetType(),
            CSSParserTokenType::kWhitespaceToken | CSSParserTokenType::kSemicolonToken
        ) {
            continue;
        }
        if token.GetType() != CSSParserTokenType::kIdentToken {
            while !token.IsEOF() && token.GetType() != CSSParserTokenType::kSemicolonToken {
                token = tokenizer.TokenizeSingleWithComments();
            }
            continue;
        }
        let mut property = token.Value().Utf8();
        if !property.starts_with("--") {
            property = LowerASCII(&property);
            property = match property.as_str() {
                "grid-gap" => "gap".to_owned(),
                "grid-row-gap" => "row-gap".to_owned(),
                "grid-column-gap" => "column-gap".to_owned(),
                _ => property,
            };
        }
        loop {
            token = tokenizer.TokenizeSingleWithComments();
            if token.GetType() != CSSParserTokenType::kWhitespaceToken {
                break;
            }
        }
        if token.GetType() != CSSParserTokenType::kColonToken {
            while !token.IsEOF() && token.GetType() != CSSParserTokenType::kSemicolonToken {
                token = tokenizer.TokenizeSingleWithComments();
            }
            continue;
        }

        let value_start = tokenizer.Offset();
        let mut value_end;
        let mut nested = 0;
        loop {
            token = tokenizer.TokenizeSingleWithComments();
            if token.IsEOF() {
                value_end = tokenizer.PreviousOffset();
                break;
            }
            if nested == 0 && token.GetType() == CSSParserTokenType::kSemicolonToken {
                value_end = tokenizer.PreviousOffset();
                break;
            }
            match token.GetBlockType() {
                BlockType::kBlockStart => nested += 1,
                BlockType::kBlockEnd if nested > 0 => nested -= 1,
                _ => {}
            }
            value_end = tokenizer.Offset();
        }
        let mut value = Trim(&Range(&tokenizer, value_start, value_end));
        let lower = LowerASCII(&value);
        let mut important = false;
        if let Some(marker) = lower.rfind("!important") {
            if Trim(&lower[marker..]) == "!important" {
                important = true;
                value = Trim(&value[..marker]);
            }
        }
        if !property.is_empty() && (!value.is_empty() || property.starts_with("--")) {
            declarations.push(CSSDeclaration {
                property,
                value,
                important,
            });
        }
    }
    declarations
}

// cpp: cssom/css_style_sheet.cc:197-328
pub fn ParseCSS(input: &str) -> CSSStyleSheet {
    InitStringStatics();
    let without_comments = RemoveComments(input);
    let source = BlinkString::FromUtf8(without_comments.as_bytes());
    let mut tokenizer = CSSTokenizer::new(&source, 0);
    let mut sheet = CSSStyleSheet::default();

    loop {
        let mut first = tokenizer.TokenizeSingleWithComments();
        while matches!(
            first.GetType(),
            CSSParserTokenType::kWhitespaceToken | CSSParserTokenType::kSemicolonToken
        ) {
            first = tokenizer.TokenizeSingleWithComments();
        }
        if first.IsEOF() {
            break;
        }

        if first.GetType() == CSSParserTokenType::kAtKeywordToken {
            let at_name = LowerASCII(&first.Value().Utf8());
            let prelude_start = tokenizer.Offset();
            let mut prelude_end = prelude_start;
            let mut found_block = false;
            let mut found_semicolon = false;
            let mut nesting = 0;
            loop {
                let token = tokenizer.TokenizeSingleWithComments();
                if token.IsEOF() {
                    break;
                }
                if nesting == 0 && token.GetType() == CSSParserTokenType::kLeftBraceToken {
                    prelude_end = tokenizer.PreviousOffset();
                    found_block = true;
                    break;
                }
                if nesting == 0 && token.GetType() == CSSParserTokenType::kSemicolonToken {
                    prelude_end = tokenizer.PreviousOffset();
                    found_semicolon = true;
                    break;
                }
                match token.GetBlockType() {
                    BlockType::kBlockStart => nesting += 1,
                    BlockType::kBlockEnd if nesting > 0 => nesting -= 1,
                    _ => {}
                }
            }
            if !found_block {
                if at_name == "layer" && found_semicolon {
                    for name in SplitLayerNames(&Range(&tokenizer, prelude_start, prelude_end)) {
                        AppendUnique(&mut sheet.layer_order, name);
                    }
                }
                continue;
            }

            let body_start = tokenizer.Offset();
            let mut body_end = body_start;
            let mut depth = 1;
            while depth > 0 {
                let token = tokenizer.TokenizeSingleWithComments();
                if token.IsEOF() {
                    body_end = tokenizer.PreviousOffset();
                    break;
                }
                match token.GetBlockType() {
                    BlockType::kBlockStart => depth += 1,
                    BlockType::kBlockEnd => {
                        depth -= 1;
                        if depth == 0 {
                            body_end = tokenizer.PreviousOffset();
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let body = Range(&tokenizer, body_start, body_end);
            if at_name == "media" {
                let condition = Trim(&Range(&tokenizer, prelude_start, prelude_end));
                AppendNestedSheet(&mut sheet, ParseCSS(&body), "", Some(&condition));
            } else if at_name == "layer" {
                let mut layer = Trim(&Range(&tokenizer, prelude_start, prelude_end));
                if layer.is_empty() {
                    layer = format!("__anonymous_{}", sheet.layer_order.len());
                }
                AppendNestedSheet(&mut sheet, ParseCSS(&body), &layer, None);
            } else if at_name == "font-face" {
                let declarations = ParseDeclarations(&body);
                if !declarations.is_empty() {
                    sheet.font_faces.push(CSSFontFaceRule { declarations });
                }
            } else if matches!(at_name.as_str(), "keyframes" | "-webkit-keyframes") {
                let name = Trim(&Range(&tokenizer, prelude_start, prelude_end));
                if let Some(rule) = ParseKeyframes(name, &body) {
                    sheet.keyframes.push(rule);
                }
            }
            continue;
        }

        let selector_start = tokenizer.PreviousOffset();
        let mut selector_end = selector_start;
        let mut found_block = false;
        loop {
            if first.IsEOF() || first.GetType() == CSSParserTokenType::kSemicolonToken {
                break;
            }
            if first.GetType() == CSSParserTokenType::kLeftBraceToken {
                selector_end = tokenizer.PreviousOffset();
                found_block = true;
                break;
            }
            first = tokenizer.TokenizeSingleWithComments();
        }
        if !found_block {
            continue;
        }
        let body_start = tokenizer.Offset();
        let mut body_end = body_start;
        let mut depth = 1;
        while depth > 0 {
            let token = tokenizer.TokenizeSingleWithComments();
            if token.IsEOF() {
                body_end = tokenizer.PreviousOffset();
                break;
            }
            match token.GetBlockType() {
                BlockType::kBlockStart => depth += 1,
                BlockType::kBlockEnd => {
                    depth -= 1;
                    if depth == 0 {
                        body_end = tokenizer.PreviousOffset();
                        break;
                    }
                }
                _ => {}
            }
        }
        let selector_text = Trim(&Range(&tokenizer, selector_start, selector_end));
        let declarations = ParseDeclarations(&Range(&tokenizer, body_start, body_end));
        if !selector_text.is_empty() {
            sheet.rules.push(CSSStyleRule {
                selector_text,
                declarations,
                ..CSSStyleRule::default()
            });
        }
    }
    sheet
}

// cpp: cssom/css_style_sheet.cc:330-334
pub fn ParseCSSDeclarationList(input: &str) -> Vec<CSSDeclaration> {
    InitStringStatics();
    ParseDeclarations(&RemoveComments(input))
}

#[cfg(test)]
mod tests {
    use super::ParseCSS;

    // Behavioral assertions translated from css/css_style_sheet_test.cc:8-69.
    #[test]
    fn preserves_qualified_rules_layers_media_and_font_faces() {
        let sheet = ParseCSS(
            r#"
            main > .card, #hero { display: flex; width: calc(100% - 2rem); }
            [dir="rtl"] p { color: #123456 !important; margin: 1px 2px; }
            @media print { .screen { display: none; }
                @media (min-width: 600px) { .wide-print { width: 600px; } }
            }
            footer { --CardGap: ; --Accent: RED; padding: 8px }
            @font-face {
                font-family: "Fixture Icons";
                src: url(icons.woff2) format("woff2"),
                     url(icons.ttf) format("truetype");
                font-weight: 700;
            }
            @layer reset, components;
            @layer components {
                .button { color: red; }
                @media screen { .layer-wide { width: 10px; } }
            }
        "#,
        );
        assert_eq!(sheet.rules.len(), 7);
        assert_eq!(sheet.rules[0].selector_text, "main > .card, #hero");
        assert_eq!(sheet.rules[0].declarations.len(), 2);
        assert_eq!(sheet.rules[0].declarations[1].value, "calc(100% - 2rem)");
        assert!(sheet.rules[1].declarations[0].important);
        assert_eq!(sheet.rules[2].selector_text, ".screen");
        assert_eq!(sheet.rules[2].media_conditions, ["print"]);
        assert_eq!(sheet.rules[3].selector_text, ".wide-print");
        assert_eq!(
            sheet.rules[3].media_conditions,
            ["print", "(min-width: 600px)"]
        );
        assert_eq!(sheet.rules[4].declarations.len(), 3);
        assert_eq!(sheet.rules[4].declarations[0].property, "--CardGap");
        assert!(sheet.rules[4].declarations[0].value.is_empty());
        assert_eq!(sheet.rules[4].declarations[1].property, "--Accent");
        assert_eq!(sheet.rules[4].declarations[2].property, "padding");
        assert_eq!(sheet.layer_order, ["reset", "components"]);
        assert_eq!(sheet.rules[5].selector_text, ".button");
        assert_eq!(sheet.rules[5].layer_name, "components");
        assert_eq!(sheet.rules[6].selector_text, ".layer-wide");
        assert_eq!(sheet.rules[6].layer_name, "components");
        assert_eq!(sheet.rules[6].media_conditions, ["screen"]);
        assert_eq!(sheet.font_faces.len(), 1);
        assert_eq!(sheet.font_faces[0].declarations.len(), 3);
        assert_eq!(sheet.font_faces[0].declarations[0].property, "font-family");
        assert_eq!(
            sheet.font_faces[0].declarations[0].value,
            "\"Fixture Icons\""
        );
        assert_eq!(sheet.font_faces[0].declarations[1].property, "src");
        assert_eq!(sheet.font_faces[0].declarations[2].value, "700");
    }
}
