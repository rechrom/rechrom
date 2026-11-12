// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! apply_block_element_command.cc:304-497. Shared native paragraph boundary
//! preparation used by IndentOutdentCommand before markup-preserving cloning.
use super::{insert_list_command, paragraph_cloning::AtEdge};
use super::{
    insert_list_command::ListEditingContext,
    paragraph_editing_context::{EditResult, EditingAborted},
    structural_editing_context::StructuralPositionQuery as Q,
};
use crate::core::editing::position::Position;

/// Source apply_block_element_command.cc. Derived commands choose only the
/// block and range action; preparation, iteration and selection restoration
/// retain one shared owner and lifecycle.
pub trait BlockElementFormatter<C: ListEditingContext> {
    fn CreateBlockElement(&self, c: &C) -> EditResult<u64>;
    fn FormatRange(
        &self,
        c: &C,
        start: &Position<u64>,
        end: &Position<u64>,
        end_of_selection: &Position<u64>,
        block: &mut Option<u64>,
        next: &mut Option<C::Tracked>,
    ) -> EditResult;
    fn FormatSelection(&self, c: &C, start: &Position<u64>, end: &Position<u64>) -> EditResult
    where
        Self: Sized,
    {
        FormatSelection(c, self, start, end)
    }
}
pub fn ApplyBlockElement<C: ListEditingContext, F: BlockElementFormatter<C>>(
    c: &C,
    formatter: &F,
    start: Position<u64>,
    end: Position<u64>,
) -> EditResult {
    let mut start = c.QueryPosition(&start, Q::Visible);
    let mut end = c.QueryPosition(&end, Q::Visible);
    let Some(_root) = start
        .ComputeContainerNode(c)
        .and_then(|n| c.RootEditable(n))
    else {
        return Ok(());
    };
    if start.IsNull() || end.IsNull() {
        return Ok(());
    }
    if c.EditingUseDomPositionApiEnabled() {
        let (dom_start, dom_end) = c.EndingDomRange();
        if [&dom_start, &dom_end]
            .iter()
            .any(|p| p.IsNull() || !p.AnchorNode().is_some_and(|n| c.IsConnected(n)))
        {
            return Ok(());
        }
        if dom_start != dom_end
            && dom_end.IsEquivalent(c, &c.QueryPosition(&dom_end, Q::ParagraphStart))
            && !dom_end.AtLastEditingPositionForNode(c)
        {
            let previous = c.QueryPosition(&dom_end, Q::PreviousWithinEditable);
            let previous = if previous.IsNull() {
                dom_start.clone()
            } else {
                previous
            };
            c.SetEndingRange(&dom_start, &previous);
            start = c.QueryPosition(&dom_start, Q::Visible);
            end = c.QueryPosition(&previous, Q::Visible);
            if start.IsNull() || end.IsNull() {
                return Err(EditingAborted);
            }
        }
    } else if start != end && AtEdge(c, &end, Q::ParagraphStart) {
        let previous = c.QueryPosition(&end, Q::PreviousWithinEditable);
        end = if previous.IsNull() {
            start.clone()
        } else {
            previous
        };
        c.SetEndingRange(&start, &end);
    }
    let (start, end) = insert_list_command::SelectionForParagraphIteration(c, start, end);
    if [&start, &end]
        .iter()
        .any(|p| !p.ComputeContainerNode(c).is_some_and(|n| c.Editable(n)))
    {
        return Ok(());
    }
    let start_index = c.IndexForVisiblePosition(&start);
    let end_index = c.IndexForVisiblePosition(&end);
    formatter.FormatSelection(c, &start, &end)?;
    c.UpdateStyleAndLayout();
    if let (Some((start_scope, start)), Some((end_scope, end))) = (start_index, end_index) {
        if start_scope == end_scope && start <= end {
            let start = c.VisiblePositionForIndex(start_scope, start);
            let end = c.VisiblePositionForIndex(end_scope, end);
            if !start.IsNull() && !end.IsNull() {
                c.SetEndingRange(&start, &end);
            }
        }
    }
    Ok(())
}

