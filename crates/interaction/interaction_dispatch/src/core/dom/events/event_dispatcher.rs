/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 1999 Antti Koivisto (koivisto@kde.org)
 * Copyright (C) 2001 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2004, 2005, 2006, 2007, 2008, 2009, 2010, 2011 Apple Inc.
 * All rights reserved.
 * Copyright (C) 2008 Nokia Corporation and/or its subsidiary(-ies)
 * Copyright (C) 2009 Torch Mobile Inc. All rights reserved.
 * Copyright (C) 2011 Google Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or modify it
 * under the terms of the GNU Library General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or (at your
 * option) any later version.
 *
 * This library is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE. See the GNU Library General Public
 * License for more details. You should have received a copy of the GNU
 * Library General Public License along with this library; see COPYING.LIB.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/event_dispatcher.h:1-96
// Source: third_party/blink/renderer/core/dom/events/event_dispatcher.cc:1-496
// Rust modules replace includes, namespace and STACK_ALLOCATED. Retained
// handles preserve node/event identity during synchronous listener reentry.

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::web_input_event::{
    Type as WebInputEventType, WebInputEvent,
};
use interaction_input::public::common::input::web_keyboard_event::WebKeyboardEvent;

use super::event::{
    EventDispatcher as EventDispatcherInterface, EventRef, EventTargetRef, PhaseType,
};
use super::event_dispatch_forbidden_scope::EventDispatchForbiddenScope;
use super::event_dispatch_result::DispatchEventResult;
use super::event_path::{EventPath, LocalEventDispatch};
use super::event_target::EventTarget;
pub use super::simulated_click_options::SimulatedClickCreationScope;
use crate::core::events::simulated_event_util::RequiredSimulatedEventInput;

// Source: event_dispatcher.h:38-48
// Derived activation state keeps its concrete owner until post-processing.
// Rust destruction replaces the source virtual GC tracing/destructor.
pub trait EventDispatchHandlingState: Any {}
pub type EventDispatchHandlingStateRef = Rc<dyn EventDispatchHandlingState>;

// Source: event_dispatcher.h:50
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventDispatchContinuation {
    kContinueDispatching,
    kDoneDispatching,
}

pub type DispatchPathRef = Rc<RefCell<EventPath>>;

// Source: event_dispatcher.cc:82-93,253,380
// The adapter initializes/reinitializes Event::GetEventPath using its existing
// factory and returns that very same concrete owner. It applies the path's
// TakeInitializationEffect at this initialization point. Neither operation
// traverses the live DOM or constructs a second path.
pub trait DispatchPathAccess {
    fn InitEventPath(&self, node: &EventTargetRef, event: &EventRef) -> DispatchPathRef;
    fn EventTargetRespectingTargetRules(&self, node: &EventTargetRef) -> EventTargetRef;
}

// Source: event_dispatcher.cc:107,115,132-148,228-249,284-287,413-457
// interaction_default supplies these synchronous source activation/default
// operations. Shared node handles carry identity without exposing DOM storage.
pub trait SimulatedNodeDefaultActions {
    fn IsDisabledFormControl(&self, node: &EventTargetRef) -> bool;
    fn IsElement(&self, node: &EventTargetRef) -> bool;
    fn SetActive(&self, node: &EventTargetRef, active: bool);
}

pub trait NodeDefaultActions {
    fn IsHTMLSelectElement(&self, node: &EventTargetRef) -> bool;
    fn HasActivationBehavior(&self, node: &EventTargetRef) -> bool;
    fn LegacyPreActivationBehavior(
        &self,
        node: &EventTargetRef,
        event: &EventRef,
    ) -> Option<EventDispatchHandlingStateRef>;
    fn RunActivationBehavior(
        &self,
        node: &EventTargetRef,
        event: &EventRef,
        state: Option<&EventDispatchHandlingStateRef>,
    );
    fn DefaultEventHandler(
        &self,
        node: &EventTargetRef,
        event: &EventRef,
        invocation: Option<&dyn crate::bindings::script_invocation::ScriptInvocation>,
    );
}

