//! Menu/key-binding style commands enter the retained Editor and ApplyStyle.
use super::{EditorCommandError, EventDispatcher};
use interaction_default::core::editing::commands::{
    editor_command_input_type::{CommandInputTypeContext, InputTypeFromCommandType},
    editor_command_table::{EditingCommandType, EditorCommandSource},
    style_commands::{IsStyleCommand, StyleWithCssValue},
};
use interaction_dispatch::core::events::input_event::InputType as I;
impl EventDispatcher<'_, '_> {
    pub(super) fn ExecuteStyleCommand(
        &self,
        command_type: EditingCommandType,
        value: &str,
        source: EditorCommandSource,
    ) -> Option<Result<bool, EditorCommandError>> {
        let command = command_type.Descriptor()?.execute;
        if !IsStyleCommand(command) {
            return None;
        }
        let result = (|| {
            if let Some(should_style_with_css) = StyleWithCssValue(command, value) {
                self.EditingOwners()
                    .ime_typing
                    .editor
                    .SetShouldStyleWithCss(should_style_with_css);
                return Ok(true);
            }
            if (source == EditorCommandSource::kMenuOrKeyBinding
                && !super::super::page_input_method_controller::CanExecuteEditingCommand(self))
                || !super::super::page_input_method_controller::PasteCanEditRichly(self)
            {
                return Ok(false);
            }
            struct InputContext<'a, 'b, 'c>(&'a EventDispatcher<'b, 'c>);
            impl CommandInputTypeContext for InputContext<'_, '_, '_> {
                fn CanEditRichly(&self) -> bool {
                    super::super::page_input_method_controller::PasteCanEditRichly(self.0)
                }
                fn VisibleSelectionIsRange(&self) -> bool {
                    self.0
                        .FrameSelection()
                        .CreateVisibleSelection(self.0.FrameSelection().Current())
                        .is_some_and(|selection| selection.anchor != selection.focus)
                }
            }
            let input_type = InputTypeFromCommandType(command_type, &InputContext(self));
            let document = self.KeyboardTargets().DocumentOwner();
            let identity = document.borrow().GetDocument().RootHandle().Identity();
            if input_type != I::kNone {
                if !self.EditingBeforeInputFromSource(command_type, input_type, source)? {
                    return Ok(true);
                }
            }
            if !self.FrameSelection().IsAvailable()
                || document.borrow().GetDocument().RootHandle().Identity() != identity
            {
                return Ok(false);
            }
            if source == EditorCommandSource::kMenuOrKeyBinding
                && self
                    .input_method_state
                    .as_ref()
                    .is_some_and(|state| state.GetActiveEditContext().is_some())
                && matches!(
                    command_type,
                    EditingCommandType::ToggleBold
                        | EditingCommandType::ToggleItalic
                        | EditingCommandType::ToggleUnderline
                )
            {
                return Ok(true);
            }
            self.FrameSelection().PrepareEditing();
            super::super::page_input_method_controller::ExecuteStyleEditorCommand(
                self, command, value,
            )
            .map_err(EditorCommandError::RequiredPort)
        })();
        Some(result)
    }
}
