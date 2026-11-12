//! Retained embedding state for EventHandler's ChromeClient notifications.
//! The emitting document and frame keep ownership of dispatch and hit testing;
//! a host retains only the values delivered at those source call sites.
use super::EventHandlerChromeEffect;

#[derive(Clone, Debug, PartialEq)]
pub struct PageChromeMouseOver {
    pub frame: u64,
    pub node: Option<u64>,
    pub title: Option<String>,
    pub point: (f32, f32),
    pub modifiers: i32,
}

/// A bounded, observable ChromeClient projection for embeddings without a
/// platform tooltip or input observer. Keep one instance per document.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PageChromeState {
    pub sequence: u64,
    pub last_effect: Option<EventHandlerChromeEffect>,
    pub will_dispatch_pointer_down: Option<(u64, u64)>,
    pub did_dispatch_mouse_down: Option<(u64, u64)>,
    pub mouse_over: Option<PageChromeMouseOver>,
    pub status_url: Option<String>,
    pub tooltip: Option<String>,
    pub tooltip_direction: Option<foundation::TextDirection>,
    pub tooltip_clear_sequence: Option<u64>,
}

impl PageChromeState {
    pub fn Apply(&mut self, effect: EventHandlerChromeEffect) {
        self.sequence += 1;
        match &effect {
            EventHandlerChromeEffect::WillDispatchPointerDown { frame, node } => {
                self.will_dispatch_pointer_down = Some((*frame, *node));
            }
            EventHandlerChromeEffect::DidDispatchMouseDown { frame, node } => {
                self.did_dispatch_mouse_down = Some((*frame, *node));
            }
            EventHandlerChromeEffect::MouseDidMoveOverElement {
                frame,
                node,
                title,
                point,
                modifiers,
            } => {
                self.mouse_over = Some(PageChromeMouseOver {
                    frame: *frame,
                    node: *node,
                    title: title.clone(),
                    point: *point,
                    modifiers: *modifiers,
                });
            }
            EventHandlerChromeEffect::ShowMouseOverUrl { url, .. } => {
                self.status_url = url.clone();
            }
            EventHandlerChromeEffect::UpdateToolTip {
                text, direction, ..
            } => {
                self.tooltip = text.clone();
                self.tooltip_direction = Some(*direction);
            }
            EventHandlerChromeEffect::ClearToolTip { .. } => {
                self.tooltip = None;
                self.tooltip_clear_sequence = Some(self.sequence);
            }
        }
        self.last_effect = Some(effect);
    }
}
