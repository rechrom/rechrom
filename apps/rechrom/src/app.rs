use crate::window_surface::WindowSurface;
use crate::{
    crash_report::{CrashReport, CrashReportSink, JsonCrashReportSink},
    engine::{self, Command, FatalError, Output, UserEvent, Viewport},
    input::InputState,
    options::Options,
};
use std::{
    io,
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

pub fn run(options: Options) -> Result<(), Box<dyn std::error::Error>> {
    let working_directory = std::env::current_dir()?;
    let current_location = options.address.clone();
    let crash_reports: Option<Box<dyn CrashReportSink>> = (!options.headless).then(|| {
        Box::new(JsonCrashReportSink::new(
            working_directory.join("rechrom-crash-reports"),
        )) as Box<dyn CrashReportSink>
    });
    browser_tracing::register_target(1, "Page", &options.address);
    browser_tracing::register_target(2, "Browser toolbar", "browser://toolbar");
    let devtools_url = devtools::serve(options.devtools_port).or_else(|error| {
        if error.kind() == io::ErrorKind::AddrInUse {
            devtools::serve(0)
        } else {
            Err(error)
        }
    })?;
    println!("devtools-url\t{devtools_url}");
    // winit owns the native application's main thread (required on macOS).
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    let mailbox: engine::FrameMailbox = Arc::new(Mutex::new(None));
    let screenshot_mailbox = mailbox.clone();
    devtools::set_screenshot_handler(Arc::new(move |target_id| {
        let frame = screenshot_mailbox.lock().unwrap();
        let frame = frame.as_ref().ok_or("No completed frame")?;
        let surface = match target_id {
            1 => &frame.content,
            2 => &frame.toolbar,
            _ => return Err("target_id must be 1 (page) or 2 (browser toolbar)".into()),
        };
        let (width, height) = surface.size();
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
    let output = Output {
        mailbox: mailbox.clone(),
        notify: Arc::new(move |event| {
            let _ = proxy.send_event(event);
        }),
    };
    let mut app = App {
        devtools_url,
        options,
        working_directory,
        current_location,
        crash_reports,
        output,
        window: None,
        commands: None,
        input: InputState::default(),
        deadline: None,
        error: None,
        presented: 0,
        drag_regions: Vec::new(),
        drag_frame_sequence: 0,
        drag_geometry_valid: std::cell::Cell::new(false),
        last_requested_display: std::cell::Cell::new(None),
    };
    event_loop.run_app(&mut app)?;
    if let Some(error) = app.error {
        return Err(io::Error::other(error).into());
    }
    Ok(())
}
struct App {
    devtools_url: String,
    options: Options,
    working_directory: std::path::PathBuf,
    current_location: String,
    crash_reports: Option<Box<dyn CrashReportSink>>,
    output: Output,
    window: Option<Arc<Window>>,
    commands: Option<mpsc::Sender<Command>>,
    input: InputState,
    deadline: Option<Instant>,
    error: Option<String>,
    presented: u64,
    drag_regions: Vec<crate::chrome::DragRegion>,
    drag_frame_sequence: u64,
    drag_geometry_valid: std::cell::Cell<bool>,
    last_requested_display: std::cell::Cell<Option<u32>>,
}
impl App {
    fn open_devtools(&self) {
        if let Err(error) = crate::devtools_window::show(&self.devtools_url) {
            eprintln!("DevTools: {error}");
        }
    }
    fn fail(
        &mut self,
        event_loop: &ActiveEventLoop,
        source: &'static str,
        error: impl std::fmt::Display,
    ) {
        let message = error.to_string();
        eprintln!(
            "rechrom-fatal\t{}",
            serde_json::json!({
                "source": source,
                "message": &message,
                "url": &self.current_location,
            })
        );
        if let Some(sink) = &self.crash_reports {
            match sink.write(CrashReport {
                source,
                message: &message,
                url: &self.current_location,
                working_directory: &self.working_directory,
            }) {
                Ok(stored) => {
                    eprintln!("rechrom-crash-report\t{}", stored.latest_path.display());
                    eprintln!("rechrom-crash-archive\t{}", stored.archived_path.display());
                }
                Err(report_error) => {
                    eprintln!("rechrom-crash-report-error\t{report_error}");
                }
            }
        }
        self.error = Some(format!("{source}: {message}"));
        event_loop.exit();
    }
    fn viewport(&self) -> Viewport {
        let window = self.window.as_ref().unwrap();
        let size = window.inner_size();
        Viewport {
            width: size.width,
            height: size.height,
            scale: window.scale_factor(),
        }
    }
    fn resize(&mut self) -> io::Result<()> {
        self.update_display_binding();
        let viewport = self.viewport();
        self.drag_geometry_valid.set(false);
        if viewport.valid() {
            self.send(Command::Resize(viewport));
        }
        // The viewport update schedules its own full layout/paint/presentation.
        // Native exposure redraws are handled separately by RedrawRequested.
        Ok(())
    }
    fn current_display_id(&self) -> Option<u32> {
        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::MonitorHandleExtMacOS;
            self.window
                .as_ref()
                .and_then(|window| window.current_monitor())
                .map(|monitor| monitor.native_id())
        }
        #[cfg(not(target_os = "macos"))]
        {
            None
        }
    }
    fn update_display_binding(&self) {
        if let Some(display_id) = self.current_display_id() {
            if self.last_requested_display.replace(Some(display_id)) != Some(display_id) {
                self.send(Command::VSyncDisplayChanged(display_id));
            }
        }
    }
    fn send(&self, command: Command) {
        let changes_chrome = matches!(
            &command,
            Command::NewTab
                | Command::CloseActiveTab
                | Command::CycleTab(_)
                | Command::Reload
                | Command::Back
                | Command::Forward
                | Command::FocusAddress
                | Command::SetActive(false)
        ) || matches!(&command, Command::Input(interaction::input_event::InputEvent::Mouse(event))
                if event.r#type == interaction::input_event::MouseEventType::kDown && event.position.y < crate::chrome::HEIGHT)
            || matches!(&command, Command::Input(interaction::input_event::InputEvent::Key(event)) if event.key == "Enter");
        if changes_chrome {
            self.drag_geometry_valid.set(false);
        }
        if let Some(sender) = &self.commands {
            let command = match command {
                Command::Input(input)
                    if std::env::var_os("BROWSER_APP_TRACE_INPUT").is_some()
                        || std::env::var_os("BROWSER_PROFILE_INPUT").is_some()
                        || browser_tracing::enabled() =>
                {
                    Command::QueuedInput {
                        input,
                        queued_at: Instant::now(),
                        samples: 1,
                    }
                }
                command => command,
            };
            let _ = sender.send(command);
        }
    }
}
impl ApplicationHandler<UserEvent> for App {
    fn open_urls(&mut self, _: &ActiveEventLoop, urls: Vec<String>) {
        let mut opened = 0;
        for raw in urls {
            let address = match crate::options::normalize_address(&raw) {
                Ok(address) => address,
                Err(error) => {
                    eprintln!("open-url: {error}");
                    continue;
                }
            };
            if opened > 0 {
                self.send(Command::NewTab);
            }
            self.send(Command::Navigate(address));
            opened += 1;
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            // DevTools Page.captureScreenshot addresses the content target, so
            // headless --height follows CDP viewport semantics.  The hidden
            // host still owns a toolbar Page; allocate its height outside the
            // requested content viewport instead of silently shrinking
            // window.innerHeight and changing responsive layout.
            let window_height = self.options.height
                + if self.options.headless {
                    crate::chrome::HEIGHT as u32
                } else {
                    0
                };
            let attributes = Window::default_attributes()
                .with_title("Rechrom")
                .with_inner_size(LogicalSize::new(self.options.width, window_height))
                .with_min_inner_size(LogicalSize::new(320, 200))
                .with_visible(!self.options.headless);
            #[cfg(target_os = "macos")]
            let attributes = {
                use winit::platform::macos::WindowAttributesExtMacOS;
                attributes
                    .with_fullsize_content_view(true)
                    .with_titlebar_transparent(true)
                    .with_title_hidden(true)
            };
            match event_loop.create_window(attributes) {
                Ok(window) => {
                    window.set_ime_allowed(true);
                    self.window = Some(Arc::new(window));
                }
                Err(error) => {
                    self.fail(event_loop, "app", error);
                    return;
                }
            }
        }
        if self.commands.is_none() {
            let window = self.window.as_ref().unwrap().clone();
            let surface = match WindowSurface::new(window.clone(), window) {
                Ok(surface) => surface,
                Err(error) => {
                    self.fail(event_loop, "app", error);
                    return;
                }
            };
            match engine::spawn(
                self.viewport(),
                self.options.address.clone(),
                self.output.clone(),
                surface,
                self.window
                    .as_ref()
                    .and_then(|window| window.current_monitor())
                    .and_then(|monitor| monitor.refresh_rate_millihertz())
                    .map(|rate| Duration::from_secs_f64(1000.0 / f64::from(rate)))
                    .unwrap_or(Duration::from_secs_f64(1.0 / 60.0)),
                self.current_display_id(),
                crate::presentation_runtime::Config {
                    // Placement is a host policy. Compositor and Display keep
                    // the same message boundary whichever executor is chosen.
                    display_executor: if std::env::var_os("RUST_AI_DISPLAY_WITH_COMPOSITOR")
                        .is_some()
                    {
                        crate::presentation_runtime::DisplayExecutor::CompositorThread
                    } else {
                        crate::presentation_runtime::DisplayExecutor::DedicatedThread
                    },
                    capture_frames: self.options.headless,
                },
            ) {
                Ok(commands) => {
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
                        use interaction::input_event::{
                            InputEvent, ScrollGranularity, WheelEvent, WheelPhase,
                        };
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
                    self.commands = Some(commands);
                    self.last_requested_display.set(self.current_display_id());
                }
                Err(error) => {
                    self.fail(event_loop, "app", error);
                    return;
                }
            }
            self.deadline = self
                .options
                .exit_after
                .map(|duration| Instant::now() + duration);
        }
        if let Err(error) = self.resize() {
            self.fail(event_loop, "app", error);
        }
    }
    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.send(Command::Stop);
        self.commands = None;
    }
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::OpenDevTools => self.open_devtools(),
            UserEvent::CloseWindow => {
                self.send(Command::Stop);
                event_loop.exit();
            }
            UserEvent::FrameReady => {} // Diagnostic mailbox path only.
            UserEvent::ChromeDragRegions {
                viewport,
                frame_sequence,
                regions,
            } => {
                // Geometry is emitted after replay/present and belongs to that
                // submitted frame, never a pending resize or layout prediction.
                if viewport == self.viewport() && frame_sequence > self.drag_frame_sequence {
                    self.drag_frame_sequence = frame_sequence;
                    self.drag_regions = regions;
                    self.drag_geometry_valid.set(true);
                }
            }
            UserEvent::FramePresented(viewport) => {
                self.presented += 1;
                if self.presented == 1 {
                    println!(
                        "window-presented\t{}x{}\tscale={}",
                        viewport.width, viewport.height, viewport.scale
                    );
                }
            }
            UserEvent::Location(address) => {
                browser_tracing::register_target(1, "Page", &address);
                self.current_location.clone_from(&address);
                if let Some(window) = &self.window {
                    window.set_title(&format!("Rechrom — {address}"));
                }
            }
            UserEvent::CursorChanged(cursor) => {
                // A delayed page reply must not restore a content cursor after
                // the pointer has left the view or the window lost activation.
                if let Some(window) = &self.window {
                    if !self.input.pointer_inside() {
                        return;
                    }
                    crate::input::set_cursor(window, cursor);
                    if std::env::var_os("BROWSER_APP_TRACE_INPUT").is_some() {
                        eprintln!("window-cursor {cursor:?}");
                    }
                }
            }
            UserEvent::CaretChanged(rect) => {
                if let (Some(window), Some(rect)) = (&self.window, rect) {
                    window.set_ime_cursor_area(
                        LogicalPosition::new(rect.x, rect.y),
                        LogicalSize::new(rect.width, rect.height),
                    );
                }
            }
            UserEvent::Message(message) => eprintln!("page: {message}"),
            UserEvent::Fatal(FatalError { source, message }) => {
                self.fail(event_loop, source, message)
            }
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else {
            return;
        };
        if window.id() != window_id {
            return;
        }
        if matches!(&event, WindowEvent::KeyboardInput { event, .. }
            if event.state == winit::event::ElementState::Pressed
            && event.logical_key == winit::keyboard::Key::Named(winit::keyboard::NamedKey::F12))
        {
            self.open_devtools();
            return;
        }
        if matches!(
            &event,
            WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            }
        ) && self.input.pointer_inside()
        {
            let point = self.input.cursor_position();
            if self.drag_geometry_valid.get()
                && self
                    .drag_regions
                    .iter()
                    .any(|region| region.contains(point.x, point.y))
            {
                for command in self.input.event(&event, window.scale_factor()) {
                    self.send(command);
                }
                let _ = window.drag_window();
                return;
            }
        }
        match &event {
            WindowEvent::CloseRequested => {
                self.send(Command::Stop);
                event_loop.exit();
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                if let Err(error) = self.resize() {
                    self.fail(event_loop, "app", error);
                }
            }
            WindowEvent::Moved(_) => self.update_display_binding(),
            WindowEvent::RedrawRequested => self.send(Command::Redraw),
            WindowEvent::Focused(active) => {
                for command in self.input.event(&event, window.scale_factor()) {
                    self.send(command);
                }
                if !active {
                    crate::input::set_cursor(window, browser::page::Cursor::kDefault);
                }
                self.send(Command::SetActive(*active));
            }
            _ => {
                if matches!(&event, WindowEvent::CursorLeft { .. }) {
                    crate::input::set_cursor(window, browser::page::Cursor::kDefault);
                }
                let scale = window.scale_factor();
                for command in self.input.event(&event, scale) {
                    self.send(command);
                }
            }
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if let Some(command) = self.input.take_due_wheel_end(now) {
            self.send(command);
        }
        if self.deadline.is_some_and(|deadline| now >= deadline) {
            self.send(Command::Stop);
            event_loop.exit();
            return;
        }
        // Keep the existing app stop deadline while waking for the official
        // pending finger-end timeout even when no more native events arrive.
        let deadline = match (self.deadline, self.input.pending_wheel_end_deadline()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        if let Some(deadline) = deadline {
            event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.send(Command::Stop);
        println!("window-closed\tpresents={}", self.presented);
    }
}
