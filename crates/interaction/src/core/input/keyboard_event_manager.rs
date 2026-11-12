// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/keyboard_event_manager.h:1-91
// Source: third_party/blink/renderer/core/input/keyboard_event_manager.cc:1-699
// Rust modules replace includes/namespaces/GC annotations. Rc retains the live
// owners corresponding to Blink's Member fields; ports project those owners,
// never copied document/editor/navigation models. Cell and short RefCell
// borrows preserve synchronous dispatch reentry and the source AutoReset
// lifetime.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU8, Ordering};

use interaction_default::core::dom::element::SelectionBehaviorOnFocus;
use interaction_default::core::dom::focus_params::{FocusParams, FocusTrigger};
use interaction_default::core::html::focus_options::FocusOptions;
use interaction_default::core::page::focus_controller::FocusDispatchScope;
use interaction_default::public::mojom::input::focus_type::FocusType;
use interaction_dispatch::core::dom::events::event::EventTargetRef;
use interaction_dispatch::core::dom::events::event_dispatch_result::DispatchEventResult;
use interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope;
use interaction_dispatch::core::events::keyboard_event::{
    KeyboardEvent, KeyboardEventKeycodeConverter, KeyboardEventRef, KeyboardEventWindow,
};
use interaction_dispatch::core::events::ui_event::InputDeviceCapabilitiesRef;
use interaction_dispatch::core::input::event_handling_util::ToWebInputEventResult;
use interaction_dispatch::public::platform::web_input_event_result::WebInputEventResult;
use interaction_input::public::common::input::web_input_event::{Type, WebInputEvent};
use interaction_input::public::common::input::web_keyboard_event::WebKeyboardEvent;
use interaction_input::ui::events::types::scroll_types::ScrollGranularity;
use interaction_scroll::core::input::scroll_manager::{
    LogicalScrollResult, ScrollEffectEmitter, ScrollKeyboardEventManager, ScrollManager,
    ScrollNodeRef, ScrollableArea, ScrollableAreaRef,
};
use interaction_scroll::core::scroll::scroll_types::ScrollDirection;

// Source: keyboard_event_manager.h:26
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum OverrideCapsLockState {
    #[default]
    kDefault,
    kOn,
    kOff,
}

// Source: keyboard_event_manager.cc:53,55-65,67-189
const kVKeyProcessKey: i32 = 229;
// Source: third_party/blink/renderer/platform/keyboard_codes.h:71-72,112-134,567
// Source: third_party/blink/renderer/platform/windows_keyboard_codes.h:64-65
const VKEY_UNKNOWN: i32 = 0;
const VKEY_CAPITAL: i32 = 0x14;
const VKEY_PRIOR: i32 = 0x21;
const VKEY_NEXT: i32 = 0x22;
const VKEY_END: i32 = 0x23;
const VKEY_HOME: i32 = 0x24;
const VKEY_LEFT: i32 = 0x25;
const VKEY_UP: i32 = 0x26;
const VKEY_RIGHT: i32 = 0x27;
const VKEY_DOWN: i32 = 0x28;

// Source: keyboard_event_manager.cc:55-65
fn IsPageUpOrDownKeyEvent(key_code: i32, modifiers: i32) -> bool {
    if modifiers & WebInputEvent::kAltKey != 0 {
        key_code == VKEY_UP || key_code == VKEY_DOWN
    } else if key_code == VKEY_PRIOR || key_code == VKEY_NEXT {
        modifiers == WebInputEvent::kNoModifiers
    } else {
        false
    }
}

