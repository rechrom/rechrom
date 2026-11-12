#![allow(non_snake_case)]
//! Persistent Page lifecycle. All frames and measurements consume the original
//! parser arena. Module loading and the remaining DOM host continuations are
//! tracked separately; this service is not evidence of complete browser parity.
//! Page connects DOM mutation notifications and the style/layout/paint lifecycle;
//! each engine owns its state and invalidation, like LocalFrameView lifecycle wiring.
pub(crate) mod canvas_2d;
use crate::interaction_services::PageInteraction;
pub use compositor::CompositorMainThreadEffect;
use compositor::{
    adapter::{
        begin_frame::{BeginFrameArgs, BeginFrameArgsType, BeginFrameId},
        geometry::{
            PointF as CompositorPointF, Rect as CompositorRect, RectF as CompositorRectF,
            Size as CompositorSize, Vector2dF,
        },
        layer_tree_impl::{Color4f as CompositorColor4f, ResourceId},
        time::{TimeDelta as CompositorTimeDelta, TimeTicks as CompositorTimeTicks},
    },
    CompositorConfig, CompositorEngine, CompositorError, CompositorExecutor, CompositorExecutors,
    CompositorFrameSubmission, CompositorInput, CompositorOutput, CompositorViewportState,
    ScrollPhase,
};
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
use layout_fragment::LayoutResultHandle;
use layoutng_assembly::layout_boundary_support::LayoutUpdateKind;
use layoutng_assembly::layout_engine::{LayoutPaintOutput, PaintFragmentCursor};
use layoutng_assembly::{
    editing_state::DocumentSelection,
    fragment_tree::{FragmentKind, FragmentPaintData, PaintResources},
    internal::{
        layout_input::{ConstraintSpace, Offset},
        layout_input_types::{Color, IntSize},
    },
};
pub use open::OpenStatus;
use open::{script_scheduler::ScriptLoadClient, OpenEngine};
use page_mutation::{PageMutation, ResourceMutation};
pub use paint::paint_engine::{CaretGeometry, PaintRect as CaretRect};
use raster::RasterEngine;
use renderer::RenderUpdate;
use renderer_skia::{PixelFormat, RendererSkiaBackend};
use resource::ResourceEngine;
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap, HashSet},
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
impl CompositorExecutor for PageCompositorExecutor {
    fn Now(&self) -> CompositorTimeTicks {
        self.TimeTicks(std::time::Instant::now())
    }

    fn Execute(&self, task: Box<dyn FnOnce() + Send>) {
        let _ = self.executor.execute(task);
    }

    fn ExecuteAfter(&self, delay: CompositorTimeDelta, task: Box<dyn FnOnce() + Send>) {
        let delay = delay.InMicroseconds().max(0) as u64;
        let _ = self
            .executor
            .execute_after(std::time::Duration::from_micros(delay), task);
    }

