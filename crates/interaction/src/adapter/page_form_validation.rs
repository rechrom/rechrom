//! Live DOM projection into Chromium's ListedElement and InputType validity.
//! No value, checkedness, custom error or validity result is owned by bindings.
use super::temporal_input_locale::TemporalFieldLayout;
use crate::{
    event::{EventType, MakeSyntheticEvent},
    input_type::{DefaultActionState, InputTypeContext},
    ownership::{InteractionDocument, ReadNode},
};
use dom::{DOMFormControlClass as C, DOMInputType as T};
pub use input_validation::{ValidationPatternEngine, ValidationRequired};
pub use interaction_default::core::html::forms::constraint_validation_message::{
    ConstraintValidationLocale, ConstraintValidationText,
};
pub use interaction_default::core::html::forms::temporal_input_type::DateTimeField;
use interaction_default::core::html::forms::{
    base_text_input_type::{NeedsToCheckDirtyFlag::kCheckDirtyFlag, PatternCache},
    html_form_element::{FormControlValidity, FormValidationOperations, HTMLFormElement},
    html_text_area_element::HTMLTextAreaElement,
    input_validation::{self, InputValidationFacts},
    listed_element::{self, ListedElementValidationOperations, ValidityState},
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};
pub enum ValidationMessageEffect {
    Show {
        anchor: u64,
        message: Rc<[u16]>,
        sub_message: Rc<[u16]>,
    },
    Hide {
        anchor: u64,
    },
}
pub trait ValidationMessageClient {
    fn Apply(&self, effect: ValidationMessageEffect) -> Result<(), ValidationRequired>;
}
pub(crate) struct FormValidationState {
    locale: RefCell<Rc<dyn ConstraintValidationLocale>>,
    temporal_locale: RefCell<Option<Rc<dyn TemporalInputLocale>>>,
    engine: RefCell<Option<Rc<dyn ValidationPatternEngine>>>,
    input_values: RefCell<Option<std::rc::Weak<dyn dom::input_value::DOMInputValueSanitizer>>>,
    client: RefCell<Option<Rc<dyn ValidationMessageClient>>>,
    message_anchors: RefCell<HashSet<dom::persistent_document::DOMOwnerHandle>>,
    temporal: RefCell<
        HashMap<
            dom::persistent_document::DOMOwnerHandle,
            Rc<
                RefCell<
                    interaction_default::core::html::forms::temporal_input_type::TemporalInputType,
                >,
            >,
        >,
    >,
    patterns: RefCell<HashMap<dom::persistent_document::DOMOwnerHandle, Rc<PatternCache>>>,
}
impl Default for FormValidationState {
    fn default() -> Self {
        Self {
            // The embedding currently declares English resources. This is a
            // resource backend, never a fallback constraint algorithm.
            locale: RefCell::new(Rc::new(EnglishValidationLocale)),
            temporal_locale: RefCell::new(None),
            engine: RefCell::new(None),
            input_values: RefCell::new(None),
            client: RefCell::new(None),
            message_anchors: RefCell::new(HashSet::new()),
            temporal: RefCell::new(HashMap::new()),
            patterns: RefCell::new(HashMap::new()),
        }
    }
}
/// Locale's date/time field sequence, after DateTimeEditBuilder has selected
/// seconds/precision. Return None when the native format cannot be represented
/// by numeric fields; Chromium's subclass ISO fallback remains explicit.
pub trait TemporalInputLocale {
    fn FieldOrder(&self, kind: T, seconds: bool, milliseconds: bool) -> Option<Vec<DateTimeField>>;
    fn FieldLayout(
        &self,
        kind: T,
        seconds: bool,
        milliseconds: bool,
    ) -> Option<TemporalFieldLayout> {
        Some(TemporalFieldLayout {
            order: self.FieldOrder(kind, seconds, milliseconds)?,
            month_labels: Vec::new(),
            ampm_labels: Vec::new(),
            current_year: None,
        })
    }
    fn ConvertFromLocalizedDigit(&self, code: u32) -> Option<u32>;
}
impl DefaultActionState {
    pub fn SetValidationLocale(&self, locale: Rc<dyn ConstraintValidationLocale>) {
        *self.validation.locale.borrow_mut() = locale;
    }
    pub fn SetTemporalInputLocale(&self, locale: Rc<dyn TemporalInputLocale>) {
        *self.validation.temporal_locale.borrow_mut() = Some(locale);
        for owner in self.validation.temporal.borrow().values() {
            owner.borrow_mut().InvalidateLayout();
        }
    }
    pub fn SetValidationMessageClient(&self, client: Rc<dyn ValidationMessageClient>) {
        *self.validation.client.borrow_mut() = Some(client);
    }

