// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/frame/event_handler_registry.h:25-137
// Source: third_party/blink/renderer/core/frame/event_handler_registry.cc:25-310,377-405

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use interaction_dispatch::core::dom::events::add_event_listener_options_resolved::AddEventListenerOptions;
use interaction_dispatch::core::dom::events::event::{EventTargetKind, EventTargetRef};

pub use crate::cc::input::event_listener_properties::{
    EventListenerClass, EventListenerProperties,
};

// Source: event_handler_registry.h:25-26,136
// These handles project the embedding's existing owners. They carry identity,
// not a replacement Frame/LayoutObject model.
pub trait RegistryFrameIdentity {}
pub type RegistryFrameRef = Rc<dyn RegistryFrameIdentity>;
pub trait RegistryLayoutIdentity {
    fn AsAny(&self) -> &dyn std::any::Any;
}
pub type RegistryLayoutRef = Rc<dyn RegistryLayoutIdentity>;

// Source: event_handler_registry.cc:36-46,298-309,385-397
// The DOM/Layout boundary supplies current facts for the exact retained target.
// For a Window, invalidation_owner is ContentLayoutObject; for a Node it is
// GetLayoutObject. A missing layout owner is a normal source branch.
pub struct WheelRegistryTargetFacts {
    pub frame: Option<RegistryFrameRef>,
    pub local_root: Option<RegistryFrameRef>,
    pub document_has_page: bool,
    pub frame_has_page: bool,
    pub invalidation_owner: Option<RegistryLayoutRef>,
}

// Source: event_handler_registry.cc:241-244,301,308
// The facade applies these synchronous typed notifications to their owners.
// No Page/ChromeClient/Layout lifecycle object crosses this package boundary.
pub enum WheelRegistryEffect {
    SetHasScrollEventHandlers {
        frame: Option<RegistryFrameRef>,
        has_handlers: bool,
    },
    SetNeedsLowLatencyInput {
        frame: Option<RegistryFrameRef>,
        needed: bool,
    },
    SetEventListenerProperties {
        frame: Option<RegistryFrameRef>,
        class: EventListenerClass,
        properties: EventListenerProperties,
    },
    MarkBlockingWheelEventHandlerChanged {
        layout: RegistryLayoutRef,
    },
    MarkEffectiveAllowedTouchActionChanged {
        layout: RegistryLayoutRef,
    },
}

pub trait WheelRegistryBoundary {
    fn FrameHasPage(&self, frame: &RegistryFrameRef) -> bool;
    fn TargetFacts(&self, target: &EventTargetRef) -> WheelRegistryTargetFacts;
    fn Emit(&self, effect: WheelRegistryEffect);
}

// Source: event_handler_registry.h:41-44
// Preserve the numeric identities in the complete upstream class enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EventHandlerClass {
    kScrollEvent = 0,
    kWheelEventBlocking = 1,
    kWheelEventPassive = 2,
    kTouchAction = 3,
    kTouchStartOrMoveEventBlocking = 4,
    kTouchStartOrMoveEventBlockingLowLatency = 5,
    kTouchStartOrMoveEventPassive = 6,
    kTouchEndOrCancelEventBlocking = 7,
    kTouchEndOrCancelEventPassive = 8,
    kPointerEvent = 9,
    kPointerRawUpdateEvent = 10,
}

impl EventHandlerClass {
    const ALL: [Self; 11] = [
        Self::kScrollEvent,
        Self::kWheelEventBlocking,
        Self::kWheelEventPassive,
        Self::kTouchAction,
        Self::kTouchStartOrMoveEventBlocking,
        Self::kTouchStartOrMoveEventBlockingLowLatency,
        Self::kTouchStartOrMoveEventPassive,
        Self::kTouchEndOrCancelEventBlocking,
        Self::kTouchEndOrCancelEventPassive,
        Self::kPointerEvent,
        Self::kPointerRawUpdateEvent,
    ];
    fn index(self) -> usize {
        self as usize
    }
}

// Source: event_handler_registry.h:90-96
#[derive(Clone, Copy, PartialEq, Eq)]
enum ChangeOperation {
    kAdd,
    kRemove,
    kRemoveAll,
}

