//! Page control facts and source default-action adapters.
//!
//! Checkable activation, radio navigation, range stepping, select decisions,
//! form reset and implicit submission execute in translated child owners.
//! Direct form preparation and the post-first-cut image-input submission
//! bridge preserve Page's existing callback contract and stay explicit below.

use crate::{
    event::{Event, EventType, MakeSyntheticEvent},
    focus_controller::FocusController,
    input_event::MouseButton,
    ownership::{
        Attribute, HasAttribute, InteractionDOMMutationEmitter, InteractionDocument, IsElement,
        ReadNode,
    },
    editing_owners::{SyntheticEventDispatcher, PageEditingOwners},
};
use dom::{
    dom_mutation::{DOMMutation, DOMMutationType},
    UserInteractionState,
};
pub use interaction_default::core::html::forms::html_form_element::RequestSubmitError;
use interaction_default::core::html::forms::{
    base_checkable_input_type::CheckableKeyboardEvent,
    checkbox_input_type::{CheckboxActivationOperations, CheckboxInputType},
    html_form_element::{
        FormEntryListOperations, FormResetOperations, FormResetState, FormSubmissionState,
        HTMLFormElement, ImplicitSubmissionOperations, RequestSubmitOperations,
    },
    html_input_element::{HTMLInputResetOperations, ResetInputElement},
    html_select_element::{
        HTMLSelectElement, SelectElementKeyboardOperations, SelectOptionOperations,
        SelectResetOperations, SelectTypeAheadOperations,
    },
    html_text_area_element::{HTMLTextAreaElement, TextAreaResetOperations},
    input_type::TextFieldEventBehavior,
    input_type::{
        TextControlSetValueSelection, Type as SourceInputType, ValueMode, ValueModeForType,
    },
    input_type_view::ClickHandlingState,
    menu_list_popup::MenuListPopup,
    radio_input_type::{RadioActivationOperations, RadioInputType, RadioKeyboardOperations},
    range_input_type::{
        EventQueueScope, RangeInputType as SourceRangeInputType, RangeKeyboardOperations,
        RangeWritingDirection,
    },
    select_type::{
        ListBoxSelectionState, SelectEvent, SelectKeyboardOperations, SelectOptionStateOperations,
    },
    step_range::NumberTypeParser,
    type_ahead::{TypeAhead, TypeAheadDataSource},
};
use interaction_default::{
    core::html::parser::html_parser_idioms::NumberTypeParser as HTMLNumberParser,
    platform::wtf::decimal::Decimal,
};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};
/// Only native source default-action state and element owners are retained
/// here, keyed by canonical identities. No DOM tree, control value, fragment
/// geometry or selection is copied into these maps.
pub struct DefaultActionState {
    anchors: RefCell<
        HashMap<
            dom::persistent_document::DOMOwnerHandle,
            Rc<interaction_default::core::html::html_anchor_element::HTMLAnchorElementBase>,
        >,
    >,
    select_host: RefCell<Option<Rc<dyn super::page_select_control::SelectControlHost>>>,
    pub(crate) numbers: HTMLNumberParser,
    pub(crate) validation: super::page_form_validation::FormValidationState,
    form_resets: RefCell<HashMap<u64, Rc<FormResetState>>>,
    forms: RefCell<HashMap<dom::persistent_document::DOMOwnerHandle, Rc<FormSubmissionState>>>,
    selects: RefCell<HashMap<u64, RetainedSelectState>>,
    form_controls: RefCell<
        HashMap<dom::persistent_document::DOMOwnerHandle, Rc<RetainedFormControlSubmission>>,
    >,
    labels: RefCell<
        HashMap<
            u64,
            Rc<interaction_default::core::html::forms::html_label_element::LabelDefaultState>,
        >,
    >,
}
#[derive(Default)]
struct RetainedFormControlSubmission {
    activated: Rc<Cell<bool>>,
    image_point: Cell<(i32, i32)>,
}
// HTMLSelectElement owns TypeAhead across SelectType changes. Only the
// subtype's original list-box state is replaced when rendering type changes.
struct RetainedSelectState {
    menu: bool,
    listbox: Rc<ListBoxSelectionState<u64>>,
    type_ahead: Rc<TypeAhead>,
    menu_active_option:
        Rc<interaction_default::core::html::forms::select_type::MenuListActiveOptionState>,
}
impl Default for DefaultActionState {
    fn default() -> Self {
        Self {
            anchors: RefCell::new(HashMap::new()),
            select_host: RefCell::new(None),
            validation: Default::default(),
            numbers: HTMLNumberParser::FromDoubleConversion(Rc::new(
                foundation::number_conversion::double_conversion::ChromiumDoubleConversion,
            )),
            labels: RefCell::new(HashMap::new()),
            selects: RefCell::new(HashMap::new()),
            form_controls: RefCell::new(HashMap::new()),
            form_resets: RefCell::new(HashMap::new()),
            forms: RefCell::new(HashMap::new()),
        }
    }
}
impl DefaultActionState {
    pub(crate) fn Anchor(
        &self,
        owner: dom::persistent_document::DOMOwnerHandle,
        create: impl FnOnce() -> Rc<
            interaction_default::core::html::html_anchor_element::HTMLAnchorElementBase,
        >,
    ) -> Rc<interaction_default::core::html::html_anchor_element::HTMLAnchorElementBase> {
        self.anchors
            .borrow_mut()
            .entry(owner)
            .or_insert_with(create)
            .clone()
    }
    pub fn SetSelectControlHost(
        &self,
        document: &InteractionDocument,
        host: Rc<dyn super::page_select_control::SelectControlHost>,
    ) {
        let lifecycle: Rc<dyn dom::attribute_lifecycle::SelectControlLifecycle> = host.clone();
        document
            .borrow()
            .GetDocument()
            .AttributeLifecycle()
            .BindSelectControl(Rc::downgrade(&lifecycle));
        *self.select_host.borrow_mut() = Some(host);
    }
    fn Select(&self, id: u64, menu: bool) -> (Rc<ListBoxSelectionState<u64>>, Rc<TypeAhead>) {
        let mut owners = self.selects.borrow_mut();
        let entry = owners.entry(id).or_insert_with(|| RetainedSelectState {
            menu,
            listbox: Rc::new(ListBoxSelectionState::default()),
            type_ahead: Rc::new(TypeAhead::new()),
            menu_active_option: Rc::new(Default::default()),
        });
        if entry.menu != menu {
            entry.menu = menu;
            entry.listbox = Rc::new(ListBoxSelectionState::default());
            entry.menu_active_option = Rc::new(Default::default());
        }
        (entry.listbox.clone(), entry.type_ahead.clone())
    }
    // Source HTMLSelectElement::DispatchBlurEvent, cc:1159. Drop the map
    // borrow before the focus transaction proceeds into author callbacks.
    pub(crate) fn ResetSelectTypeAheadForBlur(&self, id: u64) {
        let session = self.selects.borrow().get(&id).map(|s| s.type_ahead.clone());
        if let Some(session) = session {
            session.ResetSession();
        }
    }

    fn FormSubmission(
        &self,
        document: &InteractionDocument,
        id: u64,
    ) -> Option<Rc<FormSubmissionState>> {
        let identity = ReadNode(document, id, |document, node, index| {
            IsElement(node, "form").then(|| document.NodeHandle(index))
        })
        .flatten()?;
        Some(self.forms.borrow_mut().entry(identity).or_default().clone())
    }
    fn FormControl(
        &self,
        document: &InteractionDocument,
        id: u64,
    ) -> Option<Rc<RetainedFormControlSubmission>> {
        let identity = ReadNode(document, id, |document, _, index| {
            document.NodeHandle(index)
        })?;
        Some(
            self.form_controls
                .borrow_mut()
                .entry(identity)
                .or_default()
                .clone(),
        )
    }
    fn FormReset(&self, id: u64) -> Rc<FormResetState> {
        self.form_resets.borrow_mut().entry(id).or_default().clone()
    }
    pub(crate) fn Label(
        &self,
        id: u64,
    ) -> Rc<interaction_default::core::html::forms::html_label_element::LabelDefaultState> {
        self.labels.borrow_mut().entry(id).or_default().clone()
    }
}

