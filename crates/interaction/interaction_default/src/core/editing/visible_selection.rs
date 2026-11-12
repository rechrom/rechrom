// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: visible_selection.cc:58-107,240-275;
// selection_adjuster.cc:115-119,790-851. Character granularity does not expand.
use super::{
    editing_boundaries::EditableBoundaryContext,
    position::{Position, PositionWithAffinity, TextAffinity},
    selection_boundary_adjuster::{
        AdjustDOMFocusForEditingBoundary, AdjustDOMFocusForShadowBoundary,
    },
    visible_position::{CanonicalPositionOf, VisiblePosition, VisiblePositionContext},
};
use crate::core::editing::visible_position::DownstreamVisiblePositionContext;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct VisibleSelection<N> {
    pub anchor: Position<N>,
    pub focus: Position<N>,
    pub affinity: TextAffinity,
}
impl<N> Default for VisibleSelection<N> {
    fn default() -> Self {
        Self {
            anchor: Position::default(),
            focus: Position::default(),
            affinity: TextAffinity::Downstream,
        }
    }
}
/// The DOM overload used by Editor. Flat selection granularity/boundary
/// specialization is owned separately and is never replaced by this routine.
pub fn CreateVisibleSelection<C: VisiblePositionContext + EditableBoundaryContext>(
    ctx: &C,
    anchor: &Position<C::Node>,
    focus: &Position<C::Node>,
    affinity: TextAffinity,
) -> VisibleSelection<C::Node> {
    let mut result = CanonicalSelection(ctx, anchor, focus, affinity);
    if result.anchor.IsNull() {
        return result;
    }
    result.focus = AdjustDOMFocusForShadowBoundary(ctx, &result.anchor, &result.focus);
    result.focus = AdjustDOMFocusForEditingBoundary(ctx, &result.anchor, &result.focus);
    AdjustSelectionType(ctx, result)
}
fn CanonicalSelection<C: VisiblePositionContext>(
    ctx: &C,
    anchor: &Position<C::Node>,
    focus: &Position<C::Node>,
    affinity: TextAffinity,
) -> VisibleSelection<C::Node> {
    if anchor.IsNull() {
        return VisibleSelection::default();
    }
    let visible = |position: &Position<C::Node>| {
        VisiblePosition::Create(
            ctx,
            PositionWithAffinity {
                position: position.clone(),
                affinity,
            },
        )
        .DeepEquivalent()
        .clone()
    };
    let mut result = VisibleSelection {
        anchor: visible(anchor),
        focus: Position::default(),
        affinity,
    };
    result.focus = if anchor == focus {
        result.anchor.clone()
    } else {
        visible(focus)
    };
    if result.anchor.IsNull() {
        result.anchor = result.focus.clone();
    }
    if result.focus.IsNull() {
        result.focus = result.anchor.clone();
    }
    if result.anchor.IsNull() {
        return VisibleSelection::default();
    }
    result
}

/// DOM specialization of visible_selection.cc:67-117. Reuses the same
/// GranularityAdjuster and DOM shadow/editing boundary owners as character
/// canonicalization, rather than expanding a flattened string selection.
pub fn ExpandWithGranularity<C>(
    ctx: &C,
    selection: &VisibleSelection<C::Node>,
    granularity: super::text_granularity::TextGranularity,
) -> VisibleSelection<C::Node>
where
    C: super::selection_adjuster::GranularityContext + EditableBoundaryContext,
{
    let mut result =
        CanonicalSelection(ctx, &selection.anchor, &selection.focus, selection.affinity);
    if result.anchor.IsNull() {
        return result;
    }
    let forward = EditableBoundaryContext::ComparePositions(ctx, &result.anchor, &result.focus)
        != Some(Ordering::Greater);
    let (start, end) = if forward {
        (&result.anchor, &result.focus)
    } else {
        (&result.focus, &result.anchor)
    };
    let (start, end) = super::selection_adjuster::AdjustSelectionRespectingGranularity(
        ctx,
        start,
        end,
        selection.affinity,
        granularity,
        super::selection_adjuster::WordInclusion::Default,
        true,
    );
    let (anchor, focus) = if forward { (start, end) } else { (end, start) };
    result.anchor = anchor;
    result.focus = AdjustDOMFocusForShadowBoundary(ctx, &result.anchor, &focus);
    result.focus = AdjustDOMFocusForEditingBoundary(ctx, &result.anchor, &result.focus);
    result.affinity = selection.affinity;
    AdjustSelectionType(ctx, result)
}
fn AdjustSelectionType<C: VisiblePositionContext + EditableBoundaryContext>(
    ctx: &C,
    result: VisibleSelection<C::Node>,
) -> VisibleSelection<C::Node> {
    let affinity = result.affinity;
    let forward = ctx.ComparePositions(&result.anchor, &result.focus) != Some(Ordering::Greater);
    let (start, end) = if forward {
        (&result.anchor, &result.focus)
    } else {
        (&result.focus, &result.anchor)
    };
    if start == end
        || DownstreamVisiblePositionContext::MostBackwardCaretPosition(ctx, start)
            == DownstreamVisiblePositionContext::MostBackwardCaretPosition(ctx, end)
    {
        return VisibleSelection {
            anchor: start.clone(),
            focus: start.clone(),
            affinity,
        };
    }
    let mut minimum_start = DownstreamVisiblePositionContext::MostForwardCaretPosition(ctx, start);
    let mut minimum_end = DownstreamVisiblePositionContext::MostBackwardCaretPosition(ctx, end);
    if ctx.IsEditablePosition(&minimum_end) && CanonicalPositionOf(ctx, &minimum_start).IsNull() {
        minimum_start = start.clone();
    }
    if ctx.IsEditablePosition(&minimum_start) && CanonicalPositionOf(ctx, &minimum_end).IsNull() {
        minimum_end = end.clone();
    }
    if ctx.ComparePositions(&minimum_start, &minimum_end) == Some(Ordering::Greater) {
        minimum_start = start.clone();
        minimum_end = end.clone();
    }
    let collapse = minimum_start == minimum_end;
    let (anchor, focus) = if collapse || forward {
        (minimum_start, minimum_end)
    } else {
        (minimum_end, minimum_start)
    };
    VisibleSelection {
        anchor,
        focus,
        affinity: TextAffinity::Downstream,
    }
}

