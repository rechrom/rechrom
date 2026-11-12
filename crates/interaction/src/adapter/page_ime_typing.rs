//! TypingCommand's live DOM/FrameSelection leaves for IME insertion.
//! Source: typing_command.cc:402-537; insert_text_command.cc:149-184,219-287;
//! insert_into_text_node_command.cc:35-55; delete_selection_command.cc.
use super::*;
use interaction_default::core::editing::commands::{
    composite_edit_command::CompositeEditCommand,
    edit_command::PasswordEchoBehavior,
    typing_command::{TypingCommand, TypingCommandHost, TypingSelection},
};
#[path = "page_typing_deletion.rs"]
mod deletion;
#[path = "page_ime_structural.rs"]
mod structural;
pub(super) use deletion::{Delete, DeleteRange};
type NativeEditingStyle =
    interaction_default::core::editing::editing_style::EditingStyle<cssom::CSSInlineStyle>;
type NativeEditorState =
    interaction_default::core::editing::editor::EditorState<ImeTypingSelection, NativeEditingStyle>;

#[derive(Clone, PartialEq)]
pub(crate) struct ImeTypingSelection(pub DOMSelection);
impl TypingSelection for ImeTypingSelection {
    fn IsCaret(&self) -> bool {
        self.0.anchor == self.0.focus
    }
}
#[derive(Default)]
pub(crate) struct ImeTypingState {
    pub text_suggestion: interaction_default::core::editing::text_suggestion_controller::TextSuggestionControllerState,
    pub text_suggestion_host: RefCell<Option<Rc<dyn interaction_default::core::editing::text_suggestion_controller::TextSuggestionHost>>>,
    pub editor: Rc<NativeEditorState>,
    pub typing: TypingCommand<ImeTypingSelection>,
    markers: RefCell<
        Option<(
            u128,
            super::super::page_frame_selection::CompositionMarkerOwner,
        )>,
    >,
}
impl ImeTypingState {
    pub(crate) fn MarkerOwner(
        &self,
        document: u128,
    ) -> super::super::page_frame_selection::CompositionMarkerOwner {
        let mut retained = self.markers.borrow_mut();
        if retained
            .as_ref()
            .is_none_or(|(identity, _)| *identity != document)
        {
            *retained = Some((
                document,
                super::super::page_frame_selection::CompositionMarkerOwner::new(document),
            ));
        }
        retained.as_ref().unwrap().1.clone()
    }
    pub(crate) fn Reset(&self) {
        self.text_suggestion.is_suggestion_menu_open.set(false);
        self.editor.ClearTypingStyle();
        self.typing.CloseTyping();
        self.editor.Clear();
        self.editor.SetMarkSelection(None, false);
        self.markers.borrow_mut().take();
    }
}