    pub fn SetValidationPatternEngine(&self, engine: Rc<dyn ValidationPatternEngine>) {
        self.validation.patterns.borrow_mut().clear();
        *self.validation.engine.borrow_mut() = Some(engine);
    }
}
#[derive(Clone, Debug)]
pub struct ControlValidation {
    pub will_validate: bool,
    pub validity: ValidityState,
    pub message: Rc<[u16]>,
    pub sub_message: Rc<[u16]>,
}
/// Finish dirty ListedElement caches before lending the immutable DOM to
/// selector matching. This reuses IDL/submission's owning validity evaluator.
pub fn RefreshControlValidity(
    document: &InteractionDocument,
    state: &DefaultActionState,
) -> Result<(), ValidationRequired> {
    let pending = document
        .borrow()
        .GetDocument()
        .PendingControlValidityNodeIds();
    for id in pending {
        ControlValidity(document, state, id)?;
    }
    Ok(())
}
/// Element getters and submission share InputType state and canonical DOM
/// owners, including the compiled pattern and partial date/time field editor.
pub fn ControlValidity(
    document: &InteractionDocument,
    state: &DefaultActionState,
    id: u64,
) -> Result<ControlValidation, ValidationRequired> {
    let mut owner = document.borrow_mut();
    let d = owner.GetDocumentMut();
    let Some(i) = d.FindNodeById(id) else {
        return Ok(ControlValidation {
            will_validate: false,
            validity: ValidityState::default(),
            message: Rc::from([]),
            sub_message: Rc::from([]),
        });
    };
    d.UpdateControlWillValidate(i);
    let will_validate = d.CachedControlWillValidate(i).unwrap_or(false);
    let mut validity = ValidityState::default();
    let mut message: Rc<[u16]> = Rc::from([]);
    let mut sub_message: Rc<[u16]> = Rc::from([]);
    let locale = state.validation.locale.borrow().clone();
    match d.FormControlClass(i) {
        Some(C::Input) => {
            let kind = d.InputType(i).expect("input type owner");
            let node = d.Node(i);
            let attribute = |name: &str| {
                node.AttributeValueUTF16(name)
                    .unwrap_or_else(|| Rc::from([]))
            };
            let value = d.ControlValueUTF16(i);
            let inner = d.TextControlInnerEditorValueUTF16(i);
            let min = attribute("min");
            let max = attribute("max");
            let original = attribute("value");
            let step = node.FindAttribute("step").map_or("", |a| a.value.as_str());
            let group_missing = kind == T::kRadio
                && node
                    .FindAttribute("name")
                    .is_some_and(|a| !a.value.is_empty())
                && {
                    let (required, checked) = if let Some(scope) = d.RadioGroupScopeFor(i) {
                        let name = &node.FindAttribute("name").unwrap().value;
                        (
                            d.RadioIsInRequiredGroup(i),
                            d.RadioCheckedButtonForGroup(scope, name).is_some(),
                        )
                    } else {
                        let members = d.RadioGroupNodeIds(i);
                        (
                            members.iter().any(|&id| {
                                d.FindNodeById(id)
                                    .is_some_and(|n| d.Node(n).FindAttribute("required").is_some())
                            }),
                            members
                                .iter()
                                .any(|&id| d.FindNodeById(id).is_some_and(|n| d.ControlChecked(n))),
                        )
                    };
                    interaction_default::core::html::forms::radio_input_type::RadioInputType::ValueMissingWithGroup(required,checked)
                };
            let facts = InputValidationFacts {
                kind: super::page_input_type::SourceType(kind),
                value: &value,
                inner_editor_value: &inner,
                required: node.FindAttribute("required").is_some(),
                disabled_or_readonly: d.IsDisabledFormControl(i)
                    || (kind.SupportsReadOnly() && node.FindAttribute("readonly").is_some()),
                checked: d.ControlChecked(i),
                radio_group_value_missing: group_missing,
                files_empty: d.InputFiles(i).is_none_or(|files| files.length() == 0),
                multiple: node.FindAttribute("multiple").is_some(),
                dirty: d.HasDirtyControlValue(i),
                user_edit: d.TextControlLastChangeWasUserEdit(i),
                min_length: d.TextControlMinLength(i),
                max_length: d.TextControlMaxLength(i),
                pattern: node.AttributeValueUTF16("pattern"),
                minimum: &min,
                maximum: &max,
                step,
                value_attribute: &original,
            };
            let pattern = state
                .validation
                .patterns
                .borrow_mut()
                .entry(d.NodeHandle(i))
                .or_default()
                .clone();
            let engine = state.validation.engine.borrow().clone();
            validity = input_validation::Evaluate(
                &facts,
                &pattern,
                engine.as_deref(),
                &state.numbers,
                |value| {
                    String::from_utf16(value)
                        .ok()
                        .is_some_and(|value| url::Url::parse(&value).is_ok())
                },
            )?;
            if IsTemporal(kind) {
                let temporal = TemporalOwner(d, i, state);
                validity.bad_input = temporal.borrow().HasBadInput(&value);
            }
            message = input_validation::ValidationMessage(
                &facts,
                validity,
                &state.numbers,
                locale.as_ref(),
            )
            .into();
            if validity.pattern_mismatch
                && !validity.bad_input
                && !validity.value_missing
                && !validity.type_mismatch
            {
                sub_message = attribute("title");
            }
        }
        Some(C::TextArea) => {
            let value = d.ControlValueUTF16(i);
            validity.value_missing = interaction_default::core::html::forms::text_field_input_type::TextFieldInputType::ValueMissingWithState(&value,d.Node(i).FindAttribute("required").is_some(),d.IsDisabledFormControl(i)||d.Node(i).FindAttribute("readonly").is_some());
            if will_validate {
                validity.too_long = HTMLTextAreaElement::TooLongWithState(
                    &value,
                    d.TextControlMaxLength(i),
                    kCheckDirtyFlag,
                    d.TextControlLastChangeWasUserEdit(i),
                );
                validity.too_short = HTMLTextAreaElement::TooShortWithState(
                    &value,
                    d.TextControlMinLength(i),
                    kCheckDirtyFlag,
                    d.TextControlLastChangeWasUserEdit(i),
                );
            }
        }
        Some(C::Select) => {
            if d.Node(i).FindAttribute("required").is_some() {
                let collection = d.SelectCollection(i);
                let selected =
                    collection
                        .OptionNodeIds()
                        .iter()
                        .enumerate()
                        .find_map(|(offset, &id)| {
                            let option = d.FindNodeById(id)?;
                            d.OptionSelected(option).then_some((offset, option))
                        });
                let placeholder = !d.SelectIsMultiple(i)
                    && d.SelectSize(i) <= 1
                    && collection
                        .OptionNodeIds()
                        .first()
                        .and_then(|&id| d.FindNodeById(id))
                        .is_some_and(|option| {
                            d.OptionValueUTF16(option).is_empty()
                                && collection.ListItemNodeIds().first()
                                    == Some(&d.Node(option).Id())
                        });
                validity.value_missing = interaction_default::core::html::forms::html_select_element::HTMLSelectElement::ValueMissingWithState(
                    true,
                    selected.and_then(|(offset, _)| i32::try_from(offset).ok()),
                    placeholder,
                );
            }
        }
        _ => {}
    }
    if d.FormControlClass(i) == Some(C::TextArea) {
        let text = if validity.value_missing {
            Some(ConstraintValidationText::ValueMissing)
        } else if validity.too_long {
            Some(ConstraintValidationText::TooLong {
                length: d.ControlValueUTF16(i).len(),
                limit: d.TextControlMaxLength(i),
            })
        } else if validity.too_short {
            Some(ConstraintValidationText::TooShort {
                length: d.ControlValueUTF16(i).len(),
                limit: d.TextControlMinLength(i),
            })
        } else {
            None
        };
        if let Some(text) = text {
            message = locale.QueryValidationText(text, &[], &[]).into();
        }
    } else if d.FormControlClass(i) == Some(C::Select) && validity.value_missing {
        message = locale
            .QueryValidationText(ConstraintValidationText::ValueMissingSelect, &[], &[])
            .into();
    }
    let custom = d.ControlCustomValidationMessageUTF16(i);
    if d.ElementInternalsTargetIsFormAssociated(i) {
        let flags = d.ElementInternalsValidityFlags(i);
        validity = ValidityState {
            value_missing: flags & 1 != 0,
            type_mismatch: flags & 2 != 0,
            pattern_mismatch: flags & 4 != 0,
            too_long: flags & 8 != 0,
            too_short: flags & 16 != 0,
            range_underflow: flags & 32 != 0,
            range_overflow: flags & 64 != 0,
            step_mismatch: flags & 128 != 0,
            bad_input: flags & 256 != 0,
            custom_error: flags & 512 != 0,
        };
        if !validity.valid() {
            message = custom.clone();
        }
        if validity.pattern_mismatch {
            sub_message = d
                .Node(i)
                .AttributeValueUTF16("title")
                .unwrap_or_else(|| Rc::from([]));
        }
    } else {
        validity.custom_error = !custom.is_empty();
    }
    if !will_validate && !d.ElementInternalsTargetIsFormAssociated(i) {
        message = Rc::from([]);
        sub_message = Rc::from([]);
    } else if !custom.is_empty() && !d.ElementInternalsTargetIsFormAssociated(i) {
        message = custom;
        sub_message = Rc::from([]);
    }
    d.CommitControlValidity(i, validity.valid());
    Ok(ControlValidation {
        will_validate,
        validity,
        message,
        sub_message,
    })
}
/// English resources from blink_strings.grd's IDS_FORM_VALIDATION_* entries.
/// Platform embeddings can replace this backend without changing decisions.
struct EnglishValidationLocale;
impl ConstraintValidationLocale for EnglishValidationLocale {
    fn QueryValidationText(
        &self,
        kind: ConstraintValidationText,
        first: &[u16],
        second: &[u16],
    ) -> Vec<u16> {
        use ConstraintValidationText::*;
        let first = String::from_utf16_lossy(first);
        let second = String::from_utf16_lossy(second);
        let text = match kind {
            BadInputNumber => "Please enter a number.".into(),
            BadInputDateTime => "Please enter a valid value. The field is incomplete or has an invalid date.".into(),
            ValueMissing => "Please fill out this field.".into(),
            ValueMissingCheckbox => "Please check this box if you want to proceed.".into(),
            ValueMissingRadio => "Please select one of these options.".into(),
            ValueMissingFile => "Please select a file.".into(),
            ValueMissingMultipleFile => "Please select one or more files.".into(),
            ValueMissingSelect => "Please select an item in the list.".into(),
            TypeMismatch => "Invalid value.".into(),
            TypeMismatchURL => "Please enter a URL.".into(),
            TypeMismatchEmail => "Please enter an email address.".into(),
            PatternMismatch => "Please match the requested format.".into(),
            TooLong { length, limit } => format!("Please shorten this text to {limit} characters or less (you are currently using {length} characters)."),
            TooShort { length: 1, limit } => format!("Please lengthen this text to {limit} characters or more (you are currently using 1 character)."),
            TooShort { length, limit } => format!("Please lengthen this text to {limit} characters or more (you are currently using {length} characters)."),
            StepMismatchCloseToLimit => format!("Please enter a valid value. The nearest valid value is {first}."),
            StepMismatch => format!("Please enter a valid value. The two nearest valid values are {first} and {second}."),
            RangeInvalid { temporal: false } => format!("Minimum value ({first}) must be less than the maximum value ({second})."),
            RangeInvalid { temporal: true } => format!("Minimum date ({first}) must come before Maximum date ({second})."),
            ReversedRange => format!("Value must be between {first} and {second}."),
            ValueNotEqual => format!("Value must be {first}."),
            RangeUnderflow { temporal: false } => format!("Value must be greater than or equal to {first}."),
            RangeUnderflow { temporal: true } => format!("Value must be {first} or later."),
            RangeOverflow { temporal: false } => format!("Value must be less than or equal to {first}."),
            RangeOverflow { temporal: true } => format!("Value must be {first} or earlier."),
        };
        text.encode_utf16().collect()
    }
}
struct Control<'a, 'b> {
    c: &'a InputTypeContext<'b>,
    id: u64,
    value: ControlValidation,
}
impl ListedElementValidationOperations for Control<'_, '_> {
    fn WillValidate(&self) -> bool {
        self.value.will_validate
    }
    fn Valid(&self) -> bool {
        self.value.validity.valid()
    }
    fn DispatchInvalid(&self) -> bool {
        let mut event = MakeSyntheticEvent(EventType::kCustom, self.id);
        event.custom_type = "invalid".into();
        event.bubbles = false;
        event.cancelable = true;
        (self.c.dispatch_synthetic)(&mut event, self.id);
        event.default_prevented
    }
    fn IsConnected(&self) -> bool {
        ReadNode(&self.c.document, self.id, |d, _, i| {
            d.IsConnectedToDocument(i)
        })
        .unwrap_or(false)
    }
    fn DocumentIdentity(&self) -> [u64; 2] {
        self.c.document.borrow().GetDocument().Token()
    }
}
pub fn CheckControlValidity(
    c: &InputTypeContext<'_>,
    id: u64,
    report: bool,
) -> Result<bool, ValidationRequired> {
    let value = ControlValidity(&c.document, &c.default_action_state, id)?;
    let control = Control { c, id, value };
    let result = listed_element::CheckValidity(&control, report);
    if report && result.unhandled {
        ShowValidationMessage(c, id)?;
    }
    Ok(result.valid)
}
pub fn CheckControlValidityForForm(
    c: &InputTypeContext<'_>,
    id: u64,
    collect: bool,
) -> Result<FormControlValidity, ValidationRequired> {
    let value = ControlValidity(&c.document, &c.default_action_state, id)?;
    Ok(listed_element::CheckValidity(
        &Control { c, id, value },
        collect,
    ))
}
struct Form<'a, 'b> {
    c: &'a InputTypeContext<'b>,
    id: u64,
    error: Cell<Option<ValidationRequired>>,
}
impl FormValidationOperations<u64> for Form<'_, '_> {
    fn ListedElements(&self) -> Box<dyn Iterator<Item = u64> + '_> {
        let ids = ReadNode(&self.c.document, self.id, |d, _, i| {
            d.ListedFormControlNodeIds(i)
        })
        .unwrap_or_default();
        Box::new((0..ids.len()).map(move |i| ids[i]))
    }
    fn IsOwnedByForm(&self, id: &u64) -> bool {
        ReadNode(&self.c.document, *id, |d, _, i| {
            d.FormOwner(i).is_some_and(|f| d.Node(f).Id() == self.id)
        })
        .unwrap_or(false)
    }
    fn ShouldCheckValidity(&self, id: &u64) -> bool {
        ReadNode(&self.c.document, *id, |d, _, i| {
            d.IsFormAssociatedCustomElement(i)
                || matches!(
                    d.FormControlClass(i),
                    Some(C::Input | C::TextArea | C::Select | C::Button | C::Object)
                )
        })
        .unwrap_or(false)
    }
    fn CheckValidity(&self, id: &u64, collect: bool) -> FormControlValidity {
        match CheckControlValidityForForm(self.c, *id, collect) {
            Ok(value) => value,
            Err(error) => {
                self.error.set(Some(error));
                FormControlValidity {
                    valid: false,
                    unhandled: false,
                }
            }
        }
    }
    fn HideVisibleValidationMessage(&self, id: &u64) {
        if let Err(error) = HideValidationMessage(self.c, *id) {
            self.error.set(Some(error));
        }
    }
    fn PrepareValidation(&self) {}
    fn ValidationAnchorOrHostIsFocusable(&self, id: &u64) -> bool {
        self.c.focus_controller.ValidationAnchorIsFocusable(*id)
    }
    fn ShowValidationMessage(&self, id: &u64) {
        if let Err(error) = ShowValidationMessage(self.c, *id) {
            self.error.set(Some(error));
        }
    }
}
pub fn CheckFormValidity(
    c: &InputTypeContext<'_>,
    id: u64,
    report: bool,
) -> Result<bool, ValidationRequired> {
    let form = Form {
        c,
        id,
        error: Cell::new(None),
    };
    let result = if report {
        HTMLFormElement::ValidateInteractivelyWithOperations(&form)
    } else {
        !HTMLFormElement::CheckInvalidControlsWithOperations(&form, None)
    };
    form.error.get().map_or(Ok(result), Err)
}
pub fn ValidateFormInteractively(
    c: &InputTypeContext<'_>,
    id: u64,
) -> Result<bool, ValidationRequired> {
    CheckFormValidity(c, id, true)
}

