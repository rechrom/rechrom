use super::super::{
    markup_accumulator::MarkupAttribute,
    markup_formatter::{self as fmt, ResolveUrls, SerializationType},
    styled_markup_accumulator::ConvertHtmlTextToInterchangeFormat,
};
use super::*;
struct Node {
    tag: &'static str,
    text: Vec<u16>,
    parent: Option<usize>,
    children: Vec<usize>,
}
struct Document(Vec<Node>);
impl Document {
    fn InlineSiblings() -> Self {
        Self(vec![
            Node {
                tag: "div",
                text: vec![],
                parent: None,
                children: vec![1, 3],
            },
            Node {
                tag: "b",
                text: vec![],
                parent: Some(0),
                children: vec![2],
            },
            Node {
                tag: "",
                text: "hello".encode_utf16().collect(),
                parent: Some(1),
                children: vec![],
            },
            Node {
                tag: "i",
                text: vec![],
                parent: Some(0),
                children: vec![4],
            },
            Node {
                tag: "",
                text: "world".encode_utf16().collect(),
                parent: Some(3),
                children: vec![],
            },
        ])
    }
}
impl MarkupDocument for Document {
    type Node = usize;
    fn SerializationType(&self) -> SerializationType {
        SerializationType::Html
    }
    fn ElementNamespace(&self, _: usize) -> &[u16] {
        &[]
    }
    fn IsHTMLElement(&self, n: usize) -> bool {
        !self.0[n].tag.is_empty()
    }
    fn IsCDATASection(&self, _: usize) -> bool {
        false
    }
    fn DocumentTypeIdentifiers(&self, _: usize) -> (&[u16], &[u16]) {
        (&[], &[])
    }
    fn XMLDeclaration(&self) -> Option<(&[u16], &[u16], Option<bool>)> {
        None
    }
    fn Kind(&self, n: usize) -> MarkupNodeKind {
        if self.0[n].tag.is_empty() {
            MarkupNodeKind::Text
        } else {
            MarkupNodeKind::Element
        }
    }
    fn QualifiedName(&self, n: usize) -> Vec<u16> {
        self.0[n].tag.encode_utf16().collect()
    }
    fn Data(&self, n: usize) -> &[u16] {
        &self.0[n].text
    }
    fn ChildCount(&self, n: usize) -> usize {
        self.0[n].children.len()
    }
    fn ChildAt(&self, n: usize, i: usize) -> usize {
        self.0[n].children[i]
    }
    fn TemplateContents(&self, _: usize) -> Option<usize> {
        None
    }
    fn IsHTMLVoidElement(&self, _: usize) -> bool {
        false
    }
    fn TextIsRaw(&self, _: usize) -> bool {
        false
    }
    fn AttributeCount(&self, _: usize) -> usize {
        0
    }
    fn Attribute(&self, _: usize, _: usize) -> MarkupAttribute<'_> {
        unreachable!()
    }
    fn ResolvedAttributeValue(&self, _: usize, _: usize, _: ResolveUrls) -> Option<Vec<u16>> {
        None
    }
    fn ElementIsValue(&self, _: usize) -> Option<&[u16]> {
        None
    }
}
impl StyledMarkupDocument for Document {
    type Style = Vec<u16>;
    fn Parent(&self, n: usize, _: MarkupTree) -> Option<usize> {
        self.0[n].parent
    }
    fn FirstChild(&self, n: usize, _: MarkupTree) -> Option<usize> {
        self.0[n].children.first().copied()
    }
    fn NextSibling(&self, n: usize, _: MarkupTree) -> Option<usize> {
        let siblings = &self.0[self.0[n].parent?].children;
        siblings
            .get(siblings.iter().position(|v| *v == n)? + 1)
            .copied()
    }
    fn Facts(&self, n: usize) -> SerializationNodeFacts {
        SerializationNodeFacts {
            has_layout_object: true,
            is_html_element: !self.0[n].tag.is_empty(),
            selectable: true,
            ..Default::default()
        }
    }
    fn RenderedText(
        &self,
        n: usize,
        start: usize,
        end: usize,
        _: bool,
    ) -> Result<Vec<u16>, SerializationError> {
        Ok(self.Data(n)[start..end].to_vec())
    }
    fn WrappingStyle(&self, _: usize, _: bool) -> Result<Option<Self::Style>, SerializationError> {
        Ok(None)
    }
    fn InlineStyle(
        &self,
        _: usize,
        _: Option<&Self::Style>,
        _: bool,
        _: bool,
        _: bool,
        _: bool,
    ) -> Result<Self::Style, SerializationError> {
        Ok(vec![])
    }
    fn TextWrappingStyle(
        &self,
        s: &Self::Style,
        _: bool,
    ) -> Result<Self::Style, SerializationError> {
        Ok(s.clone())
    }
    fn FullySelectedRootStyle(&self, _: usize) -> Result<Self::Style, SerializationError> {
        Ok(vec![])
    }
    fn StyleText<'a>(&self, s: &'a Self::Style) -> &'a [u16] {
        s
    }
}
fn Range(
    first: usize,
    end_node: usize,
    end_offset: usize,
    past_end: Option<usize>,
) -> MarkupRange<usize> {
    MarkupRange {
        first_node: Some(first),
        past_end,
        start: Some(TextOffset {
            node: first,
            offset: 1,
        }),
        end: Some(TextOffset {
            node: end_node,
            offset: end_offset,
        }),
        highest_node_to_serialize: None,
        fully_selected_root: None,
        interchange_newline_at_start: false,
        interchange_newline_at_end: false,
        first_node_after_start_newline: None,
        only_interchange_newline: false,
    }
}
#[test]
fn PartialRangeRetainsNestedFormattingAndClosesOpenedAncestors() {
    let d = Document::InlineSiblings();
    let markup = CreateMarkup(
        &d,
        Range(2, 4, 3, None),
        MarkupTree::DOM,
        CreateMarkupOptions::default(),
    )
    .unwrap();
    assert_eq!(
        String::from_utf16(&markup).unwrap(),
        "<b>ello</b><i>wor</i>"
    );
    let markup = CreateMarkup(
        &d,
        Range(2, 2, 4, Some(3)),
        MarkupTree::DOM,
        CreateMarkupOptions::default(),
    )
    .unwrap();
    assert_eq!(String::from_utf16(&markup).unwrap(), "<b>ell</b>");
}
#[test]
fn InterchangePreservesEdgeSpacesButInteriorInlineSpaceCanBreak() {
    let space = [32];
    assert_eq!(
        ConvertHtmlTextToInterchangeFormat(&space, false, false),
        space
    );
    assert_eq!(
        String::from_utf16(&ConvertHtmlTextToInterchangeFormat(&space, true, false)).unwrap(),
        "<span>\u{a0}</span>"
    );
    assert_eq!(
        String::from_utf16(&ConvertHtmlTextToInterchangeFormat(
            &[32, 32, 32],
            false,
            false
        ))
        .unwrap(),
        "<span>\u{a0}</span> <span>\u{a0}</span>"
    );
}
#[test]
fn FormattingPreservesWtf16AndEscapesHtmlAndXmlAttributes() {
    let mut html = Vec::new();
    fmt::AppendAttributeValue(
        &mut html,
        &[60, 62, 38, 34, 160, 9, 10, 13, 0xd800],
        SerializationType::Html,
    );
    let mut expected = "&lt;&gt;&amp;&quot;&nbsp;\t\n\r"
        .encode_utf16()
        .collect::<Vec<_>>();
    expected.push(0xd800);
    assert_eq!(html, expected);
    let mut xml = Vec::new();
    fmt::AppendAttributeValue(
        &mut xml,
        &[60, 62, 38, 34, 160, 9, 10, 13],
        SerializationType::Xml,
    );
    assert_eq!(
        String::from_utf16(&xml).unwrap(),
        "&lt;&gt;&amp;&quot;\u{a0}&#9;&#10;&#13;"
    );
}
