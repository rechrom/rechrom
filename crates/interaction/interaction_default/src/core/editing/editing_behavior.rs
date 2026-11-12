// Copyright (C) 2006, 2007 Apple, Inc. All rights reserved.
// Copyright (C) 2012 Google Inc. All rights reserved.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: core/editing/editing_behavior.cc:48-337. Native key bindings and
//! writing-mode arrow transposition use the source platform branches.
use foundation::{LogicalToPhysical, TextDirection, WritingDirectionMode, WritingMode};
use interaction_dispatch::core::events::keyboard_event::KeyboardEvent;
use interaction_input::public::common::input::web_input_event::{Type, WebInputEvent};

/// Canonical EditorCommand identity; Browser does not reinterpret command names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorKeyboardCommand {
    Cancel,
    Copy,
    Cut,
    DeleteBackward,
    DeleteForward,
    DeleteToBeginningOfLine,
    DeleteWordBackward,
    DeleteWordForward,
    InsertBacktab,
    InsertLineBreak,
    InsertNewline,
    InsertTab,
    MoveDown,
    MoveDownAndModifySelection,
    MoveLeft,
    MoveLeftAndModifySelection,
    MovePageDown,
    MovePageDownAndModifySelection,
    MovePageUp,
    MovePageUpAndModifySelection,
    MoveParagraphBackward,
    MoveParagraphBackwardAndModifySelection,
    MoveParagraphForward,
    MoveParagraphForwardAndModifySelection,
    MoveRight,
    MoveRightAndModifySelection,
    MoveToBeginningOfDocument,
    MoveToBeginningOfDocumentAndModifySelection,
    MoveToBeginningOfLine,
    MoveToBeginningOfLineAndModifySelection,
    MoveToEndOfDocument,
    MoveToEndOfDocumentAndModifySelection,
    MoveToEndOfLine,
    MoveToEndOfLineAndModifySelection,
    MoveUp,
    MoveUpAndModifySelection,
    MoveWordLeft,
    MoveWordLeftAndModifySelection,
    MoveWordRight,
    MoveWordRightAndModifySelection,
    OverWrite,
    Paste,
    PasteAndMatchStyle,
    Redo,
    SelectAll,
    ToggleBold,
    ToggleItalic,
    ToggleUnderline,
    Undo,
}
impl EditorKeyboardCommand {
    pub fn CanonicalCommandType(self) -> super::commands::editor_command_table::EditingCommandType {
        use super::commands::editor_command_table::EditingCommandType as C;
        match self {
            Self::Cancel => C::Invalid,
            Self::Copy => C::Copy,
            Self::Cut => C::Cut,
            Self::DeleteBackward => C::DeleteBackward,
            Self::DeleteForward => C::DeleteForward,
            Self::DeleteToBeginningOfLine => C::DeleteToBeginningOfLine,
            Self::DeleteWordBackward => C::DeleteWordBackward,
            Self::DeleteWordForward => C::DeleteWordForward,
            Self::InsertBacktab => C::InsertBacktab,
            Self::InsertLineBreak => C::InsertLineBreak,
            Self::InsertNewline => C::InsertNewline,
            Self::InsertTab => C::InsertTab,
            Self::MoveDown => C::MoveDown,
            Self::MoveDownAndModifySelection => C::MoveDownAndModifySelection,
            Self::MoveLeft => C::MoveLeft,
            Self::MoveLeftAndModifySelection => C::MoveLeftAndModifySelection,
            Self::MovePageDown => C::MovePageDown,
            Self::MovePageDownAndModifySelection => C::MovePageDownAndModifySelection,
            Self::MovePageUp => C::MovePageUp,
            Self::MovePageUpAndModifySelection => C::MovePageUpAndModifySelection,
            Self::MoveParagraphBackward => C::MoveParagraphBackward,
            Self::MoveParagraphBackwardAndModifySelection => {
                C::MoveParagraphBackwardAndModifySelection
            }
            Self::MoveParagraphForward => C::MoveParagraphForward,
            Self::MoveParagraphForwardAndModifySelection => {
                C::MoveParagraphForwardAndModifySelection
            }
            Self::MoveRight => C::MoveRight,
            Self::MoveRightAndModifySelection => C::MoveRightAndModifySelection,
            Self::MoveToBeginningOfDocument => C::MoveToBeginningOfDocument,
            Self::MoveToBeginningOfDocumentAndModifySelection => {
                C::MoveToBeginningOfDocumentAndModifySelection
            }
            Self::MoveToBeginningOfLine => C::MoveToBeginningOfLine,
            Self::MoveToBeginningOfLineAndModifySelection => {
                C::MoveToBeginningOfLineAndModifySelection
            }
            Self::MoveToEndOfDocument => C::MoveToEndOfDocument,
            Self::MoveToEndOfDocumentAndModifySelection => C::MoveToEndOfDocumentAndModifySelection,
            Self::MoveToEndOfLine => C::MoveToEndOfLine,
            Self::MoveToEndOfLineAndModifySelection => C::MoveToEndOfLineAndModifySelection,
            Self::MoveUp => C::MoveUp,
            Self::MoveUpAndModifySelection => C::MoveUpAndModifySelection,
            Self::MoveWordLeft => C::MoveWordLeft,
            Self::MoveWordLeftAndModifySelection => C::MoveWordLeftAndModifySelection,
            Self::MoveWordRight => C::MoveWordRight,
            Self::MoveWordRightAndModifySelection => C::MoveWordRightAndModifySelection,
            Self::OverWrite => C::OverWrite,
            Self::Paste => C::Paste,
            Self::PasteAndMatchStyle => C::PasteAndMatchStyle,
            Self::Redo => C::Redo,
            Self::SelectAll => C::SelectAll,
            Self::ToggleBold => C::ToggleBold,
            Self::ToggleItalic => C::ToggleItalic,
            Self::ToggleUnderline => C::ToggleUnderline,
            Self::Undo => C::Undo,
        }
    }
    pub fn IsTextInsertion(self) -> bool {
        self.CanonicalCommandType()
            .Descriptor()
            .is_some_and(|command| command.is_text_insertion)
    }
}

