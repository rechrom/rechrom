//! Narrow Rechrom ports for ordinary synchronous Page event dispatch.
//!
//! The live DOM snapshot and canonical Event are the only path/event owners.
//! WebAPI retains its listener map; the callback receives the phase already
//! selected by EventDispatcher, NodeEventContext, or WindowEventContext.

use super::{DispatchEventResult, EventDispatcher as PageEventDispatcher};
use crate::{
    adapter::event::{EventProjection, EventProjectionOptions, EventTargetResolver},
    default_event_handler::{DefaultActivationState, DefaultEventHandler},
    event::{Event, EventListenerInvocation, EventListenerResult, EventType},
    ownership::{IsElement, ReadNode},
    InteractionEffect, InteractionOutput,
};
use interaction_dispatch::core::{
    dom::events::{
        event::{EventRef, EventTargetRef},
        event_dispatch_result::DispatchEventResult as NativeResult,
        event_dispatcher::{
            DispatchAccessibilityCache, DispatchDocument, DispatchFrameView, DispatchPathAccess,
            DispatchPathRef, EventDispatchHandlingState, EventDispatchHandlingStateRef,
            EventDispatchObserver, EventDispatcher, EventDispatcherContext, NodeDefaultActions,
            ScopedEventQueue, SimulatedEventFactory, SimulatedNodeDefaultActions,
        },
        event_path::LocalEventDispatch,
        event_target::{EventTarget, ExternalEventListenerInvoker},
    },
    events::mouse_event::{MouseEvent, MouseNestedDispatch},
};
use interaction_input::base::time::time::TimeTicks;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

impl super::super::native_interaction::InteractionDOMEvents for PageEventDispatcher<'_, '_> {
    fn State(&self) -> crate::InteractionState {
        *self.state.borrow()
    }

    fn Dispatch(
        &self,
        event: &mut Event,
        target: u64,
        options: &EventProjectionOptions,
    ) -> Result<crate::InteractionResult, super::super::native_interaction::InteractionError> {
        DispatchWithOptions(self, event, target, options)
    }
}

pub(super) fn Dispatch(
    dispatcher: &PageEventDispatcher<'_, '_>,
    event: &mut Event,
    target: u64,
) -> DispatchEventResult {
    let result = DispatchWithOptions(
        dispatcher,
        event,
        target,
        &EventProjectionOptions {
            timestamp: dispatcher.clock.Now(),
            view: Some(dispatcher.targets.WindowView()),
            ..Default::default()
        },
    );
    let Ok(result) = result else {
        return DispatchEventResult::default();
    };
    DispatchEventResult {
        target_node_id: result.target_node_id,
        default_prevented: result.default_prevented,
        propagation_stopped: result.propagation_stopped,
        default_handled: event.default_handled,
    }
}

fn DispatchWithOptions(
    dispatcher: &PageEventDispatcher<'_, '_>,
    event: &mut Event,
    target: u64,
    options: &EventProjectionOptions,
) -> Result<crate::InteractionResult, super::super::native_interaction::InteractionError> {
    let result = super::super::native_interaction::InteractionDOMDispatch::DispatchWithPorts(
        event,
        target,
        dispatcher.targets.clone(),
        options,
        |projection, event, node| {
            let ports = PageDispatchPorts {
                dispatcher,
                projection,
                compatibility: event.clone(),
                handler: DefaultEventHandler::new(dispatcher.Context()),
                activation: RefCell::new(None),
                propagation_stopped: Cell::new(false),
            };
            let hook = projection.DispatchHook(&ports);
            let scoped_events = PageScopedEvents(dispatcher);
            let context = EventDispatcherContext {
                paths: &ports,
                defaults: &ports,
                document: &ports,
                local_events: &ports,
                scoped_queue: Some(&scoped_events),
                simulated_events: Some(&ports),
                simulated_keyboard_events: None,
                simulated_defaults: Some(&ports),
                event_dispatch: hook.as_ref(),
                observer: Some(&ports),
            };
            let native = projection.EventRef();
            if target == 0 {
                projection.AttachPath(event, None);
                ports.DispatchWindowEvent(node, &native);
                ports
                    .propagation_stopped
                    .set(native.borrow().BaseEvent().PropagationStopped());
            } else {
                EventDispatcher::DispatchEvent(node.clone(), native, &context);
            }
            ports.propagation_stopped.get()
        },
    )?;
    if let Some(accumulator) = &dispatcher.input_dispatch_result {
        let previous = accumulator.get();
        accumulator.set(DispatchEventResult {
            target_node_id: result.target_node_id.or(previous.target_node_id),
            default_prevented: previous.default_prevented || result.default_prevented,
            propagation_stopped: previous.propagation_stopped || result.propagation_stopped,
            default_handled: previous.default_handled || event.default_handled,
        });
    }
    Ok(result)
}