// Source: event_handler_registry.h:25-26
// HashCountedSet::size counts distinct targets, while the value retains each
// target's listener count. Weak owners match UntracedMember lifetime. The DOM
// adapter must unregister an owner before releasing its final target handle.
#[derive(Default)]
pub struct EventTargetSet {
    entries: HashMap<
        usize,
        (
            Weak<dyn interaction_dispatch::core::dom::events::event::EventTarget>,
            usize,
        ),
    >,
}

/// The source registry's counted target sets, independent of its downstream
/// rendering notifications. Bindings and the full frame registry use this same
/// membership implementation; queries never reconstruct it from DOM listeners.
#[derive(Default)]
pub struct EventHandlerRegistryTargets {
    classes_: [RefCell<EventTargetSet>; 11],
    blocking_wheel_invalidation: RefCell<Option<Rc<dyn Fn(Option<u64>)>>>,
}

impl EventHandlerRegistryTargets {
    pub fn SetBlockingWheelInvalidation(&self, emit: Rc<dyn Fn(Option<u64>)>) {
        *self.blocking_wheel_invalidation.borrow_mut() = Some(emit);
    }
    pub fn HasBlockingWheelEventHandlers(&self) -> bool {
        self.HasEventHandlers(EventHandlerClass::kWheelEventBlocking)
    }
    pub fn HasBlockingWheelHandler(&self, target: &EventTargetRef) -> bool {
        self.classes_[EventHandlerClass::kWheelEventBlocking.index()]
            .borrow()
            .Contains(target)
    }
    fn EmitBlockingWheelInvalidation(&self, node: Option<u64>) {
        if let Some(emit) = self.blocking_wheel_invalidation.borrow().clone() {
            emit(node);
        }
    }
    // event_handler_registry.cc:103-126.
    pub fn HasPointerRawUpdateEventHandlers(&self) -> bool {
        self.HasEventHandlers(EventHandlerClass::kPointerRawUpdateEvent)
    }
    pub fn HasPointerEventHandlers(&self) -> bool {
        self.HasEventHandlers(EventHandlerClass::kPointerEvent)
    }
    pub fn HasTouchEventHandlers(&self) -> bool {
        [
            EventHandlerClass::kTouchStartOrMoveEventBlocking,
            EventHandlerClass::kTouchStartOrMoveEventBlockingLowLatency,
            EventHandlerClass::kTouchStartOrMoveEventPassive,
            EventHandlerClass::kTouchEndOrCancelEventBlocking,
            EventHandlerClass::kTouchEndOrCancelEventPassive,
        ]
        .into_iter()
        .any(|class| self.HasEventHandlers(class))
    }
    pub fn HasBlockingTouchHandler(&self, target: &EventTargetRef) -> bool {
        [
            EventHandlerClass::kTouchStartOrMoveEventBlocking,
            EventHandlerClass::kTouchStartOrMoveEventBlockingLowLatency,
        ]
        .into_iter()
        .any(|class| self.classes_[class.index()].borrow().Contains(target))
    }

    fn HasEventHandlers(&self, class: EventHandlerClass) -> bool {
        self.classes_[class.index()].borrow().size() != 0
    }

    fn Update(
        &self,
        operation: ChangeOperation,
        class: EventHandlerClass,
        target: &EventTargetRef,
    ) {
        let mut targets = self.classes_[class.index()].borrow_mut();
        let key = EventTargetSet::key(target);
        match operation {
            ChangeOperation::kAdd => {
                let entry = targets
                    .entries
                    .entry(key)
                    .or_insert_with(|| (Rc::downgrade(target), 0));
                entry.1 += 1;
            }
            ChangeOperation::kRemove => {
                if let Some(entry) = targets.entries.get_mut(&key) {
                    entry.1 -= 1;
                    if entry.1 == 0 {
                        targets.entries.remove(&key);
                    }
                }
            }
            ChangeOperation::kRemoveAll => {
                targets.entries.remove(&key);
            }
        }
    }

    pub fn BindTarget(self: &Rc<Self>, target: &EventTargetRef) -> EventHandlerRegistryTarget {
        self.BindTargetWithNodeId(target, None)
    }
    pub fn BindTargetWithNodeId(
        self: &Rc<Self>,
        target: &EventTargetRef,
        node: Option<u64>,
    ) -> EventHandlerRegistryTarget {
        EventHandlerRegistryTarget {
            owner: self.clone(),
            target: Rc::downgrade(target),
            key: EventTargetSet::key(target),
            node,
        }
    }