#[cfg(target_os = "macos")]
const KEY_DOWN: &[(i32, i32, EditorKeyboardCommand)] = &[
    (0x25, 0, EditorKeyboardCommand::MoveLeft),
    (
        0x25,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveLeftAndModifySelection,
    ),
    (
        0x25,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::MoveWordLeft,
    ),
    (
        0x25,
        WebInputEvent::kAltKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveWordLeftAndModifySelection,
    ),
    (0x27, 0, EditorKeyboardCommand::MoveRight),
    (
        0x27,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveRightAndModifySelection,
    ),
    (
        0x27,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::MoveWordRight,
    ),
    (
        0x27,
        WebInputEvent::kAltKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveWordRightAndModifySelection,
    ),
    (0x26, 0, EditorKeyboardCommand::MoveUp),
    (
        0x26,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveUpAndModifySelection,
    ),
    (
        0x21,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MovePageUpAndModifySelection,
    ),
    (0x28, 0, EditorKeyboardCommand::MoveDown),
    (
        0x28,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveDownAndModifySelection,
    ),
    (
        0x22,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MovePageDownAndModifySelection,
    ),
    (
        0x26,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::MoveParagraphBackward,
    ),
    (
        0x28,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::MoveParagraphForward,
    ),
    (0x24, 0, EditorKeyboardCommand::MoveToBeginningOfLine),
    (
        0x24,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToBeginningOfLineAndModifySelection,
    ),
    (
        0x21,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::MovePageUp,
    ),
    (
        0x22,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::MovePageDown,
    ),
    (0x23, 0, EditorKeyboardCommand::MoveToEndOfLine),
    (
        0x23,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToEndOfLineAndModifySelection,
    ),
    (0x8, 0, EditorKeyboardCommand::DeleteBackward),
    (
        0x8,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::DeleteBackward,
    ),
    (0x2e, 0, EditorKeyboardCommand::DeleteForward),
    (
        0x8,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::DeleteWordBackward,
    ),
    (
        0x2e,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::DeleteWordForward,
    ),
    (
        0x42,
        WebInputEvent::kMetaKey,
        EditorKeyboardCommand::ToggleBold,
    ),
    (
        0x49,
        WebInputEvent::kMetaKey,
        EditorKeyboardCommand::ToggleItalic,
    ),
    (
        0x55,
        WebInputEvent::kMetaKey,
        EditorKeyboardCommand::ToggleUnderline,
    ),
    (0x1b, 0, EditorKeyboardCommand::Cancel),
    (
        0xbe,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Cancel,
    ),
    (0x9, 0, EditorKeyboardCommand::InsertTab),
    (
        0x9,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertBacktab,
    ),
    (0xd, 0, EditorKeyboardCommand::InsertNewline),
    (
        0xd,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kAltKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertLineBreak,
    ),
    (
        0x2d,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Copy,
    ),
    (0x2d, WebInputEvent::kShiftKey, EditorKeyboardCommand::Paste),
    (0x2e, WebInputEvent::kShiftKey, EditorKeyboardCommand::Cut),
    (0x2d, 0, EditorKeyboardCommand::OverWrite),
];

