//! Call-scoped Page projections for Blink's retained InputMethodController.
//! The translated controller owns event order and composition replacement;
//! these ports borrow the live DOM, text-control selection, and editor owners.

use super::page_frame_selection::{
    CompositionTextQuery as Query, CompositionTextResult as ResultValue, PageFrameSelection,
};
use super::{
    event::EventTargetResolver,
    page_event_dispatcher::{DispatchEventResult, EventDispatcher},
};
use crate::{
    input_event::{CompositionEvent, CompositionEventType, TextInputEvent},
    ownership::{InteractionDOMMutationEmitter, InteractionDocument, ReadNode},
    editing_owners::{Selection, SelectionState, SyntheticEventDispatcher, PageEditingOwners},
};
use dom::{
    dom_mutation::{DOMMutation, DOMMutationType},
    DOMStaticRange,
};
use dom::{DOMBoundaryPoint, DOMNodeType, DOMRangeHandle, UserInteractionState};
use interaction_default::core::editing::set_selection_options::SetSelectionOptions;
use interaction_default::core::editing::{
    commands::typing_command::TextCompositionType,
    ime::input_method_controller::{
        CompositionCommands, CompositionEndpoint, CompositionEvents, CompositionNode,
        CompositionRange, CompositionRangeRef, CompositionSelection, ConfirmCompositionBehavior,
        InputMethodController, InputMethodControllerState, PlainTextRange,
        PreparedCompositionGeometry, RevealCompositionSelectionScope, ScopedCompositionEvents,
        TypingContinuation, UndoCompositionSelection,
    },
};
use interaction_dispatch::core::{
    dom::events::{
        event::EventTargetRef, event_dispatch_result::DispatchEventResult as NativeResult,
    },
    events::input_event::InputType,
};
use layoutng_assembly::{editing_state::DOMSelection, EditingPosition, EditingTextAffinity};
use std::{cell::RefCell, rc::Rc};

#[path = "page_event_handler_text_input.rs"]
pub mod event_handler_text_input;

#[path = "page_ime_typing.rs"]
mod typing;
pub(crate) use typing::{ImeTypingSelection, ImeTypingState};

pub(crate) fn PasteEventTarget(
    dispatcher: &EventDispatcher<'_, '_>,
    has_focus: bool,
) -> Option<EventTargetRef> {
    let frame = dispatcher.FrameSelection();
    let ResultValue::ClipboardSelection { target, .. } =
        frame.CompositionText(Query::ClipboardSelection {
            selection: frame.Current(),
            focused: dispatcher.KeyboardState().borrow().focused_node_id,
            has_focus,
        })?
    else {
        return None;
    };
    dispatcher.KeyboardTargets().ResolveTarget(target?)
}
pub(crate) fn PasteCanEditRichly(dispatcher: &EventDispatcher<'_, '_>) -> bool {
    let frame = dispatcher.FrameSelection();
    matches!(
        frame.CompositionText(Query::ClipboardSelection {
            selection: frame.Current(),
            focused: dispatcher.KeyboardState().borrow().focused_node_id,
            has_focus: false
        }),
        Some(ResultValue::ClipboardSelection {
            richly_editable: true,
            ..
        })
    )
}
pub(crate) fn PastePlainText(
    dispatcher: &EventDispatcher<'_, '_>,
    snapshot:&interaction_default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
) -> Result<(), &'static str> {
    let frame = dispatcher.FrameSelection();
    // Target lookup is performed after beforeinput, which may move selection.
    let Some(target) = PasteEventTarget(dispatcher, true) else {
        return Ok(());
    };
    let text: Vec<_> = snapshot.plain_text.encode_utf16().collect();
    let identity = dispatcher
        .KeyboardTargets()
        .DocumentOwner()
        .borrow()
        .GetDocument()
        .RootHandle()
        .Identity();
    let mut event =
        interaction_dispatch::core::events::text_event::TextEvent::CreateForPlainTextPaste(
            Some(dispatcher.KeyboardTargets().WindowView()),
            &text,
            snapshot.smart_replace,
            dispatcher.EventClock(),
        );
    use interaction_dispatch::core::dom::events::event::EventInterface;
    event.BaseEventMut().SetTrusted(true);
    let result = dispatcher
        .DispatchCanonicalKeyboardEvent(
            &target,
            interaction_dispatch::core::dom::events::event::EventRef::new(event),
        )
        .0;
    if result != NativeResult::kNotCanceled
        || !frame.IsAvailable()
        || dispatcher
            .KeyboardTargets()
            .DocumentOwner()
            .borrow()
            .GetDocument()
            .RootHandle()
            .Identity()
            != identity
    {
        return Ok(());
    }
    let fragment = dispatcher.CreatePasteTextFragment(&text)?;
    dispatcher.ApplyPasteReplacement(&fragment, snapshot.smart_replace, true, None)
}
pub(crate) fn PasteFromClipboard(
    dispatcher: &EventDispatcher<'_, '_>,
    snapshot:&interaction_default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
    transfer: Option<Rc<interaction_dispatch::core::clipboard::data_transfer::DataTransfer>>,
) -> Result<(), &'static str> {
    use base64::Engine as _;
    let frame = dispatcher.FrameSelection();
    let target = dispatcher.KeyboardTargets().DocumentOwner();
    let mut fragment = if let Some(html) = &snapshot.html {
        let markup: Vec<u16> = html.markup.encode_utf16().collect();
        crate::adapter::page_markup_sanitization::CreateStrictlyProcessedFragmentFromMarkupWithContext(frame.MarkupSanitizationHost().as_ref(),&target,&markup,html.fragment_start as usize,html.fragment_end as usize,&html.source_url).map_err(|_|"ClipboardCommands strict markup sanitization")?
    } else {
        None
    };
    // SystemClipboard::ReadImageAsImageMarkup and PNGToImageMarkup,
    // clipboard_utilities.cc:98-108. PNG follows HTML and precedes plain text.
    if fragment.is_none() {
        if let Some(bytes) = snapshot
            .png
            .as_ref()
            .and_then(|file| file.CachedBytes().ok())
            .filter(|bytes| !bytes.is_empty())
        {
            let markup = format!(
                "<img src=\"data:image/png;base64,{}\" alt=\"\"/>",
                base64::engine::general_purpose::STANDARD.encode(bytes.as_ref())
            )
            .encode_utf16()
            .collect::<Vec<_>>();
            fragment = Some(
                crate::adapter::page_selection_markup::CreateFragmentFromMarkup(
                    &target,
                    &markup,
                    0,
                    markup.len(),
                    "",
                )
                .map_err(|_| "ClipboardCommands PNG image markup")?,
            );
        }
    }
    let (fragment, match_style) = match fragment {
        Some(fragment) => (fragment, false),
        None => {
            if snapshot.plain_text.is_empty() {
                return Ok(());
            }
            (
                dispatcher.CreatePasteTextFragment(
                    &snapshot.plain_text.encode_utf16().collect::<Vec<_>>(),
                )?,
                true,
            )
        }
    };
    let Some(target) = PasteEventTarget(dispatcher, true) else {
        return Ok(());
    };
    let identity = dispatcher
        .KeyboardTargets()
        .DocumentOwner()
        .borrow()
        .GetDocument()
        .RootHandle()
        .Identity();
    let mut event =
        interaction_dispatch::core::events::text_event::TextEvent::CreateForFragmentPaste(
            Some(dispatcher.KeyboardTargets().WindowView()),
            Rc::new(fragment.clone()),
            snapshot.smart_replace,
            match_style,
            transfer
                .clone()
                .ok_or("ClipboardCommands paste DataTransfer")?,
            dispatcher.EventClock(),
        );
    use interaction_dispatch::core::dom::events::event::{EventInterface, EventRef};
    event.BaseEventMut().SetTrusted(true);
    if dispatcher
        .DispatchCanonicalKeyboardEvent(&target, EventRef::new(event))
        .0
        != NativeResult::kNotCanceled
        || !frame.IsAvailable()
        || dispatcher
            .KeyboardTargets()
            .DocumentOwner()
            .borrow()
            .GetDocument()
            .RootHandle()
            .Identity()
            != identity
    {
        return Ok(());
    }
    dispatcher.ApplyPasteReplacement(&fragment, snapshot.smart_replace, match_style, transfer)
}