// Source event_dispatcher.cc:88-93 assigns the original retained target;
// ScopedEventQueue decides whether this owner is dispatched now or at flush.
pub(super) fn DispatchScoped(
    dispatcher: &PageEventDispatcher<'_, '_>,
    mut event: Event,
    target: u64,
) {
    let Some(node) = dispatcher.targets.ResolveTarget(target) else {
        return;
    };
    event.target_node_id = target;
    let Ok(projection) = EventProjection::new(
        &event,
        dispatcher.targets.clone(),
        &EventProjectionOptions {
            timestamp: dispatcher.clock.Now(),
            view: Some(dispatcher.targets.WindowView()),
            ..Default::default()
        },
    ) else {
        return;
    };
    EventDispatcher::DispatchScopedEventWithQueue(
        &node,
        projection.EventRef(),
        &dispatcher.paths,
        &PageScopedEvents(dispatcher),
    );
}

struct PageScopedEvents<'scope, 'listener, 'frame>(&'scope PageEventDispatcher<'listener, 'frame>);
impl ScopedEventQueue for PageScopedEvents<'_, '_, '_> {
    fn EnqueueEvent(&self, event: EventRef) {
        self.0
            .scoped_queue
            .EnqueueEvent(event, &|event| DispatchQueued(self.0, event));
    }
}

pub(super) fn DispatchQueued(dispatcher: &PageEventDispatcher<'_, '_>, event: EventRef) {
    let target = event
        .borrow()
        .BaseEvent()
        .RawTarget()
        .and_then(|target| dispatcher.targets.NodeId(&target))
        .expect("queued event retains its Page target");
    DispatchCanonical(dispatcher, event, target);
}

/// FormDataEvent retains the mutable native data owner through listener reentry.
pub(super) fn DispatchFormData(
    dispatcher: &PageEventDispatcher<'_, '_>,
    target: u64,
    data: Rc<interaction_dispatch::core::fileapi::form_data_object::FormDataObject>,
) {
    let mut event = interaction_dispatch::core::events::form_data_event::FormDataEvent::new(
        data,
        dispatcher.clock.as_ref(),
    );
    use interaction_dispatch::core::dom::events::event::EventInterface;
    // EventTarget::DispatchEvent's internal (non-bindings) trust assignment.
    event.BaseEventMut().SetTrusted(true);
    DispatchCanonical(dispatcher, EventRef::new(event), target);
}

pub(super) fn DispatchCanonical(
    dispatcher: &PageEventDispatcher<'_, '_>,
    event: EventRef,
    target: u64,
) -> NativeResult {
    DispatchCanonicalWithResult(dispatcher, event, target).0
}

/// CloseWatcher is a direct EventTarget. The existing WebAPI listener owner
/// performs source listener selection against this same canonical EventRef.
pub(super) fn DispatchCloseWatcher(
    dispatcher: &PageEventDispatcher<'_, '_>,
    event: EventRef,
    target: &Rc<interaction_default::core::html::closewatcher::close_watcher::NativeCloseWatcher>,
) -> NativeResult {
    assert!(
        dispatcher.listeners.borrow().is_some() || !target.target.HasEventListeners(),
        "CloseWatcher script listeners require the current scoped runtime"
    );
    struct Listeners<'a, 'listener, 'frame> {
        dispatcher: &'a PageEventDispatcher<'listener, 'frame>,
        id: u64,
    }
    impl ExternalEventListenerInvoker for Listeners<'_, '_, '_> {
        fn FireEventListeners(&self, event: &EventRef) -> NativeResult {
            let listener = self.dispatcher.listeners.borrow().clone();
            if let Some(listener) = listener {
                let projection =
                    EventProjection::FromCanonical(event.clone(), self.dispatcher.targets.clone());
                let mut binding = projection.BindingEvent();
                binding.target_node_id = self.id;
                binding.current_target_node_id = self.id;
                let invocation = EventListenerInvocation {
                    target_node_id: self.id,
                    current_target_node_id: self.id,
                    phase: binding.phase,
                    capture_listeners: binding.capture_listeners,
                    focused_node_id: self.dispatcher.state.borrow().focused_node_id,
                    event: &binding,
                };
                let result = listener(
                    &invocation,
                    &self.dispatcher.document,
                    &self.dispatcher.emit,
                );
                projection.ApplyListenerResult(result);
            }
            EventTarget::GetDispatchEventResult(event.borrow().BaseEvent())
        }
    }
    target.target.DispatchEventWithExternalListeners(
        &event,
        &Listeners {
            dispatcher,
            id: target.host_id,
        },
    )
}