#[cfg(target_os = "windows")]
const KEY_DOWN: &[(i32, i32, EditorKeyboardCommand)] = &[
    (0x25, 0, EditorKeyboardCommand::MoveLeft),
    (
        0x25,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveLeftAndModifySelection,
    ),
    (
        0x25,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveWordLeft,
    ),
    (
        0x25,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveWordLeftAndModifySelection,
    ),
    (0x27, 0, EditorKeyboardCommand::MoveRight),
    (
        0x27,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveRightAndModifySelection,
    ),
    (
        0x27,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveWordRight,
    ),
    (
        0x27,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveWordRightAndModifySelection,
    ),
    (0x26, 0, EditorKeyboardCommand::MoveUp),
    (
        0x26,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveUpAndModifySelection,
    ),
    (
        0x21,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MovePageUpAndModifySelection,
    ),
    (0x28, 0, EditorKeyboardCommand::MoveDown),
    (
        0x28,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveDownAndModifySelection,
    ),
    (
        0x22,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MovePageDownAndModifySelection,
    ),
    (
        0x26,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveParagraphBackward,
    ),
    (
        0x28,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveParagraphForward,
    ),
    (
        0x26,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveParagraphBackwardAndModifySelection,
    ),
    (
        0x28,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveParagraphForwardAndModifySelection,
    ),
    (0x21, 0, EditorKeyboardCommand::MovePageUp),
    (0x22, 0, EditorKeyboardCommand::MovePageDown),
    (0x24, 0, EditorKeyboardCommand::MoveToBeginningOfLine),
    (
        0x24,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToBeginningOfLineAndModifySelection,
    ),
    (
        0x24,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveToBeginningOfDocument,
    ),
    (
        0x24,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToBeginningOfDocumentAndModifySelection,
    ),
    (0x23, 0, EditorKeyboardCommand::MoveToEndOfLine),
    (
        0x23,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToEndOfLineAndModifySelection,
    ),
    (
        0x23,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveToEndOfDocument,
    ),
    (
        0x23,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToEndOfDocumentAndModifySelection,
    ),
    (0x8, 0, EditorKeyboardCommand::DeleteBackward),
    (
        0x8,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::DeleteBackward,
    ),
    (0x2e, 0, EditorKeyboardCommand::DeleteForward),
    (
        0x8,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::DeleteWordBackward,
    ),
    (
        0x2e,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::DeleteWordForward,
    ),
    (
        0x42,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::ToggleBold,
    ),
    (
        0x49,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::ToggleItalic,
    ),
    (
        0x55,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::ToggleUnderline,
    ),
    (0x1b, 0, EditorKeyboardCommand::Cancel),
    (
        0xbe,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Cancel,
    ),
    (0x9, 0, EditorKeyboardCommand::InsertTab),
    (
        0x9,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertBacktab,
    ),
    (0xd, 0, EditorKeyboardCommand::InsertNewline),
    (
        0xd,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kAltKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertLineBreak,
    ),
    (
        0x2d,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Copy,
    ),
    (0x2d, WebInputEvent::kShiftKey, EditorKeyboardCommand::Paste),
    (0x2e, WebInputEvent::kShiftKey, EditorKeyboardCommand::Cut),
    (
        0x43,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Copy,
    ),
    (
        0x56,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Paste,
    ),
    (
        0x56,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::PasteAndMatchStyle,
    ),
    (0x58, WebInputEvent::kControlKey, EditorKeyboardCommand::Cut),
    (
        0x41,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::SelectAll,
    ),
    (
        0x5a,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Undo,
    ),
    (
        0x5a,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::Redo,
    ),
    (
        0x59,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Redo,
    ),
    (0x8, WebInputEvent::kAltKey, EditorKeyboardCommand::Undo),
    (
        0x8,
        WebInputEvent::kAltKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::Redo,
    ),
    (0x2d, 0, EditorKeyboardCommand::OverWrite),
];

