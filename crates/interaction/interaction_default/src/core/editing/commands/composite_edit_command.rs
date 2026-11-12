// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: composite_edit_command.cc:119-141,211-220,247-269,2365-2428.
use super::{
    paragraph_editing_context::EditResult, simple_edit_commands::SimpleEditCommand,
    undo_step::UndoStep,
};
use crate::core::editing::visible_units_paragraph::IsParagraphEdge;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

/// Source virtual command classification, retained on the same composite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositeCommandKind {
    Other,
    Typing,
    DragAndDrop,
}
enum Command<S: Clone> {
    Simple(Rc<dyn SimpleEditCommand>),
    Composite(Rc<CompositeEditCommand<S>>),
}
/// Source composite ownership, including the raw-DOM and visible selection
/// lanes. Only the top-level command owns an UndoStep; child selections remain
/// command-local and propagate in the original source order.
pub struct CompositeEditCommand<S: Clone> {
    // composite_edit_command.cc:119-121,2480-2483: the input event consumes
    // the original DataTransfer after synchronous editing and listener reentry.
    data_transfer: Option<Rc<interaction_dispatch::core::clipboard::data_transfer::DataTransfer>>,
    parent: RefCell<Weak<Self>>,
    kind: Cell<CompositeCommandKind>,
    undo_step: RefCell<Option<Rc<UndoStep<S>>>>,
    starting: RefCell<S>,
    ending: RefCell<S>,
    starting_dom: RefCell<S>,
    ending_dom: RefCell<S>,
    commands: RefCell<Vec<Command<S>>>,
    selection_is_directional: Cell<bool>,
}
impl<S: Clone> CompositeEditCommand<S> {
    pub fn new(visible: S, raw_dom: S) -> Rc<Self> {
        Self::new_with_data_transfer(visible, raw_dom, None)
    }
    pub fn new_with_data_transfer(
        visible: S,
        raw_dom: S,
        data_transfer: Option<
            Rc<interaction_dispatch::core::clipboard::data_transfer::DataTransfer>,
        >,
    ) -> Rc<Self> {
        Rc::new(Self {
            data_transfer,
            parent: RefCell::new(Weak::new()),
            kind: Cell::new(CompositeCommandKind::Other),
            undo_step: RefCell::new(None),
            starting: RefCell::new(visible.clone()),
            ending: RefCell::new(visible),
            starting_dom: RefCell::new(raw_dom.clone()),
            ending_dom: RefCell::new(raw_dom),
            commands: RefCell::new(Vec::new()),
            selection_is_directional: Cell::new(false),
        })
    }
    pub fn DataTransfer(
        &self,
    ) -> Option<&Rc<interaction_dispatch::core::clipboard::data_transfer::DataTransfer>> {
        self.data_transfer.as_ref()
    }
    pub fn SetCommandKind(&self, kind: CompositeCommandKind) {
        self.kind.set(kind);
    }
    pub fn IsTypingCommand(&self) -> bool {
        self.kind.get() == CompositeCommandKind::Typing
    }
    pub fn IsDragAndDropCommand(&self) -> bool {
        self.kind.get() == CompositeCommandKind::DragAndDrop
    }
    pub fn GetUndoStep(&self) -> Option<Rc<UndoStep<S>>> {
        self.undo_step.borrow().clone()
    }
    fn Parent(&self) -> Option<Rc<Self>> {
        self.parent.borrow().upgrade()
    }
    fn IsFirstCommand(&self, command: &Rc<Self>) -> bool {
        matches!(self.commands.borrow().first(), Some(Command::Composite(first)) if Rc::ptr_eq(first,command))
    }
    pub fn SetParent(&self, parent: Option<&Rc<Self>>) {
        *self.parent.borrow_mut() = parent.map(Rc::downgrade).unwrap_or_default();
        if let Some(parent) = parent {
            *self.starting.borrow_mut() = parent.EndingSelection();
            *self.ending.borrow_mut() = parent.EndingSelection();
            *self.starting_dom.borrow_mut() = parent.EndingDomSelection();
            *self.ending_dom.borrow_mut() = parent.EndingDomSelection();
        }
    }
    pub fn EnsureUndoStep(self: &Rc<Self>) -> Rc<UndoStep<S>> {
        let mut top = self.clone();
        while let Some(parent) = top.Parent() {
            top = parent;
        }
        let mut slot = top.undo_step.borrow_mut();
        slot.get_or_insert_with(|| {
            let step = Rc::new(UndoStep::new(
                top.StartingSelection(),
                top.EndingSelection(),
                top.StartingDomSelection(),
                top.EndingDomSelection(),
            ));
            step.SetSelectionIsDirectional(top.selection_is_directional.get());
            step
        })
        .clone()
    }
    pub fn StartingSelection(&self) -> S {
        self.starting.borrow().clone()
    }
    pub fn EndingSelection(&self) -> S {
        self.ending.borrow().clone()
    }
    pub fn StartingDomSelection(&self) -> S {
        self.starting_dom.borrow().clone()
    }
    pub fn EndingDomSelection(&self) -> S {
        self.ending_dom.borrow().clone()
    }
    pub fn SetStartingSelection(self: &Rc<Self>, selection: S) {
        let mut current = self.clone();
        loop {
            if let Some(step) = current.undo_step.borrow().as_ref() {
                step.SetStartingSelection(selection.clone());
            }
            *current.starting.borrow_mut() = selection.clone();
            let Some(parent) = current.Parent() else {
                break;
            };
            if parent.IsFirstCommand(&current) {
                break;
            }
            current = parent;
        }
    }
    pub fn SetEndingSelection(self: &Rc<Self>, selection: S) {
        let mut current = Some(self.clone());
        while let Some(command) = current {
            if let Some(step) = command.undo_step.borrow().as_ref() {
                step.SetEndingSelection(selection.clone());
            }
            *command.ending.borrow_mut() = selection.clone();
            current = command.Parent();
        }
    }
    pub fn SetStartingDomSelection(self: &Rc<Self>, selection: S) {
        let mut current = self.clone();
        loop {
            if let Some(step) = current.undo_step.borrow().as_ref() {
                step.SetStartingDomSelection(selection.clone());
            }
            *current.starting_dom.borrow_mut() = selection.clone();
            let Some(parent) = current.Parent() else {
                break;
            };
            if parent.IsFirstCommand(&current) {
                break;
            }
            current = parent;
        }
    }
    pub fn SetEndingDomSelection(self: &Rc<Self>, selection: S) {
        let mut current = Some(self.clone());
        while let Some(command) = current {
            if let Some(step) = command.undo_step.borrow().as_ref() {
                step.SetEndingDomSelection(selection.clone());
            }
            *command.ending_dom.borrow_mut() = selection.clone();
            current = command.Parent();
        }
    }
    pub fn SelectionIsDirectional(&self) -> bool {
        self.selection_is_directional.get()
    }
    pub fn SetSelectionIsDirectional(&self, directional: bool) {
        self.selection_is_directional.set(directional);
        if let Some(step) = self.undo_step.borrow().as_ref() {
            step.SetSelectionIsDirectional(directional);
        }
    }
    pub fn ApplyCommandToComposite(
        self: &Rc<Self>,
        command: Rc<dyn SimpleEditCommand>,
    ) -> EditResult {
        command.DoApply()?;
        self.EnsureUndoStep().Append(command.clone());
        self.commands.borrow_mut().push(Command::Simple(command));
        Ok(())
    }
    pub fn ApplyCompositeToComposite(
        self: &Rc<Self>,
        command: Rc<Self>,
        apply: impl FnOnce() -> EditResult,
    ) -> EditResult {
        command.SetParent(Some(self));
        command.SetSelectionIsDirectional(self.selection_is_directional.get());
        if let Err(error) = apply() {
            command.SetParent(None);
            return Err(error);
        }
        self.commands.borrow_mut().push(Command::Composite(command));
        Ok(())
    }
    pub fn AppendCommandToUndoStep(self: &Rc<Self>, command: Rc<Self>) {
        self.EnsureUndoStep().AppendStep(&command.EnsureUndoStep());
        command.undo_step.borrow_mut().take();
        command.SetParent(Some(self));
        self.commands.borrow_mut().push(Command::Composite(command));
    }
}