pub(super) fn UpdateMarkers(
    commands: &PageCompositionCommands<'_, '_, '_>,
    range: Option<DOMStaticRange>,
) {
    use super::super::page_frame_selection::{CompositionMarkerOwner, CompositionMarkerUpdate};
    let identity = commands
        .document
        .borrow()
        .GetDocument()
        .RootHandle()
        .Identity();
    let owner = {
        let mut retained = commands.selection.editor.ime_typing.markers.borrow_mut();
        if retained
            .as_ref()
            .is_none_or(|(document, _)| *document != identity)
        {
            *retained = Some((identity, CompositionMarkerOwner::new(identity)));
        }
        retained.as_ref().unwrap().1.clone()
    };
    let Some(frame) = commands.selection.frame.as_ref() else {
        return;
    };
    if let Some(ResultValue::Markers {
        ranges,
        invalidated,
    }) = frame.CompositionText(Query::Markers(CompositionMarkerUpdate::new(owner, range)))
    {
        commands
            .selection
            .selections
            .SetDocumentCompositionMarkers(ranges);
        for node in invalidated {
            frame.InvalidateCompositionMarkerPaint(node);
        }
    }
}
struct Host<'a, 'scope, 'listener, 'frame> {
    commands: &'a PageCompositionCommands<'scope, 'listener, 'frame>,
    root: u64,
}
impl Host<'_, '_, '_, '_> {
    fn Frame(&self) -> &PageFrameSelection {
        self.commands
            .selection
            .frame
            .as_deref()
            .expect("prepared IME FrameSelection")
    }
    fn Range(&self, selection: &ImeTypingSelection) -> Option<DOMStaticRange> {
        match self
            .Frame()
            .CompositionText(Query::SelectionRange(selection.0))?
        {
            ResultValue::Range(range) => Some(range),
            _ => None,
        }
    }
    fn Replace(
        &self,
        text: &[u16],
        selection: &ImeTypingSelection,
        select: bool,
        command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
    ) -> Option<ImeTypingSelection> {
        structural::Replace(self, text, selection, select, command)
    }
}
impl TypingCommandHost<ImeTypingSelection> for Host<'_, '_, '_, '_> {
    fn LastEditCommandIs(&self, command: &Rc<CompositeEditCommand<ImeTypingSelection>>) -> bool {
        self.commands
            .selection
            .editor
            .ime_typing
            .editor
            .LastEditCommand()
            .as_ref()
            .is_some_and(|last| Rc::ptr_eq(last, command))
    }
    fn IsValid(&self, selection: &ImeTypingSelection) -> bool {
        let owner = self.commands.document.borrow();
        let document = owner.GetDocument();
        [selection.0.anchor, selection.0.focus]
            .iter()
            .all(|position| {
                position
                    .node_id
                    .and_then(|id| document.FindNodeById(id))
                    .is_some_and(|index| document.IsConnectedToDocument(index))
            })
    }
    fn CompositeBirthSelection(&self) -> (ImeTypingSelection, ImeTypingSelection) {
        let selection = ImeTypingSelection(self.Frame().Current().expect("live IME selection"));
        (selection.clone(), selection)
    }
    fn EditingUseDomPositionApiEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::EditingUseDomPositionApiEnabled()
    }
    fn BeforeTextInserted(&self, text: &[u16], selection: &ImeTypingSelection) -> Option<Vec<u16>> {
        // TypingCommand::InsertText dispatches BeforeTextInsertedEvent at the
        // root editable element. Its native default handler filters input and
        // textarea text before the command mutates the inner editor; this is
        // also the path used by execCommand("insertHTML") on an input.
        if !self.IsValid(selection) {
            return None;
        }
        if !super::super::page_editing_owners::IsTextControl(&self.commands.document, self.root) {
            return Some(text.to_vec());
        }
        let (value, max, textarea) = ReadNode(
            &self.commands.document,
            self.root,
            |document, node, index| {
                (
                    document.TextControlInnerEditorValueUTF16(index),
                    document.TextControlMaxLength(index),
                    node.IsHTMLElement("textarea"),
                )
            },
        )?;
        let focused =
            self.commands.selection.interaction.borrow().focused_node_id == Some(self.root);
        let current = self.commands.selection.Current();
        let selected = current
            .and_then(|selection| value.get(selection.Start()..selection.End()))
            .unwrap_or_default();
        let mut text = text.to_vec();
        if textarea {
            interaction_default::core::html::forms::html_text_area_element::FilterBeforeTextInserted(
                &mut text,
                &value,
                max,
                focused,
                focused.then_some(selected),
            );
        } else {
            interaction_default::core::html::forms::text_field_input_type::FilterBeforeTextInserted(
                &mut text,
                value.len(),
                if focused { selected.len() } else { 0 },
                max,
            );
        }
        Some(text)
    }
    fn DispatchTextInput(&self, text: &[u16]) -> bool {
        // typing_command.cc:100-113 dispatches IncrementalInsertion, whose
        // default remains with this TypingCommand instead of entering Editor.
        let focused = self.commands.selection.interaction.borrow().focused_node_id;
        focused
            .and_then(|id| self.commands.dispatcher.KeyboardTargets().ResolveTarget(id))
            .is_some_and(|target| {
                self.commands
                    .dispatcher
                    .TypingDispatchIncrementalTextInput(&target, text)
                    == NativeResult::kNotCanceled
            })
    }
    fn PrepareForInsertion(&self) {
        self.Frame().PrepareEditing();
    }
    fn InsertTextRun(
        &self,
        text: &[u16],
        selection: &ImeTypingSelection,
        select: bool,
        _incremental: bool,
        command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
        _echo: PasswordEchoBehavior,
        _rebalance_all: bool,
    ) -> Option<ImeTypingSelection> {
        self.Replace(text, selection, select, command)
    }
    fn InsertParagraphSeparator(
        &self,
        selection: &ImeTypingSelection,
        command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
    ) -> Option<ImeTypingSelection> {
        structural::Paragraph(self, selection, command, false)
    }
    fn InsertLineBreak(
        &self,
        selection: &ImeTypingSelection,
        command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
    ) -> Option<ImeTypingSelection> {
        structural::Paragraph(self, selection, command, true)
    }
    fn InsertParagraphSeparatorInQuotedContent(
        &self,
        selection: &ImeTypingSelection,
        command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
    ) -> Option<ImeTypingSelection> {
        structural::QuotedParagraph(self, selection, command)
    }
    fn SelectionStartsInTableStructure(
        &self,
        selection: &ImeTypingSelection,
        command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
    ) -> bool {
        structural::SelectionStartsInTableStructure(self, selection, command)
    }
    fn SelectionStart(&self, selection: &ImeTypingSelection) -> Option<usize> {
        if super::super::page_editing_owners::IsTextControl(
            &self.commands.document,
            self.root,
        ) {
            return self.commands.selection.Current().map(|selection| selection.Start());
        }
        match self.Frame().CompositionText(Query::Offsets {
            root: self.root,
            range: self.Range(selection)?,
        })? {
            ResultValue::Offsets(range) => Some(range.start as usize),
            _ => None,
        }
    }
    fn SelectInsertedRange(&self, start: usize, end: &ImeTypingSelection) -> ImeTypingSelection {
        let Some(range) = self.Range(end) else {
            return end.clone();
        };
        let Some(ResultValue::Offsets(offsets)) = self.Frame().CompositionText(Query::Offsets {
            root: self.root,
            range,
        }) else {
            return end.clone();
        };
        let Some(ResultValue::Range(range)) = self.Frame().CompositionText(Query::Range {
            root: self.root,
            offsets: PlainTextRange {
                start: start as i32,
                end: offsets.end,
            },
        }) else {
            return end.clone();
        };
        ImeTypingSelection(DOMSelection {
            anchor: EditingPosition::new(range.start.node_id, range.start.offset),
            focus: EditingPosition::new(range.end.node_id, range.end.offset),
            affinity: EditingTextAffinity::Downstream,
        })
    }
    fn DeleteSelection(
        &self,
        selection: &ImeTypingSelection,
        command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
        smart: bool,
    ) -> Option<ImeTypingSelection> {
        structural::Delete(self, selection, command, smart)
    }
    fn AppliedEditing(
        &self,
        selection: &ImeTypingSelection,
        command: &Rc<CompositeEditCommand<ImeTypingSelection>>,
        input_type: InputType,
        text: Option<&[u16]>,
        composing: bool,
        _preserves_typing_style: bool,
    ) {
        // CompositeEditCommand::AppliedEditing dispatches editable-content
        // changed before the public input event. A text control serializes its
        // already-mutated resident inner editor without replacing its nodes.
        if super::super::page_editing_owners::IsTextControl(&self.commands.document, self.root) {
            (self.commands.emit)(&DOMMutation {
                mutation_type: DOMMutationType::kSyncTextControlValueFromInnerEditor,
                target_node_id: self.root,
                ..Default::default()
            });
        }
        self.Frame().SetSelection(
            Some(selection.0),
            SetSelectionOptions {
                do_not_set_focus: true,
                ..Default::default()
            },
        );
        struct History<'a, 'b, 'scope, 'listener, 'frame>(&'a Host<'b, 'scope, 'listener, 'frame>);
        impl
            interaction_default::core::editing::commands::undo_stack::UndoStackRegistrationContext<
                ImeTypingSelection,
            > for History<'_, '_, '_, '_, '_>
        {
            type Node = u64;
            fn RootEditableElement(&self, selection: &ImeTypingSelection) -> Option<u64> {
                let owner = self.0.commands.document.borrow();
                let document = owner.GetDocument();
                foundation::editable::RootEditableElement(
                    &dom::persistent_document::DOMEditableFacts(document),
                    selection.0.anchor.node_id?,
                    Default::default(),
                )
            }
            fn SetHasUndoStack(&self, node: &u64, value: bool) {
                let owner = self.0.commands.document.borrow();
                let document = owner.GetDocument();
                if let Some(index) = document.FindNodeById(*node) {
                    document.Node(index).SetHasUndoStack(value);
                }
            }
        }
        self.commands
            .selection
            .editor
            .ime_typing
            .editor
            .RegisterAppliedEditCommand(
                &History(self),
                command,
                input_type,
                foundation::RuntimeEnabledFeatures::EditingUseDomPositionApiEnabled(),
            );
        self.commands
            .dispatcher
            .CompositionDispatchInputWithTransfer(
                self.commands
                    .selection
                    .interaction
                    .borrow()
                    .focused_node_id
                    .unwrap_or(self.root),
                text.unwrap_or_default(),
                input_type,
                composing,
                command.DataTransfer().cloned(),
            );
    }
}
pub(super) fn Insert(
    commands: &PageCompositionCommands<'_, '_, '_>,
    text: &[u16],
    select: bool,
    composition: TextCompositionType,
) {
    InsertWithInputType(commands, text, select, composition, InputType::kNone);
}