// cpp: interaction/input_type.h:19-30
pub trait ActivationState: Any {
    fn AsAny(&self) -> &dyn Any;
}
#[derive(Clone)]
pub struct InputTypeContext<'a> {
    pub anchor_navigation: Option<Rc<dyn interaction_default::core::html::html_anchor_element::AnchorNavigationHost>>,
    pub event_targets: Rc<super::dom_event::DomEventTargets>,
    pub script_invocation: Option<&'a dyn interaction_dispatch::bindings::script_invocation::ScriptInvocation>,
    pub document: InteractionDocument,
    pub editing_geometry: Option<crate::adapter::page_editing_context::PreparedEditingGeometry<'a>>,
    pub editing_geometry_revision: Option<u64>,
    pub default_action_state: Rc<DefaultActionState>,
    pub selection_controller_state: Rc<interaction_default::core::editing::selection_controller::SelectionControllerState>,
    pub emit_mutation: InteractionDOMMutationEmitter,
    pub interaction_state: Rc<RefCell<UserInteractionState>>,
    pub editor: Rc<PageEditingOwners>,
    pub focus_controller: FocusController<'a>,
    pub submit_form: Option<Rc<dyn Fn(Result<Option<PrepareFormEntryListResult>, FormDataAdmission>) + 'a>>,
    pub dispatch_form_data: Rc<dyn Fn(u64, Rc<interaction_dispatch::core::fileapi::form_data_object::FormDataObject>) + 'a>,
    pub close_dialog: Rc<dyn Fn(u64, Rc<[u16]>) + 'a>,
    pub dispatch_synthetic: SyntheticEventDispatcher<'a>,
    pub dispatch_simulated_click: Rc<dyn Fn(u64, &Event) + 'a>,
    pub dispatch_dom_activate: Rc<dyn Fn(u64, &Event) -> interaction_dispatch::core::dom::events::event_dispatch_result::DispatchEventResult + 'a>,
    pub begin_event_queue_scope: Rc<dyn Fn() -> Box<dyn EventQueueScope + 'a> + 'a>,
    pub dispatch_scoped_synthetic: Rc<dyn Fn(Event, u64) + 'a>,
}
// cpp: interaction/input_type.h:35-47
// cpp: interaction/input_type.cc:475-483
pub trait InputType {
    fn HasActivationBehavior(&self) -> bool {
        false
    }
    fn LegacyPreActivationBehavior(&self, _: &mut Event) -> Option<Box<dyn ActivationState>> {
        None
    }
    fn RunActivationBehavior(&self, _: &mut Event, _: Option<&dyn ActivationState>) {}
    fn HandleDefaultEvent(&self, _: &mut Event) {}
}
struct BaseInputType<'a> {
    _node: u64,
    _context: InputTypeContext<'a>,
}
impl<'a> InputType for BaseInputType<'a> {}
// cpp: interaction/input_type.cc:18-30
fn LowerASCII(v: &str) -> String {
    v.to_ascii_lowercase()
}
fn AttributeValue(c: &InputTypeContext<'_>, id: u64, name: &str) -> String {
    Attribute(&c.document, id, name)
}
fn Element(c: &InputTypeContext<'_>, id: u64, name: &str) -> bool {
    ReadNode(&c.document, id, |_, n, _| IsElement(n, name)).unwrap_or(false)
}
// cpp: interaction/input_type.cc:39-46
fn DispatchInputAndChange(id: u64, dispatch: &SyntheticEventDispatcher<'_>) {
    let mut input = MakeSyntheticEvent(EventType::kInput, id);
    dispatch(&mut input, id);
    let mut change = MakeSyntheticEvent(EventType::kChange, id);
    dispatch(&mut change, id);
}
// cpp: interaction/input_type.cc:48-64
pub(super) fn DispatchKeyPress(
    event: &mut Event,
    id: u64,
    dispatch: &SyntheticEventDispatcher<'_>,
) -> bool {
    if event.r#type != EventType::kKeyDown
        || (event.text.is_empty() && event.key != "Enter" && event.key != " ")
    {
        return true;
    }
    let mut keypress = MakeSyntheticEvent(EventType::kKeyPress, id);
    keypress.key = event.key.clone();
    keypress.code = event.code.clone();
    keypress.text = if event.text.is_empty() {
        match event.key.as_str() {
            "Enter" => "\r".into(),
            " " => " ".into(),
            _ => String::new(),
        }
    } else {
        event.text.clone()
    };
    // KeyboardEvent::KeyboardEvent(WebKeyboardEvent) uses the first text
    // character for both charCode and keyCode on keypress.
    keypress.char_code = keypress.text.chars().next().map_or(0, u32::from);
    keypress.key_code = keypress.char_code;
    keypress.key_location = event.key_location;
    keypress.is_composing = event.is_composing;
    keypress.modifiers = event.modifiers;
    keypress.repeat = event.repeat;
    keypress.trusted = event.trusted;
    dispatch(&mut keypress, id);
    if !keypress.default_prevented {
        return true;
    }
    event.default_handled = true;
    false
}
// cpp: interaction/input_type.cc:66-90
fn FormOwner(c: &InputTypeContext<'_>, id: u64) -> Option<u64> {
    ReadNode(&c.document, id, |document, _, index| {
        document
            .FormOwner(index)
            .map(|index| document.Node(index).Id())
    })
    .flatten()
}
// cpp: interaction/input_type.cc:92-110
fn NodeText(c: &InputTypeContext<'_>, id: u64) -> String {
    ReadNode(&c.document, id, |document, _, index| {
        String::from_utf16_lossy(&document.TextContentUTF16(index))
    })
    .unwrap_or_default()
}
fn OptionValue(c: &InputTypeContext<'_>, id: u64) -> String {
    if HasAttribute(&c.document, id, "value") {
        AttributeValue(c, id, "value")
    } else {
        NodeText(c, id)
    }
}
fn SetUserEdited(c: &InputTypeContext<'_>, id: u64) {
    (c.emit_mutation)(&DOMMutation {
        mutation_type: DOMMutationType::kSetControlUserEdited,
        target_node_id: id,
        bool_value: true,
        ..Default::default()
    });
}
fn SetValue(c: &InputTypeContext<'_>, id: u64, value: String) {
    (c.emit_mutation)(&DOMMutation {
        mutation_type: DOMMutationType::kSetControlValue,
        target_node_id: id,
        value,
        ..Default::default()
    })
}
fn SetChecked(c: &InputTypeContext<'_>, id: u64, checked: bool) {
    (c.emit_mutation)(&DOMMutation {
        mutation_type: DOMMutationType::kSetControlChecked,
        target_node_id: id,
        bool_value: checked,
        ..Default::default()
    })
}
fn SetIndeterminate(c: &InputTypeContext<'_>, id: u64, indeterminate: bool) {
    (c.emit_mutation)(&DOMMutation {
        mutation_type: DOMMutationType::kSetControlIndeterminate,
        target_node_id: id,
        bool_value: indeterminate,
        ..Default::default()
    })
}
fn Checked(c: &InputTypeContext<'_>, id: u64) -> bool {
    ReadNode(&c.document, id, |d, _, i| d.ControlChecked(i)).unwrap_or(false)
}
fn Indeterminate(c: &InputTypeContext<'_>, id: u64) -> bool {
    ReadNode(&c.document, id, |d, _, i| d.ControlIndeterminate(i)).unwrap_or(false)
}
fn RadioGroup(c: &InputTypeContext<'_>, id: u64) -> Vec<u64> {
    ReadNode(&c.document, id, |d, _, i| d.RadioGroupNodeIds(i)).unwrap_or_default()
}
fn Value(c: &InputTypeContext<'_>, id: u64) -> String {
    ReadNode(&c.document, id, |d, _, i| d.ControlValue(i)).unwrap_or_default()
}
// cpp: interaction/input_type.cc:134-175
fn ResetForm(form: u64, c: &InputTypeContext<'_>) {
    let state = c.default_action_state.FormReset(form);
    HTMLFormElement::ResetWithOperations(&state, &PageFormReset { c, form });
}
/// requestSubmit's source admission, shared with the native HTMLFormElement.
/// The subsequent preparation bridge retains the same form transaction state.
pub(crate) fn RequestSubmit(
    c: &InputTypeContext<'_>,
    form: u64,
    submitter: Option<u64>,
) -> Result<(), RequestSubmitError> {
    HTMLFormElement::RequestSubmitWithOperations(submitter, &PageRequestSubmit { c, form })
}
struct PageRequestSubmit<'a, 'b> {
    c: &'a InputTypeContext<'b>,
    form: u64,
}
impl RequestSubmitOperations<u64> for PageRequestSubmit<'_, '_> {
    fn IsNativeFormControl(&self, control: &u64) -> bool {
        ReadNode(&self.c.document, *control, |document, _, index| {
            document.IsFormControlElement(index)
        })
        .unwrap_or(false)
    }
    fn CanBeSuccessfulSubmitButton(&self, control: &u64) -> bool {
        use interaction_default::core::html::forms::html_button_element::HTMLButtonElement;
        ReadNode(&self.c.document, *control, |document, node, index| {
            if IsElement(node, "button") {
                let kind = HTMLButtonElement::TypeForAttributes(
                    node.FindAttribute("type")
                        .map_or("", |attribute| attribute.value.as_str()),
                    node.FindAttribute("command").is_some(),
                );
                let owner_select = node.Parent().is_some_and(|parent| {
                    IsElement(document.Node(parent), "select")
                        && document.SelectSlottedButton(parent) == Some(index)
                });
                HTMLButtonElement::CanBeSuccessfulSubmitButtonWithFacts(kind, owner_select)
            } else {
                document.CanBeSuccessfulSubmitButton(index)
            }
        })
        .unwrap_or(false)
    }
    fn IsOwnedByForm(&self, control: &u64) -> bool {
        FormOwner(self.c, *control) == Some(self.form)
    }
    fn PrepareForSubmission(&self, submitter: Option<u64>) {
        SubmitForm(self.c, self.form, submitter, true);
    }
}
// The shared HTMLFormElement owns submission/entry-list reentry admission and
// scope lifetimes. The legacy Page callback still needs the external frame,
// validation and navigation inputs documented below; it is not a substitute
// for source CreateSubmission/target scheduling.
pub(crate) fn SubmitForm(
    c: &InputTypeContext<'_>,
    form: u64,
    submitter: Option<u64>,
    dispatch_event: bool,
) {
    let Some(state) = c.default_action_state.FormSubmission(&c.document, form) else {
        return;
    };
    // Source PrepareForSubmission cc:638-640. Script form.submit() bypasses
    // only preparation; it must still obey ScheduleFormSubmission admission.
    if dispatch_event && state.PreparationIsBlocked() {
        return;
    }
    if !ReadNode(&c.document, form, |document, _, index| {
        document.IsActive() && document.IsConnectedToDocument(index)
    })
    .unwrap_or(false)
    {
        return;
    }
    if dispatch_event {
        if ReadNode(&c.document, form, |document, _, _| {
            document
                .SecurityContext()
                .IsSandboxed(foundation::security::SandboxFlags::FORMS)
        })
        .unwrap_or(true)
        {
            return;
        }
        HTMLFormElement::ExposeUserValidityWithOperations(&PageFormUserValidity { c, form });
        let not_canceled = {
            // Source cc:700-817: restore the event-handler flag before the
            // ScheduleFormSubmission call, including canceled-event/reentry.
            let _scope = state.EnterSubmitEvent();
            let skip_validation = HasAttribute(&c.document, form, "novalidate")
                || submitter.is_some_and(|submitter| {
                    HasAttribute(&c.document, submitter, "formnovalidate")
                });
            if !skip_validation {
                match super::page_form_validation::ValidateFormInteractively(c, form) {
                    Ok(true) => {}
                    Ok(false) => return,
                    Err(error) => {
                        if let Some(callback) = &c.submit_form {
                            callback(Err(FormDataAdmission::NeedsValidation(error)));
                        }
                        return;
                    }
                }
            }
            let mut submit = MakeSyntheticEvent(EventType::kSubmit, form);
            submit.submitter_node_id = submitter;
            (c.dispatch_synthetic)(&mut submit, form);
            !submit.default_prevented
        };
        if !not_canceled {
            return;
        }
    }
    // Source cc:901-912: a formdata listener cannot submit this form while
    // its same retained ConstructEntryList owner is still active.
    if state.SchedulingIsBlocked() {
        return;
    }
    // Source cc:915-916: release is_submitting before flushing dialog events.
    let _dialog_scope = (c.begin_event_queue_scope)();
    let submit_scope = state.EnterOwnedSubmission();
    let task_token = state.BeginScheduledSubmission();
    // FormSubmission::Create cc:262-272: dialog returns before entry-list
    // construction, so neither formdata nor URL/body encoding runs.
    let method = submitter
        .filter(|submitter| HasAttribute(&c.document, *submitter, "formmethod"))
        .map(|submitter| AttributeValue(c, submitter, "formmethod"))
        .unwrap_or_else(|| AttributeValue(c, form, "method"));
    if interaction_default::core::loader::form_submission::FormSubmissionAttributes::ParseMethod(
        &method,
    ) == interaction_default::core::html::forms::html_form_element::SubmitMethod::Dialog
    {
        CloseFormDialog(c, form, submitter);
        return;
    }
    if ReadNode(&c.document, form, |document, _, _| {
        document
            .SecurityContext()
            .IsSandboxed(foundation::security::SandboxFlags::FORMS)
    })
    .unwrap_or(true)
    {
        return;
    }
    // The typed entry-list continuation carries source ownership through
    // lifecycle preparation; FrameLoader performs target scheduling only once.
    if let Some(callback) = &c.submit_form {
        let encoding = ReadNode(&c.document, form, |document, node, _| {
            interaction_dispatch::core::fileapi::form_data::FormData::EncodingFromAcceptCharset(
                node.FindAttribute("accept-charset")
                    .map_or("", |attribute| attribute.value.as_str()),
                document.EncodingName(),
            )
        })
        .unwrap_or("UTF-8");
        let mut prepared = PrepareFormEntryList(c, form, submitter, encoding);
        if let Ok(Some(prepared)) = &mut prepared {
            let token = match prepared {
                PrepareFormEntryListResult::Ready(pending) => &mut pending.token,
                PrepareFormEntryListResult::NeedsLifecycle(token) => token,
            };
            token.submission_scope = Some(submit_scope.clone());
            token.task_token = Some(task_token.clone());
        }
        callback(prepared);
    }
}
struct PageFormUserValidity<'a, 'b> {
    c: &'a InputTypeContext<'b>,
    form: u64,
}
impl interaction_default::core::html::forms::html_form_element::FormUserValidityOperations<u64>
    for PageFormUserValidity<'_, '_>
{
    fn ListedElements(&self) -> Box<dyn Iterator<Item = u64> + '_> {
        ListedControls(self.c, self.form)
    }
    fn HasState(&self, control: &u64) -> bool {
        ReadNode(&self.c.document, *control, |document, _, index| {
            document.ClassSupportsFormStateRestore(index)
        })
        .unwrap_or(false)
    }
    fn ForceUserValid(&self, control: &u64) {
        let mut owner = self.c.document.borrow_mut();
        let document = owner.GetDocumentMut();
        if let Some(index) = document.FindNodeById(*control) {
            document.ForceControlUserValid(index);
        }
    }
}

/// HTMLFormElement::SubmitDialog cc:875-879. Ordinary ancestors, including
/// the nearest closed dialog, are selected by the DOM owner. Closing itself
/// uses the existing source dialog lifecycle and queued close event.
pub(crate) fn CloseFormDialog(c: &InputTypeContext<'_>, form: u64, submitter: Option<u64>) {
    let dialog = ReadNode(&c.document, form, |document, _, index| {
        let mut parent = document.Node(index).Parent();
        while let Some(index) = parent {
            let node = document.Node(index);
            if IsElement(node, "dialog") {
                return Some(node.Id());
            }
            parent = node.Parent();
        }
        None
    })
    .flatten();
    let Some(dialog) = dialog else {
        return;
    };
    let result = submitter
        .and_then(|submitter| {
            ReadNode(&c.document, submitter, |_, node, _| {
                if IsElement(node, "input")
                    && node
                        .FindAttribute("type")
                        .is_some_and(|attribute| attribute.value.eq_ignore_ascii_case("image"))
                {
                    let point = c
                        .default_action_state
                        .FormControl(&c.document, submitter)
                        .map_or((0, 0), |state| state.image_point.get());
                    Rc::from(
                        format!("{},{}", point.0, point.1)
                            .encode_utf16()
                            .collect::<Vec<_>>(),
                    )
                } else {
                    node.AttributeValueUTF16("value")
                        .unwrap_or_else(|| Rc::from([]))
                }
            })
        })
        .unwrap_or_else(|| Rc::from([]));
    (c.close_dialog)(dialog, result);
}

