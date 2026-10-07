#![allow(non_camel_case_types, non_snake_case, dead_code)]

use dom::persistent_document::{
    CompatibilityMode, DOMAttribute, DOMNamespace, DOMNode, DOMOwnerHandle, PersistentDocument, DOM,
};
use foundation::BlinkString;

use crate::html_parser_host::{HTMLParserHost, ParserElementEvent};
use crate::html_tag_names::LookupHtmlTag;
use crate::parser::html_construction_site::HTMLConstructionSite;
use crate::parser::html_parser_options::HTMLParserOptions;
use crate::parser::html_token::{HTMLToken, TokenType};
use crate::parser::html_tokenizer::HTMLTokenizer;
use crate::parser::literal_buffer::UCharLiteralBuffer;
use crate::parser::segmented_string::{PrependType, SegmentedString};
use dom::error::{invalid_argument, logic_error};
use std::pin::Pin;

// cpp: html/html_parser.h:17-22
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum HTMLParserStatus {
    /// Available input is exhausted; Append may supply the next decoded chunk.
    #[default]
    kNeedMoreInput,
    /// Token budget is exhausted; schedule another turn rather than looping.
    kYielded,
    /// Keep the insertion point paused until the caller executes the script.
    kWaitingForScript,
    /// EOF has been processed. This does not imply DOMContentLoaded or load.
    kFinished,
}

// cpp: html/html_parser.h:24-29
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HTMLSourcePosition {
    pub line: u32,
    pub column: u32,
}
impl Default for HTMLSourcePosition {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

// cpp: html/html_parser.h:31-36
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParserScript {
    pub node_id: u64,
    pub start_position: HTMLSourcePosition,
}

// cpp: html/html_parser.h:38-41
#[derive(Clone, Debug, Default)]
pub struct HTMLParserResult {
    pub status: HTMLParserStatus,
    pub script: Option<ParserScript>,
}

// cpp: html/html_parser.cc:23-29
fn IsHTMLWhitespace(text: &str) -> bool {
    for &character in text.as_bytes() {
        if character != b' '
            && character != b'\t'
            && character != b'\n'
            && character != b'\r'
            && character != b'\x0C'
        {
            return false;
        }
    }
    true
}

// cpp: html/html_parser.cc:31-38
fn IsOneOf(name: &str, names: &[&str]) -> bool {
    for &candidate in names {
        if name == candidate {
            return true;
        }
    }
    false
}

// cpp: html/html_parser.cc:40-44
fn IsVoidElement(name: &str) -> bool {
    IsOneOf(
        name,
        &[
            "area", "base", "basefont", "bgsound", "br", "col", "embed", "frame", "hr", "img",
            "input", "keygen", "link", "meta", "param", "source", "track", "wbr",
        ],
    )
}

// cpp: html/html_parser.cc:46-52
fn ClosesParagraph(name: &str) -> bool {
    IsOneOf(
        name,
        &[
            "address",
            "article",
            "aside",
            "blockquote",
            "div",
            "dl",
            "fieldset",
            "footer",
            "form",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "header",
            "hgroup",
            "hr",
            "main",
            "menu",
            "nav",
            "ol",
            "p",
            "pre",
            "search",
            "section",
            "table",
            "ul",
        ],
    )
}

// cpp: html/html_parser.cc:54-57
fn IsFormattingElement(name: &str) -> bool {
    IsOneOf(
        name,
        &[
            "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike", "strong",
            "tt", "u",
        ],
    )
}

// cpp: html/html_parser.cc:59-72
fn IsSpecialElement(name: &str) -> bool {
    IsOneOf(
        name,
        &[
            "address",
            "applet",
            "area",
            "article",
            "aside",
            "base",
            "basefont",
            "bgsound",
            "blockquote",
            "body",
            "br",
            "button",
            "caption",
            "center",
            "col",
            "colgroup",
            "dd",
            "details",
            "dir",
            "div",
            "dl",
            "dt",
            "embed",
            "fieldset",
            "figcaption",
            "figure",
            "footer",
            "form",
            "frame",
            "frameset",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "head",
            "header",
            "hgroup",
            "hr",
            "html",
            "iframe",
            "img",
            "input",
            "li",
            "link",
            "listing",
            "main",
            "marquee",
            "menu",
            "meta",
            "nav",
            "noembed",
            "noframes",
            "noscript",
            "object",
            "ol",
            "p",
            "param",
            "plaintext",
            "pre",
            "script",
            "search",
            "section",
            "select",
            "source",
            "style",
            "summary",
            "table",
            "tbody",
            "td",
            "template",
            "textarea",
            "tfoot",
            "th",
            "thead",
            "title",
            "tr",
            "track",
            "ul",
            "wbr",
            "xmp",
        ],
    )
}

// cpp: html/html_parser.cc:74-81
fn StartsBlockElement(name: &str) -> bool {
    IsOneOf(
        name,
        &[
            "address",
            "article",
            "aside",
            "blockquote",
            "center",
            "details",
            "dialog",
            "dir",
            "div",
            "dl",
            "fieldset",
            "figcaption",
            "figure",
            "footer",
            "form",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "header",
            "hgroup",
            "li",
            "listing",
            "main",
            "menu",
            "nav",
            "ol",
            "p",
            "pre",
            "search",
            "section",
            "summary",
            "table",
            "tbody",
            "td",
            "tfoot",
            "th",
            "thead",
            "tr",
            "ul",
        ],
    )
}

// cpp: html/html_parser.cc:83-88
fn LowerASCII(mut value: String) -> String {
    value.make_ascii_lowercase();
    value
}

// cpp: html/html_parser.cc:90-92
fn Attribute<'a>(node: &'a DOMNode, name: &str) -> Option<&'a DOMAttribute> {
    node.FindAttribute(name)
}

// cpp: html/html_parser.cc:94-105
fn IsHTMLIntegrationPoint(node: &DOMNode) -> bool {
    if node.Namespace() == DOMNamespace::kSVG {
        return IsOneOf(
            &LowerASCII(node.Name().to_owned()),
            &["foreignobject", "desc", "title"],
        );
    }
    if node.Namespace() != DOMNamespace::kMathML || node.Name() != "annotation-xml" {
        return false;
    }
    let Some(encoding) = Attribute(node, "encoding") else {
        return false;
    };
    let value = LowerASCII(encoding.value.clone());
    value == "text/html" || value == "application/xhtml+xml"
}

// cpp: html/html_parser.cc:107-110
fn IsMathMLTextIntegrationPoint(node: &DOMNode) -> bool {
    node.Namespace() == DOMNamespace::kMathML
        && IsOneOf(node.Name(), &["mi", "mo", "mn", "ms", "mtext"])
}

// cpp: html/html_parser.cc:112-116
fn ProcessesStartTagAsHTML(current: &DOMNode, name: &str) -> bool {
    if IsHTMLIntegrationPoint(current) {
        return true;
    }
    IsMathMLTextIntegrationPoint(current) && !IsOneOf(name, &["mglyph", "malignmark"])
}

// cpp: html/html_parser.cc:118-134
fn BreaksOutOfForeignContent(token: &HTMLToken, name: &str) -> bool {
    if IsOneOf(
        name,
        &[
            "b",
            "big",
            "blockquote",
            "body",
            "br",
            "center",
            "code",
            "dd",
            "div",
            "dl",
            "dt",
            "em",
            "embed",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "head",
            "hr",
            "i",
            "img",
            "li",
            "listing",
            "menu",
            "meta",
            "nobr",
            "ol",
            "p",
            "pre",
            "ruby",
            "s",
            "small",
            "span",
            "strong",
            "strike",
            "sub",
            "sup",
            "table",
            "tt",
            "u",
            "ul",
            "var",
        ],
    ) {
        return true;
    }
    if name != "font" {
        return false;
    }
    for attribute in token.Attributes() {
        let attribute_name = attribute.GetName().Utf8();
        if IsOneOf(&attribute_name, &["color", "face", "size"]) {
            return true;
        }
    }
    false
}

// cpp: html/html_parser.cc:136-138
fn BufferToUtf8(buffer: &UCharLiteralBuffer<256>) -> String {
    buffer.AsString().Utf8()
}

// cpp: html/html_parser.cc:140-142
fn IdentifierToUtf8(value: &[u16]) -> String {
    BlinkString::from_utf16(value).Utf8()
}

// cpp: html/html_parser.cc:225-233
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TableMode {
    kNone,
    kTable,
    kColumnGroup,
    kSection,
    kRow,
    kCell,
    kCaption,
}

// cpp: html/html_parser.cc:235-239
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartTagDisposition {
    kProcessInBody,
    kFosterParent,
    kHandled,
}

// cpp: html/html_parser.cc:149-156,1107-1118
// The C++ document_ reference is accessed through site_ in Rust: the site
// already owns the document's exclusive borrow. Stable arena indices stand
// in for DOMNode* fields, with None preserving null formatting markers.
struct DocumentTreeBuilder<'a> {
    site_: HTMLConstructionSite<'a>,
    html_: Option<usize>,
    head_: Option<usize>,
    body_: Option<usize>,
    active_formatting_elements_: Vec<Option<usize>>,
    head_closed_: bool,
    frameset_document_: bool,
    last_start_tag_was_html_: bool,
    current_script_start_: Option<HTMLSourcePosition>,
    pending_script_: Option<ParserScript>,
    fragment_context_: Option<usize>,
}

