/*
 * Copyright (C) 2010 Google Inc. All rights reserved.
 * Copyright (C) 2011 Apple Inc. All rights reserved.
 * Copyright (C) 2012 Samsung Electronics. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *     * Neither the name of Google Inc. nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/input_type.h:56-98,104-117,139-143,188-190,203-205,212,217-218,248,259-261,287,295,311,342-344,383-384
// Source: third_party/blink/renderer/core/html/forms/input_type.cc:321-327,364-371,750-776,781-787,841-843,932-934,1349-1351
// The translated base contract includes the source's non-virtual
// CanSetStringValue dispatch; concrete families retain their other branches in
// their own modules.

pub use super::text_control_element::{TextControlSetValueSelection, TextFieldEventBehavior};
pub use crate::core::dom::element::{BaseAppearanceValue, UpdateBehavior};
use crate::public::mojom::forms::form_control_type::FormControlType;
use interaction_dispatch::core::dom::events::event::EventRef;
use interaction_dispatch::core::dom::events::event_dispatcher::EventDispatchHandlingState;
use std::rc::Rc;

// Source: input_type.h:61-98. Numeric identity follows
// public/mojom/forms/form_control_type.mojom:18-39.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Type {
    kButton = FormControlType::kInputButton as i32,
    kColor = FormControlType::kInputColor as i32,
    kFile = FormControlType::kInputFile as i32,
    kHidden = FormControlType::kInputHidden as i32,
    kImage = FormControlType::kInputImage as i32,
    kNumber = FormControlType::kInputNumber as i32,
    kRange = FormControlType::kInputRange as i32,
    kReset = FormControlType::kInputReset as i32,
    kSubmit = FormControlType::kInputSubmit as i32,
    kRadio = FormControlType::kInputRadio as i32,
    kCheckbox = FormControlType::kInputCheckbox as i32,
    kDate = FormControlType::kInputDate as i32,
    kDateTimeLocal = FormControlType::kInputDatetimeLocal as i32,
    kMonth = FormControlType::kInputMonth as i32,
    kTime = FormControlType::kInputTime as i32,
    kWeek = FormControlType::kInputWeek as i32,
    kEmail = FormControlType::kInputEmail as i32,
    kPassword = FormControlType::kInputPassword as i32,
    kSearch = FormControlType::kInputSearch as i32,
    kTelephone = FormControlType::kInputTelephone as i32,
    kURL = FormControlType::kInputUrl as i32,
    kText = FormControlType::kInputText as i32,
}

// Source: input_type.h:188-190.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueMode {
    kValue,
    kDefault,
    kDefaultOn,
    kFilename,
}

/// A retained identity, comparable without reading a DOM object or tree.
pub type OwnerIdentity = Rc<()>;
pub type InputElementRef = Rc<dyn InputElement>;

/// The element's existing control state and synchronous effects. The owner
/// implements SetChecked with its existing radio-group owner and change gate;
/// this projection stores no second element or group model.
pub trait InputElement {
    fn FormForSubmission(&self) -> Option<super::html_input_element::HTMLInputFormRef>;
    fn Checked(&self) -> bool;
    fn SetChecked(&self, checked: bool, behavior: TextFieldEventBehavior);
    fn indeterminate(&self) -> bool;
    fn setIndeterminate(&self, value: bool);
    fn GetName(&self) -> String;
    fn Value(&self) -> String;
    fn NameForFormData(&self) -> Rc<[u16]>;
    fn ValueForFormData(&self) -> Rc<[u16]>;
    fn ValueForFormState(&self) -> foundation::NullableUTF16String;
    fn DefaultValueForFormState(&self) -> foundation::NullableUTF16String;
    fn RestoreFormStateValue(&self, value: &[u16]);
    fn DirnameAndDirectionForFormData(&self) -> Option<(Rc<[u16]>, Rc<[u16]>)>;
    fn SetValueAttribute(&self, value: &str);
    fn IsRequired(&self) -> bool;
    fn HasCheckedAttribute(&self) -> bool;
    fn HasRequiredAttribute(&self) -> bool;
    fn IsActive(&self) -> bool;
    fn SetActive(&self, active: bool);
    fn HasActiveLabel(&self) -> bool;
    fn IsFocusable(&self, update: UpdateBehavior) -> bool;
    fn tabIndex(&self) -> i32;
    fn HasLayoutObject(&self) -> bool;
    fn IsRightToLeft(&self) -> bool;
    fn SetUserHasEditedTheField(&self);
    fn DispatchInputAndChangeEventIfNeeded(&self);
    fn DispatchSimulatedClick(&self, event: &EventRef);
    fn FocusForAccessKey(&self);
    fn DispatchAccessKeySimulatedClick(
        &self,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    );
}

// Source: input_type.h:56-59,341-344,383-384. The same element owner is also
// held by InputTypeView, matching the source's two base subobjects.
pub struct InputType {
    type_: Type,
    element_: InputElementRef,
}
impl InputType {
    pub fn new(input_type: Type, element: InputElementRef) -> Self {
        Self {
            type_: input_type,
            element_: element,
        }
    }
    // Source: input_type.h:111,139-143.
    pub fn r#type(&self) -> Type {
        self.type_
    }
    // Source: input_type.h:114-117. Match expresses the source numeric cast
    // without an unsafe enum conversion.
    pub fn FormControlType(&self) -> FormControlType {
        match self.type_ {
            Type::kButton => FormControlType::kInputButton,
            Type::kColor => FormControlType::kInputColor,
            Type::kFile => FormControlType::kInputFile,
            Type::kHidden => FormControlType::kInputHidden,
            Type::kImage => FormControlType::kInputImage,
            Type::kNumber => FormControlType::kInputNumber,
            Type::kRange => FormControlType::kInputRange,
            Type::kReset => FormControlType::kInputReset,
            Type::kSubmit => FormControlType::kInputSubmit,
            Type::kRadio => FormControlType::kInputRadio,
            Type::kCheckbox => FormControlType::kInputCheckbox,
            Type::kDate => FormControlType::kInputDate,
            Type::kDateTimeLocal => FormControlType::kInputDatetimeLocal,
            Type::kMonth => FormControlType::kInputMonth,
            Type::kTime => FormControlType::kInputTime,
            Type::kWeek => FormControlType::kInputWeek,
            Type::kEmail => FormControlType::kInputEmail,
            Type::kPassword => FormControlType::kInputPassword,
            Type::kSearch => FormControlType::kInputSearch,
            Type::kTelephone => FormControlType::kInputTelephone,
            Type::kURL => FormControlType::kInputUrl,
            Type::kText => FormControlType::kInputText,
        }
    }
    pub fn IsRadioInputType(&self) -> bool {
        self.type_ == Type::kRadio
    }
    pub fn IsCheckboxInputType(&self) -> bool {
        self.type_ == Type::kCheckbox
    }
    pub fn IsBaseCheckableInputType(&self) -> bool {
        self.IsRadioInputType() || self.IsCheckboxInputType()
    }
    pub fn GetElement(&self) -> &InputElementRef {
        &self.element_
    }
    // Source: input_type.cc:321-327.
    pub fn ReadingChecked(&self) {}
    pub fn WillUpdateCheckedness(&self, _checked: bool) {}
    // Source: input_type.cc:937-969. Source dispatch is a typed discriminant,
    // including date/time types that may be supplied by additional subclasses.
    pub fn IsSteppable(&self) -> bool {
        self.type_.IsSteppable()
    }
    pub fn SupportsValidation(&self) -> bool {
        true
    }
    // Source: input_type.cc:364-371.
    pub fn TypeMismatch(&self) -> bool {
        false
    }
    pub fn SupportsRequired(&self) -> bool {
        self.SupportsValidation()
    }
    // Source: input_type.cc:750-776. The source avoids a virtual call here.
    // FileInputType's selected implementation is file_input_type.cc:256-258.
    pub fn CanSetStringValue(&self) -> bool {
        match self.type_ {
            Type::kRadio | Type::kCheckbox | Type::kFile => false,
            Type::kEmail
            | Type::kPassword
            | Type::kSearch
            | Type::kTelephone
            | Type::kURL
            | Type::kText
            | Type::kDate
            | Type::kDateTimeLocal
            | Type::kMonth
            | Type::kTime
            | Type::kWeek
            | Type::kNumber
            | Type::kColor
            | Type::kRange
            | Type::kButton
            | Type::kHidden
            | Type::kImage
            | Type::kReset
            | Type::kSubmit => true,
        }
    }
    // Source: input_type.cc:781-787.
    pub fn IsKeyboardFocusableSlow(&self, update: UpdateBehavior) -> bool {
        Self::IsKeyboardFocusableForElement(self.element_.as_ref(), update)
    }
    pub fn IsKeyboardFocusableForElement(
        element: &dyn InputElement,
        update: UpdateBehavior,
    ) -> bool {
        element.IsFocusable(update) && element.tabIndex() >= 0
    }
    // Source: input_type.cc:841-843.
    pub fn ShouldSendChangeEventAfterCheckedChanged(&self) -> bool {
        true
    }
    // Source: input_type.cc:932-934.
    pub fn IsCheckable(&self) -> bool {
        false
    }
    // Source: input_type.cc:1349-1351.
    pub fn SupportsBaseAppearance(&self, _value: BaseAppearanceValue) -> bool {
        false
    }
}

// Source: input_type_view.h:65-72. Defined in that source's Rust module.
pub trait CheckableActivationState: EventDispatchHandlingState {}

/// Source locale query, kept with the existing element locale owner.
pub trait CheckableLocale {
    fn ValueMissingCheckbox(&self) -> String;
    fn ValueMissingRadio(&self) -> String;
}

// Source: input_type.cc:275-287. The small optional direction value is read
// from the same element owner only for auto-directionality-associated types.
pub trait InputTypeFormData {
    fn AppendFromElement(&mut self, name: Rc<[u16]>, value: Rc<[u16]>);
}
/// Borrowed native control name/value facts for the source append operation.
pub trait InputFormDataOperations {
    fn NameForFormData(&self) -> Rc<[u16]>;
    fn ValueForFormData(&self) -> Rc<[u16]>;
    fn ValueOrDefaultLabelForFormData(&self) -> Rc<[u16]> {
        self.ValueForFormData()
    }
    fn FilesForFormData(
        &self,
    ) -> Option<Rc<interaction_dispatch::core::fileapi::file_list::FileList>> {
        None
    }
    fn ImageCoordinatesForFormData(&self) -> Option<(i32, i32)> {
        None
    }
}
struct SourceInputFormData<'a>(&'a InputElementRef);
impl InputFormDataOperations for SourceInputFormData<'_> {
    fn NameForFormData(&self) -> Rc<[u16]> {
        self.0.NameForFormData()
    }
    fn ValueForFormData(&self) -> Rc<[u16]> {
        self.0.ValueForFormData()
    }
}
impl InputType {
    pub fn AppendToFormData(
        &self,
        data: &mut dyn InputTypeFormData,
        direction: Option<(Rc<[u16]>, Rc<[u16]>)>,
    ) {
        Self::AppendElementToFormData(self.type_, &self.element_, data, direction);
    }
    pub fn AppendElementToFormData(
        input_type: Type,
        element: &InputElementRef,
        data: &mut dyn InputTypeFormData,
        direction: Option<(Rc<[u16]>, Rc<[u16]>)>,
    ) {
        Self::AppendWithOperations(input_type, &SourceInputFormData(element), data, direction);
    }
    pub fn AppendWithOperations(
        input_type: Type,
        element: &dyn InputFormDataOperations,
        data: &mut dyn InputTypeFormData,
        direction: Option<(Rc<[u16]>, Rc<[u16]>)>,
    ) {
        if input_type != Type::kSubmit {
            data.AppendFromElement(element.NameForFormData(), element.ValueForFormData());
        }
        if let Some((dirname, direction)) = direction {
            data.AppendFromElement(dirname, direction);
        }
    }
}

/// Admission gaps are source families whose canonical payload is not yet
/// supplied by the DOM boundary. They must never be replaced by string values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormDataAppendAdmission {
    FileList,
    ImageCoordinates,
}
impl InputType {
    // HTMLInputElement cc:1128-1131 and InputType cc:270-287. This is the
    // virtual-family dispatch for the borrowed canonical DOM control.
    pub fn AppendSupportedWithOperations(
        input_type: Type,
        element: &dyn InputFormDataOperations,
        checked: bool,
        activated: bool,
        direction: Option<(Rc<[u16]>, Rc<[u16]>)>,
        data: &mut interaction_dispatch::core::fileapi::form_data::FormData,
    ) -> Result<(), FormDataAppendAdmission> {
        if input_type == Type::kImage {
            return super::image_input_type::ImageInputType::AppendWithOperations(
                activated, element, data,
            );
        }
        if element.NameForFormData().is_empty() {
            return Ok(());
        }
        match input_type {
            Type::kFile => {
                super::file_input_type::FileInputType::AppendWithOperations(element, data)?
            }
            Type::kButton | Type::kReset => {} // BaseButtonInputType cc:80.
            Type::kCheckbox | Type::kRadio => {
                super::base_checkable_input_type::BaseCheckableInputType::AppendWithOperations(
                    checked,
                    element.NameForFormData(),
                    element.ValueForFormData(),
                    data,
                );
            }
            Type::kSubmit => {
                Self::AppendWithOperations(input_type, element, data, direction);
                super::submit_input_type::SubmitInputType::AppendActivatedValue(
                    activated,
                    element.NameForFormData(),
                    element.ValueOrDefaultLabelForFormData(),
                    data,
                );
            }
            // HiddenInputType cc:90-97: encoding metadata comes from this
            // exact FormData owner, before normal base append/direction.
            Type::kHidden
                if {
                    let name = element.NameForFormData();
                    name.len() == 9
                        && name
                            .iter()
                            .zip(b"_charset_")
                            .all(|(&u, &b)| u <= 127 && (u as u8).eq_ignore_ascii_case(&b))
                } =>
            {
                let value: Rc<[u16]> = data
                    .EncodingName()
                    .encode_utf16()
                    .collect::<Vec<_>>()
                    .into();
                data.AppendFromElement(&element.NameForFormData(), &value);
            }
            _ => Self::AppendWithOperations(input_type, element, data, direction),
        }
        Ok(())
    }
    // Source BaseTextInputType cc:150-152, BaseButtonInputType cc:76-78,
    // HiddenInputType cc:103-105; inherited InputType cc:200-202 is false.
    pub fn IsAutoDirectionalityFormAssociatedForType(input_type: Type) -> bool {
        matches!(
            input_type,
            Type::kText
                | Type::kSearch
                | Type::kTelephone
                | Type::kURL
                | Type::kEmail
                | Type::kPassword
                | Type::kHidden
                | Type::kButton
                | Type::kReset
                | Type::kSubmit
                | Type::kImage
        )
    }
}

/// Source: input_type.cc:849-879. The inherited value-mode implementation
/// used by range/color/temporal types, with the same synchronous change gates.
pub trait InputValueOperations {
    fn SetNonAttributeValue(&self, value: &[u16]) {
        self.SetNonAttributeValueOwner(&foundation::NullableUTF16String::from(value));
    }
    fn SetNonAttributeValueByUserEdit(&self, value: &[u16]) {
        self.SetNonAttributeValueOwnerByUserEdit(&foundation::NullableUTF16String::from(value));
    }
    fn SetNonAttributeValueOwner(&self, value: &foundation::NullableUTF16String);
    fn SetNonAttributeValueOwnerByUserEdit(&self, value: &foundation::NullableUTF16String);
    fn DispatchInputEvent(&self);
    fn DispatchFormControlChangeEvent(&self);
}
pub fn SetNonTextInputValue(
    element: &dyn InputValueOperations,
    sanitized_value: &[u16],
    changed: bool,
    behavior: TextFieldEventBehavior,
) {
    SetNonTextInputValueOwner(
        element,
        &foundation::NullableUTF16String::from(sanitized_value),
        changed,
        behavior,
    );
}
pub fn SetNonTextInputValueOwner(
    element: &dyn InputValueOperations,
    sanitized_value: &foundation::NullableUTF16String,
    changed: bool,
    behavior: TextFieldEventBehavior,
) {
    if behavior == TextFieldEventBehavior::kDispatchNoEvent {
        element.SetNonAttributeValueOwner(sanitized_value);
    } else {
        element.SetNonAttributeValueOwnerByUserEdit(sanitized_value);
    }
    if !changed {
        return;
    }
    match behavior {
        TextFieldEventBehavior::kDispatchNoEvent => {}
        TextFieldEventBehavior::kDispatchInputEvent => element.DispatchInputEvent(),
        TextFieldEventBehavior::kDispatchChangeEvent => element.DispatchFormControlChangeEvent(),
        TextFieldEventBehavior::kDispatchInputAndChangeEvent => {
            element.DispatchInputEvent();
            element.DispatchFormControlChangeEvent();
        }
    }
}

impl Type {
    // Source: input_type.cc:937-966. Chromium deliberately uses this typed
    // switch rather than virtual dispatch for its hot step capability query.
    pub fn IsSteppable(self) -> bool {
        matches!(
            self,
            Self::kDate
                | Self::kDateTimeLocal
                | Self::kMonth
                | Self::kTime
                | Self::kWeek
                | Self::kNumber
                | Self::kRange
        )
    }
}
/// Lifetime token for the source's DOM EventQueueScope. The embedding owns
/// the scoped queue; base InputType and derived families share this typed token.
pub trait EventQueueScope {}

// Source: input_type.cc:1038-1140,1153-1263. The resident numeric subclass
// supplies its virtual values/effects; stepping decisions remain in InputType.
// This is an internal class interface, not a Browser implementation callback.
pub trait NumericInputType {
    fn CreateNumericStepRange(
        &self,
        handling: super::step_range::AnyStepHandling,
    ) -> super::step_range::StepRange<crate::platform::wtf::decimal::Decimal>;
    fn ParseCurrentNumber(&self) -> crate::platform::wtf::decimal::Decimal;
    fn DefaultNumberForStepUp(&self) -> crate::platform::wtf::decimal::Decimal;
    fn StepAttributeIsAny(&self) -> bool;
    fn NumberFromDouble(&self, value: f64) -> crate::platform::wtf::decimal::Decimal;
    fn SetNumericValue(
        &self,
        value: crate::platform::wtf::decimal::Decimal,
        behavior: TextFieldEventBehavior,
    );
    fn BeginNumericEventQueueScope(&self) -> Box<dyn EventQueueScope>;
    fn DispatchStepBeforeInputInsertText(&self, data: &str) -> bool;
    fn NotifyNumericAccessibilityValueChanged(&self);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericStepError {
    NoAllowedValueStep,
}
impl NumericStepError {
    pub fn Message(self) -> &'static str {
        "This form element does not have an allowed value step."
    }
}
pub fn ApplyNumericStep(
    owner: &dyn NumericInputType,
    current: crate::platform::wtf::decimal::Decimal,
    current_was_invalid: bool,
    mut count: f64,
    handling: super::step_range::AnyStepHandling,
    behavior: TextFieldEventBehavior,
) -> Result<(), NumericStepError> {
    let range = owner.CreateNumericStepRange(handling);
    if !range.HasStep() {
        return Err(NumericStepError::NoAllowedValueStep);
    }
    if range.Minimum() > range.Maximum() {
        return Ok(());
    }
    let aligned_maximum = range.StepSnappedMaximum();
    if !aligned_maximum.IsFinite() {
        return Ok(());
    }
    let base = range.StepBase();
    let step = range.Step();
    let _scope = owner.BeginNumericEventQueueScope();
    let mut new_value = current;
    let step_is_any = owner.StepAttributeIsAny();
    if !step_is_any && range.StepMismatch(current) {
        debug_assert!(!step.IsZero());
        if count < 0.0 {
            new_value = base + ((new_value - base) / step).Floor() * step;
            count += 1.0;
        } else if count > 0.0 {
            new_value = base + ((new_value - base) / step).Ceil() * step;
            count -= 1.0;
        }
    }
    new_value = new_value + range.Step() * owner.NumberFromDouble(count);
    if !step_is_any {
        new_value = range.AlignValueForStep(current, new_value);
    }
    if new_value < range.Minimum() {
        let aligned_minimum = base + ((range.Minimum() - base) / step).Ceil() * step;
        debug_assert!(aligned_minimum >= range.Minimum());
        new_value = aligned_minimum;
    }
    if new_value > range.Maximum() {
        new_value = aligned_maximum;
    }
    debug_assert!(!current_was_invalid || current.IsZero());
    if !current_was_invalid
        && ((count < 0.0 && current < new_value) || (count > 0.0 && current > new_value))
    {
        return Ok(());
    }
    if behavior == TextFieldEventBehavior::kDispatchChangeEvent
        && !owner.DispatchStepBeforeInputInsertText(&new_value.ToString())
    {
        return Ok(());
    }
    owner.SetNumericValue(new_value, behavior);
    owner.NotifyNumericAccessibilityValueChanged();
    Ok(())
}
pub fn StepUpNumeric(owner: &dyn NumericInputType, count: f64) -> Result<(), NumericStepError> {
    let mut current = owner.ParseCurrentNumber();
    let invalid = current.IsNaN();
    if invalid {
        current = crate::platform::wtf::decimal::Decimal::FromInt(0);
    }
    ApplyNumericStep(
        owner,
        current,
        invalid,
        count,
        super::step_range::AnyStepHandling::kRejectAny,
        TextFieldEventBehavior::kDispatchNoEvent,
    )
}
pub fn StepUpNumericFromLayoutObject(owner: &dyn NumericInputType, count: i32) {
    use super::step_range::AnyStepHandling;
    use crate::platform::wtf::decimal::Decimal;
    debug_assert_ne!(count, 0);
    if count == 0 {
        return;
    }
    let range = owner.CreateNumericStepRange(AnyStepHandling::kAnyIsDefaultStep);
    if !range.HasStep() {
        return;
    }
    let _scope = owner.BeginNumericEventQueueScope();
    let step = range.Step();
    let zero = Decimal::FromInt(0);
    let sign = if step > zero {
        count
    } else if step < zero {
        -count
    } else {
        0
    };
    let mut current = owner.ParseCurrentNumber();
    if !current.IsFinite() {
        current = owner.DefaultNumberForStepUp();
        let next_diff = step * Decimal::FromInt(count);
        if current < range.Minimum() - next_diff {
            current = range.Minimum() - next_diff;
        }
        if current > range.Maximum() - next_diff {
            current = range.Maximum() - next_diff;
        }
        owner.SetNumericValue(current, TextFieldEventBehavior::kDispatchNoEvent);
    }
    if (sign > 0 && current < range.Minimum()) || (sign < 0 && current > range.Maximum()) {
        owner.SetNumericValue(
            if sign > 0 {
                range.Minimum()
            } else {
                range.Maximum()
            },
            TextFieldEventBehavior::kDispatchChangeEvent,
        );
        return;
    }
    if (sign > 0 && current >= range.Maximum()) || (sign < 0 && current <= range.Minimum()) {
        return;
    }
    let _ = ApplyNumericStep(
        owner,
        current,
        false,
        f64::from(count),
        AnyStepHandling::kAnyIsDefaultStep,
        TextFieldEventBehavior::kDispatchChangeEvent,
    );
}

// Source: each concrete InputType::CountUsage -> CountUsageIfVisible,
// input_type.cc:1266-1272. Instrumentation owns visibility gating and counters.
#[derive(Clone, Copy)]
pub enum InputTypeUsageFeature {
    Date,
    DateTimeLocal,
    Month,
    Time,
    Week,
    Number,
    Password,
    PasswordMaxLength,
    Email,
    EmailMaxLength,
    EmailMultiple,
    EmailMultipleMaxLength,
    Telephone,
    URL,
}
pub trait InputTypeUsageCounter {
    fn CountUsageIfVisible(&self, feature: InputTypeUsageFeature);
}

// Source: input_type.cc:547-552. The same source class decides whether these
// pseudo invalidations are relevant; the DOM/Style owner applies each value.
#[derive(Clone, Copy)]
pub enum InputRangePseudoState {
    InRange,
    OutOfRange,
}
pub trait InputRangePseudoObserver {
    fn PseudoStateChanged(&self, pseudo: InputRangePseudoState);
}
pub fn InRangeChanged(input_type: Type, observer: &dyn InputRangePseudoObserver) {
    if input_type.IsSteppable() {
        observer.PseudoStateChanged(InputRangePseudoState::InRange);
        observer.PseudoStateChanged(InputRangePseudoState::OutOfRange);
    }
}

impl InputTypeFormData for interaction_dispatch::core::fileapi::form_data::FormData {
    fn AppendFromElement(&mut self, name: Rc<[u16]>, value: Rc<[u16]>) {
        interaction_dispatch::core::fileapi::form_data::FormData::AppendFromElement(
            self, &name, &value,
        );
    }
}

// Source: input_type.cc:1274-1282. The same HTML parser/Decimal owner is used
// for min, value and the default, including range keyboard stepping.
pub fn FindStepBase<D: super::step_range::Decimal>(
    minimum: &str,
    value: &str,
    default_value: D,
    parser: &dyn super::step_range::NumberTypeParser<D>,
) -> D {
    let step_base = parser.ParseToDecimalForNumberType(minimum, D::Nan());
    if step_base.IsFinite() {
        step_base
    } else {
        parser.ParseToDecimalForNumberType(value, default_value)
    }
}

// A source scoped-event guard owns its real queue drain; the marker carries
// that existing RAII lifetime across the independent default-action boundary.
impl<'a> EventQueueScope
    for interaction_dispatch::core::dom::events::scoped_event_queue::EventQueueScope<'a>
{
}

// Source virtual dispatch: base_button_input_type.cc:91-93;
// base_checkable_input_type.cc:106-108; hidden_input_type.cc:78-80;
// file_input_type.cc:252-254; range_input_type.cc:96-98;
// text_field_input_type.cc:132-134; base_temporal_input_type.cc:63-65;
// color_input_type.cc:97-99; image_input_type.h:41 (BaseButtonInputType).
pub fn ValueModeForType(input_type: Type) -> ValueMode {
    match input_type {
        Type::kButton | Type::kReset | Type::kSubmit | Type::kImage | Type::kHidden => {
            ValueMode::kDefault
        }
        Type::kCheckbox | Type::kRadio => ValueMode::kDefaultOn,
        Type::kFile => ValueMode::kFilename,
        _ => ValueMode::kValue,
    }
}
