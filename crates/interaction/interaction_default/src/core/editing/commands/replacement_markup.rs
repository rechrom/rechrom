// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Source replace_selection_command.cc:897-1096,1120-1135,2502-2572.
//! Native DOM structure is moved directly; the HTML round-tripping operation
//! repairs paragraph/header nesting without serializing or parsing any markup.
use super::{
    paragraph_editing_context::{EditResult, EditingAborted},
    replace_selection_command::ReplacementFragmentDOM,
    structural_editing_context::{StructuralEditingContext, StructuralPositionQuery},
};
use crate::core::editing::{
    position::{Position, PositionDOM, PositionTree},
    text_offset_mapping::{NextNode, NextSkippingChildren},
};

pub trait ReplacementMarkupContext: StructuralEditingContext + ReplacementFragmentDOM {
    fn IsHTMLElement(&self, node: u64) -> bool;
    fn HasLocalName(&self, node: u64, name: &str) -> bool;
    fn IsSVGStyleElement(&self, node: u64) -> bool;
    fn NodeHasVisibleLayoutText(&self, node: u64) -> bool;
}
/// These are typed mutations of ReplaceSelectionCommand's one InsertedNodes
/// owner. Helpers never retain a second interval or a copied native tree.
pub trait ReplacementInsertedNodes {
    fn FirstNodeInserted(&self) -> Option<u64>;
    fn LastLeafInserted<C: PositionTree<Node = u64>>(&self, c: &C) -> Option<u64>;
    fn PastLastLeaf<C: PositionTree<Node = u64>>(&self, c: &C) -> Option<u64>;
    fn RespondToNodeInsertion(&mut self, node: u64);
    fn WillRemoveNodePreservingChildren<C: PositionTree<Node = u64>>(&mut self, c: &C, node: u64);
    fn DidReplaceNode(&mut self, old: u64, new: u64);
    fn WillRemoveNode<C: PositionTree<Node = u64>>(&mut self, c: &C, node: u64);
}
const PROHIBITED_PARAGRAPH_CHILDREN: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
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
    "hr",
    "li",
    "listing",
    "main",
    "menu",
    "nav",
    "ol",
    "p",
    "plaintext",
    "pre",
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
    "xmp",
];
pub fn IsProhibitedParagraphChild<C: ReplacementMarkupContext>(c: &C, node: u64) -> bool {
    PROHIBITED_PARAGRAPH_CHILDREN
        .iter()
        .any(|name| c.HasLocalName(node, name))
}
pub fn IsHtmlHeaderElement<C: StructuralEditingContext>(c: &C, node: u64) -> bool {
    ["h1", "h2", "h3", "h4", "h5", "h6"]
        .iter()
        .any(|name| c.Tag(node, name))
}
pub fn IsHtmlListElement<C: StructuralEditingContext>(c: &C, node: u64) -> bool {
    c.Tag(node, "ol") || c.Tag(node, "ul")
}
pub fn HasBlockLevelContent<C: ReplacementMarkupContext>(c: &C, fragment: u64) -> bool {
    let mut node = c.ChildAt(&fragment, 0);
    while let Some(current) = node {
        if c.IsElement(current) && IsProhibitedParagraphChild(c, current) {
            return true;
        }
        node = NextNode(c, &current, Some(&fragment));
    }
    false
}
pub fn RemoveHeadContents<C: ReplacementMarkupContext>(c: &C, fragment: u64) -> EditResult {
    let mut node = c.ChildAt(&fragment, 0);
    while let Some(current) = node {
        if ["base", "link", "meta", "style", "title"]
            .iter()
            .any(|name| c.Tag(current, name))
            || c.IsSVGStyleElement(current)
        {
            node = NextSkippingChildren(c, &current, Some(&fragment));
            c.RemoveFragmentNode(current)?;
        } else {
            node = NextNode(c, &current, Some(&fragment));
        }
    }
    Ok(())
}
fn EnclosingElement<C: StructuralEditingContext>(
    c: &C,
    node: u64,
    highest: bool,
    predicate: impl Fn(u64) -> bool,
) -> Option<u64> {
    let mut ancestor = Some(node);
    let mut root = None;
    while let Some(node) = ancestor {
        if c.Editable(node) {
            root = Some(node);
        }
        ancestor = c.Parent(&node);
    }
    let mut current = c.Parent(&node);
    let mut found = None;
    while let Some(node) = current {
        if (root.is_none() || c.Editable(node)) && predicate(node) {
            if !highest {
                return Some(node);
            }
            found = Some(node);
        }
        if Some(node) == root {
            break;
        }
        current = c.Parent(&node);
    }
    found
}
/// Source :983-1026. Mutation repairs are bounded by the original insertion's
/// past-last leaf, exactly as the native NodeTraversal interval in Chromium.
pub fn MakeInsertedContentRoundTrippableWithHTMLTreeBuilder<
    C: ReplacementMarkupContext,
    N: ReplacementInsertedNodes,