impl<'a> DocumentTreeBuilder<'a> {
    // cpp: html/html_parser.cc:151-156
    fn new(
        document: &'a mut PersistentDocument,
        host: &'a mut dyn HTMLParserHost,
        fragment_context: Option<usize>,
    ) -> Self {
        let mut site = HTMLConstructionSite::new(document, Some(host));
        if let Some(context) = fragment_context {
            site.PushAlreadyParsed(context);
        }
        Self {
            site_: site,
            html_: None,
            head_: None,
            body_: None,
            active_formatting_elements_: Vec::new(),
            head_closed_: false,
            frameset_document_: false,
            last_start_tag_was_html_: true,
            current_script_start_: None,
            pending_script_: None,
            fragment_context_: fragment_context,
        }
    }

    // cpp: html/html_parser.cc:158-210
    fn Process(&mut self, token: &HTMLToken, token_position: HTMLSourcePosition) {
        match token.GetType() {
            TokenType::DOCTYPE => {
                self.ProcessDoctype(token);
            }
            TokenType::kStartTag => {
                self.last_start_tag_was_html_ = true;
                let previous_current = self.site_.CurrentElement();
                let name = BufferToUtf8(token.GetName());
                self.ProcessStartTag(token);
                let current = self.site_.CurrentElement();
                if name == "script"
                    && current.is_some()
                    && current != previous_current
                    && LowerASCII(
                        self.site_
                            .OwnerDocument()
                            .Node(current.expect("current element"))
                            .Name()
                            .to_owned(),
                    ) == "script"
                {
                    self.current_script_start_ = Some(token_position);
                }
            }
            TokenType::kEndTag => {
                let current = self.site_.CurrentElement();
                let name = BufferToUtf8(token.GetName());
                let script = current.filter(|&node| {
                    name == "script"
                        && LowerASCII(self.site_.OwnerDocument().Node(node).Name().to_owned())
                            == "script"
                });
                self.ProcessEndTag(token);
                if let Some(script) = script {
                    self.pending_script_ = Some(ParserScript {
                        node_id: self.site_.OwnerDocument().Node(script).Id(),
                        start_position: self.current_script_start_.unwrap_or(token_position),
                    });
                    self.current_script_start_ = None;
                }
            }
            TokenType::kComment => {
                self.site_.InsertComment(BufferToUtf8(token.Comment()));
            }
            TokenType::kCharacter => {
                self.ProcessCharacters(&BufferToUtf8(token.Characters()));
            }
            TokenType::kProcessingInstruction => {
                let instruction = self.site_.OwnerDocumentMut().CreateProcessingInstruction(
                    BufferToUtf8(token.GetProcessingInstructionTarget()),
                    BufferToUtf8(token.Data()),
                );
                let parent = self.site_.CurrentNode();
                self.site_
                    .OwnerDocumentMut()
                    .AppendChild(parent, instruction);
            }
            TokenType::kEndOfFile | TokenType::kUninitialized => return,
        }
    }

    // cpp: html/html_parser.cc:212
    fn LastStartTagWasHTML(&self) -> bool {
        self.last_start_tag_was_html_
    }

    // cpp: html/html_parser.cc:214-218
    fn TakePendingScript(&mut self) -> Option<ParserScript> {
        let result = self.pending_script_;
        self.pending_script_ = None;
        result
    }

    // cpp: html/html_parser.cc:220-222
    fn Finish(&mut self) {
        while self.site_.CurrentElement().is_some() {
            self.site_.Pop();
        }
    }

    // cpp: html/html_parser.cc:241-253
    fn SameFormattingElement(a: &DOMNode, b: &DOMNode) -> bool {
        if a.Name() != b.Name()
            || a.Namespace() != b.Namespace()
            || a.Attributes().len() != b.Attributes().len()
        {
            return false;
        }
        for index in 0..a.Attributes().len() {
            let x = &a.Attributes()[index];
            let y = &b.Attributes()[index];
            if x.prefix != y.prefix
                || x.local_name != y.local_name
                || x.namespace_uri != y.namespace_uri
                || x.value != y.value
            {
                return false;
            }
        }
        true
    }

    // cpp: html/html_parser.cc:255-258
    fn CloneElement(&mut self, source: usize) -> usize {
        let (namespace, name, attributes) = {
            let node = self.site_.OwnerDocument().Node(source);
            (
                node.Namespace(),
                node.Name().to_owned(),
                node.Attributes().to_vec(),
            )
        };
        self.site_
            .OwnerDocumentMut()
            .CreateElement(namespace, name, attributes)
    }

    // cpp: html/html_parser.cc:260-262
    fn FormattingIterator(&self, node: usize) -> Option<usize> {
        self.active_formatting_elements_
            .iter()
            .position(|&entry| entry == Some(node))
    }

    // cpp: html/html_parser.cc:264-283
    fn AddActiveFormattingElement(&mut self, element: usize) {
        let mut identical = Vec::new();
        let mut scope_begin = 0;
        for index in (1..=self.active_formatting_elements_.len()).rev() {
            if self.active_formatting_elements_[index - 1].is_none() {
                scope_begin = index;
                break;
            }
        }
        for index in scope_begin..self.active_formatting_elements_.len() {
            let other = self.active_formatting_elements_[index]
                .expect("formatting marker cannot occur after last marker");
            let document = self.site_.OwnerDocument();
            if Self::SameFormattingElement(document.Node(other), document.Node(element)) {
                identical.push(index);
            }
        }
        if identical.len() >= 3 {
            self.active_formatting_elements_.remove(identical[0]);
        }
        self.active_formatting_elements_.push(Some(element));
    }

    // cpp: html/html_parser.cc:285-306
    fn ReconstructActiveFormattingElements(&mut self, foster_parent: bool) -> bool {
        if self.active_formatting_elements_.is_empty() {
            return false;
        }
        let last = self.active_formatting_elements_.last().copied().flatten();
        if last.is_none_or(|node| self.site_.HasNodeInOpenElements(node)) {
            return false;
        }
        let mut index = self.active_formatting_elements_.len() - 1;
        while index > 0 {
            let previous = self.active_formatting_elements_[index - 1];
            if previous.is_none_or(|node| self.site_.HasNodeInOpenElements(node)) {
                break;
            }
            index -= 1;
        }
        let mut reconstructed = false;
        while index < self.active_formatting_elements_.len() {
            let old = self.active_formatting_elements_[index]
                .expect("active formatting entry must follow the last marker");
            let foster_this = foster_parent && self.CurrentCausesFosterParenting();
            let (name, attributes, namespace) = {
                let node = self.site_.OwnerDocument().Node(old);
                (
                    node.Name().to_owned(),
                    node.Attributes().to_vec(),
                    node.Namespace(),
                )
            };
            let replacement = self.site_.InsertElement(name, attributes, namespace);
            if foster_this {
                self.site_.InsertAtFosterParent(replacement);
            }
            self.active_formatting_elements_[index] = Some(replacement);
            reconstructed = true;
            index += 1;
        }
        reconstructed
    }

    // cpp: html/html_parser.cc:308-388
    fn CallTheAdoptionAgency(&mut self, subject: &str) {
        for _outer in 0..8 {
            let mut formatting = None;
            for index in (0..self.active_formatting_elements_.len()).rev() {
                let Some(node) = self.active_formatting_elements_[index] else {
                    break;
                };
                if self.site_.OwnerDocument().Node(node).IsHTMLElement(subject) {
                    formatting = Some(index);
                    break;
                }
            }
            let Some(formatting_index) = formatting else {
                if self.HasInScope(subject) {
                    self.site_.PopUntil(subject);
                }
                return;
            };
            let formatting_element = self.active_formatting_elements_[formatting_index]
                .expect("formatting entry must contain a node");
            if !self.site_.HasNodeInOpenElements(formatting_element) {
                self.active_formatting_elements_.remove(formatting_index);
                return;
            }

            let open = self.site_.OpenElements();
            let open_formatting = open
                .iter()
                .position(|&node| node == formatting_element)
                .expect("formatting element must be open");
            let mut furthest_block = None;
            for &node in &open[open_formatting + 1..] {
                if IsSpecialElement(self.site_.OwnerDocument().Node(node).Name()) {
                    furthest_block = Some(node);
                    break;
                }
            }
            let Some(furthest_block) = furthest_block else {
                self.site_.PopUntil(subject);
                self.active_formatting_elements_.remove(formatting_index);
                return;
            };
            let Some(common_ancestor) = self.site_.OpenElementBefore(formatting_element) else {
                return;
            };
            let mut bookmark = formatting_index;
            let mut node = furthest_block;
            let mut last_node = furthest_block;
            for inner in 1.. {
                let Some(previous) = self.site_.OpenElementBefore(node) else {
                    break;
                };
                node = previous;
                if node == formatting_element {
                    break;
                }
                let mut active = self.FormattingIterator(node);
                if inner > 3 {
                    if let Some(index) = active {
                        self.active_formatting_elements_.remove(index);
                        active = None;
                    }
                }
                let Some(active_index) = active else {
                    self.site_.RemoveFromOpenElements(node);
                    continue;
                };
                let replacement = self.CloneElement(node);
                self.active_formatting_elements_[active_index] = Some(replacement);
                self.site_.ReplaceInOpenElements(node, replacement);
                if last_node == furthest_block {
                    bookmark = active_index + 1;
                }
                self.site_
                    .OwnerDocumentMut()
                    .AppendChild(replacement, last_node);
                last_node = replacement;
            }
            let common_name = self
                .site_
                .OwnerDocument()
                .Node(common_ancestor)
                .Name()
                .to_owned();
            if IsOneOf(&common_name, &["table", "tbody", "tfoot", "thead", "tr"]) {
                self.site_.InsertAtFosterParent(last_node);
            } else {
                self.site_
                    .OwnerDocumentMut()
                    .AppendChild(common_ancestor, last_node);
            }

            let replacement = self.CloneElement(formatting_element);
            self.site_
                .OwnerDocumentMut()
                .TakeAllChildren(furthest_block, replacement);
            self.site_
                .OwnerDocumentMut()
                .AppendChild(furthest_block, replacement);
            if let Some(index) = self.FormattingIterator(formatting_element) {
                self.active_formatting_elements_.remove(index);
            }
            bookmark = bookmark.min(self.active_formatting_elements_.len());
            self.active_formatting_elements_
                .insert(bookmark, Some(replacement));
            self.site_.RemoveFromOpenElements(formatting_element);
            self.site_
                .InsertInOpenElementsAfter(furthest_block, replacement);
        }
    }

