#![allow(non_snake_case)]

use std::{
    any::Any,
    fmt,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(test)]
use css_parser::css_parser_token::{BlockType, CSSParserTokenType};
#[cfg(test)]
use css_parser::css_tokenizer::CSSTokenizer;
#[cfg(test)]
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
    /// Exact source inside the rule block. Production Style parses this with
    /// the translated Blink declaration parser; `declarations` is retained
    /// only for the CSSOM-facing rule API during its ownership migration.
    pub declaration_text: String,
    pub declarations: Vec<CSSDeclaration>,
    pub media_conditions: Vec<String>,
    pub layer_name: String,
    pub scope_conditions: Vec<crate::CSSStyleScope>,
    pub container_conditions: Vec<crate::container_condition::CSSContainerQuerySet>,
}

// cpp: cssom/css_style_sheet.h:27-31
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CSSFontFaceRule {
    pub declaration_text: String,
    pub declarations: Vec<CSSDeclaration>,
    pub media_conditions: Vec<String>,
    pub layer_name: String,
}

/// One selector block inside an author `@keyframes` rule.  Keep the selector
/// text (rather than prematurely converting it to a single offset): CSS permits
/// comma-separated offsets and the `from`/`to` aliases.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CSSKeyframeRule {
    pub key_text: String,
    pub declaration_text: String,
    pub declarations: Vec<CSSDeclaration>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CSSKeyframesRule {
    pub name: String,
    pub keyframes: Vec<CSSKeyframeRule>,
    pub media_conditions: Vec<String>,
    pub layer_name: String,
}

/// Browser-owned user-agent stylesheet role. Blink keeps these as distinct
/// `CSSDefaultStyleSheets` owners and `StyleResolver::MatchUARules` selects one
/// from the element namespace. The role lives on the CSSOM owner so the
/// existing `StyleEngine::Update(..., &[CSSStyleSheet])` package boundary can
/// carry that distinction without exposing Style's RuleSet type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSUserAgentStyleSheetRole {
    kHTML,
    /// Supplemental HTML rules selected only for quirks-mode documents.
    /// Chromium owns these in `default_html_quirks_style_` rather than
    /// merging them into the ordinary HTML RuleSet.
    kHTMLQuirks,
    /// Supplemental theme rules selected when the document media environment
    /// is in forced-colors mode.
    kForcedColors,
    /// Default rules for generated `::marker` pseudos. Chromium loads this as
    /// its own `CSSDefaultStyleSheets::marker_style_sheet_` on first marker
    /// request; keeping the role on the CSSOM owner preserves that RuleSet
    /// boundary across `StyleEngine::Update`.
    kMarker,
    /// Supplemental rules selected while the document retains at least one
    /// element with the Fullscreen API fullscreen flag.
    kFullscreen,
    kSVG,
    kMathML,
}

// cpp: cssom/css_style_sheet.h:33-41
#[derive(Clone, Default)]
pub struct CSSStyleSheet {
    pub rules: Vec<CSSStyleRule>,
    pub font_faces: Vec<CSSFontFaceRule>,
    pub keyframes: Vec<CSSKeyframesRule>,
    pub scopes: Vec<crate::CSSScopeRule>,
    pub property_rules: Vec<crate::CSSPropertyRule>,
    pub layer_order: Vec<String>,
    pub owner_node_id: u64,
    /// Link-owned sheet state. Inline and constructed sheets retain the
    /// defaults. The owner document keeps inactive alternate sheets in CSSOM,
    /// while Style only consumes the active subset.
    pub disabled: bool,
    pub alternate: bool,
    pub title: String,
    /// Media attached to the owning `<link>`. Nested `@media` conditions stay
    /// on individual rules; this outer list is prepended when the sheet is
    /// attached so the existing media evaluator sees both levels.
    pub owner_media_conditions: Vec<String>,
    // The projection stays independent of Style's concrete parser types.  The
    // erased owner is only a transfer slot: StyleEngine downcasts it into its
    // private typed-sheet sidecar, while CSSOM observes identity, revision and
    // exact source only.
    provenance: CSSStyleSheetProvenance,
}

#[derive(Clone, Default)]
struct CSSStyleSheetProvenance {
    identity: u64,
    revision: u64,
    source_text: String,
    source_is_current: bool,
    parsed_backing: Option<Rc<dyn Any>>,
    user_agent_role: Option<CSSUserAgentStyleSheetRole>,
}

static NEXT_STYLE_SHEET_IDENTITY: AtomicU64 = AtomicU64::new(1);