// Source composite_edit_command.cc:757-866,954-1029.
use super::structural_editing_context::{
    StructuralBooleanQuery, StructuralEditingContext, StructuralPositionQuery,
};
use crate::core::editing::{
    position::{Position, PositionAnchorType},
    text_offset_mapping,
};
use std::cmp::Ordering;
pub(super) fn DeleteInsignificant<C: StructuralEditingContext>(
    ctx: &C,
    start: &Position<u64>,
    end: &Position<u64>,
) -> EditResult {
    if start.IsNull() || end.IsNull() || ctx.Compare(start, end) != Some(Ordering::Less) {
        return Ok(());
    }
    let mut nodes = Vec::new();
    let mut node = start.AnchorNode().copied();
    while let Some(current) = node {
        if ctx.IsText(&current) {
            nodes.push(current);
        }
        if Some(&current) == end.AnchorNode() {
            break;
        }
        node = text_offset_mapping::NextNode(ctx, &current, None);
    }
    for node in nodes {
        let from = if Some(&node) == start.AnchorNode() {
            start.ComputeOffsetInContainerNode(ctx)
        } else {
            0
        };
        let to = if Some(&node) == end.AnchorNode() {
            end.ComputeOffsetInContainerNode(ctx)
        } else {
            ctx.CharacterDataLength(&node).unwrap_or(0)
        };
        if from >= to {
            continue;
        }
        ctx.UpdateStyleAndLayout();
        let facts = ctx.NodeFacts(node);
        if !facts.has_layout {
            continue;
        }
        if facts.insignificant_text {
            ctx.Remove(node)?;
            continue;
        }
        if to > ctx.CharacterDataLength(&node).unwrap_or(0) {
            continue;
        }
        let text = ctx.OriginalPlainText(&Position::new(node, from), &Position::new(node, to))?;
        if text.is_empty() {
            ctx.DeleteText(node, from, to - from)?;
        } else {
            ctx.ReplaceText(node, from, to - from, &text)?;
        }
    }
    Ok(())
}
pub(super) fn DeleteInsignificantDownstream<C: StructuralEditingContext>(
    ctx: &C,
    position: &Position<u64>,
) -> EditResult {
    let next = ctx.QueryPosition(position, StructuralPositionQuery::Next);
    let end = ctx.QueryPosition(&next, StructuralPositionQuery::Forward);
    DeleteInsignificant(ctx, position, &end)
}
pub fn RebalanceWhitespaceAt<C: StructuralEditingContext>(
    ctx: &C,
    position: &Position<u64>,
) -> EditResult {
    struct Rebalance<'a, C>(&'a C);
    impl<C: StructuralEditingContext> WhitespaceRebalanceContext for Rebalance<'_, C> {
        type Node = u64;
        type Position = Position<u64>;
        type VisiblePosition = Position<u64>;
        type Affinity = ();
        fn UpdateStyleAndLayout(&mut self) {
            self.0.UpdateStyleAndLayout()
        }
        fn ContainerNode(&self, p: &Position<u64>) -> Option<u64> {
            p.ComputeContainerNode(self.0)
        }
        fn AnchorType(&self, p: &Position<u64>) -> PositionAnchorType {
            p.AnchorType()
        }
        fn IsText(&self, n: &u64) -> bool {
            self.0.IsText(n)
        }
        fn TextLength(&self, n: &u64) -> usize {
            self.0.CharacterDataLength(n).unwrap_or(0)
        }
        fn ShouldPreserveWhiteSpaces(&self, n: &u64) -> bool {
            self.0.NodeFacts(*n).preserve_spaces
        }
        fn EditingOffset(&self, p: &Position<u64>) -> usize {
            p.ComputeEditingOffset(self.0)
        }
        fn TextCodeUnit(&self, n: &u64, o: usize) -> Option<u16> {
            self.0.TextCodeUnit(*n, o)
        }
        fn OffsetInNode(&self, n: &u64, o: usize) -> Position<u64> {
            Position::new(*n, o)
        }
        fn DefaultAffinity(&self) {}
        fn CreateVisiblePosition(&self, p: &Position<u64>, _: ()) -> Option<Position<u64>> {
            let p = self.0.QueryPosition(p, StructuralPositionQuery::Visible);
            (!p.IsNull()).then_some(p)
        }
        fn IsStartOfParagraph(&self, p: &Position<u64>) -> bool {
            IsParagraphEdge(self.0, p, true)
        }
        fn IsEndOfParagraph(&self, p: &Position<u64>) -> bool {
            IsParagraphEdge(self.0, p, false)
        }
        fn NextSibling(&self, n: &u64) -> Option<u64> {
            self.0.NextSibling(n)
        }
        fn ReplaceTextInNode(&mut self, n: &u64, o: usize, c: usize, t: &[u16]) -> EditResult {
            self.0.ReplaceText(*n, o, c, t)
        }
        fn IsRichlyEditable(&self, n: &u64) -> bool {
            self.0.NodeFacts(*n).richly_editable
        }
        fn HasLayoutObject(&self, n: &u64) -> bool {
            self.0.NodeFacts(*n).has_layout
        }
        fn ShouldCollapseBreaks(&self, n: &u64) -> bool {
            !self.0.NodeFacts(*n).preserve_breaks
        }
    }
    RebalanceWhitespaceAtWithContext(&mut Rebalance(ctx), position)
}