pub(super) fn DispatchCanonicalWithResult(
    dispatcher: &PageEventDispatcher<'_, '_>,
    event: EventRef,
    target: u64,
) -> (NativeResult, DispatchEventResult) {
    let projection = EventProjection::FromCanonical(event, dispatcher.targets.clone());
    let mut binding = projection.BindingEvent();
    // Restore only the old enum projection. The canonical subclass, payload,
    // target and identity remain on the same queued EventRef.
    binding.r#type = match binding.custom_type.as_str() {
        "mousemove" => EventType::kMouseMove,
        "mousedown" => EventType::kMouseDown,
        "mouseup" => EventType::kMouseUp,
        "click" => EventType::kClick,
        "dblclick" => EventType::kDoubleClick,
        "contextmenu" => EventType::kContextMenu,
        "mouseenter" => EventType::kMouseEnter,
        "mouseleave" => EventType::kMouseLeave,
        "mouseover" => EventType::kMouseOver,
        "mouseout" => EventType::kMouseOut,
        "pointermove" => EventType::kPointerMove,
        "pointerdown" => EventType::kPointerDown,
        "pointerup" => EventType::kPointerUp,
        "pointercancel" => EventType::kPointerCancel,
        "pointerenter" => EventType::kPointerEnter,
        "pointerleave" => EventType::kPointerLeave,
        "wheel" => EventType::kWheel,
        "keydown" => EventType::kKeyDown,
        "keypress" => EventType::kKeyPress,
        "keyup" => EventType::kKeyUp,
        "compositionstart" => EventType::kCompositionStart,
        "compositionupdate" => EventType::kCompositionUpdate,
        "compositionend" => EventType::kCompositionEnd,
        "textInput" => EventType::kTextInput,
        "beforeinput" => EventType::kBeforeInput,
        "input" => EventType::kInput,
        "change" => EventType::kChange,
        "focus" => EventType::kFocus,
        "focusin" => EventType::kFocusIn,
        "blur" => EventType::kBlur,
        "focusout" => EventType::kFocusOut,
        "submit" => EventType::kSubmit,
        "reset" => EventType::kReset,
        "DOMContentLoaded" => EventType::kDOMContentLoaded,
        "readystatechange" => EventType::kReadyStateChange,
        "load" => EventType::kLoad,
        _ => EventType::kCustom,
    };
    let result = dispatcher.Dispatch(&mut binding, target);
    let native = if result.default_prevented {
        NativeResult::kCanceledByEventHandler
    } else if result.default_handled {
        NativeResult::kCanceledByDefaultEventHandler
    } else {
        NativeResult::kNotCanceled
    };
    (native, result)
}

pub(super) fn DispatchSimulated(
    dispatcher: &PageEventDispatcher<'_, '_>,
    target: u64,
    underlying: &Event,
) -> Result<(), interaction_dispatch::core::events::simulated_event_util::RequiredSimulatedEventInput>
{
    use interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope;
    let Some(node) = dispatcher.targets.ResolveTarget(target) else {
        return Ok(());
    };
    let native = underlying
        .canonical
        .clone()
        .expect("button default actions retain their canonical triggering Event");
    let projection = EventProjection::FromCanonical(native.clone(), dispatcher.targets.clone());
    let ports = PageDispatchPorts {
        dispatcher,
        projection: &projection,
        compatibility: underlying.clone(),
        handler: DefaultEventHandler::new(dispatcher.Context()),
        activation: RefCell::new(None),
        propagation_stopped: Cell::new(false),
    };
    EventDispatcher::DispatchSimulatedClickWithDispatch(
        node,
        Some(&native),
        SimulatedClickCreationScope::kFromUserAgent,
        &ports,
        &ports,
        &mut |event| DispatchCanonical(dispatcher, event, target),
    )
}