// Source owners choose submit/reset behavior only after Node dispatches the
// canonical DOMActivate event. This adapter supplies live element/form facts.
fn ActivateButton(id: u64, event: &mut Event, c: &InputTypeContext<'_>) {
    use interaction_default::core::html::forms::{
        html_button_element::{ButtonFormDefaultResult, HTMLButtonElement},
        reset_input_type::ResetInputType,
        submit_input_type::SubmitInputType,
    };
    let operations = PageButtonActivation { c, node: id };
    if Element(c, id, "button") {
        event.default_handled = HTMLButtonElement::HandleFormDOMActivateWithOperations(&operations)
            == ButtonFormDefaultResult::Handled;
    } else {
        match AttributeValue(c, id, "type").to_ascii_lowercase().as_str() {
            "reset" => {
                event.default_handled = ResetInputType::HandleDOMActivateWithOperations(&operations)
            }
            "submit" => {
                event.default_handled =
                    SubmitInputType::HandleDOMActivateWithOperations(&operations)
            }
            // ImageInputType is outside README's first cut. Preserve its old
            // Page submission bridge until the effect can carry its source
            // image-submit point; do not misclassify it as SubmitInputType.
            "image" => {
                if let Some(form) = FormOwner(c, id) {
                    if let Some(state) = c.default_action_state.FormControl(&c.document, id) {
                        // ImageInputType::ExtractClickLocation, cc:82-92. The
                        // canonical MouseEvent already owns offset conversion.
                        let point = event
                            .canonical
                            .as_ref()
                            .and_then(|event| event.borrow().BaseEvent().UnderlyingEvent())
                            .and_then(|event| {
                                let event = event.borrow();
                                let mouse = event.AsMouseEvent()?;
                                mouse
                                    .HasPosition()
                                    .then(|| (mouse.offsetX() as i32, mouse.offsetY() as i32))
                            })
                            .unwrap_or_default();
                        state.image_point.set(point);
                    }
                    SubmitForm(c, form, Some(id), true);
                    event.default_handled = true;
                }
            }
            _ => {}
        }
    }
}
struct PageButtonActivation<'a, 'b> {
    c: &'a InputTypeContext<'b>,
    node: u64,
}
impl
    interaction_default::core::html::forms::base_button_input_type::ButtonFormActivationOperations<
        u64,
    > for PageButtonActivation<'_, '_>
{
    fn Form(&self) -> Option<u64> {
        FormOwner(self.c, self.node)
    }
    fn IsDisabledFormControl(&self) -> bool {
        IsDisabledFormControl(&self.c.document, self.node)
    }
    fn PrepareForSubmission(&self, form: &u64) {
        SubmitForm(self.c, *form, Some(self.node), true);
    }
    fn ResetForm(&self, form: &u64) {
        ResetForm(*form, self.c);
    }
}
impl interaction_default::core::html::forms::html_button_element::ButtonFormDefaultOperations<u64>
    for PageButtonActivation<'_, '_>
{
    fn ControlType(&self) -> interaction_default::core::html::forms::html_button_element::Type {
        interaction_default::core::html::forms::html_button_element::HTMLButtonElement::TypeForAttributes(&AttributeValue(self.c,self.node,"type"), self.HasCommandOrCommandForAttribute())
    }
    fn TypeAttribute(&self) -> String {
        AttributeValue(self.c, self.node, "type")
    }
    fn HasCommandOrCommandForAttribute(&self) -> bool {
        HasAttribute(&self.c.document, self.node, "command")
            || HasAttribute(&self.c.document, self.node, "commandfor")
    }
}
// cpp: interaction/input_type.cc:197-202
struct CheckableActivationState(ClickHandlingState<u64>);
impl ActivationState for CheckableActivationState {
    fn AsAny(&self) -> &dyn Any {
        self
    }
}
struct CheckableInputType<'a> {
    node: u64,
    radio: bool,
    c: InputTypeContext<'a>,
    // This owner is retained through the complete pre/activation transaction.
    in_click_handler: Cell<bool>,
}
impl CheckboxActivationOperations for CheckableInputType<'_> {
    fn Checked(&self) -> bool {
        Checked(&self.c, self.node)
    }
    fn Indeterminate(&self) -> bool {
        Indeterminate(&self.c, self.node)
    }
    fn SetChecked(&self, checked: bool, _: TextFieldEventBehavior) {
        SetChecked(&self.c, self.node, checked);
    }
    fn SetIndeterminate(&self, value: bool) {
        SetIndeterminate(&self.c, self.node, value);
    }
    fn SetInClickHandler(&self, value: bool) {
        self.in_click_handler.set(value);
    }
    fn SetUserHasEditedTheField(&self) {
        SetUserEdited(&self.c, self.node);
    }
    fn DispatchInputAndChangeEventIfNeeded(&self) {
        DispatchInputAndChange(self.node, &self.c.dispatch_synthetic);
    }
}
impl RadioActivationOperations<u64> for CheckableInputType<'_> {
    fn Checked(&self) -> bool {
        Checked(&self.c, self.node)
    }
    fn CheckedRadioButtonForGroup(&self) -> Option<u64> {
        RadioGroup(&self.c, self.node)
            .into_iter()
            .find(|&id| Checked(&self.c, id))
    }
    fn SetChecked(&self, checked: bool, _: TextFieldEventBehavior) {
        SetChecked(&self.c, self.node, checked);
    }
    fn IsSameRadioGroup(&self, checked: &u64) -> bool {
        RadioGroup(&self.c, self.node).contains(checked)
    }
    fn SetRadioChecked(&self, checked: &u64, _: TextFieldEventBehavior) {
        SetChecked(&self.c, *checked, true);
    }
    fn SetInClickHandler(&self, value: bool) {
        self.in_click_handler.set(value);
    }
    fn CSSUserValidAndUserInvalidForRadioEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::CSSUserValidAndUserInvalidForRadioEnabled()
    }
    fn SetUserHasEditedTheField(&self) {
        SetUserEdited(&self.c, self.node);
    }
    fn DispatchInputAndChangeEventIfNeeded(&self) {
        DispatchInputAndChange(self.node, &self.c.dispatch_synthetic);
    }
}
impl RadioKeyboardOperations<u64> for CheckableInputType<'_> {
    fn Current(&self) -> u64 {
        self.node
    }
    fn HasLayoutObject(&self) -> bool {
        self.c
            .editing_geometry
            .as_ref()
            .is_some_and(|geometry| geometry.HasLayoutObject(self.node))
    }
    fn SetActive(&self, active: bool) {
        self.c.interaction_state.borrow_mut().pressed_node_id = active.then_some(self.node);
    }
    fn IsRightToLeft(&self) -> bool {
        ReadNode(&self.c.document, self.node, |document, _, index| {
            document
                .ResolvedStyleFor(index)
                .and_then(|style| style.ComputedStyle())
                .is_some_and(|style| style.Direction() == foundation::TextDirection::kRtl)
        })
        .unwrap_or(false)
    }
    fn SpatialNavigationEnabled(&self) -> bool {
        false
    }
    fn NextRadioButtonInGroup(&self, current: &u64, forward: bool) -> Option<u64> {
        let group = RadioGroup(&self.c, *current);
        let index = group.iter().position(|id| id == current)?;
        if forward {
            group.get(index + 1).copied()
        } else {
            index
                .checked_sub(1)
                .and_then(|index| group.get(index).copied())
        }
    }
    fn IsFocusable(&self, input: &u64) -> bool {
        !IsDisabledFormControl(&self.c.document, *input)
            && self
                .c
                .editing_geometry
                .as_ref()
                .is_some_and(|geometry| geometry.HasLayoutObject(*input))
            && ReadNode(&self.c.document, *input, |document, _, index| {
                document
                    .ResolvedStyleFor(index)
                    .and_then(|style| style.ComputedStyle())
                    .is_none_or(|style| {
                        style.Visibility() == foundation::style_constants::EVisibility::kVisible
                    })
            })
            .unwrap_or(false)
    }
    fn FocusRestoringSelection(&self, input: &u64) {
        self.c.focus_controller.Focus(*input, true);
    }
    fn DispatchSimulatedClick(&self, input: &u64) {
        let mut click = MakeSyntheticEvent(EventType::kClick, *input);
        click.button = MouseButton::kPrimary;
        (self.c.dispatch_synthetic)(&mut click, *input);
    }
}
impl InputType for CheckableInputType<'_> {
    fn HasActivationBehavior(&self) -> bool {
        true
    }
    fn LegacyPreActivationBehavior(&self, _: &mut Event) -> Option<Box<dyn ActivationState>> {
        let state = if self.radio {
            RadioInputType::LegacyPreActivationWithOperations(self)
        } else {
            CheckboxInputType::LegacyPreActivationWithOperations(self)
        };
        Some(Box::new(CheckableActivationState(state)))
    }
    fn RunActivationBehavior(&self, event: &mut Event, base: Option<&dyn ActivationState>) {
        let Some(state) = base.and_then(|s| s.AsAny().downcast_ref::<CheckableActivationState>())
        else {
            return;
        };
        let canceled = event.default_prevented || event.default_handled;
        if self.radio {
            RadioInputType::RunActivationWithOperations(self, canceled, event.trusted, &state.0);
        } else {
            CheckboxInputType::RunActivationWithOperations(self, canceled, event.trusted, &state.0);
        }
        event.default_handled = true;
    }
    fn HandleDefaultEvent(&self, event: &mut Event) {
        let c = &self.c;
        let id = self.node;
        if self.radio && event.r#type == EventType::kKeyDown {
            let keyboard = CheckableKeyboardEvent {
                key: event.key.clone(),
                char_code: event.char_code as i32,
                ctrl_key: event.modifiers.control,
                meta_key: event.modifiers.meta,
                alt_key: event.modifiers.alt,
            };
            if RadioInputType::HandleKeydownWithOperations(self, &keyboard, event.default_handled) {
                event.default_handled = true;
                return;
            }
        }
        if event.r#type == EventType::kKeyDown && event.key == " " {
            if !DispatchKeyPress(event, id, &c.dispatch_synthetic) {
                return;
            }
            c.interaction_state.borrow_mut().pressed_node_id = Some(id);
            return;
        }
        if event.r#type == EventType::kKeyUp
            && event.key == " "
            && c.interaction_state.borrow().pressed_node_id == Some(id)
        {
            c.interaction_state.borrow_mut().pressed_node_id = None;
            let mut click = MakeSyntheticEvent(EventType::kClick, id);
            click.button = MouseButton::kPrimary;
            (c.dispatch_synthetic)(&mut click, id);
            event.default_handled = true;
        }
    }
}
// cpp: interaction/input_type.cc:292-334
struct TextFieldInputType<'a> {
    node: u64,
    c: InputTypeContext<'a>,
}
impl<'a> InputType for TextFieldInputType<'a> {
    fn HandleDefaultEvent(&self, event: &mut Event) {
        let c = &self.c;
        let id = self.node;
        if event.r#type == EventType::kKeyDown
            && !DispatchKeyPress(event, id, &c.dispatch_synthetic)
        {
            return;
        }
        if event.r#type == EventType::kKeyDown
            && event.key == "Enter"
            && !Element(c, id, "textarea")
        {
            let mut before = MakeSyntheticEvent(EventType::kBeforeInput, id);
            before.input_type = "insertLineBreak".into();
            (c.dispatch_synthetic)(&mut before, id);
            if !before.default_prevented {
                if let Some(form) = FormOwner(c, id) {
                    HTMLFormElement::SubmitImplicitlyWithOperations(
                        &PageImplicitSubmission { c, form },
                        true,
                    );
                }
            }
            event.default_handled = true;
            return;
        }
        // Text insertion, deletion and selection movement are owned by the
        // translated KeyboardEventManager/Editor/InputMethodController path.
        // InputType handles only its element-specific Enter/form behavior.
    }
}
// cpp: interaction/input_type.cc:336-369
struct ButtonInputType<'a> {
    node: u64,
    c: InputTypeContext<'a>,
}
struct PageButtonKeyboard<'a, 'b> {
    c: &'a InputTypeContext<'b>,
    node: u64,
    event: &'a Event,
}
impl interaction_default::core::html::forms::keyboard_clickable_input_type_view::KeyboardClickableOperations for PageButtonKeyboard<'_, '_> {
    fn SetActive(&self, active: bool) {
        let mut state=self.c.interaction_state.borrow_mut();
        if active { state.pressed_node_id=Some(self.node); }
        else if state.pressed_node_id==Some(self.node) { state.pressed_node_id=None; }
    }
    fn IsActive(&self)->bool { self.c.interaction_state.borrow().pressed_node_id==Some(self.node) }
    fn DispatchSimulatedClick(&self) { (self.c.dispatch_simulated_click)(self.node,self.event); }
}
impl interaction_default::core::html::html_element::HTMLElementKeyboardActivationOperations
    for PageButtonKeyboard<'_, '_>
{
    fn SetActive(&self, active: bool) {
        interaction_default::core::html::forms::keyboard_clickable_input_type_view::KeyboardClickableOperations::SetActive(self,active);
    }
    fn IsActive(&self) -> bool {
        interaction_default::core::html::forms::keyboard_clickable_input_type_view::KeyboardClickableOperations::IsActive(self)
    }
    fn DispatchSimulatedClick(&self) {
        (self.c.dispatch_simulated_click)(self.node, self.event);
    }
}
impl InputType for ButtonInputType<'_> {
    fn HasActivationBehavior(&self) -> bool {
        true
    }
    fn HandleDefaultEvent(&self, event: &mut Event) {
        use interaction_default::core::html::{
            forms::keyboard_clickable_input_type_view::KeyboardClickableInputTypeView as View,
            html_element::HTMLElement,
        };
        if event.r#type == EventType::kCustom && event.custom_type == "DOMActivate" {
            ActivateButton(self.node, event, &self.c);
            return;
        }
        let operations = PageButtonKeyboard {
            c: &self.c,
            node: self.node,
            event,
        };
        let handled = if Element(&self.c, self.node, "button") {
            HTMLElement::HandleKeyboardActivationWithOperations(
                crate::event::EventTypeName(event.r#type),
                &event.key,
                event.char_code as i32,
                &operations,
            )
            .default_handled
        } else {
            match event.r#type {
                EventType::kKeyDown => {
                    View::HandleKeydownWithOperations(&event.key, &operations);
                    false
                }
                EventType::kKeyPress => View::HandleKeypressWithOperations(&event.key, &operations),
                EventType::kKeyUp => View::HandleKeyupWithOperations(&event.key, &operations),
                _ => false,
            }
        };
        event.default_handled = handled;
        if event.r#type == EventType::kKeyDown && !handled {
            DispatchKeyPress(event, self.node, &self.c.dispatch_synthetic);
        }
    }
}
// cpp: interaction/input_type.cc:371-404
struct RangeInputType<'a> {
    node: u64,
    c: InputTypeContext<'a>,
}
impl NumberTypeParser<Decimal> for RangeInputType<'_> {
    fn ParseToDecimalForNumberType(&self, value: &str, default: Decimal) -> Decimal {
        self.c
            .default_action_state
            .numbers
            .ParseToDecimalForNumberType(value, default)
    }
    fn ParseToDoubleForNumberType(&self, value: &str) -> f64 {
        self.c
            .default_action_state
            .numbers
            .ParseToDoubleForNumberType(value)
    }
    fn SerializeForNumberType(&self, value: Decimal) -> String {
        self.c
            .default_action_state
            .numbers
            .SerializeForNumberType(value)
    }
}
impl RangeKeyboardOperations<Decimal> for RangeInputType<'_> {
    fn Value(&self) -> String {
        Value(&self.c, self.node)
    }
    fn Attribute(&self, name: &'static str) -> String {
        AttributeValue(&self.c, self.node, name)
    }
    fn FindStepBase(&self, default_base: Decimal) -> Decimal {
        interaction_default::core::html::forms::input_type::FindStepBase(
            &AttributeValue(&self.c, self.node, "min"),
            &AttributeValue(&self.c, self.node, "value"),
            default_base,
            self,
        )
    }
    fn SetValueAsDecimal(&self, value: Decimal, _: TextFieldEventBehavior) {
        SetValue(&self.c, self.node, self.SerializeForNumberType(value));
        (self.c.dispatch_scoped_synthetic)(
            MakeSyntheticEvent(EventType::kInput, self.node),
            self.node,
        );
        (self.c.dispatch_scoped_synthetic)(
            MakeSyntheticEvent(EventType::kChange, self.node),
            self.node,
        );
    }
    fn IsDisabledFormControl(&self) -> bool {
        IsDisabledFormControl(&self.c.document, self.node)
    }
    fn WritingDirection(&self) -> Option<RangeWritingDirection> {
        ReadNode(&self.c.document, self.node, |document, _, index| {
            let style = document.ResolvedStyleFor(index)?.ComputedStyle()?;
            let direction = style.GetWritingDirection();
            let mapper = foundation::PhysicalToLogical::new(
                direction,
                "ArrowUp",
                "ArrowRight",
                "ArrowDown",
                "ArrowLeft",
            );
            Some(RangeWritingDirection {
                inline_end: mapper.InlineEnd(),
                line_over: mapper.LineOver(),
                inline_start: mapper.InlineStart(),
                line_under: mapper.LineUnder(),
                horizontal: direction.IsHorizontal(),
                slider_vertical_appearance: style.EffectiveAppearance()
                    == foundation::AppearanceValue::kSliderVertical,
            })
        })
        .flatten()
    }
    fn NonStandardSliderVerticalEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::NonStandardAppearanceValueSliderVerticalEnabled()
    }
    fn BeginEventQueueScope(&self) -> Box<dyn EventQueueScope + '_> {
        (self.c.begin_event_queue_scope)()
    }
    fn NotifyAccessibilityValueChanged(&self) {
        let host = self.c.default_action_state.select_host.borrow().clone();
        if let Some(client) = host.and_then(|host| host.AccessibilityClient()) {
            let document = self
                .c
                .document
                .borrow()
                .GetDocument()
                .RootHandle()
                .Identity();
            client.Notify(
                document,
                super::page_select_control::SelectAccessibilityNotification::ControlValueChanged {
                    control: self.node,
                },
            );
        }
    }
}
impl InputType for RangeInputType<'_> {
    fn HandleDefaultEvent(&self, event: &mut Event) {
        if event.r#type == EventType::kKeyDown {
            event.default_handled =
                SourceRangeInputType::<Decimal>::HandleKeydownWithOperations(self, &event.key);
        }
    }
}
// cpp: interaction/input_type.cc:406-438
struct SelectInputType<'a> {
    node: u64,
    c: InputTypeContext<'a>,
}
/// The DOM collection is borrowed from its existing retained owner at each
/// operation. This adapter carries identity and transaction capabilities only.
struct PageSelectOperations<'a, 'b> {
    c: &'a InputTypeContext<'b>,
    node: u64,
    state: Rc<ListBoxSelectionState<u64>>,
}
impl PageSelectOperations<'_, '_> {
    fn Host(&self) -> Rc<dyn super::page_select_control::SelectControlHost> {
        self.c
            .default_action_state
            .select_host
            .borrow()
            .clone()
            .expect("current Document select host")
    }
    fn NotifyAccessibility(
        &self,
        notification: super::page_select_control::SelectAccessibilityNotification,
    ) {
        let Some(client) = self.Host().AccessibilityClient() else {
            return;
        };
        let document = self
            .c
            .document
            .borrow()
            .GetDocument()
            .RootHandle()
            .Identity();
        client.Notify(document, notification);
    }
    fn Inner(&self) -> Option<u64> {
        ReadNode(&self.c.document, self.node, |document, _, select| {
            document
                .SelectInnerElement(select)
                .map(|inner| document.Node(inner).Id())
        })
        .flatten()
    }
    fn NotifyMenuActiveOption(&self, option: Option<&u64>) {
        if self.HasLayout(self.node) {
            if let Some(client) = self.Host().AccessibilityClient() {
                let state = self
                    .c
                    .default_action_state
                    .selects
                    .borrow()
                    .get(&self.node)
                    .expect("retained menu-list owner")
                    .menu_active_option
                    .clone();
                let document = self
                    .c
                    .document
                    .borrow()
                    .GetDocument()
                    .RootHandle()
                    .Identity();
                state.Update(self, option, |previous| client.Notify(document, super::page_select_control::SelectAccessibilityNotification::ActiveMenuOptionChanged { select: self.node, previous }));
            }
        }
    }
    fn Collection(&self) -> Option<Rc<dom::persistent_document::DOMSelectCollection>> {
        ReadNode(&self.c.document, self.node, |d, _, select| {
            d.SelectCollection(select)
        })
    }
    fn Mutate(&self, mutation_type: DOMMutationType, id: u64, value: bool) {
        (self.c.emit_mutation)(&DOMMutation {
            mutation_type,
            target_node_id: id,
            bool_value: value,
            ..Default::default()
        });
    }
    fn HasLayout(&self, id: u64) -> bool {
        self.Host().HasLayoutObject(id)
    }
}
impl SelectOptionStateOperations<u64> for PageSelectOperations<'_, '_> {
    fn Same(&self, a: Option<&u64>, b: Option<&u64>) -> bool {
        a == b
    }
    fn ListLen(&self) -> usize {
        self.Collection().map_or(0, |c| c.ListItemNodeIds().len())
    }
    fn ListOption(&self, index: usize) -> Option<u64> {
        let id = *self.Collection()?.ListItemNodeIds().get(index)?;
        ReadNode(&self.c.document, id, |_, n, _| n.IsHTMLElement("option"))
            .unwrap_or(false)
            .then_some(id)
    }
    fn OptionLen(&self) -> usize {
        self.Collection().map_or(0, |c| c.OptionNodeIds().len())
    }
    fn Option(&self, index: usize) -> u64 {
        self.Collection().expect("live select").OptionNodeIds()[index]
    }
    fn ListIndex(&self, option: &u64) -> i32 {
        self.Collection()
            .and_then(|c| c.ListItemNodeIds().iter().position(|id| id == option))
            .map_or(-1, |i| i as i32)
    }
    fn Index(&self, option: &u64) -> i32 {
        self.Collection()
            .and_then(|c| c.OptionNodeIds().iter().position(|id| id == option))
            .map_or(-1, |i| i as i32)
    }
    fn Selected(&self, option: &u64) -> bool {
        ReadNode(&self.c.document, *option, |d, _, i| d.OptionSelected(i)).unwrap_or(false)
    }
    fn Disabled(&self, option: &u64) -> bool {
        IsDisabledFormControl(&self.c.document, *option)
    }
    fn DisplayNone(&self, option: &u64) -> bool {
        ReadNode(&self.c.document, *option, |d, _, i| {
            d.ResolvedStyleFor(i)
                .and_then(|s| s.ComputedStyle())
                .is_some_and(|s| s.Display() == foundation::style_constants::EDisplay::kNone)
        })
        .unwrap_or(true)
    }
    fn OptionHasLayout(&self, option: &u64) -> bool {
        self.HasLayout(*option)
    }
    fn SetSelected(&self, option: &u64, selected: bool, skip_observer: bool) {
        (self.c.emit_mutation)(&DOMMutation {
            mutation_type: DOMMutationType::kSetOptionSelectedState,
            target_node_id: *option,
            bool_value: selected,
            option_skip_mutation_observer: skip_observer,
            ..Default::default()
        });
    }
    fn SetDirty(&self, option: &u64) {
        self.Mutate(DOMMutationType::kSetOptionDirty, *option, true);
    }
    fn IsMultiple(&self) -> bool {
        HasAttribute(&self.c.document, self.node, "multiple")
    }
    fn NotifyFormStateChanged(&self) {
        if self.DocumentIsActive() {
            self.Host().NotifyFormStateChanged(self.node);
        }
    }
    fn UsesMenuList(&self) -> bool {
        ReadNode(&self.c.document, self.node, |d, _, i| {
            d.SelectUsesMenuList(i)
        })
        .unwrap_or(true)
    }
    fn LastOnChangeOption(&self) -> Option<u64> {
        ReadNode(&self.c.document, self.node, |d, _, i| {
            d.SelectLastOnChangeOption(i).map(|o| d.Node(o).Id())
        })
        .flatten()
    }
    fn OptionHasOwnerSelect(&self, option: &u64) -> bool {
        ReadNode(&self.c.document, *option, |d, _, index| {
            d.OptionOwnerSelect(index).is_some()
        })
        .unwrap_or(false)
    }
}
impl SelectKeyboardOperations<u64> for PageSelectOperations<'_, '_> {
    fn SetMultiSelectFocused(&self, option: &u64, focused: bool) {
        let mut state = self.c.interaction_state.borrow_mut();
        if focused {
            state.multi_select_focused_option_id = Some(*option);
        } else if state.multi_select_focused_option_id == Some(*option) {
            state.multi_select_focused_option_id = None;
        }
    }

