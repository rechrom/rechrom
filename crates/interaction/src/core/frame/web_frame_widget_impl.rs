//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: core/frame/web_frame_widget_impl.h:1316-1324;
//! core/frame/web_frame_widget_impl.cc:1061-1150,1419-1470,5626-5629.
//! The retained suppression bit belongs to the widget, while frame selection,
//! popup forwarding, plugins and EventHandler remain their existing owners.
use interaction_dispatch::public::platform::web_input_event_result::WebInputEventResult;
use interaction_input::public::common::input::{
    web_input_event::{Type, WebInputEvent},
    web_keyboard_event::WebKeyboardEvent,
};
use std::cell::Cell;

#[derive(Default)]
pub struct WidgetKeyboardState {
    suppress_next_keypress_event: Cell<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusedPluginKeyboardState {
    NotEmbeddedObject,
    EmbeddedObject {
        supports_keyboard_focus: Option<bool>,
    },
}

pub trait WidgetKeyboardContext {
    type Frame;
    type Error;
    fn HandlePopupKeyEvent(
        &self,
        event: &WebKeyboardEvent,
    ) -> Result<Option<WebInputEventResult>, Self::Error>;
    fn PagePopupCopyPasteEnabled(&self) -> bool;
    fn FocusedCoreFrame(&self) -> Result<Option<Self::Frame>, Self::Error>;
    fn HasLocalRoot(&self) -> bool;
    fn KeyEvent(&self, frame: &Self::Frame, event: &WebKeyboardEvent) -> WebInputEventResult;
    fn HandleAccessKey(&self, frame: &Self::Frame, event: &WebKeyboardEvent) -> bool;
    fn FocusedPluginKeyboardState(&self) -> Result<FocusedPluginKeyboardState, Self::Error>;
    fn SendKeyboardContextMenu(&self) -> Result<(), Self::Error>;
}

impl WidgetKeyboardState {
    pub fn HandleKeyEvent<C: WidgetKeyboardContext>(
        &self,
        context: &C,
        event: &WebKeyboardEvent,
    ) -> Result<WebInputEventResult, C::Error> {
        debug_assert!(matches!(
            event.GetType(),
            Type::kRawKeyDown | Type::kKeyDown | Type::kKeyUp
        ));
        self.suppress_next_keypress_event.set(false);
        if let Some(result) = context.HandlePopupKeyEvent(event)? {
            if event.GetType() == Type::kRawKeyDown {
                self.suppress_next_keypress_event.set(true);
            }
            return Ok(if context.PagePopupCopyPasteEnabled() {
                result
            } else {
                WebInputEventResult::kHandledSystem
            });
        }
        let Some(frame) = context.FocusedCoreFrame()? else {
            return Ok(WebInputEventResult::kNotHandled);
        };
        let result = context.KeyEvent(&frame, event);
        if !context.HasLocalRoot() {
            return Ok(result);
        }
        if result != WebInputEventResult::kNotHandled {
            if event.GetType() == Type::kRawKeyDown {
                match context.FocusedPluginKeyboardState()? {
                    FocusedPluginKeyboardState::NotEmbeddedObject => {
                        self.suppress_next_keypress_event.set(true)
                    }
                    FocusedPluginKeyboardState::EmbeddedObject {
                        supports_keyboard_focus,
                    } => {
                        if event.windows_key_code == 0x09 && supports_keyboard_focus == Some(true) {
                            self.suppress_next_keypress_event.set(true);
                        }
                    }
                }
            }
            return Ok(result);
        }
        let menu_type = if cfg!(target_os = "windows") {
            Type::kKeyUp
        } else {
            Type::kRawKeyDown
        };
        let modifiers = event.GetModifiers() & WebInputEvent::kInputModifiers;
        let menu_key = modifiers == 0 && event.windows_key_code == 0x5d;
        let shift_f10 = !cfg!(target_os = "macos")
            && modifiers == WebInputEvent::kShiftKey
            && event.windows_key_code == 0x79;
        if (menu_key && event.GetType() == menu_type)
            || (shift_f10 && event.GetType() == Type::kRawKeyDown)
        {
            context.SendKeyboardContextMenu()?;
            return Ok(WebInputEventResult::kHandledSystem);
        }
        Ok(WebInputEventResult::kNotHandled)
    }

    pub fn HandleCharEvent<C: WidgetKeyboardContext>(
        &self,
        context: &C,
        event: &WebKeyboardEvent,
    ) -> Result<WebInputEventResult, C::Error> {
        debug_assert_eq!(event.GetType(), Type::kChar);
        let suppress = self.suppress_next_keypress_event.replace(false);
        if let Some(result) = context.HandlePopupKeyEvent(event)? {
            return Ok(result);
        }
        let Some(frame) = context.FocusedCoreFrame()? else {
            return Ok(if suppress {
                WebInputEventResult::kHandledSuppressed
            } else {
                WebInputEventResult::kNotHandled
            });
        };
        if !event.IsCharacterKey() {
            return Ok(WebInputEventResult::kHandledSuppressed);
        }
        if context.HandleAccessKey(&frame, event) {
            return Ok(WebInputEventResult::kHandledSystem);
        }
        if event.is_system_key {
            return Ok(WebInputEventResult::kNotHandled);
        }
        if suppress {
            return Ok(WebInputEventResult::kHandledSuppressed);
        }
        Ok(context.KeyEvent(&frame, event))
    }
}
