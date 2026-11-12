// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! insert_list_command.h:38-85; insert_list_command.cc:49-868.
use super::{
    composite_edit_command::{self, ParagraphMoveHost},
    paragraph_cloning::{self, AtEdge, Contains, InsertAfter, ParagraphCloningContext},
    paragraph_editing_context::{EditResult, EditingAborted},
    structural_editing_context::StructuralPositionQuery as Q,
};
use crate::core::editing::position::Position;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListType {
    Ordered,
    Unordered,
}
impl ListType {
    pub fn Tag(self) -> &'static str {
        match self {
            Self::Ordered => "ol",
            Self::Unordered => "ul",
        }
    }
}
/// Source position indexing uses the native TextIterator, including all visible
/// positions. A DOM range alone cannot recover an anchor removed by cloning.
pub trait ListEditingContext:
    ParagraphCloningContext
    + super::replace_selection_command::GeneralReplacementContext
    + super::apply_style_command::ApplyStyleContext
{
    fn IsInlineElement(&self, node: u64) -> bool;
    fn EndingRange(&self) -> (Position<u64>, Position<u64>);
    fn EndingDomRange(&self) -> (Position<u64>, Position<u64>);
    fn EditingUseDomPositionApiEnabled(&self) -> bool;
    fn IndexForVisiblePosition(&self, position: &Position<u64>) -> Option<(u64, usize)>;
    fn VisiblePositionForIndex(&self, scope: u64, index: usize) -> Position<u64>;
    fn MergeIdenticalElements(&self, first: u64, second: u64) -> EditResult;
    fn SetBlockquoteStyle(&self, node: u64) -> EditResult;
    fn EnsureNodeVisibility(&self, node: u64) -> EditResult<bool>;
}
pub(super) fn IsList<C: ListEditingContext>(c: &C, node: u64) -> bool {
    c.Tag(node, "ol") || c.Tag(node, "ul")
}
pub(super) fn EnclosingList<C: ListEditingContext>(c: &C, node: u64) -> Option<u64> {
    let root = c.RootEditable(node);
    let mut runner = c.Parent(&node);
    while let Some(n) = runner {
        if IsList(c, n) {
            return Some(n);
        }
        if Some(n) == root {
            break;
        }
        runner = c.Parent(&n);
    }
    None
}
pub(super) fn EnclosingListChild<C: ListEditingContext>(c: &C, node: u64) -> Option<u64> {
    let root = c.RootEditable(node);
    let mut runner = Some(node);
    while let Some(n) = runner {
        let parent = c.Parent(&n)?;
        if (c.Tag(n, "li") || IsList(c, parent)) && Some(n) != root {
            return Some(n);
        }
        if Some(n) == root || c.NodeFacts(n).table_cell {
            return None;
        }
        runner = Some(parent);
    }
    None
}
pub(super) fn EnclosingTableCell<C: ListEditingContext>(c: &C, p: &Position<u64>) -> Option<u64> {
    let mut node = p.ComputeContainerNode(c);
    while let Some(n) = node {
        if c.NodeFacts(n).table_cell {
            return Some(n);
        }
        node = c.Parent(&n);
    }
    None
}
pub(super) fn ElementSibling<C: ListEditingContext>(
    c: &C,
    node: u64,
    forward: bool,
) -> Option<u64> {
    let mut runner = if forward {
        c.NextSibling(&node)
    } else {
        c.PreviousSibling(&node)
    };
    while let Some(n) = runner {
        if c.IsElement(n) {
            return Some(n);
        }
        runner = if forward {
            c.NextSibling(&n)
        } else {
            c.PreviousSibling(&n)
        };
    }
    None
}
/// editing_commands_utilities.cc:463-481.
pub(super) fn CanMergeLists<C: ListEditingContext>(c: &C, first: u64, second: u64) -> bool {
    IsList(c, first)
        && ((c.Tag(first, "ol") && c.Tag(second, "ol"))
            || (c.Tag(first, "ul") && c.Tag(second, "ul")))
        && c.Editable(first)
        && c.Editable(second)
        && c.RootEditable(first) == c.RootEditable(second)
        && c.QueryPosition(&Position::AfterNode(first), Q::Visible)
            == c.QueryPosition(
                &c.QueryPosition(&Position::BeforeNode(second), Q::Backward),
                Q::Visible,
            )
}
pub(super) fn MergeWithNeighboringLists<C: ListEditingContext>(
    c: &C,
    mut list: u64,
) -> EditResult<u64> {
    c.UpdateStyleAndLayout();
    if let Some(previous) = ElementSibling(c, list, false).filter(|n| CanMergeLists(c, *n, list)) {
        c.MergeIdenticalElements(previous, list)?;
    }
    c.UpdateStyleAndLayout();
    if let Some(next) = ElementSibling(c, list, true).filter(|n| CanMergeLists(c, list, *n)) {
        c.MergeIdenticalElements(list, next)?;
        list = next;
    }
    Ok(list)
}
pub(super) fn NextParagraph<C: ListEditingContext>(c: &C, p: &Position<u64>) -> Position<u64> {
    c.QueryPosition(
        &c.QueryPosition(&c.QueryPosition(p, Q::ParagraphEnd), Q::NextWithinEditable),
        Q::ParagraphStart,
    )
}
/// editing_commands_utilities.cc:487-542; editing_utilities.cc:1049-1094.
pub(super) fn SelectionForParagraphIteration<C: ListEditingContext>(
    c: &C,
    mut start: Position<u64>,
    mut end: Position<u64>,
) -> (Position<u64>, Position<u64>) {
    let upstream = c.QueryPosition(&end, Q::Backward);
    if upstream
        .AnchorNode()
        .is_some_and(|n| c.IsDisplayInsideTable(n))
        && upstream.AtLastEditingPositionForNode(c)
    {
        if let (Some(table), Some(node)) = (upstream.AnchorNode(), start.AnchorNode()) {
            if crate::core::editing::text_offset_mapping::IsDescendantOf(c, node, table) {
                let previous = c.QueryPosition(&end, Q::PreviousWithinEditable);
                end = if previous.IsNull() {
                    start.clone()
                } else {
                    previous
                };
            }
        }
    }
    let downstream = c.QueryPosition(&start, Q::Forward);
    if downstream
        .AnchorNode()
        .is_some_and(|n| c.IsDisplayInsideTable(n))
        && downstream.AtFirstEditingPositionForNode(c)
    {
        if let (Some(table), Some(node)) = (downstream.AnchorNode(), end.AnchorNode()) {
            if crate::core::editing::text_offset_mapping::IsDescendantOf(c, node, table) {
                let next = c.QueryPosition(&start, Q::NextWithinEditable);
                start = if next.IsNull() { end.clone() } else { next };
            }
        }
    }
    (start, end)
}
pub(super) fn Ordered<C: ListEditingContext>(c: &C, a: &Position<u64>, b: &Position<u64>) -> bool {
    c.Compare(a, b).is_some_and(|o| o != Ordering::Greater)
}
/// editing_utilities.cc:1227-1245, kCannotCrossEditingBoundary branch.
pub(super) fn HighestEnclosingNodeOfType<C: ListEditingContext>(
    c: &C,
    p: &Position<u64>,
    stay_within: Option<u64>,
    predicate: impl Fn(&C, u64) -> bool,
) -> Option<u64> {
    let root = p.ComputeContainerNode(c).and_then(|n| c.RootEditable(n));
    let mut node = p.ComputeContainerNode(c);
    let mut highest = None;
    while let Some(n) = node.filter(|n| Some(*n) != stay_within) {
        if (root.is_none() || c.Editable(n)) && predicate(c, n) {
            highest = Some(n);
        }
        if Some(n) == root {
            break;
        }
        node = c.Parent(&n);
    }
    highest
}
/// composite_edit_command.cc:2429-2456.
fn IsNodeVisiblyContainedWithin<C: ListEditingContext>(
    c: &C,
    node: u64,
    start: &Position<u64>,
    end: &Position<u64>,
) -> bool {
    let before = Position::BeforeNode(node);
    let after = Position::AfterNode(node);
    if Ordered(c, start, &before) && Ordered(c, &after, end) {
        return true;
    }
    let same_start = c.QueryPosition(&before, Q::Visible) == c.QueryPosition(start, Q::Visible);
    if same_start && c.Compare(&after, end) == Some(Ordering::Less) {
        return true;
    }
    let same_end = c.QueryPosition(&after, Q::Visible) == c.QueryPosition(end, Q::Visible);
    if same_end && c.Compare(start, &before) == Some(Ordering::Less) {
        return true;
    }
    same_start && same_end
}
pub struct InsertListCommand {
    pub list_type: ListType,
}
struct CurrentSelection<T> {
    start: T,
    end: T,
}
impl InsertListCommand {
    pub fn new(list_type: ListType) -> Self {
        Self { list_type }
    }
    pub fn PreservesTypingStyle(&self) -> bool {
        true
    }
    pub fn GetInputType(&self) -> interaction_dispatch::core::events::input_event::InputType {
        use interaction_dispatch::core::events::input_event::InputType as I;
        match self.list_type {
            ListType::Ordered => I::kInsertOrderedList,
            ListType::Unordered => I::kInsertUnorderedList,
        }
    }
    /// :149-368. Iterate live paragraphs, recovering disconnected source
    /// anchors through the same native visible-position indices as Blink.
    pub fn Apply<C: ListEditingContext>(
        &self,
        c: &C,
        host: &impl ParagraphMoveHost,
        start: Position<u64>,
        end: Position<u64>,
    ) -> EditResult {
        let start = c.QueryPosition(&start, Q::Visible);
        let mut end = c.QueryPosition(&end, Q::Visible);
        if start.IsNull()
            || end.IsNull()
            || start
                .AnchorNode()
                .and_then(|n| c.RootEditable(*n))
                .is_none()
            || !start.AnchorNode().is_some_and(|n| c.IsConnected(n))
            || !end.AnchorNode().is_some_and(|n| c.IsConnected(n))
        {
            return Ok(());
        }
        if start != end && AtEdge(c, &end, Q::ParagraphStart) {
            let previous = c.QueryPosition(&end, Q::PreviousWithinEditable);
            end = if previous.IsNull() {
                start.clone()
            } else {
                previous
            };
            c.SetEndingRange(&start, &end);
        }
        let (start, mut end) = SelectionForParagraphIteration(c, start, end);
        let mut current_selection = CurrentSelection {
            start: c.Track(&start)?,
            end: c.Track(&end)?,
        };
        if start == end {
            self.ApplyForSingleParagraph(c, host, &start, false, &mut current_selection)?;
            return Ok(());
        }
        let start_index = c.IndexForVisiblePosition(&start);
        let end_index = c.IndexForVisiblePosition(&end);
        let mut last = c.QueryPosition(&end, Q::ParagraphStart);
        let multiple = c.QueryPosition(&start, Q::ParagraphStart) != last;
        let force = multiple && !self.SelectionHasListOfType(c, &start, &end);
        let mut current = start.clone();
        while multiple
            && Ordered(c, &current, &last)
            && c.QueryPosition(&current, Q::ParagraphStart)
                != c.QueryPosition(&last, Q::ParagraphStart)
        {
            if !last.AnchorNode().is_some_and(|n| c.IsConnected(n)) {
                return Ok(());
            }
            c.SetEndingRange(&current, &current);
            if !self.ApplyForSingleParagraph(c, host, &current, force, &mut current_selection)? {
                break;
            }
            c.UpdateStyleAndLayout();
            if !end.AnchorNode().is_some_and(|n| c.IsConnected(n))
                || !last.AnchorNode().is_some_and(|n| c.IsConnected(n))
            {
                end = end_index.map_or_else(Position::default, |(scope, index)| {
                    c.VisiblePositionForIndex(scope, index)
                });
                if end.IsNull() {
                    return Ok(());
                }
                last = c.QueryPosition(&end, Q::ParagraphStart);
            }
            let next = NextParagraph(c, &c.EndingRange().0);
            current = if Ordered(c, &next, &current) {
                c.QueryPosition(&next, Q::Next)
            } else {
                next
            };
            if current.IsNull() {
                return Ok(());
            }
        }
        if multiple {
            c.SetEndingRange(&end, &end);
        }
        self.ApplyForSingleParagraph(
            c,
            host,
            if multiple { &end } else { &start },
            force,
            &mut current_selection,
        )?;
        c.UpdateStyleAndLayout();
        let recover = |original: &Position<u64>, index: Option<(u64, usize)>| {
            if original.AnchorNode().is_some_and(|n| c.IsConnected(n)) {
                c.QueryPosition(original, Q::Visible)
            } else {
                index.map_or_else(Position::default, |(scope, i)| {
                    c.VisiblePositionForIndex(scope, i)
                })
            }
        };
        let start = recover(&start, start_index);
        let end = recover(&end, end_index);
        if !start.IsNull() && !end.IsNull() {
            c.SetEndingRange(&start, &end);
        }
        Ok(())
    }
    fn SelectionHasListOfType<C: ListEditingContext>(
        &self,
        c: &C,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> bool {
        let mut current = c.QueryPosition(start, Q::Visible);
        if current
            .AnchorNode()
            .and_then(|n| EnclosingList(c, *n))
            .is_none()
        {
            return false;
        }
        let end = c.QueryPosition(end, Q::ParagraphStart);
        while !current.IsNull() && current != end {
            if current
                .AnchorNode()
                .and_then(|n| EnclosingList(c, *n))
                .is_none_or(|n| !c.Tag(n, self.list_type.Tag()))
            {
                return false;
            }
            let next = NextParagraph(c, &current);
            if !Ordered(c, &current, &next) || current == next {
                return false;
            }
            current = next;
        }
        true
    }
    /// :375-521, including orphan repair and whole-list conversion.
    fn ApplyForSingleParagraph<C: ListEditingContext>(
        &self,
        c: &C,
        host: &impl ParagraphMoveHost,
        position: &Position<u64>,
        force: bool,
        selection: &mut CurrentSelection<C::Tracked>,
    ) -> EditResult<bool> {
        let selection_node = *position.AnchorNode().ok_or(EditingAborted)?;
        if let Some(child) = EnclosingListChild(c, selection_node) {
            if !c.Parent(&child).is_some_and(|n| c.Editable(n)) {
                return Ok(false);
            }
            let list = if let Some(list) = EnclosingList(c, child) {
                if !c.Editable(list) || !c.Parent(&list).is_some_and(|n| c.Editable(n)) {
                    return Ok(false);
                }
                list
            } else {
                let list = c.CreateElement("ul")?;
                c.InsertBefore(list, child)?;
                c.Remove(child)?;
                c.Append(child, list)?;
                MergeWithNeighboringLists(c, list)?
            };
            let switch = !c.Tag(list, self.list_type.Tag());
            if !switch && force {
                return Ok(true);
            }
            let list_start = c.QueryPosition(&Position::BeforeNode(list), Q::Visible);
            let list_end = c.QueryPosition(&Position::AfterNode(list), Q::Visible);
            let raw_start = c.Tracked(&selection.start);
            let raw_end = c.Tracked(&selection.end);
            let selection_start = c.QueryPosition(&raw_start, Q::Visible);
            let selection_end = c.QueryPosition(&raw_end, Q::Visible);
            if switch && IsNodeVisiblyContainedWithin(c, list, &raw_start, &raw_end) {
                let start_in_list = selection_start == list_start;
                let end_in_list = selection_end == list_end;
                let new_list = c.CreateElement(self.list_type.Tag())?;
                c.InsertBefore(new_list, list)?;
                c.UpdateStyleAndLayout();
                let first = c.QueryPosition(&Position::FirstPositionInNode(list), Q::Visible);
                let last = c.QueryPosition(&Position::LastPositionInNode(c, &list), Q::Visible);
                let outer = first
                    .AnchorNode()
                    .and_then(|n| EnclosingListChild(c, *n))
                    .filter(|n| c.NodeFacts(*n).block_flow)
                    .unwrap_or(list);
                paragraph_cloning::MoveParagraphWithClones(c, &first, &last, new_list, outer)?;
                if c.IsConnected(&list) {
                    c.Remove(list)?;
                }
                let new_list = MergeWithNeighboringLists(c, new_list)?;
                if start_in_list {
                    selection.start = c.Track(&Position::FirstPositionInNode(new_list))?;
                }
                if end_in_list {
                    selection.end = c.Track(&Position::LastPositionInNode(c, &new_list))?;
                }
                c.SetEndingPosition(&Position::FirstPositionInNode(new_list));
                return Ok(true);
            }
            self.ToggleSelectedListItem(c, host, position, list, child, switch || force)?;
            c.UpdateStyleAndLayout();
        } else {
            self.ListifyParagraph(c, position)?;
        }
        Ok(true)
    }
    /// :523-641. Split the list around the original child before moving its
    /// complete contents, including nested lists, to the insertion placeholder.
    fn ToggleSelectedListItem<C: ListEditingContext>(
        &self,
        c: &C,
        host: &impl ParagraphMoveHost,
        position: &Position<u64>,
        list: u64,
        child: u64,
        switch: bool,
    ) -> EditResult {
        let (start, end, previous, next) = if c.Tag(child, "li") {
            (
                Position::FirstPositionInNode(child),
                Position::LastPositionInNode(c, &child),
                c.PreviousSibling(&child),
                c.NextSibling(&child),
            )
        } else {
            let start = c.QueryPosition(position, Q::ParagraphStart);
            let end = c.QueryPosition(&start, Q::ParagraphEnd);
            if start == end {
                return Err(EditingAborted);
            }
            let next = c
                .QueryPosition(&end, Q::Next)
                .AnchorNode()
                .and_then(|n| EnclosingListChild(c, *n));
            let previous = c
                .QueryPosition(&start, Q::Previous)
                .AnchorNode()
                .and_then(|n| EnclosingListChild(c, *n));
            (start, end, previous, next)
        };
        let placeholder = c.CreateElement("br")?;
        let insertion = if EnclosingList(c, list).is_some() {
            let li = c.CreateElement("li")?;
            c.Append(placeholder, li)?;
            li
        } else {
            placeholder
        };
        if next.is_some() && previous.is_some() {
            let split = super::composite_edit_command::SplitTreeToNode(c, next.unwrap(), list)?;
            c.SplitElement(list, split)?;
            c.InsertBefore(insertion, list)?;
        } else if next.is_some() || c.Parent(&child) != Some(list) {
            if c.Parent(&child) != Some(list) {
                let split = super::composite_edit_command::SplitTreeToNode(c, child, list)?;
                c.SplitElement(list, split)?;
            }
            c.InsertBefore(insertion, list)?;
        } else {
            InsertAfter(c, insertion, list)?;
        }
        c.UpdateStyleAndLayout();
        if c.EnsureNodeVisibility(placeholder)? {
            c.UpdateStyleAndLayout();
        }
        let mut destination = Position::BeforeNode(placeholder);
        if switch {
            if let Some(listified) = self.ListifyParagraph(c, &destination)? {
                destination = Position::BeforeNode(listified);
            }
            c.UpdateStyleAndLayout();
        }
        let start = c.QueryPosition(&start, Q::Visible);
        let mut end = c.QueryPosition(&end, Q::Visible);
        if start.IsNull() || end.IsNull() {
            return Err(EditingAborted);
        }
        if !Ordered(c, &start, &end) {
            end = start.clone();
        }
        composite_edit_command::MoveParagraphs(
            c,
            host,
            &start,
            &end,
            &destination,
            true,
            Some(child),
        )
    }
    /// :643-791. Adjacent lists must share both their table cell and enclosing
    /// list; merging uses visible adjacency, including unrendered siblings.
    fn AdjacentList<C: ListEditingContext>(
        &self,
        c: &C,
        p: &Position<u64>,
        adjacent: &Position<u64>,
    ) -> Option<u64> {
        let mut list = EnclosingList(c, *adjacent.AnchorNode()?)?;
        while let Some(parent) = EnclosingList(c, list) {
            list = parent;
        }
        if !c.Tag(list, self.list_type.Tag())
            || Contains(c, list, *p.AnchorNode()?)
            || EnclosingTableCell(c, p) != EnclosingTableCell(c, adjacent)
            || EnclosingList(c, list) != EnclosingList(c, *p.AnchorNode()?)
        {
            return None;
        }
        Some(list)
    }
    fn ListifyParagraph<C: ListEditingContext>(
        &self,
        c: &C,
        original: &Position<u64>,
    ) -> EditResult<Option<u64>> {
        let start = c.QueryPosition(original, Q::ParagraphStart);
        let end = c.QueryPosition(&start, Q::ParagraphEnd);
        if start.IsNull() || end.IsNull() {
            return Ok(None);
        }
        let original_position = original.clone();
        let original = if original.IsOffsetInAnchor() {
            Some(c.Track(original)?)
        } else {
            None
        };
        let previous = self.AdjacentList(
            c,
            &start,
            &c.QueryPosition(&start, Q::PreviousWithinEditable),
        );
        let next = self.AdjacentList(c, &start, &c.QueryPosition(&end, Q::NextWithinEditable));
        let item = c.CreateElement("li")?;
        let mut move_position = start.clone();
        let list = if let Some(previous) = previous {
            c.Append(item, previous)?;
            previous
        } else if let Some(next) = next {
            super::composite_edit_command::InsertNodeAt(c, item, &Position::BeforeNode(next))?;
            next
        } else {
            let mut start_position = start.clone();
            if start == end
                && start
                    .AnchorNode()
                    .is_some_and(|n| c.IsText(n) && c.NodeFacts(*n).block)
            {
                let br = c.CreateElement("br")?;
                super::composite_edit_command::InsertNodeAt(c, br, &start_position)?;
                start_position = Position::BeforeNode(br);
            }
            c.UpdateStyleAndLayout();
            let list = c.CreateElement(self.list_type.Tag())?;
            let mut insertion = c.QueryPosition(&start_position, Q::Backward);
            if insertion.AnchorNode().is_some_and(|n| c.Tag(*n, "span")) {
                insertion =
                    Position::BeforeNode(insertion.ComputeContainerNode(c).ok_or(EditingAborted)?);
            }
            if let Some(child) = insertion
                .AnchorNode()
                .and_then(|n| EnclosingListChild(c, *n))
                .filter(|n| c.Tag(*n, "li"))
            {
                insertion = Position::BeforeNode(child);
            }
            super::composite_edit_command::InsertNodeAt(c, list, &insertion)?;
            c.Append(item, list)?;
            c.UpdateStyleAndLayout();
            move_position = if insertion != start_position {
                start_position
            } else {
                original
                    .as_ref()
                    .map_or_else(|| original_position.clone(), |p| c.Tracked(p))
            };
            list
        };
        self.MoveIntoEmptyListItem(c, &move_position, item)?;
        c.UpdateStyleAndLayout();
        if let (Some(previous), Some(next)) = (previous, next) {
            if CanMergeLists(c, previous, next) {
                c.MergeIdenticalElements(previous, next)?;
            }
        } else if previous.is_none() && next.is_none() {
            MergeWithNeighboringLists(c, list)?;
        }
        Ok(c.ChildAt(&item, 0))
    }
    /// :793-868.
    fn MoveIntoEmptyListItem<C: ListEditingContext>(
        &self,
        c: &C,
        position: &Position<u64>,
        item: u64,
    ) -> EditResult {
        let initial = c.EndingRange();
        let placeholder = c.CreateElement("br")?;
        c.Append(placeholder, item)?;
        c.UpdateStyleAndLayout();
        let start = c.QueryPosition(position, Q::ParagraphStart);
        let end = c.QueryPosition(position, Q::ParagraphEnd);
        if start.IsNull() || end.IsNull() {
            return Err(EditingAborted);
        }
        let block = c.EnclosingBlock(&start);
        let outer = HighestEnclosingNodeOfType(c, &start, block, |c, n| c.IsInlineElement(n));
        paragraph_cloning::MoveParagraphWithClones(
            c,
            &start,
            &end,
            item,
            outer.unwrap_or(*start.AnchorNode().ok_or(EditingAborted)?),
        )?;
        c.Remove(placeholder)?;
        if let Some(node) = start.AnchorNode().filter(|n| c.IsConnected(n)) {
            c.Remove(*node)?;
        }
        if [&initial.0, &initial.1]
            .iter()
            .all(|p| p.AnchorNode().is_some_and(|n| c.IsConnected(n)))
        {
            c.SetEndingRange(&initial.0, &initial.1);
        } else {
            c.SetEndingPosition(&Position::FirstPositionInNode(item));
        }
        Ok(())
    }
}
