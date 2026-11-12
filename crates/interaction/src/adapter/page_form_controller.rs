//! Assembly of Document's retained FormController. The source controller owns
//! serialization, form keys, FIFO matching and restoration order; this port
//! only accesses canonical DOM/control values and the existing dispatcher.
use super::{
    interaction_contract::{InteractionDocument, ReadNode},
    page_interaction::Interaction,
};
use interaction_default::core::html::forms::{
    form_controller::{FormController, FormStateDocument, StateString, StateVector},
    input_type::EventQueueScope,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};
pub type FormStateTaskRunner = Rc<dyn Fn(Box<dyn FnOnce()>)>;
pub struct PageFormController {
    controller: Rc<FormController>,
    document: Rc<FormDocument>,
}
struct FormDocument {
    document: InteractionDocument,
    engine: Interaction<'static>,
    task_runner: RefCell<Option<FormStateTaskRunner>>,
    pending_tasks: RefCell<Vec<Box<dyn FnOnce()>>>,
    file_reader: RefCell<Option<Arc<dyn fileapi::NativeFileReader>>>,
}
impl PageFormController {
    pub fn Install(document: InteractionDocument, engine: Interaction<'static>) -> Rc<Self> {
        let state = Rc::new(FormDocument {
            document: document.clone(),
            engine,
            task_runner: RefCell::new(None),
            pending_tasks: RefCell::new(Vec::new()),
            file_reader: RefCell::new(None),
        });
        let owner = Rc::new(Self {
            controller: FormController::new(state.clone()),
            document: state,
        });
        let lifecycle: Rc<dyn dom::form_controller::DocumentFormController> = owner.clone();
        let document = document.borrow();
        document
            .GetDocument()
            .AttachFormController(Rc::downgrade(&lifecycle));
        document
            .GetDocument()
            .AttributeLifecycle()
            .BindFormController(Rc::downgrade(&lifecycle));
        owner
    }
    pub fn SetTaskRunner(&self, runner: FormStateTaskRunner) {
        *self.document.task_runner.borrow_mut() = Some(runner.clone());
        let pending = std::mem::take(&mut *self.document.pending_tasks.borrow_mut());
        for task in pending {
            runner(task);
        }
    }
    pub fn SetFileReader(&self, reader: Arc<dyn fileapi::NativeFileReader>) {
        *self.document.file_reader.borrow_mut() = Some(reader);
    }
}
impl dom::form_controller::DocumentFormController for PageFormController {
    fn ControlStates(&self) -> Rc<dyn navigation_history::DocumentState> {
        self.controller.ControlStates()
    }
    fn SetStateForNewControls(&self, state: &[StateString]) {
        self.controller.SetStateForNewControls(state);
    }
    fn RestoreImmediately(&self) -> Result<(), navigation_history::FormStateRestoreError> {
        self.controller.RestoreImmediately();
        self.controller.RestoreError().map_or(Ok(()), Err)
    }
    fn SetNativeFileReader(&self, reader: Arc<dyn fileapi::NativeFileReader>) {
        self.SetFileReader(reader);
    }
    fn RestoreError(&self) -> Option<navigation_history::FormStateRestoreError> {
        self.controller.RestoreError()
    }

