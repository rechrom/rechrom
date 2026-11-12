// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/pointer_event_factory.h:1-280
// Source: third_party/blink/renderer/core/events/pointer_event_factory.cc:1-881
// Rust modules replace includes/namespaces; Rc/Weak preserve Member/WeakMember
// ownership. Methods below follow the .cc source order.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::pointer_id::PointerId;
use interaction_input::public::common::input::web_input_event::Type;
use interaction_input::public::common::input::web_pointer_event::WebPointerEvent;
use interaction_input::public::common::input::web_pointer_properties::{
    Button, Buttons, PointF, PointerType, WebPointerProperties,
};

use super::mouse_event::{MenuSourceType, MouseCoordinateContext, MouseEvent, SyntheticEventType};
use super::pointer_event::{PointerEvent, PointerEventRef};
use super::pointer_event_init::PointerEventInit;
use super::pointer_event_util::PointerEventUtil;
use super::ui_event::AbstractViewRef;
use super::ui_event_with_key_state::UIEventWithKeyState;
use crate::core::dom::events::event::{EventTarget, EventTargetRef};

// Source: pointer_event_factory.h:116-124
pub struct PointerTarget {
    pub node: Option<Weak<dyn EventTarget>>,
    pub client_x: f64,
    pub client_y: f64,
}
pub type PointerTargetRef = Rc<RefCell<PointerTarget>>;
impl PointerTarget {
    pub fn new(node: Option<&EventTargetRef>, client_x: f64, client_y: f64) -> Self {
        Self {
            node: node.map(Rc::downgrade),
            client_x,
            client_y,
        }
    }
}

// Source: pointer_event_factory.cc:819-840
// The DOM owner supplies only the two tree queries used by this operation.
pub trait PointerPseudoElement {
    fn ParentOrShadowHostElement(&self) -> Option<EventTargetRef>;
    fn IsShadowIncludingInclusiveAncestorOf(&self, target: &EventTargetRef) -> bool;
}

// Source: pointer_event_factory.h:141-150
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct IncomingId(i32, i32);
impl IncomingId {
    fn new(pointer_type: PointerType, raw_id: i32) -> Self {
        Self(pointer_type as i32, raw_id)
    }
    fn PointerTypeInt(self) -> usize {
        self.0 as usize
    }
    fn GetPointerType(self) -> PointerType {
        match self.0 {
            0 => PointerType::kUnknown,
            1 => PointerType::kMouse,
            2 => PointerType::kPen,
            3 => PointerType::kTouch,
            4 => PointerType::kEraser,
            _ => unreachable!("invalid pointer type"),
        }
    }
    fn RawId(self) -> i32 {
        self.1
    }
}

// Source: pointer_event_factory.h:194-219
struct PointerAttributes {
    incoming_id: IncomingId,
    is_active_buttons: bool,
    hovering: bool,
    unique_touch_event_id: u32,
    last_position: Option<PointF>,
    last_rawupdate_position: Option<PointF>,
    pointer_down_target: Option<PointerTargetRef>,
    pointer_up_target: Option<PointerTargetRef>,
}
impl Default for PointerAttributes {
    fn default() -> Self {
        Self {
            incoming_id: IncomingId::new(PointerType::kUnknown, 0),
            is_active_buttons: false,
            hovering: true,
            unique_touch_event_id: 0,
            last_position: None,
            last_rawupdate_position: None,
            pointer_down_target: None,
            pointer_up_target: None,
        }
    }
}

// Source: pointer_event_factory.h:233-242
struct PointerIdAndAttributes {
    id: PointerId,
    attributes: PointerAttributes,
}

// Source: pointer_event_factory.h:23-33,184-192,226-275
pub struct PointerEventFactory {
    current_id_: PointerId,
    pointer_incoming_id_mapping_: HashMap<IncomingId, PointerId>,
    pointer_id_to_attributes_: HashMap<i64, PointerAttributes>,
    recently_removed_pointers_: Vec<PointerIdAndAttributes>,
    primary_id_: [PointerId; Self::kIdArraySize],
    id_count_: [i32; Self::kIdArraySize],
    device_id_browser_to_blink_mapping_: HashMap<i32, i32>,
    current_device_id_: i32,
    device_id_for_mouse_: i32,
    // Existing MouseEvent geometry port replaces LocalDOMWindow frame access.
    coordinates: MouseCoordinateContext,
}

// Source: pointer_event_factory.cc:24-26
fn ToInt(pointer_type: PointerType) -> usize {
    pointer_type as usize
}

