//! Chromium scrollbar_theme.h:116-126; scrollbar_theme_mac.cc:117-140;
//! scrollbar_theme_aura.cc:122-130,403-450; cc/input/scrollbar.h:26-27.
use foundation::{gfx::Point, scrollbar_theme_settings::ScrollbarThemeSettings};
use interaction_input::public::common::input::{
    web_input_event::WebInputEvent, web_mouse_event::WebMouseEvent, web_pointer_properties::Button,
};
#[derive(Clone, Copy)]
pub enum MouseTheme {
    Base,
    Mac,
    Aura,
}
pub fn ShouldCenterOnThumb(theme: MouseTheme, event: &WebMouseEvent) -> bool {
    let button = event.pointer_properties.button;
    match theme {
        MouseTheme::Base => false,
        MouseTheme::Mac => {
            button == Button::kLeft
                && ScrollbarThemeSettings::MacJumpOnTrackClick()
                    != (event.GetModifiers() & WebInputEvent::kAltKey != 0)
        }
        MouseTheme::Aura => {
            (cfg!(target_os = "linux") && button == Button::kMiddle)
                || (button == Button::kLeft && event.GetModifiers() & WebInputEvent::kShiftKey != 0)
        }
    }
}
pub fn ShouldDragDocumentInsteadOfThumb(theme: MouseTheme, event: &WebMouseEvent) -> bool {
    matches!(theme, MouseTheme::Mac) && event.GetModifiers() & WebInputEvent::kAltKey != 0
}
pub fn ShouldSnapBackToDragOrigin(
    theme: MouseTheme,
    horizontal: bool,
    thickness: i32,
    local: Point,
) -> bool {
    if !matches!(theme, MouseTheme::Aura) || cfg!(target_os = "linux") {
        return false;
    }
    let expansion = 8 * thickness.max(17);
    let offset = if horizontal { local.y() } else { local.x() };
    offset < -expansion || offset >= expansion + thickness
}