pub(crate) fn ApplyPasteReplacement(
    dispatcher: &EventDispatcher<'_, '_>,
    document: InteractionDocument,
    editor: Rc<PageEditingOwners>,
    selections: Rc<SelectionState>,
    interaction: Rc<RefCell<UserInteractionState>>,
    emit: InteractionDOMMutationEmitter,
    frame: Rc<PageFrameSelection>,
    fragment: &super::page_selection_markup::RetainedFragment,
    smart: bool,
    match_style: bool,
    transfer: Option<Rc<interaction_dispatch::core::clipboard::data_transfer::DataTransfer>>,
) -> Result<(), &'static str> {
    if !fragment.IsCurrent() {
        return Ok(());
    }
    let selection = Rc::new(PageCompositionSelection {
        document: document.clone(),
        editor,
        selections,
        interaction,
        frame: Some(frame),
        input_frame: dispatcher.input_frame.clone(),
    });
    let commands = PageCompositionCommands {
        document,
        dispatcher,
        selection,
        emit,
        ending: RefCell::new(None),
    };
    typing::ReplaceFragment(&commands, fragment.root, smart, match_style, transfer)
}

struct PageCompositionNode {
    _document: InteractionDocument,
    id: u64,
}
impl CompositionNode for PageCompositionNode {
    fn NodeId(&self) -> u64 {
        self.id
    }
}

fn Endpoint(document: &InteractionDocument, point: DOMBoundaryPoint) -> CompositionEndpoint {
    CompositionEndpoint {
        node: Rc::new(PageCompositionNode {
            _document: document.clone(),
            id: point.node_id,
        }),
        offset: point.offset,
    }
}