// Source composite_edit_command.cc:2128-2214.
use super::{
    break_blockquote_command::{self, BreakBlockquoteContext},
    paragraph_editing_context::EditingAborted,
    replace_selection_command,
    typing_command::TypingDeletionStructuralContext,
};
use crate::core::editing::position::PositionTree;
pub fn BreakOutOfEmptyMailBlockquotedParagraph<C: TypingDeletionStructuralContext>(
    ctx: &C,
    ending: &Position<u64>,
) -> EditResult<bool> {
    ctx.UpdateStyleAndLayout();
    let caret = ctx.QueryPosition(ending, StructuralPositionQuery::Visible);
    let Some(highest) = break_blockquote_command::EnclosingQuote(ctx, &caret, true) else {
        return Ok(false);
    };
    if !IsParagraphEdge(ctx, &caret, true) || !IsParagraphEdge(ctx, &caret, false) {
        return Ok(false);
    }
    let previous = ctx.QueryPosition(&caret, StructuralPositionQuery::PreviousWithinEditable);
    if break_blockquote_command::EnclosingQuote(ctx, &previous, false).is_some() {
        return Ok(false);
    }
    let br = ctx.CreateElement("br")?;
    ctx.InsertBefore(br, highest)?;
    ctx.UpdateStyleAndLayout();
    let at_br = ctx.QueryPosition(&Position::BeforeNode(br), StructuralPositionQuery::Visible);
    if !IsParagraphEdge(ctx, &at_br, true) {
        let second = ctx.CreateElement("br")?;
        ctx.InsertBefore(second, br)?;
        ctx.UpdateStyleAndLayout();
    }
    ctx.SetEndingPosition(&at_br);
    if !ctx.PositionBoolean(&caret, None, StructuralBooleanQuery::LineBreakExists) {
        return Ok(false);
    }
    let position = ctx.QueryPosition(&caret, StructuralPositionQuery::Forward);
    let node = *position.AnchorNode().ok_or(EditingAborted)?;
    if ctx.Tag(node, "br") {
        replace_selection_command::RemoveNodeAndPruneAncestors(ctx, node)?;
    } else if PositionTree::IsText(ctx, &node) {
        debug_assert_eq!(position.ComputeOffsetInContainerNode(ctx), 0);
        let parent = PositionTree::Parent(ctx, &node);
        ctx.DeleteText(node, 0, 1)?;
        replace_selection_command::PruneAncestors(ctx, parent)?;
    }
    Ok(true)
}

