// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source visible_units_paragraph.cc:161-183.
//! Paragraph-edge equivalence through the existing structural host queries.
use super::{
    commands::structural_editing_context::{StructuralEditingContext, StructuralPositionQuery},
    position::Position,
};

pub(crate) fn IsParagraphEdge<C: StructuralEditingContext>(
    ctx: &C,
    p: &Position<u64>,
    start: bool,
) -> bool {
    let visible = ctx.QueryPosition(p, StructuralPositionQuery::Visible);
    let edge = ctx.QueryPosition(
        p,
        if start {
            StructuralPositionQuery::ParagraphStart
        } else {
            StructuralPositionQuery::ParagraphEnd
        },
    );
    !visible.IsNull() && visible == ctx.QueryPosition(&edge, StructuralPositionQuery::Visible)
}
