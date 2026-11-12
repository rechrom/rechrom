// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! styled_markup_accumulator.cc and html_interchange.cc, pinned 6c1d401fcca5.
//! Accumulates WTF-16 markup directly from borrowed canonical document nodes.
use super::{
    markup_accumulator::MarkupDocument,
    markup_formatter::{self as fmt, ResolveUrls, SerializationType},
};

#[derive(Clone, Copy, Debug)]
pub struct TextOffset<N> {
    pub node: N,
    pub offset: usize,
}
#[derive(Clone, Copy, Debug)]
pub struct CreateMarkupOptions {
    pub resolve_urls: ResolveUrls,
    pub serialization: SerializationType,
    pub annotate_for_interchange: bool,
    pub convert_blocks_to_inlines: bool,
    pub for_markup_sanitization: bool,
    pub skip_unselectable_content: bool,
    pub ignore_css_text_transforms: bool,
}
impl Default for CreateMarkupOptions {
    fn default() -> Self {
        Self {
            resolve_urls: ResolveUrls::None,
            serialization: SerializationType::Html,
            annotate_for_interchange: false,
            convert_blocks_to_inlines: false,
            for_markup_sanitization: false,
            skip_unselectable_content: false,
            ignore_css_text_transforms: false,
        }
    }
}
pub struct StyledMarkupAccumulator<'a, D: MarkupDocument> {
    document: &'a D,
    start: Option<TextOffset<D::Node>>,
    end: Option<TextOffset<D::Node>>,
    pub options: CreateMarkupOptions,
    result: Vec<u16>,
    reversed_preceding_markup: Vec<Vec<u16>>,
}
impl<'a, D: MarkupDocument> StyledMarkupAccumulator<'a, D>
where
    D::Node: PartialEq,
{
    pub fn new(
        document: &'a D,
        start: Option<TextOffset<D::Node>>,
        end: Option<TextOffset<D::Node>>,
        options: CreateMarkupOptions,
    ) -> Self {
        Self {
            document,
            start,
            end,
            options,
            result: Vec::new(),
            reversed_preceding_markup: Vec::new(),
        }
    }
    pub fn TextBounds(&self, node: D::Node) -> (usize, usize) {
        let length = self.document.Data(node).len();
        let start = self
            .start
            .filter(|p| p.node == node)
            .map_or(0, |p| p.offset.min(length));
        let end = self
            .end
            .filter(|p| p.node == node)
            .map_or(length, |p| p.offset.min(length));
        (start.min(end), end)
    }
    pub fn AppendText(&mut self, node: D::Node) {
        let (start, end) = self.TextBounds(node);
        let mask = if self.options.serialization == SerializationType::Xml {
            fmt::PCDATA
        } else if self.document.TextIsRaw(node) {
            0
        } else {
            fmt::HTML_PCDATA
        };
        fmt::AppendCharactersReplacingEntities(
            &mut self.result,
            &self.document.Data(node)[start..end],
            mask,
        );
    }
    /// `content` is the existing TextIterator's range output (or raw selected
    /// data for select descendants); no substitute whitespace renderer lives here.
    pub fn AppendTextWithInlineStyle(
        &mut self,
        node: D::Node,
        content: &[u16],
        inline_style: Option<&[u16]>,
        preserves_breaks: bool,
    ) {
        if let Some(style) = inline_style {
            fmt::Append(&mut self.result, "<span style=\"");
            fmt::AppendAttributeValue(&mut self.result, style, self.options.serialization);
            fmt::Append(&mut self.result, "\">");
        }
        if !self.options.annotate_for_interchange {
            self.AppendText(node);
        } else {
            let mut escaped = Vec::new();
            fmt::AppendCharactersReplacingEntities(&mut escaped, content, fmt::PCDATA);
            if self.options.for_markup_sanitization || preserves_breaks {
                self.result.extend(escaped);
            } else {
                self.result.extend(ConvertHtmlTextToInterchangeFormat(
                    &escaped,
                    self.start.is_some_and(|p| p.node == node),
                    self.end.is_some_and(|p| p.node == node),
                ));
            }
        }
        if inline_style.is_some() {
            fmt::Append(&mut self.result, "</span>");
        }
    }
    pub fn ElementMarkup(&self, node: D::Node, inline_style: Option<Option<&[u16]>>) -> Vec<u16> {
        let mut out = vec![60];
        out.extend(self.document.QualifiedName(node));
        for index in 0..self.document.AttributeCount(node) {
            let attribute = self.document.Attribute(node, index);
            if inline_style.is_some()
                && attribute.namespace.is_empty()
                && attribute
                    .local_name
                    .iter()
                    .copied()
                    .eq("style".encode_utf16())
            {
                continue;
            }
            let namespace = String::from_utf16_lossy(attribute.namespace);
            let prefix = match namespace.as_str() {
                _ if self.options.serialization == SerializationType::Xml
                    && !attribute.prefix.is_empty() =>
                {
                    None
                }
                "http://www.w3.org/XML/1998/namespace" => Some("xml"),
                "http://www.w3.org/1999/xlink" => Some("xlink"),
                "http://www.w3.org/2000/xmlns/"
                    if attribute.prefix.is_empty()
                        && !attribute
                            .local_name
                            .iter()
                            .copied()
                            .eq("xmlns".encode_utf16()) =>
                {
                    Some("xmlns")
                }
                _ => None,
            }
            .map(|p| p.encode_utf16().collect::<Vec<_>>());
            let value =
                self.document
                    .ResolvedAttributeValue(node, index, self.options.resolve_urls);
            fmt::AppendAttributeWithType(
                &mut out,
                prefix.as_deref().unwrap_or(attribute.prefix),
                attribute.local_name,
                value.as_deref().unwrap_or(attribute.value),
                self.options.serialization,
            );
        }
        if let Some(Some(style)) = inline_style.filter(|style| style.is_some_and(|s| !s.is_empty()))
        {
            fmt::AppendAttributeWithType(
                &mut out,
                &[],
                &[115, 116, 121, 108, 101],
                style,
                self.options.serialization,
            );
        }
        if super::markup_accumulator::ShouldSelfClose(
            self.document,
            node,
            self.options.serialization,
        ) {
            if self.document.IsHTMLElement(node) {
                out.push(32);
            }
            out.push(47);
        }
        out.push(62);
        out
    }
    pub fn AppendElement(&mut self, node: D::Node) {
        self.result.extend(self.ElementMarkup(node, None));
    }
    pub fn AppendElementWithInlineStyle(&mut self, node: D::Node, style: Option<&[u16]>) {
        self.result.extend(self.ElementMarkup(node, Some(style)));
    }
    pub fn AppendEndTag(&mut self, node: D::Node) {
        if super::markup_accumulator::ShouldSelfClose(
            self.document,
            node,
            self.options.serialization,
        ) || self.document.ChildCount(node) == 0 && self.document.IsHTMLVoidElement(node)
        {
            return;
        }
        fmt::Append(&mut self.result, "</");
        self.result.extend(self.document.QualifiedName(node));
        self.result.push(62);
    }
    pub fn AppendStartMarkup(&mut self, node: D::Node) {
        super::markup_accumulator::AppendNonElement(
            self.document,
            node,
            self.options.serialization,
            &mut self.result,
        );
    }

    pub fn AppendInterchangeNewline(&mut self) {
        fmt::Append(&mut self.result, "<br class=\"Apple-interchange-newline\">");
    }
    pub fn PushMarkup(&mut self, markup: Vec<u16>) {
        self.reversed_preceding_markup.push(markup);
    }
    pub fn WrapWithStyleNode(&mut self, style: &[u16]) {
        let mut open = Vec::new();
        fmt::Append(&mut open, "<div style=\"");
        fmt::AppendAttributeValue(&mut open, style, self.options.serialization);
        fmt::Append(&mut open, "\">");
        self.PushMarkup(open);
        fmt::Append(&mut self.result, "</div>");
    }
    pub fn TakeResults(self) -> Vec<u16> {
        self.reversed_preceding_markup
            .into_iter()
            .rev()
            .flatten()
            .chain(self.result)
            .filter(|unit| *unit != 0)
            .collect()
    }
}
/// html_interchange.cc:45–130 with NoNbspForInterElementSpaceOnCopy enabled.
pub fn ConvertHtmlTextToInterchangeFormat(
    input: &[u16],
    selection_start: bool,
    selection_end: bool,
) -> Vec<u16> {
    let mut out = Vec::new();
    let mut i = 0;
    let converted = |out: &mut Vec<u16>| {
        fmt::Append(out, "<span>\u{a0}</span>");
    };
    while i < input.len() {
        if !matches!(input[i], 9 | 10 | 12 | 13 | 32) {
            out.push(input[i]);
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < input.len() && matches!(input[j], 9 | 10 | 12 | 13 | 32) {
            j += 1;
        }
        let mut count = j - i;
        while count > 0 {
            match count % 3 {
                0 => {
                    converted(&mut out);
                    out.push(32);
                    converted(&mut out);
                    count -= 3;
                }
                1 => {
                    if i == 0 && selection_start || i + 1 == input.len() && selection_end {
                        converted(&mut out);
                    } else {
                        out.push(32);
                    }
                    count -= 1;
                }
                2 => {
                    converted(&mut out);
                    if i != 0 && i + 2 == input.len() {
                        converted(&mut out);
                    } else {
                        out.push(32);
                    }
                    count -= 2;
                }
                _ => unreachable!(),
            }
        }
        i = j;
    }
    out
}
