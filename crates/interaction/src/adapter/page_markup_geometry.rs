//! Borrowed native ports for source StyledMarkupSerializer. The existing
//! Position/VisiblePosition, TextIterator and paragraph owners prepare the range.
use super::super::page_selection_markup::{MarkupStyleQueries, NativeMarkupDocument};
use super::*;
use cssom::editing_style::SerializationStyleOperation as StyleOp;
use interaction_default::core::editing::{
    iterators::{text_iterator, text_iterator_behavior::TextIteratorBehavior},
    serializers::{
        markup_accumulator::{MarkupAttribute, MarkupDocument, MarkupNodeKind},
        markup_formatter::ResolveUrls,
        serialization,
        styled_markup_accumulator::{CreateMarkupOptions, TextOffset},
        styled_markup_serializer::{
            self as styled, MarkupRange, MarkupTree, SerializationError, SerializationNodeFacts,
            StyledMarkupDocument,
        },
    },
};
struct NativeStyledMarkup<'a> {
    native: NativeMarkupDocument<'a>,
    context: EditingPositionContext<'a>,
    styles: &'a dyn MarkupStyleQueries,
}
impl MarkupDocument for NativeStyledMarkup<'_> {
    type Node = usize;
    fn SerializationType(
        &self,
    ) -> interaction_default::core::editing::serializers::markup_formatter::SerializationType {
        self.native.SerializationType()
    }
    fn ElementNamespace(&self, n: usize) -> &[u16] {
        self.native.ElementNamespace(n)
    }
    fn IsHTMLElement(&self, n: usize) -> bool {
        self.native.IsHTMLElement(n)
    }
    fn IsCDATASection(&self, n: usize) -> bool {
        self.native.IsCDATASection(n)
    }
    fn XMLDeclaration(&self) -> Option<(&[u16], &[u16], Option<bool>)> {
        self.native.XMLDeclaration()
    }
    fn DocumentTypeIdentifiers(&self, n: usize) -> (&[u16], &[u16]) {
        self.native.DocumentTypeIdentifiers(n)
    }
    fn Kind(&self, n: usize) -> MarkupNodeKind {
        self.native.Kind(n)
    }
    fn QualifiedName(&self, n: usize) -> Vec<u16> {
        self.native.QualifiedName(n)
    }
    fn Data(&self, n: usize) -> &[u16] {
        self.native.Data(n)
    }
    fn ChildCount(&self, n: usize) -> usize {
        self.native.ChildCount(n)
    }
    fn ChildAt(&self, n: usize, i: usize) -> usize {
        self.native.ChildAt(n, i)
    }
    fn TemplateContents(&self, n: usize) -> Option<usize> {
        self.native.TemplateContents(n)
    }
    fn IsHTMLVoidElement(&self, n: usize) -> bool {
        self.native.IsHTMLVoidElement(n)
    }
    fn TextIsRaw(&self, n: usize) -> bool {
        self.native.TextIsRaw(n)
    }
    fn AttributeCount(&self, n: usize) -> usize {
        self.native.AttributeCount(n)
    }
    fn Attribute(&self, n: usize, i: usize) -> MarkupAttribute<'_> {
        self.native.Attribute(n, i)
    }
    fn ResolvedAttributeValue(&self, n: usize, i: usize, r: ResolveUrls) -> Option<Vec<u16>> {
        self.native.ResolvedAttributeValue(n, i, r)
    }
    fn ElementIsValue(&self, n: usize) -> Option<&[u16]> {
        self.native.ElementIsValue(n)
    }
}
impl NativeStyledMarkup<'_> {
    fn Tree(&self, tree: MarkupTree) -> EditingPositionContext<'_> {
        if tree == MarkupTree::Flat {
            self.context.Flat()
        } else {
            self.context.Dom()
        }
    }
    fn Style(&self, op: StyleOp) -> Result<cssom::CSSInlineStyle, SerializationError> {
        self.styles.Query(op).map_err(|error| match error {
            cssom::editing_style::EditingStyleError::UnsupportedResource => {
                SerializationError::UnsupportedResource
            }
            _ => SerializationError::MissingStyle,
        })
    }
    fn Id(&self, n: usize) -> u64 {
        self.context.document.Node(n).Id()
    }
}
impl StyledMarkupDocument for NativeStyledMarkup<'_> {
    type Style = cssom::CSSInlineStyle;
    fn Parent(&self, n: usize, tree: MarkupTree) -> Option<usize> {
        self.context
            .Index(&PositionTree::Parent(&self.Tree(tree), &self.Id(n))?)
    }
    fn FirstChild(&self, n: usize, tree: MarkupTree) -> Option<usize> {
        self.context
            .Index(&PositionTree::ChildAt(&self.Tree(tree), &self.Id(n), 0)?)
    }
    fn NextSibling(&self, n: usize, tree: MarkupTree) -> Option<usize> {
        self.context
            .Index(&PositionTree::NextSibling(&self.Tree(tree), &self.Id(n))?)
    }
    fn Facts(&self, n: usize) -> SerializationNodeFacts {
        let node = self.context.document.Node(n);
        let layout = self.context.Layout(&node.Id());
        let facts = layout.as_ref().map(|l| l.EditingParagraphLayoutFacts());
        let style = self.context.document.ResolvedStyleFor(n);
        SerializationNodeFacts {
            has_layout_object: layout.is_some(),
            is_html_element: node.Type() == DOMNodeType::kElement
                && node.Namespace() == DOMNamespace::kHTML,
            enclosing_block: facts.as_ref().is_some_and(|f| f.is_enclosing_block),
            can_have_children_for_editing: node.Type() != DOMNodeType::kText
                && self.context.document.CanContainRangeEndPoint(n),
            selectable: facts.as_ref().is_none_or(|f| f.selectable),
            display_contents: style.is_some_and(|s| s.own_display_contents),
            preserve_breaks: style
                .and_then(|s| s.ComputedStyle())
                .is_some_and(|s| s.ShouldPreserveBreaks()),
            user_agent_shadow_host: self.context.document.ShadowRoot(n).is_some_and(|r| {
                self.context.document.ShadowRootMode(r)
                    == Some(dom::persistent_document::DOMShadowRootMode::kUserAgent)
            }),
        }
    }
    fn RenderedText(
        &self,
        n: usize,
        start: usize,
        end: usize,
        ignore: bool,
    ) -> Result<Vec<u16>, SerializationError> {
        let context = self.context;
        let mut behavior = TextIteratorBehavior::default();
        behavior.SetIgnoresCssTextTransforms(ignore);
        Ok(text_iterator::PlainText(
            &context,
            &Position::new(self.Id(n), start),
            &Position::new(self.Id(n), end),
            behavior,
        ))
    }
    fn WrappingStyle(
        &self,
        n: usize,
        annotate: bool,
    ) -> Result<Option<Self::Style>, SerializationError> {
        self.Style(StyleOp::Wrapping {
            node_id: self.Id(n),
            annotate,
        })
        .map(Some)
    }
    fn InlineStyle(
        &self,
        n: usize,
        wrapping: Option<&Self::Style>,
        annotate: bool,
        sanitize: bool,
        force_inline: bool,
        remove_float: bool,
    ) -> Result<Self::Style, SerializationError> {
        self.Style(StyleOp::Inline {
            node_id: self.Id(n),
            wrapping: wrapping.cloned(),
            annotate,
            sanitize,
            force_inline,
            remove_float,
        })
    }
    fn TextWrappingStyle(
        &self,
        wrapping: &Self::Style,
        sanitize: bool,
    ) -> Result<Self::Style, SerializationError> {
        self.Style(StyleOp::Text {
            wrapping: wrapping.clone(),
            sanitize,
        })
    }
    fn FullySelectedRootStyle(&self, n: usize) -> Result<Self::Style, SerializationError> {
        self.Style(StyleOp::FullySelectedRoot {
            node_id: self.Id(n),
        })
    }
    fn StyleText<'a>(&self, style: &'a Self::Style) -> &'a [u16] {
        style.SourceTextUTF16()
    }
}
fn SourceSelection(selection: DOMSelection) -> (Position<u64>, Position<u64>) {
    (
        SourcePosition(EditingPositionWithAffinity {
            position: selection.anchor,
            affinity: selection.affinity,
        })
        .position,
        SourcePosition(EditingPositionWithAffinity {
            position: selection.focus,
            affinity: selection.affinity,
        })
        .position,
    )
}
impl interaction_default::core::editing::serializers::markup_range::MarkupRangeContext
    for EditingPositionContext<'_>
{
    fn IsHTMLTag(&self, node: u64, name: &str) -> bool {
        self.Index(&node)
            .is_some_and(|i| self.document.Node(i).IsHTMLElement(name))
    }
    fn IsMathMLMath(&self, node: u64) -> bool {
        self.Index(&node).is_some_and(|i| {
            let node = self.document.Node(i);
            node.Namespace() == DOMNamespace::kMathML && node.Name() == "math"
        })
    }
    fn IsMailBlockquote(&self, node: u64) -> bool {
        self.Index(&node).is_some_and(|i| {
            let node = self.document.Node(i);
            node.IsHTMLElement("blockquote")
                && node
                    .FindAttribute("type")
                    .is_some_and(|a| a.value == "cite")
        })
    }
    fn IsTabSpan(&self, node: u64) -> bool {
        self.Index(&node).is_some_and(|i| {
            let node = self.document.Node(i);
            node.IsHTMLElement("span")
                && node.Children().first().is_some_and(|child| {
                    self.document.Node(*child).Type() == DOMNodeType::kText
                        && self.document.CharacterDataUTF16Slice(*child).contains(&9)
                })
                && self
                    .document
                    .ResolvedStyleFor(i)
                    .and_then(|s| s.ComputedStyle())
                    .is_some_and(|s| {
                        s.WhiteSpace() == layoutng_style::css::white_space::EWhiteSpace::kPre
                    })
        })
    }
    fn HasLayoutOwner(&self, node: u64) -> bool {
        self.Layout(&node).is_some()
    }
    fn SameNormalizedDOMRange(
        &self,
        node: u64,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> bool {
        let dom = self.Dom();
        let Some(layout) = self.layout else {
            return false;
        };
        let Some(anchor) = NativePosition(
            &Position::FirstPositionInNode(node),
            TextAffinity::Downstream,
        ) else {
            return false;
        };
        let Some(focus) = NativePosition(
            &Position::LastPositionInNode(&dom, &node),
            TextAffinity::Downstream,
        ) else {
            return false;
        };
        let Some(selection) = CreateVisibleDOMSelectionForFocus(
            self.document,
            layout,
            Some(DOMSelection {
                anchor: anchor.position,
                focus: focus.position,
                affinity: EditingTextAffinity::Downstream,
            }),
        ) else {
            return false;
        };
        let (anchor, focus) = SourceSelection(selection);
        let Some((normalized_start, normalized_end)) =
            interaction_default::core::editing::visible_selection::NormalizeSelectionRange(
                &dom, &anchor, &focus,
            )
        else {
            return false;
        };
        self.DomPosition(start) == normalized_start && self.DomPosition(end) == normalized_end
    }
}
fn PrepareRange(
    context: &EditingPositionContext<'_>,
    start: Position<u64>,
    end: Position<u64>,
    annotate: bool,
    constraining_ancestor: Option<u64>,
) -> Option<MarkupRange<usize>> {
    let range = interaction_default::core::editing::serializers::markup_range::PrepareRangeWithConstrainingAncestor(
        context, start, end, annotate, constraining_ancestor,
    )?;
    let offset = |offset: TextOffset<u64>| {
        Some(TextOffset {
            node: context.Index(&offset.node)?,
            offset: offset.offset,
        })
    };
    Some(MarkupRange {
        first_node: range.first_node.and_then(|n| context.Index(&n)),
        past_end: range.past_end.and_then(|n| context.Index(&n)),
        start: range.start.and_then(offset),
        end: range.end.and_then(offset),
        highest_node_to_serialize: range
            .highest_node_to_serialize
            .and_then(|n| context.Index(&n)),
        fully_selected_root: range.fully_selected_root.and_then(|n| context.Index(&n)),
        interchange_newline_at_start: range.interchange_newline_at_start,
        interchange_newline_at_end: range.interchange_newline_at_end,
        first_node_after_start_newline: range
            .first_node_after_start_newline
            .and_then(|n| context.Index(&n)),
        only_interchange_newline: range.only_interchange_newline,
    })
}
pub fn SerializeMarkupRange(
    document: &Document,
    layout: LayoutOutput<'_>,
    styles: &dyn MarkupStyleQueries,
    selection: DOMSelection,
    options: CreateMarkupOptions,
) -> Result<Vec<u16>, SerializationError> {
    SerializeMarkupRangeWithAncestor(document, layout, styles, selection, options, None)
}
pub fn SerializeMarkupRangeWithAncestor(
    document: &Document,
    layout: LayoutOutput<'_>,
    styles: &dyn MarkupStyleQueries,
    selection: DOMSelection,
    options: CreateMarkupOptions,
    constraining_ancestor: Option<u64>,
) -> Result<Vec<u16>, SerializationError> {
    let _lifecycle = document.Lifecycle().DisallowTransition();
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let flat = context.Flat();
    let (anchor, focus) = SourceSelection(selection);
    let Some((start, end)) =
        serialization::ComputeRangeForSerialization(&context, &flat, &anchor, &focus)
    else {
        return Ok(Vec::new());
    };
    SerializePrepared(
        &flat,
        styles,
        start,
        end,
        MarkupTree::Flat,
        options,
        constraining_ancestor,
    )
}
fn SerializePrepared(
    context: &EditingPositionContext<'_>,
    styles: &dyn MarkupStyleQueries,
    start: Position<u64>,
    end: Position<u64>,
    tree: MarkupTree,
    mut options: CreateMarkupOptions,
    constraining_ancestor: Option<u64>,
) -> Result<Vec<u16>, SerializationError> {
    options.serialization = NativeMarkupDocument(context.document).SerializationType();
    if constraining_ancestor.is_some_and(|node| context.document.FindNodeById(node).is_none()) {
        return Err(SerializationError::InvalidRange);
    }
    let range = PrepareRange(
        context,
        start,
        end,
        options.annotate_for_interchange,
        constraining_ancestor,
    )
    .ok_or(SerializationError::InvalidRange)?;
    let native = NativeStyledMarkup {
        native: NativeMarkupDocument(context.document),
        context: *context,
        styles,
    };
    styled::CreateMarkup(&native, range, tree, options)
}
pub fn SerializeMarkupBody(
    document: &Document,
    layout: LayoutOutput<'_>,
    styles: &dyn MarkupStyleQueries,
    body: usize,
) -> Result<Vec<u16>, SerializationError> {
    let _lifecycle = document.Lifecycle().DisallowTransition();
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let node = document.Node(body).Id();
    SerializePrepared(
        &context,
        styles,
        Position::FirstPositionInNode(node),
        Position::LastPositionInNode(&context, &node),
        MarkupTree::DOM,
        CreateMarkupOptions {
            annotate_for_interchange: true,
            for_markup_sanitization: true,
            ..Default::default()
        },
        None,
    )
}