    fn SpatialNavigationEnabled(&self) -> bool {
        false
    }
    fn BlockDirectionKeys(&self) -> (&'static str, &'static str) {
        ReadNode(&self.c.document, self.node, |d, _, i| {
            let direction = d
                .ResolvedStyleFor(i)?
                .ComputedStyle()?
                .GetWritingDirection();
            let mapper = foundation::PhysicalToLogical::new(
                direction,
                "ArrowUp",
                "ArrowRight",
                "ArrowDown",
                "ArrowLeft",
            );
            Some((mapper.BlockEnd(), mapper.BlockStart()))
        })
        .flatten()
        .unwrap_or(("ArrowDown", "ArrowUp"))
    }
    fn ListBoxSize(&self) -> i32 {
        let size = ReadNode(&self.c.document, self.node, |d, _, i| d.SelectSize(i)).unwrap_or(0);
        if size >= 1 {
            size as i32
        } else {
            4
        }
    }
    fn ScrollToOption(&self, option: Option<&u64>) {
        let Some(option) = option.copied() else {
            return;
        };
        // MenuList inherits SelectType's empty ScrollToOption virtual. The
        // list-box subtype owns the coalesced task and original option identity.
        if self.UsesMenuList() || !self.state.QueueScrollToOption(option) {
            return;
        }
        let state = self.state.clone();
        let host = self.Host();
        let task_host = host.clone();
        let document = Rc::downgrade(&self.c.document);
        let select_owner = ReadNode(&self.c.document, self.node, |document, _, select| {
            document.NodeHandle(select)
        })
        .expect("live select");
        host.PostUserInteractionTask(Box::new(move || {
            let Some(option) = state.TakeScrollOption() else {
                return;
            };
            let Some(document) = document.upgrade() else {
                return;
            };
            let select = {
                let owner = document.borrow();
                let document = owner.GetDocument();
                let Some(select) = document
                    .FindNodeById(select_owner.Id())
                    .filter(|&index| document.NodeHandle(index) == select_owner)
                else {
                    return;
                };
                let Some(option_index) = document.FindNodeById(option) else {
                    return;
                };
                if !document.IsConnectedToDocument(select)
                    || document.SelectUsesMenuList(select)
                    || document.OptionOwnerSelect(option_index) != Some(select)
                {
                    return;
                }
                document.Node(select).Id()
            };
            task_host.ScrollOptionIntoView(select, option);
        }));
    }
    fn IsFinishedParsingChildren(&self) -> bool {
        ReadNode(&self.c.document, self.node, |document, _, select| {
            document.SelectIsFinishedParsingChildren(select)
        })
        .unwrap_or(false)
    }
    fn ListboxActiveIndexChanged(&self) {
        self.NotifyAccessibility(super::page_select_control::SelectAccessibilityNotification::ListboxActiveIndexChanged { select: self.node });
    }
    fn SetNeedsValidityCheck(&self) {
        self.Mutate(
            DOMMutationType::kSetControlNeedsValidityCheck,
            self.node,
            true,
        );
    }
    fn DispatchInputEvent(&self) {
        let mut event = MakeSyntheticEvent(EventType::kInput, self.node);
        (self.c.dispatch_synthetic)(&mut event, self.node);
    }
    fn DispatchChangeEvent(&self) {
        let mut event = MakeSyntheticEvent(EventType::kChange, self.node);
        (self.c.dispatch_synthetic)(&mut event, self.node);
    }
    fn SelectMenuOption(&self, option: u64) {
        HTMLSelectElement::SelectMenuOptionWithOperations(self, option);
    }
    fn SubmitImplicitly(&self) {
        if let Some(form) = FormOwner(self.c, self.node) {
            HTMLFormElement::SubmitImplicitlyWithOperations(
                &PageImplicitSubmission { c: self.c, form },
                false,
            );
        }
    }
}
impl SelectElementKeyboardOperations<u64> for PageSelectOperations<'_, '_> {
    fn HasLayoutObject(&self) -> bool {
        self.HasLayout(self.node)
    }
    fn IsDisabledFormControl(&self) -> bool {
        IsDisabledFormControl(&self.c.document, self.node)
    }
    fn IsAppearanceBase(&self) -> bool {
        ReadNode(&self.c.document, self.node, |d, _, i| {
            d.ResolvedStyleFor(i)
                .and_then(|s| s.ComputedStyle())
                .is_some_and(|s| d.SelectSupportsBaseAppearance(i, s.EffectiveAppearance()))
        })
        .unwrap_or(false)
    }
    fn SetUserHasEditedTheField(&self) {
        SetUserEdited(self.c, self.node);
    }
}
impl SelectOptionOperations<u64> for PageSelectOperations<'_, '_> {
    fn SetSelectionAutofillState(&self, state: u8) {
        (self.c.emit_mutation)(&DOMMutation {
            mutation_type: DOMMutationType::kSetControlAutofillState,
            target_node_id: self.node,
            autofill_state: match state {
                1 => dom::DOMAutofillState::kPreviewed,
                2 => dom::DOMAutofillState::kAutofilled,
                _ => dom::DOMAutofillState::kNotFilled,
            },
            ..Default::default()
        });
    }
    fn SelectedcontentSpecEnabled(&self) -> bool {
        ReadNode(&self.c.document, self.node, |d, _, _| {
            d.SelectSourceFeatures().selectedcontent_spec
        })
        .unwrap_or(false)
    }
    fn UpdateSelectedcontentsSingle(&self, option: Option<&u64>) {
        (self.c.emit_mutation)(&DOMMutation {
            mutation_type: DOMMutationType::kUpdateSelectSelectedcontents,
            target_node_id: self.node,
            child_node_id: option.copied().unwrap_or(0),
            bool_value: false,
            ..Default::default()
        });
    }
    fn UpdateSelectedcontentsMultiple(&self) {
        (self.c.emit_mutation)(&DOMMutation {
            mutation_type: DOMMutationType::kUpdateSelectSelectedcontents,
            target_node_id: self.node,
            bool_value: true,
            ..Default::default()
        });
    }
    fn DidSelectOption(&self, option: Option<u64>, flags: u32, update_popup: bool) {
        if self.UsesMenuList() {
            interaction_default::core::html::forms::select_type::DidSelectMenuOptionWithOperations(
                self,
                option,
                flags,
                update_popup,
            );
        } else {
            self.state.DidSelectOption(self, option, flags);
        }
    }
    fn UpdateOptionMutationObserver(&self, option: &u64) {
        self.Mutate(
            DOMMutationType::kUpdateOptionMutationObserver,
            *option,
            false,
        );
    }
    fn DocumentIsActive(&self) -> bool {
        ReadNode(&self.c.document, self.node, |d, _, _| d.IsActive()).unwrap_or(false)
    }
    fn DidChangeSelectionInSelectControl(&self) {
        self.Host().DidChangeSelectionInSelectControl(self.node);
    }
}
impl interaction_default::core::html::forms::select_type::MenuListSelectionOperations<u64>
    for PageSelectOperations<'_, '_>
{
    fn SetLastOnChangeOption(&self, option: Option<&u64>) {
        SelectResetOperations::SetLastOnChangeOption(self, option);
    }
    fn UpdateTextStyleAndContent(&self) {
        SelectResetOperations::UpdateTextStyleAndContent(self);
    }
    fn NativePopupIsVisible(&self) -> bool {
        ReadNode(&self.c.document, self.node, |document, _, select| {
            document.SelectNativePopupIsVisible(select)
        })
        .unwrap_or(false)
    }
    fn UpdatePopupFromElement(&self) {
        let client = self
            .Host()
            .NativePopupClient()
            .expect("visible native select popup has a native popup owner");
        let document = self
            .c
            .document
            .borrow()
            .GetDocument()
            .RootHandle()
            .Identity();
        client.UpdateFromElement(document, self.node);
    }
    fn HasLayoutObject(&self) -> bool {
        self.HasLayout(self.node)
    }
    fn WillBeDestroyed(&self) -> bool {
        !ReadNode(&self.c.document, self.node, |document, _, select| {
            document.IsActive()
                && document.IsConnectedToDocument(select)
                && document.SelectUsesMenuList(select)
        })
        .unwrap_or(false)
    }
    fn DidUpdateActiveOption(&self, option: Option<&u64>) {
        self.NotifyMenuActiveOption(option);
    }
}
impl SelectResetOperations<u64> for PageSelectOperations<'_, '_> {
    fn DefaultSelected(&self, option: &u64) -> bool {
        HasAttribute(&self.c.document, *option, "selected")
    }
    fn SetDirtyState(&self, option: &u64, dirty: bool) {
        self.Mutate(DOMMutationType::kSetOptionDirty, *option, dirty);
    }
    fn Size(&self) -> u32 {
        ReadNode(&self.c.document, self.node, |d, _, i| d.SelectSize(i)).unwrap_or(0)
    }
    fn SetLastOnChangeOption(&self, option: Option<&u64>) {
        (self.c.emit_mutation)(&DOMMutation {
            mutation_type: DOMMutationType::kSetSelectLastOnChangeOption,
            target_node_id: self.node,
            child_node_id: option.copied().unwrap_or(0),
            ..Default::default()
        });
    }
    fn UpdateTextStyleAndContent(&self) {
        if !self.UsesMenuList() {
            return;
        }
        use interaction_default::core::html::forms::select_type::MenuListTextOperations;
        let cancel = ReadNode(&self.c.document, self.node, |document, _, select| {
            document.SelectIndexToSelectOnCancel(select)
        })
        .unwrap_or(-1);
        let shown = interaction_default::core::html::forms::select_type::MenuListOptionToBeShownWithOperations(self, self, cancel, self.IsAppearanceBase());
        let option_style = ReadNode(&self.c.document, self.node, |document, _, select| {
            document.SelectOptionStyleOwner(select)
        })
        .expect("current select style owner");
        let text = interaction_default::core::html::forms::select_type::UpdateMenuListTextStyleWithOperations(self, self, shown.as_ref(), &option_style, self.IsAppearanceBase());
        self.NotifyMenuActiveOption(shown.as_ref());
        self.SetInnerTextNodeValue(&text);
        if self.HasLayoutBox() {
            self.AccessibilityTextChanged();
        }
    }
    fn SetResetNeedsValidityCheck(&self) {
        self.SetNeedsValidityCheck();
    }
    fn BaseResetImpl(&self) {
        self.Mutate(DOMMutationType::kSetControlUserEdited, self.node, false);
    }
}
impl interaction_default::core::html::forms::select_type::MenuListTextOperations<u64>
    for PageSelectOperations<'_, '_>
{
    fn SelectAutofillPopoverPreviewEnabled(&self) -> bool {
        ReadNode(&self.c.document, self.node, |document, _, _| {
            document
                .SelectSourceFeatures()
                .select_autofill_popover_preview
        })
        .unwrap_or(false)
    }
    fn SuggestedOption(&self) -> Option<u64> {
        ReadNode(&self.c.document, self.node, |document, _, select| {
            document
                .SelectSuggestedOption(select)
                .map(|option| document.Node(option).Id())
        })
        .flatten()
    }
    fn OptionTextIndentedToRespectGroupLabel(&self, option: &u64) -> Rc<[u16]> {
        ReadNode(&self.c.document, *option, |document, _, option| {
            document.OptionTextIndentedToRespectGroupLabel(option)
        })
        .unwrap_or_else(|| Rc::from([]))
    }
    fn OptionStyle(
        &self,
        option: &u64,
    ) -> foundation::Persistent<layoutng_style::style::computed_style::ComputedStyle> {
        ReadNode(&self.c.document, *option, |document, _, option| {
            document
                .ResolvedStyleFor(option)
                .map(|style| style.computed_style_owner.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default()
    }
    fn MultipleOptionsSelectedText(&self, count: usize) -> Rc<[u16]> {
        self.Host().MultipleOptionsSelectedText(count)
    }
    fn InnerStyle(
        &self,
    ) -> foundation::Persistent<layoutng_style::style::computed_style::ComputedStyle> {
        self.Inner()
            .map(|inner| self.OptionStyle(&inner))
            .unwrap_or_default()
    }
    fn InnerHasLayoutObject(&self) -> bool {
        self.Inner()
            .is_some_and(|inner| self.Host().HasLayoutObject(inner))
    }
    fn SetInnerModifiedStyleOutsideStyleRecalc(
        &self,
        style: foundation::Persistent<layoutng_style::style::computed_style::ComputedStyle>,
    ) {
        self.SetInnerComputedStyle(style.clone());
        if let Some(inner) = self.Inner() {
            self.Host()
                .SetInnerModifiedStyleOutsideStyleRecalc(inner, style);
        }
    }
    fn SetInnerComputedStyle(
        &self,
        style: foundation::Persistent<layoutng_style::style::computed_style::ComputedStyle>,
    ) {
        let mut owner = self.c.document.borrow_mut();
        let document = owner.GetDocumentMut();
        if let Some(select) = document.FindNodeById(self.node) {
            document.SetSelectInnerComputedStyle(select, style);
        }
    }
    fn SetInnerTextNodeValue(&self, text: &[u16]) {
        let text_node = self.Inner().and_then(|inner| {
            ReadNode(&self.c.document, inner, |document, node, _| {
                node.Children()
                    .first()
                    .map(|&child| document.Node(child).Id())
            })
            .flatten()
        });
        if let Some(text_node) = text_node {
            (self.c.emit_mutation)(&DOMMutation {
                mutation_type: DOMMutationType::kSetTextContent,
                target_node_id: text_node,
                utf16_value: Some(Rc::from(text)),
                ..Default::default()
            });
        }
    }
    fn HasLayoutBox(&self) -> bool {
        self.Host().HasLayoutBox(self.node)
    }
    fn AccessibilityTextChanged(&self) {
        self.NotifyAccessibility(
            super::page_select_control::SelectAccessibilityNotification::TextChanged {
                select: self.node,
            },
        );
    }
}
// Source HTMLSelectElement's TypeAheadDataSource, cc:1469-1483. List
// indices include optgroup/hr; disabled/non-option entries yield empty text.
impl TypeAheadDataSource for PageSelectOperations<'_, '_> {
    fn IndexOfSelectedOption(&self) -> i32 {
        HTMLSelectElement::TypeAheadSelectedListIndexWithOperations(self)
    }
    fn OptionCount(&self) -> i32 {
        HTMLSelectElement::TypeAheadOptionCountWithOperations(self)
    }
    fn OptionAtIndex(&self, index: i32) -> Rc<[u16]> {
        HTMLSelectElement::TypeAheadOptionAtIndexWithOperations(self, index)
    }
}
impl SelectTypeAheadOperations<u64> for PageSelectOperations<'_, '_> {
    fn TypeAheadCustomizablePopupVisible(&self) -> bool {
        ReadNode(&self.c.document, self.node, |d, _, select| {
            let host_base = d
                .ResolvedStyleFor(select)
                .and_then(|s| s.ComputedStyle())
                .is_some_and(|s| d.SelectSupportsBaseAppearance(select, s.EffectiveAppearance()));
            let computed_picker_support = d
                .SelectPickerElement(select)
                .and_then(|picker| d.ResolvedStyleFor(picker))
                .and_then(|s| s.ComputedStyle())
                .map(|s| d.SelectSupportsBaseAppearance(select, s.EffectiveAppearance()));
            let base_picker = MenuListPopup::IsAppearanceBasePickerWithFacts(
                host_base,
                computed_picker_support,
                d.SelectCachedBasePickerAppearance(select),
            );
            MenuListPopup::PickerIsPopoverWithFacts(
                base_picker,
                self.IsMultiple(),
                cfg!(target_os = "android"),
            ) && d.SelectPopupIsVisible(select)
        })
        .unwrap_or(false)
    }
    fn OptionDisplayLabel(&self, option: &u64) -> Rc<[u16]> {
        ReadNode(&self.c.document, *option, |d, _, i| {
            d.OptionDisplayLabelUTF16(i)
        })
        .unwrap_or_else(|| Rc::from([]))
    }
    fn FocusOptionWithScriptTrigger(&self, option: &u64) {
        self.c.focus_controller.FocusWithScriptTrigger(*option);
    }
    fn TypeAheadListBoxOnChange(&self) {
        if !self.UsesMenuList() {
            self.state.ListBoxOnChange(self);
        }
    }
}
impl InputType for SelectInputType<'_> {
    fn HandleDefaultEvent(&self, event: &mut Event) {
        let multiple_menu = ReadNode(&self.c.document, self.node, |d, _, i| {
            d.SelectUsesMenuList(i)
        })
        .unwrap_or(true);
        let (state, type_ahead) = self.c.default_action_state.Select(self.node, multiple_menu);
        let operations = PageSelectOperations {
            c: &self.c,
            node: self.node,
            state: state.clone(),
        };
        state.ForgetRemovedOptions(&operations);
        let input = SelectEvent {
            kind: match event.r#type {
                EventType::kKeyDown => "keydown",
                EventType::kKeyPress => "keypress",
                _ => return,
            }
            .into(),
            keyboard: true,
            key: event.key.clone(),
            key_code: event.key_code as i32,
            char_code: event.char_code as i32,
            repeat: event.repeat,
            platform_time_us: event.canonical.as_ref().map_or(0, |native| {
                native
                    .borrow()
                    .BaseEvent()
                    .PlatformTimeStamp()
                    .ToInternalValue()
            }),
            shift: event.modifiers.shift,
            ctrl: event.modifiers.control,
            alt: event.modifiers.alt,
            meta: event.modifiers.meta,
            modifiers: i32::from(event.modifiers.shift)
                | i32::from(event.modifiers.control) << 1
                | i32::from(event.modifiers.alt) << 2
                | i32::from(event.modifiers.meta) << 3,
            ..Default::default()
        };
        event.default_handled = HTMLSelectElement::HandleKeyboardWithOperations(
            &operations,
            state.as_ref(),
            type_ahead.as_ref(),
            &input,
        );
        if !event.default_handled && event.r#type == EventType::kKeyDown {
            DispatchKeyPress(event, self.node, &self.c.dispatch_synthetic);
        }
    }
}
pub(crate) fn RefreshSelectTextStyleAndContent(c: &InputTypeContext<'_>, id: u64) {
    let Some(menu) = ReadNode(&c.document, id, |document, node, index| {
        node.IsHTMLElement("select")
            .then(|| document.SelectUsesMenuList(index))
    })
    .flatten() else {
        return;
    };
    let (state, _) = c.default_action_state.Select(id, menu);
    PageSelectOperations { c, node: id, state }.UpdateTextStyleAndContent();
}
pub(crate) fn SelectFinishedParsingChildren(c: &InputTypeContext<'_>, id: u64) {
    let Some(false) = ReadNode(&c.document, id, |document, node, index| {
        node.IsHTMLElement("select")
            .then(|| document.SelectUsesMenuList(index))
    })
    .flatten() else {
        return;
    };
    let (state, _) = c.default_action_state.Select(id, false);
    let owner = PageSelectOperations { c, node: id, state };
    owner.ScrollToOption(owner.SelectedOption().as_ref());
    owner.ListboxActiveIndexChanged();
}
pub(crate) fn ResetSelectControl(c: &InputTypeContext<'_>, id: u64) {
    let menu = ReadNode(&c.document, id, |d, _, i| d.SelectUsesMenuList(i)).unwrap_or(true);
    let (state, _) = c.default_action_state.Select(id, menu);
    let operations = PageSelectOperations { c, node: id, state };
    HTMLSelectElement::ResetWithOperations(&operations);
}

