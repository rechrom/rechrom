// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source delete_selection_command.cc:219-249,1037-1052.
use super::{
    editing_commands_utilities::IsSpace,
    paragraph_editing_context::EditResult,
    structural_editing_context::{
        StructuralBooleanQuery, StructuralEditingContext, StructuralPositionQuery,
    },
};
use crate::core::editing::visible_units_paragraph::IsParagraphEdge;
use crate::core::editing::{position::Position, text_offset_mapping};

pub(super) fn Trailing<C: StructuralEditingContext>(
    ctx: &C,
    position: &Position<u64>,
    non_collapsible: bool,
) -> Position<u64> {
    if position.IsNull() {
        return Position::default();
    }
    let visible = ctx.QueryPosition(position, StructuralPositionQuery::Visible);
    let forward = ctx.QueryPosition(&visible, StructuralPositionQuery::Forward);
    let is_space = forward
        .AnchorNode()
        .filter(|node| ctx.IsText(node))
        .and_then(|node| ctx.TextCodeUnit(*node, forward.ComputeOffsetInContainerNode(ctx)))
        .is_some_and(|unit| IsSpace(unit, non_collapsible));
    if is_space
        && !IsParagraphEdge(ctx, &visible, false)
        && !ctx
            .QueryPosition(&visible, StructuralPositionQuery::NextWithinEditable)
            .IsNull()
    {
        position.clone()
    } else {
        Position::default()
    }
}
pub(super) fn Fixup<C: StructuralEditingContext>(ctx: &C, position: &Position<u64>) -> EditResult {
    let Some(node) = position
        .AnchorNode()
        .copied()
        .filter(|node| ctx.IsText(node) && ctx.IsConnected(node))
    else {
        return Ok(());
    };
    ctx.UpdateStyleAndLayout();
    if !ctx.PositionBoolean(position, None, StructuralBooleanQuery::RenderedCharacter) {
        ctx.ReplaceText(node, position.ComputeOffsetInContainerNode(ctx), 1, &[0xa0])?;
    }
    Ok(())
}

// Source delete_selection_command.h:39-110; .cc:113-188,252-1418.
use super::{
    composite_edit_command::{
        AppendPlaceholder, InsertNodeAt, MoveParagraph, RemovePreservingChildren,
    },
    editing_commands_utilities::{MailQuoteLevel, OutsideSpecialElement},
    paragraph_editing_context::EditingAborted,
};
use crate::core::editing::{
    commands::editing_commands_utilities::IsBlockEdge,
    commands::structural_editing_support::{
        Ancestor, BeforeOrEqual, ChildIndex, Contains, ParentAfter, ParentBefore,
    },
    editing_utilities::IsMailQuote,
};
use std::cmp::Ordering;

