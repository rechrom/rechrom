//! Scoped Editor keyboard leaves. Source: editor_key_bindings.cc:46-135;
//! editing_utilities.cc:1750-1779; web_frame_widget_impl.cc:4164-4185.
use super::*;
use crate::adapter::{
    page_event_dispatcher::EditorCommandError,
    page_frame_selection::{CompositionTextQuery, CompositionTextResult},
    page_input_method_controller::event_handler_text_input::{
        PageEventHandlerTextInput, TextInputError,
    },
};
use interaction_default::core::editing::{
    commands::editor_command_table::EditingCommandType,
    editor_key_bindings::{self, EditorKeyboardContext},
    ime::active_edit_context::ActiveEditContext,
};

struct CommandEventBinding<'scope, 'listener, 'frame> {
    dispatcher: &'scope EventDispatcher<'listener, 'frame>,
    previous: Option<EventRef>,
}
impl Drop for CommandEventBinding<'_, '_, '_> {
    fn drop(&mut self) {
        self.dispatcher
            .ReplaceEditingCommandEvent(self.previous.take());
    }
}
struct Context<'scope, 'listener, 'frame> {
    editor: &'scope PageKeyboardEditor<'listener, 'frame>,
    text_input: Rc<PageEventHandlerTextInput<'listener, 'frame>>,
}
impl<'listener: 'frame, 'frame> Context<'_, 'listener, 'frame> {
    fn Fault(&self, error: &'static str) {
        self.editor.error.borrow_mut().get_or_insert(error);
    }
    fn Execute(&self, command: EditingCommandType) -> bool {
        if self.editor.error.borrow().is_some() {
            return false;
        }
        match self.editor.dispatcher.ExecuteEditorCommand(command) {
            Ok(handled) => handled,
            Err(EditorCommandError::RequiredPort(error)) => {
                self.Fault(error);
                false
            }
            Err(_) => {
                self.Fault("Editor keyboard command execution");
                false
            }
        }
    }
    fn Finish(&self) {
        if let Err(error) = self.text_input.Finish() {
            self.Fault(match error {
                TextInputError::DetachedTarget => "Editor text input detached target",
                TextInputError::NotTextEvent => "Editor text input event subclass",
                TextInputError::Editor(error) => error,
            });
        }
    }
}
impl<'listener: 'frame, 'frame> EditorKeyboardContext for Context<'_, 'listener, 'frame> {
    fn HandleCurrentKeyboardEvent(&self) -> bool {
        // Move the embedder's original vector out before running JS, exactly
        // as WebFrameWidgetImpl does. Reentry must not execute it a second time.
        let commands = self.editor.edit_commands.take();
        let dispatcher = &self.editor.dispatcher;
        let previous = dispatcher.ReplaceEditingCommandEvent(None);
        let _binding = CommandEventBinding {
            dispatcher,
            previous,
        };
        let mut executed = false;
        for command in commands {
            if !self
                .text_input
                .WithDefaultHandling(|| self.Execute(command))
            {
                break;
            }
            executed = true;
        }
        executed
    }
    fn SelectionFocusOrFocusedNodeWritingMode(&self) -> foundation::WritingMode {
        match self.editor.dispatcher.EditingKeyboardWritingMode() {
            Ok(mode) => mode,
            Err(error) => {
                self.Fault(error);
                foundation::WritingMode::kHorizontalTb
            }
        }
    }
    fn ExecuteCommand(&self, command: EditingCommandType, event: &KeyboardEventRef) -> bool {
        let dispatcher = &self.editor.dispatcher;
        let previous =
            dispatcher.ReplaceEditingCommandEvent(Some(EventRef::FromShared(event.clone())));
        let _binding = CommandEventBinding {
            dispatcher,
            previous,
        };
        self.text_input
            .WithDefaultHandling(|| self.Execute(command))
    }
    fn GetActiveEditContext(&self) -> Option<Rc<dyn ActiveEditContext>> {
        self.editor
            .dispatcher
            .input_method_state
            .as_ref()
            .and_then(|state| state.GetActiveEditContext())
    }
    fn FocusedElement(&self) -> Option<EventTargetRef> {
        let dispatcher = &self.editor.dispatcher;
        let id = dispatcher.KeyboardState().borrow().focused_node_id?;
        dispatcher.KeyboardTargets().ResolveTarget(id)
    }
    fn SelectionHasFocus(&self) -> bool {
        let dispatcher = &self.editor.dispatcher;
        let frame = dispatcher.FrameSelection();
        // SelectionHasFocus owns this editing lifecycle barrier; the query
        // reads the same resident FrameSelection and original focused element.
        frame.PrepareEditing();
        let result = match frame.CompositionText(CompositionTextQuery::SelectionFocus {
            selection: frame.Current(),
            focused: dispatcher.KeyboardState().borrow().focused_node_id,
        }) {
            Some(CompositionTextResult::SelectionFocus(value)) => value,
            _ => {
                self.Fault("Editor keyboard selection focus geometry");
                false
            }
        };
        result
    }
    fn CanEdit(&self) -> bool {
        self.text_input.CanEdit()
    }
    fn IsTextControl(&self, element: &EventTargetRef) -> bool {
        let dispatcher = &self.editor.dispatcher;
        dispatcher
            .KeyboardTargets()
            .NodeId(element)
            .is_some_and(|id| {
                super::super::page_editing_owners::IsTextControl(
                    &dispatcher.KeyboardTargets().DocumentOwner(),
                    id,
                )
            })
    }
    fn DelegatesFocusTextControlInputFixEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::DelegatesFocusTextControlInputFixEnabled()
    }
    fn DispatchBeforeInputInsertText(
        &self,
        target: Option<&EventTargetRef>,
        text: &[u16],
    ) -> DispatchEventResult {
        if self.editor.error.borrow().is_some() {
            return DispatchEventResult::kCanceledBeforeDispatch;
        }
        match self
            .editor
            .dispatcher
            .KeyboardBeforeInputInsertText(target, text)
        {
            Ok(result) => result,
            Err(EditorCommandError::RequiredPort(error)) => {
                self.Fault(error);
                DispatchEventResult::kCanceledBeforeDispatch
            }
            Err(_) => {
                self.Fault("Editor keyboard beforeinput dispatch");
                DispatchEventResult::kCanceledBeforeDispatch
            }
        }
    }
    fn InsertText(&self, text: &[u16], event: &KeyboardEventRef) -> bool {
        if self.editor.error.borrow().is_some() {
            return false;
        }
        self.text_input.HandleTextInputEvent(
            text,
            Some(EventRef::FromShared(event.clone())),
            interaction_dispatch::core::events::text_event::TextEventInputType::Keyboard,
        )
    }
}
impl<'listener: 'frame, 'frame> PageKeyboardEditor<'listener, 'frame> {
    pub(super) fn HandleSourceKeyboardEvent(&self, event: &KeyboardEventRef) {
        if self.error.borrow().is_some() {
            return;
        }
        let context = Context {
            editor: self,
            text_input: PageEventHandlerTextInput::new(&self.dispatcher),
        };
        editor_key_bindings::HandleKeyboardEvent(&context, event);
        context.Finish();
    }
}
