// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/inline_box_position.cc:45-186.
use super::{
    position::{Position, PositionAnchorType, PositionDOM, PositionWithAffinity, TextAffinity},
    visible_units::{EditingBoundaryCrossingRule, VisibleUnitsContext},
};
pub struct InlineAdjustmentFacts {
    pub is_text: bool,
    pub is_inline: bool,
    pub is_atomic_inline: bool,
    pub in_ng_inline_context: bool,
    pub is_block_flow: bool,
    pub has_rendered_descendants: bool,
}
pub trait InlineAdjustmentContext: VisibleUnitsContext {
    fn InlineAdjustmentFacts(
        &self,
        position: &Position<Self::Node>,
    ) -> Option<InlineAdjustmentFacts>;
    fn CanHaveChildrenForEditing(&self, node: &Self::Node) -> bool;
    fn LocalCaretEquivalent(
        &self,
        position: &Position<Self::Node>,
        rule: EditingBoundaryCrossingRule,
        backward: bool,
    ) -> Position<Self::Node>;
}
fn Equivalent<C: PositionDOM>(
    ctx: &C,
    first: &Position<C::Node>,
    second: &Position<C::Node>,
) -> bool {
    if first.IsNull() {
        return second.IsNull();
    }
    if first.AnchorType() == second.AnchorType() {
        return first == second;
    }
    first.ToOffsetInAnchor(ctx) == second.ToOffsetInAnchor(ctx)
}
fn VisuallyEquivalent<C: InlineAdjustmentContext>(
    ctx: &C,
    mut position: Position<C::Node>,
    rule: EditingBoundaryCrossingRule,
    backward: bool,
) -> Position<C::Node> {
    let mut last = Position::default();
    while !Equivalent(ctx, &position, &last) {
        last = position.clone();
        position = ctx.LocalCaretEquivalent(&position, rule, backward);
    }
    position
}
fn Compute<C: InlineAdjustmentContext>(
    ctx: &C,
    position: PositionWithAffinity<C::Node>,
    depth: usize,
    rule: EditingBoundaryCrossingRule,
) -> PositionWithAffinity<C::Node> {
    let Some(facts) = ctx.InlineAdjustmentFacts(&position.position) else {
        return PositionWithAffinity::default();
    };
    if facts.is_text {
        return position;
    }
    if matches!(
        position.position.AnchorType(),
        PositionAnchorType::BeforeAnchor | PositionAnchorType::AfterAnchor
    ) {
        if facts.in_ng_inline_context {
            return if facts.is_inline {
                position
            } else {
                PositionWithAffinity::default()
            };
        }
        if facts.is_atomic_inline {
            return position;
        }
    }
    if facts.is_block_flow
        && position
            .position
            .AnchorNode()
            .is_some_and(|node| ctx.CanHaveChildrenForEditing(node))
        && facts.has_rendered_descendants
    {
        if depth >= 256 {
            return PositionWithAffinity::default();
        }
        let downstream = VisuallyEquivalent(ctx, position.position.clone(), rule, false);
        if downstream != position.position
            && downstream
                .AnchorNode()
                .is_some_and(|node| ctx.HasLayoutObject(node))
        {
            return Compute(
                ctx,
                PositionWithAffinity {
                    position: downstream,
                    affinity: TextAffinity::Upstream,
                },
                depth + 1,
                rule,
            );
        }
        let upstream = VisuallyEquivalent(ctx, position.position.clone(), rule, true);
        if upstream == position.position
            || !upstream
                .AnchorNode()
                .is_some_and(|node| ctx.HasLayoutObject(node))
        {
            return PositionWithAffinity::default();
        }
        return Compute(
            ctx,
            PositionWithAffinity {
                position: upstream,
                affinity: TextAffinity::Upstream,
            },
            depth + 1,
            rule,
        );
    }
    if facts.is_atomic_inline {
        position
    } else {
        PositionWithAffinity::default()
    }
}
pub fn ComputeInlineAdjustedPosition<C: InlineAdjustmentContext>(
    ctx: &C,
    position: PositionWithAffinity<C::Node>,
    rule: EditingBoundaryCrossingRule,
) -> PositionWithAffinity<C::Node> {
    if position.position.IsNull() {
        return PositionWithAffinity::default();
    }
    Compute(ctx, position, 0, rule)
}