pub struct DeleteSelectionCommand;
impl DeleteSelectionCommand {
    pub fn Apply<C: StructuralEditingContext>(
        ctx: &C,
        selection: (Position<u64>, Position<u64>),
    ) -> EditResult<Position<u64>> {
        Self::ApplyWithSmartDelete(ctx, selection, false)
    }
    pub fn ApplyWithSmartDelete<C: StructuralEditingContext>(
        ctx: &C,
        selection: (Position<u64>, Position<u64>),
        smart: bool,
    ) -> EditResult<Position<u64>> {
        Self::ApplyWithOptions(ctx, selection, smart, true)
    }
    pub(super) fn ApplyWithOptions<C: StructuralEditingContext>(
        ctx: &C,
        selection: (Position<u64>, Position<u64>),
        smart: bool,
        merge_blocks: bool,
    ) -> EditResult<Position<u64>> {
        Self::ApplyWithExpansion(ctx, selection, smart, merge_blocks, false)
    }
    pub fn ApplyWithTypingOptions<C: StructuralEditingContext>(
        ctx: &C,
        selection: (Position<u64>, Position<u64>),
        smart: bool,
        expand_for_special: bool,
    ) -> EditResult<Position<u64>> {
        Self::ApplyWithExpansion(ctx, selection, smart, true, expand_for_special)
    }
    fn ApplyWithExpansion<C: StructuralEditingContext>(
        ctx: &C,
        selection: (Position<u64>, Position<u64>),
        smart: bool,
        merge_blocks: bool,
        expand_for_special: bool,
    ) -> EditResult<Position<u64>> {
        let (mut start, mut end) = selection;
        if start == end {
            return Ok(start);
        }
        ctx.UpdateStyleAndLayout();
        if ctx.Compare(&start, &end) == Some(Ordering::Greater) {
            std::mem::swap(&mut start, &mut end);
        }
        (start, end) = InitializeDeletionStartEnd(ctx, start, end, expand_for_special);
        let style_start = start.clone();
        let style_end = end.clone();
        // delete_selection_command.cc:252-400,1338-1418. Retain the four
        // caret equivalents separately; BR deletion depends on their anchor kind.
        let start_root = ctx
            .RootEditable(start.ComputeContainerNode(ctx).ok_or(EditingAborted)?)
            .ok_or(EditingAborted)?;
        let end_root = ctx.RootEditable(end.ComputeContainerNode(ctx).ok_or(EditingAborted)?);
        let mut upstream_start = ctx.QueryPosition(&start, StructuralPositionQuery::Backward);
        let mut downstream_start = ctx.QueryPosition(&start, StructuralPositionQuery::Forward);
        let mut upstream_end = ctx.QueryPosition(&end, StructuralPositionQuery::Backward);
        let mut downstream_end = ctx.QueryPosition(&end, StructuralPositionQuery::Forward);
        let downstream_container = downstream_end.ComputeContainerNode(ctx);
        let root_stays_open = downstream_container == end_root
            || end_root.is_some_and(|root| ctx.NodeFacts(root).inline);
        let visible_start = ctx.QueryPosition(&start, StructuralPositionQuery::Visible);
        let visible_end = ctx.QueryPosition(&end, StructuralPositionQuery::VisibleUpstream);
        let line_break_end =
            ctx.PositionBoolean(&visible_end, None, StructuralBooleanQuery::LineBreakExists);
        let mut need_placeholder = !root_stays_open
            && IsParagraphEdge(ctx, &visible_start, true)
            && IsParagraphEdge(ctx, &visible_end, false)
            && !line_break_end;
        if need_placeholder {
            let after = ctx
                .QueryPosition(&visible_start, StructuralPositionQuery::Forward)
                .ComputeNodeAfterPosition(ctx);
            if after.is_some_and(|table| {
                ctx.Tag(table, "table")
                    && end
                        .AnchorNode()
                        .is_some_and(|node| Contains(ctx, table, *node))
            }) {
                need_placeholder = false;
            }
        }
        let start_row = Ancestor(ctx, &start, |node| ctx.Tag(node, "tr"));
        let end_row = Ancestor(ctx, &end, |node| ctx.Tag(node, "tr"));
        let start_cell = Ancestor(ctx, &start, |node| ctx.NodeFacts(node).table_cell);
        let end_cell = Ancestor(ctx, &end, |node| ctx.NodeFacts(node).table_cell);
        let mut merge = merge_blocks
            && end_root == Some(start_root)
            && (end_cell.is_none() || start_cell == end_cell);
        let mut ending_position = if merge && !IsParagraphEdge(ctx, &downstream_end, false) {
            downstream_end.clone()
        } else {
            downstream_start.clone()
        };
        let prune_start = MailQuoteLevel(ctx, &style_start) != MailQuoteLevel(ctx, &style_end)
            && IsParagraphEdge(ctx, &style_start, true)
            && IsParagraphEdge(ctx, &style_end, true);
        if prune_start {
            merge = false;
        }
        let mut leading = super::editing_commands_utilities::LeadingCollapsibleWhitespacePosition(
            ctx,
            &upstream_start,
            false,
        );
        let mut trailing = if downstream_end
            .AnchorNode()
            .is_some_and(|node| ctx.Editable(*node))
        {
            super::delete_selection_command::Trailing(ctx, &downstream_end, false)
        } else {
            Position::default()
        };
        if smart {
            let visible = ctx.QueryPosition(&upstream_start, StructuralPositionQuery::Visible);
            let skip = !super::delete_selection_command::Trailing(ctx, &visible, true).IsNull()
                || !super::editing_commands_utilities::LeadingCollapsibleWhitespacePosition(
                    ctx,
                    &downstream_end,
                    true,
                )
                .IsNull();
            let has_leading =
                !super::editing_commands_utilities::LeadingCollapsibleWhitespacePosition(
                    ctx,
                    &upstream_start,
                    true,
                )
                .IsNull();
            if !skip && has_leading {
                let previous =
                    ctx.QueryPosition(&upstream_start, StructuralPositionQuery::Previous);
                upstream_start = ctx.QueryPosition(&previous, StructuralPositionQuery::Backward);
                downstream_start = ctx.QueryPosition(&previous, StructuralPositionQuery::Forward);
                leading = super::editing_commands_utilities::LeadingCollapsibleWhitespacePosition(
                    ctx,
                    &upstream_start,
                    false,
                );
                ctx.SetStartingSelection(&upstream_start, &upstream_end);
            }
            if !skip
                && !has_leading
                && !super::delete_selection_command::Trailing(ctx, &downstream_end, true).IsNull()
            {
                let next = ctx.QueryPosition(&downstream_end, StructuralPositionQuery::Next);
                upstream_end = ctx.QueryPosition(&next, StructuralPositionQuery::Backward);
                downstream_end = ctx.QueryPosition(&next, StructuralPositionQuery::Forward);
                trailing = super::delete_selection_command::Trailing(ctx, &downstream_end, false);
                ctx.SetStartingSelection(&downstream_start, &downstream_end);
            }
        }
        let start_block = ctx.EnclosingBlock(&downstream_start.ParentAnchoredEquivalent(ctx));
        let end_block = ctx.EnclosingBlock(&upstream_end.ParentAnchoredEquivalent(ctx));
        let line_break_before = ctx.PositionBoolean(
            &ctx.QueryPosition(&upstream_start, StructuralPositionQuery::Previous),
            None,
            StructuralBooleanQuery::LineBreakExists,
        );
        let before_br = upstream_start.ComputeNodeAfterPosition(ctx);
        let after_br = downstream_start.ComputeNodeAfterPosition(ctx);
        let both_br = before_br.is_some_and(|node| ctx.Tag(node, "br"))
            && after_br.is_some_and(|node| ctx.Tag(node, "br"));
        let only_br = both_br && after_br == upstream_end.ComputeNodeAfterPosition(ctx);
        let starts_empty_line = both_br
            && !before_br.is_some_and(|node| {
                IsBlockEdge(ctx, &Position::BeforeNode(node), true)
                    && IsBlockEdge(ctx, &Position::AfterNode(node), false)
            });
        if starts_empty_line && !only_br {
            ending_position = downstream_end.clone();
        }
        let tracked_ending = ctx.Track(&ending_position)?;
        let tracked_leading = if leading.IsNull() {
            None
        } else {
            Some(ctx.Track(&leading)?)
        };
        let tracked_trailing = if trailing.IsNull() {
            None
        } else {
            Some(ctx.Track(&trailing)?)
        };
        let tracked_upstream = ctx.Track(&upstream_start)?;
        let tracked_downstream = ctx.Track(&downstream_end)?;
        super::composite_edit_command::DeleteInsignificantDownstream(ctx, &trailing)?;
        let (typing_style, quote_style) = Self::SaveTypingStyle(
            ctx,
            &style_start,
            &style_end,
            &upstream_start,
            &downstream_end,
        )?;
        // delete_selection_command.cc:534-577: deleting a standalone BR must
        // never replace it with another placeholder BR.
        if only_br {
            ctx.Remove(after_br.ok_or(EditingAborted)?)?;
            let ending = ctx.Tracked(&tracked_ending);
            Self::CalculateTypingStyle(ctx, &ending, typing_style, quote_style)?;
            ctx.SetEndingPosition(&ending);
            super::composite_edit_command::RebalanceWhitespaceAt(ctx, &ending)?;
            return Ok(ending);
        }
        start = ctx.Tracked(&tracked_upstream).ToOffsetInAnchor(ctx);
        end = ctx.Tracked(&tracked_downstream).ToOffsetInAnchor(ctx);
        let tracked_end = ctx.Track(&end)?;
        let start_node = start.ComputeContainerNode(ctx).ok_or(EditingAborted)?;
        let end_node = end.ComputeContainerNode(ctx).ok_or(EditingAborted)?;
        let start_offset = start.ComputeOffsetInContainerNode(ctx);
        let end_offset = end.ComputeOffsetInContainerNode(ctx);
        // delete_selection_command.cc:854-1035, HandleGeneralDelete.
        if start_node == end_node {
            if ctx.IsText(&start_node) {
                ctx.DeleteText(
                    start_node,
                    start_offset,
                    end_offset.saturating_sub(start_offset),
                )?;
            } else {
                let children: Vec<_> = (start_offset..end_offset)
                    .filter_map(|index| ctx.ChildAt(&start_node, index))
                    .collect();
                for node in children {
                    Self::RemoveSelectedNode(ctx, node, start_root)?;
                }
            }
        } else {
            let mut node = if ctx.IsText(&start_node) {
                if start_offset > 0 {
                    let len = ctx.CharacterDataLength(&start_node).ok_or(EditingAborted)?;
                    ctx.DeleteText(start_node, start_offset, len.saturating_sub(start_offset))?;
                    text_offset_mapping::NextNode(ctx, &start_node, Some(&start_root))
                } else {
                    Some(start_node)
                }
            } else {
                ctx.ChildAt(&start_node, start_offset).or_else(|| {
                    text_offset_mapping::NextSkippingChildren(ctx, &start_node, Some(&start_root))
                })
            };
            let mut removed = Vec::new();
            while let Some(current) = node.filter(|current| *current != end_node) {
                if !BeforeOrEqual(
                    ctx,
                    &Position::FirstPositionInOrBeforeNode(ctx, &current),
                    &end,
                ) {
                    break;
                }
                if !Contains(ctx, current, end_node) {
                    removed.push(current);
                    node =
                        text_offset_mapping::NextSkippingChildren(ctx, &current, Some(&start_root));
                } else {
                    node = text_offset_mapping::NextNode(ctx, &current, Some(&start_root));
                }
            }
            for node in removed {
                Self::RemoveSelectedNode(ctx, node, start_root)?;
            }
            if ctx.IsConnected(&end_node) {
                if ctx.IsText(&end_node) {
                    let current = ctx.Tracked(&tracked_end).ComputeOffsetInContainerNode(ctx);
                    ctx.DeleteText(end_node, 0, current)?;
                } else {
                    let current_end = ctx.Tracked(&tracked_end);
                    let first = if Contains(ctx, end_node, start_node) {
                        let mut child = start_node;
                        while let Some(parent) =
                            ctx.Parent(&child).filter(|parent| *parent != end_node)
                        {
                            child = parent;
                        }
                        ChildIndex(ctx, child) + 1
                    } else {
                        0
                    };
                    let children: Vec<_> = (first..current_end.ComputeOffsetInContainerNode(ctx))
                        .filter_map(|index| ctx.ChildAt(&end_node, index))
                        .collect();
                    for node in children {
                        Self::RemoveSelectedNode(ctx, node, start_root)?;
                    }
                }
            }
        }
        let mut ending = ctx.Tracked(&tracked_ending);
        if let Some(leading) = &tracked_leading {
            super::delete_selection_command::Fixup(ctx, &ctx.Tracked(leading))?;
        }
        if let Some(trailing) = &tracked_trailing {
            super::delete_selection_command::Fixup(ctx, &ctx.Tracked(trailing))?;
        }
        if prune_start {
            if let Some(block) =
                start_block.filter(|block| *block != start_root && ctx.ChildCount(block) == 0)
            {
                ctx.Remove(block)?;
                ending = ctx.Tracked(&tracked_ending);
            }
            need_placeholder = false;
        }
        // delete_selection_command.cc:1054-1210. Table-cell boundaries and
        // non-inline table/image/HR paragraphs do not participate in merging.
        if merge && start_block != end_block {
            ctx.UpdateStyleAndLayout();
            let mut origin = ctx.Tracked(&tracked_downstream);
            let mut destination = ctx.Tracked(&tracked_upstream);
            let visible_origin = ctx.QueryPosition(&origin, StructuralPositionQuery::Visible);
            let visible_destination =
                ctx.QueryPosition(&destination, StructuralPositionQuery::Visible);
            if starts_empty_line && visible_origin != visible_destination
                || visible_destination.IsNull()
            {
                let br = ctx.CreateElement("br")?;
                InsertNodeAt(ctx, br, &destination)?;
                ctx.UpdateStyleAndLayout();
                destination = ctx.Tracked(&tracked_upstream);
                origin = ctx.Tracked(&tracked_downstream);
            }
            if let (Some(destination_block), Some(source_block)) = (
                ctx.EnclosingBlock(&destination),
                ctx.EnclosingBlock(&origin),
            ) {
                if destination_block != source_block
                    && ctx.IsConnected(&source_block)
                    && !(ctx.NodeFacts(source_block).atomic_block
                        && !IsParagraphEdge(ctx, &destination, true))
                {
                    let ending_destination = ctx.Track(&destination)?;
                    // Source moves the paragraph beginning at downstream_end_,
                    // not every later paragraph nested inside its block.
                    let paragraph_end =
                        ctx.QueryPosition(&origin, StructuralPositionQuery::ParagraphEnd);
                    MoveParagraph(ctx, &origin, &paragraph_end, &destination, source_block)?;
                    ending = ctx.Tracked(&ending_destination);
                }
            }
        }
        let ending_tracker = ctx.Track(&ending)?;
        Self::RemoveEmptyTableRows(ctx, start_row, end_row, &ending)?;
        ending = ctx.Tracked(&ending_tracker);
        ctx.UpdateStyleAndLayout();
        if !need_placeholder && root_stays_open {
            let visible = ctx.QueryPosition(&ending, StructuralPositionQuery::Visible);
            let has_placeholder =
                ctx.PositionBoolean(&visible, None, StructuralBooleanQuery::LineBreakExists)
                    && ctx
                        .QueryPosition(&visible, StructuralPositionQuery::NextWithinEditable)
                        .IsNull();
            need_placeholder = has_placeholder && line_break_before && !line_break_end;
        }
        if need_placeholder {
            ending = Self::RemoveRedundantBlocks(ctx, &ending)?;
            if ending
                .AnchorNode()
                .is_some_and(|node| ctx.IsConnected(node))
            {
                let br = ctx.CreateElement("br")?;
                InsertNodeAt(ctx, br, &ending)?;
            }
        }
        super::composite_edit_command::RebalanceWhitespaceAt(ctx, &ending)?;
        Self::CalculateTypingStyle(ctx, &ending, typing_style, quote_style)?;
        ctx.UpdateStyleAndLayout();
        ending = ctx.QueryPosition(&ending, StructuralPositionQuery::Visible);
        ctx.SetEndingPosition(&ending);
        Ok(ending)
    }
    fn SaveTypingStyle<C: StructuralEditingContext>(
        ctx: &C,
        start: &Position<u64>,
        end: &Position<u64>,
        upstream_start: &Position<u64>,
        downstream_end: &Position<u64>,
    ) -> EditResult<(Option<C::Style>, Option<C::Style>)> {
        // delete_selection_command.cc:417-532. Preserve the first original
        // tag for each property/value while walking only the editable ancestry.
        let mut node = start.AnchorNode().copied();
        let mut equivalent = false;
        while let Some(current) = node {
            if ["strong", "b", "em", "i", "u", "s", "strike", "sub", "sup"]
                .iter()
                .any(|tag| ctx.Tag(current, tag))
            {
                equivalent = true;
                break;
            }
            if !ctx.Editable(current) {
                break;
            }
            node = ctx.Parent(&current);
        }
        let same_text = upstream_start.AnchorNode() == downstream_end.AnchorNode()
            && upstream_start
                .AnchorNode()
                .is_some_and(|node| ctx.IsText(node));
        if same_text && !equivalent
            || start
                .AnchorNode()
                .is_some_and(|node| ctx.EditingIgnoresContent(node))
        {
            return Ok((None, None));
        }
        let mut style = ctx.CaptureStyle(start, true)?;
        if let Some(anchor) = Ancestor(ctx, start, |node| ctx.Tag(node, "a")) {
            style = ctx.RemoveStyleAddedByElement(&style, anchor)?;
        }
        let quote = if Ancestor(ctx, start, |node| IsMailQuote(ctx, node)).is_some() {
            Some(ctx.CaptureStyle(end, false)?)
        } else {
            None
        };
        Ok((Some(style), quote))
    }
    fn CalculateTypingStyle<C: StructuralEditingContext>(
        ctx: &C,
        ending: &Position<u64>,
        mut typing_style: Option<C::Style>,
        quote_style: Option<C::Style>,
    ) -> EditResult {
        // delete_selection_command.cc:1266-1299. A deletion leaving a cite
        // blockquote takes the alternate end style; PrepareToApplyAt removes
        // redundant properties before publishing the same Editor style owner.
        if quote_style.is_some() && Ancestor(ctx, ending, |node| IsMailQuote(ctx, node)).is_none() {
            typing_style = quote_style;
        }
        let prepared = match typing_style {
            Some(style) if !ending.IsNull() => ctx.PrepareStyle(&style, ending)?,
            Some(style) => Some(style),
            None => None,
        };
        ctx.SetTypingStyle(prepared);
        Ok(())
    }
    fn IsTableRowEmpty<C: StructuralEditingContext>(ctx: &C, row: u64) -> bool {
        if !ctx.Tag(row, "tr") {
            return false;
        }
        ctx.UpdateStyleAndLayout();
        (0..ctx.ChildCount(&row))
            .filter_map(|index| ctx.ChildAt(&row, index))
            .all(|cell| {
                !ctx.NodeFacts(cell).table_cell
                    || ctx.QueryPosition(
                        &Position::FirstPositionInNode(cell),
                        StructuralPositionQuery::Visible,
                    ) == ctx.QueryPosition(
                        &Position::LastPositionInNode(ctx, &cell),
                        StructuralPositionQuery::Visible,
                    )
            })
    }
    fn RemoveEmptyTableRows<C: StructuralEditingContext>(
        ctx: &C,
        start: Option<u64>,
        end: Option<u64>,
        ending: &Position<u64>,
    ) -> EditResult {
        // delete_selection_command.cc:1212-1263. Raw CompositeEditCommand
        // removals deliberately bypass the table-preserving deletion leaf.
        if let Some(end) = end.filter(|end| ctx.IsConnected(end) && Some(*end) != start) {
            let mut row = ctx.PreviousSibling(&end);
            while let Some(current) = row.filter(|row| Some(*row) != start) {
                row = ctx.PreviousSibling(&current);
                if Self::IsTableRowEmpty(ctx, current) {
                    ctx.Remove(current)?;
                }
            }
        }
        if let Some(start) = start.filter(|start| ctx.IsConnected(start) && Some(*start) != end) {
            let mut row = ctx.NextSibling(&start);
            while let Some(current) = row.filter(|row| Some(*row) != end) {
                row = ctx.NextSibling(&current);
                if Self::IsTableRowEmpty(ctx, current) {
                    ctx.Remove(current)?;
                }
            }
        }
        if let Some(end) = end.filter(|end| ctx.IsConnected(end) && Some(*end) != start) {
            if Self::IsTableRowEmpty(ctx, end)
                && !ending
                    .AnchorNode()
                    .is_some_and(|node| Contains(ctx, end, *node))
            {
                ctx.Remove(end)?;
            }
        }
        Ok(())
    }
    fn RemoveRedundantBlocks<C: StructuralEditingContext>(
        ctx: &C,
        position: &Position<u64>,
    ) -> EditResult<Position<u64>> {
        // delete_selection_command.cc:1315-1336 and CompositeEditCommand::
        // IsRemovableBlock:311-325 (the parent must have at most one child).
        let mut ending = position.clone();
        let root = ending.AnchorNode().and_then(|node| ctx.RootEditable(*node));
        let mut node = ending.ComputeContainerNode(ctx);
        while let Some(current) = node.filter(|node| Some(*node) != root) {
            let parent = ctx.Parent(&current);
            if ctx.Tag(current, "div")
                && !ctx.HasAttributes(current)
                && parent.is_none_or(|parent| ctx.ChildCount(&parent) <= 1)
            {
                let adjusted = if ending.AnchorNode() == Some(&current) {
                    Some(Position::new(
                        parent.ok_or(EditingAborted)?,
                        ctx.NodeIndex(&current) + ending.ComputeOffsetInContainerNode(ctx),
                    ))
                } else {
                    None
                };
                RemovePreservingChildren(ctx, current)?;
                if let Some(adjusted) = adjusted {
                    ending = adjusted;
                }
                node = ending.AnchorNode().copied();
            } else {
                node = parent;
            }
        }
        Ok(ending)
    }
    fn RemoveSelectedNode<C: StructuralEditingContext>(
        ctx: &C,
        node: u64,
        root: u64,
    ) -> EditResult {
        // delete_selection_command.cc:583-653,655-793. Never remove the
        // editable root or table structure; their selected contents are emptied.
        if node == root || ctx.NodeFacts(node).table_structure {
            let children: Vec<_> = (0..ctx.ChildCount(&node))
                .filter_map(|index| ctx.ChildAt(&node, index))
                .collect();
            for child in children {
                Self::RemoveSelectedNode(ctx, child, root)?;
            }
            if ctx.NodeFacts(node).table_cell && ctx.ChildCount(&node) == 0 {
                AppendPlaceholder(ctx, node)?;
            }
            Ok(())
        } else if ctx.Parent(&node).is_some_and(|parent| ctx.Editable(parent)) {
            ctx.Remove(node)
        } else {
            Ok(())
        }
    }
}

