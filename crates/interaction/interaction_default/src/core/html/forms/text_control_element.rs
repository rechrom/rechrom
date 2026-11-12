/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2000 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2004, 2005, 2006, 2007, 2008, 2009, 2010 Apple Inc. All rights
 * reserved.
 * Copyright (C) 2009, 2010, 2011 Google Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/text_control_element.h:49-61

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextFieldEventBehavior {
    kDispatchNoEvent,
    kDispatchChangeEvent,
    kDispatchInputEvent,
    kDispatchInputAndChangeEvent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextControlSetValueSelection {
    kSetSelectionToStart,
    kSetSelectionToEnd,
    kClamp,
    kDoNotSet,
}

// Source: text_control_element.h:44-48.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextFieldSelectionDirection {
    #[default]
    kSelectionHasNoDirection,
    kSelectionHasForwardDirection,
    kSelectionHasBackwardDirection,
}

use super::text_field_input_type::TextValue;
use interaction_dispatch::core::dom::events::event::EventRef;
use std::{cell::RefCell, rc::Rc};

// Source: text_control_element.h:88-114,121-155; .cc:356-403,641-651.
// The DOM element supplies this very same state owner to its interaction
// projection. It is not copied into a second selection/value model.
#[derive(Default)]
pub struct TextControlState {
    pub cached_selection_start: usize,
    pub cached_selection_end: usize,
    pub cached_selection_direction: TextFieldSelectionDirection,
    pub value_before_first_user_edit: Option<TextValue>,
    pub last_change_was_user_edit: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum SelectionMode {
    Select,
    Start,
    End,
    Preserve,
}

/// The existing DOM text/value and FrameSelection owner implement these
/// synchronous operations. Position identity is retained inside that owner.
pub trait TextControlValue {
    fn Value(&self) -> TextValue;
    fn InnerEditorValue(&self) -> TextValue;
    fn SetValue(
        &self,
        value: &[u16],
        behavior: TextFieldEventBehavior,
        selection: TextControlSetValueSelection,
    );
    fn OpenShadowRoot(&self) -> bool;
    fn IsTextControl(&self) -> bool;
    fn IsConnected(&self) -> bool;
    fn IsFocused(&self) -> bool;
    fn HasLayoutTextControl(&self) -> bool;
    fn HasFrame(&self) -> bool;
    fn EnsureInnerEditor(&self) -> bool;
    fn ShouldApplySelectionCache(&self) -> bool;
    fn ShouldConsiderSelectionAsDirectional(&self) -> bool;
    fn ComputeSelection(&self) -> (usize, usize, TextFieldSelectionDirection);
    fn IsSelectionRange(&self) -> bool;
    fn IsRunningExecCommand(&self) -> bool;
}

pub trait TextControlEffects {
    fn SetFrameSelectionFromControlOffsets(&self, anchor: usize, focus: usize, directional: bool);
    fn ScheduleSelectEvent(&self);
    fn ScheduleSelectionchangeEventOnThisOrDocument(&self);
    fn DispatchSelectEvent(&self);
    fn DispatchChangeEvent(&self);
    fn EnqueueChangeEvent(&self);
    fn InnerEditorFocusChanged(&self);
    fn FocusWithoutRestoringSelection(&self);
    fn OpaqueRangeEnabled(&self) -> bool;
    fn CaptureOpaqueRangePreEdit(&self);
    fn BeginSkipValueAutoDiff(&self);
    fn EndSkipValueAutoDiff(&self);
    fn ShouldSkipNextSetValueAutoDiff(&self) -> bool;
    fn CommitProgrammaticOpaqueRangeEdit(&self, old_value: &[u16], start: usize, end: usize);
    fn SetUserHasEditedTheField(&self);
    fn SubtreeHasChanged(&self);
    fn BaseDefaultEventHandler(&self, event: &EventRef);
    fn InnerEditorDefaultEventHandler(&self, event: &EventRef);
}

// Source: text_control_element.cc:161-181. Base DOM event creation/dispatch
// follows these virtual hooks on the original focus transaction.
pub trait TextControlFocusEventOperations {
    fn SupportsPlaceholder(&self) -> bool;
    fn UpdatePlaceholderVisibility(&self);
    fn HandleFocusEvent(
        &self,
        old: Option<interaction_dispatch::core::dom::events::event::EventTargetRef>,
        focus_type: foundation::FocusType,
    );
    fn HandleBlurEvent(&self);
}
pub fn PrepareTextControlFocusEvent(
    owner: &dyn TextControlFocusEventOperations,
    old: Option<interaction_dispatch::core::dom::events::event::EventTargetRef>,
    focus_type: foundation::FocusType,
) {
    if owner.SupportsPlaceholder() {
        owner.UpdatePlaceholderVisibility();
    }
    owner.HandleFocusEvent(old, focus_type);
}
pub fn PrepareTextControlBlurEvent(owner: &dyn TextControlFocusEventOperations) {
    if owner.SupportsPlaceholder() {
        owner.UpdatePlaceholderVisibility();
    }
    owner.HandleBlurEvent();
}

pub struct TextControlElement {
    pub state: Rc<RefCell<TextControlState>>,
    pub value: Rc<dyn TextControlValue>,
    pub effects: Rc<dyn TextControlEffects>,
}

impl TextControlElement {
    // Source: text_control_element.h:345-349. These are the actual inherited
    // empty hooks, distinct from HTMLInputElement's selected-view blur override.
    pub fn BaseHandleFocusEvent(
        _old: Option<interaction_dispatch::core::dom::events::event::EventTargetRef>,
        _focus_type: foundation::FocusType,
    ) {
    }
    pub fn BaseHandleBlurEvent() {}