/// editor_command.cc's insertion commands enter the same retained typing
/// owner as IME; paragraph structure is produced by the translated leaves.
pub(super) fn InsertNewline(
    commands: &PageCompositionCommands<'_, '_, '_>,
    line_break: bool,
) -> bool {
    let Some(root) = commands.selection.Root() else {
        return false;
    };
    let Some(selection) = commands.selection.frame.as_ref().and_then(|f| f.Current()) else {
        return false;
    };
    let host = Host { commands, root };
    let typing = &commands.selection.editor.ime_typing.typing;
    let inserted = if line_break {
        typing.InsertLineBreak(&host, &ImeTypingSelection(selection))
    } else {
        typing.InsertParagraphSeparator(&host, &ImeTypingSelection(selection))
    };
    if inserted
        && !commands
            .selection
            .editor
            .ime_typing
            .editor
            .PreventRevealSelection()
    {
        use interaction_default::core::editing::frame_selection::{
            RevealExtentOption, SelectionAlignment,
        };
        host.Frame().RevealSelection(
            SelectionAlignment::EdgeIfNeeded,
            RevealExtentOption::kDoNotRevealExtent,
        );
    }
    inserted
}
pub(super) fn InsertQuotedNewline(commands: &PageCompositionCommands<'_, '_, '_>) -> bool {
    let Some(root) = commands.selection.Root() else {
        return false;
    };
    let Some(selection) = commands
        .selection
        .frame
        .as_ref()
        .and_then(|frame| frame.Current())
    else {
        return false;
    };
    let host = Host { commands, root };
    commands
        .selection
        .editor
        .ime_typing
        .typing
        .InsertParagraphSeparatorInQuotedContent(&host, &ImeTypingSelection(selection))
}