#[cfg(target_os = "linux")]
const KEY_DOWN: &[(i32, i32, EditorKeyboardCommand)] = &[
    (0x25, 0, EditorKeyboardCommand::MoveLeft),
    (
        0x25,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveLeftAndModifySelection,
    ),
    (
        0x25,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveWordLeft,
    ),
    (
        0x25,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveWordLeftAndModifySelection,
    ),
    (0x27, 0, EditorKeyboardCommand::MoveRight),
    (
        0x27,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveRightAndModifySelection,
    ),
    (
        0x27,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveWordRight,
    ),
    (
        0x27,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveWordRightAndModifySelection,
    ),
    (0x26, 0, EditorKeyboardCommand::MoveUp),
    (
        0x26,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveUpAndModifySelection,
    ),
    (
        0x21,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MovePageUpAndModifySelection,
    ),
    (0x28, 0, EditorKeyboardCommand::MoveDown),
    (
        0x28,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveDownAndModifySelection,
    ),
    (
        0x22,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MovePageDownAndModifySelection,
    ),
    (
        0x26,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveParagraphBackward,
    ),
    (
        0x28,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveParagraphForward,
    ),
    (
        0x26,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveParagraphBackwardAndModifySelection,
    ),
    (
        0x28,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveParagraphForwardAndModifySelection,
    ),
    (0x21, 0, EditorKeyboardCommand::MovePageUp),
    (0x22, 0, EditorKeyboardCommand::MovePageDown),
    (0x24, 0, EditorKeyboardCommand::MoveToBeginningOfLine),
    (
        0x24,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToBeginningOfLineAndModifySelection,
    ),
    (
        0x24,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveToBeginningOfDocument,
    ),
    (
        0x24,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToBeginningOfDocumentAndModifySelection,
    ),
    (0x23, 0, EditorKeyboardCommand::MoveToEndOfLine),
    (
        0x23,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToEndOfLineAndModifySelection,
    ),
    (
        0x23,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveToEndOfDocument,
    ),
    (
        0x23,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToEndOfDocumentAndModifySelection,
    ),
    (0x8, 0, EditorKeyboardCommand::DeleteBackward),
    (
        0x8,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::DeleteBackward,
    ),
    (0x2e, 0, EditorKeyboardCommand::DeleteForward),
    (
        0x8,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::DeleteWordBackward,
    ),
    (
        0x2e,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::DeleteWordForward,
    ),
    (
        0x8,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::DeleteToBeginningOfLine,
    ),
    (
        0x42,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::ToggleBold,
    ),
    (
        0x49,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::ToggleItalic,
    ),
    (
        0x55,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::ToggleUnderline,
    ),
    (0x1b, 0, EditorKeyboardCommand::Cancel),
    (
        0xbe,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Cancel,
    ),
    (0x9, 0, EditorKeyboardCommand::InsertTab),
    (
        0x9,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertBacktab,
    ),
    (0xd, 0, EditorKeyboardCommand::InsertNewline),
    (
        0xd,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kAltKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertLineBreak,
    ),
    (
        0x2d,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Copy,
    ),
    (0x2d, WebInputEvent::kShiftKey, EditorKeyboardCommand::Paste),
    (0x2e, WebInputEvent::kShiftKey, EditorKeyboardCommand::Cut),
    (
        0x43,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Copy,
    ),
    (
        0x56,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Paste,
    ),
    (
        0x56,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::PasteAndMatchStyle,
    ),
    (0x58, WebInputEvent::kControlKey, EditorKeyboardCommand::Cut),
    (
        0x41,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::SelectAll,
    ),
    (
        0x5a,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Undo,
    ),
    (
        0x5a,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::Redo,
    ),
    (
        0x59,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Redo,
    ),
    (0x2d, 0, EditorKeyboardCommand::OverWrite),
];

