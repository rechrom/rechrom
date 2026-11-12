//! ClipboardCommands' menu/keybinding ports borrow live selection and dispatch
//! owners. Native clipboard bytes cross the host boundary only as values.
use super::*;
use crate::adapter::page_frame_selection::{CompositionTextQuery as Q, CompositionTextResult as V};
use interaction_default::core::editing::commands::{
    clipboard_commands::{
        self as commands, ClipboardHtml, CopyCutCommands, PasteCommands, PrimaryClipboardSnapshot,
    },
    editor_command_table::{EditorCommandSource as Source, EnabledFunction, ExecuteFunction as F},
};
use interaction_dispatch::core::{
    clipboard::data_transfer::{DataTransfer, DataTransferAccessPolicy},
    events::{
        clipboard_event::ClipboardEvent,
        input_event::{EventIsComposing, InputEvent, InputType},
    },
};

pub(super) fn Execute(
    dispatcher: &EventDispatcher<'_, '_>,
    command: F,
) -> Option<Result<bool, &'static str>> {
    ExecuteFromSource(dispatcher, command, Source::kMenuOrKeyBinding)
}
pub(super) fn CanReadClipboard(dispatcher: &EventDispatcher<'_, '_>, source: Source) -> bool {
    commands::CanReadClipboard(dispatcher.FrameSelection().ClipboardAccessPolicy(), source)
}
pub(super) fn CanWriteClipboard(dispatcher: &EventDispatcher<'_, '_>, source: Source) -> bool {
    let policy = dispatcher.FrameSelection().ClipboardAccessPolicy();
    if commands::CanWriteClipboard(policy, source, false) {
        return true;
    }
    // Source clipboard_commands.cc:135-148. KeyboardEventManager activates the
    // frame before dispatch; listeners can also consume that activation. Read
    // its live state here rather than retaining a pre-dispatch permission bit.
    let frame = dispatcher
        .input_frame
        .as_ref()
        .expect("clipboard activation permission requires its LocalFrame input owner");
    commands::CanWriteClipboard(
        policy,
        source,
        frame
            .activation
            .borrow()
            .IsActive(dispatcher.EventClock().Now()),
    )
}
pub(super) fn IsEnabled(
    dispatcher: &EventDispatcher<'_, '_>,
    function: EnabledFunction,
    source: Source,
) -> Result<bool, &'static str> {
    let owner = ClipboardCommandsOwner::New(dispatcher, source);
    use EnabledFunction::*;
    match function {
        ClipboardCommands_EnabledCopy | ClipboardCommands_EnabledCut => {
            if !CanWriteClipboard(dispatcher, source) {
                return Ok(false);
            }
            let cut = function == ClipboardCommands_EnabledCut;
            if cut && source == Source::kMenuOrKeyBinding && !owner.Focus() {
                return Ok(false);
            }
            Ok(
                !owner.DispatchWritableEvent(if cut { "beforecut" } else { "beforecopy" })?
                    || if cut {
                        CopyCutCommands::CanCut(&owner)
                    } else {
                        CopyCutCommands::CanCopy(&owner)
                    },
            )
        }
        ClipboardCommands_EnabledPaste => Ok(CanReadClipboard(dispatcher, source)
            && (source == Source::kDom || owner.Focus())
            && PasteCommands::CanPaste(&owner)),
        _ => Ok(false),
    }
}
pub(super) fn ExecuteFromSource(
    dispatcher: &EventDispatcher<'_, '_>,
    command: F,
    source: Source,
) -> Option<Result<bool, &'static str>> {
    if !matches!(
        command,
        F::ClipboardCommands_ExecuteCopy
            | F::ClipboardCommands_ExecuteCut
            | F::ClipboardCommands_ExecutePaste
            | F::ClipboardCommands_ExecutePasteAndMatchStyle
            | F::ClipboardCommands_ExecutePasteGlobalSelection
    ) {
        return None;
    }
    let owner = ClipboardCommandsOwner::New(dispatcher, source);
    Some(match command {
        F::ClipboardCommands_ExecuteCopy => commands::ExecuteCopy(&owner),
        F::ClipboardCommands_ExecuteCut => commands::ExecuteCut(&owner),
        F::ClipboardCommands_ExecutePaste => commands::ExecutePaste(&owner),
        F::ClipboardCommands_ExecutePasteAndMatchStyle => {
            commands::ExecutePasteAndMatchStyle(&owner)
        }
        F::ClipboardCommands_ExecutePasteGlobalSelection => {
            commands::ExecutePasteGlobalSelection(&owner)
        }
        _ => unreachable!(),
    })
}
struct ClipboardCommandsOwner<'a, 'listener, 'frame> {
    dispatcher: &'a EventDispatcher<'listener, 'frame>,
    identity: u128,
    transfer: RefCell<Option<Rc<DataTransfer>>>,
    source: Source,
}
impl<'a, 'listener, 'frame> ClipboardCommandsOwner<'a, 'listener, 'frame> {
    fn New(dispatcher: &'a EventDispatcher<'listener, 'frame>, source: Source) -> Self {
        Self {
            dispatcher,
            identity: dispatcher
                .document
                .borrow()
                .GetDocument()
                .RootHandle()
                .Identity(),
            transfer: RefCell::new(None),
            source,
        }
    }
    fn DispatchWritableEvent(&self, kind: &str) -> Result<bool, &'static str> {
        if self.Password() {
            return Ok(true);
        }
        let Some(target) = self.Target() else {
            return Ok(true);
        };
        let transfer = DataTransfer::Create();
        let mut event =
            ClipboardEvent::new(kind, Some(transfer.clone()), self.dispatcher.clock.as_ref());
        event.BaseEventMut().SetTrusted(true);
        let result = self
            .dispatcher
            .DispatchCanonicalEventTarget(&target, EventRef::new(event))
            .0;
        let cancelled = result != interaction_dispatch::core::dom::events::event_dispatch_result::DispatchEventResult::kNotCanceled;
        // Read the writable object before invalidating listener access, including
        // an intentionally empty object when the listener cancelled the event.
        let snapshot = if cancelled {
            let object = transfer.GetDataObject();
            let markup = object.GetData("text/html").Utf8();
            let mut snapshot = PrimaryClipboardSnapshot {
                plain_text: object.GetData("text/plain").Utf8(),
                html: if markup.is_empty() {
                    None
                } else {
                    Some(ClipboardHtml {
                        fragment_start: 0,
                        fragment_end: markup.encode_utf16().count() as u32,
                        markup,
                        source_url: String::new(),
                    })
                },
                smart_replace: false,
                ..Default::default()
            };
            for index in 0..object.length() {
                let Some(item) = object.Item(index) else {
                    continue;
                };
                if item.Kind() != interaction_dispatch::core::clipboard::data_object_item::ItemKind::kStringKind { continue; }
                let value = item.GetAsString();
                match item.GetType() {
                    "text/plain" | "text/html" | "downloadurl" => {}
                    kind => snapshot
                        .custom_data
                        .push((kind.to_owned(), value.Span16().unwrap_or_default().to_vec())),
                }
            }
            Some(snapshot)
        } else {
            None
        };
        transfer.SetAccessPolicyNumb();
        if let Some(snapshot) = snapshot {
            self.dispatcher
                .FrameSelection()
                .WriteClipboard(&snapshot)
                .map_err(|_| "ClipboardHost event write")?;
        }
        Ok(!cancelled)
    }
    fn Current(&self) -> bool {
        self.dispatcher.FrameSelection().IsAvailable()
            && self
                .dispatcher
                .document
                .borrow()
                .GetDocument()
                .RootHandle()
                .Identity()
                == self.identity
    }
    fn Focus(&self) -> bool {
        let frame = self.dispatcher.FrameSelection();
        matches!(
            frame.CompositionText(Q::SelectionFocus {
                selection: frame.Current(),
                focused: self.dispatcher.state.borrow().focused_node_id,
            }),
            Some(V::SelectionFocus(true))
        )
    }
    fn Target(&self) -> Option<EventTargetRef> {
        self.dispatcher.FrameSelection().PrepareEditing();
        super::super::page_input_method_controller::PasteEventTarget(
            self.dispatcher,
            self.source == Source::kDom || self.Focus(),
        )
    }
    fn Control(&self) -> Option<u64> {
        let document = self.dispatcher.document.borrow();
        crate::adapter::page_editing_visible_position::EnclosingTextControlForSelectionForFocus(
            document.GetDocument(),
            self.dispatcher.FrameSelection().Current(),
        )
    }
    fn Password(&self) -> bool {
        let document = self.dispatcher.document.borrow();
        let d = document.GetDocument();
        self.Control()
            .and_then(|id| d.FindNodeById(id))
            .is_some_and(|i| {
                let n = d.Node(i);
                n.IsHTMLElement("input")
                    && n.FindAttribute("type")
                        .is_some_and(|a| a.value.eq_ignore_ascii_case("password"))
            })
    }
    fn Range(&self) -> bool {
        let frame = self.dispatcher.FrameSelection();
        frame
            .CreateVisibleSelection(frame.Current())
            .is_some_and(|s| s.anchor != s.focus)
    }
    fn Smart(&self) -> bool {
        let frame = self.dispatcher.FrameSelection();
        frame
            .InteractionTextSettings()
            .is_some_and(|s| s.smart_insert_delete_enabled)
            && frame.SelectionControllerFlags().0
                == interaction_default::core::editing::text_granularity::TextGranularity::kWord
    }
    fn ClipboardSnapshot(
        &self,
        mut snapshot: PrimaryClipboardSnapshot,
    ) -> PrimaryClipboardSnapshot {
        // clipboard_commands.cc:171-175: the clipboard flavor alone must not
        // enable smart insertion when this frame's editor setting disables it.
        snapshot.smart_replace &= self
            .dispatcher
            .FrameSelection()
            .InteractionTextSettings()
            .is_some_and(|settings| settings.smart_insert_delete_enabled);
        snapshot
    }
    fn Transfer(&self, snapshot: &PrimaryClipboardSnapshot, rich: bool) -> Rc<DataTransfer> {
        let transfer = DataTransfer::Create();
        transfer.setData("text/plain", &snapshot.plain_text);
        if let Some(html) = snapshot.html.as_ref().filter(|_| rich) {
            transfer
                .GetDataObject()
                .SetHTMLAndBaseURL(&html.markup, &html.source_url);
        }
        if rich {
            let object = transfer.GetDataObject();
            for (kind, value) in [
                ("text/rtf", &snapshot.rtf),
                ("image/svg+xml", &snapshot.svg),
            ] {
                if let Some(value) = value {
                    object.SetData(kind, value);
                }
            }
            // DataObject::CreateFromClipboard substitutes admitted filenames
            // for text/uri-list; an ordinary URL list remains a string item.
            if snapshot.files.is_empty() {
                if let Some(value) = &snapshot.uri_list {
                    object.SetData("text/uri-list", value);
                }
            }
            for file in snapshot.png.iter().chain(&snapshot.files) {
                object.AddFile(file.clone());
            }
            for (kind, value) in &snapshot.custom_data {
                object.SetDataUTF16(kind, value);
            }
        }
        transfer.SetAccessPolicy(DataTransferAccessPolicy::Readable);
        transfer
    }
    fn DispatchInput(&self, input_type: InputType, transfer: Option<Rc<DataTransfer>>) -> bool {
        let Some(target) = self.Target() else {
            return true;
        };
        let mut event = InputEvent::CreateBeforeInputWithDataTransfer(
            input_type,
            transfer,
            EventIsComposing::kNotComposing,
            None,
            self.dispatcher.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        self.dispatcher.DispatchCanonicalEventTarget(&target, EventRef::new(event)).0 == interaction_dispatch::core::dom::events::event_dispatch_result::DispatchEventResult::kNotCanceled
    }
    fn ActiveEditContext(&self) -> bool {
        self.dispatcher
            .input_method_state
            .as_ref()
            .is_some_and(|s| s.GetActiveEditContext().is_some())
    }
}
impl PasteCommands for ClipboardCommandsOwner<'_, '_, '_> {
    type Error = &'static str;
    fn Source(&self) -> Source {
        self.source
    }
    fn Behavior(
        &self,
    ) -> Result<
        interaction_default::core::editing::editing_behavior::EditingBehaviorType,
        Self::Error,
    > {
        self.dispatcher
            .FrameSelection()
            .InteractionTextSettings()
            .map(|s| s.editing_behavior)
            .ok_or("ClipboardCommands EditingBehavior Settings")
    }
    fn ReadPrimarySelection(&self) -> Result<PrimaryClipboardSnapshot, Self::Error> {
        self.dispatcher
            .FrameSelection()
            .ReadPrimarySelection()
            .map(|snapshot| self.ClipboardSnapshot(snapshot))
            .map_err(|_| "ClipboardHost primary selection read")
    }
    fn ReadClipboard(&self) -> Result<PrimaryClipboardSnapshot, Self::Error> {
        self.dispatcher
            .FrameSelection()
            .ReadClipboard()
            .map(|snapshot| self.ClipboardSnapshot(snapshot))
            .map_err(|_| "ClipboardHost standard read")
    }
    fn DispatchPaste(&self, snapshot: &PrimaryClipboardSnapshot) -> Result<bool, Self::Error> {
        let Some(target) = self.Target() else {
            return Ok(true);
        };
        let transfer = self.Transfer(snapshot, true);
        let mut event = ClipboardEvent::new(
            "paste",
            Some(transfer.clone()),
            self.dispatcher.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        let result = self
            .dispatcher
            .DispatchCanonicalEventTarget(&target, EventRef::new(event))
            .0;
        transfer.SetAccessPolicyNumb();
        Ok(result == interaction_dispatch::core::dom::events::event_dispatch_result::DispatchEventResult::kNotCanceled)
    }
    fn FrameIsCurrent(&self) -> bool {
        self.Current()
    }
    fn CanPaste(&self) -> bool {
        self.dispatcher.FrameSelection().PrepareEditing();
        self.dispatcher
            .FrameSelection()
            .VisibleSelectionRootEditableElement()
            .is_some()
    }
    fn UpdateStyleAndLayoutForEditing(&self) -> Result<(), Self::Error> {
        self.dispatcher.FrameSelection().PrepareEditing();
        Ok(())
    }
    fn SelectionHasFocus(&self) -> bool {
        self.Focus()
    }
    fn CanEditRichly(&self) -> bool {
        super::super::page_input_method_controller::PasteCanEditRichly(self.dispatcher)
    }
    fn DispatchBeforeInput(
        &self,
        snapshot: &PrimaryClipboardSnapshot,
        rich: bool,
    ) -> Result<bool, Self::Error> {
        let transfer = self.Transfer(snapshot, rich);
        *self.transfer.borrow_mut() = Some(transfer.clone());
        Ok(self.DispatchInput(InputType::kInsertFromPaste, Some(transfer)))
    }
    fn HasActiveEditContext(&self) -> bool {
        self.ActiveEditContext()
    }
    fn PasteFromClipboard(&self, snapshot: &PrimaryClipboardSnapshot) -> Result<(), Self::Error> {
        super::super::page_input_method_controller::PasteFromClipboard(
            self.dispatcher,
            snapshot,
            self.transfer.borrow_mut().take(),
        )
    }
    fn PasteAsPlainText(&self, snapshot: &PrimaryClipboardSnapshot) -> Result<(), Self::Error> {
        super::super::page_input_method_controller::PastePlainText(self.dispatcher, snapshot)
    }
}
impl CopyCutCommands for ClipboardCommandsOwner<'_, '_, '_> {
    type Error = &'static str;
    fn Source(&self) -> Source {
        self.source
    }
    fn UpdateStyleAndLayoutForEditing(&self) -> Result<(), Self::Error> {
        self.dispatcher.FrameSelection().PrepareEditing();
        Ok(())
    }
    fn DispatchCopyOrCut(&self, cut: bool) -> Result<bool, Self::Error> {
        self.DispatchWritableEvent(if cut { "cut" } else { "copy" })
    }
    fn FrameIsCurrent(&self) -> bool {
        self.Current()
    }
    fn CanCopy(&self) -> bool {
        self.dispatcher.FrameSelection().PrepareEditing();
        self.Range() && !self.Password()
    }
    fn CanCut(&self) -> bool {
        if !CopyCutCommands::CanCopy(self) {
            return false;
        }
        let Some(selection) = self.dispatcher.FrameSelection().Current() else {
            return false;
        };
        let owner = self.dispatcher.document.borrow();
        let facts = dom::persistent_document::DOMEditableFacts(owner.GetDocument());
        [selection.anchor.node_id, selection.focus.node_id]
            .into_iter()
            .all(|node| {
                node.is_some_and(|node| {
                    foundation::editable::IsEditable(&facts, node, Default::default())
                })
            })
    }
    fn SelectionIsHidden(&self) -> bool {
        !self.Focus() && (!self.Range() || self.Control().is_some())
    }
    fn SelectionHasFocus(&self) -> bool {
        self.Focus()
    }
    fn WriteSelectionToClipboard(&self) -> Result<(), Self::Error> {
        let frame = self.dispatcher.FrameSelection();
        let Some(selection) = frame.Current() else {
            return Ok(());
        };
        let Some(V::Text(text)) = frame.CompositionText(Q::SelectedTextForClipboard {
            selection,
            include_image_alt_text: false,
        }) else {
            return Err("FrameSelection selected clipboard text");
        };
        let in_control = self.Control().is_some();
        let html = if in_control {
            None
        } else {
            let markup = String::from_utf16_lossy(
                &frame
                    .SelectedHtmlForClipboard()
                    .map_err(|_| "FrameSelection selected clipboard html")?,
            );
            Some(ClipboardHtml {
                fragment_start: 0,
                fragment_end: markup.encode_utf16().count() as u32,
                markup,
                source_url: self
                    .dispatcher
                    .document
                    .borrow()
                    .GetDocument()
                    .DocumentURL()
                    .to_owned(),
            })
        };
        let text = String::from_utf16_lossy(&text);
        frame
            .WriteClipboard(&PrimaryClipboardSnapshot {
                plain_text: if in_control {
                    text
                } else {
                    text.replace('\u{a0}', " ")
                },
                html,
                smart_replace: self.Smart(),
                ..Default::default()
            })
            .map_err(|_| "ClipboardHost selection write")
    }
    fn DispatchBeforeCut(&self) -> Result<bool, Self::Error> {
        Ok(self.DispatchInput(InputType::kDeleteByCut, None))
    }
    fn HasActiveEditContext(&self) -> bool {
        self.ActiveEditContext()
    }
    fn DeleteSelection(&self) -> Result<(), Self::Error> {
        super::super::page_input_method_controller::DeleteCutSelection(
            self.dispatcher,
            self.Smart(),
        )
    }
}
