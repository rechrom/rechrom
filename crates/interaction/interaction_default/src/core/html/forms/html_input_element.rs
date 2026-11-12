/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004-2012 Apple Inc. All rights reserved.
 * Copyright (C) 2010-2012 Google Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the source copyright and this
 * notice are retained. THIS SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/html_input_element.h:423-425
// Source: third_party/blink/renderer/core/html/forms/html_input_element.cc:1566-1701

use std::rc::Rc;

use interaction_dispatch::core::dom::events::event::{EventRef, EventTargetRef};

use super::input_type_view::ClickHandlingState;
use crate::public::mojom::forms::form_control_type::FormControlType;

// Source: html_input_element.cc:1576-1579,1608-1612. DynamicTo remains with
// the concrete DOM event owner; this projection only exposes the source value.
pub trait HTMLInputEventProjection {
    fn MouseButton(&self, event: &EventRef) -> Option<i16>;
}

// Source: html_input_element.cc:1674-1680. This retains the same form owner
// across the change-event reentry point rather than looking it up a second
// time through a copied DOM model.
pub trait HTMLInputFormSubmission {
    fn SubmitImplicitly(&self, event: &EventRef, can_trigger: bool);
}
pub type HTMLInputFormRef = Rc<dyn HTMLInputFormSubmission>;

// Source: html_input_element.cc:1566-1600,1608-1697. Concrete InputTypeView
// subclasses implement these calls on their existing owner. The trait is a
// package boundary, not a replacement view hierarchy.
pub trait HTMLInputTypeViewOperations {
    fn AccessKeyAction(
        &self,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    );
    fn HandleFocusInEvent(&self, old: Option<EventTargetRef>, focus_type: foundation::FocusType);
    fn HandleBlurEvent(&self);

    fn ShouldSubmitImplicitly(&self, event: &EventRef) -> bool;
    fn LegacyPreActivationBehavior(&self) -> Option<ClickHandlingState>;
    fn RunInputActivationBehavior(&self, event: &EventRef, state: &ClickHandlingState);
    fn HandleClickEvent(&self, event: &EventRef);
    fn HandleKeydownEvent(&self, event: &EventRef);
    fn HandleDOMActivateEvent(&self, event: &EventRef);
    fn HandleKeypressEvent(&self, event: &EventRef);
    fn HandleKeyupEvent(&self, event: &EventRef);
    fn FormForSubmission(&self) -> Option<HTMLInputFormRef>;
    fn HandleBeforeTextInsertedEvent(&self, event: &EventRef);
    fn HandleMouseDownEvent(&self, event: &EventRef);
    fn ForwardEvent(&self, event: &EventRef);
}

// Source: html_input_element.cc:1603-1700. These operations are dispatched to
// the resident HTMLInputElement/TextControlElement and document task runner.
// No DOM state or queued task is retained by this translated method object.
pub trait HTMLInputElementOperations {
    fn SupportsPlaceholder(&self) -> bool;
    fn UpdatePlaceholderVisibility(&self);

    fn ResetImpl(&self);
    fn InputDisabledHandlerFixEnabled(&self) -> bool;
    fn IsDisabledFormControl(&self) -> bool;
    fn IsTextField(&self) -> bool;
    fn IsSteppable(&self) -> bool;
    fn FormControlType(&self) -> FormControlType;
    fn PostSearchTask(&self);
    fn DispatchFormControlChangeEvent(&self);
    fn CanTriggerImplicitSubmission(&self) -> bool;
    fn TextControlDefaultEventHandler(&self, event: &EventRef);
}

pub struct HTMLInputElement<'a> {
    element_: &'a dyn HTMLInputElementOperations,
    input_type_view_: &'a dyn HTMLInputTypeViewOperations,
    event_projection_: &'a dyn HTMLInputEventProjection,
}

impl<'a> HTMLInputElement<'a> {
    // Source: html_input_element.cc:837-840.
    pub fn AccessKeyAction(
        &self,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    ) {
        self.input_type_view_.AccessKeyAction(scope);
    }

    // Source: html_input_element.cc:1713-1720. The disabled class still
    // consults its real inherited Node listener/editability virtual.
    pub fn WillRespondToMouseClickEvents(&self, base: impl FnOnce() -> bool) -> bool {
        !self.element_.IsDisabledFormControl() || base()
    }

