//! EditorCommand execution into the live list/indent CompositeEditCommand.
use super::{EditorCommandError, EventDispatcher};
use interaction_default::core::editing::commands::{
    editor_command_table::{EditingCommandType as C, EditorCommandSource},
    indent_outdent_command::{IndentOutdentCommand, IndentType},
    insert_list_command::{InsertListCommand, ListType},
};
impl EventDispatcher<'_, '_> {
    pub(super) fn ExecuteListIndentCommand(
        &self,
        command: C,
        source: EditorCommandSource,
    ) -> Option<Result<bool, EditorCommandError>> {
        let input_type = match command {
            C::InsertOrderedList => InsertListCommand::new(ListType::Ordered).GetInputType(),
            C::InsertUnorderedList => InsertListCommand::new(ListType::Unordered).GetInputType(),
            C::Indent => IndentOutdentCommand::new(IndentType::Indent).GetInputType(),
            C::Outdent => IndentOutdentCommand::new(IndentType::Outdent).GetInputType(),
            _ => return None,
        };
        Some((|| {
            if (source == EditorCommandSource::kMenuOrKeyBinding
                && !super::super::page_input_method_controller::CanExecuteEditingCommand(self))
                || !super::super::page_input_method_controller::PasteCanEditRichly(self)
            {
                return Ok(false);
            }
            let document = self.KeyboardTargets().DocumentOwner();
            let identity = document.borrow().GetDocument().RootHandle().Identity();
            if !self.EditingBeforeInputFromSource(command, input_type, source)? {
                return Ok(true);
            }
            if !self.FrameSelection().IsAvailable()
                || document.borrow().GetDocument().RootHandle().Identity() != identity
            {
                return Ok(false);
            }
            self.FrameSelection().PrepareEditing();
            super::super::page_input_method_controller::ExecuteListIndentEditorCommand(
                self, command, input_type,
            )
            .map_err(EditorCommandError::Aborted)
        })())
    }
}
