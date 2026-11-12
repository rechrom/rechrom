//! Browser policy and wiring. This host owns handles, tabs/history, focus and
//! Surface placement; it never calls DOM, JS, lifecycle, paint or raster.
use crate::{
    begin_frame_source,
    engine::{Command, Output, UserEvent, Viewport, TOOLBAR_HEIGHT},
    input_surface_routing::{InputSurfaceRouting, SurfaceRoute},
    page_presentation::{Document, Placement, Scene},
    page_runtime::{self as page, Event, Handle, Patch},
    window_runtime,
};
use foundation::begin_frame::BeginFrameSource;
use interaction::input_event::*;
use navigation_history::FrameLoadType;
use std::{
    collections::BTreeMap,
    io,
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};
enum Message {
    Host(Command),
    Page(Document, Event),
}

/// Collapse only the resize samples that are already adjacent in the Browser
/// Host inbox.  Chromium treats a stream of visual-properties updates as a
/// latest-value transaction, but never moves one across input or renderer
/// messages.  Keeping the first non-resize message aside preserves that same
/// ordering boundary while avoiding toolbar/page/Scene work for intermediate
/// native sizes that can no longer be presented.
fn coalesce_pending_resizes(
    first: Message,
    inbox: &mpsc::Receiver<Message>,
    pending: &mut Option<Message>,
) -> Message {
    let Message::Host(Command::Resize(mut resize)) = first else {
        return first;
    };
    loop {
        match inbox.try_recv() {
            Ok(Message::Host(Command::Resize(next)))
                if !resize.viewport.valid() || next.viewport.valid() =>
            {
                resize.merge(next)
            }
            Ok(message) => {
                *pending = Some(message);
                break;
            }
            Err(mpsc::TryRecvError::Empty | mpsc::TryRecvError::Disconnected) => break,
        }
    }
    Message::Host(Command::Resize(resize))
}

#[inline]
fn root_resize_changes_visual_properties(current: Viewport, requested: Viewport) -> bool {
    requested.valid() && requested != current
}
struct Popup {
    page: Handle,
    x: f64,
    width: f64,
    height: f64,
    actions: Vec<String>,
    selected: Option<usize>,
}
/// Non-interactive browser UI. Its independent Page never receives author
/// input or shares DOM state with the anchored document.
struct ValidationOverlay {
    owner: Document,
    control: u64,
    anchor: browser::page::ValidationMessageRect,
    size: Option<(f64, f64)>,
    hiding_deadline: Option<Instant>,
    message: Arc<[u16]>,
    page: Handle,
}
struct NativeDropRoute {
    data: Arc<interaction::dispatch::core::clipboard::data_transfer::WebDragData>,
    route: Option<SurfaceRoute>,
    last: crate::native_drag::Target,
}
struct Host {
    target_drag: Option<NativeDropRoute>,
    widget_screen_origin: Option<(i32, i32, f64)>,
    viewport: Viewport,
    output: Output,
    toolbar: Handle,
    tabs: crate::tabs::Tabs<Handle>,
    validation_overlay: Option<ValidationOverlay>,
    validation_generation: u64,
    popup: Option<Popup>,
    popup_generation: u64,
    toolbar_focused: bool,
    in_toolbar: bool,
    input_surfaces: InputSurfaceRouting,
    active: bool,
    routes: page::FrameRoutes,
    source: Arc<dyn BeginFrameSource>,
    frames: mpsc::Sender<window_runtime::Command>,
    events: Arc<dyn Fn(Document, Event) + Send + Sync>,
    threads: browser::page::PageThreads,
    prefers_reduced_motion: bool,
    text_settings: crate::interaction_text_settings::InteractionTextSettings,
    font_backend_factory: browser::BrowserFontBackendFactory,
    primary_clipboard: Arc<crate::primary_clipboard::PrimaryClipboard>,
    native_drag_window: bool,
    tabstrip: String,
    tabstrip_ids: Vec<u64>,
    reload_loading: Option<bool>,
    drag_regions: Vec<crate::chrome::DragRegion>,
    wheel_input: crate::input::InputState,
    toolbar_revision: u64,
    next_navigation_preparation: u64,
    navigation_preparations: BTreeMap<u64, NavigationPreparation>,
    auxiliary_navigation_targets: BTreeMap<u64, AuxiliaryNavigationTarget>,
}

/// The host reserves an actual browsing context before the source anchor
/// continuation resumes. A later navigation consumes this exact target;
/// neither phase reconstructs the destination from a URL or a tab selection.
struct AuxiliaryNavigationTarget {
    source: Document,
    tab_id: u64,
    host: browser::page::NavigationHostMetadata,
}

/// NavigationController retains the continuation while the outgoing document
/// publishes UpdateState. Only an acknowledgement token crosses to that Page.
struct NavigationPreparation {
    document: Document,
    token: u64,
    request: url_loader::URLRequest,
    record: bool,
    focus_address: bool,
}