// Source: keyboard_event_manager.cc:67-189. An Option replaces the bool plus
// out parameters. The omitted WebFeature output is instrumentation only.
fn MapKeyCodeForScroll(
    mut key_code: i32,
    modifiers: i32,
    mac_disable_ctrl_home_end: bool,
) -> Option<(ScrollDirection, ScrollGranularity)> {
    if modifiers & (WebInputEvent::kShiftKey | WebInputEvent::kMetaKey) != 0 {
        return None;
    }
    // Source: keyboard_event_manager.cc:76-97
    #[cfg(target_os = "android")]
    if modifiers & WebInputEvent::kKeyModifiers
        == (WebInputEvent::kControlKey | WebInputEvent::kAltKey)
    {
        let direction = match key_code {
            VKEY_UP => Some(ScrollDirection::kScrollUpIgnoringWritingMode),
            VKEY_DOWN => Some(ScrollDirection::kScrollDownIgnoringWritingMode),
            _ => None,
        };
        if let Some(direction) = direction {
            // TODO(instrumentation): RecordKeyboardShortcutForAndroid(top/bottom).
            return Some((direction, ScrollGranularity::kScrollByDocument));
        }
    }
    // Source: keyboard_event_manager.cc:99-122
    if modifiers & WebInputEvent::kAltKey != 0 {
        key_code = match key_code {
            VKEY_UP => VKEY_PRIOR,
            VKEY_DOWN => VKEY_NEXT,
            _ => return None,
        };
    }
    if modifiers & WebInputEvent::kControlKey != 0 {
        #[cfg(target_os = "macos")]
        if mac_disable_ctrl_home_end {
            return None;
        }
        if key_code != VKEY_HOME && key_code != VKEY_END {
            return None;
        }
    }
    let _ = mac_disable_ctrl_home_end;
    // Source: keyboard_event_manager.cc:124-133
    // TODO(instrumentation): Android PageUp/PageDown shortcut recorder.
    // Source: keyboard_event_manager.cc:135-188
    Some(match key_code {
        VKEY_LEFT => (
            ScrollDirection::kScrollLeftIgnoringWritingMode,
            ScrollGranularity::kScrollByLine,
        ),
        VKEY_RIGHT => (
            ScrollDirection::kScrollRightIgnoringWritingMode,
            ScrollGranularity::kScrollByLine,
        ),
        VKEY_UP => (
            ScrollDirection::kScrollUpIgnoringWritingMode,
            ScrollGranularity::kScrollByLine,
        ),
        VKEY_DOWN => (
            ScrollDirection::kScrollDownIgnoringWritingMode,
            ScrollGranularity::kScrollByLine,
        ),
        VKEY_HOME => (
            ScrollDirection::kScrollUpIgnoringWritingMode,
            ScrollGranularity::kScrollByDocument,
        ),
        VKEY_END => (
            ScrollDirection::kScrollDownIgnoringWritingMode,
            ScrollGranularity::kScrollByDocument,
        ),
        VKEY_PRIOR => (
            ScrollDirection::kScrollUpIgnoringWritingMode,
            ScrollGranularity::kScrollByPage,
        ),
        VKEY_NEXT => (
            ScrollDirection::kScrollDownIgnoringWritingMode,
            ScrollGranularity::kScrollByPage,
        ),
        _ => return None,
    })
}

// Source: keyboard_event_manager.cc:214-222,246,299-307,324-327,357
// Handles below retain canonical node/target identity across listener reentry.
// DispatchEvent must synchronously use the existing dispatch/default path; it
// must not queue the DOM event or postpone its default actions.
pub trait KeyboardNode {
    fn EventTarget(&self) -> EventTargetRef;
    fn DispatchEvent(&self, event: &KeyboardEventRef) -> DispatchEventResult;
    fn UpdateHadKeyboardEvent(&self, event: &KeyboardEventRef);
    fn IsEditable(&self) -> bool;
    fn TextControlIsDisabledOrReadOnly(&self) -> Option<bool>;
    fn RoleAttribute(&self) -> Option<String>;
    fn ScrollNode(&self) -> ScrollNodeRef;
}
pub trait KeyboardAccessKeyElement {
    fn Focus(&self, params: &FocusParams);
    fn AccessKeyAction(&self, scope: SimulatedClickCreationScope);
}

// Source: keyboard_event_manager.cc:215-216,246,272,299-307,381,419-423,628
// Each operation is on the live Document. EventTargetNodeForDocument performs
// the owning document's canonical focused-element/body/document-element query.
pub trait KeyboardDocument<'scope> {
    fn EventTargetNodeForDocument(&self) -> Option<Rc<dyn KeyboardNode + 'scope>>;
    fn GetElementByAccessKey(
        &self,
        lowered_key: &[u16],
    ) -> Option<Rc<dyn KeyboardAccessKeyElement + 'scope>>;
    fn DomWindow(&self) -> &dyn KeyboardEventWindow;
    fn IsInWebAppScope(&self) -> bool;
    fn InDesignMode(&self) -> bool;
    fn FocusedInputCapsLockStateMayHaveChanged(&self);
    fn LoseInterestInAllElements(&self);
}

// Source: keyboard_event_manager.cc:253-254; DOM string operation at 214-216.
// Both services use their original Unicode/keycode implementation; UTF-16 is
// kept intact (including unpaired surrogates) instead of a lossy UTF-8 model.
pub trait KeyboardManagerKeycodeConverter: KeyboardEventKeycodeConverter {
    fn IsDomKeyForModifier(&self, dom_key: u32) -> bool;
    fn IsEscapeDomKey(&self, dom_key: u32) -> bool;
    fn EventKeycodeConverter(&self) -> &dyn KeyboardEventKeycodeConverter;
}
pub trait KeyboardStringOperations {
    fn DeprecatedLower(&self, text: &[u16]) -> Vec<u16>;
}

// Source: keyboard_event_manager.cc:273-280. This is only the widget's typed
// display-mode fact; the manager does not retain or drive a widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardDisplayMode {
    Browser,
    MinimalUi,
    Standalone,
    Fullscreen,
    Unframed,
    WindowControlsOverlay,
    Other,
}

