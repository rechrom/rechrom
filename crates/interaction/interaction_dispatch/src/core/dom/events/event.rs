/*
 * Copyright (C) 2001 Peter Kelly (pmk@post.com)
 * Copyright (C) 2001 Tobias Anton (anton@stud.fbi.fh-darmstadt.de)
 * Copyright (C) 2006 Samuel Weinig (sam.weinig@gmail.com)
 * Copyright (C) 2003, 2004, 2005, 2006, 2007, 2008 Apple Inc. All rights
 * reserved.
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
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/event.h:1-413
// Source: third_party/blink/renderer/core/dom/events/event.cc:1-452
// C++ includes, namespace, export and wrapper annotations map to Rust modules.

use std::cell::RefCell;
use std::rc::Rc;

use interaction_input::base::time::time::TimeTicks;

use super::event_dispatch_result::DispatchEventResult;
pub use super::event_init::EventInit;

// Source: event.h:41-50,397-404
// These shared handles retain the original target, path and underlying event
// owners. Tree storage and retargeting remain with the supplying DOM owner.
pub type EventTargetRef = Rc<dyn EventTarget>;
pub type EventPathRef = Rc<RefCell<dyn EventPath>>;
#[derive(Clone)]
pub struct EventRef {
    owner: Rc<RefCell<dyn EventInterface>>,
    // A second typed view of the SAME allocation, for the source's dynamic
    // event-subclass projection. This is neither another Event nor a result
    // registry; retained script and native default handlers share its state.
    typed_owner: Rc<dyn std::any::Any>,
    identity: Rc<()>,
    // A native subclass whose fields use interior mutability may lend its
    // base Event as a dispatch view. That view retains the SAME subclass;
    // the subclass's own base handle remains undecorated, avoiding a cycle.
    class_owner: Option<Rc<dyn std::any::Any>>,
}

impl std::fmt::Debug for EventRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventRef")
            .field("identity", &self.Identity())
            .finish_non_exhaustive()
    }
}

impl EventRef {
    /// Creates the stable owner used by dispatch and underlying chains.
    pub fn new(event: impl EventInterface + 'static) -> Self {
        let identity = event.BaseEvent().identity_.clone();
        let typed = Rc::new(RefCell::new(event));
        let typed_owner: Rc<dyn std::any::Any> = typed.clone();
        Self {
            owner: typed,
            typed_owner,
            identity,
            class_owner: None,
        }
    }

    /// Projects an already shared concrete event into the dispatch interface
    /// without changing its owner or base-event identity. Coalesced/predicted
    /// PointerEvents rely on this just as Blink retains the same Member owner.
    pub fn FromShared<T: EventInterface + 'static>(owner: Rc<RefCell<T>>) -> Self {
        let identity = owner.borrow().BaseEvent().identity_.clone();
        let typed_owner: Rc<dyn std::any::Any> = owner.clone();
        let owner: Rc<RefCell<dyn EventInterface>> = owner;
        Self {
            owner,
            typed_owner,
            identity,
            class_owner: None,
        }
    }

    /// Project the concrete source owner without borrowing, copying or moving
    /// its Event. The returned handle aliases the canonical dispatch owner.
    pub fn Typed<T: EventInterface + 'static>(&self) -> Option<Rc<RefCell<T>>> {
        let typed = self.typed_owner.clone().downcast::<RefCell<T>>().ok()?;
        #[cfg(debug_assertions)]
        {
            let canonical: Rc<RefCell<dyn EventInterface>> = typed.clone();
            debug_assert!(Rc::ptr_eq(&canonical, &self.owner));
        }
        Some(typed)
    }

    /// Identity comparison does not borrow an event across script reentry.
    pub fn Identity(&self) -> usize {
        Rc::as_ptr(&self.identity) as usize
    }
    pub fn WithClassOwner<T: std::any::Any>(&self, owner: Rc<T>) -> Self {
        let mut view = self.clone();
        view.class_owner = Some(owner);
        view
    }
    pub fn ClassOwner<T: std::any::Any>(&self) -> Option<Rc<T>> {
        self.class_owner.clone()?.downcast::<T>().ok()
    }

    pub fn IsSameEvent(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.identity, &other.identity)
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, dyn EventInterface> {
        self.owner.borrow()
    }
    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, dyn EventInterface> {
        self.owner.borrow_mut()
    }
}

// Source: event.h:41-42,339-340
pub trait DOMOrigin {}
pub trait DOMWrapperWorld {
    fn WorldId(&self) -> i32;
}

// Source: event.cc:261-267,284-290,339-354,365-374
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventTargetKind {
    Node,
    Window,
    Other,
}

/// Only target projections directly read by Event. DOM retains the target.
pub trait EventTarget {
    /// Typed DOMWindow projection, retaining the original Window allocation.
    fn AsAbstractView(&self) -> Option<crate::core::events::ui_event::AbstractViewRef> {
        None
    }
    /// Native EventTargetImpl dispatches directly; Node/Window interfaces use
    /// their existing DOM path. This retains its actual source allocation.
    fn AsNativeEventTarget(&self) -> Option<Rc<super::event_target::EventTarget>> {
        None
    }
    fn Kind(&self) -> EventTargetKind;
    /// Node/Window binding checks read the original execution context's
    /// immutable origin owner. Other EventTargets have no document origin.
    fn SecurityOriginOwner(&self) -> Option<Rc<dyn foundation::security::SecurityOriginOwner>> {
        None
    }
    fn NodeId(&self) -> Option<u64> {
        None
    }
    fn IsPseudoElement(&self) -> bool {
        false
    }
    /// Source event.cc:368-369; Node::IsSVGElement identifies the SVG owner
    /// before Event reads SVGElement::CorrespondingElement.
    fn IsSVGElement(&self) -> bool {
        false
    }
    /// Source Node connection query used only by pseudo target rules.
    fn IsConnected(&self) -> bool {
        false
    }
    /// Source Node virtual used for pseudo event-target rules.
    fn HasActivationBehavior(&self) -> bool {
        false
    }
    fn CorrespondingElement(&self) -> Option<EventTargetRef> {
        None
    }
}

// Source: event.cc:301-309,326-363
/// Projection of the resident dispatch path. Implementations retain the same
/// path owner through InitializeWith; Event never reconstructs its tree.
pub trait EventPath {
    fn IsEmpty(&self) -> bool;
    fn InitializeWith(&mut self, node: &EventTargetRef, event: &Event);
    fn ComposedPathForNode(&self, node: &EventTargetRef) -> Option<Vec<EventTargetRef>>;
    fn TopScopeComposedPath(&self) -> Vec<EventTargetRef>;
    /// Internal node contexts retain closed-root nodes; unlike composedPath this
    /// source-only view is not filtered for a script tree scope.
    fn NodeTargets(&self) -> Vec<EventTargetRef>;
}

pub trait EventPathFactory {
    fn Create(&self, node: &EventTargetRef, event: &Event) -> EventPathRef;
}

// Source: event.cc:280-291,405-410,445-449
/// The DOM owner supplies these individual scope operations, retaining its
/// live tree identity. No tree getter or mutable DOM access crosses the port.
pub trait EventRetargeting {
    fn RetargetInTreeScope(
        &self,
        node: &EventTargetRef,
        current: &EventTargetRef,
    ) -> EventTargetRef;
    fn RetargetInDocument(&self, node: &EventTargetRef) -> EventTargetRef;
}

pub trait CSSPseudoElement {
    fn Element(&self) -> EventTargetRef;
}

pub trait CSSPseudoElementFactory {
    fn From(&self, pseudo: &EventTargetRef) -> Rc<dyn CSSPseudoElement>;
}

// Source: event.cc:68-76; event.h:128-129
/// Time comes from the embedding's monotonic clock at source construction.
pub trait EventClock {
    fn Now(&self) -> TimeTicks;
}

// Source: event.cc:376-394
/// Performance supplies its existing origin/clamping conversion. The optional
/// window and worker results retain the source branch ordering.
pub trait EventTimeStampContext {
    fn WindowTimeStamp(&self, timestamp: TimeTicks) -> Option<f64>;
    fn WorkerTimeStamp(&self, timestamp: TimeTicks) -> Option<f64>;
}

// Source: event.h:56-59
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bubbles {
    kYes,
    kNo,
}
// Source: event.h:61-64
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cancelable {
    kYes,
    kNo,
}
// Source: event.h:66-71
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PhaseType {
    kNone = 0,
    kCapturingPhase = 1,
    kAtTarget = 2,
    kBubblingPhase = 3,
}
// Source: event.h:73-76
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposedMode {
    kComposed,
    kScoped,
}
// Source: event.h:78-90
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PassiveMode {
    kNotPassiveDefault,
    kNotPassive,
    kPassive,
    kPassiveForcedDocumentLevel,
    kPassiveDefault,
}

// Source: event.h:52-54,364-408
// Subclasses embed this state and implement EventInterface, preserving the
// same base owner for virtual hooks and underlying-event references.
/// Immutable dynamic class fact retained by the base Event view passed to
/// EventPathFactory. initEvent changes its type, never its native subclass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventPathClass {
    Other,
    Keyboard,
    Touch,
}
pub struct Event {
    // A stable identity allows e == this before borrowing an underlying base.
    identity_: Rc<()>,
    type_: String,
    event_path_class_: EventPathClass,
    bubbles_: bool,
    cancelable_: bool,
    composed_: bool,
    propagation_stopped_: bool,
    immediate_propagation_stopped_: bool,
    default_prevented_: bool,
    default_handled_: bool,
    was_initialized_: bool,
    is_trusted_: bool,
    prevent_default_called_on_uncancelable_event_: bool,
    legacy_did_listeners_throw_flag_: bool,
    fire_only_capture_listeners_at_target_: bool,
    fire_only_non_capture_listeners_at_target_: bool,
    copy_event_path_from_underlying_event_: bool,
    invocation_target_in_shadow_tree_: bool,
    handling_passive_: PassiveMode,
    event_phase_: PhaseType,
    // Source: event.h:337,395
    // TODO: retain probe::AsyncTaskContext when async probe bindings land.
    current_target_: Option<EventTargetRef>,
    target_: Option<EventTargetRef>,
    pseudo_element_target_: Option<Rc<dyn CSSPseudoElement>>,
    underlying_event_: Option<EventRef>,
    event_path_: Option<EventPathRef>,
    platform_time_stamp_: TimeTicks,
}

impl Event {
    pub fn EventPathClass(&self) -> EventPathClass {
        self.event_path_class_
    }
    pub(crate) fn SetEventPathClass(&mut self, class: EventPathClass) {
        self.event_path_class_ = class;
    }
    // Source: event.h:92-105
    pub fn Create(clock: &dyn EventClock) -> Self {
        Self::Uninitialized(clock.Now())
    }
    pub fn CreateWithType(event_type: impl Into<String>, clock: &dyn EventClock) -> Self {
        Self::new(
            event_type,
            Bubbles::kNo,
            Cancelable::kNo,
            ComposedMode::kScoped,
            clock.Now(),
        )
    }
    pub fn CreateCancelable(event_type: impl Into<String>, clock: &dyn EventClock) -> Self {
        Self::new(
            event_type,
            Bubbles::kNo,
            Cancelable::kYes,
            ComposedMode::kScoped,
            clock.Now(),
        )
    }
    pub fn CreateBubble(event_type: impl Into<String>, clock: &dyn EventClock) -> Self {
        Self::new(
            event_type,
            Bubbles::kYes,
            Cancelable::kNo,
            ComposedMode::kScoped,
            clock.Now(),
        )
    }
    pub fn CreateCancelableBubble(event_type: impl Into<String>, clock: &dyn EventClock) -> Self {
        Self::new(
            event_type,
            Bubbles::kYes,
            Cancelable::kYes,
            ComposedMode::kScoped,
            clock.Now(),
        )
    }
    // Source: event.h:107-109,128-129
    pub fn CreateWithInitializer(
        event_type: impl Into<String>,
        init: &EventInit,
        clock: &dyn EventClock,
    ) -> Self {
        Self::WithInitializer(event_type, init, clock.Now())
    }

    // Source: event.h:132-136; event.cc:115-138
    // The base implementation does not consume related_target. Subclasses can
    // override initialization while retaining the dispatch guard below.
    pub fn initEvent(&mut self, event_type: impl Into<String>, bubbles: bool, cancelable: bool) {
        self.initEventWithRelatedTarget(event_type, bubbles, cancelable, None);
    }
    pub fn initEventWithRelatedTarget(
        &mut self,
        event_type: impl Into<String>,
        bubbles: bool,
        cancelable: bool,
        _related_target: Option<EventTargetRef>,
    ) {
        if self.IsBeingDispatched() {
            return;
        }
        self.was_initialized_ = true;
        self.propagation_stopped_ = false;
        self.immediate_propagation_stopped_ = false;
        self.default_prevented_ = false;
        self.is_trusted_ = false;
        self.prevent_default_called_on_uncancelable_event_ = false;
        self.type_ = event_type.into();
        self.bubbles_ = bubbles;
        self.cancelable_ = cancelable;
    }

    // Source: event.h:138-139
    pub fn r#type(&self) -> &str {
        &self.type_
    }
    pub fn SetType(&mut self, event_type: impl Into<String>) {
        self.type_ = event_type.into();
    }
    // Source: event.h:141-148; event.cc:261-267
    pub fn target(&self) -> Option<EventTargetRef> {
        debug_assert!(!self
            .target_
            .as_ref()
            .is_some_and(|target| target.IsPseudoElement()));
        self.target_.clone()
    }
    pub fn SetTarget(&mut self, target: Option<EventTargetRef>) {
        <Self as EventInterface>::SetTarget(self, target)
    }
    pub fn RawTarget(&self) -> Option<EventTargetRef> {
        self.target_.clone()
    }
    // Source: event.h:150-153; event.cc:405-410
    pub fn SetPseudoElementTarget(
        &mut self,
        pseudo: &EventTargetRef,
        enabled: bool,
        factory: &dyn CSSPseudoElementFactory,
    ) {
        if !enabled {
            return;
        }
        self.pseudo_element_target_ = Some(factory.From(pseudo));
    }
    // Source: event.h:155-164; event.cc:365-374
    pub fn currentTarget(&self) -> Option<EventTargetRef> {
        let current = self.current_target_.as_ref()?;
        if current.IsSVGElement() {
            if let Some(corresponding) = current.CorrespondingElement() {
                return Some(corresponding);
            }
        }
        Some(current.clone())
    }
    pub fn SetCurrentTarget(&mut self, current: Option<EventTargetRef>) {
        self.current_target_ = current;
    }
    pub fn SetInvocationTargetInShadowTree(&mut self, in_shadow_tree: bool) {
        self.invocation_target_in_shadow_tree_ = in_shadow_tree;
    }
    pub fn invocationTargetInShadowTree(&self) -> bool {
        self.invocation_target_in_shadow_tree_
    }

    // Source: event.h:175-197
    pub fn eventPhase(&self) -> PhaseType {
        self.event_phase_
    }
    pub fn SetEventPhase(&mut self, phase: PhaseType) {
        self.event_phase_ = phase;
    }
    pub fn SetFireOnlyCaptureListenersAtTarget(&mut self, value: bool) {
        debug_assert_eq!(self.event_phase_, PhaseType::kAtTarget);
        self.fire_only_capture_listeners_at_target_ = value;
    }
    pub fn SetFireOnlyNonCaptureListenersAtTarget(&mut self, value: bool) {
        debug_assert_eq!(self.event_phase_, PhaseType::kAtTarget);
        self.fire_only_non_capture_listeners_at_target_ = value;
    }
    pub fn FireOnlyCaptureListenersAtTarget(&self) -> bool {
        self.fire_only_capture_listeners_at_target_
    }
    pub fn FireOnlyNonCaptureListenersAtTarget(&self) -> bool {
        self.fire_only_non_capture_listeners_at_target_
    }
    // Source: event.h:199-207
    pub fn bubbles(&self) -> bool {
        self.bubbles_
    }
    pub fn cancelable(&self) -> bool {
        self.cancelable_
    }
    pub fn composed(&self) -> bool {
        self.composed_
    }
    pub fn PlatformTimeStamp(&self) -> TimeTicks {
        self.platform_time_stamp_
    }
    // Source: event.h:209-216
    pub fn stopPropagation(&mut self) {
        self.propagation_stopped_ = true;
    }
    pub fn SetStopPropagation(&mut self, value: bool) {
        self.propagation_stopped_ = value;
    }
    pub fn stopImmediatePropagation(&mut self) {
        self.immediate_propagation_stopped_ = true;
    }
    pub fn SetStopImmediatePropagation(&mut self, value: bool) {
        self.immediate_propagation_stopped_ = value;
    }
    // Source: event.h:218-221
    pub fn srcElement(&self) -> Option<EventTargetRef> {
        self.target()
    }
    // Source: event.h:223-224; event.cc:140-164
    pub fn legacyReturnValue(&self) -> bool {
        // TODO: event.cc:142-148 UseCounter instrumentation.
        !self.defaultPrevented()
    }
    pub fn setLegacyReturnValue(&mut self, return_value: bool) {
        <Self as EventInterface>::setLegacyReturnValue(self, return_value)
    }

    // Source: event.h:254-266
    pub fn PropagationStopped(&self) -> bool {
        self.propagation_stopped_ || self.immediate_propagation_stopped_
    }
    pub fn ImmediatePropagationStopped(&self) -> bool {
        self.immediate_propagation_stopped_
    }
    pub fn WasInitialized(&self) -> bool {
        self.was_initialized_
    }
    pub fn defaultPrevented(&self) -> bool {
        self.default_prevented_
    }
    // Source: event.cc:242-259
    pub fn preventDefault(&mut self) {
        if self.handling_passive_ != PassiveMode::kNotPassive
            && self.handling_passive_ != PassiveMode::kNotPassiveDefault
        {
            // TODO: event.cc:246-251 passive-listener console instrumentation.
            return;
        }
        if self.cancelable_ {
            self.default_prevented_ = true;
        } else {
            self.prevent_default_called_on_uncancelable_event_ = true;
        }
    }
    pub fn DefaultHandled(&self) -> bool {
        self.default_handled_
    }
    pub fn SetDefaultHandled(&mut self) {
        self.default_handled_ = true;
    }
    // Source: event.h:268-271; event.cc:396-399
    pub fn cancelBubble(&self) -> bool {
        self.PropagationStopped()
    }
    pub fn setCancelBubble(&mut self, cancel: bool) {
        if cancel {
            self.propagation_stopped_ = true;
        }
    }
    // Source: event.h:273-274; event.cc:293-299
    pub fn UnderlyingEvent(&self) -> Option<EventRef> {
        self.underlying_event_.clone()
    }
    pub fn SetUnderlyingEvent(&mut self, underlying: Option<EventRef>) {
        let mut cursor = underlying.clone();
        while let Some(event) = cursor {
            if Rc::ptr_eq(&event.identity, &self.identity_) {
                return;
            }
            cursor = event.borrow().BaseEvent().UnderlyingEvent();
        }
        self.underlying_event_ = underlying;
    }
    // Source: event.h:276-281; event.cc:301-309
    pub fn HasEventPath(&self) -> bool {
        self.event_path_.is_some()
    }
    pub fn GetEventPath(&self) -> EventPathRef {
        self.event_path_
            .as_ref()
            .expect("event path must exist")
            .clone()
    }
    pub fn InitEventPath(&mut self, node: &EventTargetRef, factory: &dyn EventPathFactory) {
        if self.copy_event_path_from_underlying_event_ {
            let underlying = self
                .underlying_event_
                .as_ref()
                .expect("copied path requires an underlying event");
            self.event_path_ = Some(underlying.borrow().BaseEvent().GetEventPath());
        } else if let Some(path) = self.event_path_.as_ref() {
            path.borrow_mut().InitializeWith(node, self);
        } else {
            self.event_path_ = Some(factory.Create(node, self));
        }
    }
    // Source: event.h:283; event.cc:326-363
    pub fn composedPath(&self, return_target_being_dispatched: bool) -> Vec<EventTargetRef> {
        let Some(current) = self.current_target_.as_ref() else {
            debug_assert_eq!(self.event_phase_, PhaseType::kNone);
            if let Some(path) = &self.event_path_ {
                debug_assert!(!path.borrow().IsEmpty());
            }
            return Vec::new();
        };
        if current.Kind() == EventTargetKind::Node {
            return self
                .GetEventPath()
                .borrow()
                .ComposedPathForNode(current)
                .expect("current node must occur in event path");
        }
        let is_window = current.Kind() == EventTargetKind::Window;
        if is_window {
            if let Some(path) = &self.event_path_ {
                let path = path.borrow();
                if !path.IsEmpty() {
                    return path.TopScopeComposedPath();
                }
            }
        }
        if if return_target_being_dispatched {
            self.IsBeingDispatched()
        } else {
            is_window
        } {
            return vec![current.clone()];
        }
        Vec::new()
    }
    // Source: event.h:285
    pub fn IsBeingDispatched(&self) -> bool {
        self.event_phase_ != PhaseType::kNone
    }
    // Source: event.h:293-294
    pub fn isTrusted(&self) -> bool {
        self.is_trusted_
    }
    pub fn SetTrusted(&mut self, value: bool) {
        self.is_trusted_ = value;
    }
    // Source: event.h:296-303; event.cc:311-320
    pub fn IsFullyTrusted(&self) -> bool {
        if !self.isTrusted() {
            return false;
        }
        let mut cursor = self.UnderlyingEvent();
        while let Some(event) = cursor {
            let event = event.borrow();
            if !event.BaseEvent().isTrusted() {
                return false;
            }
            cursor = event.BaseEvent().UnderlyingEvent();
        }
        true
    }
    // Source: event.h:305-308
    pub fn SetComposed(&mut self, value: bool) {
        debug_assert!(!self.IsBeingDispatched());
        self.composed_ = value;
    }
    // Source: event.h:310; event.cc:322-324
    pub fn SetHandlingPassive(&mut self, mode: PassiveMode) {
        self.handling_passive_ = mode;
    }
    // Source: event.h:312-326
    pub fn PreventDefaultCalledOnUncancelableEvent(&self) -> bool {
        self.prevent_default_called_on_uncancelable_event_
    }
    pub fn LegacyDidListenersThrow(&self) -> bool {
        self.legacy_did_listeners_throw_flag_
    }
    pub fn LegacySetDidListenersThrowFlag(&mut self) {
        self.legacy_did_listeners_throw_flag_ = true;
    }
    pub fn SetCopyEventPathFromUnderlyingEvent(&mut self) {
        self.copy_event_path_from_underlying_event_ = true;
    }
    // Source: event.h:353-355
    pub fn SetBubbles(&mut self, value: bool) {
        self.bubbles_ = value;
    }
    pub fn HandlingPassive(&self) -> PassiveMode {
        self.handling_passive_
    }
    // Source: event.h:357-362; event.cc:280-291
    pub fn Retarget(
        &self,
        node: Option<EventTargetRef>,
        retargeting: &dyn EventRetargeting,
    ) -> Option<EventTargetRef> {
        let node = node?;
        if let Some(current) = self.currentTarget() {
            if current.Kind() == EventTargetKind::Node {
                return Some(retargeting.RetargetInTreeScope(&node, &current));
            }
        }
        Some(retargeting.RetargetInDocument(&node))
    }
}

impl Event {
    // Source: event.cc:54-56
    pub fn Uninitialized(timestamp: TimeTicks) -> Self {
        let mut event = Self::new(
            "",
            Bubbles::kNo,
            Cancelable::kNo,
            ComposedMode::kScoped,
            timestamp,
        );
        event.was_initialized_ = false;
        event
    }
    // Source: event.cc:58-66
    pub fn WithPlatformTimeStamp(
        event_type: impl Into<String>,
        bubbles: Bubbles,
        cancelable: Cancelable,
        timestamp: TimeTicks,
    ) -> Self {
        Self::new(
            event_type,
            bubbles,
            cancelable,
            ComposedMode::kScoped,
            timestamp,
        )
    }
    // Source: event.cc:68-76
    pub fn WithClock(
        event_type: impl Into<String>,
        bubbles: Bubbles,
        cancelable: Cancelable,
        composed: ComposedMode,
        clock: &dyn EventClock,
    ) -> Self {
        Self::new(event_type, bubbles, cancelable, composed, clock.Now())
    }
    // Source: event.cc:78-101
    pub fn new(
        event_type: impl Into<String>,
        bubbles: Bubbles,
        cancelable: Cancelable,
        composed: ComposedMode,
        timestamp: TimeTicks,
    ) -> Self {
        Self {
            identity_: Rc::new(()),
            type_: event_type.into(),
            event_path_class_: EventPathClass::Other,
            bubbles_: bubbles == Bubbles::kYes,
            cancelable_: cancelable == Cancelable::kYes,
            composed_: composed == ComposedMode::kComposed,
            propagation_stopped_: false,
            immediate_propagation_stopped_: false,
            default_prevented_: false,
            default_handled_: false,
            was_initialized_: true,
            is_trusted_: false,
            prevent_default_called_on_uncancelable_event_: false,
            legacy_did_listeners_throw_flag_: false,
            fire_only_capture_listeners_at_target_: false,
            fire_only_non_capture_listeners_at_target_: false,
            copy_event_path_from_underlying_event_: false,
            invocation_target_in_shadow_tree_: false,
            handling_passive_: PassiveMode::kNotPassiveDefault,
            event_phase_: PhaseType::kNone,
            current_target_: None,
            target_: None,
            pseudo_element_target_: None,
            underlying_event_: None,
            event_path_: None,
            platform_time_stamp_: timestamp,
        }
    }
    // Source: event.cc:103-111
    pub fn WithInitializer(
        event_type: impl Into<String>,
        init: &EventInit,
        timestamp: TimeTicks,
    ) -> Self {
        Self::new(
            event_type,
            if init.bubbles {
                Bubbles::kYes
            } else {
                Bubbles::kNo
            },
            if init.cancelable {
                Cancelable::kYes
            } else {
                Cancelable::kNo
            },
            if init.composed {
                ComposedMode::kComposed
            } else {
                ComposedMode::kScoped
            },
            timestamp,
        )
    }
    // Source: event.cc:113,412-419
    // Rc field owners and Rust Drop replace default destruction and GC tracing.

    // Source: event.cc:376-394
    pub fn timeStamp(&self, context: Option<&dyn EventTimeStampContext>) -> f64 {
        let Some(context) = context else {
            return 0.0;
        };
        if let Some(timestamp) = context.WindowTimeStamp(self.platform_time_stamp_) {
            return timestamp;
        }
        if let Some(timestamp) = context.WorkerTimeStamp(self.platform_time_stamp_) {
            return timestamp;
        }
        0.0
    }
}

// Source: event.h:166-174,226-252,287-291,328-345,347-351
// Virtual methods keep source defaults here. Derived event values embed Event
// and override these methods without copying the base state or its owners.
pub trait EventInterface {
    /// Prepare virtual dispatch while borrowed, then drop the event borrow
    /// before listeners can synchronously reenter the same native Event.
    fn PrepareDispatchHook<'a>(
        &self,
        _nested: &'a dyn crate::core::events::mouse_event::MouseNestedDispatch,
    ) -> Box<dyn super::event_dispatcher::EventDispatchHook + 'a> {
        Box::new(super::event_dispatcher::BaseEventDispatchHook)
    }
    fn AsKeyboardEvent(&self) -> Option<&crate::core::events::keyboard_event::KeyboardEvent> {
        None
    }
    fn AsWheelEvent(&self) -> Option<&crate::core::events::wheel_event::WheelEvent> {
        None
    }
    fn AsTouchEvent(&self) -> Option<&crate::core::events::touch_event::TouchEvent> {
        None
    }
    fn AsTextEvent(&self) -> Option<&crate::core::events::text_event::TextEvent> {
        None
    }
    fn AsInputEvent(&self) -> Option<&crate::core::events::input_event::InputEvent> {
        None
    }
    fn AsInputEventMut(&mut self) -> Option<&mut crate::core::events::input_event::InputEvent> {
        None
    }
    fn AsCompositionEvent(
        &self,
    ) -> Option<&crate::core::events::composition_event::CompositionEvent> {
        None
    }
    // Borrow the concrete source bases without replacing the retained EventRef.
    fn AsInterestEvent(&self) -> Option<&crate::core::events::interest_event::InterestEvent> {
        None
    }
    fn AsFontFaceSetLoadEvent(
        &self,
    ) -> Option<&crate::core::events::font_face_set_load_event::FontFaceSetLoadEvent> {
        None
    }
    fn AsPageTransitionEvent(
        &self,
    ) -> Option<&crate::core::events::page_transition_event::PageTransitionEvent> {
        None
    }
    fn AsTrackEvent(&self) -> Option<&crate::core::events::track_event::TrackEvent> {
        None
    }
    fn AsFormDataEvent(&self) -> Option<&crate::core::events::form_data_event::FormDataEvent> {
        None
    }
    fn AsSecurityPolicyViolationEvent(
        &self,
    ) -> Option<&crate::core::events::security_policy_violation_event::SecurityPolicyViolationEvent>
    {
        None
    }
    fn AsSnapEvent(&self) -> Option<&crate::core::events::snap_event::SnapEvent> {
        None
    }
    fn AsToggleEvent(&self) -> Option<&crate::core::events::toggle_event::ToggleEvent> {
        None
    }
    fn AsCommandEvent(&self) -> Option<&crate::core::events::command_event::CommandEvent> {
        None
    }
    fn AsMouseEvent(&self) -> Option<&crate::core::events::mouse_event::MouseEvent> {
        None
    }
    fn AsDragEvent(&self) -> Option<&crate::core::events::drag_event::DragEvent> {
        None
    }
    fn AsPointerEvent(&self) -> Option<&crate::core::events::pointer_event::PointerEvent> {
        None
    }
    fn AsUIEventWithKeyState(
        &self,
    ) -> Option<&crate::core::events::ui_event_with_key_state::UIEventWithKeyState> {
        None
    }
    fn BaseEvent(&self) -> &Event;
    fn BaseEventMut(&mut self) -> &mut Event;
    // Source: event.h:166-170
    fn DoneDispatchingEventAtCurrentTarget(&mut self) {}
    // Source: event.h:172-173
    /// SubmitEvent exposes its retained HTMLElement owner through the base
    /// virtual boundary so bindings project identity without rebuilding it.
    fn Submitter(&self) -> Option<EventTargetRef> {
        None
    }
    fn relatedTarget(&self) -> Option<EventTargetRef> {
        None
    }
    fn SetRelatedTarget(&mut self, _target: Option<EventTargetRef>) {}
    // Source: event.h:263; event.cc:242-259
    fn preventDefault(&mut self) {
        self.BaseEventMut().preventDefault();
    }
    /// Source CaptureSourceLocation crosses the script/dispatch boundary as
    /// its small value; no script runtime is retained by an Event.
    fn preventDefaultWithLocation(
        &mut self,
        _location: crate::core::frame::reporting_context::ReportLocation,
    ) {
        self.preventDefault();
    }

    // Source: event.h:224; event.cc:152-164
    fn setLegacyReturnValue(&mut self, return_value: bool) {
        // TODO: event.cc:154-155,160-161 UseCounter instrumentation.
        if return_value {
            if !self.BaseEvent().defaultPrevented() {
                self.BaseEventMut().default_prevented_ = false;
            }
        } else {
            self.preventDefault();
        }
    }
    // Source: event.cc:166-172
    fn InterfaceName(&self) -> &str {
        "Event"
    }
    fn HasInterface(&self, name: &str) -> bool {
        self.InterfaceName() == name
    }
    // Source: event.cc:174-240
    /// Typed navigation subclasses project their source user-initiation fact.
    /// None denotes an actual non-navigation Event subclass.
    fn NavigationUserInitiated(&self) -> Option<bool> {
        None
    }
    fn IsUIEvent(&self) -> bool {
        false
    }
    fn IsMouseEvent(&self) -> bool {
        false
    }
    fn IsFocusEvent(&self) -> bool {
        false
    }
    fn IsKeyboardEvent(&self) -> bool {
        false
    }
    fn IsTouchEvent(&self) -> bool {
        false
    }
    fn IsGestureEvent(&self) -> bool {
        false
    }
    fn IsWheelEvent(&self) -> bool {
        false
    }
    fn IsPointerEvent(&self) -> bool {
        false
    }
    fn IsInputEvent(&self) -> bool {
        false
    }
    fn IsDragEvent(&self) -> bool {
        false
    }
    fn IsCompositionEvent(&self) -> bool {
        false
    }
    fn AsClipboardEvent(&self) -> Option<&crate::core::events::clipboard_event::ClipboardEvent> {
        None
    }
    fn IsClipboardEvent(&self) -> bool {
        false
    }
    fn IsBeforeTextInsertedEvent(&self) -> bool {
        false
    }
    fn IsBeforeCreatePolicyEvent(&self) -> bool {
        false
    }
    fn IsBeforeUnloadEvent(&self) -> bool {
        false
    }
    fn AsErrorEvent(&self) -> Option<&crate::core::events::error_event::ErrorEvent> {
        None
    }
    fn IsErrorEvent(&self) -> bool {
        false
    }
    fn IsPatchEvent(&self) -> bool {
        false
    }
    // Source: event.h:287-291
    fn CanBeDispatchedInWorld(&self, _world: &dyn DOMWrapperWorld) -> bool {
        true
    }
    // Source: event.h:328-333
    fn ShouldDispatchEvenWhenExecutionContextIsPaused(&self) -> bool {
        false
    }
    // Source: event.h:335; event.cc:401-403
    fn DispatchEvent(&mut self, dispatcher: &mut dyn EventDispatcher) -> DispatchEventResult {
        dispatcher.Dispatch()
    }
    // Source: event.h:339-340
    fn GetDOMOrigin(&self) -> Option<Rc<dyn DOMOrigin>> {
        None
    }
    // Source: event.h:345; event.cc:278
    fn ReceivedTarget(&mut self) {}
    // Source: event.cc:269-276
    fn SetTarget(&mut self, target: Option<EventTargetRef>) {
        let unchanged = match (&self.BaseEvent().target_, &target) {
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if unchanged {
            return;
        }
        self.BaseEventMut().target_ = target;
        if self.BaseEvent().target_.is_some() {
            self.ReceivedTarget();
        }
    }
    // Source: event.h:347-351; event.cc:421-449
    fn pseudoTarget(
        &self,
        enabled: bool,
        retargeting: &dyn EventRetargeting,
    ) -> Option<Rc<dyn CSSPseudoElement>> {
        if !enabled {
            return None;
        }
        let event = self.BaseEvent();
        let pseudo = event.pseudo_element_target_.as_ref()?;
        event.current_target_.as_ref()?;
        if (self.IsMouseEvent() || self.IsPointerEvent())
            && matches!(
                event.r#type(),
                "mouseover"
                    | "mouseout"
                    | "mouseenter"
                    | "mouseleave"
                    | "pointerover"
                    | "pointerout"
                    | "pointerenter"
                    | "pointerleave"
            )
        {
            return None;
        }
        let element = pseudo.Element();
        let retargeted = event.Retarget(Some(element.clone()), retargeting)?;
        if Rc::ptr_eq(&retargeted, &element) {
            return Some(pseudo.clone());
        }
        None
    }
}

impl EventInterface for Event {
    fn BaseEvent(&self) -> &Event {
        self
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        self
    }
}

// Source: event.cc:401-403
pub trait EventDispatcher {
    fn Dispatch(&mut self) -> DispatchEventResult;
}