impl fmt::Debug for CSSStyleSheet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CSSStyleSheet")
            .field("rules", &self.rules)
            .field("font_faces", &self.font_faces)
            .field("keyframes", &self.keyframes)
            .field("scopes", &self.scopes)
            .field("property_rules", &self.property_rules)
            .field("layer_order", &self.layer_order)
            .field("owner_node_id", &self.owner_node_id)
            .field("disabled", &self.disabled)
            .field("alternate", &self.alternate)
            .field("title", &self.title)
            .field("owner_media_conditions", &self.owner_media_conditions)
            .field("identity", &self.provenance.identity)
            .field("revision", &self.provenance.revision)
            .field("source_text", &self.provenance.source_text)
            .field("source_is_current", &self.provenance.source_is_current)
            .field("user_agent_role", &self.provenance.user_agent_role)
            .finish()
    }
}

impl PartialEq for CSSStyleSheet {
    fn eq(&self, other: &Self) -> bool {
        self.rules == other.rules
            && self.font_faces == other.font_faces
            && self.keyframes == other.keyframes
            && self.scopes == other.scopes
            && self.property_rules == other.property_rules
            && self.layer_order == other.layer_order
            && self.owner_node_id == other.owner_node_id
            && self.disabled == other.disabled
            && self.alternate == other.alternate
            && self.title == other.title
            && self.owner_media_conditions == other.owner_media_conditions
            && self.provenance.source_text == other.provenance.source_text
            && self.provenance.user_agent_role == other.provenance.user_agent_role
    }
}

impl Eq for CSSStyleSheet {}

impl CSSStyleSheet {
    /// Attach the typed result which produced this CSSOM projection. CSSOM
    /// deliberately erases its type so the dependency remains Style -> CSSOM.
    pub fn AttachParsedBacking<T: Any>(&mut self, source_text: &str, parsed: Rc<T>) {
        if self.provenance.identity == 0 {
            self.provenance.identity = NEXT_STYLE_SHEET_IDENTITY.fetch_add(1, Ordering::Relaxed);
            assert_ne!(self.provenance.identity, 0, "stylesheet identity exhausted");
        }
        if self.provenance.revision == 0 {
            self.provenance.revision = 1;
        }
        self.provenance.source_text.clear();
        self.provenance.source_text.push_str(source_text);
        self.provenance.source_is_current = true;
        self.provenance.parsed_backing = Some(parsed);
    }

    pub fn Identity(&self) -> u64 {
        self.provenance.identity
    }

    pub fn Revision(&self) -> u64 {
        self.provenance.revision
    }

    pub fn SourceText(&self) -> &str {
        &self.provenance.source_text
    }

    pub fn HasCurrentSource(&self) -> bool {
        self.provenance.source_is_current
    }

    pub fn ParsedBacking<T: Any>(&self) -> Option<Rc<T>> {
        self.provenance
            .parsed_backing
            .as_ref()
            .cloned()
            .and_then(|owner| owner.downcast::<T>().ok())
    }

    pub fn SetUserAgentStyleSheetRole(&mut self, role: CSSUserAgentStyleSheetRole) {
        self.provenance.user_agent_role = Some(role);
    }

    pub fn UserAgentStyleSheetRole(&self) -> Option<CSSUserAgentStyleSheetRole> {
        self.provenance.user_agent_role
    }

    pub(crate) fn PrepareOwnerReplacement(&mut self, previous: &Self) {
        let replacement_has_fresh_owner = self.provenance.identity != previous.provenance.identity
            || self.provenance.revision > previous.provenance.revision;
        let contents_changed = self.rules != previous.rules
            || self.font_faces != previous.font_faces
            || self.keyframes != previous.keyframes
            || self.scopes != previous.scopes
            || self.property_rules != previous.property_rules
            || self.layer_order != previous.layer_order
            || self.provenance.source_text != previous.provenance.source_text;
        self.provenance.identity = previous.provenance.identity;
        self.provenance.user_agent_role = previous.provenance.user_agent_role;
        self.provenance.revision = previous
            .provenance
            .revision
            .checked_add(1)
            .expect("stylesheet revision exhausted");
        if contents_changed && !replacement_has_fresh_owner {
            // A caller edited a cloned CSSOM projection directly. Never expose
            // the old typed owner under the new revision; StyleEngine can
            // recover once from exact source when one is available.
            self.provenance.parsed_backing = None;
            self.provenance.source_is_current = false;
        }
    }

    // cpp: CSSStyleSheet::SetMediaQueries; LinkStyle::NotifyFinished.
    pub fn SetLinkMetadata(
        &mut self,
        owner_node_id: u64,
        alternate: bool,
        disabled: bool,
        title: String,
        media: String,
    ) {
        self.owner_node_id = owner_node_id;
        self.alternate = alternate;
        self.disabled = disabled;
        self.title = title;
        let media = media.trim();
        if media.is_empty() || media.eq_ignore_ascii_case("all") {
            return;
        }
        let media = media.to_owned();
        self.owner_media_conditions.push(media.clone());
        for rule in &mut self.rules {
            rule.media_conditions.insert(0, media.clone());
        }
        for face in &mut self.font_faces {
            face.media_conditions.insert(0, media.clone());
        }
        for keyframes in &mut self.keyframes {
            keyframes.media_conditions.insert(0, media.clone());
        }
        for property in &mut self.property_rules {
            property.media_conditions.insert(0, media.clone());
        }
    }

