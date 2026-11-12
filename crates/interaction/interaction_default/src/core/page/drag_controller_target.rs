//! DragController target branches, drag_controller.cc:254-875. Page supplies
//! only live DOM/FrameSelection/Editor leaves; ordering and policy stay here.
use super::super::drag_data::DragData;
use super::*;
#[derive(Clone)]
pub enum DragTargetAction {
    Enter {
        data: Arc<WebDragData>,
        source_operations: u32,
    },
    Update {
        source_operations: u32,
    },
    Leave,
    Drop {
        operation: DragOperation,
    },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DragTargetOperation {
    pub operation: DragOperation,
    pub document_is_handling_drag: bool,
}
/// The same retained PositionWithAffinity/DOM range owner survives dispatch.
/// The Page adapter may project its own position, never another document tree.
pub trait DragCaretPosition: std::any::Any {
    fn AsAny(&self) -> &dyn std::any::Any;
    fn IsConnected(&self) -> bool;
    fn IsRichlyEditable(&self) -> bool;
}
pub trait DragFragment: std::any::Any {
    fn AsAny(&self) -> &dyn std::any::Any;
}
pub enum DragEditTarget {
    FileInput {
        element: EventTargetRef,
        disabled: bool,
        multiple: bool,
    },
    Editable {
        element: EventTargetRef,
        caret: Rc<dyn DragCaretPosition>,
    },
}
pub struct DragSelectionFacts {
    pub same_document: bool,
    pub has_focus: bool,
    pub editable: bool,
    pub range: bool,
    pub word_granularity: bool,
}
pub trait DragTargetContext {
    fn HasView(&self) -> bool;
    fn HasDocumentAt(&self, event: &WebMouseEvent) -> bool;
    fn DocumentIsEditable(&self) -> bool;
    fn NotifyDropUserActivation(&self);
    fn UpdateDragAndDrop(&self, event: &WebMouseEvent, data: &Rc<DataTransfer>) -> bool;
    fn CancelDragAndDrop(&self, event: &WebMouseEvent, data: &Rc<DataTransfer>);
    fn PerformDragAndDrop(&self, event: &WebMouseEvent, data: &Rc<DataTransfer>) -> bool;
    fn EditTarget(
        &self,
        event: &WebMouseEvent,
        data: &DragData,
        did_initiate: bool,
    ) -> Result<Option<DragEditTarget>, &'static str>;
    fn SetCanReceiveDroppedFiles(&self, target: &EventTargetRef, value: bool);
    fn SelectionFacts(&self) -> DragSelectionFacts;
    fn SmartInsertDeleteEnabled(&self) -> bool;
    fn DispatchDropTextInput(&self, caret: &dyn DragCaretPosition, data: &DragData) -> bool;
    fn ReceiveDroppedFiles(
        &self,
        target: &EventTargetRef,
        data: &DragData,
    ) -> Result<bool, &'static str>;
    fn CreateDropFragment(
        &self,
        data: &DragData,
        caret: &dyn DragCaretPosition,
        move_selection: bool,
    ) -> Result<Option<(Rc<dyn DragFragment>, bool)>, &'static str>;
    fn BeginDragAndDropCommandGroup(&self);
    fn DeleteSelectionAfterDraggingWithEvents(
        &self,
        caret: &dyn DragCaretPosition,
        smart: bool,
    ) -> Result<bool, &'static str>;
    fn SetSelectionToDragCaret(&self, caret: &dyn DragCaretPosition) -> bool;
    fn ReplaceSelectionAfterDraggingWithEvents(
        &self,
        target: &EventTargetRef,
        data: &DragData,
        fragment: &dyn DragFragment,
        smart: bool,
        plain_text: bool,
    ) -> Result<bool, &'static str>;
    fn RootEditableElement(&self) -> Option<EventTargetRef>;
    fn UpdateDragStateAfterEdit(&self, root: Option<&EventTargetRef>);
    fn AcceptsLoadDrops(&self) -> bool;
    fn LoadDroppedURL(&self, url: &str, foreground: bool);
}
struct TargetTransfer(Rc<DataTransfer>);
impl Drop for TargetTransfer {
    fn drop(&mut self) {
        self.0.SetAccessPolicyNumb();
    }
}
impl DragController {
    pub fn DragCaret(&self) -> Option<Rc<dyn DragCaretPosition>> {
        self.drag_caret.borrow().clone()
    }

