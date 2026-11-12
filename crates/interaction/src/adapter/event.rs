//! Rechrom Adapter: compatibility values at the browser/WebAPI boundary.
//!
//! Event construction and flags project into Chromium's translated Event
//! subclasses. EventDispatcher, EventPath and EventTarget retain propagation,
//! retargeting, passive cancellation and listener ordering ownership. This
//! module contains no dispatch traversal or DOM tree representation.
#![allow(non_snake_case, non_camel_case_types)]

pub use super::page_event_dispatcher::editor_target_ranges::StaticRangeForInputEvent;

use super::input_event::*;
use interaction_dispatch::core::dom::events::{
    event::{Event as DispatchEvent, EventClock, EventInit, EventRef, EventTargetRef, PhaseType},
    event_dispatcher::{BaseEventDispatchHook, DispatchPathRef, EventDispatchHook},
    event_modifier_init::EventModifierInit,
};
use interaction_dispatch::core::events::{
    composition_event::CompositionEvent as DispatchCompositionEvent,
    composition_event_init::CompositionEventInit,
    drag_event::{DragEvent as DispatchDragEvent, DragEventDispatchHook},
    focus_event::FocusEvent as DispatchFocusEvent,
    focus_event_init::FocusEventInit,
    input_event::InputEvent as DispatchInputEvent,
    input_event_init::InputEventInit,
    input_event_range::RangeExceptionState,
    keyboard_event::KeyboardEvent as DispatchKeyboardEvent,
    keyboard_event_init::KeyboardEventInit,
    mouse_event::{
        MenuSourceType, MouseCoordinateContext, MouseEvent as DispatchMouseEvent,
        MouseEventDispatchHook, MouseNestedDispatch, SyntheticEventType,
    },
    mouse_event_init::MouseEventInit,
    pointer_event::PointerEvent as DispatchPointerEvent,
    pointer_event_init::PointerEventInit,
    ui_event::AbstractViewRef,
    ui_event_init::UIEventInit,
    wheel_event::WheelEvent as DispatchWheelEvent,
    wheel_event_init::WheelEventInit,
};
use interaction_input::base::time::time::TimeTicks;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{cell::RefCell, fmt, rc::Rc};

