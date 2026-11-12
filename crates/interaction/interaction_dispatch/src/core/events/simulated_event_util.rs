// Copyright 2021 The Chromium Authors
// Use of this source code is governed by a BSD-style license in Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/simulated_event_util.h:16-24
// Source: third_party/blink/renderer/core/events/simulated_event_util.cc:41-216
// Native FrameView/LayoutBox queries are borrowed through the narrow geometry
// port; event construction retains the original clock and input capabilities.

use super::{
    mouse_event::{MenuSourceType, MouseCoordinateContext, MouseEvent, SyntheticEventType},
    pointer_event::{PointerEvent, PointerEventInit},
    pointer_event_factory::PointerEventFactory,
    ui_event::{AbstractViewRef, InputDeviceCapabilitiesRef},
    ui_event_with_key_state::{FindEventWithKeyState, UIEventWithKeyState},
};
use crate::core::dom::events::{
    event::{EventInterface, EventRef, EventTargetRef},
    simulated_click_options::SimulatedClickCreationScope,
};
use interaction_input::{
    base::time::time::TimeTicks, public::common::input::web_input_event::WebInputEvent,
};

/// Source cc:41-81. Values are obtained from the native LayoutBox/FrameView
/// at the source query point; None is unavailable, not invented screen data.
#[derive(Clone, Copy)]
pub enum AccessibilityCoordinates {
    NoLayoutBox,
    Prepared {
        screen_x: f64,
        screen_y: f64,
        client_x: f64,
        client_y: f64,
    },
}
/// Borrowed leaves of the existing LayoutBox/FrameView owners. The caller
/// retains their lifetime; this port stores neither a tree nor geometry.
pub trait AccessibilityGeometry {
    fn RootFrameBorderBoxCenter(&self) -> Option<(foundation::PhysicalOffset, f32)>;
    fn ConvertFromRootFrame(
        &self,
        point: foundation::PhysicalOffset,
    ) -> Option<foundation::PhysicalOffset>;
    fn FrameToScreen(&self, rect: foundation::gfx::Rect) -> Option<foundation::gfx::Rect>;
}

/// Source cc:41-76. No element LayoutBox leaves the initializer positionless.
/// Missing embedding coordinate capabilities remain an explicit typed error.
pub fn PrepareAccessibilityCoordinates(
    geometry: &dyn AccessibilityGeometry,
) -> Result<AccessibilityCoordinates, RequiredSimulatedEventInput> {
    use foundation::gfx;
    let Some((root_frame_center, zoom)) = geometry.RootFrameBorderBoxCenter() else {
        return Ok(AccessibilityCoordinates::NoLayoutBox);
    };
    let frame_center = geometry
        .ConvertFromRootFrame(root_frame_center)
        .ok_or(RequiredSimulatedEventInput::AccessibilityCoordinates)?;
    let frame_center_point = foundation::ToRoundedPoint(frame_center);
    let center_rect = gfx::Rect::new(frame_center_point, gfx::Size::new(1, 1));
    let screen_center = geometry
        .FrameToScreen(center_rect)
        .ok_or(RequiredSimulatedEventInput::AccessibilityCoordinates)?
        .origin();
    // AdjustForAbsoluteZoom::AdjustInt reads this object's EffectiveZoom.
    let adjust = |value: i32| {
        if zoom == 1.0 {
            return value as f64;
        }
        let mut value = value as f32;
        if zoom > 1.0 {
            value += if value < 0.0 { -0.5 } else { 0.5 };
        }
        foundation::numeric_conversion::RoundForImpreciseConversionI32((value / zoom) as f64) as f64
    };
    Ok(AccessibilityCoordinates::Prepared {
        screen_x: adjust(screen_center.x()),
        screen_y: adjust(screen_center.y()),
        client_x: adjust(frame_center_point.x()),
        client_y: adjust(frame_center_point.y()),
    })
}

