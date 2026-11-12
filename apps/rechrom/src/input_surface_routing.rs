//! Window placement routing for typed input. Pages remain independent domains;
//! retained targets are document identities, never Page/DOM/rendering owners.
use crate::page_presentation::Document;
use interaction::input::public::common::input::{
    web_gesture_event::{GestureDevice, WebGestureEvent},
    web_input_event::Type,
    web_mouse_wheel_event::Phase,
};
use interaction::input_event::{
    KeyEvent, KeyEventType, MouseEvent, MouseEventType, Offset, PointerEvent, PointerEventType,
    TouchEvent, TouchEventType, TouchPointState, WheelEvent,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SurfaceRoute {
    pub document: Document,
    pub origin: Offset,
}

#[derive(Default)]
pub(crate) struct InputSurfaceRouting {
    chrome: HashMap<Document, browser::page::PageChromeState>,
    autoscroll: HashMap<Document, AutoscrollHostState>,
    pointer_contacts: HashMap<(u8, u32), SurfaceRoute>,
    pointer_hover: HashMap<(u8, u32), SurfaceRoute>,
    touch_target: Option<SurfaceRoute>,
    active_touches: usize,
    touchscreen_targets: HashMap<u32, SurfaceRoute>,
    touchscreen_gesture_target: Option<SurfaceRoute>,
    touchpad_gesture_target: Option<SurfaceRoute>,
    last_fling_target: Option<SurfaceRoute>,
    wheel_target: Option<SurfaceRoute>,
    widget_mouse_capture: Option<SurfaceRoute>,
    last_mouse_down_target: Option<SurfaceRoute>,
    mouse_buttons: u16,
    keyboard_origin: Option<(Document, std::time::Instant)>,
}
#[derive(Default)]
struct AutoscrollHostState {
    selection_active: bool,
    ended: bool,
    middle_click:
        interaction::input::components::input::middle_click_autoscroll::MiddleClickAutoscroll,
}

impl InputSurfaceRouting {
    /// RawKeyDown and its native Char belong to the same originating widget,
    /// even if that key changes embedding focus. Retain only identity/time;
    /// the Page's WidgetKeyboardState owns all handled/keypress policy.
    pub fn Keyboard(&mut self, event: &KeyEvent, current: Option<Document>) -> Option<Document> {
        match event.r#type {
            KeyEventType::kDown => {
                self.keyboard_origin = current.zip(event.timestamp);
                current
            }
            KeyEventType::kUp => {
                self.keyboard_origin = None;
                current
            }
            KeyEventType::kChar => {
                let origin = self.keyboard_origin.take();
                match origin {
                    Some((document, timestamp)) if event.timestamp == Some(timestamp) => {
                        Some(document)
                    }
                    _ if event.is_standalone_char => current,
                    _ => None,
                }
            }
        }
    }
    /// Shared native/headless ChromeClient consumer. The Document generation
    /// prevents a replaced Page from updating its successor's host state.
    pub fn EventHandlerChrome(
        &mut self,
        document: Document,
        effect: browser::page::EventHandlerChromeEffect,
    ) {
        self.chrome.entry(document).or_default().Apply(effect);
    }

    pub fn ChromeState(&self, document: Document) -> Option<&browser::page::PageChromeState> {
        self.chrome.get(&document)
    }

    // Source components/input/render_widget_host_input_event_router.cc and
    // SetMouseCaptureTargetValidation: capture is accepted only from the most
    // recent mouse-down target, and mouse-up synchronously clears both owners.
    pub fn MouseDownTarget(&mut self, route: SurfaceRoute, buttons: u16) {
        self.last_mouse_down_target = Some(route);
        self.mouse_buttons = buttons;
    }
    pub fn CaptureMouseEventsToWidget(&mut self, route: SurfaceRoute, capture: bool) {
        if capture {
            if self.mouse_buttons != 0
                && self
                    .last_mouse_down_target
                    .is_some_and(|target| target.document == route.document)
            {
                self.widget_mouse_capture = Some(route);
            }
        } else if self
            .widget_mouse_capture
            .is_some_and(|current| current.document == route.document)
        {
            self.widget_mouse_capture = None;
        }
    }
    pub fn CapturedMouseTarget(&mut self, event: &MouseEvent) -> Option<SurfaceRoute> {
        if event.r#type == MouseEventType::kUp {
            self.mouse_buttons = event.buttons;
            self.last_mouse_down_target = None;
            return self.widget_mouse_capture.take();
        }
        if event.r#type == MouseEventType::kMove && event.buttons == 0 {
            // Blink treats a no-button move as recovery from a lost native up.
            self.mouse_buttons = 0;
            self.last_mouse_down_target = None;
            self.widget_mouse_capture = None;
            return None;
        }
        if event.r#type == MouseEventType::kDown && self.mouse_buttons == 0 {
            // A fresh native press starts with hit testing. An asynchronous
            // capture effect from the completed sequence must not turn this
            // press into a continuation of the previous widget's sequence.
            self.last_mouse_down_target = None;
            self.widget_mouse_capture = None;
            return None;
        }
        if event.buttons != 0 {
            self.mouse_buttons = event.buttons;
        }
        (event.buttons != 0)
            .then_some(self.widget_mouse_capture)
            .flatten()
    }
    pub fn ReleaseMouseCapture(&mut self) {
        self.widget_mouse_capture = None;
    }
    // Source render_widget_host_input_event_router.cc:769-828,1836-1856.
    pub fn Wheel(&mut self, event: &WheelEvent, hit: Option<SurfaceRoute>) -> Option<SurfaceRoute> {
        // MouseWheelPhaseHandler has already normalized phase-less native
        // samples into the canonical Blink phase carried by `event.phase`.
        // Routing from the raw Cocoa phase loses that synthesized Began (the
        // raw value remains zero), so no surface is ever latched for ordinary
        // mouse wheels. Chromium's router consumes the post-handler event.
        let phase = event.phase.ToWebPhase().0;
        let momentum = event
            .native
            .map_or(0, |native| native.momentum_phase as u32);
        if phase == Phase::kPhaseBegan.0 || phase == Phase::kPhaseMayBegin.0 {
            self.wheel_target = hit;
        }
        let target = self.wheel_target;
        if phase == Phase::kPhaseEnded.0 || momentum == Phase::kPhaseEnded.0 {
            self.wheel_target = None;
        }
        target
    }
    // WidgetInputHandlerHost::SetAutoscrollSelectionActiveInMainFrame and
    // ChromeClient::AutoscrollEnd share this document-generation routing owner.
    pub fn Autoscroll(
        &mut self,
        document: Document,
        effect: interaction::adapter::page_mouse_autoscroll::AutoscrollHostEffect,
    ) -> Vec<WebGestureEvent> {
        use interaction::adapter::page_mouse_autoscroll::AutoscrollHostEffect;
        let state = self.autoscroll.entry(document).or_default();
        let now = interaction::adapter::host_event_clock::HostEventClock::TimeForInstant(
            std::time::Instant::now(),
        );
        match effect {
            AutoscrollHostEffect::AutoscrollSelectionActive(active) => {
                state.selection_active = active;
                Vec::new()
            }
            AutoscrollHostEffect::AutoscrollStart { position } => {
                state.middle_click.Start(position);
                state.ended = false;
                Vec::new()
            }
            AutoscrollHostEffect::AutoscrollFling { velocity } => {
                state.middle_click.Fling(velocity, now)
            }
            AutoscrollHostEffect::AutoscrollEnd => {
                state.ended = true;
                state.middle_click.End(now)
            }
        }
    }
    pub fn Pointer(
        &mut self,
        event: &PointerEvent,
        hit: Option<SurfaceRoute>,
    ) -> Option<SurfaceRoute> {
        let key = (event.pointer_type as u8, event.pointer_id);
        let route = self.pointer_contacts.get(&key).copied().or_else(|| {
            if event.r#type == PointerEventType::kLeave {
                self.pointer_hover.get(&key).copied()
            } else {
                hit
            }
        });
        if let Some(route) = route {
            if event.r#type == PointerEventType::kDown {
                self.pointer_contacts.insert(key, route);
            }
            if event.r#type != PointerEventType::kLeave {
                self.pointer_hover.insert(key, route);
            }
        }
        if matches!(
            event.r#type,
            PointerEventType::kUp | PointerEventType::kCancel
        ) {
            self.pointer_contacts.remove(&key);
        }
        if matches!(
            event.r#type,
            PointerEventType::kLeave | PointerEventType::kCancel
        ) || (event.r#type == PointerEventType::kUp && !event.hovering)
        {
            self.pointer_hover.remove(&key);
        }
        route
    }

    // Source components/input/render_widget_host_input_event_router.h:428,468;
    // .cc:902-1009. The initial hit owns the complete native touch sequence.
    pub fn Touch(&mut self, event: &TouchEvent, hit: Option<SurfaceRoute>) -> Option<SurfaceRoute> {
        if event.r#type == TouchEventType::kStart {
            let pressed = event
                .points
                .iter()
                .filter(|point| point.state == TouchPointState::kPressed)
                .count();
            if self.active_touches == 0 {
                self.touch_target = hit;
                self.active_touches = pressed;
                if event.unique_touch_event_id != 0 {
                    if let Some(route) = hit {
                        self.touchscreen_targets
                            .insert(event.unique_touch_event_id, route);
                    }
                }
            } else {
                self.active_touches = self.active_touches.saturating_add(pressed);
            }
        }
        let route = self.touch_target;
        if matches!(event.r#type, TouchEventType::kEnd | TouchEventType::kCancel) {
            let ended = event
                .points
                .iter()
                .filter(|point| {
                    matches!(
                        point.state,
                        TouchPointState::kReleased | TouchPointState::kCancelled
                    )
                })
                .count();
            self.active_touches = self.active_touches.saturating_sub(ended);
            if self.active_touches == 0 {
                self.touch_target = None;
            }
        }
        route
    }

    // Source render_widget_host_input_event_router.cc:1655-1785,1805-1903.
    pub fn Gesture(
        &mut self,
        event: &WebGestureEvent,
        hit: Option<SurfaceRoute>,
    ) -> Option<SurfaceRoute> {
        match event.SourceDevice() {
            GestureDevice::kTouchscreen => {
                if event.GetType() == Type::kGestureFlingCancel && self.last_fling_target.is_some()
                {
                    return self.last_fling_target;
                }
                if event.unique_touch_event_id == 0 {
                    self.touchscreen_gesture_target = hit;
                } else if event.GetType() == Type::kGestureTapDown {
                    self.touchscreen_gesture_target = self
                        .touchscreen_targets
                        .remove(&event.unique_touch_event_id)
                        .or(hit);
                }
                let route = self.touchscreen_gesture_target;
                if event.GetType() == Type::kGestureFlingStart {
                    self.last_fling_target = route;
                }
                if matches!(
                    event.GetType(),
                    Type::kGestureTap
                        | Type::kGestureLongTap
                        | Type::kGestureDoubleTap
                        | Type::kGestureTwoFingerTap
                        | Type::kGestureScrollEnd
                        | Type::kGestureFlingStart
                ) {
                    self.touchscreen_gesture_target = None;
                }
                route
            }
            GestureDevice::kTouchpad => {
                if event.GetType() == Type::kGestureFlingStart {
                    self.last_fling_target = self.wheel_target;
                    return self.wheel_target;
                }
                if event.GetType() == Type::kGestureFlingCancel {
                    return self.last_fling_target.or(hit);
                }
                if matches!(
                    event.GetType(),
                    Type::kGesturePinchBegin | Type::kGestureDoubleTap
                ) {
                    self.touchpad_gesture_target = hit;
                }
                let route = self.touchpad_gesture_target;
                if matches!(
                    event.GetType(),
                    Type::kGesturePinchEnd | Type::kGestureDoubleTap
                ) {
                    self.touchpad_gesture_target = None;
                }
                route
            }
            // Scrollbar/autoscroll gestures already have a Page target and
            // enter that Page's typed handle, rather than a window hit test.
            _ => None,
        }
    }

    pub fn HandledTouchStartOrFirstTouchMove(&mut self, document: Document, id: u32) {
        if self
            .touchscreen_targets
            .get(&id)
            .is_some_and(|route| route.document == document)
        {
            self.touchscreen_targets.remove(&id);
        }
    }

    // Source render_widget_host_input_event_router.cc:394-420. A removed or
    // unembedded target cannot redirect a continuation into another Page.
    pub fn SurfaceRemoved(&mut self, document: Document) {
        // A pending keyboard origin remains a tombstone until Char/KeyUp/new
        // RawKeyDown. The embedding drops delivery when that exact Document
        // has disappeared; it must never redirect Char into a successor Page.
        self.chrome.remove(&document);
        if self
            .widget_mouse_capture
            .is_some_and(|route| route.document == document)
        {
            self.widget_mouse_capture = None;
        }
        if self
            .last_mouse_down_target
            .is_some_and(|route| route.document == document)
        {
            self.last_mouse_down_target = None;
            self.mouse_buttons = 0;
        }
        if self
            .wheel_target
            .is_some_and(|target| target.document == document)
        {
            self.wheel_target = None;
        }
        self.autoscroll.remove(&document);
        self.pointer_contacts
            .retain(|_, target| target.document != document);
        self.pointer_hover
            .retain(|_, target| target.document != document);
        self.touchscreen_targets
            .retain(|_, target| target.document != document);
        if self
            .touch_target
            .is_some_and(|target| target.document == document)
        {
            self.touch_target = None;
            self.active_touches = 0;
        }
        if self
            .touchscreen_gesture_target
            .is_some_and(|target| target.document == document)
        {
            self.touchscreen_gesture_target = None;
        }
        if self
            .touchpad_gesture_target
            .is_some_and(|target| target.document == document)
        {
            self.touchpad_gesture_target = None;
        }
        if self
            .last_fling_target
            .is_some_and(|target| target.document == document)
        {
            self.last_fling_target = None;
        }
    }
}

