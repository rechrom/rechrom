// Copyright (C) 2010 Google Inc. All rights reserved.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/text_field_input_type.h:40-54,64,66-72,83-84,92-95,99
// Source: third_party/blink/renderer/core/html/forms/text_field_input_type.cc:118-134,146-263,404-446,612-712,762-784,795-813
// The existing element owns its value and selection. UTF-16 code units retain
// WTF::String indexing, including unpaired surrogates supplied by bindings.

use super::input_type::{Type, ValueMode};
use super::text_control_element::{TextControlSetValueSelection, TextFieldEventBehavior};
use interaction_dispatch::core::dom::events::event::EventRef;
use std::rc::Rc;

pub type TextString = Vec<u16>;
pub type TextValue = Rc<[u16]>;
/// HandleKeydownEventForSpinButton physical-to-logical mapping, cc:342-351.
pub fn SpinButtonKeyDirections(
    direction: Option<foundation::WritingDirectionMode>,
) -> (&'static str, &'static str) {
    use foundation::{PhysicalToLogical, TextDirection, WritingDirectionMode, WritingMode};
    let direction = direction.unwrap_or_else(|| {
        WritingDirectionMode::new(WritingMode::kHorizontalTb, TextDirection::kLtr)
    });
    let keys = PhysicalToLogical::new(direction, "ArrowUp", "ArrowRight", "ArrowDown", "ArrowLeft");
    (keys.LineOver(), keys.LineUnder())
}

/// A source-time editing geometry readiness value. The facade obtains it at
/// text_field_input_type.cc:666-667 after any synchronous listener reentry.
/// This value never drives a rendering lifecycle from the default-action crate.
pub struct PreparedTextSelection {
    pub selected_text_length: usize,
    pub root_is_descendant_of_control: bool,
}

pub trait TextFieldValue {
    fn Value(&self) -> TextValue;
    fn InnerEditorValue(&self) -> TextValue;
    fn VisibleValue(&self) -> TextValue;
    fn SetNonAttributeValue(&self, value: &[u16]) {
        self.SetNonAttributeValueOwner(&foundation::NullableUTF16String::from(value));
    }
    fn SetNonAttributeValueByUserEdit(&self, value: &[u16]) {
        self.SetNonAttributeValueOwnerByUserEdit(&foundation::NullableUTF16String::from(value));
    }
    fn SetNonAttributeValueOwner(&self, value: &foundation::NullableUTF16String);
    fn SetNonAttributeValueOwnerByUserEdit(&self, value: &foundation::NullableUTF16String);
    fn UpdateView(&self);
    fn IsRequired(&self) -> bool;
    fn IsDisabledOrReadOnly(&self) -> bool;
    fn IsFocused(&self) -> bool;
    fn MaxLength(&self) -> i32;
    fn HasDirtyValue(&self) -> bool;
    fn LastChangeWasUserEdit(&self) -> bool;
    fn MinLength(&self) -> i32;
    fn Multiple(&self) -> bool;
    fn Pattern(&self) -> Option<TextValue>;
}

pub trait TextFieldEffects {
    fn OpaqueRangeEnabled(&self) -> bool;
    fn ShouldSkipNextSetValueAutoDiff(&self) -> bool;
    fn CommitProgrammaticOpaqueRangeEdit(&self, old: &[u16], start: usize, end: usize);
    fn SetSelectionRange(&self, start: usize, end: usize);
    fn DispatchInputEvent(&self);
    fn DispatchFormControlChangeEvent(&self);
    fn HasFilterTarget(&self) -> bool;
    fn DispatchBeforeFilterEvent(&self) -> bool;
    fn DataListOptions(&self) -> Option<Rc<dyn TextFieldFilterCollection>>;
    fn FilterTargetOptions(&self) -> Option<Rc<dyn TextFieldFilterCollection>>;
    fn DidClearValueInTextField(&self);
    fn HasChromeClient(&self) -> bool;
    fn DidChangeValueInTextField(&self);
    fn IsBaseAppearanceCombobox(&self) -> bool;
    fn DataListPopoverOpen(&self) -> bool;
    fn ShowDataListPopover(&self);
    fn SetValueFromRenderer(&self, value: &[u16]);
    fn UpdatePlaceholderVisibility(&self);
    fn PseudoStateChanged(&self, state: TextFieldPseudoState);
    fn CommitOpaqueRangeEdit(&self);
}