// Source: keyboard_event_manager.cc:437,443,467,535,544,607,625,630,640,651
// These ports call the real command/controller owners. No editor command is
// projected into a string or interpreted by the keyboard manager.
pub trait KeyboardEditor {
    fn HandleKeyboardEvent(&self, event: &KeyboardEventRef);
}
pub trait KeyboardSpatialNavigation {
    fn HandleArrowKeyboardEvent(&self, event: &KeyboardEventRef) -> bool;
    fn HandleEscapeKeyboardEvent(&self, event: &KeyboardEventRef);
    fn HandleEnterKeyboardEvent(&self, event: &KeyboardEventRef);
    fn HandleImeSubmitKeyboardEvent(&self, event: &KeyboardEventRef);
}
pub trait KeyboardPage<'scope> {
    fn IsFocusedOrMainFrame(&self, frame: &dyn KeyboardFrame<'scope>) -> bool;
    fn TabKeyCyclesThroughElements(&self) -> bool;
    fn AdvanceFocus(&self, focus_type: FocusType, capabilities: InputDeviceCapabilitiesRef)
        -> bool;
    fn AdvanceFocusWithScope(
        &self,
        focus_type: FocusType,
        capabilities: InputDeviceCapabilitiesRef,
        scope: &FocusDispatchScope<'_>,
    ) -> bool {
        let _ = scope;
        self.AdvanceFocus(focus_type, capabilities)
    }
    fn SpatialNavigationController(&self) -> &dyn KeyboardSpatialNavigation;
}
// Source: keyboard_event_manager.cc:258-259,269,367-369,437,443,540,630
// Frame has no rendering access here. Its getters return only the original
// owners used by this source file, so fresh GetDocument/GetPage calls after
// script observe navigation/detachment rather than a pre-dispatch snapshot.
pub trait KeyboardFrame<'scope> {
    fn GetDocument(&self) -> Rc<dyn KeyboardDocument<'scope> + 'scope>;
    fn GetPage(&self) -> Option<Rc<dyn KeyboardPage<'scope> + 'scope>>;
    fn GetEditor(&self) -> &dyn KeyboardEditor;
    fn NotifyInteractionUserActivation(&self);
    fn DontSendKeyEventsToJavascript(&self) -> bool;
    fn DisplayMode(&self) -> KeyboardDisplayMode;
    fn IsSpatialNavigationEnabled(&self) -> bool;
    fn HandleFocusgroupKeyboardEvent(&self, event: &KeyboardEventRef) -> bool;
    fn CloseWatcherEscapeKeyHandler(&self, event: &KeyboardEventRef);
    fn MacDisableCtrlHomeEndEnabled(&self) -> bool;
    fn MiddleClickAutoscrollEnabled(&self) -> bool;
    fn ScrollEffects(&self) -> ScrollEffectEmitter;
}

// Source: keyboard_event_manager.cc:204,228 (base::AutoReset).
pub struct HandlingKeyEventScope<'a> {
    flag: &'a Cell<bool>,
    previous: bool,
}
impl<'a> HandlingKeyEventScope<'a> {
    fn new(flag: &'a Cell<bool>) -> Self {
        Self {
            flag,
            previous: flag.replace(true),
        }
    }
}
impl Drop for HandlingKeyEventScope<'_> {
    fn drop(&mut self) {
        self.flag.set(self.previous);
    }
}

// Source: keyboard_event_manager.h:28-86
pub struct KeyboardEventManager<'scope> {
    frame_: Rc<dyn KeyboardFrame<'scope> + 'scope>,
    scroll_manager_: Rc<ScrollManager>,
    keycodes_: Rc<dyn KeyboardManagerKeycodeConverter>,
    strings_: Rc<dyn KeyboardStringOperations>,
    state: Rc<KeyboardEventManagerState>,
}

