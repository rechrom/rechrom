/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2001 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2004, 2005, 2006, 2007, 2010 Apple Inc. All rights reserved.
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
// Source: third_party/blink/renderer/core/html/forms/html_label_element.h:43-47,66-79,83
// Source: third_party/blink/renderer/core/html/forms/html_label_element.cc:55-111,127-293

use crate::core::dom::element::SelectionBehaviorOnFocus;
use crate::core::dom::focus_params::{FocusParams, FocusTrigger};
use crate::core::html::focus_options::FocusOptions;
use crate::public::mojom::input::focus_type::FocusType;
use interaction_dispatch::core::dom::events::event::{EventRef, EventTargetRef};
use std::{cell::Cell, rc::Rc};

pub type LabelNodeRef = Rc<dyn LabelNode>;
pub trait LabelNode {
    fn Target(&self) -> EventTargetRef;
    fn IsLabelable(&self) -> bool;
    fn IsInteractiveContent(&self) -> bool;
    fn ParentOrShadowHostNode(&self) -> Option<LabelNodeRef>;
    // Outer None means no reference target; Some(None) is a non-HTMLElement
    // reference target rejected by source DynamicTo<HTMLElement>.
    fn ShadowReferenceTargetFor(&self) -> Option<Option<LabelNodeRef>>;
    fn IsShadowIncludingInclusiveAncestorOf(&self, node: &LabelNodeRef) -> bool;
    fn CanStartSelection(&self) -> bool;
    fn IsMouseFocusable(&self) -> bool;
    fn IsShadowHostWithDelegatesFocus(&self) -> bool;
    fn Focus(&self, params: FocusParams);
    fn DispatchSimulatedClick(&self, event: &EventRef);
    fn WillRespondToMouseClickEvents(&self) -> bool;
    fn AccessKeyAction(
        &self,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    );
    fn IsActive(&self) -> bool;
    fn SetActive(&self, value: bool);
    fn IsHovered(&self) -> bool;
    fn SetHovered(&self, value: bool);
}
/// Geometry/selection facts read after the source's input update at cc:223.
/// The assembly supplies this prepared view at that source query point.
pub struct PreparedLabelSelection {
    pub has_frame: bool,
    pub label_layout_selectable: bool,
    pub selection_is_range: bool,
    pub mouse_down_was_single_click_in_selection: bool,
}
pub struct LabelClickValues {
    pub has_position: bool,
    pub click_count: i32,
}
pub trait LabelEnvironment {
    fn Target(&self) -> EventTargetRef;
    fn ForAttribute(&self) -> Option<String>;
    fn IsInTreeScope(&self) -> bool;
    fn Descendants(&self) -> Box<dyn Iterator<Item = LabelNodeRef> + '_>;
    fn ElementById(&self, id: &str) -> Option<LabelNodeRef>;
    fn NodeForTarget(&self, target: &EventTargetRef) -> Option<LabelNodeRef>;
    fn EventPathNodes(&self, event: &EventRef) -> Box<dyn Iterator<Item = LabelNodeRef> + '_>;
    fn InteractiveContentCheckBeforeHandlerEnabled(&self) -> bool;
    fn PreparedSelection(&self) -> PreparedLabelSelection;
    fn MouseClickValues(&self, event: &EventRef) -> Option<LabelClickValues>;
    fn BaseDefaultEventHandler(&self, event: &EventRef);
    fn BaseWillRespondToMouseClickEvents(&self) -> bool;
    fn BaseAccessKeyAction(
        &self,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    );
    fn IsActive(&self) -> bool;
    fn SetActive(&self, active: bool);
    fn IsHovered(&self) -> bool;
    fn SetHovered(&self, hovered: bool);
}
/// Retained source state independent of the transient DOM/event projection.
#[derive(Default)]
pub struct LabelDefaultState {
    processing_click: Cell<bool>,
}

pub trait LabelDefaultOperations<N> {
    fn Control(&self) -> Option<N>;
    fn RawTarget(&self) -> Option<N>;
    fn ControlContains(&self, control: &N, target: &N) -> bool;
    fn TargetIsInteractiveContent(&self, target: &N) -> bool;
    fn PreparedSelection(&self) -> PreparedLabelSelection;
    fn MouseClickValues(&self) -> Option<LabelClickValues>;
    fn TargetCanStartSelection(&self, target: &N) -> bool;
    fn ControlIsMouseFocusableOrDelegatesFocus(&self, control: &N) -> bool;
    fn FocusControl(&self, control: &N, focus_type: FocusType);
    fn DispatchSimulatedClick(&self, control: &N);
}

