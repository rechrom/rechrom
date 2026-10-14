//! Displayless host for the same Page, Compositor, Viz and Renderer pipeline.
//!
//! The application owns only an event mailbox and an offscreen render target;
//! no winit event loop, native display connection, or hidden window is created.
use crate::{
    engine::{self, Command, FatalError, Output, UserEvent, Viewport},
    options::Options,
    window_surface::DisplayTarget,
};
use std::{
    io,
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};

pub(crate) fn run(options: Options) -> Result<(), Box<dyn std::error::Error>> {
    let viewport = viewport(options.width, options.height, options.device_scale_factor);
    if !viewport.valid() {
        return Err(
            io::Error::other("headless device viewport exceeds render target limits").into(),
        );
    }
    let _devtools_url = crate::devtools_host::serve(options.devtools_port, &options.address)?;
    let mailbox: engine::FrameMailbox = Arc::new(Mutex::new(None));
    crate::devtools_host::attach_screenshots(mailbox.clone());
    let (events, receiver) = mpsc::channel();
    let output = Output {
        mailbox,
        notify: Arc::new(move |event| {
            let _ = events.send(event);
        }),
    };
    let commands = engine::spawn(
        viewport,
        options.address.clone(),
        output,
        DisplayTarget::Offscreen,
        Duration::from_secs_f64(1.0 / 60.0),
        None,
        crate::app::presentation_config(true),
    )?;
    crate::devtools_host::attach_commands(&commands);
    pump_events(
        receiver,
        commands,
        viewport,
        options.address,
        options.exit_after,
    )?;
    Ok(())
}

fn viewport(width: u32, content_height: u32, scale: f64) -> Viewport {
    // CLI dimensions describe the Page target in CSS pixels. The
    // production embedding still renders its toolbar Page above that target.
    Viewport {
        width: (f64::from(width) * scale).round() as u32,
        height: (f64::from(content_height) * scale).round() as u32
            + (engine::TOOLBAR_HEIGHT * scale).round() as u32,
        scale,
    }
}

struct StopEngine(mpsc::Sender<Command>);

impl Drop for StopEngine {
    fn drop(&mut self) {
        // Use the existing host dispatch lifecycle on every exit, including a
        // timeout, a fatal engine event, and an event-channel disconnection.
        let _ = self.0.send(Command::Stop);
    }
}

fn pump_events(
    receiver: mpsc::Receiver<UserEvent>,
    commands: mpsc::Sender<Command>,
    viewport: Viewport,
    mut current_location: String,
    exit_after: Option<Duration>,
) -> io::Result<()> {
    let _stop = StopEngine(commands);
    let deadline = exit_after.map(|duration| Instant::now() + duration);
    let mut frame_ready = false;
    let result = loop {
        let event = if let Some(deadline) = deadline {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                break Ok(());
            };
            match receiver.recv_timeout(remaining) {
                Ok(event) => event,
                Err(mpsc::RecvTimeoutError::Timeout | mpsc::RecvTimeoutError::Disconnected) => {
                    break Ok(());
                }
            }
        } else {
            match receiver.recv() {
                Ok(event) => event,
                Err(_) => break Ok(()),
            }
        };
        match event {
            UserEvent::CloseWindow => break Ok(()),
            UserEvent::FrameReady => {
                if !frame_ready {
                    frame_ready = true;
                    println!(
                        "headless-frame-ready\t{}x{}\tscale={}",
                        viewport.logical_width(),
                        viewport.content_height(),
                        viewport.scale
                    );
                }
            }
            UserEvent::Location(address) => {
                browser_tracing::register_target(1, "Page", &address);
                current_location = address;
            }
            UserEvent::Message(message) => eprintln!("page: {message}"),
            UserEvent::Fatal(FatalError { source, message }) => {
                eprintln!(
                    "rechrom-fatal\t{}",
                    serde_json::json!({
                        "source": source,
                        "message": &message,
                        "url": &current_location,
                    })
                );
                break Err(io::Error::other(format!("{source}: {message}")));
            }
            // These are native UI effects. Headless never opens a DevTools
            // window or waits for a native presentation acknowledgement.
            UserEvent::OpenDevTools
            | UserEvent::FramePresented(_)
            | UserEvent::ChromeDragRegions { .. }
            | UserEvent::CursorChanged(_)
            | UserEvent::CaretChanged(_) => {}
        }
    };
    println!("headless-closed");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_keeps_toolbar_outside_requested_css_page() {
        for scale in [1.0, 1.25, 2.0] {
            let viewport = viewport(1280, 800, scale);
            assert!(viewport.valid());
            assert_eq!(viewport.logical_width(), 1280.0);
            assert_eq!(viewport.content_height(), 800.0);
            assert_eq!(viewport.scale, scale);
            assert_eq!(
                viewport.height,
                (800.0 * scale) as u32 + viewport.toolbar_pixels()
            );
        }
    }

    #[test]
    fn close_stops_engine_without_waiting_for_a_presented_frame() {
        let (events, receiver) = mpsc::channel();
        let (commands, command_receiver) = mpsc::channel();
        events.send(UserEvent::CloseWindow).unwrap();
        pump_events(
            receiver,
            commands,
            viewport(1280, 800, 1.0),
            "about:blank".into(),
            None,
        )
        .unwrap();
        assert!(matches!(command_receiver.try_recv(), Ok(Command::Stop)));
    }

    #[test]
    fn deadline_stops_engine_even_when_events_are_queued() {
        let (events, receiver) = mpsc::channel();
        let (commands, command_receiver) = mpsc::channel();
        events.send(UserEvent::FrameReady).unwrap();
        pump_events(
            receiver,
            commands,
            viewport(1280, 800, 1.0),
            "about:blank".into(),
            Some(Duration::ZERO),
        )
        .unwrap();
        assert!(matches!(command_receiver.try_recv(), Ok(Command::Stop)));
    }

    #[test]
    fn fatal_event_returns_error_and_stops_engine() {
        let (events, receiver) = mpsc::channel();
        let (commands, command_receiver) = mpsc::channel();
        events
            .send(UserEvent::Fatal(FatalError::new("display", "test failure")))
            .unwrap();
        let error = pump_events(
            receiver,
            commands,
            viewport(1280, 800, 1.0),
            "about:blank".into(),
            None,
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "display: test failure");
        assert!(matches!(command_receiver.try_recv(), Ok(Command::Stop)));
    }

    #[test]
    fn disconnected_events_stop_engine() {
        let (events, receiver) = mpsc::channel();
        let (commands, command_receiver) = mpsc::channel();
        drop(events);
        pump_events(
            receiver,
            commands,
            viewport(1280, 800, 1.0),
            "about:blank".into(),
            None,
        )
        .unwrap();
        assert!(matches!(command_receiver.try_recv(), Ok(Command::Stop)));
    }
}
