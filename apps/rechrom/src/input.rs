use crate::engine::Command;
use interaction::input_event::*;
use std::time::{Duration, Instant};
use winit::{
    event::{DeviceId, ElementState, Ime, MouseScrollDelta, Touch, TouchPhase, WindowEvent},
    keyboard::{Key, KeyLocation, ModifiersState, NamedKey, PhysicalKey},
};

fn physical_code(key: PhysicalKey) -> String {
    let PhysicalKey::Code(key) = key else {
        return String::new();
    };
    match format!("{key:?}").as_str() {
        "SuperLeft" => "MetaLeft".into(),
        "SuperRight" => "MetaRight".into(),
        code => code.into(),
    }
}

/// A wheel sample before Chromium's phase handler, rails filter and frame
/// aligned queue. Native input and deterministic replay meet at this boundary.
#[derive(Clone, Debug)]
pub struct WheelSample {
    pub delta: MouseScrollDelta,
    pub phase: TouchPhase,
    pub macos_metadata: Option<winit::event::MacOSMouseWheelMetadata>,
    pub position: Offset,
    pub modifiers: EventModifiers,
    pub scale: f64,
    pub observed_at: Instant,
}

fn key_location(location: KeyLocation) -> u32 {
    match location {
        KeyLocation::Standard => 0,
        KeyLocation::Left => 1,
        KeyLocation::Right => 2,
        KeyLocation::Numpad => 3,
    }
}

