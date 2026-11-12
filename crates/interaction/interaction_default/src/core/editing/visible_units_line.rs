// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/visible_units_line.cc:434-481.
use super::{
    position::{Position, PositionWithAffinity},
    text_offset_mapping::{self, TextOffsetMappingContext},
};
use crate::core::editing::visible_position::DownstreamVisiblePositionContext;
pub trait VisibleLineContext: TextOffsetMappingContext {
    fn InlineFormattingContextForPosition(
        &self,
        position: &Position<Self::Node>,
    ) -> Option<(u64, bool)>;
    fn InSameNGLineBox(
        &self,
        a: &PositionWithAffinity<Self::Node>,
        b: &PositionWithAffinity<Self::Node>,
    ) -> bool;
    fn ToFlatPosition(&self, position: &Position<Self::Node>) -> Position<Self::Node>;
    fn RootEditableElementOf(&self, position: &Position<Self::Node>) -> Option<Self::Node>;
    fn StartOfLine(
        &self,
        position: &PositionWithAffinity<Self::Node>,
    ) -> PositionWithAffinity<Self::Node>;
    fn CanonicalPositionOf(&self, position: &Position<Self::Node>) -> Position<Self::Node>;
    // FindForwardInlineContents always traverses the flat tree, even when the
    // positions and the enclosing InSameLine call use DOM strategy.
    fn ForwardInlineContentsInFlatTree(
        &self,
        position: &Position<Self::Node>,
    ) -> Option<Self::InlineContents>;
}
pub trait StartOfLineContext:
    VisibleLineContext
    + super::inline_box_position::InlineAdjustmentContext
    + super::editing_boundaries::EditableBoundaryContext
{
    fn NativeStartOfLine(
        &self,
        position: &PositionWithAffinity<Self::Node>,
    ) -> PositionWithAffinity<Self::Node>;
    fn IsLayoutBlock(&self, node: &Self::Node) -> bool;
}
pub trait EndOfLineContext: StartOfLineContext {
    fn NativeEndOfLine(
        &self,
        position: &PositionWithAffinity<Self::Node>,
    ) -> PositionWithAffinity<Self::Node>;
}
/// Source: visible_units_line.cc:109-152,353-371. Native line queries apply the
/// source trailing-space soft-wrap adjustment on the same OffsetMapping.
pub fn EndOfLine<C: EndOfLineContext>(
    ctx: &C,
    current: &PositionWithAffinity<C::Node>,
) -> PositionWithAffinity<C::Node> {
    if current.position.IsNull() {
        return PositionWithAffinity::default();
    }
    let adjusted = super::inline_box_position::ComputeInlineAdjustedPosition(
        ctx,
        current.clone(),
        super::visible_units::EditingBoundaryCrossingRule::CanCross,
    );
    let end = if ctx
        .InlineFormattingContextForPosition(&adjusted.position)
        .is_some()
    {
        ctx.NativeEndOfLine(&adjusted)
    } else if current
        .position
        .AnchorNode()
        .is_some_and(|node| ctx.IsLayoutBlock(node))
        && current.position.ComputeEditingOffset(ctx) == 0
    {
        current.clone()
    } else {
        PositionWithAffinity::default()
    };
    super::editing_boundaries::AdjustForwardPositionToAvoidCrossingEditingBoundaries(
        ctx,
        end,
        &current.position,
    )
}
/// Source visible_units_line.cc:155-189,240-250. Normalization and editing
/// boundary adjustment remain in Editing; only the resident line query is native.
pub fn StartOfLine<C: StartOfLineContext>(
    ctx: &C,
    current: &PositionWithAffinity<C::Node>,
) -> PositionWithAffinity<C::Node> {
    if current.position.IsNull() {
        return PositionWithAffinity::default();
    }
    let adjusted = super::inline_box_position::ComputeInlineAdjustedPosition(
        ctx,
        current.clone(),
        super::visible_units::EditingBoundaryCrossingRule::CanCross,
    );
    let line_start = if ctx
        .InlineFormattingContextForPosition(&adjusted.position)
        .is_some()
    {
        ctx.NativeStartOfLine(&adjusted)
    } else if current
        .position
        .AnchorNode()
        .is_some_and(|node| ctx.IsLayoutBlock(node))
        && current.position.ComputeEditingOffset(ctx) == 0
    {
        current.clone()
    } else {
        PositionWithAffinity::default()
    };
    super::editing_boundaries::AdjustBackwardPositionToAvoidCrossingEditingBoundaries(
        ctx,
        line_start,
        &current.position,
    )
}
pub fn InSameLine<C: VisibleLineContext>(
    ctx: &C,
    first: &PositionWithAffinity<C::Node>,
    second: &PositionWithAffinity<C::Node>,
) -> bool {
    if first.position.IsNull() || second.position.IsNull() {
        return false;
    }
    let block1 = ctx.InlineFormattingContextForPosition(&first.position);
    let block2 = ctx.InlineFormattingContextForPosition(&second.position);
    if block1.is_some() || block2.is_some() {
        if block1.is_some_and(|(_, inline)| inline) || block2.is_some_and(|(_, inline)| inline) {
            let first = ctx.ForwardInlineContentsInFlatTree(&ctx.ToFlatPosition(&first.position));
            let second = ctx.ForwardInlineContentsInFlatTree(&ctx.ToFlatPosition(&second.position));
            if first != second {
                return false;
            }
        } else {
            if block1.map(|(id, _)| id) != block2.map(|(id, _)| id)
                || !ctx.InSameNGLineBox(first, second)
            {
                return false;
            }
        }
        return ctx.RootEditableElementOf(&first.position)
            == ctx.RootEditableElementOf(&second.position);
    }
    let start1 = ctx.StartOfLine(first);
    let start2 = ctx.StartOfLine(second);
    if start1 == start2 {
        return true;
    }
    let canonical1 = ctx.CanonicalPositionOf(&start1.position);
    canonical1 == start2.position || canonical1 == ctx.CanonicalPositionOf(&start2.position)
}