// Source composite_edit_command.cc:486-495,666-708,868-943,1102-1118,
// 1238-1271,2220-2325. All positions use the live DOM owner.
use super::{
    replace_selection_command::GeneralReplacementContext,
    structural_editing_context::{StructuralBooleanQuery as B, StructuralPositionQuery as Q},
};
fn Ancestor<C: GeneralReplacementContext>(
    c: &C,
    p: &Position<u64>,
    predicate: impl Fn(u64) -> bool,
) -> Option<u64> {
    let mut node = p.AnchorNode().copied();
    while let Some(n) = node {
        if predicate(n) {
            return Some(n);
        }
        node = c.Parent(&n);
    }
    None
}
fn Contains<C: PositionTree<Node = u64>>(c: &C, parent: u64, node: u64) -> bool {
    node == parent || text_offset_mapping::IsDescendantOf(c, &node, &parent)
}
pub fn RemovePreservingChildren<C: StructuralEditingContext>(c: &C, node: u64) -> EditResult {
    let children: Vec<_> = (0..c.ChildCount(&node))
        .filter_map(|i| c.ChildAt(&node, i))
        .collect();
    for child in children {
        c.Remove(child)?;
        c.InsertBefore(child, node)?;
    }
    c.Remove(node)
}
fn PushAnchorElementDown<C: GeneralReplacementContext>(c: &C, anchor: u64) -> EditResult {
    let mut first = anchor;
    while let Some(node) = c.ChildAt(&first, 0) {
        first = node;
    }
    let mut last = anchor;
    while let Some(node) = c
        .ChildCount(&last)
        .checked_sub(1)
        .and_then(|i| c.ChildAt(&last, i))
    {
        last = node;
    }
    let start = if c.EditingUseDomPositionApi() {
        Position::FirstPositionInOrBeforeNode(c, &first)
    } else {
        c.QueryPosition(&Position::FirstPositionInNode(anchor), Q::Visible)
    };
    let end = if c.EditingUseDomPositionApi() {
        Position::LastPositionInOrAfterNode(c, &last)
    } else {
        c.QueryPosition(&Position::LastPositionInNode(c, &anchor), Q::Visible)
    };
    c.SetEndingReplacementSelection(&start, &end);
    c.ApplyReplacementStyledElement(anchor, &start, &end)?;
    if c.IsConnected(&anchor) {
        RemovePreservingChildren(c, anchor)?;
    }
    Ok(())
}
pub fn PositionAvoidingSpecialElementBoundary<C: GeneralReplacementContext>(
    c: &C,
    original: &Position<u64>,
) -> EditResult<Position<u64>> {
    if original.IsNull() {
        return Ok(original.clone());
    }
    c.UpdateStyleAndLayout();
    let canonical = c.QueryPosition(original, Q::Visible);
    let Some(mut anchor) = Ancestor(c, original, |n| c.Tag(n, "a") && c.HasLinkHref(n)) else {
        return Ok(original.clone());
    };
    let mut result = original.clone();
    if !c.NodeFacts(anchor).block {
        let first = c.QueryPosition(&Position::FirstPositionInNode(anchor), Q::Visible);
        let last = c.QueryPosition(&Position::LastPositionInNode(c, &anchor), Q::Visible);
        if canonical == last {
            if original
                .AnchorNode()
                .and_then(|n| c.EnclosingBlock(&Position::FirstPositionInNode(*n)))
                .is_some_and(|block| text_offset_mapping::IsDescendantOf(c, &block, &anchor))
            {
                PushAnchorElementDown(c, anchor)?;
                let Some(new) = Ancestor(c, original, |n| c.Tag(n, "a") && c.HasLinkHref(n)) else {
                    return Ok(original.clone());
                };
                anchor = new;
            }
            c.UpdateStyleAndLayout();
            let downstream = c.QueryPosition(
                if c.EditingUseDomPositionApi() {
                    original
                } else {
                    &canonical
                },
                Q::Forward,
            );
            let line_break = if c.EditingUseDomPositionApi() {
                c.RawLineBreakExists(original)
            } else {
                c.PositionBoolean(&canonical, None, B::LineBreakExists)
            };
            let line_container = if c.EditingUseDomPositionApi() {
                original
            } else {
                &downstream
            };
            if line_break
                && line_container
                    .AnchorNode()
                    .is_some_and(|n| Contains(c, anchor, *n))
            {
                return Ok(original.clone());
            }
            result = Position::AfterNode(anchor).ToOffsetInAnchor(c);
        }
        if canonical == first {
            if original
                .AnchorNode()
                .and_then(|n| c.EnclosingBlock(&Position::FirstPositionInNode(*n)))
                .is_some_and(|block| text_offset_mapping::IsDescendantOf(c, &block, &anchor))
            {
                PushAnchorElementDown(c, anchor)?;
                let Some(new) = Ancestor(c, original, |n| c.Tag(n, "a") && c.HasLinkHref(n)) else {
                    return Ok(original.clone());
                };
                anchor = new;
            }
            result = Position::BeforeNode(anchor).ToOffsetInAnchor(c);
        }
    }
    if result.IsNull()
        || result
            .AnchorNode()
            .and_then(|n| c.RootEditable(*n))
            .is_none()
    {
        result = original.clone();
    }
    Ok(result)
}
pub fn PositionOutsideTabSpan<C: GeneralReplacementContext>(
    c: &C,
    p: &Position<u64>,
) -> EditResult<Position<u64>> {
    let Some(node) = p
        .AnchorNode()
        .copied()
        .filter(|n| c.IsTabSpanElementTextNode(*n))
    else {
        return Ok(p.clone());
    };
    match p.AnchorType() {
        PositionAnchorType::BeforeAnchor => {
            return Ok(Position::BeforeNode(node).ToOffsetInAnchor(c))
        }
        PositionAnchorType::AfterAnchor => return Ok(Position::AfterNode(node).ToOffsetInAnchor(c)),
        PositionAnchorType::AfterChildren => return Err(EditingAborted),
        PositionAnchorType::OffsetInAnchor => {}
    }
    let span = c.Parent(&node).ok_or(EditingAborted)?;
    c.UpdateStyleAndLayout();
    let facts = c.NodeFacts(node);
    let offset = p.ComputeOffsetInContainerNode(c);
    if offset <= facts.caret_min {
        return Ok(Position::BeforeNode(span).ToOffsetInAnchor(c));
    }
    if offset >= facts.caret_max {
        return Ok(Position::AfterNode(
            if c.TabSpanCheckSibling() && c.NextSibling(&node).is_some() {
                node
            } else {
                span
            },
        )
        .ToOffsetInAnchor(c));
    }
    c.SplitText(node, offset)?;
    if c.Parent(&span)
        .is_some_and(|p| c.IsElement(p) && c.Editable(p))
    {
        let facts = c.NodeFacts(span);
        let parent = if !facts.has_layout || !facts.inline {
            c.WrapReplacementContentsInDummySpan(span)?
        } else {
            span
        };
        c.SplitElement(parent, node)?;
    }
    Ok(Position::BeforeNode(span).ToOffsetInAnchor(c))
}
pub fn PrepareWhitespaceAtPositionForSplit<C: GeneralReplacementContext>(
    c: &C,
    p: &mut Position<u64>,
) -> EditResult {
    let Some(text) = p
        .AnchorNode()
        .copied()
        .filter(|n| c.IsText(n) && c.NodeFacts(*n).richly_editable)
    else {
        return Ok(());
    };
    let facts = c.NodeFacts(text);
    if c.CharacterDataLength(&text) == Some(0) || facts.has_layout && facts.preserve_spaces {
        return Ok(());
    }
    let upstream = c.QueryPosition(p, Q::Backward);
    let tracked = c.Track(&upstream)?;
    super::composite_edit_command::DeleteInsignificant(
        c,
        &upstream,
        &c.QueryPosition(p, Q::Forward),
    )?;
    c.UpdateStyleAndLayout();
    *p = c.QueryPosition(&c.Tracked(&tracked), Q::Forward);
    let previous = if c.EditingUseDomPositionApi() {
        c.MoveByGrapheme(p, false)
    } else {
        c.QueryPosition(&c.QueryPosition(p, Q::Visible), Q::Previous)
    };
    ReplaceCollapsibleWhitespace(c, &previous)?;
    c.UpdateStyleAndLayout();
    let position = if c.EditingUseDomPositionApi() {
        p.clone()
    } else {
        c.QueryPosition(p, Q::Visible)
    };
    ReplaceCollapsibleWhitespace(c, &position)
}
fn ReplaceCollapsibleWhitespace<C: GeneralReplacementContext>(
    c: &C,
    p: &Position<u64>,
) -> EditResult {
    if !crate::core::editing::editing_utilities::IsCollapsibleWhitespace(
        c.CharacterAtVisibleBoundary(p, false),
    ) {
        return Ok(());
    }
    let p = c.QueryPosition(p, Q::Forward);
    if let Some(text) = p.ComputeContainerNode(c).filter(|n| c.IsText(n)) {
        c.ReplaceText(text, p.ComputeOffsetInContainerNode(c), 1, &[0xA0])?;
    }
    Ok(())
}
pub fn RemovePlaceholderAt<C: GeneralReplacementContext>(c: &C, p: &Position<u64>) -> EditResult {
    if let Some(node) = p.AnchorNode() {
        if c.NodeFacts(*node).br {
            c.Remove(*node)
        } else {
            c.DeleteText(*node, p.ComputeOffsetInContainerNode(c), 1)
        }
    } else {
        Ok(())
    }
}