    // Source: text_control_element.cc:183-239.
    pub fn DefaultEventHandler(&self, event: &EventRef, clamp_unfocused_cache: bool) {
        let (event_type, input_event) = {
            let event = event.borrow();
            (event.BaseEvent().r#type().to_owned(), event.IsInputEvent())
        };
        if self.effects.OpaqueRangeEnabled() && event_type == "beforeinput" && input_event {
            self.effects.CaptureOpaqueRangePreEdit();
        }
        if event_type == "webkitEditableContentChanged" && self.value.HasLayoutTextControl() {
            let user_edit = !self.value.IsRunningExecCommand();
            self.state.borrow_mut().last_change_was_user_edit = user_edit;
            if user_edit {
                self.effects.SetUserHasEditedTheField();
            }
            if self.value.IsFocused() {
                let (start, end, direction) = self.value.ComputeSelection();
                self.CacheSelection(start, end, direction);
            } else if clamp_unfocused_cache {
                let len = self.value.InnerEditorValue().len();
                let (start, end, direction) = {
                    let state = self.state.borrow();
                    (
                        state.cached_selection_start,
                        state.cached_selection_end,
                        state.cached_selection_direction,
                    )
                };
                if start > len || end > len {
                    self.CacheSelection(start.min(len), end.min(len), direction);
                }
            }
            self.effects.SubtreeHasChanged();
            return;
        }
        self.effects.BaseDefaultEventHandler(event);
    }
    pub fn ForwardEvent(&self, event: &EventRef) {
        if matches!(event.borrow().BaseEvent().r#type(), "blur" | "focus") {
            return;
        }
        self.effects.InnerEditorDefaultEventHandler(event);
    }

    // Source: text_control_element.cc:332-354.
    pub fn setSelectionStart(&self, start: usize) {
        self.setSelectionRangeForBinding(
            start,
            start.max(self.selectionEnd()),
            self.selectionDirection(),
        );
    }
    pub fn setSelectionEnd(&self, end: usize) {
        self.setSelectionRangeForBinding(
            end.min(self.selectionStart()),
            end,
            self.selectionDirection(),
        );
    }
    pub fn setSelectionDirection(&self, direction: TextFieldSelectionDirection) {
        self.setSelectionRangeForBinding(self.selectionStart(), self.selectionEnd(), direction);
    }
    pub fn select(&self) {
        self.setSelectionRangeForBinding(
            0,
            u32::MAX as usize,
            TextFieldSelectionDirection::kSelectionHasNoDirection,
        );
        self.effects.FocusWithoutRestoringSelection();
        self.RestoreCachedSelection();
    }
    // Source: text_control_element.cc:356-403.
    pub fn SetValueBeforeFirstUserEditIfNotSet(&self) {
        if self.state.borrow().value_before_first_user_edit.is_some() {
            return;
        }
        let value = self.value.Value();
        self.state.borrow_mut().value_before_first_user_edit = Some(value);
    }
    pub fn CheckIfValueWasReverted(&self, value: &[u16]) {
        let equal = self.state.borrow().value_before_first_user_edit.as_deref() == Some(value);
        if equal {
            self.ClearValueBeforeFirstUserEdit();
        }
    }
    pub fn ClearValueBeforeFirstUserEdit(&self) {
        self.state.borrow_mut().value_before_first_user_edit = None;
    }
    /// Source post-base portion of TextControlElement::SetFocused. A focus
    /// assembly that executes the translated base virtual chain separately
    /// calls this once after that base chain completes.
    pub fn DidSetFocused(&self, focused: bool) {
        if !focused {
            self.DispatchFormControlChangeEvent();
        }
        self.effects.InnerEditorFocusChanged();
    }
    pub fn DispatchFormControlChangeEvent(&self) {
        let value = self.value.Value();
        let changed = self
            .state
            .borrow()
            .value_before_first_user_edit
            .as_ref()
            .is_some_and(|old| *old != value);
        self.ClearValueBeforeFirstUserEdit();
        if changed {
            self.effects.DispatchChangeEvent();
        }
    }
    pub fn EnqueueChangeEvent(&self) {
        let value = self.value.Value();
        let changed = self
            .state
            .borrow()
            .value_before_first_user_edit
            .as_ref()
            .is_some_and(|old| *old != value);
        if changed {
            self.effects.EnqueueChangeEvent();
        }
        self.ClearValueBeforeFirstUserEdit();
    }
    // Source: text_control_element.cc:406-483. UTF-16 offsets preserve the
    // source StringView ranges, including a cut inside a surrogate pair.
    pub fn setRangeText(
        &self,
        replacement: &[u16],
        mut start: usize,
        mut end: usize,
        mode: SelectionMode,
    ) -> Result<(), &'static str> {
        if start > end {
            return Err("IndexSizeError");
        }
        if self.value.OpenShadowRoot() {
            return Ok(());
        }
        let original = self.value.InnerEditorValue();
        let mut selection_start = self.selectionStart();
        let mut selection_end = self.selectionEnd();
        start = start.min(original.len());
        end = end.min(original.len());
        let mut text = original[..start].to_vec();
        text.extend_from_slice(replacement);
        text.extend_from_slice(&original[end..]);
        self.effects.BeginSkipValueAutoDiff();
        self.value.SetValue(
            &text,
            TextFieldEventBehavior::kDispatchNoEvent,
            TextControlSetValueSelection::kDoNotSet,
        );
        if self.effects.OpaqueRangeEnabled() {
            self.effects
                .CommitProgrammaticOpaqueRangeEdit(&original, start, end);
        }
        self.effects.EndSkipValueAutoDiff();
        match mode {
            SelectionMode::Select => {
                selection_start = start;
                selection_end = start + replacement.len();
            }
            SelectionMode::Start => {
                selection_start = start;
                selection_end = start;
            }
            SelectionMode::End => {
                selection_start = start + replacement.len();
                selection_end = selection_start;
            }
            SelectionMode::Preserve => {
                let delta = replacement.len() as isize - (end - start) as isize;
                if selection_start > end {
                    selection_start = selection_start.wrapping_add_signed(delta);
                } else if selection_start > start {
                    selection_start = start;
                }
                if selection_end > end {
                    selection_end = selection_end.wrapping_add_signed(delta);
                } else if selection_end > start {
                    selection_end = start + replacement.len();
                }
            }
        }
        self.setSelectionRangeForBinding(
            selection_start,
            selection_end,
            TextFieldSelectionDirection::kSelectionHasNoDirection,
        );
        Ok(())
    }
    // Source: text_control_element.cc:485-496.
    pub fn setSelectionRangeForBinding(
        &self,
        start: usize,
        end: usize,
        direction: TextFieldSelectionDirection,
    ) {
        if self.SetSelectionRange(start, end, direction) {
            self.effects.ScheduleSelectEvent();
        }
    }
    // Source: text_control_element.cc:485-496. Script supplies the original
    // string; unrecognized strings retain the source "none" interpretation.
    pub fn setSelectionRangeForBindingString(&self, start: usize, end: usize, direction: &str) {
        let direction = match direction {
            "forward" => TextFieldSelectionDirection::kSelectionHasForwardDirection,
            "backward" => TextFieldSelectionDirection::kSelectionHasBackwardDirection,
            _ => TextFieldSelectionDirection::kSelectionHasNoDirection,
        };
        self.setSelectionRangeForBinding(start, end, direction);
    }
    // Source: text_control_element.cc:576-651.
    pub fn SetSelectionRange(
        &self,
        mut start: usize,
        mut end: usize,
        mut direction: TextFieldSelectionDirection,
    ) -> bool {
        if self.value.OpenShadowRoot() || !self.value.IsTextControl() {
            return false;
        }
        let editor = self.value.EnsureInnerEditor();
        end = end.min(self.value.InnerEditorValue().len());
        start = start.min(end);
        let frame = self.value.HasFrame();
        if direction == TextFieldSelectionDirection::kSelectionHasNoDirection
            && frame
            && self.value.ShouldConsiderSelectionAsDirectional()
        {
            direction = TextFieldSelectionDirection::kSelectionHasForwardDirection;
        }
        let changed = self.CacheSelection(start, end, direction);
        if self.value.ShouldApplySelectionCache() || !self.value.IsConnected() || !frame || !editor
        {
            if changed {
                self.effects.ScheduleSelectionchangeEventOnThisOrDocument();
            }
            return changed;
        }
        let (anchor, focus) =
            if direction == TextFieldSelectionDirection::kSelectionHasBackwardDirection {
                (end, start)
            } else {
                (start, end)
            };
        self.effects.SetFrameSelectionFromControlOffsets(
            anchor,
            focus,
            direction != TextFieldSelectionDirection::kSelectionHasNoDirection,
        );
        changed
    }
    pub fn CacheSelection(
        &self,
        start: usize,
        end: usize,
        direction: TextFieldSelectionDirection,
    ) -> bool {
        debug_assert!(start <= end);
        let mut state = self.state.borrow_mut();
        let changed = state.cached_selection_start != start
            || state.cached_selection_end != end
            || state.cached_selection_direction != direction;
        state.cached_selection_start = start;
        state.cached_selection_end = end;
        state.cached_selection_direction = direction;
        changed
    }
    // Source: text_control_element.cc:666-675,718-725,745-752. The live owner
    // computes offsets; cached state is authoritative while unfocused.
    pub fn selectionStart(&self) -> usize {
        if !self.value.IsTextControl() {
            return 0;
        }
        if self.value.ShouldApplySelectionCache() {
            self.state.borrow().cached_selection_start
        } else {
            self.value.ComputeSelection().0
        }
    }
    pub fn selectionEnd(&self) -> usize {
        if !self.value.IsTextControl() {
            return 0;
        }
        if self.value.ShouldApplySelectionCache() {
            self.state.borrow().cached_selection_end
        } else {
            self.value.ComputeSelection().1
        }
    }
    pub fn selectionDirection(&self) -> TextFieldSelectionDirection {
        if self.value.ShouldApplySelectionCache() {
            self.state.borrow().cached_selection_direction
        } else {
            self.value.ComputeSelection().2
        }
    }
    // Source: text_control_element.cc:871-899.
    pub fn RestoreCachedSelection(&self) {
        let (start, end, direction) = {
            let state = self.state.borrow();
            (
                state.cached_selection_start,
                state.cached_selection_end,
                state.cached_selection_direction,
            )
        };
        if self.SetSelectionRange(start, end, direction) {
            self.effects.ScheduleSelectEvent();
        }
    }
    pub fn SelectionChanged(&self, user_triggered: bool, preserve_unfocused_cache: bool) {
        if !self.value.HasLayoutTextControl() {
            return;
        }
        if self.value.IsFocused() || !preserve_unfocused_cache {
            let (start, end, direction) = self.value.ComputeSelection();
            self.CacheSelection(start, end, direction);
        }
        if !self.value.HasFrame() || !user_triggered || !self.value.IsSelectionRange() {
            return;
        }
        self.effects.DispatchSelectEvent();
    }
}

// Source: text_control_element.cc:728-743.
pub fn DirectionString(direction: TextFieldSelectionDirection) -> &'static str {
    match direction {
        TextFieldSelectionDirection::kSelectionHasNoDirection => "none",
        TextFieldSelectionDirection::kSelectionHasForwardDirection => "forward",
        TextFieldSelectionDirection::kSelectionHasBackwardDirection => "backward",
    }
}