/// HTMLSelectElement::CanSelectAll/SelectAll over the retained list-box owner.
/// Returns false for a menu-list so FrameSelection continues with document
/// selection; a single-select list box is admitted and source SelectAll is a
/// handled no-op.
pub(crate) fn SelectAllSelectControl(c: &InputTypeContext<'_>, id: u64) -> bool {
    let menu = ReadNode(&c.document, id, |d, node, index| {
        node.IsHTMLElement("select")
            .then(|| d.SelectUsesMenuList(index))
    })
    .flatten()
    .unwrap_or(true);
    if menu {
        return false;
    }
    let (state, _) = c.default_action_state.Select(id, false);
    let operations = PageSelectOperations {
        c,
        node: id,
        state: state.clone(),
    };
    if operations.HasLayout(id) {
        state.SelectAll(&operations);
    }
    true
}

/// HTMLSelectElement::StopAutoscroll, cc:1375-1378; ListBoxSelectType's
/// retained change cache is read by the original ListBoxOnChange algorithm.
/// The context is borrowed only while this synchronous release runs.
pub(crate) fn StopSelectControlAutoscroll(c: &InputTypeContext<'_>, id: u64) {
    if !ReadNode(&c.document, id, |d, n, i| {
        n.IsHTMLElement("select") && !d.IsDisabledFormControl(i)
    })
    .unwrap_or(false)
    {
        return;
    }
    let menu = ReadNode(&c.document, id, |d, _, i| d.SelectUsesMenuList(i)).unwrap_or(true);
    if menu {
        return;
    }
    let (state, _) = c.default_action_state.Select(id, false);
    let operations = PageSelectOperations {
        c,
        node: id,
        state: state.clone(),
    };
    state.HandleMouseRelease(&operations);
}
// cpp: interaction/input_type.cc:442-466
pub fn IsDisabledFormControl(document: &InteractionDocument, id: u64) -> bool {
    ReadNode(document, id, |document, _, index| {
        document.IsDisabledFormControl(index)
    })
    .unwrap_or(false)
}
// cpp: interaction/input_type.cc:468-473
pub fn IsTextField(document: &InteractionDocument, id: u64) -> bool {
    ReadNode(document, id, |document, node, index| {
        node.IsHTMLElement("textarea")
            || document
                .InputType(index)
                .is_some_and(dom::persistent_document::DOMInputType::IsTextFieldInputType)
    })
    .unwrap_or(false)
}
// cpp: interaction/input_type.cc:485-503
pub fn CreateInputType<'a>(id: u64, c: InputTypeContext<'a>) -> Box<dyn InputType + 'a> {
    if let Some(temporal) = super::page_form_validation::TemporalInput(id, c.clone()) {
        return temporal;
    }
    if IsTextField(&c.document, id) {
        return Box::new(TextFieldInputType { node: id, c });
    }
    if Element(&c, id, "input") {
        let ty = LowerASCII(&AttributeValue(&c, id, "type"));
        if ty == "checkbox" || ty == "radio" {
            return Box::new(CheckableInputType {
                node: id,
                radio: ty == "radio",
                c,
                in_click_handler: Cell::new(false),
            });
        }
        if ty == "range" {
            return Box::new(RangeInputType { node: id, c });
        }
        if matches!(ty.as_str(), "button" | "submit" | "reset" | "image") {
            return Box::new(ButtonInputType { node: id, c });
        }
    }
    if Element(&c, id, "select") {
        return Box::new(SelectInputType { node: id, c });
    }
    if Element(&c, id, "button") {
        return Box::new(ButtonInputType { node: id, c });
    }
    Box::new(BaseInputType {
        _node: id,
        _context: c,
    })
}