pub struct HTMLLabelElement {
    environment: Rc<dyn LabelEnvironment>,
    default_state_: LabelDefaultState,
}
impl HTMLLabelElement {
    // Source: html_label_element.cc:55-56.
    pub fn new(environment: Rc<dyn LabelEnvironment>) -> Self {
        Self {
            environment,
            default_state_: LabelDefaultState::default(),
        }
    }
    // Source: html_label_element.cc:60-97.
    pub fn controlForBinding(&self) -> Option<LabelNodeRef> {
        let Some(control_id) = self.environment.ForAttribute() else {
            for element in self.environment.Descendants() {
                if element.IsLabelable() {
                    // TODO: cc:69-72 non-form-associated label UseCounter.
                    return Some(element);
                }
            }
            return None;
        };
        if !self.environment.IsInTreeScope() {
            return None;
        }
        let control = self.environment.ElementById(&control_id)?;
        if control.IsLabelable() {
            // TODO: cc:86-89 non-form-associated label UseCounter.
            Some(control)
        } else {
            None
        }
    }
    // Source: html_label_element.cc:99-111.
    pub fn Control(&self) -> Option<LabelNodeRef> {
        let control = self.controlForBinding()?;
        control.ShadowReferenceTargetFor().unwrap_or(Some(control))
    }
    // Source: html_label_element.cc:334-340.
    pub fn AccessKeyAction(
        &self,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    ) {
        if let Some(control) = self.Control() {
            control.AccessKeyAction(scope);
        } else {
            self.environment.BaseAccessKeyAction(scope);
        }
    }
    // Source: html_label_element.cc:127-153.
    pub fn SetActive(&self, active: bool) {
        if active != self.environment.IsActive() {
            self.environment.SetActive(active);
        }
        if let Some(control) = self.Control() {
            if control.IsActive() != self.environment.IsActive() {
                control.SetActive(self.environment.IsActive());
            }
        }
    }
    pub fn SetHovered(&self, hovered: bool) {
        if hovered != self.environment.IsHovered() {
            self.environment.SetHovered(hovered);
        }
        if let Some(control) = self.Control() {
            if control.IsHovered() != self.environment.IsHovered() {
                control.SetHovered(self.environment.IsHovered());
            }
        }
    }
    pub fn IsInteractiveContent(&self) -> bool {
        true
    }
    // Source: html_label_element.cc:155-167. Ancestors are the live DOM owner.
    pub fn IsInInteractiveContent(&self, node: Option<LabelNodeRef>) -> bool {
        let Some(mut node) = node else {
            return false;
        };
        // The environment's label target is projected into that same tree.
        let label = self
            .environment
            .NodeForTarget(&self.environment.Target())
            .expect("label DOM node");
        if !label.IsShadowIncludingInclusiveAncestorOf(&node) {
            return false;
        }
        while !Rc::ptr_eq(&node.Target(), &self.environment.Target()) {
            if node.IsInteractiveContent() {
                return true;
            }
            let Some(parent) = node.ParentOrShadowHostNode() else {
                return false;
            };
            node = parent;
        }
        false
    }
    // Source: html_label_element.cc:169-184. The immutable dispatch snapshot
    // supplies this path; no live-tree reconstruction crosses the boundary.
    pub fn IsInInteractiveContentEvent(&self, event: &EventRef) -> bool {
        Self::IsInteractiveContentInPath(
            self.environment.EventPathNodes(event),
            &self.environment.Target(),
            |node, target| Rc::ptr_eq(&node.Target(), target),
            |node| node.IsInteractiveContent(),
        )
    }
    // Source: html_label_element.cc:169-184. Iteration is over the canonical
    // immutable dispatch snapshot, including when identities are opaque ids.
    pub fn IsInteractiveContentInPath<N, L>(
        nodes: impl Iterator<Item = N>,
        label: &L,
        same: impl Fn(&N, &L) -> bool,
        interactive: impl Fn(&N) -> bool,
    ) -> bool {
        for node in nodes {
            if same(&node, label) {
                return false;
            }
            if interactive(&node) {
                return true;
            }
        }
        unreachable!("label must be in its event path")
    }
    // Source html_label_element.cc:268-270. Both the retained node projection
    // and the borrowed Page projection use the same focus transaction values.
    pub fn ControlFocusParams(focus_type: FocusType) -> FocusParams {
        FocusParams::new(
            SelectionBehaviorOnFocus::kRestore,
            focus_type,
            None,
            Rc::new(FocusOptions::default()),
            FocusTrigger::kScript,
        )
    }
    // Source: html_label_element.cc:186-293.
    pub fn DefaultEventHandler(&self, event: &EventRef) {
        self.DefaultEventHandlerInternal(event);
        self.environment.BaseDefaultEventHandler(event);
    }
    pub fn DefaultEventHandlerInternal(&self, event: &EventRef) {
        let (click, trusted) = {
            let event = event.borrow();
            (
                event.BaseEvent().r#type() == "click",
                event.BaseEvent().isTrusted(),
            )
        };
        if Self::HandleDefaultEventWithOperations(
            &self.default_state_,
            &SourceLabelDefaultOperations { label: self, event },
            click,
            trusted,
        ) {
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        }
    }
    // Source: html_label_element.cc:186-293. Borrowed native identities use
    // this same source transaction, including the synchronous reentry guard.
    pub fn HandleDefaultEventWithOperations<N>(
        state: &LabelDefaultState,
        operations: &dyn LabelDefaultOperations<N>,
        click: bool,
        trusted: bool,
    ) -> bool {
        if !click || state.processing_click.get() {
            return false;
        }
        let Some(control) = operations.Control() else {
            return false;
        };
        let target = operations.RawTarget();
        if let Some(target) = &target {
            if operations.ControlContains(&control, target)
                || operations.TargetIsInteractiveContent(target)
            {
                return false;
            }
        }
        let mut selected = false;
        let selection = operations.PreparedSelection();
        if let Some(mouse) = operations
            .MouseClickValues()
            .filter(|mouse| mouse.has_position)
        {
            if selection.has_frame
                && selection.label_layout_selectable
                && selection.selection_is_range
                && !selection.mouse_down_was_single_click_in_selection
                && target
                    .as_ref()
                    .is_some_and(|target| operations.TargetCanStartSelection(target))
            {
                selected = true;
                if mouse.click_count == 1 {
                    return false;
                }
            }
        }
        state.processing_click.set(true);
        if operations.ControlIsMouseFocusableOrDelegatesFocus(&control) && !selected {
            operations.FocusControl(
                &control,
                if trusted {
                    FocusType::kMouse
                } else {
                    FocusType::kScript
                },
            );
        }
        operations.DispatchSimulatedClick(&control);
        state.processing_click.set(false);
        true
    }
    pub fn HasActivationBehavior(&self) -> bool {
        true
    }
    pub fn WillRespondToMouseClickEvents(&self) -> bool {
        if self
            .Control()
            .is_some_and(|control| control.WillRespondToMouseClickEvents())
        {
            return true;
        }
        self.environment.BaseWillRespondToMouseClickEvents()
    }
}

