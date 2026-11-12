/* Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004, 2005, 2006, 2007, 2008 Apple Inc. All rights reserved.
 * This library is free software under the GNU Library General Public License,
 * version 2 or (at your option) any later version; see Chromium COPYING.LIB.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/html_text_area_element.h:52-58,100-108,122-124,153,159-160
// Source: third_party/blink/renderer/core/html/forms/html_text_area_element.cc:94-102,379-415,417-548,550-559,575-685

use super::{
    text_control_element::{
        TextControlElement, TextControlSetValueSelection, TextFieldEventBehavior,
    },
    text_field_input_type::{TextString, TextValue},
};
use crate::core::dom::element::SelectionBehaviorOnFocus;
use crate::platform::wtf::text::line_ending::NormalizeLineEndingsToLf;
use crate::public::web::web_autofill_state::WebAutofillState;
use interaction_dispatch::core::dom::events::event::EventRef;
use std::rc::Rc;

pub trait TextAreaFormDataOperations {
    type Error;
    fn NameForFormData(&self) -> TextValue;
    fn UpdateStyleAndLayoutForForm(&self) -> Result<(), Self::Error>;
    fn WrapAttribute(&self) -> TextValue;
    fn ValueWithHardLineBreaks(&self) -> Result<TextValue, Self::Error>;
    fn Value(&self) -> TextValue;
    fn DirnameAndDirectionForFormData(&self) -> Option<(TextValue, TextValue)>;
}
struct SourceTextAreaFormData<'a>(&'a HTMLTextAreaElement);
impl TextAreaFormDataOperations for SourceTextAreaFormData<'_> {
    type Error = std::convert::Infallible;
    fn NameForFormData(&self) -> TextValue {
        self.0.value.NameForFormData()
    }
    fn UpdateStyleAndLayoutForForm(&self) -> Result<(), Self::Error> {
        self.0.effects.UpdateStyleAndLayoutForForm();
        Ok(())
    }
    fn WrapAttribute(&self) -> TextValue {
        self.0.value.WrapAttribute()
    }
    fn ValueWithHardLineBreaks(&self) -> Result<TextValue, Self::Error> {
        Ok(self.0.value.ValueWithHardLineBreaks())
    }
    fn Value(&self) -> TextValue {
        self.0.value.Value()
    }
    fn DirnameAndDirectionForFormData(&self) -> Option<(TextValue, TextValue)> {
        self.0.value.DirnameAndDirectionForFormData()
    }
}

pub trait TextAreaResetOperations {
    fn DefaultValue(&self) -> TextValue;
    fn SetNonDirtyValue(&self, value: &[u16], selection: TextControlSetValueSelection);
    fn BaseResetImpl(&self);
}

pub trait TextAreaValue {
    fn IsDirty(&self) -> bool;
    fn NameForFormData(&self) -> TextValue;
    fn WrapAttribute(&self) -> TextValue;
    fn ValueWithHardLineBreaks(&self) -> TextValue;
    fn DirnameAndDirectionForFormData(&self) -> Option<(TextValue, TextValue)>;
    fn Value(&self) -> TextValue;
    fn SetStoredValue(&self, value: &[u16]);
    fn SetInnerEditorValue(&self, value: &[u16]);
    fn InnerEditorValue(&self) -> TextValue;
    fn SerializeInnerEditorValue(&self) -> TextValue;
    fn DefaultValue(&self) -> TextValue;
    fn SetDirty(&self, value: bool);
    fn MaxLength(&self) -> i32;
    fn MinLength(&self) -> i32;
    fn IsRequired(&self) -> bool;
    fn LastChangeWasUserEdit(&self) -> bool;
    fn IsFocused(&self) -> bool;
    fn IsDisabledOrReadOnly(&self) -> bool;
    fn HasFrame(&self) -> bool;
    fn HasPage(&self) -> bool;
    fn HasLayoutObject(&self) -> bool;
}
pub trait TextAreaEffects {
    fn UpdateStyleAndLayoutForForm(&self);
    fn FocusFromUserGesture(&self);
    fn BaseResetImpl(&self);
    fn RevealSelection(&self);
    fn HandleKeyboardEventOnEditableElement(&self, event: &EventRef) -> bool;
    fn AdjustPlaceholderBreakElement(&self);
    fn SetNeedsValidityCheck(&self);
    fn SetAutofillState(&self, state: WebAutofillState);
    fn UpdatePlaceholderVisibility(&self);
    fn HasDirectionAuto(&self) -> bool;
    fn CalculateAndAdjustAutoDirectionality(&self);
    fn CommitOpaqueRangeEdit(&self);
    fn DidClearValueInTextField(&self);
    fn DidChangeValueInTextField(&self);
    fn SetSuggestedValueBaseNull(&self);
    fn SetLastChangeWasNotUserEdit(&self);
    fn InvalidateTextControlValue(&self);
    fn NotifyFormStateChanged(&self);
    fn DispatchInputEvent(&self);
}

/// The control, value and effect projections all retain one DOM textarea owner.
pub struct HTMLTextAreaElement {
    pub control: Rc<TextControlElement>,
    pub value: Rc<dyn TextAreaValue>,
    pub effects: Rc<dyn TextAreaEffects>,
}
impl HTMLTextAreaElement {
    // html_text_area_element.cc:182-189.
    pub fn SaveFormControlState(&self) -> Vec<TextValue> {
        Self::SaveFormControlStateWithValue(self.value.IsDirty(), self.Value())
    }
    pub fn SaveFormControlStateWithValue(dirty: bool, value: TextValue) -> Vec<TextValue> {
        if dirty {
            vec![value]
        } else {
            Vec::new()
        }
    }
    pub fn RestoreFormControlState(&self, state: &[TextValue]) {
        self.SetValue(
            &state[0],
            TextFieldEventBehavior::kDispatchNoEvent,
            TextControlSetValueSelection::kSetSelectionToEnd,
            WebAutofillState::kNotFilled,
        );
    }
    /// html_text_area_element.cc:363-377.
    pub fn AppendToFormData(&self, data: &mut dyn super::input_type::InputTypeFormData) {
        match Self::AppendWithOperations(&SourceTextAreaFormData(self), data) {
            Ok(()) => {}
            Err(never) => match never {},
        }
    }
    pub fn AppendWithOperations<O: TextAreaFormDataOperations>(
        owner: &O,
        data: &mut dyn super::input_type::InputTypeFormData,
    ) -> Result<(), O::Error> {
        let name = owner.NameForFormData();
        if name.is_empty() {
            return Ok(());
        }
        owner.UpdateStyleAndLayoutForForm()?;
        let name = owner.NameForFormData();
        // ParseAttribute(wrap), cc:325-328, including the legacy aliases.
        let wrap = owner.WrapAttribute();
        let hard_wrap = [b"hard".as_slice(), b"physical".as_slice(), b"on".as_slice()]
            .iter()
            .any(|value| {
                wrap.len() == value.len()
                    && wrap.iter().zip(*value).all(|(&unit, &byte)| {
                        unit <= 127 && (unit as u8).eq_ignore_ascii_case(&byte)
                    })
            });
        let text = if hard_wrap {
            owner.ValueWithHardLineBreaks()?
        } else {
            owner.Value()
        };
        data.AppendFromElement(name, text);
        if let Some((name, direction)) = owner.DirnameAndDirectionForFormData() {
            data.AppendFromElement(name, direction);
        }
        Ok(())
    }
    // Source: html_text_area_element.cc:810-812. This override only focuses.
    pub fn AccessKeyAction(&self) {
        self.effects.FocusFromUserGesture();
    }
    // Source: html_text_area_element.h: FormControlType() class constant.
    pub fn FormControlType(
        &self,
    ) -> crate::public::mojom::forms::form_control_type::FormControlType {
        crate::public::mojom::forms::form_control_type::FormControlType::kTextArea
    }

    // Source: html_text_area_element.cc:893-896.
    pub fn NativeArrowKeyAxes(&self) -> foundation::focusgroup_flags::FocusgroupFlags {
        NativeArrowKeyAxes()
    }
    // Source: html_text_area_element.cc:752-807. The ListedElement caller
    // applies willValidate before checking this control's constraint values.
    pub fn ValueMissing(&self, value: &[u16]) -> bool {
        super::text_field_input_type::TextFieldInputType::ValueMissingWithState(
            value,
            self.value.IsRequired(),
            self.value.IsDisabledOrReadOnly(),
        )
    }
    // html_text_area_element.cc:728-750.
    pub fn ValidationMessageUTF16(
        &self,
        will_validate: bool,
        custom: &[u16],
        locale: &dyn super::constraint_validation_message::ConstraintValidationLocale,
    ) -> TextString {
        use super::base_text_input_type::NeedsToCheckDirtyFlag::kCheckDirtyFlag;
        use super::constraint_validation_message::ConstraintValidationText as L;
        if !will_validate {
            return Vec::new();
        }
        if !custom.is_empty() {
            return custom.to_vec();
        }
        let value = self.value.Value();
        if self.ValueMissing(&value) {
            return locale.QueryValidationText(L::ValueMissing, &[], &[]);
        }
        if self.TooLong(&value, kCheckDirtyFlag) {
            return locale.QueryValidationText(
                L::TooLong {
                    length: value.len(),
                    limit: self.value.MaxLength(),
                },
                &[],
                &[],
            );
        }
        if self.TooShort(&value, kCheckDirtyFlag) {
            return locale.QueryValidationText(
                L::TooShort {
                    length: value.len(),
                    limit: self.value.MinLength(),
                },
                &[],
                &[],
            );
        }
        Vec::new()
    }
    pub fn TooLong(
        &self,
        value: &[u16],
        check: super::base_text_input_type::NeedsToCheckDirtyFlag,
    ) -> bool {
        Self::TooLongWithState(
            value,
            self.value.MaxLength(),
            check,
            self.value.LastChangeWasUserEdit(),
        )
    }
    pub fn TooLongWithState(
        value: &[u16],
        max: i32,
        check: super::base_text_input_type::NeedsToCheckDirtyFlag,
        user_edit: bool,
    ) -> bool {
        (check == super::base_text_input_type::NeedsToCheckDirtyFlag::kIgnoreDirtyFlag || user_edit)
            && max >= 0
            && ComputeLengthForAPIValue(value) > max as usize
    }
    pub fn TooShort(
        &self,
        value: &[u16],
        check: super::base_text_input_type::NeedsToCheckDirtyFlag,
    ) -> bool {
        Self::TooShortWithState(
            value,
            self.value.MinLength(),
            check,
            self.value.LastChangeWasUserEdit(),
        )
    }
    pub fn TooShortWithState(
        value: &[u16],
        min: i32,
        check: super::base_text_input_type::NeedsToCheckDirtyFlag,
        user_edit: bool,
    ) -> bool {
        let len = ComputeLengthForAPIValue(value);
        (check == super::base_text_input_type::NeedsToCheckDirtyFlag::kIgnoreDirtyFlag || user_edit)
            && min > 0
            && len > 0
            && len < min as usize
    }
    pub fn IsValidValue(&self, candidate: &[u16]) -> bool {
        use super::base_text_input_type::NeedsToCheckDirtyFlag::kIgnoreDirtyFlag as I;
        !self.ValueMissing(candidate) && !self.TooLong(candidate, I) && !self.TooShort(candidate, I)
    }
    pub fn IsValidControlValue(&self, value: &[u16], has_custom_error: bool) -> bool {
        use super::base_text_input_type::NeedsToCheckDirtyFlag::kCheckDirtyFlag as C;
        !has_custom_error
            && !self.ValueMissing(value)
            && !self.TooLong(value, C)
            && !self.TooShort(value, C)
    }
    // Source: html_text_area_element.cc:379-415.
    pub fn ResetImpl(&self) {
        Self::ResetWithOperations(self);
    }
    // Source: html_text_area_element.cc:379-384.
    pub fn ResetWithOperations(operations: &dyn TextAreaResetOperations) {
        operations.SetNonDirtyValue(
            &operations.DefaultValue(),
            TextControlSetValueSelection::kSetSelectionToEnd,
        );
        operations.BaseResetImpl();
    }
    pub fn HasCustomFocusLogic(&self) -> bool {
        true
    }
    pub fn IsKeyboardFocusableSlow(&self, is_focusable: bool, tab_index: i32) -> bool {
        is_focusable && tab_index >= 0
    }
    pub fn MayTriggerVirtualKeyboard(&self) -> bool {
        true
    }
    pub fn UpdateSelectionOnFocus(&self, behavior: SelectionBehaviorOnFocus, prevent_scroll: bool) {
        match behavior {
            SelectionBehaviorOnFocus::kReset | SelectionBehaviorOnFocus::kRestore => {
                self.control.RestoreCachedSelection()
            }
            SelectionBehaviorOnFocus::kNone => return,
        }
        if !prevent_scroll && self.value.HasFrame() {
            self.effects.RevealSelection();
        }
    }
    // Source: html_text_area_element.cc:417-442. BeforeTextInsertedEvent text
    // storage is passed separately because EventInterface retains its owner.
    pub fn DefaultEventHandler(
        &self,
        event: &EventRef,
        autofill_keydown: bool,
        before_text: Option<&mut TextString>,
        selected_text: Option<&[u16]>,
        clamp_cache: bool,
    ) {
        let (kind, keyboard, mouse, drag, wheel, before) = {
            let event = event.borrow();
            (
                event.BaseEvent().r#type().to_owned(),
                event.IsKeyboardEvent(),
                event.IsMouseEvent(),
                event.IsDragEvent(),
                event.HasInterface("WheelEvent"),
                event.IsBeforeTextInsertedEvent(),
            )
        };
        if autofill_keydown
            && keyboard
            && kind == "keydown"
            && self.value.IsFocused()
            && !self.value.IsDisabledOrReadOnly()
            && self.value.HasPage()
            && self.effects.HandleKeyboardEventOnEditableElement(event)
        {
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
            return;
        }
        if self.value.HasLayoutObject() && (mouse || drag || wheel || kind == "blur") {
            self.control.ForwardEvent(event);
        } else if self.value.HasLayoutObject() && before {
            if let Some(text) = before_text {
                self.HandleBeforeTextInsertedEvent(text, selected_text);
            }
        }
        self.control.DefaultEventHandler(event, clamp_cache);
    }
    // Source: html_text_area_element.cc:444-486. The debug-only descendant
    // assertion reads native DOM nodes and does not affect execution.
    pub fn SubtreeHasChanged(&self) {
        self.effects.AdjustPlaceholderBreakElement();
        self.control.SetValueBeforeFirstUserEditIfNotSet();
        self.UpdateValue();
        self.control.CheckIfValueWasReverted(&self.value.Value());
        self.effects.SetNeedsValidityCheck();
        self.effects.SetAutofillState(WebAutofillState::kNotFilled);
        self.effects.UpdatePlaceholderVisibility();
        if self.effects.HasDirectionAuto() {
            self.effects.CalculateAndAdjustAutoDirectionality();
        }
        if self.control.effects.OpaqueRangeEnabled() {
            self.effects.CommitOpaqueRangeEdit();
        }
        if !self.value.IsFocused() {
            return;
        }
        if self.value.Value().is_empty() {
            self.effects.DidClearValueInTextField();
        }
        self.effects.DidChangeValueInTextField();
    }
    // Source: html_text_area_element.cc:488-527. `selected_text` is supplied
    // from the source-time prepared selection at cc:512.
    pub fn HandleBeforeTextInsertedEvent(
        &self,
        text: &mut TextString,
        selected_text: Option<&[u16]>,
    ) {
        let max = self.value.MaxLength();
        if max < 0 {
            return;
        }
        if FilterBeforeTextInserted(
            text,
            &self.value.InnerEditorValue(),
            max,
            self.value.IsFocused(),
            selected_text,
        ) {
            self.effects.DidClearValueInTextField();
        }
    }
    // Source: html_text_area_element.cc:550-559.
    pub fn UpdateValue(&self) {
        self.value
            .SetStoredValue(&self.value.SerializeInnerEditorValue());
        self.effects.NotifyFormStateChanged();
        self.value.SetDirty(true);
        self.effects.UpdatePlaceholderVisibility();
    }
    pub fn Value(&self) -> TextValue {
        self.value.Value()
    }
    // Source: html_text_area_element.cc:575-589.
    pub fn SetValue(
        &self,
        value: &[u16],
        behavior: TextFieldEventBehavior,
        selection: TextControlSetValueSelection,
        autofill_state: WebAutofillState,
    ) {
        self.SetValueCommon(value, behavior, selection, autofill_state);
        self.value.SetDirty(true);
    }
    pub fn SetNonDirtyValue(&self, value: &[u16], selection: TextControlSetValueSelection) {
        self.SetValueCommon(
            value,
            TextFieldEventBehavior::kDispatchNoEvent,
            selection,
            WebAutofillState::kNotFilled,
        );
        self.value.SetDirty(false);
    }
    // Source: html_text_area_element.cc:591-685.
    pub fn SetValueCommon(
        &self,
        new_value: &[u16],
        behavior: TextFieldEventBehavior,
        selection: TextControlSetValueSelection,
        autofill_state: WebAutofillState,
    ) {
        let normalized = NormalizeLineEndingsToLf(new_value);
        self.effects.SetSuggestedValueBaseNull();
        if normalized.as_ref() == self.value.Value().as_ref() {
            return;
        }
        let old = self.value.Value();
        let clamp = selection == TextControlSetValueSelection::kClamp;
        let start = if clamp {
            self.control.selectionStart()
        } else {
            0
        };
        let end = if clamp {
            self.control.selectionEnd()
        } else {
            0
        };
        if behavior != TextFieldEventBehavior::kDispatchNoEvent {
            self.control.SetValueBeforeFirstUserEditIfNotSet();
        }
        self.value.SetStoredValue(&normalized);
        self.value.SetInnerEditorValue(&normalized);
        if self.control.effects.OpaqueRangeEnabled()
            && !self.control.effects.ShouldSkipNextSetValueAutoDiff()
        {
            self.control
                .effects
                .CommitProgrammaticOpaqueRangeEdit(&old, 0, old.len());
        }
        if behavior == TextFieldEventBehavior::kDispatchNoEvent {
            self.effects.SetLastChangeWasNotUserEdit();
        } else {
            self.control.CheckIfValueWasReverted(&self.value.Value());
        }
        self.effects.UpdatePlaceholderVisibility();
        // Source cc:637-644 emits invalidation to the owning engines.
        self.effects.InvalidateTextControlValue();
        self.effects.SetNeedsValidityCheck();
        match selection {
            TextControlSetValueSelection::kSetSelectionToEnd => {
                let end = self.value.Value().len();
                self.control.SetSelectionRange(end, end, Default::default());
            }
            TextControlSetValueSelection::kSetSelectionToStart => {
                self.control.SetSelectionRange(0, 0, Default::default());
            }
            TextControlSetValueSelection::kClamp => {
                let len = self.value.Value().len();
                self.control
                    .SetSelectionRange(start.min(len), end.min(len), Default::default());
            }
            TextControlSetValueSelection::kDoNotSet => {}
        }
        self.effects.SetAutofillState(autofill_state);
        self.effects.NotifyFormStateChanged();
        match behavior {
            TextFieldEventBehavior::kDispatchChangeEvent => {
                self.control.DispatchFormControlChangeEvent()
            }
            TextFieldEventBehavior::kDispatchInputEvent => self.effects.DispatchInputEvent(),
            TextFieldEventBehavior::kDispatchInputAndChangeEvent => {
                self.effects.DispatchInputEvent();
                self.control.DispatchFormControlChangeEvent();
            }
            TextFieldEventBehavior::kDispatchNoEvent => {}
        }
        self.effects.SetAutofillState(autofill_state);
    }
}

// Source: html_text_area_element.cc:893-896.
pub fn NativeArrowKeyAxes() -> foundation::focusgroup_flags::FocusgroupFlags {
    use foundation::focusgroup_flags::FocusgroupFlags as F;
    F::kInline | F::kBlock
}
// Source: html_text_area_element.cc:94-102.
pub fn ComputeLengthForAPIValue(text: &[u16]) -> usize {
    text.len() - text.windows(2).filter(|pair| pair == &&[13, 10]).count()
}
// Source: html_text_area_element.cc:529-548.
pub fn SanitizeUserInputValue(proposed: &[u16], max: usize) -> TextString {
    let mut submission_length = 0;
    let mut i = 0;
    while i < proposed.len() {
        if proposed[i] == 13 && i + 1 < proposed.len() && proposed[i + 1] == 10 {
            i += 1;
            continue;
        }
        submission_length += 1;
        if submission_length == max {
            i += 1;
            break;
        }
        if submission_length > max {
            break;
        }
        i += 1;
    }
    if i > 0 && (0xd800..=0xdbff).contains(&proposed[i - 1]) {
        i -= 1;
    }
    proposed[..i].to_vec()
}

// Source: html_text_area_element.h:111; TextControlElement.h:345-349.
impl super::text_control_element::TextControlFocusEventOperations for HTMLTextAreaElement {
    fn SupportsPlaceholder(&self) -> bool {
        true
    }
    fn UpdatePlaceholderVisibility(&self) {
        self.effects.UpdatePlaceholderVisibility();
    }
    fn HandleFocusEvent(
        &self,
        old: Option<interaction_dispatch::core::dom::events::event::EventTargetRef>,
        focus_type: foundation::FocusType,
    ) {
        TextControlElement::BaseHandleFocusEvent(old, focus_type);
    }
    fn HandleBlurEvent(&self) {
        TextControlElement::BaseHandleBlurEvent();
    }
}

impl TextAreaResetOperations for HTMLTextAreaElement {
    fn DefaultValue(&self) -> TextValue {
        self.value.DefaultValue()
    }
    fn SetNonDirtyValue(&self, value: &[u16], selection: TextControlSetValueSelection) {
        HTMLTextAreaElement::SetNonDirtyValue(self, value, selection);
    }
    fn BaseResetImpl(&self) {
        self.effects.BaseResetImpl();
    }
}

/// html_text_area_element.cc:512-548. Reuses the retained control's exact
/// WTF-16/API-length filtering in the synchronous paste default handler.
pub fn FilterBeforeTextInserted(
    text: &mut TextString,
    current: &[u16],
    max: i32,
    focused: bool,
    selected_text: Option<&[u16]>,
) -> bool {
    if max < 0 {
        return false;
    }
    let current_length = ComputeLengthForAPIValue(current);
    if current_length + ComputeLengthForAPIValue(text) < max as usize {
        return false;
    }
    let selected = if focused {
        ComputeLengthForAPIValue(
            selected_text.expect("textarea insertion requires prepared selection"),
        )
    } else {
        0
    };
    debug_assert!(current_length >= selected);
    let appendable = (max as usize).saturating_sub(current_length - selected);
    *text = SanitizeUserInputValue(text, appendable);
    selected == current_length && selected != 0 && !text.is_empty()
}
