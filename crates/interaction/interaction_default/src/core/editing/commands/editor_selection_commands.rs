//! editor_command.cc:260-281,325-342,382-396. Geometry is borrowed from the
//! existing native position owner; no alternate selection or text model.
use crate::core::editing::{
    editing_boundaries::EditableBoundaryContext,
    position::{Position, PositionWithAffinity},
    selection_adjuster::GranularityContext,
    selection_modifier::SelectionModifierContext,
    text_granularity::TextGranularity,
    visible_navigation,
    visible_position::VisiblePosition,
    visible_selection::{self, VisibleSelection},
    visible_units::EditingBoundaryCrossingRule as Rule,
    visible_units_commands::{self, CommandPositionContext},
};
use std::cmp::Ordering;
pub fn ExpandSelectionToGranularity<C: GranularityContext + EditableBoundaryContext>(
    ctx: &C,
    selection: &VisibleSelection<C::Node>,
    granularity: TextGranularity,
) -> Option<(Position<C::Node>, Position<C::Node>)> {
    let expanded = visible_selection::ExpandWithGranularity(ctx, selection, granularity);
    let range = visible_selection::NormalizeSelectionRange(ctx, &expanded.anchor, &expanded.focus)?;
    (range.0 != range.1).then_some(range)
}
pub fn UnionEphemeralRanges<C: EditableBoundaryContext>(
    ctx: &C,
    a: (Position<C::Node>, Position<C::Node>),
    b: (Position<C::Node>, Position<C::Node>),
) -> Option<(Position<C::Node>, Position<C::Node>)> {
    // Retain the source endpoint comparisons, including its <= end choice.
    let start = if ctx.ComparePositions(&a.0, &b.0)? != Ordering::Greater {
        a.0
    } else {
        b.0
    };
    let end = if ctx.ComparePositions(&a.1, &b.1)? != Ordering::Greater {
        a.1
    } else {
        b.1
    };
    Some((start, end))
}
pub fn ComputeRangeForTranspose<C: SelectionModifierContext + CommandPositionContext>(
    ctx: &C,
    selection: &VisibleSelection<C::Node>,
) -> Option<(Position<C::Node>, Position<C::Node>)> {
    if selection.anchor.IsNull() || selection.anchor != selection.focus {
        return None;
    }
    let caret = VisiblePosition::Create(
        ctx,
        PositionWithAffinity {
            position: selection.anchor.clone(),
            affinity: selection.affinity,
        },
    );
    let next = if visible_units_commands::IsEndOfParagraph(ctx, &caret) {
        caret
    } else {
        visible_navigation::NextPositionOf(ctx, caret.ToPositionWithAffinity(), Rule::CannotCross)
    };
    let previous =
        visible_navigation::PreviousPositionOf(ctx, next.DeepEquivalent(), Rule::CannotCross);
    if next.DeepEquivalent() == previous.DeepEquivalent() {
        return None;
    }
    let before =
        visible_navigation::PreviousPositionOf(ctx, previous.DeepEquivalent(), Rule::CannotCross);
    if next.IsNull()
        || visible_units_commands::StartOfParagraph(ctx, &next).DeepEquivalent()
            != visible_units_commands::StartOfParagraph(ctx, &before).DeepEquivalent()
    {
        return None;
    }
    Some((
        before.DeepEquivalent().clone(),
        next.DeepEquivalent().clone(),
    ))
}