    // Source: html_input_element.cc:2559-2569.
    pub fn NativeArrowKeyAxes(
        &self,
        base: &dyn crate::core::html::html_element::HTMLElementNativeArrowKeyState,
    ) -> foundation::focusgroup_flags::FocusgroupFlags {
        NativeArrowKeyAxes(
            self.element_.IsTextField(),
            self.element_.IsSteppable(),
            self.element_.FormControlType() == FormControlType::kInputRadio,
            base,
        )
    }
    pub fn new(
        element: &'a dyn HTMLInputElementOperations,
        input_type_view: &'a dyn HTMLInputTypeViewOperations,
        event_projection: &'a dyn HTMLInputEventProjection,
    ) -> Self {
        Self {
            element_: element,
            input_type_view_: input_type_view,
            event_projection_: event_projection,
        }
    }

    // Source: html_input_element.cc:402-410. Only DOMFocusIn invokes the
    // selected InputTypeView; focusin and DOMFocusIn remain separate events.
    pub fn PrepareFocusInEvent(
        &self,
        event_type: &str,
        old: Option<EventTargetRef>,
        focus_type: foundation::FocusType,
    ) {
        if event_type == "DOMFocusIn" {
            self.input_type_view_.HandleFocusInEvent(old, focus_type);
        }
    }
    // Source: html_input_element.cc:413-415.
    pub fn HandleBlurEvent(&self) {
        self.input_type_view_.HandleBlurEvent();
    }

    // Source: html_input_element.cc:1566-1582.
    pub fn LegacyPreActivationBehavior(&self, event: &EventRef) -> Option<ClickHandlingState> {
        let event_type = event.borrow().BaseEvent().r#type().to_owned();
        if event_type == "textInput" && self.input_type_view_.ShouldSubmitImplicitly(event) {
            event.borrow_mut().BaseEventMut().stopPropagation();
            return None;
        }
        if event_type != "click" {
            return None;
        }
        if self.event_projection_.MouseButton(event) != Some(0) {
            return None;
        }
        self.input_type_view_.LegacyPreActivationBehavior()
    }

    // Source: html_input_element.cc:1584-1600.
    pub fn RunActivationBehavior(&self, event: &EventRef, state: Option<&ClickHandlingState>) {
        let Some(state) = state else { return };
        self.input_type_view_
            .RunInputActivationBehavior(event, state);
    }

    // Source: html_input_element.h:423-425;
    // html_input_element.cc:1603-1701.
    pub fn DefaultEventHandler(&self, event: &EventRef) {
        if self.element_.InputDisabledHandlerFixEnabled() && self.element_.IsDisabledFormControl() {
            return;
        }

        let (event_type, is_mouse, is_keyboard, is_before_text_inserted) = {
            let event = event.borrow();
            (
                event.BaseEvent().r#type().to_owned(),
                event.IsMouseEvent(),
                event.IsKeyboardEvent(),
                event.IsBeforeTextInsertedEvent(),
            )
        };

        if is_mouse && event_type == "click" && self.event_projection_.MouseButton(event) == Some(0)
        {
            self.input_type_view_.HandleClickEvent(event);
            if event.borrow().BaseEvent().DefaultHandled() {
                return;
            }
        }

        if is_keyboard && event_type == "keydown" {
            self.input_type_view_.HandleKeydownEvent(event);
            if event.borrow().BaseEvent().DefaultHandled() {
                return;
            }
        }

        let call_base_class_early =
            self.element_.IsTextField() && matches!(event_type.as_str(), "keydown" | "keypress");
        if call_base_class_early {
            self.element_.TextControlDefaultEventHandler(event);
            if event.borrow().BaseEvent().DefaultHandled() {
                return;
            }
        }

        if event_type == "DOMActivate" {
            self.input_type_view_.HandleDOMActivateEvent(event);
            if event.borrow().BaseEvent().DefaultHandled() {
                return;
            }
        }

        if is_keyboard && event_type == "keypress" {
            self.input_type_view_.HandleKeypressEvent(event);
            if event.borrow().BaseEvent().DefaultHandled() {
                return;
            }
        }

        if is_keyboard && event_type == "keyup" {
            self.input_type_view_.HandleKeyupEvent(event);
            if event.borrow().BaseEvent().DefaultHandled() {
                return;
            }
        }

        if self.input_type_view_.ShouldSubmitImplicitly(event) {
            if self.element_.FormControlType() == FormControlType::kInputSearch {
                self.element_.PostSearchTask();
            }
            self.element_.DispatchFormControlChangeEvent();
            if let Some(form) = self.input_type_view_.FormForSubmission() {
                form.SubmitImplicitly(event, self.element_.CanTriggerImplicitSubmission());
            }
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
            return;
        }

        if is_before_text_inserted {
            self.input_type_view_.HandleBeforeTextInsertedEvent(event);
        }

        if is_mouse && event_type == "mousedown" {
            self.input_type_view_.HandleMouseDownEvent(event);
            if event.borrow().BaseEvent().DefaultHandled() {
                return;
            }
        }

        self.input_type_view_.ForwardEvent(event);
        if !call_base_class_early && !event.borrow().BaseEvent().DefaultHandled() {
            self.element_.TextControlDefaultEventHandler(event);
        }
    }
}