pub trait LogicalLineContext:
    EndOfLineContext + super::visible_position::VisiblePositionContext
{
    fn IsLayoutBlockFlowNode(&self, node: &Self::Node) -> bool;
    fn NativeLogicalStartOfLine(
        &self,
        position: &PositionWithAffinity<Self::Node>,
    ) -> PositionWithAffinity<Self::Node>;
    fn NativeLogicalEndOfLine(
        &self,
        position: &PositionWithAffinity<Self::Node>,
    ) -> PositionWithAffinity<Self::Node>;
}
// visible_units_line.cc:88-106,287-326,383-406. Preserve DOM containment even
// when the position strategy is flat; native cursor ordering is queried only
// after the source caret/inline normalization.
pub fn LogicalLineBoundary<C: LogicalLineContext>(
    ctx: &C,
    current: &PositionWithAffinity<C::Node>,
    end: bool,
) -> PositionWithAffinity<C::Node> {
    use super::{
        editing_boundaries::EditableBoundaryContext, position::PositionAnchorType,
        visible_position::VisiblePositionContext, visible_units::EditingBoundaryCrossingRule,
    };
    let position = &current.position;
    let needs_normalization = !position.IsNull()
        && !matches!(
            position.AnchorType(),
            PositionAnchorType::BeforeAnchor | PositionAnchorType::AfterAnchor
        )
        && position
            .AnchorNode()
            .is_some_and(|node| !ctx.IsText(node) && ctx.IsLayoutBlockFlowNode(node));
    let normalized = if needs_normalization {
        PositionWithAffinity {
            position: if end {
                DownstreamVisiblePositionContext::MostForwardCaretPosition(ctx, position)
            } else {
                DownstreamVisiblePositionContext::MostBackwardCaretPosition(ctx, position)
            },
            affinity: current.affinity,
        }
    } else {
        current.clone()
    };
    let adjusted = super::inline_box_position::ComputeInlineAdjustedPosition(
        ctx,
        normalized.clone(),
        EditingBoundaryCrossingRule::CanCross,
    );
    let candidate = if !adjusted.position.IsNull()
        && ctx
            .InlineFormattingContextForPosition(&adjusted.position)
            .is_some()
    {
        if end {
            ctx.NativeLogicalEndOfLine(&adjusted)
        } else {
            ctx.NativeLogicalStartOfLine(&adjusted)
        }
    } else if normalized
        .position
        .AnchorNode()
        .is_some_and(|node| ctx.IsLayoutBlock(node))
        && normalized.position.ComputeEditingOffset(ctx) == 0
    {
        normalized
    } else {
        PositionWithAffinity::default()
    };
    if let Some(root) = EditableBoundaryContext::HighestEditableRoot(ctx, position) {
        if !end && candidate.position.IsNull() {
            let mut node = position.ComputeContainerNode(ctx);
            while let Some(current) = node {
                if current == root {
                    break;
                }
                if ctx.IsLayoutBlock(&current) {
                    return PositionWithAffinity {
                        position: Position::FirstPositionInNode(current),
                        affinity: super::position::TextAffinity::Downstream,
                    };
                }
                node = ctx.Parent(&current);
            }
            return PositionWithAffinity {
                position: Position::FirstPositionInNode(root),
                affinity: super::position::TextAffinity::Downstream,
            };
        }
        if !candidate
            .position
            .ComputeContainerNode(ctx)
            .is_some_and(|container| ctx.DOMContains(&root, &container))
        {
            return PositionWithAffinity {
                position: if end {
                    Position::LastPositionInNode(ctx, &root)
                } else {
                    Position::FirstPositionInNode(root)
                },
                affinity: super::position::TextAffinity::Downstream,
            };
        }
    }
    if end {
        super::editing_boundaries::AdjustForwardPositionToAvoidCrossingEditingBoundaries(
            ctx, candidate, position,
        )
    } else {
        super::editing_boundaries::AdjustBackwardPositionToAvoidCrossingEditingBoundaries(
            ctx, candidate, position,
        )
    }
}