pub(crate) fn TranslatePointer(event: &mut PointerEvent, origin: Offset) {
    event.position.x -= origin.x;
    event.position.y -= origin.y;
    for sample in &mut event.coalesced {
        TranslatePointer(sample, origin);
    }
    for sample in &mut event.predicted {
        TranslatePointer(sample, origin);
    }
}
pub(crate) fn TranslateTouch(event: &mut TouchEvent, origin: Offset) {
    for point in &mut event.points {
        point.position.x -= origin.x;
        point.position.y -= origin.y;
    }
    for sample in &mut event.coalesced {
        TranslateTouch(sample, origin);
    }
    for sample in &mut event.predicted {
        TranslateTouch(sample, origin);
    }
}
pub(crate) fn TranslateGesture(event: &mut WebGestureEvent, origin: Offset) {
    let point = event.PositionInWidget();
    let scale = event.FrameScale();
    event.SetPositionInWidget(foundation::gfx::PointF::new(
        point.x() - origin.x as f32 * scale,
        point.y() - origin.y as f32 * scale,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_char_routing_preserves_widget_identity_across_focus_and_removal() {
        let origin = (u64::MAX, 1);
        let next = (7, 2);
        let now = std::time::Instant::now();
        let mut raw = KeyEvent {
            r#type: KeyEventType::kDown,
            timestamp: Some(now),
            ..Default::default()
        };
        let mut character = raw.clone();
        character.r#type = KeyEventType::kChar;
        let mut routes = InputSurfaceRouting::default();
        assert_eq!(routes.Keyboard(&raw, Some(origin)), Some(origin));
        routes.SurfaceRemoved(origin);
        // Returning the original identity lets the embedding drop a removed
        // producer rather than accidentally selecting the newly focused Page.
        assert_eq!(routes.Keyboard(&character, Some(next)), Some(origin));
        assert_eq!(routes.Keyboard(&character, Some(next)), None);
        assert_eq!(routes.Keyboard(&raw, Some(origin)), Some(origin));
        character.timestamp = Some(now + std::time::Duration::from_millis(1));
        assert_eq!(routes.Keyboard(&character, Some(next)), None);
        character.is_standalone_char = true;
        assert_eq!(routes.Keyboard(&character, Some(next)), Some(next));
        assert_eq!(routes.Keyboard(&raw, Some(origin)), Some(origin));
        raw.r#type = KeyEventType::kUp;
        assert_eq!(routes.Keyboard(&raw, Some(next)), Some(next));
        character.is_standalone_char = false;
        character.timestamp = Some(now);
        assert_eq!(routes.Keyboard(&character, Some(next)), None);
    }

    #[test]
    fn phase_less_native_wheel_latches_from_normalized_begin() {
        let route = SurfaceRoute {
            document: (7, 3),
            origin: Offset { x: 4.0, y: 8.0 },
        };
        let event = WheelEvent {
            phase: interaction::input_event::WheelPhase::kBegan,
            native: Some(interaction::input_event::WheelMetadata {
                timestamp_seconds: 1.0,
                phase: 0,
                momentum_phase: 0,
            }),
            ..Default::default()
        };
        let mut routes = InputSurfaceRouting::default();

        assert_eq!(routes.Wheel(&event, Some(route)), Some(route));
    }

    #[test]
    fn mouse_up_ends_capture_before_a_delayed_page_effect() {
        let toolbar = SurfaceRoute {
            document: (1, 1),
            origin: Offset::default(),
        };
        let page = SurfaceRoute {
            document: (2, 1),
            origin: Offset { x: 0.0, y: 86.0 },
        };
        let mut routes = InputSurfaceRouting::default();
        let down = MouseEvent {
            r#type: MouseEventType::kDown,
            button: interaction::input_event::MouseButton::kPrimary,
            buttons: 1,
            ..Default::default()
        };
        assert_eq!(routes.CapturedMouseTarget(&down), None);
        routes.MouseDownTarget(toolbar, down.buttons);
        routes.CaptureMouseEventsToWidget(toolbar, true);

        let up = MouseEvent {
            r#type: MouseEventType::kUp,
            ..Default::default()
        };
        assert_eq!(routes.CapturedMouseTarget(&up), Some(toolbar));
        assert!(routes.widget_mouse_capture.is_none());

        // The renderer effect can arrive after the raw up. It must not revive
        // capture and steal the next press from another Page surface.
        routes.CaptureMouseEventsToWidget(toolbar, true);
        assert!(routes.widget_mouse_capture.is_none());
        assert_eq!(routes.CapturedMouseTarget(&down), None);
        routes.MouseDownTarget(page, down.buttons);
        // A capture effect from the prior toolbar sequence can arrive after
        // the next press, but it cannot capture a different document.
        routes.CaptureMouseEventsToWidget(toolbar, true);
        assert!(routes.widget_mouse_capture.is_none());
        routes.CaptureMouseEventsToWidget(page, true);
        assert_eq!(routes.widget_mouse_capture, Some(page));
    }
}