// Source: html_input_element.cc:2559-2569. Scalar leaf also used by live DOM
// focus navigation, without constructing unrelated event-handler adapters.
pub fn NativeArrowKeyAxes(
    is_text_field: bool,
    is_steppable: bool,
    is_radio: bool,
    base: &dyn crate::core::html::html_element::HTMLElementNativeArrowKeyState,
) -> foundation::focusgroup_flags::FocusgroupFlags {
    use foundation::focusgroup_flags::FocusgroupFlags as F;
    if is_text_field || is_steppable || is_radio {
        F::kInline | F::kBlock
    } else {
        crate::core::html::html_element::NativeArrowKeyAxes(base)
    }
}

// Source: html_input_element.cc:1192-1198,1216-1260. The DOM owner performs
// its resident checkedness/group/invalidation mutation; all reentrant event
// emission remains in the same source order and outside DOM borrows.
pub trait HTMLInputCheckedOperations {
    fn SetAutofillNotFilled(&self);
    fn SetDirtyCheckedness(&self);
    fn Checked(&self) -> bool;
    fn WillUpdateCheckedness(&self, checked: bool);
    fn SetCheckedAndUpdateGroup(&self, checked: bool);
    fn CheckedStateChanged(&self);
    fn IsConnected(&self) -> bool;
    fn ShouldSendChangeEventAfterCheckedChanged(&self) -> bool;
    fn DispatchInputEvent(&self);
    fn DispatchChangeEvent(&self);
}

pub fn SetInputChecked(
    owner: &dyn HTMLInputCheckedOperations,
    checked: bool,
    behavior: super::text_control_element::TextFieldEventBehavior,
) {
    use super::text_control_element::TextFieldEventBehavior;
    owner.SetAutofillNotFilled();
    owner.SetDirtyCheckedness();
    if owner.Checked() == checked {
        return;
    }
    owner.WillUpdateCheckedness(checked);
    owner.SetCheckedAndUpdateGroup(checked);
    owner.CheckedStateChanged();
    if behavior == TextFieldEventBehavior::kDispatchInputAndChangeEvent
        && owner.IsConnected()
        && owner.ShouldSendChangeEventAfterCheckedChanged()
    {
        owner.DispatchInputEvent();
    }
    owner.SetAutofillNotFilled();
}

pub fn DispatchInputCheckedEvents(owner: &dyn HTMLInputCheckedOperations) {
    if owner.IsConnected() && owner.ShouldSendChangeEventAfterCheckedChanged() {
        owner.DispatchInputEvent();
        owner.DispatchChangeEvent();
    }
}

/// Source: html_input_element.cc:358-386. The caller selects this virtual path
/// only for IsTextField(); non-text inputs inherit TextControl/Element's path.
pub trait HTMLInputTextFocusOperations {
    fn Select(&self);
    fn RestoreCachedSelection(&self);
    fn HasLayoutObject(&self) -> bool;
    fn ScrollRectIntoView(&self);
    fn HasFrame(&self) -> bool;
    fn RevealSelection(&self);
}
pub fn UpdateInputTextSelectionOnFocus(
    owner: &dyn HTMLInputTextFocusOperations,
    behavior: crate::core::dom::element::SelectionBehaviorOnFocus,
    prevent_scroll: bool,
) {
    use crate::core::dom::element::SelectionBehaviorOnFocus as S;
    match behavior {
        S::kReset => owner.Select(),
        S::kRestore => owner.RestoreCachedSelection(),
        S::kNone => return,
    }
    if !prevent_scroll {
        if owner.HasLayoutObject() {
            owner.ScrollRectIntoView();
        }
        if owner.HasFrame() {
            owner.RevealSelection();
        }
    }
}