// Source: pointer_event_factory.cc:28-46
fn ButtonToButtonsBitfield(button: Button) -> u16 {
    match button {
        Button::kNoButton => Buttons::kNoButton as u16,
        Button::kLeft => Buttons::kLeft as u16,
        Button::kRight => Buttons::kRight as u16,
        Button::kMiddle => Buttons::kMiddle as u16,
        Button::kBack => Buttons::kBack as u16,
        Button::kForward => Buttons::kForward as u16,
        Button::kEraser => Buttons::kEraser as u16,
    }
}

// Source: pointer_event_factory.cc:48-63
fn PointerEventNameForEventType(event_type: Type) -> &'static str {
    match event_type {
        Type::kPointerDown => "pointerdown",
        Type::kPointerUp => "pointerup",
        Type::kPointerMove => "pointermove",
        Type::kPointerRawUpdate => "pointerrawupdate",
        Type::kPointerCancel => "pointercancel",
        _ => unreachable!("invalid pointer event type"),
    }
}

// Source: pointer_event_factory.cc:65-73
fn GetPointerEventPressure(force: f32, buttons: u16) -> f32 {
    if buttons == 0 {
        return 0.0;
    }
    if force.is_nan() {
        return 0.5;
    }
    force
}

// Source: pointer_event_factory.cc:75-138
fn UpdateCommonPointerEventInit(
    web_pointer_event: &WebPointerEvent,
    last_global_position: PointF,
    dom_window: Option<&AbstractViewRef>,
    pointer_event_init: &mut PointerEventInit,
    coordinates: &MouseCoordinateContext,
) {
    // Shared identity/type/primary fields deliberately remain unchanged.
    let in_root = web_pointer_event.WebPointerEventInRootFrame();
    let properties = &web_pointer_event.pointer_properties;
    MouseEvent::SetCoordinatesFromWebPointerProperties(
        &in_root.pointer_properties,
        dom_window,
        coordinates.views.as_deref(),
        &mut pointer_event_init.mouse_event_init,
    );
    if !properties.is_raw_movement_event
        && matches!(
            web_pointer_event.GetType(),
            Type::kPointerMove | Type::kPointerRawUpdate
        )
    {
        // Rust float-to-int casts saturate and truncate like saturated_cast<int>.
        let device_scale_factor = 1.0;
        pointer_event_init.mouse_event_init.movement_x =
            ((properties.PositionInScreen().x() * device_scale_factor) as i32)
                .wrapping_sub((last_global_position.x() * device_scale_factor) as i32);
        pointer_event_init.mouse_event_init.movement_y =
            ((properties.PositionInScreen().y() * device_scale_factor) as i32)
                .wrapping_sub((last_global_position.y() * device_scale_factor) as i32);
    }
    // Unknown shape and pointerup retain the dictionary defaults of 1.
    if in_root.HasWidth() && in_root.HasHeight() && web_pointer_event.GetType() != Type::kPointerUp
    {
        let scale_factor = dom_window
            .and_then(|view| coordinates.views.as_ref()?.LayoutZoomFactor(view))
            .map_or(1.0, |zoom| 1.0 / zoom);
        pointer_event_init.width = Some((in_root.width * scale_factor) as f64);
        pointer_event_init.height = Some((in_root.height * scale_factor) as f64);
    }
    pointer_event_init.pressure = Some(GetPointerEventPressure(
        properties.force,
        pointer_event_init.mouse_event_init.buttons,
    ));
    pointer_event_init.tilt_x = Some(properties.tilt_x.round() as i32);
    pointer_event_init.tilt_y = Some(properties.tilt_y.round() as i32);
    pointer_event_init.altitude_angle = Some(PointerEventUtil::AltitudeFromTilt(
        properties.tilt_x,
        properties.tilt_y,
    ));
    pointer_event_init.azimuth_angle = Some(PointerEventUtil::AzimuthFromTilt(
        properties.tilt_x,
        properties.tilt_y,
    ));
    pointer_event_init.tangential_pressure = Some(properties.tangential_pressure);
    pointer_event_init.twist = Some(properties.twist);
}

// Source: pointer_event_factory.cc:142-160. Mouse synthesis shares this source
// owner rather than retaining another pointer-type spelling table.
pub fn PointerTypeName(pointer_type: PointerType) -> &'static str {
    match pointer_type {
        PointerType::kUnknown => "",
        PointerType::kMouse => "mouse",
        PointerType::kPen => "pen",
        PointerType::kTouch => "touch",
        // Chromium maps hovering erasers to pen until the spec supports them.
        PointerType::kEraser => "pen",
    }
}