    fn RunsTasksInCurrentSequence(&self) -> bool {
        std::thread::current().id() == self.sequence
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PreparedRenderInput {
    width: u32,
    height: u32,
    scale: u64,
    layout_zoom: u64,
}
struct PendingPageRender {
    artifact: Arc<paint::PaintArtifact>,
    input: PreparedRenderInput,
    viewport: CompositorViewportState,
    needs_begin_frames: bool,
}
impl PendingPageRender {
    fn Matches(
        &self,
        artifact: &Arc<paint::PaintArtifact>,
        input: PreparedRenderInput,
        viewport: CompositorViewportState,
    ) -> bool {
        Arc::ptr_eq(&self.artifact, artifact)
            && self.input == input
            && self.viewport.size == viewport.size
            && self.viewport.device_scale_factor.to_bits() == viewport.device_scale_factor.to_bits()
            && self.viewport.page_scale_factor.to_bits() == viewport.page_scale_factor.to_bits()
            && self.viewport.min_page_scale_factor.to_bits()
                == viewport.min_page_scale_factor.to_bits()
            && self.viewport.max_page_scale_factor.to_bits()
                == viewport.max_page_scale_factor.to_bits()
            && self.viewport.visible == viewport.visible
    }
}
struct PresentablePageRender {
    artifact: std::sync::Weak<paint::PaintArtifact>,
    input: PreparedRenderInput,
}
#[derive(Clone, Copy, Debug)]
pub struct PageCompositorClockMapping {
    origin: std::time::Instant,
}
impl PageCompositorClockMapping {
    fn Instant(self, ticks: CompositorTimeTicks) -> Option<std::time::Instant> {
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

    fn TimeTicks(self, instant: std::time::Instant) -> CompositorTimeTicks {
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
pub struct PageRenderPipeline {
    page_scale: (f32, f32, f32),
    compositor: Option<CompositorEngine>,
    scroll_animation_commit_requests: Vec<compositor::adapter::input_types::ElementId>,
    executor: Arc<PageCompositorExecutor>,
    raster_executor: Arc<PageCompositorExecutor>,
    raster_engine: RasterEngine,
    // Present only for synchronous embedding. External Page producers stop at
    // CompositorFrame and submit into the window-owned shared Viz service.
    viz: Option<viz::adapter::VizEngine>,
    renderer: Option<renderer::Renderer<RendererSkiaBackend>>,
    surface: viz::adapter::SurfaceId,
    next_surface: u64,
    local_surface_id: Option<compositor::runtime::types::LocalSurfaceId>,
    surface_parameters: Option<(u32, u32, u32)>,
    rendered_surface: Option<viz::adapter::SurfaceId>,
    external_presentation: bool,
    transparent_background: bool,
    outbound_frames: std::collections::VecDeque<CompositorFrameSubmission>,
    outbound_child_frames: std::collections::VecDeque<PageChildCompositorFrame>,
    video_surfaces: HashMap<u64, VideoSurfaceEntry>,
    deferred_effects: Vec<CompositorMainThreadEffect>,
    commit_finish_times: Vec<std::time::Instant>,
    begin_frame_sequence: u64,
    video_frame_token: u32,
    needs_begin_frames: bool,
    work_pending: bool,
    // Manual driving is confined to the synchronous internal presenter.
    // External presentation receives only the host's physical BeginFrames.
    input_frame_issued_since_physical: bool,
    // The synthetic input BeginFrame may need one or more owner-sequence
    // Polls to reach ScheduledActionDraw. Keep those Polls runnable, then hold
    // later input work for the next physical interval once a frame is sent.
    input_frame_pump_pending: bool,
    // A budget-limited Poll may leave already runnable typed impl tasks. This
    // is narrower than `work_pending`, which input also uses to describe work
    // that an executor or the next BeginFrame will wake independently.
    owner_turn_pending: bool,
    // A main-frame input remains strongly identified until cc consumes it at
    // ScheduledActionCommit. Drawing and FrameSink submission are allowed to
    // follow asynchronously after this slot is released.
    pending: Option<PendingPageRender>,
    // A weak identity marker avoids forcing PaintEngine's next property-only
    // scroll update to deep-copy the complete PaintArtifact.
    presentable: Option<PresentablePageRender>,
}
#[derive(Default)]
pub struct CompositorWheelResult {
    pub dom_ack: Option<u64>,
    /// Original gesture completions, including deferred hit-test dispatches.
    pub gesture_acks: Vec<(u64, bool)>,
    pub compositor_owned: bool,
    pub submitted: bool,
    pub effects: Vec<CompositorMainThreadEffect>,
}
pub enum PageBrowserInput {
    DocumentDetached { generation: u64 },
    Compositor(compositor::CompositorMutation),
    Gesture(interaction::input_event::WebGestureEvent),
}
pub type PageCompositorFrame = CompositorFrameSubmission;
/// A delegated video child surface submission. The Page producer allocates
/// the stable public surface identity referenced by its SurfaceDrawQuad; the
/// window-owned Viz service consumes this frame before the parent frame.
pub struct PageChildCompositorFrame {
    pub surface_id: viz::adapter::SurfaceId,
    pub frame: compositor::CompositorFrame,
}
pub type PageReturnedResource = compositor::runtime::resources::RasterReturnedResource;
impl PageRenderPipeline {
    pub fn PresentationClockMapping(&self) -> PageCompositorClockMapping {
        PageCompositorClockMapping {
            origin: self.executor.origin,
        }
    }

    pub fn ReceivePresentationFeedback(
        &mut self,
        feedback: PageFramePresentationFeedback,
    ) -> io::Result<CompositorWheelResult> {
        let PageFramePresentationFeedback {
            surface_id,
            viz_submission,
            frame_token,
            frame_id,
            outcome,
        } = feedback;
        let PagePresentationOutcome::Presented {
            timestamp,
            interval,
            flags,
        } = outcome
        else {
            let PagePresentationOutcome::Unavailable(reason) = outcome else {
                unreachable!()
            };
            let reason = match reason {
                PagePresentationUnavailableReason::BackendHasNoTimestamp => {
                    compositor::PresentationUnavailableReason::BackendHasNoTimestamp
                }
                PagePresentationUnavailableReason::OffscreenTarget => {
                    compositor::PresentationUnavailableReason::OffscreenTarget
                }
                PagePresentationUnavailableReason::DiscardedBeforePresentation => {
                    compositor::PresentationUnavailableReason::DiscardedBeforePresentation
                }
            };
            return Ok(CompositorWheelResult {
                effects: vec![
                    CompositorMainThreadEffect::PresentationFeedbackUnavailable {
                        frame_token,
                        surface_id,
                        viz_submission,
                        reason,
                    },
                ],
                ..Default::default()
            });
        };
        let mapping = self.PresentationClockMapping();
        let mut details = compositor::engine::CompositorFrameTimingDetails::default();
        details.frame_id = frame_id;
        details.presentation_feedback.timestamp = mapping.TimeTicks(timestamp);
        details.presentation_feedback.interval =
            CompositorTimeDelta::Microseconds(interval.as_micros().min(i64::MAX as u128) as i64);
        details.presentation_feedback.flags = flags;
        let output = self
            .compositor
            .as_mut()
            .ok_or_else(|| io::Error::other("presentation feedback requires a compositor"))?
            .DidPresentFrame(frame_token, details)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let (submitted, _, _, _) = self.AcceptOutput(output)?;
        Ok(CompositorWheelResult {
            submitted,
            effects: self.TakeMainThreadEffects(),
            ..Default::default()
        })
    }

    pub fn new(executor: Arc<dyn executor::Executor>) -> Self {
        Self::NewWithPresentation(executor.clone(), executor, false, false)
    }

    pub fn new_with_executors(
        compositor: Arc<dyn executor::Executor>,
        raster: Arc<dyn executor::Executor>,
    ) -> Self {
        Self::NewWithPresentation(compositor, raster, false, false)
    }

    /// Builds only the Page-owned cc producer. The embedder's shared Viz service
    /// consumes the move-only CompositorFrame after the FrameSink boundary, so
    /// impl-thread input never waits for display aggregation or drawing.
    pub fn new_external(
        compositor: Arc<dyn executor::Executor>,
        raster: Arc<dyn executor::Executor>,
        transparent_background: bool,
    ) -> Self {
        Self::NewWithPresentation(compositor, raster, true, transparent_background)
    }

    fn NewWithPresentation(
        compositor_executor: Arc<dyn executor::Executor>,
        raster_executor: Arc<dyn executor::Executor>,
        external_presentation: bool,
        transparent_background: bool,
    ) -> Self {
        static NEXT_COMMAND_BUFFER: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(1);
        let command_buffer = NEXT_COMMAND_BUFFER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let raster_engine = RasterEngine::new(command_buffer, 64 * 1024 * 1024);
        Self {
            page_scale: (1.0, 1.0, 1.0),
            compositor: None,
            scroll_animation_commit_requests: Vec::new(),
            executor: Arc::new(PageCompositorExecutor::new(compositor_executor)),
            raster_executor: Arc::new(PageCompositorExecutor::new(raster_executor)),
            raster_engine,
            viz: (!external_presentation).then(viz::adapter::VizEngine::New),
            renderer: (!external_presentation)
                .then(|| renderer::Renderer::new(RendererSkiaBackend::default())),
            surface: viz::adapter::SurfaceId(1),
            next_surface: 2,
            local_surface_id: None,
            surface_parameters: None,
            rendered_surface: None,
            external_presentation,
            transparent_background,
            outbound_frames: Default::default(),
            outbound_child_frames: Default::default(),
            video_surfaces: HashMap::new(),
            deferred_effects: Vec::new(),
            commit_finish_times: Vec::new(),
            begin_frame_sequence: 0,
            video_frame_token: 0,
            needs_begin_frames: false,
            work_pending: false,
            input_frame_issued_since_physical: false,
            input_frame_pump_pending: false,
            owner_turn_pending: false,
            pending: None,
            presentable: None,
        }
    }

    /// Selects the embedder-owned Viz boundary before this producer creates
    /// any cc state. The desktop runtime owns the real root compositor on its
    /// impl sequence, but media discovery happens on the document sequence;
    /// its VideoFrameSubmitter therefore stops at a delegated child
    /// CompositorFrame and lets the runtime transport that frame with the next
    /// parent submission.
    ///
    /// Source: media/blink/webmediaplayer_impl.cc and
    ///         media/blink/video_frame_submitter.cc.
    fn SetExternalPresentation(&mut self, external: bool) {
        assert!(
            self.compositor.is_none()
                && self.outbound_frames.is_empty()
                && self.outbound_child_frames.is_empty()
                && self.video_surfaces.is_empty(),
            "presentation ownership must be selected before compositor initialization"
        );
        if self.external_presentation == external {
            return;
        }
        self.external_presentation = external;
        if external {
            self.viz = None;
            self.renderer = None;
        } else {
            self.viz = Some(viz::adapter::VizEngine::New());
            self.renderer = Some(renderer::Renderer::new(RendererSkiaBackend::default()));
        }
    }

    fn BuildVideoChildFrame(
        &mut self,
        video: &image_resource::VideoFrameResource,
    ) -> io::Result<compositor::CompositorFrame> {
        let resource = self
            .raster_engine
            .CreateImmutableVideoTransferableResource(video)?;
        let width = i32::try_from(video.width)
            .map_err(|_| io::Error::other("video frame width overflow"))?;
        let height = i32::try_from(video.height)
            .map_err(|_| io::Error::other("video frame height overflow"))?;
        let rect = CompositorRect::new(0, 0, width, height);
        let mut pass = compositor::CompositorRenderPass::new(1);
        pass.id = compositor::runtime::frame::RenderPassId(1);
        pass.output_rect = rect;
        pass.damage_rect = rect;
        pass.has_damage_from_contributing_content = true;
        pass.has_transparent_background = false;
        let shared = pass.AppendSharedQuadState();
        {
            let state = pass.SharedQuadStateMutable(shared);
            state.quad_layer_rect = rect;
            state.visible_quad_layer_rect = rect;
            state.are_contents_opaque = true;
            state.opacity = 1.0;
        }
        pass.AppendTextureQuad(
            shared,
            rect,
            rect,
            false,
            ResourceId(resource.id.0),
            CompositorRectF::new(0.0, 0.0, 1.0, 1.0),
            true,
            CompositorColor4f::default(),
            false,
            false,
            true,
            true,
        );
        let mut frame = compositor::CompositorFrame::default();
        self.video_frame_token = self.video_frame_token.wrapping_add(1).max(1);
        frame.metadata.device_scale_factor = 1.0;
        frame.metadata.page_scale_factor = 1.0;
        frame.metadata.may_contain_video = true;
        frame.metadata.frame_token = self.video_frame_token;
        frame.metadata.visible_viewport_size = CompositorSize::new(width, height);
        frame.metadata.root_background_color = CompositorColor4f {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        frame.render_pass_list.push(Box::new(pass));
        frame.resource_list.push(resource);
        Ok(frame)
    }

    fn SubmitVideoChildFrame(
        &mut self,
        surface_id: viz::adapter::SurfaceId,
        video: &image_resource::VideoFrameResource,
    ) -> io::Result<()> {
        let frame = self.BuildVideoChildFrame(video)?;
        if self.external_presentation {
            if let Some(pending) = self
                .outbound_child_frames
                .iter_mut()
                .find(|pending| pending.surface_id == surface_id)
            {
                pending.frame = frame;
            } else {
                self.outbound_child_frames
                    .push_back(PageChildCompositorFrame { surface_id, frame });
            }
        } else {
            self.viz
                .as_mut()
                .expect("internal presentation owns Viz")
                .SubmitFrame(surface_id, frame)
                .map_err(|error| io::Error::other(error.to_string()))?;
        }
        Ok(())
    }

    fn PublishVideoFrame(
        &mut self,
        node_id: u64,
        video: &image_resource::VideoFrameResource,
    ) -> io::Result<Arc<dyn paint::paint_engine::ForeignLayer>> {
        const VIDEO_SURFACE_NAMESPACE: u64 = 1_u64 << 32;
        static NEXT_VIDEO_SURFACE: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(VIDEO_SURFACE_NAMESPACE);

        self.EnsureCompositor(None)?;
        let frame_size = (video.width, video.height);
        let replace_surface = self
            .video_surfaces
            .get(&node_id)
            .is_none_or(|current| current.frame_size != frame_size);
        if replace_surface {
            let raw = NEXT_VIDEO_SURFACE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if raw < VIDEO_SURFACE_NAMESPACE || raw >= (1_u64 << 63) {
                return Err(io::Error::other("video SurfaceId namespace exhausted"));
            }
            let surface_id = viz::adapter::SurfaceId(raw);
            let range = viz::adapter::VizEngine::RuntimeSurfaceRange(surface_id);
            let layer = self
                .compositor
                .as_mut()
                .expect("initialized compositor")
                .CreateVideoSurfaceForeignLayer(range);
            self.video_surfaces.insert(
                node_id,
                VideoSurfaceEntry {
                    surface_id,
                    frame_size,
                    layer,
                },
            );
        }
        let (surface_id, layer) = {
            let entry = self
                .video_surfaces
                .get(&node_id)
                .expect("video surface entry was created");
            (entry.surface_id, Arc::clone(&entry.layer))
        };
        self.SubmitVideoChildFrame(surface_id, video)?;
        Ok(layer)
    }

    fn SubmitInternalFrame(
        &mut self,
        submission: CompositorFrameSubmission,
    ) -> io::Result<(
        bool,
        Vec<compositor::runtime::resources::RasterReturnedResource>,
    )> {
        let local_surface_id = submission.local_surface_id;
        let frame = submission.frame;
        let frame_token = frame.metadata.frame_token;
        let root = frame
            .render_pass_list
            .last()
            .ok_or_else(|| io::Error::other("compositor frame has no root render pass"))?;
        let parameters = (
            root.output_rect.width.max(0) as u32,
            root.output_rect.height.max(0) as u32,
            frame.metadata.device_scale_factor.to_bits(),
        );
        let mut returned = Vec::new();
        if let Some(current) = self.local_surface_id {
            let same_embed = current.embed_token == local_surface_id.embed_token;
            if same_embed && !local_surface_id.IsSameOrNewerThan(&current) {
                returned.extend(frame.resource_list.iter().map(
                    compositor::runtime::resources::RasterTransferableResource::ToReturnedResource,
                ));
                return Ok((false, returned));
            }
        }
        let generation_changed = self
            .local_surface_id
            .is_some_and(|current| current != local_surface_id);
        if !generation_changed
            && self
                .surface_parameters
                .is_some_and(|current| current != parameters)
        {
            return Err(io::Error::other(
                "CompositorFrame dimensions changed without a new LocalSurfaceId",
            ));
        }
        if generation_changed {
            if let Some(resources) = self
                .viz
                .as_mut()
                .expect("internal presentation owns Viz")
                .DestroySurface(self.surface)
            {
                returned.extend(resources);
            }
            self.surface = viz::adapter::SurfaceId(self.next_surface);
            self.next_surface = self.next_surface.wrapping_add(1).max(1);
            self.rendered_surface = None;
        }
        self.local_surface_id = Some(local_surface_id);
        self.surface_parameters = Some(parameters);
        let accepted = self
            .viz
            .as_mut()
            .expect("internal presentation owns Viz")
            .SubmitFrame(self.surface, frame)
            .map_err(|error| io::Error::other(error.to_string()))?;
        returned.extend(accepted.returned_resources);
        // The in-process CPU target has no native presentation clock. This
        // reports the actual accepted Viz identity without manufacturing time.
        self.deferred_effects.push(
            CompositorMainThreadEffect::PresentationFeedbackUnavailable {
                frame_token,
                surface_id: self.surface.0,
                viz_submission: accepted.submission,
                reason: compositor::PresentationUnavailableReason::OffscreenTarget,
            },
        );
        Ok((true, returned))
    }

    fn EnsureCompositor(&mut self, viewport: Option<CompositorViewportState>) -> io::Result<()> {
        if self.compositor.is_none() {
            let viewport = viewport.unwrap_or(CompositorViewportState {
                size: CompositorSize::default(),
                device_scale_factor: 1.0,
                page_scale_factor: 1.0,
                min_page_scale_factor: 1.0,
                max_page_scale_factor: 1.0,
                visible: true,
            });
            let executor: Arc<dyn CompositorExecutor> = self.executor.clone();
            let raster_provider = Arc::new(self.raster_engine.clone());
            let ui_raster = raster_provider.clone();
            self.compositor = Some(
                CompositorEngine::new(
                    CompositorConfig {
                        transparent_background: self.transparent_background,
                        initial_viewport_size: viewport.size,
                        initial_device_scale_factor: viewport.device_scale_factor,
                        ..Default::default()
                    },
                    CompositorExecutors {
                        compositor: executor.clone(),
                        raster: self.raster_executor.clone(),
                        raster_provider,
                        ui_raster,
                    },
                )
                .map_err(|error| io::Error::other(error.to_string()))?,
            );
        }
        Ok(())
    }

    /// Supplies the widget state before Page asks cc for its first
    /// BeginMainFrame. No document tree or PaintArtifact crosses this setup
    /// boundary.
    pub fn InitializeViewport(&mut self, width: u32, height: u32, scale: f64) -> io::Result<()> {
        if !scale.is_finite() || scale <= 0.0 || scale > f32::MAX as f64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "viewport scale must be positive",
            ));
        }
        let viewport = CompositorViewportState {
            size: CompositorSize::new(
                i32::try_from(width).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "viewport width is too large")
                })?,
                i32::try_from(height).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "viewport height is too large")
                })?,
            ),
            device_scale_factor: scale as f32,
            page_scale_factor: self.page_scale.0,
            min_page_scale_factor: self.page_scale.1,
            max_page_scale_factor: self.page_scale.2,
            visible: true,
        };
        self.EnsureCompositor(Some(viewport))?;
        let output = self
            .compositor
            .as_mut()
            .expect("initialized compositor")
            .SetViewport(viewport)
            .map_err(|error| io::Error::other(error.to_string()))?;
        self.AcceptOutput(output)?;
        Ok(())
    }

    fn AcceptOutput(&mut self, first: CompositorOutput) -> io::Result<(bool, bool, bool, bool)> {
        let mut pending = std::collections::VecDeque::from([first]);
        let mut submitted = false;
        let mut submitted_latest_commit = false;
        let mut needs_begin_frames = false;
        let mut main_commit_pending = false;
        let mut turns = 0usize;
        while let Some(mut output) = pending.pop_front() {
            turns += 1;
            if turns > 64 {
                return Err(io::Error::other(
                    "compositor output did not reach a stable frame boundary",
                ));
            }
            // Each output reports the engine's current subscription state.
            // Ack/reclaim outputs later in this queue supersede earlier values.
            needs_begin_frames = output.needs_begin_frames;
            main_commit_pending = output.main_commit_pending;
            self.needs_begin_frames = needs_begin_frames;
            self.work_pending = output.work_pending;
            if output.main_thread_effects.iter().any(|effect| {
                matches!(
                    effect,
                    CompositorMainThreadEffect::MainFrameCommitDeferred { .. }
                )
            }) {
                // ProxyMain aborted this main-frame input before cc commit.
                // Release only its in-flight slot; it never became presentable.
                self.pending.take();
            }
            for effect in &output.main_thread_effects {
                if let CompositorMainThreadEffect::ViewportScaleChanged { scale, .. } = effect {
                    self.page_scale.0 = *scale;
                }
            }
            for effect in output.main_thread_effects.drain(..) {
                if self.external_presentation {
                    if let CompositorMainThreadEffect::MainFrameCommitCompleted {
                        commit_finish_time,
                    } = &effect
                    {
                        if let Some(instant) =
                            self.PresentationClockMapping().Instant(*commit_finish_time)
                        {
                            self.commit_finish_times.push(instant);
                        }
                        continue;
                    }
                }
                self.deferred_effects.push(effect);
            }
            let frame_matches_latest_commit = output.frame_matches_latest_commit;
            let Some(frame) = output.frame else {
                continue;
            };
            if self.external_presentation {
                submitted = true;
                submitted_latest_commit |= frame_matches_latest_commit;
                let compositor = self.compositor.as_mut().expect("initialized compositor");
                // The move into the Page FrameSink queue transfers ownership;
                // Chromium acknowledges submission at this boundary, before
                // Viz draws the frame. Resource returns arrive separately when
                // Viz replaces the prior surface frame.
                self.outbound_frames.push_back(frame);
                pending.push_back(
                    compositor
                        .DidReceiveFrameAck()
                        .map_err(|error| io::Error::other(error.to_string()))?,
                );
            } else {
                let (accepted, returned_resources) = self.SubmitInternalFrame(frame)?;
                submitted |= accepted;
                submitted_latest_commit |= accepted && frame_matches_latest_commit;
                let compositor = self.compositor.as_mut().expect("initialized compositor");
                pending.push_back(
                    compositor
                        .DidReceiveFrameAck()
                        .map_err(|error| io::Error::other(error.to_string()))?,
                );
                if !returned_resources.is_empty() {
                    pending.push_back(
                        compositor
                            .ReclaimResources(returned_resources)
                            .map_err(|error| io::Error::other(error.to_string()))?,
                    );
                }
            }
        }
        // CompositorOutput::work_pending is the Engine owner-turn contract:
        // Poll exhausted its bounded turn while typed impl tasks remain
        // immediately runnable. Preserve an already-posted continuation and
        // make the final stable output from the Ack/Reclaim chain visible to
        // NeedsPump(). Chromium continues ProcessScheduledActions on the impl
        // sequence and can still replace the current deadline with IMMEDIATE;
        // waiting for an unrelated executor wake or the next physical frame
        // would add a full resize frame here.
        self.owner_turn_pending |= self.work_pending;
        Ok((
            submitted,
            submitted_latest_commit,
            needs_begin_frames,
            main_commit_pending,
        ))
    }

    fn AcceptPendingOutput(&mut self, output: CompositorOutput) -> io::Result<bool> {
        // Source: cc/trees/single_thread_proxy.cc CompositeImmediately and
        // Scheduler::DidNotProduceFrame. Once Poll is stable, requests neither
        // another BeginFrame nor main-frame work, and emitted no replacement,
        // the committed main frame is complete and keeps the existing surface.
        // Waiting for a CompositorFrame here would block the next artifact
        // behind a frame Chromium intentionally does not emit.
        let stable_frame_reuse = output.frame.is_none()
            && !output.needs_begin_frames
            && !output.main_thread_effects.iter().any(|effect| {
                matches!(
                    effect,
                    CompositorMainThreadEffect::BeginMainFrame
                        | CompositorMainThreadEffect::RequestMainFrame
                        | CompositorMainThreadEffect::MainFrameCommitDeferred { .. }
                )
            });
        let (submitted, submitted_latest_commit, needs_begin_frames, main_commit_pending) =
            self.AcceptOutput(output)?;
        if let Some(pending) = self.pending.as_mut() {
            pending.needs_begin_frames = needs_begin_frames;
        }
        let internal_presentable =
            !main_commit_pending && (submitted_latest_commit || stable_frame_reuse);
        // ProxyImpl releases DataForCommit at ScheduledActionCommit, so the
        // externally driven Page releases its commit slot at that boundary.
        // The synchronous internal presenter additionally uses this slot as
        // its call-scoped completion guard and waits for the matching frame.
        if !main_commit_pending && (self.external_presentation || internal_presentable) {
            if let Some(pending) = self.pending.take() {
                self.presentable = Some(PresentablePageRender {
                    artifact: Arc::downgrade(&pending.artifact),
                    input: pending.input,
                });
            }
        }
        Ok(if self.external_presentation {
            submitted || stable_frame_reuse
        } else {
            internal_presentable
        })
    }

    fn ContinuePending(&mut self) -> io::Result<bool> {
        let output = self
            .compositor
            .as_mut()
            .expect("initialized compositor")
            .Poll()
            .map_err(|error| io::Error::other(error.to_string()))?;
        if self.AcceptPendingOutput(output)? {
            return Ok(true);
        }
        // The external FrameSink is driven by the host BeginFrame source.
        // Polling a pending commit must not invent additional frame clocks.
        // Source: cc/trees/proxy_impl.cc and cc/scheduler/scheduler.cc
        //         OnBeginFrame / ProcessScheduledActions.
        if self.external_presentation {
            return Ok(false);
        }
        for _ in 0..8 {
            if !self
                .pending
                .as_ref()
                .is_some_and(|pending| pending.needs_begin_frames)
            {
                break;
            }
            let output = self.BeginFrame()?;
            if self.AcceptPendingOutput(output)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn BeginFrame(&mut self) -> io::Result<CompositorOutput> {
        self.begin_frame_sequence = self.begin_frame_sequence.wrapping_add(1).max(1);
        let now = self.executor.Now();
        let interval = BeginFrameArgs::DefaultInterval();
        let mut trace = browser_tracing::span("compositor", "PageCompositor.ManualBeginFrame");
        trace.set(
            "external_presentation",
            self.external_presentation as u8 as f64,
        );
        trace.set("sequence", self.begin_frame_sequence as f64);
        self.compositor
            .as_mut()
            .expect("initialized compositor")
            .BeginFrame(BeginFrameArgs {
                frame_time: now,
                deadline: now + interval,
                interval,
                unthrottled_interval: interval,
                frame_id: BeginFrameId {
                    source_id: BeginFrameArgs::kManualSourceId,
                    sequence_number: self.begin_frame_sequence,
                },
                trace_id: self.begin_frame_sequence as i64,
                dispatch_time: now,
                client_arrival_time: now,
                type_: BeginFrameArgsType::NORMAL,
                ..BeginFrameArgs::default()
            })
            .map_err(|error| io::Error::other(error.to_string()))
    }

    pub fn BeginPhysicalFrame(
        &mut self,
        args: foundation::begin_frame::BeginFrameArgs,
    ) -> io::Result<CompositorWheelResult> {
        // The internal presenter advances its scheduler with BeginFrame().
        // Routing a host frame into that same scheduler mixes two clock
        // sources and can move frame_time backwards. Only externally
        // presented Pages subscribe to the host's physical BeginFrames.
        if !self.external_presentation || self.compositor.is_none() {
            return Ok(CompositorWheelResult::default());
        }
        let mut trace = browser_tracing::span("compositor", "PageCompositor.PhysicalBeginFrame");
        trace.set("source_id", args.source_id as f64);
        trace.set("sequence", args.sequence_number as f64);
        self.input_frame_issued_since_physical = false;
        self.input_frame_pump_pending = false;
        let interval = CompositorTimeDelta::Microseconds(
            args.interval.as_micros().min(i64::MAX as u128) as i64,
        );
        let profile = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        let output = self
            .compositor
            .as_mut()
            .expect("checked compositor")
            .BeginFrame(BeginFrameArgs {
                frame_time: self.executor.TimeTicks(args.frame_time),
                deadline: self.executor.TimeTicks(args.deadline),
                interval,
                unthrottled_interval: interval,
                frame_id: BeginFrameId {
                    source_id: args.source_id,
                    sequence_number: args.sequence_number,
                },
                trace_id: args.sequence_number.min(i64::MAX as u64) as i64,
                dispatch_time: self.executor.Now(),
                client_arrival_time: self.executor.Now(),
                type_: BeginFrameArgsType::NORMAL,
                ..BeginFrameArgs::default()
            })
            .map_err(|error| io::Error::other(error.to_string()))?;
        let began_at = profile.map(|_| std::time::Instant::now());
        let mut result = CompositorWheelResult::default();
        let mut output = output;
        let stable_frame_reuse = self.pending.is_some()
            && output.frame.is_none()
            && !output.needs_begin_frames
            && !output.main_thread_effects.iter().any(|effect| {
                matches!(
                    effect,
                    CompositorMainThreadEffect::BeginMainFrame
                        | CompositorMainThreadEffect::RequestMainFrame
                )
            });
        for effect in std::mem::take(&mut output.main_thread_effects) {
            if let CompositorMainThreadEffect::GestureDisposition {
                dispatch_id,
                handled,
            } = effect
            {
                result.gesture_acks.push((dispatch_id, handled));
            } else if let CompositorMainThreadEffect::WheelDisposition { compositor_owned } = effect
            {
                result.compositor_owned |= compositor_owned;
            } else {
                result.effects.push(effect);
            }
        }
        let (submitted, submitted_latest_commit, needs_begin_frames, main_commit_pending) =
            self.AcceptOutput(output)?;
        for effect in self.TakeMainThreadEffects() {
            if let CompositorMainThreadEffect::GestureDisposition {
                dispatch_id,
                handled,
            } = effect
            {
                result.gesture_acks.push((dispatch_id, handled));
            } else if let CompositorMainThreadEffect::WheelDisposition { compositor_owned } = effect
            {
                result.compositor_owned |= compositor_owned;
            } else {
                result.effects.push(effect);
            }
        }
        if let Some(pending) = self.pending.as_mut() {
            pending.needs_begin_frames = needs_begin_frames;
        }
        if !main_commit_pending
            && (self.external_presentation || submitted_latest_commit || stable_frame_reuse)
        {
            if let Some(pending) = self.pending.take() {
                self.presentable = Some(PresentablePageRender {
                    artifact: Arc::downgrade(&pending.artifact),
                    input: pending.input,
                });
            }
        }
        result.submitted = submitted;
        self.input_frame_issued_since_physical = submitted;
        if let (Some(started), Some(began)) = (profile, began_at) {
            eprintln!(
                "page-compositor-begin-profile begin_ms={:.3} output_ms={:.3} total_ms={:.3} submitted={} effects={}",
                (began - started).as_secs_f64() * 1000.0,
                began.elapsed().as_secs_f64() * 1000.0,
                started.elapsed().as_secs_f64() * 1000.0,
                result.submitted,
                result.effects.len(),
            );
        }
        Ok(result)
    }

    fn BeginInputFrame(&mut self) -> io::Result<CompositorWheelResult> {
        let mut output = self.BeginFrame()?;
        let stable_frame_reuse = self.pending.is_some()
            && output.frame.is_none()
            && !output.needs_begin_frames
            && !output.main_thread_effects.iter().any(|effect| {
                matches!(
                    effect,
                    CompositorMainThreadEffect::BeginMainFrame
                        | CompositorMainThreadEffect::RequestMainFrame
                )
            });
        let mut result = CompositorWheelResult::default();
        for effect in std::mem::take(&mut output.main_thread_effects) {
            if let CompositorMainThreadEffect::GestureDisposition {
                dispatch_id,
                handled,
            } = effect
            {
                result.gesture_acks.push((dispatch_id, handled));
            } else if let CompositorMainThreadEffect::WheelDisposition { compositor_owned } = effect
            {
                result.compositor_owned |= compositor_owned;
            } else {
                result.effects.push(effect);
            }
        }
        let (submitted, submitted_latest_commit, needs_begin_frames, main_commit_pending) =
            self.AcceptOutput(output)?;
        for effect in self.TakeMainThreadEffects() {
            if let CompositorMainThreadEffect::GestureDisposition {
                dispatch_id,
                handled,
            } = effect
            {
                result.gesture_acks.push((dispatch_id, handled));
            } else if let CompositorMainThreadEffect::WheelDisposition { compositor_owned } = effect
            {
                result.compositor_owned |= compositor_owned;
            } else {
                result.effects.push(effect);
            }
        }
        if let Some(pending) = self.pending.as_mut() {
            pending.needs_begin_frames = needs_begin_frames;
        }
        if !main_commit_pending
            && (self.external_presentation || submitted_latest_commit || stable_frame_reuse)
        {
            if let Some(pending) = self.pending.take() {
                self.presentable = Some(PresentablePageRender {
                    artifact: Arc::downgrade(&pending.artifact),
                    input: pending.input,
                });
            }
        }
        result.submitted = submitted;
        Ok(result)
    }

    pub fn NeedsBeginFrames(&self) -> bool {
        self.needs_begin_frames
    }

    pub fn NeedsPump(&self) -> bool {
        // Keep posting owner turns only when an actual Poll exhausted its
        // budget with runnable typed tasks left. Input's broader work bit can
        // also mean that a task-runner callback or physical BeginFrame will
        // wake us later, and must not create a busy Pump loop.
        self.input_frame_pump_pending || self.owner_turn_pending
    }

    /// Notify cc that the Page needs a main-frame lifecycle before that
    /// lifecycle has produced its immutable PaintArtifact. This is the
    /// external Page runtime's ProxyMain::SetNeedsCommit boundary.
    pub fn RequestMainFrame(&mut self) -> io::Result<CompositorWheelResult> {
        self.EnsureCompositor(None)?;
        let output = self
            .compositor
            .as_mut()
            .expect("initialized compositor")
            .RequestMainFrame()
            .map_err(|error| io::Error::other(error.to_string()))?;
        let stable_frame_reuse = self.pending.is_some()
            && output.frame.is_none()
            && !output.needs_begin_frames
            && !output.main_thread_effects.iter().any(|effect| {
                matches!(
                    effect,
                    CompositorMainThreadEffect::BeginMainFrame
                        | CompositorMainThreadEffect::RequestMainFrame
                )
            });
        let (submitted, submitted_latest_commit, needs_begin_frames, main_commit_pending) =
            self.AcceptOutput(output)?;
        if let Some(pending) = self.pending.as_mut() {
            pending.needs_begin_frames = needs_begin_frames;
        }
        if !main_commit_pending
            && (self.external_presentation || submitted_latest_commit || stable_frame_reuse)
        {
            if let Some(pending) = self.pending.take() {
                self.presentable = Some(PresentablePageRender {
                    artifact: Arc::downgrade(&pending.artifact),
                    input: pending.input,
                });
            }
        }
        Ok(CompositorWheelResult {
            submitted,
            effects: self.TakeMainThreadEffects(),
            ..Default::default()
        })
    }

    /// Completes a dispatched BeginMainFrame when Page's lifecycle produced no
    /// PaintArtifact. Chromium reports this as
    /// BeginMainFrameAborted(kFinishedNoUpdates); leaving the transaction in
    /// SENT state would prevent Scheduler from admitting the next readiness
    /// request.
    pub fn FinishMainFrameWithoutUpdates(&mut self) -> io::Result<CompositorWheelResult> {
        let output = self
            .compositor
            .as_mut()
            .ok_or_else(|| io::Error::other("no compositor owns the BeginMainFrame"))?
            .FinishMainFrameWithoutUpdates()
            .map_err(|error| match error {
                CompositorError::NotReady { stage } => {
                    io::Error::new(io::ErrorKind::WouldBlock, stage)
                }
                error => io::Error::other(error.to_string()),
            })?;
        let (submitted, _, needs_begin_frames, _) = self.AcceptOutput(output)?;
        if let Some(pending) = self.pending.as_mut() {
            pending.needs_begin_frames = needs_begin_frames;
        }
        Ok(CompositorWheelResult {
            submitted,
            effects: self.TakeMainThreadEffects(),
            ..Default::default()
        })
    }

    pub fn TakeFrames(&mut self) -> Vec<CompositorFrameSubmission> {
        self.outbound_frames.drain(..).collect()
    }

    /// Actual impl FinishCommit timestamps translated out of this pipeline's
    /// injected clock. External hosts route these before swap/presentation.
    pub fn TakeCommitFinishTimes(&mut self) -> Vec<std::time::Instant> {
        std::mem::take(&mut self.commit_finish_times)
    }

    pub fn TakeChildFrames(&mut self) -> Vec<PageChildCompositorFrame> {
        self.outbound_child_frames.drain(..).collect()
    }

    pub fn TakeMainThreadEffects(&mut self) -> Vec<CompositorMainThreadEffect> {
        std::mem::take(&mut self.deferred_effects)
    }

    /// Runs work posted to cc's injected task runner on the Page impl sequence.
    /// The executor only transports owned callbacks; tree and scheduler owners
    /// remain resident here, matching Chromium's task-runner wakeup boundary.
    pub fn Pump(&mut self) -> io::Result<CompositorWheelResult> {
        let Some(compositor) = self.compositor.as_mut() else {
            return Ok(CompositorWheelResult::default());
        };
        let input_frame_pump = std::mem::take(&mut self.input_frame_pump_pending);
        let mut output = compositor
            .Poll()
            .map_err(|error| io::Error::other(error.to_string()))?;
        let mut result = CompositorWheelResult::default();
        let stable_frame_reuse = self.pending.is_some()
            && output.frame.is_none()
            && !output.needs_begin_frames
            && !output.main_thread_effects.iter().any(|effect| {
                matches!(
                    effect,
                    CompositorMainThreadEffect::BeginMainFrame
                        | CompositorMainThreadEffect::RequestMainFrame
                )
            });
        for effect in std::mem::take(&mut output.main_thread_effects) {
            if let CompositorMainThreadEffect::GestureDisposition {
                dispatch_id,
                handled,
            } = effect
            {
                result.gesture_acks.push((dispatch_id, handled));
            } else if let CompositorMainThreadEffect::WheelDisposition { compositor_owned } = effect
            {
                result.compositor_owned |= compositor_owned;
            } else {
                result.effects.push(effect);
            }
        }
        let (submitted, submitted_latest_commit, needs_begin_frames, main_commit_pending) =
            self.AcceptOutput(output)?;
        for effect in self.TakeMainThreadEffects() {
            if let CompositorMainThreadEffect::GestureDisposition {
                dispatch_id,
                handled,
            } = effect
            {
                result.gesture_acks.push((dispatch_id, handled));
            } else if let CompositorMainThreadEffect::WheelDisposition { compositor_owned } = effect
            {
                result.compositor_owned |= compositor_owned;
            } else {
                result.effects.push(effect);
            }
        }
        if let Some(pending) = self.pending.as_mut() {
            pending.needs_begin_frames = needs_begin_frames;
        }
        if !main_commit_pending
            && (self.external_presentation || submitted_latest_commit || stable_frame_reuse)
        {
            if let Some(pending) = self.pending.take() {
                self.presentable = Some(PresentablePageRender {
                    artifact: Arc::downgrade(&pending.artifact),
                    input: pending.input,
                });
            }
        }
        result.submitted = submitted;
        self.input_frame_issued_since_physical |= submitted;
        self.input_frame_pump_pending = input_frame_pump && !submitted && self.work_pending;
        self.owner_turn_pending = self.work_pending;
        Ok(result)
    }

    pub fn ReclaimResources(
        &mut self,
        resources: Vec<compositor::runtime::resources::RasterReturnedResource>,
    ) -> io::Result<()> {
        if resources.is_empty() || self.compositor.is_none() {
            return Ok(());
        }
        let output = self
            .compositor
            .as_mut()
            .expect("checked compositor")
            .ReclaimResources(resources)
            .map_err(|error| io::Error::other(error.to_string()))?;
        self.AcceptOutput(output)?;
        Ok(())
    }

    /// Route a wheel gesture through cc's InputHandler on the resident active
    /// tree. The returned frame is already submitted to this Page's Viz surface;
    /// Page later reflects the emitted offset on its main-thread DOM owner.
    pub fn Wheel(
        &mut self,
        wheel: &::interaction::input_event::WheelEvent,
    ) -> io::Result<CompositorWheelResult> {
        let timestamp = self
            .executor
            .TimeTicks(wheel.timestamp.unwrap_or_else(std::time::Instant::now));
        self.WheelAtTime(wheel, timestamp, self.InputWidgetScale())
    }
    /// Source-generated wheel retains its original TimeTicks through both the
    /// compositor boundary and the document's canonical input packet.
    pub fn CanonicalFlingWheel(
        &mut self,
        event: &interaction::input::public::common::input::web_mouse_wheel_event::WebMouseWheelEvent,
    ) -> io::Result<(interaction::input_event::WheelEvent, CompositorWheelResult)> {
        let wheel = interaction::adapter::generated_fling_input::WheelHostSidecar(event);
        let timestamp = CompositorTimeTicks::FromMicroseconds(event.TimeStamp().ToInternalValue());
        // FlingController's canonical WebMouseWheelEvent already carries the
        // root-widget position produced by InputRouter's one-time scaling.
        let result = self.WheelAtTime(&wheel, timestamp, 1.0)?;
        Ok((wheel, result))
    }
    fn InputWidgetScale(&self) -> f32 {
        // Chromium's InputRouter scales host CSS/DIP coordinates before the
        // renderer sees a WebInputEvent. Rechrom keeps the host packet in CSS
        // coordinates so Interaction can retain that boundary, then performs
        // the identical one-time conversion at each renderer consumer. The
        // active compositor tree corresponds to the last presentable artifact;
        // a newer pending artifact must not change input coordinates early.
        let scale = self
            .presentable
            .as_ref()
            .map(|render| f64::from_bits(render.input.layout_zoom))
            .or_else(|| {
                self.pending
                    .as_ref()
                    .map(|render| f64::from_bits(render.input.layout_zoom))
            })
            .unwrap_or(1.0);
        if scale.is_finite() && scale > 0.0 && scale <= f32::MAX as f64 {
            scale as f32
        } else {
            1.0
        }
    }
    fn WheelAtTime(
        &mut self,
        wheel: &interaction::input_event::WheelEvent,
        event_timestamp: CompositorTimeTicks,
        position_scale: f32,
    ) -> io::Result<CompositorWheelResult> {
        use interaction::input_event::{ScrollGranularity, WheelPhase};
        let phase = match wheel.phase {
            WheelPhase::kBegan => ScrollPhase::Begin,
            WheelPhase::kEnded | WheelPhase::kCancelled => ScrollPhase::End,
            WheelPhase::kNone | WheelPhase::kChanged => ScrollPhase::Update,
        };
        let (delta, unconstrained) = wheel.GestureScrollDelta();
        let granularity = match wheel.delta_units {
            ScrollGranularity::kScrollByPrecisePixel => {
                compositor::adapter::input_types::ScrollGranularity::kScrollByPrecisePixel
            }
            ScrollGranularity::kScrollByPixel => {
                compositor::adapter::input_types::ScrollGranularity::kScrollByPixel
            }
            ScrollGranularity::kScrollByLine => {
                compositor::adapter::input_types::ScrollGranularity::kScrollByLine
            }
            ScrollGranularity::kScrollByPage => {
                compositor::adapter::input_types::ScrollGranularity::kScrollByPage
            }
            ScrollGranularity::kScrollByDocument => {
                compositor::adapter::input_types::ScrollGranularity::kScrollByDocument
            }
        };
        let input = CompositorInput::Wheel {
            position: CompositorPointF::new(
                wheel.position.x as f32 * position_scale,
                wheel.position.y as f32 * position_scale,
            ),
            delta: Vector2dF::new(delta.x as f32, delta.y as f32),
            unconstrained_delta: Vector2dF::new(unconstrained.x as f32, unconstrained.y as f32),
            granularity,
            is_inertial: wheel
                .native
                .is_some_and(|native| native.momentum_phase != 0),
            has_synthetic_phase: wheel.has_synthetic_phase,
            event_timestamp,
            phase,
        };
        self.WheelInput(
            compositor::CompositorMutation::Input(input),
            delta.x != 0. || delta.y != 0.,
        )
    }
    pub fn InputTimeForInstant(
        &self,
        instant: std::time::Instant,
    ) -> interaction::input::base::time::time::TimeTicks {
        interaction::input::base::time::time::TimeTicks::FromInternalValue(
            self.executor
                .TimeTicks(instant)
                .since_origin()
                .InMicroseconds(),
        )
    }
    /// Widget input gestures are already recognized; only values cross into cc.
    pub fn Gesture(
        &mut self,
        gesture: &interaction::input_event::WebGestureEvent,
    ) -> io::Result<CompositorWheelResult> {
        self.WheelInput(
            compositor::CompositorMutation::Input(Self::GestureInput(gesture)?),
            true,
        )
    }
    pub fn GestureWithDispatchId(
        &mut self,
        gesture: &interaction::input_event::WebGestureEvent,
        dispatch_id: u64,
    ) -> io::Result<CompositorWheelResult> {
        let mut input = Self::GestureInput(gesture)?;
        match &mut input {
            CompositorInput::ScrollGesture(gesture) => gesture.dispatch_id = dispatch_id,
            CompositorInput::PinchGesture {
                dispatch_id: id, ..
            } => *id = dispatch_id,
            _ => unreachable!(),
        }
        self.WheelInput(compositor::CompositorMutation::Input(input), true)
    }
    pub fn GestureInput(
        gesture: &interaction::input_event::WebGestureEvent,
    ) -> io::Result<CompositorInput> {
        use compositor::adapter::input_handler::ScrollInputType;
        use interaction::input::public::common::input::{
            web_gesture_event::{GestureDevice, InertialPhaseState},
            web_input_event::Type,
        };
        let input_type = match gesture.SourceDevice() {
            GestureDevice::kTouchpad => ScrollInputType::kWheel,
            GestureDevice::kTouchscreen => ScrollInputType::kTouchscreen,
            GestureDevice::kSyntheticAutoscroll => ScrollInputType::kAutoscroll,
            GestureDevice::kScrollbar => ScrollInputType::kScrollbar,
            GestureDevice::kUninitialized => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "gesture source device is uninitialized",
                ));
            }
        };
        let phase = match gesture.GetType() {
            Type::kGestureScrollBegin | Type::kGesturePinchBegin => ScrollPhase::Begin,
            Type::kGestureScrollUpdate | Type::kGesturePinchUpdate => ScrollPhase::Update,
            Type::kGestureScrollEnd | Type::kGesturePinchEnd => ScrollPhase::End,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "gesture requires its source main-thread or browser fling owner",
                ));
            }
        };
        let anchor = gesture.PositionInWidget();
        let position = CompositorPointF::new(anchor.x(), anchor.y());
        if matches!(
            gesture.GetType(),
            Type::kGesturePinchBegin | Type::kGesturePinchUpdate | Type::kGesturePinchEnd
        ) {
            return Ok(CompositorInput::PinchGesture {
                dispatch_id: 0,
                anchor: position,
                scale: if phase == ScrollPhase::Update {
                    gesture.data.pinch_update.scale
                } else {
                    1.0
                },
                input_type,
                phase,
            });
        }
        let delta = match phase {
            ScrollPhase::Begin => Vector2dF::new(
                gesture.data.scroll_begin.delta_x_hint,
                gesture.data.scroll_begin.delta_y_hint,
            ),
            ScrollPhase::Update => Vector2dF::new(
                gesture.data.scroll_update.delta_x,
                gesture.data.scroll_update.delta_y,
            ),
            ScrollPhase::End => Vector2dF::new(
                gesture.data.scroll_end.delta_x_compensated,
                gesture.data.scroll_end.delta_y_compensated,
            ),
        };
        let granularity = match gesture.DeltaUnits() {
            interaction::input::ui::events::types::scroll_types::ScrollGranularity::kScrollByPrecisePixel => {
                compositor::adapter::input_types::ScrollGranularity::kScrollByPrecisePixel
            }
            interaction::input::ui::events::types::scroll_types::ScrollGranularity::kScrollByPixel => {
                compositor::adapter::input_types::ScrollGranularity::kScrollByPixel
            }
            interaction::input::ui::events::types::scroll_types::ScrollGranularity::kScrollByLine => {
                compositor::adapter::input_types::ScrollGranularity::kScrollByLine
            }
            interaction::input::ui::events::types::scroll_types::ScrollGranularity::kScrollByPage => {
                compositor::adapter::input_types::ScrollGranularity::kScrollByPage
            }
            interaction::input::ui::events::types::scroll_types::ScrollGranularity::kScrollByDocument => {
                compositor::adapter::input_types::ScrollGranularity::kScrollByDocument
            }
        };
        let mut reasons = compositor::adapter::input_types::MainThreadHitTestReasons::default();
        if phase == ScrollPhase::Begin {
            use compositor::adapter::input_types::MainThreadHitTestReason;
            for reason in [
                MainThreadHitTestReason::kScrollbarScrolling,
                MainThreadHitTestReason::kMainThreadScrollHitTestRegion,
                MainThreadHitTestReason::kFailedHitTest,
            ] {
                if gesture.data.scroll_begin.main_thread_hit_tested_reasons & (1 << reason as u32)
                    != 0
                {
                    reasons.Put(reason);
                }
            }
        }
        Ok(CompositorInput::ScrollGesture(
            compositor::engine::CompositorScrollGesture {
                dispatch_id: 0,
                position,
                delta,
                granularity,
                input_type,
                unconstrained_delta: if phase == ScrollPhase::Update {
                    Vector2dF::new(
                        gesture.data.scroll_update.delta_x_unconstrained,
                        gesture.data.scroll_update.delta_y_unconstrained,
                    )
                } else {
                    delta
                },
                is_inertial: gesture.InertialPhase() == InertialPhaseState::kMomentum,
                synthetic: phase != ScrollPhase::Update && gesture.Synthetic(),
                target_viewport: phase == ScrollPhase::Begin
                    && gesture.data.scroll_begin.target_viewport,
                scroll_element_id: compositor::adapter::input_types::ElementId::new(
                    if phase == ScrollPhase::Begin {
                        gesture.data.scroll_begin.scrollable_area_element_id
                    } else {
                        0
                    },
                ),
                main_thread_hit_tested_reasons: reasons,
                event_timestamp: CompositorTimeTicks::FromMicroseconds(
                    gesture.TimeStamp().ToInternalValue(),
                ),
                phase,
            },
        ))
    }
    pub fn ContinueWheelInput(
        &mut self,
        input: CompositorInput,
    ) -> io::Result<CompositorWheelResult> {
        self.WheelInput(compositor::CompositorMutation::Input(input), true)
    }
    pub(super) fn ScrollAnimationTime(
        &self,
        sample: std::time::Instant,
    ) -> scroll_animation::TimeTicks {
        self.executor.TimeTicks(sample)
    }
    pub fn SetScrollAnimationCommitRequests(
        &mut self,
        requests: Vec<compositor::adapter::input_types::ElementId>,
    ) {
        self.scroll_animation_commit_requests = requests;
    }
    pub fn ContinueCompositorMutation(
        &mut self,
        mutation: compositor::CompositorMutation,
    ) -> io::Result<CompositorWheelResult> {
        if let compositor::CompositorMutation::Input(
            CompositorInput::SetPageScaleFactorAndLimits {
                scale,
                minimum,
                maximum,
            },
        ) = &mutation
        {
            self.page_scale = (*scale, *minimum, *maximum);
        }
        self.EnsureCompositor(None)?;
        self.WheelInput(mutation, false)
    }
    fn WheelInput(
        &mut self,
        input: compositor::CompositorMutation,
        allow_input_frame: bool,
    ) -> io::Result<CompositorWheelResult> {
        let profile = std::env::var_os("BROWSER_PROFILE_INPUT").map(|_| std::time::Instant::now());
        let first = self
            .compositor
            .as_mut()
            .ok_or_else(|| {
                io::Error::other("wheel input requires a committed resident compositor")
            })?
            .Mutate(input)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let input_done = profile.map(|start| start.elapsed());
        let mut result = CompositorWheelResult::default();
        let mut output = first;
        let stable_frame_reuse = self.pending.is_some()
            && output.frame.is_none()
            && !output.needs_begin_frames
            && !output.main_thread_effects.iter().any(|effect| {
                matches!(
                    effect,
                    CompositorMainThreadEffect::BeginMainFrame
                        | CompositorMainThreadEffect::RequestMainFrame
                )
            });
        for effect in std::mem::take(&mut output.main_thread_effects) {
            if let CompositorMainThreadEffect::GestureDisposition {
                dispatch_id,
                handled,
            } = effect
            {
                result.gesture_acks.push((dispatch_id, handled));
            } else if let CompositorMainThreadEffect::WheelDisposition { compositor_owned } = effect
            {
                result.compositor_owned |= compositor_owned;
            } else if let CompositorMainThreadEffect::WheelRequiresDOMAcknowledgement { request } =
                effect
            {
                assert!(
                    result.dom_ack.replace(request).is_none(),
                    "one original wheel acknowledgement"
                );
            } else {
                result.effects.push(effect);
            }
        }
        let (submitted, submitted_latest_commit, needs_begin_frames, main_commit_pending) =
            self.AcceptOutput(output)?;
        for effect in self.TakeMainThreadEffects() {
            if let CompositorMainThreadEffect::GestureDisposition {
                dispatch_id,
                handled,
            } = effect
            {
                result.gesture_acks.push((dispatch_id, handled));
            } else if let CompositorMainThreadEffect::WheelDisposition { compositor_owned } = effect
            {
                result.compositor_owned |= compositor_owned;
            } else if let CompositorMainThreadEffect::WheelRequiresDOMAcknowledgement { request } =
                effect
            {
                assert!(
                    result.dom_ack.replace(request).is_none(),
                    "one original wheel acknowledgement"
                );
            } else {
                result.effects.push(effect);
            }
        }
        if let Some(pending) = self.pending.as_mut() {
            pending.needs_begin_frames = needs_begin_frames;
        }
        if !main_commit_pending
            && (self.external_presentation || submitted_latest_commit || stable_frame_reuse)
        {
            if let Some(pending) = self.pending.take() {
                self.presentable = Some(PresentablePageRender {
                    artifact: Arc::downgrade(&pending.artifact),
                    input: pending.input,
                });
            }
        }
        result.submitted = submitted;
        // A compositor scroll requests redraw on the existing scheduler.
        // External presentation receives its next opportunity from the host,
        // not a second manual BeginFrame generated by the input callback.
        let input_frame = !self.external_presentation
            && result.compositor_owned
            && allow_input_frame
            && !result.submitted
            && !self.input_frame_issued_since_physical;
        if input_frame {
            // The synchronous internal presenter has no external frame source;
            // its caller explicitly drives a frame after the input mutation.
            self.input_frame_issued_since_physical = true;
            let immediate = self.BeginInputFrame()?;
            self.input_frame_pump_pending = !immediate.submitted && self.work_pending;
            result.submitted |= immediate.submitted;
            result.compositor_owned |= immediate.compositor_owned;
            result.effects.extend(immediate.effects);
        }
        if let Some(start) = profile {
            eprintln!(
                "page-compositor-wheel-profile input_ms={:.3} output_ms={:.3} total_ms={:.3} compositor_owned={} submitted={} input_frame={} effects={}",
                input_done.unwrap().as_secs_f64() * 1000.0,
                (start.elapsed() - input_done.unwrap()).as_secs_f64() * 1000.0,
                start.elapsed().as_secs_f64() * 1000.0,
                result.compositor_owned,
                result.submitted,
                input_frame,
                result.effects.len(),
            );
        }
        Ok(result)
    }

    pub fn Render(
        &mut self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
        target: &mut [u32],
        format: PixelFormat,
        stride: usize,
    ) -> io::Result<RenderUpdate> {
        self.RenderToTarget(
            artifact, width, height, scale, target, format, stride, false,
        )
    }

    pub fn RenderRetained(
        &mut self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
        target: &mut [u32],
        format: PixelFormat,
        stride: usize,
    ) -> io::Result<RenderUpdate> {
        self.RenderToTarget(artifact, width, height, scale, target, format, stride, true)
    }

    /// Commit PaintEngine output and produce zero or more move-only frames.
    /// External Page runtimes call `TakeFrames` and hand those values to their
    /// Page-owned Viz/Renderer sequence.
    pub fn CommitArtifact(
        &mut self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
    ) -> io::Result<()> {
        self.CommitArtifactInternal(artifact, width, height, scale, false)
    }

    /// Completes the BeginMainFrame which admitted this Page lifecycle. If
    /// PaintEngine retained the exact presentable artifact, cc receives
    /// Chromium's FINISHED_NO_UPDATES response instead of a duplicate commit.
    pub fn CommitArtifactForMainFrame(
        &mut self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
    ) -> io::Result<()> {
        self.CommitArtifactInternal(artifact, width, height, scale, true)
    }

    /// Produces a new frame from cc's resident active tree. Embedders use this
    /// when frame-scoped metadata changes independently of PaintArtifact
    /// pixels; resource ownership and draw scheduling remain inside cc.
    pub fn RequestRedraw(&mut self) -> io::Result<()> {
        let output = self
            .compositor
            .as_mut()
            .ok_or_else(|| io::Error::other("redraw requires a presentable compositor tree"))?
            .RequestRedraw()
            .map_err(|error| io::Error::other(error.to_string()))?;
        self.AcceptOutput(output)?;
        Ok(())
    }

    fn CommitArtifactInternal(
        &mut self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
        responds_to_main_frame: bool,
    ) -> io::Result<()> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "raster scale must be positive",
            ));
        }
        if width > i32::MAX as u32 || height > i32::MAX as u32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "render target exceeds compositor geometry range",
            ));
        }
        let layout_zoom = artifact
            .resources
            .as_ref()
            .map_or(1.0, |resources| resources.layout_zoom_factor)
            .max(f64::MIN_POSITIVE);
        let raster_scale = scale / layout_zoom;
        if !raster_scale.is_finite() || raster_scale <= 0.0 || raster_scale > f32::MAX as f64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "artifact-to-device scale is outside compositor range",
            ));
        }
        let input = PreparedRenderInput {
            width,
            height,
            scale: scale.to_bits(),
            layout_zoom: layout_zoom.to_bits(),
        };
        let viewport = CompositorViewportState {
            size: CompositorSize::new(width as i32, height as i32),
            device_scale_factor: raster_scale as f32,
            page_scale_factor: self.page_scale.0,
            min_page_scale_factor: self.page_scale.1,
            max_page_scale_factor: self.page_scale.2,
            visible: true,
        };
        if let Some(pending) = self.pending.as_ref() {
            let requested_pending = pending.Matches(artifact, input, viewport);
            self.EnsureCompositor(Some(viewport))?;
            let submitted = self.ContinuePending()?;
            if !requested_pending {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    if submitted {
                        "previous Page compositor commit became presentable; retry the newer PaintArtifact"
                    } else {
                        "previous Page compositor commit is still pending"
                    },
                ));
            }
            // Commit() already consumed the BeginMainFrame before raster and
            // frame submission complete.  A retry for that same accepted
            // artifact must therefore complete the Page transaction even
            // while its pixels are still pending; presentation progresses on
            // subsequent impl frames.  Chromium likewise acknowledges the
            // main-thread commit at ProxyImpl::ScheduledActionCommit, not when
            // DrawLayers later submits a CompositorFrame.
            if responds_to_main_frame && self.scroll_animation_commit_requests.is_empty() {
                return Ok(());
            }
            if self.pending.is_some()
                || (!submitted && self.scroll_animation_commit_requests.is_empty())
            {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "compositor is still preparing the committed PaintArtifact",
                ));
            }
        }
        let already_presentable = self.scroll_animation_commit_requests.is_empty()
            && self.presentable.as_ref().is_some_and(|presentable| {
                presentable.input == input
                    && presentable
                        .artifact
                        .upgrade()
                        .is_some_and(|prepared| Arc::ptr_eq(&prepared, artifact))
            });
        if already_presentable && responds_to_main_frame {
            let output = self
                .compositor
                .as_mut()
                .expect("a presentable artifact has an initialized compositor")
                .FinishMainFrameWithoutUpdates()
                .map_err(|error| match error {
                    CompositorError::NotReady { stage } => {
                        io::Error::new(io::ErrorKind::WouldBlock, stage)
                    }
                    error => io::Error::other(error.to_string()),
                })?;
            self.AcceptOutput(output)?;
        }
        if !already_presentable {
            self.EnsureCompositor(Some(viewport))?;
            let first = match self
                .compositor
                .as_mut()
                .expect("initialized compositor")
                .CommitWithScrollAnimations(
                    artifact.clone(),
                    viewport,
                    std::mem::take(&mut self.scroll_animation_commit_requests),
                ) {
                Ok(output) => output,
                Err(CompositorError::NotReady { stage }) => {
                    return Err(io::Error::new(io::ErrorKind::WouldBlock, stage));
                }
                Err(error) => return Err(io::Error::other(error.to_string())),
            };
            self.pending = Some(PendingPageRender {
                artifact: artifact.clone(),
                input,
                viewport,
                needs_begin_frames: false,
            });
            let presentable = self.AcceptPendingOutput(first)?;
            // AcceptPendingOutput clears `pending` only after cc reports that
            // the main commit itself completed.  Frame submission may remain
            // asynchronous, but a later PaintArtifact must not replace the
            // ready commit or the pending tree before that boundary.  Doing
            // so pairs a newer reflected scroll delta with an older absolute
            // main-thread base and makes activation move the viewport back.
            if self.external_presentation && self.pending.is_none() {
                return Ok(());
            }
            let presentable = presentable || self.ContinuePending()?;
            if !presentable && !responds_to_main_frame {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "compositor is still preparing the committed PaintArtifact",
                ));
            }
        }
        Ok(())
    }

    fn RenderToTarget(
        &mut self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
        target: &mut [u32],
        format: PixelFormat,
        stride: usize,
        retained: bool,
    ) -> io::Result<RenderUpdate> {
        let profile_started = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        self.CommitArtifact(artifact, width, height, scale)?;
        let committed_at = profile_started.map(|_| std::time::Instant::now());

        let surface = self.surface;
        let output = viz::adapter::DeviceRect::new(0, 0, width as i32, height as i32);
        let placement = viz::adapter::SurfacePlacement {
            surface_id: surface,
            destination: output,
        };
        let (update, viz_output) = {
            let mut viz_output = self
                .viz
                .as_mut()
                .expect("internal presentation owns Viz")
                .Aggregate(output, &[placement])
                .map_err(|error| io::Error::other(error.to_string()))?;
            let aggregated_at = profile_started.map(|_| std::time::Instant::now());
            let aggregate_damage_pixels = viz_output.DamagePixels();
            let owned_target = target.to_vec();
            let (update, rendered_target) = self
                .renderer
                .as_mut()
                .expect("internal presentation owns Renderer")
                .Render(
                    &mut viz_output,
                    renderer_skia::RenderTarget {
                        width,
                        height,
                        pixels: renderer_skia::PixelStorage::owned(owned_target),
                        format,
                        row_stride: stride,
                        retained: retained && self.rendered_surface == Some(surface),
                    },
                )?;
            target.copy_from_slice(&rendered_target.pixels);
            if let (Some(started), Some(committed), Some(aggregated)) =
                (profile_started, committed_at, aggregated_at)
            {
                eprintln!(
                    "page-render-profile commit_ms={:.3} aggregate_ms={:.3} raster_ms={:.3} total_ms={:.3} passes={} quads={} frame_damage_pixels={} replay_damage_pixels={}",
                    (committed - started).as_secs_f64() * 1000.0,
                    (aggregated - committed).as_secs_f64() * 1000.0,
                    aggregated.elapsed().as_secs_f64() * 1000.0,
                    started.elapsed().as_secs_f64() * 1000.0,
                    update.passes,
                    update.quads,
                    aggregate_damage_pixels,
                    update.damage_pixels,
                );
            }
            (update, viz_output)
        };
        self.viz
            .as_mut()
            .expect("internal presentation owns Viz")
            .DidDraw(viz_output);
        self.rendered_surface = Some(surface);
        Ok(update)
    }

    fn InvalidatePreparedArtifact(&mut self) {
        self.presentable = None;
    }
}
impl Default for PageRenderPipeline {
    fn default() -> Self {
        Self::new(executor::inline())
    }
}