struct SourceLabelDefaultOperations<'a> {
    label: &'a HTMLLabelElement,
    event: &'a EventRef,
}
impl LabelDefaultOperations<LabelNodeRef> for SourceLabelDefaultOperations<'_> {
    fn Control(&self) -> Option<LabelNodeRef> {
        self.label.Control()
    }
    fn RawTarget(&self) -> Option<LabelNodeRef> {
        self.event
            .borrow()
            .BaseEvent()
            .RawTarget()
            .and_then(|target| self.label.environment.NodeForTarget(&target))
    }
    fn ControlContains(&self, control: &LabelNodeRef, target: &LabelNodeRef) -> bool {
        control.IsShadowIncludingInclusiveAncestorOf(target)
    }
    fn TargetIsInteractiveContent(&self, target: &LabelNodeRef) -> bool {
        if self
            .label
            .environment
            .InteractiveContentCheckBeforeHandlerEnabled()
        {
            self.label.IsInInteractiveContentEvent(self.event)
        } else {
            self.label.IsInInteractiveContent(Some(target.clone()))
        }
    }
    fn PreparedSelection(&self) -> PreparedLabelSelection {
        self.label.environment.PreparedSelection()
    }
    fn MouseClickValues(&self) -> Option<LabelClickValues> {
        self.label.environment.MouseClickValues(self.event)
    }
    fn TargetCanStartSelection(&self, target: &LabelNodeRef) -> bool {
        target.CanStartSelection()
    }
    fn ControlIsMouseFocusableOrDelegatesFocus(&self, control: &LabelNodeRef) -> bool {
        control.IsMouseFocusable() || control.IsShadowHostWithDelegatesFocus()
    }
    fn FocusControl(&self, control: &LabelNodeRef, focus_type: FocusType) {
        control.Focus(HTMLLabelElement::ControlFocusParams(focus_type));
    }
    fn DispatchSimulatedClick(&self, control: &LabelNodeRef) {
        control.DispatchSimulatedClick(self.event);
    }
}