    fn DragIsMove(&self, context: &dyn DragTargetContext, event: &WebMouseEvent) -> bool {
        let f = context.SelectionFacts();
        #[cfg(target_os = "macos")]
        let copy =
            interaction_input::public::common::input::web_input_event::WebInputEvent::kAltKey;
        #[cfg(not(target_os = "macos"))]
        let copy =
            interaction_input::public::common::input::web_input_event::WebInputEvent::kControlKey;
        self.DidInitiateDrag()
            && f.same_document
            && f.has_focus
            && f.editable
            && f.range
            && event.GetModifiers() & copy == 0
    }
    fn ClearDropTarget(&self, context: &dyn DragTargetContext) {
        let previous = self.file_input_under_mouse.borrow_mut().take();
        if let Some(previous) = previous {
            context.SetCanReceiveDroppedFiles(&previous, false);
        }
        self.drag_caret.borrow_mut().take();
    }
    fn ConcludeEditDrag(
        &self,
        context: &dyn DragTargetContext,
        event: &WebMouseEvent,
        data: &DragData,
    ) -> Result<bool, &'static str> {
        let file = self.file_input_under_mouse.borrow_mut().take();
        if let Some(file) = &file {
            context.SetCanReceiveDroppedFiles(file, false);
        }
        if !context.HasDocumentAt(event) {
            return Ok(false);
        }
        let mut target = context.EditTarget(event, data, self.DidInitiateDrag())?;
        let caret = self.drag_caret.borrow().clone();
        if let Some(caret) = &caret {
            if context.DispatchDropTextInput(caret.as_ref(), data) {
                return Ok(true);
            }
        }
        if caret.is_some() {
            target = context.EditTarget(event, data, self.DidInitiateDrag())?;
        }
        if data.ContainsFiles() {
            if let Some(file) = file {
                return match target {
                    Some(DragEditTarget::FileInput {
                        disabled: false, ..
                    }) => context.ReceiveDroppedFiles(&file, data),
                    _ => Ok(false),
                };
            }
        }
        let Some(DragEditTarget::Editable { element, .. }) = target else {
            self.drag_caret.borrow_mut().take();
            return Ok(false);
        };
        let Some(caret) = caret.filter(|caret| caret.IsConnected()) else {
            self.drag_caret.borrow_mut().take();
            return Ok(false);
        };
        self.drag_caret.borrow_mut().take();
        let move_selection = self.DragIsMove(context, event);
        let Some((fragment, plain)) =
            context.CreateDropFragment(data, caret.as_ref(), move_selection)?
        else {
            return Ok(false);
        };
        let root = context.RootEditableElement();
        context.BeginDragAndDropCommandGroup();
        let smart_delete = context.SmartInsertDeleteEnabled();
        let smart_insert = if move_selection {
            smart_delete && context.SelectionFacts().word_granularity && data.CanSmartReplace()
        } else {
            !plain && data.CanSmartReplace()
        };
        if move_selection
            && !context.DeleteSelectionAfterDraggingWithEvents(caret.as_ref(), smart_delete)?
        {
            return Ok(false);
        }
        if context.SetSelectionToDragCaret(caret.as_ref()) {
            if !context.ReplaceSelectionAfterDraggingWithEvents(
                &element,
                data,
                fragment.as_ref(),
                smart_insert,
                plain,
            )? {
                return Ok(false);
            }
        }
        context.UpdateDragStateAfterEdit(root.as_ref());
        Ok(true)
    }
    pub fn HandleTargetDrag(
        &self,
        context: &dyn DragTargetContext,
        action: &DragTargetAction,
        event: &WebMouseEvent,
    ) -> Result<DragTargetOperation, &'static str> {
        match action {
            DragTargetAction::Enter {
                data,
                source_operations,
            } => {
                self.target_data
                    .replace(Some(DragDataObject::FromWebDragData(data)?));
                self.target_operations.set(*source_operations);
            }
            DragTargetAction::Update { source_operations } => {
                self.target_operations.set(*source_operations)
            }
            _ => {}
        }
        let Some(data) = self.target_data.borrow().clone() else {
            return Ok(DragTargetOperation::default());
        };
        let data = DragData {
            data,
            source_operations: self.target_operations.get(),
        };
        let policy = if matches!(action, DragTargetAction::Drop { .. }) {
            DataTransferAccessPolicy::Readable
        } else {
            DataTransferAccessPolicy::TypesReadable
        };
        let transfer = TargetTransfer(DataTransfer::CreateDragTarget(
            data.data.clone(),
            policy,
            data.source_operations,
        ));
        match action {
            DragTargetAction::Enter { .. } | DragTargetAction::Update { .. } => {
                if !context.HasView() || !context.HasDocumentAt(event) {
                    self.ClearDropTarget(context);
                    return Ok(DragTargetOperation::default());
                }
                self.document_under_mouse.set(true);
                let mut handled = context.UpdateDragAndDrop(event, &transfer.0);
                // A dragenter handler can spin a nested event loop and deliver
                // DragExited. Do not revive that document on the outer return.
                if !self.document_under_mouse.get() {
                    return Ok(DragTargetOperation::default());
                }
                let mut operation = DragOperation::None;
                if handled {
                    self.ClearDropTarget(context);
                    if !transfer.0.DropEffectIsInitialized() {
                        transfer.0.SetDestinationOperationFromEffectAllowed();
                    }
                    operation = transfer.0.DestinationOperation();
                    if data.source_operations & u32::from(operation as u8) == 0 {
                        operation = DragOperation::None;
                    }
                } else {
                    match context.EditTarget(event, &data, self.DidInitiateDrag())? {
                        Some(DragEditTarget::FileInput {
                            element,
                            disabled,
                            multiple,
                        }) => {
                            self.ClearDropTarget(context);
                            let allowed = !disabled
                                && if multiple {
                                    data.NumberOfFiles() > 0
                                } else {
                                    data.NumberOfFiles() == 1
                                };
                            context.SetCanReceiveDroppedFiles(&element, allowed);
                            self.file_input_under_mouse.replace(Some(element));
                            handled = true;
                            if allowed {
                                operation = if self.DragIsMove(context, event) {
                                    DragOperation::Move
                                } else {
                                    DragOperation::Copy
                                };
                            }
                        }
                        Some(DragEditTarget::Editable { caret, .. }) => {
                            self.ClearDropTarget(context);
                            self.drag_caret.replace(Some(caret));
                            handled = true;
                            operation = if self.DragIsMove(context, event) {
                                DragOperation::Move
                            } else {
                                DragOperation::Copy
                            };
                        }
                        None => self.ClearDropTarget(context),
                    }
                }
                self.document_is_handling_drag.set(handled);
                if !handled && context.AcceptsLoadDrops() && !context.DocumentIsEditable() {
                    operation = data.OperationForLoad(self.DidInitiateDrag());
                }
                Ok(DragTargetOperation {
                    operation,
                    document_is_handling_drag: handled,
                })
            }
            DragTargetAction::Leave => {
                self.document_under_mouse.set(false);
                if context.HasView() {
                    context.CancelDragAndDrop(event, &transfer.0);
                }
                self.ClearDropTarget(context);
                self.target_data.borrow_mut().take();
                self.document_is_handling_drag.set(false);
                Ok(DragTargetOperation::default())
            }
            DragTargetAction::Drop { operation } => {
                if context.HasDocumentAt(event) {
                    context.NotifyDropUserActivation();
                }
                let handled = if context.HasView() && self.document_is_handling_drag.get() {
                    transfer.0.SetDestinationOperation(*operation);
                    context.PerformDragAndDrop(event, &transfer.0)
                } else {
                    false
                };
                let result = if handled {
                    self.ClearDropTarget(context);
                    Ok(true)
                } else {
                    self.ConcludeEditDrag(context, event, &data)
                };
                if matches!(result, Ok(false))
                    && context.AcceptsLoadDrops()
                    && !context.DocumentIsEditable()
                    && data.OperationForLoad(self.DidInitiateDrag()) != DragOperation::None
                {
                    for (index, url) in data.URLs().iter().enumerate() {
                        context.LoadDroppedURL(url, index == 0);
                    }
                }
                self.ClearDropTarget(context);
                self.target_data.borrow_mut().take();
                self.document_is_handling_drag.set(false);
                self.document_under_mouse.set(false);
                result.map(|_| DragTargetOperation::default())
            }
        }
    }
}
