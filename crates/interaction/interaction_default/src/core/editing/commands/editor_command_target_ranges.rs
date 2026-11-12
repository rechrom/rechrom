// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source editor_command.cc:221-259; editing_utilities.cc:761-774,846-868.
use super::editor_command_input_type::SelectionModifyDirection;
use crate::core::editing::{
    backspace_state_machine::PreviousBackwardDeletionOffset,
    editing_boundaries::EditableBoundaryContext,
    frame_selection::SelectionValue,
    position::{Position, PositionDOM, PositionTree, TextAffinity},
    selection_modifier::{
        NoXPosForVerticalArrowNavigation, SelectionModifier, SelectionModifierContext,
        SelectionModifyAlteration,
    },
    text_granularity::TextGranularity,
    visible_selection::VisibleSelection,
};
use interaction_dispatch::core::events::input_event::StaticRangeRef;
use std::rc::Rc;
pub trait BackwardDeletionContext: PositionDOM + EditableBoundaryContext {
    /// Retain the canonical CharacterData UTF-16 owner, without flattening DOM.
    fn CharacterDataUTF16ForDeletion(&self, node: &Self::Node) -> Rc<[u16]>;
}
pub fn PreviousBackwardDeletionPosition<C: BackwardDeletionContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Position<C::Node> {
    position.PreviousWithOffset(ctx, |node, offset| {
        if offset <= 1 {
            return 0;
        }
        if !ctx.IsText(node) {
            return offset - 1;
        }
        PreviousBackwardDeletionOffset(&ctx.CharacterDataUTF16ForDeletion(node), offset)
    })
}
pub fn NarrowSelectionToBackwardDeletionUnit<C: BackwardDeletionContext>(
    ctx: &C,
    selection: &VisibleSelection<C::Node>,
) -> VisibleSelection<C::Node> {
    if selection.anchor.IsNull() || selection.anchor == selection.focus {
        return selection.clone();
    }
    if selection.anchor.ComputeContainerNode(ctx) != selection.focus.ComputeContainerNode(ctx) {
        return selection.clone();
    }
    let anchor_offset = selection.anchor.ComputeOffsetInContainerNode(ctx);
    let focus_offset = selection.focus.ComputeOffsetInContainerNode(ctx);
    // C++ subtraction is signed: a forward selection does not enter this path.
    if anchor_offset <= focus_offset || anchor_offset - focus_offset <= 1 {
        return selection.clone();
    }
    let end = if ctx.ComparePositions(&selection.anchor, &selection.focus)
        == Some(std::cmp::Ordering::Greater)
    {
        &selection.anchor
    } else {
        &selection.focus
    };
    VisibleSelection {
        anchor: end.clone(),
        focus: PreviousBackwardDeletionPosition(ctx, end),
        affinity: TextAffinity::Downstream,
    }
}
pub trait EditorCommandTargetRangeContext<S: SelectionValue> {
    type Geometry: SelectionModifierContext + BackwardDeletionContext;
    fn Geometry(&self) -> &Self::Geometry;
    fn DOMGeometry(&self) -> &Self::Geometry;
    fn ConvertRawDOMSelectionToFlat(
        &self,
        selection: &S,
    ) -> VisibleSelection<<Self::Geometry as PositionTree>::Node>;
    fn HasActiveEditContext(&self) -> bool;
    fn UpdateStyleAndLayoutForEditing(&self);
    fn CurrentSelection(&self) -> S;
    fn CurrentSelectionIsDirectional(&self) -> bool;
    fn StaticRangeForDOMSelection(
        &self,
        selection: &VisibleSelection<<Self::Geometry as PositionTree>::Node>,
    ) -> StaticRangeRef;
}
pub fn RangesFromCurrentSelectionOrExtendCaret<
    S: SelectionValue,
    C: EditorCommandTargetRangeContext<S>,
>(
    ctx: &C,
    direction: SelectionModifyDirection,
    granularity: TextGranularity,
) -> Option<Vec<StaticRangeRef>>
where
    C::Geometry: BackwardDeletionContext,
{
    if ctx.HasActiveEditContext() {
        return None;
    }
    ctx.UpdateStyleAndLayoutForEditing();
    let selection = ComputeTargetRangeSelection(
        ctx.Geometry(),
        ctx.DOMGeometry(),
        ctx.ConvertRawDOMSelectionToFlat(&ctx.CurrentSelection()),
        ctx.CurrentSelectionIsDirectional(),
        direction,
        granularity,
    );
    Some(
        selection
            .map(|selection| vec![ctx.StaticRangeForDOMSelection(&selection)])
            .unwrap_or_default(),
    )
}

/// The read-only selection stage of editor_command.cc:235-258. The modifier
/// owns flat-tree movement and its Selection() performs the source DOM-tree
/// conversion before backward-code-point narrowing. Native assembly may borrow
/// resident geometry here without implementing mutation or event-dispatch ports.
pub fn ComputeTargetRangeSelection<C>(
    geometry: &C,
    dom_geometry: &C,
    flat_selection: VisibleSelection<C::Node>,
    directional: bool,
    direction: SelectionModifyDirection,
    granularity: TextGranularity,
) -> Option<VisibleSelection<C::Node>>
where
    C: SelectionModifierContext + BackwardDeletionContext,
{
    let mut modifier = SelectionModifier::new(flat_selection, NoXPosForVerticalArrowNavigation());
    modifier.SetSelectionIsDirectional(directional);
    let before = modifier.Selection(geometry);
    let extended_caret = !before.anchor.IsNull() && before.anchor == before.focus;
    if extended_caret {
        modifier.Modify(
            geometry,
            SelectionModifyAlteration::Extend,
            direction,
            granularity,
        );
    }
    let mut selection = modifier.Selection(geometry);
    if selection.anchor.IsNull() {
        return None;
    }
    if foundation::RuntimeEnabledFeatures::TargetRangesForBackwardDeletionUnitEnabled()
        && extended_caret
        && direction == SelectionModifyDirection::Backward
        && granularity == TextGranularity::kCharacter
    {
        selection = NarrowSelectionToBackwardDeletionUnit(dom_geometry, &selection);
    }
    Some(selection)
}
