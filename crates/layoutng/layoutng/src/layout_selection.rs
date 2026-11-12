//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Source: core/editing/layout_selection.{h,cc}. This is the derived paint
//! range owner. Canonical raw endpoints stay in editing_state::SelectionState.
#![allow(non_snake_case)]
use crate::{
    editing_state::DocumentSelection,
    internal::{
        layout_node_metadata::{Node, Text},
        layout_object::LayoutObject,
        layout_object_builder::LayoutObjectTree,
        layout_text::LayoutText,
        selection_state::SelectionState,
    },
    EditingPosition,
};
use layoutng_style::style::computed_style_constants::PseudoId;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectionPaintRange {
    pub start_node: Option<u64>,
    pub start_offset: Option<u32>,
    pub end_node: Option<u64>,
    pub end_offset: Option<u32>,
}
impl SelectionPaintRange {
    pub fn IsNull(&self) -> bool {
        self.start_node.is_none()
    }
}
/// A live flat-tree range iterator, borrowed only for this commit. Its nodes
/// are never retained as a second selection/document model.
pub struct LayoutSelectionRange<I> {
    pub nodes: I,
    pub start: EditingPosition,
    pub end: EditingPosition,
}
#[derive(Default)]
pub struct LayoutSelection {
    has_pending_selection: bool,
    paint_range: SelectionPaintRange,
    // Compatibility projection into the existing text Paint ABI. It is
    // published at the same commit as paint_range, never an editable selection.
    document_projection: Option<DocumentSelection>,
}
#[derive(Default)]
pub struct SelectionStyleAndPaintInvalidation {
    pub(crate) paint_objects: Vec<*mut LayoutObject>,
    /// Element::SetNeedsStyleRecalc(kLocalStyleChange), followed by
    /// PseudoStateChanged(kPseudoSelection), for exactly these selected nodes.
    pub selection_pseudo_elements: Vec<u64>,
}
pub(crate) struct SelectionCommit {
    pub committed: bool,
    pub paint_objects: Vec<*mut LayoutObject>,
}
fn ObjectForNode(tree: &LayoutObjectTree, id: u64) -> *mut LayoutObject {
    let Some(object) = tree.GetLayoutObjectForNode(id) else {
        return std::ptr::null_mut();
    };
    let Some(node) = (unsafe { object.GetNode().as_ref() }) else {
        return std::ptr::null_mut();
    };
    node.GetLayoutObject()
}
fn NodeId(object: &LayoutObject) -> Option<u64> {
    unsafe { object.GetNode().as_ref() }.map(Node::InputId)
}
// layout_selection.cc:345-375. The native attachment tree already omits
// display:contents nodes, which this source visitor never visits themselves.
// SelectionState::kContain retains the traversal path to selected leaves.
fn VisitSelectedInclusiveDescendantsOf(
    object: *mut LayoutObject,
    visitor: &mut impl FnMut(*mut LayoutObject),
) {
    let Some(owner) = (unsafe { object.as_ref() }) else {
        return;
    };
    if owner.GetSelectionState() == SelectionState::kNone {
        return;
    }
    let mut child = owner.SlowFirstChild();
    visitor(object);
    while let Some(owner) = unsafe { child.as_ref() } {
        let next = owner.NextSibling();
        VisitSelectedInclusiveDescendantsOf(child, visitor);
        child = next;
    }
}
fn FirstLetterPartFor(object: &LayoutObject) -> *mut LayoutObject {
    if !object.IsText() {
        return std::ptr::null_mut();
    }
    let text = unsafe { &*(object as *const LayoutObject).cast::<LayoutText>() };
    #[cfg(feature = "translation_in_progress")]
    if text.IsTextFragment() {
        return unsafe {
            &*(text as *const LayoutText).cast::<crate::layout_text_fragment::LayoutTextFragment>()
        }
        .GetFirstLetterPart()
        .cast();
    }
    text.GetFirstLetterPart().cast()
}
fn VisitLayoutObjectsOf(object: *mut LayoutObject, visitor: &mut impl FnMut(*mut LayoutObject)) {
    let owner = unsafe { &*object };
    if owner.GetSelectionState() == SelectionState::kContain {
        return;
    }
    let first = FirstLetterPartFor(owner);
    if !first.is_null() {
        visitor(first);
    }
    visitor(object);
}
fn SetShouldInvalidateIfNeeded(object: *mut LayoutObject, changed: &mut Vec<*mut LayoutObject>) {
    changed.push(object);
    let owner = unsafe { &mut *object };
    if owner.ShouldInvalidateSelection() {
        return;
    }
    owner.SetShouldInvalidateSelection();
    if !owner.IsSVG() {
        return;
    }
    let mut parent = owner.Parent();
    while let Some(owner) = unsafe { parent.as_mut() } {
        if owner.IsSVGRoot() {
            return;
        }
        if owner.IsSVGText() {
            if !owner.ShouldInvalidateSelection() {
                owner.SetShouldInvalidateSelection();
                changed.push(parent);
            }
            return;
        }
        parent = owner.Parent();
    }
}
fn InvalidateNode(tree: &LayoutObjectTree, node: u64, changed: &mut Vec<*mut LayoutObject>) {
    let object = ObjectForNode(tree, node);
    if object.is_null() {
        return;
    }
    let first = FirstLetterPartFor(unsafe { &*object });
    if !first.is_null() {
        SetShouldInvalidateIfNeeded(first, changed);
    }
    SetShouldInvalidateIfNeeded(object, changed);
}
fn SetSelectionStateIfNeeded(object: *mut LayoutObject, state: SelectionState) {
    let owner = unsafe { &mut *object };
    if owner.GetSelectionState() == state {
        return;
    }
    owner.SetSelectionState(state);
    let mut parent = owner.Parent();
    while let Some(owner) = unsafe { parent.as_mut() } {
        if owner.GetSelectionState() == SelectionState::kContain {
            return;
        }
        owner.SetSelectionState(SelectionState::kContain);
        parent = owner.Parent();
    }
}
fn TextOffset(object: &LayoutObject, boundary: &EditingPosition, start: bool) -> Option<u32> {
    let node = unsafe { object.GetNode().as_ref() }?;
    if !node.IsTextNode() {
        return None;
    }
    Some(if boundary.node_id == Some(node.InputId()) {
        boundary.offset.min(u32::MAX as usize) as u32
    } else if start {
        0
    } else {
        unsafe { &*object.GetNode().cast::<Text>() }.length()
    })
}
fn PaintOffset(object: &LayoutObject, offset: Option<u32>, start: bool) -> Option<u32> {
    if !object.IsText() {
        return None;
    }
    if !object.IsInLayoutNGInlineFormattingContext() {
        return offset;
    }
    #[cfg(feature = "translation_in_progress")]
    {
        let text = unsafe { &*(object as *const LayoutObject).cast::<LayoutText>() };
        let node = unsafe { object.GetNode().as_ref() }?;
        let position = if node.IsTextNode() {
            EditingPosition::new(node.InputId(), offset? as usize)
        } else {
            EditingPosition::WithAnchor(
                node.InputId(),
                0,
                if start {
                    crate::EditingPositionAnchorType::BeforeAnchor
                } else {
                    crate::EditingPositionAnchorType::AfterAnchor
                },
            )
        };
        let mapping = unsafe { text.GetOffsetMapping().as_ref() }?;
        return mapping.GetTextContentOffsetForNode(node, &position);
    }
    #[cfg(not(feature = "translation_in_progress"))]
    {
        let _ = start;
        offset
    }
}
/// Source LayoutSelectionStatus (inline text-content offsets, not DOM offsets).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InlineSelectionStatus {
    pub start: u32,
    pub end: u32,
    pub select_soft_line_break: bool,
}
impl LayoutSelection {
    pub fn SetHasPendingSelection(&mut self) {
        self.has_pending_selection = true;
    }
    pub fn HasPendingSelection(&self) -> bool {
        self.has_pending_selection
    }
    /// LayoutSelection::ComputeSelectionStatus(LayoutText), source cc:542-655.
    /// This range is local to the exact LayoutText, including first-letter parts.
    pub fn TextSelectionRange(&self, text: &LayoutText) -> std::ops::Range<u32> {
        use crate::internal::layout_pass_scope::LayoutObjectFactoryScope;
        let fragment = if text.IsTextFragment() {
            let factories = LayoutObjectFactoryScope::Objects();
            let read = unsafe { factories.as_ref() }
                .and_then(|f| f.text_fragment_selection_facts)
                .expect("source inline text-fragment selection facts installed");
            Some(read(text))
        } else {
            None
        };
        let state = fragment.map_or_else(|| text.GetSelectionState(), |f| f.state);
        if state == SelectionState::kNone {
            return 0..0;
        }
        let node_length = fragment.and_then(|f| f.node_length).or_else(|| {
            let node = unsafe { text.GetNode().as_ref()? };
            node.IsTextNode()
                .then(|| unsafe { &*(node as *const Node).cast::<Text>() }.length())
        });
        let Some(length) = node_length else {
            return if text.IsBR() && state == SelectionState::kEnd {
                0..0
            } else {
                0..text.TransformedTextLength()
            };
        };
        let range = self.PaintRange();
        // FixSelectionPaintRangeNullOpt is stable in the fixed source manifest.
        let selected = match state {
            SelectionState::kInside => 0..length,
            SelectionState::kStart => {
                let Some(start) = range.start_offset else {
                    return 0..0;
                };
                start..length
            }
            SelectionState::kEnd => {
                let Some(end) = range.end_offset else {
                    return 0..0;
                };
                0..end
            }
            SelectionState::kStartAndEnd => {
                let (Some(start), Some(end)) = (range.start_offset, range.end_offset) else {
                    return 0..0;
                };
                start..end
            }
            _ => unreachable!("source selected text has endpoint/interior selection state"),
        };
        if let Some(fragment) = fragment {
            let clamp = |offset: u32| offset.saturating_sub(fragment.start).min(fragment.length);
            clamp(selected.start)..clamp(selected.end)
        } else {
            selected
        }
    }
    /// layout_selection.cc:699-761. The resident InlineCursor supplies only
    /// its current text span and soft-line-break flag; all selection edges and
    /// first-letter state are read from this original paint-range owner.
    pub fn ComputeInlineSelectionStatus(
        &self,
        text: &LayoutText,
        start: u32,
        end: u32,
        before_soft_line_break: bool,
    ) -> InlineSelectionStatus {
        use crate::internal::layout_pass_scope::LayoutObjectFactoryScope;
        let state = if text.IsTextFragment() {
            let factories = LayoutObjectFactoryScope::Objects();
            let facts = unsafe { factories.as_ref() }
                .and_then(|f| f.text_fragment_selection_facts)
                .expect("source first-letter selection facts");
            facts(text).state
        } else {
            text.GetSelectionState()
        };
        let clamp = |offset: u32| offset.max(start).min(end);
        match state {
            SelectionState::kStart => {
                let Some(edge) = self.paint_range.start_offset else {
                    return InlineSelectionStatus::default();
                };
                InlineSelectionStatus {
                    start: clamp(edge),
                    end,
                    select_soft_line_break: edge <= end && before_soft_line_break,
                }
            }
            SelectionState::kEnd => {
                let Some(edge) = self.paint_range.end_offset else {
                    return InlineSelectionStatus::default();
                };
                InlineSelectionStatus {
                    start,
                    end: clamp(edge),
                    select_soft_line_break: end < edge && before_soft_line_break,
                }
            }
            SelectionState::kStartAndEnd => {
                let (Some(a), Some(b)) =
                    (self.paint_range.start_offset, self.paint_range.end_offset)
                else {
                    return InlineSelectionStatus::default();
                };
                InlineSelectionStatus {
                    start: clamp(a),
                    end: clamp(b),
                    select_soft_line_break: a <= end && end < b && before_soft_line_break,
                }
            }
            SelectionState::kInside => InlineSelectionStatus {
                start,
                end,
                select_soft_line_break: before_soft_line_break,
            },
            _ => InlineSelectionStatus::default(),
        }
    }
    /// Source layout_selection.cc:983-1024. The caller performs source Commit
    /// first; this traversal follows existing selected/contain bits only.
    pub fn AbsoluteSelectionBounds(&self, tree: &LayoutObjectTree) -> foundation::gfx::Rect {
        if self.paint_range.IsNull() {
            return foundation::gfx::Rect::default();
        }
        let mut selected = foundation::PhysicalRect::default();
        VisitSelectedInclusiveDescendantsOf(tree.Root() as *const _ as *mut _, &mut |object| {
            VisitLayoutObjectsOf(object, &mut |object| {
                let object = unsafe { &*object };
                if object.IsRooted() && object.CanUpdateSelectionOnRootLineBoxes() {
                    selected.Unite(&object.AbsoluteSelectionRect());
                }
            });
        });
        foundation::ToPixelSnappedRect(selected)
    }
    pub fn PaintRange(&self) -> SelectionPaintRange {
        self.paint_range
    }
    pub fn DocumentProjection(&self) -> Option<DocumentSelection> {
        self.document_projection
    }
    // layout_selection.cc:954-977. Reset old states, calculate/set the new
    // range, compare invalidation candidates, and only then publish paint_range.
    pub(crate) fn Commit<I: Iterator<Item = u64> + Clone>(
        &mut self,
        tree: &mut LayoutObjectTree,
        range: Option<LayoutSelectionRange<I>>,
        hidden: bool,
        projection: Option<DocumentSelection>,
    ) -> SelectionCommit {
        if !self.has_pending_selection {
            return SelectionCommit {
                committed: false,
                paint_objects: Vec::new(),
            };
        }
        self.has_pending_selection = false;
        let mut paint_objects = Vec::new();
        let old_range = self.paint_range;
        let mut old = HashMap::new();
        VisitSelectedInclusiveDescendantsOf(tree.RootMut(), &mut |object| {
            let owner = unsafe { &mut *object };
            let state = owner.GetSelectionState();
            owner.SetSelectionState(SelectionState::kNone);
            if state != SelectionState::kContain {
                if let Some(node) = NodeId(owner) {
                    old.insert(node, state);
                }
            }
        });
        let mut new_range = SelectionPaintRange::default();
        let mut selected = HashSet::new();
        if let Some(range) = range.filter(|_| !hidden) {
            let mut start = None;
            let mut end = None;
            for node in range.nodes.clone() {
                let object = ObjectForNode(tree, node);
                if object.is_null() || !unsafe { &*object }.CanBeSelectionLeaf() {
                    continue;
                }
                if start.is_none() {
                    start = Some(node);
                    end = Some(node);
                    continue;
                }
                if end != start {
                    let node = end.unwrap();
                    SetSelectionStateIfNeeded(ObjectForNode(tree, node), SelectionState::kInside);
                    selected.insert(node);
                }
                end = Some(node);
            }
            if let (Some(start), Some(end)) = (start, end) {
                let mut paint_start = None;
                let mut paint_end = None;
                // layout_selection.cc:834-882, preserving the separate pass
                // that skips LayoutText owners without FragmentItems.
                for node in range.nodes {
                    let object = ObjectForNode(tree, node);
                    if object.is_null() || !unsafe { &*object }.CanBeSelectionLeaf() {
                        continue;
                    }
                    if unsafe { &*object }.IsText()
                        && unsafe { &*object.cast::<LayoutText>() }.FirstInlineFragmentItemIndex()
                            == 0
                    {
                        continue;
                    }
                    if paint_start.is_none() {
                        paint_start = Some(node);
                        paint_end = Some(node);
                        continue;
                    }
                    if paint_end != paint_start {
                        unsafe { &mut *ObjectForNode(tree, paint_end.unwrap()) }
                            .SetSelectionStateForPaint(SelectionState::kInside);
                    }
                    paint_end = Some(node);
                }
                if let (Some(ps), Some(pe)) = (paint_start, paint_end) {
                    unsafe { &mut *ObjectForNode(tree, ps) }.SetSelectionStateForPaint(
                        if ps == pe {
                            SelectionState::kStartAndEnd
                        } else {
                            SelectionState::kStart
                        },
                    );
                    if ps != pe {
                        unsafe { &mut *ObjectForNode(tree, pe) }
                            .SetSelectionStateForPaint(SelectionState::kEnd);
                    }
                }
                SetSelectionStateIfNeeded(
                    ObjectForNode(tree, start),
                    if start == end {
                        SelectionState::kStartAndEnd
                    } else {
                        SelectionState::kStart
                    },
                );
                selected.insert(start);
                if start != end {
                    SetSelectionStateIfNeeded(ObjectForNode(tree, end), SelectionState::kEnd);
                    selected.insert(end);
                }
                let start_object = unsafe { &*ObjectForNode(tree, start) };
                let end_object = unsafe { &*ObjectForNode(tree, end) };
                new_range = SelectionPaintRange {
                    start_node: Some(start),
                    start_offset: PaintOffset(
                        start_object,
                        TextOffset(start_object, &range.start, true),
                        true,
                    ),
                    end_node: Some(end),
                    end_offset: PaintOffset(
                        end_object,
                        TextOffset(end_object, &range.end, false),
                        false,
                    ),
                };
            }
        }
        // layout_selection.cc:289-334. State changes and moving text edges
        // invalidate exactly their source display-item clients.
        for node in selected {
            if !old.contains_key(&node) {
                InvalidateNode(tree, node, &mut paint_objects);
            }
        }
        for (node, old_state) in old {
            let object = ObjectForNode(tree, node);
            if !object.is_null() && unsafe { &*object }.GetSelectionState() != old_state {
                InvalidateNode(tree, node, &mut paint_objects);
            }
        }
        for (node, old_node, offset, old_offset) in [
            (
                new_range.start_node,
                old_range.start_node,
                new_range.start_offset,
                old_range.start_offset,
            ),
            (
                new_range.end_node,
                old_range.end_node,
                new_range.end_offset,
                old_range.end_offset,
            ),
        ] {
            if node.is_some() && node == old_node && offset != old_offset {
                let node = node.unwrap();
                let object = ObjectForNode(tree, node);
                if !object.is_null() && unsafe { &*(*object).GetNode() }.IsTextNode() {
                    InvalidateNode(tree, node, &mut paint_objects);
                }
            }
        }
        self.paint_range = new_range;
        self.document_projection = (!new_range.IsNull()).then_some(projection).flatten();
        SelectionCommit {
            committed: true,
            paint_objects,
        }
    }
    // layout_selection.cc:1026-1063. This does not Commit(): focus invalidates
    // the currently committed selected subtree and its ::selection styles.
    pub fn InvalidateStyleAndPaintForSelection(
        &self,
        tree: &mut LayoutObjectTree,
    ) -> SelectionStyleAndPaintInvalidation {
        let mut effect = SelectionStyleAndPaintInvalidation::default();
        if self.paint_range.IsNull() {
            return effect;
        }
        VisitSelectedInclusiveDescendantsOf(tree.RootMut(), &mut |object| {
            let owner = unsafe { &*object };
            if let Some(node) = unsafe { owner.GetNode().as_ref() } {
                if node.IsElementNode()
                    && owner
                        .StyleRef()
                        .HasPseudoElementStyle(PseudoId::kPseudoIdSelection)
                {
                    effect.selection_pseudo_elements.push(node.InputId());
                }
            }
            VisitLayoutObjectsOf(object, &mut |object| {
                unsafe { &mut *object }.SetShouldInvalidateSelection();
                effect.paint_objects.push(object);
            });
        });
        effect
    }
    pub fn ContextDestroyed(&mut self) {
        self.has_pending_selection = false;
        self.paint_range = Default::default();
        self.document_projection = None;
    }
}