// Source composite_edit_command.cc:346-425,510-528,1032-1053,2328-2362.
use super::editing_commands_utilities::EnclosingListChild;
use crate::core::editing::{
    commands::structural_editing_support::{ParentAfter, ParentBefore},
    editing_utilities::IsList,
};

pub(super) fn AppendPlaceholder<C: StructuralEditingContext>(ctx: &C, parent: u64) -> EditResult {
    let br = ctx.CreateElement("br")?;
    ctx.Append(br, parent)
}
pub(super) fn InsertAfter<C: StructuralEditingContext>(
    ctx: &C,
    node: u64,
    reference: u64,
) -> EditResult {
    if let Some(next) = ctx.NextSibling(&reference) {
        ctx.InsertBefore(node, next)
    } else {
        ctx.Append(node, ctx.Parent(&reference).ok_or(EditingAborted)?)
    }
}
pub(super) fn MoveRemainingSiblings<C: StructuralEditingContext>(
    ctx: &C,
    mut node: Option<u64>,
    past: Option<u64>,
    parent: u64,
) -> EditResult {
    // composite_edit_command.cc:510-528 collects siblings before synchronous
    // mutation reentry; only those original identities are moved.
    let mut nodes = Vec::new();
    while let Some(current) = node.filter(|current| Some(*current) != past) {
        nodes.push(current);
        node = ctx.NextSibling(&current);
    }
    for node in nodes {
        ctx.Remove(node)?;
        ctx.Append(node, parent)?;
    }
    Ok(())
}
pub(super) fn SplitTreeToNode<C: StructuralEditingContext>(
    ctx: &C,
    mut node: u64,
    end: u64,
) -> EditResult<u64> {
    // composite_edit_command.cc:2328-2362. Original nodes remain the right
    // half; do not split ancestors into visually empty left halves.
    if !Contains(ctx, end, node) {
        return Ok(end);
    }
    while let Some(parent) = ctx.Parent(&node).filter(|parent| *parent != end) {
        ctx.UpdateStyleAndLayout();
        if ctx.PreviousSibling(&node).is_some()
            && ctx.QueryPosition(
                &Position::FirstPositionInNode(parent),
                StructuralPositionQuery::Backward,
            ) != ctx.QueryPosition(&ParentBefore(ctx, node), StructuralPositionQuery::Backward)
        {
            ctx.SplitElement(parent, node)?;
        }
        node = parent;
    }
    Ok(node)
}
pub fn InsertNodeAt<C: StructuralEditingContext>(
    ctx: &C,
    node: u64,
    position: &Position<u64>,
) -> EditResult {
    let container = position.ComputeContainerNode(ctx).ok_or(EditingAborted)?;
    let offset = position.ComputeOffsetInContainerNode(ctx);
    if ctx.IsText(&container) {
        let length = ctx.CharacterDataLength(&container).ok_or(EditingAborted)?;
        if offset == length {
            InsertAfter(ctx, node, container)
        } else {
            if offset != 0 {
                ctx.SplitText(container, offset)?;
            }
            ctx.InsertBefore(node, container)
        }
    } else if let Some(child) = ctx.ChildAt(&container, offset) {
        ctx.InsertBefore(node, child)
    } else {
        ctx.Append(node, container)
    }
}

pub fn BreakOutOfEmptyListItem<C: StructuralEditingContext>(
    ctx: &C,
    position: &Position<u64>,
) -> EditResult<Option<Position<u64>>> {
    // composite_edit_command.cc:1994-2120. Only an empty list paragraph may
    // leave its list; preserve list halves and the native original node ids.
    if !IsParagraphEdge(ctx, position, true) || !IsParagraphEdge(ctx, position, false) {
        return Ok(None);
    }
    let Some(item) = EnclosingListChild(ctx, position) else {
        return Ok(None);
    };
    let Some(list) = ctx.Parent(&item).filter(|list| {
        IsList(ctx, *list) && ctx.Editable(*list) && ctx.RootEditable(item) != Some(*list)
    }) else {
        return Ok(None);
    };
    let style = ctx.MergeTypingStyle(ctx.CaptureStyle(position, false)?)?;
    let mut new_block = None;
    if let Some(outer) = ctx.Parent(&list) {
        if ctx.Tag(outer, "li") {
            if ctx.QueryPosition(&ParentAfter(ctx, outer), StructuralPositionQuery::Visible)
                == ctx.QueryPosition(&ParentAfter(ctx, list), StructuralPositionQuery::Visible)
            {
                ctx.SplitElement(outer, list)?;
                RemovePreservingChildren(ctx, ctx.Parent(&list).ok_or(EditingAborted)?)?;
                new_block = Some(ctx.CreateElement("li")?);
            }
        } else if IsList(ctx, outer) {
            new_block = Some(ctx.CreateElement("li")?);
        }
    }
    let new_block = match new_block {
        Some(block) => block,
        None => ctx.CreateDefaultParagraphElement()?,
    };
    let mut previous = ctx.PreviousSibling(&item);
    while previous.is_some_and(|node| !ctx.IsElement(node)) {
        previous = ctx.PreviousSibling(&previous.unwrap());
    }
    let mut next = ctx.NextSibling(&item);
    while next.is_some_and(|node| !ctx.IsElement(node)) {
        next = ctx.NextSibling(&next.unwrap());
    }
    if next.is_some() {
        if previous.is_some_and(|node| ctx.Tag(node, "li") || IsList(ctx, node)) {
            ctx.SplitElement(list, item)?;
        }
        ctx.InsertBefore(new_block, list)?;
        ctx.Remove(item)?;
    } else {
        InsertAfter(ctx, new_block, list)?;
        ctx.Remove(if previous.is_some() { item } else { list })?;
    }
    AppendPlaceholder(ctx, new_block)?;
    let mut ending = Position::FirstPositionInNode(new_block);
    if let Some(style) = ctx.PrepareStyle(&style, &ending)? {
        ending = ctx.ApplyStyle(&style, &ending)?;
    }
    ctx.SetEndingPosition(&ending);
    Ok(Some(ending))
}

