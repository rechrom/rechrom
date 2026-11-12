// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source editing_commands_utilities.cc:705-737.
use crate::core::editing::commands::structural_editing_support::Ancestor;
use crate::core::editing::text_granularity::TextGranularity as G;
use interaction_dispatch::core::events::input_event::InputType as I;

pub fn DeletionInputTypeFromTextGranularity(forward: bool, granularity: G) -> I {
    match (forward, granularity) {
        (true, G::kWord) => I::kDeleteWordForward,
        (true, G::kLineBoundary) => I::kDeleteSoftLineForward,
        (true, G::kParagraphBoundary) => I::kDeleteHardLineForward,
        (true, _) => I::kDeleteContentForward,
        (false, G::kWord) => I::kDeleteWordBackward,
        (false, G::kLineBoundary) => I::kDeleteSoftLineBackward,
        (false, G::kParagraphBoundary) => I::kDeleteHardLineBackward,
        (false, _) => I::kDeleteContentBackward,
    }
}

// Source editing_commands_utilities.cc:306-384.
use super::structural_editing_context::{
    StructuralBooleanQuery, StructuralEditingContext, StructuralPositionQuery,
};
use crate::core::editing::position::Position;
pub(super) fn IsSpace(unit: u16, non_collapsible: bool) -> bool {
    if non_collapsible {
        foundation::unicode::IsSpaceOrNewline(unit) || unit == 0xa0
    } else {
        crate::core::editing::editing_utilities::IsCollapsibleWhitespace(unit)
    }
}
fn PreviousCharacter<C: StructuralEditingContext>(
    ctx: &C,
    position: &Position<u64>,
) -> Position<u64> {
    let Some(anchor) = position.AnchorNode() else {
        return Position::default();
    };
    let root = ctx.RootEditable(*anchor);
    let start_line = ctx.PositionBoolean(position, None, StructuralBooleanQuery::StartOfLine);
    let rendered = ctx.PositionBoolean(
        position,
        None,
        StructuralBooleanQuery::VisuallyEquivalentCandidate,
    );
    let mut current = position.clone();
    while !current.AtStartOfTree(ctx) {
        let previous = current.PreviousCodeUnitPosition(ctx);
        if previous == current || previous.IsNull() {
            break;
        }
        current = previous;
        if current
            .AnchorNode()
            .and_then(|node| ctx.RootEditable(*node))
            != root
        {
            return position.clone();
        }
        if if start_line || !rendered {
            ctx.PositionBoolean(
                &current,
                None,
                StructuralBooleanQuery::VisuallyEquivalentCandidate,
            )
        } else {
            ctx.PositionBoolean(
                position,
                Some(&current),
                StructuralBooleanQuery::RendersInDifferentPosition,
            )
        } {
            return current;
        }
    }
    position.clone()
}
pub(super) fn LeadingCollapsibleWhitespacePosition<C: StructuralEditingContext>(
    ctx: &C,
    position: &Position<u64>,
    non_collapsible: bool,
) -> Position<u64> {
    if position.IsNull()
        || ctx
            .QueryPosition(position, StructuralPositionQuery::Backward)
            .AnchorNode()
            .is_some_and(|node| ctx.Tag(*node, "br"))
    {
        return Position::default();
    }
    let previous = PreviousCharacter(ctx, position);
    let Some(node) = previous
        .AnchorNode()
        .copied()
        .filter(|node| ctx.IsText(node))
    else {
        return Position::default();
    };
    if previous == *position
        || Ancestor(ctx, &previous, |node| ctx.NodeFacts(node).block_flow)
            != Ancestor(ctx, position, |node| ctx.NodeFacts(node).block_flow)
    {
        return Position::default();
    }
    if !non_collapsible && ctx.NodeFacts(node).has_layout && ctx.NodeFacts(node).preserve_spaces {
        return Position::default();
    }
    if ctx
        .TextCodeUnit(node, previous.ComputeOffsetInContainerNode(ctx))
        .is_some_and(|unit| IsSpace(unit, non_collapsible))
        && ctx.Editable(node)
    {
        previous
    } else {
        Position::default()
    }
}