#[cfg(test)]
#[test]
fn compositor_work_pending_posts_an_owner_turn_until_pump_consumes_it() {
    let mut pipeline = PageRenderPipeline::default();
    pipeline
        .AcceptOutput(CompositorOutput {
            frame: None,
            frame_matches_latest_commit: false,
            main_thread_effects: Vec::new(),
            needs_begin_frames: false,
            work_pending: true,
            main_commit_pending: false,
        })
        .unwrap();
    assert!(pipeline.NeedsPump());

    // A later stable output cannot erase the already queued continuation.
    pipeline
        .AcceptOutput(CompositorOutput {
            frame: None,
            frame_matches_latest_commit: false,
            main_thread_effects: Vec::new(),
            needs_begin_frames: false,
            work_pending: false,
            main_commit_pending: false,
        })
        .unwrap();
    assert!(pipeline.NeedsPump());
}

// cpp: browser/browser.h:47-51
pub struct PageFrame {
    pub sequence: u64,
    /// Last compositor scroll delta reflected by this committed snapshot.
    pub compositor_scroll_sequence: u64,
    pub fragments: LayoutResultHandle,
    pub display_items: Arc<paint::paint_engine::PaintArtifact>,
    /// Rechrom-specific certification for its immutable document-image record
    /// catalog. Blink's SVGImage instead sizes its isolated document and asks
    /// the view for a PaintRecord at draw time.
    ///
    /// Certified once when this immutable artifact enters the frame. Retained
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
    fn DidPresentFrame(&mut self, _frame: &PageFrame) {}
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
#[path = "page/layout_output_tests.rs"]
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

impl ResourceReferenceDirty {
    const STYLE_SHEETS: u8 = 1 << 0;
    const IMAGES: u8 = 1 << 1;
    const FONTS: u8 = 1 << 2;
    const ALL: Self = Self(Self::STYLE_SHEETS | Self::IMAGES | Self::FONTS);
    const VIEWPORT: Self = Self(Self::IMAGES | Self::FONTS);

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
    rendering: Rc<RefCell<PageRenderPipeline>>,
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
    layout_engine: RefCell<layoutng_assembly::layout_engine::LayoutEngine>,
    paint_engine: RefCell<paint::PaintEngine>,
    /// Embedder/default faces plus decoded @font-face resources. System faces
    /// stay in FontEngine's versioned query cache and are never promoted into
    /// this document-owned catalog.
    document_fonts: RefCell<font_engine::FontFaceCatalog>,
    document_font_inputs: PageDocumentFontInputs,
    system_fonts: RefCell<crate::font_session::SystemFontSession>,
    interaction: Rc<RefCell<UserInteractionState>>,
    layout_editing: Rc<layoutng_assembly::editing_state::LayoutEditingState>,
    resident_frame_selection:
        RefCell<Weak<interaction::adapter::page_frame_selection::PageFrameSelection>>,
    interaction_text_settings:
        RefCell<Option<interaction::adapter::page_pointer_hit_test::InteractionTextSettings>>,
    scrollbar_scroll_runtime: RefCell<Weak<PageInteractionScrollRuntime>>,
    blocking_wheel_registry_dirty: Cell<bool>,
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
            layoutng_style::style::scroll_enums::mojom::blink::ScrollType,
        )>,
    >,
    // Ownership travels with the queued offsets and is cleared only when the
    // queue is consumed. An empty event flush therefore cannot turn a later
    // compositor batch into a main-thread Paint update.
    compositor_scroll_updates: Cell<bool>,
    animation_engine: RefCell<animation::AnimationEngine>,
    animation_begin_frame_requested: Cell<bool>,
    pending_compositor_mutations: RefCell<Vec<PageBrowserInput>>,
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
            scroll_type: layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kUser,
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
        let now = state.rendering.borrow().executor.TimeTicks(frame_time);
        if self.animators.TickAnimations(now) {
            state.animation_begin_frame_requested.set(true);
        }
    }
}