    fn RemoveTarget(&self, key: usize) {
        for class in &self.classes_ {
            class.borrow_mut().entries.remove(&key);
        }
    }
}

/// Source-local target registration lifetime. The token holds no target alive;
/// its teardown supplies the weak-node cleanup from registry.cc:312-338.
pub struct EventHandlerRegistryTarget {
    owner: Rc<EventHandlerRegistryTargets>,
    target: Weak<dyn interaction_dispatch::core::dom::events::event::EventTarget>,
    key: usize,
    node: Option<u64>,
}

impl EventHandlerRegistryTarget {
    pub fn DidAddEventHandler(&self, event_type: &str, options: &AddEventListenerOptions) {
        if let (Some(class), Some(target)) = (
            EventHandlerRegistry::EventTypeToClass(event_type, options),
            self.target.upgrade(),
        ) {
            let before = self.owner.HasBlockingWheelHandler(&target);
            self.owner.Update(ChangeOperation::kAdd, class, &target);
            if before != self.owner.HasBlockingWheelHandler(&target) {
                self.owner.EmitBlockingWheelInvalidation(self.node);
            }
        }
    }
    pub fn DidRemoveEventHandler(&self, event_type: &str, options: &AddEventListenerOptions) {
        if let (Some(class), Some(target)) = (
            EventHandlerRegistry::EventTypeToClass(event_type, options),
            self.target.upgrade(),
        ) {
            let before = self.owner.HasBlockingWheelHandler(&target);
            self.owner.Update(ChangeOperation::kRemove, class, &target);
            if before != self.owner.HasBlockingWheelHandler(&target) {
                self.owner.EmitBlockingWheelInvalidation(self.node);
            }
        }
    }
    pub fn DidRemoveAllEventHandlers(&self) {
        let before = self.owner.classes_[EventHandlerClass::kWheelEventBlocking.index()]
            .borrow()
            .entries
            .contains_key(&self.key);
        self.owner.RemoveTarget(self.key);
        if before {
            self.owner.EmitBlockingWheelInvalidation(self.node);
        }
    }
}

impl Drop for EventHandlerRegistryTarget {
    fn drop(&mut self) {
        self.DidRemoveAllEventHandlers();
    }
}

impl EventTargetSet {
    fn key(target: &EventTargetRef) -> usize {
        Rc::as_ptr(target) as *const () as usize
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn Contains(&self, target: &EventTargetRef) -> bool {
        self.entries.contains_key(&Self::key(target))
    }

    pub fn Targets(&self) -> impl Iterator<Item = (EventTargetRef, usize)> + '_ {
        self.entries.values().map(|(target, count)| {
            (
                target
                    .upgrade()
                    .expect("DOM must unregister a target before releasing its owner"),
                *count,
            )
        })
    }
}

// Source: event_handler_registry.cc:25-34
pub fn GetEventListenerProperties(
    has_blocking: bool,
    has_passive: bool,
) -> EventListenerProperties {
    if has_blocking && has_passive {
        return EventListenerProperties::kBlockingAndPassive;
    }
    if has_blocking {
        return EventListenerProperties::kBlocking;
    }
    if has_passive {
        return EventListenerProperties::kPassive;
    }
    EventListenerProperties::kNone
}

// Source: event_handler_registry.h:28-37,136-137
pub struct EventHandlerRegistry {
    frame_: RegistryFrameRef,
    targets_: EventHandlerRegistryTargets,
    boundary_: Rc<dyn WheelRegistryBoundary>,
    blocking_wheel_listener_revision_: Cell<u64>,
}

impl EventHandlerRegistry {
    // Source: event_handler_registry.cc:50
    pub fn new(frame: RegistryFrameRef, boundary: Rc<dyn WheelRegistryBoundary>) -> Self {
        Self {
            frame_: frame,
            targets_: EventHandlerRegistryTargets::default(),
            boundary_: boundary,
            blocking_wheel_listener_revision_: Cell::new(0),
        }
    }

