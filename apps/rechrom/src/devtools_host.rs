//! Shared DevTools attachment for native and displayless application hosts.
//!
//! Both hosts expose the same page commands and the same completed Renderer
//! pixels. The transport remains the existing local HTTP control endpoint.
use crate::engine::{Command, CopyOutputTarget, FrameMailbox, ResizeRequest, Viewport};
use std::{
    io,
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};

pub(crate) fn serve(port: u16, address: &str) -> io::Result<String> {
    browser_tracing::register_target(1, "Page", address);
    browser_tracing::register_target(2, "Browser toolbar", "browser://toolbar");
    let url = devtools::serve(port).or_else(|error| {
        if error.kind() == io::ErrorKind::AddrInUse {
            devtools::serve(0)
        } else {
            Err(error)
        }
    })?;
    println!("devtools-url\t{url}");
    Ok(url)
}

pub(crate) fn attach_screenshots(mailbox: FrameMailbox) {
    devtools::set_screenshot_handler(Arc::new(move |target_id| {
        // Snapshot one completed frame under the publication lock. PNG
        // compression must not block Display from publishing the next frame.
        let frame = {
            let _copy = browser_tracing::span("devtools", "Screenshot.CopyPixels");
            let frame = mailbox.lock().unwrap();
            let frame = frame.as_ref().ok_or("No completed frame")?;
            copy_screenshot(frame, target_id)?
        };
        encode_screenshot(frame, target_id)
    }));
}

/// Install the displayless CopyOutput path. Unlike native diagnostics, an
/// offscreen target remains owned by Display after presentation, so screenshot
/// readback can be deferred until DevTools actually asks for it.
fn attach_offscreen_screenshots(
    commands: &mpsc::Sender<Command>,
    expected_viewport: Arc<Mutex<Viewport>>,
) {
    let commands = commands.clone();
    devtools::set_screenshot_handler(Arc::new(move |target_id| {
        let target = match target_id {
            1 => CopyOutputTarget::Page,
            2 => CopyOutputTarget::Toolbar,
            3 => CopyOutputTarget::Root,
            _ => {
                return Err(
                    "target_id must be 1 (page), 2 (browser toolbar), or 3 (browser root)".into(),
                )
            }
        };
        let expected_viewport = *expected_viewport.lock().unwrap();
        let (reply, result) = mpsc::channel();
        commands
            .send(Command::CopyOutput {
                expected_viewport,
                target,
                reply,
            })
            .map_err(|error| error.to_string())?;
        let output = result
            .recv_timeout(Duration::from_secs(10))
            .map_err(|error| error.to_string())??;
        encode_screenshot(
            (
                output.width,
                output.height,
                output.pixels,
                output.provenance,
            ),
            target_id,
        )
    }));
}

pub(crate) fn attach_offscreen(commands: &mpsc::Sender<Command>, initial_viewport: Viewport) {
    let expected_viewport = Arc::new(Mutex::new(initial_viewport));
    attach_offscreen_screenshots(commands, expected_viewport.clone());
    attach_commands_with_expected_viewport(commands, Some(expected_viewport));
}

fn copy_screenshot(
    frame: &crate::engine::WindowFrame,
    target_id: u64,
) -> Result<
    (
        u32,
        u32,
        Vec<u8>,
        Option<crate::page_presentation::FrameProvenance>,
    ),
    String,
> {
    let surface = match target_id {
        1 => &frame.content,
        2 => &frame.toolbar,
        3 => {
            let (toolbar_width, toolbar_height) = frame.toolbar.size();
            let (content_width, content_height) = frame.content.size();
            if toolbar_width != content_width {
                return Err("browser root child widths do not match".into());
            }
            let mut pixels =
                Vec::with_capacity(frame.toolbar.pixels().len() + frame.content.pixels().len());
            pixels.extend_from_slice(frame.toolbar.pixels());
            pixels.extend_from_slice(frame.content.pixels());
            return Ok((
                toolbar_width,
                toolbar_height + content_height,
                pixels,
                frame.provenance,
            ));
        }
        _ => {
            return Err(
                "target_id must be 1 (page), 2 (browser toolbar), or 3 (browser root)".into(),
            )
        }
    };
    let (width, height) = surface.size();
    Ok((width, height, surface.pixels().to_vec(), frame.provenance))
}

fn encode_screenshot(
    (width, height, mut pixels, provenance): (
        u32,
        u32,
        Vec<u8>,
        Option<crate::page_presentation::FrameProvenance>,
    ),
    target_id: u64,
) -> Result<Vec<u8>, String> {
    if crate::engine::memory_diagnostics_enabled() {
        eprintln!(
            "pipeline-capture time_ns={} target={} provenance={:?}",
            crate::engine::diagnostic_time_ns(),
            target_id,
            provenance
        );
    }
    // Page.captureScreenshot returns the composited page against its default
    // canvas background. Keep the protocol image opaque like Chromium during
    // ordinary use. Raster parity diagnostics can preserve the completed
    // surface alpha so coverage can be compared before the final white blend.
    let _encode = browser_tracing::span("devtools", "Screenshot.Encode");
    if std::env::var("RECHROM_TRANSPARENT_BACKGROUND").as_deref() != Ok("1") {
        for pixel in pixels.chunks_exact_mut(4) {
            pixel[3] = 255;
        }
    }
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
        writer
            .write_image_data(&pixels)
            .map_err(|error| error.to_string())?;
    }
    Ok(encoded)
}