/// The source InputType virtual facts sampled before changing its owner.
/// This is a small value, never a parallel control or view representation.
#[derive(Clone, Copy)]
pub struct InputTypeChangeCapabilities {
    pub value_mode: super::input_type::ValueMode,
    pub respect_size: bool,
    pub successful_submit: bool,
    pub required: bool,
    pub read_only: bool,
    pub checkable: bool,
    pub steppable: bool,
    pub list: bool,
    pub text_field: bool,
    pub placeholder: bool,
    pub selection: bool,
    pub telephone: bool,
    pub auto_directionality: bool,
}
#[derive(Clone, Copy, Debug)]
pub enum InputTypeChangePseudo {
    Required,
    Optional,
    ReadOnly,
    ReadWrite,
    Checked,
    Indeterminate,
    InRange,
    OutOfRange,
    HasDatalist,
    TextField,
    PlaceholderShown,
}
/// Synchronous owner operations at the exact source call points. No command
/// is buffered past listener reentry; the Browser routes each to its resident
/// DOM, InputType/View, FrameSelection, radio or form owner immediately.
pub enum InputTypeChangeEffect {
    RemoveFromRadioButtonGroup,
    ClosePopupView,
    DestroyShadowSubtree,
    DropInnerEditor,
    ForceReattachLayoutTree,
    Pseudo(InputTypeChangePseudo),
    WillBeDestroyed,
    InstallNewTypeAndView,
    UpdateDirectionality,
    CreateShadowSubtreeWithoutUpdatingView,
    UpdateWillValidateCache,
    UpdatePlaceholder,
    WarnIfValueIsInvalid(Rc<[u16]>),
    SetValueAttribute(Rc<[u16]>),
    StoreValue {
        value: foundation::NullableUTF16String,
        dirty: bool,
    },
    SetValue {
        value: foundation::NullableUTF16String,
        dirty: bool,
    },
    UpdateView,
    InvalidateSizeAttributes,
    ScheduleSelectionAfterLayout,
    ClearValueBeforeFirstUserEdit,
    AddToRadioButtonGroup,
    SetSelectionToStart,
    DisconnectAllOpaqueRanges,
    UpdateHasBeenPasswordField,
    MaybeSetHasBeenHeuristicCustomPasswordJS,
    SetNeedsValidityCheck,
    InvalidateDefaultButtonStyle,
    NotifyFormStateChanged,
}
pub trait HTMLInputTypeChangeOperations {
    fn Apply(&self, effect: InputTypeChangeEffect);
    fn IsRequired(&self) -> bool;
    fn DirectionAttribute(&self) -> Option<String>;
    fn HasDirtyValue(&self) -> bool;
    fn NonAttributeValue(&self) -> foundation::NullableUTF16String;
    fn ValueAttribute(&self) -> Option<Rc<[u16]>>;
    fn SanitizeNewValue(
        &self,
        value: &foundation::NullableUTF16String,
    ) -> foundation::NullableUTF16String;
    fn ConvertSanitizedValueToStateValue(
        &self,
        value: foundation::NullableUTF16String,
    ) -> foundation::NullableUTF16String;
    fn IsFocused(&self) -> bool;
    fn OpaqueRangeEnabled(&self) -> bool;
    fn HasConnectedFormOwner(&self) -> bool;
}
/// Source: html_input_element.cc:462-681. Creation/normalization and the
/// equal-type early return occur in the factory before entering this method.
pub fn UpdateInputType(
    owner: &dyn HTMLInputTypeChangeOperations,
    old: InputTypeChangeCapabilities,
    new: InputTypeChangeCapabilities,
) {
    use super::input_type::ValueMode as V;
    use InputTypeChangeEffect as E;
    use InputTypeChangePseudo as P;
    owner.Apply(E::RemoveFromRadioButtonGroup);
    owner.Apply(E::ClosePopupView);
    owner.Apply(E::DestroyShadowSubtree);
    owner.Apply(E::DropInnerEditor);
    owner.Apply(E::ForceReattachLayoutTree);
    if old.required != new.required && owner.IsRequired() {
        owner.Apply(E::Pseudo(P::Required));
        owner.Apply(E::Pseudo(P::Optional));
    }
    if old.read_only != new.read_only {
        owner.Apply(E::Pseudo(P::ReadOnly));
        owner.Apply(E::Pseudo(P::ReadWrite));
    }
    if old.checkable != new.checkable {
        owner.Apply(E::Pseudo(P::Checked));
    }
    owner.Apply(E::Pseudo(P::Indeterminate));
    if old.steppable || new.steppable {
        owner.Apply(E::Pseudo(P::InRange));
        owner.Apply(E::Pseudo(P::OutOfRange));
    }
    if old.list != new.list {
        owner.Apply(E::Pseudo(P::HasDatalist));
    }
    if old.text_field != new.text_field {
        owner.Apply(E::Pseudo(P::TextField));
    }
    owner.Apply(E::WillBeDestroyed);
    owner.Apply(E::InstallNewTypeAndView);
    let dir = owner.DirectionAttribute();
    if (dir.is_none() && (old.telephone || new.telephone))
        || (dir
            .as_deref()
            .is_some_and(|dir| dir.eq_ignore_ascii_case("auto"))
            && (old.auto_directionality || new.auto_directionality))
    {
        owner.Apply(E::UpdateDirectionality);
    }
    owner.Apply(E::CreateShadowSubtreeWithoutUpdatingView);
    owner.Apply(E::UpdateWillValidateCache);
    if old.placeholder != new.placeholder {
        owner.Apply(E::UpdatePlaceholder);
        owner.Apply(E::Pseudo(P::PlaceholderShown));
    }
    if old.value_mode == V::kValue && matches!(new.value_mode, V::kDefault | V::kDefaultOn) {
        let value = owner.NonAttributeValue();
        if owner.HasDirtyValue() && !value.is_empty() {
            owner.Apply(E::SetValueAttribute(value.Owner()));
        }
        owner.Apply(E::StoreValue {
            value: foundation::NullableUTF16String::Null(),
            dirty: false,
        });
    } else if old.value_mode != V::kValue && new.value_mode == V::kValue {
        let value = foundation::NullableUTF16String::FromAttribute(owner.ValueAttribute());
        owner.Apply(E::WarnIfValueIsInvalid(value.Owner()));
        let value = owner.ConvertSanitizedValueToStateValue(owner.SanitizeNewValue(&value));
        owner.Apply(E::StoreValue {
            value,
            dirty: false,
        });
    } else if old.value_mode != V::kFilename && new.value_mode == V::kFilename {
        owner.Apply(E::StoreValue {
            value: foundation::NullableUTF16String::Null(),
            dirty: false,
        });
    } else {
        if !owner.HasDirtyValue() {
            if let Some(value) = owner.ValueAttribute() {
                owner.Apply(E::WarnIfValueIsInvalid(value));
            }
        }
        if new.value_mode == V::kValue {
            let previous = owner.NonAttributeValue();
            let mut value = owner.SanitizeNewValue(&previous);
            let dirty = owner.HasDirtyValue();
            if !dirty {
                value = owner.ConvertSanitizedValueToStateValue(value);
            }
            if value.as_ref() != previous.as_ref() {
                owner.Apply(E::SetValue { value, dirty });
            }
        }
    }
    owner.Apply(E::UpdateView);
    if old.respect_size != new.respect_size {
        owner.Apply(E::InvalidateSizeAttributes);
    }
    if owner.IsFocused() {
        owner.Apply(E::ScheduleSelectionAfterLayout);
    }
    owner.Apply(E::ClearValueBeforeFirstUserEdit);
    owner.Apply(E::AddToRadioButtonGroup);
    if !old.selection && new.selection {
        owner.Apply(E::SetSelectionToStart);
    }
    if old.selection && owner.OpaqueRangeEnabled() {
        owner.Apply(E::DisconnectAllOpaqueRanges);
    }
    owner.Apply(E::UpdateHasBeenPasswordField);
    owner.Apply(E::MaybeSetHasBeenHeuristicCustomPasswordJS);
    owner.Apply(E::SetNeedsValidityCheck);
    if (old.successful_submit || new.successful_submit) && owner.HasConnectedFormOwner() {
        owner.Apply(E::InvalidateDefaultButtonStyle);
    }
    owner.Apply(E::NotifyFormStateChanged);
}