/// AccessKeyAction enters EventDispatcher's source simulated-click path with
/// no underlying event. Keep this separate from control activation, whose
/// triggering Event identity is required above.
// Source: event_dispatcher.cc:95-157; element.cc:3258-3265.
pub(super) fn DispatchSimulatedWithoutUnderlying(
    dispatcher: &PageEventDispatcher<'_, '_>,
    target: u64,
    creation_scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
) -> Result<(), interaction_dispatch::core::events::simulated_event_util::RequiredSimulatedEventInput>
{
    let Some(node) = dispatcher.targets.ResolveTarget(target) else {
        return Ok(());
    };
    let ports = PageSimulatedClickPorts { dispatcher };
    EventDispatcher::DispatchSimulatedClickWithDispatch(
        node,
        None,
        creation_scope,
        &ports,
        &ports,
        &mut |event| DispatchCanonical(dispatcher, event, target),
    )
}

pub(super) fn DispatchDOMActivate(
    dispatcher: &PageEventDispatcher<'_, '_>,
    target: u64,
    underlying: &Event,
) -> NativeResult {
    let Some(node) = dispatcher.targets.ResolveTarget(target) else {
        return NativeResult::kNotCanceled;
    };
    let native = underlying
        .canonical
        .as_ref()
        .expect("DOMActivate retains its canonical underlying event");
    let detail = native
        .borrow()
        .AsUIEventWithKeyState()
        .map_or(0, |event| event.detail());
    let event = interaction_default::core::dom::node::Node::CreateDOMActivateEvent(
        native,
        Some(dispatcher.targets.WindowView()),
        detail,
        dispatcher.clock.as_ref(),
        node.IsConnected(),
        foundation::RuntimeEnabledFeatures::DOMActivateBubblesInheritanceEnabled(),
    );
    EventDispatcher::DispatchScopedEventWithQueue(
        &node,
        event.clone(),
        &dispatcher.paths,
        &PageScopedEvents(dispatcher),
    );
    let result = EventTarget::GetDispatchEventResult(event.borrow().BaseEvent());
    result
}

struct PageDispatchPorts<'scope, 'listener, 'frame> {
    dispatcher: &'scope PageEventDispatcher<'listener, 'frame>,
    projection: &'scope EventProjection,
    compatibility: Event,
    handler: DefaultEventHandler<'scope>,
    // This per-dispatch owner preserves the exact preactivation state across
    // listener reentry. The source token below neither copies nor replaces it.
    activation: RefCell<Option<DefaultActivationState<'scope>>>,
    propagation_stopped: Cell<bool>,
}

impl PageDispatchPorts<'_, '_, '_> {
    fn NodeId(&self, target: &EventTargetRef) -> u64 {
        self.dispatcher
            .targets
            .NodeId(target)
            .expect("retained Page target")
    }
    fn Event(&self) -> Event {
        let mut event = self.compatibility.clone();
        self.projection.SyncTo(&mut event);
        event
    }
    fn Apply(&self, event: &Event) {
        self.projection.ApplyListenerResult(EventListenerResult {
            prevent_default: event.default_prevented,
            stop_propagation: event.propagation_stopped,
            stop_immediate_propagation: event.immediate_propagation_stopped,
        });
        if event.default_handled {
            self.projection
                .EventRef()
                .borrow_mut()
                .BaseEventMut()
                .SetDefaultHandled();
        }
    }
    fn Invoke(&self) {
        // Clone the callback before entering it; no listener slot, DOM, Event,
        // or state borrow survives synchronous JavaScript/nested dispatch.
        let Some(listener) = self.dispatcher.listeners.borrow().clone() else {
            return;
        };
        let event = self.Event();
        let focused_node_id = self.dispatcher.state.borrow().focused_node_id;
        let invocation = EventListenerInvocation {
            target_node_id: event.target_node_id,
            current_target_node_id: event.current_target_node_id,
            phase: event.phase,
            capture_listeners: event.capture_listeners,
            focused_node_id,
            event: &event,
        };
        let result = listener(
            &invocation,
            &self.dispatcher.document,
            &self.dispatcher.emit,
        );
        self.projection.ApplyListenerResult(result);
    }
}