/// Source visible_selection.cc:58-107 and SelectionAdjuster's flat-tree
/// specializations. Order remains canonicalization, granularity, shadow,
/// editing and finally selection type adjustment.
pub fn ComputeVisibleSelectionInFlatTree<C>(
    ctx: &C,
    anchor: &Position<C::Node>,
    focus: &Position<C::Node>,
    affinity: TextAffinity,
    granularity: super::text_granularity::TextGranularity,
    inclusion: super::selection_adjuster::WordInclusion,
) -> VisibleSelection<C::Node>
where
    C: super::selection_adjuster::GranularityContext
        + super::selection_boundary_adjuster::FlatSelectionBoundaryContext
        + EditableBoundaryContext,
{
    if granularity == super::text_granularity::TextGranularity::kWord {
        return CreateVisibleWordSelectionInFlatTree(ctx, anchor, focus, affinity, inclusion);
    }
    if granularity == super::text_granularity::TextGranularity::kCharacter {
        return CreateVisibleCharacterSelectionInFlatTree(ctx, anchor, focus, affinity);
    }
    let mut result = CanonicalSelection(ctx, anchor, focus, affinity);
    if result.anchor.IsNull() {
        return result;
    }
    let forward = EditableBoundaryContext::ComparePositions(ctx, &result.anchor, &result.focus)
        != Some(Ordering::Greater);
    let (start, end) = if forward {
        (&result.anchor, &result.focus)
    } else {
        (&result.focus, &result.anchor)
    };
    let (start, end) = super::selection_adjuster::AdjustSelectionRespectingGranularity(
        ctx,
        start,
        end,
        affinity,
        granularity,
        inclusion,
        false,
    );
    if EditableBoundaryContext::ComparePositions(ctx, &start, &end) == Some(Ordering::Equal) {
        let collapse = if forward { start } else { end };
        result.anchor = collapse.clone();
        result.focus = collapse;
    } else if forward {
        result.anchor = start;
        result.focus = end;
    } else {
        result.anchor = end;
        result.focus = start;
    }
    result.focus = super::selection_boundary_adjuster::AdjustFlatFocusForShadowBoundary(
        ctx,
        &result.anchor,
        &result.focus,
    );
    result.focus = super::selection_boundary_adjuster::AdjustFlatFocusForEditingBoundary(
        ctx,
        &result.anchor,
        &result.focus,
    );
    AdjustSelectionType(ctx, result)
}

