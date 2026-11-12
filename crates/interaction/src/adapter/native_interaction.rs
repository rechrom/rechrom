//! Page assembly for the translated interaction owners.
//!
//! This adapter deliberately has no hit-test, focus, editing, propagation, or
//! default-action algorithm. Page supplies the live source owners, including an
//! EventHandler constructed with its original manager factory. Clones retain
//! those owners; no mutable facade borrow survives a synchronous callback.
//!
//! The old `WithSelectionState` constructor is not sufficient for this path.
//! Page must assemble the EventHandler frame/factory, canonical DOM event
//! targets/path/default/listener ports, its DOM state owner and existing Editor,
//! then pass `InteractionAssembly` to `Interaction::new`.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use interaction_dispatch::core::dom::events::event::{EventRef, EventTargetRef};
use interaction_dispatch::core::dom::events::event_dispatcher::{
    DispatchDocument, DispatchPathAccess, DispatchPathRef, EventDispatchObserver, EventDispatcher,
    EventDispatcherContext, NodeDefaultActions, ScopedEventQueue, SimulatedEventFactory,
    SimulatedNodeDefaultActions,
};
use interaction_dispatch::core::dom::events::event_path::LocalEventDispatch;
use interaction_dispatch::core::events::mouse_event::MouseNestedDispatch;
use interaction_dispatch::public::platform::web_input_event_result::WebInputEventResult;
use interaction_input::base::time::time::TimeTicks;
use interaction_input::platform::widget::input::widget_base_input_handler::GetCoalescedWebPointerEventForTouch;
use interaction_input::public::common::input::web_input_event::WebInputEventPayload;
use interaction_input::public::common::input::web_input_event::{Type, WebInputEvent};
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;
use interaction_input::public::common::input::web_mouse_wheel_event::WebMouseWheelEvent;
use interaction_input::public::common::input::web_pointer_event::TouchState;
use interaction_input::public::common::input::web_pointer_event::WebPointerEvent;
use interaction_input::public::common::input::web_pointer_properties::PointerType;

use super::event::{
    Event, EventDispatchState, EventProjection, EventProjectionOptions, EventTargetResolver,
    UnresolvedEventTarget,
};
use super::frame_aligned_input_queue::BorrowedInputPacket;
use super::input_event::{
    CompositionEvent, ConvertedWebInputEvent, FocusEvent, InputEvent, TextInputEvent,
    WebInputConversionError, WheelEvent,
};
use super::interaction_contract::{InteractionOutputEmitter, InteractionResult, InteractionState};
use crate::core::frame::web_frame_widget_impl::FocusedPluginKeyboardState;
use crate::core::input::event_handler::{EventHandler, EventHandlerHitResult};
use interaction_input::public::common::input::web_keyboard_event::WebKeyboardEvent;

/// An absent source path is an integration error, never a successful no-op.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionPort {
    Wheel,
    InputMethod,
    Focus,
    FormSubmission,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InteractionError {
    SourceInput(String),
    MissingPort(InteractionPort),
    Conversion(WebInputConversionError),
    UnresolvedTarget(u64),
    /// Raw source input uses hit testing/focus/capture, not a host target hint.
    TargetedRawInput(u64),
    InvalidPacket(&'static str),
    DetachedFrame,
    LifecycleNotReady,
    InvalidFormSubmission(
        interaction_default::core::html::forms::html_form_element::RequestSubmitError,
    ),
}

impl std::fmt::Display for InteractionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceInput(message) => formatter.write_str(message),
            Self::MissingPort(port) => write!(formatter, "missing interaction port: {port:?}"),
            _ => write!(formatter, "{self:?}"),
        }
    }
}
impl std::error::Error for InteractionError {}

impl From<WebInputConversionError> for InteractionError {
    fn from(error: WebInputConversionError) -> Self {
        Self::Conversion(error)
    }
}

impl From<UnresolvedEventTarget> for InteractionError {
    fn from(error: UnresolvedEventTarget) -> Self {
        Self::UnresolvedTarget(error.0)
    }
}

/// Raw input does not expose a DOM target or propagation flags. Do not infer
/// those facts from WebInputEventResult's handled/canceled disposition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionInputResult {
    WebInput(WebInputEventResult),
    DOM(InteractionResult),
}

