use super::input_event::*;
use std::sync::atomic::{AtomicU64, Ordering};

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
    pub underlying_event: Option<InputEvent>,
    pub target_node_id: u64,
    pub current_target_node_id: u64,
    pub submitter_node_id: Option<u64>,
    pub related_target_node_id: Option<u64>,
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
    pub text: String,
    pub input_type: String,
    pub repeat: bool,
    pub identity: u64,
}
impl Default for Event {
    fn default() -> Self {
        Self {
            r#type: EventType::kMouseMove,
            custom_type: String::new(),
            underlying_event: None,
            target_node_id: 0,
            current_target_node_id: 0,
            submitter_node_id: None,
            related_target_node_id: None,
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
            text: String::new(),
            input_type: String::new(),
            repeat: false,
            identity: NextEventIdentity(),
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
        InputEvent::Wheel(v) => {
            event.r#type = EventType::kWheel;
            event.modifiers = v.modifiers;
            event.target_node_id = v.target_node_id.unwrap_or(0);
        }
        InputEvent::Key(v) => {
            event.r#type = if v.r#type == KeyEventType::kDown {
                EventType::kKeyDown
            } else {
                EventType::kKeyUp
            };
            event.key = v.key.clone();
            event.text = v.text.clone();
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
pub type EventListenerDispatcher<'a> = std::rc::Rc<
    dyn Fn(
            &EventListenerInvocation<'_>,
            &crate::ownership::InteractionDocument,
            &crate::ownership::InteractionDOMMutationEmitter,
        ) -> EventListenerResult
        + 'a,
>;