pub trait TextFieldFilterOption {
    fn DisplayLabel(&self) -> TextValue;
    fn SetFiltered(&self, filtered: bool);
}
pub trait TextFieldFilterCollection {
    fn Len(&self) -> usize;
    fn Item(&self, index: usize) -> Rc<dyn TextFieldFilterOption>;
}
pub trait TextFieldInteraction {
    fn ChooseActiveDataListOption(&self);
    fn MoveActiveDataListOption(&self, backwards: bool);
    fn HasFilterTarget(&self) -> bool;
    fn ToggleActiveFilterTargetOption(&self, event: &EventRef);
    fn MoveActiveFilterTargetOption(&self, backwards: bool);
    fn StartFilterTargetFiltering(&self);
    fn HandleKeyboardEventOnEditableElement(&self, event: &EventRef) -> bool;
    fn EndEditing(&self);
    fn SpinButton(&self) -> Option<Rc<super::spin_button_element::SpinButtonElement>>;
    fn WheelRegistration(&self) -> &dyn TextFieldWheelRegistration;
    fn FocusedElementIsOutsideOpenDataList(&self) -> bool;
    fn HideDataListPopoverWithoutMovingFocus(&self);
    fn StopFilterTargetFiltering(&self);
}

// Source: text_field_input_type.cc:363-402. Existing spin-button, layout and
// inner-editor owners perform their own work; this interface carries no tree.
pub trait TextFieldWheelRegistration {
    fn HasFrame(&self) -> bool;
    fn HasSpinButton(&self) -> bool;
    fn HasLayoutObject(&self) -> bool;
    fn DidAddBlockingWheelEventHandler(&self);
    fn DidRemoveBlockingWheelEventHandler(&self);
}
pub trait TextFieldSpinFocus {
    fn FocusHostForUserGesture(&self);
    fn SetSelectionRange(&self, start: usize, end: usize);
}
pub trait TextFieldEventForwarding {
    fn ForwardSpinButtonEvent(&self, event: &EventRef);
    fn HasLayoutObject(&self) -> bool;
    fn GetForceReattachLayoutTree(&self) -> bool;
    fn ResetInnerEditorScrollOffset(&self);
    fn ForwardInnerEditorEvent(&self, event: &EventRef);
}

#[derive(Clone, Copy)]
pub enum TextFieldPseudoState {
    Valid,
    Invalid,
    UserValid,
    UserInvalid,
    InRange,
    OutOfRange,
}

/// The two source base subobjects retain the same element identity through
/// these value/effect projections. Neither projection stores another control.
pub struct TextFieldInputType {
    pub input_type: Type,
    pub value: Rc<dyn TextFieldValue>,
    pub effects: Rc<dyn TextFieldEffects>,
    has_registered_wheel_event_handler_: std::cell::Cell<bool>,
}