/// Reads DOM's canonical user-action state. This adapter retains no state copy.
pub trait InteractionStateOwner {
    fn State(&self) -> InteractionState;
}

/// A call-scoped view of the existing DOM dispatch owner. Embeddings may lend
/// their prepared local/default/listener ports through this interface; event
/// identity and listener registrations remain with their original owners.
pub trait InteractionDOMEvents {
    fn State(&self) -> InteractionState;
    fn Dispatch(
        &self,
        event: &mut Event,
        target: u64,
        options: &EventProjectionOptions,
    ) -> Result<InteractionResult, InteractionError>;
}

/// Fallible Page ports retain the first source-leaf failure while satisfying
/// Chromium's infallible manager signatures. Drain that failure before the
/// input transaction returns, including source early-return branches.
pub trait InteractionInputCompletion {
    fn Finish(&self) -> Result<(), InteractionError>;
}

/// Widget ingress facts only. Coordinate transforms use the live FrameView;
/// NotifyInput includes the source queued-timestamp bookkeeping. These methods
/// must not run an alternative interaction/default-action implementation.
pub trait InteractionHost {
    /// WidgetEventHandler suppresses detached mouse input and returns
    /// NotHandled for detached wheel/pointer input before entering a manager.
    fn HasRootView(&self) -> bool;
    fn HasLocalRoot(&self) -> bool;
    fn HandlePopupKeyEvent(
        &self,
        event: &WebKeyboardEvent,
    ) -> Result<Option<WebInputEventResult>, InteractionError>;
    fn PagePopupCopyPasteEnabled(&self) -> bool;
    fn FocusedPluginKeyboardState(&self) -> Result<FocusedPluginKeyboardState, InteractionError>;
    /// CurrentInputEvent is scoped to the synchronous ingress transaction,
    /// including listener reentry; the returned native guard restores nesting.
    fn EnterCurrentInputEvent(
        &self,
        event: &ConvertedWebInputEvent,
    ) -> Result<Box<dyn std::any::Any>, InteractionError>;
    fn InputTimestamp(&self, input: &InputEvent) -> Result<TimeTicks, InteractionError>;
    fn NotifyInput(&self, event: &dyn WebInputEventPayload) -> Result<(), InteractionError>;
    /// WidgetEventHandler's accessibility hover precedes its input switch.
    fn TouchAccessibilityHover(&self, event: &WebMouseEvent) -> Result<(), InteractionError>;
    fn TransformMouseEvent(&self, event: &WebMouseEvent)
        -> Result<WebMouseEvent, InteractionError>;
    fn TransformPointerEvent(
        &self,
        event: &WebPointerEvent,
    ) -> Result<WebPointerEvent, InteractionError>;
    fn TransformWheelEvent(
        &self,
        event: &WebMouseWheelEvent,
    ) -> Result<WebMouseWheelEvent, InteractionError>;
    fn TransformGestureEvent(
        &self,
        event: &super::input_event::WebGestureEvent,
    ) -> Result<super::input_event::WebGestureEvent, InteractionError>;
}

pub trait InteractionWidgetKeyboardInput {
    fn HandleKeyboardEvent(
        &self,
        event: &WebKeyboardEvent,
    ) -> Result<WebInputEventResult, InteractionError>;
}

/// The scroll package's original MouseWheelEventManager/ScrollManager owners.
/// Native/compositor metadata stays separate from the raw Web event.
pub trait InteractionWheelInput {
    fn HandleWheelEvent(
        &self,
        event: &WebMouseWheelEvent,
        host_event: &WheelEvent,
        output: &InteractionOutputEmitter,
    ) -> Result<WebInputEventResult, InteractionError>;
}

/// InputMethodController and Editor entry points, using their existing shared
/// selection/composition owners. A keydown is never substituted for IME input.
pub trait InteractionInputMethod {
    fn HandleComposition(
        &self,
        event: &CompositionEvent,
        output: &InteractionOutputEmitter,
    ) -> Result<InteractionInputResult, InteractionError>;
    fn HandleTextInput(
        &self,
        event: &TextInputEvent,
        output: &InteractionOutputEmitter,
    ) -> Result<InteractionInputResult, InteractionError>;
}