/// Retains the DOM's registered live Range, including its mutation adjustments.
// Source: input_method_controller.h:192-194; input_method_controller.cc:1128-1134.
struct PageCompositionRange {
    document: InteractionDocument,
    root: u64,
    range: DOMRangeHandle,
    frame: Option<Rc<PageFrameSelection>>,
}
impl CompositionRange for PageCompositionRange {
    fn RootEditableNodeId(&self) -> Option<u64> {
        self.IsConnected().then_some(self.root)
    }
    fn Collapsed(&self) -> bool {
        self.document
            .borrow()
            .GetDocument()
            .Range(&self.range)
            .is_none_or(|range| range.Collapsed())
    }
    fn IsConnected(&self) -> bool {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        document.Range(&self.range).is_some_and(|range| {
            [range.start, range.end].iter().all(|point| {
                document
                    .FindNodeById(point.node_id)
                    .is_some_and(|node| document.IsConnectedToDocument(node))
            })
        })
    }
    fn CollapseAtDocumentStart(&self) {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let point = DOMBoundaryPoint {
            node_id: document.Node(document.Root()).Id(),
            offset: 0,
        };
        let _ = document.SetRange(&self.range, point, point);
    }
    fn Text(&self) -> Vec<u16> {
        self.PlainTextEmittingOriginalText()
    }
    fn PlainTextEmittingOriginalText(&self) -> Vec<u16> {
        if !super::page_editing_owners::IsTextControl(&self.document, self.root) {
            let range = self.document.borrow().GetDocument().Range(&self.range);
            return range
                .and_then(|range| {
                    self.frame.as_ref()?.CompositionText(Query::Text {
                        range,
                        original: true,
                    })
                })
                .and_then(|value| {
                    if let ResultValue::Text(text) = value {
                        Some(text)
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
        }
        let Some(offsets) = self.PlainTextRange() else {
            return Vec::new();
        };
        ReadNode(&self.document, self.root, |document, _, root| {
            let text = document.TextControlInnerEditorValueUTF16(root);
            text.get(offsets.start as usize..offsets.end as usize)
                .unwrap_or_default()
                .to_vec()
        })
        .unwrap_or_default()
    }
    fn PlainTextRange(&self) -> Option<PlainTextRange> {
        if !super::page_editing_owners::IsTextControl(&self.document, self.root) {
            let range = self.document.borrow().GetDocument().Range(&self.range)?;
            return match self.frame.as_ref()?.CompositionText(Query::Offsets {
                root: self.root,
                range,
            })? {
                ResultValue::Offsets(range) => Some(range),
                _ => None,
            };
        }
        ReadNode(&self.document, self.root, |document, _, root| {
            let range = document.Range(&self.range)?;
            Some(PlainTextRange {
                start: i32::try_from(document.TextControlInnerEditorOffset(root, range.start)?)
                    .ok()?,
                end: i32::try_from(document.TextControlInnerEditorOffset(root, range.end)?).ok()?,
            })
        })
        .flatten()
    }
    fn DOMEndpoints(&self) -> Option<(CompositionEndpoint, CompositionEndpoint)> {
        let range = self.document.borrow().GetDocument().Range(&self.range)?;
        Some((
            Endpoint(&self.document, range.start),
            Endpoint(&self.document, range.end),
        ))
    }
    fn SetStart(&self, endpoint: &CompositionEndpoint) {
        let _ = self.document.borrow().GetDocument().SetRangeStart(
            &self.range,
            DOMBoundaryPoint {
                node_id: endpoint.node.NodeId(),
                offset: endpoint.offset,
            },
        );
    }
    fn SetEnd(&self, endpoint: &CompositionEndpoint) {
        let _ = self.document.borrow().GetDocument().SetRangeEnd(
            &self.range,
            DOMBoundaryPoint {
                node_id: endpoint.node.NodeId(),
                offset: endpoint.offset,
            },
        );
    }
}

struct PageCompositionSelection {
    document: InteractionDocument,
    selections: Rc<SelectionState>,
    interaction: Rc<RefCell<UserInteractionState>>,
    editor: Rc<PageEditingOwners>,
    frame: Option<Rc<PageFrameSelection>>,
    input_frame: Option<Rc<super::source_input_managers::PointerFrameInputState>>,
}
impl PageCompositionSelection {
    fn Root(&self) -> Option<u64> {
        if let Some(root) = self.selections.ActiveControl() {
            return (self.interaction.borrow().focused_node_id == Some(root)).then_some(root);
        }
        let frame = self.frame.as_ref()?;
        let control = {
            let owner = self.document.borrow();
            super::page_focus_controller::EnclosingTextControlForSelectionForFocus(
                owner.GetDocument(),
                frame.Current(),
            )
        };
        if let Some(control) = control {
            return (self.interaction.borrow().focused_node_id == Some(control)).then_some(control);
        }
        frame.VisibleSelectionRootEditableElement()
    }
    fn Current(&self) -> Option<Selection> {
        let root = self.Root()?;
        if super::page_editing_owners::IsTextControl(&self.document, root) {
            return self.selections.Get(root);
        }
        let range = self.CurrentDOMRange()?;
        let ResultValue::Offsets(offsets) = self
            .frame
            .as_ref()?
            .CompositionText(Query::Offsets { root, range })?
        else {
            return None;
        };
        let selection = self.selections.DOMSelection()?;
        let ResultValue::AnchorFirst(forward) = self
            .frame
            .as_ref()?
            .CompositionText(Query::AnchorFirst(selection))?
        else {
            return None;
        };
        Some(Selection {
            anchor: if forward { offsets.start } else { offsets.end } as usize,
            focus: if forward { offsets.end } else { offsets.start } as usize,
        })
    }
    fn CurrentDOMRange(&self) -> Option<DOMStaticRange> {
        match self
            .frame
            .as_ref()?
            .CompositionText(Query::SelectionRange(self.selections.DOMSelection()?))?
        {
            ResultValue::Range(range) => Some(range),
            _ => None,
        }
    }
    fn SetDOMRange(&self, range: DOMStaticRange, forward: bool, show_handle: bool) -> bool {
        let Some(frame) = &self.frame else {
            return false;
        };
        let start = EditingPosition::new(range.start.node_id, range.start.offset);
        let end = EditingPosition::new(range.end.node_id, range.end.offset);
        frame.SetSelection(
            Some(DOMSelection {
                anchor: if forward { start } else { end },
                focus: if forward { end } else { start },
                affinity: EditingTextAffinity::Downstream,
            }),
            SetSelectionOptions {
                should_show_handle: show_handle,
                do_not_set_focus: true,
                ..Default::default()
            },
        );
        true
    }
}
// Source: input_method_controller.cc:563-598,1358-1368,1377-1415.
impl CompositionSelection for PageCompositionSelection {
    fn IsNone(&self) -> bool {
        self.Current().is_none()
    }
    fn IsRange(&self) -> bool {
        self.Current()
            .is_some_and(|selection| !selection.Collapsed())
    }
    fn IsAnchorFirst(&self) -> bool {
        self.Current()
            .is_none_or(|selection| selection.anchor <= selection.focus)
    }
    fn IsHandleVisible(&self) -> bool {
        self.selections.SelectionHandlesVisible()
    }
    fn SelectedText(&self) -> Vec<u16> {
        if self.selections.ActiveControl().is_none() {
            return self
                .CurrentDOMRange()
                .and_then(|range| {
                    self.frame.as_ref()?.CompositionText(Query::Text {
                        range,
                        original: false,
                    })
                })
                .and_then(|value| {
                    if let ResultValue::Text(text) = value {
                        Some(text)
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
        }
        let (Some(root), Some(selection)) = (self.Root(), self.Current()) else {
            return Vec::new();
        };
        ReadNode(&self.document, root, |document, _, root| {
            let text = document.TextControlInnerEditorValueUTF16(root);
            text.get(selection.Start()..selection.End())
                .unwrap_or_default()
                .to_vec()
        })
        .unwrap_or_default()
    }
    fn GetSelectionOffsets(&self) -> Option<PlainTextRange> {
        let selection = self.Current()?;
        Some(PlainTextRange {
            start: i32::try_from(selection.Start()).ok()?,
            end: i32::try_from(selection.End()).ok()?,
        })
    }
    fn CanEdit(&self) -> bool {
        self.Root().is_some_and(|root| {
            if super::page_editing_owners::IsTextControl(&self.document, root) {
                self.editor.CaretFor(&self.document, root).is_some()
            } else {
                self.selections
                    .DOMSelection()
                    .and_then(|selection| selection.anchor.node_id)
                    .is_some_and(|node| {
                        let owner = self.document.borrow();
                        foundation::editable::IsEditable(
                            &dom::persistent_document::DOMEditableFacts(owner.GetDocument()),
                            node,
                            Default::default(),
                        )
                    })
            }
        })
    }
    fn HasRootEditableElement(&self) -> bool {
        self.Root().is_some()
    }
    fn RootEditableNodeId(&self) -> Option<u64> {
        self.Root()
    }
    fn RootEditableTextLength(&self) -> Option<i32> {
        if self.selections.ActiveControl().is_none() {
            return match self
                .frame
                .as_ref()?
                .CompositionText(Query::Length { root: self.Root()? })?
            {
                ResultValue::Length(length) => length.try_into().ok(),
                _ => None,
            };
        }
        ReadNode(&self.document, self.Root()?, |document, _, root| {
            i32::try_from(document.ControlValueUTF16(root).len()).ok()
        })
        .flatten()
    }
    fn RestoreSelection(&self, range: PlainTextRange, forward: bool, show_handle: bool) -> bool {
        let Some(root) = self.Root() else {
            return false;
        };
        let Some(length) = self.RootEditableTextLength() else {
            return false;
        };
        if range.start < 0 || range.end < range.start || range.end > length {
            return false;
        }
        if self.selections.ActiveControl().is_none() {
            let Some(ResultValue::Range(range)) = self.frame.as_ref().and_then(|frame| {
                frame.CompositionText(Query::Range {
                    root,
                    offsets: range,
                })
            }) else {
                return false;
            };
            return self.SetDOMRange(range, forward, show_handle);
        }
        let (start, end) = (range.start as usize, range.end as usize);
        self.selections.Set(
            root,
            Selection {
                anchor: if forward { start } else { end },
                focus: if forward { end } else { start },
            },
        );
        self.selections.SetSelectionHandlesVisible(show_handle);
        true
    }
    fn SetSelectionOffsets(&self, range: PlainTextRange, continuation: TypingContinuation) -> bool {
        let changed = self.RestoreSelection(range, true, self.IsHandleVisible());
        if changed && continuation == TypingContinuation::kEnd {
            self.editor.ime_typing.typing.CloseTyping();
        }
        changed
    }
    fn SelectComposition(&self, range: &CompositionRangeRef) {
        if self.selections.ActiveControl().is_none() {
            if let Some((start, end)) = range.DOMEndpoints() {
                self.SetDOMRange(
                    DOMStaticRange {
                        start: DOMBoundaryPoint {
                            node_id: start.node.NodeId(),
                            offset: start.offset,
                        },
                        end: DOMBoundaryPoint {
                            node_id: end.node.NodeId(),
                            offset: end.offset,
                        },
                    },
                    true,
                    self.IsHandleVisible(),
                );
            }
            return;
        }
        if let Some(offsets) = range.PlainTextRange() {
            self.SetSelectionOffsets(offsets, TypingContinuation::kContinue);
        }
    }
    fn HandlingInputEvent(&self) -> Option<bool> {
        self.input_frame
            .as_ref()
            .map(|frame| frame.handling_input_event.get())
    }
    fn SetHandlingInputEvent(&self, handling: bool) {
        // Source input_method_controller.cc:564-598. SelectComposition clears
        // this widget flag only around selection and restores the same owner.
        self.input_frame
            .as_ref()
            .expect("composition widget input owner")
            .handling_input_event
            .set(handling);
    }
}

struct PageRevealSelectionScope;
impl RevealCompositionSelectionScope for PageRevealSelectionScope {}

struct PageCompositionEvents<'scope, 'listener, 'frame> {
    dispatcher: &'scope EventDispatcher<'listener, 'frame>,
    interaction: Rc<RefCell<UserInteractionState>>,
}
// Source: input_method_controller.cc:385-426,436-504.
impl CompositionEvents for PageCompositionEvents<'_, '_, '_> {
    fn FocusedElement(&self) -> Option<EventTargetRef> {
        self.dispatcher
            .KeyboardTargets()
            .ResolveTarget(self.interaction.borrow().focused_node_id?)
    }
    fn DispatchComposition(
        &self,
        target: &EventTargetRef,
        kind: &'static str,
        text: &[u16],
        scoped: bool,
    ) {
        self.dispatcher
            .CompositionDispatchEvent(target, kind, text, scoped);
    }
    fn DispatchBeforeInput(
        &self,
        target: &EventTargetRef,
        input_type: InputType,
        text: &[u16],
        composing: bool,
    ) -> NativeResult {
        self.dispatcher
            .CompositionBeforeInput(target, input_type, text, composing)
    }
    fn EventQueueScope(&self) -> Box<dyn ScopedCompositionEvents + '_> {
        self.dispatcher.CompositionEventQueueScope()
    }
    fn EventsAreScoped(&self) -> bool {
        self.dispatcher.CompositionEventsAreScoped()
    }
    fn RevealSelectionScope(&self) -> Box<dyn RevealCompositionSelectionScope + '_> {
        // The text-control adapter has no implicit reveal operation to suppress.
        Box::new(PageRevealSelectionScope)
    }
}

struct PageCompositionCommands<'scope, 'listener, 'frame> {
    document: InteractionDocument,
    dispatcher: &'scope EventDispatcher<'listener, 'frame>,
    selection: Rc<PageCompositionSelection>,
    emit: InteractionDOMMutationEmitter,
    ending: RefCell<Option<(DOMBoundaryPoint, DOMBoundaryPoint)>>,
}
impl PageCompositionCommands<'_, '_, '_> {
    fn MarkerOwner(&self) -> super::page_frame_selection::CompositionMarkerOwner {
        let identity = self.document.borrow().GetDocument().RootHandle().Identity();
        self.selection.editor.ime_typing.MarkerOwner(identity)
    }
    fn ApplyMarkerUpdate(&self, update: super::page_frame_selection::CompositionMarkerUpdate) {
        let frame = self
            .selection
            .frame
            .as_ref()
            .expect("IME markers require the canonical FrameSelection geometry");
        let Some(ResultValue::Markers {
            ranges,
            invalidated,
        }) = frame.CompositionText(Query::Markers(update))
        else {
            panic!("IME marker update requires prepared resident geometry");
        };
        self.selection
            .selections
            .SetDocumentCompositionMarkers(ranges);
        for node in invalidated {
            frame.InvalidateCompositionMarkerPaint(node);
        }
    }
    fn Insert(&self, text: &[u16], select: bool, composition: TextCompositionType) {
        self.InsertWithInputType(text, select, composition, InputType::kNone);
    }
    fn InsertWithInputType(
        &self,
        text: &[u16],
        select: bool,
        composition: TextCompositionType,
        input_type: InputType,
    ) {
        let Some(root) = self.selection.Root().filter(|_| self.selection.CanEdit()) else {
            return;
        };
        // Source: typing_command.cc:446-448. A caret plus empty insertion is
        // already complete; it must not publish a second input after deletion.
        if text.is_empty() && !self.selection.IsRange() {
            return;
        }
        if !super::page_editing_owners::IsTextControl(&self.document, root) {
            if input_type == InputType::kNone {
                typing::Insert(self, text, select, composition);
            } else {
                typing::InsertWithInputType(self, text, select, composition, input_type);
            }
            return;
        }
        let composing = composition != TextCompositionType::kTextCompositionNone;
        let dispatcher = self.dispatcher;
        let dispatch: SyntheticEventDispatcher<'_> = Rc::new(move |event, target| {
            let data = event
                .data_utf16
                .as_ref()
                .and_then(|data| data.clone())
                .unwrap_or_else(|| Rc::from(event.text.encode_utf16().collect::<Vec<_>>()));
            dispatcher.CompositionDispatchInput(
                target,
                &data,
                if composing {
                    InputType::kInsertCompositionText
                } else if input_type != InputType::kNone {
                    input_type
                } else {
                    InputType::kInsertText
                },
                composing,
            );
        });
        *self.ending.borrow_mut() = self.selection.editor.InsertImeText(
            &self.document,
            root,
            text,
            select,
            composing,
            &self.emit,
            &dispatch,
        );
    }
}
// Source: input_method_controller.cc:476-499,1103-1157,1358-1375.
impl CompositionCommands for PageCompositionCommands<'_, '_, '_> {
    type ImeTextSpan = interaction_default::core::editing::ime::ime_text_span::ImeTextSpan;
    fn IsAvailable(&self) -> bool {
        self.selection.Root().is_some_and(|root| {
            ReadNode(&self.document, root, |document, _, index| {
                document.IsConnectedToDocument(index)
            })
            .unwrap_or(false)
        })
    }
    fn HasDocument(&self) -> bool {
        true
    }
    fn NeedsIncrementalInsertion(&self, _text: &[u16]) -> bool {
        // Rechrom's control editor does not install an incremental insertion owner.
        false
    }
    fn DeleteSelection(&self) {
        if self.selection.IsRange() {
            self.Insert(&[], false, TextCompositionType::kTextCompositionUpdate);
        }
    }
    fn InsertCompositionText(
        &self,
        text: &[u16],
        select_inserted: bool,
        composition: TextCompositionType,
        _incremental: bool,
    ) {
        if composition == TextCompositionType::kTextCompositionConfirm
            && self.selection.selections.ActiveControl().is_some()
        {
            let Some(target) =
                self.dispatcher
                    .KeyboardTargets()
                    .ResolveTarget(match self.selection.Root() {
                        Some(root) => root,
                        None => return,
                    })
            else {
                return;
            };
            if self
                .dispatcher
                .CompositionDispatchTextInput(&target, text, true)
                != NativeResult::kNotCanceled
            {
                return;
            }
        }
        self.Insert(text, select_inserted, composition);
    }
    fn HandleCompositionTextInput(&self, text: &[u16]) {
        self.InsertCompositionText(
            text,
            false,
            TextCompositionType::kTextCompositionConfirm,
            false,
        );
    }
    fn EditorInsertText(&self, text: &[u16]) {
        let Some(root) = self.selection.Root() else {
            return;
        };
        let Some(target) = self.dispatcher.KeyboardTargets().ResolveTarget(root) else {
            return;
        };
        if self
            .dispatcher
            .CompositionDispatchTextInput(&target, text, false)
            == NativeResult::kNotCanceled
        {
            self.Insert(text, false, TextCompositionType::kTextCompositionNone);
        }
    }
    fn CloseTyping(&self) {
        self.selection.editor.ime_typing.typing.CloseTyping();
    }
    fn UpdateOpenTypingCommandSelection(&self) {
        typing::UpdateSelection(self);
    }
    fn ValidUndoEndingSelection(&self) -> Option<UndoCompositionSelection> {
        let (start, end) = (*self.ending.borrow())?;
        let anchor_is_text = ReadNode(&self.document, start.node_id, |_, node, _| {
            node.Type() == DOMNodeType::kText
        })?;
        Some(UndoCompositionSelection {
            anchor_is_text,
            most_forward_anchor: Endpoint(&self.document, start),
            focus: Endpoint(&self.document, end),
        })
    }
    fn CreateCompositionRange(&self) -> CompositionRangeRef {
        let root = self
            .selection
            .Root()
            .expect("composition has editable root");
        let range = {
            let mut owner = self.document.borrow_mut();
            let document = owner.GetDocumentMut();
            let point = DOMBoundaryPoint {
                node_id: document.Node(document.Root()).Id(),
                offset: 0,
            };
            document.CreateRange(point, point).expect("document range")
        };
        Rc::new(PageCompositionRange {
            document: self.document.clone(),
            root,
            range,
            frame: self.selection.frame.clone(),
        })
    }
    fn IsTextTooLongAt(&self, range: &CompositionRangeRef) -> bool {
        // Source input_method_controller.cc:600-609. The element retains
        // dirty/user-edit state and its native TooLong predicate in DOM.
        let Some((start, _)) = range.DOMEndpoints() else {
            return false;
        };
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let mut node = document.FindNodeById(start.node.NodeId());
        while let Some(index) = node {
            if document.Node(index).IsHTMLElement("input")
                || document.Node(index).IsHTMLElement("textarea")
            {
                return document.TextControlTooLong(index);
            }
            node = document.ParentOrShadowHostElement(index);
        }
        false
    }
    fn RemoveSuggestionMarkerInRangeOnFinish(&self, range: &CompositionRangeRef) {
        // Source input_method_controller.cc:1370-1375. Read the same live Range
        // owner before removing only suggestions marked remove-on-finish.
        if let Some((start, end)) = range.DOMEndpoints() {
            self.ApplyMarkerUpdate(
                super::page_frame_selection::CompositionMarkerUpdate::RemoveSuggestionsOnFinish(
                    self.MarkerOwner(),
                    DOMStaticRange {
                        start: DOMBoundaryPoint {
                            node_id: start.node.NodeId(),
                            offset: start.offset,
                        },
                        end: DOMBoundaryPoint {
                            node_id: end.node.NodeId(),
                            offset: end.offset,
                        },
                    },
                ),
            );
        }
    }
    fn RemoveCompositionMarkers(&self) {
        let control = self
            .selection
            .selections
            .ActiveControl()
            .filter(|root| self.selection.selections.Composition(*root).is_some());
        self.selection.selections.ClearCompositions();
        typing::UpdateMarkers(self, None);
        // Text-control composition ranges live on SelectionState rather than
        // DocumentMarkerController, but their underline is recorded by the
        // generated inner-editor text client. Removal must invalidate that
        // original marker owner just like DocumentMarkerController::Remove.
        if let (Some(frame), Some(control)) = (self.selection.frame.as_ref(), control) {
            frame.InvalidateCompositionMarkerPaint(control);
        }
    }
    fn RemoveAutocorrectSuggestionMarkers(&self) {
        // Source input_method_controller.cc:2051-2060.
        self.ApplyMarkerUpdate(
            super::page_frame_selection::CompositionMarkerUpdate::RemoveAutocorrect(
                self.MarkerOwner(),
            ),
        );
    }
    fn AddDefaultCompositionMarker(&self, range: &CompositionRangeRef) {
        if let (Some(root), Some(offsets)) = (range.RootEditableNodeId(), range.PlainTextRange()) {
            self.selection.selections.SetComposition(
                root,
                Some(Selection {
                    anchor: offsets.start as usize,
                    focus: offsets.end as usize,
                }),
            );
            if !super::page_editing_owners::IsTextControl(&self.document, root) {
                if let Some((start, end)) = range.DOMEndpoints() {
                    typing::UpdateMarkers(
                        self,
                        Some(DOMStaticRange {
                            start: DOMBoundaryPoint {
                                node_id: start.node.NodeId(),
                                offset: start.offset,
                            },
                            end: DOMBoundaryPoint {
                                node_id: end.node.NodeId(),
                                offset: end.offset,
                            },
                        }),
                    );
                }
            }
        }
    }
    fn AddImeTextSpans(&self, spans: &[Self::ImeTextSpan], root: u64, offset: i32) {
        if spans.is_empty() {
            return;
        }
        self.ApplyMarkerUpdate(
            super::page_frame_selection::CompositionMarkerUpdate::AddImeSpans(
                self.MarkerOwner(),
                root,
                offset.try_into().expect("nonnegative IME text span offset"),
                spans.to_vec(),
            ),
        );
    }
    fn InvalidateCompositionAnchor(&self, _range: &CompositionRangeRef) {
        self.document
            .borrow_mut()
            .GetDocumentMut()
            .StyleStateMut()
            .impact
            .Merge(dom::style_state::StyleUpdateImpact {
                paint: true,
                ..Default::default()
            });
    }
}