impl TextFieldInputType {
    // Source: text_field_input_type.cc:118-134.
    pub fn new(
        input_type: Type,
        value: Rc<dyn TextFieldValue>,
        effects: Rc<dyn TextFieldEffects>,
    ) -> Self {
        Self {
            input_type,
            value,
            effects,
            has_registered_wheel_event_handler_: std::cell::Cell::new(false),
        }
    }
    // Source: text_field_input_type.cc:844-861.
    pub fn FocusAndSelectSpinButtonOwner(&self, focus: &dyn TextFieldSpinFocus) {
        focus.FocusHostForUserGesture();
        focus.SetSelectionRange(0, i32::MAX as usize);
    }
    pub fn ShouldSpinButtonRespondToMouseEvents(&self) -> bool {
        !self.value.IsDisabledOrReadOnly()
    }
    pub fn ShouldSpinButtonRespondToWheelEvents(&self) -> bool {
        self.ShouldSpinButtonRespondToMouseEvents() && self.value.IsFocused()
    }
    pub fn SpinButtonDidReleaseMouseCapture(
        &self,
        dispatch: super::spin_button_element::EventDispatch,
    ) {
        if dispatch == super::spin_button_element::EventDispatch::Allowed {
            self.effects.DispatchFormControlChangeEvent();
        }
    }
    // Source: text_field_input_type.cc:938-956. Registration state belongs to
    // this same source view, rather than to a native widget or Page adapter.
    pub fn UpdateWheelEventRegistration(
        &self,
        registration: &dyn TextFieldWheelRegistration,
        detaching: bool,
    ) {
        if !registration.HasFrame() {
            return;
        }
        let should_register = !detaching
            && registration.HasSpinButton()
            && registration.HasLayoutObject()
            && self.ShouldSpinButtonRespondToWheelEvents();
        if should_register && !self.has_registered_wheel_event_handler_.get() {
            registration.DidAddBlockingWheelEventHandler();
            self.has_registered_wheel_event_handler_.set(true);
        } else if !should_register && self.has_registered_wheel_event_handler_.get() {
            registration.DidRemoveBlockingWheelEventHandler();
            self.has_registered_wheel_event_handler_.set(false);
        }
    }
    pub fn CreateView(&self) -> &Self {
        self
    }
    pub fn GetValueMode(&self) -> ValueMode {
        ValueMode::kValue
    }
    // Source: text_field_input_type.cc:146-159.
    pub fn MayTriggerVirtualKeyboard(&self) -> bool {
        true
    }
    pub fn ValueMissing(&self, value: &[u16]) -> bool {
        Self::ValueMissingWithState(
            value,
            self.value.IsRequired(),
            self.value.IsDisabledOrReadOnly(),
        )
    }
    pub fn ValueMissingWithState(
        value: &[u16],
        required: bool,
        disabled_or_readonly: bool,
    ) -> bool {
        required && value.is_empty() && !disabled_or_readonly
    }
    pub fn CanSetSuggestedValue(&self) -> bool {
        true
    }
    // Source: text_field_input_type.cc:612-614.
    pub fn SupportsReadOnly(&self) -> bool {
        true
    }

    // Source: text_field_input_type.cc:161-238.
    pub fn SetValue(
        &self,
        sanitized: &[u16],
        changed: bool,
        behavior: TextFieldEventBehavior,
        selection: TextControlSetValueSelection,
    ) {
        self.SetValueOwner(
            &foundation::NullableUTF16String::from(sanitized),
            changed,
            behavior,
            selection,
        );
    }
    pub fn SetValueOwner(
        &self,
        sanitized: &foundation::NullableUTF16String,
        changed: bool,
        behavior: TextFieldEventBehavior,
        selection: TextControlSetValueSelection,
    ) {
        let old = self.value.Value();
        if behavior == TextFieldEventBehavior::kDispatchNoEvent {
            self.value.SetNonAttributeValueOwner(sanitized);
        } else {
            self.value.SetNonAttributeValueOwnerByUserEdit(sanitized);
        }
        let update = changed
            || (behavior == TextFieldEventBehavior::kDispatchNoEvent
                && sanitized.as_ref() != self.value.InnerEditorValue().as_ref());
        if update {
            self.value.UpdateView();
        }
        if !changed {
            return;
        }
        if self.effects.OpaqueRangeEnabled() && !self.effects.ShouldSkipNextSetValueAutoDiff() {
            self.effects
                .CommitProgrammaticOpaqueRangeEdit(&old, 0, old.len());
        }
        if selection == TextControlSetValueSelection::kSetSelectionToEnd {
            let end = self.value.VisibleValue().len();
            self.effects.SetSelectionRange(end, end);
        }
        match behavior {
            TextFieldEventBehavior::kDispatchChangeEvent => {
                if self.value.IsFocused() {
                    self.effects.DispatchInputEvent();
                } else {
                    self.effects.DispatchFormControlChangeEvent();
                }
            }
            TextFieldEventBehavior::kDispatchInputEvent => self.effects.DispatchInputEvent(),
            TextFieldEventBehavior::kDispatchInputAndChangeEvent => {
                self.effects.DispatchInputEvent();
                self.effects.DispatchFormControlChangeEvent();
            }
            TextFieldEventBehavior::kDispatchNoEvent => {}
        }
        self.FilterOptions();
    }

