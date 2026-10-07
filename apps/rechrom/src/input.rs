use crate::engine::Command;
use interaction::input_event::*;
use layoutng_assembly::internal::layout_input::Offset;
use std::time::{Duration, Instant};
use winit::{
    event::{ElementState, Ime, MouseScrollDelta, TouchPhase, WindowEvent},
    keyboard::{Key, ModifiersState, NamedKey},
};

#[derive(Default)]
pub struct InputState {
    modifiers: ModifiersState,
    cursor: Offset,
    pressed: Option<(MouseButton, Offset)>,
    last_click: Option<(Instant, MouseButton, Offset)>,
    composing: bool,
    pointer_inside: bool,
    last_motion: Option<(Offset, EventModifiers, Option<MouseButton>)>,
    pending_wheel_end: Option<(Instant, WheelEvent)>,
    wheel_rails_delta: Offset,
}
impl InputState {
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
            self.take_pending_wheel_end()
        } else {
            None
        }
    }
    fn take_pending_wheel_end(&mut self) -> Option<Command> {
        self.pending_wheel_end
            .take()
            .map(|(_, wheel)| Command::Input(InputEvent::Wheel(wheel)))
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
    fn mouse(&self, r#type: MouseEventType, button: MouseButton) -> Command {
        Command::Input(InputEvent::Mouse(MouseEvent {
            r#type,
            button,
            position: self.cursor,
            modifiers: self.modifiers(),
            ..Default::default()
        }))
    }
    pub fn event(&mut self, event: &WindowEvent, scale: f64) -> Vec<Command> {
        let mut result = Vec::new();
        if let Some(end) = self.take_due_wheel_end(Instant::now()) {
            result.push(end);
        }
        match event {
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
                    self.pressed.map(|(button, _)| button),
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
                    self.pressed = Some((button, self.cursor));
                    result.push(self.mouse(MouseEventType::kDown, button));
                } else {
                    result.push(self.mouse(MouseEventType::kUp, button));
                    if self.pressed.take().is_some_and(|(down, position)| {
                        down == button && near(position, self.cursor)
                    }) {
                        result.push(self.mouse(
                            if button == MouseButton::kSecondary {
                                MouseEventType::kContextMenu
                            } else {
                                MouseEventType::kClick
                            },
                            button,
                        ));
                        let now = Instant::now();
                        if button == MouseButton::kPrimary
                            && self.last_click.is_some_and(|(time, last, position)| {
                                last == button
                                    && now.duration_since(time) < Duration::from_millis(400)
                                    && near(position, self.cursor)
                            })
                        {
                            result.push(self.mouse(MouseEventType::kDoubleClick, button));
                            self.last_click = None;
                        } else {
                            self.last_click = Some((now, button, self.cursor));
                        }
                    }
                }
            }
            WindowEvent::MouseWheel {
                delta,
                phase,
                native,
                ..
            } => {
                let (delta, delta_units) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (
                        Offset {
                            x: -f64::from(*x) * 40.0,
                            y: -f64::from(*y) * 40.0,
                        },
                        ScrollGranularity::kScrollByPixel,
                    ),
                    MouseScrollDelta::PixelDelta(position) => (
                        Offset {
                            x: -position.x / scale,
                            y: -position.y / scale,
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
                    native: native.map(|event| NativeWheelMetadata {
                        timestamp_seconds: event.timestamp_seconds,
                        phase: event.phase,
                        momentum_phase: event.momentum_phase,
                    }),
                    position: self.cursor,
                    delta,
                    modifiers: self.modifiers(),
                    ..Default::default()
                };
                // content::MouseWheelRailsFilterMac: retain a short decayed
                // history for the gesture. A 2:1 dominant axis is railed;
                // genuinely diagonal gestures remain free.
                wheel.rails_mode = self.update_wheel_rails_mode(&wheel);
                if let Some(native) = native {
                    // RWHV Mac::RouteOrProcessWheelEvent (2155-2161) does not
                    // forward finger Ended's original delta. PhaseHandler
                    // (42-66) waits 100ms, cancelling that end on momentum Began.
                    // Real devices can repeat a nonzero delta in Ended and
                    // momentum Began at the same timestamp; timestamp is not
                    // used to deduplicate either event.
                    if native.phase == 8 {
                        wheel.delta = Offset::default();
                        wheel.phase = WheelPhase::kEnded;
                        // This is a scheduled synthetic zero-delta boundary,
                        // not a second dispatch of the original NSEvent.
                        wheel.native = None;
                        self.pending_wheel_end =
                            Some((Instant::now() + Duration::from_millis(100), wheel));
                        return result;
                    }
                    if native.phase == 1 {
                        if let Some(end) = self.take_pending_wheel_end() {
                            result.push(end);
                        }
                    } else if native.momentum_phase == 1 {
                        // MouseWheelEventQueue keeps the existing GSB alive
                        // through finger Ended -> momentum Began, and forwards
                        // an inertial GSU. Winit reports Started for both kinds
                        // of Began; treating this continuation as kBegan would
                        // flush our frame queue and apply its delta immediately.
                        // Preserve the original momentum phase in `native`.
                        if self.pending_wheel_end.take().is_some() {
                            wheel.phase = WheelPhase::kChanged;
                        }
                    }
                    if native.phase & 16 != 0 || native.momentum_phase & 16 != 0 {
                        wheel.phase = WheelPhase::kCancelled;
                    }
                }
                if wheel.phase == WheelPhase::kCancelled {
                    // MouseWheelEventQueue can apply a nonzero cancellation
                    // delta before ending the gesture. Keep that native delta.
                    self.pending_wheel_end = None;
                }
                result.push(Command::Input(InputEvent::Wheel(wheel)));
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } if !is_synthetic => {
                let key = key_name(&event.logical_key);
                let down = event.state == ElementState::Pressed;
                if down && !event.repeat {
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
                let text = if down
                    && !self.composing
                    && !self.modifiers.control_key()
                    && !self.modifiers.super_key()
                {
                    event
                        .text
                        .as_ref()
                        .map(|text| text.chars().filter(|c| !c.is_control()).collect())
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                result.push(Command::Input(InputEvent::Key(KeyEvent {
                    r#type: if down {
                        KeyEventType::kDown
                    } else {
                        KeyEventType::kUp
                    },
                    key,
                    text,
                    modifiers: self.modifiers(),
                    repeat: event.repeat,
                    ..Default::default()
                })));
            }
            WindowEvent::Ime(Ime::Preedit(data, selection)) => {
                if !self.composing && !data.is_empty() {
                    self.composing = true;
                    result.push(composition(CompositionEventType::kStart, ""));
                }
                if self.composing {
                    result.push(Command::Input(InputEvent::Composition(CompositionEvent {
                        r#type: CompositionEventType::kUpdate,
                        data: data.clone(),
                        selection: *selection,
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
                if self.composing {
                    result.push(composition(CompositionEventType::kEnd, ""));
                    self.composing = false;
                }
                if let Some(end) = self.take_pending_wheel_end() {
                    result.push(end);
                }
                self.pointer_inside = false;
                self.last_motion = None;
                self.modifiers = ModifiersState::empty();
                self.pressed = None;
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
    fn repeated_native_wheel_motion_preserves_coalescing_and_pointer_boundaries() {
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
            native: None,
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
        let merged = queue.PopFront().unwrap();
        assert!(matches!(merged.input, InputEvent::Wheel(event)
            if event.phase == WheelPhase::kChanged && event.delta.y == 8.0));
        assert_eq!(merged.samples, 2);
        assert!(queue.IsEmpty());
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
            state.event(
                &WindowEvent::MouseInput {
                    device_id,
                    state: button_state,
                    button: winit::event::MouseButton::Left,
                },
                2.0,
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
    fn wheel_gesture_phases_survive_native_adaptation() {
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
    fn native_finger_end_waits_for_momentum_and_never_reapplies_its_delta() {
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
            native: Some(NativeWheelMetadata {
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
    fn ime_preedit_preserves_native_byte_selection_and_cancels_on_focus_loss() {
        let mut state = InputState::default();
        let commands = state.event(
            &WindowEvent::Ime(Ime::Preedit("你🙂好".into(), Some((3, 7)))),
            2.0,
        );
        assert!(
            matches!(&commands[1], Command::Input(InputEvent::Composition(event))
            if event.r#type == CompositionEventType::kUpdate && event.selection == Some((3, 7)))
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
pub fn set_cursor(window: &winit::window::Window, cursor: rechrom::page::Cursor) {
    use rechrom::page::Cursor as C;
    use winit::window::CursorIcon as W;
    window.set_cursor_visible(cursor != C::kNone);
    if cursor == C::kNone {
        return;
    }
    window.set_cursor(match cursor {
        C::kAuto | C::kDefault | C::kNone => W::Default,
        C::kPointer => W::Pointer,
        C::kText => W::Text,
        C::kVerticalText => W::VerticalText,
        C::kCrosshair => W::Crosshair,
        C::kCopy => W::Copy,
        C::kMove => W::Move,
        C::kCell => W::Cell,
        C::kContextMenu => W::ContextMenu,
        C::kAlias => W::Alias,
        C::kProgress => W::Progress,
        C::kNoDrop => W::NoDrop,
        C::kNotAllowed => W::NotAllowed,
        C::kZoomIn => W::ZoomIn,
        C::kZoomOut => W::ZoomOut,
        C::kEResize => W::EResize,
        C::kNeResize => W::NeResize,
        C::kNwResize => W::NwResize,
        C::kNResize => W::NResize,
        C::kSeResize => W::SeResize,
        C::kSwResize => W::SwResize,
        C::kSResize => W::SResize,
        C::kWResize => W::WResize,
        C::kEwResize => W::EwResize,
        C::kNsResize => W::NsResize,
        C::kNeswResize => W::NeswResize,
        C::kNwseResize => W::NwseResize,
        C::kColResize => W::ColResize,
        C::kRowResize => W::RowResize,
        C::kWait => W::Wait,
        C::kHelp => W::Help,
        C::kAllScroll => W::AllScroll,
        C::kGrab => W::Grab,
        C::kGrabbing => W::Grabbing,
    });
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