fn WithController<'scope, 'listener: 'scope, 'frame: 'scope>(
    document: InteractionDocument,
    dispatcher: &'scope EventDispatcher<'listener, 'frame>,
    editor: Rc<PageEditingOwners>,
    selections: Rc<SelectionState>,
    interaction: Rc<RefCell<UserInteractionState>>,
    emit: InteractionDOMMutationEmitter,
    retained: Rc<InputMethodControllerState>,
    frame: Option<Rc<PageFrameSelection>>,
) -> InputMethodController<'scope, PageCompositionCommands<'scope, 'listener, 'frame>> {
    let selection = Rc::new(PageCompositionSelection {
        document: document.clone(),
        editor,
        selections,
        interaction: interaction.clone(),
        frame,
        input_frame: dispatcher.input_frame.clone(),
    });
    InputMethodController::WithState(
        Rc::new(PageCompositionEvents {
            dispatcher,
            interaction,
        }),
        selection.clone(),
        Rc::new(PageCompositionCommands {
            document,
            dispatcher,
            selection,
            emit,
            ending: RefCell::new(None),
        }),
        retained,
    )
}

/// Native preedit notifications enter the source SetComposition transaction.
// Source: input_method_controller.cc:1009-1192.
pub(crate) fn DispatchPageCompositionEvent(
    document: InteractionDocument,
    dispatcher: &EventDispatcher<'_, '_>,
    editor: Rc<PageEditingOwners>,
    selections: Rc<SelectionState>,
    interaction: Rc<RefCell<UserInteractionState>>,
    emit: InteractionDOMMutationEmitter,
    retained: Rc<InputMethodControllerState>,
    frame: Option<Rc<PageFrameSelection>>,
    event: &CompositionEvent,
    geometry: &dyn PreparedCompositionGeometry,
) -> DispatchEventResult {
    let controller = WithController(
        document,
        dispatcher,
        editor,
        selections,
        interaction,
        emit,
        retained,
        frame,
    );
    let text = event.data.encode_utf16().collect::<Vec<_>>();
    match event.r#type {
        // SetComposition emits compositionstart with the original selected text
        // immediately before its first nonempty update, as Blink does.
        CompositionEventType::kStart => {}
        CompositionEventType::kUpdate => {
            let (start, end) = event
                .selection
                .unwrap_or((text.len(), text.len()));
            controller.SetComposition(
                &text,
                &event.spans,
                i32::try_from(start).unwrap_or(i32::MAX),
                i32::try_from(end).unwrap_or(i32::MAX),
                geometry,
            );
        }
        CompositionEventType::kEnd => {
            if text.is_empty() {
                controller
                    .FinishComposingText(ConfirmCompositionBehavior::kDoNotKeepSelection, geometry);
            } else {
                controller.CommitText(&text, &event.spans, 0, geometry);
            }
        }
    }
    DispatchEventResult {
        target_node_id: controller.selection.RootEditableNodeId(),
        ..Default::default()
    }
}