impl DispatchPathAccess for PageDispatchPorts<'_, '_, '_> {
    fn InitEventPath(&self, node: &EventTargetRef, event: &EventRef) -> DispatchPathRef {
        let path = self.dispatcher.paths.InitEventPath(node, event);
        self.projection
            .AttachPath(&self.compatibility, Some(path.clone()));
        path
    }
    fn EventTargetRespectingTargetRules(&self, node: &EventTargetRef) -> EventTargetRef {
        self.dispatcher.paths.EventTargetRespectingTargetRules(node)
    }
}

impl LocalEventDispatch for PageDispatchPorts<'_, '_, '_> {
    fn DispatchWindowEvent(&self, window: &EventTargetRef, event: &EventRef) -> NativeResult {
        EventTarget::DispatchEventAtTarget(event, window.clone(), window.clone(), true, self)
    }
    fn DispatchWindowEventWithTarget(
        &self,
        window: &EventTargetRef,
        event: &EventRef,
        target: &EventTargetRef,
    ) -> NativeResult {
        // LocalDOMWindow::DispatchEvent(event, legacy_target), cc:2371.
        event.borrow_mut().BaseEventMut().SetTrusted(true);
        EventTarget::DispatchEventAtTarget(event, target.clone(), window.clone(), true, self)
    }
    fn HandleNodeLocalEvents(&self, _: &EventTargetRef, _: &EventRef) {
        self.Invoke();
    }
    fn FireWindowEventListeners(&self, _: &EventTargetRef, _: &EventRef) {
        self.Invoke();
    }
    fn DidFinishEvent(&self, _: &EventRef) {}
}
impl ExternalEventListenerInvoker for PageDispatchPorts<'_, '_, '_> {
    fn FireEventListeners(&self, event: &EventRef) -> NativeResult {
        self.Invoke();
        EventTarget::GetDispatchEventResult(event.borrow().BaseEvent())
    }
}

struct ActivationToken;
impl EventDispatchHandlingState for ActivationToken {}
impl NodeDefaultActions for PageDispatchPorts<'_, '_, '_> {
    fn IsHTMLSelectElement(&self, node: &EventTargetRef) -> bool {
        ReadNode(
            &self.dispatcher.document,
            self.NodeId(node),
            |_, node, _| IsElement(node, "select"),
        )
        .unwrap_or(false)
    }
    fn HasActivationBehavior(&self, node: &EventTargetRef) -> bool {
        self.handler.HasActivationBehavior(self.NodeId(node))
    }
    fn LegacyPreActivationBehavior(
        &self,
        node: &EventTargetRef,
        _: &EventRef,
    ) -> Option<EventDispatchHandlingStateRef> {
        let mut event = self.Event();
        let state = self
            .handler
            .LegacyPreActivationBehavior(self.NodeId(node), &mut event);
        self.Apply(&event);
        let token = state
            .as_ref()
            .map(|_| Rc::new(ActivationToken) as EventDispatchHandlingStateRef);
        *self.activation.borrow_mut() = state;
        token
    }
    fn RunActivationBehavior(
        &self,
        node: &EventTargetRef,
        _: &EventRef,
        _: Option<&EventDispatchHandlingStateRef>,
    ) {
        let mut event = self.Event();
        // Take the state out before default actions can reenter this facade.
        let state = self.activation.borrow_mut().take();
        self.handler
            .RunActivationBehavior(self.NodeId(node), &mut event, state.as_ref());
        self.Apply(&event);
    }
    fn DefaultEventHandler(
        &self,
        node: &EventTargetRef,
        native: &EventRef,
        _: Option<&dyn interaction_dispatch::bindings::script_invocation::ScriptInvocation>,
    ) {
        let mut event = self.Event();
        self.handler.Handle(self.NodeId(node), &mut event);
        self.Apply(&event);
        if event.default_handled || event.default_prevented {
            return;
        }
        let raw_target = native
            .borrow()
            .BaseEvent()
            .RawTarget()
            .map(|target| self.NodeId(&target));
        if raw_target != Some(self.NodeId(node)) {
            return;
        }
        if interaction_default::core::dom::node::Node::IsMiddleClickAutoscrollMouseDown(
            foundation::RuntimeEnabledFeatures::MiddleClickAutoscrollEnabled(),
            native.borrow().BaseEvent().r#type(),
            native.borrow().AsMouseEvent().is_some(),
            native.borrow().AsMouseEvent().map(|event| event.button()),
        ) {
            if let Some(start) = self.dispatcher.MiddleClickDefaultHandler() {
                start(self.NodeId(node));
            }
            return;
        }
        // Node.cc:3432-3437 forwards the original TextEvent only at its raw
        // target. Clone the callback before entry to allow listener reentry.
        if native.borrow().AsTextEvent().is_some() {
            if let Some(handler) = self.dispatcher.TextInputDefaultHandler() {
                handler(native);
            }
            return;
        }
        let Some(keyboard) =
            native.Typed::<interaction_dispatch::core::events::keyboard_event::KeyboardEvent>()
        else {
            return;
        };
        // Clone before invoking: listener/default reentry may replace the
        // call-scoped binding but must never conflict with a RefCell borrow.
        let keyboard_default = self.dispatcher.KeyboardDefaultHandler();
        if let Some(handler) = keyboard_default {
            handler(&keyboard, self.NodeId(node));
        }
    }
}

