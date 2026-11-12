/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2001 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2004, 2005, 2006, 2007, 2008, 2009 Apple Inc. All rights
 * reserved.
 *           (C) 2006 Alexey Proskuryakov (ap@nypop.com)
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
// Source: third_party/blink/renderer/core/html/forms/html_form_element.h:105-112,117,127-134,198,209-210,237,242,269,359-364,372
// Source: third_party/blink/renderer/core/html/forms/html_form_element.cc:527-537,552-590,592-1043,1077-1110,1485-1532
// This module owns only the translated interaction transaction flags. DOM owns
// the form, its live listed-elements collection, controls and attribute values.

use crate::core::loader::form_submission::FormSubmissionAttributes;
use interaction_dispatch::core::dom::events::event::{
    EventRef, EventTargetKind, EventTargetRef, PhaseType,
};
use interaction_dispatch::core::dom::events::event_dispatch_result::DispatchEventResult;
use interaction_dispatch::core::dom::events::event_path::LocalEventDispatch;
use interaction_dispatch::core::fileapi::{form_data::FormData, form_data_object::FormDataObject};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub type FormControlRef = Rc<dyn FormControl>;
pub type SubmitBehaviorRef = Rc<dyn SubmitBehavior>;
pub type FormToolRef = Rc<dyn FormTool>;
pub type SubmissionRef = Rc<dyn FormSubmission>;
pub type SubmissionTargetRef = Rc<dyn SubmissionTarget>;
pub type FormDialogRef = EventTargetRef;

/// Source html_form_element.h:364 and cc:1045-1074. Retained by the form;
/// the DOM and the entry data stay in their original owners.
#[derive(Default)]
pub struct FormEntryListState {
    constructing: Cell<bool>,
}
/// Source html_form_element.h:362-364. Both submission entry points and
/// ConstructEntryList use this one form-owned transaction state. The state
/// outlives synchronous listener reentry and contains no DOM/navigation copy.
#[derive(Default)]
pub struct FormSubmissionState {
    is_submitting: Cell<bool>,
    in_user_js_submit_event: Cell<bool>,
    entry_list: FormEntryListState,
    scheduled_submission: Rc<Cell<u64>>,
}
/// The source AutoReset lifetime; the previous value is restored before the
/// enclosing dialog EventQueueScope flushes events.
pub struct FormSubmissionScope<'a> {
    _scope: FlagScope<'a>,
}
/// Source AutoReset retained across a Page lifecycle handoff. Clones share
/// one exit point; abandonment restores the original form owner's flag.
pub struct OwnedFormSubmissionScope {
    state: Rc<FormSubmissionState>,
    previous: bool,
}
impl Drop for OwnedFormSubmissionScope {
    fn drop(&mut self) {
        self.state.is_submitting.set(self.previous);
    }
}
/// Opaque source form task version. The lease does not retain a form, DOM,
/// frame or callback; a later submission invalidates every earlier lease.
#[derive(Clone, Debug)]
pub struct FormSubmissionTaskToken {
    version: u64,
    owner: Weak<Cell<u64>>,
}
impl FormSubmissionTaskToken {
    pub fn IsCurrent(&self) -> bool {
        self.owner
            .upgrade()
            .is_some_and(|owner| owner.get() == self.version)
    }
}
impl FormSubmissionState {
    pub fn BeginScheduledSubmission(&self) -> FormSubmissionTaskToken {
        let version = self
            .scheduled_submission
            .get()
            .checked_add(1)
            .expect("form submission task version");
        self.scheduled_submission.set(version);
        FormSubmissionTaskToken {
            version,
            owner: Rc::downgrade(&self.scheduled_submission),
        }
    }
    pub fn EntryList(&self) -> &FormEntryListState {
        &self.entry_list
    }
    // Source html_form_element.cc:638-640.
    pub fn PreparationIsBlocked(&self) -> bool {
        self.is_submitting.get() || self.in_user_js_submit_event.get()
    }
    // Source html_form_element.cc:901-912, in source short-circuit order.
    pub fn SchedulingIsBlocked(&self) -> bool {
        self.entry_list.constructing.get() || self.is_submitting.get()
    }
    // Source html_form_element.cc:700-701, lifetime ends at cc:817.
    pub fn EnterSubmitEvent(&self) -> FormSubmissionScope<'_> {
        FormSubmissionScope {
            _scope: FlagScope::Set(&self.in_user_js_submit_event, true),
        }
    }
    pub fn EnterOwnedSubmission(self: &Rc<Self>) -> Rc<OwnedFormSubmissionScope> {
        Rc::new(OwnedFormSubmissionScope {
            state: self.clone(),
            previous: self.is_submitting.replace(true),
        })
    }
    // Source html_form_element.cc:916; lives through Create and finish effects.
    pub fn EnterSubmission(&self) -> FormSubmissionScope<'_> {
        FormSubmissionScope {
            _scope: FlagScope::Set(&self.is_submitting, true),
        }
    }
}
/// An owning continuation for ConstructEntryList's AutoReset lifetime. Layout
/// input is borrowed only while entries are appended; event dispatch happens
/// after the embedding releases that borrow. Dropping an abandoned transaction
/// restores both source flags.
pub struct PendingFormEntryList {
    state: Rc<FormSubmissionState>,
    activated: Option<Rc<Cell<bool>>>,
    data: Rc<FormDataObject>,
}
impl Drop for PendingFormEntryList {
    fn drop(&mut self) {
        if let Some(activated) = &self.activated {
            activated.set(false);
        }
        self.state.entry_list.constructing.set(false);
    }
}
impl PendingFormEntryList {
    pub fn Finish<C, O: FormEntryListOperations<C>>(self, operations: &O) -> Rc<FormData> {
        operations.DispatchFormData(self.data.clone());
        self.data.Read(|data| Rc::new(data.clone()))
    }
}
pub trait FormEntryListOperations<C> {
    type Error;
    type Submitter;
    fn ListedElements(&self) -> Box<dyn Iterator<Item = C> + '_>;
    fn SetActivatedSubmit(&self, control: &Self::Submitter, value: bool);
    fn IsDisabledForFormData(&self, control: &C) -> bool;
    fn AppendToFormData(&self, control: &C, data: &mut FormData) -> Result<(), Self::Error>;
    fn IsNonEmptyPasswordControl(&self, control: &C) -> bool;
    fn DispatchFormData(&self, data: Rc<FormDataObject>);
}
struct SourceFormEntries<'a, 'b> {
    form: &'a HTMLFormElement,
    scope: &'a FormDispatchScope<'b>,
}
impl FormEntryListOperations<FormControlRef> for SourceFormEntries<'_, '_> {
    type Error = std::convert::Infallible;
    type Submitter = FormControlRef;
    fn ListedElements(&self) -> Box<dyn Iterator<Item = FormControlRef> + '_> {
        self.form.environment.ListedElements()
    }
    fn SetActivatedSubmit(&self, control: &FormControlRef, value: bool) {
        control.SetActivatedSubmit(value);
    }
    fn IsDisabledForFormData(&self, control: &FormControlRef) -> bool {
        control.IsDisabledForFormData()
    }
    fn AppendToFormData(
        &self,
        control: &FormControlRef,
        data: &mut FormData,
    ) -> Result<(), Self::Error> {
        control.AppendToFormData(data);
        Ok(())
    }
    fn IsNonEmptyPasswordControl(&self, control: &FormControlRef) -> bool {
        control.IsNonEmptyPasswordControl()
    }
    fn DispatchFormData(&self, data: Rc<FormDataObject>) {
        self.form.environment.DispatchFormData(data, self.scope);
    }
}