/// apply_block_element_command.cc:191-302. Keep relocatable next-paragraph
/// positions through each derived range mutation and table-cell boundary.
pub fn FormatSelection<C: ListEditingContext, F: BlockElementFormatter<C>>(
    c: &C,
    formatter: &F,
    start: &Position<u64>,
    end: &Position<u64>,
) -> EditResult {
    let caret = c.QueryPosition(start, Q::Forward);
    if caret
        .AnchorNode()
        .is_some_and(|n| Some(*n) == c.RootEditable(*n) || c.NodeFacts(*n).table_cell)
    {
        let block = formatter.CreateBlockElement(c)?;
        super::composite_edit_command::InsertNodeAt(c, block, &caret)?;
        let br = c.CreateElement("br")?;
        c.Append(br, block)?;
        c.SetEndingPosition(&Position::BeforeNode(br));
        return Ok(());
    }
    let mut last = c.QueryPosition(&end, Q::ParagraphEnd);
    let after_last = c.QueryPosition(&c.QueryPosition(&last, Q::Next), Q::ParagraphEnd);
    let after_last = if after_last.IsNull() {
        None
    } else {
        Some(c.Track(&after_last)?)
    };
    let mut current = c.QueryPosition(&start, Q::ParagraphEnd);
    let mut target_quote = None;
    loop {
        if current.IsNull() || after_last.as_ref().is_some_and(|p| current == c.Tracked(p)) {
            break;
        }
        let at_end = current == last;
        let (mut from, mut to) =
            RangeForParagraphSplittingTextNodesIfNeeded(c, &current, &mut last)?;
        current = c.QueryPosition(&to, Q::Visible);
        let next = EndOfNextParagraphSplittingTextNodesIfNeeded(
            c, &current, &mut last, &mut from, &mut to,
        )?;
        let mut next = if next.IsNull() {
            None
        } else {
            Some(c.Track(&next)?)
        };
        let old_end = c.Track(&to)?;
        let cell = super::insert_list_command::EnclosingTableCell(c, &from);
        c.SetEndingRange(&from, &from);
        formatter.FormatRange(c, &from, &to, end, &mut target_quote, &mut next)?;
        if at_end {
            break;
        }
        let Some(next) = next else {
            break;
        };
        let next = c.Tracked(&next);
        if next.IsNull() || c.Tracked(&old_end).IsEquivalent(c, &next) {
            break;
        }
        if cell.is_some() && cell != super::insert_list_command::EnclosingTableCell(c, &next) {
            target_quote = None;
        }
        c.UpdateStyleAndLayout();
        let next = c.QueryPosition(&next, Q::Visible);
        current = next;
    }
    Ok(())
}