/// Product composition supplies process-wide execution services. Page state
/// remains independent; only finite background work runs on these shared
/// workers, as it does on Chromium's browser/renderer worker pools.
fn page_threads() -> io::Result<browser::page::PageThreads> {
    let inline = std::env::var("RECHROM_EXECUTOR").as_deref() == Ok("inline");
    let (executor, script_compile_executor, compositor_executor, raster_executor): (
        Arc<dyn executor::Executor>,
        Arc<dyn executor::Executor>,
        Arc<dyn executor::Executor>,
        Arc<dyn executor::Executor>,
    ) = if inline {
        let executor = executor::inline();
        (
            executor.clone(),
            executor.clone(),
            executor.clone(),
            executor,
        )
    } else {
        // Match Chromium's scheduling boundary: impl input and frame
        // scheduling have a dedicated sequence, while raster playback uses a
        // worker pool. ScriptStreamer posts compile work to worker_pool rather
        // than the image-decode sequence (script_streamer.cc:1104-1109), so a
        // document with several large bundles cannot make decode or compile
        // wait behind the other class of work.
        let compile_lanes = script_compile_lanes(
            std::thread::available_parallelism()
                .map(std::num::NonZeroUsize::get)
                .unwrap_or(2),
        );
        (
            Arc::new(executor::PoolExecutor::new(
                "page-work",
                16 * 1024 * 1024,
                2,
            )?),
            Arc::new(executor::PoolExecutor::new(
                "page-script-compile",
                16 * 1024 * 1024,
                compile_lanes,
            )?),
            Arc::new(executor::PoolExecutor::new_with_thread_initializer(
                "page-compositor",
                16 * 1024 * 1024,
                1,
                Arc::new(crate::engine::set_presentation_thread_priority),
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
        script_compile_executor,
        compositor_executor,
        raster_executor,
    })
}

// Chromium's worker pool scales with the machine, while the renderer main and
// compositor sequences remain independently runnable. Bound the embedder pool
// so background compilation gains parallel progress without creating one
// large-stack thread per logical core on very large hosts.
fn script_compile_lanes(parallelism: usize) -> usize {
    parallelism.saturating_sub(2).clamp(2, 8)
}

#[cfg(test)]
mod tests {
    use super::{
        coalesce_pending_resizes, root_resize_changes_visual_properties, script_compile_lanes,
        Message,
    };
    use crate::engine::{Command, ResizeRequest, Viewport};
    use std::{
        sync::mpsc,
        time::{Duration, Instant},
    };

    #[test]
    fn script_compile_pool_leaves_main_and_compositor_capacity() {
        assert_eq!(script_compile_lanes(1), 2);
        assert_eq!(script_compile_lanes(4), 2);
        assert_eq!(script_compile_lanes(10), 8);
        assert_eq!(script_compile_lanes(64), 8);
    }

    #[test]
    fn host_coalesces_only_adjacent_resize_samples() {
        let (sender, inbox) = mpsc::channel();
        let queued_at = Instant::now();
        let request = |width, offset| {
            ResizeRequest::new(
                Viewport {
                    width,
                    height: 700,
                    scale: 1.0,
                },
                queued_at + Duration::from_millis(offset),
            )
        };
        sender
            .send(Message::Host(Command::Resize(request(901, 1))))
            .unwrap();
        sender
            .send(Message::Host(Command::Resize(request(902, 2))))
            .unwrap();
        sender.send(Message::Host(Command::Redraw)).unwrap();
        sender
            .send(Message::Host(Command::Resize(request(903, 3))))
            .unwrap();

        let mut pending = None;
        let merged = coalesce_pending_resizes(
            Message::Host(Command::Resize(request(900, 0))),
            &inbox,
            &mut pending,
        );
        let Message::Host(Command::Resize(merged)) = merged else {
            panic!("resize must remain a resize");
        };
        assert_eq!(merged.viewport.width, 902);
        assert_eq!(merged.root_viewport.width, 902);
        assert_eq!(merged.queued_at, queued_at);
        assert_eq!(merged.samples, 3);
        assert!(matches!(
            pending.take(),
            Some(Message::Host(Command::Redraw))
        ));

        // The resize after Redraw remains queued.  Coalescing cannot move a
        // visual-properties update across an observable host message.
        let Message::Host(Command::Resize(next)) = inbox.recv().unwrap() else {
            panic!("resize after ordering boundary must remain queued");
        };
        assert_eq!(next.viewport.width, 903);
        assert_eq!(next.samples, 1);
    }

    #[test]
    fn identical_root_visual_properties_are_a_no_op() {
        let current = Viewport {
            width: 900,
            height: 700,
            scale: 2.0,
        };
        assert!(!root_resize_changes_visual_properties(current, current));
        assert!(!root_resize_changes_visual_properties(
            current,
            Viewport {
                width: 0,
                ..current
            }
        ));
        assert!(root_resize_changes_visual_properties(
            current,
            Viewport {
                width: 901,
                ..current
            }
        ));
        assert!(root_resize_changes_visual_properties(
            current,
            Viewport {
                scale: 1.0,
                ..current
            }
        ));
    }

    #[test]
    fn display_binding_remains_an_ordering_boundary_for_duplicate_resize() {
        let (sender, inbox) = mpsc::channel();
        let viewport = Viewport {
            width: 900,
            height: 700,
            scale: 2.0,
        };
        let request = ResizeRequest::new(viewport, Instant::now());
        sender
            .send(Message::Host(Command::VSyncDisplayChanged(42)))
            .unwrap();
        sender
            .send(Message::Host(Command::Resize(request)))
            .unwrap();

        let mut pending = None;
        let resize = coalesce_pending_resizes(
            Message::Host(Command::Resize(request)),
            &inbox,
            &mut pending,
        );
        assert!(matches!(resize, Message::Host(Command::Resize(_))));
        assert!(matches!(
            pending,
            Some(Message::Host(Command::VSyncDisplayChanged(42)))
        ));
        assert!(matches!(
            inbox.recv().unwrap(),
            Message::Host(Command::Resize(_))
        ));
    }
}

pub(crate) fn spawn(
    viewport: Viewport,
    address: String,
    output: Output,
    target: crate::window_surface::DisplayTarget,
    interval: Duration,
    display_id: Option<u32>,
    config: crate::presentation_runtime::Config,
    prefers_reduced_motion: bool,
) -> io::Result<mpsc::Sender<Command>> {
    if !viewport.valid() {
        return Err(io::Error::other("invalid window viewport"));
    }
    let frame_source = if target.is_window() {
        begin_frame_source::create(interval, display_id)?
    } else {
        begin_frame_source::create_offscreen(interval)?
    };
    let routes: page::FrameRoutes = Arc::new(Mutex::new(Default::default()));
    let ticks = routes.clone();
    let compositor_source = frame_source.GetFrameSource(1)?;
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
    frame_source
        .GetFrameSource(0)?
        .SetClient(Arc::new(|_| {}))?;
    let primary_clipboard = target.primary_clipboard();
    let native_drag_window = target.is_window();
    let frames = window_runtime::spawn(
        config,
        target,
        output.clone(),
        frame_source.GetFrameSource(2)?,
    )?;
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
    // Creating these pools starts eleven OS threads. Do it once during browser
    // startup, not once per navigation or popup.
    let threads = page_threads()?;
    // BrowserView constructs the initial TabView before its first layout. Keep
    // the same ownership boundary here: the toolbar parser receives the first
    // tab in its initial document, so startup metadata changes update existing
    // nodes instead of replacing an empty connected subtree asynchronously.
    let tabs = crate::tabs::Tabs::new();
    let tabstrip = crate::chrome::tab_strip(&tabs);
    let tabstrip_ids = tabs.ordered().iter().map(|tab| tab.id).collect();
    let toolbar_document = crate::chrome::document(&tabs);
    let text_settings = crate::interaction_text_settings::current()?;
    let font_backend_factory = browser::CreateBrowserFontBackendFactory();
    let toolbar = page::spawn(page::Params {
        document: (0, 0),
        url: "browser://toolbar".into(),
        viewport: Viewport {
            height: viewport.toolbar_pixels(),
            ..viewport
        },
        request: url_loader::URLRequest {
            url: crate::engine::data_url(&toolbar_document),
            ..Default::default()
        },
        scripting: false,
        prefers_reduced_motion,
        system_cursor_size: crate::cursor_screen_info::current(),
        text_settings: text_settings.clone(),
        font_backend_factory: font_backend_factory.clone(),
        primary_clipboard: primary_clipboard.clone(),
        native_drag_window,
        threads: threads.clone(),
        // BrowserView owns the tab-strip/navigation backdrops in the root Viz
        // frame. ToolbarView contributes only controls above those quads, so
        // its delegated Surface must preserve alpha instead of repainting the
        // same browser-owned pixels through the Page lifecycle.
        transparent: true,
        controls: true,
        source: source.clone(),
        routes: routes.clone(),
        frames: frames.clone(),
        events: events.clone(),
    })?;
    std::thread::Builder::new()
        .name("browser-host".into())
        .spawn(move || {
            let _trace_scope = browser_tracing::scope(browser_tracing::Context {
                target_id: 1,
                ..Default::default()
            });
            let mut host = Host {
                target_drag: None,
                widget_screen_origin: None,
                viewport,
                output,
                toolbar,
                tabs,
                validation_overlay: None,
                validation_generation: 0,
                popup: None,
                popup_generation: 0,
                toolbar_focused: false,
                in_toolbar: false,
                input_surfaces: InputSurfaceRouting::default(),
                active: true,
                routes,
                source,
                frames,
                events,
                threads,
                prefers_reduced_motion,
                text_settings,
                font_backend_factory,
                primary_clipboard,
                native_drag_window,
                tabstrip,
                tabstrip_ids,
                reload_loading: None,
                drag_regions: vec![],
                wheel_input: crate::input::InputState::default(),
                toolbar_revision: 0,
                next_navigation_preparation: 0,
                navigation_preparations: BTreeMap::new(),
                auxiliary_navigation_targets: BTreeMap::new(),
            };
            if let Err(error) = host.navigate(
                url_loader::URLRequest {
                    url: address,
                    ..Default::default()
                },
                true,
                false,
            ) {
                (host.output.notify)(UserEvent::Message(error.to_string()));
            }
            let mut pending_message = None;
            loop {
                host.expire_validation_message(Instant::now());
                let message = if let Some(message) = pending_message.take() {
                    message
                } else if let Some(deadline) = host
                    .wheel_input
                    .pending_wheel_end_deadline()
                    .into_iter()
                    .chain(
                        host.validation_overlay
                            .as_ref()
                            .and_then(|overlay| overlay.hiding_deadline),
                    )
                    .min()
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
                let message = coalesce_pending_resizes(message, &inbox, &mut pending_message);
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
    fn publish_widget_screen_geometry(&self) {
        let Some((x, y, scale)) = self.widget_screen_origin else {
            return;
        };
        let geometry = |content: bool| {
            interaction::adapter::page_nonlocated_context_menu_coordinates::NativeContextMenuWidgetGeometry {
            screen_origin_dip: foundation::gfx::Point::new(x, y.saturating_add(if content { TOOLBAR_HEIGHT.round() as i32 } else { 0 })),
            blink_to_dip_scale: (1.0 / scale) as f32,
        }
        };
        self.toolbar
            .send(page::Command::ContextMenuWidgetGeometry(geometry(false)));
        for tab in self.tabs.ordered() {
            if let Some(page) = &tab.page {
                page.send(page::Command::ContextMenuWidgetGeometry(geometry(true)));
            }
        }
        if let Some(popup) = &self.popup {
            // Popup Page is embedded across the content viewport; its x
            // placement is authored inside that Page, not a native view offset.
            popup
                .page
                .send(page::Command::ContextMenuWidgetGeometry(geometry(true)));
        }
    }
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
        let _trace = browser_tracing::span("browser", "Host.CreatePage");
        let page = page::spawn(page::Params {
            document,
            url,
            viewport: self.content_viewport(),
            request,
            scripting: !controls,
            prefers_reduced_motion: self.prefers_reduced_motion,
            system_cursor_size: crate::cursor_screen_info::current(),
            text_settings: self.text_settings.clone(),
            font_backend_factory: self.font_backend_factory.clone(),
            primary_clipboard: self.primary_clipboard.clone(),
            native_drag_window: self.native_drag_window,
            threads: self.threads.clone(),
            controls,
            transparent,
            source: self.source.clone(),
            routes: self.routes.clone(),
            frames: self.frames.clone(),
            events: self.events.clone(),
        })?;
        if let Some((x, y, scale)) = self.widget_screen_origin {
            page.send(page::Command::ContextMenuWidgetGeometry(
                interaction::adapter::page_nonlocated_context_menu_coordinates::NativeContextMenuWidgetGeometry {
                    screen_origin_dip: foundation::gfx::Point::new(x, y.saturating_add(TOOLBAR_HEIGHT.round() as i32)),
                    blink_to_dip_scale: (1.0 / scale) as f32,
                },
            ));
        }
        Ok(page)
    }
    fn scene(&self) {
        self.scene_at(self.viewport);
    }
    fn scene_at(&self, viewport: Viewport) {
        let toolbar_height = viewport.toolbar_pixels();
        let mut browser_root = crate::chrome::browser_root_frame(viewport);
        if std::env::var("RECHROM_TRANSPARENT_BACKGROUND").as_deref() == Ok("1") {
            // Preserve the toolbar's explicit opaque quads while leaving the
            // content area behind a transparent Page unfilled. This mirrors
            // DevTools' transparent default-background override for raster
            // parity diagnostics.
            browser_root.background_color = [0; 4];
        }
        let content = crate::page_presentation::DeviceRect::new(
            0,
            toolbar_height as i32,
            viewport.width,
            viewport.height.saturating_sub(toolbar_height).max(1),
        );
        let mut placements = vec![Placement {
            document: self.toolbar.document,
            destination: crate::page_presentation::DeviceRect::new(
                0,
                0,
                viewport.width,
                toolbar_height,
            ),
        }];
        let mut live_documents = vec![self.toolbar.document];
        for tab in self.tabs.ordered() {
            if let Some(page) = &tab.page {
                live_documents.push(page.document);
            }
            if let Some(page) = tab
                .pending_navigation
                .as_ref()
                .and_then(|pending| pending.page.as_ref())
            {
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
        if let Some(overlay) = &self.validation_overlay {
            if self
                .tabs
                .active
                .page
                .as_ref()
                .is_some_and(|page| page.document == overlay.owner)
            {
                placements.push(Placement {
                    document: overlay.page.document,
                    destination: content,
                });
                live_documents.push(overlay.page.document);
            }
        }
        let _ = self.frames.send(window_runtime::Command::Display(
            crate::display::Message::Scene(Scene {
                viewport,
                browser_root,
                placements,
                live_documents,
                metadata: self.drag_regions.clone(),
                toolbar_revision: self.toolbar_revision,
            }),
        ));
        // Cross-thread frame delivery starts only after embedding metadata is
        // enqueued, so a fast new producer cannot be pruned as an orphan.
        self.toolbar.visible(true);
        if let Some(page) = &self.tabs.active.page {
            page.visible(true);
        }
        // A provisional main frame is not embedded until HistoryCommitted,
        // but Chromium still treats its RenderWidget as shown so it can run
        // the BeginMainFrame that produces that commit.  Leaving it hidden
        // consumes the initial physical pulse while cc is waiting on main;
        // the committed replacement can then never publish its first frame.
        if let Some(page) = self
            .tabs
            .active
            .pending_navigation
            .as_ref()
            .and_then(|pending| pending.page.as_ref())
        {
            page.visible(true);
        }
        if let Some(popup) = &self.popup {
            popup.page.visible(true);
        }
        if let Some(overlay) = &self.validation_overlay {
            overlay.page.visible(true);
        }
    }
    fn refresh_interaction_text_settings(&mut self) -> io::Result<()> {
        // Platform settings may have changed while the application was in the
        // background. Query on activation, outside the pointer/input hot path.
        let settings = crate::interaction_text_settings::current()?;
        if settings == self.text_settings {
            return Ok(());
        }
        self.text_settings = settings.clone();
        self.toolbar
            .send(page::Command::InteractionTextSettings(settings.clone()));
        for tab in self.tabs.ordered() {
            if let Some(page) = &tab.page {
                page.send(page::Command::InteractionTextSettings(settings.clone()));
            }
        }
        if let Some(popup) = &self.popup {
            popup
                .page
                .send(page::Command::InteractionTextSettings(settings));
        }
        Ok(())
    }
    fn active_pages(&mut self) {
        self.set_active_pages();
        self.toolbar_revision = self
            .toolbar_revision
            .max(self.toolbar.patch(self.active_toolbar_patch()));
    }
    fn set_active_pages(&self) {
        self.toolbar
            .send(page::Command::Active(self.active && self.toolbar_focused));
        if let Some(page) = &self.tabs.active.page {
            page.send(page::Command::Active(self.active && !self.toolbar_focused));
        }
    }
    fn active_toolbar_patch(&self) -> Patch {
        Patch::Attribute {
            id: "omnibox".into(),
            name: "class".into(),
            value: Some(if self.toolbar_focused { "focused" } else { "" }.into()),
        }
    }
    fn address(&mut self) {
        self.toolbar_revision = self.toolbar_revision.max(self.toolbar.patch(Patch::Value {
            id: "address".into(),
            value: self.tabs.active.location.clone(),
        }));
    }
    fn refresh(&mut self) {
        let patches = self.refresh_patches();
        self.toolbar_revision = self.toolbar_revision.max(self.toolbar.patches(patches));
    }
    fn refresh_patches(&mut self) -> Vec<Patch> {
        let mut patches = Vec::new();
        let ordered = self.tabs.ordered();
        let ids = ordered.iter().map(|tab| tab.id).collect::<Vec<_>>();
        let html = crate::chrome::tab_strip(&self.tabs);
        if html != self.tabstrip {
            if ids != self.tabstrip_ids {
                // TabStripModelObserver creates/removes TabViews only when the
                // model structure changes. Metadata changes update the
                // existing view below; replacing the whole connected subtree
                // on every title/loading update is neither Chromium's model
                // nor safe for frame-scoped browser UI state.
                patches.push(Patch::Attribute {
                    id: "tabs".into(),
                    name: "style".into(),
                    value: Some(format!("flex-basis:{}px", ordered.len() * 238 - 6)),
                });
                patches.push(Patch::Html {
                    id: "tabs".into(),
                    value: html.clone(),
                });
                self.tabstrip_ids = ids;
            } else {
                for (index, tab) in ordered.iter().enumerate() {
                    let active = tab.id == self.tabs.active.id;
                    let before_active = ordered
                        .get(index + 1)
                        .is_some_and(|next| next.id == self.tabs.active.id);
                    let class = format!(
                        "tab{}{}",
                        if active { " active" } else { "" },
                        if before_active { " beforeactive" } else { "" }
                    );
                    patches.extend([
                        Patch::Attribute {
                            id: format!("tab-{}", tab.id),
                            name: "class".into(),
                            value: Some(class),
                        },
                        Patch::Attribute {
                            id: format!("tab-{}", tab.id),
                            name: "title".into(),
                            value: Some(tab.location.clone()),
                        },
                        Patch::Html {
                            id: format!("favicon-{}", tab.id),
                            value: crate::chrome::tab_icon(tab.loading).into(),
                        },
                        Patch::Text {
                            id: format!("title-{}", tab.id),
                            value: tab.title.clone(),
                        },
                    ]);
                }
            }
            self.tabstrip = html;
        }
        for (id, enabled) in [
            (
                "back",
                self.tabs.active.history_index_for_offset(-1).is_some(),
            ),
            (
                "forward",
                self.tabs.active.history_index_for_offset(1).is_some(),
            ),
        ] {
            patches.push(Patch::Attribute {
                id: id.into(),
                name: "disabled".into(),
                value: (!enabled).then(String::new),
            });
        }
        let loading = self.tabs.active.loading;
        if self.reload_loading != Some(loading) {
            self.reload_loading = Some(loading);
            patches.push(Patch::Html {
                id: "reload".into(),
                value: crate::chrome::reload_icon(loading).into(),
            });
        }
        patches
    }
    /// Commit the browser-owned state for one user action as one toolbar
    /// mutation batch. The toolbar Page still runs its normal lifecycle, but
    /// the root scene no longer waits through several intermediate toolbar
    /// frames before it can expose an already-ready tab Surface.
    fn sync_toolbar(&mut self, include_address: bool, focus_address: bool) {
        self.set_active_pages();
        let mut patches = vec![self.active_toolbar_patch()];
        if include_address {
            patches.push(Patch::Value {
                id: "address".into(),
                value: self.tabs.active.location.clone(),
            });
        }
        patches.extend(self.refresh_patches());
        if focus_address {
            patches.push(Patch::Focus {
                id: "address".into(),
                select_all: true,
            });
        }
        self.toolbar_revision = self.toolbar_revision.max(self.toolbar.patches(patches));
    }
    fn navigate(
        &mut self,
        request: url_loader::URLRequest,
        record: bool,
        focus_address: bool,
    ) -> io::Result<()> {
        crate::options::normalize_address(&request.url)?;
        if self.prepare_navigation_state(self.tabs.active.id, &request, record, focus_address) {
            return Ok(());
        }
        self.navigate_prepared(request, record, focus_address)
    }
    fn prepare_navigation_state(
        &mut self,
        tab_id: u64,
        request: &url_loader::URLRequest,
        record: bool,
        focus_address: bool,
    ) -> bool {
        let Some(page) = self
            .tabs
            .ordered()
            .into_iter()
            .find(|tab| tab.id == tab_id)
            .and_then(|tab| tab.page.as_ref())
            .filter(|page| !page.failed())
        else {
            return false;
        };
        self.next_navigation_preparation += 1;
        let token = self.next_navigation_preparation;
        self.navigation_preparations.insert(
            tab_id,
            NavigationPreparation {
                document: page.document,
                token,
                request: request.clone(),
                record,
                focus_address,
            },
        );
        page.send(page::Command::PrepareNavigation { token });
        true
    }
    fn continue_prepared_navigation(
        &mut self,
        document: Document,
        token: Option<u64>,
    ) -> io::Result<()> {
        if !self
            .navigation_preparations
            .get(&document.0)
            .is_some_and(|preparation| {
                preparation.document == document
                    && token.is_none_or(|token| token == preparation.token)
            })
        {
            return Ok(());
        }
        let preparation = self.navigation_preparations.remove(&document.0).unwrap();
        if self.tabs.active.id == document.0 {
            self.navigate_prepared(
                preparation.request,
                preparation.record,
                preparation.focus_address,
            )
        } else {
            self.navigate_background_prepared(document.0, preparation.request, preparation.record)
        }
    }
    fn navigate_prepared(
        &mut self,
        mut request: url_loader::URLRequest,
        record: bool,
        focus_address: bool,
    ) -> io::Result<()> {
        let _trace = browser_tracing::span("browser", "Host.Navigate");
        let location = crate::options::normalize_address(&request.url)?;
        self.tabs.active.prepare_navigation(
            &mut request,
            if record {
                FrameLoadType::Standard
            } else {
                FrameLoadType::ReplaceCurrentItem
            },
        );
        request.url = match location.as_str() {
            "about:home" => crate::engine::data_url(crate::engine::HOME),
            "about:blank" => crate::engine::data_url("<!doctype html><html><body></body></html>"),
            _ => location.clone(),
        };
        let generation = self.tabs.active.document_generation + 1;
        self.tabs.active.begin_navigation(generation, &request);

        // Browser chrome publishes the pending navigation before Page
        // creation. This keeps the omnibox and tab strip responsive even when
        // the new document needs threads, a runtime, or network setup.
        self.tabs.active.location = location.clone();
        self.tabs.active.title = location_title(&location);
        self.tabs.active.loading = true;
        self.dismiss_popup();
        self.dismiss_validation_message();
        self.toolbar_focused = focus_address;
        self.sync_toolbar(true, focus_address);
        self.scene();
        (self.output.notify)(UserEvent::Location(location.clone()));

        let page = match self.make_page(
            (self.tabs.active.id, generation),
            request,
            location.clone(),
            false,
            std::env::var("RECHROM_TRANSPARENT_BACKGROUND").as_deref() == Ok("1"),
        ) {
            Ok(page) => page,
            Err(error) => {
                self.cancel_pending_navigation(self.tabs.active.id);
                return Err(error);
            }
        };
        self.tabs.active.pending_navigation.as_mut().unwrap().page = Some(page);
        self.set_active_pages();
        self.scene();
        Ok(())
    }
    fn show(&mut self) {
        self.dismiss_popup();
        self.toolbar_focused = false;
        if let Some(page) = &self.tabs.active.page {
            // Source: RenderWidgetHostImpl::WasShown() calls
            // SynchronizeVisualProperties() unconditionally. The renderer
            // throttle owns value de-duplication. In particular, WasHidden()
            // may have cleared an in-flight latest value after BrowserView
            // already stored the new root viewport on the tab.
            page.resize(
                crate::engine::ResizeRequest::new(self.viewport, Instant::now())
                    .with_viewport(self.content_viewport()),
            );
        }
        if let Some(page) = self
            .tabs
            .active
            .pending_navigation
            .as_ref()
            .and_then(|pending| pending.page.as_ref())
        {
            page.resize(
                crate::engine::ResizeRequest::new(self.viewport, Instant::now())
                    .with_viewport(self.content_viewport()),
            );
        }
        self.tabs.active.viewport = Some(self.viewport);
        self.sync_toolbar(true, false);
        self.scene();
        (self.output.notify)(UserEvent::Location(self.tabs.active.location.clone()));
    }
    fn hide(&mut self) {
        self.dismiss_validation_message();
        if let Some(page) = &self.tabs.active.page {
            self.input_surfaces.SurfaceRemoved(page.document);
            page.visible(false);
            page.send(page::Command::Active(false));
        }
        if let Some(page) = self
            .tabs
            .active
            .pending_navigation
            .as_ref()
            .and_then(|pending| pending.page.as_ref())
        {
            page.visible(false);
            page.send(page::Command::Active(false));
        }
    }
    fn activate(&mut self, id: u64) {
        let _trace = browser_tracing::span("browser", "Host.ActivateTab");
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
        let _trace = browser_tracing::span("browser", "Host.NewTab");
        crate::options::normalize_address(&request.url)?;
        self.hide();
        self.tabs.add();
        self.navigate(request, true, focus)
    }
    fn new_background_tab(&mut self, request: url_loader::URLRequest) -> io::Result<()> {
        crate::options::normalize_address(&request.url)?;
        let id = self.tabs.add_background();
        self.navigate_background(id, request, true)
    }

    fn reserve_auxiliary_navigation_target(
        &mut self,
        source: Document,
        lookup: browser::page::AuxiliaryNavigationTargetRequest,
    ) -> io::Result<bool> {
        use browser::page::NavigationTarget;
        use interaction::default::core::loader::navigation_policy::NavigationPolicy::*;
        let Some(mut metadata) = lookup.request.host else {
            return Ok(false);
        };
        let background = match metadata.policy {
            kNavigationPolicyCurrentTab | kNavigationPolicyNewForegroundTab => false,
            kNavigationPolicyNewBackgroundTab => true,
            // This embedding has no corresponding native target service.
            // Return rejection to the source continuation instead of silently
            // changing a window, download or PiP request into a tab.
            _ => return Ok(false),
        };
        let existing = if metadata.policy == kNavigationPolicyCurrentTab && !metadata.noopener {
            match &lookup.request.target {
                NavigationTarget::Named(name) => self
                    .tabs
                    .ordered()
                    .into_iter()
                    .find(|tab| tab.name == *name)
                    .map(|tab| tab.id),
                NavigationTarget::SelfFrame | NavigationTarget::Parent | NavigationTarget::Top => {
                    Some(source.0)
                }
                NavigationTarget::NewContext => None,
            }
        } else {
            None
        };
        let tab_id = if let Some(id) = existing {
            id
        } else {
            // An auxiliary context has an actual initial empty document.
            // Its eventual destination is loaded only after the originating
            // Page resumes its source navigation continuation.
            let blank = url_loader::URLRequest {
                url: "about:blank".into(),
                ..Default::default()
            };
            let id = if background {
                self.new_background_tab(blank)?;
                self.tabs.background.last().expect("new background tab").id
            } else {
                self.new_tab(blank, false)?;
                self.tabs.active.id
            };
            if !metadata.noopener {
                if let NavigationTarget::Named(name) = lookup.request.target {
                    let tab = if self.tabs.active.id == id {
                        &mut self.tabs.active
                    } else {
                        self.tabs
                            .background
                            .iter_mut()
                            .find(|tab| tab.id == id)
                            .unwrap()
                    };
                    tab.name = name;
                }
            }
            id
        };
        metadata.auxiliary_target = Some(lookup.request_id);
        let tab = if self.tabs.active.id == tab_id {
            &mut self.tabs.active
        } else {
            self.tabs
                .background
                .iter_mut()
                .find(|tab| tab.id == tab_id)
                .expect("reserved browsing context exists")
        };
        tab.navigation_host = Some(metadata.clone());
        self.auxiliary_navigation_targets.insert(
            lookup.request_id,
            AuxiliaryNavigationTarget {
                source,
                tab_id,
                host: metadata,
            },
        );
        Ok(true)
    }
    fn close(&mut self, id: u64) {
        self.navigation_preparations.remove(&id);
        self.auxiliary_navigation_targets
            .retain(|_, target| target.source.0 != id && target.tab_id != id);
        if let Some(document) = self
            .tabs
            .ordered()
            .into_iter()
            .find(|tab| tab.id == id)
            .and_then(|tab| tab.page.as_ref())
            .map(|page| page.document)
        {
            self.input_surfaces.SurfaceRemoved(document);
        }
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
        self.history_for(self.tabs.active.id, delta)
    }
    fn history_for(&mut self, tab_id: u64, delta: isize) -> io::Result<()> {
        let Some(tab) = self.tabs.ordered().into_iter().find(|tab| tab.id == tab_id) else {
            return Ok(());
        };
        if let Some(next) = tab.history_index_for_offset(delta) {
            // NavigateToExistingPendingEntry cancels a slow pending load when
            // traversal returns to the still-live last committed document.
            if next == tab.history_index && tab.pending_navigation.is_some() {
                self.cancel_pending_navigation(tab_id);
                return Ok(());
            }
            let item = tab.history[next].clone();
            if tab
                .page
                .as_ref()
                .is_some_and(|page| tab.history_document_generation == Some(page.document.1))
                && tab.history.get(tab.history_index).is_some_and(|current| {
                    current.document_sequence_number == item.document_sequence_number
                })
            {
                let state = navigation_history::SessionHistoryState {
                    offset: next as i32,
                    length: tab.history.len() as i32,
                };
                self.cancel_pending_navigation(tab_id);
                if let Some(page) = self
                    .tabs
                    .ordered()
                    .into_iter()
                    .find(|tab| tab.id == tab_id)
                    .and_then(|tab| tab.page.as_ref())
                {
                    page.send(page::Command::TraverseHistory { item, state });
                }
                return Ok(());
            }
            let mut request = url_loader::URLRequest {
                url: tab.history_url(&item).to_owned(),
                ..Default::default()
            };
            browser::page::RestoreHistoryRequest(&mut request, &item);
            tab.prepare_navigation(&mut request, FrameLoadType::BackForward);
            request.navigation.as_mut().unwrap().history_item = Some(item);
            if tab_id == self.tabs.active.id {
                self.navigate(request, false, false)?;
            } else {
                self.navigate_background(tab_id, request, false)?;
            }
        }
        Ok(())
    }
    fn reload(&mut self) -> io::Result<()> {
        // NavigationControllerImpl::Reload discards pending navigations once
        // a committed entry exists, then reloads that committed entry.
        if !self.tabs.active.history.is_empty() {
            self.cancel_pending_navigation(self.tabs.active.id);
        }
        let mut request = url_loader::URLRequest {
            url: self.tabs.active.location.clone(),
            ..Default::default()
        };
        self.tabs
            .active
            .prepare_navigation(&mut request, FrameLoadType::Reload);
        if let Some(item) = self.tabs.active.history.get(self.tabs.active.history_index) {
            browser::page::RestoreHistoryRequest(&mut request, item);
        }
        self.navigate(request, false, false)
    }
    fn navigate_background(
        &mut self,
        tab_id: u64,
        request: url_loader::URLRequest,
        record: bool,
    ) -> io::Result<()> {
        crate::options::normalize_address(&request.url)?;
        if self.prepare_navigation_state(tab_id, &request, record, false) {
            return Ok(());
        }
        self.navigate_background_prepared(tab_id, request, record)
    }
    fn navigate_background_prepared(
        &mut self,
        tab_id: u64,
        mut request: url_loader::URLRequest,
        record: bool,
    ) -> io::Result<()> {
        let location = crate::options::normalize_address(&request.url)?;
        let Some(tab) = self.tabs.background.iter_mut().find(|tab| tab.id == tab_id) else {
            return Ok(());
        };
        let generation = tab.document_generation + 1;
        tab.prepare_navigation(
            &mut request,
            if record {
                FrameLoadType::Standard
            } else {
                FrameLoadType::ReplaceCurrentItem
            },
        );
        tab.begin_navigation(generation, &request);
        request.url = match location.as_str() {
            "about:home" => crate::engine::data_url(crate::engine::HOME),
            "about:blank" => crate::engine::data_url("<!doctype html><html><body></body></html>"),
            _ => location.clone(),
        };
        tab.location = location.clone();
        tab.title = location_title(&location);
        tab.loading = true;
        self.refresh();
        self.scene();
        let page = match self.make_page(
            (tab_id, generation),
            request,
            location,
            false,
            std::env::var("RECHROM_TRANSPARENT_BACKGROUND").as_deref() == Ok("1"),
        ) {
            Ok(page) => page,
            Err(error) => {
                self.cancel_pending_navigation(tab_id);
                return Err(error);
            }
        };
        page.visible(false);
        page.send(page::Command::Active(false));
        if let Some(tab) = self.tabs.background.iter_mut().find(|tab| tab.id == tab_id) {
            tab.pending_navigation.as_mut().unwrap().page = Some(page);
        }
        self.scene();
        Ok(())
    }
    fn cancel_pending_navigation(&mut self, tab_id: u64) -> bool {
        let cancelled_preparation = self.navigation_preparations.remove(&tab_id).is_some();
        self.discard_pending_navigation(tab_id) || cancelled_preparation
    }
    fn discard_pending_navigation(&mut self, tab_id: u64) -> bool {
        let tab = if self.tabs.active.id == tab_id {
            &mut self.tabs.active
        } else if let Some(tab) = self.tabs.background.iter_mut().find(|tab| tab.id == tab_id) {
            tab
        } else {
            return false;
        };
        let Some(generation) = tab.pending_navigation.as_ref().map(|pending| {
            if let Some(page) = &pending.page {
                page.send(page::Command::StopLoading);
            }
            pending.generation
        }) else {
            return false;
        };
        tab.discard_pending_navigation(generation);
        if let Some(page) = &tab.page {
            page.send(page::Command::StopLoading);
        }
        tab.loading = false;
        if self.tabs.active.id == tab_id {
            self.sync_toolbar(true, false);
            (self.output.notify)(UserEvent::Location(self.tabs.active.location.clone()));
        } else {
            self.refresh();
        }
        self.scene();
        true
    }
    fn focus_address(&mut self) {
        self.dismiss_validation_message();
        self.input_surfaces.ReleaseMouseCapture();
        self.dismiss_popup();
        self.toolbar_focused = true;
        self.active_pages();
        self.toolbar_revision = self.toolbar_revision.max(self.toolbar.patch(Patch::Focus {
            id: "address".into(),
            select_all: true,
        }));
        self.scene();
    }
    fn dismiss_validation_message(&mut self) {
        if self.validation_overlay.take().is_some() {
            self.scene();
        }
    }
    fn validation_message(
        &mut self,
        document: Document,
        update: browser::page::ValidationMessageUpdate,
    ) -> io::Result<()> {
        use browser::page::ValidationMessageUpdate;
        match update {
            ValidationMessageUpdate::Hide { control, immediate } => {
                if self
                    .validation_overlay
                    .as_ref()
                    .is_some_and(|overlay| overlay.owner == document && overlay.control == control)
                {
                    if immediate {
                        self.dismiss_validation_message();
                    } else {
                        self.start_hiding_validation_message();
                    }
                }
            }
            ValidationMessageUpdate::Show {
                control,
                anchor,
                message,
            } => {
                if !self
                    .tabs
                    .active
                    .page
                    .as_ref()
                    .is_some_and(|page| page.document == document)
                    || self.toolbar_focused
                    || !self.active
                {
                    return Ok(());
                }
                if self
                    .validation_overlay
                    .as_ref()
                    .is_some_and(|overlay| overlay.hiding_deadline.is_some())
                {
                    self.validation_overlay.take();
                }
                let viewport = self.content_viewport();
                if let Some(overlay) = self
                    .validation_overlay
                    .as_mut()
                    .filter(|overlay| overlay.owner == document && overlay.control == control)
                {
                    overlay.anchor = anchor;
                    let mut patches = overlay.size.map_or_else(Vec::new, |size| {
                        crate::chrome::validation_bubble_position(anchor, size, viewport)
                    });
                    if overlay.message != message {
                        patches.push(Patch::Text {
                            id: "main-message".into(),
                            value: String::from_utf16_lossy(&message),
                        });
                        overlay.message = message;
                    }
                    overlay.page.patches(patches);
                } else {
                    self.validation_generation += 1;
                    let html = crate::chrome::validation_bubble_document(&message);
                    let page = self.make_page(
                        (u64::MAX - 1, self.validation_generation),
                        url_loader::URLRequest {
                            url: crate::engine::data_url(&html),
                            ..Default::default()
                        },
                        "browser://validation-message".into(),
                        true,
                        true,
                    )?;
                    self.validation_overlay = Some(ValidationOverlay {
                        owner: document,
                        control,
                        anchor,
                        size: None,
                        hiding_deadline: None,
                        message,
                        page,
                    });
                }
                self.scene();
            }
        }
        Ok(())
    }
    fn expire_validation_message(&mut self, now: Instant) {
        if self.validation_overlay.as_ref().is_some_and(|overlay| {
            overlay
                .hiding_deadline
                .is_some_and(|deadline| deadline <= now)
        }) {
            self.dismiss_validation_message();
        }
    }
    fn start_hiding_validation_message(&mut self) {
        let height = self.content_viewport().height as f64 / self.viewport.scale;
        if let Some(overlay) = self.validation_overlay.as_mut() {
            if overlay.hiding_deadline.is_some() {
                return;
            }
            let bottom = overlay.size.is_some_and(|(_, bubble_height)| {
                height - overlay.anchor.y - overlay.anchor.height < bubble_height
            });
            overlay.page.patch(Patch::Attribute {
                id: "container".into(),
                name: "class".into(),
                value: Some(
                    if bottom {
                        "hiding bottom-arrow"
                    } else {
                        "hiding"
                    }
                    .into(),
                ),
            });
            // ValidationMessageClientImpl::HideValidationMessage's reset
            // timer equals the source #container.hiding opacity transition.
            overlay.hiding_deadline = Some(Instant::now() + Duration::from_micros(133_330));
        }
    }
    fn validation_bubble_size(&mut self, document: Document, size: (f64, f64)) {
        let viewport = self.content_viewport();
        if let Some(overlay) = self.validation_overlay.as_mut().filter(|overlay| {
            overlay.page.document == document && overlay.hiding_deadline.is_none()
        }) {
            overlay.size = Some(size);
            overlay
                .page
                .patches(crate::chrome::validation_bubble_position(
                    overlay.anchor,
                    size,
                    viewport,
                ));
        }
    }
    fn popup(&mut self, kind: &str) -> io::Result<()> {
        self.dismiss_validation_message();
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
            "reload" if self.tabs.active.loading => {
                if !self.cancel_pending_navigation(self.tabs.active.id) {
                    if let Some(page) = &self.tabs.active.page {
                        page.send(page::Command::StopLoading);
                    }
                }
                Ok(())
            }
            "reload" => self.reload(),
            "home" => self.navigate(
                url_loader::URLRequest {
                    url: "about:home".into(),
                    ..Default::default()
                },
                true,
                false,
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
                self.dismiss_popup();
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
                false,
            ),
            Command::Reload => self.reload(),
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
                if active {
                    self.refresh_interaction_text_settings()?;
                }
                self.active = active;
                if !active {
                    self.input_surfaces.ReleaseMouseCapture();
                    self.dismiss_popup();
                    self.dismiss_validation_message();
                    self.scene();
                }
                self.active_pages();
                self.scene();
                Ok(())
            }
            Command::Resize(resize) => {
                let _scope = browser_tracing::scope(browser_tracing::Context {
                    target_id: 1,
                    task_id: resize.trace_id(),
                    ..Default::default()
                });
                let started = Instant::now();
                browser_tracing::interval(
                    "resize",
                    "ResizeHostQueue",
                    resize.queued_at,
                    started,
                    &[("samples", resize.samples as f64)],
                );
                let mut trace = browser_tracing::span("resize", "Host.Resize");
                let viewport = resize.viewport;
                trace.set("samples", resize.samples as f64);
                trace.set("width", viewport.width as f64);
                trace.set("height", viewport.height as f64);
                trace.set("device_scale", viewport.scale);
                if root_resize_changes_visual_properties(self.viewport, viewport) {
                    let previous_toolbar = Viewport {
                        height: self.viewport.toolbar_pixels(),
                        ..self.viewport
                    };
                    self.viewport = viewport;
                    self.dismiss_popup();
                    let toolbar_viewport = Viewport {
                        height: viewport.toolbar_pixels(),
                        ..viewport
                    };
                    if toolbar_viewport != previous_toolbar {
                        self.toolbar.resize(resize.with_viewport(toolbar_viewport));
                    }
                    if let Some(page) = &self.tabs.active.page {
                        page.resize(resize.with_viewport(self.content_viewport()));
                    }
                    if let Some(overlay) = &self.validation_overlay {
                        overlay
                            .page
                            .resize(resize.with_viewport(self.content_viewport()));
                    }
                    self.tabs.active.viewport = Some(viewport);
                    // BrowserView updates the root geometry synchronously.
                    // Publish that geometry now. Existing toolbar and
                    // WebContents Surfaces remain source-sized resize
                    // fallbacks; their new-size frames replace them when
                    // ready. The later DragRegions event refreshes metadata
                    // without being the resize geometry gate.
                    self.scene();
                    trace.set("accepted", 1.0);
                } else {
                    trace.set("accepted", 0.0);
                    trace.set(
                        "duplicate",
                        (viewport.valid() && viewport == self.viewport) as u8 as f64,
                    );
                }
                Ok(())
            }
            Command::WidgetScreenOrigin {
                x_dip,
                y_dip,
                scale,
            } => {
                if !scale.is_finite() || scale <= 0.0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid native window scale",
                    ));
                }
                self.widget_screen_origin = Some((x_dip, y_dip, scale));
                self.publish_widget_screen_geometry();
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
            Command::CopyOutput {
                expected_viewport,
                target,
                reply,
            } => {
                let fresh_documents = match target {
                    crate::engine::CopyOutputTarget::Page => self
                        .tabs
                        .active
                        .page
                        .as_ref()
                        .map(|page| vec![page.document])
                        .unwrap_or_default(),
                    crate::engine::CopyOutputTarget::Toolbar => vec![self.toolbar.document],
                    crate::engine::CopyOutputTarget::Root => {
                        let mut documents = vec![self.toolbar.document];
                        if let Some(page) = &self.tabs.active.page {
                            documents.push(page.document);
                        }
                        if let Some(popup) = &self.popup {
                            documents.push(popup.page.document);
                        }
                        if let Some(overlay) = &self.validation_overlay {
                            if self
                                .tabs
                                .active
                                .page
                                .as_ref()
                                .is_some_and(|page| page.document == overlay.owner)
                            {
                                documents.push(overlay.page.document);
                            }
                        }
                        documents
                    }
                };
                // Register the display-side minimum before asking any Page to
                // advance. The window-runtime channel therefore observes this
                // barrier before the resulting CompositorFrames, even though
                // each Page producer remains independently sequenced.
                self.frames
                    .send(window_runtime::Command::Display(
                        crate::display::Message::CopyOutput {
                            expected_viewport,
                            target,
                            fresh_documents: fresh_documents.clone(),
                            reply,
                        },
                    ))
                    .map_err(|_| io::Error::other("Display stopped"))?;
                for document in fresh_documents {
                    if document == self.toolbar.document {
                        self.toolbar.copy_output_fence();
                    } else if let Some(page) = self
                        .tabs
                        .active
                        .page
                        .as_ref()
                        .filter(|page| page.document == document)
                    {
                        page.copy_output_fence();
                    } else if let Some(popup) = self
                        .popup
                        .as_ref()
                        .filter(|popup| popup.page.document == document)
                    {
                        popup.page.copy_output_fence();
                    } else if let Some(overlay) = self
                        .validation_overlay
                        .as_ref()
                        .filter(|overlay| overlay.page.document == document)
                    {
                        overlay.page.copy_output_fence();
                    }
                }
                Ok(())
            }
            Command::TargetDrag(target) => {
                self.target_drag(target);
                Ok(())
            }
            Command::Input(input) => self.input(input, Instant::now(), 1),
            Command::Wheel(sample) => {
                let (delta_x, delta_y) = match &sample.delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        (f64::from(*x), f64::from(*y))
                    }
                    winit::event::MouseScrollDelta::PixelDelta(point) => (point.x, point.y),
                };
                // MouseWheelPhaseHandler's accepted sample is the common
                // native/replay boundary.  Timestamp it with the platform
                // observation rather than this owner-task dequeue so the
                // frame-aligned queue and eventual presentation retain one
                // input identity across threads.
                let input_id = browser_tracing::instant_id(sample.observed_at);
                let _input_scope = browser_tracing::enabled().then(|| {
                    browser_tracing::scope(browser_tracing::Context {
                        target_id: 1,
                        input_id,
                        ..Default::default()
                    })
                });
                browser_tracing::instant_at(
                    "input",
                    "WheelSample",
                    sample.observed_at,
                    &[
                        ("delta_x", delta_x),
                        ("delta_y", delta_y),
                        (
                            "phase",
                            sample
                                .macos_metadata
                                .as_ref()
                                .map_or(0.0, |event| event.phase as f64),
                        ),
                        (
                            "momentum_phase",
                            sample
                                .macos_metadata
                                .as_ref()
                                .map_or(0.0, |event| event.momentum_phase as f64),
                        ),
                        ("position_x", sample.position.x),
                        ("position_y", sample.position.y),
                        ("scale_factor", sample.scale),
                        (
                            "precise",
                            matches!(&sample.delta, winit::event::MouseScrollDelta::PixelDelta(_))
                                as u8 as f64,
                        ),
                    ],
                );
                browser_tracing::instant(
                    "input",
                    "WheelDispatched",
                    &[
                        ("delta_x", delta_x),
                        ("delta_y", delta_y),
                        (
                            "phase",
                            sample
                                .macos_metadata
                                .as_ref()
                                .map_or(0.0, |event| event.phase as f64),
                        ),
                        (
                            "momentum_phase",
                            sample
                                .macos_metadata
                                .as_ref()
                                .map_or(0.0, |event| event.momentum_phase as f64),
                        ),
                    ],
                );
                let queued_at = sample.observed_at;
                for command in self.wheel_input.accept_wheel_sample(sample, Instant::now()) {
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
    fn dismiss_popup(&mut self) {
        if let Some(popup) = self.popup.take() {
            self.input_surfaces.SurfaceRemoved(popup.page.document);
        }
    }
    fn surface_route(&self, document: Document) -> Option<SurfaceRoute> {
        self.input_surface(document).map(|_| SurfaceRoute {
            document,
            origin: Offset {
                x: 0.,
                y: if document == self.toolbar.document {
                    0.
                } else {
                    TOOLBAR_HEIGHT
                },
            },
        })
    }
    fn input_surface_at(&self, point: Offset) -> Option<SurfaceRoute> {
        let content_origin = Offset {
            x: 0.0,
            y: TOOLBAR_HEIGHT,
        };
        if let Some(popup) = &self.popup {
            if point.x >= popup.x
                && point.x < popup.x + popup.width
                && point.y >= TOOLBAR_HEIGHT
                && point.y < TOOLBAR_HEIGHT + popup.height
            {
                return Some(SurfaceRoute {
                    document: popup.page.document,
                    origin: content_origin,
                });
            }
        }
        if point.y < TOOLBAR_HEIGHT {
            Some(SurfaceRoute {
                document: self.toolbar.document,
                origin: Offset::default(),
            })
        } else {
            self.tabs.active.page.as_ref().map(|page| SurfaceRoute {
                document: page.document,
                origin: content_origin,
            })
        }
    }

    fn input_surface(&self, document: Document) -> Option<&Handle> {
        if document == self.toolbar.document {
            return Some(&self.toolbar);
        }
        if let Some(popup) = &self.popup {
            if document == popup.page.document {
                return Some(&popup.page);
            }
        }
        self.tabs
            .active
            .page
            .as_ref()
            .filter(|page| page.document == document)
    }

    fn route_typed_surface_input(
        &mut self,
        mut input: InputEvent,
        queued_at: Instant,
        samples: usize,
    ) {
        use crate::input_surface_routing::{TranslateGesture, TranslatePointer, TranslateTouch};
        let point = match &input {
            InputEvent::Wheel(event) => Some(event.position),
            InputEvent::Pointer(event) => Some(event.position),
            InputEvent::Touch(event) => event.points.first().map(|point| point.position),
            InputEvent::Gesture(event) => {
                let point = event.PositionInRootFrame();
                Some(Offset {
                    x: point.x() as f64,
                    y: point.y() as f64,
                })
            }
            _ => unreachable!(),
        };
        let hit = point.and_then(|point| self.input_surface_at(point));
        let route = match &input {
            InputEvent::Wheel(event) => self.input_surfaces.Wheel(event, hit),
            InputEvent::Pointer(event) => self.input_surfaces.Pointer(event, hit),
            InputEvent::Touch(event) => self.input_surfaces.Touch(event, hit),
            InputEvent::Gesture(event) => self.input_surfaces.Gesture(event, hit),
            _ => unreachable!(),
        };
        let Some(route) = route else {
            return;
        };
        if self.input_surface(route.document).is_none() {
            self.input_surfaces.SurfaceRemoved(route.document);
            return;
        }
        let pressed = matches!(&input, InputEvent::Pointer(event) if event.r#type == PointerEventType::kDown)
            || matches!(&input, InputEvent::Touch(event) if event.r#type == TouchEventType::kStart);
        if pressed {
            let popup = self.popup.as_ref().map(|popup| popup.page.document);
            if popup != Some(route.document) {
                let mut scene_changed = false;
                if let Some(document) = popup {
                    self.input_surfaces.SurfaceRemoved(document);
                    self.dismiss_popup();
                    scene_changed = true;
                }
                let toolbar_focused = route.document == self.toolbar.document;
                if self.toolbar_focused != toolbar_focused {
                    self.toolbar_focused = toolbar_focused;
                    self.active_pages();
                    scene_changed = true;
                }
                if scene_changed {
                    self.scene();
                }
            }
        }
        // Transform only the platform-neutral ingress value. Queue admission
        // creates the canonical packet once; Page subsequently borrows it.
        match &mut input {
            InputEvent::Wheel(event) => {
                event.position.x -= route.origin.x;
                event.position.y -= route.origin.y;
            }
            InputEvent::Pointer(event) => TranslatePointer(event, route.origin),
            InputEvent::Touch(event) => TranslateTouch(event, route.origin),
            InputEvent::Gesture(event) => TranslateGesture(event, route.origin),
            _ => unreachable!(),
        }
        if let Some(page) = self.input_surface(route.document) {
            page.input(input, queued_at, samples);
        }
    }

    fn send_target_drag(
        &self,
        route: SurfaceRoute,
        target: crate::native_drag::Target,
        action: interaction::default::core::page::drag_controller::DragTargetAction,
    ) {
        let mut target = target.for_request();
        target.position.0 -= route.origin.x as f32;
        target.position.1 -= route.origin.y as f32;
        if let Some(page) = self.input_surface(route.document) {
            page.send(page::Command::SystemDragTarget { target, action });
        } else {
            target
                .operation
                .store(0, std::sync::atomic::Ordering::Release);
        }
    }
    fn target_drag(&mut self, target: crate::native_drag::Target) {
        use interaction::default::core::page::drag_controller::DragTargetAction as A;
        use winit::event::NativeDragDropPhase as P;
        if target.phase == P::Enter {
            if let Some(old) = self.target_drag.take() {
                if let Some(route) = old.route {
                    self.send_target_drag(route, old.last, A::Leave);
                }
            }
            let Some(data) = target.data.clone() else {
                target
                    .operation
                    .store(0, std::sync::atomic::Ordering::Release);
                return;
            };
            self.target_drag = Some(NativeDropRoute {
                data,
                route: None,
                last: target.clone(),
            });
        }
        let Some(mut session) = self.target_drag.take() else {
            target
                .operation
                .store(0, std::sync::atomic::Ordering::Release);
            return;
        };
        if target.phase == P::Leave {
            if let Some(route) = session.route {
                self.send_target_drag(route, target, A::Leave);
            }
            return;
        }
        let hit = self.input_surface_at(Offset {
            x: target.position.0 as f64,
            y: target.position.1 as f64,
        });
        if session.route != hit {
            if let Some(route) = session.route {
                self.send_target_drag(route, target.clone(), A::Leave);
            }
            if let Some(route) = hit {
                self.send_target_drag(
                    route,
                    target.clone(),
                    A::Enter {
                        data: session.data.clone(),
                        source_operations: target.source_operations,
                    },
                );
            }
            session.route = hit;
        } else if target.phase == P::Over {
            if let Some(route) = hit {
                self.send_target_drag(
                    route,
                    target.clone(),
                    A::Update {
                        source_operations: target.source_operations,
                    },
                );
            }
        }
        if target.phase == P::Drop {
            if let Some(route) = hit {
                let operation = match target.operation.load(std::sync::atomic::Ordering::Acquire) {
                    1 => interaction::dispatch::core::clipboard::data_transfer::DragOperation::Copy,
                    2 => interaction::dispatch::core::clipboard::data_transfer::DragOperation::Link,
                    16 => {
                        interaction::dispatch::core::clipboard::data_transfer::DragOperation::Move
                    }
                    _ => interaction::dispatch::core::clipboard::data_transfer::DragOperation::None,
                };
                self.send_target_drag(route, target, A::Drop { operation });
            }
        } else {
            session.last = target;
            self.target_drag = Some(session);
        }
    }
    fn input(
        &mut self,
        mut input: InputEvent,
        queued_at: Instant,
        samples: usize,
    ) -> io::Result<()> {
        if let InputEvent::Key(key) = &input {
            let current = self
                .popup
                .as_ref()
                .map(|popup| popup.page.document)
                .or_else(|| {
                    if self.toolbar_focused {
                        Some(self.toolbar.document)
                    } else {
                        self.tabs.active.page.as_ref().map(|page| page.document)
                    }
                });
            let Some(target) = self.input_surfaces.Keyboard(key, current) else {
                return Ok(());
            };
            if Some(target) != current {
                let origin = self.input_surface(target).or_else(|| {
                    self.tabs
                        .background
                        .iter()
                        .find_map(|tab| tab.page.as_ref().filter(|page| page.document == target))
                });
                if let Some(origin) = origin {
                    origin.input(input, queued_at, samples);
                }
                return Ok(());
            }
        }
        if let InputEvent::Mouse(event) = &mut input {
            let captured = self.input_surfaces.CapturedMouseTarget(event);
            if event.r#type == MouseEventType::kDown {
                if let Some(route) = captured.or_else(|| self.input_surface_at(event.position)) {
                    self.input_surfaces
                        .MouseDownTarget(route, event.buttons | event.button.ButtonsMask());
                }
            }
            if let Some(route) = captured {
                if self.input_surface(route.document).is_some() {
                    event.position.x -= route.origin.x;
                    event.position.y -= route.origin.y;
                    self.input_surface(route.document)
                        .unwrap()
                        .input(input, queued_at, samples);
                } else {
                    self.input_surfaces.SurfaceRemoved(route.document);
                }
                return Ok(());
            }
        }
        if let Some(popup) = &mut self.popup {
            if let InputEvent::Key(e) = &input {
                if e.r#type == KeyEventType::kDown {
                    if e.key == "Escape" {
                        self.dismiss_popup();
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
                            self.dismiss_popup();
                            self.scene();
                            return self.action(&action);
                        }
                    }
                }
            }
            // Popups are independent Page domains. Keyboard and IME input
            // belongs to the active popup before toolbar/content selection;
            // neither Page can synchronously dispatch into the other.
            if matches!(
                &input,
                InputEvent::Key(_) | InputEvent::Composition(_) | InputEvent::TextInput(_)
            ) {
                popup.page.input(input, queued_at, samples);
                return Ok(());
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
                    self.dismiss_popup();
                    self.scene();
                }
            }
        }
        if matches!(
            &input,
            InputEvent::Pointer(_)
                | InputEvent::Touch(_)
                | InputEvent::Gesture(_)
                | InputEvent::Wheel(_)
        ) {
            self.route_typed_surface_input(input, queued_at, samples);
            return Ok(());
        }
        let toolbar = match &mut input {
            InputEvent::Mouse(e) => {
                let toolbar = if e.r#type == MouseEventType::kLeave {
                    self.in_toolbar
                } else {
                    e.position.y < TOOLBAR_HEIGHT
                };
                if e.r#type == MouseEventType::kDown && self.toolbar_focused != toolbar {
                    self.toolbar_focused = toolbar;
                    self.active_pages();
                    self.scene();
                }
                if self.in_toolbar != toolbar {
                    let leave = InputEvent::Mouse(MouseEvent {
                        r#type: MouseEventType::kLeave,
                        position: Offset { x: -1.0, y: -1.0 },
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
        if let Event::NavigationPrepared { token } = event {
            return self.continue_prepared_navigation(document, Some(token));
        }
        if matches!(&event, Event::Error(error) if error.terminal) {
            // A terminal producer may fail before running the save task. The
            // host still owns the continuation and can replace that producer.
            self.continue_prepared_navigation(document, None)?;
        }
        if let Event::HistoryStateUpdated(item) = event {
            let tab = if self.tabs.active.id == document.0 {
                Some(&mut self.tabs.active)
            } else {
                self.tabs
                    .background
                    .iter_mut()
                    .find(|tab| tab.id == document.0)
            };
            if let Some(tab) = tab {
                tab.update_history_state(item);
            }
            return Ok(());
        }
        let pending_tab = self
            .tabs
            .ordered()
            .into_iter()
            .find(|tab| {
                tab.id == document.0
                    && tab
                        .pending_navigation
                        .as_ref()
                        .is_some_and(|pending| pending.generation == document.1)
            })
            .map(|tab| tab.id);
        if let Some(tab_id) = pending_tab {
            match &event {
                Event::HistoryCommitted { .. } => {
                    let active = self.tabs.active.id == tab_id;
                    let tab = if active {
                        &mut self.tabs.active
                    } else {
                        self.tabs
                            .background
                            .iter_mut()
                            .find(|tab| tab.id == tab_id)
                            .unwrap()
                    };
                    if let Some(previous) = tab.commit_pending_navigation(document.1) {
                        self.input_surfaces.SurfaceRemoved(previous.document);
                        self.auxiliary_navigation_targets
                            .retain(|_, target| target.source != previous.document);
                    }
                    tab.viewport = Some(self.viewport);
                    if active {
                        self.set_active_pages();
                    } else if let Some(page) = &tab.page {
                        page.send(page::Command::Active(false));
                    }
                }
                Event::Error(error) => {
                    (self.output.notify)(UserEvent::PageError(error.clone()));
                    if error.terminal {
                        self.discard_pending_navigation(tab_id);
                    }
                    return Ok(());
                }
                Event::Status { loading: false, .. } => {
                    self.discard_pending_navigation(tab_id);
                    return Ok(());
                }
                _ => return Ok(()),
            }
        }
        if let Event::ValidationMessageVisibility(visibility) = &event {
            // Preserve the detach/hide notification even when a replacement
            // document already owns the active tab. Identity remains explicit.
            let event = visibility;
            // Shared native/headless semantic accessibility ingress. A
            // platform AX tree can consume this same immutable alert;
            // keep it observable even when that tree is unavailable.
            eprintln!(
                "rechrom-accessibility-event {}",
                serde_json::json!({
                    "kind": "validation-message-visibility",
                    "role": "alert",
                    "tab_id": document.0,
                    "document_generation": document.1,
                    "sequence": event.sequence,
                    "control": event.control,
                    "visible": event.visible,
                    "message": String::from_utf16_lossy(&event.message),
                })
            );
            return Ok(());
        }

        if let Event::ValidationBubbleSize { width, height } = &event {
            self.validation_bubble_size(document, (*width, *height));
            return Ok(());
        }

        if let Event::TouchDisposition {
            unique_touch_event_id,
            consumed,
        } = event
        {
            if consumed {
                self.input_surfaces
                    .HandledTouchStartOrFirstTouchMove(document, unique_touch_event_id);
            }
            return Ok(());
        }
        if let Event::Error(error) = event {
            // Deliver even a late error from a replaced/closed generation. It
            // remains useful to automation, but cannot mutate the new Page.
            (self.output.notify)(UserEvent::PageError(error.clone()));
            if error.terminal {
                self.auxiliary_navigation_targets
                    .retain(|_, target| target.source != document);
                if self.validation_overlay.as_ref().is_some_and(|overlay| {
                    overlay.owner == document || overlay.page.document == document
                }) {
                    self.dismiss_validation_message();
                }
                self.input_surfaces.SurfaceRemoved(document);
                if self
                    .popup
                    .as_ref()
                    .is_some_and(|p| p.page.document == document)
                {
                    self.dismiss_popup();
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
                    tab.loading = false;
                    tab.title = "Page error".into();
                    self.refresh();
                    self.scene();
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
        if failed && !matches!(&event, Event::HistoryCommitted { .. }) {
            return Ok(());
        }
        match event {
            Event::HistoryNavigation(delta) if !toolbar && !popup => {
                self.history_for(document.0, delta as isize)?;
            }
            Event::HistoryCommitted { item, load_type } if !toolbar && !popup => {
                let tab = if active {
                    &mut self.tabs.active
                } else {
                    self.tabs
                        .background
                        .iter_mut()
                        .find(|tab| tab.id == document.0)
                        .unwrap()
                };
                let location_changed = tab.pending_navigation.is_none()
                    && tab.location != item.url
                    && !(tab.location.starts_with("about:") && item.url.starts_with("data:"));
                if location_changed {
                    tab.location = item.url.clone();
                }
                tab.commit_history(item, load_type);
                tab.history_document_generation = Some(document.1);
                if let (Some(page), Some(item)) =
                    (tab.page.as_ref(), tab.history.get(tab.history_index))
                {
                    page.send(page::Command::SessionHistoryState {
                        item: item.clone(),
                        state: navigation_history::SessionHistoryState {
                            offset: tab.history_index as i32,
                            length: tab.history.len() as i32,
                        },
                    });
                }
                if active && location_changed {
                    self.address();
                    (self.output.notify)(UserEvent::Location(self.tabs.active.location.clone()));
                }
                self.refresh();
                self.scene();
            }
            Event::ChildHistoryCommitted { item, load_type } if !toolbar && !popup => {
                let tab = if active {
                    &mut self.tabs.active
                } else {
                    self.tabs
                        .background
                        .iter_mut()
                        .find(|tab| tab.id == document.0)
                        .unwrap()
                };
                tab.commit_child_history(item, load_type);
                tab.history_document_generation = Some(document.1);
                if let (Some(page), Some(item)) =
                    (tab.page.as_ref(), tab.history.get(tab.history_index))
                {
                    page.send(page::Command::SessionHistoryState {
                        item: item.clone(),
                        state: navigation_history::SessionHistoryState {
                            offset: tab.history_index as i32,
                            length: tab.history.len() as i32,
                        },
                    });
                }
                self.refresh();
                self.scene();
            }
            Event::ValidationMessage(update) => self.validation_message(document, update)?,
            Event::CaptureMouseEventsToWidget(capture) => {
                if let Some(route) = self.surface_route(document) {
                    self.input_surfaces
                        .CaptureMouseEventsToWidget(route, capture && self.active);
                }
            }
            Event::DragRegions {
                regions,
                root_viewport,
            } if toolbar => {
                // The visual-properties acknowledgement belongs to the exact
                // root that produced it. A superseded toolbar frame must not
                // roll BrowserView geometry back after a newer resize.
                if root_viewport.is_some_and(|viewport| viewport != self.viewport) {
                    return Ok(());
                }
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
                if let Some(pending) = &mut tab.pending_navigation {
                    pending.previous_loading = loading;
                    if !pending.previous_location.starts_with("about:")
                        && !url.starts_with("data:")
                        && !url.is_empty()
                    {
                        pending.previous_location = url;
                    }
                    if !title.is_empty() && pending.previous_location != "about:home" {
                        pending.previous_title = title;
                    }
                    return Ok(());
                }
                tab.loading = loading;
                let changed = !tab.location.starts_with("about:")
                    && !url.starts_with("data:")
                    && !url.is_empty()
                    && tab.location != url;
                if changed {
                    tab.location = url;
                }
                if !title.is_empty() && tab.location != "about:home" {
                    tab.title = title;
                }
                if active && changed {
                    self.address();
                    (self.output.notify)(UserEvent::Location(self.tabs.active.location.clone()));
                }
                self.refresh();
                self.scene();
            }
            Event::DropNavigate(request) if !toolbar && !popup => {
                if request.foreground {
                    self.new_tab(request.request, false)?;
                } else {
                    self.new_background_tab(request.request)?;
                }
            }
            Event::AuxiliaryNavigationTarget(lookup) => {
                let request_id = lookup.request_id;
                let result = if toolbar || popup {
                    Ok(false)
                } else {
                    self.reserve_auxiliary_navigation_target(document, lookup)
                };
                let accepted = result.as_ref().is_ok_and(|accepted| *accepted);
                if let Some(source) = self.input_surface(document).or_else(|| {
                    self.tabs
                        .background
                        .iter()
                        .filter_map(|tab| tab.page.as_ref())
                        .find(|page| page.document == document)
                }) {
                    source.send(page::Command::CompleteAuxiliaryNavigationTarget {
                        request_id,
                        accepted,
                    });
                }
                result?;
            }
            Event::Navigate(mut request) if !toolbar && !popup => {
                use browser::page::NavigationTarget;
                use interaction::default::core::loader::navigation_policy::NavigationPolicy;
                if request
                    .host
                    .as_ref()
                    .is_some_and(|host| host.policy == NavigationPolicy::kNavigationPolicyDownload)
                {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "the browser host has no download service",
                    ));
                }
                if let Some(request_id) =
                    request.host.as_ref().and_then(|host| host.auxiliary_target)
                {
                    // The source may have detached, or the accepted target may
                    // have closed before its navigation continuation ran.
                    // Never recover by creating a second target.
                    if !self
                        .auxiliary_navigation_targets
                        .get(&request_id)
                        .is_some_and(|target| target.source == document)
                    {
                        return Ok(());
                    }
                    let target = self
                        .auxiliary_navigation_targets
                        .remove(&request_id)
                        .unwrap();
                    request.host = Some(target.host);
                    let tab = if self.tabs.active.id == target.tab_id {
                        Some(&mut self.tabs.active)
                    } else {
                        self.tabs
                            .background
                            .iter_mut()
                            .find(|tab| tab.id == target.tab_id)
                    };
                    let Some(tab) = tab else {
                        return Ok(());
                    };
                    tab.navigation_host = request.host;
                    if target.tab_id == self.tabs.active.id {
                        self.navigate(request.request, !request.replace_history, false)?;
                    } else {
                        self.navigate_background(
                            target.tab_id,
                            request.request,
                            !request.replace_history,
                        )?;
                    }
                    return Ok(());
                }
                let target_id = match &request.target {
                    NavigationTarget::NewContext => None,
                    NavigationTarget::Named(name) => self
                        .tabs
                        .ordered()
                        .into_iter()
                        .find(|tab| tab.name == *name)
                        .map(|tab| tab.id),
                    NavigationTarget::SelfFrame
                    | NavigationTarget::Parent
                    | NavigationTarget::Top => Some(document.0),
                };
                if target_id.is_none() {
                    let name = match request.target {
                        NavigationTarget::Named(name) => name,
                        _ => String::new(),
                    };
                    self.new_tab(request.request, false)?;
                    self.tabs.active.name = name;
                    self.tabs.active.navigation_host = request.host;
                } else if target_id == Some(self.tabs.active.id) {
                    self.tabs.active.navigation_host = request.host;
                    self.navigate(request.request, !request.replace_history, false)?;
                } else {
                    // Navigation belongs to the originating tab even when a
                    // newer host input has already selected another one.
                    let target_id = target_id.unwrap();
                    if let Some(tab) = self
                        .tabs
                        .background
                        .iter_mut()
                        .find(|tab| tab.id == target_id)
                    {
                        tab.navigation_host = request.host;
                    }
                    self.navigate_background(target_id, request.request, !request.replace_history)?;
                }
            }
            Event::ControlActivated { target } if toolbar || popup => {
                if let Some(action) = target.filter(|target| target != "address") {
                    if popup {
                        self.dismiss_popup();
                        self.scene();
                    }
                    self.action(&action)?;
                }
            }
            Event::Input {
                input,
                target: _,
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
                if toolbar && submit_address {
                    self.navigate(
                        url_loader::URLRequest {
                            url: value,
                            ..Default::default()
                        },
                        true,
                        false,
                    )?;
                }
            }
            Event::CursorChanged(cursor) => {
                if (toolbar && self.in_toolbar) || (!toolbar && active && !self.in_toolbar) || popup
                {
                    (self.output.notify)(UserEvent::CursorChanged(cursor));
                }
            }
            Event::StartSystemDrag(mut start) => {
                if toolbar || popup || active {
                    start.origin = (0., if toolbar { 0. } else { TOOLBAR_HEIGHT });
                    (self.output.notify)(UserEvent::StartSystemDrag(start));
                } else {
                    start.fail(crate::native_drag::Failure::BusyOrUnavailable);
                }
            }
            Event::Autoscroll(effect) => {
                let gestures = self.input_surfaces.Autoscroll(document, effect);
                if let Some(page) = self.input_surface(document).or_else(|| {
                    self.tabs
                        .background
                        .iter()
                        .filter_map(|tab| tab.page.as_ref())
                        .find(|page| page.document == document)
                }) {
                    for gesture in gestures {
                        page.input(InputEvent::Gesture(gesture), Instant::now(), 1);
                    }
                }
            }
            Event::EventHandlerChrome(effect) => {
                self.input_surfaces.EventHandlerChrome(document, effect)
            }
            Event::Feedback {
                mut caret,
                text_input,
            } => {
                if (toolbar && self.toolbar_focused) || (active && !self.toolbar_focused) {
                    if !toolbar {
                        if let Some(rect) = &mut caret {
                            rect.y += TOOLBAR_HEIGHT;
                        }
                    }
                    (self.output.notify)(UserEvent::CaretChanged(caret));
                    (self.output.notify)(UserEvent::TextInputStateChanged(text_input));
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
