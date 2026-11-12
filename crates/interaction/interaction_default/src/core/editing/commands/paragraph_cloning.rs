// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! CompositeEditCommand's markup-preserving paragraph move. The native DOM
//! remains the owner; clones are actual DOM nodes, and deletion is the existing
//! source command in the same composite/undo transaction.
use super::{
    delete_selection_command::DeleteSelectionCommand,
    paragraph_editing_context::{EditResult, EditingAborted},
    replace_selection_command::{self, ReplacementFacts},
    structural_editing_context::{StructuralBooleanQuery, StructuralPositionQuery as Q},
};
use crate::core::editing::{
    position::Position,
    text_offset_mapping::{IsDescendantOf, NextSkippingChildren},
};

pub trait ParagraphCloningContext: ReplacementFacts {
    fn CloneNode(&self, node: u64, deep: bool) -> EditResult<u64>;
    fn SetEndingRange(&self, start: &Position<u64>, end: &Position<u64>);
}
pub(super) fn Contains<C: ParagraphCloningContext>(c: &C, parent: u64, node: u64) -> bool {
    parent == node || IsDescendantOf(c, &node, &parent)
}
pub(super) fn InsertAfter<C: ParagraphCloningContext>(c: &C, node: u64, after: u64) -> EditResult {
    if let Some(next) = c.NextSibling(&after) {
        c.InsertBefore(node, next)
    } else {
        c.Append(node, c.Parent(&after).ok_or(EditingAborted)?)
    }
}
/// composite_edit_command.cc:1277-1373.
pub fn CloneParagraphUnderNewElement<C: ParagraphCloningContext>(
    c: &C,
    start: &Position<u64>,
    end: &Position<u64>,
    mut outer: u64,
    block: u64,
) -> EditResult {
    let start_node = *start.AnchorNode().ok_or(EditingAborted)?;
    let end_node = *end.AnchorNode().ok_or(EditingAborted)?;
    let mut last = if c.RootEditable(outer) == Some(outer) {
        block
    } else {
        let clone = c.CloneNode(outer, c.IsDisplayInsideTable(&outer))?;
        c.Append(clone, block)?;
        clone
    };
    if start_node != outer && c.IsElement(last) && IsDescendantOf(c, &start_node, &outer) {
        let mut ancestors = Vec::new();
        let mut runner = Some(start_node);
        while let Some(node) = runner.filter(|n| *n != outer) {
            ancestors.push(node);
            runner = c.Parent(&node);
        }
        for node in ancestors.into_iter().rev() {
            let clone = c.CloneNode(node, c.IsDisplayInsideTable(&node))?;
            c.Append(clone, last)?;
            last = clone;
        }
    }
    if !c.IsConnected(&outer) {
        return Ok(());
    }
    if start_node != end_node && !IsDescendantOf(c, &start_node, &end_node) {
        while !IsDescendantOf(c, &end_node, &outer) {
            let Some(parent) = c.Parent(&outer) else {
                return Ok(());
            };
            outer = parent;
        }
        let mut start_node = start_node;
        let mut runner = NextSkippingChildren(c, &start_node, Some(&outer));
        while let Some(node) = runner {
            while c.Parent(&start_node) != c.Parent(&node) {
                start_node = c.Parent(&start_node).ok_or(EditingAborted)?;
                last = c.Parent(&last).ok_or(EditingAborted)?;
            }
            if c.Parent(&last).is_none() {
                return Ok(());
            }
            let clone = c.CloneNode(node, true)?;
            InsertAfter(c, clone, last)?;
            last = clone;
            if node == end_node || IsDescendantOf(c, &end_node, &node) {
                break;
            }
            runner = NextSkippingChildren(c, &node, Some(&outer));
        }
    }
    Ok(())
}
pub(super) fn AtEdge<C: ParagraphCloningContext>(c: &C, p: &Position<u64>, q: Q) -> bool {
    let visible = c.QueryPosition(p, Q::Visible);
    !visible.IsNull() && visible == c.QueryPosition(&c.QueryPosition(p, q), Q::Visible)
}
/// composite_edit_command.cc:1434-1490, DOM-position overload.
pub(super) fn CleanupAfterDeletion<C: ParagraphCloningContext>(
    c: &C,
    caret: &Position<u64>,
    destination: Option<&Position<u64>>,
) -> EditResult {
    c.UpdateStyleAndLayout();
    if destination.is_some_and(|p| p == caret)
        || !AtEdge(c, caret, Q::ParagraphStart)
        || !AtEdge(c, caret, Q::ParagraphEnd)
    {
        return Ok(());
    }
    let position = c.QueryPosition(caret, Q::Forward);
    let Some(node) = position.AnchorNode().copied() else {
        return Ok(());
    };
    if let Some(destination) = destination.and_then(|p| p.AnchorNode()) {
        if *destination == node {
            return Err(EditingAborted);
        }
        if IsDescendantOf(c, destination, &node) {
            return Ok(());
        }
    }
    if c.Tag(node, "br") {
        replace_selection_command::RemoveNodeAndPruneAncestors(c, node)
    } else if c.NodeFacts(node).block {
        // A null destination renders in the same position. Preserve that
        // source prune branch, including protection for an editable root.
        if !c.PositionBoolean(
            &position,
            destination,
            StructuralBooleanQuery::RendersInDifferentPosition,
        ) {
            replace_selection_command::PruneAncestors(c, Some(node))
        } else {
            replace_selection_command::RemoveNodeAndPruneAncestors(c, node)
        }
    } else if c.PositionBoolean(&position, None, StructuralBooleanQuery::LineBreakExists) {
        if c.CharacterDataLength(&node) == Some(1) {
            replace_selection_command::RemoveNodeAndPruneAncestors(c, node)
        } else {
            c.DeleteText(node, position.ComputeOffsetInContainerNode(c), 1)
        }
    } else {
        Ok(())
    }
}
/// composite_edit_command.cc:1498-1577. Cloning and source deletion are kept in
/// this order: moving the original children is not an equivalent operation.
pub fn MoveParagraphWithClones<C: ParagraphCloningContext>(
    c: &C,
    start: &Position<u64>,
    end: &Position<u64>,
    block: u64,
    outer: u64,
) -> EditResult {
    if start.IsNull() || end.IsNull() {
        return Err(EditingAborted);
    }
    let before = c.QueryPosition(start, Q::Previous);
    let after = c.QueryPosition(end, Q::Next);
    let before = if before.IsNull() {
        None
    } else {
        Some(c.Track(&before)?)
    };
    let after = if after.IsNull() {
        None
    } else {
        Some(c.Track(&after)?)
    };
    let from = c.QueryPosition(start, Q::Forward);
    let mut to = if start == end {
        from.clone()
    } else {
        c.QueryPosition(end, Q::Backward)
    };
    if c.Compare(&from, &to) == Some(std::cmp::Ordering::Greater) {
        to = from.clone();
    }
    CloneParagraphUnderNewElement(c, &from, &to, outer, block)?;
    c.SetEndingRange(&from, &to);
    let caret = DeleteSelectionCommand::Apply(c, (from, to))?;
    CleanupAfterDeletion(c, &caret, None)?;
    c.UpdateStyleAndLayout();
    let before = before.as_ref().map_or_else(Position::default, |p| {
        c.QueryPosition(&c.Tracked(p), Q::Visible)
    });
    let after = after.as_ref().map_or_else(Position::default, |p| {
        c.QueryPosition(&c.Tracked(p), Q::Visible)
    });
    if !before.IsNull()
        && before
            .AnchorNode()
            .is_some_and(|n| !c.IsDisplayInsideTable(n))
        && ((!AtEdge(c, &before, Q::ParagraphStart) && !AtEdge(c, &before, Q::ParagraphEnd))
            || before == after)
    {
        let br = c.CreateElement("br")?;
        super::composite_edit_command::InsertNodeAt(c, br, &before)?;
    }
    Ok(())
}