impl interaction::adapter::page_scroll_manager::PageScrollRuntime for PageInteractionScrollRuntime {
    fn UserAnimationState(
        &self,
        area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
    ) -> layoutng_assembly::native_user_scroll_source::NativeUserScrollAnimationState {
        let enabled = self
            .owner
            .Page()
            .is_some_and(|state| state.has_begin_frame_source.get());
        let snapshot = self
            .animators
            .GetOrCreate(area.Identity(), enabled)
            .Snapshot();
        layoutng_assembly::native_user_scroll_source::NativeUserScrollAnimationState {
            enabled: snapshot.enabled,
            has_running_animation: snapshot.has_running_animation,
            desired_target_offset: snapshot.desired_target_offset,
        }
    }

    fn SnapAnimationState(
        &self,
        area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
    ) -> layoutng_assembly::native_scroll_source::NativeSnapAnimationState {
        let user = self.UserAnimationState(area);
        let scroll_element_id = area.GetScrollElementId();
        let programmatic_target = self
            .owner
            .WithProgrammaticAnimations(|animations| animations.TargetForElement(scroll_element_id))
            .flatten();
        layoutng_assembly::native_scroll_source::NativeSnapAnimationState {
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
        area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
        plan: layoutng_assembly::native_scroll_source::NativeLayoutSnapPlan,
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
                == layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior::kSmooth
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
        area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
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
        area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
        request: layoutng_assembly::native_user_scroll_source::NativeUserScrollAnimationRequest,
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
            layoutng_assembly::native_user_scroll_source::NativeUserScrollGranularity::PrecisePixel => ScrollGranularity::kScrollByPrecisePixel,
            layoutng_assembly::native_user_scroll_source::NativeUserScrollGranularity::Pixel => ScrollGranularity::kScrollByPixel,
            layoutng_assembly::native_user_scroll_source::NativeUserScrollGranularity::Line => ScrollGranularity::kScrollByLine,
            layoutng_assembly::native_user_scroll_source::NativeUserScrollGranularity::Page => ScrollGranularity::kScrollByPage,
            layoutng_assembly::native_user_scroll_source::NativeUserScrollGranularity::Document => ScrollGranularity::kScrollByDocument,
        };
        let Some(state) = self.owner.Page() else {
            return Err(completion);
        };
        let now = state
            .rendering
            .borrow()
            .executor
            .TimeTicks(std::time::Instant::now());
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
        _area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
        plan: layoutng_assembly::native_user_scroll_source::NativeScrollFinishedPlan,
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
        layout: &layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout,
        query: impl FnOnce(
            &layoutng_assembly::native_autoscroll_source::NativeAutoscrollSource<'_>,
        ) -> T,
    ) -> Option<T> {
        self.0.WithLayout(|engine| {
            let source = engine.NativeAutoscrollSource(layout, Default::default())?;
            Some(query(&source))
        })?
    }