#[cfg(target_os = "android")]
const KEY_DOWN: &[(i32, i32, EditorKeyboardCommand)] = &[
    (0x25, 0, EditorKeyboardCommand::MoveLeft),
    (
        0x25,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveLeftAndModifySelection,
    ),
    (
        0x25,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveWordLeft,
    ),
    (
        0x25,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveWordLeftAndModifySelection,
    ),
    (0x27, 0, EditorKeyboardCommand::MoveRight),
    (
        0x27,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveRightAndModifySelection,
    ),
    (
        0x27,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveWordRight,
    ),
    (
        0x27,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveWordRightAndModifySelection,
    ),
    (
        0x25,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::MoveToBeginningOfLine,
    ),
    (
        0x27,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::MoveToEndOfLine,
    ),
    (
        0x25,
        WebInputEvent::kAltKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToBeginningOfLineAndModifySelection,
    ),
    (
        0x27,
        WebInputEvent::kAltKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToEndOfLineAndModifySelection,
    ),
    (0x26, 0, EditorKeyboardCommand::MoveUp),
    (
        0x26,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveUpAndModifySelection,
    ),
    (
        0x21,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MovePageUpAndModifySelection,
    ),
    (0x28, 0, EditorKeyboardCommand::MoveDown),
    (
        0x28,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveDownAndModifySelection,
    ),
    (
        0x22,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MovePageDownAndModifySelection,
    ),
    (
        0x26,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveParagraphBackward,
    ),
    (
        0x28,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveParagraphForward,
    ),
    (
        0x26,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveParagraphBackwardAndModifySelection,
    ),
    (
        0x28,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveParagraphForwardAndModifySelection,
    ),
    (0x21, 0, EditorKeyboardCommand::MovePageUp),
    (0x22, 0, EditorKeyboardCommand::MovePageDown),
    (0x24, 0, EditorKeyboardCommand::MoveToBeginningOfLine),
    (
        0x24,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToBeginningOfLineAndModifySelection,
    ),
    (
        0x24,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveToBeginningOfDocument,
    ),
    (
        0x24,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToBeginningOfDocumentAndModifySelection,
    ),
    (0x23, 0, EditorKeyboardCommand::MoveToEndOfLine),
    (
        0x23,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToEndOfLineAndModifySelection,
    ),
    (
        0x23,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveToEndOfDocument,
    ),
    (
        0x23,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToEndOfDocumentAndModifySelection,
    ),
    (0x8, 0, EditorKeyboardCommand::DeleteBackward),
    (
        0x8,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::DeleteBackward,
    ),
    (0x2e, 0, EditorKeyboardCommand::DeleteForward),
    (
        0x8,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::DeleteWordBackward,
    ),
    (
        0x2e,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::DeleteWordForward,
    ),
    (
        0x42,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::ToggleBold,
    ),
    (
        0x49,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::ToggleItalic,
    ),
    (
        0x55,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::ToggleUnderline,
    ),
    (0x1b, 0, EditorKeyboardCommand::Cancel),
    (
        0xbe,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Cancel,
    ),
    (0x9, 0, EditorKeyboardCommand::InsertTab),
    (
        0x9,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertBacktab,
    ),
    (0xd, 0, EditorKeyboardCommand::InsertNewline),
    (
        0xd,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kAltKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertLineBreak,
    ),
    (
        0x2d,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Copy,
    ),
    (0x2d, WebInputEvent::kShiftKey, EditorKeyboardCommand::Paste),
    (0x2e, WebInputEvent::kShiftKey, EditorKeyboardCommand::Cut),
    (
        0x43,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Copy,
    ),
    (
        0x56,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Paste,
    ),
    (
        0x56,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::PasteAndMatchStyle,
    ),
    (0x58, WebInputEvent::kControlKey, EditorKeyboardCommand::Cut),
    (
        0x41,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::SelectAll,
    ),
    (
        0x5a,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Undo,
    ),
    (
        0x5a,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::Redo,
    ),
    (
        0x59,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Redo,
    ),
    (0x2d, 0, EditorKeyboardCommand::OverWrite),
    (
        0x8,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::DeleteToBeginningOfLine,
    ),
];