/// Native committed text shares the same retained composition owner as preedit.
// Source: input_method_controller.cc:694-717,892-964.
pub(crate) fn DispatchPageTextInputEvent(
    document: InteractionDocument,
    dispatcher: &EventDispatcher<'_, '_>,
    editor: Rc<PageEditingOwners>,
    selections: Rc<SelectionState>,
    interaction: Rc<RefCell<UserInteractionState>>,
    emit: InteractionDOMMutationEmitter,
    retained: Rc<InputMethodControllerState>,
    frame: Option<Rc<PageFrameSelection>>,
    event: &TextInputEvent,
    geometry: &dyn PreparedCompositionGeometry,
) -> DispatchEventResult {
    let controller = WithController(
        document,
        dispatcher,
        editor,
        selections,
        interaction,
        emit,
        retained,
        frame,
    );
    let committed = controller.CommitText(
        &event.text.encode_utf16().collect::<Vec<_>>(),
        &event.spans,
        0,
        geometry,
    );
    DispatchEventResult {
        target_node_id: controller.selection.RootEditableNodeId(),
        default_prevented: !committed,
        ..Default::default()
    }
}

/// Page supplies the lifecycle barrier at each annotated controller source line.
pub(crate) struct PageCompositionGeometry<'scope>(pub &'scope dyn Fn(u32));
impl PreparedCompositionGeometry for PageCompositionGeometry<'_> {
    fn PrepareAt(&self, source_line: u32) {
        (self.0)(source_line);
    }
}