struct PageImplicitSubmission<'a, 'b> {
    c: &'a InputTypeContext<'b>,
    form: u64,
}
impl ImplicitSubmissionOperations<u64> for PageImplicitSubmission<'_, '_> {
    fn ListedElements(&self) -> Box<dyn Iterator<Item = u64> + '_> {
        ListedControls(self.c, self.form)
    }
    fn IsNativeFormControl(&self, control: &u64) -> bool {
        ReadNode(&self.c.document, *control, |document, _, index| {
            document.IsFormControlElement(index)
        })
        .unwrap_or(false)
    }
    fn CanBeSuccessfulSubmitButton(&self, control: &u64) -> bool {
        ReadNode(&self.c.document, *control, |document, _, index| {
            document.CanBeSuccessfulSubmitButton(index)
        })
        .unwrap_or(false)
    }
    fn IsSuccessfulSubmitButton(&self, control: &u64) -> bool {
        !IsDisabledFormControl(&self.c.document, *control)
            && self.CanBeSuccessfulSubmitButton(control)
    }
    fn CanTriggerImplicitSubmission(&self, control: &u64) -> bool {
        ReadNode(&self.c.document, *control, |document, _, index| {
            document
                .InputType(index)
                .is_some_and(dom::persistent_document::DOMInputType::IsTextFieldInputType)
        })
        .unwrap_or(false)
    }
    fn SubmitBehaviorIsEffectivelyDisabled(&self, _: &u64) -> Option<bool> {
        None
    }
    fn DispatchSimulatedClick(&self, control: &u64) {
        let mut click = MakeSyntheticEvent(EventType::kClick, *control);
        click.button = MouseButton::kPrimary;
        (self.c.dispatch_synthetic)(&mut click, *control);
    }
    fn PrepareForSubmission(&self) {
        SubmitForm(self.c, self.form, None, true);
    }
}

struct ListedControlIterator {
    owner: Rc<Vec<u64>>,
    index: usize,
}
impl Iterator for ListedControlIterator {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let result = self.owner.get(self.index).copied();
        self.index += usize::from(result.is_some());
        result
    }
}
fn ListedControls<'a>(
    c: &'a InputTypeContext<'_>,
    form: u64,
) -> Box<dyn Iterator<Item = u64> + 'a> {
    let owner = ReadNode(&c.document, form, |document, _, index| {
        document.ListedFormControlNodeIds(index)
    })
    .unwrap_or_default();
    Box::new(ListedControlIterator { owner, index: 0 })
}
fn EmitControlFlag(
    c: &InputTypeContext<'_>,
    node: u64,
    mutation_type: DOMMutationType,
    value: bool,
) {
    (c.emit_mutation)(&DOMMutation {
        mutation_type,
        target_node_id: node,
        bool_value: value,
        ..Default::default()
    });
}
struct PageFormReset<'a, 'b> {
    c: &'a InputTypeContext<'b>,
    form: u64,
}
impl FormResetOperations<u64> for PageFormReset<'_, '_> {
    fn HasFrame(&self) -> bool {
        true
    }
    fn DispatchReset(
        &self,
    ) -> interaction_dispatch::core::dom::events::event_dispatch_result::DispatchEventResult {
        let mut event = MakeSyntheticEvent(EventType::kReset, self.form);
        (self.c.dispatch_synthetic)(&mut event, self.form);
        use interaction_dispatch::core::dom::events::event_dispatch_result::DispatchEventResult as R;
        if event.default_prevented {
            R::kCanceledByEventHandler
        } else if event.default_handled {
            R::kCanceledByDefaultEventHandler
        } else {
            R::kNotCanceled
        }
    }
    fn ListedElements(&self) -> Box<dyn Iterator<Item = u64> + '_> {
        ListedControls(self.c, self.form)
    }
    fn IsNativeFormControl(&self, control: &u64) -> bool {
        ReadNode(&self.c.document, *control, |document, _, index| {
            document.IsFormControlElement(index)
        })
        .unwrap_or(false)
    }
    fn ResetControl(&self, control: &u64) {
        let class = ReadNode(&self.c.document, *control, |document, _, index| {
            document.FormControlClass(index)
        })
        .flatten();
        use dom::DOMFormControlClass as C;
        let leaf = PageControlReset {
            c: self.c,
            node: *control,
        };
        match class {
            Some(C::Input) => ResetInputElement(&leaf),
            Some(C::TextArea) => HTMLTextAreaElement::ResetWithOperations(&leaf),
            Some(C::Select) => ResetSelectControl(self.c, *control),
            Some(C::Output) => {
                EmitControlFlag(self.c, *control, DOMMutationType::kResetOutput, false)
            }
            _ => {}
        }
    }
    fn IsElementInternals(&self, control: &u64) -> bool {
        ReadNode(&self.c.document, *control, |d, _, i| {
            d.IsFormAssociatedCustomElement(i)
        })
        .unwrap_or(false)
    }
    fn EnqueueFormResetCallback(&self, control: &u64) {
        let mut owner = self.c.document.borrow_mut();
        let d = owner.GetDocumentMut();
        if let Some(i) = d.FindNodeById(*control) {
            d.EnqueueElementInternalsFormReset(i);
        }
    }
    fn CancelActiveTool(&self) {
        let host = self.c.default_action_state.select_host.borrow().clone();
        if let Some(client) = host.and_then(|host| host.FormToolClient()) {
            let document = self
                .c
                .document
                .borrow()
                .GetDocument()
                .RootHandle()
                .Identity();
            client.CancelActiveScriptTool(document, self.form);
        }
    }
    fn HasPage(&self) -> bool {
        self.c.default_action_state.select_host.borrow().is_some()
    }
    fn FormElementReset(&self) {
        if let Some(host) = self.c.default_action_state.select_host.borrow().clone() {
            host.FormElementReset(self.form);
        }
    }
}
struct PageControlReset<'a, 'b> {
    c: &'a InputTypeContext<'b>,
    node: u64,
}
impl PageControlReset<'_, '_> {
    fn BaseReset(&self) {
        EmitControlFlag(
            self.c,
            self.node,
            DOMMutationType::kSetControlUserEdited,
            false,
        );
    }
    fn SetNonDirty(&self, value: Rc<[u16]>) {
        let changed = ReadNode(&self.c.document, self.node, |document, _, index| {
            document.ControlValueUTF16(index).as_ref() != value.as_ref()
        })
        .unwrap_or(false);
        (self.c.emit_mutation)(&DOMMutation {
            mutation_type: DOMMutationType::kSetNonDirtyControlValue,
            target_node_id: self.node,
            utf16_value: Some(value),
            ..Default::default()
        });
        if changed {
            self.c
                .editor
                .DidResetControlValue(&self.c.document, self.node);
        }
    }
}
impl HTMLInputResetOperations for PageControlReset<'_, '_> {
    fn ValueMode(&self) -> ValueMode {
        let input_type = ReadNode(&self.c.document, self.node, |document, _, index| {
            document.InputType(index)
        })
        .flatten()
        .expect("reset native input class");
        let source = SourceType(input_type);
        ValueModeForType(source)
    }
    fn DefaultValue(&self) -> foundation::NullableUTF16String {
        foundation::NullableUTF16String::FromAttribute(
            ReadNode(&self.c.document, self.node, |document, _, index| {
                document.InputValueAttributeUTF16(index)
            })
            .flatten(),
        )
    }
    fn SetNonDirtyValue(&self, value: &foundation::NullableUTF16String) {
        self.SetNonDirty(value.Owner());
    }
    fn SetNeedsValidityCheck(&self) {
        EmitControlFlag(
            self.c,
            self.node,
            DOMMutationType::kSetControlNeedsValidityCheck,
            false,
        );
    }
    fn SetCheckedFromDefaultAttribute(&self) {
        SetChecked(
            self.c,
            self.node,
            HasAttribute(&self.c.document, self.node, "checked"),
        );
    }
    fn ClearDirtyCheckedness(&self) {
        EmitControlFlag(
            self.c,
            self.node,
            DOMMutationType::kSetControlCheckednessDirty,
            false,
        );
    }
    fn BaseResetImpl(&self) {
        self.BaseReset();
    }
}
impl TextAreaResetOperations for PageControlReset<'_, '_> {
    fn DefaultValue(&self) -> Rc<[u16]> {
        ReadNode(&self.c.document, self.node, |document, _, index| {
            document.TextContentUTF16(index)
        })
        .unwrap_or_else(|| Rc::from([]))
    }
    fn SetNonDirtyValue(&self, value: &[u16], _: TextControlSetValueSelection) {
        self.SetNonDirty(Rc::from(value));
    }
    fn BaseResetImpl(&self) {
        self.BaseReset();
    }
}