fn ShowValidationMessage(c: &InputTypeContext<'_>, id: u64) -> Result<(), ValidationRequired> {
    let client = c
        .default_action_state
        .validation
        .client
        .borrow()
        .clone()
        .ok_or(ValidationRequired::ValidationMessageClient)?;
    let anchor = ReadNode(&c.document, id, |d, _, i| {
        d.Node(d.ElementInternalsValidationAnchor(i)).Id()
    })
    .unwrap_or(id);
    c.focus_controller.FocusWithScriptTrigger(anchor);
    // Focus listeners may change constraints or detach the control. Re-read
    // the same ListedElement before showing its current message.
    let value = ControlValidity(&c.document, &c.default_action_state, id)?;
    let Some(identity) = ReadNode(&c.document, id, |d, _, i| {
        d.IsConnectedToDocument(i).then(|| d.NodeHandle(i))
    })
    .flatten() else {
        return Ok(());
    };
    c.default_action_state
        .validation
        .message_anchors
        .borrow_mut()
        .insert(identity);
    client.Apply(if value.message.is_empty() {
        ValidationMessageEffect::Hide { anchor: id }
    } else {
        ValidationMessageEffect::Show {
            anchor: id,
            message: value.message,
            sub_message: value.sub_message,
        }
    })
}
fn HideValidationMessage(c: &InputTypeContext<'_>, id: u64) -> Result<(), ValidationRequired> {
    let known = ReadNode(&c.document, id, |d, _, i| {
        c.default_action_state
            .validation
            .message_anchors
            .borrow()
            .contains(&d.NodeHandle(i))
    })
    .unwrap_or(false);
    if !known {
        return Ok(());
    }
    c.default_action_state
        .validation
        .client
        .borrow()
        .clone()
        .ok_or(ValidationRequired::ValidationMessageClient)?
        .Apply(ValidationMessageEffect::Hide { anchor: id })
}