pub trait InteractionFocusInput {
    fn HandleFocus(
        &self,
        event: &FocusEvent,
        output: &InteractionOutputEmitter,
    ) -> Result<InteractionResult, InteractionError>;
    fn HandleFocusWithLocalEvents(
        &self,
        event: &FocusEvent,
        output: &InteractionOutputEmitter,
        local_events: &dyn LocalEventDispatch,
    ) -> Result<InteractionResult, InteractionError>;
}

pub trait InteractionFormSubmission {
    fn SubmitForm(
        &self,
        form: u64,
        submitter: Option<u64>,
        dispatch_event: bool,
        output: &InteractionOutputEmitter,
    ) -> Result<(), InteractionError>;
    fn SubmitFormWithLocalEvents(
        &self,
        form: u64,
        submitter: Option<u64>,
        dispatch_event: bool,
        output: &InteractionOutputEmitter,
        local_events: &dyn LocalEventDispatch,
    ) -> Result<(), InteractionError>;
}

/// Distinct ports retain the same owners used by native manager-generated
/// events. Their effects must use the assembly's InteractionOutputEmitter.
pub struct InteractionDOMAssembly<'scope> {
    pub targets: Rc<dyn EventTargetResolver>,
    pub paths: Rc<dyn DispatchPathAccess>,
    pub defaults: Rc<dyn NodeDefaultActions + 'scope>,
    pub document: Rc<dyn DispatchDocument + 'scope>,
    pub local_events: Rc<dyn LocalEventDispatch + 'scope>,
    pub scoped_queue: Rc<dyn ScopedEventQueue + 'scope>,
    pub simulated_events: Rc<dyn SimulatedEventFactory + 'scope>,
    pub simulated_defaults: Option<Rc<dyn SimulatedNodeDefaultActions + 'scope>>,
    pub nested_mouse: Rc<dyn MouseNestedDispatch + 'scope>,
    pub state: Rc<dyn InteractionStateOwner>,
}

pub struct InteractionDOMDispatch<'scope> {
    ports: InteractionDOMAssembly<'scope>,
}

impl InteractionDOMEvents for InteractionDOMDispatch<'_> {
    fn State(&self) -> InteractionState {
        self.State()
    }
    fn Dispatch(
        &self,
        event: &mut Event,
        target: u64,
        options: &EventProjectionOptions,
    ) -> Result<InteractionResult, InteractionError> {
        self.Dispatch(event, target, options)
    }
}

impl<'scope> InteractionDOMDispatch<'scope> {
    pub fn new(ports: InteractionDOMAssembly<'scope>) -> Self {
        Self { ports }
    }

    pub fn State(&self) -> InteractionState {
        self.ports.state.State()
    }

    pub fn Dispatch(
        &self,
        event: &mut Event,
        target: u64,
        options: &EventProjectionOptions,
    ) -> Result<InteractionResult, InteractionError> {
        self.DispatchWithLocalEvents(event, target, options, self.ports.local_events.as_ref())
    }

    /// Borrows the binding's current runtime invocation scope. Source dispatch,
    /// path, activation and default processing still use this same assembly.
    pub fn DispatchWithLocalEvents(
        &self,
        event: &mut Event,
        target: u64,
        options: &EventProjectionOptions,
        local_events: &dyn LocalEventDispatch,
    ) -> Result<InteractionResult, InteractionError> {
        Self::DispatchWithPorts(
            event,
            target,
            self.ports.targets.clone(),
            options,
            |projection, event, target_owner| {
                let path_access = ProjectionPathAccess {
                    source: self.ports.paths.as_ref(),
                    projection,
                    compatibility: event,
                };
                let observer = PropagationSnapshot(Cell::new(false));
                let hook = projection.DispatchHook(self.ports.nested_mouse.as_ref());
                let context = EventDispatcherContext {
                    paths: &path_access,
                    defaults: self.ports.defaults.as_ref(),
                    document: self.ports.document.as_ref(),
                    local_events,
                    scoped_queue: Some(self.ports.scoped_queue.as_ref()),
                    simulated_events: Some(self.ports.simulated_events.as_ref()),
                    simulated_keyboard_events: None,
                    simulated_defaults: self.ports.simulated_defaults.as_deref(),
                    event_dispatch: hook.as_ref(),
                    observer: Some(&observer),
                };
                let canonical = projection.EventRef();
                if target == 0 {
                    projection.AttachPath(event, None);
                    local_events.DispatchWindowEvent(target_owner, &canonical);
                    observer.DidFinishPropagation(&canonical);
                } else {
                    EventDispatcher::DispatchEvent(target_owner.clone(), canonical, &context);
                }
                observer.0.get()
            },
        )
    }