    // cpp: html/html_parser.cc:390-392
    fn InsideTemplate(&self) -> bool {
        self.site_.HasInOpenElements("template")
    }

    // cpp: html/html_parser.cc:394-402
    fn InsideSelect(&self) -> bool {
        for &entry in self.site_.OpenElements().iter().rev() {
            let node = self.site_.OwnerDocument().Node(entry);
            if node.IsHTMLElement("select") {
                return true;
            }
            if node.IsHTMLElement("template") || node.IsHTMLElement("html") {
                return false;
            }
        }
        false
    }

    // cpp: html/html_parser.cc:404-420
    fn CurrentTableMode(&self) -> TableMode {
        for &entry in self.site_.OpenElements().iter().rev() {
            let node = self.site_.OwnerDocument().Node(entry);
            if node.Namespace() != DOMNamespace::kHTML {
                continue;
            }
            if node.Name() == "td" || node.Name() == "th" {
                return TableMode::kCell;
            }
            if node.Name() == "tr" {
                return TableMode::kRow;
            }
            if IsOneOf(node.Name(), &["tbody", "thead", "tfoot"]) {
                return TableMode::kSection;
            }
            if node.Name() == "caption" {
                return TableMode::kCaption;
            }
            if node.Name() == "colgroup" {
                return TableMode::kColumnGroup;
            }
            if node.Name() == "table" {
                return TableMode::kTable;
            }
            if node.Name() == "template" || node.Name() == "html" {
                break;
            }
        }
        TableMode::kNone
    }

    // cpp: html/html_parser.cc:422-428
    fn CurrentCausesFosterParenting(&self) -> bool {
        let Some(current) = self.site_.CurrentElement() else {
            return false;
        };
        let node = self.site_.OwnerDocument().Node(current);
        node.Namespace() == DOMNamespace::kHTML
            && IsOneOf(node.Name(), &["table", "tbody", "tfoot", "thead", "tr"])
    }

    // cpp: html/html_parser.cc:429-436
    fn ClearStackBackTo(&mut self, names: &[&str]) {
        while let Some(current) = self.site_.CurrentElement() {
            let node = self.site_.OwnerDocument().Node(current);
            if node.Namespace() == DOMNamespace::kHTML && IsOneOf(node.Name(), names) {
                return;
            }
            self.site_.Pop();
        }
    }

    // cpp: html/html_parser.cc:438-452
    fn CloseCell(&mut self) {
        let open = self.site_.OpenElements().to_vec();
        for entry in open.into_iter().rev() {
            if self.site_.OwnerDocument().Node(entry).IsHTMLElement("td") {
                self.site_.PopUntil("td");
                self.ClearActiveFormattingThroughLastMarker();
                return;
            }
            if self.site_.OwnerDocument().Node(entry).IsHTMLElement("th") {
                self.site_.PopUntil("th");
                self.ClearActiveFormattingThroughLastMarker();
                return;
            }
        }
    }

    // cpp: html/html_parser.cc:454-457
    fn InsertCell(&mut self, token: &HTMLToken) {
        self.site_
            .InsertElementFromToken(token, DOMNamespace::kHTML);
        self.active_formatting_elements_.push(None);
    }

    // cpp: html/html_parser.cc:459-498
    fn ProcessStartTagInSelect(&mut self, token: &HTMLToken, name: &str) -> StartTagDisposition {
        if !self.InsideSelect() {
            return StartTagDisposition::kProcessInBody;
        }
        if IsOneOf(
            name,
            &[
                "caption", "table", "tbody", "tfoot", "thead", "tr", "td", "th",
            ],
        ) {
            self.site_.PopUntil("select");
            return StartTagDisposition::kProcessInBody;
        }
        if name == "option" {
            if self.site_.CurrentElement().is_some_and(|current| {
                self.site_
                    .OwnerDocument()
                    .Node(current)
                    .IsHTMLElement("option")
            }) {
                self.site_.Pop();
            }
            self.site_
                .InsertElementFromToken(token, DOMNamespace::kHTML);
            return StartTagDisposition::kHandled;
        }
        if name == "optgroup" {
            if self.site_.CurrentElement().is_some_and(|current| {
                self.site_
                    .OwnerDocument()
                    .Node(current)
                    .IsHTMLElement("option")
            }) {
                self.site_.Pop();
            }
            if self.site_.CurrentElement().is_some_and(|current| {
                self.site_
                    .OwnerDocument()
                    .Node(current)
                    .IsHTMLElement("optgroup")
            }) {
                self.site_.Pop();
            }
            self.site_
                .InsertElementFromToken(token, DOMNamespace::kHTML);
            return StartTagDisposition::kHandled;
        }
        if name == "hr" {
            if self.site_.CurrentElement().is_some_and(|current| {
                self.site_
                    .OwnerDocument()
                    .Node(current)
                    .IsHTMLElement("option")
            }) {
                self.site_.Pop();
            }
            if self.site_.CurrentElement().is_some_and(|current| {
                self.site_
                    .OwnerDocument()
                    .Node(current)
                    .IsHTMLElement("optgroup")
            }) {
                self.site_.Pop();
            }
            self.site_
                .InsertElementFromToken(token, DOMNamespace::kHTML);
            self.site_.Pop();
            return StartTagDisposition::kHandled;
        }
        if IsOneOf(name, &["input", "keygen", "textarea", "select"]) {
            self.site_.PopUntil("select");
            return StartTagDisposition::kProcessInBody;
        }
        if name == "script" {
            return StartTagDisposition::kProcessInBody;
        }
        StartTagDisposition::kHandled
    }