/// Borrowed flat-tree facts for Position::NodeAsRangeFirst/PastLastNode and
/// RangeTraversal. Only the selected interval is traversed, never a DOM census.
pub trait SelectionFlatTree {
    fn Parent(&self, node: u64) -> Option<u64>;
    fn FirstChild(&self, node: u64) -> Option<u64>;
    fn NextSibling(&self, node: u64) -> Option<u64>;
    fn ChildAt(&self, node: u64, offset: usize) -> Option<u64>;
    fn IsCharacterData(&self, node: u64) -> bool;
}
fn NextSkippingChildren(tree: &impl SelectionFlatTree, mut node: u64) -> Option<u64> {
    loop {
        if let Some(next) = tree.NextSibling(node) {
            return Some(next);
        }
        node = tree.Parent(node)?;
    }
}
pub struct FlatRangeNodes<'a, T> {
    tree: &'a T,
    next: Option<u64>,
    past_last: Option<u64>,
}
impl<T> Clone for FlatRangeNodes<'_, T> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            next: self.next,
            past_last: self.past_last,
        }
    }
}
impl<'a, T: SelectionFlatTree> FlatRangeNodes<'a, T> {
    // core/editing/position.cc:311-350, ephemeral_range.cc::Nodes. The
    // embedding supplies source ToOffsetInAnchor flat-tree boundary values.
    pub fn new(tree: &'a T, start: EditingPosition, end: EditingPosition) -> Self {
        debug_assert_eq!(
            start.anchor,
            crate::EditingPositionAnchorType::OffsetInAnchor
        );
        debug_assert_eq!(end.anchor, crate::EditingPositionAnchorType::OffsetInAnchor);
        let first = start.node_id.and_then(|node| {
            if tree.IsCharacterData(node) {
                Some(node)
            } else {
                tree.ChildAt(node, start.offset).or_else(|| {
                    if start.offset == 0 {
                        Some(node)
                    } else {
                        NextSkippingChildren(tree, node)
                    }
                })
            }
        });
        let past_last = end.node_id.and_then(|node| {
            if tree.IsCharacterData(node) {
                NextSkippingChildren(tree, node)
            } else {
                tree.ChildAt(node, end.offset)
                    .or_else(|| NextSkippingChildren(tree, node))
            }
        });
        Self {
            tree,
            next: first,
            past_last,
        }
    }
}
impl<T: SelectionFlatTree> Iterator for FlatRangeNodes<'_, T> {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let current = self.next?;
        if Some(current) == self.past_last {
            self.next = None;
            return None;
        }
        self.next = self
            .tree
            .FirstChild(current)
            .or_else(|| NextSkippingChildren(self.tree, current));
        Some(current)
    }
}

use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