// Source composite_edit_command.cc:1580-1604,1840-1875.
pub(super) fn MoveParagraph<C: StructuralEditingContext>(
    ctx: &C,
    start: &Position<u64>,
    end: &Position<u64>,
    destination: &Position<u64>,
    source_block: u64,
) -> EditResult {
    MoveParagraphWithResult(ctx, start, end, destination, source_block).map(|_| ())
}
pub(super) fn MoveParagraphWithResult<C: StructuralEditingContext>(
    ctx: &C,
    start: &Position<u64>,
    end: &Position<u64>,
    destination: &Position<u64>,
    source_block: u64,
) -> EditResult<(Position<u64>, Position<u64>)> {
    // composite_edit_command.cc:1580-1604,1840-1875 preserves the
    // paragraph's inherited editing properties while moving its native
    // nodes out of their old block. Keep this separate from Editor's
    // pending typing style, which is recomputed after deletion.
    let moving_style = ctx.CaptureStyle(start, false)?;
    let destination = ctx.Track(destination)?;
    let mut start = ctx
        .QueryPosition(start, StructuralPositionQuery::Forward)
        .ToOffsetInAnchor(ctx);
    let end = end.ToOffsetInAnchor(ctx);
    let end_node = end.ComputeContainerNode(ctx).ok_or(EditingAborted)?;
    let end_offset = end.ComputeOffsetInContainerNode(ctx);
    // Split the trailing boundary before splitting the leading hierarchy.
    // SplitTextNode/Element keep the original owner on the right, so the
    // resulting sibling interval is stable while native children move.
    let mut past = if ctx.IsText(&end_node) {
        let length = ctx.CharacterDataLength(&end_node).ok_or(EditingAborted)?;
        if end_offset == 0 {
            Some(end_node)
        } else if end_offset < length {
            ctx.SplitText(end_node, end_offset)?;
            if start.ComputeContainerNode(ctx) == Some(end_node) {
                start = Position::new(
                    ctx.PreviousSibling(&end_node).ok_or(EditingAborted)?,
                    start.ComputeOffsetInContainerNode(ctx),
                );
            }
            Some(end_node)
        } else {
            text_offset_mapping::NextSkippingChildren(ctx, &end_node, Some(&source_block))
        }
    } else {
        end.ComputeNodeAfterPosition(ctx)
    };
    if let Some(after) = past {
        if Contains(ctx, source_block, after) && ctx.Parent(&after) != Some(source_block) {
            past = Some(SplitTreeToNode(ctx, after, source_block)?);
        }
    }
    let node = start.ComputeContainerNode(ctx).ok_or(EditingAborted)?;
    let first = if node == source_block {
        start.ComputeNodeAfterPosition(ctx)
    } else {
        let node = if ctx.IsText(&node) {
            let offset = start.ComputeOffsetInContainerNode(ctx);
            let length = ctx.CharacterDataLength(&node).ok_or(EditingAborted)?;
            if offset > 0 && offset < length {
                ctx.SplitText(node, offset)?;
                node
            } else if offset >= length {
                match text_offset_mapping::NextSkippingChildren(ctx, &node, Some(&source_block)) {
                    Some(next) => next,
                    None => {
                        let p = ctx.Tracked(&destination);
                        return Ok((p.clone(), p));
                    }
                }
            } else {
                node
            }
        } else {
            start.ComputeNodeAfterPosition(ctx).unwrap_or(node)
        };
        Some(SplitTreeToNode(ctx, node, source_block)?)
    };
    let mut moving = Vec::new();
    let mut node = first;
    while let Some(current) = node.filter(|current| Some(*current) != past) {
        moving.push(current);
        node = ctx.NextSibling(&current);
    }
    let mut insertion = ctx.Tracked(&destination);
    if let Some(style) = ctx.PrepareStyle(&moving_style, &insertion)? {
        insertion = ctx.ApplyStyle(&style, &insertion)?;
    }
    let mut inserted_start = None;
    for node in moving {
        ctx.Remove(node)?;
        InsertNodeAt(ctx, node, &insertion)?;
        if inserted_start.is_none() {
            inserted_start = Some(Position::FirstPositionInOrBeforeNode(ctx, &node));
        }
        insertion = ParentAfter(ctx, node);
    }
    if ctx.ChildCount(&source_block) == 0 && ctx.Parent(&source_block).is_some() {
        ctx.Remove(source_block)?;
    }
    Ok((
        inserted_start.unwrap_or_else(|| insertion.clone()),
        insertion,
    ))
}

// Source composite_edit_command.cc:106-117,757-866.
use crate::core::editing::editing_utilities::{IsWhitespace, StringWithRebalancedWhitespace};