    // cpp: html/html_parser.cc:500-612
    fn ProcessStartTagInTable(&mut self, token: &HTMLToken, name: &str) -> StartTagDisposition {
        loop {
            match self.CurrentTableMode() {
                TableMode::kNone => return StartTagDisposition::kProcessInBody,
                TableMode::kCell => {
                    if !IsOneOf(
                        name,
                        &[
                            "caption", "col", "colgroup", "tbody", "td", "tfoot", "th", "thead",
                            "tr", "table",
                        ],
                    ) {
                        return StartTagDisposition::kProcessInBody;
                    }
                    self.CloseCell();
                    continue;
                }
                TableMode::kCaption => {
                    if !IsOneOf(
                        name,
                        &[
                            "caption", "col", "colgroup", "tbody", "tfoot", "thead", "tr", "td",
                            "th", "table",
                        ],
                    ) {
                        return StartTagDisposition::kProcessInBody;
                    }
                    self.site_.PopUntil("caption");
                    self.ClearActiveFormattingThroughLastMarker();
                    continue;
                }
                TableMode::kRow => {
                    if name == "td" || name == "th" {
                        self.ClearStackBackTo(&["tr", "template", "html"]);
                        self.InsertCell(token);
                        return StartTagDisposition::kHandled;
                    }
                    if name == "tr"
                        || IsOneOf(
                            name,
                            &[
                                "caption", "col", "colgroup", "tbody", "tfoot", "thead", "table",
                            ],
                        )
                    {
                        self.site_.PopUntil("tr");
                        continue;
                    }
                    return if self.CurrentCausesFosterParenting() {
                        StartTagDisposition::kFosterParent
                    } else {
                        StartTagDisposition::kProcessInBody
                    };
                }
                TableMode::kSection => {
                    if name == "tr" {
                        self.ClearStackBackTo(&["tbody", "thead", "tfoot", "template", "html"]);
                        self.site_
                            .InsertElementFromToken(token, DOMNamespace::kHTML);
                        return StartTagDisposition::kHandled;
                    }
                    if name == "td" || name == "th" {
                        self.ClearStackBackTo(&["tbody", "thead", "tfoot", "template", "html"]);
                        self.site_
                            .InsertElement("tr".to_owned(), Vec::new(), DOMNamespace::kHTML);
                        self.InsertCell(token);
                        return StartTagDisposition::kHandled;
                    }
                    if IsOneOf(
                        name,
                        &[
                            "caption", "col", "colgroup", "tbody", "tfoot", "thead", "table",
                        ],
                    ) {
                        let current = self.site_.CurrentElement();
                        self.ClearStackBackTo(&["tbody", "thead", "tfoot", "template", "html"]);
                        if self.site_.CurrentElement().is_some_and(|node| {
                            IsOneOf(
                                self.site_.OwnerDocument().Node(node).Name(),
                                &["tbody", "thead", "tfoot"],
                            )
                        }) {
                            self.site_.Pop();
                        } else if current.is_some() {
                            return StartTagDisposition::kHandled;
                        }
                        continue;
                    }
                    return if self.CurrentCausesFosterParenting() {
                        StartTagDisposition::kFosterParent
                    } else {
                        StartTagDisposition::kProcessInBody
                    };
                }
                TableMode::kColumnGroup => {
                    if name == "col" {
                        self.site_
                            .InsertElementFromToken(token, DOMNamespace::kHTML);
                        self.site_.Pop();
                        return StartTagDisposition::kHandled;
                    }
                    self.site_.PopUntil("colgroup");
                    continue;
                }
                TableMode::kTable => {
                    if name == "caption" {
                        self.ClearStackBackTo(&["table", "template", "html"]);
                        self.site_
                            .InsertElementFromToken(token, DOMNamespace::kHTML);
                        self.active_formatting_elements_.push(None);
                        return StartTagDisposition::kHandled;
                    }
                    if name == "colgroup" {
                        self.ClearStackBackTo(&["table", "template", "html"]);
                        self.site_
                            .InsertElementFromToken(token, DOMNamespace::kHTML);
                        return StartTagDisposition::kHandled;
                    }
                    if name == "col" {
                        self.ClearStackBackTo(&["table", "template", "html"]);
                        self.site_.InsertElement(
                            "colgroup".to_owned(),
                            Vec::new(),
                            DOMNamespace::kHTML,
                        );
                        continue;
                    }
                    if IsOneOf(name, &["tbody", "tfoot", "thead"]) {
                        self.ClearStackBackTo(&["table", "template", "html"]);
                        self.site_
                            .InsertElementFromToken(token, DOMNamespace::kHTML);
                        return StartTagDisposition::kHandled;
                    }
                    if name == "tr" {
                        self.ClearStackBackTo(&["table", "template", "html"]);
                        self.site_.InsertElement(
                            "tbody".to_owned(),
                            Vec::new(),
                            DOMNamespace::kHTML,
                        );
                        continue;
                    }
                    if name == "td" || name == "th" {
                        self.ClearStackBackTo(&["table", "template", "html"]);
                        self.site_.InsertElement(
                            "tbody".to_owned(),
                            Vec::new(),
                            DOMNamespace::kHTML,
                        );
                        self.site_
                            .InsertElement("tr".to_owned(), Vec::new(), DOMNamespace::kHTML);
                        self.InsertCell(token);
                        return StartTagDisposition::kHandled;
                    }
                    if name == "table" {
                        self.site_.PopUntil("table");
                        continue;
                    }
                    return if self.CurrentCausesFosterParenting() {
                        StartTagDisposition::kFosterParent
                    } else {
                        StartTagDisposition::kProcessInBody
                    };
                }
            }
        }
    }

    // cpp: html/html_parser.cc:614-642
    fn ProcessEndTagInSelect(&mut self, name: &str) -> bool {
        if !self.InsideSelect() {
            return false;
        }
        if IsOneOf(
            name,
            &[
                "caption", "table", "tbody", "tfoot", "thead", "tr", "td", "th",
            ],
        ) {
            self.site_.PopUntil("select");
            return false;
        }
        if name == "option" {
            if self.site_.CurrentElement().is_some_and(|current| {
                self.site_
                    .OwnerDocument()
                    .Node(current)
                    .IsHTMLElement("option")
            }) {
                self.site_.Pop();
            }
            return true;
        }
        if name == "optgroup" {
            if let Some(current) = self.site_.CurrentElement() {
                if self
                    .site_
                    .OwnerDocument()
                    .Node(current)
                    .IsHTMLElement("option")
                {
                    if self.site_.OpenElementBefore(current).is_some_and(|before| {
                        self.site_
                            .OwnerDocument()
                            .Node(before)
                            .IsHTMLElement("optgroup")
                    }) {
                        self.site_.Pop();
                    }
                }
            }
            if self.site_.CurrentElement().is_some_and(|current| {
                self.site_
                    .OwnerDocument()
                    .Node(current)
                    .IsHTMLElement("optgroup")
            }) {
                self.site_.Pop();
            }
            return true;
        }
        if name == "select" {
            self.site_.PopUntil("select");
            return true;
        }
        if name == "template" {
            return false;
        }
        true
    }

    // cpp: html/html_parser.cc:644-727
    fn ProcessEndTagInTable(&mut self, name: &str) -> bool {
        loop {
            match self.CurrentTableMode() {
                TableMode::kNone => return false,
                TableMode::kCell => {
                    if name == "td" || name == "th" {
                        if self.HasInScope(name) {
                            self.CloseCell();
                        }
                        return true;
                    }
                    if IsOneOf(name, &["table", "tbody", "tfoot", "thead", "tr"]) {
                        self.CloseCell();
                        continue;
                    }
                    return false;
                }
                TableMode::kCaption => {
                    if name == "caption" {
                        self.site_.PopUntil("caption");
                        self.ClearActiveFormattingThroughLastMarker();
                        return true;
                    }
                    if name == "table" {
                        self.site_.PopUntil("caption");
                        self.ClearActiveFormattingThroughLastMarker();
                        continue;
                    }
                    if IsOneOf(
                        name,
                        &[
                            "body", "col", "colgroup", "html", "tbody", "td", "tfoot", "th",
                            "thead", "tr",
                        ],
                    ) {
                        return true;
                    }
                    return false;
                }
                TableMode::kRow => {
                    if name == "tr" {
                        self.site_.PopUntil("tr");
                        return true;
                    }
                    if IsOneOf(name, &["table", "tbody", "tfoot", "thead"]) {
                        self.site_.PopUntil("tr");
                        continue;
                    }
                    if name == "td" || name == "th" {
                        return true;
                    }
                    return false;
                }
                TableMode::kSection => {
                    let mut section = None;
                    for &entry in self.site_.OpenElements().iter().rev() {
                        let node = self.site_.OwnerDocument().Node(entry);
                        if node.IsHTMLElement("tbody")
                            || node.IsHTMLElement("thead")
                            || node.IsHTMLElement("tfoot")
                        {
                            section = Some(entry);
                            break;
                        }
                    }
                    if IsOneOf(name, &["tbody", "tfoot", "thead"]) {
                        if section.is_some_and(|entry| {
                            self.site_.OwnerDocument().Node(entry).Name() == name
                        }) {
                            self.site_.PopUntil(name);
                        }
                        return true;
                    }
                    if name == "table" {
                        if let Some(section) = section {
                            let section_name =
                                self.site_.OwnerDocument().Node(section).Name().to_owned();
                            self.site_.PopUntil(&section_name);
                        }
                        continue;
                    }
                    if IsOneOf(
                        name,
                        &[
                            "body", "caption", "col", "colgroup", "html", "td", "th", "tr",
                        ],
                    ) {
                        return true;
                    }
                    return false;
                }
                TableMode::kColumnGroup => {
                    if name == "colgroup" {
                        self.site_.PopUntil("colgroup");
                        return true;
                    }
                    if name == "col" {
                        return true;
                    }
                    self.site_.PopUntil("colgroup");
                    continue;
                }
                TableMode::kTable => {
                    if name == "table" {
                        self.site_.PopUntil("table");
                        return true;
                    }
                    if IsOneOf(
                        name,
                        &[
                            "body", "caption", "col", "colgroup", "html", "tbody", "td", "tfoot",
                            "th", "thead", "tr",
                        ],
                    ) {
                        return true;
                    }
                    return false;
                }
            }
        }
    }

    // cpp: html/html_parser.cc:729-742
    fn HasInScope(&self, name: &str) -> bool {
        for &entry in self.site_.OpenElements().iter().rev() {
            let node = self.site_.OwnerDocument().Node(entry);
            if node.Type() == dom::persistent_document::DOMNodeType::kElement {
                if node.Namespace() == DOMNamespace::kHTML {
                    if node.Name() == name {
                        return true;
                    }
                } else if LowerASCII(node.Name().to_owned()) == name {
                    return true;
                }
            }
            if node.IsHTMLElement("template") {
                return false;
            }
        }
        false
    }