fn IsTemporal(kind: T) -> bool {
    matches!(
        kind,
        T::kDate | T::kDateTimeLocal | T::kMonth | T::kTime | T::kWeek
    )
}
fn TemporalOwner(
    d: &dom::Document,
    i: usize,
    state: &DefaultActionState,
) -> Rc<RefCell<interaction_default::core::html::forms::temporal_input_type::TemporalInputType>> {
    let kind = super::page_input_type::SourceType(d.InputType(i).expect("temporal input"));
    let revision = d.ControlValueRevision(i);
    let value = d.ControlValueUTF16(i);
    let owner = state
        .validation
        .temporal
        .borrow_mut()
        .entry(d.NodeHandle(i))
        .or_insert_with(|| {
            Rc::new(RefCell::new(
                interaction_default::core::html::forms::temporal_input_type::TemporalInputType::new(
                    kind, &value, revision,
                ),
            ))
        })
        .clone();
    owner.borrow_mut().SynchronizeValue(kind, &value, revision);
    let minimum = d
        .Node(i)
        .AttributeValueUTF16("min")
        .unwrap_or_else(|| Rc::from([]));
    let maximum = d
        .Node(i)
        .AttributeValueUTF16("max")
        .unwrap_or_else(|| Rc::from([]));
    let value_attribute = d
        .Node(i)
        .AttributeValueUTF16("value")
        .unwrap_or_else(|| Rc::from([]));
    let step = d
        .Node(i)
        .FindAttribute("step")
        .map_or("", |a| a.value.as_str());
    if !owner
        .borrow()
        .NeedsLayout(&minimum, &maximum, &value_attribute, step)
    {
        return owner;
    }
    let (seconds, milliseconds) =
        input_validation::TemporalPrecision(kind, &value, &minimum, step, &state.numbers);
    let layout = state
        .validation
        .temporal_locale
        .borrow()
        .as_ref()
        .and_then(|locale| locale.FieldLayout(d.InputType(i).unwrap(), seconds, milliseconds));
    let mut layout = layout
        .filter(|layout| !layout.order.is_empty())
        .unwrap_or_else(|| TemporalFieldLayout {
            order: interaction_default::core::html::forms::temporal_input_type::FallbackFieldOrder(
                kind,
                seconds,
                milliseconds,
            ),
            month_labels: Vec::new(),
            ampm_labels: Vec::new(),
            current_year: None,
        });
    owner
        .borrow_mut()
        .SetFieldOrder(std::mem::take(&mut layout.order), &minimum, step);
    let (editor_step, base) = input_validation::TemporalEditorStep(
        kind,
        &minimum,
        &value_attribute,
        step,
        &state.numbers,
    );
    owner.borrow_mut().ConfigureFields(
        &maximum,
        &value_attribute,
        editor_step,
        base,
        layout.month_labels,
        layout.ampm_labels,
        layout.current_year,
    );
    owner
}
pub(super) fn TemporalInput<'a>(
    id: u64,
    c: InputTypeContext<'a>,
) -> Option<Box<dyn crate::input_type::InputType + 'a>> {
    ReadNode(&c.document, id, |d, _, i| {
        d.InputType(i).is_some_and(IsTemporal)
    })
    .unwrap_or(false)
    .then(|| Box::new(PageTemporalInput { id, c }) as Box<dyn crate::input_type::InputType + 'a>)
}
struct PageTemporalInput<'a> {
    id: u64,
    c: InputTypeContext<'a>,
}
impl crate::input_type::InputType for PageTemporalInput<'_> {
    fn HandleDefaultEvent(&self, event: &mut crate::event::Event) {
        if !matches!(event.r#type, EventType::kKeyDown | EventType::kKeyPress) {
            return;
        }
        let Some((owner, readonly, old_value)) = ReadNode(&self.c.document, self.id, |d, n, i| {
            if d.IsDisabledFormControl(i) {
                return None;
            }
            Some((
                TemporalOwner(d, i, &self.c.default_action_state),
                n.FindAttribute("readonly").is_some(),
                d.ControlValueUTF16(i),
            ))
        })
        .flatten() else {
            return;
        };
        let handled = if event.r#type == EventType::kKeyDown {
            owner
                .borrow_mut()
                .HandleKeydown(&event.key, event.modifiers.alt, readonly)
        } else if !readonly
            && !event.modifiers.control
            && !event.modifiers.meta
            && !event.modifiers.alt
        {
            let locale = self
                .c
                .default_action_state
                .validation
                .temporal_locale
                .borrow()
                .clone();
            let digit = if let Some(locale) = locale {
                locale.ConvertFromLocalizedDigit(event.char_code)
            } else {
                event
                    .char_code
                    .checked_sub('0' as u32)
                    .filter(|digit| *digit <= 9)
            };
            let timestamp = event.canonical.as_ref().map_or(0, |native| {
                native
                    .borrow()
                    .BaseEvent()
                    .PlatformTimeStamp()
                    .ToInternalValue()
            });
            let mut editor = owner.borrow_mut();
            editor.HandleSymbol(event.char_code, timestamp)
                || digit.is_some_and(|digit| editor.HandleDigit(digit))
        } else {
            false
        };
        if !handled {
            if event.r#type == EventType::kKeyDown {
                super::page_input_type::DispatchKeyPress(
                    event,
                    self.id,
                    &self.c.dispatch_synthetic,
                );
            }
            return;
        }
        event.default_handled = true;
        let new_value = owner.borrow().Value();
        let changed = old_value.as_ref() != new_value.as_slice();
        {
            let mut document = self.c.document.borrow_mut();
            let d = document.GetDocumentMut();
            let Some(i) = d.FindNodeById(self.id) else {
                return;
            };
            if changed {
                d.SetStoredControlValueUTF16(i, &new_value, true);
            }
            d.SetControlNeedsValidityCheck(i);
            d.TextControlSetUserEdited(i, true);
            owner.borrow_mut().DidCommitValue(d.ControlValueRevision(i));
        }
        (self.c.emit_mutation)(&dom::dom_mutation::DOMMutation {
            mutation_type: dom::dom_mutation::DOMMutationType::kSetControlNeedsValidityCheck,
            target_node_id: self.id,
            ..Default::default()
        });
        if changed {
            let mut input = MakeSyntheticEvent(EventType::kInput, self.id);
            (self.c.dispatch_synthetic)(&mut input, self.id);
        }
        let mut change = MakeSyntheticEvent(EventType::kChange, self.id);
        (self.c.dispatch_synthetic)(&mut change, self.id);
    }
}