impl PointerEventFactory {
    // Source: pointer_event_factory.h:35-37
    pub fn PointerTypeNameForWebPointPointerType(pointer_type: PointerType) -> &'static str {
        PointerTypeName(pointer_type)
    }

    // Source: pointer_event_factory.h:171-175; pointer_event_factory.cc:162-229
    fn CreateEventSequence(
        &self,
        web_pointer_event: &WebPointerEvent,
        pointer_event_init: &PointerEventInit,
        event_list: &[WebPointerEvent],
        view: Option<&AbstractViewRef>,
    ) -> Vec<PointerEventRef> {
        let event_type = PointerEventNameForEventType(web_pointer_event.GetType());
        let mut result = Vec::new();
        if let Some(first) = event_list.first() {
            let mut last_global_position = self.GetLastPointerPosition(
                pointer_event_init.pointer_id.unwrap_or(0),
                &first.pointer_properties,
                web_pointer_event.GetType(),
            );
            for event in event_list {
                debug_assert_eq!(
                    web_pointer_event.pointer_properties.id,
                    event.pointer_properties.id
                );
                debug_assert_eq!(web_pointer_event.GetType(), event.GetType());
                debug_assert_eq!(
                    web_pointer_event.pointer_properties.pointer_type,
                    event.pointer_properties.pointer_type
                );
                let mut init = PointerEventInit::default();
                // Generated dictionary defaults make button/buttons/view present.
                init.mouse_event_init.button = pointer_event_init.mouse_event_init.button;
                init.mouse_event_init.buttons = pointer_event_init.mouse_event_init.buttons;
                init.is_primary = pointer_event_init.is_primary;
                init.pointer_id = pointer_event_init.pointer_id;
                init.pointer_type = pointer_event_init.pointer_type.clone();
                init.mouse_event_init.event_modifier_init.ui_event_init.view = pointer_event_init
                    .mouse_event_init
                    .event_modifier_init
                    .ui_event_init
                    .view
                    .clone();
                init.mouse_event_init
                    .event_modifier_init
                    .ui_event_init
                    .event_init
                    .cancelable = false;
                init.mouse_event_init
                    .event_modifier_init
                    .ui_event_init
                    .event_init
                    .bubbles = false;
                UpdateCommonPointerEventInit(
                    event,
                    last_global_position,
                    view,
                    &mut init,
                    &self.coordinates,
                );
                UIEventWithKeyState::SetFromWebInputEventModifiers(
                    &mut init.mouse_event_init.event_modifier_init,
                    event.GetModifiers(),
                );
                last_global_position = *event.pointer_properties.PositionInScreen();
                init.persistent_device_id = pointer_event_init.persistent_device_id;
                let mut pointer_event = PointerEvent::new(
                    event_type,
                    &init,
                    event.TimeStamp(),
                    SyntheticEventType::kRealOrIndistinguishable,
                    MenuSourceType::kNone,
                    false,
                    self.coordinates.clone(),
                );
                pointer_event.SetTrusted(true);
                result.push(Rc::new(RefCell::new(pointer_event)));
            }
        }
        result
    }

    // Source: pointer_event_factory.h:98-100; pointer_event_factory.cc:231-235
    pub const kReservedNonPointerId: PointerId = -1;
    pub const kInvalidId: PointerId = 0;
    pub const kMouseId: PointerId = 1;
    // Source: pointer_event_factory.h:252,259-260
    const kRemovedPointersCapacity: usize = 10;
    const kIdArraySize: usize = PointerType::kMaxValue as usize + 1;

    // Source: pointer_event_factory.h:159-161; pointer_event_factory.cc:237-285
    fn ConvertIdTypeButtonsEvent(&mut self, event: &WebPointerEvent) -> Option<PointerEventInit> {
        let mut pointer_type = event.pointer_properties.pointer_type;
        let mut buttons = if event.hovering {
            MouseEvent::WebInputEventModifiersToButtons(event.GetModifiers() as u32)
        } else if matches!(event.GetType(), Type::kPointerUp | Type::kPointerCancel) {
            Buttons::kNoButton as u16
        } else {
            // Source's non-hovering assumption is retained for styluses too.
            Buttons::kLeft as u16
        };
        if pointer_type == PointerType::kEraser {
            if buttons != 0 {
                buttons |= Buttons::kEraser as u16;
                buttons &= !(Buttons::kLeft as u16);
            }
            pointer_type = PointerType::kPen;
        }
        let incoming_id = IncomingId::new(pointer_type, event.pointer_properties.id);
        let pointer_id = self.AddOrUpdateIdAndActiveButtons(
            incoming_id,
            buttons != 0,
            event.hovering,
            event.GetType(),
            event.unique_touch_event_id,
        );
        if pointer_id == Self::kInvalidId {
            return None;
        }
        let mut init = PointerEventInit::default();
        init.mouse_event_init.buttons = buttons;
        init.pointer_id = Some(pointer_id);
        init.pointer_type = Some(Self::PointerTypeNameForWebPointPointerType(pointer_type).into());
        init.is_primary = Some(self.IsPrimaryId(pointer_id));
        Some(init)
    }

    // Source: pointer_event_factory.h:163; pointer_event_factory.cc:287-302
    fn SetEventSpecificFields(&self, init: &mut PointerEventInit, event_type: &str) {
        let enter_or_leave = matches!(event_type, "pointerenter" | "pointerleave");
        let ui_init = &mut init.mouse_event_init.event_modifier_init.ui_event_init;
        ui_init.event_init.bubbles = !enter_or_leave;
        ui_init.event_init.cancelable = !matches!(
            event_type,
            "pointerenter"
                | "pointerleave"
                | "pointercancel"
                | "pointerrawupdate"
                | "gotpointercapture"
                | "lostpointercapture"
        );
        ui_init.event_init.composed = !enter_or_leave;
        ui_init.detail = 0;
    }

    // Source: pointer_event_factory.h:41-46; pointer_event_factory.cc:304-385
    pub fn Create(
        &mut self,
        event: &WebPointerEvent,
        coalesced_events: &[WebPointerEvent],
        predicted_events: &[WebPointerEvent],
        view: Option<AbstractViewRef>,
    ) -> Option<PointerEventRef> {
        let event_type = event.GetType();
        debug_assert!(matches!(
            event_type,
            Type::kPointerDown
                | Type::kPointerUp
                | Type::kPointerMove
                | Type::kPointerRawUpdate
                | Type::kPointerCancel
        ));
        let mut init = self.ConvertIdTypeButtonsEvent(event)?;
        let mut name = PointerEventNameForEventType(event_type);
        if matches!(event_type, Type::kPointerDown | Type::kPointerUp) {
            let mut button = event.pointer_properties.button;
            if event.pointer_properties.pointer_type == PointerType::kEraser
                && button == Button::kLeft
            {
                button = Button::kEraser;
            }
            init.mouse_event_init.button = button as i16;
            if (event_type == Type::kPointerDown
                && (init.mouse_event_init.buttons & !ButtonToButtonsBitfield(button)) != 0)
                || (event_type == Type::kPointerUp && init.mouse_event_init.buttons != 0)
            {
                name = "pointermove";
            }
        } else {
            init.mouse_event_init.button = Button::kNoButton as i16;
        }
        init.mouse_event_init.event_modifier_init.ui_event_init.view = view.clone();
        UpdateCommonPointerEventInit(
            event,
            self.GetLastPointerPosition(
                init.pointer_id.unwrap(),
                &event.pointer_properties,
                event_type,
            ),
            view.as_ref(),
            &mut init,
            &self.coordinates,
        );
        UIEventWithKeyState::SetFromWebInputEventModifiers(
            &mut init.mouse_event_init.event_modifier_init,
            event.GetModifiers(),
        );
        self.SetEventSpecificFields(&mut init, name);
        let mut coalesced = Vec::new();
        let mut predicted = Vec::new();
        if matches!(name, "pointermove" | "pointerrawupdate") {
            coalesced = self.CreateEventSequence(event, &init, coalesced_events, view.as_ref());
        }
        if name == "pointermove" {
            predicted = self.CreateEventSequence(event, &init, predicted_events, view.as_ref());
        }
        init.coalesced_events = Some(coalesced);
        init.predicted_events = Some(predicted);
        self.SetLastPosition(
            init.pointer_id.unwrap(),
            *event.pointer_properties.PositionInScreen(),
            event_type,
        );
        // This assignment intentionally follows sequence creation, as in Chromium.
        init.persistent_device_id = Some(self.GetBlinkDeviceId(event));
        Some(Rc::new(RefCell::new(PointerEvent::new(
            name,
            &init,
            event.TimeStamp(),
            SyntheticEventType::kRealOrIndistinguishable,
            MenuSourceType::kNone,
            event.GetPreventCountingAsInteraction(),
            self.coordinates.clone(),
        ))))
    }

    // Source: pointer_event_factory.h:112-114; pointer_event_factory.cc:387-401
    pub fn SetLastPosition(&mut self, pointer_id: PointerId, position: PointF, event_type: Type) {
        let attributes = self
            .pointer_id_to_attributes_
            .entry(pointer_id as i64)
            .or_default();
        if event_type == Type::kPointerRawUpdate {
            attributes.last_rawupdate_position = Some(position);
        } else {
            attributes.last_position = Some(position);
        }
    }

    // Source: pointer_event_factory.h:102-103; pointer_event_factory.cc:403-407
    pub fn RemoveLastPosition(&mut self, pointer_id: PointerId) {
        let attributes = self
            .pointer_id_to_attributes_
            .get_mut(&(pointer_id as i64))
            .expect("pointer id must exist");
        attributes.last_position = None;
        attributes.last_rawupdate_position = None;
    }

    // Source: pointer_event_factory.h:105-110; pointer_event_factory.cc:409-427
    pub fn GetLastPointerPosition(
        &self,
        pointer_id: PointerId,
        event: &WebPointerProperties,
        event_type: Type,
    ) -> PointF {
        if let Some(attributes) = self.pointer_id_to_attributes_.get(&(pointer_id as i64)) {
            let position = if event_type == Type::kPointerRawUpdate {
                attributes.last_rawupdate_position
            } else {
                attributes.last_position
            };
            if let Some(position) = position {
                return position;
            }
        }
        *event.PositionInScreen()
    }

    // Source: pointer_event_factory.h:48-50; pointer_event_factory.cc:429-452
    pub fn CreatePointerCancelEvent(
        &mut self,
        pointer_id: PointerId,
        platform_time_stamp: TimeTicks,
        device_id: i32,
    ) -> PointerEventRef {
        let attributes = self
            .pointer_id_to_attributes_
            .get_mut(&(pointer_id as i64))
            .expect("pointercancel requires an existing pointer");
        attributes.is_active_buttons = false;
        attributes.hovering = true;
        let mut init = PointerEventInit::default();
        init.pointer_id = Some(pointer_id);
        init.pointer_type = Some(
            Self::PointerTypeNameForWebPointPointerType(attributes.incoming_id.GetPointerType())
                .into(),
        );
        init.is_primary = Some(self.IsPrimaryId(pointer_id));
        self.SetEventSpecificFields(&mut init, "pointercancel");
        init.persistent_device_id = Some(device_id);
        Rc::new(RefCell::new(PointerEvent::new(
            "pointercancel",
            &init,
            platform_time_stamp,
            SyntheticEventType::kRealOrIndistinguishable,
            MenuSourceType::kNone,
            false,
            self.coordinates.clone(),
        )))
    }

    // Source: pointer_event_factory.h:165-169; pointer_event_factory.cc:454-495
    fn CreatePointerEventFrom(
        &self,
        event: &PointerEvent,
        event_type: &str,
        related_target: Option<EventTargetRef>,
    ) -> PointerEventRef {
        let mut init = PointerEventInit::default();
        init.pointer_id = Some(event.pointerId());
        init.pointer_type = Some(event.pointerType().into());
        init.is_primary = Some(event.isPrimary());
        init.width = Some(event.width());
        init.height = Some(event.height());
        init.mouse_event_init.screen_x = event.screenX();
        init.mouse_event_init.screen_y = event.screenY();
        init.mouse_event_init.client_x = event.clientX();
        init.mouse_event_init.client_y = event.clientY();
        init.mouse_event_init.button = event.button();
        init.mouse_event_init.buttons = event.buttons();
        init.pressure = Some(event.pressure());
        init.tilt_x = Some(event.tiltX());
        init.tilt_y = Some(event.tiltY());
        init.tangential_pressure = Some(event.tangentialPressure());
        init.twist = Some(event.twist());
        init.mouse_event_init.event_modifier_init.ui_event_init.view = event.view();
        init.persistent_device_id = Some(event.persistentDeviceId());
        self.SetEventSpecificFields(&mut init, event_type);
        // Every concrete PointerEvent has the UIEventWithKeyState base.
        UIEventWithKeyState::SetFromWebInputEventModifiers(
            &mut init.mouse_event_init.event_modifier_init,
            event.GetModifiers(),
        );
        if related_target.is_some() {
            init.mouse_event_init.related_target = related_target;
        }
        Rc::new(RefCell::new(PointerEvent::new(
            event_type,
            &init,
            event.PlatformTimeStamp(),
            SyntheticEventType::kRealOrIndistinguishable,
            MenuSourceType::kNone,
            false,
            self.coordinates.clone(),
        )))
    }

    // Source: pointer_event_factory.h:52-53; pointer_event_factory.cc:497-511
    pub fn CreatePointerRawUpdateEvent(&self, event: &PointerEvent) -> PointerEventRef {
        let button = match event.button() {
            -1 => Button::kNoButton,
            0 => Button::kLeft,
            1 => Button::kMiddle,
            2 => Button::kRight,
            3 => Button::kBack,
            4 => Button::kForward,
            5 => Button::kEraser,
            _ => unreachable!("invalid pointer button"),
        };
        debug_assert!(
            event.r#type() == "pointermove"
                && (event.buttons() & !ButtonToButtonsBitfield(button)) != 0
                && event.button() != 0
        );
        self.CreatePointerEventFrom(event, "pointerrawupdate", event.relatedTarget())
    }

    // Source: pointer_event_factory.h:55-56; pointer_event_factory.cc:513-521
    pub fn CreatePointerCaptureEvent(
        &self,
        event: &PointerEvent,
        event_type: &str,
    ) -> PointerEventRef {
        debug_assert!(matches!(
            event_type,
            "gotpointercapture" | "lostpointercapture"
        ));
        self.CreatePointerEventFrom(event, event_type, event.relatedTarget())
    }

    // Source: pointer_event_factory.h:58-61; pointer_event_factory.cc:523-533
    pub fn CreatePointerBoundaryEvent(
        &self,
        event: &PointerEvent,
        event_type: &str,
        related_target: Option<EventTargetRef>,
    ) -> PointerEventRef {
        debug_assert!(matches!(
            event_type,
            "pointerout" | "pointerleave" | "pointerover" | "pointerenter"
        ));
        self.CreatePointerEventFrom(event, event_type, related_target)
    }

    // Source: pointer_event_factory.h:39; pointer_event_factory.cc:535-537
    pub fn new(coordinates: MouseCoordinateContext) -> Self {
        let mut factory = Self {
            current_id_: 0,
            pointer_incoming_id_mapping_: HashMap::new(),
            pointer_id_to_attributes_: HashMap::new(),
            recently_removed_pointers_: Vec::with_capacity(Self::kRemovedPointersCapacity),
            primary_id_: [0; Self::kIdArraySize],
            id_count_: [0; Self::kIdArraySize],
            device_id_browser_to_blink_mapping_: HashMap::new(),
            current_device_id_: 1,
            device_id_for_mouse_: 0,
            coordinates,
        };
        factory.Clear();
        factory
    }

    // Source: pointer_event_factory.h:63-64; pointer_event_factory.cc:539-563
    pub fn Clear(&mut self) {
        for pointer_type in 0..Self::kIdArraySize {
            self.primary_id_[pointer_type] = Self::kInvalidId;
            self.id_count_[pointer_type] = 0;
        }
        self.pointer_incoming_id_mapping_.clear();
        self.pointer_id_to_attributes_.clear();
        self.recently_removed_pointers_.clear();
        self.device_id_browser_to_blink_mapping_.clear();
        self.primary_id_[ToInt(PointerType::kMouse)] = Self::kMouseId;
        self.pointer_id_to_attributes_.insert(
            Self::kMouseId as i64,
            PointerAttributes {
                incoming_id: IncomingId::new(PointerType::kMouse, 0),
                ..PointerAttributes::default()
            },
        );
        self.current_id_ = Self::kMouseId + 1;
        self.current_device_id_ = 1;
        self.device_id_for_mouse_ = 0;
    }

    // Source: pointer_event_factory.h:152-156; pointer_event_factory.cc:565-607
    fn AddOrUpdateIdAndActiveButtons(
        &mut self,
        incoming_id: IncomingId,
        is_active_buttons: bool,
        hovering: bool,
        event_type: Type,
        unique_touch_event_id: u32,
    ) -> PointerId {
        if incoming_id.GetPointerType() == PointerType::kMouse {
            self.pointer_id_to_attributes_
                .get_mut(&(Self::kMouseId as i64))
                .unwrap()
                .is_active_buttons = is_active_buttons;
            return Self::kMouseId;
        }
        if let Some(mapped_id) = self.pointer_incoming_id_mapping_.get(&incoming_id).copied() {
            let attributes = self
                .pointer_id_to_attributes_
                .get_mut(&(mapped_id as i64))
                .unwrap();
            assert_eq!(attributes.incoming_id, incoming_id);
            attributes.is_active_buttons = is_active_buttons;
            attributes.hovering = hovering;
            return mapped_id;
        }
        if event_type == Type::kPointerCancel {
            return Self::kInvalidId;
        }
        let type_int = incoming_id.PointerTypeInt();
        let mapped_id = self.GetNextAvailablePointerid();
        if self.id_count_[type_int] == 0 {
            self.primary_id_[type_int] = mapped_id;
        }
        self.id_count_[type_int] += 1;
        self.pointer_incoming_id_mapping_
            .insert(incoming_id, mapped_id);
        self.pointer_id_to_attributes_.insert(
            mapped_id as i64,
            PointerAttributes {
                incoming_id,
                is_active_buttons,
                hovering,
                unique_touch_event_id,
                last_position: None,
                last_rawupdate_position: None,
                pointer_down_target: None,
                pointer_up_target: None,
            },
        );
        mapped_id
    }

    // Source: pointer_event_factory.h:65-69; pointer_event_factory.cc:609-631
    pub fn Remove(&mut self, mapped_id: PointerId) -> bool {
        if mapped_id == Self::kMouseId {
            return false;
        }
        let Some(attributes) = self.pointer_id_to_attributes_.remove(&(mapped_id as i64)) else {
            return false;
        };
        let incoming_id = attributes.incoming_id;
        let type_int = incoming_id.PointerTypeInt();
        self.pointer_incoming_id_mapping_.remove(&incoming_id);
        if self.primary_id_[type_int] == mapped_id {
            self.primary_id_[type_int] = Self::kInvalidId;
        }
        self.id_count_[type_int] -= 1;
        self.SaveRecentlyRemovedPointer(mapped_id, attributes);
        true
    }

    // Source: pointer_event_factory.h:71-72; pointer_event_factory.cc:633-646
    pub fn GetPointerIdsOfNonHoveringPointers(&self) -> Vec<PointerId> {
        let mut mapped_ids: Vec<_> = self
            .pointer_id_to_attributes_
            .iter()
            .filter_map(|(id, attributes)| (!attributes.hovering).then_some(*id as PointerId))
            .collect();
        mapped_ids.sort_unstable();
        mapped_ids
    }

    // Source: pointer_event_factory.h:157; pointer_event_factory.cc:648-656
    fn IsPrimaryId(&self, mapped_id: PointerId) -> bool {
        self.pointer_id_to_attributes_
            .get(&(mapped_id as i64))
            .is_some_and(|attributes| {
                self.primary_id_[attributes.incoming_id.PointerTypeInt()] == mapped_id
            })
    }

    // Source: pointer_event_factory.h:74-75; pointer_event_factory.cc:658-660
    pub fn IsActive(&self, pointer_id: PointerId) -> bool {
        self.pointer_id_to_attributes_
            .contains_key(&(pointer_id as i64))
    }

    // Source: pointer_event_factory.h:95-96; pointer_event_factory.cc:662-677
    pub fn IsPrimary(&self, properties: &WebPointerProperties) -> bool {
        if properties.pointer_type == PointerType::kMouse {
            return true;
        }
        if self.id_count_[ToInt(properties.pointer_type)] == 0 {
            return true;
        }
        let pointer_id = self.GetPointerEventId(properties);
        pointer_id != Self::kInvalidId && self.IsPrimaryId(pointer_id)
    }

    // Source: pointer_event_factory.h:77-78; pointer_event_factory.cc:679-683
    pub fn IsActiveButtonsState(&self, pointer_id: PointerId) -> bool {
        self.pointer_id_to_attributes_
            .get(&(pointer_id as i64))
            .is_some_and(|attributes| attributes.is_active_buttons)
    }

    // Source: pointer_event_factory.h:91-93; pointer_event_factory.cc:685-691
    pub fn GetPointerType(&self, pointer_id: PointerId) -> PointerType {
        self.pointer_id_to_attributes_
            .get(&(pointer_id as i64))
            .map_or(PointerType::kUnknown, |attributes| {
                attributes.incoming_id.GetPointerType()
            })
    }

    // Source: pointer_event_factory.h:80-82; pointer_event_factory.cc:693-701
    pub fn GetPointerEventId(&self, properties: &WebPointerProperties) -> PointerId {
        if properties.pointer_type == PointerType::kMouse {
            return Self::kMouseId;
        }
        self.pointer_incoming_id_mapping_
            .get(&IncomingId::new(properties.pointer_type, properties.id))
            .copied()
            .unwrap_or(Self::kInvalidId)
    }

    // Source: pointer_event_factory.h:84-89; pointer_event_factory.cc:703-738
    pub fn GetPointerIdForTouchGesture(&mut self, unique_touch_event_id: u32) -> PointerId {
        let matches = |attributes: &PointerAttributes| {
            attributes.incoming_id.GetPointerType() == PointerType::kTouch
                && attributes.unique_touch_event_id == unique_touch_event_id
        };
        for (id, attributes) in &self.pointer_id_to_attributes_ {
            if matches(attributes) {
                return *id as PointerId;
            }
        }
        for removed in &self.recently_removed_pointers_ {
            if matches(&removed.attributes) {
                return removed.id;
            }
        }
        let pointer_id = self.GetNextAvailablePointerid();
        let attributes = PointerAttributes {
            incoming_id: IncomingId::new(PointerType::kTouch, Self::kInvalidId),
            ..PointerAttributes::default()
        };
        // Source does not assign unique_touch_event_id on the reserved entry.
        self.SaveRecentlyRemovedPointer(pointer_id, attributes);
        pointer_id
    }

    // Source: pointer_event_factory.h:221-222; pointer_event_factory.cc:740-749
    fn SaveRecentlyRemovedPointer(&mut self, pointer_id: PointerId, attributes: PointerAttributes) {
        if self.recently_removed_pointers_.len() == Self::kRemovedPointersCapacity {
            self.recently_removed_pointers_.remove(0);
        }
        self.recently_removed_pointers_
            .push(PointerIdAndAttributes {
                id: pointer_id,
                attributes,
            });
    }

    // Source: pointer_event_factory.h:177-180; pointer_event_factory.cc:751-775
    fn GetBlinkDeviceId(&mut self, event: &WebPointerEvent) -> i32 {
        if event.pointer_properties.pointer_type == PointerType::kMouse {
            if self.device_id_for_mouse_ == 0 {
                self.device_id_for_mouse_ = self.current_device_id_;
                self.current_device_id_ = self.current_device_id_.wrapping_add(1);
            }
            return self.device_id_for_mouse_;
        }
        let incoming_id = event.pointer_properties.device_id;
        if incoming_id == -1 {
            return 0;
        }
        let id = self
            .device_id_browser_to_blink_mapping_
            .entry(incoming_id)
            .or_insert_with(|| {
                let id = self.current_device_id_;
                self.current_device_id_ = self.current_device_id_.wrapping_add(1);
                id
            });
        // TODO: pointer_event_factory.cc:772-773 TRACE_EVENT_INSTANT instrumentation.
        *id
    }

    // Source: pointer_event_factory.h:182; pointer_event_factory.cc:777-780
    fn GetNextAvailablePointerid(&mut self) -> PointerId {
        let id = self.current_id_;
        self.current_id_ = self.current_id_.wrapping_add(1);
        id
    }

    // Source: pointer_event_factory.h:126-131; pointer_event_factory.cc:782-792
    pub fn SetPointerDownTarget(
        &mut self,
        pointer_id: PointerId,
        target: Option<PointerTargetRef>,
    ) {
        let attributes = self
            .pointer_id_to_attributes_
            .entry(pointer_id as i64)
            .or_default();
        attributes.pointer_down_target = target;
        attributes.pointer_up_target = None;
    }

    // Source: pointer_event_factory.h:132; pointer_event_factory.cc:794-801
    pub fn SetPointerUpTarget(&mut self, pointer_id: PointerId, target: Option<PointerTargetRef>) {
        self.pointer_id_to_attributes_
            .entry(pointer_id as i64)
            .or_default()
            .pointer_up_target = target;
    }

    // Source: pointer_event_factory.h:133; pointer_event_factory.cc:803-809
    pub fn GetPointerDownTarget(&self, pointer_id: PointerId) -> Option<PointerTargetRef> {
        self.GetPointerAttributesForId(pointer_id)?
            .pointer_down_target
            .clone()
    }

    // Source: pointer_event_factory.h:134; pointer_event_factory.cc:811-817
    pub fn GetPointerUpTarget(&self, pointer_id: PointerId) -> Option<PointerTargetRef> {
        self.GetPointerAttributesForId(pointer_id)?
            .pointer_up_target
            .clone()
    }

    // Source: pointer_event_factory.h:136; pointer_event_factory.cc:819-842
    pub fn HandlePseudoElementRemoved(&mut self, pseudo: &dyn PointerPseudoElement) {
        let Some(originating_element) = pseudo.ParentOrShadowHostElement() else {
            return;
        };
        for attributes in self.pointer_id_to_attributes_.values() {
            for target in [
                &attributes.pointer_down_target,
                &attributes.pointer_up_target,
            ]
            .into_iter()
            .flatten()
            {
                let mut target = target.borrow_mut();
                if let Some(node) = target.node.as_ref().and_then(Weak::upgrade) {
                    if node.IsPseudoElement() && pseudo.IsShadowIncludingInclusiveAncestorOf(&node)
                    {
                        target.node = Some(Rc::downgrade(&originating_element));
                    }
                }
            }
        }
    }

    // Source: pointer_event_factory.h:224; pointer_event_factory.cc:844-860
    fn GetPointerAttributesForId(&self, pointer_id: PointerId) -> Option<&PointerAttributes> {
        if let Some(attributes) = self.pointer_id_to_attributes_.get(&(pointer_id as i64)) {
            return Some(attributes);
        }
        // Touch's click is delivered after Remove; retain the same target owners.
        self.recently_removed_pointers_
            .iter()
            .find(|removed| removed.id == pointer_id)
            .map(|removed| &removed.attributes)
    }

    // Source: pointer_event_factory.h:135; pointer_event_factory.cc:862-869
    pub fn RemovePointerTargets(&mut self, pointer_id: PointerId) {
        let Some(attributes) = self.pointer_id_to_attributes_.get_mut(&(pointer_id as i64)) else {
            return;
        };
        attributes.pointer_down_target = None;
        attributes.pointer_up_target = None;
    }

    // Source: pointer_event_factory.h:123,138,218,241; pointer_event_factory.cc:871-879
    // Rc/Weak and Drop trace/release the corresponding strong and weak owners.
}

impl Default for PointerEventFactory {
    fn default() -> Self {
        Self::new(MouseCoordinateContext::default())
    }
}
