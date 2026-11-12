//! Widget ingress leaves. Chromium widget_event_handler.cc:20-54 and
//! web_frame_widget_impl.cc:3507-3509. Retained services contain no Page loan.
use super::local_frame::LocalFrameDomain;
use super::*;
use interaction::adapter::{
    input_event::{ConvertedWebInputEvent, InputEvent, WebGestureEvent},
    native_interaction::{InteractionError, InteractionHost},
    page_event_handler_node::PageEventHandlerAtomicMoveState,
};
use interaction::core::events::web_input_event_conversion::{self as conversion, LocalFrameView};
use interaction::input::{
    base::time::time::TimeTicks,
    public::common::input::{
        web_input_event::WebInputEventPayload, web_keyboard_event::WebKeyboardEvent,
        web_mouse_event::WebMouseEvent, web_mouse_wheel_event::WebMouseWheelEvent,
        web_pointer_event::WebPointerEvent,
    },
};

/// LocalFrameView's mandatory LayoutShiftTracker. The concrete source owner
/// consumes the full payload because hovering pointerdown has distinct policy.
pub(super) trait LayoutShiftInputTracker {
    fn CurrentDocumentMatches(&self) -> bool;
    fn NotifyInput(&self, event: &dyn WebInputEventPayload);
}
/// ExistingAXObjectCache is optional in Chromium. Keeping the cache optional
/// preserves that gate without replacing a missing cache with another owner.
pub(super) trait TouchAccessibilityCache {
    fn OnTouchAccessibilityHover(&self, point: foundation::gfx::Point);
}
/// Nullable ChromeClientImpl services. These are absent unless an embedding
/// installs the corresponding real client; a notification is never fabricated.
pub(super) trait AutofillInputClient {
    fn WillDispatchPointerDown(&self, frame: u64, node: u64);
    fn DidDispatchMouseDown(&self, frame: u64, node: u64);
}
pub(super) trait RecordReplayInputClient {
    fn DidDispatchMouseDown(&self, frame: u64, node: u64);
}
pub(super) trait PrescientNetworking {
    fn IsDNSPrefetchEnabled(&self, frame: u64) -> bool;
    fn PrefetchDNS(&self, url: Option<&str>);
}
#[derive(Default)]
struct ToolTipCache {
    last_point: foundation::PhysicalOffset,
    last_text: Option<String>,
    last_node: Option<u128>,
}
#[derive(Default)]
pub(super) struct NativeEventHandlerHostState {
    pub plugins: crate::plugin_registry::PagePluginRegistry,
    pub layout_shift_trackers: RefCell<HashMap<u64, Rc<dyn LayoutShiftInputTracker>>>,
    pub accessibility_caches: RefCell<HashMap<u64, Rc<dyn TouchAccessibilityCache>>>,
    pub system_cursor_size_dip: Cell<Option<interaction::core::input::event_handler_cursor::SizeF>>,
    pub cursor: RefCell<Cursor>,
    pub autofill_clients: RefCell<HashMap<u64, Rc<dyn AutofillInputClient>>>,
    pub record_replay_clients: RefCell<HashMap<u64, Rc<dyn RecordReplayInputClient>>>,
    pub prescient_networking: RefCell<HashMap<u64, Rc<dyn PrescientNetworking>>>,
    tooltip: RefCell<ToolTipCache>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventHandlerChromeEffect {
    WillDispatchPointerDown {
        frame: u64,
        node: u64,
    },
    DidDispatchMouseDown {
        frame: u64,
        node: u64,
    },
    MouseDidMoveOverElement {
        frame: u64,
        node: Option<u64>,
        title: Option<String>,
        point: (f32, f32),
        modifiers: i32,
    },
    ClearToolTip {
        frame: u64,
    },
    ShowMouseOverUrl {
        frame: u64,
        url: Option<String>,
    },
    UpdateToolTip {
        frame: u64,
        text: Option<String>,
        direction: foundation::TextDirection,
    },
}

thread_local! {
    /// CurrentInputEvent is widget-thread state, including synchronous reentry.
    static CURRENT_INPUT: RefCell<Option<ConvertedWebInputEvent>> = const { RefCell::new(None) };
}
struct CurrentInputScope(Option<ConvertedWebInputEvent>);
pub(super) fn CurrentInputStartTime() -> Option<TimeTicks> {
    CURRENT_INPUT.with(|current| {
        let current = current.borrow();
        Some(match current.as_ref()? {
            ConvertedWebInputEvent::Mouse(e) => e.TimeStamp(),
            ConvertedWebInputEvent::Pointer(e) => e.TimeStamp(),
            ConvertedWebInputEvent::Touch(e) => e.TimeStamp(),
            ConvertedWebInputEvent::Gesture(e) => e.TimeStamp(),
            ConvertedWebInputEvent::Wheel(e) => e.TimeStamp(),
            ConvertedWebInputEvent::Keyboard(e) => e.TimeStamp(),
        })
    })
}
pub(super) fn CurrentInputNavigationModifiers(
) -> Option<interaction::default::core::loader::navigation_policy::NavigationEventModifiers> {
    use interaction::default::core::loader::navigation_policy::CurrentInputNavigationModifiers;
    CURRENT_INPUT.with(|current| {
        let event = current.borrow();
        match event.as_ref()? {
            ConvertedWebInputEvent::Mouse(event) => CurrentInputNavigationModifiers(event),
            ConvertedWebInputEvent::Pointer(event) => CurrentInputNavigationModifiers(event),
            ConvertedWebInputEvent::Touch(event) => CurrentInputNavigationModifiers(event),
            ConvertedWebInputEvent::Gesture(event) => CurrentInputNavigationModifiers(event),
            ConvertedWebInputEvent::Wheel(event) => CurrentInputNavigationModifiers(event),
            ConvertedWebInputEvent::Keyboard(event) => CurrentInputNavigationModifiers(event),
        }
    })
}
impl Drop for CurrentInputScope {
    fn drop(&mut self) {
        CURRENT_INPUT.with(|current| *current.borrow_mut() = self.0.take());
    }
}

pub(super) struct NativeEventHandlerHost {
    domain: Weak<LocalFrameDomain>,
    document: interaction::InteractionDocument,
}
impl NativeEventHandlerHost {
    pub(super) fn new(domain: &Rc<LocalFrameDomain>, token: u64) -> Rc<Self> {
        Rc::new(Self {
            domain: Rc::downgrade(domain),
            document: domain
                .Document(token)
                .expect("native host Document")
                .Handle(),
        })
    }
    fn Domain(&self) -> Result<Rc<LocalFrameDomain>, InteractionError> {
        self.domain.upgrade().ok_or(InteractionError::DetachedFrame)
    }
    fn Page(&self) -> Result<Rc<PageState>, InteractionError> {
        self.Domain()?
            .page
            .upgrade()
            .ok_or(InteractionError::DetachedFrame)
    }
    fn FrameView(&self) -> Result<InputConversionView, InteractionError> {
        if !self.HasRootView() {
            return Err(InteractionError::DetachedFrame);
        }
        Ok(InputConversionView)
    }
}
/// The current local root has no active VisualViewport or elastic overscroll.
/// This is the same source state used by PageScrollFrameState. CSS zoom is not
/// LocalFrameView::InputEventsScaleFactor (local_frame_view.cc:2082-2085).
struct InputConversionView;
impl LocalFrameView for InputConversionView {
    fn LocalFrameRootView(&self) -> Option<&dyn LocalFrameView> {
        Some(self)
    }
    fn InputEventsScaleFactor(&self) -> f32 {
        1.0
    }
    fn VisualViewportVisibleRectOrigin(&self) -> foundation::gfx::PointF {
        Default::default()
    }
    fn ElasticOverscroll(&self) -> conversion::Vector2dF {
        Default::default()
    }
}
impl InteractionHost for NativeEventHandlerHost {
    fn HasLocalRoot(&self) -> bool {
        self.domain.upgrade().is_some_and(|domain| {
            super::local_frame_event_handler::NativeEventHandlerFrame::new(&domain, domain.root)
                .IsAttached()
        })
    }
    fn HandlePopupKeyEvent(
        &self,
        _event: &WebKeyboardEvent,
    ) -> Result<
        Option<
            interaction::dispatch::public::platform::web_input_event_result::WebInputEventResult,
        >,
        InteractionError,
    > {
        // WebFrameWidgetImpl::HandleKeyEvent/HandleCharEvent forward to a
        // co-located PagePopup when ChromeClient owns one. Rechrom presents
        // popups as independent Page domains: BrowserHost selects the active
        // popup Page before dispatch, so this widget has no local PagePopup.
        Ok(None)
    }
    fn PagePopupCopyPasteEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::PagePopupCopyPasteEnabled()
    }
    fn FocusedPluginKeyboardState(
        &self,
    ) -> Result<
        interaction::core::frame::web_frame_widget_impl::FocusedPluginKeyboardState,
        InteractionError,
    > {
        use interaction::core::frame::web_frame_widget_impl::FocusedPluginKeyboardState;
        let domain = self.Domain()?;
        let root =
            super::local_frame_event_handler::NativeEventHandlerFrame::new(&domain, domain.root);
        let Some(token) = root
            .FocusedLocalFrameInWidgetIdentity()
            .and_then(|identity| domain.EventHandlerFrameToken(&identity))
        else {
            return Ok(FocusedPluginKeyboardState::NotEmbeddedObject);
        };
        let Some(focused) = domain
            .FocusInteraction(token)
            .and_then(|interaction| interaction.DocumentActiveElementNode())
        else {
            return Ok(FocusedPluginKeyboardState::NotEmbeddedObject);
        };
        let embedded = domain
            .WithLayout(token, |layout| {
                layout
                    .EventRegistryLayout(Some(focused))
                    .is_some_and(|layout| layout.IsEmbeddedObject())
            })
            .unwrap_or(false);
        if !embedded {
            return Ok(FocusedPluginKeyboardState::NotEmbeddedObject);
        }
        let document = domain
            .Document(token)
            .ok_or(InteractionError::DetachedFrame)?;
        let owner = {
            let document = document.borrow();
            let document = document.GetDocument();
            let node = document
                .FindNodeById(focused)
                .ok_or(InteractionError::DetachedFrame)?;
            document.NodeHandle(node)
        };
        let page = self.Page()?;
        let supports_keyboard_focus =
            match page.native_event_handler_host.plugins.LookupPlugin(owner) {
                crate::plugin_registry::PluginInstanceLookup::Unsupported => None,
            };
        Ok(FocusedPluginKeyboardState::EmbeddedObject {
            supports_keyboard_focus,
        })
    }
    fn HasRootView(&self) -> bool {
        self.domain.upgrade().is_some_and(|domain| {
            super::local_frame_event_handler::NativeEventHandlerFrame::new(&domain, domain.root)
                .View()
                .is_some()
        })
    }
    fn EnterCurrentInputEvent(
        &self,
        event: &ConvertedWebInputEvent,
    ) -> Result<Box<dyn std::any::Any>, InteractionError> {
        let previous = CURRENT_INPUT.with(|current| current.replace(Some(event.clone())));
        Ok(Box::new(CurrentInputScope(previous)))
    }
    fn InputTimestamp(&self, input: &InputEvent) -> Result<TimeTicks, InteractionError> {
        let page = self.Page()?;
        let instant = match input {
            InputEvent::Touch(event) => event.timestamp,
            InputEvent::Key(event) => event.timestamp,
            InputEvent::Wheel(event) => event.timestamp,
            InputEvent::Gesture(event) => return Ok(event.TimeStamp()),
            _ => None,
        }
        .unwrap_or_else(std::time::Instant::now);
        use interaction::adapter::host_event_clock::HostInputClock;
        let timestamp = page.rendering.borrow().executor.TimeForInstant(instant);
        Ok(timestamp)
    }
    fn NotifyInput(&self, event: &dyn WebInputEventPayload) -> Result<(), InteractionError> {
        let domain = self.Domain()?;
        let page = self.Page()?;
        if self.HasRootView() {
            let replace_tracker = page
                .native_event_handler_host
                .layout_shift_trackers
                .borrow()
                .get(&domain.root)
                .is_none_or(|tracker| !tracker.CurrentDocumentMatches());
            if replace_tracker {
                page.native_event_handler_host
                    .layout_shift_trackers
                    .borrow_mut()
                    .insert(
                        domain.root,
                        super::layout_shift_input::PageLayoutShiftInputTracker::new(
                            &domain,
                            domain.root,
                        ),
                    );
            }
            let tracker = page
                .native_event_handler_host
                .layout_shift_trackers
                .borrow()
                .get(&domain.root)
                .cloned()
                .ok_or_else(|| {
                    InteractionError::SourceInput(
                        "LocalFrameView::LayoutShiftTracker is not bound".into(),
                    )
                })?;
            tracker.NotifyInput(event);
        }
        page.window_performance_timing
            .EventTimingSource()
            .ResponsivenessMetrics()
            .borrow_mut()
            .SetCurrentInteractionEventQueuedTimestamp(event.Event().QueuedTimeStamp());
        Ok(())
    }
    fn TouchAccessibilityHover(&self, event: &WebMouseEvent) -> Result<(), InteractionError> {
        let domain = self.Domain()?;
        let page = self.Page()?;
        let transformed = self.TransformMouseEvent(event)?;
        let point = transformed.PositionInRootFrame();
        let location =
            interaction::hit_test::core::layout::hit_test_location::HitTestLocation::FromPoint(
                foundation::gfx::Point::new(point.x().floor() as i32, point.y().floor() as i32),
            );
        use interaction::adapter::page_pointer_hit_test::PagePointerGeometry;
        let mut hit = super::pointer_geometry::PagePointerGeometryProvider(Rc::downgrade(&page))
            .HitTest(
                location,
                interaction::hit_test::core::layout::hit_test_request::RequestType::kReadOnly
                    as u32
                    | interaction::hit_test::core::layout::hit_test_request::RequestType::kActive
                        as u32,
            )
            .map_err(|error| {
                InteractionError::SourceInput(format!("accessibility hover: {error:?}"))
            })?
            .result;
        hit.SetToShadowHostIfInUAShadowRoot();
        if hit.InnerNodeFrame().is_some() {
            let token = hit.InnerNode().and_then(|node| {
                let id = node.DOMNodeId()?;
                let identity = node.DOMIdentity();
                std::iter::once(domain.root)
                    .chain(domain.frames.borrow().keys().copied())
                    .find(|token| {
                        domain.Document(*token).is_some_and(|document| {
                            let owner = document.borrow();
                            let document = owner.GetDocument();
                            document.FindNodeById(id).is_some_and(|index| {
                                document.NodeHandle(index).Identity() == identity
                            })
                        })
                    })
            });
            if let Some(cache) = token.and_then(|token| {
                page.native_event_handler_host
                    .accessibility_caches
                    .borrow()
                    .get(&token)
                    .cloned()
            }) {
                cache.OnTouchAccessibilityHover(hit.RoundedPointInInnerNodeFrame());
            }
        }
        Ok(())
    }
    fn TransformMouseEvent(
        &self,
        event: &WebMouseEvent,
    ) -> Result<WebMouseEvent, InteractionError> {
        Ok(conversion::TransformWebMouseEvent(
            Some(&self.FrameView()?),
            event,
        ))
    }
    fn TransformPointerEvent(
        &self,
        event: &WebPointerEvent,
    ) -> Result<WebPointerEvent, InteractionError> {
        Ok(conversion::TransformWebPointerEvent(
            Some(&self.FrameView()?),
            event,
        ))
    }
    fn TransformWheelEvent(
        &self,
        event: &WebMouseWheelEvent,
    ) -> Result<WebMouseWheelEvent, InteractionError> {
        Ok(conversion::TransformWebMouseWheelEvent(
            Some(&self.FrameView()?),
            event,
        ))
    }
    fn TransformGestureEvent(
        &self,
        event: &WebGestureEvent,
    ) -> Result<WebGestureEvent, InteractionError> {
        let view = self.FrameView()?;
        let mut event = event.clone();
        event.SetFrameScale(view.InputEventsScaleFactor());
        event.SetFrameTranslate(
            view.VisualViewportVisibleRectOrigin().OffsetFromOrigin() + view.ElasticOverscroll(),
        );
        Ok(event)
    }
}
impl PageEventHandlerAtomicMoveState for NativeEventHandlerHost {
    fn StatePreservingAtomicMoveInProgress(&self) -> bool {
        let moving = self
            .document
            .borrow()
            .GetDocument()
            .StatePreservingAtomicMoveInProgress();
        moving
    }
}

