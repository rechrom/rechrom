// Copyright (C) 2006, 2007 Apple, Inc. All rights reserved.
// Copyright (C) 2012 Google, Inc. All rights reserved.
// BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/editor.h:82,277; editor_key_bindings.cc:46-135.
//! Editor's keyboard transaction. Commands, live selection and event dispatch
//! remain with their existing owners; no document or input state is copied.
use super::{
    commands::editor_command_table::EditingCommandType, editing_behavior,
    ime::active_edit_context::ActiveEditContext,
};
use foundation::WritingMode;
use interaction_dispatch::core::{
    dom::events::{event::EventTargetRef, event_dispatch_result::DispatchEventResult},
    events::keyboard_event::KeyboardEventRef,
};
use interaction_input::public::common::input::web_input_event::Type;
use std::rc::Rc;

pub trait EditorKeyboardContext {
    fn HandleCurrentKeyboardEvent(&self) -> bool;
    fn SelectionFocusOrFocusedNodeWritingMode(&self) -> WritingMode;
    fn ExecuteCommand(&self, command: EditingCommandType, event: &KeyboardEventRef) -> bool;
    fn GetActiveEditContext(&self) -> Option<Rc<dyn ActiveEditContext>>;
    fn FocusedElement(&self) -> Option<EventTargetRef>;
    fn SelectionHasFocus(&self) -> bool;
    fn CanEdit(&self) -> bool;
    fn IsTextControl(&self, element: &EventTargetRef) -> bool;
    fn DelegatesFocusTextControlInputFixEnabled(&self) -> bool;
    fn DispatchBeforeInputInsertText(
        &self,
        target: Option<&EventTargetRef>,
        text: &[u16],
    ) -> DispatchEventResult;
    fn InsertText(&self, text: &[u16], event: &KeyboardEventRef) -> bool;
}

// Source: editor_key_bindings.cc:46-126.
pub fn HandleEditingKeyboardEvent(
    context: &impl EditorKeyboardContext,
    event: &KeyboardEventRef,
) -> bool {
    // The event owns this immutable WebKeyboardEvent. Clone its small input
    // value so no RefCell borrow crosses command execution or script reentry.
    let Some(key) = event.borrow().KeyEvent().cloned() else {
        return false;
    };
    let writing_mode = context.SelectionFocusOrFocusedNodeWritingMode();
    let command = editing_behavior::InterpretKeyEvent(&event.borrow(), writing_mode)
        .map(|command| command.CanonicalCommandType());
    if key.GetType() == Type::kRawKeyDown {
        if command.is_none()
            || command
                .and_then(EditingCommandType::Descriptor)
                .is_some_and(|descriptor| descriptor.is_text_insertion)
        {
            return false;
        }
        return context.ExecuteCommand(command.expect("nonempty source command"), event);
    }
    if command.is_some_and(|command| context.ExecuteCommand(command, event)) {
        return true;
    }
    if !editing_behavior::ShouldInsertCharacter(&event.borrow()) {
        return false;
    }
    // Source String(char16_t*) consumes the original null-terminated text.
    let text = &key.text[..key
        .text
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(key.text.len())];
    if let Some(edit_context) = context.GetActiveEditContext() {
        let target = event.borrow().RawTarget();
        if context.DispatchBeforeInputInsertText(target.as_ref(), text)
            != DispatchEventResult::kNotCanceled
        {
            return true;
        }
        edit_context.InsertText(text);
        return true;
    }
    // Commands and beforeinput can synchronously change focus and selection.
    let Some(focused_element) = context.FocusedElement() else {
        return false;
    };
    if !context.SelectionHasFocus() {
        return false;
    }
    if !context.CanEdit()
        && !(context.DelegatesFocusTextControlInputFixEnabled()
            && context.IsTextControl(&focused_element))
    {
        return false;
    }
    let target = event.borrow().RawTarget();
    if context.DispatchBeforeInputInsertText(target.as_ref(), text)
        != DispatchEventResult::kNotCanceled
    {
        return true;
    }
    context.InsertText(text, event)
}

// Source: editor_key_bindings.cc:128-135.
pub fn HandleKeyboardEvent(context: &impl EditorKeyboardContext, event: &KeyboardEventRef) {
    if context.HandleCurrentKeyboardEvent() || HandleEditingKeyboardEvent(context, event) {
        event.borrow_mut().SetDefaultHandled();
    }
}