pub trait WhitespaceRebalanceContext {
    type Node;
    type Position;
    type VisiblePosition;
    type Affinity: Copy;
    fn UpdateStyleAndLayout(&mut self);
    fn ContainerNode(&self, position: &Self::Position) -> Option<Self::Node>;
    fn AnchorType(&self, position: &Self::Position) -> PositionAnchorType;
    fn IsText(&self, node: &Self::Node) -> bool;
    fn TextLength(&self, node: &Self::Node) -> usize;
    fn ShouldPreserveWhiteSpaces(&self, node: &Self::Node) -> bool;
    fn EditingOffset(&self, position: &Self::Position) -> usize;
    fn TextCodeUnit(&self, node: &Self::Node, offset: usize) -> Option<u16>;
    fn OffsetInNode(&self, node: &Self::Node, offset: usize) -> Self::Position;
    fn DefaultAffinity(&self) -> Self::Affinity;
    fn CreateVisiblePosition(
        &self,
        position: &Self::Position,
        affinity: Self::Affinity,
    ) -> Option<Self::VisiblePosition>;
    fn IsStartOfParagraph(&self, position: &Self::VisiblePosition) -> bool;
    fn IsEndOfParagraph(&self, position: &Self::VisiblePosition) -> bool;
    fn NextSibling(&self, node: &Self::Node) -> Option<Self::Node>;
    fn ReplaceTextInNode(
        &mut self,
        node: &Self::Node,
        offset: usize,
        count: usize,
        data: &[u16],
    ) -> EditResult;
    fn IsRichlyEditable(&self, node: &Self::Node) -> bool;
    fn HasLayoutObject(&self, node: &Self::Node) -> bool;
    fn ShouldCollapseBreaks(&self, node: &Self::Node) -> bool;
}
pub fn CanRebalance<H: WhitespaceRebalanceContext>(host: &mut H, position: &H::Position) -> bool {
    host.UpdateStyleAndLayout();
    let Some(node) = host.ContainerNode(position) else {
        return false;
    };
    host.AnchorType(position) == PositionAnchorType::OffsetInAnchor
        && host.IsText(&node)
        && host.IsRichlyEditable(&node)
        && host.TextLength(&node) != 0
        && (!host.HasLayoutObject(&node) || !host.ShouldPreserveWhiteSpaces(&node))
}
pub fn RebalanceWhitespaceAtWithContext<H: WhitespaceRebalanceContext>(
    host: &mut H,
    position: &H::Position,
) -> EditResult {
    if !CanRebalance(host, position) {
        return Ok(());
    }
    let node = host
        .ContainerNode(position)
        .expect("CanRebalance requires text");
    let offset = host.EditingOffset(position);
    if !host.TextCodeUnit(&node, offset).is_some_and(IsWhitespace)
        && !offset
            .checked_sub(1)
            .and_then(|offset| host.TextCodeUnit(&node, offset))
            .is_some_and(IsWhitespace)
    {
        return Ok(());
    }
    RebalanceWhitespaceOnTextSubstring(host, &node, offset, offset)
}
pub fn RebalanceWhitespaceOnTextSubstring<H: WhitespaceRebalanceContext>(
    host: &mut H,
    node: &H::Node,
    start: usize,
    end: usize,
) -> EditResult {
    let whitespace = |host: &H, offset| {
        host.TextCodeUnit(node, offset).is_some_and(|unit| {
            IsWhitespace(unit)
                && (unit != 10 || !host.HasLayoutObject(node) || host.ShouldCollapseBreaks(node))
        })
    };
    let mut upstream = start;
    while upstream > 0 && whitespace(host, upstream - 1) {
        upstream -= 1;
    }
    let mut downstream = end;
    let text_length = host.TextLength(node);
    while downstream < text_length && whitespace(host, downstream) {
        downstream += 1;
    }
    if upstream == downstream {
        return Ok(());
    }
    let original = (upstream..downstream)
        .filter_map(|offset| host.TextCodeUnit(node, offset))
        .collect::<Vec<_>>();
    host.UpdateStyleAndLayout();
    let start_pos = host.OffsetInNode(node, upstream);
    let end_pos = host.OffsetInNode(node, downstream);
    let start_visible = host.CreateVisiblePosition(&start_pos, host.DefaultAffinity());
    let end_visible = host.CreateVisiblePosition(&end_pos, host.DefaultAffinity());
    let at_start = start_visible
        .as_ref()
        .is_some_and(|position| host.IsStartOfParagraph(position));
    let at_end = end_visible
        .as_ref()
        .is_some_and(|position| host.IsEndOfParagraph(position));
    let next_is_nonwhitespace_text = host.NextSibling(node).is_some_and(|node| {
        host.IsText(&node)
            && host.TextLength(&node) != 0
            && host
                .TextCodeUnit(&node, 0)
                .is_some_and(|unit| !IsWhitespace(unit))
    });
    let rebalanced = StringWithRebalancedWhitespace(
        &original,
        at_start || upstream == 0,
        (at_end || downstream == text_length) && !next_is_nonwhitespace_text,
    );
    if original != rebalanced {
        host.ReplaceTextInNode(node, upstream, downstream - upstream, &rebalanced)?;
    }
    Ok(())
}

// Source composite_edit_command.h:233-239,282-337; .cc:1598-1990.
// Serialized paragraph movement preserves native fragment identity and order.
use super::{
    apply_style_command::{ApplyStyleQuery, ApplyStyleResult, StyleExtraction},
    delete_selection_command::DeleteSelectionCommand,
    insert_list_command::{ListEditingContext, Ordered},
    paragraph_cloning::{self, AtEdge},
    replace_selection_command::{
        ReplaceSelectionCommand, ReplaceSelectionOptions, ReplacementFragment,
    },
};

