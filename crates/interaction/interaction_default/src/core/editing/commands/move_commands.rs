//! Chromium move_commands.cc:189-661. Command identities select the existing
//! FrameSelection/SelectionModifier operation; geometry remains with its owner.
use super::editor_command_input_type::SelectionModifyDirection as D;
use super::editor_command_table::ExecuteFunction as F;
use crate::core::editing::{
    selection_modifier::SelectionModifyAlteration as A, text_granularity::TextGranularity as G,
};

pub fn Movement(command: F) -> Option<(A, D, G, bool)> {
    let (alter, direction, granularity) = match command {
        F::MoveCommands_ExecuteMoveBackward => (A::Move, D::Backward, G::kCharacter),
        F::MoveCommands_ExecuteMoveBackwardAndModifySelection => {
            (A::Extend, D::Backward, G::kCharacter)
        }
        F::MoveCommands_ExecuteMoveForward => (A::Move, D::Forward, G::kCharacter),
        F::MoveCommands_ExecuteMoveForwardAndModifySelection => {
            (A::Extend, D::Forward, G::kCharacter)
        }
        F::MoveCommands_ExecuteMoveLeft => (A::Move, D::Left, G::kCharacter),
        F::MoveCommands_ExecuteMoveLeftAndModifySelection => (A::Extend, D::Left, G::kCharacter),
        F::MoveCommands_ExecuteMoveRight => (A::Move, D::Right, G::kCharacter),
        F::MoveCommands_ExecuteMoveRightAndModifySelection => (A::Extend, D::Right, G::kCharacter),
        F::MoveCommands_ExecuteMoveUp => (A::Move, D::Backward, G::kLine),
        F::MoveCommands_ExecuteMoveUpAndModifySelection => (A::Extend, D::Backward, G::kLine),
        F::MoveCommands_ExecuteMoveDown => (A::Move, D::Forward, G::kLine),
        F::MoveCommands_ExecuteMoveDownAndModifySelection => (A::Extend, D::Forward, G::kLine),
        F::MoveCommands_ExecuteMoveParagraphBackward => (A::Move, D::Backward, G::kParagraph),
        F::MoveCommands_ExecuteMoveParagraphBackwardAndModifySelection => {
            (A::Extend, D::Backward, G::kParagraph)
        }
        F::MoveCommands_ExecuteMoveParagraphForward => (A::Move, D::Forward, G::kParagraph),
        F::MoveCommands_ExecuteMoveParagraphForwardAndModifySelection => {
            (A::Extend, D::Forward, G::kParagraph)
        }
        F::MoveCommands_ExecuteMoveWordBackward => (A::Move, D::Backward, G::kWord),
        F::MoveCommands_ExecuteMoveWordBackwardAndModifySelection => {
            (A::Extend, D::Backward, G::kWord)
        }
        F::MoveCommands_ExecuteMoveWordForward => (A::Move, D::Forward, G::kWord),
        F::MoveCommands_ExecuteMoveWordForwardAndModifySelection => {
            (A::Extend, D::Forward, G::kWord)
        }
        F::MoveCommands_ExecuteMoveWordLeft => (A::Move, D::Left, G::kWord),
        F::MoveCommands_ExecuteMoveWordLeftAndModifySelection => (A::Extend, D::Left, G::kWord),
        F::MoveCommands_ExecuteMoveWordRight => (A::Move, D::Right, G::kWord),
        F::MoveCommands_ExecuteMoveWordRightAndModifySelection => (A::Extend, D::Right, G::kWord),
        F::MoveCommands_ExecuteMoveToBeginningOfDocument => {
            (A::Move, D::Backward, G::kDocumentBoundary)
        }
        F::MoveCommands_ExecuteMoveToBeginningOfDocumentAndModifySelection => {
            (A::Extend, D::Backward, G::kDocumentBoundary)
        }
        F::MoveCommands_ExecuteMoveToEndOfDocument => (A::Move, D::Forward, G::kDocumentBoundary),
        F::MoveCommands_ExecuteMoveToEndOfDocumentAndModifySelection => {
            (A::Extend, D::Forward, G::kDocumentBoundary)
        }
        F::MoveCommands_ExecuteMoveToBeginningOfLine => (A::Move, D::Backward, G::kLineBoundary),
        F::MoveCommands_ExecuteMoveToBeginningOfLineAndModifySelection => {
            (A::Extend, D::Backward, G::kLineBoundary)
        }
        F::MoveCommands_ExecuteMoveToEndOfLine => (A::Move, D::Forward, G::kLineBoundary),
        F::MoveCommands_ExecuteMoveToEndOfLineAndModifySelection => {
            (A::Extend, D::Forward, G::kLineBoundary)
        }
        F::MoveCommands_ExecuteMoveToLeftEndOfLine => (A::Move, D::Left, G::kLineBoundary),
        F::MoveCommands_ExecuteMoveToLeftEndOfLineAndModifySelection => {
            (A::Extend, D::Left, G::kLineBoundary)
        }
        F::MoveCommands_ExecuteMoveToRightEndOfLine => (A::Move, D::Right, G::kLineBoundary),
        F::MoveCommands_ExecuteMoveToRightEndOfLineAndModifySelection => {
            (A::Extend, D::Right, G::kLineBoundary)
        }
        F::MoveCommands_ExecuteMoveToBeginningOfParagraph => {
            (A::Move, D::Backward, G::kParagraphBoundary)
        }
        F::MoveCommands_ExecuteMoveToBeginningOfParagraphAndModifySelection => {
            (A::Extend, D::Backward, G::kParagraphBoundary)
        }
        F::MoveCommands_ExecuteMoveToEndOfParagraph => (A::Move, D::Forward, G::kParagraphBoundary),
        F::MoveCommands_ExecuteMoveToEndOfParagraphAndModifySelection => {
            (A::Extend, D::Forward, G::kParagraphBoundary)
        }
        F::MoveCommands_ExecuteMoveToBeginningOfSentence => {
            (A::Move, D::Backward, G::kSentenceBoundary)
        }
        F::MoveCommands_ExecuteMoveToBeginningOfSentenceAndModifySelection => {
            (A::Extend, D::Backward, G::kSentenceBoundary)
        }
        F::MoveCommands_ExecuteMoveToEndOfSentence => (A::Move, D::Forward, G::kSentenceBoundary),
        F::MoveCommands_ExecuteMoveToEndOfSentenceAndModifySelection => {
            (A::Extend, D::Forward, G::kSentenceBoundary)
        }
        _ => return None,
    };
    // These four source entry points return MoveSelection's result. The other
    // commands consume the key even when the modifier reaches a boundary.
    let returns_modified = matches!(
        command,
        F::MoveCommands_ExecuteMoveLeft
            | F::MoveCommands_ExecuteMoveRight
            | F::MoveCommands_ExecuteMoveUp
            | F::MoveCommands_ExecuteMoveDown
    );
    Some((alter, direction, granularity, returns_modified))
}

