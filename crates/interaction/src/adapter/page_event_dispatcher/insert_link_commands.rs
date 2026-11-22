//! InsertCommands and link/format commands share the existing native parser,
//! CompositeEditCommand, ReplaceSelection and ApplyStyle owners.
use super::{EditorCommandError, EventDispatcher};
use interaction_default::core::editing::commands::{
    editor_command_input_type::{CommandInputTypeContext, InputTypeFromCommandType},
    editor_command_table::{EditingCommandType, EditorCommandSource, ExecuteFunction as F},
};
use interaction_dispatch::core::events::input_event::InputType as I;
impl EventDispatcher<'_, '_> {
    pub(super) fn ExecuteEditorHostCommand(
        &self,
        action: F,
    ) -> Option<Result<bool, EditorCommandError>> {
        use interaction_scroll::core::input::scroll_manager::{
            ScrollDirection as D, ScrollGranularity as G,
        };
        let (direction, granularity) = match action {
            F::ExecuteScrollPageBackward => (D::kScrollBlockDirectionBackward, G::kScrollByPage),
            F::ExecuteScrollPageForward => (D::kScrollBlockDirectionForward, G::kScrollByPage),
            F::ExecuteScrollLineUp => (D::kScrollUpIgnoringWritingMode, G::kScrollByLine),
            F::ExecuteScrollLineDown => (D::kScrollDownIgnoringWritingMode, G::kScrollByLine),
            F::ExecuteScrollToBeginningOfDocument => {
                (D::kScrollBlockDirectionBackward, G::kScrollByDocument)
            }
            F::ExecuteScrollToEndOfDocument => {
                (D::kScrollBlockDirectionForward, G::kScrollByDocument)
            }
            F::ExecutePrint => {
                return Some(Err(EditorCommandError::RequiredPort("ChromeClient::Print")))
            }
            F::ExecuteIgnoreSpelling => {
                return Some(Err(EditorCommandError::RequiredPort(
                    "SpellChecker::IgnoreSpelling",
                )))
            }
            F::ExecuteFindString => {
                return Some(Err(EditorCommandError::RequiredPort(
                    "Editor::FindString native FindBuffer",
                )))
            }
            F::ClipboardCommands_ExecutePasteFromImageUrl => {
                return Some(Err(EditorCommandError::RequiredPort(
                    "ClipboardCommands image resource observer",
                )))
            }
            _ => return None,
        };
        Some((|| {
            self.FrameSelection().PrepareEditing();
            let scroll = self
                .editor_scroll
                .as_ref()
                .ok_or(EditorCommandError::RequiredPort(
                    "EventHandler retained ScrollManager",
                ))?;
            Ok(scroll.ScrollManager().BubblingScroll(
                direction,
                granularity,
                None,
                self.mouse_state
                    .MousePressNode()
                    .and_then(|node| node.NodeId())
                    .map(|node| scroll.KeyboardScrollNode(node)),
                false,
            ))
        })())
    }
    pub(super) fn ExecuteInsertLinkCommand(
        &self,
        command_type: EditingCommandType,
        value: &str,
        source: EditorCommandSource,
    ) -> Option<Result<bool, EditorCommandError>> {
        let action = command_type.Descriptor()?.execute;
        let insert = matches!(
            action,
            F::InsertCommands_ExecuteInsertHtml
                | F::InsertCommands_ExecuteInsertImage
                | F::InsertCommands_ExecuteInsertHorizontalRule
        );
        if !insert
            && !matches!(
                action,
                F::ExecuteCreateLink
                    | F::ExecuteUnlink
                    | F::ExecuteRemoveFormat
                    | F::ExecuteFormatBlock
            )
        {
            return None;
        }
        Some((|| {
            use super::super::page_input_method_controller as editor;
            if source == EditorCommandSource::kMenuOrKeyBinding
                && !editor::CanExecuteEditingCommand(self)
            {
                return Ok(false);
            }
            if action != F::InsertCommands_ExecuteInsertHtml && !editor::PasteCanEditRichly(self) {
                return Ok(false);
            }
            if action == F::ExecuteCreateLink && value.is_empty() {
                return Ok(false);
            }
            struct Input<'a, 'b, 'c>(&'a EventDispatcher<'b, 'c>);
            impl CommandInputTypeContext for Input<'_, '_, '_> {
                fn CanEditRichly(&self) -> bool {
                    editor::PasteCanEditRichly(self.0)
                }
                fn VisibleSelectionIsRange(&self) -> bool {
                    self.0
                        .FrameSelection()
                        .CreateVisibleSelection(self.0.FrameSelection().Current())
                        .is_some_and(|s| s.anchor != s.focus)
                }
            }
            let input = InputTypeFromCommandType(command_type, &Input(self));
            let owner = self.KeyboardTargets().DocumentOwner();
            let identity = owner.borrow().GetDocument().RootHandle().Identity();
            if input != I::kNone
                && !self.EditingBeforeInputFromSource(command_type, input, source)?
            {
                return Ok(true);
            }
            if !self.FrameSelection().IsAvailable()
                || owner.borrow().GetDocument().RootHandle().Identity() != identity
            {
                return Ok(false);
            }
            self.FrameSelection().PrepareEditing();
            if !insert {
                let applied_input = if action == F::ExecuteRemoveFormat {
                    I::kFormatRemove
                } else {
                    input
                };
                return editor::ExecuteLinkFormatEditorCommand(self, action, value, applied_input)
                    .map_err(EditorCommandError::RequiredPort);
            }
            use super::super::page_selection_markup as markup;
            let fragment = match action {
                F::InsertCommands_ExecuteInsertHtml => {
                    let text = value.encode_utf16().collect::<Vec<_>>();
                    markup::CreateFragmentForInsertHTML(&owner, &text)
                }
                F::InsertCommands_ExecuteInsertImage => {
                    markup::CreateFragmentFromElement(&owner, "img", "src", value)
                }
                F::InsertCommands_ExecuteInsertHorizontalRule => {
                    markup::CreateFragmentFromElement(&owner, "hr", "id", value)
                }
                _ => unreachable!(),
            }
            .map_err(|_| EditorCommandError::RequiredPort("InsertCommands native fragment"))?;
            // insert_commands.cc:116-133: INPUT inserts textContent (including
            // script/style text) with BR converted to LF through InsertText.
            if action == F::InsertCommands_ExecuteInsertHtml {
                let input = self
                    .FrameSelection()
                    .Current()
                    .and_then(|s| s.anchor.node_id)
                    .is_some_and(|node| {
                        let doc = owner.borrow();
                        let doc = doc.GetDocument();
                        let mut node = doc.FindNodeById(node);
                        while let Some(index) = node {
                            if doc.Node(index).IsHTMLElement("input") {
                                return true;
                            }
                            node = doc.ParentOrShadowHostElement(index);
                        }
                        false
                    });
                if input {
                    let text = markup::FragmentTextContentWithBreaks(&fragment);
                    return Ok(editor::ExecuteTypingEditorCommand(
                        self,
                        F::InsertCommands_ExecuteInsertText,
                        &String::from_utf16_lossy(&text),
                    ));
                }
            }
            editor::ApplyCommandFragment(self, &fragment).map_err(EditorCommandError::RequiredPort)
        })())
    }
}