impl DispatchDocument for PageDispatchPorts<'_, '_, '_> {
    fn BeginUIEventTiming(&self, event: &EventRef) -> Box<dyn std::any::Any> {
        use interaction_default::core::timing::event_timing::UIEventTiming;
        // A retired Document may retain its dispatcher after its initial
        // Window was reused. Resolve frame availability on that Document,
        // before consulting the retained Window's now-current environment.
        let performance = self
            .dispatcher
            .document
            .borrow()
            .GetDocument()
            .IsActive()
            .then(|| self.dispatcher.event_timing.clone())
            .flatten();
        Box::new(match performance {
            Some(performance) => UIEventTiming::new(performance, event.clone()),
            None => UIEventTiming::ForAbsentFrame(),
        })
    }
    fn IsMainSequence(&self) -> bool {
        true
    }
    fn View(&self, _: &EventTargetRef) -> Option<Rc<dyn DispatchFrameView>> {
        None
    }
    // The old Page endpoint does not provide listener-interest summaries. Do
    // not skip capture: WebAPI performs its own registration selection.
    fn HasCaptureListener(&self, _: &EventTargetRef) -> bool {
        true
    }
    fn SkipEventCaptureEnabled(&self) -> bool {
        false
    }
    fn WideViewportQuirkEnabled(&self, _: &EventTargetRef) -> Option<bool> {
        None
    }
    fn ExistingAXObjectCache(
        &self,
        _: &EventTargetRef,
    ) -> Option<Rc<dyn DispatchAccessibilityCache>> {
        None
    }
    fn ClearPopoverPickerPointerdown(&self, _: &EventTargetRef) {}
}
impl EventDispatchObserver for PageDispatchPorts<'_, '_, '_> {
    fn DidFinishPropagation(&self, event: &EventRef) {
        self.propagation_stopped
            .set(event.borrow().BaseEvent().PropagationStopped());
    }
    fn DidDispatch(&self, event: &EventRef) {
        let target = {
            let event = event.borrow();
            let base = event.BaseEvent();
            if base.r#type() != "click"
                || !base.isTrusted()
                || base.defaultPrevented()
                || event.AsMouseEvent().is_none_or(|mouse| mouse.button() != 0)
            {
                return;
            }
            base.RawTarget().map(|target| self.NodeId(&target))
        };
        if let (Some(target_node_id), Some(emit)) =
            (target, self.dispatcher.interaction_output.as_ref())
        {
            emit(InteractionOutput::Effect(
                InteractionEffect::MouseClickActivation { target_node_id },
            ));
        }
    }
}
impl MouseNestedDispatch for PageDispatchPorts<'_, '_, '_> {
    fn DispatchEvent(&self, node: EventTargetRef, event: MouseEvent) -> NativeResult {
        let projection =
            EventProjection::FromCanonical(EventRef::new(event), self.dispatcher.targets.clone());
        let mut binding = projection.BindingEvent();
        let result = self.dispatcher.Dispatch(&mut binding, self.NodeId(&node));
        if result.default_prevented {
            NativeResult::kCanceledByEventHandler
        } else if result.default_handled {
            NativeResult::kCanceledByDefaultEventHandler
        } else {
            NativeResult::kNotCanceled
        }
    }
    fn Now(&self) -> TimeTicks {
        self.dispatcher.clock.Now()
    }
}

