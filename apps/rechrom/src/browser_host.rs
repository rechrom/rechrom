//! Browser policy and wiring. This host owns handles, tabs/history, focus and
//! Surface placement; it never calls DOM, JS, lifecycle, paint or raster.
use crate::{
    begin_frame_source,
    engine::{Command, Output, UserEvent, Viewport, TOOLBAR_HEIGHT},
    page_presentation::{Document, Placement, Scene},
    page_runtime::{self as page, Event, Handle, Patch},
    window_runtime,
};
use foundation::begin_frame::BeginFrameSource;
use interaction::input_event::*;
use std::{
    io,
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};
enum Message {
    Host(Command),
    Page(Document, Event),
}
struct Popup {
    page: Handle,
    x: f64,
    width: f64,
    height: f64,
    actions: Vec<String>,
    selected: Option<usize>,
}
struct Host {
    viewport: Viewport,
    output: Output,
    toolbar: Handle,
    tabs: crate::tabs::Tabs<Handle>,
    popup: Option<Popup>,
    popup_generation: u64,
    toolbar_focused: bool,
    in_toolbar: bool,
    active: bool,
    routes: page::FrameRoutes,
    source: Arc<dyn BeginFrameSource>,
    frames: mpsc::Sender<window_runtime::Command>,
    events: Arc<dyn Fn(Document, Event) + Send + Sync>,
    tabstrip: String,
    reload_loading: Option<bool>,
    drag_regions: Vec<crate::chrome::DragRegion>,
    wheel_input: crate::input::InputState,
}
/// Product composition supplies a separate executor for each Page domain.
fn page_threads() -> io::Result<browser::page::PageThreads> {
    let inline = std::env::var("RECHROM_EXECUTOR").as_deref() == Ok("inline");
    let (executor, compositor_executor, raster_executor): (
        Arc<dyn executor::Executor>,
        Arc<dyn executor::Executor>,
        Arc<dyn executor::Executor>,
    ) = if inline {
        let executor = executor::inline();
        (executor.clone(), executor.clone(), executor)
    } else {
        // Match Chromium's scheduling boundary: impl input and frame
        // scheduling have a dedicated sequence, while raster playback uses a
        // worker pool and decode/compile work has separate capacity. Keep the
        // raster throughput previously available to each Page. The extra
        // sequencing lane is reserved for input and frame scheduling.
        (
            Arc::new(executor::PoolExecutor::new(
                "page-work",
                16 * 1024 * 1024,
                2,
            )?),
            Arc::new(executor::PoolExecutor::new(
                "page-compositor",
                16 * 1024 * 1024,
                1,
            )?),
            Arc::new(executor::PoolExecutor::new(
                "page-raster",
                16 * 1024 * 1024,
                8,
            )?),
        )
    };
    Ok(browser::page::PageThreads {
        raster: true,
        decode: true,
        scriptCompile: true,
        executor,
        compositor_executor,
        raster_executor,
    })
}

