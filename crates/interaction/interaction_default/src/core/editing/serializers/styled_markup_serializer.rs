// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! styled_markup_serializer.cc, pinned 6c1d401fcca5. Traversal borrows native
//! DOM/flat-tree identities. Style algebra is performed by the Style owner.
use super::{
    markup_accumulator::{MarkupDocument, MarkupNodeKind},
    styled_markup_accumulator::{CreateMarkupOptions, StyledMarkupAccumulator, TextOffset},
};
use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkupTree {
    DOM,
    Flat,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct SerializationNodeFacts {
    pub has_layout_object: bool,
    pub is_html_element: bool,
    pub enclosing_block: bool,
    pub can_have_children_for_editing: bool,
    pub selectable: bool,
    pub display_contents: bool,
    pub preserve_breaks: bool,
    pub user_agent_shadow_host: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerializationError {
    InvalidRange,
    MissingStyle,
    MissingLayout,
    UnsupportedResource,
}
/// Geometry preparation uses the existing VisiblePosition/paragraph owner.
/// These are range-local facts; no fragment or document tree is materialized.
pub struct MarkupRange<N> {
    pub first_node: Option<N>,
    pub past_end: Option<N>,
    pub start: Option<TextOffset<N>>,
    pub end: Option<TextOffset<N>>,
    pub highest_node_to_serialize: Option<N>,
    pub fully_selected_root: Option<N>,
    pub interchange_newline_at_start: bool,
    pub interchange_newline_at_end: bool,
    pub first_node_after_start_newline: Option<N>,
    pub only_interchange_newline: bool,
}
pub trait StyledMarkupDocument: MarkupDocument
where
    Self::Node: Eq + Hash,
{
    type Style: Clone;
    fn Parent(&self, node: Self::Node, tree: MarkupTree) -> Option<Self::Node>;
    fn FirstChild(&self, node: Self::Node, tree: MarkupTree) -> Option<Self::Node>;
    fn NextSibling(&self, node: Self::Node, tree: MarkupTree) -> Option<Self::Node>;
    fn Facts(&self, node: Self::Node) -> SerializationNodeFacts;
    fn RenderedText(
        &self,
        node: Self::Node,
        start: usize,
        end: usize,
        ignore_css_text_transforms: bool,
    ) -> Result<Vec<u16>, SerializationError>;
    fn WrappingStyle(
        &self,
        parent: Self::Node,
        annotate: bool,
    ) -> Result<Option<Self::Style>, SerializationError>;
    fn InlineStyle(
        &self,
        node: Self::Node,
        wrapping: Option<&Self::Style>,
        annotate: bool,
        sanitize: bool,
        force_inline: bool,
        remove_float: bool,
    ) -> Result<Self::Style, SerializationError>;
    fn TextWrappingStyle(
        &self,
        wrapping: &Self::Style,
        sanitize: bool,
    ) -> Result<Self::Style, SerializationError>;
    fn FullySelectedRootStyle(&self, node: Self::Node) -> Result<Self::Style, SerializationError>;
    fn StyleText<'a>(&self, style: &'a Self::Style) -> &'a [u16];
}

fn Tag<D: MarkupDocument>(document: &D, node: D::Node, name: &str) -> bool {
    document.Kind(node) == MarkupNodeKind::Element
        && document
            .QualifiedName(node)
            .into_iter()
            .eq(name.encode_utf16())
}
fn TablePart<D: MarkupDocument>(d: &D, n: D::Node) -> bool {
    [
        "tbody", "tfoot", "thead", "tr", "td", "th", "caption", "colgroup",
    ]
    .iter()
    .any(|name| Tag(d, n, name))
}
fn Presentational<D: MarkupDocument>(d: &D, n: D::Node) -> bool {
    [
        "b", "strong", "i", "em", "u", "s", "strike", "sub", "sup", "font", "tt", "big", "small",
    ]
    .iter()
    .any(|name| Tag(d, n, name))
}
fn Descendant<D: StyledMarkupDocument>(
    d: &D,
    mut n: D::Node,
    ancestor: D::Node,
    tree: MarkupTree,
) -> bool
where
    D::Node: Eq + Hash,
{
    while let Some(parent) = d.Parent(n, tree) {
        if parent == ancestor {
            return true;
        }
        n = parent;
    }
    false
}
fn NextSkippingChildren<D: StyledMarkupDocument>(
    d: &D,
    mut n: D::Node,
    tree: MarkupTree,
) -> Option<D::Node>
where
    D::Node: Eq + Hash,
{
    loop {
        if let Some(next) = d.NextSibling(n, tree) {
            return Some(next);
        }
        n = d.Parent(n, tree)?;
    }
}
fn Next<D: StyledMarkupDocument>(d: &D, n: D::Node, tree: MarkupTree) -> Option<D::Node>
where
    D::Node: Eq + Hash,
{
    d.FirstChild(n, tree)
        .or_else(|| NextSkippingChildren(d, n, tree))
}
fn AncestorTag<D: StyledMarkupDocument>(
    d: &D,
    mut n: D::Node,
    names: &[&str],
    tree: MarkupTree,
) -> Option<D::Node>
where
    D::Node: Eq + Hash,
{
    while let Some(parent) = d.Parent(n, tree) {
        if names.iter().any(|name| Tag(d, parent, name)) {
            return Some(parent);
        }
        n = parent;
    }
    None
}
struct Traverser<'a, D: StyledMarkupDocument>
where
    D::Node: Eq + Hash,
{
    document: &'a D,
    accumulator: StyledMarkupAccumulator<'a, D>,
    last_closed: Option<D::Node>,
    wrapping_style: Option<D::Style>,
    tree: MarkupTree,
    emitting: bool,
}
impl<'a, D: StyledMarkupDocument> Traverser<'a, D>
where
    D::Node: Eq + Hash,
{
    fn ApplyWrappingStyle(&self, node: D::Node) -> bool {
        self.last_closed.is_some_and(|last| {
            self.document.Parent(last, self.tree) == self.document.Parent(node, self.tree)
        }) && self.wrapping_style.is_some()
    }
    fn NeedsInlineStyle(&self, node: D::Node) -> bool {
        let facts = self.document.Facts(node);
        facts.is_html_element
            && (self.accumulator.options.annotate_for_interchange
                || self.accumulator.options.convert_blocks_to_inlines && facts.enclosing_block)
    }
    fn InlineStyle(
        &self,
        node: D::Node,
        remove_float: bool,
    ) -> Result<D::Style, SerializationError> {
        let options = self.accumulator.options;
        self.document.InlineStyle(
            node,
            if self.ApplyWrappingStyle(node) {
                self.wrapping_style.as_ref()
            } else {
                None
            },
            options.annotate_for_interchange,
            options.for_markup_sanitization,
            options.convert_blocks_to_inlines && self.document.Facts(node).enclosing_block,
            remove_float,
        )
    }
    fn WrapWithNode(&mut self, node: D::Node) -> Result<(), SerializationError> {
        if !self.emitting {
            return Ok(());
        }
        if self.document.Kind(node) != MarkupNodeKind::Element {
            return Ok(());
        }
        let style = if self.ApplyWrappingStyle(node) || self.NeedsInlineStyle(node) {
            Some(self.InlineStyle(node, true)?)
        } else {
            None
        };
        let open = self.accumulator.ElementMarkup(
            node,
            style.as_ref().map(|s| Some(self.document.StyleText(s))),
        );
        self.accumulator.PushMarkup(open);
        self.accumulator.AppendEndTag(node);
        Ok(())
    }
    fn AppendStartMarkup(&mut self, node: D::Node) -> Result<(), SerializationError> {
        if !self.emitting {
            return Ok(());
        }
        match self.document.Kind(node) {
            MarkupNodeKind::Text => {
                // CDATASection is a native Text subclass with a distinct
                // node type in source StyledMarkupTraverser.cc:588–632.
                if self.document.IsCDATASection(node) {
                    self.accumulator.AppendStartMarkup(node);
                    return Ok(());
                }
                if self
                    .document
                    .Parent(node, MarkupTree::DOM)
                    .is_some_and(|p| Tag(self.document, p, "textarea"))
                {
                    self.accumulator.AppendText(node);
                    return Ok(());
                }
                let style = if self.ApplyWrappingStyle(node) {
                    Some(self.document.TextWrappingStyle(
                        self.wrapping_style.as_ref().unwrap(),
                        self.accumulator.options.for_markup_sanitization,
                    )?)
                } else {
                    None
                };
                let (start, end) = self.accumulator.TextBounds(node);
                let text = if !self.accumulator.options.annotate_for_interchange
                    || AncestorTag(self.document, node, &["select"], MarkupTree::DOM).is_some()
                {
                    self.document.Data(node)[start..end].to_vec()
                } else {
                    self.document.RenderedText(
                        node,
                        start,
                        end,
                        self.accumulator.options.ignore_css_text_transforms,
                    )?
                };
                self.accumulator.AppendTextWithInlineStyle(
                    node,
                    &text,
                    style.as_ref().map(|s| self.document.StyleText(s)),
                    self.document.Facts(node).preserve_breaks,
                );
            }
            MarkupNodeKind::Element => {
                if self.document.Facts(node).is_html_element
                    && self.accumulator.options.annotate_for_interchange
                    || self.ApplyWrappingStyle(node)
                {
                    let style = self.InlineStyle(node, false)?;
                    self.accumulator
                        .AppendElementWithInlineStyle(node, Some(self.document.StyleText(&style)));
                } else {
                    self.accumulator.AppendElement(node);
                }
            }
            _ => self.accumulator.AppendStartMarkup(node),
        }
        Ok(())
    }
    fn AppendEndMarkup(&mut self, node: D::Node) {
        if self.emitting && self.document.Kind(node) == MarkupNodeKind::Element {
            self.accumulator.AppendEndTag(node);
        }
    }
    fn ContainsOnlyBR(&self, node: D::Node) -> bool {
        self.document.ChildCount(node) == 1
            && Tag(self.document, self.document.ChildAt(node, 0), "br")
    }
    fn HasSelectableDescendants(&self, node: D::Node, cache: &mut HashMap<D::Node, bool>) -> bool {
        let facts = self.document.Facts(node);
        if !facts.has_layout_object || facts.selectable {
            return true;
        }
        if let Some(result) = cache.get(&node) {
            return *result;
        }
        let mut child = self.document.FirstChild(node, self.tree);
        while let Some(n) = child {
            if self.HasSelectableDescendants(n, cache) {
                cache.insert(node, true);
                return true;
            }
            child = self.document.NextSibling(n, self.tree);
        }
        cache.insert(node, false);
        false
    }
    fn Traverse(
        &mut self,
        start: Option<D::Node>,
        past_end: Option<D::Node>,
    ) -> Result<Option<D::Node>, SerializationError> {
        let mut ancestors_to_close = Vec::new();
        let mut has_selectable_descendants = HashMap::new();
        let mut current = start;
        let mut last_closed = None;
        while let Some(node) = current.filter(|n| Some(*n) != past_end) {
            let mut next;
            if self.tree == MarkupTree::Flat && self.document.Facts(node).user_agent_shadow_host {
                let old_tree = self.tree;
                let old_wrapping = self.wrapping_style.clone();
                self.tree = MarkupTree::DOM;
                self.wrapping_style = if self.emitting {
                    match self
                        .last_closed
                        .and_then(|n| self.document.Parent(n, MarkupTree::DOM))
                        .filter(|n| self.document.Kind(*n) == MarkupNodeKind::Element)
                    {
                        Some(parent) => self.document.WrappingStyle(
                            parent,
                            self.accumulator.options.annotate_for_interchange,
                        )?,
                        None => None,
                    }
                } else {
                    None
                };
                last_closed = self.Traverse(
                    Some(node),
                    NextSkippingChildren(self.document, node, MarkupTree::DOM),
                )?;
                self.tree = old_tree;
                self.wrapping_style = old_wrapping;
                next = NextSkippingChildren(self.document, node, self.tree);
            } else {
                next = Next(self.document, node, self.tree);
                let facts = self.document.Facts(node);
                if facts.enclosing_block
                    && facts.can_have_children_for_editing
                    && next == past_end
                    && !self.ContainsOnlyBR(node)
                    && !TablePart(self.document, node)
                {
                    current = next;
                    continue;
                }
                let skip = self.accumulator.options.skip_unselectable_content
                    && facts.has_layout_object
                    && !facts.selectable
                    && !self.HasSelectableDescendants(node, &mut has_selectable_descendants);
                let unrendered = facts.display_contents
                    || AncestorTag(self.document, node, &["select"], MarkupTree::DOM).is_some()
                    || self.accumulator.options.for_markup_sanitization
                        && Tag(self.document, node, "iframe");
                if (facts.has_layout_object || unrendered) && !skip {
                    self.AppendStartMarkup(node)?;
                    if self.document.FirstChild(node, self.tree).is_some() {
                        if next == past_end && self.ContainsOnlyBR(node) {
                            let br = next.ok_or(SerializationError::InvalidRange)?;
                            self.AppendStartMarkup(br)?;
                            self.AppendEndMarkup(br);
                            last_closed = Some(br);
                        } else {
                            ancestors_to_close.push(node);
                            current = next;
                            continue;
                        }
                    } else {
                        self.AppendEndMarkup(node);
                        last_closed = Some(node);
                    }
                } else {
                    next = NextSkippingChildren(self.document, node, self.tree);
                    if past_end.is_some_and(|end| Descendant(self.document, end, node, self.tree)) {
                        next = past_end;
                    }
                }
            }
            if self.document.NextSibling(node, self.tree).is_some() && next != past_end {
                current = next;
                continue;
            }
            while let Some(&ancestor) = ancestors_to_close.last() {
                if next.is_some_and(|n| {
                    Some(n) != past_end && Descendant(self.document, n, ancestor, self.tree)
                }) {
                    break;
                }
                self.AppendEndMarkup(ancestor);
                last_closed = Some(ancestor);
                ancestors_to_close.pop();
            }
            let next_parent = next.and_then(|n| self.document.Parent(n, self.tree));
            if next != past_end && Some(node) != next_parent {
                let last = last_closed
                    .filter(|last| Descendant(self.document, node, *last, self.tree))
                    .unwrap_or(node);
                let mut parent = self.document.Parent(last, self.tree);
                while let Some(n) = parent.filter(|n| Some(*n) != next_parent) {
                    if self.document.Facts(n).has_layout_object {
                        self.WrapWithNode(n)?;
                        last_closed = Some(n);
                    }
                    parent = self.document.Parent(n, self.tree);
                }
            }
            current = next;
        }
        while let Some(ancestor) = ancestors_to_close.pop() {
            self.AppendEndMarkup(ancestor);
            last_closed = Some(ancestor);
        }
        Ok(last_closed)
    }
}

pub fn CreateMarkup<D: StyledMarkupDocument>(
    document: &D,
    range: MarkupRange<D::Node>,
    tree: MarkupTree,
    options: CreateMarkupOptions,
) -> Result<Vec<u16>, SerializationError>
where
    D::Node: Eq + Hash,
{
    let mut first = range.first_node;
    let mut accumulator = StyledMarkupAccumulator::new(document, range.start, range.end, options);
    if options.annotate_for_interchange && range.interchange_newline_at_start {
        accumulator.AppendInterchangeNewline();
        if range.only_interchange_newline {
            return Ok(accumulator.TakeResults());
        }
        first = range.first_node_after_start_newline;
        if first == range.past_end {
            return Ok(accumulator.TakeResults());
        }
    }
    let mut last_closed = range.highest_node_to_serialize;
    let mut append_parent = false;
    if last_closed.is_none() {
        // Dry traversal has the identical renderability/selection policy but no
        // text/style/raster work or markup allocation.
        let mut dry = Traverser {
            document,
            accumulator: StyledMarkupAccumulator::new(
                document,
                None,
                None,
                CreateMarkupOptions::default(),
            ),
            last_closed: None,
            wrapping_style: None,
            tree,
            emitting: false,
        };
        last_closed = dry.Traverse(first, range.past_end)?;
        if let Some(last) = last_closed {
            if document.Kind(last) == MarkupNodeKind::Text {
                if let Some(parent) = document
                    .Parent(last, tree)
                    .filter(|p| Presentational(document, *p))
                {
                    last_closed = Some(parent);
                    append_parent = true;
                }
            } else if TablePart(document, last) {
                if let Some(table) = AncestorTag(document, last, &["table"], tree) {
                    last_closed = Some(table);
                    append_parent = true;
                }
            } else if ["li", "dt", "dd"]
                .iter()
                .any(|tag| Tag(document, last, tag))
            {
                if let Some(list) =
                    AncestorTag(document, last, &["ul", "ol", "dl"], MarkupTree::DOM)
                {
                    last_closed = Some(list);
                    append_parent = true;
                }
            }
        }
    }
    let wrapping_style = match last_closed
        .and_then(|n| document.Parent(n, tree))
        .filter(|n| document.Kind(*n) == MarkupNodeKind::Element)
    {
        Some(parent) => document.WrappingStyle(parent, options.annotate_for_interchange)?,
        None => None,
    };
    let mut traverser = Traverser {
        document,
        accumulator,
        last_closed,
        wrapping_style,
        tree,
        emitting: true,
    };
    let closed = traverser.Traverse(first, range.past_end)?;
    if let (Some(highest), Some(last)) = (range.highest_node_to_serialize, closed) {
        let mut ancestor = document.Parent(last, tree);
        while let Some(node) = ancestor {
            if !Tag(document, node, "mtr") {
                if Some(node) == range.fully_selected_root && !options.convert_blocks_to_inlines {
                    let style = document.FullySelectedRootStyle(node)?;
                    traverser
                        .accumulator
                        .WrapWithStyleNode(document.StyleText(&style));
                } else {
                    traverser.WrapWithNode(node)?;
                }
            }
            if node == highest {
                break;
            }
            ancestor = document.Parent(node, tree);
        }
    } else if append_parent {
        if let Some(node) = last_closed {
            traverser.WrapWithNode(node)?;
        }
    }
    if !closed.is_some_and(|n| Tag(document, n, "br"))
        && options.annotate_for_interchange
        && range.interchange_newline_at_end
    {
        traverser.accumulator.AppendInterchangeNewline();
    }
    Ok(traverser.accumulator.TakeResults())
}

#[cfg(test)]
#[path = "styled_markup_serializer_tests.rs"]
mod tests;