pub trait FormUserValidityOperations<C> {
    fn ListedElements(&self) -> Box<dyn Iterator<Item = C> + '_>;
    fn HasState(&self, control: &C) -> bool;
    fn ForceUserValid(&self, control: &C);
}
struct SourceFormUserValidity<'a>(&'a HTMLFormElement);
impl FormUserValidityOperations<FormControlRef> for SourceFormUserValidity<'_> {
    fn ListedElements(&self) -> Box<dyn Iterator<Item = FormControlRef> + '_> {
        self.0.environment.ListedElements()
    }
    fn HasState(&self, control: &FormControlRef) -> bool {
        control.HasState()
    }
    fn ForceUserValid(&self, control: &FormControlRef) {
        control.ForceUserValid();
    }
}
/// Source validation order, independent of DOM/control storage and thread.
/// Snapshot only listed identities before invalid listeners reenter the form.
pub trait FormValidationOperations<C> {
    fn ListedElements(&self) -> Box<dyn Iterator<Item = C> + '_>;
    fn IsOwnedByForm(&self, control: &C) -> bool;
    fn ShouldCheckValidity(&self, control: &C) -> bool;
    fn CheckValidity(&self, control: &C, collect_unhandled: bool) -> FormControlValidity;
    fn HideVisibleValidationMessage(&self, control: &C);
    fn PrepareValidation(&self);
    fn ValidationAnchorOrHostIsFocusable(&self, control: &C) -> bool;
    fn ShowValidationMessage(&self, control: &C);
}
struct SourceFormValidation<'a>(&'a HTMLFormElement);
impl FormValidationOperations<FormControlRef> for SourceFormValidation<'_> {
    fn ListedElements(&self) -> Box<dyn Iterator<Item = FormControlRef> + '_> {
        self.0.environment.ListedElements()
    }
    fn IsOwnedByForm(&self, control: &FormControlRef) -> bool {
        control
            .FormOwner()
            .is_some_and(|owner| Rc::ptr_eq(&owner, &self.0.Target()))
    }
    fn ShouldCheckValidity(&self, control: &FormControlRef) -> bool {
        if control.IsNativeFormControl() {
            control.IsSubmittableElement()
        } else {
            control.IsElementInternals()
        }
    }
    fn CheckValidity(&self, control: &FormControlRef, collect: bool) -> FormControlValidity {
        control.CheckValidity(collect)
    }
    fn HideVisibleValidationMessage(&self, control: &FormControlRef) {
        control.HideVisibleValidationMessage();
    }
    fn PrepareValidation(&self) {
        let _prepared = self.0.environment.PreparedValidation();
    }
    fn ValidationAnchorOrHostIsFocusable(&self, control: &FormControlRef) -> bool {
        control.ValidationAnchorOrHostIsFocusable()
    }
    fn ShowValidationMessage(&self, control: &FormControlRef) {
        control.ShowValidationMessage();
    }
}

#[derive(Default)]
pub struct FormResetState {
    in_reset: Cell<bool>,
}
pub trait FormResetOperations<C> {
    fn HasFrame(&self) -> bool;
    fn DispatchReset(&self) -> DispatchEventResult;
    fn ListedElements(&self) -> Box<dyn Iterator<Item = C> + '_>;
    fn IsNativeFormControl(&self, control: &C) -> bool;
    fn ResetControl(&self, control: &C);
    fn IsElementInternals(&self, control: &C) -> bool;
    fn EnqueueFormResetCallback(&self, control: &C);
    fn CancelActiveTool(&self);
    fn HasPage(&self) -> bool;
    fn FormElementReset(&self);
}

/// Only source implicit-submission primitives. Listed elements and their
/// identities remain with the existing DOM collection owner.
pub trait ImplicitSubmissionOperations<C> {
    fn ListedElements(&self) -> Box<dyn Iterator<Item = C> + '_>;
    fn IsNativeFormControl(&self, control: &C) -> bool;
    fn CanBeSuccessfulSubmitButton(&self, control: &C) -> bool;
    fn IsSuccessfulSubmitButton(&self, control: &C) -> bool;
    fn CanTriggerImplicitSubmission(&self, control: &C) -> bool;
    fn SubmitBehaviorIsEffectivelyDisabled(&self, control: &C) -> Option<bool>;
    fn DispatchSimulatedClick(&self, control: &C);
    fn PrepareForSubmission(&self);
}