    /// Owns the canonical Event projection and its synchronous dispatch scope.
    /// The caller borrows its existing dispatch/default/listener ports only
    /// for this transaction. WebAPI's EventTarget keeps listener registration
    /// and selection; no second listener map or projection registry is needed.
    pub(crate) fn DispatchWithPorts(
        event: &mut Event,
        target: u64,
        targets: Rc<dyn EventTargetResolver>,
        options: &EventProjectionOptions,
        dispatch: impl FnOnce(&EventProjection, &Event, &EventTargetRef) -> bool,
    ) -> Result<InteractionResult, InteractionError> {
        let target_owner = targets
            .ResolveTarget(target)
            .ok_or(InteractionError::UnresolvedTarget(target))?;
        event.target_node_id = target;
        let projection = EventProjection::new(event, targets, options)?;
        event.canonical = Some(projection.EventRef());
        let _cleanup = ProjectionCleanup(event.dispatch_state.clone());
        let propagation_stopped = dispatch(&projection, event, &target_owner);
        projection.Finish(event);
        // Materialize the binding-visible result before the cleanup guard is
        // dropped. A Ref returned directly from the result expression would
        // otherwise live through local destruction and collide with the
        // guard's mutable cleanup borrow.
        let target_node_id = event.dispatch_state.borrow().final_target;
        Ok(InteractionResult {
            target_node_id,
            default_prevented: event.default_prevented,
            propagation_stopped,
            input_result: None,
        })
    }
}

// A retained compatibility event must not keep a dispatch path alive if its
// listener unwinds. This owns only the compatibility view, never the canonical
// Event or source propagation state. The ordinary exit still uses Finish.
struct ProjectionCleanup(Rc<RefCell<EventDispatchState>>);

impl Drop for ProjectionCleanup {
    fn drop(&mut self) {
        let mut state = self.0.borrow_mut();
        state.path = None;
        state.finished = true;
    }
}

struct ProjectionPathAccess<'a> {
    source: &'a dyn DispatchPathAccess,
    projection: &'a EventProjection,
    compatibility: &'a Event,
}

impl DispatchPathAccess for ProjectionPathAccess<'_> {
    fn InitEventPath(&self, node: &EventTargetRef, event: &EventRef) -> DispatchPathRef {
        let path = self.source.InitEventPath(node, event);
        self.projection
            .AttachPath(self.compatibility, Some(path.clone()));
        path
    }

    fn EventTargetRespectingTargetRules(&self, node: &EventTargetRef) -> EventTargetRef {
        self.source.EventTargetRespectingTargetRules(node)
    }
}

struct PropagationSnapshot(Cell<bool>);
impl EventDispatchObserver for PropagationSnapshot {
    fn DidFinishPropagation(&self, event: &EventRef) {
        self.0.set(event.borrow().BaseEvent().PropagationStopped());
    }
    fn DidDispatch(&self, _: &EventRef) {}
}

/// All fields are existing live owners. E is the embedding's original Editor
/// type (the retained translated Editor plus its small geometry query facade),
/// never a copied selection state.
pub struct InteractionAssembly<'scope, R: EventHandlerHitResult, E: ?Sized> {
    pub handler: Rc<EventHandler<'scope, R>>,
    pub host: Rc<dyn InteractionHost + 'scope>,
    pub editor: Rc<E>,
    pub dom_events: Rc<dyn InteractionDOMEvents + 'scope>,
    pub output: InteractionOutputEmitter,
    pub completion: Rc<dyn InteractionInputCompletion + 'scope>,
    pub keyboard: Rc<dyn InteractionWidgetKeyboardInput + 'scope>,
    pub wheel: Option<Rc<dyn InteractionWheelInput + 'scope>>,
    pub input_method: Option<Rc<dyn InteractionInputMethod + 'scope>>,
    pub focus: Option<Rc<dyn InteractionFocusInput + 'scope>>,
    pub forms: Option<Rc<dyn InteractionFormSubmission + 'scope>>,
}

pub struct Interaction<'scope, R: EventHandlerHitResult, E: ?Sized> {
    owners: Rc<InteractionAssembly<'scope, R, E>>,
}