fn InitializeDeletionStartEnd<C: StructuralEditingContext>(
    ctx: &C,
    mut start: Position<u64>,
    mut end: Position<u64>,
    expand: bool,
) -> (Position<u64>, Position<u64>) {
    let original_start = start.clone();
    let original_end = end.clone();
    if start.AnchorNode().is_some_and(|node| ctx.Tag(*node, "hr")) {
        start = Position::BeforeNode(*start.AnchorNode().unwrap());
    } else if end.AnchorNode().is_some_and(|node| ctx.Tag(*node, "hr")) {
        end = Position::AfterNode(*end.AnchorNode().unwrap());
    }
    if !expand {
        return (start, end);
    }
    loop {
        let (s, start_container) = OutsideSpecialElement(ctx, &start, true);
        let (e, end_container) = OutsideSpecialElement(ctx, &end, false);
        if start_container.is_none() && end_container.is_none() {
            break;
        }
        if ctx.QueryPosition(&start, StructuralPositionQuery::Visible)
            != ctx.QueryPosition(&original_start, StructuralPositionQuery::Visible)
            || ctx.QueryPosition(&end, StructuralPositionQuery::Visible)
                != ctx.QueryPosition(&original_end, StructuralPositionQuery::Visible)
        {
            break;
        }
        if let Some(container) = start_container.filter(|_| end_container.is_none()) {
            if ctx
                .Compare(&ParentAfter(ctx, container), &end)
                .is_some_and(|order| order != Ordering::Less)
            {
                break;
            }
        }
        if let Some(container) = end_container.filter(|_| start_container.is_none()) {
            if ctx
                .Compare(&start, &ParentBefore(ctx, container))
                .is_some_and(|order| order != Ordering::Less)
            {
                break;
            }
        }
        if start_container
            .zip(end_container)
            .is_some_and(|(start, end)| start != end && Contains(ctx, end, start))
        {
            start = s;
        } else if start_container
            .zip(end_container)
            .is_some_and(|(start, end)| start != end && Contains(ctx, start, end))
        {
            end = e;
        } else {
            start = s;
            end = e;
        }
    }
    (start, end)
}