// Source: html_form_element.cc:552-588,662-694,703-715,735-752,855-869,1092-1098,1485-1495.
// All identities project the existing DOM control; no alternate control tree.
pub trait FormControl {
    fn Target(&self) -> EventTargetRef;
    fn IsNativeFormControl(&self) -> bool;
    fn CanBeSuccessfulSubmitButton(&self) -> bool;
    fn IsSuccessfulSubmitButton(&self) -> bool;
    fn IsActivatedSubmit(&self) -> bool;
    fn CanTriggerImplicitSubmission(&self) -> bool;
    fn SubmitBehavior(&self) -> Option<SubmitBehaviorRef>;
    fn DispatchSimulatedClick(&self, event: &EventRef);
    fn FormOwner(&self) -> Option<EventTargetRef>;
    fn BlocksFormSubmission(&self) -> bool;
    fn HasState(&self) -> bool;
    fn ForceUserValid(&self);
    fn FormNoValidate(&self) -> bool;
    fn HideVisibleValidationMessage(&self);
    fn ValidationAnchorOrHostIsFocusable(&self) -> bool;
    fn ShowValidationMessage(&self);
    fn IsNotCandidateOrValid(&self) -> bool;
    fn WebMCPParameterName(&self) -> Option<String>;
    fn validationMessage(&self) -> Vec<u16>;
    fn Reset(&self);
    fn IsElementInternals(&self) -> bool;
    fn EnqueueFormResetCallback(&self);
    fn IsSubmittableElement(&self) -> bool;
    fn CheckValidity(&self, collect_unhandled: bool) -> FormControlValidity;
    fn IsDisabledForFormData(&self) -> bool;
    fn SetActivatedSubmit(&self, value: bool);
    fn AppendToFormData(&self, data: &mut FormData);
    fn IsNonEmptyPasswordControl(&self) -> bool;
}
pub struct FormControlValidity {
    pub valid: bool,
    pub unhandled: bool,
}
pub trait SubmitBehavior {
    fn IsEffectivelyDisabled(&self) -> bool;
    fn formNoValidate(&self) -> bool;
}

// Source: html_form_element.cc:720-727,756-758,769-813,825-829,837-842,1101-1104.
// The same executing tool owner survives script disconnection of the form.
pub trait FormTool {
    fn CurrentlyRunning(&self) -> bool;
    fn SetHandlingSubmit(&self, value: bool) -> bool;
    fn Done(&self, result: FormToolResult);
    fn ObserveRespondWith(&self, promise: RespondWithPromise);
}
pub enum FormToolResult {
    Navigation,
    InvocationFailed(Vec<u16>),
    Cancelled(String),
}
pub trait FormRespondWithPromise {
    fn ContextIsValid(&self) -> bool;
}
pub type RespondWithPromise = Rc<dyn FormRespondWithPromise>;

// Source: html_form_element.cc:761-776,789-790.
pub struct SubmitEventOutcome {
    pub result: DispatchEventResult,
    pub respond_with: Option<RespondWithPromise>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmitMethod {
    Get,
    Post,
    Dialog,
}
pub trait FormSubmission: std::any::Any {
    fn Method(&self) -> SubmitMethod;
    fn ActionIsEmpty(&self) -> bool;
    fn ActionIsJavaScript(&self) -> bool;
    fn HasData(&self) -> bool;
    fn TargetFrame(&self) -> Option<SubmissionTargetRef>;
    fn Result(&self) -> String;
}
pub trait SubmissionTarget {
    fn Identity(&self) -> [u64; 2];
    fn HasPage(&self) -> bool;
    fn NavigationShouldReplaceCurrentHistoryEntry(
        &self,
        request: &crate::core::loader::frame_load_request::FrameLoadRequest,
        load: navigation_history::FrameLoadType,
    ) -> bool;
    fn NotifyFormNavigation(
        &self,
        request: &crate::core::loader::frame_load_request::FrameLoadRequest,
    );
    fn Navigate(
        &self,
        request: Rc<crate::core::loader::frame_load_request::FrameLoadRequest>,
        load: navigation_history::FrameLoadType,
    );
    fn ScheduleFormSubmission(
        &self,
        runner: &dyn crate::core::loader::frame_navigation_lifecycle::FormSubmissionTaskRunner,
        submission: Rc<crate::core::loader::form_submission::FormSubmissionOwner>,
    ) -> CancelSubmission;
    fn IsLocal(&self) -> bool;
    fn IsNavigationAllowed(&self) -> bool;
    fn IsFormFrame(&self) -> bool;
}

// Source: html_form_element.cc:875-879,1006-1042. Effects route to the existing
// navigation/document owner at the same source call site; it makes no form
// decisions. The immutable submission owner is never reconstructed by Page.
pub enum FormSubmissionEffect {
    CloseDialog {
        dialog: FormDialogRef,
        result: String,
    },
    Navigate(SubmissionRef),
    CancelParsing(SubmissionTargetRef),
    CancelPendingJavaScriptUrls(SubmissionTargetRef),
    CancelClientNavigation(SubmissionTargetRef),
}
pub type CancelSubmission = Box<dyn FnOnce()>;

/// Readiness for the source's focusable validation query at cc:605-607.
/// The assembly prepares resident geometry before invoking ValidateInteractively.
pub struct PreparedFormValidation;
/// A synchronous JS entry lends its current listener dispatcher. The form
/// never stores this borrow or reaches back into the JavaScript runtime.
#[derive(Default)]
pub struct FormDispatchScope<'a> {
    pub local_events: Option<&'a dyn LocalEventDispatch>,
}
pub trait FormEnvironment {
    fn Target(&self) -> EventTargetRef;
    fn HasFrame(&self) -> bool;
    fn HasViewAndPage(&self) -> bool;
    fn HasPage(&self) -> bool;
    fn IsConnected(&self) -> bool;
    fn IsSandboxedForms(&self) -> bool;
    fn NoValidate(&self) -> bool;
    fn UnclosedFormControlIsInvalidEnabled(&self) -> bool;
    fn ListedElements(&self) -> Box<dyn Iterator<Item = FormControlRef> + '_>;
    fn PreparedValidation(&self) -> PreparedFormValidation;
    fn DispatchError(&self);
    fn DispatchReset(&self) -> DispatchEventResult;
    fn DispatchSubmit(
        &self,
        submitter: Option<FormControlRef>,
        composed: bool,
        agent_invoked: bool,
        scope: &FormDispatchScope<'_>,
    ) -> SubmitEventOutcome;
    fn SubmitterShadowReferenceTargetEnabled(&self, submitter: &FormControlRef) -> bool;
    fn DispatchWillSendSubmitEvent(&self);
    fn HandleLocalEvents(&self, event: &EventRef);
    fn CreateSubmission(
        &self,
        event: Option<&EventRef>,
        submitter: Option<FormControlRef>,
        scope: &FormDispatchScope<'_>,
        completion: Box<dyn FnOnce(Option<SubmissionRef>)>,
    );
    fn DispatchFormData(&self, data: Rc<FormDataObject>, scope: &FormDispatchScope<'_>);
    fn AllowFormAction(&self, submission: &SubmissionRef) -> bool;
    fn ApplySubmissionEffect(&self, effect: FormSubmissionEffect);
    fn FirstAncestorDialog(&self) -> Option<FormDialogRef>;
    fn ScheduleSubmission(
        &self,
        target: SubmissionTargetRef,
        use_target_scheduler: bool,
        submission: SubmissionRef,
    ) -> CancelSubmission;
    fn BeginDialogEventQueueScope(&self) -> Box<dyn FormEventQueueScope>;
    fn FormElementReset(&self);
}
pub trait FormEventQueueScope {}

/// Source html_form_element.cc:850-873. Admission reads the same control and
/// form identities that preparation will receive, before any script dispatch.
pub trait RequestSubmitOperations<C> {
    fn IsNativeFormControl(&self, control: &C) -> bool;
    fn CanBeSuccessfulSubmitButton(&self, control: &C) -> bool;
    fn IsOwnedByForm(&self, control: &C) -> bool;
    fn PrepareForSubmission(&self, submitter: Option<C>);
}
struct SourceRequestSubmit<'a, 'b> {
    form: &'a HTMLFormElement,
    scope: &'a FormDispatchScope<'b>,
}
impl RequestSubmitOperations<FormControlRef> for SourceRequestSubmit<'_, '_> {
    fn IsNativeFormControl(&self, control: &FormControlRef) -> bool {
        control.IsNativeFormControl()
    }
    fn CanBeSuccessfulSubmitButton(&self, control: &FormControlRef) -> bool {
        control.CanBeSuccessfulSubmitButton()
    }
    fn IsOwnedByForm(&self, control: &FormControlRef) -> bool {
        control
            .FormOwner()
            .is_some_and(|owner| Rc::ptr_eq(&owner, &self.form.Target()))
    }
    fn PrepareForSubmission(&self, submitter: Option<FormControlRef>) {
        self.form
            .PrepareForSubmissionWithScope(None, submitter, self.scope);
    }
}

// Source: html_form_element.h:359-364,372. Cell guards deliberately release
// borrows before script so a submit/reset listener can synchronously reenter.
pub struct HTMLFormElement {
    self_: Weak<Self>,
    attributes: RefCell<FormSubmissionAttributes>,
    environment: Rc<dyn FormEnvironment>,
    submission_state_: FormSubmissionState,
    reset_state_: FormResetState,
    cancel_last_submission_: RefCell<Option<CancelSubmission>>,
    active_webmcp_tool_: RefCell<Option<FormToolRef>>,
}
struct FlagScope<'a> {
    flag: &'a Cell<bool>,
    old: bool,
}
impl<'a> FlagScope<'a> {
    fn Set(flag: &'a Cell<bool>, value: bool) -> Self {
        Self {
            flag,
            old: flag.replace(value),
        }
    }
}
impl Drop for FlagScope<'_> {
    fn drop(&mut self) {
        self.flag.set(self.old);
    }
}
struct ToolSubmitScope {
    tool: FormToolRef,
    old: bool,
}
impl Drop for ToolSubmitScope {
    fn drop(&mut self) {
        self.tool.SetHandlingSubmit(self.old);
    }
}