// cpp: interaction/event.h:21-56
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventType {
    kMouseMove,
    kMouseDown,
    kMouseUp,
    kClick,
    kDoubleClick,
    kContextMenu,
    kMouseEnter,
    kMouseLeave,
    kPointerMove,
    kPointerRawUpdate,
    kPointerDown,
    kPointerUp,
    kPointerCancel,
    kPointerEnter,
    kPointerLeave,
    kWheel,
    kKeyDown,
    kKeyPress,
    kKeyUp,
    kCompositionStart,
    kCompositionUpdate,
    kCompositionEnd,
    kTextInput,
    kBeforeInput,
    kInput,
    kChange,
    kFocus,
    kFocusIn,
    kBlur,
    kFocusOut,
    kSubmit,
    kReset,
    kDOMContentLoaded,
    kReadyStateChange,
    kLoad,
    kCustom,
    kMouseOver,
    kMouseOut,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EventPhase {
    #[default]
    kNone,
    kCapturing,
    kAtTarget,
    kBubbling,
}
// cpp: interaction/event.cc:10-13
pub fn NextEventIdentity() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}
// cpp: interaction/event.h:60-86
#[derive(Clone, Debug)]
pub struct Event {
    pub r#type: EventType,
    pub custom_type: String,
    pub interface_name: String,
    pub canonical: Option<EventRef>,
    /// Native binding namespace for node references; Window (id 0) is global.
    pub node_owner_identity: Option<u128>,
    pub data_transfer:
        Option<Rc<interaction_dispatch::core::clipboard::data_transfer::DataTransfer>>,
    pub underlying_event: Option<InputEvent>,
    pub target_node_id: u64,
    pub current_target_node_id: u64,
    pub submitter_node_id: Option<u64>,
    pub related_target_node_id: Option<u64>,
    /// Trace projection of ToggleEvent::source_; payload stays on native Event.
    pub source_node_id: Option<u64>,
    pub phase: EventPhase,
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub trusted: bool,
    pub default_prevented: bool,
    pub default_handled: bool,
    pub propagation_stopped: bool,
    pub immediate_propagation_stopped: bool,
    pub capture_listeners: bool,
    pub modifiers: EventModifiers,
    pub button: MouseButton,
    pub key: String,
    pub code: String,
    pub text: String,
    /// Exact DOMString code units captured from a native event. The outer
    /// option distinguishes a native payload from a compatibility UTF-8 value;
    /// the inner option preserves InputEvent.data's nullable value.
    pub data_utf16: Option<Option<Rc<[u16]>>>,
    pub key_code: u32,
    pub char_code: u32,
    pub key_location: u32,
    pub is_composing: bool,
    pub input_type: String,
    pub repeat: bool,
    pub identity: u64,
    pub dispatch_state: std::rc::Rc<std::cell::RefCell<EventDispatchState>>,
}
impl Default for Event {
    fn default() -> Self {
        Self {
            r#type: EventType::kMouseMove,
            custom_type: String::new(),
            interface_name: "Event".into(),
            canonical: None,
            node_owner_identity: None,
            data_transfer: None,
            underlying_event: None,
            target_node_id: 0,
            current_target_node_id: 0,
            submitter_node_id: None,
            related_target_node_id: None,
            source_node_id: None,
            phase: EventPhase::kNone,
            bubbles: true,
            cancelable: true,
            composed: true,
            trusted: true,
            default_prevented: false,
            default_handled: false,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            capture_listeners: false,
            modifiers: EventModifiers::default(),
            button: MouseButton::kNone,
            key: String::new(),
            code: String::new(),
            text: String::new(),
            data_utf16: None,
            key_code: 0,
            char_code: 0,
            key_location: 0,
            is_composing: false,
            input_type: String::new(),
            repeat: false,
            identity: NextEventIdentity(),
            dispatch_state: Default::default(),
        }
    }
}
// cpp: interaction/event.cc:43-100
pub fn MakeEvent(input: &InputEvent) -> Event {
    let mut event = Event {
        underlying_event: Some(input.clone()),
        ..Default::default()
    };
    match input {
        InputEvent::Mouse(v) => {
            event.r#type = match v.r#type {
                MouseEventType::kMove => EventType::kMouseMove,
                MouseEventType::kDown => EventType::kMouseDown,
                MouseEventType::kUp => EventType::kMouseUp,
                MouseEventType::kClick => EventType::kClick,
                MouseEventType::kDoubleClick => EventType::kDoubleClick,
                MouseEventType::kContextMenu => EventType::kContextMenu,
                MouseEventType::kEnter => EventType::kMouseEnter,
                MouseEventType::kLeave => EventType::kMouseLeave,
            };
            event.button = v.button;
            event.modifiers = v.modifiers;
            event.target_node_id = v.target_node_id.unwrap_or(0);
            event.bubbles = !matches!(v.r#type, MouseEventType::kEnter | MouseEventType::kLeave);
        }
        InputEvent::Pointer(v) => {
            event.r#type = match v.r#type {
                PointerEventType::kMove => EventType::kPointerMove,
                PointerEventType::kRawUpdate => EventType::kPointerRawUpdate,
                PointerEventType::kDown => EventType::kPointerDown,
                PointerEventType::kUp => EventType::kPointerUp,
                PointerEventType::kCancel => EventType::kPointerCancel,
                PointerEventType::kEnter => EventType::kPointerEnter,
                PointerEventType::kLeave => EventType::kPointerLeave,
            };
            event.button = v.button;
            event.modifiers = v.modifiers;
            event.target_node_id = v.target_node_id.unwrap_or(0);
            event.bubbles = !matches!(
                v.r#type,
                PointerEventType::kEnter | PointerEventType::kLeave
            );
        }
        InputEvent::Touch(_) | InputEvent::Gesture(_) => {
            unreachable!("touch requires the native PointerEventManager/TouchEventManager port")
        }
        InputEvent::Wheel(v) => {
            event.r#type = EventType::kWheel;
            event.modifiers = v.modifiers;
            event.target_node_id = v.target_node_id.unwrap_or(0);
        }
        InputEvent::Key(v) => {
            event.r#type = match v.r#type {
                KeyEventType::kDown => EventType::kKeyDown,
                KeyEventType::kUp => EventType::kKeyUp,
                KeyEventType::kChar => EventType::kKeyPress,
            };
            event.key = v.key.clone();
            event.code = v.code.clone();
            event.text = v.text.clone();
            event.key_code = v.key_code;
            event.key_location = v.location;
            event.is_composing = v.is_composing;
            event.modifiers = v.modifiers;
            event.repeat = v.repeat;
            event.target_node_id = v.target_node_id.unwrap_or(0);
        }
        InputEvent::Composition(v) => {
            event.r#type = match v.r#type {
                CompositionEventType::kStart => EventType::kCompositionStart,
                CompositionEventType::kUpdate => EventType::kCompositionUpdate,
                CompositionEventType::kEnd => EventType::kCompositionEnd,
            };
            event.text = v.data.clone();
            event.target_node_id = v.target_node_id.unwrap_or(0);
        }
        InputEvent::TextInput(v) => {
            event.r#type = EventType::kTextInput;
            event.text = v.text.clone();
            event.input_type = "insertText".into();
            event.target_node_id = v.target_node_id.unwrap_or(0);
        }
        InputEvent::Focus(v) => {
            event.r#type = if v.r#type == FocusEventType::kFocus {
                EventType::kFocus
            } else {
                EventType::kBlur
            };
            event.target_node_id = v.target_node_id;
            event.related_target_node_id = v.related_target_node_id;
            event.bubbles = false;
            event.composed = true;
        }
    }
    event
}
// cpp: interaction/event.cc:102-123
pub fn MakeSyntheticEvent(r#type: EventType, target_node_id: u64) -> Event {
    let mut event = Event {
        r#type,
        target_node_id,
        ..Default::default()
    };
    match r#type {
        EventType::kInput => event.cancelable = false,
        EventType::kChange => {
            event.cancelable = false;
            event.composed = false;
        }
        EventType::kFocus | EventType::kBlur => {
            event.bubbles = false;
            event.cancelable = false;
        }
        EventType::kFocusIn | EventType::kFocusOut => event.cancelable = false,
        EventType::kReset => event.composed = false,
        _ => {}
    }
    if matches!(
        r#type,
        EventType::kDOMContentLoaded | EventType::kReadyStateChange | EventType::kLoad
    ) {
        event.bubbles = r#type == EventType::kDOMContentLoaded;
        event.cancelable = false;
        event.composed = false;
    }
    event
}
// cpp: interaction/event.cc:125-163
pub fn EventTypeName(r#type: EventType) -> &'static str {
    match r#type {
        EventType::kMouseMove => "mousemove",
        EventType::kMouseDown => "mousedown",
        EventType::kMouseUp => "mouseup",
        EventType::kClick => "click",
        EventType::kDoubleClick => "dblclick",
        EventType::kContextMenu => "contextmenu",
        EventType::kMouseEnter => "mouseenter",
        EventType::kMouseLeave => "mouseleave",
        EventType::kMouseOver => "mouseover",
        EventType::kMouseOut => "mouseout",
        EventType::kPointerMove => "pointermove",
        EventType::kPointerRawUpdate => "pointerrawupdate",
        EventType::kPointerDown => "pointerdown",
        EventType::kPointerUp => "pointerup",
        EventType::kPointerCancel => "pointercancel",
        EventType::kPointerEnter => "pointerenter",
        EventType::kPointerLeave => "pointerleave",
        EventType::kWheel => "wheel",
        EventType::kKeyDown => "keydown",
        EventType::kKeyPress => "keypress",
        EventType::kKeyUp => "keyup",
        EventType::kCompositionStart => "compositionstart",
        EventType::kCompositionUpdate => "compositionupdate",
        EventType::kCompositionEnd => "compositionend",
        EventType::kTextInput => "textInput",
        EventType::kBeforeInput => "beforeinput",
        EventType::kInput => "input",
        EventType::kChange => "change",
        EventType::kFocus => "focus",
        EventType::kFocusIn => "focusin",
        EventType::kBlur => "blur",
        EventType::kFocusOut => "focusout",
        EventType::kSubmit => "submit",
        EventType::kReset => "reset",
        EventType::kDOMContentLoaded => "DOMContentLoaded",
        EventType::kReadyStateChange => "readystatechange",
        EventType::kLoad => "load",
        EventType::kCustom => "",
    }
}
// cpp: interaction/event.h:92-108
pub struct EventListenerInvocation<'a> {
    pub event: &'a Event,
    pub target_node_id: u64,
    pub current_target_node_id: u64,
    pub phase: EventPhase,
    pub capture_listeners: bool,
    pub focused_node_id: Option<u64>,
}
impl<'a> EventListenerInvocation<'a> {
    pub fn new(event: &'a Event) -> Self {
        Self {
            event,
            target_node_id: 0,
            current_target_node_id: 0,
            phase: EventPhase::kAtTarget,
            capture_listeners: false,
            focused_node_id: None,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct EventListenerResult {
    pub prevent_default: bool,
    pub stop_propagation: bool,
    pub stop_immediate_propagation: bool,
}
// C++ const-callable std::function can reenter dispatch. Rc<Fn> keeps that
// behavior. The retained DOM handle permits callback DOM mutation
// without a borrowed Document or arena node address surviving the call.
pub type GenericEventListenerDispatcher<'a, Document, MutationEmitter> = std::rc::Rc<
    dyn Fn(&EventListenerInvocation<'_>, &Document, &MutationEmitter) -> EventListenerResult + 'a,
>;

pub type EventListenerDispatcher<'a> = GenericEventListenerDispatcher<
    'a,
    super::interaction_contract::InteractionDocument,
    super::interaction_contract::InteractionDOMMutationEmitter,
>;

/// Read-only projection of EventTarget's original listener map. The owner is
/// retained independently of the JavaScript bindings borrow, so synchronous
/// listener mutation is visible to later phases in the same dispatch.
///
/// Chromium: core/dom/events/event_target.h:414-438;
/// core/dom/events/event_listener_map.cc:56-76.
pub trait EventListenerInterest {
    fn HasPointerEventHandlers(&self) -> bool;
    fn HasPointerRawUpdateEventHandlers(&self) -> bool;
    fn HasTouchEventHandlers(&self) -> bool;
    fn HasBlockingTouchHandler(
        &self,
        document: dom::persistent_document::DOMOwnerHandle,
        target_node_id: u64,
    ) -> bool;
    fn HasAttributeEventListener(
        &self,
        document: dom::persistent_document::DOMOwnerHandle,
        target_node_id: u64,
        event_type: &str,
    ) -> bool;
    fn HasEventListeners(
        &self,
        document: dom::persistent_document::DOMOwnerHandle,
        target_node_id: u64,
        event_type: &str,
    ) -> bool;
    fn HasCapturingEventListeners(
        &self,
        document: dom::persistent_document::DOMOwnerHandle,
        target_node_id: u64,
        event_type: &str,
    ) -> bool;
}

pub type EventListenerInterestRef = std::rc::Rc<dyn EventListenerInterest>;

// Rechrom Adapter port. IDs are WebAPI identities (0 denotes Window), not
// indices into another tree. ResolveTarget must return the DOM owner's stable
// handle for every repeated lookup, including detached retained nodes.
pub trait EventTargetResolver {
    fn ResolveTarget(&self, node_id: u64) -> Option<EventTargetRef>;
    fn NodeId(&self, target: &EventTargetRef) -> Option<u64>;
    fn DocumentOwnerIdentity(&self) -> Option<u128> {
        None
    }
}

/// Read-only compatibility view over the exact translated dispatch owner.
/// The embedding attaches it after DispatchPathAccess initializes that owner.
pub struct EventPath {
    event: EventRef,
    path: Option<DispatchPathRef>,
    targets: Rc<dyn EventTargetResolver>,
}

impl EventPath {
    pub fn FromDispatch(
        event: EventRef,
        path: Option<DispatchPathRef>,
        targets: Rc<dyn EventTargetResolver>,
    ) -> Self {
        Self {
            event,
            path,
            targets,
        }
    }

    pub fn ComposedPath(&self, current_target: u64) -> Vec<u64> {
        let event = self.event.borrow();
        let base = event.BaseEvent();
        // Chromium event.cc:326-363 supplies closed-root filtering and the
        // non-Node Window branch from its resident currentTarget/path.
        if base
            .currentTarget()
            .as_ref()
            .and_then(|t| self.targets.NodeId(t))
            != Some(current_target)
        {
            return Vec::new();
        }
        base.composedPath(true)
            .iter()
            .filter_map(|target| self.targets.NodeId(target))
            .collect()
    }

    pub fn Nodes(&self) -> std::vec::IntoIter<u64> {
        // Binding GC roots need every invocation identity, including nodes
        // hidden from composedPath(). Read contexts; do not rebuild a path.
        let mut ids = Vec::new();
        if let Some(path) = &self.path {
            let path = path.borrow();
            ids.extend(
                path.NodeEventContexts()
                    .iter()
                    .filter_map(|entry| self.targets.NodeId(&entry.GetNode())),
            );
            if !path.IsEmpty() {
                path.EnsureWindowEventContext();
                ids.extend(
                    path.GetWindowEventContext()
                        .Window()
                        .as_ref()
                        .and_then(|target| self.targets.NodeId(target)),
                );
            }
        } else {
            let event = self.event.borrow();
            ids.extend(
                event
                    .BaseEvent()
                    .currentTarget()
                    .as_ref()
                    .and_then(|target| self.targets.NodeId(target)),
            );
        }
        ids.into_iter()
    }
}

impl fmt::Debug for EventPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("EventPath").finish_non_exhaustive()
    }
}

/// Shared by compatibility Event clones retained by JavaScript bindings.
/// Completion drops only this view; canonical Event/path lifetime stays with
/// the translated owner. No DOM storage or independently computed path lives here.
#[derive(Debug, Default)]
pub struct EventDispatchState {
    pub path: Option<Rc<EventPath>>,
    pub final_target: Option<u64>,
    pub finished: bool,
}

/// Explicit host facts missing from the historical Event value. Timestamp is
/// captured by the host in its existing monotonic clock domain.
#[derive(Clone, Default)]
pub struct EventProjectionOptions {
    pub timestamp: TimeTicks,
    pub view: Option<AbstractViewRef>,
    pub coordinates: MouseCoordinateContext,
    pub screen_x: f64,
    pub screen_y: f64,
    pub buttons: u16,
    pub detail: i32,
}

struct ProjectionClock(TimeTicks);
impl EventClock for ProjectionClock {
    fn Now(&self) -> TimeTicks {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnresolvedEventTarget(pub u64);

// Shared concrete handles permit source subclass hooks to be prepared before
// callback entry. EventRef::FromShared retains the very same base owner.
enum ProjectionDispatch {
    BindingOnly,
    Canonical,
    Base,
    Drag,
    Mouse(Rc<RefCell<DispatchMouseEvent>>),
    Pointer(Rc<RefCell<DispatchPointerEvent>>),
    Focus(Rc<RefCell<DispatchFocusEvent>>),
    Input(Rc<RefCell<DispatchInputEvent>>),
}

pub struct EventProjection {
    event: EventRef,
    targets: Rc<dyn EventTargetResolver>,
    dispatch: ProjectionDispatch,
}

impl EventProjection {
    /// Projects an existing native event for bindings. This retains its exact
    /// EventRef and never chooses or substitutes a native virtual dispatch hook.
    pub fn FromCanonical(event: EventRef, targets: Rc<dyn EventTargetResolver>) -> Self {
        Self {
            event,
            targets,
            dispatch: ProjectionDispatch::BindingOnly,
        }
    }

    pub fn BindingEvent(&self) -> Event {
        let mut event = Event::default();
        event.node_owner_identity = self.targets.DocumentOwnerIdentity();
        {
            let owner = self.event.borrow();
            let base = owner.BaseEvent();
            event.r#type = EventType::kCustom;
            event.custom_type = base.r#type().to_owned();
            event.interface_name = owner.InterfaceName().to_owned();
            event.canonical = Some(self.event.clone());
            event.data_transfer = owner
                .AsDragEvent()
                .and_then(|drag| drag.dataTransfer())
                .or_else(|| {
                    owner
                        .AsClipboardEvent()
                        .and_then(|event| event.clipboardData())
                });
            event.source_node_id = owner
                .AsToggleEvent()
                .and_then(|toggle| toggle.OriginalSource())
                .as_ref()
                .and_then(|source| self.targets.NodeId(source));
            event.bubbles = base.bubbles();
            event.cancelable = base.cancelable();
            event.composed = base.composed();
            event.trusted = base.isTrusted();
            event.submitter_node_id = owner
                .Submitter()
                .as_ref()
                .and_then(|target| self.targets.NodeId(target));
            if let Some(keys) = owner.AsUIEventWithKeyState() {
                event.modifiers = EventModifiers {
                    alt: keys.altKey(),
                    control: keys.ctrlKey(),
                    meta: keys.metaKey(),
                    shift: keys.shiftKey(),
                };
            }
            if let Some(mouse) = owner.AsMouseEvent() {
                event.button = match mouse.button() {
                    0 => MouseButton::kPrimary,
                    1 => MouseButton::kMiddle,
                    2 => MouseButton::kSecondary,
                    _ => MouseButton::kNone,
                };
                event.underlying_event = Some(InputEvent::Mouse(MouseEvent {
                    position: Offset {
                        x: mouse.clientX(),
                        y: mouse.clientY(),
                    },
                    button: event.button,
                    modifiers: event.modifiers,
                    ..Default::default()
                }));
            }
            if let Some(pointer) = owner.AsPointerEvent() {
                event.button = match pointer.button() {
                    0 => MouseButton::kPrimary,
                    1 => MouseButton::kMiddle,
                    2 => MouseButton::kSecondary,
                    _ => MouseButton::kNone,
                };
                event.underlying_event = Some(InputEvent::Pointer(PointerEvent {
                    position: Offset {
                        x: pointer.clientX(),
                        y: pointer.clientY(),
                    },
                    button: event.button,
                    modifiers: event.modifiers,
                    ..Default::default()
                }));
            }
            if let Some(wheel) = owner.AsWheelEvent() {
                event.underlying_event = Some(InputEvent::Wheel(WheelEvent {
                    position: owner
                        .AsMouseEvent()
                        .map_or(Offset::default(), |mouse| Offset {
                            x: mouse.clientX(),
                            y: mouse.clientY(),
                        }),
                    delta: Offset {
                        x: wheel.deltaX(),
                        y: wheel.deltaY(),
                    },
                    modifiers: event.modifiers,
                    ..Default::default()
                }));
            }
            if let Some(key) = owner.AsKeyboardEvent() {
                event.key = key.key().to_owned();
                event.code = key.code().to_owned();
                event.key_code = key.keyCode() as u32;
                event.char_code = key.charCode() as u32;
                event.key_location = key.location();
                event.repeat = key.repeat();
                event.is_composing = key.isComposing();
            }
            if let Some(input) = owner.AsInputEvent() {
                event.data_transfer = input.dataTransfer();
                event.data_utf16 = Some(input.dataUTF16());
                event.text = input
                    .data()
                    .map(String::from_utf16_lossy)
                    .unwrap_or_default();
                event.input_type = input.inputType().to_owned();
                event.is_composing = input.isComposing();
            }
            if let Some(text) = owner.AsTextEvent() {
                event.data_utf16 = Some(Some(text.dataUTF16()));
                event.text = String::from_utf16_lossy(text.data());
            }
            if let Some(composition) = owner.AsCompositionEvent() {
                event.data_utf16 = Some(Some(composition.dataUTF16()));
                event.text = String::from_utf16_lossy(composition.data());
            }
        }
        self.AttachPath(&event, None);
        self.SyncTo(&mut event);
        event
    }
    pub fn new(
        compatibility: &Event,
        targets: Rc<dyn EventTargetResolver>,
        options: &EventProjectionOptions,
    ) -> Result<Self, UnresolvedEventTarget> {
        let target = targets
            .ResolveTarget(compatibility.target_node_id)
            .ok_or(UnresolvedEventTarget(compatibility.target_node_id))?;
        if let Some(event) = compatibility.canonical.clone() {
            {
                let mut owner = event.borrow_mut();
                owner.SetTarget(Some(target));
                owner.BaseEventMut().SetTrusted(compatibility.trusted);
                owner
                    .BaseEventMut()
                    .SetStopPropagation(compatibility.propagation_stopped);
                owner
                    .BaseEventMut()
                    .SetStopImmediatePropagation(compatibility.immediate_propagation_stopped);
            }
            return Ok(Self {
                event,
                targets,
                dispatch: ProjectionDispatch::Canonical,
            });
        }
        let related = compatibility
            .related_target_node_id
            .map(|id| targets.ResolveTarget(id).ok_or(UnresolvedEventTarget(id)))
            .transpose()?;
        let name = if compatibility.custom_type.is_empty() {
            EventTypeName(compatibility.r#type)
        } else {
            &compatibility.custom_type
        };
        let ui = UIEventInit {
            event_init: EventInit {
                bubbles: compatibility.bubbles,
                cancelable: compatibility.cancelable,
                composed: compatibility.composed,
            },
            view: options.view.clone(),
            detail: options.detail,
            ..Default::default()
        };
        let modifiers = EventModifierInit {
            ui_event_init: ui.clone(),
            ctrl_key: compatibility.modifiers.control,
            alt_key: compatibility.modifiers.alt,
            shift_key: compatibility.modifiers.shift,
            meta_key: compatibility.modifiers.meta,
            ..Default::default()
        };
        let position = compatibility
            .underlying_event
            .as_ref()
            .and_then(|input| match input {
                InputEvent::Mouse(input) => Some(input.position),
                InputEvent::Pointer(input) => Some(input.position),
                InputEvent::Wheel(input) => Some(input.position),
                _ => None,
            });
        let mouse = MouseEventInit {
            event_modifier_init: modifiers.clone(),
            screen_x: options.screen_x,
            screen_y: options.screen_y,
            client_x: position.map_or(0.0, |point| point.x),
            client_y: position.map_or(0.0, |point| point.y),
            button: match compatibility.button {
                MouseButton::kPrimary => 0,
                MouseButton::kMiddle => 1,
                MouseButton::kSecondary => 2,
                MouseButton::kNone => -1,
            },
            buttons: options.buttons,
            related_target: related.clone(),
            ..Default::default()
        };
        let clock = ProjectionClock(options.timestamp);
        let (event, dispatch) = if compatibility.interface_name == "ClipboardEvent" {
            (EventRef::new(interaction_dispatch::core::events::clipboard_event::ClipboardEvent::WithInitializer(&name,compatibility.data_transfer.clone(),&ui.event_init,&clock)),ProjectionDispatch::Base)
        } else if compatibility.interface_name == "DragEvent" {
            (
                EventRef::new(DispatchDragEvent::new(
                    name,
                    &mouse,
                    compatibility.data_transfer.clone(),
                    options.timestamp,
                    SyntheticEventType::kRealOrIndistinguishable,
                    options.coordinates.clone(),
                )),
                ProjectionDispatch::Drag,
            )
        } else {
            match compatibility.r#type {
                EventType::kMouseMove
                | EventType::kMouseDown
                | EventType::kMouseUp
                | EventType::kClick
                | EventType::kDoubleClick
                | EventType::kContextMenu
                | EventType::kMouseEnter
                | EventType::kMouseLeave
                | EventType::kMouseOver
                | EventType::kMouseOut => {
                    let event = Rc::new(RefCell::new(DispatchMouseEvent::new(
                        name,
                        &mouse,
                        options.timestamp,
                        SyntheticEventType::kRealOrIndistinguishable,
                        MenuSourceType::kNone,
                        None,
                        options.coordinates.clone(),
                    )));
                    (
                        EventRef::FromShared(event.clone()),
                        ProjectionDispatch::Mouse(event),
                    )
                }
                EventType::kPointerMove
                | EventType::kPointerRawUpdate
                | EventType::kPointerDown
                | EventType::kPointerUp
                | EventType::kPointerCancel
                | EventType::kPointerEnter
                | EventType::kPointerLeave => {
                    let mut init = PointerEventInit {
                        mouse_event_init: mouse,
                        ..Default::default()
                    };
                    if let Some(InputEvent::Pointer(input)) = &compatibility.underlying_event {
                        init.pointer_id = Some(input.pointer_id as i32);
                        init.pointer_type = Some(
                            match input.pointer_type {
                                PointerType::kMouse => "mouse",
                                PointerType::kPen => "pen",
                                PointerType::kTouch => "touch",
                            }
                            .into(),
                        );
                        init.pressure = Some(input.pressure as f32);
                        init.is_primary = Some(input.primary);
                    }
                    let event = Rc::new(RefCell::new(DispatchPointerEvent::new(
                        name,
                        &init,
                        options.timestamp,
                        SyntheticEventType::kRealOrIndistinguishable,
                        MenuSourceType::kNone,
                        false,
                        options.coordinates.clone(),
                    )));
                    (
                        EventRef::FromShared(event.clone()),
                        ProjectionDispatch::Pointer(event),
                    )
                }
                EventType::kWheel => {
                    let mut init = WheelEventInit {
                        mouse_event_init: mouse,
                        ..Default::default()
                    };
                    if let Some(InputEvent::Wheel(input)) = &compatibility.underlying_event {
                        init.delta_x = input.delta.x;
                        init.delta_y = input.delta.y;
                        init.delta_mode = match input.delta_units {
                            ScrollGranularity::kScrollByLine => 1,
                            ScrollGranularity::kScrollByPage
                            | ScrollGranularity::kScrollByDocument => 2,
                            _ => 0,
                        };
                        init.momentum = input
                            .native
                            .is_some_and(|native| native.momentum_phase != 0);
                    }
                    (
                        EventRef::new(DispatchWheelEvent::new(
                            name,
                            &init,
                            options.timestamp,
                            options.coordinates.clone(),
                            None,
                        )),
                        ProjectionDispatch::Base,
                    )
                }
                EventType::kKeyDown | EventType::kKeyPress | EventType::kKeyUp => {
                    let init = KeyboardEventInit {
                        event_modifier_init: modifiers,
                        key: compatibility.key.clone(),
                        code: compatibility.code.clone(),
                        location: compatibility.key_location,
                        repeat: compatibility.repeat,
                        is_composing: compatibility.is_composing,
                        char_code: compatibility.char_code,
                        key_code: compatibility.key_code,
                    };
                    (
                        EventRef::new(DispatchKeyboardEvent::WithInitializer(
                            name,
                            &init,
                            options.timestamp,
                        )),
                        ProjectionDispatch::Base,
                    )
                }
                EventType::kFocus
                | EventType::kFocusIn
                | EventType::kBlur
                | EventType::kFocusOut => {
                    let init = FocusEventInit {
                        ui_event_init: ui,
                        related_target: related,
                    };
                    let event = Rc::new(RefCell::new(DispatchFocusEvent::WithInitializer(
                        name, &init, &clock,
                    )));
                    (
                        EventRef::FromShared(event.clone()),
                        ProjectionDispatch::Focus(event),
                    )
                }
                EventType::kCompositionStart
                | EventType::kCompositionUpdate
                | EventType::kCompositionEnd => {
                    let init = CompositionEventInit {
                        ui_event_init: ui,
                        data: compatibility.text.clone(),
                        data_utf16: compatibility.data_utf16.clone().flatten(),
                    };
                    (
                        EventRef::new(DispatchCompositionEvent::WithInitializer(
                            name, &init, &clock,
                        )),
                        ProjectionDispatch::Base,
                    )
                }
                EventType::kBeforeInput | EventType::kInput => {
                    let init = InputEventInit {
                        ui_event_init: ui,
                        data: Some(Some(compatibility.text.clone())),
                        data_utf16: compatibility.data_utf16.clone(),
                        input_type: Some(compatibility.input_type.clone()),
                        is_composing: Some(compatibility.is_composing),
                        ..Default::default()
                    };
                    let mut exceptions = RangeExceptionState::default();
                    let event = Rc::new(RefCell::new(DispatchInputEvent::WithInitializer(
                        name,
                        &init,
                        &mut exceptions,
                        &clock,
                    )));
                    // No StaticRanges cross this compatibility value boundary.
                    debug_assert!(!exceptions.HadException());
                    (
                        EventRef::FromShared(event.clone()),
                        ProjectionDispatch::Input(event),
                    )
                }
                _ => (
                    EventRef::new(DispatchEvent::WithInitializer(
                        name,
                        &ui.event_init,
                        options.timestamp,
                    )),
                    ProjectionDispatch::Base,
                ),
            }
        };
        {
            let mut owner = event.borrow_mut();
            owner.SetTarget(Some(target));
            owner.BaseEventMut().SetTrusted(compatibility.trusted);
            if compatibility.default_prevented {
                owner.preventDefault();
            }
            let base = owner.BaseEventMut();
            if compatibility.default_handled {
                base.SetDefaultHandled();
            }
            base.SetStopPropagation(compatibility.propagation_stopped);
            base.SetStopImmediatePropagation(compatibility.immediate_propagation_stopped);
        }
        Ok(Self {
            event,
            targets,
            dispatch,
        })
    }

    pub fn EventRef(&self) -> EventRef {
        self.event.clone()
    }

    /// Prepare the source virtual dispatch hook without holding an event
    /// RefCell borrow across synchronous JavaScript/listener reentry.
    pub fn DispatchHook<'a>(
        &self,
        nested: &'a dyn MouseNestedDispatch,
    ) -> Box<dyn EventDispatchHook + 'a> {
        match &self.dispatch {
            ProjectionDispatch::Mouse(event) => Box::new(MouseEventDispatchHook {
                mouse: event.borrow().DispatchValues(),
                nested,
            }),
            ProjectionDispatch::Pointer(event) => Box::new(event.borrow().DispatchHook(nested)),
            ProjectionDispatch::Focus(event) => Box::new(event.borrow().DispatchHook()),
            ProjectionDispatch::Input(event) => Box::new(event.borrow().DispatchHook()),
            ProjectionDispatch::BindingOnly => {
                panic!("a binding view cannot dispatch a native event")
            }
            ProjectionDispatch::Canonical => self.event.borrow().PrepareDispatchHook(nested),
            ProjectionDispatch::Base => Box::new(BaseEventDispatchHook),
            ProjectionDispatch::Drag => Box::new(DragEventDispatchHook),
        }
    }

    pub fn AttachPath(&self, compatibility: &Event, path: Option<DispatchPathRef>) {
        *compatibility.dispatch_state.borrow_mut() = EventDispatchState {
            path: Some(Rc::new(EventPath::FromDispatch(
                self.event.clone(),
                path,
                self.targets.clone(),
            ))),
            final_target: None,
            finished: false,
        };
    }

    /// Snapshot only binding-visible fields. Chromium dispatch has already
    /// assigned the retargeted target, currentTarget and phase for this callback.
    pub fn SyncTo(&self, compatibility: &mut Event) {
        compatibility.node_owner_identity = self.targets.DocumentOwnerIdentity();
        let owner = self.event.borrow();
        let base = owner.BaseEvent();
        compatibility.target_node_id = base
            .target()
            .as_ref()
            .and_then(|target| self.targets.NodeId(target))
            .unwrap_or(0);
        compatibility.current_target_node_id = base
            .currentTarget()
            .as_ref()
            .and_then(|target| self.targets.NodeId(target))
            .unwrap_or(0);
        compatibility.related_target_node_id = owner
            .relatedTarget()
            .as_ref()
            .and_then(|target| self.targets.NodeId(target));
        if owner.HasInterface("SubmitEvent") {
            compatibility.submitter_node_id = owner
                .Submitter()
                .as_ref()
                .and_then(|target| self.targets.NodeId(target));
        }
        compatibility.phase = match base.eventPhase() {
            PhaseType::kNone => EventPhase::kNone,
            PhaseType::kCapturingPhase => EventPhase::kCapturing,
            PhaseType::kAtTarget => EventPhase::kAtTarget,
            PhaseType::kBubblingPhase => EventPhase::kBubbling,
        };
        compatibility.capture_listeners = base.eventPhase() == PhaseType::kCapturingPhase
            || base.FireOnlyCaptureListenersAtTarget();
        compatibility.default_prevented = base.defaultPrevented();
        compatibility.default_handled = base.DefaultHandled();
        compatibility.propagation_stopped = base.PropagationStopped();
        compatibility.immediate_propagation_stopped = base.ImmediatePropagationStopped();
    }

    /// Apply listener requests on the canonical Event while its translated
    /// passive-listener mode is active. Event::preventDefault owns cancelability
    /// and passive checks; the adapter must never set defaultPrevented directly.
    pub fn ApplyListenerResult(&self, result: EventListenerResult) {
        let mut owner = self.event.borrow_mut();
        if result.prevent_default {
            owner.preventDefault();
        }
        let base = owner.BaseEventMut();
        if result.stop_propagation {
            base.stopPropagation();
        }
        if result.stop_immediate_propagation {
            base.stopImmediatePropagation();
        }
    }

    /// Run after translated dispatch post-processing, so its target-clearing
    /// and propagation-reset semantics remain authoritative for retained events.
    pub fn Finish(&self, compatibility: &mut Event) {
        self.SyncTo(compatibility);
        let final_target = self
            .event
            .borrow()
            .BaseEvent()
            .target()
            .as_ref()
            .and_then(|target| self.targets.NodeId(target));
        let mut state = compatibility.dispatch_state.borrow_mut();
        state.path = None;
        state.final_target = final_target;
        state.finished = true;
    }
}
