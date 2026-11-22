//! Read-only resident geometry leaves for ApplyStyleCommand. TextIterator's
//! original selection-indexing behavior is shared in both index directions.
use super::*;
use interaction_default::core::editing::{
    editing_boundaries,
    iterators::{
        text_iterator::{RangeLength, TextIterator},
        text_iterator_behavior::TextIteratorBehavior,
    },
};
#[derive(Clone, Copy)]
pub enum ApplyStyleGeometryQuery {
    HasRenderedDescendants(u64),
    TextIndex {
        root: u64,
        position: EditingPosition,
    },
    PositionAtIndex {
        root: u64,
        index: usize,
    },
    VisiblyContained {
        node: u64,
        start: EditingPosition,
        end: EditingPosition,
    },
    DistinctCandidate {
        position: EditingPosition,
        forward: bool,
    },
}
pub enum ApplyStyleGeometryResult {
    Index(usize),
    Position(Option<EditingPosition>),
    Bool(bool),
}
fn Source(p: EditingPosition) -> Position<u64> {
    SourcePosition(EditingPositionWithAffinity {
        position: p,
        affinity: EditingTextAffinity::Downstream,
    })
    .position
}
fn Native(p: &Position<u64>) -> Option<EditingPosition> {
    NativePosition(p, TextAffinity::Downstream).map(|p| p.position)
}
fn Compare(
    c: &EditingPositionContext<'_>,
    a: &Position<u64>,
    b: &Position<u64>,
) -> Option<std::cmp::Ordering> {
    compare_positions::CompareDOMPositions(c, a, b)
}
pub(super) fn Query(
    document: &Document,
    layout: LayoutOutput<'_>,
    query: ApplyStyleGeometryQuery,
) -> Option<ApplyStyleGeometryResult> {
    use ApplyStyleGeometryQuery as Q;
    use ApplyStyleGeometryResult as R;
    let c = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    Some(match query {
        Q::HasRenderedDescendants(node) => R::Bool(
            c.Layout(&node)
                .is_some_and(|n| n.HasRenderedNonAnonymousDescendantsWithHeight()),
        ),
        Q::TextIndex { root, position } => R::Index(RangeLength(
            &c,
            &Position::FirstPositionInNode(root),
            &Source(position),
            TextIteratorBehavior::AllVisiblePositionsIncludingShadowRootRangeLengthBehavior(),
        )),
        Q::PositionAtIndex { root, index } => {
            let start = Position::FirstPositionInNode(root);
            let end = Position::LastPositionInNode(&c, &root);
            let mut iterator = TextIterator::new(
                &c,
                &start,
                &end,
                TextIteratorBehavior::AllVisiblePositionsIncludingShadowRootRangeLengthBehavior(),
            );
            if index == 0 && iterator.AtEnd() {
                return Some(R::Position(Native(&Position::new(
                    iterator.CurrentContainer(),
                    0,
                ))));
            }
            let mut offset = 0;
            let mut result = None;
            while !iterator.AtEnd() {
                let length = iterator.length();
                let first = iterator
                    .StartPositionInCurrentContainer()
                    .ToOffsetInAnchor(&c);
                let mut last = iterator
                    .EndPositionInCurrentContainer()
                    .ToOffsetInAnchor(&c);
                if index >= offset && index <= offset + length {
                    if length == 1
                        && (matches!(iterator.CharacterAt(0), 10 | 9)
                            || iterator.IsInsideAtomicInlineElement())
                    {
                        iterator.Advance();
                        if !iterator.AtEnd() {
                            last = iterator.StartPositionInCurrentContainer();
                        } else {
                            let next = visible_navigation::NextPositionOf(
                                &c,
                                &VisiblePosition::Create(
                                    &c,
                                    PositionWithAffinity {
                                        position: first.clone(),
                                        affinity: TextAffinity::Downstream,
                                    },
                                )
                                .ToPositionWithAffinity(),
                                Rule::CannotCross,
                            );
                            if !next.IsNull() {
                                last = next.DeepEquivalent().clone();
                            }
                        }
                    }
                    let p = if first.ComputeContainerNode(&c).is_some_and(|n| c.IsText(&n)) {
                        Position::new(
                            first.ComputeContainerNode(&c)?,
                            first.ComputeOffsetInContainerNode(&c) + index - offset,
                        )
                    } else if index == offset {
                        first
                    } else {
                        last
                    };
                    result = Native(&p);
                    break;
                }
                offset += length;
                iterator.Advance();
            }
            R::Position(result)
        }
        Q::DistinctCandidate { position, forward } => {
            let p = Source(position);
            let p = if forward {
                editing_boundaries::NextVisuallyDistinctCandidate(&c, &p)
            } else {
                editing_boundaries::PreviousVisuallyDistinctCandidate(&c, &p)
            };
            R::Position(Native(&p))
        }
        Q::VisiblyContained { node, start, end } => {
            let start = Source(start);
            let end = Source(end);
            let before = Position::BeforeNode(node).ToOffsetInAnchor(&c);
            let after = Position::AfterNode(node).ToOffsetInAnchor(&c);
            let fully = Compare(&c, &start, &before)? != std::cmp::Ordering::Greater
                && Compare(&c, &after, &end)? != std::cmp::Ordering::Greater;
            let visible = |p: &Position<u64>| {
                VisiblePosition::Create(
                    &c,
                    PositionWithAffinity {
                        position: p.clone(),
                        affinity: TextAffinity::Downstream,
                    },
                )
                .DeepEquivalent()
                .clone()
            };
            let same_start = visible(&before) == visible(&start);
            let same_end = visible(&after) == visible(&end);
            R::Bool(
                fully
                    || same_start && Compare(&c, &after, &end)? == std::cmp::Ordering::Less
                    || same_end && Compare(&c, &start, &before)? == std::cmp::Ordering::Less
                    || same_start && same_end,
            )
        }
    })
}
