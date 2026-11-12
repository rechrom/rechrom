//! EventHandler/Editor text input over the canonical Page dispatch and editing
//! owners. Source: event_handler.cc:2504-2535; node.cc:3406-3437;
//! editor.cc:114-201,491-559. This module is a child of the IME adapter so text
//! defaults and composition share the original TypingCommand and EditorState.
use super::*;
use crate::core::input::event_handler::{self, EventHandlerTextInput};
use interaction_default::core::editing::{
    commands::{
        edit_command::PasswordEchoBehavior, replace_selection_command::ReplaceSelectionOptions,
    },
    editor::{
        Editor, EditorCommands, EditorSelection, PreparedEditorGeometry,
        ReplaceSelectionCommandOptions, TextEventData,
    },
    frame_selection::{RevealExtentOption, SelectionAlignment},
};
use interaction_dispatch::core::{
    clipboard::data_transfer::DataTransfer,
    dom::events::event::{EventInterface, EventRef},
    events::text_event::{TextEvent, TextEventInputType},
};
use std::cell::Cell;

#[derive(Debug)]
pub enum TextInputError {
    DetachedTarget,
    NotTextEvent,
    Editor(&'static str),
}
impl std::fmt::Display for TextInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for TextInputError {}

/// This transaction owns only cloned Rc handles. No DOM or Layout borrow spans
/// dispatch, and a listener can synchronously enter the same retained owners.
#[derive(Clone)]
pub struct PageEventHandlerTextInput<'listener, 'frame> {
    dispatcher: EventDispatcher<'listener, 'frame>,
    failed: Rc<Cell<bool>>,
    error: Rc<RefCell<Option<TextInputError>>>,
    dispatched: Rc<Cell<DispatchEventResult>>,
}
impl<'listener: 'frame, 'frame> PageEventHandlerTextInput<'listener, 'frame> {
    pub fn new(dispatcher: &EventDispatcher<'listener, 'frame>) -> Rc<Self> {
        Rc::new(Self {
            dispatcher: dispatcher.clone(),
            failed: Rc::new(Cell::new(false)),
            error: Rc::new(RefCell::new(None)),
            dispatched: Rc::new(Cell::new(DispatchEventResult::default())),
        })
    }
    fn Fault(&self, error: TextInputError) {
        if !self.failed.replace(true) {
            *self.error.borrow_mut() = Some(error);
        }
    }
    pub fn TakeError(&self) -> Option<TextInputError> {
        self.error.borrow_mut().take()
    }
    pub fn Finish(&self) -> Result<(), TextInputError> {
        match self.TakeError() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
    pub fn TakeDispatchResult(&self) -> DispatchEventResult {
        self.dispatched.take()
    }
    pub fn HandleTextInputEvent(
        &self,
        text: &[u16],
        underlying: Option<EventRef>,
        kind: TextEventInputType,
    ) -> bool {
        event_handler::HandleTextInputEvent(text, underlying, kind, self)
    }
    /// Source EditorCommand execution can dispatch a text event through its
    /// existing caller; bind the same native default owner for that scope.
    pub(crate) fn WithDefaultHandling<T>(&self, run: impl FnOnce() -> T) -> T {
        let context = self.clone();
        let previous = self.dispatcher.TextInputDefaultHandler();
        self.dispatcher
            .SetTextInputDefaultHandler(Some(Rc::new(move |event| {
                context.DefaultTextInputEventHandler(event)
            })));
        let _binding = DefaultBinding {
            dispatcher: &self.dispatcher,
            previous,
        };
        run()
    }
    pub(crate) fn CanEdit(&self) -> bool {
        let commands = Rc::new(TextCommands {
            context: self.clone(),
            commands: EditorSelectionCommands(&self.dispatcher),
        });
        let selection = Rc::new(TextSelection {
            dispatcher: self.dispatcher.clone(),
        });
        Editor::WithState(
            selection,
            commands,
            self.dispatcher
                .EditingOwners()
                .ime_typing
                .editor
                .clone(),
        )
        .CanEdit()
    }
    fn DefaultTextInputEventHandler(&self, event: &EventRef) {
        event_handler::DefaultTextInputEventHandler(event, self)
    }
}
struct DefaultBinding<'scope, 'listener, 'frame> {
    dispatcher: &'scope EventDispatcher<'listener, 'frame>,
    previous: Option<Rc<dyn Fn(&EventRef) + 'frame>>,
}
impl Drop for DefaultBinding<'_, '_, '_> {
    fn drop(&mut self) {
        self.dispatcher
            .SetTextInputDefaultHandler(self.previous.take());
    }
}
impl<'listener: 'frame, 'frame> EventHandlerTextInput
    for PageEventHandlerTextInput<'listener, 'frame>
{
    type InputType = TextEventInputType;
    fn EventTargetNodeForDocument(&self) -> Option<EventTargetRef> {
        self.dispatcher
            .KeyboardTargets()
            .EventTargetNodeForDocument(self.dispatcher.KeyboardState().borrow().focused_node_id)
    }
    fn CreateTextEvent(&self, text: &[u16], kind: TextEventInputType) -> EventRef {
        let mut event = TextEvent::new(
            Some(self.dispatcher.KeyboardTargets().WindowView()),
            text,
            kind,
            self.dispatcher.EventClock(),
        );
        event.BaseEventMut().SetTrusted(true);
        EventRef::new(event)
    }
    fn DispatchEvent(&self, target: &EventTargetRef, event: &EventRef) {
        if self.failed.get() {
            return;
        }
        if self.dispatcher.KeyboardTargets().NodeId(target).is_none() {
            self.Fault(TextInputError::DetachedTarget);
            return;
        }
        self.WithDefaultHandling(|| {
            let (_, result) = self
                .dispatcher
                .DispatchCanonicalKeyboardEvent(target, event.clone());
            self.dispatched.set(result);
        });
    }
    fn EditorHandleTextEvent(&self, event: &EventRef) -> bool {
        if self.failed.get() {
            return false;
        }
        let data = {
            let native = event.borrow();
            let Some(text) = native.AsTextEvent() else {
                self.Fault(TextInputError::NotTextEvent);
                return false;
            };
            // editor.cc:183-196 reads this Android-only WebKeyboardEvent fact
            // under the same platform guard as the original source field.
            #[cfg(target_os = "android")]
            let physical = native
                .BaseEvent()
                .UnderlyingEvent()
                .is_some_and(|underlying| {
                    underlying
                        .borrow()
                        .AsKeyboardEvent()
                        .and_then(|key| key.KeyEvent())
                        .is_some_and(|key| key.is_confirmed_physical_keyboard_input)
                });
            TextEventData {
                event: event.clone(),
                data: text.data().to_vec(),
                is_drop: text.IsDrop(),
                is_incremental_insertion: text.IsIncrementalInsertion(),
                is_paste: text.IsPaste(),
                is_line_break: text.IsLineBreak(),
                is_composition: text.IsComposition(),
                smart_replace: text.ShouldSmartReplace(),
                match_style: text.ShouldMatchStyle(),
                pasting_fragment: text
                    .PastingFragment::<super::super::page_selection_markup::RetainedFragment>(),
                data_transfer: text.DataTransfer(),
                #[cfg(target_os = "android")]
                confirmed_physical_keyboard: physical,
            }
        };
        // editor.cc:145-158: drop/incremental events leave their defaults with
        // the initiating owner, without forcing an editing lifecycle update.
        if !data.is_drop && !data.is_incremental_insertion {
            self.dispatcher.FrameSelection().PrepareEditing();
        }
        let commands = Rc::new(TextCommands {
            context: self.clone(),
            commands: EditorSelectionCommands(&self.dispatcher),
        });
        let selection = Rc::new(TextSelection {
            dispatcher: self.dispatcher.clone(),
        });
        let retained = self
            .dispatcher
            .EditingOwners()
            .ime_typing
            .editor
            .clone();
        let editor = Editor::WithState(selection, commands, retained);
        let _events = self.dispatcher.EditorCommandEventQueueScope();
        let handled = editor.HandleTextEvent(&data, &PreparedEditorGeometry);
        handled && !self.failed.get()
    }
}