use interaction::adapter::{
    page_event_dispatcher::event_handler_frame_services::{
        PageEventHandlerChromeClient, PageEventHandlerCursorClient,
    },
    page_event_handler_binding::{PageEventHandlerEmbedding, PageEventHandlerSource},
    page_event_handler_frame::{PageEventHandlerFrameDomain, PageEventHandlerFrameInputClient},
    page_event_handler_hit_test::{
        PageEventHandlerCoordinates, PageEventHandlerHitResult, PageEventHandlerHitTestLifecycle,
    },
    page_event_handler_node::{PageEventHandlerNodeGeometry, PageEventHandlerResidentNodeGeometry},
    page_event_handler_services::PageEventHandlerTimerServices,
    page_pointer_hit_test::PagePointerGeometry,
};
use interaction::dispatch::core::dom::events::event::EventTargetRef;
use interaction::hit_test::core::layout::hit_test_location::HitTestLocation;

/// Stable frame connections only. The native binder lends listener/manager
/// projections synchronously and retains neither Page nor its mutable engines.
pub(super) struct NativeEventHandlerEmbedding {
    domain: Weak<LocalFrameDomain>,
    token: u64,
    frame: Rc<super::local_frame_event_handler::NativeEventHandlerFrame>,
    host: Rc<NativeEventHandlerHost>,
    geometry: Rc<dyn PagePointerGeometry>,
    timers: Rc<dyn PageEventHandlerTimerServices>,
}
impl NativeEventHandlerEmbedding {
    pub(super) fn new(domain: &Rc<LocalFrameDomain>, token: u64) -> Rc<Self> {
        let page = domain.page.upgrade().expect("live native frame Page");
        let replace_tracker = page
            .native_event_handler_host
            .layout_shift_trackers
            .borrow()
            .get(&token)
            .is_none_or(|tracker| !tracker.CurrentDocumentMatches());
        if replace_tracker {
            page.native_event_handler_host
                .layout_shift_trackers
                .borrow_mut()
                .insert(
                    token,
                    super::layout_shift_input::PageLayoutShiftInputTracker::new(domain, token),
                );
        }
        let geometry: Rc<dyn PagePointerGeometry> = if token == domain.root {
            Rc::new(super::pointer_geometry::PagePointerGeometryProvider(
                Rc::downgrade(&page),
            ))
        } else {
            let frame = domain.Frame(token).expect("attached native child Frame");
            let runtime = frame
                .runtime
                .borrow()
                .clone()
                .expect("native child Document");
            super::local_frame_input::ChildInputGeometry::new(&frame, &runtime)
        };
        Rc::new(Self {
            domain: Rc::downgrade(domain),
            token,
            frame: super::local_frame_event_handler::NativeEventHandlerFrame::new(domain, token),
            host: NativeEventHandlerHost::new(domain, token),
            geometry,
            timers: super::popup_tasks::NativeEventHandlerTimerServices::new(domain, token),
        })
    }
    fn Domain(&self) -> Rc<LocalFrameDomain> {
        self.domain.upgrade().expect("live EventHandler domain")
    }
    fn Chrome(&self) -> Rc<NativeEventHandlerChrome> {
        Rc::new(NativeEventHandlerChrome {
            domain: self.domain.clone(),
            token: self.token,
        })
    }
}
impl PageEventHandlerEmbedding for NativeEventHandlerEmbedding {
    fn NodeGeometry(&self) -> Rc<dyn PageEventHandlerNodeGeometry> {
        Rc::new(PageEventHandlerResidentNodeGeometry(self.geometry.clone()))
    }
    fn AtomicMoveState(&self) -> Rc<dyn PageEventHandlerAtomicMoveState> {
        self.host.clone()
    }
    fn HitTestLifecycle(&self) -> Rc<dyn PageEventHandlerHitTestLifecycle> {
        self.frame.clone()
    }
    fn Coordinates(&self) -> Rc<dyn PageEventHandlerCoordinates> {
        self.frame.clone()
    }
    fn FrameDomain(&self) -> Rc<dyn PageEventHandlerFrameDomain> {
        self.frame.clone()
    }
    fn FrameInputClient(&self) -> Rc<dyn PageEventHandlerFrameInputClient> {
        self.frame.clone()
    }
    fn ChromeClient(&self) -> Rc<dyn PageEventHandlerChromeClient> {
        self.Chrome()
    }
    fn CursorClient(&self) -> Rc<dyn PageEventHandlerCursorClient> {
        self.Chrome()
    }
    fn MiddleClickAutoscrollFrame(
        &self,
    ) -> Option<Rc<dyn interaction::default::core::page::autoscroll_controller::AutoscrollFrame>>
    {
        Some(super::middle_click_autoscroll::MiddleClickFrame::new(
            &self.Domain(),
            self.token,
        ))
    }
    fn Host(&self) -> Rc<dyn InteractionHost> {
        self.host.clone()
    }
    fn Timers(&self) -> Rc<dyn PageEventHandlerTimerServices> {
        self.timers.clone()
    }
    fn InputWidgetScale(&self) -> f32 {
        let domain = self.Domain();
        let page = domain.page.upgrade().expect("live native Page");
        let scale = page.constraints.borrow().layout_zoom_factor as f32;
        scale
    }
    fn PageActive(&self) -> bool {
        self.domain
            .upgrade()
            .and_then(|domain| domain.page.upgrade())
            .is_some_and(|page| page.page_focused.get())
    }
    fn CursorVisible(&self) -> Option<bool> {
        // Current software widgets expose no OS cursor hiding mode. The
        // autoscroll controller's override is the canonical cursor gate.
        Some(true)
    }
    fn NativeFrameSource(&self, identity: &Rc<()>) -> Option<Rc<PageEventHandlerSource>> {
        self.frame.NativeFrameSource(identity)
    }
}

