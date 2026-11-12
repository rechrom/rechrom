/*
 * Copyright (C) 2001 Peter Kelly (pmk@post.com)
 * Copyright (C) 2001 Tobias Anton (anton@stud.fbi.fh-darmstadt.de)
 * Copyright (C) 2006 Samuel Weinig (sam.weinig@gmail.com)
 * Copyright (C) 2003, 2004, 2005, 2006, 2008 Apple Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or modify it
 * under the terms of the GNU Library General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or (at your
 * option) any later version.
 *
 * This library is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE. See the GNU Library General Public
 * License for more details. You should have received a copy of the GNU
 * Library General Public License along with this library; see COPYING.LIB.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/mouse_event.h:1-253
// Source: third_party/blink/renderer/core/events/mouse_event.cc:1-582
// Rust composition and shared owner projections replace includes, namespace,
// wrapper metadata, C++ inheritance and GC tracing.

use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::web_input_event::WebInputEvent;
use interaction_input::public::common::input::web_pointer_properties::{
    Button, Buttons, PointF, WebPointerProperties,
};
pub use interaction_input::ui::base::mojom::menu_source_type::MenuSourceType;

use crate::core::dom::events::event::{
    Event, EventClock, EventInterface, EventTargetKind, EventTargetRef,
};
use crate::core::dom::events::event_dispatch_result::DispatchEventResult;
use crate::core::dom::events::event_dispatcher::{EventDispatchHook, EventDispatcher};

pub use super::mouse_event_init::MouseEventInit;
use super::ui_event::{AbstractViewRef, InputDeviceCapabilitiesRef, UIEvent, UIEventInterface};
use super::ui_event_with_key_state::{UIEventWithKeyState, UIEventWithKeyStateInterface};

// Source: mouse_event.h:47-56
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntheticEventType {
    kRealOrIndistinguishable,
    kFromTouch,
    kPositionless,
}

// Source: mouse_event.h:141-147
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionType {
    kPosition,
    kPositionless,
}

// Source: mouse_event.cc:54-60,170-182,204-217
// A view query returns None for a non-LocalDOMWindow or a detached frame. The
// retained AbstractView is only an identity; the injected provider owns these
// small platform-neutral geometry facts and pointer-lock state.
pub trait MouseViewGeometry {
    fn IsLocalDOMWindow(&self, view: &AbstractViewRef) -> bool;
    fn HasFrameView(&self, view: &AbstractViewRef) -> bool;
    fn LayoutZoomFactor(&self, view: &AbstractViewRef) -> Option<f32>;
    fn LayoutViewportScrollOffset(&self, view: &AbstractViewRef) -> Option<(i32, i32)>;
    fn ConvertFromRootFrame(&self, view: &AbstractViewRef, point: PointF) -> Option<PointF>;
    fn PointerLockPosition(&self, view: &AbstractViewRef) -> Option<(PointF, PointF)>;
}

// Source: mouse_event.cc:63-76,479-500,510-522
// These are borrowed operations on the canonical layout owner. The cursor is
// restricted to the source's SVG-root walk and coordinate conversion; it does
// not expose LayoutObject storage or a rendering lifecycle.
pub type MouseLayoutRef = Rc<dyn MouseCoordinateLayout>;
pub trait MouseCoordinateLayout {
    fn IsSVG(&self) -> bool;
    fn IsSVGRoot(&self) -> bool;
    fn Parent(&self) -> Option<MouseLayoutRef>;
    fn Node(&self) -> Option<EventTargetRef>;
    fn AbsoluteToLocalPoint(&self, point: PointF) -> PointF;
    fn BorderOffset(&self) -> Option<(f32, f32)>;
    fn DocumentToFrame(&self, point: PointF) -> Option<PointF>;
    fn EnclosingSelfPaintingLayerAbsoluteOrigin(&self) -> (f32, f32);
}

// Source: mouse_event.cc:458-479,506-508
// The embedding satisfies Document::UpdateStyleAndLayout(kInput) before
// admitting this synchronous query (cc:475-476). This interface then borrows
// the resident result. It neither buffers the result nor drives lifecycle.
pub trait MouseRelativeGeometry {
    fn Layout(&self, node: &EventTargetRef) -> Option<MouseLayoutRef>;
    fn ParentNode(&self, node: &EventTargetRef) -> Option<EventTargetRef>;
    fn DocumentView(&self, node: &EventTargetRef) -> Option<AbstractViewRef>;
}

// Source: mouse_event.cc:98-119,276-278
pub trait MouseScriptContext {
    fn IsIsolatedWorld(&self) -> bool;
    fn WindowExecutionContext(&self) -> Option<AbstractViewRef>;
}

#[derive(Clone, Default)]
pub struct MouseCoordinateContext {
    pub views: Option<Rc<dyn MouseViewGeometry>>,
    pub relative: Option<Rc<dyn MouseRelativeGeometry>>,
}

// Source: mouse_event.h:198-208,230-234
#[derive(Clone, Copy, Default)]
struct RelativePosition {
    offset_x: f64,
    offset_y: f64,
    layer_x: f32,
    layer_y: f32,
    cached: bool,
}

// Source: mouse_event.h:43,198-243
pub struct MouseEvent {
    key_state: UIEventWithKeyState,
    screen_x_: f64,
    screen_y_: f64,
    client_x_: f64,
    client_y_: f64,
    page_x_: f64,
    page_y_: f64,
    relative_position_: Cell<RelativePosition>,
    movement_delta_: (i32, i32),
    absolute_location_: PointF,
    position_type_: PositionType,
    button_: i16,
    buttons_: u16,
    related_target_: Option<EventTargetRef>,
    synthetic_event_type_: SyntheticEventType,
    menu_source_type_: MenuSourceType,
    coordinate_context_: MouseCoordinateContext,
}

// Source: mouse_event.cc:54-61
fn LayoutZoomFactor(view: Option<&AbstractViewRef>, context: &MouseCoordinateContext) -> f32 {
    view.and_then(|view| context.views.as_ref()?.LayoutZoomFactor(view))
        .unwrap_or(1.0)
}

// Source: mouse_event.cc:63-77
fn FindTargetLayoutObject(
    target: &mut Option<EventTargetRef>,
    geometry: &dyn MouseRelativeGeometry,
) -> Option<MouseLayoutRef> {
    let mut layout = geometry.Layout(target.as_ref()?)?;
    if !layout.IsSVG() {
        return Some(layout);
    }
    while !layout.IsSVGRoot() {
        layout = layout
            .Parent()
            .expect("SVG layout ancestor chain reaches its root");
    }
    *target = layout.Node();
    Some(layout)
}

// Source: mouse_event.cc:79-94
fn ButtonsToWebInputEventModifiers(buttons: u16) -> i32 {
    let mut modifiers = 0;
    for (button, modifier) in [
        (Buttons::kLeft as u16, WebInputEvent::kLeftButtonDown),
        (Buttons::kRight as u16, WebInputEvent::kRightButtonDown),
        (Buttons::kMiddle as u16, WebInputEvent::kMiddleButtonDown),
        (Buttons::kBack as u16, WebInputEvent::kBackButtonDown),
        (Buttons::kForward as u16, WebInputEvent::kForwardButtonDown),
    ] {
        if buttons & button != 0 {
            modifiers |= modifier;
        }
    }
    modifiers
}

impl MouseEvent {
    // Source: mouse_event.h:58; mouse_event.cc:133-138
    pub fn Create(clock: &dyn EventClock) -> Self {
        Self::Uninitialized(clock.Now())
    }

    pub fn Uninitialized(time: TimeTicks) -> Self {
        Self {
            key_state: UIEventWithKeyState::Uninitialized(time),
            screen_x_: 0.0,
            screen_y_: 0.0,
            client_x_: 0.0,
            client_y_: 0.0,
            page_x_: 0.0,
            page_y_: 0.0,
            relative_position_: Cell::new(RelativePosition::default()),
            movement_delta_: (0, 0),
            absolute_location_: PointF::default(),
            position_type_: PositionType::kPosition,
            button_: 0,
            buttons_: 0,
            related_target_: None,
            synthetic_event_type_: SyntheticEventType::kRealOrIndistinguishable,
            menu_source_type_: MenuSourceType::kNone,
            coordinate_context_: MouseCoordinateContext::default(),
        }
    }

    // Source: mouse_event.h:67-69; mouse_event.cc:98-120
    pub fn CreateFromScript(
        script: Option<&dyn MouseScriptContext>,
        event_type: impl Into<String>,
        initializer: &MouseEventInit,
        clock: &dyn EventClock,
        coordinates: MouseCoordinateContext,
    ) -> Self {
        let mut fallback = None;
        if let Some(script) = script {
            if script.IsIsolatedWorld() {
                let init = &initializer.event_modifier_init;
                // Preserve the source call's ctrl, alt, shift, meta order.
                UIEventWithKeyState::DidCreateEventInIsolatedWorld(
                    init.ctrl_key,
                    init.alt_key,
                    init.shift_key,
                    init.meta_key,
                );
            }
            if initializer.event_modifier_init.ui_event_init.view.is_none() {
                fallback = script.WindowExecutionContext();
            }
        }
        Self::new(
            event_type,
            initializer,
            clock.Now(),
            SyntheticEventType::kRealOrIndistinguishable,
            MenuSourceType::kNone,
            fallback,
            coordinates,
        )
    }

    // Source: mouse_event.h:61-65,71-77; mouse_event.cc:122-131,140-161
    pub fn new(
        event_type: impl Into<String>,
        initializer: &MouseEventInit,
        platform_time_stamp: TimeTicks,
        synthetic_event_type: SyntheticEventType,
        menu_source_type: MenuSourceType,
        fallback_dom_window: Option<AbstractViewRef>,
        coordinate_context: MouseCoordinateContext,
    ) -> Self {
        let mut event = Self {
            key_state: UIEventWithKeyState::WithInitializer(
                event_type,
                &initializer.event_modifier_init,
                platform_time_stamp,
            ),
            screen_x_: initializer.screen_x,
            screen_y_: initializer.screen_y,
            client_x_: 0.0,
            client_y_: 0.0,
            page_x_: 0.0,
            page_y_: 0.0,
            relative_position_: Cell::new(RelativePosition::default()),
            movement_delta_: (initializer.movement_x, initializer.movement_y),
            absolute_location_: PointF::default(),
            position_type_: if synthetic_event_type == SyntheticEventType::kPositionless {
                PositionType::kPositionless
            } else {
                PositionType::kPosition
            },
            button_: initializer.button,
            buttons_: initializer.buttons,
            related_target_: initializer.related_target.clone(),
            synthetic_event_type_: synthetic_event_type,
            menu_source_type_: menu_source_type,
            coordinate_context_: coordinate_context,
        };
        event.InitCoordinates(
            initializer.client_x,
            initializer.client_y,
            fallback_dom_window,
        );
        event.key_state.modifiers_ |= ButtonsToWebInputEventModifiers(event.buttons_);
        event
    }

    /// The same retained coordinate services used by this native event. A
    /// simulated derivative borrows these services rather than rebuilding them.
    pub fn CoordinateContext(&self) -> MouseCoordinateContext {
        self.coordinate_context_.clone()
    }

    // Source: mouse_event.h:185-187; mouse_event.cc:163-195
    pub fn InitCoordinates(
        &mut self,
        client_x: f64,
        client_y: f64,
        fallback: Option<AbstractViewRef>,
    ) {
        self.client_x_ = client_x;
        self.page_x_ = client_x;
        self.client_y_ = client_y;
        self.page_y_ = client_y;
        self.absolute_location_ = PointF::new(client_x as f32, client_y as f32);
        let view = self.view();
        let local_view = view.as_ref().filter(|view| {
            self.coordinate_context_
                .views
                .as_ref()
                .is_some_and(|geometry| geometry.IsLocalDOMWindow(view))
        });
        let zoom = LayoutZoomFactor(local_view.or(fallback.as_ref()), &self.coordinate_context_);
        if let (Some(view), Some(geometry)) = (local_view, &self.coordinate_context_.views) {
            if let Some((x, y)) = geometry.LayoutViewportScrollOffset(view) {
                self.page_x_ += (x as f32 / zoom) as f64;
                self.page_y_ += (y as f32 / zoom) as f64;
            }
        }
        self.absolute_location_.ScaleUniform(zoom);
        self.relative_position_.set(RelativePosition {
            offset_x: self.page_x_,
            offset_y: self.page_y_,
            layer_x: self.page_x_ as f32,
            layer_y: self.page_y_ as f32,
            cached: false,
        });
    }

    // Source: mouse_event.h:81-84; mouse_event.cc:197-234
    pub fn SetCoordinatesFromWebPointerProperties(
        properties: &WebPointerProperties,
        window: Option<&AbstractViewRef>,
        geometry: Option<&dyn MouseViewGeometry>,
        initializer: &mut MouseEventInit,
    ) {
        let mut client = PointF::default();
        let mut screen = *properties.PositionInScreen();
        if let (Some(window), Some(geometry)) = (window, geometry) {
            if geometry.HasFrameView(window) {
                let mut root = *properties.PositionInWidget();
                if let Some((locked_root, locked_screen)) = geometry.PointerLockPosition(window) {
                    root = locked_root;
                    screen = locked_screen;
                }
                if let Some(mut frame) = geometry.ConvertFromRootFrame(window, root) {
                    frame.ScaleUniform(1.0 / geometry.LayoutZoomFactor(window).unwrap_or(1.0));
                    client = frame;
                }
            }
        }
        initializer.screen_x = screen.x() as f64;
        initializer.screen_y = screen.y() as f64;
        initializer.client_x = client.x() as f64;
        initializer.client_y = client.y() as f64;
        if properties.is_raw_movement_event {
            initializer.movement_x = properties.movement_x;
            initializer.movement_y = properties.movement_y;
        }
    }

    // Source: mouse_event.h:80; mouse_event.cc:236-254
    pub fn WebInputEventModifiersToButtons(modifiers: u32) -> u16 {
        let mut buttons = 0;
        for (modifier, button) in [
            (WebInputEvent::kLeftButtonDown, Buttons::kLeft as u16),
            (WebInputEvent::kRightButtonDown, Buttons::kRight as u16),
            (WebInputEvent::kMiddleButtonDown, Buttons::kMiddle as u16),
            (WebInputEvent::kBackButtonDown, Buttons::kBack as u16),
            (WebInputEvent::kForwardButtonDown, Buttons::kForward as u16),
        ] {
            if modifiers & modifier as u32 != 0 {
                buttons |= button;
            }
        }
        buttons
    }

    // Source: mouse_event.h:86-102; mouse_event.cc:256-284
    #[allow(clippy::too_many_arguments)]
    pub fn initMouseEvent(
        &mut self,
        script: Option<&dyn MouseScriptContext>,
        event_type: impl Into<String>,
        bubbles: bool,
        cancelable: bool,
        view: Option<AbstractViewRef>,
        detail: i32,
        screen_x: i32,
        screen_y: i32,
        client_x: i32,
        client_y: i32,
        ctrl: bool,
        alt: bool,
        shift: bool,
        meta: bool,
        button: i16,
        related_target: Option<EventTargetRef>,
        buttons: u16,
    ) {
        if self.IsBeingDispatched() {
            return;
        }
        if script.is_some_and(|script| script.IsIsolatedWorld()) {
            UIEventWithKeyState::DidCreateEventInIsolatedWorld(ctrl, alt, shift, meta);
        }
        self.InitModifiers(ctrl, alt, shift, meta);
        self.InitMouseEventInternal(
            event_type,
            bubbles,
            cancelable,
            view,
            detail,
            screen_x as f64,
            screen_y as f64,
            client_x as f64,
            client_y as f64,
            self.GetModifiers(),
            button,
            related_target,
            None,
            buttons,
        );
    }

    // Source: mouse_event.h:211-224; mouse_event.cc:286-314
    #[allow(clippy::too_many_arguments)]
    pub fn InitMouseEventInternal(
        &mut self,
        event_type: impl Into<String>,
        bubbles: bool,
        cancelable: bool,
        view: Option<AbstractViewRef>,
        detail: i32,
        screen_x: f64,
        screen_y: f64,
        client_x: f64,
        client_y: f64,
        modifiers: i32,
        button: i16,
        related_target: Option<EventTargetRef>,
        capabilities: Option<InputDeviceCapabilitiesRef>,
        buttons: u16,
    ) {
        self.InitUIEventInternal(
            event_type,
            bubbles,
            cancelable,
            related_target.clone(),
            view,
            detail,
            capabilities,
        );
        self.screen_x_ = screen_x;
        self.screen_y_ = screen_y;
        self.button_ = button;
        self.buttons_ = buttons;
        self.related_target_ = related_target;
        self.key_state.modifiers_ = modifiers;
        self.InitCoordinates(client_x, client_y, None);
        // Source FIXME: SyntheticEventType is not reset here.
    }

    // Source: mouse_event.h:104-107; mouse_event.cc:316-323
    pub fn InitCoordinatesForTesting(
        &mut self,
        screen_x: f64,
        screen_y: f64,
        client_x: f64,
        client_y: f64,
    ) {
        self.screen_x_ = screen_x;
        self.screen_y_ = screen_y;
        self.InitCoordinates(client_x, client_y, None);
    }

    // Source: mouse_event.h:111-115; mouse_event.cc:333-353
    pub fn button(&self) -> i16 {
        if self.button_ == -1
            || matches!(
                self.r#type(),
                "mousemove" | "mouseleave" | "mouseenter" | "mouseover" | "mouseout"
            )
        {
            0
        } else {
            self.button_
        }
    }
    pub fn IsLeftButton(&self) -> bool {
        self.button() == Button::kLeft as i16
    }
    pub fn IsLinkClickButton(&self) -> bool {
        matches!(self.button(), 0 | 1)
    }
    // Source: mouse_event.h:117-125,132,139
    pub fn buttons(&self) -> u16 {
        self.buttons_
    }
    pub fn ButtonDown(&self) -> bool {
        self.button_ != -1
    }
    pub fn relatedTarget(&self) -> Option<EventTargetRef> {
        self.related_target_.clone()
    }
    pub fn SetRelatedTarget(&mut self, target: Option<EventTargetRef>) {
        self.related_target_ = target;
    }
    pub fn GetSyntheticEventType(&self) -> SyntheticEventType {
        self.synthetic_event_type_
    }
    pub fn FromTouch(&self) -> bool {
        self.synthetic_event_type_ == SyntheticEventType::kFromTouch
    }
    pub fn ClickCount(&self) -> i32 {
        self.detail()
    }
    // Source: mouse_event.h:134-137; mouse_event.cc:325-331,355-362
    pub fn InterfaceName(&self) -> &str {
        "MouseEvent"
    }
    pub fn IsMouseEvent(&self) -> bool {
        true
    }
    pub fn which(&self) -> u32 {
        (self.button_ as i32 + 1) as u32
    }

    // Source: mouse_event.h:126-127; mouse_event.cc:364-382
    pub fn toElement(&self) -> Option<EventTargetRef> {
        let target = if matches!(self.r#type(), "mouseout" | "mouseleave") {
            self.relatedTarget()
        } else {
            self.RawTarget()
        };
        target.filter(|target| target.Kind() == EventTargetKind::Node)
    }
    pub fn fromElement(&self) -> Option<EventTargetRef> {
        let target = if !matches!(self.r#type(), "mouseout" | "mouseleave") {
            self.relatedTarget()
        } else {
            self.RawTarget()
        };
        target.filter(|target| target.Kind() == EventTargetKind::Node)
    }
    // Source: mouse_event.h:130
    pub fn dataTransfer(&self) -> Option<Rc<dyn MouseDataTransfer>> {
        None
    }
    // Source: mouse_event.h:149-181,192
    pub fn screenX(&self) -> f64 {
        self.screen_x_.floor()
    }
    pub fn screenY(&self) -> f64 {
        self.screen_y_.floor()
    }
    pub fn clientX(&self) -> f64 {
        self.client_x_.floor()
    }
    pub fn clientY(&self) -> f64 {
        self.client_y_.floor()
    }
    pub fn movementX(&self) -> i32 {
        self.movement_delta_.0
    }
    pub fn movementY(&self) -> i32 {
        self.movement_delta_.1
    }
    pub fn pageX(&self) -> f64 {
        self.page_x_.floor()
    }
    pub fn pageY(&self) -> f64 {
        self.page_y_.floor()
    }
    // Source: mouse_event.h:149-164. Derived translated PointerEvent keeps the
    // source protected fractional coordinates instead of the rounded Web API
    // accessors above.
    pub(crate) fn FractionalScreenX(&self) -> f64 {
        self.screen_x_
    }
    pub(crate) fn FractionalScreenY(&self) -> f64 {
        self.screen_y_
    }
    pub(crate) fn FractionalClientX(&self) -> f64 {
        self.client_x_
    }
    pub(crate) fn FractionalClientY(&self) -> f64 {
        self.client_y_
    }
    pub(crate) fn FractionalPageX(&self) -> f64 {
        self.page_x_
    }
    pub(crate) fn FractionalPageY(&self) -> f64 {
        self.page_y_
    }
    pub(crate) fn FractionalOffsetX(&self) -> f64 {
        if !self.relative_position_.get().cached {
            self.ComputeRelativePosition();
        }
        self.relative_position_.get().offset_x
    }
    pub(crate) fn FractionalOffsetY(&self) -> f64 {
        if !self.relative_position_.get().cached {
            self.ComputeRelativePosition();
        }
        self.relative_position_.get().offset_y
    }
    pub fn x(&self) -> f64 {
        self.clientX()
    }
    pub fn y(&self) -> f64 {
        self.clientY()
    }
    pub fn HasPosition(&self) -> bool {
        self.position_type_ == PositionType::kPosition
    }
    pub fn GetMenuSourceType(&self) -> MenuSourceType {
        self.menu_source_type_
    }
    pub fn AbsoluteLocation(&self) -> &PointF {
        &self.absolute_location_
    }
    pub fn RawButton(&self) -> i16 {
        self.button_
    }

    // Source: mouse_event.h:189; mouse_event.cc:384-387
    // The base and related-target Rc owners retain the original GC identities.

    // Source: mouse_event.h:194; mouse_event.cc:453-455
    pub fn ReceivedTarget(&mut self) {
        let mut relative = self.relative_position_.get();
        relative.cached = false;
        self.relative_position_.set(relative);
    }

    // Source: mouse_event.h:196; mouse_event.cc:457-526
    pub fn ComputeRelativePosition(&self) {
        let Some(target) = self
            .RawTarget()
            .filter(|target| target.Kind() == EventTargetKind::Node)
        else {
            return;
        };
        let mut relative = RelativePosition {
            offset_x: self.page_x_,
            offset_y: self.page_y_,
            layer_x: self.page_x_ as f32,
            layer_y: self.page_y_ as f32,
            cached: false,
        };
        let Some(geometry) = &self.coordinate_context_.relative else {
            return;
        };
        let view = self.view();
        let view = view
            .filter(|view| {
                self.coordinate_context_
                    .views
                    .as_ref()
                    .is_some_and(|geometry| geometry.IsLocalDOMWindow(view))
            })
            .or_else(|| geometry.DocumentView(&target));
        let zoom = LayoutZoomFactor(view.as_ref(), &self.coordinate_context_);
        let inverse_zoom = 1.0 / zoom;
        // Source cc:475-476 is an embedding precondition on this live query.
        let mut target = Some(target);
        if let Some(layout) = FindTargetLayoutObject(&mut target, geometry.as_ref()) {
            let local = layout.AbsoluteToLocalPoint(self.absolute_location_);
            // TODO: mouse_event.cc:483-487 UseCounter for inline layout.
            let (border_x, border_y) = layout.BorderOffset().unwrap_or((0.0, 0.0));
            relative.offset_x = ((local.x() - border_x) * inverse_zoom) as f64;
            relative.offset_y = ((local.y() - border_y) * inverse_zoom) as f64;
        }
        let mut node = target;
        let layer_layout = loop {
            let Some(current) = node else {
                break None;
            };
            if let Some(layout) = geometry.Layout(&current) {
                break Some(layout);
            }
            node = geometry.ParentNode(&current);
        };
        if let Some(layout) = layer_layout {
            let mut point = PointF::new(relative.layer_x * zoom, relative.layer_y * zoom);
            point = layout.DocumentToFrame(point).unwrap_or(point);
            let (x, y) = layout.EnclosingSelfPaintingLayerAbsoluteOrigin();
            relative.layer_x = (point.x() - x) * inverse_zoom;
            relative.layer_y = (point.y() - y) * inverse_zoom;
        }
        relative.cached = true;
        self.relative_position_.set(relative);
    }

    // Source: mouse_event.h:226-227; mouse_event.cc:528-546
    // TODO: UseCounter instrumentation for media/canvas/frame/SVG targets.
    pub fn RecordLayerXYMetrics(&self) {}

    // Source: mouse_event.h:160-164; mouse_event.cc:548-580
    pub fn layerX(&self) -> i32 {
        if !self.relative_position_.get().cached {
            self.ComputeRelativePosition();
        }
        self.RecordLayerXYMetrics();
        self.relative_position_.get().layer_x.floor() as i32
    }
    pub fn layerY(&self) -> i32 {
        if !self.relative_position_.get().cached {
            self.ComputeRelativePosition();
        }
        self.RecordLayerXYMetrics();
        self.relative_position_.get().layer_y.floor() as i32
    }
    pub fn offsetX(&self) -> f64 {
        if !self.HasPosition() {
            return 0.0;
        }
        if !self.relative_position_.get().cached {
            self.ComputeRelativePosition();
        }
        self.relative_position_.get().offset_x.round()
    }
    pub fn offsetY(&self) -> f64 {
        if !self.HasPosition() {
            return 0.0;
        }
        if !self.relative_position_.get().cached {
            self.ComputeRelativePosition();
        }
        self.relative_position_.get().offset_y.round()
    }
}

pub trait MouseDataTransfer {}

impl Deref for MouseEvent {
    type Target = UIEventWithKeyState;
    fn deref(&self) -> &Self::Target {
        &self.key_state
    }
}
impl DerefMut for MouseEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.key_state
    }
}
impl EventInterface for MouseEvent {
    fn PrepareDispatchHook<'a>(
        &self,
        nested: &'a dyn crate::core::events::mouse_event::MouseNestedDispatch,
    ) -> Box<dyn crate::core::dom::events::event_dispatcher::EventDispatchHook + 'a> {
        Box::new(MouseEventDispatchHook {
            mouse: self.DispatchValues(),
            nested,
        })
    }
    fn AsMouseEvent(&self) -> Option<&MouseEvent> {
        Some(self)
    }
    fn AsUIEventWithKeyState(&self) -> Option<&UIEventWithKeyState> {
        Some(&self.key_state)
    }
    fn BaseEvent(&self) -> &Event {
        self.key_state.BaseEvent()
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        self.key_state.BaseEventMut()
    }
    fn InterfaceName(&self) -> &str {
        "MouseEvent"
    }
    fn IsUIEvent(&self) -> bool {
        true
    }
    fn IsMouseEvent(&self) -> bool {
        true
    }
    fn relatedTarget(&self) -> Option<EventTargetRef> {
        self.related_target_.clone()
    }
    fn SetRelatedTarget(&mut self, target: Option<EventTargetRef>) {
        self.related_target_ = target;
    }
    fn ReceivedTarget(&mut self) {
        MouseEvent::ReceivedTarget(self);
    }
}
impl UIEventInterface for MouseEvent {
    fn BaseUIEvent(&self) -> &UIEvent {
        self.key_state.BaseUIEvent()
    }
    fn BaseUIEventMut(&mut self) -> &mut UIEvent {
        self.key_state.BaseUIEventMut()
    }
    fn which(&self) -> u32 {
        MouseEvent::which(self)
    }
}
impl UIEventWithKeyStateInterface for MouseEvent {
    fn BaseUIEventWithKeyState(&self) -> &UIEventWithKeyState {
        &self.key_state
    }
    fn BaseUIEventWithKeyStateMut(&mut self) -> &mut UIEventWithKeyState {
        &mut self.key_state
    }
}

// Source: mouse_event.h:111,126-130,151-167
// Derived pointer/wheel events keep the same MouseEvent owner and override
// only the source virtual projections; no coordinate or target state is copied.
pub trait MouseEventInterface: UIEventWithKeyStateInterface {
    fn BaseMouseEvent(&self) -> &MouseEvent;
    fn BaseMouseEventMut(&mut self) -> &mut MouseEvent;
    fn button(&self) -> i16 {
        self.BaseMouseEvent().button()
    }
    fn toElement(&self) -> Option<EventTargetRef> {
        self.BaseMouseEvent().toElement()
    }
    fn fromElement(&self) -> Option<EventTargetRef> {
        self.BaseMouseEvent().fromElement()
    }
    fn dataTransfer(&self) -> Option<Rc<dyn MouseDataTransfer>> {
        None
    }
    fn screenX(&self) -> f64 {
        self.BaseMouseEvent().screenX()
    }
    fn screenY(&self) -> f64 {
        self.BaseMouseEvent().screenY()
    }
    fn clientX(&self) -> f64 {
        self.BaseMouseEvent().clientX()
    }
    fn clientY(&self) -> f64 {
        self.BaseMouseEvent().clientY()
    }
    fn offsetX(&self) -> f64 {
        self.BaseMouseEvent().offsetX()
    }
    fn offsetY(&self) -> f64 {
        self.BaseMouseEvent().offsetY()
    }
    fn pageX(&self) -> f64 {
        self.BaseMouseEvent().pageX()
    }
    fn pageY(&self) -> f64 {
        self.BaseMouseEvent().pageY()
    }
}
impl MouseEventInterface for MouseEvent {
    fn BaseMouseEvent(&self) -> &MouseEvent {
        self
    }
    fn BaseMouseEventMut(&mut self) -> &mut MouseEvent {
        self
    }
}

// Source: mouse_event.cc:435-447
// Nested dispatch routes the completed translated MouseEvent into the same
// dispatcher assembly. It does not decide the double-click algorithm.
pub trait MouseNestedDispatch {
    fn DispatchEvent(&self, node: EventTargetRef, event: MouseEvent) -> DispatchEventResult;
    fn Now(&self) -> TimeTicks;
}

// Source: mouse_event.h:183; mouse_event.cc:389-451
// This per-dispatch value contains only readonly mouse values required after
// listener invocation. The original EventRef is borrowed only in short blocks.
pub struct MouseEventDispatchHook<'a> {
    pub mouse: MouseDispatchValues,
    pub nested: &'a dyn MouseNestedDispatch,
}

pub struct MouseDispatchValues {
    detail: i32,
    view: Option<AbstractViewRef>,
    screen_x: f64,
    screen_y: f64,
    client_x: f64,
    client_y: f64,
    modifiers: i32,
    button: i16,
    buttons: u16,
    capabilities: Option<InputDeviceCapabilitiesRef>,
    coordinates: MouseCoordinateContext,
}
impl MouseEvent {
    pub fn DispatchValues(&self) -> MouseDispatchValues {
        MouseDispatchValues {
            detail: self.detail(),
            view: self.view(),
            screen_x: self.screenX(),
            screen_y: self.screenY(),
            client_x: self.clientX(),
            client_y: self.clientY(),
            modifiers: self.GetModifiers(),
            button: self.button(),
            buttons: self.buttons(),
            capabilities: self.sourceCapabilities(),
            coordinates: self.coordinate_context_.clone(),
        }
    }
}
impl EventDispatchHook for MouseEventDispatchHook<'_> {
    fn DispatchEvent(&self, dispatcher: &mut EventDispatcher<'_>) -> DispatchEventResult {
        let event = dispatcher.GetEvent();
        let path = dispatcher.GetEventPath();
        let related = event.borrow().relatedTarget();
        path.borrow_mut()
            .AdjustForRelatedTarget(&dispatcher.GetNode(), related);
        let is_click = event.borrow().BaseEvent().r#type() == "click";
        if !event.borrow().BaseEvent().isTrusted() {
            return dispatcher.Dispatch();
        }
        if is_click
            || matches!(
                event.borrow().BaseEvent().r#type(),
                "mousedown" | "mouseup" | "dblclick"
            )
        {
            path.borrow_mut().AdjustForDisabledFormControl();
        }
        if event.borrow().BaseEvent().r#type().is_empty() {
            return DispatchEventResult::kNotCanceled;
        }
        // TODO: mouse_event.cc:407-420 disabled-control parent UseCounter.
        {
            let state = event.borrow();
            if let (Some(target), Some(related)) =
                (state.BaseEvent().RawTarget(), state.relatedTarget())
            {
                debug_assert!(!Rc::ptr_eq(&target, &related));
            }
        }
        let related = event.borrow().relatedTarget();
        let result = dispatcher.Dispatch();
        if !is_click || self.mouse.detail != 2 {
            return result;
        }
        let mut double_click = MouseEvent::Uninitialized(self.nested.Now());
        double_click.coordinate_context_ = self.mouse.coordinates.clone();
        let (bubbles, cancelable, composed, trusted, handled) = {
            let event = event.borrow();
            let base = event.BaseEvent();
            (
                base.bubbles(),
                base.cancelable(),
                base.composed(),
                base.isTrusted(),
                base.DefaultHandled(),
            )
        };
        double_click.InitMouseEventInternal(
            "dblclick",
            bubbles,
            cancelable,
            self.mouse.view.clone(),
            self.mouse.detail,
            self.mouse.screen_x,
            self.mouse.screen_y,
            self.mouse.client_x,
            self.mouse.client_y,
            self.mouse.modifiers,
            self.mouse.button,
            related,
            self.mouse.capabilities.clone(),
            self.mouse.buttons,
        );
        double_click.SetComposed(composed);
        double_click.SetTrusted(trusted);
        if handled {
            double_click.SetDefaultHandled();
        }
        let double_result = self
            .nested
            .DispatchEvent(dispatcher.GetNode(), double_click);
        if double_result != DispatchEventResult::kNotCanceled {
            double_result
        } else {
            result
        }
    }
}