struct PageInputValueSanitizer {
    state: Rc<DefaultActionState>,
}
impl dom::input_value::DOMInputValueSanitizer for PageInputValueSanitizer {
    fn ValueMode(&self, kind: T) -> dom::input_value::DOMInputValueMode {
        use dom::input_value::DOMInputValueMode as M;
        use interaction_default::core::html::forms::input_type::{ValueMode, ValueModeForType};
        match ValueModeForType(super::page_input_type::SourceType(kind)) {
            ValueMode::kValue => M::Value,
            ValueMode::kDefault => M::Default,
            ValueMode::kDefaultOn => M::DefaultOn,
            ValueMode::kFilename => M::Filename,
        }
    }
    fn SanitizeValue(&self, request: dom::input_value::DOMInputValueRequest<'_>) -> Rc<[u16]> {
        let facts = InputValidationFacts {
            kind: super::page_input_type::SourceType(request.kind),
            value: &request.proposed,
            inner_editor_value: &[],
            required: false,
            disabled_or_readonly: false,
            checked: false,
            radio_group_value_missing: false,
            files_empty: true,
            multiple: request.multiple,
            dirty: false,
            user_edit: false,
            min_length: -1,
            max_length: -1,
            pattern: None,
            minimum: request.minimum,
            maximum: request.maximum,
            step: request.step,
            value_attribute: request.value_attribute,
        };
        let value = interaction_default::core::html::forms::input_value::SanitizeValue(
            &facts,
            &self.state.numbers,
        );
        if value.as_slice() == request.proposed.as_ref() {
            request.proposed
        } else {
            value.into()
        }
    }
    fn DidCommitValue(
        &self,
        identity: dom::persistent_document::DOMOwnerHandle,
        kind: T,
        value: &[u16],
        revision: u64,
    ) {
        if IsTemporal(kind) {
            let source = super::page_input_type::SourceType(kind);
            let temporal=self.state.validation.temporal.borrow_mut().entry(identity).or_insert_with(||Rc::new(RefCell::new(interaction_default::core::html::forms::temporal_input_type::TemporalInputType::new(source,value,revision)))).clone();
            temporal
                .borrow_mut()
                .SynchronizeValue(source, value, revision);
        } else {
            self.state
                .validation
                .temporal
                .borrow_mut()
                .remove(&identity);
        }
    }
}
pub(super) fn InstallInputValueSanitizer(
    document: &InteractionDocument,
    state: Rc<DefaultActionState>,
) {
    let cached = state
        .validation
        .input_values
        .borrow()
        .as_ref()
        .and_then(|weak| weak.upgrade());
    let sanitizer = cached.unwrap_or_else(|| {
        let sanitizer: Rc<dyn dom::input_value::DOMInputValueSanitizer> =
            Rc::new(PageInputValueSanitizer {
                state: state.clone(),
            });
        *state.validation.input_values.borrow_mut() = Some(Rc::downgrade(&sanitizer));
        sanitizer
    });
    document
        .borrow_mut()
        .GetDocumentMut()
        .InstallInputValueSanitizer(sanitizer);
}