/// Native command entry; the caller already owns the synchronous dispatcher.
/// Source editor_command.cc:1065-1120, sharing IME's existing insertion leaves.
pub(crate) fn ExecuteYank(
    dispatcher: &EventDispatcher<'_, '_>,
    document: InteractionDocument,
    editor: Rc<PageEditingOwners>,
    selections: Rc<SelectionState>,
    interaction: Rc<RefCell<UserInteractionState>>,
    emit: InteractionDOMMutationEmitter,
    frame: Rc<PageFrameSelection>,
    select_inserted: bool,
) -> Result<bool, interaction_default::core::editing::kill_ring::RequiredKillRingPlatform> {
    use interaction_default::core::editing::kill_ring::EditorYankContext;
    struct Context<'a, 'b, 'c>(PageCompositionCommands<'a, 'b, 'c>, u128);
    impl EditorYankContext for Context<'_, '_, '_> {
        fn DispatchBeforeInputInsertFromYank(&self, text: &foundation::String) -> bool {
            let Some(target) = self
                .0
                .dispatcher
                .KeyboardTargets()
                .EventTargetNodeForDocument(self.0.selection.interaction.borrow().focused_node_id)
            else {
                return true;
            };
            self.0.dispatcher.CompositionBeforeInput(
                &target,
                InputType::kInsertFromYank,
                text.Span16().unwrap_or_default(),
                false,
            ) == NativeResult::kNotCanceled
        }
        fn IsFrameAttached(&self) -> bool {
            self.0
                .document
                .borrow()
                .GetDocument()
                .RootHandle()
                .Identity()
                == self.1
                && self
                    .0
                    .selection
                    .frame
                    .as_ref()
                    .is_some_and(|frame| frame.IsAvailable())
        }
        fn UpdateStyleAndLayout(&self) {
            if let Some(frame) = &self.0.selection.frame {
                frame.PrepareEditing();
            }
        }
        fn InsertTextWithoutSendingTextEvent(&self, text: &foundation::String, select: bool) {
            self.0.InsertWithInputType(
                text.Span16().unwrap_or_default(),
                select,
                TextCompositionType::kTextCompositionNone,
                InputType::kInsertFromYank,
            );
        }
    }
    let identity = document.borrow().GetDocument().RootHandle().Identity();
    let state = editor.ime_typing.editor.clone();
    let selection = Rc::new(PageCompositionSelection {
        document: document.clone(),
        editor,
        selections,
        interaction,
        frame: Some(frame),
        input_frame: dispatcher.input_frame.clone(),
    });
    state.ExecuteYank(
        &Context(
            PageCompositionCommands {
                document,
                dispatcher,
                selection,
                emit,
                ending: RefCell::new(None),
            },
            identity,
        ),
        select_inserted,
    )
}

/// EnabledInEditableText at editor_command.cc:1202-1226. Called after the
/// command lifecycle barrier, using the current shared FrameSelection owner.
pub(crate) fn CanExecuteEditingCommand(dispatcher: &EventDispatcher<'_, '_>) -> bool {
    if dispatcher
        .input_method_state
        .as_ref()
        .is_some_and(|state| state.GetActiveEditContext().is_some())
    {
        return true;
    }
    dispatcher.FrameSelection().PrepareEditing();
    let selection = PageCompositionSelection {
        document: dispatcher.KeyboardTargets().DocumentOwner(),
        selections: dispatcher.EditingSelectionState(),
        editor: dispatcher.EditingOwners(),
        interaction: dispatcher.KeyboardState(),
        frame: Some(dispatcher.FrameSelection().clone()),
        input_frame: dispatcher.input_frame.clone(),
    };
    selection.CanEdit()
        && matches!(
            dispatcher
                .FrameSelection()
                .CompositionText(Query::SelectionFocus {
                    selection: dispatcher.FrameSelection().Current(),
                    focused: dispatcher.KeyboardState().borrow().focused_node_id
                }),
            Some(ResultValue::SelectionFocus(true))
        )
}