#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "android"
)))]
const KEY_DOWN: &[(i32, i32, EditorKeyboardCommand)] = &[
    (0x25, 0, EditorKeyboardCommand::MoveLeft),
    (
        0x25,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveLeftAndModifySelection,
    ),
    (
        0x25,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveWordLeft,
    ),
    (
        0x25,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveWordLeftAndModifySelection,
    ),
    (0x27, 0, EditorKeyboardCommand::MoveRight),
    (
        0x27,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveRightAndModifySelection,
    ),
    (
        0x27,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveWordRight,
    ),
    (
        0x27,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveWordRightAndModifySelection,
    ),
    (0x26, 0, EditorKeyboardCommand::MoveUp),
    (
        0x26,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveUpAndModifySelection,
    ),
    (
        0x21,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MovePageUpAndModifySelection,
    ),
    (0x28, 0, EditorKeyboardCommand::MoveDown),
    (
        0x28,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveDownAndModifySelection,
    ),
    (
        0x22,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MovePageDownAndModifySelection,
    ),
    (
        0x26,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveParagraphBackward,
    ),
    (
        0x28,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveParagraphForward,
    ),
    (
        0x26,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveParagraphBackwardAndModifySelection,
    ),
    (
        0x28,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveParagraphForwardAndModifySelection,
    ),
    (0x21, 0, EditorKeyboardCommand::MovePageUp),
    (0x22, 0, EditorKeyboardCommand::MovePageDown),
    (0x24, 0, EditorKeyboardCommand::MoveToBeginningOfLine),
    (
        0x24,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToBeginningOfLineAndModifySelection,
    ),
    (
        0x24,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveToBeginningOfDocument,
    ),
    (
        0x24,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToBeginningOfDocumentAndModifySelection,
    ),
    (0x23, 0, EditorKeyboardCommand::MoveToEndOfLine),
    (
        0x23,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToEndOfLineAndModifySelection,
    ),
    (
        0x23,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::MoveToEndOfDocument,
    ),
    (
        0x23,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::MoveToEndOfDocumentAndModifySelection,
    ),
    (0x8, 0, EditorKeyboardCommand::DeleteBackward),
    (
        0x8,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::DeleteBackward,
    ),
    (0x2e, 0, EditorKeyboardCommand::DeleteForward),
    (
        0x8,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::DeleteWordBackward,
    ),
    (
        0x2e,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::DeleteWordForward,
    ),
    (
        0x42,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::ToggleBold,
    ),
    (
        0x49,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::ToggleItalic,
    ),
    (
        0x55,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::ToggleUnderline,
    ),
    (0x1b, 0, EditorKeyboardCommand::Cancel),
    (
        0xbe,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Cancel,
    ),
    (0x9, 0, EditorKeyboardCommand::InsertTab),
    (
        0x9,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertBacktab,
    ),
    (0xd, 0, EditorKeyboardCommand::InsertNewline),
    (
        0xd,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kAltKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kAltKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertNewline,
    ),
    (
        0xd,
        WebInputEvent::kShiftKey,
        EditorKeyboardCommand::InsertLineBreak,
    ),
    (
        0x2d,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Copy,
    ),
    (0x2d, WebInputEvent::kShiftKey, EditorKeyboardCommand::Paste),
    (0x2e, WebInputEvent::kShiftKey, EditorKeyboardCommand::Cut),
    (
        0x43,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Copy,
    ),
    (
        0x56,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Paste,
    ),
    (
        0x56,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::PasteAndMatchStyle,
    ),
    (0x58, WebInputEvent::kControlKey, EditorKeyboardCommand::Cut),
    (
        0x41,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::SelectAll,
    ),
    (
        0x5a,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Undo,
    ),
    (
        0x5a,
        WebInputEvent::kControlKey | WebInputEvent::kShiftKey,
        EditorKeyboardCommand::Redo,
    ),
    (
        0x59,
        WebInputEvent::kControlKey,
        EditorKeyboardCommand::Redo,
    ),
    (0x2d, 0, EditorKeyboardCommand::OverWrite),
];

