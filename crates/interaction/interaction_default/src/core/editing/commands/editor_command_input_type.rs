// editor_command.cc:122-225,2228-2285. Source command identity decides event
// types and deletion target granularity, before any synchronous event dispatch.
use super::editor_command_table::EditingCommandType as C;
use crate::core::editing::text_granularity::TextGranularity as G;
use interaction_dispatch::core::events::input_event::InputType as I;
pub trait CommandInputTypeContext {
    fn CanEditRichly(&self) -> bool;
    fn VisibleSelectionIsRange(&self) -> bool;
}
pub fn InputTypeFromCommandType(command: C, ctx: &impl CommandInputTypeContext) -> I {
    let deletion = |range, caret| {
        if ctx.VisibleSelectionIsRange() {
            range
        } else {
            caret
        }
    };
    match command {
        C::InsertNewline => {
            if ctx.CanEditRichly() {
                I::kInsertParagraph
            } else {
                I::kInsertLineBreak
            }
        }
        C::InsertBacktab | C::InsertText => I::kInsertText,
        C::InsertLineBreak => I::kInsertLineBreak,
        C::InsertParagraph | C::InsertNewlineInQuotedContent => I::kInsertParagraph,
        C::InsertHorizontalRule => I::kInsertHorizontalRule,
        C::InsertOrderedList => I::kInsertOrderedList,
        C::InsertUnorderedList => I::kInsertUnorderedList,
        C::CreateLink => I::kInsertLink,
        C::Delete | C::DeleteBackward | C::DeleteBackwardByDecomposingPreviousCharacter => {
            I::kDeleteContentBackward
        }
        C::DeleteForward => I::kDeleteContentForward,
        C::DeleteToBeginningOfLine => {
            deletion(I::kDeleteContentBackward, I::kDeleteSoftLineBackward)
        }
        C::DeleteToEndOfLine => deletion(I::kDeleteContentForward, I::kDeleteSoftLineForward),
        C::DeleteWordBackward => deletion(I::kDeleteContentBackward, I::kDeleteWordBackward),
        C::DeleteWordForward => deletion(I::kDeleteContentForward, I::kDeleteWordForward),
        C::DeleteToBeginningOfParagraph => I::kDeleteHardLineBackward,
        C::DeleteToEndOfParagraph => I::kDeleteHardLineForward,
        C::Undo => I::kHistoryUndo,
        C::Redo => I::kHistoryRedo,
        C::Bold | C::ToggleBold => I::kFormatBold,
        C::Italic | C::ToggleItalic => I::kFormatItalic,
        C::Underline | C::ToggleUnderline => I::kFormatUnderline,
        C::Strikethrough => I::kFormatStrikeThrough,
        C::Superscript => I::kFormatSuperscript,
        C::Subscript => I::kFormatSubscript,
        _ => I::kNone,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionModifyDirection {
    Backward,
    Forward,
    Left,
    Right,
}
/// The target-range extension differs deliberately from delete execution:
/// DeleteToEndOfLine reports kLine, while its execution uses LineBoundary.
pub fn DeletionTargetGranularity(command: C) -> Option<(SelectionModifyDirection, G)> {
    use SelectionModifyDirection::{Backward as B, Forward as F};
    Some(match command {
        C::Delete | C::DeleteBackward => (B, G::kCharacter),
        C::DeleteForward => (F, G::kCharacter),
        C::DeleteToBeginningOfLine => (B, G::kLineBoundary),
        C::DeleteToBeginningOfParagraph => (B, G::kParagraph),
        C::DeleteToEndOfLine => (F, G::kLine),
        C::DeleteToEndOfParagraph => (F, G::kParagraph),
        C::DeleteWordBackward => (B, G::kWord),
        C::DeleteWordForward => (F, G::kWord),
        _ => return None,
    })
}