impl HTMLFormElement {
    // html_form_element.h:360 / cc:722-724. The active tool is owned by this
    // retained form, rather than supplied by an embedding-wide lookup callback.
    pub fn ActiveWebMCPTool(&self) -> Option<FormToolRef> {
        self.active_webmcp_tool_.borrow().clone()
    }
    pub fn SetActiveWebMCPTool(&self, tool: Option<FormToolRef>) {
        *self.active_webmcp_tool_.borrow_mut() = tool;
    }
    pub fn new(environment: Rc<dyn FormEnvironment>) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            attributes: RefCell::new(FormSubmissionAttributes::default()),
            environment,
            submission_state_: FormSubmissionState::default(),
            reset_state_: FormResetState::default(),
            cancel_last_submission_: RefCell::new(None),
            active_webmcp_tool_: RefCell::new(None),
        })
    }
    pub fn ParseSubmissionAttribute(&self, name: &str, value: Option<&str>) {
        self.attributes.borrow_mut().ParseAttribute(name, value);
    }
    pub fn SubmissionAttributes(&self) -> FormSubmissionAttributes {
        self.attributes.borrow().clone()
    }
    // Source html_form_element.cc:1045-1074. Each Append dispatch borrows only
    // its original control; formdata listeners mutate the same data owner.
    pub fn ConstructEntryList(
        &self,
        submitter: Option<FormControlRef>,
        encoding: &str,
    ) -> Option<Rc<FormDataObject>> {
        self.ConstructEntryListWithScope(submitter, encoding, &FormDispatchScope::default())
    }
    pub fn ConstructEntryListWithScope(
        &self,
        submitter: Option<FormControlRef>,
        encoding: &str,
        scope: &FormDispatchScope<'_>,
    ) -> Option<Rc<FormDataObject>> {
        match Self::ConstructEntryListWithOperations(
            self.submission_state_.EntryList(),
            submitter,
            encoding,
            &SourceFormEntries { form: self, scope },
        ) {
            Ok(data) => data,
            Err(never) => match never {},
        }
    }
    // Source html_form_element.cc:1045-1074. The admission error is a typed
    // missing host input; the native projection above is infallible.
    pub fn ConstructEntryListWithOperations<C, O: FormEntryListOperations<C>>(
        state: &FormEntryListState,
        submitter: Option<O::Submitter>,
        encoding: &str,
        operations: &O,
    ) -> Result<Option<Rc<FormDataObject>>, O::Error> {
        if state.constructing.get() {
            return Ok(None);
        }
        let data = FormDataObject::new(FormData::WithEncoding(encoding));
        let _scope = FlagScope::Set(&state.constructing, true);
        struct Activated<'a, C, O: FormEntryListOperations<C>>(
            &'a O,
            Option<O::Submitter>,
            std::marker::PhantomData<C>,
        );
        impl<C, O: FormEntryListOperations<C>> Drop for Activated<'_, C, O> {
            fn drop(&mut self) {
                if let Some(control) = &self.1 {
                    self.0.SetActivatedSubmit(control, false);
                }
            }
        }
        if let Some(control) = &submitter {
            operations.SetActivatedSubmit(control, true);
        }
        let _activated = Activated::<C, O>(operations, submitter, std::marker::PhantomData);
        Self::AppendEntryListWithOperations(&data, operations)?;
        operations.DispatchFormData(data.clone());
        Ok(Some(data))
    }
    /// Same source entry loop as ConstructEntryList, with the original flag
    /// lifetime retained through the later formdata dispatch.
    pub fn PrepareEntryListWithOperations<C, O: FormEntryListOperations<C>>(
        state: Rc<FormSubmissionState>,
        activated: Option<Rc<Cell<bool>>>,
        encoding: &str,
        operations: &O,
    ) -> Result<Option<PendingFormEntryList>, O::Error> {
        if state.entry_list.constructing.replace(true) {
            return Ok(None);
        }
        if let Some(activated) = &activated {
            activated.set(true);
        }
        let pending = PendingFormEntryList {
            state,
            activated,
            data: FormDataObject::new(FormData::WithEncoding(encoding)),
        };
        Self::AppendEntryListWithOperations(&pending.data, operations)?;
        Ok(Some(pending))
    }
    fn AppendEntryListWithOperations<C, O: FormEntryListOperations<C>>(
        data: &FormDataObject,
        operations: &O,
    ) -> Result<(), O::Error> {
        for control in operations.ListedElements() {
            if !operations.IsDisabledForFormData(&control) {
                data.Mutate(|data| operations.AppendToFormData(&control, data))?;
            }
            if operations.IsNonEmptyPasswordControl(&control) {
                data.Mutate(|data| data.SetContainsPasswordData(true));
            }
        }
        Ok(())
    }
    pub fn Method(&self) -> SubmitMethod {
        self.attributes.borrow().method
    }
    pub fn Target(&self) -> EventTargetRef {
        self.environment.Target()
    }
    /// The form-data owner enters the same source flag while building entries.
    pub fn SetConstructingEntryList(&self, value: bool) -> bool {
        self.submission_state_
            .entry_list
            .constructing
            .replace(value)
    }

    /// FormData::Create cc:104-149. Returned entry-list copy is distinct from
    /// the object exposed during formdata dispatch, as required by the source.
    pub fn CreateFormData(
        &self,
        submitter: Option<FormControlRef>,
        scope: &FormDispatchScope<'_>,
    ) -> Result<
        Rc<FormDataObject>,
        interaction_dispatch::core::fileapi::form_data_object::FormDataConstructionError,
    > {
        use interaction_dispatch::core::fileapi::form_data_object::FormDataConstructionError as E;
        if let Some(control) = &submitter {
            if !control.IsNativeFormControl() || !control.CanBeSuccessfulSubmitButton() {
                return Err(E::TypeError(
                    "The specified element is not a submit button.",
                ));
            }
            if !control
                .FormOwner()
                .is_some_and(|owner| Rc::ptr_eq(&owner, &self.Target()))
            {
                return Err(E::NotFound(
                    "The specified element is not owned by this form element.",
                ));
            }
        }
        let data = self
            .ConstructEntryListWithScope(submitter, "UTF-8", scope)
            .ok_or(E::InvalidState("The form is constructing entry list."))?;
        Ok(FormDataObject::new(data.Read(Clone::clone)))
    }
    // Source: html_form_element.cc:527-537.
    pub fn HandleLocalEvents(&self, event: &EventRef) {
        self.HandleLocalEventsWithBase(event, &|event| self.environment.HandleLocalEvents(event));
    }
    /// The base call borrows the current native or JavaScript dispatch scope.
    /// This keeps the virtual override at the source Node local-event entry
    /// without storing a JavaScript runtime borrow in the form environment.
    pub fn HandleLocalEventsWithBase(&self, event: &EventRef, base: &dyn Fn(&EventRef)) {
        let stop = {
            let event = event.borrow();
            let base = event.BaseEvent();
            base.eventPhase() != PhaseType::kCapturingPhase
                && base.RawTarget().is_some_and(|target| {
                    target.Kind() == EventTargetKind::Node && !Rc::ptr_eq(&target, &self.Target())
                })
                && matches!(base.r#type(), "submit" | "reset")
        };
        if stop {
            event.borrow_mut().BaseEventMut().stopPropagation();
            return;
        }
        base(event);
    }

    // Source: html_form_element.cc:552-590.
    pub fn SubmitImplicitly(&self, event: &EventRef, from_implicit_submission_trigger: bool) {
        Self::SubmitImplicitlyWithOperations(
            &SourceImplicitSubmission { form: self, event },
            from_implicit_submission_trigger,
        );
    }
    // Source: html_form_element.cc:552-590. Shared by resident form owners and
    // opaque live-DOM ids; no form tree or validation algorithm is copied.
    pub fn SubmitImplicitlyWithOperations<C>(
        operations: &dyn ImplicitSubmissionOperations<C>,
        from_implicit_submission_trigger: bool,
    ) {
        let mut submission_trigger_count = 0;
        for control in operations.ListedElements() {
            if operations.IsNativeFormControl(&control) {
                if operations.CanBeSuccessfulSubmitButton(&control) {
                    if operations.IsSuccessfulSubmitButton(&control) {
                        operations.DispatchSimulatedClick(&control);
                        return;
                    }
                    if from_implicit_submission_trigger {
                        return;
                    }
                } else if operations.CanTriggerImplicitSubmission(&control) {
                    submission_trigger_count += 1;
                }
                continue;
            }
            if let Some(disabled) = operations.SubmitBehaviorIsEffectivelyDisabled(&control) {
                if !disabled {
                    operations.DispatchSimulatedClick(&control);
                    return;
                }
                if from_implicit_submission_trigger {
                    return;
                }
            }
        }
        if from_implicit_submission_trigger && submission_trigger_count == 1 {
            operations.PrepareForSubmission();
        }
    }
    // Source: html_form_element.cc:592-633.
    pub fn ValidateInteractively(&self) -> bool {
        Self::ValidateInteractivelyWithOperations(&SourceFormValidation(self))
    }
    pub fn ValidateInteractivelyWithOperations<C: Clone>(
        operations: &dyn FormValidationOperations<C>,
    ) -> bool {
        for control in operations.ListedElements() {
            operations.HideVisibleValidationMessage(&control);
        }
        let mut unhandled = Vec::new();
        if !Self::CheckInvalidControlsWithOperations(operations, Some(&mut unhandled)) {
            return true;
        }
        operations.PrepareValidation();
        for control in unhandled {
            if operations.ValidationAnchorOrHostIsFocusable(&control) {
                operations.ShowValidationMessage(&control);
                break;
            }
        }
        false
    }

    // Source cc:692-698: expose user-valid/user-invalid before the validation
    // bypass decision, including forms carrying novalidate.
    pub fn ExposeUserValidityWithOperations<C>(operations: &dyn FormUserValidityOperations<C>) {
        for control in operations.ListedElements() {
            if operations.HasState(&control) {
                operations.ForceUserValid(&control);
            }
        }
    }

    // Source: html_form_element.cc:635-832.
    pub fn PrepareForSubmission(
        &self,
        event: Option<&EventRef>,
        submitter: Option<FormControlRef>,
    ) {
        self.PrepareForSubmissionWithScope(event, submitter, &FormDispatchScope::default());
    }
    pub fn PrepareForSubmissionWithScope(
        &self,
        event: Option<&EventRef>,
        submitter: Option<FormControlRef>,
        scope: &FormDispatchScope<'_>,
    ) {
        if !self.environment.HasFrame() || self.submission_state_.PreparationIsBlocked() {
            return;
        }
        if !self.environment.IsConnected() {
            /* TODO cc:643-646 console. */
            return;
        }
        if self.environment.IsSandboxedForms() {
            /* TODO cc:652-658 console. */
            return;
        }
        for control in self.environment.ListedElements() {
            if control.IsNativeFormControl() && control.BlocksFormSubmission() {
                // TODO cc:666-667 UseCounter,669-677 console.
                if self.environment.UnclosedFormControlIsInvalidEnabled() {
                    self.environment.DispatchError();
                    return;
                }
            }
        }
        Self::ExposeUserValidityWithOperations(&SourceFormUserValidity(self));
        let should_submit;
        let executing_tool;
        {
            let _submit_scope = self.submission_state_.EnterSubmitEvent();
            let behavior = submitter
                .as_ref()
                .and_then(|control| control.SubmitBehavior());
            let submit_button = submitter
                .as_ref()
                .filter(|control| behavior.is_none() && control.IsNativeFormControl());
            let skip_validation = !self.environment.HasPage()
                || self.environment.NoValidate()
                || behavior
                    .as_ref()
                    .is_some_and(|behavior| behavior.formNoValidate())
                || submit_button.is_some_and(|control| control.FormNoValidate());
            // TODO cc:717 UseCounter.
            executing_tool = self.active_webmcp_tool_.borrow().clone();
            let declarative_call = executing_tool
                .as_ref()
                .is_some_and(|tool| tool.CurrentlyRunning());
            let _tool_scope = if declarative_call {
                let tool = executing_tool.as_ref().unwrap().clone();
                let old = tool.SetHandlingSubmit(true);
                Some(ToolSubmitScope { tool, old })
            } else {
                None
            };
            if !skip_validation && !self.ValidateInteractively() {
                should_submit = false;
                if declarative_call {
                    let mut message = "Form validation failed: "
                        .encode_utf16()
                        .collect::<Vec<_>>();
                    for control in self.environment.ListedElements() {
                        if !control.IsNotCandidateOrValid() {
                            message.extend(
                                control
                                    .WebMCPParameterName()
                                    .unwrap_or_else(|| "{unknown}".to_owned())
                                    .encode_utf16(),
                            );
                            message.extend(": ".encode_utf16());
                            message.extend(control.validationMessage());
                            message.extend(". ".encode_utf16());
                        }
                    }
                    executing_tool
                        .as_ref()
                        .unwrap()
                        .Done(FormToolResult::InvocationFailed(message));
                }
            } else {
                self.environment.DispatchWillSendSubmitEvent();
                let composed = submitter.as_ref().is_some_and(|control| {
                    self.environment
                        .SubmitterShadowReferenceTargetEnabled(control)
                });
                let outcome = self.environment.DispatchSubmit(
                    submitter.clone(),
                    composed,
                    declarative_call,
                    scope,
                );
                should_submit = outcome.result == DispatchEventResult::kNotCanceled;
                // The retained tool is rechecked after script can disconnect it.
                if executing_tool
                    .as_ref()
                    .is_some_and(|tool| tool.CurrentlyRunning())
                {
                    if let Some(promise) = outcome.respond_with {
                        assert!(!should_submit);
                        executing_tool.as_ref().unwrap().ObserveRespondWith(promise);
                    } else if !should_submit {
                        executing_tool.as_ref().unwrap().Done(FormToolResult::InvocationFailed("The site has a programming error: it called preventDefault() on the 'submit' event, without also calling respondWith() with the tool result".encode_utf16().collect()));
                    }
                }
            }
        }
        if should_submit {
            let cancel = self.cancel_last_submission_.borrow_mut().take();
            if let Some(cancel) = cancel {
                cancel();
            }
            self.ScheduleFormSubmissionWithScope(event, submitter, scope);
            if let Some(tool) = executing_tool.filter(|tool| tool.CurrentlyRunning()) {
                tool.Done(FormToolResult::Navigation);
            }
        }
    }

    // Source: html_form_element.cc:834-843.
    pub fn submitFromJavaScript(&self) {
        self.submitFromJavaScriptWithScope(&FormDispatchScope::default());
    }
    pub fn submitFromJavaScriptWithScope(&self, scope: &FormDispatchScope<'_>) {
        self.ScheduleFormSubmissionWithScope(None, None, scope);
        if let Some(tool) = self
            .ActiveWebMCPTool()
            .filter(|tool| tool.CurrentlyRunning())
        {
            tool.Done(FormToolResult::Navigation);
        }
    }
    // Source: html_form_element.cc:845-873.
    pub fn requestSubmit(
        &self,
        submitter: Option<FormControlRef>,
    ) -> Result<(), RequestSubmitError> {
        self.requestSubmitWithScope(submitter, &FormDispatchScope::default())
    }
    pub fn requestSubmitWithScope(
        &self,
        submitter: Option<FormControlRef>,
        scope: &FormDispatchScope<'_>,
    ) -> Result<(), RequestSubmitError> {
        Self::RequestSubmitWithOperations(submitter, &SourceRequestSubmit { form: self, scope })
    }
    pub fn RequestSubmitWithOperations<C>(
        submitter: Option<C>,
        operations: &dyn RequestSubmitOperations<C>,
    ) -> Result<(), RequestSubmitError> {
        if let Some(control) = &submitter {
            if !operations.IsNativeFormControl(control)
                || !operations.CanBeSuccessfulSubmitButton(control)
            {
                return Err(RequestSubmitError::NotSubmitButton);
            }
            if !operations.IsOwnedByForm(control) {
                return Err(RequestSubmitError::NotOwnedByForm);
            }
        }
        operations.PrepareForSubmission(submitter);
        Ok(())
    }
    // Source: html_form_element.cc:875-879.
    pub fn SubmitDialog(&self, submission: &SubmissionRef) {
        if let Some(dialog) = self.environment.FirstAncestorDialog() {
            self.environment
                .ApplySubmissionEffect(FormSubmissionEffect::CloseDialog {
                    dialog,
                    result: submission.Result(),
                });
        }
    }

    // Source: html_form_element.cc:882-1043.
    pub fn ScheduleFormSubmission(
        &self,
        event: Option<&EventRef>,
        mut submitter: Option<FormControlRef>,
    ) {
        self.ScheduleFormSubmissionWithScope(event, submitter, &FormDispatchScope::default());
    }
    pub fn ScheduleFormSubmissionWithScope(
        &self,
        event: Option<&EventRef>,
        mut submitter: Option<FormControlRef>,
        scope: &FormDispatchScope<'_>,
    ) {
        if !self.environment.HasViewAndPage() || !self.environment.HasFrame() {
            return;
        }
        if !self.environment.IsConnected() {
            /* TODO cc:894-897 console. */
            return;
        }
        if self.submission_state_.SchedulingIsBlocked() {
            // TODO cc:902-906 console when constructing the entry list.
            return;
        }
        let _dialog_scope = self.environment.BeginDialogEventQueueScope();
        let _submit_scope = self.submission_state_.EnterSubmission();
        if event.is_some() && submitter.is_none() {
            for control in self.environment.ListedElements() {
                if !control.IsNativeFormControl() {
                    continue;
                }
                debug_assert!(!control.IsActivatedSubmit());
                if control.IsSuccessfulSubmitButton() {
                    submitter = Some(control);
                    break;
                }
            }
        }
        let form = self.self_.clone();
        self.environment.CreateSubmission(
            event,
            submitter,
            scope,
            Box::new(move |submission| {
                if let (Some(form), Some(submission)) = (form.upgrade(), submission) {
                    form.FinishScheduleFormSubmission(submission);
                }
            }),
        );
    }
    fn FinishScheduleFormSubmission(&self, submission: SubmissionRef) {
        let target = submission.TargetFrame();
        if !self.environment.IsConnected() {
            /* TODO cc:947-950 console. */
            return;
        }
        if submission.Method() == SubmitMethod::Dialog {
            self.SubmitDialog(&submission);
            return;
        }
        debug_assert!(matches!(
            submission.Method(),
            SubmitMethod::Get | SubmitMethod::Post
        ));
        debug_assert!(submission.HasData());
        if submission.ActionIsEmpty() {
            return;
        }
        if self.environment.IsSandboxedForms() {
            /* TODO cc:968-975 console. */
            return;
        }
        if submission.ActionIsJavaScript() && !self.environment.AllowFormAction(&submission) {
            return;
        }
        // TODO cc:993-1001 UseCounter/MixedContent metrics.
        let Some(target) = target else {
            return;
        };
        if submission.ActionIsJavaScript() {
            self.environment
                .ApplySubmissionEffect(FormSubmissionEffect::Navigate(submission));
            return;
        }
        let use_target_scheduler = target.IsLocal();
        if target.IsLocal() {
            if !target.IsNavigationAllowed() {
                return;
            }
            if target.IsFormFrame() {
                self.environment
                    .ApplySubmissionEffect(FormSubmissionEffect::CancelParsing(target.clone()));
            }
            self.environment.ApplySubmissionEffect(
                FormSubmissionEffect::CancelPendingJavaScriptUrls(target.clone()),
            );
            self.environment
                .ApplySubmissionEffect(FormSubmissionEffect::CancelClientNavigation(
                    target.clone(),
                ));
        }
        let cancel = self
            .environment
            .ScheduleSubmission(target, use_target_scheduler, submission);
        *self.cancel_last_submission_.borrow_mut() = Some(cancel);
    }

    // Source: html_form_element.cc:1077-1110.
    pub fn reset(&self) {
        Self::ResetWithOperations(&self.reset_state_, &SourceFormReset { form: self });
    }
    // Source: html_form_element.cc:1077-1110. Retain the source reentry flag,
    // freeze listed-element identities before the first control reset callback.
    pub fn ResetWithOperations<C>(state: &FormResetState, operations: &dyn FormResetOperations<C>) {
        if state.in_reset.get() || !operations.HasFrame() {
            return;
        }
        state.in_reset.set(true);
        if operations.DispatchReset() != DispatchEventResult::kNotCanceled {
            state.in_reset.set(false);
            return;
        }
        let elements: Vec<_> = operations.ListedElements().collect();
        for control in elements {
            if operations.IsNativeFormControl(&control) {
                operations.ResetControl(&control);
            } else if operations.IsElementInternals(&control) {
                operations.EnqueueFormResetCallback(&control);
            }
        }
        operations.CancelActiveTool();
        state.in_reset.set(false);
        if operations.HasPage() {
            operations.FormElementReset();
        }
    }
    // Source: html_form_element.cc:1485-1495.
    pub fn FindDefaultButton(&self) -> Option<FormControlRef> {
        for control in self.environment.ListedElements() {
            if control.IsNativeFormControl() && control.CanBeSuccessfulSubmitButton() {
                return Some(control);
            }
            if control.SubmitBehavior().is_some() {
                return Some(control);
            }
        }
        None
    }

    // Source: html_form_element.cc:1497-1532. This exact source snapshot allows
    // invalid-event handlers to mutate the live form/listed-elements owner.
    pub fn checkValidity(&self) -> bool {
        !self.CheckInvalidControlsAndCollectUnhandled(None)
    }
    pub fn CheckInvalidControlsAndCollectUnhandled(
        &self,
        unhandled: Option<&mut Vec<FormControlRef>>,
    ) -> bool {
        Self::CheckInvalidControlsWithOperations(&SourceFormValidation(self), unhandled)
    }
    // Source cc:1497-1532: ownership is checked again after each invalid
    // callback; a listener may move a control into a different form.
    pub fn CheckInvalidControlsWithOperations<C: Clone>(
        operations: &dyn FormValidationOperations<C>,
        mut unhandled: Option<&mut Vec<C>>,
    ) -> bool {
        let elements: Vec<_> = operations.ListedElements().collect();
        let mut invalid_controls_count = 0;
        for control in elements {
            if !operations.IsOwnedByForm(&control) || !operations.ShouldCheckValidity(&control) {
                continue;
            }
            let result = operations.CheckValidity(&control, unhandled.is_some());
            if result.unhandled {
                if let Some(unhandled) = unhandled.as_deref_mut() {
                    unhandled.push(control.clone());
                }
            }
            if !result.valid && operations.IsOwnedByForm(&control) {
                invalid_controls_count += 1;
            }
        }
        invalid_controls_count != 0
    }

    pub fn reportValidity(&self) -> bool {
        self.ValidateInteractively()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestSubmitError {
    NotSubmitButton,
    NotOwnedByForm,
}
impl RequestSubmitError {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotSubmitButton => "The specified element is not a submit button.",
            Self::NotOwnedByForm => "The specified element is not owned by this form element.",
        }
    }
}

