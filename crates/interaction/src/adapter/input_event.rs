// Copyright 2014 The Chromium Authors
// Chromium key mapping data is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.

//! Rechrom host input boundary. These values retain the existing embedding API.
//!
//! Adapter: routing hints, native wheel metadata and `Instant` stay on the host
//! value; none is embedded into a Chromium Web event. Conversion borrows its
//! input, so the facade can retain those values alongside the translated event.
//! The embedding supplies `TimeTicks` in its chosen monotonic clock domain;
//! an `Instant` cannot be assigned Chromium's epoch without that clock mapping.

use interaction_input::base::time::time::TimeTicks;
pub use interaction_input::public::common::input::web_gesture_event::WebGestureEvent;
use interaction_input::public::common::input::web_input_event::DispatchType as WebDispatchType;
use interaction_input::public::common::input::web_input_event::{
    RailsMode, Type as WebEventType, WebInputEvent,
};
use interaction_input::public::common::input::web_keyboard_event::WebKeyboardEvent;
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;
use interaction_input::public::common::input::web_mouse_wheel_event::{
    Phase, ScrollGranularity as WebScrollGranularity, WebMouseWheelEvent,
};
use interaction_input::public::common::input::web_pointer_event::WebPointerEvent;
use interaction_input::public::common::input::web_pointer_properties::{
    Button, PointerType as WebPointerType,
};
use interaction_input::public::common::input::web_touch_event::WebTouchEvent;
use interaction_input::public::common::input::web_touch_point::{
    TouchState as WebTouchState, WebTouchPoint,
};

// Adapter: platform-independent host coordinates. The Page adapter converts
// its layout-space value at the boundary; interaction does not depend on
// LayoutNG storage or types.
pub use layoutng_assembly::internal::layout_input::Offset;