    // Source: text_field_input_type.cc:240-263. The named combobox/select
    // operations are the translated owners of those separate source helpers.
    pub fn HandleKeydownEvent(&self, event: &EventRef, interaction: &dyn TextFieldInteraction) {
        if !self.value.IsFocused() {
            return;
        }
        if self.effects.IsBaseAppearanceCombobox() {
            if self.HandleKeydownForCustomizableCombobox(event, interaction) {
                event.borrow_mut().BaseEventMut().SetDefaultHandled();
            }
            return;
        }
        if self.HandleKeydownForFilterableSelect(event, interaction) {
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
            return;
        }
        if self.effects.HasChromeClient() && interaction.HandleKeyboardEventOnEditableElement(event)
        {
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        }
    }

    // Source: text_field_input_type.cc:266-335. Read scalar keyboard data
    // before any option/popover operation can synchronously run script.
    fn PlainKey(event: &EventRef) -> Option<String> {
        let event = event.borrow();
        let key = event.AsKeyboardEvent()?;
        (!key.ctrlKey() && !key.altKey() && !key.metaKey() && !key.shiftKey())
            .then(|| key.key().to_owned())
    }
    fn HandleKeydownForCustomizableCombobox(
        &self,
        event: &EventRef,
        interaction: &dyn TextFieldInteraction,
    ) -> bool {
        let Some(key) = Self::PlainKey(event) else {
            return false;
        };
        if key == "Enter" && self.effects.DataListPopoverOpen() {
            interaction.ChooseActiveDataListOption();
            return true;
        }
        if key == "ArrowUp" || key == "ArrowDown" {
            if self.effects.DataListPopoverOpen() {
                interaction.MoveActiveDataListOption(key == "ArrowUp");
            } else {
                self.effects.ShowDataListPopover();
            }
            return true;
        }
        false
    }
    fn HandleKeydownForFilterableSelect(
        &self,
        event: &EventRef,
        interaction: &dyn TextFieldInteraction,
    ) -> bool {
        if !interaction.HasFilterTarget() {
            return false;
        }
        let Some(key) = Self::PlainKey(event) else {
            return false;
        };
        match key.as_str() {
            "Enter" => interaction.ToggleActiveFilterTargetOption(event),
            "ArrowUp" => interaction.MoveActiveFilterTargetOption(true),
            "ArrowDown" => interaction.MoveActiveFilterTargetOption(false),
            _ => return false,
        }
        true
    }
    // Source: text_field_input_type.cc:861-879,938-956.
    pub fn HandleFocusInEvent(&self, interaction: &dyn TextFieldInteraction) {
        if self.effects.IsBaseAppearanceCombobox() {
            self.effects.ShowDataListPopover();
        } else if interaction.HasFilterTarget() {
            interaction.StartFilterTargetFiltering();
        }
        self.UpdateWheelEventRegistration(interaction.WheelRegistration(), false);
    }
    // Source: text_field_input_type.cc:898-925. Collections retain native DOM
    // option owners. There is no filtered copy of the select/datalist tree.
    pub fn FilterOptions(&self) {
        if !self.effects.IsBaseAppearanceCombobox() && !self.effects.HasFilterTarget() {
            return;
        }
        if !self.effects.DispatchBeforeFilterEvent() {
            return;
        }
        let options = if self.effects.IsBaseAppearanceCombobox() {
            self.effects.DataListOptions()
        } else if self.effects.HasFilterTarget() {
            self.effects.FilterTargetOptions()
        } else {
            None
        };
        let Some(options) = options else { return };
        for index in 0..options.Len() {
            let option = options.Item(index);
            let label = foundation::String::from_utf16(&option.DisplayLabel()).FoldCase();
            let input = foundation::String::from_utf16(&self.value.Value()).FoldCase();
            let label = label.Span16().unwrap_or(&[]);
            let input = input.Span16().unwrap_or(&[]);
            let matches = input.is_empty() || label.windows(input.len()).any(|part| part == input);
            option.SetFiltered(!matches);
        }
    }
    // Source: text_field_input_type.cc:523-528.
    pub fn DestroyShadowSubtree(
        &self,
        interaction: &dyn TextFieldInteraction,
        destroy_base: impl FnOnce(),
    ) {
        destroy_base();
        if let Some(spin) = interaction.SpinButton() {
            spin.RemoveSpinButtonOwner();
        }
        self.UpdateWheelEventRegistration(interaction.WheelRegistration(), false);
    }
    // Source: text_field_input_type.cc:363-402. All live facts are read after
    // spin-button dispatch, since that dispatch may synchronously run script.
    pub fn ForwardEvent(&self, event: &EventRef, owner: &dyn TextFieldEventForwarding) {
        owner.ForwardSpinButtonEvent(event);
        if event.borrow().BaseEvent().DefaultHandled() {
            return;
        }
        if event.borrow().BaseEvent().r#type() == "input" {
            self.FilterOptions();
        }
        let (forward, blur) = {
            let event = event.borrow();
            let kind = event.BaseEvent().r#type();
            (
                event.IsMouseEvent()
                    || event.IsDragEvent()
                    || event.HasInterface("WheelEvent")
                    || matches!(kind, "blur" | "focus"),
                kind == "blur",
            )
        };
        if owner.HasLayoutObject() && !owner.GetForceReattachLayoutTree() && forward {
            if blur {
                owner.ResetInnerEditorScrollOffset();
            }
            owner.ForwardInnerEditorEvent(event);
        }
    }

    // Source: text_field_input_type.cc:404-436. InputTypeView::HandleBlurEvent
    // is empty; no RefCell borrow crosses the source EndEditing dispatch.
    pub fn HandleBlurEvent(&self, interaction: &dyn TextFieldInteraction) {
        interaction.EndEditing();
        if let Some(spin) = interaction.SpinButton() {
            spin.ReleaseCapture(super::spin_button_element::EventDispatch::Allowed);
        }
        self.UpdateWheelEventRegistration(interaction.WheelRegistration(), false);
        if self.effects.IsBaseAppearanceCombobox()
            && interaction.FocusedElementIsOutsideOpenDataList()
        {
            interaction.HideDataListPopoverWithoutMovingFocus();
        }
        interaction.StopFilterTargetFiltering();
    }

    // Source: text_field_input_type.cc:438-446. TextEvent supplies these exact
    // scalar flags; the caller retains the original EventRef during dispatch.
    pub fn ShouldSubmitImplicitly(
        &self,
        text_event: Option<(bool, bool, &[u16])>,
        base_result: bool,
    ) -> bool {
        if let Some((paste, drop, data)) = text_event {
            if !paste && !drop && data == [b'\n' as u16] {
                return true;
            }
        }
        base_result
    }

    // Source: text_field_input_type.cc:636-646.
    pub fn SanitizeValue(&self, proposed: &[u16]) -> TextString {
        Self::SanitizeValueFor(proposed)
    }
    pub fn SanitizeValueFor(proposed: &[u16]) -> TextString {
        if !proposed.iter().any(|&c| MayBeASCIILineBreak(c)) && proposed.len() < i32::MAX as usize {
            return proposed.to_vec();
        }
        let removed: TextString = proposed
            .iter()
            .copied()
            .filter(|&c| !IsASCIILineBreak(c))
            .collect();
        LimitLength(&removed, i32::MAX as usize)
    }

    // Source: text_field_input_type.cc:648-712. Layout readiness at 666-667 is
    // represented by `prepared`, acquired at that source point by the facade.
    pub fn HandleBeforeTextInsertedEvent(
        &self,
        text: &mut TextString,
        prepared: Option<&PreparedTextSelection>,
        delegates_focus_fix: bool,
    ) {
        let old_length = self.value.InnerEditorValue().len();
        let mut selection_length = 0;
        if self.value.IsFocused() {
            let selected = prepared.expect("focused text insertion requires prepared selection");
            if !delegates_focus_fix || selected.root_is_descendant_of_control {
                selection_length = selected.selected_text_length;
            }
        }
        debug_assert!(old_length >= selection_length);
        if FilterBeforeTextInserted(text, old_length, selection_length, self.value.MaxLength())
            && self.effects.HasChromeClient()
        {
            self.effects.DidClearValueInTextField();
        }
    }

    // Source: text_field_input_type.cc:762-784.
    pub fn ConvertFromVisibleValue(&self, visible: &[u16]) -> TextString {
        visible.to_vec()
    }
    pub fn SubtreeHasChanged(&self) {
        let value =
            self.SanitizeValue(&self.ConvertFromVisibleValue(&self.value.InnerEditorValue()));
        self.CommitSubtreeValue(&value);
        self.DidSetValueByUserEdit();
    }
    // Source: text_field_input_type.cc:768-783. Derived input types compute
    // their virtual ConvertFromVisibleValue/SanitizeUserInputValue first, then
    // share these exact commit/invalidation points before their user-edit hook.
    pub fn CommitSubtreeValue(&self, value: &[u16]) {
        self.effects.SetValueFromRenderer(value);
        self.effects.UpdatePlaceholderVisibility();
        for state in [
            TextFieldPseudoState::Valid,
            TextFieldPseudoState::Invalid,
            TextFieldPseudoState::UserValid,
            TextFieldPseudoState::UserInvalid,
            TextFieldPseudoState::InRange,
            TextFieldPseudoState::OutOfRange,
        ] {
            self.effects.PseudoStateChanged(state);
        }
        if self.effects.OpaqueRangeEnabled() {
            self.effects.CommitOpaqueRangeEdit();
        }
    }
    // Source: text_field_input_type.cc:795-813.
    pub fn DidSetValueByUserEdit(&self) {
        if !self.value.IsFocused() {
            return;
        }
        if self.effects.HasChromeClient() {
            if self.value.Value().is_empty() {
                self.effects.DidClearValueInTextField();
            }
            self.effects.DidChangeValueInTextField();
        }
        if self.effects.IsBaseAppearanceCombobox() && !self.effects.DataListPopoverOpen() {
            self.effects.ShowDataListPopover();
        }
    }
}