/// Source: html_input_element.cc:1137-1147. The same retained class owns its
/// value mode, checkable state and base form-control reset, in source order.
pub trait HTMLInputResetOperations {
    fn ValueMode(&self) -> super::input_type::ValueMode;
    fn DefaultValue(&self) -> foundation::NullableUTF16String;
    fn SetNonDirtyValue(&self, value: &foundation::NullableUTF16String);
    fn SetNeedsValidityCheck(&self);
    fn SetCheckedFromDefaultAttribute(&self);
    fn ClearDirtyCheckedness(&self);
    fn BaseResetImpl(&self);
}
pub fn ResetInputElement(owner: &dyn HTMLInputResetOperations) {
    use super::input_type::ValueMode;
    match owner.ValueMode() {
        ValueMode::kValue => {
            owner.SetNonDirtyValue(&owner.DefaultValue());
            owner.SetNeedsValidityCheck();
        }
        ValueMode::kFilename => {
            owner.SetNonDirtyValue(&foundation::NullableUTF16String::Null());
            owner.SetNeedsValidityCheck();
        }
        _ => {}
    }
    owner.SetCheckedFromDefaultAttribute();
    owner.ClearDirtyCheckedness();
    owner.BaseResetImpl();
}

// Source: html_input_element.cc:2625-2630,2670-2743. These are the input
// subclass's command virtuals; the real HTMLElement base target stays separate.
pub trait HTMLInputCommandOperations {
    fn IsNumberInputType(&self) -> bool;
    fn IsMutable(&self) -> bool;
    fn FrameIsCrossOrigin(&self) -> bool;
    fn HasTransientUserActivation(&self) -> bool;
    fn AddJavaScriptConsoleWarning(&self, warning: &str);
    fn OpenInputPopupView(&self);
    fn StepUpInput(&self, count: f64);
}
pub fn IsValidInputBuiltinCommand(
    parent_is_valid: bool,
    actions_v2_enabled: bool,
    number_input: bool,
    command: crate::core::dom::element::CommandEventType,
) -> bool {
    use crate::core::dom::element::CommandEventType as C;
    if !actions_v2_enabled || parent_is_valid {
        return parent_is_valid;
    }
    if number_input && matches!(command, C::kStepUp | C::kStepDown) {
        return true;
    }
    command == C::kShowPicker
}
pub fn HandleInputCommandInternal(
    element: &dyn HTMLInputCommandOperations,
    command: crate::core::dom::element::CommandEventType,
    parent_handled: impl FnOnce() -> bool,
) -> bool {
    use crate::core::dom::element::CommandEventType as C;
    if parent_handled() {
        return true;
    }
    if !element.IsMutable() {
        return false;
    }
    if command == C::kShowPicker {
        if element.FrameIsCrossOrigin() {
            element
                .AddJavaScriptConsoleWarning("Input cannot be invoked from cross-origin iframe.");
            return false;
        }
        if !element.HasTransientUserActivation() {
            element.AddJavaScriptConsoleWarning("Input cannot be invoked without a user gesture.");
            return false;
        }
        element.OpenInputPopupView();
        return true;
    }
    if element.IsNumberInputType() {
        if command == C::kStepUp {
            element.StepUpInput(1.0);
            return true;
        }
        if command == C::kStepDown {
            element.StepUpInput(-1.0);
            return true;
        }
    }
    false
}