    fn WithUserScroll<T>(
        &self,
        area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
        query: impl FnOnce(
            &mut layoutng_assembly::native_user_scroll_source::NativeUserScrollSource<'_>,
        ) -> T,
    ) -> Option<T> {
        self.0.WithLayout(|engine| {
            let mut source = engine.NativeUserScrollSource(area)?;
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
                    debug_assert!(layout.EventRegistryLayout(Some(node_id)).is_some());
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
    ) -> Option<layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout> {
        self.0
            .WithLayout(|layout| layout.LogicalScrollLayoutFor(node_id))?
    }

    fn LayoutView(
        &self,
    ) -> Option<layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout> {
        self.0
            .WithLayout(|layout| layout.LogicalScrollLayoutView())?
    }

    fn NodeFacts(
        &self,
        layout: &layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout,
    ) -> Option<layoutng_assembly::native_autoscroll_source::NativeLogicalScrollNodeFacts> {
        self.WithAutoscroll(layout, |source| source.LogicalScrollNodeFacts())?
    }

    fn EnclosingBox(
        &self,
        layout: &layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout,
    ) -> Option<layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout> {
        self.WithAutoscroll(layout, |source| source.EnclosingBox())?
    }

    fn LayoutBox(
        &self,
        layout: &layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout,
    ) -> Option<layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout> {
        self.WithAutoscroll(layout, |source| source.LogicalLayoutBox())?
    }

    fn LayoutBoxForScrolling(
        &self,
        layout: &layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout,
    ) -> Option<layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout> {
        self.WithAutoscroll(layout, |source| source.LayoutBoxForScrolling())?
    }

    fn BoxFacts(
        &self,
        layout: &layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout,
    ) -> layoutng_assembly::native_autoscroll_source::NativeLogicalScrollBoxFacts {
        self.WithAutoscroll(layout, |source| source.LogicalScrollBoxFacts())
            .expect("retained logical-scroll layout remains attached")
    }

    fn BoxNode(
        &self,
        layout: &layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout,
    ) -> Option<layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout> {
        self.WithAutoscroll(layout, |source| source.LogicalNode())?
    }

    fn ContainingBlock(
        &self,
        layout: &layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout,
    ) -> Option<layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout> {
        self.WithAutoscroll(layout, |source| source.ContainingBlock())?
    }

    fn ScrollableArea(
        &self,
        layout: &layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout,
    ) -> Option<layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea> {
        self.WithAutoscroll(layout, |source| source.LogicalScrollableArea())?
    }

    fn AreaForScrolling(
        &self,
        layout: &layoutng_assembly::native_autoscroll_source::NativeAutoscrollLayout,
    ) -> Option<layoutng_assembly::native_autoscroll_source::NativeLogicalScrollingArea> {
        self.WithAutoscroll(layout, |source| source.LogicalAreaForScrolling())?
    }

    fn LayoutViewportArea(
        &self,
    ) -> Option<layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea> {
        self.0
            .WithLayout(|layout| layout.LayoutViewportLogicalScrollArea())?
    }

    fn AreaFacts(
        &self,
        area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
    ) -> Option<layoutng_assembly::native_user_scroll_source::NativeUserScrollAreaFacts> {
        self.WithUserScroll(area, |source| source.Facts())
    }

    fn AreaOffset(
        &self,
        area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
    ) -> Option<foundation::gfx::Vector2dF> {
        self.WithUserScroll(area, |source| source.GetScrollOffset())
    }

    fn PlanSnapForUserScroll(
        &self,
        area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
        kind: layoutng_assembly::internal::layout_scrollable_area::NativeUserSnapScrollKind,
        direction: foundation::PhysicalDirection,
        line_step: f32,
        animation: layoutng_assembly::native_scroll_source::NativeSnapAnimationState,
    ) -> Option<(
        bool,
        layoutng_assembly::native_scroll_source::NativeLayoutSnapPlan,
    )> {
        self.WithUserScroll(area, |source| {
            source.PlanSnapForUserScroll(kind, direction, line_step, animation)
        })
    }

    fn UserScroll(
        &self,
        area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
        granularity: layoutng_assembly::native_user_scroll_source::NativeUserScrollGranularity,
        delta: foundation::gfx::Vector2dF,
        source_type: interaction::scroll::core::input::scroll_manager::ScrollSourceType,
        animation: layoutng_assembly::native_user_scroll_source::NativeUserScrollAnimationState,
    ) -> Option<layoutng_assembly::native_user_scroll_source::NativeUserScrollPlan> {
        self.WithUserScroll(area, |source| {
            source.UserScroll(granularity, delta, source_type, animation)
        })
    }

    fn WillConsumeUserScroll(
        &self,
        area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
        granularity: layoutng_assembly::native_user_scroll_source::NativeUserScrollGranularity,
        delta: foundation::gfx::Vector2dF,
        animation: layoutng_assembly::native_user_scroll_source::NativeUserScrollAnimationState,
    ) -> Option<bool> {
        self.WithUserScroll(area, |source| {
            source.WillConsumeUserScroll(granularity, delta, animation)
        })
    }

    fn OnScrollFinished(
        &self,
        area: &layoutng_assembly::native_autoscroll_source::NativeLogicalScrollArea,
        enqueue_scrollend: bool,
    ) -> Option<layoutng_assembly::native_user_scroll_source::NativeScrollFinishedPlan> {
        self.WithUserScroll(area, |source| source.OnScrollFinished(enqueue_scrollend))?
    }

    fn WheelScrollableAreas(
        &self,
        target_node_id: u64,
    ) -> Box<
        dyn Iterator<
            Item = layoutng_assembly::native_wheel_scrollable_traversal::NativeWheelScrollableArea,
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
    ) -> Option<layoutng_assembly::native_mouse_coordinates::NativeMouseCoordinateLayout> {
        let state = self.0.upgrade()?;
        state.EnsureLayoutOutput();
        let result = state.layout_engine.borrow().MouseCoordinateLayout(node_id);
        result
    }
    fn DragBoundingBox(&self, node_id: u64) -> Option<foundation::gfx::Rect> {
        let state = self.0.upgrade()?;
        state.EnsureLayoutOutput();
        let layout = state.layout_engine.borrow();
        Some(
            layout
                .MouseCoordinateLayout(node_id)?
                .AbsoluteBoundingBoxIncludingDescendants(&layout),
        )
    }
    fn SimulatedClickRootFrameCenter(
        &self,
        node: u64,
    ) -> Option<(foundation::PhysicalOffset, f32)> {
        let state = self.0.upgrade()?;
        let layout = state.layout_engine.borrow();
        layout
            .MouseCoordinateLayout(node)?
            .SimulatedClickRootFrameCenter(&layout)
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
            .MouseLayoutViewportScrollOffsetInt();
        offset
    }

    fn HasLayoutObject(&self, node_id: u64) -> bool {
        self.0.upgrade().is_some_and(|state| {
            state
                .layout_engine
                .borrow()
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
    ) -> Option<layoutng_assembly::layout_engine::LayoutFocusNavigationFacts> {
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
            layout.UpdateEditingNodeFactsForFocus(node_id, facts);
        }
        for node_id in update.snap_container_candidates {
            if let Some(mut source) = layout.NativeScrollSource(node_id) {
                source.UpdateFocusDataForSnapAreas();
            }
        }
    }

    fn CurrentSelection(
        state: &PageState,
    ) -> Option<layoutng_assembly::editing_state::DOMSelection> {
        let owner = state.document.borrow();
        interaction::adapter::page_frame_selection::CurrentDOMSelectionForFocus(
            owner.GetDocument(),
            &state.layout_editing.selections,
        )
    }

    fn CaretBounds(
        state: &PageState,
        selection: Option<layoutng_assembly::editing_state::DOMSelection>,
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
            self.rendering.borrow().executor.clone(),
        ))
    }