pub(super) fn SourceType(input_type: dom::persistent_document::DOMInputType) -> SourceInputType {
    use dom::persistent_document::DOMInputType as D;
    match input_type {
        D::kButton => SourceInputType::kButton,
        D::kColor => SourceInputType::kColor,
        D::kFile => SourceInputType::kFile,
        D::kHidden => SourceInputType::kHidden,
        D::kImage => SourceInputType::kImage,
        D::kNumber => SourceInputType::kNumber,
        D::kRange => SourceInputType::kRange,
        D::kReset => SourceInputType::kReset,
        D::kSubmit => SourceInputType::kSubmit,
        D::kRadio => SourceInputType::kRadio,
        D::kCheckbox => SourceInputType::kCheckbox,
        D::kDate => SourceInputType::kDate,
        D::kDateTimeLocal => SourceInputType::kDateTimeLocal,
        D::kMonth => SourceInputType::kMonth,
        D::kTime => SourceInputType::kTime,
        D::kWeek => SourceInputType::kWeek,
        D::kEmail => SourceInputType::kEmail,
        D::kPassword => SourceInputType::kPassword,
        D::kSearch => SourceInputType::kSearch,
        D::kTelephone => SourceInputType::kTelephone,
        D::kURL => SourceInputType::kURL,
        D::kText => SourceInputType::kText,
    }
}

/// Missing source payload/lifecycle input; no fallback data is synthesized.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormDataAdmission {
    NotAForm { node: u64 },
    NeedsFormDataLayout { control: u64 },
    NeedsFileList { control: u64 },
    NeedsImageCoordinates { control: u64 },
    StaleSubmission { form: u64 },
    NeedsValidation(super::page_form_validation::ValidationRequired),
}
/// A source form submission's stable identity across lifecycle preparation.
/// Values remain on the live DOM until the entry-list append stage.
#[derive(Clone)]
pub struct FormSubmissionToken {
    identity: dom::persistent_document::DOMOwnerHandle,
    document_token: [u64; 2],
    require_active_connected: bool,
    task_token:
        Option<interaction_default::core::html::forms::html_form_element::FormSubmissionTaskToken>,
    form: u64,
    submitter: Option<u64>,
    encoding: Rc<str>,
    state: Rc<FormSubmissionState>,
    submission_scope: Option<
        Rc<interaction_default::core::html::forms::html_form_element::OwnedFormSubmissionScope>,
    >,
}
impl FormSubmissionToken {
    pub(crate) fn SubmissionScope(
        &self,
    ) -> Option<
        Rc<interaction_default::core::html::forms::html_form_element::OwnedFormSubmissionScope>,
    > {
        self.submission_scope.clone()
    }
    pub fn Scheduling(&self) -> Option<super::interaction_contract::FormSubmissionScheduling> {
        Some(super::interaction_contract::FormSubmissionScheduling {
            task: self.task_token.clone()?,
            form_owner: self.identity,
            document_token: self.document_token,
        })
    }
    pub fn Form(&self) -> u64 {
        self.form
    }
    pub fn Submitter(&self) -> Option<u64> {
        self.submitter
    }
    pub fn Encoding(&self) -> &str {
        &self.encoding
    }
    fn IsCurrent(&self, c: &InputTypeContext<'_>) -> bool {
        ReadNode(&c.document, self.form, |document, node, index| {
            document.Token() == self.document_token
                && document.NodeHandle(index) == self.identity
                && IsElement(node, "form")
                && (!self.require_active_connected
                    || (document.IsActive() && document.IsConnectedToDocument(index)))
        })
        .unwrap_or(false)
    }
}
impl std::fmt::Debug for FormSubmissionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormSubmissionToken")
            .field("form", &self.form)
            .field("submitter", &self.submitter)
            .finish()
    }
}
/// Single-use entry-list continuation. Clones retain one source transaction;
/// only its first Finish can consume the mutable formdata owner.
#[derive(Clone)]
pub struct PendingFormEntryList {
    token: FormSubmissionToken,
    entries: Rc<
        RefCell<
            Option<interaction_default::core::html::forms::html_form_element::PendingFormEntryList>,
        >,
    >,
}
impl PendingFormEntryList {
    pub fn Token(&self) -> &FormSubmissionToken {
        &self.token
    }
    pub fn Form(&self) -> u64 {
        self.token.Form()
    }
    pub fn Submitter(&self) -> Option<u64> {
        self.token.Submitter()
    }
}
impl std::fmt::Debug for PendingFormEntryList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingFormEntryList")
            .field("token", &self.token)
            .finish()
    }
}
#[derive(Clone, Debug)]
pub enum PrepareFormEntryListResult {
    Ready(PendingFormEntryList),
    NeedsLifecycle(FormSubmissionToken),
}
/// FormData::Create validates its optional submitter without preparing or
/// dispatching a form submission. Both paths share the source entry loop.
pub(crate) fn PrepareFormData(
    c: &InputTypeContext<'_>,
    form: u64,
    submitter: Option<u64>,
) -> Result<
    PrepareFormEntryListResult,
    interaction_dispatch::core::fileapi::form_data_object::FormDataConstructionError,
> {
    use interaction_dispatch::core::fileapi::form_data_object::FormDataConstructionError as E;
    if !ReadNode(&c.document, form, |_, node, _| IsElement(node, "form")).unwrap_or(false) {
        return Err(E::TypeError("The specified element is not a form element."));
    }
    let operations = PageRequestSubmit { c, form };
    if let Some(submitter) = submitter {
        if !operations.IsNativeFormControl(&submitter)
            || !operations.CanBeSuccessfulSubmitButton(&submitter)
        {
            return Err(E::TypeError(
                "The specified element is not a submit button.",
            ));
        }
        if !operations.IsOwnedByForm(&submitter) {
            return Err(E::NotFound(
                "The specified element is not owned by this form element.",
            ));
        }
    }
    PrepareFormEntryListForPurpose(c, form, submitter, "UTF-8", false)
        .map_err(FormDataConstructionErrorFor)?
        .ok_or(E::InvalidState("The form is constructing entry list."))
}
pub fn FormDataConstructionErrorFor(
    error: FormDataAdmission,
) -> interaction_dispatch::core::fileapi::form_data_object::FormDataConstructionError {
    use interaction_dispatch::core::fileapi::form_data_object::FormDataConstructionError as E;
    match error {
        FormDataAdmission::NotAForm { .. } => {
            E::TypeError("The specified element is not a form element.")
        }
        FormDataAdmission::StaleSubmission { .. } => {
            E::InvalidState("The form's Document changed during entry construction.")
        }
        FormDataAdmission::NeedsFormDataLayout { .. } => {
            E::InvalidState("FormData requires current form layout.")
        }
        FormDataAdmission::NeedsFileList { .. } => {
            E::InvalidState("FormData requires the input's FileList.")
        }
        FormDataAdmission::NeedsImageCoordinates { .. } => {
            E::InvalidState("FormData requires the image submitter's coordinates.")
        }
        FormDataAdmission::NeedsValidation(_) => {
            E::InvalidState("FormData entry construction cannot require constraint validation.")
        }
    }
}
pub(crate) fn PrepareFormEntryList(
    c: &InputTypeContext<'_>,
    form: u64,
    submitter: Option<u64>,
    encoding: &str,
) -> Result<Option<PrepareFormEntryListResult>, FormDataAdmission> {
    PrepareFormEntryListForPurpose(c, form, submitter, encoding, true)
}
fn PrepareFormEntryListForPurpose(
    c: &InputTypeContext<'_>,
    form: u64,
    submitter: Option<u64>,
    encoding: &str,
    require_active_connected: bool,
) -> Result<Option<PrepareFormEntryListResult>, FormDataAdmission> {
    let state = c
        .default_action_state
        .FormSubmission(&c.document, form)
        .ok_or(FormDataAdmission::NotAForm { node: form })?;
    let (identity, document_token) = ReadNode(&c.document, form, |document, _, index| {
        (document.NodeHandle(index), document.Token())
    })
    .ok_or(FormDataAdmission::NotAForm { node: form })?;
    ResumeFormEntryList(
        c,
        FormSubmissionToken {
            identity,
            document_token,
            require_active_connected,
            task_token: None,
            form,
            submitter,
            encoding: Rc::from(encoding),
            state,
            submission_scope: None,
        },
    )
}
pub(crate) fn ResumeFormEntryList(
    c: &InputTypeContext<'_>,
    token: FormSubmissionToken,
) -> Result<Option<PrepareFormEntryListResult>, FormDataAdmission> {
    if !token.IsCurrent(c) {
        return Err(FormDataAdmission::StaleSubmission { form: token.form });
    }
    // The embedding updates lifecycle outside the Interaction/Layout borrow.
    // This preflight precedes activation and entry-list side effects.
    let geometry_current = c.editing_geometry.is_some()
        && c.editing_geometry_revision
            == Some(c.document.borrow().GetMeasurementGeometryRevision())
        && {
            let owner = c.document.borrow();
            let style = owner.GetDocument().StyleState();
            !style.all_dirty
                && style.changes.is_empty()
                && style.geometry_input_nodes.is_empty()
                && style.editing_input_nodes.is_empty()
        };
    if !geometry_current
        && ListedControls(c, token.form).any(|control| {
            ReadNode(&c.document, control, |document, node, index| {
                document.FormControlClass(index) == Some(dom::DOMFormControlClass::TextArea)
                    && !document.IsDisabledFormControl(index)
                    && node
                        .AttributeValueUTF16("name")
                        .is_some_and(|name| !name.is_empty())
            })
            .unwrap_or(false)
        })
    {
        return Ok(Some(PrepareFormEntryListResult::NeedsLifecycle(token)));
    }
    let activated = token
        .submitter
        .and_then(|control| c.default_action_state.FormControl(&c.document, control))
        .map(|state| state.activated.clone());
    let entries = HTMLFormElement::PrepareEntryListWithOperations(
        token.state.clone(),
        activated,
        &token.encoding,
        &PageFormEntries {
            c,
            form: token.form,
        },
    )?;
    Ok(entries.map(|entries| {
        PrepareFormEntryListResult::Ready(PendingFormEntryList {
            token,
            entries: Rc::new(RefCell::new(Some(entries))),
        })
    }))
}
pub(crate) fn FinishEntryList(
    c: &InputTypeContext<'_>,
    pending: PendingFormEntryList,
) -> Result<Rc<interaction_dispatch::core::fileapi::form_data::FormData>, FormDataAdmission> {
    let entries =
        pending
            .entries
            .borrow_mut()
            .take()
            .ok_or(FormDataAdmission::StaleSubmission {
                form: pending.token.form,
            })?;
    if !pending.token.IsCurrent(c) {
        return Err(FormDataAdmission::StaleSubmission {
            form: pending.token.form,
        });
    }
    let data = entries.Finish(&PageFormEntries {
        c,
        form: pending.token.form,
    });
    if !pending.token.IsCurrent(c) {
        return Err(FormDataAdmission::StaleSubmission {
            form: pending.token.form,
        });
    }
    Ok(data)
}
/// Source ConstructEntryList with no submitter, used by form.submit(). The
/// returned owner is the one exposed synchronously to formdata listeners.
pub(crate) fn ConstructFormEntryList(
    c: &InputTypeContext<'_>,
    form: u64,
    encoding: &str,
) -> Result<
    Option<Rc<interaction_dispatch::core::fileapi::form_data_object::FormDataObject>>,
    FormDataAdmission,