    // cpp: html/html_parser.cc:744-756
    fn HasInListItemScope(&self, name: &str) -> bool {
        for &entry in self.site_.OpenElements().iter().rev() {
            let node = self.site_.OwnerDocument().Node(entry);
            if node.IsHTMLElement(name) {
                return true;
            }
            if node.IsHTMLElement("ol")
                || node.IsHTMLElement("ul")
                || node.IsHTMLElement("template")
                || node.IsHTMLElement("html")
            {
                return false;
            }
        }
        false
    }

    // cpp: html/html_parser.cc:758-764
    fn ClearActiveFormattingThroughLastMarker(&mut self) {
        while !self.active_formatting_elements_.is_empty() {
            let entry = self
                .active_formatting_elements_
                .pop()
                .expect("nonempty formatting list");
            if entry.is_none() {
                return;
            }
        }
    }

    // cpp: html/html_parser.cc:766-773
    fn HasTableInCurrentTemplateScope(&self) -> bool {
        for &entry in self.site_.OpenElements().iter().rev() {
            let node = self.site_.OwnerDocument().Node(entry);
            if node.IsHTMLElement("table") {
                return true;
            }
            if node.IsHTMLElement("template") {
                return false;
            }
        }
        false
    }

    // cpp: html/html_parser.cc:775-786
    fn InsertTemplate(&mut self, token: &HTMLToken) {
        if !self.InsideTemplate() && self.fragment_context_.is_none() {
            if !self.head_closed_ {
                self.EnsureHead();
            } else {
                self.EnsureBody();
            }
        }
        self.site_
            .InsertElementFromToken(token, DOMNamespace::kHTML);
        self.active_formatting_elements_.push(None);
    }

    // cpp: html/html_parser.cc:788-793
    fn EnsureHtml(&mut self) -> usize {
        if let Some(html) = self.html_ {
            return html;
        }
        let html = self
            .site_
            .InsertElement("html".to_owned(), Vec::new(), DOMNamespace::kHTML);
        self.html_ = Some(html);
        html
    }

    // cpp: html/html_parser.cc:795-803
    fn EnsureHead(&mut self) -> usize {
        self.EnsureHtml();
        if let Some(head) = self.head_ {
            return head;
        }
        while self.site_.CurrentElement() != self.html_ {
            self.site_.Pop();
        }
        let head = self
            .site_
            .InsertElement("head".to_owned(), Vec::new(), DOMNamespace::kHTML);
        self.head_ = Some(head);
        head
    }

    // cpp: html/html_parser.cc:805-820
    fn EnsureBody(&mut self) -> usize {
        self.EnsureHtml();
        if self.site_.HasInOpenElements("head") {
            self.site_.PopUntil("head");
        }
        self.head_closed_ = true;
        if self.body_.is_none() {
            while self.site_.CurrentElement() != self.html_ {
                self.site_.Pop();
            }
            self.body_ = Some(self.site_.InsertElement(
                "body".to_owned(),
                Vec::new(),
                DOMNamespace::kHTML,
            ));
        } else if !self.site_.HasInOpenElements("body") {
            while self.site_.CurrentElement() != self.html_ {
                self.site_.Pop();
            }
            self.site_
                .PushAlreadyParsed(self.body_.expect("body node exists"));
        }
        self.body_.expect("body node exists")
    }

    // cpp: html/html_parser.cc:822-830
    fn MergeAttributes(&mut self, element: usize, token: &HTMLToken) {
        for attribute in token.Attributes() {
            let name = attribute.GetName().Utf8();
            if self
                .site_
                .OwnerDocument()
                .Node(element)
                .FindAttribute(&name)
                .is_none()
            {
                self.site_.OwnerDocumentMut().SetAttribute(
                    element,
                    DOMAttribute {
                        local_name: name,
                        value: attribute.Value().Utf8(),
                        ..Default::default()
                    },
                );
            }
        }
    }

    // cpp: html/html_parser.cc:832-843
    fn ProcessDoctype(&mut self, token: &HTMLToken) {
        if self.fragment_context_.is_some() {
            return;
        }
        let root = self.site_.OwnerDocument().Root();
        if !self.site_.OwnerDocument().Node(root).Children().is_empty() {
            return;
        }
        let doctype = self.site_.OwnerDocumentMut().CreateDocumentType(
            BufferToUtf8(token.GetName()),
            IdentifierToUtf8(token.PublicIdentifier()),
            IdentifierToUtf8(token.SystemIdentifier()),
        );
        self.site_.OwnerDocumentMut().AppendChild(root, doctype);
        if token.ForceQuirks() || self.site_.OwnerDocument().Node(doctype).Name() != "html" {
            self.site_
                .OwnerDocumentMut()
                .SetCompatibilityMode(CompatibilityMode::kQuirks);
        }
    }

    // cpp: html/html_parser.cc:844-1006
    fn ProcessStartTag(&mut self, token: &HTMLToken) {
        let name = BufferToUtf8(token.GetName());
        let initial_current = self.site_.CurrentElement();
        let process_template_as_html = initial_current.is_none_or(|current| {
            let node = self.site_.OwnerDocument().Node(current);
            node.Namespace() == DOMNamespace::kHTML || ProcessesStartTagAsHTML(node, &name)
        });
        if name == "template" && process_template_as_html {
            self.InsertTemplate(token);
            return;
        }
        let in_template = self.InsideTemplate();
        if in_template && name == "html" {
            self.MergeAttributes(self.html_.expect("template requires html element"), token);
            return;
        }
        if in_template && IsOneOf(&name, &["head", "body", "frameset"]) {
            return;
        }
        if in_template
            && IsOneOf(
                &name,
                &[
                    "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style",
                    "title",
                ],
            )
        {
            self.site_
                .InsertElementFromToken(token, DOMNamespace::kHTML);
            if token.SelfClosing() || IsVoidElement(&name) {
                self.site_.Pop();
            }
            return;
        }
        if self.fragment_context_.is_some() && IsOneOf(&name, &["html", "head", "body"]) {
            return;
        }
        if self.fragment_context_.is_none() && name == "html" {
            if self.html_.is_none() {
                self.html_ = Some(
                    self.site_
                        .InsertElementFromToken(token, DOMNamespace::kHTML),
                );
            } else {
                self.MergeAttributes(self.html_.expect("html element exists"), token);
            }
            return;
        }
        if self.fragment_context_.is_none() && name == "head" {
            self.EnsureHtml();
            if self.head_.is_none() {
                while self.site_.CurrentElement() != self.html_ {
                    self.site_.Pop();
                }
                self.head_ = Some(
                    self.site_
                        .InsertElementFromToken(token, DOMNamespace::kHTML),
                );
            }
            return;
        }
        if self.fragment_context_.is_none()
            && !self.head_closed_
            && IsOneOf(
                &name,
                &[
                    "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style",
                    "title",
                ],
            )
        {
            self.EnsureHead();
            self.site_
                .InsertElementFromToken(token, DOMNamespace::kHTML);
            if token.SelfClosing() || IsVoidElement(&name) {
                self.site_.Pop();
            }
            return;
        }
        if self.fragment_context_.is_none() && name == "body" {
            self.EnsureHtml();
            if self.body_.is_none() {
                if self.site_.HasInOpenElements("head") {
                    self.site_.PopUntil("head");
                }
                while self.site_.CurrentElement() != self.html_ {
                    self.site_.Pop();
                }
                self.body_ = Some(
                    self.site_
                        .InsertElementFromToken(token, DOMNamespace::kHTML),
                );
            } else {
                self.MergeAttributes(self.body_.expect("body element exists"), token);
            }
            self.head_closed_ = true;
            return;
        }
        if self.fragment_context_.is_none() && name == "frameset" && self.body_.is_none() {
            self.EnsureHtml();
            if self.site_.HasInOpenElements("head") {
                self.site_.PopUntil("head");
            }
            while self.site_.CurrentElement() != self.html_ {
                self.site_.Pop();
            }
            self.head_closed_ = true;
            self.site_
                .InsertElementFromToken(token, DOMNamespace::kHTML);
            self.frameset_document_ = true;
            return;
        }

        if !in_template
            && self.fragment_context_.is_none()
            && !self.site_.HasInOpenElements("frameset")
        {
            self.EnsureBody();
        }
        let current = self.site_.CurrentElement();
        if current.is_some_and(|index| {
            let node = self.site_.OwnerDocument().Node(index);
            node.Namespace() != DOMNamespace::kHTML && !ProcessesStartTagAsHTML(node, &name)
        }) {
            if BreaksOutOfForeignContent(token, &name) {
                while let Some(index) = self.site_.CurrentElement() {
                    let node = self.site_.OwnerDocument().Node(index);
                    if node.Namespace() == DOMNamespace::kHTML
                        || IsHTMLIntegrationPoint(node)
                        || IsMathMLTextIntegrationPoint(node)
                    {
                        break;
                    }
                    self.site_.Pop();
                }
            } else {
                self.last_start_tag_was_html_ = false;
                let namespace = self
                    .site_
                    .OwnerDocument()
                    .Node(current.expect("current node"))
                    .Namespace();
                self.site_.InsertElementFromToken(token, namespace);
                if token.SelfClosing() || IsVoidElement(&name) {
                    self.site_.Pop();
                }
                return;
            }
        }
        let select_disposition = self.ProcessStartTagInSelect(token, &name);
        if select_disposition == StartTagDisposition::kHandled {
            return;
        }
        let table_disposition = self.ProcessStartTagInTable(token, &name);
        if table_disposition == StartTagDisposition::kHandled {
            return;
        }
        let foster_parent = table_disposition == StartTagDisposition::kFosterParent;
        if name == "svg" || name == "math" {
            if foster_parent {
                self.ReconstructActiveFormattingElements(true);
            }
            let foster_insert = foster_parent && self.CurrentCausesFosterParenting();
            self.last_start_tag_was_html_ = false;
            let inserted = self.site_.InsertElementFromToken(
                token,
                if name == "svg" {
                    DOMNamespace::kSVG
                } else {
                    DOMNamespace::kMathML
                },
            );
            if foster_insert {
                self.site_.InsertAtFosterParent(inserted);
            }
            if token.SelfClosing() {
                self.site_.Pop();
            }
            return;
        }
        if name == "li" && self.HasInListItemScope("li") {
            self.site_.PopUntil("li");
        }
        if (name == "dt" || name == "dd") && (self.HasInScope("dt") || self.HasInScope("dd")) {
            self.site_
                .PopUntil(if self.HasInScope("dt") { "dt" } else { "dd" });
        }
        if name == "option" && self.HasInScope("option") {
            self.site_.PopUntil("option");
        }
        if ClosesParagraph(&name) && self.HasInScope("p") {
            self.site_.PopUntil("p");
        }
        if IsOneOf(&name, &["h1", "h2", "h3", "h4", "h5", "h6"]) {
            for heading in ["h1", "h2", "h3", "h4", "h5", "h6"] {
                if self.HasInScope(heading) {
                    self.site_.PopUntil(heading);
                    break;
                }
            }
        }
        let template_without_table = in_template && !self.HasTableInCurrentTemplateScope();
        if template_without_table && name == "col" {
            self.site_
                .InsertElement("colgroup".to_owned(), Vec::new(), DOMNamespace::kHTML);
        }
        if template_without_table && IsOneOf(&name, &["tr", "td", "th"]) {
            self.site_
                .InsertElement("tbody".to_owned(), Vec::new(), DOMNamespace::kHTML);
        }
        if template_without_table && IsOneOf(&name, &["td", "th"]) {
            self.site_
                .InsertElement("tr".to_owned(), Vec::new(), DOMNamespace::kHTML);
        }
        if name == "tr"
            && self
                .site_
                .CurrentElement()
                .is_some_and(|node| self.site_.OwnerDocument().Node(node).IsHTMLElement("table"))
        {
            self.site_
                .InsertElement("tbody".to_owned(), Vec::new(), DOMNamespace::kHTML);
        }
        if (name == "td" || name == "th")
            && self.site_.CurrentElement().is_some_and(|node| {
                IsOneOf(
                    self.site_.OwnerDocument().Node(node).Name(),
                    &["table", "tbody", "thead", "tfoot"],
                )
            })
        {
            if self
                .site_
                .CurrentElement()
                .is_some_and(|node| self.site_.OwnerDocument().Node(node).IsHTMLElement("table"))
            {
                self.site_
                    .InsertElement("tbody".to_owned(), Vec::new(), DOMNamespace::kHTML);
            }
            self.site_
                .InsertElement("tr".to_owned(), Vec::new(), DOMNamespace::kHTML);
        }

        if !StartsBlockElement(&name) {
            self.ReconstructActiveFormattingElements(foster_parent);
        }
        let foster_insert = foster_parent && self.CurrentCausesFosterParenting();
        let inserted = self
            .site_
            .InsertElementFromToken(token, DOMNamespace::kHTML);
        if foster_insert {
            self.site_.InsertAtFosterParent(inserted);
        }
        if IsFormattingElement(&name) {
            self.AddActiveFormattingElement(inserted);
        }
        if token.SelfClosing() || IsVoidElement(&name) {
            self.site_.Pop();
        }
    }