// Source editing_commands_utilities.cc:176-271,379-387,418-439.
use crate::core::editing::{
    commands::structural_editing_support::{ParentAfter, ParentBefore},
    editing_utilities::{IsList, IsMailQuote},
};

pub(super) fn MailQuoteLevel<C: StructuralEditingContext>(ctx: &C, p: &Position<u64>) -> usize {
    let mut count = 0;
    let mut node = p.AnchorNode().copied();
    while let Some(current) = node {
        if IsMailQuote(ctx, current) {
            count += 1;
        }
        node = ctx.Parent(&current);
    }
    count
}
pub(super) fn EnclosingListChild<C: StructuralEditingContext>(
    ctx: &C,
    p: &Position<u64>,
) -> Option<u64> {
    Ancestor(ctx, p, |node| {
        ctx.Tag(node, "li") || ctx.Parent(&node).is_some_and(|parent| IsList(ctx, parent))
    })
}
/// editing_commands_utilities.cc:176-271.
/// Source expansion uses visible equivalence but preserves the original DOM
/// selection and editable-root containment when extending special containers.
fn ContainingSpecialElement<C: StructuralEditingContext>(
    ctx: &C,
    position: &Position<u64>,
    before: bool,
) -> Option<u64> {
    let root = ctx.RootEditable(position.ComputeContainerNode(ctx)?);
    let mut runner = position.AnchorNode().copied();
    while let Some(node) = runner {
        if ctx.RootEditable(node) != root {
            break;
        }
        if ctx.NodeFacts(node).special_element {
            let visible = ctx.QueryPosition(position, StructuralPositionQuery::Visible);
            let edge = if before {
                Position::FirstPositionInOrBeforeNode(ctx, &node)
            } else {
                Position::LastPositionInOrAfterNode(ctx, &node)
            };
            let edge = ctx.QueryPosition(&edge, StructuralPositionQuery::Visible);
            let table = ctx.NodeFacts(node).has_layout && ctx.Tag(node, "table");
            if table
                && (!before
                    || visible
                        .ComputeContainerNode(ctx)
                        .is_none_or(|container| !ctx.NodeFacts(container).list_item))
            {
                let adjacent = ctx.QueryPosition(
                    &edge,
                    if before {
                        StructuralPositionQuery::Next
                    } else {
                        StructuralPositionQuery::Previous
                    },
                );
                if visible == adjacent {
                    return Some(node);
                }
            }
            if visible == edge {
                return Some(node);
            }
        }
        runner = ctx.Parent(&node);
    }
    None
}
pub(super) fn OutsideSpecialElement<C: StructuralEditingContext>(
    ctx: &C,
    position: &Position<u64>,
    before: bool,
) -> (Position<u64>, Option<u64>) {
    let Some(node) = ContainingSpecialElement(ctx, position, before) else {
        return (position.clone(), None);
    };
    let result = if before {
        ParentBefore(ctx, node)
    } else {
        ParentAfter(ctx, node)
    };
    if result.IsNull()
        || result.AnchorNode().and_then(|node| ctx.RootEditable(*node))
            != position
                .AnchorNode()
                .and_then(|node| ctx.RootEditable(*node))
    {
        return (position.clone(), None);
    }
    (result, Some(node))
}

// Source editing_commands_utilities.cc:866-879.
// Structural host projection; retains the existing query and null behavior.
pub(crate) fn IsBlockEdge<C: StructuralEditingContext>(
    ctx: &C,
    p: &Position<u64>,
    start: bool,
) -> bool {
    let edge = ctx.QueryPosition(
        p,
        if start {
            StructuralPositionQuery::BlockStart
        } else {
            StructuralPositionQuery::BlockEnd
        },
    );
    ctx.QueryPosition(p, StructuralPositionQuery::Visible)
        == ctx.QueryPosition(&edge, StructuralPositionQuery::Visible)
}