// Source: html_input_element.cc:1533-1563. Renderer values have already been
// sanitized; this source path preserves the same dirty-value and event order.
pub trait HTMLInputRendererValueOperations {
    fn IsFileInputType(&self) -> bool;
    fn ClearSuggestedValueWithoutViewUpdate(&self);
    fn SanitizeUserInputValue(&self, value: &[u16]) -> Rc<[u16]>;
    fn SetValueBeforeFirstUserEditIfNotSet(&self);
    fn StoreRendererValueAndSetDirty(&self, value: &[u16]);
    fn HasInnerEditor(&self) -> bool;
    fn ClearNeedsToUpdateViewValue(&self);
    fn CheckIfValueWasReverted(&self, value: &[u16]);
    fn IsTextField(&self) -> bool;
    fn DispatchInputEvent(&self);
    fn NotifyFormStateChanged(&self);
    fn SetNeedsValidityCheck(&self);
    fn ClearAutofillState(&self);
}
pub fn SetInputValueFromRenderer(element: &dyn HTMLInputRendererValueOperations, value: &[u16]) {
    debug_assert!(!element.IsFileInputType());
    element.ClearSuggestedValueWithoutViewUpdate();
    let sanitized = element.SanitizeUserInputValue(value);
    debug_assert!(sanitized.as_ref() == value || sanitized.is_empty());
    element.SetValueBeforeFirstUserEditIfNotSet();
    element.StoreRendererValueAndSetDirty(value);
    if element.HasInnerEditor() {
        element.ClearNeedsToUpdateViewValue();
    }
    element.CheckIfValueWasReverted(value);
    if !element.IsTextField() {
        element.DispatchInputEvent();
    }
    element.NotifyFormStateChanged();
    element.SetNeedsValidityCheck();
    element.ClearAutofillState();
}