// Source: text_field_input_type.cc:616-634.
pub fn IsASCIILineBreak(c: u16) -> bool {
    c == 13 || c == 10
}
fn MayBeASCIILineBreak(c: u16) -> bool {
    c <= 13
}
pub fn LimitLength(string: &[u16], max_length: usize) -> TextString {
    let mut len = max_length.min(string.len());
    if len == string.len() {
        return string.to_vec();
    }
    if len > 0 && (0xd800..=0xdbff).contains(&string[len - 1]) {
        len -= 1;
    }
    string[..len].to_vec()
}

/// text_field_input_type.cc:670-712. Shared by the original TextFieldInputType
/// owner and its call-scoped paste default-handler projection. The return value
/// is the source DidClearValueInTextField condition; the embedding owns Chrome.
pub fn FilterBeforeTextInserted(
    text: &mut TextString,
    old_length: usize,
    selection_length: usize,
    max: i32,
) -> bool {
    debug_assert!(old_length >= selection_length);
    let appendable = (if max < 0 {
        i32::MAX as usize
    } else {
        max as usize
    })
    .saturating_sub(old_length - selection_length);
    let mut len = text.len();
    while len > 0 && IsASCIILineBreak(text[len - 1]) {
        len -= 1;
    }
    let mut event_text = Vec::with_capacity(len);
    let mut index = 0;
    while index < len {
        let c = text[index];
        if c == 13 && index + 1 < len && text[index + 1] == 10 {
            index += 1;
        }
        event_text.push(if IsASCIILineBreak(c) { 32 } else { c });
        index += 1;
    }
    *text = LimitLength(&event_text, appendable);
    selection_length == old_length && selection_length != 0 && !event_text.is_empty()
}
