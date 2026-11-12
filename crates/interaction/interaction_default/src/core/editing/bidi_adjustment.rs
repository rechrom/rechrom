// Copyright 2017 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/bidi_adjustment.cc:536-589,665-696,766-770.
// Native Layout resolves visual run boundaries; this owner preserves the
// original flat-tree visible-position canonicalization and endpoint order.
use super::{
    inline_box_position::{ComputeInlineAdjustedPosition, InlineAdjustmentContext},
    position::{Position, PositionWithAffinity, TextAffinity},
    visible_position::{VisiblePosition, VisiblePositionContext},
    visible_units::EditingBoundaryCrossingRule,
};
use crate::core::editing::visible_position::DownstreamVisiblePositionContext;

pub enum BidiRangeAdjustment<N> {
    Unchanged,
    AdjustedBase(Position<N>),
    AdjustedExtent(Position<N>),
}
pub trait BidiAdjustmentContext: InlineAdjustmentContext + VisiblePositionContext {
    fn AdjustInlineBidiRange(
        &self,
        base: &PositionWithAffinity<Self::Node>,
        extent: &PositionWithAffinity<Self::Node>,
    ) -> BidiRangeAdjustment<Self::Node>;
}
fn InlineAdjusted<C: BidiAdjustmentContext>(
    context: &C,
    position: &PositionWithAffinity<C::Node>,
) -> PositionWithAffinity<C::Node> {
    if !position
        .position
        .AnchorNode()
        .is_some_and(|node| DownstreamVisiblePositionContext::HasLayoutObject(context, node))
    {
        return PositionWithAffinity::default();
    }
    ComputeInlineAdjustedPosition(
        context,
        position.clone(),
        EditingBoundaryCrossingRule::CanCross,
    )
}
pub fn AdjustForRangeSelection<C: BidiAdjustmentContext>(
    context: &C,
    visible_base: &PositionWithAffinity<C::Node>,
    visible_extent: &PositionWithAffinity<C::Node>,
) -> (Position<C::Node>, Position<C::Node>) {
    let unchanged = || {
        (
            visible_base.position.clone(),
            visible_extent.position.clone(),
        )
    };
    // Source:536-553. BidiCaretAffinity keeps its existing NG-specific branch;
    // the disabled-feature path below performs actual visual-run adjustment.
    if context.BidiCaretAffinityEnabled()
        && (context
            .InlineFormattingContext(&visible_base.position)
            .is_some()
            || context
                .InlineFormattingContext(&visible_extent.position)
                .is_some())
    {
        return unchanged();
    }
    let base = InlineAdjusted(context, visible_base);
    let extent = InlineAdjusted(context, visible_extent);
    if base.position.IsNull() || extent.position.IsNull() {
        return unchanged();
    }
    let visible = |position| {
        VisiblePosition::Create(
            context,
            PositionWithAffinity {
                position,
                affinity: TextAffinity::Downstream,
            },
        )
        .DeepEquivalent()
        .clone()
    };
    match context.AdjustInlineBidiRange(&base, &extent) {
        BidiRangeAdjustment::Unchanged => unchanged(),
        BidiRangeAdjustment::AdjustedBase(position) => {
            (visible(position), visible_extent.position.clone())
        }
        BidiRangeAdjustment::AdjustedExtent(position) => {
            (visible_base.position.clone(), visible(position))
        }
    }
}