pub struct SimulatedEventEnvironment {
    pub view: Option<AbstractViewRef>,
    pub non_touch_capabilities: Option<InputDeviceCapabilitiesRef>,
    pub coordinates: MouseCoordinateContext,
    pub now: Option<TimeTicks>,
    pub accessibility_coordinates: Option<AccessibilityCoordinates>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequiredSimulatedEventInput {
    Timestamp,
    AccessibilityCoordinates,
}

// Source: simulated_event_util.cc:79-109.
fn PopulateSimulatedMouseEventInit(
    initializer: &mut PointerEventInit,
    underlying: Option<&EventRef>,
    scope: SimulatedClickCreationScope,
    environment: &SimulatedEventEnvironment,
) -> Result<(), RequiredSimulatedEventInput> {
    let modifiers = FindEventWithKeyState(underlying.cloned())
        .and_then(|event| {
            event
                .borrow()
                .AsUIEventWithKeyState()
                .map(|state| state.GetModifiers())
        })
        .unwrap_or(WebInputEvent::kNoModifiers as i32);
    let mouse = &mut initializer.mouse_event_init;
    if scope == SimulatedClickCreationScope::kFromAccessibility && environment.view.is_some() {
        match environment
            .accessibility_coordinates
            .ok_or(RequiredSimulatedEventInput::AccessibilityCoordinates)?
        {
            AccessibilityCoordinates::NoLayoutBox => {}
            AccessibilityCoordinates::Prepared {
                screen_x,
                screen_y,
                client_x,
                client_y,
            } => {
                mouse.screen_x = screen_x;
                mouse.screen_y = screen_y;
                mouse.client_x = client_x;
                mouse.client_y = client_y;
            }
        }
    }
    if let Some(underlying) = underlying {
        let event = underlying.borrow();
        if let Some(underlying) = event.AsMouseEvent() {
            mouse.screen_x = underlying.screenX();
            mouse.screen_y = underlying.screenY();
            mouse.event_modifier_init.ui_event_init.source_capabilities =
                environment.non_touch_capabilities.clone();
        }
    }
    let ui = &mut mouse.event_modifier_init.ui_event_init;
    ui.event_init.bubbles = true;
    ui.event_init.cancelable = true;
    ui.event_init.composed = true;
    ui.view = environment.view.clone();
    UIEventWithKeyState::SetFromWebInputEventModifiers(&mut mouse.event_modifier_init, modifiers);
    mouse.buttons = MouseEvent::WebInputEventModifiersToButtons(modifiers as u32);
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EventClassType {
    Mouse,
    Pointer,
}

// Source: simulated_event_util.cc:113-192.
fn CreateMouseOrPointerEvent(
    class: EventClassType,
    event_type: &str,
    underlying: Option<&EventRef>,
    scope: SimulatedClickCreationScope,
    environment: &SimulatedEventEnvironment,
) -> Result<EventRef, RequiredSimulatedEventInput> {
    let mut initializer = PointerEventInit::default();
    PopulateSimulatedMouseEventInit(&mut initializer, underlying, scope, environment)?;
    let timestamp = underlying
        .map(|event| event.borrow().BaseEvent().PlatformTimeStamp())
        .or(environment.now)
        .ok_or(RequiredSimulatedEventInput::Timestamp)?;
    let underlying_mouse = underlying.and_then(|event| {
        let event = event.borrow();
        event
            .AsMouseEvent()
            .map(|mouse| (mouse.clientX(), mouse.clientY()))
    });
    let synthetic_type = if underlying_mouse.is_some() {
        SyntheticEventType::kRealOrIndistinguishable
    } else {
        SyntheticEventType::kPositionless
    };
    if scope == SimulatedClickCreationScope::kFromAccessibility {
        if matches!(event_type, "click" | "pointerdown" | "mousedown") {
            initializer.mouse_event_init.button = 0;
            initializer.mouse_event_init.buttons =
                MouseEvent::WebInputEventModifiersToButtons(WebInputEvent::kLeftButtonDown as u32);
        }
        if matches!(event_type, "pointerup" | "mouseup") {
            initializer.mouse_event_init.button = 0;
        }
        if event_type == "click" {
            initializer
                .mouse_event_init
                .event_modifier_init
                .ui_event_init
                .detail = 1;
        }
    }
    let trusted = scope == SimulatedClickCreationScope::kFromAccessibility
        || (scope == SimulatedClickCreationScope::kFromUserAgent
            && underlying.is_none_or(|event| event.borrow().BaseEvent().isTrusted()));
    if class == EventClassType::Pointer {
        initializer.pointer_id = Some(PointerEventFactory::kReservedNonPointerId);
        if scope == SimulatedClickCreationScope::kFromAccessibility {
            initializer.pointer_id = Some(PointerEventFactory::kMouseId);
            initializer.pointer_type = Some("mouse".into());
            initializer.is_primary = Some(true);
        } else if scope == SimulatedClickCreationScope::kFromUserAgent {
            if let Some(underlying) = underlying {
                let event = underlying.borrow();
                if let Some(pointer) = event.AsPointerEvent() {
                    initializer.pointer_id = Some(pointer.pointerId());
                    initializer.pointer_type = Some(pointer.pointerType().into());
                    initializer.is_primary = Some(pointer.isPrimary());
                    initializer
                        .mouse_event_init
                        .event_modifier_init
                        .ui_event_init
                        .detail = pointer.detail();
                }
            }
        }
        let mut event = PointerEvent::new(
            event_type,
            &initializer,
            timestamp,
            synthetic_type,
            MenuSourceType::kNone,
            false,
            environment.coordinates.clone(),
        );
        event.BaseEventMut().SetTrusted(trusted);
        event.BaseEventMut().SetUnderlyingEvent(underlying.cloned());
        if let Some((x, y)) = underlying_mouse {
            event.InitCoordinates(x, y, None);
        }
        Ok(EventRef::new(event))
    } else {
        let mut event = MouseEvent::new(
            event_type,
            &initializer.mouse_event_init,
            timestamp,
            synthetic_type,
            MenuSourceType::kNone,
            None,
            environment.coordinates.clone(),
        );
        event.BaseEventMut().SetTrusted(trusted);
        event.BaseEventMut().SetUnderlyingEvent(underlying.cloned());
        if let Some((x, y)) = underlying_mouse {
            event.InitCoordinates(x, y, None);
        }
        Ok(EventRef::new(event))
    }
}

pub struct SimulatedEventUtil;
impl SimulatedEventUtil {
    // Source: simulated_event_util.cc:196-216; h:20-23.
    pub fn CreateEvent(
        event_type: &str,
        _node: &EventTargetRef,
        underlying: Option<&EventRef>,
        scope: SimulatedClickCreationScope,
        environment: &SimulatedEventEnvironment,
    ) -> Result<EventRef, RequiredSimulatedEventInput> {
        debug_assert!(matches!(
            event_type,
            "click" | "mousedown" | "mouseup" | "pointerdown" | "pointerup"
        ));
        let class = if matches!(event_type, "click" | "pointerdown" | "pointerup") {
            EventClassType::Pointer
        } else {
            EventClassType::Mouse
        };
        CreateMouseOrPointerEvent(class, event_type, underlying, scope, environment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        dom::events::{
            event::{EventTarget, EventTargetKind},
            event_dispatch_result::DispatchEventResult,
            event_dispatcher::{
                EventDispatcher, SimulatedEventFactory, SimulatedNodeDefaultActions,
            },
        },
        events::{
            keyboard_event::{KeyboardEvent, KeyboardEventInit},
            mouse_event::MenuSourceType,
        },
    };
    use std::{cell::Cell, rc::Rc};
    struct Target;
    impl EventTarget for Target {
        fn Kind(&self) -> EventTargetKind {
            EventTargetKind::Node
        }
    }
    struct Ports {
        active: Cell<bool>,
    }
    impl SimulatedNodeDefaultActions for Ports {
        fn IsDisabledFormControl(&self, _: &EventTargetRef) -> bool {
            false
        }
        fn IsElement(&self, _: &EventTargetRef) -> bool {
            true
        }
        fn SetActive(&self, _: &EventTargetRef, active: bool) {
            self.active.set(active);
        }
    }
    fn environment() -> SimulatedEventEnvironment {
        SimulatedEventEnvironment {
            view: None,
            non_touch_capabilities: None,
            coordinates: Default::default(),
            now: None,
            accessibility_coordinates: None,
        }
    }
    impl SimulatedEventFactory for Ports {
        fn CreateEvent(
            &self,
            kind: &'static str,
            node: &EventTargetRef,
            underlying: Option<&EventRef>,
            scope: SimulatedClickCreationScope,
        ) -> Result<EventRef, RequiredSimulatedEventInput> {
            SimulatedEventUtil::CreateEvent(kind, node, underlying, scope, &environment())
        }
    }
    #[test]
    fn accessibility_coordinates_use_live_box_frame_screen_and_absolute_zoom() {
        struct Geometry(bool);
        impl AccessibilityGeometry for Geometry {
            fn RootFrameBorderBoxCenter(&self) -> Option<(foundation::PhysicalOffset, f32)> {
                self.0.then(|| {
                    (
                        foundation::PhysicalOffset::new(
                            foundation::LayoutUnit::from_f64(21.25),
                            foundation::LayoutUnit::from_f64(-17.75),
                        ),
                        2.0,
                    )
                })
            }
            fn ConvertFromRootFrame(
                &self,
                point: foundation::PhysicalOffset,
            ) -> Option<foundation::PhysicalOffset> {
                Some(
                    point
                        + foundation::PhysicalOffset::new(
                            foundation::LayoutUnit::from_f64(10.0),
                            foundation::LayoutUnit::from_f64(20.0),
                        ),
                )
            }
            fn FrameToScreen(&self, rect: foundation::gfx::Rect) -> Option<foundation::gfx::Rect> {
                assert_eq!(
                    (rect.x(), rect.y(), rect.width(), rect.height()),
                    (31, 2, 1, 1)
                );
                Some(foundation::gfx::Rect::new(
                    foundation::gfx::Point::new(rect.x() + 200, rect.y() + 100),
                    rect.size(),
                ))
            }
        }
        let AccessibilityCoordinates::Prepared {
            screen_x,
            screen_y,
            client_x,
            client_y,
        } = PrepareAccessibilityCoordinates(&Geometry(true)).unwrap()
        else {
            panic!("box has coordinates")
        };
        assert_eq!(
            (screen_x, screen_y, client_x, client_y),
            (115.0, 51.0, 15.0, 1.0)
        );
        assert!(matches!(
            PrepareAccessibilityCoordinates(&Geometry(false)),
            Ok(AccessibilityCoordinates::NoLayoutBox)
        ));
    }

    #[test]
    fn simulated_button_click_keeps_native_provenance_and_reentry_guard() {
        let node: EventTargetRef = Rc::new(Target);
        let mut key = KeyboardEvent::WithInitializer(
            "keypress",
            &KeyboardEventInit {
                key: "Enter".into(),
                char_code: 13,
                ..Default::default()
            },
            TimeTicks::FromInternalValue(73),
        );
        key.BaseEventMut().SetTrusted(true);
        let key = EventRef::new(key);
        let ports = Ports {
            active: Cell::new(true),
        };
        let calls = Cell::new(0);
        EventDispatcher::DispatchSimulatedClickWithDispatch(
            node.clone(),
            Some(&key),
            SimulatedClickCreationScope::kFromUserAgent,
            &ports,
            &ports,
            &mut |event| {
                calls.set(calls.get() + 1);
                assert!(!ports.active.get());
                {
                    let event = event.borrow();
                    let pointer = event
                        .AsPointerEvent()
                        .expect("UA click is a native PointerEvent");
                    assert_eq!(
                        pointer.pointerId(),
                        PointerEventFactory::kReservedNonPointerId
                    );
                    assert!(!pointer.isPrimary());
                    assert_eq!(pointer.pointerType(), "");
                    assert!(event.BaseEvent().isTrusted());
                    assert_eq!(
                        event.BaseEvent().PlatformTimeStamp(),
                        TimeTicks::FromInternalValue(73)
                    );
                    assert!(event
                        .BaseEvent()
                        .UnderlyingEvent()
                        .unwrap()
                        .IsSameEvent(&key));
                }
                EventDispatcher::DispatchSimulatedClickWithDispatch(
                    node.clone(),
                    Some(&key),
                    SimulatedClickCreationScope::kFromUserAgent,
                    &ports,
                    &ports,
                    &mut |_| {
                        calls.set(calls.get() + 1);
                        DispatchEventResult::kNotCanceled
                    },
                )
                .unwrap();
                DispatchEventResult::kNotCanceled
            },
        )
        .unwrap();
        assert_eq!(calls.get(), 1);
        let mut init = PointerEventInit::default();
        init.pointer_id = Some(9);
        init.pointer_type = Some("pen".into());
        init.is_primary = Some(true);
        init.mouse_event_init.client_x = 12.0;
        init.mouse_event_init.client_y = 24.0;
        init.mouse_event_init
            .event_modifier_init
            .ui_event_init
            .detail = 2;
        let mut pointer = PointerEvent::new(
            "pointerup",
            &init,
            TimeTicks::FromInternalValue(101),
            SyntheticEventType::kRealOrIndistinguishable,
            MenuSourceType::kNone,
            false,
            Default::default(),
        );
        pointer.BaseEventMut().SetTrusted(true);
        let pointer = EventRef::new(pointer);
        let click = SimulatedEventUtil::CreateEvent(
            "click",
            &node,
            Some(&pointer),
            SimulatedClickCreationScope::kFromUserAgent,
            &environment(),
        )
        .unwrap();
        let click = click.borrow();
        let native = click.AsPointerEvent().unwrap();
        assert_eq!(
            (
                native.pointerId(),
                native.pointerType(),
                native.isPrimary(),
                native.detail()
            ),
            (9, "pen", true, 2)
        );
        assert_eq!((native.clientX(), native.clientY()), (12.0, 24.0));
    }
}