> {
    let state = c
        .default_action_state
        .FormSubmission(&c.document, form)
        .ok_or(FormDataAdmission::NotAForm { node: form })?;
    HTMLFormElement::ConstructEntryListWithOperations(
        state.EntryList(),
        None::<u64>,
        encoding,
        &PageFormEntries { c, form },
    )
}
struct PageFormEntries<'a, 'b> {
    c: &'a InputTypeContext<'b>,
    form: u64,
}
struct PageControlFormData<'a, 'b> {
    c: &'a InputTypeContext<'b>,
    node: u64,
}
impl PageControlFormData<'_, '_> {
    fn Attribute(&self, name: &str) -> Rc<[u16]> {
        ReadNode(&self.c.document, self.node, |_, node, _| {
            node.AttributeValueUTF16(name)
        })
        .flatten()
        .unwrap_or_default()
    }
    fn Direction(&self) -> Option<(Rc<[u16]>, Rc<[u16]>)> {
        ReadNode(&self.c.document, self.node, |document, node, index| {
            node.AttributeValueUTF16("dirname")
                .map(|name| (name, document.TextControlDirectionForFormData(index)))
        })
        .flatten()
    }
}
impl interaction_default::core::html::forms::input_type::InputFormDataOperations
    for PageControlFormData<'_, '_>
{
    fn NameForFormData(&self) -> Rc<[u16]> {
        self.Attribute("name")
    }
    fn ValueForFormData(&self) -> Rc<[u16]> {
        ReadNode(&self.c.document, self.node, |document, _, index| {
            document.ControlValueUTF16(index)
        })
        .unwrap_or_default()
    }
    fn ValueOrDefaultLabelForFormData(&self) -> Rc<[u16]> {
        ReadNode(&self.c.document, self.node, |document, _, index| {
            document.InputValueOrDefaultLabelUTF16(index)
        })
        .unwrap_or_default()
    }
    fn FilesForFormData(
        &self,
    ) -> Option<Rc<interaction_dispatch::core::fileapi::file_list::FileList>> {
        ReadNode(&self.c.document, self.node, |document, _, index| {
            document.InputFiles(index)
        })
        .flatten()
    }
    fn ImageCoordinatesForFormData(&self) -> Option<(i32, i32)> {
        Some(
            self.c
                .default_action_state
                .FormControl(&self.c.document, self.node)?
                .image_point
                .get(),
        )
    }
}
impl FormEntryListOperations<u64> for PageFormEntries<'_, '_> {
    type Error = FormDataAdmission;
    type Submitter = u64;
    fn ListedElements(&self) -> Box<dyn Iterator<Item = u64> + '_> {
        ListedControls(self.c, self.form)
    }
    fn SetActivatedSubmit(&self, control: &u64, value: bool) {
        if let Some(state) = self
            .c
            .default_action_state
            .FormControl(&self.c.document, *control)
        {
            state.activated.set(value);
        }
    }
    fn IsDisabledForFormData(&self, node: &u64) -> bool {
        ReadNode(&self.c.document, *node, |d, _, i| {
            d.IsDisabledFormControl(i)
        })
        .unwrap_or(true)
    }
    fn IsNonEmptyPasswordControl(&self, node: &u64) -> bool {
        ReadNode(&self.c.document, *node, |d, _, i| {
            d.InputType(i) == Some(dom::persistent_document::DOMInputType::kPassword)
                && !d.ControlValueUTF16(i).is_empty()
        })
        .unwrap_or(false)
    }
    fn AppendToFormData(
        &self,
        node: &u64,
        data: &mut interaction_dispatch::core::fileapi::form_data::FormData,
    ) -> Result<(), Self::Error> {
        use interaction_default::core::html::forms::input_type::{
            FormDataAppendAdmission, InputFormDataOperations, InputType as SourceInput,
        };
        let control = PageControlFormData {
            c: self.c,
            node: *node,
        };
        if ReadNode(&self.c.document, *node, |d, _, i| {
            if !d.IsFormAssociatedCustomElement(i) {
                return false;
            }
            d.AppendElementInternalsToFormData(i, data);
            true
        })
        .unwrap_or(false)
        {
            return Ok(());
        }
        let class = ReadNode(&self.c.document, *node, |d, _, i| d.FormControlClass(i)).flatten();
        match class {
            Some(dom::DOMFormControlClass::Input) => {
                let kind = ReadNode(&self.c.document, *node, |d, _, i| d.InputType(i))
                    .flatten()
                    .expect("native input type");
                let kind = SourceType(kind);
                let direction = if SourceInput::IsAutoDirectionalityFormAssociatedForType(kind) {
                    control.Direction()
                } else {
                    None
                };
                SourceInput::AppendSupportedWithOperations(
                    kind,
                    &control,
                    Checked(self.c, *node),
                    self.c
                        .default_action_state
                        .FormControl(&self.c.document, *node)
                        .is_some_and(|state| state.activated.get()),
                    direction,
                    data,
                )
                .map_err(|gap| match gap {
                    FormDataAppendAdmission::FileList => {
                        FormDataAdmission::NeedsFileList { control: *node }
                    }
                    FormDataAppendAdmission::ImageCoordinates => {
                        FormDataAdmission::NeedsImageCoordinates { control: *node }
                    }
                })
            }
            Some(dom::DOMFormControlClass::Button) => {
                use interaction_default::core::html::forms::html_button_element::HTMLButtonElement;
                HTMLButtonElement::AppendWithOperations(
                    HTMLButtonElement::TypeForAttributes(
                        &AttributeValue(self.c, *node, "type"),
                        HasAttribute(&self.c.document, *node, "command"),
                    ),
                    self.c
                        .default_action_state
                        .FormControl(&self.c.document, *node)
                        .is_some_and(|state| state.activated.get()),
                    control.Attribute("name"),
                    control.Attribute("value"),
                    data,
                );
                Ok(())
            }
            Some(dom::DOMFormControlClass::Select) => {
                let collection = ReadNode(&self.c.document, *node, |d, _, i| d.SelectCollection(i))
                    .expect("native select");
                HTMLSelectElement::AppendWithOperations(
                    &PageSelectFormData {
                        control,
                        collection,
                    },
                    data,
                );
                Ok(())
            }
            Some(dom::DOMFormControlClass::TextArea) => {
                HTMLTextAreaElement::AppendWithOperations(&control, data)
            }
            // Source ListedElement::AppendToFormData is empty for these
            // non-submittable built-ins (listed_element.h:92).
            Some(
                dom::DOMFormControlClass::Fieldset
                | dom::DOMFormControlClass::Output
                | dom::DOMFormControlClass::Object,
            )
            | None => Ok(()),
        }
    }
    fn DispatchFormData(
        &self,
        data: Rc<interaction_dispatch::core::fileapi::form_data_object::FormDataObject>,
    ) {
        (self.c.dispatch_form_data)(self.form, data);
    }
}
struct PageSelectFormData<'a, 'b> {
    control: PageControlFormData<'a, 'b>,
    collection: Rc<dom::DOMSelectCollection>,
}
impl interaction_default::core::html::forms::html_select_element::SelectFormDataOperations
    for PageSelectFormData<'_, '_>
{
    type Option = u64;
    fn NameForFormData(&self) -> Rc<[u16]> {
        self.control.Attribute("name")
    }
    fn OptionLen(&self) -> usize {
        self.collection.OptionNodeIds().len()
    }
    fn Option(&self, index: usize) -> u64 {
        self.collection.OptionNodeIds()[index]
    }
    fn Selected(&self, option: &u64) -> bool {
        ReadNode(&self.control.c.document, *option, |d, _, i| {
            d.OptionSelected(i)
        })
        .unwrap_or(false)
    }
    fn IsDisabled(&self, option: &u64) -> bool {
        ReadNode(&self.control.c.document, *option, |d, _, i| {
            d.IsDisabledFormControl(i)
        })
        .unwrap_or(true)
    }
    fn ValueForFormData(&self, option: &u64) -> Rc<[u16]> {
        ReadNode(&self.control.c.document, *option, |d, _, i| {
            d.OptionValueUTF16(i)
        })
        .unwrap_or_default()
    }
}
impl interaction_default::core::html::forms::html_text_area_element::TextAreaFormDataOperations
    for PageControlFormData<'_, '_>
{
    type Error = FormDataAdmission;
    fn NameForFormData(&self) -> Rc<[u16]> {
        self.Attribute("name")
    }
    fn UpdateStyleAndLayoutForForm(&self) -> Result<(), Self::Error> {
        self.c
            .editing_geometry
            .ok_or(FormDataAdmission::NeedsFormDataLayout { control: self.node })?
            .AssertPrepared();
        Ok(())
    }
    fn WrapAttribute(&self) -> Rc<[u16]> {
        self.Attribute("wrap")
    }
    fn ValueWithHardLineBreaks(&self) -> Result<Rc<[u16]>, Self::Error> {
        let geometry = self
            .c
            .editing_geometry
            .ok_or(FormDataAdmission::NeedsFormDataLayout { control: self.node })?;
        let editor = ReadNode(&self.c.document, self.node, |document, _, index| {
            document
                .TextControlInnerEditorElement(index)
                .map(|editor| document.Node(editor).Id())
        })
        .flatten();
        Ok(editor
            .and_then(|editor| geometry.Output().TextControlValueWithHardLineBreaks(editor))
            .unwrap_or_else(|| self.Value()))
    }
    fn Value(&self) -> Rc<[u16]> {
        interaction_default::core::html::forms::input_type::InputFormDataOperations::ValueForFormData(self)
    }
    fn DirnameAndDirectionForFormData(&self) -> Option<(Rc<[u16]>, Rc<[u16]>)> {
        self.Direction()
    }
}

impl interaction_default::core::html::forms::html_select_element::SelectFormStateOperations<u64>
    for PageSelectOperations<'_, '_>
{
    fn ValueForFormState(&self, option: &u64) -> Rc<[u16]> {
        ReadNode(&self.c.document, *option, |document, _, index| {
            document.OptionValueUTF16(index)
        })
        .expect("resident form-state option")
    }
}
/// HTMLInputElement/HTMLTextAreaElement/HTMLSelectElement source virtuals.
/// History reads the same canonical value/selectedness owners as script and
/// default actions; no entry-list or CSSOM serialization is involved.
pub(crate) fn SaveFormControlState(c: &InputTypeContext<'_>, node: u64) -> Vec<Rc<[u16]>> {
    if let Some(state) = ReadNode(&c.document, node, |document, _, index| {
        document
            .IsFormAssociatedCustomElement(index)
            .then(|| document.SaveElementInternalsFormControlState(index))
    })
    .flatten()
    {
        return state;
    }
    if let Some(fields) = super::page_form_validation::SaveTemporalFields(c, node) {
        return fields;
    }
    use interaction_default::core::html::forms::{
        base_checkable_input_type::BaseCheckableInputType, file_input_type,
        input_type_view::InputTypeView,
    };
    let class = ReadNode(&c.document, node, |document, _, index| {
        document.FormControlClass(index)
    })
    .flatten();
    match class {
        Some(dom::DOMFormControlClass::Select) => {
            let menu = ReadNode(&c.document, node, |document, _, index| {
                document.SelectUsesMenuList(index)
            })
            .expect("live select");
            let (state, _) = c.default_action_state.Select(node, menu);
            HTMLSelectElement::SaveFormControlStateWithOperations(&PageSelectOperations {
                c,
                node,
                state,
            })
        }
        Some(dom::DOMFormControlClass::TextArea) => {
            ReadNode(&c.document, node, |document, _, index| {
                HTMLTextAreaElement::SaveFormControlStateWithValue(
                    document.HasDirtyControlValue(index),
                    document.ControlValueUTF16(index),
                )
            })
            .expect("live textarea")
        }
        Some(dom::DOMFormControlClass::Input) => {
            ReadNode(&c.document, node, |document, _, index| {
                match document.InputType(index).expect("input type owner") {
                    dom::DOMInputType::kRadio | dom::DOMInputType::kCheckbox => {
                        vec![BaseCheckableInputType::SaveFormControlStateWithChecked(
                            document.ControlChecked(index),
                        )
                        .encode_utf16()
                        .collect::<Vec<_>>()
                        .into()]
                    }
                    dom::DOMInputType::kFile => file_input_type::SaveFormControlState(
                        document
                            .InputFiles(index)
                            .expect("file list owner")
                            .as_ref(),
                        document.FormControllerDropReferencedFilePaths(),
                    ),
                    _ => InputTypeView::SaveFormControlStateWithValues(
                        document.ControlValueUTF16(index),
                        document
                            .InputValueAttributeUTF16(index)
                            .unwrap_or_else(|| Rc::from([])),
                    ),
                }
            })
            .expect("live input")
        }
        _ => unreachable!("FormController only enumerates source stateful classes"),
    }
}
pub(crate) fn RestoreFormControlState(
    c: &InputTypeContext<'_>,
    node: u64,
    state: &[Rc<[u16]>],
    reader: Option<std::sync::Arc<dyn fileapi::NativeFileReader>>,
) -> Result<(), navigation_history::FormStateRestoreError> {
    if ReadNode(&c.document, node, |document, _, index| {
        document.IsFormAssociatedCustomElement(index)
    })
    .unwrap_or(false)
    {
        let (value, state) = dom::Document::DecodeElementInternalsFormControlState(state, reader)?;
        let mut owner = c.document.borrow_mut();
        let document = owner.GetDocumentMut();
        if let Some(index) = document.FindNodeById(node) {
            document.RestoreElementInternalsFormControlState(index, value, state);
        }
        return Ok(());
    }
    if super::page_form_validation::RestoreTemporalFields(c, node, state) {
        return Ok(());
    }
    let class = ReadNode(&c.document, node, |document, _, index| {
        document.FormControlClass(index)
    })
    .flatten();
    match class {
        Some(dom::DOMFormControlClass::Select) => {
            let menu = ReadNode(&c.document, node, |document, _, index| {
                document.SelectUsesMenuList(index)
            })
            .expect("live select");
            let (selected, _) = c.default_action_state.Select(node, menu);
            HTMLSelectElement::RestoreFormControlStateWithOperations(
                &PageSelectOperations {
                    c,
                    node,
                    state: selected,
                },
                state,
            );
        }
        Some(dom::DOMFormControlClass::Input)
            if ReadNode(&c.document, node, |document, _, index| {
                document.InputType(index) == Some(dom::DOMInputType::kFile)
            })
            .unwrap_or(false) =>
        {
            let reader = reader
                .ok_or(navigation_history::FormStateRestoreError::NativeFileReaderUnavailable)?;
            if let Some(files) =
                interaction_default::core::html::forms::file_input_type::RestoreFormControlState(
                    state, reader,
                )?
            {
                let mut owner = c.document.borrow_mut();
                let document = owner.GetDocumentMut();
                let index = document.FindNodeById(node).expect("live file input");
                document.AssignInputFiles(index, files);
            }
            dom::attribute_lifecycle::DrainAttributeLifecycle(&c.document);
        }
        Some(dom::DOMFormControlClass::Input)
            if ReadNode(&c.document, node, |document, _, index| {
                document.InputType(index).is_some_and(|kind| {
                    matches!(
                        kind,
                        dom::DOMInputType::kRadio | dom::DOMInputType::kCheckbox
                    )
                })
            })
            .unwrap_or(false) =>
        {
            SetChecked(c, node, state[0].as_ref() == [111, 110]);
        }
        Some(dom::DOMFormControlClass::Input | dom::DOMFormControlClass::TextArea) => {
            (c.emit_mutation)(&DOMMutation {
                mutation_type: DOMMutationType::kSetControlValue,
                target_node_id: node,
                utf16_value: Some(state[0].clone()),
                ..Default::default()
            });
            c.editor.DidResetControlValue(&c.document, node);
        }
        _ => {}
    }
    Ok(())
}