/// Only the native serializer/parser and constructor's synchronous event ports
/// cross this boundary. The source command owns deletion, cleanup, paste order,
/// captured style, selection indices, and the same CompositeEditCommand.
pub trait ParagraphMoveHost {
    fn CreateFragment(
        &self,
        start: &Position<u64>,
        end: &Position<u64>,
        constraining_ancestor: Option<u64>,
    ) -> EditResult<Option<u64>>;
    fn PrepareFragment(&self, fragment: Option<u64>, root: u64) -> EditResult<ReplacementFragment>;
    fn RangeLength(&self, start: &Position<u64>, end: &Position<u64>) -> EditResult<usize>;
    fn PositionForIndex(&self, index: usize) -> Position<u64>;
    fn DocumentElement(&self) -> Option<u64>;
}
fn Track<C: ListEditingContext>(c: &C, p: Position<u64>) -> EditResult<Option<C::Tracked>> {
    if p.IsNull() {
        Ok(None)
    } else {
        c.Track(&p).map(Some)
    }
}
fn VisibleTracked<C: ListEditingContext>(c: &C, tracked: &Option<C::Tracked>) -> Position<u64> {
    tracked.as_ref().map_or_else(Position::default, |p| {
        c.QueryPosition(&c.Tracked(p), Q::Visible)
    })
}
/// :1840-1990, kPreserveSelection and caller-selected style policy.
pub fn MoveParagraphs<C: ListEditingContext, H: ParagraphMoveHost>(
    c: &C,
    host: &H,
    start: &Position<u64>,
    end: &Position<u64>,
    destination: &Position<u64>,
    preserve_style: bool,
    constraining_ancestor: Option<u64>,
) -> EditResult {
    let start = c.QueryPosition(start, Q::Visible);
    let end = c.QueryPosition(end, Q::Visible);
    let destination = c.QueryPosition(destination, Q::Visible);
    if start.IsNull() || start == destination {
        return Ok(());
    }
    if end.IsNull() || destination.IsNull() {
        return Err(EditingAborted);
    }
    if Ordered(c, &start, &destination) && Ordered(c, &destination, &end) {
        return Err(EditingAborted);
    }
    let selection = c.EndingRange();
    let selection = (
        c.QueryPosition(&selection.0, Q::Visible),
        c.QueryPosition(&selection.1, Q::Visible),
    );
    let indices = if selection.0.IsNull()
        || selection.1.IsNull()
        || c.Compare(&selection.0, &end) == Some(Ordering::Greater)
        || c.Compare(&selection.1, &start) == Some(Ordering::Less)
    {
        None
    } else {
        let start_index = if Ordered(c, &start, &selection.0) {
            host.RangeLength(
                &start.ParentAnchoredEquivalent(c),
                &selection.0.ParentAnchoredEquivalent(c),
            )?
        } else {
            0
        };
        let end_index = if Ordered(c, &selection.1, &end) {
            host.RangeLength(
                &start.ParentAnchoredEquivalent(c),
                &selection.1.ParentAnchoredEquivalent(c),
            )?
        } else {
            0
        };
        Some((start_index, end_index))
    };
    let before = Track(c, c.QueryPosition(&start, Q::PreviousWithinEditable))?;
    let after = Track(c, c.QueryPosition(&end, Q::NextWithinEditable))?;
    let empty = start == end;
    let (from, mut to) = if empty {
        (start.clone(), end.clone())
    } else {
        (
            c.QueryPosition(&start, Q::Forward),
            c.QueryPosition(&end, Q::Backward),
        )
    };
    if c.Compare(&from, &to) == Some(Ordering::Greater) {
        to = from.clone();
    }
    let fragment = if empty {
        None
    } else {
        host.CreateFragment(
            &from.ParentAnchoredEquivalent(c),
            &to.ParentAnchoredEquivalent(c),
            constraining_ancestor,
        )?
    };
    let empty_style = if empty && preserve_style {
        let style = c.MergeTypingStyle(c.CaptureStyle(&start, false)?)?;
        match c.ApplyStyleQuery(ApplyStyleQuery::Extract(style, StyleExtraction::Block))? {
            ApplyStyleResult::Extract { remaining, .. } => Some(remaining),
            _ => return Err(EditingAborted),
        }
    } else {
        None
    };
    c.SetEndingRange(&from, &to);
    let destination_visible = c.QueryPosition(&destination, Q::Visible);
    if destination_visible
        .ComputeContainerNode(c)
        .and_then(|n| c.RootEditable(n))
        .is_none()
        || c.EndingRange()
            .0
            .ComputeContainerNode(c)
            .and_then(|n| c.RootEditable(n))
            .is_none()
    {
        return Ok(());
    }
    let caret = DeleteSelectionCommand::Apply(c, (from, to))?;
    paragraph_cloning::CleanupAfterDeletion(c, &caret, Some(&destination))?;
    if !destination.AnchorNode().is_some_and(|n| c.IsConnected(n)) {
        return Err(EditingAborted);
    }
    c.UpdateStyleAndLayout();
    let before = VisibleTracked(c, &before);
    let after = VisibleTracked(c, &after);
    if !before.IsNull()
        && ((!AtEdge(c, &before, Q::ParagraphStart) && !AtEdge(c, &before, Q::ParagraphEnd))
            || before == after)
    {
        let br = c.CreateElement("br")?;
        super::composite_edit_command::InsertNodeAt(c, br, &before)?;
    }
    c.UpdateStyleAndLayout();
    let document_element = host.DocumentElement().ok_or(EditingAborted)?;
    let destination_index = host.RangeLength(
        &Position::FirstPositionInNode(document_element),
        &destination.ParentAnchoredEquivalent(c),
    )?;
    let destination = c.QueryPosition(&destination, Q::Visible);
    if destination.IsNull() {
        return Err(EditingAborted);
    }
    c.SetEndingRange(&destination, &destination);
    let root = destination
        .ComputeContainerNode(c)
        .and_then(|n| c.RootEditable(n))
        .ok_or(EditingAborted)?;
    let fragment = host.PrepareFragment(fragment, root)?;
    ReplaceSelectionCommand::new(ReplaceSelectionOptions {
        select_replacement: true,
        moving_paragraph: true,
        match_style: !preserve_style,
        ..Default::default()
    })
    .DoApply(c, &fragment, &destination, &destination)
    .map_err(|_| EditingAborted)?;
    c.UpdateStyleAndLayout();
    let ending = c.EndingRange();
    if let Some(style) = empty_style.filter(|_| {
        ending.0 == ending.1
            && AtEdge(c, &ending.0, Q::ParagraphStart)
            && AtEdge(c, &ending.0, Q::ParagraphEnd)
    }) {
        c.ApplyReplacementStyleRange(&style, &ending.0, &ending.1)?;
    }
    if let Some((start, end)) = indices {
        if host.DocumentElement().is_some() {
            c.UpdateStyleAndLayout();
            let start = host.PositionForIndex(destination_index.saturating_add(start));
            let end = host.PositionForIndex(destination_index.saturating_add(end));
            if !start.IsNull() && !end.IsNull() {
                c.SetEndingRange(&start, &end);
            }
        }
    }
    Ok(())
}