pub(super) fn ExecuteStyle(
    commands: &PageCompositionCommands<'_, '_, '_>,
    command: interaction_default::core::editing::commands::editor_command_table::ExecuteFunction,
    value: &str,
) -> Result<bool, &'static str> {
    let Some(root) = commands.selection.Root() else {
        return Ok(false);
    };
    let Some(selection) = commands
        .selection
        .frame
        .as_ref()
        .and_then(|frame| frame.Current())
    else {
        return Ok(false);
    };
    let selection = ImeTypingSelection(selection);
    let composite = CompositeEditCommand::new(selection.clone(), selection);
    let host = Host { commands, root };
    let outcome = structural::ExecuteStyle(&host, command, value, &composite)
        .map_err(|_| "StyleCommands declaration or application")?;
    if outcome.applied {
        let ending = composite.EndingSelection();
        host.AppliedEditing(&ending, &composite, outcome.input_type, None, false, false);
        commands
            .selection
            .editor
            .ime_typing
            .editor
            .ClearTypingStyle();
    }
    Ok(outcome.accepted)
}

pub(super) fn LinkFormat(
    commands: &PageCompositionCommands<'_, '_, '_>,
    action: interaction_default::core::editing::commands::editor_command_table::ExecuteFunction,
    value: &str,
    input_type: InputType,
) -> Result<bool, &'static str> {
    let Some(root) = commands.selection.Root() else {
        return Ok(false);
    };
    let Some(frame) = commands.selection.frame.as_ref() else {
        return Ok(false);
    };
    let Some(selection) = frame.Current() else {
        return Ok(false);
    };
    let selection = ImeTypingSelection(selection);
    let command = CompositeEditCommand::new(selection.clone(), selection);
    command.EnsureUndoStep();
    command.SetSelectionIsDirectional(frame.SelectionIsDirectional());
    let host = Host { commands, root };
    commands.selection.editor.ime_typing.typing.CloseTyping();
    let accepted = structural::ApplyLinkFormat(&host, action, value, &command)
        .map_err(|_| "link/format command aborted")?;
    if accepted {
        let ending = command.EndingSelection();
        let text = value.encode_utf16().collect::<Vec<_>>();
        host.AppliedEditing(
            &ending,
            &command,
            input_type,
            (input_type == InputType::kInsertLink).then_some(text.as_slice()),
            false,
            false,
        );
        if action != interaction_default::core::editing::commands::editor_command_table::ExecuteFunction::ExecuteFormatBlock {
            commands.selection.editor.ime_typing.editor.ClearTypingStyle();
        }
    }
    Ok(accepted)
}