pub(crate) fn ExecuteTypingEditorCommand(
    dispatcher: &EventDispatcher<'_, '_>,
    command: interaction_default::core::editing::commands::editor_command_table::ExecuteFunction,
    value: &str,
) -> bool {
    use interaction_default::core::editing::commands::editor_command_table::ExecuteFunction as F;
    let text_event = match command {
        F::InsertCommands_ExecuteInsertTab | F::InsertCommands_ExecuteInsertBacktab => {
            Some(([9], false))
        }
        F::InsertCommands_ExecuteInsertLineBreak => Some(([10], true)),
        F::InsertCommands_ExecuteInsertNewline => Some(([10], !PasteCanEditRichly(dispatcher))),
        _ => None,
    };
    if let Some((data, line_break)) = text_event {
        let identity = dispatcher
            .KeyboardTargets()
            .DocumentOwner()
            .borrow()
            .GetDocument()
            .RootHandle()
            .Identity();
        match dispatcher.EditorCommandDispatchTextInput(&data, line_break) {
            None => return false,
            Some(result) if result != NativeResult::kNotCanceled => return true,
            Some(_) => {}
        }
        if !dispatcher.FrameSelection().IsAvailable()
            || dispatcher
                .KeyboardTargets()
                .DocumentOwner()
                .borrow()
                .GetDocument()
                .RootHandle()
                .Identity()
                != identity
        {
            return false;
        }
        dispatcher.FrameSelection().PrepareEditing();
    }
    let _events = dispatcher.EditorCommandEventQueueScope();
    let commands = PageCompositionCommands {
        document: dispatcher.KeyboardTargets().DocumentOwner(),
        dispatcher,
        selection: Rc::new(PageCompositionSelection {
            document: dispatcher.KeyboardTargets().DocumentOwner(),
            selections: dispatcher.EditingSelectionState(),
            interaction: dispatcher.KeyboardState(),
            editor: dispatcher.EditingOwners(),
            frame: Some(dispatcher.FrameSelection().clone()),
            input_frame: dispatcher.input_frame.clone(),
        }),
        emit: dispatcher.EditorCommandMutationEmitter(),
        ending: RefCell::new(None),
    };
    match command {
        F::ExecuteUndo | F::ExecuteRedo => {
            typing::ReplayHistory(&commands, command == F::ExecuteRedo);
            true
        }
        F::InsertCommands_ExecuteInsertText => {
            typing::InsertWithInputType(
                &commands,
                &value.encode_utf16().collect::<Vec<_>>(),
                false,
                TextCompositionType::kTextCompositionNone,
                InputType::kInsertText,
            );
            true
        }
        F::InsertCommands_ExecuteInsertLineBreak => typing::InsertNewline(&commands, true),
        F::InsertCommands_ExecuteInsertParagraph => typing::InsertNewline(&commands, false),
        F::InsertCommands_ExecuteInsertNewlineInQuotedContent => {
            typing::InsertQuotedNewline(&commands)
        }
        F::InsertCommands_ExecuteInsertNewline => typing::InsertNewline(
            &commands,
            text_event.is_some_and(|(_, line_break)| line_break),
        ),
        F::InsertCommands_ExecuteInsertTab | F::InsertCommands_ExecuteInsertBacktab => {
            typing::InsertWithInputType(
                &commands,
                &[9],
                false,
                TextCompositionType::kTextCompositionNone,
                InputType::kInsertText,
            );
            true
        }
        _ => false,
    }
}

fn EditorSelectionCommands<'scope, 'listener: 'scope, 'frame: 'scope>(
    dispatcher: &'scope EventDispatcher<'listener, 'frame>,
) -> PageCompositionCommands<'scope, 'listener, 'frame> {
    PageCompositionCommands {
        document: dispatcher.KeyboardTargets().DocumentOwner(),
        dispatcher,
        selection: Rc::new(PageCompositionSelection {
            document: dispatcher.KeyboardTargets().DocumentOwner(),
            selections: dispatcher.EditingSelectionState(),
            interaction: dispatcher.KeyboardState(),
            editor: dispatcher.EditingOwners(),
            frame: Some(dispatcher.FrameSelection().clone()),
            input_frame: dispatcher.input_frame.clone(),
        }),
        emit: dispatcher.EditorCommandMutationEmitter(),
        ending: RefCell::new(None),
    }
}
pub(crate) fn ReplaceEditorSelectionText(
    dispatcher: &EventDispatcher<'_, '_>,
    text: &[u16],
    input_type: InputType,
) -> Result<(), &'static str> {
    let fragment = dispatcher.CreatePasteTextFragment(text)?;
    let commands = EditorSelectionCommands(dispatcher);
    let _events = dispatcher.EditorCommandEventQueueScope();
    typing::ReplaceFragmentWithOptions(&commands,fragment.root,interaction_default::core::editing::commands::replace_selection_command::ReplaceSelectionOptions {prevent_nesting:true,match_style:true,..Default::default()},input_type,None)
}
pub(crate) fn DeleteEditorSelection(
    dispatcher: &EventDispatcher<'_, '_>,
) -> Result<
    (),
    interaction_default::core::editing::commands::paragraph_editing_context::EditingAborted,
> {
    let commands = EditorSelectionCommands(dispatcher);
    let Some(frame) = commands.selection.frame.as_ref() else {
        return Ok(());
    };
    if !commands.selection.CanEdit()
        || !frame
            .CreateVisibleSelection(frame.Current())
            .is_some_and(|s| s.anchor != s.focus)
    {
        return Ok(());
    }
    let _events = dispatcher.EditorCommandEventQueueScope();
    typing::DeleteRange(&commands, false, true)?;
    commands
        .selection
        .editor
        .ime_typing
        .editor
        .SetStartNewKillRingSequence(false);
    Ok(())
}

pub(crate) struct DragCommandOwners {
    pub document: InteractionDocument,
    pub editor: Rc<PageEditingOwners>,
    pub selections: Rc<SelectionState>,
    pub interaction: Rc<RefCell<UserInteractionState>>,
    pub emit: InteractionDOMMutationEmitter,
    pub frame: Rc<PageFrameSelection>,
}

