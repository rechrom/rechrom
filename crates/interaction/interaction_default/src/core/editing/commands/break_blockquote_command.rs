// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Source break_blockquote_command.cc:63-349. ReplaceSelectionCommand invokes
//! this source command after its own deletion, so its entry is a native caret.
use super::{
    paragraph_editing_context::{EditResult, EditingAborted},
    structural_editing_context::{
        StructuralBooleanQuery, StructuralEditingContext, StructuralPositionQuery,
    },
};
use crate::core::editing::{
    position::{Position, PositionTree},
    text_offset_mapping::{IsDescendantOf, NextNode},
};

#[derive(Clone, Copy, Default)]
pub struct BlockPlaceholderFacts {
    pub block_flow: bool,
    pub height_zero: bool,
    pub empty_list_item: bool,
}
pub trait BreakBlockquoteContext: StructuralEditingContext {
    fn ListItemNumber(&self, node: u64) -> Option<i32>;
    fn SetListStart(&self, node: u64, number: i32) -> EditResult;
    fn BlockPlaceholderFacts(&self, node: u64) -> BlockPlaceholderFacts;
    fn MoveByGrapheme(&self, position: &Position<u64>, forward: bool) -> Position<u64>;
    fn RebalanceBreakWhitespace(&self, position: &Position<u64>) -> EditResult;
}
fn IsMailQuote<C: StructuralEditingContext>(c: &C, node: u64) -> bool {
    c.Tag(node, "blockquote") && c.AttributeEquals(node, "type", "cite")
}
fn HighestEditableRoot<C: StructuralEditingContext>(c: &C, start: u64) -> Option<u64> {
    let mut node = Some(start);
    let mut root = None;
    while let Some(current) = node {
        if c.Editable(current) {
            root = Some(current);
        }
        node = c.Parent(&current);
    }
    root
}
pub(super) fn EnclosingQuote<C: StructuralEditingContext>(
    c: &C,
    position: &Position<u64>,
    highest: bool,
) -> Option<u64> {
    let mut node = position.ComputeContainerNode(c);
    let root = HighestEditableRoot(c, node?);
    let mut found = None;
    while let Some(current) = node {
        if (root.is_none() || c.Editable(current)) && IsMailQuote(c, current) {
            if !highest {
                return Some(current);
            }
            found = Some(current);
        }
        if Some(current) == root {
            break;
        }
        node = c.Parent(&current);
    }
    found
}
fn TopBlockquoteOf<C: StructuralEditingContext>(c: &C, start: &Position<u64>) -> Option<u64> {
    EnclosingQuote(
        c,
        &c.QueryPosition(start, StructuralPositionQuery::Forward),
        true,
    )
}
fn IsVisibleEdge<C: StructuralEditingContext>(
    c: &C,
    visible: &Position<u64>,
    node: u64,
    first: bool,
) -> bool {
    if visible.IsNull()
        || !visible
            .ComputeContainerNode(c)
            .is_some_and(|container| IsDescendantOf(c, &container, &node))
    {
        return false;
    }
    let adjacent = c.QueryPosition(
        visible,
        if first {
            StructuralPositionQuery::Previous
        } else {
            StructuralPositionQuery::Next
        },
    );
    adjacent.IsNull()
        || !adjacent
            .AnchorNode()
            .is_some_and(|anchor| IsDescendantOf(c, anchor, &node))
}
fn InsertAfter<C: StructuralEditingContext>(c: &C, node: u64, after: u64) -> EditResult {
    if let Some(next) = c.NextSibling(&after) {
        c.InsertBefore(node, next)
    } else {
        c.Append(node, c.Parent(&after).ok_or(EditingAborted)?)
    }
}
fn ParentElement<C: StructuralEditingContext>(c: &C, node: u64) -> Option<u64> {
    let mut parent = c.Parent(&node);
    while let Some(current) = parent {
        if c.IsElement(current) {
            return Some(current);
        }
        parent = c.Parent(&current);
    }
    None
}
pub struct BreakBlockquoteCommand;
impl BreakBlockquoteCommand {
    pub fn SelectionStartsInTableStructure<C: BreakBlockquoteContext>(
        c: &C,
        selection: (Position<u64>, Position<u64>),
    ) -> bool {
        let start = if c.Compare(&selection.0, &selection.1) == Some(std::cmp::Ordering::Greater) {
            selection.1
        } else {
            selection.0
        };
        let root = start.AnchorNode().and_then(|node| c.RootEditable(*node));
        let mut node = start.AnchorNode().copied();
        while let Some(current) = node {
            if (root.is_none() || c.Editable(current)) && c.NodeFacts(current).table_structure {
                return true;
            }
            if Some(current) == root {
                break;
            }
            node = c.Parent(&current);
        }
        false
    }
    /// TypingCommand's quoted-newline entry, typing_command.cc:837-852;
    /// BreakBlockquoteCommand::DoApply at :106-139.
    pub fn InsertParagraphSeparatorInQuotedContent<C: BreakBlockquoteContext>(
        c: &C,
        selection: (Position<u64>, Position<u64>),
    ) -> EditResult<Option<Position<u64>>> {
        let (start, end) =
            if c.Compare(&selection.0, &selection.1) == Some(std::cmp::Ordering::Greater) {
                (selection.1, selection.0)
            } else {
                selection
            };
        let visible = c.QueryPosition(&start, StructuralPositionQuery::Visible);
        if TopBlockquoteOf(c, &visible).is_none() {
            return Ok(None);
        }
        let caret = if start != end {
            super::delete_selection_command::DeleteSelectionCommand::ApplyWithTypingOptions(
                c,
                (start, end),
                false,
                true,
            )?
        } else {
            start
        };
        Self::ApplyAtCaret(c, &caret).map(Some)
    }
    /// Source :96-106,129-349. The calling CompositeEditCommand has already
    /// deleted its selection and routes this same ending caret into the owner.
    pub fn ApplyAtCaret<C: BreakBlockquoteContext>(
        c: &C,
        start: &Position<u64>,
    ) -> EditResult<Position<u64>> {
        let Some(top) = TopBlockquoteOf(c, start).filter(|n| c.Parent(n).is_some()) else {
            return Ok(start.clone());
        };
        let visible = c.QueryPosition(start, StructuralPositionQuery::Visible);
        let mut position = c.QueryPosition(start, StructuralPositionQuery::Forward);
        let br = c.CreateElement("br")?;
        let last = IsVisibleEdge(c, &visible, top, false);
        if IsVisibleEdge(c, &visible, top, true) && !last {
            c.InsertBefore(br, top)?;
            return Self::Finish(c, br);
        }
        InsertAfter(c, br, top)?;
        c.UpdateStyleAndLayout();
        if last {
            return Self::Finish(c, br);
        }
        if c.PositionBoolean(&visible, None, StructuralBooleanQuery::LineBreakExists) {
            position = c.MoveByGrapheme(&position, true);
        }
        while EnclosingQuote(c, &position, false).is_some_and(|quote| {
            IsVisibleEdge(
                c,
                &c.QueryPosition(&position, StructuralPositionQuery::Visible),
                quote,
                true,
            )
        }) {
            position = c.MoveByGrapheme(&position, false);
        }
        let mut first = *position.AnchorNode().ok_or(EditingAborted)?;
        if c.IsText(&first) {
            let offset = position.ComputeOffsetInContainerNode(c);
            if offset >= c.CharacterDataLength(&first).ok_or(EditingAborted)? {
                first = NextNode(c, &first, None).ok_or(EditingAborted)?;
            } else if offset > 0 {
                c.SplitText(first, offset)?;
            }
        } else if position.ComputeEditingOffset(c) > 0 {
            first = c
                .ChildAt(&first, position.ComputeEditingOffset(c))
                .or_else(|| NextNode(c, &first, None))
                .ok_or(EditingAborted)?;
        }
        if !IsDescendantOf(c, &first, &top) {
            let ending = Position::FirstPositionInOrBeforeNode(c, &first);
            c.SetEndingPosition(&ending);
            return Ok(ending);
        }
        let mut ancestors = Vec::new();
        let mut ancestor = ParentElement(c, first);
        while let Some(node) = ancestor.filter(|n| *n != top) {
            ancestors.push(node);
            ancestor = ParentElement(c, node);
        }
        let clone = c.CloneWithoutChildren(top)?;
        InsertAfter(c, clone, br)?;
        let mut cloned_ancestor = clone;
        for index in (0..ancestors.len()).rev() {
            let child = c.CloneWithoutChildren(ancestors[index])?;
            if c.Tag(child, "ol") {
                let mut list_child = Some(if index > 0 {
                    ancestors[index - 1]
                } else {
                    first
                });
                while let Some(node) = list_child.filter(|n| !c.Tag(*n, "li")) {
                    list_child = c.NextSibling(&node);
                }
                if let Some(number) = list_child.and_then(|n| c.ListItemNumber(n)) {
                    c.SetListStart(child, number)?;
                }
            }
            c.Append(child, cloned_ancestor)?;
            cloned_ancestor = child;
        }
        super::composite_edit_command::MoveRemainingSiblings(
            c,
            Some(first),
            None,
            cloned_ancestor,
        )?;
        if let Some(original_parent) = ancestors.first().copied() {
            let mut ancestor = Some(original_parent);
            let mut cloned_parent = ParentElement(c, cloned_ancestor);
            while let Some(current) = ancestor.filter(|n| *n != top) {
                let parent = cloned_parent.ok_or(EditingAborted)?;
                super::composite_edit_command::MoveRemainingSiblings(
                    c,
                    c.NextSibling(&current),
                    None,
                    parent,
                )?;
                ancestor = ParentElement(c, current);
                cloned_parent = ParentElement(c, parent);
            }
            if c.ChildCount(&original_parent) == 0 {
                c.Remove(original_parent)?;
            }
        }
        c.UpdateStyleAndLayout();
        let facts = c.BlockPlaceholderFacts(clone);
        if facts.block_flow && (facts.height_zero || facts.empty_list_item) {
            let placeholder = c.CreateElement("br")?;
            c.Append(placeholder, clone)?;
        }
        Self::Finish(c, br)
    }
    fn Finish<C: BreakBlockquoteContext>(c: &C, br: u64) -> EditResult<Position<u64>> {
        let ending = Position::BeforeNode(br);
        c.SetEndingPosition(&ending);
        let visible = c.QueryPosition(&ending, StructuralPositionQuery::Visible);
        c.RebalanceBreakWhitespace(&visible)?;
        Ok(ending)
    }
}
