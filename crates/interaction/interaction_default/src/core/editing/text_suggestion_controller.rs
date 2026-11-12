//! Chromium suggestion/text_suggestion_controller.cc:78-103,215-254,565-599.
use super::{
    ephemeral_range::EphemeralRange,
    markers::document_marker_controller::{
        DocumentMarker, DocumentMarkerController, MarkerType, MarkerTypes,
    },
    position::{Position, PositionTree},
    position_iterator::{NextGraphemePosition, PositionIteratorContext, PreviousGraphemePosition},
};
use std::{cell::Cell, hash::Hash, rc::Rc};

#[derive(Default)]
pub struct TextSuggestionControllerState {
    pub is_suggestion_menu_open: Cell<bool>,
}
pub trait TextSuggestionHost {
    fn StartSuggestionMenuTimer(&self) -> Result<(), &'static str>;
}
pub fn ComputeRangeSurroundingCaret<C: PositionIteratorContext>(
    context: &C,
    caret: &Position<C::Node>,
) -> EphemeralRange<C::Node> {
    let offset = caret.ComputeOffsetInContainerNode(context);
    if let Some(text) = caret
        .ComputeContainerNode(context)
        .filter(|n| context.IsText(n))
    {
        if offset != 0 && Some(offset) != context.CharacterDataLength(&text) {
            return EphemeralRange::new(
                Position::new(text.clone(), offset - 1),
                Position::new(text, offset + 1),
            );
        }
    }
    let previous = PreviousGraphemePosition(context, caret);
    let next = NextGraphemePosition(context, caret);
    EphemeralRange::new(
        if previous.IsNull() {
            caret.clone()
        } else {
            previous
        },
        if next.IsNull() { caret.clone() } else { next },
    )
}
pub fn FirstMarkerIntersectingRange<C: PositionIteratorContext>(
    context: &C,
    markers: &DocumentMarkerController<C::Node>,
    range: &EphemeralRange<C::Node>,
    types: MarkerTypes,
) -> Option<(C::Node, Rc<DocumentMarker>)>
where
    C::Node: Eq + Hash + 'static,
{
    let start = range.StartPosition();
    let end = range.EndPosition();
    let start_node = start.ComputeContainerNode(context);
    let end_node = end.ComputeContainerNode(context);
    for node in range.Nodes(context).filter(|n| context.IsText(n)) {
        let a = if Some(&node) == start_node.as_ref() {
            start.ComputeOffsetInContainerNode(context)
        } else {
            0
        };
        let b = if Some(&node) == end_node.as_ref() {
            end.ComputeOffsetInContainerNode(context)
        } else {
            context.CharacterDataLength(&node).unwrap_or(0)
        };
        if let Some(marker) =
            markers.FirstMarkerIntersectingOffsetRange(context, &node, a, b, types)
        {
            return Some((node, marker));
        }
    }
    None
}
pub fn HasPotentialSuggestionTap<C: PositionIteratorContext>(
    context: &C,
    markers: &DocumentMarkerController<C::Node>,
    caret: &Position<C::Node>,
) -> bool
where
    C::Node: Eq + Hash + 'static,
{
    let types = MarkerTypes::Misspelling().Union(MarkerTypes::Single(MarkerType::Suggestion));
    let range = ComputeRangeSurroundingCaret(context, caret);
    FirstMarkerIntersectingRange(context, markers, &range, types).is_some_and(|(_, marker)| {
        marker
            .Suggestion()
            .is_none_or(|s| !s.Suggestions().is_empty())
    })
}