// Source: event_dispatcher.cc:84,468-469
// The captured view is retained for this dispatch transaction only.
pub trait DispatchFrameView {
    fn SyncSelectionForced(&self);
}

// Source: event_dispatcher.cc:401-402
/// Neutral AX selection intent, produced by BlinkAXEventIntent's source owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AXSelectionCommand {
    MoveSelection,
    ExtendSelection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AXSelectionMoveDirection {
    Backward,
    Forward,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AXSelectionTextBoundary {
    Character,
    WordStart,
    WordEnd,
    SentenceStart,
    SentenceEnd,
    LineStart,
    LineEnd,
    ParagraphStart,
    ParagraphEnd,
    WebPage,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AXSelectionIntent {
    pub command: AXSelectionCommand,
    pub direction: AXSelectionMoveDirection,
    pub boundary: AXSelectionTextBoundary,
}
/// The actual AX cache retains the source active-intents counted set until Drop.
pub trait DispatchAccessibilityIntentScope {}
#[derive(Clone, Copy, Debug)]
pub enum AXPositionAnchorKind {
    OffsetInAnchor,
    BeforeAnchor,
    AfterAnchor,
    AfterChildren,
}
#[derive(Clone)]
pub struct AXTextSelectionEndpoint {
    pub node: EventTargetRef,
    pub offset: usize,
    pub anchor: AXPositionAnchorKind,
}
pub trait DispatchAccessibilityCache {
    /// AXObjectCache load notifications; the caller only reads ExistingCache.
    fn HandleLoadStart(&self);
    fn HandleLoadComplete(&self);
    fn HandleDeletionOrInsertionInTextField(
        &self,
        anchor: AXTextSelectionEndpoint,
        focus: AXTextSelectionEndpoint,
        is_deletion: bool,
    );
    /// ScopedBlinkAXEventIntent.cc:15-31,49-64. No cache is created here.
    fn BeginSelectionIntent(
        &self,
        intent: AXSelectionIntent,
    ) -> Box<dyn DispatchAccessibilityIntentScope>;
    // Document::RefreshAccessibilityTree / HTMLElement popover invoker dirtiness.
    fn MarkDocumentDirty(&self);
    fn MarkElementDirty(&self, node: EventTargetRef);
    fn HandleClicked(&self, target_node: Option<EventTargetRef>);
    /// AXObjectCache::HandleScrolledToAnchor, used by element/text anchors.
    fn HandleScrolledToAnchor(&self, target_node: EventTargetRef);
    /// Source frame_selection.cc:1161-1170 and editor.cc:420-425.
    fn SelectionChanged(&self, target_node: EventTargetRef);
    fn HandleEditableTextContentChanged(&self, anchor_node: Option<EventTargetRef>);
    /// Source DocumentMarkerController::InvalidatePaintForNode, cc:170-174.
    fn HandleTextMarkerDataAdded(&self, start: &EventTargetRef, end: &EventTargetRef);
}

// Source: event_dispatcher.cc:76,84,161,300-301,401-402,433-434,474
// These individual document facts/effects keep document and rendering
// ownership outside dispatch. They carry no propagation or activation policy.
pub trait DispatchDocument {
    /// EventDispatcher.cc:195-198. The returned stack scope performs source
    /// processing-end bookkeeping after listener/default action reentry.
    fn BeginUIEventTiming(&self, event: &EventRef) -> Box<dyn std::any::Any>;
    fn IsMainSequence(&self) -> bool;
    fn View(&self, node: &EventTargetRef) -> Option<Rc<dyn DispatchFrameView>>;
    fn HasCaptureListener(&self, node: &EventTargetRef) -> bool;
    fn SkipEventCaptureEnabled(&self) -> bool;
    fn WideViewportQuirkEnabled(&self, node: &EventTargetRef) -> Option<bool>;
    fn ExistingAXObjectCache(
        &self,
        node: &EventTargetRef,
    ) -> Option<Rc<dyn DispatchAccessibilityCache>>;
    fn ClearPopoverPickerPointerdown(&self, node: &EventTargetRef);
}

// Source: event_dispatcher.cc:92
// ScopedEventQueue retains the original event owner until its source flush.
pub trait ScopedEventQueue {
    fn EnqueueEvent(&self, event: EventRef);
}

// Source: event_dispatcher.cc:120-154,161-175
// Concrete translated event constructors consume the raw keyboard value or
// simulated-event inputs here; the factory does not choose dispatch ordering.
pub trait SimulatedEventFactory {
    fn CreateEvent(
        &self,
        event_type: &'static str,
        node: &EventTargetRef,
        underlying_event: Option<&EventRef>,
        creation_scope: SimulatedClickCreationScope,
    ) -> Result<EventRef, RequiredSimulatedEventInput>;
}
pub trait SimulatedKeyboardEventFactory {
    fn CreateKeyboardEvent(
        &self,
        raw_event: WebKeyboardEvent,
        input_element: &EventTargetRef,
        cancelable: bool,
    ) -> EventRef;
    fn Now(&self) -> TimeTicks;
}

// Source: event_dispatcher.cc:79; event.cc:401-403
// EventInterface's virtual override is invoked through the shared EventRef,
// with no mutable RefCell guard held while Dispatch calls back into the Event.
// Mouse/Pointer overrides adjust this same path before entering Dispatch.
pub trait EventDispatchHook {
    fn DispatchEvent(&self, dispatcher: &mut EventDispatcher<'_>) -> DispatchEventResult;
}

pub struct BaseEventDispatchHook;
impl EventDispatchHook for BaseEventDispatchHook {
    fn DispatchEvent(&self, dispatcher: &mut EventDispatcher<'_>) -> DispatchEventResult {
        dispatcher.Dispatch()
    }
}

/// Optional embedding instrumentation at the source dispatch cleanup boundary.
/// The event is the canonical owner; no Event borrow may cross this callback.
/// Source propagation flags are observed before PostProcess clears them, while
/// default actions and the ordinary final target cleanup retain their order.
pub trait EventDispatchObserver {
    fn DidFinishPropagation(&self, event: &EventRef);
    fn DidDispatch(&self, event: &EventRef);
}

// Distinct, small ports mirror the source owners called by the dispatcher.
pub struct EventDispatcherContext<'a> {
    pub paths: &'a dyn DispatchPathAccess,
    pub defaults: &'a dyn NodeDefaultActions,
    pub document: &'a dyn DispatchDocument,
    pub local_events: &'a dyn LocalEventDispatch,
    // Ordinary dispatch needs neither the scoped queue nor simulated-event
    // host facts. Their separate source entry points require these extensions.
    pub scoped_queue: Option<&'a dyn ScopedEventQueue>,
    pub simulated_events: Option<&'a dyn SimulatedEventFactory>,
    pub simulated_keyboard_events: Option<&'a dyn SimulatedKeyboardEventFactory>,
    pub simulated_defaults: Option<&'a dyn SimulatedNodeDefaultActions>,
    pub event_dispatch: &'a dyn EventDispatchHook,
    pub observer: Option<&'a dyn EventDispatchObserver>,
}

// Source: event_dispatcher.h:52-91
pub struct EventDispatcher<'a> {
    node_: EventTargetRef,
    event_: EventRef,
    view_: Option<Rc<dyn DispatchFrameView>>,
    path_: DispatchPathRef,
    context_: &'a EventDispatcherContext<'a>,
    #[cfg(debug_assertions)]
    event_dispatched_: bool,
}

// Source: event_dispatcher.cc:99-105
// Entries retain the nodes until this synchronous simulated-click call ends.
thread_local! {
    static NODES_DISPATCHING_SIMULATED_CLICKS: RefCell<HashMap<usize, EventTargetRef>> =
        RefCell::new(HashMap::new());
}

fn NodeIdentity(node: &EventTargetRef) -> usize {
    Rc::as_ptr(node) as *const () as usize
}

struct SimulatedClickGuard(EventTargetRef);
impl Drop for SimulatedClickGuard {
    // Source: event_dispatcher.cc:156
    fn drop(&mut self) {
        NODES_DISPATCHING_SIMULATED_CLICKS.with(|nodes| {
            let mut nodes = nodes.borrow_mut();
            nodes
                .remove(&NodeIdentity(&self.0))
                .expect("simulated click node remains registered");
        });
    }
}

impl<'a> EventDispatcher<'a> {
    // Source: event_dispatcher.h:56; event_dispatcher.cc:72-80
    pub fn DispatchEvent(
        node: EventTargetRef,
        event: EventRef,
        context: &'a EventDispatcherContext<'a>,
    ) -> DispatchEventResult {
        // TODO: cc:73-74 disabled-by-default blink.debug tracing.
        debug_assert!(!EventDispatchForbiddenScope::IsEventDispatchForbidden(
            context.document.IsMainSequence(),
        ));
        let mut dispatcher = Self::new(node, event.clone(), context);
        let result = context.event_dispatch.DispatchEvent(&mut dispatcher);
        if let Some(observer) = context.observer {
            observer.DidDispatch(&event);
        }
        context.local_events.DidFinishEvent(&event);
        result
    }

    // Source: event_dispatcher.h:69; event_dispatcher.cc:82-86
    pub fn new(
        node: EventTargetRef,
        event: EventRef,
        context: &'a EventDispatcherContext<'a>,
    ) -> Self {
        let view = context.document.View(&node);
        let path = context.paths.InitEventPath(&node, &event);
        Self {
            node_: node,
            event_: event,
            view_: view,
            path_: path,
            context_: context,
            #[cfg(debug_assertions)]
            event_dispatched_: false,
        }
    }

    // Source: event_dispatcher.h:57; event_dispatcher.cc:88-93
    pub fn DispatchScopedEvent(
        node: &EventTargetRef,
        event: EventRef,
        context: &EventDispatcherContext<'_>,
    ) {
        Self::DispatchScopedEventWithQueue(
            node,
            event,
            context.paths,
            context
                .scoped_queue
                .expect("scoped dispatch requires its queue owner"),
        );
    }

    // Source: event_dispatcher.cc:88-93. Scoped emission only needs retained
    // target rules and its queue; no default/listener invocation context exists
    // until that queue actually dispatches the same Event owner.
    pub fn DispatchScopedEventWithQueue(
        node: &EventTargetRef,
        event: EventRef,
        paths: &dyn DispatchPathAccess,
        queue: &dyn ScopedEventQueue,
    ) {
        let target = paths.EventTargetRespectingTargetRules(node);
        event.borrow_mut().SetTarget(Some(target));
        queue.EnqueueEvent(event);
    }

    // Source: event_dispatcher.h:59-61; event_dispatcher.cc:95-157
    pub fn DispatchSimulatedClick(
        node: EventTargetRef,
        underlying_event: Option<&EventRef>,
        creation_scope: SimulatedClickCreationScope,
        context: &'a EventDispatcherContext<'a>,
    ) -> Result<(), RequiredSimulatedEventInput> {
        Self::DispatchSimulatedClickWithDispatch(
            node.clone(),
            underlying_event,
            creation_scope,
            context
                .simulated_defaults
                .expect("simulated click requires its node-default owner"),
            context
                .simulated_events
                .expect("simulated click requires its event factory"),
            &mut |event| Self::DispatchEvent(node.clone(), event, context),
        )
    }

    // Source: event_dispatcher.cc:95-157. Per-event embedding contexts may
    // borrow the current binding invocation. This callback invokes the same
    // source DispatchEvent once; it never chooses event order or activation.
    pub fn DispatchSimulatedClickWithDispatch(
        node: EventTargetRef,
        underlying_event: Option<&EventRef>,
        creation_scope: SimulatedClickCreationScope,
        defaults: &dyn SimulatedNodeDefaultActions,
        factory: &dyn SimulatedEventFactory,
        dispatch_event: &mut dyn FnMut(EventRef) -> DispatchEventResult,
    ) -> Result<(), RequiredSimulatedEventInput> {
        if defaults.IsDisabledFormControl(&node) {
            return Ok(());
        }
        let already_dispatching = NODES_DISPATCHING_SIMULATED_CLICKS
            .with(|nodes| nodes.borrow().contains_key(&NodeIdentity(&node)));
        if already_dispatching {
            return Ok(());
        }
        NODES_DISPATCHING_SIMULATED_CLICKS
            .with(|nodes| nodes.borrow_mut().insert(NodeIdentity(&node), node.clone()));
        let _guard = SimulatedClickGuard(node.clone());
        let is_element = defaults.IsElement(&node);
        let mut dispatch =
            |event_type| -> Result<DispatchEventResult, RequiredSimulatedEventInput> {
                let event =
                    factory.CreateEvent(event_type, &node, underlying_event, creation_scope)?;
                Ok(dispatch_event(event))
            };
        if creation_scope == SimulatedClickCreationScope::kFromAccessibility {
            let dispatch_result = dispatch("pointerdown")?;
            let prevent_mouse_events =
                dispatch_result == DispatchEventResult::kCanceledByEventHandler;
            if !prevent_mouse_events {
                dispatch("mousedown")?;
            }
            if is_element {
                defaults.SetActive(&node, true);
            }
            // Embedding input failures must not leave source active state set.
            let release_result = (|| {
                dispatch("pointerup")?;
                if !prevent_mouse_events {
                    dispatch("mouseup")?;
                }
                Ok::<(), RequiredSimulatedEventInput>(())
            })();
            if release_result.is_err() && is_element {
                defaults.SetActive(&node, false);
            }
            release_result?;
        }
        if is_element {
            defaults.SetActive(&node, false);
        }
        dispatch("click")?;
        Ok(())
    }

    // Source: event_dispatcher.h:62; event_dispatcher.cc:159-178
    pub fn DispatchSimulatedEnterEvent(
        input_element: &EventTargetRef,
        context: &EventDispatcherContext<'_>,
    ) {
        for event_type in [
            WebInputEventType::kRawKeyDown,
            WebInputEventType::kChar,
            WebInputEventType::kKeyUp,
        ] {
            let mut enter = WebKeyboardEvent::new(
                event_type,
                WebInputEvent::kNoModifiers,
                context
                    .simulated_keyboard_events
                    .expect("simulated enter requires its keyboard-event factory")
                    .Now(),
            );
            // Source: ui/events/keycodes/dom/dom_key_data.inc:35 and
            // ui/events/keycodes/dom/dom_key.h:68-78,130-133,158-160.
            // DomKey::ENTER is TYPE_UNICODE | U+000D, not the raw code point.
            const ENTER_DOM_KEY: u32 = (1 << 22) | 0x000d;
            // Source: platform/keyboard_codes.h:56-57;
            // ui/events/keycodes/keyboard_codes_posix.h:50
            const VKEY_RETURN: i32 = 0x0d;
            enter.dom_key = ENTER_DOM_KEY;
            enter.dom_code = ENTER_DOM_KEY as i32;
            enter.native_key_code = VKEY_RETURN;
            enter.windows_key_code = VKEY_RETURN;
            enter.text[0] = VKEY_RETURN as u16;
            enter.unmodified_text[0] = VKEY_RETURN as u16;
            let event = context
                .simulated_keyboard_events
                .expect("simulated enter requires its keyboard-event factory")
                .CreateKeyboardEvent(enter, input_element, true);
            event.borrow_mut().BaseEventMut().SetTrusted(true);
            Self::DispatchScopedEvent(input_element, event, context);
        }
    }

    // Source: event_dispatcher.h:64; event_dispatcher.cc:180-275
    pub fn Dispatch(&mut self) -> DispatchEventResult {
        // TODO: cc:182-183 disabled-by-default blink.debug tracing.
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.event_dispatched_);
            self.event_dispatched_ = true;
        }
        if self.path_.borrow().IsEmpty() {
            return DispatchEventResult::kNotCanceled;
        }
        let _event_timing = self.context_.document.BeginUIEventTiming(&self.event_);
        // TODO: cc:200-203 trusted-change layout-shift tracking.
        self.path_.borrow().EnsureWindowEventContext();
        let is_click = {
            let event = self.event_.borrow();
            event.IsMouseEvent() && event.BaseEvent().r#type() == "click"
        };
        // TODO: cc:209-217 ad-stack debug check and ad-click UseCounter.
        let (is_activation_event, bubbles) = {
            let event = self.event_.borrow();
            (
                is_click || event.BaseEvent().r#type() == "textInput",
                event.BaseEvent().bubbles(),
            )
        };
        let mut activation_target =
            if is_activation_event && self.context_.defaults.HasActivationBehavior(&self.node_) {
                Some(self.node_.clone())
            } else {
                None
            };
        if is_activation_event && activation_target.is_none() && bubbles {
            let size = self.path_.borrow().size();
            let start = if self.node_.IsPseudoElement() { 0 } else { 1 };
            for index in start..size {
                let node = self.path_.borrow().at(index).GetNode();
                if self.context_.defaults.HasActivationBehavior(&node) {
                    activation_target = Some(node);
                    break;
                }
            }
        }
        self.SetOriginalTarget();
        debug_assert!(!EventDispatchForbiddenScope::IsEventDispatchForbidden(
            self.context_.document.IsMainSequence(),
        ));
        debug_assert!(self.event_.borrow().BaseEvent().RawTarget().is_some());
        // TODO: cc:258-260 DevTools EventDispatch timeline instrumentation.
        let mut pre_dispatch_event_handler_result = None;
        if self.DispatchEventLegacyPreActivationBehavior(
            activation_target.as_ref(),
            &mut pre_dispatch_event_handler_result,
        ) == EventDispatchContinuation::kContinueDispatching
            && self.DispatchEventAtCapturing() == EventDispatchContinuation::kContinueDispatching
        {
            self.DispatchEventAtBubbling();
        }
        self.DispatchEventPostProcess(
            activation_target.as_ref(),
            pre_dispatch_event_handler_result.as_ref(),
        );
        EventTarget::GetDispatchEventResult(self.event_.borrow().BaseEvent())
    }

    // Source: event_dispatcher.h:65-66
    pub fn GetNode(&self) -> EventTargetRef {
        self.node_.clone()
    }
    pub fn GetEvent(&self) -> EventRef {
        self.event_.clone()
    }
    pub fn GetEventPath(&self) -> DispatchPathRef {
        self.path_.clone()
    }

    // Source: event_dispatcher.h:71-80; event_dispatcher.cc:277-292
    fn DispatchEventLegacyPreActivationBehavior(
        &self,
        activation_target: Option<&EventTargetRef>,
        pre_dispatch_event_handler_result: &mut Option<EventDispatchHandlingStateRef>,
    ) -> EventDispatchContinuation {
        if let Some(target) = activation_target {
            *pre_dispatch_event_handler_result = self
                .context_
                .defaults
                .LegacyPreActivationBehavior(target, &self.event_);
        }
        if self.path_.borrow().IsEmpty() || self.event_.borrow().BaseEvent().PropagationStopped() {
            EventDispatchContinuation::kDoneDispatching
        } else {
            EventDispatchContinuation::kContinueDispatching
        }
    }

    // Source: event_dispatcher.h:81; event_dispatcher.cc:294-335
    fn DispatchEventAtCapturing(&self) -> EventDispatchContinuation {
        self.event_
            .borrow_mut()
            .BaseEventMut()
            .SetEventPhase(PhaseType::kCapturingPhase);
        if !self.context_.document.HasCaptureListener(&self.node_) {
            debug_assert!(self.context_.document.SkipEventCaptureEnabled());
            return EventDispatchContinuation::kContinueDispatching;
        }
        let window = self.path_.borrow().GetWindowEventContext().clone();
        if window.HandleLocalEvents(&self.event_, self.context_.local_events)
            && self.event_.borrow().BaseEvent().PropagationStopped()
        {
            return EventDispatchContinuation::kDoneDispatching;
        }
        let size = self.path_.borrow().size();
        for index in (0..size).rev() {
            let context = self.path_.borrow().at(index).clone();
            if context.CurrentTargetSameAsTarget() {
                {
                    let mut owner = self.event_.borrow_mut();
                    let event = owner.BaseEventMut();
                    event.SetEventPhase(PhaseType::kAtTarget);
                    event.SetFireOnlyCaptureListenersAtTarget(true);
                }
                context.HandleLocalEvents(&self.event_, self.context_.local_events);
                self.event_
                    .borrow_mut()
                    .BaseEventMut()
                    .SetFireOnlyCaptureListenersAtTarget(false);
            } else {
                self.event_
                    .borrow_mut()
                    .BaseEventMut()
                    .SetEventPhase(PhaseType::kCapturingPhase);
                context.HandleLocalEvents(&self.event_, self.context_.local_events);
            }
            if self.event_.borrow().BaseEvent().PropagationStopped() {
                return EventDispatchContinuation::kDoneDispatching;
            }
        }
        EventDispatchContinuation::kContinueDispatching
    }

    // Source: event_dispatcher.h:82; event_dispatcher.cc:337-375
    fn DispatchEventAtBubbling(&self) {
        let size = self.path_.borrow().size();
        for index in 0..size {
            let context = self.path_.borrow().at(index).clone();
            if context.CurrentTargetSameAsTarget() {
                {
                    let mut owner = self.event_.borrow_mut();
                    let event = owner.BaseEventMut();
                    event.SetEventPhase(PhaseType::kAtTarget);
                    event.SetFireOnlyNonCaptureListenersAtTarget(true);
                }
                context.HandleLocalEvents(&self.event_, self.context_.local_events);
                self.event_
                    .borrow_mut()
                    .BaseEventMut()
                    .SetFireOnlyNonCaptureListenersAtTarget(false);
            } else {
                let bubbles = {
                    let owner = self.event_.borrow();
                    let event = owner.BaseEvent();
                    event.bubbles() && !event.cancelBubble()
                };
                if !bubbles {
                    continue;
                }
                self.event_
                    .borrow_mut()
                    .BaseEventMut()
                    .SetEventPhase(PhaseType::kBubblingPhase);
                context.HandleLocalEvents(&self.event_, self.context_.local_events);
            }
            if self.event_.borrow().BaseEvent().PropagationStopped() {
                return;
            }
        }
        let bubbles = {
            let owner = self.event_.borrow();
            let event = owner.BaseEvent();
            event.bubbles() && !event.cancelBubble()
        };
        if bubbles {
            self.event_
                .borrow_mut()
                .BaseEventMut()
                .SetEventPhase(PhaseType::kBubblingPhase);
            let window = self.path_.borrow().GetWindowEventContext().clone();
            window.HandleLocalEvents(&self.event_, self.context_.local_events);
        }
    }

    // Source: event_dispatcher.h:83-84; event_dispatcher.cc:377-494
    fn DispatchEventPostProcess(
        &self,
        activation_target: Option<&EventTargetRef>,
        pre_dispatch_event_handler_result: Option<&EventDispatchHandlingStateRef>,
    ) {
        self.SetOriginalTarget();
        if let Some(observer) = self.context_.observer {
            observer.DidFinishPropagation(&self.event_);
        }
        {
            let mut owner = self.event_.borrow_mut();
            let event = owner.BaseEventMut();
            event.SetEventPhase(PhaseType::kNone);
            event.SetCurrentTarget(None);
            event.SetStopPropagation(false);
            event.SetStopImmediatePropagation(false);
        }
        let is_click = {
            let event = self.event_.borrow();
            event.IsMouseEvent() && event.BaseEvent().r#type() == "click"
        };
        if is_click {
            if let Some(cache) = self.context_.document.ExistingAXObjectCache(&self.node_) {
                let target_node = self
                    .event_
                    .borrow()
                    .BaseEvent()
                    .RawTarget()
                    .filter(|target| target.Kind() == super::event::EventTargetKind::Node);
                cache.HandleClicked(target_node);
            }
            if let Some(target) = activation_target {
                self.context_.defaults.RunActivationBehavior(
                    target,
                    &self.event_,
                    pre_dispatch_event_handler_result,
                );
            }
        }
        let mut is_trusted_or_click = self.event_.borrow().BaseEvent().isTrusted() || is_click;
        let is_mouse_down = {
            let owner = self.event_.borrow();
            owner.IsMouseEvent() && owner.BaseEvent().r#type() == "mousedown"
        };
        if !is_trusted_or_click
            && is_mouse_down
            && self.context_.defaults.IsHTMLSelectElement(&self.node_)
        {
            if let Some(enabled) = self.context_.document.WideViewportQuirkEnabled(&self.node_) {
                is_trusted_or_click = enabled;
            }
        }
        let allow_default = {
            let owner = self.event_.borrow();
            let event = owner.BaseEvent();
            !event.defaultPrevented() && !event.DefaultHandled() && is_trusted_or_click
        };
        if allow_default {
            self.context_
                .local_events
                .WithScriptInvocation(&mut |invocation| {
                    self.context_.defaults.DefaultEventHandler(
                        &self.node_,
                        &self.event_,
                        invocation,
                    );
                });
            let bubble_defaults = {
                let owner = self.event_.borrow();
                let event = owner.BaseEvent();
                !event.DefaultHandled() && !event.defaultPrevented() && event.bubbles()
            };
            if bubble_defaults {
                let size = self.path_.borrow().size();
                let start = if self.node_.IsPseudoElement() { 0 } else { 1 };
                for index in start..size {
                    let node = self.path_.borrow().at(index).GetNode();
                    self.context_
                        .local_events
                        .WithScriptInvocation(&mut |invocation| {
                            self.context_.defaults.DefaultEventHandler(
                                &node,
                                &self.event_,
                                invocation,
                            );
                        });
                    let owner = self.event_.borrow();
                    let event = owner.BaseEvent();
                    if event.DefaultHandled() || event.defaultPrevented() {
                        break;
                    }
                }
            }
        } else {
            // Source: event_dispatcher.cc:464-470
            #[cfg(target_os = "macos")]
            if self.event_.borrow().BaseEvent().r#type() == "keypress" {
                if let Some(view) = &self.view_ {
                    view.SyncSelectionForced();
                }
            }
        }
        let is_mouse_up = {
            let owner = self.event_.borrow();
            owner.IsMouseEvent() && owner.BaseEvent().r#type() == "mouseup"
        };
        if is_mouse_up {
            self.context_
                .document
                .ClearPopoverPickerPointerdown(&self.node_);
        }
        // TODO: cc:477-487 untrusted-select-mousedown UseCounter instrumentation.
        let target = self.path_.borrow().GetWindowEventContext().Target();
        let mut owner = self.event_.borrow_mut();
        owner.SetTarget(target);
        if owner.BaseEvent().RawTarget().is_none() {
            owner.SetRelatedTarget(None);
        }
    }

    // Source: event_dispatcher.cc:253,380
    fn SetOriginalTarget(&self) {
        let target = self
            .context_
            .paths
            .EventTargetRespectingTargetRules(&self.node_);
        self.event_.borrow_mut().SetTarget(Some(target));
    }
}

impl EventDispatcherInterface for EventDispatcher<'_> {
    fn Dispatch(&mut self) -> DispatchEventResult {
        EventDispatcher::Dispatch(self)
    }
}