/// The same word specialization as ComputeVisibleSelectionInFlatTree. This
/// admits a scoped word-boundary service without unrelated sentence ports.
pub fn CreateVisibleWordSelectionInFlatTree<C>(
    ctx: &C,
    anchor: &Position<C::Node>,
    focus: &Position<C::Node>,
    affinity: TextAffinity,
    inclusion: super::selection_adjuster::WordInclusion,
) -> VisibleSelection<C::Node>
where
    C: super::selection_adjuster::WordGranularityContext
        + super::selection_boundary_adjuster::FlatSelectionBoundaryContext
        + EditableBoundaryContext,
{
    let mut result = CanonicalSelection(ctx, anchor, focus, affinity);
    if result.anchor.IsNull() {
        return result;
    }
    let forward = EditableBoundaryContext::ComparePositions(ctx, &result.anchor, &result.focus)
        != Some(Ordering::Greater);
    let (start, end) = if forward {
        (&result.anchor, &result.focus)
    } else {
        (&result.focus, &result.anchor)
    };
    let new_start = super::selection_adjuster::ComputeWordStart(
        ctx,
        &PositionWithAffinity {
            position: start.clone(),
            affinity,
        },
        inclusion,
    );
    let start = if new_start.IsNull() {
        start.clone()
    } else {
        new_start
    };
    let new_end = super::selection_adjuster::ComputeWordEnd(
        ctx,
        &start,
        &PositionWithAffinity {
            position: end.clone(),
            affinity,
        },
        inclusion,
    );
    let end = if new_end.IsNull() {
        end.clone()
    } else {
        new_end
    };
    if EditableBoundaryContext::ComparePositions(ctx, &start, &end) == Some(Ordering::Equal) {
        let collapse = if forward { start } else { end };
        result.anchor = collapse.clone();
        result.focus = collapse;
    } else if forward {
        result.anchor = start;
        result.focus = end;
    } else {
        result.anchor = end;
        result.focus = start;
    }
    result.focus = super::selection_boundary_adjuster::AdjustFlatFocusForShadowBoundary(
        ctx,
        &result.anchor,
        &result.focus,
    );
    result.focus = super::selection_boundary_adjuster::AdjustFlatFocusForEditingBoundary(
        ctx,
        &result.anchor,
        &result.focus,
    );
    AdjustSelectionType(ctx, result)
}

pub fn CreateVisibleCharacterSelectionInFlatTree<C>(
    ctx: &C,
    anchor: &Position<C::Node>,
    focus: &Position<C::Node>,
    affinity: TextAffinity,
) -> VisibleSelection<C::Node>
where
    C: VisiblePositionContext
        + super::selection_boundary_adjuster::FlatSelectionBoundaryContext
        + EditableBoundaryContext,
{
    let mut result = CanonicalSelection(ctx, anchor, focus, affinity);
    if result.anchor.IsNull() {
        return result;
    }
    result.focus = super::selection_boundary_adjuster::AdjustFlatFocusForShadowBoundary(
        ctx,
        &result.anchor,
        &result.focus,
    );
    result.focus = super::selection_boundary_adjuster::AdjustFlatFocusForEditingBoundary(
        ctx,
        &result.anchor,
        &result.focus,
    );
    AdjustSelectionType(ctx, result)
}
pub fn CreateVisibleSelectionInFlatTree<C>(
    ctx: &C,
    anchor: &Position<C::Node>,
    focus: &Position<C::Node>,
    affinity: TextAffinity,
) -> VisibleSelection<C::Node>
where
    C: super::selection_adjuster::GranularityContext
        + super::selection_boundary_adjuster::FlatSelectionBoundaryContext
        + EditableBoundaryContext,
{
    ComputeVisibleSelectionInFlatTree(
        ctx,
        anchor,
        focus,
        affinity,
        super::text_granularity::TextGranularity::kCharacter,
        super::selection_adjuster::WordInclusion::Default,
    )
}

/// Source visible_selection.cc:195-234 and editing_utilities.cc:1603-1621.
/// DOM selection normalization preserves source upstream caret styling and the
/// minimum rendered range. Only the existing position/caret owners are read.
pub fn NormalizeSelectionRange<C: VisiblePositionContext + EditableBoundaryContext>(
    ctx: &C,
    anchor: &Position<C::Node>,
    focus: &Position<C::Node>,
) -> Option<(Position<C::Node>, Position<C::Node>)> {
    if anchor.IsNull() || focus.IsNull() {
        return None;
    }
    let (start, end) = if super::compare_positions::CompareDOMPositions(ctx, anchor, focus)?
        == Ordering::Greater
    {
        (focus, anchor)
    } else {
        (anchor, focus)
    };
    if anchor == focus {
        let caret = DownstreamVisiblePositionContext::MostBackwardCaretPosition(ctx, start)
            .ParentAnchoredEquivalent(ctx);
        return Some((caret.clone(), caret));
    }
    let start = DownstreamVisiblePositionContext::MostForwardCaretPosition(ctx, start)
        .ParentAnchoredEquivalent(ctx);
    let end = DownstreamVisiblePositionContext::MostBackwardCaretPosition(ctx, end)
        .ParentAnchoredEquivalent(ctx);
    if super::compare_positions::CompareDOMPositions(ctx, &start, &end)? == Ordering::Greater {
        Some((end, start))
    } else {
        Some((start, end))
    }
}