pub(crate) fn ExecuteStyleEditorCommand(
    dispatcher: &EventDispatcher<'_, '_>,
    command: interaction_default::core::editing::commands::editor_command_table::ExecuteFunction,
    value: &str,
) -> Result<bool, &'static str> {
    let commands = PageCompositionCommands {
        document: dispatcher.KeyboardTargets().DocumentOwner(),
        dispatcher,
        selection: Rc::new(PageCompositionSelection {
            document: dispatcher.KeyboardTargets().DocumentOwner(),
            selections: dispatcher.EditingSelectionState(),
            interaction: dispatcher.KeyboardState(),
            editor: dispatcher.EditingOwners(),
            frame: Some(dispatcher.FrameSelection().clone()),
            input_frame: dispatcher.input_frame.clone(),
        }),
        emit: dispatcher.EditorCommandMutationEmitter(),
        ending: RefCell::new(None),
    };
    typing::ExecuteStyle(&commands, command, value)
}
fn DragCommands<'scope, 'listener, 'frame>(
    dispatcher: &'scope EventDispatcher<'listener, 'frame>,
) -> PageCompositionCommands<'scope, 'listener, 'frame> {
    let owners = dispatcher.DragCommandOwners();
    PageCompositionCommands {
        document: owners.document.clone(),
        dispatcher,
        selection: Rc::new(PageCompositionSelection {
            document: owners.document,
            editor: owners.editor,
            selections: owners.selections,
            interaction: owners.interaction,
            frame: Some(owners.frame),
            input_frame: dispatcher.input_frame.clone(),
        }),
        emit: owners.emit,
        ending: RefCell::new(None),
    }
}
pub(crate) fn ApplyCommandFragment(
    dispatcher: &EventDispatcher<'_, '_>,
    fragment: &super::page_selection_markup::RetainedFragment,
) -> Result<bool, &'static str> {
    if !fragment.IsCurrent() {
        return Ok(false);
    }
    let _events = dispatcher.EditorCommandEventQueueScope();
    typing::ReplaceFragmentWithOptions(&DragCommands(dispatcher), fragment.root,
        interaction_default::core::editing::commands::replace_selection_command::ReplaceSelectionOptions {
            prevent_nesting: true, ..Default::default()
        }, InputType::kNone, None)?;
    Ok(true)
}
pub(crate) fn ExecuteLinkFormatEditorCommand(
    dispatcher: &EventDispatcher<'_, '_>,
    action: interaction_default::core::editing::commands::editor_command_table::ExecuteFunction,
    value: &str,
    input_type: InputType,
) -> Result<bool, &'static str> {
    let _events = dispatcher.EditorCommandEventQueueScope();
    typing::LinkFormat(&DragCommands(dispatcher), action, value, input_type)
}
pub(crate) fn ApplyDragReplacement(
    dispatcher: &EventDispatcher<'_, '_>,
    fragment: &super::page_selection_markup::RetainedFragment,
    smart: bool,
    plain: bool,
    transfer: Rc<interaction_dispatch::core::clipboard::data_transfer::DataTransfer>,
) -> Result<(), &'static str> {
    if !fragment.IsCurrent() {
        return Ok(());
    }
    typing::ReplaceFragmentWithOptions(&DragCommands(dispatcher),fragment.root,
        interaction_default::core::editing::commands::replace_selection_command::ReplaceSelectionOptions {
            select_replacement:true,prevent_nesting:true,smart_replace:smart,match_style:plain,..Default::default()
        },InputType::kInsertFromDrop,Some(transfer))
}
pub(crate) fn DeleteDraggedSelection(
    dispatcher: &EventDispatcher<'_, '_>,
    smart: bool,
) -> Result<(), &'static str> {
    typing::DeleteDraggedSelection(&DragCommands(dispatcher), smart)
}
pub(crate) fn DeleteCutSelection(
    dispatcher: &EventDispatcher<'_, '_>,
    smart: bool,
) -> Result<(), &'static str> {
    typing::DeleteSelectionWithInputType(&DragCommands(dispatcher), smart, InputType::kDeleteByCut)
}
pub(crate) fn ExecuteListIndentEditorCommand(
    dispatcher: &EventDispatcher<'_, '_>,
    command: interaction_default::core::editing::commands::editor_command_table::EditingCommandType,
    input_type: InputType,
) -> Result<
    bool,
    interaction_default::core::editing::commands::paragraph_editing_context::EditingAborted,
> {
    let _events = dispatcher.EditorCommandEventQueueScope();
    typing::ListIndent(&DragCommands(dispatcher), command, input_type)
}
pub(crate) fn BeginDragCommandGroup(dispatcher: &EventDispatcher<'_, '_>) {
    let Some(selection) = dispatcher.FrameSelection().Current() else {
        return;
    };
    let selection = typing::ImeTypingSelection(selection);
    let group=interaction_default::core::editing::commands::composite_edit_command::CompositeEditCommand::new(selection.clone(),selection);
    group.SetCommandKind(interaction_default::core::editing::commands::composite_edit_command::CompositeCommandKind::DragAndDrop);
    dispatcher
        .DragCommandOwners()
        .editor
        .ime_typing
        .editor
        .SetLastEditCommand(Some(group));
}

/// editor_command.cc:478-539; EditorCommand::Execute at :2096-2198.
/// The outer command owner dispatches beforeinput; this function re-reads the
/// selection after reentry and enters the retained TypingCommand exactly once.
pub(crate) fn ExecuteDeleteWithDirection(
    dispatcher: &EventDispatcher<'_, '_>,
    document: InteractionDocument,
    editor: Rc<PageEditingOwners>,
    selections: Rc<SelectionState>,
    interaction: Rc<RefCell<UserInteractionState>>,
    emit: InteractionDOMMutationEmitter,
    frame: Rc<PageFrameSelection>,
    granularity: interaction_default::core::editing::text_granularity::TextGranularity,
    forward: bool,
    kill_ring: bool,
    is_typing_action: bool,
    range_only: bool,
) -> Result<
    bool,
    interaction_default::core::editing::commands::paragraph_editing_context::EditingAborted,
> {
    use interaction_default::core::editing::frame_selection::{
        RevealExtentOption, SelectionAlignment,
    };
    let retained = editor.ime_typing.editor.clone();
    let selection = Rc::new(PageCompositionSelection {
        document: document.clone(),
        editor,
        selections,
        interaction,
        frame: Some(frame.clone()),
        input_frame: dispatcher.input_frame.clone(),
    });
    if !selection.CanEdit() {
        return Ok(true);
    }
    let commands = PageCompositionCommands {
        document,
        dispatcher,
        selection,
        emit,
        ending: RefCell::new(None),
    };
    let is_range = frame
        .CreateVisibleSelection(frame.Current())
        .is_some_and(|selection| selection.anchor != selection.focus);
    if range_only && !is_range {
        return Ok(true);
    }
    if is_range && !is_typing_action {
        typing::DeleteRange(&commands, forward, kill_ring)?;
    } else {
        typing::Delete(&commands, granularity, forward, kill_ring)?;
    }
    if !retained.PreventRevealSelection() && frame.IsAvailable() {
        frame.RevealSelection(
            SelectionAlignment::EdgeIfNeeded,
            RevealExtentOption::kDoNotRevealExtent,
        );
    }
    // Selection publication sets this flag; consecutive kill commands append
    // to the same native sequence, including edits that changed the selection.
    if kill_ring {
        retained.SetStartNewKillRingSequence(false);
    }
    Ok(true)
}
