/*
 * Copyright (C) 2006, 2007 Apple Inc. All rights reserved.
 * Copyright (C) 2008 Nuanti Ltd.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/page/focus_controller.h:39-58,69-88,116-131,135,148-160
// Source: third_party/blink/renderer/core/page/focus_controller.cc:1113-1175,1755-1760,1784-2023,2368-2471,2489-2493
// Scope/tabindex traversal is translated in focus_navigation.rs.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::core::dom::element::SelectionBehaviorOnFocus;
use crate::core::dom::focus_params::FocusParams;
use crate::public::mojom::input::focus_type::FocusType;

/// Call-scoped Page bridge which creates a fresh canonical Event/projection for
/// every focus/blur dispatch. It must not reuse the triggering key/mouse event's
/// LocalEventDispatch because that owner is bound to a different Event.
pub trait FocusEventDispatch {
    fn DispatchElementFocusEvent(
        &self,
        target: u64,
        event_type: &'static str,
        related_target: Option<u64>,
        focus_type: FocusType,
    );
    fn DispatchWindowFocusEvent(&self, event_type: &'static str);
    fn DispatchElementFocusEventInDocument(
        &self,
        _document: u128,
        target: u64,
        event_type: &'static str,
        related_target: Option<u64>,
        focus_type: FocusType,
    ) {
        self.DispatchElementFocusEvent(target, event_type, related_target, focus_type);
    }
    fn DispatchWindowFocusEventInDocument(&self, _document: u128, event_type: &'static str) {
        self.DispatchWindowFocusEvent(event_type);
    }

    fn StopNodeAutoscroll(&self, _node: u64) {}
}

/// Script-free dispatch context borrowed for a synchronous focus transaction.
/// Native entry points use the retained source dispatcher; JavaScript host
/// continuations supply their current binding-local listener invoker.
#[derive(Clone, Copy, Default)]
pub struct FocusDispatchScope<'a> {
    pub local_events:
        Option<&'a dyn interaction_dispatch::core::dom::events::event_path::LocalEventDispatch>,
    pub focus_events: Option<&'a dyn FocusEventDispatch>,
}

// Source: focus_controller.h:39-49,153-154. Each handle retains the original
// document/frame/element owner; these ports contain no copied tree or layout.
pub type FocusPageRef = Rc<dyn FocusPage>;
pub type FocusFrameRef = Rc<dyn FocusFrame>;
pub type FocusDocumentRef = Rc<dyn FocusDocument>;
pub type FocusElementRef = Rc<dyn FocusElement>;

// Source: focus_controller.cc:1785,1858,1901-1908,1989,2001-2005
pub trait FocusPage {
    fn Identity(&self) -> Rc<()>;
    fn MainFrame(&self) -> Option<FocusFrameRef>;
    fn IsMainFrameFencedFrameRoot(&self) -> bool;
    fn CanTakeFocus(&self, direction: FocusType) -> bool;
    fn TakeFocus(&self, direction: FocusType);
}

// Source: focus_controller.cc:1808-1852,1862-1908,1918-1950,1957-1972,2397-2405,2433-2443
// The frame port is limited to focus ownership and the existing frame tree.
pub trait FocusFrame {
    fn PageIdentity(&self) -> Option<Rc<()>>;
    fn IsLocalFrame(&self) -> bool;
    fn IsAttached(&self) -> bool;
    fn IsOutermostMainFrame(&self) -> bool;
    fn HasView(&self) -> bool;
    fn Owner(&self) -> Option<FocusElementRef>;
    fn Document(&self) -> Option<FocusDocumentRef>;
    fn LocalFrameRootDocument(&self) -> FocusDocumentRef;
    fn Parent(&self) -> Option<FocusFrameRef>;
    fn Top(&self) -> FocusFrameRef;
    fn LocalFrameRoot(&self) -> FocusFrameRef;
    fn AdvanceFocusFromRemote(&self, direction: FocusType, from: &FocusFrameRef);
    fn TraverseNext(&self) -> Option<FocusFrameRef>;
    fn IsDescendantOf(&self, frame: Option<&FocusFrameRef>) -> bool;
    fn SetSelectionFrameIsFocused(&self, focused: bool);
    fn DispatchWindowEvent(&self, event_type: &'static str, scope: &FocusDispatchScope<'_>);
    fn DidFocus(&self);
    fn StopAutoscroll(&self);
    fn StopAutoscrollWithScope(&self, _scope: &FocusDispatchScope<'_>) {
        self.StopAutoscroll();
    }
    fn WillChangeInputMethodFocus(&self);
    fn SelectionPageActivationChanged(&self);
}

// Source: focus_controller.cc:1113-1175,1864-1875,1928-1950,2385-2425,2434-2442
pub trait FocusDocument {
    fn FocusedElement(&self) -> Option<FocusElementRef>;
    fn ActiveElement(&self) -> Option<FocusElementRef>;
    fn Frame(&self) -> Option<FocusFrameRef>;
    fn ClearFocusedElement(&self, scope: &FocusDispatchScope<'_>);
    fn SetFocusedElement(
        &self,
        element: Option<FocusElementRef>,
        params: &FocusParams,
        scope: &FocusDispatchScope<'_>,
    ) -> bool;
    fn IsActive(&self) -> bool;
    fn UnloadStarted(&self) -> bool;
    fn HasView(&self) -> bool;
    fn IsPagePaused(&self) -> bool;
    fn DispatchWindowEvent(&self, event_type: &'static str, scope: &FocusDispatchScope<'_>);
    fn FocusNavigation(&self) -> Rc<dyn super::focus_navigation::FocusNavigationTree>;
    fn SequentialFocusNavigationStartingPoint(
        &self,
        direction: FocusType,
    ) -> Option<super::focus_navigation::FocusNavigationNode>;
    fn ClearSequentialFocusNavigationStartingPoint(&self);
    fn UpdateStyleAndLayoutForFocus(&self);
}

// Source: focus_controller.cc:1113-1175,1796-1804,1826-1830,1937-1942,2368-2395
pub trait FocusElement {
    fn Document(&self) -> FocusDocumentRef;
    fn IsFrameOwnerElement(&self) -> bool;
    fn IsFencedFrameElement(&self) -> bool;
    fn Focus(&self, params: &FocusParams, scope: &FocusDispatchScope<'_>);
    fn ContentFrame(&self) -> Option<FocusFrameRef>;
    fn IsPlugin(&self) -> bool;
    fn IsKeyboardFocusable(&self) -> bool;
    fn IsRootEditableElement(&self) -> bool;
    fn IsEditable(&self) -> bool;
    fn HasRootEditableElement(&self) -> bool;
    fn HandleInterestForHoverOrFocus(&self, focused: bool);
    fn SetFocused(&self, focused: bool, focus_type: FocusType);
    fn SetHasFocusWithinUpToAncestor(&self, focused: bool, need_snap_container_search: bool);
    fn DispatchBlurEvent(&self, focus_type: FocusType, scope: &FocusDispatchScope<'_>);
    fn DispatchFocusEvent(&self, focus_type: FocusType, scope: &FocusDispatchScope<'_>);
    fn DispatchFocusOutEvent(&self, event_type: &'static str, scope: &FocusDispatchScope<'_>);
    fn DispatchFocusInEvent(
        &self,
        event_type: &'static str,
        focus_type: FocusType,
        scope: &FocusDispatchScope<'_>,
    );
}

// Source: focus_controller.cc:2441-2442. Page routes this synchronous typed
// invalidation fact to its resident rendering owner at the source call point.
#[derive(Clone)]
pub enum FocusEffect {
    InvalidateAllCustomScrollbarsOnActiveChanged { document: FocusDocumentRef },
}
pub type FocusEffectEmitter = Rc<dyn Fn(FocusEffect)>;

// Source: focus_controller.h:43,131,159; focus_controller.cc:2456-2471
pub trait FocusChangedObserver {
    fn FocusedFrameChanged(&self);
}

fn SameFrame(a: Option<&FocusFrameRef>, b: Option<&FocusFrameRef>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
fn SameDocument(a: Option<&FocusDocumentRef>, b: Option<&FocusDocumentRef>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
fn SameElement(a: Option<&FocusElementRef>, b: Option<&FocusElementRef>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

// Source: focus_controller.cc:1113-1122
fn DispatchBlurEvent(
    document: &FocusDocumentRef,
    focused_element: &FocusElementRef,
    scope: &FocusDispatchScope<'_>,
) {
    focused_element.DispatchBlurEvent(FocusType::kPage, scope);
    if SameElement(Some(focused_element), document.FocusedElement().as_ref()) {
        focused_element.DispatchFocusOutEvent("focusout", scope);
        if SameElement(Some(focused_element), document.FocusedElement().as_ref()) {
            focused_element.DispatchFocusOutEvent("DOMFocusOut", scope);
        }
    }
}

// Source: focus_controller.cc:1124-1136
fn DispatchFocusEvent(
    document: &FocusDocumentRef,
    focused_element: &FocusElementRef,
    scope: &FocusDispatchScope<'_>,
) {
    focused_element.DispatchFocusEvent(FocusType::kPage, scope);
    if SameElement(Some(focused_element), document.FocusedElement().as_ref()) {
        focused_element.DispatchFocusInEvent("focusin", FocusType::kPage, scope);
        if SameElement(Some(focused_element), document.FocusedElement().as_ref()) {
            focused_element.DispatchFocusInEvent("DOMFocusIn", FocusType::kPage, scope);
        }
    }
}

// Source: focus_controller.cc:1138-1175
fn DispatchEventsOnWindowAndFocusedElement(
    document: &FocusDocumentRef,
    focused: bool,
    scope: &FocusDispatchScope<'_>,
) {
    if document.IsPagePaused() {
        return;
    }
    if !focused {
        if let Some(element) = document.FocusedElement() {
            element.SetFocused(false, FocusType::kPage);
            element.SetHasFocusWithinUpToAncestor(false, false);
            DispatchBlurEvent(document, &element, scope);
        }
    }
    document.DispatchWindowEvent(if focused { "focus" } else { "blur" }, scope);
    if focused {
        if let Some(element) = document.FocusedElement() {
            element.SetFocused(true, FocusType::kPage);
            element.SetHasFocusWithinUpToAncestor(true, false);
            DispatchFocusEvent(document, &element, scope);
        }
    }
}

// Source: focus_controller.h:51-58,148-159
// Interior mutability permits source synchronous callbacks to reenter this
// same owner. Every borrow is released before invoking any external operation.
pub struct FocusController {
    page_: FocusPageRef,
    focused_frame_: RefCell<Option<FocusFrameRef>>,
    is_active_: Cell<bool>,
    is_focused_: Cell<bool>,
    is_changing_focused_frame_: Cell<bool>,
    is_emulating_focus_: Cell<bool>,
    focus_changed_observers_: RefCell<HashMap<usize, Weak<dyn FocusChangedObserver>>>,
    emit_: FocusEffectEmitter,
}

impl FocusController {
    // Source: focus_controller.h:56-58; focus_controller.cc:1755-1760
    pub fn new(page: FocusPageRef, emit: FocusEffectEmitter) -> Self {
        Self {
            page_: page,
            focused_frame_: RefCell::new(None),
            is_active_: Cell::new(false),
            is_focused_: Cell::new(false),
            is_changing_focused_frame_: Cell::new(false),
            is_emulating_focus_: Cell::new(false),
            focus_changed_observers_: RefCell::new(HashMap::new()),
            emit_: emit,
        }
    }

    // Source: focus_controller.h:69; focus_controller.cc:1784-1855
    pub fn SetFocusedFrame(&self, frame: Option<FocusFrameRef>, notify_embedder: bool) {
        self.SetFocusedFrameWithScope(frame, notify_embedder, &FocusDispatchScope::default());
    }

    pub fn SetFocusedFrameWithScope(
        &self,
        frame: Option<FocusFrameRef>,
        notify_embedder: bool,
        scope: &FocusDispatchScope<'_>,
    ) {
        debug_assert!(frame.as_ref().is_none_or(|frame| frame
            .PageIdentity()
            .is_some_and(|page| Rc::ptr_eq(&page, &self.page_.Identity()))));
        if SameFrame(self.FocusedFrameIncludingRemote().as_ref(), frame.as_ref())
            || (self.is_changing_focused_frame_.get() && frame.is_some())
        {
            return;
        }
        self.is_changing_focused_frame_.set(true);
        if let Some(frame) = &frame {
            if let Some(owner) = frame
                .Owner()
                .filter(|owner| owner.IsFrameOwnerElement() && owner.IsFencedFrameElement())
            {
                self.SetFocusedElementWithScope(Some(owner), Some(frame.clone()), scope);
            }
        }
        let old = self.FocusedFrameIncludingRemote();
        let old_local = old.as_ref().filter(|frame| frame.IsLocalFrame()).cloned();
        let new_local = frame.as_ref().filter(|frame| frame.IsLocalFrame()).cloned();
        let old_remote = old.as_ref().filter(|frame| !frame.IsLocalFrame());
        let new_remote = frame
            .as_ref()
            .filter(|frame| !frame.IsLocalFrame())
            .cloned();
        *self.focused_frame_.borrow_mut() = frame;
        let old_owner = old_remote
            .and_then(|frame| frame.Owner())
            .filter(|owner| owner.IsFrameOwnerElement());
        let new_owner = new_remote
            .and_then(|frame| frame.Owner())
            .filter(|owner| owner.IsFrameOwnerElement());
        if !SameElement(old_owner.as_ref(), new_owner.as_ref()) {
            if let Some(owner) = old_owner {
                owner.HandleInterestForHoverOrFocus(false);
            }
            if let Some(owner) = new_owner {
                owner.HandleInterestForHoverOrFocus(true);
            }
        }
        if let Some(old) = old_local.filter(|frame| frame.HasView()) {
            old.SetSelectionFrameIsFocused(false);
            old.DispatchWindowEvent("blur", scope);
        }
        if let Some(new) = new_local.filter(|frame| frame.HasView() && self.IsFocused()) {
            new.SetSelectionFrameIsFocused(true);
            new.DispatchWindowEvent("focus", scope);
        }
        self.is_changing_focused_frame_.set(false);
        if notify_embedder {
            if let Some(frame) = self
                .FocusedFrameIncludingRemote()
                .filter(|frame| frame.IsAttached())
            {
                frame.DidFocus();
            }
        }
        self.NotifyFocusChangedObservers();
    }

    // Source: focus_controller.h:70; focus_controller.cc:1857-1884
    pub fn FocusDocumentView(&self, frame: Option<FocusFrameRef>, notify_embedder: bool) {
        self.FocusDocumentViewWithScope(frame, notify_embedder, &FocusDispatchScope::default());
    }
    pub fn FocusDocumentViewWithScope(
        &self,
        frame: Option<FocusFrameRef>,
        notify_embedder: bool,
        scope: &FocusDispatchScope<'_>,
    ) {
        debug_assert!(frame.as_ref().is_none_or(|frame| frame
            .PageIdentity()
            .is_some_and(|page| Rc::ptr_eq(&page, &self.page_.Identity()))));
        if SameFrame(self.FocusedFrameIncludingRemote().as_ref(), frame.as_ref()) {
            return;
        }
        if let Some(old) = self.FocusedFrame().filter(|frame| frame.HasView()) {
            if let Some(document) = old.Document() {
                if document.FocusedElement().is_some() {
                    document.ClearFocusedElement(scope);
                }
            }
        }
        let new_local = frame.as_ref().filter(|frame| frame.IsLocalFrame());
        if let Some(new) = new_local.filter(|frame| frame.HasView()) {
            if let Some(document) = new.Document() {
                if let Some(element) = document.FocusedElement() {
                    DispatchFocusEvent(&document, &element, scope);
                }
            }
        }
        if new_local.is_some_and(|frame| !frame.HasView()) {
            return;
        }
        self.SetFocusedFrameWithScope(frame, notify_embedder, scope);
    }

    // Source: focus_controller.h:71; focus_controller.cc:1886-1891
    pub fn FocusedFrame(&self) -> Option<FocusFrameRef> {
        self.FocusedFrameIncludingRemote()
            .filter(|frame| frame.IsLocalFrame())
    }
    // Source: focus_controller.h:72-74
    pub fn FocusedFrameIncludingRemote(&self) -> Option<FocusFrameRef> {
        self.focused_frame_.borrow().clone()
    }
    // Source: focus_controller.h:75; focus_controller.cc:1893-1909
    pub fn FocusedOrMainFrame(&self) -> Option<FocusFrameRef> {
        if let Some(frame) = self.FocusedFrame() {
            return Some(frame);
        }
        let main = self.page_.MainFrame()?;
        let mut frame = Some(main.Top());
        while let Some(current) = frame {
            if current.IsLocalFrame() {
                return Some(current);
            }
            frame = current.TraverseNext();
        }
        Some(main)
    }
    // Source: focus_controller.h:77-78; focus_controller.cc:1911-1914
    pub fn FrameDetached(&self, frame: &FocusFrameRef) {
        if SameFrame(Some(frame), self.FocusedFrameIncludingRemote().as_ref()) {
            self.SetFocusedFrame(None, true);
        }
    }
    // Source: focus_controller.h:80-84; focus_controller.cc:1916-1926
    pub fn FocusedFrameOwnerElement(
        &self,
        current_frame: &FocusFrameRef,
    ) -> Option<FocusElementRef> {
        let mut frame = self.FocusedFrameIncludingRemote();
        while let Some(current) = frame {
            let parent = current.Parent();
            if SameFrame(parent.as_ref(), Some(current_frame)) {
                return current.Owner();
            }
            frame = parent;
        }
        None
    }
    // Source: focus_controller.h:86-88; focus_controller.cc:1928-1951
    pub fn IsDocumentFocused(&self, document: &FocusDocumentRef) -> bool {
        self.IsDocumentFocusedWithPageState(document, self.IsActive(), self.IsFocused())
    }
    /// Embeddings with an external native view supply its current activation
    /// and focus values at this same source query; frame identity remains here.
    pub fn IsDocumentFocusedWithPageState(
        &self,
        document: &FocusDocumentRef,
        page_active: bool,
        page_focused: bool,
    ) -> bool {
        if !page_active {
            return false;
        }
        let Some(frame) = self.FocusedFrameIncludingRemote() else {
            return false;
        };
        if let Some(owner) = frame
            .Owner()
            .filter(|owner| owner.IsFrameOwnerElement() && owner.IsFencedFrameElement())
        {
            if SameElement(Some(&owner), document.ActiveElement().as_ref()) {
                return owner
                    .Document()
                    .Frame()
                    .is_some_and(|frame| frame.IsDescendantOf(document.Frame().as_ref()));
            }
        }
        if !page_focused {
            return false;
        }
        frame.IsDescendantOf(document.Frame().as_ref())
    }

    // Source: focus_controller.h:151; focus_controller.cc:1953-1976
    fn FocusHasChanged(&self, scope: &FocusDispatchScope<'_>) {
        let focused = self.IsFocused();
        if !focused {
            if let Some(frame) = self
                .FocusedOrMainFrame()
                .filter(|frame| frame.IsLocalFrame())
            {
                frame.StopAutoscrollWithScope(scope);
            }
        }
        if self.FocusedFrameIncludingRemote().is_none() && focused {
            self.SetFocusedFrame(self.page_.MainFrame(), true);
        }
        if let Some(frame) = self.FocusedFrame().filter(|frame| frame.HasView()) {
            frame.SetSelectionFrameIsFocused(focused);
            let document = frame
                .Document()
                .expect("local frame with view has document");
            DispatchEventsOnWindowAndFocusedElement(&document, focused, scope);
        }
        self.NotifyFocusChangedObservers();
    }
    // Source: focus_controller.h:124; focus_controller.cc:1978-1992
    pub fn SetFocused(&self, focused: bool) {
        self.SetFocusedWithScope(focused, &FocusDispatchScope::default());
    }
    pub fn SetFocusedWithScope(&self, focused: bool, scope: &FocusDispatchScope<'_>) {
        debug_assert!(!focused || self.is_active_.get());
        if self.is_focused_.get() == focused {
            return;
        }
        self.is_focused_.set(focused);
        if !self.is_emulating_focus_.get() {
            self.FocusHasChanged(scope);
        }
        if !self.is_focused_.get() && self.page_.IsMainFrameFencedFrameRoot() {
            self.SetFocusedFrame(None, true);
        }
    }
    // Source: focus_controller.h:125
    pub fn IsFocused(&self) -> bool {
        self.is_focused_.get() || self.is_emulating_focus_.get()
    }
    // Source: focus_controller.h:127; focus_controller.cc:1994-2012
    pub fn SetFocusEmulationEnabled(&self, emulate_focus: bool) {
        if emulate_focus == self.is_emulating_focus_.get() {
            return;
        }
        let active = self.IsActive();
        let focused = self.IsFocused();
        self.is_emulating_focus_.set(emulate_focus);
        if self
            .page_
            .MainFrame()
            .is_none_or(|frame| !frame.IsLocalFrame())
        {
            return;
        }
        if active != self.IsActive() {
            self.ActiveHasChanged();
        }
        if focused != self.IsFocused() {
            self.FocusHasChanged(&FocusDispatchScope::default());
        }
    }
    // Source: focus_controller.h:129; focus_controller.cc:2014-2023
    pub fn UpdateFocusOnNavigationCommit(&self, frame: FocusFrameRef, was_focused: bool) {
        if was_focused {
            self.SetFocusedFrame(Some(frame), true);
            return;
        }
        if self.is_emulating_focus_.get() && frame.IsOutermostMainFrame() {
            self.SetFocusedFrame(Some(frame), true);
        }
    }
}

// Source: focus_controller.cc:2368-2371
fn RelinquishesEditingFocus(element: &FocusElementRef) -> bool {
    debug_assert!(element.IsEditable());
    element.Document().Frame().is_some() && element.HasRootEditableElement()
}

impl FocusController {
    // Source: focus_controller.h:117-119; focus_controller.cc:2373-2379
    pub fn SetFocusedElement(
        &self,
        element: Option<FocusElementRef>,
        frame: Option<FocusFrameRef>,
    ) -> bool {
        let params = FocusParams {
            selection_behavior: SelectionBehaviorOnFocus::kNone,
            ..FocusParams::default()
        };
        self.SetFocusedElementWithParams(element, frame, &params)
    }

    pub fn SetFocusedElementWithScope(
        &self,
        element: Option<FocusElementRef>,
        frame: Option<FocusFrameRef>,
        scope: &FocusDispatchScope<'_>,
    ) -> bool {
        let params = FocusParams {
            selection_behavior: SelectionBehaviorOnFocus::kNone,
            ..FocusParams::default()
        };
        self.SetFocusedElementWithParamsAndScope(element, frame, &params, scope)
    }
    // Source: focus_controller.h:116; focus_controller.cc:2381-2429
    pub fn SetFocusedElementWithParams(
        &self,
        element: Option<FocusElementRef>,
        frame: Option<FocusFrameRef>,
        params: &FocusParams,
    ) -> bool {
        self.SetFocusedElementWithParamsAndScope(
            element,
            frame,
            params,
            &FocusDispatchScope::default(),
        )
    }

    pub fn SetFocusedElementWithParamsAndScope(
        &self,
        element: Option<FocusElementRef>,
        frame: Option<FocusFrameRef>,
        params: &FocusParams,
        scope: &FocusDispatchScope<'_>,
    ) -> bool {
        let old_frame = self.FocusedFrame();
        let old_document = old_frame.as_ref().and_then(|frame| frame.Document());
        let old_element = old_document
            .as_ref()
            .and_then(|document| document.FocusedElement());
        if element.is_some() && SameElement(old_element.as_ref(), element.as_ref()) {
            return true;
        }
        if let Some(old) = &old_element {
            if old.IsRootEditableElement() && !RelinquishesEditingFocus(old) {
                return false;
            }
        }
        if let Some(old) = old_frame {
            old.WillChangeInputMethodFocus();
        }
        let new_document = if let Some(element) = &element {
            Some(element.Document())
        } else {
            frame
                .as_ref()
                .filter(|frame| frame.IsLocalFrame())
                .and_then(|frame| frame.Document())
        };
        if let Some(new_document) = &new_document {
            if SameDocument(old_document.as_ref(), Some(new_document))
                && SameElement(new_document.FocusedElement().as_ref(), element.as_ref())
            {
                return true;
            }
        }
        if let Some(old) = &old_document {
            if !SameDocument(Some(old), new_document.as_ref()) {
                old.ClearFocusedElement(scope);
            }
        }
        if frame
            .as_ref()
            .is_some_and(|frame| frame.PageIdentity().is_none())
        {
            self.SetFocusedFrameWithScope(None, true, scope);
            return false;
        }
        self.SetFocusedFrameWithScope(frame, true, scope);
        if let Some(document) = new_document {
            if !document.SetFocusedElement(element, params, scope) {
                return false;
            }
        }
        true
    }

    // Source: focus_controller.h:150; focus_controller.cc:2431-2445
    fn ActiveHasChanged(&self) {
        if let Some(frame) = self
            .FocusedOrMainFrame()
            .filter(|frame| frame.IsLocalFrame())
        {
            let document = frame.LocalFrameRootDocument();
            if !document.IsActive() {
                return;
            }
            if document.HasView() {
                (self.emit_)(FocusEffect::InvalidateAllCustomScrollbarsOnActiveChanged {
                    document,
                });
            }
            frame.SelectionPageActivationChanged();
        }
    }
    // Source: focus_controller.h:121; focus_controller.cc:2447-2454
    pub fn SetActive(&self, active: bool) {
        if self.is_active_.get() == active {
            return;
        }
        self.is_active_.set(active);
        if !self.is_emulating_focus_.get() {
            self.ActiveHasChanged();
        }
    }
    // Source: focus_controller.h:122
    pub fn IsActive(&self) -> bool {
        self.is_active_.get() || self.is_emulating_focus_.get()
    }
    // Source: focus_controller.h:131; focus_controller.cc:2456-2461
    pub fn RegisterFocusChangedObserver(&self, observer: Rc<dyn FocusChangedObserver>) {
        let key = Rc::as_ptr(&observer) as *const () as usize;
        let mut observers = self.focus_changed_observers_.borrow_mut();
        debug_assert!(!observers
            .get(&key)
            .is_some_and(|observer| observer.strong_count() > 0));
        observers.insert(key, Rc::downgrade(&observer));
    }
    // Source: focus_controller.h:148; focus_controller.cc:2463-2471
    fn NotifyFocusChangedObservers(&self) {
        let observers = self.focus_changed_observers_.borrow().clone();
        for observer in observers.values() {
            if let Some(observer) = observer.upgrade() {
                observer.FocusedFrameChanged();
            }
        }
    }
    // Source: focus_controller.h:135; focus_controller.cc:2489-2493
    // Rc/Weak field ownership and Drop replace tracing; no live tree is copied.
}

// Source: focus_controller.cc:2044-2068,2100-2274. The scope owner performs
// source traversal; Page only supplies the original DOM/Layout/host values.
impl FocusController {
    pub fn AdvanceFocus(
        &self,
        direction: FocusType,
        initial_focus: bool,
        source_capabilities: Option<
            interaction_dispatch::core::events::ui_event::InputDeviceCapabilitiesRef,
        >,
    ) -> bool {
        self.AdvanceFocusWithScope(
            direction,
            initial_focus,
            source_capabilities,
            &FocusDispatchScope::default(),
        )
    }

    pub fn AdvanceFocusWithScope(
        &self,
        direction: FocusType,
        initial_focus: bool,
        source_capabilities: Option<
            interaction_dispatch::core::events::ui_event::InputDeviceCapabilitiesRef,
        >,
        scope: &FocusDispatchScope<'_>,
    ) -> bool {
        assert!(matches!(
            direction,
            FocusType::kForward | FocusType::kBackward
        ));
        let Some(frame) = self
            .FocusedOrMainFrame()
            .filter(|frame| frame.IsLocalFrame())
        else {
            return false;
        };
        let Some(document) = frame.Document() else {
            return false;
        };
        let current = if initial_focus {
            None
        } else {
            document.SequentialFocusNavigationStartingPoint(direction)
        };
        document.UpdateStyleAndLayoutForFocus();
        let tree = document.FocusNavigation();
        if let (Some(current), Some(focused)) = (current, document.FocusedElement()) {
            if tree
                .NodeForElement(&focused)
                .and_then(|node| tree.ScrollMarkerOrigin(node))
                == Some(current)
                && tree.IsKeyboardFocusable(current)
            {
                self.SetFocusedFrameWithScope(document.Frame(), true, scope);
                if let Some(element) = tree.Element(current) {
                    element.Focus(
                        &FocusParams {
                            selection_behavior: SelectionBehaviorOnFocus::kReset,
                            r#type: direction,
                            source_capabilities,
                            focus_trigger:
                                crate::core::dom::focus_params::FocusTrigger::kUserGesture,
                            ..FocusParams::default()
                        },
                        scope,
                    );
                    return true;
                }
            }
        }
        let current_in_tree_scope = current.filter(|&current| {
            let root = tree.TreeScopeRoot(current);
            root == tree.Root() || tree.ShadowHost(root).is_some()
        });
        let navigation = if let Some(current) = current_in_tree_scope {
            super::focus_navigation::ScopedFocusNavigation::ForElement(tree, current)
        } else {
            super::focus_navigation::ScopedFocusNavigation::ForDocument(tree)
        };
        let mut element = navigation.FindAcrossScopes(direction);
        if element.is_none() {
            let local_root = frame.LocalFrameRoot();
            if !SameFrame(Some(&local_root), Some(&frame.Top())) {
                document.ClearFocusedElement(scope);
                document.ClearSequentialFocusNavigationStartingPoint();
                self.SetFocusedFrameWithScope(None, true, scope);
                if let Some(remote) = local_root.Parent() {
                    remote.AdvanceFocusFromRemote(direction, &local_root);
                    return true;
                }
                return false;
            }
            if (!initial_focus || self.page_.IsMainFrameFencedFrameRoot())
                && self.page_.CanTakeFocus(direction)
            {
                document.ClearFocusedElement(scope);
                document.ClearSequentialFocusNavigationStartingPoint();
                self.SetFocusedFrameWithScope(None, true, scope);
                self.page_.TakeFocus(direction);
                return true;
            }
            let Some(main) = self.page_.MainFrame().filter(|frame| frame.IsLocalFrame()) else {
                return false;
            };
            let Some(main_document) = main.Document() else {
                return false;
            };
            element = super::focus_navigation::ScopedFocusNavigation::ForDocument(
                main_document.FocusNavigation(),
            )
            .FindAcrossScopes(direction);
        }
        let Some(element) = element else {
            return false;
        };
        if SameElement(Some(&element), document.FocusedElement().as_ref()) {
            if !SameFrame(self.FocusedFrame().as_ref(), document.Frame().as_ref()) {
                self.SetFocusedFrameWithScope(document.Frame(), true, scope);
                DispatchFocusEvent(&document, &element, scope);
            }
            return true;
        }
        let remote_frame = element
            .ContentFrame()
            .is_some_and(|frame| !frame.IsLocalFrame());
        if element.IsFrameOwnerElement()
            && (remote_frame || !element.IsPlugin() || !element.IsKeyboardFocusable())
        {
            if element.ContentFrame().is_none() {
                return false;
            }
            document.ClearFocusedElement(scope);
            if let Some(remote) = element.ContentFrame().filter(|frame| !frame.IsLocalFrame()) {
                remote.AdvanceFocusFromRemote(direction, &frame);
            } else {
                self.SetFocusedFrameWithScope(element.ContentFrame(), true, scope);
            }
            return true;
        }
        let new_document = element.Document();
        if !SameDocument(Some(&new_document), Some(&document)) {
            document.ClearFocusedElement(scope);
            document.ClearSequentialFocusNavigationStartingPoint();
        }
        self.SetFocusedFrameWithScope(new_document.Frame(), true, scope);
        let params = FocusParams {
            selection_behavior: SelectionBehaviorOnFocus::kReset,
            r#type: direction,
            source_capabilities,
            focus_trigger: crate::core::dom::focus_params::FocusTrigger::kUserGesture,
            ..FocusParams::default()
        };
        element.Focus(&params, scope);
        true
    }
}