// HTMLInputElement retains the same form across its change-event reentry point.
impl super::html_input_element::HTMLInputFormSubmission for HTMLFormElement {
    fn SubmitImplicitly(&self, event: &EventRef, can_trigger: bool) {
        HTMLFormElement::SubmitImplicitly(self, event, can_trigger);
    }
}

struct SourceImplicitSubmission<'a> {
    form: &'a HTMLFormElement,
    event: &'a EventRef,
}
impl ImplicitSubmissionOperations<FormControlRef> for SourceImplicitSubmission<'_> {
    fn ListedElements(&self) -> Box<dyn Iterator<Item = FormControlRef> + '_> {
        self.form.environment.ListedElements()
    }
    fn IsNativeFormControl(&self, control: &FormControlRef) -> bool {
        control.IsNativeFormControl()
    }
    fn CanBeSuccessfulSubmitButton(&self, control: &FormControlRef) -> bool {
        control.CanBeSuccessfulSubmitButton()
    }
    fn IsSuccessfulSubmitButton(&self, control: &FormControlRef) -> bool {
        control.IsSuccessfulSubmitButton()
    }
    fn CanTriggerImplicitSubmission(&self, control: &FormControlRef) -> bool {
        control.CanTriggerImplicitSubmission()
    }
    fn SubmitBehaviorIsEffectivelyDisabled(&self, control: &FormControlRef) -> Option<bool> {
        control
            .SubmitBehavior()
            .map(|behavior| behavior.IsEffectivelyDisabled())
    }
    fn DispatchSimulatedClick(&self, control: &FormControlRef) {
        control.DispatchSimulatedClick(self.event);
    }
    fn PrepareForSubmission(&self) {
        self.form.PrepareForSubmission(Some(self.event), None);
    }
}