    // cpp: html/html_parser.cc:1008-1060
    fn ProcessEndTag(&mut self, token: &HTMLToken) {
        let name = BufferToUtf8(token.GetName());
        let initial_current = self.site_.CurrentElement();
        if name == "template"
            && initial_current.is_none_or(|current| {
                self.site_.OwnerDocument().Node(current).Namespace() == DOMNamespace::kHTML
            })
        {
            if !self.InsideTemplate() {
                return;
            }
            self.site_.PopUntil("template");
            self.ClearActiveFormattingThroughLastMarker();
            return;
        }
        if self.InsideTemplate() && IsOneOf(&name, &["html", "head", "body"]) {
            return;
        }
        if self.site_.CurrentElement().is_some_and(|current| {
            self.site_.OwnerDocument().Node(current).Namespace() != DOMNamespace::kHTML
        }) {
            if name == "br" || name == "p" {
                while let Some(current) = self.site_.CurrentElement() {
                    let node = self.site_.OwnerDocument().Node(current);
                    if node.Namespace() == DOMNamespace::kHTML
                        || IsHTMLIntegrationPoint(node)
                        || IsMathMLTextIntegrationPoint(node)
                    {
                        break;
                    }
                    self.site_.Pop();
                }
            } else {
                let open = self.site_.OpenElements().to_vec();
                for entry in open.into_iter().rev() {
                    let node = self.site_.OwnerDocument().Node(entry);
                    if node.Namespace() == DOMNamespace::kHTML {
                        break;
                    }
                    if LowerASCII(node.Name().to_owned()) == name {
                        self.site_.PopUntil(&name);
                        return;
                    }
                }
            }
        }
        if self.ProcessEndTagInSelect(&name) {
            return;
        }
        if self.ProcessEndTagInTable(&name) {
            return;
        }
        if name == "html" || name == "body" {
            return;
        }
        if IsFormattingElement(&name) {
            self.CallTheAdoptionAgency(&name);
            return;
        }
        if name == "head" {
            self.head_closed_ = true;
        }
        if self.HasInScope(&name) {
            self.site_.PopUntil(&name);
        }
    }

    // cpp: html/html_parser.cc:1062-1105
    fn ProcessCharacters(&mut self, data: &str) {
        if self.fragment_context_.is_none()
            && self.site_.CurrentElement().is_none()
            && IsHTMLWhitespace(data)
        {
            return;
        }
        if self.fragment_context_.is_none() && self.html_.is_none() && IsHTMLWhitespace(data) {
            return;
        }
        if self.fragment_context_.is_none()
            && !self.head_closed_
            && self.head_.is_none()
            && IsHTMLWhitespace(data)
        {
            return;
        }
        if self.site_.CurrentElement().is_some_and(|current| {
            IsOneOf(
                self.site_.OwnerDocument().Node(current).Name(),
                &["title", "style", "script", "noframes"],
            )
        }) {
            self.site_.InsertText(data);
            return;
        }
        if self.frameset_document_ && IsHTMLWhitespace(data) {
            return;
        }
        if self.fragment_context_.is_none()
            && self.body_.is_none()
            && self.site_.CurrentElement() == self.html_
            && IsHTMLWhitespace(data)
        {
            self.site_.InsertText(data);
            return;
        }
        if !self.InsideTemplate() && self.fragment_context_.is_none() {
            self.EnsureBody();
        }
        let current = self.site_.CurrentElement();
        if current.is_some_and(|current| {
            let node = self.site_.OwnerDocument().Node(current);
            node.Namespace() != DOMNamespace::kHTML
                && !IsHTMLIntegrationPoint(node)
                && !IsMathMLTextIntegrationPoint(node)
        }) {
            self.site_.InsertText(data);
            return;
        }
        let in_table_mode = current.is_some_and(|current| {
            IsOneOf(
                self.site_.OwnerDocument().Node(current).Name(),
                &["table", "tbody", "tfoot", "thead", "tr"],
            )
        });
        if in_table_mode && !IsHTMLWhitespace(data) {
            if self.ReconstructActiveFormattingElements(true) {
                self.site_.InsertText(data);
            } else {
                self.site_.InsertTextAtFosterParent(data);
            }
        } else {
            self.ReconstructActiveFormattingElements(false);
            self.site_.InsertText(data);
        }
    }
}

// cpp: html/html_parser.cc:1121-1124
struct InertHTMLParserHost;