struct History<'a, 'scope, 'listener, 'frame>(
    &'a PageCompositionCommands<'scope, 'listener, 'frame>,
);
impl interaction_default::core::editing::commands::undo_step::UndoStepContext<ImeTypingSelection>
    for History<'_, '_, '_, '_>
{
    type Node = u64;
    fn UpdateStyleAndLayout(&self) {
        if let Some(frame) = &self.0.selection.frame {
            frame.PrepareEditing();
        }
    }
    fn EventQueueScope(
        &self,
    ) -> interaction_dispatch::core::dom::events::scoped_event_queue::EventQueueScope<'_> {
        self.0.dispatcher.EditorCommandEventQueueScope()
    }
    fn RootEditableElement(&self, selection: &ImeTypingSelection) -> Option<u64> {
        let owner = self.0.document.borrow();
        foundation::editable::RootEditableElement(
            &dom::persistent_document::DOMEditableFacts(owner.GetDocument()),
            selection.0.anchor.node_id?,
            Default::default(),
        )
    }
    fn DispatchEditableContentChangedEvents(&self, starting: Option<&u64>, ending: Option<&u64>) {
        for root in [
            starting.copied(),
            ending.copied().filter(|r| Some(*r) != starting.copied()),
        ]
        .into_iter()
        .flatten()
        {
            let control = {
                let owner = self.0.document.borrow();
                super::super::page_focus_controller::EnclosingTextControlForSelectionForFocus(
                    owner.GetDocument(),
                    Some(DOMSelection {
                        anchor: EditingPosition::new(root, 0),
                        focus: EditingPosition::new(root, 0),
                        affinity: EditingTextAffinity::Downstream,
                    }),
                )
            };
            if let Some(control) = control {
                (self.0.emit)(&DOMMutation {
                    mutation_type: DOMMutationType::kSyncTextControlValueFromInnerEditor,
                    target_node_id: control,
                    ..Default::default()
                });
            }
        }
    }
    fn DispatchHistoryInput(
        &self,
        starting: Option<&u64>,
        ending: Option<&u64>,
        input_type: InputType,
    ) {
        for root in [
            starting.copied(),
            ending.copied().filter(|r| Some(*r) != starting.copied()),
        ]
        .into_iter()
        .flatten()
        {
            self.0
                .dispatcher
                .CompositionDispatchInput(root, &[], input_type, false);
        }
    }
    fn CorrectedSelectionAfterCommand(&self, selection: &ImeTypingSelection) -> ImeTypingSelection {
        // editing_commands_utilities.cc:781-793. An invalid original endpoint
        // produces a null selection; it must never be published as a reused ID.
        let empty = || {
            ImeTypingSelection(DOMSelection {
                anchor: EditingPosition::default(),
                focus: EditingPosition::default(),
                affinity: EditingTextAffinity::Downstream,
            })
        };
        let valid = {
            let owner = self.0.document.borrow();
            let document = owner.GetDocument();
            [selection.0.anchor, selection.0.focus]
                .iter()
                .all(|position| {
                    position.node_id.is_none_or(|node| {
                        document.FindNodeById(node).is_some_and(|node| {
                            document.IsConnectedToDocument(node)
                                && (position.anchor
                                    != layoutng_assembly::EditingPositionAnchorType::OffsetInAnchor
                                    || position.offset <= document.LastOffsetForEditing(node))
                        })
                    })
                })
        };
        if !valid {
            return empty();
        }
        if foundation::RuntimeEnabledFeatures::RemoveVisibleSelectionInDOMSelectionEnabled() {
            let Some(frame) = self.0.selection.frame.as_ref() else {
                return empty();
            };
            frame.PrepareEditing();
            frame
                .CreateVisibleSelection(Some(selection.0))
                .map(ImeTypingSelection)
                .unwrap_or_else(empty)
        } else {
            selection.clone()
        }
    }
    fn ChangeSelectionAfterCommand(
        &self,
        selection: &ImeTypingSelection,
        options: SetSelectionOptions,
    ) {
        if let Some(frame) = &self.0.selection.frame {
            frame.ChangeSelectionAfterCommand(selection.0, options);
        }
    }
    fn ClearLastEditCommand(&self) {
        self.0
            .selection
            .editor
            .ime_typing
            .editor
            .SetLastEditCommand(None);
    }
    fn RegisterRedoStep(
        &self,
        step: Rc<
            interaction_default::core::editing::commands::undo_step::UndoStep<ImeTypingSelection>,
        >,
    ) {
        self.0
            .selection
            .editor
            .ime_typing
            .editor
            .GetUndoStack()
            .RegisterRedoStep(step);
    }
    fn RegisterUndoStep(
        &self,
        step: Rc<
            interaction_default::core::editing::commands::undo_step::UndoStep<ImeTypingSelection>,
        >,
    ) {
        self.0
            .selection
            .editor
            .ime_typing
            .editor
            .GetUndoStack()
            .RegisterUndoStep(self, step);
    }
    fn RespondToChangedContentsForFrameSelection(&self) {
        if let Some(frame) = &self.0.selection.frame {
            frame.RespondToChangedContents();
        }
    }
}
impl
    interaction_default::core::editing::commands::undo_stack::UndoStackRegistrationContext<
        ImeTypingSelection,
    > for History<'_, '_, '_, '_>
{
    type Node = u64;
    fn RootEditableElement(&self, selection: &ImeTypingSelection) -> Option<u64> {
        interaction_default::core::editing::commands::undo_step::UndoStepContext::RootEditableElement(self, selection)
    }
    fn SetHasUndoStack(&self, node: &u64, value: bool) {
        let owner = self.0.document.borrow();
        if let Some(index) = owner.GetDocument().FindNodeById(*node) {
            owner.GetDocument().Node(index).SetHasUndoStack(value);
        }
    }
}
pub(super) fn ReplayHistory(commands: &PageCompositionCommands<'_, '_, '_>, redo: bool) {
    commands.selection.editor.ime_typing.typing.CloseTyping();
    let stack = commands.selection.editor.ime_typing.editor.GetUndoStack();
    if redo {
        stack.Redo(&History(commands));
    } else {
        stack.Undo(&History(commands));
    }
}
pub(super) fn InsertWithInputType(
    commands: &PageCompositionCommands<'_, '_, '_>,
    text: &[u16],
    select: bool,
    composition: TextCompositionType,
    input_type: InputType,
) {
    let Some(root) = commands.selection.Root() else {
        return;
    };
    // FrameSelection::GetSelectionInDOMTree projects an active text control's
    // cached UTF-16 offsets through its resident inner editor. Reading only
    // SelectionState::DOMSelection skipped every focused input/textarea,
    // because activating a control intentionally clears that document range.
    let Some(selection) = commands
        .selection
        .frame
        .as_ref()
        .and_then(|frame| frame.Current())
    else {
        return;
    };
    InsertAtSelection(
        commands,
        text,
        &ImeTypingSelection(selection),
        root,
        select,
        composition,
        input_type,
        PasswordEchoBehavior::kDoNotEcho,
    );
}
/// Editor and IME share the original TypingCommand owner. The Editor supplies
/// SelectionForCommand's exact endpoints and password echo policy unchanged.
pub(super) fn InsertAtSelection(
    commands: &PageCompositionCommands<'_, '_, '_>,
    text: &[u16],
    selection: &ImeTypingSelection,
    root: u64,
    select: bool,
    composition: TextCompositionType,
    input_type: InputType,
    echo: PasswordEchoBehavior,
) -> bool {
    let host = Host { commands, root };
    let typing = &commands.selection.editor.ime_typing.typing;
    let inserted = typing.InsertText(
        &host,
        text,
        selection,
        select,
        composition,
        false,
        input_type,
        echo,
    );
    if inserted {
        if let Some(ending) = typing.EndingSelection() {
            // TypingCommand's selected inserted range is its final ending
            // selection, after the element-by-element insertion publications.
            if commands.selection.selections.DOMSelection() != Some(ending.0) {
                host.Frame().SetSelection(
                    Some(ending.0),
                    SetSelectionOptions {
                        do_not_set_focus: true,
                        ..Default::default()
                    },
                );
            }
            if let Some(range) = host.Range(&ending) {
                *commands.ending.borrow_mut() = Some((range.start, range.end));
            }
        }
    }
    inserted
}
pub(super) fn ReplaceFragment(
    commands: &PageCompositionCommands<'_, '_, '_>,
    fragment: u64,
    smart: bool,
    match_style: bool,
    transfer: Option<Rc<interaction_dispatch::core::clipboard::data_transfer::DataTransfer>>,
) -> Result<(), &'static str> {
    ReplaceFragmentWithOptions(commands, fragment,
        interaction_default::core::editing::commands::replace_selection_command::ReplaceSelectionOptions {
            smart_replace: smart, match_style, prevent_nesting: true, sanitize_fragment: true, ..Default::default()
        }, InputType::kInsertFromPaste, transfer)
}
pub(super) fn ReplaceFragmentWithOptions(
    commands: &PageCompositionCommands<'_, '_, '_>,
    fragment: u64,
    options: interaction_default::core::editing::commands::replace_selection_command::ReplaceSelectionOptions,
    input_type: InputType,
    transfer: Option<Rc<interaction_dispatch::core::clipboard::data_transfer::DataTransfer>>,
) -> Result<(), &'static str> {
    let Some(root) = commands.selection.Root() else {
        return Ok(());
    };
    let Some(selection) = commands.selection.frame.as_ref().and_then(|f| f.Current()) else {
        return Ok(());
    };
    let selection = ImeTypingSelection(selection);
    let command =
        CompositeEditCommand::new_with_data_transfer(selection.clone(), selection, transfer);
    let host = Host { commands, root };
    commands.selection.editor.ime_typing.typing.CloseTyping();
    let data = structural::ApplyReplacement(&host, fragment, options, &command)
        .map_err(|_| "ReplaceSelectionCommand aborted")?;
    let ending = command.EndingSelection();
    commands
        .selection
        .editor
        .ime_typing
        .editor
        .ClearTypingStyle();
    host.AppliedEditing(&ending, &command, input_type, Some(&data), false, false);
    Ok(())
}
pub(super) fn DeleteDraggedSelection(
    commands: &PageCompositionCommands<'_, '_, '_>,
    smart: bool,
) -> Result<(), &'static str> {
    DeleteSelectionWithInputType(commands, smart, InputType::kDeleteByDrag)
}
pub(super) fn DeleteSelectionWithInputType(
    commands: &PageCompositionCommands<'_, '_, '_>,
    smart: bool,
    input_type: InputType,
) -> Result<(), &'static str> {
    let Some(root) = commands.selection.Root() else {
        return Ok(());
    };
    let Some(selection) = commands
        .selection
        .frame
        .as_ref()
        .and_then(|frame| frame.Current())
    else {
        return Ok(());
    };
    let selection = ImeTypingSelection(selection);
    let command = CompositeEditCommand::new(selection.clone(), selection.clone());
    let host = Host { commands, root };
    commands.selection.editor.ime_typing.typing.CloseTyping();
    let ending = structural::Delete(&host, &selection, &command, smart)
        .ok_or("DeleteSelectionCommand deletion")?;
    host.AppliedEditing(&ending, &command, input_type, None, false, false);
    Ok(())
}
pub(super) fn ListIndent(
    commands: &PageCompositionCommands<'_, '_, '_>,
    action: interaction_default::core::editing::commands::editor_command_table::EditingCommandType,
    input_type: InputType,
) -> Result<
    bool,
    interaction_default::core::editing::commands::paragraph_editing_context::EditingAborted,