struct SourceFormReset<'a> {
    form: &'a HTMLFormElement,
}
impl FormResetOperations<FormControlRef> for SourceFormReset<'_> {
    fn HasFrame(&self) -> bool {
        self.form.environment.HasFrame()
    }
    fn DispatchReset(&self) -> DispatchEventResult {
        self.form.environment.DispatchReset()
    }
    fn ListedElements(&self) -> Box<dyn Iterator<Item = FormControlRef> + '_> {
        self.form.environment.ListedElements()
    }
    fn IsNativeFormControl(&self, control: &FormControlRef) -> bool {
        control.IsNativeFormControl()
    }
    fn ResetControl(&self, control: &FormControlRef) {
        control.Reset();
    }
    fn IsElementInternals(&self, control: &FormControlRef) -> bool {
        control.IsElementInternals()
    }
    fn EnqueueFormResetCallback(&self, control: &FormControlRef) {
        control.EnqueueFormResetCallback();
    }
    fn CancelActiveTool(&self) {
        if let Some(tool) = self.form.ActiveWebMCPTool() {
            tool.Done(FormToolResult::Cancelled(
                "Tool execution cancelled by a form reset".to_owned(),
            ));
        }
    }
    fn HasPage(&self) -> bool {
        self.form.environment.HasPage()
    }
    fn FormElementReset(&self) {
        self.form.environment.FormElementReset();
    }
}

#[cfg(test)]
mod continuation_tests {
    use super::*;
    #[test]
    fn submission_continuation_blocks_reentry_until_last_owner_exits() {
        let state = Rc::new(FormSubmissionState::default());
        let transaction = state.EnterOwnedSubmission();
        let continuation = transaction.clone();
        drop(transaction);
        assert!(state.PreparationIsBlocked());
        assert!(state.SchedulingIsBlocked());
        drop(continuation);
        assert!(!state.PreparationIsBlocked());
        assert!(!state.SchedulingIsBlocked());
    }
}