    // cpp: CSSStyleSheet::CanBeActivated. A title identifies a stylesheet
    // set. Persistent untitled sheets are always active; titled sheets are
    // active only when their title is the document's preferred set. An
    // alternate sheet without a title is never activatable.
    pub fn CanBeActivated(&self, preferred_set_name: &str) -> bool {
        !self.disabled
            && (self.title.is_empty() || self.title == preferred_set_name)
            && (!self.alternate || !self.title.is_empty())
    }
}

// cpp: cssom/css_style_sheet.cc:16-37
#[cfg(test)]
fn Trim(text: &str) -> String {
    text.trim_matches(|character: char| character.is_ascii_whitespace())
        .to_owned()
}

#[cfg(test)]
fn LowerASCII(text: &str) -> String {
    text.to_ascii_lowercase()
}

#[cfg(test)]
fn AppendUnique(values: &mut Vec<String>, value: String) {
    if !value.is_empty() && !values.contains(&value) {
        values.push(value);
    }
}

// cpp: cssom/css_style_sheet.cc:39-57
#[cfg(test)]
fn SplitLayerNames(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(Trim)
        .filter(|name| !name.is_empty())
        .collect()
}

#[cfg(test)]
fn QualifiedLayerName(outer: &str, inner: &str) -> String {
    match (outer.is_empty(), inner.is_empty()) {
        (true, _) => inner.to_owned(),
        (_, true) => outer.to_owned(),
        _ => format!("{outer}.{inner}"),
    }
}

// cpp: cssom/css_style_sheet.cc:59-78
#[cfg(test)]
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

#[cfg(test)]
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
        let declaration_text = Range(&tokenizer, body_start, body_end);
        let declarations = ParseDeclarations(&declaration_text);
        if !key_text.is_empty() && !declarations.is_empty() {
            keyframes.push(CSSKeyframeRule {
                key_text,
                declaration_text,
                declarations,
            });
        }
    }
    (!name.is_empty() && !keyframes.is_empty()).then_some(CSSKeyframesRule {
        name,
        keyframes,
        media_conditions: Vec::new(),
        layer_name: String::new(),
    })
}

// cpp: cssom/css_style_sheet.cc:115-119
#[cfg(test)]
fn Range(tokenizer: &CSSTokenizer, start: u32, end: u32) -> String {
    tokenizer
        .StringRangeAt(start, end - start)
        .ToString()
        .Utf8()
}

// cpp: cssom/css_style_sheet.cc:121-193
#[cfg(test)]
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
#[cfg(test)]
pub fn ParseCSS(input: &str) -> CSSStyleSheet {
    InitStringStatics();
    // Preserve comments in the source range. Blink tokenization discards them
    // at grammar boundaries without concatenating the surrounding tokens.
    let source = BlinkString::FromUtf8(input.as_bytes());
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
                let declaration_text = body;
                let declarations = ParseDeclarations(&declaration_text);
                if !declarations.is_empty() {
                    sheet.font_faces.push(CSSFontFaceRule {
                        declaration_text,
                        declarations,
                        ..CSSFontFaceRule::default()
                    });
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
        let declaration_text = Range(&tokenizer, body_start, body_end);
        let declarations = ParseDeclarations(&declaration_text);
        if !selector_text.is_empty() {
            sheet.rules.push(CSSStyleRule {
                selector_text,
                declaration_text,
                declarations,
                ..CSSStyleRule::default()
            });
        }
    }
    sheet
}

// cpp: cssom/css_style_sheet.cc:330-334
#[cfg(test)]
pub fn ParseCSSDeclarationList(input: &str) -> Vec<CSSDeclaration> {
    InitStringStatics();
    // Preserve token boundaries exactly as the stylesheet path does.  The
    // tokenizer consumes comments as CSS whitespace; deleting them first
    // would incorrectly turn `1/**/0px` into `10px`.
    ParseDeclarations(input)
}

#[cfg(test)]
mod tests {
    use super::{ParseCSS, ParseCSSDeclarationList};

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

    #[test]
    fn declaration_lists_preserve_comment_token_boundaries() {
        let declarations = ParseCSSDeclarationList("width:1/**/0px;color:r/**/ed");
        assert_eq!(declarations[0].value, "1/**/0px");
        assert_eq!(declarations[1].value, "r/**/ed");
    }
}
