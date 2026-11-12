//! DragController's live DOM/FrameSelection/Editor leaves. Source policy is in
//! default::DragController; every layout/DOM loan ends before listener entry.
use super::*;
use crate::adapter::page_editing_visible_position::{
    CompositionTextQuery as Q, CompositionTextResult as V, MouseSelectionGeometryQuery as MQ,
    MouseSelectionGeometryResult as MV,
};
use interaction_default::core::page::{
    drag_controller::{DragCaretPosition, DragEditTarget, DragFragment, DragSelectionFacts},
    drag_data::DragData,
};
use interaction_dispatch::core::{
    dom::events::event::EventInterface,
    events::{
        input_event::{EventIsComposing, InputEvent as NativeInputEvent, InputType},
        text_event::{TextEvent, TextEventInputType},
    },
};
use layoutng_assembly::{editing_state::DOMSelection, EditingPosition, EditingTextAffinity};
struct DropCaret {
    document: InteractionDocument,
    range: dom::DOMRangeHandle,
    affinity: EditingTextAffinity,
    original_anchor: u64,
}
impl DropCaret {
    fn Selection(&self) -> Option<DOMSelection> {
        let owner = self.document.borrow();
        let d = owner.GetDocument();
        let range = d.Range(&self.range)?;
        Some(DOMSelection {
            anchor: EditingPosition::new(range.start.node_id, range.start.offset),
            focus: EditingPosition::new(range.end.node_id, range.end.offset),
            affinity: self.affinity,
        })
    }
    fn Node(&self) -> Option<u64> {
        self.Selection()?.anchor.node_id
    }
}
impl DragCaretPosition for DropCaret {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn IsConnected(&self) -> bool {
        let owner = self.document.borrow();
        let d = owner.GetDocument();
        d.IsActive()
            && d.FindNodeById(self.original_anchor)
                .is_some_and(|node| d.IsConnectedToDocument(node))
            && d.Range(&self.range).is_some_and(|range| {
                [range.start, range.end].iter().all(|p| {
                    d.FindNodeById(p.node_id)
                        .is_some_and(|n| d.IsConnectedToDocument(n))
                })
            })
    }
    fn IsRichlyEditable(&self) -> bool {
        let Some(node) = self.Node() else {
            return false;
        };
        let owner = self.document.borrow();
        foundation::editable::IsRichlyEditable(
            &dom::persistent_document::DOMEditableFacts(owner.GetDocument()),
            node,
            Default::default(),
        )
    }
}
impl DragFragment for crate::adapter::page_selection_markup::RetainedFragment {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
}
impl PageDragTarget<'_, '_, '_, '_> {
    pub(super) fn HasDocumentAt(&self, event: &WebMouseEvent) -> bool {
        self.source
            .Hit(event.PositionInRootFrame())
            .and_then(|hit| hit.InnerNode().and_then(|n| n.DOMNodeId()))
            .is_some()
    }
    pub(super) fn DocumentIsEditable(&self) -> bool {
        let owner = self.source.pointer.dispatcher.document.borrow();
        let d = owner.GetDocument();
        d.IsEditableForEditing(d.Root())
    }
    pub(super) fn EditTarget(
        &self,
        event: &WebMouseEvent,
        data: &DragData,
        did_initiate: bool,
    ) -> Result<Option<DragEditTarget>, &'static str> {
        if !data.ContainsCompatibleContent() {
            return Ok(None);
        }
        self.source
            .pointer
            .geometry
            .PrepareLifecycleExceptPaint()
            .map_err(|_| "DragController target lifecycle")?;
        let Some(hit) = self.source.Hit(event.PositionInRootFrame()) else {
            return Ok(None);
        };
        let Some(id) = hit.InnerNode().and_then(|node| node.DOMNodeId()) else {
            return Ok(None);
        };
        let file = {
            let owner = self.source.pointer.dispatcher.document.borrow();
            let d = owner.GetDocument();
            let mut node = d.FindNodeById(id);
            let mut file = None;
            while let Some(i) = node {
                if d.InputType(i) == Some(dom::persistent_document::DOMInputType::kFile) {
                    file = Some((
                        d.Node(i).Id(),
                        d.IsDisabledFormControl(i),
                        d.Node(i).FindAttribute("multiple").is_some(),
                    ));
                    break;
                }
                node = d.OwnerShadowHost(i);
            }
            file
        };
        if data.ContainsFiles() {
            if let Some((id, disabled, multiple)) = file {
                if let Some(element) = self.source.pointer.dispatcher.targets.ResolveTarget(id) {
                    return Ok(Some(DragEditTarget::FileInput {
                        element,
                        disabled,
                        multiple,
                    }));
                }
            }
        }
        let editable = {
            let owner = self.source.pointer.dispatcher.document.borrow();
            let d = owner.GetDocument();
            d.FindNodeById(id)
                .is_some_and(|node| d.IsEditableForEditing(node))
        };
        if !editable {
            return Ok(None);
        }
        if did_initiate
            && self.SelectionFacts().same_document
            && self.source.SelectionContains(event.PositionInRootFrame())
        {
            return Ok(None);
        }
        let p = event.PositionInRootFrame();
        let frame = &self.source.pointer.dispatcher.focus_services.selection;
        let Some(selection) =
            frame.CaretSelectionForViewportPoint((p.x().round() as i32, p.y().round() as i32))
        else {
            return Ok(None);
        };
        let Some(V::Range(range)) = frame.CompositionText(Q::SelectionRange(selection)) else {
            return Err("DragCaret native boundary range");
        };
        let document = self.source.pointer.dispatcher.document.clone();
        let range = document
            .borrow_mut()
            .GetDocumentMut()
            .CreateRange(range.start, range.end)
            .map_err(|_| "DragCaret native range")?;
        let caret = Rc::new(DropCaret {
            document,
            range,
            affinity: selection.affinity,
            original_anchor: selection.anchor.node_id.ok_or("DragCaret missing anchor")?,
        });
        let Some(element) = self
            .source
            .pointer
            .dispatcher
            .targets
            .ResolveTarget(hit.InnerElement().and_then(|n| n.DOMNodeId()).unwrap_or(id))
        else {
            return Ok(None);
        };
        Ok(Some(DragEditTarget::Editable { element, caret }))
    }
    pub(super) fn SetCanReceiveDroppedFiles(&self, target: &EventTargetRef, value: bool) {
        let Some(id) = self.source.SourceId(target) else {
            return;
        };
        let changed = {
            let mut owner = self.source.pointer.dispatcher.document.borrow_mut();
            let d = owner.GetDocumentMut();
            d.FindNodeById(id)
                .is_some_and(|node| d.SetFileInputCanReceiveDroppedFiles(node, value))
        };
        if changed {
            self.source.pointer.frame.DragTargetStateChanged();
        }
    }
    pub(super) fn SelectionFacts(&self) -> DragSelectionFacts {
        let p = self.source.pointer;
        let frame = &p.dispatcher.focus_services.selection;
        frame.PrepareEditing();
        let selection = frame.Current();
        let editable = frame.VisibleSelectionRootEditableElement().is_some();
        let has_focus = if let Some(settings) = p.geometry.InteractionTextSettings() {
            matches!(
                frame.CompositionText(Q::MouseSelectionGeometry {
                    query: MQ::SelectionFocus {
                        selection,
                        focused: p.dispatcher.state.borrow().focused_node_id
                    },
                    ui_locale: settings.ui_locale
                }),
                Some(V::MouseSelectionGeometry(MV::Boolean(true)))
            )
        } else {
            false
        };
        let source = self.source.DragSource();
        let same_document = source.as_ref().is_some_and(|source| {
            p.dispatcher.targets.OwnerDocumentHandle(source)
                == Some(p.dispatcher.document.borrow().GetDocument().RootHandle())
        });
        DragSelectionFacts {
            same_document,
            has_focus,
            editable,
            range: selection.is_some_and(|s| s.anchor != s.focus),
            word_granularity: frame.SelectionControllerFlags().0
                == interaction_default::core::editing::text_granularity::TextGranularity::kWord,
        }
    }
    pub(super) fn DispatchDropTextInput(
        &self,
        caret: &dyn DragCaretPosition,
        data: &DragData,
    ) -> bool {
        let Some(caret) = caret.AsAny().downcast_ref::<DropCaret>() else {
            return false;
        };
        let Some(node) = caret.Node() else {
            return false;
        };
        let target = {
            let owner = caret.document.borrow();
            let d = owner.GetDocument();
            let mut node = d.FindNodeById(node);
            let mut target = None;
            while let Some(i) = node {
                if d.Node(i).Type() == dom::DOMNodeType::kElement {
                    target = Some(d.Node(i).Id());
                    break;
                }
                node = d.Node(i).Parent().or_else(|| d.OwnerShadowHost(i));
            }
            target
        };
        let Some(target) =
            target.and_then(|id| self.source.pointer.dispatcher.targets.ResolveTarget(id))
        else {
            return false;
        };
        let text = if caret.IsRichlyEditable() {
            Vec::new()
        } else {
            data.PlainText()
        };
        let mut event = TextEvent::new(
            Some(self.source.pointer.dispatcher.targets.WindowView()),
            &text,
            TextEventInputType::Drop,
            self.source.pointer.dispatcher.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        self.source
            .pointer
            .DispatchCanonical(&target, EventRef::new(event))
            != NativeDispatchEventResult::kNotCanceled
    }
    pub(super) fn ReceiveDroppedFiles(
        &self,
        target: &EventTargetRef,
        data: &DragData,
    ) -> Result<bool, &'static str> {
        let Some(id) = self.source.SourceId(target) else {
            return Ok(false);
        };
        interaction_default::core::html::forms::file_input_type::ReceiveDroppedFiles(
            &DropFiles {
                dispatcher: self.source.pointer.dispatcher,
                node: id,
            },
            &data.Files(),
        )
    }
    pub(super) fn CreateDropFragment(
        &self,
        data: &DragData,
        caret: &dyn DragCaretPosition,
        move_selection: bool,
    ) -> Result<Option<(Rc<dyn DragFragment>, bool)>, &'static str> {
        let caret = caret
            .AsAny()
            .downcast_ref::<DropCaret>()
            .ok_or("DragCaret owner mismatch")?;
        let dispatcher = self.source.pointer.dispatcher;
        let frame = &dispatcher.focus_services.selection;
        let rich = caret.IsRichlyEditable();
        if rich || move_selection {
            if let Some((html, base)) = data.HTML() {
                let target = dispatcher.targets.DocumentOwner();
                if let Some(fragment)=crate::adapter::page_markup_sanitization::CreateStrictlyProcessedFragmentFromMarkupWithContext(frame.MarkupSanitizationHost().as_ref(),&target,&html,0,html.len(),&base).map_err(|_|"DragData HTML sanitization")? {return Ok(Some((Rc::new(fragment),false)));}
            }
            if rich && data.Contains("text/uri-list") {
                if let Some(url) = data.URLs().first() {
                    let title = data.URLTitle();
                    let label = if title.is_empty() {
                        let plain = data.PlainText();
                        if plain.is_empty() {
                            url.encode_utf16().collect()
                        } else {
                            plain
                        }
                    } else {
                        title.encode_utf16().collect()
                    };
                    let mut owner = dispatcher.document.borrow_mut();
                    let d = owner.GetDocumentMut();
                    let identity = d.RootHandle().Identity();
                    let root = d.CreateDocumentFragment();
                    let anchor = d.CreateElement(
                        dom::DOMNamespace::kHTML,
                        "a".into(),
                        vec![dom::persistent_document::DOMAttribute {
                            local_name: "href".into(),
                            value: url.clone(),
                            ..Default::default()
                        }],
                    );
                    let text = d.CreateTextUTF16(&label);
                    d.AppendChild(anchor, text);
                    d.AppendChild(root, anchor);
                    let fragment = crate::adapter::page_selection_markup::RetainedFragment {
                        root: d.Node(root).Id(),
                        document: dispatcher.document.clone(),
                        identity,
                    };
                    return Ok(Some((Rc::new(fragment), false)));
                }
            }
        }
        let text = data.PlainText();
        if text.is_empty() {
            return Ok(None);
        }
        let selection = caret
            .Selection()
            .ok_or("DragCaret disconnected selection")?;
        let Some(V::FragmentFromTextContext(mut context)) =
            frame.CompositionText(Q::FragmentFromTextContext(selection))
        else {
            return Err("DragData text fragment context");
        };
        context.default_paragraph_separator = dispatcher
            .editor
            .ime_typing
            .editor
            .DefaultParagraphSeparator();
        let fragment = crate::adapter::page_selection_markup::CreateFragmentFromText(
            &dispatcher.document,
            &dispatcher.emit,
            &context,
            &text,
        )
        .map_err(|_| "DragData text fragment")?;
        Ok(Some((Rc::new(fragment), true)))
    }
    pub(super) fn BeginDragAndDropCommandGroup(&self) {
        crate::adapter::page_input_method_controller::BeginDragCommandGroup(
            self.source.pointer.dispatcher,
        );
    }
    pub(super) fn DeleteSelectionAfterDraggingWithEvents(
        &self,
        _caret: &dyn DragCaretPosition,
        smart: bool,
    ) -> Result<bool, &'static str> {
        let p = self.source.pointer;
        let frame = &p.dispatcher.focus_services.selection;
        let Some(target) =
            crate::adapter::page_input_method_controller::PasteEventTarget(p.dispatcher, false)
        else {
            return Ok(true);
        };
        let allowed =
            p.dispatcher
                .CompositionBeforeInput(&target, InputType::kDeleteByDrag, &[], false)
                == NativeDispatchEventResult::kNotCanceled;
        if !frame.IsAvailable() {
            return Ok(false);
        }
        if p.dispatcher
            .input_method_state
            .as_ref()
            .is_some_and(|s| s.GetActiveEditContext().is_some())
        {
            return Ok(true);
        }
        if allowed {
            crate::adapter::page_input_method_controller::DeleteDraggedSelection(
                p.dispatcher,
                smart,
            )?;
        }
        Ok(true)
    }
    pub(super) fn SetSelectionToDragCaret(&self, caret: &dyn DragCaretPosition) -> bool {
        let Some(caret) = caret.AsAny().downcast_ref::<DropCaret>() else {
            return false;
        };
        let Some(selection) = caret.Selection() else {
            return false;
        };
        let connected = {
            let owner = caret.document.borrow();
            let d = owner.GetDocument();
            [selection.anchor, selection.focus].iter().all(|p| {
                p.node_id
                    .and_then(|id| d.FindNodeById(id))
                    .is_some_and(|node| d.IsConnectedToDocument(node))
            })
        };
        if !connected {
            return false;
        }
        let frame = &self.source.pointer.dispatcher.focus_services.selection;
        frame.SetSelection(caret.Selection(), Default::default());
        frame.PrepareEditing();
        frame.VisibleSelectionRootEditableElement().is_some()
    }
    pub(super) fn ReplaceSelectionAfterDraggingWithEvents(
        &self,
        target: &EventTargetRef,
        data: &DragData,
        fragment: &dyn DragFragment,
        smart: bool,
        plain: bool,
    ) -> Result<bool, &'static str> {
        let p = self.source.pointer;
        let frame = &p.dispatcher.focus_services.selection;
        let connected = p.dispatcher.targets.NodeId(target).is_some_and(|id| {
            let owner = p.dispatcher.document.borrow();
            let d = owner.GetDocument();
            d.FindNodeById(id)
                .is_some_and(|node| d.IsConnectedToDocument(node))
        });
        if !connected {
            return Ok(true);
        }
        let transfer=DataTransfer::CreateDragTarget(data.data.clone(),interaction_dispatch::core::clipboard::data_transfer::DataTransferAccessPolicy::Readable,data.source_operations);
        let mut event = NativeInputEvent::CreateBeforeInputWithDataTransfer(
            InputType::kInsertFromDrop,
            Some(transfer.clone()),
            EventIsComposing::kNotComposing,
            None,
            p.dispatcher.clock.as_ref(),
        );
        event.BaseEventMut().SetTrusted(true);
        let insert = p.DispatchCanonical(target, EventRef::new(event))
            == NativeDispatchEventResult::kNotCanceled;
        if !frame.IsAvailable() {
            return Ok(false);
        }
        if p.dispatcher
            .input_method_state
            .as_ref()
            .is_some_and(|s| s.GetActiveEditContext().is_some())
        {
            return Ok(true);
        }
        if insert {
            let fragment = fragment
                .AsAny()
                .downcast_ref::<crate::adapter::page_selection_markup::RetainedFragment>()
                .ok_or("DragData fragment owner mismatch")?;
            crate::adapter::page_input_method_controller::ApplyDragReplacement(
                p.dispatcher,
                fragment,
                smart,
                plain,
                transfer,
            )?;
        }
        Ok(true)
    }
    pub(super) fn RootEditableElement(&self) -> Option<EventTargetRef> {
        self.source
            .pointer
            .dispatcher
            .focus_services
            .selection
            .VisibleSelectionRootEditableElement()
            .and_then(|id| self.source.pointer.dispatcher.targets.ResolveTarget(id))
    }
    pub(super) fn UpdateDragStateAfterEdit(&self, root: Option<&EventTargetRef>) {
        let Some(root) = root else {
            return;
        };
        let source = self.source.DragSource();
        if source.as_ref().is_some_and(|source| {
            !self
                .source
                .pointer
                .Read(source, |d, i| d.IsConnectedToDocument(i))
                .unwrap_or(false)
        }) {
            self.source.SetDragSource(Some(root));
        }
    }
}
struct DropFiles<'a, 'listener, 'frame> {
    dispatcher: &'a EventDispatcher<'listener, 'frame>,
    node: u64,
}
impl interaction_default::core::html::forms::file_input_type::FileInputDropOperations
    for DropFiles<'_, '_, '_>
{
    fn Multiple(&self) -> bool {
        crate::ownership::HasAttribute(&self.dispatcher.document, self.node, "multiple")
    }
    fn Directory(&self) -> bool {
        crate::ownership::HasAttribute(&self.dispatcher.document, self.node, "webkitdirectory")
    }
    fn CurrentFiles(&self) -> Rc<interaction_dispatch::core::fileapi::file_list::FileList> {
        crate::ownership::ReadNode(&self.dispatcher.document, self.node, |d, _, i| {
            d.InputFiles(i)
        })
        .flatten()
        .unwrap_or_default()
    }
    fn SetFiles(&self, files: Rc<interaction_dispatch::core::fileapi::file_list::FileList>) {
        (self.dispatcher.emit)(&dom::dom_mutation::DOMMutation {
            mutation_type: dom::dom_mutation::DOMMutationType::kSetInputFiles,
            target_node_id: self.node,
            file_list: Some(dom::dom_mutation::DOMFileListValue(files)),
            ..Default::default()
        });
    }
    fn DispatchInput(&self) {
        let mut event =
            crate::event::MakeSyntheticEvent(crate::event::EventType::kInput, self.node);
        self.dispatcher.Dispatch(&mut event, self.node);
    }
    fn DispatchChange(&self) {
        let mut event =
            crate::event::MakeSyntheticEvent(crate::event::EventType::kChange, self.node);
        self.dispatcher.Dispatch(&mut event, self.node);
    }
    fn DispatchCancel(&self) {
        let mut event =
            crate::event::MakeSyntheticEvent(crate::event::EventType::kCustom, self.node);
        event.custom_type = "cancel".into();
        event.bubbles = true;
        self.dispatcher.Dispatch(&mut event, self.node);
    }
    fn EnumerateDroppedDirectory(
        &self,
        directory: std::sync::Arc<interaction_dispatch::core::fileapi::file::File>,
    ) -> Result<(), &'static str> {
        self.dispatcher
            .mouse_frame_geometry
            .borrow()
            .clone()
            .as_ref()
            .filter(|frame| frame.EnumerateDroppedDirectory(self.node, directory.clone()))
            .map(|_| ())
            .ok_or("FileInputType host directory enumeration unavailable")
    }
}