/// Source simulated_event_util.cc:41-76. Geometry is borrowed from the
/// existing FrameView/LayoutBox provider at each event's construction point.
fn AccessibilityCoordinates(
    dispatcher: &PageEventDispatcher<'_, '_>,
    node: &EventTargetRef,
    scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
) -> Result<
    Option<interaction_dispatch::core::events::simulated_event_util::AccessibilityCoordinates>,
    interaction_dispatch::core::events::simulated_event_util::RequiredSimulatedEventInput,
> {
    use interaction_dispatch::core::{
        dom::events::simulated_click_options::SimulatedClickCreationScope,
        events::simulated_event_util::{AccessibilityCoordinates, RequiredSimulatedEventInput},
    };
    if scope != SimulatedClickCreationScope::kFromAccessibility {
        return Ok(None);
    }
    let Some(id) = dispatcher.targets.NodeId(node) else {
        return Ok(Some(AccessibilityCoordinates::NoLayoutBox));
    };
    let is_element = ReadNode(&dispatcher.document, id, |_, node, _| {
        node.Type() == dom::persistent_document::DOMNodeType::kElement
    })
    .unwrap_or(false);
    if !is_element {
        return Ok(Some(AccessibilityCoordinates::NoLayoutBox));
    }
    // Clone the provider before querying: no facade RefCell borrow crosses
    // the owner query or a subsequent synchronous event dispatch.
    let frame = dispatcher
        .mouse_frame_geometry
        .borrow()
        .clone()
        .ok_or(RequiredSimulatedEventInput::AccessibilityCoordinates)?;
    interaction_dispatch::core::events::simulated_event_util::PrepareAccessibilityCoordinates(
        &PageSimulatedClickGeometry {
            frame: frame.as_ref(),
            node: id,
        },
    )
    .map(Some)
}

struct PageSimulatedClickGeometry<'a> {
    frame: &'a dyn super::MouseFrameGeometry,
    node: u64,
}
impl interaction_dispatch::core::events::simulated_event_util::AccessibilityGeometry
    for PageSimulatedClickGeometry<'_>
{
    fn RootFrameBorderBoxCenter(&self) -> Option<(foundation::PhysicalOffset, f32)> {
        self.frame.SimulatedClickRootFrameCenter(self.node)
    }
    fn ConvertFromRootFrame(
        &self,
        point: foundation::PhysicalOffset,
    ) -> Option<foundation::PhysicalOffset> {
        let converted = self.frame.ConvertFromRootFrame(
            interaction_input::public::common::input::web_pointer_properties::PointF::new(
                point.left.ToFloat(),
                point.top.ToFloat(),
            ),
        )?;
        Some(foundation::PhysicalOffset::new(
            foundation::LayoutUnit::from_f64(f64::from(converted.x())),
            foundation::LayoutUnit::from_f64(f64::from(converted.y())),
        ))
    }
    fn FrameToScreen(&self, rect: foundation::gfx::Rect) -> Option<foundation::gfx::Rect> {
        let root_rect = self.frame.NonLocatedRectToRootFrame(rect)?;
        Some(
            self.frame
                .NonLocatedContextMenuCoordinates()?
                .RootFrameRectToScreenDIPs(root_rect),
        )
    }
}

