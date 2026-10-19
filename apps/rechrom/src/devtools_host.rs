//! Shared DevTools attachment for native and displayless application hosts.
//!
//! Both hosts expose the same page commands and the same completed Renderer
//! pixels. The transport remains the existing local HTTP control endpoint.
use crate::engine::{Command, FrameMailbox};
use std::{
    io,
    sync::{mpsc, Arc},
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
        let frame = mailbox.lock().unwrap();
        let frame = frame.as_ref().ok_or("No completed frame")?;
        let surface = match target_id {
            1 => &frame.content,
            2 => &frame.toolbar,
            _ => return Err("target_id must be 1 (page) or 2 (browser toolbar)".into()),
        };
        let (width, height) = surface.size();
        if crate::engine::memory_diagnostics_enabled() {
            eprintln!(
                "pipeline-capture time_ns={} target={} provenance={:?}",
                crate::engine::diagnostic_time_ns(),
                target_id,
                frame.provenance
            );
        }
        // Page.captureScreenshot returns the composited page against its
        // default canvas background. Keep the protocol image opaque like
        // Chromium unless transparent-background emulation is explicitly
        // added later. Some otherwise blank tiles retain zero alpha in the
        // internal surface even though native presentation is opaque.
        let mut pixels = surface.pixels().to_vec();
        for pixel in pixels.chunks_exact_mut(4) {
            pixel[3] = 255;
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
    }));
}

pub(crate) fn attach_commands(commands: &mpsc::Sender<Command>) {
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
        resize_commands
            .send(Command::Resize(viewport))
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
        use interaction::input_event::{InputEvent, ScrollGranularity, WheelEvent, WheelPhase};
        use layoutng_assembly::internal::layout_input::Offset;
        let phase = match wheel.phase.as_str() {
            "began" => WheelPhase::kBegan,
            "changed" => WheelPhase::kChanged,
            "ended" => WheelPhase::kEnded,
            "cancelled" => WheelPhase::kCancelled,
            "none" => WheelPhase::kNone,
            _ => return Err("Unsupported wheel phase".into()),
        };
        // Local synthetic input shares the native dispatch queue
        // and window coordinates, without fabricating NSEvent time.
        let input = InputEvent::Wheel(WheelEvent {
            position: Offset {
                x: wheel.x,
                y: wheel.y,
            },
            delta: Offset {
                x: wheel.delta_x,
                y: wheel.delta_y,
            },
            phase,
            native: None,
            delta_units: if wheel.precise {
                ScrollGranularity::kScrollByPrecisePixel
            } else {
                ScrollGranularity::kScrollByPixel
            },
            ..Default::default()
        });
        wheel_commands
            .send(Command::QueuedInput {
                input,
                queued_at: Instant::now(),
                samples: 1,
            })
            .map_err(|error| error.to_string())
    }));
    let mouse_commands = commands.clone();
    devtools::set_mouse_handler(Arc::new(move |mouse| {
        use interaction::input_event::{InputEvent, MouseButton, MouseEvent, MouseEventType};
        use layoutng_assembly::internal::layout_input::Offset;
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
            "mouseReleased" if mouse.click_count == 2 => &[
                MouseEventType::kUp,
                MouseEventType::kClick,
                MouseEventType::kDoubleClick,
            ],
            "mouseReleased" => &[MouseEventType::kUp, MouseEventType::kClick],
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
    let key_commands = commands.clone();
    devtools::set_key_handler(Arc::new(move |key| {
        use interaction::input_event::{EventModifiers, InputEvent, KeyEvent, KeyEventType};
        let down = key.event_type != "keyUp";
        let modifiers = EventModifiers {
            alt: key.modifiers & 1 != 0,
            control: key.modifiers & 2 != 0,
            meta: key.modifiers & 4 != 0,
            shift: key.modifiers & 8 != 0,
        };
        // Native input strips control characters and command-modified text;
        // Enter is a key/default action, not an inserted carriage return.
        let text = if down && !modifiers.control && !modifiers.meta {
            key.text.chars().filter(|c| !c.is_control()).collect()
        } else {
            String::new()
        };
        key_commands
            .send(Command::QueuedInput {
                input: InputEvent::Key(KeyEvent {
                    r#type: if down {
                        KeyEventType::kDown
                    } else {
                        KeyEventType::kUp
                    },
                    key: key.key,
                    text,
                    modifiers,
                    repeat: key.repeat,
                    ..Default::default()
                }),
                queued_at: Instant::now(),
                samples: 1,
            })
            .map_err(|error| error.to_string())
    }));
}