/// FileChooser completion enters through the current dispatcher. A disconnected,
/// disabled or repurposed control cannot receive an obsolete chooser result.
pub(crate) fn DirectoryFilesChosen(
    dispatcher: &EventDispatcher<'_, '_>,
    node: u64,
    result: Result<Vec<std::sync::Arc<fileapi::File>>, fileapi::NativeFileError>,
) {
    use interaction_default::core::html::forms::file_input_type::{
        FileInputDropOperations, SetFilesAndDispatchEvents,
    };
    let input = DropFiles { dispatcher, node };
    let valid = crate::ownership::ReadNode(&dispatcher.document, node, |d, _, index| {
        d.IsActive()
            && d.IsConnectedToDocument(index)
            && d.InputType(index) == Some(dom::persistent_document::DOMInputType::kFile)
            && !d.IsDisabledFormControl(index)
    })
    .unwrap_or(false)
        && input.Directory();
    if !valid {
        return;
    }
    match result {
        Ok(files) => {
            let selected = Rc::new(fileapi::FileList::default());
            for file in files {
                selected.Append(file);
            }
            SetFilesAndDispatchEvents(&input, selected);
        }
        Err(_) => input.DispatchCancel(),
    }
}

pub(crate) fn CurrentDragCaret(owners: &ResidentCursorOwners) -> Option<DOMSelection> {
    let caret = owners.drag.DragCaret()?;
    if !caret.IsConnected() {
        return None;
    }
    caret.AsAny().downcast_ref::<DropCaret>()?.Selection()
}