pub fn InterpretKeyEvent(
    event: &KeyboardEvent,
    writing_mode: WritingMode,
) -> Option<EditorKeyboardCommand> {
    let key = event.KeyEvent()?;
    let modifiers = key.GetModifiers()
        & (WebInputEvent::kShiftKey
            | WebInputEvent::kAltKey
            | WebInputEvent::kControlKey
            | WebInputEvent::kMetaKey);
    if key.GetType() == Type::kRawKeyDown {
        let mapping = LogicalToPhysical::new(
            WritingDirectionMode::new(writing_mode, TextDirection::kLtr),
            0x25,
            0x27,
            0x26,
            0x28,
        );
        let code = match event.keyCode() {
            0x25 => mapping.Left(),
            0x27 => mapping.Right(),
            0x26 => mapping.Top(),
            0x28 => mapping.Bottom(),
            code => code,
        };
        if let Some((_, _, command)) = KEY_DOWN
            .iter()
            .find(|(key, mods, _)| *key == code && *mods == modifiers)
        {
            return Some(*command);
        }
        if modifiers == 0 {
            return match event.key() {
                "Copy" => Some(EditorKeyboardCommand::Copy),
                "Cut" => Some(EditorKeyboardCommand::Cut),
                "Paste" => Some(EditorKeyboardCommand::Paste),
                _ => None,
            };
        }
        return None;
    }
    match (event.charCode(), modifiers) {
        (9, 0) => Some(EditorKeyboardCommand::InsertTab),
        (9, WebInputEvent::kShiftKey) => Some(EditorKeyboardCommand::InsertBacktab),
        (13, 0) => Some(EditorKeyboardCommand::InsertNewline),
        (13, WebInputEvent::kShiftKey) => Some(EditorKeyboardCommand::InsertLineBreak),
        _ => None,
    }
}
pub fn ShouldInsertCharacter(event: &KeyboardEvent) -> bool {
    let Some(key) = event.KeyEvent() else {
        return false;
    };
    if key.text[1] != 0 {
        return true;
    }
    let ch = key.text[0];
    if ch < 0x20 {
        return false;
    }
    #[cfg(target_os = "linux")]
    if event.ctrlKey() {
        return false;
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    if ch < 0x80 {
        if event.ctrlKey() && !event.altKey() {
            return false;
        }
        #[cfg(target_os = "macos")]
        if event.metaKey() {
            return false;
        }
    }
    true
}

/// Source mojom::EditingBehavior, used by the retained Page Settings owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditingBehaviorType {
    Mac,
    Windows,
    Unix,
    Android,
    ChromeOS,
}
impl EditingBehaviorType {
    /// Source editing_behavior.h:58-66.
    pub fn ShouldConsiderSelectionAsDirectional(self) -> bool {
        self != Self::Mac
    }
    pub fn ShouldCenterAlignWhenSelectionIsRevealed(self) -> bool {
        self == Self::Mac
    }
    /// Source core/editing/editing_behavior.h:85-88.
    pub fn ShouldSelectOnContextualMenuClick(self) -> bool {
        matches!(self, Self::Mac | Self::ChromeOS)
    }
}

impl EditingBehaviorType {
    /// editing_behavior.h:131-134: platform behavior, not clipboard availability.
    pub fn SupportsGlobalSelection(self) -> bool {
        !matches!(self, Self::Windows | Self::Mac)
    }
}

impl EditingBehaviorType {
    // editing_behavior.h:46-50,79-81,94-105.
    pub fn ShouldAlwaysGrowSelectionWhenExtendingToBoundary(self) -> bool {
        self == Self::Mac
    }
    pub fn ShouldExtendSelectionByWordOrLineAcrossCaret(self) -> bool {
        self != Self::Mac
    }
    pub fn ShouldSkipSpaceWhenMovingRight(self) -> bool {
        self == Self::Windows
    }
    pub fn ShouldMoveCaretToHorizontalBoundaryWhenPastTopOrBottom(self) -> bool {
        !matches!(self, Self::Windows | Self::Android | Self::ChromeOS)
    }
}

impl EditingBehaviorType {
    /// editing_behavior.h:107-120.
    pub fn ShouldUndoOfDeleteSelectText(self) -> bool {
        self == Self::Mac
    }
    pub fn ShouldMergeContentWithTablesOnBackspace(self) -> bool {
        self == Self::Mac
    }
}

impl EditingBehaviorType {
    /// editing_behavior.h:69-74.
    pub fn ShouldToggleStyleBasedOnStartOfSelection(self) -> bool {
        self == Self::Mac
    }
}
