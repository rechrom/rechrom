// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source insert_line_break_command.h:34-44; .cc:70-146,210-267.
use super::{
    composite_edit_command::{InsertAfter, InsertNodeAt},
    delete_selection_command::DeleteSelectionCommand,
    paragraph_editing_context::{EditResult, EditingAborted},
    structural_editing_context::{StructuralEditingContext, StructuralPositionQuery},
};
use crate::core::editing::{
    commands::structural_editing_support::{ParentAfter, ParentBefore},
    position::Position,
    visible_units_paragraph::IsParagraphEdge,
};

/// Source insert_line_break_command.cc:70-146,210-267. Preserve the source
/// preformatted LF branch; rich editing inserts an actual BR and, at the end
/// of a paragraph, the additional placeholder needed to expose the new line.
pub fn InsertLineBreak<C: StructuralEditingContext>(
    ctx: &C,
    selection: (Position<u64>, Position<u64>),
) -> EditResult<Position<u64>> {
    let mut position = if selection.0 != selection.1 {
        DeleteSelectionCommand::Apply(ctx, selection)?
    } else {
        selection.0
    };
    ctx.UpdateStyleAndLayout();
    position = ctx.QueryPosition(&position, StructuralPositionQuery::Forward);
    let node = position.ComputeContainerNode(ctx).ok_or(EditingAborted)?;
    if ctx.IsText(&node) && ctx.NodeFacts(node).preserve_breaks {
        let offset = position.ComputeOffsetInContainerNode(ctx);
        ctx.InsertText(node, offset, &[10])?;
        position = Position::new(node, offset + 1);
    } else {
        let at_end = IsParagraphEdge(ctx, &position, false);
        let br = ctx.CreateElement("br")?;
        InsertNodeAt(ctx, br, &position)?;
        position = ParentAfter(ctx, br);
        if at_end {
            let placeholder = ctx.CreateElement("br")?;
            InsertAfter(ctx, placeholder, br)?;
            position = ParentBefore(ctx, placeholder);
        }
    }
    ctx.SetEndingPosition(&position);
    Ok(position)
}