impl<R: EventHandlerHitResult, E: ?Sized> Clone for Interaction<'_, R, E> {
    fn clone(&self) -> Self {
        Self {
            owners: self.owners.clone(),
        }
    }
}

impl<'scope, R: EventHandlerHitResult, E: ?Sized> Interaction<'scope, R, E> {
    pub fn new(assembly: InteractionAssembly<'scope, R, E>) -> Self {
        Self {
            owners: Rc::new(assembly),
        }
    }

    pub fn State(&self) -> InteractionState {
        self.owners.dom_events.State()
    }

    pub fn EventHandler(&self) -> &Rc<EventHandler<'scope, R>> {
        &self.owners.handler
    }

    pub fn OutputEmitter(&self) -> InteractionOutputEmitter {
        self.owners.output.clone()
    }

    pub fn Dispatch(&self, input: &InputEvent) -> Result<InteractionInputResult, InteractionError> {
        self.DispatchInput(input, None)
    }

    /// Dispatches the queue's canonical packet. In particular, do not round-trip
    /// queued timestamps, dispatch type, transformed coordinates or predicted
    /// samples through the host-facing InputEvent projection.
    pub fn DispatchPacket(
        &self,
        input: &InputEvent,
        packet: BorrowedInputPacket<'_>,
    ) -> Result<InteractionInputResult, InteractionError> {
        self.DispatchInput(input, Some(packet))
    }

    fn DispatchInput(
        &self,
        input: &InputEvent,
        packet: Option<BorrowedInputPacket<'_>>,
    ) -> Result<InteractionInputResult, InteractionError> {
        let result = self.DispatchInputInternal(input, packet);
        self.owners.completion.Finish()?;
        result
    }

    fn DispatchInputInternal(
        &self,
        input: &InputEvent,
        packet: Option<BorrowedInputPacket<'_>>,
    ) -> Result<InteractionInputResult, InteractionError> {
        let owners = self.owners.as_ref();
        match input {
            InputEvent::Focus(event) => {
                return owners
                    .focus
                    .as_ref()
                    .ok_or(InteractionError::MissingPort(InteractionPort::Focus))?
                    .HandleFocus(event, &owners.output)
                    .map(InteractionInputResult::DOM);
            }
            InputEvent::Composition(event) => {
                return owners
                    .input_method
                    .as_ref()
                    .ok_or(InteractionError::MissingPort(InteractionPort::InputMethod))?
                    .HandleComposition(event, &owners.output);
            }
            InputEvent::TextInput(event) => {
                return owners
                    .input_method
                    .as_ref()
                    .ok_or(InteractionError::MissingPort(InteractionPort::InputMethod))?
                    .HandleTextInput(event, &owners.output);
            }
            _ => {}
        }
        let target = match input {
            InputEvent::Mouse(event) => event.target_node_id,
            InputEvent::Pointer(event) => event.target_node_id,
            InputEvent::Touch(_) | InputEvent::Gesture(_) => None,
            InputEvent::Wheel(event) => event.target_node_id,
            InputEvent::Key(event) => event.target_node_id,
            _ => unreachable!("host-only inputs returned above"),
        };
        if let Some(target) = target {
            return Err(InteractionError::TargetedRawInput(target));
        }
        let converted = if let Some(packet) = packet {
            ConvertedPacket(input, packet)?
        } else {
            input.ToWebEvent(owners.host.InputTimestamp(input)?)?
        };
        let timestamp = ConvertedTimestamp(&converted);
        let _current_input = owners.host.EnterCurrentInputEvent(&converted)?;
        let result = match converted {
            ConvertedWebInputEvent::Gesture(event) => {
                owners.host.NotifyInput(&event)?;
                let event = owners.host.TransformGestureEvent(&event)?;
                if event.IsScrollEvent() {
                    return Err(InteractionError::SourceInput(
                        "gesture scroll requires compositor routing".into(),
                    ));
                }
                owners.handler.HandleGestureEvent(&event)
            }
            ConvertedWebInputEvent::Mouse(event) => {
                owners.host.NotifyInput(&event)?;
                if event.GetModifiers() & WebInputEvent::kIsTouchAccessibility != 0 {
                    owners.host.TouchAccessibilityHover(&event)?;
                }
                // widget_event_handler.cc:57-143 has no mouse-enter branch.
                if event.GetType() == Type::kMouseEnter {
                    return Ok(InteractionInputResult::WebInput(
                        WebInputEventResult::kNotHandled,
                    ));
                }
                if !owners.host.HasRootView() {
                    return Ok(InteractionInputResult::WebInput(
                        WebInputEventResult::kHandledSuppressed,
                    ));
                }
                // WidgetEventHandler::HandleMouseDown cc:179-188 runs this
                // cancellation before the FrameView coordinate transform.
                if event.GetType() == Type::kMouseDown
                    && crate::core::input::widget_event_handler::IsDoubleAltClick(&event)
                {
                    owners.handler.CancelDelayedNavigationTask();
                }
                let event = owners.host.TransformMouseEvent(&event)?;
                match event.GetType() {
                    Type::kContextMenu => owners.handler.SendContextMenuEvent(&event, None),
                    Type::kMouseDown => {
                        owners.handler.HandleMousePressEvent(&event);
                        WebInputEventResult::kHandledSystem
                    }
                    Type::kMouseUp => owners.handler.HandleMouseReleaseEvent(&event),
                    Type::kMouseMove => {
                        let coalesced = MouseSamples(packet, true, owners.host.as_ref())?;
                        let predicted = MouseSamples(packet, false, owners.host.as_ref())?;
                        owners
                            .handler
                            .HandleMouseMoveEvent(&event, &coalesced, &predicted);
                        WebInputEventResult::kHandledSystem
                    }
                    Type::kMouseLeave => {
                        owners.handler.HandleMouseLeaveEvent(&event);
                        WebInputEventResult::kHandledSystem
                    }
                    _ => unreachable!("mouse adapter only produces the checked source types"),
                }
            }
            ConvertedWebInputEvent::Pointer(event) => {
                owners.host.NotifyInput(&event)?;
                if !owners.host.HasRootView() {
                    return Ok(InteractionInputResult::WebInput(
                        WebInputEventResult::kNotHandled,
                    ));
                }
                let event = owners.host.TransformPointerEvent(&event)?;
                let InputEvent::Pointer(host) = input else {
                    unreachable!()
                };
                let convert = |sample: &super::input_event::PointerEvent| {
                    owners
                        .host
                        .TransformPointerEvent(&sample.ToWebEvent(timestamp)?)
                };
                let coalesced = if packet.is_some() {
                    PointerSamples(packet, true, owners.host.as_ref())?
                } else {
                    host.coalesced
                        .iter()
                        .map(convert)
                        .collect::<Result<Vec<_>, _>>()?
                };
                let predicted = if packet.is_some() {
                    PointerSamples(packet, false, owners.host.as_ref())?
                } else {
                    host.predicted
                        .iter()
                        .map(convert)
                        .collect::<Result<Vec<_>, _>>()?
                };
                owners
                    .handler
                    .HandlePointerEvent(&event, &coalesced, &predicted)
            }
            ConvertedWebInputEvent::Touch(event) => {
                // WidgetBaseInputHandler.cc:199-233. Touch packets do not enter
                // WidgetEventHandler or masquerade as compatibility mouse input.
                if event.GetType() == Type::kTouchScrollStarted {
                    let mut pointer = WebPointerEvent::CreatePointerCausesUaActionEvent(
                        PointerType::kUnknown,
                        event.TimeStamp(),
                    );
                    pointer.SetQueuedTimeStamp(event.QueuedTimeStamp());
                    owners.host.NotifyInput(&pointer)?;
                    if !owners.host.HasRootView() {
                        return Ok(InteractionInputResult::WebInput(
                            WebInputEventResult::kNotHandled,
                        ));
                    }
                    owners.handler.HandlePointerEvent(&pointer, &[], &[])
                } else {
                    let InputEvent::Touch(host) = input else {
                        unreachable!()
                    };
                    let convert = |sample: &super::input_event::TouchEvent| -> Result<Box<dyn WebInputEventPayload>, InteractionError> {
                        Ok(Box::new(sample.ToWebEvent(timestamp)?))
                    };
                    let coalesced = if let Some(packet) = packet {
                        packet
                            .CoalescedEvents()
                            .map(WebInputEventPayload::Clone)
                            .collect()
                    } else if host.coalesced.is_empty() {
                        vec![Box::new(event.clone()) as Box<dyn WebInputEventPayload>]
                    } else {
                        host.coalesced
                            .iter()
                            .map(convert)
                            .collect::<Result<Vec<_>, _>>()?
                    };
                    let predicted = if let Some(packet) = packet {
                        packet
                            .PredictedEvents()
                            .map(WebInputEventPayload::Clone)
                            .collect()
                    } else {
                        host.predicted
                            .iter()
                            .map(convert)
                            .collect::<Result<Vec<_>, _>>()?
                    };
                    for point in &event.touches[..event.touches_length] {
                        if point.state == TouchState::kStateStationary {
                            continue;
                        }
                        let mut pointer = WebPointerEvent::FromTouch(&event, point);
                        pointer.SetQueuedTimeStamp(event.QueuedTimeStamp());
                        let packet = GetCoalescedWebPointerEventForTouch(
                            &pointer,
                            &coalesced,
                            &predicted,
                            &(),
                        );
                        let convert_pointer = |payload: &Box<dyn WebInputEventPayload>| {
                            owners.host.TransformPointerEvent(
                                payload
                                    .AsAny()
                                    .downcast_ref::<WebPointerEvent>()
                                    .expect("pointer sample"),
                            )
                        };
                        let coalesced = packet
                            .GetCoalescedEventsPointers()
                            .iter()
                            .map(convert_pointer)
                            .collect::<Result<Vec<_>, _>>()?;
                        let predicted = packet
                            .GetPredictedEventsPointers()
                            .iter()
                            .map(convert_pointer)
                            .collect::<Result<Vec<_>, _>>()?;
                        owners.host.NotifyInput(&pointer)?;
                        if !owners.host.HasRootView() {
                            return Ok(InteractionInputResult::WebInput(
                                WebInputEventResult::kNotHandled,
                            ));
                        }
                        let pointer = owners.host.TransformPointerEvent(&pointer)?;
                        owners
                            .handler
                            .HandlePointerEvent(&pointer, &coalesced, &predicted);
                    }
                    owners.handler.DispatchBufferedTouchEvents()
                }
            }
            ConvertedWebInputEvent::Keyboard(event) => {
                owners.host.NotifyInput(&event)?;
                owners.keyboard.HandleKeyboardEvent(&event)?
            }
            ConvertedWebInputEvent::Wheel(event) => {
                let wheel = owners
                    .wheel
                    .as_ref()
                    .ok_or(InteractionError::MissingPort(InteractionPort::Wheel))?;
                owners.host.NotifyInput(&event)?;
                if !owners.host.HasRootView() {
                    return Ok(InteractionInputResult::WebInput(
                        WebInputEventResult::kNotHandled,
                    ));
                }
                let event = owners.host.TransformWheelEvent(&event)?;
                let InputEvent::Wheel(host_event) = input else {
                    unreachable!()
                };
                wheel.HandleWheelEvent(&event, host_event, &owners.output)?
            }
        };
        Ok(InteractionInputResult::WebInput(result))
    }

    pub fn DispatchTargeted(
        &self,
        input: &InputEvent,
    ) -> Result<InteractionInputResult, InteractionError> {
        // Only source focus/IME paths currently accept explicit target facts.
        // Dispatch performs the same validation for every caller.
        self.Dispatch(input)
    }

    pub fn DispatchDOMEvent(
        &self,
        event: &mut Event,
        target: u64,
        options: &EventProjectionOptions,
    ) -> Result<InteractionResult, InteractionError> {
        self.owners.dom_events.Dispatch(event, target, options)
    }

    pub fn DispatchFocusWithLocalEvents(
        &self,
        event: &FocusEvent,
        local_events: &dyn LocalEventDispatch,
    ) -> Result<InteractionResult, InteractionError> {
        self.owners
            .focus
            .as_ref()
            .ok_or(InteractionError::MissingPort(InteractionPort::Focus))?
            .HandleFocusWithLocalEvents(event, &self.owners.output, local_events)
    }

    pub fn SubmitFormWithLocalEvents(
        &self,
        form: u64,
        submitter: Option<u64>,
        dispatch_event: bool,
        local_events: &dyn LocalEventDispatch,
    ) -> Result<(), InteractionError> {
        self.owners
            .forms
            .as_ref()
            .ok_or(InteractionError::MissingPort(
                InteractionPort::FormSubmission,
            ))?
            .SubmitFormWithLocalEvents(
                form,
                submitter,
                dispatch_event,
                &self.owners.output,
                local_events,
            )
    }

    pub fn SubmitForm(
        &self,
        form: u64,
        submitter: Option<u64>,
        dispatch_event: bool,
    ) -> Result<(), InteractionError> {
        self.owners
            .forms
            .as_ref()
            .ok_or(InteractionError::MissingPort(
                InteractionPort::FormSubmission,
            ))?
            .SubmitForm(form, submitter, dispatch_event, &self.owners.output)
    }
}