// Source: keyboard_event_manager.h:75-86. The facade's synchronous input
// scope and full translated manager retain the same state owner.
pub struct KeyboardEventManagerState {
    scrollend_event_target_: RefCell<Option<Weak<dyn ScrollableArea>>>,
    is_handling_key_event_: Cell<bool>,
    last_scrolling_keycode_: Cell<i32>,
    has_pending_scrollend_on_key_up_: Cell<bool>,
}
impl Default for KeyboardEventManagerState {
    fn default() -> Self {
        Self {
            scrollend_event_target_: RefCell::new(None),
            is_handling_key_event_: Cell::new(false),
            last_scrolling_keycode_: Cell::new(VKEY_UNKNOWN),
            has_pending_scrollend_on_key_up_: Cell::new(false),
        }
    }
}
impl KeyboardEventManagerState {
    // Source: keyboard_event_manager.cc:204,228. RAII preserves nested input.
    pub fn EnterKeyEvent(&self) -> HandlingKeyEventScope<'_> {
        HandlingKeyEventScope::new(&self.is_handling_key_event_)
    }
    pub fn is_handling_key_event(&self) -> bool {
        self.is_handling_key_event_.get()
    }
}
impl std::ops::Deref for KeyboardEventManager<'_> {
    type Target = KeyboardEventManagerState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'scope> KeyboardEventManager<'scope> {
    // Source: keyboard_event_manager.h:31-37
    #[cfg(target_os = "macos")]
    pub const kAccessKeyModifiers: i32 = WebInputEvent::kControlKey | WebInputEvent::kAltKey;
    #[cfg(not(target_os = "macos"))]
    pub const kAccessKeyModifiers: i32 = WebInputEvent::kAltKey;

    // Source: keyboard_event_manager.h:39-42,75-86; .cc:193-201
    // Rc replaces Blink's fixed frame/scroll/service Members; Weak retains the
    // source scroll-area lifetime. Rust destruction replaces Trace, not a
    // second reference graph. Copy/Clone remain unavailable.
    pub fn new(
        frame: Rc<dyn KeyboardFrame<'scope> + 'scope>,
        scroll_manager: Rc<ScrollManager>,
        keycodes: Rc<dyn KeyboardManagerKeycodeConverter>,
        strings: Rc<dyn KeyboardStringOperations>,
    ) -> Self {
        Self::WithState(
            frame,
            scroll_manager,
            keycodes,
            strings,
            Rc::new(KeyboardEventManagerState::default()),
        )
    }
    pub fn WithState(
        frame: Rc<dyn KeyboardFrame<'scope> + 'scope>,
        scroll_manager: Rc<ScrollManager>,
        keycodes: Rc<dyn KeyboardManagerKeycodeConverter>,
        strings: Rc<dyn KeyboardStringOperations>,
        state: Rc<KeyboardEventManagerState>,
    ) -> Self {
        Self {
            frame_: frame,
            scroll_manager_: scroll_manager,
            keycodes_: keycodes,
            strings_: strings,
            state,
        }
    }

    // Source: keyboard_event_manager.cc:203-224
    pub fn HandleAccessKey(&self, evt: &WebKeyboardEvent) -> bool {
        let _handling = self.state.EnterKeyEvent();
        debug_assert_eq!(Self::kAccessKeyModifiers & WebInputEvent::kShiftKey, 0);
        if evt.GetModifiers() & (WebInputEvent::kKeyModifiers & !WebInputEvent::kShiftKey)
            != Self::kAccessKeyModifiers
        {
            return false;
        }
        let length = evt
            .unmodified_text
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(evt.unmodified_text.len());
        let key = self
            .strings_
            .DeprecatedLower(&evt.unmodified_text[..length]);
        let Some(elem) = self.frame_.GetDocument().GetElementByAccessKey(&key) else {
            return false;
        };
        elem.Focus(&FocusParams::new(
            SelectionBehaviorOnFocus::kReset,
            FocusType::kAccessKey,
            None,
            Rc::new(FocusOptions::default()),
            FocusTrigger::kUserGesture,
        ));
        elem.AccessKeyAction(SimulatedClickCreationScope::kFromUserAgent);
        true
    }

    // Source: keyboard_event_manager.cc:226-417
    pub fn KeyEvent(&self, initial_key_event: &WebKeyboardEvent) -> WebInputEventResult {
        let _handling = self.state.EnterKeyEvent();
        if initial_key_event.windows_key_code == VKEY_CAPITAL {
            self.CapsLockStateMayHaveChanged();
        }
        // Source: keyboard_event_manager.cc:232-242
        if self.scroll_manager_.MiddleClickAutoscrollInProgress() {
            debug_assert!(self.frame_.MiddleClickAutoscrollEnabled());
            if matches!(
                initial_key_event.GetType(),
                Type::kKeyDown | Type::kRawKeyDown
            ) {
                self.scroll_manager_.StopMiddleClickAutoscroll();
            }
            return WebInputEventResult::kHandledSuppressed;
        }
        // Source: keyboard_event_manager.cc:244-260
        let Some(mut node) = self.frame_.GetDocument().EventTargetNodeForDocument() else {
            return WebInputEventResult::kNotHandled;
        };
        if !self
            .keycodes_
            .IsDomKeyForModifier(initial_key_event.dom_key)
            && !self.keycodes_.IsEscapeDomKey(initial_key_event.dom_key)
            && matches!(
                initial_key_event.GetType(),
                Type::kKeyDown | Type::kRawKeyDown
            )
        {
            self.frame_.NotifyInteractionUserActivation();
        }
        // Source: keyboard_event_manager.cc:262-314
        let mut should_send_key_events_to_js = !self.frame_.DontSendKeyEventsToJavascript();
        if !should_send_key_events_to_js && self.frame_.GetDocument().IsInWebAppScope() {
            should_send_key_events_to_js = matches!(
                self.frame_.DisplayMode(),
                KeyboardDisplayMode::MinimalUi
                    | KeyboardDisplayMode::Standalone
                    | KeyboardDisplayMode::Fullscreen
                    | KeyboardDisplayMode::Unframed
                    | KeyboardDisplayMode::WindowControlsOverlay
            );
        }
        let mut send_key_event = true;
        let mut event_cancellable = true;
        if !should_send_key_events_to_js {
            for dom_key in [0x00200309, 0x00200310] {
                if initial_key_event.dom_key == dom_key {
                    send_key_event = false;
                }
            }
            for dom_key in [0x00400031, 0x00400032, 0x00400033] {
                let is_editable = node.IsEditable()
                    || node
                        .TextControlIsDisabledOrReadOnly()
                        .is_some_and(|value| !value)
                    || node
                        .RoleAttribute()
                        .is_some_and(|role| role.eq_ignore_ascii_case("textbox"));
                if initial_key_event.dom_key == dom_key && !is_editable {
                    event_cancellable = false;
                }
            }
        } else {
            send_key_event = initial_key_event.dom_key != 0x00200309;
        }

        // Source: keyboard_event_manager.cc:316-415. Event creation borrows the
        // current document only at each source construction point. Dispatch is
        // synchronous; subsequent focus/frame queries happen after it returns.
        let dispatch_result = match initial_key_event.GetType() {
            Type::kKeyUp => {
                let event = self.CreateKeyboardEvent(initial_key_event, event_cancellable);
                let target = node.EventTarget();
                {
                    let mut event = event.borrow_mut();
                    event.SetTarget(Some(target));
                    event.SetStopPropagation(!send_key_event);
                }
                node.DispatchEvent(&event)
            }
            Type::kRawKeyDown | Type::kKeyDown => {
                let mut web_event = initial_key_event.clone();
                web_event.SetType(Type::kRawKeyDown);
                let event = self.CreateKeyboardEvent(&web_event, event_cancellable);
                let target = node.EventTarget();
                {
                    let mut event = event.borrow_mut();
                    event.SetTarget(Some(target));
                    event.SetStopPropagation(!send_key_event);
                }
                // Source: keyboard_event_manager.cc:340-352
                if initial_key_event.GetType() == Type::kKeyDown
                    && self.HandleAccessKey(initial_key_event)
                {
                    event.borrow_mut().preventDefault();
                }
                // Source: keyboard_event_manager.cc:354-363
                let had_keyboard_event = {
                    let event = event.borrow();
                    !event.ctrlKey() && !event.altKey() && !event.metaKey()
                };
                if had_keyboard_event {
                    node.UpdateHadKeyboardEvent(&event);
                }
                let result = node.DispatchEvent(&event);
                if result != DispatchEventResult::kNotCanceled {
                    return ToWebInputEventResult(result);
                }
                // Source: keyboard_event_manager.cc:365-384
                if self
                    .frame_
                    .GetPage()
                    .is_some_and(|page| !page.IsFocusedOrMainFrame(self.frame_.as_ref()))
                {
                    return WebInputEventResult::kHandledSystem;
                }
                if initial_key_event.GetType() != Type::kKeyDown {
                    return WebInputEventResult::kNotHandled;
                }
                node = match self.frame_.GetDocument().EventTargetNodeForDocument() {
                    Some(node) => node,
                    None => return WebInputEventResult::kNotHandled,
                };
                // Source: keyboard_event_manager.cc:386-399
                #[cfg(target_os = "macos")]
                if (0xf700..=0xf7ff).contains(&initial_key_event.text[0]) {
                    return WebInputEventResult::kNotHandled;
                }
                if initial_key_event.text[0] == 0 {
                    return WebInputEventResult::kNotHandled;
                }
                self.DispatchCharEvent(initial_key_event, &node, event_cancellable, send_key_event)
            }
            Type::kChar => {
                self.DispatchCharEvent(initial_key_event, &node, event_cancellable, send_key_event)
            }
            _ => unreachable!("KeyEvent requires a keyboard input event"),
        };
        ToWebInputEventResult(dispatch_result)
    }

    // Source: keyboard_event_manager.cc:321-323,335-336,405-406.
    // Only the identical source constructor expression is factored out.
    fn CreateKeyboardEvent(&self, key: &WebKeyboardEvent, cancellable: bool) -> KeyboardEventRef {
        let document = self.frame_.GetDocument();
        Rc::new(RefCell::new(KeyboardEvent::CreateFromWebEvent(
            key,
            Some(document.DomWindow()),
            cancellable,
            self.keycodes_.EventKeycodeConverter(),
        )))
    }

    // Source: keyboard_event_manager.cc:401-411 (shared C++ fallthrough block).
    fn DispatchCharEvent(
        &self,
        initial: &WebKeyboardEvent,
        node: &Rc<dyn KeyboardNode + 'scope>,
        cancellable: bool,
        send_key_event: bool,
    ) -> DispatchEventResult {
        let mut char_event = initial.clone();
        char_event.SetType(Type::kChar);
        let event = self.CreateKeyboardEvent(&char_event, cancellable);
        let target = node.EventTarget();
        {
            let mut event = event.borrow_mut();
            event.SetTarget(Some(target));
            event.SetStopPropagation(!send_key_event);
        }
        node.DispatchEvent(&event)
    }

    // Source: keyboard_event_manager.cc:419-424. The Document port applies the
    // source FocusedElement + DynamicTo<HTMLInputElement> guard on its owner.
    pub fn CapsLockStateMayHaveChanged(&self) {
        self.frame_
            .GetDocument()
            .FocusedInputCapsLockStateMayHaveChanged();
    }

    // Source: keyboard_event_manager.cc:426-490
    pub fn DefaultKeyboardEventHandler(
        &self,
        event: &KeyboardEventRef,
        possible_focused_node: Option<&dyn KeyboardNode>,
    ) {
        self.DefaultKeyboardEventHandlerWithFocusScope(
            event,
            possible_focused_node,
            &FocusDispatchScope::default(),
        )
    }

    /// Rust carries the call-scoped listener dispatcher which C++ reaches
    /// through LocalFrame. KeyboardEventManager borrows it only for this
    /// synchronous default-action stack.
    pub fn DefaultKeyboardEventHandlerWithFocusScope(
        &self,
        event: &KeyboardEventRef,
        possible_focused_node: Option<&dyn KeyboardNode>,
        focus_scope: &FocusDispatchScope<'_>,
    ) {
        if event.borrow().r#type() == "keydown" {
            let is_process_key = event.borrow().keyCode() == kVKeyProcessKey;
            if !is_process_key && matches!(event.borrow().key(), "Home" | "End") {
                if self.frame_.HandleFocusgroupKeyboardEvent(event) {
                    event.borrow_mut().SetDefaultHandled();
                    return;
                }
            }
            self.frame_.GetEditor().HandleKeyboardEvent(event);
            if event.borrow().DefaultHandled() {
                return;
            }
            if is_process_key {
                return;
            }
            enum KeyAction {
                Tab,
                Escape,
                Enter,
                ImeSubmit,
                Navigation,
            }
            let action = {
                let event = event.borrow();
                match event.key() {
                    "Tab" => KeyAction::Tab,
                    "Escape" => KeyAction::Escape,
                    "Enter" => KeyAction::Enter,
                    _ if event
                        .KeyEvent()
                        .is_some_and(|key| key.dom_key == 0x00200310) =>
                    {
                        KeyAction::ImeSubmit
                    }
                    _ => KeyAction::Navigation,
                }
            };
            match action {
                KeyAction::Tab => {
                    self.DefaultTabEventHandlerWithFocusScope(event, focus_scope);
                }
                KeyAction::Escape => self.DefaultEscapeEventHandler(event),
                KeyAction::Enter => self.DefaultEnterEventHandler(event),
                KeyAction::ImeSubmit => self.DefaultImeSubmitHandler(event),
                KeyAction::Navigation => {
                    self.DefaultNavigationKeyEventHandler(event, possible_focused_node)
                }
            }
        } else if event.borrow().r#type() == "keypress" {
            self.frame_.GetEditor().HandleKeyboardEvent(event);
            if event.borrow().DefaultHandled() {
                return;
            }
            if event.borrow().key() == "Enter" {
                self.DefaultEnterEventHandler(event);
            } else if event.borrow().charCode() == ' ' as i32 {
                self.DefaultSpaceEventHandler(event, possible_focused_node);
            }
        } else if event.borrow().r#type() == "keyup" {
            if event.borrow().DefaultHandled() {
                return;
            }
            if event.borrow().key() == "Enter" {
                self.DefaultEnterEventHandler(event);
            }
            if event.borrow().keyCode() == self.last_scrolling_keycode_.get() {
                let area = self
                    .scrollend_event_target_
                    .borrow()
                    .as_ref()
                    .and_then(Weak::upgrade);
                if let Some(area) = area {
                    if self.has_pending_scrollend_on_key_up_.get() {
                        area.OnScrollFinished(true, &self.frame_.ScrollEffects());
                    }
                }
                self.scrollend_event_target_.borrow_mut().take();
                self.last_scrolling_keycode_.set(VKEY_UNKNOWN);
                self.has_pending_scrollend_on_key_up_.set(false);
            }
        }
    }

    // Source: keyboard_event_manager.cc:492-524
    fn DefaultSpaceEventHandler(
        &self,
        event: &KeyboardEventRef,
        possible_focused_node: Option<&dyn KeyboardNode>,
    ) {
        debug_assert_eq!(event.borrow().r#type(), "keypress");
        if event.borrow().ctrlKey() || event.borrow().metaKey() || event.borrow().altKey() {
            return;
        }
        let direction = if event.borrow().shiftKey() {
            ScrollDirection::kScrollBlockDirectionBackward
        } else {
            ScrollDirection::kScrollBlockDirectionForward
        };
        self.scrollend_event_target_.borrow_mut().take();
        if self.scroll_manager_.LogicalScroll(
            direction,
            ScrollGranularity::kScrollByPage,
            None,
            possible_focused_node.map(KeyboardNode::ScrollNode),
            true,
        ) == LogicalScrollResult::kScrolled
        {
            // TODO(instrumentation): UseCounter(kScrollByKeyboardSpacebarKey).
            self.last_scrolling_keycode_.set(event.borrow().keyCode());
            self.has_pending_scrollend_on_key_up_.set(true);
            event.borrow_mut().SetDefaultHandled();
        }
    }

    // Source: keyboard_event_manager.cc:526-572
    fn DefaultNavigationKeyEventHandler(
        &self,
        event: &KeyboardEventRef,
        possible_focused_node: Option<&dyn KeyboardNode>,
    ) {
        debug_assert_eq!(event.borrow().r#type(), "keydown");
        let Some(page) = self.frame_.GetPage() else {
            return;
        };
        if self.frame_.HandleFocusgroupKeyboardEvent(event) {
            event.borrow_mut().SetDefaultHandled();
            return;
        }
        if self.frame_.IsSpatialNavigationEnabled()
            && !self.frame_.GetDocument().InDesignMode()
            && !IsPageUpOrDownKeyEvent(event.borrow().keyCode(), event.borrow().GetModifiers())
        {
            if page
                .SpatialNavigationController()
                .HandleArrowKeyboardEvent(event)
            {
                event.borrow_mut().SetDefaultHandled();
                return;
            }
        }
        if event
            .borrow()
            .KeyEvent()
            .is_some_and(|key| key.is_system_key)
        {
            return;
        }
        let (key_code, modifiers) = {
            let event = event.borrow();
            (event.keyCode(), event.GetModifiers())
        };
        let disable_ctrl_home_end = self.frame_.MacDisableCtrlHomeEndEnabled();
        let Some((direction, granularity)) =
            MapKeyCodeForScroll(key_code, modifiers, disable_ctrl_home_end)
        else {
            return;
        };
        self.scrollend_event_target_.borrow_mut().take();
        if self.scroll_manager_.BubblingScroll(
            direction,
            granularity,
            None,
            possible_focused_node.map(KeyboardNode::ScrollNode),
            true,
        ) {
            // TODO(instrumentation): UseCounter(scroll_use_uma).
            self.last_scrolling_keycode_.set(event.borrow().keyCode());
            self.has_pending_scrollend_on_key_up_.set(true);
            event.borrow_mut().SetDefaultHandled();
        }
    }

    // Source: keyboard_event_manager.cc:574-616
    pub fn DefaultTabEventHandler(&self, event: &KeyboardEventRef) -> bool {
        self.DefaultTabEventHandlerWithFocusScope(event, &FocusDispatchScope::default())
    }

    pub fn DefaultTabEventHandlerWithFocusScope(
        &self,
        event: &KeyboardEventRef,
        focus_scope: &FocusDispatchScope<'_>,
    ) -> bool {
        debug_assert_eq!(event.borrow().r#type(), "keydown");
        if event.borrow().ctrlKey() || event.borrow().metaKey() {
            return false;
        }
        #[cfg(not(target_os = "macos"))]
        if event.borrow().altKey() {
            return false;
        }
        let Some(page) = self.frame_.GetPage() else {
            return false;
        };
        if !page.TabKeyCyclesThroughElements() {
            return false;
        }
        let focus_type = if event.borrow().shiftKey() {
            FocusType::kBackward
        } else {
            FocusType::kForward
        };
        if self.frame_.GetDocument().InDesignMode() {
            return false;
        }
        let capabilities = self
            .frame_
            .GetDocument()
            .DomWindow()
            .FiresTouchEvents(false);
        if page.AdvanceFocusWithScope(focus_type, capabilities, focus_scope) {
            event.borrow_mut().SetDefaultHandled();
            return true;
        }
        false
    }

    // Source: keyboard_event_manager.cc:618-631
    fn DefaultEscapeEventHandler(&self, event: &KeyboardEventRef) {
        let Some(page) = self.frame_.GetPage() else {
            return;
        };
        let document = self.frame_.GetDocument();
        if self.frame_.IsSpatialNavigationEnabled() && !document.InDesignMode() {
            page.SpatialNavigationController()
                .HandleEscapeKeyboardEvent(event);
        }
        document.LoseInterestInAllElements();
        self.frame_.CloseWatcherEscapeKeyHandler(event);
    }

    // Source: keyboard_event_manager.cc:633-642
    fn DefaultEnterEventHandler(&self, event: &KeyboardEventRef) {
        let Some(page) = self.frame_.GetPage() else {
            return;
        };
        if self.frame_.IsSpatialNavigationEnabled() && !self.frame_.GetDocument().InDesignMode() {
            page.SpatialNavigationController()
                .HandleEnterKeyboardEvent(event);
        }
    }

    // Source: keyboard_event_manager.cc:644-653
    fn DefaultImeSubmitHandler(&self, event: &KeyboardEventRef) {
        let Some(page) = self.frame_.GetPage() else {
            return;
        };
        if self.frame_.IsSpatialNavigationEnabled() && !self.frame_.GetDocument().InDesignMode() {
            page.SpatialNavigationController()
                .HandleImeSubmitKeyboardEvent(event);
        }
    }

    // Source: keyboard_event_manager.cc:657-660; .h:65-67 (Internals hook).
    pub(crate) fn SetCurrentCapsLockState(state: OverrideCapsLockState) {
        g_override_caps_lock_state.store(state as u8, Ordering::Relaxed);
    }

    // Source: keyboard_event_manager.cc:662-678
    pub fn CurrentCapsLockState(
        platform: &dyn KeyboardPlatformModifiers,
    ) -> Result<bool, &'static str> {
        match g_override_caps_lock_state.load(Ordering::Relaxed) {
            value if value == OverrideCapsLockState::kDefault as u8 => {
                #[cfg(target_os = "macos")]
                {
                    Ok(platform.GetCurrentKeyModifiers()?.alpha_lock)
                }
                #[cfg(not(target_os = "macos"))]
                {
                    let _ = platform;
                    Ok(false)
                }
            }
            value if value == OverrideCapsLockState::kOn as u8 => Ok(true),
            _ => Ok(false),
        }
    }

    // Source: keyboard_event_manager.cc:680-697
    pub fn GetCurrentModifierState(
        platform: &dyn KeyboardPlatformModifiers,
    ) -> Result<i32, &'static str> {
        #[cfg(target_os = "macos")]
        {
            let current = platform.GetCurrentKeyModifiers()?;
            let mut modifiers = 0;
            if current.shift {
                modifiers |= WebInputEvent::kShiftKey;
            }
            if current.control {
                modifiers |= WebInputEvent::kControlKey;
            }
            if current.option {
                modifiers |= WebInputEvent::kAltKey;
            }
            if current.command {
                modifiers |= WebInputEvent::kMetaKey;
            }
            Ok(modifiers)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = platform;
            Ok(0)
        }
    }

    // Source: keyboard_event_manager.h:51-58
    pub fn SetScrollendEventTarget(&self, scrollable_area: &ScrollableAreaRef) {
        *self.scrollend_event_target_.borrow_mut() = Some(Rc::downgrade(scrollable_area));
    }
    pub fn HasPendingScrollendOnKeyUp(&self) -> bool {
        self.has_pending_scrollend_on_key_up_.get()
    }
    pub fn is_handling_key_event(&self) -> bool {
        self.is_handling_key_event_.get()
    }
}