    /// Embedding cache identity for the source counted-set membership. Adding
    /// another listener to the same target does not change its hit-test region.
    pub fn BlockingWheelListenerRevision(&self) -> u64 {
        self.blocking_wheel_listener_revision_.get()
    }

    // Source: event_handler_registry.cc:59-95
    pub fn EventTypeToClass(
        event_type: &str,
        options: &AddEventListenerOptions,
    ) -> Option<EventHandlerClass> {
        if event_type == "scroll" {
            Some(EventHandlerClass::kScrollEvent)
        } else if matches!(event_type, "wheel" | "mousewheel") {
            Some(if options.passive() {
                EventHandlerClass::kWheelEventPassive
            } else {
                EventHandlerClass::kWheelEventBlocking
            })
        } else if matches!(event_type, "touchend" | "touchcancel") {
            Some(if options.passive() {
                EventHandlerClass::kTouchEndOrCancelEventPassive
            } else {
                EventHandlerClass::kTouchEndOrCancelEventBlocking
            })
        } else if matches!(event_type, "touchstart" | "touchmove") {
            Some(if options.passive() {
                EventHandlerClass::kTouchStartOrMoveEventPassive
            } else {
                EventHandlerClass::kTouchStartOrMoveEventBlocking
            })
        } else if event_type == "pointerrawupdate" {
            Some(EventHandlerClass::kPointerRawUpdateEvent)
        } else if matches!(
            event_type,
            "gotpointercapture"
                | "lostpointercapture"
                | "pointercancel"
                | "pointerdown"
                | "pointerenter"
                | "pointerleave"
                | "pointermove"
                | "pointerout"
                | "pointerover"
                | "pointerup"
        ) {
            Some(EventHandlerClass::kPointerEvent)
        } else {
            None
        }
    }