pub(super) fn ConvertedPacket(
    input: &InputEvent,
    packet: BorrowedInputPacket<'_>,
) -> Result<ConvertedWebInputEvent, InteractionError> {
    let payload = packet.Event().AsAny();
    let invalid = || InteractionError::InvalidPacket("host input and canonical payload disagree");
    Ok(match input {
        InputEvent::Mouse(_) => ConvertedWebInputEvent::Mouse(payload.downcast_ref::<WebMouseEvent>().ok_or_else(invalid)?.clone()),
        InputEvent::Pointer(_) => ConvertedWebInputEvent::Pointer(payload.downcast_ref::<WebPointerEvent>().ok_or_else(invalid)?.clone()),
        InputEvent::Touch(_) => ConvertedWebInputEvent::Touch(payload.downcast_ref::<interaction_input::public::common::input::web_touch_event::WebTouchEvent>().ok_or_else(invalid)?.clone()),
        InputEvent::Gesture(_) => ConvertedWebInputEvent::Gesture(payload.downcast_ref::<super::input_event::WebGestureEvent>().ok_or_else(invalid)?.clone()),
        InputEvent::Wheel(_) => ConvertedWebInputEvent::Wheel(payload.downcast_ref::<WebMouseWheelEvent>().ok_or_else(invalid)?.clone()),
        InputEvent::Key(_) => ConvertedWebInputEvent::Keyboard(payload.downcast_ref::<interaction_input::public::common::input::web_keyboard_event::WebKeyboardEvent>().ok_or_else(invalid)?.clone()),
        _ => return Err(InteractionError::InvalidPacket("host-only input has no raw packet")),
    })
}