impl HTMLParserHost for InertHTMLParserHost {
    // The C++ inert host intentionally ignores parser-element events.
    fn HandleParserElement(&mut self, _event: ParserElementEvent<'_>) {}
}

/// Resident incremental parser state. Tokenizer, input insertion point and
/// tree-builder stacks survive each Pump; the DOM and host are borrowed only
/// during that call. Append accepts decoded text, FinishInput marks network
/// EOF, and Pump reports input/script waits without blocking the event loop.
/// Rendering readiness and script/resource scheduling belong to the caller.
pub struct HTMLDocumentParserState {
    decoder_: Option<crate::text_decoder::TextDecoder>,
    storage_: Option<HTMLDocumentParserContinuation>,
}
struct HTMLDocumentParserContinuation {
    document: DOMOwnerHandle,
    tree: DocumentTreeBuilderState,
    input: SegmentedString,
    tokenizer: Pin<Box<HTMLTokenizer>>,
    input_finished: bool,
    waiting_for_script: bool,
    finished: bool,
    current_script: Option<ParserScript>,
}
struct DocumentTreeBuilderState {
    open_elements: Vec<usize>,
    html_: Option<usize>,
    head_: Option<usize>,
    body_: Option<usize>,
    active_formatting_elements_: Vec<Option<usize>>,
    head_closed_: bool,
    frameset_document_: bool,
    last_start_tag_was_html_: bool,
    current_script_start_: Option<HTMLSourcePosition>,
    pending_script_: Option<ParserScript>,
    fragment_context_: Option<usize>,
}
/// Scoped adapter for existing script tasks. Drop restores the continuation
/// even when a host callback unwinds, without extending any Rust lifetime.
struct ParserBorrow<'state, 'document> {
    state: &'state mut HTMLDocumentParserState,
    parser: Option<HTMLDocumentParser<'document>>,
}
impl Drop for ParserBorrow<'_, '_> {
    fn drop(&mut self) {
        self.state.storage_ = self
            .parser
            .take()
            .expect("borrowed parser exists")
            .Suspend()
            .storage_;
    }
}

impl HTMLDocumentParserState {
    /// Create a resident parser for this document. Append receives decoded text;
    /// AppendBytes uses the incremental decoder at the parser input boundary.
    pub fn new(document: &mut PersistentDocument, host: &mut dyn HTMLParserHost) -> Self {
        HTMLDocumentParser::new(document, host).Suspend()
    }
    pub fn new_fragment(
        document: &mut PersistentDocument,
        host: &mut dyn HTMLParserHost,
        context: usize,
    ) -> Self {
        HTMLDocumentParser::new_fragment(document, host, context).Suspend()
    }
    pub fn SetTextEncoding(&mut self, label: &str) -> std::io::Result<()> {
        if self.decoder_.is_some() {
            return Err(std::io::Error::other(
                "parser text encoding already initialized",
            ));
        }
        self.decoder_ = Some(crate::text_decoder::TextDecoder::new(label)?);
        Ok(())
    }
    pub fn AppendBytes(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        if self.decoder_.is_none() {
            self.SetTextEncoding("utf-8")?;
        }
        let text = self.decoder_.as_mut().unwrap().append(bytes, false)?;
        self.Append(&text);
        Ok(())
    }
    pub fn FinishBytes(&mut self) -> std::io::Result<()> {
        if let Some(decoder) = &mut self.decoder_ {
            let tail = decoder.append(&[], true)?;
            self.Append(&tail);
        }
        self.FinishInput();
        Ok(())
    }
    pub fn Append(&mut self, text: &str) {
        let storage = self.storage_.as_mut().expect("parser continuation exists");
        AppendParserInput(&mut storage.input, storage.input_finished, text.as_bytes());
    }
    pub fn FinishInput(&mut self) {
        let storage = self.storage_.as_mut().expect("parser continuation exists");
        FinishParserInput(&mut storage.input, &mut storage.input_finished);
    }
    /// Pump one bounded turn, releasing the DOM borrow before returning.
    pub fn Pump(
        &mut self,
        document: &mut PersistentDocument,
        host: &mut dyn HTMLParserHost,
        token_budget: usize,
    ) -> HTMLParserResult {
        self.WithParser(document, host, |parser| parser.Pump(token_budget))
    }
    /// Temporarily attach existing parser/script adapters. The same DOM must
    /// be supplied each time; the continuation is restored on normal return
    /// and unwinding. No frame, resource wait or lifecycle event is implied.
    pub fn WithParser<R>(
        &mut self,
        document: &mut PersistentDocument,
        host: &mut dyn HTMLParserHost,
        task: impl FnOnce(&mut HTMLDocumentParser<'_>) -> R,
    ) -> R {
        assert_eq!(
            self.storage_
                .as_ref()
                .expect("parser continuation exists")
                .document,
            document.RootHandle(),
            "cannot resume HTML parsing in a different document"
        );
        let continuation = HTMLDocumentParserState {
            decoder_: None,
            storage_: self.storage_.take(),
        };
        let parser = HTMLDocumentParser::Resume(document, host, continuation);
        let mut scope = ParserBorrow {
            state: self,
            parser: Some(parser),
        };
        task(scope.parser.as_mut().unwrap())
    }
    pub fn ResumeAfterScript(&mut self) {
        let storage = self.storage_.as_mut().expect("parser continuation exists");
        if !storage.waiting_for_script {
            logic_error("HTML parser is not waiting for a script");
        }
        storage.waiting_for_script = false;
        storage.current_script = None;
    }
    pub fn InsertFromScript(&mut self, text: &str) {
        let storage = self.storage_.as_mut().expect("parser continuation exists");
        InsertParserInput(&mut storage.input, storage.waiting_for_script, text);
    }
    pub fn IsPaused(&self) -> bool {
        self.storage_
            .as_ref()
            .expect("parser continuation exists")
            .waiting_for_script
    }
    pub fn IsFinished(&self) -> bool {
        self.storage_
            .as_ref()
            .expect("parser continuation exists")
            .finished
    }
}

fn AppendParserInput(input: &mut SegmentedString, finished: bool, html: &[u8]) {
    if finished {
        logic_error("cannot append HTML after FinishInput");
    }
    if !html.is_empty() {
        input.Append(&SegmentedString::from_string(&BlinkString::FromUtf8(html)));
    }
}
fn FinishParserInput(input: &mut SegmentedString, finished: &mut bool) {
    if *finished {
        logic_error("FinishInput may only be called once");
    }
    input.Append(&SegmentedString::from_string(&BlinkString::from_utf16(&[
        0,
    ])));
    input.Close();
    *finished = true;
}
fn InsertParserInput(input: &mut SegmentedString, waiting_for_script: bool, html: &str) {
    if !waiting_for_script {
        logic_error("script insertion requires a paused HTML parser");
    }
    if !html.is_empty() {
        let mut inserted = SegmentedString::from_string(&BlinkString::FromUtf8(html.as_bytes()));
        inserted.SetExcludeLineNumbers();
        input.Prepend(&inserted, PrependType::kNewInput);
    }
}

// cpp: html/html_parser.cc:1129-1141
struct HTMLDocumentParserStorage<'a> {
    tree_builder: DocumentTreeBuilder<'a>,
    input: SegmentedString,
    tokenizer: Pin<Box<HTMLTokenizer>>,
    input_finished: bool,
    waiting_for_script: bool,
    finished: bool,
    current_script: Option<ParserScript>,
}

impl<'a> HTMLDocumentParserStorage<'a> {
    // cpp: html/html_parser.cc:1130-1132
    fn new(
        document: &'a mut PersistentDocument,
        host: &'a mut dyn HTMLParserHost,
        fragment_context: Option<usize>,
    ) -> Self {
        Self {
            tree_builder: DocumentTreeBuilder::new(document, host, fragment_context),
            input: SegmentedString::default(),
            tokenizer: HTMLTokenizer::new(HTMLParserOptions::default()),
            input_finished: false,
            waiting_for_script: false,
            finished: false,
            current_script: None,
        }
    }
}

// cpp: html/html_parser.h:48-70
// cpp: html/html_parser.cc:1143-1158
pub struct HTMLDocumentParser<'a> {
    storage_: Box<HTMLDocumentParserStorage<'a>>,
}

impl<'a> HTMLDocumentParser<'a> {
    /// Release the DOM and host borrows while retaining the exact parser state.
    fn Suspend(self) -> HTMLDocumentParserState {
        let document = self
            .storage_
            .tree_builder
            .site_
            .OwnerDocument()
            .RootHandle();
        let HTMLDocumentParserStorage {
            tree_builder,
            input,
            tokenizer,
            input_finished,
            waiting_for_script,
            finished,
            current_script,
        } = *self.storage_;
        let DocumentTreeBuilder {
            site_,
            html_,
            head_,
            body_,
            active_formatting_elements_,
            head_closed_,
            frameset_document_,
            last_start_tag_was_html_,
            current_script_start_,
            pending_script_,
            fragment_context_,
        } = tree_builder;
        HTMLDocumentParserState {
            decoder_: None,
            storage_: Some(HTMLDocumentParserContinuation {
                document,
                tree: DocumentTreeBuilderState {
                    open_elements: site_.IntoOpenElements(),
                    html_,
                    head_,
                    body_,
                    active_formatting_elements_,
                    head_closed_,
                    frameset_document_,
                    last_start_tag_was_html_,
                    current_script_start_,
                    pending_script_,
                    fragment_context_,
                },
                input,
                tokenizer,
                input_finished,
                waiting_for_script,
                finished,
                current_script,
            }),
        }
    }
    /// Reattach exclusively to the same document; parser node indices must
    /// never be interpreted in a different arena.
    fn Resume(
        document: &'a mut PersistentDocument,
        host: &'a mut dyn HTMLParserHost,
        mut state: HTMLDocumentParserState,
    ) -> Self {
        let state = state.storage_.take().expect("parser continuation exists");
        assert_eq!(
            state.document,
            document.RootHandle(),
            "cannot resume HTML parsing in a different document"
        );
        let HTMLDocumentParserContinuation {
            document: _,
            tree,
            input,
            tokenizer,
            input_finished,
            waiting_for_script,
            finished,
            current_script,
        } = state;
        let DocumentTreeBuilderState {
            open_elements,
            html_,
            head_,
            body_,
            active_formatting_elements_,
            head_closed_,
            frameset_document_,
            last_start_tag_was_html_,
            current_script_start_,
            pending_script_,
            fragment_context_,
        } = tree;
        Self {
            storage_: Box::new(HTMLDocumentParserStorage {
                tree_builder: DocumentTreeBuilder {
                    site_: HTMLConstructionSite::Resume(document, host, open_elements),
                    html_,
                    head_,
                    body_,
                    active_formatting_elements_,
                    head_closed_,
                    frameset_document_,
                    last_start_tag_was_html_,
                    current_script_start_,
                    pending_script_,
                    fragment_context_,
                },
                input,
                tokenizer,
                input_finished,
                waiting_for_script,
                finished,
                current_script,
            }),
        }
    }