// Source: html_input_element.cc:1736-1793. MIME token grammar is RFC2616;
// extensions deliberately accept every value beginning with a dot (length>=2).
fn IsRFC2616TokenCharacter(ch: u16) -> bool {
    ch <= 0x7f
        && ch > 0x20
        && !matches!(ch, 0x22 | 0x28 | 0x29 | 0x2c | 0x2f | 0x7b | 0x7d | 0x7f)
        && !(0x3a..=0x40).contains(&ch)
        && !(0x5b..=0x5d).contains(&ch)
}
fn ParseAcceptAttribute(value: &str, predicate: impl Fn(&[u16]) -> bool) -> Vec<String> {
    let mut types = Vec::new();
    for part in value.split(',') {
        let part = part.trim_matches(|ch| matches!(ch, '\t' | '\n' | '\u{000c}' | '\r' | ' '));
        if part.is_empty() || !predicate(&part.encode_utf16().collect::<Vec<_>>()) {
            continue;
        }
        types.push(
            foundation::String::FromUtf8(part.as_bytes())
                .DeprecatedLower()
                .Utf8(),
        );
    }
    types
}
pub fn InputAcceptMIMETypes(value: &str) -> Vec<String> {
    ParseAcceptAttribute(value, |value| {
        let Some(slash) = value.iter().position(|ch| *ch == 0x2f) else {
            return false;
        };
        slash != 0
            && slash + 1 != value.len()
            && value
                .iter()
                .enumerate()
                .all(|(i, ch)| i == slash || IsRFC2616TokenCharacter(*ch))
    })
}
pub fn InputAcceptFileExtensions(value: &str) -> Vec<String> {
    ParseAcceptAttribute(value, |value| value.len() >= 2 && value[0] == 0x2e)
}

// Source: html_input_element.cc:923-947. Attribute storage and selector-key
// invalidation have already happened; this is the input's owning parse stage.
pub trait HTMLInputValueAttributeOperations {
    fn HasDirtyValue(&self) -> bool;
    fn ValueModeIsValue(&self) -> bool;
    fn SanitizeAndStoreAttributeStateValue(&self, value: &foundation::NullableUTF16String);
    fn UpdatePlaceholderVisibility(&self);
    fn SetNeedsSubtreeStyleRecalcForValueAttribute(&self);
    fn SetNeedsToUpdateViewValue(&self);
    fn SetNeedsValidityCheck(&self);
    fn WarnIfValueIsInvalidAndElementIsVisible(&self, value: &[u16]);
    fn InRangeChanged(&self);
    fn HasCreatedShadowSubtree(&self) -> bool;
    fn NeedsShadowSubtree(&self) -> bool;
    fn ValueAttributeChangedView(&self);
    fn SetNeedsUpdateViewInCreateShadowSubtree(&self);
}
pub fn ParseInputValueAttribute(
    element: &dyn HTMLInputValueAttributeOperations,
    value: &foundation::NullableUTF16String,
) {
    if !element.HasDirtyValue() {
        if element.ValueModeIsValue() {
            element.SanitizeAndStoreAttributeStateValue(value);
        }
        element.UpdatePlaceholderVisibility();
        element.SetNeedsSubtreeStyleRecalcForValueAttribute();
        element.SetNeedsToUpdateViewValue();
    }
    element.SetNeedsValidityCheck();
    element.WarnIfValueIsInvalidAndElementIsVisible(value);
    element.InRangeChanged();
    if element.HasCreatedShadowSubtree() || !element.NeedsShadowSubtree() {
        element.ValueAttributeChangedView();
    } else {
        element.SetNeedsUpdateViewInCreateShadowSubtree();
    }
}