pub fn PageMovement(
    command: F,
) -> Option<(
    A,
    super::super::selection_modifier::SelectionModifyVerticalDirection,
)> {
    use super::super::selection_modifier::SelectionModifyVerticalDirection as V;
    Some(match command {
        F::MoveCommands_ExecuteMovePageDown => (A::Move, V::Down),
        F::MoveCommands_ExecuteMovePageDownAndModifySelection => (A::Extend, V::Down),
        F::MoveCommands_ExecuteMovePageUp => (A::Move, V::Up),
        F::MoveCommands_ExecuteMovePageUpAndModifySelection => (A::Extend, V::Up),
        _ => return None,
    })
}
/// Source :65-112. The host supplies resident LayoutBox/viewport facts.
pub fn VerticalScrollDistance(
    overflow_scrollable: bool,
    editable: bool,
    padding_height: i32,
    visible_height: i32,
) -> i32 {
    if !overflow_scrollable && !editable {
        return 0;
    }
    foundation::scroll_utils::CalculatePageStep(padding_height.min(visible_height))
}
pub fn PageMovementSelectionOptions(
    alter: A,
    behavior: super::super::editing_behavior::EditingBehaviorType,
) -> super::super::set_selection_options::SetSelectionOptions {
    use super::super::set_selection_options::{
        CursorAlignOnScroll, SetSelectionBy, SetSelectionOptions,
    };
    SetSelectionOptions {
        set_selection_by: SetSelectionBy::kUser,
        should_close_typing: true,
        should_clear_typing_style: true,
        cursor_align_on_scroll: if alter == A::Move {
            CursorAlignOnScroll::kAlways
        } else {
            CursorAlignOnScroll::kIfNeeded
        },
        is_directional: alter == A::Extend || behavior.ShouldConsiderSelectionAsDirectional(),
        ..Default::default()
    }
}