pub(crate) fn spawn(
    viewport: Viewport,
    address: String,
    output: Output,
    target: crate::window_surface::DisplayTarget,
    interval: Duration,
    display_id: Option<u32>,
    config: crate::presentation_runtime::Config,
) -> io::Result<mpsc::Sender<Command>> {
    if !viewport.valid() {
        return Err(io::Error::other("invalid window viewport"));
    }
    let native = if target.is_native() {
        begin_frame_source::create(interval, display_id)?
    } else {
        begin_frame_source::create_offscreen(interval)?
    };
    let routes: page::FrameRoutes = Arc::new(Mutex::new(Default::default()));
    let ticks = routes.clone();
    let compositor_source = native.GetFrameSource(1)?;
    compositor_source.SetClient(Arc::new(move |frame| {
        let targets: Vec<_> = ticks
            .lock()
            .unwrap()
            .values()
            .filter(|route| route.visible())
            .cloned()
            .collect();
        for route in targets {
            route.begin_frame(frame);
        }
    }))?;
    // Input is routed on receipt; compositor pulses and window draw deadlines
    // are separate phases of the common physical clock.
    native.GetFrameSource(0)?.SetClient(Arc::new(|_| {}))?;
    let frames = window_runtime::spawn(config, target, output.clone(), native.GetFrameSource(2)?)?;
    let (sender, receiver) = mpsc::channel();
    let (messages, inbox) = mpsc::channel();
    let events_sender = messages.clone();
    let events: Arc<dyn Fn(Document, Event) + Send + Sync> = Arc::new(move |document, event| {
        let _ = events_sender.send(Message::Page(document, event));
    });
    std::thread::Builder::new()
        .name("browser-host-input".into())
        .spawn(move || {
            while let Ok(command) = receiver.recv() {
                let stop = matches!(command, Command::Stop);
                if messages.send(Message::Host(command)).is_err() || stop {
                    break;
                }
            }
        })?;
    let source: Arc<dyn BeginFrameSource> = compositor_source;
    let toolbar = page::spawn(page::Params {
        document: (0, 0),
        url: "browser://toolbar".into(),
        viewport: Viewport {
            height: viewport.toolbar_pixels(),
            ..viewport
        },
        request: url_loader::URLRequest {
            url: crate::engine::data_url(&crate::chrome::document()),
            ..Default::default()
        },
        scripting: false,
        threads: page_threads()?,
        transparent: false,
        controls: true,
        source: source.clone(),
        routes: routes.clone(),
        frames: frames.clone(),
        events: events.clone(),
    })?;
    std::thread::Builder::new()
        .name("browser-host".into())
        .spawn(move || {
            let mut host = Host {
                viewport,
                output,
                toolbar,
                tabs: crate::tabs::Tabs::new(),
                popup: None,
                popup_generation: 0,
                toolbar_focused: false,
                in_toolbar: false,
                active: true,
                routes,
                source,
                frames,
                events,
                tabstrip: String::new(),
                reload_loading: None,
                drag_regions: vec![],
                wheel_input: crate::input::InputState::default(),
            };
            if let Err(error) = host.navigate(
                url_loader::URLRequest {
                    url: address,
                    ..Default::default()
                },
                true,
            ) {
                (host.output.notify)(UserEvent::Message(error.to_string()));
            }
            loop {
                let message = if let Some(deadline) = host.wheel_input.pending_wheel_end_deadline()
                {
                    match inbox.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                        Ok(message) => message,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            if let Some(command) =
                                host.wheel_input.take_due_wheel_end(Instant::now())
                            {
                                if let Err(error) = host.command(command) {
                                    (host.output.notify)(UserEvent::Message(error.to_string()));
                                }
                            }
                            continue;
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                } else {
                    match inbox.recv() {
                        Ok(message) => message,
                        Err(_) => break,
                    }
                };
                if matches!(&message, Message::Host(Command::Stop)) {
                    break;
                }
                let result = match message {
                    Message::Host(command) => host.command(command),
                    Message::Page(document, event) => host.event(document, event),
                };
                if let Err(error) = result {
                    (host.output.notify)(UserEvent::Message(error.to_string()));
                }
            }
            let _ = host.frames.send(window_runtime::Command::Display(
                crate::display::Message::Stop,
            ));
        })?;
    Ok(sender)
}
impl Host {
    fn content_viewport(&self) -> Viewport {
        Viewport {
            height: self
                .viewport
                .height
                .saturating_sub(self.viewport.toolbar_pixels())
                .max(1),
            ..self.viewport
        }
    }
    fn make_page(
        &self,
        document: Document,
        request: url_loader::URLRequest,
        url: String,
        controls: bool,
        transparent: bool,
    ) -> io::Result<Handle> {
        page::spawn(page::Params {
            document,
            url,
            viewport: self.content_viewport(),
            request,
            scripting: !controls,
            threads: page_threads()?,
            controls,
            transparent,
            source: self.source.clone(),
            routes: self.routes.clone(),
            frames: self.frames.clone(),
            events: self.events.clone(),
        })
    }
    fn scene(&self) {
        let toolbar_height = self.viewport.toolbar_pixels();
        let content = crate::page_presentation::DeviceRect::new(
            0,
            toolbar_height as i32,
            self.viewport.width,
            self.viewport.height.saturating_sub(toolbar_height).max(1),
        );
        let mut placements = vec![Placement {
            document: self.toolbar.document,
            destination: crate::page_presentation::DeviceRect::new(
                0,
                0,
                self.viewport.width,
                toolbar_height,
            ),
        }];
        let mut live_documents = vec![self.toolbar.document];
        for tab in self.tabs.ordered() {
            if let Some(page) = &tab.page {
                live_documents.push(page.document);
            }
        }
        if let Some(page) = &self.tabs.active.page {
            placements.push(Placement {
                document: page.document,
                destination: content,
            });
        }
        if let Some(popup) = &self.popup {
            placements.push(Placement {
                document: popup.page.document,
                destination: content,
            });
            live_documents.push(popup.page.document);
        }
        let _ = self.frames.send(window_runtime::Command::Display(
            crate::display::Message::Scene(Scene {
                viewport: self.viewport,
                placements,
                live_documents,
                metadata: self.drag_regions.clone(),
            }),
        ));
        // Cross-thread frame delivery starts only after embedding metadata is
        // enqueued, so a fast new producer cannot be pruned as an orphan.
        self.toolbar.visible(true);
        if let Some(page) = &self.tabs.active.page {
            page.visible(true);
        }
        if let Some(popup) = &self.popup {
            popup.page.visible(true);
        }
    }
    fn active_pages(&self) {
        self.toolbar
            .send(page::Command::Active(self.active && self.toolbar_focused));
        if let Some(page) = &self.tabs.active.page {
            page.send(page::Command::Active(self.active && !self.toolbar_focused));
        }
        self.toolbar.patch(Patch::Attribute {
            id: "omnibox".into(),
            name: "class".into(),
            value: Some(if self.toolbar_focused { "focused" } else { "" }.into()),
        });
    }
    fn address(&self) {
        self.toolbar.patch(Patch::Value {
            id: "address".into(),
            value: self.tabs.active.location.clone(),
        });
    }
    fn refresh(&mut self) {
        let html = crate::chrome::tab_strip(&self.tabs);
        if html != self.tabstrip {
            self.toolbar.patch(Patch::Attribute {
                id: "tabs".into(),
                name: "style".into(),
                value: Some(format!(
                    "flex-basis:{}px",
                    self.tabs.ordered().len() * 238 - 6
                )),
            });
            self.toolbar.patch(Patch::Html {
                id: "tabs".into(),
                value: html.clone(),
            });
            self.tabstrip = html;
        }
        for (id, enabled) in [
            ("back", self.tabs.active.history_index > 0),
            (
                "forward",
                self.tabs.active.history_index + 1 < self.tabs.active.history.len(),
            ),
        ] {
            self.toolbar.patch(Patch::Attribute {
                id: id.into(),
                name: "disabled".into(),
                value: (!enabled).then(String::new),
            });
        }
        let loading = self.tabs.active.page.as_ref().is_some_and(|p| p.loading);
        if self.reload_loading != Some(loading) {
            self.reload_loading = Some(loading);
            self.toolbar.patch(Patch::Html {
                id: "reload".into(),
                value: crate::chrome::reload_icon(loading).into(),
            });
        }
    }
    fn navigate(&mut self, mut request: url_loader::URLRequest, record: bool) -> io::Result<()> {
        let location = crate::options::normalize_address(&request.url)?;
        request.url = match location.as_str() {
            "about:home" => crate::engine::data_url(crate::engine::HOME),
            "about:blank" => crate::engine::data_url("<!doctype html><html><body></body></html>"),
            _ => location.clone(),
        };
        let generation = self.tabs.active.document_generation + 1;
        let page = self.make_page(
            (self.tabs.active.id, generation),
            request,
            location.clone(),
            false,
            false,
        )?;
        self.tabs.active.document_generation = generation;
        self.tabs.active.page = Some(page);
        self.tabs.active.viewport = Some(self.viewport);
        self.tabs.active.location = location.clone();
        self.tabs.active.title = location_title(&location);
        if record {
            let tab = &mut self.tabs.active;
            tab.history.truncate(tab.history_index + 1);
            tab.history.push(location.clone());
            tab.history_index = tab.history.len() - 1;
        }
        self.popup = None;
        self.toolbar_focused = false;
        self.active_pages();
        self.address();
        self.refresh();
        self.scene();
        (self.output.notify)(UserEvent::Location(location));
        Ok(())
    }
    fn show(&mut self) {
        self.popup = None;
        self.toolbar_focused = false;
        if let Some(page) = &self.tabs.active.page {
            if self.tabs.active.viewport != Some(self.viewport) {
                page.resize(self.content_viewport());
            }
            page.visible(true);
        }
        self.tabs.active.viewport = Some(self.viewport);
        self.active_pages();
        self.address();
        self.refresh();
        self.scene();
        (self.output.notify)(UserEvent::Location(self.tabs.active.location.clone()));
    }
    fn hide(&self) {
        if let Some(page) = &self.tabs.active.page {
            page.visible(false);
            page.send(page::Command::Active(false));
        }
    }
    fn activate(&mut self, id: u64) {
        if id == self.tabs.active.id {
            return;
        }
        if self.tabs.background.iter().any(|t| t.id == id) {
            self.hide();
            self.tabs.activate(id);
            self.show();
        }
    }
    fn new_tab(&mut self, request: url_loader::URLRequest, focus: bool) -> io::Result<()> {
        crate::options::normalize_address(&request.url)?;
        self.hide();
        self.tabs.add();
        self.navigate(request, true)?;
        if focus {
            self.focus_address();
        }
        Ok(())
    }
    fn close(&mut self, id: u64) {
        if self.tabs.is_only(id) {
            (self.output.notify)(UserEvent::CloseWindow);
            return;
        }
        if self.tabs.close(id) {
            self.show();
        } else {
            self.refresh();
            self.scene();
        }
    }
    fn history(&mut self, delta: isize) -> io::Result<()> {
        let next = self.tabs.active.history_index as isize + delta;
        if next >= 0 && (next as usize) < self.tabs.active.history.len() {
            let url = self.tabs.active.history[next as usize].clone();
            self.tabs.active.history_index = next as usize;
            self.navigate(
                url_loader::URLRequest {
                    url,
                    ..Default::default()
                },
                false,
            )?;
        }
        Ok(())
    }
    fn focus_address(&mut self) {
        self.popup = None;
        self.toolbar_focused = true;
        self.active_pages();
        self.toolbar.patch(Patch::Focus {
            id: "address".into(),
            select_all: true,
        });
        self.scene();
    }
    fn popup(&mut self, kind: &str) -> io::Result<()> {
        if self.popup.take().is_some() {
            self.scene();
            return Ok(());
        }
        let rows = match kind {
            "tabsearch" => self
                .tabs
                .ordered()
                .iter()
                .map(|t| (format!("activate-{}", t.id), t.title.clone(), String::new()))
                .collect(),
            "site" => vec![(
                "info".into(),
                self.tabs.active.location.clone(),
                String::new(),
            )],
            _ => vec![
                ("newtab".into(), "New tab".into(), "⌘T".into()),
                ("reload".into(), "Reload".into(), "⌘R".into()),
                ("devtools".into(), "Developer tools".into(), "F12".into()),
                ("home".into(), "Home".into(), String::new()),
                ("close".into(), "Close tab".into(), "⌘W".into()),
            ],
        };
        let width = 280.0_f64.min(self.viewport.logical_width() - 16.0);
        let x = match kind {
            "tabsearch" => 92.0_f64.min(self.viewport.logical_width() - width - 8.0),
            "site" => 121.0_f64.min(self.viewport.logical_width() - width - 8.0),
            _ => self.viewport.logical_width() - width - 8.0,
        };
        self.popup_generation += 1;
        let page = self.make_page(
            (u64::MAX, self.popup_generation),
            url_loader::URLRequest {
                url: crate::engine::data_url(&crate::chrome::popup_document(&rows, width, x)),
                ..Default::default()
            },
            "browser://popup".into(),
            true,
            true,
        )?;
        self.popup = Some(Popup {
            page,
            x,
            width,
            height: (rows.len() as f64 * 32.0 + 18.0).min(self.viewport.content_height()),
            actions: rows.iter().map(|r| r.0.clone()).collect(),
            selected: None,
        });
        self.scene();
        Ok(())
    }
    fn action(&mut self, id: &str) -> io::Result<()> {
        match id {
            "back" => self.history(-1),
            "forward" => self.history(1),
            "reload" if self.tabs.active.page.as_ref().is_some_and(|p| p.loading) => {
                if let Some(page) = &self.tabs.active.page {
                    page.send(page::Command::StopLoading);
                }
                Ok(())
            }
            "reload" => self.navigate(
                url_loader::URLRequest {
                    url: self.tabs.active.location.clone(),
                    ..Default::default()
                },
                false,
            ),
            "home" => self.navigate(
                url_loader::URLRequest {
                    url: "about:home".into(),
                    ..Default::default()
                },
                true,
            ),
            "newtab" => self.new_tab(
                url_loader::URLRequest {
                    url: "about:home".into(),
                    ..Default::default()
                },
                true,
            ),
            "close" => {
                self.close(self.tabs.active.id);
                Ok(())
            }
            "devtools" => {
                self.popup = None;
                self.scene();
                (self.output.notify)(UserEvent::OpenDevTools);
                Ok(())
            }
            "menu" | "tabsearch" | "site" => self.popup(id),
            id if id.starts_with("close-") => {
                if let Ok(id) = id[6..].parse() {
                    self.close(id);
                }
                Ok(())
            }
            id if id.starts_with("activate-")
                || id.starts_with("tab-")
                || id.starts_with("title-") =>
            {
                if let Some(id) = id.split_once('-').and_then(|(_, id)| id.parse().ok()) {
                    self.activate(id);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    fn command(&mut self, command: Command) -> io::Result<()> {
        match command {
            Command::Navigate(url) => self.navigate(
                url_loader::URLRequest {
                    url,
                    ..Default::default()
                },
                true,
            ),
            Command::Reload => self.navigate(
                url_loader::URLRequest {
                    url: self.tabs.active.location.clone(),
                    ..Default::default()
                },
                false,
            ),
            Command::Back => self.history(-1),
            Command::Forward => self.history(1),
            Command::NewTab => self.new_tab(
                url_loader::URLRequest {
                    url: "about:home".into(),
                    ..Default::default()
                },
                true,
            ),
            Command::CloseActiveTab => {
                self.close(self.tabs.active.id);
                Ok(())
            }
            Command::CycleTab(delta) => {
                let tabs = self.tabs.ordered();
                let i = tabs
                    .iter()
                    .position(|t| t.id == self.tabs.active.id)
                    .unwrap();
                let id = tabs[(i as isize + delta).rem_euclid(tabs.len() as isize) as usize].id;
                self.activate(id);
                Ok(())
            }
            Command::FocusAddress => {
                self.focus_address();
                Ok(())
            }
            Command::SetActive(active) => {
                self.active = active;
                if !active {
                    self.popup = None;
                    self.scene();
                }
                self.active_pages();
                Ok(())
            }
            Command::Resize(viewport) => {
                if viewport.valid() {
                    self.viewport = viewport;
                    self.popup = None;
                    self.toolbar.resize(Viewport {
                        height: viewport.toolbar_pixels(),
                        ..viewport
                    });
                    if let Some(page) = &self.tabs.active.page {
                        page.resize(self.content_viewport());
                    }
                    self.tabs.active.viewport = Some(viewport);
                    self.scene();
                }
                Ok(())
            }
            Command::VSyncDisplayChanged(id) => {
                self.source.set_display(id);
                Ok(())
            }
            Command::Redraw => {
                self.scene();
                Ok(())
            }
            Command::Evaluate { source, reply } => {
                if let Some(page) = &self.tabs.active.page {
                    page.send(page::Command::Evaluate { source, reply });
                } else {
                    let _ = reply.send(Err("No active Page".into()));
                }
                Ok(())
            }
            Command::Input(input) => self.input(input, Instant::now(), 1),
            Command::NativeWheel(sample) => {
                let (delta_x, delta_y) = match &sample.delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        (f64::from(*x), f64::from(*y))
                    }
                    winit::event::MouseScrollDelta::PixelDelta(point) => (point.x, point.y),
                };
                browser_tracing::instant(
                    "input",
                    "NativeWheelDispatched",
                    &[
                        ("delta_x", delta_x),
                        ("delta_y", delta_y),
                        (
                            "native_phase",
                            sample
                                .native
                                .as_ref()
                                .map_or(0.0, |event| event.phase as f64),
                        ),
                        (
                            "momentum_phase",
                            sample
                                .native
                                .as_ref()
                                .map_or(0.0, |event| event.momentum_phase as f64),
                        ),
                    ],
                );
                let queued_at = sample.observed_at;
                for command in self.wheel_input.accept_native_wheel(sample, Instant::now()) {
                    if let Command::Input(input) = command {
                        self.input(input, queued_at, 1)?;
                    }
                }
                Ok(())
            }
            Command::QueuedInput {
                input,
                queued_at,
                samples,
            } => self.input(input, queued_at, samples),
            Command::BeginFrame(frame) => {
                let targets: Vec<_> = self
                    .routes
                    .lock()
                    .unwrap()
                    .values()
                    .filter(|route| route.visible())
                    .cloned()
                    .collect();
                for route in targets {
                    route.begin_frame(frame);
                }
                Ok(())
            }
            Command::Wake(_) | Command::Stop => Ok(()),
        }
    }
    fn input(
        &mut self,
        mut input: InputEvent,
        queued_at: Instant,
        samples: usize,
    ) -> io::Result<()> {
        if let Some(popup) = &mut self.popup {
            if let InputEvent::Key(e) = &input {
                if e.r#type == KeyEventType::kDown {
                    if e.key == "Escape" {
                        self.popup = None;
                        self.scene();
                        return Ok(());
                    }
                    if matches!(e.key.as_str(), "ArrowDown" | "ArrowUp" | "Tab")
                        && !popup.actions.is_empty()
                    {
                        let reverse = e.key == "ArrowUp" || (e.key == "Tab" && e.modifiers.shift);
                        let n = popup.actions.len();
                        let i = popup.selected.map_or(if reverse { n - 1 } else { 0 }, |i| {
                            if reverse {
                                (i + n - 1) % n
                            } else {
                                (i + 1) % n
                            }
                        });
                        popup.selected = Some(i);
                        popup.page.patch(Patch::Focus {
                            id: popup.actions[i].clone(),
                            select_all: false,
                        });
                        return Ok(());
                    }
                    if e.key == "Enter" {
                        if let Some(i) = popup.selected {
                            let action = popup.actions[i].clone();
                            self.popup = None;
                            self.scene();
                            return self.action(&action);
                        }
                    }
                }
            }
            if let InputEvent::Mouse(e) = &mut input {
                let inside = e.position.x >= popup.x
                    && e.position.x < popup.x + popup.width
                    && e.position.y >= TOOLBAR_HEIGHT
                    && e.position.y < TOOLBAR_HEIGHT + popup.height;
                if inside {
                    e.position.y -= TOOLBAR_HEIGHT;
                    popup.page.input(input, queued_at, samples);
                    return Ok(());
                }
                if e.r#type == MouseEventType::kDown {
                    self.popup = None;
                    self.scene();
                }
            }
        }
        let toolbar = match &mut input {
            InputEvent::Mouse(e) => {
                let toolbar = if e.r#type == MouseEventType::kLeave {
                    self.in_toolbar
                } else {
                    e.position.y < TOOLBAR_HEIGHT
                };
                if e.r#type == MouseEventType::kDown {
                    self.toolbar_focused = toolbar;
                    self.active_pages();
                }
                if self.in_toolbar != toolbar {
                    let leave = InputEvent::Mouse(MouseEvent {
                        r#type: MouseEventType::kLeave,
                        position: layoutng_assembly::internal::layout_input::Offset {
                            x: -1.0,
                            y: -1.0,
                        },
                        ..Default::default()
                    });
                    if toolbar {
                        if let Some(p) = &self.tabs.active.page {
                            p.input(leave, queued_at, 1);
                        }
                    } else {
                        self.toolbar.input(leave, queued_at, 1);
                    }
                }
                self.in_toolbar = toolbar;
                if !toolbar {
                    e.position.y -= TOOLBAR_HEIGHT;
                }
                toolbar
            }
            InputEvent::Wheel(e) => {
                let toolbar = e.position.y < TOOLBAR_HEIGHT;
                if !toolbar {
                    e.position.y -= TOOLBAR_HEIGHT;
                }
                toolbar
            }
            _ => self.toolbar_focused,
        };
        if toolbar {
            self.toolbar.input(input, queued_at, samples);
        } else if let Some(page) = &self.tabs.active.page {
            page.input(input, queued_at, samples);
        }
        Ok(())
    }
    fn event(&mut self, document: Document, event: Event) -> io::Result<()> {
        if let Event::Error(error) = event {
            // Deliver even a late error from a replaced/closed generation. It
            // remains useful to automation, but cannot mutate the new Page.
            (self.output.notify)(UserEvent::PageError(error.clone()));
            if error.terminal {
                if self
                    .popup
                    .as_ref()
                    .is_some_and(|p| p.page.document == document)
                {
                    self.popup = None;
                    self.scene();
                }
                let tab = if self
                    .tabs
                    .active
                    .page
                    .as_ref()
                    .is_some_and(|p| p.document == document)
                {
                    Some(&mut self.tabs.active)
                } else {
                    self.tabs
                        .background
                        .iter_mut()
                        .find(|t| t.page.as_ref().is_some_and(|p| p.document == document))
                };
                if let Some(tab) = tab {
                    if let Some(page) = &mut tab.page {
                        page.loading = false;
                    }
                    tab.title = "Page error".into();
                    self.refresh();
                }
            }
            return Ok(());
        }
        let toolbar = document == self.toolbar.document;
        let popup = self
            .popup
            .as_ref()
            .is_some_and(|p| p.page.document == document);
        let active = self
            .tabs
            .active
            .page
            .as_ref()
            .is_some_and(|p| p.document == document);
        if !toolbar
            && !popup
            && !active
            && !self
                .tabs
                .background
                .iter()
                .any(|t| t.page.as_ref().is_some_and(|p| p.document == document))
        {
            return Ok(());
        }
        let failed = if toolbar {
            self.toolbar.failed()
        } else if popup {
            self.popup.as_ref().is_some_and(|p| p.page.failed())
        } else if active {
            self.tabs.active.page.as_ref().is_some_and(Handle::failed)
        } else {
            self.tabs
                .background
                .iter()
                .filter_map(|t| t.page.as_ref())
                .find(|p| p.document == document)
                .is_some_and(Handle::failed)
        };
        if failed {
            return Ok(());
        }
        match event {
            Event::DragRegions(regions) if toolbar => {
                self.drag_regions = regions;
                self.scene();
            }
            Event::Status {
                url,
                title,
                loading,
            } if !toolbar && !popup => {
                let tab = if active {
                    &mut self.tabs.active
                } else {
                    self.tabs
                        .background
                        .iter_mut()
                        .find(|t| t.id == document.0)
                        .unwrap()
                };
                if let Some(page) = &mut tab.page {
                    page.loading = loading;
                }
                let changed = !tab.location.starts_with("about:")
                    && !url.starts_with("data:")
                    && !url.is_empty()
                    && tab.location != url;
                if changed {
                    tab.location = url;
                    if let Some(entry) = tab.history.get_mut(tab.history_index) {
                        *entry = tab.location.clone();
                    }
                }
                if !title.is_empty() && tab.location != "about:home" {
                    tab.title = title;
                }
                if active && changed {
                    self.address();
                    (self.output.notify)(UserEvent::Location(self.tabs.active.location.clone()));
                }
                self.refresh();
            }
            Event::Navigate(request) if !toolbar && !popup => {
                if request.target.eq_ignore_ascii_case("_blank") {
                    self.new_tab(request.request, false)?;
                } else if active {
                    self.navigate(request.request, !request.replace_history)?;
                } else {
                    // Navigation belongs to the originating tab even when a
                    // newer host input has already selected another one.
                    let location = crate::options::normalize_address(&request.request.url)?;
                    let generation = document.1 + 1;
                    let page = self.make_page(
                        (document.0, generation),
                        request.request,
                        location.clone(),
                        false,
                        false,
                    )?;
                    page.visible(false);
                    page.send(page::Command::Active(false));
                    if let Some(tab) = self.tabs.background.iter_mut().find(|t| t.id == document.0)
                    {
                        tab.page = Some(page);
                        tab.document_generation = generation;
                        tab.location = location.clone();
                        tab.title = location_title(&location);
                        if !request.replace_history {
                            tab.history.truncate(tab.history_index + 1);
                            tab.history.push(location);
                            tab.history_index = tab.history.len() - 1;
                        }
                    }
                    self.refresh();
                    self.scene();
                }
            }
            Event::Input {
                input,
                target,
                focused,
                value,
                prevented,
            } if (toolbar || popup) && !prevented => {
                let submit_address = matches!(
                    &input,
                    InputEvent::Key(e)
                        if e.r#type == KeyEventType::kDown
                            && e.key == "Enter"
                            && focused.as_deref() == Some("address")
                );
                let action = match input {
                    InputEvent::Mouse(e) if e.r#type == MouseEventType::kClick => {
                        target.filter(|target| target != "address")
                    }
                    InputEvent::Key(e)
                        if (e.r#type == KeyEventType::kDown && e.key == "Enter")
                            || (e.r#type == KeyEventType::kUp
                                && e.key == " "
                                && focused.as_deref() != Some("address")) =>
                    {
                        focused
                    }
                    _ => None,
                };
                if toolbar && submit_address {
                    self.navigate(
                        url_loader::URLRequest {
                            url: value,
                            ..Default::default()
                        },
                        true,
                    )?;
                } else if let Some(action) = action {
                    if popup {
                        self.popup = None;
                        self.scene();
                    }
                    self.action(&action)?;
                }
            }
            Event::Feedback { cursor, mut caret } => {
                if (toolbar && self.in_toolbar) || (!toolbar && active && !self.in_toolbar) || popup
                {
                    (self.output.notify)(UserEvent::CursorChanged(cursor));
                }
                if (toolbar && self.toolbar_focused) || (active && !self.toolbar_focused) {
                    if !toolbar {
                        if let Some(rect) = &mut caret {
                            rect.y += TOOLBAR_HEIGHT;
                        }
                    }
                    (self.output.notify)(UserEvent::CaretChanged(caret));
                }
            }
            Event::Error(_) => unreachable!("handled before lifetime filtering"),
            _ => {}
        }
        Ok(())
    }
}
fn location_title(location: &str) -> String {
    match location {
        "about:home" => "New tab".into(),
        "about:blank" => "Blank page".into(),
        _ => url::Url::parse(location)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned))
            .unwrap_or_else(|| "Document".into()),
    }
}