impl SimulatedEventFactory for PageDispatchPorts<'_, '_, '_> {
    fn CreateEvent(
        &self,
        event_type: &'static str,
        node: &EventTargetRef,
        underlying_event: Option<&EventRef>,
        creation_scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    ) -> Result<
        EventRef,
        interaction_dispatch::core::events::simulated_event_util::RequiredSimulatedEventInput,
    > {
        use interaction_dispatch::core::events::simulated_event_util::{
            SimulatedEventEnvironment, SimulatedEventUtil,
        };
        let coordinates = underlying_event
            .and_then(|event| {
                event
                    .borrow()
                    .AsMouseEvent()
                    .map(|mouse| mouse.CoordinateContext())
            })
            .unwrap_or_else(|| self.dispatcher.MouseCoordinateContext());
        SimulatedEventUtil::CreateEvent(
            event_type,
            node,
            underlying_event,
            creation_scope,
            &SimulatedEventEnvironment {
                view: Some(self.dispatcher.targets.WindowView()),
                non_touch_capabilities: Some(
                    self.dispatcher.targets.InputDeviceCapabilities(false),
                ),
                coordinates,
                now: underlying_event
                    .is_none()
                    .then(|| self.dispatcher.clock.Now()),
                accessibility_coordinates: AccessibilityCoordinates(
                    self.dispatcher,
                    node,
                    creation_scope,
                )?,
            },
        )
    }
}

struct PageSimulatedClickPorts<'scope, 'listener, 'frame> {
    dispatcher: &'scope PageEventDispatcher<'listener, 'frame>,
}

impl PageSimulatedClickPorts<'_, '_, '_> {
    fn NodeId(&self, node: &EventTargetRef) -> u64 {
        self.dispatcher
            .targets
            .NodeId(node)
            .expect("simulated click target belongs to the current Page")
    }
}

impl SimulatedNodeDefaultActions for PageSimulatedClickPorts<'_, '_, '_> {
    fn IsDisabledFormControl(&self, node: &EventTargetRef) -> bool {
        crate::input_type::IsDisabledFormControl(&self.dispatcher.document, self.NodeId(node))
    }

    fn IsElement(&self, node: &EventTargetRef) -> bool {
        ReadNode(
            &self.dispatcher.document,
            self.NodeId(node),
            |_, node, _| node.Type() == dom::persistent_document::DOMNodeType::kElement,
        )
        .unwrap_or(false)
    }

    fn SetActive(&self, node: &EventTargetRef, active: bool) {
        let id = self.NodeId(node);
        let mut state = self.dispatcher.state.borrow_mut();
        if active {
            state.pressed_node_id = Some(id);
        } else if state.pressed_node_id == Some(id) {
            state.pressed_node_id = None;
        }
    }
}

impl SimulatedEventFactory for PageSimulatedClickPorts<'_, '_, '_> {
    fn CreateEvent(
        &self,
        event_type: &'static str,
        node: &EventTargetRef,
        underlying_event: Option<&EventRef>,
        creation_scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    ) -> Result<
        EventRef,
        interaction_dispatch::core::events::simulated_event_util::RequiredSimulatedEventInput,
    > {
        use interaction_dispatch::core::events::simulated_event_util::{
            SimulatedEventEnvironment, SimulatedEventUtil,
        };
        debug_assert!(underlying_event.is_none());
        SimulatedEventUtil::CreateEvent(
            event_type,
            node,
            None,
            creation_scope,
            &SimulatedEventEnvironment {
                view: Some(self.dispatcher.targets.WindowView()),
                non_touch_capabilities: Some(
                    self.dispatcher.targets.InputDeviceCapabilities(false),
                ),
                coordinates: self.dispatcher.MouseCoordinateContext(),
                now: Some(self.dispatcher.clock.Now()),
                accessibility_coordinates: AccessibilityCoordinates(
                    self.dispatcher,
                    node,
                    creation_scope,
                )?,
            },
        )
    }
}
impl SimulatedNodeDefaultActions for PageDispatchPorts<'_, '_, '_> {
    fn IsDisabledFormControl(&self, node: &EventTargetRef) -> bool {
        crate::input_type::IsDisabledFormControl(&self.dispatcher.document, self.NodeId(node))
    }
    fn IsElement(&self, node: &EventTargetRef) -> bool {
        ReadNode(
            &self.dispatcher.document,
            self.NodeId(node),
            |_, node, _| node.Type() == dom::persistent_document::DOMNodeType::kElement,
        )
        .unwrap_or(false)
    }
    fn SetActive(&self, node: &EventTargetRef, active: bool) {
        let id = self.NodeId(node);
        let mut state = self.dispatcher.state.borrow_mut();
        if active {
            state.pressed_node_id = Some(id);
        } else if state.pressed_node_id == Some(id) {
            state.pressed_node_id = None;
        }
    }
}