    fn StatefulControlConnected(&self, node: u64) {
        self.controller.InvalidateStatefulFormControlList();
        self.controller.RestoreControlStateFor(node);
    }
    fn CustomElementUpgraded(&self, node: u64) {
        self.controller.InvalidateStatefulFormControlList();
        self.controller.RestoreControlStateOnUpgrade(node);
    }
    fn FinishedParsing(
        &self,
        node: u64,
        kind: dom::attribute_lifecycle::FormStateParsingKind,
        was_finished: bool,
    ) {
        if !was_finished {
            return;
        }
        match kind {
            dom::attribute_lifecycle::FormStateParsingKind::Control => {
                self.controller.RestoreControlStateFor(node)
            }
            dom::attribute_lifecycle::FormStateParsingKind::Form => {
                self.controller.RestoreControlStateIn(node)
            }
        }
    }
    fn DocumentFinishedParsing(&self) {
        self.controller.ScheduleRestore();
    }
    fn WillDeleteForm(&self, node: u64) {
        self.controller.WillDeleteForm(node);
    }
}
impl FormDocument {
    fn Read<R>(
        &self,
        control: u64,
        read: impl FnOnce(&dom::Document, &dom::persistent_document::DOMNode, usize) -> R,
    ) -> R {
        ReadNode(&self.document, control, read).expect("retained form-state control identity")
    }
}
impl FormStateDocument for FormDocument {
    fn StatefulControlRevision(&self) -> u64 {
        self.document
            .borrow()
            .GetDocument()
            .StatefulFormControlRevision()
    }
    fn StatefulControls(&self) -> Vec<u64> {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let mut node = document.NextFormStateElement(document.Root());
        let mut result = Vec::new();
        while let Some(index) = node {
            if document.ClassSupportsFormStateRestore(index) {
                result.push(document.Node(index).Id());
            }
            node = document.NextFormStateElement(index);
        }
        result
    }
    fn ListedControls(&self, form: u64) -> Vec<u64> {
        self.Read(form, |document, _, index| {
            document
                .ListedFormControlNodeIds(index)
                .iter()
                .copied()
                .filter(|&id| {
                    document
                        .FindNodeById(id)
                        .is_some_and(|index| document.ClassSupportsFormStateRestore(index))
                })
                .collect()
        })
    }
    fn OwnerFormForState(&self, control: u64) -> Option<u64> {
        self.Read(control, |document, node, index| {
            if node.FindAttribute("form").is_some() {
                None
            } else {
                document
                    .FormOwner(index)
                    .map(|form| document.Node(form).Id())
            }
        })
    }
    fn FormSignature(&self, form: u64) -> StateString {
        self.Read(form, |document, _, form| {
            let mut signature = document.FormSignatureActionURL(form).to_vec();
            signature.extend(" [".encode_utf16());
            let mut named = 0;
            for &id in document.ListedFormControlNodeIds(form).iter() {
                let index = document.FindNodeById(id).expect("listed control");
                let node = document.Node(index);
                if !document.ClassSupportsFormStateRestore(index)
                    || node.FindAttribute("form").is_some()
                    || document.FormOwner(index).is_none()
                {
                    continue;
                }
                let name = node
                    .AttributeValueUTF16("name")
                    .unwrap_or_else(|| Rc::from([]));
                if name.is_empty() {
                    continue;
                }
                signature.extend_from_slice(&name);
                signature.push(32);
                named += 1;
                if named == 2 {
                    break;
                }
            }
            signature.push(93);
            signature.into()
        })
    }
    fn ControlName(&self, control: u64) -> StateString {
        self.Read(control, |_, node, _| {
            node.AttributeValueUTF16("name")
                .unwrap_or_else(|| Rc::from([]))
        })
    }
    fn ControlType(&self, control: u64) -> StateString {
        self.Read(control, |document, _, index| {
            document.FormStateControlType(index)
        })
    }
    fn ShouldSaveAndRestore(&self, control: u64) -> bool {
        self.Read(control, |document, node, index| {
            if document.IsFormAssociatedCustomElement(index) {
                return document.IsConnectedToDocument(index)
                    && !document.FormOwner(index).is_some_and(|form| {
                        document
                            .Node(form)
                            .FindAttribute("autocomplete")
                            .is_some_and(|attribute| attribute.value.eq_ignore_ascii_case("off"))
                    });
            }
            use dom::DOMInputType as T;
            document.IsConnectedToDocument(index)
                && document.ClassSupportsFormStateRestore(index)
                && !document.InputType(index).is_some_and(|kind| {
                    matches!(
                        kind,
                        T::kButton | T::kReset | T::kSubmit | T::kImage | T::kHidden | T::kPassword
                    )
                })
                && !node
                    .FindAttribute("autocomplete")
                    .is_some_and(|attribute| attribute.value.eq_ignore_ascii_case("off"))
                && !document.FormOwner(index).is_some_and(|form| {
                    document
                        .Node(form)
                        .FindAttribute("autocomplete")
                        .is_some_and(|attribute| attribute.value.eq_ignore_ascii_case("off"))
                })
        })
    }
    fn SaveControlState(&self, control: u64) -> StateVector {
        self.engine.SaveFormControlState(&self.document, control)
    }
    fn RestoreControlState(
        &self,
        control: u64,
        state: &[StateString],
    ) -> Result<(), navigation_history::FormStateRestoreError> {
        self.engine.RestoreFormControlState(
            &self.document,
            control,
            state,
            self.file_reader.borrow().clone(),
        )
    }
    fn IsDisabledOrReadonly(&self, control: u64) -> bool {
        self.Read(control, |document, node, index| {
            document.IsDisabledFormControl(index) || node.FindAttribute("readonly").is_some()
        })
    }
    fn IsDirtyControl(&self, control: u64) -> bool {
        self.Read(control, |document, _, index| {
            document.ControlUserHasEditedTheField(index)
        })
    }
    fn HasFinishedParsing(&self) -> bool {
        self.document.borrow().GetDocument().ParsingState()
            == dom::document_parser_lifecycle::DocumentParsingState::Finished
    }
    fn IsActive(&self) -> bool {
        self.document.borrow().GetDocument().IsActive()
    }
    fn BeginEventQueueScope(&self) -> Box<dyn EventQueueScope + '_> {
        self.engine.BeginFormStateEventQueueScope(&self.document)
    }
    fn PostRestoreTask(&self, task: Box<dyn FnOnce()>) {
        let runner = self.task_runner.borrow().clone();
        if let Some(runner) = runner {
            runner(task);
        } else {
            self.pending_tasks.borrow_mut().push(task);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dom::form_controller::DocumentFormController as _;
    use dom::{
        custom_elements::{
            CustomElementCallbacks, CustomElementDescriptor, CustomElementOptions,
            CustomElementReaction,
        },
        element_internals::ElementInternalsValue as V,
        persistent_document::{DOMAttribute, DOMNamespace},
    };
    fn document() -> (InteractionDocument, Rc<PageFormController>, usize) {
        let document = Rc::new(RefCell::new(dom::DOM::new()));
        let element = {
            let mut owner = document.borrow_mut();
            let d = owner.GetDocumentMut();
            d.Lifecycle()
                .AdvanceTo(dom::document_lifecycle::LifecycleState::VisualUpdatePending);
            let form = d.CreateElementDefault(DOMNamespace::kHTML, "form".into());
            let element = d.CreateElementDefault(DOMNamespace::kHTML, "x-history-field".into());
            d.SetAttribute(
                element,
                DOMAttribute {
                    local_name: "name".into(),
                    value: "field".into(),
                    ..Default::default()
                },
            );
            d.AppendChild(d.Root(), form);
            d.AppendChild(form, element);
            element
        };
        let engine = Interaction::new(
            Rc::new(|_| panic!("custom restoration only enqueues native reactions")),
            None,
            Rc::new(super::super::host_event_clock::HostEventClock),
        );
        let controller = PageFormController::Install(document.clone(), engine);
        (document, controller, element)
    }
    fn upgrade(document: &InteractionDocument, element: usize) {
        {
            let mut owner = document.borrow_mut();
            let d = owner.GetDocumentMut();
            let definition = d
                .RegisterCustomElementDefinitionWithOptions(
                    CustomElementDescriptor {
                        name: "x-history-field".into(),
                        local_name: "x-history-field".into(),
                    },
                    Vec::new(),
                    CustomElementCallbacks {
                        form_state_restore: true,
                        ..Default::default()
                    },
                    CustomElementOptions {
                        form_associated: true,
                        ..Default::default()
                    },
                )
                .unwrap();
            assert!(matches!(
                d.PopCustomElementReaction(),
                Some(CustomElementReaction::Upgrade { .. })
            ));
            assert!(d.BeginCustomElementUpgrade(element, definition));
            d.AttachElementInternals(element).unwrap();
            d.SetElementInternalsFormValue(
                element,
                Some(V::String("constructor-value".into())),
                Some(V::String("constructor-state".into())),
            );
            d.CompleteCustomElementUpgrade(element, definition, true);
        }
        dom::attribute_lifecycle::DrainAttributeLifecycle(document);
    }
    #[test]
    fn independent_document_history_owners_restore_custom_state_after_late_upgrade() {
        let mut histories = Vec::new();
        for role in ["root", "child"] {
            let (document, controller, element) = document();
            upgrade(&document, element);
            let item = navigation_history::HistoryItemOwner::Create();
            item.SetDocumentStateOwner(controller.ControlStates());
            let mut owner = document.borrow_mut();
            let d = owner.GetDocumentMut();
            let mut state = fileapi::form_data::FormData::default();
            state.AppendString("role".into(), role.into());
            d.SetElementInternalsFormValue(
                element,
                Some(V::String(format!("{role}-edited-after-attachment"))),
                Some(V::FormData(Rc::new(state))),
            );
            drop(owner);
            // Freeze the existing HistoryItem payload exactly as host/session
            // history does; the new document never borrows the previous owner.
            histories.push((role, item.Snapshot()));
        }
        for (role, snapshot) in histories {
            let (document, controller, element) = document();
            let item = navigation_history::HistoryItemOwner::Restore(&snapshot);
            controller.SetStateForNewControls(&item.DocumentState());
            controller.RestoreImmediately().unwrap();
            {
                let mut owner = document.borrow_mut();
                let d = owner.GetDocumentMut();
                for name in ["disabled", "readonly"] {
                    d.SetAttribute(
                        element,
                        DOMAttribute {
                            local_name: name.into(),
                            ..Default::default()
                        },
                    );
                }
            }
            upgrade(&document, element);
            let mut owner = document.borrow_mut();
            let d = owner.GetDocumentMut();
            let Some(CustomElementReaction::FormStateRestore {
                element: target,
                state: V::FormData(state),
                ..
            }) = d.PopCustomElementReaction()
            else {
                panic!("native custom state restore reaction");
            };
            assert_eq!(target, element);
            assert_eq!(state.Get("role").unwrap().StringForFormData(), role);
            assert!(d.PopCustomElementReaction().is_none());
            let values = d
                .SaveElementInternalsFormControlState(element)
                .iter()
                .map(|value| String::from_utf16_lossy(value))
                .collect::<Vec<_>>();
            assert_eq!(
                values,
                [
                    "Value",
                    "USVString",
                    &format!("{role}-edited-after-attachment"),
                    "State",
                    "USVString",
                    "constructor-state"
                ]
            );
        }
    }
}
