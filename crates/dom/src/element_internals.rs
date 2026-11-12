// Copyright 2018 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in third_party/LICENSE.chromium.
//! Blink core/html/custom/element_internals.cc and custom_state_set.cc.
//! The target's DOM owner retains form values, validity and custom states;
//! bindings only convert IDL values and invoke the existing reaction queue.
#![allow(non_snake_case)]
use crate::{
    custom_elements::{CustomElementReaction, CustomElementState},
    error::{DOMException, DOMExceptionKind},
    PersistentDocument,
};
use fileapi::form_data::{EntryValue, FormData};
use std::{rc::Rc, sync::Arc};

#[derive(Clone)]
pub enum ElementInternalsValue {
    String(String),
    File(Arc<fileapi::File>),
    FormData(Rc<FormData>),
}
impl std::fmt::Debug for ElementInternalsValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => f.debug_tuple("String").field(value).finish(),
            Self::File(_) => f.write_str("File"),
            Self::FormData(_) => f.write_str("FormData"),
        }
    }
}
impl PartialEq for ElementInternalsValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(left), Self::String(right)) => left == right,
            (Self::File(left), Self::File(right)) => Arc::ptr_eq(left, right),
            (Self::FormData(left), Self::FormData(right)) => Rc::ptr_eq(left, right),
            _ => false,
        }
    }
}
impl Eq for ElementInternalsValue {}
impl ElementInternalsValue {
    fn AppendToControlState(&self, state: &mut Vec<Rc<[u16]>>) {
        let text = |value: &str| -> Rc<[u16]> { value.encode_utf16().collect::<Vec<_>>().into() };
        match self {
            Self::String(value) => state.extend([text("USVString"), text(value)]),
            Self::File(file) => {
                state.push(text("File"));
                let mut values = Vec::new();
                file.AppendToControlState(&mut values);
                state.extend(values.iter().map(|value| text(value)));
            }
            Self::FormData(data) => {
                state.push(text("FormData"));
                data.AppendToControlState(state);
            }
        }
    }
    fn RestoreFromControlState(
        state: &[Rc<[u16]>],
        section: &str,
        index: &mut usize,
        reader: Option<Arc<dyn fileapi::NativeFileReader>>,
    ) -> Result<Option<Self>, navigation_history::FormStateRestoreError> {
        use navigation_history::FormStateRestoreError as E;
        if state.len() < index.saturating_add(3)
            || !state[*index].iter().copied().eq(section.encode_utf16())
        {
            return Ok(None);
        }
        let kind = state[*index + 1].clone();
        *index += 2;
        if kind.as_ref() == [85, 83, 86, 83, 116, 114, 105, 110, 103] {
            let value = Self::String(String::from_utf16_lossy(&state[*index]));
            *index += 1;
            Ok(Some(value))
        } else if kind.as_ref() == [70, 105, 108, 101] {
            let reader = reader.ok_or(E::NativeFileReaderUnavailable)?;
            fileapi::File::CreateFromControlState(state, index, reader)
                .map(|value| value.map(Self::File))
                .map_err(|error| E::NativeFileUnavailable(format!("{error:?}")))
        } else if kind.as_ref() == [70, 111, 114, 109, 68, 97, 116, 97] {
            FormData::CreateFromControlState(state, index, reader)
                .map(|value| value.map(|data| Self::FormData(Rc::new(data))))
                .map_err(|error| match error {
                    fileapi::form_data::FormDataStateRestoreError::NativeFileReaderUnavailable => {
                        E::NativeFileReaderUnavailable
                    }
                    fileapi::form_data::FormDataStateRestoreError::NativeFile(error) => {
                        E::NativeFileUnavailable(format!("{error:?}"))
                    }
                })
        } else {
            Ok(None)
        }
    }
}

#[derive(Default)]
pub(crate) struct ElementInternals {
    attached: bool,
    value: Option<ElementInternalsValue>,
    state: Option<ElementInternalsValue>,
    flags: u16,
    anchor: Option<usize>,
    anchor_revision: u64,
    states: Vec<String>,
    form: Option<usize>,
    disabled: bool,
}