    // Source: event_handler_registry.cc:97-101
    pub fn EventHandlerTargets(
        &self,
        handler_class: EventHandlerClass,
    ) -> std::cell::Ref<'_, EventTargetSet> {
        self.CheckConsistency(handler_class);
        self.targets_.classes_[handler_class.index()].borrow()
    }

    // Source: event_handler_registry.cc:103-107
    pub fn HasEventHandlers(&self, handler_class: EventHandlerClass) -> bool {
        self.CheckConsistency(handler_class);
        self.targets_.classes_[handler_class.index()]
            .borrow()
            .size()
            != 0
    }

    // Source: event_handler_registry.cc:109-126
    fn UpdateEventHandlerTargets(
        &self,
        operation: ChangeOperation,
        handler_class: EventHandlerClass,
        target: &EventTargetRef,
    ) {
        self.targets_.Update(operation, handler_class, target);
    }

    // Source: event_handler_registry.cc:128-141
    fn UpdateEventHandlerInternal(
        &self,
        operation: ChangeOperation,
        handler_class: EventHandlerClass,
        target: &EventTargetRef,
    ) -> bool {
        let old_num_handlers = self.targets_.classes_[handler_class.index()]
            .borrow()
            .size();
        self.UpdateEventHandlerTargets(operation, handler_class, target);
        let new_num_handlers = self.targets_.classes_[handler_class.index()]
            .borrow()
            .size();
        let handlers_changed = old_num_handlers != new_num_handlers;
        if handlers_changed && handler_class == EventHandlerClass::kWheelEventBlocking {
            self.blocking_wheel_listener_revision_.set(
                self.blocking_wheel_listener_revision_
                    .get()
                    .checked_add(1)
                    .expect("blocking wheel listener revision exhausted"),
            );
        }
        if operation != ChangeOperation::kRemoveAll && handlers_changed {
            self.NotifyHandlersChanged(target, handler_class, new_num_handlers > 0);
        }
        handlers_changed
    }

    // Source: event_handler_registry.cc:143-152
    fn UpdateEventHandlerOfType(
        &self,
        operation: ChangeOperation,
        event_type: &str,
        options: &AddEventListenerOptions,
        target: &EventTargetRef,
    ) {
        if let Some(handler_class) = Self::EventTypeToClass(event_type, options) {
            self.UpdateEventHandlerInternal(operation, handler_class, target);
        }
    }

    // Source: event_handler_registry.cc:154-159
    pub fn DidAddEventHandler(
        &self,
        target: &EventTargetRef,
        event_type: &str,
        options: &AddEventListenerOptions,
    ) {
        self.UpdateEventHandlerOfType(ChangeOperation::kAdd, event_type, options, target);
    }

    // Source: event_handler_registry.cc:161-166
    pub fn DidRemoveEventHandler(
        &self,
        target: &EventTargetRef,
        event_type: &str,
        options: &AddEventListenerOptions,
    ) {
        self.UpdateEventHandlerOfType(ChangeOperation::kRemove, event_type, options, target);
    }

    // Source: event_handler_registry.cc:168-171
    pub fn DidAddEventHandlerClass(
        &self,
        target: &EventTargetRef,
        handler_class: EventHandlerClass,
    ) {
        self.UpdateEventHandlerInternal(ChangeOperation::kAdd, handler_class, target);
    }

    // Source: event_handler_registry.cc:173-177
    pub fn DidRemoveEventHandlerClass(
        &self,
        target: &EventTargetRef,
        handler_class: EventHandlerClass,
    ) {
        self.UpdateEventHandlerInternal(ChangeOperation::kRemove, handler_class, target);
    }

    // Source: event_handler_registry.cc:179-199
    // The immutable registration view is borrowed from the listener owner.
    // Each event type and its listener order match that owner's EventTypes/
    // GetEventListeners values. No target or listener state is duplicated.
    pub fn DidMoveIntoLocalRoot(
        &self,
        target: &EventTargetRef,
        registrations: &[(&str, &[AddEventListenerOptions])],
    ) {
        for (event_type, listeners) in registrations {
            for options in listeners.iter().rev() {
                if let Some(handler_class) = Self::EventTypeToClass(event_type, options) {
                    self.DidAddEventHandlerClass(target, handler_class);
                }
            }
        }
    }

    // Source: event_handler_registry.cc:201-203
    pub fn DidMoveOutOfLocalRoot(&self, target: &EventTargetRef) {
        self.DidRemoveAllEventHandlers(target);
    }

    // Source: event_handler_registry.cc:205-221
    pub fn DidRemoveAllEventHandlers(&self, target: &EventTargetRef) {
        let handlers_changed = EventHandlerClass::ALL.map(|handler_class| {
            self.UpdateEventHandlerInternal(ChangeOperation::kRemoveAll, handler_class, target)
        });
        for (handler_class, changed) in EventHandlerClass::ALL.into_iter().zip(handlers_changed) {
            if changed {
                let has_handlers = self.targets_.classes_[handler_class.index()]
                    .borrow()
                    .Contains(target);
                self.NotifyHandlersChanged(target, handler_class, has_handlers);
            }
        }
    }

    // Source: event_handler_registry.cc:223-310
    fn NotifyHandlersChanged(
        &self,
        target: &EventTargetRef,
        handler_class: EventHandlerClass,
        _has_active_handlers: bool,
    ) {
        // Source: event_handler_registry.cc:36-46,227
        assert!(matches!(
            target.Kind(),
            EventTargetKind::Node | EventTargetKind::Window
        ));
        let frame = self.boundary_.TargetFacts(target).frame;
        if !self.GetPage() {
            return;
        }
        match handler_class {
            EventHandlerClass::kScrollEvent => {
                self.boundary_
                    .Emit(WheelRegistryEffect::SetHasScrollEventHandlers {
                        frame: frame.clone(),
                        has_handlers: _has_active_handlers,
                    });
            }
            EventHandlerClass::kWheelEventBlocking | EventHandlerClass::kWheelEventPassive => {
                self.boundary_
                    .Emit(WheelRegistryEffect::SetEventListenerProperties {
                        frame: frame.clone(),
                        class: EventListenerClass::kMouseWheel,
                        properties: GetEventListenerProperties(
                            self.HasEventHandlers(EventHandlerClass::kWheelEventBlocking),
                            self.HasEventHandlers(EventHandlerClass::kWheelEventPassive),
                        ),
                    });
            }
            EventHandlerClass::kTouchStartOrMoveEventBlockingLowLatency => {
                self.boundary_
                    .Emit(WheelRegistryEffect::SetNeedsLowLatencyInput {
                        frame: frame.clone(),
                        needed: _has_active_handlers,
                    });
                self.boundary_
                    .Emit(WheelRegistryEffect::SetEventListenerProperties {
                        frame: frame.clone(),
                        class: EventListenerClass::kTouchStartOrMove,
                        properties: GetEventListenerProperties(
                            self.HasEventHandlers(EventHandlerClass::kTouchAction)
                                || self.HasEventHandlers(
                                    EventHandlerClass::kTouchStartOrMoveEventBlocking,
                                )
                                || self.HasEventHandlers(
                                    EventHandlerClass::kTouchStartOrMoveEventBlockingLowLatency,
                                ),
                            self.HasEventHandlers(EventHandlerClass::kTouchStartOrMoveEventPassive)
                                || self.HasEventHandlers(EventHandlerClass::kPointerEvent),
                        ),
                    });
            }
            EventHandlerClass::kTouchAction
            | EventHandlerClass::kTouchStartOrMoveEventBlocking
            | EventHandlerClass::kTouchStartOrMoveEventPassive
            | EventHandlerClass::kPointerEvent => {
                self.boundary_
                    .Emit(WheelRegistryEffect::SetEventListenerProperties {
                        frame: frame.clone(),
                        class: EventListenerClass::kTouchStartOrMove,
                        properties: GetEventListenerProperties(
                            self.HasEventHandlers(EventHandlerClass::kTouchAction)
                                || self.HasEventHandlers(
                                    EventHandlerClass::kTouchStartOrMoveEventBlocking,
                                )
                                || self.HasEventHandlers(
                                    EventHandlerClass::kTouchStartOrMoveEventBlockingLowLatency,
                                ),
                            self.HasEventHandlers(EventHandlerClass::kTouchStartOrMoveEventPassive)
                                || self.HasEventHandlers(EventHandlerClass::kPointerEvent),
                        ),
                    });
            }
            EventHandlerClass::kPointerRawUpdateEvent => {
                self.boundary_
                    .Emit(WheelRegistryEffect::SetEventListenerProperties {
                        frame: frame.clone(),
                        class: EventListenerClass::kPointerRawUpdate,
                        properties: GetEventListenerProperties(
                            false,
                            self.HasEventHandlers(EventHandlerClass::kPointerRawUpdateEvent),
                        ),
                    });
            }
            EventHandlerClass::kTouchEndOrCancelEventBlocking
            | EventHandlerClass::kTouchEndOrCancelEventPassive => {
                self.boundary_
                    .Emit(WheelRegistryEffect::SetEventListenerProperties {
                        frame: frame.clone(),
                        class: EventListenerClass::kTouchEndOrCancel,
                        properties: GetEventListenerProperties(
                            self.HasEventHandlers(
                                EventHandlerClass::kTouchEndOrCancelEventBlocking,
                            ),
                            self.HasEventHandlers(EventHandlerClass::kTouchEndOrCancelEventPassive),
                        ),
                    });
            }
        }
        if matches!(
            handler_class,
            EventHandlerClass::kTouchStartOrMoveEventBlocking
                | EventHandlerClass::kTouchStartOrMoveEventBlockingLowLatency
        ) {
            if let Some(layout) = self.boundary_.TargetFacts(target).invalidation_owner {
                self.boundary_
                    .Emit(WheelRegistryEffect::MarkEffectiveAllowedTouchActionChanged { layout });
            }
        } else if handler_class == EventHandlerClass::kWheelEventBlocking {
            // Re-read the live target facts after the ChromeClient notification,
            // matching the source's possible synchronous reentry.
            if let Some(layout) = self.boundary_.TargetFacts(target).invalidation_owner {
                self.boundary_
                    .Emit(WheelRegistryEffect::MarkBlockingWheelEventHandlerChanged { layout });
            }
        }
    }

    // Source: event_handler_registry.cc:377-400
    fn CheckConsistency(&self, handler_class: EventHandlerClass) {
        #[cfg(debug_assertions)]
        for (target, _) in self.targets_.classes_[handler_class.index()]
            .borrow()
            .Targets()
        {
            let facts = self.boundary_.TargetFacts(&target);
            match target.Kind() {
                EventTargetKind::Node => debug_assert!(facts.document_has_page),
                EventTargetKind::Window => {
                    debug_assert!(facts.frame.is_some());
                    debug_assert!(facts.frame_has_page);
                }
                EventTargetKind::Other => unreachable!("unexpected registry target"),
            }
            debug_assert!(facts
                .local_root
                .as_ref()
                .is_some_and(|root| Rc::ptr_eq(root, &self.frame_)));
        }
        #[cfg(not(debug_assertions))]
        let _ = handler_class;
    }

    // Source: event_handler_registry.cc:402-405
    fn GetPage(&self) -> bool {
        let has_page = self.boundary_.FrameHasPage(&self.frame_);
        debug_assert!(has_page);
        has_page
    }
}