fn TextContainer<C: ListEditingContext>(c: &C, p: &Position<u64>) -> Option<u64> {
    if !p.IsOffsetInAnchor() {
        return None;
    }
    p.ComputeContainerNode(c)
        .filter(|n| c.IsText(n) && c.NodeFacts(*n).has_layout)
}
fn IsNewLine<C: ListEditingContext>(c: &C, p: &Position<u64>) -> bool {
    p.ComputeContainerNode(c).is_some_and(|n| {
        c.IsText(&n) && c.TextCodeUnit(n, p.ComputeOffsetInContainerNode(c)) == Some(10)
    })
}
/// :330-426. SplitTextNode retains the original text as the right half. All
/// source offsets are adjusted immediately before the next lifecycle query.
pub fn RangeForParagraphSplittingTextNodesIfNeeded<C: ListEditingContext>(
    c: &C,
    current_end: &Position<u64>,
    last_end: &mut Position<u64>,
) -> EditResult<(Position<u64>, Position<u64>)> {
    let mut start = c.QueryPosition(current_end, Q::ParagraphStart);
    let mut end = current_end.clone();
    let mut same = false;
    if let Some(start_text) = TextContainer(c, &start) {
        same = TextContainer(c, &end) == Some(start_text);
        let same_last = TextContainer(c, last_end) == Some(start_text);
        let facts = c.NodeFacts(start_text);
        if facts.preserve_breaks
            && IsNewLine(c, &start)
            && !IsNewLine(c, &start.PreviousCodeUnitPosition(c))
            && start.ComputeOffsetInContainerNode(c) > 0
        {
            start = c.QueryPosition(&end.PreviousCodeUnitPosition(c), Q::ParagraphStart);
        }
        let offset = start.ComputeOffsetInContainerNode(c);
        if facts.preserve_spaces && offset > 0 {
            let text = start.ComputeContainerNode(c).ok_or(EditingAborted)?;
            c.SplitText(text, offset)?;
            c.UpdateStyleAndLayout();
            start = Position::FirstPositionInNode(text);
            if same {
                end = Position::new(
                    text,
                    end.ComputeOffsetInContainerNode(c).saturating_sub(offset),
                );
            }
            if same_last {
                *last_end = Position::new(
                    text,
                    last_end
                        .ComputeOffsetInContainerNode(c)
                        .saturating_sub(offset),
                );
            }
        }
    }
    if let Some(end_text) = TextContainer(c, &end) {
        let same_last = TextContainer(c, last_end) == Some(end_text);
        let facts = c.NodeFacts(end_text);
        let offset = end.ComputeOffsetInContainerNode(c);
        let length = c.CharacterDataLength(&end_text).ok_or(EditingAborted)?;
        if facts.preserve_breaks
            && start == end
            && offset < length
            && same_last
            && offset >= last_end.ComputeOffsetInContainerNode(c)
        {
            *last_end = end.clone();
        }
        if c.Editable(end_text) && facts.preserve_spaces && offset > 0 && offset < length {
            c.SplitText(end_text, offset)?;
            c.UpdateStyleAndLayout();
            let previous = c.PreviousSibling(&end_text).ok_or(EditingAborted)?;
            if same {
                start = Position::FirstPositionInOrBeforeNode(c, &previous);
            }
            if same_last {
                *last_end = if last_end.ComputeOffsetInContainerNode(c) == offset {
                    Position::LastPositionInOrAfterNode(c, &previous)
                } else {
                    Position::new(
                        end_text,
                        last_end
                            .ComputeOffsetInContainerNode(c)
                            .saturating_sub(offset),
                    )
                };
            }
            end = Position::LastPositionInNode(c, &previous);
        }
    }
    Ok((start, end))
}
/// :428-497. A leading preserved newline is isolated before cloning trims the
/// moved range, so iteration keeps the same next paragraph owner.
pub fn EndOfNextParagraphSplittingTextNodesIfNeeded<C: ListEditingContext>(
    c: &C,
    current_end: &Position<u64>,
    last_end: &mut Position<u64>,
    start: &mut Position<u64>,
    end: &mut Position<u64>,
) -> EditResult<Position<u64>> {
    let next = c.QueryPosition(&c.QueryPosition(current_end, Q::Next), Q::ParagraphEnd);
    let Some(text) = TextContainer(c, &next) else {
        return Ok(next);
    };
    let offset = next.ComputeOffsetInContainerNode(c);
    if !c.NodeFacts(text).preserve_breaks
        || offset == 0
        || !IsNewLine(c, &Position::FirstPositionInNode(text))
    {
        return Ok(next);
    }
    if c.CharacterDataLength(&text).is_some_and(|len| len > 1) {
        c.SplitText(text, 1)?;
    }
    c.UpdateStyleAndLayout();
    let previous = c.PreviousSibling(&text).filter(|n| c.IsText(n));
    if start.ComputeContainerNode(c) == Some(text) {
        if let Some(previous) = previous {
            *start = Position::new(previous, start.ComputeOffsetInContainerNode(c));
        }
    }
    if end.ComputeContainerNode(c) == Some(text) {
        if let Some(previous) = previous {
            *end = Position::new(previous, end.ComputeOffsetInContainerNode(c));
        }
    }
    if last_end.ComputeContainerNode(c) == Some(text) {
        let last_offset = last_end.ComputeOffsetInContainerNode(c);
        if last_offset < offset {
            if let Some(previous) = previous.filter(|n| {
                c.CharacterDataLength(n)
                    .is_some_and(|len| last_offset <= len)
            }) {
                *last_end = Position::new(previous, last_offset);
            }
        } else {
            *last_end = Position::new(text, last_offset - 1);
        }
    }
    Ok(c.QueryPosition(&Position::new(text, offset - 1), Q::Visible))
}