/// Platform keyboard fields are carried through as source input facts. The
/// character remains separate from keydown, including command/control text.
fn native_keyboard_text(
    event: &winit::event::KeyEvent,
) -> (String, String, i32, bool, Option<String>) {
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    {
        use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
        #[cfg(not(target_os = "macos"))]
        use winit::platform::scancode::PhysicalKeyExtScancode;
        let text = event
            .text
            .as_ref()
            .map(|text| {
                event
                    .text_with_all_modifiers()
                    .unwrap_or(text.as_str())
                    .to_owned()
            })
            .unwrap_or_default();
        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::KeyEventExtMacOS;
            let text = event
                .text_with_all_modifiers()
                .map(str::to_owned)
                .unwrap_or(text);
            return (
                text,
                event.unmodified_text().to_owned(),
                i32::from(event.native_key_code()),
                event.ime_processed(),
                event.committed_text().map(str::to_owned),
            );
        }
        #[cfg(not(target_os = "macos"))]
        {
            let unmodified = event
                .key_without_modifiers()
                .to_text()
                .unwrap_or_default()
                .to_owned();
            let native = event
                .physical_key
                .to_scancode()
                .and_then(|code| i32::try_from(code).ok())
                .unwrap_or_default();
            return (text, unmodified, native, false, None);
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    (
        event
            .text
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        String::new(),
        0,
        false,
        None,
    )
}

fn native_system_key(modifiers: EventModifiers, key_code: u32) -> bool {
    #[cfg(target_os = "macos")]
    {
        modifiers.meta
            && !(modifiers
                == EventModifiers {
                    meta: true,
                    ..Default::default()
                }
                && matches!(key_code, 66 | 73))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = key_code;
        modifiers.alt
    }
}

fn native_windows_key_code(event: &winit::event::KeyEvent, logical_key: &str) -> u32 {
    use winit::keyboard::KeyCode;
    if let PhysicalKey::Code(code) = event.physical_key {
        let keypad = match code {
            KeyCode::Numpad0 => Some(96),
            KeyCode::Numpad1 => Some(97),
            KeyCode::Numpad2 => Some(98),
            KeyCode::Numpad3 => Some(99),
            KeyCode::Numpad4 => Some(100),
            KeyCode::Numpad5 => Some(101),
            KeyCode::Numpad6 => Some(102),
            KeyCode::Numpad7 => Some(103),
            KeyCode::Numpad8 => Some(104),
            KeyCode::Numpad9 => Some(105),
            _ => None,
        };
        if let Some(code) = keypad {
            return code;
        }
    }
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    {
        use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
        let unmodified = LegacyKeyCode(&key_name(&event.key_without_modifiers()));
        if unmodified != 0 {
            return unmodified;
        }
    }
    LegacyKeyCode(logical_key)
}

// Winit's native contact identity is a (device, u64) pair. Chromium's
// MotionEvent/WebTouchPoint ID is a bounded signed ID, retained for the life
// of a contact, independently of the original platform integer's width.
struct NativeTouchContact {
    device: DeviceId,
    native_id: u64,
    point: TouchPoint,
}

#[derive(Default)]
pub struct InputState {
    modifiers: ModifiersState,
    cursor: Offset,
    pressed: Option<(MouseButton, Offset, i32)>,
    buttons: u16,
    last_click: Option<(Instant, MouseButton, Offset, i32)>,
    composing: bool,
    pointer_inside: bool,
    last_motion: Option<(Offset, EventModifiers, Option<MouseButton>)>,
    pending_wheel_end: Option<(Instant, WheelEvent)>,
    explicit_wheel_phase_active: bool,
    phase_less_wheel_active: bool,
    phase_less_wheel_origin: Offset,
    phase_less_wheel_modifiers: EventModifiers,
    wheel_rails_delta: Offset,
    native_wheel_time_anchor: Option<(f64, Instant)>,
    native_touches: Vec<NativeTouchContact>,
    native_touch_devices: Vec<DeviceId>,
    next_touch_event_id: u32,
}
impl InputState {
    #[cfg(target_os = "macos")]
    fn capture_macos_wheel(
        &self,
        metadata: &winit::event::MacOSMouseWheelMetadata,
        delta: &MouseScrollDelta,
        phase: TouchPhase,
        scale: f64,
    ) {
        use std::{
            fs::OpenOptions,
            io::Write,
            sync::{Mutex, OnceLock},
        };

        struct Capture {
            started: Instant,
            file: std::fs::File,
            sequence: u64,
        }
        static CAPTURE: OnceLock<Option<Mutex<Capture>>> = OnceLock::new();
        let capture = CAPTURE.get_or_init(|| {
            let path = std::env::var_os("RECHROM_CAPTURE_NATIVE_WHEEL")?;
            let file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(path)
                .ok()?;
            Some(Mutex::new(Capture {
                started: Instant::now(),
                file,
                sequence: 0,
            }))
        });
        let Some(capture) = capture else { return };
        let Ok(mut capture) = capture.lock() else {
            return;
        };
        capture.sequence += 1;
        let (delta_kind, host_delta_x, host_delta_y) = match delta {
            MouseScrollDelta::LineDelta(x, y) => ("line", f64::from(*x), f64::from(*y)),
            MouseScrollDelta::PixelDelta(point) => ("physical_pixel", point.x, point.y),
        };
        let phase = match phase {
            TouchPhase::Started => "started",
            TouchPhase::Moved => "moved",
            TouchPhase::Ended => "ended",
            TouchPhase::Cancelled => "cancelled",
        };
        let record = serde_json::json!({
            "version": 1,
            "sequence": capture.sequence,
            "delivery_offset_ns": capture.started.elapsed().as_nanos().min(u64::MAX as u128) as u64,
            "nsevent_timestamp_seconds": metadata.timestamp_seconds,
            "phase": phase,
            "native_phase": metadata.phase,
            "momentum_phase": metadata.momentum_phase,
            "is_continuous": metadata.is_continuous,
            "point_delta_x": metadata.point_delta_x,
            "point_delta_y": metadata.point_delta_y,
            "cocoa_delta_x": metadata.cocoa_delta_x,
            "cocoa_delta_y": metadata.cocoa_delta_y,
            "precise": metadata.precise,
            "host_delta_kind": delta_kind,
            "host_delta_x": host_delta_x,
            "host_delta_y": host_delta_y,
            "scale": metadata.scale_factor,
            "consumer_scale": scale,
            "event_position_x": metadata.position_x,
            "event_position_y": metadata.position_y,
            "cursor_x": self.cursor.x,
            "cursor_y": self.cursor.y,
            "modifiers": {
                "alt": self.modifiers.alt_key(),
                "control": self.modifiers.control_key(),
                "meta": self.modifiers.super_key(),
                "shift": self.modifiers.shift_key(),
            },
        });
        let _ = serde_json::to_writer(&mut capture.file, &record);
        let _ = capture.file.write_all(b"\n");
        let _ = capture.file.flush();
    }
    // MouseWheelPhaseHandler::max_time_between_phase_ended_and_momentum_phase_began.
    pub fn pending_wheel_end_deadline(&self) -> Option<Instant> {
        self.pending_wheel_end
            .as_ref()
            .map(|(deadline, _)| *deadline)
    }
    pub fn take_due_wheel_end(&mut self, now: Instant) -> Option<Command> {
        if self
            .pending_wheel_end_deadline()
            .is_some_and(|deadline| now >= deadline)
        {
            self.take_pending_wheel_end_at(now)
        } else {
            None
        }
    }
    fn take_pending_wheel_end(&mut self) -> Option<Command> {
        self.take_pending_wheel_end_at(Instant::now())
    }
    fn take_pending_wheel_end_at(&mut self, timestamp: Instant) -> Option<Command> {
        self.pending_wheel_end.take().map(|(_, mut wheel)| {
            self.phase_less_wheel_active = false;
            // This end is synthesized when the phase-handler deadline fires,
            // so its event time is the firing time rather than the original
            // finger-up sample that armed the timer.
            wheel.timestamp = Some(timestamp);
            Command::Input(InputEvent::Wheel(wheel))
        })
    }
    pub fn pointer_inside(&self) -> bool {
        self.pointer_inside
    }
    pub fn cursor_position(&self) -> Offset {
        self.cursor
    }
    fn modifiers(&self) -> EventModifiers {
        EventModifiers {
            alt: self.modifiers.alt_key(),
            control: self.modifiers.control_key(),
            meta: self.modifiers.super_key(),
            shift: self.modifiers.shift_key(),
        }
    }
    pub fn wheel_sample(
        &mut self,
        delta: MouseScrollDelta,
        phase: TouchPhase,
        macos_metadata: Option<winit::event::MacOSMouseWheelMetadata>,
        scale: f64,
    ) -> WheelSample {
        #[cfg(target_os = "macos")]
        if let Some(metadata) = macos_metadata.as_ref() {
            self.capture_macos_wheel(metadata, &delta, phase, scale);
        }
        let position = macos_metadata.as_ref().map_or(self.cursor, |event| Offset {
            x: event.position_x,
            y: event.position_y,
        });
        let observed_at = macos_metadata
            .as_ref()
            .map_or_else(Instant::now, |metadata| {
                let (native, local) = *self
                    .native_wheel_time_anchor
                    .get_or_insert((metadata.timestamp_seconds, Instant::now()));
                let delta = metadata.timestamp_seconds - native;
                if delta >= 0.0 {
                    local + Duration::from_secs_f64(delta)
                } else {
                    local - Duration::from_secs_f64(-delta)
                }
            });
        WheelSample {
            delta,
            phase,
            macos_metadata,
            position,
            modifiers: self.modifiers(),
            scale,
            observed_at,
        }
    }

    pub fn accept_wheel_sample(&mut self, sample: WheelSample, now: Instant) -> Vec<Command> {
        let mut result = Vec::new();
        if let Some(end) = self.take_due_wheel_end(now) {
            result.push(end);
        }
        let WheelSample {
            delta,
            phase,
            macos_metadata,
            position,
            modifiers,
            scale,
            observed_at,
        } = sample;
        let has_platform_phase = macos_metadata
            .as_ref()
            .is_some_and(|event| event.phase != 0 || event.momentum_phase != 0);
        let event_scale = macos_metadata
            .as_ref()
            .map_or(scale, |event| event.scale_factor);
        let (delta, delta_units) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (
                Offset {
                    x: -f64::from(x) * 40.0,
                    y: -f64::from(y) * 40.0,
                },
                ScrollGranularity::kScrollByPixel,
            ),
            MouseScrollDelta::PixelDelta(point) => (
                Offset {
                    x: -point.x / event_scale,
                    y: -point.y / event_scale,
                },
                ScrollGranularity::kScrollByPrecisePixel,
            ),
        };
        let mut wheel = WheelEvent {
            delta_units,
            phase: match phase {
                TouchPhase::Started => WheelPhase::kBegan,
                TouchPhase::Moved => WheelPhase::kChanged,
                TouchPhase::Ended => WheelPhase::kEnded,
                TouchPhase::Cancelled => WheelPhase::kCancelled,
            },
            native: macos_metadata.as_ref().map(|event| WheelMetadata {
                timestamp_seconds: event.timestamp_seconds,
                phase: event.phase,
                momentum_phase: event.momentum_phase,
            }),
            timestamp: Some(observed_at),
            position,
            delta,
            modifiers,
            ..Default::default()
        };
        if !has_platform_phase {
            match phase {
                TouchPhase::Started => {
                    if let Some(end) = self.take_pending_wheel_end() {
                        result.push(end);
                    }
                    self.explicit_wheel_phase_active = true;
                    self.phase_less_wheel_active = false;
                }
                TouchPhase::Moved if self.explicit_wheel_phase_active => {}
                TouchPhase::Moved => {
                    // MouseWheelPhaseHandler::AddPhaseIfNeededAndScheduleEndEvent
                    // gives phase-less wheel devices a 500 ms latching
                    // transaction. This is upstream of Blink's wheel event and
                    // synthesized GestureScrollBegin/Update/End sequence; our
                    // compact host feeds that same phase into the compositor.
                    let outside_slop = self.phase_less_wheel_active
                        && ((position.x - self.phase_less_wheel_origin.x).powi(2)
                            + (position.y - self.phase_less_wheel_origin.y).powi(2)
                            >= 100.0);
                    let modifiers_changed = self.phase_less_wheel_active
                        && modifiers != self.phase_less_wheel_modifiers;
                    if outside_slop || modifiers_changed {
                        if let Some(end) = self.take_pending_wheel_end() {
                            result.push(end);
                        }
                    }
                    if self.phase_less_wheel_active {
                        wheel.phase = WheelPhase::kChanged;
                    } else {
                        wheel.phase = WheelPhase::kBegan;
                        self.phase_less_wheel_active = true;
                        self.phase_less_wheel_origin = position;
                        self.phase_less_wheel_modifiers = modifiers;
                    }
                    wheel.has_synthetic_phase = true;
                    let mut end = wheel.clone();
                    end.delta = Offset::default();
                    end.phase = WheelPhase::kEnded;
                    end.native = None;
                    self.pending_wheel_end = Some((now + Duration::from_millis(500), end));
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    self.explicit_wheel_phase_active = false;
                    self.phase_less_wheel_active = false;
                    self.pending_wheel_end = None;
                }
            }
        } else {
            self.explicit_wheel_phase_active = false;
            self.phase_less_wheel_active = false;
        }
        wheel.rails_mode = self.update_wheel_rails_mode(&wheel);
        if let Some(metadata) = macos_metadata {
            // RenderWidgetHostViewMac's MouseWheelPhaseHandler waits 100 ms
            // before ending a finger gesture so OS momentum can continue the
            // same latch. The original Ended delta is never dispatched.
            if metadata.phase == 8 {
                wheel.delta = Offset::default();
                wheel.phase = WheelPhase::kEnded;
                wheel.native = None;
                self.pending_wheel_end = Some((now + Duration::from_millis(100), wheel));
                return result;
            }
            if metadata.phase == 1 {
                if let Some(end) = self.take_pending_wheel_end() {
                    result.push(end);
                }
            } else if metadata.momentum_phase == 1 && self.pending_wheel_end.take().is_some() {
                wheel.phase = WheelPhase::kChanged;
            }
            if metadata.phase & 16 != 0 || metadata.momentum_phase & 16 != 0 {
                wheel.phase = WheelPhase::kCancelled;
            }
        }
        if wheel.phase == WheelPhase::kCancelled {
            self.pending_wheel_end = None;
        }
        result.push(Command::Input(InputEvent::Wheel(wheel)));
        result
    }
    fn mouse_with_click_count(
        &self,
        r#type: MouseEventType,
        button: MouseButton,
        click_count: i32,
    ) -> Command {
        Command::Input(InputEvent::Mouse(MouseEvent {
            r#type,
            button,
            buttons: self.buttons,
            click_count,
            position: self.cursor,
            modifiers: self.modifiers(),
            ..Default::default()
        }))
    }
    fn mouse(&self, r#type: MouseEventType, button: MouseButton) -> Command {
        self.mouse_with_click_count(r#type, button, 0)
    }
    fn next_touch_id(&mut self) -> u32 {
        self.next_touch_event_id = self.next_touch_event_id.wrapping_add(1).max(1);
        self.next_touch_event_id
    }
    // Chromium MotionEvent::Action CANCEL terminates the complete gesture
    // stream. The platform may report cancellation separately for each finger;
    // after this packet later cancellations are stale and ignored.
    fn cancel_native_touches(&mut self, timestamp: Instant) -> Option<Command> {
        if self.native_touches.is_empty() {
            return None;
        }
        let points = self
            .native_touches
            .drain(..)
            .map(|mut contact| {
                contact.point.state = TouchPointState::kCancelled;
                contact.point.movement_x = 0;
                contact.point.movement_y = 0;
                contact.point
            })
            .collect();
        let unique_touch_event_id = self.next_touch_id();
        Some(Command::Input(InputEvent::Touch(TouchEvent {
            r#type: TouchEventType::kCancel,
            points,
            timestamp: Some(timestamp),
            unique_touch_event_id,
            modifiers: self.modifiers(),
            ..Default::default()
        })))
    }
    fn native_touch(&mut self, event: &Touch, scale: f64, timestamp: Instant) -> Option<Command> {
        let index = self
            .native_touches
            .iter()
            .position(|contact| contact.device == event.device_id && contact.native_id == event.id);
        if event.phase == TouchPhase::Cancelled {
            return index.and_then(|_| self.cancel_native_touches(timestamp));
        }
        let index = if event.phase == TouchPhase::Started {
            // Duplicate DOWN is not a new contact; malformed/stale input cannot
            // replace an active source pointer or silently change its identity.
            if index.is_some() || self.native_touches.len() == 16 {
                return None;
            }
            let pointer_id = (0..16)
                .find(|id| {
                    self.native_touches
                        .iter()
                        .all(|c| c.point.pointer_id != *id)
                })
                .unwrap();
            let device_id = if let Some(index) = self
                .native_touch_devices
                .iter()
                .position(|device| *device == event.device_id)
            {
                i32::try_from(index).unwrap_or(-1)
            } else {
                let index = self.native_touch_devices.len();
                self.native_touch_devices.push(event.device_id);
                i32::try_from(index).unwrap_or(-1)
            };
            self.native_touches.push(NativeTouchContact {
                device: event.device_id,
                native_id: event.id,
                point: TouchPoint {
                    pointer_id,
                    device_id,
                    ..Default::default()
                },
            });
            self.native_touches.len() - 1
        } else {
            index?
        };
        for contact in &mut self.native_touches {
            contact.point.state = TouchPointState::kStationary;
            contact.point.movement_x = 0;
            contact.point.movement_y = 0;
        }
        let point = &mut self.native_touches[index].point;
        let position = Offset {
            x: event.location.x / scale,
            y: event.location.y / scale,
        };
        if event.phase != TouchPhase::Started {
            point.movement_x = (position.x - point.position.x).round() as i32;
            point.movement_y = (position.y - point.position.y).round() as i32;
        }
        point.position = position;
        point.pressure = event
            .force
            .as_ref()
            .map_or(f32::NAN, |force| force.normalized().clamp(0., 1.) as f32);
        // Winit Touch carries no radius, rotation, tool identity, hardware
        // timestamp, screen position or history. WebTouchPoint defaults (zero
        // major, unknown force) deliberately retain source unsupported values;
        // do not infer stylus classification from calibrated force.
        point.state = match event.phase {
            TouchPhase::Started => TouchPointState::kPressed,
            TouchPhase::Moved => TouchPointState::kMoved,
            TouchPhase::Ended => TouchPointState::kReleased,
            TouchPhase::Cancelled => unreachable!(),
        };
        let kind = match event.phase {
            TouchPhase::Started => TouchEventType::kStart,
            TouchPhase::Moved => TouchEventType::kMove,
            TouchPhase::Ended => TouchEventType::kEnd,
            TouchPhase::Cancelled => unreachable!(),
        };
        let points = self
            .native_touches
            .iter()
            .map(|contact| contact.point.clone())
            .collect();
        if kind == TouchEventType::kEnd {
            self.native_touches.remove(index);
        }
        let unique_touch_event_id = self.next_touch_id();
        Some(Command::Input(InputEvent::Touch(TouchEvent {
            r#type: kind,
            points,
            timestamp: Some(timestamp),
            modifiers: self.modifiers(),
            touch_start_or_first_touch_move: kind == TouchEventType::kStart,
            unique_touch_event_id,
            ..Default::default()
        })))
    }
    pub fn event(&mut self, event: &WindowEvent, scale: f64) -> Vec<Command> {
        let mut result = Vec::new();
        if let Some(end) = self.take_due_wheel_end(Instant::now()) {
            result.push(end);
        }
        match event {
            WindowEvent::Touch(touch) => {
                if let Some(command) = self.native_touch(touch, scale, Instant::now()) {
                    result.push(command);
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                if self.modifiers != modifiers.state() {
                    self.last_motion = None;
                }
                self.modifiers = modifiers.state();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let position = Offset {
                    x: position.x / scale,
                    y: position.y / scale,
                };
                let motion = (
                    position,
                    self.modifiers(),
                    self.pressed.map(|(button, _, _)| button),
                );
                // Winit's macOS scrollWheel calls mouse_motion even when the
                // cursor has not moved. Chromium's RWHVCocoa scrollWheel routes
                // only the wheel event (render_widget_host_view_cocoa.mm:1800).
                // Keep real moves and pointer/modifier/button boundaries, but
                // do not manufacture DOM mousemove for those repeated samples.
                let repeated = self.pointer_inside && self.last_motion == Some(motion);
                self.pointer_inside = true;
                self.cursor = position;
                if repeated {
                    return result;
                }
                self.last_motion = Some(motion);
                result.push(self.mouse(MouseEventType::kMove, MouseButton::kNone));
            }
            WindowEvent::CursorEntered { .. } => self.last_motion = None,
            WindowEvent::CursorLeft { .. } => {
                self.pointer_inside = false;
                self.last_motion = None;
                result.push(self.mouse(MouseEventType::kLeave, MouseButton::kNone))
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.last_motion = None;
                let button = match button {
                    winit::event::MouseButton::Left => MouseButton::kPrimary,
                    winit::event::MouseButton::Right => MouseButton::kSecondary,
                    winit::event::MouseButton::Middle => MouseButton::kMiddle,
                    _ => return result,
                };
                if *state == ElementState::Pressed {
                    self.buttons |= button.ButtonsMask();
                } else {
                    self.buttons &= !button.ButtonsMask();
                }
                if *state == ElementState::Pressed {
                    let now = Instant::now();
                    let click_count = self
                        .last_click
                        .filter(|(time, last, position, _)| {
                            *last == button
                                && now.duration_since(*time) < Duration::from_millis(400)
                                && near(*position, self.cursor)
                        })
                        .map_or(1, |(_, _, _, count)| count.saturating_add(1));
                    self.pressed = Some((button, self.cursor, click_count));
                    result.push(self.mouse_with_click_count(
                        MouseEventType::kDown,
                        button,
                        click_count,
                    ));
                } else {
                    let click_count = self
                        .pressed
                        .take()
                        .filter(|(down, position, _)| {
                            *down == button && near(*position, self.cursor)
                        })
                        .map_or(0, |(_, _, count)| count);
                    result.push(self.mouse_with_click_count(
                        MouseEventType::kUp,
                        button,
                        click_count,
                    ));
                    if click_count > 0 {
                        if button == MouseButton::kSecondary {
                            result.push(self.mouse_with_click_count(
                                MouseEventType::kContextMenu,
                                button,
                                click_count,
                            ));
                        }
                        let now = Instant::now();
                        self.last_click = Some((now, button, self.cursor, click_count));
                    }
                }
            }
            WindowEvent::MouseWheel {
                delta,
                phase,
                native: macos_metadata,
                ..
            } => {
                let sample = self.wheel_sample(*delta, *phase, *macos_metadata, scale);
                result.extend(self.accept_wheel_sample(sample, Instant::now()));
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } if !is_synthetic => {
                let key = key_name(&event.logical_key);
                let down = event.state == ElementState::Pressed;
                let (mut text, mut unmodified_text, native_key_code, ime_processed, committed_text) =
                    native_keyboard_text(event);
                if down && !event.repeat && !ime_processed && !self.composing {
                    let command = self.modifiers.control_key() || self.modifiers.super_key();
                    let shortcut = if command && key.eq_ignore_ascii_case("l") {
                        Some(Command::FocusAddress)
                    } else if command && key.eq_ignore_ascii_case("r") {
                        Some(Command::Reload)
                    } else if command && key.eq_ignore_ascii_case("t") {
                        Some(Command::NewTab)
                    } else if command && key.eq_ignore_ascii_case("w") {
                        Some(Command::CloseActiveTab)
                    } else if self.modifiers.control_key() && key == "Tab" {
                        Some(Command::CycleTab(if self.modifiers.shift_key() {
                            -1
                        } else {
                            1
                        }))
                    } else if self.modifiers.alt_key() && key == "ArrowLeft" {
                        Some(Command::Back)
                    } else if self.modifiers.alt_key() && key == "ArrowRight" {
                        Some(Command::Forward)
                    } else {
                        None
                    };
                    if let Some(shortcut) = shortcut {
                        result.push(shortcut);
                        return result;
                    }
                }
                let key_code = native_windows_key_code(event, &key);
                // Source WebKeyboardEventBuilder::Build normalizes AppKit's
                // return/backtab/private-use characters before delivery.
                #[cfg(target_os = "macos")]
                {
                    for value in [&mut text, &mut unmodified_text] {
                        if key_code == 13 {
                            *value = "\r".into();
                        } else if key_code == 9 {
                            *value = "\t".into();
                        } else if value.as_str() == "\u{7f}" {
                            *value = "\u{8}".into();
                        } else if value.chars().count() == 1
                            && value
                                .chars()
                                .next()
                                .is_some_and(|ch| ('\u{f700}'..='\u{f7ff}').contains(&ch))
                        {
                            *value = "\0".into();
                        }
                    }
                }
                let native = KeyEvent {
                    timestamp: Some(Instant::now()),
                    r#type: if down {
                        KeyEventType::kDown
                    } else {
                        KeyEventType::kUp
                    },
                    edit_commands: crate::native_kill_ring::editing_commands(event),
                    code: physical_code(event.physical_key),
                    key_code,
                    unmodified_text,
                    native_key_code,
                    ime_processed,
                    caps_lock: crate::native_keyboard_modifiers::current()
                        .ok()
                        .map(|modifiers| modifiers.alpha_lock),
                    is_system_key: native_system_key(self.modifiers(), key_code),
                    location: key_location(event.location),
                    is_composing: self.composing || ime_processed,
                    key,
                    text,
                    modifiers: self.modifiers(),
                    repeat: event.repeat,
                    ..Default::default()
                };
                let character_text = committed_text.unwrap_or_else(|| native.text.clone());
                let char_event = down
                    && !native.is_composing
                    && !character_text.is_empty()
                    && character_text != "\0";
                result.push(Command::Input(InputEvent::Key(native.clone())));
                if char_event {
                    let mut character = native;
                    character.r#type = KeyEventType::kChar;
                    character.text = character_text;
                    character.edit_commands.clear();
                    result.push(Command::Input(InputEvent::Key(character)));
                }
            }
            WindowEvent::Ime(Ime::Preedit(data, selection)) => {
                if !self.composing && !data.is_empty() {
                    self.composing = true;
                    result.push(composition(CompositionEventType::kStart, ""));
                }
                if self.composing {
                    // winit exposes native preedit selection as UTF-8 byte
                    // offsets. Convert at the host boundary so Interaction's
                    // platform-neutral contract remains Blink-compatible UTF-16.
                    let selection = selection.map(|(start, end)| {
                        let offset = |offset: usize| {
                            let mut boundary = offset.min(data.len());
                            while !data.is_char_boundary(boundary) {
                                boundary -= 1;
                            }
                            data[..boundary].encode_utf16().count()
                        };
                        (offset(start), offset(end))
                    });
                    result.push(Command::Input(InputEvent::Composition(CompositionEvent {
                        r#type: CompositionEventType::kUpdate,
                        data: data.clone(),
                        selection,
                        ..Default::default()
                    })));
                }
            }
            WindowEvent::Ime(Ime::Commit(data)) => {
                if self.composing {
                    result.push(composition(CompositionEventType::kEnd, data));
                    self.composing = false;
                } else {
                    result.push(Command::Input(InputEvent::TextInput(TextInputEvent {
                        text: data.clone(),
                        ..Default::default()
                    })));
                }
            }
            WindowEvent::Ime(Ime::Disabled) => {
                if self.composing {
                    result.push(composition(CompositionEventType::kEnd, ""));
                    self.composing = false;
                }
            }
            WindowEvent::Focused(false) => {
                if let Some(cancel) = self.cancel_native_touches(Instant::now()) {
                    result.push(cancel);
                }
                if self.composing {
                    result.push(composition(CompositionEventType::kEnd, ""));
                    self.composing = false;
                }
                if let Some(end) = self.take_pending_wheel_end() {
                    result.push(end);
                }
                self.explicit_wheel_phase_active = false;
                self.phase_less_wheel_active = false;
                self.pointer_inside = false;
                self.last_motion = None;
                self.modifiers = ModifiersState::empty();
                self.pressed = None;
                self.buttons = 0;
            }
            _ => {}
        }
        result
    }

    fn update_wheel_rails_mode(&mut self, wheel: &WheelEvent) -> WheelRailsMode {
        let Some(native) = wheel.native else {
            return WheelRailsMode::kFree;
        };
        if native.phase == 0 && native.momentum_phase == 0 {
            return WheelRailsMode::kFree;
        }
        if native.phase == 1 {
            self.wheel_rails_delta = Offset::default();
        }
        if wheel.delta.x == 0.0 && wheel.delta.y == 0.0 {
            return WheelRailsMode::kFree;
        }
        self.wheel_rails_delta.x = self.wheel_rails_delta.x * 0.8 + wheel.delta.x.abs();
        self.wheel_rails_delta.y = self.wheel_rails_delta.y * 0.8 + wheel.delta.y.abs();
        let maximum = self.wheel_rails_delta.x.max(self.wheel_rails_delta.y);
        let minimum = self.wheel_rails_delta.x.min(self.wheel_rails_delta.y);
        if minimum * 2.0 > maximum {
            WheelRailsMode::kFree
        } else if self.wheel_rails_delta.y >= self.wheel_rails_delta.x {
            WheelRailsMode::kVertical
        } else {
            WheelRailsMode::kHorizontal
        }
    }
}
fn near(a: Offset, b: Offset) -> bool {
    (a.x - b.x).abs() <= 5.0 && (a.y - b.y).abs() <= 5.0
}
fn composition(r#type: CompositionEventType, data: &str) -> Command {
    Command::Input(InputEvent::Composition(CompositionEvent {
        r#type,
        data: data.into(),
        ..Default::default()
    }))
}
fn key_name(key: &Key) -> String {
    match key {
        Key::Character(text) => text.to_string(),
        Key::Named(NamedKey::Space) => " ".into(),
        Key::Named(key) => format!("{key:?}"),
        Key::Dead(_) | Key::Unidentified(_) => "Unidentified".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_touch_retains_contacts_ids_and_css_coordinates() {
        let device_id = DeviceId::dummy();
        let mut input = InputState::default();
        let touch = |phase, id, x| {
            WindowEvent::Touch(Touch {
                device_id,
                phase,
                id,
                location: winit::dpi::PhysicalPosition::new(x, 80.),
                force: Some(winit::event::Force::Normalized(0.75)),
            })
        };
        let packet = |commands: Vec<Command>| {
            let mut commands = commands.into_iter();
            let Some(Command::Input(InputEvent::Touch(packet))) = commands.next() else {
                panic!("native touch packet");
            };
            assert!(commands.next().is_none());
            packet
        };
        let first = packet(input.event(&touch(TouchPhase::Started, u64::MAX, 100.), 2.));
        assert_eq!(first.points[0].position, Offset { x: 50., y: 40. });
        assert_eq!(first.points[0].pressure, 0.75);
        assert!(first.timestamp.is_some());
        assert_eq!(first.points[0].device_id, 0);
        let second = packet(input.event(&touch(TouchPhase::Started, 9, 200.), 2.));
        assert_eq!(second.points.len(), 2);
        assert_eq!(second.points[0].pointer_id, first.points[0].pointer_id);
        assert_eq!(second.points[0].state, TouchPointState::kStationary);
        assert_ne!(second.points[0].pointer_id, second.points[1].pointer_id);
        let moved = packet(input.event(&touch(TouchPhase::Moved, u64::MAX, 120.), 2.));
        assert!(moved.unique_touch_event_id > second.unique_touch_event_id);
        let up = packet(input.event(&touch(TouchPhase::Ended, u64::MAX, 120.), 2.));
        assert_eq!(up.points[0].state, TouchPointState::kReleased);
        let cancel = packet(input.event(&WindowEvent::Focused(false), 2.));
        assert_eq!(cancel.r#type, TouchEventType::kCancel);
        assert_eq!(cancel.points.len(), 1);
        assert_eq!(cancel.points[0].pointer_id, second.points[1].pointer_id);
        assert!(input.native_touches.is_empty());
    }

    #[test]
    fn repeated_wheel_motion_preserves_coalescing_and_pointer_boundaries() {
        use interaction::frame_aligned_input_queue::FrameAlignedInputQueue;
        let device_id = winit::event::DeviceId::dummy();
        let motion = |x| WindowEvent::CursorMoved {
            device_id,
            position: winit::dpi::PhysicalPosition::new(x, 80.0),
        };
        let wheel = WindowEvent::MouseWheel {
            device_id,
            phase: TouchPhase::Moved,
            delta: MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(0.0, -8.0)),
            native: Some(winit::event::MacOSMouseWheelMetadata {
                timestamp_seconds: 1.0,
                phase: 0,
                momentum_phase: 0,
                is_continuous: Some(false),
                point_delta_x: None,
                point_delta_y: None,
                cocoa_delta_x: 0.0,
                cocoa_delta_y: -4.0,
                precise: true,
                position_x: 40.0,
                position_y: 80.0,
                scale_factor: 2.0,
            }),
        };
        let mut state = InputState::default();
        assert_eq!(state.event(&motion(40.0), 2.0).len(), 1);
        let mut queue = FrameAlignedInputQueue::default();
        for _ in 0..2 {
            // Winit sends this unchanged CursorMoved before each scrollWheel.
            assert!(state.event(&motion(40.0), 2.0).is_empty());
            for command in state.event(&wheel, 2.0) {
                let Command::Input(input) = command else {
                    panic!("expected native input")
                };
                queue.Push(input, Instant::now(), 1);
            }
        }
        let mut dispatched = Vec::new();
        queue.Dispatch(|entry| {
            dispatched.push(entry);
            false
        });
        let mut dispatched = dispatched.into_iter();
        let begin = dispatched.next().unwrap();
        assert!(matches!(begin.input, InputEvent::Wheel(event)
            if event.phase == WheelPhase::kBegan && event.delta.y == 4.0));
        let update = dispatched.next().unwrap();
        assert!(matches!(update.input, InputEvent::Wheel(event)
            if event.phase == WheelPhase::kChanged && event.delta.y == 4.0));
        assert!(queue.IsEmpty());
        let deadline = state.pending_wheel_end_deadline().unwrap();
        assert!(
            matches!(state.take_due_wheel_end(deadline), Some(Command::Input(InputEvent::Wheel(event)))
            if event.phase == WheelPhase::kEnded && event.delta == Offset::default())
        );
        assert_eq!(state.event(&motion(42.0), 2.0).len(), 1);
        assert!(state.event(&motion(42.0), 2.0).is_empty());

        state.event(
            &WindowEvent::ModifiersChanged(ModifiersState::SHIFT.into()),
            2.0,
        );
        assert_eq!(state.event(&motion(42.0), 2.0).len(), 1);
        state.event(
            &WindowEvent::ModifiersChanged(ModifiersState::empty().into()),
            2.0,
        );
        assert_eq!(state.event(&motion(42.0), 2.0).len(), 1);
        for button_state in [ElementState::Pressed, ElementState::Released] {
            let commands = state.event(
                &WindowEvent::MouseInput {
                    device_id,
                    state: button_state,
                    button: winit::event::MouseButton::Left,
                },
                2.0,
            );
            assert_eq!(
                commands.len(),
                1,
                "the source mouse manager generates click/double-click"
            );
            let expected = if button_state == ElementState::Pressed {
                MouseEventType::kDown
            } else {
                MouseEventType::kUp
            };
            assert!(
                matches!(&commands[0], Command::Input(InputEvent::Mouse(mouse))
                if mouse.r#type == expected && mouse.click_count == 1)
            );
            assert_eq!(state.event(&motion(42.0), 2.0).len(), 1);
        }
        state.event(&WindowEvent::CursorLeft { device_id }, 2.0);
        state.event(&WindowEvent::CursorEntered { device_id }, 2.0);
        assert_eq!(state.event(&motion(42.0), 2.0).len(), 1);
        state.event(&WindowEvent::Focused(false), 2.0);
        assert_eq!(state.event(&motion(42.0), 2.0).len(), 1);
    }
    #[test]
    fn wheel_gesture_phases_survive_platform_adaptation() {
        let mut state = InputState::default();
        for (native, expected) in [
            (TouchPhase::Started, WheelPhase::kBegan),
            (TouchPhase::Moved, WheelPhase::kChanged),
            (TouchPhase::Ended, WheelPhase::kEnded),
            (TouchPhase::Cancelled, WheelPhase::kCancelled),
        ] {
            let commands = state.event(
                &WindowEvent::MouseWheel {
                    device_id: winit::event::DeviceId::dummy(),
                    delta: MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(
                        4.0, -8.0,
                    )),
                    phase: native,
                    native: None,
                },
                2.0,
            );
            let expected_delta = Offset { x: -2.0, y: 4.0 };
            assert!(
                matches!(&commands[0], Command::Input(InputEvent::Wheel(w)) if w.phase == expected && w.delta == expected_delta)
            );
        }
    }

    #[test]
    fn finger_end_waits_for_momentum_and_never_reapplies_its_delta() {
        use winit::event::MacOSMouseWheelMetadata;
        let event = |phase, momentum_phase, timestamp_seconds| WindowEvent::MouseWheel {
            device_id: winit::event::DeviceId::dummy(),
            delta: MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(0.0, 318.0)),
            phase: if phase == 8 {
                TouchPhase::Ended
            } else if momentum_phase == 1 || phase == 1 {
                TouchPhase::Started
            } else {
                TouchPhase::Moved
            },
            native: Some(MacOSMouseWheelMetadata {
                timestamp_seconds,
                phase,
                momentum_phase,
                is_continuous: Some(true),
                point_delta_x: Some(0),
                point_delta_y: Some(318),
                cocoa_delta_x: 0.0,
                cocoa_delta_y: 318.0,
                precise: true,
                position_x: 0.0,
                position_y: 0.0,
                scale_factor: 2.0,
            }),
        };
        let mut state = InputState::default();
        // Observed native transition: the same 159 CSS px appears in both
        // finger Ended and momentum Began, sharing the timestamp.
        assert!(state.event(&event(8, 0, 123.0), 2.0).is_empty());
        let deadline = state.pending_wheel_end_deadline().unwrap();
        assert!(state
            .take_due_wheel_end(deadline - Duration::from_millis(1))
            .is_none());
        let commands = state.event(&event(0, 1, 123.0), 2.0);
        assert!(state.pending_wheel_end_deadline().is_none());
        assert!(
            matches!(commands.as_slice(), [Command::Input(InputEvent::Wheel(w))]
            if w.phase == WheelPhase::kChanged && w.delta.y == -159.0
                && w.native.is_some_and(|m| m.phase == 0 && m.momentum_phase == 1))
        );
        let Command::Input(continuation) = &commands[0] else {
            unreachable!()
        };
        assert!(
            interaction::frame_aligned_input_queue::FrameAlignedInputQueue::IsFrameAligned(
                continuation
            )
        );

        // Without momentum, the real deadline emits one zero-delta end.
        assert!(state.event(&event(8, 0, 124.0), 2.0).is_empty());
        let deadline = state.pending_wheel_end_deadline().unwrap();
        assert!(
            matches!(state.take_due_wheel_end(deadline), Some(Command::Input(InputEvent::Wheel(w)))
            if w.phase == WheelPhase::kEnded && w.delta == Offset::default())
        );
        assert!(state.take_due_wheel_end(deadline).is_none());
        // After the gesture actually ended, a standalone momentum start still
        // establishes a new scroll instead of pretending a sequence exists.
        let commands = state.event(&event(0, 1, 124.2), 2.0);
        assert!(
            matches!(commands.as_slice(), [Command::Input(InputEvent::Wheel(w))]
            if w.phase == WheelPhase::kBegan)
        );

        // A new finger gesture first dispatches a pending zero-delta end.
        assert!(state.event(&event(8, 0, 125.0), 2.0).is_empty());
        let commands = state.event(&event(1, 0, 126.0), 2.0);
        assert!(
            matches!(commands.as_slice(), [Command::Input(InputEvent::Wheel(end)), Command::Input(InputEvent::Wheel(begin))]
            if end.phase == WheelPhase::kEnded && end.delta == Offset::default()
                && begin.phase == WheelPhase::kBegan)
        );
        let commands = state.event(&event(16, 0, 127.0), 2.0);
        assert!(
            matches!(commands.as_slice(), [Command::Input(InputEvent::Wheel(w))]
            if w.phase == WheelPhase::kCancelled && w.delta.y == -159.0)
        );

        let wheel = |phase, momentum_phase, timestamp_seconds| WheelEvent {
            phase: WheelPhase::kChanged,
            native: Some(WheelMetadata {
                timestamp_seconds,
                phase,
                momentum_phase,
            }),
            ..Default::default()
        };
        assert!(wheel(4, 0, 1.0).CanCoalesce(&wheel(4, 0, 2.0)));
        assert!(!wheel(4, 0, 1.0).CanCoalesce(&wheel(0, 4, 2.0)));
    }

    #[test]
    fn ime_preedit_converts_utf8_byte_selection_and_cancels_on_focus_loss() {
        let mut state = InputState::default();
        let commands = state.event(
            &WindowEvent::Ime(Ime::Preedit("你🙂好".into(), Some((3, 7)))),
            2.0,
        );
        assert!(
            matches!(&commands[1], Command::Input(InputEvent::Composition(event))
            if event.r#type == CompositionEventType::kUpdate && event.selection == Some((1, 3)))
        );
        let commands = state.event(&WindowEvent::Focused(false), 2.0);
        assert!(
            matches!(&commands[0], Command::Input(InputEvent::Composition(event))
            if event.r#type == CompositionEventType::kEnd && event.data.is_empty())
        );
        assert!(!state.composing);
        let commands = state.event(&WindowEvent::Ime(Ime::Commit("新".into())), 2.0);
        assert!(
            matches!(&commands[0], Command::Input(InputEvent::TextInput(event)) if event.text == "新")
        );
    }

    #[test]
    fn ime_clear_before_commit_does_not_commit_or_duplicate_text() {
        let mut state = InputState::default();
        assert_eq!(
            state
                .event(
                    &WindowEvent::Ime(Ime::Preedit("ni".into(), Some((2, 2)))),
                    2.0
                )
                .len(),
            2
        );
        let clear = state.event(&WindowEvent::Ime(Ime::Preedit("".into(), None)), 2.0);
        assert!(
            matches!(&clear[0], Command::Input(InputEvent::Composition(event)) if event.r#type == CompositionEventType::kUpdate)
        );
        let commit = state.event(&WindowEvent::Ime(Ime::Commit("你".into())), 2.0);
        assert_eq!(commit.len(), 1);
        assert!(
            matches!(&commit[0], Command::Input(InputEvent::Composition(event)) if event.r#type == CompositionEventType::kEnd && event.data == "你")
        );
    }
}

/// Native cursor adaptation is deliberately the last boundary: Page/interaction
/// never depends on winit or platform cursor handles.
pub fn set_cursor(
    event_loop: &winit::event_loop::ActiveEventLoop,
    window: &winit::window::Window,
    cursor: &interaction::Cursor,
) -> Result<(), String> {
    use interaction::cursor::CursorType as C;
    use interaction::Cursor;
    use winit::window::CursorIcon as W;

    match cursor {
        Cursor::Custom {
            bitmap,
            hot_spot,
            image_scale_factor,
        } => {
            if !image_scale_factor.is_finite() || *image_scale_factor <= 0.0 {
                return Err("invalid cursor image scale".into());
            }
            let size = bitmap.Size();
            let width = u16::try_from(size.width).map_err(|_| "invalid cursor width")?;
            let height = u16::try_from(size.height).map_err(|_| "invalid cursor height")?;
            let row_bytes = bitmap.RowBytes();
            let packed_row_bytes = usize::from(width) * 4;
            let required = row_bytes
                .checked_mul(usize::from(height))
                .ok_or("cursor bitmap size overflow")?;
            let pixels = bitmap.Rgba8();
            if width == 0 || height == 0 || row_bytes < packed_row_bytes || pixels.len() < required
            {
                return Err("invalid cursor bitmap storage".into());
            }
            let mut rgba = Vec::with_capacity(packed_row_bytes * usize::from(height));
            for row in pixels[..required].chunks_exact(row_bytes) {
                rgba.extend_from_slice(&row[..packed_row_bytes]);
            }
            let hotspot_x = u16::try_from(hot_spot.x).map_err(|_| "invalid cursor hotspot x")?;
            let hotspot_y = u16::try_from(hot_spot.y).map_err(|_| "invalid cursor hotspot y")?;
            #[cfg(target_os = "macos")]
            let source = {
                use winit::platform::macos::CustomCursorExtMacOS;
                winit::window::CustomCursor::from_rgba_with_scale_factor(
                    rgba,
                    width,
                    height,
                    hotspot_x,
                    hotspot_y,
                    *image_scale_factor,
                )
            };
            #[cfg(not(target_os = "macos"))]
            let source = {
                // These winit backends accept native device pixels. Cursor
                // image scale must match the output's device scale; unlike
                // AppKit they do not have a separate image logical-size API.
                if (*image_scale_factor as f64 - window.scale_factor()).abs() > f64::EPSILON {
                    return Err("native cursor image scale does not match output scale".into());
                }
                winit::window::CustomCursor::from_rgba(rgba, width, height, hotspot_x, hotspot_y)
            };
            let source = source.map_err(|error| error.to_string())?;
            window.set_cursor(event_loop.create_custom_cursor(source));
            window.set_cursor_visible(true);
        }
        Cursor::Standard(kind) => {
            window.set_cursor_visible(*kind != C::kNone);
            if *kind == C::kNone {
                return Ok(());
            }
            window.set_cursor(match kind {
                C::kNull | C::kPointer => W::Default,
                C::kHand => W::Pointer,
                C::kIBeam => W::Text,
                C::kVerticalText => W::VerticalText,
                C::kCross => W::Crosshair,
                C::kCopy | C::kDndCopy => W::Copy,
                C::kMove | C::kDndMove => W::Move,
                C::kCell => W::Cell,
                C::kContextMenu => W::ContextMenu,
                C::kAlias | C::kDndLink => W::Alias,
                C::kProgress => W::Progress,
                C::kNoDrop | C::kDndNone => W::NoDrop,
                C::kNotAllowed
                | C::kEastWestNoResize
                | C::kNorthSouthNoResize
                | C::kNorthEastSouthWestNoResize
                | C::kNorthWestSouthEastNoResize => W::NotAllowed,
                C::kZoomIn => W::ZoomIn,
                C::kZoomOut => W::ZoomOut,
                C::kEastResize | C::kEastPanning => W::EResize,
                C::kNorthEastResize | C::kNorthEastPanning => W::NeResize,
                C::kNorthWestResize | C::kNorthWestPanning => W::NwResize,
                C::kNorthResize | C::kNorthPanning => W::NResize,
                C::kSouthEastResize | C::kSouthEastPanning => W::SeResize,
                C::kSouthWestResize | C::kSouthWestPanning => W::SwResize,
                C::kSouthResize | C::kSouthPanning => W::SResize,
                C::kWestResize | C::kWestPanning => W::WResize,
                C::kEastWestResize | C::kMiddlePanningHorizontal => W::EwResize,
                C::kNorthSouthResize | C::kMiddlePanningVertical => W::NsResize,
                C::kNorthEastSouthWestResize => W::NeswResize,
                C::kNorthWestSouthEastResize => W::NwseResize,
                C::kColumnResize => W::ColResize,
                C::kRowResize => W::RowResize,
                C::kWait => W::Wait,
                C::kHelp => W::Help,
                C::kMiddlePanning => W::AllScroll,
                C::kGrab => W::Grab,
                C::kGrabbing => W::Grabbing,
                C::kNone => return Ok(()),
                C::kCustom => return Err("custom cursor type has no bitmap".into()),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod cursor_tests {
    use super::*;
    #[test]
    fn leaving_view_rejects_late_cursor_feedback() {
        let mut input = InputState::default();
        let device_id = winit::event::DeviceId::dummy();
        input.event(
            &WindowEvent::CursorMoved {
                device_id,
                position: winit::dpi::PhysicalPosition::new(40.0, 80.0),
            },
            2.0,
        );
        assert!(input.pointer_inside());
        let commands = input.event(&WindowEvent::CursorLeft { device_id }, 2.0);
        assert!(!input.pointer_inside());
        assert!(
            matches!(&commands[0], Command::Input(InputEvent::Mouse(event)) if event.r#type == MouseEventType::kLeave)
        );
        input.event(
            &WindowEvent::CursorMoved {
                device_id,
                position: winit::dpi::PhysicalPosition::new(40.0, 80.0),
            },
            2.0,
        );
        input.event(&WindowEvent::Focused(false), 2.0);
        assert!(!input.pointer_inside());
    }
}