/// Standalone DOM bindings use the identical source value lifecycle.
pub fn InstallStandaloneInputValueSanitizer(
    document: &InteractionDocument,
    state: Rc<DefaultActionState>,
) {
    InstallInputValueSanitizer(document, state);
}

pub(crate) fn SaveTemporalFields(c: &InputTypeContext<'_>, node: u64) -> Option<Vec<Rc<[u16]>>> {
    ReadNode(&c.document, node, |d, _, i| {
        if !d.InputType(i).is_some_and(IsTemporal) {
            return None;
        }
        Some(
            TemporalOwner(d, i, &c.default_action_state)
                .borrow()
                .Fields()
                .SaveFormControlState(),
        )
    })
    .flatten()
}
pub(crate) fn RestoreTemporalFields(
    c: &InputTypeContext<'_>,
    node: u64,
    state: &[Rc<[u16]>],
) -> bool {
    let Some(owner) = ReadNode(&c.document, node, |d, _, i| {
        d.InputType(i)
            .is_some_and(IsTemporal)
            .then(|| TemporalOwner(d, i, &c.default_action_state))
    })
    .flatten() else {
        return false;
    };
    owner.borrow_mut().RestoreFields(interaction_default::core::html::forms::temporal_input_type::DateTimeFieldsState::RestoreFormControlState(state));
    let value = owner.borrow().Value();
    {
        let mut document = c.document.borrow_mut();
        let d = document.GetDocumentMut();
        let Some(i) = d.FindNodeById(node) else {
            return true;
        };
        d.SetStoredControlValueUTF16(i, &value, false);
        d.SetControlValueDirty(i, true);
        owner.borrow_mut().DidCommitValue(d.ControlValueRevision(i));
    }
    (c.emit_mutation)(&dom::dom_mutation::DOMMutation {
        mutation_type: dom::dom_mutation::DOMMutationType::kSetControlNeedsValidityCheck,
        target_node_id: node,
        ..Default::default()
    });
    true
}