> {
    use interaction_default::core::editing::commands::paragraph_editing_context::EditingAborted;
    let Some(root) = commands.selection.Root() else {
        return Ok(false);
    };
    let Some(frame) = commands.selection.frame.as_ref() else {
        return Ok(false);
    };
    let Some(selection) = frame.Current() else {
        return Ok(false);
    };
    let Some(visible) = frame.CreateVisibleSelection(Some(selection)) else {
        return Ok(false);
    };
    let command =
        CompositeEditCommand::new(ImeTypingSelection(visible), ImeTypingSelection(selection));
    command.EnsureUndoStep();
    command.SetSelectionIsDirectional(frame.SelectionIsDirectional());
    let host = Host { commands, root };
    commands.selection.editor.ime_typing.typing.CloseTyping();
    structural::ApplyListIndent(&host, action, &command)?;
    let ending = command.EndingSelection();
    if !host.IsValid(&ending) {
        return Err(EditingAborted);
    }
    host.AppliedEditing(&ending, &command, input_type, None, false, true);
    Ok(true)
}
pub(super) fn UpdateSelection(commands: &PageCompositionCommands<'_, '_, '_>) {
    let (Some(root), Some(selection)) = (
        commands.selection.Root(),
        commands.selection.selections.DOMSelection(),
    ) else {
        return;
    };
    commands
        .selection
        .editor
        .ime_typing
        .typing
        .UpdateSelectionIfDifferentFromCurrentSelection(
            &Host { commands, root },
            &ImeTypingSelection(selection),
            false,
        );
}