>(
    c: &C,
    inserted: &N,
) -> EditResult {
    let past = inserted.PastLastLeaf(c);
    let mut node = inserted.FirstNodeInserted();
    while let Some(current) = node.filter(|n| Some(*n) != past) {
        node = NextNode(c, &current, None);
        if !c.IsHTMLElement(current) || !c.IsConnected(&current) {
            continue;
        }
        if IsProhibitedParagraphChild(c, current) {
            if let Some(paragraph) = EnclosingElement(c, current, false, |n| c.Tag(n, "p")) {
                MoveElementOutOfAncestor(c, current, paragraph)?;
            }
        }
        if IsHtmlHeaderElement(c, current) {
            if let Some(header) = EnclosingElement(c, current, true, |n| IsHtmlHeaderElement(c, n))
            {
                MoveElementOutOfAncestor(c, current, header)?;
            }
        }
    }
    Ok(())
}
/// Source :1028-1064. SplitTreeToNode's split-ancestor branch uses the
/// ancestor's parent as the stopping owner, retaining the original right half.
pub fn MoveElementOutOfAncestor<C: StructuralEditingContext>(
    c: &C,
    element: u64,
    ancestor: u64,
) -> EditResult {
    let Some(parent) = c.Parent(&ancestor).filter(|n| c.Editable(*n)) else {
        return Ok(());
    };
    c.UpdateStyleAndLayout();
    let end = c.QueryPosition(
        &Position::LastPositionInOrAfterNode(c, &element),
        StructuralPositionQuery::Visible,
    );
    let ancestor_end = c.QueryPosition(
        &Position::LastPositionInNode(c, &ancestor),
        StructuralPositionQuery::Visible,
    );
    if end == ancestor_end {
        c.Remove(element)?;
        if let Some(next) = c.NextSibling(&ancestor) {
            c.InsertBefore(element, next)?;
        } else {
            c.Append(element, parent)?;
        }
    } else {
        let split_to = super::composite_edit_command::SplitTreeToNode(c, element, parent)?;
        c.Remove(element)?;
        c.InsertBefore(element, split_to)?;
    }
    if c.ChildCount(&ancestor) == 0 {
        c.Remove(ancestor)?;
    }
    Ok(())
}
/// Source :1072-1096. ResolvedTextLength belongs to the native LayoutText
/// owner; DOM CharacterData length is not a proxy for rendered text.
pub fn RemoveUnrenderedTextNodesAtEnds<C: ReplacementMarkupContext, N: ReplacementInsertedNodes>(
    c: &C,
    inserted: &mut N,
) -> EditResult {
    c.UpdateStyleAndLayout();
    if let Some(last) = inserted.LastLeafInserted(c).filter(|n| c.IsText(n)) {
        if !c.NodeHasVisibleLayoutText(last)
            && EnclosingElement(c, last, false, |n| c.Tag(n, "select")).is_none()
            && EnclosingElement(c, last, false, |n| c.Tag(n, "script")).is_none()
        {
            inserted.WillRemoveNode(c, last);
            c.Remove(last)?;
        }
    }
    if let Some(first) = inserted.FirstNodeInserted().filter(|n| c.IsText(n)) {
        c.UpdateStyleAndLayout();
        if !c.NodeHasVisibleLayoutText(first) {
            inserted.WillRemoveNode(c, first);
            c.Remove(first)?;
        }
    }
    Ok(())
}
fn IsParagraphEdge<C: StructuralEditingContext>(c: &C, p: &Position<u64>, start: bool) -> bool {
    let visible = c.QueryPosition(p, StructuralPositionQuery::Visible);
    !visible.IsNull()
        && visible
            == c.QueryPosition(
                &c.QueryPosition(
                    p,
                    if start {
                        StructuralPositionQuery::ParagraphStart
                    } else {
                        StructuralPositionQuery::ParagraphEnd
                    },
                ),
                StructuralPositionQuery::Visible,
            )
}
fn InsertAfter<C: StructuralEditingContext>(c: &C, node: u64, after: u64) -> EditResult {
    if let Some(next) = c.NextSibling(&after) {
        c.InsertBefore(node, next)
    } else {
        c.Append(node, c.Parent(&after).ok_or(EditingAborted)?)
    }
}
/// Source :2502-2572. List children are detached through the fragment DOM
/// operation, then inserted with ordinary CompositeEditCommand children.
pub fn InsertAsListItems<C: ReplacementMarkupContext, N: ReplacementInsertedNodes>(
    c: &C,
    mut list: u64,
    block: u64,
    position: &Position<u64>,
    inserted: &mut N,
) -> EditResult<u64> {
    let list_is_item = c.Tag(list, "li");
    let mut item = if list_is_item {
        Some(list)
    } else {
        while c.ChildCount(&list) == 1
            && c.ChildAt(&list, 0).is_some_and(|n| IsHtmlListElement(c, n))
        {
            list = c.ChildAt(&list, 0).unwrap();
        }
        c.ChildAt(&list, 0)
    };
    c.UpdateStyleAndLayout();
    let start = IsParagraphEdge(c, position, true);
    let end = IsParagraphEdge(c, position, false);
    let middle = !start && !end;
    let mut last = block;
    if middle {
        let anchor = *position.AnchorNode().ok_or(EditingAborted)?;
        let offset = position.ComputeOffsetInContainerNode(c);
        if c.IsText(&anchor) && offset > 0 {
            c.SplitText(anchor, offset)?;
        }
        let split_to = c.Parent(&last).unwrap_or(last);
        super::composite_edit_command::SplitTreeToNode(c, anchor, split_to)?;
    }
    while let Some(current) = item {
        if !list_is_item {
            c.RemoveFragmentNode(current)?;
        }
        if start || middle {
            c.InsertBefore(current, last)?;
            inserted.RespondToNodeInsertion(current);
        } else if end {
            InsertAfter(c, current, last)?;
            inserted.RespondToNodeInsertion(current);
            last = current;
        } else {
            return Err(EditingAborted);
        }
        if list_is_item {
            break;
        }
        item = c.ChildAt(&list, 0);
    }
    if start || middle {
        if let Some(previous) = c.PreviousSibling(&last) {
            return Ok(previous);
        }
    }
    Ok(last)
}