// Adapter: retained Rechrom host API; not a Chromium declaration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EventModifiers {
    pub alt: bool,
    pub control: bool,
    pub meta: bool,
    pub shift: bool,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MouseEventType {
    #[default]
    kMove,
    kDown,
    kUp,
    kClick,
    kDoubleClick,
    kContextMenu,
    kEnter,
    kLeave,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MouseButton {
    #[default]
    kNone,
    kPrimary,
    kMiddle,
    kSecondary,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerEventType {
    #[default]
    kMove,
    kRawUpdate,
    kDown,
    kUp,
    kCancel,
    kEnter,
    kLeave,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerType {
    #[default]
    kMouse,
    kPen,
    kTouch,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyEventType {
    #[default]
    kDown,
    kUp,
    /// The platform's separate committed character event (WebInputEvent::kChar).
    kChar,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompositionEventType {
    kStart,
    #[default]
    kUpdate,
    kEnd,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FocusEventType {
    #[default]
    kFocus,
    kBlur,
}
#[derive(Clone, Debug, Default)]
pub struct MouseEvent {
    pub r#type: MouseEventType,
    pub position: Offset,
    pub button: MouseButton,
    /// DOM buttons bit mask sampled by the host, including held buttons on move.
    pub buttons: u16,
    /// Native multi-click count carried by WebMouseEvent. The translated
    /// MouseEventManager owns click dispatch; this value is not a request to
    /// synthesize a DOM click in the host.
    pub click_count: i32,
    pub modifiers: EventModifiers,
    pub target_node_id: Option<u64>,
}
#[derive(Clone, Debug)]
pub struct PointerEvent {
    pub r#type: PointerEventType,
    pub pointer_type: PointerType,
    pub pointer_id: u32,
    pub position: Offset,
    pub button: MouseButton,
    pub buttons: u16,
    pub modifiers: EventModifiers,
    pub pressure: f64,
    pub primary: bool,
    pub target_node_id: Option<u64>,
    /// Device movement is independent of the absolute widget position.
    pub movement_x: i32,
    pub movement_y: i32,
    pub relative_motion: bool,
    pub width: f32,
    pub height: f32,
    pub hovering: bool,
    pub coalesced: Vec<PointerEvent>,
    pub predicted: Vec<PointerEvent>,
}
impl Default for PointerEvent {
    fn default() -> Self {
        Self {
            r#type: PointerEventType::kMove,
            pointer_type: PointerType::kMouse,
            pointer_id: 1,
            position: Offset::default(),
            button: MouseButton::kNone,
            buttons: 0,
            modifiers: EventModifiers::default(),
            pressure: 0.0,
            primary: true,
            target_node_id: None,
            movement_x: 0,
            movement_y: 0,
            relative_motion: false,
            width: 0.0,
            height: 0.0,
            hovering: false,
            coalesced: Vec::new(),
            predicted: Vec::new(),
        }
    }
}

/// Platform-neutral touch packet. Points include stationary contacts so ID
/// matching and sequence boundaries are decided by WebTouchEvent itself.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TouchEventType {
    #[default]
    kMove,
    kStart,
    kEnd,
    kCancel,
    kScrollStarted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TouchPointState {
    #[default]
    kUndefined,
    kReleased,
    kPressed,
    kMoved,
    kStationary,
    kCancelled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputDispatchType {
    #[default]
    kBlocking,
    kNonBlocking,
    kPassive,
    kForcedNonBlockingDueToFling,
}

#[derive(Clone, Debug)]
pub struct TouchPoint {
    pub pointer_id: u32,
    pub device_id: i32,
    pub pointer_type: PointerType,
    pub state: TouchPointState,
    pub position: Offset,
    pub movement_x: i32,
    pub movement_y: i32,
    pub pressure: f32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub rotation_angle: f32,
}

impl Default for TouchPoint {
    fn default() -> Self {
        Self {
            pointer_id: 0,
            device_id: -1,
            pointer_type: PointerType::kTouch,
            state: TouchPointState::kUndefined,
            position: Offset::default(),
            movement_x: 0,
            movement_y: 0,
            pressure: f32::NAN,
            radius_x: 0.0,
            radius_y: 0.0,
            rotation_angle: 0.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TouchEvent {
    pub timestamp: Option<std::time::Instant>,
    pub r#type: TouchEventType,
    pub points: Vec<TouchPoint>,
    pub modifiers: EventModifiers,
    pub dispatch_type: InputDispatchType,
    pub compositor_nonblocking: bool,
    pub compositor_nonblocking_due_to_fling: bool,
    pub moved_beyond_slop_region: bool,
    pub hovering: bool,
    pub touch_start_or_first_touch_move: bool,
    pub unique_touch_event_id: u32,
    pub coalesced: Vec<TouchEvent>,
    pub predicted: Vec<TouchEvent>,
}
/// Native wheel gesture boundary retained for safe host input coalescing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WheelPhase {
    #[default]
    kNone,
    kBegan,
    kChanged,
    kEnded,
    kCancelled,
}
/// WebInputEvent::RailsMode. On macOS Chromium derives this once at the
/// native event boundary and keeps the original wheel deltas for DOM event
/// dispatch; default scrolling consumes only the selected axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WheelRailsMode {
    #[default]
    kFree,
    kHorizontal,
    kVertical,
}
/// Original independent NSEvent phases (public NSEventPhase bits), plus the
/// boot-relative event timestamp. The effective WheelEvent.phase still drives
/// host dispatch; momentum identity must also survive coalescing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelMetadata {
    pub timestamp_seconds: f64,
    pub phase: u64,
    pub momentum_phase: u64,
}
#[derive(Clone, Debug, Default)]
pub struct WheelEvent {
    /// InputHandler disposition supplied by the compositor. DOM dispatch still
    /// runs; its default scroll is synchronized through main-frame deltas.
    pub compositor_owned: bool,
    /// Original impl-owned wheel callbacks. A coalesced DOM wheel acknowledges
    /// every original sample with the same canonical cancellation result.
    pub compositor_ack: Vec<u64>,
    /// Source raw wheel disposition. This is independent of whether a scroll
    /// begin is currently waiting for a main-thread hit-test callback.
    pub compositor_nonblocking: bool,
    pub phase: WheelPhase,
    /// WebMouseWheelEvent::has_synthetic_phase. The host phase handler sets
    /// this when it adds Begin/Changed/End around a phase-less wheel stream.
    pub has_synthetic_phase: bool,
    pub rails_mode: WheelRailsMode,
    pub native: Option<WheelMetadata>,
    /// Monotonic event time captured at the platform input boundary. Keeping
    /// this with the sample prevents queueing and impl-thread batching from
    /// changing the velocity seen by Chromium's elastic scroll controller.
    pub timestamp: Option<std::time::Instant>,
    pub delta_units: ScrollGranularity,
    pub position: Offset,
    pub delta: Offset,
    pub modifiers: EventModifiers,
    pub target_node_id: Option<u64>,
}

// ui/events/types/scroll_types.h. Native wheel deltas are already expressed
// in pixels; precise pixels retain the device distinction for input coalescing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ScrollGranularity {
    kScrollByPrecisePixel,
    #[default]
    kScrollByPixel,
    kScrollByLine,
    kScrollByPage,
    kScrollByDocument,
}

impl WheelEvent {
    /// MouseWheelEventQueue.cc:141-229. Host deltas already use scroll-offset
    /// sign; source wheel-to-gesture conversion preserves that sign roundtrip.
    pub fn GestureScrollDelta(&self) -> (Offset, Offset) {
        let mut delta = self.delta;
        #[cfg(not(target_os = "macos"))]
        if self.modifiers.shift && delta.x == 0.0 {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }
        if self.delta_units == ScrollGranularity::kScrollByPage {
            let sign = |value: f64| {
                if value > 0.0 {
                    1.0
                } else if value < 0.0 {
                    -1.0
                } else {
                    0.0
                }
            };
            delta = Offset {
                x: sign(delta.x),
                y: sign(delta.y),
            };
            return (delta, delta);
        }
        let unconstrained = delta;
        match self.rails_mode {
            WheelRailsMode::kFree => {}
            WheelRailsMode::kHorizontal => delta.y = 0.,
            WheelRailsMode::kVertical => delta.x = 0.,
        }
        (delta, unconstrained)
    }
}

impl WheelEvent {
    /// WebMouseWheelEvent::CanCoalesce / HaveConsistentPhase: compare both
    /// original native phases, but not timestamps; reversal is not a boundary.
    pub fn CanCoalesce(&self, next: &Self) -> bool {
        self.compositor_owned == next.compositor_owned
            && self.modifiers == next.modifiers
            && self.delta_units == next.delta_units
            && self.phase == next.phase
            && self.has_synthetic_phase == next.has_synthetic_phase
            && self.rails_mode == next.rails_mode
            && match (self.native, next.native) {
                (Some(a), Some(b)) => a.phase == b.phase && a.momentum_phase == b.momentum_phase,
                (None, None) => true,
                _ => false,
            }
    }

    /// Platform-specific default scroll action after macOS wheel railing.
    /// Keep `delta` untouched so JavaScript observes the native event.
    pub fn DefaultScrollDelta(&self) -> Offset {
        match self.rails_mode {
            WheelRailsMode::kFree => self.delta,
            WheelRailsMode::kHorizontal => Offset {
                x: self.delta.x,
                y: 0.0,
            },
            WheelRailsMode::kVertical => Offset {
                x: 0.0,
                y: self.delta.y,
            },
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct KeyEvent {
    /// Explicit protocol Char input may target the current widget without a
    /// matching native RawKeyDown. Native paired Char events leave this false.
    pub is_standalone_char: bool,
    /// RawKeyDown and its Char retain the same native ingress timestamp.
    pub timestamp: Option<std::time::Instant>,
    /// Native embedder key-binding commands (Chromium's EditCommand side
    /// channel). Typed IDs preserve delivery order without exposing selectors.
    pub edit_commands:
        Vec<interaction_default::core::editing::commands::editor_command_table::EditingCommandType>,
    pub r#type: KeyEventType,
    pub key: String,
    /// UI Events `code`, derived from the physical key rather than the active
    /// keyboard layout (Blink's WebKeyboardEvent::dom_code).
    pub code: String,
    pub text: String,
    /// Native layout text before command/control/option modifiers. Access-key
    /// lookup uses this source field independently of the committed character.
    pub unmodified_text: String,
    pub native_key_code: i32,
    pub is_system_key: bool,
    pub is_browser_shortcut: bool,
    /// The platform IME consumed this physical key. Preserve RawKeyDown with
    /// VKEY_PROCESSKEY while composition/commit travels its own source path.
    pub ime_processed: bool,
    pub caps_lock: Option<bool>,
    /// Blink exposes WebKeyboardEvent::windows_key_code as KeyboardEvent.keyCode
    /// for keydown/keyup. Keypress derives its character code at dispatch.
    pub key_code: u32,
    /// DOM_KEY_LOCATION_STANDARD/LEFT/RIGHT/NUMPAD.
    pub location: u32,
    pub is_composing: bool,
    pub modifiers: EventModifiers,
    pub repeat: bool,
    pub target_node_id: Option<u64>,
}

/// Chromium retains the legacy Windows virtual-key value even on non-Windows
/// platforms because web content still consumes KeyboardEvent.keyCode/which.
pub fn LegacyKeyCode(key: &str) -> u32 {
    if key.len() == 1 {
        let byte = key.as_bytes()[0];
        return match byte {
            b'a'..=b'z' => u32::from(byte.to_ascii_uppercase()),
            b'A'..=b'Z' | b'0'..=b'9' => u32::from(byte),
            b';' | b':' => 186,
            b'=' | b'+' => 187,
            b',' | b'<' => 188,
            b'-' | b'_' => 189,
            b'.' | b'>' => 190,
            b'/' | b'?' => 191,
            b'`' | b'~' => 192,
            b'[' | b'{' => 219,
            b'\\' | b'|' => 220,
            b']' | b'}' => 221,
            b'\'' | b'"' => 222,
            _ => 0,
        };
    }
    match key {
        "Backspace" => 8,
        "Tab" => 9,
        "Enter" => 13,
        "Shift" => 16,
        "Control" => 17,
        "Alt" => 18,
        "CapsLock" => 20,
        "Escape" => 27,
        " " | "Spacebar" => 32,
        "PageUp" => 33,
        "PageDown" => 34,
        "End" => 35,
        "Home" => 36,
        "ArrowLeft" => 37,
        "ArrowUp" => 38,
        "ArrowRight" => 39,
        "ArrowDown" => 40,
        "Insert" => 45,
        "Delete" => 46,
        "Meta" => 91,
        "ContextMenu" => 93,
        "NumLock" => 144,
        "ScrollLock" => 145,
        _ if key.starts_with('F') => key[1..]
            .parse::<u32>()
            .ok()
            .filter(|number| (1..=24).contains(number))
            .map_or(0, |number| 111 + number),
        _ => 0,
    }
}

/// Best-effort DOM `code` for protocol input that did not supply a physical
/// code. Native input always supplies the real winit physical key.
pub fn DomCodeForKey(key: &str) -> String {
    if key.len() == 1 {
        let byte = key.as_bytes()[0];
        return match byte {
            b'a'..=b'z' | b'A'..=b'Z' => format!("Key{}", (byte as char).to_ascii_uppercase()),
            b'0'..=b'9' => format!("Digit{}", byte as char),
            b' ' => "Space".into(),
            _ => String::new(),
        };
    }
    match key {
        "Control" => "ControlLeft",
        "Shift" => "ShiftLeft",
        "Alt" => "AltLeft",
        "Meta" => "MetaLeft",
        value
            if matches!(
                value,
                "Backspace"
                    | "Tab"
                    | "Enter"
                    | "Escape"
                    | "PageUp"
                    | "PageDown"
                    | "End"
                    | "Home"
                    | "ArrowLeft"
                    | "ArrowUp"
                    | "ArrowRight"
                    | "ArrowDown"
                    | "Insert"
                    | "Delete"
                    | "ContextMenu"
                    | "CapsLock"
                    | "NumLock"
                    | "ScrollLock"
            ) || value.starts_with('F') =>
        {
            value
        }
        _ => "",
    }
    .into()
}
#[derive(Clone, Debug, Default)]
pub struct CompositionEvent {
    pub spans: Vec<interaction_default::core::editing::ime::ime_text_span::ImeTextSpan>,
    /// UTF-16 code-unit offsets within preedit, matching Blink's input contract.
    pub selection: Option<(usize, usize)>,
    pub r#type: CompositionEventType,
    pub data: String,
    pub target_node_id: Option<u64>,
}
#[derive(Clone, Debug, Default)]
pub struct TextInputEvent {
    pub spans: Vec<interaction_default::core::editing::ime::ime_text_span::ImeTextSpan>,
    pub text: String,
    pub target_node_id: Option<u64>,
}
#[derive(Clone, Debug, Default)]
pub struct FocusEvent {
    pub r#type: FocusEventType,
    pub target_node_id: u64,
    pub related_target_node_id: Option<u64>,
}
#[derive(Clone, Debug)]
pub enum InputEvent {
    Mouse(MouseEvent),
    Pointer(PointerEvent),
    Touch(TouchEvent),
    /// The recognizer already owns Chromium's platform-neutral gesture value,
    /// including its monotonic timestamp, contact area and touch sequence ID.
    Gesture(WebGestureEvent),
    Wheel(WheelEvent),
    Key(KeyEvent),
    Composition(CompositionEvent),
    TextInput(TextInputEvent),
    Focus(FocusEvent),
}

/// Adapter: a typed raw event, without the host's target-routing hints.
#[derive(Clone, Debug)]
pub enum ConvertedWebInputEvent {
    Mouse(WebMouseEvent),
    Pointer(WebPointerEvent),
    Touch(WebTouchEvent),
    Gesture(WebGestureEvent),
    Wheel(WebMouseWheelEvent),
    Keyboard(WebKeyboardEvent),
}

/// Adapter: IME, committed text and explicit focus use their own source paths,
/// rather than masquerading as raw keyboard or pointer events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostOnlyInputKind {
    Composition,
    TextInput,
    Focus,
}

/// Adapter: host information that cannot be represented by a raw Web event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebInputConversionError {
    SyntheticMouseEvent(MouseEventType),
    PointerBoundaryEvent(PointerEventType),
    PointerIdOutOfRange(u32),
    TouchPointLimitExceeded(usize),
    KeyboardTextTooLong,
    NativeWheelPhaseOutOfRange,
    HostOnly(HostOnlyInputKind),
}

impl EventModifiers {
    // Source: third_party/blink/public/common/input/web_input_event.h:87-103.
    // Adapter: this host API exposes only these four modifier bits.
    pub fn ToWebModifiers(self) -> i32 {
        (if self.alt { WebInputEvent::kAltKey } else { 0 })
            | (if self.control {
                WebInputEvent::kControlKey
            } else {
                0
            })
            | (if self.meta {
                WebInputEvent::kMetaKey
            } else {
                0
            })
            | (if self.shift {
                WebInputEvent::kShiftKey
            } else {
                0
            })
    }
}

impl MouseButton {
    pub fn ButtonsMask(self) -> u16 {
        match self {
            Self::kNone => 0,
            Self::kPrimary => 1,
            Self::kSecondary => 2,
            Self::kMiddle => 4,
        }
    }
    // Source: third_party/blink/public/mojom/input/input_event.mojom:7-17.
    // Adapter: explicit mapping avoids relying on the host enum's discriminants.
    pub fn ToWebButton(self) -> Button {
        match self {
            Self::kNone => Button::kNoButton,
            Self::kPrimary => Button::kLeft,
            Self::kMiddle => Button::kMiddle,
            Self::kSecondary => Button::kRight,
        }
    }

    // Adapter: only a down sample proves this button is held. The host API has
    // no complete buttons mask; up/move cannot reconstruct other held buttons.
    fn DownModifier(self) -> i32 {
        match self {
            Self::kNone => 0,
            Self::kPrimary => WebInputEvent::kLeftButtonDown,
            Self::kMiddle => WebInputEvent::kMiddleButtonDown,
            Self::kSecondary => WebInputEvent::kRightButtonDown,
        }
    }
}

fn ButtonsModifiers(buttons: u16) -> i32 {
    (if buttons & 1 != 0 {
        WebInputEvent::kLeftButtonDown
    } else {
        0
    }) | (if buttons & 2 != 0 {
        WebInputEvent::kRightButtonDown
    } else {
        0
    }) | (if buttons & 4 != 0 {
        WebInputEvent::kMiddleButtonDown
    } else {
        0
    }) | (if buttons & 8 != 0 {
        WebInputEvent::kBackButtonDown
    } else {
        0
    }) | (if buttons & 16 != 0 {
        WebInputEvent::kForwardButtonDown
    } else {
        0
    })
}

// Source: ui/events/blink/web_input_event.cc:411-438. Mouse move/drag UI
// events carry the currently pressed buttons in their flags. Chromium derives
// WebMouseEvent::button from those flags even though no button changed on the
// move itself. EventHandler uses that field to distinguish a real drag from a
// move delivered after a lost native release.
fn ButtonFromButtons(buttons: u16) -> Button {
    let mut button = Button::kNoButton;
    if buttons & 1 != 0 {
        button = Button::kLeft;
    }
    if buttons & 4 != 0 {
        button = Button::kMiddle;
    }
    if buttons & 2 != 0 {
        button = Button::kRight;
    }
    button
}

impl MouseEvent {
    // Source: third_party/blink/public/common/input/web_mouse_event.h:30-51.
    // Adapter: click/dblclick are DOM synthesis requests, not raw Web event
    // types. Widget coordinates are supplied; screen coordinates, movement and
    // a full pressed-buttons mask are absent from this host contract and remain
    // at source defaults. Routing target_node_id is deliberately not projected.
    pub fn ToWebEvent(
        &self,
        time_stamp: TimeTicks,
    ) -> Result<WebMouseEvent, WebInputConversionError> {
        self.ToWebEventWithWidgetScale(time_stamp, 1.0)
    }

    /// The Page host contract carries CSS viewport coordinates, while Blink's
    /// WebMouseEvent carries root-widget coordinates. Apply the current layout
    /// zoom exactly once at this adapter boundary; MouseEvent construction
    /// converts the root-frame point back to CSS client coordinates.
    pub fn ToWebEventWithWidgetScale(
        &self,
        time_stamp: TimeTicks,
        widget_scale: f32,
    ) -> Result<WebMouseEvent, WebInputConversionError> {
        let event_type = match self.r#type {
            MouseEventType::kMove => WebEventType::kMouseMove,
            MouseEventType::kDown => WebEventType::kMouseDown,
            MouseEventType::kUp => WebEventType::kMouseUp,
            MouseEventType::kContextMenu => WebEventType::kContextMenu,
            MouseEventType::kEnter => WebEventType::kMouseEnter,
            MouseEventType::kLeave => WebEventType::kMouseLeave,
            MouseEventType::kClick | MouseEventType::kDoubleClick => {
                return Err(WebInputConversionError::SyntheticMouseEvent(self.r#type));
            }
        };
        let mut modifiers = self.modifiers.ToWebModifiers() | ButtonsModifiers(self.buttons);
        if self.r#type == MouseEventType::kDown {
            modifiers |= self.button.DownModifier();
        }
        let mut event = WebMouseEvent::WithType(
            event_type,
            modifiers,
            time_stamp,
            WebMouseEvent::kMousePointerId,
        );
        event.pointer_properties.SetPositionInWidgetXY(
            self.position.x as f32 * widget_scale,
            self.position.y as f32 * widget_scale,
        );
        event.pointer_properties.button = if matches!(
            self.r#type,
            MouseEventType::kMove | MouseEventType::kEnter | MouseEventType::kLeave
        ) && self.button == MouseButton::kNone
        {
            ButtonFromButtons(self.buttons)
        } else {
            self.button.ToWebButton()
        };
        event.click_count = self.click_count;
        Ok(event)
    }
}

impl PointerType {
    // Source: ui/events/event_constants.h:211-219.
    pub fn ToWebPointerType(self) -> WebPointerType {
        match self {
            Self::kMouse => WebPointerType::kMouse,
            Self::kPen => WebPointerType::kPen,
            Self::kTouch => WebPointerType::kTouch,
        }
    }
}

impl PointerEvent {
    /// Adapter projection after source queue coalescing. Host-only routing and
    /// primary-pointer hints remain on this value; source owns pointer identity.
    pub(crate) fn ProjectWebEvent(&mut self, event: &WebPointerEvent) {
        let modifiers = event.GetModifiers();
        self.buttons = (if modifiers & WebInputEvent::kLeftButtonDown != 0 {
            1
        } else {
            0
        }) | (if modifiers & WebInputEvent::kRightButtonDown != 0 {
            2
        } else {
            0
        }) | (if modifiers & WebInputEvent::kMiddleButtonDown != 0 {
            4
        } else {
            0
        }) | (if modifiers & WebInputEvent::kBackButtonDown != 0 {
            8
        } else {
            0
        }) | (if modifiers & WebInputEvent::kForwardButtonDown != 0 {
            16
        } else {
            0
        });
        self.r#type = match event.GetType() {
            WebEventType::kPointerDown => PointerEventType::kDown,
            WebEventType::kPointerUp => PointerEventType::kUp,
            WebEventType::kPointerCancel => PointerEventType::kCancel,
            WebEventType::kPointerRawUpdate => PointerEventType::kRawUpdate,
            WebEventType::kPointerMove => PointerEventType::kMove,
            _ => unreachable!("host pointer packet type"),
        };
        let position = event.pointer_properties.PositionInWidget();
        self.position = Offset {
            x: f64::from(position.x()),
            y: f64::from(position.y()),
        };
        self.movement_x = event.pointer_properties.movement_x;
        self.movement_y = event.pointer_properties.movement_y;
        self.pressure = f64::from(event.pointer_properties.force);
        self.width = event.width;
        self.height = event.height;
        self.hovering = event.hovering;
        self.relative_motion = event.GetModifiers() & WebInputEvent::kRelativeMotionEvent != 0;
        self.button = match event.pointer_properties.button {
            Button::kLeft => MouseButton::kPrimary,
            Button::kMiddle => MouseButton::kMiddle,
            Button::kRight => MouseButton::kSecondary,
            Button::kNoButton => MouseButton::kNone,
            _ => unreachable!("host-supported pointer button"),
        };
    }
    // Source: third_party/blink/public/common/input/web_pointer_event.h:24-40.
    // Adapter: primary is a DOM pointer-manager fact, not a raw Web property.
    // It remains on this borrowed host value, along with target_node_id.
    // Enter/leave likewise go through source boundary dispatch, not pointermove.
    pub fn ToWebEvent(
        &self,
        time_stamp: TimeTicks,
    ) -> Result<WebPointerEvent, WebInputConversionError> {
        self.ToWebEventWithWidgetScale(time_stamp, 1.0)
    }

    pub fn ToWebEventWithWidgetScale(
        &self,
        time_stamp: TimeTicks,
        widget_scale: f32,
    ) -> Result<WebPointerEvent, WebInputConversionError> {
        let event_type = match self.r#type {
            PointerEventType::kMove => WebEventType::kPointerMove,
            PointerEventType::kRawUpdate => WebEventType::kPointerRawUpdate,
            PointerEventType::kDown => WebEventType::kPointerDown,
            PointerEventType::kUp => WebEventType::kPointerUp,
            PointerEventType::kCancel => WebEventType::kPointerCancel,
            PointerEventType::kEnter | PointerEventType::kLeave => {
                return Err(WebInputConversionError::PointerBoundaryEvent(self.r#type));
            }
        };
        let id = i32::try_from(self.pointer_id)
            .map_err(|_| WebInputConversionError::PointerIdOutOfRange(self.pointer_id))?;
        let mut event = WebPointerEvent::default();
        event.SetType(event_type);
        event.SetTimeStamp(time_stamp);
        let mut modifiers = self.modifiers.ToWebModifiers() | ButtonsModifiers(self.buttons);
        if self.r#type == PointerEventType::kDown {
            modifiers |= self.button.DownModifier();
        }
        event.SetModifiers(modifiers);
        if self.relative_motion {
            event.SetModifiers(modifiers | WebInputEvent::kRelativeMotionEvent);
        }
        event.pointer_properties.id = id;
        event.pointer_properties.pointer_type = self.pointer_type.ToWebPointerType();
        event.pointer_properties.button = self.button.ToWebButton();
        event.pointer_properties.force = self.pressure as f32;
        event.pointer_properties.movement_x = self.movement_x;
        event.pointer_properties.movement_y = self.movement_y;
        event.width = self.width * widget_scale;
        event.height = self.height * widget_scale;
        event.hovering = self.hovering;
        event.pointer_properties.SetPositionInWidgetXY(
            self.position.x as f32 * widget_scale,
            self.position.y as f32 * widget_scale,
        );
        // Adapter: contact geometry, hover state, screen position, movement,
        // twist and tilt are unavailable; retain source defaults.
        Ok(event)
    }
}

impl InputDispatchType {
    pub fn ToWebDispatchType(self) -> WebDispatchType {
        match self {
            Self::kBlocking => WebDispatchType::kBlocking,
            Self::kNonBlocking => WebDispatchType::kEventNonBlocking,
            Self::kPassive => WebDispatchType::kListenersNonBlockingPassive,
            Self::kForcedNonBlockingDueToFling => {
                WebDispatchType::kListenersForcedNonBlockingDueToFling
            }
        }
    }
    pub fn FromWebDispatchType(value: WebDispatchType) -> Self {
        match value {
            WebDispatchType::kBlocking => Self::kBlocking,
            WebDispatchType::kEventNonBlocking => Self::kNonBlocking,
            WebDispatchType::kListenersNonBlockingPassive => Self::kPassive,
            WebDispatchType::kListenersForcedNonBlockingDueToFling => {
                Self::kForcedNonBlockingDueToFling
            }
        }
    }
}

impl TouchPointState {
    pub fn ToWebState(self) -> WebTouchState {
        match self {
            Self::kUndefined => WebTouchState::kStateUndefined,
            Self::kReleased => WebTouchState::kStateReleased,
            Self::kPressed => WebTouchState::kStatePressed,
            Self::kMoved => WebTouchState::kStateMoved,
            Self::kStationary => WebTouchState::kStateStationary,
            Self::kCancelled => WebTouchState::kStateCancelled,
        }
    }
    pub fn FromWebState(value: WebTouchState) -> Self {
        match value {
            WebTouchState::kStateUndefined => Self::kUndefined,
            WebTouchState::kStateReleased => Self::kReleased,
            WebTouchState::kStatePressed => Self::kPressed,
            WebTouchState::kStateMoved => Self::kMoved,
            WebTouchState::kStateStationary => Self::kStationary,
            WebTouchState::kStateCancelled => Self::kCancelled,
        }
    }
}

impl TouchEvent {
    pub(crate) fn ProjectWebEvent(&mut self, event: &WebTouchEvent) {
        self.dispatch_type = InputDispatchType::FromWebDispatchType(event.dispatch_type);
        self.moved_beyond_slop_region = event.moved_beyond_slop_region;
        self.hovering = event.hovering;
        self.touch_start_or_first_touch_move = event.touch_start_or_first_touch_move;
        self.unique_touch_event_id = event.unique_touch_event_id;
        // Source coalescing replaces the array with the newer host packet.
        // Match by ID because its source algorithm permits reordered contacts.
        for point in &mut self.points {
            let source = event.touches[..event.touches_length]
                .iter()
                .find(|source| {
                    u32::try_from(source.pointer_properties.id).ok() == Some(point.pointer_id)
                })
                .expect("source touch retains its host contact IDs");
            point.state = TouchPointState::FromWebState(source.state);
            let position = source.pointer_properties.PositionInWidget();
            point.position = Offset {
                x: f64::from(position.x()),
                y: f64::from(position.y()),
            };
            point.movement_x = source.pointer_properties.movement_x;
            point.movement_y = source.pointer_properties.movement_y;
            point.pressure = source.pointer_properties.force;
            point.radius_x = source.radius_x;
            point.radius_y = source.radius_y;
            point.rotation_angle = source.rotation_angle;
        }
    }
    pub fn ToWebEvent(
        &self,
        time_stamp: TimeTicks,
    ) -> Result<WebTouchEvent, WebInputConversionError> {
        self.ToWebEventWithWidgetScale(time_stamp, 1.0)
    }

    pub fn ToWebEventWithWidgetScale(
        &self,
        time_stamp: TimeTicks,
        widget_scale: f32,
    ) -> Result<WebTouchEvent, WebInputConversionError> {
        if self.points.len() > WebTouchEvent::kTouchesLengthCap {
            return Err(WebInputConversionError::TouchPointLimitExceeded(
                self.points.len(),
            ));
        }
        let event_type = match self.r#type {
            TouchEventType::kMove => WebEventType::kTouchMove,
            TouchEventType::kStart => WebEventType::kTouchStart,
            TouchEventType::kEnd => WebEventType::kTouchEnd,
            TouchEventType::kCancel => WebEventType::kTouchCancel,
            TouchEventType::kScrollStarted => WebEventType::kTouchScrollStarted,
        };
        let mut event = WebTouchEvent::new(event_type, self.modifiers.ToWebModifiers(), time_stamp);
        event.touches_length = self.points.len();
        for (index, point) in self.points.iter().enumerate() {
            let id = i32::try_from(point.pointer_id)
                .map_err(|_| WebInputConversionError::PointerIdOutOfRange(point.pointer_id))?;
            let mut pointer = interaction_input::public::common::input::web_pointer_properties::WebPointerProperties::WithId(id);
            pointer.pointer_type = point.pointer_type.ToWebPointerType();
            pointer.device_id = point.device_id;
            pointer.SetPositionInWidgetXY(
                point.position.x as f32 * widget_scale,
                point.position.y as f32 * widget_scale,
            );
            pointer.movement_x = point.movement_x;
            pointer.movement_y = point.movement_y;
            pointer.force = point.pressure;
            let mut touch = WebTouchPoint::new(pointer);
            touch.state = point.state.ToWebState();
            touch.radius_x = point.radius_x * widget_scale;
            touch.radius_y = point.radius_y * widget_scale;
            touch.rotation_angle = point.rotation_angle;
            event.touches[index] = touch;
        }
        event.dispatch_type = self.dispatch_type.ToWebDispatchType();
        event.moved_beyond_slop_region = self.moved_beyond_slop_region;
        event.hovering = self.hovering;
        event.touch_start_or_first_touch_move = self.touch_start_or_first_touch_move;
        event.unique_touch_event_id = self.unique_touch_event_id;
        Ok(event)
    }
}

impl ScrollGranularity {
    // Source: ui/events/types/scroll_types.h:12-20.
    pub fn ToWebGranularity(self) -> WebScrollGranularity {
        match self {
            Self::kScrollByPrecisePixel => WebScrollGranularity::kScrollByPrecisePixel,
            Self::kScrollByPixel => WebScrollGranularity::kScrollByPixel,
            Self::kScrollByLine => WebScrollGranularity::kScrollByLine,
            Self::kScrollByPage => WebScrollGranularity::kScrollByPage,
            Self::kScrollByDocument => WebScrollGranularity::kScrollByDocument,
        }
    }
}

impl WheelPhase {
    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:17-40.
    pub fn ToWebPhase(self) -> Phase {
        match self {
            Self::kNone => Phase::kPhaseNone,
            Self::kBegan => Phase::kPhaseBegan,
            Self::kChanged => Phase::kPhaseChanged,
            Self::kEnded => Phase::kPhaseEnded,
            Self::kCancelled => Phase::kPhaseCancelled,
        }
    }
}

impl WheelRailsMode {
    // Source: third_party/blink/public/mojom/input/input_event.mojom:153-160.
    pub fn ToWebRailsMode(self) -> RailsMode {
        match self {
            Self::kFree => RailsMode::kRailsModeFree,
            Self::kHorizontal => RailsMode::kRailsModeHorizontal,
            Self::kVertical => RailsMode::kRailsModeVertical,
        }
    }
}

impl WheelEvent {
    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:17-85.
    // Adapter: Rechrom host deltas use scroll-offset/DOM sign; raw Web wheel
    // deltas use the opposite sign. Preserve both native NSEvent phase masks,
    // whose public bits match WebMouseWheelEvent::Phase, instead of folding
    // momentum into the effective host phase. Synthetic phase-less streams use
    // the phase assigned at the host boundary.
    pub fn ToWebEvent(
        &self,
        time_stamp: TimeTicks,
    ) -> Result<WebMouseWheelEvent, WebInputConversionError> {
        self.ToWebEventWithWidgetScale(time_stamp, 1.0)
    }

    /// Rechrom's host wheel position is expressed in CSS viewport pixels;
    /// Blink's WebMouseWheelEvent carries root-widget coordinates. Preserve
    /// the same one-time CSS-to-widget conversion used by native mouse input.
    pub fn ToWebEventWithWidgetScale(
        &self,
        time_stamp: TimeTicks,
        widget_scale: f32,
    ) -> Result<WebMouseWheelEvent, WebInputConversionError> {
        let mut event = WebMouseWheelEvent::new(
            WebEventType::kMouseWheel,
            self.modifiers.ToWebModifiers(),
            time_stamp,
        );
        event.pointer_properties.SetPositionInWidgetXY(
            self.position.x as f32 * widget_scale,
            self.position.y as f32 * widget_scale,
        );
        if self.compositor_nonblocking {
            event.dispatch_type = interaction_input::public::common::input::web_input_event::DispatchType::kEventNonBlocking;
        }
        event.delta_x = -self.delta.x as f32;
        event.delta_y = -self.delta.y as f32;
        event.delta_units = self.delta_units.ToWebGranularity();
        event.rails_mode = self.rails_mode.ToWebRailsMode();
        event.has_synthetic_phase = self.has_synthetic_phase;
        if let Some(native) = self.native {
            event.phase = Phase(
                u32::try_from(native.phase)
                    .map_err(|_| WebInputConversionError::NativeWheelPhaseOutOfRange)?,
            );
            event.momentum_phase = Phase(
                u32::try_from(native.momentum_phase)
                    .map_err(|_| WebInputConversionError::NativeWheelPhaseOutOfRange)?,
            );
            if self.has_synthetic_phase {
                event.phase = self.phase.ToWebPhase();
            }
        } else {
            event.phase = self.phase.ToWebPhase();
        }
        // Adapter: compositor_owned is a routing/acknowledgement fact, not
        // WebMouseWheelEvent::DispatchType (which expresses cancelability).
        // Wheel ticks, unaccelerated deltas, and screen position are absent.
        // event_action is filled by the source wheel pipeline, not this adapter.
        Ok(event)
    }
}

impl KeyEvent {
    // Source: third_party/blink/public/common/input/web_keyboard_event.h:19-95.
    // Native ingress preserves RawKeyDown/Char/KeyUp as separate transactions.
    // IME commitment never becomes a synthetic keyboard character event.
    pub fn ToWebEvent(
        &self,
        time_stamp: TimeTicks,
    ) -> Result<WebKeyboardEvent, WebInputConversionError> {
        let mut modifiers = self.modifiers.ToWebModifiers();
        modifiers |= match self.location {
            1 => WebInputEvent::kIsLeft,
            2 => WebInputEvent::kIsRight,
            3 => WebInputEvent::kIsKeyPad,
            _ => 0,
        };
        if self.repeat {
            modifiers |= WebInputEvent::kIsAutoRepeat;
        }
        if self.is_composing || self.ime_processed {
            modifiers |= WebInputEvent::kIsComposing;
        }
        if self.caps_lock == Some(true) {
            modifiers |= WebInputEvent::kCapsLockOn;
        }
        let mut event = WebKeyboardEvent::new(
            match self.r#type {
                KeyEventType::kDown => WebEventType::kRawKeyDown,
                KeyEventType::kUp => WebEventType::kKeyUp,
                KeyEventType::kChar => WebEventType::kChar,
            },
            modifiers,
            time_stamp,
        );
        event.windows_key_code = if self.ime_processed && self.r#type == KeyEventType::kDown {
            229
        } else {
            self.key_code as i32
        };
        event.native_key_code = self.native_key_code;
        event.is_system_key = self.is_system_key;
        event.is_browser_shortcut = self.is_browser_shortcut;
        event.dom_code = WebDomCode(&self.code);
        event.dom_key = WebDomKey(&self.key);
        for (index, unit) in self.text.encode_utf16().enumerate() {
            if index >= WebKeyboardEvent::kTextLengthCap {
                return Err(WebInputConversionError::KeyboardTextTooLong);
            }
            event.text[index] = unit;
        }
        for (index, unit) in self.unmodified_text.encode_utf16().enumerate() {
            if index >= WebKeyboardEvent::kTextLengthCap {
                return Err(WebInputConversionError::KeyboardTextTooLong);
            }
            event.unmodified_text[index] = unit;
        }
        Ok(event)
    }
}

impl InputEvent {
    // Adapter: the single facade entry point preserves separate raw-Web and
    // host-only source paths. It does not dispatch events or run default actions.
    pub fn ToWebEvent(
        &self,
        time_stamp: TimeTicks,
    ) -> Result<ConvertedWebInputEvent, WebInputConversionError> {
        self.ToWebEventWithWidgetScale(time_stamp, 1.0)
    }

    pub fn ToWebEventWithWidgetScale(
        &self,
        time_stamp: TimeTicks,
        widget_scale: f32,
    ) -> Result<ConvertedWebInputEvent, WebInputConversionError> {
        match self {
            Self::Mouse(event) => event
                .ToWebEventWithWidgetScale(time_stamp, widget_scale)
                .map(ConvertedWebInputEvent::Mouse),
            Self::Pointer(event) => event
                .ToWebEventWithWidgetScale(time_stamp, widget_scale)
                .map(ConvertedWebInputEvent::Pointer),
            Self::Touch(event) => event
                .ToWebEventWithWidgetScale(time_stamp, widget_scale)
                .map(ConvertedWebInputEvent::Touch),
            // A recognized gesture carries the same source timestamp and
            // FrameScale/FrameTranslate as its originating touch sequence.
            // WidgetEventHandler transforms that value, not a host projection.
            Self::Gesture(event) => Ok(ConvertedWebInputEvent::Gesture(event.clone())),
            Self::Wheel(event) => event
                .ToWebEventWithWidgetScale(time_stamp, widget_scale)
                .map(ConvertedWebInputEvent::Wheel),
            Self::Key(event) => event
                .ToWebEvent(time_stamp)
                .map(ConvertedWebInputEvent::Keyboard),
            Self::Composition(_) => Err(WebInputConversionError::HostOnly(
                HostOnlyInputKind::Composition,
            )),
            Self::TextInput(_) => Err(WebInputConversionError::HostOnly(
                HostOnlyInputKind::TextInput,
            )),
            Self::Focus(_) => Err(WebInputConversionError::HostOnly(HostOnlyInputKind::Focus)),
        }
    }
}

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: ui/events/keycodes/dom/keycode_converter.cc:306-315.
// Source: ui/events/keycodes/dom/dom_code_data.inc:86-613.
// Adapter: the host carries the UI Events string, WebKeyboardEvent carries the
// USB usage value. Unknown/empty strings map to DomCode::NONE as in Chromium.
fn WebDomCode(code: &str) -> i32 {
    match code {
        "Hyper" => 0x000010,
        "Super" => 0x000011,
        "FnLock" => 0x000013,
        "Suspend" => 0x000014,
        "Resume" => 0x000015,
        "Turbo" => 0x000016,
        "Sleep" => 0x010082,
        "WakeUp" => 0x010083,
        "Fn" => 0x010097,
        "DisplayToggleIntExt" => 0x0100b5,
        "KeyA" => 0x070004,
        "KeyB" => 0x070005,
        "KeyC" => 0x070006,
        "KeyD" => 0x070007,
        "KeyE" => 0x070008,
        "KeyF" => 0x070009,
        "KeyG" => 0x07000a,
        "KeyH" => 0x07000b,
        "KeyI" => 0x07000c,
        "KeyJ" => 0x07000d,
        "KeyK" => 0x07000e,
        "KeyL" => 0x07000f,
        "KeyM" => 0x070010,
        "KeyN" => 0x070011,
        "KeyO" => 0x070012,
        "KeyP" => 0x070013,
        "KeyQ" => 0x070014,
        "KeyR" => 0x070015,
        "KeyS" => 0x070016,
        "KeyT" => 0x070017,
        "KeyU" => 0x070018,
        "KeyV" => 0x070019,
        "KeyW" => 0x07001a,
        "KeyX" => 0x07001b,
        "KeyY" => 0x07001c,
        "KeyZ" => 0x07001d,
        "Digit1" => 0x07001e,
        "Digit2" => 0x07001f,
        "Digit3" => 0x070020,
        "Digit4" => 0x070021,
        "Digit5" => 0x070022,
        "Digit6" => 0x070023,
        "Digit7" => 0x070024,
        "Digit8" => 0x070025,
        "Digit9" => 0x070026,
        "Digit0" => 0x070027,
        "Enter" => 0x070028,
        "Escape" => 0x070029,
        "Backspace" => 0x07002a,
        "Tab" => 0x07002b,
        "Space" => 0x07002c,
        "Minus" => 0x07002d,
        "Equal" => 0x07002e,
        "BracketLeft" => 0x07002f,
        "BracketRight" => 0x070030,
        "Backslash" => 0x070031,
        "Semicolon" => 0x070033,
        "Quote" => 0x070034,
        "Backquote" => 0x070035,
        "Comma" => 0x070036,
        "Period" => 0x070037,
        "Slash" => 0x070038,
        "CapsLock" => 0x070039,
        "F1" => 0x07003a,
        "F2" => 0x07003b,
        "F3" => 0x07003c,
        "F4" => 0x07003d,
        "F5" => 0x07003e,
        "F6" => 0x07003f,
        "F7" => 0x070040,
        "F8" => 0x070041,
        "F9" => 0x070042,
        "F10" => 0x070043,
        "F11" => 0x070044,
        "F12" => 0x070045,
        "PrintScreen" => 0x070046,
        "ScrollLock" => 0x070047,
        "Pause" => 0x070048,
        "Insert" => 0x070049,
        "Home" => 0x07004a,
        "PageUp" => 0x07004b,
        "Delete" => 0x07004c,
        "End" => 0x07004d,
        "PageDown" => 0x07004e,
        "ArrowRight" => 0x07004f,
        "ArrowLeft" => 0x070050,
        "ArrowDown" => 0x070051,
        "ArrowUp" => 0x070052,
        "NumLock" => 0x070053,
        "NumpadDivide" => 0x070054,
        "NumpadMultiply" => 0x070055,
        "NumpadSubtract" => 0x070056,
        "NumpadAdd" => 0x070057,
        "NumpadEnter" => 0x070058,
        "Numpad1" => 0x070059,
        "Numpad2" => 0x07005a,
        "Numpad3" => 0x07005b,
        "Numpad4" => 0x07005c,
        "Numpad5" => 0x07005d,
        "Numpad6" => 0x07005e,
        "Numpad7" => 0x07005f,
        "Numpad8" => 0x070060,
        "Numpad9" => 0x070061,
        "Numpad0" => 0x070062,
        "NumpadDecimal" => 0x070063,
        "IntlBackslash" => 0x070064,
        "ContextMenu" => 0x070065,
        "Power" => 0x070066,
        "NumpadEqual" => 0x070067,
        "F13" => 0x070068,
        "F14" => 0x070069,
        "F15" => 0x07006a,
        "F16" => 0x07006b,
        "F17" => 0x07006c,
        "F18" => 0x07006d,
        "F19" => 0x07006e,
        "F20" => 0x07006f,
        "F21" => 0x070070,
        "F22" => 0x070071,
        "F23" => 0x070072,
        "F24" => 0x070073,
        "Open" => 0x070074,
        "Help" => 0x070075,
        "Select" => 0x070077,
        "Again" => 0x070079,
        "Undo" => 0x07007a,
        "Cut" => 0x07007b,
        "Copy" => 0x07007c,
        "Paste" => 0x07007d,
        "Find" => 0x07007e,
        "AudioVolumeMute" => 0x07007f,
        "AudioVolumeUp" => 0x070080,
        "AudioVolumeDown" => 0x070081,
        "NumpadComma" => 0x070085,
        "IntlRo" => 0x070087,
        "KanaMode" => 0x070088,
        "IntlYen" => 0x070089,
        "Convert" => 0x07008a,
        "NonConvert" => 0x07008b,
        "Lang1" => 0x070090,
        "Lang2" => 0x070091,
        "Lang3" => 0x070092,
        "Lang4" => 0x070093,
        "Lang5" => 0x070094,
        "Abort" => 0x07009b,
        "Props" => 0x0700a3,
        "NumpadParenLeft" => 0x0700b6,
        "NumpadParenRight" => 0x0700b7,
        "NumpadBackspace" => 0x0700bb,
        "NumpadMemoryStore" => 0x0700d0,
        "NumpadMemoryRecall" => 0x0700d1,
        "NumpadMemoryClear" => 0x0700d2,
        "NumpadMemoryAdd" => 0x0700d3,
        "NumpadMemorySubtract" => 0x0700d4,
        "NumpadClear" => 0x0700d8,
        "NumpadClearEntry" => 0x0700d9,
        "ControlLeft" => 0x0700e0,
        "ShiftLeft" => 0x0700e1,
        "AltLeft" => 0x0700e2,
        "MetaLeft" => 0x0700e3,
        "ControlRight" => 0x0700e4,
        "ShiftRight" => 0x0700e5,
        "AltRight" => 0x0700e6,
        "MetaRight" => 0x0700e7,
        "BrightnessUp" => 0x0c006f,
        "BrightnessDown" => 0x0c0070,
        "MediaPlay" => 0x0c00b0,
        "MediaPause" => 0x0c00b1,
        "MediaRecord" => 0x0c00b2,
        "MediaFastForward" => 0x0c00b3,
        "MediaRewind" => 0x0c00b4,
        "MediaTrackNext" => 0x0c00b5,
        "MediaTrackPrevious" => 0x0c00b6,
        "MediaStop" => 0x0c00b7,
        "Eject" => 0x0c00b8,
        "MediaPlayPause" => 0x0c00cd,
        "MediaSelect" => 0x0c0183,
        "LaunchMail" => 0x0c018a,
        "LaunchApp2" => 0x0c0192,
        "LaunchApp1" => 0x0c0194,
        "LaunchControlPanel" => 0x0c019f,
        "SelectTask" => 0x0c01a2,
        "LaunchScreenSaver" => 0x0c01b1,
        "LaunchAssistant" => 0x0c01cb,
        "BrowserSearch" => 0x0c0221,
        "BrowserHome" => 0x0c0223,
        "BrowserBack" => 0x0c0224,
        "BrowserForward" => 0x0c0225,
        "BrowserStop" => 0x0c0226,
        "BrowserRefresh" => 0x0c0227,
        "BrowserFavorites" => 0x0c022a,
        "ZoomToggle" => 0x0c0232,
        "MailReply" => 0x0c0289,
        "MailForward" => 0x0c028b,
        "MailSend" => 0x0c028c,
        "KeyboardLayoutSelect" => 0x0c029d,
        "ShowAllWindows" => 0x0c029f,
        _ => 0,
    }
}

// Source: ui/events/keycodes/dom/keycode_converter.cc:400-425.
// Source: ui/events/keycodes/dom/dom_key.h:45-82,140-150,177-183.
// Source: ui/events/keycodes/dom/dom_key_data.inc:26-540.
// Adapter: encode the host's UI Events key string as Chromium's tagged DomKey;
// a Unicode scalar is not the raw dom_key integer value.
fn WebDomKey(key: &str) -> u32 {
    const SYMBOLIC: u32 = 1 << 21;
    const CODEPOINT: u32 = 1 << 22;
    match key {
        "Backspace" => CODEPOINT | 0x0008,
        "Tab" => CODEPOINT | 0x0009,
        "Enter" => CODEPOINT | 0x000D,
        "Escape" => CODEPOINT | 0x001B,
        "Delete" => CODEPOINT | 0x007F,
        "Unidentified" => SYMBOLIC | 0x0001,
        "Accel" => SYMBOLIC | 0x0101,
        "Alt" => SYMBOLIC | 0x0102,
        "AltGraph" => SYMBOLIC | 0x0103,
        "CapsLock" => SYMBOLIC | 0x0104,
        "Control" => SYMBOLIC | 0x0105,
        "Fn" => SYMBOLIC | 0x0106,
        "FnLock" => SYMBOLIC | 0x0107,
        "Hyper" => SYMBOLIC | 0x0108,
        "Meta" => SYMBOLIC | 0x0109,
        "NumLock" => SYMBOLIC | 0x010A,
        "ScrollLock" => SYMBOLIC | 0x010C,
        "Shift" => SYMBOLIC | 0x010D,
        "Super" => SYMBOLIC | 0x010E,
        "Symbol" => SYMBOLIC | 0x010F,
        "SymbolLock" => SYMBOLIC | 0x0110,
        "ShiftLevel5" => SYMBOLIC | 0x0111,
        ".AltGraphLatch" => SYMBOLIC | 0x0112,
        "ArrowDown" => SYMBOLIC | 0x0301,
        "ArrowLeft" => SYMBOLIC | 0x0302,
        "ArrowRight" => SYMBOLIC | 0x0303,
        "ArrowUp" => SYMBOLIC | 0x0304,
        "End" => SYMBOLIC | 0x0305,
        "Home" => SYMBOLIC | 0x0306,
        "PageDown" => SYMBOLIC | 0x0307,
        "PageUp" => SYMBOLIC | 0x0308,
        "Clear" => SYMBOLIC | 0x0401,
        "Copy" => SYMBOLIC | 0x0402,
        "CrSel" => SYMBOLIC | 0x0403,
        "Cut" => SYMBOLIC | 0x0404,
        "EraseEof" => SYMBOLIC | 0x0405,
        "ExSel" => SYMBOLIC | 0x0406,
        "Insert" => SYMBOLIC | 0x0407,
        "Paste" => SYMBOLIC | 0x0408,
        "Redo" => SYMBOLIC | 0x0409,
        "Undo" => SYMBOLIC | 0x040A,
        "Accept" => SYMBOLIC | 0x0501,
        "Again" => SYMBOLIC | 0x0502,
        "Attn" => SYMBOLIC | 0x0503,
        "Cancel" => SYMBOLIC | 0x0504,
        "ContextMenu" => SYMBOLIC | 0x0505,
        "Execute" => SYMBOLIC | 0x0506,
        "Find" => SYMBOLIC | 0x0507,
        "Help" => SYMBOLIC | 0x0508,
        "Pause" => SYMBOLIC | 0x0509,
        "Play" => SYMBOLIC | 0x050A,
        "Props" => SYMBOLIC | 0x050B,
        "Select" => SYMBOLIC | 0x050C,
        "ZoomIn" => SYMBOLIC | 0x050D,
        "ZoomOut" => SYMBOLIC | 0x050E,
        "BrightnessDown" => SYMBOLIC | 0x0601,
        "BrightnessUp" => SYMBOLIC | 0x0602,
        "Eject" => SYMBOLIC | 0x0604,
        "LogOff" => SYMBOLIC | 0x0605,
        "Power" => SYMBOLIC | 0x0606,
        "PowerOff" => SYMBOLIC | 0x0607,
        "PrintScreen" => SYMBOLIC | 0x0608,
        "Hibernate" => SYMBOLIC | 0x0609,
        "Standby" => SYMBOLIC | 0x060A,
        "WakeUp" => SYMBOLIC | 0x060B,
        "AllCandidates" => SYMBOLIC | 0x0701,
        "Alphanumeric" => SYMBOLIC | 0x0702,
        "CodeInput" => SYMBOLIC | 0x0703,
        "Compose" => SYMBOLIC | 0x0704,
        "Convert" => SYMBOLIC | 0x0705,
        "FinalMode" => SYMBOLIC | 0x0706,
        "GroupFirst" => SYMBOLIC | 0x0707,
        "GroupLast" => SYMBOLIC | 0x0708,
        "GroupNext" => SYMBOLIC | 0x0709,
        "GroupPrevious" => SYMBOLIC | 0x070A,
        "ModeChange" => SYMBOLIC | 0x070B,
        "NextCandidate" => SYMBOLIC | 0x070C,
        "NonConvert" => SYMBOLIC | 0x070D,
        "PreviousCandidate" => SYMBOLIC | 0x070E,
        "Process" => SYMBOLIC | 0x070F,
        "SingleCandidate" => SYMBOLIC | 0x0710,
        "HangulMode" => SYMBOLIC | 0x0711,
        "HanjaMode" => SYMBOLIC | 0x0712,
        "JunjaMode" => SYMBOLIC | 0x0713,
        "Eisu" => SYMBOLIC | 0x0714,
        "Hankaku" => SYMBOLIC | 0x0715,
        "Hiragana" => SYMBOLIC | 0x0716,
        "HiraganaKatakana" => SYMBOLIC | 0x0717,
        "KanaMode" => SYMBOLIC | 0x0718,
        "KanjiMode" => SYMBOLIC | 0x0719,
        "Katakana" => SYMBOLIC | 0x071A,
        "Romaji" => SYMBOLIC | 0x071B,
        "Zenkaku" => SYMBOLIC | 0x071C,
        "ZenkakuHankaku" => SYMBOLIC | 0x071D,
        "F1" => SYMBOLIC | 0x0801,
        "F2" => SYMBOLIC | 0x0802,
        "F3" => SYMBOLIC | 0x0803,
        "F4" => SYMBOLIC | 0x0804,
        "F5" => SYMBOLIC | 0x0805,
        "F6" => SYMBOLIC | 0x0806,
        "F7" => SYMBOLIC | 0x0807,
        "F8" => SYMBOLIC | 0x0808,
        "F9" => SYMBOLIC | 0x0809,
        "F10" => SYMBOLIC | 0x080A,
        "F11" => SYMBOLIC | 0x080B,
        "F12" => SYMBOLIC | 0x080C,
        "F13" => SYMBOLIC | 0x080D,
        "F14" => SYMBOLIC | 0x080E,
        "F15" => SYMBOLIC | 0x080F,
        "F16" => SYMBOLIC | 0x0810,
        "F17" => SYMBOLIC | 0x0811,
        "F18" => SYMBOLIC | 0x0812,
        "F19" => SYMBOLIC | 0x0813,
        "F20" => SYMBOLIC | 0x0814,
        "F21" => SYMBOLIC | 0x0815,
        "F22" => SYMBOLIC | 0x0816,
        "F23" => SYMBOLIC | 0x0817,
        "F24" => SYMBOLIC | 0x0818,
        "Soft1" => SYMBOLIC | 0x0901,
        "Soft2" => SYMBOLIC | 0x0902,
        "Soft3" => SYMBOLIC | 0x0903,
        "Soft4" => SYMBOLIC | 0x0904,
        "Soft5" => SYMBOLIC | 0x0905,
        "Soft6" => SYMBOLIC | 0x0906,
        "Soft7" => SYMBOLIC | 0x0907,
        "Soft8" => SYMBOLIC | 0x0908,
        "ChannelDown" => SYMBOLIC | 0x0D0A,
        "ChannelUp" => SYMBOLIC | 0x0D0B,
        "Close" => SYMBOLIC | 0x0A01,
        "MailForward" => SYMBOLIC | 0x0A02,
        "MailReply" => SYMBOLIC | 0x0A03,
        "MailSend" => SYMBOLIC | 0x0A04,
        "MediaClose" => SYMBOLIC | 0x0D5B,
        "MediaFastForward" => SYMBOLIC | 0x0D2C,
        "MediaPause" => SYMBOLIC | 0x0D2E,
        "MediaPlay" => SYMBOLIC | 0x0D2F,
        "MediaPlayPause" => SYMBOLIC | 0x0A05,
        "MediaRecord" => SYMBOLIC | 0x0D30,
        "MediaRewind" => SYMBOLIC | 0x0D31,
        "MediaStop" => SYMBOLIC | 0x0A07,
        "MediaTrackNext" => SYMBOLIC | 0x0A08,
        "MediaTrackPrevious" => SYMBOLIC | 0x0A09,
        "New" => SYMBOLIC | 0x0A0A,
        "Open" => SYMBOLIC | 0x0A0B,
        "Print" => SYMBOLIC | 0x0A0C,
        "Save" => SYMBOLIC | 0x0A0D,
        "SpellCheck" => SYMBOLIC | 0x0A0E,
        "Key11" => SYMBOLIC | 0x1201,
        "Key12" => SYMBOLIC | 0x1202,
        "AudioBalanceLeft" => SYMBOLIC | 0x0D01,
        "AudioBalanceRight" => SYMBOLIC | 0x0D02,
        "AudioBassBoostDown" => SYMBOLIC | 0x0D03,
        "AudioBassBoostToggle" => SYMBOLIC | 0x0E02,
        "AudioBassBoostUp" => SYMBOLIC | 0x0D04,
        "AudioFaderFront" => SYMBOLIC | 0x0D05,
        "AudioFaderRear" => SYMBOLIC | 0x0D06,
        "AudioSurroundModeNext" => SYMBOLIC | 0x0D07,
        "AudioTrebleDown" => SYMBOLIC | 0x0E04,
        "AudioTrebleUp" => SYMBOLIC | 0x0E05,
        "AudioVolumeDown" => SYMBOLIC | 0x0A0F,
        "AudioVolumeUp" => SYMBOLIC | 0x0A10,
        "AudioVolumeMute" => SYMBOLIC | 0x0A11,
        "MicrophoneToggle" => SYMBOLIC | 0x0E06,
        "MicrophoneVolumeDown" => SYMBOLIC | 0x0E07,
        "MicrophoneVolumeUp" => SYMBOLIC | 0x0E08,
        "MicrophoneVolumeMute" => SYMBOLIC | 0x0E09,
        "SpeechCorrectionList" => SYMBOLIC | 0x0F01,
        "SpeechInputToggle" => SYMBOLIC | 0x0F02,
        "LaunchApplication1" => SYMBOLIC | 0x0B06,
        "LaunchApplication2" => SYMBOLIC | 0x0B01,
        "LaunchCalendar" => SYMBOLIC | 0x0B02,
        "LaunchContacts" => SYMBOLIC | 0x0B0C,
        "LaunchMail" => SYMBOLIC | 0x0B03,
        "LaunchMediaPlayer" => SYMBOLIC | 0x0B04,
        "LaunchMusicPlayer" => SYMBOLIC | 0x0B05,
        "LaunchPhone" => SYMBOLIC | 0x0B0D,
        "LaunchScreenSaver" => SYMBOLIC | 0x0B07,
        "LaunchSpreadsheet" => SYMBOLIC | 0x0B08,
        "LaunchWebBrowser" => SYMBOLIC | 0x0B09,
        "LaunchWebCam" => SYMBOLIC | 0x0B0A,
        "LaunchWordProcessor" => SYMBOLIC | 0x0B0B,
        "LaunchAssistant" => SYMBOLIC | 0x0B0E,
        "LaunchControlPanel" => SYMBOLIC | 0x0B0F,
        "BrowserBack" => SYMBOLIC | 0x0C01,
        "BrowserFavorites" => SYMBOLIC | 0x0C02,
        "BrowserForward" => SYMBOLIC | 0x0C03,
        "BrowserHome" => SYMBOLIC | 0x0C04,
        "BrowserRefresh" => SYMBOLIC | 0x0C05,
        "BrowserSearch" => SYMBOLIC | 0x0C06,
        "BrowserStop" => SYMBOLIC | 0x0C07,
        "AppSwitch" => SYMBOLIC | 0x1001,
        "Call" => SYMBOLIC | 0x1002,
        "Camera" => SYMBOLIC | 0x0603,
        "CameraFocus" => SYMBOLIC | 0x1003,
        "EndCall" => SYMBOLIC | 0x1004,
        "GoBack" => SYMBOLIC | 0x1005,
        "GoHome" => SYMBOLIC | 0x1006,
        "HeadsetHook" => SYMBOLIC | 0x1007,
        "LastNumberRedial" => SYMBOLIC | 0x1008,
        "Notification" => SYMBOLIC | 0x1009,
        "MannerMode" => SYMBOLIC | 0x100A,
        "VoiceDial" => SYMBOLIC | 0x100B,
        "TV" => SYMBOLIC | 0x0D49,
        "TV3DMode" => SYMBOLIC | 0x1101,
        "TVAntennaCable" => SYMBOLIC | 0x1102,
        "TVAudioDescription" => SYMBOLIC | 0x1103,
        "TVAudioDescriptionMixDown" => SYMBOLIC | 0x1104,
        "TVAudioDescriptionMixUp" => SYMBOLIC | 0x1105,
        "TVContentsMenu" => SYMBOLIC | 0x1106,
        "TVDataService" => SYMBOLIC | 0x1107,
        "TVInput" => SYMBOLIC | 0x0D4A,
        "TVInputComponent1" => SYMBOLIC | 0x1108,
        "TVInputComponent2" => SYMBOLIC | 0x1109,
        "TVInputComposite1" => SYMBOLIC | 0x110A,
        "TVInputComposite2" => SYMBOLIC | 0x110B,
        "TVInputHDMI1" => SYMBOLIC | 0x110C,
        "TVInputHDMI2" => SYMBOLIC | 0x110D,
        "TVInputHDMI3" => SYMBOLIC | 0x110E,
        "TVInputHDMI4" => SYMBOLIC | 0x110F,
        "TVInputVGA1" => SYMBOLIC | 0x1110,
        "TVMediaContext" => SYMBOLIC | 0x1111,
        "TVNetwork" => SYMBOLIC | 0x1112,
        "TVNumberEntry" => SYMBOLIC | 0x1113,
        "TVPower" => SYMBOLIC | 0x0D4B,
        "TVRadioService" => SYMBOLIC | 0x1114,
        "TVSatellite" => SYMBOLIC | 0x1115,
        "TVSatelliteBS" => SYMBOLIC | 0x1116,
        "TVSatelliteCS" => SYMBOLIC | 0x1117,
        "TVSatelliteToggle" => SYMBOLIC | 0x1118,
        "TVTerrestrialAnalog" => SYMBOLIC | 0x1119,
        "TVTerrestrialDigital" => SYMBOLIC | 0x111A,
        "TVTimer" => SYMBOLIC | 0x111B,
        "AVRInput" => SYMBOLIC | 0x0D08,
        "AVRPower" => SYMBOLIC | 0x0D09,
        "ColorF0Red" => SYMBOLIC | 0x0D0C,
        "ColorF1Green" => SYMBOLIC | 0x0D0D,
        "ColorF2Yellow" => SYMBOLIC | 0x0D0E,
        "ColorF3Blue" => SYMBOLIC | 0x0D0F,
        "ColorF4Grey" => SYMBOLIC | 0x0D10,
        "ColorF5Brown" => SYMBOLIC | 0x0D11,
        "ClosedCaptionToggle" => SYMBOLIC | 0x0D12,
        "Dimmer" => SYMBOLIC | 0x0D13,
        "DisplaySwap" => SYMBOLIC | 0x0D14,
        "DVR" => SYMBOLIC | 0x0D4F,
        "Exit" => SYMBOLIC | 0x0D15,
        "FavoriteClear0" => SYMBOLIC | 0x0D16,
        "FavoriteClear1" => SYMBOLIC | 0x0D17,
        "FavoriteClear2" => SYMBOLIC | 0x0D18,
        "FavoriteClear3" => SYMBOLIC | 0x0D19,
        "FavoriteRecall0" => SYMBOLIC | 0x0D1A,
        "FavoriteRecall1" => SYMBOLIC | 0x0D1B,
        "FavoriteRecall2" => SYMBOLIC | 0x0D1C,
        "FavoriteRecall3" => SYMBOLIC | 0x0D1D,
        "FavoriteStore0" => SYMBOLIC | 0x0D1E,
        "FavoriteStore1" => SYMBOLIC | 0x0D1F,
        "FavoriteStore2" => SYMBOLIC | 0x0D20,
        "FavoriteStore3" => SYMBOLIC | 0x0D21,
        "Guide" => SYMBOLIC | 0x0D22,
        "GuideNextDay" => SYMBOLIC | 0x0D23,
        "GuidePreviousDay" => SYMBOLIC | 0x0D24,
        "Info" => SYMBOLIC | 0x0D25,
        "InstantReplay" => SYMBOLIC | 0x0D26,
        "Link" => SYMBOLIC | 0x0D27,
        "ListProgram" => SYMBOLIC | 0x0D28,
        "LiveContent" => SYMBOLIC | 0x0D29,
        "Lock" => SYMBOLIC | 0x0D2A,
        "MediaApps" => SYMBOLIC | 0x0D2B,
        "MediaAudioTrack" => SYMBOLIC | 0x0D50,
        "MediaLast" => SYMBOLIC | 0x0D2D,
        "MediaSkipBackward" => SYMBOLIC | 0x0D51,
        "MediaSkipForward" => SYMBOLIC | 0x0D52,
        "MediaSkip" => SYMBOLIC | 0x0D32,
        "MediaStepBackward" => SYMBOLIC | 0x0D53,
        "MediaStepForward" => SYMBOLIC | 0x0D54,
        "MediaTopMenu" => SYMBOLIC | 0x0D55,
        "NavigateIn" => SYMBOLIC | 0x0D56,
        "NavigateNext" => SYMBOLIC | 0x0D57,
        "NavigateOut" => SYMBOLIC | 0x0D58,
        "NavigatePrevious" => SYMBOLIC | 0x0D59,
        "NextFavoriteChannel" => SYMBOLIC | 0x0D33,
        "NextUserProfile" => SYMBOLIC | 0x0D34,
        "OnDemand" => SYMBOLIC | 0x0D35,
        "Pairing" => SYMBOLIC | 0x0D5A,
        "PinPDown" => SYMBOLIC | 0x0D36,
        "PinPMove" => SYMBOLIC | 0x0D37,
        "PinPToggle" => SYMBOLIC | 0x0D38,
        "PinPUp" => SYMBOLIC | 0x0D39,
        "PlaySpeedDown" => SYMBOLIC | 0x0D3A,
        "PlaySpeedReset" => SYMBOLIC | 0x0D3B,
        "PlaySpeedUp" => SYMBOLIC | 0x0D3C,
        "RandomToggle" => SYMBOLIC | 0x0D3D,
        "RcLowBattery" => SYMBOLIC | 0x0D3E,
        "RecordSpeedNext" => SYMBOLIC | 0x0D3F,
        "RfBypass" => SYMBOLIC | 0x0D40,
        "ScanChannelsToggle" => SYMBOLIC | 0x0D41,
        "ScreenModeNext" => SYMBOLIC | 0x0D42,
        "Settings" => SYMBOLIC | 0x0D43,
        "SplitScreenToggle" => SYMBOLIC | 0x0D44,
        "STBInput" => SYMBOLIC | 0x0D45,
        "STBPower" => SYMBOLIC | 0x0D46,
        "Subtitle" => SYMBOLIC | 0x0D47,
        "Teletext" => SYMBOLIC | 0x0D48,
        "VideoModeNext" => SYMBOLIC | 0x0D4C,
        "Wink" => SYMBOLIC | 0x0D4D,
        "ZoomToggle" => SYMBOLIC | 0x0D4E,
        "Dead" => SYMBOLIC | CODEPOINT | 0xffff,
        _ => {
            let mut chars = key.chars();
            match (chars.next(), chars.next()) {
                (Some(character), None) => CODEPOINT | character as u32,
                _ => 0,
            }
        }
    }
}