impl PersistentDocument {
    pub fn SaveElementInternalsFormControlState(&self, element: usize) -> Vec<Rc<[u16]>> {
        let Some(internals) = self.custom_elements.internals.get(&element) else {
            return Vec::new();
        };
        let mut state = Vec::new();
        if let Some(value) = &internals.value {
            state.push("Value".encode_utf16().collect::<Vec<_>>().into());
            value.AppendToControlState(&mut state);
        }
        if let Some(value) = &internals.state {
            state.push("State".encode_utf16().collect::<Vec<_>>().into());
            value.AppendToControlState(&mut state);
        }
        state
    }
    /// RestoreFromFormControlState decodes immutable values before borrowing
    /// the live target; committing only enqueues a custom-element reaction.
    pub fn DecodeElementInternalsFormControlState(
        state: &[Rc<[u16]>],
        reader: Option<Arc<dyn fileapi::NativeFileReader>>,
    ) -> Result<
        (Option<ElementInternalsValue>, Option<ElementInternalsValue>),
        navigation_history::FormStateRestoreError,
    > {
        let mut index = 0;
        let value = ElementInternalsValue::RestoreFromControlState(
            state,
            "Value",
            &mut index,
            reader.clone(),
        )?;
        let state =
            ElementInternalsValue::RestoreFromControlState(state, "State", &mut index, reader)?;
        Ok((value, state))
    }
    pub fn RestoreElementInternalsFormControlState(
        &mut self,
        element: usize,
        value: Option<ElementInternalsValue>,
        state: Option<ElementInternalsValue>,
    ) {
        self.EnsureElementInternals(element);
        if let Some(value) = value {
            self.custom_elements
                .internals
                .get_mut(&element)
                .unwrap()
                .value = Some(value);
        }
        if let Some(state) = state {
            if let Some(definition) = self
                .CustomElementDefinitionForElement(element)
                .filter(|d| d.options.form_associated && d.callbacks.form_state_restore)
            {
                self.EnqueueInternalsReaction(CustomElementReaction::FormStateRestore {
                    element,
                    id: definition.id,
                    state,
                });
            }
        }
    }
    pub(crate) fn EnsureElementInternals(&mut self, element: usize) {
        self.custom_elements.internals.entry(element).or_default();
    }
    pub fn DidAttachElementInternals(&self, element: usize) -> bool {
        self.custom_elements
            .internals
            .get(&element)
            .is_some_and(|i| i.attached)
    }
    pub fn IsFormAssociatedCustomElement(&self, element: usize) -> bool {
        self.CustomElementDefinitionForElement(element)
            .is_some_and(|d| d.options.form_associated)
    }
    // The constructor may use internals before upgrade commits Custom state.
    pub fn ElementInternalsTargetIsFormAssociated(&self, element: usize) -> bool {
        self.IsFormAssociatedCustomElement(element)
            || (matches!(
                self.CustomElementState(element),
                CustomElementState::Undefined
                    | CustomElementState::Failed
                    | CustomElementState::Precustomized
            ) && self
                .LookupCustomElementDefinition(element)
                .is_some_and(|d| d.options.form_associated))
    }
    pub fn AttachElementInternals(&mut self, element: usize) -> Result<(), DOMException> {
        let definition = self
            .CustomElementDefinitionForElement(element)
            .or_else(|| self.LookupCustomElementDefinition(element));
        let supported = self.ElementIsValueUTF16(element).is_none()
            && definition.is_some_and(|d| !d.options.disable_internals)
            && matches!(
                self.CustomElementState(element),
                CustomElementState::Custom | CustomElementState::Precustomized
            );
        if !supported
            || self
                .custom_elements
                .internals
                .get(&element)
                .is_some_and(|i| i.attached)
        {
            return Err(DOMException {
                kind: DOMExceptionKind::NotSupportedError,
                message: "Element internals are unavailable or already attached.",
            });
        }
        self.EnsureElementInternals(element);
        self.custom_elements
            .internals
            .get_mut(&element)
            .unwrap()
            .attached = true;
        Ok(())
    }
    pub fn SetElementInternalsFormValue(
        &mut self,
        element: usize,
        value: Option<ElementInternalsValue>,
        state: Option<ElementInternalsValue>,
    ) {
        self.EnsureElementInternals(element);
        let internals = self.custom_elements.internals.get_mut(&element).unwrap();
        internals.value = value;
        internals.state = state;
    }
    pub fn ElementInternalsFormOwner(&self, element: usize) -> Option<usize> {
        self.custom_elements
            .internals
            .get(&element)
            .and_then(|i| i.form)
    }
    pub fn SetElementInternalsValidity(
        &mut self,
        element: usize,
        flags: u16,
        message: Rc<[u16]>,
        anchor: Option<usize>,
    ) -> Result<(), DOMException> {
        if anchor.is_some_and(|anchor| !self.IsShadowIncludingInclusiveAncestor(element, anchor)) {
            return Err(DOMException {
                kind: DOMExceptionKind::NotFoundError,
                message:
                    "The validation anchor must be a shadow-including descendant of the target.",
            });
        }
        self.EnsureElementInternals(element);
        let internals = self.custom_elements.internals.get_mut(&element).unwrap();
        internals.flags = flags;
        if internals.anchor.unwrap_or(element) != anchor.unwrap_or(element) {
            internals.anchor_revision = internals
                .anchor_revision
                .checked_add(1)
                .expect("ElementInternals validation anchor revision");
        }
        internals.anchor = anchor;
        self.SetControlCustomValidationMessageUTF16(element, message);
        self.SetControlNeedsValidityCheck(element);
        Ok(())
    }
    pub fn ElementInternalsValidityFlags(&self, element: usize) -> u16 {
        self.custom_elements
            .internals
            .get(&element)
            .map_or(0, |i| i.flags)
    }
    pub fn ElementInternalsValidationAnchor(&self, element: usize) -> usize {
        self.custom_elements
            .internals
            .get(&element)
            .and_then(|i| i.anchor)
            .unwrap_or(element)
    }
    pub fn ElementInternalsValidationAnchorRevision(&self, element: usize) -> u64 {
        self.custom_elements
            .internals
            .get(&element)
            .map_or(0, |i| i.anchor_revision)
    }
    pub fn AppendElementInternalsToFormData(&self, element: usize, data: &mut FormData) {
        if self.IsDisabledFormControl(element) {
            return;
        }
        let Some(value) = self
            .custom_elements
            .internals
            .get(&element)
            .and_then(|i| i.value.as_ref())
        else {
            return;
        };
        let name = self
            .Node(element)
            .FindAttribute("name")
            .map_or("", |a| a.value.as_str());
        match value {
            ElementInternalsValue::String(value) if !name.is_empty() => {
                data.AppendString(name.into(), value.clone())
            }
            ElementInternalsValue::File(file) if !name.is_empty() => {
                data.AppendFile(name.into(), file.clone())
            }
            ElementInternalsValue::FormData(value) => {
                for entry in value.Entries() {
                    match &entry.value {
                        EntryValue::String(value) => {
                            data.AppendString(entry.name.clone(), value.clone())
                        }
                        EntryValue::File(file) => data.AppendFile(entry.name.clone(), file.clone()),
                    }
                }
            }
            _ => {}
        }
    }
    pub fn ElementInternalsAnchorReferences(&self) -> Vec<(u64, Option<u64>)> {
        self.custom_elements
            .internals
            .iter()
            .map(|(&element, internals)| {
                (
                    self.Node(element).Id(),
                    internals.anchor.map(|anchor| self.Node(anchor).Id()),
                )
            })
            .collect()
    }
    pub fn ElementInternalsStateAt(&self, element: usize, index: usize) -> Option<String> {
        self.custom_elements
            .internals
            .get(&element)?
            .states
            .get(index)
            .cloned()
    }
    pub fn ElementInternalsStateIndex(&self, element: usize, state: &str) -> Option<usize> {
        self.custom_elements
            .internals
            .get(&element)?
            .states
            .iter()
            .position(|s| s == state)
    }
    pub fn ElementInternalsHasState(&self, element: usize, state: &str) -> bool {
        self.ElementInternalsStateIndex(element, state).is_some()
    }
    pub fn ChangeElementInternalsState(
        &mut self,
        element: usize,
        state: Option<String>,
        add: bool,
    ) -> bool {
        self.EnsureElementInternals(element);
        let internals = self.custom_elements.internals.get_mut(&element).unwrap();
        let mut changed = false;
        let clear = state.is_none();
        if let Some(state) = state {
            let index = internals.states.iter().position(|s| s == &state);
            if add && index.is_none() {
                internals.states.push(state);
                changed = true;
            } else if !add {
                if let Some(index) = index {
                    internals.states.remove(index);
                    changed = true;
                }
            }
        } else {
            changed = !internals.states.is_empty();
            internals.states.clear();
        }
        if changed || add || clear {
            self.ControlPseudoStateChanged(element, crate::style_state::ControlPseudoState::State);
        }
        changed
    }
    pub fn ElementInternalsLabels(&self, element: usize) -> Vec<usize> {
        fn visit(d: &PersistentDocument, node: usize, element: usize, labels: &mut Vec<usize>) {
            if d.Node(node).IsHTMLElement("label") && d.LabelControl(node) == Some(element) {
                labels.push(node);
            }
            for &child in d.Node(node).Children() {
                visit(d, child, element, labels);
            }
        }
        let mut labels = Vec::new();
        visit(self, self.TreeScopeRoot(element), element, &mut labels);
        labels
    }
    pub(crate) fn ElementInternalsDidUpgrade(&mut self, element: usize) {
        self.InvalidateElementInternalsCollections();
        self.SynchronizeElementInternals(element);
        self.SetControlNeedsValidityCheck(element);
        self.StatefulCustomElementDidUpgrade(element);
    }
    pub(crate) fn SynchronizeAllElementInternals(&mut self) {
        let elements: Vec<_> = self.custom_elements.internals.keys().copied().collect();
        for element in elements {
            self.SynchronizeElementInternals(element);
        }
    }
    fn SynchronizeElementInternals(&mut self, element: usize) {
        let Some(definition) = self
            .CustomElementDefinitionForElement(element)
            .filter(|d| d.options.form_associated)
        else {
            return;
        };
        let form = self.ComputeListedFormOwner(element);
        let disabled = self.IsDisabledFormControl(element);
        let internals = self.custom_elements.internals.get_mut(&element).unwrap();
        let old_form = internals.form;
        let form_changed = old_form != form;
        let disabled_changed = internals.disabled != disabled;
        internals.form = form;
        internals.disabled = disabled;
        if form_changed {
            for form in [old_form, form].into_iter().flatten() {
                self.ControlPseudoStateChanged(form, crate::style_state::ControlPseudoState::Valid);
                self.ControlPseudoStateChanged(
                    form,
                    crate::style_state::ControlPseudoState::Invalid,
                );
            }
        }
        if disabled_changed {
            self.InvalidateNodeStyle(element);
            self.SetControlNeedsValidityCheck(element);
        }
        if form_changed && definition.callbacks.form_associated {
            self.EnqueueInternalsReaction(CustomElementReaction::FormAssociated {
                element,
                id: definition.id,
                form,
            });
        }
        if disabled_changed && definition.callbacks.form_disabled {
            self.EnqueueInternalsReaction(CustomElementReaction::FormDisabled {
                element,
                id: definition.id,
                disabled,
            });
        }
    }
    pub fn EnqueueElementInternalsFormReset(&mut self, element: usize) {
        if let Some(definition) = self
            .CustomElementDefinitionForElement(element)
            .filter(|d| d.options.form_associated && d.callbacks.form_reset)
        {
            self.EnqueueInternalsReaction(CustomElementReaction::FormReset {
                element,
                id: definition.id,
            });
        }
    }
}
