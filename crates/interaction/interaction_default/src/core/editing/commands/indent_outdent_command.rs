// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! indent_outdent_command.h:34-71; indent_outdent_command.cc:66-608.
use super::{
    apply_block_element_command,
    composite_edit_command::{self, ParagraphMoveHost},
    insert_list_command::{
        self, CanMergeLists, ElementSibling, EnclosingList, EnclosingListChild, EnclosingTableCell,
        HighestEnclosingNodeOfType, InsertListCommand, ListEditingContext, ListType,
    },
    paragraph_cloning::{self, AtEdge, Contains, InsertAfter},
    paragraph_editing_context::{EditResult, EditingAborted},
    structural_editing_context::StructuralPositionQuery as Q,
};
use crate::core::editing::position::Position;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndentType {
    Indent,
    Outdent,
}
pub struct IndentOutdentCommand {
    pub action: IndentType,
}

struct IndentFormatter<'a, H> {
    command: &'a IndentOutdentCommand,
    host: &'a H,
}
impl<C: ListEditingContext, H: ParagraphMoveHost>
    apply_block_element_command::BlockElementFormatter<C> for IndentFormatter<'_, H>
{
    fn CreateBlockElement(&self, c: &C) -> EditResult<u64> {
        IndentOutdentCommand::CreateBlockElement(c)
    }
    fn FormatSelection(&self, c: &C, start: &Position<u64>, end: &Position<u64>) -> EditResult {
        if self.command.action == IndentType::Outdent {
            IndentOutdentCommand::OutdentRegion(c, self.host, start, end)
        } else {
            apply_block_element_command::FormatSelection(c, self, start, end)
        }
    }
    fn FormatRange(
        &self,
        c: &C,
        from: &Position<u64>,
        to: &Position<u64>,
        _: &Position<u64>,
        block: &mut Option<u64>,
        next: &mut Option<C::Tracked>,
    ) -> EditResult {
        if IndentOutdentCommand::TryIndentingAsListItem(c, from, to, next)? {
            *block = None;
            Ok(())
        } else {
            IndentOutdentCommand::IndentIntoBlockquote(c, from, to, block)
        }
    }
}
impl IndentOutdentCommand {
    pub fn new(action: IndentType) -> Self {
        Self { action }
    }
    pub fn PreservesTypingStyle(&self) -> bool {
        true
    }
    pub fn GetInputType(&self) -> interaction_dispatch::core::events::input_event::InputType {
        use interaction_dispatch::core::events::input_event::InputType as I;
        match self.action {
            IndentType::Indent => I::kFormatIndent,
            IndentType::Outdent => I::kFormatOutdent,
        }
    }
    /// Shared ApplyBlockElementCommand preparation and paragraph iteration.
    pub fn Apply<C: ListEditingContext>(
        &self,
        c: &C,
        host: &impl ParagraphMoveHost,
        start: Position<u64>,
        end: Position<u64>,
    ) -> EditResult {
        apply_block_element_command::ApplyBlockElement(
            c,
            &IndentFormatter {
                command: self,
                host,
            },
            start,
            end,
        )
    }
    /// indent_outdent_command.cc:486-575. An entire list child may move in one
    /// command, so disconnected next-paragraph anchors are recovered from the
    /// command's ending selection before advancing.
    fn OutdentRegion<C: ListEditingContext>(
        c: &C,
        host: &impl ParagraphMoveHost,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> EditResult {
        let mut current = c.QueryPosition(start, Q::ParagraphEnd);
        let last = c.QueryPosition(end, Q::ParagraphEnd);
        if current == last {
            return Self::OutdentParagraph(c, host, start);
        }
        let original_end = c.EndingRange().1;
        let after = c.QueryPosition(&c.QueryPosition(&last, Q::Next), Q::ParagraphEnd);
        while !current.IsNull() && current != after {
            let mut next = c.QueryPosition(&c.QueryPosition(&current, Q::Next), Q::ParagraphEnd);
            let caret = if current == last {
                original_end.clone()
            } else {
                current.clone()
            };
            c.SetEndingRange(&caret, &caret);
            Self::OutdentParagraph(c, host, &caret)?;
            if !after.IsNull() && !after.AnchorNode().is_some_and(|n| c.IsConnected(n)) {
                break;
            }
            c.UpdateStyleAndLayout();
            if !next.IsNull() && !next.AnchorNode().is_some_and(|n| c.IsConnected(n)) {
                let ending = c.EndingRange().0;
                if let Some(child) = ending
                    .AnchorNode()
                    .and_then(|n| EnclosingListChild(c, *n))
                    .filter(|n| c.Tag(*n, "li"))
                {
                    c.SetEndingPosition(&Position::LastPositionInNode(c, &child));
                }
                let ending = c.QueryPosition(&c.EndingRange().1, Q::Visible);
                next = c.QueryPosition(&c.QueryPosition(&ending, Q::Next), Q::ParagraphEnd);
            }
            if current == next {
                break;
            }
            current = c.QueryPosition(&next, Q::Visible);
        }
        Ok(())
    }
    fn CreateBlockElement<C: ListEditingContext>(c: &C) -> EditResult<u64> {
        let quote = c.CreateElement("blockquote")?;
        c.SetBlockquoteStyle(quote)?;
        Ok(quote)
    }
    /// indent_outdent_command.cc:74-175. The list container is a sibling of
    /// the selected LI, as in Chromium's markup-preserving clone path.
    fn TryIndentingAsListItem<C: ListEditingContext>(
        c: &C,
        start: &Position<u64>,
        end: &Position<u64>,
        next_paragraph: &mut Option<C::Tracked>,
    ) -> EditResult<bool> {
        let Some(list) = start.AnchorNode().and_then(|n| EnclosingList(c, *n)) else {
            return Ok(false);
        };
        let Some(item) = c.EnclosingBlock(start).filter(|n| c.Tag(*n, "li")) else {
            return Ok(false);
        };
        let previous = ElementSibling(c, item, false);
        let next = ElementSibling(c, item, true);
        let new_list = c.CreateElement(if c.Tag(list, "ol") { "ol" } else { "ul" })?;
        c.InsertBefore(new_list, item)?;
        c.UpdateStyleAndLayout();
        let last = c.ChildAt(&item, c.ChildCount(&item).saturating_sub(1));
        let keep = end
            .AnchorNode()
            .is_some_and(|n| *n == item || last.is_some_and(|last| Contains(c, last, *n)));
        let (from, to) = if keep {
            (start.clone(), end.clone())
        } else {
            let first = c.ChildAt(&item, 0).ok_or(EditingAborted)?;
            (
                Position::BeforeNode(first),
                Position::AfterNode(last.ok_or(EditingAborted)?),
            )
        };
        let from = c.QueryPosition(&from, Q::Visible);
        let to = c.QueryPosition(&to, Q::Visible);
        if from.IsNull() || to.IsNull() {
            return Err(EditingAborted);
        }
        if !keep {
            let next = c.QueryPosition(&c.QueryPosition(&to, Q::Next), Q::ParagraphEnd);
            if !next.IsNull() {
                *next_paragraph = Some(c.Track(&next)?);
            }
        }
        paragraph_cloning::MoveParagraphWithClones(c, &from, &to, new_list, item)?;
        if !keep && c.IsConnected(&item) {
            c.Remove(item)?;
        }
        c.UpdateStyleAndLayout();
        if let Some(previous) = previous.filter(|n| CanMergeLists(c, *n, new_list)) {
            c.MergeIdenticalElements(previous, new_list)?;
        }
        c.UpdateStyleAndLayout();
        if let Some(next) = next.filter(|n| CanMergeLists(c, new_list, *n)) {
            c.MergeIdenticalElements(new_list, next)?;
        }
        Ok(true)
    }
    /// :177-277. Split only within the editable root, list block, or table cell
    /// selected by the source command; the blockquote owner is reused in order.
    fn IndentIntoBlockquote<C: ListEditingContext>(
        c: &C,
        start: &Position<u64>,
        end: &Position<u64>,
        target: &mut Option<u64>,
    ) -> EditResult {
        let container = start.ComputeContainerNode(c).ok_or(EditingAborted)?;
        let split_to = EnclosingTableCell(c, start).or_else(|| {
            if EnclosingList(c, container).is_some() {
                c.EnclosingBlock(start)
            } else {
                c.RootEditable(container)
            }
        });
        let Some(split_to) = split_to else {
            return Ok(());
        };
        let outer = if container == split_to {
            container
        } else {
            super::composite_edit_command::SplitTreeToNode(c, container, split_to)?
        };
        c.UpdateStyleAndLayout();
        let highest_inline =
            HighestEnclosingNodeOfType(c, end, Some(outer), |c, n| c.IsInlineElement(n));
        if let Some(inline) = highest_inline {
            let next = c.QueryPosition(&c.QueryPosition(end, Q::Next), Q::Forward);
            if AtEdge(c, &next, Q::ParagraphStart)
                && next.AnchorNode().is_some_and(|n| Contains(c, inline, *n))
            {
                let highest_block =
                    HighestEnclosingNodeOfType(c, &next, Some(inline), |c, n| c.NodeFacts(n).block);
                let highest_inline =
                    HighestEnclosingNodeOfType(c, &next, Some(inline), |c, n| c.IsInlineElement(n));
                let split = highest_block
                    .or(highest_inline)
                    .or_else(|| next.AnchorNode().copied())
                    .ok_or(EditingAborted)?;
                c.SplitElement(inline, split)?;
            }
        }
        c.UpdateStyleAndLayout();
        let mut from = c.QueryPosition(start, Q::Visible);
        let quote = if let Some(quote) = *target {
            quote
        } else {
            let quote = Self::CreateBlockElement(c)?;
            if outer == container {
                if c.Tag(outer, "blockquote") {
                    c.InsertBefore(quote, outer)?;
                } else {
                    super::composite_edit_command::InsertNodeAt(c, quote, start)?;
                }
            } else {
                c.InsertBefore(quote, outer)?;
            }
            *target = Some(quote);
            c.UpdateStyleAndLayout();
            from = c.QueryPosition(&Position::AfterNode(quote), Q::Visible);
            quote
        };
        let end = c.QueryPosition(end, Q::Visible);
        if !from.IsNull() && !end.IsNull() {
            paragraph_cloning::MoveParagraphWithClones(c, &from, &end, quote, outer)?;
        }
        Ok(())
    }
    /// :279-484. List outdent delegates to InsertListCommand. Blockquotes
    /// either lose their wrapper or split around the paragraph being removed.
    fn OutdentParagraph<C: ListEditingContext>(
        c: &C,
        host: &impl ParagraphMoveHost,
        position: &Position<u64>,
    ) -> EditResult {
        let start = c.QueryPosition(position, Q::ParagraphStart);
        let end = c.QueryPosition(&start, Q::ParagraphEnd);
        let mut runner = start.AnchorNode().copied();
        let mut enclosing = None;
        while let Some(node) = runner {
            if insert_list_command::IsList(c, node) || c.Tag(node, "blockquote") {
                enclosing = Some(node);
                break;
            }
            if !c.Editable(node) {
                break;
            }
            runner = c.Parent(&node);
        }
        let Some(enclosing) =
            enclosing.filter(|n| c.Parent(n).is_some_and(|parent| c.Editable(parent)))
        else {
            return Ok(());
        };
        if insert_list_command::IsList(c, enclosing) {
            let list_type = if c.Tag(enclosing, "ol") {
                ListType::Ordered
            } else {
                ListType::Unordered
            };
            let selection = c.EndingRange();
            InsertListCommand::new(list_type).Apply(c, host, selection.0, selection.1)?;
            return Ok(());
        }
        let first = c.QueryPosition(&Position::FirstPositionInNode(enclosing), Q::Visible);
        let block_start = if c.NodeFacts(enclosing).inline {
            first
        } else {
            c.QueryPosition(&first, Q::BlockStart)
        };
        let block_end = c.QueryPosition(&Position::LastPositionInNode(c, &enclosing), Q::BlockEnd);
        let tracked_start = c.Track(&start)?;
        let tracked_end = c.Track(&end)?;
        if start == block_start && end == block_end {
            let first_child = c.ChildAt(&enclosing, 0);
            let last_child = c.ChildAt(&enclosing, c.ChildCount(&enclosing).saturating_sub(1));
            let split_point = c.NextSibling(&enclosing);
            let children: Vec<_> = (0..c.ChildCount(&enclosing))
                .filter_map(|i| c.ChildAt(&enclosing, i))
                .collect();
            for child in children {
                c.Remove(child)?;
                c.InsertBefore(child, enclosing)?;
            }
            c.Remove(enclosing)?;
            if let Some(point) = split_point {
                if let Some(parent) = c.Parent(&point).filter(|n| {
                    c.Tag(*n, "blockquote")
                        && !c.Tag(point, "blockquote")
                        && c.Parent(n).is_some_and(|p| c.Editable(p))
                }) {
                    c.SplitElement(parent, point)?;
                }
            }
            c.UpdateStyleAndLayout();
            let from = first_child
                .map(Position::BeforeNode)
                .unwrap_or_else(|| c.Tracked(&tracked_start));
            if !AtEdge(c, &from, Q::ParagraphStart) {
                let br = c.CreateElement("br")?;
                super::composite_edit_command::InsertNodeAt(c, br, &from)?;
            }
            c.UpdateStyleAndLayout();
            let to = last_child
                .map(Position::AfterNode)
                .unwrap_or_else(|| c.Tracked(&tracked_end));
            if !AtEdge(c, &to, Q::ParagraphEnd) || split_point.is_some_and(|n| c.Tag(n, "br")) {
                let br = c.CreateElement("br")?;
                super::composite_edit_command::InsertNodeAt(c, br, &to)?;
            }
            return Ok(());
        }
        let mut split = enclosing;
        if let Some(block) = c.EnclosingBlock(&start) {
            if block != enclosing {
                let previous = c.QueryPosition(&start, Q::PreviousCandidate);
                let mut previous_node = previous.AnchorNode().copied();
                let mut previous_quote = None;
                while let Some(node) = previous_node {
                    if insert_list_command::IsList(c, node) || c.Tag(node, "blockquote") {
                        previous_quote = Some(node);
                        break;
                    }
                    previous_node = c.Parent(&node);
                }
                let end = c.Parent(&enclosing).unwrap_or(enclosing);
                if previous_quote.is_none() || previous_quote == Some(enclosing) {
                    split = super::composite_edit_command::SplitTreeToNode(c, block, end)?;
                } else if let Some(node) = start.AnchorNode().copied() {
                    super::composite_edit_command::SplitTreeToNode(c, node, end)?;
                }
            } else if let Some(node) = start.AnchorNode().copied() {
                if let Some(previous) = c
                    .PreviousSibling(&enclosing)
                    .filter(|n| c.NodeFacts(*n).inline && !c.Tag(*n, "br"))
                {
                    let br = c.CreateElement("br")?;
                    super::composite_edit_command::InsertNodeAt(
                        c,
                        br,
                        &Position::AfterNode(previous),
                    )?;
                }
                let highest = HighestEnclosingNodeOfType(c, &start, Some(enclosing), |c, n| {
                    c.IsInlineElement(n)
                })
                .unwrap_or(node);
                c.SplitElement(enclosing, highest)?;
            }
        }
        c.UpdateStyleAndLayout();
        let from = c.QueryPosition(&c.Tracked(&tracked_start), Q::ParagraphStart);
        let to = c.QueryPosition(&c.Tracked(&tracked_end), Q::ParagraphEnd);
        if from.IsNull() || to.IsNull() {
            return Ok(());
        }
        let placeholder = c.CreateElement("br")?;
        c.InsertBefore(placeholder, split)?;
        c.UpdateStyleAndLayout();
        let destination = Position::BeforeNode(placeholder);
        composite_edit_command::MoveParagraphs(c, host, &from, &to, &destination, true, None)
    }
}