// Source: event_handler_registry.cc:52-57 (wheel classes only)
impl Drop for EventHandlerRegistry {
    fn drop(&mut self) {
        for handler_class in EventHandlerClass::ALL {
            self.CheckConsistency(handler_class);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use interaction_dispatch::core::dom::events::event::EventTarget;

    struct Frame;
    impl RegistryFrameIdentity for Frame {}
    struct Target;
    impl EventTarget for Target {
        fn Kind(&self) -> EventTargetKind {
            EventTargetKind::Node
        }
    }
    struct Boundary(RegistryFrameRef);
    impl WheelRegistryBoundary for Boundary {
        fn FrameHasPage(&self, _: &RegistryFrameRef) -> bool {
            true
        }
        fn TargetFacts(&self, _: &EventTargetRef) -> WheelRegistryTargetFacts {
            WheelRegistryTargetFacts {
                frame: Some(self.0.clone()),
                local_root: Some(self.0.clone()),
                document_has_page: true,
                frame_has_page: true,
                invalidation_owner: None,
            }
        }
        fn Emit(&self, _: WheelRegistryEffect) {}
    }

    #[test]
    fn blocking_wheel_invalidation_tracks_distinct_target_membership() {
        let registry = Rc::new(EventHandlerRegistryTargets::default());
        let changes = Rc::new(RefCell::new(Vec::new()));
        let output = changes.clone();
        registry.SetBlockingWheelInvalidation(Rc::new(move |node| output.borrow_mut().push(node)));
        let target: EventTargetRef = Rc::new(Target);
        let token = registry.BindTargetWithNodeId(&target, Some(42));
        let options = AddEventListenerOptions::default();
        token.DidAddEventHandler("wheel", &options);
        token.DidAddEventHandler("wheel", &options);
        assert_eq!(&*changes.borrow(), &[Some(42)]);
        token.DidRemoveEventHandler("wheel", &options);
        assert!(registry.HasBlockingWheelEventHandlers());
        token.DidRemoveAllEventHandlers();
        assert!(!registry.HasBlockingWheelEventHandlers());
        assert_eq!(&*changes.borrow(), &[Some(42), Some(42)]);
        drop(token);
        assert_eq!(changes.borrow().len(), 2);
    }

    #[test]
    fn blocking_revision_tracks_membership_without_duplicate_registrations() {
        let frame: RegistryFrameRef = Rc::new(Frame);
        let registry = EventHandlerRegistry::new(frame.clone(), Rc::new(Boundary(frame)));
        let a: EventTargetRef = Rc::new(Target);
        let b: EventTargetRef = Rc::new(Target);
        let blocking = EventHandlerClass::kWheelEventBlocking;
        let passive = EventHandlerClass::kWheelEventPassive;
        registry.DidAddEventHandlerClass(&a, passive);
        assert_eq!(registry.BlockingWheelListenerRevision(), 0);
        registry.DidAddEventHandlerClass(&a, blocking);
        registry.DidAddEventHandlerClass(&a, blocking);
        assert_eq!(registry.BlockingWheelListenerRevision(), 1);
        registry.DidRemoveEventHandlerClass(&a, blocking);
        assert_eq!(registry.BlockingWheelListenerRevision(), 1);
        registry.DidAddEventHandlerClass(&b, blocking);
        assert_eq!(registry.BlockingWheelListenerRevision(), 2);
        registry.DidRemoveAllEventHandlers(&a);
        assert_eq!(registry.BlockingWheelListenerRevision(), 3);
        registry.DidMoveOutOfLocalRoot(&b);
        assert_eq!(registry.BlockingWheelListenerRevision(), 4);
        registry.DidRemoveAllEventHandlers(&b);
        assert_eq!(registry.BlockingWheelListenerRevision(), 4);
    }
}