pub(crate) fn attach_commands(commands: &mpsc::Sender<Command>) {
    attach_commands_with_expected_viewport(commands, None);
}

fn attach_commands_with_expected_viewport(
    commands: &mpsc::Sender<Command>,
    expected_viewport: Option<Arc<Mutex<Viewport>>>,
) {
    let resize_commands = commands.clone();
    devtools::set_device_metrics_handler(Arc::new(move |metrics| {
        let scale = metrics.device_scale_factor;
        let viewport = crate::engine::Viewport {
            width: (f64::from(metrics.width) * scale).round() as u32,
            height: (f64::from(metrics.height) * scale).round() as u32
                + (crate::engine::TOOLBAR_HEIGHT * scale).round() as u32,
            scale,
        };
        if !viewport.valid() {
            return Err("device viewport exceeds render target limits".into());
        }
        if let Some(expected_viewport) = &expected_viewport {
            *expected_viewport.lock().unwrap() = viewport;
        }
        resize_commands
            .send(Command::Resize(ResizeRequest::new(
                viewport,
                Instant::now(),
            )))
            .map_err(|error| error.to_string())
    }));
    let navigate_commands = commands.clone();
    devtools::set_navigate_handler(Arc::new(move |url| {
        navigate_commands
            .send(Command::Navigate(url))
            .map_err(|error| error.to_string())
    }));
    let reload_commands = commands.clone();
    devtools::set_reload_handler(Arc::new(move || {
        reload_commands
            .send(Command::Reload)
            .map_err(|error| error.to_string())
    }));
    let evaluate_commands = commands.clone();
    devtools::set_evaluate_handler(Arc::new(move |source| {
        let (reply, result) = mpsc::channel();
        evaluate_commands
            .send(Command::Evaluate { source, reply })
            .map_err(|error| error.to_string())?;
        result
            .recv_timeout(Duration::from_secs(10))
            .map_err(|error| error.to_string())?
    }));
    let wheel_commands = commands.clone();
    devtools::set_wheel_handler(Arc::new(move |wheel| {
        use interaction::input_event::EventModifiers;
        use interaction::input_event::Offset;
        let phase = match wheel.phase.as_str() {
            "began" => winit::event::TouchPhase::Started,
            "changed" | "none" => winit::event::TouchPhase::Moved,
            "ended" => winit::event::TouchPhase::Ended,
            "cancelled" => winit::event::TouchPhase::Cancelled,
            _ => return Err("Unsupported wheel phase".into()),
        };
        let scale = wheel.scale_factor.unwrap_or(1.0);
        let platform_metadata = match (
            wheel.timestamp_seconds,
            wheel.platform_phase,
            wheel.momentum_phase,
        ) {
            (Some(timestamp_seconds), Some(phase), Some(momentum_phase)) => {
                Some(winit::event::MacOSMouseWheelMetadata {
                    timestamp_seconds,
                    phase,
                    momentum_phase,
                    is_continuous: Some(wheel.precise),
                    point_delta_x: None,
                    point_delta_y: None,
                    cocoa_delta_x: -wheel.delta_x,
                    cocoa_delta_y: -wheel.delta_y,
                    precise: wheel.precise,
                    position_x: wheel.x,
                    position_y: wheel.y,
                    scale_factor: scale,
                })
            }
            (None, None, None) => None,
            _ => {
                return Err(
                    "platform timestamp, phase and momentum phase must be provided together".into(),
                )
            }
        };
        // Both synthetic control input and captured platform replay enter before
        // the common phase handler, rails filter and frame-aligned queue.
        let sample = crate::input::WheelSample {
            delta: winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(
                -wheel.delta_x * scale,
                -wheel.delta_y * scale,
            )),
            phase,
            macos_metadata: platform_metadata,
            position: Offset {
                x: wheel.x,
                y: wheel.y,
            },
            modifiers: EventModifiers::default(),
            scale,
            observed_at: Instant::now(),
        };
        wheel_commands
            .send(Command::Wheel(sample))
            .map_err(|error| error.to_string())
    }));
    let mouse_commands = commands.clone();
    devtools::set_mouse_handler(Arc::new(move |mouse| {
        use interaction::input_event::Offset;
        use interaction::input_event::{InputEvent, MouseButton, MouseEvent, MouseEventType};
        let button = match mouse.button.as_str() {
            "none" => MouseButton::kNone,
            "left" => MouseButton::kPrimary,
            "right" => MouseButton::kSecondary,
            "middle" => MouseButton::kMiddle,
            _ => return Err("Unsupported mouse button".into()),
        };
        let kinds: &[MouseEventType] = match mouse.event_type.as_str() {
            "mouseMoved" => &[MouseEventType::kMove],
            "mousePressed" => &[MouseEventType::kDown],
            "mouseReleased" => &[MouseEventType::kUp],
            _ => return Err("Unsupported mouse event type".into()),
        };
        let queued_at = Instant::now();
        for &kind in kinds {
            mouse_commands
                .send(Command::QueuedInput {
                    input: InputEvent::Mouse(MouseEvent {
                        r#type: kind,
                        button: if kind == MouseEventType::kMove {
                            MouseButton::kNone
                        } else {
                            button
                        },
                        click_count: i32::try_from(mouse.click_count).unwrap_or(i32::MAX),
                        // CDP mouseMoved keeps `button` at none while
                        // `buttons` carries the currently depressed buttons.
                        // Preserve that distinction so headless input can
                        // exercise the native drag/selection path.
                        buttons: if matches!(
                            kind,
                            MouseEventType::kDown | MouseEventType::kMove
                        ) {
                            button.ButtonsMask()
                        } else {
                            0
                        },
                        // Both Page targets use local CSS coordinates. The
                        // native queue uses full-window CSS coordinates.
                        position: Offset {
                            x: mouse.x,
                            y: mouse.y
                                + if mouse.target_id == 1 {
                                    crate::chrome::HEIGHT
                                } else {
                                    0.0
                                },
                        },
                        ..Default::default()
                    }),
                    queued_at,
                    samples: 1,
                })
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }));
    let text_commands = commands.clone();
    devtools::set_insert_text_handler(Arc::new(move |text| {
        use interaction::input_event::{InputEvent, TextInputEvent};
        text_commands
            .send(Command::QueuedInput {
                input: InputEvent::TextInput(TextInputEvent {
                    text,
                    ..Default::default()
                }),
                queued_at: Instant::now(),
                samples: 1,
            })
            .map_err(|error| error.to_string())
    }));
    let composition_commands = commands.clone();
    devtools::set_ime_composition_handler(Arc::new(move |composition| {
        use interaction::input_event::{
            CompositionEvent, CompositionEventType, InputEvent,
        };

        // CDP already uses Blink's UTF-16 composition-selection contract.
        let selection = Some((composition.selection_start, composition.selection_end));
        composition_commands
            .send(Command::QueuedInput {
                input: InputEvent::Composition(CompositionEvent {
                    r#type: CompositionEventType::kUpdate,
                    data: composition.text,
                    selection,
                    ..Default::default()
                }),
                queued_at: Instant::now(),
                samples: 1,
            })
            .map_err(|error| error.to_string())
    }));
    let key_commands = commands.clone();
    devtools::set_key_handler(Arc::new(move |key| {
        use interaction::input_event::{
            DomCodeForKey, EventModifiers, InputEvent, KeyEvent, KeyEventType, LegacyKeyCode,
        };
        let modifiers = EventModifiers {
            alt: key.modifiers & 1 != 0,
            control: key.modifiers & 2 != 0,
            meta: key.modifiers & 4 != 0,
            shift: key.modifiers & 8 != 0,
        };
        let combined = key.event_type == "keyDown";
        let queued_at = Instant::now();
        let mut input = KeyEvent {
            is_standalone_char: key.event_type == "char",
            timestamp: Some(queued_at),
                    edit_commands: key.commands.iter().map(|command|interaction::default::core::editing::commands::editor_command_table::EditingCommandType::FromCommandName(&command.encode_utf16().collect::<Vec<_>>())).collect(),
                    r#type: match key.event_type.as_str() {
                        "keyUp" => KeyEventType::kUp,
                        "char" => KeyEventType::kChar,
                        _ => KeyEventType::kDown,
                    },
                    code: if key.code.is_empty() {
                        DomCodeForKey(&key.key)
                    } else {
                        key.code
                    },
                    key_code: if key.windows_virtual_key_code == 0 {
                        LegacyKeyCode(&key.key)
                    } else {
                        key.windows_virtual_key_code
                    },
                    location: key.location,
                    key: key.key,
                    text: key.text,
                    unmodified_text: key.unmodified_text,
                    native_key_code: key.native_virtual_key_code,
                    is_system_key: key.is_system_key,
                    modifiers,
                    repeat: key.repeat,
                    ..Default::default()
                };
        if input.r#type == KeyEventType::kChar {
            input.edit_commands.clear();
        }
        let character = (combined && !input.text.is_empty()).then(|| {
            let mut event = input.clone();
            event.r#type = KeyEventType::kChar;
            event.edit_commands.clear();
            event
        });
        key_commands
            .send(Command::QueuedInput {
                input: InputEvent::Key(input),
                queued_at,
                samples: 1,
            })
            .map_err(|error| error.to_string())?;
        if let Some(character) = character {
            key_commands
                .send(Command::QueuedInput {
                    input: InputEvent::Key(character),
                    queued_at,
                    samples: 1,
                })
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }));
}
