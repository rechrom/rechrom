// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source selection_modifier_line.cc:240-682. Flat traversal and editable-root
// ownership stay in Editing; native leaves query the resident InlineCursor.
use super::{
    editing_boundaries::{self, EditableBoundaryContext},
    position::{Position, PositionWithAffinity, TextAffinity},
    visible_position::{DownstreamVisiblePositionContext, VisiblePosition, VisiblePositionContext},
    visible_units::{self, VisibleUnitsContext},
};
use foundation::LayoutUnit;

/// Distinguish a missing line (which searches the DOM for another line) from
/// an existing line with no usable caret (which uses the content boundary).
pub enum LinePositionResult<N> {
    NoLine,
    Line(Option<PositionWithAffinity<N>>),
}
pub trait LineMovementContext: VisiblePositionContext + EditableBoundaryContext {
    fn NativePositionOnLine(
        &self,
        position: &PositionWithAffinity<Self::Node>,
        coordinate: LayoutUnit,
        adjacent: Option<bool>,
        only_editable: bool,
    ) -> LinePositionResult<Self::Node>;
    fn AtomicInlineFormattingContextNode(
        &self,
        position: &Position<Self::Node>,
    ) -> Option<Self::Node>;
    fn InlineBlockLineNavigationEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::InlineBlockLineNavigationEnabled()
    }
    fn RootEditableElementOfNode(&self, node: &Self::Node) -> Option<Self::Node>;
    fn CaretMinOffset(&self, node: &Self::Node) -> usize;
}
fn pwa<N>(position: Position<N>) -> PositionWithAffinity<N> {
    PositionWithAffinity {
        position,
        affinity: TextAffinity::Downstream,
    }
}
fn PreviousNode<C: LineMovementContext>(ctx: &C, start: &C::Node) -> Option<C::Node> {
    if let Some(mut node) = ctx.PreviousSibling(start) {
        while !editing_boundaries::IsAtomicNode(ctx, &node) {
            node = ctx.ChildAt(&node, ctx.ChildCount(&node).checked_sub(1)?)?;
        }
        Some(node)
    } else {
        ctx.Parent(start)
    }
}
fn NextNode<C: LineMovementContext>(ctx: &C, start: &C::Node) -> Option<C::Node> {
    if !editing_boundaries::IsAtomicNode(ctx, start) {
        if let Some(child) = ctx.ChildAt(start, 0) {
            return Some(child);
        }
    }
    let mut node = start.clone();
    loop {
        if let Some(sibling) = ctx.NextSibling(&node) {
            return Some(sibling);
        }
        node = ctx.Parent(&node)?;
    }
}
fn LeafWithEditability<C: LineMovementContext>(
    ctx: &C,
    start: &C::Node,
    next: bool,
    editable: bool,
) -> Option<C::Node> {
    let mut node = if next {
        NextNode(ctx, start)
    } else {
        PreviousNode(ctx, start)
    };
    while let Some(current) = node {
        if editing_boundaries::IsAtomicNode(ctx, &current)
            && VisibleUnitsContext::IsEditable(ctx, &current) == editable
        {
            return Some(current);
        }
        node = if next {
            NextNode(ctx, &current)
        } else {
            PreviousNode(ctx, &current)
        };
    }
    None
}
fn NodeInSameLine<C: LineMovementContext>(
    ctx: &C,
    node: &C::Node,
    position: &PositionWithAffinity<C::Node>,
) -> bool {
    if !DownstreamVisiblePositionContext::HasLayoutObject(ctx, node) {
        return true;
    }
    let first = VisiblePosition::Create(ctx, pwa(Position::FirstPositionInOrBeforeNode(ctx, node)));
    DownstreamVisiblePositionContext::InSameLine(ctx, first.ToPositionWithAffinity(), position)
}
fn RootInlineCandidate<C: LineMovementContext>(
    ctx: &C,
    start: &C::Node,
    position: &PositionWithAffinity<C::Node>,
    next: bool,
) -> Position<C::Node> {
    let root = ctx.HighestEditableRoot(&position.position);
    let editable = if next {
        position
            .position
            .ComputeContainerNode(ctx)
            .is_some_and(|n| VisibleUnitsContext::IsEditable(ctx, &n))
    } else {
        VisibleUnitsContext::IsEditable(ctx, start)
    };
    let mut node = LeafWithEditability(ctx, start, next, editable);
    while node
        .as_ref()
        .is_some_and(|n| NodeInSameLine(ctx, n, position))
    {
        // PreviousLeafWithSameEditability re-reads each leaf's editability.
        let current = node.as_ref().unwrap();
        node = LeafWithEditability(
            ctx,
            current,
            next,
            if next {
                editable
            } else {
                VisibleUnitsContext::IsEditable(ctx, current)
            },
        );
    }
    while let Some(current) = node {
        if ctx.IsShadowRoot(&current)
            || ctx.HighestEditableRoot(&Position::FirstPositionInOrBeforeNode(ctx, &current))
                != root
        {
            break;
        }
        let candidate = if !next && ctx.IsBR(&current) {
            Position::BeforeNode(current.clone())
        } else {
            Position::EditingPositionOf(
                ctx,
                current.clone(),
                if next {
                    ctx.CaretMinOffset(&current)
                } else {
                    ctx.CaretMaxOffset(&current)
                },
            )
        };
        if visible_units::IsVisuallyEquivalentCandidate(ctx, &candidate) {
            return candidate;
        }
        node = LeafWithEditability(
            ctx,
            &current,
            next,
            if next {
                editable
            } else {
                VisibleUnitsContext::IsEditable(ctx, &current)
            },
        );
    }
    Position::default()
}
fn LinePosition<C: LineMovementContext>(
    ctx: &C,
    position: &PositionWithAffinity<C::Node>,
    coordinate: LayoutUnit,
    next: bool,
) -> PositionWithAffinity<C::Node> {
    let Some(node) = position.position.AnchorNode() else {
        return Default::default();
    };
    if !DownstreamVisiblePositionContext::HasLayoutObject(ctx, node) {
        return Default::default();
    }
    let editable = ctx.IsEditablePosition(&position.position);
    let mut line = ctx.NativePositionOnLine(position, coordinate, Some(next), editable);
    if matches!(line, LinePositionResult::NoLine) && ctx.InlineBlockLineNavigationEnabled() {
        if let Some(atomic) = ctx.AtomicInlineFormattingContextNode(&position.position) {
            let outer = pwa(if next {
                Position::AfterNode(atomic)
            } else {
                Position::BeforeNode(atomic)
            });
            line = ctx.NativePositionOnLine(&outer, coordinate, Some(next), editable);
        }
    }
    if matches!(line, LinePositionResult::NoLine) {
        let mut search_start = node.clone();
        if next {
            search_start = if let Some(child) =
                ctx.ChildAt(node, position.position.ComputeEditingOffset(ctx))
            {
                child
            } else {
                let mut last = node.clone();
                while let Some(child) = ctx
                    .ChildCount(&last)
                    .checked_sub(1)
                    .and_then(|i| ctx.ChildAt(&last, i))
                {
                    last = child;
                }
                last
            };
        }
        let candidate = RootInlineCandidate(ctx, &search_start, position, next);
        if !candidate.IsNull() {
            let visible = VisiblePosition::Create(ctx, pwa(candidate.clone()));
            line = ctx.NativePositionOnLine(
                visible.ToPositionWithAffinity(),
                coordinate,
                None,
                editable,
            );
            if matches!(line, LinePositionResult::NoLine) {
                return pwa(candidate);
            }
        }
    }
    if let LinePositionResult::Line(Some(candidate)) = line {
        if editable && !ctx.IsEditablePosition(&candidate.position) {
            return if next {
                editing_boundaries::AdjustForwardPositionToAvoidCrossingEditingBoundaries(
                    ctx,
                    candidate,
                    &position.position,
                )
            } else {
                editing_boundaries::AdjustBackwardPositionToAvoidCrossingEditingBoundaries(
                    ctx,
                    candidate,
                    &position.position,
                )
            };
        }
        return candidate;
    }
    let root = if VisibleUnitsContext::IsEditable(ctx, node) {
        ctx.RootEditableElementOfNode(node)
    } else {
        ctx.DocumentElement(node)
    };
    root.map_or_else(Default::default, |root| {
        pwa(if next {
            Position::LastPositionInNode(ctx, &root)
        } else {
            Position::FirstPositionInNode(root)
        })
    })
}
pub fn PreviousLinePosition<C: LineMovementContext>(
    ctx: &C,
    position: &PositionWithAffinity<C::Node>,
    coordinate: LayoutUnit,
) -> PositionWithAffinity<C::Node> {
    LinePosition(ctx, position, coordinate, false)
}
pub fn NextLinePosition<C: LineMovementContext>(
    ctx: &C,
    position: &PositionWithAffinity<C::Node>,
    coordinate: LayoutUnit,
) -> PositionWithAffinity<C::Node> {
    LinePosition(ctx, position, coordinate, true)
}