struct NativeEventHandlerChrome {
    domain: Weak<LocalFrameDomain>,
    token: u64,
}
impl NativeEventHandlerChrome {
    fn Owners(&self) -> (Rc<LocalFrameDomain>, Rc<PageState>) {
        let domain = self.domain.upgrade().expect("live ChromeClient domain");
        let page = domain.page.upgrade().expect("live ChromeClient Page");
        (domain, page)
    }
    fn Apply(&self, effect: EventHandlerChromeEffect) {
        self.Owners()
            .1
            .client
            .borrow_mut()
            .DidEventHandlerChromeEffect(effect);
    }
    // Element::DefaultToolTip and the form-control overrides. Validity and
    // FileList remain with their existing owners; this computes no new state.
    fn DefaultToolTip(&self, result: &PageEventHandlerHitResult) -> Option<String> {
        let id = result.Native().InnerNode()?.DOMNodeId()?;
        let (domain, _) = self.Owners();
        let identity =
            super::local_frame_event_handler::NativeEventHandlerFrame::new(&domain, self.token)
                .HitNodeFrameIdentity(result.Native().InnerNode()?)?;
        let token = domain.EventHandlerFrameToken(&identity)?;
        let document = domain.Document(token)?;
        let control = {
            let owner = document.borrow();
            let tree = owner.GetDocument();
            let mut index = tree.FindNodeById(id)?;
            if tree.Node(index).IsHTMLElement("option")
                || tree.Node(index).IsHTMLElement("optgroup")
            {
                loop {
                    index = tree.Node(index).Parent()?;
                    if tree.Node(index).IsHTMLElement("select") {
                        break;
                    }
                }
            }
            let node = tree.Node(index);
            if !(node.IsHTMLElement("input")
                || node.IsHTMLElement("textarea")
                || node.IsHTMLElement("select"))
            {
                return None;
            }
            if tree.InputType(index) == Some(dom::DOMInputType::kFile) {
                let files = tree
                    .InputFiles(index)
                    .expect("file input retains canonical FileList");
                return Some(if files.IsEmpty() {
                    tree.InputFileNoFileLabel(index)
                        .unwrap_or_else(|| tree.FileInputStatusText(index))
                        .to_owned()
                } else {
                    (0..files.length())
                        .map(|index| files.item(index).expect("admitted file").name().to_owned())
                        .collect::<Vec<_>>()
                        .join("\n")
                });
            }
            if (node.IsHTMLElement("textarea") && node.FindAttribute("novalidate").is_some())
                || tree
                    .FormOwner(index)
                    .is_some_and(|form| tree.Node(form).FindAttribute("novalidate").is_some())
            {
                return None;
            }
            tree.Node(index).Id()
        };
        let engine = domain
            .FocusInteraction(token)
            .expect("tooltip control Interaction");
        let validation = interaction::adapter::page_form_validation::ControlValidity(
            &document.Handle(),
            &engine.FormControlState(),
            control,
        )
        .unwrap_or_else(|error| {
            std::panic::panic_any(InteractionError::SourceInput(format!(
                "default tooltip: {error:?}"
            )))
        });
        Some(String::from_utf16_lossy(&validation.message))
    }
    fn SelectNativeCursor(
        &self,
        location: &HitTestLocation,
        result: &interaction::adapter::page_pointer_hit_test::PointerHitResult,
    ) {
        let (domain, page) = self.Owners();
        let frame =
            super::local_frame_event_handler::NativeEventHandlerFrame::new(&domain, self.token);
        let identity = result
            .InnerNode()
            .and_then(|node| frame.HitNodeFrameIdentity(node));
        let Some(token) = identity.and_then(|identity| domain.EventHandlerFrameToken(&identity))
        else {
            return;
        };
        let engine = domain
            .FocusInteraction(token)
            .expect("cursor target source Interaction");
        let document = domain.Document(token).expect("cursor target Document");
        let host_metrics = interaction::adapter::page_cursor::ResidentCursorHostMetrics {
            device_scale_factor: page.constraints.borrow().device_pixel_ratio as f32,
            system_cursor_size_dip: page
                .native_event_handler_host
                .system_cursor_size_dip
                .get()
                .expect("native widget installs immutable cursor metrics before input"),
        };
        let preferred_color_scheme = match page.preferred_color_scheme.get() {
            PreferredColorScheme::kDark => {
                interaction::core::input::event_handler_cursor::ColorScheme::kDark
            }
            PreferredColorScheme::kLight => {
                interaction::core::input::event_handler_cursor::ColorScheme::kLight
            }
        };
        let (selections, queries): (Rc<layoutng_assembly::editing_state::SelectionState>, Rc<dyn interaction::adapter::resident_editing_positions::ResidentEditingPositionQueries>) = if token == domain.root {
            (page.layout_editing.selections.clone(), Rc::new(super::resident_editing_positions::PageResidentEditingPositions(Rc::downgrade(&page))))
        } else {
            let frame = domain.Frame(token).expect("cursor child Frame");
            let runtime = frame.runtime.borrow().clone().expect("cursor child Document");
            (runtime.editing.selections.clone(), runtime.positions.get().expect("cursor child position provider").clone())
        };
        let mut failures = Vec::new();
        let cursor = interaction::adapter::page_cursor::SelectEventHandlerCursor(
            &engine,
            &document.Handle(),
            location,
            result,
            &selections,
            queries,
            host_metrics,
            preferred_color_scheme,
            Arc::new(paint_skia::svg_shape_path::SkiaNativePathFactory),
            &mut failures,
        );
        for failure in failures {
            eprintln!("rechrom-cursor-image-error\t{:?}", failure);
        }
        let cursor = cursor.unwrap_or_else(|error| {
            std::panic::panic_any(InteractionError::SourceInput(format!(
                "source cursor: {error:?}"
            )))
        });
        if let Some(cursor) = cursor {
            if *page.native_event_handler_host.cursor.borrow() != cursor {
                *page.native_event_handler_host.cursor.borrow_mut() = cursor.clone();
                page.client.borrow_mut().DidChangeCursor(cursor);
            }
        }
    }
}
impl PageEventHandlerChromeClient for NativeEventHandlerChrome {
    fn WillDispatchPointerDown(&self, target: &EventTargetRef) {
        let client = self
            .Owners()
            .1
            .native_event_handler_host
            .autofill_clients
            .borrow()
            .get(&self.token)
            .cloned();
        if let Some(client) = client {
            let node = target.NodeId().expect("pointerdown Node");
            client.WillDispatchPointerDown(self.token, node);
            self.Apply(EventHandlerChromeEffect::WillDispatchPointerDown {
                frame: self.token,
                node,
            });
        }
    }
    fn DidDispatchMouseDown(&self, target: &EventTargetRef) {
        let (_, page) = self.Owners();
        let autofill = page
            .native_event_handler_host
            .autofill_clients
            .borrow()
            .get(&self.token)
            .cloned();
        let replay = page
            .native_event_handler_host
            .record_replay_clients
            .borrow()
            .get(&self.token)
            .cloned();
        let node = target.NodeId().expect("mousedown Node");
        if let Some(client) = &autofill {
            client.DidDispatchMouseDown(self.token, node);
        }
        if let Some(client) = &replay {
            client.DidDispatchMouseDown(self.token, node);
        }
        if autofill.is_some() || replay.is_some() {
            self.Apply(EventHandlerChromeEffect::DidDispatchMouseDown {
                frame: self.token,
                node,
            });
        }
    }
    fn MouseDidMoveOverElement(
        &self,
        location: &HitTestLocation,
        result: &PageEventHandlerHitResult,
    ) {
        let (_, page) = self.Owners();
        let url = result
            .Native()
            .AbsoluteLinkURL()
            .map(|url| url.GetString().to_owned());
        if result.Native().GetScrollbar().is_none() && result.Native().InnerNode().is_some() {
            let networking = page
                .native_event_handler_host
                .prescient_networking
                .borrow()
                .get(&self.token)
                .cloned();
            if let Some(networking) = networking {
                if networking.IsDNSPrefetchEnabled(self.token) {
                    networking.PrefetchDNS(url.as_deref());
                }
            }
        }
        self.Apply(EventHandlerChromeEffect::ShowMouseOverUrl {
            frame: self.token,
            url,
        });
        let mut direction = foundation::TextDirection::kLtr;
        let mut title = result.Native().Title(&mut direction);
        if title.is_none() {
            title = self.DefaultToolTip(result);
            direction = foundation::TextDirection::kLtr;
        }
        let modifiers = CURRENT_INPUT.with(|current| {
            current.borrow().as_ref().map_or(0, |event| match event {
                ConvertedWebInputEvent::Mouse(event) => event.GetModifiers(),
                ConvertedWebInputEvent::Pointer(event) => event.GetModifiers(),
                ConvertedWebInputEvent::Touch(event) => event.GetModifiers(),
                ConvertedWebInputEvent::Gesture(event) => event.GetModifiers(),
                ConvertedWebInputEvent::Wheel(event) => event.GetModifiers(),
                ConvertedWebInputEvent::Keyboard(event) => event.GetModifiers(),
            })
        });
        self.Apply(EventHandlerChromeEffect::MouseDidMoveOverElement {
            frame: self.token,
            node: result
                .Native()
                .InnerNode()
                .and_then(|node| node.DOMNodeId()),
            title: title.clone(),
            point: (
                location.Point().left.ToFloat(),
                location.Point().top.ToFloat(),
            ),
            modifiers,
        });
        if result.Native().GetScrollbar().is_some() {
            self.ClearToolTip();
            return;
        }
        let node = result
            .Native()
            .InnerNodeOrImageMapImage()
            .map(|node| node.DOMIdentity());
        let cache = page.native_event_handler_host.tooltip.borrow();
        if cache.last_point == *location.Point() && cache.last_text == title {
            return;
        }
        let clear = cache.last_node != node
            && cache
                .last_text
                .as_ref()
                .is_some_and(|text| !text.is_empty())
            && cache.last_text == title;
        drop(cache);
        if clear {
            self.ClearToolTip();
        }
        let mut cache = page.native_event_handler_host.tooltip.borrow_mut();
        cache.last_point = *location.Point();
        cache.last_text = title.clone();
        cache.last_node = node;
        drop(cache);
        self.Apply(EventHandlerChromeEffect::UpdateToolTip {
            frame: self.token,
            text: title,
            direction,
        });
    }
    fn ClearToolTip(&self) {
        self.Apply(EventHandlerChromeEffect::ClearToolTip { frame: self.token });
    }
}
impl PageEventHandlerCursorClient for NativeEventHandlerChrome {
    fn SelectAndSetCursor(&self, location: &HitTestLocation, result: &PageEventHandlerHitResult) {
        self.SelectNativeCursor(location, result.Native());
    }
    fn CursorUpdateTimerFired(&self) {
        let (domain, page) = self.Owners();
        if page.autoscroll_cursor_overridden.get() {
            return;
        }
        let frame =
            super::local_frame_event_handler::NativeEventHandlerFrame::new(&domain, domain.root);
        if frame.View().is_none() || !page.layout_engine.borrow().HasCompletedLayoutForHitTest() {
            return;
        }
        let engine = domain
            .FocusInteraction(domain.root)
            .expect("root cursor source Interaction");
        let point = engine
            .SourceInputOwners()
            .mouse
            .LastKnownMousePositionInViewport();
        let point = page.web_view_zoom.borrow().ViewportToRootFrame(point);
        let location = HitTestLocation::FromPointF(point);
        let result = super::pointer_geometry::PagePointerGeometryProvider(Rc::downgrade(&page))
            .HitTest(location.clone(), interaction::hit_test::core::layout::hit_test_request::RequestType::kReadOnly as u32
                | interaction::hit_test::core::layout::hit_test_request::RequestType::kAllowChildFrameContent as u32)
            .unwrap_or_else(|error| std::panic::panic_any(InteractionError::SourceInput(format!("cursor timer: {error:?}"))));
        self.SelectNativeCursor(&location, &result.result);
    }
}