// Source: html_input_element.cc:1388-1407,1410-1456. Keep common SetValue
// gates/order on HTMLInputElement, with retained InputType virtual calls.
use super::text_control_element::{TextControlSetValueSelection, TextFieldEventBehavior};
use crate::public::web::web_autofill_state::WebAutofillState;
pub trait HTMLInputValueOperations {
    fn IsFileInputType(&self) -> bool;
    fn Value(&self) -> foundation::NullableUTF16String;
    fn IsAutofilled(&self) -> bool;
    fn WarnIfInvalidAndVisible(&self, value: &[u16]);
    fn CanSetValue(&self, value: &[u16]) -> bool;
    fn ClearSuggestedValue(&self);
    fn SetAutofillState(&self, state: WebAutofillState);
    fn BeginEventQueueScope(&self) -> Box<dyn super::input_type::EventQueueScope>;
    fn SanitizeValue(
        &self,
        value: &foundation::NullableUTF16String,
    ) -> foundation::NullableUTF16String;
    fn SetLastChangeWasNotUserEdit(&self);
    fn SetNeedsToUpdateViewValue(&self);
    fn SetTypeValue(
        &self,
        value: &foundation::NullableUTF16String,
        changed: bool,
        behavior: TextFieldEventBehavior,
        selection: TextControlSetValueSelection,
    );
    fn DidSetValueOnCurrentView(&self, value: &[u16], changed: bool);
    fn NotifyFormStateChanged(&self);
    fn HasBeenPasswordField(&self) -> bool;
    fn HasChromeClient(&self) -> bool;
    fn PasswordFieldReset(&self);
}
#[derive(Clone, Copy, Debug)]
pub struct InputValueBindingError;
impl InputValueBindingError {
    pub fn Message(self) -> &'static str {
        "This input element accepts a filename, which may only be programmatically set to the empty string."
    }
}
pub fn SetInputValueForBinding(
    element: &dyn HTMLInputValueOperations,
    value: &[u16],
) -> Result<(), InputValueBindingError> {
    if element.IsFileInputType() && !value.is_empty() {
        return Err(InputValueBindingError);
    }
    let old_value = element.Value();
    let was_autofilled = element.IsAutofilled();
    let changed = old_value != foundation::NullableUTF16String::from(value);
    let autofill = if was_autofilled && !changed && !value.is_empty() {
        WebAutofillState::kAutofilled
    } else {
        WebAutofillState::kNotFilled
    };
    SetInputValue(
        element,
        value,
        TextFieldEventBehavior::kDispatchNoEvent,
        TextControlSetValueSelection::kSetSelectionToEnd,
        autofill,
    );
    Ok(())
}
pub fn SetInputValue(
    element: &dyn HTMLInputValueOperations,
    value: &[u16],
    behavior: TextFieldEventBehavior,
    selection: TextControlSetValueSelection,
    autofill: WebAutofillState,
) {
    SetInputValueOwner(
        element,
        &foundation::NullableUTF16String::from(value),
        behavior,
        selection,
        autofill,
    );
}
pub fn SetInputValueOwner(
    element: &dyn HTMLInputValueOperations,
    value: &foundation::NullableUTF16String,
    behavior: TextFieldEventBehavior,
    selection: TextControlSetValueSelection,
    autofill: WebAutofillState,
) {
    element.WarnIfInvalidAndVisible(value);
    if !element.CanSetValue(value) {
        return;
    }
    element.ClearSuggestedValue();
    element.SetAutofillState(autofill);
    {
        let _scope = element.BeginEventQueueScope();
        let sanitized = element.SanitizeValue(value);
        let changed = sanitized != element.Value();
        element.SetLastChangeWasNotUserEdit();
        element.SetNeedsToUpdateViewValue();
        element.SetTypeValue(&sanitized, changed, behavior, selection);
        element.DidSetValueOnCurrentView(&sanitized, changed);
        if changed {
            element.NotifyFormStateChanged();
            if sanitized.is_empty() && element.HasBeenPasswordField() && element.HasChromeClient() {
                element.PasswordFieldReset();
            }
        }
    }
    element.SetAutofillState(autofill);
}

// Source: html_input_element.cc:389-400. SpellChecker and ChromeClient are
// the same owning frame services; the element decides whether editing ends.
pub trait HTMLInputEndEditingOperations {
    fn DocumentIsActive(&self) -> bool;
    fn IsTextField(&self) -> bool;
    fn SpellCheckerDidEndEditing(&self);
    fn ChromeClientDidEndEditing(&self);
}
pub fn EndInputEditing(element: &dyn HTMLInputEndEditingOperations) {
    debug_assert!(element.DocumentIsActive());
    if !element.DocumentIsActive() || !element.IsTextField() {
        return;
    }
    element.SpellCheckerDidEndEditing();
    element.ChromeClientDidEndEditing();
}

impl super::text_control_element::TextControlFocusEventOperations for HTMLInputElement<'_> {
    fn SupportsPlaceholder(&self) -> bool {
        self.element_.SupportsPlaceholder()
    }
    fn UpdatePlaceholderVisibility(&self) {
        self.element_.UpdatePlaceholderVisibility();
    }
    fn HandleFocusEvent(&self, old: Option<EventTargetRef>, focus_type: foundation::FocusType) {
        super::text_control_element::TextControlElement::BaseHandleFocusEvent(old, focus_type);
    }
    fn HandleBlurEvent(&self) {
        HTMLInputElement::HandleBlurEvent(self);
    }
}

/// HTMLInputElement::HasValidDataListOptions, cc2100-2110. This query checks
/// option emptiness/disabled state; type parsing belongs to chooser suggestions.
pub trait DataListValiditySource {
    fn OptionCount(&self) -> usize;
    fn OptionValidityFacts(&self, index: usize) -> Option<(bool, bool)>;
}
pub fn HasValidDataListOptions(source: &dyn DataListValiditySource) -> bool {
    for index in 0..source.OptionCount() {
        let Some((empty, disabled)) = source.OptionValidityFacts(index) else {
            break;
        };
        if !empty && !disabled {
            return true;
        }
    }
    false
}
