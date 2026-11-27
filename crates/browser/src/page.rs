#![allow(non_snake_case)]
//! Persistent Page lifecycle. All frames and measurements consume the original
//! parser arena. Module loading and the remaining DOM host continuations are
//! tracked separately; this service is not evidence of complete browser parity.
//! Page connects DOM mutation notifications and the style/layout/paint lifecycle;
//! each engine owns its state and invalidation, like LocalFrameView lifecycle wiring.
pub(crate) mod canvas_2d;
use paint::{PaintLifecycle as _, PaintPropertyUpdates as _, PaintResourceUpdates as _};
use layout::fragment::ResidentBoxGeometry as _;
use crate::interaction_services::PageInteraction;
mod main_compositor_effect;
pub use main_compositor_effect::{CompositorMainThreadEffect, PresentationUnavailableReason};
use viz_common::ResourceId;
use compositor_tree::geometry::{
    PointF as CompositorPointF, Rect as CompositorRect, RectF as CompositorRectF,
    Size as CompositorSize, Vector2dF,
};
use compositor_values::begin_frame::{BeginFrameArgs, BeginFrameArgsType, BeginFrameId};
use compositor_values::color4f::Color4f as CompositorColor4f;
use compositor_values::input_to_impl::{InputToImpl, ScrollPhase};
use scroll_animation::{TimeDelta as CompositorTimeDelta, TimeTicks as CompositorTimeTicks};
use document::{
    DocumentEffects, DocumentEngine, LayoutMutation as DocumentLayoutMutation, ResourceRequest,
};
use document_loader::{DocumentLoadBudget, ResourceFetcher, ResourceFetcherClient};
use dom::{
    dom_mutation::DOMMutationType, persistent_document::DOMNodeType, Document,
    UserInteractionState, DOM,
};
use foundation::content_security_policy::CSPRequestPolicy;
use image_decoder::image_decoder::ImageDecoder;
use image_resource::DocumentImageDecoder;
pub use interaction::cursor::Cursor;
use interaction::event::{EventListenerInvocation, EventType, MakeSyntheticEvent};
pub use interaction::frame_aligned_input_queue::DispatchedFrameInput;
use javascript::javascript_runtime::{
    JavaScriptException, JavaScriptHostRuntime, JavaScriptRealm, JavaScriptResult,
    JavaScriptRuntime,
};
use javascript::script_engine::ScriptEngine;
use layout::fragment::LayoutResultHandle;
use layout::lifecycle::LayoutUpdateKind;
use layout::paint::{LayoutPaintOutput, PaintFragmentCursor};
use layout::{editing::DocumentSelection, fragment::FragmentKind, paint::PaintResources, input::{ConstraintSpace, Offset, Color, IntSize}};
pub use open::OpenStatus;
use open::{script_scheduler::ScriptLoadClient, OpenEngine};
use page_mutation::{PageMutation, ResourceMutation};
pub use paint::paint_engine::{CaretGeometry, PaintArtifact, PaintRect as CaretRect};
use raster::RasterEngine;
use renderer::RenderUpdate;
use renderer_skia::{PixelFormat, RendererSkiaBackend};
use resource::ResourceEngine;
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    io,
    rc::{Rc, Weak},
    sync::Arc,
};
pub use style::{media_queries::forced_colors::ForcedColors, PreferredColorScheme};
use url_loader::{URLLoader, URLRequest, URLWakeCallback};
use webapi::{
    dom_bindings::{
        DOMBindingsHost, DOMJavaScriptBindings, ViewportScrollBehavior, ViewportScrollState,
    },
    web_api_engine::{WebApiEffect, WebApiEngine, WebApiMutation},
    window_bindings::WindowJavaScriptBindings,
};

fn FrameFetchSecurity(document: &Document) -> Arc<document_loader::RawFetchSecurityContext> {
    let security = document.SecurityContext();
    let client_security = document_loader::DeriveRawLocalNetworkClientSecurityState(
        security.AddressSpace(),
        security.IsSecureContext(),
        false,
        false,
        false,
    );
    let frame = document_loader::RawFetchFrameSecurity {
        origin: security.Origin().ProtectedOriginForRequest(),
        settings: document_loader::RawFetchFrameSettings {
            strict_mixed_content_checking: false,
            strictly_block_blockable_mixed_content: false,
            allow_running_insecure_content: false,
        },
        block_all_mixed_content: document
            .ContentSecurityPolicy()
            .PrepareRequestURL(document.BaseURL().AsStr())
            .block_all_mixed_content,
    };
    Arc::new(document_loader::RawFetchSecurityContext {
        current_frame: Some(frame.clone()),
        top_frame: Some(frame),
        current_frame_is_top: true,
        content_settings_client: None,
        mixed_content_autoupgrade: true,
        local_network_access_checks: false,
        target_address_space: security.AddressSpace(),
        local_network_access_client_security_state: Some(client_security),
        block_local_request: false,
        local_network_access_policy: None,
        keepalive_redirect_frame: None,
        address_space_overrides: None,
    })
}
struct PageCompositorExecutor {
    executor: Arc<dyn executor::Executor>,
    origin: std::time::Instant,
    sequence: std::thread::ThreadId,
}
impl PageCompositorExecutor {
    fn new(executor: Arc<dyn executor::Executor>) -> Self {
        Self {
            executor,
            origin: std::time::Instant::now(),
            sequence: std::thread::current().id(),
        }
    }

    /// base::TimeTicks::Now() in this Page's compositor clock domain.
    fn Now(&self) -> CompositorTimeTicks {
        self.TimeTicks(std::time::Instant::now())
    }

    /// Posts `task` to the sequence this executor runs.
    fn Execute(&self, task: Box<dyn FnOnce() + Send>) {
        let _ = self.executor.execute(task);
    }

    /// Convert an Instant into cc's one absolute TimeTicks domain without
    /// advancing that domain's clock. In particular, converting a future
    /// BeginFrame deadline must not make Now() equal that deadline.
    fn TimeTicks(&self, instant: std::time::Instant) -> CompositorTimeTicks {
        // Leave an hour before the origin so a physical frame sampled just
        // before Page construction remains ordered and all normal ticks stay
        // non-null, matching base::TimeTicks' absolute conversion semantics.
        const ORIGIN_OFFSET_US: i64 = 60 * 60 * 1_000_000;
        let value = if instant >= self.origin {
            let delta = instant
                .duration_since(self.origin)
                .as_micros()
                .min(i64::MAX as u128) as i64;
            ORIGIN_OFFSET_US.saturating_add(delta)
        } else {
            let delta = self
                .origin
                .duration_since(instant)
                .as_micros()
                .min(i64::MAX as u128) as i64;
            ORIGIN_OFFSET_US.saturating_sub(delta)
        };
        CompositorTimeTicks::FromMicroseconds(value.max(1))
    }
}
struct PageInteractionEventClock(Arc<PageCompositorExecutor>);
impl interaction::adapter::host_event_clock::HostInputClock for PageCompositorExecutor {
    fn TimeForInstant(
        &self,
        instant: std::time::Instant,
    ) -> interaction::input::base::time::time::TimeTicks {
        interaction::input::base::time::time::TimeTicks::FromInternalValue(
            self.TimeTicks(instant).since_origin().InMicroseconds(),
        )
    }
}
impl interaction::dispatch::core::dom::events::event::EventClock for PageInteractionEventClock {
    fn Now(&self) -> interaction::input::base::time::time::TimeTicks {
        let micros = self
            .0
            .TimeTicks(std::time::Instant::now())
            .since_origin()
            .InMicroseconds();
        interaction::input::base::time::time::TimeTicks::FromInternalValue(micros)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct PageCompositorClockMapping {
    origin: std::time::Instant,
}
impl PageCompositorClockMapping {
    /// The input-event TimeTicks of `instant` in this compositor clock.
    pub fn InputTime(
        self,
        instant: std::time::Instant,
    ) -> interaction::input::base::time::time::TimeTicks {
        interaction::input::base::time::time::TimeTicks::FromInternalValue(
            self.TimeTicks(instant).since_origin().InMicroseconds(),
        )
    }
    pub fn Instant(self, ticks: CompositorTimeTicks) -> Option<std::time::Instant> {
        const ORIGIN_OFFSET_US: i64 = 60 * 60 * 1_000_000;
        let delta = ticks
            .since_origin()
            .InMicroseconds()
            .checked_sub(ORIGIN_OFFSET_US)?;
        let duration = std::time::Duration::from_micros(delta.unsigned_abs());
        if delta >= 0 {
            self.origin.checked_add(duration)
        } else {
            self.origin.checked_sub(duration)
        }
    }

    pub fn TimeTicks(self, instant: std::time::Instant) -> CompositorTimeTicks {
        const ORIGIN_OFFSET_US: i64 = 60 * 60 * 1_000_000;
        let value = if instant >= self.origin {
            ORIGIN_OFFSET_US.saturating_add(
                instant
                    .duration_since(self.origin)
                    .as_micros()
                    .min(i64::MAX as u128) as i64,
            )
        } else {
            ORIGIN_OFFSET_US.saturating_sub(
                self.origin
                    .duration_since(instant)
                    .as_micros()
                    .min(i64::MAX as u128) as i64,
            )
        };
        CompositorTimeTicks::FromMicroseconds(value.max(1))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PagePresentationUnavailableReason {
    BackendHasNoTimestamp,
    OffscreenTarget,
    DiscardedBeforePresentation,
}
#[derive(Clone, Copy, Debug)]
pub enum PagePresentationOutcome {
    Presented {
        timestamp: std::time::Instant,
        interval: std::time::Duration,
        flags: u32,
    },
    Unavailable(PagePresentationUnavailableReason),
}
#[derive(Clone, Copy, Debug)]
pub struct PageFramePresentationFeedback {
    pub surface_id: u64,
    pub viz_submission: u64,
    pub frame_token: u32,
    pub frame_id: BeginFrameId,
    pub outcome: PagePresentationOutcome,
}
struct VideoSurfaceEntry {
    surface_id: viz::adapter::SurfaceId,
    frame_size: (u32, u32),
    layer: Arc<dyn paint::paint_engine::ForeignLayer>,
}
/// A task the document thread posts to the compositor thread (Blink's
/// WidgetInputHandlerManager::InputThreadTaskRunner(), the compositor
/// thread's default task runner in threaded mode,
/// third_party/blink/renderer/platform/widget/input/
/// widget_input_handler_manager.cc:1250-1260). Each is posted at the point
/// Blink posts it and runs in posting order with the ProxyMain -> ProxyImpl
/// tasks, which Blink posts to the same runner (WidgetBase::
/// InitializeCompositing passes `DefaultTaskRunner()` as the impl task
/// runner, platform/widget/widget_base.cc:239-245). The one exception is
/// the main-thread hit-test reply, which Blink posts to the compositor
/// thread's input-blocking runner (`CompositorInputBlockingTaskRunner`).
pub enum CompositorThreadTask {
    /// The document input target was replaced (the input router retires
    /// its target-bound gesture state).
    DocumentDetached { generation: u64 },
    /// A document-thread request for the impl input handler.
    ImplInput(InputToImpl),
    /// A browser-dispatched scroll or pinch gesture for the impl input
    /// handler (WidgetInputHandlerManager::DispatchScrollGestureToCompositor,
    /// widget_input_handler_manager.cc:626-638); the input router converts
    /// it to its InputHandlerProxy call.
    GestureScroll(interaction::input_event::WebGestureEvent),
    /// A Page-generated gesture for the input router.
    Gesture(interaction::input_event::WebGestureEvent),
    /// DidHandleInputEventSentToMain of a queued wheel
    /// (widget_input_handler_manager.cc:1171-1230).
    WheelEventAck {
        document_generation: u64,
        result: interaction::input::public::mojom::input::input_event_result::InputEventResultState,
    },
    /// DidHandleInputEventSentToMain of a browser-dispatched gesture.
    GestureEventAck { dispatch_id: u64, consumed: bool },
    /// DidHandleInputEventSentToMain of a touch start or first touch move;
    /// the compositor thread runs the browser's DispatchEventCallback
    /// (CallCallback, widget_input_handler_manager.cc:1209-1218).
    TouchEventAck {
        unique_touch_event_id: u32,
        consumed: bool,
    },
}
/// The compositor thread's task runner as the Page holds it. The embedding
/// injects the posting form over the same ordered queue its ProxyMain tasks
/// use; without one (single-thread compositing) it is the in-process form,
/// which the Page runs as its own main-thread tasks, as Blink's
/// InputThreadTaskRunner() falls back to the main-thread task runner
/// (widget_input_handler_manager.cc:1257-1258).
pub type CompositorTaskRunner = layerize::commit::proxy::ProxyTaskQueue<CompositorThreadTask>;
/// The compositor thread's input-blocking task runner: Blink's
/// CompositorThreadScheduler::InputTaskRunner(), whose queue runs at
/// TaskPriority::kHighestPriority, so the compositor thread's sequence
/// manager selects its tasks before those of the normal-priority default
/// queue (scheduler/worker/non_main_thread_scheduler_helper.cc:27-33,51-52;
/// compositor_thread_scheduler_impl.cc:81-84;
/// widget_input_handler_manager.cc:303-306,1250-1258). Only
/// FindScrollTargetOnMainThread posts to it (cc:504-520).
pub type CompositorInputBlockingTaskRunner = CompositorTaskRunner;
fn PostMainThreadEventAck(
    runner: &CompositorTaskRunner,
    ack: interaction::frame_aligned_input_queue::MainThreadEventAck,
) {
    use interaction::frame_aligned_input_queue::MainThreadEventAck;
    use interaction::input::public::mojom::input::input_event_result::InputEventResultState;
    // A stopped compositor thread drops the posted task, as a deleted
    // task runner does.
    match ack {
        MainThreadEventAck::Wheel(disposition) => {
            let _ = runner.PostTask(CompositorThreadTask::WheelEventAck {
                document_generation: disposition.document_generation,
                result: disposition.result,
            });
            for request in disposition.requests {
                let _ = runner.PostTask(CompositorThreadTask::ImplInput(
                    InputToImpl::WheelDOMAcknowledgement {
                        request,
                        default_prevented: disposition.result == InputEventResultState::kConsumed,
                    },
                ));
            }
        }
        MainThreadEventAck::BrowserGesture(disposition) => {
            let _ = runner.PostTask(CompositorThreadTask::GestureEventAck {
                dispatch_id: disposition.dispatch_id,
                consumed: disposition.consumed,
            });
        }
        MainThreadEventAck::Touch(disposition) => {
            let _ = runner.PostTask(CompositorThreadTask::TouchEventAck {
                unique_touch_event_id: disposition.unique_touch_event_id,
                consumed: disposition.consumed,
            });
        }
    }
}
/// A delegated video child surface submission. The Page producer allocates
/// the stable public surface identity referenced by its SurfaceDrawQuad; the
/// window-owned Viz service consumes this frame before the parent frame.
pub struct PageChildCompositorFrame {
    pub surface_id: viz::adapter::SurfaceId,
    pub frame: viz_common::CompositorFrame,
}
pub type PageReturnedResource = viz_common::returned_resource::RasterReturnedResource;
/// The main LayerTreeHost's main-thread task runner (copy-output results,
/// view-transition callbacks) over this Page's document executor.
struct PageMainTaskRunner(Arc<PageCompositorExecutor>);
impl layerize::commit::main_callbacks::ResultTaskRunner for PageMainTaskRunner {
    fn PostTask(&self, task: Box<dyn FnOnce() + Send>) {
        self.0.Execute(task)
    }
}
// cpp: browser/browser.h:47-51
pub struct PageFrame {
    pub sequence: u64,
    /// Last compositor scroll delta reflected by this committed snapshot.
    pub compositor_scroll_sequence: u64,
    pub fragments: LayoutResultHandle,
    // The PaintArtifact is not a frame field: PaintEngine's persistent
    // PaintController data is its only document-side owner, as Chromium's
    // PaintControllerPersistentData is (paint_controller.h:85-99). Page reads
    // it through `Page::CurrentPaintArtifact` (LocalFrameView::GetPaintArtifact,
    // local_frame_view.cc:4399-4406).
    /// Rechrom-specific certification for its immutable document-image record
    /// catalog. Blink's SVGImage instead sizes its isolated document and asks
    /// the view for a PaintRecord at draw time.
    ///
    /// Certified once when a newly committed artifact enters the frame. Retained
    /// scroll-property updates do not change display items or their catalog.
    pub document_image_records_complete: bool,
}

/// External drop navigation has Chromium's separate foreground/background tab
/// policy; its request carries the opaque external initiator origin.
#[derive(Clone)]
pub struct DirectoryEnumerationRequest {
    pub request_id: u64,
    pub document_identity: u128,
    pub target: u64,
    pub directory: Arc<fileapi::File>,
}
pub struct DirectoryEnumerationResult {
    pub request: DirectoryEnumerationRequest,
    pub files: Result<Vec<Arc<fileapi::File>>, fileapi::NativeFileError>,
}
#[derive(Clone, Debug)]
pub struct DropNavigationRequest {
    pub request: URLRequest,
    pub foreground: bool,
}
pub use interaction::default::core::loader::frame_load_request::FrameNavigationTarget as NavigationTarget;
/// Local document navigation request; tab/window policy belongs to the host.
#[derive(Clone, Debug)]
pub struct NavigationRequest {
    pub request: URLRequest,
    pub target: NavigationTarget,
    /// Replace the current session-history entry (Location.replace).
    pub replace_history: bool,
    /// Immutable loader/ChromeClient facts for the host's target and download
    /// routing. No Frame, Window, request owner or script value crosses here.
    pub host: Option<NavigationHostMetadata>,
}
#[derive(Clone, Debug)]
pub struct NavigationHostMetadata {
    pub policy: interaction::default::core::loader::navigation_policy::NavigationPolicy,
    pub auxiliary_target: Option<u64>,
    pub noopener: bool,
    pub explicit_opener: bool,
    pub never_send_referrer: bool,
    pub href_translate: Option<String>,
    pub suggested_filename: Option<String>,
    pub triggering_event_info:
        interaction::default::core::loader::frame_load_request::TriggeringEventInfo,
    pub input_start_time: interaction::input::base::time::time::TimeTicks,
    pub client_navigation_reason:
        interaction::default::core::loader::frame_load_request::ClientNavigationReason,
    pub initiator:
        Option<interaction::default::core::loader::frame_load_request::NavigationInitiator>,
    pub blob_url_token:
        Option<Arc<interaction::dispatch::core::fileapi::blob_url_registry::BlobURLToken>>,
}
#[derive(Clone, Debug)]
pub struct AuxiliaryNavigationTargetRequest {
    pub request_id: u64,
    pub request: NavigationRequest,
}
mod chrome_state;
pub use chrome_state::{PageChromeMouseOver, PageChromeState};

// cpp: browser/browser.h:53-67
pub trait PageClient {
    /// Completion of the source trusted primary click, after cancellation and
    /// default handlers. This is a semantic notification, never raw input.
    fn DidDispatchClick(&mut self, _document: u128, _target_node_id: u64) {}
    /// EventHandler's synchronous ChromeClient notifications. Every embedding
    /// must consume the typed effect or retain it in a per-document state.
    fn DidEventHandlerChromeEffect(&mut self, effect: EventHandlerChromeEffect);

    fn CurrentKeyboardModifiers(
        &self,
    ) -> Result<
        interaction::core::input::keyboard_event_manager::NativeKeyboardModifiers,
        &'static str,
    > {
        Err("native keyboard modifiers")
    }
    fn ClipboardAccessPolicy(
        &self,
    ) -> interaction::default::core::editing::commands::clipboard_commands::ClipboardAccessPolicy
    {
        Default::default()
    }
    /// Host FileChooser enumeration. Completion returns to this document's
    /// ordinary task queue; the request retains the admitted File authority.
    fn EnumerateDroppedDirectory(&self, _request: DirectoryEnumerationRequest) -> bool {
        false
    }

    fn AcceptsLoadDrops(&self) -> bool {
        false
    }
    /// This capability is advertised only by a host that routes the complete
    /// foreground/background navigation request after the Page task returns.
    fn DidRequestDropNavigation(&mut self, _request: &DropNavigationRequest) {}

    fn CaptureMouseEventsToWidget(&mut self, _capture: bool) {}
    /// Native source-drag capability. The host receives immutable data only;
    /// completion returns to this Page's document task queue.
    fn StartSystemDrag(
        &self,
        _request: interaction::default::core::page::drag_controller::SystemDragRequest,
    ) -> interaction::default::core::page::drag_controller::SystemDragSupport {
        interaction::default::core::page::drag_controller::SystemDragSupport::Unsupported
    }
    /// The embedding reads the actual platform primary selection during the
    /// current editing transaction. A standalone Page has no clipboard host.
    fn ReadPrimarySelection(
        &self,
    ) -> Result<
        interaction::default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
        interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError,
    > {
        Err(interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError::Unavailable)
    }
    fn ReadClipboard(
        &self,
    ) -> Result<
        interaction::default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
        interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError,
    > {
        Err(interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError::Unavailable)
    }
    fn WriteClipboard(
        &self,
        _snapshot: &interaction::default::core::editing::commands::clipboard_commands::PrimaryClipboardSnapshot,
    ) -> Result<
        (),
        interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError,
    > {
        Err(interaction::default::core::editing::commands::clipboard_commands::ClipboardHostError::Unavailable)
    }
    /// LocalFrameClient/ChromeClient input-host notifications. Standalone
    /// Pages have no remote widget; the application implementation routes the
    /// value with its original Document generation.
    fn DidAutoscroll(
        &mut self,
        _effect: interaction::adapter::page_mouse_autoscroll::AutoscrollHostEffect,
    ) {
    }
    /// The embedding host schedules navigation after the active Page call returns.
    fn DidRequestNavigation(&mut self, _request: &NavigationRequest) {}
    /// Create/admit the actual target before returning acceptance through
    /// Page::CompleteAuxiliaryNavigationTarget. Navigation follows separately
    /// with the accepted request_id in NavigationHostMetadata::auxiliary_target.
    fn DidRequestAuxiliaryNavigationTarget(&mut self, _request: &AuxiliaryNavigationTargetRequest) {
    }
    /// RendererContentSettingsClient popup content-setting exception. Ordinary
    /// user activation remains a separate source LocalFrame navigation fact.
    fn AllowsPopupNavigation(&self) -> bool {
        false
    }
    /// Session traversal is browser policy; the originating Page never reads
    /// another document's retained HistoryItem snapshots.
    fn DidRequestHistoryNavigation(&mut self, _delta: i32) {}

    fn DidCommit(&mut self, _url: &str) {}
    /// Immutable session-history state leaves the document sequence only after
    /// its loader has installed the committed HistoryItem owner.
    fn DidCommitHistory(
        &mut self,
        _item: Arc<navigation_history::HistoryItemSnapshot>,
        _load_type: navigation_history::FrameLoadType,
    ) {
    }
    /// A subframe commit changes one FrameNavigationEntry within the current
    /// joint entry. The browser decides append, replacement and traversal.
    fn DidCommitChildHistory(
        &mut self,
        _item: Arc<navigation_history::HistoryItemSnapshot>,
        _load_type: navigation_history::FrameLoadType,
    ) {
    }
    /// Refresh an existing entry without changing the committed history index
    /// or acknowledging a pending navigation.
    fn DidUpdateHistoryState(&mut self, _item: Arc<navigation_history::HistoryItemSnapshot>) {}
    /// Host notification independent of frame painting; map this to the window cursor.
    fn DidChangeCursor(&mut self, _cursor: Cursor) {}
    /// LocalFrameClient::FocusedElementChanged projection. DOM remains the
    /// focused-element owner; the host receives identities only.
    fn DidChangeFocusedElement(
        &mut self,
        _old_node_id: Option<u64>,
        _new_node_id: Option<u64>,
        _focus_type: interaction::default::public::mojom::input::focus_type::FocusType,
    ) {
    }
    /// ValidationMessageClient's browser-owned overlay state. Coordinates are
    /// CSS pixels in this Page's content viewport, independent of author DOM.
    fn DidChangeValidationMessage(&mut self, _update: ValidationMessageUpdate) {}
    /// AXObjectCache::HandleValidationMessageVisibilityChanged notification.
    /// Embeddings with an accessibility cache announce the text on show.
    fn DidChangeValidationMessageVisibility(&mut self, _event: ValidationMessageVisibility) {}
    fn SelectAccessibilityClient(
        &self,
    ) -> Option<Rc<dyn interaction::adapter::page_select_control::SelectAccessibilityClient>> {
        None
    }
    fn NativeSelectPopupClient(
        &self,
    ) -> Option<Rc<dyn interaction::adapter::page_select_control::NativeSelectPopupClient>> {
        None
    }
    fn FormToolClient(
        &self,
    ) -> Option<Rc<dyn interaction::adapter::page_select_control::FormToolClient>> {
        None
    }
    fn DidResetFormElement(&mut self, _document: u128, _form: u64) {}
    /// ChromeClient::DidChangeSelectionInSelectControl; document identity is
    /// preserved for child frames instead of projecting their node ids upward.
    fn DidChangeSelectionInSelectControl(&mut self, _document: u128, _select: u64) {}
    /// WebLocalizedString::kSelectMenuListText, default en-US catalog entry.
    fn MultipleOptionsSelectedText(&self, count: usize) -> Rc<[u16]> {
        format!("{count} selected")
            .encode_utf16()
            .collect::<Vec<_>>()
            .into()
    }
    /// `artifact` is borrowed from PaintEngine's persistent PaintController
    /// data for the duration of the call; the client does not retain it.
    fn DidPresentFrame(
        &mut self,
        _frame: &PageFrame,
        _artifact: &paint::paint_engine::PaintArtifact,
    ) {
    }
    fn DidFinishLoad(&mut self) {}
    fn DidFail(&mut self, _message: &str) {}
    fn DidFailResource(&mut self, _url: &str, _message: &str) {}
    fn DidReportScriptError(&mut self, _error: &JavaScriptException) {}
}
struct ScriptClient(Rc<RefCell<dyn PageClient>>, Weak<PageState>);
impl ScriptLoadClient for ScriptClient {
    fn DidPumpParser(&mut self) {
        if let Some(state) = self.1.upgrade() {
            state.InvalidateLayoutOutput();
            state.InvalidateStyle();
            // The parser has returned the same Document arena and released
            // its borrow. Complete allocation/attribute lifecycle effects
            // before the scheduler admits queued parser or async scripts.
            dom::attribute_lifecycle::DrainAttributeLifecycle(&state.document.Handle());
            state.DrainPopupLifecycleWithoutScript();
        }
    }
    fn DidPumpParserWithRuntime(
        &mut self,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
    ) {
        self.DidPumpParser();
        if let Some(state) = self.1.upgrade() {
            runtime
                .WithHostRuntime(realm, &mut |runtime| {
                    state.DrainPopupLifecycleWithRuntime(runtime)
                })
                .unwrap_or_else(|error| std::panic::panic_any(error));
        }
    }
    fn DidApplyStyleSheet(&mut self, _: &str) {
        if let Some(state) = self.1.upgrade() {
            // A stylesheet first invalidates computed style. Whether it also
            // invalidates geometry is decided by the computed-style diff in
            // ResolveStyles, matching Blink's style/layout lifecycle split.
            state.InvalidateStylePreservingGeometry();
        }
    }
    fn DidReportScriptError(&mut self, e: &JavaScriptException) {
        self.0.borrow_mut().DidReportScriptError(e);
    }
    fn DidFailResource(&mut self, url: &str, message: &str) {
        self.0.borrow_mut().DidFailResource(url, message);
    }
}

#[cfg(test)]
mod layout_output_tests;
struct PendingImageEvent {
    id: u64,
    ticket: dom::persistent_document::HTMLImageLoaderEventTicket,
    loaded: bool,
    active: Rc<Cell<bool>>,
}
struct PendingStyleSheetEvent {
    id: u64,
    signature: String,
    loaded: bool,
    active: Rc<Cell<bool>>,
}
#[derive(Clone, Copy)]
struct PendingTextOverflowLayoutInvalidation {
    document_identity: u128,
    node_id: u64,
}

#[derive(Clone, Copy, Default)]
struct ResourceReferenceDirty(u8);

/// Document-wide resource reference walks a mutation can require. Web fonts
/// have no category: FontFaceSetDocument/CSSFontSelector demand is admitted by
/// every resource poll pass (ResourceFetcher::PollPendingFontsWithBudget runs
/// StartPendingFonts), so no mutation needs a separate font rediscovery.
impl ResourceReferenceDirty {
    const STYLE_SHEETS: u8 = 1 << 0;
    const IMAGES: u8 = 1 << 1;
    const ALL: Self = Self(Self::STYLE_SHEETS | Self::IMAGES);
    const VIEWPORT: Self = Self(Self::IMAGES);
    /// Element::InvalidateStyleAttribute (core/dom/element.cc:13195-13203)
    /// and animation style samples only SetNeedsStyleRecalc on their element.
    /// They never connect a link/style owner or change an image request;
    /// images the new style references are admitted through the Document's
    /// style-image reference revision after that element's recalc.
    const STYLE_ONLY: Self = Self(0);

    fn Contains(self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    fn IsEmpty(self) -> bool {
        self.0 == 0
    }

    fn Insert(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

/// CSSFontSelector inputs belong to one Document. ConstraintSpace also holds
/// the surviving Frame viewport, so installation publishes only these small
/// font aliases; preparing a replacement cannot mutate the old selector.
pub(super) struct PageDocumentFontInputs {
    engine: font_engine::FontEngineHandle,
    queries: Option<font_engine::FontQueryCollector>,
    faces: font_engine::FontFaceCatalog,
    pending: Cell<bool>,
}
impl PageDocumentFontInputs {
    pub(super) fn Initial(constraints: &ConstraintSpace) -> Self {
        Self {
            engine: constraints
                .font_engine
                .clone()
                .expect("initial Document font owner"),
            queries: constraints.font_queries.clone(),
            faces: constraints.fonts.clone(),
            pending: Cell::new(false),
        }
    }
}

struct PagePaintLifecycle(Rc<dom::document_lifecycle::DocumentLifecycle>);
impl paint::paint_engine::PaintLifecycleObserver for PagePaintLifecycle {
    fn SetIntersectionObservationState(
        &self,
        _: foundation::intersection_observation_state::IntersectionObservationState,
    ) {
    }
    fn AdvanceTo(&self, phase: paint::paint_engine::PaintLifecyclePhase) {
        use dom::document_lifecycle::LifecycleState as State;
        use paint::paint_engine::PaintLifecyclePhase as Phase;
        self.0.AdvanceTo(match phase {
            Phase::InPrePaint => State::InPrePaint,
            Phase::PrePaintClean => State::PrePaintClean,
            Phase::InPaint => State::InPaint,
            Phase::PaintClean => State::PaintClean,
        });
    }
}

/// Stable local-root embedding owners. A Document replacement keeps the Frame,
/// widget, rendering service and host policy; document engines and bindings
/// remain on the original PageState for as long as their Window is retained.
struct PageDomainServices {
    web_view_zoom: RefCell<interaction::default::core::exported::web_view_zoom::WebViewZoom>,
    native_event_handler_host: native_event_handler_host::NativeEventHandlerHostState,
    native_file_reader: RefCell<Option<Arc<dyn fileapi::NativeFileReader>>>,
    kill_ring: RefCell<Option<Rc<dyn interaction::default::core::editing::kill_ring::KillRingPlatform>>>,
    document_input_queue: std::cell::OnceCell<interaction::frame_aligned_input_queue::FrameAlignedInputQueue>,
    context_menu_widget_geometry: Cell<Option<interaction::adapter::page_nonlocated_context_menu_coordinates::NativeContextMenuWidgetGeometry>>,
    frame_owner_loading_client: std::cell::OnceCell<Rc<dyn dom::frame_owner_loading::FrameOwnerLoadingClient>>,
    local_frames: std::cell::OnceCell<Rc<local_frame::LocalFrameDomain>>,
    // The Page's compositor clock and task runners (cc's injected
    // executors), and the RasterEngine of the document sequence (drag
    // images, VideoFrameSubmitter resources; the internal presenter's impl
    // half is given the same engine).
    compositor_executor: Arc<PageCompositorExecutor>,
    compositor_raster_executor: Arc<PageCompositorExecutor>,
    raster_engine: RasterEngine,
    // The embedding presents this Page's CompositorFrames (the impl half
    // runs on its compositor thread); otherwise the synchronous internal
    // presenter owns the impl half (`synchronous_presenter`).
    external_presentation: Cell<bool>,
    // The cc main half on this thread (S1b-7, `main_compositor`): ProxyMain
    // (stopped before the LayerTreeHost shuts down), its Send + Sync request
    // endpoint, the LayerizeEngine and its PropertyTreeFeatures.
    proxy_main: std::cell::OnceCell<RefCell<main_compositor::MainProxy>>,
    proxy_main_endpoint: std::cell::OnceCell<
        Arc<layerize::runtime::proxy_main::ProxyMainEndpoint<layerize::commit::MainFamily>>,
    >,
    layerize: std::cell::OnceCell<RefCell<main_compositor::MainLayerize>>,
    main_property_tree_features:
        std::cell::OnceCell<layerize::commit::property_tree_services::PropertyTreeFeatureValues>,
    main_compositor_shut_down: Cell<bool>,
    // A visibility change made inside ProxyMain::BeginMainFrame (the Page
    // lifecycle), applied when it returns.
    pending_main_compositor_visible: Cell<Option<bool>>,
    // A main-half call made outside the Page's own tasks (composited
    // scrollbar layer updates from scrollbar animation callbacks) whose
    // in-process Proxy tasks and effects the internal presenter runs at the
    // next lifecycle.
    main_compositor_mutated_outside_lifecycle: Cell<bool>,
    // ProxyMain::BeginMainFrame is running (PaintArtifactCompositor::Update
    // is only admitted inside it).
    in_main_compositor_frame: Cell<bool>,
    main_compositor_effects: RefCell<Vec<CompositorMainThreadEffect>>,
    // The compositor thread's task runner (input-queue posts).
    compositor_task_runner: RefCell<CompositorTaskRunner>,
    // Its input-blocking runner. Absent (single-thread compositing), the
    // hit-test reply runs in place where the main-thread runner would run it
    // (InputThreadTaskRunner(kInputBlocking) falls back to the main-thread
    // runner, widget_input_handler_manager.cc:1250-1258).
    compositor_input_blocking_task_runner: RefCell<Option<CompositorInputBlockingTaskRunner>>,
    // PaintArtifactCompositor update sequence (paint_artifact_compositor.cc
    // g_s_property_tree_sequence_number role).
    compositor_pac_sequence: Cell<i32>,
    // Blink's ParentLocalSurfaceIdAllocator: the parent sequence advances
    // whenever the surface parameters change
    // (components/viz/common/surfaces/parent_local_surface_id_allocator.cc:
    // 74-96; cc/trees/unbounded_frame_sink_handler.cc:56-77).
    compositor_local_surface_id: Cell<Option<compositor_values::surface_id::LocalSurfaceId>>,
    compositor_surface_parameters: Cell<Option<(CompositorSize, u32)>>,
    // LayerTreeHost page scale limits (WebViewImpl -> LayerTreeHost).
    compositor_page_scale: Cell<(f32, f32, f32)>,
    // The last artifact pushed to PAC and its viewport
    // (LocalFrameView::PushPaintArtifactToCompositor), so an unchanged Page
    // frame does not run PaintArtifactCompositor::Update again.
    pushed_paint_artifact:
        RefCell<Option<(std::sync::Weak<paint::PaintArtifact>, u64, (u32, u32, u64))>>,
    // The layout zoom of the pushed artifact (InputRouter's widget scale).
    compositor_input_scale: Cell<f64>,
    // VideoFrameSubmitter state: the video SurfaceLayers, their frame
    // tokens and the child frames queued for the external runtime.
    video_surfaces: RefCell<HashMap<u64, VideoSurfaceEntry>>,
    video_frame_token: Cell<u32>,
    outbound_child_frames: RefCell<std::collections::VecDeque<PageChildCompositorFrame>>,
    synchronous_presenter: RefCell<Option<synchronous_presenter::SynchronousPresenter>>,
    constraints: Rc<RefCell<ConstraintSpace>>,
    platform_fonts: font_engine::FontFaceCatalog,
    visual_viewport_identity: u64,
    client: Rc<RefCell<dyn PageClient>>,
    should_paint_base_background_color: Rc<Cell<bool>>,
    base_background_color: Rc<Cell<Color>>,
    should_clear_document_background: Rc<Cell<bool>>,
    event_timing_widget: Rc<event_timing::PageEventTimingWidget>,
    external_compositor_clock: RefCell<Option<PageCompositorClockMapping>>,
    performance_visible: Cell<bool>,
    performance_visibility_changed_at: Cell<Option<std::time::Instant>>,
    has_begin_frame_source: Rc<Cell<bool>>,
    begin_frame_source: RefCell<Option<Arc<dyn foundation::begin_frame::BeginFrameSource>>>,
    preferred_color_scheme: Rc<Cell<PreferredColorScheme>>,
    forced_colors: Rc<Cell<ForcedColors>>,
    /// Host view activation is independent of Document's focused element.
    page_focused: Cell<bool>,
}
impl Drop for PageDomainServices {
    // The impl half is destroyed before the main half (ProxyMain::Stop,
    // proxy_main.cc:916-946).
    fn drop(&mut self) {
        if let Some(mut presenter) = self.synchronous_presenter.get_mut().take() {
            presenter.Shutdown();
        }
        self.ShutdownMainCompositor();
    }
}
#[derive(Clone, Copy)]
struct CompositorAnimationInstallation {
    identity: u64,
    generation: u64,
    element_id: u64,
}

const COMPOSITOR_TRANSFORM_ANIMATION_SALT: u64 = 0x5452_414e_5346_4f52;
const COMPOSITOR_OPACITY_ANIMATION_SALT: u64 = 0x4f50_4143_4954_5900;

fn CompositorAnimationIdentity(effect_id: u64, property: &'static str) -> u64 {
    effect_id
        ^ match property {
            "transform" => COMPOSITOR_TRANSFORM_ANIMATION_SALT,
            "opacity" => COMPOSITOR_OPACITY_ANIMATION_SALT,
            _ => unreachable!("unsupported compositor animation property"),
        }
}

fn CompositorAnimationTimingFunction(
    easing: &animation::CSSTimingFunction,
) -> layerize::commit::animation::AnimationTimingFunction {
    match easing {
        animation::CSSTimingFunction::Linear => layerize::commit::animation::AnimationTimingFunction::Linear,
        animation::CSSTimingFunction::CubicBezier([x1, y1, x2, y2]) => {
            layerize::commit::animation::AnimationTimingFunction::CubicBezier(
                scroll_animation::cubic_bezier::CubicBezier::new(*x1, *y1, *x2, *y2),
            )
        }
        animation::CSSTimingFunction::Steps {
            count,
            jump_at_start,
        } => layerize::commit::animation::AnimationTimingFunction::Steps {
            count: *count,
            jump_at_start: *jump_at_start,
        },
    }
}

fn CompositorPropertyAnimationMutation(
    candidate: &animation::CompositorAnimationCandidate,
    binding: &paint::CompositedAnimationPropertyBinding,
    start_time: Option<scroll_animation::TimeTicks>,
) -> Option<(&'static str, layerize::commit::animation::PropertyAnimationMutation)> {
    let (property, curve, target_property) = match binding.property {
        paint::CompositedAnimationProperty::Transform => {
            let animation::CompositorPropertyCurve::Transform(keyframes) = candidate
                .curves
                .iter()
                .find(|curve| matches!(curve, animation::CompositorPropertyCurve::Transform(_)))?
            else {
                unreachable!()
            };
            let mut curve = layerize::commit::animation::KeyframedTransformAnimationCurve::default();
            for (offset, value) in keyframes {
                curve.AddKeyframe(
                    scroll_animation::TimeDelta::Microseconds(
                        (candidate.timing.duration.max(0.0) * offset * 1_000_000.0) as i64,
                    ),
                    value.clone(),
                    CompositorAnimationTimingFunction(&candidate.timing.easing),
                );
            }
            (
                "transform",
                layerize::commit::animation::CompositorAnimationCurve::Transform(curve),
                layerize::commit::animation::AnimationTargetProperty::TRANSFORM as i32,
            )
        }
        paint::CompositedAnimationProperty::Opacity => {
            let animation::CompositorPropertyCurve::Opacity(keyframes) = candidate
                .curves
                .iter()
                .find(|curve| matches!(curve, animation::CompositorPropertyCurve::Opacity(_)))?
            else {
                unreachable!()
            };
            let mut curve = layerize::commit::animation::KeyframedFloatAnimationCurve::default();
            for (offset, value) in keyframes {
                curve.AddKeyframe(
                    scroll_animation::TimeDelta::Microseconds(
                        (candidate.timing.duration.max(0.0) * offset * 1_000_000.0) as i64,
                    ),
                    *value,
                    CompositorAnimationTimingFunction(&candidate.timing.easing),
                );
            }
            (
                "opacity",
                layerize::commit::animation::CompositorAnimationCurve::Float(curve),
                layerize::commit::animation::AnimationTargetProperty::OPACITY as i32,
            )
        }
    };
    let identity = CompositorAnimationIdentity(candidate.effect_id, property);
    let direction = match candidate.timing.direction {
        animation::CSSPlaybackDirection::Normal => layerize::commit::animation::AnimationDirection::NORMAL,
        animation::CSSPlaybackDirection::Reverse => layerize::commit::animation::AnimationDirection::REVERSE,
        animation::CSSPlaybackDirection::Alternate => {
            layerize::commit::animation::AnimationDirection::ALTERNATE_NORMAL
        }
        animation::CSSPlaybackDirection::AlternateReverse => {
            layerize::commit::animation::AnimationDirection::ALTERNATE_REVERSE
        }
    };
    let fill_mode = match candidate.timing.fill {
        animation::CSSFillMode::None => layerize::commit::animation::AnimationFillMode::NONE,
        animation::CSSFillMode::Forwards => layerize::commit::animation::AnimationFillMode::FORWARDS,
        animation::CSSFillMode::Backwards => layerize::commit::animation::AnimationFillMode::BACKWARDS,
        animation::CSSFillMode::Both => layerize::commit::animation::AnimationFillMode::BOTH,
    };
    Some((
        property,
        layerize::commit::animation::PropertyAnimationMutation::Install {
            identity,
            element: compositor_values::input_types::ElementId::new(
                binding.element_id.GetInternalValue(),
            ),
            curve,
            target_property,
            iterations: candidate.timing.iterations,
            iteration_start: 0.0,
            start_delay: scroll_animation::TimeDelta::Microseconds(
                (candidate.timing.delay * 1_000_000.0) as i64,
            ),
            direction,
            playback_rate: 1.0,
            fill_mode,
            start_time,
            hold_time: (candidate.timing.play_state == animation::CSSPlayState::Paused).then(
                || {
                    scroll_animation::TimeDelta::Microseconds(
                        (candidate.local_time * 1_000_000.0) as i64,
                    )
                },
            ),
        },
    ))
}

struct PageState {
    domain: Rc<PageDomainServices>,
    validation_message: RefCell<Option<validation_message::VisibleValidationMessage>>,
    validation_message_visibility: RefCell<Option<ValidationMessageVisibility>>,
    directory_drop_requests: RefCell<HashMap<u64, u64>>,
    // Continuations contain form identity/attributes only, never Interaction
    // or borrowed fragments. The current Page transaction drains them.
    pending_form_submissions: RefCell<std::collections::VecDeque<interaction::PreparedFormSubmission>>,
    popup_tasks: popup_tasks::PagePopupTasks,
    popup_interaction: RefCell<Weak<PageInteraction>>,
    popup_source_interaction: RefCell<Option<interaction::Interaction<'static>>>,
    document_loader_owners: RefCell<Option<open::OpenDocumentOwners>>,
    document: Rc<DocumentEngine>,
    style_engine: RefCell<style::StyleEngine>,
    style_font_metrics: Rc<crate::style_services::StyleFontMetricsResolver>,
    layout_engine: RefCell<layout::LayoutEngine>,
    paint_engine: RefCell<paint::PaintEngine>,
    /// Embedder/default faces plus decoded @font-face resources. System faces
    /// stay in FontEngine's versioned query cache and are never promoted into
    /// this document-owned catalog.
    document_fonts: RefCell<font_engine::FontFaceCatalog>,
    document_font_inputs: PageDocumentFontInputs,
    system_fonts: RefCell<crate::font_session::SystemFontSession>,
    interaction: Rc<RefCell<UserInteractionState>>,
    layout_editing: Rc<layout::editing::LayoutEditingState>,
    resident_frame_selection:
        RefCell<Weak<interaction::adapter::page_frame_selection::PageFrameSelection>>,
    interaction_text_settings:
        RefCell<Option<interaction::adapter::page_pointer_hit_test::InteractionTextSettings>>,
    scrollbar_scroll_runtime: RefCell<Weak<PageInteractionScrollRuntime>>,
    blocking_wheel_registry_dirty: Cell<bool>,
    /// The root frame's blocking wheel target identities, as Blink's
    /// EventHandlerRegistry holds them between listener changes
    /// (event_handler_registry.cc:128-170 updates only on add/remove).
    /// Valid only for the bindings whose registry invalidation this Page
    /// bound; the invalidation callback clears it.
    blocking_wheel_targets: RefCell<Option<local_frame_scroll::BlockingWheelTargetsCache>>,
    /// The bindings (by address) whose blocking-wheel invalidation callback
    /// reaches this Page; 0 when none was bound.
    blocking_wheel_invalidation_bindings: Cell<usize>,
    mouse_autoscroll:
        RefCell<Weak<interaction::adapter::page_mouse_autoscroll::PageMouseAutoscroll>>,
    autoscroll_cursor_overridden: Cell<bool>,
    bindings: RefCell<Weak<RefCell<DOMJavaScriptBindings>>>,
    resources: RefCell<Option<Rc<ResourceFetcher>>>,
    connected_resources: open::ConnectedResourceDiscovery,
    /// LazyLoadMediaObserver's retained HTML image candidates. ResourceFetcher
    /// announces loader monitoring transitions; lifecycle visibility checks
    /// must not rediscover them with a document-wide geometry walk.
    lazy_image_nodes: RefCell<HashSet<u64>>,
    image_events: RefCell<Vec<PendingImageEvent>>,
    pending_image_events: RefCell<HashMap<u64, Rc<Cell<bool>>>>,
    style_sheet_events: RefCell<Vec<PendingStyleSheetEvent>>,
    pending_style_sheet_events: RefCell<HashMap<u64, Rc<Cell<bool>>>>,
    selectionchange_event_targets: RefCell<Vec<u64>>,
    pending_text_overflow_layout_invalidations: RefCell<Vec<PendingTextOverflowLayoutInvalidation>>,
    scroll_event_targets: RefCell<Vec<u64>>,
    scroll_finished_hover_dirty: Cell<bool>,
    dirty: Cell<bool>,
    // Only offsets have changed since the last completed presentation.
    scroll_only: Cell<bool>,
    styles_resolved: Cell<bool>,
    resource_references_dirty: Cell<ResourceReferenceDirty>,
    // Document::StyleImageReferenceRevision last handed to image discovery.
    discovered_style_image_revision: Cell<u64>,
    // A stable image resource id published a new immutable content snapshot.
    // This never means DOM, layout geometry or display-item recording changed.
    paint_resources_dirty: Cell<bool>,
    /// HTMLVideoElement's original decoded planes. Layout and software Paint
    /// continue to consume the separate PaintImage fallback; Browser retains
    /// this producer value for VideoFrameSubmitter/compositor publication.
    video_frame_resources: RefCell<HashMap<u64, image_resource::VideoFrameResource>>,
    pending_paint_mutations: RefCell<Vec<paint::paint_engine::PaintMutation>>,
    layout_revision: Cell<Option<u64>>,
    #[cfg(test)]
    full_layout_lifecycles: Cell<usize>,
    scroll_updates: RefCell<
        Vec<(
            u64,
            Offset,
            layout_style::style::scroll_enums::mojom::blink::ScrollType,
        )>,
    >,
    // Ownership travels with the queued offsets and is cleared only when the
    // queue is consumed. An empty event flush therefore cannot turn a later
    // compositor batch into a main-thread Paint update.
    compositor_scroll_updates: Cell<bool>,
    animation_engine: RefCell<animation::AnimationEngine>,
    compositor_driven_animations: RefCell<
        BTreeMap<(animation::CSSAnimationKey, &'static str), CompositorAnimationInstallation>,
    >,
    pending_property_animation_replacements:
        RefCell<Vec<layerize::commit::animation::PropertyAnimationMutation>>,
    pending_property_animation_removals: RefCell<BTreeSet<u64>>,
    animation_begin_frame_requested: Cell<bool>,
    window_performance_timing: Rc<webapi::window_bindings::WindowPerformanceTiming>,
    programmatic_scroll_animators: std::cell::OnceCell<interaction::scroll::core::scroll::programmatic_scroll_animator::ProgrammaticScrollAnimatorRegistry>,
}
impl std::ops::Deref for PageState {
    type Target = PageDomainServices;
    fn deref(&self) -> &Self::Target {
        &self.domain
    }
}

/// Read-only LocalFrameView facts for native MouseEvent initialization. Each
/// Browser Page is an independent local root, so root-frame conversion is the
/// source identity branch; Layout remains the owner of zoom and scroll state.
struct PageMouseFrameGeometry(std::rc::Weak<PageState>);

/// Thin Page assembly port over LayoutNG's resident logical-scroll owners.
/// Every method enters the native Layout scope for one call; no LayoutObject,
/// containing-block chain, or scroll tree is copied into Browser.
struct PageInteractionScrollLayout(local_frame_scroll::InteractionScrollOwner);

struct PageUserScrollAnimatorSink(local_frame_scroll::InteractionScrollOwner);

impl interaction::scroll::core::scroll::scroll_animator::UserScrollAnimatorSink
    for PageUserScrollAnimatorSink
{
    fn StartOrUpdate(
        &self,
        _request: interaction::scroll::core::scroll::scroll_animator::UserScrollAnimationStart,
    ) -> Option<interaction::scroll::core::scroll::scroll_animator::UserScrollAnimationRoute> {
        let state = self.0.Page()?;
        state.animation_begin_frame_requested.set(true);
        Some(
            interaction::scroll::core::scroll::scroll_animator::UserScrollAnimationRoute::MainThread,
        )
    }

    fn Cancel(
        &self,
        _area_identity: usize,
        _token: interaction::scroll::core::scroll::scroll_animator::UserScrollAnimationToken,
    ) {
    }

    fn ApplyOffset(
        &self,
        mutation: interaction::scroll::core::scroll::scroll_animator::UserScrollAnimatedOffset,
    ) {
        let Some(node_id) = mutation.node_id else {
            return;
        };
        self.0.ApplyScroll(page_mutation::ScrollMutation {
            target_node_id: node_id,
            offset: Offset {
                x: f64::from(mutation.offset.x()),
                y: f64::from(mutation.offset.y()),
            },
            scroll_type: layout_style::style::scroll_enums::mojom::blink::ScrollType::kUser,
        });
    }

    fn RequestAnimationFrame(&self, _area_identity: usize) {
        if let Some(state) = self.0.Page() {
            state.animation_begin_frame_requested.set(true);
        }
    }
}

struct PageInteractionScrollRuntime {
    owner: local_frame_scroll::InteractionScrollOwner,
    layout: Rc<dyn interaction::adapter::page_scroll_manager::PageLogicalScrollLayout>,
    animators: interaction::scroll::core::scroll::scroll_animator::UserScrollAnimatorRegistry,
}

impl PageInteractionScrollRuntime {
    fn new(
        owner: local_frame_scroll::InteractionScrollOwner,
        layout: Rc<dyn interaction::adapter::page_scroll_manager::PageLogicalScrollLayout>,
    ) -> Self {
        let sink = Rc::new(PageUserScrollAnimatorSink(owner.clone()));
        Self {
            owner,
            layout,
            animators:
                interaction::scroll::core::scroll::scroll_animator::UserScrollAnimatorRegistry::new(
                    sink,
                    Arc::new(Default::default()),
                ),
        }
    }

    fn Tick(&self, frame_time: std::time::Instant) {
        let Some(state) = self.owner.Page() else {
            return;
        };
        let now = state.compositor_executor.TimeTicks(frame_time);
        if self.animators.TickAnimations(now) {
            state.animation_begin_frame_requested.set(true);
        }
    }
}

impl interaction::adapter::page_scroll_manager::PageScrollRuntime for PageInteractionScrollRuntime {
    fn UserAnimationState(
        &self,
        area: &layout::scroll::NativeLogicalScrollArea,
    ) -> layout::scroll::NativeUserScrollAnimationState {
        let enabled = self
            .owner
            .Page()
            .is_some_and(|state| state.has_begin_frame_source.get());
        let snapshot = self
            .animators
            .GetOrCreate(area.Identity(), enabled)
            .Snapshot();
        layout::scroll::NativeUserScrollAnimationState {
            enabled: snapshot.enabled,
            has_running_animation: snapshot.has_running_animation,
            desired_target_offset: snapshot.desired_target_offset,
        }
    }

    fn SnapAnimationState(
        &self,
        area: &layout::scroll::NativeLogicalScrollArea,
    ) -> layout::scroll::NativeSnapAnimationState {
        let user = self.UserAnimationState(area);
        let scroll_element_id = area.GetScrollElementId();
        let programmatic_target = self
            .owner
            .WithProgrammaticAnimations(|animations| animations.TargetForElement(scroll_element_id))
            .flatten();
        layout::scroll::NativeSnapAnimationState {
            scroll_element_id,
            has_running_animation: user.has_running_animation,
            programmatic_target,
        }
    }

    fn LineStep(&self) -> f32 {
        foundation::scroll_utils::kPixelsPerLineStep as f32
    }

    fn ApplySnapPlan(
        &self,
        area: &layout::scroll::NativeLogicalScrollArea,
        plan: layout::scroll::NativeLayoutSnapPlan,
    ) {
        let Some(_state) = self.owner.Page() else {
            return;
        };
        if plan.cancel_user_animation {
            self.animators.Cancel(area.Identity());
        }
        if plan.cancel_programmatic_animation {
            self.owner.WithProgrammaticAnimations(|animations| {
                animations.Cancel(area.GetScrollElementId())
            });
        }
        if let Some(request) = plan.request {
            let behavior = if request.behavior
                == layout_style::style::scroll_enums::mojom::blink::ScrollBehavior::kSmooth
            {
                ViewportScrollBehavior::Smooth
            } else {
                ViewportScrollBehavior::Instant
            };
            self.owner.SetProgrammaticScrollOffset(
                request.node_id,
                Offset {
                    x: f64::from(request.offset.x()),
                    y: f64::from(request.offset.y()),
                },
                behavior,
                request.source_type,
            );
        }
    }

    fn ApplyImmediateUserScrollActions(
        &self,
        area: &layout::scroll::NativeLogicalScrollArea,
        cancel_programmatic_animation: bool,
        finish_previous_animation: bool,
    ) {
        let Some(_state) = self.owner.Page() else {
            return;
        };
        if cancel_programmatic_animation {
            self.owner.WithProgrammaticAnimations(|animations| {
                animations.Cancel(area.GetScrollElementId())
            });
        }
        if finish_previous_animation {
            self.animators.Cancel(area.Identity());
        }
    }

    fn StartUserScrollAnimation(
        &self,
        area: &layout::scroll::NativeLogicalScrollArea,
        request: layout::scroll::NativeUserScrollAnimationRequest,
        completion: interaction::scroll::core::input::scroll_manager::ScrollCompletion,
    ) -> Result<(), interaction::scroll::core::input::scroll_manager::ScrollCompletion> {
        use interaction::scroll::core::input::scroll_manager::ScrollGranularity;
        let enabled = self
            .owner
            .Page()
            .is_some_and(|state| state.has_begin_frame_source.get());
        let animator = self.animators.GetOrCreate(area.Identity(), enabled);
        let Some(current_offset) = self.layout.AreaOffset(area) else {
            return Err(completion);
        };
        let granularity = match request.granularity {
            layout::scroll::NativeUserScrollGranularity::PrecisePixel => ScrollGranularity::kScrollByPrecisePixel,
            layout::scroll::NativeUserScrollGranularity::Pixel => ScrollGranularity::kScrollByPixel,
            layout::scroll::NativeUserScrollGranularity::Line => ScrollGranularity::kScrollByLine,
            layout::scroll::NativeUserScrollGranularity::Page => ScrollGranularity::kScrollByPage,
            layout::scroll::NativeUserScrollGranularity::Document => ScrollGranularity::kScrollByDocument,
        };
        let Some(state) = self.owner.Page() else {
            return Err(completion);
        };
        let now = state
            .compositor_executor.TimeTicks(std::time::Instant::now());
        animator
            .StartOrUpdate(
                interaction::scroll::core::scroll::scroll_animator::UserScrollAnimationRequest {
                    node_id: request.node_id,
                    scroll_element_id: request.scroll_element_id,
                    source_type: request.source_type,
                    granularity,
                    current_offset,
                    target_offset: request.target_offset,
                },
                now,
                completion,
            )
            .map(|_| ())
    }

    fn ApplyScrollFinished(
        &self,
        _area: &layout::scroll::NativeLogicalScrollArea,
        plan: layout::scroll::NativeScrollFinishedPlan,
    ) {
        if plan.mark_hover_state_dirty {
            if let Some(state) = self.owner.Page() {
                state.scroll_finished_hover_dirty.set(true);
            }
        }
    }
}

/// Live frame facts consumed by the translated ScrollManager. Unsupported
/// source owners remain explicit `None`; they are not replaced with Browser
/// policy or a second scroll state.
struct PageInteractionScrollFrameState {
    owner: local_frame_scroll::InteractionScrollOwner,
    mouse: Rc<interaction::core::input::mouse_event_manager::MouseEventManagerState>,
    keyboard: Rc<interaction::core::input::keyboard_event_manager::KeyboardEventManagerState>,
}

impl PageInteractionScrollLayout {
    fn WithAutoscroll<T>(
        &self,
        layout: &layout::scroll::NativeAutoscrollLayout,
        query: impl FnOnce(
            &layout::scroll::NativeAutoscrollSource<'_>,
        ) -> T,
    ) -> Option<T> {
        self.0.WithLayout(|engine| {
            let source = engine.ScrollMut().NativeAutoscrollSource(layout, Default::default())?;
            Some(query(&source))
        })?
    }

    fn WithUserScroll<T>(
        &self,
        area: &layout::scroll::NativeLogicalScrollArea,
        query: impl FnOnce(
            &mut layout::scroll::NativeUserScrollSource<'_>,
        ) -> T,
    ) -> Option<T> {
        self.0.WithLayout(|engine| {
            let mut source = engine.ScrollMut().NativeUserScrollSource(area)?;
            Some(query(&mut source))
        })?
    }
}

impl interaction::adapter::page_scroll_manager::PageScrollFrameState
    for PageInteractionScrollFrameState
{
    fn FocusedNodeId(&self) -> Option<u64> {
        self.owner.FocusedNodeId()
    }

    fn VisualViewportIsActive(&self) -> bool {
        // The current Page has no resident VisualViewport ScrollableArea.
        false
    }

    fn MousePressed(&self) -> bool {
        self.mouse.MousePressed()
    }

    fn BubbleLogicalScrollInParentFrame(
        &self,
        direction: interaction::scroll::core::input::scroll_manager::ScrollDirection,
        granularity: interaction::scroll::core::input::scroll_manager::ScrollGranularity,
    ) -> bool {
        self.owner
            .BubbleLogicalScrollInParentFrame(direction, granularity)
    }

    fn AssertLayoutPrepared(&self, node_id: Option<u64>) {
        self.owner
            .WithLayout(|layout| {
                debug_assert!(layout.Output().is_some());
                if let Some(node_id) = node_id {
                    debug_assert!(layout
                        .EventRegistry()
                        .EventRegistryLayout(Some(node_id))
                        .is_some());
                }
            })
            .expect("logical-scroll Frame owner remains alive");
    }

    fn RespectOverscrollBehaviorForScrollBubbling(&self) -> bool {
        foundation::RuntimeEnabledFeatures::RespectOverscrollBehaviorForScrollBubblingEnabled()
    }

    fn AutoscrollController(
        &self,
    ) -> Option<Rc<dyn interaction::scroll::core::input::scroll_manager::ScrollAutoscrollController>>
    {
        self.owner.AutoscrollController()
    }

    fn KeyboardEventManager(
        &self,
    ) -> Option<Rc<dyn interaction::scroll::core::input::scroll_manager::ScrollKeyboardEventManager>>
    {
        Some(self.keyboard.clone())
    }
}

impl interaction::adapter::page_scroll_manager::PageLogicalScrollLayout
    for PageInteractionScrollLayout
{
    fn LayoutForNode(
        &self,
        node_id: u64,
    ) -> Option<layout::scroll::NativeAutoscrollLayout> {
        self.0
            .WithLayout(|layout| layout.Scroll().LogicalScrollLayoutFor(node_id))?
    }

    fn LayoutView(
        &self,
    ) -> Option<layout::scroll::NativeAutoscrollLayout> {
        self.0
            .WithLayout(|layout| layout.Scroll().LogicalScrollLayoutView())?
    }

    fn NodeFacts(
        &self,
        layout: &layout::scroll::NativeAutoscrollLayout,
    ) -> Option<layout::scroll::NativeLogicalScrollNodeFacts> {
        self.WithAutoscroll(layout, |source| source.LogicalScrollNodeFacts())?
    }

    fn EnclosingBox(
        &self,
        layout: &layout::scroll::NativeAutoscrollLayout,
    ) -> Option<layout::scroll::NativeAutoscrollLayout> {
        self.WithAutoscroll(layout, |source| source.EnclosingBox())?
    }

    fn LayoutBox(
        &self,
        layout: &layout::scroll::NativeAutoscrollLayout,
    ) -> Option<layout::scroll::NativeAutoscrollLayout> {
        self.WithAutoscroll(layout, |source| source.LogicalLayoutBox())?
    }

    fn LayoutBoxForScrolling(
        &self,
        layout: &layout::scroll::NativeAutoscrollLayout,
    ) -> Option<layout::scroll::NativeAutoscrollLayout> {
        self.WithAutoscroll(layout, |source| source.LayoutBoxForScrolling())?
    }

    fn BoxFacts(
        &self,
        layout: &layout::scroll::NativeAutoscrollLayout,
    ) -> layout::scroll::NativeLogicalScrollBoxFacts {
        self.WithAutoscroll(layout, |source| source.LogicalScrollBoxFacts())
            .expect("retained logical-scroll layout remains attached")
    }

    fn BoxNode(
        &self,
        layout: &layout::scroll::NativeAutoscrollLayout,
    ) -> Option<layout::scroll::NativeAutoscrollLayout> {
        self.WithAutoscroll(layout, |source| source.LogicalNode())?
    }

    fn ContainingBlock(
        &self,
        layout: &layout::scroll::NativeAutoscrollLayout,
    ) -> Option<layout::scroll::NativeAutoscrollLayout> {
        self.WithAutoscroll(layout, |source| source.ContainingBlock())?
    }

    fn ScrollableArea(
        &self,
        layout: &layout::scroll::NativeAutoscrollLayout,
    ) -> Option<layout::scroll::NativeLogicalScrollArea> {
        self.WithAutoscroll(layout, |source| source.LogicalScrollableArea())?
    }

    fn AreaForScrolling(
        &self,
        layout: &layout::scroll::NativeAutoscrollLayout,
    ) -> Option<layout::scroll::NativeLogicalScrollingArea> {
        self.WithAutoscroll(layout, |source| source.LogicalAreaForScrolling())?
    }

    fn LayoutViewportArea(
        &self,
    ) -> Option<layout::scroll::NativeLogicalScrollArea> {
        self.0
            .WithLayout(|layout| layout.LayoutViewportLogicalScrollArea())?
    }

    fn AreaFacts(
        &self,
        area: &layout::scroll::NativeLogicalScrollArea,
    ) -> Option<layout::scroll::NativeUserScrollAreaFacts> {
        self.WithUserScroll(area, |source| source.Facts())
    }

    fn AreaOffset(
        &self,
        area: &layout::scroll::NativeLogicalScrollArea,
    ) -> Option<foundation::gfx::Vector2dF> {
        self.WithUserScroll(area, |source| source.GetScrollOffset())
    }

    fn PlanSnapForUserScroll(
        &self,
        area: &layout::scroll::NativeLogicalScrollArea,
        kind: layout::scroll::NativeUserSnapScrollKind,
        direction: foundation::PhysicalDirection,
        line_step: f32,
        animation: layout::scroll::NativeSnapAnimationState,
    ) -> Option<(
        bool,
        layout::scroll::NativeLayoutSnapPlan,
    )> {
        self.WithUserScroll(area, |source| {
            source.PlanSnapForUserScroll(kind, direction, line_step, animation)
        })
    }

    fn UserScroll(
        &self,
        area: &layout::scroll::NativeLogicalScrollArea,
        granularity: layout::scroll::NativeUserScrollGranularity,
        delta: foundation::gfx::Vector2dF,
        source_type: interaction::scroll::core::input::scroll_manager::ScrollSourceType,
        animation: layout::scroll::NativeUserScrollAnimationState,
    ) -> Option<layout::scroll::NativeUserScrollPlan> {
        self.WithUserScroll(area, |source| {
            source.UserScroll(granularity, delta, source_type, animation)
        })
    }

    fn WillConsumeUserScroll(
        &self,
        area: &layout::scroll::NativeLogicalScrollArea,
        granularity: layout::scroll::NativeUserScrollGranularity,
        delta: foundation::gfx::Vector2dF,
        animation: layout::scroll::NativeUserScrollAnimationState,
    ) -> Option<bool> {
        self.WithUserScroll(area, |source| {
            source.WillConsumeUserScroll(granularity, delta, animation)
        })
    }

    fn OnScrollFinished(
        &self,
        area: &layout::scroll::NativeLogicalScrollArea,
        enqueue_scrollend: bool,
    ) -> Option<layout::scroll::NativeScrollFinishedPlan> {
        self.WithUserScroll(area, |source| source.OnScrollFinished(enqueue_scrollend))?
    }

    fn WheelScrollableAreas(
        &self,
        target_node_id: u64,
    ) -> Box<
        dyn Iterator<
            Item = layout::scroll::NativeWheelScrollableArea,
        >,
    > {
        self.0
            .WithLayout(|layout| {
                Box::new(layout.WheelScrollableAreaTraversal(target_node_id))
                    as Box<dyn Iterator<Item = _>>
            })
            .unwrap_or_else(|| Box::new(std::iter::empty()))
    }
}

impl interaction::event_dispatcher::MouseFrameGeometry for PageMouseFrameGeometry {
    fn AcceptsLoadDrops(&self) -> bool {
        self.0
            .upgrade()
            .is_some_and(|state| state.client.borrow().AcceptsLoadDrops())
    }
    fn LoadDroppedURL(&self, url: &str, foreground: bool, user_activation: bool) {
        let Some(state) = self.0.upgrade() else {
            return;
        };
        let origin = state
            .document
            .borrow()
            .GetDocument()
            .SecurityContext()
            .Origin()
            .DeriveNewOpaqueOrigin();
        let request = DropNavigationRequest {
            request: URLRequest {
                url: url.to_owned(),
                navigation: Some(navigation_history::NavigationRequestMetadata {
                    load_type: navigation_history::FrameLoadType::Standard,
                    history_item: None,
                    back_entries: Arc::from([]),
                    forward_entries: Arc::from([]),
                    navigation_api_state: None,
                    navigation_type: navigation_history::WebNavigationType::Other,
                    is_browser_initiated: false,
                    had_transient_user_activation: user_activation,
                    requestor_origin: Some(navigation_history::RequestorOrigin::Opaque(
                        origin.OpaqueNonce().expect("unique opaque drop origin"),
                    )),
                    propagated_text_fragment_token: None,
                    internal_scroll_to_text_fragment: None,
                }),
                ..Default::default()
            },
            foreground,
        };
        state.client.borrow_mut().DidRequestDropNavigation(&request);
    }
    fn EnumerateDroppedDirectory(&self, target: u64, directory: Arc<fileapi::File>) -> bool {
        let Some(state) = self.0.upgrade() else {
            return false;
        };
        static NEXT_REQUEST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let request_id = NEXT_REQUEST.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let document_identity = state
            .document
            .borrow()
            .GetDocument()
            .RootHandle()
            .Identity();
        state
            .directory_drop_requests
            .borrow_mut()
            .insert(target, request_id);
        let accepted =
            state
                .client
                .borrow()
                .EnumerateDroppedDirectory(DirectoryEnumerationRequest {
                    request_id,
                    document_identity,
                    target,
                    directory,
                });
        if !accepted {
            state.directory_drop_requests.borrow_mut().remove(&target);
        }
        accepted
    }
    fn DragTargetStateChanged(&self) {
        if let Some(state) = self.0.upgrade() {
            state.InvalidateStylePreservingGeometry();
        }
    }

    fn NodeDragImage(
        &self,
        node: u64,
    ) -> io::Result<
        Option<Arc<interaction::dispatch::core::clipboard::data_transfer::DragImageBitmap>>,
    > {
        PageMouseFrameGeometry::NodeDragImage(self, node)
    }
    fn SelectionDragOverlay(
        &self,
    ) -> io::Result<interaction::default::core::page::drag_controller::DragOverlay> {
        PageMouseFrameGeometry::SelectionDragOverlay(self)
    }
    fn ResourceDragImage(
        &self,
        node: u64,
        size: (i32, i32),
    ) -> io::Result<
        Option<Arc<interaction::dispatch::core::clipboard::data_transfer::DragImageBitmap>>,
    > {
        PageMouseFrameGeometry::ResourceDragImage(self, node, size)
    }
    fn LinkDragImage(
        &self,
        url: &str,
        label: &[u16],
    ) -> io::Result<
        Option<Arc<interaction::dispatch::core::clipboard::data_transfer::DragImageBitmap>>,
    > {
        PageMouseFrameGeometry::LinkDragImage(self, url, label)
    }

    fn HasContentLayout(&self) -> bool {
        self.0.upgrade().is_some_and(|state| {
            state
                .layout_engine
                .borrow()
                .Scroll()
                .LogicalScrollLayoutView()
                .is_some()
        })
    }
    fn DragFrameToViewport(&self, point: foundation::gfx::Point) -> Option<foundation::gfx::Point> {
        // This Page is the actual local root and its VisualViewport currently
        // has no pinch offset/scale, matching its compositor viewport metadata.
        self.HasFrameView().then_some(point)
    }

    fn AutoscrollLayoutForNode(
        &self,
        node: u64,
    ) -> Option<Rc<dyn interaction::default::core::page::autoscroll_controller::AutoscrollLayout>>
    {
        mouse_autoscroll::LayoutForNode(&self.0.upgrade()?, node)
    }
    fn ScheduleAutoscrollAnimation(&self) {
        if let Some(state) = self.0.upgrade() {
            state.animation_begin_frame_requested.set(true);
        }
    }

    fn StartSystemDrag(
        &self,
        request: interaction::default::core::page::drag_controller::SystemDragRequest,
    ) -> interaction::default::core::page::drag_controller::SystemDragSupport {
        let Some(state) = self.0.upgrade() else {
            return interaction::default::core::page::drag_controller::SystemDragSupport::Unsupported;
        };
        let result = state.client.borrow().StartSystemDrag(request);
        result
    }
    fn DragLayout(
        &self,
        node_id: u64,
    ) -> Option<layout::scroll::NativeMouseCoordinateLayout> {
        let state = self.0.upgrade()?;
        state.EnsureLayoutOutput();
        let result = state.layout_engine.borrow().Pointer().MouseCoordinateLayout(node_id);
        result
    }
    fn DragBoundingBox(&self, node_id: u64) -> Option<foundation::gfx::Rect> {
        let state = self.0.upgrade()?;
        state.EnsureLayoutOutput();
        let layout = state.layout_engine.borrow();
        let object = layout.Pointer().MouseCoordinateLayout(node_id)?;
        Some(layout.Pointer().AbsoluteBoundingBoxIncludingDescendants(&object))
    }
    fn SimulatedClickRootFrameCenter(
        &self,
        node: u64,
    ) -> Option<(foundation::PhysicalOffset, f32)> {
        let state = self.0.upgrade()?;
        let layout = state.layout_engine.borrow();
        let object = layout.Pointer().MouseCoordinateLayout(node)?;
        layout.Pointer().SimulatedClickRootFrameCenter(&object)
    }
    fn HasFrameView(&self) -> bool {
        self.0.upgrade().is_some_and(|state| {
            state.document.borrow().GetDocument().IsActive()
                && state.layout_engine.borrow().Output().is_some()
        })
    }

    fn LayoutZoomFactor(&self) -> Option<f32> {
        let state = self.0.upgrade()?;
        let zoom = state
            .layout_engine
            .borrow()
            .GetConstraints()
            .layout_zoom_factor as f32;
        Some(zoom)
    }

    fn DevicePixelRatio(&self) -> Option<f64> {
        let state = self.0.upgrade()?;
        let device_pixel_ratio = state
            .layout_engine
            .borrow()
            .GetConstraints()
            .device_pixel_ratio;
        Some(device_pixel_ratio)
    }

    fn LayoutViewportScrollOffset(&self) -> Option<(i32, i32)> {
        let state = self.0.upgrade()?;
        let offset = state
            .layout_engine
            .borrow()
            .Pointer().MouseLayoutViewportScrollOffsetInt();
        offset
    }

    fn HasLayoutObject(&self, node_id: u64) -> bool {
        self.0.upgrade().is_some_and(|state| {
            state
                .layout_engine
                .borrow()
                .EventRegistry()
                .EventRegistryLayout(Some(node_id))
                .is_some()
        })
    }

    fn IsUserScrollable(&self, node_id: u64) -> bool {
        self.0.upgrade().is_some_and(|state| {
            let engine = state.layout_engine.borrow();
            engine
                .Output()
                .and_then(|output| output.Node(node_id))
                .is_some_and(|node| node.FocusNavigationFacts().is_user_scrollable)
        })
    }

    fn IsCanvasLayoutObject(&self, node_id: u64) -> bool {
        self.0.upgrade().is_some_and(|state| {
            let engine = state.layout_engine.borrow();
            engine
                .Output()
                .and_then(|output| output.Node(node_id))
                .is_some_and(|node| node.FocusNavigationFacts().is_canvas)
        })
    }

    fn ConvertFromRootFrame(
        &self,
        point: interaction::input::public::common::input::web_pointer_properties::PointF,
    ) -> Option<interaction::input::public::common::input::web_pointer_properties::PointF> {
        self.HasFrameView().then_some(point)
    }

    fn RootFrameToViewport(
        &self,
        point: foundation::gfx::PointF,
    ) -> Option<foundation::gfx::PointF> {
        if !self.HasFrameView() {
            return None;
        }
        let state = self.0.upgrade()?;
        let point = state.web_view_zoom.borrow().RootFrameToViewport(point);
        Some(point)
    }

    fn NonLocatedContextMenuCoordinates(&self) -> Option<interaction::adapter::page_nonlocated_context_menu_coordinates::NativeContextMenuFrameCoordinates>{
        if !self.HasFrameView() {
            return None;
        }
        let state = self.0.upgrade()?;
        let widget = state.context_menu_widget_geometry.get()?;
        let constraints = state.constraints.borrow();
        // This provider is constructed for the Page domain's actual local
        // root. Its VisualViewport currently has no pinch scale/offset, as
        // recorded by the same compositor metadata producer.
        let coordinates = interaction::adapter::page_nonlocated_context_menu_coordinates::NativeContextMenuFrameCoordinates {
            visual_viewport_offset: foundation::gfx::PointF::default(),
            visual_viewport_scale: 1.0,
            visual_viewport_size: foundation::gfx::Size::new(
                (constraints.available_size.width * constraints.layout_zoom_factor).round() as i32,
                (constraints.available_size.height * constraints.layout_zoom_factor).round() as i32,
            ),
            widget_screen_origin_dip: widget.screen_origin_dip,
            blink_to_dip_scale: widget.blink_to_dip_scale,
            outermost_main_frame: true,
        };
        coordinates.IsValid().then_some(coordinates)
    }
    fn NonLocatedRectFromRootFrame(
        &self,
        rect: foundation::gfx::Rect,
    ) -> Option<foundation::gfx::Rect> {
        // LocalFrameView::ConvertFromRootFrame's ParentFrameView()==nullptr
        // branch. Child frame views are separate LocalFrame owners and never
        // use this Page-local-root provider.
        self.HasFrameView().then_some(rect)
    }
    fn NonLocatedPointToRootFrame(
        &self,
        point: foundation::gfx::Point,
    ) -> Option<foundation::gfx::Point> {
        self.HasFrameView().then_some(point)
    }
    fn NonLocatedRectToRootFrame(
        &self,
        rect: foundation::gfx::Rect,
    ) -> Option<foundation::gfx::Rect> {
        self.HasFrameView().then_some(rect)
    }

    fn PointerLockPosition(
        &self,
    ) -> Option<(
        interaction::input::public::common::input::web_pointer_properties::PointF,
        interaction::input::public::common::input::web_pointer_properties::PointF,
    )> {
        // Browser has no pointer-lock controller owner yet.
        None
    }
}

impl interaction::adapter::page_element_focusability::PageElementFocusLayout
    for PageMouseFrameGeometry
{
    fn HasLayoutObject(&self, node_id: u64) -> bool {
        interaction::event_dispatcher::MouseFrameGeometry::HasLayoutObject(self, node_id)
    }

    fn IsUserScrollable(&self, node_id: u64) -> bool {
        interaction::event_dispatcher::MouseFrameGeometry::IsUserScrollable(self, node_id)
    }

    fn IsCanvasLayoutObject(&self, node_id: u64) -> bool {
        interaction::event_dispatcher::MouseFrameGeometry::IsCanvasLayoutObject(self, node_id)
    }
}

impl interaction::adapter::page_focus_navigation::FocusNavigationLayout for PageMouseFrameGeometry {
    fn GridFocusgroup(
        &self,
        root: u64,
        query: interaction::adapter::page_focus_navigation::FocusgroupGridQuery,
    ) -> Option<interaction::adapter::page_focus_navigation::FocusgroupGridResult> {
        let state = self.0.upgrade()?;
        let layout = state.layout_engine.borrow();
        let document = state.document.borrow();
        interaction::adapter::page_focus_navigation::QueryGridFocusgroup(
            document.GetDocument(),
            layout.Output()?,
            root,
            query,
        )
    }
    fn UpdateStyleAndLayoutForFocus(&self) {
        if let Some(state) = self.0.upgrade() {
            state.EnsureLayoutOutput();
        }
    }

    fn Facts(
        &self,
        node_id: u64,
    ) -> Option<layout::output::LayoutFocusNavigationFacts> {
        let state = self.0.upgrade()?;
        let layout = state.layout_engine.borrow();
        layout
            .Output()?
            .Node(node_id)
            .map(|node| node.FocusNavigationFacts())
    }

    fn ReadingFlowNodes(&self, node_id: u64) -> Vec<u64> {
        let Some(state) = self.0.upgrade() else {
            return Vec::new();
        };
        let layout = state.layout_engine.borrow();
        layout
            .Output()
            .and_then(|output| output.Node(node_id))
            .map_or_else(Vec::new, |node| node.ReadingFlowNodes())
    }
}

impl interaction::adapter::page_focus_navigation::FocusNavigationFramePort
    for PageMouseFrameGeometry
{
    fn FrameTree(
        &self,
    ) -> Option<Rc<dyn interaction::adapter::page_focus_navigation::FocusNavigationFrameTree>> {
        let page = self.0.upgrade()?;
        Some(local_frame_focus::NativeFocusFrameTree::Root(
            page.local_frames.get()?,
        ))
    }

    fn HasView(&self) -> bool {
        interaction::event_dispatcher::MouseFrameGeometry::HasFrameView(self)
    }

    fn IsPagePaused(&self) -> bool {
        // Rechrom has no Page-pauser owner; this Page domain is running while
        // it can receive a host input transaction.
        false
    }

    fn CanTakeFocus(
        &self,
        _direction: interaction::default::public::mojom::input::focus_type::FocusType,
    ) -> bool {
        // Browser chrome does not participate in this Page's focus traversal.
        false
    }

    fn TakeFocus(
        &self,
        _direction: interaction::default::public::mojom::input::focus_type::FocusType,
    ) {
        debug_assert!(false, "TakeFocus requires CanTakeFocus");
    }

    fn StopAutoscroll(
        &self,
        scope: &interaction::default::core::page::focus_controller::FocusDispatchScope<'_>,
    ) {
        if let Some(state) = self.0.upgrade() {
            if let Some(owner) = state.mouse_autoscroll.borrow().upgrade() {
                owner.StopAutoscrollWithNodeRelease(&|node| {
                    if let Some(events) = scope.focus_events {
                        events.StopNodeAutoscroll(node);
                    }
                });
            }
        }
    }
    fn FrameEffect(
        &self,
        effect: interaction::adapter::page_focus_navigation::FocusNavigationFrameEffect,
    ) {
        let Some(state) = self.0.upgrade() else {
            return;
        };
        match effect {
            interaction::adapter::page_focus_navigation::FocusNavigationFrameEffect::SelectionFrameIsFocused(_)
            | interaction::adapter::page_focus_navigation::FocusNavigationFrameEffect::DidFocus
            | interaction::adapter::page_focus_navigation::FocusNavigationFrameEffect::WillChangeInputMethodFocus => {
                // These source notifications have no separate embedder owner in
                // the current single-frame Page; DOM focus state is canonical.
            }
            interaction::adapter::page_focus_navigation::FocusNavigationFrameEffect::StopAutoscroll => {
                if let Some(owner) = state.mouse_autoscroll.borrow().upgrade() {
                    use interaction::scroll::core::input::scroll_manager::ScrollAutoscrollController;
                    owner.StopAutoscroll();
                }
            }
            interaction::adapter::page_focus_navigation::FocusNavigationFrameEffect::SelectionPageActivationChanged => {
                state.dirty.set(true);
            }
        }
    }

    fn HandleInterestForHoverOrFocus(&self, _node_id: u64, _focused: bool) {
        // InterestTargetController is not enabled by this embedding.
    }

    fn UpdateFocusDataForSnapAreas(
        &self,
        update: interaction::adapter::page_focus_controller::FocusWithinSnapUpdate,
    ) {
        self.ApplyFocusWithinSnapUpdate(update);
    }
}

impl PageMouseFrameGeometry {
    fn ApplyFocusWithinSnapUpdate(
        &self,
        update: interaction::adapter::page_focus_controller::FocusWithinSnapUpdate,
    ) {
        let Some(state) = self.0.upgrade() else {
            return;
        };
        let editing_facts = {
            let owner = state.document.borrow();
            let document = owner.GetDocument();
            update
                .editing_fact_nodes
                .into_iter()
                .filter_map(|node_id| {
                    document
                        .FindNodeById(node_id)
                        .map(|node| (node_id, document.EditingNodeFacts(node)))
                })
                .collect::<Vec<_>>()
        };
        let mut layout = state.layout_engine.borrow_mut();
        for (node_id, facts) in editing_facts {
            layout.ApplyMutation(layout::mutation::LayoutMutation::Selection(
                layout::views::selection::SelectionMutation::UpdateEditingNodeFactsForFocus {
                    node_id,
                    facts,
                },
            ));
        }
        for node_id in update.snap_container_candidates {
            if let Some(mut source) = layout.ScrollMut().NativeScrollSource(node_id) {
                source.UpdateFocusDataForSnapAreas();
            }
        }
    }

    fn CurrentSelection(
        state: &PageState,
    ) -> Option<layout::editing::DOMSelection> {
        let owner = state.document.borrow();
        interaction::adapter::page_frame_selection::CurrentDOMSelectionForFocus(
            owner.GetDocument(),
            &state.layout_editing.selections,
        )
    }

    fn CaretBounds(
        state: &PageState,
        selection: Option<layout::editing::DOMSelection>,
        extent: interaction::default::core::editing::frame_selection::RevealExtentOption,
    ) -> foundation::gfx::Rect {
        state.EnsureLayoutOutput();
        let layout = state.layout_engine.borrow();
        let Some(output) = layout.Output() else {
            return Default::default();
        };
        let owner = state.document.borrow();
        interaction::adapter::page_focus_controller::CaretBoundsForSelectionForFocus(
            owner.GetDocument(),
            output,
            selection,
            extent,
            foundation::LayoutUnit::from_f64(layout.GetConstraints().device_pixel_ratio.max(1.0))
                .ToDouble(),
        )
    }
}

impl interaction::adapter::page_focus_controller::DocumentFocusLifecycle
    for PageMouseFrameGeometry
{
    fn PageFocusTarget(
        &self,
        node: u64,
    ) -> Option<(
        Rc<interaction::default::core::page::focus_controller::FocusController>,
        interaction::default::core::page::focus_controller::FocusFrameRef,
        interaction::default::core::page::focus_controller::FocusElementRef,
    )> {
        let page = self.0.upgrade()?;
        let domain = page.local_frames.get()?;
        let controller = domain.focus_controller.borrow().clone()?;
        Some((
            controller,
            domain.FocusFrame(domain.root)?,
            domain
                .FocusInteraction(domain.root)?
                .FocusElementOwner(node)?,
        ))
    }

    fn StopClearFocusedElementTimer(&self) {
        // This single-frame embedding does not schedule Blink's deferred clear
        // timer. The required port still terminates at the Page owner rather
        // than being silently supplied by Interaction.
    }

    fn HasFrame(&self) -> bool {
        self.0
            .upgrade()
            .is_some_and(|state| state.document.borrow().GetDocument().IsActive())
    }

    fn IsPageFocused(&self) -> bool {
        self.0
            .upgrade()
            .is_some_and(|state| state.page_focused.get())
    }

    fn UpdateStyleAndLayoutTreeForElement(&self, _node_id: u64) {
        if let Some(state) = self.0.upgrade() {
            state.ResolveStyles();
        }
    }

    fn UpdateStyleAndLayoutForNode(&self, node_id: u64) {
        if let Some(state) = self.0.upgrade() {
            state.EnsureLayoutOutputForNode(node_id);
        }
    }

    fn UpdateStyleAndLayoutTree(&self) {
        if let Some(state) = self.0.upgrade() {
            state.ResolveStyles();
        }
    }

    fn FrameSelectionHost(
        &self,
    ) -> Rc<dyn interaction::adapter::page_frame_selection::PageFrameSelectionHost> {
        frame_selection_host::ResidentFrameSelectionHost::Root(self.0.clone())
    }

    fn IsLayoutEmbeddedContent(&self, node_id: u64) -> bool {
        self.0.upgrade().is_some_and(|state| {
            state.EnsureLayoutOutput();
            state
                .layout_engine
                .borrow()
                .Output()
                .and_then(|output| output.Node(node_id))
                .is_some_and(|node| node.FocusNavigationFacts().is_embedded_content)
        })
    }

    fn ScrollBoundingBoxForFocusIntoView(&self, node_id: u64) {
        let Some(state) = self.0.upgrade() else {
            return;
        };
        let result = {
            let mut layout = state.layout_engine.borrow_mut();
            layout
                .ScrollMut()
                .NativeScrollSource(node_id)
                .map(|mut source| source.ScrollFocusIntoView(true))
        };
        if let Some(result) = result {
            state.ApplyNativeScrollResult(result);
        }
    }

    fn UpdateFocusDataForSnapAreas(
        &self,
        update: interaction::adapter::page_focus_controller::FocusWithinSnapUpdate,
    ) {
        self.ApplyFocusWithinSnapUpdate(update);
    }

    fn NotifyFocusedElementChanged(
        &self,
        old_node_id: Option<u64>,
        new_node_id: Option<u64>,
        focus_type: interaction::default::public::mojom::input::focus_type::FocusType,
    ) {
        if let Some(state) = self.0.upgrade() {
            let navigation = state
                .document
                .borrow()
                .GetDocument()
                .WindowNavigationApiOwner();
            if let Some(navigation) = navigation {
                navigation.FocusedElementChanged();
            }
            state.ValidationMessageFocusChanged(new_node_id);
            state.dirty.set(true);
            state
                .client
                .borrow_mut()
                .DidChangeFocusedElement(old_node_id, new_node_id, focus_type);
        }
    }
}

impl PageState {
    /// Document::Shutdown releases the resident Layout/Style graph before the
    /// Document lifecycle and Window execution context are retired. Root
    /// navigation keeps PageDomainServices alive, so relying on PageState's
    /// eventual Rc destruction leaves resolver-owned cycles from the old
    /// document detached from every live root but still allocated.
    fn ShutdownDocumentEngines(&self) {
        if self.style_engine.borrow().IsDetached() {
            return;
        }
        {
            let mut heap = foundation::LayoutHeapScope::new();
            heap.DeferCollection();
            {
                let document = self.document.borrow();
                let mut layout = self.layout_engine.borrow_mut();
                crate::persistent_layout::DetachDocumentLayoutTreeWithEngine(
                    &mut layout,
                    &document,
                );
            }
            self.style_engine.borrow_mut().DidDetach();
        }
        // Dropping Persistent roots records heap activity but deliberately
        // does not use allocation pressure to request a collection. Retiring
        // a whole Document is the explicit cppgc scheduling point: enqueue a
        // low-priority owner turn after the non-nestable detach scope exits.
        foundation::RequestLayoutHeapCollection();
    }

    fn InteractionMouseFrameGeometry(self: &Rc<Self>) -> Rc<PageMouseFrameGeometry> {
        Rc::new(PageMouseFrameGeometry(Rc::downgrade(self)))
    }

    fn InteractionEventClock(
        &self,
    ) -> Rc<dyn interaction::dispatch::core::dom::events::event::EventClock> {
        Rc::new(PageInteractionEventClock(
            self.compositor_executor.clone(),
        ))
    }

    pub(super) fn PublishPreparedDocumentInputs(&self) {
        if self.document_font_inputs.pending.replace(false) {
            let mut constraints = self.constraints.borrow_mut();
            constraints.images = Default::default();
            constraints.canvases = Default::default();
            constraints.fullscreen_element_blocks_layout_view_scroll = false;
            constraints.font_engine = Some(self.document_font_inputs.engine.clone());
            constraints.font_queries = self.document_font_inputs.queries.clone();
            constraints.fonts = self.document_font_inputs.faces.clone();
        }
    }
    fn DocumentLifecycle(&self) -> Rc<dom::document_lifecycle::DocumentLifecycle> {
        self.document.borrow().GetDocument().Lifecycle().clone()
    }
    fn CommitRetainedPaintLifecycle(&self) {
        use dom::document_lifecycle::LifecycleState::*;
        let lifecycle = self.DocumentLifecycle();
        match lifecycle.GetState() {
            PaintClean => return,
            StyleClean => lifecycle.AdvanceTo(LayoutClean),
            LayoutClean | CompositingInputsClean | PrePaintClean => {}
            state => panic!("retained paint requires a completed lifecycle phase: {state:?}"),
        }
        if lifecycle.GetState() != PrePaintClean {
            lifecycle.AdvanceTo(InPrePaint);
            lifecycle.AdvanceTo(PrePaintClean);
        }
        lifecycle.AdvanceTo(InPaint);
        lifecycle.AdvanceTo(PaintClean);
    }
    pub(super) fn InstallDocumentLoaderOwners(&self, owners: open::OpenDocumentOwners) {
        *self.document_loader_owners.borrow_mut() = Some(owners);
    }

    pub(super) fn DocumentLoaderOwners(&self) -> open::OpenDocumentOwners {
        self.document_loader_owners
            .borrow()
            .clone()
            .expect("Page installs its current DocumentLoader owners before interaction assembly")
    }

    pub(super) fn DocumentLoaderLifecycle(&self) -> Rc<open::OpenDocumentLifecycleState> {
        self.DocumentLoaderOwners().lifecycle
    }

    pub(super) fn DocumentLoaderInitialScroll(&self) -> Rc<document_loader::InitialScrollState> {
        self.DocumentLoaderOwners().initial_scroll
    }

    pub(super) fn DocumentLoaderNavigation(
        &self,
    ) -> Rc<document_loader::DocumentLoaderNavigationState> {
        self.DocumentLoaderOwners().navigation
    }

    fn ParseCSSKeyframeOffsets(value: &str) -> Vec<f64> {
        value
            .split(',')
            .filter_map(|value| {
                let value = value.trim();
                if value.eq_ignore_ascii_case("from") {
                    return Some(0.0);
                }
                if value.eq_ignore_ascii_case("to") {
                    return Some(1.0);
                }
                value
                    .strip_suffix('%')
                    .and_then(|value| value.trim().parse::<f64>().ok())
                    .map(|value| value / 100.0)
                    .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
            })
            .collect()
    }

    fn CSSAnimationTiming(
        data: &layout_style::style::css_timing_data::CSSAnimationData,
        index: usize,
    ) -> animation::CSSAnimationTiming {
        use animation::{CSSFillMode, CSSPlayState, CSSPlaybackDirection, CSSTimingFunction};
        use layout_style::style::{
            computed_style_constants::EAnimPlayState,
            css_timing_data::{
                EaseType, FillMode, PlaybackDirection, StepPosition, TimingFunction,
            },
        };

        let timing = data.ConvertToTiming(index);
        let direction = match timing.direction {
            PlaybackDirection::NORMAL => CSSPlaybackDirection::Normal,
            PlaybackDirection::REVERSE => CSSPlaybackDirection::Reverse,
            PlaybackDirection::ALTERNATE_NORMAL => CSSPlaybackDirection::Alternate,
            PlaybackDirection::ALTERNATE_REVERSE => CSSPlaybackDirection::AlternateReverse,
        };
        let fill = match timing.fill_mode {
            FillMode::FORWARDS => CSSFillMode::Forwards,
            FillMode::BACKWARDS => CSSFillMode::Backwards,
            FillMode::BOTH => CSSFillMode::Both,
            FillMode::AUTO | FillMode::NONE => CSSFillMode::None,
        };
        let easing = match &*timing.timing_function {
            TimingFunction::Linear(_) => CSSTimingFunction::Linear,
            TimingFunction::CubicBezierPreset(preset) => {
                CSSTimingFunction::CubicBezier(match preset {
                    EaseType::EASE => [0.25, 0.1, 0.25, 1.0],
                    EaseType::EASE_IN => [0.42, 0.0, 1.0, 1.0],
                    EaseType::EASE_OUT => [0.0, 0.0, 0.58, 1.0],
                    EaseType::EASE_IN_OUT => [0.42, 0.0, 0.58, 1.0],
                })
            }
            TimingFunction::CubicBezier(values) => CSSTimingFunction::CubicBezier(*values),
            TimingFunction::Steps {
                number_of_steps,
                step_position,
            } => CSSTimingFunction::Steps {
                count: (*number_of_steps).max(1) as u32,
                jump_at_start: matches!(
                    step_position,
                    StepPosition::START | StepPosition::JUMP_START | StepPosition::JUMP_BOTH
                ),
            },
        };
        animation::CSSAnimationTiming {
            delay: timing.start_delay.time_delay,
            duration: timing.iteration_duration.unwrap_or(0.0),
            iterations: timing.iteration_count,
            direction,
            fill,
            play_state: if *layout_style::style::css_timing_data::CSSTimingData::GetRepeated(
                data.PlayStateList(),
                index,
            ) == EAnimPlayState::kPaused
            {
                CSSPlayState::Paused
            } else {
                CSSPlayState::Running
            },
            easing,
        }
    }

    /// Synchronize CSSAnimationUpdate after style recalc, before layout.
    ///
    /// Chromium source: css_animations.cc CalculateAnimationUpdate consumes
    /// the element's computed CSSAnimationData and the winning @keyframes rule;
    /// CSSAnimations::MaybeApplyPendingUpdate then publishes sampled animation
    /// declarations at the animation cascade origin. Keep the same ownership
    /// split here: Style owns the computed lists/rules, Animation owns time and
    /// sampling, and Document owns the sampled cascade declarations.
    fn SynchronizeCSSAnimations(&self) -> bool {
        use animation::{CSSAnimationDefinition, CSSAnimationKey, CSSKeyframe};
        let _trace = browser_tracing::span("lifecycle", "Page.SynchronizeCSSAnimations");

        let media_data = crate::style_services::MediaValuesForLayout(
            &self.constraints.borrow(),
            self.preferred_color_scheme.get(),
            self.forced_colors.get(),
        );
        let media = style::media_queries::MediaValuesCached::new(&media_data);
        let definitions =
            {
                let owner = self.document.borrow();
                let document = owner.GetDocument();
                let mut rules = HashMap::<&str, &cssom::CSSKeyframesRule>::new();
                for sheet in document.ActiveStyleSheetsAllScopes() {
                    if !sheet
                        .owner_media_conditions
                        .iter()
                        .all(|query| crate::style_services::ResponsiveMediaMatches(query, &media))
                    {
                        continue;
                    }
                    for rule in &sheet.keyframes {
                        if rule.media_conditions.iter().all(|query| {
                            crate::style_services::ResponsiveMediaMatches(query, &media)
                        }) {
                            // ActiveStyleSheetsAllScopes is in cascade/source order;
                            // the last winning name has the same replacement rule as
                            // StyleEngine::KeyframeStylesForAnimation for ordinary
                            // document-scoped author sheets.
                            rules.insert(rule.name.as_str(), rule);
                        }
                    }
                }

                let mut definitions = Vec::new();
                for node in 0..document.NodeCount() {
                    let Some(resolved) = document.ResolvedStyleFor(node) else {
                        continue;
                    };
                    let style = unsafe { &*resolved.computed_style_owner.Get() };
                    let animations = style.Animations().Get();
                    if animations.is_null() {
                        continue;
                    }
                    let animations = unsafe { &*animations };
                    for (list_index, name) in animations.NameList().iter().enumerate() {
                        let name = name.Get();
                        if name.is_null() {
                            continue;
                        }
                        let name = unsafe { &*name }.GetName().Utf8();
                        let Some(rule) = rules.get(name.as_str()) else {
                            continue;
                        };
                        let mut keyframes = BTreeMap::<u64, Vec<cssom::CSSDeclaration>>::new();
                        for keyframe in &rule.keyframes {
                            let declarations = if keyframe.declarations.is_empty() {
                                style::ParseCSSDeclarationList(&keyframe.declaration_text)
                            } else {
                                keyframe.declarations.clone()
                            };
                            for offset in Self::ParseCSSKeyframeOffsets(&keyframe.key_text) {
                                keyframes
                                    .entry(offset.to_bits())
                                    .or_default()
                                    .extend(declarations.iter().cloned());
                            }
                        }
                        let mut keyframes = keyframes
                            .into_iter()
                            .map(|(offset, declarations)| CSSKeyframe {
                                offset: f64::from_bits(offset),
                                declarations,
                            })
                            .collect::<Vec<_>>();
                        keyframes.sort_by(|left, right| left.offset.total_cmp(&right.offset));
                        if keyframes.is_empty() {
                            continue;
                        }
                        definitions.push(CSSAnimationDefinition {
                            key: CSSAnimationKey {
                                node_id: document.Node(node).Id(),
                                list_index: list_index as u32,
                            },
                            name,
                            timing: Self::CSSAnimationTiming(animations, list_index),
                            keyframes,
                        });
                    }
                }
                definitions
            };
        let output = self
            .animation_engine
            .borrow_mut()
            .ApplyMutation(animation::AnimationMutation::SynchronizeCSSAnimations(
                definitions,
            ))
            .unwrap_or_else(|_| {
                dom::error::invalid_argument("CSS animation synchronization failed")
            });
        // CSSAnimations::MaybeApplyPendingUpdate removes or replaces the cc
        // KeyframeModel before the corresponding source effect resumes main-
        // thread sampling. Multiple effects targeting one property require
        // Blink's group-composition path; until that path is translated, keep
        // them on the document sequence instead of choosing one arbitrarily.
        let candidates = self.animation_engine.borrow().CompositorCandidates();
        let mut eligible =
            BTreeMap::<(animation::CSSAnimationKey, &'static str), Option<(u64, u64)>>::new();
        for candidate in &candidates {
            for property in candidate.curves.iter().map(|curve| match curve {
                animation::CompositorPropertyCurve::Transform(_) => "transform",
                animation::CompositorPropertyCurve::Opacity(_) => "opacity",
            }) {
                eligible
                    .entry((candidate.key, property))
                    .and_modify(|value| *value = None)
                    .or_insert(Some((candidate.effect_id, candidate.generation)));
            }
        }
        let mut replacements = Vec::new();
        {
            let mut removals = self.pending_property_animation_removals.borrow_mut();
            self.compositor_driven_animations
                .borrow_mut()
                .retain(|(key, property), installed| {
                    let current = eligible
                        .get(&(*key, *property))
                        .and_then(|candidate| *candidate);
                    let Some((effect_id, generation)) = current else {
                        removals.insert(installed.identity);
                        return false;
                    };
                    if installed.identity != CompositorAnimationIdentity(effect_id, *property) {
                        removals.insert(installed.identity);
                        return false;
                    }
                    if installed.generation == generation {
                        return true;
                    }
                    let Some(candidate) = candidates.iter().find(|candidate| {
                        candidate.key == *key && candidate.generation == generation
                    }) else {
                        removals.insert(installed.identity);
                        return false;
                    };
                    let binding = paint::CompositedAnimationPropertyBinding {
                        node_id: key.node_id,
                        property: match *property {
                            "transform" => paint::CompositedAnimationProperty::Transform,
                            "opacity" => paint::CompositedAnimationProperty::Opacity,
                            _ => unreachable!("unsupported compositor animation property"),
                        },
                        element_id: foundation::CompositorElementId::new(installed.element_id),
                    };
                    let start_time = candidate.sample_time.map(|sample_time| {
                        self.compositor_executor.TimeTicks(sample_time)
                            - scroll_animation::TimeDelta::Microseconds(
                                (candidate.local_time * 1_000_000.0) as i64,
                            )
                    });
                    let Some((_, mutation)) =
                        CompositorPropertyAnimationMutation(candidate, &binding, start_time)
                    else {
                        removals.insert(installed.identity);
                        return false;
                    };
                    replacements.push(mutation);
                    installed.generation = generation;
                    true
                });
        }
        self.pending_property_animation_replacements
            .borrow_mut()
            .extend(replacements);
        let changed = !output.document.is_empty();
        self.ApplyAnimationOutput(output);
        changed
    }

    /// LayoutImage::Paint owns the observation point while UseCounter remains
    /// document state. Paint publishes the exact LayoutImage resource ids it
    /// visited; Page joins that immutable fact with the live resource-client
    /// relation after a successful paint commit.
    ///
    /// cpp: core/layout/layout_image.cc:302-314
    fn CountPaintUseCounters(&self, artifact: &paint::paint_engine::PaintArtifact) {
        let mut owner = self.document.borrow_mut();
        let document = owner.GetDocumentMut();
        for image_id in artifact.PaintedLayoutImageResourceIds() {
            if document.ImageResourceMaybeAnimated(image_id)
                && document.ImageResourceUsedByMultipleReplacedImages(image_id)
            {
                document.CountUse(dom::WebFeature::kAnimatedImageUsedMoreThanOnce);
            }
        }
    }

    fn WithEngines(
        document: Rc<DocumentEngine>,
        layout_engine: layout::LayoutEngine,
        paint_engine: paint::PaintEngine,
        compositor_executors: (Arc<dyn executor::Executor>, Arc<dyn executor::Executor>),
        constraints: ConstraintSpace,
        client: Rc<RefCell<dyn PageClient>>,
    ) -> Rc<Self> {
        {
            let mut owner = document.borrow_mut();
            let document = owner.GetDocumentMut();
            document.SetIsInitialEmptyDocument(true);
            document.SetDocumentURL("about:blank".into());
            let origin = document.SecurityContext().Origin().clone();
            document.InitializeSecurityContextForNavigation(origin, false);
        }
        Self::WithDocumentEngines(
            document,
            layout_engine,
            paint_engine,
            Some(compositor_executors),
            constraints,
            client,
            None,
            Rc::new(webapi::window_bindings::WindowPerformanceTiming::new()),
        )
    }

    pub(super) fn ForNewDocument(
        previous: &Rc<Self>,
        document: Rc<DocumentEngine>,
        timing: Rc<webapi::window_bindings::WindowPerformanceTiming>,
    ) -> Rc<Self> {
        let mut constraints = previous.constraints.borrow().clone();
        constraints.fonts = previous.platform_fonts.clone();
        constraints.font_queries = Some(font_engine::FontQueryCollector::default());
        Self::WithDocumentEngines(
            document,
            layout::LayoutEngine::new(&crate::CreateLayoutRegistry()),
            paint::PaintEngine::new(),
            None,
            constraints,
            previous.client.clone(),
            Some(previous),
            timing,
        )
    }

    fn WithDocumentEngines(
        document: Rc<DocumentEngine>,
        layout_engine: layout::LayoutEngine,
        paint_engine: paint::PaintEngine,
        compositor_executors: Option<(Arc<dyn executor::Executor>, Arc<dyn executor::Executor>)>,
        mut constraints: ConstraintSpace,
        client: Rc<RefCell<dyn PageClient>>,
        previous: Option<&Rc<Self>>,
        window_performance_timing: Rc<webapi::window_bindings::WindowPerformanceTiming>,
    ) -> Rc<Self> {
        document
            .borrow()
            .GetDocument()
            .InitializeCanvasRuntimeFeatures(
                constraints.canvas_draw_element_enabled,
                foundation::RuntimeEnabledFeatures::ElementCanvasTransformEnabled(),
            );
        document.borrow_mut().GetDocumentMut().BindDOMWindow();
        document.borrow().GetDocument().SetLifecycleActive(true);
        let style_engine = style::StyleEngine::new(&document.borrow());
        let document_fonts = constraints.fonts.clone();
        let platform_fonts = document_fonts.clone();
        let mut font_engine = font_engine::FontEngine::new(
            previous.map_or_else(font_engine::FontContextVersion::default, |previous| {
                previous.system_fonts.borrow().ContextVersion()
            }),
            constraints.font_backend_factory.clone(),
        );
        font_engine.SetDeviceScaleFactor(
            (constraints.device_pixel_ratio / constraints.layout_zoom_factor.max(f64::MIN_POSITIVE))
                as f32,
        );
        font_engine
            .ApplyMutation(font_engine::FontMutation::SetFaces(
                constraints.fonts.iter().cloned().collect(),
            ))
            .expect("initial font catalog mutation");
        font_engine.SetQueryCollector(constraints.font_queries.clone());
        let font_engine = font_engine::FontEngineHandle::new(font_engine);
        constraints.font_engine = Some(font_engine.clone());
        let style_font_metrics = Rc::new(crate::style_services::StyleFontMetricsResolver::new(
            &constraints,
        ));
        let connected_resources = open::ConnectedResourceDiscovery::new(document.Handle());
        let preferred_color_scheme = if constraints.preferred_color_scheme == 1 {
            PreferredColorScheme::kDark
        } else {
            PreferredColorScheme::kLight
        };
        let document_font_inputs = PageDocumentFontInputs::Initial(&constraints);
        let replacing = previous.is_some();
        document_font_inputs.pending.set(replacing);
        // A navigation replaces the document-owned AnimationEngine while the
        // Page-domain compositor intentionally survives. Carry the old
        // KeyframeModel identities into the replacement document's first
        // transaction so animations cannot keep ticking against retired
        // property-tree element ids.
        let mut pending_property_animation_removals = previous
            .map(|previous| {
                previous
                    .pending_property_animation_removals
                    .borrow()
                    .clone()
            })
            .unwrap_or_default();
        if let Some(previous) = previous {
            pending_property_animation_removals.extend(
                previous
                    .compositor_driven_animations
                    .borrow()
                    .values()
                    .map(|installed| installed.identity),
            );
        }
        let system_fonts = previous.map_or_else(
            || {
                crate::font_session::SystemFontSession::with_engine(
                    font_engine.clone(),
                    Box::new(crate::system_font_provider::BrowserSystemFontProvider::default()),
                )
            },
            |previous| {
                previous
                    .system_fonts
                    .borrow()
                    .ForDocument(font_engine.clone())
            },
        );
        let domain = previous
            .map(|previous| previous.domain.clone())
            .unwrap_or_else(|| {
                Rc::new(PageDomainServices {
                    web_view_zoom: Default::default(),
                    native_event_handler_host: Default::default(),
                    native_file_reader: RefCell::new(None),
                    kill_ring: RefCell::new(None),
                    document_input_queue: Default::default(),
                    context_menu_widget_geometry: Cell::new(None),
                    frame_owner_loading_client: Default::default(),
                    local_frames: Default::default(),
                    visual_viewport_identity: foundation::NewUniqueObjectId(),
                    compositor_executor: Arc::new(PageCompositorExecutor::new(
                        compositor_executors
                            .as_ref()
                            .expect("initial Page compositor executors")
                            .0
                            .clone(),
                    )),
                    compositor_raster_executor: Arc::new(PageCompositorExecutor::new(
                        compositor_executors
                            .as_ref()
                            .expect("initial Page compositor executors")
                            .1
                            .clone(),
                    )),
                    raster_engine: compositor_input::NewPageRasterEngine(),
                    external_presentation: Cell::new(false),
                    proxy_main: Default::default(),
                    proxy_main_endpoint: Default::default(),
                    layerize: Default::default(),
                    main_property_tree_features: Default::default(),
                    main_compositor_shut_down: Cell::new(false),
                    pending_main_compositor_visible: Cell::new(None),
                    main_compositor_mutated_outside_lifecycle: Cell::new(false),
                    in_main_compositor_frame: Cell::new(false),
                    main_compositor_effects: RefCell::new(Vec::new()),
                    compositor_task_runner: RefCell::new(CompositorTaskRunner::default()),
                    compositor_input_blocking_task_runner: RefCell::new(None),
                    compositor_pac_sequence: Cell::new(0),
                    compositor_local_surface_id: Cell::new(None),
                    compositor_surface_parameters: Cell::new(None),
                    compositor_page_scale: Cell::new((1.0, 1.0, 1.0)),
                    pushed_paint_artifact: RefCell::new(None),
                    compositor_input_scale: Cell::new(1.0),
                    video_surfaces: RefCell::new(HashMap::new()),
                    video_frame_token: Cell::new(0),
                    outbound_child_frames: RefCell::new(Default::default()),
                    synchronous_presenter: RefCell::new(None),
                    constraints: Rc::new(RefCell::new(constraints)),
                    platform_fonts,
                    client: client.clone(),
                    should_paint_base_background_color: Rc::new(Cell::new(true)),
                    base_background_color: Rc::new(Cell::new(Color {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    })),
                    should_clear_document_background: Rc::new(Cell::new(true)),
                    event_timing_widget: Default::default(),
                    external_compositor_clock: RefCell::new(None),
                    performance_visible: Cell::new(true),
                    performance_visibility_changed_at: Cell::new(None),
                    has_begin_frame_source: Rc::new(Cell::new(false)),
                    begin_frame_source: RefCell::new(None),
                    preferred_color_scheme: Rc::new(Cell::new(preferred_color_scheme)),
                    forced_colors: Rc::new(Cell::new(ForcedColors::kNone)),
                    page_focused: Cell::new(true),
                })
            });
        let state = Rc::new(Self {
            domain,
            validation_message: Default::default(),
            validation_message_visibility: Default::default(),
            directory_drop_requests: RefCell::new(HashMap::new()),
            popup_tasks: Default::default(),
            pending_form_submissions: Default::default(),
            popup_interaction: Default::default(),
            popup_source_interaction: Default::default(),
            document,
            style_engine: RefCell::new(style_engine),
            style_font_metrics,
            layout_engine: RefCell::new(layout_engine),
            paint_engine: RefCell::new(paint_engine),
            document_fonts: RefCell::new(document_fonts),
            document_font_inputs,
            system_fonts: RefCell::new(system_fonts),
            interaction: Rc::new(RefCell::new(UserInteractionState::default())),
            layout_editing: Rc::new(Default::default()),
            resident_frame_selection: RefCell::new(Weak::new()),
            // Editing behavior and the UI locale are Page/embedder settings,
            // not Document state. Preserve the live values when navigation
            // installs a replacement PageState.
            interaction_text_settings: RefCell::new(
                previous.and_then(|previous| previous.interaction_text_settings.borrow().clone()),
            ),
            scrollbar_scroll_runtime: RefCell::new(Weak::new()),
            blocking_wheel_registry_dirty: Cell::new(false),
            blocking_wheel_targets: RefCell::new(None),
            blocking_wheel_invalidation_bindings: Cell::new(0),
            mouse_autoscroll: RefCell::new(Weak::new()),
            autoscroll_cursor_overridden: Cell::new(false),
            document_loader_owners: Default::default(),
            bindings: RefCell::new(Weak::new()),
            resources: RefCell::new(None),
            connected_resources,
            lazy_image_nodes: RefCell::new(HashSet::new()),
            image_events: RefCell::new(Vec::new()),
            pending_image_events: RefCell::new(HashMap::new()),
            style_sheet_events: RefCell::new(Vec::new()),
            pending_style_sheet_events: RefCell::new(HashMap::new()),
            selectionchange_event_targets: RefCell::new(Vec::new()),
            pending_text_overflow_layout_invalidations: RefCell::new(Vec::new()),
            scroll_event_targets: RefCell::new(Vec::new()),
            scroll_finished_hover_dirty: Cell::new(false),
            dirty: Cell::new(true),
            scroll_only: Cell::new(false),
            styles_resolved: Cell::new(false),
            resource_references_dirty: Cell::new(ResourceReferenceDirty::ALL),
            discovered_style_image_revision: Cell::new(0),
            paint_resources_dirty: Cell::new(false),
            video_frame_resources: RefCell::new(HashMap::new()),
            pending_paint_mutations: RefCell::new(Vec::new()),
            layout_revision: Cell::new(None),
            #[cfg(test)]
            full_layout_lifecycles: Cell::new(0),
            scroll_updates: RefCell::new(Vec::new()),
            compositor_scroll_updates: Cell::new(false),
            animation_engine: RefCell::new(animation::AnimationEngine::new()),
            compositor_driven_animations: RefCell::new(BTreeMap::new()),
            pending_property_animation_replacements: RefCell::new(Vec::new()),
            pending_property_animation_removals: RefCell::new(pending_property_animation_removals),
            animation_begin_frame_requested: Cell::new(false),
            window_performance_timing,
            programmatic_scroll_animators: Default::default(),
        });
        state.InstallProgrammaticScroll();
        if !replacing {
            let domain = local_frame::LocalFrameDomain::new(Rc::downgrade(&state));
            let frame_client: Rc<dyn dom::frame_owner_loading::FrameOwnerLoadingClient> =
                Rc::new(frame_owner_loading::PageFrameOwnerLoadingClient {
                    domain: Rc::downgrade(&domain),
                    parent: domain.root,
                });
            assert!(state.local_frames.set(domain).is_ok());
            assert!(state.frame_owner_loading_client.set(frame_client).is_ok());
        }
        state.BindWindowEventTiming(
            state.local_frames.get().unwrap().root,
            &state.document,
            &state.window_performance_timing,
        );
        dom::frame_owner_loading::BindFrameOwnerLoadingClient(
            &state.document.Handle(),
            state.frame_owner_loading_client.get().unwrap(),
        );
        state.InstallPopupTaskServices();
        state
    }

    #[cfg(test)]
    fn new(constraints: ConstraintSpace, client: Rc<RefCell<dyn PageClient>>) -> Rc<Self> {
        Self::WithEngines(
            Rc::new(DocumentEngine::new()),
            layout::LayoutEngine::new(&crate::CreateLayoutRegistry()),
            paint::PaintEngine::new(),
            (executor::inline(), executor::inline()),
            constraints,
            client,
        )
    }

    /// Route Layout's source ScrollRectToVisible result back through the Page
    /// owners. Layout has already mutated its resident ScrollableAreas; DOM
    /// receives the final offsets and smooth requests retain the existing
    /// programmatic animator owner.
    fn ApplyNativeScrollResult(
        &self,
        result: layout::scroll::NativeScrollResult,
    ) {
        let did_scroll = result.did_scroll;
        for update in result.offsets {
            if let Some(effect) = self
                .document
                .ApplyScrollOffset(update.node_id, update.offset)
            {
                let mut targets = self.scroll_event_targets.borrow_mut();
                if !targets.contains(&effect.event_target_node_id) {
                    targets.push(effect.event_target_node_id);
                }
            }
        }
        for request in result.programmatic_requests {
            let behavior = if request.behavior
                == layout_style::style::scroll_enums::mojom::blink::ScrollBehavior::kSmooth
            {
                ViewportScrollBehavior::Smooth
            } else {
                ViewportScrollBehavior::Instant
            };
            self.SetProgrammaticScrollOffset(
                request.node_id,
                Offset {
                    x: request.offset.x() as f64,
                    y: request.offset.y() as f64,
                },
                behavior,
                request.source_type,
            );
        }
        if did_scroll {
            self.document.borrow_mut().InvalidatePaint();
            self.dirty.set(true);
        }
    }

    /// SelectionEditor posts this invalidation to the internal-default task
    /// runner so the synchronous selection/hit-test transaction has fully
    /// unwound before LayoutNG is dirtied. Page owns only task routing; the
    /// source invalidation walk remains in Layout.
    fn RunTextOverflowLayoutInvalidationTasks(&self) {
        let pending =
            std::mem::take(&mut *self.pending_text_overflow_layout_invalidations.borrow_mut());
        if pending.is_empty() {
            return;
        }
        let document_identity = self.document.borrow().GetDocument().RootHandle().Identity();
        let invalidated = {
            let mut layout = self.layout_engine.borrow_mut();
            pending.into_iter().fold(false, |changed, pending| {
                (pending.document_identity == document_identity
                    && layout.ApplyMutation(layout::mutation::LayoutMutation::Selection(
                        layout::views::selection::SelectionMutation::InvalidateTextOverflowLayoutForNode {
                            node_id: pending.node_id,
                        },
                    )))
                    || changed
            })
        };
        if invalidated {
            self.InvalidateLayoutOutput();
            self.scroll_only.set(false);
            self.dirty.set(true);
        }
    }

    fn InvalidateLayoutOutput(&self) {
        self.layout_revision.set(None);
        self.DocumentLifecycle().ScheduleLayout();
    }
    fn MarkLayoutCurrent(&self) {
        self.layout_revision.set(Some(
            self.document.borrow().GetMeasurementGeometryRevision(),
        ));
    }

    /// LocalFrameView::GetPaintArtifact (local_frame_view.cc:4399-4406): the
    /// current artifact is read from its one owner, PaintEngine's persistent
    /// PaintController data (paint_controller.h:96-99). Callers borrow it;
    /// only the Layerize push shares the `Arc`, as PaintArtifactCompositor
    /// retains the artifact it was updated with.
    fn CurrentPaintArtifact(
        &self,
    ) -> Option<std::cell::Ref<'_, Arc<paint::paint_engine::PaintArtifact>>> {
        std::cell::Ref::filter_map(self.paint_engine.borrow(), |paint| paint.GetPaintResult()).ok()
    }

    /// Resource-only update of the PaintController-owned artifact, in place
    /// (ImageResourceContent observers invalidate only the sampling records).
    fn PublishPaintResources(&self) {
        let pending = std::mem::take(&mut *self.pending_paint_mutations.borrow_mut());
        let update = if pending.is_empty() {
            let constraints = self.constraints.borrow();
            paint::paint_engine::PaintResourceUpdate::Snapshot(Arc::new(PaintResources {
                fonts: constraints.fonts.clone(),
                images: constraints.images.clone(),
                canvases: constraints.canvases.clone(),
                device_pixel_ratio: constraints.device_pixel_ratio,
                preferred_color_scheme: constraints.preferred_color_scheme,
                layout_zoom_factor: constraints.layout_zoom_factor,
                base_background_color: self.base_background_color.get(),
                paints_opaque_base_background: self.should_paint_base_background_color.get(),
                should_clear_document_background: self.should_clear_document_background.get(),
                viewport: constraints.viewport,
            }))
        } else {
            paint::paint_engine::PaintResourceUpdate::Mutations(pending)
        };
        if !self.paint_engine.borrow_mut().PublishResourceUpdate(update) {
            dom::error::logic_error("paint resource update rejected");
        }
        self.paint_resources_dirty.set(false);
    }
    fn InvalidateStyle(&self) {
        self.InvalidateLayoutOutput();
        self.InvalidateStylePreservingGeometry();
    }
    fn InvalidateStylePreservingGeometry(&self) {
        self.InvalidateStylePreservingGeometryWithResources(ResourceReferenceDirty::ALL);
    }
    fn InvalidateStylePreservingGeometryWithResources(&self, resources: ResourceReferenceDirty) {
        self.DocumentLifecycle().ScheduleVisualUpdate();
        self.styles_resolved.set(false);
        self.MarkResourceReferencesDirty(resources);
        self.scroll_only.set(false);
        self.dirty.set(true);
    }
    fn MarkResourceReferencesDirty(&self, resources: ResourceReferenceDirty) {
        let mut dirty = self.resource_references_dirty.get();
        dirty.Insert(resources);
        self.resource_references_dirty.set(dirty);
    }
    // cpp: browser/browser.cc:947-981
    fn TryApplyDOMMutation(
        &self,
        m: &dom::dom_mutation::DOMMutation,
        dispatch_image: &mut dyn FnMut(u64, bool),
    ) -> Result<(), dom::dom_mutation::DOMMutationError> {
        if self.ApplyPopupMutationWithoutScript(m)? {
            self.DrainPopupLifecycleWithoutScript();
            return Ok(());
        }
        let mut mutation_trace = browser_tracing::span("dom", "Page.ApplyDOMMutation");
        mutation_trace.set("mutation_type", m.mutation_type as u8 as f64);
        mutation_trace.set("node_id", m.target_node_id as f64);
        let result = {
            let _trace = browser_tracing::span("dom", "Page.DocumentApplyDOMMutation");
            self.document.ApplyDOMMutation(m)
        };
        {
            let _trace = browser_tracing::span("dom", "Page.ProcessRenderBlockingMutations");
            self.connected_resources.ProcessRenderBlockingMutations();
        }
        // Canonical CharacterData/Element removal supplements run even when
        // a throwing compound mutation already changed part of the DOM.
        let effect = result?;
        {
            let _trace = browser_tracing::span("dom", "Page.DrainAttributeLifecycle");
            dom::attribute_lifecycle::DrainAttributeLifecycle(&self.document.Handle());
        }
        // InlineCSSStyleDeclaration::DidMutate -> Element::
        // InvalidateStyleAttribute (inline_css_style_declaration.cc:36-55,
        // element.cc:13195-13203) only marks the element for a local style
        // recalc; Document also classifies it as reaching no ParseAttribute
        // resource hook. The declaration's images are requested by that
        // element's style resolution, so no document-wide walk is admitted.
        let resources = if m.mutation_type == DOMMutationType::kSetAttribute
            && m.inline_style.is_some()
        {
            ResourceReferenceDirty::STYLE_ONLY
        } else {
            ResourceReferenceDirty::ALL
        };
        self.ApplyDocumentLifecycleEffectsWithResources(&effect, resources);
        self.DrainPopupLifecycleWithoutScript();
        // A changed image request cancels its queued event, even if src later
        // changes back to the same URL before the event task runs.
        for request in &effect.resource {
            let Some(resources) = self.resources.borrow().as_ref().cloned() else {
                continue;
            };
            if let ResourceRequest::LoadDeferredImage { node_id } = request {
                resources.LoadDeferredHTMLImage(*node_id);
                continue;
            }
            let node_id = match request {
                ResourceRequest::ResetImage { node_id }
                | ResourceRequest::ImageReferrerPolicyChanged { node_id, .. } => *node_id,
                _ => continue,
            };
            let Some(index) = ({ self.document.borrow().GetDocument().FindNodeById(node_id) })
            else {
                continue;
            };
            if let ResourceRequest::ImageReferrerPolicyChanged { previous, .. } = request {
                if !resources.ImageReferrerPolicyChangeRequiresUpdate(index, previous.as_deref()) {
                    continue;
                }
            }
            self.CancelImageEvent(node_id);
            resources
                .UpdateHTMLImageLoader(
                    index,
                    dom::persistent_document::ImageLoaderUpdate::IgnorePreviousError,
                    false,
                )
                .expect("image attribute update");
        }
        let Some(connected) = effect
            .resource
            .into_iter()
            .find_map(|request| match request {
                ResourceRequest::DiscoverConnectedSubtree(request) => Some(request),
                ResourceRequest::ResetImage { .. }
                | ResourceRequest::ImageReferrerPolicyChanged { .. }
                | ResourceRequest::LoadDeferredImage { .. } => None,
            })
        else {
            return Ok(());
        };
        let node = self
            .document
            .borrow()
            .GetDocument()
            .FindNodeById(connected.root_node_id);
        if let Some(i) = node {
            // Attribute parsing belongs to this element. Descendant style
            // resource discovery still runs through the invalidated StyleState;
            // only insertion/content replacement prepares a connected subtree.
            let result = if m.mutation_type == DOMMutationType::kSetAttribute {
                // HTMLImageElement::ParseAttribute only updates the image
                // request for source/request attributes. Styling an already
                // loaded image must not post another load event.
                self.connected_resources.PrepareNode(
                    i,
                    connected.prepare_scripts,
                    connected.image_request,
                    dispatch_image,
                )
            } else {
                self.connected_resources.PrepareSubtree(
                    i,
                    connected.prepare_scripts,
                    dispatch_image,
                )
            };
            let resource_effects = result.unwrap_or_else(|e| std::panic::panic_any(e));
            if resource_effects.style_sheet_changed {
                self.InvalidateStylePreservingGeometry();
            }
        }
        // Preparation invalidates newly parsed sheets itself. Image completion
        // registers a later DOM task; its listener writes use this same path.
        Ok(())
    }

    fn ApplyDOMMutation(
        &self,
        m: &dom::dom_mutation::DOMMutation,
        dispatch_image: &mut dyn FnMut(u64, bool),
    ) {
        self.TryApplyDOMMutation(m, dispatch_image)
            .unwrap_or_else(|error| std::panic::panic_any(error));
    }

    fn ApplyDocumentLifecycleEffectsWithResources(
        &self,
        effects: &DocumentEffects,
        resources: ResourceReferenceDirty,
    ) {
        // Blink leaves detailed dirty nodes on Document. The effect only admits
        // the relevant lifecycle engines. Attribute changes keep committed
        // geometry until computed-style diffing proves layout changed.
        if effects.layout == Some(DocumentLayoutMutation::InvalidateGeometry) {
            self.InvalidateLayoutOutput();
        }
        if effects.style.is_some() {
            self.InvalidateStylePreservingGeometryWithResources(resources);
        }
    }
    fn ApplyAnimationOutput(&self, output: animation::AnimationOutput) {
        for mutation in output.document {
            let effects = self.document.ApplyAnimationMutation(mutation);
            // Animation/transition samples are element style changes
            // (Element::SetNeedsAnimationStyleRecalc); they connect no
            // stylesheet owner and change no element image request.
            self.ApplyDocumentLifecycleEffectsWithResources(
                &effects,
                ResourceReferenceDirty::STYLE_ONLY,
            );
        }
        for mutation in output.paint {
            self.paint_engine
                .borrow_mut()
                .ApplyAnimationMutation(mutation);
        }
        for effect in output.effects {
            match effect {
                animation::AnimationEffect::RequestBeginFrame => {
                    self.animation_begin_frame_requested.set(true)
                }
            }
        }
    }
    fn ApplyAnimationMutation(&self, mutation: animation::AnimationMutation) {
        let begin_frame_sample =
            matches!(&mutation, animation::AnimationMutation::BeginFrame { .. });
        let output = self
            .animation_engine
            .borrow_mut()
            .ApplyMutation(mutation)
            .unwrap_or_else(|_| {
                dom::error::invalid_argument("animation time must be finite and frame ordered")
            });
        if begin_frame_sample {
            let mut trace = browser_tracing::span("animation", "Animation.SampleCSSAnimations");
            trace.set("document_mutations", output.document.len() as f64);
            trace.set("paint_mutations", output.paint.len() as f64);
            trace.set("requested_begin_frame", output.effects.len() as f64);
            if browser_tracing::enabled() {
                let compositor_driven = self.compositor_driven_animations.borrow();
                for mutation in &output.document {
                    if let animation::DocumentMutation::ApplyStyleBatch(samples) = mutation {
                        for sample in samples {
                            trace.set("sample_node_id", sample.node_id as f64);
                            trace.set("sample_effect_id", sample.effect_id as f64);
                            trace.set("sample_declarations", sample.declarations.len() as f64);
                            trace.set(
                                "sample_compositor_transform",
                                compositor_driven.keys().any(|(key, property)| {
                                    key.node_id == sample.node_id && *property == "transform"
                                }) as u8 as f64,
                            );
                            trace.set(
                                "sample_compositor_opacity",
                                compositor_driven.keys().any(|(key, property)| {
                                    key.node_id == sample.node_id && *property == "opacity"
                                }) as u8 as f64,
                            );
                            for declaration in &sample.declarations {
                                if declaration.property == "opacity" {
                                    if let Ok(value) = declaration.value.parse::<f64>() {
                                        trace.set("sample_opacity", value);
                                    }
                                } else if declaration.property == "transform" {
                                    let number = declaration
                                        .value
                                        .split(|character: char| {
                                            !(character.is_ascii_digit()
                                                || matches!(character, '.' | '-' | '+' | 'e' | 'E'))
                                        })
                                        .find_map(|value| value.parse::<f64>().ok());
                                    if let Some(value) = number {
                                        trace.set("sample_transform_first_number", value);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        self.ApplyAnimationOutput(output);
    }
    fn ApplyInteractionOutput(&self, output: ::interaction::InteractionOutput) {
        match output {
            ::interaction::InteractionOutput::Effect(
                ::interaction::InteractionEffect::MouseClickActivation { target_node_id },
            ) => {
                let document = self.document.borrow().GetDocument().RootHandle().Identity();
                self.client
                    .borrow_mut()
                    .DidDispatchClick(document, target_node_id);
            }
            ::interaction::InteractionOutput::Effect(
                ::interaction::InteractionEffect::SignalCloseWatchers,
            ) => {
                let domain = self.local_frames.get().expect("Page LocalFrame domain");
                domain.WithInputListeners(domain.root, |engine, document| {
                    engine
                        .SignalCloseWatchers(document)
                        .unwrap_or_else(|error| std::panic::panic_any(error))
                });
            }
            ::interaction::InteractionOutput::Effect(
                ::interaction::InteractionEffect::NotifyHistoryUserActivation(now),
            ) => {
                let domain = self.local_frames.get().expect("Page LocalFrame domain");
                domain.NotifyHistoryUserActivation(domain.root, now);
            }
            ::interaction::InteractionOutput::Effect(
                ::interaction::InteractionEffect::ConsumeHistoryUserActivation,
            ) => {
                self.local_frames
                    .get()
                    .expect("Page LocalFrame domain")
                    .ConsumeHistoryUserActivation();
            }
            ::interaction::InteractionOutput::Effect(
                ::interaction::InteractionEffect::CaptureMouseEventsToWidget(capture),
            ) => {
                self.client.borrow_mut().CaptureMouseEventsToWidget(capture);
            }
            ::interaction::InteractionOutput::DocumentMutation(mutation) => {
                self.ApplyMutation(PageMutation::DOMMutation(mutation));
            }
            ::interaction::InteractionOutput::StateMutation(mutation) => {
                self.ApplyMutation(PageMutation::InteractionStateMutation(mutation));
            }
            ::interaction::InteractionOutput::Effect(
                ::interaction::InteractionEffect::PrepareFormSubmission(submission),
            ) => {
                self.pending_form_submissions
                    .borrow_mut()
                    .push_back(submission);
            }
            ::interaction::InteractionOutput::Effect(
                ::interaction::InteractionEffect::FormSubmissionError(error),
            ) => {
                // Removal/document replacement during synchronous listeners
                // abandons this source transaction without navigating.
                if !matches!(
                    error,
                    interaction::input_type::FormDataAdmission::StaleSubmission { .. }
                ) {
                    self.client
                        .borrow_mut()
                        .DidFail(&format!("form submission: {error:?}"));
                }
            }
            ::interaction::InteractionOutput::Effect(
                ::interaction::InteractionEffect::SubmitForm {
                    parameters,
                    entries,
                },
            ) => {
                let base = self
                    .resources
                    .borrow()
                    .as_ref()
                    .map(|resources| resources.BaseURL())
                    .unwrap_or_default();
                match crate::form_submission::BuildRequest(&base, &parameters, &entries) {
                    Ok(Some(request)) => self.local_frames.get().unwrap().RouteTargetNavigation(
                        self.local_frames.get().unwrap().root,
                        request,
                        parameters.scheduling.clone(),
                    ),
                    Err(error) => self.client.borrow_mut().DidFail(&error.to_string()),
                    Ok(None) => {}
                }
            }
        }
    }
    /// Page owns lifecycle preparation; Interaction owns all entry decisions
    /// and synchronous formdata dispatch. Resume's Layout borrow ends before
    /// Finish can enter any author listener.
    fn CompleteFormDataConstruction(
        &self,
        interaction: &interaction::Interaction<'_>,
        document: &interaction::InteractionDocument,
        token: interaction::input_type::FormSubmissionToken,
    ) -> Result<
        Rc<interaction::dispatch::core::fileapi::form_data_object::FormDataObject>,
        interaction::dispatch::core::fileapi::form_data_object::FormDataConstructionError,
    > {
        use interaction::{
            dispatch::core::fileapi::form_data_object::FormDataConstructionError as E,
            input_type::PrepareFormEntryListResult,
        };
        if !Rc::ptr_eq(document, &self.document.Handle()) {
            return Err(E::InvalidState("The form belongs to a different Document."));
        }
        self.EnsureLayoutOutput();
        let prepared = {
            let layout = self.layout_engine.borrow();
            let output = layout
                .Output()
                .ok_or(E::InvalidState("FormData requires current form layout."))?;
            interaction
                .ResumeFormSubmission(
                    document,
                    token,
                    interaction::adapter::page_editing_context::PreparedEditingGeometry::new(
                        output,
                    ),
                )
                .map_err(interaction::input_type::FormDataConstructionErrorFor)?
        };
        // The resident Layout borrow above is gone before author formdata
        // listeners run, in this same constructor binding invocation.
        match prepared {
            Some(PrepareFormEntryListResult::Ready(pending)) => {
                interaction.FinishFormData(document, pending)
            }
            Some(PrepareFormEntryListResult::NeedsLifecycle(_)) => Err(E::InvalidState(
                "FormData layout preparation did not complete.",
            )),
            None => Err(E::InvalidState("The form is constructing entry list.")),
        }
    }
    fn CompleteFormSubmissions(
        &self,
        interaction: &interaction::Interaction<'_>,
        document: &interaction::InteractionDocument,
    ) {
        loop {
            let Some(submission) = self.pending_form_submissions.borrow_mut().pop_front() else {
                break;
            };
            self.EnsureLayoutOutput();
            let prepared = {
                let layout = self.layout_engine.borrow();
                layout.Output().map(|output| {
                    interaction.ResumeFormSubmission(
                        document,
                        submission.token,
                        interaction::adapter::page_editing_context::PreparedEditingGeometry::new(
                            output,
                        ),
                    )
                })
            };
            match prepared {
                Some(Ok(Some(interaction::input_type::PrepareFormEntryListResult::Ready(
                    pending,
                )))) => {
                    interaction.FinishFormSubmission(document, pending, submission.parameters);
                }
                Some(Ok(None)) => {}
                Some(Err(error)) => {
                    self.ApplyInteractionOutput(interaction::InteractionOutput::Effect(
                        interaction::InteractionEffect::FormSubmissionError(error),
                    ))
                }
                Some(Ok(Some(
                    interaction::input_type::PrepareFormEntryListResult::NeedsLifecycle(_),
                )))
                | None => {
                    self.client
                        .borrow_mut()
                        .DidFail("form submission has no prepared layout output");
                }
            }
        }
    }
    fn QueueImageEvent(&self, id: u64, loaded: bool) {
        let ticket = {
            let owner = self.document.borrow();
            let d = owner.GetDocument();
            let Some(index) = d.FindNodeById(id) else {
                return;
            };
            let Some(loader) = d.ExistingHTMLImageLoader(index) else {
                return;
            };
            let Some(ticket) = loader.TakeEventTicket() else {
                return;
            };
            ticket
        };
        self.CancelImageEvent(id);
        let active = Rc::new(Cell::new(true));
        self.pending_image_events
            .borrow_mut()
            .insert(id, active.clone());
        self.image_events.borrow_mut().push(PendingImageEvent {
            id,
            ticket,
            loaded,
            active,
        });
    }
    fn CancelImageEvent(&self, id: u64) {
        if let Some(active) = self.pending_image_events.borrow_mut().remove(&id) {
            active.set(false);
        }
    }
    fn CancelImageEvents(&self) {
        for (_, active) in self.pending_image_events.borrow_mut().drain() {
            active.set(false);
        }
        self.image_events.borrow_mut().clear();
    }
    fn QueueStyleSheetEvent(&self, id: u64, signature: String, loaded: bool) {
        let is_link = {
            let owner = self.document.borrow();
            let tree = owner.GetDocument();
            tree.FindNodeById(id)
                .is_some_and(|i| tree.Node(i).IsHTMLElement("link"))
        };
        if !is_link {
            return;
        }
        self.CancelStyleSheetEvent(id);
        let active = Rc::new(Cell::new(true));
        self.pending_style_sheet_events
            .borrow_mut()
            .insert(id, active.clone());
        self.style_sheet_events
            .borrow_mut()
            .push(PendingStyleSheetEvent {
                id,
                signature,
                loaded,
                active,
            });
    }
    fn CancelStyleSheetEvent(&self, id: u64) {
        if let Some(active) = self.pending_style_sheet_events.borrow_mut().remove(&id) {
            active.set(false);
        }
    }
    fn CancelStyleSheetEvents(&self) {
        for (_, active) in self.pending_style_sheet_events.borrow_mut().drain() {
            active.set(false);
        }
        self.style_sheet_events.borrow_mut().clear();
    }
    fn ConnectResources(self: &Rc<Self>, resources: Rc<ResourceFetcher>) {
        // Commit into the parser's supplied arena. That arena owns its CSSOM
        // collection, just as Blink's Document owns its StyleEngine collection.
        let state = Rc::downgrade(self);
        resources.SetStyleSheetReceiver(Rc::new(move |document, sheet| {
            let effects = DocumentEngine::AppendStyleSheetTo(document, sheet);
            if let Some(state) = state.upgrade() {
                // A newly connected sheet can change both geometry and the set
                // of referenced @font-face/image resources. CSSOM owns rule
                // invalidation; Page owns lifecycle and resource discovery.
                // The parser lends this exact Document under its existing
                // mutable arena borrow. Schedule against that owner directly;
                // re-borrowing PageState::document here would violate the
                // same-document handoff contract.
                if effects.layout == Some(DocumentLayoutMutation::InvalidateGeometry) {
                    state.layout_revision.set(None);
                    document.Lifecycle().ScheduleLayout();
                }
                if effects.style.is_some() {
                    document.Lifecycle().ScheduleVisualUpdate();
                    state.styles_resolved.set(false);
                    state.MarkResourceReferencesDirty(ResourceReferenceDirty::ALL);
                    state.scroll_only.set(false);
                    state.dirty.set(true);
                }
            }
        }));
        let lazy_state = Rc::downgrade(self);
        resources.SetHTMLImageLazyMonitoring(Rc::new(move |node, monitored| {
            let Some(state) = lazy_state.upgrade() else {
                return;
            };
            if monitored {
                state.lazy_image_nodes.borrow_mut().insert(node);
            } else {
                state.lazy_image_nodes.borrow_mut().remove(&node);
            }
        }));
        // Source: ImageLoader::UpdateLayoutObject and
        // LayoutImage::ImageChanged. ImageResourceContent completion updates
        // the resident LayoutImage which observes that exact content owner;
        // rebuilding DOM/style inputs alone cannot replace this notification
        // because LayoutImageResource intentionally preserves native object
        // identity across lifecycle updates.
        let image_layout_state = Rc::downgrade(self);
        resources.SetHTMLImageLoaderLayoutUpdate(Rc::new(move |node, update| {
            let Some(state) = image_layout_state.upgrade() else {
                return;
            };
            let Some(result) = state
                .layout_engine
                .borrow_mut()
                .Resources()
                .UpdateHTMLImageResource(node, update)
            else {
                return;
            };
            if result.needs_layout {
                state.InvalidateLayoutOutput();
            } else {
                state.DocumentLifecycle().ScheduleVisualUpdate();
            }
            state.scroll_only.set(false);
            state.dirty.set(true);
        }));
        self.style_engine
            .borrow()
            .CSSOMSource()
            .BindReferrerPolicy(resources.ReferrerPolicyReadHandle());
        self.connected_resources.SetResources(resources.clone());
        *self.resources.borrow_mut() = Some(resources);
    }
    fn CreateWebApiDOMHost(self: &Rc<Self>) -> DOMBindingsHost {
        let state = self;
        let style = state.clone();
        let metric = state.clone();
        let geometry = state.clone();
        let scroll = state.clone();
        let viewport_scroll = state.clone();
        let viewport_scroll_write = state.clone();
        let stylesheet = state.clone();
        let constructed_stylesheet = state.clone();
        let animation = state.clone();
        let fonts = state.clone();
        let focus_domain = Rc::downgrade(state.local_frames.get().unwrap());
        let focus_frame = state.local_frames.get().unwrap().root;
        let host = DOMBindingsHost {
            document_editor_command: None,
            execute_document_editor_command: None,
            selection: None,
            text_control_selection: None,
            resolve_event_target: None,
            convert_window_view: Some(local_frame_window::WindowViewConverter(Rc::downgrade(
                state.local_frames.get().unwrap(),
            ))),
            project_window_view: Some(local_frame_window::WindowViewProjector(Rc::downgrade(
                state.local_frames.get().unwrap(),
            ))),
            read_layout_zoom: Some({
                let state = Rc::downgrade(state);
                Rc::new(move || {
                    state.upgrade().map_or(1., |state| {
                        state.constraints.borrow().layout_zoom_factor as f32
                    })
                })
            }),
            retain_drag_image: Some(Rc::new(drag_image::RetainImageResource)),
            read_document_focus: Some(Rc::new(move |identity| {
                focus_domain
                    .upgrade()?
                    .DocumentFocusState(focus_frame, identity)
            })),
            validate_form: None,
            read_frame: Some(local_frame_window::FrameReader(
                Rc::downgrade(state.local_frames.get().unwrap()),
                state.document.clone(),
            )),
            event_clock: Some(self.InteractionEventClock()),
            cssom_document: self.resources.borrow().as_ref().map(|resources| {
                (
                    self.style_engine.borrow().CSSOMSource(),
                    resources.ReferrerPolicyReadHandle(),
                )
            }),
            read_font_loading: Some(Box::new(move || {
                fonts
                    .resources
                    .borrow()
                    .as_ref()
                    .is_some_and(|resources| resources.HasPendingFonts())
            })),
            write_viewport_scroll: Some(Box::new(move |target_node_id, offset, behavior| {
                viewport_scroll_write.SetProgrammaticScrollOffset(
                    target_node_id,
                    offset,
                    behavior,
                    compositor_values::scroll_source_type::ScrollSourceType::kAbsoluteScroll,
                );
            })),
            write_scroll: Some(Box::new(move |target_node_id, offset| {
                scroll.ProgrammaticAnimations().CancelAll();
                scroll.ApplyMutation(PageMutation::ScrollMutation(
                    page_mutation::ScrollMutation {
                        target_node_id,
                        offset,
                        scroll_type: layout_style::style::scroll_enums::mojom::blink::ScrollType::kProgrammatic,
                    },
                ));
            })),
            // cpp: browser/browser.cc:634-637
            sample_animation: Some(Box::new(move |node_id, effect_id, time, declarations| {
                animation.ApplyAnimationMutation(animation::AnimationMutation::Tick(
                    animation::AnimationTick {
                        monotonic_time: time / 1000.0,
                        samples: vec![animation::AnimationStyleSample {
                            node_id,
                            effect_id,
                            declarations,
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                ));
            })),
            update_style: Some(Box::new(move |id| {
                style.ResolveStyles();
                let mut document = style.document.borrow_mut();
                let Some(node) = document.GetDocument().FindNodeById(id) else {
                    return;
                };
                crate::style_services::EnsureComputedStyleForNode(
                    &mut style.style_engine.borrow_mut(),
                    &mut document,
                    &style.constraints.borrow(),
                    style.preferred_color_scheme.get(),
                    style.forced_colors.get(),
                    node,
                );
            })),
            emit_style_sheet: Some(Box::new(move |sheet| {
                let base = stylesheet
                    .resources
                    .borrow()
                    .as_ref()
                    .expect("CSSOM before navigation")
                    .BaseURL();
                stylesheet.ApplyMutation(PageMutation::CSSOMMutation(
                    page_mutation::CSSOMMutation {
                        style_sheet: sheet,
                        base_url: base,
                    },
                ));
            })),
            invalidate_style: Some(Box::new(move || {
                constructed_stylesheet.InvalidateStylePreservingGeometry();
            })),
            read_viewport_scroll: Some(Box::new(move || {
                // Assembly only: Layout supplies its viewport range, DOM owns
                // the latest offset, and canonical Style supplies scroll policy.
                viewport_scroll.EnsureLayoutOutput();
                let layout = viewport_scroll.layout_engine.borrow();
                let range = layout.Output()?.Root().ScrollRange();
                let owner = viewport_scroll.document.borrow();
                let tree = owner.GetDocument();
                let root = tree
                    .Node(tree.Root())
                    .Children()
                    .iter()
                    .copied()
                    .find(|&index| tree.Node(index).IsHTMLElement("html"))?;
                let native = tree
                    .ResolvedStyleFor(root)
                    .map(|resolved| unsafe { &*resolved.computed_style_owner.Get() })?;
                Some(ViewportScrollState {
                    target_node_id: tree.Node(root).Id(),
                    offset: tree.ScrollOffsetFor(root),
                    minimum: range.minimum,
                    maximum: range.maximum,
                    layout_zoom_factor: viewport_scroll.constraints.borrow().layout_zoom_factor,
                    smooth_behavior: native.GetScrollBehavior().value() == 2,
                    snap: !native.GetScrollSnapType().is_none,
                })
            })),
            read_metric: Some(Box::new(move |id, name| {
                if !metric.EnsureLayoutOutputForNode(id) {
                    return None;
                }
                let query = layout::output::LayoutMetric::try_from(name).ok()?;
                let layout = metric.layout_engine.borrow();
                let output = layout.Output()?;
                output.Node(id).map_or(Some(0.0), |node| node.Metric(query))
            })),
            dispatch_synthetic: None,
            submit_form: None,
            read_geometry: Some(Box::new(move |id| {
                if !geometry.EnsureLayoutOutputForNode(id) {
                    return Vec::new();
                }
                let frame_layout_zoom = geometry.constraints.borrow().layout_zoom_factor as f32;
                let standardized_browser_zoom = geometry
                    .document
                    .borrow()
                    .GetDocument()
                    .StandardizedBrowserZoomEnabled();
                geometry
                    .layout_engine
                    .borrow()
                    .Output()
                    .and_then(|output| output.Node(id))
                    .map_or_else(Vec::new, |node| {
                        node.HTMLClientRectsForDocument(
                            frame_layout_zoom,
                            standardized_browser_zoom,
                        )
                        .into_iter()
                        .map(|rect| paint::paint_engine::PaintRect {
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: rect.height,
                        })
                        .collect()
                    })
            })),
        };
        host
    }

    fn ConnectDOMBindings(&self, bindings: &Rc<RefCell<DOMJavaScriptBindings>>) {
        *self.bindings.borrow_mut() = Rc::downgrade(bindings);
        self.blocking_wheel_targets.borrow_mut().take();
        self.blocking_wheel_invalidation_bindings.set(0);
        if let Some(domain) = self.local_frames.get() {
            let page = domain.page.Downgrade();
            // The callback clears the cache of the PageState it names; only
            // that PageState may retain targets for these bindings.
            let reaches_self = std::ptr::eq(page.as_ptr(), self);
            local_frame_scroll::InteractionScrollOwner::Root(page)
                .BindBlockingWheelInvalidation(&bindings.borrow());
            if reaches_self {
                self.blocking_wheel_invalidation_bindings
                    .set(Rc::as_ptr(bindings) as *const () as usize);
            }
        }
    }
    /// Element::RecalcOwnStyle still installs a visually identical computed
    /// style on its LayoutObject. Chromium performs that owner update during
    /// UpdateStyleAndLayoutTree without scheduling Layout/PrePaint/Paint. Keep
    /// the same identity/lifecycle here before a retained scroll or frame can
    /// observe the canonical Element style.
    fn SynchronizeNonVisualStyleOwners(&self) {
        let (geometry_can_remain_current, owner_updates, geometry_revision) = {
            let owner = self.document.borrow();
            let state = owner.GetDocument().StyleState();
            (
                self.styles_resolved.get()
                    && state.impact.IsEmpty()
                    && state.paint_style_nodes.is_empty()
                    && !state.layout_inputs_require_tree
                    && state.layout_tree_roots.is_empty()
                    && state.layout_reattach_nodes.is_empty()
                    && state.geometry_input_nodes.is_empty()
                    && state.editing_input_nodes.is_empty(),
                !state.layout_style_nodes.is_empty(),
                state.measurement_geometry_revision,
            )
        };
        let revision_needs_certification = self.layout_revision.get() != Some(geometry_revision);
        if !geometry_can_remain_current || (!owner_updates && !revision_needs_certification) {
            return;
        }
        let mut trace = browser_tracing::span("lifecycle", "Page.SynchronizeNonVisualStyleOwners");
        if owner_updates {
            crate::persistent_layout::SynchronizeLayoutTreeWithEngine(
                &mut self.layout_engine.borrow_mut(),
                &mut self.document.borrow_mut(),
                &self.interaction.borrow(),
                &self.constraints.borrow(),
            );
        }
        let geometry_current = self.layout_engine.borrow().Output().is_some();
        if geometry_current {
            // Style recalc may have conservatively revoked Page's revision
            // before the native SetStyle diff proved that no LayoutObject needs
            // layout. Re-certify the same resident result; never do this when
            // the owning LayoutEngine reports a dirty root.
            self.MarkLayoutCurrent();
        }
        trace.set("owner_updates", owner_updates as u8 as f64);
        trace.set("geometry_current", geometry_current as u8 as f64);
        trace.set(
            "remaining",
            self.document
                .borrow()
                .GetDocument()
                .StyleState()
                .layout_style_nodes
                .len() as f64,
        );
    }
    fn ApplyPendingScrollUpdates(&self, frame: Option<&mut PageFrame>) -> bool {
        let mut trace = browser_tracing::span("lifecycle", "Page.ApplyPendingScrollUpdates");
        let mut updates = self.scroll_updates.borrow_mut();
        trace.set("pending_scrolls", updates.len() as f64);
        trace.set("records_reused", 0.0);
        if updates.is_empty() {
            return false;
        }
        let compositor_owned = self.compositor_scroll_updates.get();
        let mut reused_recordings = false;
        let reused = frame.is_some_and(|frame| {
            // A pending tree/layout change must replace the resident geometry,
            // but an unresolved selector mutation or a paint-only style change
            // does not stop Blink from synchronizing the compositor delta to
            // its existing ScrollableArea before scroll events run.  Do not
            // manufacture a layout invalidation merely because style recalc
            // has not classified that mutation yet.
            let style_impact = self.document.borrow().GetStyleImpact();
            if style_impact.layout || style_impact.reattach {
                trace.set("impact_rejected", 1.0);
                return false;
            }
            // A DOM/style write initially disqualifies the input batch, but
            // resolving it can prove that no layout or paint input changed.
            // Style resolution and geometry queries never consume paint
            // impact; only a committed Paint does. Re-admit that resolved noop
            // here, without skipping the resource/style flush above or the
            // engine's native NeedsLayout/result-identity checks below.
            let offset_only =
                self.scroll_only.get() || (!style_impact.layout && !style_impact.reattach);
            trace.set("offset_only", offset_only as u8 as f64);
            trace.set(
                "resolved_noop",
                (self.styles_resolved.get()
                    && style_impact.IsEmpty()
                    && offset_only
                    && !self.scroll_only.get()) as u8 as f64,
            );
            // A synchronous script/layout query may have advanced Layout to a
            // geometry-only result while the presented frame still owns the
            // last Paint-certified handle. Retained scroll publication cannot
            // promote that layout-only owner to a painted PageFrame: it has no
            // fragment-local Paint input for the next input dispatch. Reject
            // the shortcut before moving the frame owner; the ordinary
            // LayoutUpdateKind::Paint lifecycle below will recertify it.
            let resident_paint_current = self
                .layout_engine
                .borrow()
                .GetLayoutResult()
                .is_some_and(|fragments| PaintFragmentCursor::TryFromHandle(fragments).is_some());
            trace.set(
                "resident_paint_current",
                resident_paint_current as u8 as f64,
            );
            if !resident_paint_current {
                return false;
            }
            // Move Page's published owner back through Layout. A synchronous
            // CSSOM measurement may already have advanced the resident result;
            // in that case AdoptPublishedLayoutResult keeps that newer owner
            // and retires the stale presented handle. Both paths only move the
            // original LayoutResult owner; they do not copy fragment data.
            let mut layout_output = {
                let mut layout = self.layout_engine.borrow_mut();
                layout.AdoptPublishedLayoutResult(std::mem::take(&mut frame.fragments));
                layout
                    .TakePaintOutput()
                    .expect("scroll update has a resident layout result")
            };
            let lifecycle = self.DocumentLifecycle();
            lifecycle.EnsureStateAtMost(dom::document_lifecycle::LifecycleState::StyleClean);
            lifecycle.AdvanceTo(dom::document_lifecycle::LifecycleState::InPerformLayout);
            let scroll_layout = {
                let mut engine = self.layout_engine.borrow_mut();
                crate::persistent_layout::LayoutPersistentScrollDocumentWithMode(
                    &mut engine,
                    &mut self.document.borrow_mut(),
                    &self.constraints.borrow(),
                    &updates,
                    layout_output.ResultHandleMut(),
                    offset_only,
                )
            };
            lifecycle.AdvanceTo(dom::document_lifecycle::LifecycleState::AfterPerformLayout);
            lifecycle.AdvanceTo(dom::document_lifecycle::LifecycleState::LayoutClean);
            let Some(scroll_layout) = scroll_layout else {
                lifecycle.ScheduleLayout();
                frame.fragments = self
                    .layout_engine
                    .borrow_mut()
                    .CommitRetainedPaint(layout_output);
                return false;
            };
            let style_or_paint_pending = !self.styles_resolved.get() || style_impact.paint;
            if style_or_paint_pending {
                // The scroll event must observe the synchronized ScrollableArea,
                // but the ordinary rendering update still owns pending Style
                // and Paint work. Return the resident owner without declaring
                // the lifecycle PaintClean. A cull refresh specifically needs
                // a later Paint even if style recalc proves visually identical.
                frame.fragments = self
                    .layout_engine
                    .borrow_mut()
                    .CommitRetainedPaint(layout_output);
                self.MarkLayoutCurrent();
                if scroll_layout != crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry
                    && style_impact.IsEmpty()
                {
                    self.document.borrow_mut().InvalidatePaint();
                }
                trace.set("deferred_lifecycle", 1.0);
                trace.set(
                    "retained_geometry",
                    (scroll_layout
                        == crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry)
                        as u8 as f64,
                );
                reused_recordings =
                    scroll_layout == crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry;
                return true;
            }
            trace.set(
                "retained_geometry",
                (scroll_layout == crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry)
                    as u8 as f64,
            );
            if browser_tracing::profile_input()
                && (!self.scroll_only.get()
                    || scroll_layout
                        != crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry)
            {
                eprintln!(
                    "scroll-layout-admission input_offset_only={} resolved_noop={} mode={}",
                    self.scroll_only.get(),
                    offset_only && !self.scroll_only.get(),
                    if scroll_layout
                        == crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry
                    {
                        "retained-geometry"
                    } else if offset_only {
                        // These are the two remaining native fast-path gates;
                        // do not attribute this fallback to the input flag.
                        "native-needs-layout-or-missing-result"
                    } else {
                        "unresolved-mixed-input"
                    }
                );
            }
            // The impl tree has already applied a compositor-owned offset.
            // Keep Layout's resident source objects/fragments current for
            // CSSOM geometry and scroll events, but do not rebuild or copy the
            // immutable PaintArtifact merely to restate the same offset.  cc's
            // SyncedScrollOffset retires the reflected delta when the
            // BeginMainFrame finishes without updates.  A cull refresh still
            // falls through to normal Paint so newly exposed content exists.
            if compositor_owned
                && scroll_layout == crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry
            {
                frame.fragments = self
                    .layout_engine
                    .borrow_mut()
                    .CommitRetainedPaint(layout_output);
                self.MarkLayoutCurrent();
                self.CommitRetainedPaintLifecycle();
                trace.set("compositor_geometry_only", 1.0);
                reused_recordings = true;
                return true;
            }
            let mut paint = self.paint_engine.borrow_mut();
            {
                // Stop identifying the currently submitted artifact as the next
                // commit input before Paint mutates its retained snapshot.
                self.InvalidatePreparedArtifact();
            }
            // PaintEngine's persistent PaintController data is the artifact's
            // only document-side owner (paint_controller.h:85-99), so a
            // property-only update mutates it without a Page round-trip.
            let property_update = (scroll_layout
                == crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry)
                .then(|| paint.TryUpdateScrollProperties(&layout_output));
            reused_recordings = matches!(&property_update, Some(Ok(_)));
            if reused_recordings {
                self.CommitRetainedPaintLifecycle();
            }
            let paint_output = if !reused_recordings {
                trace.set("full_paint", 1.0);
                if browser_tracing::profile_input() {
                    if let Some(Err(reason)) = property_update {
                        eprintln!("scroll-paint-reuse rejected={reason:?}");
                    }
                }
                let layout = self.layout_engine.borrow();
                let _svg_resources =
                    layout
                        .Pointer()
                        .SVGResourceQueryScope()
                        .expect("scroll Paint retains native SVG resources");
                paint.PaintWithLifecycle(
                    &layout_output,
                    self.layout_editing.PaintState(),
                    &PagePaintLifecycle(self.DocumentLifecycle()),
                    self.document
                        .borrow()
                        .GetDocument()
                        .CanvasDocumentIdentity(),
                );
                Some(paint.CommitPaintResult())
            } else {
                None
            };
            drop(paint);
            if let Some(paint_output) = paint_output {
                // Layout's commit hands back the committed artifact handle;
                // Page only inspects it here and leaves ownership with
                // PaintEngine (local_frame_view.cc:3185-3227).
                let (fragments, artifact) = self
                    .layout_engine
                    .borrow_mut()
                    .CommitPaint(layout_output, paint_output, Some(&self.layout_editing));
                frame.fragments = fragments;
                frame.document_image_records_complete =
                    artifact.HasExactDocumentImageContainerRecords();
                self.CountPaintUseCounters(&artifact);
            } else {
                frame.fragments = self
                    .layout_engine
                    .borrow_mut()
                    .CommitRetainedPaint(layout_output);
            }
            // Share the immutable post-scroll output. Every mutation clears
            // measurement first, so callers retaining an older frame are the
            // only reason a later update performs copy-on-write.
            self.MarkLayoutCurrent();
            true
        });
        if !reused {
            // A resource/style mutation may reject the retained scroll-only
            // path, but Chromium still applies the explicit scroll to its
            // resident ScrollableArea before the ensuing full layout. Deliver
            // the same typed mutation now so ScrollAnchor drops the obsolete
            // pre-user-scroll baseline; the full layout remains responsible
            // for publishing the replacement result.
            let mut applied_scrolls = 0usize;
            {
                let mut layout = self.layout_engine.borrow_mut();
                let mut document = self.document.borrow_mut();
                for &(node_id, offset, scroll_type) in updates.iter() {
                    if document.EmitScrollOffset(node_id, offset, scroll_type, |mutation| {
                        layout.ApplyMutation(mutation)
                    }) {
                        applied_scrolls += 1;
                    }
                }
            }
            trace.set("mixed_scrolls_applied", applied_scrolls as f64);
            // The scroll offsets above reached their resident ScrollableAreas
            // (an unapplied one stays queued as a per-node geometry input),
            // and the mutation that rejected the retained path carries its
            // own invalidation. Chromium runs layout over the same
            // LayoutObject tree here; it does not rebuild the tree.
            self.document.borrow_mut().InvalidateLayoutGeometry();
        }
        updates.clear();
        drop(updates);
        self.compositor_scroll_updates.set(false);
        trace.set("records_reused", reused_recordings as u8 as f64);
        reused_recordings
    }

    // cpp: browser/browser.cc:1748-1758
    fn ResolveStyles(&self) {
        // Document.cc2630-2636: a retained, stopped Document has no live View
        // to update after its Window/DocumentLoader is replaced.
        if !self.document.borrow().GetDocument().IsActive() {
            return;
        }
        let mut trace = browser_tracing::span("lifecycle", "Page.ResolveStyles");
        // HTMLInputElement::InitializeTypeInParsing completes after the parser
        // has installed the element's initial attributes and before the first
        // style recalc can observe its UA shadow tree or control value. DOM
        // records that typed allocation fact while borrowed; release every
        // owner here before the source callback constructs the retained
        // InputType and may synchronously mutate the same document.
        dom::attribute_lifecycle::DrainAttributeLifecycle(&self.document.Handle());
        if let Some(interaction) = self.popup_source_interaction.borrow().clone() {
            interaction::adapter::page_form_validation::RefreshControlValidity(
                &self.document.Handle(),
                &interaction.FormControlState(),
            )
            .expect("Page installs native constraint-validation collaborators before Style");
        }
        // Document.cc:2786: update the dirty overscroll invoker registry before
        // active style. Initial documents have no native source graph yet.
        let has_native_style_changes = {
            let owner = self.document.borrow();
            let pending = owner.GetDocument().StyleState();
            pending.all_dirty || !pending.changes.is_empty()
        };
        let cache_hit = self.styles_resolved.get() && !has_native_style_changes;
        trace.set("cache_hit", cache_hit as u8 as f64);
        if cache_hit {
            return;
        }
        let lifecycle = self.DocumentLifecycle();
        lifecycle.EnsureStateAtMost(dom::document_lifecycle::LifecycleState::StyleClean);
        lifecycle.AdvanceTo(dom::document_lifecycle::LifecycleState::InStyleRecalc);
        let mut svg_resources_changed = false;
        let mut font_style_retries = 0;
        loop {
            let style_complete;
            svg_resources_changed |= {
                let interaction = *self.interaction.borrow();
                let mut style_engine = self.style_engine.borrow_mut();
                let mut document = self.document.borrow_mut();
                style_engine
                    .SetUserInteractionState(&mut document, interaction)
                    .expect("Page StyleEngine belongs to its retained Document");
                document.GetDocumentMut().UpdateSVGUseInstances();
                style_complete = crate::style_services::ResolveLayoutStylesWithEngineAndForcedColorsUsingFontMetrics(
                    &mut style_engine,
                    &mut document,
                    &self.constraints.borrow(),
                    self.preferred_color_scheme.get(),
                    self.forced_colors.get(),
                    self.style_font_metrics.clone(),
                );
                if style_complete {
                    // Source: core/paint/image_painter.cc:73-86. The focused
                    // HTMLAreaElement participates in image-map focus painting
                    // even though it has no LayoutObject and normally lives in a
                    // display:none subtree. Resolve its query-only style before
                    // exporting the associated LayoutImage's immutable input.
                    if let Some(area_id) = interaction.focused_node_id {
                        if let Some(area) = document
                            .GetDocument()
                            .FindNodeById(area_id)
                            .filter(|&node| document.GetDocument().Node(node).IsHTMLElement("area"))
                        {
                            crate::style_services::EnsureComputedStyleForNode(
                                &mut style_engine,
                                &mut document,
                                &self.constraints.borrow(),
                                self.preferred_color_scheme.get(),
                                self.forced_colors.get(),
                                area,
                            );
                            if document.GetDocument().ImageMapAreaHasPotentialPath(area_id) {
                                document
                                    .GetDocumentMut()
                                    .InvalidateImageMapPaintOwnerForAreaId(area_id);
                            }
                        }
                    }
                    document.UpdateSVGResources()
                } else {
                    false
                }
            };

            // Native HTML DidRecalcStyle runs after releasing the canonical
            // DOM/style borrows and before attachment. Its option observers
            // register on the same document Agent; no script is run here.
            if style_complete {
                dom::attribute_lifecycle::DrainAttributeLifecycle(&self.document.Handle());
            }

            // CSSFontSelector::WillUseFontData reaches FontCache synchronously
            // in Chromium. Keep Style and Layout platform-independent here:
            // Style emits pure misses and Page completes them before Layout.
            // Only metric-dependent CSS values require a style retry; an
            // ordinary font-family miss can enter Layout with its face ready.
            let metrics_pending = self.style_font_metrics.TakeMetricsPending();
            let font_completion = self
                .system_fonts
                .borrow_mut()
                .CompletePass(&mut self.constraints.borrow_mut());
            if !style_complete && !font_completion.changed {
                panic!(
                    "native custom style has no usable font after completing its source queries"
                );
            }
            if style_complete && (!font_completion.changed || !metrics_pending) {
                trace.set("font_style_retries", font_style_retries as f64);
                break;
            }
            font_style_retries += 1;
            let invalidated = self
                .document
                .borrow_mut()
                .GetDocumentMut()
                .InvalidateStylesForFontMetrics();
            trace.set("font_metric_style_invalidations", invalidated as f64);
        }
        if svg_resources_changed {
            // Local SVG target/content changes invalidate consumers even when
            // every consumer's canonical clip-path is unchanged. Revoke an
            // animation-only paint proof before the retained-frame gate.
            self.ApplyAnimationMutation(animation::AnimationMutation::InvalidatePaintBatch);
            self.scroll_only.set(false);
            self.dirty.set(true);
        }
        {
            let document = self.document.borrow();
            let state = document.GetDocument().StyleState();
            trace.set("document_nodes", document.GetDocument().NodeCount() as f64);
            trace.set("resolved_nodes", state.stats.resolved_nodes as f64);
            trace.set("changed_nodes", state.stats.changed_nodes as f64);
            trace.set("rule_sets_built", state.stats.rule_sets_built as f64);
            trace.set(
                "selector_dependency_sets_built",
                state.stats.rule_sets_built as f64,
            );
        }
        self.styles_resolved.set(true);
        lifecycle.AdvanceTo(dom::document_lifecycle::LifecycleState::StyleClean);
        // Virtual scrollbar styles depend on originating selectors/attributes
        // even when the element keeps its previous ComputedStyle identity.
        // A first sample participates in the animation cascade and therefore
        // needs one more style pass before Layout consumes the computed style.
        // An unchanged definition produces no Document mutation, so this
        // converges without polling or a page-specific delay.
        if self.SynchronizeCSSAnimations() {
            self.ResolveStyles();
        }
    }

    // cpp: style_engine.cc:3865-3950; container_query_evaluator.cc:369-385.
    // Chromium interleaves descendant style recalc with the sized container's
    // layout. This assembly exports complete native layout batches, so advance
    // style/layout until the published content sizes are unchanged before any
    // measurement or paint consumes the batch.
    fn LayoutWithContainerQueries(&self, purpose: LayoutUpdateKind) -> bool {
        // Source: PaintLayerScrollableArea::UserInputScrollable. Fullscreen is
        // retained by DOM, while Layout owns the scrollbar policy decision.
        // Publish only the document-level fact that crosses that boundary.
        let fullscreen_blocks_layout_view_scroll = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            let document_element = document
                .Node(document.Root())
                .Children()
                .iter()
                .copied()
                .find(|&child| document.Node(child).Type() == DOMNodeType::kElement);
            document
                .FullscreenElement()
                .is_some_and(|fullscreen| Some(fullscreen) != document_element)
        };
        self.constraints
            .borrow_mut()
            .fullscreen_element_blocks_layout_view_scroll = fullscreen_blocks_layout_view_scroll;
        let mut observed = std::collections::HashSet::new();
        let mut updated = false;
        let limit = self.document.borrow().GetDocument().NodeCount() + 1;
        for _ in 0..limit {
            #[cfg(test)]
            self.full_layout_lifecycles
                .set(self.full_layout_lifecycles.get() + 1);
            let sizes = {
                loop {
                    let lifecycle = self.DocumentLifecycle();
                    lifecycle
                        .EnsureStateAtMost(dom::document_lifecycle::LifecycleState::StyleClean);
                    // LayoutObjectTreeBuilder::Attach creates the canonical
                    // object before HTMLInputElement::AttachLayoutTree runs.
                    // Synchronize that owner first, then release DOM/Layout
                    // borrows so the typed attachment callback can update the
                    // image loader and usage state synchronously. Layout must
                    // consume the same already-attached tree afterwards.
                    {
                        let mut layout = self.layout_engine.borrow_mut();
                        let mut document = self.document.borrow_mut();
                        crate::persistent_layout::SynchronizeLayoutTreeWithEngine(
                            &mut layout,
                            &mut document,
                            &self.interaction.borrow(),
                            &self.constraints.borrow(),
                        );
                    }
                    dom::attribute_lifecycle::DrainAttributeLifecycle(&self.document.Handle());
                    lifecycle.AdvanceTo(dom::document_lifecycle::LifecycleState::InPerformLayout);
                    updated |=
                        crate::font_session::LayoutWithSystemFontsAtStageAfterTreeSynchronization(
                            &mut self.layout_engine.borrow_mut(),
                            &mut self.document.borrow_mut(),
                            &self.interaction.borrow(),
                            &mut self.constraints.borrow_mut(),
                            &mut self.system_fonts.borrow_mut(),
                            purpose,
                        );
                    lifecycle
                        .AdvanceTo(dom::document_lifecycle::LifecycleState::AfterPerformLayout);
                    lifecycle.AdvanceTo(dom::document_lifecycle::LifecycleState::LayoutClean);
                    if let Some(domain) = self.local_frames.get() {
                        domain.root_view.DidLayout();
                    }
                    // Apply ready (including MemoryCache) font responses at
                    // the host boundary before exposing provisional layout.
                    // No layout/DOM borrow survives decoding or callbacks, and
                    // an outstanding network load never waits here.
                    let mut font_trace =
                        browser_tracing::span("lifecycle", "Page.PollPendingFontsAfterLayout");
                    let completed = self.resources.borrow().as_ref().map_or(0, |resources| {
                        resources.PollPendingFonts(&mut LayoutFontResourceClient { state: self })
                    });
                    font_trace.set("completed", completed as f64);
                    if completed == 0 {
                        break;
                    }
                    self.layout_engine.borrow_mut().ReleaseLayoutResult();
                }
                let layout = self.layout_engine.borrow();
                if !self.style_engine.borrow().HasContainerQueries() {
                    drop(layout);
                    if self.ReflectLayoutScrollOffsets() {
                        updated = true;
                        continue;
                    }
                    return updated;
                }
                layout.ContainerQuerySizes()
            };
            let changed = {
                let mut container_trace =
                    browser_tracing::span("lifecycle", "Page.PublishContainerSizes");
                container_trace.set("containers", sizes.len() as f64);
                let changed = crate::style_services::PublishContainerSizes(
                    &mut self.style_engine.borrow_mut(),
                    &mut self.document.borrow_mut(),
                    &sizes,
                );
                container_trace.set("changed", changed as u8 as f64);
                changed
            };
            if !changed {
                if self.ReflectLayoutScrollOffsets() {
                    updated = true;
                    continue;
                }
                return updated;
            }
            let signature = sizes
                .iter()
                .map(|s| (s.node_id, s.width.to_bits(), s.height.to_bits()))
                .collect::<Vec<_>>();
            if !observed.insert(signature) {
                break;
            }
            self.styles_resolved.set(false);
            self.ResolveStyles();
        }
        std::panic::panic_any(foundation::UnsupportedLayout::new(
            "container query style/layout did not converge (style_engine.cc:3865)",
        ));
    }

    // Layout owns clamping after overflow/scrollbar changes. Reflect only its
    // final native values to Document and the existing scroll-event queue; this
    // is not a new scroll request and must not invalidate the completed layout.
    fn ReflectLayoutScrollOffsets(&self) -> bool {
        let mut offsets = self
            .layout_engine
            .borrow()
            .Output()
            .map_or_else(Vec::new, |output| output.ScrollOffsets());
        offsets.retain(|(id, offset)| {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            document.FindNodeById(*id).is_some_and(|index| {
                document.Node(index).Type() == dom::persistent_document::DOMNodeType::kElement
                    && document.ScrollOffsetFor(index) != *offset
            })
        });
        for (node_id, offset) in offsets {
            if let Some(effect) = self.document.ApplyScrollOffset(node_id, offset) {
                // A synchronous geometry query can run before an older queued
                // request reaches layout. Never restore that superseded offset.
                self.scroll_updates
                    .borrow_mut()
                    .retain(|(id, _, _)| *id != node_id);
                let mut targets = self.scroll_event_targets.borrow_mut();
                if !targets.contains(&effect.event_target_node_id) {
                    targets.push(effect.event_target_node_id);
                }
            }
        }
        // LocalFrameView.cc:2978 executes snap updates after converged
        // style/layout, before any Paint consumes this result. Instant scroll
        // effects use the existing mutation/lifecycle queue and repeat this
        // lifecycle when they invalidate native geometry.
        // Document.cc:3011-3025, before Paint can consume this Layout result.
        // Publish its existing measurement revision before finder reentry.
        self.layout_revision.set(Some(
            self.document.borrow().GetMeasurementGeometryRevision(),
        ));
        let snapshot_invalidated_style = !self.styles_resolved.get();
        if snapshot_invalidated_style {
            self.ResolveStyles();
        }
        snapshot_invalidated_style
    }

    // cpp: Document::UpdateStyleAndLayoutForNode. Detached nodes have no
    // layout object and must not flush the active document's lifecycle.
    fn EnsureLayoutOutputForNode(&self, id: u64) -> bool {
        let connected = {
            let owner = self.document.borrow();
            let tree = owner.GetDocument();
            tree.IsActive()
                && tree
                    .FindNodeById(id)
                    .is_some_and(|index| tree.IsConnectedToDocument(index))
        };
        if connected {
            self.EnsureLayoutOutput();
        }
        connected
    }

    // Document::UpdateStyleAndLayoutForNode. Updating layout publishes no
    // geometry tree: binding calls borrow the engine's resident output below.
    fn EnsureLayoutOutput(&self) {
        let mut trace = browser_tracing::span("layout", "Page.EnsureLayoutOutput");
        self.ResolveStyles();
        let revision = self.document.borrow().GetMeasurementGeometryRevision();
        let current = self.layout_revision.get() == Some(revision)
            && self.layout_engine.borrow().Output().is_some()
            && crate::persistent_layout::MeasurementInputsMatch(
                &self.layout_engine.borrow(),
                &mut self.document.borrow_mut(),
                &self.interaction.borrow(),
            );
        trace.set("cache_hit", current as u8 as f64);
        if current {
            return;
        }
        self.LayoutWithContainerQueries(LayoutUpdateKind::Layout);
        self.MarkLayoutCurrent();
        // Document::UpdateStyleAndLayoutForNode may synchronously replace the
        // resident LayoutResult, but it does not run PrePaint/Paint. Keep the
        // Page lifecycle pending until that exact owner has been committed to
        // PageFrame; otherwise the old displayed frame and the new resident
        // geometry diverge while `dirty` incorrectly says Paint is current.
        self.dirty.set(true);
    }

    // cpp: browser/browser.cc:947-1044
    fn ApplyMutation(&self, mutation: PageMutation) {
        // One BeginFrame can deliver several Element.animate() samples and a
        // document-image frame. Neither is an unrelated outer-document edit:
        // keep accumulating the targeted paint batch until this lifecycle is
        // committed. Geometry/style mutations still revoke the proof.
        if !matches!(
            &mutation,
            PageMutation::AnimationTick(_)
                | PageMutation::ResourceMutation(ResourceMutation::DocumentImageFrameChanged(_))
        ) {
            self.ApplyAnimationMutation(animation::AnimationMutation::InvalidatePaintBatch);
        }
        match mutation {
            PageMutation::DOMMutation(m) => {
                // Runtime-independent callers are no-JS Page mutations and
                // interaction control-value/checked actions. JavaScript DOM
                // mutations and public Page::Apply use the runtime scope below.
                self.ApplyDOMMutation(&m, &mut |_, _| {
                    assert!(
                        self.bindings.borrow().upgrade().is_none(),
                        "cached image event requires the Page runtime scope"
                    );
                });
            }
            PageMutation::ResourceMutation(m) => self.ApplyResourceMutation(m),
            PageMutation::InteractionStateMutation(m) => {
                let previous = *self.interaction.borrow();
                if previous != m.state {
                    let selector_state_changed = previous.focused_node_id
                        != m.state.focused_node_id
                        || previous.focus_visible_node_id != m.state.focus_visible_node_id
                        || previous.hovered_node_id != m.state.hovered_node_id
                        || previous.pressed_node_id != m.state.pressed_node_id
                        || previous.multi_select_focused_option_id
                            != m.state.multi_select_focused_option_id;
                    let needs_paint = {
                        let owner = self.document.borrow();
                        let layout = self.layout_engine.borrow();
                        let document = owner.GetDocument();
                        if !::interaction::ValidateInteractionState(document, m.state) {
                            dom::error::invalid_argument(
                                "interaction state references a missing DOM node",
                            );
                        }
                        let appearance = |id| layout.HasEffectiveAppearance(id);
                        let facts = ::interaction::InteractionStateDocumentWithAppearance::new(
                            document,
                            &appearance,
                        );
                        ::interaction::InteractionStateNeedsPaint(&facts, previous, m.state)
                    };
                    *self.interaction.borrow_mut() = m.state;
                    if let Some(bindings) = self.bindings.borrow().upgrade() {
                        bindings
                            .borrow_mut()
                            .SetFocusedNode(m.state.focused_node_id);
                    }
                    if selector_state_changed {
                        // User-action selectors and
                        // :-internal-multi-select-focus consume this snapshot.
                        // The StyleEngine setter below turns the state change
                        // into selector invalidation at the next lifecycle.
                        self.styles_resolved.set(false);
                        self.scroll_only.set(false);
                        self.dirty.set(true);
                    }
                    if needs_paint {
                        let mut owner = self.document.borrow_mut();
                        owner.InvalidatePaint();
                        owner
                            .GetDocumentMut()
                            .StyleStateMut()
                            .paint_style_nodes
                            .extend(
                                [
                                    previous.hovered_node_id,
                                    m.state.hovered_node_id,
                                    previous.pressed_node_id,
                                    m.state.pressed_node_id,
                                    previous.focus_visible_node_id,
                                    m.state.focus_visible_node_id,
                                    previous.focused_node_id,
                                    m.state.focused_node_id,
                                    previous.multi_select_focused_option_id,
                                    m.state.multi_select_focused_option_id,
                                ]
                                .into_iter()
                                .flatten(),
                            );
                        drop(owner);
                        // Focus/hover/pressed state changes native appearance
                        // and focus/caret paint, but not box geometry. Preserve
                        // the last geometry as a stale-paint candidate so a
                        // synchronous getBoundingClientRect() in the input
                        // listener does not force a duplicate full layout.
                        // UpdateFrameIfNeeded still rejects it for PaintCurrent
                        // and exports the new interaction paint state.
                        self.scroll_only.set(false);
                        self.dirty.set(true);
                    }
                }
            }
            PageMutation::CSSOMMutation(m) => {
                self.resources
                    .borrow()
                    .as_ref()
                    .expect("stylesheet before navigation")
                    .AddParsedStyleSheet(m.style_sheet, &m.base_url)
                    .unwrap_or_else(|e| std::panic::panic_any(e));
                self.InvalidateStylePreservingGeometry();
            }
            PageMutation::ViewportMutation(m) => {
                if !m.size.width.is_finite()
                    || !m.size.height.is_finite()
                    || m.size.width <= 0.0
                    || m.size.height <= 0.0
                {
                    dom::error::invalid_argument("viewport size must be positive");
                }
                let mut constraints = self.constraints.borrow_mut();
                let viewport_changed = constraints.available_size != m.size
                    || constraints.viewport.is_some_and(|viewport| {
                        viewport.size.width != m.size.width.round() as i32
                            || viewport.size.height != m.size.height.round() as i32
                    });
                constraints.available_size = m.size;
                if let Some(viewport) = &mut constraints.viewport {
                    viewport.size = IntSize {
                        width: m.size.width.round() as i32,
                        height: m.size.height.round() as i32,
                    };
                }
                drop(constraints);
                // A changed initial containing block requires layout even
                // when no viewport-dependent computed style changes.
                if viewport_changed {
                    self.document.borrow_mut().InvalidateLayoutGeometry();
                }
                self.InvalidateLayoutOutput();
                self.InvalidateStylePreservingGeometryWithResources(if viewport_changed {
                    ResourceReferenceDirty::VIEWPORT
                } else {
                    ResourceReferenceDirty::default()
                });
            }
            PageMutation::ScrollMutation(m) => {
                if let Some(effect) = self.document.ApplyScrollOffset(m.target_node_id, m.offset) {
                    self.compositor_scroll_updates.set(false);
                    self.InvalidateLayoutOutput();
                    browser_tracing::instant(
                        "input",
                        "ScrollApplied",
                        &[
                            ("scroll_node", effect.target_node_id as f64),
                            ("before_x", effect.before.x),
                            ("before_y", effect.before.y),
                            ("offset_x", effect.offset.x),
                            ("offset_y", effect.offset.y),
                            ("delta_x", effect.offset.x - effect.before.x),
                            ("delta_y", effect.offset.y - effect.before.y),
                        ],
                    );
                    let mut updates = self.scroll_updates.borrow_mut();
                    if let Some((_, offset, scroll_type)) = updates
                        .iter_mut()
                        .find(|(id, _, _)| *id == effect.target_node_id)
                    {
                        *offset = effect.offset;
                        *scroll_type = m.scroll_type;
                    } else {
                        updates.push((effect.target_node_id, effect.offset, m.scroll_type));
                    }
                    let mut targets = self.scroll_event_targets.borrow_mut();
                    if !targets.contains(&effect.event_target_node_id) {
                        targets.push(effect.event_target_node_id);
                    }
                    // Keep the proof across coalesced offset writes, but never
                    // admit a scroll that follows another pending mutation.
                    if !self.dirty.get() {
                        self.scroll_only.set(true);
                    }
                    self.dirty.set(true);
                }
            }
            PageMutation::AnimationTick(m) => {
                self.ApplyAnimationMutation(animation::AnimationMutation::Tick(m));
            }
        }
    }

    // cpp: browser/browser.cc:1046-1067
    fn ApplyResourceMutation(&self, m: ResourceMutation) {
        match m {
            ResourceMutation::PlatformImageResourceReady(image) => {
                self.constraints.borrow_mut().PushImage(image);
                self.dirty.set(true);
            }
            ResourceMutation::ImageResourceReady(m) => {
                dom::image_resource::AddImageResourceWithContent(
                    &mut self.document.borrow_mut(),
                    &mut self.constraints.borrow_mut(),
                    m.source,
                    m.image,
                    m.content,
                );
                // Completion updates intrinsic metadata and the resource ids of
                // existing style consumers, without changing their URL/font
                // references. Preserve any pending DOM/sheet discovery; load
                // listeners invalidate references through the normal mutation path.
                self.InvalidateLayoutOutput();
                self.styles_resolved.set(false);
                self.scroll_only.set(false);
                self.dirty.set(true);
            }
            ResourceMutation::VideoFrameChanged(m) => {
                use image_resource::{
                    PaintImageAnimationMetadata, PaintImageAnimationType, PaintImageContent,
                };
                let valid_metadata = matches!(
                    &m.frame.compositor_metadata,
                    Ok(metadata)
                        if matches!(
                            metadata.animation,
                            PaintImageAnimationMetadata::Known(info)
                                if info.animation_type == PaintImageAnimationType::Video
                        )
                );
                let valid_pixels = matches!(&m.frame.content, PaintImageContent::Bitmap(pixels)
                    if (m.frame.width as usize)
                        .checked_mul(m.frame.height as usize)
                        .and_then(|count| count.checked_mul(4)) == Some(pixels.len()));
                if m.frame.id == 0
                    || m.frame.paint_image_id == image_resource::PaintImageId::INVALID
                    || m.frame.width == 0
                    || m.frame.height == 0
                    || !valid_metadata
                    || !valid_pixels
                {
                    dom::error::invalid_argument(
                        "video frame requires typed metadata and RGBA pixels",
                    );
                }
                let mut document = self.document.borrow_mut();
                let previous = document.GetDocument().VideoFrameMetadataFor(m.node_id);
                if previous.is_some_and(|previous| {
                    previous.resource_id != m.frame.id || m.frame.revision <= previous.revision
                }) {
                    dom::error::invalid_argument(
                        "video frame identity must be stable and revision must increase",
                    );
                }
                let metadata = dom::VideoFrameMetadata {
                    resource_id: m.frame.id,
                    revision: m.frame.revision,
                    width: m.frame.width,
                    height: m.frame.height,
                };
                let geometry_changed = document
                    .GetDocumentMut()
                    .SetVideoFrameMetadata(m.node_id, metadata);
                drop(document);
                if !m.resource.Validate()
                    || m.resource.width != m.frame.width
                    || m.resource.height != m.frame.height
                    || m.resource.orientation != m.frame.orientation
                {
                    dom::error::invalid_argument(
                        "video frame resource must match the software fallback",
                    );
                }
                let foreign_layer = self
                    .PublishVideoFrame(m.node_id, &m.resource)
                    .unwrap_or_else(|error| {
                        panic!("video child Surface publication failed: {error}")
                    });
                self.paint_engine
                    .borrow_mut()
                    .SetForeignLayer(m.node_id, Some(foreign_layer));
                self.video_frame_resources
                    .borrow_mut()
                    .insert(m.node_id, m.resource);
                let mut constraints = self.constraints.borrow_mut();
                if let Some(existing) = constraints
                    .ImagesMut()
                    .iter_mut()
                    .find(|image| image.id == m.frame.id)
                {
                    *existing = m.frame;
                } else {
                    constraints.PushImage(m.frame);
                }
                drop(constraints);
                if geometry_changed {
                    self.InvalidateLayoutOutput();
                }
                self.paint_resources_dirty.set(true);
                self.scroll_only.set(false);
                self.dirty.set(true);
            }
            ResourceMutation::DocumentImageFrameChanged(m) => {
                let mut constraints = self.constraints.borrow_mut();
                let image = constraints
                    .ImagesMut()
                    .iter_mut()
                    .find(|image| image.id == m.frame.resource_id)
                    .unwrap_or_else(|| {
                        dom::error::invalid_argument("animated image resource missing")
                    });
                if image.width != m.frame.intrinsic_size.width
                    || image.height != m.frame.intrinsic_size.height
                {
                    dom::error::invalid_argument("animated image changed intrinsic dimensions");
                }
                if m.frame.revision <= image.revision {
                    dom::error::invalid_argument("animated image revision must increase");
                }
                image.revision = m.frame.revision;
                image.UpdateDocumentFrameMetadata(&m.frame);
                image.document_container_key = Some(m.frame.container_key.clone());
                let image_resource::PaintImageContent::Document(content) = &image.content else {
                    dom::error::invalid_argument("document image frame targeted a bitmap resource")
                };
                image.content = image_resource::PaintImageContent::Document(Arc::new(
                    content.with_variant(m.frame.container_key.clone(), m.frame.record.clone()),
                ));
                let snapshot = Arc::new(image.clone());
                drop(constraints);
                if let Some(content) = self
                    .document
                    .borrow()
                    .GetDocument()
                    .ImageResourceContentFor(&m.source)
                {
                    content.SetImageSnapshot(snapshot);
                }
                self.pending_paint_mutations.borrow_mut().push(
                    paint::paint_engine::PaintMutation::ImageChanged {
                        image_id: m.frame.resource_id,
                        revision: m.frame.revision,
                        frame: m.frame,
                    },
                );
                self.paint_resources_dirty.set(true);
                self.dirty.set(true);
            }
            ResourceMutation::DocumentImageIntrinsicSizeChanged(m) => {
                let mut constraints = self.constraints.borrow_mut();
                let image = constraints
                    .ImagesMut()
                    .iter_mut()
                    .find(|image| image.id == m.resource_id)
                    .unwrap_or_else(|| {
                        dom::error::invalid_argument("document image resource missing")
                    });
                if m.revision <= image.revision || m.size.width == 0 || m.size.height == 0 {
                    dom::error::invalid_argument("invalid document image intrinsic size revision");
                }
                image.width = m.size.width;
                image.height = m.size.height;
                image.revision = m.revision;
                let resolution_scale = image.resolution_scale;
                let orientation = image.orientation;
                let is_cors_same_origin = image.is_cors_same_origin;
                drop(constraints);
                if !self
                    .document
                    .borrow_mut()
                    .GetDocumentMut()
                    .UpdateImageResourceIntrinsicSize(
                        &m.source,
                        dom::ImageResourceMetadata {
                            id: m.resource_id,
                            natural_width: m.size.width as f64 / resolution_scale,
                            natural_height: m.size.height as f64 / resolution_scale,
                            resolution_scale,
                            orientation,
                            known_to_be_opaque: false,
                            is_cors_same_origin,
                        },
                    )
                {
                    dom::error::invalid_argument("document image source missing");
                }
                self.InvalidateLayoutOutput();
                // StyleFetchedImage retains the resource's natural size and
                // resolution. Re-resolve actual consumers so CSS backgrounds,
                // masks, border images and generated content observe the new
                // intrinsic dimensions; a layout-only invalidation would keep
                // their old immutable StyleImage payload.
                self.styles_resolved.set(false);
                self.document.borrow_mut().InvalidateLayout();
                self.scroll_only.set(false);
                self.dirty.set(true);
            }
            ResourceMutation::FontFaceResourceChanged { owner, font } => {
                self.InvalidateLayoutOutput();
                let mut constraints = self.constraints.borrow_mut();
                if let Some(engine) = &constraints.font_engine {
                    let mutation = match font {
                        Some(font) => {
                            font_engine::FontMutation::UpsertRule(font_engine::FontRule {
                                id: owner,
                                family: font.family.clone(),
                                state: font_engine::FontFaceState::Ready(font),
                            })
                        }
                        None => font_engine::FontMutation::RemoveRule(owner),
                    };
                    engine
                        .with_mut(|engine| engine.ApplyMutation(mutation))
                        .expect("source FontFaceCache mutation");
                    constraints.fonts = engine.with_mut(|engine| engine.ResolvedFaces());
                }
                drop(constraints);
                self.styles_resolved.set(false);
                self.document.borrow_mut().InvalidateLayoutGeometry();
                self.scroll_only.set(false);
                self.dirty.set(true);
            }
            ResourceMutation::FontResourceReady(m) => {
                self.InvalidateLayoutOutput();
                self.document_fonts.borrow_mut().push(m.font);
                let mut constraints = self.constraints.borrow_mut();
                if let Some(engine) = &constraints.font_engine {
                    let faces = self.document_fonts.borrow().iter().cloned().collect();
                    engine
                        .with_mut(|engine| {
                            engine.ApplyMutation(font_engine::FontMutation::SetFaces(faces))
                        })
                        .expect("web font catalog mutation");
                    constraints.fonts = engine.with_mut(|engine| engine.ResolvedFaces());
                }
                drop(constraints);
                // Source: core/css/style_engine.cc:1353-1383. Blink's
                // FontsNeedUpdate invalidates the matched-properties cache,
                // marks font-dependent style dirty, and then invalidates
                // affected layout. Rechrom resolves ch/ex/cap/ic into the
                // immutable ComputedStyle length at this package boundary, so
                // retaining that style would preserve the fallback font's
                // metric after the web face is admitted. Re-resolve style but
                // retain the attached LayoutObject topology.
                self.styles_resolved.set(false);
                self.document.borrow_mut().InvalidateLayoutGeometry();
                self.scroll_only.set(false);
                self.dirty.set(true);
            }
            ResourceMutation::ResourceLoadFailed(m) => {
                if m.kind == page_mutation::ResourceKind::kFont {
                    // A failed face can expose an earlier overlapping face or
                    // another family. Re-enter style's font-relative length
                    // conversion and text selection to request it.
                    self.InvalidateLayoutOutput();
                    self.styles_resolved.set(false);
                    // Trying the next source can change fallback metrics, but
                    // it cannot change the resident LayoutObject topology.
                    self.document.borrow_mut().InvalidateLayoutGeometry();
                    self.scroll_only.set(false);
                    self.dirty.set(true);
                }
                if m.kind == page_mutation::ResourceKind::kImage {
                    if let Some(source) = m.source {
                        let changed = self
                            .document
                            .borrow_mut()
                            .GetDocumentMut()
                            .SetImageResourceFailed(source);
                        if changed {
                            // Mirror successful completion: the document marks
                            // actual image clients for immutable-style refresh.
                            self.InvalidateLayoutOutput();
                            self.styles_resolved.set(false);
                            self.scroll_only.set(false);
                            self.dirty.set(true);
                        }
                    }
                }
                self.client.borrow_mut().DidFailResource(&m.url, &m.error)
            }
        }
    }
}

mod validation_message;
pub use validation_message::{
    ValidationMessageRect, ValidationMessageUpdate, ValidationMessageVisibility,
};

mod begin_frame;
mod drag_image;
mod event_timing;
mod frame_owner_loading;
mod frame_selection_host;
mod layout_shift_input;
pub(crate) mod local_frame;
mod local_frame_event_handler;
mod local_frame_focus;
mod local_frame_history;
mod local_frame_input;
mod local_frame_intersection;
mod local_frame_navigation;
mod local_frame_scroll;
mod local_frame_window;
mod middle_click_autoscroll;
mod mouse_autoscroll;
mod native_drag;
mod native_event_handler_host;
pub use native_event_handler_host::EventHandlerChromeEffect;
mod navigation_api;
mod select_control_host;
pub use navigation_api::RestoreHistoryRequest;
mod opening;
mod page_scale;
mod pointer_geometry;
mod pointer_scroll_controls;
mod popup_tasks;
mod programmatic_scroll;
mod scripting;
mod scroll;
pub(crate) mod main_compositor;
pub use main_compositor::{ImplToMainQueue, MainFrameResult, MainToImplQueue};
pub(crate) mod synchronous_presenter;
pub(crate) mod compositor_input;
pub use compositor_input::{
    CanonicalFlingWheelInput, CompositorWheelResult, GestureCompositorInput,
    EmbedderCompositorConfig, GestureCompositorInputWithDispatchId, InputWidgetScale,
    NewCompositorServices, PageCompositorFrame,
    ImplInput, PresentationFeedbackDetails, WheelCompositorInput,
};
use opening::PageOpenAdapter;
use scripting::PageScripts;

/// Optional execution services. None disables scripting without constructing
/// a runtime, realm, Window bindings or DOM bindings.
pub struct ScriptEnvironment {
    pub runtime: Box<dyn JavaScriptRuntime>,
    pub xhr: Box<dyn xhr_transport::XMLHttpRequestTransport>,
    pub user_agent: String,
}

/// Per-Page worker policy. Actor/event-loop placement belongs to the embedder.
#[allow(non_snake_case)]
#[derive(Clone)]
pub struct PageThreads {
    pub raster: bool,
    pub decode: bool,
    pub scriptCompile: bool,
    /// Background document work such as image decode.
    pub executor: std::sync::Arc<dyn executor::Executor>,
    /// Chromium's script-streaming worker-pool boundary. Script compilation
    /// must not queue behind image decode for the same document.
    pub script_compile_executor: std::sync::Arc<dyn executor::Executor>,
    /// Chromium's compositor sequence. It must remain runnable while raster
    /// workers are saturated so impl input and frame scheduling keep moving.
    pub compositor_executor: std::sync::Arc<dyn executor::Executor>,
    /// Chromium's raster worker pool, independent from the compositor sequence.
    pub raster_executor: std::sync::Arc<dyn executor::Executor>,
}
impl Default for PageThreads {
    fn default() -> Self {
        let executor = executor::inline();
        Self {
            raster: false,
            decode: false,
            scriptCompile: false,
            executor: executor.clone(),
            script_compile_executor: executor.clone(),
            compositor_executor: executor.clone(),
            raster_executor: executor,
        }
    }
}

// cpp: browser/browser.h:69-95
/// One persistent Page lifecycle, independent of whether scripts are enabled.
pub struct Page {
    previous_document_frame_sequence: u64,
    form_controller: Option<Rc<interaction::adapter::page_form_controller::PageFormController>>,
    threads: PageThreads,
    begin_frame: begin_frame::PageBeginFrame,
    state: Rc<PageState>,
    interaction: ::interaction::Interaction<'static>,
    interaction_scroll_runtime: Option<Rc<PageInteractionScrollRuntime>>,
    interaction_scroll_owners:
        Option<Rc<interaction::adapter::page_scroll_manager::PageScrollManagerOwners>>,
    open_engine: OpenEngine,
    resources: Rc<ResourceFetcher>,
    scripts: Option<Box<PageScripts>>,
    script_client: Rc<RefCell<dyn ScriptLoadClient>>,
    frame: Option<PageFrame>,
    cursor_position: Option<Offset>,
    // Blink's MouseEventManager marks hover dirty after layout and resolves it
    // from the last native mouse position at the next BeginMainFrame.
    hover_state_dirty: bool,
    // ScrollableArea::OnScrollFinished marks hover dirty once per completed
    // gesture, after compositor scrolling has stopped moving content beneath
    // a stationary pointer.
    wheel_scroll_active: bool,
    // Impl-thread offsets are reflected to DOM immediately for CSSOM/events,
    // then folded into one main-thread property update when the gesture ends.
    // Native Page runtimes place the compositor on their impl executor. Direct
    // embedders keep the inline pipeline used by PaintInto.
    external_compositor: bool,
    processed_compositor_scroll_sequence: u64,
    /// ChromeClient screen-info input, supplied by the host independently of
    /// CSS/viewport size. Absence is an integration error, never a guessed size.
    // HitTestCache's point/DOM version proof, restricted to one clean committed
    // frame. No fragment Rc is retained; scroll/layout changes advance sequence.
    cursor_hit_test: Option<(
        u64,
        Option<Offset>,
        dom::persistent_document::DOMOwnerHandle,
        u64,
    )>,
    // Chromium retains wheel event regions in the committed layer tree. Keep
    // the flattened root-scroll equivalent until either listener identity or
    // committed fragment geometry changes.
    wheel_event_regions: RefCell<::interaction::WheelEventRegionResolver>,
    #[cfg(test)]
    cursor_hit_test_queries: usize,

    active: bool,
    active_state_published: bool,
    selection_revision: u64,
    document_selection: Option<DocumentSelection>,
    // The focused element FrameCaret last resolved its appearance against.
    // With `selection_revision`, it identifies a caret input change made by
    // an ordinary task, which must schedule the caret's visual update.
    caret_focused_node: Option<u64>,
    editing_paint_dirty: bool,
    // ImageResourceObserver::ImageChanged can invalidate native display-item
    // clients without producing a DOM style impact. Keep that lifecycle proof
    // separate from resource-catalog publication and editing invalidation.
    document_image_environment_dirty: bool,
}
/// Read-only copied owner statistics. Requested capacities overlap across
/// shared owners and cannot be summed into process RSS.
#[derive(Clone, Debug)]
pub struct PageMemoryDiagnostics {
    pub loading: bool,
    pub dirty: bool,
    pub styles_resolved: bool,
    pub pending_scroll_updates: usize,
    pub frame_sequence: Option<u64>,
    pub recording_revision: Option<u64>,
    pub dom: dom::persistent_document::DOMMemoryDiagnostics,
    pub custom_properties: style::document_style_engine::CustomPropertyMemoryDiagnostics,
    pub javascript: Option<javascript::javascript_runtime::JavaScriptMemoryDiagnostics>,
    pub host: Option<webapi::dom_bindings::DOMHostMemoryDiagnostics>,
    pub resources: resource::ResourceMemoryDiagnostics,
    pub catalog_images: usize,
    pub unique_bitmap_capacity: usize,
}
impl Page {
    pub fn JavaScriptGarbageCollectionRequested(&self) -> bool {
        self.scripts
            .as_ref()
            .is_some_and(|scripts| scripts.GarbageCollectionRequested())
    }

    pub fn CollectJavaScriptGarbage(&mut self) {
        if let Some(scripts) = &mut self.scripts {
            scripts.CollectGarbage();
        }
    }

    /// Does not advance lifecycle, execute JS, load resources or collect heaps.
    pub fn MemoryDiagnostics(&self) -> PageMemoryDiagnostics {
        let (javascript, host) = self.scripts.as_ref().map_or((None, None), |scripts| {
            let (js, host) = scripts.MemoryDiagnostics();
            (js, Some(host))
        });
        let constraints = self.state.constraints.borrow();
        let mut seen = std::collections::HashSet::new();
        let unique_bitmap_capacity = constraints
            .images
            .iter()
            .filter_map(|image| {
                if let image_resource::PaintImageContent::Bitmap(pixels) = &image.content {
                    seen.insert(Arc::as_ptr(&pixels))
                        .then_some(pixels.capacity())
                } else {
                    None
                }
            })
            .sum();
        PageMemoryDiagnostics {
            loading: self.IsLoading(),
            dirty: self.state.dirty.get(),
            styles_resolved: self.state.styles_resolved.get(),
            pending_scroll_updates: self.state.scroll_updates.borrow().len(),
            frame_sequence: self.frame.as_ref().map(|f| f.sequence),
            recording_revision: self
                .CurrentPaintArtifact()
                .map(|artifact| artifact.recording_revision),
            dom: self
                .state
                .document
                .borrow()
                .GetDocument()
                .MemoryDiagnostics(),
            custom_properties: self
                .state
                .style_engine
                .borrow()
                .CustomPropertyMemoryDiagnostics(),
            javascript,
            host,
            resources: self.resources.MemoryDiagnostics(),
            catalog_images: constraints.images.len(),
            unique_bitmap_capacity,
        }
    }
    /// Publish HTMLImageElement's rendered auto-sizes width and run the same
    /// source reselection that ResizeObserver::OnResize triggers in Blink.
    /// The content-box width is unzoomed and rounded before it becomes the
    /// retained `last auto-sizes width`.
    fn ReconcileResponsiveAutoSizes(
        &mut self,
        mut output: LayoutPaintOutput,
    ) -> io::Result<LayoutPaintOutput> {
        let mut trace = browser_tracing::span("lifecycle", "Page.ReconcileResponsiveAutoSizes");
        // ResizeObserverController::GatherObservations visits observer
        // targets only. Without a lazy-loaded auto-sized <img> target
        // (Document::ObserveForLazyLoadedAutoSizedImg) Blink has no such
        // observer, so no fragment walk runs.
        if !self
            .state
            .document
            .borrow()
            .GetDocument()
            .HasLazyLoadedAutoSizedImages()
        {
            trace.set("iterations", 0.0);
            trace.set("responsive_images", 0.0);
            return Ok(output);
        }
        fn collect(
            document: &Document,
            node: PaintFragmentCursor<'_>,
            output: &mut Vec<(usize, f64)>,
        ) {
            if node.ViewKind() == FragmentKind::kBox && node.ViewReplacedContent().is_some() {
                if let Some(index) = document.FindNodeById(node.ViewNodeId()) {
                    if document.AllowsResponsiveImageAutoSizes(index) {
                        let zoom = node.paint.EffectiveZoom().max(f32::MIN_POSITIVE);
                        let width = (node.ViewContentSize().width as f32 / zoom).round() as f64;
                        output.push((index, width));
                    }
                }
            }
            for child in node.Children() {
                collect(document, child, output);
            }
        }

        // A candidate change can alter intrinsic geometry. Re-run the native
        // layout batch until the rounded content widths and selected sources
        // stabilize, with the document size as the same finite convergence
        // bound used by the container-query lifecycle.
        let limit = self.state.document.borrow().GetDocument().NodeCount() + 1;
        let mut observed = std::collections::HashSet::new();
        let mut iterations = 0usize;
        let mut responsive_images = 0usize;
        for _ in 0..limit {
            let widths = {
                let _trace = browser_tracing::span("lifecycle", "Page.CollectResponsiveAutoSizes");
                let owner = self.state.document.borrow();
                let mut widths = Vec::new();
                collect(owner.GetDocument(), output.PaintCursor(), &mut widths);
                widths
            };
            responsive_images = responsive_images.max(widths.len());
            if widths.is_empty() {
                break;
            }
            iterations += 1;
            let signature = widths
                .iter()
                .map(|(node, width)| (*node, width.to_bits()))
                .collect::<Vec<_>>();
            if !observed.insert(signature) {
                break;
            }
            let width_changed = {
                let _trace = browser_tracing::span("lifecycle", "Page.PublishResponsiveAutoSizes");
                let mut owner = self.state.document.borrow_mut();
                let document = owner.GetDocumentMut();
                widths.into_iter().fold(false, |changed, (node, width)| {
                    document.SetResponsiveImageAutoSize(node, width) || changed
                })
            };
            if !width_changed {
                break;
            }
            let selection_changed = {
                let _trace = browser_tracing::span("lifecycle", "Page.ReselectResponsiveImages");
                self.resources.QueueReferencedImages()?
            };
            if !selection_changed {
                break;
            }
            let _trace = browser_tracing::span("lifecycle", "Page.RelayoutResponsiveImages");
            self.state
                .LayoutWithContainerQueries(LayoutUpdateKind::Paint);
            output = self
                .state
                .layout_engine
                .borrow_mut()
                .TakePaintOutput()
                .expect("responsive auto-size relayout has fragments");
        }
        trace.set("iterations", iterations as f64);
        trace.set("responsive_images", responsive_images as f64);
        Ok(output)
    }

    /// Resolve the per-use SVG viewport after Layout has produced the replaced
    /// content rectangle and before Paint snapshots image resources. Blink
    /// creates SVGImageForContainer from this same LayoutImageResource size.
    /// Each distinct ContainerKey is retained as an immutable resource
    /// variant. Paint binds each image display item to its exact key, avoiding
    /// the resource-wide last-use-wins behavior.
    fn ConfigureDocumentImageContainers(
        &mut self,
        output: &mut LayoutPaintOutput,
    ) -> io::Result<bool> {
        fn collect(
            node: PaintFragmentCursor<'_>,
            images: &[layout::input::PaintImage],
            device_pixel_ratio: f64,
            preferred_color_scheme: u8,
            output: &mut BTreeMap<u64, Vec<image_resource::ContainerKey>>,
        ) {
            if let (Some(id), Some(replaced)) = (
                node.paint.style.image_resource_id,
                node.ViewReplacedContent().as_ref(),
            ) {
                if let Some(image) = images
                    .iter()
                    .find(|image| image.id == id)
                    .filter(|image| image.IsDocumentImage())
                {
                    let zoom = f64::from(node.paint.EffectiveZoom()).max(f64::MIN_POSITIVE);
                    let width = (replaced.size.width / zoom).round().max(1.0) as u32;
                    let height = (replaced.size.height / zoom).round().max(1.0) as u32;
                    let mut key =
                        image_resource::ContainerKey::new(width, height, zoom, device_pixel_ratio);
                    key.preferred_color_scheme = if node.paint.style.color_scheme_flags_is_normal {
                        preferred_color_scheme
                    } else {
                        u8::from(node.paint.style.used_dark_color_scheme)
                    };
                    key.preferred_color_scheme_is_explicit = true;
                    if let Some(current) = &image.document_container_key {
                        key.view_specification = current.view_specification.clone();
                    }
                    let keys = output.entry(id).or_default();
                    if !keys.contains(&key) {
                        keys.push(key);
                    }
                }
            }
            for child in node.Children() {
                collect(
                    child,
                    images,
                    device_pixel_ratio,
                    preferred_color_scheme,
                    output,
                );
            }
        }
        let preferred_color_scheme = match self.state.preferred_color_scheme.get() {
            PreferredColorScheme::kDark => 1,
            _ => 0,
        };
        let requirements = {
            let constraints = self.state.constraints.borrow();
            let mut requirements = BTreeMap::new();
            // Only document (SVG) images have per-container variants
            // (SVGImageForContainer); without one the walk finds nothing.
            if constraints.images.iter().any(|image| image.IsDocumentImage()) {
                collect(
                    output.PaintCursor(),
                    &constraints.images,
                    constraints.device_pixel_ratio,
                    preferred_color_scheme,
                    &mut requirements,
                );
            }
            requirements
        };
        let resources = self.resources.clone();
        let state = self.state.clone();
        let mut client = ResourceClient {
            state,
            scripts: self.scripts.as_deref_mut(),
        };
        for (resource_id, requirements) in requirements {
            for container in requirements {
                let already_published = self
                    .state
                    .constraints
                    .borrow()
                    .images
                    .iter()
                    .find(|image| image.id == resource_id)
                    .is_some_and(|image| match &image.content {
                        image_resource::PaintImageContent::Document(content) => {
                            content.exact_record_for(&container).is_some()
                        }
                        image_resource::PaintImageContent::Bitmap(_) => false,
                    });
                if already_published {
                    continue;
                }
                resources.ConfigureDocumentImageContainer(
                    resource_id,
                    container,
                    preferred_color_scheme,
                    &mut client,
                )?;
            }
        }
        // Paint-only fragment data can retain an earlier resource catalog.
        // Rebind the current immutable catalog even when every container was
        // already published (or there are only background-image clients).
        // Otherwise a resize back to a retained variant or a host preference
        // change would record against stale resources despite exact fulfillment.
        {
            let constraints = self.state.constraints.borrow();
            let resources = Arc::new(PaintResources {
                fonts: constraints.fonts.clone(),
                images: constraints.images.clone(),
                canvases: constraints.canvases.clone(),
                device_pixel_ratio: constraints.device_pixel_ratio,
                preferred_color_scheme: constraints.preferred_color_scheme,
                layout_zoom_factor: constraints.layout_zoom_factor,
                base_background_color: self.state.base_background_color.get(),
                paints_opaque_base_background: self.state.should_paint_base_background_color.get(),
                should_clear_document_background: self.state.should_clear_document_background.get(),
                viewport: constraints.viewport,
            });
            let mut trace = browser_tracing::span("resource", "Page.BindPaintResources");
            // The document's catalog is held once by the Layout output, as
            // ImageResourceContent/Font retain their identity when current.
            let previous = output.DocumentResources().cloned();
            let rebound = output.SetDocumentResources(resources.clone());
            trace.set("rebound", rebound as u8 as f64);
            drop(constraints);
            // ImageResourceObserver::ImageChanged and the frame-view resource
            // notifications reach their LayoutObjects; the caller's
            // RefreshOutputPrePaintInputs exports the resulting dirty bits.
            if let Some(previous) = previous.filter(|_| rebound) {
                let clients = self
                    .state
                    .layout_engine
                    .borrow_mut()
                    .Resources()
                    .NotifyPaintResourcesChanged(&[previous], &resources);
                trace.set("invalidated_clients", clients as f64);
            }
            return Ok(rebound);
        }
    }

    fn ConfigurePaintDocumentImageContainers(
        &mut self,
        output: &mut LayoutPaintOutput,
        requirements: Vec<(image_resource::ImageId, image_resource::ContainerKey)>,
    ) -> io::Result<bool> {
        if requirements.is_empty() {
            return Ok(false);
        }
        let preferred_color_scheme = match self.state.preferred_color_scheme.get() {
            PreferredColorScheme::kDark => 1,
            _ => 0,
        };
        let resources = self.resources.clone();
        let state = self.state.clone();
        let mut client = ResourceClient {
            state,
            scripts: self.scripts.as_deref_mut(),
        };
        let mut changed = false;
        for (resource_id, mut container) in requirements {
            if !container.preferred_color_scheme_is_explicit {
                container.preferred_color_scheme = preferred_color_scheme;
                container.preferred_color_scheme_is_explicit = true;
            }
            let already_published = self
                .state
                .constraints
                .borrow()
                .images
                .iter()
                .find(|image| image.id == resource_id)
                .is_some_and(|image| match &image.content {
                    image_resource::PaintImageContent::Document(content) => {
                        content.exact_record_for(&container).is_some()
                    }
                    image_resource::PaintImageContent::Bitmap(_) => false,
                });
            if already_published {
                continue;
            }
            changed |= resources.ConfigureDocumentImageContainer(
                resource_id,
                container,
                preferred_color_scheme,
                &mut client,
            )?;
        }
        let constraints = self.state.constraints.borrow();
        let resources = Arc::new(PaintResources {
            fonts: constraints.fonts.clone(),
            images: constraints.images.clone(),
            canvases: constraints.canvases.clone(),
            device_pixel_ratio: constraints.device_pixel_ratio,
            preferred_color_scheme: constraints.preferred_color_scheme,
            layout_zoom_factor: constraints.layout_zoom_factor,
            base_background_color: self.state.base_background_color.get(),
            paints_opaque_base_background: self.state.should_paint_base_background_color.get(),
            should_clear_document_background: self.state.should_clear_document_background.get(),
            viewport: constraints.viewport,
        });
        // Retained fragment-local Paint data can still point at the catalog
        // from before an earlier container mutation. Even when the requested
        // variant is already present in Page's current catalog, bind that
        // catalog and re-record once so the immutable artifact publishes the
        // exact record named by its display items.
        let previous = output.DocumentResources().cloned();
        let rebound = output.SetDocumentResources(resources.clone());
        drop(constraints);
        // Deliver the catalog change to its LayoutObject clients before the
        // caller republishes PrePaint inputs (RefreshOutputPrePaintInputs).
        if let Some(previous) = previous.filter(|_| rebound) {
            self.state
                .layout_engine
                .borrow_mut()
                .Resources()
                .NotifyPaintResourcesChanged(&[previous], &resources);
        }
        Ok(changed || rebound)
    }

    pub fn HasBlockingWheelListener(&self) -> bool {
        self.scripts
            .as_ref()
            .is_some_and(|scripts| scripts.HasBlockingWheelListener())
            || self
                .state
                .local_frames
                .get()
                .is_some_and(|frames| frames.HasBlockingWheelListener())
    }

    /// Resolve cancelable wheel listener targets against the current immutable
    /// Fragment tree. A document/window target covers the viewport; element
    /// targets include painted descendants because wheel events bubble through
    /// their DOM ancestors.
    pub fn BlockingWheelEventRegions(&self) -> ::interaction::BlockingWheelEventRegions {
        let frame_sequence = self.frame.as_ref().map_or(0, |frame| frame.sequence);
        let (child_targets, child_pending) = self.state.local_frames.get().map_or_else(
            || (Vec::new(), false),
            |frames| frames.BlockingWheelListenerTargets(),
        );
        let artifact = self
            .CurrentPaintArtifact()
            .filter(|_| !child_pending && !self.state.blocking_wheel_registry_dirty.get());
        let owner = self.state.document.borrow();
        let mut resolver = self.wheel_event_regions.borrow_mut();
        if child_pending || self.state.blocking_wheel_registry_dirty.get() {
            resolver.Invalidate();
        }
        let regions = resolver.ResolvePlatformWithLocalFrameTargets(
            frame_sequence,
            self.scripts
                .as_ref()
                .map_or_else(Vec::new, |scripts| scripts.BlockingWheelListenerTargets()),
            child_targets,
            owner.GetDocument(),
            artifact.as_deref().map(|artifact| &**artifact),
        );
        regions
    }

    // cpp: browser/browser.cc:610-737,1834-1856
    pub fn Create(
        loader: Rc<RefCell<dyn URLLoader>>,
        images: Rc<RefCell<dyn ImageDecoder>>,
        document_images: Rc<RefCell<dyn DocumentImageDecoder>>,
        constraints: ConstraintSpace,
        scripting: Option<ScriptEnvironment>,
        client: Option<Rc<RefCell<dyn PageClient>>>,
    ) -> Self {
        Self::CreateWithThreads(
            loader,
            images,
            document_images,
            constraints,
            scripting,
            client,
            PageThreads::default(),
        )
    }

    pub fn CreateWithThreads(
        loader: Rc<RefCell<dyn URLLoader>>,
        images: Rc<RefCell<dyn ImageDecoder>>,
        document_images: Rc<RefCell<dyn DocumentImageDecoder>>,
        constraints: ConstraintSpace,
        mut scripting: Option<ScriptEnvironment>,
        client: Option<Rc<RefCell<dyn PageClient>>>,
        threads: PageThreads,
    ) -> Self {
        // Keep the complete Page dependency graph visible in this one
        // composition root. The individual services remain independent and
        // communicate through their existing typed inputs and immutable
        // snapshots; Page only owns their lifetime and routing.
        let client = client.unwrap_or_else(|| Rc::new(RefCell::new(NullPageClient::default())));
        let mut decode_engine = decode::DecodeEngine::WithDecoders(images.clone(), document_images);
        decode_engine.SetImageBitmapRasterizer(Rc::new(
            crate::image_bitmap_service::RasterImageBitmapService,
        ));
        if threads.decode {
            let factory = images
                .borrow()
                .BackgroundFactory()
                .expect("threaded decode requires an image decoder factory");
            decode_engine.SetExecutor(threads.executor.clone(), factory);
        }
        if let Some(scripting) = &mut scripting {
            // Source: third_party/blink/renderer/bindings/core/v8/
            // script_streamer.cc:1104-1109. Streaming compilation is posted
            // to Chromium's worker pool independently of resource decoding.
            scripting.runtime.SetCompilationExecutor(
                threads
                    .scriptCompile
                    .then(|| threads.script_compile_executor.clone()),
            );
        }
        let decode_engine = Rc::new(decode_engine);
        let resource_engine = Rc::new(ResourceEngine::WithDecodeEngine(
            loader.clone(),
            decode_engine,
            String::new(),
        ));
        let document = Rc::new(DocumentEngine::new());
        let layout_engine =
            layout::LayoutEngine::new(&crate::CreateLayoutRegistry());
        let paint_engine = paint::PaintEngine::new();
        let compositor_executors = (
            if threads.raster {
                threads.compositor_executor.clone()
            } else {
                executor::inline()
            },
            if threads.raster {
                threads.raster_executor.clone()
            } else {
                executor::inline()
            },
        );
        let state = PageState::WithEngines(
            document,
            layout_engine,
            paint_engine,
            compositor_executors,
            constraints,
            client.clone(),
        );
        let resources = opening::CreateRootDocumentResources(&state, resource_engine.clone());
        let script_client: Rc<RefCell<dyn ScriptLoadClient>> =
            Rc::new(RefCell::new(ScriptClient(client, Rc::downgrade(&state))));
        let scripts = scripting.map(|environment| {
            Box::new(PageScripts::new(
                state.clone(),
                environment,
                script_client.clone(),
            ))
        });
        let open_engine = OpenEngine::new(
            resource_engine.clone(),
            scripts.as_ref().map(|scripts| scripts.Executor()),
        );
        state.InstallDocumentLoaderOwners(open_engine.DocumentOwners());
        let (interaction, form_controller) =
            opening::InstallRootDocumentInteraction(&state, &scripts);
        let begin_frame =
            begin_frame::PageBeginFrame::WithInputClock(state.compositor_executor.clone());
        let _ = state.document_input_queue.set(begin_frame.InputQueue());
        Self {
            previous_document_frame_sequence: 0,
            form_controller,
            threads,
            begin_frame,
            state,
            interaction,
            interaction_scroll_runtime: None,
            interaction_scroll_owners: None,
            open_engine,
            resources,
            scripts,
            script_client,
            frame: None,
            cursor_position: None,
            hover_state_dirty: false,
            wheel_scroll_active: false,
            external_compositor: false,
            processed_compositor_scroll_sequence: 0,
            cursor_hit_test: None,
            wheel_event_regions: RefCell::new(Default::default()),
            #[cfg(test)]
            cursor_hit_test_queries: 0,
            active: true,
            active_state_published: false,
            selection_revision: 0,
            document_selection: None,
            caret_focused_node: None,
            editing_paint_dirty: false,
            document_image_environment_dirty: false,
        }
    }
    pub fn ResourceEngine(&self) -> Rc<ResourceEngine> {
        self.open_engine.Resources()
    }

    /// EventHandler is created after the first prepared LayoutView, matching
    /// Blink's frame lifecycle. The resulting owner is retained for the Page
    /// domain and reused by every later wheel transaction.
    fn EnsureInteractionScrollOwners(&mut self) {
        let layout_viewport_identity = self
            .state
            .layout_engine
            .borrow()
            .LayoutViewportLogicalScrollArea()
            .expect("prepared local root owns a layout viewport ScrollableArea")
            .Identity();
        if self
            .interaction_scroll_owners
            .as_ref()
            .is_some_and(|owners| owners.LayoutViewportIdentity() == layout_viewport_identity)
        {
            return;
        }
        debug_assert!(self.state.layout_engine.borrow().Output().is_some());
        let input = self.interaction.SourceInputOwners();
        let autoscroll = {
            let mut slot = input.autoscroll_services.borrow_mut();
            slot.get_or_insert_with(|| {
                interaction::adapter::page_mouse_autoscroll::PageMouseAutoscroll::new(
                    input.autoscroll.clone(),
                    Rc::new(mouse_autoscroll::PageAutoscrollStop(Rc::downgrade(
                        &self.state,
                    ))),
                )
            })
            .clone()
        };
        *self.state.mouse_autoscroll.borrow_mut() = Rc::downgrade(&autoscroll);
        let (owners, runtime) =
            local_frame_scroll::InteractionScrollOwner::Root(Rc::downgrade(&self.state))
                .Assemble(&self.interaction);
        *self.state.scrollbar_scroll_runtime.borrow_mut() = Rc::downgrade(&runtime);
        self.interaction_scroll_runtime = Some(runtime);
        self.interaction_scroll_owners = Some(owners);
    }
    /// Select whether ViewPainter records the main-frame base background.
    ///
    /// This is the embedding form of LocalFrameView::
    /// ShouldPaintBaseBackgroundColor/BaseBackgroundColor. Ordinary Pages keep
    /// the default opaque white root group. Browser-owned Views can disable it
    /// when their parent compositor supplies the exact backdrop.
    pub fn SetShouldPaintBaseBackgroundColor(&mut self, should_paint: bool) {
        if self
            .state
            .should_paint_base_background_color
            .replace(should_paint)
            == should_paint
        {
            return;
        }
        self.InvalidateBaseBackgroundColor();
    }
    /// Set LocalFrameView's embedder-provided base background color.
    pub fn SetBaseBackgroundColor(&mut self, color: Color) {
        if self.state.base_background_color.replace(color) == color {
            return;
        }
        self.InvalidateBaseBackgroundColor();
    }
    /// Set Settings::GetShouldClearDocumentBackground for ViewPainter.
    pub fn SetShouldClearDocumentBackground(&mut self, should_clear: bool) {
        if self
            .state
            .should_clear_document_background
            .replace(should_clear)
            == should_clear
        {
            return;
        }
        self.InvalidateBaseBackgroundColor();
    }
    fn InvalidateBaseBackgroundColor(&mut self) {
        // LocalFrameView::SetBaseBackgroundColor invalidates the complete
        // background and schedules a visual update. The root backdrop is a
        // display item, so this cannot be a resource-only retained update.
        self.state.document.borrow_mut().InvalidatePaint();
        self.state.paint_resources_dirty.set(true);
        self.state.scroll_only.set(false);
        self.state.dirty.set(true);
        self.state
            .InvalidatePreparedArtifact();
        self.RequestBeginFrameIfNeeded();
    }
    /// Install the one-shot token backing a CropTarget and schedule the full
    /// paint update required to publish its region-capture bounds.
    pub fn SetRegionCaptureCropId(&mut self, node_id: u64, crop_id: [u64; 2]) -> bool {
        let changed = self
            .state
            .document
            .borrow_mut()
            .SetRegionCaptureCropId(node_id, crop_id);
        if changed {
            self.state.paint_resources_dirty.set(true);
            self.state.scroll_only.set(false);
            self.state.dirty.set(true);
            self.state
                .InvalidatePreparedArtifact();
            self.RequestBeginFrameIfNeeded();
        }
        changed
    }
    /// Install one embedding-owned tracked-element descriptor. Feature keys
    /// are unique per element, matching Element::SetTrackedElementSubRect.
    pub fn SetTrackedElementSubRect(
        &mut self,
        node_id: u64,
        tracked: layout::input::TrackedElementSubRect,
    ) -> bool {
        let changed = self
            .state
            .document
            .borrow_mut()
            .SetTrackedElementSubRect(node_id, tracked);
        if changed {
            self.state.paint_resources_dirty.set(true);
            self.state.scroll_only.set(false);
            self.state.dirty.set(true);
            self.state
                .InvalidatePreparedArtifact();
            self.RequestBeginFrameIfNeeded();
        }
        changed
    }
    /// Remove one tracked-element feature and republish the fragment without
    /// its compositor metadata.
    pub fn ClearTrackedElementSubRect(&mut self, node_id: u64, feature: i32) -> bool {
        let changed = self
            .state
            .document
            .borrow_mut()
            .ClearTrackedElementSubRect(node_id, feature);
        if changed {
            self.state.paint_resources_dirty.set(true);
            self.state.scroll_only.set(false);
            self.state.dirty.set(true);
            self.state
                .InvalidatePreparedArtifact();
            self.RequestBeginFrameIfNeeded();
        }
        changed
    }
    /// Inject an embedding-owned platform font service. Discovery/opening run
    /// at the host boundary, never inside FontEngine or Layout computation.
    pub fn SetSystemFontProvider(&mut self, provider: Box<dyn font_engine::SystemFontProvider>) {
        self.state
            .system_fonts
            .borrow_mut()
            .SetProvider(provider, &mut self.state.constraints.borrow_mut());
        self.state.InvalidateLayoutOutput();
        // A different platform font catalog invalidates text measurement, not
        // the DOM-to-LayoutObject attachment tree.
        self.state.document.borrow_mut().InvalidateLayoutGeometry();
        self.state.dirty.set(true);
        self.state.scroll_only.set(false);
        self.RequestBeginFrameIfNeeded();
    }
    /// Called by the embedding after installed fonts or platform font settings
    /// change. Existing frames retain their immutable bytes; the next lifecycle
    /// re-resolves only the fonts actually requested by the document.
    pub fn SystemFontsChanged(&mut self) {
        self.state
            .system_fonts
            .borrow_mut()
            .EnvironmentChanged(&mut self.state.constraints.borrow_mut());
        self.state.InvalidateLayoutOutput();
        self.state.document.borrow_mut().InvalidateLayoutGeometry();
        self.state.dirty.set(true);
        self.state.scroll_only.set(false);
        self.RequestBeginFrameIfNeeded();
    }
    /// Recover from a compositor-side retained recording rejection by asking
    /// Paint for one new artifact. The active compositor tree remains valid
    /// until that replacement reaches raster.
    pub fn RecoverRetainedRaster(&mut self) {
        self.state.document.borrow_mut().InvalidatePaint();
        self.state.scroll_only.set(false);
        self.state.dirty.set(true);
        self.state
            .InvalidatePreparedArtifact();
        self.RequestBeginFrameIfNeeded();
    }
    pub fn SetPreferredColorScheme(&mut self, preference: PreferredColorScheme) {
        if let Some(scripts) = &self.scripts {
            scripts.SetPreferredColorScheme(preference);
        }
        if self.state.preferred_color_scheme.replace(preference) != preference {
            {
                let mut constraints = self.state.constraints.borrow_mut();
                constraints.preferred_color_scheme =
                    u8::from(preference == PreferredColorScheme::kDark);
                let invalidated = self
                    .state
                    .layout_engine
                    .borrow_mut()
                    .Resources()
                    .NotifyDocumentImageEnvironmentChanged(&constraints.images);
                self.document_image_environment_dirty |= invalidated != 0;
            }
            self.state.InvalidateLayoutOutput();
            self.state.InvalidateStyle();
        }
    }
    pub fn SetForcedColors(&mut self, forced_colors: ForcedColors) {
        if let Some(scripts) = &self.scripts {
            scripts.SetForcedColors(forced_colors);
        }
        if self.state.forced_colors.replace(forced_colors) != forced_colors {
            self.state.InvalidateLayoutOutput();
            self.state.InvalidateStyle();
        }
    }
    // cpp: browser/browser.cc:740-799
    /// Start navigation without waiting for headers, body or parser completion.
    /// The host event loop drives `RunTask`; later failures reach DidFail and
    /// that task's result. IsLoading includes pending document resources/scripts.
    pub fn EnableDiagnosticReadAhead(&mut self, scripting: bool) {
        self.open_engine.EnableDiagnosticReadAhead(scripting);
    }
    pub fn Open(&mut self, url: &str, chunk_size: usize, token_budget: usize) -> io::Result<()> {
        self.OpenRequest(
            &URLRequest {
                url: url.into(),
                ..Default::default()
            },
            chunk_size,
            token_budget,
        )
    }
    pub fn OpenRequest(
        &mut self,
        request: &URLRequest,
        chunk_size: usize,
        token_budget: usize,
    ) -> io::Result<()> {
        let mut trace = browser_tracing::span("navigation", "OpenRequest");
        trace.set("chunk_size", chunk_size as f64);
        trace.set("token_budget", token_budget as f64);
        self.SaveHistoryState();
        let result = self.OpenDocument(request, chunk_size, token_budget);
        trace.set("failed", result.is_err() as u8 as f64);
        if let Err(error) = &result {
            let mut adapter = PageOpenAdapter::new(
                &mut self.state,
                &mut self.resources,
                &mut self.interaction,
                &mut self.form_controller,
                &mut self.scripts,
                &mut self.script_client,
            );
            self.open_engine.Fail(&mut adapter, error);
        }
        result
    }
    /// Serialize the loader's live HistoryItem on its document sequence before
    /// the host continues navigation. FormController remains the lazy owner.
    pub fn SaveHistoryState(&self) {
        self.state.SaveHistoryState();
    }
    fn OpenDocument(
        &mut self,
        request: &URLRequest,
        chunk_size: usize,
        token_budget: usize,
    ) -> io::Result<()> {
        self.open_engine.PrepareForDocumentNavigation();
        self.open_engine
            .Open(
                request.clone(),
                DocumentLoadBudget {
                    body_bytes: chunk_size,
                    parser_tokens: token_budget,
                },
            )
            .map(|_| ())
    }
    /// Drive the ordinary OpenEngine path to completion for serialized fixtures.
    /// This method supplies only the host loop; URL loading, parsing, resource
    /// readiness and load completion remain owned by OpenEngine.
    #[cfg(test)]
    pub(crate) fn OpenSynchronously(
        &mut self,
        url: &str,
        chunk_size: usize,
        token_budget: usize,
    ) -> io::Result<()> {
        self.Open(url, chunk_size, token_budget)?;
        while self.open_engine.IsLoading() {
            self.RunTask()?;
        }
        Ok(())
    }
    pub fn IsLoading(&self) -> bool {
        self.open_engine.IsLoading()
            || self
                .state
                .local_frames
                .get()
                .unwrap()
                .HasPendingNavigations()
    }
    /// A body-stream failure after parser initialization keeps this document.
    /// The host can report the failure without replacing its received content.
    pub fn HasCommittedDocument(&self) -> bool {
        self.open_engine.HasCommittedDocument()
    }
    pub fn LoadingFailure(&self) -> Option<&str> {
        self.open_engine.LoadingFailure()
    }
    pub fn OpenStatus(&self) -> OpenStatus {
        self.open_engine.Status()
    }
    pub fn Threads(&self) -> &PageThreads {
        &self.threads
    }

    /// Select the native Page runtime's impl-executor compositor. The runtime
    /// routes compositor disposition back through the same InputEvent, while
    /// direct embedders continue using Page's inline PaintInto pipeline.
    pub fn SetExternalCompositor(&mut self, external: bool) {
        // Media: webmediaplayer_impl.cc / video_frame_submitter.cc select the
        // embedder-owned Viz boundary before this producer creates cc state.
        assert!(
            !self.state.HasMainCompositor()
                && self.state.synchronous_presenter.borrow().is_none()
                && self.state.outbound_child_frames.borrow().is_empty()
                && self.state.video_surfaces.borrow().is_empty(),
            "presentation ownership must be selected before compositor initialization"
        );
        self.state.external_presentation.set(external);
        self.external_compositor = external;
    }

    /// Creates the Page's main compositor half on this (document) thread and
    /// starts its impl half on the compositor thread through `to_impl`
    /// (ProxyMain::Start, proxy_main.cc:898-914: posts Initialize with the
    /// ProxyImpl construction values and waits for it).
    pub fn StartExternalCompositor(
        &mut self,
        width: u32,
        height: u32,
        scale: f64,
        to_impl: MainToImplQueue,
        compositor_task_runner: CompositorTaskRunner,
        compositor_input_blocking_task_runner: CompositorInputBlockingTaskRunner,
    ) -> io::Result<()> {
        assert!(self.external_compositor && self.state.external_presentation.get());
        assert!(
            !compositor_task_runner.IsInProcess()
                && !compositor_input_blocking_task_runner.IsInProcess(),
            "the embedding posts to its compositor thread"
        );
        *self.state.compositor_input_blocking_task_runner.borrow_mut() =
            Some(compositor_input_blocking_task_runner);
        // WidgetInputHandlerManager posts each main-thread acknowledgement
        // from the event's callback (DidHandleInputEventSentToMain,
        // widget_input_handler_manager.cc:1171-1230).
        let acks = compositor_task_runner.clone();
        self.begin_frame
            .InputQueue()
            .SetDidHandleInputEventSentToMain(Arc::new(move |ack| {
                PostMainThreadEventAck(&acks, ack)
            }));
        let in_process = std::mem::replace(
            &mut *self.state.compositor_task_runner.borrow_mut(),
            compositor_task_runner,
        );
        while let Some(task) = in_process.TakeTask() {
            self.state.PostCompositorTask(task);
        }
        let viewport = PageDomainServices::InitialCompositorViewport(width, height, scale)?;
        let init = self
            .state
            .CreateMainCompositor(PageDomainServices::CompositorConfig(viewport), to_impl)?;
        self.state.StartMainCompositor(Some(init));
        self.state.SetMainCompositorViewport(viewport)
    }

    /// ProxyMain::SetNeedsAnimate (Blink's ChromeClient::ScheduleAnimation
    /// -> LayerTreeHost::SetNeedsAnimate).
    pub fn SetNeedsCompositorAnimate(&self) {
        self.state.SetNeedsMainCompositorAnimate();
    }

    /// The Send + Sync ProxyMain request side, for the embedding's frame
    /// source (ScheduleAnimation from any document task).
    pub fn CompositorAnimateRequests(&self) -> Option<Arc<dyn Fn() + Send + Sync>> {
        let requests = self.state.MainCompositorRequests()?;
        Some(Arc::new(move || requests.SetNeedsAnimate(false)))
    }

    /// ProxyMain::BeginMainFrame of the external runtime, one synchronous
    /// call (proxy_main.cc:174-580): the impl deltas and mutator events are
    /// applied, `before_lifecycle` and the Page lifecycle run, the resulting
    /// PaintArtifact is pushed (LocalFrameView::PushPaintArtifactToCompositor)
    /// at `width`x`height`@`scale`, and the commit or abort is posted to
    /// the impl thread.
    pub fn RunExternalBeginMainFrame(
        &mut self,
        state: Box<main_compositor::BeginMainFrameState>,
        args: foundation::begin_frame::BeginFrameArgs,
        width: u32,
        height: u32,
        scale: f64,
        before_lifecycle: &mut dyn FnMut(&mut Page),
    ) -> io::Result<MainFrameResult> {
        // The domain outlives a document replacement inside the lifecycle.
        let main = self.state.domain.clone();
        if !main.HasMainCompositor() {
            return Err(io::Error::other("external compositor is not started"));
        }
        let mut failure = None;
        let result = main.BeginMainCompositorFrame(state, &mut |effects| {
            let lifecycle = (|| -> io::Result<()> {
                {
                    let _trace =
                        browser_tracing::span("lifecycle", "Page.ApplyCompositorWheelEffects");
                    self.ApplyCompositorWheelEffects(effects)?;
                }
                {
                    let _trace = browser_tracing::span("lifecycle", "Page.BeforeLifecycle");
                    before_lifecycle(self);
                }
                self.UpdateRendering(args)?;
                // LocalFrameView::PushPaintArtifactToCompositor
                // (local_frame_view.cc:3253,3348-3351) hands
                // PaintArtifactCompositor::Update the PaintController's
                // artifact; Layerize retains that same Arc, nothing else does.
                let artifact = self.CurrentPaintArtifact().map(|artifact| Arc::clone(&artifact));
                if let Some(artifact) = artifact {
                    let _trace =
                        browser_tracing::span("lifecycle", "Page.PushPaintArtifactToCompositor");
                    main.UpdateMainCompositorPaintArtifact(&artifact, width, height, scale)?;
                }
                Ok(())
            })();
            if let Err(error) = lifecycle {
                failure = Some(error);
            }
        });
        if let Some(error) = failure {
            return Err(error);
        }
        let _trace = browser_tracing::span("lifecycle", "Page.ApplyMainCompositorEffects");
        let effects = main.TakeMainCompositorEffects();
        self.ApplyCompositorWheelEffects(effects)?;
        Ok(result)
    }

    /// The layout zoom of the artifact last pushed to the main half (the
    /// impl side's InputRouter widget scale).
    pub fn CompositorInputScale(&self) -> f64 {
        self.state.MainCompositorInputScale()
    }

    /// A ProxyImpl -> ProxyMain task other than BeginMainFrame.
    pub fn RunCompositorImplTask(&mut self, task: main_compositor::ImplToMainTask) -> io::Result<()> {
        if !self.state.HasMainCompositor() {
            return Ok(());
        }
        self.state.RunMainCompositorImplTask(task);
        let effects = self.state.TakeMainCompositorEffects();
        for effect in &effects {
            // FinishCommit's time is in the impl half's clock.
            if let CompositorMainThreadEffect::MainFrameCommitCompleted { commit_finish_time } =
                effect
            {
                if let Some(time) = self
                    .state
                    .external_compositor_clock
                    .borrow()
                    .and_then(|clock| clock.Instant(*commit_finish_time))
                {
                    self.EventTimingDidCommit(time);
                }
            }
        }
        self.ApplyCompositorWheelEffects(
            effects
                .into_iter()
                .filter(|effect| {
                    !matches!(
                        effect,
                        CompositorMainThreadEffect::MainFrameCommitCompleted { .. }
                    )
                })
                .collect(),
        )
    }

    pub fn BindExternalCompositorPresentationClock(&self, clock: PageCompositorClockMapping) {
        *self.state.external_compositor_clock.borrow_mut() = Some(clock);
    }

    pub fn NotifyPerformancePageVisibilityChanged(
        &self,
        visible: bool,
        changed_at: std::time::Instant,
    ) {
        self.state.performance_visible.set(visible);
        self.state
            .performance_visibility_changed_at
            .set(Some(changed_at));
        if !visible {
            let time = self
                .state
                .CompositorClock().InputTime(changed_at);
            self.state
                .ForEachWindowPerformance(|performance| performance.PageHidden(time));
        }
    }

    /// Transfers delegated media frames produced while applying document
    /// resource mutations. The external runtime submits these child Surfaces
    /// immediately before the parent CompositorFrame that references them.
    pub fn TakeChildCompositorFrames(&self) -> Vec<PageChildCompositorFrame> {
        self.state.TakeChildFrames()
    }

    pub fn SetLoadingWakeCallback(&mut self, callback: URLWakeCallback) {
        *self
            .state
            .local_frames
            .get()
            .unwrap()
            .loading_wake
            .borrow_mut() = Some(callback.clone());
        self.open_engine.SetWakeCallback(callback);
    }
    pub fn StopLoading(&mut self) {
        self.open_engine.Stop();
    }
    /// OpenEngine owns navigation admission and completion state. Page adapts
    /// the selected parser/script implementation and routes lifecycle effects.
    fn PumpLoading(&mut self) -> io::Result<()> {
        let mut trace = browser_tracing::span("lifecycle", "Page.PumpLoading");
        trace.set("loading", self.open_engine.IsLoading() as u8 as f64);
        if !self.open_engine.IsLoading() {
            return Ok(());
        }
        let previous_document = self.state.document.clone();
        let mut adapter = PageOpenAdapter::new(
            &mut self.state,
            &mut self.resources,
            &mut self.interaction,
            &mut self.form_controller,
            &mut self.scripts,
            &mut self.script_client,
        );
        let result = self.open_engine.Advance(&mut adapter);
        if !Rc::ptr_eq(&previous_document, &self.state.document) {
            self.DidInstallNewDocument();
        }
        result?;
        Ok(())
    }

    /// Frame-owned adapters carry no retained pointer into the old Document.
    /// Their next use binds the replacement's canonical Layout/Interaction.
    fn DidInstallNewDocument(&mut self) {
        self.interaction_scroll_runtime = None;
        self.interaction_scroll_owners = None;
        if let Some(frame) = self.frame.take() {
            self.previous_document_frame_sequence = frame.sequence;
        }
        self.cursor_hit_test = None;
        self.hover_state_dirty = self.cursor_position.is_some();
        self.wheel_scroll_active = false;
        *self.wheel_event_regions.borrow_mut() = Default::default();
        self.selection_revision = 0;
        self.document_selection = None;
        self.caret_focused_node = None;
        self.editing_paint_dirty = false;
        self.document_image_environment_dirty = false;
    }
    fn ProcessFontLoadingEffects(&mut self) {
        loop {
            let font_loading = {
                let owner = self.state.document.borrow();
                owner.GetDocument().FontLoading()
            };
            let effects = font_loading.TakeEffects();
            if effects.is_empty() {
                break;
            }
            for effect in effects {
                match effect {
                    dom::font_loading::FontLoadingEffect::Task { run, .. } => run(),
                    dom::font_loading::FontLoadingEffect::CheckStyleAndLayout { set } => {
                        self.state.EnsureLayoutOutput();
                        set.CompleteLayoutCheck();
                    }
                    dom::font_loading::FontLoadingEffect::FaceDescriptorChanged { face } => {
                        self.resources.FontFaceMembershipChanged(&face);
                    }
                    dom::font_loading::FontLoadingEffect::ConsoleWarning(message) => {
                        browser_tracing::instant_with_text(
                            "loading",
                            "FontLoadingWarning",
                            &message,
                            &[],
                        );
                    }
                    dom::font_loading::FontLoadingEffect::SetEvent { .. }
                    | dom::font_loading::FontLoadingEffect::LinkPreloadEvent { .. }
                    | dom::font_loading::FontLoadingEffect::SetReady { .. } => {}
                }
            }
        }
    }
    pub fn IsRenderingReady(&self) -> bool {
        // The Page lifecycle owns the first-content barrier. Parser/resource
        // scheduling reports Blink-style render-blocking sheets discovered
        // before body insertion; renderer and host presentation never infer
        // loading policy. Once the first frame exists, later sheets restyle
        // normally without freezing input, matching rendering_has_begun_.
        if self.frame.is_some() {
            return true;
        }
        if !self.open_engine.HasCommittedDocument() {
            return false;
        }
        if !self
            .state
            .document
            .borrow()
            .GetDocument()
            .HaveRenderBlockingResourcesLoaded()
        {
            return false;
        }
        self.state.document.HasRenderableRoot()
    }
    // cpp: browser/browser.cc:1646-1680
    fn ParseDocument(
        &mut self,
        source: &str,
        chunk_size: usize,
        token_budget: usize,
    ) -> io::Result<()> {
        self.open_engine.PrepareForDocumentNavigation();
        let previous_document = self.state.document.clone();
        let mut adapter = PageOpenAdapter::new(
            &mut self.state,
            &mut self.resources,
            &mut self.interaction,
            &mut self.form_controller,
            &mut self.scripts,
            &mut self.script_client,
        );
        let result = self
            .open_engine
            .ParseSuppliedDocument(&mut adapter, source, chunk_size, token_budget)
            .map(|_| ());
        if !Rc::ptr_eq(&previous_document, &self.state.document) {
            self.DidInstallNewDocument();
        }
        result
    }
    // cpp: browser/browser.cc:922-929
    pub fn Apply(&mut self, mutation: PageMutation) -> io::Result<()> {
        if let PageMutation::DOMMutation(m) = &mutation {
            if m.mutation_type == DOMMutationType::kParseDocument {
                self.ParseDocument(&m.value, 16 * 1024, 4096)?;
                if let Some(scripts) = &self.scripts {
                    scripts.FlushTasks();
                }
                self.FinishSuppliedDocumentResources()?;
                return self.UpdateFrameIfNeeded();
            }
        }
        if let Some(scripts) = &mut self.scripts {
            scripts.ApplyMutation(mutation)?;
        } else {
            self.state.ApplyMutation(mutation);
        }
        self.UpdateFrameIfNeeded()
    }
    pub fn Evaluate(&mut self, source: &str, source_name: &str) -> io::Result<JavaScriptResult> {
        self.EvaluateWithDisposition(source, source_name, false)
    }
    /// Execute an Inspector Runtime.evaluate task. Its direct exception is the
    /// protocol result, not an uncaught page-script error; asynchronous
    /// exceptions created by its microtask checkpoint remain page errors.
    pub fn EvaluateForInspector(
        &mut self,
        source: &str,
        source_name: &str,
    ) -> io::Result<JavaScriptResult> {
        self.EvaluateWithDisposition(source, source_name, true)
    }
    fn EvaluateWithDisposition(
        &mut self,
        source: &str,
        source_name: &str,
        inspector: bool,
    ) -> io::Result<JavaScriptResult> {
        // Runtime.evaluate is a browser task boundary. Admit ready network and
        // decode completions before script observes document.fonts/images;
        // otherwise a DevTools-driven readiness loop can see "loading"
        // forever while the completed resource waits for an unrelated wake.
        self.PumpLoading()?;
        let Some(scripts) = &mut self.scripts else {
            return Ok(JavaScriptResult::Failure(
                javascript::javascript_runtime::JavaScriptExceptionKind::kRuntimeError,
                "No JavaScript runtime",
                "",
                0,
                0,
            ));
        };
        let result = if inspector {
            scripts.EvaluateForInspector(source, source_name)?
        } else {
            scripts.Evaluate(source, source_name)?
        };
        self.UpdateFrameAndStartVisibleDeferredImages()?;
        Ok(result)
    }
    /// Execute one complete Page task turn, including its microtask checkpoint
    /// and any lifecycle update caused by that task. Task selection and wake
    /// delivery remain the responsibility of the embedding event loop.
    pub fn RunTask(&mut self) -> io::Result<()> {
        self.RunTaskTurn(0.0)
    }

    /// Drive Page tasks for a bounded interval. This is an explicit embedding
    /// helper for synchronous document bootstrap and tests; native operation
    /// uses `RunTask`, one event-loop turn at a time.
    pub fn RunFor(&mut self, budget: std::time::Duration) -> io::Result<()> {
        self.RunTaskTurn(budget.as_secs_f64() * 1000.0)
    }

    fn RunTaskTurn(&mut self, milliseconds: f64) -> io::Result<()> {
        let mut trace = browser_tracing::span("lifecycle", "Page.RunTask");
        trace.set("budget_ms", milliseconds);
        trace.set("loading", self.open_engine.IsLoading() as u8 as f64);
        let prologue_trace = browser_tracing::span("lifecycle", "Page.TaskTurnPrologue");
        self.state.RunTextOverflowLayoutInvalidationTasks();
        self.state.local_frames.get().unwrap().RunIntersectionTask();
        self.state.local_frames.get().unwrap().RunFormStateTask();
        self.RunGestureTimeouts()?;
        if self.begin_frame.source.is_none() {
            self.FlushFrameInputs()?;
        }
        self.open_engine.BeginTaskTurn(milliseconds);
        drop(prologue_trace);
        let result = (|| {
            self.state
                .document
                .borrow()
                .GetDocument()
                .FireDueRenderBlockingTimers(std::time::Instant::now());
            let started = browser_tracing::profile_input()
                .then(std::time::Instant::now);
            let was_loading = self.open_engine.IsLoading();
            self.PumpLoading()?;
            // A clean frame does not imply that late image/font requests have no
            // work. Continue their transport/completion lifecycle after window load.
            if !was_loading
                && (self.resources.HasPendingStyleSheets()
                    || self.resources.HasPendingImages()
                    || self.resources.HasPendingFonts()
                    || self.resources.HasPendingRawResources())
            {
                // Resource completion is an ordinary Page task. Keep the
                // same one-completion admission used while the document is
                // loading; a late image/font must not turn one wake into an
                // unbounded decode batch.
                self.ResolveStylesAndLoadResources(1)?;
            }
            let housekeeping_trace =
                browser_tracing::span("lifecycle", "Page.TaskTurnHousekeeping");
            self.ProcessFontLoadingEffects();
            let loading_done = started.map(|start| start.elapsed());
            let ran_popup_task = self.RunDuePopupTask(std::time::Instant::now())?;
            drop(housekeeping_trace);
            let run_host_tasks = if !ran_popup_task {
                if let Some(scripts) = &mut self.scripts {
                    self.open_engine.RunScriptTaskTurn(
                        milliseconds,
                        |scheduler, executor, limit| {
                            scripts.PumpModuleTasks(scheduler, executor, limit)
                        },
                    )?
                } else {
                    false
                }
            } else {
                false
            };
            if run_host_tasks {
                self.scripts
                    .as_deref_mut()
                    .expect("host tasks require a script environment")
                    .RunHostTaskTurn(milliseconds)?;
            }
            // Hosts without a BeginFrameSource treat this task turn as their
            // rendering opportunity. Native hosts dispatch the same queue in
            // Page::UpdateRendering before rAF.
            if self.begin_frame.source.is_none() {
                if !self.state.scroll_event_targets.borrow().is_empty() {
                    self.state.ApplyPendingScrollUpdates(self.frame.as_mut());
                }
                if let Some(scripts) = &mut self.scripts {
                    scripts.DispatchPendingScrollEvents();
                }
            }
            {
                let _trace = browser_tracing::span("lifecycle", "Page.AdvanceNavigations");
                self.state
                    .local_frames
                    .get()
                    .unwrap()
                    .AdvanceNavigations(milliseconds)?;
            }
            let tasks_done = started.map(|start| start.elapsed());
            self.UpdateFrameAndStartVisibleDeferredImages()?;
            if let Some(start) = started {
                if start.elapsed() >= std::time::Duration::from_millis(16) {
                    eprintln!(
                        "page-task-profile loading_ms={:.3} script_tasks_ms={:.3} lifecycle_ms={:.3} total_ms={:.3}",
                        loading_done.unwrap().as_secs_f64() * 1000.0,
                        (tasks_done.unwrap() - loading_done.unwrap()).as_secs_f64() * 1000.0,
                        (start.elapsed() - tasks_done.unwrap()).as_secs_f64() * 1000.0,
                        start.elapsed().as_secs_f64() * 1000.0
                    );
                }
            }
            Ok(())
        })();
        // IO failures must not leave a one-turn allowance attached to later
        // Evaluate/Apply/Dispatch or the explicit synchronous parser adapter.
        self.open_engine.EndTaskTurn();
        {
            let _trace = browser_tracing::span("lifecycle", "Page.UpdateCursor");
            self.UpdateCursor();
        }
        result
    }
    /// Earliest ordinary Window task deadline. Document/resource transports
    /// wake the owner through `SetLoadingWakeCallback`; this value covers
    /// timers and the remaining poll-based Window task sources.
    pub fn NextTaskDeadline(&self, now: std::time::Instant) -> Option<std::time::Instant> {
        if !self
            .state
            .pending_text_overflow_layout_invalidations
            .borrow()
            .is_empty()
        {
            return Some(now);
        }
        let script_deadline = self
            .scripts
            .as_ref()
            .and_then(|scripts| scripts.NextTaskDeadline(now));
        let render_blocking_deadline = self
            .state
            .document
            .borrow()
            .GetDocument()
            .NextRenderBlockingTimerDeadline();
        let caret_deadline = self.state.layout_editing.caret.borrow().NextBlinkDeadline();
        [
            script_deadline,
            self.NextGestureDeadline(now),
            render_blocking_deadline,
            caret_deadline,
            self.state.popup_tasks.NextDeadline(),
            self.state
                .local_frames
                .get()
                .unwrap()
                .intersection_deadline
                .get(),
        ]
        .into_iter()
        .flatten()
        .min()
    }
    pub fn ResizeViewport(&mut self, width: f64, height: f64, device_scale: f64) -> io::Result<()> {
        if !width.is_finite()
            || !height.is_finite()
            || width <= 0.0
            || height <= 0.0
            || !device_scale.is_finite()
            || device_scale <= 0.0
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid host viewport",
            ));
        }
        // Source: third_party/blink/renderer/core/frame/web_frame_widget_impl.cc
        // SetVisualProperties. Identical visual properties do not schedule a
        // style/layout lifecycle; the browser host may resend an acknowledged
        // child viewport while only a sibling or the root size changes.
        // Source: WebViewImpl::SetZoomFactorForDeviceScaleFactor and
        // WebFrameWidgetImpl::SetZoomInternal. Preserve the browser/CSS zoom
        // component while replacing the hardware DSF component.
        let (old_device_scale, old_layout_zoom) = {
            let constraints = self.state.constraints.borrow();
            (
                constraints.device_pixel_ratio,
                constraints.layout_zoom_factor,
            )
        };
        let page_zoom = old_layout_zoom / old_device_scale.max(f64::MIN_POSITIVE);
        let layout_zoom = device_scale * page_zoom;
        let unchanged = {
            let constraints = self.state.constraints.borrow();
            constraints.available_size.width.to_bits() == width.to_bits()
                && constraints.available_size.height.to_bits() == height.to_bits()
                && constraints.device_pixel_ratio.to_bits() == device_scale.to_bits()
                && constraints.layout_zoom_factor.to_bits() == layout_zoom.to_bits()
        };
        if unchanged {
            return Ok(());
        }
        let device_scale_changed = old_device_scale.to_bits() != device_scale.to_bits();
        let layout_zoom_changed = old_layout_zoom.to_bits() != layout_zoom.to_bits();
        if device_scale_changed || layout_zoom_changed {
            self.state.document.borrow_mut().InvalidateLayoutGeometry();
        }
        {
            let mut constraints = self.state.constraints.borrow_mut();
            constraints.device_pixel_ratio = device_scale;
            constraints.layout_zoom_factor = layout_zoom;
        }
        if device_scale_changed || layout_zoom_changed {
            let constraints = self.state.constraints.borrow();
            let metric_scale = constraints.device_pixel_ratio
                / constraints.layout_zoom_factor.max(f64::MIN_POSITIVE);
            if let Some(engine) = &constraints.font_engine {
                engine.with_mut(|engine| engine.SetDeviceScaleFactor(metric_scale as f32));
            }
            drop(constraints);
            self.state.InvalidateStyle();
        }
        if let Some(scripts) = &self.scripts {
            scripts.ResizeViewport(width, height, device_scale);
        }
        self.Apply(PageMutation::ViewportMutation(
            page_mutation::ViewportMutation {
                size: layout::input::Size { width, height },
            },
        ))
    }
    // cpp: browser/browser.cc:894-903
    pub fn Dispatch(
        &mut self,
        input: &::interaction::input_event::InputEvent,
    ) -> io::Result<::interaction::InteractionResult> {
        // Blink input dispatch runs on the Oilpan-owning main thread. Hit
        // testing editable text may materialize a ShapeResult while resolving
        // the caret offset, so the complete input/default-action lifecycle is
        // a managed-heap allocation boundary just like style and layout.
        let mut heap_scope = foundation::LayoutHeapScope::new();
        heap_scope.DeferCollection();
        self.FlushFrameInputs()?;
        // EventHandler hit testing enters resident PaintLayer/fragment owners.
        // Complete the Page lifecycle before lending that lease; queued input
        // already performs this barrier in FlushFrameInputs, while an immediate
        // input may arrive after a DOM/style mutation with no queued samples.
        self.CompleteLifecycleForImmediateInput()?;
        let started = browser_tracing::profile_input().then(std::time::Instant::now);
        let result = self.DispatchInputWithoutLifecycle(input)?;
        let dispatched = started.map(|start| start.elapsed());
        self.UpdateFrameAndStartVisibleDeferredImages()?;
        self.UpdateCursor();
        if let Some(start) = started {
            eprintln!(
                "page-input-profile dispatch_ms={:.3} lifecycle_cursor_ms={:.3}",
                dispatched.unwrap().as_secs_f64() * 1000.0,
                (start.elapsed() - dispatched.unwrap()).as_secs_f64() * 1000.0
            );
        }
        Ok(result)
    }
    fn DispatchInputWithoutLifecycle(
        &mut self,
        input: &::interaction::input_event::InputEvent,
    ) -> io::Result<::interaction::InteractionResult> {
        self.DispatchInputWithoutLifecyclePacket(input, None)
    }

    fn RecomputeMouseHoverStateWithoutLifecycle(&mut self) -> io::Result<()> {
        self.EnsureInteractionScrollOwners();
        let frame = self
            .frame
            .as_ref()
            .ok_or_else(|| io::Error::other("cannot update hover before the first frame"))?;
        let fragments = PaintFragmentCursor::FromHandle(&frame.fragments);
        if let Some(scripts) = &mut self.scripts {
            scripts.RecomputeMouseHoverState(fragments)?;
        } else {
            self.interaction
                .RecomputeMouseHoverState(
                    &self.state.document.Handle(),
                    interaction::adapter::interaction::InteractionFragmentSource::new(
                        fragments,
                        &paint_skia::path_hit_test::SkiaPathHitTestFactory,
                    ),
                )
                .map_err(|error| io::Error::other(format!("native synthetic hover: {error:?}")))?;
            self.state
                .CompleteFormSubmissions(&self.interaction, &self.state.document.Handle());
        }
        self.state
            .local_frames
            .get()
            .unwrap()
            .CompleteChildInputEffects();
        if self.interaction.TakeUnbufferedInputRequest()
            || self
                .state
                .local_frames
                .get()
                .unwrap()
                .TakeChildUnbufferedInputRequest()
        {
            self.begin_frame.RequestUnbufferedInputEvents();
        }
        Ok(())
    }

    fn DispatchInputWithoutLifecyclePacket(
        &mut self,
        input: &::interaction::input_event::InputEvent,
        packet: Option<interaction::frame_aligned_input_queue::BorrowedInputPacket<'_>>,
    ) -> io::Result<::interaction::InteractionResult> {
        self.DispatchInputWithoutLifecyclePacketFromBrowser(input, packet, false)
    }
    fn DispatchInputWithoutLifecyclePacketFromBrowser(
        &mut self,
        input: &::interaction::input_event::InputEvent,
        packet: Option<interaction::frame_aligned_input_queue::BorrowedInputPacket<'_>>,
        browser_gesture_dispatched: bool,
    ) -> io::Result<::interaction::InteractionResult> {
        if let interaction::input_event::InputEvent::Gesture(gesture) = input {
            if self.external_compositor && !browser_gesture_dispatched {
                self.state
                    .PostCompositorTask(CompositorThreadTask::Gesture(gesture.clone()));
                return Ok(interaction::InteractionResult::default());
            }
            if self.interaction.WillHandleCursorControlGesture(gesture) {
                return Ok(interaction::InteractionResult { input_result: Some(interaction::dispatch::public::platform::web_input_event_result::WebInputEventResult::kHandledSuppressed), ..Default::default() });
            }
            if gesture.GetType() == interaction::input::public::common::input::web_input_event::Type::kGestureDoubleTap {
                self.HandleDoubleTapZoom(gesture)?;
                return Ok(interaction::InteractionResult { input_result: Some(interaction::dispatch::public::platform::web_input_event_result::WebInputEventResult::kHandledSystem), ..Default::default() });
            }
            if gesture.IsScrollEvent() {
                self.RouteGestureScrollToImpl(gesture)?;
                return Ok(interaction::InteractionResult::default());
            }
        }
        self.interaction.ValidateInput(input).map_err(|error| {
            io::Error::other(format!("required native interaction input port: {error:?}"))
        })?;
        if let ::interaction::input_event::InputEvent::Mouse(event) = input {
            self.cursor_position =
                if event.r#type == ::interaction::input_event::MouseEventType::kLeave {
                    None
                } else {
                    Some(event.position)
                };
        }

        // Chromium sends a non-blocking wheel gesture to cc first. DOM still
        // receives the same event, marked with the impl-thread disposition, so
        // its default action does not apply the delta a second time. Regions
        // with a cancelable listener are assigned wholly to the main thread.
        let dispatch_profile = browser_tracing::profile_input()
            .then(std::time::Instant::now);
        let mut routed_input = input.clone();
        let mut compositor_wheel = CompositorWheelResult::default();
        if let ::interaction::input_event::InputEvent::Wheel(wheel) = &mut routed_input {
            // A browser/compositor ingress may already carry the impl-thread
            // disposition.  DOM dispatch must observe that same event without
            // injecting its delta into cc a second time; the later typed
            // ScrollOffsetChanged acknowledgement reflects the actual result.
            if !self.external_compositor
                && !wheel.compositor_owned
                && wheel.compositor_ack.is_empty()
            {
                compositor_wheel = self.state.WithResidentSynchronousPresenter(
                    |presenter, main| presenter.Wheel(main, wheel),
                )?;
                wheel.compositor_owned = compositor_wheel.compositor_owned;
                wheel.compositor_nonblocking = compositor_wheel.dom_ack.is_none();
                if let Some(request) = compositor_wheel.dom_ack {
                    wheel.compositor_ack.push(request);
                }
            }
        }
        let compositor_done = dispatch_profile.map(|start| start.elapsed());

        self.EnsureInteractionScrollOwners();
        let frame = self
            .frame
            .as_ref()
            .ok_or_else(|| io::Error::other("cannot dispatch input before the first frame"))?;
        let fragments = PaintFragmentCursor::FromHandle(&frame.fragments);
        let result = if let Some(scripts) = &mut self.scripts {
            scripts.Dispatch(&routed_input, fragments, packet)?
        } else {
            let result = if let Some(packet) = packet {
                self.interaction.TryDispatchPacket(
                    &routed_input,
                    &self.state.document.Handle(),
                    interaction::adapter::interaction::InteractionFragmentSource::new(
                        fragments,
                        &paint_skia::path_hit_test::SkiaPathHitTestFactory,
                    ),
                    packet,
                )
            } else {
                self.interaction.TryDispatch(
                    &routed_input,
                    &self.state.document.Handle(),
                    interaction::adapter::interaction::InteractionFragmentSource::new(
                        fragments,
                        &paint_skia::path_hit_test::SkiaPathHitTestFactory,
                    ),
                )
            };
            let result =
                result.map_err(|error| io::Error::other(format!("native input: {error:?}")))?;
            self.state
                .CompleteFormSubmissions(&self.interaction, &self.state.document.Handle());
            result
        };
        self.state
            .local_frames
            .get()
            .unwrap()
            .CompleteChildInputEffects();
        if self.interaction.TakeUnbufferedInputRequest()
            || self
                .state
                .local_frames
                .get()
                .unwrap()
                .TakeChildUnbufferedInputRequest()
        {
            self.begin_frame.RequestUnbufferedInputEvents();
        }
        if packet.is_none() {
            self.begin_frame.DidDispatchImmediateInput(input);
        }
        let dom_done = dispatch_profile.map(|start| start.elapsed());

        if self.state.scroll_finished_hover_dirty.replace(false) {
            self.hover_state_dirty = true;
        }

        if !result.default_prevented {
            if let ::interaction::input_event::InputEvent::Wheel(_) = &routed_input {
                if compositor_wheel.compositor_owned {
                    self.ApplyCompositorWheelEffects(compositor_wheel.effects)?;
                    if compositor_wheel.submitted {
                        self.PresentCompositorOnlyFrame();
                    }
                }
            }
        }
        if let ::interaction::input_event::InputEvent::Wheel(wheel) = &routed_input {
            // Production queued wheels ACK each original callback through
            // MainThreadEventQueue, including a detached/no-consumer packet.
            // Inline/direct ingress creates its cc request here instead.
            if packet.is_none() || !self.external_compositor {
                for acknowledgement in
                    ::interaction::WheelDOMAcknowledgements(wheel, result.default_prevented)
                {
                    self.RouteImplInput(InputToImpl::WheelDOMAcknowledgement {
                        request: acknowledgement.request,
                        default_prevented: acknowledgement.default_prevented,
                    })?;
                }
            }
        }
        if let (Some(start), Some(compositor), Some(dom)) =
            (dispatch_profile, compositor_done, dom_done)
        {
            if matches!(
                routed_input,
                ::interaction::input_event::InputEvent::Wheel(_)
            ) {
                eprintln!(
                    "page-wheel-dispatch-profile compositor_ms={:.3} dom_ms={:.3} reflect_ms={:.3} total_ms={:.3}",
                    compositor.as_secs_f64() * 1000.0,
                    (dom - compositor).as_secs_f64() * 1000.0,
                    (start.elapsed() - dom).as_secs_f64() * 1000.0,
                    start.elapsed().as_secs_f64() * 1000.0,
                );
            }
        }

        let restart = matches!(
            routed_input,
            ::interaction::input_event::InputEvent::Key(_)
                | ::interaction::input_event::InputEvent::TextInput(_)
                | ::interaction::input_event::InputEvent::Composition(_)
                | ::interaction::input_event::InputEvent::Focus(_)
        ) || matches!(&routed_input, ::interaction::input_event::InputEvent::Mouse(event)
                if event.r#type == ::interaction::input_event::MouseEventType::kDown);
        self.UpdateCaret(restart);
        Ok(result)
    }
    /// Host view activation is independent of the DOM's focused element.
    pub fn SetActive(&mut self, active: bool) -> io::Result<()> {
        if !active {
            self.interaction.CaptureMouseEventsToWidget(false);
        }
        if self.active != active || !self.active_state_published {
            self.active = active;
            self.active_state_published = true;
            self.state.page_focused.set(active);
            let document = self.state.document.Handle();
            self.interaction.SetPageActive(&document, active);
            if !active {
                self.state.HideValidationMessage();
            }
            self.UpdateCaret(true);
            self.UpdateFrameAndStartVisibleDeferredImages()?;
        }
        Ok(())
    }
    pub fn Caret(&self) -> Option<CaretGeometry> {
        self.CurrentPaintArtifact()
            .and_then(|artifact| artifact.caret)
    }
    pub fn TextInputState(&self) -> Option<interaction::adapter::interaction::TextInputState> {
        self.interaction
            .TextInputState(&self.state.document.Handle())
    }
    fn UpdateCaret(&mut self, restart: bool) {
        // Caret geometry can boundary-reshape a ShapeResult even when Layout is
        // otherwise current (for example the toolbar's first blink after native
        // activation). Some callers already own a lifecycle scope, but the
        // BeginFrame-only blink path does not.
        let mut heap_scope = foundation::LayoutHeapScope::new();
        heap_scope.DeferCollection();
        let drag_position = self.interaction.DragCaretSelection().and_then(|selection| {
            self.state.EnsureLayoutOutput();
            let layout = self.state.layout_engine.borrow();
            let document = self.state.document.borrow();
            interaction::adapter::page_frame_selection::DragCaretForSelection(
                document.GetDocument(),
                layout.Output()?,
                Some(selection),
            )
        });
        let mut drag = self.state.layout_editing.drag_caret.borrow_mut();
        let previous_drag = drag.paint_state().map(|state| state.node_id);
        let drag_changed = drag.update(drag_position);
        let current_drag = drag.paint_state().map(|state| state.node_id);
        drop(drag);
        if drag_changed {
            let mut layout = self.state.layout_engine.borrow_mut();
            for node_id in previous_drag
                .into_iter()
                .chain(current_drag)
                .collect::<HashSet<_>>()
            {
                layout.ApplyMutation(
                    layout::mutation::LayoutMutation::SelectionPaint { node_id },
                );
            }
        }

        // FrameSelection::UpdateAppearance only asks Layout for caret geometry
        // when the active frame owns an editable or document selection. Flushing
        // style and layout unconditionally on every BeginFrame creates a new
        // layout-only paint sidecar before a transform/opacity animation can
        // update the resident PrePaint properties.
        let focused_node = self.state.interaction.borrow().focused_node_id;
        let has_document_selection = self.state.layout_editing.selections.Document().is_some();
        let has_text_control_selection = self
            .interaction
            .HasTextInputState(&self.state.document.Handle());
        self.caret_focused_node = focused_node;
        let needs_caret_geometry =
            self.active && (has_text_control_selection || has_document_selection);
        let position = if needs_caret_geometry {
            self.state.EnsureLayoutOutput();
            let layout = self.state.layout_engine.borrow();
            let document = self.state.document.borrow();
            layout.Output().and_then(|output| {
                interaction::adapter::page_frame_selection::FrameCaretForSelection(
                    document.GetDocument(),
                    output,
                    &self.state.layout_editing.selections,
                    focused_node,
                )
            })
        } else {
            None
        };
        let mut caret = self.state.layout_editing.caret.borrow_mut();
        let previous_node = caret.paint_state().map(|state| state.node_id);
        let now = std::time::Instant::now();
        let changed = caret.update(position, now, restart);
        let paint_state = caret.paint_state();
        browser_tracing::instant(
            "input",
            "Page.CaretState",
            &[
                ("restart", restart as u8 as f64),
                ("geometry_needed", needs_caret_geometry as u8 as f64),
                ("position", position.is_some() as u8 as f64),
                ("enabled", caret.IsCaretEnabled() as u8 as f64),
                (
                    "visible",
                    paint_state.is_some_and(|state| state.visible) as u8 as f64,
                ),
                ("changed", changed as u8 as f64),
                (
                    "next_blink_ms",
                    caret.NextBlinkDeadline().map_or(-1.0, |deadline| {
                        deadline.saturating_duration_since(now).as_secs_f64() * 1000.0
                    }),
                ),
            ],
        );
        let current_node = paint_state.map(|state| state.node_id);
        let block_text_invalidations = caret.TakeBlockCaretTextInvalidations();
        drop(caret);
        if !block_text_invalidations.is_empty() {
            self.state
                .layout_engine
                .borrow_mut()
                .ApplyMutation(layout::mutation::LayoutMutation::Selection(
                    layout::views::selection::SelectionMutation::InvalidateBlockCaretText {
                        anchors: &block_text_invalidations,
                    },
                ));
        }
        let revision = self.state.layout_editing.selections.Revision();
        let selection_changed = self.selection_revision != revision;
        let current_document_selection = self.state.layout_editing.selections.Document();
        let previous_document_selection = self.document_selection;
        self.selection_revision = revision;
        self.document_selection = current_document_selection;
        if selection_changed || previous_node != current_node {
            // layout_selection.cc:287-337 invalidates objects whose selection
            // state changed, plus a stable endpoint object when only its text
            // offset moved. Derive the same state from immutable fragment order;
            // Page still routes only owner ids through the typed Layout boundary.
            #[derive(Clone, Copy, PartialEq, Eq)]
            enum Membership {
                Start,
                End,
                StartAndEnd,
                Inside,
            }
            fn selected_membership(
                fragment: PaintFragmentCursor<'_>,
                selection: DocumentSelection,
                output: &mut HashMap<u64, Membership>,
            ) {
                let (start, end) = selection.Ordered();
                let order = fragment.paint.LogicalTreeOrder();
                if matches!(
                    fragment.ViewKind(),
                    FragmentKind::kText | FragmentKind::kGeneratedText
                ) && order >= start.tree_order
                    && order <= end.tree_order
                    && fragment.ViewNodeId() != 0
                {
                    let state = if order == start.tree_order && order == end.tree_order {
                        Membership::StartAndEnd
                    } else if order == start.tree_order {
                        Membership::Start
                    } else if order == end.tree_order {
                        Membership::End
                    } else {
                        Membership::Inside
                    };
                    output.insert(fragment.ViewNodeId(), state);
                }
                for child in fragment.Children() {
                    selected_membership(child, selection, output);
                }
            }
            let mut affected_nodes: Vec<u64> = [previous_node, current_node]
                .into_iter()
                .flatten()
                .filter(|node_id| *node_id != 0)
                .collect();
            if let Some(frame) = &self.frame {
                // A synchronous focus/edit dispatch can invalidate Layout after
                // the PageFrame was published but before this caret bookkeeping
                // runs.  The handle retains native ownership, not a promise that
                // its root still has committed Paint input. Chromium defers the
                // fragment membership walk to the next lifecycle update in this
                // state; endpoint owners below remain sufficient to invalidate
                // the old/new caret clients now.
                if let Some(fragments) = PaintFragmentCursor::TryFromHandle(&frame.fragments) {
                    let mut old_membership = HashMap::new();
                    let mut new_membership = HashMap::new();
                    if let Some(selection) = previous_document_selection {
                        selected_membership(fragments, selection, &mut old_membership);
                    }
                    if let Some(selection) = current_document_selection {
                        selected_membership(fragments, selection, &mut new_membership);
                    }
                    for node_id in old_membership.keys().chain(new_membership.keys()) {
                        if old_membership.get(node_id) != new_membership.get(node_id) {
                            affected_nodes.push(*node_id);
                        }
                    }
                    if let (Some(old), Some(new)) =
                        (previous_document_selection, current_document_selection)
                    {
                        let (old_start, old_end) = old.Ordered();
                        let (new_start, new_end) = new.Ordered();
                        for (old_endpoint, new_endpoint) in
                            [(old_start, new_start), (old_end, new_end)]
                        {
                            if old_endpoint.node_id == new_endpoint.node_id
                                && old_endpoint.offset != new_endpoint.offset
                            {
                                affected_nodes.push(new_endpoint.node_id);
                            }
                        }
                    }
                }
            }
            let mut layout = self.state.layout_engine.borrow_mut();
            affected_nodes.sort_unstable();
            affected_nodes.dedup();
            for node_id in affected_nodes {
                layout.ApplyMutation(
                    layout::mutation::LayoutMutation::SelectionPaint { node_id },
                );
            }
        }
        if changed || selection_changed || drag_changed {
            self.editing_paint_dirty = true;
            // FrameCaret and CaretDisplayItemClient schedule paint
            // invalidation; moving/hiding the caret or changing a selection
            // does not make the text control need layout. Control-value DOM
            // mutations still invalidate geometry through ApplyDOMMutation.
            // Keep the completed fragment geometry for the paint-only pass.
            self.state.document.borrow_mut().InvalidatePaint();
            self.state.scroll_only.set(false);
            self.state.dirty.set(true);
            // UpdateCursorFeedback runs after the immediate input lifecycle.
            // A composition-only or caret-only change can therefore be the
            // first work for the next visual update; explicitly schedule it
            // just as FrameCaret and LayoutSelection do in Blink.
            self.RequestBeginFrameIfNeeded();
        }
    }
    pub fn Cursor(&self) -> Cursor {
        self.state.native_event_handler_host.cursor.borrow().clone()
    }
    /// Platform/default-locale and WebPreferences inputs for the frame's
    /// editing behavior. Neither value is derived from author document state.
    /// The embedding supplies native process-global kill-ring operations.
    /// Shared host file reader for native File controls, including session
    /// history restoration. File bytes remain lazy and outside Interaction.
    pub fn SetNativeFileReader(&self, reader: Arc<dyn fileapi::NativeFileReader>) {
        *self.state.native_file_reader.borrow_mut() = Some(reader.clone());
        if let Some(controller) = &self.form_controller {
            controller.SetFileReader(reader.clone());
        }
        if let Some(interaction) = self.state.popup_interaction.borrow().upgrade() {
            interaction.SetFormStateFileReader(reader.clone());
        }
        if let Some(domain) = self.state.local_frames.get() {
            for frame in domain.frames.borrow().values() {
                if let Some(runtime) = frame.runtime.borrow().as_ref() {
                    if let Some(controller) =
                        runtime.document.borrow().GetDocument().FormController()
                    {
                        controller.SetNativeFileReader(reader.clone());
                    }
                }
            }
        }
    }
    pub fn SetKillRing(
        &self,
        ring: Rc<dyn interaction::default::core::editing::kill_ring::KillRingPlatform>,
    ) {
        *self.state.kill_ring.borrow_mut() = Some(ring.clone());
        self.interaction.SetKillRing(ring);
    }
    pub fn SetInteractionTextSettings(
        &mut self,
        editing_behavior: interaction::default::core::editing::editing_behavior::EditingBehaviorType,
        ui_locale: &std::ffi::CStr,
        smart_insert_delete_enabled: bool,
    ) {
        *self.state.interaction_text_settings.borrow_mut() = Some(
            interaction::adapter::page_pointer_hit_test::InteractionTextSettings {
                editing_behavior,
                smart_insert_delete_enabled,
                ui_locale: ui_locale.to_owned(),
            },
        );
    }
    /// Locale's native format/symbol data. DateTimeEdit behavior and retained
    /// partial values remain owned by Interaction, independent of this host.
    pub fn SetTemporalInputLocale(
        &mut self,
        locale: interaction::adapter::temporal_input_locale::TemporalInputLocaleData,
    ) {
        self.interaction
            .FormControlState()
            .SetTemporalInputLocale(Rc::new(locale));
    }
    /// Native widget ViewRect origin and original device scale, supplied by
    /// the host on moves, display-scale changes and visual-properties updates.
    pub fn SetContextMenuWidgetGeometry(
        &mut self,
        geometry: interaction::adapter::page_nonlocated_context_menu_coordinates::NativeContextMenuWidgetGeometry,
    ) -> io::Result<()> {
        if !geometry.blink_to_dip_scale.is_finite() || geometry.blink_to_dip_scale <= 0.0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid native widget scale",
            ));
        }
        self.state.context_menu_widget_geometry.set(Some(geometry));
        Ok(())
    }

    /// Source RenderWidgetHostViewBase::SetSystemCursorSize, cc:526-530.
    /// The native host reports DIP dimensions and republishes on OS cursor
    /// settings changes; layout zoom and device scale do not alter this input.
    pub fn SetSystemCursorSize(&mut self, width_dip: f32, height_dip: f32) -> io::Result<()> {
        if !width_dip.is_finite() || !height_dip.is_finite() || width_dip < 0.0 || height_dip < 0.0
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid native cursor size",
            ));
        }
        let size = interaction::core::input::event_handler_cursor::SizeF {
            width: width_dip,
            height: height_dip,
        };
        if self
            .state
            .native_event_handler_host
            .system_cursor_size_dip
            .get()
            != Some(size)
        {
            self.state
                .native_event_handler_host
                .system_cursor_size_dip
                .set(Some(size));
            self.cursor_hit_test = None;
            self.UpdateCursorFeedback(true);
        }
        Ok(())
    }
    /// Preserve the widget's native mouse position across document replacement.
    /// The new document resolves its hover target after it has first produced
    /// hit-test geometry, matching Blink's persistent MouseEventManager state.
    pub fn SetLastMousePosition(&mut self, position: Option<Offset>) {
        if self.cursor_position != position {
            self.cursor_position = position;
            self.hover_state_dirty = position.is_some();
            self.cursor_hit_test = None;
            self.RequestBeginFrameIfNeeded();
        }
    }
    pub fn FocusedNodeId(&self) -> Option<u64> {
        self.state.interaction.borrow().focused_node_id
    }
    fn UpdateCursor(&mut self) {
        self.UpdateCursorFeedback(false);
    }
    fn UpdateCursorFeedback(&mut self, force: bool) {
        let activities = self
            .state
            .layout_engine
            .borrow_mut()
            .ScrollMut()
            .TakeNativeScrollbarActivities();
        self.interaction
            .HandleNativeScrollbarActivities(activities)
            .unwrap_or_else(|error| std::panic::panic_any(error));
        if self.state.autoscroll_cursor_overridden.get() {
            return;
        }
        // Blink LayoutView::HitTestNoLifecycleUpdate consults HitTestCache for
        // the same point and DOM version. Coalesce repeated cursor queries
        // from lifecycle and BeginFrame/Flush tails with the same proof here.
        // Pending mutations/uncertain styles cannot use an old hit result.
        let query = if !self.state.dirty.get() && self.state.styles_resolved.get() {
            let owner = self.state.document.borrow();
            let measurement_revision = owner.GetMeasurementGeometryRevision();
            let document = owner.GetDocument();
            let style = document.StyleState();
            if style.impact.IsEmpty()
                && !style.all_dirty
                && style.changes.is_empty()
                && style.dirty_style_elements.is_empty()
                && self.state.layout_revision.get() == Some(measurement_revision)
            {
                document
                    .CollectionMembershipKey()
                    .zip(self.frame.as_ref())
                    .map(|((owner, revision), frame)| {
                        (frame.sequence, self.cursor_position, owner, revision)
                    })
            } else {
                None
            }
        } else {
            None
        };
        // EventHandler::SelectCursor is a no-lifecycle-update hit test.  Blink
        // simply waits for the next clean lifecycle when style, Layout, or
        // fragment-local PrePaint state is not current; that transient state is
        // not a cursor failure.  Keep the synthetic hover request armed so the
        // clean BeginFrame retries against the newly presented fragment owner.
        if self.cursor_position.is_some()
            && (query.is_none()
                || self.frame.as_ref().is_none_or(|frame| {
                    PaintFragmentCursor::TryFromHandle(&frame.fragments).is_none()
                }))
        {
            self.cursor_hit_test = None;
            self.hover_state_dirty = true;
            self.RequestBeginFrameIfNeeded();
            return;
        }
        if !force && query.is_some() && query == self.cursor_hit_test {
            return;
        }
        self.cursor_hit_test = query;
        #[cfg(test)]
        {
            self.cursor_hit_test_queries += 1;
        }
        if query.is_some() {
            self.EnsureInteractionScrollOwners();
        }
        let mut image_failures = Vec::new();
        let selected = match (self.cursor_position, self.frame.as_ref()) {
            (Some(point), Some(frame)) => (|| {
                use interaction::adapter::page_cursor::{
                    PrepareResidentCursor, PreparedResidentCursorQuery, ResidentCursorError,
                    ResidentCursorHostMetrics, SelectPreparedResidentCursor,
                };
                let host_metrics = ResidentCursorHostMetrics {
                    device_scale_factor: self.state.constraints.borrow().device_pixel_ratio as f32,
                    system_cursor_size_dip: self
                        .state
                        .native_event_handler_host
                        .system_cursor_size_dip
                        .get()
                        .ok_or(ResidentCursorError::MissingHostCursorMetrics)?,
                };
                let preferred_color_scheme = match self.state.preferred_color_scheme.get() {
                    PreferredColorScheme::kDark => {
                        interaction::core::input::event_handler_cursor::ColorScheme::kDark
                    }
                    PreferredColorScheme::kLight => {
                        interaction::core::input::event_handler_cursor::ColorScheme::kLight
                    }
                };
                let queries = Rc::new(resident_editing_positions::PageResidentEditingPositions(
                    Rc::downgrade(&self.state),
                ));
                let scroll = self
                    .interaction_scroll_owners
                    .as_ref()
                    .ok_or(ResidentCursorError::MissingScrollOwner)?
                    .ScrollManager();
                // Source hit traversal borrows the live Layout/Paint owners.
                // Only canonical node/scrollbar identities escape this scope.
                let hit = {
                    let layout = self.state.layout_engine.borrow();
                    let output = layout
                        .Output()
                        .ok_or(ResidentCursorError::PrePaintNotReady)?;
                    let paint = self.state.paint_engine.borrow();
                    let fragments = PaintFragmentCursor::TryFromHandle(&frame.fragments)
                        .ok_or(ResidentCursorError::PrePaintNotReady)?;
                    PrepareResidentCursor(
                        &self.interaction,
                        &self.state.document.Handle(),
                        point,
                        PreparedResidentCursorQuery {
                            image_failures: &mut image_failures,
                            layout: output,
                            fragments,
                            paint: &paint,
                            scroll: &scroll,
                            paths: &paint_skia::path_hit_test::SkiaPathHitTestFactory,
                            selections: &self.state.layout_editing.selections,
                            lifecycle_clean: query.is_some(),
                            child_frames: &local_frame::LocalFrameQueries {
                                domain: self.state.local_frames.get().unwrap(),
                                paint: &paint,
                            },
                            native_paths: Arc::new(
                                paint_skia::svg_shape_path::SkiaNativePathFactory,
                            ),
                            editing_positions: queries.clone(),
                            host_metrics,
                            preferred_color_scheme,
                        },
                    )?
                };
                SelectPreparedResidentCursor(
                    &self.interaction,
                    hit,
                    &scroll,
                    &self.state.layout_editing.selections,
                    queries.as_ref(),
                    host_metrics,
                    preferred_color_scheme,
                    &mut image_failures,
                )
            })(),
            _ => Ok(Some(Cursor::default())),
        };
        for failure in image_failures {
            eprintln!(
                "rechrom-cursor-image-error\t{}",
                serde_json::json!({
                    "url": self.URL(), "resource_id": failure.resource_id,
                    "kind": format!("{:?}", failure.kind), "message": failure.message,
                })
            );
        }
        let cursor = match selected {
            Ok(Some(cursor)) => cursor,
            Ok(None) => return,
            // A synchronous input default action can invalidate PrePaint after
            // the cheap lifecycle check above but before the resident layer
            // query completes. Chromium defers cursor selection until the next
            // clean lifecycle in this case; it is not a terminal query error.
            Err(interaction::adapter::page_cursor::ResidentCursorError::PrePaintNotReady) => {
                self.cursor_hit_test = None;
                self.hover_state_dirty = true;
                self.RequestBeginFrameIfNeeded();
                return;
            }
            Err(error) => {
                eprintln!(
                    "rechrom-cursor-error\t{}",
                    serde_json::json!({
                        "url": self.URL(), "error": format!("{error:?}"),
                    })
                );
                return;
            }
        };
        // The embedding restores the active Page's current cursor when routing
        // changes between toolbar/content. Re-hit-testing a forced mouse move
        // must not turn an unchanged cursor value into another host event.
        if cursor != *self.state.native_event_handler_host.cursor.borrow() {
            *self.state.native_event_handler_host.cursor.borrow_mut() = cursor.clone();
            self.state.client.borrow_mut().DidChangeCursor(cursor);
        }
    }
    pub fn URL(&self) -> &str {
        self.open_engine.CurrentURL()
    }
    pub fn Document(&self) -> std::cell::Ref<'_, DOM> {
        self.state.document.borrow()
    }
    /// The main frame's title as Document::DispatchDidReceiveTitle reports it
    /// to the browser: the canonical document title, shortened to
    /// mojom::blink::kMaxTitleChars.
    pub fn Title(&self) -> String {
        const kMaxTitleChars: usize = 4 * 1024;
        let title = self.state.document.borrow().GetDocument().Title();
        title.chars().take(kMaxTitleChars).collect()
    }
    #[cfg(test)]
    pub(crate) fn WindowOwnersForTesting(
        &self,
    ) -> (
        Rc<DocumentEngine>,
        Rc<RefCell<WindowJavaScriptBindings>>,
        Rc<webapi::window_bindings::WindowPerformanceTiming>,
    ) {
        (
            self.state.document.clone(),
            self.scripts
                .as_ref()
                .expect("scripted Window fixture")
                .InteractionEnvironment()
                .window,
            self.state.window_performance_timing.clone(),
        )
    }
    /// Read-only access to resident native state for diagnostics.
    #[doc(hidden)]
    pub fn GetLayoutEngine(
        &self,
    ) -> std::cell::Ref<'_, layout::LayoutEngine> {
        self.state.layout_engine.borrow()
    }
    /// Last semantic alert/visibility event, shared by native and headless hosts.
    pub fn ValidationMessageVisibility(&self) -> Option<ValidationMessageVisibility> {
        self.state.validation_message_visibility.borrow().clone()
    }
    /// Current browser-owned validation overlay state for headless diagnostics.
    pub fn ValidationMessage(&self) -> Option<ValidationMessageUpdate> {
        self.state
            .validation_message
            .borrow()
            .as_ref()
            .map(|visible| ValidationMessageUpdate::Show {
                control: visible.control,
                anchor: visible.anchor,
                message: visible.message.clone(),
            })
    }
    pub fn CurrentFrame(&self) -> Option<&PageFrame> {
        self.frame.as_ref()
    }

    /// LocalFrameView::GetPaintArtifact (local_frame_view.cc:4399-4406): the
    /// artifact of the committed frame, borrowed from PaintEngine's
    /// persistent PaintController data, its only document-side owner. None
    /// before the first frame commits or after a new document is installed.
    /// Do not hold the borrow across calls that run Paint.
    pub fn CurrentPaintArtifact(
        &self,
    ) -> Option<std::cell::Ref<'_, Arc<paint::paint_engine::PaintArtifact>>> {
        self.frame.as_ref()?;
        self.state.CurrentPaintArtifact()
    }

    /// Admit or prioritize image requests from current client geometry.
    /// ClientRects is evaluated after pending scroll offsets reach Layout, so
    /// this covers both programmatic and compositor-owned viewport scrolling
    /// without asking script to rediscover the same DOM state. CSS images are
    /// included with replaced images: a late mask/background in a long page
    /// must not remain behind earlier offscreen image transfers once its owner
    /// becomes visible.
    fn StartVisibleDeferredImages(&self) -> io::Result<usize> {
        // Source: third_party/blink/renderer/core/frame/settings.json5 and
        // LazyLoadImageObserver. Before load, Chromium uses a zero margin so
        // near-viewport images do not compete with render-blocking resources;
        // after load it expands to the effective-connection-type margin.
        // Rechrom does not expose an ECT estimate yet, so use kUnknown.
        const LAZY_IMAGE_LOADING_MARGIN_PX_UNKNOWN: f64 = 3000.0;
        let lazy_image_margin = if matches!(self.open_engine.Status(), OpenStatus::Finished) {
            LAZY_IMAGE_LOADING_MARGIN_PX_UNKNOWN
        } else {
            0.0
        };
        let (viewport_width, viewport_height) = {
            let constraints = self.state.constraints.borrow();
            (
                constraints.available_size.width,
                constraints.available_size.height,
            )
        };
        if !self.resources.HasViewportImageWork() {
            return Ok(0);
        }
        let lazy_image_nodes = self
            .state
            .lazy_image_nodes
            .borrow()
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let (sources, visible_images) = {
            let owner = self.state.document.borrow();
            let document = owner.GetDocument();
            // ClientRects reflect the root scroll already committed to Layout,
            // while Document can be one lifecycle ahead after an impl-thread or
            // script scroll. Compare in the rects' coordinate space by applying
            // only that uncommitted scroll delta.
            let viewport_root = document
                .Node(document.Root())
                .Children()
                .iter()
                .copied()
                .find(|&index| document.Node(index).IsHTMLElement("html"));
            let document_viewport_origin =
                viewport_root.map_or_else(Offset::default, |root| document.ScrollOffsetFor(root));
            let layout = self.state.layout_engine.borrow();
            let Some(output) = layout.Output() else {
                return Ok(0);
            };
            let layout_viewport_origin = viewport_root
                .map(|root| document.Node(root).Id())
                .and_then(|root_id| {
                    output
                        .ScrollOffsets()
                        .into_iter()
                        .find_map(|(id, offset)| (id == root_id).then_some(offset))
                })
                .unwrap_or_default();
            let viewport_origin = Offset {
                x: document_viewport_origin.x - layout_viewport_origin.x,
                y: document_viewport_origin.y - layout_viewport_origin.y,
            };
            let mut sources = HashSet::new();
            let mut visible_images = Vec::new();
            let mut visibility = HashMap::new();
            let mut is_visible = |index: usize| {
                if !document.IsConnectedToDocument(index) {
                    return false;
                }
                // LayoutInline and generated-content owners may not publish a
                // standalone LayoutOutput node. Use the nearest flat-tree
                // ancestor which does own geometry; do not continue to a
                // visible body/root after finding an offscreen local owner.
                let mut geometry_owner = Some(index);
                loop {
                    let Some(owner) = geometry_owner else {
                        return false;
                    };
                    if let Some(visible) = visibility.get(&owner) {
                        return *visible;
                    }
                    if let Some(layout_node) = output.Node(document.Node(owner).Id()) {
                        let rects = layout_node.ClientRects();
                        let intersects = rects.iter().any(|rect| {
                            rect.width > 0.0
                                && rect.height > 0.0
                                && rect.x < viewport_origin.x + viewport_width + lazy_image_margin
                                && rect.y < viewport_origin.y + viewport_height + lazy_image_margin
                                && rect.x + rect.width > viewport_origin.x - lazy_image_margin
                                && rect.y + rect.height > viewport_origin.y - lazy_image_margin
                        });
                        visibility.insert(owner, intersects);
                        return intersects;
                    }
                    geometry_owner = document.FlatTreeParentElement(owner);
                }
            };
            // ComputedStyle already owns this sparse consumer index. Keep
            // source order and query geometry only for actual image owners.
            for (index, referenced) in document.ReferencedStyleImageConsumers() {
                if !is_visible(index) {
                    continue;
                }
                sources.extend(
                    referenced
                        .iter()
                        .filter(|source| !source.is_empty())
                        .cloned(),
                );
            }
            // HTMLImageLoader registers only lazy candidates with the source
            // observer. Resolve ids against the live arena so detach/adoption
            // cannot retain an obsolete geometry owner.
            for node_id in lazy_image_nodes {
                let Some(index) = document.FindNodeById(node_id) else {
                    continue;
                };
                if is_visible(index) {
                    visible_images.push(node_id);
                }
            }
            (sources, visible_images)
        };
        let (started, prioritized) = self.resources.StartVisibleImageSources(&sources)?;
        let (html_started, html_prioritized) =
            self.resources.StartVisibleHTMLImages(&visible_images);
        let (started, prioritized) = (started + html_started, prioritized + html_prioritized);
        if started != 0 || prioritized != 0 {
            browser_tracing::instant(
                "input",
                "VisibleLazyImageAdmission",
                &[
                    ("visible_sources", sources.len() as f64),
                    ("started", started as f64),
                    ("prioritized", prioritized as f64),
                ],
            );
        }
        Ok(started + prioritized)
    }
    #[cfg(test)]
    pub(crate) fn FullLayoutLifecycleCount(&self) -> usize {
        self.state.full_layout_lifecycles.get()
    }

    // Explicit complete-source mutation/fixture callers retain their synchronous
    // resource-completion contract. Network navigation never enters this adapter.
    fn FinishSuppliedDocumentResources(&mut self) -> io::Result<()> {
        self.state.ResolveStyles();
        self.resources.QueueReferencedStyleSheets()?;
        self.resources.QueueReferencedImages()?;
        self.resources.PrepareFontFaces();
        self.state.EnsureLayoutOutput();
        self.resources.StartPendingFonts();
        let mut client = ResourceClient {
            state: self.state.clone(),
            scripts: self.scripts.as_deref_mut(),
        };
        self.resources.LoadPendingStyleSheets(&mut client);
        self.resources.LoadPendingImages(&mut client);
        self.resources.LoadPendingFonts(&mut client);
        self.state.ResolveStyles();
        Ok(())
    }

    // cpp: browser/browser.cc:1761-1771
    fn ResolveStylesAndLoadResources(&mut self, completion_allowance: usize) -> io::Result<()> {
        opening::ResolvePageStylesAndLoadResources(
            &self.state,
            &self.resources,
            &mut self.scripts,
            completion_allowance,
            true,
            false,
        )
        .map(|_| ())
    }

    // cpp: browser/browser.cc:1731-1745
    fn UpdateFrameAndStartVisibleDeferredImages(&mut self) -> io::Result<()> {
        self.UpdateFrameIfNeeded()?;
        let _trace = browser_tracing::span("lifecycle", "Page.StartVisibleDeferredImages");
        self.StartVisibleDeferredImages()?;
        Ok(())
    }

    fn UpdateFrameIfNeeded(&mut self) -> io::Result<()> {
        // An ordinary task with a BeginFrameSource does not run the document
        // lifecycle: it only schedules the rendering opportunity
        // (Document::ScheduleLayoutTreeUpdate -> LocalFrameView::
        // ScheduleAnimation; third_party/blink/renderer/core/dom/
        // document.cc, core/frame/local_frame_view.cc). The post-lifecycle
        // steps below belong to that lifecycle update
        // (LocalFrameView::RunPostLifecycleSteps -> ComputeIntersections;
        // ValidationMessageClientImpl::UpdatePrePaint), so a deferred task
        // turn leaves them to the BeginMainFrame that runs it.
        let lifecycle_deferred = self.LifecycleDeferredForBeginFrame();
        self.UpdateFrameIfNeededInternal()?;
        if lifecycle_deferred {
            self.ScheduleDueIntersectionObservation();
            return Ok(());
        }
        {
            let _trace = browser_tracing::span("lifecycle", "Page.UpdateValidationMessage");
            self.state.UpdateValidationMessage();
        }
        let _trace = browser_tracing::span("lifecycle", "Page.UpdateIntersections");
        self.state.local_frames.get().unwrap().UpdateIntersections()
    }

    /// IntersectionObserverController::ScheduleIntersectionObserverForDocument
    /// and a delayed observer's next update both end in
    /// LocalFrameView::ScheduleAnimation (intersection_observer_controller.cc,
    /// local_frame_view.cc). Once the domain's deadline is due and its posted
    /// observer tasks have run, request that rendering opportunity; the
    /// BeginMainFrame's post-lifecycle step computes the intersections.
    fn ScheduleDueIntersectionObservation(&mut self) {
        let domain = self.state.local_frames.get().unwrap();
        let Some(deadline) = domain.intersection_deadline.get() else {
            return;
        };
        if deadline > std::time::Instant::now() || !domain.intersection_tasks.borrow().is_empty() {
            return;
        }
        domain.intersection_deadline.set(None);
        self.state.animation_begin_frame_requested.set(true);
        self.RequestBeginFrameIfNeeded();
    }

    /// FrameCaret's appearance inputs changed outside the rendering lifecycle:
    /// caret_blink_timer_ fired (FrameCaret::CaretBlinkTimerFired), or the
    /// selection, focused element or drag caret moved
    /// (FrameSelection::DidSetSelectionDeprecated ->
    /// ScheduleVisualUpdateForPaintInvalidationIfNeeded;
    /// third_party/blink/renderer/core/editing/frame_caret.cc,
    /// frame_selection.cc). Only then does a deferred task turn update the
    /// caret; every other caret update belongs to the BeginMainFrame.
    fn CaretAppearanceNeedsUpdate(&self) -> bool {
        let editing = &self.state.layout_editing;
        editing
            .caret
            .borrow()
            .NextBlinkDeadline()
            .is_some_and(|deadline| deadline <= std::time::Instant::now())
            || editing.selections.Revision() != self.selection_revision
            || self.state.interaction.borrow().focused_node_id != self.caret_focused_node
            || editing.drag_caret.borrow().paint_state().is_some()
            || self.interaction.DragCaretSelection().is_some()
    }

    // Blink's CancelAnimationOnCompositor / restart of a changed compositor
    // animation, synchronously on the main AnimationHost in this lifecycle.
    fn ApplyPendingPropertyAnimationRemovals(&mut self) -> io::Result<()> {
        let removals =
            std::mem::take(&mut *self.state.pending_property_animation_removals.borrow_mut());
        let mut mutated = false;
        for identity in removals {
            mutated |= self.state.ApplySourcePropertyAnimation(
                layerize::commit::animation::PropertyAnimationMutation::Remove { identity },
            )?;
        }
        if mutated {
            self.DidMutateMainCompositor()?;
        }
        Ok(())
    }

    fn ApplyPendingPropertyAnimationReplacements(&mut self) -> io::Result<()> {
        let replacements = std::mem::take(
            &mut *self
                .state
                .pending_property_animation_replacements
                .borrow_mut(),
        );
        let mut mutated = false;
        for replacement in replacements {
            mutated |= self.state.ApplySourcePropertyAnimation(replacement)?;
        }
        if mutated {
            self.DidMutateMainCompositor()?;
        }
        Ok(())
    }

    fn InstallCompositorPropertyAnimations(
        &mut self,
        bindings: &[paint::CompositedAnimationPropertyBinding],
    ) -> io::Result<()> {
        let mut trace =
            browser_tracing::span("animation", "Animation.InstallCompositorPropertyAnimations");
        let candidates = self.state.animation_engine.borrow().CompositorCandidates();
        trace.set("bindings", bindings.len() as f64);
        trace.set("candidates", candidates.len() as f64);
        for candidate in &candidates {
            let mut transform = false;
            let mut opacity = false;
            for curve in &candidate.curves {
                match curve {
                    animation::CompositorPropertyCurve::Transform(_) => transform = true,
                    animation::CompositorPropertyCurve::Opacity(_) => opacity = true,
                }
            }
            browser_tracing::instant(
                "animation",
                "Animation.CompositorCandidate",
                &[
                    ("node_id", candidate.key.node_id as f64),
                    ("list_index", candidate.key.list_index as f64),
                    ("generation", candidate.generation as f64),
                    ("transform", transform as u8 as f64),
                    ("opacity", opacity as u8 as f64),
                ],
            );
        }
        for binding in bindings {
            browser_tracing::instant(
                "animation",
                "Animation.CompositorBinding",
                &[
                    ("node_id", binding.node_id as f64),
                    (
                        "property",
                        match binding.property {
                            paint::CompositedAnimationProperty::Transform => 1.0,
                            paint::CompositedAnimationProperty::Opacity => 2.0,
                        },
                    ),
                    ("element_id", binding.element_id.GetInternalValue() as f64),
                ],
            );
        }
        let mut planned = BTreeSet::new();
        let mut installed = Vec::new();
        let mut mutations = Vec::new();
        let mut missing_candidate = 0usize;
        let mut ambiguous_candidate = 0usize;
        let mut duplicate_binding = 0usize;
        let mut already_installed = 0usize;
        let mut unsupported_mutation = 0usize;
        for binding in bindings {
            let property = match binding.property {
                paint::CompositedAnimationProperty::Transform => "transform",
                paint::CompositedAnimationProperty::Opacity => "opacity",
            };
            let mut matching = candidates.iter().filter(|candidate| {
                candidate.key.node_id == binding.node_id
                    && candidate.curves.iter().any(|curve| {
                        matches!(
                            (property, curve),
                            (
                                "transform",
                                animation::CompositorPropertyCurve::Transform(_)
                            ) | ("opacity", animation::CompositorPropertyCurve::Opacity(_))
                        )
                    })
            });
            let Some(candidate) = matching.next() else {
                missing_candidate += 1;
                continue;
            };
            // Blink's effect stack owns composition when multiple animations
            // target one property. Until that translated owner is connected,
            // leave the ambiguous property on the document sequence.
            if matching.next().is_some() {
                ambiguous_candidate += 1;
                continue;
            }
            if !planned.insert((candidate.key, property)) {
                duplicate_binding += 1;
                continue;
            }
            let identity = CompositorAnimationIdentity(candidate.effect_id, property);
            if self
                .state
                .compositor_driven_animations
                .borrow()
                .get(&(candidate.key, property))
                .is_some_and(|current| {
                    current.identity == identity
                        && current.generation == candidate.generation
                        && current.element_id == binding.element_id.GetInternalValue()
                })
            {
                already_installed += 1;
                continue;
            }
            let start_time = candidate.sample_time.map(|sample_time| {
                self.state
                    .compositor_executor.TimeTicks(sample_time)
                    - scroll_animation::TimeDelta::Microseconds(
                        (candidate.local_time * 1_000_000.0) as i64,
                    )
            });
            let Some((property, mutation)) =
                CompositorPropertyAnimationMutation(candidate, binding, start_time)
            else {
                unsupported_mutation += 1;
                continue;
            };
            mutations.push(mutation);
            installed.push((
                candidate.key,
                property,
                CompositorAnimationInstallation {
                    identity,
                    generation: candidate.generation,
                    element_id: binding.element_id.GetInternalValue(),
                },
            ));
        }
        trace.set("missing_candidate", missing_candidate as f64);
        trace.set("ambiguous_candidate", ambiguous_candidate as f64);
        trace.set("duplicate_binding", duplicate_binding as f64);
        trace.set("already_installed", already_installed as f64);
        trace.set("unsupported_mutation", unsupported_mutation as f64);
        // CompositorAnimations::StartAnimationOnCompositor on the main
        // AnimationHost in this lifecycle task; the KeyframeModels and the
        // property-tree element ids travel in the same commit.
        let mut applied = Vec::with_capacity(installed.len());
        for (mutation, installation) in mutations.into_iter().zip(installed) {
            if self.state.ApplySourcePropertyAnimation(mutation)? {
                applied.push(installation);
            }
        }
        if !applied.is_empty() {
            self.DidMutateMainCompositor()?;
        }
        trace.set("installed", applied.len() as f64);
        for (key, property, installation) in applied {
            self.state
                .animation_engine
                .borrow_mut()
                .SetCompositorDriven(key, [property.to_owned()]);
            self.state
                .compositor_driven_animations
                .borrow_mut()
                .insert((key, property), installation);
        }
        Ok(())
    }

    fn UpdateFrameIfNeededInternal(&mut self) -> io::Result<()> {
        if !self.external_compositor {
            // Single-thread compositing: the tasks posted to the in-process
            // compositor task runner run as main-thread tasks
            // (widget_input_handler_manager.cc:1257-1258).
            loop {
                let task = self.state.compositor_task_runner.borrow().TakeTask();
                match task {
                    Some(CompositorThreadTask::ImplInput(input)) => self.RouteImplInput(input)?,
                    Some(_) => unreachable!("only input-handler requests run in process"),
                    None => break,
                }
            }
        }
        if self
            .state
            .main_compositor_mutated_outside_lifecycle
            .replace(false)
        {
            self.DidMutateMainCompositor()?;
        }
        // Blink's lifecycle remains on the Oilpan-owning thread through
        // PrePaint and Paint. Caret geometry can reshape a text slice while
        // recording paint, so the managed allocation boundary cannot end at
        // LayoutDocumentWithEngine.
        let mut heap_scope = foundation::LayoutHeapScope::new();
        heap_scope.DeferCollection();
        let mut trace = browser_tracing::span("lifecycle", "Page.UpdateFrameIfNeeded");
        trace.set("dirty", self.state.dirty.get() as u8 as f64);
        trace.set("presented", 0.0);
        // ScrollableArea::DidCompositorScroll commits the native offset before
        // the rendering lifecycle publishes its retained paint properties.
        // A caret geometry query in between would observe the newer DOM scalar
        // and force an unpublished LayoutResult, replacing the canonical
        // fragment owner for an otherwise property-only scroll. Defer only
        // that caret update until the typed scroll mutation has reached the
        // resident LayoutNG result; ordinary frames keep their existing order.
        let defer_caret_for_scroll = !self.state.scroll_updates.borrow().is_empty();
        // FrameSelection::UpdateAppearance runs in the lifecycle
        // (LocalFrameView::PerformPostLayoutTasks -> FrameSelection::
        // DidLayout). A task turn whose lifecycle is deferred only services
        // the caret's own blink timer and selection/focus scheduling.
        if !defer_caret_for_scroll
            && (!self.LifecycleDeferredForBeginFrame() || self.CaretAppearanceNeedsUpdate())
        {
            let _trace = browser_tracing::span("lifecycle", "Page.UpdateCaret");
            self.UpdateCaret(false);
        }
        if self.DeferLifecycleForBeginFrame() {
            trace.set("deferred_for_begin_frame", 1.0);
            return Ok(());
        }
        if !self.state.dirty.get() {
            if let Some(frame) = &mut self.frame {
                frame.compositor_scroll_sequence = self.processed_compositor_scroll_sequence;
            }
            trace.set("clean", 1.0);
            return Ok(());
        }
        // Document::RenderingHasBegun (document.h): once the first frame is
        // presented, Blink no longer consults the render-blocking or
        // renderable-root gates (a document-tree search for <body>), so a
        // presented Page neither evaluates nor traces them.
        let rendering_gate_needed = self.frame.is_none();
        let rendering_blocked = if rendering_gate_needed {
            let document_committed = self.open_engine.HasCommittedDocument();
            let render_blocking_resources = !self
                .state
                .document
                .borrow()
                .GetDocument()
                .HaveRenderBlockingResourcesLoaded();
            let renderable_root = self.state.document.HasRenderableRoot();
            trace.set("document_committed", document_committed as u8 as f64);
            trace.set(
                "render_blocking_resources",
                render_blocking_resources as u8 as f64,
            );
            if browser_tracing::enabled() {
                if let Some(manager) = self
                    .state
                    .document
                    .borrow()
                    .GetDocument()
                    .GetRenderBlockingResourceManager()
                {
                    let (styles, scripts, font_preloads, imperative_fonts, expect_links) =
                        manager.borrow().DebugPendingCounts();
                    trace.set("render_blocking_styles", styles as f64);
                    trace.set("render_blocking_scripts", scripts as f64);
                    trace.set("render_blocking_font_preloads", font_preloads as f64);
                    trace.set("render_blocking_imperative_fonts", imperative_fonts as f64);
                    trace.set("render_blocking_expect_links", expect_links as u8 as f64);
                }
            }
            trace.set("renderable_root", renderable_root as u8 as f64);
            !document_committed || render_blocking_resources || !renderable_root
        } else {
            false
        };
        if rendering_blocked {
            self.state
                .local_frames
                .get()
                .unwrap()
                .root_view
                .lifecycle_updates_throttled
                .set(true);
            trace.set("rendering_ready", 0.0);
            return Ok(());
        }
        self.state
            .local_frames
            .get()
            .unwrap()
            .root_view
            .lifecycle_updates_throttled
            .set(false);
        trace.set("rendering_ready", 1.0);
        // Source: HTMLCanvasElement::Paint / CanvasRenderingContext::
        // PaintRenderingResultsToCanvas. Snapshot at the rendering-opportunity
        // boundary so Layout/Paint consume one immutable front buffer for the
        // complete lifecycle and never call back into WebAPI during replay.
        {
            let _trace = browser_tracing::span("lifecycle", "Page.SnapshotCanvasPaintResources");
            let canvases = self
                .state
                .document
                .borrow()
                .GetDocument()
                .SnapshotCanvasPaintResources();
            self.state.constraints.borrow_mut().canvases = canvases.into();
        }
        let profile = browser_tracing::profile_input().then(std::time::Instant::now);
        // Resource completions are separate Page tasks. An offset-only input
        // must not synchronously decode images or dispatch unrelated load
        // listeners before presenting its already committed paint artifact.
        // Subsequent task turns continue polling those completions normally.
        let retained_input_only = (self.state.scroll_only.get()
            || self.state.paint_resources_dirty.get())
            && !self.state.local_frames.get().unwrap().NeedsLifecycle()
            && self.frame.is_some()
            && !self.editing_paint_dirty
            && !self.document_image_environment_dirty
            && self.state.styles_resolved.get()
            && self.state.document.borrow().GetStyleImpact().IsEmpty();
        trace.set("retained_input_only", retained_input_only as u8 as f64);
        if !retained_input_only {
            // Blink runs resource completion callbacks as ordinary tasks.
            // A rendering opportunity may discover references and resolve
            // already-applied style mutations, but it must not synchronously
            // poll/decode unrelated network completions. This keeps the
            // compositor active tree drawable while BeginMainFrame is busy.
            self.ResolveStylesAndLoadResources(0)?;
        }
        {
            let _trace = browser_tracing::span("lifecycle", "Page.ApplyPendingPropertyAnimations");
            self.ApplyPendingPropertyAnimationReplacements()?;
            self.ApplyPendingPropertyAnimationRemovals()?;
        }
        {
            let _trace = browser_tracing::span("lifecycle", "Page.SynchronizeNonVisualStyleOwners");
            self.state.SynchronizeNonVisualStyleOwners();
        }
        let resources_done = profile.map(|start| start.elapsed());
        let reused_recordings = self.state.ApplyPendingScrollUpdates(self.frame.as_mut());
        trace.set("scroll_records_reused", reused_recordings as u8 as f64);
        let scroll_done = profile.map(|start| start.elapsed());
        if defer_caret_for_scroll {
            self.UpdateCaret(false);
        }
        // A retained resource mutation may admit a document image after the
        // display list has already recorded its concrete use-site container.
        // Chromium resolves SVGImageForContainer before publishing that
        // PaintRecord. Do not take the resource-only fast path until every
        // recorded container has an exact producer-owned variant; the normal
        // Paint pass below will configure the missing variants and re-record.
        let document_image_update_requires_repaint = self
            .frame
            .as_ref()
            .is_some_and(|frame| !frame.document_image_records_complete);
        // A context can be created after the canvas element's first paint.
        // The retained display list then has no resource dependency to update;
        // run HTMLCanvasPainter once to install its DrawImage record. Clear
        // snapshots are included deliberately so the next draw can remain a
        // resource-only update.
        let canvas_update_requires_repaint = self.CurrentPaintArtifact().is_some_and(|artifact| {
            self.state
                .constraints
                .borrow()
                .canvases
                .iter()
                .any(|canvas| {
                    artifact
                        .image_dependencies
                        .dependencies(canvas.resource_id)
                        .is_empty()
                })
        });
        let paint_resource_update_requires_repaint =
            document_image_update_requires_repaint || canvas_update_requires_repaint;
        trace.set(
            "paint_resource_update_requires_repaint",
            paint_resource_update_requires_repaint as u8 as f64,
        );
        // A CSSOM flush already brought native geometry up to date even when
        // the document's frame impact remains pending until Paint commits.
        let revision = self
            .state
            .document
            .borrow()
            .GetMeasurementGeometryRevision();
        let published_layout_revision = self.state.layout_revision.get();
        let layout_revision_current = published_layout_revision == Some(revision);
        let layout_output_current = self.state.layout_engine.borrow().Output().is_some();
        let measurement_trace = browser_tracing::span("lifecycle", "Page.MeasurementInputsMatch");
        let measurement_inputs_match = layout_revision_current
            && layout_output_current
            && crate::persistent_layout::MeasurementInputsMatch(
                &self.state.layout_engine.borrow(),
                &mut self.state.document.borrow_mut(),
                &self.state.interaction.borrow(),
            );
        drop(measurement_trace);
        let layout_geometry_current =
            layout_revision_current && layout_output_current && measurement_inputs_match;
        browser_tracing::instant(
            "lifecycle",
            "Page.LayoutGeometryDecision",
            &[
                ("document_revision", revision as f64),
                (
                    "published_revision",
                    published_layout_revision.unwrap_or(u64::MAX) as f64,
                ),
                (
                    "layout_revision_current",
                    layout_revision_current as u8 as f64,
                ),
                ("layout_output_current", layout_output_current as u8 as f64),
                (
                    "measurement_inputs_match",
                    measurement_inputs_match as u8 as f64,
                ),
            ],
        );
        trace.set(
            "resident_fragments_before_layout",
            self.state
                .layout_engine
                .borrow()
                .GetLayoutResult()
                .is_some() as u8 as f64,
        );
        browser_tracing::instant(
            "layout",
            "Page.LayoutFragmentOwnership",
            &[
                ("stage", 0.0),
                (
                    "resident",
                    self.state
                        .layout_engine
                        .borrow()
                        .GetLayoutResult()
                        .is_some() as u8 as f64,
                ),
            ],
        );
        let paint_only_style = layout_geometry_current
            || (self.frame.is_some() && {
                let impact = self.state.document.borrow().GetStyleImpact();
                impact.paint && !impact.layout && !impact.reattach
            });
        trace.set("paint_only_style", paint_only_style as u8 as f64);
        let frame_present = self.frame.is_some();
        let presented_layout_current = self.PresentedFrameHasCurrentLayoutResult();
        let style_impact_empty = self.state.document.borrow().GetStyleImpact().IsEmpty();
        let local_frame_lifecycle_pending = self.state.local_frames.get().unwrap().NeedsLifecycle();
        browser_tracing::instant(
            "lifecycle",
            "Page.FrameRetentionDecision",
            &[
                ("frame_present", frame_present as u8 as f64),
                (
                    "presented_layout_current",
                    presented_layout_current as u8 as f64,
                ),
                ("editing_paint_dirty", self.editing_paint_dirty as u8 as f64),
                (
                    "document_image_environment_dirty",
                    self.document_image_environment_dirty as u8 as f64,
                ),
                (
                    "layout_geometry_current",
                    layout_geometry_current as u8 as f64,
                ),
                ("style_impact_empty", style_impact_empty as u8 as f64),
                (
                    "paint_resource_repaint",
                    paint_resource_update_requires_repaint as u8 as f64,
                ),
                (
                    "local_frame_lifecycle_pending",
                    local_frame_lifecycle_pending as u8 as f64,
                ),
                ("scroll_records_reused", reused_recordings as u8 as f64),
                ("scroll_only", self.state.scroll_only.get() as u8 as f64),
            ],
        );
        if frame_present
            && presented_layout_current
            && !self.editing_paint_dirty
            && !self.document_image_environment_dirty
            && layout_geometry_current
            && style_impact_empty
            && !paint_resource_update_requires_repaint
        {
            trace.set("frame_retained", 1.0);
            self.state.CommitRetainedPaintLifecycle();
            if self.state.paint_resources_dirty.get() {
                self.state.PublishPaintResources();
                trace.set("paint_resources_updated", 1.0);
            }
            // Keep the host's presentation/lifecycle contract while reusing
            // the existing fragments and display list for an ineffective edit.
            if let Some(start) = profile {
                eprintln!(
                    "page-lifecycle-profile resources_ms={:.3} scroll_layout_paint_ms={:.3} layout_ms=0 paint_ms=0 total_ms={:.3} reused=true scroll_records_reused={}",
                    resources_done.unwrap().as_secs_f64() * 1000.0,
                    (scroll_done.unwrap() - resources_done.unwrap()).as_secs_f64() * 1000.0,
                    start.elapsed().as_secs_f64() * 1000.0,
                    reused_recordings
                );
            }
            let frame = self.frame.as_mut().unwrap();
            self.state.MarkLayoutCurrent();
            frame.sequence += 1;
            frame.compositor_scroll_sequence = self.processed_compositor_scroll_sequence;
            self.state.scroll_only.set(false);
            self.state.dirty.set(false);
            self.state.document.borrow_mut().DidCommitPaint();
            if let Some(scripts) = &mut self.scripts {
                scripts.DidPaint();
            }
            self.state.client.borrow_mut().DidPresentFrame(
                frame,
                &self
                    .state
                    .CurrentPaintArtifact()
                    .expect("committed frame has a PaintController artifact"),
            );
            trace.set("presented", 1.0);
            self.UpdateCursor();
            return Ok(());
        }
        // Geometry cleanliness authorizes reuse of native layout only. A
        // rendering update must materialize current paint inputs; no old
        // exported tree can stand in for changed SVG resources or style.
        let layout_output_reused = layout_geometry_current;
        trace.set("layout_output_reused", layout_output_reused as u8 as f64);
        let animation_paint_nodes = if paint_only_style {
            self.state
                .paint_engine
                .borrow()
                .AnimationTargetedNodes()
                .to_vec()
        } else {
            Vec::new()
        };
        // Keep Chromium's last committed fragment snapshot resident until DOM
        // synchronization classifies the mutation. Tree/constraint mutations
        // invalidate it inside LayoutEngine; a single subtree update retains
        // it so native layout can replace only that native fragment subtree.

        trace.set(
            "targeted_animation_paint",
            (!animation_paint_nodes.is_empty()) as u8 as f64,
        );
        let paint_only_fragments = paint_only_style
            .then(|| {
                let mut layout = self.state.layout_engine.borrow_mut();
                if animation_paint_nodes.is_empty() {
                    crate::persistent_layout::PublishPaintDataWithEngine(
                        &mut layout,
                        &mut self.state.document.borrow_mut(),
                        &self.state.interaction.borrow(),
                    )
                } else {
                    crate::persistent_layout::RefreshTargetedPaintDataWithEngine(
                        &mut layout,
                        &self.state.document.borrow(),
                        &animation_paint_nodes,
                    )
                }
            })
            .flatten();
        trace.set(
            "paint_only_geometry_reused",
            paint_only_fragments.is_some() as u8 as f64,
        );
        let (mut layout_output, layout_advanced) = if let Some(fragments) = paint_only_fragments {
            (
                self.state
                    .layout_engine
                    .borrow_mut()
                    .TakePublishedPaintOutput(fragments),
                true,
            )
        } else {
            trace.set("full_layout", 1.0);
            if let Some(frame) = self.frame.as_mut() {
                // LocalFrameView and LayoutObject share one resident
                // FragmentData lifecycle. Transfer Page's published owner
                // before synchronizing mutations so a pure viewport resize can
                // retain exact native fragment pointers.
                self.state
                    .layout_engine
                    .borrow_mut()
                    .AdoptPublishedLayoutResult(std::mem::take(&mut frame.fragments));
            }
            let layout_advanced = self
                .state
                .LayoutWithContainerQueries(LayoutUpdateKind::Paint);
            (
                self.state
                    .layout_engine
                    .borrow_mut()
                    .TakePaintOutput()
                    .expect("successful layout has fragments"),
                layout_advanced,
            )
        };
        // A paint-only style update resolves at StyleClean, then
        // PublishPaintDataWithEngine proves that the resident LayoutResult is
        // still the current geometry and exports its refreshed paint inputs.
        // That successful proof is the retained-geometry equivalent of the
        // LayoutClean transition performed by LayoutWithContainerQueries.
        // Commit it before any late paint preparation can enter PrePaint.
        if self.state.DocumentLifecycle().GetState()
            == dom::document_lifecycle::LifecycleState::StyleClean
        {
            self.state
                .DocumentLifecycle()
                .AdvanceTo(dom::document_lifecycle::LifecycleState::LayoutClean);
        }
        trace.set("layout_advanced", layout_advanced as u8 as f64);
        let layout_done = profile.map(|start| start.elapsed());
        if !layout_advanced
            && !self.state.local_frames.get().unwrap().NeedsLifecycle()
            && self.frame.is_some()
            && !self.editing_paint_dirty
            && !self.document_image_environment_dirty
            && !paint_resource_update_requires_repaint
        {
            // LayoutView::UpdateLayout proved the native tree and exported
            // FragmentData clean. Chromium does not run PrePaint/Paint again
            // for the duplicate viewport task; return the transferred owner
            // to LocalFrameView and keep its committed display list.
            trace.set("layout_clean_frame_retained", 1.0);
            self.state.CommitRetainedPaintLifecycle();
            let fragments = self
                .state
                .layout_engine
                .borrow_mut()
                .CommitRetainedPaint(layout_output);
            let frame = self.frame.as_mut().unwrap();
            frame.fragments = fragments;
            if self.state.paint_resources_dirty.get() {
                self.state.PublishPaintResources();
                trace.set("paint_resources_updated", 1.0);
            }
            if let Some(start) = profile {
                let elapsed = start.elapsed();
                eprintln!(
                    "page-lifecycle-profile resources_ms={:.3} scroll_layout_paint_ms={:.3} layout_ms={:.3} paint_ms=0 total_ms={:.3} reused=true layout_clean=true scroll_records_reused={}",
                    resources_done.unwrap().as_secs_f64() * 1000.0,
                    (scroll_done.unwrap() - resources_done.unwrap()).as_secs_f64() * 1000.0,
                    (layout_done.unwrap() - scroll_done.unwrap()).as_secs_f64() * 1000.0,
                    elapsed.as_secs_f64() * 1000.0,
                    reused_recordings
                );
            }
            self.state.MarkLayoutCurrent();
            frame.sequence += 1;
            frame.compositor_scroll_sequence = self.processed_compositor_scroll_sequence;
            self.state.scroll_only.set(false);
            self.state.dirty.set(false);
            self.state.document.borrow_mut().DidCommitPaint();
            self.state.paint_resources_dirty.set(false);
            if let Some(scripts) = &mut self.scripts {
                scripts.DidPaint();
            }
            self.state.client.borrow_mut().DidPresentFrame(
                frame,
                &self
                    .state
                    .CurrentPaintArtifact()
                    .expect("committed frame has a PaintController artifact"),
            );
            trace.set("presented", 1.0);
            self.UpdateCursor();
            return Ok(());
        }
        layout_output = self.ReconcileResponsiveAutoSizes(layout_output)?;
        if self.state.blocking_wheel_registry_dirty.replace(false) {
            self.state
                .layout_engine
                .borrow_mut()
                .RefreshOutputPrePaintInputs(&mut layout_output);
        }
        let document_image_trace =
            browser_tracing::span("lifecycle", "Page.ConfigureDocumentImageContainers");
        if self.ConfigureDocumentImageContainers(&mut layout_output)? {
            self.state
                .layout_engine
                .borrow_mut()
                .RefreshOutputPrePaintInputs(&mut layout_output);
        }
        drop(document_image_trace);
        let editing_trace = browser_tracing::span("lifecycle", "Page.CommitEditingState");
        // LocalFrameView::RunPrePaintLifecyclePhase commits FrameSelection's
        // pending LayoutSelection after Layout and before fragment Paint input
        // is consumed. Synchronous pointer queries use the same commit as a
        // read barrier, but the last event in a drag has no following query;
        // keep the rendering lifecycle as the authoritative commit point.
        let pending_layout_selection = self
            .state
            .layout_editing
            .selections
            .layout_selection
            .borrow()
            .HasPendingSelection();
        if pending_layout_selection {
            let selection = PageMouseFrameGeometry::CurrentSelection(&self.state);
            let document = self.state.document.borrow();
            let committed =
                interaction::adapter::page_focus_controller::CommitLayoutSelectionForFocus(
                    document.GetDocument(),
                    &mut self.state.layout_engine.borrow_mut(),
                    selection,
                    false,
                );
            trace.set("layout_selection_committed", committed as u8 as f64);
        }
        self.state.document.borrow().EmitEditingState(
            self.state.layout_editing.clone(),
            |mutation| {
                self.state
                    .layout_engine
                    .borrow_mut()
                    .ApplyMutation(mutation);
            },
        );
        drop(editing_trace);
        let prepare_trace = browser_tracing::span("lifecycle", "Page.PreparePaintInputs");
        self.state
            .paint_engine
            .borrow_mut()
            .SetBlockingWheelEventHandlerTargets(
                local_frame_scroll::InteractionScrollOwner::Root(Rc::downgrade(&self.state))
                    .BlockingWheelTargets(),
            );
        let child_paint = self
            .state
            .local_frames
            .get()
            .unwrap()
            .PreparePaint(&self.state.layout_engine.borrow())?;
        drop(prepare_trace);
        let mut prepared_animation_prepaint = None;
        // Chromium's deferred transform/opacity update is rejected whenever
        // the LayoutObject already needs a full paint-property update. A
        // document-image frame arriving while animation style is sampled is
        // the package-boundary equivalent: the retained PaintArtifact must
        // first publish the new image record, otherwise the direct property
        // update can animate a layer whose drawable content is still empty.
        let direct_animation_update_admissible = !self.state.paint_resources_dirty.get()
            && self.state.pending_paint_mutations.borrow().is_empty();
        trace.set(
            "animation_direct_update_admissible",
            direct_animation_update_admissible as u8 as f64,
        );
        if direct_animation_update_admissible
            && !animation_paint_nodes.is_empty()
            && child_paint.is_empty()
        {
            self.state.SyncVisualViewportPaintState();
            // B5 gate: the main half holds a resident
            // PaintArtifact, in both presentation modes.
            let direct_compositor_ready = self
                .state
                .HasResidentPaintArtifact();
            trace.set("animation_paint_nodes", animation_paint_nodes.len() as f64);
            trace.set("animation_child_paint", child_paint.len() as f64);
            trace.set(
                "animation_direct_compositor_ready",
                direct_compositor_ready as u8 as f64,
            );
            let (prepared, update) = {
                let layout = self.state.layout_engine.borrow();
                let _layout_paint_scope = layout.PaintScope();
                let _svg_resources =
                    layout
                        .Pointer()
                        .SVGResourceQueryScope()
                        .expect("PrePaint retains native SVG resource owners");
                let mut paint = self.state.paint_engine.borrow_mut();
                if direct_compositor_ready {
                    paint.PrePaintCompositedAnimationUpdate(
                        &layout_output,
                        &animation_paint_nodes,
                        &PagePaintLifecycle(self.state.DocumentLifecycle()),
                    )
                } else {
                    (
                        Some(paint.PrePaintWithLifecycle(
                            &layout_output,
                            &PagePaintLifecycle(self.state.DocumentLifecycle()),
                        )),
                        None,
                    )
                }
            };
            // PaintArtifactCompositor::DirectlyUpdate* in this lifecycle task,
            // on the main half (S1b-8). A rejected node is not an error: as in
            // Blink, it keeps kChangedOnlySimpleValues and this lifecycle's
            // full Paint updates PAC (paint_property_tree_builder.cc;
            // local_frame_view.cc:2965-2975,3348).
            let directly_updated = update.as_ref().is_some_and(|update| {
                self.state
                    .DirectlyUpdateCompositedAnimationProperties(update)
            });
            if update.is_some() {
                self.DidMutateMainCompositor()?;
                trace.set(
                    "composited_animation_direct_update_rejected",
                    (!directly_updated) as u8 as f64,
                );
            }
            if let Some(update) = update.filter(|_| directly_updated) {
                trace.set("composited_animation_property_update", 1.0);
                // Blink snapshots eligible keyframes once, binds them to the
                // compositor element id published by PrePaint, and lets cc
                // tick the active property tree thereafter. The first direct
                // update supplies that canonical binding; later BeginFrames
                // no longer sample the same property on the document thread.
                // The property values above were written first, so the same
                // commit carries both (animation_host.cc:485-518).
                self.InstallCompositorPropertyAnimations(&update.bindings)?;
                self.state.CommitRetainedPaintLifecycle();
                let fragments = self
                    .state
                    .layout_engine
                    .borrow_mut()
                    .CommitRetainedPaint(layout_output);
                let frame = self.frame.as_mut().unwrap();
                frame.fragments = fragments;
                frame.compositor_scroll_sequence = self.processed_compositor_scroll_sequence;
                self.state.MarkLayoutCurrent();
                self.state.document.borrow_mut().DidCommitPaint();
                debug_assert!(self.state.pending_paint_mutations.borrow().is_empty());
                self.state
                    .ApplyAnimationMutation(animation::AnimationMutation::CommitPaint);
                self.state.scroll_only.set(false);
                self.state.dirty.set(false);
                // PaintArtifactCompositor::DirectlyUpdateCompositedOpacityValue
                // and DirectlyUpdateTransform never enter Blink's paint
                // lifecycle.  In particular, they must not run the post-paint
                // IntersectionObserver/ResizeObserver update: doing so turns a
                // compositor-only animation into synchronous geometry queries
                // and a document-wide Layout paint-data publication every
                // frame.  The ordinary Paint paths below remain the sole
                // owners of PageScripts::DidPaint.
                self.UpdateCursor();
                return Ok(());
            }
            prepared_animation_prepaint = prepared;
        }
        let candidates_trace = browser_tracing::span("lifecycle", "Page.CompositorAnimationNodes");
        let compositor_animation_nodes = self
            .state
            .animation_engine
            .borrow()
            .CompositorCandidates()
            .into_iter()
            .map(|candidate| candidate.key.node_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        drop(candidates_trace);
        let mut full_paint_animation_bindings = Vec::new();
        let paint_output = {
            trace.set("full_paint", 1.0);
            {
                let layout = self.state.layout_engine.borrow();
                let _layout_paint_scope = layout.PaintScope();
                let _svg_resources =
                    layout
                        .Pointer()
                        .SVGResourceQueryScope()
                        .expect("Paint retains native SVG resource owners");
                let lifecycle = PagePaintLifecycle(self.state.DocumentLifecycle());
                let child_layouts = child_paint
                    .iter()
                    .map(|child| child.runtime.layout.borrow())
                    .collect::<Vec<_>>();
                let views = local_frame::LocalFrameDomain::PaintInputs(
                    &layout_output,
                    &layout,
                    &child_paint,
                    &child_layouts,
                );
                let document = self
                    .state
                    .document
                    .borrow()
                    .GetDocument()
                    .CanvasDocumentIdentity();
                self.state.SyncVisualViewportPaintState();
                let mut paint = self.state.paint_engine.borrow_mut();
                if let Some(prepared) = prepared_animation_prepaint.take() {
                    paint.PaintAfterPrePaintWithLocalFrameViews(
                        &layout_output,
                        &views,
                        self.state.layout_editing.PaintState(),
                        &lifecycle,
                        document,
                        prepared,
                    );
                } else {
                    paint.PaintWithLocalFrameViews(
                        &layout_output,
                        &views,
                        self.state.layout_editing.PaintState(),
                        &lifecycle,
                        document,
                    );
                }
            }
            let paint_document_image_trace =
                browser_tracing::span("lifecycle", "Page.ConfigurePaintDocumentImageContainers");
            let document_image_requirements = self
                .state
                .paint_engine
                .borrow()
                .GetPaintResult()
                .expect("successful paint has artifact")
                .DocumentImageContainerRequirements();
            let rerecord = self.ConfigurePaintDocumentImageContainers(
                &mut layout_output,
                document_image_requirements,
            )?;
            drop(paint_document_image_trace);
            if rerecord {
                self.state
                    .layout_engine
                    .borrow_mut()
                    .RefreshOutputPrePaintInputs(&mut layout_output);
                // Background, mask and border-image target sizes are owned by
                // Paint geometry. Re-record once after their typed SVG
                // variants arrive so every display item names its exact key.
                let layout = self.state.layout_engine.borrow();
                let _layout_paint_scope = layout.PaintScope();
                let _svg_resources =
                    layout
                        .Pointer()
                        .SVGResourceQueryScope()
                        .expect("Paint retains native SVG resource owners");
                let child_layouts = child_paint
                    .iter()
                    .map(|child| child.runtime.layout.borrow())
                    .collect::<Vec<_>>();
                let views = local_frame::LocalFrameDomain::PaintInputs(
                    &layout_output,
                    &layout,
                    &child_paint,
                    &child_layouts,
                );
                self.state.SyncVisualViewportPaintState();
                self.state
                    .paint_engine
                    .borrow_mut()
                    .PaintWithLocalFrameViews(
                        &layout_output,
                        &views,
                        self.state.layout_editing.PaintState(),
                        &PagePaintLifecycle(self.state.DocumentLifecycle()),
                        self.state
                            .document
                            .borrow()
                            .GetDocument()
                            .CanvasDocumentIdentity(),
                    );
            }
            let mut paint = self.state.paint_engine.borrow_mut();
            {
                let _trace =
                    browser_tracing::span("lifecycle", "Page.CompositedAnimationBindings");
                full_paint_animation_bindings =
                    paint.CompositedAnimationPropertyBindings(&compositor_animation_nodes);
            }
            paint.CommitPaintResult()
        };
        let commit_trace = browser_tracing::span("lifecycle", "Page.CommitPaintToLayout");
        self.state
            .local_frames
            .get()
            .unwrap()
            .CommitPaint(child_paint, &paint_output);
        let (fragments, artifact) = self.state.layout_engine.borrow_mut().CommitPaint(
            layout_output,
            paint_output,
            Some(&self.state.layout_editing),
        );
        drop(commit_trace);
        self.state
            .window_performance_timing
            .EventTimingSource()
            .OnPaintFinished();
        trace.set(
            "resident_fragments_after_paint",
            self.state
                .layout_engine
                .borrow()
                .GetLayoutResult()
                .is_some() as u8 as f64,
        );
        browser_tracing::instant(
            "layout",
            "Page.LayoutFragmentOwnership",
            &[
                ("stage", 1.0),
                (
                    "resident",
                    self.state
                        .layout_engine
                        .borrow()
                        .GetLayoutResult()
                        .is_some() as u8 as f64,
                ),
            ],
        );
        if let Some(start) = profile {
            let elapsed = start.elapsed();
            eprintln!(
                "page-lifecycle-profile resources_ms={:.3} scroll_layout_paint_ms={:.3} layout_ms={:.3} paint_ms={:.3} total_ms={:.3} reused=false layout_output_reused={}",
                resources_done.unwrap().as_secs_f64() * 1000.0,
                (scroll_done.unwrap() - resources_done.unwrap()).as_secs_f64() * 1000.0,
                (layout_done.unwrap() - scroll_done.unwrap()).as_secs_f64() * 1000.0,
                (elapsed - layout_done.unwrap()).as_secs_f64() * 1000.0,
                elapsed.as_secs_f64() * 1000.0,
                layout_output_reused
            );
        }
        let use_counter_trace = browser_tracing::span("lifecycle", "Page.CountPaintUseCounters");
        let document_image_records_complete = artifact.HasExactDocumentImageContainerRecords();
        self.state.CountPaintUseCounters(&artifact);
        // Layout's commit returned the committed artifact handle for these
        // inspections only; PaintEngine keeps the artifact (local_frame_view.cc:
        // 3185-3227), Page stores no reference to it.
        drop(artifact);
        drop(use_counter_trace);
        let frame = PageFrame {
            compositor_scroll_sequence: self.processed_compositor_scroll_sequence,
            sequence: self
                .frame
                .as_ref()
                .map_or(self.previous_document_frame_sequence + 1, |f| {
                    f.sequence + 1
                }),
            fragments,
            document_image_records_complete,
        };
        // Native geometry remains current after presentation. Bindings borrow
        // that output directly; no measurement holder retains the paint tree.
        self.state.MarkLayoutCurrent();
        self.frame = Some(frame);
        self.open_engine.DidBeginRendering();
        self.editing_paint_dirty = false;
        self.document_image_environment_dirty = false;
        // LocalFrameView::PerformPostLayoutTasks marks hover dirty. The
        // synthetic move is deliberately deferred to BeginFrame so script
        // cannot re-enter the layout that just completed.
        self.hover_state_dirty = self.cursor_position.is_some();
        let commit_trace = browser_tracing::span("lifecycle", "Page.DidCommitPaint");
        self.state.document.borrow_mut().DidCommitPaint();
        self.state.paint_resources_dirty.set(false);
        self.state.pending_paint_mutations.borrow_mut().clear();
        self.state
            .ApplyAnimationMutation(animation::AnimationMutation::CommitPaint);
        self.state.scroll_only.set(false);
        self.state.dirty.set(false);
        drop(commit_trace);
        if let Some(scripts) = &mut self.scripts {
            let _trace = browser_tracing::span("lifecycle", "Page.ScriptsDidPaint");
            scripts.DidPaint();
        }
        {
            let _trace = browser_tracing::span("lifecycle", "Page.DidPresentFrame");
            let artifact = self
                .state
                .CurrentPaintArtifact()
                .expect("committed frame has a PaintController artifact");
            self.state
                .client
                .borrow_mut()
                .DidPresentFrame(self.frame.as_ref().unwrap(), &artifact);
        }
        // The complete PaintArtifact must enter the Page-domain compositor
        // before its KeyframeModels. This is the package-boundary form of
        // Chromium's single main-frame commit: the element ids above name the
        // property nodes in this exact artifact, never the retired tree.
        self.InstallCompositorPropertyAnimations(&full_paint_animation_bindings)?;
        trace.set("presented", 1.0);
        let _cursor_trace = browser_tracing::span("lifecycle", "Page.UpdateCursor");
        self.UpdateCursor();
        Ok(())
    }
}
#[cfg(test)]
#[test]
fn cursor_hit_test_same_point_reuses_clean_frame_and_scroll_invalidates() {
    crate::test_thread::run(|| {
        struct NoNetwork;
        impl URLLoader for NoNetwork {
            fn Load(
                &mut self,
                _: &URLRequest,
            ) -> io::Result<Box<dyn url_loader::URLLoadOperation>> {
                Err(io::Error::other("unexpected fixture network request"))
            }
        }
        let registry = crate::CreateLayoutRegistry();
        let mut page = Page::Create(
            Rc::new(RefCell::new(NoNetwork)),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(crate::CreateSVGImageDecoder(&registry))),
            crate::CreateBrowserConstraints(320, 200),
            None,
            None,
        );
        *page.state.document.borrow_mut() = html::html_parser::ParseHTML(
            "<!doctype html><body style='margin:0'><div id=s style='width:100px;height:40px;overflow:hidden;cursor:crosshair'><div style='height:200px'>scrolling</div></div></body>",
        );
        page.open_engine.AdoptDocument().unwrap();
        page.cursor_position = Some(Offset { x: 10.0, y: 10.0 });
        page.UpdateFrameIfNeeded().unwrap();
        assert!(page.cursor_hit_test.is_some());
        let queries = page.cursor_hit_test_queries;
        page.UpdateCursor();
        assert_eq!(
            page.cursor_hit_test_queries, queries,
            "the lifecycle tail must reuse its clean hit"
        );
        let scroller = {
            let owner = page.state.document.borrow();
            let document = owner.GetDocument();
            (0..document.NodeCount())
                .find_map(|i| {
                    document
                        .Node(i)
                        .FindAttribute("id")
                        .filter(|attribute| attribute.value == "s")
                        .map(|_| document.Node(i).Id())
                })
                .unwrap()
        };
        let before = page.CurrentFrame().unwrap().sequence;
        page.state.ApplyMutation(PageMutation::ScrollMutation(
            page_mutation::ScrollMutation {
                target_node_id: scroller,
                offset: Offset { x: 0.0, y: 20.0 },
                scroll_type:
                    layout_style::style::scroll_enums::mojom::blink::ScrollType::kProgrammatic,
            },
        ));
        page.UpdateFrameIfNeeded().unwrap();
        assert!(page.CurrentFrame().unwrap().sequence > before);
        fn scroll_offset(fragment: PaintFragmentCursor<'_>, id: u64) -> Option<Offset> {
            if fragment.ViewNodeId() == id {
                return Some(fragment.paint.ScrollOffset());
            }
            fragment
                .Children()
                .find_map(|child| scroll_offset(child, id))
        }
        assert_eq!(
            scroll_offset(
                PaintFragmentCursor::FromHandle(&page.CurrentFrame().unwrap().fragments),
                scroller,
            ),
            Some(Offset { x: 0.0, y: 20.0 })
        );
        assert_eq!(
            page.cursor_hit_test_queries,
            queries + 1,
            "same point after real scroll needs a fresh hit"
        );
        page.UpdateCursor();
        assert_eq!(page.cursor_hit_test_queries, queries + 1);
        page.UpdateCursorFeedback(true);
        assert_eq!(
            page.cursor_hit_test_queries,
            queries + 2,
            "immediate mouse feedback remains forced"
        );
    });
}
// Font completion during a layout transaction cannot dispatch DOM listeners.
struct LayoutFontResourceClient<'a> {
    state: &'a PageState,
}
impl ResourceFetcherClient for LayoutFontResourceClient<'_> {
    fn HasJavaScript(&self) -> bool {
        false
    }
    fn ApplyResourceMutation(&mut self, mutation: ResourceMutation) {
        self.state.ApplyResourceMutation(mutation);
    }
    fn DispatchImageEvent(&mut self, _: u64, _: bool) {
        unreachable!("font completion cannot dispatch an image event");
    }
    fn DispatchStyleSheetEvent(&mut self, _: u64, _: String, _: bool) {
        unreachable!("font completion cannot dispatch a stylesheet event");
    }
}
struct ResourceClient<'a> {
    state: Rc<PageState>,
    scripts: Option<&'a mut PageScripts>,
}
impl ResourceFetcherClient for ResourceClient<'_> {
    fn HasJavaScript(&self) -> bool {
        self.scripts.is_some()
    }
    fn ApplyResourceMutation(&mut self, mutation: ResourceMutation) {
        self.state.ApplyResourceMutation(mutation);
    }
    fn DispatchImageEvent(&mut self, id: u64, loaded: bool) {
        self.scripts
            .as_deref_mut()
            .expect("image listeners require scripting")
            .DispatchImageEvent(id, loaded);
    }
    fn DispatchStyleSheetEvent(&mut self, id: u64, signature: String, loaded: bool) {
        self.scripts
            .as_deref_mut()
            .expect("stylesheet listeners require scripting")
            .DispatchStyleSheetEvent(id, signature, loaded);
    }
}
impl Drop for Page {
    fn drop(&mut self) {
        self.state.HideValidationMessage();
        self.state.local_frames.get().unwrap().DetachChildren();
        self.state.ShutdownDocumentEngines();
        self.state
            .document
            .borrow()
            .GetDocument()
            .SetLifecycleActive(false);
        self.state.CancelImageEvents();
        self.state.CancelStyleSheetEvents();
        self.state.connected_resources.SetScriptPreparer(None);
    }
}

// Mutation notification adapter. Image completion only registers a DOM task;
// it must not call listeners recursively inside the emitting host operation.
struct ScopedResourceClient<'a> {
    state: Rc<PageState>,
    runtime: &'a mut dyn JavaScriptHostRuntime,
}
impl ResourceFetcherClient for ScopedResourceClient<'_> {
    fn HasJavaScript(&self) -> bool {
        true
    }
    fn ApplyResourceMutation(&mut self, mutation: ResourceMutation) {
        self.state.ApplyResourceMutation(mutation);
    }
    fn DispatchImageEvent(&mut self, id: u64, loaded: bool) {
        self.state.QueueImageEvent(id, loaded);
    }
    fn DispatchStyleSheetEvent(&mut self, id: u64, signature: String, loaded: bool) {
        self.state.QueueStyleSheetEvent(id, signature, loaded);
    }
}
fn MakeResourceElementEvent(id: u64, loaded: bool) -> ::interaction::event::Event {
    let mut event = MakeSyntheticEvent(
        if loaded {
            EventType::kLoad
        } else {
            EventType::kCustom
        },
        id,
    );
    if !loaded {
        event.custom_type = "error".into();
    }
    event.bubbles = false;
    event.cancelable = false;
    event.composed = false;
    event.trusted = true;
    event
}
struct JavaScriptResourceClient<'a> {
    state: Rc<PageState>,
    bindings: Rc<RefCell<DOMJavaScriptBindings>>,
    runtime: &'a mut dyn JavaScriptRuntime,
    realm: &'a JavaScriptRealm,
    engine: ::interaction::Interaction<'static>,
}
impl ResourceFetcherClient for JavaScriptResourceClient<'_> {
    fn HasJavaScript(&self) -> bool {
        true
    }
    fn ApplyResourceMutation(&mut self, mutation: ResourceMutation) {
        self.state.ApplyResourceMutation(mutation);
    }
    fn DispatchImageEvent(&mut self, id: u64, loaded: bool) {
        self.state.QueueImageEvent(id, loaded);
    }
    fn DispatchStyleSheetEvent(&mut self, id: u64, signature: String, loaded: bool) {
        self.state.QueueStyleSheetEvent(id, signature, loaded);
    }
}
impl JavaScriptResourceClient<'_> {
    fn DispatchSelectionchangeEventTask(&mut self, id: u64) {
        let mut event = MakeSyntheticEvent(EventType::kCustom, id);
        event.custom_type = "selectionchange".into();
        let runtime = RefCell::new(&mut *self.runtime);
        let listeners = Rc::new(
            |invocation: &EventListenerInvocation<'_>,
             _: &::interaction::ownership::InteractionDocument,
             _: &::interaction::ownership::InteractionDOMMutationEmitter| {
                DispatchListeners(
                    &self.state,
                    &self.bindings,
                    invocation,
                    &mut **runtime.borrow_mut(),
                    self.realm,
                )
            },
        );
        self.engine.WithScopedListeners(listeners).DispatchDOMEvent(
            &mut event,
            id,
            &self.state.document.Handle(),
        );
    }

    fn DispatchImageEventTask(&mut self, id: u64, loaded: bool) {
        let mut event = MakeResourceElementEvent(id, loaded);
        let runtime = RefCell::new(&mut *self.runtime);
        let listeners = Rc::new(
            |invocation: &EventListenerInvocation<'_>,
             _: &::interaction::ownership::InteractionDocument,
             _: &::interaction::ownership::InteractionDOMMutationEmitter| {
                DispatchListeners(
                    &self.state,
                    &self.bindings,
                    invocation,
                    &mut **runtime.borrow_mut(),
                    self.realm,
                )
            },
        );
        self.engine.WithScopedListeners(listeners).DispatchDOMEvent(
            &mut event,
            id,
            &self.state.document.Handle(),
        );
        drop(runtime);
        // WindowJavaScriptBindings completes this DOM task with the normal
        // full microtask checkpoint and exception reporting, exactly once.
    }
    fn DispatchStyleSheetEventTask(&mut self, id: u64, loaded: bool) {
        let mut event = MakeResourceElementEvent(id, loaded);
        let runtime = RefCell::new(&mut *self.runtime);
        let listeners = Rc::new(
            |invocation: &EventListenerInvocation<'_>,
             _: &::interaction::ownership::InteractionDocument,
             _: &::interaction::ownership::InteractionDOMMutationEmitter| {
                DispatchListeners(
                    &self.state,
                    &self.bindings,
                    invocation,
                    &mut **runtime.borrow_mut(),
                    self.realm,
                )
            },
        );
        self.engine.WithScopedListeners(listeners).DispatchDOMEvent(
            &mut event,
            id,
            &self.state.document.Handle(),
        );
    }
}

// FrameSelection schedules `selectionchange` on the user-interaction task
// source. Keep the DOM's scheduled-target bit until the queued task begins so
// synchronous selection mutations coalesce exactly once.
fn EnqueueSelectionchangeEventTasks(
    state: &Rc<PageState>,
    window: &Rc<RefCell<WindowJavaScriptBindings>>,
    bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
    engine: &::interaction::Interaction<'static>,
) {
    let targets = std::mem::take(&mut *state.selectionchange_event_targets.borrow_mut());
    for target in targets {
        let weak_state = Rc::downgrade(state);
        let weak_bindings = Rc::downgrade(bindings);
        let engine = engine.clone();
        window.borrow_mut().EnqueueTaskWithRuntime(
            Box::new(move |runtime, realm| {
                let (Some(state), Some(bindings)) = (weak_state.upgrade(), weak_bindings.upgrade())
                else {
                    return;
                };
                let connected = {
                    let mut owner = state.document.borrow_mut();
                    let document = owner.GetDocumentMut();
                    document.ClearScheduledSelectionchangeEvent(target);
                    document.FindNodeById(target).is_some()
                };
                if connected {
                    JavaScriptResourceClient {
                        state,
                        bindings,
                        runtime,
                        realm,
                        engine,
                    }
                    .DispatchSelectionchangeEventTask(target);
                }
            }),
            None,
        );
    }
}

// ImageLoader::ImageNotifyFinished posts a cancellable DOM-manipulation task
// (blink/renderer/core/loader/image_loader.cc). Keep registration outside any
// borrowed Window host object: mutations can be emitted from a host callback.
fn EnqueueImageEventTasks(
    state: &Rc<PageState>,
    window: &Rc<RefCell<WindowJavaScriptBindings>>,
    bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
    engine: &::interaction::Interaction<'static>,
) {
    let events = std::mem::take(&mut *state.image_events.borrow_mut());
    for event in events {
        if !event.active.get() {
            continue;
        }
        let weak_state = Rc::downgrade(state);
        let weak_bindings = Rc::downgrade(bindings);
        let engine = engine.clone();
        window.borrow_mut().EnqueueTaskWithRuntime(
            Box::new(move |runtime, realm| {
                if !event.active.replace(false) {
                    return;
                }
                let (Some(state), Some(bindings)) = (weak_state.upgrade(), weak_bindings.upgrade())
                else {
                    return;
                };
                let delay = {
                    let owner = state.document.borrow();
                    let document = owner.GetDocument();
                    if document.FindNodeById(event.id).is_some_and(|node| {
                        document.NodeHandle(node) == event.ticket.loader.Element()
                    }) {
                        event.ticket.loader.BeginEventDispatch(&event.ticket)
                    } else {
                        None
                    }
                };
                if let Some(_delay) = delay {
                    let skip = {
                        let owner = state.document.borrow();
                        let document = owner.GetDocument();
                        document.FindNodeById(event.id).is_some_and(|index| {
                            document.HTMLImageLoaderShouldSkipEventDispatch(index)
                        })
                    };
                    if !skip {
                        JavaScriptResourceClient {
                            state: state.clone(),
                            bindings,
                            runtime,
                            realm,
                            engine,
                        }
                        .DispatchImageEventTask(event.id, event.loaded);
                    }
                }
                // A listener may have queued another request for the same element.
                let mut pending = state.pending_image_events.borrow_mut();
                if pending
                    .get(&event.id)
                    .is_some_and(|active| Rc::ptr_eq(active, &event.active))
                {
                    pending.remove(&event.id);
                }
            }),
            None,
        );
    }
}

// LinkStyle::NotifyLoadedSheetAndAllCriticalSubresources schedules, rather
// than synchronously dispatches, HTMLLinkElement's load/error event. Preserve
// the request signature so an event from an obsolete href cannot complete a
// newer Rspack/Webpack stylesheet request on the same element.
fn EnqueueStyleSheetEventTasks(
    state: &Rc<PageState>,
    window: &Rc<RefCell<WindowJavaScriptBindings>>,
    bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
    engine: &::interaction::Interaction<'static>,
) {
    let events = std::mem::take(&mut *state.style_sheet_events.borrow_mut());
    for event in events {
        if !event.active.get() {
            continue;
        }
        let weak_state = Rc::downgrade(state);
        let weak_bindings = Rc::downgrade(bindings);
        let engine = engine.clone();
        window.borrow_mut().EnqueueTaskWithRuntime(
            Box::new(move |runtime, realm| {
                if !event.active.replace(false) {
                    return;
                }
                let (Some(state), Some(bindings)) = (weak_state.upgrade(), weak_bindings.upgrade())
                else {
                    return;
                };
                let current_request = {
                    let owner = state.document.borrow();
                    let tree = owner.GetDocument();
                    tree.FindNodeById(event.id).is_some_and(|i| {
                        let node = tree.Node(i);
                        let attr = |name| {
                            node.FindAttribute(name)
                                .map_or("", |attribute| attribute.value.as_str())
                        };
                        node.IsHTMLElement("link")
                            && format!(
                                "{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}",
                                attr("href"),
                                attr("rel"),
                                attr("type"),
                                node.FindAttribute("disabled").is_some(),
                                attr("title"),
                                attr("media")
                            ) == event.signature
                    })
                };
                if current_request {
                    JavaScriptResourceClient {
                        state: state.clone(),
                        bindings,
                        runtime,
                        realm,
                        engine,
                    }
                    .DispatchStyleSheetEventTask(event.id, event.loaded);
                }
                let mut pending = state.pending_style_sheet_events.borrow_mut();
                if pending
                    .get(&event.id)
                    .is_some_and(|active| Rc::ptr_eq(active, &event.active))
                {
                    pending.remove(&event.id);
                }
            }),
            None,
        );
    }
}

// cpp: browser/browser.cc:702-734
fn DispatchListeners(
    state: &PageState,
    bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
    invocation: &EventListenerInvocation<'_>,
    runtime: &mut dyn JavaScriptRuntime,
    realm: &JavaScriptRealm,
) -> ::interaction::event::EventListenerResult {
    DOMJavaScriptBindings::DispatchEventListeners(
        bindings,
        invocation,
        runtime,
        realm,
        &mut |error| state.client.borrow_mut().DidReportScriptError(error),
    )
}

// cpp: browser/browser.cc:1859-1877
/// Real network navigation with the translated Rust runtime and Page services.
/// The returned Page owns its original DOM, runtime, resources and latest frame.
pub fn OpenUrl(
    url: &str,
    width: u32,
    height: u32,
    client: Rc<RefCell<dyn PageClient>>,
) -> io::Result<Page> {
    let options = url_loader::DefaultURLLoaderOptions::default();
    let loader = url_loader::DefaultURLLoader::new(options.clone())?;
    OpenUrlWithDefaultLoader(url, width, height, client, options, loader)
}

pub(crate) fn OpenUrlWithDefaultLoader(
    url: &str,
    width: u32,
    height: u32,
    client: Rc<RefCell<dyn PageClient>>,
    options: url_loader::DefaultURLLoaderOptions,
    loader: url_loader::DefaultURLLoader,
) -> io::Result<Page> {
    let loader = Rc::new(RefCell::new(loader));
    let xhr = xhr_transport::CreateHTTPXMLHttpRequestTransport(&options)?;
    let registry = crate::CreateLayoutRegistry();
    let mut page = Page::Create(
        loader,
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(crate::CreateSVGImageDecoder(&registry))),
        crate::CreateBrowserConstraints(width, height),
        Some(ScriptEnvironment {
            runtime: Box::new(
                javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::new(),
            ),
            xhr: xhr,
            user_agent: options.user_agent,
        }),
        Some(client),
    );
    page.Open(url, 16384, 4096)?;
    while page.IsLoading() {
        page.RunFor(std::time::Duration::from_millis(1))?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    Ok(page)
}

#[derive(Default)]
pub(crate) struct NullPageClient(pub PageChromeState);
impl PageClient for NullPageClient {
    fn DidEventHandlerChromeEffect(&mut self, effect: EventHandlerChromeEffect) {
        self.0.Apply(effect);
    }
}
impl Page {
    /// Plan and raster the current Page artifact into the host's borrowed target.
    pub fn PaintInto(
        &self,
        width: u32,
        height: u32,
        scale: f64,
        target: &mut [u32],
        format: PixelFormat,
        stride: usize,
    ) -> io::Result<RenderUpdate> {
        // The render path pushes the artifact to Layerize, which retains the
        // PaintController's Arc (local_frame_view.cc:3348-3351); release the
        // PaintEngine borrow before rendering.
        let artifact = self
            .CurrentPaintArtifact()
            .map(|artifact| Arc::clone(&artifact))
            .ok_or_else(|| io::Error::other("Page has no frame"))?;
        self.PaintArtifactInto(&artifact, width, height, scale, target, format, stride)
    }

    /// The host may append transient chrome content to this Page's recording.
    /// Both paths use the same Page-owned Compositor, Viz and Renderer instances,
    /// so retained identities stay coherent.
    pub fn PaintArtifactInto(
        &self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
        target: &mut [u32],
        format: PixelFormat,
        stride: usize,
    ) -> io::Result<RenderUpdate> {
        self.state
            .Render(artifact, width, height, scale, target, format, stride)
    }
}

mod resident_editing_positions;