    pub(super) fn PublishPreparedDocumentInputs(&self) {
        if self.document_font_inputs.pending.replace(false) {
            let mut constraints = self.constraints.borrow_mut();
            constraints.images.clear();
            constraints.canvases.clear();
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
        data: &layoutng_style::style::css_timing_data::CSSAnimationData,
        index: usize,
    ) -> animation::CSSAnimationTiming {
        use animation::{CSSFillMode, CSSPlayState, CSSPlaybackDirection, CSSTimingFunction};
        use layoutng_style::style::{
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
            play_state: if *layoutng_style::style::css_timing_data::CSSTimingData::GetRepeated(
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
                let mut rules = HashMap::<String, cssom::CSSKeyframesRule>::new();
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
                            rules.insert(rule.name.clone(), rule.clone());
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
                        let Some(rule) = rules.get(&name) else {
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
        layout_engine: layoutng_assembly::layout_engine::LayoutEngine,
        paint_engine: paint::PaintEngine,
        rendering: PageRenderPipeline,
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
            Some(rendering),
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
            layoutng_assembly::layout_engine::LayoutEngine::new(&crate::CreateLayoutAssembly()),
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
        layout_engine: layoutng_assembly::layout_engine::LayoutEngine,
        paint_engine: paint::PaintEngine,
        rendering: Option<PageRenderPipeline>,
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
                    rendering: Rc::new(RefCell::new(
                        rendering.expect("initial Page rendering service"),
                    )),
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
            paint_resources_dirty: Cell::new(false),
            video_frame_resources: RefCell::new(HashMap::new()),
            pending_paint_mutations: RefCell::new(Vec::new()),
            layout_revision: Cell::new(None),
            #[cfg(test)]
            full_layout_lifecycles: Cell::new(0),
            scroll_updates: RefCell::new(Vec::new()),
            compositor_scroll_updates: Cell::new(false),
            animation_engine: RefCell::new(animation::AnimationEngine::new()),
            animation_begin_frame_requested: Cell::new(false),
            pending_compositor_mutations: RefCell::new(Vec::new()),
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
            layoutng_assembly::layout_engine::LayoutEngine::new(&crate::CreateLayoutAssembly()),
            paint::PaintEngine::new(),
            PageRenderPipeline::default(),
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
        result: layoutng_assembly::native_scroll_source::NativeScrollResult,
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
                == layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior::kSmooth
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
                    && layout.InvalidateTextOverflowLayoutForNode(pending.node_id))
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

    fn PublishPaintResources(&self, frame: &mut PageFrame) {
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
        let mut paint = self.paint_engine.borrow_mut();
        frame.display_items = paint
            .PublishResourceUpdate(std::mem::take(&mut frame.display_items), update)
            .unwrap_or_else(|| dom::error::logic_error("paint resource update rejected"));
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
        let result = self.document.ApplyDOMMutation(m);
        self.connected_resources.ProcessRenderBlockingMutations();
        // Canonical CharacterData/Element removal supplements run even when
        // a throwing compound mutation already changed part of the DOM.
        let effect = result?;
        dom::attribute_lifecycle::DrainAttributeLifecycle(&self.document.Handle());
        self.ApplyDocumentLifecycleEffects(&effect);
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

    fn ApplyDocumentLifecycleEffects(&self, effects: &DocumentEffects) {
        // Blink leaves detailed dirty nodes on Document. The effect only admits
        // the relevant lifecycle engines. Attribute changes keep committed
        // geometry until computed-style diffing proves layout changed.
        if effects.layout == Some(DocumentLayoutMutation::InvalidateGeometry) {
            self.InvalidateLayoutOutput();
        }
        if effects.style.is_some() {
            self.InvalidateStylePreservingGeometry();
        }
    }
    fn ApplyAnimationOutput(&self, output: animation::AnimationOutput) {
        for mutation in output.document {
            let effects = self.document.ApplyAnimationMutation(mutation);
            self.ApplyDocumentLifecycleEffects(&effects);
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
        let output = self
            .animation_engine
            .borrow_mut()
            .ApplyMutation(mutation)
            .unwrap_or_else(|_| {
                dom::error::invalid_argument("animation time must be finite and frame ordered")
            });
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
                    compositor::ScrollSourceType::kAbsoluteScroll,
                );
            })),
            write_scroll: Some(Box::new(move |target_node_id, offset| {
                scroll.ProgrammaticAnimations().CancelAll();
                scroll.ApplyMutation(PageMutation::ScrollMutation(
                    page_mutation::ScrollMutation {
                        target_node_id,
                        offset,
                        scroll_type: layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kProgrammatic,
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
                let query = layoutng_assembly::layout_engine::LayoutMetric::try_from(name).ok()?;
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
        if let Some(domain) = self.local_frames.get() {
            local_frame_scroll::InteractionScrollOwner::Root(domain.page.Downgrade())
                .BindBlockingWheelInvalidation(&bindings.borrow());
        }
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
            // Reject mixed lifecycles before copy-on-write can duplicate an old
            // snapshot that the ordinary layout export will replace anyway.
            if !self.document.borrow().GetStyleImpact().IsEmpty() {
                trace.set("impact_rejected", 1.0);
                return false;
            }
            // A DOM/style write initially disqualifies the input batch, but
            // resolving it can prove that no layout or paint input changed.
            // Style resolution and geometry queries never consume paint
            // impact; only a committed Paint does. Re-admit that resolved noop
            // here, without skipping the resource/style flush above or the
            // engine's native NeedsLayout/result-identity checks below.
            let offset_only = self.scroll_only.get()
                || (self.styles_resolved.get() && {
                    let owner = self.document.borrow();
                    let style = owner.GetDocument().StyleState();
                    !style.all_dirty && style.changes.is_empty()
                });
            trace.set("offset_only", offset_only as u8 as f64);
            trace.set(
                "resolved_noop",
                (offset_only && !self.scroll_only.get()) as u8 as f64,
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
            if !self.styles_resolved.get() {
                // Source post-layout snapshot invalidation resumes the same
                // ordinary style/layout lifecycle before PrePaint consumes it.
                frame.fragments = self
                    .layout_engine
                    .borrow_mut()
                    .CommitRetainedPaint(layout_output);
                return false;
            }
            trace.set(
                "retained_geometry",
                (scroll_layout == crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry)
                    as u8 as f64,
            );
            if std::env::var_os("BROWSER_PROFILE_INPUT").is_some()
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
                self.rendering.borrow_mut().InvalidatePreparedArtifact();
            }
            // The frame and Engine reference the same committed artifact.
            // Move the frame's reference back before a property-only update.
            paint.AdoptPaintResult(std::mem::take(&mut frame.display_items));
            let property_update = (scroll_layout
                == crate::persistent_layout::ScrollLayoutUpdate::RetainedGeometry)
                .then(|| paint.TryUpdateScrollProperties(&layout_output));
            reused_recordings = matches!(&property_update, Some(Ok(_)));
            if reused_recordings {
                self.CommitRetainedPaintLifecycle();
            }
            let paint_output = if !reused_recordings {
                trace.set("full_paint", 1.0);
                if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                    if let Some(Err(reason)) = property_update {
                        eprintln!("scroll-paint-reuse rejected={reason:?}");
                    }
                }
                let layout = self.layout_engine.borrow();
                let _svg_resources =
                    layoutng_assembly::internal::layout_pass_scope::SVGResourceQueryScope::new(
                        layout
                            .GetLayoutTree()
                            .expect("scroll Paint retains native SVG resources"),
                    );
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
            let retained_artifact = paint_output.is_none().then(|| {
                paint
                    .GetPaintResult()
                    .expect("paint result available")
                    .clone()
            });
            drop(paint);
            if let Some(paint_output) = paint_output {
                (frame.fragments, frame.display_items) = self
                    .layout_engine
                    .borrow_mut()
                    .CommitPaint(layout_output, paint_output, Some(&self.layout_editing));
                frame.document_image_records_complete =
                    frame.display_items.HasExactDocumentImageContainerRecords();
                self.CountPaintUseCounters(&frame.display_items);
            } else {
                frame.fragments = self
                    .layout_engine
                    .borrow_mut()
                    .CommitRetainedPaint(layout_output);
                frame.display_items = retained_artifact.expect("retained Paint artifact");
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
                    let mut applied = false;
                    if document.EmitScrollOffset(node_id, offset, scroll_type, |mutation| {
                        applied = layout.ApplyMutation(mutation);
                    }) && applied
                    {
                        applied_scrolls += 1;
                    }
                }
            }
            trace.set("mixed_scrolls_applied", applied_scrolls as f64);
            self.document.borrow_mut().InvalidateLayout();
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
                self.constraints.borrow_mut().images.push(image);
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
                    .rendering
                    .borrow_mut()
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
                    .images
                    .iter_mut()
                    .find(|image| image.id == m.frame.id)
                {
                    *existing = m.frame;
                } else {
                    constraints.images.push(m.frame);
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
                    .images
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
                    .images
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
                .frame
                .as_ref()
                .map(|f| f.display_items.recording_revision),
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
        fn collect(
            document: &Document,
            node: PaintFragmentCursor<'_>,
            output: &mut Vec<(usize, f64)>,
        ) {
            if node.kind == FragmentKind::kBox && node.paint.replaced_content.is_some() {
                if let Some(index) = document.FindNodeById(node.node_id) {
                    if document.AllowsResponsiveImageAutoSizes(index) {
                        let zoom = node.paint.effective_zoom.max(f32::MIN_POSITIVE);
                        let width = (node.content_size.width as f32 / zoom).round() as f64;
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
            images: &[layoutng_assembly::internal::layout_input::PaintImage],
            device_pixel_ratio: f64,
            preferred_color_scheme: u8,
            output: &mut BTreeMap<u64, Vec<image_resource::ContainerKey>>,
        ) {
            if let (Some(id), Some(replaced)) = (
                node.paint.style.image_resource_id,
                node.paint.replaced_content.as_ref(),
            ) {
                if let Some(image) = images
                    .iter()
                    .find(|image| image.id == id)
                    .filter(|image| image.IsDocumentImage())
                {
                    let zoom = f64::from(node.paint.effective_zoom).max(f64::MIN_POSITIVE);
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
        fn install_resources(
            node: &mut FragmentPaintData,
            resources: &Arc<PaintResources>,
            catalogs: &mut std::collections::HashMap<*const PaintResources, bool>,
        ) -> bool {
            // Fragment-local paint inputs share one immutable catalog. Compare
            // each distinct prior catalog once, preserving resource identity
            // when it is current, as ImageResourceContent/Font retain theirs.
            let changed = node.paint.resources.as_ref().is_none_or(|previous| {
                if Arc::ptr_eq(previous, resources) {
                    return false;
                }
                *catalogs
                    .entry(Arc::as_ptr(previous))
                    .or_insert_with(|| previous.as_ref() != resources.as_ref())
            });
            if changed {
                node.paint.resources = Some(resources.clone());
            }
            changed
        }

        let preferred_color_scheme = match self.state.preferred_color_scheme.get() {
            PreferredColorScheme::kDark => 1,
            _ => 0,
        };
        let requirements = {
            let constraints = self.state.constraints.borrow();
            let mut requirements = BTreeMap::new();
            collect(
                output.PaintCursor(),
                &constraints.images,
                constraints.device_pixel_ratio,
                preferred_color_scheme,
                &mut requirements,
            );
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
            let mut catalogs = std::collections::HashMap::new();
            let mut rebound = false;
            output.VisitPaintInputsMut(|input| {
                rebound |= install_resources(input, &resources, &mut catalogs)
            });
            trace.set("compared_catalogs", catalogs.len() as f64);
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
        fn install(node: &mut FragmentPaintData, resources: &Arc<PaintResources>) -> bool {
            let rebound = node.paint.resources.as_deref() != Some(resources.as_ref());
            node.paint.resources = Some(resources.clone());
            rebound
        }
        // Retained fragment-local Paint data can still point at the catalog
        // from before an earlier container mutation. Even when the requested
        // variant is already present in Page's current catalog, bind that
        // catalog and re-record once so the immutable artifact publishes the
        // exact record named by its display items.
        let mut rebound = false;
        output.VisitPaintInputsMut(|input| rebound |= install(input, &resources));
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
        let owner = self.state.document.borrow();
        let mut resolver = self.wheel_event_regions.borrow_mut();
        if child_pending || self.state.blocking_wheel_registry_dirty.get() {
            resolver.Invalidate();
        }
        resolver.ResolvePlatformWithLocalFrameTargets(
            frame_sequence,
            self.scripts
                .as_ref()
                .map_or_else(Vec::new, |scripts| scripts.BlockingWheelListenerTargets()),
            child_targets,
            owner.GetDocument(),
            self.frame
                .as_ref()
                .filter(|_| !child_pending && !self.state.blocking_wheel_registry_dirty.get())
                .map(|frame| frame.display_items.as_ref()),
        )
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
            layoutng_assembly::layout_engine::LayoutEngine::new(&crate::CreateLayoutAssembly());
        let paint_engine = paint::PaintEngine::new();
        let rendering = PageRenderPipeline::new_with_executors(
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
            rendering,
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
            begin_frame::PageBeginFrame::WithInputClock(state.rendering.borrow().executor.clone());
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
            .rendering
            .borrow_mut()
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
                .rendering
                .borrow_mut()
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
        tracked: layoutng_assembly::internal::layout_input::TrackedElementSubRect,
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
                .rendering
                .borrow_mut()
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
                .rendering
                .borrow_mut()
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
            .rendering
            .borrow_mut()
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
        self.state
            .rendering
            .borrow_mut()
            .SetExternalPresentation(external);
        self.external_compositor = external;
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
                .rendering
                .borrow()
                .InputTimeForInstant(changed_at);
            self.state
                .ForEachWindowPerformance(|performance| performance.PageHidden(time));
        }
    }

    /// Transfers delegated media frames produced while applying document
    /// resource mutations. The external runtime submits these child Surfaces
    /// immediately before the parent CompositorFrame that references them.
    pub fn TakeChildCompositorFrames(&self) -> Vec<PageChildCompositorFrame> {
        self.state.rendering.borrow_mut().TakeChildFrames()
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
        self.state.RunTextOverflowLayoutInvalidationTasks();
        self.state.local_frames.get().unwrap().RunIntersectionTask();
        self.state.local_frames.get().unwrap().RunFormStateTask();
        self.RunGestureTimeouts()?;
        if self.begin_frame.source.is_none() {
            self.FlushFrameInputs()?;
        }
        self.open_engine.BeginTaskTurn(milliseconds);
        let result = (|| {
            self.state
                .document
                .borrow()
                .GetDocument()
                .FireDueRenderBlockingTimers(std::time::Instant::now());
            let started = std::env::var_os("BROWSER_PROFILE_INPUT")
                .is_some()
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
            self.ProcessFontLoadingEffects();
            let loading_done = started.map(|start| start.elapsed());
            let ran_popup_task = self.RunDuePopupTask(std::time::Instant::now())?;
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
            self.state
                .local_frames
                .get()
                .unwrap()
                .AdvanceNavigations(milliseconds)?;
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
        self.UpdateCursor();
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
                size: layoutng_assembly::internal::layout_input::Size { width, height },
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
        let started = std::env::var_os("BROWSER_PROFILE_INPUT").map(|_| std::time::Instant::now());
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
                    .pending_compositor_mutations
                    .borrow_mut()
                    .push(PageBrowserInput::Gesture(gesture.clone()));
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
                self.RouteCompositorMutation(compositor::CompositorMutation::Input(
                    PageRenderPipeline::GestureInput(gesture)?,
                ))?;
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
        let dispatch_profile = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
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
                compositor_wheel = self.state.rendering.borrow_mut().Wheel(wheel)?;
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
            if let ::interaction::input_event::InputEvent::Wheel(wheel) = &routed_input {
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
                    self.RouteWheelCompositorInput(CompositorInput::WheelDOMAcknowledgement {
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
        self.frame
            .as_ref()
            .and_then(|frame| frame.display_items.caret)
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
                    layoutng_assembly::layout_engine::LayoutMutation::SelectionPaint { node_id },
                );
            }
        }

        let position = if self.active {
            self.state.EnsureLayoutOutput();
            let layout = self.state.layout_engine.borrow();
            let document = self.state.document.borrow();
            layout.Output().and_then(|output| {
                interaction::adapter::page_frame_selection::FrameCaretForSelection(
                    document.GetDocument(),
                    output,
                    &self.state.layout_editing.selections,
                    self.state.interaction.borrow().focused_node_id,
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
                .InvalidateBlockCaretText(&block_text_invalidations);
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
                let order = fragment.paint.logical_tree_order;
                if matches!(
                    fragment.kind,
                    FragmentKind::kText | FragmentKind::kGeneratedText
                ) && order >= start.tree_order
                    && order <= end.tree_order
                    && fragment.node_id != 0
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
                    output.insert(fragment.node_id, state);
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
                    layoutng_assembly::layout_engine::LayoutMutation::SelectionPaint { node_id },
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
    ) -> std::cell::Ref<'_, layoutng_assembly::layout_engine::LayoutEngine> {
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
        self.StartVisibleDeferredImages()?;
        Ok(())
    }

    fn UpdateFrameIfNeeded(&mut self) -> io::Result<()> {
        self.UpdateFrameIfNeededInternal()?;
        self.state.UpdateValidationMessage();
        self.state.local_frames.get().unwrap().UpdateIntersections()
    }
    fn UpdateFrameIfNeededInternal(&mut self) -> io::Result<()> {
        if !self.external_compositor {
            let pending =
                std::mem::take(&mut *self.state.pending_compositor_mutations.borrow_mut());
            for input in pending {
                match input {
                    PageBrowserInput::Compositor(input) => self.RouteCompositorMutation(input)?,
                    PageBrowserInput::Gesture(_) => {
                        unreachable!("external browser input is drained by the embedding")
                    }
                    PageBrowserInput::DocumentDetached { .. } => {
                        unreachable!("external document client detach is drained by the embedding")
                    }
                }
            }
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
        if !defer_caret_for_scroll {
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
        trace.set("renderable_root", renderable_root as u8 as f64);
        if self.frame.is_none()
            && (!document_committed || render_blocking_resources || !renderable_root)
        {
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
        let canvases = self
            .state
            .document
            .borrow()
            .GetDocument()
            .SnapshotCanvasPaintResources();
        self.state.constraints.borrow_mut().canvases = canvases;
        let profile = std::env::var_os("BROWSER_PROFILE_INPUT").map(|_| std::time::Instant::now());
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
        let canvas_update_requires_repaint = self.frame.as_ref().is_some_and(|frame| {
            self.state
                .constraints
                .borrow()
                .canvases
                .iter()
                .any(|canvas| {
                    frame
                        .display_items
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
        let layout_geometry_current = self.state.layout_revision.get() == Some(revision)
            && self.state.layout_engine.borrow().Output().is_some()
            && crate::persistent_layout::MeasurementInputsMatch(
                &self.state.layout_engine.borrow(),
                &mut self.state.document.borrow_mut(),
                &self.state.interaction.borrow(),
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
        if self.frame.is_some()
            && self.PresentedFrameHasCurrentLayoutResult()
            && !self.editing_paint_dirty
            && !self.document_image_environment_dirty
            && layout_geometry_current
            && self.state.document.borrow().GetStyleImpact().IsEmpty()
            && !paint_resource_update_requires_repaint
        {
            trace.set("frame_retained", 1.0);
            self.state.CommitRetainedPaintLifecycle();
            if self.state.paint_resources_dirty.get() {
                self.state
                    .PublishPaintResources(self.frame.as_mut().unwrap());
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
            self.state.client.borrow_mut().DidPresentFrame(frame);
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
                    crate::persistent_layout::PopulatePaintDataWithEngine(
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
        // PopulatePaintDataWithEngine proves that the resident LayoutResult is
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
                self.state.PublishPaintResources(frame);
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
            self.state.client.borrow_mut().DidPresentFrame(frame);
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
        if self.ConfigureDocumentImageContainers(&mut layout_output)? {
            self.state
                .layout_engine
                .borrow_mut()
                .RefreshOutputPrePaintInputs(&mut layout_output);
        }
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
        let paint_output = {
            trace.set("full_paint", 1.0);
            {
                let layout = self.state.layout_engine.borrow();
                let _layout_paint_scope = layout.PaintScope();
                let _svg_resources =
                    layoutng_assembly::internal::layout_pass_scope::SVGResourceQueryScope::new(
                        layout
                            .GetLayoutTree()
                            .expect("Paint retains native SVG resource owners"),
                    );
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
                self.state
                    .paint_engine
                    .borrow_mut()
                    .PaintWithLocalFrameViews(
                        &layout_output,
                        &views,
                        self.state.layout_editing.PaintState(),
                        &lifecycle,
                        document,
                    );
            }
            let document_image_requirements = self
                .state
                .paint_engine
                .borrow()
                .GetPaintResult()
                .expect("successful paint has artifact")
                .DocumentImageContainerRequirements();
            if self.ConfigurePaintDocumentImageContainers(
                &mut layout_output,
                document_image_requirements,
            )? {
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
                    layoutng_assembly::internal::layout_pass_scope::SVGResourceQueryScope::new(
                        layout
                            .GetLayoutTree()
                            .expect("Paint retains native SVG resource owners"),
                    );
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
            paint.CommitPaintResult()
        };
        self.state
            .local_frames
            .get()
            .unwrap()
            .CommitPaint(child_paint, &paint_output);
        let (fragments, display_items) = self.state.layout_engine.borrow_mut().CommitPaint(
            layout_output,
            paint_output,
            Some(&self.state.layout_editing),
        );
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
        let document_image_records_complete = display_items.HasExactDocumentImageContainerRecords();
        self.state.CountPaintUseCounters(&display_items);
        let frame = PageFrame {
            compositor_scroll_sequence: self.processed_compositor_scroll_sequence,
            sequence: self
                .frame
                .as_ref()
                .map_or(self.previous_document_frame_sequence + 1, |f| {
                    f.sequence + 1
                }),
            fragments,
            display_items,
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
        self.state.document.borrow_mut().DidCommitPaint();
        self.state.paint_resources_dirty.set(false);
        self.state.pending_paint_mutations.borrow_mut().clear();
        self.state
            .ApplyAnimationMutation(animation::AnimationMutation::CommitPaint);
        self.state.scroll_only.set(false);
        self.state.dirty.set(false);
        if let Some(scripts) = &mut self.scripts {
            scripts.DidPaint();
        }
        self.state
            .client
            .borrow_mut()
            .DidPresentFrame(self.frame.as_ref().unwrap());
        trace.set("presented", 1.0);
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
        let assembly = crate::CreateLayoutAssembly();
        let mut page = Page::Create(
            Rc::new(RefCell::new(NoNetwork)),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(crate::CreateSVGImageDecoder(&assembly))),
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
                    layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kProgrammatic,
            },
        ));
        page.UpdateFrameIfNeeded().unwrap();
        assert!(page.CurrentFrame().unwrap().sequence > before);
        fn scroll_offset(fragment: PaintFragmentCursor<'_>, id: u64) -> Option<Offset> {
            if fragment.node_id == id {
                return Some(fragment.paint.scroll_offset);
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
    let assembly = crate::CreateLayoutAssembly();
    let mut page = Page::Create(
        loader,
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(crate::CreateSVGImageDecoder(&assembly))),
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
        let artifact = &self
            .CurrentFrame()
            .ok_or_else(|| io::Error::other("Page has no frame"))?
            .display_items;
        self.PaintArtifactInto(artifact, width, height, scale, target, format, stride)
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
            .rendering
            .borrow_mut()
            .Render(artifact, width, height, scale, target, format, stride)
    }
}

mod resident_editing_positions;