    // cpp: html/html_parser.cc:1143-1148
    pub fn new(document: &'a mut PersistentDocument, host: &'a mut dyn HTMLParserHost) -> Self {
        foundation::InitStringStatics();
        Self {
            storage_: Box::new(HTMLDocumentParserStorage::new(document, host, None)),
        }
    }

    // cpp: html/html_parser.cc:1150-1155
    pub fn new_fragment(
        document: &'a mut PersistentDocument,
        host: &'a mut dyn HTMLParserHost,
        fragment_context: usize,
    ) -> Self {
        foundation::InitStringStatics();
        Self {
            storage_: Box::new(HTMLDocumentParserStorage::new(
                document,
                host,
                Some(fragment_context),
            )),
        }
    }

    // cpp: html/html_parser.h:55-56
    // cpp: html/html_parser.cc:1160-1167
    pub fn Append(&mut self, html: &str) {
        self.AppendBytes(html.as_bytes());
    }

    // std::string_view in the source can contain malformed UTF-8; BlinkString
    // performs the conversion from bytes after the parser receives the input.
    pub fn AppendBytes(&mut self, html: &[u8]) {
        AppendParserInput(&mut self.storage_.input, self.storage_.input_finished, html);
    }

    // cpp: html/html_parser.h:56
    // cpp: html/html_parser.cc:1169-1177
    pub fn FinishInput(&mut self) {
        FinishParserInput(&mut self.storage_.input, &mut self.storage_.input_finished);
    }

    // cpp: html/html_parser.h:57
    // cpp: html/html_parser.cc:1179-1223
    // CurrentLine/CurrentColumn retain their source calls; those
    // SegmentedString methods await foundation::OrdinalNumber.
    pub fn Pump(&mut self, token_budget: usize) -> HTMLParserResult {
        let storage = &mut self.storage_;
        if storage.finished {
            return HTMLParserResult {
                status: HTMLParserStatus::kFinished,
                script: None,
            };
        }
        if storage.waiting_for_script {
            return HTMLParserResult {
                status: HTMLParserStatus::kWaitingForScript,
                script: storage.current_script,
            };
        }
        if token_budget == 0 {
            invalid_argument("HTML parser token budget must be positive");
        }
        if storage.input.IsEmpty() && !storage.input_finished {
            return HTMLParserResult {
                status: HTMLParserStatus::kNeedMoreInput,
                script: None,
            };
        }

        let mut processed = 0;
        while processed < token_budget {
            let token_position = HTMLSourcePosition {
                line: storage.input.CurrentLine().OneBasedInt() as u32,
                column: storage.input.CurrentColumn().OneBasedInt() as u32,
            };
            let Some(token) = storage.tokenizer.as_mut().NextToken(&mut storage.input) else {
                return HTMLParserResult {
                    status: HTMLParserStatus::kNeedMoreInput,
                    script: None,
                };
            };

            let eof = token.GetType() == TokenType::kEndOfFile;
            storage.tree_builder.Process(token, token_position);
            let speculative_tag = if token.GetType() == TokenType::kStartTag
                && storage.tree_builder.LastStartTagWasHTML()
                && !token.GetName().IsEmpty()
            {
                Some(LookupHtmlTag(token.GetName().as_slice()))
            } else {
                None
            };
            if let Some(tag) = speculative_tag {
                storage.tokenizer.as_mut().UpdateStateForTag(tag);
            }
            storage.tokenizer.as_mut().ClearToken();
            processed += 1;

            if let Some(script) = storage.tree_builder.TakePendingScript() {
                storage.waiting_for_script = true;
                storage.current_script = Some(script);
                return HTMLParserResult {
                    status: HTMLParserStatus::kWaitingForScript,
                    script: Some(script),
                };
            }
            if eof {
                storage.tree_builder.Finish();
                storage.finished = true;
                return HTMLParserResult {
                    status: HTMLParserStatus::kFinished,
                    script: None,
                };
            }
        }
        HTMLParserResult {
            status: HTMLParserStatus::kYielded,
            script: None,
        }
    }

    // cpp: html/html_parser.h:58
    // cpp: html/html_parser.cc:1225-1230
    pub fn ResumeAfterScript(&mut self) {
        if !self.storage_.waiting_for_script {
            logic_error("HTML parser is not waiting for a script");
        }
        self.storage_.waiting_for_script = false;
        self.storage_.current_script = None;
    }

    // Borrow the paused parser's persistent DOM for a synchronous script task.
    // The caller must return the same Document arena before calling Pump again.
    // Rust's exclusive reborrow enforces that no parser method can run inside
    // this callback, matching the source parser's script pause boundary.
    pub fn WithPausedDocument<R>(&mut self, task: impl FnOnce(&mut PersistentDocument) -> R) -> R {
        if !self.storage_.waiting_for_script {
            logic_error("HTML parser is not waiting for a script");
        }
        self.WithDocument(task)
    }

    // A Page may run an async task between Pump calls, including while the
    // parser needs more input. An exclusive reborrow permits that source access
    // without retaining aliases to the arena during tokenizer/tree work.
    pub fn WithDocument<R>(&mut self, task: impl FnOnce(&mut PersistentDocument) -> R) -> R {
        task(self.storage_.tree_builder.site_.OwnerDocumentMut())
    }

    // cpp: html/html_parser.h:59
    // cpp: html/html_parser.cc:1232-1241
    pub fn InsertFromScript(&mut self, html: &str) {
        InsertParserInput(
            &mut self.storage_.input,
            self.storage_.waiting_for_script,
            html,
        );
    }

    // cpp: html/html_parser.h:61
    // cpp: html/html_parser.cc:1243-1245
    pub fn IsPaused(&self) -> bool {
        self.storage_.waiting_for_script
    }

    // cpp: html/html_parser.h:62
    // cpp: html/html_parser.cc:1247-1249
    pub fn IsFinished(&self) -> bool {
        self.storage_.finished
    }
}

// cpp: html/html_parser.h:75-77
// cpp: html/html_parser.cc:1251-1263
pub fn ParseHTML(html: &str) -> DOM {
    ParseHTMLBytes(html.as_bytes())
}

// cpp: html/html_parser.h:75-77
// cpp: html/html_parser.cc:1251-1263
pub fn ParseHTMLBytes(html: &[u8]) -> DOM {
    let mut owner = DOM::new();
    let mut host = InertHTMLParserHost;
    {
        let mut parser = HTMLDocumentParser::new(owner.GetDocumentMut(), &mut host);
        parser.AppendBytes(html);
        parser.FinishInput();
        while !parser.IsFinished() {
            let result = parser.Pump(usize::MAX);
            if result.status == HTMLParserStatus::kWaitingForScript {
                parser.ResumeAfterScript();
            }
        }
    }
    owner
}

// cpp: html/html_parser.h:79-82
// cpp: html/html_parser.cc:1265-1285
// DOMNode& identity maps to an arena index. Its owner handle retains the
// document-arena identity even when the context belongs to another arena.
pub fn ParseHTMLFragment(
    document: &mut PersistentDocument,
    context: &DOMNode,
    html: &str,
) -> usize {
    let fragment = document.CreateDocumentFragment();
    let synthetic_context = document.CreateElement(
        context.Namespace(),
        context.Name().to_owned(),
        context.Attributes().to_vec(),
    );
    if let Some(owner) = context.OwnerDocumentNode() {
        document.AdoptNodeByHandle(fragment, owner);
        document.AdoptNodeByHandle(synthetic_context, owner);
    }
    let mut host = InertHTMLParserHost;
    {
        let mut parser = HTMLDocumentParser::new_fragment(document, &mut host, synthetic_context);
        parser.Append(html);
        parser.FinishInput();
        while !parser.IsFinished() {
            let result = parser.Pump(usize::MAX);
            if result.status == HTMLParserStatus::kWaitingForScript {
                parser.ResumeAfterScript();
            }
        }
    }
    let contents = document.TemplateContents(synthetic_context);
    document.TakeAllChildren(contents.unwrap_or(synthetic_context), fragment);
    fragment
}