fn ConvertedTimestamp(event: &ConvertedWebInputEvent) -> TimeTicks {
    match event {
        ConvertedWebInputEvent::Mouse(event) => event.TimeStamp(),
        ConvertedWebInputEvent::Pointer(event) => event.TimeStamp(),
        ConvertedWebInputEvent::Touch(event) => event.TimeStamp(),
        ConvertedWebInputEvent::Gesture(event) => event.TimeStamp(),
        ConvertedWebInputEvent::Wheel(event) => event.TimeStamp(),
        ConvertedWebInputEvent::Keyboard(event) => event.TimeStamp(),
    }
}

fn PacketSamples<'a>(
    packet: Option<BorrowedInputPacket<'a>>,
    coalesced: bool,
) -> Vec<&'a dyn WebInputEventPayload> {
    packet.map_or_else(Vec::new, |packet| {
        if coalesced {
            packet.CoalescedEvents().collect()
        } else {
            packet.PredictedEvents().collect()
        }
    })
}

fn MouseSamples(
    packet: Option<BorrowedInputPacket<'_>>,
    coalesced: bool,
    host: &dyn InteractionHost,
) -> Result<Vec<WebMouseEvent>, InteractionError> {
    PacketSamples(packet, coalesced)
        .into_iter()
        .map(|payload| {
            host.TransformMouseEvent(
                payload
                    .AsAny()
                    .downcast_ref::<WebMouseEvent>()
                    .ok_or(InteractionError::InvalidPacket("mouse sample"))?,
            )
        })
        .collect()
}

fn PointerSamples(
    packet: Option<BorrowedInputPacket<'_>>,
    coalesced: bool,
    host: &dyn InteractionHost,
) -> Result<Vec<WebPointerEvent>, InteractionError> {
    PacketSamples(packet, coalesced)
        .into_iter()
        .map(|payload| {
            host.TransformPointerEvent(
                payload
                    .AsAny()
                    .downcast_ref::<WebPointerEvent>()
                    .ok_or(InteractionError::InvalidPacket("pointer sample"))?,
            )
        })
        .collect()
}