pub trait MarkCommandContext {
    type Selection;
    type Error;
    fn CurrentVisibleSelection(&self) -> Option<Self::Selection>;
    fn MarkVisibleSelection(&self) -> Option<Self::Selection>;
    fn MarkIsDirectional(&self) -> bool;
    fn SetMarkFromCurrentSelection(&self);
    fn UnionMarkAndSelection(
        &self,
        mark: Self::Selection,
        selection: Self::Selection,
    ) -> Result<Option<Self::Selection>, Self::Error>;
    fn SetSelection(
        &self,
        selection: Self::Selection,
        options: crate::core::editing::set_selection_options::SetSelectionOptions,
    );
    fn DeleteSelectionWithKillRing(&self) -> Result<(), Self::Error>;
    fn UpdateStyleAndLayout(&self);
}
pub fn ExecuteMarkCommand<C: MarkCommandContext>(
    ctx: &C,
    command: super::editor_command_table::ExecuteFunction,
) -> Option<Result<bool, C::Error>> {
    use super::editor_command_table::ExecuteFunction as F;
    use crate::core::editing::set_selection_options::SetSelectionOptions as O;
    if !matches!(
        command,
        F::ExecuteSetMark
            | F::ExecuteSwapWithMark
            | F::ExecuteSelectToMark
            | F::ExecuteDeleteToMark
    ) {
        return None;
    }
    Some((|| {
        match command {
            F::ExecuteSetMark => ctx.SetMarkFromCurrentSelection(),
            F::ExecuteSwapWithMark => {
                let Some(mark) = ctx.MarkVisibleSelection() else {
                    return Ok(false);
                };
                if ctx.CurrentVisibleSelection().is_none() {
                    return Ok(false);
                }
                let directional = ctx.MarkIsDirectional();
                ctx.SetMarkFromCurrentSelection();
                ctx.SetSelection(
                    mark,
                    O {
                        is_directional: directional,
                        ..Default::default()
                    },
                );
            }
            F::ExecuteSelectToMark => {
                let Some((mark, selection)) = ctx
                    .MarkVisibleSelection()
                    .zip(ctx.CurrentVisibleSelection())
                else {
                    return Ok(false);
                };
                let Some(union) = ctx.UnionMarkAndSelection(mark, selection)? else {
                    return Ok(false);
                };
                ctx.SetSelection(
                    union,
                    O {
                        should_close_typing: true,
                        ..Default::default()
                    },
                );
            }
            F::ExecuteDeleteToMark => {
                if let Some((mark, selection)) = ctx
                    .MarkVisibleSelection()
                    .zip(ctx.CurrentVisibleSelection())
                {
                    if let Some(union) = ctx.UnionMarkAndSelection(mark, selection)? {
                        ctx.SetSelection(
                            union,
                            O {
                                should_close_typing: true,
                                ..Default::default()
                            },
                        );
                    }
                }
                ctx.DeleteSelectionWithKillRing()?;
                ctx.UpdateStyleAndLayout();
                ctx.SetMarkFromCurrentSelection();
            }
            _ => unreachable!(),
        }
        Ok(true)
    })())
}
pub trait TransposeCommandContext {
    type Range;
    type Error;
    fn PrepareForEditing(&self);
    fn ComputeRangeAndText(&self) -> Result<Option<(Self::Range, [u16; 2])>, Self::Error>;
    fn DispatchBeforeInput(
        &self,
        range: &Self::Range,
        text: &[u16; 2],
    ) -> Result<bool, Self::Error>;
    fn IsDocumentAttached(&self) -> bool;
    fn SelectRangeIfDifferent(&self, range: &Self::Range);
    fn ReplaceSelection(&self, text: &[u16; 2]) -> Result<(), Self::Error>;
}
/// editor_command.cc:964-1037. beforeinput is deliberately between two
/// independent reads of the original range and text owners.
pub fn ExecuteTranspose<C: TransposeCommandContext>(ctx: &C) -> Result<bool, C::Error> {
    ctx.PrepareForEditing();
    let Some((range, text)) = ctx.ComputeRangeAndText()? else {
        return Ok(false);
    };
    if !ctx.DispatchBeforeInput(&range, &[text[1], text[0]])? || !ctx.IsDocumentAttached() {
        return Ok(false);
    }
    ctx.PrepareForEditing();
    let Some((range, text)) = ctx.ComputeRangeAndText()? else {
        return Ok(false);
    };
    ctx.SelectRangeIfDifferent(&range);
    ctx.ReplaceSelection(&[text[1], text[0]])?;
    Ok(true)
}
