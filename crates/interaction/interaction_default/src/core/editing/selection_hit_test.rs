// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: editing_utilities.cc:1344-1435.
use super::{
    editing_boundaries::EditableBoundaryContext,
    position::{Position, PositionWithAffinity},
};
pub trait SelectionHitTestContext<H>: EditableBoundaryContext {
    fn TargetNode(&self, hit: &H) -> Option<Self::Node>;
    fn HasLayoutObject(&self, node: &Self::Node) -> bool;
    fn IsSelectable(&self, node: &Self::Node) -> bool;
    fn TextControlForPosition(&self, position: &Position<Self::Node>) -> Option<Self::Node>;
    fn PlaceholderElement(&self, control: &Self::Node) -> Option<Self::Node>;
    fn InnerEditorElement(&self, control: &Self::Node) -> Option<Self::Node>;
    fn HaveSameOutOfFlowAncestor(&self, a: &Self::Node, b: &Self::Node) -> bool;
    fn VisibleToHitTesting(&self, node: &Self::Node) -> bool;
    fn HitPosition(&self, hit: &H) -> PositionWithAffinity<Self::Node>;
    fn PositionForPointInContainBoundary(
        &self,
        hit: &H,
        target: &Self::Node,
        editable: &Self::Node,
    ) -> PositionWithAffinity<Self::Node>;
    fn NoExtendSelectionToUserSelectNoneOutOfFlowEnabled(&self) -> bool;
    fn NoExtendSelectionToUserSelectNoneOutOfFlowUnlessEditableEnabled(&self) -> bool;
}
pub fn UserSelectContainBoundaryOf<C: SelectionHitTestContext<H>, H>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Option<C::Node> {
    if let Some(control) = ctx.TextControlForPosition(position) {
        if ctx
            .PlaceholderElement(&control)
            .as_ref()
            .zip(position.ComputeContainerNode(ctx).as_ref())
            .is_some_and(|(placeholder, container)| ctx.DOMContains(placeholder, container))
        {
            return None;
        }
        return ctx.InnerEditorElement(&control);
    }
    ctx.RootEditableElementOf(position)
}
pub fn PositionRespectingEditingBoundary<C: SelectionHitTestContext<H>, H>(
    ctx: &C,
    position: &Position<C::Node>,
    hit: &H,
) -> PositionWithAffinity<C::Node> {
    let Some(target) = ctx.TargetNode(hit) else {
        return PositionWithAffinity::default();
    };
    if !SelectionHitTestContext::HasLayoutObject(ctx, &target) {
        return PositionWithAffinity::default();
    }
    let editable = UserSelectContainBoundaryOf(ctx, position);
    if (editable.is_none()
        || !ctx.NoExtendSelectionToUserSelectNoneOutOfFlowUnlessEditableEnabled())
        && ctx.NoExtendSelectionToUserSelectNoneOutOfFlowEnabled()
        && !position.IsNull()
        && !SelectionHitTestContext::IsSelectable(ctx, &target)
        && !ctx.HaveSameOutOfFlowAncestor(position.AnchorNode().unwrap(), &target)
    {
        return PositionWithAffinity::default();
    }
    if editable
        .as_ref()
        .is_none_or(|editable| ctx.DOMContains(editable, &target))
    {
        return ctx.HitPosition(hit);
    }
    let editable = editable.unwrap();
    if !SelectionHitTestContext::HasLayoutObject(ctx, &editable)
        || !ctx.VisibleToHitTesting(&editable)
    {
        return PositionWithAffinity::default();
    }
    ctx.PositionForPointInContainBoundary(hit, &target, &editable)
}

/// Source selection_controller.cc:1360-1370. Spellcheck's enablement sees the
/// original DOM parent-anchored position; marker lookup then uses flat strategy.
pub fn HitTestResultIsMisspelled<D, F>(
    dom: &D,
    flat: &F,
    markers: &super::markers::document_marker_controller::DocumentMarkerController<D::Node>,
    position: &super::position::Position<D::Node>,
    page_default: bool,
) -> bool
where
    D: super::spellcheck::spell_checker::SpellCheckingContext
        + super::position_conversion::PositionTreeScope,
    D::Node: Eq + std::hash::Hash + 'static,
    F: super::iterators::text_iterator::TextIteratorContext<Node = D::Node>
        + super::text_offset_mapping::InlineContentsContext
        + super::position_units_word::WordBreakContext
        + super::editing_boundaries::EditableBoundaryContext,
{
    if position.IsNull() {
        return false;
    }
    let position = position.ParentAnchoredEquivalent(dom);
    if !super::spellcheck::spell_checker::IsSpellCheckingEnabledAt(dom, &position, page_default) {
        return false;
    }
    let position = super::position_conversion::ToPositionInFlatTree(dom, flat, &position);
    markers
        .FirstMarkerGroupAroundPosition(
            flat,
            &position,
            super::markers::document_marker_controller::MarkerTypes::Misspelling(),
        )
        .is_some()
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MarkerAnnotationType {
    Glic,
    SharedHighlight,
}
/// Source annotation_agent_impl.cc:199-215,468-484, with source Glic priority.
pub fn AnnotationAtHitPosition<D, F>(
    dom: &D,
    flat: &F,
    markers: &super::markers::document_marker_controller::DocumentMarkerController<D::Node>,
    has_inner_node: bool,
    has_frame: bool,
    position: &super::position::Position<D::Node>,
) -> Option<MarkerAnnotationType>
where
    D: super::position_conversion::PositionTreeScope,
    D::Node: Eq + std::hash::Hash + 'static,
    F: super::iterators::text_iterator::TextIteratorContext<Node = D::Node>
        + super::text_offset_mapping::InlineContentsContext
        + super::position_units_word::WordBreakContext
        + super::editing_boundaries::EditableBoundaryContext,
{
    use super::markers::document_marker_controller::{MarkerType, MarkerTypes};
    if !has_inner_node || !has_frame {
        return None;
    }
    let position = super::position_conversion::ToPositionInFlatTree(dom, flat, position);
    if !markers
        .MarkersAroundPosition(flat, &position, MarkerTypes::Single(MarkerType::Glic))
        .is_empty()
    {
        return Some(MarkerAnnotationType::Glic);
    }
    if !markers
        .MarkersAroundPosition(
            flat,
            &position,
            MarkerTypes::Single(MarkerType::TextFragment),
        )
        .is_empty()
    {
        return Some(MarkerAnnotationType::SharedHighlight);
    }
    None
}
