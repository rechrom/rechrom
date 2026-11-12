/*
 * Copyright (C) 2010 Nokia Corporation and/or its subsidiary(-ies).
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2001 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2004, 2005, 2006, 2007, 2009, 2010, 2011 Apple Inc.
 * All rights reserved.
 *           (C) 2006 Alexey Proskuryakov (ap@nypop.com)
 * Copyright (C) 2010 Google Inc. All rights reserved.
 * Copyright (C) 2009 Torch Mobile Inc. All rights reserved.
 * (http://www.torchmobile.com/)
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB. If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/html/forms/html_select_element.h:493-504
// Source: core/html/forms/html_select_element.cc:778-821,1050-1187,1340-1348,1380-1416,1683-1689,1697-1737
// DOM owns the option collection and selectedness. This wrapper borrows those
// same owners and keeps no second option tree or selected-option cache.

use super::select_type::{SelectBehavior, SelectEnvironment, SelectEvent};
use super::type_ahead::{kCycleFirstChar, kMatchPrefix, TypeAhead, TypeAheadDataSource};
use crate::core::dom::element::CommandEventType;
use interaction_dispatch::core::dom::events::event::{EventRef, EventTargetRef};
use std::rc::Rc;

// Source html_select_element.cc:1860-1924. Base virtual calls borrow the
// existing HTMLElement collaborator, retaining its invoker and command value.
pub trait SelectCommandEnvironment {
    fn HTMLCommandActionsV2Enabled(&self) -> bool;
    fn BaseIsValidBuiltinCommand(
        &self,
        invoker: &EventTargetRef,
        command: CommandEventType,
    ) -> bool;
    fn BaseHandleCommandInternal(
        &self,
        invoker: &EventTargetRef,
        command: CommandEventType,
    ) -> bool;
    fn FrameIsSameOrigin(&self) -> Option<bool>;
    fn HasTransientUserActivation(&self) -> bool;
    fn UpdateStyleAndLayout(&self);
    fn LockedAncestorPreventingPaint(&self) -> bool;
    fn HasLayoutBox(&self) -> bool;
    fn AddCommandWarning(&self, warning: &str);
}

pub type OptionRef = Rc<dyn SelectOption>;
pub trait SelectOption: std::any::Any {
    fn ValueForFormData(&self) -> Rc<[u16]>;
    fn Selected(&self) -> bool;
    fn ValueIsEmpty(&self) -> bool;
    fn DefaultSelected(&self) -> bool;
    fn SetSelectedState(&self, value: bool, skip_mutation_observer: bool);
    fn SetDirty(&self, value: bool);
    fn IsDisabledFormControl(&self) -> bool;
    fn IsDisplayNone(&self) -> bool;
    fn HasLayoutObject(&self) -> bool;
    fn HasOwnerSelectElement(&self) -> bool;
    fn ListIndex(&self) -> i32;
    fn Index(&self) -> i32;
    fn UpdateMutationObserver(&self, in_style_recalc: bool);
    fn SetMultiSelectFocusedState(&self, value: bool);
}
pub fn SameOption(a: Option<&OptionRef>, b: Option<&OptionRef>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
// Borrowed indexed access is the retained DOM collection itself, including
// non-option list entries. No Vec of DOM nodes crosses this boundary.
pub trait SelectFormDataOperations {
    type Option;
    fn NameForFormData(&self) -> Rc<[u16]>;
    fn OptionLen(&self) -> usize;
    fn Option(&self, index: usize) -> Self::Option;
    fn Selected(&self, option: &Self::Option) -> bool;
    fn IsDisabled(&self, option: &Self::Option) -> bool;
    fn ValueForFormData(&self, option: &Self::Option) -> Rc<[u16]>;
}
struct SourceSelectFormData<'a>(&'a dyn SelectElement);
impl SelectFormDataOperations for SourceSelectFormData<'_> {
    type Option = OptionRef;
    fn NameForFormData(&self) -> Rc<[u16]> {
        self.0.NameForFormData()
    }
    fn OptionLen(&self) -> usize {
        self.0.OptionLen()
    }
    fn Option(&self, index: usize) -> OptionRef {
        self.0.Option(index)
    }
    fn Selected(&self, option: &OptionRef) -> bool {
        option.Selected()
    }
    fn IsDisabled(&self, option: &OptionRef) -> bool {
        option.IsDisabledFormControl()
    }
    fn ValueForFormData(&self, option: &OptionRef) -> Rc<[u16]> {
        option.ValueForFormData()
    }
}
pub trait SelectOptionCollection {
    fn ListLen(&self) -> usize;
    fn ListOption(&self, index: usize) -> Option<OptionRef>;
    fn OptionLen(&self) -> usize;
    fn Option(&self, index: usize) -> OptionRef;
}
pub trait SelectElement: SelectOptionCollection {
    fn NameForFormData(&self) -> Rc<[u16]>;
    fn IsFocused(&self) -> bool;
    fn FocusFromUserGesture(&self);
    fn DispatchAccessKeySimulatedClick(
        &self,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    );
    fn OwnsOption(&self, option: &OptionRef) -> bool;
    fn AccessKeyEventQueueScope(
        &self,
    ) -> interaction_dispatch::core::dom::events::scoped_event_queue::EventQueueScope;
    fn AccessKeySelectBehavior(&self) -> Rc<dyn SelectBehavior>;
    fn IsMultiple(&self) -> bool;
    fn IsRequired(&self) -> bool;
    fn Size(&self) -> u32;
    fn BaseResetImpl(&self);
    fn UsesMenuList(&self) -> bool;
    fn SupportsBaseAppearance(&self, appearance: foundation::AppearanceValue) -> bool;
    fn SelectedOption(&self) -> Option<OptionRef>;
    fn LastSelectedOption(&self) -> Option<OptionRef>;
    fn LastOnChangeOption(&self) -> Option<OptionRef>;
    fn SetLastOnChangeOption(&self, option: Option<OptionRef>);
    fn SetAutofillState(&self, state: u8);
    fn SelectedcontentSpecEnabled(&self) -> bool;
    fn SelectedcontentMultipleEnabled(&self) -> bool;
    fn UpdateAllSelectedcontentsMultiple(&self);
    fn UpdateAllSelectedcontentsSingle(&self, option: Option<&OptionRef>);
    fn NotifyFormStateChanged(&self);
    fn DocumentIsActive(&self) -> bool;
    fn DidChangeSelectionInSelectControl(&self);
    fn SetNeedsValidityCheck(&self);
    fn DispatchInputEvent(&self);
    fn DispatchChangeEvent(&self);
    fn HasLayoutObject(&self) -> bool;
    fn IsDisabledFormControl(&self) -> bool;
    fn SetUserHasEditedTheField(&self);
    fn TypeAheadShouldHandle(&self, event: &SelectEvent) -> bool;
    fn TypeAheadFind(&self, event: &SelectEvent);
    fn TypeAheadResetSession(&self);
    fn BaseDefaultEventHandler(&self, event: &EventRef);
    fn DocumentMatchesFrameDocument(&self) -> bool;
    fn IndexToSelectOnCancel(&self) -> i32;
    fn SetIndexToSelectOnCancel(&self, index: i32);
}
// Source: html_select_element.h:493-504.
pub const kDeselectOtherOptionsFlag: u32 = 1;
pub const kDispatchInputAndChangeEventFlag: u32 = 2;
pub const kMakeOptionDirtyFlag: u32 = 4;
pub const kDontUpdateSelectedcontentFlag: u32 = 8;
pub const USER_SELECTION_FLAGS: u32 = 1 | 2 | 4;

// Source: html_select_element.h ResetReason; cc:778-821.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResetReason {
    Reset,
    SelectedOptionRemoved,
    OptionInsertedOrRemoved,
}

pub struct HTMLSelectElement {
    pub owner: Rc<dyn SelectElement>,
}
impl HTMLSelectElement {
    // html_select_element.cc:505-507,664-666.
    pub fn CanSelectAll(&self) -> bool {
        !self.owner.UsesMenuList()
    }
    pub fn SelectAll(&self) {
        self.owner.AccessKeySelectBehavior().SelectAll();
    }

    // html_select_element.cc:1189-1289. Both native and document adapters
    // borrow the same retained list and option selectedness owners.
    pub fn SaveFormControlState(&self) -> Vec<Rc<[u16]>> {
        let behavior = self.owner.AccessKeySelectBehavior();
        Self::SaveFormControlStateWithOperations(&NativeSelectOptionOperations {
            select: self,
            behavior: behavior.as_ref(),
        })
    }
    pub fn RestoreFormControlState(&self, state: &[Rc<[u16]>]) {
        let behavior = self.owner.AccessKeySelectBehavior();
        Self::RestoreFormControlStateWithOperations(
            &NativeSelectOptionOperations {
                select: self,
                behavior: behavior.as_ref(),
            },
            state,
        );
    }
    /// html_select_element.cc:1328-1338. Iterate the canonical option list,
    /// using each option's own disabledness rather than select list indexes.
    pub fn AppendToFormData(&self, data: &mut dyn super::input_type::InputTypeFormData) {
        Self::AppendWithOperations(&SourceSelectFormData(self.owner.as_ref()), data);
    }
    pub fn AppendWithOperations<O: SelectFormDataOperations>(
        owner: &O,
        data: &mut dyn super::input_type::InputTypeFormData,
    ) {
        let name = owner.NameForFormData();
        if name.is_empty() {
            return;
        }
        for index in 0..owner.OptionLen() {
            let option = owner.Option(index);
            if owner.Selected(&option) && !owner.IsDisabled(&option) {
                data.AppendFromElement(name.clone(), owner.ValueForFormData(&option));
            }
        }
    }
    // Source: html_select_element.cc:551-555.
    pub fn AccessKeyAction(
        &self,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    ) {
        self.owner.FocusFromUserGesture();
        self.owner.DispatchAccessKeySimulatedClick(scope);
    }
    // Source: html_select_element.cc:1512-1541. Re-read the SelectType after
    // focus/simulated-click reentry, exactly at each original virtual call.
    pub fn SelectOptionByAccessKey(&self, option: Option<OptionRef>) {
        use interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope;
        if !self.owner.IsFocused() {
            self.AccessKeyAction(SimulatedClickCreationScope::kFromUserAgent);
        }
        let Some(option) = option.filter(|option| self.owner.OwnsOption(option)) else {
            return;
        };
        let _scope = self.owner.AccessKeyEventQueueScope();
        let flags = kDispatchInputAndChangeEventFlag
            | if self.owner.IsMultiple() {
                0
            } else {
                kDeselectOtherOptionsFlag
            };
        if option.Selected() {
            if self.owner.UsesMenuList() {
                self.SelectOption(
                    None,
                    flags,
                    0,
                    self.owner.AccessKeySelectBehavior().as_ref(),
                );
            } else {
                option.SetSelectedState(false, false);
            }
        } else {
            self.SelectOption(
                Some(option.clone()),
                flags,
                0,
                self.owner.AccessKeySelectBehavior().as_ref(),
            );
        }
        option.SetDirty(true);
        self.owner
            .AccessKeySelectBehavior()
            .SetListBoxActiveSelection(Some(option));
        self.owner.AccessKeySelectBehavior().ListBoxOnChange();
        self.owner.AccessKeySelectBehavior().ScrollToSelection();
    }

    // Source html_select_element.cc:179-209. The transient HashSet contains
    // canonical option identities, exactly as the source old_selection set.
    pub fn SelectMultipleOptions(&self, indices: &[i32], behavior: &dyn SelectBehavior) {
        assert!(self.owner.IsMultiple());
        let identity = |option: &OptionRef| Rc::as_ptr(option) as *const () as usize;
        let mut old = std::collections::HashSet::new();
        for index in 0..self.owner.OptionLen() {
            let option = self.owner.Option(index);
            if option.Selected() {
                old.insert(identity(&option));
                option.SetSelectedState(false, false);
            }
        }
        let mut has_new_selection = false;
        for &index in indices {
            if let Some(option) = usize::try_from(index)
                .ok()
                .and_then(|index| self.owner.ListOption(index))
            {
                option.SetSelectedState(true, false);
                option.SetDirty(true);
                if !old.remove(&identity(&option)) {
                    has_new_selection = true;
                }
            }
        }
        behavior.UpdateTextStyleAndContent();
        self.owner.SetNeedsValidityCheck();
        if has_new_selection || !old.is_empty() {
            self.owner.DispatchInputEvent();
            self.owner.DispatchChangeEvent();
        }
    }
    // Source html_select_element.cc:108-111.
    /// html_select_element.cc:1553-1559. Base parser state and the multiple
    /// selectedcontent update have already committed in the DOM owner before
    /// its synchronous lifecycle notification reaches this retained wrapper.
    pub fn FinishParsingChildrenAfterDOM(&self, environment: &dyn SelectEnvironment) {
        if self.owner.UsesMenuList() {
            return;
        }
        environment.ScrollToOption(self.owner.SelectedOption().as_ref());
        if environment.HasAXObjectCache() {
            environment.ListboxActiveIndexChanged();
        }
    }
    pub fn FormControlType(
        &self,
    ) -> crate::public::mojom::forms::form_control_type::FormControlType {
        use crate::public::mojom::forms::form_control_type::FormControlType as T;
        if self.owner.IsMultiple() {
            T::kSelectMultiple
        } else {
            T::kSelectOne
        }
    }
    // Source: html_select_element.cc:1576-1580.
    pub fn NativeArrowKeyAxes(&self) -> foundation::focusgroup_flags::FocusgroupFlags {
        NativeArrowKeyAxes()
    }
    // Element cc:14137-14164; HTMLSelectElement cc:2324-2337.
    pub fn IsAppearanceBase(&self, appearance: Option<foundation::AppearanceValue>) -> bool {
        appearance.is_some_and(|appearance| self.owner.SupportsBaseAppearance(appearance))
    }
    // Source html_select_element.cc:1860-1924.
    pub fn IsValidBuiltinCommand(
        &self,
        invoker: &EventTargetRef,
        command: CommandEventType,
        environment: &dyn SelectCommandEnvironment,
    ) -> bool {
        let parent = environment.BaseIsValidBuiltinCommand(invoker, command);
        if !environment.HTMLCommandActionsV2Enabled() {
            return parent;
        }
        parent || command == CommandEventType::kShowPicker
    }
    pub fn HandleCommandInternal(
        &self,
        invoker: &EventTargetRef,
        command: CommandEventType,
        environment: &dyn SelectCommandEnvironment,
        behavior: &dyn SelectBehavior,
    ) -> bool {
        assert!(self.IsValidBuiltinCommand(invoker, command, environment));
        if environment.BaseHandleCommandInternal(invoker, command) {
            return true;
        }
        if command != CommandEventType::kShowPicker || self.owner.IsDisabledFormControl() {
            return false;
        }
        if environment.FrameIsSameOrigin() == Some(false) {
            environment.AddCommandWarning("Select cannot be invoked from cross-origin iframe.");
            return false;
        }
        if !environment.HasTransientUserActivation() {
            environment.AddCommandWarning("Select cannot be invoked without a user gesture.");
            return false;
        }
        environment.UpdateStyleAndLayout();
        if environment.LockedAncestorPreventingPaint() || !environment.HasLayoutBox() {
            environment.AddCommandWarning("Select cannot be invoked when not being rendered.");
            return false;
        }
        behavior.ShowPicker();
        true
    }
    pub fn new(owner: Rc<dyn SelectElement>) -> Self {
        Self { owner }
    }
    // Source: html_select_element.cc:1371-1377. GetLayoutBox stays with the
    // resident Layout owner; this result controls whether that box is exposed.
    pub fn HasAutoscrollBox(&self) -> bool {
        !self.owner.UsesMenuList()
    }
    // Source html_select_element.cc:123-148,161-171,211-217.
    pub fn HasPlaceholderLabelOption(&self) -> bool {
        !self.owner.IsMultiple()
            && self.owner.Size() <= 1
            && self
                .owner
                .ListOption(0)
                .is_some_and(|option| option.ValueIsEmpty())
    }
    pub fn ValueMissing(&self) -> bool {
        Self::ValueMissingWithState(
            self.owner.IsRequired(),
            self.owner
                .SelectedOption()
                .as_ref()
                .map(|option| option.Index()),
            self.HasPlaceholderLabelOption(),
        )
    }
    pub fn ValueMissingWithState(
        required: bool,
        selected_index: Option<i32>,
        placeholder: bool,
    ) -> bool {
        required && selected_index.is_none_or(|index| index == 0 && placeholder)
    }
    pub fn ListBoxSize(&self) -> u32 {
        debug_assert!(!self.owner.UsesMenuList());
        let size = self.owner.Size();
        if size >= 1 {
            size
        } else {
            4
        }
    }

    pub fn StopAutoscroll(&self, behavior: &dyn SelectBehavior) {
        if !self.owner.IsDisabledFormControl() {
            behavior.HandleMouseRelease();
        }
    }
    // Source: html_select_element.cc:778-821.
    pub fn ResetToDefaultSelection(&self, reason: ResetReason, behavior: &dyn SelectBehavior) {
        Self::ResetToDefaultSelectionWithOperations(
            &NativeSelectOptionOperations {
                select: self,
                behavior,
            },
            reason,
        );
    }
    // Source: html_select_element.cc:1340-1348.
    pub fn ResetImpl(&self, behavior: &dyn SelectBehavior) {
        Self::ResetWithOperations(&NativeSelectOptionOperations {
            select: self,
            behavior,
        });
    }

    // Source: html_select_element.cc:1050-1111.
    pub fn SelectOption(
        &self,
        option: Option<OptionRef>,
        flags: u32,
        autofill_state: u8,
        behavior: &dyn SelectBehavior,
    ) {
        Self::SelectOptionWithOperations(
            &NativeSelectOptionOperations {
                select: self,
                behavior,
            },
            option,
            flags,
            autofill_state,
        );
    }
    // Source: html_select_element.cc:1113-1141.
    pub fn SelectOptionFromPopoverPickerOrListbox(
        &self,
        option: OptionRef,
        behavior: &dyn SelectBehavior,
    ) {
        if !self.owner.UsesMenuList() || self.owner.IsMultiple() {
            option.SetSelectedState(!option.Selected(), false);
            option.SetDirty(true);
            if !self.owner.IsMultiple() {
                for i in 0..self.owner.OptionLen() {
                    let other = self.owner.Option(i);
                    if !Rc::ptr_eq(&option, &other) {
                        other.SetSelectedState(false, false);
                    }
                }
            }
            self.owner.SetNeedsValidityCheck();
            self.owner.DispatchInputEvent();
            self.owner.DispatchChangeEvent();
            if !self.owner.IsMultiple() {
                self.owner
                    .UpdateAllSelectedcontentsSingle(if option.Selected() {
                        Some(&option)
                    } else {
                        None
                    });
            } else if self.owner.SelectedcontentMultipleEnabled() {
                self.owner.UpdateAllSelectedcontentsMultiple();
            }
            behavior.UpdateTextStyleAndContent();
        } else {
            self.SelectOptionByPopup(Some(option), behavior);
            behavior.HidePopup();
        }
    }
    // Source: html_select_element.cc:1143-1163.
    pub fn PrepareFocusEvent(&self, behavior: &dyn SelectBehavior) {
        if self.owner.UsesMenuList() {
            behavior.SaveLastSelection();
        }
    }
    pub fn PrepareBlurEvent(&self, behavior: &dyn SelectBehavior) {
        self.owner.TypeAheadResetSession();
        behavior.DidBlur();
    }
    // Source: html_select_element.cc:1165-1187.
    pub fn DeselectItemsWithoutValidation(&self, exclude: Option<&OptionRef>) -> bool {
        Self::DeselectItemsWithoutValidationWithOperations(self, exclude)
    }
    // Source: html_select_element.cc:1380-1416.
    pub fn DefaultEventHandler(
        &self,
        event: &EventRef,
        input: &SelectEvent,
        behavior: &dyn SelectBehavior,
    ) {
        if !self.owner.HasLayoutObject() {
            return;
        }
        let repeat = input.keyboard && input.repeat;
        if matches!(input.kind.as_str(), "click" | "change") || input.kind == "keydown" && !repeat {
            self.owner.SetUserHasEditedTheField();
        }
        if self.owner.IsDisabledFormControl() {
            self.owner.BaseDefaultEventHandler(event);
            return;
        }
        if !repeat {
            if behavior.DefaultEventHandler(event, input) {
                event.borrow_mut().BaseEventMut().SetDefaultHandled();
                return;
            }
            if input.keyboard && TypeAhead::ShouldHandleKeyboardEvent(input) {
                self.owner.TypeAheadFind(input);
                event.borrow_mut().BaseEventMut().SetDefaultHandled();
                return;
            }
        }
        self.owner.BaseDefaultEventHandler(event);
    }
    // Source: html_select_element.cc:1683-1689,1697-1737.
    pub fn PopupDidHide(&self, behavior: &dyn SelectBehavior) {
        behavior.PopupDidHide();
    }
    pub fn SetIndexToSelectOnCancel(&self, index: i32, behavior: &dyn SelectBehavior) {
        self.owner.SetIndexToSelectOnCancel(index);
        behavior.UpdateTextStyleAndContent();
    }
    pub fn SelectOptionByPopup(&self, option: Option<OptionRef>, behavior: &dyn SelectBehavior) {
        debug_assert!(self.owner.UsesMenuList());
        if !self.owner.DocumentMatchesFrameDocument() {
            return;
        }
        self.SetIndexToSelectOnCancel(-1, behavior);
        if SameOption(option.as_ref(), self.owner.SelectedOption().as_ref()) {
            return;
        }
        self.SelectOption(option, USER_SELECTION_FLAGS, 0, behavior);
    }
    pub fn SelectOptionByPopupIndex(&self, index: i32, behavior: &dyn SelectBehavior) {
        let option = if index < 0 {
            None
        } else {
            self.owner.ListOption(index as usize)
        };
        self.SelectOptionByPopup(option, behavior);
    }
    pub fn PopupDidCancel(&self, behavior: &dyn SelectBehavior) {
        if self.owner.IndexToSelectOnCancel() >= 0 {
            self.SelectOptionByPopupIndex(self.owner.IndexToSelectOnCancel(), behavior);
        }
    }
    pub fn ProvisionalSelectionChanged(&self, index: u32, behavior: &dyn SelectBehavior) {
        self.SetIndexToSelectOnCancel(index as i32, behavior);
    }
    pub fn ShowPopup(&self, behavior: &dyn SelectBehavior) {
        behavior.ShowPopup(false);
    }
    pub fn HidePopup(&self, behavior: &dyn SelectBehavior) {
        behavior.HidePopup();
    }
}

// Source: html_select_element.cc:1576-1580.
pub fn NativeArrowKeyAxes() -> foundation::focusgroup_flags::FocusgroupFlags {
    use foundation::focusgroup_flags::FocusgroupFlags as F;
    F::kInline | F::kBlock
}

/// Borrowed HTMLSelectElement control facts. Persistent keyboard selection
/// fields remain in ListBoxSelectionState, and last-on-change identity remains
/// on the canonical DOM select owner.
pub trait SelectElementKeyboardOperations<O: Clone>:
    super::select_type::SelectKeyboardOperations<O> + SelectResetOperations<O>
{
    fn HasLayoutObject(&self) -> bool;
    fn IsDisabledFormControl(&self) -> bool;
    fn IsAppearanceBase(&self) -> bool;
    fn SetUserHasEditedTheField(&self);
}
impl HTMLSelectElement {
    // Source html_select_element.cc:1380-1416; select_type.cc:358-437,
    // 1444-1596. Only the reachable keyboard/default-action ranges use this
    // borrowed entry; popup host lifecycle remains a separate capability.
    pub fn HandleKeyboardWithOperations<O: Clone>(
        owner: &impl SelectTypeAheadOperations<O>,
        state: &super::select_type::ListBoxSelectionState<O>,
        type_ahead: &TypeAhead,
        input: &SelectEvent,
    ) -> bool {
        if !owner.HasLayoutObject() {
            return false;
        }
        if input.kind == "keydown" && !input.repeat {
            owner.SetUserHasEditedTheField();
        }
        if owner.IsDisabledFormControl() || input.repeat {
            return false;
        }
        let handled = if owner.IsAppearanceBase() {
            false
        } else if owner.UsesMenuList() {
            if input.kind == "keydown" && input.keyboard && input.modifiers & 15 == 0 {
                super::select_type::HandleMenuListKeydownWithOperations(owner, input)
            } else if input.kind == "keypress" && input.keyboard && input.key_code == 13 {
                owner.SubmitImplicitly();
                Self::DispatchEventsIfSelectedOptionChangedWithOperations(owner);
                true
            } else {
                false
            }
        } else {
            state.HandleKeyboard(owner, input)
        };
        if handled {
            return true;
        }
        if input.keyboard && TypeAhead::ShouldHandleKeyboardEvent(input) {
            Self::TypeAheadFindWithOperations(owner, type_ahead, input);
            return true;
        }
        false
    }
    // Source html_select_element.cc:1050-1111; select_type.cc:922-961.
    // This is the source menu-list selection transaction, including pointer
    // identity comparison before input/change dispatch (equal values are not
    // equal options).
    pub fn SelectMenuOptionWithOperations<O: Clone>(
        owner: &impl SelectElementKeyboardOperations<O>,
        option: O,
    ) {
        Self::SelectOptionWithOperations(owner, Some(option), USER_SELECTION_FLAGS, 0);
    }
    pub fn DispatchEventsIfSelectedOptionChangedWithOperations<O: Clone>(
        owner: &impl SelectElementKeyboardOperations<O>,
    ) {
        let selected = owner.SelectedOption();
        if !owner.Same(owner.LastOnChangeOption().as_ref(), selected.as_ref()) {
            owner.SetLastOnChangeOption(selected.as_ref());
            owner.DispatchInputEvent();
            owner.DispatchChangeEvent();
        }
    }
}

/// Source selection transaction's live state/effect leaves. The native
/// retained owner and the borrowed DOM adapter use this same implementation.
pub trait SelectOptionOperations<O: Clone>:
    super::select_type::SelectOptionStateOperations<O>
{
    fn SetSelectionAutofillState(&self, state: u8);
    fn SelectedcontentSpecEnabled(&self) -> bool;
    fn UpdateSelectedcontentsSingle(&self, option: Option<&O>);
    fn UpdateSelectedcontentsMultiple(&self);
    fn DidSelectOption(&self, option: Option<O>, flags: u32, update_popup: bool);
    fn UpdateOptionMutationObserver(&self, option: &O);
    fn DocumentIsActive(&self) -> bool;
    fn DidChangeSelectionInSelectControl(&self);
}
pub trait SelectResetOperations<O: Clone>: SelectOptionOperations<O> {
    fn DefaultSelected(&self, option: &O) -> bool;
    fn SetDirtyState(&self, option: &O, dirty: bool);
    fn Size(&self) -> u32;
    fn SetLastOnChangeOption(&self, option: Option<&O>);
    fn UpdateTextStyleAndContent(&self);
    fn SetResetNeedsValidityCheck(&self);
    fn BaseResetImpl(&self);
}
pub trait SelectFormStateOperations<O: Clone>: SelectResetOperations<O> {
    fn ValueForFormState(&self, option: &O) -> Rc<[u16]>;
}
impl HTMLSelectElement {
    pub fn SaveFormControlStateWithOperations<O: Clone>(
        owner: &impl SelectFormStateOperations<O>,
    ) -> Vec<Rc<[u16]>> {
        let mut state = Vec::new();
        for index in 0..owner.ListLen() {
            let Some(option) = owner
                .ListOption(index)
                .filter(|option| owner.Selected(option))
            else {
                continue;
            };
            state.push(owner.ValueForFormState(&option));
            state.push(index.to_string().encode_utf16().collect::<Vec<_>>().into());
            if !owner.IsMultiple() {
                break;
            }
        }
        state
    }
    pub fn RestoreFormControlStateWithOperations<O: Clone>(
        owner: &impl SelectFormStateOperations<O>,
        state: &[Rc<[u16]>],
    ) {
        use crate::platform::wtf::text::string_to_number::StringToUintLoose;
        let size = owner.ListLen();
        if size == 0 || state.len() < 2 || state.len() % 2 != 0 {
            return;
        }
        Self::SelectOptionWithOperations(owner, None, kDeselectOtherOptionsFlag, 0);
        let search = |value: &[u16], begin: usize, end: usize| -> Option<usize> {
            (begin..end.min(owner.ListLen())).find(|&index| {
                owner
                    .ListOption(index)
                    .is_some_and(|option| owner.ValueForFormState(&option).as_ref() == value)
            })
        };
        if !owner.IsMultiple() {
            let index = StringToUintLoose(&state[1]).unwrap_or(0) as usize;
            let option = (index < size)
                .then(|| owner.ListOption(index))
                .flatten()
                .filter(|option| owner.ValueForFormState(option) == state[0])
                .or_else(|| search(&state[0], 0, size).and_then(|index| owner.ListOption(index)));
            if let Some(option) = option {
                owner.SetSelected(&option, true, false);
                owner.SetDirty(&option);
                owner.SetLastOnChangeOption(Some(&option));
            } else {
                Self::ResetToDefaultSelectionWithOperations(owner, ResetReason::Reset);
            }
            owner.UpdateSelectedcontentsSingle(owner.LastOnChangeOption().as_ref());
        } else {
            let mut start = 0;
            for values in state.chunks_exact(2) {
                let index = StringToUintLoose(&values[1]).unwrap_or(0) as usize;
                let found = if index < size
                    && owner
                        .ListOption(index)
                        .is_some_and(|option| owner.ValueForFormState(&option) == values[0])
                {
                    Some(index)
                } else {
                    search(&values[0], start, size).or_else(|| search(&values[0], 0, start))
                };
                if let Some(index) = found {
                    let option = owner.ListOption(index).expect("source matched option");
                    owner.SetSelected(&option, true, false);
                    owner.SetDirty(&option);
                    start = index + 1;
                }
            }
            owner.UpdateSelectedcontentsMultiple();
        }
        owner.SetResetNeedsValidityCheck();
        owner.UpdateTextStyleAndContent();
    }
}
impl HTMLSelectElement {
    // Source html_select_element.cc:1050-1111.
    pub fn SelectOptionWithOperations<O: Clone>(
        owner: &impl SelectOptionOperations<O>,
        option: Option<O>,
        flags: u32,
        autofill_state: u8,
    ) {
        let mut update_popup = false;
        owner.SetSelectionAutofillState(if option.is_some() { autofill_state } else { 0 });
        if let Some(option) = &option {
            update_popup = !owner.Selected(option);
            owner.SetSelected(option, true, true);
            if flags & kMakeOptionDirtyFlag != 0 {
                owner.SetDirty(option);
            }
        }
        if flags & kDeselectOtherOptionsFlag != 0 {
            update_popup |=
                Self::DeselectItemsWithoutValidationWithOperations(owner, option.as_ref());
        }
        if !owner.SelectedcontentSpecEnabled() || flags & kDontUpdateSelectedcontentFlag == 0 {
            if owner.IsMultiple() {
                owner.UpdateSelectedcontentsMultiple();
            } else {
                owner.UpdateSelectedcontentsSingle(option.as_ref());
            }
        }
        owner.DidSelectOption(option.clone(), flags, update_popup);
        if let Some(option) = &option {
            owner.UpdateOptionMutationObserver(option);
        }
        owner.NotifyFormStateChanged();
        if owner.DocumentIsActive() {
            owner.DidChangeSelectionInSelectControl();
        }
        owner.SetSelectionAutofillState(if option.is_some() { autofill_state } else { 0 });
    }
    // Source html_select_element.h:508-509; cc:1165-1187.
    // LastOnChangeOption is the canonical select-owned identity. Preserve its
    // single-menu fast path and the fallback's owner check and update order.
    pub fn DeselectItemsWithoutValidationWithOperations<O: Clone>(
        owner: &impl super::select_type::SelectOptionStateOperations<O>,
        exclude: Option<&O>,
    ) -> bool {
        if !owner.IsMultiple() && owner.UsesMenuList() {
            if let Some(last) = owner.LastOnChangeOption() {
                if !owner.Same(Some(&last), exclude) {
                    owner.SetSelected(&last, false, false);
                    return true;
                }
            }
        }
        let mut changed = false;
        for i in 0..owner.OptionLen() {
            let option = owner.Option(i);
            if owner.Same(Some(&option), exclude) || !owner.OptionHasOwnerSelect(&option) {
                continue;
            }
            changed |= owner.Selected(&option);
            owner.SetSelected(&option, false, false);
        }
        changed
    }
    // Source html_select_element.cc:778-821.
    pub fn ResetToDefaultSelectionWithOperations<O: Clone>(
        owner: &impl SelectResetOperations<O>,
        reason: ResetReason,
    ) {
        if owner.IsMultiple() {
            return;
        }
        let mut first_enabled = None;
        let mut last_selected = None;
        let mut changed = false;
        for i in 0..owner.OptionLen() {
            let option = owner.Option(i);
            if owner.Selected(&option) {
                if let Some(previous) = &last_selected {
                    owner.SetSelected(previous, false, false);
                    changed = true;
                }
                last_selected = Some(option.clone());
            }
            if first_enabled.is_none() && !owner.Disabled(&option) {
                first_enabled = Some(option);
                if reason == ResetReason::SelectedOptionRemoved {
                    break;
                }
            }
        }
        if last_selected.is_none()
            && owner.Size() <= 1
            && first_enabled.as_ref().is_none_or(|o| !owner.Selected(o))
        {
            let flags = match reason {
                ResetReason::SelectedOptionRemoved => kDontUpdateSelectedcontentFlag,
                ResetReason::OptionInsertedOrRemoved => {
                    kDeselectOtherOptionsFlag | kDontUpdateSelectedcontentFlag
                }
                ResetReason::Reset => kDeselectOtherOptionsFlag,
            };
            Self::SelectOptionWithOperations(owner, first_enabled.clone(), flags, 0);
            last_selected = first_enabled;
            changed = true;
        }
        if changed {
            owner.SetResetNeedsValidityCheck();
        }
        owner.SetLastOnChangeOption(last_selected.as_ref());
    }
    // Source html_select_element.cc:1340-1348.
    pub fn ResetWithOperations<O: Clone>(owner: &impl SelectResetOperations<O>) {
        for i in 0..owner.OptionLen() {
            let option = owner.Option(i);
            owner.SetSelected(&option, owner.DefaultSelected(&option), false);
            owner.SetDirtyState(&option, false);
        }
        Self::ResetToDefaultSelectionWithOperations(owner, ResetReason::Reset);
        owner.UpdateTextStyleAndContent();
        owner.SetResetNeedsValidityCheck();
        owner.BaseResetImpl();
    }
}
struct NativeSelectOptionOperations<'a> {
    select: &'a HTMLSelectElement,
    behavior: &'a dyn SelectBehavior,
}
// A package-private native port borrows the existing HTMLSelectElement;
// it never constructs an alternate select, collection, or selectedness owner.
pub(super) trait NativeSelectElementPort {
    fn SourceSelectElement(&self) -> &HTMLSelectElement;
}
impl NativeSelectElementPort for HTMLSelectElement {
    fn SourceSelectElement(&self) -> &HTMLSelectElement {
        self
    }
}
impl NativeSelectElementPort for NativeSelectOptionOperations<'_> {
    fn SourceSelectElement(&self) -> &HTMLSelectElement {
        self.select
    }
}
impl<T: NativeSelectElementPort> super::select_type::SelectOptionStateOperations<OptionRef> for T {
    fn Same(&self, a: Option<&OptionRef>, b: Option<&OptionRef>) -> bool {
        SameOption(a, b)
    }
    fn ListLen(&self) -> usize {
        self.SourceSelectElement().owner.ListLen()
    }
    fn ListOption(&self, i: usize) -> Option<OptionRef> {
        self.SourceSelectElement().owner.ListOption(i)
    }
    fn OptionLen(&self) -> usize {
        self.SourceSelectElement().owner.OptionLen()
    }
    fn Option(&self, i: usize) -> OptionRef {
        self.SourceSelectElement().owner.Option(i)
    }
    fn ListIndex(&self, o: &OptionRef) -> i32 {
        o.ListIndex()
    }
    fn Index(&self, o: &OptionRef) -> i32 {
        o.Index()
    }
    fn Selected(&self, o: &OptionRef) -> bool {
        o.Selected()
    }
    fn Disabled(&self, o: &OptionRef) -> bool {
        o.IsDisabledFormControl()
    }
    fn DisplayNone(&self, o: &OptionRef) -> bool {
        o.IsDisplayNone()
    }
    fn OptionHasLayout(&self, o: &OptionRef) -> bool {
        o.HasLayoutObject()
    }
    fn SetSelected(&self, o: &OptionRef, v: bool, skip: bool) {
        o.SetSelectedState(v, skip);
    }
    fn SetDirty(&self, o: &OptionRef) {
        o.SetDirty(true);
    }
    fn IsMultiple(&self) -> bool {
        self.SourceSelectElement().owner.IsMultiple()
    }
    fn NotifyFormStateChanged(&self) {
        self.SourceSelectElement().owner.NotifyFormStateChanged();
    }

    fn UsesMenuList(&self) -> bool {
        self.SourceSelectElement().owner.UsesMenuList()
    }
    fn LastOnChangeOption(&self) -> Option<OptionRef> {
        self.SourceSelectElement().owner.LastOnChangeOption()
    }
    fn OptionHasOwnerSelect(&self, option: &OptionRef) -> bool {
        option.HasOwnerSelectElement()
    }
    fn SelectedOption(&self) -> Option<OptionRef> {
        self.SourceSelectElement().owner.SelectedOption()
    }
    fn LastSelectedOption(&self) -> Option<OptionRef> {
        self.SourceSelectElement().owner.LastSelectedOption()
    }
}
impl SelectOptionOperations<OptionRef> for NativeSelectOptionOperations<'_> {
    fn SetSelectionAutofillState(&self, state: u8) {
        self.select.owner.SetAutofillState(state);
    }
    fn SelectedcontentSpecEnabled(&self) -> bool {
        self.select.owner.SelectedcontentSpecEnabled()
    }
    fn UpdateSelectedcontentsSingle(&self, o: Option<&OptionRef>) {
        self.select.owner.UpdateAllSelectedcontentsSingle(o);
    }
    fn UpdateSelectedcontentsMultiple(&self) {
        self.select.owner.UpdateAllSelectedcontentsMultiple();
    }
    fn DidSelectOption(&self, o: Option<OptionRef>, flags: u32, update: bool) {
        self.behavior.DidSelectOption(o, flags, update);
    }
    fn UpdateOptionMutationObserver(&self, o: &OptionRef) {
        o.UpdateMutationObserver(false);
    }
    fn DocumentIsActive(&self) -> bool {
        self.select.owner.DocumentIsActive()
    }
    fn DidChangeSelectionInSelectControl(&self) {
        self.select.owner.DidChangeSelectionInSelectControl();
    }
}
impl SelectResetOperations<OptionRef> for NativeSelectOptionOperations<'_> {
    fn DefaultSelected(&self, o: &OptionRef) -> bool {
        o.DefaultSelected()
    }
    fn SetDirtyState(&self, o: &OptionRef, v: bool) {
        o.SetDirty(v);
    }
    fn Size(&self) -> u32 {
        self.select.owner.Size()
    }
    fn SetLastOnChangeOption(&self, o: Option<&OptionRef>) {
        self.select.owner.SetLastOnChangeOption(o.cloned());
    }
    fn UpdateTextStyleAndContent(&self) {
        self.behavior.UpdateTextStyleAndContent();
    }
    fn SetResetNeedsValidityCheck(&self) {
        self.select.owner.SetNeedsValidityCheck();
    }
    fn BaseResetImpl(&self) {
        self.select.owner.BaseResetImpl();
    }
}

impl SelectFormStateOperations<OptionRef> for NativeSelectOptionOperations<'_> {
    fn ValueForFormState(&self, option: &OptionRef) -> Rc<[u16]> {
        option.ValueForFormData()
    }
}

// Source html_select_element.h:527-530; html_select_element.cc:1469-1510.
// The TypeAheadDataSource reads the same live list/labels as SelectType; only
// the source focus and final change callbacks are added to the transaction.
pub trait SelectTypeAheadOperations<O: Clone>:
    SelectElementKeyboardOperations<O> + TypeAheadDataSource
{
    fn TypeAheadCustomizablePopupVisible(&self) -> bool;
    fn OptionDisplayLabel(&self, option: &O) -> Rc<[u16]>;
    fn FocusOptionWithScriptTrigger(&self, option: &O);
    fn TypeAheadListBoxOnChange(&self);
}
impl HTMLSelectElement {
    // Source html_select_element.cc:1469-1483. Use list indices, including
    // optgroup/hr slots; disabled option labels deliberately do not match.
    pub fn TypeAheadSelectedListIndexWithOperations<O: Clone>(
        owner: &impl SelectTypeAheadOperations<O>,
    ) -> i32 {
        owner
            .SelectedOption()
            .as_ref()
            .map_or(-1, |option| owner.ListIndex(option))
    }
    pub fn TypeAheadOptionCountWithOperations<O: Clone>(
        owner: &impl SelectTypeAheadOperations<O>,
    ) -> i32 {
        owner.ListLen() as i32
    }
    pub fn TypeAheadOptionAtIndexWithOperations<O: Clone>(
        owner: &impl SelectTypeAheadOperations<O>,
        index: i32,
    ) -> Rc<[u16]> {
        if let Some(option) = usize::try_from(index)
            .ok()
            .and_then(|i| owner.ListOption(i))
        {
            if !owner.Disabled(&option) {
                return owner.OptionDisplayLabel(&option);
            }
        }
        Rc::from([])
    }
    // Source html_select_element.cc:1485-1510. Session matching completes
    // before selection/focus callbacks, so listener reentry borrows live owners.
    pub fn TypeAheadFindWithOperations<O: Clone>(
        owner: &impl SelectTypeAheadOperations<O>,
        type_ahead: &TypeAhead,
        event: &SelectEvent,
    ) {
        let index = type_ahead.HandleEvent(
            owner,
            event,
            event.char_code as u16,
            kMatchPrefix | kCycleFirstChar,
        );
        if index < 0 {
            return;
        }
        let option = owner
            .ListOption(index as usize)
            .expect("typeahead matched a live option");
        let customizable_popup = owner.TypeAheadCustomizablePopupVisible();
        let customizable_in_page = !owner.UsesMenuList() && owner.IsAppearanceBase();
        if customizable_popup || customizable_in_page {
            owner.FocusOptionWithScriptTrigger(&option);
            return;
        }
        Self::SelectOptionWithOperations(owner, Some(option), USER_SELECTION_FLAGS, 0);
        owner.TypeAheadListBoxOnChange();
    }
}