fn EmptySelection() -> ImeTypingSelection {
    ImeTypingSelection(DOMSelection {
        anchor: EditingPosition::default(),
        focus: EditingPosition::default(),
        affinity: EditingTextAffinity::Downstream,
    })
}
struct TextSelection<'listener, 'frame> {
    dispatcher: EventDispatcher<'listener, 'frame>,
}
impl TextSelection<'_, '_> {
    fn Control(&self, selection: &ImeTypingSelection) -> Option<u64> {
        let document = self.dispatcher.KeyboardTargets().DocumentOwner();
        let owner = document.borrow();
        super::super::page_focus_controller::EnclosingTextControlForSelectionForFocus(
            owner.GetDocument(),
            Some(selection.0),
        )
    }
    fn Target(&self, event: &EventRef) -> Option<u64> {
        self.dispatcher
            .KeyboardTargets()
            .NodeId(&event.borrow().BaseEvent().RawTarget()?)
    }
    fn Editable(&self, selection: &ImeTypingSelection, rich: bool) -> bool {
        let document = self.dispatcher.KeyboardTargets().DocumentOwner();
        if let Some(control) = self.Control(selection) {
            return !rich
                && self
                    .dispatcher
                    .EditingOwners()
                    .CaretFor(&document, control)
                    .is_some();
        }
        let owner = document.borrow();
        let facts = dom::persistent_document::DOMEditableFacts(owner.GetDocument());
        selection.0.anchor.node_id.is_some_and(|id| {
            if rich {
                foundation::editable::IsRichlyEditable(&facts, id, Default::default())
            } else {
                foundation::editable::IsEditable(&facts, id, Default::default())
            }
        })
    }
    fn Reveal(&self) {
        if !self
            .dispatcher
            .EditingOwners()
            .ime_typing
            .editor
            .PreventRevealSelection()
        {
            self.dispatcher.FrameSelection().RevealSelection(
                SelectionAlignment::EdgeIfNeeded,
                RevealExtentOption::kDoNotRevealExtent,
            );
        }
    }
}
impl EditorSelection<ImeTypingSelection> for TextSelection<'_, '_> {
    fn IsNone(&self, value: &ImeTypingSelection) -> bool {
        value.0.anchor.IsNull() || value.0.focus.IsNull()
    }
    fn IsRangeForEditing(&self, value: &ImeTypingSelection) -> bool {
        !self.IsNone(value) && value.0.anchor != value.0.focus
    }
    fn IsContentEditable(&self, value: &ImeTypingSelection) -> bool {
        self.Editable(value, false)
    }
    fn IsRichlyEditable(&self, value: &ImeTypingSelection) -> bool {
        self.Editable(value, true)
    }
    fn Current(&self) -> ImeTypingSelection {
        self.dispatcher
            .FrameSelection()
            .Current()
            .map(ImeTypingSelection)
            .unwrap_or_else(EmptySelection)
    }
    fn EventTargetIsTextControl(&self, event: &EventRef) -> bool {
        self.Target(event).is_some_and(|id| {
            super::super::page_editing_owners::IsTextControl(
                &self.dispatcher.KeyboardTargets().DocumentOwner(),
                id,
            )
        })
    }
    fn CurrentSelectionBelongsToEventTarget(
        &self,
        event: &EventRef,
        selection: &ImeTypingSelection,
    ) -> bool {
        self.Target(event) == self.Control(selection)
    }
    fn EventTargetSavedSelection(&self, event: &EventRef) -> ImeTypingSelection {
        let Some(control) = self.Target(event) else {
            return EmptySelection();
        };
        let Some(saved) = self.dispatcher.EditingSelectionState().Get(control) else {
            return EmptySelection();
        };
        let document = self.dispatcher.KeyboardTargets().DocumentOwner();
        let owner = document.borrow();
        let document = owner.GetDocument();
        let Some(index) = document.FindNodeById(control) else {
            return EmptySelection();
        };
        let (Some(anchor), Some(focus)) = (
            document.TextControlInnerEditorPosition(index, saved.anchor),
            document.TextControlInnerEditorPosition(index, saved.focus),
        ) else {
            return EmptySelection();
        };
        ImeTypingSelection(DOMSelection {
            anchor: EditingPosition::new(anchor.node_id, anchor.offset),
            focus: EditingPosition::new(focus.node_id, focus.offset),
            affinity: EditingTextAffinity::Downstream,
        })
    }
    fn CreateVisibleSelection(&self, value: &ImeTypingSelection) -> ImeTypingSelection {
        self.dispatcher
            .FrameSelection()
            .CreateVisibleSelection(Some(value.0))
            .map(ImeTypingSelection)
            .unwrap_or_else(EmptySelection)
    }
    fn HasRootEditableElementForSelection(&self, value: &ImeTypingSelection) -> bool {
        let document = self.dispatcher.KeyboardTargets().DocumentOwner();
        let owner = document.borrow();
        value
            .0
            .anchor
            .node_id
            .and_then(|id| {
                foundation::editable::RootEditableElement(
                    &dom::persistent_document::DOMEditableFacts(owner.GetDocument()),
                    id,
                    Default::default(),
                )
            })
            .is_some()
    }
    fn RevealFocusedOrMainFrameSelection(&self, _: &ImeTypingSelection) {
        self.Reveal();
    }
    fn RevealSelectionAfterEditingOperation(&self) {
        self.Reveal();
    }
}
struct TextCommands<'scope, 'listener, 'frame> {
    context: PageEventHandlerTextInput<'listener, 'frame>,
    commands: PageCompositionCommands<'scope, 'listener, 'frame>,
}
impl<'listener: 'frame, 'frame> EditorCommands<ImeTypingSelection>
    for TextCommands<'_, 'listener, 'frame>
{
    type Fragment = super::super::page_selection_markup::RetainedFragment;
    type DataTransfer = Rc<DataTransfer>;
    type TypingStyle =
        interaction_default::core::editing::editing_style::EditingStyle<cssom::CSSInlineStyle>;
    fn HandleTextInputEvent(&self, text: &[u16], underlying: Option<&EventRef>) -> bool {
        self.context
            .HandleTextInputEvent(text, underlying.cloned(), TextEventInputType::Keyboard)
    }
    fn InsertText(
        &self,
        text: &[u16],
        selection: &ImeTypingSelection,
        select: bool,
        echo: PasswordEchoBehavior,
        composing: bool,
        input_type: InputType,
        transfer: Option<&Rc<DataTransfer>>,
    ) -> bool {
        if transfer.is_some() {
            self.context.Fault(TextInputError::Editor(
                "typing data transfer requires a source TypingCommand transport",
            ));
            return false;
        }
        let source = TextSelection {
            dispatcher: self.context.dispatcher.clone(),
        };
        if let Some(control) = source.Control(selection) {
            let dispatcher = &self.context.dispatcher;
            let dispatch: SyntheticEventDispatcher<'_> = Rc::new(move |event, target| {
                let data = event
                    .data_utf16
                    .as_ref()
                    .and_then(Clone::clone)
                    .unwrap_or_else(|| Rc::from(event.text.encode_utf16().collect::<Vec<_>>()));
                dispatcher.CompositionDispatchInput(
                    target,
                    &data,
                    if composing {
                        InputType::kInsertCompositionText
                    } else {
                        input_type
                    },
                    composing,
                );
            });
            return self
                .commands
                .selection
                .editor
                .InsertImeText(
                    &self.commands.document,
                    control,
                    text,
                    select,
                    composing,
                    &self.commands.emit,
                    &dispatch,
                )
                .is_some();
        }
        let root = {
            let owner = self.commands.document.borrow();
            selection.0.anchor.node_id.and_then(|id| {
                foundation::editable::RootEditableElement(
                    &dom::persistent_document::DOMEditableFacts(owner.GetDocument()),
                    id,
                    Default::default(),
                )
            })
        };
        let Some(root) = root else {
            return false;
        };
        typing::InsertAtSelection(
            &self.commands,
            text,
            selection,
            root,
            select,
            if composing {
                TextCompositionType::kTextCompositionConfirm
            } else {
                TextCompositionType::kTextCompositionNone
            },
            input_type,
            echo,
        )
    }
    fn InsertLineBreak(&self) -> bool {
        typing::InsertNewline(&self.commands, true)
    }
    fn InsertParagraphSeparator(&self) -> bool {
        typing::InsertNewline(&self.commands, false)
    }
    fn CreateFragmentFromText(&self, text: &[u16]) -> Option<Self::Fragment> {
        match self.context.dispatcher.CreatePasteTextFragment(text) {
            Ok(fragment) => Some(fragment),
            Err(error) => {
                self.context.Fault(TextInputError::Editor(error));
                None
            }
        }
    }
    fn ReplaceSelection(
        &self,
        fragment: &Self::Fragment,
        options: ReplaceSelectionCommandOptions,
        _: PasswordEchoBehavior,
        input_type: InputType,
        transfer: Option<&Rc<DataTransfer>>,
    ) {
        if !fragment.IsCurrent() {
            self.context
                .Fault(TextInputError::Editor("paste fragment document changed"));
            return;
        }
        if let Err(error) = typing::ReplaceFragmentWithOptions(
            &self.commands,
            fragment.root,
            ReplaceSelectionOptions {
                prevent_nesting: options.prevent_nesting,
                sanitize_fragment: options.sanitize_fragment,
                select_replacement: options.select_replacement,
                smart_replace: options.smart_replace,
                match_style: options.match_style,
                ..Default::default()
            },
            input_type,
            transfer.cloned(),
        ) {
            self.context.Fault(TextInputError::Editor(error));
        }
    }
}