// Source: keyboard_event_manager.cc:655. This remains one process-wide test
// override, with the source zero-initialized default.
static g_override_caps_lock_state: AtomicU8 = AtomicU8::new(OverrideCapsLockState::kDefault as u8);

// Source: keyboard_event_manager.cc:665-666,681-692. The host converts Carbon's
// native flag values without letting a platform API or OS dependency enter
// Interaction. The source non-Mac zero/false branches are retained above.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeKeyboardModifiers {
    pub alpha_lock: bool,
    pub shift: bool,
    pub control: bool,
    pub option: bool,
    pub command: bool,
}
pub trait KeyboardPlatformModifiers {
    fn GetCurrentKeyModifiers(&self) -> Result<NativeKeyboardModifiers, &'static str>;
}

// Source: keyboard_event_manager.h:51-56; scroll_manager.cc:294-311.
// The ScrollManager callback reaches the same manager/weak area owner. No
// manager state is borrowed across the animation-completion callback.
impl ScrollKeyboardEventManager for KeyboardEventManager<'_> {
    fn HasPendingScrollendOnKeyUp(&self) -> bool {
        KeyboardEventManager::HasPendingScrollendOnKeyUp(self)
    }
    fn SetScrollendEventTarget(&self, area: &ScrollableAreaRef) {
        KeyboardEventManager::SetScrollendEventTarget(self, area);
    }
}

// Source: keyboard_event_manager.h:51-58; scroll_manager.cc:294-311.
// The Page adapter constructs the call-scoped KeyboardEventManager around this
// retained state owner. ScrollManager must observe that same owner even between
// key transactions; retaining a second manager merely to expose these two
// fields would split scrollend identity and lifetime.
impl ScrollKeyboardEventManager for KeyboardEventManagerState {
    fn HasPendingScrollendOnKeyUp(&self) -> bool {
        self.has_pending_scrollend_on_key_up_.get()
    }

    fn SetScrollendEventTarget(&self, area: &ScrollableAreaRef) {
        *self.scrollend_event_target_.borrow_mut() = Some(Rc::downgrade(area));
    }
}
